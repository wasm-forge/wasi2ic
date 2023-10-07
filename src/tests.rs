
use crate::*;

#[warn(dead_code)]
fn print_wasm(module: &mut walrus::Module) {
    let wasm = module.emit_wasm();
    let text = wasmprinter::print_bytes(wasm).unwrap();

    println!("{}", text);
}

#[test]
fn test_add_start_entry() {

    let wat = r#"
        (module
            (func $add (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add
            )

            (func $_start
            i32.const 2
            i32.const 3
            call $add
            i32.const 5
            i32.const 7
            call $add
            drop
            drop
            )
        
        )
    "#;

    let binary = wat::parse_str(wat).unwrap();
    let mut module = walrus::Module::from_buffer(&binary).unwrap();

    assert!(module.start.is_none());

    add_start_entry(&mut module);

    assert!(module.start.is_some());
}

#[test]
fn test_remove_start_export() {

    let wat = r#"
        (module
            (func $add (param i32 i32) (result i32)
            local.get 0
            local.get 1
            i32.add
            )

            (func $_start
            i32.const 2
            i32.const 3
            call $add
            i32.const 5
            i32.const 7
            call $add
            drop
            drop
            )

            (export "_start" (func $_start))
        )
    "#;

    let binary = wat::parse_str(wat).unwrap();
    let mut module = walrus::Module::from_buffer(&binary).unwrap();

    let mut export_found: Option<walrus::ExportId> = None;

    // try to find the start export
    for export in module.exports.iter() {
        if !export.name.starts_with("_start") {
            continue;
        }

        match export.item {
            walrus::ExportItem::Function(_) => {
                export_found = Some(export.id());
            },
            _ => {},
        }
    }  

    assert!(export_found.is_some());

    remove_start_export(&mut module);

    let mut export_found: Option<walrus::ExportId> = None;
    // try to find the start export
    for export in module.exports.iter() {
        if !export.name.starts_with("_start") {
            continue;
        }

        match export.item {
            walrus::ExportItem::Function(_) => {
                export_found = Some(export.id());
            },
            _ => {},
        }
    }

    assert!(None == export_found);

}
