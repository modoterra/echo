use std::fs;
use std::path::{Path, PathBuf};

use echo_ast::{Program, Stmt};
use echo_diagnostics::Diagnostic;
use echo_source::{SourceFile, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CompilationGraphEntry {
    pub dispatch_path: String,
    pub path: PathBuf,
    pub span: Span,
}

pub fn compile_entries(
    source: &SourceFile,
    program: &Program,
) -> Result<Vec<CompilationGraphEntry>, Vec<Diagnostic>> {
    let mut entries = Vec::new();
    let mut diagnostics = Vec::new();
    let source_dir = source_dir(&source.path);

    for statement in &program.statements {
        let Stmt::Compile(statement) = statement else {
            continue;
        };

        for entry in &statement.entries {
            match resolve_compile_entry(&entry.value, &source_dir, &source.path, entry.span) {
                Ok(mut resolved) => entries.append(&mut resolved),
                Err(diagnostic) => diagnostics.push(diagnostic),
            }
        }
    }

    if diagnostics.is_empty() {
        entries.sort_by(|left, right| left.dispatch_path.cmp(&right.dispatch_path));
        entries.dedup_by(|left, right| left.path == right.path);
        Ok(entries)
    } else {
        Err(diagnostics)
    }
}

fn resolve_compile_entry(
    value: &str,
    source_dir: &Path,
    entry_path: &Path,
    span: Span,
) -> Result<Vec<CompilationGraphEntry>, Diagnostic> {
    if let Some(relative) = value.strip_prefix("./") {
        return resolve_path_entry(source_dir.join(relative), span);
    }

    if value.starts_with('/') {
        return resolve_path_entry(PathBuf::from(value), span);
    }

    resolve_package_entry(value, entry_path, span)
}

fn resolve_path_entry(path: PathBuf, span: Span) -> Result<Vec<CompilationGraphEntry>, Diagnostic> {
    if has_glob(&path) {
        let matches = expand_glob(&path);
        if matches.is_empty() {
            return Err(Diagnostic::new(
                format!("compile entry `{}` did not match any files", path.display()),
                span,
            ));
        }
        return Ok(matches
            .into_iter()
            .filter_map(|path| graph_entry(path, span))
            .collect());
    }

    graph_entry(path.clone(), span)
        .map(|entry| vec![entry])
        .ok_or_else(|| {
            Diagnostic::new(
                format!("failed to resolve compile entry `{}`", path.display()),
                span,
            )
        })
}

fn resolve_package_entry(
    value: &str,
    entry_path: &Path,
    span: Span,
) -> Result<Vec<CompilationGraphEntry>, Diagnostic> {
    let Some((vendor, package)) = value.split_once('/') else {
        return Err(Diagnostic::new(
            format!("compile package entry `{value}` must use `vendor/package` form"),
            span,
        ));
    };
    if vendor.is_empty() || package.is_empty() || package.contains('/') {
        return Err(Diagnostic::new(
            format!("compile package entry `{value}` must use `vendor/package` form"),
            span,
        ));
    }

    let Some(project_root) = find_project_root(entry_path) else {
        return Err(Diagnostic::new(
            format!("failed to resolve package `{value}`: no project root found"),
            span,
        ));
    };
    let package_root = project_root.join("vendor").join(vendor).join(package);
    if !package_root.is_dir() {
        return Err(Diagnostic::new(
            format!(
                "failed to resolve package `{value}` under `{}`",
                package_root.display()
            ),
            span,
        ));
    }

    let globs = echo_package_source_globs(&package_root)
        .unwrap_or_else(|| vec!["src/**/*.echo".to_string(), "src/**/*.php".to_string()]);
    let mut entries = Vec::new();
    for pattern in globs {
        let pattern_path = package_root.join(pattern);
        entries.extend(expand_glob(&pattern_path).into_iter().filter_map(|path| {
            graph_entry_with_dispatch(path, span, |canonical| {
                canonical
                    .strip_prefix(&package_root)
                    .ok()
                    .map(|relative| format!("{value}/{}", relative.display()))
                    .unwrap_or_else(|| canonical.display().to_string())
            })
        }));
    }

    if entries.is_empty() {
        return Err(Diagnostic::new(
            format!("package `{value}` did not expose any compile graph sources"),
            span,
        ));
    }

    Ok(entries)
}

fn echo_package_source_globs(package_root: &Path) -> Option<Vec<String>> {
    let metadata = fs::read_to_string(package_root.join("echo.toml")).ok()?;
    let mut globs = Vec::new();
    for line in metadata.lines() {
        let line = line.trim();
        if !line.starts_with('"') {
            continue;
        }
        let Some(end) = line[1..].find('"') else {
            continue;
        };
        globs.push(line[1..1 + end].to_string());
    }
    (!globs.is_empty()).then_some(globs)
}

fn graph_entry(path: PathBuf, span: Span) -> Option<CompilationGraphEntry> {
    graph_entry_with_dispatch(path, span, |canonical| canonical.display().to_string())
}

fn graph_entry_with_dispatch(
    path: PathBuf,
    span: Span,
    dispatch: impl FnOnce(&Path) -> String,
) -> Option<CompilationGraphEntry> {
    let canonical = fs::canonicalize(path).ok()?;
    if !canonical.is_file() || !is_source_file(&canonical) {
        return None;
    }
    let dispatch_path = dispatch(&canonical);
    Some(CompilationGraphEntry {
        dispatch_path,
        path: canonical,
        span,
    })
}

fn source_dir(path: &Path) -> PathBuf {
    path.parent()
        .map(|path| fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."))
}

fn find_project_root(entry_path: &Path) -> Option<PathBuf> {
    let mut current = entry_path.parent();
    while let Some(path) = current {
        if path.join("composer.json").is_file() || path.join("echo.toml").is_file() {
            return Some(fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf()));
        }
        current = path.parent();
    }
    None
}

fn has_glob(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|text| text.contains('*'))
    })
}

fn expand_glob(pattern: &Path) -> Vec<PathBuf> {
    let mut prefix = PathBuf::new();
    let mut parts = Vec::new();
    let mut glob_started = false;
    for component in pattern.components() {
        let text = component.as_os_str().to_string_lossy().to_string();
        if !glob_started && !text.contains('*') {
            prefix.push(component.as_os_str());
        } else {
            glob_started = true;
            parts.push(text);
        }
    }
    let mut output = Vec::new();
    collect_glob_matches(&prefix, &parts, &mut output);
    output
}

fn collect_glob_matches(current: &Path, parts: &[String], output: &mut Vec<PathBuf>) {
    if parts.is_empty() {
        if current.is_file() && is_source_file(current) {
            output.push(current.to_path_buf());
        }
        return;
    }

    let Some((part, rest)) = parts.split_first() else {
        return;
    };
    if part == "**" {
        collect_glob_matches(current, rest, output);
        let Ok(entries) = fs::read_dir(current) else {
            return;
        };
        for entry in entries.flatten().filter(|entry| entry.path().is_dir()) {
            collect_glob_matches(&entry.path(), parts, output);
        }
        return;
    }

    let Ok(entries) = fs::read_dir(current) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if glob_part_matches(part, name) {
            collect_glob_matches(&path, rest, output);
        }
    }
}

fn glob_part_matches(pattern: &str, name: &str) -> bool {
    if pattern == "*" {
        return true;
    }
    if let Some((prefix, suffix)) = pattern.split_once('*') {
        return name.starts_with(prefix) && name.ends_with(suffix);
    }
    pattern == name
}

fn is_source_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|extension| extension.to_str()),
        Some("php" | "echo" | "xo")
    )
}
