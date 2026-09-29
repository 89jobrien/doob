# Fonts

Self-hosted. No CDN request is made at render time — the site works offline
and from `file://`. The `@font-face` rules and the system-stack fallbacks are
both in `style.css`, so the site renders acceptably even if these files are
missing.

All faces are the **latin** subset, pulled from Fontsource. Every one is under
the SIL Open Font License 1.1.

## doob

| File | Family | Weights | Source |
| --- | --- | --- | --- |
| `bricolage-grotesque-latin-wght-normal.woff2` | Bricolage Grotesque | 200–800 | <https://fontsource.org/fonts/bricolage-grotesque> |
| `newsreader-latin-wght-normal.woff2` | Newsreader | 200–800 | <https://fontsource.org/fonts/newsreader> |
| `fragment-mono-latin-400-normal.woff2` | Fragment Mono | 400 | <https://fontsource.org/fonts/fragment-mono> |

Fragment Mono ships a single weight on Fontsource, which is the right amount
for a utility face.

```bash
# re-fetch, from the repo root
base=https://cdn.jsdelivr.net/npm
curl -sLo site/fonts/bricolage-grotesque-latin-wght-normal.woff2 \
  "$base/@fontsource-variable/bricolage-grotesque@latest/files/bricolage-grotesque-latin-wght-normal.woff2"
curl -sLo site/fonts/newsreader-latin-wght-normal.woff2 \
  "$base/@fontsource-variable/newsreader@latest/files/newsreader-latin-wght-normal.woff2"
curl -sLo site/fonts/fragment-mono-latin-400-normal.woff2 \
  "$base/@fontsource/fragment-mono@latest/files/fragment-mono-latin-400-normal.woff2"
```

## Why these faces

doob is a card index: records in lanes, priorities as stamped labels, commands
and record ids as monospace. Bricolage Grotesque has the character of a
labelled tab without being a monospace face, and its skeleton is deliberately
unlike Newsreader's, so the display and body contrast reads as a choice rather
than an accident. Fragment Mono appears only where the product actually prints
monospace — commands, uuids, statuses.
