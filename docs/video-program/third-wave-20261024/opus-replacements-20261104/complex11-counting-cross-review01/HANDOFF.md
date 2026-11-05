# Complex 11: cross-review of counting-teaching02

BronzeJaguar, for BlueSalmon, answering Mail 2897. Bead 87do.39. Narration is ElevenLabs Oliver.

**Verdict: integrate teaching02, and put C1 in with the plate core.** The mathematics holds and I found no defect in the normalization demonstration itself. There is one integration blocker (F1), one teaching defect with a staged fix (F2), one reading defect with a staged fix for you to accept or refuse (F3), and one validation gap (F4).

**State: nothing here or in teaching02 has been compiled, tested by Cargo, rendered or heard.** No picture of this code exists. The evidence is source reading, Python mirrors, the frozen binder, the desk tools, `rustfmt --check` and old preview02 frames. Whether the turn works on screen and the voice lands on it is still unknown.

**Output path.** The mail's `P` is a file path, so I read `P/complex11-counting-cross-review01/**` as the sibling folder `complex11-counting-cross-review01/`. teaching02 stays sealed, and its seal still holds on all 33 files.

## Overlap

This is a fresh read of WhiteSwan's delta. It is not independent of the base.

- **I wrote the base.** I authored the native chapter 11 and most of the crate helpers White builds on, the desk tools in `complex/checks`, the binder `complex/reading-repair02/bind_oliver_plan.py`, and the glyph patch that touches `LEASH_TEX`.
- **F2's digit place is my own original code**, so that finding is against me as much as anyone.
- **Shared memory.** White and I share one auto-memory directory, the same model and the same skills. The index carries White's entries, so a shared blind spot is possible.
- **I saw White's framing first.** I read White's Mail 2864 while sealing closing15, and read this handoff before the code. White also answered my Mail 2818 about the closing caption.

## Verdict by focus

| Focus | Result | Evidence and limit |
|---|---|---|
| The 2i demonstration | **Holds.** The printed row expands correctly. One tracker drives field and curve. t = 0 matches the painter cell for cell, t = 1 matches the picture of f within 1.39e-15, and every cell changes between them. | `math_mirror.py`, 24 of 24, 6 plants caught. A mirror of the source, not frames |
| Zeros and winding | **Hold at all 49 steps:** the same 5 roots at the marks, 5 zeros in the window, windings 1 and 5, all colours meeting at each mark once. The curve stays off 0 and inside the window: radius 0.45–0.9534 at t = 0, 0.5592–1.0432 at step 24, 0.675–1.1176 at t = 1, window half 1.5. | Same |
| Preserved teaching | **Holds.** All 17 of root's tests remain, 13 are added. The patch takes 28 lines out of root's file: imports, the card text you asked to change, a literal now named `ROOT_WALK_SQUASH`, the 11_6 line that faded the loop and marks, and the labels it repositions. | `preserved.txt`, `strings.txt` |
| Cue sentences and holds | **Hold.** Sentence indices 1, 2, 3, 4 and 6 are right, and 11_7 is 96 words against the limit of 100. Requests carry no neighbour context, so "only 11_7 needs a take" holds. The frozen binder derives every hold row of White's map (440, 503, 604) and reports the same 8 problems as canonical, adding none. | `binder_compare.txt`, `reading_mirror.txt`. Both `mod.rs` tests are mirrored, not run |
| Painter order | **Holds as staged.** Loop and marks go back over the field, the painter leaves under it, `turning.stop` comes before the clear, and everything clears for the wide plane. | `order_walk.py`: 0 faults, 16 plants caught, using the engine's `add_to_scene` rule read from source |

**White's two questions for a fresh reader:**

- **`nothing else lands on 0`: correct.** `w ↦ w ÷ 2i` is a bijection of the plane fixing 0, so the words give both inclusions: no root is lost and none is gained.
- **Does the turn teach the point?** On paper, yes: f is shown as the same picture turned, the marks stay, and the turn back *is* the division the voice is naming. Whether a viewer takes that from the screen needs a preview.

## Findings

### F1. Blocker once the plate core goes in: `counting.rs:656`

