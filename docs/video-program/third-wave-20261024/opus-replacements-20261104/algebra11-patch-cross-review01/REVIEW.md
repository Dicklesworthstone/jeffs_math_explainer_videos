# Algebra chapter 11 patch: independent cross-review

WhiteSwan, for Mail 1909 on bead `mx-video-program-program-87do.64`. Reviews LavenderPike's frozen bundle `algebra-picture-review-11_permutations/`.

## Authorship

I wrote none of chapter 11, its bundle or its earlier repairs, and this is my first work on `permutations.rs`. My other work in this program is the apportionment film and reviews and repairs on logarithm, complex and exponential.

Earlier chapter 11 work belongs to root (`root-permutation*`), RubyRaven (`logarithm/permutation-cross-review01.md`) and LavenderPike (`algebra/permutation-*` and this bundle, which its README calls an author's review of the author's own code). My name appears in none of those folders. An Agent Mail search finds no mail from me mentioning `permutations` or `algebra`, while the same sender filter does find my exponential mails.

This review is not fully blind to that earlier work. The shared memory index loaded in my session summarizes it in a line each, including a few of RubyRaven's findings. I read those lines but not the memory files, and I opened only the first lines of RubyRaven's review, to see who wrote it.

The follow-up patch in this folder is my own work, so it has had no independent review.

## Verdict

**Apply the bundle's patch.** It is exact, its mathematics is true, every mobject it adds has a removal path, and it changes no spoken word. Every picture block fits its sentence at the scratch rate, apart from AD's deliberate push and X's small overrun (see "X in full"). All five things Mail 1909 named hold up: 11_4 slot 4, the 11_5 transposition, the cycle arrow directions, the 11_22 ring and the 11_24 payoff delay. The 11_22 caption needs the two small repairs in F2 and F3.

**Then apply the follow-up, `patches/permutations-followup.patch`.** One of its repairs corrects a defect the patch introduces (F1, the stale colour key). One restores sync the patch loses (F2, the ring). The rest are small word, phrase and crossfade repairs. It is 15 hunks, 22 lines added and 15 removed, and it changes no spoken word.

**Whichever you apply, the direction map needs its six chapter 11 hold rows rebound.** See "Hold rebind" below.

This is a source, arithmetic and baseline-frame review. Neither patch has been compiled or rendered, so I have seen no pixels of either result. This is not listening acceptance or full audiovisual acceptance.

## Bindings

Paths are relative to `opus-replacements-20261104/`, except the four canonical files, which are relative to the repo root.

| File | sha256 |
|---|---|
| `algebra-picture-review-11_permutations/README.md` | `ee414efcc1077b70d7e6db51a5b307180ab4602ba6aa13f7f457ce046fa50bdc` |
| `algebra-picture-review-11_permutations/REVIEW.md` | `3ac0eedf91c4b7033fe555b38241fa8f750bbc0ac44f7f574755560b2150a6e4` |
| `algebra-picture-review-11_permutations/PROGRESS.md` | `34826dbe9063666f71fbe389236cd1939359dc29b70e8f47b9cc7f831f6e70c9` |
| `algebra-picture-review-11_permutations/HASHES.sha256` | `a0e3c53ba6ad0fa1d93d743a90dfc4ba5668e5698b557f39334c889957fc2e68` |
| `algebra-picture-review-11_permutations/patches/permutations.patch` | `380bb1b17e2c6c897c7e0ffbe581e2f4fc20733889186b29c89d47d5565da54b` |
| `algebra-picture-review-11_permutations/baseline/permutations.rs` | `de3c7b10d96e856ab218d07463a33aa57f97acd41188b1dc267556cfd28e3c68` |
| `algebra-picture-review-11_permutations/candidate/permutations.rs` | `521089439e9d0f173b3ccdb5630db9422d1a12a2ef1a24ce49b76380c922684a` |
| `explainer_corpus/abstract_algebra_expanded/src/permutations.rs` | `de3c7b10d96e856ab218d07463a33aa57f97acd41188b1dc267556cfd28e3c68` |
| `explainer_corpus/abstract_algebra_expanded/script.tsv` | `83619d48676b32d5d93bf7527477aa752a902dd16927c548e4b3052cc2d689c0` |
| `explainer_corpus/abstract_algebra_expanded/docs/narration-direction-map-oliver24.json` | `7f05be9377a54b2c4e1a03ca83b563eb87d90734818273d59d95cea566784d4a` |
| `explainer_corpus/abstract_algebra_expanded/narration_units.json` | `4d7b7c0f3437e9de6501ba1527cc37924f48d3ad0c4cad34a9c4d789424b470d` |
| `algebra11-patch-cross-review01/patches/permutations-followup.patch` | `43541f26216ce15da21553bc92b50aa40391f2319988400234a4ba61cc183dd7` |

