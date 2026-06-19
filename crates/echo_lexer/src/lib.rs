use echo_diagnostics::Diagnostic;
use echo_source::Span;
use logos::Logos;

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    OpenPhp,
    Echo,
    Ident(String),
    Variable(String),
    String(String),
    Number(String),
    Semicolon,
    Colon,
    Comma,
    OpenParen,
    CloseParen,
    OpenBrace,
    CloseBrace,
    Plus,
    Minus,
    Star,
    Slash,
    Percent,
    Dot,
    Pipe,
    Ampersand,
    Equals,
    Backslash,
    Question,
    LessThan,
    GreaterThan,
    OpenBracket,
    CloseBracket,
    Eof,
}

#[derive(Debug, Clone, PartialEq, Logos)]
enum RawToken {
    #[token("<?php")]
    #[token("<?PHP")]
    OpenPhp,

    #[token("echo")]
    Echo,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice().to_string())]
    Ident(String),

    #[regex(r"\$[A-Za-z_][A-Za-z0-9_]*", |lex| lex.slice()[1..].to_string())]
    Variable(String),

    #[regex(r#""([^"\\]|\\.)*""#, |lex| parse_quoted_string(lex.slice()))]
    #[regex(r#"'([^'\\]|\\.)*'"#, |lex| parse_quoted_string(lex.slice()))]
    String(String),

    #[regex(r"[0-9]+(\.[0-9]+)?([eE][+-]?[0-9]+)?", |lex| lex.slice().to_string())]
    Number(String),

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token(",")]
    Comma,

    #[token("(")]
    OpenParen,

    #[token(")")]
    CloseParen,

    #[token("{")]
    OpenBrace,

    #[token("}")]
    CloseBrace,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("%")]
    Percent,

    #[regex(r"//[^\r\n]*", logos::skip)]
    #[regex(r"#[^\r\n]*", logos::skip)]
    #[regex(r"/\*([^*]|\*[^/])*\*/", logos::skip)]
    Comment,

    #[token(".")]
    Dot,

    #[token("|")]
    Pipe,

    #[token("&")]
    Ampersand,

    #[token("=")]
    Equals,

    #[token("\\")]
    Backslash,

    #[token("?")]
    Question,

    #[token("<")]
    LessThan,

    #[token(">")]
    GreaterThan,

    #[token("[")]
    OpenBracket,

    #[token("]")]
    CloseBracket,

    #[regex(r"[ \t\r\n\f]+", logos::skip)]
    Whitespace,
}

pub fn lex(source: &str) -> Result<Vec<Token>, Vec<Diagnostic>> {
    let normalized_source = normalize_heredoc_literals(source);
    let mut lexer = RawToken::lexer(&normalized_source);
    let mut tokens = Vec::new();
    let mut diagnostics = Vec::new();

    while let Some(result) = lexer.next() {
        let range = lexer.span();
        let span = Span::new(range.start, range.end);

        match result {
            Ok(raw) => tokens.push(Token {
                kind: raw.into_token_kind(),
                span,
            }),
            Err(()) => diagnostics.push(Diagnostic::new(
                format!("unexpected token `{}`", lexer.slice()),
                span,
            )),
        }
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        span: Span::new(source.len(), source.len()),
    });

    if diagnostics.is_empty() {
        Ok(tokens)
    } else {
        Err(diagnostics)
    }
}

impl RawToken {
    fn into_token_kind(self) -> TokenKind {
        match self {
            RawToken::OpenPhp => TokenKind::OpenPhp,
            RawToken::Echo => TokenKind::Echo,
            RawToken::Ident(value) => TokenKind::Ident(value),
            RawToken::Variable(value) => TokenKind::Variable(value),
            RawToken::String(value) => TokenKind::String(value),
            RawToken::Number(value) => TokenKind::Number(value),
            RawToken::Semicolon => TokenKind::Semicolon,
            RawToken::Colon => TokenKind::Colon,
            RawToken::Comma => TokenKind::Comma,
            RawToken::OpenParen => TokenKind::OpenParen,
            RawToken::CloseParen => TokenKind::CloseParen,
            RawToken::OpenBrace => TokenKind::OpenBrace,
            RawToken::CloseBrace => TokenKind::CloseBrace,
            RawToken::Plus => TokenKind::Plus,
            RawToken::Minus => TokenKind::Minus,
            RawToken::Star => TokenKind::Star,
            RawToken::Slash => TokenKind::Slash,
            RawToken::Percent => TokenKind::Percent,
            RawToken::Comment => unreachable!("logos skipped comments"),
            RawToken::Dot => TokenKind::Dot,
            RawToken::Pipe => TokenKind::Pipe,
            RawToken::Ampersand => TokenKind::Ampersand,
            RawToken::Equals => TokenKind::Equals,
            RawToken::Backslash => TokenKind::Backslash,
            RawToken::Question => TokenKind::Question,
            RawToken::LessThan => TokenKind::LessThan,
            RawToken::GreaterThan => TokenKind::GreaterThan,
            RawToken::OpenBracket => TokenKind::OpenBracket,
            RawToken::CloseBracket => TokenKind::CloseBracket,
            RawToken::Whitespace => unreachable!("logos skipped whitespace"),
        }
    }
}