`picture_cells(&inp)` takes one argument today. `complex-color-repair03/patches/A-plate-core.patch` makes it `picture_cells(plane, style: &PlateStyle)`, so this line fails with E0061 whichever rung you select.

Neither lane could see it. Canonical has no caller outside `draw.rs`, so the plate lane's checks had nothing to catch, and teaching02 was written against the old signature. The desk arity tool flags exactly this line on staged + plate core, not on staged alone, and not with C1.

**C1:** `let (cols, rows) = picture_cells(&inp, &super::draw::PLATE);`. It moves no line. Apply it only together with A.

**It must be `PLATE`, not `PLATE_NOW`.** `Turning` computes its colours for this size and ignores the result of `strips.recolor`, which returns false on a size mismatch. With the wrong size it would compile and leave the field still while the curve turns.

A rung with `samples > 1` would also break the unseen takeover, because `Turning` paints single-sample colours where the painter averaged. Every rung now defined has `samples: 1`, so this is only a note if that changes.

### F2. Teaching: the digit `0` sits inside the disk that "never holds zero"

In 11_8 the picture and voice say the reach never contains zero. The digit naming zero is at (−0.25, −0.3), inside the sweep. It is inside the disk for **25.9% of the turn: 0.42 s at the 1.6 s floor, 0.62 s at 2.4 s.**

Old preview02 frames show this (old source, scratch voice): the rim runs through the digit at 546, and the digit is inside the teal circle at 547 (188.8 s) and 548 (189.0 s) while the actual axis crossing stays outside. See the top row of `checks/outputs/c3-digit-on-zero-NOT-A-RENDER.png`.

White's plate keeps the digit readable, and that repair is right for the defect White measured. It does not touch this one, and a legible `0` inside the disk may read more clearly as "zero is inside". Its cost is also larger than "a short break": the plate breaks the line for 12.5% of the turn and the rim for 38.8%.

**C3 puts the digit and plate on zero**, the only nearby place the disk never reaches. Past the sweep's outer edge at radius 1.785 the digit would be too far from zero to name it.

- The rim never touches the plate: clearance 0.0396 plane units, 0.0277 scene units, 7.5 px at 4K.
- The line's end simply goes under the plate, so there are no breaks.
- **Moving the digit alone would be wrong.** The line would fade in over the digit for 0.2 s and then be cut by the plate. C3 therefore roots the line under the plate before its fade starts, with no film time added. The engine roots a mob at `begin` only if it is absent (`fmn-anim/src/frame.rs:195`), and `fade_in` starts from opacity 0 at `begin` (`fmn-anim/src/fading.rs:52`). I caught this reading the scene, not from a tool. The walker now checks it. Unseen on a frame.
- **Cost for you to judge:** the plate hides the axis crossing through the whole of 11_8's frame. The bottom row of the sheet is the idea, hand-edited on old frames. It is not a render, and the breaks in its white line are scars of the edit.

If you take C3, the `DIGIT` box at `complex11-counting-teaching02/checks/anchor_probe.py:38` must recentre on (410.5, 310.5) at 720p. That file is sealed, so edit a copy.

### F3. Reading: the result row is short of its time

`p(z) = f(z) ÷ 2i = z⁵ − 3z + 1` prints 30 characters, so 2.00 s at the program's 15 a second as a lower bound. It arrives together with the 39-character `ZERO_KEPT` and the 15-character card, and mathematics reads slower than prose. It is cleared on sentence 2's first word.

**C2 (your call):** `line.wait_for(stage, "with z")?;` replaces `line.wait_sentence(stage, 2)?;`. This is a third phrase key where your Mail 2648 asked for sentence keys. It also puts `zⁿ` on the words that name it, where the staged order has it fully on screen before them.

| Rate, characters a second | Row clean, staged | With C2 | `zⁿ` done against "z to the n", staged | With C2 |
|---|---|---|---|---|
| Slowest Oliver cue, 10.62 | 2.70 s | 4.02 s | −1.09 s | +0.23 s |
| Scratch, 12.7 | 2.18 s | 3.28 s | −0.80 s | +0.31 s |
| Oliver overall, 13.96 | **1.94 s** | 2.94 s | −0.66 s | +0.34 s |
| Fastest Oliver cue, 16.48 | **1.56 s** | 2.41 s | −0.45 s | +0.40 s |

