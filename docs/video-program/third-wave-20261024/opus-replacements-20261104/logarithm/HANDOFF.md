# Logarithm rebuild: repaired handoff to root (round 2, after Mail 803)

| | |
|---|---|
| Bead | `mx-video-program-program-87do.63` (root-owned; not touched) |
| Author | RubyRaven, Claude Code Opus 5.5, staging only |
| Date | 2026-11-04 |
| Answers | BlueSalmon Mail 803 (root cross-review: five repairs plus a source check) |
| Reservation | 296 on `docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm/**`, renewed to 11:37Z |
| Round-1 snapshot | `snapshots/handoff-mail785/` (26 files plus SHA256SUMS, taken before any repair) |

**This is not a completion claim.** The five repairs are implemented in source and
script, but the candidate is uncompiled, unrendered, unsynthesised and unheard, and the
teaching risks listed at the end remain.

I edited no canonical file, Beads, TODO, catalog or shared file. I ran no Cargo, render,
TTS, ASR, commit or push, and launched no agents.

## The five repairs

### 1. 11_4: commuting shown, not asserted (`src/chapters/unreachable.rs`)

- **11_4** "suppose A = e^X, X real". The probe vector u = (1, 1) travels two routes,
  each drawn with arrows and a midpoint dot: first X then e^X (purple), and first e^X
  then X (teal). A gold dot marks "same end point". Then the series
  A = I + X + ½X² + ⅙X³ + …, and `X X^k = X^{k+1} = X^k X ⇒ XA = AX`.
- **11_5** shows what commuting costs, for A = diag(−2, −½) and a candidate X with
  off-diagonal entries. The two routes A-then-X and X-then-A end apart, with a red gap
  bar and a live gap readout.
  - Screen derivation: "each gap part = ±1.5 × an off-diagonal entry of X × a part of u,
    where 1.5 = (−½) − (−2)".
  - The entries tween to 0 and the ends meet: "the ends meet only when b = c = 0".
  - Algebra: (XA − AX)u = (b(a₂ − a₁)u₂, c(a₁ − a₂)u₁).
- **11_6** "e^(tp) > 0 for all real t and p: no axis reverses", against A's target F,
  outlined in red. A half turn then a stretch needs two flows.
- **11_7 (new)** −I by the spin. "equal stretches: (−1) − (−1) = 0, no gap, so every X
  commutes with −I". Then the 2×2 rule, and Culver's condition in two gold lines.
- **11_8** is the old 11_6 text (complex entries).
- Tests:
  - `x_and_its_own_exponential_end_both_routes_together` checks X·X^k = X^k·X for
    k = 0…11 (worst float mismatch 7e-17 in a Python mirror).
  - `each_part_of_the_gap_is_an_off_diagonal_entry_times_the_stretch_difference`, down
    to scale 10⁻⁶.
  - `the_ends_meet_once_the_off_diagonal_entries_are_zero`, plus the probe/candidate
    fact tests.

### 2. 10_3: no "possibly complex stretch directions" (`src/chapters/doing_nothing.rs`)

- **10_3** For numbers the principal angle stays within a half turn. Flows turn too: at
  a steady rate round stretched circles, perhaps spiralling, or not at all.
  - A turning line shows the spiral flow (0.872, inside the window), a stretch flow and
    the zero flow (0), and the ellipse flows (±2π, outside).
- **10_4 (new)** The same rule picks exactly one logarithm when A sends no direction to
  zero or to a negative multiple of itself: the principal logarithm, real for real A,
  and 0 for the identity.
  - The word eigenvalue appears only in the closing textbook line.
  - For real 2×2 A, "no eigenvalue in (−∞, 0]" is exactly "no direction sent to zero or
    a negative multiple", since a real eigenvalue has a real eigenvector.

### 3. 12: the 3D half-turn puzzle taught, posed and answered (`src/chapters/what_it_forgets.rs`)

- **12_4 (taught)** A solid cube with a gold rod, "the rod: our axis", through the
  centres of its top and bottom faces.
  - Each corner's circle about the rod is previewed.
  - One corner is followed at ½ lap per unit time, with time and laps-so-far readouts.
  - Close: "every corner halfway round: a half turn about the rod".
