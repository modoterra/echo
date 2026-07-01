# Canonical Echo Syntax

This reference defines the canonical Echo spellings for source syntax,
generated code, and formatter output. PHP-compatible forms remain legal, but
examples should prefer the Echo-native forms below unless compatibility
requires otherwise.

## File and Module Names

Echo source files use `snake_case.echo` file names, and directories use
lowercase names with underscores when needed. Echo-native package files declare
module names as lowercase `snake_case` segments separated by dots.

```echo
module modoterra.laravel_echo.console

let $command_name = "echo:start"
```

This keeps Echo package structure readable without carrying PHP class-file
naming conventions into Echo source.

Module names must stay lowercase and snake_case.

```echo
module app.http.router
```

These spellings are invalid Echo module declarations:

```echo
module App.Http.Router
module app.http-router
module app.2http.router
module app..router
```

This gives Echo package modules one canonical spelling instead of inheriting
PHP namespace casing rules.

The parser may still parse a non-canonical module declaration so diagnostics can point at the exact segment. The resolver or semantic layer should reject it with a module-name diagnostic rather than treating it as a distinct module identity.

Every Echo file that is meant to be imported by module identity should declare
a `module`. Entry scripts and anonymous one-off scripts may omit the
declaration; they do not need a placeholder such as `module main`.

```echo
use app.http.router

echo router.route($request)
```

This keeps importable package files explicit while preserving lightweight
script files for entrypoints.

## Types and Declarations

Classes, traits, interfaces, enums, and type names use `PascalCase`.
Functions, variables, modules, and file/module identifiers use `snake_case`.
Echo-native exported declarations use `pub`.

```echo
module app.console

pub class StartServerCommand {
    pub fn handle($server_name) {
        echo $server_name
    }
}

fn normalize_path($input) {
    return $input
}
```

This style separates type-like names from value-like names and keeps Echo
examples consistent across packages.

Module exports are explicit. Use Echo declaration forms such as `pub fn` and `pub type`; non-`pub` declarations are module-private.

```echo
module app.support.request_id

pub fn request_id() {
    return "req_123"
}

fn normalize_header($value) {
    return $value
}
```

Other modules may import `request_id`, but not `normalize_header`.

Private-by-default is the rule for modules and classes. Top-level declarations, class fields, methods, factories, and trait members are private unless marked `pub`; interface members are public by definition because interfaces are contracts.

Strict Echo does not support overloads anywhere. Within a declaration scope, a callable, method, factory, facet method, or value member name may have only one definition regardless of signature.

Classes must also be exported explicitly before other modules can import them.

```echo
module app.user

pub class User {
    $name: string

    pub fn name(): string {
        return $this->name
    }
}
```

Class properties use suffix type annotations, matching `let` bindings. Class members are private by default. `pub $field` allows outside code to read and assign that property; a private field may only be accessed by class methods and factories. `pub fn` allows outside code to call the method.

Methods do not declare an explicit receiver parameter. Use `$this` inside class instance methods and factories. Use `$parent` for superclass instance access. Do not use bare `this`, bare `self`, or `Self` in strict Echo. `$static` has no special meaning.

`$this` is not reassignable and has no `mut $this` form. Object field access refers to the object's field storage; assigning `$this->name` updates the field when visibility permits. Binding a field into a local with `let` copies the current field value into fresh local storage.

```echo
let $name = $this->name
```

Later reassignment of `$name` does not update `$this->name`.

Structural object fields follow the same storage rule as class fields. Field access is an assignable place when visibility and mutability allow it; `let` copies the current value into fresh local storage; passing a field to a `mut` parameter aliases that field storage for the call.

```echo
let $user = { name: "Ada" }

$user.name = "Grace"
let $name = $user.name
trim_in_place($user.name)
```

Object literals, class factories, and Echo list literals produce reference values. Binding an object or list value copies the reference value, not the underlying object or list.

```echo
let $copy = $user
```

After this binding, `$copy` and `$user` refer to the same object.

Use `copy <expression>` when a new underlying object or collection graph should be created from the current data.

`copy` is a reserved keyword in strict Echo.

`copy` applies to readable existing storage places, not literals or temporary expression results. The operand does not need to be assignable because `copy` does not mutate the source place. Valid operands include readable variables, field access, and indexed elements. Function calls, factory calls, literals, arithmetic expressions, and string expressions are invalid operands because they already produce fresh temporary values.

```echo
let $source = { name: "Ada" }
let $copy = copy $source

let $name_copy = copy $source.name
let $first_copy = copy $items[0]

let $bad_call = copy get_user()
let $bad_factory = copy User.create("Ada")
let $bad_number = copy 443
let $bad_total = copy ($a + $b)
```

The first three copies are valid. The call, factory, literal, and arithmetic copies are invalid.

```echo
let $copy = copy $user
let $items_copy = copy $items
```

The `copy` expression performs a deep graph copy for Echo reference values. It creates new storage for the copied object or collection graph, recursively copies nested reference values, and preserves internal sharing and cycles within the copied graph.

```echo
let $shared = {1, 2}
let $object = { a: $shared, b: $shared }
let $copy = copy $object
```

In the copy, `$copy.a` and `$copy.b` refer to the same copied list, while neither refers to the original `$shared` list.

Class instances, structural objects, and Echo lists participate in `copy` by default. Values that are not copyable, such as runtime task handles, file handles, processes, sockets, and futures, cannot be copied. This is a compile-time error when the type is statically known and a runtime error otherwise. There is no magic clone hook or implicit clone API for `copy`.

Copying a class instance preserves the instance's full internal state, including private fields, and preserves the instance's dynamic runtime type. If a binding has static type `User` but currently refers to an `AdminUser`, `copy` creates a new `AdminUser` instance. `copy` operates on the object's storage graph; it does not require source-level access to each field.

Copying an `unknown` value is allowed when the operand is an existing storage place. The copy is checked at runtime and fails if the actual value is not copyable.

Copying a nullable value preserves `null` and copies present values according to their normal copy rules. If a `?User` value is `null`, `copy` produces `null`; if it contains a `User` reference, `copy` deep-copies that user graph.

`copy` is legal but redundant for discrete copyable values such as integers and strings. It produces the same value a normal binding copy would produce, but can be useful in generic code.

PHP-compatible arrays keep PHP compatibility semantics. Do not redefine PHP array copy behavior through Echo reference-value or `copy` rules.

Reference identity and strict value equality are distinct. `is same` checks whether two reference values point at the same underlying storage. `==` checks strict structural value equality and must be cycle-aware for copied object or collection graphs.

```echo
let $a = { field: 4 }
let $b = $a
let $c = copy $a
```

Here, `$a is same $b` is true, `$a is same $c` is false, and `$a == $c` is true. Later compiler layers may remove or simplify `copy` when it is applied to a value that cannot carry observable reference identity, such as a discrete integer or string. Copies of reference-value graphs remain source-observable because they create new underlying storage.

`$parent` is a superclass instance receiver, so it uses class instance access with `->`.

```echo
pub class AdminUser extends User {
    pub fn name(): string {
        return $parent->name()
    }
}
```

Strict Echo constructs class instances through explicit factory methods, not `new` or implicit default constructors. A `factory` block contains named construction functions; their bodies use regular Echo statements, and the semantics pass verifies that every required field is initialized before construction completes.

```echo
pub class User {
    $name: string
    $email: string

    factory {
        pub create($name: string, $email: string) {
            $this->name = $name
            $this->email = $email
        }

        pub guest() {
            create("Guest", "")
        }

        from_email($email: string) {
            let $name = $email.before("@")

            $this->name = $name
            $this->email = $email
        }
    }
}
```

Factory methods are private by default. Mark a factory with `pub` to create a public construction path; private factories can be shared by other factories in the same class. Private factories are callable only from factory bodies of that class. A class with no public factories is extension-only and cannot be constructed by normal strict Echo source.

```echo
pub class Token {
    $value: string

    factory {
        pub from_header($header: string) {
            parse($header)
        }

        parse($raw: string) {
            $this->value = $raw.trim()
        }
    }
}
```

Factory methods do not declare a return type. The result type is already known from the containing class. A factory completes by initializing the instance; it may not `return` a value. A factory may use bare `return` for early exit only when all required fields are initialized on every path reaching that return.

A factory may delegate to another factory in the same class. Successful delegation satisfies initialization for that path. Factories are construction-only callables; they may be called from other factories, not from ordinary instance methods.

Factory names are callable names, so they use `snake_case`. `create` is a common convention for the primary factory, but it is not special syntax or special semantics.

Strict Echo class construction calls factory methods by name.

```echo
let $user = User.create("Echo", "echo@example.com")
let $guest = User.guest()
```

Strict Echo uses dot style for type-level factory calls and does not use PHP `::` syntax. PHP-compatible `new User(...)` and `User::create(...)` remain PHP-compatible syntax, but strict Echo classes do not use `new` or `::`.

Strict Echo keeps `extends` and `implements` for class inheritance and interface implementation.

```echo
pub class AdminUser extends User implements Authenticatable {
    pub fn can_manage_users(): bool {
        return true
    }
}
```

Strict Echo does not use an `abstract` keyword. A class with no public factory cannot be constructed; it can only be extended. Concrete construction requires at least one public factory.

```echo
type Record = {
    id: int
}

pub class Repository {
    fn table(): string

    pub fn find($id: int): ?Record {
        return null
    }
}

pub class UserRepository extends Repository {
    fn table(): string {
        return "users"
    }

    factory {
        pub create() {
        }
    }
}
```

Body-less methods in classes are requirements for subclasses. A class with a factory must satisfy every inherited or locally declared body-less method before it is constructable.

