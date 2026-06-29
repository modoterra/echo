# Single Parser Pipeline and AST to HIR Direction

Echo supports `.php`, `.echo`, and `.xo` files through one shared parser
pipeline. File extension is not parser configuration: valid PHP syntax and Echo
extensions are accepted by the same language parser.

```text
source text
  -> lexer/parser
  -> AST
  -> HIR lowering
  -> MIR
  -> LLVM IR
  -> AOT/native build or LLVM JIT
```

This pipeline is the architectural boundary for parser work: the parser records
source facts, and later compiler stages own semantic lowering and execution.

Module and namespace syntax is also a source fact at this layer. Echo
`module acme.http` and PHP-compatible `namespace Acme\Http` may preserve their
original spelling in the AST, but corresponding declarations should resolve to
the same internal identity before HIR/MIR/codegen.

Opt-in modernization policies are source declarations, not parser modes. A
future form such as `semantics { strict }` should parse as normal Echo syntax
and be enforced later by semantic analysis.

This document describes the parser foundation and the required long-term
compiler stages.

## Goals

- Parse `.php`, `.echo`, and `.xo` files without duplicating the parser.
- Preserve PHP compatibility while making Echo extensions available everywhere.
- Allow the `<?php` opening tag to be present or omitted.
- Record source-level facts in the AST when they are semantically relevant.
- Leave HIR lowering as a later pass after AST construction.

## Parser API

The parser exposes one language entrypoint:

```rust
pub fn parse(source: &str) -> Result<Program, Vec<Diagnostic>>;
```

Callers should not infer parser behavior from file extension. Ecosystem code may
still use extensions for discovery, editor activation, package layout, or
stock-PHP expectations, but not to decide which syntax is legal.

## AST Requirements

The AST represents source syntax and source-level facts. It should not lower
`echo` statements into write operations or backend concepts.

`Program` should include opening-tag information only when downstream tools need
to explain the original source spelling:

```rust
pub struct Program {
    pub opening_tag: Option<OpeningTag>,
    pub statements: Vec<Stmt>,
}

pub struct OpeningTag {
    pub span: Span,
}
```

This AST shape lets downstream tools explain whether the opening tag was present
in the original source without creating a separate PHP/Echo language mode.

`OpeningTag` can stay minimal until the parser needs to distinguish forms such
as `<?php`, `<?=`, or surrounding trivia.

## Required Behavior

- `.php`, `.echo`, `.xo`, and unknown extensions parse through the same grammar.
- `<?php` may be present or omitted.
- PHP-compatible syntax remains valid.
- Echo extensions remain available regardless of extension.
- Parser diagnostics should describe syntax problems, not file-mode policy.

## Test Coverage

Add or update tests for:

- `.php` with `<?php` succeeds;
- `.php` without `<?php` succeeds when the source is otherwise valid Echo;
- `.echo` without `<?php` succeeds;
- `.echo` with `<?php` succeeds;
- AST output includes `source_kind`;
- AST output includes `opening_tag` when present.

Existing examples should keep working:

```sh
cargo run -p xo -- ast examples/hello.echo
cargo run -p xo -- ast examples/hello.php
```

These commands compare both extension defaults through the same `xo ast` surface, which is the quickest manual check for parser-mode drift.

## Implementation Notes

Suggested slice:

1. Inspect `echo_ast` and `echo_parser` public APIs.
2. Keep one parser entrypoint for all supported file extensions.
3. Extend `Program` with optional opening tag and existing statements when
   downstream tools need that source fact.
4. Update opening-tag parsing and diagnostics.
5. Update `xo ast` and other parser call sites to use the shared parser.
7. Add focused tests.
8. Run formatting, checks, tests, and the two example `xo ast` commands.

Validation commands:

```sh
cargo fmt-check
cargo check --workspace
cargo test --workspace
cargo run -p xo -- ast examples/hello.echo
cargo run -p xo -- ast examples/hello.php
```

