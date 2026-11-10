# G0-4 — Corpus harvest: ratification note (fm-or4)

**Status:** Ratified, 2026-07-23. This note is the **tier-1 definition of
fmd-math** (plan §11.5, §20.1 spike 4) and the **normative specification
of the public coverage ratchet**: W6 (fm-wgl, fm-mol) implements the
dashboard from this text alone. The executable form is
`scripts/harvest_tex_corpus.py` (stdlib-only Python, deterministic,
runtime ≈ 2 min); the committed artifacts live in `docs/g0/g0-4-corpus/`.

## The pins

| Source | Commit | Checkout |
|---|---|---|
| `3b1b/videos` | `e317d6c5eaa8370a2deb4d148c246b0d0e9fbe6f` | `scripts/videos_ref` (gitignored) |
| `3b1b/manim` (the Reference) | `6199a00d4c1b1127ebe45cb629c3f22538b10e13` | `scripts/manim_ref` (gitignored) |

Both are recorded in `SUITE.lock [reference]`. The harvest is repeatable:
re-running the tool against these pins reproduces every committed artifact
**byte-for-byte** (verified; sorted iteration everywhere, no timestamps).

## Headline numbers

| | |
|---|---|
| Files scanned | 504 (`videos_ref` all `*.py` + `manim_ref/manimlib`) |
| Tex-family call sites | 12,258 — `OldTex` 5,196 · `OldTexText` 4,433 · `Tex` 2,303 · `TexText` 325 · `SingleStringTex` 1 |
| **Denominator** | **9,269 distinct (mode, string) pairs · 17,711 occurrences** |
| Corpus hash | `a8325e49e0ce78fcc735533952740e9adeaaa5cb10f9c13d73aaa3ba4bf883fc` |
| Dynamic args (excluded) | 2,209 (their 1,116 distinct literal fragments feed advisory counts only) |
| Distinct constructs | 206 |
| §11.4 seed surface alone | **98.95 %** of occurrence mass |
| **Tier 1 (170 constructs)** | **99.54 %** of occurrence mass |

## Extraction rules (what the denominator is)

1. Every `*.py` file under the two pinned trees is parsed with Python's
   `ast`. Call sites of `Tex`, `OldTex`, `SingleStringTex` (math mode) and
   `TexText`, `OldTexText` (text mode) are collected — these are the only
   Tex-family constructor names that exist at the pins.
2. **Each fully-literal positional string argument is one corpus
   occurrence.** The multi-arg idiom `OldTex("a", "=", "b")` therefore
   contributes three occurrences — exactly matching the Reference's
   per-argument `SingleStringTex` semantics. The most frequent corpus
   strings are single tokens (`=` ×904, `+` ×565) for this reason.
3. **Reference preset strings:** a string default value of any function
   parameter whose name contains `tex` (e.g. `Brace.__init__`'s
   `R"\underbrace{\qquad}"`) is a math-mode occurrence.
4. **Dynamic arguments** (f-strings, concatenations, `"".join(...)`,
   variables, `*args`) are excluded from the denominator — a fragment is
   not a complete formula and must never be scored as one. Their nested
   string constants feed an **advisory** construct tally, whose
   distribution mirrors the mainline ranking (top advisory constructs:
   `script:sub/sup`, `\over`, `\text`, `\frac`, `\textbf` …), evidence
   that the exclusion does not bias the construct order.
5. **Instrumented run: deliberately not performed.** Executing the videos
   tree requires LaTeX, per-video assets, and era-specific helper code —
   exactly the substrate this program deletes. The right instrumentation
   point is W10's VIDEO_CORPUS.lock work, where the pinned scenes execute
   under fmn-python and composed strings can be captured at the `Tex`
   boundary. The static denominator here is fixed regardless; an
   instrumented harvest would only ever *add* a second corpus, never
   mutate this one.

## Construct lexing (what a "construct" is)

A TeX-lexer-level pass over each corpus string (rank needs token identity,
not tree shape):

- **Control words** `\frac`, **control symbols** `\\`, `\,`, `\{` …
- **Environments** as `env:name` (from `\begin{name}`).
- **Structural**: `script:sup` (`^`), `script:sub` (`_`), `prime` (`'`,
  math mode only), `alignment-tab` (`&`), `tie` (`~`), `math-island`
  (each unescaped `$…$` region in a text-mode string).
- **Non-ASCII characters** as `char:U+XXXX` (font-coverage surface).
- Text-mode strings are lexed as mainland text; their `$…$` islands are
  lexed as math (apostrophes in prose are not primes).
- Unescaped `%` opens a TeX comment to end-of-line; `\%` is a construct.

## The tier-1 cut (normative)

**T1 = the §11.4 seed surface ∪ a greedy rank extension until ≥ 99.5 % of
corpus occurrences are strings composed entirely of T1 constructs.**
The result: **170 constructs, 99.537 % coverage** — the `tier` column of
`construct_table.tsv` is the machine-readable form. T2 is everything
else observed (36 constructs, none above 13 occurrences), scheduled by
rank. §11.4-enumerated constructs that never occur in the corpus remain
in-scope for W6 — the seed is normative for the engine surface; the
corpus only orders the work.

