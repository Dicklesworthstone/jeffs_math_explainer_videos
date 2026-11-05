# complex-stage03-cross-review01: the merged complex candidate, combined-repair-stage03

LavenderPike, for BlueSalmon. Bead `mx-video-program-program-87do.39`. Date 2026-11-05 (UTC). Narration stays ElevenLabs Oliver.

**State: source review only.** Nothing was compiled, tested by Cargo, rendered, transcribed, synthesized or heard, and no picture of stage03 was looked at. Root's native build of this merge (206 tests) is root's evidence; I did not rerun it. No take was selected. No canonical file, candidate file, Bead or commit was touched: everything I wrote is in this folder or in my scratch space.

Reviewed: `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-repair-stage03/` as its receipt `combined-repair-stage03-integration01.json` (`3f652c2d…`) hashes it. All 55 files are still those bytes (`checks/outputs/stage03_checks.txt`, section A). Every file I read is hashed in `inputs.sha256`.

## Verdict

**The mathematics and the teaching hold in every cue I reviewed, and every hook is said, once, in the voice's order.** Chapter 11's acted division is correct, and its turning field takes over from the painter on exactly the painter's colours. The 02–05 teaching beats land on the words that name them. The 13–15 repairs and the new 14_3/15_2 wording are integrated as staged.

**One integration gap needs fixing before any narration plan or binder runs on this tree (F1).** The candidate's `narration_units.json` still holds the old 11_7 words and the old script hash. Its hold map binds 20 of 23 rows to bytes that are gone. A staged fix exists for the plan, and the exact map delta is in `proposed/`.

The other five findings are Low. One is hygiene before promotion (F2), one is wording for root to decide (F3), two are test guards (F4, F5) and one is storyboard text (F6). Nothing found blocks the native qualification already done.

## Independence

- **Not mine, reviewed independently of their authors.** Chapter 11's teaching02 (WhiteSwan) and its corrections C1–C3 (BronzeJaguar). Root's 11_7 wording repair. The 02–05 repair03 (BronzeJaguar) with cross-review04's rebinding (RubyRaven). The 13/14/15 radius, pole-order and closing patches (BronzeJaguar). The glyph-spacing patch (BronzeJaguar). The 06–09 repair03 (WhiteSwan), which I checked only for presence.
- **Mine, so self-review only.** The plate core in `draw.rs` and `plate_layout.rs` (complex-color-repair03), with root's selection of `PLATE_CELLS4`. I also cross-reviewed the `main.rs` plan budget (engine-geometry-cross-review01). F4 sits where WhiteSwan's `Turning` meets my `PlateStyle`. F5 is my own test.
- **Limits of that independence.** The authors and I are the same model. The project memory index loaded at the start of my session holds one-line notes written by the sessions that made several of these bundles. So this review is independent of authorship, not of the model or that memory.

## Findings

