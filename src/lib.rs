mod common;

/// rewire wasi functions.
/// If there are no functions found for replacement, the module processing will not happen.
/// This is done to avoid any modification if the ic-wasi-polyfill library was not included in the build.
///
/// returns true if the module was modified
pub fn process_module(m: &mut walrus::Module) -> bool {
    common::do_module_replacements(m)
}
