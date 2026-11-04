# Complex chapters 06–09: repair of five picture findings

WhiteSwan for BlueSalmon, Mail 2525, bead `.39`. Date: 2026-11-04. Narration stays ElevenLabs Oliver; nothing here touches it.

**Status: staged and not installed. The chapter code has not been compiled or rendered.** The new tests ran only in a stand-in build outside the crate. Root integrates, compiles, renders and judges the pictures.

No cue word changes, so `script.tsv` stays `2968b0fd76540cbc`. No on-screen string changes, and no timed call changes. The film keeps every step of its mathematics.

## What changed

| Finding | The fix | Staged lines | Same as the review? |
|---|---|---|---|
| 06-N1 | On "Squashing fails too", `right: 1`, `up: −1` and the first X leave with the reflection. `right: 1   up: 2` moves up to y 1.3 and the second X to y 0.3, above the kept `no complex derivative`. The closing fade no longer lists what has already gone | `one_derivative.rs:66-68`, `:681-690`, `:701`, `:725`, `:729-740` | Yes |
| 07-N1 | The map arrow's tail starts at x −0.42, about 10 px past `Re`. The tip stays at 0.6 | `one_point.rs:82`, `:241`, `:247` | Yes, plus one addition below |
| 08-N1 | The `z_k` dot and name from 08_1 come back on the same step, in the fade that brings `straight up`, and leave with the product diagram | `loop_sums.rs:462-467` | Yes |
| 08-N2 | All 16 circulation heads are narrower, 0.16 long by 0.11 wide, and the middle of each base sits on the arc | `loop_sums.rs:210`, `:215-227` | **No, see below** |
| 09-N2 | `a = 0` moves to `p.p(1.05, -0.4)`, about 14 px above the lower `−0.5` card and 9 px clear of the small ring's dots | `boundary.rs:92-93`, `:289`, `:382` | Yes |

### Where this departs from my review

- **08-N2: the review's fix was not enough.** It proposed changing one number, `CIRCULATION.1 - 0.62`. I drew that with PIL at native size, and it still read as a triangle pointing down, because a head as wide as it is long has three similar corners. Narrowing the head made the tip its one clear point. `arrow_head_points` ties width to length and lives in `draw.rs`, which is out of scope, so `circulation_head` now builds its own corners. The ring's head at `:307` still uses the helper. I judged this only on a PIL drawing, so **please look at an 8× crop of the real render**. The shape is one constant, `CIRCULATION_HEAD`.
- **07-N1: one addition.** The `z ↦ z²` name moves 0.09 right, to stay centred over the shorter arrow. My review had flagged that offset and left it. To undo this, set `map_at`'s x back to 0.0.

`boundary.rs` and `one_derivative.rs` also gain named constants for positions the new tests read. `SHARE_CARD_FROM_POLE`, `CROSS_AT` and `VERDICT_AT` keep their old values.

## How to apply

From the repo root, first check that canonical has not moved:

```
shasum -a 256 -c docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/baseline.sha256
patch -p1 -i docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/patches/06-one-derivative.patch
```

Do the same for `07-one-point`, `08-loop-sums` and `09-boundary`. Each patch touches one file, so any subset works in any order.

| File | Baseline | Staged | Patch |
|---|---|---|---|
| `one_derivative.rs` | `b67231d94cafe4f5` | `84ed107492e65b39` | `aa3bdf95753793b7` |
| `one_point.rs` | `43b6f1c323c5f715` | `76a8bf1dc2ae4725` | `5f26db201643c60a` |
| `loop_sums.rs` | `cf383504cdc80652` | `5458765d947f2b93` | `fcf2b239ff876a5e` |
| `boundary.rs` | `328d2acbc247eccc` | `69e41733fd0e4d4e` | `30b82a476d058f8c` |

- **`draw.rs` is untouched,** so LavenderPike's colour work does not collide with this. S2 stays with LavenderPike.
- **`complex-glyph-spacing.patch`** (my unapplied `z_{\,0}` patch) applies beside these in either order and gives the same files.

## Hold and phrase deltas, bound to the source