| ID | Severity | Where | Defect | Narrow fix |
|---|---|---|---|---|
| F1 | Medium (gate) | `narration_units.json:3`, `:7`, `:552`; `docs/narration-direction-map-oliver10.json` | The plan pins script `2968b0fd…` and the old six-sentence 11_7. The map binds 20/23 hold rows and 5/5 helper rows to bytes that are gone | Apply teaching02's `narration-units.patch`. Build the map from teaching02's oliver11 plus `proposed/hold-rebind-stage03.json`. Point the plan at it |
| F2 | Low | `src/chapters/{counting,mod,one_derivative}.rs.orig`, `.ee/hook-state/` | `patch` backups and hook state sit in the crate, and the receipt hashes them as candidate files | Move the four aside before promotion and leave them out of the promotion list |
| F3 | Low (root's call) | `script.tsv:104` 15_2 sentence 5 | "though not always everywhere" now hangs on "at most one", so it can be heard as uniqueness failing | Change that clause to "though there may be none". This needs a new 15_2 take |
| F4 | Low (latent) | `counting.rs:654`, `:668`; `draw.rs:800`, `:757` | `Turning` repaints through `portrait_colors`, which matches `portrait`'s colouring only while `PLATE.samples == 1`. No test says so | Add a one-line test guard |
| F5 | Low (mine) | `draw.rs:1607`–`1612` | The budget test calls a 6.4 square "the widest picture", but 05's 8.4-wide plates have 960 columns, past its 800 | Test the planes the film paints |
| F6 | Low (docs) | `docs/storyboard.md:134`, `:138`, `:151` | Three picture cells quote strings the film no longer draws | Quote the current strings |

Paths in the table are under `…/combined-repair-stage03/explainer_corpus/complex_analysis_visual/`.

### F1. The narration plan and the hold map do not bind stage03

**The plan.** `narration_units.json` (`3c4f53d7…`) still says `script_sha256` `2968b0fd…` at lines 3 and 7, but stage03's `script.tsv` is `1e24aba8…`. Unit `oliver_11_7` (line 552) still has the six-sentence 11_7 ("every polynomial has roots. Take one of degree n…"). It is the only unit whose words differ from the script. The retained pair `counting-oliver02` was requested on the new words; its `11_7.txt` equals the script's 11_7. So a guard that compares a take's text with its unit would refuse the retained takes, and a plan built from this file would name words the film no longer says. The receipt's `hold_rebind: false` covers the map, but it does not name the plan.

**The map.** `docs/narration-direction-map-oliver10.json` (`341356cd…`) binds rows by file hash and line. Against stage03, 20 of 23 `visual_holds` rows and all 5 `helpers` rows fail, because the file hash moved and the marker is off its line (`holds_check.txt`). teaching02's `holds/narration-direction-map-oliver11.json` (`759e0eba…`) has the right script hash and the right counting lines (440, 503, 604). But it pins teaching02's `counting.rs` `dba35730…`, which C1–C3 and the glyph patch have since changed, along with every other file's pre-repair hash.

**Fix, three mechanical steps:**

1. Apply `complex11-counting-teaching02/patches/narration-units.patch` to the plan. On a scratch copy of stage03's plan it applies cleanly, gives `3989e185…` (the hash teaching02's handoff names), and all 105 units then equal the script (section E).
2. Make a new map from teaching02's oliver11 with the 25 rows in `proposed/hold-rebind-stage03.json` set: 20 `visual_holds` and 5 `helpers`, each a new `sha256` and `line`. Give it a new name. "oliver11" already exists three times: teaching02's `holds/`, and RubyRaven's `complex02-05-cross-review04/rebound/{canonical,candidate}/`. BronzeJaguar, who is binding stage03 (Mail 3536), will propose a free name. One marker changes, 03_3's, to `let asked = predict(&kit, stage, &line, &ASK, out.p(ASK_AT))?;`, as repair03's handoff states. After the delta all 28 file-bound rows bind stage03's bytes (`rebind_delta.txt`). Overrides, kinds and `basis` texts are untouched; I did not re-review the `basis` texts against `READING_WINDOWS`.
3. Point the plan's `direction_contract.map` at that new map and its hash. teaching02's patch points it at oliver11 `759e0eba…`, which does not bind.

`checks/rebind_delta.py` does step 2 in memory and checks every row, so root's binder can be compared against it.

### F2. Stray files in the candidate tree

`counting.rs.orig`, `mod.rs.orig` and `one_derivative.rs.orig` are byte-equal to partial06's before-bytes of those three files (`cdc154ec…`, `a1353544…`, `84ed1074…`). They are exactly the three files whose glyph-patch hunks applied at an offset, which is when `patch` keeps a backup. `.ee/hook-state/session_start_orient.last.json` is session-hook state. None of the four is compiled. All four are among the 55 files the stage03 receipt hashes, so a tree copy into canonical would carry them along (section B). Move them aside, not delete them, and leave them out of the promotion list.

A related receipt note: `src/plate_layout.rs` (mine) is new in stage03. It is absent from both stage02 trees, and the receipt hashes it under `files` but leaves it out of `changed`, which lists only `main.rs`, `counting.rs` and `draw.rs`. The receipt should name it as added. BronzeJaguar saw this too (Mail 3536).

### F3. 15_2's last clause after the 14_3 change

Root's Mail 622 changed 14_3 from "only one way to extend" to "at most one holomorphic extension". That change is right, because the old words implied an extension always exists. 15_2 was changed to match: "It allows at most one extension to a given connected region, though not always everywhere, and along different paths a continuation can branch."

In the old sentence, "though not always everywhere" said the extension does not always reach everywhere: 14_7's "no way across at all". Now it follows "at most one", and it can be heard as "uniqueness does not always hold". That is the opposite of 14_3.

