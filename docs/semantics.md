# Semantics

Local semantic and type analysis: bindings, scopes, and analysis diagnostics.

| | |
|--|--|
| **Status** | Active — file-local v0; type/literal **direction** locked below |
| **Owners** | `echo_semantics` |
| **Related** | `docs/syntax.md`, `docs/parser.md`, `docs/modules.md`, ADR 0001 |
| **CLI** | `xo check [--diag-codes] <file>` |

## Types and literals (direction)

### Inference default

- Types of **bindings, params, returns, fields** are **inferred**.
- Programmers do not annotate those sites with colon-style types.
- **No English keywords** and **no** user-written type names (`int`, `result`,
  `option`, …). Kinds enter the language **only through surface syntax**
  (literals, leaders, shapes) and checker/runtime labels used in diagnostics.

### Literal width tags (not types / not generics)

Echo has **no** surface type-annotation language and **no** generics. The **only**
explicit kind-related surface (v1) is a **width tag** on a numeric literal that
fixes storage/precision of **that literal only**:

```echo
$ a = <i32> 123_456
$ b = <f64> 3.14
$ c = <i32> -32
```

| Form | Role |
|------|------|
| `<width> number` | **Prefix only** (locked) — tag before the numeric literal |

**Formatting preference (not required):** one space after `>`:

```echo
$ ok = <i32> 1
$ also_ok = <i32>1
$ neg = <i32> -32
$ neg2 = <i32>-32
```

**Sign:** the minus is part of the **literal after the tag**. A width tag
**cannot follow a unary** (`-` / `!`): write `<i32> -32`, not `-<i32> 32`.

**First-version widths:** `i32`, `i64`, `f32`, `f64`.

| Untagged lit | Default width |
|--------------|----------------|
| integer (decimal / `0x` / `0b`) | `i64` |
| float | `f64` |

**Integer bases (v1):** decimal, `0x`/`0X` hex, `0b`/`0B` binary; `_` separators
allowed in the digit body. Through run as signed `i64` (same as decimal).

**Width mixing (v1):**

- Untagged numbers use the defaults above (so they unify with each other).
- **Two different explicit width tags never mix** (e.g. `<i32> 1 + <i64> 2` → error).
- Tagged + untagged: untagged keeps its default; unifies only if that default
  matches the tag (`<i64> 1 + 2` ok; `<i32> 1 + 2` error because `2` is `i64`).

- **No** suffix form `42<i32>`.
- **Not** bind ascription (`$ x : …` is out).
- Width tags apply to **numeric literals only** in v1 (not strings, bytes, durations, …).
- Width names are **machine widths**, not a general type system the user writes
  elsewhere.

### Core value kinds (sketch)

Checker/runtime labels only. Users never write these as types. Each kind is
introduced **by syntax** (or by `%` name for named shapes):

| Kind (internal label) | Surface that produces it |
|----------------------|--------------------------|
| integers / floats | number lits; optional `<i32>` / `<i64>` / `<f32>` / `<f64>` tags |
| bool | `\|` / `_` |
| string | `'…'` / `"…"` (pure vs rich is lit syntax only) |
| bytes | `b'…'` / `b"…"` |
| duration | `5s`, `10ms`, `2m`, `1h` |
| list | `[a, b, c]`; `[]` element unknown until use |
| anon product | `{ k: v }` — not a map |
| named shape | `% name` + `name { … }` / `mod.name { … }` |
| **result** | any `!` path in a function (`^` ok / `!` err) — **not** a struct |
| **option** | bare `^` + valued `^`, no `!` — **not** a struct; **no** `?expr` |
| function | `(…){ … }` values |
| map / set / … | **stdlib later**, not core lits |

**`result` / `option` are not user types:** they are return *shapes* of
functions, produced and consumed only via `^` / `!` and `|` match arms. There is
no `% result`, no constructor, no keyword. Identifiers like `result` remain free
names (no keywords).

### Collections

```echo
$ xs = [1, 2, 3]           ; List
$ row = { name: "Ada", n: 1 }  ; anon struct (product), NOT a map
$ u = user { name: "Ada", visits: 0 }  ; nominal struct
```

- **`[]` stays the list literal** — do not overload for set/map.
- **Maps, sets, and other structures** are **stdlib later**, not language
  literals. Core stays list + anon/named structs.
