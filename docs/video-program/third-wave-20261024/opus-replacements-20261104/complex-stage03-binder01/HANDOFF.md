# complex-stage03-binder01: strict source binder for complex stage04 (and stage03)

BronzeJaguar, for BlueSalmon; cc LavenderPike. Bead `mx-video-program-program-87do.39`. Date 2026-11-05 (UTC). Narration stays ElevenLabs Oliver.

**State: frozen for root.** This bundle holds a binder, its two bound plans, a sealed oliver10 baseline and checks. It is a source-text audit only:
- Nothing was compiled, tested by Cargo, linted by Clippy, rendered, synthesized or heard.
- No take was made or selected, and no timing was measured.
- Candidate, stage and canonical files, Beads and the TODO are untouched.
- It is **not independent**: I wrote parts of these scenes and earlier repairs, and the binder core is RubyRaven's revision of my older binder.

P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`. EC = `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`.

## The plan for stage04

Root's `EC/combined-repair-stage04-integration01.json` (`295c27e3…`) is stage03 plus LavenderPike's four sealed review-fixes patches. Bound strictly, it gives:

| File (in `bound-plan-stage04/`) | sha256 |
|---|---|
| `explainer_corpus/complex_analysis_visual/docs/narration-direction-map-oliver12.json` | `70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c` |
| `explainer_corpus/complex_analysis_visual/narration_units.json` | `e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6` |
| `source-audit.json` (RubyRaven's binder's audit) | `3f176c7dfbf158ea…` |
| `strict-audit.json` (the checks below, every cue's hook table, findings) | `ee1e77184d9f6ca1…` |
| `delta-from-oliver10.json` | `a20ad76131deca18…` |
| `delta-from-stage03.json` | `9474cd23cde8a988…` |

The binding covers 105 units for 105 cues in 15 chapters, with 13 prediction holds, 10 reading holds and 5 helpers. It reads 334 hooks on 99 cues, and 94 of them wait on a phrase. Those are 93 distinct (cue, phrase) pairs, because 13_3 hooks "so the bump" twice: `until` at `real_integral.rs:436` and `wait_for` at `:440`. The script is `12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3`.

**Name:** `oliver12` is proposed. oliver11 is already three different maps in P (teaching02 `759e0eba…` and cross-review04's two rebound maps). There is no complex oliver12 anywhere in P or EC. The name is root's call; it is one argument.

**Installation is root's.** The two plan files go to the same paths in the crate. Stage04's own `narration_units.json` is still `3c4f53d7…`, which binds the old script `2968b0fd…`. If the bound plan is installed in stage04 and named in a new receipt, the tool accepts it only if it equals this binding byte for byte (check d21; d22 refuses stage03's units installed instead).

## Exactly what changes against stage03

`delta-from-stage03.json` lists every plan difference and marks each one allowed. All of them are the declared ones:
- **Map:**
  - `scripts.complex_analysis_visual`: `1e24aba8…` → `12442480…`.
  - The `sha256` of six rows, and nothing else of them:
    - visual_holds 11_5 (prediction, `counting.rs:440`), 11_6 (reading, `:503`) and 11_8 (reading, `:604`): `c1df16ea…` → `d41d4141…`.
    - helpers `draw.rs` `predict` (1112), `reading_window` (1166) and `precisely_over` (1198): `865599e9…` → `3f1c581d…`.
  - No line, marker, kind or basis moves.
- **Units:**
  - `script_sha256` and `direction_contract.script_sha256` → `12442480…`.
  - `direction_contract.map.sha256` → `70888624…`.
  - In oliver_15_2's text, "though not always everywhere" becomes "though there may be none".

This is what Lavender's HANDOFF ("What the hold rows need") and Mails 3580/3585 predict.

## Exactly what changes against oliver10

oliver10 (`341356cd…`) binds canonical. At the seal, the pinned binder rebuilt canonical's oliver10 map and units byte for byte from live canonical. Those bytes, the audit and the script are kept in `baseline/canonical-oliver10/`, so this delta survives canonical moving.
- **Script:** only 11_7 (root's wording repair; now seven sentences) and 15_2 (F3).
- **Map:**
  - 20 of 23 holds change file hash and line.
  - 03_3's marker also changes: `out.p(C::new(0.0, 0.4))` → `out.p(ASK_AT)`.
  - The three `draw.rs` helpers move 871→1112, 925→1166 and 957→1198, with a new hash.
  - The two `mod.rs` helper rows change hash only.
  - `scripts` changes.
- **Hooks:**
  - 11 cues change their hooks: 02_5, 02_6, 03_2, 03_3, 03_5, 04_3, 04_4, 05_3, 05_8, 05_9 and 11_7. Each call removed or added is listed in `hooks_per_cue`.
  - 78 cues only move lines, and 16 are untouched.
- **Chapter closing tails** (the last cue's waits, breaths and finish):
  - 12 move lines with the same calls; 3 are unchanged.
  - None changes.
  - closing.rs ends 15_4 with `music_breath(1.5)` at 467, `music_breath(3.0)` at 478 and `kit.finish` at 480.

## What the binder checks beyond RubyRaven's

`bind_complex_stage03.py` runs the pinned byte copy `binder/bind_oliver_plan.py` (`26e10809…`, equal to RubyRaven's sealed original, which is also pinned) and refuses on any problem it raises. It then adds:

| Check | What it closes |
|---|---|
| **Pins** (exit 3) | Both receipts (`3f652c2d…`, `295c27e3…`), the binder and its original, the kit's `kit.rs`/`narration.rs`, Lavender's hold delta and four patches, teaching02's map and units patch, and the four baseline files |
| **Receipts** | Every listed file on disk with its hash; nothing unlisted |
| **Declared stage04** | stage04 differs from stage03 in exactly four files, each from exactly its stage03 hash to its declared hash, each named in `changed`. The film code of `counting.rs` and `draw.rs` (everything before the one `#[cfg(test)]` / `mod tests {` block, the split the crate's own tests use) is byte-equal to stage03's. Lavender's F2 strays may be moved aside. Anything else refuses |
| **Script** | Exactly `1e24aba8` (stage03) or exactly stage03 with the one F3 clause (`12442480`, stage04 only). No generic acceptance |
| **S1** | Each phrase a hook waits on occurs once in its cue and starts a word. `Spoken::at` takes the first occurrence; RubyRaven's binder checks only that the phrase is present |
| **S2** | Each cue's hooks in execution order land in spoken order. Loops with derived values are unrolled; for 11_1 that gives s2, s3, s3, s4, s4, end. Helpers expand at their call sites |
| **S3** | All 7 non-literal hook values in the binder's DYNAMIC table are re-derived from the source text and must equal it: the_bump's array, gallery's `named`, NAMED's four phrases, NEEDS_SAID, counting's `i + 2` / `i + 3` over three functions, and walk_and_count's literal at each call site. RubyRaven's table pins only NAMED's last phrase and never compares walk_and_count's call sites |
| **S4** | After each of closing's two post-voice beats' voice-end waits, only comments, the beat and `kit.hold` come before the next say. Nothing arrives |
| **S5** | Each chapter's closing tail against oliver10 |
| **S6** | Every cue whose words changed since oliver10 (11_7, 15_2) must match its reviewed hook table, read against source and text |
| **Cross-checks** | The map equals Lavender's 25-row F1 delta applied to teaching02's oliver11, carried to stage04 by the F3 script hash and the six rows' hashes from stage04's receipt, byte for byte. The units, pointed at teaching02's oliver11 with F3 undone, equal teaching02's patched plan `3989e185…` |
| **OUT** | Never overwrites, and refuses `explainer_corpus/`, both stage roots and other P bundles |

