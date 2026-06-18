# Effect Blocks

This document captures the design direction for first-class `effect {}` blocks
in Echo.

## Goal

`effect {}` should make optional, result, task, future, and error-producing
code read like direct imperative code while preserving strong static typing.

The feature is inspired by monadic bind, Rust's `?`, and Effect-TS-style typed
errors, but the user-facing value is not functional-programming purity. The
value is flatter control flow, less manual unwrap boilerplate, and precise type
information through the compiler.

## Core Syntax

An effect block is an expression:

```echo
$result = effect {
    $user = findUser($id)
    $profile = loadProfile($user)
    loadAccount($profile)
}
```

The block evaluates to a value. The resulting type is inferred from the final
expression or from an explicit `return`.

The last expression is returned implicitly:

```echo
$account = effect {
    $user = findUser($id)
    loadAccount($user)
}
```

is equivalent to:

```echo
$account = effect {
    $user = findUser($id)
    return loadAccount($user)
}
```

## Flattening Semantics

Inside an effect block, assignments bind successful inner values and
short-circuit on failure:

```echo
$account = effect {
    $user = findUser($id)
    $profile = loadProfile($user)
    $account = loadAccount($profile)

    return $account
}
```

For optional values, this behaves like:

```echo
$user = findUser($id)

if ($user === none) {
    return none
}

$profile = loadProfile($user)

if ($profile === none) {
    return none
}

$account = loadAccount($profile)

if ($account === none) {
    return none
}

return $account
```

The surface syntax stays imperative. The compiler owns the unwrapping and
short-circuiting semantics.

## Supported Effect Shapes

Initial support should target these generic shapes:

```echo
optional<T>
result<T, E>
task<T>
future<T>
```

### Optional

`optional<T>` unwraps to `T` inside the block and short-circuits with `none`.

```echo
$user = effect {
    $id = parseUserId($input)
    findUser($id)
}
```

If `parseUserId()` returns `none`, the whole effect returns `none`.

### Result

`result<T, E>` unwraps to `T` inside the block and short-circuits on error.
Error types are accumulated by the compiler.

```echo
function findUser(int $id): result<User, UserNotFound>
function loadProfile(User $user): result<Profile, ProfileMissing>
function loadAccount(Profile $profile): result<Account, AccountMissing>

$account = effect {
    $user = findUser($id)
    $profile = loadProfile($user)
    loadAccount($profile)
}
```

The inferred type is:

```echo
result<Account, UserNotFound | ProfileMissing | AccountMissing>
```

### Task And Future

Tasks and futures should integrate with `effect {}` without forcing nested
success/error handling around every asynchronous boundary.

Explicit `join` is the conservative initial syntax:

```echo
$task = defer {
    return loadUser(123)
}

$user = effect {
    join $task
}
```

For a task whose joined result is `result<User, LoadUserError>`, the effect
returns:

```echo
result<User, LoadUserError>
```

More complex task chaining stays flat:

```echo
$userTask = defer {
    return findUser(123)
}

$account = effect {
    $user = join $userTask
    $profile = loadProfile($user)
    loadAccount($profile)
}
```

Open design point: allowing implicit task binding:

```echo
$user = $userTask
```

inside `effect {}` is attractive, but it makes task boundaries less visible.
The first implementation should prefer explicit `join` unless the type system
and diagnostics make implicit joins obvious and predictable.

## Type Inference

The compiler must track generic effect shapes through the whole block:

```echo
optional<T>
result<T, E>
task<T>
future<T>
```

Within the block, a binding sees the unwrapped success type:

```echo
$account = effect {
    $user = findUser($id)          // result<User, UserError> binds User
    $profile = loadProfile($user) // result<Profile, ProfileError> binds Profile
    loadAccount($profile)         // result<Account, AccountError>
}
```

The block result preserves the outer effect shape and accumulates failures:

```echo
result<Account, UserError | ProfileError | AccountError>
```

Mixed effect shapes need a clear unification rule before implementation. For
example, combining `optional<T>` and `result<U, E>` could either be rejected or
lift `none` into a typed result error. The first version should reject ambiguous
mixing unless an explicit conversion exists.

## Typed Error Accumulation

