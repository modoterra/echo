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

    #[regex(r#""([^"\\]|\\.)*""#, parse_string)]
    String(String),

    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
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
    let mut lexer = RawToken::lexer(source);
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

fn parse_string(lex: &mut logos::Lexer<RawToken>) -> String {
    let slice = lex.slice();
    let inner = &slice[1..slice.len() - 1];

    let mut output = String::new();
    let mut chars = inner.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        match chars.next() {
            Some('n') => output.push('\n'),
            Some('t') => output.push('\t'),
            Some('r') => output.push('\r'),
            Some('"') => output.push('"'),
            Some('\\') => output.push('\\'),
            Some(other) => output.push(other),
            None => output.push('\\'),
        }
    }

    output
}
