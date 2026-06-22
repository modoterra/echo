# Strict-Mode Type System

## Goal

Echo mode is the true PHP superset: PHP compatibility plus Echo language features. Strict mode is Echo-only safety mode: it keeps Echo features on, but rejects unsafe or ambiguous PHP compatibility patterns. Strict mode adds a cleaner compiler-native type model that separates arrays, lists, tuples, structural objects, classes, and receiver extensions.

The strict-mode goal is to avoid PHP associative-array ambiguity and give the compiler strong layout and access guarantees.

## Modes

Echo mode:

- Valid PHP stays valid.
- Echo language features are available.
- PHP type declarations are supported.
- Untyped PHP patterns are allowed.
- PHP associative arrays remain valid.
- PHP object/class access remains `->`.
- PHP array access remains `$value["key"]` and `$value[0]` according to PHP semantics.

Strict mode:

- Echo language features remain available.
- Stronger semantics are enforced.
- Associative arrays are rejected.
- Explicit array keys are rejected, including numeric keys.
- Arrays, lists, objects, tuples, and classes are separate concepts.
- Dot access is for Echo structural objects and extension receiver members.
- `->` remains for PHP/class member access.
- `[]` is a PHP-compatible array literal and indexed access delimiter.
- `$value[] = item` grows non-fixed arrays only; it is not list append syntax.
- `{}` is a list literal.
- `{ field: value }` is a structural object literal.
- `()` is reserved for tuple values.
- Fixed-size arrays are distinct from dynamic arrays and lists.

File extension defaults:

```text
.php          Echo mode by default
.echo/.xo     Strict mode by default
```

This table is the user-facing default: PHP files preserve compatibility first, while Echo source files opt into strict safety unless overridden.

CLI overrides:

```sh
xo run --strict file.php  # strict safety on a PHP file
xo run --unsafe file.echo # Echo superset mode on an Echo file
```

These commands let a project test strict diagnostics on PHP input or temporarily run Echo source with PHP-compatible unsafe patterns enabled.

`--unsafe` means unsafe PHP compatibility patterns are allowed. It does not disable Echo language features.

## Current Coverage

- `.echo` and `.xo` files default to strict mode; `.php` files default to Echo superset mode.
- `xo` supports `--strict` and `--unsafe` mode overrides.
- Strict mode currently rejects PHP reference assignment (`$b =& $a`) as an unsafe PHP compatibility pattern.
- Strict mode rejects dynamic function-call statements (`$fn()`) as unsafe dynamic dispatch.
- Strict mode rejects user `namespace std ...` declarations; only trusted packaged stdlib source may declare std modules. PHP namespaces such as `namespace std\Net` remain valid.
- Echo superset mode still accepts PHP reference assignment for compatibility.

## Value Families

Strict Echo separates these families:

```php
[1, 2, 3]              // array literal
{1, 2, 3}              // list literal
{}                     // empty list by default, or empty object with expected object type
(1, "Echo")            // tuple literal
{id: 1, email: "x"}    // object/record literal
```

This example shows why strict mode needs separate literal families: the delimiter and keyed/unkeyed shape tell the compiler which value model is intended.

Type families:

```text
list<T>       linked heap-backed dynamic list, written with {}
array<T>      dynamic contiguous zero-indexed array, written with []
array<T>[N]   fixed-size contiguous zero-indexed array, written with []
map<K, V>     keyed Echo collection, distinct from PHP arrays
set<T>        unique-value Echo collection
type T = { }  structural object type alias
{ ... }       structural object value when keyed
class T       nominal class declaration; new T is a class instance, not an object
trait<T>      reusable method/contract surface
interface<T>  nominal behavior contract
( ... )       tuple literal, always inferred from direct literal instantiation
enum T        nominal enum; PHP singleton/backed forms plus Echo payload variants
range<T>      iterable range value such as 1..30
buffer        byte-oriented storage built from string prefixes such as x"...", b"...", bb"..."
```

The type-family table is the vocabulary for diagnostics and hover text; strict mode should name the collection kind rather than collapsing everything into PHP arrays.

