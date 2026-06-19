# Parser Modes and AST to HIR Direction

Echo should support PHP-compatible files and Echo source files through one
shared parser pipeline. Source mode is parser configuration, not a separate
grammar implementation.

```text
source text
  -> lexer/parser
  -> AST
  -> HIR lowering
  -> MIR
  -> LLVM IR
  -> AOT/native build or LLVM JIT
```

This document describes the parser-mode foundation and the required long-term
compiler stages. It is not a request to implement HIR, MIR, or JIT in the
parser-mode slice.

## Goals

- Parse `.php` and `.echo` files without duplicating the parser.
- Preserve PHP compatibility by requiring `<?php` in PHP mode.
- Allow Echo files to omit `<?php`.
- Record source-level facts in the AST, including source mode and opening-tag
  presence.
- Keep future Echo-only syntax behind source-mode configuration.
- Leave HIR lowering as a later pass after AST construction.

## Source Modes

The parser should be configured with a small source-mode object. Naming may
follow existing crate conventions, but the shape should be equivalent to:

```rust
pub enum SourceKind {
    Php,
    Echo,
}

pub struct ParserConfig {
    pub source_kind: SourceKind,
    pub require_opening_tag: bool,
    pub allow_echo_extensions: bool,
}
```

Recommended constructors:

```rust
impl ParserConfig {
    pub fn php() -> Self {
        Self {
            source_kind: SourceKind::Php,
            require_opening_tag: true,
            allow_echo_extensions: false,
        }
    }

    pub fn echo() -> Self {
        Self {
            source_kind: SourceKind::Echo,
            require_opening_tag: false,
            allow_echo_extensions: true,
        }
    }
}
```

`SourceKind` should live where shared syntax data can use it without creating
dependency cycles. If `Program` stores the mode, `echo_ast` is the preferred
home.

## Parser API

The parser should expose a config-aware entrypoint:

```rust
pub fn parse_source(source: &str, config: ParserConfig) -> Result<Program, Vec<Diagnostic>>;
```

Convenience wrappers may preserve simpler call sites:

```rust
pub fn parse_echo_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_source(source, ParserConfig::echo())
}

pub fn parse_php_source(source: &str) -> Result<Program, Vec<Diagnostic>> {
    parse_source(source, ParserConfig::php())
}
```

There must still be one parser implementation. PHP and Echo behavior should
branch from configuration flags at validation points and extension grammar
points, not from separate parsers.

## AST Requirements

The AST represents source syntax and source-level facts. It should not lower
`echo` statements into write operations or backend concepts.

`Program` should include source mode and opening-tag information:

```rust
pub struct Program {
    pub source_kind: SourceKind,
    pub opening_tag: Option<OpeningTag>,
    pub statements: Vec<Stmt>,
}

pub struct OpeningTag {
    pub span: Span,
}
```

`OpeningTag` can stay minimal until the parser needs to distinguish forms such
as `<?php`, `<?=`, or surrounding trivia.

## Required Behavior

PHP mode:

- requires an opening `<?php` tag;
- parses currently supported PHP-compatible syntax;
- rejects missing opening tags with a clear diagnostic;
- does not allow future Echo-only syntax by default.

Echo mode:

- allows the opening `<?php` tag to be omitted;
- records `Some(OpeningTag)` when the tag is present;
- records `None` when the tag is absent;
- parses currently supported PHP-compatible syntax;
- enables future Echo-only syntax through `allow_echo_extensions`.

The CLI should choose mode from the input file extension:

```text
*.php  -> ParserConfig::php()
*.echo -> ParserConfig::echo()
```

Unknown extensions should follow the current CLI/source-file convention, but the
choice must be explicit in code. Today `echo_source::SourceFile::new` treats
`.echo` and `.xo` as strict-mode source and other extensions as Echo superset
source, so parser-mode selection should be reviewed against that behavior before
changing defaults.

## Test Coverage

Add or update tests for:

- `.php` with `<?php` succeeds;
- `.php` without `<?php` fails with a useful parser diagnostic;
- `.echo` without `<?php` succeeds;
- `.echo` with `<?php` succeeds;
- AST output includes `source_kind`;
- AST output includes `opening_tag` when present.

Existing examples should keep working:

```sh
cargo run -p xo -- ast examples/hello.echo
cargo run -p xo -- ast examples/hello.php
```

## Implementation Notes

Suggested slice:

