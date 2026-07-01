use echo_ast::{
    ClassMember, EnumMember, FunctionDeclStmt, ImportSource, InterfaceMember, NamespaceSource,
    Program, Stmt,
};
use echo_index::{
    DependencyFact, DependencyKind, FileId, FqName, IndexFacts, ReferenceFact, ReferenceKind,
    Signature, SymbolFact, SymbolKind, SymbolName, TextRange,
};
use echo_source::Span;
use smol_str::SmolStr;

mod dependencies;
mod phpdoc;

use phpdoc::phpdoc_var_annotations;

pub fn index_facts(program: &Program, file_id: FileId) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id);
    extractor.extract(program);
    extractor.into_facts()
}

pub fn index_facts_from_source(source: &str, program: &Program, file_id: FileId) -> IndexFacts {
    let mut extractor = IndexFactExtractor::new(file_id);
    extractor.source_text = Some(source.to_string());
    extractor.source_dir.clone_from(&program.source_dir);
    extractor.extract_phpdoc_var_source_facts(source);
    extractor.extract_statements(&program.statements);
    extractor.into_facts()
}

struct IndexFactExtractor {
    file_id: FileId,
    source_text: Option<String>,
    source_dir: Option<String>,
    namespace: Vec<SmolStr>,
    declarations: Vec<SymbolFact>,
    dependencies: Vec<DependencyFact>,
    references: Vec<ReferenceFact>,
}

