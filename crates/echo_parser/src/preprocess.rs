pub(crate) fn strip_comments_preserving_spans(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut index = 0;
    let bytes = source.as_bytes();

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        byte if byte == quote => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'/') => {
                while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
                    output[index] = b' ';
                    index += 1;
                }
            }
            b'#' => {
                while index < bytes.len() && bytes[index] != b'\n' && bytes[index] != b'\r' {
                    output[index] = b' ';
                    index += 1;
                }
            }
            b'/' if bytes.get(index + 1) == Some(&b'*') => {
                // PHP C-style comments end at the first `*/`; they do not nest.
                // Source: https://www.php.net/manual/en/language.basic-syntax.comments.php
                output[index] = b' ';
                output[index + 1] = b' ';
                index += 2;

                while index < bytes.len() {
                    if bytes[index] == b'*' && bytes.get(index + 1) == Some(&b'/') {
                        output[index] = b' ';
                        output[index + 1] = b' ';
                        index += 2;
                        break;
                    }

                    if bytes[index] != b'\n' && bytes[index] != b'\r' {
                        output[index] = b' ';
                    }
                    index += 1;
                }
            }
            _ => index += 1,
        }
    }

    String::from_utf8(output).expect("comment stripping preserves UTF-8")
}

pub(crate) fn normalize_bracketed_namespaces(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut namespace_block_depths = Vec::new();
    let mut brace_depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        byte if byte == quote => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }
            }
            b'n' if is_keyword_at(bytes, index, b"namespace") => {
                let mut cursor = index + b"namespace".len();
                while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                    cursor += 1;
                }
                while bytes.get(cursor).is_some_and(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'\\')
                }) {
                    cursor += 1;
                }
                while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
                    cursor += 1;
                }
                if bytes.get(cursor) == Some(&b'{') {
                    output[cursor] = b';';
                    brace_depth += 1;
                    namespace_block_depths.push(brace_depth);
                    index = cursor + 1;
                } else {
                    index += 1;
                }
            }
            b'{' => {
                brace_depth += 1;
                index += 1;
            }
            b'}' => {
                if namespace_block_depths.last() == Some(&brace_depth) {
                    output[index] = b' ';
                    namespace_block_depths.pop();
                }
                brace_depth = brace_depth.saturating_sub(1);
                index += 1;
            }
            _ => index += 1,
        }
    }

    String::from_utf8(output).expect("namespace normalization preserves UTF-8")
}

fn is_keyword_at(source: &[u8], index: usize, keyword: &[u8]) -> bool {
    source.get(index..index + keyword.len()) == Some(keyword)
        && index
            .checked_sub(1)
            .and_then(|before| source.get(before))
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
        && source
            .get(index + keyword.len())
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
}

pub(crate) fn virtualize_statement_terminators(source: &str) -> String {
    let mut output = source.as_bytes().to_vec();
    let mut line_start = 0;
    let mut index = 0;
    let bytes = source.as_bytes();
    let mut paren_depth = 0usize;
    let mut bracket_depth = 0usize;

    while index < bytes.len() {
        match bytes[index] {
            b'"' | b'\'' => {
                let quote = bytes[index];
                index += 1;
                while index < bytes.len() {
                    match bytes[index] {
                        b'\\' => index = (index + 2).min(bytes.len()),
                        byte if byte == quote => {
                            index += 1;
                            break;
                        }
                        _ => index += 1,
                    }
                }
            }
            b'}' => {
                virtualize_before_close_brace(&mut output, bytes, index);
                index += 1;
            }
            b'(' => {
                paren_depth += 1;
                index += 1;
            }
            b')' => {
                paren_depth = paren_depth.saturating_sub(1);
                index += 1;
            }
            b'[' => {
                bracket_depth += 1;
                index += 1;
            }
            b']' => {
                bracket_depth = bracket_depth.saturating_sub(1);
                index += 1;
            }
            b'\n' => {
                let next = next_significant_byte(bytes, index + 1);
                if paren_depth == 0
                    && bracket_depth == 0
                    && !next_significant_starts_with_keyword(bytes, index + 1, b"else")
                    && !next_significant_starts_with_keyword(bytes, index + 1, b"elseif")
                    && !next_significant_starts_with_keyword(bytes, index + 1, b"catch")
                    && !next_significant_starts_with_keyword(bytes, index + 1, b"finally")
                    && should_end_statement(&source[line_start..index], next)
                {
                    output[index] = b';';
                }
                line_start = index + 1;
                index += 1;
            }
            b'\r' => {
                if bytes.get(index + 1) == Some(&b'\n') {
                    output[index] = b' ';
                    let next = next_significant_byte(bytes, index + 2);
                    if paren_depth == 0
                        && bracket_depth == 0
                        && !next_significant_starts_with_keyword(bytes, index + 2, b"else")
                        && !next_significant_starts_with_keyword(bytes, index + 2, b"elseif")
                        && !next_significant_starts_with_keyword(bytes, index + 2, b"catch")
                        && !next_significant_starts_with_keyword(bytes, index + 2, b"finally")
                        && should_end_statement(&source[line_start..index], next)
                    {
                        output[index + 1] = b';';
                    }
                    index += 2;
                } else {
                    let next = next_significant_byte(bytes, index + 1);
                    if paren_depth == 0
                        && bracket_depth == 0
                        && !next_significant_starts_with_keyword(bytes, index + 1, b"else")
                        && !next_significant_starts_with_keyword(bytes, index + 1, b"elseif")
                        && !next_significant_starts_with_keyword(bytes, index + 1, b"catch")
                        && !next_significant_starts_with_keyword(bytes, index + 1, b"finally")
                        && should_end_statement(&source[line_start..index], next)
                    {
                        output[index] = b';';
                    }
                    index += 1;
                }
                line_start = index;
            }
            _ => index += 1,
        }
    }

    String::from_utf8(output).expect("virtual semicolons preserve UTF-8")
}

