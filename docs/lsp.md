# Echo LSP

`echo_lsp` exposes Echo compiler and frontend capabilities through the Language
Server Protocol. The server should be thin: it tracks open documents, calls the
shared parser, semantic, and index APIs, and converts Echo ranges and
diagnostics to LSP protocol types.

The LSP must not implement language syntax or semantics locally.

```text
echo_lexer
  -> echo_parser
  -> echo_semantics
  -> echo_index
  -> echo_lsp
  -> editor
```

`xo` should expose the server with:

```sh
RUST_LOG=info xo lsp 2> /tmp/echo-lsp.log
```

This launch form keeps stdout clean for LSP JSON-RPC traffic while preserving server diagnostics in a log file.

The command starts an LSP server over stdio. Stdout is reserved for JSON-RPC
protocol traffic; logs must go to stderr or through LSP log messages.

## First Slice

The first implementation slice should provide:

- `echo_index` declaration indexing.
- `echo_lsp` server startup over stdio.
- `xo lsp`.
- LSP `initialize`, `initialized`, `shutdown`, and `exit`.
- Full document sync for `didOpen`, `didChange`, and `didClose`.
- Syntax diagnostics from the shared parser/frontend.
- Diagnostic clearing on document close.

Do not advertise formatting or code actions until each capability is
implemented. Document symbols, workspace symbols, hover, go-to definition,
references, completion, signature help, rename, and full-document semantic
tokens are implemented from shared compiler/frontend facts.

## Crate Shape

Add:

```text
crates/echo_lsp/
```

Suggested modules:

```text
crates/echo_lsp/src/
  lib.rs
  server.rs
  document.rs
  diagnostics.rs
  position.rs
```

Potential later modules:

```text
symbols.rs
hover.rs
completion.rs
definition.rs
semantic_tokens.rs
config.rs
```

Suggested dependencies:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tower-lsp-server = "0.21"
lsp-types = "0.97"
dashmap = "6"
ropey = "1"
url = "2"

echo_ast = { path = "../echo_ast" }
echo_parser = { path = "../echo_parser" }
echo_index = { path = "../echo_index" }
```

Tokio belongs at the LSP boundary. Do not introduce async runtime requirements
into `echo_parser`, `echo_ast`, `echo_semantics`, or `echo_codegen`.

If parsing becomes expensive later, the LSP can move parsing onto a blocking
task without changing parser APIs:

```rust
tokio::task::spawn_blocking(move || {
    echo_parser::parse_source(&source, mode)
})
.await
```

Use this only at the LSP boundary if parsing needs to move off the async worker. Parser APIs should stay synchronous and reusable by the CLI, tests, and future tools.

## File Modes

The LSP must select source mode from the URI and pass it into the shared parser
or frontend. It must not duplicate parser-mode rules.

```rust
pub fn mode_from_uri(uri: &Url) -> EchoFileMode {
    match uri.path().rsplit('.').next() {
        Some("php") => EchoFileMode::PhpCompat,
        Some("echo") => EchoFileMode::Echo,
        _ => EchoFileMode::Echo,
    }
}
```

Mode behavior:

- `.echo`: native Echo mode; opening `<?php` may be omitted.
- `.php`: PHP compatibility mode; opening `<?php` is required.
- unknown extension: default to Echo mode for now.

This should stay aligned with [Parser Modes](parser-modes.md).

## Document Store

Use full document sync in the first slice:

```rust
use ropey::Rope;
use lsp_types::Url;