Extension-only classes may declare fields. A constructable subclass must initialize all required inherited fields as well as its own required fields.

```echo
pub class Model {
    $id: int

    fn table(): string
}

pub class User extends Model {
    $email: string

    factory {
        pub create($id: int, $email: string) {
            $this->id = $id
            $this->email = $email
        }
    }

    fn table(): string {
        return "users"
    }
}
```

Class fields may declare defaults. Fields without defaults are required and must be initialized by factories; fields with defaults may be omitted or explicitly assigned by a factory. A constructed class instance always has every declared field; strict Echo has no undefined class properties.

```echo
pub class User {
    $name: string
    $active: bool = true

    factory {
        pub create($name: string) {
            $this->name = $name
        }
    }
}
```

Top-level language constants use `const` and may be exported with `pub`. Constants infer their type by default and must be initialized with values that are fully resolved at compile time. Optional type annotations are allowed when the constant should be checked as a specific type, such as a sized integer.

```echo
module app.config

pub const DEFAULT_TIMEOUT = 30
pub const DEFAULT_PORT: uint16 = 443
const INTERNAL_PREFIX = "app:"
```

Top-level constant names use `SCREAMING_SNAKE_CASE`.

Class value members do not use `let` or `const`. A non-`$` value member inside a class is a static immutable class member. `pub` controls visibility only; class value members are never assignable after declaration. Class value members infer their type and must be initialized with values that are fully resolved at compile time. Optional type annotations are allowed when the value should be checked as a specific type.

```echo
pub class UserRole {
    pub ADMIN = "admin"
    DEFAULT_PORT: uint16 = 443
    GUEST = "guest"

    $label: string
}
```

Class value member names use `SCREAMING_SNAKE_CASE`.

Class value members are accessed through the class object.

```echo
echo UserRole.ADMIN
```

Strict Echo does not use static methods. Use factories for construction and module functions for type-related utilities.

Interface members are public by definition, so omit `pub` inside interfaces.

```echo
pub interface Authenticatable {
    fn user_id(): string
}
```

Traits declare reusable class behavior. Strict Echo applies trait behavior with `mixin`, keeping `use` reserved for imports in Echo-native syntax.

```echo
pub trait HasTimestamps {
    pub fn touch() {
        $this->updated_at = time.now()
    }
}

pub class User implements Authenticatable {
    mixin HasTimestamps
}
```

Use `fn` for Echo-native functions and methods. The `function` keyword remains accepted for PHP-compatible source; it declares the same kind of callable as `fn`, but the spelling is semantic metadata that tools and compatibility policy can observe.

```echo
pub fn handle($request: Request): Response {
    return response($request)
}

class StartServerCommand {
    pub fn handle() {
        return 0
    }
}
```

This lets tools distinguish canonical Echo declarations from PHP-compatible declarations without changing callable runtime behavior.

The same rule applies to methods: `pub fn` is canonical Echo spelling, while `public function` remains PHP-compatible spelling for the same method model. Under strict Echo semantics, PHP-compatible declaration spelling such as `public function` is not allowed.

Strict-mode rejection happens after parsing. The AST should still represent PHP-compatible spelling truthfully, and the semantics pass decides whether that spelling is allowed under the file's active semantic policy.

```echo
class UserController {
    pub fn show($request) {
        return response($request)
    }
}
```

Echo-native `fn` parameters use suffix type annotations, matching `let` bindings. PHP-compatible `function` declarations keep PHP-style prefix parameter types.

Function and method return types use `: Type` after the parameter list.

Functions, methods, and factories use explicit `return` for returned values. Do not rely on the last expression in a normal function body. If a function or method omits a return annotation, the return type is inferred.

A bare `return` carries no value and has `void` meaning. It is not `null`, does not satisfy a nullable return type, and must not be treated as an implicit `return null`.

```echo
fn display_name($user: User): string {
    return $user->name()
}

fn inferred_display_name($user: User) {
    return $user->name()
}
```

Function type signatures may include parameter names. Names narrow the signature because they permit named-call checking. An unnamed positional function type is broader and may accept a callable with named parameters, but a named function type requires matching parameter names.

```echo
type Handler = fn(Request): Response
type NamedHandler = fn($request: Request): Response
```

Callable compatibility uses contravariant parameter types and covariant return types. A callable may accept broader parameter types than required, and may return a narrower result type than required.

```echo
type AdminRenderer = fn($user: AdminUser): string

fn render_user($user: User): string {
    return $user.name()
}

let $renderer: AdminRenderer = render_user
```

`render_user` is assignable because every `AdminUser` is also a `User`.

Default parameters participate in callable compatibility. A callable with defaults may satisfy a callable type that provides fewer arguments, but a callable with extra required parameters may not.

```echo
type Greeter = fn($name: string): string

fn greet($name: string, $punctuation: string = "!"): string {
    return "Hello {$name}{$punctuation}"
}

let $greeter: Greeter = greet
```

Variadic parameters follow the same rule: the assigned callable must handle every call the target callable type permits.

```echo
type Logger = fn($message: string): void

fn log($message: string, ...$context: string): void {
    write_log($message, $context)
}

let $logger: Logger = log
```

A fixed-arity callable is not assignable to a variadic callable type unless it can accept the variadic calls promised by that type.

`mut` parameters are part of callable compatibility. A callable that requires a `mut` parameter is not assignable to a non-`mut` callable type because callers of the non-`mut` type may pass non-assignable expressions. A non-`mut` callable may satisfy a `mut` callable type because it can accept the assignable argument and choose not to mutate it.

```echo
type MutFormatter = fn(mut $value: string): string

fn trimmed($value: string): string {
    return $value.trim()
}

let $format: MutFormatter = trimmed
```

`fn(Request): Response` matches `fn($request: Request): Response`, but `fn($request: Request): Response` does not match an arbitrary `fn(Request): Response`.

For multi-parameter named function signatures, names and types must match in order.

```echo
type Handler = fn($request: Request, $ctx: Context): Response
```

Strict Echo has no `mixed` type, no `any` type, no `scalar` type, no `resource` type, and no broad `object` top type. Code should model uncertainty with concrete unions, generics, named structural types, interfaces, or error/result shapes instead of opting out of type information.

Use `unknown` for values from external boundaries that must be narrowed before use. `unknown` is safe because code cannot perform concrete operations on it until a type check, decoder, or pattern match proves a narrower type.

```echo
let $value: unknown = json.decode($body)

if $value is UserPayload {
    save_user($value)
}
```

Functions may declare that they act as type guards by returning `$param is Type`. A guard function still returns a boolean value at runtime, but a true result narrows the named parameter for subsequent control flow.

```echo
fn is_user_payload($value: unknown): $value is UserPayload {
    return $value is UserPayload
}

if is_user_payload($value) {
    save_user($value)
}
```

Guard return types may only target function parameters. Do not allow guard declarations for fields, properties, member paths, or arbitrary expressions.

```echo
fn has_payload($request: Request): $request.body is UserPayload {
    return $request.body is UserPayload
}
```

The `has_payload` signature is invalid because it tries to narrow a field path instead of a parameter.

Guard functions may be generic. Generic guards can narrow through relationships already present in the signature, or through compiler/runtime-supported type checks. Types are not runtime values in Echo source, so callers pass generic type arguments rather than `Type<T>` objects.

```echo
fn is_present<T>($value: ?T): $value is T {
    return $value is not null
}

fn is_type<T>($value: unknown): $value is T {
    return intrinsic.type_matches<T>($value)
}

if is_type<UserPayload>($value) {
    save_user($value)
}
```

Runtime generic type checks such as `is_type<T>` are not ordinary userland comparisons; they require compiler/runtime support.

Use `as` for aliases and pattern bindings, not for general casts. Strict Echo has no `$value as Type` cast operator.

```echo
use app.User as AppUser
FileNotFound as $err => handle_missing($err)

let $user = $value as User
```

The final binding is invalid in strict Echo. Use narrowing, decoders, or an explicit `unsafe` operation instead.

Do not add dedicated `typeof` or `typeid` syntax to strict Echo. Runtime type inspection belongs in standard library or compiler intrinsic APIs where needed; the syntax-level ergonomics are `is`, `is not`, guard return types, and `match`.

Do not use `unknown` as an unchecked cast escape hatch. Narrow it through type guards, `match`, or explicit decoders. Risky unchecked conversion belongs inside an explicit `unsafe` block.

```echo
let $value: unknown = json.decode($body)
let $payload = UserPayload.decode($value)

unsafe {
    let $payload = cast<UserPayload>($value)
}
```

`unsafe` is a strict Echo block for explicitly risky operations. It does not disable normal type checking for the whole block; it only permits operations that are otherwise unavailable, such as unchecked casts, FFI calls, or runtime layout assumptions. Strict Echo has no pointer types and no dereference syntax; if low-level operations are ever needed, add them explicitly inside the unsafe design instead of reserving pointer syntax now.

Primitive types include booleans, text, bytes, bottom markers, sized numeric types, arbitrary-precision integers, and action values.

```echo
let $ok: bool = true
let $name: string = "Ada"
let $payload: bytes = b'hello'
let $count: int = 42
let $offset: int64 = 9_223_372_036_854_775_807
let $limit: uint = 100
let $port: uint16 = 443
let $ratio: float64 = 0.5
let $exact: bigint = 340282366920938463463374607431768211456
```

