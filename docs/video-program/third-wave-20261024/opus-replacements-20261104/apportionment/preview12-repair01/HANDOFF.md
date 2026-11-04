# Apportionment chapter 12: staged F1–F5 picture repair

**Bead:** mx-video-program-program-87do.30. **Grant:** Agent Mail 1269. **Review repaired:** `../preview12-picture-review01.md`.

**Status: staged, uncompiled, unrendered.** Nothing here has been through Cargo, Clippy, a render, listening, fine sync or independent AV acceptance. The current preview is a scratch voice. The narrator stays ElevenLabs Oliver. No cue's words change, so this repair adds no cue to synthesize.

## What to apply

From the repository root, after confirming the canonical hashes below:

```
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/apportionment/preview12-repair01/preview12-repair01.patch
```

| File under `explainer_corpus/apportionment/` | Base sha256 (= canonical now) | Candidate sha256 |
|---|---|---|
| `src/chapters/both_promises.rs` | 81d2f0aa92091dc9 | da41049bbaf9d9fe |
| `src/model.rs` | 91baa9ca3f1d050d | d7bb080faba88e63 |
| `src/facts.rs` | ec55517c9544e275 | b26bf1e409958a2a |
| `oracle.py` | 56009064259c3fba | d9a5a6ab098cae54 |
| `docs/storyboard.md` | 31bf5bf4fd382a40 | 85d3b36b4a9bb11d |
| `docs/narration-direction-map-oliver03.json` | 317f821a32e6a804 | 3fc8199165851dc2 |
| `narration_units.json` | 7adcd8ad17bddd52 | 7b2024b64bb8eb7d |

Patch sha256: `910636402ec6c0ba35842e21077061564fde0dab2786d151e055a5a2d6ef31b4`. Full hashes are in `MANIFEST.json`. Read but not changed: `script.tsv` cfcec06911c7d94e, `src/chapters/draw.rs` dd385ed5f45f09ff, `docs/description.md` 0c87d6bcdabea112. No other chapter changes, and no helper outside `both_promises.rs` is touched.

## The design decision

The old picture could not be repaired by hiding fills. With Cedar 8 (10½ to 12) missing, no tile handed out by house 6 ends later than 12, so `then` = 6 is impossible even as a supposition.

**The missing tile is now Cedar 5 (6 to 7½).** It is the film's own one skip, shown in 12_2 and named in 12_7 as "6 to 7 1/2: not entered at 6". At house 6 the rule really gave Birch 2 (4⅓ to 8⅔), which ends later than 7½ while Cedar 5 waits. So:

- **Up to `then` the picture is the real run:** 0, 2, 4, a run with no hole.
- **Everything after `then` is hollow:** a red ring on the supposed tile, and eight detached chips that are a count, not seats.
- **The real run returns whole:** when the supposition fails, the same 13 fills return in one shade.

The existing `stretch.*` values and the script are unchanged: 6 to 13, 7 seats, 0 + 1 + 4 = 5.

## F1–F5

Lines are in the candidate `both_promises.rs`.

| Finding | Repair | Where |
|---|---|---|
| **F1** hypothetical shown as actual | "Could": fills past 13 fade, marker back to 13 (real 1, 3, 9). "Suppose one were": the 7 fills after 6 fade *before* the red ring appears (real 0, 2, 4), and "?" goes on Cedar 5. "Go back": ink `then` at 6, legend "solid: real up to then, hollow: only supposed". "a tile that ends later": Birch 2 ringed, with a line and "Birch's seat 2 ends at 8 2/3, later than 7 1/2". "no passed tile": every supposed mobject fades, then fresh fills of the real seats 7 to 13 and "the real run at 13 seats: 1, 3, 9". "never hands out": veils on unentered tiles, none over a fill. | 805–1140, `model::stretch_proof` |
| **F2** solid boxes | Morph into stroked green `quota_box` copies, the `no_rule.rs:309-311` pattern. `refill` is gone from the chapter. | 1186–1193 |
| **F3** rings against labels | Hero rings on the whole tiles inside, except Cedar 5, which keeps its red ring: always 0, 1, 4. A guard ties them to the count. Both kinds of ring leave together, so no moment shows 4 beside "= 5". | 1008–1019, 1086 |
| **F4** still picture under the densest sentences | Each dependency acts at its phrase. "and the missing tile too": eighth chip, red. "had not been entered": veils on tiles started before 6, `then` pulses, wait named. "picked a tile ending sooner": both rings pulse, Birch 2 veiled. "ends by now" (a new beat): veils on tiles ending past 13, band, `now` pulses, 5 whole slots stay clear. "one more": count, red chip pulses. "never holds more": sum flashes, a bracket shows room for 7, "8 tiles cannot fit". No word or explanation cut. | 941–985, 1008–1075 |
| **F5** ring on 2,000 | `think(&kit, stage, v(1.2, -2.9))` via `PREDICT_AT`. Right edge 1.93 against the list's 2.28. The review proposed x 0.4; both clear, and 1.2 keeps the ring nearer the card in question. | 347, 65 |