- **12_5 (posed)** The question and three hints (laps, direction, axis), then the hold.
- **12_6 (resolved on screen)**
  - 1½ and 2½ laps land the same; so does ½ lap the other way.
  - A spin about another axis drags the top face's centre off the rod.
  - "so: infinitely many spins, all about the rod".
  - A turning line: ±π and ±3π, with the slowest two tying on the window's open edge.
  - Precisely line: "eigenvalue −1, so no principal log (Higham §4 Fact 1)".
  - Then "for the number −1, Log(−1) = iπ by convention".
- **Occlusion.** Only the faces turned toward the viewer are drawn, opaque. The rod, the
  other axis, the circles, the traces and the marks are drawn only where a ray-box slab
  test (`hidden`) says the solid does not cover them. Everything that can be hidden is
  drawn inside the same `always_redraw`, so later static mobjects cannot paint over the
  cube.
- Tests:
  - three faces seen at the start (top, +z, −x) and after the half turn (top, −z, +x);
    never a back face.
  - the rod is visible only outside the cube; the marks stay in view.
  - the shown rates all end at the half turn; the slowest two tie; no other axis lands.

### 4. Straight slides become endpoint comparisons, with the singular midpoint

- **09_1** (`matrix_logs.rs`) never moves the plane. A ghost of the plane before stays
  put, the plane after is laid over it, and dashed chords pair marked points with where
  they land.
  - Script: the matrix "only says where each point ends up, not how it gets there; the
    travel comes next".
- **11_1** compares the mirror before and after; "dashes pair each corner with where it
  lands". Then, in red, "the average ½(I + mirror) has det 0: it squashes the plane flat
  onto a line" (fact `matrix.mirror_average_det`).

### 5. 05–08: a concrete prediction or visible payoff in every cue

Nothing was shortened. `docs/storyboard.md` has the cue-by-cue table.

| cue | added |
|---|---|
| 05_1 → 05_2 | **predict** "i × i = ?"; answered by two gold quarter arcs landing on −1 |
| 05_2 | worked input e^{ln 2 + iπ/2} = 2i, with an arrow to 2i (fact `complex.worked`) |
| 06_4 → 06_5 | **predict**: a side loop (0 not inside), walked a third of the way: will the log come back? It returns, "change 0" |
| 06_6 (new) | the concrete candidate "keep the angle between −π and π" walked beside ours. A gap-in-laps gauge sits at 0, then at −1 a red ring and "πi → −πi, gap 0 → −1" (fact `loop.principal_gap`). Then the proof setup. The old 06_6 is now 06_7 |
| 07_2 | worked Log i = πi/2 and Log(−i) = −πi/2, with gold and purple arcs (fact `branch.log_i_pi`) |
| 07_3 → 07_4 (new) | **predict**: the plane rolls back, and the cut's two sides are marked (gold above, purple below, hidden until asked). Where do they go? 07_4 unrolls: top edge, bottom edge, "angle just above the cut: near π; just below: near −π", then the 2π jump. Old 07_4/07_5 are now 07_5/07_6 |
| 08_1 → 08_2 | **predict** "Log(z²) = ? i" on the window line; 08_2 opens "Not one and a half pi: that is outside the window" |
| 08_3 | "for z²: k = −1, one lap down", computed from the facts |

## Sources (Mail 803's check), in full in `explainer_corpus/logarithm_mysteries/docs/sources.md`

- **Read and quoted:**
  - N. J. Higham, *Functions of Matrices*, MIMS EPrint 2005.21 (2006, the Handbook of
    Linear Algebra chapter), §4: Fact 1 (principal log: no eigenvalues on R⁻, the open
    strip, real for real A), Fact 3 (real log ⇔ nonsingular and an even number of Jordan
    blocks of each size per negative eigenvalue), and Fact 5 (det A = exp(tr log A)).
  - J. Gallier, arXiv 0805.0245v2: Thm 3.3 (every invertible complex matrix has a log,
    via the Jordan form), Thm 3.4 (Culver's condition, with proof) and Thm 3.11(a) (real
    invertible with no negative eigenvalues ⇒ unique real log in the strip).