Use `int`, `uint`, and `float` as the ergonomic default numeric types. `int` is an alias for `int64`; `uint` is an alias for `uint64`; `float` is an alias for `float64`. Use explicit sizes such as `int8`, `int16`, `int32`, `int64`, `uint8`, `uint16`, `uint32`, `uint64`, `float32`, and `float64` when binary layout, FFI, protocol boundaries, storage, or overflow behavior needs a fixed width.

Do not add `double` to strict Echo. `float64` is the explicit 64-bit float spelling, and `float` is its ergonomic alias.

Strict Echo integer arithmetic is checked by default. Compile-time constant overflow is a compile error, and runtime overflow is a runtime error unless code explicitly asks for wrapping, saturating, or checked-result behavior through standard library APIs.

```echo
let $port: uint16 = 70000
let $next = $count + 1
let $wrapped = std.math.wrap_add($count, 1)
```

The `uint16` binding is invalid because the literal does not fit the requested type. The `+` operator performs checked arithmetic for fixed-width integers. The exact standard library spelling for wrapping or saturating arithmetic is deferred.

Use `/` for float division, `//` for integer division, and `%` for remainder.

```echo
let $ratio = 5 / 2
let $whole = 5 // 2
let $left = 5 % 2
```

Strict Echo supports postfix increment and decrement as statements only. They perform checked arithmetic like `+ 1` and `- 1`, and they do not produce expression values.

```echo
$count++
$count--
```

Do not use prefix increment/decrement or use increment/decrement inside expressions.

Compound assignment is statement-only self-referential mutation. It mutates an existing assignment target, evaluates that target once, and uses the same checked/operator semantics as the underlying operator.

```echo
$count += 1
$count -= 1
$count *= 2
$count /= 2
$count //= 2
$count %= 2
$flags &= $mask
$flags |= $mask
$flags ^= $mask
$items[$i] += 1
```

Do not use string-concat compound assignment in strict Echo because strict Echo has no concat operator.

Use `0x`, `0b`, and `0o` prefixes for hexadecimal, binary, and octal integer literals. Decimal literals have no prefix.

```echo
let $mask = 0xff
let $flags = 0b1010_0101
let $mode = 0o755
```

Digit separators may appear between digits, but not at the start, end, or doubled. Legacy leading-zero octal literals are PHP-compatible syntax only, not strict Echo syntax.

```echo
let $bad_mode = 0755
```

The `n` bigint suffix may be used with decimal and base-prefixed integer literals.

```echo
let $big_hex = 0xffff_ffff_ffff_ffffn
let $big_bits = 0b1111_0000n
let $big_mode = 0o755n
```

Big integer math should be supported by an explicit arbitrary-precision type, not by making default `int` unbounded.

```echo
let $count: int = 42
let $exact: bigint = 340282366920938463463374607431768211456
```

`bigint` is a core numeric type so literals and arithmetic can have language-level semantics. Standard library math APIs may provide parsing, formatting, modular arithmetic, and other helpers around the core type.

Plain integer literals infer as `int`. Oversized integer literals are overflow errors unless an explicit expected type makes the literal a `bigint` or the literal uses the `n` bigint suffix.

```echo
let $count = 42
let $too_large = 340282366920938463463374607431768211456
let $exact: bigint = 340282366920938463463374607431768211456
let $also_exact = 340282366920938463463374607431768211456n
```

The second binding is invalid because the literal overflows `int64` and no `bigint` type was requested.

The `n` bigint suffix is the only numeric literal suffix in strict Echo for now. Sized integers and floats use explicit type context instead of suffixes.

```echo
let $port: uint16 = 443
let $ratio: float32 = 0.5
```

Strict Echo allows safe numeric widening and rejects narrowing unless code asks for an explicit conversion. Signed and unsigned mixing requires explicit conversion except for literals proven to fit the target type.

```echo
let $small: int32 = 10
let $large: int64 = $small
let $exact: bigint = $large

let $byte: uint8 = 255
let $count: uint64 = $byte

let $rough: float32 = 0.5
let $precise: float64 = $rough
```

Invalid examples:

```echo
let $large: int64 = 10
let $small: int32 = $large

let $signed: int32 = -1
let $unsigned: uint32 = $signed

let $ratio: float64 = 0.5
let $smaller: float32 = $ratio
```

Literal assignment may use the target type when the literal is exactly representable and in range.

```echo
let $port: uint16 = 443
let $bad_port: uint16 = 70000
```

Integers do not implicitly widen to floats because conversion may lose precision. Numeric literals may be checked in a float context, but runtime integer values require explicit conversion.

```echo
let $ratio: float64 = 1

let $count: int = get_count()
let $bad_ratio: float64 = $count
let $ok_ratio = float64.from_int($count)
```

`bigint` does not implicitly narrow to fixed-width integers. Use an explicit conversion API that can report failure.

```echo
let $n = 10n
let $bad_count: int = $n
let $count = int.try_from($n)
```

Do not add a separate `byte` primitive. A single byte is `uint8`; a sequence of bytes is `bytes`.

```echo
let $tag: uint8 = 0xff
let $payload: bytes = x'ff00a1'
```

`success<T>` and `failure<E>` are primitive wrapper types for action results. `success<T>` carries a successful value `T`; `failure<E>` carries a short-circuit value `E`.

```echo
let $loaded: success<User> = ok $user
let $missing: failure<FindUserError> = fail FindUserError.NotFound
```

`action<T, E>` is the primitive effect-compatible supertype for computations that either produce `success<T>` or short-circuit with `failure<E>`. Concrete action families narrow from it.

```echo
// option<User> is an action<User, void>.
let $maybe_user: option<User> = some $user

// outcome<User, LoadError> is an action<User, LoadError>.
let $loaded: outcome<User, LoadError> = ok $user

// future<User, LoadError> is an action<User, LoadError>.
let $profile: future<User, LoadError> = fetch_profile($user)
```

All action families expose the same effect-binding shape. `option<T>` contains `some<T>` or `none`; `outcome<T, E>` contains `success<T>` or `failure<E>`; `future<T, E>` eventually completes as `success<T>` or `failure<E>`. Inside `effect`, binding an action unwraps the success payload `T`, not the wrapper, and short-circuiting preserves the selected concrete action family.

`option<T>` is the primitive concrete action type for computations that either produce a value `T` or short-circuit with explicit absence. Construct option values with `some <expression>` and `none`.

`some<T>` is the primitive wrapper type for present option values. Conceptually, `option<T>` is the action container for `some<T>|none`, but it remains a primitive action type rather than an ordinary union because `effect` gives it sequencing semantics.

`ok`, `fail`, `some`, and `none` are reserved keywords in strict Echo.

```echo
fn find_cached_user($id: UserId): option<User> {
    if cache.has($id) {
        return some cache.get($id)
    }

    return none
}
```

`none` is not `null` and is not a stored `void` value. `null` is the null value for nullable types such as `?T`; `none` is the absence value for `option<T>`; bare `return` carries no value and has `void` meaning. `option<T>` narrows from `action<T, void>` because its short-circuit channel carries no payload. Do not implicitly convert between `null`, `none`, and bare `void` control flow.

```echo
let $missing_user: option<User> = none
let $empty_name: ?string = null
let $bad_option: option<User> = null
let $bad_nullable: ?User = none
let $maybe_nullable: option<?User> = some null
```

The `some null` value means the option is present and carries `null`; it is distinct from `none`.

Action wrapper values are matchable outside `effect`.

```echo
let $label = match $maybe_user {
    some as $user => $user.name,
    none => "guest"
}
```

Use `some <expression>` and `none` to construct option values. Use `some as pattern` and `none` in patterns. Wrapper names are type and pattern forms, not constructor calls; do not write `some($value)` as an expression.

Omit `as` when a wrapper pattern only needs to test the case and ignore the payload.

```echo
let $label = match $maybe_user {
    some => "present",
    none => "missing"
}
```

`outcome<T, E>` is the primitive concrete action type for computations that either produce `success<T>` or short-circuit with a typed `failure<E>`. Conceptually, it is the action container for `success<T>|failure<E>`, but it remains a primitive action type rather than an ordinary union because `effect` gives it sequencing semantics.

```echo
fn find_user($id: UserId): outcome<User, FindUserError>
```

Effect blocks understand `action` directly: binding an `action<T, E>` unwraps `success<T>` to `T` or short-circuits with `failure<E>`. An `effect` block resolves to a concrete action family, chosen by its postfix type annotation, surrounding expected type, or the first effectful binding when unambiguous.

Construct outcome values with `ok <expression>` and `fail <expression>`.

```echo
fn find_user($id: UserId): outcome<User, FindUserError> {
    if not users.exists($id) {
        return fail FindUserError.NotFound
    }

    return ok users.get($id)
}
```

`ok` and `fail` are expressions that construct `success<T>` and `failure<E>`. When used where an `outcome<T, E>` is expected, `ok $value` supplies the success side and needs context for the failure type; `fail $error` supplies the failure side and needs context for the success type.

```echo
let $ready: outcome<User, FindUserError> = ok $user
let $missing: outcome<User, FindUserError> = fail FindUserError.NotFound
let $ambiguous = ok $user
```

The ambiguous binding is invalid because the failure type cannot be inferred.

Inside an `effect` block, `fail <expression>` may rely on the enclosing expected `outcome<T, E>` type. If no surrounding annotation or return contract supplies the missing success type, the expression is ambiguous.

```echo
fn load_user($id: UserId): outcome<User, LoadError> {
    return effect {
        fail LoadError.NotFound
    }
}
```

Outcome wrapper values are matchable outside `effect`.

```echo
let $label = match $result {
    ok as $user => $user.name,
    fail as $error => $error.message
}
```