The cross-checks verify my binding against two proposals made without this tool. No mapping was copied from them: the map is derived from source, and the result equals theirs.

### The 11_7 and 15_2 hook tables (reviewed)

- **11_7** (`counting.rs`):
  - :525 `wait_for "complex polynomial"` (s0)
  - :529 `sentence(1)`, :532 `wait_sentence(1)` and :536 `until "which leaves"` (s1, "Divide by its leading coefficient…")
  - :554 `wait_for "with z"` (s2)
  - :563 `wait_sentence(3)` ("On a huge circle…")
  - :571 `wait_sentence(4)` ("Picture z to the n as a walker…")
  - :573 `sentence(6)`, floored at 5 s ("So there are exactly n roots inside…")
- **15_2** (`closing.rs`):
  - :321 `wait_sentence(1)`
  - the strip loop :329–333: `wait_sentence` 2, 3 and 4, then `wait_for "and it turns a real integral"` (s4), then `wait_sentence(5)`
  - F3 leaves each in its sentence; sentences 0–4 are byte-equal.

## How the one script delta is accepted

Only two scripts are accepted: stage03's exact bytes, and stage03's with one exact substitution in 15_2. That substitution's hash is fixed in the code (`12442480…`) and matches Lavender's `F3-15_2-clause.patch`. The F3 script is accepted only through stage04 mode, whose pinned receipt names `script.tsv`. A second script change refuses (d16), and so does F3 with the storyboard left behind (d17). A later script change needs a new declaration in a new bundle.