Typed error accumulation is a core requirement, not a later presentation layer.

Given:

```echo
function findUser(int $id): result<User, UserNotFound>
function loadProfile(User $user): result<Profile, ProfileMissing>
function loadAccount(Profile $profile): result<Account, AccountMissing>
```

This:

```echo
$account = effect {
    $user = findUser($id)
    $profile = loadProfile($user)
    loadAccount($profile)
}
```

infers:

```echo
result<
    Account,
    UserNotFound
    | ProfileMissing
    | AccountMissing
>
```

No manual union construction should be required. Error unions are compiler type
facts, not runtime list values.

## Error Handling Integration

Effect results should work naturally with the broader error-handling model:

```echo
try {
    $account = effect {
        $user = findUser($id)
        $profile = loadProfile($user)
        loadAccount($profile)
    }

    render($account)
}
catch ($error) {
    renderError($error)
}
```

The exact relationship between `result<T, E>` and `try`/`catch` still needs an
ADR-level decision:

- `try` may unwrap `result<T, E>` and catch `E`.
- `try` may remain exception-only while results are handled explicitly.
- Echo may provide a conversion boundary between typed results and thrown
  errors.

The effect-block design should not assume exception-heavy control flow.

## Compiler Model

The compiler should lower effect blocks into explicit bind-like control flow.
The source-level model is:

```echo
effect {
    $a = step1()
    $b = step2($a)
    step3($b)
}
```

Conceptually, this is:

```echo
step1()
    .bind(fn($a) =>
        step2($a)
            .bind(fn($b) =>
                step3($b)
            )
    )
```

Actual generated IR does not need runtime monad objects or a runtime monad
framework. Prefer compile-time lowering to efficient imperative control flow:

1. Evaluate the next step.
2. Test the effect shape for short-circuit state.
3. Return the short-circuit value if present.
4. Bind the unwrapped value into the local variable.
5. Continue.

This lowering belongs in the shared compiler pipeline, not in the REPL or CLI.
REPL examples should exercise the same parser, type analysis, IR, and runtime
semantics as files.

## Interaction With Existing Echo Features

### Imperative Syntax

Effect blocks should preserve Echo's imperative programming model. They are not
a request to make ordinary code point-free or callback-heavy.

### Generics

Effect blocks depend on first-class generic type understanding. The compiler
must be able to inspect `optional<T>`, `result<T, E>`, `task<T>`, and
`future<T>`, bind `T`, and preserve or combine the outer shape.

### Reflection

Reflection should eventually expose effect-block-inferred function return
types. For example, a function returning an effect block should reflect the
inferred `result<T, E1 | E2>` or explicit declared return type.

### Concurrency

`defer` and `join` are the initial concurrency bridge:

```echo
$account = effect {
    $user = join $userTask
    $profile = loadProfile($user)
    loadAccount($profile)
}
```

This should flatten task completion and typed error propagation without hiding
where concurrency boundaries occur.

## Acceptance Criteria

- `effect {}` parses as an expression.
- The block result type is inferred from the final expression or explicit
  `return`.
- `optional<T>` is supported and short-circuits on `none`.
- `result<T, E>` is supported and short-circuits on error.
- `task<T>` and `future<T>` integrate with `join`.
- Successful bindings see unwrapped `T` values.
- Error types accumulate automatically into precise unions.
- Generic type information is preserved.
- Nested unwrapping and pyramid-of-doom conditionals are unnecessary.
- Lowering produces efficient imperative control flow.
- No runtime monad framework is required.
- The behavior works through regular files and the REPL because it is owned by
  shared parser, type, lowering, and runtime layers.

## Open Questions

- Should task/future values bind implicitly inside `effect {}`, or should
  `join` remain required?
- What is the first-class spelling for optional absence: `none`, `null`, or a
  distinct optional constructor?
- Can optional and result effects mix in one block, and if so what converts
  `none` into a typed error?
- Does `try` unwrap typed results, or does it remain separate from result-based
  error handling?
- Where in the compiler pipeline should effect lowering happen relative to type
  inference and IR generation?
- Should effect blocks support custom effect-like types through traits,
  interfaces, or compiler-known generic shapes only?
