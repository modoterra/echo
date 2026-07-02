# PHP-Compatible Value Assignment And References

Echo follows PHP's ordinary assignment model for rebinding variables: assigning
one variable to another copies the current value binding, so later rebinding the
source variable does not affect the target variable. For example, `$a = 5; $b =
$a; $a = 3;` leaves `$b` as `5`.

PHP-compatible arrays require copy-on-write behavior. `$b = $a` may share the
same backing array internally, but the first write through either variable must
separate the array when the backing value is shared, so `$a[] = 2` after `$b =
$a` must not mutate `$b`. Echo list and structural object values may have their
own Echo-native rules, but PHP `[]` arrays must preserve PHP's observable
copy-on-write semantics.

PHP-compatible objects remain handle-like values. Assigning an object variable
copies the object handle, not the object contents, so property writes through
either variable observe the same object identity unless an explicit clone
operation is used.

PHP references are a separate concept from ordinary assignment and require an
explicit reference-cell model. Syntax such as `$b =& $a`, by-reference
parameters, by-reference returns, `foreach (&$value)`, and array-element
references must alias the same mutable cell where PHP requires aliasing. Echo
must not approximate these forms by pointer-copying ordinary arrays or objects;
the runtime value model needs to distinguish ordinary copy-on-write values from
reference cells.

This decision keeps the common scalar case simple while preserving the PHP
compatibility boundary for arrays, objects, and explicit references. The runtime
may use sharing internally for performance, but observable mutation must match
PHP's value/reference rules.
