# CC0 square candidate02: independent review of the delta (CC1, 2026-11-10 ~21:55Z)

**Scope:** a source-only review. I did not compile, run cargo tests or Clippy, render, probe glyphs or listen to anything. **No rendered or visual acceptance is claimed.** Nothing in production was touched, and every file I wrote is in my scratch, `.../scratchpad/square-cc0-review01/`.

**Independence:**
- I authored candidate01 (L12, L11 and L5 over build268). This review is therefore independent **only of CC0's delta** (F1, F2 and F4, about 9 hunks).
- My checks of the quarter construction, the L11 walk and the L5 name placement re-check my own work.
- Two earlier limits of mine also apply: the boundary review (boundary-review01) was source-only, and I wrote that review's base (752).

Narration is ElevenLabs Frederick Surrey, unchanged. The delta touches no audio, script.tsv or cue words.

## Inputs (copied before review)

- **snapshot01** (copied ~21:47Z): all 42 files of CC0's folder, verified by `shasum -c` after the copy. The list is in `source-hashes01.txt`.
- **snapshot02-additions** (~21:50Z): two files CC0 added later.
  - `candidate02-notes.md` (e9c5da03…)
  - `receipt.json` (f0a4b7a3…)
- I rechecked the original 42 at the second copy and none had changed. All 43 hashes listed in CC0's receipt match my copies.

| Role | sha256 |
|---|---|
| candidate02-loop_sums.rs | 977c77df1741c970b76a6ebbbebcacf5cbee1db6e62d564a8428d97c077f835f |
| candidate02-mod.rs (= candidate01-mod.rs) | 4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578 |
| delta patch from candidate01 | a7788d8e729369f5496e0bfb0370a59cddb5c5d25c52e33f8e364130bcf1b3fe |
| full loop_sums patch from build268 | 53a79e38d3aa3d7dce578792e16c007cb27fc4feb0d6c7cb4414def41eea2ed5 |
| mod patch (= frozen candidate01 mod patch, cmp equal) | c1fb51678b7e82da025ca399e3b38b8bfd75afb60c09f47af1c7faea5bec8bc8 |

## Lineage and round trips (my scratch, `roundtrip01/`)

- CC0's `inputs/` are byte-equal to the frozen `square-repair-cc1-frozen01` files:
  - build268 loop_sums 8a13e8a1… and mod e1f4d1b8…;
  - candidate01 loop_sums 9a91ed2a… and mod 4de1e4a5….
- Five round trips, all exact:

| Applied | Result |
|---|---|
| candidate01 + delta | 977c77df |
| build268 + full patch | 977c77df |
| build268 mod + mod patch | 4de1e4a5 |
| reverse delta on candidate02 | 9a91ed2a |
| reverse full patch on candidate02 | 8a13e8a1 |

- **B1 still binds.** The committed tree is loop_sums cf383504 and mod 57c46d67; I re-hashed both today. They are not the base. Apply only onto 8a13e8a1 / e1f4d1b8, or copy the candidate files and confirm 977c77df / 4de1e4a5.
- **rustfmt** (repo rustfmt.toml: edition 2024, max_width 120; rustfmt 1.10.0-nightly 2026-08-30), via stdin compared byte for byte:
  - candidate02 loop_sums, candidate02 mod and candidate01 loop_sums: unchanged, so formatted;
  - draft1 differs, as CC0 reports. That is the control.

## Timing invariance

- I wrote a new comparator for this review, `timing-skeleton02.py`. Its output is `timing-skeleton02.out` (sha below).
- It covers the whole film part, which is wider than CC0's 111-event comparator: it also counts `.rt(`, `stage.prepare`, bare `fade_out`/`fade_in` and `tween`, and it compares every numeric literal in each call.
- Result: **184 events, identical** for build268 → candidate02 and for candidate01 → candidate02.
- Positive controls:
  - my variant `c_aside.rt(0.3 → 0.35)` is caught at event 63;
  - CC0's T1 quarter fade 0.5 → 0.6 is caught at event 76.
