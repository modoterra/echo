use echo_index::TextRange;
use ropey::Rope;
use tower_lsp_server::ls_types::{Position, Range};

pub fn offset_to_position(text: &Rope, offset: usize) -> Position {
    let offset = offset.min(text.len_bytes());
    let char_idx = text.byte_to_char(offset);
    let line_idx = text.char_to_line(char_idx);
    let line_start_char = text.line_to_char(line_idx);
    let character = text
        .slice(line_start_char..char_idx)
        .chars()
        .map(|ch| ch.len_utf16() as u32)
        .sum();

    Position {
        line: line_idx as u32,
        character,
    }
}

pub fn range_to_lsp_range(text: &Rope, range: TextRange) -> Range {
    Range {
        start: offset_to_position(text, range.start as usize),
        end: offset_to_position(text, range.end as usize),
    }
}

pub fn position_to_offset(text: &Rope, position: Position) -> Option<usize> {
    let line_idx = position.line as usize;
    if line_idx >= text.len_lines() {
        return None;
    }

    let line = text.line(line_idx);
    let mut utf16_units = 0u32;
    let mut line_char_offset = 0usize;

    for ch in line.chars() {
        if ch == '\n' || ch == '\r' {
            break;
        }

        if utf16_units == position.character {
            break;
        }

        let next_units = utf16_units + ch.len_utf16() as u32;
        if next_units > position.character {
            return None;
        }

        utf16_units = next_units;
        line_char_offset += 1;
    }

    if utf16_units != position.character {
        return None;
    }

    let char_idx = text.line_to_char(line_idx) + line_char_offset;
    Some(text.char_to_byte(char_idx))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_ascii_offsets_to_positions() {
        let text = Rope::from_str("one\ntwo\n");

        assert_eq!(
            offset_to_position(&text, 4),
            Position {
                line: 1,
                character: 0
            }
        );
        assert_eq!(position_to_offset(&text, Position::new(1, 2)), Some(6));
    }

    #[test]
    fn maps_utf16_positions() {
        let text = Rope::from_str("a😀b\n");

        assert_eq!(
            offset_to_position(&text, "a😀".len()),
            Position {
                line: 0,
                character: 3
            }
        );
        assert_eq!(
            position_to_offset(&text, Position::new(0, 3)),
            Some("a😀".len())
        );
        assert_eq!(position_to_offset(&text, Position::new(0, 2)), None);
    }
}