Use `ok <expression>` and `fail <expression>` to construct outcome wrapper values. Use `ok as pattern` and `fail as pattern` in patterns. Wrapper names are type and pattern forms, not constructor calls; do not write `success($value)` or `failure($error)` as expressions.

Omit `as` when an outcome wrapper pattern only needs to test the case and ignore the payload.

```echo
let $label = match $result {
    ok => "success",
    fail => "failure"
}
```

`future<T, E>` is the primitive concrete action type for monadic future-like work. It is not tied to the Echo event loop in the language model. An implementation may back future values with runtime tasks, but that is not part of the source-level contract. Future values come from future-targeted `effect` blocks and future-specific standard-library or runtime APIs, not from raw `run`, `ok`, or `fail` syntax.

`future<T, E>` has no direct keyword constructor. Use future-producing APIs or a future-targeted `effect` block.

Future values are opaque to pattern matching. Do not match a `future<T, E>` directly with `success` or `failure` patterns; observe or sequence it through future APIs or `effect`.

Use `await` in imperative code to wait for a `future<T, E>` to finalize. `await future<T, E>` produces `T` on success and panics with `E` on failure. It does not return `outcome<T, E>` and it is unrelated to `join`, which only works on runtime `task<T>` handles.

```echo
let $user = await fetch_user($id)

try {
    let $profile = await fetch_profile($user)
} recover {
    LoadError => echo "could not load profile"
}
```

`await` is valid inside imperative task bodies created by `defer` and `run`. If the awaited future fails and the task body does not recover the panic, the task fails according to normal task panic behavior.

Top-level scripts are imperative execution contexts, so `await` is valid at top level. A failed future panics unless the script recovers it.

Functions do not need an `async` marker to use `await`. `await` is an explicit blocking operation in ordinary imperative code, not syntax for declaring an async function.

`await` is a unary prefix operator. Member access and calls bind tighter than `await`, so use parentheses when accessing the awaited result.

```echo
let $user = await fetch_user($id)
let $name = (await fetch_user($id)).name
```

`null` is a literal value, not a standalone type name. Nullable types use concise `?T` spelling. General unions use `A|B|C` spelling for real type alternatives. Do not write `T|null` in strict Echo.

```echo
let $user: ?User = null
let $id: int|string = "guest"
fn find_user($id: int): ?User {
    return null
}
```

`never` is the bottom type for code paths that do not produce a value.

```echo
fn fail($message: string): never {
    panic AppError { message: $message }
}
```

`void` is the explicit return type for functions that must not return a value. Use it when the absence of a return value is part of the contract and should be checked.

```echo
fn log_user($user: User): void {
    logger.info("User {$user.id}")
}
```

A `void` function may return early with bare `return`, but it may not return an expression. A function returning `?T` must use `return null` when it returns the null value; a bare `return` is still `void`, not `null`.

Preserve author order in union type syntax. The compiler may canonicalize unions internally for type equality, but a formatter should not reorder source unions because order can communicate intent.

```echo
fn parse_user_id($raw: int|string): ?int {
    return null
}

type SaveUserResult = SavedUser|ValidationError|PermissionDenied
```

Generic type arguments use angle brackets with no space before `<` and spaces after commas.

```echo
let $users: list<User> = {}
let $counts: array<string, int> = []
let $result: Result<SavedUser, ValidationError> = save_user($input)
```

Generic type parameters are covariant when they are only produced by the generic type. Use explicit `in` for contravariant parameters that are only consumed. A parameter used in both produced and consumed positions is invariant.

```echo
type Loader<T> = {
    load: fn(): outcome<T, LoadError>
}

type Sink<in T> = {
    send: fn($value: T): void
}

type Cell<T> = {
    get: fn(): T
    set: fn($value: T): void
}
```

`Loader<T>` is covariant in `T` because it only returns values. `Sink<in T>` is contravariant because it only accepts values. `Cell<T>` is invariant because it both returns and accepts `T`. Declaring `in T` and then returning `T` from the type is invalid.

Multiline calls use commas between arguments, but no trailing comma.

```echo
send_email(
    $user.email,
    "Welcome",
    $body
)
```

Named arguments use `name: value`; argument names do not use `$`.

```echo
send_email(
    to: $user.email,
    subject: "Welcome",
    body: $body
)
```

After a named argument appears, following arguments must also be named.

```echo
connect("localhost", port: 5432)
connect(host: "localhost", port: 5432)
```

Variadic parameters and spread calls use `...`. Variadic parameter types follow the variable name.

```echo
fn sum(...$values: int): int {
    return 0
}

sum(...$values)
```

Named variadic packs use the parameter name and an explicit spread marker. The spread expression must be a compatible collection for the variadic item type.

```echo
sum(1, 2, 3)
sum(...$numbers)
sum(values: ...{1, 2, 3})
```

Do not pass an unspread collection to a variadic parameter by name.

```echo
sum(values: {1, 2, 3})
```

Do not mix positional variadic values with a named variadic pack.

```echo
sum(1, values: ...{2, 3})
```

Parameters may declare default values. Required parameters must come before parameters with defaults. Defaults should be compile-time constants or simple literal values in v1.

```echo
fn connect($host: string, $port: uint16 = 5432): Connection {
    return net.connect($host, $port)
}
```

A function may combine defaulted parameters and one variadic parameter. The variadic parameter must be last.

```echo
fn log($level: LogLevel = LogLevel.Info, ...$messages: string): void {
}

log(level: LogLevel.Warning, messages: ...{"disk low", "retrying"})
```

A variadic parameter collects zero or more values into a `list<T>`. When no values are supplied, the collected list is empty.

```echo
log()
log(LogLevel.Info, "started")
log(LogLevel.Info, "started", "finished")
```

Function types may include variadic parameters, but not default values.

```echo
type Logger = fn($level: LogLevel, ...$messages: string): void
```

Closures use `fn`. Arrow closures use `=>`; block closures use explicit `return`.

```echo
let $active = fn ($user: User): bool => $user.active

let $active_block = fn ($user: User): bool {
    return $user.active
}
```

Closures use the same parameter rules as functions, including named parameters, default values, and variadics.

```echo
let $format = fn ($value: int, $base: int = 10): string => int.format($value, $base)
let $sum = fn (...$values: int): int => values.total($values)
```

Function and closure parameters are fresh local bindings by default: the argument value is copied into new parameter storage for the call. Add `mut` before a parameter when the parameter should alias the same storage as the assignable place passed by the caller.

Echo distinguishes binding storage from the value stored in that binding. Non-`mut` binding creates fresh storage and copies the current value into it. If the copied value is discrete, such as an integer or string, the new binding is independent. If the copied value is a reference value, such as an object or Echo list value, the new binding stores a copy of that reference and points at the same underlying object or collection. `mut` does not copy the value; it aliases the original assignable storage.

```echo
let $count = 4
let $count_copy = $count
$count_copy = 5

let $object = { field: 4 }
let $object_copy = $object
$object_copy.field = 5

let $items = {1, 2}
let $items_copy = $items
$items_copy.append(3)
```

After these assignments, `$count` is still `4`, while `$object.field` is `5` because both object bindings hold reference values for the same underlying object. `$items` and `$items_copy` refer to the same Echo list, so the append is visible through both bindings.

```echo
let $normalize = fn (mut $name: string): string {
    $name = $name.trim()

    return $name.lower()
}
```

`mut` is an explicit mutable argument contract. Passing an assignable variable to a `mut` parameter makes the parameter point at the same storage, so reassignment in the callable updates that caller-visible binding. Non-assignable expressions cannot be passed to `mut` parameters.

Call sites do not repeat `mut`. The compiler checks assignability from the callee signature.

```echo
let $name = " Ada "
trim_in_place($name)
```

Any assignable place may be passed to a `mut` parameter, including local variables, fields, and indexed elements. Non-assignable expressions are invalid.

```echo
trim_in_place($user.name)
trim_in_place($names[$index])
trim_in_place(" Ada ")
trim_in_place($first . $last)
```

The first two calls are valid assignment targets. The string literal and concatenation expression are invalid for `mut`.

Closures capture outer bindings by value at call time. Strict Echo does not use PHP-style closure `use` capture lists.

```echo
let $offset = 10
let $add_offset = fn ($value: int): int => $value + $offset
```

When a closure is called, it initializes fresh closure-local captured storage from the current values of the outer bindings it references. A closure body may pass a captured binding to a `mut` parameter, but that mutation applies to the closure's captured storage for that invocation, not to the original outer binding.

```echo
let $name = " Ada "

let $normalize = fn (): void {
    trim_in_place($name)
}

$normalize()
echo $name
```

The call may trim the `$name` captured inside `$normalize`, but it does not reassign the outer `$name`.

Caller-visible mutation requires `mut` through the whole call chain. If any function or closure boundary captures a value without a `mut` parameter contract, mutation stops at that boundary and cannot reassign the original caller binding.

```echo
fn trim_in_place(mut $value: string): void {
    $value = $value.trim()
}

let $normalize = fn (mut $name: string): void {
    trim_in_place($name)
}

let $name = " Ada "
$normalize($name)
```

The outer `$name` is trimmed because both callable boundaries use `mut`.

Functions, methods, factories, receiver methods, closures, and callbacks are different source forms for callables. After resolution, each can be modeled as a callable body with parameter slots, bound receiver or captured values when present, and a return contract.

```echo
fn save($user: User): void {
}

class Repo {
    pub fn save($user: User): void {
    }
}

facet User as $user {
    pub fn label(): string {
        return $user.NAME
    }
}

let $save = fn ($user: User): void {
}
```

Generators use `gen fn`. The return annotation names the yielded item type.