Pulses are `indicate` at scale 1.0 (`pulse_all`, line 218), so a tile's drawn length never grows. F7's 1.25× flash on tiles is therefore absent from the proof; the one `flash_all` there is on the sum, a text.

## New on-screen facts (429 → 443)

The 14 new keys are in `facts.rs`, independently in `oracle.py`, and pinned in a new facts test:

- `stretch.need`
- `stretch.then.{ash,birch,cedar}`, `stretch.now.{ash,birch,cedar}`
- `stretch.later.{town,seat,end}`
- `stretch.missing.{town,seat,start,end}`

`docs/storyboard.md` is rewritten for 12_1, 12_8, 12_9 and 12_10 in `picture` and `features` only. `words` is byte-identical.

## Tests added (10, uncompiled, not run)

**`model.rs` (+5):**

- `the_supposition_starts_from_a_real_moment`
- `no_state_of_the_picture_is_an_impossible_run`, with the old Cedar 8 picture as a negative control
- `the_counts_on_screen_hold`
- `every_tile_is_ruled_out_or_inside_exactly_once`
- `the_real_run_at_now_keeps_both_halves_of_the_promise`

**`facts.rs` (+1):** `the_supposition_on_screen`.

**`both_promises.rs` (+4, layout):**

- `the_think_ring_covers_no_card`
- `the_count_row_keeps_its_parts_apart`
- `the_count_panel_clears_the_axis_and_the_frame`
- `the_labels_in_the_stretch_clear_the_markers_and_rows`

The layout tests compare through a non-constant helper, because a plain `assert!` on constants would trip Clippy (`draw.rs` uses `const { assert!(…) }` for the same reason).

## Checks run, with logs in `checks/`

| Check | Result |
|---|---|
| `proof_picture_check.py`: exact-fraction walk, oracle diff, before/during/after sets, counts, layout boxes per state, the Rust layout inequalities by hand, timing simulation | PASS. Base oracle 429 values, none changed, 14 added. Real 13 → real 6 → same 13. Controls fail as they should: the old picture is no run, Ash 1 crosses `then`, Cedar 9 passes `now`, the old ring hits the list |
| `source_check.py` on candidate | PASS: the 18 phrases in order; 22 supposed mobjects, all faded once; `since_then` fades before the red ring |
| `source_check.py` on base (negative control) | FAILS 33 checks, as it should |
| `roundtrip.py` | PASS: forward and reverse byte-identical on copies; base equals canonical now; `git apply --check` accepts it |
| `rustfmt --check` through `main.rs`, on a scratch crate | exit 0. Parses every module; this is not a compile |
| `phrase_scan.py` | 244 phrases, 0 problems |
| `regen_script.py` | `script.tsv` unchanged, 0 problems |
| `bind_oliver_plan.py --check`, `validate_oliver_plan.py` | **Both stop, on candidate and on canonical alike.** See risk 7 |
| Same two, on a scratch rebind of the candidate | Validator passes, 15 markers. The rebind differs from the candidate's map only in 03_3, 04_3 and 05_3, the holds stale when it ran. The 12_1 hold equals the rebind's |

