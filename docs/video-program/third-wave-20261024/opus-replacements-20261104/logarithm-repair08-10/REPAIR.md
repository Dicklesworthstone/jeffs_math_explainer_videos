# Logarithm chapters 08–10: staged picture repairs

Author repair for bead `mx-video-program-program-87do.63`, Mail 1470. WhiteSwan, 2026-11-04.

**Status: all 17 findings are repaired in source. Nothing has been compiled, tested, rendered, seen or heard.** Root integrates and verifies. This is not an independent acceptance.

## What to apply

| Patch | Files | Depends on |
|---|---|---|
| `patches/shared-draw-helpers.patch` | `draw.rs` (F1, F5) | nothing |
| `patches/chapters-08-10.patch` | `product_rule.rs`, `matrix_logs.rs`, `doing_nothing.rs`, `docs/storyboard.md` | nothing |

Each applies on its own, and in either order the two reproduce `candidate/` byte for byte (`checks/patch_checks.out.txt`). Without the shared patch, chapter 09 still has its unreadable cards and chapter 10 its broken border.

No script or take changes. `script.tsv` is not in the candidate and its hash is unchanged. No separate accuracy change to the narration is proposed.

## Binding

Baselines in `baseline/` equal canonical as of 10:59Z; re-check before applying, since canonical moves between syncs.

| File | sha256 prefix |
|---|---|
| `src/chapters/draw.rs` | `59ebd09d2fa5c793` |
| `src/chapters/product_rule.rs` | `a8b4e31d98b23bd7` |
| `src/chapters/matrix_logs.rs` | `4250a5f0294f58a3` |
| `src/chapters/doing_nothing.rs` | `bf0008113e804186` |
| `docs/storyboard.md` | `c5bc6e7e9fbd0797` |
| `script.tsv` (unchanged) | `3e4e34af7090e448` |

The review is frozen and untouched: `logarithm/preview02-independent-review-08-10/REVIEW.md`, sha256 `4b1122ae78e42734c7db985fe119d18b905d50080d945c08dbbc6ffe6bc613e4`. Its evidence is bound to preview02. Candidate hashes are in `SHA256SUMS`.

## The 17 findings

Line numbers are in the candidate. Times are scratch-voice estimates.

| ID | What the picture should now show | Where |
|---|---|---|
| **F1** | Decimal cards read clearly: columns apart, brackets outside the digits. `matrix_card` typesets its entries first, then sizes columns and brackets from the widest one. | `draw.rs:491` |
| **F2** | "turn-and-scale matrix ↔ complex number" is whole, spanning about 3.09..6.81. | `matrix_logs.rs:377` |
| **F3** | "keep −π < turning < π" and "ellipse flows: ±2π" no longer touch: spans 2.83..4.87 and 5.19..6.71. | `doing_nothing.rs:523`, `:534` |
| **F4** | For numbers, the window is open at −π and closed at +π. On "strictly between" in 10_4 the closed dot swaps to an open ring. | `doing_nothing.rs:415`, `:555` |
| **F5** | The outer grid box stays whole when a flow comes home, and the stretch flow's x = ±2 lines stay drawn. A mirror keeps 73 of 73 samples on every border line, where the old test kept 0 for 2πJ. | `draw.rs:569` |
| **F6** | The `z²` label sits beside the arrow tip, off the vertical axis. | `product_rule.rs:160` |
| **F7** | The √z definition is centred above the axis's top end (y 2.48..2.86; axis ends 2.30), so the axis does not cross it. | `product_rule.rs:359` |
| **F8** | "spiral flow: turns 0.872 radians per unit time, about 0.28π", right edge about 6.61. | `doing_nothing.rs:464` |
| **F9** | On "Here is the picture that matters" the resting plane fades in. Arrows and X card follow at sentence 2, motion at sentence 3. | `matrix_logs.rs:233` |
| **F10** | The spiral sweeps at constant speed from "turns round at a steady" until just before "or does not turn". Only then does the stretch flow and its label appear. | `doing_nothing.rs:451`–`:501` |
| **F11** | The chapter 09 target is labelled "J: quarter turn", which defines the symbol before chapter 10 uses it. | `matrix_logs.rs:342` |
| **F12** | "principal window" no longer flashes back during the 08_4 fade. It stays out of `line`, so it is never faded twice. | `product_rule.rs:204` |
| **F13** | The green product line moves 0.2 down, to y −1.2, so it stops reading as part of step 4. | `product_rule.rs:404` |
| **F14** | Chapter 09's grids stop at clip 4.4: left edge −6.63 at worst, was −7.88. Turned grids and the three small panels lose nothing. | `matrix_logs.rs:92` |
| **F15** | No literal carets: "turning rate of the flow of X = imaginary part of an eigenvalue of X" and "…one logarithm has all eigenvalues −π < Im < π…". | `doing_nothing.rs:66`–`:67` |
| **F16** | The three flow lines fade at 10_4 sentence 1 and the rule's statement takes their rows. The fine print falls from 218 to 197 characters and holds at least 13.2 s (14.9 characters per second). | `doing_nothing.rs:562`, `:599` |
| **F17** | No pops or grid jumps: panels and grids fade in at rest and hand over to the live copy at t = 0, flows rewind instead of snapping, and "Squash the circle" is now a real squash. | `matrix_logs.rs:330`; `doing_nothing.rs:306`, `:444` |

Phrase timing (F4, F10) uses the take's word marks where the narrator has them, otherwise the phrase's character share of its sentence. A phrase that cannot be timed fails the render rather than guessing. Fifteen new tests cover this and the layout: draw 4, product_rule 1, matrix_logs 3, doing_nothing 7.

## Where this differs from the review

