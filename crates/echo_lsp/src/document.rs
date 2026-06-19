use echo_index::{EchoFileMode, FileId};
use ropey::Rope;
use tower_lsp_server::ls_types::Uri;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Uri,
    pub version: i32,
    pub mode: EchoFileMode,
    pub text: Rope,
    pub file_id: FileId,
}

impl Document {
    pub fn new(uri: Uri, version: i32, text: String, file_id: FileId) -> Self {
        let mode = mode_from_uri(&uri);
        Self {
            uri,
            version,
            mode,
            text: Rope::from_str(&text),
            file_id,
        }
    }

    pub fn replace_text(&mut self, version: i32, text: String) {
        self.version = version;
        self.mode = mode_from_uri(&self.uri);
        self.text = Rope::from_str(&text);
    }

    pub fn text_string(&self) -> String {
        self.text.to_string()
    }
}

pub fn mode_from_uri(uri: &Uri) -> EchoFileMode {
    let path = uri.path().decode().to_string_lossy();
    match path.rsplit('.').next() {
        Some("php") => EchoFileMode::PhpCompat,
        Some("echo") | Some("xo") => EchoFileMode::Echo,
        _ => EchoFileMode::Echo,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn detects_file_mode_from_uri_extension() {
        assert_eq!(
            mode_from_uri(&Uri::from_file_path(Path::new("/project/example.php")).unwrap()),
            EchoFileMode::PhpCompat
        );
        assert_eq!(
            mode_from_uri(&Uri::from_file_path(Path::new("/project/example.echo")).unwrap()),
            EchoFileMode::Echo
        );
        assert_eq!(
            mode_from_uri(&Uri::from_file_path(Path::new("/project/example.txt")).unwrap()),
            EchoFileMode::Echo
        );
    }
}
