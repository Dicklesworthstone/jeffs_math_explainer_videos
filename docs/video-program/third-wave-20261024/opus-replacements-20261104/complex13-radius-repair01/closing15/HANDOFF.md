# closing15: the growing circle, its labels and the poles' marks

BronzeJaguar, for BlueSalmon, answering Mail 2751. Bead 87do.39. Narration is ElevenLabs Oliver.

**State:** the repair is staged and frozen for root. **Nothing here has been compiled, tested by Cargo, rendered or heard.** I wrote this chapter, so this is not independent. No canonical or shared file, Bead or commit was touched. **Cue words are unchanged.** One on-screen caption changes, as you offered; the numbers do not. This folder has its own seal, so the chapter 13 and chapter 14 hashes do not move.

## What the frames show

You were right about frame 190. **The sweep has four crossings, not one.** The 0.5 s grid catches only the √10 one, so I took the bands from the source's rate and confirmed them on frames decoded between the samples.

| Label | Preview time (scratch voice) | Length | On frames02? |
|---|---|---|---|
| "x = 3" | 64.80–64.95 | 0.15 s | **Missed.** Struck through at a decoded 64.88 |
| "√10 ≈ 3.162" | 65.15–65.68 | 0.53 s | Frame 190 only |
| "−i" | 65.82–66.00 | 0.18 s | **Missed.** Your 66 s frame shows the rim just leaving |
| "i" | 65.88–66.00 | 0.11 s | **Missed**, same |

So at 66 s the circle has not been approaching the names; it has just crossed them. It settles clear by 66.1 s, about 0.2 plane units from their far corners.

**Same class as chapters 13 and 14, beyond your brief.** Everything that runs through a pole lies over its mark. Measured against the mark alone at 47.63 s, mean over its disc, 0–255; a clean mark on this lossy video reads up to about 3.5.

| Local time | Over the mark | Difference |
|---|---|---|
| 11.3–13.6 (15_1) | Gold radius 1 ring, solid band through both marks | Read by eye on zooms |
| 48–61 | Dim radius 1 ring | 17 |
| 61.5–64.6 | Ring and the dashed spoke, into the hole | 19–20 |
| **66.1–72** | **Gold circle straight through ring and hole** | **50** |

## The repair

Only `closing.rs`, +131 −12. **No film time is added and no hook moves**, since `add_to_scene` takes none. Circle, spokes, radius, order and narration are unchanged.

**I chose backing, not a delayed reveal.** The derivation runs spokes → "√10 ≈ 3.162" → circle, and the voice says the root before the circle grows. Delaying the label would break that order, and would not help "x = 3", which is up long before.

| Change | Effect | Where (staged) |
|---|---|---|
| `label_back`: a ground-coloured rectangle, 0.05 scene units past the label | The ring passes behind both labels and they stay whole | `:117`, `:435` |
| `root_under(stage, mobs, on_top)` | Roots the mobs, then re-adds what must stay over them, before the fade or growth that shows them | `:108` |
| Ring grows under backings, labels and marks | Replaces `add_to_scene(ring)`. Rim ends at the marks | `:441` |
| Radius 1 ring and spokes go on under the marks in 15_3–15_4, ring in 15_1 | The marks stay whole from start to fade | `:304`, `:387`, `:412` |
| Names i, −i move left of the axis, in 15_3–15_4 only | The circle never reaches them: nearest 0.40 plane units past the rim, so no backing is needed | `:80`, `:371`, `:378` |
| `all.extend(backs)` | Backings leave with the final fade | `:469` |
| Caption, see below | | `:32` |
| Tests: `the_growing_circle_passes_behind_its_labels_and_never_reaches_the_names`, `the_poles_marks_stay_on_top_of_what_runs_through_them`, a caption-length bound | Geometry, source order, wording fit | `:559`, `:594`, `:654` |

**What you will see change, beyond the fix:**