pub struct Document {
    pub uri: Url,
    pub version: i32,
    pub mode: EchoFileMode,
    pub text: Rope,
    pub file_id: echo_index::FileId,
}
```

Server state can stay simple:

```rust
pub struct Backend {
    client: Client,
    documents: DashMap<Url, Document>,
    index: Mutex<EchoIndex>,
}
```

Either `tokio::sync::Mutex` or `std::sync::Mutex` is acceptable for the first
slice.

## Position Mapping

LSP positions are UTF-16 line/character pairs. Echo parser spans are byte
offsets. Add `position.rs` helpers:

```rust
pub fn offset_to_position(text: &Rope, offset: usize) -> lsp_types::Position;
pub fn range_to_lsp_range(text: &Rope, range: TextRange) -> lsp_types::Range;
pub fn position_to_offset(text: &Rope, position: lsp_types::Position) -> Option<usize>;
```

Prefer correct UTF-16 behavior. If the first implementation is ASCII-only for
the current examples, leave explicit TODOs and tests that make the limitation
visible.

## Diagnostics Flow

On `didOpen` and full `didChange`:

1. Update the document text.
2. Determine `EchoFileMode` from the URI.
3. Call the shared parser or frontend.
4. Convert Echo diagnostics and ranges to LSP diagnostics and ranges.
5. Publish diagnostics.
6. Update `echo_index` with declaration facts when those facts are available.

If the parser currently returns `Result<Program, Error>`, adapt temporarily in
`echo_lsp`:

- `Ok(program)`: publish an empty diagnostic list unless parser warnings are
  available.
- `Err(error)`: convert the parser error into one LSP diagnostic.

Over time, prefer a recoverable parser result:

```rust
pub struct ParseResult {
    pub program: Option<Program>,
    pub diagnostics: Vec<EchoDiagnostic>,
}
```

Shared diagnostic shape should eventually live outside `echo_lsp`, likely in
`echo_diagnostics`:

```rust
pub struct EchoDiagnostic {
    pub message: String,
    pub span: TextRange,
    pub severity: EchoSeverity,
    pub code: Option<String>,
}

pub enum EchoSeverity {
    Error,
    Warning,
    Information,
    Hint,
}
```

LSP severity mapping:

```text
Error       -> DiagnosticSeverity::ERROR
Warning     -> DiagnosticSeverity::WARNING
Information -> DiagnosticSeverity::INFORMATION
Hint        -> DiagnosticSeverity::HINT
```

This mapping keeps internal diagnostic vocabulary stable while translating to the severity values expected by editors.

## Capabilities

The M1 server should advertise only full document sync:

```rust
ServerCapabilities {
    text_document_sync: Some(TextDocumentSyncCapability::Kind(
        TextDocumentSyncKind::FULL,
    )),
    ..Default::default()
}
```

Document symbols and workspace symbols are served from `echo_index` declaration
facts. Hover is served from declaration and dependency facts, including PHP
`use`, `require`, and `require_once` facts extracted by `echo_semantics`.
Go-to definition resolves declaration/dependency self-navigation and PHP
imported static class references such as `Request::capture()` back to the
matching `use Illuminate\Http\Request;` dependency fact.
For open filesystem-backed PHP files, the server also follows concrete
`require`, `require_once`, `include`, `include_once`, and Composer autoload file
dependencies extracted by `echo_semantics`, parses those imported PHP source
units into `echo_index`, and can resolve PHPDoc-typed receiver method calls such
as `$app->handleRequest(...)` to indexed class methods in included vendor files.
Go-to definition on PHP `use` imports consults Composer `autoload_classmap.php`
and `autoload_psr4.php` beside the indexed `vendor/autoload.php`, so imports
such as `use Illuminate\Http\Request;` can open their concrete vendor class file
even when that vendor file is not yet fully parseable by Echo.
References are initially same-file and import-aware, so finding references for
that imported `Request` name returns both the `use` dependency and static class
reference.
Semantic tokens are full-document only for now and are converted from shared
`echo_lexer` tokens, with lightweight LSP-side classification for PHP keywords,
variables, strings, numbers, functions, methods, classes, namespaces, and
operators.
PHPDoc `@var Type $name` annotations are indexed by `echo_semantics` for local
variable hover/type facts, which covers Laravel bootstrap annotations such as
`/** @var Application $app */`.
Completion currently includes reflected PHP builtins, PHP `use` imports, indexed
local variables, and the Laravel bootstrap `Application` method surface needed
for `$app->handleRequest(...)`.
Signature help currently includes reflected PHP builtin functions and the
Laravel bootstrap call shapes for `Request::capture()` and
`Application::handleRequest(Request $request)`.
Rename is same-file and covers PHP variables, PHPDoc local-variable annotations,
and imported class references such as `Request` in the Laravel bootstrap
fixture.

## CLI Wiring

Add an `lsp` subcommand to `xo`:

```sh
cargo run -p xo -- lsp 2> /tmp/echo-lsp.log
```

This command exercises the development binary without installing `xo`, while still separating protocol output from diagnostic logs.

If `xo` uses an async main:

```rust
Command::Lsp => {
    echo_lsp::run_stdio().await?;
}
```

If `xo` remains synchronous:

```rust
Command::Lsp => {
    tokio::runtime::Runtime::new()?
        .block_on(echo_lsp::run_stdio())?;
}
```

Avoid forcing async into non-LSP compiler code.

## Editor Launch

The server launch command is:

```sh
xo lsp 2> ~/.cache/echo-lsp.log
```

This is the shape an editor integration should run: `xo` owns the stdio protocol and stderr is available for troubleshooting outside the editor UI.

Editor-specific setup should point the editor's Echo or PHP language server
command to that executable and argument. VS Code and Zed extension packaging can
be documented once the integration shape is real; the first slice only needs a
working stdio server.

## Manual Smoke Test

Use an LSP-capable editor or a small JSON-RPC client:

```text
initialize
didOpen valid .echo file
expect empty diagnostics or parser diagnostics