- **`{ k: v }` = anonymous struct** (structural fields). **Not** a map.

### Option (core)

**Produce** (return shape only; no `?expr`, no none assignment lit):

```echo
$ find_user = (id) {
    ? missing {
        ^                 ; none
    }
    ^ u                   ; some(u)
}
```

| Rule | Meaning |
|------|---------|
| Valued `^` **and** bare `^`, **no** `!` | Function result is **Option[T]** |
| Bare `^` | Return **none** |
| `^ v` | Return **some(v)** |
| Expr `?expr` / none assignment lit | **Out** |
| Any `!` in body | **Result** (see below); + bare/value `^` → **Result(Option[T], E)** |
| `std/option` | **No** |
| Statement `?` | **if** only |

**Consume** (locked) — `|` match; arm dialect for Option:

```echo
| find_user(id) {
    $ user {
        render(user)
    }
    : {
        render_missing()
    }
}
```

| Arm | Meaning |
|-----|---------|
| `$ name { … }` | **some** — payload as `name` |
| `: { … }` | **none** — no payload |

Unhandled Option → compile error.

### Bytes and locators (`p`)

```echo
$ a = b'raw'
$ b = b"with\nescapes"
$ abs = p'/home/user'       ; absolute path
$ rel = p'home/user'        ; relative path
$ url = p'http://xo.run'    ; full URI/URL
```

- Bytes are **language literals**, parallel to strings (pure `b'…'` / rich `b"…"`).
- Through run: heap bytes handle (not a string). Print only after
  `str.from_bytes` (UTF-8 lossy). Content equality via `==`. **No** string/`bytes`
  concatenation with `+` (forbidden; use rich string interp for text).
- **`p` literals** are a single **locator** kind (URI/URL family), not plain
  `String`:
  - absolute path (e.g. starts with `/` on Unix-style)
  - relative path
  - full URI/URL (e.g. `http://…`)
- Pure `p'…'` and rich `p"…"` parallel string/bytes (escapes/interp on rich).
- **Through run:** heap locator handle (distinct from string). Print via
  `str.from_locator` (path/URI text). Content `==`. No `+` concat. No path
  normalization in v1 (stored text is the payload as written).

### Rich string interpolation (v1)

| Form | Meaning |
|------|---------|
| `{name}` | Local / param / `#` const by name |
| `{.field}` | Method body only — field of the receiver `.` |
| `#` consts mixed with live names | Consts bake into the string at lower time |

**No** `{module.export}` or `{a.b}` paths in v1. **No** string `+`.

### Durations

```echo
$ t = 5s
$ d = 10ms
$ u = 100us
```

**V1 suffixes (locked):** `us`, `ms`, `s`, `m`, `h`  
(microseconds, milliseconds, seconds, minutes, hours).

**Through run:** stored as **i64 nanoseconds**. `+` / `-` on two durations;
content `==`. Print via `str.from_duration` (largest exact unit among
`h`/`m`/`s`/`ms`/`us`, else `ns`). Not mixed with plain integers.

### Named struct lits and defaults

- Fields **with a default** on the `%` shape (`$`/`~`/`#` member with `= expr`)
  may be **omitted** in the lit; the default is applied at lower time.
- Fields **without** a default must appear in the lit (`sem-struct-missing-field`).
- Unknown field names → `sem-struct-unknown-field`.
- Method members cannot be set in a lit → `sem-struct-method-field`.
- Duplicate field keys in one lit → `sem-struct-dup-field`.
- Defaults should be lowerable expressions (v1: lits / simple values).

```echo
% user {
    $ name
    ~ visits = 0
}
$ u = user { name: "Ada" }    ; visits defaults to 0
```

### Functions are closed (locked)

A function body (free or method) may only use:

| Allowed | Notes |
|---------|--------|
| **Parameters** | Introduced on the function |
| **Locals** defined in this function | Including loop `* item`, match binds, etc. |
| **`.` / `.field` / `.method`** | **Only** when this activation is a **method call** |
| **`#` constants** | Compile-time; not runtime capture |
| **Import modules** | e.g. `io.print`, `str.from_int` |

| Forbidden | Diagnostic / status |
|-----------|---------------------|
| Outer **data** `$` / `~` (no function return shape) as **values** | `sem-capture` — pass as a parameter or put state on a struct |
| Outer **function values** (binds with a return shape) | **Allowed** as value **or** callee — code refs, not env capture |
| Outer **params** / data `$`/`~` holding a function | **Forbidden** in nested closed bodies — pass the handle as a param to the nested function, or call in the outer body |

