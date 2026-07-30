# Echo public site (www)

Positioning and structure for xo.run. In-repo design notes live under `docs/`.
Edition / Spec: Echo 2026.

## Voice

Write public copy as calm programming-language documentation. Speak the way a
clear engineer would explain a form at a whiteboard: facts, forms, and examples
first. Vary sentence length. Prefer plain verbs. Keep the register spoken, not
performed.

Zig clarity + TypeScript pedagogy + Elixir polish still apply as product taste:

- Short sentences, concrete claims
- Glyphs as first-class characters in copy
- Show code, then result
- Honest early-stage status; no fake logos or unearned social proof
- Calm white space; code is the hero medium

## Editorial craft

Public docs aim for Laravel-class clarity (example-first, skimmable headings,
explicit call forms) without Laravel chapter inventory or framework framing.
Language law stays Echo 2026 Spec + Reference + suite.

### Std function entries (Laravel-style)

Every public `std/` export is a first-class reference entry under the spoken-doc
rules. Render order on the module page:

1. **Description** — one or two sentences on what the export does
2. **Signature** — call form after import, e.g. `io.print(value)`
3. **Parameters** — `name: meaning.` fields (or `No parameters.`)
4. **Return value** — what the call yields, including result/option shapes
5. **Example** — short Echo snippet that imports the module and uses the export

Keep description and returns in spoken prose. Keep params structured. Do not
add marketing cadence, antithesis, or em dashes in those fields.

### Spoken-doc intent

Body copy teaches. Lead with what a form does, how you write it, and what
`xo` or std does next. Let the example carry weight. Avoid marketing cadence.
Avoid throat-clearing openers ("In this section…", "It's important to…"). Avoid
landing sentences that restate the paragraph in a punchline. Avoid setup/payoff
pairs that withhold the form until a final flourish.

### Prose ban list (OBJECTIVE)

Do not use these rhetorical patterns in user-facing narrative under `www/`:

| Pattern | What to avoid |
| ------- | ------------- |
| Antithesis | X-vs-Y framing as flourish |
| Corrective negation | "not X, but Y" as a device |
| Paragraph pinning | Closing every block with a moral |
| Parataxis | Stacked fragments that only list tone |
| Summary beats | "In short…", "The key takeaway…" |
| Rhetorical crutches | "Simply put", "Needless to say" |
| Negative parallelisms | "No A. No B. No C." as rhythm |
| Negative anaphoras | Repeated "not … / not …" openings |
| Contrasting pairs | Book/Docs, old/new, cost/gain as twin slogans |
| Rule of three | Forced triple cadence for style |
| Em dashes in prose | Use periods, commas, or parentheses |
| Throat-clearing openers | Warm-up clauses before the fact |
| Landing sentences | Final "that's why" punchlines |
| Setup/payoff constructions | Withhold the rule for a reveal |
| Parallel sentence structures in one paragraph | Same skeleton repeated for effect |
| Stacked noun phrases | Dense modifier piles |
| Filler intensifiers | genuinely, really, truly, actually |
| Corporate-register verbs | leverage, underscore, reflect (as prose verbs) |
| Nominalization | Turning clear verbs into heavy abstract nouns |
| Hedging qualifiers | somewhat, relatively, in many ways (unless a real limit) |

Language-law statements may still forbid a form ("trailing commas are rejected",
"this shape is invalid"). State the rule plainly. Do not dress the forbid as
antithesis or corrective flourish.

Code samples, shell snippets, call forms, and structured param fields are
non-prose. Param fields may use a fixed shape such as `name: meaning.` Keep
them free of banned flourish.

### Good shape

```text
Leaders start statements. A space follows the glyph (except bare < and >).
The body opens with { on the same line as the introducer.
```

```text
xo check resolves and type-checks the module graph.
xo run compiles and executes through the shared LLVM path.
xo build emits a native executable from that same pipeline.
```

## Positioning

**H1 (definition):** Echo is a compiled language. Statement leaders carry control
and binding structure; the rest of the line stays an ordinary expression.

**Subhead:** Write clear programs. Check them. Ship native binaries with `xo`.

Keep product claims factual. Do not invent APIs, partner logos, or unearned
maturity.

## Pillars

1. **Leaders** — `$` `~` `?` `*` `^` (and the rest of the glyph set) mark statement roles
2. **Errors as values** — `!` / match; optionals follow the same idea
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

The left docs TOC uses **one** slate track + gradient train (same as **On this
page**). Nested Standard library group children indent under that rail. Do not
add a second rail or nested train.

## Homepage sections

1. Hero (definition + CTAs + source → `xo` → native visual)
2. Factual proof rail (Echo 2026 · AOT + JIT · Rust · open source)
3. **See it work** — tabbed demos (leaders, Result, structs, tasks)
4. Toolchain story (`check` → `run` → `build`) + source install
5. Echo 2026 / learn path (First program · Reference · Language Spec)
6. Final install CTA
7. Footer

Homepage trust stays factual. Rust, LLVM, the public edition, and the
machine-checked suite are implementation facts.

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