**Proposed:** "…to a given connected region, though there may be none, and along different paths a continuation can branch." Section G checks the rest of the cue is untouched: still six sentences, no digit, and no 15_2 hook reads sentence 5's words, since the last panel waits on sentence 5 by index. The caption "at most one extension; paths can branch" stays. The change costs one Oliver take of 15_2, plus `docs/storyboard.md:284` and the plan's 15_2 unit. If root would rather not retake, the sentence is defensible as it stands; this finding is about how it can be heard.

### F4. The turning field assumes one sample per cell

`Turning::build` (`counting.rs:653`) builds its field with `portrait`, which colours through `plate_colors(…, style.samples)` (`draw.rs:800`). Each repaint at `:668` uses `portrait_colors`. The two agree only because `plate_colors` falls back to `portrait_colors` at one sample (`draw.rs:757`). `PLATE_CELLS4` inherits `samples: 1` from `PLATE_NOW` through `PLATE_BLEND` and `PLATE_TILE` (section C).

Today the takeover is exact. The painter's crossfade ends with `b = 1` painting the quintic itself. The field uses the same `PLATE` cells (C1), the same enhance 1.0, and `turned_by(0)` is exactly 1, so the turn back repaints 11_6's colours bit for bit.

If root ever picks a sampled style, the first repaint would jump and the turn back would no longer land on 11_6's colours, and no test would notice: no counting test reads `PLATE` or `samples`. I wrote `PlateStyle`, so this half is mine.

**Proposed guard, test only, uncompiled:**

```rust
    /// `Turning` builds with `portrait` and repaints with `portrait_colors`: one colouring only at one sample.
    #[test]
    fn the_turning_field_repaints_as_the_plate_paints() {
        assert_eq!(super::super::draw::PLATE.samples, 1);
    }
```

An alternative is to make `plate_colors` `pub(crate)` and call it at `:668` with `super::draw::PLATE.samples`. That changes film code, so I would take the guard.

### F5. My budget test bounds the wrong picture (self-review)

`draw.rs:1607` `a_full_picture_stays_within_the_strip_budget` says "the widest picture the chapters draw: 6.4 × 6.4 scene units" and asserts `cols <= 800`. Under `PLATE_CELLS4`, 05's `trig_plane()` (`gallery.rs:130`) and `gamma_plane()` (`gallery.rs:140`) are 8.4 wide, which is 960 columns. Chapter 14's `gamma_plane()` (`continuing.rs:101`) is 7.5 wide, 857 columns. All three are painted (section D).

There is no runtime risk that I found. The geometry budget counted all three (`engine-geometry-budget01/checks/demand.py`, its rows "05 trig", "05 gamma" and "continuing gamma"). The tallest picture has 731 rows, well under `GROUP_TRANSFORM_LIMIT` (2,000). The test just checks a case that is not the worst.

**Proposed, uncompiled:**

```rust
        // the pictures the chapters paint at their widest (05's sin and Γ) and tallest (04's square)
        for p in [
            super::super::gallery::trig_plane(),
            super::super::gallery::gamma_plane(),
            Plane::new(v(0.0, 0.0), 6.4, (-2.0, 2.0), (-2.0, 2.0)),
        ] {
            let (cols, rows) = picture_cells(&p, &PLATE);
            assert!(rows <= 800 && cols <= 960, "{cols} × {rows}");
            assert!(rows < explainer_kit::GROUP_TRANSFORM_LIMIT);
        }
```

### F6. Storyboard cells that quote strings the film does not draw

| Row | The storyboard quotes | The film draws |
|---|---|---|
| `storyboard.md:134` 04_2 | "band: size × 1.69" | `BAND` "next band: size × " plus `color.band_ratio` |
| `storyboard.md:138` 04_6 | "bump: where are the poles? how many turns?" | `ASK` "where are the bump's poles?" / "how many turns at each?" |
| `storyboard.md:151` 05_7 | "series converges to the right" | `CONVERGES` "series converges: Re s > 1" |

This is the same kind of drift repair03's handoff listed for root. Nothing reads these cells. Quote the current strings.

## What holds, and how I know

### Chapter 11