The follow-up's base is the frozen candidate `52108943…`. Its result is `4f861c3b6704dfd89774909f88168bc7252a66cc54a4c5d62f1f547060c28473`.

All 11 entries in the bundle's `HASHES.sha256` verify. Canonical `permutations.rs` still equals the bundle's baseline. Both are unchanged by this review.

## Findings

Line numbers are in the frozen candidate.

### F1. The colour key goes stale at 11_21 sentence 5. Medium/Low. Introduced by V.

`Gold: even` and `Blue: odd` appear at sentence 3 (:3188-3192) and stay on screen until the wipe (:3241). Sentence 5 (:3212) repaints one even cell green and three hero yellow, so only 8 of the 12 even cells are still gold while the key says gold means even.

Frame 1090 shows those sentence 5 colours in the baseline. `Palette::gold()` is `#F0AC5F`, which reads as orange, and `Palette::hero()` is `#FFD866`, which reads as gold. So after sentence 5 the cells a viewer would call gold are the three double swaps. The bundle's own REVIEW.md records having mixed up these two colours.

Repair: the keys fade at the start of sentence 5, and they read `Even` and `Odd`, each in its colour, with no colour name. The block becomes 1.40 s in a 5.54 s window.

### F2. The caption delays the ring against its words. Low/Medium. Introduced by AA.

`reading` fades in by itself (:3476) before the ring's own fade, so all six hops of the six-cycle run 0.4 s later. By the kit's rule, the ring lands this long after each hop's words end (scratch rate):

| Hop | Baseline | Candidate |
|---|---|---|
| 2 to 3 | +0.24 | +0.64 |
| 3 to 5 | +0.29 | +0.69 |
| 5 to 9 | +0.42 | +0.82 |
| 9 to 8 | +0.47 | +0.87 |
| 8 to 6 | +0.60 | +1.00 |
| 6 to 2 | +0.22 | +0.62 |

At hop 5 the ring lands a full second after the words, while the voice is already on the next hop. Scratch timing is only a model, but this scene's whole job is to walk the trail "in the words' order" (the source comment at :3464).

Repair: `walk_cycle` takes a `beside` list that fades in with the ring, so the caption and ring arrive together. The block is 7.70 s, as in the baseline, and the hops land where they did in the baseline.

### F3. `slot` in a cue that says `position`. Low. Introduced by AA.

The 11_22 voice says "position" and "Follow positions", and the caption just before it, at the same spot, reads `Positions 1 and 10 stay`. Then comes `Card 2 went from slot 2 to slot 3`. Repair: `Card 2 went from position 2 to position 3`, 4.92 wide.

### F4. The 11_20 trades finish before their words start. Low. Introduced by T.

The trades run 0.80-3.50 s into sentence 0. "since they compose" starts at 3.64 s and "and each one can be undone" at 5.05 s, so the picture is over before it is named. Repair: a new anchor `THEY_COMPOSE`, with one `until_phrase` before the trades. They then run 3.64-4.84 and 5.14-6.34 in a 7.06 s window, leaving 0.72 s spare, or 0.38 s at the fast voice rate below. `ANCHORS` grows to 17, so the existing anchor test covers it.

### F5. `lines` where the voice says `wires`. Low. Not a regression, but AH rewrites the line.

The voice says "With eight wires" and "their starting wires". AH's `all data back on their starting lines` fixes the old logic error but keeps a word the voice never uses. Repair: `starting wires`, and the title `The same shuffle, on eight wires`, so the two captions agree. Both fit (6.05 and 4.51 wide). If root prefers to leave the title alone, drop that hunk only.

### F6. Four more same-spot word crossfades. Low. Pre-existing, the same class as AE, G and J.