**Function values are nameless.** `$ name = (params) { … }` names a **binding**,
not the function. Nested binds are ordinary closed values (no env):

```echo
$ apply = (x) {
    $ double = (n) {
        ^ n + n
    }
    ^ double(x)
}
```

**Not** in the language: closure environments / capture of outer `$`/`~`.
Need outer state → **parameter** or **method on a `%` value**.

#### Implementation status

| Capability | Status |
|------------|--------|
| Bind `$ f = (params) { … }` (incl. nested) | **Yes** — value is a closed body (`FnRef` / `FnValue`) |
| Call `f(args)` when `f` names a function bind | **Yes** — direct body call |
| Pass / rebind / store `f` as a value | **Yes** — handle `{ code, ret_shape }` |
| Return a function value | **Yes** — `^ d` then call the result |
| Store on struct / anon field; call `b.f(args)` | **Yes** — field load + indirect call (methods still win if same name) |
| Call through a param/local (`f(x)` when `f` holds a function) | **Yes** — indirect call; plain **or** result/option |
| Match on call-through result/option | **Yes** — `| f()` / `| g()` when `f`/`g` hold shaped fns |
| Nested body uses outer **free fn bind** | **Yes** — allowed code-ref (not env) |
| Nested body uses outer **param** as call/value | **No** — `sem-capture` (value **and** callee) |
| Methods as first-class values | **No** — methods stay `recv.method()` |

Design: functions are values like numbers (closed, no capture). Runtime value =
`KIND_FN` handle with code pointer + return shape (plain / result / option).
Direct calls still use the body’s LLVM type; indirect calls pick i64 vs i128
from the stored shape. See `docs/hir.md`.

**Honesty:** call-through was incomplete when nested bodies used outer params as
callees (`expr_callee` skipped capture) → SEGV. Fixed: callees share
`check_name_use` with value uses (`echo26/check/capture/003`–`004`).

### Bind before use (locked)

A name is in scope only **after** its bind in that region. Same rule for every
kind of value — numbers, function values, anything:

```echo
; illegal
$ a = b + 4
$ b = 5

; illegal (same rule — not a special “function forward ref” ban)
$ a = b()
$ b = () {
    ^ 1
}

; ok
$ b = 5
$ a = b + 4
```

Unbound use → `sem-unbound`. Function-value binds introduce the name **before**
the body is checked so self-calls (`fact(n-1)` inside `$ fact = …`) work; that
is still “name already bound,” not a second namespace.

```echo
; Good: explicit param
$ add_n = (n, x) {
    ^ n + x
}

; Good: state on struct + method
% counter {
    ~ n
    $ inc = () {
        ~ .n = .n + 1
    }
}

; Error: outer ~ not visible inside f
~ n = 0
$ f = () {
    ^ n    ; sem-capture
}
```

### Method bodies = function bodies (locked)

A **method body and a free function body use the same rules** for returns and
shape:

| Produce | Meaning |
|---------|---------|
| `^ expr` | ok / plain value return |
| bare `^` | option none (when that shape applies) |
| `! expr` | result **err** |

Any `!` path → **result-shaped** (method or free). Consume with `|` the same way
(`$ name` ok/some, `! name` err, `: ` none).

**Difference is entry, not the body language:**

- Method call `recv.method(args)` injects the receiver as `.` for that activation.
- Free call `f(args)` has no `.`.
- Methods are **not** freestanding values; free function values are.

### Method fall-off return (locked)

**In a method body only**, if the function is **plain** (no `!`, no option bare-`^`
pattern) and a control path **falls off the end** without `^` / `!`, that path
returns the **receiver** `.`. If the method uses `!` (or option bare-`^`), fall-off
→ `.` does **not** apply — same as free functions: shape is owned by `^` / `!`.

```echo
$ inc = () {
    ~ .n = .n + 1
    ; no ^  — same as  ^ .
}
c.inc().value()
```

| Context | Fall-off means |
|---------|----------------|
| Method, plain shape | return `.` |
| Free function | return plain 0 / no value (unchanged) |
| Option / Result shape (method or free) | **not** this rule (bare `^` / `!` / `^ v` own the shape) |
| Explicit `^ expr` | always wins (including `^ .` and `^ .field`) |