- **Mathematics.**
  - On |z| = 1, |z⁵ + 1| ≤ 2 < 3 = |−3z|, and 2 is reached at z = 1.
  - 2iz⁵ − 6iz + 2i = 2i(z⁵ − 3z + 1), and division by 2i ≠ 0 keeps exactly the roots: "0 ÷ 2i = 0, and nothing else lands on 0" states both directions.
  - "Nonconstant" is said and drawn. A constant makes no turns (`counting.rs:1683`).
  - On |z| = 3 the walker is 243 and the leash at most 10.
  - 11_8's z⁸ samples all land on 1. L = 8, so 8 steps reach 2π·8/8 ≫ |a| and are refused, while 64 steps reach 0.785 < 1 and give 8.
  - The 11_2 example (z − 1)/(z + 1)² counts +1 − 2 = −1.
- **The acted division keeps to its words.** On "complex polynomial" (sentence 0) the row appears and the field turns. On sentence 1 the turn goes back, landing by "which leaves". "with z" swaps the row (C2). Sentence 3 brings the wide plane, sentence 4 the dog, and the walk ends on sentence 6. The tests at `counting.rs:1640` and `:1821` pin that order and the sentence count, 7.
- **The takeover is exact,** as F4 sets out. The curve also starts point for point (`counting.rs:1768`).
- **11_3's inset on sentence 4 is as designed.** The storyboard says "On the last two sentences", and "The count is always a net total" is the right place for "+1 − 1 = 0".
- **Patches.** teaching02, C1, C2, C3, root's sentence shift and script patch, and the glyph patch are all present. Each hunk not found verbatim was later rewritten by a named patch (`patch_presence.txt`):
  - C1: `picture_cells(&inp, &PLATE)`
  - C2: "with z" and its storyboard row
  - C3: the 0 name on zero
  - teaching02, over root's shift: the `SQUASHED` text and the claim's row
- **The retained 11_7 pair.** `counting-oliver02/11_7.txt` equals the script's 11_7. Its downloads record says `listening: false`, `fine_sync: false`, `selection: null` (section F). No take is chosen here.

### Chapters 02–05

- **Mathematics.**
  - (1 + i)² = 2i, with length √2 and an eighth of a turn.
  - z² takes |z| = ½ to |w| = ¼ twice. Off-axis lines become parabolas; the real axis folds onto the positive ray and the imaginary axis onto the negative one. Right angles are kept except at 0.
  - 1/z on the unit circle is the conjugate.
  - eˣ⁺ⁱʸ has size eˣ and direction y.
  - z²'s colours run twice around 0, and 1/z's backwards. 1/(1 + z²) has simple poles at ±i, on the unit circle.
  - The band ratio e^{π/6} ≈ 1.688 against phase lines every 2π/12 makes exp's tiles exact squares.
  - sin i ≈ 1.175i and sin 3i ≈ 10.02i. tan → ±i far above and below the real line.
  - Γ(n + 1) = n!, with poles at 0, −1, … and no zeros.
  - ζ's series converges for Re s > 1; ζ has its pole at 1 and trivial zeros at −2, −4, …. The strip up to Im s = 32 holds exactly the four zeros it labels.
- **Beats.**
  - 02_5 sweeps the old angle on "an eighth of a turn" without answering the question.
  - 02_6 pulses the lengths on sentence 2 and the angles on "and angles add", then clears the grid on "Every complex derivative".
  - 03_2 shows one point (1.4 at 30° to 1.96 at 60°): length on "squares each length", angle on "doubles each angle".
  - 03_4's eight sentences each have their picture, including the new axes fold and the rays row.
  - 03_5's flip chords come on "each point flips".
  - 04's needle stands on arg f under the walking dot. 04_3 runs it twice around the wheel, 04_4's order card comes on "twice for a zero of order two", and 04_5 runs backwards with a −1 count.
  - 05_8's card comes on "is the Riemann hypothesis".
- **05_8 no longer says "trillions".** That is root's Mail 622 rule ("a count needs a dated primary source checked by root first"), recorded at `storyboard.md:152`. It is a decision, not lost depth.
- **Patches.** repair03 is present in all four files (54/54 hunks) and cross-review04's rebinding in 5/5.
- **Removed lines flagged as still present.** The scan flags eight across the 19 patches: five in repair03, one in C2 and two in root's sentence shift. Six are generic `wait_sentence(stage, 1)` and `(stage, 2)` calls, which stand in many cues. The other two are the fades of `out_frame` and `links` in `gallery.rs`. I traced those by hand: each survives once in another cue (`:407` in 05_1 and `:820` in 05_7), while repair03 removed the 05_2 and 05_9 copies.