## Exact remaining risks

1. **Uncompiled.** All Rust here is uncompiled, and its 10 new tests have never run. Types and names were checked by eye against the definitions; the numerics are mirrored in Python. Expect possible compile or Clippy repairs. Facts against oracle in Rust is unproven: the frozen `checks/rust_mirror.py` does not cover the 14 new keys, though `proof_picture_check.py` does.
2. **No frames of any of this.** Each of these is read from source and not yet seen:
   - `indicate` at scale 1.0 as a pure recolour (read at pin a399d626)
   - white on the hero `now` line
   - stroked green morphs staying hollow
   - `flash_all(&boxes[..2], 1.0)` at line 1195 now flashing outlines only (it still scales 1.25)
3. **Native lint may report designed coincidences:**
   - the red ring's left edge lies on the `then` line, because Cedar 5 starts exactly at 6
   - Birch 3's right edge lies on the `now` line, because it ends exactly at 13
   - `then` crosses Birch 2's ring
   - the `then` marker sweeps across "?" on its way from 13 to 6
   - veils overlap fills

   Judge these on the frames.
4. **Text widths are estimates.** I used 0.0048 of size per character, against 0.0042–0.0045 measured on this film's frames. Heights are also estimates. The tightest inequality is the prediction note's bottom, by 0.04, at the same y as every other chapter's default spot.
5. **Timing is a simulation.** It uses character shares of the scratch lengths (12_8 29.246 s, 12_9 36.169 s) at 0.8, 1.0 and 1.2 times. Worst lateness is 0.30 s, and the picture ends before the voice at all three. No Oliver takes exist for this, and real sentence onsets will move it. Fine sync is root's.
6. **Teaching judgments I cannot settle from source.**
   - `then` = 6 remains an illustration of a supposition that the theorem says cannot happen.
   - Under the supposition the seven hand-outs are not placed on tiles: chips count them and rings mark the whole slots, so "So all those tiles fit" points at where they would have to go.
   - A sharp viewer may ask why Ash 1 (entered at 6, ends 9¾) is out: it could only be handed out after the "last moment" by ending later than the missing tile. The veil treats it as entered at `then`, which is true, and the speech covers it, but only just.
   - 12_8 now carries 10 phrase beats in about 30 s, so density is a motion judgment.
7. **Plan holds go stale as other chapters change in canonical, and this patch does not touch them.** At 09:13Z four were stale, by hash only, with markers still at their lines:
   - 03_3 `largest_tails.rs` (map 2df4984d9bb6c99d, file d15246943cc8828c) and 04_3 `sixteenth.rs` (16d00b28154994f2, fbf41f13db0d933d): root's result-handle cleanup, Mail 1294
   - 05_3 `dials.rs` (cce01aa2b55ca52a, ba3db7f7ec45ed58): changed 09:00:42Z
   - 06_1 `not_every_time.rs` (b6d46197836253c5, d3e88f7bcad146b9): changed 09:11:44Z

   None of these files is in the patch. The logged rebind predates the last change, so it shows only the first three. Until root rebinds, `--check` and the validator stop there, as they already do on canonical. A full rebind writes the same 12_1 hold this patch does.
8. **The plan's hashes change.** The map's 12_1 hold moves to line 347 with the new marker, and the units' `direction_contract.map.sha256` becomes 3fc8199165851dc2. No unit text changes. Anything that bound the old map hash needs root's rebind.
9. **Canonical moves.** The base equals canonical as of the last `roundtrip.py` run. If root has edited any of the 7 files since, the patch needs a rebase; check `MANIFEST.json` first.
10. **Outside this grant, left as they were:** F6, F7's flashes outside the proof (including line 1195), and F8–F13.

## Frozen and untouched

The prior review (`preview12-picture-review01.md`, 1b512984370b4e8f), `candidate/`, `snapshots/`, `checks/`, `HANDOFF.md` and `ORDERS.md` one level up, and the canonical crate. No Cargo, render, synthesis, commit, push or Beads action. No memory file was changed, as the grant requires.
