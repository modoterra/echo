# Typed Echo Type System

## Goal

Echo is the true PHP superset: PHP compatibility plus Echo language features.
Typed Echo is a future type-system direction, not a parser mode. It adds a
compiler-native type model that separates arrays, lists, tuples, structural
objects, classes, and receiver extensions so the compiler can reason about
shape and mutation more precisely.

The typed Echo goal is to avoid PHP associative-array ambiguity and give the
compiler strong layout and access guarantees.

## Baseline Echo

Echo:

- Valid PHP stays valid.
- Echo language features are available.
- PHP type declarations are supported.
- Untyped PHP patterns are allowed.
- PHP associative arrays remain valid.
- PHP object/class access remains `->`.
- PHP array access remains `$value["key"]` and `$value[0]` according to PHP semantics.

Future typed Echo:

- Echo language features remain available.
- Stronger semantics are enforced.
- Associative arrays are rejected.
- Explicit array keys are rejected, including numeric keys.
- Arrays, lists, objects, tuples, and classes are separate concepts.
- Dot access is for Echo structural objects and facet receiver members.
- `->` remains for PHP/class member access.
- `[]` is a PHP-compatible array literal and indexed access delimiter.
- `$value[] = item` grows non-fixed arrays only; it is not list append syntax.
- `{}` is a list literal.
- `{ field: value }` is a structural object literal.
- `()` is reserved for tuple values.
- Fixed-size arrays are distinct from dynamic arrays and lists.

## Current Coverage

- `.php`, `.echo`, and `.xo` files currently compile as the same Echo language.
- Echo does not expose parser-mode switches.
- PHP reference assignment, keyed arrays, dynamic calls, and Echo extensions are parsed in the same language.
- The stronger typed model in this document is not currently enforced as a file mode.

## Language Data Structures

Typed Echo separates these families:

```echo
[1, 2, 3]              // array literal
{1, 2, 3}              // list literal
{}                     // empty list by default, or empty object with expected object type
(1, "Echo")            // tuple literal
{id: 1, email: "x"}    // object/record literal
```

This example shows why typed Echo needs separate literal families: the delimiter and keyed/unkeyed shape tell the compiler which value model is intended.

Type families:

```text
list<T>             linked heap-backed dynamic list, written with {}
array<T>            dynamic contiguous zero-indexed array, written with []
array<T>[N]         fixed-size contiguous zero-indexed array, written with []
object              structural object value, written as { field: value }
class T             nominal class declaration; new T is a class instance, not an object
map<K, V>           keyed Echo collection, distinct from PHP arrays
set<T>              unique-value Echo collection
trait<T>            reusable method/contract surface
interface<T>        nominal behavior contract
tuple               always inferred from a direct literal such as (1, 2, 3)
enum T              nominal enum; PHP singleton/backed forms plus Echo payload variants
range<T>            iterable range value such as 1..30
buffer              byte-oriented storage built from string prefixes such as x"...", b"...", bb"..."
```

The type-family table is the vocabulary for diagnostics and hover text; typed Echo should name the collection kind rather than collapsing everything into PHP arrays.

## Arrays

Strict arrays are contiguous indexed sequences. They are not PHP hash maps.

```echo
let $a: array<int> = [1, 2, 3]
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

The expanded meaning is the invariant codegen and diagnostics can rely on when typed Echo accepts an array.

Typed Echo rejects associative arrays and explicit keys:

```echo
let $user = ["id" => 1]
let $bad = [0 => "a", 1 => "b"]
let $bad = [1 => "a"]
```

These examples are rejected because explicit keys would reintroduce PHP hash-map ambiguity into strict arrays.

Use a structural object instead:

```echo
let $user = {
    id: 1
    email: "a@example.com"
}
```

This replacement is the typed Echoling path for named data: fields belong to object types, not array keys.

Array reads use indexes:

```echo
let $first: int = $a[0]
```

Indexed reads are valid because strict arrays preserve contiguous zero-based positions.

Echo's base PHP-compatible language follows PHP square-bracket array element access:
https://www.php.net/manual/en/language.types.array.php

Indexed assignment is replacement only:

```echo
$a[0] = 10
```

This assignment updates an existing slot; it does not change array length.

Append is valid for non-fixed arrays:

```echo
$a[] = 4
```

Append is the only typed Echo growth operation for dynamic arrays.

Reject indexed assignment as growth:

```echo
let $a = [1]

