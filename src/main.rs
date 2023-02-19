use walrus::{ir::Instr, FunctionId, ImportId};
use std::{collections::{HashMap, HashSet}, f64::consts::PI};
use log;

fn get_replacement_module_id(module: &walrus::Module, module_name: &str, fn_name: &str, source_id: FunctionId) -> Option<FunctionId> {

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
                    m, imp.module.as_str(), imp.name.as_str(), fn_id);

                if let Some(rep_id) = replace_id {
                    fn_replacement_ids.insert(fn_id, rep_id);
                }
            },

            walrus::ImportKind::Table(_) => todo!(),
            walrus::ImportKind::Memory(_) => todo!(),
            walrus::ImportKind::Global(_) => todo!(),
        }
    }

    println!("Gathered replacement IDs {:?}", fn_replacement_ids);

    fn_replacement_ids
}


fn replace_calls(m: &mut walrus::Module, fn_replacement_ids: &HashMap<FunctionId, FunctionId>) {
    // replace dependent calls
    for fun in m.funcs.iter_mut() {

        log::debug!("Processing function `{:?}`", fun.name);
    
        match &mut fun.kind {

            walrus::FunctionKind::Import(imp_fun) => {
            
                // println!("Imported function type: {:?}", m.types.get(imp_fun.ty));

            },

            walrus::FunctionKind::Local(local_fun) => {

                // println!("Local function type: {:?}", m.types.get(local_fun.ty()));

                let entry = local_fun.entry_block();
            
                for (ins, _location) in local_fun.block_mut(entry).instrs.iter_mut() {



                    if let Instr::RefFunc(ref_func_inst) = ins {

                        let new_id_opt = fn_replacement_ids.get(&ref_func_inst.func);

                        if let Some(new_id) = new_id_opt {

                            log::debug!("Replace ref_func old ID: {:?}, new ID {:?}", ref_func_inst.func, *new_id);

                            ref_func_inst.func = *new_id;
                        
                        }
                    }

                    if let Instr::Call(call_inst) = ins {

                        let new_id_opt = fn_replacement_ids.get(&call_inst.func);

                        if let Some(new_id) = new_id_opt {

                            log::debug!("Replace function call old ID: {:?}, new ID {:?}", call_inst.func, *new_id);

                            call_inst.func = *new_id;
                        
                        }
                    }
                }
            },
            walrus::FunctionKind::Uninitialized(_) => {},
        }
    }
}


fn remove_imports(m: &mut walrus::Module, fn_replacement_ids: &HashMap<FunctionId, FunctionId>) {

    // remove imports
    let mut to_remove: HashSet<ImportId> = HashSet::new();

    for imp in m.imports.iter() {

        let import_id = imp.id();

        match imp.kind {
        
            walrus::ImportKind::Function(fun_id) => {

                if fn_replacement_ids.get(&fun_id).is_some() {

                    to_remove.insert(import_id);
                    
    //                m.funcs.delete(fun_id);
                }
            },

            walrus::ImportKind::Table(_) => todo!(),
            walrus::ImportKind::Memory(_) => todo!(),
            walrus::ImportKind::Global(_) => todo!(),
        }
    }

  /* 
    for import_id in to_remove {
        println!("Removing {:?}", import_id);
        m.imports.delete(import_id);
    }
    */

    walrus::passes::gc::run(m);

    

}

fn main() -> anyhow::Result<()> {

    env_logger::init();

    let a = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("Usage: cleanwasi <input.wasm> [output.wasm]"))?;

    let mut m = walrus::Module::from_file(&a)?;
    
    let fn_replacement_ids = gather_replacement_ids(&m);

    replace_calls(&mut m, &fn_replacement_ids);

    remove_imports(&mut m, &fn_replacement_ids);

    // store 
    let wasm = m.emit_wasm();

    if let Some(destination) = std::env::args().nth(2) {
         std::fs::write(destination, wasm)?;
    }



    Ok(())
}
