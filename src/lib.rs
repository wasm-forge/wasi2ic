mod common;

/// Rewire WASI functions.
/// If there are no functions found for replacement, the module processing will not happen.
/// This is done to avoid any modification if the ic-wasi-polyfill library was not included in the build.
///
/// returns true if the module was modified
pub fn process_module(m: &mut walrus::Module) -> bool {
    common::do_module_replacements(m)
}

/// Convenience function to get the list of functions imported
///
/// returns pairs of values: (module name, function name)
pub fn module_imports(m: &mut walrus::Module) -> Vec<(String, String)> {
    common::get_module_imports(m)
}
