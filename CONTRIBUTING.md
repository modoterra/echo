# Contributing to Echo

Thanks for your interest in Echo. This document covers legal requirements and
the practical workflow for contributions.

## License and ownership

Echo is released under the **MIT License**. See [`LICENSE`](LICENSE).

Copyright in the project is held by **Modoterra Corporation**. Every
contribution must be made under the **Echo Contributor License Agreement**,
which **assigns all copyright and intellectual property** in your contribution
to Modoterra Corporation. See [`CLA.md`](CLA.md).

### Accepting the CLA

Before we can merge your pull request, you must accept the CLA.

Include this block in the pull request description (copy exactly is fine):

```text
I have read and agree to the Echo Contributor License Agreement (CLA.md).
I assign all copyright and intellectual property rights in my Contribution(s)
to Modoterra Corporation as described in CLA.md.
```

If you contribute on behalf of an employer or company, you must have authority
to bind that entity, or the entity must accept the CLA (see `CLA.md`).

We cannot accept contributions that are not original to you (or your entity)
unless third-party material is clearly identified and under a compatible terms
set, as described in the CLA.

## Before you start

1. Read [`AGENTS.md`](AGENTS.md) for product invariants and proof requirements.
2. Skim [`docs/architecture.md`](docs/architecture.md) and the relevant layer
   docs under [`docs/`](docs/).
3. Prefer small, reviewable changes that land as full vertical slices when the
   change is language-facing (syntax → parse → semantics → IR → codegen →
   runtime → proofs → docs, as applicable).

## Development setup

Requirements and the local edit/test loop are documented in
[`docs/development-speed.md`](docs/development-speed.md).

Useful commands:

```bash
cargo build -p xo
cargo test -p <crate>
scripts/gate changed
scripts/gate echo26    # language / runtime surface changes
scripts/gate workspace # broad check
```

## Pull requests

- Keep the PR focused on one concern.
- Update **crate tests**, **Echo 2026** fixtures (`echo26/` / `e26`), and
  **examples** when your change affects those surfaces (see `AGENTS.md`).
- Update the matching docs under `docs/` (and `www/` when user-facing Spec or
  Reference text changes).
- Do not reintroduce PHP compatibility goals.
- Do not invent language edge cases without authority in the public Spec,
  implementer docs, an ADR, or an explicit maintainer decision recorded in-tree.

### PR checklist

- [ ] CLA acceptance statement included (see above)
- [ ] Focused tests added or updated where behavior changed
- [ ] `scripts/gate changed` (or relevant crate / echo26 gates) pass locally
- [ ] Docs updated when durable facts or user-visible rules changed
- [ ] No unrelated refactors mixed into the same PR

## Reporting issues

Use GitHub issues for bugs and concrete proposals. Include:

- what you expected;
- what happened;
- `xo` / commit version if relevant;
- a minimal reproduction when possible.

## Code of conduct expectations

Be respectful and constructive. Harassment, spam, and bad-faith contributions
are not welcome. Maintainers may refuse or revert contributions that violate
the CLA, project standards, or these expectations.

## Questions

For design discussion, prefer issues or discussions that can be linked from
commits and ADRs. Durable decisions belong in `docs/` and `docs/adr/`, not only
in chat.