These are estimates. The Oliver rates come from Parrondo narration02 (169 cues, median 13.91), another film, and sentence shares are by character. The model reproduces White's turn table as its control. An aligned 11_7 take replaces all of this.

### F4. Validation gap: the turn is missing from the geometry probes

The turn is the film's heaviest repaint, up to 96 full repaints. White's "25,600 cells" is stale under the plates now selected. Neither Lavender's chapter run (04, 05, 14, Mail 2921) nor RubyRaven's table (Mail 2916) includes 11_7. So your cells2/cells4 comparison (Mail 2938) would not see it.

| Rung on the 5.6 window | Cells a repaint | One plate, knots | Takeover or repaint (×2) | Worst (×3) | Worst of `1<<22` |
|---|---|---|---|---|---|
| now | 25,600 | 103,840 | 19.8% of `1<<20` | 29.7% | 7.4% |
| cells2 | 102,400 | 207,680 | 39.6% | 59.4% | 14.9% |
| cells4 | 409,600 | 824,960 | **157.3%: refused under the default before any repaint** | 236.0% | 59.0% |

This is arithmetic, with controls against the published knot counts, and nothing is observed. The rule that a repaint needs twice the live knots is RubyRaven's.

**Action: add 11_7's turn to whichever probe decides the rung.** If time is the cost, `TURN_STEPS` at 24 halves it, and the test's `< 2°` must loosen with it.

### Lows