fn parse_quoted_string(slice: &str) -> String {
    let quote = slice.as_bytes()[0];
    let inner = &slice[1..slice.len() - 1];

    let mut output = String::new();
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        match chars.next() {
            Some('n') if quote == b'"' => output.push('\n'),
            Some('t') if quote == b'"' => output.push('\t'),
            Some('r') if quote == b'"' => output.push('\r'),
            Some('"') if quote == b'"' => output.push('"'),
            Some('\'') if quote == b'\'' => output.push('\''),
            Some('\\') => output.push('\\'),
            Some(other) => {
                if quote == b'\'' {
                    output.push('\\');
                }
                output.push(other);
            }
            None => output.push('\\'),
        }
    }

    output
}

fn normalize_heredoc_literals(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index + 3 <= bytes.len() {
        if &bytes[index..index + 3] != b"<<<" {
            index += 1;
            continue;
        }

        let Some((label, content_start)) = parse_heredoc_label(source, index + 3) else {
            index += 3;
            continue;
        };
        let Some(end_start) = find_heredoc_end(source, content_start, &label) else {
            index += 3;
            continue;
        };

        output[index] = b'"';
        for byte in output.iter_mut().take(content_start).skip(index + 1) {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
        for byte in output.iter_mut().take(end_start).skip(content_start) {
            if *byte == b'"' || *byte == b'\\' {
                *byte = b' ';
            }
        }
        output[end_start] = b'"';
        let mut end = end_start + label.len();
        if output.get(end) == Some(&b';') {
            end += 1;
        }
        for byte in output.iter_mut().take(end).skip(end_start + 1) {
            if *byte != b'\n' && *byte != b'\r' {
                *byte = b' ';
            }
        }
        index = end;
    }

    String::from_utf8(output).expect("heredoc normalization preserves UTF-8")
}

fn parse_heredoc_label(source: &str, mut index: usize) -> Option<(String, usize)> {
    let bytes = source.as_bytes();
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }

    let quote = match bytes.get(index) {
        Some(b'\'' | b'"') => {
            let quote = bytes[index];
            index += 1;
            Some(quote)
        }
        _ => None,
    };

    let label_start = index;
    while matches!(
        bytes.get(index),
        Some(b'A'..=b'Z' | b'a'..=b'z' | b'_' | b'0'..=b'9')
    ) {
        index += 1;
    }
    if index == label_start {
        return None;
    }
    let label = source[label_start..index].to_string();

    if let Some(quote) = quote {
        if bytes.get(index) != Some(&quote) {
            return None;
        }
        index += 1;
    }
    while matches!(bytes.get(index), Some(b' ' | b'\t')) {
        index += 1;
    }
    match bytes.get(index) {
        Some(b'\n') => Some((label, index + 1)),
        Some(b'\r') if bytes.get(index + 1) == Some(&b'\n') => Some((label, index + 2)),
        _ => None,
    }
}

fn find_heredoc_end(source: &str, mut index: usize, label: &str) -> Option<usize> {
    while index < source.len() {
        let line_end = source[index..]
            .find(['\r', '\n'])
            .map(|offset| index + offset)
            .unwrap_or(source.len());
        let line = &source[index..line_end];
        let terminator = line.strip_suffix(';').unwrap_or(line);
        if terminator == label {
            return Some(index);
        }
        index = line_end;
        if source.as_bytes().get(index) == Some(&b'\r') {
            index += 1;
        }
        if source.as_bytes().get(index) == Some(&b'\n') {
            index += 1;
        }
    }
    None
}
