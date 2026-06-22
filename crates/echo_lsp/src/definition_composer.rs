use std::path::{Path, PathBuf};

use echo_index::{DependencyKind, DependencyQuery, EchoIndex};

pub(crate) fn composer_class_file(index: &EchoIndex, class_name: &str) -> Option<PathBuf> {
    for dependency in index.dependencies(DependencyQuery::all()) {
        if dependency.kind != DependencyKind::ComposerAutoload {
            continue;
        }
        let autoload = std::fs::canonicalize(&dependency.target).ok()?;
        let vendor_dir = autoload.parent()?;
        let composer_dir = vendor_dir.join("composer");
        if let Some(path) = composer_classmap_file(&composer_dir, vendor_dir, class_name) {
            return Some(path);
        }
        if let Some(path) = composer_psr4_file(&composer_dir, vendor_dir, class_name) {
            return Some(path);
        }
    }
    None
}

fn composer_classmap_file(
    composer_dir: &Path,
    vendor_dir: &Path,
    class_name: &str,
) -> Option<PathBuf> {
    let base_dir = vendor_dir.parent()?;
    let source = std::fs::read_to_string(composer_dir.join("autoload_classmap.php")).ok()?;
    for line in source.lines() {
        let Some((class, path_expr)) = parse_composer_entry(line) else {
            continue;
        };
        if php_string_literal_key(class) == class_name {
            return resolve_composer_path(path_expr, vendor_dir, base_dir);
        }
    }
    None
}

fn composer_psr4_file(composer_dir: &Path, vendor_dir: &Path, class_name: &str) -> Option<PathBuf> {
    let base_dir = vendor_dir.parent()?;
    let source = std::fs::read_to_string(composer_dir.join("autoload_psr4.php")).ok()?;
    let mut best: Option<(usize, PathBuf)> = None;
    for line in source.lines() {
        let Some((prefix, path_expr)) = parse_composer_entry(line) else {
            continue;
        };
        let prefix = php_string_literal_key(prefix);
        if !class_name.starts_with(&prefix) {
            continue;
        }
        let relative_class = class_name[prefix.len()..].replace('\\', "/");
        let Some(base_path) = resolve_composer_path(path_expr, vendor_dir, base_dir) else {
            continue;
        };
        let candidate = base_path.join(format!("{relative_class}.php"));
        if candidate.exists()
            && best
                .as_ref()
                .is_none_or(|(prefix_len, _)| prefix.len() > *prefix_len)
        {
            best = Some((prefix.len(), candidate));
        }
    }
    best.map(|(_, path)| path)
}

fn php_string_literal_key(source: &str) -> String {
    source.replace("\\\\", "\\")
}

fn parse_composer_entry(line: &str) -> Option<(&str, &str)> {
    let line = line.trim();
    let key_start = line.find('\'')? + 1;
    let key_end = line[key_start..].find('\'')? + key_start;
    let key = &line[key_start..key_end];
    let (_, value) = line[key_end..].split_once("=>")?;
    Some((key, value.trim().trim_end_matches(',')))
}

fn resolve_composer_path(expr: &str, vendor_dir: &Path, base_dir: &Path) -> Option<PathBuf> {
    let expr = expr.trim();
    if let Some(rest) = expr.strip_prefix("array(") {
        let first = rest.split(',').next()?.trim();
        return resolve_composer_path(first, vendor_dir, base_dir);
    }
    let (root, suffix) = if let Some(suffix) = expr.strip_prefix("$vendorDir . ") {
        (vendor_dir, suffix)
    } else if let Some(suffix) = expr.strip_prefix("$baseDir . ") {
        (base_dir, suffix)
    } else {
        return None;
    };
    let suffix_start = suffix.find('\'')? + 1;
    let suffix_end = suffix[suffix_start..].find('\'')? + suffix_start;
    Some(root.join(suffix[suffix_start..suffix_end].trim_start_matches('/')))
}
