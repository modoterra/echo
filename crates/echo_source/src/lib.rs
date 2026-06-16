use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceMode {
    PhpFile,
    EchoFile,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

#[derive(Debug, Clone)]
pub struct SourceFile {
    pub path: PathBuf,
    pub text: String,
    pub mode: SourceMode,
}

impl SourceFile {
    pub fn new(path: PathBuf, text: String) -> Self {
        let mode = match path.extension().and_then(|ext| ext.to_str()) {
            Some("echo") | Some("xo") => SourceMode::EchoFile,
            _ => SourceMode::PhpFile,
        };

        Self { path, text, mode }
    }
}