## Findings, each explicit

1. **Lavender F1:** addressed by this binding. Their delta and this tool agree byte for byte, for both stage03 and stage04.
2. **Stale plan in the crates:** stage03's and stage04's `narration_units.json` are still `3c4f53d7…` (old script). Root's installation replaces it.
3. **stage03 receipt:** `src/plate_layout.rs` (new, `f450c320…`) is not in its `changed` list. Recorded, not refused.
4. **Lavender F2:** stage04 still holds `counting.rs.orig`, `mod.rs.orig`, `one_derivative.rs.orig` and `.ee/hook-state/…`. No binder reads them. Root's to move aside before promotion; the tool accepts either state (d20).
5. **RubyRaven's binder, blind spots** (closed by S1–S3 above, not patched in their sealed bundle):
   - phrase uniqueness, word start and order are unchecked;
   - reading_color's `from` pins only the violet line;
   - walk_and_count's call-site literals are not compared.

   d08–d13 plant each one: their binder exits 0 and this tool refuses.
6. **15_2's recap strip:**
   - The last panel waits on sentence 5, then fades its picture (0.5 s) and caption (0.4 s).
   - After the voice ends come the 1.2 s beat (`closing.rs:349`) and the kit's 0.6 s breath (`BREATH`) before 15_3.
   - Whether that is enough time to read the strip is root's and Lavender's call. F3 shortens sentence 5 by four characters.
7. **15_2 needs a new Oliver take.** The current take's words no longer equal the script. Root's call when.
8. **F5's new bound is not fragile:** `picture_cells` rounds `8.4 / (0.035 / 4)` to exactly 960 columns.
9. **Not this bundle's:** Lavender also noted a stale doc comment at `draw.rs:615–616`.
10. **My own guard, fixed before the seal:** my first film-code guard split each file at its first bare `#[cfg(test)]`.
    - `draw.rs` has a `#[cfg(test)] pub fn contains` at line 416, so that split compared only lines 1–415 of 1,285.
    - `two_planes.rs`, `one_derivative.rs` and `real_integral.rs` also carry test-only items near their tops. The independent phrase scan in the checks failed on exactly this and exposed it.
    - The guard now splits at the one `#[cfg(test)]` / `mod tests {` block. It refuses a file with none or several.
    - `draw.rs`'s film code is byte-equal to stage03's on all 1,285 lines. d19b plants a comment change below line 416 with the declared hash forged to match, and the tool refuses it.
    - The bound plans did not change.

## Checks

`checks/run_checks.py FRESH_SCRATCH` writes only into its scratch folder. It reads the repository and changes nothing in it. A clean rerun prints `checks/outputs/run_checks-01.txt` (`f9837e46…`) byte for byte. Two runs into two different scratch folders gave identical output: the scratch path prints as `<scratch>`.

