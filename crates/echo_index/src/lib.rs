//! Project facts extracted from parsed modules (for resolver / tooling).

#![forbid(unsafe_code)]

use echo_ast::{BindLeader, File, ImportPathSeg, Stmt};
use echo_source::{SourceId, Span};

/// Stable crate identity for workspace linkage checks.
pub fn crate_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// One import path as written (`std/io`, `./user`, …).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportFact {
    pub segments: Vec<PathSeg>,
    pub span: Span,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PathSeg {
    Dot,
    Name(String),
}

/// Exported name (`\ name`). Kind filled when the name is defined in-module.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportFact {
    pub name: String,
    pub span: Span,
    /// Set when the export resolves to a local bind or primary struct name.
    pub kind: Option<ExportKind>,
}

/// What an export refers to (for import binding).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExportKind {
    Mutable,
    Immutable,
    Const,
    /// Primary `% struct_name` type name.
    Struct,
}

/// Member on a `%` / `@` struct.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberFact {
    pub name: String,
    pub leader: BindLeader,
    pub span: Span,
}

/// `%` or `@` declaration facts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFact {
    pub name: String,
    pub span: Span,
    pub members: Vec<MemberFact>,
    /// True for `%`, false for `@`.
    pub is_primary: bool,
}

/// Facts for one source file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleFacts {
    pub source: SourceId,
    pub imports: Vec<ImportFact>,
    pub exports: Vec<ExportFact>,
    pub structs: Vec<StructFact>,
    /// Top-level bind names (`$` / `~` / `#`).
    pub top_binds: Vec<(String, BindLeader, Span)>,
}

/// Extract indexable facts from a parsed file.
#[must_use]
pub fn extract(file: &File) -> ModuleFacts {
    let mut imports = Vec::new();
    let mut exports = Vec::new();
    let mut structs = Vec::new();
    let mut top_binds = Vec::new();

    for stmt in &file.stmts {
        match stmt {
            Stmt::Import(i) => {
                let segments = i
                    .path
                    .iter()
                    .map(|s| match s {
                        ImportPathSeg::Dot => PathSeg::Dot,
                        ImportPathSeg::Name(n) => PathSeg::Name(n.name.clone()),
                    })
                    .collect();
                imports.push(ImportFact {
                    segments,
                    span: i.span,
                });
            }
            Stmt::Export(e) => {
                for n in &e.names {
                    exports.push(ExportFact {
                        name: n.name.clone(),
                        span: n.span,
                        kind: None,
                    });
                }
            }
            Stmt::Struct(s) => {
                structs.push(StructFact {
                    name: s.name.name.clone(),
                    span: s.span,
                    members: member_facts(&s.members),
                    is_primary: true,
                });
            }
            Stmt::StructExt(s) => {
                structs.push(StructFact {
                    name: s.name.name.clone(),
                    span: s.span,
                    members: member_facts(&s.members),
                    is_primary: false,
                });
            }
            Stmt::Bind(b) => {
                top_binds.push((b.name.name.clone(), b.leader, b.name.span));
            }
            _ => {}
        }
    }

    // Resolve export kinds against local definitions.
    for exp in &mut exports {
        exp.kind = resolve_export_kind(&exp.name, &top_binds, &structs);
    }

    ModuleFacts {
        source: file.source,
        imports,
        exports,
        structs,
        top_binds,
    }
}

fn resolve_export_kind(
    name: &str,
    top_binds: &[(String, BindLeader, Span)],
    structs: &[StructFact],
) -> Option<ExportKind> {
    if let Some((_, leader, _)) = top_binds.iter().find(|(n, _, _)| n == name) {
        return Some(match leader {
            BindLeader::Tilde => ExportKind::Mutable,
            BindLeader::Dollar => ExportKind::Immutable,
            BindLeader::Hash => ExportKind::Const,
        });
    }
    if structs.iter().any(|s| s.is_primary && s.name == name) {
        return Some(ExportKind::Struct);
    }
    None
}

fn member_facts(members: &[Stmt]) -> Vec<MemberFact> {
    let mut out = Vec::new();
    for m in members {
        if let Stmt::Bind(b) = m {
            out.push(MemberFact {
                name: b.name.name.clone(),
                leader: b.leader,
                span: b.name.span,
            });
        }
    }
    out
}

/// Display import path for diagnostics (`./user`, `std/io`).
#[must_use]
pub fn format_import_path(segments: &[PathSeg]) -> String {
    let mut parts = Vec::new();
    for (i, seg) in segments.iter().enumerate() {
        match seg {
            PathSeg::Dot => {
                if i == 0 {
                    parts.push(".".to_string());
                } else {
                    parts.push(".".to_string());
                }
            }
            PathSeg::Name(n) => parts.push(n.clone()),
        }
    }
    // Join: ./config → . / config → special-case leading Dot
    if matches!(segments.first(), Some(PathSeg::Dot)) {
        let rest: Vec<_> = segments[1..]
            .iter()
            .filter_map(|s| match s {
                PathSeg::Name(n) => Some(n.as_str()),
                PathSeg::Dot => Some("."),
            })
            .collect();
        if rest.is_empty() {
            ".".into()
        } else {
            format!("./{}", rest.join("/"))
        }
    } else {
        segments
            .iter()
            .filter_map(|s| match s {
                PathSeg::Name(n) => Some(n.as_str()),
                PathSeg::Dot => Some("."),
            })
            .collect::<Vec<_>>()
            .join("/")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use echo_parser::parse;
    use echo_source::SourceMap;

    #[test]
    fn extracts_import_export_struct() {
        let src = "\
/ ./user
% user {
    $ name
}
\\ user
";
        let mut map = SourceMap::new();
        let id = map.add("t.echo", src);
        let p = parse(map.get(id).unwrap());
        let facts = extract(p.file.as_ref().unwrap());
        assert_eq!(facts.imports.len(), 1);
        assert_eq!(format_import_path(&facts.imports[0].segments), "./user");
        assert_eq!(facts.exports.len(), 1);
        assert_eq!(facts.structs.len(), 1);
        assert!(facts.structs[0].is_primary);
        assert_eq!(facts.structs[0].members[0].name, "name");
    }
}