```echo
gen fn connections($server: TcpServer): TcpConnection {
    loop {
        let $conn = net.accept($server)

        if $conn is null {
            break
        }

        yield $conn
    }
}
```

## Collections

Collection delimiters have distinct meanings.

```echo
let $php_array = ["id" => 1, "name" => "Echo"]
let $list: list<int> = {1, 2, 3}
let $user = { id: 1, name: "Echo" }
let $pair = (1, "Echo")
let $rgb: array<int>[3] = [255, 128, 0]
```

Use `[]` for PHP arrays, `{}` for Echo lists, `{ field: value }` for Echo structural objects, `()` for tuples, and fixed-size array types for fixed-size arrays.

PHP-compatible source keeps PHP `[]` syntax and PHP array quirks for compatibility. Under strict Echo semantics, `[]` cannot be used as an associative PHP array; use it only for regular Echo arrays or fixed Echo arrays, and use typed structural objects for record-like data.

```echo
let $payload = ["id" => 1, "name" => "Echo"]
let $user: UserPayload = { id: 1, name: "Echo" }
```

Strict Echo arrays are contiguous, index-addressable arrays, not associative maps. Use `array<T>` for regular arrays, `array<T>[N]` for fixed arrays, and `list<T>` for Echo lists.

```echo
let $ids: array<int> = [1, 2, 3]
let $fixed: array<int>[3] = [1, 2, 3]
let $list: list<int> = {1, 2, 3}
```

Maps, dictionaries, sets, ordered maps, and similar containers are standard library types, not core literal forms. The runtime may provide efficient backing data structures for those stdlib containers without adding dedicated syntax or special semantic lowering.

Import std container types as normal exported symbols from `std.containers`. Use direct imports for one type and grouped imports for several.

```echo
use std.containers.Set
from std.containers use Dict, Map, Set
```

Use explicit commas between literal elements and fields in both single-line and multiline literals. Do not use trailing commas; a trailing comma implies an additional tuple member and is reserved for one-element tuple syntax.

```echo
let $user = {
    name: "Echo",
    email: "echo@example.com"
}

let $ids = {
    1,
    2,
    3
}
```

Type object fields are declarations, not runtime literal fields, so they stay newline-separated without commas.

Parentheses without commas are grouping. A one-element tuple requires a trailing comma.

```echo
let $value = (1)
let $single = (1,)
let $pair = (1, "Echo")
```

An untyped `{}` literal is an empty list. A typed `{}` literal may be an empty structural object only when every field has a default value.

```echo
let $items = {}
let $options: Options = {}
```

The `Options` example is valid only if every field has a default value.

Strict Echo has no omitted, undefined, or optional-present fields in structural objects. A field exists for every value of its type. Use `?T` when the field's value may be null, and use defaults when a field can be omitted from construction syntax while still existing on the object value.

```echo
type Options = {
    timeout: int = 30
    label: ?string
    description: ?string = null
}
```

Here `timeout` and `description` may be omitted from the object literal because they have defaults, but the resulting object still has both fields. `label` is required and may be null.

Object literals may use shorthand for simple variables. The field name is the variable name without `$`.

```echo
let $name = "Echo"
let $email = "echo@example.com"
let $user = { $name, $email }
```

This is equivalent to:

```echo
let $user = { name: $name, email: $email }
```

Only simple variables use shorthand. Expression fields stay explicit.

```echo
let $user = {
    name: $name.trim(),
    id: $request->id,
}
```

Structural objects and class instances use different access syntax. A plain object is a structural value with a type; object fields use dot access. A class instance is a special object form with class identity; instance properties and methods use `->` access to distinguish instance dispatch from structural object access.

```echo
type UserPayload = {
    name: string
}

let $payload: UserPayload = { name: "Echo" }
echo $payload.name

let $user = User.create("Echo")
echo $user->name()
```

Plain structural object fields are public data. If a field exists on the object's type, outside code may read and assign it with dot access. Use a class when private state or controlled mutation is required.

```echo
let $payload: UserPayload = { name: "Echo" }
$payload.name = "Ada"
```

Structural objects are closed over their declared fields. Assignment may update an existing field, but it may not add a new field.

```echo
$payload.email = "echo@example.com"
```

The assignment is invalid unless `email` is declared by the object's type.

Binding keywords apply to the variable binding, not to fields inside the value. `let $payload = ...` declares the binding; `const $payload = ...` prevents rebinding `$payload`. Neither form makes `$payload.name` immutable.

```echo
let $payload: UserPayload = { name: "Echo" }
$payload.name = "Ada"

const $fixed_payload: UserPayload = { name: "Echo" }
$fixed_payload.name = "Ada"
```

Structural type facet blocks declare their receiver alias with `as $alias`. There is no implicit `$self`, and facet methods do not declare an explicit receiver parameter. The alias follows normal `$snake_case` variable naming and is scoped to the facet block's methods.

```echo
facet UserPayload as $payload {
    pub fn display_name(): string {
        return $payload.name
    }

    fn normalized_name(): string {
        return $payload.name.trim()
    }
}
```

Facet methods control visibility individually. Do not write `pub facet`; mark exported receiver methods with `pub`.

Facet blocks only add receiver methods. They cannot add fields, constants, class value members, factories, traits, or nested declarations.

`facet Type as $alias` defines one method surface for the type or object value, not class instances. The receiver alias uses dot access. It does not add `->` instance methods.

```echo
class User {
    NAME = "Hello"
}

facet User as $user {
    pub fn display_label(): string {
        return $user.NAME
    }
}
```

Any module may declare a facet for any type visible to it, including primitives and imported types. Facet methods are admitted into the closed compilation graph. If two admitted facets define the same receiver method for the same target type and method name, compilation fails regardless of signature; graph order is only for deterministic diagnostics, not conflict resolution.

```echo
module app.user

pub type UserPayload = {
    name: string
}

facet UserPayload as $payload {
    pub fn display_name(): string {
        return $payload.name
    }
}
```

```echo
module package_a

facet int as $n {
    pub fn label(): string {
        return "a"
    }
}
```

```echo
module package_b

facet int as $n {
    pub fn label(): string {
        return "b"
    }
}
```

If both modules are admitted into the same compilation graph, the duplicate `int.label()` facet method is a compile error.

Public facet methods from admitted graph units are globally visible for receiver-method lookup. Private facet methods are visible only inside their declaring module. This avoids a separate facet-import syntax while keeping conflicts explicit at compile time.

```echo
facet int as $n {
    pub fn label(): string {
        return "{$n}"
    }

    fn debug_bits(): string {
        return int.to_binary($n)
    }
}
```

Facet methods and class instance methods live in separate method spaces because the call syntax differs. Dot calls use type/object value facet lookup; `->` calls use class instance method lookup.

```echo
$user->name()
User.name()
```

## Enums

Enum names and enum cases use `PascalCase`. Strict Echo uses dot style for enum case access, not PHP `::` syntax. Prefer inference for enum cases, literals, constants, and variables when the type is clear from the initializer or surrounding context.

```echo
pub enum OrderStatus {
    Pending
    Paid
    Cancelled
}

let $status = OrderStatus.Pending
```

Enum cases are variants, not constants, so they do not use `SCREAMING_SNAKE_CASE`.

Backed enums declare their backing type after the enum name and assign values to cases.

```echo
pub enum OrderStatus: string {
    Pending = "pending"
    Paid = "paid"
}
```

Backed enum case values are checked against the enum backing type. Cases do not declare their own type annotations.

```echo
pub enum Status: uint8 {
    Pending = 1
    Paid = 2
}
```

Payload enum cases declare their payload types in parentheses. Generic enum names use normal generic type syntax.

```echo
pub enum Result<T, E> {
    Ok(T)
    Err(E)
}

let $result = Result.Ok($user)
```

Enum bodies declare cases only. Add behavior with `facet`, not methods inside the enum body.

```echo
facet OrderStatus as $status {
    pub fn label(): string {
        return match $status {
            OrderStatus.Pending => "Pending",
            OrderStatus.Paid => "Paid"
        }
    }
}
```

Use `match` as an expression. Match arms use `=>`, are comma-separated, and strict Echo checks matches for exhaustiveness whenever the input type has statically known cases, including enums, `option`, `outcome`, wrapper values, and finite unions.

```echo
let $message = match $result {
    Result.Ok as $user => "Saved {$user.name}",
    Result.Err as $error => $error.message
}
```

Enum case construction and enum case matching use different syntax because they do different things. Construction passes existing values into the enum case constructor, while matching creates new bindings from the matched payload.

```echo
let $result = Result.Ok($user)

let $message = match $result {
    Result.Ok as $saved_user => "Saved {$saved_user.name}",
    Result.Err as $error => $error.message
}
```

An enum case pattern may omit `as` when the arm only needs to test the case and ignore its payload.

```echo
let $label = match $result {
    Result.Ok => "saved",
    Result.Err => "failed"
}
```

When an enum case pattern uses `as`, the payload pattern may be any shared destructuring pattern.

```echo
let $label = match $result {
    Result.Ok as { name: $name } => $name,
    Result.Err as $error => $error.message
}
```

Match arms use commas between arms and no trailing comma. Match arm source order is not semantic; the compiler normalizes arms before checking and lowering. Use `_` as a catch-all arm when the match intentionally ignores remaining cases. `_` may appear anywhere in source, and specific arms after `_` are still meaningful before normalization. If `_` is absent, the compiler should report the missing cases it can prove. Duplicate or conflicting arms are diagnostics after normalization.

The formatter should not alphabetically sort match arms. It may move `_` to the end. The compiler always treats `_` as the final catch-all arm during normalization regardless of source order.