**64 checks, 0 failed.**

| Section | Checks | What |
|---|---|---|
| **A** | 17 | Every pinned input, recomputed |
| **B** | 2 | `--check` on both sealed bound plans: in step |
| **C** | 16 | Recomputed without importing either binder. See the list below |
| **D** | 29 | Real mismatches in scratch mirrors. See the list below |

**Section C** covers:
- stage04's receipt against disk (55 files);
- the four-file delta at the declared hashes;
- the script as stage03 plus F3 only, with 15_2's first five sentences byte-equal;
- the storyboard lines 134, 138, 151 and 284;
- the film code of `counting.rs` and `draw.rs`;
- the units against the script and the map's bytes;
- every map row's hash and marker line in stage04;
- the six-row delta from stage03 and the 25-row delta from oliver10;
- a separate phrase scan, which finds the same 93 (cue, phrase) pairs as the strict audit, each once in its cue and at a word start;
- 11_7's and 15_2's reviewed hook lines;
- no absolute path in any output.

**Section D**, with a fresh mirror for each test:
- d00: the unchanged control binds to the sealed bytes.
- d01–d04: stale pins exit 3.
- d05–d06: stage03's real script and canonical's real `counting.rs` exit 1.
- d07: the forged control binds.
- d08–d13, d15, d16, d18: semantic plants. RubyRaven's binder alone exits 0 on each, and this tool refuses with that check's message (S3 ×2, S6, S1, S2, S4, cross-check, script, undeclared file).
- d14, d17: both binders refuse.
- d19, d19b: film code changed with the declared hash forged. Refused.
- d20–d22: strays moved aside and the right installed plan bind; the wrong installed units refuse.
- d23–d27: the OUT, map-version, overwrite and stale `--check` guards.

## Reproduce

From the repo root:

```sh
python3 -I -B P/complex-stage03-binder01/bind_complex_stage03.py --check                 # stage04, bound-plan-stage04
python3 -I -B P/complex-stage03-binder01/bind_complex_stage03.py --stage stage03 --check # stage03, bound-plan-stage03
python3 -I -B P/complex-stage03-binder01/bind_complex_stage03.py --out EC/combined-repair-stage04-bound-plan01
```

Replace `P` and `EC` with their full paths above. The third command binds into a new folder: OUT may not lie inside a stage root, because an extra file there would break that stage's receipt check. Exit codes:
- 0: bound or in step
- 1: refused (every problem listed) or a stale OUT
- 2: usage error, or OUT refused
- 3: a pinned input changed

The tool also reports whether live canonical still rebuilds to the sealed oliver10 baseline. At the seal it does, byte for byte.

## Limits

- Source text only. Hook positions are characters in the cue text. Every word moment is an estimate until Oliver word marks exist; there is no measured timing, fine sync or listening.
- Reading and prediction seconds are authored pauses, not measurements.
- Not native, picture or audio acceptance. Cargo, Clippy, the glyph probe, render, take, listening and the final cut remain root's.
- The F4/F5 tests were not compiled here either.
- `bound-plan-stage03/` is evidence for the superseded stage03 15_2. Its map is labelled `oliver12-stage03` so that no two different oliver12 maps exist.

## Files

- `HANDOFF.md`: this file.
- `bind_complex_stage03.py`: the binder.
- `binder/bind_oliver_plan.py`: RubyRaven's binder, a byte copy of `P/complex02-05-cross-review04/binder/bind_oliver_plan.py`.
- `baseline/canonical-oliver10/`: oliver10's map, units, audit and script, rebuilt from canonical at the seal.
- `bound-plan-stage04/`: the plan for installation and its audits.
- `bound-plan-stage03/`: stage03 evidence.
- `checks/run_checks.py`, `checks/outputs/run_checks-01.txt`: the checks and their sealed output.
- `SHA256SUMS`: this bundle.
