# Www Is Language User Documentation

The `www` site is Echo's public language and `xo` user documentation surface. It should explain how people use Echo programs, Echo syntax, standard library packages, PHP compatibility behavior, `xo` user commands, and user-facing diagnostics.

Repository-local contributor and maintainer material belongs outside `www`. That includes benchmark harness instructions, internal scripts, fixture workflows, CI/debugging notes, architecture work logs, agent instructions, and other information whose primary audience is someone developing this repository rather than someone using Echo.

For example, a `www` page may document how a user runs an Echo program:

```sh
xo run app/main.echo
```

That page should explain the user-visible behavior of `xo run`, expected input/output, and common diagnostics. It should not explain repository-only workflows such as running `scripts/check-fast bench-echo`, reading `test-results/echo/`, or maintaining benchmark fixture reports. Those belong in `docs/`, `AGENTS.md`, crate-level docs, or other contributor-facing files in the GitHub repository.

The split keeps the public site focused and trustworthy for language users. It also lets contributor documentation be more operational and repository-specific without leaking maintenance details into the product documentation.

When a topic has both audiences, write separate docs for each audience instead of blending them. The website should describe the stable user contract; repository docs should describe how contributors verify, benchmark, or maintain that contract.