$a[1] = 3  // reject
$a[5] = 9  // reject
```

These assignments would create new slots through indexed replacement syntax, so typed Echo rejects them instead of allowing sparse growth.

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

```echo
let $a: array<int>[3] = [1, 2, 3]
```

This declaration is for fixed-size storage when both element type and length are part of the type.

Inferred element type:

```echo
let $a: array[3] = [1, 2, 3]
```

This form fixes the length while allowing the element type to come from the literal.

Valid:

```echo
let $rgb: array<int>[3] = [255, 128, 0]

$rgb[0] = 0
$rgb[1] = 64
$rgb[2] = 255
```

This is the intended mutation model for fixed arrays: every write targets an existing known slot.

Reject:

```echo
$rgb[] = 255  // fixed-size array cannot append
$rgb[3] = 255 // out of bounds
```

Both operations would change or exceed fixed capacity, so they should be compile-time errors when the index is known.

Dynamic indexes may be allowed with runtime bounds checks, but they are still replacement-only:

```echo
let $i: int = getIndex()
$rgb[$i] = 10
```

This pattern is only safe if lowering preserves a bounds check; even then, it remains replacement rather than growth.

## Dynamic Arrays

```echo
let $ids: array<int> = [1, 2, 3]
let $ids = [1, 2, 3] // inferred array<int>
```

These examples show explicit and inferred dynamic arrays; both represent the same growable contiguous collection kind.

Dynamic arrays are contiguous and can grow by append:

```echo
$ids[] = 4  // ok for dynamic array
$ids[0] = 9 // ok replacement
$ids[3] = 9 // reject if used as growth
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

```echo
let $xs: list<int> = {1, 2, 3}
let $names: list<string> = {"Chris", "Echo"}
```

Brace list literals make linked list construction visually distinct from PHP-compatible arrays.

Empty braces default to an empty list unless expected type context says otherwise:

```echo
let $xs = {}             // infer empty list
let $ids: list<int> = {} // empty list<int>
```

This default keeps `{}` useful for empty collections while still allowing type context to refine the element type.

With expected object context, `{}` can mean an empty object satisfying that type:

```echo
type Options = {
    retries?: int
    timeout?: int
}

let $opts: Options = {} // empty object satisfying Options
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

```echo
let $bad = {
    id: 1
    "loose"
}
```

Mixed literals are rejected because they do not have a single typed Echo value family.

Lists use list-specific receiver functions for mutation. PHP array append syntax
is not list append:

```echo
let $xs: list<int> = {1, 2, 3}

$xs[] = 4 // reject: list is not array
$xs.push(4)
let $last: ?int = $xs.pop()
```

This example shows the intended list API: list mutation goes through receiver functions, not PHP array growth syntax.

Do not use indexed assignment as list growth:

```echo
$xs[3] = 4 // reject when this grows the list
```

Indexed assignment would imply contiguous array behavior, so growing a linked list this way is rejected.

## Tuples

Tuples are fixed positional values.

```echo
let $pair = (1, "Echo") // inferred tuple<int, string>
```

Tuples are for small positional products where both length and element order are part of the type.

Access uses bracket indexes:

```echo
echo $pair[0]
echo $pair[1]
```

Bracket access keeps tuple reads close to array reads while still allowing the compiler to bounds-check literal indexes.

Do not use dot indexes:

```echo
$pair.0 // reject
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

These rules keep tuple support small and statically checkable in the first typed Echo implementation.

## Enums

Echo enums are nominal types. PHP enums are the compatibility floor: pure enums and backed enums should remain compatible, while Echo-native enums may also carry payloads.

Pure enums use singleton cases:

```echo
enum Status {
    Draft
    Published
    Archived
}

let $status = Status::Draft
```

This is equivalent to a PHP pure enum in shape: each case is a singleton value of the enum type, not a string or integer constant.

Backed enums declare a scalar backing type and assign every case a backing value:

