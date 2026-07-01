# Strings

This note tracks the PHP-compatible string literal surface for Echo. Where
compatibility matters, the lexer and parser should follow PHP's string grammar
from https://www.php.net/manual/en/language.types.string.php.

Supported literal forms:

- Single-quoted strings: `'text'`.
- Double-quoted strings: `"text"`.
- Heredoc strings: `<<<LABEL ... LABEL`.
- Nowdoc strings: `<<<'LABEL' ... LABEL`.

Current compatibility boundary:

- Single-quoted strings only treat `\\` and `\'` as escapes, matching PHP's
  single-quoted string rules.
- Double-quoted strings currently decode the simple escapes already supported by
  Echo: `\n`, `\t`, `\r`, `\"`, and `\\`.
- Heredoc and nowdoc are accepted as static string literals.
- Variable interpolation inside double-quoted strings and heredoc is not yet
  implemented.

```php
<?php
$raw = 'Hello {$name}';
$line = "Hello\n";
$block = <<<'TEXT'
Hello, world
TEXT;
```

This example shows the three forms that already matter most in compatibility
work: raw single-quoted text, double-quoted escape handling, and nowdoc-style
literal blocks. Keep interpolation behavior explicit until the parser supports
it end to end.

String literal parsing belongs in `echo_lexer` and `echo_parser` so diagnostics,
LSP consumers, AST generation, IR generation, and future execution paths share
the same source behavior.
