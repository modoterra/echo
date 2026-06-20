use std::collections::HashMap;

use echo_ast::{
    BinaryOp, ClassMember, Expr, FunctionDeclStmt, ImportSource, MagicConstantKind,
    NamespaceSource, Program, RequireKind, Stmt, UnaryOp,
};
use echo_diagnostics::Diagnostic;
use echo_index::{
    DependencyFact, DependencyKind, EchoFileMode, FileId, FqName, IndexFacts, ReferenceFact,
    ReferenceKind, Signature, SymbolFact, SymbolKind, SymbolName, TextRange,
};
use echo_source::Span;
use smol_str::SmolStr;

#[derive(Debug, Clone)]
pub struct Analysis {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
}

impl Analysis {
    pub fn expression_type(&self, expr: &Expr) -> Type {
        self.expression_type_at(expr.span())
    }

    pub fn expression_type_at(&self, span: Span) -> Type {
        self.expression_types
            .get(&SpanKey::from(span))
            .cloned()
            .unwrap_or(Type::Unknown)
    }

    pub fn variable(&self, name: &str) -> Option<&VariableInfo> {
        self.variables.get(name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VariableInfo {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Type {
    Null,
    Bool,
    Int,
    Float,
    Number,
    String,
    Array,
    List,
    Object(Option<String>),
    Task,
    Thread,
    Process,
    Never,
    Unknown,
    Named(String),
}

impl Type {
    pub fn display_name(&self) -> String {
        match self {
            Self::Null => "null".to_string(),
            Self::Bool => "bool".to_string(),
            Self::Int => "int".to_string(),
            Self::Float => "float".to_string(),
            Self::Number => "number".to_string(),
            Self::String => "string".to_string(),
            Self::Array => "array".to_string(),
            Self::List => "list".to_string(),
            Self::Object(Some(name)) if !name.is_empty() => name.clone(),
            Self::Object(Some(_)) => "object".to_string(),
            Self::Object(None) => "object".to_string(),
            Self::Task => "task".to_string(),
            Self::Thread => "thread".to_string(),
            Self::Process => "process".to_string(),
            Self::Never => "never".to_string(),
            Self::Unknown => "unknown".to_string(),
            Self::Named(name) => name.clone(),
        }
    }
}

pub fn analyze(program: &Program) -> Result<Analysis, Vec<Diagnostic>> {
    let mut analyzer = Analyzer::default();
    analyzer.analyze_statements(&program.statements);
    if analyzer.diagnostics.is_empty() {
        Ok(Analysis {
            expression_types: analyzer.expression_types,
            variables: analyzer.variables,
        })
    } else {
        Err(analyzer.diagnostics)
    }
}

pub fn index_facts(program: &Program, file_id: FileId, mode: EchoFileMode) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id, mode);
    extractor.extract(program);
    extractor.into_facts()
}

pub fn index_facts_from_source(
    source: &str,
    program: &Program,
    file_id: FileId,
    mode: EchoFileMode,
) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id, mode);
    extractor.source_text = Some(source.to_string());
    extractor.source_dir.clone_from(&program.source_dir);
    extractor.extract_phpdoc_var_source_facts(source);
    extractor.extract_statements(&program.statements);
    extractor.into_facts()
}

struct IndexFactExtractor {
    file_id: FileId,
    mode: EchoFileMode,
    source_text: Option<String>,
    source_dir: Option<String>,
    namespace: Vec<SmolStr>,
    declarations: Vec<SymbolFact>,
    dependencies: Vec<DependencyFact>,
    references: Vec<ReferenceFact>,
}

impl IndexFactExtractor {
    fn new(file_id: FileId, mode: EchoFileMode) -> Self {
        Self {
            file_id,
            mode,
            source_text: None,
            source_dir: None,
            namespace: Vec::new(),
            declarations: Vec::new(),
            dependencies: Vec::new(),
            references: Vec::new(),
        }
    }

    fn extract(&mut self, program: &Program) {
        self.source_dir.clone_from(&program.source_dir);
        self.extract_statements(&program.statements);
    }

    fn extract_phpdoc_var_source_facts(&mut self, source: &str) {
        for annotation in phpdoc_var_annotations(source) {
            self.declarations.push(SymbolFact {
                name: SymbolName::new(annotation.variable.as_str()),
                fq_name: Some(self.fq_name(&annotation.variable)),
                kind: SymbolKind::LocalVariable,
                range: annotation.range,
                selection_range: annotation.selection_range,
                visibility: None,
                signature: Some(Signature {
                    text: annotation.ty.clone(),
                }),
            });
            self.references.push(ReferenceFact {
                kind: ReferenceKind::ClassLike,
                name: annotation.ty,
                qualifier: None,
                range: annotation.ty_range,
            });
        }
    }

