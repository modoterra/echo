# Strings

Echo's PHP compatibility mode follows PHP string literal syntax from
https://www.php.net/manual/en/language.types.string.php.

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

String literal parsing belongs in `echo_lexer` and `echo_parser` so diagnostics,
LSP consumers, AST generation, IR generation, and future execution paths share
the same source behavior.
