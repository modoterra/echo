# First-Class Typed Error Values

This document specifies the first version of Echo's language-level error
system. It is an RFA-style design note, not an implementation status report.

Echo should model errors as first-class typed values with distinct control-flow
semantics:

- `error` declares nominal error types and constructs error values.
- `panic` raises errors into panic flow.
- `recover` handles panic flow by matching typed errors.

The central invariant is:

```text
EchoValue::Object and EchoValue::Error are distinct value categories.
Only EchoValue::Error participates in panic/recover.
EchoError can carry any EchoValue, including EchoValue::Object, through fields
or generics.
```

This invariant is the rule every parser, semantic, runtime, and IDE feature should preserve: error-ness is a value category, not an object flag.

Do not model errors as objects with a mutable error flag. Objects and errors may
both be record-like, but they are different language categories:

```text
object = domain/data value category
error  = failure/control-flow value category

Objects are not errors.
Errors are not objects.
Errors can carry objects.
```

This category split keeps domain data and failure control flow separate while still allowing errors to include rich structured payloads.

## Error Declarations

Echo should add an `error` declaration form:

```echo
error FileNotFound {
    path: String
    message: String = "file not found"
}

error InvalidConfig {
    path: String
    line: Int
    message: String
}
```

This declaration form is for nominal failure cases with typed fields, defaults, and stable names recover arms can match.

Rules:

- `error Name { fields }` declares a nominal error type.
- Error types are distinct from object types.
- Only error types may appear in typed `recover` arms.

This should be invalid:

```echo
object User {
    id: Int
}

recover {
    risky()
} {
    User as err => {}
}
```

This invalid example shows the boundary: object types may carry data, but they cannot be used as typed recover targets.

`User` is an object type, not an error type.

## Error Values

Echo should add an `error` expression form for constructing error values:

```echo
let err = error FileNotFound {
    path: "echo.toml"
}
```

This expression constructs an error value without raising it, which is useful when code needs to store, return, or wrap failures explicitly.

This produces a distinct error value:

```text
EchoValue::Error(EchoError { type_id: FileNotFound, fields: ... })
```

The runtime category is the key result: the value is represented as an error even though its fields can look record-like.

Rules:

- `error Name { values }` constructs an `EchoError` value of the named error
  type.
- The named type must be an error type.
- The result is `EchoValue::Error`.
- `echo.is_error(err)` should be true.
- `echo.is_object(err)` should be false.

Object and error checks must stay distinct:

```echo
object User {
    id: Int
}

error UserError {
    user: User
    message: String
}

let user = User { id: 1 }
let err = error UserError {
    user: user,
    message: "bad user",
}

echo.is_object(user) // true
echo.is_error(user)  // false

echo.is_object(err)  // false
echo.is_error(err)   // true
```

This example is the practical test for the model: a domain object can be carried by an error, but object and error predicates still report different categories.

A future shared record predicate may treat both objects and errors as
record-like values:

```echo
echo.is_record(User { id: 1 })                    // true, future feature
echo.is_record(error FileNotFound { path: "x" })  // true, future feature
```

This future predicate would describe field shape only; it must not replace `is_object` or `is_error`.

Do not add this predicate as part of the first typed-error slice unless the
record model already exists.

## Panic

Echo should add a `panic` expression or statement form:

```echo
panic FileNotFound {
    path: "echo.toml"
}

let err = error FileNotFound {
    path: "echo.toml"
}

panic err
panic "bad"
panic 5
```

These forms cover the raising boundary: typed errors can be constructed and raised directly, existing errors can be re-raised, and non-error payloads are wrapped.

Rules:

- `panic Name { values }` constructs and raises an error value of the named
  error type.
- `panic <error-value>` raises the existing error value.
- `panic <non-error-value>` raises a built-in generic `Panic` or
  `GenericError` value carrying the original value.
- `panic <object-value>` does not make the object recoverable by its object
  type. It wraps the object in a generic error payload.

Example:

