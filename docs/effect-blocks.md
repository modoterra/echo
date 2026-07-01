# Effect Blocks

`effect {}` is Echo's monadic expression block for action values. It sequences
`option<T>`, `outcome<T, E>`, and `future<T, E>` without turning ordinary Echo
code into callback chains or adding a runtime monad framework.

The core action relationship is:

```echo
option<T> <: action<T, void>
outcome<T, E> <: action<T, E>
future<T, E> <: action<T, E>
```

An effect block contains zero or more `let` bindings followed by one final
expression. If a `let` binding receives an action value, the binding unwraps the
success payload or short-circuits the selected action family. If a binding
receives a pure value, it behaves like a normal local helper.

```echo
let $label = effect {
    let $user = load_user($id)
    let $prefix = "User"

    ok "{$prefix}: {$user.name}"
}: outcome<string, LoadError>
```

The final expression may be pure or action-valued. The postfix type after the
closing brace selects or narrows the concrete action family. Without an
annotation, wrapper expressions can seed inference: `ok $value` defaults toward
`outcome<T, unknown>`, `fail $error` toward `outcome<unknown, E>`, `some $value`
toward `option<T>`, and `none` toward `option<unknown>` until context narrows
the missing type.

## Action Families

`option<T>` represents a present value or absence:

```echo
let $found: option<User> = some $user
let $missing: option<User> = none
```

`none` is not `null` and is not bare `void` control flow. `option<T>` narrows
from `action<T, void>` because its short-circuit channel carries no payload.

`outcome<T, E>` represents success or typed failure:

```echo
let $loaded: outcome<User, LoadError> = ok $user
let $failed: outcome<User, LoadError> = fail $error
```

`ok` constructs `success<T>`, and `fail` constructs `failure<E>`. In an
`outcome<T, E>` context, the missing side is inferred from the expected type.

`future<T, E>` is a monadic future-like action value. It is not an event-loop
handle, has no direct constructor syntax, and is opaque to pattern matching.
Future values come from future-producing APIs or from future-targeted effects.

```echo
let $profile = effect {
    let $user = fetch_user($id)

    $user.profile
}: future<Profile, NetworkError|LoadError>
```

`effect` uses one concrete action family per block. Do not implicitly mix
`option`, `outcome`, and `future`; convert explicitly when moving between
families.

```echo
let $user = effect {
    let $id = maybe_user_id($request).ok_or(missing_user_id_error())
    let $user = load_user($id)

    $user
}: outcome<User, LoadError>
```

Within one concrete action family, failure payloads union naturally:

```echo
let $account = effect {
    let $user = load_user($id)
    let $profile = load_profile($id)

    Account.create($user, $profile)
}: outcome<Account, UserError|ProfileError>
```

## Imperative Boundaries

Effects are not imperative control-flow blocks. They do not allow reassignment,
`const`, labels, `flow`, `loop`, `if`, `match`, `return`, `break`, `continue`,
`yield`, `panic`, `await`, or `jump`. Branching belongs in effect-producing
functions or combinators.

Use `await` in imperative code to wait for a `future<T, E>`:

```echo
let $user = await fetch_user($id)
```

`await` returns `T` on success and panics with `E` on failure. It does not
return `outcome<T, E>` and it is unrelated to `join`.

Runtime `task<T>` handles are separate from monadic `future<T, E>` action
values. `defer` creates an unscheduled task handle, `run` schedules a task
handle, and `join` waits for runtime task handles only.

```echo
let $task = defer {
    return fetch_user($id)
}

run $task
let $user = join $task
```

`run { ... }` is shorthand for `run defer { ... }`. Task bodies are imperative
callable-like blocks and use `return`; they do not automatically produce
`future<T, E>`.

## Compiler Model

Effect lowering belongs in the shared compiler pipeline. The semantic analysis
for action unwrapping, short-circuiting, failure unioning, and final result
typing should be shared by file compilation, REPL behavior, future LSP features,
HIR, MIR, and codegen.

The lowering model is efficient imperative control flow:

1. Evaluate the next expression.
2. If it is pure, bind the value directly.
3. If it is action-valued, test for success or short-circuit.
4. Bind the success payload into the local variable.
5. Continue to the next binding or final expression.

No runtime monad framework is required. The bind-style vocabulary is a mental
model for type checking and lowering, not a request to emit callback chains.