    fn extract_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.extract_statement(statement);
        }
    }

    fn extract_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Namespace(statement) => {
                self.namespace = match statement.source {
                    NamespaceSource::Php => statement
                        .name
                        .parts
                        .iter()
                        .map(|part| SmolStr::new(part.as_str()))
                        .collect(),
                    NamespaceSource::Std => {
                        let mut namespace = vec![SmolStr::new("std")];
                        namespace.extend(
                            statement
                                .name
                                .parts
                                .iter()
                                .map(|part| SmolStr::new(part.as_str())),
                        );
                        namespace
                    }
                };

                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_string()),
                    fq_name: Some(FqName::new(Vec::new(), namespace_display(&self.namespace))),
                    kind: SymbolKind::Namespace,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: None,
                });
            }
            Stmt::Use(statement) => {
                let target = statement.name.as_string();
                self.dependencies.push(DependencyFact {
                    kind: DependencyKind::PhpUse,
                    target: target.clone(),
                    alias: statement.alias.clone(),
                    range: span_range(statement.span),
                    target_range: self
                        .span_range_for_text(statement.span, &target)
                        .unwrap_or_else(|| span_range(statement.span)),
                });
            }
            Stmt::Import(statement) => {
                let (kind, target) = match &statement.source {
                    ImportSource::Std => {
                        (DependencyKind::EchoStdImport, statement.name.as_string())
                    }
                    ImportSource::File(path) => (
                        DependencyKind::EchoFileImport,
                        format!("{path}#{}", statement.name.as_string()),
                    ),
                };
                self.dependencies.push(DependencyFact {
                    kind,
                    target,
                    alias: statement.alias.clone(),
                    range: span_range(statement.span),
                    target_range: span_range(statement.span),
                });
            }
            Stmt::FunctionDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Function,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: function_signature(statement),
                });
                self.extract_statements(&statement.body);
            }
            Stmt::ClassDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Class,
                    range: span_range(statement.span),
                    selection_range: self
                        .span_range_for_text(statement.span, &statement.name)
                        .unwrap_or_else(|| span_range(statement.span)),
                    visibility: None,
                    signature: None,
                });

                for member in &statement.members {
                    let ClassMember::Method(method) = member;
                    self.declarations.push(SymbolFact {
                        name: SymbolName::new(method.name.as_str()),
                        fq_name: Some(
                            self.fq_name(&format!("{}::{}", statement.name, method.name)),
                        ),
                        kind: SymbolKind::Method,
                        range: span_range(method.span),
                        selection_range: self
                            .span_range_for_text(method.span, &method.name)
                            .unwrap_or_else(|| span_range(method.span)),
                        visibility: None,
                        signature: Some(Signature {
                            text: method_signature(&method.params, method.return_type.as_deref()),
                        }),
                    });
                }
            }
            Stmt::TypeDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::TypeAlias,
                    range: span_range(statement.span),
                    selection_range: span_range(statement.span),
                    visibility: None,
                    signature: None,
                });
            }
            Stmt::Loop(statement) => self.extract_statements(&statement.body),
            Stmt::If(statement) => {
                self.extract_expr_dependencies(&statement.condition);
                self.extract_statements(&statement.body);
            }
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                for expr in &statement.args {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                for expr in &statement.args {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::Assign(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Let(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::AssignRef(_) | Stmt::Break(_) => {}
            Stmt::Return(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Yield(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Expr(statement) => self.extract_expr_dependencies(&statement.expr),
            Stmt::Append(statement) => self.extract_expr_dependencies(&statement.value),
        }
    }

    fn extract_expr_dependencies(&mut self, expr: &Expr) {
        match expr {
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::MethodCall(expr) => {
                if let Expr::Variable(variable) = &expr.object {
                    let method_start = expr.object.span().end + 2;
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::Method,
                        name: expr.method.clone(),
                        qualifier: Some(variable.name.clone()),
                        range: TextRange::new(
                            method_start as u32,
                            method_start.saturating_add(expr.method.len()) as u32,
                        ),
                    });
                }
                self.extract_expr_dependencies(&expr.object);
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::StaticCall(expr) => {
                let name = expr.class_name.as_string();
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    range: TextRange::new(
                        expr.span.start as u32,
                        expr.span.start.saturating_add(name.len()) as u32,
                    ),
                    name,
                    qualifier: None,
                });
                let method_start = expr.span.start + expr.class_name.as_string().len() + 2;
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::StaticMethod,
                    name: expr.method.clone(),
                    qualifier: Some(expr.class_name.as_string()),
                    range: TextRange::new(
                        method_start as u32,
                        method_start.saturating_add(expr.method.len()) as u32,
                    ),
                });
                for arg in &expr.args {
                    self.extract_expr_dependencies(arg);
                }
            }
            Expr::Assign(expr) => self.extract_expr_dependencies(&expr.value),
            Expr::Require(expr) => {
                let target = self
                    .const_string_expr(&expr.path)
                    .map(|target| self.resolve_source_path(&target))
                    .unwrap_or_else(|| expr_path_label(&expr.path));
                self.dependencies.push(DependencyFact {
                    kind: require_dependency_kind(expr.kind, &target),
                    target,
                    alias: None,
                    range: span_range(expr.span),
                    target_range: span_range(expr.path.span()),
                });
                self.extract_expr_dependencies(&expr.path);
            }
            Expr::Defer(expr) => self.extract_statements(&expr.body),
            Expr::Run(expr) => match expr {
                echo_ast::RunExpr::Block { body, .. } => self.extract_statements(body),
                echo_ast::RunExpr::Task { expr, .. } => self.extract_expr_dependencies(expr),
                echo_ast::RunExpr::Group { entries, .. } => {
                    for entry in entries {
                        self.extract_statements(entry);
                    }
                }
            },
            Expr::Fork(expr) => match expr {
                echo_ast::ForkExpr::Block { body, .. } => self.extract_statements(body),
                echo_ast::ForkExpr::Task { expr, .. } => self.extract_expr_dependencies(expr),
            },
            Expr::Spawn(expr) => self.extract_expr_dependencies(&expr.command),
            Expr::Join(expr) => self.extract_expr_dependencies(&expr.handle),
            Expr::Loop(expr) => self.extract_statements(&expr.body),
            Expr::Unary(expr) => self.extract_expr_dependencies(&expr.expr),
            Expr::Binary(expr) => {
                if let Some(target) = self.const_dir_path_binary(expr) {
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::FilePath,
                        name: target,
                        qualifier: None,
                        range: span_range(expr.span),
                    });
                }
                self.extract_expr_dependencies(&expr.left);
                self.extract_expr_dependencies(&expr.right);
            }
            Expr::Field(expr) => self.extract_expr_dependencies(&expr.object),
            Expr::Index(expr) => {
                self.extract_expr_dependencies(&expr.collection);
                self.extract_expr_dependencies(&expr.index);
            }
            Expr::Object(expr) => {
                for field in &expr.fields {
                    self.extract_expr_dependencies(&field.value);
                }
            }
            Expr::List(expr) => {
                for value in &expr.values {
                    self.extract_expr_dependencies(value);
                }
            }
            Expr::Array(expr) => {
                for element in &expr.elements {
                    if let Some(key) = &element.key {
                        self.extract_expr_dependencies(key);
                    }
                    self.extract_expr_dependencies(&element.value);
                }
            }
            Expr::Null(_)
            | Expr::Bool(_)
            | Expr::String(_)
            | Expr::Number(_)
            | Expr::Variable(_)
            | Expr::MagicConstant(_) => {}
        }
    }

    fn const_string_expr(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::String(expr) => Some(expr.value.clone()),
            Expr::MagicConstant(expr) if expr.kind == MagicConstantKind::Dir => {
                self.source_dir.clone()
            }
            Expr::Binary(expr) if expr.op == BinaryOp::Concat => self.const_string_binary(expr),
            _ => None,
        }
    }

    fn const_string_binary(&self, expr: &echo_ast::BinaryExpr) -> Option<String> {
        if expr.op != BinaryOp::Concat {
            return None;
        }
        let left = self.const_string_expr(&expr.left)?;
        let right = self.const_string_expr(&expr.right)?;
        Some(format!("{left}{right}"))
    }

    fn const_dir_path_binary(&self, expr: &echo_ast::BinaryExpr) -> Option<String> {
        if !binary_contains_dir_magic_constant(expr) {
            return None;
        }
        self.const_string_binary(expr)
    }

    fn resolve_source_path(&self, target: &str) -> String {
        if std::path::Path::new(target).is_absolute() {
            return target.to_string();
        }
        self.source_dir
            .as_ref()
            .map(|source_dir| {
                std::path::Path::new(source_dir)
                    .join(target)
                    .to_string_lossy()
                    .to_string()
            })
            .unwrap_or_else(|| target.to_string())
    }

    fn fq_name(&self, name: &str) -> FqName {
        FqName::new(self.namespace.clone(), name)
    }

    fn span_range_for_text(&self, span: Span, needle: &str) -> Option<TextRange> {
        let source = self.source_text.as_ref()?;
        let haystack = source.get(span.start..span.end)?;
        let start = haystack.find(needle)? + span.start;
        Some(TextRange::new(start as u32, (start + needle.len()) as u32))
    }

    fn into_facts(self) -> IndexFacts {
        IndexFacts {
            file_id: self.file_id,
            mode: self.mode,
            declarations: self.declarations,
            dependencies: self.dependencies,
            references: self.references,
        }
    }
}

