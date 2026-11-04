# Apportionment 12 repair02: source review and N3 recommendation

LavenderPike, 2026-11-04. Answers Mail 1548 (BlueSalmon), thread `mx-video-program-program-87do.30`.
Candidate reviewed: `root-apportionment-proof-repair02/candidate/.../both_promises.rs`, sha256 `ae70d701b9b5515d41fc865815741cfeb588f7d2fdc6a681861b7b0895d332c9`.
This is a source review. Nothing here was compiled, rendered or heard, and it is not acceptance of any picture.

## Verdict

- **Take the candidate.** By reading, I found no compile, handle-lifetime or word-timing blocker. All seven fixes do what `repair-check01.json` says, and root's label change does what Mail 1548 says.
- **N3: use the rewind in `n3-rewind.patch`, not my earlier sketch.** Root is right that my sketch risks F1, so I withdraw it. In the new order, nothing names a missing tile until its fill has gone.
- **Two small changes before the render:**
  - S1: replace my N2 legend morph with a separate key that fades in with the first grey (`grey-key.patch`). The morph was my idea, and it has the same flaw as N6.
  - S2: update the storyboard's picture column. It now contradicts the film at 12_9.

| ID | Sev | Problem | Action |
|---|---|---|---|
| N3 | Medium, judgement | Seven fills vanish on "Suppose one were", 3.5 s before their reason | `n3-rewind.patch` |
| S1 | Low, cheap to fix | Legend morph re-lays 45 unchanged glyphs and delays the first grey by 0.4 s | `grey-key.patch` |
| S2 | Low, doc only | Storyboard still says rings land at "So all those tiles fit" | Wording below |
| S3 | Low, cosmetic | Red ring tints, then snaps red, when `bring_to_front` follows a fade | Look on frames; I would leave it |
| S4 | Low, optional | The still at 12_9's start grows from 5.0 s to about 5.5 s | Optional marker pulses |

## How independent this is

Independent on execution, not on design. Most of these fixes follow sketches in my own review, so I am a poor judge of whether they are good ideas. S1 exists because I looked harder at one of them. I did not write root's patch, and I read it line by line against the helpers. **`n3-rewind.patch` and `grey-key.patch` are mine and have had no independent review.**

## What I checked

Evidence is reading plus the commands named here. `cargo` was not run.

| Check | Result |
|---|---|
| Patch bytes | `repair02.patch` on `baseline` gives `ae70d701`. `baseline` equals canonical `c667ad1a`. The only change from `before-chip-label-followup01.rs` is the `counted` block |
| rustfmt 1.10.0-nightly, `--check` | Candidate exit 0. This proves it parses and is formatted, not that it compiles |
| Imports | `pause`, `BIRCH`, `morph_all` and `run_tracker` already imported. `flash_all` keeps other callers, so no unused import |
| `stage.scene_mut()`, `?` on `SceneError` | At pin a399d626: `fmn/src/lib.rs:1496`, `fmn/src/lib.rs:1397`. Same call form at `bakery_algorithm/.../room.rs:289`, inside `fn construct -> fmn::Result<()>` |
| `bring_to_front` | `runtime.rs` calls `add`, and `add_many_to_scene` removes the families, appends, then stable-sorts by `(z_index, position)`. Re-adding rooted handles is the intended use. f566 shows the band drawing over this text, so position decides the order here |
| `fit` rings built early | `ring` calls `stroke`, which only does `arena_mut().add` (`draw.rs:83`). Detached, so nothing shows until the fade |
| Word phrases | All five `wait_for` phrases in the changed code are in `script.tsv`, in spoken order |
| `pulse_all` on hollow chips | `indicate` at scale 1.0 recolours only. f581 (204.0) shows the eighth chip yellow and still hollow, so pulsing all eight keeps "hollow because it stands for a count" |
| Tests | `the_count_row_keeps_its_parts_apart` guards the 41-character label, and the new labels are 21 and 33. The legend test checks height only. `every_line_is_said_once_in_order` is unaffected, since there's no new `say` |
| Width on frames | Legend measures 481 to 798 px on f566, so 63 characters would span about 418 to 862 px at y 95. Clear of "suppose" (y 62) and "now" (y 134) |