## Arrays

Strict arrays are contiguous indexed sequences. They are not PHP hash maps.

```php
let $a: array<int> = [1, 2, 3];
```

This is the strict dynamic-array shape: a contiguous integer-indexed sequence with a known element type.

This means:

```text
dynamic contiguous array<int>
indexes: 0, 1, 2
no holes
no explicit keys
no associative behavior
```

The expanded meaning is the invariant codegen and diagnostics can rely on when strict mode accepts an array.

Strict mode rejects associative arrays and explicit keys:

```php
let $user = ["id" => 1];
let $bad = [0 => "a", 1 => "b"];
let $bad = [1 => "a"];
```

These examples are rejected because explicit keys would reintroduce PHP hash-map ambiguity into strict arrays.

Use a structural object instead:

```php
let $user = {
    id: 1,
    email: "a@example.com",
};
```

This replacement is the strict-mode modeling path for named data: fields belong to object types, not array keys.

Array reads use indexes:

```php
let int $first = $a[0];
```

Indexed reads are valid because strict arrays preserve contiguous zero-based positions.

Echo superset mode follows PHP square-bracket array element access:
https://www.php.net/manual/en/language.types.array.php

Indexed assignment is replacement only:

```php
$a[0] = 10;
```

This assignment updates an existing slot; it does not change array length.

Append is valid for non-fixed arrays:

```php
$a[] = 4;
```

Append is the only strict-mode growth operation for dynamic arrays.

Reject indexed assignment as growth:

```php
let $a = [1];

$a[1] = 3;  // reject
$a[5] = 9;  // reject
```

These assignments would create new slots through indexed replacement syntax, so strict mode rejects them instead of allowing sparse growth.

Rule:

```text
$array[$i] = value is replacement only.
$array[] = value appends only when $array has non-fixed array type.
$list[] = value is invalid because lists are distinct from arrays.
Indexed assignment never creates a new slot.
Indexed assignment never creates holes.
```

This rule gives both diagnostics and runtime lowering one simple distinction: replacement uses an index, growth uses append.

## Fixed-Size Arrays

Explicit element type:

```php
let $a: array<int>[3] = [1, 2, 3];
```

This declaration is for fixed-size storage when both element type and length are part of the type.

Inferred element type:

```php
let $a: array[3] = [1, 2, 3];
```

This form fixes the length while allowing the element type to come from the literal.

Valid:

```php
let $rgb: array<int>[3] = [255, 128, 0];

$rgb[0] = 0;
$rgb[1] = 64;
$rgb[2] = 255;
```

This is the intended mutation model for fixed arrays: every write targets an existing known slot.

Reject:

```php
$rgb[] = 255;  // fixed-size array cannot append
$rgb[3] = 255; // out of bounds
```

Both operations would change or exceed fixed capacity, so they should be compile-time errors when the index is known.

Dynamic indexes may be allowed with runtime bounds checks, but they are still replacement-only:

```php
let int $i = getIndex();
$rgb[$i] = 10;
```

This pattern is only safe if lowering preserves a bounds check; even then, it remains replacement rather than growth.

## Dynamic Arrays

```php
let array<int> $ids = [1, 2, 3];
let $ids = [1, 2, 3]; // inferred array<int>
```

These examples show explicit and inferred dynamic arrays; both represent the same growable contiguous collection kind.

Dynamic arrays are contiguous and can grow by append:

```php
$ids[] = 4;  // ok for dynamic array
$ids[0] = 9; // ok replacement
$ids[3] = 9; // reject if used as growth
```

This block demonstrates the strict distinction between appending and replacing: only `$ids[]` can grow the dynamic array.

Implementation rule:

```text
array<T> is a contiguous buffer/vector-like storage primitive.
array<T>[N] is a fixed-size contiguous storage primitive.
```

The storage rule keeps arrays separate from lists at the runtime representation level.

## Lists

`list<T>` is not an alias for `array<T>`.

A list is:

```text
heap-backed
linked
not fixed-size
not contiguous memory
not array storage
```

These properties explain why lists do not reuse array append or fixed-array indexing semantics.

