use echo_lexer::{Token, TokenKind};
use ropey::Rope;
use tower_lsp_server::ls_types::{
    SemanticToken, SemanticTokenType, SemanticTokens, SemanticTokensFullOptions,
    SemanticTokensLegend, SemanticTokensOptions, WorkDoneProgressOptions,
};

use crate::position::offset_to_position;

const TOKEN_TYPE_KEYWORD: u32 = 0;
const TOKEN_TYPE_VARIABLE: u32 = 1;
const TOKEN_TYPE_STRING: u32 = 2;
const TOKEN_TYPE_NUMBER: u32 = 3;
const TOKEN_TYPE_FUNCTION: u32 = 4;
const TOKEN_TYPE_METHOD: u32 = 5;
const TOKEN_TYPE_CLASS: u32 = 6;
const TOKEN_TYPE_NAMESPACE: u32 = 7;
const TOKEN_TYPE_OPERATOR: u32 = 8;

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::KEYWORD,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::METHOD,
            SemanticTokenType::CLASS,
            SemanticTokenType::NAMESPACE,
            SemanticTokenType::OPERATOR,
        ],
        token_modifiers: Vec::new(),
    }
}

pub fn semantic_tokens_options() -> SemanticTokensOptions {
    SemanticTokensOptions {
        work_done_progress_options: WorkDoneProgressOptions {
            work_done_progress: None,
        },
        legend: semantic_tokens_legend(),
        range: None,
        full: Some(SemanticTokensFullOptions::Bool(true)),
    }
}

pub fn semantic_tokens_for_source(source: &str) -> SemanticTokens {
    let rope = Rope::from_str(source);
    let tokens = echo_lexer::lex(source).unwrap_or_default();

    SemanticTokens {
        result_id: None,
        data: encode_semantic_tokens(&rope, &classify_tokens(&tokens)),
    }
}

fn classify_tokens(tokens: &[Token]) -> Vec<SemanticTokenSource> {
    tokens
        .iter()
        .enumerate()
        .filter_map(|(index, token)| {
            let token_type = match &token.kind {
                TokenKind::OpenPhp | TokenKind::Echo => TOKEN_TYPE_KEYWORD,
                TokenKind::Variable(_) => TOKEN_TYPE_VARIABLE,
                TokenKind::String(_) => TOKEN_TYPE_STRING,
                TokenKind::Number(_) => TOKEN_TYPE_NUMBER,
                TokenKind::Ident(value) if is_keyword(value) => TOKEN_TYPE_KEYWORD,
                TokenKind::Ident(_) if previous_is_object_operator(tokens, index) => {
                    TOKEN_TYPE_METHOD
                }
                TokenKind::Ident(_) if next_is_static_operator(tokens, index) => TOKEN_TYPE_CLASS,
                TokenKind::Ident(_) if next_is_open_paren(tokens, index) => TOKEN_TYPE_FUNCTION,
                TokenKind::Ident(value) if starts_uppercase(value) => TOKEN_TYPE_CLASS,
                TokenKind::Ident(_) if in_qualified_name(tokens, index) => TOKEN_TYPE_NAMESPACE,
                TokenKind::Ident(_) => TOKEN_TYPE_NAMESPACE,
                TokenKind::Dot
                | TokenKind::Plus
                | TokenKind::Minus
                | TokenKind::Star
                | TokenKind::Slash
                | TokenKind::Percent
                | TokenKind::Pipe
                | TokenKind::Ampersand
                | TokenKind::Equals
                | TokenKind::LessThan
                | TokenKind::GreaterThan => TOKEN_TYPE_OPERATOR,
                TokenKind::Eof
                | TokenKind::Semicolon
                | TokenKind::Colon
                | TokenKind::Comma
                | TokenKind::OpenParen
                | TokenKind::CloseParen
                | TokenKind::OpenBrace
                | TokenKind::CloseBrace
                | TokenKind::Backslash
                | TokenKind::Question
                | TokenKind::OpenBracket
                | TokenKind::CloseBracket => return None,
            };

            Some(SemanticTokenSource {
                start: token.span.start,
                end: token.span.end,
                token_type,
            })
        })
        .collect()
}

