

# wasi2ic

`wasi2ic` is a tool to convert WebAssembly System Interface (WASI) dependent Wasm module to run on the Internet Computer (IC) computer.


## Usage

To use this tool, navigate to the directory where the WASI-dependent project resides, and run the following command:

`wasi2ic <input-wasm-file> <output_wasm_file>`

This will read the old Wasm file and create a new Wasm file with the WASI dependencies removed.

