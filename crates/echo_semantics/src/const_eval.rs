//! Compile-time evaluation of `#` constant expressions.
//!
//! Authority: `docs/syntax.md` — **Const `#`**: literals + ops on other `#` only;
//! **no calls**.

use std::collections::HashMap;

use echo_ast::{BinaryOp, Expr, UnaryOp};

/// Value of a `#` constant after evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstValue {
    Int(i64),
    Bool(bool),
    /// Decoded UTF-8 payload (no quotes).
    Str(Vec<u8>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConstError {
    pub message: String,
}

impl ConstError {
    fn new(msg: impl Into<String>) -> Self {
        Self {
            message: msg.into(),
        }
    }
}

/// Evaluate `expr` in a map of already-defined `#` names.
pub fn eval_const_expr(
    expr: &Expr,
    env: &HashMap<String, ConstValue>,
) -> Result<ConstValue, ConstError> {
    match expr {
        Expr::Number { text, .. } => {
            let t = text.replace('_', "");
            let is_radix = t.starts_with("0x")
                || t.starts_with("0X")
                || t.starts_with("0b")
                || t.starts_with("0B");
            if !is_radix && (t.contains('.') || t.contains('e') || t.contains('E')) {
                // Width-tagged floats: store truncated i64 for v1 numeric path.
                let f: f64 = t
                    .parse()
                    .map_err(|_| ConstError::new(format!("invalid float const `{text}`")))?;
                Ok(ConstValue::Int(f as i64))
            } else {
                let n = echo_ast::parse_int_literal(text).map_err(ConstError::new)?;
                Ok(ConstValue::Int(n))
            }
        }
        Expr::Bool { value, .. } => Ok(ConstValue::Bool(*value)),
        Expr::String { kind, text, .. } => {
            let bytes = decode_string_token(*kind, text)
                .map_err(ConstError::new)?;
            Ok(ConstValue::Str(bytes))
        }
        Expr::Name(id) => env
            .get(&id.name)
            .cloned()
            .ok_or_else(|| {
                ConstError::new(format!(
                    "`#` const expression may only use literals and other `#` constants (unknown `{name}`)",
                    name = id.name
                ))
            }),
        Expr::Group { expr, .. } => eval_const_expr(expr, env),
        Expr::WidthCast { expr, .. } => {
            // Const width cast: evaluate inner; bit width checks deferred to MIR.
            eval_const_expr(expr, env)
        }
        Expr::Unary { op, expr, .. } => {
            let v = eval_const_expr(expr, env)?;
            match (op, v) {
                (UnaryOp::Neg, ConstValue::Int(n)) => Ok(ConstValue::Int(n.wrapping_neg())),
                (UnaryOp::Not, ConstValue::Bool(b)) => Ok(ConstValue::Bool(!b)),
                (UnaryOp::Not, ConstValue::Int(n)) => Ok(ConstValue::Bool(n == 0)),
                (UnaryOp::BitNot, ConstValue::Int(n)) => Ok(ConstValue::Int(!n)),
                _ => Err(ConstError::new("invalid unary in `#` const expression")),
            }
        }
        Expr::Binary {
            op, left, right, ..
        } => {
            let l = eval_const_expr(left, env)?;
            let r = eval_const_expr(right, env)?;
            eval_binop(*op, l, r)
        }
        Expr::Call { .. } => Err(ConstError::new(
            "`#` const expression cannot call functions",
        )),
        Expr::Field { .. }
        | Expr::Index { .. }
        | Expr::Receiver { .. }
        | Expr::List { .. }
        | Expr::Object { .. }
        | Expr::StructLit { .. }
        | Expr::Fn { .. }
        | Expr::Duration { .. }
        | Expr::Bytes { .. }
        | Expr::Locator { .. }
        | Expr::Range { .. } => Err(ConstError::new(
            "`#` const expression only allows literals and ops on `#` constants",
        )),
    }
}

fn eval_binop(op: BinaryOp, l: ConstValue, r: ConstValue) -> Result<ConstValue, ConstError> {
    use BinaryOp::*;
    match (op, l, r) {
        (Add, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_add(b))),
        (Sub, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_sub(b))),
        (Mul, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_mul(b))),
        (Div, ConstValue::Int(a), ConstValue::Int(b)) => {
            if b == 0 {
                return Err(ConstError::new("division by zero in `#` const"));
            }
            Ok(ConstValue::Int(a / b))
        }
        (Rem, ConstValue::Int(a), ConstValue::Int(b)) => {
            if b == 0 {
                return Err(ConstError::new("remainder by zero in `#` const"));
            }
            Ok(ConstValue::Int(a % b))
        }
        (Eq | EqEqEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a == b)),
        (NotEq | NotEqEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a != b)),
        (Lt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a < b)),
        (Gt, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a > b)),
        (LtEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a <= b)),
        (GtEq, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Bool(a >= b)),
        (And, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a && b)),
        (Or, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a || b)),
        (BitAnd, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a & b)),
        (BitOr, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a | b)),
        (BitXor, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a ^ b)),
        (Shl, ConstValue::Int(a), ConstValue::Int(b)) => {
            Ok(ConstValue::Int(a.wrapping_shl((b as u32) & 63)))
        }
        (Shr, ConstValue::Int(a), ConstValue::Int(b)) => {
            Ok(ConstValue::Int(a.wrapping_shr((b as u32) & 63)))
        }
        (Eq | EqEqEq, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a == b)),
        (NotEq | NotEqEq, ConstValue::Bool(a), ConstValue::Bool(b)) => Ok(ConstValue::Bool(a != b)),
        // No `+` string concat — use rich `"…{name}…"` interpolation instead.
        (Eq | EqEqEq, ConstValue::Str(a), ConstValue::Str(b)) => Ok(ConstValue::Bool(a == b)),
        (NotEq | NotEqEq, ConstValue::Str(a), ConstValue::Str(b)) => Ok(ConstValue::Bool(a != b)),
        _ => Err(ConstError::new(
            "invalid operands for operator in `#` const expression",
        )),
    }
}

