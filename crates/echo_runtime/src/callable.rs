#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EchoSymbol {
    name: String,
}

impl EchoSymbol {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }

    pub fn as_str(&self) -> &str {
        &self.name
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EchoCallable {
    Function(EchoSymbol),
}