- **Screen lines now match what was read:**
  - 11_7: "Culver, Proc. AMS 17 (1966) 1146–1151, as stated by Higham (Handbook of
    Linear Algebra) and Gallier (Thm 3.4)".
  - 11_8: "Gallier, arXiv 0805.0245, Theorem 3.3, proved via the Jordan form". It
    replaces round 1's unread "Higham Thm 1.27".
  - 10_4: "(Higham; Gallier Thm 3.11)".
  - 12_6: Higham §4 Fact 1, with the one-line step from "eigenvalue −1" to "no log has
    every eigenvalue in the open strip".
- **Gaps:**
  - **Culver's paper is unread.** Two download attempts returned the AMS "Verifying your
    connection" page. The DOI is carried from round 1, unresolved.
  - **Higham's 2008 book is unread.** Its Thm 1.31 appears only as Gallier's
    cross-reference.
  - **Carried from round 1, not re-read:** DLMF §4.2 (chapter 07) and ISO 16 (chapter
    00).

## Narration: ElevenLabs Oliver

All narration is **ElevenLabs Oliver** (owner requirement; reminders Mails 698, 726,
753 and 831). The Michael and FrankenTTS defaults in the skills do not apply.

- I launched no synthesis.
- I did not touch the retained takes in
  `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-20261102`. That directory
  is unchanged since 2026-11-03, so no take of this candidate exists yet.

**What the orchestrator needs to synthesise: all 75 cues.** None has a take. Against the
Mail 785 script:

- **Reworded (18):** 05_1 05_2 06_4 06_5 06_6 07_2 07_3 07_4 08_1 08_2 08_3 09_1 10_3
  11_1 11_4 11_5 11_6 12_4.
- **New (4):** 10_4 11_7 12_5 12_6.
- **Renumbered with identical text (4):** 06_7 = old 06_6, 07_5 = old 07_4, 07_6 = old
  07_5, 11_8 = old 11_6. Bind by text, never by the old id.
- **Unchanged (49):** the rest.
- **Queue rule:** if any Mail 785 cue was queued, the 26 above supersede it by id.
- **Takes:** round 1 found no retained take matching any of its 68 cues. The 22 new or
  reworded texts were written after those takes, so none can match either.
- **Spoken text and waits:**
  - Spoken text contains no digits.
  - Sentence waits index `script.tsv` sentences (split after ". ", "? ", "! "). A
    rewording that changes a cue's sentence count breaks its chapter's waits;
    `docs/check_cues.py` catches it.

## Files (all under `explainer_corpus/logarithm_mysteries/`)

| candidate | integrates as | note |
|---|---|---|
| `script.tsv` | canonical `script.tsv` | 75 cues, 364 sentences, 4,592 words |
| `src/main.rs` | canonical `src/main.rs` | same `Explainer` contract; adds `mod model;` |
| `src/facts.rs` | canonical `src/facts.rs` | 93 keys (round 1: 81; this round adds 12: the flow's turning rate (two keys), the flip-pair and probe gaps, the probe and candidate, the mirror average's det, the spin rates, i·i, the worked 2i, the principal gap and Log ±i) |
| `src/model.rs` | new | numerics: complex numbers, lifts, 2×2 matrices, flows, turning rate, principal log |
| `src/chapters/mod.rs`, `draw.rs`, `thumbnail.rs` | as in round 1 | |
| `src/chapters/{ratio_ruler … what_it_forgets}.rs` | new | chapters 00–12. Changed this round: complex_turns, lifting_loops, branch_cut, product_rule, matrix_logs, doing_nothing, unreachable, what_it_forgets |
| `oracle.py` | beside the crate | Python mirror of facts.rs, 93 keys |
| `docs/check_cues.py` | review tool | unchanged |
| `docs/storyboard.md` | storyboard | rewritten: cue table for 05–08, 09–12 beats, holds, sources |
| `docs/sources.md` | new | quotes, conditions, where each claim is used, gaps |

As before: `Cargo.toml` and `src/score.rs` stay canonical. At integration, move aside
the canonical `bookends.rs`, `real_inverse.rs`, `winding.rs`, `product_law.rs`,
`matrix_log.rs` and `branch_cut.rs` (same name, different content).

