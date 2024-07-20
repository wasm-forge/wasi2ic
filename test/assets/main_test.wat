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

    (func $_initialize (;6;) (type 0)
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




    (export "_initialize" (func $_initialize))

)