
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



#[test]
fn test_gather_replacement_ids() {

    let wat = r#"
    (module
        (type (;0;) (func))
        (type (;1;) (func (param i32)))
        (type (;2;) (func (param i32 i32)))
        (type (;3;) (func (param i32 i32) (result i32)))
        (type (;4;) (func (param i32 i32 i32)))
        (type (;5;) (func (param i32 i32 i32 i32) (result i32)))
        (import "ic0" "debug_print" (func $_dprint (;0;) (type 2)))
        (import "ic0" "msg_reply" (func $_msg_reply (;1;) (type 0)))
        (import "wasi_snapshot_preview1" "fd_write" (func $_wasi_snapshot_preview_fd_write (;2;) (type 5)))
        (import "wasi_snapshot_preview1" "random_get" (func $_wasi_snapshot_preview_random_get (;3;) (type 3)))
        (import "wasi_snapshot_preview1" "environ_get" (func $__imported_wasi_snapshot_preview1_environ_get (;4;) (type 3)))
        (import "wasi_snapshot_preview1" "proc_exit" (func $__imported_wasi_snapshot_preview1_proc_exit (;5;) (type 1)))

        (func $_start (;6;) (type 0)
          i32.const 1
          i32.const 2
          call $__ic_custom_random_get
          i32.const 1
          i32.const 2
          call $_wasi_snapshot_preview_random_get
          i32.const 4
          i32.const 5
          call $__ic_custom_fd_write
          drop
        )

        (func $__ic_custom_random_get (;7;) (type 3) (param i32 i32) (result i32)
          call $_msg_reply
          i32.const 421
        )

        (func $__ic_custom_fd_write (;8;) (type 5) (param i32 i32 i32 i32) (result i32)
          i32.const 0
          i32.const 0
          call $_dprint
          i32.const 42
        )

        (export "_start" (func $_start))
      )
    "#;

    let binary = wat::parse_str(wat).unwrap();
    let module = walrus::Module::from_buffer(&binary).unwrap();

    let id_reps: HashMap<usize, usize> = gather_replacement_ids(&module).iter().map(|(x, y)| (x.index(), y.index())).collect();

    assert!(id_reps[&2] == 8);
    assert!(id_reps[&3] == 7);

}
