# xo.run

Static React site for Echo / `xo.run`, built with Vite, TypeScript, React, and
Tailwind CSS.

Public positioning, homepage outline, and nav rules: [`SITE.md`](SITE.md).

## Commands

```bash
npm install
npm run dev
npm run lint
npm run format
npm run build
```

`/try` needs the compiler frontend wasm. From the repo root:

```bash
just wasm    # writes www/public/echo-wasm/
just try     # wasm + npm --prefix www run dev
```

`npm run dev` starts the local site. `npm run lint`, `npm run format`, and
`npm run build` validate the site before publishing `www/dist`.

## Cloudflare Pages

Same layout as the previous `xo.run` site:

| Setting        | Value           |
| -------------- | --------------- |
| Root directory | `www`           |
| Build command  | `npm run build` |
| Build output   | `dist`          |

`/try` loads `public/echo-wasm/` (from `just wasm`). Rebuild and commit those
bindings when the frontend or `std/**/*.echo` changes so Pages can ship the
playground without a Rust toolchain.

SPA deep links use the same `public/404.html` → `/?/path` bounce and
`index.html` restore script as before. Custom domain: `public/CNAME` → `xo.run`.

## Search

Docs search uses MiniSearch over content in `src/docs/`. Open the palette from
the top bar, or press `/` / `Ctrl+K` / `Cmd+K`.

Lexical search works for every build. Hybrid semantic ranking needs the local
embedding model under `public/models/xmlml6v2` and:

```bash
npm run build:semantic
```

Without the semantic index, the palette still works with lexical ranking and
shows Semantic as inactive.