- **The ring has a bite as it passes a label:** about 0.15 s at "x = 3", where it takes up to about a quarter of the small ring, and about 0.53 s at √10. It is gone by the finished rim, since both backings stay at least 0.2 plane units inside it. On the synthetic sheet it reads as passing behind. **This is my main worry; anchors C11 and C14 ask you to judge it.** `BACK_PAD` at `:84` is the one knob. Dropping the backings altogether would run gold behind gold text, which I think reads worse.
- **The names sit left of the axis in the last two cues,** while 15_1 keeps them on the right. I saw no conflict at the new places: all their pixels are bare ground on the frames at 64.66 and 67.5 s. The change of side falls across the 15_2 strip, so the two poses are never adjacent.
- **The hole of each mark stays dark,** and the rings and dashes visibly stop at the backing. The circle is still seen to pass through ±i, on the line through the mark.

**If `root_under` flashes:** no rendered chapter has this order yet, as with chapter 14 and the 05-pole. I read `begin_animations` at the pin: it roots only what is not on stage, and no frame is drawn between the re-add and a fade starting at zero. Swap each `root_under` with the fade after it and the fix still holds for everything but those 0.4–0.5 s. The ring's call has no fade, so it needs no swap.

**To take only what you asked for,** drop the three marks-only calls (`:304`, `:387`, `:412`), `over_ring.extend(&marks_here)`, `marks`, `marks_here` and those rows of the second test. The labels and names are unaffected. I would keep them: the fault is the same as in chapter 14 and is at its worst on the film's last picture.