The bundle left the `exchange` sites open because half-second samples did not show garbling. The source settles four of them, because old and new words sit at the same coordinates:

| Line | Cue | Old words | New words |
|---|---|---|---|
| :1804 | 11_8 s4 | `N = number of swaps` | the sign formula |
| :2538 | 11_12 s3 | `On the clock: first return = order` | `Here: every card in its own slot…` |
| :2638 | 11_14 s5 | `Card 4 never moves: no condition` | `3 + 2 = 5: neither loop is home` |
| :4040 | 11_27 s5 | `One extra swap: odd, so not home` | `Next: when do two groups…` |

Repair: `swap_caption` at all four, with at least 3.7 s spare in each even at the fast voice rate. I would leave the other four on the bundle's list as they are: :1716 (a lamp and value updating), :3009 (table cells becoming signs) and :3514 (cards repainted in place) replace an item with its own updated form, and at :3537 arrows give way to strips elsewhere on the screen.

### F7. 11_24 to 11_25 is the one long cross-cue gap with no hold row. Note for root. AD lengthens it.

In the baseline's cue table, seven chapter 11 gaps exceed 1 s. Six have hold rows. The seventh, 11_24 to 11_25, is picture-driven and has neither a `k.hold` nor a row: 3.43 s of silence in the baseline, 4.63 s with AD, against a cue-transition policy of 0.45-1.4 s. By my reading, `tools/narration_director.py:382-387` would class it as `scene_wait_review` rather than refuse it, but I did not run that tool.

The silence is deliberate, because the staircase is moving through most of it and sentence 3 asks the viewer to watch. Root should decide whether to accept the review flag or declare the hold in the next map revision. A row needs a marker, which would mean a no-op `k.hold(stage)?;` after `full_deck` in `construct`. I have not patched this.

## The five named checks

**11_4 slot 4: correct.** Frames 0186 and 0193 show the defect: the link from slot 4 to slot 1 runs up through the number 4. The candidate puts the lower row's numbers under their rings at y -1.72, the same 0.62 offset as the upper row. In frame 0193 the written inverse spans x -1.49 to +1.48, so it clears both moved numbers at x -3.2 and +4.7. Only `undo_and_disjoint` draws slot numbers on this board (11_1 and 11_2 draw cards only), so numbers do not flip sides between scenes.

**11_5 transposition: correct.** It shows cards 1 and 3 trading, which is the (1 3) acted first in the spoken example. They pass on opposite sides, because the same negative arc bends a rightward move up and a leftward move down. The word prints by 0.8 s, and the swap ends at 1.7 s, inside sentence 0's 2.6 s. All three mobjects leave at sentence 2 and none is handed on. Before the printed formula appears, nothing is on screen but this beat (frame 0600 shows what follows). Both factorisations check.

**Cycle arrow directions: correct everywhere.** `checks/arrow_replay.py` rebuilds each arrow from the pin's `Arrow` and `PathFunc` formulas, and I traced `link_offsets` by hand as well:

- Every link leaves from over its own slot, and its tip stops 0.66 above the next slot, within 0.232 of that slot's centre line. The slot rings are at least 0.31 in radius, so each arrow reads as pointing at that slot. Arrows stay at least 0.131 from the slot numbers (baseline 0.136).
- Every arc bulges to the intended side, and each tip points the way the link travels.
- At each slot the head and tail stand on opposite sides, nested so that the short arc stays under the long one.
- Shaft crossings match the baseline: 0, 0 and 1. The one in 11_22 is 5 to 9 against 6 to 2, which cannot be avoided because those chords interleave.
- Swaps stay a single double-headed arc (largest stray 0.006).
- `Arrow` and the ring's motion use the same sign convention, and frame 1276 shows both bulging up on a negative arc. Every hop's ring travels on its arrow's side.

The head-to-next-link gap rises from about 0.001 in the baseline to a minimum of 0.057. **This is the one thing I would look at first:** slot 5 in 11_4 and 11_12 at 0.057 (15 px at 4K, 5 px at 720p), and slot 4 in 11_15 at 0.070. There the closing arc passes just over the incoming head. It should read as two separate marks, but only a picture can settle that. If it looks crowded, `LINK_SEP` 0.20 gives 0.096 and 0.109, moves the arc tops by only 0.01, and the new test scales with the constant. I do not propose changing it before seeing a picture.

