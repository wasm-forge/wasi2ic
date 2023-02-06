use walrus::{ir::Instr, FunctionId};
use std::collections::HashMap;


fn get_replacement_module_id(module: &walrus::Module, module_name: &str, fn_name: &str, source_id: FunctionId) -> Option<FunctionId> {

    if module_name != "wasi_unstable" {
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

    fn_replacement_ids
}


fn main() -> anyhow::Result<()> {
    println!("Hello, walrus!");

    env_logger::init();

    let a = std::env::args()
        .nth(1)
        .ok_or_else(|| anyhow::anyhow!("provide wasm file as the first input argument"))?;

    let mut m = walrus::Module::from_file(&a)?;


    for imp in m.imports.iter() {
        println!("{:?}", imp);
    }

    
    let fn_replacement_ids = gather_replacement_ids(&m);

    // replace dependent calls
    for fun in m.funcs.iter_mut() {
        
        println!("\n\n======= {:?} id = {:?}", fun.name, fun.id());

        match &mut fun.kind {

            walrus::FunctionKind::Import(imp_fun) => {
                
                // println!("Imported function type: {:?}", m.types.get(imp_fun.ty));
            },

            walrus::FunctionKind::Local(local_fun) => {

                // println!("Local function type: {:?}", m.types.get(local_fun.ty()));

                let entry = local_fun.entry_block();
                
                for (ins, _location) in local_fun.block_mut(entry).instrs.iter_mut() {

                    if let Instr::Call(call_inst) = ins {

                        let new_id_opt = fn_replacement_ids.get(&call_inst.func);

                        if let Some(new_id) = new_id_opt {

                            call_inst.func = *new_id;
                            
                        }
                    }
                }
            },
            walrus::FunctionKind::Uninitialized(_) => {},
        }
    }


    // remove imports


    // let wasm = m.emit_wasm();

    // if let Some(destination) = std::env::args().nth(2) {
    //     std::fs::write(destination, wasm)?;
    // }

    println!("Done!");    
    Ok(())

}
