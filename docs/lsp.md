# Language server

Editor integration presentation boundary.

| | |
|--|--|
| **Status** | **Active** — full planned depth (diagnostics + navigation + assist + format + tokens) |
| **Owners** | `echo_lsp` (`xo lsp`) |
| **Related** | [`incremental.md`](incremental.md), [`pipeline.md`](pipeline.md), ADR 0001, [`diagnostics.md`](diagnostics.md), [`implementation.md`](implementation.md) §2.9 |

## Scope

LSP features consume the **shared pipeline**. The server must not reimplement
semantics, parsing, or resolution.

| Capability | Source of truth |
|------------|-----------------|
| Diagnostics | `echo_pipeline::analyze` / same codes as `xo check` |
| Hover / completion / signature help | semantic model + AST + index facts |
| Go to definition / references / rename | AST name walk + binds + imports |
| Document / workspace symbols | `echo_index::extract` |
| Formatting | `echo_parser::format_source` (same as `xo fmt`) |
| Semantic tokens | shared lexer (+ AST roles when available) |

## Facts

- **`DocumentStore`**: open / change / close; path from `file://` URIs.
- **Overlays**: dirty buffer text passed to `check_entry_with_overlays` so
  multi-file imports still resolve against disk + open buffers.
- **Diagnostics**: `analyze_path` → shared check + project `.xo` cache.
- **Positions**: UTF-16 columns ([`position.rs`](../crates/echo_lsp/src/position.rs)).
- **Protocol**: Content-Length JSON-RPC on stdio.
- **Rename**: in-document; refuses new names that would shadow existing binds
  (language no-shadowing).

## Run

```bash
cargo build -p xo
./target/debug/xo lsp
# point the editor at this binary as the language server
```

## Advertised capabilities

`initialize` reports:

| Capability | Method(s) |
|------------|-----------|
| Full document sync | `textDocument/didOpen` / `didChange` / `didClose` / `didSave` |
| Diagnostics | `textDocument/publishDiagnostics` |
| Hover | `textDocument/hover` |
| Go to definition | `textDocument/definition` |
| Find references | `textDocument/references` |
| Completion | `textDocument/completion` |
| Signature help | `textDocument/signatureHelp` |
| Rename | `textDocument/rename` |
| Document symbols | `textDocument/documentSymbol` |
| Workspace symbols | `workspace/symbol` |
| Formatting | `textDocument/formatting` |
| Semantic tokens (full) | `textDocument/semanticTokens/full` |

**Not in this milestone** (optional later): inlay hints, code actions / quick fixes,
range formatting.

## Dependency on infra

LSP must not invent a second cache. Diagnostics use the same phase fingerprints
and store as `xo check` ([`incremental.md`](incremental.md)).