**11_22 ring: geometry correct, two repairs (F2, F3).** The ring rests on slot centres and follows its arrow's side. The disagreement AA reports is real and is inherent to a deck that has already shuffled once. At rest the ring shows card 6 at slot 2, 2 at 3, 3 at 5, 5 at 9, 9 at 8 and 8 at 6. Frame 1276 reads 1 6 2 7 3 8 4 9 5 10, as the replay predicts. The caption's rule holds on all six links: an arrow from position N lands on card N. So the caption is true and is the right repair. It clears the deck, the blue arc is not yet drawn, and it leaves before the blue walk.

**11_24 payoff delay: accept the 1.2 s.** Frames 1411, 1412 and 1415 show the defect: the count reaches 8 only as the in-shuffle caption fades in over it, and the bars are already moving within 0.5 s of the cue. The replay predicts the baseline's 11_25 at 631.399 s against the render's 631.433 s.

`scene.notes[3]` at :3757 is `predicted` (:3735, the fourth entry of the list at :3751), which prints lcm(1,2,8) = 8, and `scene.count.mob` is the fresh `Shuffles: 8`, because `advance` replaces it. The last bar lands 6.50 s into sentence 3, the count is fully up at 6.80 s, the two pulse until 7.40 s and hold until 8.00 s, and the first in-shuffle starts at 8.80 s. So the restored staircase stands still for 2.3 s, against about 0.6 s in the baseline. The pulse's flash lasts 0.6 s, then 0.6 s is clean. See F7 for the gap.

## Decisions for root

The README's display strings and timing notes come first. From Z down are the items `REVIEW.md` leaves to root or only recommends.

| Item | Bundle's position | My view |
|---|---|---|
| I | `Transposition: swap two labels` | Accept. 4.32 wide. |
| V | `Gold: even`, `Blue: odd` | Keep the idea, including ink until sentence 3. Change to `Even`, `Odd`, and fade at sentence 5 (F1). |
| AA | `Card 2 went from slot 2 to slot 3` | Accept the fact. Use `position`, and fade it with the ring (F2, F3). |
| AC | `By size: 2 fixed, 1 swap, 6 cycles of length 8` | Accept. True: the strip sorts by length, while real positions interleave (the swap is 18 and 35, and the first long cycle starts 2, 3, 5, 9). The visible offset AC leaves unpatched should be judged on a picture. |
| AH | `After 3 stages: all data back on their starting lines` | Accept the correction. Use `wires` (F5). |
| AG | `Stage 4` | Accept. x 3.9 is that column's centre by the same rule as stages 1-3; compare frame 1510. |
| AD | Deliberate 1.2 s push | Accept (F7 is the only follow-on). |
| X | 0.35 s rest, `sr` may run up to 0.35 s late | Accept, but it reaches further than the README says. See below. |
| Z | 11_10 small row, left to root | Leave out of this pass. It is a new design beat, and it should be judged on a picture because it sits under a prediction question. |
| AM | Rest only; counter left to root | Rest only. `advance` costs 0.3 s a run, so 2.1 s for seven, against 0.28 s spare. |
| H | Pass a palette colour to `pulse` | The defect is real (frame 0419), but this remedy will not restore the number. `indicate` sends the whole family to one flat colour, disc and digit alike. Filled tokens need a scale-only pulse or a `halo`. A shared helper, so a separate pass. |
| N | `advance` timing, recommended | Leave: it moves many timings for a one-sample lag. |
| C, D, R, U, W, AK, AL | Recommended | No objection, and none blocks. Judged from the bundle's table and the source only; I did not open their frames. |
| Unpatched `exchange` sites | Look on the next preview | Four are settled by source and patched (F6). The rest stay. |

**X in full.** The left picture takes 2.35 s against 1.97 s between its anchor and the next, so the `sr` picture starts 0.38 s late (0.03 in the baseline). The right picture then needs 2.35 s of the 2.26 s left, so the callback at sentence 4 lands late too:

| Voice rate | Sentence 4 late, candidate | Baseline |
|---|---|---|
| scratch | 0.47 s | 0.00 s |
| x0.90 | 0.89 s | 0.19 s |
| x1.05 | 0.28 s | 0.00 s |
| x1.20 | 0.00 s | 0.00 s |

