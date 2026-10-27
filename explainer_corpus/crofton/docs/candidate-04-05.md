# Candidate chapters 04 and 05 (unregistered)

Written by GentleLantern for root's 04/05 assignment (Mail 6876, with the
corrections of Mail 6881, 6884 and 6890, 2026-10-24), Bead
`mx-video-program-program-87do.22.1`. Three new files, none registered,
none compiled:

- `src/chapters/placement.rs`: the helper both chapters need. A
  placement of the sheet (a direction and a slide), the two ways it moves
  (sliding across its lines, turning rigidly about a point), the runs of
  counts along a move, the lines clipped to the card at any direction, and
  the live sheet: lines, crossings, counter, the halo that picks out a
  line, and the two controls, all redrawn every frame from one set of
  trackers. It also holds the words the two chapters share and a copy of
  03's envelope.
- `src/chapters/one_count_one_placement.rs`: chapter 04,
  `OneCountOnePlacement`.
- `src/chapters/a_coin_that_never_changes.rs`: chapter 05,
  `ACoinThatNeverChanges`, with the coin's meeting rule (`meetings`,
  `through`) and the `Survey` that `facts.rs` would print the coin's facts
  from.

The rows below are in `docs/storyboard.md`'s format, ready to move there
when root registers the scenes. `tools/storyboard.py`'s own `parse` and
`check` read this file as it stands. The registry, `main.rs`, `model.rs`,
`facts.rs`, `palette.rs`, `oracle.py`, the scripts, the storyboard, the
full draft and the delivered chapters 01 to 03 are unchanged. Nothing here
has been compiled, rendered, narrated or listened to.

Chapter 04 is one scene of eleven beats, as Mail 6884 allows: the
controlled sliding and turning experiment, both repeat rules and the plan
are kept whole. No beat was cut to meet a time estimate.

