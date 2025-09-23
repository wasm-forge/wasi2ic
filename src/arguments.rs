use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about=format!("Wasi dependency removal V{}", env!("CARGO_PKG_VERSION")), long_about = None)]
pub struct Wasm2icArgs {
    /// Quiet mode
    #[arg(long, short, default_value_t = false)]
    pub quiet: bool,

    /// Show WASM module imports
    #[arg(long, short, default_value_t = false)]
    pub imports: bool,

    /// Input file to process (*.wasm or *.wat).
    pub input_file: String,

    /// Output file to store the processed Wasm (*.wasm or *.wat).
    #[arg(default_value_t = String::from("no_wasi.wasm"))]
    pub output_file: String,
}
