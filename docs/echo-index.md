# Echo Index

`echo_index` is the project-wide database for source facts. It stores files,
declarations, symbols, and later references and relations. It is deliberately
independent from the LSP protocol and should not expose `lsp_types`.

The index stores language facts, not AST nodes. `echo_semantics` should produce
facts from parsed and analyzed source. Tools such as `echo_lsp`, `xo repl`, and
future refactoring commands should query those facts rather than reimplementing
language semantics.

```text
echo_parser
  -> AST
  -> echo_semantics
  -> IndexFacts
  -> echo_index
  -> echo_lsp / xo / future tools
```

## Goals

- Track files and source modes.
- Store declarations with stable IDs.
- Answer document-symbol and workspace-symbol queries.
- Support incremental updates without leaving stale symbols.
- Stay independent from editor protocols and compiler backend details.

First slice scope is declaration indexing only. References, relations,
workspace dependency invalidation are future work.

## Ownership Boundary

`echo_index` should consume facts, not AST:

```rust
pub struct IndexFacts {
    pub file_id: FileId,
    pub mode: EchoFileMode,
    pub declarations: Vec<SymbolFact>,
    pub dependencies: Vec<DependencyFact>,
    pub references: Vec<ReferenceFact>,
}
```

This boundary keeps ownership clear:

- `echo_parser` owns syntax.
- `echo_semantics` owns meaning: scopes, declarations, local references,
  semantic diagnostics, and fact production.
- `echo_index` owns storage and queries over project-wide facts.
- `echo_lsp` owns protocol conversion and editor communication.

## Crate Shape

Add:

```text
crates/echo_index/
```

Suggested initial modules:

```text
crates/echo_index/src/
  lib.rs
  file.rs
  index.rs
  name.rs
  symbol.rs
  query.rs
```

Optional dependencies:

```toml
[dependencies]
smol_str = "0.3"
rustc-hash = "2"
```

Standard `HashMap` is acceptable for the first slice.

## Core Types