- The delta adds no timed call. The redrawn name rides the existing 0.5 s quarter fade-in and the 0.4 s `square_picture` fade-out.

## F1/F2: rim-start leader with intended crossing (`LEADER_START` 0.075)

My own mirror is `square-geometry01.py` → `.out`. It rebuilds the nest from the source rules, using true stroke widths: Q_0 and Q_1 5, Q_2–Q_5 3, leader 1.5, dot radius 0.07.

| Quantity | Value |
|---|---|
| Rim junction | The cap reaches 0.0675 from c, so it overlaps the dot by 0.0025 (0.7 px at 4K, 0.2 px at 960). The leader is drawn after the dot, on top. Round caps on open strokes are confirmed at fmn a399d626 (`fmn-conformance/tests/certified_engine.rs:361`: "round caps, with nobody asking for one"). |
| Sides whose centre lines it crosses | Q_3 right and Q_4 right (x = −2.1), and Q_5 bottom at s 0.0769 |
| Side it touches in ink | Q_5 right (centre-line gap 0.0144) |
| Stretch on nest ink | rim to s 0.146: 20.5 px at 4K (5 px at 960); then clear for 81 px at 4K |
| Test classification (reach 0.0325) | touched = [(3,1), (4,1), (5,0), (5,1)], exactly as the test expects; no near misses |
| Least untouched gap | Q_3/Q_4 bottoms at the corner (−2.1, 0.4): 0.0585, which is +0.006 over reach + 0.02 |
| From s 0.25 out | clear of every side |
| δ-circle (radius 0.318) | crosses once, drawn later on top; unchanged |

**Q_5 against the dot:** Q_5's half side (0.0625) is smaller than the dot's radius, so Q_5 shows only as corner stubs ringing the dot. They reach 9 px beyond the rim at 4K. The leader leaves through the bottom-right stub. Even if the eye assigns the start to that stub, the stub belongs to the square that shrinks to c, so the association with c holds.

The comments in the source (`LEADER_START` doc, L5 scene comment, test doc) match these numbers.

**Verdict on F1: resolved in source terms.** "Reads as leaving the dot" is a 4K-still judgment I cannot make.

**Verdict on F2: resolved.**
- The false no-crossing guarantee and the "start past 0.15" assertion are gone.
- The new test binds the rim start, the ray, the exact touched set in ink, the no-near-miss margin, the clear run from 0.25 and the single δ crossing.
- I re-ran CC0's `test-mirror03.py` after reading it (it reads two files and writes nothing). It gave 46/46 PASS. Its output is byte-identical to CC0's `controls/candidate02.mirror03.out` apart from the mod file's basename, and the two mod files are byte-identical.
- My frozen `test-mirror02` hard-codes 0.18 and cannot see this change. Do not cite it for candidate02.

## F4: c-name redraw and tint

- `centre_name_at` gives the name one place. The original `centre_name` and the new copy use it identically.
- The copy is drawn in `quarter_picture` in this order: fill, kept outline, kept diagonal, dot, **c name**, Q_{n+1} name. My source check confirms the order, and that the copy lies after the fill and before the fade-in.
- After L12 the film uses `centre_name` only in the `square_picture` list. Both copies leave together at 0.4 s, so no state can diverge.
- The name stands inside the kept fill. Its estimated glyph box clears the nearest L12 stroke by 36.5 px at 4K.
- An opaque copy above a 0.16 fill shows the pure colour. During the 0.5 s fade the residual tint is at most 0.16·a(1−a), about 4%.
- **Verdict on F4: resolved in source terms.**

## Real-quarter semantics (unchanged by the delta; not independent, my candidate01)

- KEPT_QUARTER 3 is the top-left `proof_children` quarter of the magnified Q_n.
- c, the centre of Q_n, is a corner of all four quarters, so the kept one contains c whichever the rule keeps.
- The kept quarter's diagonal is parallel to D_n and half as long, and its perimeter is P_n/2. On screen this gives D_nP_n/4 per cut and |I_{n+1}| ≥ |I_n|/4, with nothing calling the quarter the largest.
- The delta changes only the explanatory comment, and the new wording is accurate.

