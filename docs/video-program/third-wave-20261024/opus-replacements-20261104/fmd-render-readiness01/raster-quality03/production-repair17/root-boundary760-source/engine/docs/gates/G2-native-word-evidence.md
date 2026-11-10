# G2 — The Native Word evidence packet

- **Gate bead:** `fm-i1q` (flagship gate; **still OPEN** — this packet marshals
  evidence, it does not pass the gate)
- **Packet status:** **Re-marshaled, gate not passed** (2026-10-05; first
  marshaled 2026-08-21 at `0883b5e` by `VioletPike`, fm-i1q.1)
- **Marshal:** `FuchsiaHorizon` (fm-fmd-repin-g2-truth-y3gr)
- **Program owner of record:** Jeffrey Emanuel
- **Evidence source commit:** `6c5dd1c9` (`main`, 2026-10-05), plus the
  Markdown inline-math change that lands with this packet. The packet lands
  on `5bb1d37b`; the commits between touch curved arrows, animation groups
  and portal Tex preflight, and are cited where they bear on a row
- **Suite pin under test:** `franken_markdown @ 68fe29b8af0ddb763a7bf25af3b9c8307122d9d6`
  (`SUITE.lock:33`)
- **Host:** Linux x86-64, kernel 7.0.0-30-generic

This is the recorded packet required by `docs/GOVERNANCE.md` §2, in the
G1 (`docs/gates/G1-core-2d-evidence.md`) shape. It is **not** a pass record:
four of the eight criteria are open. Each row names the command or file
that reproduces it; rows that rest on a closed bead say so.

## Gate disposition

- [ ] **PASS** — not claimable: criteria 1, 5, 7 and 8 are open (5 is partial).
- [x] **OPEN** — `fm-i1q` stays open. The concrete remaining work is
  enumerated in "What still blocks G2" below.

## Acceptance matrix

The eight criteria are quoted from `fm-i1q` (plan §20.3).