List literals use unkeyed brace literals:

```php
let $xs: list<int> = {1, 2, 3};
let $names: list<string> = {"Chris", "Echo"};
```

Brace list literals make linked list construction visually distinct from PHP-compatible arrays.

Empty braces default to an empty list unless expected type context says otherwise:

```php
let $xs = {};            // infer empty list
let $ids: list<int> = {}; // empty list<int>
```

This default keeps `{}` useful for empty collections while still allowing type context to refine the element type.

With expected object context, `{}` can mean an empty object satisfying that type:

```php
type Options = {
    retries?: int
    timeout?: int
}

let $opts: Options = {}; // empty object satisfying Options
```

This contextual form lets optional-field objects be constructed without inventing a separate empty-object token.

Brace literal disambiguation:

```text
{}                          empty list by default
{} with expected object type empty object
{value, value}               list literal
{field: value, field: value} object/record literal
mixed keyed/unkeyed          reject
```

The disambiguation table is the parser and diagnostics rule for deciding whether braces mean list or object.

Reject mixed brace literals:

```php
let $bad = {
    id: 1,
    "loose",
};
```

Mixed literals are rejected because they do not have a single strict-mode value family.

Lists use list-specific receiver functions for mutation. PHP array append syntax
is not list append:

```php
let $xs: list<int> = {1, 2, 3};

$xs[] = 4; // reject: list is not array
$xs.push(4);
let ?int $last = $xs.pop();
```

This example shows the intended list API: list mutation goes through receiver functions, not PHP array growth syntax.

Do not use indexed assignment as list growth:

```php
$xs[3] = 4; // reject when this grows the list
```

Indexed assignment would imply contiguous array behavior, so growing a linked list this way is rejected.

## Tuples

Tuples are fixed positional values.

```php
let (int, string) $pair = (1, "Echo");
let $pair = (1, "Echo"); // inferred tuple<int, string>
```

Tuples are for small positional products where both length and element order are part of the type.

Access uses bracket indexes:

```php
echo $pair[0];
echo $pair[1];
```

Bracket access keeps tuple reads close to array reads while still allowing the compiler to bounds-check literal indexes.

Do not use dot indexes:

```php
$pair.0; // reject
```

Dot access is reserved for named fields and receiver members, so tuple positions should not use it.

Tuple rules:

```text
Tuple indexes should be integer literals where possible.
Out-of-bounds tuple indexes are compile errors.
Tuples do not support append.
Tuples do not support named fields.
Tuples do not use dot access.
Tuples are not extendable in v1.
```

These rules keep tuple support small and statically checkable in the first strict-mode implementation.

## Objects And Shapes

Strict Echo uses structural objects for named-field data.

Type syntax:

```php
type User = {
    const id: int
    email: string
    displayName?: string
}
```

This type alias defines named structural data with required, optional, and immutable fields.

Value syntax:

```php
let User $user = User {
    id: 1
    email: "a@example.com"
}
```

The constructor shape makes the intended structural type explicit while keeping field order irrelevant.

Field access:

```php
echo $user.email;
```

Dot access is the strict-mode path for structural fields and should power IDE definition and hover on object shapes.

Field mutation:

```php
$user.email = "b@example.com";
$user.displayName = "Chris";
```

These writes are valid because the fields are declared and mutable after construction.

Reject unknown fields:

```php
$user.unknown = true;
echo $user.unknown;
```

Unknown fields are hard errors so structural objects stay closed and typo-resistant.

Objects are mutable by default. A field is immutable only when declared `const`.

```php
type Person = {
    const id: int
    name: string
    age?: int
}
```

This declaration illustrates the default: `name` and `age` are mutable, while `id` is construction-only.

Valid:

```php
let Person $p = Person {
    id: 1
    name: "Chris"
}

$p.name = "Echo";
$p.age = 36;
```

The valid writes show mutable required and optional fields being updated after construction.

Reject:

```php
$p.id = 2;        // const field
$p.email = "..."; // non-existing field
```

These rejections cover the two main object safety checks: const fields cannot be reassigned and undeclared fields cannot appear.