**Handle lifetimes:**

| Handle | Rooted | Changed | Removed |
|---|---|---|---|
| `need_t` | 12_8, "Every tile handed out since then" | Morph to `counted`, then `need_count` | `supposed`, line 1111 |
| `counted`, `need_count`, `legend3` | Never: `Transform` targets stay detached (`draw.rs:924`) | None | Nothing to remove |
| `legend` | 12_8 landing | Morph to `legend3` | `supposed` |
| `fit` | "But each town's row fits at most" | None | `supposed` |
| `gone`, `ask`, `wait_t` | Earlier, re-added by `bring_to_front` | None | `supposed` |
| `c15[ASH]`, `c15[CEDAR]` | Never | None | Arena only, harmless |

Repeated morphs on one retained handle are already used on `c_j` (lines 409 to 463). Root's label order is right: `morph_all` then `fade_in_all` play one after the other, so no frame shows the red chip under "handed out since then".

**Correction to my REVIEW N6:** I said lines 409 to 534 might want the same treatment. Cedar only reaches two digits at the walk's last step, so 499 to 504 is the only morph of an unchanged two-digit counter, and root's fix covers it. Unchanged single digits were not seen garbled on any sample, though half-second sampling is weak evidence for a short morph.

## Word timing

**No Oliver take for 12_8 exists to measure.** I searched the root workspace for `*oliver_12_8*` and the rebuild folder for `*12_8*`, and found none. So this is a tolerance, not fine sync. `checks/n3_and_timing.py` re-implements `Spoken::at` on scratch lengths, and its phrase times land within about 0.3 s of frame events (f488, f532, f563, f579, f590). Times are rounded up to whole frames, as the engine does.

"Tol" is how much faster than scratch the voice could run in that window before the picture passes the next phrase.

| Window starts at | Scratch s | Window | Baseline | Candidate | Tol | With N3 | Tol |
|---|---|---|---|---|---|---|---|
| "Suppose one were" | 170.44 | 1.43 | 0.80 | 0.80 | 1.78 | 0.40 | 3.57 |
| "Go back to the last moment" | 171.87 | 3.73 | 2.20 | 2.20 | 1.69 | 2.77 | 1.35 |
| "Every tile handed out since then" | 182.17 | 2.69 | 0.90 | 1.20 | 2.25 | | |
| "and the missing tile too" | 184.87 | 2.06 | 0.30 | 0.60 | 3.43 | | |
| "had not been entered at that moment" | 186.93 | 2.93 | 1.10 | 1.50 | 1.96 | | |
| "So all those tiles fit inside the stretch" | 197.30 | 6.13 | 1.00 | 0.60 | 10.22 | | |
| "one more of them than seats added" | 203.43 | 3.83 | 1.00 | 1.00 | 3.83 | | |
| "But each town's row fits at most" | 207.26 | 6.05 | 1.20 | 2.20 | 2.75 | | |

The tightest window is the N3 rewind, at 1.35. Scratch runs at about 2.6 words a second here (75 words in 29.246 s), so the voice would need to exceed about 3.5 words a second to pass it. `wait_until` doesn't fail in that case; the next beat just starts late. Re-check this window once takes bind.

**F6** adds 1.0 s to the chapter, and the 12_3 to 12_4 gap goes from about 1.2 s to about 2.2 s. If the Oliver take runs longer, `wipe` holds for the voice, so the pause only ever guarantees the hold.

## N3 recommendation

**Withdrawn:** my sketch put the ring and "?" in at "Suppose one were" and kept the fills until "Go back". That leaves a red "?" on Cedar 5's green fill, with Cedar 6 to 9 filled beside it, for about 2.5 s. It says "missing" over a tile that is visibly handed out. Don't implement it.

**Recommended (`n3-rewind.patch`, applies to `ae70d701`):**

