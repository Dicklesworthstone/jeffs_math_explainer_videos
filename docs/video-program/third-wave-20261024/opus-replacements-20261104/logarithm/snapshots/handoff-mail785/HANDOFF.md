# Logarithm rebuild: handoff to root

| | |
|---|---|
| Bead | `mx-video-program-program-87do.63` (root-owned; not touched) |
| Author | RubyRaven, Claude Code Opus 5.5, staging only |
| Date | 2026-11-04 |
| Reservation | 296 on `docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/**`, renewed to 09:37Z |

## Scope, as carried out

I rebuilt the logarithm film as a substantial motivated explanation in native Rust
FrankenManim. It covers:

- Multiplicative measurement: musical pitch and the keyboard.
- The ratio ruler, and the slide rule as a worked machine (2.5 × 3.2, then 4 × 6 off the
  end, through the other index, then the decimal point).
- The dial that forgets decades.
- The exponential and ln on the reals, with domain and inverse identities.
- The complex exponential folding the plane, the column of logarithms, and the lift
  around a loop with its no-continuous-log proof.
- The branch cut and the principal Log, with exact domains.
- The product rule with its lap count, and the −1 = 1 puzzle.
- Matrices as flows; the identity's many logarithms and the principal matrix logarithm.
- Real matrices with no real logarithm: det < 0, then diag(−2, −½), then Culver's rule.
- Complex matrix logarithms.
- A recap, and a take-away puzzle.

It is 13 chapters, 68 cues, 317 sentences and about 3,800 words, with no runtime
ceiling.

I did not edit any canonical file, Beads, TODO, catalog or shared file. I ran no Cargo,
render, TTS, ASR, commit or push, and launched no agents.

## Narration: ElevenLabs Oliver (acknowledged)

I acknowledge the owner requirement (BlueSalmon Mails 698, 726 and 753; 726 and 753
acked in Mail).

- All narration is **ElevenLabs Oliver**. The Michael and FrankenTTS defaults in the
  skills do not apply.
- I launched no synthesis.
- I did not touch the retained takes under
  `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-20261102`.

What the orchestrator needs to synthesise:

- **All 68 cues are new text.** I rechecked the final script against the 244 retained
  receipt files. The text-bearing fields hold 17 distinct texts (an earlier, broader
  extraction found 30). None matches any of the 68 cues or 317 sentences exactly, so no
  take can be reused by binding.
- Since root's review (Mail 624), 15 cues were reworded: 00_3 00_4 00_5 01_5 06_5 06_6
  07_2 07_4 07_5 10_3 11_2 11_3 11_4 11_5 12_3. The reviewed copy is preserved at
  `explainer_corpus/logarithm_mysteries/docs/script-rev1-reviewed-mail624.tsv`. If any
  pre-review cue was already queued, the 15 above supersede it.
- Spoken text contains no digits; numbers are spelled as they should be read.
- Sentence waits are indexes into `script.tsv` sentences (the kit splits sentences after
  ". ", "? " or "! "). Rewording a cue in a way that changes its sentence count breaks
  that chapter's waits. `docs/check_cues.py` catches it.

## Files (all under `explainer_corpus/logarithm_mysteries/`)

| candidate | integrates as | note |
|---|---|---|
| `script.tsv` | canonical `script.tsv` | 68 cues |
| `src/main.rs` | canonical `src/main.rs` | same `Explainer` contract; adds `mod model;` |
| `src/facts.rs` | canonical `src/facts.rs` | same signature `facts() -> Result<Facts, String>`; 81 keys |
| `src/model.rs` | new | numerics: positions, complex numbers, lifts, 2×2 matrices, `mexp`, flows |
| `src/chapters/mod.rs` | canonical `src/chapters/mod.rs` | 13-chapter registry, `TEXT` supplement |
| `src/chapters/draw.rs` | new | shared props and colour roles |
| `src/chapters/{ratio_ruler, building_ruler, slide_rule, folded_decade, undoing_growth, complex_turns, lifting_loops, branch_cut, product_rule, matrix_logs, doing_nothing, unreachable, what_it_forgets}.rs` | new | chapters 00–12 |
| `src/chapters/thumbnail.rs` | canonical `src/chapters/thumbnail.rs` | the column of logs of −1 over its shadow |
| `oracle.py` | beside the crate | Python mirror of facts.rs (`python3 -I oracle.py`) |
| `docs/check_cues.py` | review tool | cue order, sentence indexes, registry, digits |
| `docs/storyboard.md` | storyboard | beats, holds, colour roles, sources, drawing aids, take-away answer |

Kept canonical and not mirrored: `Cargo.toml` and `src/score.rs`.

At integration, root should move these canonical chapter files aside; their keys
disappear:

- `bookends.rs`, `real_inverse.rs`, `winding.rs`, `product_law.rs`, `matrix_log.rs`.
- `branch_cut.rs` (same name, different content).

