

# wasi2ic

![Tests](https://github.com/wasm-forge/wasi2ic/actions/workflows/rust.yml/badge.svg?event=push)
[![Coverage](https://codecov.io/gh/wasm-forge/wasi2ic/graph/badge.svg?token=XE48Z6JSYS)](https://codecov.io/gh/wasm-forge/wasi2ic)

`wasi2ic` is a tool to convert WebAssembly System Interface (WASI) dependent Wasm module for running it on the Internet Computer (IC) computer. The tool reads a Wasm binary and automatically replaces all WASI calls with their corresponding polyfill implementation.


## Installing wasi2ic

```bash
  cargo install wasi2ic
```

## Usage

To use this tool , navigate to the directory where the WASI-dependent binary resides, and run the command:

```bash
  wasi2ic <input-wasm-file> <output_wasm_file>
```

This will read the input Wasm file and produce a new Wasm file with the WASI dependencies removed and and WASI rerouted to their IC-specific implementations.


The tool requires that the polyfill implementation is present in your Wasm binary, to include the polyfill implementation into your canister project you need to add the polyfill dependency. For more information check out the [examples](https://github.com/wasm-forge/examples) repository.


## Related repositories


| Repository                                    |  Description                  | 
| --------------------------------------------- | ----------------------------- |
| [ic-wasi-polyfill](https://github.com/wasm-forge/ic-wasi-polyfill) | Polyfill library implementing the low-level WASI calls. |
| [stable-fs](https://github.com/wasm-forge/stable-fs) | Simple file system implementation based on the stable structures. The file system implements backend for the ic-wasi-polyfill |
| [examples](https://github.com/wasm-forge/examples) | Repository containing various examplpes of runnig WASI projects on the IC. |