**The delta is 0 at every timed call.** `checks/source_checks.py` lists every timed call in film order, 396 across the four chapters, and they are identical in order and arguments. Items join existing fades, and a fade's length doesn't depend on how many items it holds.

What does change is how long some things stay on screen. Times are scratch voice, read from preview02's frames. Word times marked "est." are word-share estimates, and a gap under about 2 s between one and a beat is unproved.

| Object | Now | After the repair | Tied to |
|---|---|---|---|
| `right: 1`, `up: −1`, first X | until the close, f464 (175.35) | gone by f442 (167.5), about 8 s less | `wait_sentence(4)`, "Squashing fails too" |
| `no complex derivative` | f428 (161.9) to the close | the same; alone in the column f442 to f447, about 2.5 s | unchanged |
| `right: 1   up: 2`, second X | f447 (170.0), f448 (170.2) | the same frames, new place | after the square grows |
| Map arrow | f3 (0.8) to f84 (35.033) | the same | unchanged |
| `z_k` dot and name | absent | new, with `straight up`: f91 (39.0) to f103 (45.0), gone by f105 (46.0) | after "Now one over z.", before "The arrows stack" (est. 45.5) |
| Circulation marks | f180 (78.0) to f204 (90.0) | the same | unchanged |
| `a = 0` | f106 (40.0) to f352 (119.0) | the same, new place | unchanged |

**One sync risk under Oliver.** The dot arrives about 1.4 s before "the turn" (est. 40.4), which is inside the unproved range. It comes 1.2 s after "Now one over z." ends, plus a 0.3 s fade. If Oliver reaches "the turn of the step's position" sooner than that, the dot lags those words. Check this on the rerender.

## Question windows

The source from each question's cue through its first answer is unchanged. Cue 09_9 was compared through the end of the chapter, because it has no second-sentence marker to stop on.

| Question | Effect of the repair | Risk added |
|---|---|---|
| 06_8 to 06_9 | 06-N1 acts on 06_9's fifth sentence, after the answers have started at f415 (156.5) | None |
| 07_3 to 07_4 | The map arrow has left by f85 (35.333), before the question at f97 (41.0) | None |
| 09_4 to 09_5 | `a = 0` stands in its new place through the think time, f183 to f194. It gives nothing away, and the old card never covered a pole | None. 09-H, the clue cloud, is unchanged and stays root's call |
| 09_8 to 09_9 | `a = 0` has been gone since f354 | None |
| Chapter 08 | It has no question. The new dot answers nothing | None |

These are source and preview02 facts. All the windows still need rechecking under Oliver, as they did before this repair.

## What to look at on the rerender

Frames are preview02's, under `narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-combined-picture-frames02/`. `checks/anchor-frames.sha256` has their hashes. With no timed call changed, a rerender on the scratch voice should keep these indices. Under Oliver, find the moment by its words.

| Probe | Frame | Moment | preview02 shows | Must show |
|---|---|---|---|---|
| | 06 f443 (168.0) | "Squashing fails too" | `up: −1` beside `x + 2iy` | The column holds only `no complex derivative` |
| A06 | 06 f450 (171.0) | After "Double every height" | Two readouts and two Xs | `right: 1   up: 2`, then X, then `no complex derivative`; nothing lower |
| A07 | 07 f10 (3.5) | Any frame of 07_1 | Arrow through the `e` of `Re` | A clear gap of about 10 px; the name centred over the arrow |
| A08N1 | 08 f92 (39.5) | "dividing by z undoes the turn" | Nothing marked on the ring | Green dot on the ring between two white dots, `z_k` beside it |
| | 08 f105 (46.0) | "The arrows stack" | No dot | The dot and name gone before the chain lays |
| A08N2 | 08 f186 (81.0) | From "Around each one" until the marks fade | Triangles that read as pointing down | Heads that point on round the circle, anticlockwise; check an 8× crop |
| A09 | 09 f240 (85.0) | From "Each pole pulls" to the end of 09_5 | `a = 0` half hidden | `a = 0` whole, above the lower `−0.5` card |
| | 09 f120 (45.8) | "Eight sample points" | Card lower down | The card clear of the small ring's dots |

