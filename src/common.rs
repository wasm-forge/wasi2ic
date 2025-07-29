use std::collections::HashMap;
use walrus::ElementItems;
use walrus::{ir::Instr, FunctionId};

fn get_replacement_module_id(
    module: &walrus::Module,
    module_name: &str,
    import_name: &str,
    fn_id: FunctionId,
) -> Option<FunctionId> {
    // we only support wasi_unstable and wasi_snapshot_preview1 modules
    if module_name != "wasi_unstable" && module_name != "wasi_snapshot_preview1" {
        return None;
    }

    let searched_function_name = format!("__ic_custom_{import_name}");

    for fun in module.funcs.iter() {
        if let Some(name) = &fun.name {
            if *name == searched_function_name {
                log::debug!(
                    "Function replacement found: {:?} -> {:?}.",
                    module.funcs.get(fn_id).name,
                    module.funcs.get(fun.id()).name
                );

                assert_eq!(
                    module.funcs.get(fn_id).ty(),
                    module.funcs.get(fun.id()).ty()
                );

                return Some(fun.id());
            }
        }
    }

    // additionally try searching exports, in case the original function was renamed
    for export in module.exports.iter() {
        if export.name != searched_function_name {
            continue;
        }

        match export.item {
            walrus::ExportItem::Function(exported_function) => {
                log::debug!(
                    "Function replacement found in exports: {:?} -> {:?}.",
                    module.funcs.get(fn_id).name,
                    module.funcs.get(exported_function).name
                );

                assert_eq!(
                    module.funcs.get(fn_id).ty(),
                    module.funcs.get(exported_function).ty()
                );

                return Some(exported_function);
            }
            walrus::ExportItem::Table(_)
            | walrus::ExportItem::Memory(_)
            | walrus::ExportItem::Global(_) => {}
        }
    }

    log::warn!(
        "Could not find the replacement for the WASI function: {module_name}::{import_name}"
    );

    None
}

pub(crate) fn gather_replacement_ids(m: &walrus::Module) -> HashMap<FunctionId, FunctionId> {
    // gather functions for replacements
    let mut fn_replacement_ids: HashMap<FunctionId, FunctionId> = HashMap::new();

    for imp in m.imports.iter() {
        match imp.kind {
            walrus::ImportKind::Function(fn_id) => {
                let replace_id =
                    get_replacement_module_id(m, imp.module.as_str(), imp.name.as_str(), fn_id);

                if let Some(rep_id) = replace_id {
                    fn_replacement_ids.insert(fn_id, rep_id);
                }
            }

            walrus::ImportKind::Table(_)
            | walrus::ImportKind::Memory(_)
            | walrus::ImportKind::Global(_) => {}
        }
    }

    log::debug!("Gathered replacement IDs {fn_replacement_ids:?}");

    fn_replacement_ids
}

fn replace_calls(m: &mut walrus::Module, fn_replacement_ids: &HashMap<FunctionId, FunctionId>) {
    for elem in m.elements.iter_mut() {
        match &mut elem.items {
            ElementItems::Functions(function_ids) => {
                for func_id in function_ids.iter_mut() {
                    if let Some(new_id) = fn_replacement_ids.get(func_id) {
                        log::debug!(
                            "Replace func id old ID: {:?}, new ID {:?}",
                            func_id,
                            *new_id
                        );

                        *func_id = *new_id;
                    }
                }
            }
            &mut ElementItems::Expressions(_, _) => {}
        }
    }

    // replace dependent calls
    for fun in m.funcs.iter_mut() {
        log::debug!("Processing function `{:?}`", fun.name);

        match &mut fun.kind {
            walrus::FunctionKind::Import(_import_fun) => {}

            walrus::FunctionKind::Local(local_fun) => {
                let block_id: walrus::ir::InstrSeqId = local_fun.entry_block();
                replace_calls_in_instructions(block_id, fn_replacement_ids, local_fun);
            }

            walrus::FunctionKind::Uninitialized(_) => {}
        }
    }
}