```echo
enum Status: string {
    Draft = "draft"
    Published = "published"
    Archived = "archived"
}

let $status = Status::from("draft")
echo $status->value
```

Backed enums preserve PHP's `from`, `tryFrom`, and `value` behavior. The backing type must be `string` or `int` for PHP compatibility.

Echo-native payload enums allow associated data:

```echo
enum Result<T, E> {
    Ok(T)
    Err(E)
}

enum ParseResult {
    Ok(value: AstNode)
    Err(error: ParseError)
}
```

Payload variants are for recoverable results, typed errors, parser states, AST nodes, and protocol states where each case may need different data.

Enums are intended to work with exhaustive `match`:

```echo
match result {
    Ok(value) => compile(value)
    Err(error) => report(error)
}
```

The match form destructures payloads and should be exhaustive when the matched expression has a known enum type.

Backed enums cannot carry payloads:

```echo
enum Bad: string {
    Ok(value: string) = "ok" // reject
}
```

Backing values are scalar identity, while payloads are runtime data; mixing them would make enum identity ambiguous.

Enum declaration shape:

```text
EnumDecl
  name
  generic_params?
  backing_type?: string | int
  cases: EnumCase[]

EnumCase
  name
  backing_value?: scalar
  payload?: positional types | named fields
```

Semantic validation:

```text
If enum has backing_type:
  every case must have a valid backing scalar
  no case may have payload fields
  backing type must be string or int

If enum has payload cases:
  enum is Echo-native algebraic enum
  no scalar backing
  match can destructure payloads

If enum has neither:
  enum is a pure singleton enum
```

This model keeps PHP enum compatibility while letting Echo use algebraic enums for `Option`, `Result`, typed errors, protocol states, parser states, and AST nodes.

## Ranges

Ranges are iterable values inferred from range literals:

```echo
let $ids = 1..30

for $id in $ids {
    echo $id
}
```

The literal creates a `range<int>` value that can be consumed anywhere an iterable is accepted.

Conceptual shape:

```text
Range<T> {
    start: T
    end: T
    step: T
    inclusive: bool
}
```

The range value records the bounds and step without eagerly allocating a list or array.

## Slices

Slices are bounded views over array storage:

```echo
let $items: array<string> = ["draft", "published", "archived"]
let $visible = $items[:2]
```

The slice expression should preserve bounds information and avoid copying when the runtime representation allows a borrowed or view-backed slice.

## Buffers

Buffers are byte-oriented values constructed from prefixed string literals:

```echo
let $signature = x"AABBEE"
let $payload = b"regular bytes"
let $mask = bb"1111_0001"
let $text = u"unicode"
let $wide = uu"Unicode 16"
```

The prefixes describe the literal decoding rule and produce buffer/string-family values without overloading plain strings.

## Objects And Shapes

Strict Echo uses structural objects for named-field data.

Type syntax:

```echo
type User = {
    const id: int
    email: string
    displayName?: string
}
```

This type alias defines named structural data with required, optional, and immutable fields.

Value syntax:

```echo
let $user: User = User {
    id: 1
    email: "a@example.com"
}
```

The constructor shape makes the intended structural type explicit while keeping field order irrelevant.

Field access:

```echo
echo $user.email
```

Dot access is the typed Echo path for structural fields and should power IDE definition and hover on object shapes.

Field mutation:

```echo
$user.email = "b@example.com"
$user.displayName = "Chris"
```

These writes are valid because the fields are declared and mutable after construction.

Reject unknown fields:

```echo
$user.unknown = true
echo $user.unknown
```

Unknown fields are hard errors so structural objects stay closed and typo-resistant.

Objects are mutable by default. A field is immutable only when declared `const`.

```echo
type Person = {
    const id: int
    name: string
    age?: int
}
```

This declaration illustrates the default: `name` and `age` are mutable, while `id` is construction-only.

Valid:

```echo
let $p: Person = Person {
    id: 1
    name: "Chris"
}

$p.name = "Echo"
$p.age = 36
```

The valid writes show mutable required and optional fields being updated after construction.

Reject:

