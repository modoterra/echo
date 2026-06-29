use echo_index::FileId;
use ropey::Rope;
use tower_lsp_server::ls_types::Uri;

#[derive(Debug, Clone)]
pub struct Document {
    pub uri: Uri,
    pub version: i32,
    pub text: Rope,
    pub file_id: FileId,
}

impl Document {
    pub fn new(uri: Uri, version: i32, text: String, file_id: FileId) -> Self {
        Self {
            uri,
            version,
            text: Rope::from_str(&text),
            file_id,
        }
    }

    pub fn replace_text(&mut self, version: i32, text: String) {
        self.version = version;
        self.text = Rope::from_str(&text);
    }

    pub fn text_string(&self) -> String {
        self.text.to_string()
    }
}