```echo
object User {
    id: Int
}

recover {
    panic User { id: 1 }
} {
    User as err => {
        // invalid recover arm, or never matches; User is not an error type
    }

    Panic as err => {
        echo err.value // User { id: 1 }
    }

    error as err => {
        panic err
    }
}
```

This example demonstrates why panic payloads must be normalized into errors before recovery: the `User` object is carried by `Panic`, not matched as a `User`.

The generic built-in error type should eventually be equivalent to:

```echo
error Panic<T> {
    value: T
    message: String? = null
}
```

The generic shape preserves the original payload type for code that recovers and inspects generic panic values.

If generic support is not ready, v1 may use a non-generic runtime shape:

```echo
error Panic {
    value: mixed
    message: String? = null
}
```

The non-generic fallback keeps behavior implementable while still preserving the value that caused panic flow.

## Recover

Echo should add a `recover` expression or block form:

```echo
let config = recover {
    load_config("echo.toml")
} {
    FileNotFound as err => default_config()

    InvalidConfig as err => {
        echo err.message
        panic err
    }

    error as err => {
        panic err
    }
}
```

This recover block is the user-facing handling shape: specific error arms handle known failures and the catch-all arm preserves unhandled panic flow.

Rules:

- `recover { block } { arms }` evaluates the body block.
- If the body returns normally, `recover` returns the body result.
- If the body panics, `recover` matches the active `EchoError` against arms.
- Arms are matched top-to-bottom.
- Typed arms must name error types.
- The catch-all arm is `error as err => expr`.
- Inside a typed arm, the binding is narrowed to that error type.
- Inside `error as err`, the binding has the general error type.

Block bodies should be supported:

```echo
let config = recover {
    load_config("echo.toml")
} {
    FileNotFound as err => {
        log("missing config", path: err.path)
        default_config()
    }

    InvalidConfig as err => {
        log(err.message)
        panic err
    }
}
```

Block arms support logging, fallback construction, and re-panicking without forcing all recovery logic into single expressions.

## Generics

Errors should be able to carry any `EchoValue`, including objects, through typed
generic fields:

```echo
object UserInput {
    email: String
}

error ValidationError<T> {
    value: T
    message: String
}

let err = error ValidationError<UserInput> {
    value: UserInput { email: "bad" },
    message: "invalid email",
}
```

This pattern lets validation errors carry the exact input shape that failed, which makes recovery code more precise.

Expected behavior:

```echo
echo.is_error(err)   // true
echo.is_object(err)  // false

err.value.email      // accessible once field/type support exists
```

The expected behavior keeps category checks separate from payload access: the error is not an object, but its field can contain one.

Recover matching should eventually support specialized and unspecialized generic
matches:

```echo
let user = recover {
    parse_user(input)
} {
    ValidationError<UserInput> as err => {
        echo err.value.email
        User::guest()
    }

    ValidationError as err => {
        panic err
    }

    error as err => {
        panic err
    }
}
```

This example shows the desired recovery hierarchy: handle a specific payload specialization first, then fall back to the general error family.

Minimum v1 behavior:

```text
ValidationError as err matches all ValidationError<T> specializations.
```

This minimum rule gives generic errors useful recovery semantics before full specialization matching exists.

Specialized generic matching can be a follow-up if full generic matching is too
large for the first slice.

## AST Requirements

Add AST nodes for the following concepts. Exact Rust names may differ, but the
language concepts should be preserved:

```rust
ErrorDecl {
    name,
    generic_params,
    fields,
    span,
}

ErrorExpr {
    name,
    generic_args,
    fields,
    span,
}

PanicExpr {
    payload,
    span,
}

RecoverExpr {
    body,
    arms,
    span,
}

RecoverArm {
    pattern,
    binding,
    body,
    span,
}

RecoverPattern {
    ErrorType(name, generic_args),
    AnyError,
}
```

These AST concepts preserve typed-error syntax without committing to exact Rust names or a REPL-only evaluator.

`panic` may be represented as an expression, a statement, or both, depending on
the existing AST conventions. The language behavior must be shared; do not add
REPL-only panic handling.

## Parser Behavior

