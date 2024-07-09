use clap::Parser;

/*
    /// Do not demangle function names.
    #[arg(long, default_value_t = false)]
    pub no_demangle: bool,

    /// List all functions.
    #[arg(short, long, default_value_t = false)]
    pub list_all_functions: bool,

    /// List local functions.
    #[arg(long, default_value_t = false)]
    pub list_local_functions: bool,

    /// List imports.
    #[arg(long, default_value_t = false)]
    pub list_imports: bool,

    /// List start function.
    #[arg(long, default_value_t = false)]
    pub list_start_function: bool,
*/

#[derive(Parser, Debug)]
#[command(version, about=format!("Wasi dependency removal V{}", env!("CARGO_PKG_VERSION")), long_about = None)]
pub struct Wasm2icArgs {
    /// Quiet mode
    #[arg(long, short, default_value_t = false)]
    pub quiet: bool,

    /// Input file to process (*.wasm or *.wat).
    pub input_file: String,

    /// Output file to store the processed Wasm (*.wasm).
    #[arg(default_value_t = String::from("no_wasi.wasm"))]
    pub output_file: String,
}