Match arms may include guards with `if` after the pattern. A guarded arm does not make that pattern exhaustive because the guard may be false.

```echo
let $label = match $user {
    User { name: $name } if $name != "" => $name,
    _ => "guest"
}
```

Match patterns use the same destructuring pattern language as `let` and assignment destructuring. Tuple, object, wrapper, and enum-case patterns may nest the same way destructuring patterns nest elsewhere. Pattern bindings are scoped to the arm guard and expression.

```echo
let $label = match $result {
    ok as { name: $name, email: $email } => "{$name} <{$email}>",
    fail as $error => $error.message
}
```

Type patterns narrow and bind values with `Type as $name`. Type patterns may also destructure using the same shared destructuring rules.

```echo
let $label = match $value {
    User as $user => $user.name,
    Team { name: $name } => $name,
    _ => "unknown"
}
```

Use a bare type pattern when the arm only needs to test the type and does not need a bound value.

```echo
let $kind = match $value {
    User => "user",
    Team => "team",
    _ => "unknown"
}
```

Match exhaustiveness is checked against the static input type. A union is exhausted when all of its remaining alternatives are matched. `unknown` is a type and can be matched as a type pattern. `_` matches whatever remains unmatched for the static input type.

Literal patterns are allowed and follow the same strict comparison and type rules as normal Echo values.

```echo
let $label = match $status_code {
    200 => "ok",
    404 => "missing",
    _ => "other"
}
```

Range patterns use inclusive `start..end` syntax. Endpoints must be compile-time constants, and range patterns are limited to ordered integer-like types for now.

```echo
let $category = match $status_code {
    200..299 => "ok",
    400..499 => "client error",
    _ => "other"
}
```

Unguarded overlapping literal and range patterns are compile-time diagnostics after normalization. Guarded arms may overlap because their guards are evaluated in source order among non-`_` arms.

Use `or` for pattern alternatives. Do not use `|` for OR-patterns.

```echo
let $kind = match $method {
    "GET" or "HEAD" => "read",
    "POST" or "PUT" or "PATCH" => "write",
    _ => "other"
}
```

Each alternative in an OR-pattern must bind the same variable names with compatible types. OR-patterns participate in exhaustiveness and overlap checks after normalization.

There are no `and` patterns. Use a guarded arm when a pattern also needs a boolean condition.

```echo
let $label = match $value {
    User as $user if $user.active => $user.name,
    _ => "inactive"
}
```

There are no `not` patterns. Use positive patterns plus `_` for the remaining cases.

```echo
let $presence = match $value {
    null => "missing",
    _ => "present"
}
```

A bare variable pattern matches any remaining value and binds it for the arm guard and expression. It is a catch-all binding pattern, so the compiler normalizes it with other catch-all arms after specific patterns unless it has a guard.

```echo
let $debug = match $value {
    $anything => inspect($anything)
}
```

Multiple unguarded catch-all arms are invalid. Do not combine an unguarded bare variable pattern with an unguarded `_` arm.

Use `_` for wildcard match arms.

```echo
let $label = match $status {
    OrderStatus.Pending => "pending",
    OrderStatus.Paid => "paid",
    _ => "unknown"
}
```

## Control Flow and Operators

Strict Echo control-flow conditions do not use PHP-style parentheses.

```echo
if $count > 0 {
    echo "ok"
} else {
    echo "empty"
}
```

Strict Echo supports `if` with an optional `else`, but does not use `else if` or PHP `elseif`. Use guard clauses for sequential statement conditions and `match` for multi-branch value selection.

```echo
if $user.locked {
    audit.locked_login($user)
    return false
}

if not $user.verified {
    mail.send_verification($user)
    return false
}

return true
```

Use word boolean operators in Echo-native source.

```echo
if not $user.active or $user.locked {
    return false
}

if $user.active and $user.verified {
    return true
}
```

Symbolic boolean operators such as `!`, `&&`, and `||` are PHP-compatible syntax, not strict Echo syntax.

Strict Echo keeps symbolic bitwise operators for numeric and bytes-oriented work. They are not boolean operators.

```echo
let $masked = $flags & $mask
let $combined = $flags | $mask
let $flipped = $flags ^ $mask
let $left = $flags << 2
let $right = $flags >> 2
let $inverse = ~$mask
```

Shift operators are checked. For fixed-width integers, the shift count must be non-negative and less than the bit width. For `bigint`, the shift count must be non-negative and may be rejected by runtime/resource limits.

Strict Echo supports `**` exponentiation with typed semantics. Integer exponentiation uses checked integer arithmetic unless the expected or result type is `bigint`. Float exponentiation uses float semantics. Negative integer exponents require float context.

```echo
let $area = $side ** 2
let $huge = 2n ** 256
let $fraction = 2.0 ** -2
```

Use `is` and `is not` for null, type, and pattern checks.

```echo
if $conn is null {
    break
}

if $value is User {
    echo $value.name
}
```

`is` checks may use generic type arguments when the type is runtime-verifiable.

```echo
if $value is list<UserPayload> {
    process($value)
}
```

If a type cannot be verified from available runtime metadata, semantics should reject the `is` check or require an explicit decoder.

`is not` participates in flow narrowing. This is especially useful for guard clauses.

```echo
if $value is not UserPayload {
    return null
}

save_user($value)
```

For unions, the negative check removes the tested alternative when that is statically knowable.

Strict Echo has no labeled `break` or labeled `continue`. Loop control remains local to the innermost loop.

Explicit jumps use `jump`, not `goto`, and are only valid inside a `flow` block. Labels are scoped to the containing `flow` block, and `jump label` may only target a label in the same block. Labels may appear anywhere inside the `flow` block, including nested ordinary blocks.

```echo
let $result = flow {
    start:
    if is_ready() {
        break "ready"
    }

    prepare()
    jump start
}: string
```

The `flow` block is an expression. It produces values through `break value`, using the same value-exit mechanism as `loop`. A bare `break` exits with `void`, not `null`. `break` always exits the nearest breakable expression, such as the innermost `loop` or `flow`. `continue` is only valid for loops, not for `flow`. `jump label` only transfers control inside the same `flow` block and does not carry a value. Fallthrough to the end of a `flow` block returns `void`.

```echo
let $status = flow {
    retry:
    if fetch_ready() {
        break "ready"
    }

    if should_stop() {
        break "stopped"
    }

    wait()
    jump retry
}: string
```

The optional `: Type` annotation after the block declares the `flow` result type. Without an annotation, the result type is inferred from all `break value` exits plus possible `void` fallthrough.

The `flow` block is a visible boundary for unstructured local control flow. Do not jump into or out of a `flow` block. A `flow` block creates an isolated lexical scope; bindings declared inside it are not visible after the block. Existing outer bindings remain visible inside the block and may be mutated according to their normal mutability rules.

A `jump` may cross ordinary block structure, including jumping into or out of `if` bodies, nested lexical blocks, and loop bodies inside the same `flow`. This is intentionally unstructured; `flow` exists for code that needs complete local control over execution order. Lexical visibility is still textual: jumping into a block does not make that block's local names visible outside their declared lexical scope.

A `jump` must not bypass required local initialization or otherwise violate memory and value safety. Each label has an incoming variable state, and every `jump label` must satisfy the same definitely-initialized local requirements as normal fallthrough to that label. Violations are compile-time errors. The compiler also rejects jumps that would observe moved or invalidated values or cross protected execution boundaries such as nested functions, closures, generators, `recover`, or `ensure`.

Within `flow`, `jump` behaves like explicit local instruction-pointer movement. Statements only take effect when execution reaches them. Jumping over a `defer` is legal; the skipped `defer` is not registered. Already registered defers are not cancelled by later jumps. Jumping out of a lexical scope runs any active defers owned by that scope in normal defer order.

A `jump` may enter or leave a `try` body when the jump is otherwise safe. It may not jump into `recover` or `ensure` bodies, because those regions are entered by panic handling and finalization control flow rather than by ordinary execution. `ensure` runs only when the `try` body's own flow completes through the `try` construct; jumping out of a `try` body skips that `ensure`.

```echo
flow {
    try {
        jump done
    } ensure {
        cleanup()
    }

    done:
    break "ok"
}: string
```

The example returns `"ok"` without running `cleanup()`.

Panic control flow is unchanged inside `flow`. A `panic` reached inside a `try` body enters that `try` construct's `recover` and `ensure` handling normally.

```echo
flow {
    jump show

    let $name = "Ada"

    show:
    echo $name
}
```

The example is invalid because the jump reaches `show` without initializing `$name`.

```echo
let $value: User|Admin|Guest = load()

if $value is not Guest {
    process_member($value)
}
```

Strict Echo uses `==` for strict value equality and `!=` for strict value inequality. There is no relaxed equality operator in strict Echo. Do not use PHP `===`; use `is same` and `is not same` for identity/reference-target checks.

```echo
let $a = {}
let $b = $a
let $c = {}

if $a == $c {
    echo "same value"
}

if $a is same $b {
    echo "same identity"
}

if $a is not same $c {
    echo "different identity"
}
```

The only strict Echo looping construct is `loop`. Use plain `loop` for unconditional loops and `loop ... as ...` forms for iteration.

```echo
loop {
    if $done {
        break
    }

    process_next()
}

loop $users as $user {
    echo $user.name
}

loop $users as ($index, $user) {
    echo "{$index}: {$user.name}"
}
```

`loop` is an expression. `break` exits the loop with `null`, `break <value>` exits with that value, and `continue` restarts the loop. `return` still returns from the enclosing function, method, or factory.