## 04_one_count_one_placement: One count, one placement (~116 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Before hunting for an exchange rate, look harder at the count itself. Here's the same sheet. If it slides slowly up the card, will the count stay at seven? ⟨predict⟩ | Where 03 ended: its title over the dimmed card, coast, sheet, dots and "7 crossings → ? cm", the envelope bright. The title, the count and its arrow fade as the card, coast, lines and seven dots brighten in 1 s. On "the count itself" a live counter, "7 crossings", fades in where the count was. Its number is the sheet's crossings at the placement shown, solved from the model every frame (`open.crossings`). Countdown | live counter from the model, think countdown | |
| 04_2 | No. The count jumps from seven to nine, dips to eight for a moment, and settles back at seven. The coast hasn't changed at all. The lines just meet it in different places. | First, in silence, the sheet slides 1.5 cm up the card (`slide.distance`) in 6.07 s. It moves at 0.35 cm a second, and at a fifth of that from 0.94 to 1.10 cm past the start, where the brief 8 lies (0.994 to 1.050 cm; `slide.shortest_from` 0.99, `slide.shortest_to` 1.05). Every frame the crossings are solved again: gold dots come and go where the lines meet the coast, and the counter follows. Each new count is logged in gold to the counter's right the moment it is read, 7, 9, 8, 7 (`slide.sequence`). The four counts show for 1.13, 2.28, 0.80 and 1.86 s. Then the words. On "dips to eight" a gold ring circles the logged 8. On "The coast hasn't changed" the ring goes, and the coast's pieces thicken for a moment and settle back | live crossings, counter and log from the model, highlight ring | hold |
| 04_3 | The sheet can turn, too. Give it an eighth of a turn about the middle of the card, and watch the count. | The log fades, since it belongs to that slide. On "Give it an eighth" a small `ink` ring marks the card's middle, and the sheet turns 45° about it in 5 s, rigidly, the counter following every frame. Then the sheet slides across its lines in 2.5 s to the rule's slide for that direction, 1.20 cm (`turn.45.slide`), and the ring goes | rotation, live counter | hold |
| 04_4 | It changed again. Now it's eight. | The counter reads 8 (`turn.45.crossings`); the picture holds | live counter | |
| 04_5 | So a count isn't a property of the coast alone. It comes from a placement of the sheet, and a placement is two choices. How far the sheet is slid, and which way its lines run. | On "two choices" two controls appear under the card, bound to the sheet. "slide" is a track from 0 to 2 cm, its knob at 1.20. "direction" is a half dial from 0° to 180°, its needle at 45°. On "How far the sheet is slid" a soft halo lies along the track, and the sheet slides 0.3 cm up and back, 1 s each way, the knob with it. On "which way its lines run" the halo moves to the dial, and the sheet turns 5° about the middle and back, the needle with it. The counter stays at 8 | controls bound to the sheet's trackers, highlight halo | |
| 04_6 | Some choices are repeats. If I slide the sheet by exactly one gap, two centimetres, what will the count be? ⟨predict⟩ | The dial's halo goes. On "slide the sheet by exactly one gap" one line gets a soft halo, and an `ink` bracket one gap long, labelled "2 cm", stands across from it to where the next line lies, over empty card (`sheet.gap`). On "what will the count be" the counter fades out. Countdown | think countdown, highlight halo | |
| 04_7 | Eight again. Every line lands exactly where its neighbour was, so the coast sees the same lines. Only slides within one gap are different. | First, in silence, dashed footprints of the lines stay where they lie, the counter returns, and the sheet slides one gap across its lines in 8 s at a steady pace, the counter following every frame. The knob runs to the track's end, reappears at 0 and comes back to 1.20. Each line lands on the footprint of the next, the haloed one at the bracket's far end, and the counter ends at 8. Then the words. On "Only slides within one gap" the halo moves to the track, and the bracket and the line's halo go | live counter, controls | hold |
| 04_8 | Turning has repeats too. Here's half a turn, about this point. | On "about this point" an `ink` ring marks a point midway between two lines, at (−0.14, 0.14) cm, the track's halo goes and the knob hides. The sheet turns 180° about the point in 7 s, rigidly, the needle sweeping round to where it started and the counter following every frame. The knob returns at 1.20, since half a turn names the slide the other way round | rotation, live counter | hold |
| 04_9 | The lines run the same way again, and they lie in the same places, so the count is eight again. Only directions within half a turn are different. | The lines have landed on their footprints, and the counter reads 8. On "Only directions within half a turn" a soft halo lies along the dial, and the footprints and the ring go. Then, in silence, the dial's halo goes and the sheet goes back to the first placement a choice at a time. It turns back 45° about the card's middle in 4 s, then slides to 0.45 cm in 2 s, the counter following | controls, live counter | hold |
| 04_10 | Our first count, seven, was one reading from one placement among many. | On "Our first count" gold rings circle the seven crossings of the first placement; on "one placement among many" they go | highlight rings | |
| 04_11 | So here's the plan. Try every slide and every direction, average the counts, and ask what that average knows about the coast. | On "Try every slide" the sheet slides one whole gap in 4 s, the knob running to the track's end and round to where it started, the counter following. Then, about a point on the middle line at (0, 0.45) cm and with the knob hidden, it turns half a turn in 5 s, the needle sweeping its dial. The controls, the ring and the counter fade, and "average count = ?" appears where the count was, the ? in gold | live counter, controls | hold |

### Notes for 04

- Every count on screen is solved from the model at the placement shown,
  every frame. The log, the rings and the words are refused at build time
  unless the model gives what the voice says: 7, 9, 8, 7 on the slide; 8
  after the eighth of a turn, after the gap slide and after the half turn;
  7 back at the first placement and at the end. The facts must say the
  same: `slide.distance`, `slide.sequence`, `turn.45.crossings` and
  `open.crossings`, which the oracle computes on its own.
- Refusal controls in the chapter's tests. Sliding down 1.5 cm reads
  7, 9, 7, 8, 9; a 1.0 cm slide reads 7, 9, 8; half a turn about the
  card's middle instead of the midway point ends at 11. Each is refused.
- Predictions. 04_1 can be reasoned from 02: lines meet the coast where
  they happen to cross it, so sliding should change the count. 04_6 can
  be solved from the bracket, since one gap carries each line onto its
  neighbour.