fn span_range(span: Span) -> TextRange {
    TextRange::new(span.start as u32, span.end as u32)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PhpDocVarAnnotation {
    ty: String,
    variable: String,
    range: TextRange,
    ty_range: TextRange,
    selection_range: TextRange,
}

fn phpdoc_var_annotations(source: &str) -> Vec<PhpDocVarAnnotation> {
    let mut annotations = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = source[search_start..].find("/**") {
        let comment_start = search_start + relative_start;
        let content_start = comment_start + 3;
        let Some(relative_end) = source[content_start..].find("*/") else {
            break;
        };
        let comment_end = content_start + relative_end + 2;
        let comment = &source[content_start..content_start + relative_end];

        for (line_start, line) in comment_lines(comment, content_start) {
            let trimmed = line.trim_start_matches([' ', '\t', '*']);
            let Some(var_offset) = trimmed.find("@var") else {
                continue;
            };
            let annotation_start = line_start + line.len() - trimmed.len() + var_offset;
            let after_var = &trimmed[var_offset + 4..];
            let after_var_offset = annotation_start + 4;
            let Some(annotation) = parse_phpdoc_var_annotation(after_var, after_var_offset) else {
                continue;
            };
            annotations.push(annotation);
        }

        search_start = comment_end;
    }

    annotations
}

fn comment_lines(comment: &str, base_offset: usize) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    comment.split_inclusive('\n').map(move |line| {
        let line_start = base_offset + offset;
        offset += line.len();
        (line_start, line.trim_end_matches(['\r', '\n']))
    })
}

fn parse_phpdoc_var_annotation(text: &str, base_offset: usize) -> Option<PhpDocVarAnnotation> {
    let trimmed_start = text.len() - text.trim_start().len();
    let text = text.trim_start();
    let base_offset = base_offset + trimmed_start;

    let ty_end = text.find(char::is_whitespace)?;
    let ty = text[..ty_end].trim();
    if ty.is_empty() {
        return None;
    }

    let after_ty = &text[ty_end..];
    let variable_relative = after_ty.find('$')?;
    let variable_start_in_text = ty_end + variable_relative;
    let variable_text = &text[variable_start_in_text + 1..];
    let variable_len = variable_text
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(char::len_utf8)
        .sum::<usize>();
    if variable_len == 0 {
        return None;
    }

    let selection_start = base_offset + variable_start_in_text;
    let selection_end = selection_start + 1 + variable_len;

    Some(PhpDocVarAnnotation {
        ty: ty.to_string(),
        variable: variable_text[..variable_len].to_string(),
        range: TextRange::new(base_offset as u32, selection_end as u32),
        ty_range: TextRange::new(base_offset as u32, (base_offset + ty.len()) as u32),
        selection_range: TextRange::new(selection_start as u32, selection_end as u32),
    })
}

