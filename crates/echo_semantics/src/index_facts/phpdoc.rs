use echo_index::TextRange;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct PhpDocVarAnnotation {
    pub(super) ty: String,
    pub(super) variable: String,
    pub(super) range: TextRange,
    pub(super) ty_range: TextRange,
    pub(super) selection_range: TextRange,
}

pub(super) fn phpdoc_var_annotations(source: &str) -> Vec<PhpDocVarAnnotation> {
    let mut annotations = Vec::new();
    let mut search_start = 0;

    while let Some(relative_start) = source[search_start..].find("/**") {
        let comment_start = search_start + relative_start;
        let content_start = comment_start + 3;
        let Some(relative_end) = source[content_start..].find("*/") else {
            break;
        };
        let comment_end = content_start + relative_end + 2;
        let comment = &source[content_start..content_start + relative_end];

        for (line_start, line) in comment_lines(comment, content_start) {
            let trimmed = line.trim_start_matches([' ', '\t', '*']);
            let Some(var_offset) = trimmed.find("@var") else {
                continue;
            };
            let annotation_start = line_start + line.len() - trimmed.len() + var_offset;
            let after_var = &trimmed[var_offset + 4..];
            let after_var_offset = annotation_start + 4;
            let Some(annotation) = parse_phpdoc_var_annotation(after_var, after_var_offset) else {
                continue;
            };
            annotations.push(annotation);
        }

        search_start = comment_end;
    }

    annotations
}

fn comment_lines(comment: &str, base_offset: usize) -> impl Iterator<Item = (usize, &str)> {
    let mut offset = 0;
    comment.split_inclusive('\n').map(move |line| {
        let line_start = base_offset + offset;
        offset += line.len();
        (line_start, line.trim_end_matches(['\r', '\n']))
    })
}

fn parse_phpdoc_var_annotation(text: &str, base_offset: usize) -> Option<PhpDocVarAnnotation> {
    let trimmed_start = text.len() - text.trim_start().len();
    let text = text.trim_start();
    let base_offset = base_offset + trimmed_start;

    let ty_end = text.find(char::is_whitespace)?;
    let ty = text[..ty_end].trim();
    if ty.is_empty() {
        return None;
    }

    let after_ty = &text[ty_end..];
    let variable_relative = after_ty.find('$')?;
    let variable_start_in_text = ty_end + variable_relative;
    let variable_text = &text[variable_start_in_text + 1..];
    let variable_len = variable_text
        .chars()
        .take_while(|ch| ch.is_ascii_alphanumeric() || *ch == '_')
        .map(char::len_utf8)
        .sum::<usize>();
    if variable_len == 0 {
        return None;
    }

    let selection_start = base_offset + variable_start_in_text;
    let selection_end = selection_start + 1 + variable_len;

    Some(PhpDocVarAnnotation {
        ty: ty.to_string(),
        variable: variable_text[..variable_len].to_string(),
        range: TextRange::new(base_offset as u32, selection_end as u32),
        ty_range: TextRange::new(base_offset as u32, (base_offset + ty.len()) as u32),
        selection_range: TextRange::new(selection_start as u32, selection_end as u32),
    })
}
