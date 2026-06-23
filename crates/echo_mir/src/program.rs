use crate::MirStmt;
use echo_ast::{FunctionDeclStmt, ImportStmt, TypedParam};

#[derive(Debug, Clone)]
pub struct MirProgram {
    pub(crate) source_dir: Option<String>,
    pub(crate) imports: Vec<ImportStmt>,
    pub(crate) functions: Vec<MirFunction>,
    pub(crate) statements: Vec<MirStmt>,
}

impl MirProgram {
    pub fn source_dir(&self) -> Option<&str> {
        self.source_dir.as_deref()
    }

    pub fn imports(&self) -> &[ImportStmt] {
        &self.imports
    }

    pub fn functions(&self) -> &[MirFunction] {
        &self.functions
    }

    pub fn statements(&self) -> &[MirStmt] {
        &self.statements
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct MirFunction {
    pub source: FunctionDeclStmt,
    pub name: String,
    pub params: Vec<TypedParam>,
    pub return_type: Option<String>,
    pub body: Vec<MirStmt>,
}
