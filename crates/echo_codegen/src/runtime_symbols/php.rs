mod array;
mod crypto;
mod environment;
mod filesystem;
mod math;
mod output;
mod string;
mod types;

pub(super) fn symbols() -> Vec<(&'static str, usize)> {
    let mut symbols = Vec::new();
    symbols.extend(array::symbols());
    symbols.extend(environment::symbols());
    symbols.extend(filesystem::symbols());
    symbols.extend(crypto::symbols());
    symbols.extend(math::symbols());
    symbols.extend(output::symbols());
    symbols.extend(string::symbols());
    symbols.extend(types::symbols());
    symbols
}
