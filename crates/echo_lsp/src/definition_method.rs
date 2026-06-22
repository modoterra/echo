use echo_index::{DependencyFact, DependencyKind, Symbol, SymbolKind};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct MethodCallAt {
    pub(super) receiver: String,
    pub(super) name: String,
}

pub(super) fn method_call_at(source: &str, offset: usize) -> Option<MethodCallAt> {
    let offset = offset.min(source.len());
    let (name_start, name_end, name) = identifier_at(source, offset)?;
    if name_start < 2 || &source[name_start - 2..name_start] != "->" {
        return None;
    }
    let (receiver_start, _, receiver) = identifier_at(source, name_start.saturating_sub(3))?;
    if receiver_start == 0 || source.as_bytes().get(receiver_start - 1) != Some(&b'$') {
        return None;
    }
    if name_end < source.len()
        && source[name_end..]
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_alphanumeric() || ch == '_')
    {
        return None;
    }
    Some(MethodCallAt { receiver, name })
}

fn identifier_at(source: &str, offset: usize) -> Option<(usize, usize, String)> {
    let bytes = source.as_bytes();
    if bytes.is_empty() || offset > bytes.len() {
        return None;
    }
    let mut start = offset.min(bytes.len().saturating_sub(1));
    while start > 0 && is_identifier_byte(bytes[start - 1]) {
        start -= 1;
    }
    let mut end = start;
    while end < bytes.len() && is_identifier_byte(bytes[end]) {
        end += 1;
    }
    if start == end || offset < start || offset > end {
        return None;
    }
    Some((start, end, source[start..end].to_string()))
}

fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

pub(super) fn local_variable_type(symbols: &[Symbol], name: &str) -> Option<String> {
    symbols
        .iter()
        .find(|symbol| symbol.kind == SymbolKind::LocalVariable && symbol.name.text == name)
        .and_then(|symbol| {
            symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.clone())
        })
}

pub(super) fn resolve_imported_type(dependencies: &[DependencyFact], ty: &str) -> Option<String> {
    dependencies
        .iter()
        .filter(|dependency| dependency.kind == DependencyKind::PhpUse)
        .find(|dependency| {
            dependency.alias.as_deref() == Some(ty)
                || dependency
                    .target
                    .rsplit('\\')
                    .next()
                    .is_some_and(|name| name == ty)
        })
        .map(|dependency| dependency.target.clone())
}
