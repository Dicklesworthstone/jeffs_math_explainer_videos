# continuation14: the pole's mark under the disks in 14_3

BronzeJaguar, for BlueSalmon, answering Mail 2631. Bead 87do.39. Narration is ElevenLabs Oliver.

**State:** a narrow draw-order repair is staged and frozen for root. **Nothing here has been compiled, tested by Cargo, rendered or heard.** I wrote this chapter, so this is not independent. No canonical or shared file, Bead or commit was touched. Cue words are unchanged, and there are no new strings or numbers. This folder has its own seal, so the chapter 13 hashes in `../SHA256SUMS` do not move.

## What the frames show

You were right about 66.5–74 s. **The same fault starts earlier:** the gold disk's rim runs through the ring for about 6 s, from 59.1 to 65.8 s, and the region's fill tints the mark from 50.1 s.

Measured on all frames02 frames from 43 to 85.6 s, against frame 103. "Difference" is the mean over the mark's disc, 0–255. On this lossy video a truly clean mark reads up to about 3.5.

| Local time (scratch voice) | What is over the mark | Difference | Ring pixels surviving | Reads as |
|---|---|---|---|---|
| 43–49.9 | Nothing | 0–0.4 | 100% | Clean |
| 50.5–59 | Region fill | 18.5 | 100% | Ring fine, hole tinted teal |
| **59.7–65.5** | **Gold disk** | **43** | **38%** | **Gold stripe down the ring** |
| 65.8–66.3 | Nothing | About 3 | 100% | Clean, briefly |
| 66.5 | 1 green disk | 18 | 85% | One rim across the ring |
| 68.0 | 3 green disks | 51 | 24% | Ring breaking up |
| 69.0–71.5 | 4 green disks | 62.5 | 12% | Green star on a pink smudge |
| **72.5–74.0** | **6–8 green disks** | **69–73** | **0%** | **No clean ring pixel left** |
| 74.4–85.6 | Second mark on top | 4.3 | 100% | Restored, ring drawn twice |

**Cause:** `pole1` goes on before all of these, and each later fade roots its mob on top. The second mark at the chain's end was the old cure, and it arrived after the whole chain.

## The repair

Only `continuing.rs`, +48 −4. Disk geometry, fills, strokes, the path, the walker and all narration are unchanged. The only code removed is the second mark (the four `−` lines).

| Change | Effect | Where (staged) |
|---|---|---|
| New `root_under(stage, mobs, mark)`: roots the mobs, then re-adds the mark over them | The fade that follows finds them on stage, so it does not re-root them | `:237` |
| Called before the region's, gold disk's and each chain disk's fade | Every fill and rim that reaches the pole stops at the mark, even while faint | `:462`, `:476`, `:495` |
| `pole_top` removed, with its `gone.extend` | One mark from start to finish, so nothing pops at the chain's end | Was `:503` |
| New test `the_poles_mark_stays_on_top_of_all_that_reaches_it` | Order of each re-add and fade; one mark; the mark too small to hide the walk | `:934` |

**What you will see change, beyond the fix:**

- **The ring is drawn once, not twice.** It will be a touch softer at the edge than your 74.5 s frame, and match 43–50 s instead. Anchor B14 asks you to judge this.
- **The hole stays dark over the region** (the old frames tint it teal), which suits "every z except 1".
- **"Here every path" can arrive up to 0.2 s earlier,** because the second mark's 0.2 s fade is gone. It still waits for sentence 6, and 14_3 still ends on the voice's clock. I make no pacing claim.

The mark cannot hide anything that teaches: the path stays 0.92 from the pole and the walker's edge 0.84 at least, against the backing's 0.13 scene units. The name "1" is 12 px clear of the backing on frame 103.

**Fallback:** I found no chapter that roots a mob and then fades it within a few lines, so this order has no rendered precedent here. I read `begin_animations` at the pin: it roots only what is not already on stage, and no frame is drawn between the re-add and the fade's start at zero opacity, so I expect no flash. Repair03's 05-pole and my chapter 13 spoke rely on the same reading. If it does flash, swap each `root_under` with the fade after it. The rim then shows over the mark for that fade's 0.3–0.6 s, which still fixes the holds.

**Seen, not changed:** the walker and path pieces lie under later disks, so fills tint them and rims cross them. They read fine on the sheets, and this is outside the brief.

## Apply

```
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/continuation14/complex14-pole-order.patch
```