This validation sequence combines formatting, typechecking, tests, and both parser-mode smoke checks before the slice is considered complete.

## LLVM-First Execution Model

Parser modes and HIR are separate concerns.

AST is the syntax-level representation. HIR is the resolved representation of
Echo/PHP meaning. MIR is Echo's required backend-neutral compiler IR. LLVM IR
is the single backend target for the current architecture.

Echo's primary compiler path is intentionally linear:

```text
Source -> AST -> HIR -> MIR -> LLVM IR
```

This compact pipeline names ownership boundaries: AST remains syntax, HIR/MIR own resolved meaning, and LLVM IR is the current backend target.

From LLVM IR, Echo can support multiple execution modes without introducing a
second language engine:

```text
LLVM IR -> AOT/native build
LLVM IR -> JIT/in-process execution
```

Both execution modes consume the same generated LLVM IR, which keeps `xo run`, `xo build`, and embedding from becoming separate language implementations.

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

This example shows where semantic lowering belongs: string and variable syntax become write operations after parsing, not inside the parser.

HIR should be built by a lowering or semantic layer, not by another parser. The
HIR layer could live in:

```text
crates/echo_hir
```

The crate boundary gives HIR a home without coupling parser internals to backend-specific lowering.

With an entrypoint like:

```rust
pub fn lower_program(program: &echo_ast::Program) -> Result<HirProgram, Vec<Diagnostic>>;
```

This API shape lets callers feed parsed AST into semantic lowering and receive diagnostics without introducing another source parser.

MIR should be built from HIR as the compiler-owned representation that makes
LLVM lowering regular and testable. It is required, but it is not an execution
engine and it is not VM bytecode.

Do not add broad HIR or MIR implementation as part of the parser-pipeline slice
unless the current code already has a natural, tiny stub location.

## LLVM IR as the Execution Boundary

Echo does not plan a custom bytecode VM or interpreter as a second execution
engine. LLVM IR is the execution boundary for both native builds and in-process
LLVM JIT execution.

For example, the same Echo program could accidentally behave differently if a
custom bytecode engine and LLVM codegen both independently implemented language
behavior.

LLVM JIT gives Echo most of the near-term benefits of embedded execution while
preserving the same LLVM backend used for native builds.

This means portable bytecode, a tiny interpreter, deterministic bytecode
debugger stepping, and plugin scripts without native code execution are outside
the planned architecture unless Echo deliberately revisits the execution
boundary.

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

This list is illustrative runtime ABI vocabulary: generated LLVM should call stable runtime operations instead of reimplementing language behavior.

AOT and JIT both use the same runtime contract:

```text
AOT:
  LLVM IR -> object/binary -> linked with echo_runtime

JIT:
  LLVM IR -> in-process execution -> registered echo_runtime symbols
```

The contract keeps AOT and JIT execution honest by making both paths call the same Rust-owned runtime symbols.

This keeps executable behavior in one Rust-owned runtime layer.

## Roadmap

Current:

```text
AST -> LLVM IR -> temp native binary
```

This is the present implementation checkpoint and explains why parser-mode changes currently need to keep direct AST-to-LLVM behavior working.

Next:

```text
AST -> HIR -> MIR -> LLVM IR -> native binary
```

This next step introduces resolved compiler IR without changing the backend target.

Then:

```text
AST -> HIR -> MIR -> LLVM IR -> JIT
```

This later step adds faster embedded execution without creating a second semantic engine.

## Constraints

- Do not duplicate the parser.
- Do not build separate PHP and Echo grammars.
- Do not lower AST directly into LLVM-specific concepts.
- Do not require `<?php` for `.echo`.
- Do not allow `.php` files without `<?php`.
- Keep the initial change small and reviewable.

The central decision is:

```text
one parser configuration, not per-extension parsers
```

This is the core rule for future parser work: add supported syntax to the
shared parser rather than forking PHP and Echo grammars.

Echo should evolve as a shared PHP-compatible grammar with Echo extensions
available in every supported source file.