| ID | Review proposed | Staged | Why |
|---|---|---|---|
| F1 | Fixed gaps of 0.25 and 0.10 | The same at size 30, scaled with text size | See chapter 11 below: a fixed gap puts the 11–12 stage's size-20 tile on its outline |
| F2 | x = 5.0 | x = 4.95 | 5.0 sits exactly on the lint limit of 6.86 |
| F3 | Rule at x = 3.7 | Rule at 3.85, label moved 0.15 right | Keeps the rule nearer the middle of its window, same gap |
| F7 | About 0.9 right of the axis | Above the axis's end | Stays centred over the panel |
| F10 | Tuned wait, or split the sentence (new take) | Phrase timing, no new take | Keeps every take |
| F15 | `exp(tX)` and `exp X = A` | Reworded without either | Shorter, which F16 also needs. "(complex vectors allowed)" is dropped from the textbook line. |

**Correction to the review's F10 note, which stays frozen:** by the scratch share rule, sentence 3 starts at about 48.0 s, not about 49.5 s. 49.5 s is when the old code's sentence 2 run and word fade finished. The finding stands.

## Shared helpers and chapter 11

Isolated in `shared-draw-helpers.patch`. Checked against canonical `unreachable.rs` and the 11–12 stage as it stood at 10:53Z (RubyRaven notified). Both estimates use the CMR10 table; "½" has no table width and is assumed at 0.75 em.

| Chapter 11 site | Half-width | Effect |
|---|---|---|
| `m_card` | 0.656 → 0.663 | Slight |
| `a_card` (−2, −½) | 0.615 → 0.683 | Spans 2.97..4.33. "A =" ends 2.72, "X =" starts 4.90: both clear |
| `minus_i` | 0.615 → 0.622 | Slight |
| `x_card`, `x_diag` | Unchanged | None |
| 11–12 stage's size-2 tile (size 20) | 0.410 → 0.414 | Inside that stage's own bound of 0.44 and its outline of 0.49 |

`clipped_grid` in chapter 11 uses clip 3.2. No grid line lies on that box, so F5's 1e-9 slack changes nothing there unless a flow lands within 1e-9 of it. The identity grid is unchanged sample for sample in the mirror. Other chapters do not call either helper.

## Checks run

`checks/repair_checks.py` (143 checks pass, output in `.out.txt`) is a set of Python mirrors of the crate's arithmetic: bindings, source strings per finding, card clearances and neighbours, clip samples, label spans, row spacing, reading load, phrase timing, continuity at each static-to-live handover, and the numbers in each new test. `rustfmt --check` passes on the four Rust files; that checks parsing and format only. Both patches pass `git apply --check` against canonical.

Timing was simulated at scratch length and at illustrative ×0.8 and ×1.25. **These are arithmetic, not Oliver measurements.**

| Line length factor | Spiral sweep vs phrase | Stretch label vs "or does not turn" | 10_3 beats end vs line end | Added end hold |
|---|---|---|---|---|
| ×0.8 | starts on it | on it | 22.1 of 24.4 | 6.8 s |
| ×1.0 | starts on it | on it | 27.5 of 30.4 | 5.1 s |
| ×1.25 | starts on it | on it | 34.2 of 37.9 | 3.1 s |

## Not tested

- **Native:** no `cargo check`, test, Clippy, `--lint` or facts check. All Rust is uncompiled; the 15 new tests have never run. Root's first build may find errors I could not.
- **Picture:** no render and no frame seen. Every span above comes from a mirror, the CMR10 table, or a width measured on preview02. No motion has been viewed, so these need eyes: the hand-over from each resting copy to its live copy (a one-frame flicker or z-order change is possible), the squash, and the rewinds.
- **Audio:** nothing heard, no take touched, no synthesis, no fine sync, no independent AV review. Oliver's real timing is unknown here. `narration_units.json` lists units for 00_1 to 06_1 only, and its `script_sha256` (`15eb9ae95f709797`) differs from `script.tsv`; I did not investigate either, and nothing here depends on them.
- **Timeline:** chapter 10 runs about 5 s longer on the scratch rule (a breath, then the hold). Any existing stem for it must be re-scored or `kit.finish` will refuse it. The chapter was already 98.9 s, past the skills' guidance of about 85 s. `FINE_PRINT_S` is a proposal; lowering it shortens the hold.

## Look at these first

Scratch times from preview02; they will shift.

1. **09, about 12–14 s and 44–68 s:** rotation card and `e_card` (F1).
2. **09, about 87 s:** dictionary line (F2), with the wider `pair` card and the ↔ beside it.
3. **10, about 40–47 s, and 10_4's first sentence:** the +π end, closed then opening (F4).
4. **10, about 67 s to the end:** the two labels (F3), the rule's rows, the fine print (F15, F16).
5. **10, about 11–14 s, 20–25 s, 29–37 s:** the whole box when home (F5).
6. **10, about 49–60 s, under the real voice:** spiral then stretch (F10), the finding most exposed to Oliver's pacing.
7. **08, native frames 130 and 300:** the axis against both labels (F6, F7).
8. **10_2's first seconds and 09_4:** the squash and the panel fades (F17).

## Left alone

**Nits, not among the 17:** upright `a`, `b` beside italic `a + bi`; upright "of I"; the stretch ring and spiral dot overlapping by about 2 px (a ring of r 0.13 clears it); the storyboard heading "each stated on screen", though no frame states the grid clip.

**Pacing notes, not defects:** 09_3's stillness; 09_4's static panels before the spin, kept as the payoff; chapter 10's static left half from about 60 s on; no picture of a reversed direction in 10_4.

Also out of scope: the hollow `draw::arrow` question in the 01–04 lane.
