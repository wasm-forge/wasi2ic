mod arguments;
mod common;
use crate::arguments::Wasm2icArgs;
use clap::Parser;
use std::path::Path;

fn is_wat(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        if ext == "wat" {
            return true;
        }
    }

    false
}

//fn do_wasm_file_processing(input_wasm: &Path, output_wasm: &Path) -> Result<(), anyhow::Error> {
pub fn do_wasm_file_processing(args: &Wasm2icArgs) -> Result<(), anyhow::Error> {
    log::info!(
        "Processing input file: '{}', writing output into '{}'",
        args.input_file,
        args.output_file
    );

    if !args.quiet {
        println!(
            "wasi2ic {}: processing input file: '{}', writing output into '{}'",
            env!("CARGO_PKG_VERSION"),
            args.input_file,
            args.output_file
        );
    }

    let input_wasm = Path::new(&args.input_file);
    let wasm = if is_wat(input_wasm) {
        wat::parse_file(input_wasm)?
    } else {
        std::fs::read(input_wasm)?
    };

    // use the same parser as dfx here
    let mut module = ic_wasm::utils::parse_wasm(&wasm, true)?; //walrus::Module::from_buffer_with_config(&wasm, &config)?;

    common::do_module_replacements(&mut module);

    let wasm = module.emit_wasm();

    let output_wasm = Path::new(&args.output_file);
    if is_wat(output_wasm) {
        // write using wat printer
        let wat = wasmprinter::print_bytes(&wasm)?;
        std::fs::write(output_wasm, wat)?;
    } else {
        std::fs::write(output_wasm, wasm)?;
    };

    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    let args = arguments::Wasm2icArgs::parse();
    do_wasm_file_processing(&args)?;
    Ok(())
}

#[cfg(test)]
mod tests;