## Checks run here, with results

All checks were run from `explainer_corpus/logarithm_mysteries/`.

1. `python3 -I -B oracle.py` prints **93** keys. facts.rs's count test expects 93. The
   Rust `self_check` and tests have not run.
2. `python3 -I -B docs/check_cues.py`: **OK**. It reports 75 cues and 364 sentences,
   every cue said once and in order, every `until_sentence` valid, increasing and on the
   current line, the registry covering every prefix, and no digits in spoken text.
3. `rustfmt --edition 2024 --config max_width=120 --check src/chapters/mod.rs src/facts.rs src/model.rs`
   is **clean**. This parses every chapter module.
   - `src/main.rs` was checked in a scratch copy beside canonical `score.rs`. The only
     diffs are in that pre-existing canonical file.
4. A scratch check confirmed that all 13 `predict` calls are the last beat of their cue
   (the next statement is `kit.say`). The 05–08 questions end in "?". 00_4 ends on its
   deliberate wrong-answer lure and 12_5 on its hints.
5. Python mirrors of the new tests' numerics all hold:
   - the 06 principal-gap jump (0 up to −1, −1 just after);
   - the 07 marked sides (0.14 apart on the cut plane, within 0.06 of the strip edges)
     and label widths;
   - 05's 2i inside the panel;
   - 08's k = −1;
   - the 11 route gaps and commuting powers;
   - the 12 cube faces, visibility and rates.
6. Every test name and fact key cited in `docs/sources.md` was grepped and exists.

**For root (not run by me):**

1. `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test`.
   There are 101 `#[test]`s; round 1 had 80.
2. The glyph probe (new strings include "→", "²", "½", "−", "π", "×" and `\;`) and the
   storyboard check.
3. A draft render.

## Gaps (explicit; none of these has been done)

- **Uncompiled.** About 10,000 lines have never been through Cargo. Type, borrow and
  Clippy errors are likely. APIs were checked by reading the kit and the pin's sources,
  for example that fmn's `fade_out` restores the mobject so 07 can fade the strip's
  height marks back in.
- **Unrendered.** No frame has been seen. Occlusion, overlaps and label fit are
  coordinates plus width-estimate tests, not pixels.
- **No synthesis, listening, fine-sync or independent AV review.** ASR has not been run
  and would not count as listening. THINK = 4 s on 13 holds and every pacing figure are
  proposals.
- **Sources:** Culver unread; Higham's book unread; DLMF and ISO 16 not re-read
  (above).
- **Literals that stay literal:** `matrix.half_turn_speed` is `"0.5"` in facts.rs, and
  11_2's "t = 1" and "t = 3" labels are hard-coded. Both are unchanged from round 1.
- **Hook artifacts, not mine.** `.ee/` directories exist at the crate root and in
  `src/chapters/` (session-hook state). The snapshot excluded them; root should exclude
  them at integration.

## Teaching risks that remain (honest, not exhaustive)

1. **Length.** About 28–31 minutes of speech, unmeasured, up from about 25. The repairs
   added cues rather than cutting.
   - The 05–08 block is now 25 cues of complex analysis before the matrix turn.
   - Root may want a pacing decision after the first timed draft. I did not shorten,
     per Mail 803.
2. **11_4–11_7 is the densest algebra in the film.** Four cues of commuting, gap
   components and the 2×2 rule, each with a picture.
   - The live gap readout and the "±1.5 × entry × part" line need a frame check for
     legibility.
   - The ends-meet tween's timing needs the real take.
3. **The 06_6 cue is long (8 sentences).** It carries both the candidate's break and the
   proof's setup.
   - It could split at "So that candidate breaks at minus one", which would make 06_6 /
     06_7 / 06_8.
   - I left it whole, so as not to renumber cues again without root's say.
4. **12's cube is unseen.** The slab-test occlusion is tested numerically only. The
   corner trace and the other-axis drag need a frame review for readability at 4K.
5. **07's side labels** ("just above"/"just below") sit near the r = 1.5 and 2.3
   circles of the cut-plane grid. They may touch a thin line.