**Backings cannot be seen until the ring reaches them:** no pixel of either added band is off ground at 64.66 or 67.5 s (worst 12 of 255, within this video's noise), and by geometry they miss the axis, the radius 1 ring and both spokes.

## The summary card

Staged as you offered: **"output turns count" / "zeros − poles"**, replacing "a loop counts". It is at index 2 of `CAPTIONS`, registered by index in `mod.rs`, so the registry is untouched, and no other file carries the old string.

- **Fit, estimated, not rendered:** about 1.50 scene units in a 2.3 panel, with its longest neighbour at 1.78 measured. The panel's other captions fit as before.
- **Flag, no narration changed:** the voice in 15_2 stays "It lets a loop count zeros and poles". **The caption's subject now differs from the voice's.** The meaning agrees, and the voice is not wrong, as it speaks loosely of the loop. The panel still draws only the input loop.
- **Worth a look before you take it:** `"a loop's output turns" / "count zeros − poles"` (about 1.67 and 1.55 units) keeps the loop that the panel and voice both name, and adds the distinction. **I slightly prefer it.** Swap only `:32`; no line moves, and only the hold row's file hash changes, so re-run the driver. The checks and the length bound already allow it.
- **WhiteSwan:** I asked in Mail 2818 (cc you). Their Mail 2864 to you says the wording "doesn't conflict" with teaching02; it answers for `output turns count`, not the alternative. No patch of theirs touches `closing.rs` or carries the old caption (`outputs/overlap.txt`).

## Apply

```
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/closing15/complex15-closing-sweep.patch
```

- Patch sha256 `361c3c825e0a5a7fa726e83c525c2b696a9f6046722eb943cf72275cf45a671a`.
- Base `closing.rs` is `65ba58acfb38…`. Staged result is `5b6bff1ed831…`.
- **No overlap:** none of LavenderPike's `complex-color-repair03/patches/*.patch`, repair03's, the chapter 13 or 14 patches, or WhiteSwan's complex11 patches touches `closing.rs`. See `checks/outputs/overlap.txt`. I edit no shared file.

## Bindings

**Hold row:** `bindings/visual-holds-delta.json`, against `docs/narration-direction-map-oliver10.json` (`341356cda34a…`). Replace only the listed fields. Sections other than `visual_holds` do not change.

| Cue | Kind | Old line | New line | Marker |
|---|---|---|---|---|
| 15_3 | prediction | 369 | 402 | same |

WhiteSwan's candidate `oliver11` map carries the same old 15_3 row; it differs from `oliver10` only at 11_5, 11_6 and 11_8. So this delta applies to either, and only its `map_sha256` field names `oliver10`.

**Phrase hooks:** `bindings/phrase-hooks.tsv` lists 12 kept, none added or removed.

**Binder:** the frozen `build()` still refuses on canonical with the same 8 existing problems (added 0, gone 0). Two of them are this chapter's unclassified pauses, at 15_1 and 15_2; those holds' lines move, and they are not this repair's.

## Native review anchors

`native-review-anchors.tsv` lists 21 poses with what each must show. Times use these, on the render's own clock:

```
Q    = time of "at i and minus i" in 15_1
S4   = start of sentence 4 of 15_2
L3   = 15_3 start, L4 = 15_4 start, from the render's cues.tsv
P    = time of "the square root of ten" in 15_4
G0   = P + 0.4                         the circle starts growing
t(r) = G0 + 1.4 × (r − 0.02) / 3.142   when the circle has radius r
```

On preview02 these read Q ≈ 10.7, S4 ≈ 28.5, P ≈ 64.265 and G0 ≈ 64.665. Those times are only for comparing with the old frames, and **they are not Oliver's**. The circle's 1.4 s is fixed, so the bands above keep their lengths on any voice.

## Checks run

All in `checks/`, outputs in `checks/outputs/`.

| Check | Result | What it cannot show |
|---|---|---|
| `rustfmt --check`, staged file and `main.rs` of canonical + repair (parses every module) | Exit 0 | Types, borrows, names |
| Patch round trip on scratch copies | Apply equals staged, `-R` equals canonical | Later merges |
| `sweep_check.py`: bands from the source, and a walk of 15_1 and 15_3–15_4 with the engine's rooting rule | Canonical: 8 faults. Repair: none. **All 12 planted faults caught.** A Python mirror of the new order test holds on the repair and fails on canonical | Pixels. It models z-order from source text. Label sizes are measured off old frames |
| `preview_times.py` | The tables above, decoded between the samples, and the bare-ground count | **Old source only** |
| `synthetic_sweep.py`, `outputs/synthetic-sweep-NOT-A-RENDER.png` | Pixels pasted on 10 old poses and 6 mark zooms, for judging the look. It reads clearly, with a visible bite at 64.88 and 65.2–65.55 | **Not a render.** Edges and name places are the script's |
| Desk checks, canonical against repair | Scope and desk nothing new. Arity, shape and unused differ only in count lines | Anything needing the compiler |
| `binder_compare`, `strings.txt` | Added 0, gone 0, one hold row derived. Strings: one new, one gone | Any audio time |

**Pictures read:** sheets 047 and 030; frame 125; decoded zooms of both poles at 16 times from 10.8 to 13.8 s and at 62.5–70 s; decoded crops of the sweep at 14 times; the synthetic sheet. Other frames were measured, not read by eye.

## Not run, and the limits

- No `cargo` of any kind, so **the new tests are unrun**. No render, ASR, TTS or listening. No native picture of this repair exists. Sampled frames are not continuous AV, and the synthetic sheet and the walk are not native proof. No listening or pacing acceptance is claimed or implied.
- All times use scratch-voice rates. The label half sizes in the new test are measured on preview02 and rounded up; a font change would need them re-measured.
- **Compile risk is low but real.** `Rectangle` without `corner_radius` is optional by my reading of `fmn-library/src/poly.rs`, and `card` is the pattern. `all.extend(backs)` extends from an array, and `over_ring.extend(&marks_here)` copies as `gone.extend(&pole_mobs)` already does. Only rustc can say.
- **The hold row is bound to this exact file hash.** Any later edit to `closing.rs` moves it again.

## Regenerate

```
python3 -B checks/run_checks.py REPO FMN_CRATES FRESH_SCRATCH_DIR
```

It writes under this folder and the scratch directory only, stops if canonical has moved, and needs a new scratch directory each run. PIL needs site-packages, so the driver and picture checks run without `-I`. It does not rewrite this file, `native-review-anchors.tsv` or `SHA256SUMS`.
