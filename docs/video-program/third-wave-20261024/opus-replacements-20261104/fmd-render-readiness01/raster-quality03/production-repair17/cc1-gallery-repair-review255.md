# Gallery repair review 255 (cc1): independent review of candidate 249 (05_9 critical point)

**Bound to:**

| File | sha256 |
|---|---|
| `gallery-review-repair249.rs` | `d57e003a4067f3e67abb654e98c19ca0274c9b7edae5ac26da756757e33230b5`, 1,443 lines |
| `gallery-review-repair249.md` | `dbd8cadf07338533dc28ae5c6707015f968261206845e67f61d5157f99e7a2c1` |
| `mod-review-repair256.rs` (root's registry) | `e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd` |

Any other bytes need a new review.

**Verdict: sound to integrate, with no blocking finding at these hashes.**
- The mathematical exception is correct and correctly placed.
- The source layout clears its neighbours.
- The new marks join exactly one existing fade-in and one existing fade-out.
- Every timed call and cue call is unchanged.
- The registry need is met by 256.
- The first draft's test defect (R1) is fixed in `d57e003a`, with the branch-free polynomial equality.
- There are two Lows: the contrast of the ×, and the reading load.

This review covers source and existing stills only. Nothing was compiled, tested, rendered or heard. Reading time and the native picture remain open (see Limits).

## Verification

- **Base.**
  - The candidate's base is production19 `gallery.rs`, `7d1cf20f…`. That is the `production-source-preserved214` copy named in its `receipt.json`, and the file is unchanged.
  - Canonical `explainer_corpus/.../chapters/gallery.rs` is `6f310eb7…`, last changed 2026-11-04, and differs from production19 by 267 lines. **Do not apply the candidate to canonical**; it belongs on the production19 or build workspace.
- **Diff.**
  - My `diff -u` from the base to `d57e003a` touches only the hunks listed in the table below.
  - The diff embedded in `gallery-review-repair249.md` equals my diff line for line (145 of 145 body lines).
  - `rustfmt 1.10.0-nightly --check` (edition 2024, `max_width` 120) passes on the candidate. This checks formatting and parsing only.
- **Version history.** I first reviewed v1 (`1d707048…`). The only changes to v2 are:
  - local names shortened to `cross` and `unless`, which keeps the fade call on one line under rustfmt's call-width rule;
  - the corner check (R1 below).

| Hunk | Change |
|---|---|
| Module doc | Adds that z = 0 is where the cubic's tiles are not square. |
| Constants | `UNLESS_NOTE`, `UNLESS_AT = (−4.5, −0.1)`, `CRITICAL_MARK = 0.12`. |
| 05_9 tile beat | After `line.wait_for(stage, "nearly square")`: two ink strokes form a × at z = 0 on the cubic's panel (width 3), and the note is placed (18 pt, dim). Both join `fade_in_all(stage, &[outline, tile_says, cross[0], cross[1], unless], 0.5)`. |
| Closing | `all.extend(cross); all.push(unless);` before the existing `fade_out_all(stage, &all, 0.5)`. |
| Tests | The expected fade line in `the_tile_is_ready_for_the_words_that_name_it`, and the new test `the_note_marks_where_the_tiles_are_not_square`. |

## Mathematical exception

The note says "nearly square unless the derivative is 0, as at z = 0". **It is correct.**

- **The general condition.** The enhanced portrait's tiles are preimages of the square grid of step s = 2π/12 under log f. They are nearly square where log f is conformal, i.e. where f′/f is finite and nonzero. Production19's `model.rs` says this beside `PHASE_LINES`.
- **Between zeros and poles.** There, that condition is exactly f′ ≠ 0.
- **Near a zero or pole.** log f ≈ m·log(z − z₀), so the tiles are small polar squares and the claim holds. The voice already sets zeros and poles aside.
- **The cubic.** For z³ − 1, f′ = 3z² vanishes only at 0, and f(0) = −1 is not a zero. So z = 0 is the cubic's only critical point in ℂ, and the module doc's "the one place on the cubic" is right.
- **Why it is a tile centre.** log f(0) = −πi. `(-ONE)` carries im = −0.0, so `atan2` gives −π; +πi is equivalent mod 2πi. With `BAND_OFFSET` 0.5, both 0 and ±π = ±6s are band centres.
- **The cell at 0.** f + 1 = z³ is three-to-one near 0. So the tile holding −1 pulls back to one cell with 12 corners.

My mirror of production19's arithmetic gives:
- the corners at |z| = 0.6874 and 0.7501, at least 0.300 apart;
- each corner satisfying z³ − 1 = e^corner to about 1.2e−16;
- the traced tile's side as 0.326, so the 0 cell is about four tile sides across.

**Other panels.** These critical points sit inside their mosaic windows:
- sin: ±π/2 and ±3π/2;
- Γ: 1.4616, −0.5041, −1.5735, −2.6107 and −3.6353;
- ζ: −2.7173 and −4.9364.

There are none for e^z or for (z − 1)/(z + 1)², whose critical point is at z = 3, outside its window. All five of the other panels leave 0.3 s after "in little tiles", before "nearly square" is spoken (`TILE_BEATS[1]`). So "as at z = 0" covers everything on screen when the claim is made. **O1:** none of this needs a change.

**The zoom (O2).** The zoom window's lower-left corner, −0.036 + 0.413i (|z| = 0.415), lies inside the 0 cell. So the zoom already shows a sliver of the non-square cell at that corner. This is consistent with the note, and nothing needs to change. The picture review should look at that corner.

**Wording precision (O3).** The note names a point, but the distortion covers a neighbourhood: the 12-cornered cell plus bent neighbouring tiles. "as at z = 0" with the × is an acceptable on-screen shorthand. It matches the 06 wording root adopted ("unless the derivative is 0", candidate 231).

## Source layout bounds

These come from the source and CMR10 arithmetic. The mosaic's cubic panel spans scene x −6.6 to −2.4 and y 0.25 to 2.85, at 0.7778 scene units per unit of the plane.

| Object | Placement | Clearance |
|---|---|---|
| × at z = 0 | Scene (−4.500, 1.317); arms ±0.093 scene each way (about 50 px across at 4K); stroke 0.03 | Zoom box's bottom edge (z 0.413) is 0.228 above the arm tips. The box's left edge (x −4.528) starts above that, so no crossing. Zero marks: arm tip to mark centre 0.646 (mark radius 0.11). Links leave from the box's right corners (x −3.517), away from it. |
| Note | Centre (−4.5, −0.1); 53 characters, **22.48 ems** at 18 pt = 4.046 wide (the candidate assumes 22.5), spanning x −6.523 to −2.477 | Panel bottom (0.25): 0.20. Tile label's top (about −0.44): 0.19. Lower link at the note's top (y 0.05, link at x −2.223): 0.254. Frame's left edge (−7.111): 0.588, i.e. 159 px at 4K. |
| Note's contrast | `Palette::dim()` `#8A93A3` on `#0E1116` | 6.11:1 |

During the note's interval nothing else sits in that area: the bottom-row panels have gone, there is no lower third in 05_9, and `next` is at y −1.4. The new test asserts these bounds with margins of 0.15 or more. The half heights are estimates, as the test says, so the picture decides.

## Fade and list membership

- **Made after the voice reaches the words.** Both marks are made only after `line.wait_for(stage, "nearly square")`, so nothing appears early.
- **In once, out once, nowhere else.** They are faded in once, in the existing 0.5 s fade with `outline` and `tile_says`, and out once, in the existing closing 0.5 s fade. They are on no other list (`gone`, `leaving`, `framing`), so nothing fades twice.
- **Layering.** They are added after the cubic painter's group, so they draw above the plate.
- **The existing guard still holds.** The 05_9 closing section still excludes `zedge`, `converges`, `pole_says` and `squashed`.
- **The source-reading assertions pass.** I mirrored them on the exact bytes. Both `runs_in_order` lists are satisfied. The 05_9 section contains the `tile_says` line. The substring counts are as asserted: `UNLESS_NOTE` 1, `unless` 3, `cross` 4. No "across" or other name in 05_9 inflates those counts. No `.leave(stage` appears between "in little tiles" and "nearly square".

## Durations and cue calls

- **Timed calls.** I extracted every timed call from production19 and from the candidate, in source order. That covers `say`, `wait_for`, `wait_sentence`, `wait_until`, `hold`, `music_breath`, `reading_window`, `finish`, the fade calls with their run times, `show`, `leave`, `play` and `pause`. Both have **143, in an identical sequence**. CC0 counted 136 with a different pattern, also identical. The new mobs add no play time.
- **Frame count.** With the final05 timeline (breath at 9669 for 2 s, then 0.5 + 0.5 + 0.5), the chapter stays at **9,879 frames**.
- **Unchanged.** The script, cues, takes and narration (ElevenLabs Frederick) are unchanged.

## Registry needs

- **The one line needed.** `UNLESS_NOTE` is a new fixed on-screen string, so `TEXT` needs exactly one line, `gallery::UNLESS_NOTE`.
  - `mod-review-repair256.rs` (`e1f4d1b8…`) is `mod-review-repair236.rs` (`b5817c50…`) plus exactly that line, at 356 after `gallery::NEXT`.
  - It is not a duplicate: it differs from `one_derivative::UNLESS_ZERO`, so `no_supplement_is_empty_or_duplicated` is satisfied.
- **Glyphs.** The string is ASCII letters, 0, a comma and "=". All are already drawn by other gallery strings.
- **Nothing else.** No `TEX`, `MONO`, `READING_WINDOWS` or `TO_READ` change is needed; 05_9 has no reading window, and none may be added without changing timing.
- **Integration note.** Against production19's `mod.rs` (`4f4b17c3…`), 256 also carries 236's three `one_derivative` entries and the "Selected square, magnified" caption. It is cumulative, so it has to ship with those candidates. It is not a standalone change to production19.

## Findings

### R1 (resolved; would have failed the test gate): branch-cut failure in v1's corner check

- **The failing line.** In `1d707048`, the new test checked `assert!(((z * z * z - ONE).ln() - w).abs() < s, "{z:?}");`.
- **Why it failed.** w = (−ONE).ln() = −πi. The corners w + (±1 − i)·s/2 have angle −π − s/2, so the principal ln wraps them by 2π.
  - 6 of the 12 corners fail by 6.0271 against s = 0.5236: (a, b) = (±1, −1), k = 0..2.
  - The other six pass at 0.3702.
- **I sent CC0** the exact line, the cause, and an angular-distance fix: wrap `d.im` into [−π, π) with `rem_euclid(TAU)` before `hypot`. The mirror gives 0.3702 < s for all 12 corners with that fix.
- **At `d57e003a`** the test instead asserts `(z * z * z - ONE - corner).abs() < 1e-9` with `corner = (w + C::new(a, b).scale(s / 2.0)).exp()`. This is the branch-free polynomial equality. It is log equality mod 2πi, stronger than either log check, and holds to about 1.2e−16 in the mirror. **Resolved.** The radius (0.55, 0.8) and spacing (> 0.1) bounds are unchanged and hold.

### L1 (Low): the white × has 2.35:1 contrast on the cyan cell

- **Measurement.** In the existing final05 stills (frames 8880, 9000, 9300, 9420, 9600 and 9780), the plate under the ×'s footprint at 4K (705, 724) is a steady (71, 172, 176). Ink `#ECEFF4` against it is **2.35:1**, under the usual 3:1 for graphic marks.
- **Why it matters.** The chapter's own marks get a dark backing "so it reads on any colour" (`zero_mark`). The × does not.
- **Mitigation.** It is 50 px across with 8 px strokes at 4K, so it is probably findable, but it is the weakest element of the repair.
- **Possible fix**, within the same fade and with no timing change:
  - draw each arm twice, a `ground()` stroke of about width 6 under the ink stroke of width 3; a dark × alone measures 6.99:1;
  - or draw it in `ground()`.

  Either way the new test's `cross` count and order strings must be updated. The native picture decides.

### L2 (Low, kept open): reading load at the end of 05_9

- **What must be read.**
  - The note is 53 characters, about 3.5 s at the program's 15 characters a second. Its unspoken part, "unless the derivative is 0, as at z = 0", is 39 characters, about 2.6 s.
  - It arrives with the tile label (34 characters) and is followed at sentence 4 by `next` (41 characters).
- **What I measured.** In the existing stills, the tile label is not yet visible at 154.0 s and is fully up at 155.0 s. The closing fade starts at 163.15 s (timeline breath 9669 + 120 frames). So the note is up for about 8.2–9.2 s.
- **CC0's figure.** CC0 estimates about 2.6 s of that is free of speech. On their count, the text the voice does not say rises from about 43 to about 82 characters.
- **No timing fix allowed.** This cannot be fixed without a timing change, which this repair must not make. Whether the ending gives enough reading time is left explicitly to the native picture review and to the listening and sync acceptance.

### Observations (no change)

O1 (critical points in the other panels), O2 (the zoom's corner sliver of the 0 cell) and O3 (point against neighbourhood wording) are covered above.

## Limits

- **Independence.**
  - I read the production19 source and the candidate, and wrote my own mirrors of the maths, layout, source tests and timed calls, before reading `gallery-review-repair249.md`.
  - However, F2 itself comes from CC0's review 240, which I read beforehand.
  - The v1 → v2 test fix answers my own report (R1). So that part is not independent of me.
- **Not run.** No compile, cargo test, Clippy, glyph probe, build, render or new extraction.
  - All the Rust is uncompiled. `rustfmt --check` is a format and parse check only.
  - Root's build256 scripts list `gallery.rs` among their expected files. I did not inspect their recipe, and I claim nothing about them.
- **Arithmetic, not measurement.** The layout, contrast and reading figures are arithmetic from source plus measurements on existing final05 stills (`complex-final05-pictures153`), not on a render of this candidate. The native picture decides the note's legibility and margin, the ×'s legibility (L1), and the zoom corner (O2).
- **Open.** Listening, fine sync, motion review, reading-time acceptance (L2) and independent final AV acceptance all remain open.
- **Not addressed.** F1 (plate fades) is not addressed and belongs to CC2's investigation 246.
- **What I wrote.** I edited no candidate, canonical, shared, TODO or Beads file. This file is the only file I wrote.