fn decode_string_token(
    kind: echo_ast::StringKind,
    raw: &str,
) -> Result<Vec<u8>, String> {
    // Mirror echo_mir decode (pure/rich); keep independent to avoid crate cycles.
    match kind {
        echo_ast::StringKind::Pure => {
            let b = raw.as_bytes();
            if b.len() < 2 || b[0] != b'\'' || b[b.len() - 1] != b'\'' {
                return Err(format!("invalid pure string `{raw}`"));
            }
            Ok(b[1..b.len() - 1].to_vec())
        }
        echo_ast::StringKind::Rich => {
            let b = raw.as_bytes();
            if b.len() < 2 || b[0] != b'"' || b[b.len() - 1] != b'"' {
                return Err(format!("invalid rich string `{raw}`"));
            }
            let inner = &b[1..b.len() - 1];
            let mut out = Vec::new();
            let mut i = 0;
            while i < inner.len() {
                if inner[i] == b'\\' {
                    i += 1;
                    if i >= inner.len() {
                        return Err("rich string ends with lone backslash".into());
                    }
                    match inner[i] {
                        b'n' => out.push(b'\n'),
                        b't' => out.push(b'\t'),
                        b'r' => out.push(b'\r'),
                        b'\\' => out.push(b'\\'),
                        b'"' => out.push(b'"'),
                        b'{' => out.push(b'{'),
                        b'}' => out.push(b'}'),
                        other => out.push(other),
                    }
                    i += 1;
                } else {
                    out.push(inner[i]);
                    i += 1;
                }
            }
            Ok(out)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_ast::Ident;
    use echo_source::{BytePos, SourceMap, Span};

    fn dummy_span() -> Span {
        let mut map = SourceMap::new();
        let id = map.add("t.echo", "");
        Span::new(id, BytePos(0), BytePos(0))
    }

    fn num(n: &str) -> Expr {
        Expr::Number {
            text: n.into(),
            width: None,
            span: dummy_span(),
        }
    }

    fn name(n: &str) -> Expr {
        Expr::Name(Ident {
            name: n.into(),
            span: dummy_span(),
        })
    }

    #[test]
    fn add_literals() {
        let e = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(num("2")),
            right: Box::new(num("3")),
            span: dummy_span(),
        };
        assert_eq!(
            eval_const_expr(&e, &HashMap::new()).unwrap(),
            ConstValue::Int(5)
        );
    }

    #[test]
    fn uses_prior_const() {
        let mut env = HashMap::new();
        env.insert("A".into(), ConstValue::Int(10));
        let e = Expr::Binary {
            op: BinaryOp::Mul,
            left: Box::new(name("A")),
            right: Box::new(num("2")),
            span: dummy_span(),
        };
        assert_eq!(eval_const_expr(&e, &env).unwrap(), ConstValue::Int(20));
    }

    #[test]
    fn rejects_call() {
        let e = Expr::Call {
            callee: Box::new(name("f")),
            args: vec![],
            span: dummy_span(),
        };
        assert!(eval_const_expr(&e, &HashMap::new()).is_err());
    }

    #[test]
    fn rejects_runtime_name() {
        let e = name("x");
        assert!(eval_const_expr(&e, &HashMap::new()).is_err());
    }
}
