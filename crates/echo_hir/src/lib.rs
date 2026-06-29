use echo_ast::{Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_semantics::Analysis;

#[derive(Debug, Clone)]
pub struct HirProgram {
    source: Program,
    statements: Vec<HirStmt>,
    analysis: Analysis,
}

impl HirProgram {
    pub fn source(&self) -> &Program {
        &self.source
    }

    pub fn statements(&self) -> &[HirStmt] {
        &self.statements
    }

    pub fn analysis(&self) -> &Analysis {
        &self.analysis
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum HirStmt {
    Syntax(Stmt),
}

impl HirStmt {
    pub fn syntax(&self) -> &Stmt {
        match self {
            Self::Syntax(statement) => statement,
        }
    }
}

pub fn lower_program(program: &Program) -> Result<HirProgram, Vec<Diagnostic>> {
    let analysis = echo_semantics::analyze(program)?;
    let statements = program
        .statements
        .iter()
        .cloned()
        .map(HirStmt::Syntax)
        .collect();

    Ok(HirProgram {
        source: program.clone(),
        statements,
        analysis,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::{Expr, ExprStmt, Stmt, VariableExpr};
    use echo_source::Span;

    fn program(statements: Vec<Stmt>) -> Program {
        Program {
            open_tag: None,
            statements,
            source_id: None,
            source_dir: None,
            span: Span::new(0, 0),
        }
    }

    #[test]
    fn lower_program_preserves_syntax() {
        let program = program(vec![]);
        let hir = lower_program(&program).expect("empty program should lower");

        assert_eq!(hir.source(), &program);
        assert_eq!(hir.statements(), &[]);
    }

    #[test]
    fn lower_program_uses_shared_semantic_analysis() {
        let diagnostics = lower_program(&program(vec![Stmt::Expr(ExprStmt {
            expr: Expr::Variable(VariableExpr {
                name: "missing".to_string(),
                span: Span::new(0, 8),
            }),
            span: Span::new(0, 8),
        })]))
        .expect_err("undefined variable should fail HIR lowering");

        assert_eq!(diagnostics[0].message, "undefined variable `$missing`");
    }
}
