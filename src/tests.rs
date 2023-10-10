

use crate::*;



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
        (import "wasi_unstable" "fd_write" (func $_wasi_unstable_fd_write (;2;) (type 5)))
        (import "wasi_unstable" "random_get" (func $_wasi_unstable_random_get (;3;) (type 3)))
        (import "wasi_unstable" "environ_get" (func $__imported_wasi_unstable_environ_get (;4;) (type 3)))
        (import "wasi_unstable" "proc_exit" (func $__imported_wasi_unstable_proc_exit (;5;) (type 1)))

        (func $_start (;6;) (type 0)
            i32.const 1
            i32.const 2
            call $__ic_custom_random_get
            i32.const 1
            i32.const 2
            call $_wasi_unstable_random_get
            i32.const 4
            i32.const 5
            call $_wasi_unstable_fd_write
            drop
        )

        (func $__ic_custom_random_get (;8;) (type 3) (param i32 i32) (result i32)
            call $_msg_reply

            i32.const 421
        )

        (func $ic_dummy_fd_write (;7;) (type 5) (param i32 i32 i32 i32) (result i32)
            i32.const 0
            i32.const 0
            call $_dprint
            i32.const 42
        )

        (export "__ic_custom_fd_write" (func $ic_dummy_fd_write))
        (export "_start" (func $_start))
    )
    "#;

    let binary = wat::parse_str(wat).unwrap();
    let module = walrus::Module::from_buffer(&binary).unwrap();

    let id_reps: HashMap<usize, usize> = gather_replacement_ids(&module).iter().map(|(x, y)| (x.index(), y.index())).collect();

    assert!(id_reps[&2] == 8);
    assert!(id_reps[&3] == 7);

}


#[test]
fn test_do_module_replacements() {

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
        
        (table (;0;) 5 5 funcref)
        (elem (;0;) (i32.const 1) func $_wasi_snapshot_preview_fd_write $_wasi_snapshot_preview_random_get )

        (func $_start (;6;) (type 0)
            i32.const 1
            i32.const 2
            call $_wasi_snapshot_preview_random_get

            (block $test_block 
                i32.const 0
                (if 
                    (then
                        (block $test_block3
                            i32.const 1
                            i32.const 2
                
                            call $_wasi_snapshot_preview_random_get
    
                            drop
                        )
                    )
                    (else 
                        (loop $test_loop (result i32)
                            i32.const 1
                            i32.const 2
                
                            call $_wasi_snapshot_preview_random_get
    
                            br_if $test_loop 

                            i32.const 2
                        )

                        drop

                    )
                )
            )

            i32.const 3
            i32.const 4
            i32.const 5
            call $_wasi_snapshot_preview_fd_write

            call $__imported_wasi_snapshot_preview1_proc_exit
        )

        (func $__ic_custom_random_get (;7;) (type 3) (param i32 i32) (result i32)
            i32.const 0
            ref.func $_wasi_snapshot_preview_random_get
            table.set 0

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
    let mut module = walrus::Module::from_buffer(&binary).unwrap();

    do_module_replacements(&mut module);

    // we expect random_get and fd_write to be replaced, environ_get to be removed and the calls to the proc_exit to remain
    let imports = module.imports;

    let result = imports.find("ic0", "debug_print");
    assert!(result.is_some());
    let result = imports.find("ic0", "msg_reply");
    assert!(result.is_some());
    let result = imports.find("wasi_snapshot_preview1", "proc_exit");
    assert!(result.is_some());
    let result = imports.find("wasi_snapshot_preview1", "fd_write");
    assert!(None == result);
    let result = imports.find("wasi_snapshot_preview1", "random_get");
    assert!(None == result);
    let result = imports.find("wasi_snapshot_preview1", "environ_get");
    assert!(None == result);

}



#[test]
fn test_file_processing() {
    
    std::fs::create_dir_all("target/test").unwrap();

    let input_file = Path::new("test/assets/main_test.wat");
    assert!(input_file.exists());

    let output_wasm = Path::new("target/test/nowasi.wasm");

    let _ = std::fs::remove_file(output_wasm);

    assert!(!output_wasm.exists());

    do_wasm_file_processing(input_file, output_wasm).unwrap();

    assert!(output_wasm.exists());

    let mut module = walrus::Module::from_file(output_wasm).unwrap();

    // we expect random_get and fd_write to be replaced, environ_get to be removed and the calls to the proc_exit to remain
    let imports = module.imports;

    let result = imports.find("ic0", "debug_print");
    assert!(result.is_some());
    let result = imports.find("ic0", "msg_reply");
    assert!(result.is_some());
    let result = imports.find("wasi_snapshot_preview1", "proc_exit");
    assert!(result.is_some());
    let result = imports.find("wasi_snapshot_preview1", "fd_write");
    assert!(None == result);
    let result = imports.find("wasi_snapshot_preview1", "random_get");
    assert!(None == result);
    let result = imports.find("wasi_snapshot_preview1", "environ_get");
    assert!(None == result);
}
