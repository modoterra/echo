use crate::{FileId, FqName, SymbolName, TextRange};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub u64);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Trait,
    Enum,
    Constant,
    Property,
    Parameter,
    LocalVariable,
    Namespace,
    TypeAlias,
    ErrorType,
    Extension,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Visibility {
    Public,
    Protected,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature {
    pub text: String,
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub id: SymbolId,
    pub file_id: FileId,
    pub name: SymbolName,
    pub fq_name: Option<FqName>,
    pub kind: SymbolKind,
    pub range: TextRange,
    pub selection_range: TextRange,
    pub visibility: Option<Visibility>,
    pub container: Option<SymbolId>,
    pub signature: Option<Signature>,
}

#[derive(Debug, Clone)]
pub struct SymbolFact {
    pub name: SymbolName,
    pub fq_name: Option<FqName>,
    pub kind: SymbolKind,
    pub range: TextRange,
    pub selection_range: TextRange,
    pub visibility: Option<Visibility>,
    pub signature: Option<Signature>,
}
