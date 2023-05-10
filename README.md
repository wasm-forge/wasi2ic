

# wasi2ic

![Tests](https://github.com/wasm-forge/wasi2ic/actions/workflows/rust.yml/badge.svg?event=push)


`wasi2ic` is a tool to convert WebAssembly System Interface (WASI) dependent Wasm module to run on the Internet Computer (IC) computer.


## Compiling wasi2ic

* Clone the https://github.com/wasm-forge/wasi2ic: 
```bash
  git clone https://github.com/wasm-forge/wasi2ic
```

* Enter the `wasi2ic` folder

* Compile the project with `cargo build` and copy the tool to some executable path or alternatively use `cargo install --path .`, but note that the project is still in the early development.


## Usage

To use this tool, navigate to the directory where the WASI-dependent project resides, and run the command:

```bash
  wasi2ic <input-wasm-file> <output_wasm_file>
```

This will read the old Wasm file and create a new Wasm file with the WASI dependencies removed.

**NOTE**

To use the tool, you need to add a polyfill dependency into your rust project:
```bash
cargo add --git https://github.com/wasm-forge/ic-wasi-polyfill
```

This will create the polyfill methods in your `.wasm` file, which are needed for `wasi2ic`.

**NOTE2**

The polyfill library is still in early development, and not all methods are implemented. Feel free to review its current state before use.