Findings that shape W6:

- **`\over` is rank 5** (760 occurrences) — the primitive-style fraction
  is mainline 3b1b idiom, not a curiosity. It is T1 and must parse with
  its TeX semantics (the whole enclosing group splits at the `\over`).
- **Text mode is load-bearing**: 4,758 call sites (≈ 39 %) are
  `TexText`/`OldTexText`; `math-island` is the #4 construct. The TexText
  contract (text mainland + `$…$` islands, `\textbf`/`\emph`/`\underline`
  in text) ships with tier 1, not later.
- **No `matrix`-family environment occurs anywhere in the corpus** — 3b1b
  builds matrices exclusively through the `Matrix` mobject (which
  composes per-element `Tex` plus `\left[\begin{array}{c}…` brackets).
  The `matrix` envs stay T1 by seed declaration, and the de-TeX'd Matrix
  design (§12) is confirmed as the actual load path.
- **The default preamble is part of the surface**: the Reference's
  `tex_templates.yml` default template defines `\minus` (36 occurrences,
  T1 by rank) and loads `dsfont` (`\mathds`, 38), `wasysym`
  (`\male`/`\female`/`\earth`/`\mars`), `pifont` (`\ding`). The fmd-math
  **default preamble pack** must therefore define `\minus` and map
  `\mathds` → blackboard; the wasysym/pifont singles are T2 by rank.
- The empty string occurs (`Tex("")` ×13, `TexText("")` ×12): both are
  valid, trivially-covered corpus members.
- Unicode in strings is rare and text-mode (`ö`, `ä`, `’` …): a handful
  of T2 `char:U+XXXX` entries; no CJK, no bidi — consistent with the
  typography tiering (§2.3).

## The public coverage ratchet (normative counting rules)

1. **The denominator is fixed.** It is the multiset D of (mode, string)
   pairs whose per-string hashes and occurrence counts are committed in
   `denominator.tsv`, identified globally by the corpus hash
   `a8325e49e0ce78fcc735533952740e9adeaaa5cb10f9c13d73aaa3ba4bf883fc`
   (sha256 of the sorted `sha256\tmode\tcount` lines). The ratchet never
   re-harvests against moving trees.
2. **Four published numbers**, recomputed per fmd-math release:
   - *occurrence-weighted parse coverage* = Σ count(s) over s ∈ D that
     **parse** end-to-end ÷ Σ count(s) over all of D (= 17,711);
   - *occurrence-weighted layout coverage* = the same with **parse +
     full layout** succeeding (no unsupported-construct error at either
     stage);
   - *unique-string parse coverage* and *unique-string layout coverage* =
     the same two ratios with every count(s) replaced by 1 (denominator
     9,269).
3. **Whole-string success only.** A string is covered iff the entire
   string succeeds in its recorded mode (math strings under the math
   grammar; text strings under the TexText contract). A partial parse is
   a failure. Failures must carry the precise named construct
   (`\substack is not yet supported; tier T2, tracked at …`) — never
   silence, never garbage.
4. **The dashboard publishes** the four coverage numbers plus the
   per-construct failure tally — construct names and counts only,
   **never the 3b1b-authored string text** (§15.3).
5. **Monotonicity.** Within a `rules_version`, each of the four numbers
   may only rise release-over-release; a decrease is a regression and
   blocks. Lexer/extraction changes bump `rules_version` in the tool,
   re-derive the denominator from the same pins, restate the corpus
   hash, and annotate the boundary on the public chart; coverage is
   comparable only within a version.
6. **Verifiability without disclosure.** Anyone with the pinned trees can
   re-run `scripts/harvest_tex_corpus.py` and reproduce
   `denominator.tsv` byte-for-byte; the committed hashes prove corpus
   integrity while the strings themselves stay private.

## Licensing (§15.3)

The 3b1b-authored strings are CC BY-NC-SA-adjacent course material and
feed **private fixtures only**: the full corpus (strings + provenance)
lives in `corpus/tex_corpus.jsonl`, which is **gitignored** alongside the
source checkouts. The committed artifacts — `construct_table.tsv`,
`denominator.tsv` (hashes + counts), `harvest_manifest.json` — contain
construct names, statistics, and hashes, and are publishable. The public
ratchet reports numbers and construct names, never corpus text.

## Seeding W10 (VIDEO_CORPUS.lock)

`harvest_manifest.json` records per-directory occurrence mass. The
TeX-heaviest eras are `_2017` (4,279), `_2018` (3,332), `_2019` (2,482),
`_2016` (1,747), `_2020` (1,732); `_2015` is near-empty (164) and the
modern era is lighter (`_2024` 392 · `_2025` 760 · `_2026` 253) — scene
selection for the G4a gallery should draw its TeX-stress scenes from the
2016–2020 band and its API-modernity scenes from 2023+ (fm-rqc).
