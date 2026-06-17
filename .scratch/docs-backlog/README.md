# Docs Backlog

Practical slices picked from `docs/`.

## Slices

1. [done] Finish output-buffering observable bool returns.
2. [done] Add `flush()` PHP builtin.
3. [done] Add PHP boolean literals `true` and `false`.
4. [done] Implement `ob_implicit_flush()` minimally.
5. [done] Make stdlib imports more real.
6. [done] Promote the supported generator syntax fixture.
7. [done] Add LLVM optimization flags.
8. [done] Start strict-mode type-system diagnostics with one parser/AST-local rule.

## Current Notes

- Slice 1 added fixtures `074_ob_control_success_returns` and `075_ob_control_failure_returns`.
- Slice 2 added fixture `076_flush_builtin` proving `flush()` does not flush active user-level output buffers.
- Slice 3 added fixture `077_boolean_literals` for PHP `true` and `false` literals.
- Slice 4 added fixture `078_ob_implicit_flush_builtin`; current runtime stores the implicit-flush flag, and system stdout is already flushed when bytes reach it.
- Slice 5 validates `from std use ...` against packaged std modules and added Echo fixture `027_std_time_import`.
- Slice 6 removed `unsupported.txt` from `025_generator_syntax`; declared but uncalled generator functions now parse, lower, run, and build as no-op declarations. `026_concurrent_http_tasks` remains unsupported because blocking socket I/O does not yet suspend/wake Echo tasks.
- Slice 7 added `xo build -O0/-O1/-O2/-O3/-Oz`, `xo build --emit-ir`, optimized IR through external `opt`, and native optimization through `clang -O*`.
- Slice 8 rejects PHP reference assignment in strict mode while preserving it in Echo superset mode.