- In 04_2 and 04_7 the answer beat's picture leads its words. The
  experiment runs in silence after the countdown, the viewer watching the
  counter for themselves, and the words then name what was seen. Nothing
  that answers either question is on screen before the countdown ends.
- The brief 8 is the risk CrimsonGrove flagged (frame risk 4). At an even
  0.35 cm a second it would last 0.16 s (the tests' negative control).
  Slowed, it shows for 0.80 s, and the log keeps it on screen after.
- The half turn is made about a point midway between two lines, so the
  lines land on their footprints. Half a turn about any point leaves them
  running the same way; about another point they land slid, which is a
  slide within one gap. So directions within half a turn and slides within
  one gap still cover every placement. The test about the middle (11
  crossings) is that control. Root may want a "precisely" card for this.
- Semantic pauses: the two countdowns; the silent slide (04_2) and gap
  slide (04_7); the hold on "Now it's eight" (04_4); the silent return
  before 04_10; the stand on "average count = ?" (at least 2.6 s).
- The true length stays sealed. No word on screen contains 25.8, 25.81 or
  25.813 (a test).
- Pronunciation risks: "eighth" (EYTTH, one syllable plus th), "neighbour",
  "centimetres". Nothing has been listened to.

## 05_a_coin_that_never_changes: A coin that never changes (~79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | To find an exchange rate, start with a curve whose length we already know. Take a coin, two centimetres across, exactly as wide as the gap between the lines. | Where 04 ended. The question and the seven dots fade, and the card and coast dim to a quarter in 1 s; the sheet stays bright at the first placement. On "Take a coin" a teal coin 2 cm across fades in on the sheet, in the band over the coast at (1.5, 3.05) cm, across the line at 2.45. A short `ink` mark from near its centre to its rim will show it turning. On "two centimetres across" a teal bar as tall as the coin stands beside it, labelled "2 cm" (`coin.width`). On "exactly as wide as the gap" an `ink` bar of the same length runs beside it from the line at 2.45 to the line at 4.45 | coin, bars | |
| 05_2 | Drop it on the sheet. Its rim crosses this line twice. Now slide it and turn it as much as you like. How many crossings will it have, wherever it lands? ⟨predict⟩ | On "Drop it on the sheet" the bars go, and two gold dots appear where the rim meets the line at 2.45, solved from where the coin lies. On "crosses this line twice" a soft halo lies along that line. Countdown | live meeting points, highlight halo, think countdown | |
| 05_3 | Always two. The lines are exactly a coin's width apart, so one line runs through the coin, and only one. Its rim crosses that line twice. Turning changes nothing, because a circle looks the same turned any way. | On "Always two" a live counter, "2 crossings", fades in where the count was (`coin.crossings`). On "Turning changes nothing" the coin turns once on the spot in 2.4 s, its mark going round while the dots and the halo stay put. Then, in silence, it is set down at four more places, sliding and turning on the way, 1.6 s each with half a second's rest: (2.6, 0.75), (−1.5, −3.2), (5.2, −2.3) and (4.5, 1.85) cm. Every frame the meeting points and the line through it are solved again. The halo jumps to the next line the moment the coin's centre passes midway between two lines, and the counter stays at 2 | live meeting points, halo and counter from the model | hold |
| 05_4 | There's an awkward moment, when the coin fits exactly between two lines, just touching both. Then no line runs through it, but it meets each line at a single point, which still makes two. | On "when the coin fits exactly" the coin slides down 0.4 cm in 2.5 s to (4.5, 1.45), exactly between the lines at 0.45 and 2.45. As it comes, the two dots on the upper line close up; at the stop they become one at its top as one appears at its bottom, and the halo goes. On "a single point" gold rings circle the two dots, and the complete picture holds at least 2.6 s. The rings go. The coin is nudged 0.2 cm up, back, 0.2 cm down and back, 0.8 s each, one line through it each time, then stands at the touching placement for 2.6 s | live meeting points, highlight rings, counter | hold |
| 05_5 | Its rim is pi times its width. Roll the coin one full turn, and it covers six point two eight centimetres. | "rim = π × 2 cm" appears under the coast. The coin slides up to (0, 3.45) cm, between the lines at 2.45 and 4.45 and touching both, its mark pointing down at the line. On "Roll the coin one full turn" it rolls 2π cm to the right along the line at 2.45 in 3.5 s without slipping, the mark going round once. Its rim is laid out behind it as a teal stroke on that line; the dots stay at its top and bottom and the counter at 2. On "six point two eight" "6.28 cm" appears over the stroke (`coin.rim`) | rolling, live meeting points, TeX | |
| 05_6 | Two crossings for six point two eight centimetres. So for the coin, each crossing is worth pi centimetres, about three point one four. | "rim = π × 2 cm" goes, and "2 crossings ↔ 6.28 cm" appears under the coast. On "each crossing is worth" "1 crossing ↔ π cm ≈ 3.14 cm" appears under it (`coin.worth`). The picture stands at least 2.6 s | TeX derivation | hold |

### Notes for 05

- A crossing is wherever the coast meets a line (02_4), so for the coin a
  crossing is wherever its rim meets a line. A line nearer the centre than
  the radius meets the rim at two points; a line exactly the radius away
  meets it at one; a line further off misses it. The counter, the dots
  and the halo are computed from that rule every frame.
- "Always two" is tested at its edges. At every twentieth of a centimetre
  across one gap, against exact integer arithmetic, a coin of radius 1
  always meets the lines at two points, including the touching heights;
  radii 0.999 and 1.001 meet them at 0 or 2 and at 2 or 4 points. The
  chapter's own moves are checked every 0.01 cm, and in exact fractions
  every 1/400 of each move. Where the coin touches, it touches two lines
  at one point each. Nudged 0.2 cm either way, one line runs through it
  again.
- Prediction (05_2): solvable from 05_1's two bars, since the coin is
  exactly one gap across.
- No camera zoom for the touching placement (the draft's 05_4). A
  magnified inset flattens the arc against the line, so tangency would
  look like overlap. Instead the coin approaches slowly, the two dots
  close up, the rings mark the result and the picture holds.
- The roll lays the rim along a line, so the 6.28 cm is shown as a length
  on the sheet, not only stated. It ends where 06 can take the coin from.
- Semantic pauses: the countdown; the silent wander after 05_3 (the
  counter staying at 2 is the point); the hold on the touching placement
  and the stand after the nudges; the stand on the two rows.
- Pronunciation risks: "pi" (as "pie"), "six point two eight", "three
  point one four", "awkward", "coin's width". Nothing has been listened
  to.

## Differences from the full draft

- 04 has eleven beats where the draft has seven. 04_1 now ends with a
  prediction ("will the count stay at seven?"), and 04_2 answers it. The
  draft's 04_3 is split into 04_3 (the turn) and 04_4 (the count). The
  draft's 04_6 becomes 04_7, 04_8 and 04_9, one repeat rule each, with
  the half turn made, not described. The draft's 04_7 becomes 04_10 and
  04_11.
- 04_2 says the brief 8 ("dips to eight for a moment"), which the draft
  logged but did not say.
- The draft's ghost copy of the sheet (04_5) is dashed footprints of the
  lines. The draft's forty flashing placements in 04_7 are the actual
  survey: one whole gap of slides, then half a turn.
- In 05 the coin appears on the sheet, not rolling in beside the card; two
  bars compare its width with the gap. The draft's 5 s smooth wander is
  four landings. The draft's 05_4 zoom is replaced by a slow approach,
  rings and nudges. The draft's 05_5 is split into 05_5 (the rim, rolled
  out) and 05_6 (the exchange rate).
- 05_3 says "The lines are exactly a coin's width apart, so one line runs
  through the coin, and only one", where the draft says "A coin as wide as
  the gap always has exactly one line running through it". The draft's
  version is false at the touching placement, where no line runs through.
- Root decides whether to fold these rows back into the full draft.

## Integration steps for root

Rebased against the files as they stood at 10:12Z. Their sha256 prefixes:
`mod.rs` 77a655094f75866c, `main.rs` 2011973e834139af, `facts.rs`
d306f21403a18a21, `palette.rs` ce817eba2884ac69, `oracle.py`
29b79947f3df2227, `test_oracle.py` d5f3a90018fb5d4e, `model.rs`
ce81892a6dbdf281, `script.tsv` a9a0ed8f890215c6. Until steps 1 to 4 are
in, the two chapters do not compile (`palette::coin`, the new modules) or
their `words()` refuse (missing facts).

1. `src/chapters/mod.rs`. Declare the three modules, keeping the list in
   order:
   `pub mod a_coin_that_never_changes;` before `pub mod a_sheet_of_lines;`,
   and `pub mod one_count_one_placement;` and `pub mod placement;` after
   `pub mod glyphs;`. Add the registry entries after 03. Proposed
   intensities are 0.5 and 0.4:
   `Chapter { key: "04_one_count_one_placement", make: |kit| Box::new(one_count_one_placement::OneCountOnePlacement { kit }), intensity: 0.5 }`
   and
   `Chapter { key: "05_a_coin_that_never_changes", make: |kit| Box::new(a_coin_that_never_changes::ACoinThatNeverChanges { kit }), intensity: 0.4 }`.
   The module doc's "the draft's chapters 04 to 28" becomes "06 to 28".
2. `src/palette.rs`. A role for the coin, which 05 uses as
   `palette::coin()`, and the same role added to
   `every_role_is_its_own_color`'s list:

   ```rust
   /// The coin of known length (05), its rim and the rim laid out.
   pub fn coin() -> Srgb {
       Palette::teal()
   }
   ```

   In `docs/storyboard.md`'s Color roles table, add the row
   `| the coin of known length, its rim laid out | teal #5CD0B3 |` (the hex
   in backticks), and take "teal coin" out of the sentence under the table.
3. `src/facts.rs`. Two constants after `SHARE_DIRECTIONS`:

   ```rust
   /// Chapter 04's slide, cm, and the samples it is read at (the oracle's
   /// `SLIDE_SHOWN` and `SLIDE_SHOWN_STEPS`).
   pub const SLIDE_SHOWN: f64 = 1.5;
   pub const SLIDE_SHOWN_STEPS: usize = 15000;
   ```

   Then, after the `swap.*` loop and before the "film's identity" comment,
   using the `slide` already computed there:

   ```rust
   // Chapter 04: the slide, sampled as the oracle samples it, and the
   // eighth of a turn at the rule's slide
   let first = crate::chapters::placement::Pose::new(0.0, slide);
   let slid = crate::chapters::placement::Move::slide(first, SLIDE_SHOWN);
   let runs = crate::chapters::placement::runs(&coast, &slid, SLIDE_SHOWN_STEPS);
   f.add("slide.distance", fixed(SLIDE_SHOWN, 1))?;
   f.add(
       "slide.sequence",
       runs.iter().map(|r| r.count.to_string()).collect::<Vec<_>>().join(","),
   )?;
   let turned = crate::chapters::sheet::shown_slide(&coast, PI / 4.0);
   f.add("turn.45.slide", fixed(turned, 2))?;
   f.add("turn.45.crossings", crate::chapters::sheet::crossings(&coast, PI / 4.0, turned).len())?;

   // Chapter 05: the coin, as wide as the gap, at every twentieth of a
   // centimetre across one gap; each answer must be the same everywhere
   let coin = crate::chapters::a_coin_that_never_changes::Survey::across_a_gap(COIN / 2.0, slide);
   f.add("coin.width", fixed(COIN, 0))?;
   f.add("coin.rim", fixed(PI * COIN, 2))?;
   for (key, seen) in [
       ("coin.crossings", &coin.counts),
       ("coin.lines_through", &coin.through),
       ("coin.touching_lines", &coin.touched),
       ("coin.touch_points", &coin.points),
   ] {
       let [n] = seen.as_slice() else {
           return Err(format!("{key}: the coin shows {seen:?} across one gap"));
       };
       f.add(key, *n)?;
   }
   ```

   Expected values, all already printed by the oracle except the last
   three: `slide.distance` 1.5, `slide.sequence` 7,9,8,7, `turn.45.slide`
   1.20, `turn.45.crossings` 8, `coin.width` 2, `coin.rim` 6.28,
   `coin.crossings` 2, `coin.lines_through` 1, `coin.touching_lines` 2,
   `coin.touch_points` 1. `placement::runs` samples the slide at
   u = i/15000, i = 0 to 15000, the oracle's `first_slide + 1.5 i / 15000`.
4. `oracle.py`. The three new coin keys, in exact fractions, after the
   `check(COIN == GAP and …)` line of the coin block (4-space indent, inside
   the same function):

   ```python
       # chapter 05, exactly: a line d from a coin's centre meets its rim at 2
       # points if d < r, at 1 if d = r (x^2 = r^2 - d^2 = 0), else at none;
       # the coin set down at every 1/360 cm across one gap
       def coin_kinds(r):
           def roots(d):
               return 2 if d < r else 1 if d == r else 0
           kinds = set()
           for c in (Fraction(k, 360) for k in range(721)):
               per_line = [roots(abs(c - Fraction(GAP) * k)) for k in range(-3, 4)]
               kinds.add((per_line.count(2), per_line.count(1), sum(per_line)))
           return kinds

       seen = coin_kinds(Fraction(COIN) / 2)
       check(seen == {(1, 0, 2), (0, 2, 2)}, f"a coin as wide as the gap has one line through it and two points, or touches two lines at a point each: {sorted(seen)}")
       f["coin.lines_through"] = "1"
       f["coin.touching_lines"] = "2"
       f["coin.touch_points"] = "1"
   ```

   Run on its own (session scratch), the check passes. Radii 999/1000 and
   1001/1000 give {(0, 0, 0), (1, 0, 2)} and {(1, 0, 2), (2, 0, 4)}, which
   it would refuse. A `test_oracle.py` case for those two radii is
   optional; `test_the_coin_edge_is_where_the_count_changes` already
   covers them by sampling.
5. `src/main.rs`, the number guard. These rows let every number word in
   04 and 05 pass (checked; see below). SPOKEN, after 03_7's:
   `("04_1", "stay at seven", "open.crossings", "7")`,
   `("04_2", "from seven to nine, dips to eight for a moment, and settles back at seven", "slide.sequence", "7,9,8,7")`,
   `("04_4", "Now it's eight", "turn.45.crossings", "8")`,
   `("04_6", "two centimetres", "sheet.gap", "2")`,
   `("04_7", "Eight again", "turn.45.crossings", "8")`,
   `("04_9", "the count is eight again", "turn.45.crossings", "8")`,
   `("04_10", "Our first count, seven", "open.crossings", "7")`,
   `("05_1", "two centimetres across", "coin.width", "2")`,
   `("05_2", "crosses this line twice", "coin.crossings", "2")`,
   `("05_3", "Always two", "coin.crossings", "2")`,
   `("05_3", "one line runs through the coin, and only one", "coin.lines_through", "1")`,
   `("05_3", "crosses that line twice", "coin.crossings", "2")`,
   `("05_4", "between two lines", "coin.touching_lines", "2")`,
   `("05_4", "a single point", "coin.touch_points", "1")`,
   `("05_4", "still makes two", "coin.crossings", "2")`,
   `("05_5", "six point two eight centimetres", "coin.rim", "6.28")`,
   `("05_6", "Two crossings", "coin.crossings", "2")`,
   `("05_6", "six point two eight centimetres", "coin.rim", "6.28")`,
   `("05_6", "worth pi centimetres", "coin.worth", "3.14")`,
   `("05_6", "three point one four", "coin.worth", "3.14")`.
   DEFINITIONAL, after 03_5's (a choice, a unit or a definition, not a
   count the facts hold): `("04_5", "two choices")`,
   `("04_6", "exactly one gap")`, `("04_7", "within one gap")`,
   `("04_8", "half a turn")`, `("04_9", "within half a turn")`,
   `("04_10", "one reading from one placement")`,
   `("05_5", "pi times its width")`, `("05_5", "one full turn")`.
