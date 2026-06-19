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

CLI overrides:

```sh
xo run --strict file.php  # strict safety on a PHP file
xo run --unsafe file.echo # Echo superset mode on an Echo file
```

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

Type families:

```text
array<T>     dynamic contiguous zero-indexed array
array<T>[N]  fixed-size contiguous zero-indexed array
array[N]     fixed-size array with inferred element type
list<T>      linked heap-backed dynamic list
type T = { } structural object type alias
{ ... }      structural object value when keyed
( ... )      tuple value
```

## Arrays

Strict arrays are contiguous indexed sequences. They are not PHP hash maps.

```php
let array<int> $a = [1, 2, 3];
```

This means:

```text
dynamic contiguous array<int>
indexes: 0, 1, 2
no holes
no explicit keys
no associative behavior
```

Strict mode rejects associative arrays and explicit keys:

```php
let $user = ["id" => 1];
let $bad = [0 => "a", 1 => "b"];
let $bad = [1 => "a"];
```

Use a structural object instead:

```php
let $user = {
    id: 1,
    email: "a@example.com",
};
```

Array reads use indexes:

```php
let int $first = $a[0];
```

Indexed assignment is replacement only:

```php
$a[0] = 10;
```

Append is valid for non-fixed arrays:

```php
$a[] = 4;
```

Reject indexed assignment as growth:

```php
let $a = [1];

$a[1] = 3;  // reject
$a[5] = 9;  // reject
```

Rule:

```text
$array[$i] = value is replacement only.
$array[] = value appends only when $array has non-fixed array type.
$list[] = value is invalid because lists are distinct from arrays.
Indexed assignment never creates a new slot.
Indexed assignment never creates holes.
```

## Fixed-Size Arrays

Explicit element type:

```php
let array<int>[3] $a = [1, 2, 3];
```

Inferred element type:

```php
let array[3] $a = [1, 2, 3];
```

Valid:

```php
let array<int>[3] $rgb = [255, 128, 0];

$rgb[0] = 0;
$rgb[1] = 64;
$rgb[2] = 255;
```

Reject:

```php
$rgb[] = 255;  // fixed-size array cannot append
$rgb[3] = 255; // out of bounds
```

Dynamic indexes may be allowed with runtime bounds checks, but they are still replacement-only:

```php
let int $i = getIndex();
$rgb[$i] = 10;
```

## Dynamic Arrays

```php
let array<int> $ids = [1, 2, 3];
let $ids = [1, 2, 3]; // inferred array<int>
```

Dynamic arrays are contiguous and can grow by append:

```php
$ids[] = 4;  // ok for dynamic array
$ids[0] = 9; // ok replacement
$ids[3] = 9; // reject if used as growth
```

Implementation rule:

```text
array<T> is a contiguous buffer/vector-like storage primitive.
array<T>[N] is a fixed-size contiguous storage primitive.
```

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

List literals use unkeyed brace literals:

```php
let list<int> $xs = {1, 2, 3};
let list<string> $names = {"Chris", "Echo"};
```

Empty braces default to an empty list unless expected type context says otherwise:

```php
let $xs = {};            // infer empty list
let list<int> $ids = {}; // empty list<int>
```

With expected object context, `{}` can mean an empty object satisfying that type:

```php
type Options = {
    retries?: int
    timeout?: int
}

let Options $opts = {}; // empty object satisfying Options
```

Brace literal disambiguation:

```text
{}                          empty list by default
{} with expected object type empty object
{value, value}               list literal
{field: value, field: value} object/record literal
mixed keyed/unkeyed          reject
```

Reject mixed brace literals:

```php
let $bad = {
    id: 1,
    "loose",
};
```

Lists use list-specific receiver functions for mutation. PHP array append syntax
is not list append:

```php
let list<int> $xs = {1, 2, 3};

$xs[] = 4; // reject: list is not array
$xs.push(4);
let ?int $last = $xs.pop();
```

Do not use indexed assignment as list growth:

```php
$xs[3] = 4; // reject when this grows the list
```

## Tuples

Tuples are fixed positional values.

```php
let (int, string) $pair = (1, "Echo");
let $pair = (1, "Echo"); // inferred tuple<int, string>
```

Access uses bracket indexes:

```php
echo $pair[0];
echo $pair[1];
```

Do not use dot indexes:

```php
$pair.0; // reject
```

Tuple rules:

```text
Tuple indexes should be integer literals where possible.
Out-of-bounds tuple indexes are compile errors.
Tuples do not support append.
Tuples do not support named fields.
Tuples do not use dot access.
Tuples are not extendable in v1.
```

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