fn expr_contains_dir_magic_constant(expr: &Expr) -> bool {
    match expr {
        Expr::MagicConstant(expr) => expr.kind == MagicConstantKind::Dir,
        Expr::Binary(expr) => binary_contains_dir_magic_constant(expr),
        _ => false,
    }
}

fn binary_contains_dir_magic_constant(expr: &echo_ast::BinaryExpr) -> bool {
    expr_contains_dir_magic_constant(&expr.left) || expr_contains_dir_magic_constant(&expr.right)
}

fn require_dependency_kind(kind: RequireKind, target: &str) -> DependencyKind {
    if target.ends_with("/vendor/autoload.php") || target.ends_with("\\vendor\\autoload.php") {
        DependencyKind::ComposerAutoload
    } else {
        match kind {
            RequireKind::Require => DependencyKind::Require,
            RequireKind::RequireOnce => DependencyKind::RequireOnce,
        }
    }
}

fn expr_path_label(expr: &Expr) -> String {
    match expr {
        Expr::Variable(expr) => format!("${}", expr.name),
        _ => "<dynamic path>".to_string(),
    }
}

fn namespace_display(namespace: &[SmolStr]) -> String {
    namespace
        .iter()
        .map(SmolStr::as_str)
        .collect::<Vec<_>>()
        .join("\\")
}

fn function_signature(statement: &FunctionDeclStmt) -> Option<Signature> {
    Some(Signature {
        text: method_signature(&statement.params, statement.return_type.as_deref()),
    })
}

fn method_signature(params: &[echo_ast::TypedParam], return_type: Option<&str>) -> String {
    let params = params
        .iter()
        .map(|param| match &param.ty {
            Some(ty) => format!("{ty} ${}", param.name),
            None => format!("${}", param.name),
        })
        .collect::<Vec<_>>()
        .join(", ");

    match return_type {
        Some(return_type) => format!("({params}): {return_type}"),
        None => format!("({params})"),
    }
}

#[derive(Default)]
struct Analyzer {
    expression_types: HashMap<SpanKey, Type>,
    variables: HashMap<String, VariableInfo>,
    diagnostics: Vec<Diagnostic>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct SpanKey {
    start: usize,
    end: usize,
}

impl From<Span> for SpanKey {
    fn from(span: Span) -> Self {
        Self {
            start: span.start,
            end: span.end,
        }
    }
}

impl Analyzer {
    fn analyze_statements(&mut self, statements: &[Stmt]) {
        for statement in statements {
            self.analyze_statement(statement);
        }
    }

