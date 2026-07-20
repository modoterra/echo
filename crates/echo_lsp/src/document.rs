//! Open document store (editor buffers).

use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One open text document (LSP `TextDocumentItem` subset).
#[derive(Debug, Clone)]
pub struct OpenDocument {
    pub uri: String,
    /// Filesystem path when `uri` is `file://…`; otherwise unset.
    pub path: Option<PathBuf>,
    pub version: i32,
    pub text: String,
}

/// In-memory open documents keyed by URI.
#[derive(Debug, Default)]
pub struct DocumentStore {
    docs: HashMap<String, OpenDocument>,
}

impl DocumentStore {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn open(&mut self, uri: String, version: i32, text: String) {
        let path = uri_to_path(&uri);
        self.docs.insert(
            uri.clone(),
            OpenDocument {
                uri,
                path,
                version,
                text,
            },
        );
    }

    pub fn change(&mut self, uri: &str, version: i32, text: String) -> bool {
        if let Some(doc) = self.docs.get_mut(uri) {
            doc.version = version;
            doc.text = text;
            true
        } else {
            false
        }
    }

    pub fn close(&mut self, uri: &str) -> Option<OpenDocument> {
        self.docs.remove(uri)
    }

    #[must_use]
    pub fn get(&self, uri: &str) -> Option<&OpenDocument> {
        self.docs.get(uri)
    }

    /// Overlay map for resolver: canonical path → buffer text.
    #[must_use]
    pub fn overlays(&self) -> HashMap<PathBuf, String> {
        let mut map = HashMap::new();
        for doc in self.docs.values() {
            if let Some(path) = &doc.path {
                let key = path.canonicalize().unwrap_or_else(|_| path.clone());
                map.insert(key, doc.text.clone());
            }
        }
        map
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.docs.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.docs.is_empty()
    }

    /// Iterate open documents (order unspecified).
    pub fn iter(&self) -> impl Iterator<Item = &OpenDocument> {
        self.docs.values()
    }
}

/// Convert `file://` URI to a path (best-effort; non-file URIs → `None`).
#[must_use]
pub fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    // file:///home/… or file://localhost/home/…
    let path = if rest.starts_with('/') {
        PathBuf::from(rest)
    } else if let Some(stripped) = rest.strip_prefix("localhost") {
        PathBuf::from(stripped)
    } else {
        PathBuf::from(rest)
    };
    Some(percent_decode_path(path))
}

fn percent_decode_path(path: PathBuf) -> PathBuf {
    let s = path.to_string_lossy();
    if !s.contains('%') {
        return path;
    }
    let mut out = String::with_capacity(s.len());
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let (Some(h), Some(l)) = (from_hex(bytes[i + 1]), from_hex(bytes[i + 2])) {
                out.push((h << 4 | l) as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    PathBuf::from(out)
}

fn from_hex(b: u8) -> Option<u8> {
    match b {
        b'0'..=b'9' => Some(b - b'0'),
        b'a'..=b'f' => Some(b - b'a' + 10),
        b'A'..=b'F' => Some(b - b'A' + 10),
        _ => None,
    }
}

/// Path → `file://` URI for publishDiagnostics.
#[must_use]
pub fn path_to_uri(path: &Path) -> String {
    let abs = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    format!("file://{}", abs.display())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_uri_roundtrip_linux() {
        let p = uri_to_path("file:///tmp/foo.echo").unwrap();
        assert_eq!(p, PathBuf::from("/tmp/foo.echo"));
    }

    #[test]
    fn open_change_close() {
        let mut s = DocumentStore::new();
        s.open("file:///tmp/a.echo".into(), 1, "$ x = 1\n".into());
        assert_eq!(s.len(), 1);
        assert!(s.change("file:///tmp/a.echo", 2, "$ x = 2\n".into()));
        assert_eq!(s.get("file:///tmp/a.echo").unwrap().version, 2);
        assert!(s.close("file:///tmp/a.echo").is_some());
        assert!(s.is_empty());
    }
}