For the five named probes, a pixel check is ready:

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex06-09-repair03/checks/anchor_probe.py <frames-dir> A06=450 A07=10 A08N1=92 A08N2=186 A09=240
```

A pass means the right pixels are in the right rectangles on those frames. It is not picture acceptance.

**Expected from the lint:** the line `09_5 … painted over "a = 0"` should go, and the other four lines of `09_boundary.layout.tsv` should stay. The other three chapters' layout files should stay empty. The lint can't see 07-N1 or 08-N2.

## Regression checks

**Six new tests go into the chapters.** They run under `cargo test`, and none needs a render.

| Test | File | Fails if |
|---|---|---|
| `the_reflections_numbers_leave_before_doublings_heading` | `one_derivative.rs:962` | The reflection's numbers stay beside the new heading, fade twice at the close, or the verdict leaves too |
| `the_right_column_reads_rates_then_cross_then_verdict` | `one_derivative.rs:984` | The column's order or spacing breaks |
| `the_map_arrow_starts_clear_of_re_and_stops_short_of_the_right_plane` | `one_point.rs:902` | The tail reaches `Re`, or the tip nears the right plane |
| `a_circulation_head_has_one_point_and_sits_on_its_arc` | `loop_sums.rs:789` | The head is as wide as long, its base leaves the arc, or its size changes |
| `the_position_is_on_the_ring_while_the_voice_undoes_its_turn` | `loop_sums.rs:810` | The dot misses the words, stays on the growing ring, or stops showing the true angles |
| `the_centre_card_is_clear_of_what_joins_it_on_the_picture` | `boundary.rs:903` | The card meets a share card or either ring |

Two of them read the source text, the pattern this crate already uses where no render can run. Card and text sizes in the tests were measured on preview02's frames at 1280 × 720, so they would need updating if a font size changes.

**What ran, and the result:**

| Check | Result | What it shows |
|---|---|---|
| `checks/standalone_tests.py` | PASS: the 6 new tests and the existing circulation test pass, and 16 of 16 planted defects are caught | The test logic works. It uses real `rustc`, the real `model.rs`, and shims lifted from `draw.rs` and `plot.rs`. **It is not the crate's test run** |
| `checks/source_checks.py` | PASS: 43 checks | Baselines, patch round trips, timed calls, strings, windows and the glyph patch. Its controls detect a changed fade time and an added wait |
| `checks/anchor_probe.py` | PASS: all 5 probes fail on preview02 and pass on the composites | The probe can tell the two states apart |
| `checks/desk_composites.py` | I looked at all five images | Where things land, using real preview02 pixels. A08N2 is drawn with PIL. **These are not renders** |
| `rustfmt --check` on the four files | Clean | Formatting and syntax only |
| Frame scan of the new card and dot rectangles | Nothing else enters either one: `a = 0` on every frame f106 to f352, `z_k` on even frames f86 to f104 | Sampled frames only; run in scratch, not kept |

The planted defects found a real gap: my first head test passed a head shrunk along its axis. I added a length check, and the mutant is now caught.

## Not run

- **Crate:** no `cargo build`, `cargo test` or Clippy. **The edits inside `construct` have never been type-checked.** They copy calls already in those files, but that is no substitute for compiling.
- **Render:** no franken_manim render, `--lint` or frame extraction, so no real picture of any fix exists.
- **Sound:** no TTS, no listening and no ASR. Nothing here is Oliver fine sync. ASR would not stand in for listening in any case.
- **Final review:** no continuous AV review. Sampled frames and composites say nothing about motion between samples or how anything looks at 4K.

## Limits and authorship

- **Not independent.** I wrote the review these fixes answer, then repaired it, and I judged the 08-N2 change myself. BronzeJaguar's lane wrote these chapters and LavenderPike's repair added some of the labels moved here. This needs a reader other than me.
- **Left open.** S2 (LavenderPike), the 40 Low findings, and 09-H. Earlier author-overlap notes are in `../complex06-09-picture-review01/REVIEW.md`.
- **An option I did not build.** A thin radius from 0 to the `z_k` dot would show the position's angle directly, instead of leaving the viewer to read it from the axes. It would be one line, but it goes beyond the finding. Root's call once the dot has been seen.
- **Prior work is untouched.** Every earlier folder, canonical source, root's frames and Beads are as they were.