Use stable IDs for files and symbols:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FileId(pub u32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SymbolId(pub u64);
```

Reuse an existing shared span or text range type if one is already available
without creating dependency cycles. Otherwise add minimal offset-based types:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextRange {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextOffset(pub u32);
```

File mode should match parser/frontend source-mode behavior:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EchoFileMode {
    Echo,
    PhpCompat,
}
```

`echo_index` may store URIs as `String` to avoid depending on LSP types:

```rust
pub struct IndexedFile {
    pub file_id: FileId,
    pub uri: String,
    pub path: Option<std::path::PathBuf>,
    pub version: Option<i32>,
    pub mode: EchoFileMode,
    pub content_hash: Option<u64>,
}
```

## Names

Names should preserve PHP ambiguity. A fully qualified name is not necessarily
unique, because PHP compatibility allows conditional or otherwise ambiguous
definitions.

```rust
use smol_str::SmolStr;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SymbolName {
    pub text: SmolStr,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FqName {
    pub namespace: Vec<SmolStr>,
    pub name: SmolStr,
}
```

Lookup maps that use fully qualified names should prefer:

```rust
HashMap<FqName, Vec<SymbolId>>
```

not:

```rust
HashMap<FqName, SymbolId>
```

## Symbols

Initial symbol kinds:

```rust
pub enum SymbolKind {
    Function,
    Method,
    Class,
    Interface,
    Trait,
    Enum,
    Constant,
    Property,
    Parameter,
    LocalVariable,
    Namespace,
    TypeAlias,
    ErrorType,
    Extension,
}
```

`ErrorType` is included for Echo's first-class error values. `Extension` is
included for future extension blocks.

```rust
pub enum Visibility {
    Public,
    Protected,
    Private,
}

pub struct Signature {
    pub text: String,
}

pub struct Symbol {
    pub id: SymbolId,
    pub file_id: FileId,
    pub name: SymbolName,
    pub fq_name: Option<FqName>,
    pub kind: SymbolKind,
    pub range: TextRange,
    pub selection_range: TextRange,
    pub visibility: Option<Visibility>,
    pub container: Option<SymbolId>,
    pub signature: Option<Signature>,
}
```

The fact boundary should use the same symbol metadata without assigning IDs:

```rust
pub struct SymbolFact {
    pub name: SymbolName,
    pub fq_name: Option<FqName>,
    pub kind: SymbolKind,
    pub range: TextRange,
    pub selection_range: TextRange,
    pub visibility: Option<Visibility>,
    pub signature: Option<Signature>,
}
```

## Dependency Facts

Imports and include-like constructs should be indexed as dependencies before
they are resolved. That lets `echo_index` answer future reindexing and lookup
questions without owning PHP autoload, Composer, filesystem, or Echo module
semantics.

Initial dependency kinds should cover:

```rust
pub enum DependencyKind {
    PhpUse,
    EchoStdImport,
    EchoFileImport,
    Require,
    RequireOnce,
    Include,
    IncludeOnce,
    ComposerAutoload,
}
```

Meaning:

- `PhpUse`: PHP namespace import syntax, resolved through PHP-compatible
  namespace and autoload rules.
- `EchoStdImport`: `from std use ...`, resolved only against trusted Echo
  standard-library facts.
- `EchoFileImport`: `from "./file.echo" use ...`, resolved through Echo file
  module loading.
- `Require` / `RequireOnce` / `Include` / `IncludeOnce`: PHP include graph
  edges. These should be facts first; execution and conditional include
  semantics belong in semantic/runtime layers.
- `ComposerAutoload`: the concrete `vendor/autoload.php` source edge. Composer
  metadata and classmap resolution can layer on top later.

The index stores these facts without executing include files or inferring
conditional availability. Consumers such as `echo_lsp` may use concrete
filesystem paths from these facts to parse imported PHP source units into the
same index, so features such as definition lookup can cross from an entrypoint
through `require_once` into vendored declarations.

## Initial Index API

The first implementation should support declaration storage and basic symbol
queries:

```rust
pub struct EchoIndex {
    next_file_id: u32,
    next_symbol_id: u64,
    files: HashMap<FileId, IndexedFile>,
    symbols: HashMap<SymbolId, Symbol>,
    symbols_by_file: HashMap<FileId, Vec<SymbolId>>,
    symbols_by_name: HashMap<String, Vec<SymbolId>>,
    symbols_by_fq_name: HashMap<FqName, Vec<SymbolId>>,
}

impl EchoIndex {
    pub fn new() -> Self;
    pub fn alloc_file_id(&mut self) -> FileId;
    pub fn insert_file(&mut self, file: IndexedFile);
    pub fn remove_file(&mut self, file_id: FileId);
    pub fn update_file(&mut self, file_id: FileId, facts: IndexFacts);
    pub fn document_symbols(&self, file_id: FileId) -> Vec<&Symbol>;
    pub fn workspace_symbols(&self, query: &str, limit: usize) -> Vec<&Symbol>;
    pub fn symbol(&self, symbol_id: SymbolId) -> Option<&Symbol>;
    pub fn symbols_by_fq_name(&self, fq_name: &FqName) -> Vec<&Symbol>;
}
```

`update_file` must remove old symbols for the file before inserting new facts.
`remove_file` must remove file metadata, symbols declared in that file, and
dead symbol IDs from lookup maps.

Fully qualified lookup intentionally returns multiple symbols. PHP-compatible
autoload, conditional declarations, and include order can make a name ambiguous,
so the index should preserve candidates and leave resolution policy to
`echo_semantics` or a future project resolver.

## First-Slice Tests

Run:

```sh
cargo test -p echo_index
```

Required coverage:

- One file declaring function `foo` and class `User` returns both from
  `document_symbols`.
- Two files allow `workspace_symbols("User")` to find `User`.
- Updating a file removes old symbols from that file and inserts new symbols.
- Updating one file does not remove symbols from other files.
- Removing a file removes its symbols from document and workspace queries.

## Future Work

References should be added after declarations are stable:

```rust
pub struct ReferenceId(pub u64);

pub enum ReferenceKind {
    FunctionCall,
    MethodCall,
    StaticMethodCall,
    ClassName,
    InterfaceName,
    TraitName,
    TypeName,
    ConstantAccess,
    PropertyAccess,
    VariableRead,
    VariableWrite,
    ParameterType,
    ReturnType,
    Attribute,
    ErrorType,
    PanicValue,
    RecoverBinding,
}
```

Current and future queries:

```rust
pub fn document_symbols(&self, file_id: FileId) -> Vec<&Symbol>;
pub fn workspace_symbols(&self, query: &str, limit: usize) -> Vec<&Symbol>;
pub fn dependencies(&self, query: DependencyQuery<'_>) -> Vec<&DependencyFact>;
pub fn references(&self, query: ReferenceQuery) -> Vec<&ReferenceFact>;
pub fn definition_at(&self, file_id: FileId, offset: TextOffset) -> Option<DefinitionLocation>;
```

Future cross-file reference-to-symbol queries and relations can later support
rename, implementation lookup, type hierarchy, and call hierarchy foundations:

```rust
pub struct RelationTable {
    extends: HashMap<SymbolId, SymbolId>,
    implements: HashMap<SymbolId, Vec<SymbolId>>,
    uses_traits: HashMap<SymbolId, Vec<SymbolId>>,
    contains: HashMap<SymbolId, Vec<SymbolId>>,
}
```

Stub files should eventually provide declaration-only facts for PHP built-ins
and Echo runtime APIs instead of hardcoding broad standard-library maps in Rust:

```text
stubs/
  php/
    core.echoi
    standard.echoi
    date.echoi
    json.echoi
  echo/
    core.echoi
    runtime.echoi
```