Sentence 4 is about 4 s long, and what lands late is two still tiles and `rs ≠ sr` under "the same refusal to commute", so I accept this, but root should know it.

## Corrections to the bundle's statements

None of these changes a picture.

- **Patch size.** README says 153 lines added. It is 158 (`git diff --numstat`, my hunk walker and a `grep` agree). Hunks (32) and removals (32) are right.
- **Windows.** README's table takes each sentence's share of words of the whole cue length. The kit takes its share of characters of the speech (`estimated_starts`). I reproduced README's figures with its method and the kit's with the kit's. All blocks still fit, but the margins move:

| Item | Block | README window, spare | Kit window, spare | Spare, fast voice x0.90 |
|---|---|---|---|---|
| AA 11_22 s2 | 8.10 | 9.76, 1.66 | 8.98, 0.88 | -0.01 |
| AA 11_22 s3 | 3.50 | 5.08, 1.58 | 4.38, 0.88 | 0.45 |
| AM 11_27 s1 | 2.69 | 2.72, 0.03 | 2.97, 0.28 | -0.02 |

An overrun of a few hundredths only makes the next `until_sentence` a no-op, so neither is a defect. F2 lifts AA s2's fast-voice spare to 0.39. Every block figure in README's table matched my term-by-term count. All 13 rows are in `checks/timing_replay.out.txt`.

- **Text widths.** The 0.50 em estimate runs high. By the font's tables, `After 3 stages…` is 5.98 wide, not the desk check's 7.16. Nothing is near the frame edge, the two keys clear both cell blocks, and every new glyph is in the font.

## Hold rebind

The patch moves all six chapter 11 hold markers and changes the file hash that the direction map binds, so those rows are stale once either patch lands. In all six rows the say-to-next-say text is unchanged and the same marker stays under the same cues, so only `sha256` and `line` change.

| Hold | Kind | Bound line (`de3c7b10…`) | After bundle patch (`52108943…`) | After follow-up (`4f861c3b…`) |
|---|---|---|---|---|
| 11_2 to 11_3 | reading | 4020 | 4108 | 4115 |
| 11_3 to 11_4 | reading | 4070 | 4158 | 4165 |
| 11_12 to 11_13 | prediction | 4082 | 4170 | 4177 |
| 11_15 to 11_16 | prediction | 4094 | 4182 | 4189 |
| 11_6 to 11_7 | prediction | 4160 | 4269 | 4276 |
| 11_10 to 11_22 | prediction | 4217 | 4326 | 4333 |

`narration_units.json` (which the bundle did not examine; sha `4d7b7c0f…`) has 27 chapter 11 units, one cue each, and every text equals the script. Neither patch changes it. Its `direction_contract` binds the map by hash, so rebinding the map means updating that binding too.

No quiz is spoiled. Neither patch touches a hold's interval, and V's change removes an early colour answer under "How many even permutations?".

## Patch, mathematics and lifecycle

**Patch.** It applies forward and reverse at fuzz 0, and an independent hunk walker reaches the same bytes. The `k.say` order is identical and every `until_sentence` index is valid. The only change inside a tests module is one added test. The display-string census matches README's list exactly, and the two removed strings appear nowhere else in the crate. `rustfmt --check` passes on both results.