### Method chains (v1)

```echo
$ n = c.inc().value()
$ n2 = c.inc().inc().value()
```

- Receiver may be a name, `.` (in a method), or **another call** that returns the
  same struct (self-returning methods: `^ .` or plain fall-off).
- Struct type flows through self-returning methods so the next `.method` resolves.
- Field access on a call result (`c.inc().n`) is not required for v1 chains.

### Value vs reference (locked)

**Always:** params / rebind / assignment **copy the binding**.

**Implementation (MIR):** monomorphic free-fn call sites flow named-struct types
onto callee parameters so `f(c)` with `c: % conn` makes methods on the param
resolve (`c.read`). See `collect_free_fn_param_structs` in `echo_mir`.

**What that copies** depends on the value class — there is **no** freestanding
userland “pointer type” between them.

| Class | Pass / rebind | Share object? | `===` |
|-------|----------------|---------------|--------|
| **Ref** (`RefValue`) | copy the **reference** | yes | same object |
| **Value** (`StaticValue`) | copy the **value** | no | same as content / bits |

#### Classification (complete for userland data)

| Kind | Class | Example | Notes |
|------|--------|---------|--------|
| Named struct | **Ref** | `% conn { … }`, `user { … }` | Methods live here |
| Anon struct | **Ref** | `{ k: v }` | Product, not a map |
| List | **Ref** | `[1, 2, 3]` | Shared aggregate |
| Int (`i64` / `<i32>` / …) | **Value** | `42`, `0xff` | Bits |
| Float (`f64` / `<f32>` / …) | **Value** | `3.14` | Bits (like int; not a heap-ref type) |
| Bool | **Value** | `\|`, `_` | 0/1 |
| String | **Value** | `'hi'`, `"a{x}"` | Content value |
| Bytes | **Value** | `b'…'` | Not a string |
| Locator | **Value** | `p'/tmp'` | Path/URI text |
| Duration | **Value** | `5s` | Nanos bits |
| Range | **Value** | `1..10` | Inclusive range value |
| Function value | **Value** | `$ f = (x) { ^ x }` | Closed callable; methods are **not** values |

**One-liner:**

```text
struct or list  → copy the reference (share)
anything else   → copy the value
```

#### IR taxonomy (HIR/MIR direction)

Every data operand is a `Value`:

```text
Value
├── StaticValue   # ints, floats, bool, string, bytes, locator, duration, range, fn, …
└── RefValue
    ├── Struct    # named + anon
    └── List
```

Option / Result are **return shapes** (not leaves under `Value`); after `|`
match the payload is one of the kinds above. Module objects and methods are not
first-class data values.

#### Sockets and other runtime resources (locked)

**Not** language kinds and **not** `RefValue::Socket`.

| Layer | What it is |
|-------|------------|
| **Runtime** (`runtime.tcp_listen`, `tcp_accept`, …) | Opaque heap handles (`KIND_TCP_*`) or a small **anon product** (e.g. accept → `{ conn, remote }`) — free functions only |
| **Std / userland** | **Named structs only**: `% listener`, `% conn`, … with a **`handle` field** holding those opaque bits |

**Passing a “socket” in Echo means passing a struct by reference:**

```echo
% conn {
    $ handle          ; opaque runtime stream id — field only
    $ read = (n) { ^ runtime.tcp_read(.handle, n) }
}

$ c = tcp.connect(addr)     ; RefValue::Struct (% conn)
$ f = (peer) { … }          ; peer is the same: struct ref
f(c)                        ; copy the ref → share one connection
```

**Accept bridge (std must reify):**

```echo
$ a = runtime.tcp_accept(.handle)
; a is an anon struct product { conn: OpaqueHandle, remote: String }
; not a user socket type — reify immediately:
^ conn {
    remote: a.remote,
    handle: a.conn,
    open: |
}
```

- **`a`** = anon **struct ref** (temporary bridge).
- **What callers pass** after `accept` / `listen` / `connect` = **`% conn` / `% listener` struct refs**.
- Sharing I/O = sharing that **struct**; close/mutations via one name affect all aliases.

See [`stdlib.md`](stdlib.md) § Runtime vs std surface.