Root regenerates the narration units and bindings from the new `script.tsv`.

## Checks run here, with results

All checks were run from `explainer_corpus/logarithm_mysteries/`.

1. `python3 -I oracle.py` prints 81 keys, matching the set facts.rs adds. The Rust side's
   own `self_check` and tests have not run.
2. `python3 -I -B docs/check_cues.py` gives **OK**:
   - 68 cues and 317 sentences, every cue said once and in order.
   - Every `until_sentence` index is valid, increasing and on the current line.
   - Registry keys are ordered and cover every cue prefix.
   - No digits in spoken text.

   It also lists string literals containing digits for review; all are labels, indexes
   or fact-formatted.
3. `rustfmt --edition 2024 --config max_width=120 --check src/chapters/mod.rs src/facts.rs src/model.rs`
   is clean. Through `mod.rs` this parses and formats every chapter module.
   `src/main.rs` was parse-checked in a scratch copy beside the canonical `score.rs`. The
   only diff reported is in that canonical `score.rs`, which is pre-existing and not
   mine.
4. Python prototypes of the chapter 11 and 12 numerics. They checked:
   - det e^{tX} at t = −2, 1 and 3 (0.670, 1.221, 1.822).
   - The square's extent under the flow, against the grid clip.
   - The cube's projection and the text column's clearances.
   - Text-width estimates.
   - Margins for the half-turn-answer tests: other axes miss by at least 0.138, and
     slower spins by at least 0.036.

**For root (not run by me):**
`cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` (80
`#[test]`s), then the kit's glyph probe and storyboard check, then a draft render.

## Gaps (explicit; none of these has been done)

- **Uncompiled.** No Cargo was run, so type, borrow and clippy errors are likely
  somewhere in 8,265 lines. APIs were checked by reading the kit and pin sources, not by
  compiling.
- **Unrendered.** No frame has been seen. Layout comes from coordinates and
  width-estimate tests, not pixels.
- **No synthesis, listening, fine-sync or independent AV review.** ASR has not been run
  and would not count as listening. The holds (THINK = 4 s, nine predictions) and every
  pacing estimate are proposals.
- **TeX at pin a399d626.** The film keeps to the corpus's commands. `\operatorname`,
  `\tfrac`, `\text{...}`, `\Rightarrow`, `\neq`, `\quad` and `\qquad` need the glyph
  probe. `\phantom`, `\cdots` and `smallmatrix` are deliberately avoided.
- **`draw::matrix_card` geometry corrected.** It was about three times too large. This
  changes the cards in 09, 10 and 11; none have been seen.
- **Performance.** `always_redraw` is used throughout: clipped grids in 10 and 11, the
  live readouts, the dots on four orbits in 12, and the cube in 12.
- **Sources.**
  - Culver (Proc. AMS 17 (1966) 1146–1151, doi 10.1090/S0002-9939-1966-0202740-6):
    volume, pages and theorem wording were matched only against secondary statements;
    the paper itself has not been read.
  - Higham Thms 1.27 and 1.31: carried from the earlier review round, not re-read here.
- **Small literals.** `matrix.half_turn_speed` is a literal `"0.5"` in facts.rs, not
  derived. In 11_2 the "t = 1" and "t = 3" labels hard-code times that match the fact
  keys' names.
- **Hook artifacts, not mine.** `.ee/hook-state/` directories (session-hook state)
  appeared at the crate root and in `src/chapters/`. I left them in place. Root should
  exclude them at integration.

## Strongest unresolved teaching defects

1. **Length and sequencing risk.** Four complex-analysis chapters (05–08) run back to
   back before the matrix turn. The film is around 25 minutes of speech, unmeasured. The
   lift proof (06_5–06_6) is the most abstract stretch, and the gap gauge is untested on
   a viewer.
2. **The 11_4 argument is asserted more than shown.**
   - "A logarithm commutes with A because A is a series in it" goes by in one sentence
     with one formula.
   - "Commuting with distinct stretches keeps each axis" is only illustrated, by gold
     axes.
   - A picture of XA versus AX acting on one test vector would make it seen rather than
     accepted.
3. **10_3's eigenvalue sentence is overloaded.** It defines eigenvalues ("the numbers,
   possibly complex, by which it stretches its own special directions"). That reading
   fails for the complex eigenvalues it immediately uses (±2πi have no real direction),
   so the window picture carries the idea on faith.
4. **The 12_4 take-away needs 3D spin intuition the film never builds.** Every flow
   before it is planar. The answer (odd half turns either way about the same axis) is
   kept for the description, so the film ends on an open question many viewers cannot
   check.
5. **Two drawing conventions could mislead.**
   - The straight slides in 09_1 and 11_1 are labelled as not flows but look like
     motion.
   - The 12_4 cube has no depth sorting (translucent faces), so its orientation after the
     half turn may read ambiguously.
