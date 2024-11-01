


## wasi2ic: WASI Polyfill Tool

![Tests](https://github.com/wasm-forge/wasi2ic/actions/workflows/rust.yml/badge.svg?event=push)
[![Coverage](https://codecov.io/gh/wasm-forge/wasi2ic/graph/badge.svg?token=XE48Z6JSYS)](https://codecov.io/gh/wasm-forge/wasi2ic)

`wasi2ic` is a command-line tool that replaces WebAssembly System Interface (WASI) specific function calls with their 
corresponding polyfill implementations. This allows you to run Wasm binaries compiled for `wasm32-wasi` on the Internet 
Computer.

## Installation

Install `wasi2ic` using Cargo:

```bash
cargo install wasi2ic
```

## Usage

Replace WASI dependencies in a Wasm file using:

```bash
wasi2ic <input-wasm-file> <output_wasm_file>
```

This command reads the input Wasm file, removes WASI dependencies, and reroutes WASI calls to their IC-specific 
implementations. Note that the polyfill implementation must be present in your Wasm binary.

To include the polyfill implementation in your Canister project, add the ic-wasi-polyfill dependency in it:
```bash
cargo add ic-wasi-polyfill
```

Also you will need to add the initialization call to the polyfill library, basic example using memory manager:

```rust
use ic_stable_structures::{memory_manager::MemoryManager, DefaultMemoryImpl};
use std::cell::RefCell;

thread_local! {
    // The memory manager enables multiple virtual memories in one.
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> =
        RefCell::new(MemoryManager::init(DefaultMemoryImpl::default()));
}

#[ic_cdk::init]
fn init() {
    MEMORY_MANAGER.with(|m| {
        let m = m.borrow();
        ic_wasi_polyfill::init_with_memory_manager(&[0u8; 32], &[], &m, 200..210);
    });
}
```


For more detailed information, see our [examples repository](https://github.com/wasm-forge/examples).


## Related repositories


| Repository                                    |  Description                  | 
| --------------------------------------------- | ----------------------------- |
| [ic-wasi-polyfill](https://github.com/wasm-forge/ic-wasi-polyfill) | Polyfill library implementing the low-level WASI calls. |
| [stable-fs](https://github.com/wasm-forge/stable-fs) | Simple file system implementation based on the stable structures. The file system implements backend for the ic-wasi-polyfill |
| [examples](https://github.com/wasm-forge/examples) | Repository containing various examplpes of running WASI projects on the IC. |