```echo
$p.id = 2        // const field
$p.email = "..." // non-existing field
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

```echo
type Options = {
    const requestId?: string
    retries?: int
}

let $opts: Options = Options {
    requestId: "abc"
}

let $empty: Options = {}
$empty.requestId = "abc" // reject
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

These are rejected as native syntax so typed Echo can define its own type grammar rather than importing docblock pseudo-type names.

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

## Receiver Functions And `facet`

Current preferred keyword: `facet`.

Rationale:

- It can apply multiple times to the same type or alias.
- It defines a method surface for a type without making everything a class.
- It works for aliases such as `type UserList = list<User>`.
- Alternatives like `on`, `impl`, `with`, `attach`, and `methods` were considered but are not chosen.

Syntax:

```php
facet list<T> as $list {
    fn push($value: T): void {
        // implementation
    }

    fn pop(): ?T {
        // implementation
    }
}
```

This is the facet-method shape for collection APIs: the receiver type gains a method surface without converting the value family into a class.

Receiver binding is explicit through `as $alias`:

```php
facet UserList as $users {
    fn active(): UserList {
        return $users.filter(fn ($user: User): bool => $user.active)
    }
}
```

The explicit receiver alias keeps method bodies clear and avoids `$this` for non-class values.

Using `$this` is not allowed in facet blocks:

```php
facet UserList as $users {
    fn active(): UserList {
        return $this.filter(fn ($user: User): bool => $user.active)
    }
}
```

The `$this` example is invalid because `$this` is reserved for class instances.

Rule:

```text
The receiver variable is declared by `as $alias`.
Methods do not declare an explicit receiver parameter.
```

The rule keeps facet methods distinct from class methods while preserving direct method-call syntax at call sites.

## Access Model

Strict Echo separates access operators:

```php
$value.field       // Echo structural object field access
$value.method()    // Echo receiver function from facet block
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

`->` remains PHP/class-oriented. Dot access is Echo member access for structural object fields and receiver functions from `facet` blocks.

Classes continue to use PHP-style `->`:

```php
class User {
    pub fn save(): void {
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

This example is the canonical typed Echo data-transfer shape: construct a structural object, then read named fields through dot access.

## Implementation Plan

1. Add typed Echo parsing support for declarations, array/list/object/tuple literals, structural type aliases, and facet blocks.
2. Add AST nodes for `TypeExpr`, structural fields, array/list/object/tuple literals, field/index access, receiver calls, and facet blocks.
3. Add typed Echo semantic validation.
4. Add type inference for arrays, lists, empty braces, objects, tuples, and contextual object construction.
5. Add object field checking for required fields, optional fields, unknown fields, and const fields.
6. Add facet-block method resolution for dot receiver calls.
7. Lower typed Echo values to compiler/runtime representations.

Typed Echo validation must reject:

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

This rejection list is the minimum typed Echo diagnostic surface for ambiguous PHP array behavior and unsafe structural-object access.

## Acceptance Criteria

These parse in typed Echo:

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

facet list<T> as $list {
    fn pop(): ?T {
        // placeholder
    }
}
```

This acceptance block groups the syntax families that must parse together before typed Echo collection and object support is credible.

These reject in typed Echo:

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

This rejection block proves typed Echo is enforcing the intended safety boundary rather than accepting PHP's dynamic array/object patterns.

Echo still allows PHP associative arrays:

```php
$user = [
    "id" => 1,
    "email" => "a@example.com",
];

echo $user["email"];
```

This example guards compatibility: the same associative array pattern remains valid in Echo's base PHP-compatible language.

Typed Echo rejects the same associative array syntax.

## Notes

- Keep `->` for PHP/class access.
- Use `.` for Echo structural field access and facet receiver calls.
- Use `[]` for array/list indexing and non-fixed array append.
- `list<T>` is distinct from `array<T>` because list is linked and heap-backed.
- `{}` defaults to empty list, but can satisfy an expected empty/optional-field object type.
- Object fields are mutable by default.
- `const` fields are immutable after construction.
- Non-existing fields are hard errors.
- `facet Type as $receiver { ... }` is the current receiver-function design, but the keyword can remain open if a stronger term is discovered.