fn should_end_statement(line: &str, next: Option<u8>) -> bool {
    let trimmed = line.trim();
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("<?php") {
        return false;
    }

    if matches!(
        next,
        Some(b'.' | b'+' | b'-' | b'*' | b'/' | b',' | b'{' | b')' | b']' | b'?' | b':')
    ) {
        return false;
    }

    if trimmed.starts_with("function ") {
        return false;
    }

    if trimmed.starts_with("declare") && trimmed.ends_with(')') {
        let after_declare = &trimmed["declare".len()..];
        if after_declare
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_whitespace() || ch == '(')
        {
            return false;
        }
    }

    if matches!(
        trimmed.as_bytes().last(),
        Some(b'=' | b'.' | b'+' | b'-' | b'*' | b'/' | b',' | b'(' | b'[' | b'{')
    ) {
        return false;
    }

    matches!(
        trimmed.as_bytes().last(),
        Some(b')' | b']' | b'}' | b'"')
            | Some(b'0'..=b'9')
            | Some(b'a'..=b'z')
            | Some(b'A'..=b'Z')
            | Some(b'_')
    )
}

fn next_significant_byte(source: &[u8], start: usize) -> Option<u8> {
    source[start..]
        .iter()
        .copied()
        .find(|byte| !byte.is_ascii_whitespace())
}

fn next_significant_starts_with_keyword(source: &[u8], start: usize, keyword: &[u8]) -> bool {
    let Some(offset) = source[start..]
        .iter()
        .position(|byte| !byte.is_ascii_whitespace())
    else {
        return false;
    };
    let start = start + offset;
    source[start..].starts_with(keyword)
        && source
            .get(start + keyword.len())
            .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
}

fn virtualize_before_close_brace(output: &mut [u8], source: &[u8], close_brace: usize) {
    let Some((previous_index, previous)) =
        previous_significant_byte_with_index(source, close_brace)
    else {
        return;
    };

    if !matches!(previous, b')' | b']' | b'"' | b'0'..=b'9' | b'a'..=b'z' | b'A'..=b'Z' | b'_') {
        return;
    }

    for index in previous_index + 1..close_brace {
        if !source[index].is_ascii_whitespace() {
            break;
        }
        output[index] = b';';
        return;
    }
}

fn previous_significant_byte_with_index(source: &[u8], end: usize) -> Option<(usize, u8)> {
    source[..end]
        .iter()
        .enumerate()
        .rev()
        .find_map(|(index, byte)| (!byte.is_ascii_whitespace()).then_some((index, *byte)))
}

pub(crate) fn normalize_heredoc_literals(source: &str) -> String {
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

pub(crate) fn unescape_double_quoted_string(input: String) -> String {
    let mut output = String::new();
    let mut chars = input.chars();

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

pub(crate) fn unescape_single_quoted_string(input: String) -> String {
    let mut output = String::new();
    let mut chars = input.chars();

    while let Some(c) = chars.next() {
        if c != '\\' {
            output.push(c);
            continue;
        }

        match chars.next() {
            Some('\\') => output.push('\\'),
            Some('\'') => output.push('\''),
            Some(other) => {
                output.push('\\');
                output.push(other);
            }
            None => output.push('\\'),
        }
    }

    output
}
