# PHP Arithmetic

Source: https://www.php.net/manual/en/language.operators.arithmetic.php

Echo arithmetic should follow PHP compatibility semantics in the base language.
The PHP arithmetic operator set includes unary identity and negation, binary
addition, subtraction, multiplication, division, modulo, and exponentiation.

Current implementation status:

- Integer and floating-point numeric literals are parsed and lowered.
- Unary `+` and `-` are parsed and lowered.
- Binary `+`, `-`, `*`, `/`, `%`, and `**` are parsed and lowered.
- Arithmetic operands are coerced through a shared numeric-context helper for
  `null`, booleans, integers, floats, and numeric strings.
- Division follows PHP's integer-result rule when two integer operands divide
  evenly; otherwise it returns a float.
- Modulo converts operands to integers before processing and preserves the sign
  of the dividend.
- Exponentiation is right-associative and binds more tightly than unary signs.
- PHP arrays support PHP `+` union semantics: the result keeps the left array's
  keys and only takes right-side keys that do not exist on the left.
- Non-numeric strings, objects, resources, and unsupported numeric cases return
  an explicit runtime error value until PHP warning/error behavior is modeled
  more precisely.

Implementation direction:

- Parser precedence should match PHP operator precedence, with `+` and `-` at
  the same left-associative precedence level.
- Codegen should lower arithmetic operators through core runtime ABI symbols,
  not through REPL-specific evaluation.
- Unsupported arithmetic cases should remain explicit errors until the matching
  PHP behavior is implemented.
