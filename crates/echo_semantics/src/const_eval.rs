//! Compile-time evaluation of `#` constant expressions.
//!
//! Authority: `docs/syntax.md` — **Const `#`**: literals + ops on other `#` only;
//! **no calls**.

use std::collections::HashMap;

use echo_ast::{BinaryOp, Expr, UnaryOp, Width};

/// Value of a `#` constant after evaluation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConstValue {
    Int(i64),
    /// IEEE-754 bits (so the enum stays `Eq`).
    Float(u64),
    Bool(bool),
    /// Decoded UTF-8 payload (no quotes).
    Str(Vec<u8>),
    /// Duration as i64 nanoseconds (same as runtime).
    Duration(i64),
    /// Decoded bytes payload (no `b` prefix / quotes).
    Bytes(Vec<u8>),
    /// Locator UTF-8 text as written (no `p` prefix / quotes).
    Locator(String),
}

impl ConstValue {
    /// Bytes to bake into rich `"…"`, `b"…"`, `p"…"` interpolation of this name.
    ///
    /// Duration bakes as decimal nanoseconds (same as live unboxed duration interp).
    #[must_use]
    pub fn interp_bytes(&self) -> Vec<u8> {
        match self {
            Self::Str(b) | Self::Bytes(b) => b.clone(),
            Self::Locator(s) => s.as_bytes().to_vec(),
            Self::Int(i) | Self::Duration(i) => i.to_string().into_bytes(),
            Self::Float(bits) => f64::from_bits(*bits).to_string().into_bytes(),
            Self::Bool(b) => {
                if *b {
                    b"1".to_vec()
                } else {
                    b"0".to_vec()
                }
            }
        }
    }
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
                let f: f64 = t
                    .parse()
                    .map_err(|_| ConstError::new(format!("invalid float const `{text}`")))?;
                Ok(ConstValue::Float(f.to_bits()))
            } else {
                let n = echo_ast::parse_int_literal(text).map_err(ConstError::new)?;
                Ok(ConstValue::Int(n))
            }
        }
        Expr::Bool { value, .. } => Ok(ConstValue::Bool(*value)),
        Expr::String { kind, text, .. } => {
            let bytes = decode_string_token(*kind, text).map_err(ConstError::new)?;
            Ok(ConstValue::Str(bytes))
        }
        Expr::Duration { text, .. } => {
            let nanos = echo_ast::parse_duration_nanos(text).map_err(ConstError::new)?;
            Ok(ConstValue::Duration(nanos))
        }
        Expr::Bytes { kind, text, .. } => {
            let bytes = decode_prefixed_token(b'b', "bytes", *kind, text)?;
            Ok(ConstValue::Bytes(bytes))
        }
        Expr::Locator { kind, text, .. } => {
            let bytes = decode_prefixed_token(b'p', "locator", *kind, text)?;
            let text = String::from_utf8(bytes)
                .map_err(|_| ConstError::new("locator payload is not UTF-8"))?;
            Ok(ConstValue::Locator(text))
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
        Expr::WidthCast { width, tag, expr, .. } => {
            let Some(w) = width else {
                return Err(ConstError::new(format!(
                    "unknown width tag `{tag}` in `#` const"
                )));
            };
            let v = eval_const_expr(expr, env)?;
            apply_const_cast(v, *w)
        }
        Expr::Unary { op, expr, .. } => {
            let v = eval_const_expr(expr, env)?;
            match (op, v) {
                (UnaryOp::Neg, ConstValue::Int(n)) => Ok(ConstValue::Int(n.wrapping_neg())),
                (UnaryOp::Neg, ConstValue::Float(b)) => {
                    Ok(ConstValue::Float((-f64::from_bits(b)).to_bits()))
                }
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
        | Expr::Range { .. } => Err(ConstError::new(
            "`#` const expression only allows literals and ops on `#` constants",
        )),
    }
}

fn apply_const_cast(v: ConstValue, w: Width) -> Result<ConstValue, ConstError> {
    match (v, w) {
        (ConstValue::Int(n), w) if w.is_int() => Ok(ConstValue::Int(cast_int_bits(n, w))),
        (ConstValue::Int(n), Width::F64) => Ok(ConstValue::Float((n as f64).to_bits())),
        (ConstValue::Int(n), Width::F32) => Ok(ConstValue::Float((n as f32 as f64).to_bits())),
        (ConstValue::Float(b), w) if w.is_int() => {
            let f = f64::from_bits(b);
            Ok(ConstValue::Int(cast_int_bits(f as i64, w)))
        }
        (ConstValue::Float(b), Width::F32) => {
            let f = f64::from_bits(b) as f32;
            Ok(ConstValue::Float((f as f64).to_bits()))
        }
        (ConstValue::Float(b), Width::F64) => Ok(ConstValue::Float(b)),
        _ => Err(ConstError::new(
            "width cast in `#` const only applies to integers and floats",
        )),
    }
}

fn cast_int_bits(n: i64, w: Width) -> i64 {
    match w {
        Width::I8 => n as i8 as i64,
        Width::I16 => n as i16 as i64,
        Width::I32 => n as i32 as i64,
        Width::I64 => n,
        Width::Ui8 => (n as u8) as i64,
        Width::Ui16 => (n as u16) as i64,
        Width::Ui32 => (n as u32) as i64,
        Width::Ui64 => n,
        Width::F32 | Width::F64 => n,
    }
}

fn eval_binop(op: BinaryOp, l: ConstValue, r: ConstValue) -> Result<ConstValue, ConstError> {
    use BinaryOp::*;
    match (op, l, r) {
        (Add, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_add(b))),
        (Add, ConstValue::Duration(a), ConstValue::Duration(b)) => {
            Ok(ConstValue::Duration(a.wrapping_add(b)))
        }
        (Add, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(
            (f64::from_bits(a) + f64::from_bits(b)).to_bits(),
        )),
        (Sub, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_sub(b))),
        (Sub, ConstValue::Duration(a), ConstValue::Duration(b)) => {
            Ok(ConstValue::Duration(a.wrapping_sub(b)))
        }
        (Sub, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(
            (f64::from_bits(a) - f64::from_bits(b)).to_bits(),
        )),
        (Mul, ConstValue::Int(a), ConstValue::Int(b)) => Ok(ConstValue::Int(a.wrapping_mul(b))),
        (Mul, ConstValue::Float(a), ConstValue::Float(b)) => Ok(ConstValue::Float(
            (f64::from_bits(a) * f64::from_bits(b)).to_bits(),
        )),
        (Div, ConstValue::Int(a), ConstValue::Int(b)) => {
            if b == 0 {
                return Err(ConstError::new("division by zero in `#` const"));
            }
            Ok(ConstValue::Int(a / b))
        }
        (Div, ConstValue::Float(a), ConstValue::Float(b)) => {
            let d = f64::from_bits(b);
            if d == 0.0 {
                return Err(ConstError::new("division by zero in `#` const"));
            }
            Ok(ConstValue::Float((f64::from_bits(a) / d).to_bits()))
        }
        (Rem, ConstValue::Int(a), ConstValue::Int(b)) => {
            if b == 0 {
                return Err(ConstError::new("remainder by zero in `#` const"));
            }
            Ok(ConstValue::Int(a % b))
        }
        (Eq | EqEqEq, ConstValue::Float(a), ConstValue::Float(b)) => {
            Ok(ConstValue::Bool(f64::from_bits(a) == f64::from_bits(b)))
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
        (Eq | EqEqEq, ConstValue::Duration(a), ConstValue::Duration(b)) => {
            Ok(ConstValue::Bool(a == b))
        }
        (NotEq | NotEqEq, ConstValue::Duration(a), ConstValue::Duration(b)) => {
            Ok(ConstValue::Bool(a != b))
        }
        (Eq | EqEqEq, ConstValue::Bytes(a), ConstValue::Bytes(b)) => Ok(ConstValue::Bool(a == b)),
        (NotEq | NotEqEq, ConstValue::Bytes(a), ConstValue::Bytes(b)) => {
            Ok(ConstValue::Bool(a != b))
        }
        (Eq | EqEqEq, ConstValue::Locator(a), ConstValue::Locator(b)) => {
            Ok(ConstValue::Bool(a == b))
        }
        (NotEq | NotEqEq, ConstValue::Locator(a), ConstValue::Locator(b)) => {
            Ok(ConstValue::Bool(a != b))
        }
        _ => Err(ConstError::new(
            "invalid operands for operator in `#` const expression",
        )),
    }
}