```echo
let $found = loop $users as $user {
    if $user.id == $target_id {
        break $user
    }
}
```

The inferred loop result type comes from `break <value>` expressions plus possible `null`; narrower types may be inferred when control flow proves them. Add a result type after the loop block when the loop expression needs an explicit type.

```echo
let $found = loop $users as $user {
    if $user.id == $target_id {
        break $user
    }
}: ?User
```

`yield` is reserved for generator or iterator-producing behavior, not for returning a loop expression value.

PHP-style `for`, `while`, and `foreach` loops remain PHP-compatible syntax, not strict Echo syntax.

Strict Echo uses `delete` for removing entries from containers that support removal. `delete` returns `true` when an entry was removed and `false` when no entry was removed. Do not use `delete` for variables, structural object fields, or class instance fields; strict fields always exist. Do not add `free` as a source-level memory operation, and do not use PHP `unset` in strict Echo.

```echo
delete $dict[$key]
delete $list[$index]
let $removed = delete $dict[$missing_key]

delete $user.name
free $user
unset($user)
```

The field deletion, `free`, and `unset` examples are invalid in strict Echo. Use lexical scopes for temporary variables, nullable assignment for nullable fields, container APIs or `delete` for removable entries, and explicit close/cleanup APIs for resources.

Deleting from an ordered list or non-fixed Echo array removes the element and shifts following elements left; strict Echo lists and arrays do not have holes. Deleting from fixed arrays is invalid.

```echo
delete $users[2]
delete $items[2]
delete $rgb[1]
```

Deleting an out-of-bounds list or non-fixed array index returns `false`. The fixed-array deletion is invalid when `$rgb` has a fixed array type.

`delete` is only for primitive container structures that the language defines as deletable. Do not add magic deletion hooks, `__delete` methods, facet-based operator overloads, or user-defined delete participation.

Std containers such as `Dict`, `Map`, and `Set` remove entries through methods, not `delete` syntax.

```echo
$dict.remove($key)
$set.remove($value)
```

Strict Echo appends to lists and non-fixed arrays through methods. PHP `$array[] = item` append syntax is PHP-compatible syntax only, not strict Echo syntax for lists or arrays.

```echo
$items.append($item)
```

Append mutates the receiver and returns `void`; do not use append chaining.

## Strings

Strict Echo uses double-quoted strings for interpolation and single-quoted strings for raw text with no interpolation. Do not use PHP `.` concatenation in strict Echo; use interpolation or standard library concat/join helpers.

```echo
let $message = "Saved {$user.name}"
let $label = "{$name} listening on {$port}"
let $template = 'Hello {$name}'
```

Single-quoted strings never interpolate. Braces inside single quotes are literal text.

Interpolation uses braces around normal Echo expressions. The expression must type-check as `string`; strict Echo does not implicitly format non-string values.

```echo
let $message = "Total {($price * $count).as_str()}"
```

Convert values explicitly inside or before interpolation.

```echo
let $message = "Count {$count.as_str()}"
let $hex = "Hex {$count.format(16)}"
let $bad = "Total {$price * $count}"
```

The arithmetic interpolation is invalid because `$price * $count` is not a `string`.

PHP magic constants such as `__DIR__`, `__FILE__`, `__LINE__`, and `__CLASS__` are PHP-compatible syntax, not strict Echo syntax. Strict Echo source/path reflection should use standard library APIs once those APIs are designed.

Strict Echo `string` is Unicode text. It is not exposed as "a UTF-8 string" or "a UTF-32 string" at the type level; UTF-8, UTF-16, and UTF-32 are encodings used when converting text to bytes or decoding bytes into text.

```echo
let $name = "Ada 🔥"
let $bytes = encoding.utf8.encode($name)
let $same_name = encoding.utf8.decode($bytes)
```

Literal prefixes should be reserved for byte-oriented literals, not alternate Unicode string widths. Strict Echo should not add `u""` or `uu""` string spellings unless a future design gives them behavior that is not better expressed through standard library encoding APIs.

Byte literals produce `bytes`, not `string`. Use `b'...'` for raw byte text encoded from the literal source text as UTF-8, and use `x'...'` for exact hexadecimal bytes. Byte literal prefixes only apply to single-quoted literals, so byte literals never interpolate.

```echo
let $line = b'GET /health HTTP/1.1\r\n'
let $nul = b'\x00'
let $template = b'GET {$path} HTTP/1.1\r\n'
let $fire_text = "🔥"
let $fire_bytes = b'🔥'
let $same_fire_bytes = x'f09f94a5'
```

The only escapes allowed inside `b'...'` are byte-oriented escapes: `\\`, `\'`, `\n`, `\r`, `\t`, `\0`, and `\xNN`. Unicode escapes such as `\u{1F525}` are not valid in byte text; write the Unicode text directly when UTF-8 bytes are intended, or use `x'...'` when exact byte values matter.

Hex byte literals contain case-insensitive hex byte pairs. ASCII whitespace may separate pairs for readability, but `_`, `,`, `0x`, comments, and non-ASCII whitespace are invalid inside `x'...'`. After removing ASCII whitespace, the literal must contain an even number of hex digits.

```echo
let $packet = x'ff 00 a1'
let $multiline = x'
    48 54 54 50
    2f 31 2e 31
'
```

Each pair becomes one byte. If already-built text must become bytes, encode or decode the text explicitly through the standard library.

The `b'🔥'` literal is valid and produces the same bytes as `x'f09f94a5'`.

`string` and `bytes` do not implicitly convert to each other. Encode or decode explicitly.

```echo
let $text: string = "Ada"
let $payload: bytes = $text
let $encoded = encoding.utf8.encode($text)
let $decoded = encoding.utf8.decode($encoded)
```

## Effects and Concurrency

`effect {}` is an expression for direct-style `action<T, E>` code. First bindings inside effects still use `let`; the effect-specific behavior is that action values unwrap on success or short-circuit on failure. An effect's result type may be written after the closing brace.

```echo
let $account = effect {
    let $user = find_user($id)
    let $profile = load_profile($user)

    load_account($profile)
}: outcome<Account, LoadError>
```

Effect result inference may start from the final wrapper expression and then narrow from context. A final `ok $value` defaults the block to an `outcome<T, unknown>` shape until an expected type or postfix annotation supplies a more precise failure type. A final `fail $error` defaults to `outcome<unknown, E>` until context supplies the success type. A final `some $value` defaults to `option<T>`, and a final `none` defaults to `option<unknown>` until context supplies `T`.

```echo
let $ready = effect {
    ok $user
}: outcome<User, LoadError>

let $maybe = effect {
    some $user
}: option<User>
```

When an effect block sequences action values with different failure payload types in the same concrete family, the inferred failure type is their union. Handle or narrow the union with `match` after the effect when a caller needs to distinguish cases.

```echo
let $account = effect {
    let $user = load_user($id)
    let $profile = load_profile($id)

    Account.create($user, $profile)
}: outcome<Account, UserError|ProfileError>
```

An effect block uses one concrete action family. Do not implicitly mix `option`, `outcome`, and `future` in the same effect. Convert explicitly at the boundary when a sequence needs to move from one family to another.

```echo
let $user = effect {
    let $id = maybe_user_id($request).ok_or(missing_user_id_error())
    let $user = load_user($id)

    $user
}: outcome<User, LoadError>
```

Ordinary `effect` can target `future<T, E>` when the selected concrete action family is `future`. The postfix annotation or surrounding expected type tells the compiler and runtime to sequence future-producing expressions.

```echo
let $user_task = effect {
    let $id = fetch_user_id($request)
    let $user = fetch_user($id)

    $user
}: future<User, NetworkError|LoadError>
```

Inside a future-targeted effect, pure bindings are immediate local computations and future-valued bindings sequence future completion. A successful future binds its success payload; a failed future short-circuits the resulting future. A pure final expression is lifted into a completed successful future.

The final expression is the implicit effect result. `return` is invalid inside an effect block.

Effects are special expression regions for Echo's monadic and functional model, not imperative control-flow regions. Effect bodies contain zero or more `let` bindings followed by one final result expression. Each non-final line must be a `let` binding; if the right-hand side is effectful, the binding automatically unwraps the successful value or short-circuits failure, and if it is pure, the binding behaves like a normal local helper. The final result expression may be pure or effectful.

Effects do not allow reassignment, `const`, labels, `flow`, `loop`, `if`, `match`, `return`, `break`, `continue`, `yield`, `panic`, `await`, or `jump`. Branching belongs in effect-producing functions or combinators, not as imperative statements inside the effect block.

Strict Echo concurrency uses `defer`, `run`, `fork`, `spawn`, and `join`. `defer` creates an unscheduled `task<T>` runtime handle, `run` schedules a task handle on the Echo event loop, `fork` is for OS-thread-backed work, and `spawn` is for child processes.

The runtime `task<T>` handle is distinct from the monadic `future<T, E>` action type. A task handle represents scheduled or unscheduled executable work for `run` and `join`; `future<T, E>` is an action-family type used by `effect`.

Task bodies are imperative callable-like blocks, not effect blocks. Use `return` inside a task body to complete it. A task body returns whatever its body returns; it does not automatically wrap the result in monadic `future<T, E>`. Use `effect` inside a task body when it wants monadic action sequencing.

```echo
let $task = defer {
    return fetch_user($id)
}

run $task
let $user = join $task
```

```echo
let $task = run defer {
    return fetch_user($id)
}

let $user = join $task
```

`run { ... }` is shorthand for `run defer { ... }`.

`run $task` returns the same task handle after scheduling, so scheduled handles can be assigned and joined.