**Mathematics** (`checks/math_replay.py`, from definitions, not the film's helpers). All of these hold:

- (1 3 4) inverse is (1 4 3), and frame 0186's cards match the forward run.
- (1 3 5) and (2 6) commute, and (2 6)(3 5 1) is the same map with 4 fixed.
- (1 3 4) = (1 4)(1 3) = (3 4)(1 4), and a length-m cycle is built from m-1 swaps, checked to m = 8.
- 11_20's trades compose to a three-cycle and undo to home.
- There are 12 even and 12 odd arrangements, the extra swap pairs them, and the even ones split 1 + 3 + 8.
- Ten cards: fixed points 1 and 10, cycles (2 3 5 9 8 6) and (4 7), lcm 6.
- Fifty-two out: 2 fixed, swap (18 35), six 8-cycles. In: one 52-cycle, and 52/8 = 6.5.
- Eight wires: (2 3 5), (4 7 6), order 3, and stage 4 leaves every token where stage 1 did.

**Lifecycle** (`checks/lifecycle_scan.py` and reading). `swap_name` and `swap_cards` fade at 11_5 sentence 2. `reading` fades at 11_22 sentence 3. `even_key`, `odd_key` and `fourth` join the scene's returned list, and `construct` fades those lists at :4315, :4319 and :4346; the follow-up removes the keys earlier. T's `row` is the same row later sentences use, it ends at home so `Swaps: 0` stays true, and its `run_word` calls and final list are unchanged. `recolor` ran on these same text cells in the baseline's sentence 5, so V's new use is a known-good path. AH's and AG's new words use existing helpers.

**Reading windows.** At 15 characters a second, every new caption has at least twice the time it needs at the scratch rate. The tightest is `After 3 stages…`, needing 3.53 s with 8.17 s on screen, then `Transposition: swap two labels`, needing 2.00 s with 4.89 s. In the follow-up, the longer AA caption needs 2.73 s of at least 8.58 s, and `Even` and `Odd` together need under 0.5 s.

## What to look at on the first preview

By cue and sentence, because the patch moves times:

1. 11_12 and 11_4, slot 5, and 11_15, slot 4: closing arc over the incoming head.
2. 11_4 sentence 0: lower numbers under their rings, clear of the link and the written inverse.
3. 11_22 sentence 2: ring against "two goes to three… six back to two", in Oliver's timing.
4. 11_21 sentences 3-5: key arrives, leaves, cells take three colours.
5. 11_20 sentence 0: the trades on "since they compose".
6. 11_24 end to 11_25: the 8 stands, then captions change cleanly.
7. 11_5 sentences 0-2: the card swap, then the formula.
8. 11_26 sentence 3 onward: `Stage 4` over the red column, and the two captions.

## Apply

From the repo root, only if canonical is still `de3c7b10…`, otherwise use `git merge-file`:

```
patch -p1 --fuzz=0 < docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra-picture-review-11_permutations/patches/permutations.patch
patch -p1 --fuzz=0 < docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra11-patch-cross-review01/patches/permutations-followup.patch
```

`checks/followup_check.py` runs that chain from canonical in a temporary directory, and both patches in reverse, and reaches the stated hashes.

## Tested and not tested

**Run, with saved output in `checks/`:** patch fidelity at fuzz 0, the hunk walker, `rustfmt --check` on both results (edition 2024, repo config), arrow geometry from the pin's formulas, arithmetic from definitions, the kit's scratch timing against the baseline's cue table (all 27 cue lengths match), the hold map, font-table widths and glyphs, and the lifecycle text scan.

**Looked at:** baseline scratch-voice frames at half-second sampling: 0186, 0193, 0230, 0399, 0412, 0419, 0600, 0955, 1040, 1066, 1090, 1161, 1276, 1289, 1411, 1412, 1415, 1484, 1510. They show the defects and the layouts the patch works in, not its results.

**Not run or not done:**

- No Cargo, `rustc`, test suite, Clippy, oracle, layout lint or render, for either patch. LavenderPike reports extracting and running `link_offsets` and its test with `rustc --test`; I did not repeat that, and I traced the same logic by hand and in Python. The follow-up's type-checking rests on reading only. `swap_caption` returns `fmn::Result<()>`, so it can stand as the tail at :4040, and `fade`'s `for &item` shows `Mob` is `Copy`, which `appearing.extend(beside)` needs. None of that has been through a compiler.
- `tools/narration_director.py` was read, not run. I did not find which validator reads the algebra map's `line` and `marker` fields, though root's stage `source-plan-validation01.json` records `hold_line` and `interval_sha256`.
- No Oliver timing for this chapter. The local inventory has 0 chapter 11 takes and lists all 27 as missing. The x0.90 and x1.20 rates are the p10 and p90 of duration against scratch length over 124 retained takes from other chapters, some of whose cue text may be stale. They are a sensitivity range, not a measurement.
- No listening, fine sync or full audiovisual acceptance. Current narration is ElevenLabs Oliver. I synthesized nothing and listened to nothing.
- Canonical, the input bundle, the Beads and the shared files were not edited.