| G2 criterion | Evidence | Result |
|---|---|---|
| (1) Tier-1 construct set lays out correctly and beautifully | Typesetting returns Ok for 99.994 % of corpus occurrences, but only a sample is checked against real TeX: the dashboard's oracle row ("Layout within 5% of real TeX", `fm-tex-layout-oracle-bkbc`) is 25.589 % of occurrences and 3.010 % of unique strings at the `3ed1d332` pin, with unsampled strings counted as unverified. `crates/fmn-library/tests/tex_reference_boxes.rs` puts 49/111 authored tier-1 formulas within 5 % of the Reference's measured `Tex` box and 75/111 within 10 % (`ef0eddb2`; the test's floors). Beauty: the `text_sample` and math panels are ratified (`fm-6ppv`, ADR-0018 delegated review), but the gallery is static spike renders (`fm-5wq.50`) | **Partial** |
| (2) Span map drives isolate / t2c / slicing / TransformMatchingTex end-to-end | `crates/fmn-library/src/tex.rs` (isolate/t2c by source identity), `crates/fmn-anim/src/transform_matching.rs` (native span keys), portal binds `8ec3b03`/`d7fab57`/`511f7f1`; the portal gate suites `matching_transform_semantics`, `matching_authoring`, `live_tex` and `bridge` exercise them (`scripts/check_portal_runtime.sh`) | **Green** |
| (3) De-TeX'd classes native (W7DETEX) | `fm-y69` and `fm-ebl` closed; `crates/fmn-library/src/brace.rs`, `numbers.rs` (DecimalNumber), `matchers.rs`/`controls.rs` (Checkmark/Exmark, controls), matrix delimiters from the extensible-delimiter engine | **Green** |
| (4) SVGMobject works for user files (W2SVG) | `fm-6nm` and `fm-5wq.4.50` closed: portal `SVGMobject` builds a VMobject family through Chisel's hardened processor (`crates/fmn-geom/src/svg.rs`), covered by `crates/fmn-python/tests/bridge.py` | **Green** |
| (5) Typeset caching live (W6TEX + W8CACHE) | All three front doors attach the persistent typeset store at the one resolved root: standalone `fmn` (`0126fd51`), `fmn::render` (`a89eaee9`) and `fmn-python` (`59807496`, plus `directories.cache` from the config). Cache keys name the bundled faces' SHA-256s and the pinned `franken_markdown` rev. Fresh-process acceptance passes on Linux (`crates/fmn-cli/tests/typeset_cache.rs`): a warm second run serves 21/21 layouts from disk with byte-identical `--reproducible` frames; a corrupt entry is detected and recomputed; `--clear-cache` empties the store. Static strings are preflighted on the worker pool before `construct` (the formula sheet: 21 layouts before construct, 0 in construct, 0 in play): native scenes from a declared manifest, portal scenes from literal `Tex`/`TexText` calls found in their source. The same tests pass on macOS arm64, and the portal suites pass on a fresh wheel. PG-7's formula workloads time real layouts and verified store hits since `a362dcec` (`fm-a87y`); open: a qualified PG-7 observation (`fm-inr.1`); native scenes need a declared manifest (`fm-typeset-cache-preflight-wiring-1sn0`) | **Partial** |
| (6) Coverage-ratchet dashboard public and live (W6RATCHET) | `docs/ratchet/dashboard.md`, regenerated at the `68fe29b8` pin (`8e6d7372`): frozen G0-4 denominator, CI-enforced pin/ratchet lockstep, and honest columns ("typeset returned Ok" apart from "checked by an oracle"). Recomputing it needs the private corpus, which exists only on the project host | **Green** |
| (7) fmd renders `$…$` in HTML/PDF via the same crates | Re-measured at the `f059cac6` pin (see "Cross-repo payoff"): PDF inline and display math both go through fmd-math `Layout` (`fm-djcw`, UPSTREAM_LEDGER row 21). HTML emits MathML from the fmd-math parse tree and the browser lays it out; this packet records that as **not** using the shared layout | **PDF GREEN; HTML NOT GREEN (MathML; owner decision on whether it satisfies "via the same crates")** |
| (8) PG-1(G2) and PG-7 enforced and blocking | Policy rows are `blocking` in `docs/performance/PERF_GATES.tsv` and the rig is in-tree (`crates/fmn-conformance/src/perf_pg7.rs`, `perf_frontdoor.rs`, `bin/fmn-perf.rs`). ADR-0024 makes host qualification satisfiable (`fm-5wq.8` closed), but no pinned-host observation is committed (`fm-inr.1`), and the Reference side of PG-1 is still a calibration capture (`fm-5wq.17`) | **NOT GREEN** |

## Dependency closure

Of `fm-i1q`'s 25 blockers, 18 are closed and 7 are open.

- **Closed:** `fm-hk9`, `fm-wgl`, `fm-ydw`, `fm-fjq`, `fm-7dw`, `fm-u1u`,
  `fm-70s`, `fm-kg9`, `fm-ebl`, `fm-fw6`, `fm-y69`, `fm-mol`, `fm-6nm`,
  `fm-6ppv`, `fm-5wq.8`, `fm-djcw`, this packet's own bead
  `fm-fmd-repin-g2-truth-y3gr`, and the prerequisite gate `fm-o3j` (G1
  passed 2026-08-20, `f1248b6`).
- **Open, by criterion:**
  - (1) `fm-tex-display-style-nclg` (in progress), `fm-tex-metrics-glyphs-ru72`, `fm-5wq.50`;
  - (5) `fm-typeset-cache-preflight-wiring-1sn0`;
  - (7) `fm-mmzl` (the HTML ruling, `agent:claim:manual`);
  - (8) `fm-inr` (in progress), `fm-5wq.17`.

## The coverage ratchet (criterion 1 numerator, criterion 6)

`docs/ratchet/dashboard.md`, computed against `franken_markdown 3ed1d3320069`
(`ef0eddb2`):

| Plane | Occurrence-weighted | Unique-string |
|---|---|---|
| Parse | 99.994 % | 99.989 % |
| Parse + typeset returned Ok | 99.994 % | 99.989 % |
| Layout within 5% of real TeX (oracle) | 25.589 % | 3.010 % |

- Denominator frozen at G0-4: 9269 distinct strings, 17711 occurrences,
  corpus hash `a8325e49…4bf883fc`, rules_version 1.
- One construct still fails, at parse: `\dx` (1 occurrence). The five
  math-alphanumeric codepoints that failed layout at `82588865` typeset since
  the `e911be2a` repin.
- "Typeset returned Ok" means fmd-math produced a layout without an error,
  not that the layout is right. That is why criterion 1 rests on the box
  oracle and the open metric beads, not on this percentage.
- Enforcement is structural: a `SUITE.lock` pin bump without a ratchet
  re-run fails CI, coverage decreases fail CI, and every out-of-tier
  construct must fail with its precise named tier-tagged error.
- The trend across ten pin revisions is monotone rising
  (`5310d87a` 98.916 % → `68fe29b8` 99.994 % typeset-occurrence).

Against R1's escalation path: coverage has **not** missed the checkpoint;
no public construct-sprint amendment is required at these numbers.

## Layout correctness (criterion 1, correctness half)

- **Box oracle.** `crates/fmn-library/tests/tex_reference_boxes.rs`
  compares native `Tex` boxes with the Reference's measured boxes for 111
  authored tier-1 formulas. At the `3ed1d332` pin (`ef0eddb2`), 49/111 are
  within 5 % on both axes and 75/111 within 10 %; those are the test's
  ratchet floors (26 and 60 at `68fe29b8`). Run it
  with `cargo test -p fmn-library --test tex_reference_boxes -- --nocapture`.
  The output logs every row.
- **Style.** Every `Tex` used to be laid out in TeX text style, while the
  Reference's `Tex` is `align*` display math: `Tex(\sum…\frac)` was 0.588
  tall against the Reference's 1.331. `f3ef1f30` makes display the default.
  Since `68fe29b8`, display integrals take cmex10's 2.0x display size, and
  their heights match. The all-rows tolerance is still open
  (`fm-tex-display-style-nclg`).
- **Largest remaining misses** (the test's logged rows at `68fe29b8`):
  - `cases` and matrices are 21–37 % short;
  - `\emptyset`/`\vec` are 62–81 % wide;
  - `\sqrt` and `\ldots`/`\cdots` are 24–34 % narrow;
  - display integrals are up to 35 % wide (`\iint` +35 %, `\oint` +22 %,
    `\int_0^1` +11 %), because the fallback integral glyph is wider than
    CM's;
  - relation glyphs (≥ ≤ ≠) come from the Noto fallback, because the
    bundled CM Unicode faces lack those codepoints
    (`fm-tex-metrics-glyphs-ru72`).

  The README formula is 3.606 x 1.128 against the Reference's 3.941 x 1.150.
- **Corpus scale.** The corpus TeX box oracle (`fm-tex-layout-oracle-bkbc`,
  open) compares a 585-string sample (6847 occurrences) with real TeX's
  boxes: 279 within 5 % and 421 within 10 % at `3ed1d332`. The rest of the
  9269-string corpus is unverified. The G0-3 ratification and
  `crates/fmn-tex/tests/fmd_math_surface.rs` check fmd-math against TeX's
  published rules construct by construct, not corpus-wide.

## Look Gallery (criterion 1, beauty half)

- **`text_sample`** (`da6ca7f`, fm-gfn): `docs/g0/g0-2-renders/fmn-text-sample.png`,
  SHA-256 `8f175ae968fcc0aeaa30e1595b492b9d24d5d59d287362efb2346fc7c0fec143`.
  - Verdict **different-but-fine**, ratified 2026-08-24 (`fm-6ppv`,
    `docs/g0/G0-2-look-study-ratification.md`).
  - The face divergence is the deliberate D-08/BN-05 bundled-font call.
    What must correspond does: centring, `next_to` spacing, the 60/32 size
    ratio, true italic contrast, the em dash, and the AA edge character.
- **Math panel.** Captured against the Reference in the 2026-08-23
  supplement session (`PROVENANCE.json` supplements). Settled
  different-but-fine (BN-05) and ratified 2026-08-24 by delegated blind
  review under ADR-0018 (`docs/g0/G0-2-look-study-ratification.md` §9).
- **Honest gap.** Both panels are static spike renders
  (`g0_2_look` in `spikes/g0-8-accelerator`), judged by agents, and there is
  a single math panel. The gallery still has to be regenerated from the
  production entry points with build provenance, a math sheet and an owner
  verdict lane (`fm-5wq.50`). A corpus-scale math gallery rides
  `fm-tex-layout-oracle-bkbc`.

## Span maps end-to-end (criterion 2)

- Provenance source: fm-70s closed — every glyph carries source-span
  provenance from fmd-math layout; there is no render-twice-and-align path
  anywhere in the tree.
- Native consumers: `crates/fmn-library/src/tex.rs` applies `isolate=` and
  `tex_to_color_map` by source identity through the span map;
  `crates/fmn-anim/src/transform_matching.rs` matches Scribe primitives by
  native span keys (`511f7f1`).
- Portal: `TransformMatchingTex` bound through `fmn-python` on the native
  span maps (`8ec3b03`, `d7fab57`, fm-5wq.4.49), alongside the indication
  family (fm-5wq.4.48). The portal gate suites `matching_transform_semantics`,
  `matching_authoring`, `live_tex` and `bridge` run them against an installed
  wheel (`scripts/check_portal_runtime.sh`).

## De-TeX'd natives (criterion 3)

`fm-y69` and `fm-ebl` closed. `Brace`/`BraceLabel` are parametric path
generators (`crates/fmn-library/src/brace.rs`), `DecimalNumber` is pure text
(`crates/fmn-library/src/numbers.rs`), Matrix brackets come from the
extensible-delimiter engine, and Checkmark/Exmark/controls are native
(`crates/fmn-library/src/matchers.rs`, `controls.rs`; portal exposure at
`9b0db8c`). None of these classes routes through a typesetter.

## SVGMobject (criterion 4)

The Chisel SVG document processor is hardened, with explicit accept/reject
(`fm-6nm`, `crates/fmn-geom/src/svg.rs`). Since `fm-5wq.4.50` closed, portal
`SVGMobject("file.svg")` constructs a real VMobject family through it. This
is covered by `crates/fmn-python/tests/bridge.py`, which the `fmn-python`
cargo suite embeds and executes.

## Typeset caching (criterion 5) — PARTIAL

`fmn-cache` is the content-addressed store (`fm-fw6`). `fmn-tex` owns
Tex/TexText typeset caching and the pre-play preflight (`fm-7dw`;
`crates/fmn-tex/src/typeset.rs`, `engine.rs`, `session.rs`).

Production wiring, as of 2026-10-09:
- **`fmn`.** Every builtin render opens the store at the root that
  `--cache-dir`, `directories.cache` or the platform convention selects. This
  is the same root `--clear-cache` and `fmn doctor` resolve. The render shares
  it across the prerun and the render (`0126fd51`). Each robot render record
  carries a `typesetting` object: hits, misses, bytes read and written,
  corrupt entries recomputed, store reads and writes that failed
  (`store_errors`), layouts before `construct()`, before the first frame
  and inside play, and the preflight's requests, workers, active workers and
  wall time. None of these enter the manifest. An unavailable root is
  `cache_error`; a root that opens but cannot read or write (read-only,
  replaced) is `persistent: true` with nonzero `store_errors`.
- **`fmn::render`.** Uses the same session and counters (`a89eaee9`,
  `46450092`, `0126fd51`).
- **Portal.** Scene constructors use the persistent cache (`59807496`),
  now at the config's `directories.cache` when set; a value the cache
  refuses leaves the thread typesetting in memory with the reason reported,
  never a failed Scene. `Scene.run` preflights every literal
  `Tex`/`TexText` string in the scene class's own source, its scene bases
  and its mixins, on the native worker pool before `setup()`.
- **Keys.** The engine fingerprint folds in the SHA-256 of every bundled
  face, the pinned `franken_markdown` rev from `SUITE.lock` and the fmn-tex
  version, besides the probe layouts. A pin bump or a font edit therefore
  cold-starts the cache even where no probe would have noticed; a unit test
  in `engine.rs` fails if that identity stops reaching the fingerprint.

Acceptance evidence (Linux, through RCH):
- **Cache, fresh processes.** `crates/fmn-cli/tests/typeset_cache.rs`
  renders `formula_sheet.v1` (20 formulas) under `--reproducible`. The cold
  run lays out 21 strings (20 formulas plus the scale probe) and publishes
  them. A second process serves all 21 from disk with 0 layouts, the same
  frame bytes and the same closure digest. A flipped byte in one entry is
  detected, recomputed, republished and healed. `--clear-cache` empties the
  store and the next run is cold with the same bits.
- **Preflight.** The same sheet is typeset in one batch on 2 or more
  workers before `construct()` begins: all 21 layouts precede construction,
  so construction lays out none and play none. "0 layouts inside play"
  alone is not evidence of a preflight on this sheet, which builds every
  formula before its first play: with its manifest emptied, play still lays
  out 0, but construction lays out all 21 (`e5b0163f`). The tests assert
  the before-`construct()` count.
  `crates/fmn/tests/native_typesetting.rs` plants typesetting inside a
  segment and the report catches it. The portal suite
  `crates/fmn-python/tests/typeset_static_preflight.py` finds 21 literal
  strings, preflights them, and leaves construction one dynamic layout.
- **e2e.** `lifecycle.typeset_cache_warm_second_run.v1` and
  `lifecycle.typeset_preflight_before_first_play.v1` run in the fast tier.

Further evidence, 2026-10-09:
- **macOS 26.2 arm64, local:**
  - the CLI cache and preflight tests pass 7/7;
  - fmn-tex and fmn-cache pass;
  - the portal typesetting suites pass 3/3.
  - Explicit cache roots under `$TMPDIR` (behind the `/var` link) stay
    persistent. The platform-convention base under a linked `$HOME` is still
    refused; that is recorded on `fm-macos-var-symlink-gate-aqr1`. The
    render reports it (`persistent: false`, `cache_error` naming the `/var`
    link) and finishes in memory.
  - After the store-error fix (`c21d049e`), on the same Mac: the CLI suite
    passes 8/8, including the read/write-failure test (42 store errors for
    21 requests, frames identical); fmn-tex passes 74/74. With the counting
    removed, that test fails (0 store errors), as does its fmn-tex
    counterpart.
  - Hit and miss are byte-identical for `png`, `png_sequence` and `y4m`
    under `--reproducible`, with the same closure digest for the same cache
    root. A different `--cache-dir` then changed the closure digest, because
    the resolved config (C4) included `directories.cache`
    (`fm-cache-dir-in-certified-closure-2y2a`). Since `98d3e923`, C4 carries
    it as an inert marker, as it does `render.threads`: two cache roots give
    one closure digest and the same frames (`typeset_cache.rs`), while
    `tex.template`, `text.font` and the other `directories` keys still
    separate configs. With the normalization removed, the test fails on two
    different digests.
- **Clean wheel.** At `e5b0163f` (CPython 3.13, Linux), the three typesetting
  suites pass and the runtime receipt passes (2007 reviewed rows, 0
  contradictions). The wheel was built from `6a21499d`, the same patch
  before a rebase onto a different base; that SHA is not on `main`. The
  gate now runs those suites (`3a625f16`). Separately,
  the gate was red on `paired_output_cli` (`fm-djox`), which also failed at
  `241def91`, before this work. At `b27526fb` (CPython 3.13, Linux, a fresh
  wheel) the whole gate passes, `paired_output_cli` included: 78/78 runpy
  suites, among them the typesetting suites with the refused-root receipt
  and trailing-mixin checks, and the receipt (2007 reviewed rows, 0
  contradictions).
- **Front-door timing.** `fmn` (release-perf), formula sheet, preflight wall
  for 20 formulas, on a shared 128-thread Linux host at load 2–6. These are
  unqualified observations, not a PG-7 pass:
  - cold: 114 ms on 1 worker, 64 ms on 20;
  - warm, from disk: about 1 ms;
  - frames identical.

- **PG-7 producers (`fm-a87y`, fixed at `a362dcec`).** From `472a19c0`
  until that commit, `formula-cold` and `formula-cached` timed the engine's
  memory front, not a layout or a store hit. Each formula repetition now
  clears that front outside the clock, and the run is refused unless the
  engine's counters show exactly one layout (cold) or one verified store hit
  (cached) per repetition, with no memory-front hit. The trace schema is
  `fmn-perf-pg7-trace/2`. Before and after, `fmn-perf measure-pg7`
  (release-perf), five interleaved rounds of 24 samples, one pinned CPU, on a
  shared 128-thread Linux host at load about 1. These are unqualified
  observations, not a PG-7 pass:
  - `formula-cold`: median 1.65 µs (memory hits) before, 2.34 µs after
    (24/24 layouts in every round). The fixture `q^7+z` is the 5-byte
    corpus median, so a real layout is that small;
  - `formula-cached`: median 2.47 µs (memory hits) before, 28.7 µs after
    (24/24 verified store hits; round medians 25–43 µs);
  - `text-10k-glyph`: unchanged workload and definition, 2.28 ms before,
    2.37 ms after.

What keeps this row partial:
- No qualified PG-7 observation exists (`fm-inr.1`).
- Native scenes are preflighted only from their declared manifest. Rust
  source cannot be inspected at run time, and an undeclared static `Tex` is
  laid out in `construct`, not on the pool.

The row was first marked green on the closed beads alone, then corrected to
NOT GREEN on 2026-10-05, and is partial since `0126fd51`.

## Cross-repo payoff (criterion 7)

fmd-math and fmd-font are franken_markdown workspace crates consumed here as
git dependencies at the pinned rev (`SUITE.lock:33`). Whether fmd itself
renders `$…$` through the same crates was first measured on 2026-09-25 at
franken_markdown `09562c1f` and re-measured at `68fe29b8` on 2026-10-05.

**Re-measured on 2026-10-06 at the `f059cac6` pin**, which carries fm-djcw's PDF
inline math. The `fmd` binary was built from that rev through rch
(`cargo build --release --features cli --bin fmd`). The input is the same
probe document:

```markdown
# Criterion 7 probe

Inline math: $\frac{a}{b} + x^2$ in a sentence.

$$\int_0^1 \sqrt{1 - x^2}\,dx = \frac{\pi}{4}$$
```

`fmd render probe.md --to both --out probe.html` produces `probe.html`
(41,526 bytes, SHA-256 `3b3df92d…a74f50`, byte-identical to the 2026-10-05
measurement) and `probe.pdf` (17,594 bytes, SHA-256 `530154af…bd48400`):

- **PDF:** both formulas go through fmd-math's `Layout`. The structure tree
  has two `/Formula` elements, with `/Alt` set to each source. The content
  stream fills both from fmd-math glyph outlines, and its text never selects
  the monospace face. The inline formula is one unbreakable box on the text
  baseline, typeset in text style at body size.
  `pdftotext -layout probe.pdf -` recovers both sources through their
  `/ActualText` anchors. The page rasterized with `pdftoppm -r 110 -png`
  shows a typeset fraction, plus sign and superscript inside the sentence.
- **HTML:** unchanged. Both formulas become MathML `<math display="inline">`
  and `<math display="block">` elements, generated from the fmd-math parse
  tree and laid out by the browser. fmd-math's `Layout` is not involved.

So criterion 7 now holds for PDF: inline and display math both go through
the shared layout. HTML uses the shared parser but not the shared layout.
This packet does not reinterpret the criterion. Whether parser-only MathML
satisfies "via the same crates" for HTML is recorded here as an open owner
decision (`fm-mmzl`), not as green.

## Performance gates (criterion 8) — PG-1 is NOT green

What exists at the evidence commit:

- The §17.2 policy catalog is machine-readable and marks both PG-1 rows and
  all three PG-7 rows `blocking`/`core` (`docs/performance/PERF_GATES.tsv`),
  with the explicit rule that **a row becomes pass-capable only after a
  content-addressed pinned-host observation is committed** through
  `fmn_conformance::perf::Baseline`.
- The rig code is in-tree: `crates/fmn-conformance/src/perf.rs`,
  `perf_pg7.rs`, `perf_frontdoor.rs`, `perf_host.rs`, and the
  `fmn-perf` binary. The Python Reference wall-clock captured in
  `docs/performance/reference-baseline-2026-07-28.json` is calibration-only
  shared-host evidence, not the PG-1 denominator
  (`docs/performance/PERFORMANCE_GATES.md:57`).
- ADR-0024 (`fm-5wq.8`, closed 2026-09-27) makes qualification satisfiable.
  G2's PG-1(G2) and PG-7 close on one qualified Linux observation: an
  isolated 8-physical-core slice of a bare-metal host. The Apple profile
  gates only Apple-named rows.
- Pinning the host and landing the replayable baseline corpus is an owner
  host action (`fm-inr.1`, in progress). Deriving PG-1's Reference side from
  raw samples on real-GL hardware is `fm-5wq.17`.

What does not exist: any committed qualified observation for PG-1(G2)
(≤ 0.5× Reference wall-clock) or PG-7 (formula < 3 ms cold / < 100 µs
cached; 10k-glyph < 20 ms). **PG-1 has no attributable pass and is not
marked green here.** Under ADR-0018, inconclusive perf evidence is not a
HOLD, but it is also not a pass, and G2 makes these gates blocking.

## What still blocks G2

1. **Criterion 1, layout correctness and beauty.**
   - The display-style all-rows tolerance (`fm-tex-display-style-nclg`).
   - The metric and glyph misses the box oracle names (`fm-tex-metrics-glyphs-ru72`).
   - A corpus-wide layout oracle (`fm-tex-layout-oracle-bkbc`).
   - A Look Gallery regenerated from production entry points with an owner verdict lane (`fm-5wq.50`).
2. **Criterion 5:** a qualified PG-7 observation (`fm-inr.1`); the producer
   now times real layouts and store hits (`fm-a87y`, `a362dcec`); native preflight
   without a declared manifest (`fm-typeset-cache-preflight-wiring-1sn0`).
3. **Criterion 7:** PDF is done (`fm-djcw`). HTML math is browser-laid-out
   MathML from the fmd-math parser; whether criterion 7 requires fmd-math
   `Layout` for HTML too is an owner ruling (`fm-mmzl`).
4. **Criterion 8:** a qualified Linux observation for PG-1(G2) and PG-7
   (`fm-inr.1`), and an honest PG-1 Reference side (`fm-5wq.17`).

## Validation provenance note

Every row cites a closed bead, a committed file or commit, or a command a
reader can re-run. Rows that need a fresh run name it (the box oracle, the
criterion-7 probe, the portal gate suites). Green is never inferred from a
closed bead alone; criterion 5 was, and is corrected here. The gate bead
`fm-i1q` remains open; only the program owner's process closes it.

## Marshal history

- **2026-08-21:** first marshal at `0883b5e`, against the `82588865` pin.
  Criteria 4 and 8 were open there; criteria 5 and 7 were marked green
  without production evidence.
- **2026-08-24:** criterion 4 turned green (`fm-5wq.4.50`), and the
  `text_sample` and math panels were ratified.
- **2026-09-25:** criterion 7 was measured and is not green. The repin to
  `e911be2a` cleared the layout-plane failures.
- **2026-10-04:** a side-by-side against the pinned Reference
  (`docs/IMPLEMENTATION_STATUS.md`) found the following, followed by the
  repin to `68fe29b8`:
  - the text-style `Tex` default;
  - the box-oracle misses;
  - the criterion-5 overclaim.
- **2026-10-05:** re-marshaled at the `68fe29b8` pin. Earlier post-marshal
  notes are folded into the rows above, and their text stays in git history.