1. At "Suppose one were": the red caption only. No fills change, no tile is marked.
2. At "Go back to the last moment": the ink marker appears, then walks back from 13 to 6 as 12_7's walk in reverse. Each step removes the newest fill, then moves the marker one house.
3. On landing: `then` and the legend, then the ring and "?" on Cedar 5, which is hollow by then.

| Scratch s | Screen |
|---|---|
| 170.44 to 170.84 | "suppose: a passed tile not handed out" fades in; real run 1, 3, 9 still whole |
| 171.87 to 172.17 | Ink marker fades in at 13 |
| 172.17 to 174.04 | Fills leave one per house: C9, B3, C8, C7, A1, C6, C5 |
| 174.04 to 174.34 | `then` and legend |
| 174.34 to 174.64 | Red ring and "?" on hollow Cedar 5 |
| 175.60 | "a tile that ends later than the missing one", as now |

**Why this doesn't restore F1.** From 174.64 on, the set of solid tiles and the marker positions match the candidate. Only the stacking of `gone` and `ask` differs, and the candidate's later `bring_to_front` puts them on top anyway. Against the rows of the author's F1 table:

| F1 row | Rewind |
|---|---|
| Hole in a row (Cedar 8 unfilled, Cedar 9 filled) | Never. `checks/n3_and_timing.py` confirms each of the 7 states is the real run at the marker, with no hole |
| "the last moment ... later than the missing one" contradicted by later fills | The later fills are leaving before `then` or the missing tile exists on screen, and are gone by landing |
| "Every tile handed out since then ... had not been entered" | Spoken at 182, long after they have gone |
| "ends by now" | Same |
| Counting whole fills inside the band | No band until 194 |

It also answers N3 itself. The voice says "Go back" and the screen goes back, so the fills leave for a reason being spoken, in the grammar the viewer saw three seconds earlier when the fills after 13 cleared as the marker returned (lines 859 to 860).

**What remains, for root to judge:** the whole real run sits under the red caption for about 1.3 s before it starts to leave. No tile is marked and the caption is a general hypothesis, so I read this as a question on screen, not a contradiction. If root reads it as F1, hold the caption back to "Go back" too and add it to the `then_mark` fade. That removes it entirely but gives "Suppose one were" no picture beat.

**Fallback if the steps stutter:** replace the loop with `fade_out_all(stage, &since_then, 0.4)?; run_tracker(stage, then, from as f64, 1.2)?;`. That is exactly the earlier beat's fade-then-run, and the ring and caption order stays as above. Stutter is a motion question that source can't answer. Steps are 0.12 s because that costs the same at 30 and 60 fps (4 and 8 frames, 1.867 s in all). 0.1 s would round up unevenly, since its binary value sits just above a whole frame.

**If root prefers no change:** the candidate is honest as it stands. Its fault is order, not truth. I would still change it, since the render is happening anyway and everything after 174.64 is unaffected.

**On the first render, look at:**
- From "Suppose" to landing, every sample's solid set should equal a real run. That is 1,3,9 until the rewind starts, then the states listed in `checks/n3_and_timing.out.txt`, ending at 0,2,4.
- No frame should show the ring or "?" with Cedar 5 filled.
- The ring should land before "a tile that ends later".
- Motion, for stutter.

## Findings on the candidate

### S1 Low. My N2 morph has the flaw N6 was about

`morph_all(&[(legend, legend3)], 0.4)` turns a 45-character line into a 63-character centred one, so all 45 unchanged glyphs slide about 0.7 units left while being re-paired. The film's own morphs garble mid-transit (f168). This one runs on "had not been entered at that moment", when the viewer should be looking at the rows, and it delays the first grey by 0.4 s. This is unrendered, so the size of the effect is unknown.

**Fix (`grey-key.patch`, applies to `ae70d701`, mine, uncompiled):** leave the legend still and fade in a separate `grey: ruled out` at `x = 2.0` in the same call as the first veils, so the key and the first grey arrive together. It adds one handle to `supposed` and two guards to the existing label test. By the frame measure above it ends about px 927 at y 95, clear of "now". It returns this window to its baseline 1.10 s.