**Implementation note:** ABI packing (string heap cells, float box at universal
`i64` slots, raw tcp handles) must not reappear as a third userland category.

### Equality (v1)

- `==` / `!=` — **deep** content equality:
  - ints/bools/floats: numeric bits
  - strings / bytes / locators: payload content
  - lists: same length + recursive deep eq on elements
  - structs: same field names + recursive deep eq on values
- `===` / `!==` — **identity**:
  - **structs and lists:** same object (same ref)
  - **StaticValue kinds:** same as deep equality (no separate user-visible “same heap cell” for strings)
- Different kinds → **check error** when types known; at runtime mixed kinds →
  not equal (no silent cross-kind true).

### Arithmetic (v1)

| Op | Rule |
|----|------|
| `+ - *` on ints | same int width; result that width |
| `+ - *` on floats | same float width; result that width |
| `/` int ÷ int | **integer division**, truncate toward zero |
| `/` float ÷ float | float division |
| int with float | **error** (no implicit mix) |
| different explicit widths | **error** (see width mixing above) |

### Function returns (direction)

- **Multiple return paths** with different value kinds → result is a **union** of
  those kinds (track in inference).
- **Fall-off / bare `^`** participates as **nothing** in that union when mixed
  with valued returns.
- **Result** when any `!` path exists; plain value when no `!` in the body.

### Method typing vs runtime (locked)

- Methods exist only on **named `%` shapes** (and values known to be that type).
- **`/ runtime` exports are free functions only** — never method receivers.
- Std **must** wrap runtime resources in **`% conn` / `% listener` / …`** so
  userland only sees **struct refs** (plus ordinary values) — never bare socket
  handles as a language type. Typing uses normal struct return / match refine.
- See `docs/stdlib.md` § Runtime vs std surface.

### Named-struct return unions (locked)

When every valued plain `^` is a **named struct lit** (`type { … }`), the function
**returns those types** (one or several):

```echo
$ shape = (k) {
    ? k == 0 {
        ^ circle { r: 1 }
    }
    ^ rect { w: 2, h: 3 }
}
```

| Rule | Meaning |
|------|---------|
| Single type | Call result has that struct type (methods/fields flow as today) |
| Multiple types (**union**) | Call result is **not** monomorphic — no method call on the raw result until refined |
| Refine | `\| x { % circle { … } % rect { … } }` — inside a `% T` arm, `x` is typed as `T` for field/method flow |
| Runtime | Named lits still set type tags; `% T` match uses `struct_type_is` |
| Exhaustiveness | Default `: { }` allowed; static exhaustiveness of all union members is **not** required in v0 |

Not a user-written type syntax — inferred from return paths only (same spirit as result/option shapes).

### Option vs Result vs `!`

| Mechanism | Role |
|-----------|------|
| `^ expr` / bare `^` | Return ok value, none (option-shaped), or plain value |
| `! expr` | Function return **err** payload (result shape) — not process abort |
| option | Only as **fn shape** from bare `^` + valued `^` (no `?expr`) |
| result | Fn outcome **ok \| err** when any `!` path exists (syntax-driven) |
| Hard process abort | **Not designed** (no panics) |

### Producing Result

Inside a function:

```echo
$ f = (x) {
    ? x < 0 {
        ! "negative"
    }
    ^ x
}
```

- `^ x` → ok branch when the function is result-shaped  
- `! "negative"` → err branch  
- **If any `!` path exists**, the function is **result-shaped** (ok | err).  
  - Valued `^` + `!` → result of those payloads.  
  - Bare `^` + valued `^` + `!` → result whose ok side is option-shaped.  
- **If no `!`**, but bare `^` + valued `^`: **option-shaped** (not result).  
- **If no `!` and no bare-`^` option pattern**: plain value (union of returns).

### Inclusive range values (locked)

```echo
$ r = 1..3                 ; value: integers 1, 2, 3
* x : r { … }              ; iterate
* y : 4..6 { … }           ; same with a range expr
| n {
    1..10 { … }            ; match if n is in the range (inclusive)
}
```

| Rule | Meaning |
|------|---------|
| Form | `lo..hi` (token `..`) |
| Bounds | Inclusive when `lo ≤ hi`; empty when `lo > hi` |
| Ends | Integer-like values (inferred `i64`) |
| Value | First-class handle (heap); deep `==` compares lo/hi |
| For-in | `* item : range` yields each integer in order |
| Match arm | Syntactic `lo..hi` means **membership**, not “equals a range object” |

### Ordinary value match arms (locked)

Ordinary scrutinees (not Option/Result dialect) use **value arms** and/or
**type arms**:

```echo
| x {
    1, 2, y {
        ; runs if x == 1 or x == 2 or x == y (deep ==)
    }
    4..6 {
        ; runs if 4 ≤ x ≤ 6
    }
    % circle {
        ; runs if x is a named struct with type tag `circle`
    }
    : {
        ; default
    }
}
```

- Each value arm head is one or more expressions (no trailing comma).
- Match if scrutinee **deep-equals** any listed value, or lies in a syntactic range.
- Expressions need not be literals (names, calls, arith, …) — same as any value.
- **`% TypeName { … }`** — type arm: match when the scrutinee’s runtime type tag
  equals `TypeName` (set when constructing a tagged lit `TypeName { … }`). The
  type name must resolve to a `%` struct in scope (`sem-match-type`).
- Default still `: { … }`. No ordinary `$ name` arm.
- Cannot mix value/`% type` arms with `$` / `!` Option/Result arms on the same match.

### Handling Result and Option with `|` match (locked)

**Shared rule:** `$ name { … }` = success-with-payload arm.

| Scrutinee | Success arm | Empty / fail arm |
|-----------|-------------|------------------|
| **Option** | `$ name { … }` some | `: { … }` none |
| **Result** | `$ name { … }` ok | `! name { … }` err |

```echo
; Option
| find_user(id) {
    $ user { render(user) }
    : { render_missing() }
}