`join` works on runtime `task<T>` handles. It has no relationship to monadic `future<T, E>` action values.

`run` can also start a group of lightweight tasks. A comma-separated `run` block starts each entry concurrently, preserves result order by source order, and does not use a trailing comma.

```echo
let $tasks = run {
    fetch_user($id),
    fetch_posts($id)
}

let ($user, $posts) = join $tasks
```

```echo
let $worker = fork {
    return resize_image($path)
}

let $image = join $worker
```

`spawn` starts child processes and accepts either a command string or a list of command arguments.

```echo
let $worker = spawn "worker --queue=emails"
let $php = spawn {"php", "--version"}
let $status = join $php
```

## Errors and Recovery

Errors are nominal failure types. `error` is a special case of `type` for values that participate in panic/recovery and result error channels.

```echo
pub error FileNotFound {
    path: string
    message: string = "file not found"
}
```

Error names use `PascalCase`. Error fields follow the same field syntax as structural type fields: newline-separated declarations, nullable values with `?T`, and defaults when useful. Error values do not have omitted or undefined fields.

Construct errors with normal type-object construction. The declared type determines that the constructed value is an error value rather than an ordinary object value.

```echo
let $err = FileNotFound {
    path: "echo.toml"
}
```

Use `panic` to raise a constructed error value or an existing error value.

```echo
panic FileNotFound {
    path: "echo.toml"
}

let $err = FileNotFound {
    path: "echo.toml"
}

panic $err
```

Use `try { ... } recover { ... }` to handle panic flow. Unmatched panics continue propagating, so a wildcard arm is optional.

```echo
let $result = try {
    risky()
} recover {
    FileNotFound as $err => handle_missing($err)
}
```

Recover arms use commas between arms and no trailing comma. Add `_` only when you want to stop the default propagation of otherwise unmatched panics.

```echo
let $result = try {
    risky()
} recover {
    FileNotFound as $err => handle_missing($err),
    _ as $err => fallback($err)
}
```

Use optional `ensure` for cleanup that must run after the try/recover path. `ensure` does not determine the expression value.

```echo
let $result = try {
    open_file($path)
} recover {
    FileNotFound as $err => fallback_file()
} ensure {
    close_handles()
}
```

## Imports

Canonical Echo files order top-level syntax as module declaration, semantics declaration, compile declaration, imports, declarations, then executable statements. `semantics` is file-wide and prelude-only: it appears after `module` and before `compile`, imports, declarations, and executable statements. A perfect format run should keep `use ...` imports and `from ... use ...` imports in separate blocks, and should separate std, relative, and package imports within those forms.

Each file may have at most one `semantics` block. If a future file needs multiple semantic flags, they belong in the same block.

Semantic options are bare flags, one per line.

```echo
semantics {
    strict
}
```

Do not use strings or key/value pairs for mode flags unless a future semantic option actually needs a value.

A `semantics` block after `compile`, imports, declarations, or executable statements is invalid.

```echo
module app.http.router

semantics {
    strict
}

compile {
    "./routes/*.php"
}

use std.http
use app.support.request_id
use illuminate.routing.Router

from std use time
from "./contracts.echo" use Middleware
from illuminate.http use Request, Response

type RouteConfig = {
    method: string
    path: string
}

fn handle($request) {
    return http.responseText("ok")
}
```

This order puts source policy and graph-shaping declarations before name binding, then puts reusable declarations before executable code.

Within each import form, canonical grouping is std imports first, relative path imports second, and package/module imports third. Filesystem paths use quoted strings such as `"./contracts.echo"`; bare module-name syntax uses the module system. Absolute host-path imports are accepted for edge cases but discouraged in package code because they make source less portable.

Direct `use ...` imports are for module and package identities, not filesystem paths. File-path imports use the grouped form.

```echo
from "./support/request_id.echo" use request_id
```

When a file declares a module, prefer importing that module by identity.

```echo
use app.support.request_id
```

Package imports use canonical Echo module identity, not Composer package-name spelling. Package names such as `"vendor/package"` are for acquisition and graph admission, while imports use dot-separated module paths supplied by package metadata.

```echo
use illuminate.console.Command
from illuminate.console use Command, InputOption
```

The final segment disambiguates module imports from exported symbols by canonical naming. Lowercase `snake_case` final segments bind modules, while `PascalCase` or other exported member names bind declarations exported from an Echo module or PHP namespace.

```echo
use std.process
use illuminate.console.Command
```

Here `process` is a module binding and `Command` is an exported symbol binding. PHP namespace imports follow the same exported-symbol rule; `PascalCase\PascalCase` spelling is a namespace plus member path, not an Echo module path.

Prefer direct `use` when importing one symbol.

```echo
use illuminate.console.Command
```

Use grouped `from ... use ...` imports when importing multiple symbols from the same module.

```echo
from illuminate.console use Command, InputOption, OutputStyle
```

Aliases are allowed for symbol conflicts or clearer local names.

```echo
use illuminate.console.Command as LaravelCommand
from illuminate.console use Command, InputOption as Option
```

Whole-module imports bind the module under its final segment, or under an explicit alias.

```echo
use std.process
use std.time as clock

process.run("php", {"--version"})
clock.sleep(100)
```

This keeps single-symbol imports compact while keeping larger imports scannable.

## Compile Declarations

Put `compile` declarations after the optional module or namespace declaration and before executable statements. Prefer one string entry per line.

Each file may have at most one `compile` block. Put every graph admission entry for that file in the same block.

A perfect format run groups `compile` entries by relative paths, absolute paths, then packages, and sorts entries lexicographically within each group. Absolute host paths are accepted but discouraged for portable package code.

```echo
module app.bootstrap

compile {
    "./routes/*.php"
    "./plugins/**/*.php"
    "modoterra/laravel-echo"
}

let $target = "./routes/web.php"
require $target
```

The declaration makes dynamic include targets part of the closed compilation graph before execution.

## Variables and Inference

Echo variables keep PHP's `$` sigil permanently. Use `let` with inference for Echo examples, and omit semicolons unless the documented mode specifically requires PHP syntax.

```echo
let $name = "Echo"
let $port = 8080

echo "{$name} listening on {$port}"
```

This shows the current Echo surface directly instead of mixing in older typed-variable sketches or PHP-only statement style.

Do not remove `$` from local or runtime variables in Echo-native syntax.

Use `let` for first binding and plain assignment for rebinding. Use local `const` when the variable binding must never be reassigned.

```echo
let $count = 0
$count = $count + 1

const $config = load_config()
```

Strict Echo has no uninitialized local variables and no `undefined` value. `let` requires an initializer. If a variable must exist before it has a real value, model that explicitly with a nullable type.

```echo
let $user: ?User = null
```

Local `const` may hold runtime values. It means the variable binding always points at the same value location; it does not require the initializer to be compile-time resolved. Top-level `const` declarations and class value members are the compile-time constant forms.

Local `const` requires an initializer at the declaration site.

```echo
const $config = load_config()
const $port: uint16 = 443
```

Closures may capture local `const` bindings. Captured `const` bindings still cannot be reassigned, but fields inside the captured value may be mutated when the value supports mutation.

```echo
const $payload = { name: "Echo" }

let $rename = fn () {
    $payload.name = "Ada"
}
```

`const` may be used with destructuring. Each produced binding is non-reassignable.

```echo
let ($user, $posts) = join $tasks
const ($user, $posts) = join $tasks
```

`let` destructuring creates normal reassignable bindings; `const` destructuring creates non-reassignable bindings.

Destructuring patterns are separate from declaration keywords. `let pattern = expr` declares reassignable bindings, `const pattern = expr` declares non-reassignable bindings, and bare `pattern = expr` assigns existing targets.

Tuple destructuring requires exact arity.

```echo
let ($user, $posts) = join $tasks
($user, $posts) = refresh()
let ($first, _) = pair()
```

Use `_` in tuple destructuring to ignore a position. `_` does not bind and may appear multiple times. Object destructuring is already partial, so do not use `_` there.

Tuple destructuring does not support defaults.

Object destructuring supports shorthand for same-name bindings and long form for remapping fields to different variable names.

```echo
let { $name, $email } = $user
let { name: $display_name, email: $contact } = $user
const { id: $id } = $user
{ $name, $email } = refresh_user()
```

`{ $name }` is shorthand for `{ name: $name }`.

Destructuring binds or assigns variables only. Do not destructure directly into object fields, class properties, or indexes. In assignment destructuring, every variable target must already be bound. `{ ... }` in destructuring position is a pattern delimiter, not an object literal.

Destructuring patterns may be nested. Leaf targets are still variables only.

```echo
let { user: { $name, $email }, posts: ($first, $second) } = $result
```

Object destructuring is partial by default. Mentioned fields are extracted and unmentioned fields are ignored.

Do not write type annotations inside destructuring patterns. Destructured binding types are inferred from the source expression and pattern shape.

Do not use defaults in destructuring. Optional and nullable values should flow through the destructured type and be handled explicitly after binding.

Under strict Echo semantics, assignment to an unbound variable is invalid; PHP-compatible source may still use assignment-created variables.

Assignment is a statement in strict Echo, not an expression. Do not assign inside conditions, argument lists, or other expressions. Destructuring assignment follows the same rule.

```echo
let $user = find_user($id)

if $user is not null {
    save($user)
}
```

```echo
if $user = find_user($id) {
    save($user)
}
```

The second example is invalid in strict Echo.

Typed `let` bindings always put the type after the variable name.

```echo
let $users: list<User> = {}
let $count: int = 0
let $user: ?User = null
```

Do not write prefix-typed `let` bindings.