## API and test plausibility (by reading, at fmn a399d626)

- `kit.tex(stage, "c", 26.0, output(), &[], centre_name_at)?` has the same signature as the existing line.
- `Vec3 = [Semantic; 3]` with `Semantic = f64` (`fmn-core/src/types.rs:5,13`), so it is Copy and can be passed twice.
- `kit.tex(...)?` inside an array/vec literal with sequential `stage` borrows is used in compiled chapter code, for example `boundary.rs:398` and `bookends.rs:40`.
- In the test:
  - `Vec<(usize, usize)> == [(…); 4]` compiles via Vec's array PartialEq.
  - `sides(q)` with `q: &Vec<C>` deref-coerces.
  - `nest[k][1]` infers usize.
  - I spotted no default-Clippy trigger. Strict Clippy is not run.
- **Test rename:** in production-repair17, only review notes, frozen candidate01 and my test-mirror02 name the old test. No root recipe binds it, by search.

## Findings

**No blocker in the delta.**

**Low:**
- **L1:** the rim overlap is thin, 0.7 px at 4K. It relies on round caps (confirmed) and on the dot's flattened outline sitting at 0.07. If a 4K still shows a hairline at the junction, `LEADER_START` 0.072 keeps the centre line outside the rim, doubles the overlap to 1.5 px and still passes every assertion in the test: the rim, the touched set and no near miss (`leader-start-sweep01.out`). Do not change it without a still.
- **L2:** the leader's first 20.5 px at 4K lie on hero ink (the Q_5 stub and the x = −2.1 edge). Legibility of the crossing (dim 1.5 over stroke 3) is a 4K-still check. The association risk is low, as argued above.
- **L3:** the c glyph and its dot are now both drawn twice; the dot's double has been there since candidate01, my own code. Two minor costs:
  - their anti-aliased edges composite twice, so they look slightly heavier;
  - in the 0.4 s `square_picture` fade they fade as 1−(1−o)², reaching 75% when single labels are at 50%, so they lag slightly.

  If a still shows either, a re-root of the existing `centre_dot`/`centre_name` above the fill (the `add_to_scene` re-append) avoids both. The cost is a tint of at most 16% during the 0.5 s fade-in. This is optional and not needed now.
- **L4:** the touched-set test is a deliberate golden with a +0.006 margin. A failure after any `C_NAME_OFFSET`/`LEADER_START` edit means "re-review the crossing". It is not a geometry bug.

**Carried, unchanged by the delta:**
- F5: the P_n baseline is unmeasured.
- N2: every earlier still of L5/L11/L12, including root's 344–359, describes neither candidate.
- B1: lineage, as above.

## Not run (root's guarded flow)

- cargo build, plus the four square tests and the rest of the chapter's tests, at fmn a399d626.
- strict Clippy (`-D warnings`).
- the glyph probe.
- a native render, and 4K stills of:
  - L5: the rim junction, the crossing, and the leader visible over the edges;
  - L12: c untinted, no visible doubling, a smooth 0.5 s fade;
  - L11;
  - the F5 baseline.

## My evidence files

All are in `/private/tmp/claude-501/-Users-jemanuel-projects-jeffs-math-explainer-videos/b0a72b9b-6c8d-4794-942e-44d50a9d8626/scratchpad/square-cc0-review01/`:
- `source-hashes01.txt`
- `snapshot01/` (42 files)
- `snapshot02-additions/` (2 files)
- `roundtrip01/` (5 files)
- `timing-skeleton02.py` and `.out`
- `control-c-aside-035-loop_sums.rs.txt`
- `square-geometry01.py` and `.out`
- `leader-start-sweep01.py` and `.out`
- `cc0-mirror03-rerun-on-candidate02.out`
- `fmt-tmp.rs`: a throwaway rustfmt output, last written for draft1.