impl IndexFactExtractor {
    fn new(file_id: FileId) -> Self {
        Self {
            file_id,
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
            Stmt::Compile(statement) => {
                for entry in &statement.entries {
                    self.dependencies.push(DependencyFact {
                        kind: DependencyKind::Compile,
                        target: entry.value.clone(),
                        alias: None,
                        range: span_range(statement.span),
                        target_range: span_range(entry.span),
                    });
                }
            }
            Stmt::Namespace(statement) => {
                self.namespace = match statement.source {
                    NamespaceSource::Php | NamespaceSource::Echo => statement
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
            Stmt::UnnamedExport(statement) => self.extract_expr_dependencies(&statement.value),
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
                    match member {
                        ClassMember::Method(method) => {
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
                                    text: method_signature(
                                        &method.params,
                                        method.return_type.as_deref(),
                                    ),
                                }),
                            });
                            self.extract_statements(&method.body);
                        }
                        ClassMember::Property(property) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(property.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!(
                                        "{}::${}",
                                        statement.name, property.name
                                    )),
                                ),
                                kind: SymbolKind::Property,
                                range: span_range(property.span),
                                selection_range: self
                                    .span_range_for_text(property.span, &property.name)
                                    .unwrap_or_else(|| span_range(property.span)),
                                visibility: None,
                                signature: None,
                            });
                            if let Some(value) = &property.value {
                                self.extract_expr_dependencies(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(constant.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!("{}::{}", statement.name, constant.name)),
                                ),
                                kind: SymbolKind::Constant,
                                range: span_range(constant.span),
                                selection_range: self
                                    .span_range_for_text(constant.span, &constant.name)
                                    .unwrap_or_else(|| span_range(constant.span)),
                                visibility: None,
                                signature: None,
                            });
                            self.extract_expr_dependencies(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::InterfaceDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Interface,
                    range: span_range(statement.span),
                    selection_range: self
                        .span_range_for_text(statement.span, &statement.name)
                        .unwrap_or_else(|| span_range(statement.span)),
                    visibility: None,
                    signature: None,
                });

                for parent in &statement.parents {
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::ClassLike,
                        name: parent.as_string(),
                        qualifier: None,
                        range: span_range(statement.span),
                    });
                }

                for member in &statement.members {
                    match member {
                        InterfaceMember::Method(method) => {
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
                                    text: method_signature(
                                        &method.params,
                                        method.return_type.as_deref(),
                                    ),
                                }),
                            });
                        }
                        InterfaceMember::Const(constant) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(constant.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!("{}::{}", statement.name, constant.name)),
                                ),
                                kind: SymbolKind::Constant,
                                range: span_range(constant.span),
                                selection_range: self
                                    .span_range_for_text(constant.span, &constant.name)
                                    .unwrap_or_else(|| span_range(constant.span)),
                                visibility: None,
                                signature: None,
                            });
                            self.extract_expr_dependencies(&constant.value);
                        }
                    }
                }
            }
            Stmt::TraitDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Trait,
                    range: span_range(statement.span),
                    selection_range: self
                        .span_range_for_text(statement.span, &statement.name)
                        .unwrap_or_else(|| span_range(statement.span)),
                    visibility: None,
                    signature: None,
                });

                for member in &statement.members {
                    match member {
                        ClassMember::Method(method) => {
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
                                    text: method_signature(
                                        &method.params,
                                        method.return_type.as_deref(),
                                    ),
                                }),
                            });
                            self.extract_statements(&method.body);
                        }
                        ClassMember::Property(property) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(property.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!(
                                        "{}::${}",
                                        statement.name, property.name
                                    )),
                                ),
                                kind: SymbolKind::Property,
                                range: span_range(property.span),
                                selection_range: self
                                    .span_range_for_text(property.span, &property.name)
                                    .unwrap_or_else(|| span_range(property.span)),
                                visibility: None,
                                signature: None,
                            });
                            if let Some(value) = &property.value {
                                self.extract_expr_dependencies(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(constant.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!("{}::{}", statement.name, constant.name)),
                                ),
                                kind: SymbolKind::Constant,
                                range: span_range(constant.span),
                                selection_range: self
                                    .span_range_for_text(constant.span, &constant.name)
                                    .unwrap_or_else(|| span_range(constant.span)),
                                visibility: None,
                                signature: None,
                            });
                            self.extract_expr_dependencies(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::EnumDecl(statement) => {
                self.declarations.push(SymbolFact {
                    name: SymbolName::new(statement.name.as_str()),
                    fq_name: Some(self.fq_name(&statement.name)),
                    kind: SymbolKind::Enum,
                    range: span_range(statement.span),
                    selection_range: self
                        .span_range_for_text(statement.span, &statement.name)
                        .unwrap_or_else(|| span_range(statement.span)),
                    visibility: None,
                    signature: None,
                });

                for interface in &statement.interfaces {
                    self.references.push(ReferenceFact {
                        kind: ReferenceKind::ClassLike,
                        name: interface.as_string(),
                        qualifier: None,
                        range: span_range(statement.span),
                    });
                }

                for member in &statement.members {
                    match member {
                        EnumMember::Case(case) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(case.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!("{}::{}", statement.name, case.name)),
                                ),
                                kind: SymbolKind::Constant,
                                range: span_range(case.span),
                                selection_range: self
                                    .span_range_for_text(case.span, &case.name)
                                    .unwrap_or_else(|| span_range(case.span)),
                                visibility: None,
                                signature: None,
                            });
                            if let Some(value) = &case.value {
                                self.extract_expr_dependencies(value);
                            }
                        }
                        EnumMember::Method(method) => {
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
                                    text: method_signature(
                                        &method.params,
                                        method.return_type.as_deref(),
                                    ),
                                }),
                            });
                            self.extract_statements(&method.body);
                        }
                        EnumMember::TraitUse(_) => {}
                    }
                }
            }
            Stmt::FacetDecl(statement) => {
                for member in &statement.members {
                    match member {
                        ClassMember::Method(method) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(method.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!("{}::{}", statement.target, method.name)),
                                ),
                                kind: SymbolKind::Method,
                                range: span_range(method.span),
                                selection_range: self
                                    .span_range_for_text(method.span, &method.name)
                                    .unwrap_or_else(|| span_range(method.span)),
                                visibility: None,
                                signature: Some(Signature {
                                    text: method_signature(
                                        &method.params,
                                        method.return_type.as_deref(),
                                    ),
                                }),
                            });
                            self.extract_statements(&method.body);
                        }
                        ClassMember::Property(property) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(property.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!(
                                        "{}::${}",
                                        statement.target, property.name
                                    )),
                                ),
                                kind: SymbolKind::Property,
                                range: span_range(property.span),
                                selection_range: self
                                    .span_range_for_text(property.span, &property.name)
                                    .unwrap_or_else(|| span_range(property.span)),
                                visibility: None,
                                signature: None,
                            });
                            if let Some(value) = &property.value {
                                self.extract_expr_dependencies(value);
                            }
                        }
                        ClassMember::Const(constant) => {
                            self.declarations.push(SymbolFact {
                                name: SymbolName::new(constant.name.as_str()),
                                fq_name: Some(
                                    self.fq_name(&format!(
                                        "{}::{}",
                                        statement.target, constant.name
                                    )),
                                ),
                                kind: SymbolKind::Constant,
                                range: span_range(constant.span),
                                selection_range: self
                                    .span_range_for_text(constant.span, &constant.name)
                                    .unwrap_or_else(|| span_range(constant.span)),
                                visibility: None,
                                signature: None,
                            });
                            self.extract_expr_dependencies(&constant.value);
                        }
                        ClassMember::TraitUse(_) => {}
                    }
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
            Stmt::While(statement) => {
                self.extract_expr_dependencies(&statement.condition);
                self.extract_statements(&statement.body);
            }
            Stmt::For(statement) => {
                for expr in &statement.init {
                    self.extract_expr_dependencies(expr);
                }
                for expr in &statement.conditions {
                    self.extract_expr_dependencies(expr);
                }
                for expr in &statement.increments {
                    self.extract_expr_dependencies(expr);
                }
                self.extract_statements(&statement.body);
            }
            Stmt::Foreach(statement) => {
                self.extract_expr_dependencies(&statement.iterable);
                self.extract_statements(&statement.body);
            }
            Stmt::Switch(statement) => {
                self.extract_expr_dependencies(&statement.expr);
                for case in &statement.cases {
                    if let Some(condition) = &case.condition {
                        self.extract_expr_dependencies(condition);
                    }
                    self.extract_statements(&case.body);
                }
            }
            Stmt::If(statement) => {
                self.extract_expr_dependencies(&statement.condition);
                self.extract_statements(&statement.body);
                for clause in &statement.elseif_clauses {
                    self.extract_expr_dependencies(&clause.condition);
                    self.extract_statements(&clause.body);
                }
                self.extract_statements(&statement.else_body);
            }
            Stmt::Try(statement) => {
                self.extract_statements(&statement.body);
                for catch in &statement.catches {
                    self.extract_statements(&catch.body);
                }
                self.extract_statements(&statement.finally_body);
            }
            Stmt::Echo(statement) => {
                for expr in &statement.exprs {
                    self.extract_expr_dependencies(expr);
                }
            }
            Stmt::FunctionCall(statement) => {
                for arg in &statement.args {
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Stmt::DynamicFunctionCall(statement) => {
                for arg in &statement.args {
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Stmt::Assign(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::CoalesceAssign(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::ListAssign(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Let(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::AssignRef(_) | Stmt::Global(_) | Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::StaticVar(statement) => {
                for var in &statement.vars {
                    if let Some(value) = &var.value {
                        self.extract_expr_dependencies(value);
                    }
                }
            }
            Stmt::Return(statement) => {
                if let Some(value) = &statement.value {
                    self.extract_expr_dependencies(value);
                }
            }
            Stmt::Throw(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Yield(statement) => self.extract_expr_dependencies(&statement.value),
            Stmt::Expr(statement) => self.extract_expr_dependencies(&statement.expr),
            Stmt::Append(statement) => {
                self.extract_expr_dependencies(&statement.target);
                self.extract_expr_dependencies(&statement.value);
            }
        }
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
            declarations: self.declarations,
            dependencies: self.dependencies,
            references: self.references,
        }
    }
}

fn span_range(span: Span) -> TextRange {
    TextRange::new(span.start as u32, span.end as u32)
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

#[cfg(test)]
#[path = "index_facts/tests.rs"]
mod tests;