- **Phrase keys (White's decision for you): I recommend accepting them.** `complex polynomial` and `which leaves` use existing machinery, 11_8 already uses four, the binder accepts them, and they become word-exact once `phrase-timings.json` exists. The first turn does need to finish before "Divide".
- **Phase line on the real axis:** it falls there at steps 8, 24 and 40 of 48, one step each way, not only at the composite's step 24. This is correct behaviour; check it by eye.
- **Row before picture:** the row names f about 1.7 s before the picture finishes reaching it. Acceptable to me; check it by eye.
- **Under `now` only:** the field over the painter hides row stripes for 0.4 s. Moot if you select cells2.
- **`ROW_HALF` test:** it uses 32 pt while `degree_n` is 36 pt. It still holds by hand (2.56 > 2.50, 3.33 > 3.15).
- **`2{i}`:** I did not read fmd-math's `occurrences`, so "only the leading `2i` is gold" rests on White's reasoning. The glyph probe must show it, along with `≠` and `\div` in these rows.
- **`2i` not in facts:** I agree it is illustrative, pinned by a test. Your call, as White says.

## Corrections

All are in `corrections/`, against staged `counting.rs` `dba35730…`. They are frozen and uncompiled.

| Item | Status | Change | SHA-256 |
|---|---|---|---|
| `C1-after-plate-core.patch` | **Required with plate core A** | +1 −1 | `e86add3502cfa833e2115c68db931d1d627f83774fdc6607647fe9cfbdb9baf1` |
| `C3-digit-on-zero.patch` | Recommended, you judge the look | +24 −26, tests included | `7d69c30600b2366c59a95964d15a77971436c7412999372a8bed3e14f91eec10` |
| `C2-row-swap-on-its-words.patch` | Your call | +5 −4, tests included | `5900758bfff25c4c225a5760f0abf23a5b6ccf13571d583de8f60c2edd6373ca` |
| `C2-storyboard.patch` | Only with C2, on teaching02's storyboard `6deb9f01…` (applied and reversed by `git apply`) | one phrase, gives `cb99b96278938aed…` | `071e6eb19816c8153ce251faa1fb3d7820e4bfeb57b800b74508645808dc5d87` |

`corrections/superseded/C2-result-row-floor.patch` is my first C2. **Do not apply it.** The frozen binder refused its `wait_until`, so I redesigned it and kept the file only because nothing here is deleted.

**Order:**

1. teaching02's section 1 patches.
2. C3 and, if accepted, C2 with its storyboard patch.
3. Plate core A, the `select-*` patch and C1 in one step.
4. The glyph patch, at any point.
5. Rebind the holds map last.

Every order of every subset of C1, C2 and C3 gives the same bytes and reverses, by `git apply`. The glyph patch applies on each state I tried.

| State | `counting.rs` SHA-256 | With glyph patch |
|---|---|---|
| S (staged) | `dba35730d15b3c5e4ff7fb0fe7444265424b920d87861562b11d8468f9d290e4` | `138b5d84d0c9240d…` |
| C1 | `d227e5e089ecbde0dc5bd0638a1f4d600f89b66c18abc07906f1f8a1bb0690ec` | |
| C3 | `cb996633b61066cb6861e638b8b3aa2657856b96044314fccfbd079f04bce2f0` | `09dd1523d92dd05b…` |
| C1+C3 | `d5763e3f090f17b81748fdcfa1753a7af8588e97a03d44e0557322a14cf75be7` | |
| C2+C3 | `cdc154ec074aa446b96cae783fa7e26b0fec3b9cf18e9c16ed19284aab22e32c` | `65466eb8678d8d7e…` |
| C1+C2+C3 | `f3fc8e3535b916e01782f0d627b53aed6c56d9a0261ace1306221bbf4a553bd5` | `c1df16ea04d4283d…` |

**Holds:** no correction moves one. 11_5, 11_6 and 11_8 stay on lines 440, 503 and 604 in every state; only the `sha256` changes. `bindings/holds-by-state.json` lists them. With plate core A, the map's `draw.rs` rows go stale too, so rebind from the final bytes with the binder, then the plan's map hash. C2 + C3 adds nothing to the binder's 8 problems.

**Takes:** no correction changes a word. 11_7 is still the only cue needing a take. C2 changes one storyboard phrase, not the script.

## Checks

Everything is under `checks/`. Regenerate from the repo root with a scratch path that does not exist yet:

```
/usr/bin/python3 -I -B <this folder>/checks/run_checks.py <repo> <fmn-crates-at-a399d62> <fresh-scratch>
```

It starts `c3_synthetic.py` with `python3 -B`, because that one needs PIL, and it reads the old preview02 frames from their current location. It writes only here and in the scratch folder.

| Check | Result | What it cannot show |
|---|---|---|
| `baseline.txt` | teaching02's seal, patches and pinned inputs hold | Anything about behaviour |
| `math_mirror.py` | 24 of 24, 6 plants caught | That the Rust computes the same; no frame |
| `order_walk.py`, on staged and on C1+C2+C3 | 0 faults, 16 plants caught on each | It is a statement walk of the source, not a run |
| `reading_mirror.py` | All pass; reproduces White's table as control | Real word times; `mod.rs` tests are mirrored only |
| `plate_budget.py` | Five controls against published numbers pass | Real cost, time or memory |
| `oliver_rate.py` | 13.96 overall, 10.62–16.48 by cue | Parrondo's rate, not this cue's |
| Frozen binder, `binder_compare.txt` | Added 0 on staged and on C2+C3 | Only what the binder checks |
| Desk tools, `desk_checks.txt` | Arity flags only F1. The other rows new on plate core (`Steps`, `Blend`, `mut m`, `mean`, the dead rungs) I read as tool noise: those items are in use or carry `allow(dead_code)` | Not `cargo check` or Clippy. `supplements` exits 1 on canonical too, unchanged |
| `rustfmt --check` | Clean on staged, staged + plate, C2+C3 and C1+C2+C3 + plate | Not a compile |
| `c3-digit-on-zero-NOT-A-RENDER.png` | The idea only | Old frames, hand-edited, scratch voice |

**Still open, all root's:** Cargo build, tests and Clippy on the corrected state; the glyph probe; a chapter 11 render; the takeover seam, turn steps and mid-turn look at preview; the 11_7 Oliver take, listening and fine sync. White's section 6 list stands, with the digit box change above if you take C3.

I changed nothing in canonical, teaching02, Beads or any sealed folder, and started no agents.