    fn analyze_statement(&mut self, statement: &Stmt) {
        match statement {
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                self.resolve_variable(&statement.name, statement.span);
                for expr in &statement.args {
                    self.analyze_expr(expr);
                }
            }
            Stmt::FunctionDecl(statement) => self.analyze_function_decl(statement),
            Stmt::Assign(statement) => {
                let ty = self.analyze_expr(&statement.value);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Let(statement) => {
                let value_ty = self.analyze_expr(&statement.value);
                let ty = statement
                    .ty
                    .as_ref()
                    .map(|ty| Type::Named(ty.clone()))
                    .unwrap_or(value_ty);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::AssignRef(statement) => {
                let ty = self
                    .resolve_variable(&statement.target, statement.span)
                    .unwrap_or(Type::Unknown);
                self.bind_variable(&statement.name, ty, statement.span);
            }
            Stmt::Return(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Yield(statement) => {
                self.analyze_expr(&statement.value);
            }
            Stmt::Expr(statement) => {
                self.analyze_expr(&statement.expr);
            }
            Stmt::Namespace(_)
            | Stmt::Use(_)
            | Stmt::Import(_)
            | Stmt::ClassDecl(_)
            | Stmt::TypeDecl(_)
            | Stmt::Break(_) => {}
            Stmt::Loop(statement) => self.analyze_statements(&statement.body),
            Stmt::If(statement) => {
                self.analyze_expr(&statement.condition);
                self.analyze_statements(&statement.body);
            }
            Stmt::Append(statement) => {
                let target_ty = self.resolve_variable(&statement.target, statement.span);
                if let Some(ty) = target_ty {
                    if !ty.allows_php_append_syntax() {
                        self.diagnostics.push(Diagnostic::new(
                            format!(
                                "PHP array append syntax requires array target, found {}",
                                ty.display_name()
                            ),
                            statement.span,
                        ));
                    }
                }
                self.analyze_expr(&statement.value);
            }
        }
    }

    fn analyze_function_decl(&mut self, statement: &FunctionDeclStmt) {
        let saved_variables = self.variables.clone();
        for param in &statement.params {
            let ty = param
                .ty
                .as_ref()
                .map(|ty| Type::Named(ty.clone()))
                .unwrap_or(Type::Unknown);
            self.bind_variable(&param.name, ty, statement.span);
        }
        self.analyze_statements(&statement.body);
        self.variables = saved_variables;
    }

    fn analyze_expr(&mut self, expr: &Expr) -> Type {
        let ty = match expr {
            Expr::Null(_) => Type::Null,
            Expr::Bool(_) => Type::Bool,
            Expr::String(_) => Type::String,
            Expr::Number(expr) if expr.value.contains(['.', 'e', 'E']) => Type::Float,
            Expr::Number(_) => Type::Int,
            Expr::Variable(expr) => self
                .resolve_variable(&expr.name, expr.span)
                .unwrap_or(Type::Unknown),
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                echo_reflection::function(&expr.name)
                    .and_then(|function| function.return_type.clone())
                    .map(Type::Named)
                    .unwrap_or(Type::Unknown)
            }
            Expr::MethodCall(expr) => {
                self.analyze_expr(&expr.object);
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::StaticCall(expr) => {
                for arg in &expr.args {
                    self.analyze_expr(arg);
                }
                Type::Unknown
            }
            Expr::Assign(expr) => {
                let ty = self.analyze_expr(&expr.value);
                self.variables.insert(
                    expr.name.clone(),
                    VariableInfo {
                        name: expr.name.clone(),
                        ty: ty.clone(),
                        span: expr.span,
                    },
                );
                ty
            }
            Expr::MagicConstant(_) => Type::String,
            Expr::Require(expr) => {
                self.analyze_expr(&expr.path);
                Type::Bool
            }
            Expr::Defer(expr) => {
                self.analyze_statements(&expr.body);
                Type::Task
            }
            Expr::Run(expr) => {
                match expr {
                    echo_ast::RunExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::RunExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                    echo_ast::RunExpr::Group { entries, .. } => {
                        for entry in entries {
                            self.analyze_statements(entry);
                        }
                        return Type::List;
                    }
                }
                Type::Task
            }
            Expr::Fork(expr) => {
                match expr {
                    echo_ast::ForkExpr::Block { body, .. } => self.analyze_statements(body),
                    echo_ast::ForkExpr::Task { expr, .. } => {
                        self.analyze_expr(expr);
                    }
                }
                Type::Thread
            }
            Expr::Spawn(expr) => {
                self.analyze_expr(&expr.command);
                Type::Process
            }
            Expr::Join(expr) => match self.analyze_expr(&expr.handle) {
                Type::Process => Type::Int,
                Type::Task | Type::Thread | Type::Unknown => Type::Unknown,
                _ => Type::Unknown,
            },
            Expr::Loop(expr) => {
                self.analyze_statements(&expr.body);
                Type::Unknown
            }
            Expr::Unary(expr) => {
                self.analyze_expr(&expr.expr);
                match expr.op {
                    UnaryOp::Plus | UnaryOp::Minus => Type::Number,
                }
            }
            Expr::Binary(expr) => {
                self.analyze_expr(&expr.left);
                self.analyze_expr(&expr.right);
                match expr.op {
                    BinaryOp::Add
                    | BinaryOp::Sub
                    | BinaryOp::Mul
                    | BinaryOp::Div
                    | BinaryOp::Mod
                    | BinaryOp::Pow => Type::Number,
                    BinaryOp::Concat => Type::String,
                    BinaryOp::Is | BinaryOp::IsNot => Type::Bool,
                }
            }
            Expr::Field(expr) => {
                self.analyze_expr(&expr.object);
                Type::Unknown
            }
            Expr::Index(expr) => {
                let collection_ty = self.analyze_expr(&expr.collection);
                self.analyze_expr(&expr.index);
                if !collection_ty.allows_index_access() {
                    self.diagnostics.push(Diagnostic::new(
                        format!(
                            "index access requires array or list target, found {}",
                            collection_ty.display_name()
                        ),
                        expr.span,
                    ));
                }
                Type::Unknown
            }
            Expr::Object(expr) => {
                for field in &expr.fields {
                    self.analyze_expr(&field.value);
                }
                Type::Object(Some(expr.name.clone()))
            }
            Expr::List(expr) => {
                for value in &expr.values {
                    self.analyze_expr(value);
                }
                Type::List
            }
            Expr::Array(expr) => {
                for element in &expr.elements {
                    if let Some(key) = &element.key {
                        self.analyze_expr(key);
                    }
                    self.analyze_expr(&element.value);
                }
                Type::Array
            }
        };
        self.expression_types
            .insert(SpanKey::from(expr.span()), ty.clone());
        ty
    }

    fn bind_variable(&mut self, name: &str, ty: Type, span: Span) {
        self.variables.insert(
            name.to_string(),
            VariableInfo {
                name: name.to_string(),
                ty,
                span,
            },
        );
    }

    fn resolve_variable(&mut self, name: &str, span: Span) -> Option<Type> {
        let ty = self.variables.get(name).map(|variable| variable.ty.clone());
        if ty.is_none() {
            self.diagnostics.push(Diagnostic::new(
                format!("undefined variable `${name}`"),
                span,
            ));
        }
        ty
    }
}

impl Type {
    fn allows_php_append_syntax(&self) -> bool {
        match self {
            Self::Array => true,
            Self::Named(name) => !name.contains('[') && name.starts_with("array"),
            _ => false,
        }
    }

    fn allows_index_access(&self) -> bool {
        match self {
            Self::Array | Self::List => true,
            Self::Named(name) => name.starts_with("array") || name.starts_with("list"),
            Self::Unknown => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use echo_ast::{
        AppendStmt, ArrayExpr, AssignExpr, AssignStmt, BinaryExpr, ClassDeclStmt, ExprStmt,
        ForkExpr, FunctionCallExpr, FunctionDeclStmt, IfStmt, ImportStmt, IndexExpr, JoinExpr,
        LetStmt, ListExpr, MagicConstantExpr, MethodDecl, NamespaceStmt, NumberLiteral, ObjectExpr,
        ObjectField, QualifiedName, RequireExpr, RunExpr, SpawnExpr, StaticCallExpr, StringLiteral,
        TypeDeclStmt, UseStmt, VariableExpr,
    };

    use super::*;

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_dir: None,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn tracks_variable_type_from_let_initializer() {
        let variable = Expr::Variable(VariableExpr {
            name: "a".to_string(),
            span: Span::new(12, 14),
        });
        let analysis = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Expr(ExprStmt {
                expr: variable.clone(),
                span: Span::new(12, 14),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("a").map(|info| &info.ty),
            Some(&Type::Array)
        );
        assert_eq!(analysis.expression_type(&variable), Type::Array);
    }

    #[test]
    fn reports_undefined_variable() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Variable(VariableExpr {
                name: "missing".to_string(),
                span: Span::new(0, 8),
            }),
            span: Span::new(0, 8),
        })]))
        .expect_err("undefined variable should be diagnostic");