Support these forms:

```echo
error FileNotFound {
    path: String
    message: String = "file not found"
}
```

This parser form introduces a nominal error type with typed fields and defaults.

```echo
let err = error FileNotFound {
    path: "echo.toml"
}
```

This parser form constructs an error value and should be valid anywhere an expression is accepted.

```echo
panic FileNotFound {
    path: "echo.toml"
}
```

This parser form combines construction and panic flow for the common case where the error is raised immediately.

```echo
panic err
panic "bad"
panic 5
```

These payload forms ensure panic syntax works for existing error values and for values that must be wrapped by the built-in panic error.

```echo
let value = recover {
    risky()
} {
    FileNotFound as err => fallback()
    error as err => panic err
}
```

This parser form is the core recovery shape: body first, then typed arms and the catch-all `error` arm.

Typed error construction should require braces:

```echo
panic EndOfFile {}
let err = error EndOfFile {}
```

The empty-brace form keeps construction unambiguous even for errors without fields.

Do not rely on these ambiguous forms unless the language has a clear
identifier/type namespace distinction:

```echo
panic EndOfFile
error EndOfFile
```

These forms are intentionally deferred because they make it harder for parser and diagnostics to distinguish values from types.

## Runtime Model

Introduce a distinct `EchoError` representation in the runtime value model.

Conceptual shape:

```rust
pub enum EchoValue {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    String(EchoString),
    Array(EchoArray),
    Object(EchoObject),
    Error(EchoError),
}
```

The runtime value enum needs a real error variant so predicates, panic flow, and recovery cannot confuse errors with objects.

Conceptual error shape:

```rust
pub struct EchoError {
    pub type_id: EchoErrorTypeId,
    pub generic_args: Vec<EchoTypeId>,
    pub fields: EchoFieldMap,
    pub cause: Option<Box<EchoError>>,
    pub trace: Option<EchoTrace>,
}
```

This conceptual shape captures the metadata needed for matching, payload fields, causes, and future traces without prescribing memory layout.

Adapt this shape to Echo's actual memory model if values are boxed,
reference-counted, interned, or arena-allocated elsewhere.

Required predicates:

```rust
is_object(EchoValue::Object(_)) == true
is_object(EchoValue::Error(_)) == false

is_error(EchoValue::Error(_)) == true
is_error(EchoValue::Object(_)) == false
```

These predicates are the runtime acceptance test for keeping object and error categories separate.

Optional future predicate:

```rust
is_record(Object) == true
is_record(Error) == true
```

This future predicate can describe field-like access, but it must not participate in panic/recover matching.

## Control-Flow Model

Evaluation should represent normal value flow and panic flow explicitly in the
interpreter, VM, MIR, or codegen layer.

Conceptual model:

```rust
enum EchoEvalResult {
    Value(EchoValue),
    Panic(EchoError),
}
```

This result type makes panic flow explicit in an interpreter, VM, MIR, or codegen layer without relying on host-language exceptions.

Rules:

- Normal expressions and blocks return `Value(v)`.
- `panic` returns `Panic(error)`.
- `recover` evaluates its body; if it receives `Value(v)`, it returns
  `Value(v)`.
- If `recover` receives `Panic(err)`, it matches `err.type_id` against arms.

Do not jump directly to LLVM native exception handling for v1 unless Echo
already has a reliable exception lowering path. Prefer explicit propagation
first because it is simpler to validate.

## Type Rules

Minimum rules:

- Error declarations create nominal error types.
- Object declarations create nominal object types.
- The two namespaces may share syntax, but not kind.
- Only error types can be constructed with `error Name { ... }`.
- Only error types can be used in typed `recover` arms.
- Only `EchoValue::Error` can be raised directly as itself.
- Non-error panic payloads are wrapped in the built-in `Panic` or
  `GenericError` type.

Recommended static checks:

- `recover` arm type must be an error type.
- `panic Name { ... }` requires `Name` to be an error type.
- `error Name { ... }` expression requires `Name` to be an error type.
- Field initializers must satisfy declared field types once type checking
  exists.