didChange invalid syntax
expect diagnostics

didChange valid syntax
expect diagnostics cleared

didClose
expect diagnostics cleared

shutdown
exit
```

Manual editor acceptance:

- Open `examples/hello.echo`.
- Introduce a syntax error.
- Confirm diagnostics appear.
- Fix the syntax error.
- Confirm diagnostics disappear.

## Validation

First slice commands:

```sh
cargo check --workspace
cargo test -p echo_index
cargo test -p echo_lsp
cargo run -p xo -- lsp
```

This sequence verifies the workspace, the shared index crate, the LSP crate, and finally the CLI entrypoint that editors will invoke.

Behavioral acceptance:

- `echo_lsp` starts over stdio through `xo lsp`.
- `initialize` returns valid capabilities.
- `shutdown` exits cleanly.
- `didOpen` parses and publishes diagnostics.
- `didChange` reparses and republishes diagnostics.
- `didClose` clears diagnostics for that URI.
- The server does not print non-protocol logs to stdout.
- `echo_lsp` depends on `echo_index`.
- `echo_index` does not depend on `echo_lsp` or `lsp_types`.
- Parser and compiler APIs remain synchronous.

## Capability Roadmap

Suggested order after M1:

```text
M2: syntax diagnostics
M3: document symbols
M4: semantic tokens
M5: hover
M6: workspace symbol index
M7: go-to definition
M8: references
M9: completion
M10: signature help
M11: rename
M12: formatting
M13: code actions
M14: inlay hints
M15: call hierarchy/type hierarchy
M16: workspace diagnostics and file rename handling
```

The server should advertise each capability only when it is implemented and
tested.

## Constraints

Do:

- Keep `echo_lsp` thin.
- Keep `echo_index` independent from LSP.
- Use shared parser/frontend APIs.
- Use full document sync first.
- Make diagnostics work before richer IDE features.
- Use stable IDs and facts instead of AST node references.
- Add tests for index update and removal behavior.

Do not:

- Make `echo_lsp` its own parser.
- Make `echo_lsp` own language semantics.
- Expose `lsp_types` from `echo_index`.
- Introduce Tokio into parser or compiler internals.
- Advertise LSP capabilities before they work.
- Implement autocomplete before diagnostics and document sync are stable.
- Hardcode the full PHP standard library into Rust maps.
- Store raw AST nodes in `echo_index` as the long-term model.