6. `docs/storyboard.md`. Move the two chapter sections above in after 03's
   notes, add their bullets to "Differences from the full draft", and
   update the status paragraph. Then regenerate the script with
   `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected.
7. Nothing in `glyphs.rs`: every string 04 and 05 set goes through the
   kit or `glyphs::record`, so the probe collects it. The new symbols are
   ↔ (TeX `\leftrightarrow`) and ≈. The pinned math engine maps both
   (`fmd-math/src/commands.rs`, franken_markdown 126a530), and the bundled
   Noto Sans Math subset has both glyphs (U+2194 and U+2248, read with
   fontTools). CM Unicode has neither, so they come from the fallback
   face.
8. Then Cargo fmt, Clippy and the tests, `facts-check`, the glyph probe
   and a first render of 04 and 05 for the frame risks below.

Duplication root may want to fold later. `placement::envelope` copies
03's private `envelope`. 04 copies 03's `ARROW`, `ASK`, `DIMMED`,
`COAST_W`, `TITLE_AT` and `TITLE_SIZE` to rebuild 03's end state. Making
03's items `pub(super)` would remove the copies, but it means editing the
delivered 03, which this assignment does not allow.

A non-blocking note on the delivered tests. `Curve::upto` and
`Curve::between` keep only a straight run's ends, so distances measured
against `upto` samples miss the interior of the beach. In 04 this made a
pivot look 0.43 cm from the coast when it is 0.425. The new tests sample
by arc length (`along`). Some frozen distance checks in `sheet.rs` and
`the_question.rs` use `upto` and are weaker than they read; the cases I
checked still pass with arc-length samples.

## Frame risks to reread at the first render

- 04's eighth of a turn and both half turns pass through many counts
  that last less than a frame (in the eighth, runs of 0.7, 1.8 and 1.4
  frames). The counter flickers there; the words do not name those
  counts. The knob is hidden during the half turns.
- 04's gap slide has a 0.0059 cm run of 8, 1.4 frames at 8 s.
- The sheet's panel is card-sized and does not turn with the lines.
  This reads as a sheet larger than the card; check that it does.
- Text extents are estimated, not measured. The checks assume
  "crossings" at 30 points is at most 1.8 frame units wide, "true length"
  at 20 points at most 1.6, and "pause for more time" at 18 points at
  most 2.4.
- 04's bracket sits over empty card 2.55 cm from the coast, its label
  3.23 cm; the pivots are 0.425 cm (middle), 0.567 cm (half-turn point)
  and 0.426 cm (survey point) from the coast.
- In 05 the coin passes over the dimmed coast between landings (its fill
  is 0.16 teal). At its stops it clears the coast by at least 0.245 cm
  (at (−1.5, −3.2)); the downward nudge clears it by 0.252 cm.
- During the roll the coin touches the line at 4.45, which lies 0.05 cm
  inside the card's top edge, and its top dot (radius 0.08 frame units)
  pokes just past that edge.
- ↔ and ≈ come from the Noto math subset, the rest of the rows from CM.
  Check that the mix reads as one line of type.
- The two rows sit between the coast's lowest point (frame y −1.363) and
  the line at −3.55 cm (frame −2.59), at y −1.78 and −2.22. By the
  estimate the upper row clears the coast by 0.36 and the lower row the
  line by 0.22. These are tight.
- At the touching placement the counter says "2 crossings" where the
  lines only touch. That follows 02_4's definition, and 05_4's words say
  so, but check that it reads as intended.
- `sheet::crossings` leaves out exact tangencies of the coast (they have
  measure zero). The coin's counter counts them, as 05_4 says. The two
  rules agree everywhere the film shows the coast.

## Timing, estimated and not measured

The tool's estimates count words, a breath, the prediction pause and a
2.5 s hold. The scenes' silent stretches are longer than that hold. Scene
seconds below are what the code plays, before the narration's own length.

- 04: the silent slide 6.07 s (04_2); the turn 5 s and settle 2.5 s
  (04_3); the nudges 4 s (04_5); the gap slide 8 s (04_7); the half turn
  7 s (04_8); the silent return 6 s (04_9 to 04_10); the survey 4 s and
  5 s (04_11); two 4 s countdowns. The tool estimates 116.3 s and prints
  its advisory for chapters over 85 s; Mail 6884 keeps 04 as one scene.
- 05: the turn on the spot 2.4 s and the silent wander 8.4 s (05_3); the
  approach 2.5 s, the touching hold of at least 2.6 s, the nudges 3.2 s
  and the stand 2.6 s (05_4); the move to the roll 1.6 s and the roll
  3.5 s (05_5); one 4 s countdown. The tool estimates 78.8 s.
- Phrase moments come from each sentence's share of characters. With no
  narrator these are estimates. In 05_4 the approach (2.5 s) has to fit
  between "when the coin fits exactly" and "a single point", about 4.9 s
  of words. In 05_5 the roll (3.5 s) runs from "Roll the coin" to about
  "six point two eight", about 3.4 s of words. If speech runs ahead, the
  label waits for the roll to finish.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --check` (the repo's `rustfmt.toml`, width 120)
  on the three new files exits 0.
- The fmn and kit calls were read against the pinned source (a399d626)
  and the kit. `Stage` derefs to the mobject stage (`fmn/src/lib.rs:1512`),
  whose `add_value_tracker`, `tracker_value` and `set_tracker_value`
  (`fmn-mobject/src/dynamics.rs:79,123,156`) the scenes use; the last one's
  error converts with `?` as in other topics. `set_color_lanes`,
  `set_family_opacity`, `always_redraw`, `add_updater` and the
  `fmn::core::rate` functions are the same calls the corpus makes. On the
  kit side: `Kit::text`, `text_t2c`, `tex` with `t2c`, `hold`, `finish`
  and `tween`, and `Facts::add`'s `Result<&mut Facts, String>`.
- Python transliterations of the new Rust and of every test assertion,
  run on a fresh `oracle.py` stdout (218 facts, 0 failed cross-checks;
  `test_oracle.py` 48 tests OK). They are session scratch, not committed.
  - `ch04_mirror`: 91 ok, 0 failed, after `sheet_mirror`'s own checks
    pass. It covers `placement.rs` and chapter 04. The runs are 7, 9, 8, 7
    and the brief 8 lies from 0.99405 to 1.04995 cm. Legs are 2.697,
    2.227 and 1.143 s, and every run shows for at least 0.799 s. The 13
    moves chain, the settles and returns are as planned, the nudges stay
    at 8, and the bracket and pivots clear the coast. It also runs the
    three refusals and the new check of the four facts.
  - `ch05_mirror`: 109 ok, 0 failed, after `sheet_mirror`'s checks pass.
    It covers the twentieths rule for radii 1, 0.999 and 1.001, and
    `Survey` for the same radii. Every 0.01 cm of every move meets the
    lines at 2 points. It also covers the drop, the touching placement
    and the nudges, the roll, the screen words, both refusals, the sealed
    length and the layout, including the think note and the rows' boxes.
    The INDEPENDENT parts are exact fractions every 1/400 of each move
    and at the drop, stop and roll heights, and the proposed oracle keys.
    Without those keys `words()` is refused, which is the expected state
    today.
  - The drop was moved from (2.0, 3.05) to (1.5, 3.05) cm after the
    mirror found its right meeting point 0.26 frame units from the think
    note. It is now 0.64.
  - `oracle_coin_patch`: the proposed oracle block above, run on its own
    with the oracle's constants: its check passes, and the two other radii
    fail it.
- `candidate0405_check`: `tools/storyboard.py`'s own `parse` and `check`
  on this file, nothing written. It also checks the words rule on the
  script these rows make, every phrase the two scenes wait for (read from
  their tests), and every SPOKEN row's value against the oracle's facts
  and the proposed coin keys. It runs `main.rs`'s number guard (its own
  word list and matching rule) with the rows above, plus negative
  controls. Results are in the handoff mail.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact. The independent parts are the oracle's own counts, the exact
  fraction checks and the arc-length coast samples.