fn decode_prefixed_token(
    prefix: u8,
    what: &str,
    kind: echo_ast::StringKind,
    raw: &str,
) -> Result<Vec<u8>, ConstError> {
    let b = raw.as_bytes();
    if b.first() != Some(&prefix) {
        return Err(ConstError::new(format!("invalid {what} token `{raw}`")));
    }
    decode_string_token(kind, &raw[1..]).map_err(ConstError::new)
}

fn decode_string_token(kind: echo_ast::StringKind, raw: &str) -> Result<Vec<u8>, String> {
    // Pure copy; rich uses the locked escape table in `echo_syntax`.
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
                    match echo_syntax::decode_escape(&inner[i..]) {
                        Ok((byte, n)) => {
                            out.push(byte);
                            i += n;
                        }
                        Err(e) => return Err(e.to_string()),
                    }
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

    #[test]
    fn float_lit_keeps_fraction() {
        let e = num("1.5");
        match eval_const_expr(&e, &HashMap::new()).unwrap() {
            ConstValue::Float(b) => assert_eq!(f64::from_bits(b), 1.5),
            other => panic!("expected float, got {other:?}"),
        }
    }

    #[test]
    fn rich_const_decodes_locked_escapes() {
        let e = Expr::String {
            kind: echo_ast::StringKind::Rich,
            text: r#""A\x42\{c\}""#.into(),
            span: dummy_span(),
        };
        match eval_const_expr(&e, &HashMap::new()).unwrap() {
            ConstValue::Str(b) => assert_eq!(b, b"AB{c}"),
            other => panic!("expected str, got {other:?}"),
        }
    }

    #[test]
    fn rich_const_rejects_unknown_escape() {
        let e = Expr::String {
            kind: echo_ast::StringKind::Rich,
            text: r#""\q""#.into(),
            span: dummy_span(),
        };
        assert!(eval_const_expr(&e, &HashMap::new()).is_err());
    }

    #[test]
    fn width_cast_int_to_int() {
        let e = Expr::WidthCast {
            width: Some(echo_ast::Width::I8),
            tag: "i8".into(),
            expr: Box::new(num("300")),
            span: dummy_span(),
        };
        assert_eq!(
            eval_const_expr(&e, &HashMap::new()).unwrap(),
            ConstValue::Int(300i64 as i8 as i64)
        );
    }

    fn duration(text: &str) -> Expr {
        Expr::Duration {
            text: text.into(),
            span: dummy_span(),
        }
    }

    fn bytes_pure(text: &str) -> Expr {
        Expr::Bytes {
            kind: echo_ast::StringKind::Pure,
            text: text.into(),
            span: dummy_span(),
        }
    }

    fn locator_pure(text: &str) -> Expr {
        Expr::Locator {
            kind: echo_ast::StringKind::Pure,
            text: text.into(),
            span: dummy_span(),
        }
    }

    #[test]
    fn duration_lit_and_add() {
        let e = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(duration("5s")),
            right: Box::new(duration("10ms")),
            span: dummy_span(),
        };
        assert_eq!(
            eval_const_expr(&e, &HashMap::new()).unwrap(),
            ConstValue::Duration(5_000_000_000 + 10_000_000)
        );
    }

    #[test]
    fn duration_uses_prior_const() {
        let mut env = HashMap::new();
        env.insert("A".into(), ConstValue::Duration(1_000_000_000));
        let e = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(name("A")),
            right: Box::new(duration("500ms")),
            span: dummy_span(),
        };
        assert_eq!(
            eval_const_expr(&e, &env).unwrap(),
            ConstValue::Duration(1_500_000_000)
        );
    }

    #[test]
    fn bytes_pure_lit() {
        assert_eq!(
            eval_const_expr(&bytes_pure("b'raw'"), &HashMap::new()).unwrap(),
            ConstValue::Bytes(b"raw".to_vec())
        );
    }

    #[test]
    fn locator_pure_lit() {
        assert_eq!(
            eval_const_expr(&locator_pure("p'/tmp'"), &HashMap::new()).unwrap(),
            ConstValue::Locator("/tmp".into())
        );
    }

    #[test]
    fn duration_eq_same_nanos() {
        let e = Expr::Binary {
            op: BinaryOp::Eq,
            left: Box::new(duration("5s")),
            right: Box::new(duration("5000ms")),
            span: dummy_span(),
        };
        assert_eq!(
            eval_const_expr(&e, &HashMap::new()).unwrap(),
            ConstValue::Bool(true)
        );
    }

    #[test]
    fn duration_does_not_mix_with_int() {
        let e = Expr::Binary {
            op: BinaryOp::Add,
            left: Box::new(duration("5s")),
            right: Box::new(num("1")),
            span: dummy_span(),
        };
        assert!(eval_const_expr(&e, &HashMap::new()).is_err());
    }
}
