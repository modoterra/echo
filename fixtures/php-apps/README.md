# PHP Application Fixtures

This directory is for local, full-application PHP fixtures such as a fresh
Laravel install, Symfony skeleton, WordPress checkout, or other vendor-heavy
projects.

Application fixtures are intentionally ignored by git. They are too large and
too dependency-specific for the normal `tests/php/<fixture>/` harness, which
expects small checked-in fixtures with `program.php`, `stdin.txt`, and
`stdout.txt`.

Use one directory per application:

```sh
mkdir -p fixtures/php-apps
composer create-project laravel/laravel fixtures/php-apps/laravel-fresh
```

Then point exploratory tools at files inside the app:

```sh
cargo run -p xo -- ast fixtures/php-apps/laravel-fresh/public/index.php
scripts/check-fast lsp
```

Keep small, stable discoveries as normal compatibility fixtures under
`tests/php/`. Keep the full application here as a local corpus for parser,
indexing, LSP, and future whole-project compatibility experiments.
