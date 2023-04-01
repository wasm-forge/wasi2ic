use walrus::{ir::Instr, FunctionId};
use std::collections::HashMap;
use log;

fn get_replacement_module_id(module: &walrus::Module, module_name: &str, fn_name: &str) -> Option<FunctionId> {

    // for now we only support wasi_unstable and wasi_snapshot_preview1
    if module_name != "wasi_unstable" && module_name != "wasi_snapshot_preview1" {
        return None;
    }

    let searched_function_name = format!("__ic_custom_{}", fn_name);

    for fun in module.funcs.iter() {

        if let Some(name) = &fun.name {

            if name.starts_with(searched_function_name.as_str()) {
                return Some(fun.id());
            }
        }
    }

    None
}

fn gather_replacement_ids(m: &walrus::Module) -> HashMap<FunctionId, FunctionId> {

    // gather functions for replacements
    let mut fn_replacement_ids: HashMap<FunctionId, FunctionId> = HashMap::new();

    for imp in m.imports.iter() {

        match imp.kind {
        
            walrus::ImportKind::Function(fn_id) => {
                
                let replace_id = get_replacement_module_id(
                    m, imp.module.as_str(), imp.name.as_str());
                
                if let Some(rep_id) = replace_id {
                    fn_replacement_ids.insert(fn_id, rep_id);
                }

            },

            walrus::ImportKind::Table(_) => todo!(),
            
            walrus::ImportKind::Memory(_) => {},
            walrus::ImportKind::Global(_) => {},
        }
    }

    log::debug!("Gathered replacement IDs {:?}", fn_replacement_ids);

    fn_replacement_ids
}


fn replace_calls(m: &mut walrus::Module, fn_replacement_ids: &HashMap<FunctionId, FunctionId>) {
    // replace dependent calls
    for fun in m.funcs.iter_mut() {

        log::debug!("Processing function `{:?}`", fun.name);
    
        match &mut fun.kind {

            walrus::FunctionKind::Import(_import_fun) => {

            },

            walrus::FunctionKind::Local(local_fun) => {

                let block_id: walrus::ir::InstrSeqId = local_fun.entry_block();

                replace_calls_in_instructions(block_id, fn_replacement_ids, local_fun);

            },

            walrus::FunctionKind::Uninitialized(_) => {},
        }
    }
}


fn replace_calls_in_instructions(block_id: walrus::ir::InstrSeqId, fn_replacement_ids: &HashMap<FunctionId, FunctionId>, local_fun: &mut walrus::LocalFunction) {
    let instructions = &mut local_fun.block_mut(block_id).instrs;
    
    let mut block_ids = vec![];

    for (ins, _location) in instructions.iter_mut() {

        match ins {
            Instr::RefFunc(ref_func_inst) => {
                let new_id_opt = fn_replacement_ids.get(&ref_func_inst.func);

                if let Some(new_id) = new_id_opt {
    
                    log::debug!("Replace ref_func old ID: {:?}, new ID {:?}", ref_func_inst.func, *new_id);
    
                    ref_func_inst.func = *new_id;
            
                }
            },
            Instr::Call(call_inst) => {
                let new_id_opt = fn_replacement_ids.get(&call_inst.func);

                if let Some(new_id) = new_id_opt {
    
                    log::debug!("Replace function call old ID: {:?}, new ID {:?}", call_inst.func, *new_id);
    
                    call_inst.func = *new_id;
            
                }
            },

            Instr::Block(block_ins) => {
                block_ids.push(block_ins.seq);
            },

            Instr::Loop(loop_ins) => {
                block_ids.push(loop_ins.seq);
            },

            Instr::IfElse(if_else) => {
                block_ids.push(if_else.consequent);
                block_ids.push(if_else.alternative);
            },

            Instr::CallIndirect(_) => {},
            Instr::LocalGet(_) => {},
            Instr::LocalSet(_) => {},
            Instr::LocalTee(_) => {},
            Instr::GlobalGet(_) => {},
            Instr::GlobalSet(_) => {},
            Instr::Const(_) => {},
            Instr::Binop(_) => {},
            Instr::Unop(_) => {},
            Instr::Select(_) => {},
            Instr::Unreachable(_) => {},
            Instr::Br(_) => {},
            Instr::BrIf(_) => {},
            Instr::BrTable(_) => {},
            Instr::Drop(_) => {},
            Instr::Return(_) => {},
            Instr::MemorySize(_) => {},
            Instr::MemoryGrow(_) => {},
            Instr::MemoryInit(_) => {},
            Instr::DataDrop(_) => {},
            Instr::MemoryCopy(_) => {},
            Instr::MemoryFill(_) => {},
            Instr::Load(_) => {},
            Instr::Store(_) => {},
            Instr::AtomicRmw(_) => {},
            Instr::Cmpxchg(_) => {},
            Instr::AtomicNotify(_) => {},
            Instr::AtomicWait(_) => {},
            Instr::AtomicFence(_) => {},
            Instr::TableGet(_) => {},
            Instr::TableSet(_) => {},
            Instr::TableGrow(_) => {},
            Instr::TableSize(_) => {},
            Instr::TableFill(_) => {},
            Instr::RefNull(_) => {},
            Instr::RefIsNull(_) => {},
            Instr::V128Bitselect(_) => {},
            Instr::I8x16Swizzle(_) => {},
            Instr::I8x16Shuffle(_) => {},
            Instr::LoadSimd(_) => {},
            Instr::TableInit(_) => {},
            Instr::ElemDrop(_) => {},
            Instr::TableCopy(_) => {},
        }
    }

    // process additional instructins inside blocks
    for block_id in block_ids {
        replace_calls_in_instructions(block_id, fn_replacement_ids, local_fun)
    }

}


fn main() -> anyhow::Result<()> {

    let exe_name = std::env::current_exe().unwrap().file_name().unwrap().to_str().unwrap().to_owned();

    env_logger::init();

    let input_wasm = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("The launch parameters are incorrect, try: {} <input.wasm> [output.wasm]", {exe_name}))?;

    // load Wasm module from file
    let mut module = walrus::Module::from_file(&input_wasm)?;
    
    // find corresponding IDs for replacements
    let fn_replacement_ids = gather_replacement_ids(&module);

    // do recursive call replacement
    replace_calls(&mut module, &fn_replacement_ids);

    // clean-up unused imports
    walrus::passes::gc::run(&mut module);

    // try store as binary
    let wasm = module.emit_wasm();

    // store binary representation into file
    if let Some(output_wasm) = std::env::args().nth(2) {
         std::fs::write(output_wasm, wasm)?;
    }


    Ok(())
}
