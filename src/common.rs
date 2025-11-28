use std::collections::{HashMap, HashSet};
use walrus::ElementItems;
use walrus::{ir::Instr, FunctionId};

const WASI_UNSTABLE: &str = "wasi_unstable";
const WASI_SNAPSHOT_PREVIEW1: &str = "wasi_snapshot_preview1";

fn get_replacement_module_id(
    module: &walrus::Module,
    module_name: &str,
    import_name: &str,
    fn_id: FunctionId,
) -> Option<FunctionId> {
    // we only support wasi_unstable and wasi_snapshot_preview1 modules
    if module_name != WASI_UNSTABLE && module_name != WASI_SNAPSHOT_PREVIEW1 {
        return None;
    }

    let searched_function_name = format!("__ic_custom_{import_name}");

    // 1) Search by function name
    for fun in module.funcs.iter() {
        if let Some(name) = &fun.name {
            if *name == searched_function_name {
                let original_ty = module.funcs.get(fn_id).ty();
                let replacement_ty = module.funcs.get(fun.id()).ty();

                if original_ty != replacement_ty {
                    log::error!(
                        "Type mismatch for replacement {}::{}: original {:?}, replacement {:?}",
                        module_name,
                        import_name,
                        original_ty,
                        replacement_ty
                    );
                    return None;
                }

                if matches!(module.funcs.get(fun.id()).kind, walrus::FunctionKind::Import(_)) {
                    log::error!(
                        "Replacement function {} must not be an imported function",
                        searched_function_name
                    );
                    return None;
                }

                log::debug!(
                    "Function replacement found: {:?} -> {:?}.",
                    module.funcs.get(fn_id).name,
                    module.funcs.get(fun.id()).name
                );

                return Some(fun.id());
            }
        }
    }

    // 2) additionally try searching exports, in case the original function was renamed
    for export in module.exports.iter() {
        if export.name != searched_function_name {
            continue;
        }

        match export.item {
            walrus::ExportItem::Function(exported_function) => {
                let original_ty = module.funcs.get(fn_id).ty();
                let replacement_ty = module.funcs.get(exported_function).ty();

                if original_ty != replacement_ty {
                    log::error!(
                        "Type mismatch for exported replacement {}::{}: original {:?}, replacement {:?}",
                        module_name,
                        import_name,
                        original_ty,
                        replacement_ty
                    );
                    return None;
                }

                if matches!(
                    module.funcs.get(exported_function).kind,
                    walrus::FunctionKind::Import(_)
                ) {
                    log::error!(
                        "Exported replacement function {} must not be an imported function",
                        searched_function_name
                    );
                    return None;
                }

                log::debug!(
                    "Function replacement found in exports: {:?} -> {:?}.",
                    module.funcs.get(fn_id).name,
                    module.funcs.get(exported_function).name
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
    // First, patch element segments
    for elem in m.elements.iter_mut() {
        match &mut elem.items {
            ElementItems::Functions(function_ids) => {
                for func_id in function_ids.iter_mut() {
                    if let Some(&new_id) = fn_replacement_ids.get(func_id) {
                        log::debug!(
                            "Replace func id in element: old ID: {:?}, new ID {:?}",
                            func_id,
                            new_id
                        );

                        *func_id = new_id;
                    }
                }
            }
            ElementItems::Expressions(_, _) => {
                // Expressions do not contain FunctionId directly here.
            }
        }
    }

    // Then, replace dependent calls in function bodies
    for fun in m.funcs.iter_mut() {
        log::debug!("Processing function `{:?}`", fun.name);

        match &mut fun.kind {
            walrus::FunctionKind::Import(_import_fun) => {
                // nothing to rewrite in imported bodies
            }

            walrus::FunctionKind::Local(local_fun) => {
                let block_id: walrus::ir::InstrSeqId = local_fun.entry_block();
                let mut visited: HashSet<walrus::ir::InstrSeqId> = HashSet::new();
                replace_calls_in_instructions(block_id, fn_replacement_ids, local_fun, &mut visited);
            }

            walrus::FunctionKind::Uninitialized(_) => {}
        }
    }
}

fn replace_calls_in_instructions(
    block_id: walrus::ir::InstrSeqId,
    fn_replacement_ids: &HashMap<FunctionId, FunctionId>,
    local_fun: &mut walrus::LocalFunction,
    visited: &mut HashSet<walrus::ir::InstrSeqId>,
) {
    if !visited.insert(block_id) {
        // already visited this block in this function; avoid infinite recursion
        return;
    }

    log::debug!("Entering block {block_id:?}");

    let instructions = &mut local_fun.block_mut(block_id).instrs;

    let mut block_ids = Vec::new();

    for (ins, _location) in instructions.iter_mut() {
        match ins {
            Instr::RefFunc(ref_func_inst) => {
                if let Some(&new_id) = fn_replacement_ids.get(&ref_func_inst.func) {
                    log::debug!(
                        "Replace ref_func old ID: {:?}, new ID {:?}",
                        ref_func_inst.func,
                        new_id
                    );
                    ref_func_inst.func = new_id;
                }
            }
            Instr::Call(call_inst) => {
                if let Some(&new_id) = fn_replacement_ids.get(&call_inst.func) {
                    log::debug!(
                        "Replace function call: old ID: {:?}, new ID {:?}",
                        call_inst.func,
                        new_id
                    );
                    call_inst.func = new_id;
                }
            }
            Instr::ReturnCall(call_inst) => {
                if let Some(&new_id) = fn_replacement_ids.get(&call_inst.func) {
                    log::debug!(
                        "Replace function return call: old ID: {:?}, new ID {:?}",
                        call_inst.func,
                        new_id
                    );
                    call_inst.func = new_id;
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
            // | Instr::TernOp(_) // will be needed with walrus version 0.23 or later
            | Instr::ReturnCallIndirect(_) => {
            }
        }
    }

    // process additional instructions inside nested blocks
    for nested_block_id in block_ids {
        replace_calls_in_instructions(nested_block_id, fn_replacement_ids, local_fun, visited)
    }
}

pub(crate) fn add_start_entry(module: &mut walrus::Module) {
    // try to find the start (_initialize) function
    let initialize_function = module.funcs.by_name("_initialize");
    log::info!("_initialize function found: {initialize_function:?}");

    if let Some(initialize) = initialize_function {
        if module.start.is_none() {
            log::info!("Setting module start function to _initialize");
            module.start = Some(initialize);
        } else {
            log::debug!("Module already has a start function; leaving it unchanged");
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

        if let walrus::ExportItem::Function(_) = export.item {
            export_found = Some(export.id());
            break;
        }
    }

    // remove export, if it was found
    if let Some(export_id) = export_found {
        log::debug!("Removing _initialize export");
        module.exports.delete(export_id);
    }
}

pub(crate) fn do_module_replacements(module: &mut walrus::Module) -> bool {
    // find corresponding IDs for replacements
    let fn_replacement_ids = gather_replacement_ids(module);

    if fn_replacement_ids.is_empty() {
        // do not modify module, if there are no functions to rewire
        log::debug!("No WASI imports to replace; leaving module unchanged");
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

pub(crate) fn get_module_imports(module: &walrus::Module) -> Vec<(String, String)> {
    let mut module_imports: Vec<(String, String)> = Vec::new();

    for imp in module.imports.iter() {
        match imp.kind {
            walrus::ImportKind::Function(_fn_id) => {
                module_imports.push((
                    imp.module.as_str().to_string(),
                    imp.name.as_str().to_string(),
                ));
            }

            walrus::ImportKind::Table(_)
            | walrus::ImportKind::Memory(_)
            | walrus::ImportKind::Global(_) => {}
        }
    }

    module_imports
}
