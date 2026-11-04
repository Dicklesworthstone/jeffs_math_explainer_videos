# Apportionment 12_8: independent review of the rewind and grey key

Reviewer: BronzeJaguar. Assignment: Agent Mail 1584 from BlueSalmon, thread `mx-video-program-program-87do.30`. Date: 2026-11-04.

I wrote neither patch, nor root's repair02, nor chapter 12. LavenderPike wrote the patches and WhiteSwan the chapter.

**Status: source reading and exact arithmetic only.** Nothing was compiled, tested, run through Clippy, rendered or heard. No frame was looked at. Oliver has no take for 12_8, so this is not listening or fine sync.

## Result

**No fix is needed. Both patches are right as staged, and the rewind does not restore the red ring over a real fill.**

| Question | Answer |
|---|---|
| Can the ring or "?" show over Cedar 5's fill? | No. See the next section |
| Mathematics | Holds. Two exact derivations of the run agree through 16 seats, and Cedar 5, Birch 2 and the five whole tiles come out as the film says |
| Fill order | Newest first: C9, B3, C8, C7, A1, C6, C5. After each fade the fills are exactly the real run one seat earlier, with no hole in any row |
| Marker state | The ink marker lands on exactly 6. The hero marker, its count and the `now` label stay at 13 |
| API and handle lifetime | Sound. Every call matches the pin and the crate's helpers |
| Cue and phrase order | Sound. Neither patch adds or removes a `wait_for`, so phrase marks need no rebinding for this change |
| Grey key | Sound. It is rooted once, leaves with `supposed`, and has no new glyph |

Three items remain for root (below). None blocks the build.

## Bindings

| Item | sha256 |
|---|---|
| Root candidate `root-apportionment-proof-repair02/candidate/…/both_promises.rs` | `ae70d701b9b5515d…` |
| `apportionment-proof-repair02-source-review/n3-rewind.patch` | `8e726f41c6d6ab41…` |
| `apportionment-proof-repair02-source-review/grey-key.patch` | `2ed99396251ab675…` |
| `combined-repair-stage03/…/both_promises.rs` | `c00c98e4f6b4b309…` |

Applied to a scratch copy of the candidate, the two patches give the staged file byte for byte, in either order, with no fuzz. `rustfmt --check` exits 0 on each result. `script.tsv` and `model.rs` in the stage equal canonical.

Line numbers are the staged file's. Engine paths are under `crates/` at pin `a399d626`.

## Why the ring cannot sit on the fill

1. **The ring and "?" are detached until line 912.** `gone` comes from `ring` → `stroke` → `arena_mut().add` (`draw.rs:83`). `ask` comes from `Kit::text`, which also only adds to the arena (`kit.rs:403`). Lines 869–870 build them, line 911 is a comment, and line 912 is the first call that uses them.
2. **The fills have all left by then.** Each `play` runs to its end before the next call, and `fade_out` is a remover (`fmn-anim/src/fading.rs:94`), so the loop at line 892 finishes all seven fades first.
3. **Cedar 5's fill is always in that loop.** `missing` is a town's next tile at `from` (`stretch_proof`), and line 833 stops the render unless the fills match the real run. Exactly, it is `since_then[0]`, hand-out 7, the last to leave.
4. **Nothing covers that tile again while the ring is up.**
   - `before` needs a start under 6, and Cedar 5 starts at 6.
   - `after` needs an end over 13, and it ends at 7½.
   - `fit` excludes it (line 1048).
   - `back` restores its fill only after line 1147 has faded `gone` and `ask`.
   - `unentered` needs a start of 13 or more.

At "Suppose one were", only the caption fades in. No tile is marked.

## The run, exactly

Tiles are from (k−1)·39/p to k·39/p, in units of a thousand people. I derived the run from "the entered, unheld tile that ends soonest", and again from Jefferson's list with the upper-quota skip. They agree through 16 with no ties.

| Hand-out | Tile | Span | Marker after its fade and step | Seats after |
|---|---|---|---|---|
| 13 | Cedar 9 | 12 to 13½ | 12 | 1, 3, 8 |
| 12 | Birch 3 | 8⅔ to 13 | 11 | 1, 2, 8 |
| 11 | Cedar 8 | 10½ to 12 | 10 | 1, 2, 7 |
| 10 | Cedar 7 | 9 to 10½ | 9 | 1, 2, 6 |
| 9 | Ash 1 | 0 to 9¾ | 8 | 0, 2, 6 |
| 8 | Cedar 6 | 7½ to 9 | 7 | 0, 2, 5 |
| 7 | Cedar 5 | 6 to 7½ | 6 | 0, 2, 4 |

- Hand-out 6 is Birch 2 (4⅓ to 8⅔), which ends later than 7½. Cedar 5 is the only tile that qualifies as missing.
- Each step reverses 12_7, where the marker moves and then the fill arrives, so the rewind passes through the same states.
- The veils cover every tile exactly once. Whole tiles inside the stretch are 0, 1 and 4, which is 5 against room for 7 and a need of 8.

## API, handles and marker