**Morphs I would leave:** `need_t` to `counted` to `need_count`. There the words really change, and that is the meaning of the moment. Expect 0.3 to 0.4 s of scrambled caption; judge it on frames.

### S2 Low. The storyboard contradicts the film

`docs/storyboard.md` is outside repair02. No test binds its picture column, so nothing fails. But it is the chapter's written spec, and a later reviewer could "repair" the film back to it. Suggested picture-column wording only; `script.tsv` should not change, which `tools/storyboard.py --check` can confirm:

- **12_9, replace** "At "So all those tiles fit" hero rings on the whole tiles inside the band; the missing one keeps its red ring, so the rings count 0, 1, 4 from here on. At "one more of them" beside the chips "7 seats from then to now: 7 + 1 = 8 tiles" in red, and the red chip pulses. At "But each town's row fits at most" under them ...
- **with** "At "So all those tiles fit" the chips pulse: they are the count. At "one more of them" the label beside the chips becomes "7 seats from then to now: 7 + 1 = 8 tiles" in red, and the red chip pulses. At "But each town's row fits at most" hero rings on the whole tiles inside the band, the missing one keeping its red ring on top, so the rings count 0, 1, 4; under them ...
- **12_8 chips:** "seven hollow chips under the axis, one per seat, labelled "handed out since then"; at "and the missing tile too" the label becomes "since then, plus the missing tile" and an eighth chip appears, red."
- **12_8 veils:** add the grey key, in whichever form root takes.
- **12_8, if N3 is taken, replace** the "Suppose one were" and "Go back" sentences **with** "At "Suppose one were" only "suppose: a passed tile not handed out". At "Go back" a second, ink marker walks 13 → 6 and the seven fills after 6 leave one per house, newest first, every state a real run; "then", with "solid: real up to then, hollow: only supposed", and Cedar's fifth tile (6 to 7 1/2), the wait 12_2 and 12_7 showed, is ringed red with "?"."
- **12_5:** "each town's first tile flashes" becomes "pulses, with no growth".

### S3 Low. Stacking changes when the fade ends

`bring_to_front` runs after each fade finishes. So the band's 0.4 s fade tints the red items, and on one frame they snap back to red (about 194.4). The same happens with the rings: Cedar 6's gold covers the red ring's right edge for about 1 s, then red returns on top (about 208.3). Both are small. With the ring on top, the white `then` line also gets a short red segment across Cedar's row. I would leave all of this and look at it on frames.

### S4 Low, optional. A slightly longer still

The rings used to end at 198.4; the chip pulse ends at 197.9. So the still before "one more of them" grows from 5.0 s to about 5.5 s, while the later one shrinks from 5.0 s to about 3.8 s. If root wants a beat, the voice names both markers:

```rust
s.wait_for(stage, "from that moment to now")?;
pulse_all(stage, &then_mark[..1], None, 0.5)?;
pulse_all(stage, &mark[..1], Some(PULSE_WHITE), 0.5)?;
```

Not needed for correctness, and not in either patch.

## Patches

Both are proposals against `ae70d701`, not applied to anything but scratch copies. They touch separate hunks.

| Applied | Result sha256 |
|---|---|
| `n3-rewind.patch` | `7e2ed19ef731778677ed11948ca90ce4937e84e8c56de64d51131c81da15d1c2` |
| `grey-key.patch` | `020f6ad6d144933aad1b1a43bb708594f815141385441626951109630e92ca75` |
| Both, either order (same bytes) | `c00c98e4f6b4b3094cfcd8643ecea1d61e5157963459b589aecdd17aa34c9144` |

All three results pass `rustfmt --check` with the repo's `rustfmt.toml`. All three are uncompiled, and the new guards have not been run.

## Not done

No `cargo`, Clippy, tests, oracle, render, synthesis or listening. No canonical, source, Beads or memory edits. Nothing here accepts a picture. Everything marked as checked is by reading, rustfmt, sampled frames from preview02, or `checks/n3_and_timing.py`, which is mine and is not native evidence.
