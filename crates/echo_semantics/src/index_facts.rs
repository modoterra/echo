use echo_ast::{
    BinaryOp, ClassMember, Expr, FunctionDeclStmt, ImportSource, MagicConstantKind,
    NamespaceSource, Program, RequireKind, Stmt,
};
use echo_index::{
    DependencyFact, DependencyKind, EchoFileMode, FileId, FqName, IndexFacts, ReferenceFact,
    ReferenceKind, Signature, SymbolFact, SymbolKind, SymbolName, TextRange,
};
use echo_source::Span;
use smol_str::SmolStr;

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
            Expr::TypeAscription(expr) => self.extract_expr_dependencies(&expr.expr),
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