### Chapters 13–15

- **13.** The radius patch is present (9/9) and its hooks are said once. Chapter 13 has no wording change. WhiteSwan's 13-14 cross-review found no fault; I did not redo its geometry.
- **14.**
  - 14_3's seven sentences each get their row: the region C ∖ {1} is connected, and the first re-centred disk (−0.5, radius 1.5) reaches −2, so it pokes outside the unit disk. The pole-order patch is present (7/7): every fill reaching the pole is rooted under its mark before it fades in.
  - Elsewhere in chapter 14:
    - Σzⁿ is 2 at ½, and 1/(1 − 1.2) = −5.
    - 1 + 2 + 4 + … is "−1" only as 1/(1 − z) at 2. ζ(−1) = −1/12 is a value, not a sum.
    - √ comes back as −1, and log gains 2πi.
    - Σz^{2^k} has the unit circle as a natural boundary.
    - Γ(z) = Γ(z + 1)/z.
- **15.** The five panels come on sentences 2, 3, 4, "and it turns a real integral" and 5. The closing patch is present (14/14).

### Protected depth against the script root reviewed (Mail 622)

Every changed cue keeps or adds its explanation:

- 03_4 adds the axes fold.
- 05_8 drops the count by root's sourcing rule.
- 11_3 adds "net total" and the unwinding example.
- 11_6 corrects "three z" to "minus three z".
- 11_7 adds "nonconstant", the acted division and "positive integer n".
- 11_8 adds the derivative bound that makes the count certain.
- 14_3 replaces an existence claim with "at most one", a forced path and "not always so".
- 15_2 follows 14_3 (F3).

### Integration of my own parts (self-review)

- `plate_layout.rs` is byte-equal to mine (`f450c320…`).
- `draw.rs` differs from my complex-color-repair03 candidate only by root's selection: `PLATE = PLATE_CELLS4`, the `allow` dropped from the now-used `PLATE_CELLS4`, and two test assertions.
- Every `picture_cells` caller passes a style: `counting.rs:656`, `draw.rs:799` and `:851`.
- `main.rs` declares the `1 << 22` retained-knot budget I cross-reviewed.
- Chapters 06–10 were outside this review. They hold WhiteSwan's 06–09 repair03 (all hunks present) and earlier picture repairs, some of them mine.

## Checks

Run from the repo root: `python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-stage03-cross-review01/checks/run_checks.py <fresh-scratch-dir> --verify`. It writes only into the scratch folder and compares each output with the sealed one. Standard library, plus the system `patch` for one trial on a scratch copy.

| Check | Result | What it cannot show |
|---|---|---|
| `hooks_scan.py`: every `say`, sentence and phrase hook against `script.tsv` | 105 cues said once, in script order, no digits. One problem row, `counting.rs:1148`, is the scanner crediting the `approach_pole` helper to the file's last cue; section I shows 11_4 is its only caller and "There the count" opens an 11_4 sentence | Timing. Oliver's lengths are unmeasured |
| `patch_presence.py`: each integrated patch's new side in the stage | 19 patches; every hunk present or rewritten by a later named patch | That the code compiles or draws |
| `holds_check.py`: map rows and the plan against the bytes | oliver10 and oliver11: 20/23 hold rows stale. Plan: wrong script hash, 11_7 stale | What a binder would decide |
| `rebind_delta.py`: the F1 map delta, applied in memory | 25 rows; all 28 file-bound rows then bind | The `basis` texts' durations |
| `stage03_checks.py`: sections A–I | All PASS; each FAIL row would be a failed premise | Anything needing rustc, a render or audio |

## Not done, and the limits

- No `cargo`, `rustc`, Clippy, render, frame, ASR, TTS or listening. The proposed Rust in F4 and F5 is uncompiled.
- No timing on Oliver. One beat to look at first: 02_6's rule and first underline pulse take about 1.05 s after sentence 2 starts, and "and angles add" is 17 of its 32 characters in.
- Chapters 06–10 and 12 were not reviewed beyond the hook scan and patch presence.

## Files

- `REVIEW.md`: this file.
- `proposed/hold-rebind-stage03.json`: the F1 map delta, oliver11 base.
- `inputs.sha256`: every file read.
- `checks/`: the six scripts and `checks/outputs/`.
- `SHA256SUMS`: the seal.