fn encode_semantic_tokens(text: &Rope, sources: &[SemanticTokenSource]) -> Vec<SemanticToken> {
    let mut encoded = Vec::new();
    let mut last_line = 0;
    let mut last_start = 0;

    for source in sources {
        if source.start >= source.end {
            continue;
        }

        let start = offset_to_position(text, source.start);
        let end = offset_to_position(text, source.end);
        if start.line != end.line {
            continue;
        }

        let delta_line = start.line - last_line;
        let delta_start = if delta_line == 0 {
            start.character - last_start
        } else {
            start.character
        };

        encoded.push(SemanticToken {
            delta_line,
            delta_start,
            length: end.character - start.character,
            token_type: source.token_type,
            token_modifiers_bitset: 0,
        });

        last_line = start.line;
        last_start = start.character;
    }

    encoded
}

fn is_keyword(value: &str) -> bool {
    matches!(
        value,
        "use"
            | "if"
            | "require"
            | "require_once"
            | "true"
            | "false"
            | "null"
            | "define"
            | "__DIR__"
    )
}

fn previous_is_object_operator(tokens: &[Token], index: usize) -> bool {
    index >= 2
        && matches!(tokens[index - 2].kind, TokenKind::Minus)
        && matches!(tokens[index - 1].kind, TokenKind::GreaterThan)
}

fn next_is_static_operator(tokens: &[Token], index: usize) -> bool {
    index + 2 < tokens.len()
        && matches!(tokens[index + 1].kind, TokenKind::Colon)
        && matches!(tokens[index + 2].kind, TokenKind::Colon)
}

fn next_is_open_paren(tokens: &[Token], index: usize) -> bool {
    index + 1 < tokens.len() && matches!(tokens[index + 1].kind, TokenKind::OpenParen)
}

fn in_qualified_name(tokens: &[Token], index: usize) -> bool {
    (index > 0 && matches!(tokens[index - 1].kind, TokenKind::Backslash))
        || (index + 1 < tokens.len() && matches!(tokens[index + 1].kind, TokenKind::Backslash))
}

fn starts_uppercase(value: &str) -> bool {
    value
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_uppercase())
}

#[derive(Debug, Clone, Copy)]
struct SemanticTokenSource {
    start: usize,
    end: usize,
    token_type: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provides_full_semantic_tokens_for_laravel_entrypoint_features() {
        let tokens = semantic_tokens_for_source(
            r#"<?php
use Illuminate\Http\Request;
$app->handleRequest(Request::capture());
"#,
        );

        assert!(
            tokens
                .data
                .iter()
                .any(|token| token.token_type == TOKEN_TYPE_KEYWORD)
        );
        assert!(
            tokens
                .data
                .iter()
                .any(|token| token.token_type == TOKEN_TYPE_VARIABLE)
        );
        assert!(
            tokens
                .data
                .iter()
                .any(|token| token.token_type == TOKEN_TYPE_METHOD)
        );
        assert!(
            tokens
                .data
                .iter()
                .any(|token| token.token_type == TOKEN_TYPE_CLASS)
        );
        assert!(
            tokens
                .data
                .iter()
                .any(|token| token.token_type == TOKEN_TYPE_FUNCTION)
        );
    }

    #[test]
    fn encodes_tokens_with_lsp_deltas() {
        let sources = vec![
            SemanticTokenSource {
                start: 0,
                end: 5,
                token_type: TOKEN_TYPE_KEYWORD,
            },
            SemanticTokenSource {
                start: 6,
                end: 10,
                token_type: TOKEN_TYPE_KEYWORD,
            },
        ];

        let tokens = encode_semantic_tokens(&Rope::from_str("<?php\nuse X;\n"), &sources);

        assert_eq!(tokens.len(), 2);
        assert_eq!(tokens[0].delta_line, 0);
        assert_eq!(tokens[0].delta_start, 0);
        assert_eq!(tokens[1].delta_line, 1);
        assert_eq!(tokens[1].delta_start, 0);
    }
}