Value syntax:

```php
let User $user = User {
    id: 1
    email: "a@example.com"
}
```

Field access:

```php
echo $user.email;
```

Field mutation:

```php
$user.email = "b@example.com";
$user.displayName = "Chris";
```

Reject unknown fields:

```php
$user.unknown = true;
echo $user.unknown;
```

Objects are mutable by default. A field is immutable only when declared `const`.

```php
type Person = {
    const id: int
    name: string
    age?: int
}
```

Valid:

```php
let Person $p = Person {
    id: 1
    name: "Chris"
}

$p.name = "Echo";
$p.age = 36;
```

Reject:

```php
$p.id = 2;        // const field
$p.email = "..."; // non-existing field
```

Optional fields are declared fields, not dynamic fields.

```php
age?: int
```

This means:

```text
The field exists in the type.
The field may be absent from the initial object literal.
The field may be assigned later unless it is const.
```

Const optional fields are construction-only:

```php
type Options = {
    const requestId?: string
    retries?: int
}

let Options $opts = Options {
    requestId: "abc"
}

let Options $empty = {};
$empty.requestId = "abc"; // reject
```

## Types

Nullable shorthand:

```php
let ?User $user = null;
```

Equivalent to:

```php
let User|null $user = null;
```

This applies anywhere types are allowed: locals, returns, fields, generics where applicable, and structural object fields.

Local declarations:

```php
let int $count = 0;
let ?User $user = findUser($id);
let $name = "Echo";
```

Local constants:

```php
const int MaxRetries = 3;
const string AppName = "Echo";
```

Type aliases:

```php
type UserId = int;
type UserList = list<User>;
type UserPayload = {
    id: UserId
    email: string
}
```

Do not mirror PHPDoc/Psalm/PHPStan hyphenated pseudo-types as language syntax:

```php
non-empty-string // reject as native type syntax
positive-int     // reject as native type syntax
numeric-string   // reject as native type syntax
class-string<T>  // reject as native type syntax
```

Use aliases for now:

```php
type Email = string;
type Port = int;
```

Future refinement types are possible, but not v1:

```php
type Port = int where $ >= 1 && $ <= 65535;
```

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

Receiver binding is explicit through `as $name`:

```php
extend UserList as $users {
    function active(): UserList {
        // implementation
    }
}
```

Using `$this` is allowed only when explicitly bound:

```php
extend UserList as $this {
    function active(): UserList {
        // $this is the receiver because the block declared it
    }
}
```

Rule:

```text
The receiver variable is declared by `as $name`.
No receiver variable exists unless declared.
```

## Access Model

Strict Echo separates access operators:

```php
$value.field       // Echo structural object field access
$value.method()    // Echo receiver function from extend block
$value->member     // PHP/class property or method access
$value[index]      // array or tuple index access
$value[] = item    // non-fixed array append only
```

Examples:

```php
$user.email;       // Echo structural object field
$users.pop();      // Echo receiver function
$phpUser->save();  // PHP/class method
$array[0];         // array index
$tuple[0];         // tuple index
$array[] = 4;      // ok when $array is a non-fixed array
```

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

## Acceptance Criteria

These parse in strict mode:

```php
let array<int>[3] $a = [1, 2, 3];
let array[3] $b = [1, 2, 3];
let array<int> $c = [1, 2, 3];
let list<int> $xs = {1, 2, 3};
let list<int> $empty = {};
let (int, string) $pair = (1, "Echo");

type User = {
    const id: int
    email: string
    displayName?: string
}

let User $user = User {
    id: 1
    email: "a@example.com"
}

extend list<T> as $list {
    function pop(): ?T {
        // placeholder
    }
}
```

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

Echo mode still allows PHP associative arrays:

```php
$user = [
    "id" => 1,
    "email" => "a@example.com",
];

echo $user["email"];
```

Strict mode rejects the same associative array syntax.

## Notes

- Keep `->` for PHP/class access.
- Use `.` for Echo structural field access and extension receiver calls.
- Use `[]` for array append and array/tuple indexing.
- `list<T>` is distinct from `array<T>` because list is linked and heap-backed.
- `{}` defaults to empty list, but can satisfy an expected empty/optional-field object type.
- Object fields are mutable by default.
- `const` fields are immutable after construction.
- Non-existing fields are hard errors.
- `extend Type as $receiver { ... }` is the current receiver-function design, but the keyword can remain open if a stronger term is discovered.
