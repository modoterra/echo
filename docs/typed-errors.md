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

Do not model errors as objects with a mutable error flag. Objects and errors may
both be record-like, but they are different language categories:

```text
object = domain/data value category
error  = failure/control-flow value category

Objects are not errors.
Errors are not objects.
Errors can carry objects.
```

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

`User` is an object type, not an error type.

## Error Values

Echo should add an `error` expression form for constructing error values:

```echo
let err = error FileNotFound {
    path: "echo.toml"
}
```

This produces a distinct error value:

```text
EchoValue::Error(EchoError { type_id: FileNotFound, fields: ... })
```

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

A future shared record predicate may treat both objects and errors as
record-like values:

```echo
echo.is_record(User { id: 1 })                    // true, future feature
echo.is_record(error FileNotFound { path: "x" })  // true, future feature
```

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

The generic built-in error type should eventually be equivalent to:

```echo
error Panic<T> {
    value: T
    message: String? = null
}
```

If generic support is not ready, v1 may use a non-generic runtime shape:

```echo
error Panic {
    value: mixed
    message: String? = null
}
```

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

Expected behavior:

```echo
echo.is_error(err)   // true
echo.is_object(err)  // false

err.value.email      // accessible once field/type support exists
```

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

Minimum v1 behavior:

```text
ValidationError as err matches all ValidationError<T> specializations.
```

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

```echo
let err = error FileNotFound {
    path: "echo.toml"
}
```

```echo
panic FileNotFound {
    path: "echo.toml"
}
```

```echo
panic err
panic "bad"
panic 5
```

```echo
let value = recover {
    risky()
} {
    FileNotFound as err => fallback()
    error as err => panic err
}
```

Typed error construction should require braces:

```echo
panic EndOfFile {}
let err = error EndOfFile {}
```

Do not rely on these ambiguous forms unless the language has a clear
identifier/type namespace distinction:

```echo
panic EndOfFile
error EndOfFile
```

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

Adapt this shape to Echo's actual memory model if values are boxed,
reference-counted, interned, or arena-allocated elsewhere.

Required predicates:

```rust
is_object(EchoValue::Object(_)) == true
is_object(EchoValue::Error(_)) == false

is_error(EchoValue::Error(_)) == true
is_error(EchoValue::Object(_)) == false
```

Optional future predicate:

```rust
is_record(Object) == true
is_record(Error) == true
```

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

Panic with a non-error payload wraps a generic error:

```echo
recover {
    panic 5
} {
    Panic as err => echo err.value
    error as err => panic err
}
```

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

```echo
recover {
    panic MyError { message: "bad" }
} {
    MyError as err => echo err.message
    error as err => panic err
}
```

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

Core principle:

```text
Objects are data.
Errors are failure values.
Errors can carry data.
Only errors participate in panic/recover.
```
