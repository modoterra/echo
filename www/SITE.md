# Echo public site (www)

Positioning and structure for xo.run. Implementer facts stay in `docs/`
(ADR 0007). Edition / Spec: Echo 2026 (ADR 0015).

## Voice

Zig clarity + TypeScript pedagogy + Elixir polish.

- Short sentences, concrete claims
- Glyphs as first-class characters in copy
- Show code, then result
- Honest early-stage status; no fake logos or unearned social proof
- Calm white space; code is the hero medium

## Positioning

**H1 (definition):** Echo is a compiled language with leaders instead of keywords.

**Subhead:** Write clear programs. Check them. Ship native binaries with `xo`.

## Pillars

1. **Leaders, not keywords** — `$` `~` `?` `*` `^` (and the rest of the glyph set)
2. **Errors are values** — `!` / match; optionals the same idea
3. **Small loop, native output** — `xo check` / `run` / `build`; I/O from `std`

## Primary nav

| Item              | Path       | Notes                      |
| ----------------- | ---------- | -------------------------- |
| Home              | `/`        | Product narrative          |
| Docs              | `/docs`    | Form-by-form Reference     |
| Book              | `/book`    | Narrative why / when       |
| Echo 2026         | `/e26`     | Edition + Spec TOC + suite |
| **Install** (CTA) | `/install` | Solid button; get `xo`     |

## Docs left rail

The left docs TOC uses the same slate track + gradient train as **On this page**
(right column). Nested children use the same pattern as `echo-php-old`.

## Homepage sections

1. Hero (definition + CTAs + source → `xo` → native visual)
2. Factual proof rail (Echo 2026 · AOT + JIT · Rust · open source)
3. **See it work** — tabbed demos (leaders, Result, structs, tasks)
4. Toolchain story (`check` → `run` → `build`) + source install
5. Echo 2026 / learn path (First program · Reference · Language Spec)
6. Final install CTA
7. Footer

Homepage trust stays factual. Rust, LLVM, the public edition, and the
machine-checked suite are implementation facts, not partner or customer logos.

## Echo 2026 section

| Path            | Role                                      |
| --------------- | ----------------------------------------- |
| `/e26`          | Edition overview                          |
| `/e26/spec`     | Language Spec TOC (Reference + suite map) |
| `/e26/run`      | Suite runner                              |
| `/e26/layout`   | Fixture layout                            |
| `/e26/protocol` | Candidate binary protocol                 |

Cross-links: Reference ↔ Spec ↔ suite pages keep the triangle explicit.

## Out of scope (later)

Playground, richer download tabs, `/e26` URL rename to `/echo-2026`.