        assert_eq!(diagnostics[0].message, "undefined variable `$missing`");
        assert_eq!(diagnostics[0].span, Span::new(0, 8));
    }

    #[test]
    fn infers_literal_and_operator_types() {
        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::String(StringLiteral {
                value: "x".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::String);

        let analysis = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Number(NumberLiteral {
                value: "1.5".to_string(),
                span: Span::new(0, 3),
            }),
            span: Span::new(0, 3),
        })]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type_at(Span::new(0, 3)), Type::Float);
    }

    #[test]
    fn infers_concurrency_handle_types() {
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(
            analysis.variable("task").map(|info| &info.ty),
            Some(&Type::Task)
        );
        assert_eq!(
            analysis.variable("thread").map(|info| &info.ty),
            Some(&Type::Thread)
        );
        assert_eq!(
            analysis.variable("process").map(|info| &info.ty),
            Some(&Type::Process)
        );
    }

    #[test]
    fn infers_join_type_from_handle_variable_type() {
        let task_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "task".to_string(),
                span: Span::new(32, 37),
            })),
            span: Span::new(27, 37),
        });
        let thread_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "thread".to_string(),
                span: Span::new(48, 55),
            })),
            span: Span::new(43, 55),
        });
        let process_join = Expr::Join(JoinExpr {
            handle: Box::new(Expr::Variable(VariableExpr {
                name: "process".to_string(),
                span: Span::new(66, 75),
            })),
            span: Span::new(61, 75),
        });
        let analysis = analyze(&program(vec![
            Stmt::Assign(AssignStmt {
                name: "task".to_string(),
                value: Expr::Run(RunExpr::Block {
                    body: vec![],
                    span: Span::new(0, 8),
                }),
                span: Span::new(0, 8),
            }),
            Stmt::Assign(AssignStmt {
                name: "thread".to_string(),
                value: Expr::Fork(ForkExpr::Block {
                    body: vec![],
                    span: Span::new(9, 18),
                }),
                span: Span::new(9, 18),
            }),
            Stmt::Assign(AssignStmt {
                name: "process".to_string(),
                value: Expr::Spawn(SpawnExpr {
                    command: Box::new(Expr::String(StringLiteral {
                        value: "true".to_string(),
                        span: Span::new(25, 31),
                    })),
                    span: Span::new(19, 31),
                }),
                span: Span::new(19, 31),
            }),
            Stmt::Expr(ExprStmt {
                expr: task_join.clone(),
                span: Span::new(27, 37),
            }),
            Stmt::Expr(ExprStmt {
                expr: thread_join.clone(),
                span: Span::new(43, 55),
            }),
            Stmt::Expr(ExprStmt {
                expr: process_join.clone(),
                span: Span::new(61, 75),
            }),
        ]))
        .expect("program should analyze");

        assert_eq!(analysis.expression_type(&task_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&thread_join), Type::Unknown);
        assert_eq!(analysis.expression_type(&process_join), Type::Int);
    }

    #[test]
    fn allows_php_append_syntax_for_arrays() {
        analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect("array append should analyze");
    }

    #[test]
    fn rejects_php_append_syntax_for_lists() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: None,
                value: Expr::List(ListExpr {
                    values: vec![],
                    span: Span::new(9, 11),
                }),
                span: Span::new(0, 11),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(19, 20),
                }),
                span: Span::new(12, 21),
            }),
        ]))
        .expect_err("list append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found list"
        );
    }

    #[test]
    fn rejects_php_append_syntax_for_fixed_size_arrays() {
        let diagnostics = analyze(&program(vec![
            Stmt::Let(LetStmt {
                name: "a".to_string(),
                ty: Some("array<int>[3]".to_string()),
                value: Expr::Array(ArrayExpr {
                    elements: vec![],
                    span: Span::new(22, 24),
                }),
                span: Span::new(0, 24),
            }),
            Stmt::Append(AppendStmt {
                target: "a".to_string(),
                value: Expr::Number(NumberLiteral {
                    value: "1".to_string(),
                    span: Span::new(32, 33),
                }),
                span: Span::new(25, 34),
            }),
        ]))
        .expect_err("fixed-size array append should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "PHP array append syntax requires array target, found array<int>[3]"
        );
    }

    #[test]
    fn allows_index_access_for_arrays_and_lists() {
        analyze(&program(vec![
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::Array(ArrayExpr {
                        elements: vec![],
                        span: Span::new(0, 2),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(3, 4),
                    }),
                    span: Span::new(0, 5),
                })),
                span: Span::new(0, 5),
            }),
            Stmt::Expr(ExprStmt {
                expr: Expr::Index(Box::new(IndexExpr {
                    collection: Expr::List(ListExpr {
                        values: vec![],
                        span: Span::new(6, 8),
                    }),
                    index: Expr::Number(NumberLiteral {
                        value: "0".to_string(),
                        span: Span::new(9, 10),
                    }),
                    span: Span::new(6, 11),
                })),
                span: Span::new(6, 11),
            }),
        ]))
        .expect("array and list index access should analyze");
    }

    #[test]
    fn rejects_index_access_for_objects() {
        let diagnostics = analyze(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Index(Box::new(IndexExpr {
                collection: Expr::Object(ObjectExpr {
                    name: String::new(),
                    fields: vec![ObjectField {
                        name: "a".to_string(),
                        value: Expr::Number(NumberLiteral {
                            value: "1".to_string(),
                            span: Span::new(5, 6),
                        }),
                    }],
                    span: Span::new(0, 7),
                }),
                index: Expr::String(StringLiteral {
                    value: "a".to_string(),
                    span: Span::new(8, 11),
                }),
                span: Span::new(0, 12),
            })),
            span: Span::new(0, 12),
        })]))
        .expect_err("object index access should be rejected");

        assert_eq!(
            diagnostics[0].message,
            "index access requires array or list target, found object"
        );
    }

    #[test]
    fn extracts_declaration_facts_with_namespace() {
        let facts = index_facts(
            &program(vec![
                Stmt::Namespace(NamespaceStmt {
                    source: NamespaceSource::Php,
                    name: QualifiedName::new(vec!["App".to_string(), "Http".to_string()]),
                    span: Span::new(0, 19),
                }),
                Stmt::FunctionDecl(FunctionDeclStmt {
                    name: "handler".to_string(),
                    params: vec![],
                    return_type: Some("string".to_string()),
                    is_intrinsic: false,
                    is_generator: false,
                    body: vec![],
                    span: Span::new(20, 52),
                }),
                Stmt::ClassDecl(ClassDeclStmt {
                    name: "UserController".to_string(),
                    members: vec![ClassMember::Method(MethodDecl {
                        name: "show".to_string(),
                        params: vec![],
                        return_type: None,
                        is_static: false,
                        is_intrinsic: false,
                        span: Span::new(76, 88),
                    })],
                    span: Span::new(53, 90),
                }),
                Stmt::TypeDecl(TypeDeclStmt {
                    name: "Payload".to_string(),
                    fields: vec![],
                    span: Span::new(91, 106),
                }),
            ]),
            FileId(7),
            EchoFileMode::PhpCompat,
        );

        assert_eq!(facts.file_id, FileId(7));
        assert_eq!(facts.mode, EchoFileMode::PhpCompat);
        assert_eq!(
            facts
                .declarations
                .iter()
                .map(|symbol| (symbol.name.text.as_str(), symbol.kind))
                .collect::<Vec<_>>(),
            vec![
                ("App\\Http", SymbolKind::Namespace),
                ("handler", SymbolKind::Function),
                ("UserController", SymbolKind::Class),
                ("show", SymbolKind::Method),
                ("Payload", SymbolKind::TypeAlias),
            ]
        );
        assert_eq!(
            facts.declarations[1]
                .fq_name
                .as_ref()
                .map(FqName::as_string),
            Some("App\\Http\\handler".to_string())
        );
        assert_eq!(
            facts.declarations[3]
                .fq_name
                .as_ref()
                .map(FqName::as_string),
            Some("App\\Http\\UserController::show".to_string())
        );
    }

    #[test]
    fn extracts_import_dependency_facts() {
        let facts = index_facts(
            &program(vec![
                Stmt::Use(UseStmt {
                    name: QualifiedName::new(vec![
                        "Psr".to_string(),
                        "Log".to_string(),
                        "LoggerInterface".to_string(),
                    ]),
                    alias: Some("Logger".to_string()),
                    span: Span::new(0, 36),
                }),
                Stmt::Import(ImportStmt {
                    source: ImportSource::Std,
                    name: QualifiedName::new(vec!["net".to_string(), "TcpServer".to_string()]),
                    alias: None,
                    span: Span::new(37, 63),
                }),
                Stmt::Import(ImportStmt {
                    source: ImportSource::File("./routes.echo".to_string()),
                    name: QualifiedName::new(vec!["route".to_string()]),
                    alias: Some("appRoute".to_string()),
                    span: Span::new(64, 112),
                }),
            ]),
            FileId(9),
            EchoFileMode::Echo,
        );

        assert_eq!(
            facts
                .dependencies
                .iter()
                .map(|dependency| (
                    dependency.kind,
                    dependency.target.as_str(),
                    dependency.alias.as_deref()
                ))
                .collect::<Vec<_>>(),
            vec![
                (
                    DependencyKind::PhpUse,
                    "Psr\\Log\\LoggerInterface",
                    Some("Logger")
                ),
                (DependencyKind::EchoStdImport, "net\\TcpServer", None),
                (
                    DependencyKind::EchoFileImport,
                    "./routes.echo#route",
                    Some("appRoute")
                ),
            ]
        );
    }

    #[test]
    fn extracts_require_dependency_facts_from_expressions() {
        let mut source = program(vec![
            Stmt::If(IfStmt {
                condition: Expr::FunctionCall(FunctionCallExpr {
                    name: "file_exists".to_string(),
                    args: vec![Expr::Assign(Box::new(AssignExpr {
                        name: "maintenance".to_string(),
                        value: Expr::Binary(Box::new(BinaryExpr {
                            left: Expr::MagicConstant(MagicConstantExpr {
                                kind: MagicConstantKind::Dir,
                                span: Span::new(20, 27),
                            }),
                            op: BinaryOp::Concat,
                            right: Expr::String(StringLiteral {
                                value: "/../storage/framework/maintenance.php".to_string(),
                                span: Span::new(28, 67),
                            }),
                            span: Span::new(20, 67),
                        })),
                        span: Span::new(5, 67),
                    }))],
                    span: Span::new(0, 68),
                }),
                body: vec![Stmt::Expr(ExprStmt {
                    expr: Expr::Require(Box::new(RequireExpr {
                        kind: RequireKind::Require,
                        path: Expr::Variable(VariableExpr {
                            name: "maintenance".to_string(),
                            span: Span::new(82, 94),
                        }),
                        span: Span::new(74, 94),
                    })),
                    span: Span::new(74, 95),
                })],
                span: Span::new(0, 100),
            }),
            Stmt::Expr(ExprStmt {
                expr: Expr::Require(Box::new(RequireExpr {
                    kind: RequireKind::Require,
                    path: Expr::Binary(Box::new(BinaryExpr {
                        left: Expr::MagicConstant(MagicConstantExpr {
                            kind: MagicConstantKind::Dir,
                            span: Span::new(108, 115),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "/../vendor/autoload.php".to_string(),
                            span: Span::new(116, 141),
                        }),
                        span: Span::new(108, 141),
                    })),
                    span: Span::new(100, 141),
                })),
                span: Span::new(100, 142),
            }),
            Stmt::Assign(AssignStmt {
                name: "app".to_string(),
                value: Expr::Require(Box::new(RequireExpr {
                    kind: RequireKind::RequireOnce,
                    path: Expr::Binary(Box::new(BinaryExpr {
                        left: Expr::MagicConstant(MagicConstantExpr {
                            kind: MagicConstantKind::Dir,
                            span: Span::new(163, 170),
                        }),
                        op: BinaryOp::Concat,
                        right: Expr::String(StringLiteral {
                            value: "/../bootstrap/app.php".to_string(),
                            span: Span::new(171, 194),
                        }),
                        span: Span::new(163, 194),
                    })),
                    span: Span::new(150, 194),
                })),
                span: Span::new(143, 195),
            }),
        ]);
        source.source_dir = Some("/project/public".to_string());

        let facts = index_facts(&source, FileId(10), EchoFileMode::PhpCompat);

        assert_eq!(
            facts
                .dependencies
                .iter()
                .map(|dependency| (dependency.kind, dependency.target.as_str()))
                .collect::<Vec<_>>(),
            vec![
                (DependencyKind::Require, "$maintenance"),
                (
                    DependencyKind::ComposerAutoload,
                    "/project/public/../vendor/autoload.php"
                ),
                (
                    DependencyKind::RequireOnce,
                    "/project/public/../bootstrap/app.php"
                ),
            ]
        );
        assert_eq!(facts.dependencies[1].range, TextRange::new(100, 141));
        assert_eq!(facts.dependencies[1].target_range, TextRange::new(108, 141));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.name == "/project/public/../storage/framework/maintenance.php"
                && reference.range == TextRange::new(20, 67)
        }));
        assert!(facts.references.iter().any(|reference| {
            reference.kind == ReferenceKind::FilePath
                && reference.name == "/project/public/../vendor/autoload.php"
                && reference.range == TextRange::new(108, 141)
        }));
    }

    #[test]
    fn extracts_static_class_reference_facts() {
        let facts = index_facts(
            &program(vec![
                Stmt::Use(UseStmt {
                    name: QualifiedName::new(vec![
                        "Illuminate".to_string(),
                        "Http".to_string(),
                        "Request".to_string(),
                    ]),
                    alias: None,
                    span: Span::new(0, 30),
                }),
                Stmt::Expr(ExprStmt {
                    expr: Expr::StaticCall(StaticCallExpr {
                        class_name: QualifiedName::new(vec!["Request".to_string()]),
                        method: "capture".to_string(),
                        args: Vec::new(),
                        span: Span::new(31, 49),
                    }),
                    span: Span::new(31, 50),
                }),
            ]),
            FileId(11),
            EchoFileMode::PhpCompat,
        );

        assert_eq!(facts.references.len(), 2);
        assert_eq!(facts.references[0].kind, ReferenceKind::ClassLike);
        assert_eq!(facts.references[0].name, "Request");
        assert_eq!(facts.references[0].range, TextRange::new(31, 38));
        assert_eq!(facts.references[1].kind, ReferenceKind::StaticMethod);
        assert_eq!(facts.references[1].name, "capture");
        assert_eq!(facts.references[1].qualifier.as_deref(), Some("Request"));
        assert_eq!(facts.references[1].range, TextRange::new(40, 47));
    }

    #[test]
    fn resolves_plain_require_string_relative_to_source_dir() {
        let mut source = program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Require(Box::new(RequireExpr {
                kind: RequireKind::RequireOnce,
                path: Expr::String(StringLiteral {
                    value: "../bootstrap/app.php".to_string(),
                    span: Span::new(14, 36),
                }),
                span: Span::new(0, 36),
            })),
            span: Span::new(0, 37),
        })]);
        source.source_dir = Some("/project/public".to_string());

        let facts = index_facts(&source, FileId(13), EchoFileMode::PhpCompat);

        assert_eq!(facts.dependencies.len(), 1);
        assert_eq!(facts.dependencies[0].kind, DependencyKind::RequireOnce);
        assert_eq!(
            facts.dependencies[0].target,
            "/project/public/../bootstrap/app.php"
        );
        assert_eq!(facts.dependencies[0].target_range, TextRange::new(14, 36));
    }

    #[test]
    fn extracts_phpdoc_var_local_variable_fact_from_source() {
        let program = program(vec![Stmt::Assign(AssignStmt {
            name: "app".to_string(),
            value: Expr::Require(Box::new(RequireExpr {
                kind: RequireKind::RequireOnce,
                path: Expr::String(StringLiteral {
                    value: "/bootstrap/app.php".to_string(),
                    span: Span::new(49, 69),
                }),
                span: Span::new(36, 69),
            })),
            span: Span::new(29, 70),
        })]);
        let facts = index_facts_from_source(
            "/** @var Application $app */\n$app = require_once '/bootstrap/app.php';",
            &program,
            FileId(12),
            EchoFileMode::PhpCompat,
        );

        let symbol = facts
            .declarations
            .iter()
            .find(|symbol| symbol.kind == SymbolKind::LocalVariable)
            .expect("phpdoc variable symbol");

        assert_eq!(symbol.name.text.as_str(), "app");
        assert_eq!(
            symbol
                .signature
                .as_ref()
                .map(|signature| signature.text.as_str()),
            Some("Application")
        );
        assert_eq!(symbol.selection_range, TextRange::new(21, 25));
    }
}