- **Loop:** `since_then` is a `Vec<Mob>`, and `Mob` is `Copy`. `enumerate().rev()` works on a slice iterator. `(from_h + i) as f64` ends at `from`, because line 835 fixes the length at `to − from`.
- **No double fade:** `since_then` is used only at lines 843 and 892. `fills` is not read after line 832. So no fill is faded twice, and a fill can't flash back when a later fade re-roots it.
- **Short plays:** 0.12 s is allowed, and the file already uses 0.1 s (line 472). `fade_out_all` clears updaters on the fill only, so the marker's updater keeps running.
- **Frames:** `n = ceil(run_time × fps)` on the f64's exact value (`fmn-anim/src/clock.rs:189`). 0.12 s gives 4 frames at 30 fps and 8 at 60. The rewind is 56 and 112 frames, 1.8667 s at both rates. From "Go back" to the ring fully in is 83 and 166 frames, 2.7667 s at both.
- **Stacking:** `gone` is now rooted after `then_mark`. The ring's 4.5 left edge lies exactly on the 4.0 ink marker across Cedar's row from landing, where before that began at line 1019. The final stacking is unchanged.
- **Grey key:**
  - `grey_key` is rooted once at line 993 and is in `supposed` at line 1130.
  - `legend3` is gone from the file, and `morph_all` keeps its other callers.
  - The `unentered` veils arrive after the key has left, so "grey: ruled out" is never beside them.
  - `text_left` shares `text`'s vertical centre. The key and legend both have ascenders and descenders, so they share a baseline.
  - Both new guards pass by arithmetic: 2.0 > 1.944, and 3.296 < 6.96. By CMR10 estimate the legend ends at 1.74 and the key spans 2.00 to 3.17.

## Phrase order and timing

Root's `checks01/phrase_scan.py` on the stage reports 93 lines said, 254 literal phrases and 0 problems. Every 12_8 phrase occurs once, in spoken order.

I walked the staged beats in source order, carrying any lag. Times are from the cue's start, using the scratch rule at other word rates, not Oliver.

| Words a second | Rewind ends | Ring in | "a tile that ends later" | "the missing one" | Last beat ends, of line |
|---|---|---|---|---|---|
| 2.0 | 7.93 | 8.53 | 10.58 | 13.46 | 35.37 of 37.90 |
| 2.6 (scratch) | 6.63 | 7.23 | 8.16 | 10.38 | 27.50 of 29.25 |
| 3.0 | 6.03 | 6.63 | 7.09 | 9.02 | 24.00 of 25.40 |
| 3.5 | 5.50 | 6.10 | 6.09 | 7.75 | 20.77 of 21.83 |
| 4.0 | 5.10 | 5.70 | 5.35 | 6.80 | 18.33 of 19.15 |

- The ring is in before "a tile that ends later" up to about 3.5 words a second, and before the words "the missing one" up to about 5.25.
- Lag past that point does not build up, and no beat overruns the line, even at 4.0.
- For scale, all 46 Oliver takes of this film's earlier script run from 1.80 to 3.00 words a second over a whole take, median 2.33, from their WAV headers. Those are different words, and a local rate can differ.

## For root

1. **The storyboard is stale in stage03.** `docs/storyboard.md` still equals canonical and says the fills fade and the ring lands at "Suppose one were". I checked LavenderPike's S2 wording for 12_8 against the staged code, and it is accurate. This is a doc change only, so a later reviewer does not "repair" the film back to it.
2. **Look at these on the first frames.** Source cannot settle them.
   - **Stutter:** the marker rests while each fill leaves, then moves one house.
   - **Whether the walk reads as a search:** six of the seven fills it passes (all but Cedar 5) end later than 7½, so a viewer who reads the walk as a search for "the last moment" could ask why it did not stop at 13. Nothing false is drawn. No tile is named while they leave, and the legend then says the run is real only up to `then`. Even at 4.0 words a second, the rewind ends before the voice states the condition (5.10 s against 5.35 s).
   - **If either one reads badly:** use LavenderPike's fallback, `fade_out_all(stage, &since_then, 0.4)?; run_tracker(stage, then, from as f64, 1.2)?;`. It has the same landing and the same ring-after-fill order, and takes 49 frames at 30 fps instead of 56.
   - **"grey: ruled out"** now sits directly above the label "now", which spans about x 2.47 to 2.86, with about 0.25 between them. The old long legend stopped short of that label, at about x 2.38.
3. **Re-check when takes bind.** The ring should land before "a tile that ends later" and the fills should be gone by then.

## Two small corrections to LavenderPike's review

Neither changes a verdict.

- `checks/n3_and_timing.py` rounds with a 1e-6 slack, which is Kit's wait rule, not the engine's play rule. So it counts 0.4 s as 12 frames at 30 fps where the engine plays 13, and likewise 0.2 s as 6 where the engine plays 7. Its "Suppose" row should read 0.43, not 0.40. The rewind window uses only 0.3 s and 0.12 s, so it is unaffected.
- The old legend line is 62 characters, not 63.

## Not checked

Compile, Clippy, tests, render, frames, motion, listening, Oliver timing and fine sync. Real text boxes are also unchecked: widths here are the test rule, CMR10 estimates and LavenderPike's frame measure. All of that remains with root.

## Files

- `REVIEW.md`, `SHA256SUMS`
- `checks/rewind_checks.py`, `walk_beats.py`, `patch_orders.py`, `oliver_pace.py`, each with its `.out.txt`. The 46 "unread" entries in `oliver_pace.out.txt` are the receipts' `source_download` names, not missing takes; the 46 stored takes were all read.