Optional fields are declared fields, not dynamic fields.

```php
age?: int
```

The shorthand marks a known field as possibly absent; it does not permit arbitrary dynamic properties.

This means:

```text
The field exists in the type.
The field may be absent from the initial object literal.
The field may be assigned later unless it is const.
```

The meaning block is the semantic contract for construction checking and later field assignment.

Const optional fields are construction-only:

```php
type Options = {
    const requestId?: string
    retries?: int
}

let $opts: Options = Options {
    requestId: "abc"
}

let $empty: Options = {};
$empty.requestId = "abc"; // reject
```

This example shows that optionality controls presence, while `const` still controls whether later assignment is allowed.

## Types

Nullable shorthand:

```php
let ?User $user = null;
```

Nullable shorthand keeps common optional values concise in local declarations, returns, fields, and generics.

Equivalent to:

```php
let User|null $user = null;
```

The expanded form is the underlying union type the compiler should expose in type facts.

This applies anywhere types are allowed: locals, returns, fields, generics where applicable, and structural object fields.

Local declarations:

```php
let int $count = 0;
let ?User $user = findUser($id);
let $name = "Echo";
```

This block shows strict locals can be explicitly typed, nullable, or inferred depending on how much information the code needs to state.

Local constants:

```php
const int MaxRetries = 3;
const string AppName = "Echo";
```

Local constants are for values that should not be reassigned after declaration.

Type aliases:

```php
type UserId = int;
type UserList = list<User>;
type UserPayload = {
    id: UserId
    email: string
}
```

Aliases let programs name primitive, collection, and structural shapes so signatures and field declarations stay readable.

Do not mirror PHPDoc/Psalm/PHPStan hyphenated pseudo-types as language syntax:

```php
non-empty-string // reject as native type syntax
positive-int     // reject as native type syntax
numeric-string   // reject as native type syntax
class-string<T>  // reject as native type syntax
```

These are rejected as native syntax so strict mode can define its own type grammar rather than importing docblock pseudo-type names.

Use aliases for now:

```php
type Email = string;
type Port = int;
```

Aliases provide a stable naming layer today without promising refinement semantics.

Future refinement types are possible, but not v1:

```php
type Port = int where $ >= 1 && $ <= 65535;
```

This sketch shows the likely future direction while keeping v1 focused on simpler aliases and structural types.

## Receiver Functions And `extend`

Current preferred keyword: `extend`.

Rationale:

- It can apply multiple times to the same type or alias.
- It extends the surface of a type without making everything a class.
- It works for aliases such as `type UserList = list<User>`.
- Alternatives like `on`, `impl`, `with`, `attach`, and `methods` were considered but are not chosen.

Syntax:

```php
extend list<T> as $list {
    function push(T $value): void {
        // implementation
    }

    function pop(): ?T {
        // implementation
    }
}
```

This is the extension-method shape for collection APIs: the receiver type is extended without converting the value family into a class.

Receiver binding is explicit through `as $name`:

```php
extend UserList as $users {
    function active(): UserList {
        // implementation
    }
}
```

The explicit receiver name makes method bodies clear and avoids implicit `$this` for non-class values.

Using `$this` is allowed only when explicitly bound:

```php
extend UserList as $this {
    function active(): UserList {
        // $this is the receiver because the block declared it
    }
}
```

This form supports familiar method-body spelling while still requiring the receiver to be declared.

Rule:

```text
The receiver variable is declared by `as $name`.
No receiver variable exists unless declared.
```

The rule keeps extension methods lexically explicit and prevents hidden receiver bindings.

## Access Model

Strict Echo separates access operators:

```php
$value.field       // Echo structural object field access
$value.method()    // Echo receiver function from extend block
$value->member     // PHP/class property or method access
$value[index]      // array, fixed-size array, or list index access
$value[] = item    // non-fixed array append only
```

This table is the operator contract: each access form maps to one value family instead of overloading PHP arrays and objects for everything.

Examples:

```php
$user.email;       // Echo structural object field
$users.pop();      // Echo receiver function
$phpUser->save();  // PHP/class method
$array[0];         // array index
$tuple[0];         // tuple index
$array[] = 4;      // ok when $array is a non-fixed array
```

The examples show how the access contract reads in real code and where PHP/class access remains separate from Echo structural access.

`->` remains PHP/class-oriented. Dot access is Echo member access for structural object fields and receiver functions from `extend` blocks.

Classes continue to use PHP-style `->`:

```php
class User {
    function save(): void {
        // ...
    }
}

let User $user = new User();
$user->save();
```

Classes keep PHP-style member access for compatibility and to avoid confusing class methods with structural-object fields.

Structural objects use dot:

```php
type UserPayload = {
    id: int
    email: string
}

let UserPayload $payload = UserPayload {
    id: 1
    email: "a@example.com"
}

echo $payload.email;
```

This example is the canonical strict-mode data-transfer shape: construct a structural object, then read named fields through dot access.

## Implementation Plan

1. Add strict-mode parsing support for declarations, array/list/object/tuple literals, structural type aliases, and extend blocks.
2. Add AST nodes for `TypeExpr`, structural fields, array/list/object/tuple literals, field/index access, receiver calls, and extend blocks.
3. Add strict-mode semantic validation.
4. Add type inference for arrays, lists, empty braces, objects, tuples, and contextual object construction.
5. Add object field checking for required fields, optional fields, unknown fields, and const fields.
6. Add extend-block method resolution for dot receiver calls.
7. Lower strict-mode values to compiler/runtime representations.

Strict-mode validation must reject:

```php
["id" => 1]
[0 => "a"]
[1 => "a"]
$a[1] = 3       // when this grows an array
$a[5] = 9       // sparse/growth assignment
$fixed[] = 4
$fixed[3] = 4
$obj.unknown
$obj.constField = x
```

This rejection list is the minimum strict-mode diagnostic surface for ambiguous PHP array behavior and unsafe structural-object access.

## Acceptance Criteria

These parse in strict mode:

```php
let $a: array<int>[3] = [1, 2, 3];
let $b: array[3] = [1, 2, 3];
let $c: array<int> = [1, 2, 3];
let $xs: list<int> = {1, 2, 3};
let $empty: list<int> = {};
let $pair = (1, "Echo");

type User = {
    const id: int
    email: string
    displayName?: string
}

let $user: User = User {
    id: 1
    email: "a@example.com"
}

extend list<T> as $list {
    function pop(): ?T {
        // placeholder
    }
}
```

This acceptance block groups the syntax families that must parse together before strict-mode collection and object support is credible.

These reject in strict mode:

```php
let $bad = ["id" => 1];
let $bad = [0 => "a"];
let $bad = [1 => "a"];

let $bad = {
    id: 1,
    "loose",
};

let $a = [1];
$a[1] = 3;
$a[5] = 9;

let array<int>[1] $fixed = [1];
$fixed[] = 2;
$fixed[1] = 2;

type User = {
    const id: int
    email: string
}

let User $user = User {
    id: 1
    email: "a@example.com"
}

$user.id = 2;
$user.unknown = true;
```

This rejection block proves strict mode is enforcing the intended safety boundary rather than accepting PHP's dynamic array/object patterns.

Echo mode still allows PHP associative arrays:

```php
$user = [
    "id" => 1,
    "email" => "a@example.com",
];

echo $user["email"];
```

This example guards compatibility: the same associative array pattern remains valid when a file is intentionally using Echo superset mode.

Strict mode rejects the same associative array syntax.

## Notes

- Keep `->` for PHP/class access.
- Use `.` for Echo structural field access and extension receiver calls.
- Use `[]` for array/list indexing and non-fixed array append.
- `list<T>` is distinct from `array<T>` because list is linked and heap-backed.
- `{}` defaults to empty list, but can satisfy an expected empty/optional-field object type.
- Object fields are mutable by default.
- `const` fields are immutable after construction.
- Non-existing fields are hard errors.
- `extend Type as $receiver { ... }` is the current receiver-function design, but the keyword can remain open if a stronger term is discovered.