; Result
| load_user(id) {
    $ user { render(user) }
    ! error { io.log(error) }
}
```

- Statement `! expr` and match arm `! name` are the same **error** story
  (produce vs bind). Dual-use by position.
- Statement `:` (else) / match default / Option **none** arm: same glyph,
  contextual meaning.
- Unhandled **Option** or **Result** → compile error.
- **No** propagate glyph.

### Modules

Imports stay **module-scoped** (`http.serve`, not bare flood). See `modules.md`.

---

## Pipeline

```text
echo_source → echo_lexer → echo_parser → echo_semantics
                              ↓
                       echo_diagnostics
```

## Facts (v0 implemented today)

| Rule | Code |
|------|------|
| No shadowing / reintroduce of a visible name | `sem-shadow` |
| `~` cannot assign through an immutable / const binding | `sem-immutable` |
| `#` name must be SCREAMING_SNAKE | `sem-hash-name` |
| `#` init is const (lits + `#` only; no calls) | `sem-const` |
| Receiver `.` / `.field` only in method bodies | `sem-receiver` |
| `<` / `>` only inside loops | `sem-break` / `sem-continue` |
| Top-level is the program body; `^` returns process status | (no `sem-return` at top-level) |
| Name used before its bind (any value, incl. calls) | `sem-unbound` |
| Assign target name must already exist | `sem-unbound` |
| List push `~ xs[] = e` / `~ a.b[] = e` | append via runtime list_push (index omitted) |
| `module.foo` not an export | `sem-module-export` |
| Unhandled Result / Option value | `sem-unhandled-result` / `sem-unhandled-option` |
| Incomplete/wrong `|` arms | `sem-match-incomplete` / `sem-match-arm` |
| `!` outside function | `sem-error-return` |
| Kind mismatch | `sem-type-mismatch` |
| Call non-function | `sem-not-callable` |
| Wrong arity | `sem-arity` |
| Missing field | `sem-no-field` |

- `$` / `#` introduce immutable / const once; `~` intro or update mutable.
- Method bodies = function values that are **struct members**.
- Imports: last path segment → module name; use `module.export`.

## Inference (v1)

Runs after name/effect checks (`infer.rs` + `unify.rs`):

- Scalars, lists, index, anon/named structs, fields  
- Ops (no int/float mix; int `/` truncates)  
- Calls (`sem-not-callable`, `sem-arity`; imported fns: arity unknown)  
- Codes: `sem-type-mismatch`, `sem-not-callable`, `sem-arity`, `sem-no-field`  

## Open questions

- Width tags / bytes / duration / `p` lits (lexer + kinds)  
- Locator classification edge cases  
- Stdlib map/set design (post-core)  
- Richer cross-module function signatures