- Patch sha256 `65457b27ecfaee02a82c585770392e513282e85f28b267b811d185fc168c2d38`.
- Base `continuing.rs` is `088034785bc2…`, byte-equal in canonical and in `combined-picture-stage01/candidate` when I checked. Staged result is `c24f264b9837…`.
- **No overlap with LavenderPike's colour plate work.** None of `complex-color-repair03/patches/*.patch` touches `continuing.rs`, and I edit no shared file. Its plate-lab test reads a plane literal from this file, and that literal is still there. Nor do repair03's or the chapter 13 patch touch this file. See `checks/outputs/overlap.txt`.

## Bindings

**Hold rows:** `bindings/visual-holds-delta.json`, against `docs/narration-direction-map-oliver10.json` (`341356cda34a…`). Replace only the listed fields. Sections other than `visual_holds` do not change.

| Cue | Kind | Old line | New line | Marker |
|---|---|---|---|---|
| 14_3 | reading | 512 | 522 | same |
| 14_4 | prediction | 522 | 531 | same |

**Phrase hooks:** `bindings/phrase-hooks.tsv` lists 30 kept, none added or removed.

**Binder:** the frozen `build()` still refuses on canonical with the same 8 existing problems (added 0, gone 0).

## Native review anchors

`native-review-anchors.tsv` lists 16 poses with what each must show. Times use these, on the render's own clock:

```
L   = 14_3 start in the render's cues.tsv
Sj  = start of sentence j of 14_3, as line.sentence(j) returns it:
      S1 "Two extensions", S3 "Re-centre", S4 "Chain disks", S5 "Each step", S6 "Here every"
W   = max(S5, S4 + 3.4)        the chain's first four steps can overrun sentence 5
```

The k-th green disk starts fading in at `S4 + 0.8 + 0.65(k−1)` for k up to 4, and `W + 0.3 + 0.65(k−5)` from 5. The chain ends at `W + 2.9`.

On preview02 these read S1 ≈ 50.1, S3 ≈ 58.8, S4 ≈ 65.5 and W ≈ 71.25, from decoding that video every 0.1 s (`outputs/preview_times.txt`). Those times are only for comparing with the old frames, and **they are not Oliver's**.

**A number to check on the next preview:** run `checks/pole_frames.py` on its frames folder, with REF taken from the scene's first second. From B1 until the fade at B15, expect a difference of about 5 or less with 100% of the ring on every sampled frame, against 18–73 now. Between samples, the anchors are what count.

## Checks run

All in `checks/`, outputs in `checks/outputs/`.

| Check | Result | What it cannot show |
|---|---|---|
| `rustfmt --check`, staged file and `main.rs` of canonical + repair (parses every module) | exit 0 | Types, borrows, names |
| Patch round trip on scratch copies | apply equals staged, `-R` equals canonical | Later merges |
| `draw_order_check.py`, walking 14_3's statements with the engine's rooting rule | Canonical: 13 faults. Repair: none over 18 fades. **All 9 planted faults caught.** A Python mirror of the new test's text checks holds on the repair and fails on canonical | Pixels. It models z-order from source text |
| `pole_frames.py`, `preview_times.py` | The table above and the sentence times | **Old source only** |
| `synthetic_pole.py`, `outputs/synthetic-pole-NOT-A-RENDER.png` | The clean mark's pixels pasted over 9 old poses and 3 full frames, for judging the look. It reads clearly | **Not a render.** Edge feathering is the script's |
| Desk checks, canonical against repair | Scope and desk nothing new. Arity, shape and unused differ only in count lines. `cauchy.rs:21` is the one unused row, and it is on canonical too | Anything needing the compiler |
| `binder_compare` (in `run_checks.py`) | Added 0, gone 0, two hold rows derived | Any audio time |

**Pictures read:** sheets 034, 037, 039, 040 and 041; zooms of 18 frames from 44 to 80 s; the synthetic sheet. Other frames were measured, not read by eye.

## Not run, and the limits

- No `cargo` of any kind, so **the new test is unrun**. No render, ASR, TTS or listening. No native picture of this repair exists. Sampled frames are not continuous AV, and the synthetic sheet and the walk are not native proof. No listening or pacing acceptance is claimed or implied.
- All times use scratch-voice rates.
- **Compile risk is low:** `add_to_scene` is used across this crate, and `pole_mark` returns `[Mob; 2]`. The new parts are the `mobs.iter().chain(mark)` loop and the `(w[0] + w[1]).scale(0.5)` line in the test. Only rustc can say.
- **Hold rows are bound to this exact file hash.** Any later edit to `continuing.rs` moves them again.

## Regenerate

```
python3 -B checks/run_checks.py REPO FMN_CRATES FRESH_SCRATCH_DIR
```

It writes under this folder and the scratch directory only, stops if canonical has moved, and needs a new scratch directory each run. PIL needs site-packages, so the driver and picture checks run without `-I`. It does not rewrite this file, `native-review-anchors.tsv` or `SHA256SUMS`.
