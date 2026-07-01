use echo_ast::{BinaryOp, Expr, IncludeKind, MagicConstantKind};
use echo_index::{DependencyFact, DependencyKind, ReferenceFact, ReferenceKind, TextRange};

use super::{IndexFactExtractor, span_range};

impl IndexFactExtractor {
    pub(super) fn extract_expr_dependencies(&mut self, expr: &Expr) {
        match expr {
            Expr::FunctionCall(expr) => {
                for arg in &expr.args {
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Expr::Print(expr) => self.extract_expr_dependencies(&expr.value),
            Expr::DynamicFunctionCall(expr) => {
                for arg in &expr.args {
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Expr::DynamicCall(expr) => {
                self.extract_expr_dependencies(&expr.callee);
                for arg in &expr.args {
                    self.extract_expr_dependencies(&arg.value);
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
                    self.extract_expr_dependencies(&arg.value);
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
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Expr::New(expr) => {
                match &expr.target {
                    echo_ast::NewTarget::Class(class_name) => {
                        let name = class_name.as_string();
                        self.references.push(ReferenceFact {
                            kind: ReferenceKind::ClassLike,
                            range: TextRange::new(
                                expr.span.start.saturating_add(4) as u32,
                                expr.span.start.saturating_add(4 + name.len()) as u32,
                            ),
                            name,
                            qualifier: None,
                        });
                    }
                    echo_ast::NewTarget::Expr(target) => {
                        self.extract_expr_dependencies(target);
                    }
                    echo_ast::NewTarget::AnonymousClass(class) => {
                        if let Some(parent) = &class.parent {
                            self.references.push(ReferenceFact {
                                kind: ReferenceKind::ClassLike,
                                range: span_range(class.span),
                                name: parent.as_string(),
                                qualifier: None,
                            });
                        }
                        for interface in &class.interfaces {
                            self.references.push(ReferenceFact {
                                kind: ReferenceKind::ClassLike,
                                range: span_range(class.span),
                                name: interface.as_string(),
                                qualifier: None,
                            });
                        }
                        for member in &class.members {
                            match member {
                                echo_ast::ClassMember::Method(method) => {
                                    self.extract_statements(&method.body);
                                }
                                echo_ast::ClassMember::Property(property) => {
                                    if let Some(value) = &property.value {
                                        self.extract_expr_dependencies(value);
                                    }
                                }
                                echo_ast::ClassMember::Const(constant) => {
                                    self.extract_expr_dependencies(&constant.value);
                                }
                                echo_ast::ClassMember::TraitUse(_) => {}
                            }
                        }
                    }
                }
                for arg in &expr.args {
                    self.extract_expr_dependencies(&arg.value);
                }
            }
            Expr::Closure(expr) => self.extract_statements(&expr.body),
            Expr::ArrowFunction(expr) => self.extract_expr_dependencies(&expr.body),
            Expr::Assign(expr) => self.extract_expr_dependencies(&expr.value),
            Expr::StaticPropertyFetch(expr) => {
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    range: TextRange::new(
                        expr.span.start as u32,
                        expr.span
                            .start
                            .saturating_add(expr.class_name.as_string().len())
                            as u32,
                    ),
                    name: expr.class_name.as_string(),
                    qualifier: None,
                });
            }
            Expr::StaticPropertyAssign(expr) | Expr::StaticPropertyCoalesceAssign(expr) => {
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    range: TextRange::new(
                        expr.span.start as u32,
                        expr.span
                            .start
                            .saturating_add(expr.class_name.as_string().len())
                            as u32,
                    ),
                    name: expr.class_name.as_string(),
                    qualifier: None,
                });
                self.extract_expr_dependencies(&expr.value);
            }
            Expr::ClassConstantFetch(expr) => {
                self.references.push(ReferenceFact {
                    kind: ReferenceKind::ClassLike,
                    range: TextRange::new(
                        expr.span.start as u32,
                        expr.span
                            .start
                            .saturating_add(expr.class_name.as_string().len())
                            as u32,
                    ),
                    name: expr.class_name.as_string(),
                    qualifier: None,
                });
            }
            Expr::Include(expr) => {
                let target = self
                    .const_string_expr(&expr.path)
                    .map(|target| self.resolve_source_path(&target))
                    .unwrap_or_else(|| expr_path_label(&expr.path));
                self.dependencies.push(DependencyFact {
                    kind: include_dependency_kind(&expr.kind, &target),
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
            Expr::Cast(expr) => self.extract_expr_dependencies(&expr.expr),
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
            Expr::Ternary(expr) => {
                self.extract_expr_dependencies(&expr.condition);
                self.extract_expr_dependencies(&expr.if_true);
                self.extract_expr_dependencies(&expr.if_false);
            }
            Expr::Match(expr) => {
                self.extract_expr_dependencies(&expr.subject);
                for arm in &expr.arms {
                    for condition in &arm.conditions {
                        self.extract_expr_dependencies(condition);
                    }
                    self.extract_expr_dependencies(&arm.value);
                }
            }
            Expr::TypeAscription(expr) => self.extract_expr_dependencies(&expr.expr),
            Expr::Field(expr) => self.extract_expr_dependencies(&expr.object),
            Expr::Index(expr) => {
                self.extract_expr_dependencies(&expr.collection);
                self.extract_expr_dependencies(&expr.index);
            }
            Expr::TargetAssign(expr) => {
                self.extract_expr_dependencies(&expr.target);
                self.extract_expr_dependencies(&expr.value);
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
            | Expr::Constant(_)
            | Expr::ReceiverConst(_)
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

fn include_dependency_kind(kind: &IncludeKind, target: &str) -> DependencyKind {
    if target.ends_with("/vendor/autoload.php") || target.ends_with("\\vendor\\autoload.php") {
        DependencyKind::ComposerAutoload
    } else {
        match kind {
            IncludeKind::Require => DependencyKind::Require,
            IncludeKind::RequireOnce => DependencyKind::RequireOnce,
            IncludeKind::Include => DependencyKind::Include,
            IncludeKind::IncludeOnce => DependencyKind::IncludeOnce,
        }
    }
}

fn expr_path_label(expr: &Expr) -> String {
    match expr {
        Expr::Variable(expr) => format!("${}", expr.name),
        _ => "<dynamic path>".to_string(),
    }
}