## Examples

Basic typed error:

```echo
error FileNotFound {
    path: String
    message: String = "file not found"
}

fn load_config(path: String): String {
    panic FileNotFound { path: path }
}

let config = recover {
    load_config("echo.toml")
} {
    FileNotFound as err => "{}"
    error as err => panic err
}

echo config
```

This example shows the full happy-path/fallback loop: a typed panic is raised, recovered by its nominal type, and the program continues with a replacement value.

Error distinct from object:

```echo
object User {
    id: Int
}

error UserError {
    user: User
    message: String
}

let user = User { id: 1 }
let err = error UserError {
    user: user,
    message: "bad user",
}

echo.is_object(user) // true
echo.is_error(user)  // false

echo.is_object(err)  // false
echo.is_error(err)   // true
```

This example is the smallest end-to-end check that errors can carry objects without becoming objects themselves.

Panic with a non-error payload wraps a generic error:

```echo
recover {
    panic 5
} {
    Panic as err => echo err.value
    error as err => panic err
}
```

This recovery shape shows how non-error panic payloads become recoverable only through a built-in error wrapper.

Generic validation error:

```echo
object UserInput {
    email: String
}

error ValidationError<T> {
    value: T
    message: String
}

let user = recover {
    panic ValidationError<UserInput> {
        value: UserInput { email: "bad" },
        message: "invalid email",
    }
} {
    ValidationError as err => {
        echo err.message
        UserInput { email: "guest@example.com" }
    }
}
```

This example applies typed generic errors to validation: recovery can inspect the message and return a safe replacement value.

## Acceptance Criteria

Parser and AST:

- Can parse error declarations.
- Can parse error value construction.
- Can parse panic expressions or statements.
- Can parse recover blocks with typed arms and catch-all `error` arms.
- Rejects or reports clear diagnostics for typed recover arms using non-error
  object types.

Runtime and type behavior:

- `EchoValue` has a distinct error variant/category.
- `EchoError` is not represented as `EchoObject` at the language level.
- `echo.is_error(error MyError {})` returns true.
- `echo.is_object(error MyError {})` returns false.
- Objects can be stored inside errors as fields or payloads.
- `panic` of a typed error raises that error.
- `panic` of an existing error raises that error.
- `panic` of a non-error wraps it in the built-in `Panic` or `GenericError`
  error type.
- `recover` matches by error type.
- `recover error as err` catches any error.

Validation examples:

```echo
object User {
    id: Int
}

error MyError {
    message: String
}

let user = User { id: 1 }
let err = error MyError { message: "bad" }

echo.is_object(user) // true
echo.is_error(user)  // false

echo.is_object(err)  // false
echo.is_error(err)   // true
```

This validation block proves the runtime category split from user code, not only from internal Rust types.

```echo
recover {
    panic MyError { message: "bad" }
} {
    MyError as err => echo err.message
    error as err => panic err
}
```

This validation block proves recover matching by specific error type and catch-all propagation for unhandled errors.

## Non-Goals

- Do not implement errors as normal objects with a mutable error flag exposed to
  userland.
- Do not allow arbitrary objects to become panicable control-flow values.
- Do not add APIs equivalent to `echo.mark_error(user)`.
- Do not implement a separate REPL-only evaluator for error behavior.
- Do not use LLVM native exception handling for v1 unless the project already
  has a reliable explicit reason and test coverage for that path.

Invalid direction:

```echo
let user = User { id: 1 }
echo.mark_error(user)
panic user
```

This invalid direction would blur data and control flow by mutating an object into something panicable.

Correct direction:

```echo
error UserError {
    user: User
    message: String
}

panic UserError {
    user: user,
    message: "bad user",
}
```

This direction keeps the domain object intact and wraps it in a typed error value before entering panic flow.

Core principle:

```text
Objects are data.
Errors are failure values.
Errors can carry data.
Only errors participate in panic/recover.
```

This principle is the final audit rule for the feature: if a design path makes arbitrary objects recoverable as errors, it violates the typed-error model.