1. Inspect `echo_ast` and `echo_parser` public APIs.
2. Add source-mode/config types.
3. Extend `Program` with source mode, optional opening tag, and existing
   statements.
4. Add the config-aware parser entrypoint.
5. Update opening-tag parsing and diagnostics.
6. Update `xo ast` and other parser call sites to select parser mode.
7. Add focused tests.
8. Run formatting, checks, tests, and the two example `xo ast` commands.

Validation commands:

```sh
cargo fmt --all -- --check
cargo check --workspace
cargo test --workspace
cargo run -p xo -- ast examples/hello.echo
cargo run -p xo -- ast examples/hello.php
```

## LLVM-First Execution Model

Parser modes and HIR are separate concerns.

AST is the syntax-level representation. HIR is the resolved representation of
Echo/PHP meaning. MIR is Echo's required backend-neutral compiler IR. LLVM IR
is the single backend target for the current architecture.

Echo's primary compiler path is intentionally linear:

```text
Source -> AST -> HIR -> MIR -> LLVM IR
```

From LLVM IR, Echo can support multiple execution modes without introducing a
second language engine:

```text
LLVM IR -> AOT/native build
LLVM IR -> JIT/in-process execution
```

`xo build` uses the AOT path and links generated code with `echo_runtime`.

`xo run` may initially build and run a temporary native binary. Later, it should
move to LLVM JIT for faster development loops.

Embedded Echo should use LLVM JIT in the host process. The host registers
`echo_runtime` symbols so generated LLVM code can call the same runtime
functions used by AOT builds.

This avoids semantic drift because Echo has one compiler backend and one runtime
contract.

Future lowering example:

```text
AST:
  EchoStmt([
    StringLiteral("Hello"),
    Variable("$name")
  ])

HIR:
  Write(String("Hello"))
  Write(ConvertToPrintable(ReadLocal(name)))
```

HIR should be built by a lowering or semantic layer, not by another parser. The
HIR layer could live in:

```text
crates/echo_hir
```

With an entrypoint like:

```rust
pub fn lower_program(program: &echo_ast::Program) -> Result<HirProgram, Vec<Diagnostic>>;
```

MIR should be built from HIR as the compiler-owned representation that makes
LLVM lowering regular and testable. It is required, but it is not an execution
engine and it is not VM bytecode.

Do not add broad HIR or MIR implementation as part of the parser-mode slice
unless the current code already has a natural, tiny stub location.

## Why Not a Custom VM Now?

A custom Echo VM is optional future work, not part of the current compiler
architecture. A VM would create a second execution engine beside LLVM codegen,
which increases the risk of semantic drift.

For example, the same Echo program could accidentally behave differently under a
custom VM and LLVM codegen if both independently implement language behavior.

LLVM JIT gives Echo most of the near-term benefits of an embedded execution mode
while preserving the same LLVM backend used for native builds.

A custom VM may be reconsidered later if Echo needs portable bytecode, stronger
sandboxing, a tiny interpreter, deterministic debugger stepping, or plugin
scripts without native code execution. Until then, the architecture remains
LLVM-first.

## Runtime Contract

`echo_runtime` remains the shared semantic/runtime layer. LLVM-generated code
should call runtime functions such as:

```text
echo_write
echo_write_i64
echo_write_string
echo_value_add
echo_to_printable
echo_array_get
echo_array_set
echo_call_dynamic
echo_ob_start
echo_ob_flush
echo_throw
echo_shutdown
```

AOT and JIT both use the same runtime contract:

```text
AOT:
  LLVM IR -> object/binary -> linked with echo_runtime

JIT:
  LLVM IR -> in-process execution -> registered echo_runtime symbols
```

This keeps executable behavior in one Rust-owned runtime layer.

## Roadmap

Current:

```text
AST -> LLVM IR -> temp native binary
```

Next:

```text
AST -> HIR -> MIR -> LLVM IR -> native binary
```

Then:

```text
AST -> HIR -> MIR -> LLVM IR -> JIT
```

## Constraints

- Do not duplicate the parser.
- Do not build separate PHP and Echo grammars.
- Do not lower AST directly into LLVM-specific concepts.
- Do not require `<?php` for `.echo`.
- Do not allow `.php` files without `<?php`.
- Keep the initial change small and reviewable.

The central decision is:

```text
multiple parser configurations, not multiple parsers
```

Echo should evolve as a shared PHP-compatible grammar with Echo extensions
enabled only in Echo mode.
