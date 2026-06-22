mod core;
mod php;
mod stdlib;

pub(crate) fn jit_runtime_symbol_addresses() -> Vec<(&'static str, usize)> {
    let mut symbols = core::symbols();
    symbols.extend(php::symbols());
    symbols.extend(stdlib::symbols());
    symbols.push((
        "echo_shutdown",
        echo_runtime::echo_shutdown as extern "C" fn() as usize,
    ));
    symbols
}