fn replace_calls_in_instructions(
    block_id: walrus::ir::InstrSeqId,
    fn_replacement_ids: &HashMap<FunctionId, FunctionId>,
    local_fun: &mut walrus::LocalFunction,
) {
    log::debug!("Entering block {block_id:?}");

    let instructions = &mut local_fun.block_mut(block_id).instrs;

    let mut block_ids = vec![];

    for (ins, _location) in instructions.iter_mut() {
        match ins {
            Instr::RefFunc(ref_func_inst) => {
                let new_id_opt = fn_replacement_ids.get(&ref_func_inst.func);

                if let Some(new_id) = new_id_opt {
                    log::debug!(
                        "Replace ref_func old ID: {:?}, new ID {:?}",
                        ref_func_inst.func,
                        *new_id
                    );

                    ref_func_inst.func = *new_id;
                }
            }
            Instr::Call(call_inst) => {
                let new_id_opt = fn_replacement_ids.get(&call_inst.func);

                if let Some(new_id) = new_id_opt {
                    log::debug!(
                        "Replace function call: old ID: {:?}, new ID {:?}",
                        call_inst.func,
                        *new_id
                    );

                    call_inst.func = *new_id;
                }
            }
            Instr::ReturnCall(call_inst) => {
                let new_id_opt = fn_replacement_ids.get(&call_inst.func);

                if let Some(new_id) = new_id_opt {
                    log::debug!(
                        "Replace function return call: old ID: {:?}, new ID {:?}",
                        call_inst.func,
                        *new_id
                    );

                    call_inst.func = *new_id;
                }
            }
            Instr::Block(block_ins) => {
                block_ids.push(block_ins.seq);
            }
            Instr::Loop(loop_ins) => {
                block_ids.push(loop_ins.seq);
            }
            Instr::IfElse(if_else) => {
                block_ids.push(if_else.consequent);
                block_ids.push(if_else.alternative);
            }
            Instr::CallIndirect(_)
            | Instr::LocalGet(_)
            | Instr::LocalSet(_)
            | Instr::LocalTee(_)
            | Instr::GlobalGet(_)
            | Instr::GlobalSet(_)
            | Instr::Const(_)
            | Instr::Binop(_)
            | Instr::Unop(_)
            | Instr::Select(_)
            | Instr::Unreachable(_)
            | Instr::Br(_)
            | Instr::BrIf(_)
            | Instr::BrTable(_)
            | Instr::Drop(_)
            | Instr::Return(_)
            | Instr::MemorySize(_)
            | Instr::MemoryGrow(_)
            | Instr::MemoryInit(_)
            | Instr::DataDrop(_)
            | Instr::MemoryCopy(_)
            | Instr::MemoryFill(_)
            | Instr::Load(_)
            | Instr::Store(_)
            | Instr::AtomicRmw(_)
            | Instr::Cmpxchg(_)
            | Instr::AtomicNotify(_)
            | Instr::AtomicWait(_)
            | Instr::AtomicFence(_)
            | Instr::TableGet(_)
            | Instr::TableSet(_)
            | Instr::TableGrow(_)
            | Instr::TableSize(_)
            | Instr::TableFill(_)
            | Instr::RefNull(_)
            | Instr::RefIsNull(_)
            | Instr::V128Bitselect(_)
            | Instr::I8x16Swizzle(_)
            | Instr::I8x16Shuffle(_)
            | Instr::LoadSimd(_)
            | Instr::TableInit(_)
            | Instr::ElemDrop(_)
            | Instr::TableCopy(_)
            //| Instr::TernOp(_) // will be needed with walrus version 0.23 or later
            | Instr::ReturnCallIndirect(_) => {} 

            _ => {}
        }
    }

    // process additional instructins inside blocks
    for block_id in block_ids {
        replace_calls_in_instructions(block_id, fn_replacement_ids, local_fun)
    }
}

pub(crate) fn add_start_entry(module: &mut walrus::Module) {
    // try to find the start (_initialize) function
    let initialize_function = module.funcs.by_name("_initialize");
    log::info!("_initialize function found: {initialize_function:?}\n");

    if let Some(initialize) = initialize_function {
        if module.start.is_none() {
            module.start = Some(initialize);
        }
    }
}

pub(crate) fn remove_start_export(module: &mut walrus::Module) {
    let mut export_found: Option<walrus::ExportId> = None;

    // try to find the start export
    for export in module.exports.iter() {
        if !export.name.starts_with("_initialize") {
            continue;
        }

        match export.item {
            walrus::ExportItem::Function(_) => {
                export_found = Some(export.id());
            }
            walrus::ExportItem::Table(_)
            | walrus::ExportItem::Memory(_)
            | walrus::ExportItem::Global(_) => {}
        }
    }

    // remove export, if it was found
    if let Some(export_id) = export_found {
        module.exports.delete(export_id);
    }
}

pub(crate) fn do_module_replacements(module: &mut walrus::Module) -> bool {
    // find corresponding IDs for replacements
    let fn_replacement_ids = gather_replacement_ids(module);

    if fn_replacement_ids.is_empty() {
        // do not modify module, if there are no functions to rewire
        return false;
    }

    // do recursive call replacement
    replace_calls(module, &fn_replacement_ids);

    // add _initialize entry (this is needed to do initialization)
    add_start_entry(module);

    // remove the _initialize export to clean up the module exports
    remove_start_export(module);

    // clean-up unused imports
    walrus::passes::gc::run(module);

    true
}
