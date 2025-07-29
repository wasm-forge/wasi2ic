mod common;

/// rewire wasi functions.
/// If there are no functions found for replacement, the module processing will not happen.
/// This is done to avoid any modification if the ic-wasi-polyfill library was not included in the build.
pub fn process_module(m: &mut walrus::Module) {
    common::do_module_replacements(m);
}
