use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolName {
    pub text: SmolStr,
}

impl SymbolName {
    pub fn new(text: impl Into<SmolStr>) -> Self {
        Self { text: text.into() }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FqName {
    pub namespace: Vec<SmolStr>,
    pub name: SmolStr,
}

impl FqName {
    pub fn new(namespace: Vec<SmolStr>, name: impl Into<SmolStr>) -> Self {
        Self {
            namespace,
            name: name.into(),
        }
    }

    pub fn as_string(&self) -> String {
        if self.namespace.is_empty() {
            self.name.to_string()
        } else {
            format!("{}\\{}", self.namespace.join("\\"), self.name)
        }
    }
}
