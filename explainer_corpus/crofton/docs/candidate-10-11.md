# Candidate chapters 10 and 11 (unregistered)

Written by GentleLantern for root's 10/11 assignment (Mail 6929,
2026-10-24, marker CROFTON1301-C10, with Mail 6931's correction to the 06
diagnosis), continuing `mx-video-program-program-87do.22.3`. Two new files,
neither registered nor compiled:

- `src/chapters/a_crowd_of_sticks.rs`: chapter 10, `ACrowdOfSticks`. It
  rebuilds 09's last frame over the shore and lays the coast's shadows on
  a bar. It also holds the model the facts are printed from (`piece`,
  `cuts`, `sticks`, `shadows`, `bar_spans`, `Marks`) and the `Shore` that
  both chapters draw on.
- `src/chapters/turning_the_sheet.rs`: chapter 11, `TurningTheSheet`,
  with the grid average `mean_total` and `GRID`/`MOST`. It starts on 10's
  `Shore` at 10's end.

No helper file was needed. Both chapters use `placement.rs`, `sheet.rs`,
`draw.rs`, 07's stick model, 08's console, rows and backings
(`the_shadow_rule`) and 09's words and plot helpers (`turning_the_stick`)
as they stand. They need root's 08/09 integration first (Mail 6913's
frozen files): 10 imports 08's and 09's public items, and its rebuild of
09's end calls 09's `words()`, which reads 08/09's facts. 10 and 11 must
also go in together. Some of `Shore`'s public fields (`beach_bar`) are
read only by 11, so 10 alone fails Clippy's dead-code lint.

The rows below are in `docs/storyboard.md`'s format, ready to move there
when root registers the scenes. `tools/storyboard.py`'s own `parse` and
`check` read this file as it stands. Nothing shared was edited: the
registry, `main.rs`, `model.rs`, `facts.rs`, `palette.rs`, `oracle.py`, the
scripts, the storyboards, chapters 01 to 09 and the pause overrides are as
root left them. Nothing here has been compiled, rendered, narrated or
listened to.

What the two chapters keep apart, as Mail 6929 asks:

- one placement's count: 7, at the first placement (10_2, 10_6, 10_9), and
  one placement's 7 × π ≈ 22.0 cm (11_4);
- the average over every slide at one direction: 15.26 ÷ 2 = 7.63 at 0°.
  It is exact for this model. It is 08's shadow rule summed over 67
  one-way sticks, and each stick's travel is in closed form (10_8). Then
  each of 11's four directions, 7.63 to 8.74, the same way;
- the four-direction estimate: the four slide averages, averaged, times π,
  25.53 cm, about 1% short (11_5);
- numerical angular quadrature: evenly spread directions from 0°, 4, 8,
  16, 32 and 3,600 of them, 25.53, 25.74, 25.80, 25.81, 25.81. The
  computer's 3,600 directions average 16.433 cm of shadow, which the screen
  halves and multiplies by π. That is computed, not proved (11_9). The
  exact identity behind it, 2L/π, is not claimed here. Chapter 12 says why.

## 10_a_crowd_of_sticks: A crowd of sticks (~118 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Back to the coast. Look closely at any smooth curve, and it's almost straight. So think of the coast as a crowd of tiny sticks, laid end to end. | Where 09 ended, rebuilt over the shore in 09's drawing order: the stick lying along the lines with its light and its point of shadow on the ruler, the ruler, 08's rule, 09's plot with the arch, its two measured points, the dashed half line, the flattened fill and its words, 09's rows, "6 cm" over the stick, and the ratio, the coin and its "?". The card and the coast are dimmed as 07 left them, 08's console and 09's panels are up. On "Back to the coast" all of 09 fades, with the console and the panels (0.4 s), and the card and the coast come back to full strength. On "Look closely" an inset fades in at the card's upper right: a disc of radius 1.2 showing the bay three times over round its lowest point, with a small ring on the bay where it looks. Inside it the magnified bay is almost straight, and the two whole sticks there are purple chords with their three end ticks. On "a crowd of tiny sticks" ink ticks across the coast at both ends of every stick, lagged over 2.5 s. There are 67 sticks, none longer than 0.45 cm, cut at the joints and where the coast lies level with the lines (`sticks.count`) | inset magnifier from the model, lagged ticks | |
| 10_2 | Every crossing of the coast is a crossing of one of these sticks. So the coast's count is just its sticks' counts, added up, and here that's seven. | On "Every crossing of the coast" the inset goes and the first placement's seven gold dots fade in. On "one of these sticks" a halo lights each of the seven sticks that hold them, each dot inside its own stick. On "here that's seven" "7 crossings" fades in at the card's upper left, counted live from the model (`open.crossings`) | highlight, live counter from the model | |
| 10_3 | Each stick runs one way across the lines, as the straight stick did, so the shadow rule holds for it. Its average count over every slide is its shadow divided by the gap. So the coast's average count is all its sticks' shadows, added up, then divided by the gap. | On "the shadow rule holds for it" 08's console fades in under the coast with 08's rule on its top row, `average over every slide = shadow ÷ gap`, and the seven halos go. On "all its sticks' shadows" the coast's rule under it, `average for the coast = all its shadows ÷ gap` | TeX | |
| 10_4 | Lay their shadows end to end along this bar, starting at the cove. Along the beach, nothing is added. It lies along the lines, so it casts no shadow. | On "Lay their shadows end to end" the top rule goes. On the console an axis under the bar's place, ticked at every gap of shadow, numbered "0" to "9" with "gaps" after the last: one frame unit a gap, 2 cm of shadow. A purple trace along the coast and the bar at the console's left both start empty. On "starting at the cove" the trace runs along the cove in 2.5 s and the bar grows with it, each stick's shadow a span, alternating full and lighter purple. On "Along the beach" the trace runs along the beach in 3 s and the bar does not grow. On "casts no shadow" "beach 0 cm" over the bar's end (`beach.shadow_0`). After the line, in silence, the trace and the bar run on over the climb, the rise, the headland and the fall in 7 s, to 3.85 gaps, unlabelled | live bar from the model | hold |
| 10_5 | Round the bay, the coast goes down and then comes back up, so both of its shadows go in. That's right, because a line through the bay crosses it twice, once on the way down and once on the way back up. | On "Round the bay" "beach 0 cm" goes, a halo lights the bay's way down, from its left end to its lowest point, and the trace and the bar run on over it in 2.5 s, its span haloed on the bar as well. On "comes back up" the way back up is haloed on the coast and on the bar, and laid in 2.5 s. On "a line through the bay crosses it twice" a halo on the lower line through the bay (`open.bay_crossings`). On "once on the way down" a gold ring round its crossing left of the lowest point. On "once on the way back up" a ring round the one right of it | live bar from the model, highlight | |
| 10_6 | Our first count was seven, and sliding showed nine and eight as well. With every shadow in, will the average come out above seven, or below? ⟨predict⟩ | On "Our first count was seven" the rings, the bay's halos and the line's halo go. The bar stands at 5.37 gaps, unlabelled. On "nine and eight as well" 04's readings in gold in the band over the coast, "7 9 8 7" (`slide.sequence`). On "above seven, or below" a dashed gold mark across the axis at seven gaps (`open.crossings`). Countdown at the card's upper right | think countdown | hold |
| 10_7 | Above. The last stretch, round the hook to the far end, finishes the bar. | On "Above" the countdown's ? goes. On "The last stretch" the trace and the bar run on over the point, the run, the hook and the end in 6 s. After the hook the bar is at 6.84 gaps; it passes the mark on the last piece and stops at 7.63 | live bar from the model | hold |
| 10_8 | In all, the shadows come to fifteen point two six centimetres. Divided by the gap, that's seven point six three crossings on average, over every slide. | On "fifteen point two six centimetres" the trace goes and "15.26 cm" stands over the bar's end (`turn.0.reach`). On "seven point six three crossings" the coast's rule becomes "15.26 ÷ 2 = 7.63 crossings, the average over every slide" (`turn.0.reach`, `sheet.gap`, `turn.0.slide_mean`) | TeX | |
| 10_9 | So seven was a low reading. And this average needed no sliding at all, just the shadows. But it holds only for lines running this one way. | The mark at seven, the readings and the bar's 7.63 stay. On "this one way" "lines at 0°" at the card's upper right, where the countdown stood. The picture stands at least 2.6 s | callback | hold |

### Notes for 10

- Additivity is earned on screen before it is used. Every crossing at the
  first placement lies inside one stick, at least 0.031 cm from its ends
  (`Marks::new` refuses a crossing within 0.02 cm of a stick's end, with a
  control). The seven lit sticks hold the seven dots. Each stick runs one
  way across the lines: its travel equals the height between its ends,
  which `words()` checks for every stick. The sticks' shadows add up to
  the coast's travel at every degree of half a turn (a test).
- "Each stick runs one way, as the straight stick did, so the shadow rule
  holds for it": 08's cut argument needs a piece whose height across the
  lines only rises or only falls, not a straight one. The sticks are the
  coast's own pieces. The inset's chords picture "almost straight" (each
  lies within about 0.01 cm of the bay). For a one-way piece the shadow
  is the same as its chord's, so the bar loses nothing to the chords.
- The bay counts twice (Mail 6929). At 0° the bay is two one-way pieces.
  It goes down 1.75 cm (its radius, from its left end at the centre's
  height to its lowest point) and comes back up 1.30 cm. Each way is
  haloed and laid separately, then the lower line and its two crossings,
  one on each way, are ringed one at a time. `Marks::new` refuses a
  coast whose bay is not crossed once each way by one line.
- The beach's zero comes before any abstraction (Mail 6929): 10_4 shows
  the bar standing still while the trace runs along the beach, and "beach
  0 cm". 11_2 then shows its whole 3.5 cm.
- Prediction (10_6): the bar stands at 10.75 cm, 5.37 gaps, when the
  question is asked, and the stretch left adds 4.51 cm. The bar passes the
  mark only on the last piece, so the answer is not on screen before
  "Above". The readings 7, 9, 8, 7 let the viewer reason that the average
  is above seven, and the bar then shows it.
- Cuts: the two ends, the joints, and the four points where an arc lies
  level with the lines (its angle a quarter turn off their normal). That
  makes 15 one-way stretches. Each is split into the fewest equal sticks
  no longer than 0.45 cm: 67 sticks, 0.236 to 0.438 cm. No stick length
  from 0.30 to 0.60 cm keeps every crossing more than 0.073 cm from a stick's
  end, so 0.45 stays and the 0.031 cm is a frame risk below.
- The live bar, its halos and the trace are drawn every frame from the
  sticks' shadows at the sheet's direction (`bar_spans`), so 11 turns them
  with the sheet without a second model.

## 11_turning_the_sheet: Turning the sheet (~102 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So far the lines have all run one way. Turn the sheet an eighth of a turn, and every shadow changes. Now the average comes to eight point seven four crossings. | Where 10 ended. On "So far the lines have all run one way" the count, its dots, 04's readings, the mark, "15.26 cm" and the result go. 09's panel comes back at the card's upper right, and on it a table under "average over every slide", with its first row "0° 15.260 ÷ 2 = 7.63" (`turn.0.reach_3dp`, `sheet.gap`, `turn.0.slide_mean`). On "Turn the sheet an eighth of a turn" "lines at 0°" goes and the sheet turns 45° about the card's centre in 3 s. The bar is redrawn every frame from the same sticks' shadows at the sheet's direction, and grows. On "eight point seven four crossings" the row "45° 17.485 ÷ 2 = 8.74" (`turn.45.reach_3dp`, `turn.45.slide_mean`) | live bar from the model, table built from the model | |
| 11_2 | Another eighth, and watch the beach. Lying along the lines, it cast no shadow. Now it runs straight across them, and its shadow is its whole length, three and a half centimetres. Here the average is eight point two three. | On "Another eighth" a halo lights the beach on the coast and its eight sticks' spans on the bar, then the sheet turns on to 90° in 3 s and the beach's spans grow to its whole length (`beach.shadow_0`, `beach.shadow_90`). On "three and a half centimetres" "beach 3.5 cm" over its spans (`beach.shadow_90`). On "eight point two three" the row "90° 16.452 ÷ 2 = 8.23" (`turn.90.reach_3dp`, `turn.90.slide_mean`) | live bar from the model, highlight | |
| 11_3 | And one more eighth, to three eighths of a turn. Here the average is seven point nine one. | On "And one more eighth" "beach 3.5 cm" and the beach's halos go, and the sheet turns on to 135° in 3 s. On "seven point nine one" the row "135° 15.821 ÷ 2 = 7.91" (`turn.135.reach_3dp`, `turn.135.slide_mean`) | live bar from the model, table built from the model | |
| 11_4 | Each direction sees the coast differently. Average those four, and multiply by pi. Will that beat the first placement's twenty-two point zero centimetres? ⟨predict⟩ | On "Average those four" under the bar's axis `(7.63 + 8.74 + 8.23 + 7.91) ÷ 4 × π`, the table's four averages as shown. On "twenty-two point zero centimetres" over the bar "one placement 7 × π ≈ 22.0 cm" (`open.crossings`, `open.estimate`). An opaque backing fades in at the card's upper left, where the count stood, and the countdown on it | think countdown, TeX | hold |
| 11_5 | Easily. Twenty-five point five three centimetres, only about one percent short of the true length. | On "Easily" the countdown and its backing go. On "Twenty-five point five three centimetres" "≈ 25.53 cm" in green after the row, which now reads "(7.63 + 8.74 + 8.23 + 7.91) ÷ 4 × π ≈ 25.53 cm" (`turn.<β>.slide_mean`, `turn.slide_estimate`). No rounded mean of the four is shown. On "about one percent short" "about 1% short" after it (`turn.slide_short_percent`) | TeX | |
| 11_6 | But four directions are only four. Turn the sheet back to the start, then through half a turn, and watch the total shadow. | On "Turn the sheet back to the start" the table gives way, on the same panel, to an empty plot: the direction "0°", "90°", "180°" across, the total shadow "14", "16", "18" up, under "total shadow in cm". The sheet turns back to 0° in 3 s, the bar with it. After the line, in silence: a gold point on the plot at 0° and 15.26, and the sheet turns half a turn in 12 s at a steady pace, about a point on one of its lines, so its lines end where they began. The point rides the curve, which is drawn behind it every frame from the coast at the sheet's direction, between 15.26 and 17.55; the bar follows. Then the point goes and the whole curve stays | live plot from the model, live bar | hold |
| 11_7 | Our four directions are these four points. More directions read the curve more finely. Eight give twenty-five point seven four, and thirty-two give twenty-five point eight one. | On "Our four directions are these four points" four gold points on the curve at 0°, 45°, 90° and 135°. The bar, its axis and 11_4's rows go, and on the console a list comes up, "directions" over "estimate in cm", its first column "4" over "25.53" (`grid.4.estimate`). On "Eight give" four more points between them, and the column "8" over "25.74" (`grid.8.estimate`). On "thirty-two give" the rest of sixteen and thirty-two directions' points, and the columns "16" over "25.80" and "32" over "25.81" (`grid.16.estimate`, `grid.32.estimate`). Each estimate is its directions' average shadow, over the gap, times π | plot, table built from the model | |
| 11_8 | Flatten the curve, as we flattened the stick's arch, and its height gives twenty-five point eight one centimetres, the true length to two decimal places. | On "Flatten the curve" the points go and the region under the curve fills in translucent purple. In 2.5 s its top moves straight up or down to one height, the curve's average, and the curve stays drawn over it. On "its height gives" its top in ink, "16.433 cm" at its right (`grid.3600.mean_reach`), and under the list "16.433 ÷ 2 = 8.2165" (`grid.3600.mean_crossings`). On "twenty-five point eight one centimetres" "8.2165 × π ≈ 25.81 cm" in green after it (`grid.3600.estimate`). On "the true length to two decimal places" "true length 25.81 cm" in green after that (`coast.length_2dp`) | morph from the model, TeX | |
| 11_9 | That's the computer averaging three thousand six hundred directions. It shows the bet working on this coast, but it doesn't prove that it must. Why it has to is the next question. | On "three thousand six hundred directions" the list's last column, "3600" over a green "25.81" (`grid.most`, `grid.3600.estimate`). On "it doesn't prove that it must" "computed, not proved" right of the list. The picture stands at least 2.6 s | callback | hold |

### Notes for 11

- The table halves each direction's travel shown to three places (17.485 ÷
  2 = 8.74, where 17.49 ÷ 2 would be 8.745). `words()` refuses a row whose
  shown operands do not give its shown average.
- 11_5 shows no rounded mean. The four shown averages, added, ÷ 4 × π,
  give 25.53. Their mean, 8.1275, would show as 8.13, and 8.13 × π reads
  25.54. A test and the oracle check both, as a control on the row's
  design.
- 11_2 ties direction to the beach seen in 10 (Mail 6929): lying along the
  lines at 0° it cast nothing; at 90° its eight sticks' spans on the bar
  make its whole 3.5 cm. `words()` refuses unless the beach's shadow at
  90° is its length.
- The grid (11_7 to 11_9) is a numerical average, kept apart from the
  identity. Each grid starts at 0° and steps 180° ÷ m (the endpoint
  rule), so the four are 11_1's directions. 8, 16 and 32 each hold the
  one before, so 11_7 adds only the new points. 3,600 does not hold 32 and
  is never drawn as points. On this coast every grid shown comes nearer
  the length from below: 25.5326, 25.7414, 25.7953, 25.8084, 25.8129. That
  is checked for this coast, not claimed in general. The 3,600-direction
  average shadow, 16.432981 cm, is 3.0e−7 from 2L/π. A midpoint grid of
  3,600 and endpoint grids of 7,200 and 14,400 round to the same shown
  16.433, 8.2165 and 25.81. So the screen's numbers do not depend on the
  grid, but nothing here proves the identity. The voice says so ("it
  doesn't prove that it must"), and the caption says "computed, not
  proved".
- New keys keep the numerical grid apart from the identity's.
  `grid.3600.mean_reach` and `grid.3600.mean_crossings` are the grid's
  average to three and four places. The oracle's
  `crofton.mean_reach_3dp` is 2L/π itself. `words()` refuses unless the
  grid's average agrees with it at the three places shown.
- Prediction (11_4): one placement's 22.0 cm against an average the
  viewer has not seen. The answer, 25.53, is not on screen before
  "Easily".
- The flattening moves each point of the curve's top straight up or down
  by the same share, to the 3,600-direction average. The polygon has 361
  samples, so its own average is over 360 directions. The two averages
  differ by 3.0e−5 cm, and the flattening changes the fill's area by
  1.2e−5 of it (the shoelace formula; see Checks), too little to see. The
  picture says "the curve's average", not "the same area".

## Differences from the full draft

- 10_1: the draft's camera zoom on the bay became an inset that magnifies
  the bay three times round its lowest point. There is no camera route,
  and the whole coast stays in view, with the two whole sticks there as
  chords and the ticks of all 67 on the coast.
- 10_2 and 10_3: the draft's "each tiny stick casts a small shadow" now
  earns its step. Every crossing is one stick's (the seven lit sticks).
  Each stick runs one way, so 08's rule holds for it, and then the coast's
  rule follows.
- The draft's column stacking on the ruler became a bar on 08's console,
  ticked in gaps, laid in order along the coast from the cove, with a
  trace on the coast showing the part being laid.
- The draft's 10_3 became 10_5: the bay's two ways are haloed and laid
  separately, then the line and its two crossings are ringed one at a
  time.
- The draft's 10_4 prediction is 10_6, with a mark at seven gaps. The
  draft's 10_5 became 10_7 and 10_8. Its range "7 to 9" is not shown,
  since it is not said.
- The draft's 10_6 ("The other slides, with eight and nine, pull the
  average up") became 10_9: seven was a low reading, the average needed no
  sliding, and it holds for one direction, which leads into 11.
- The draft's 11_1 told all three directions in one line. Here each eighth
  of a turn has its own line, and the draft's 11_2 (the beach along, then
  across) is shown at the 90° step.
- The draft's 11_3 "our first twenty-two" became "the first placement's
  twenty-two point zero centimetres", the 06 row as shown.
- The draft's 11_4 "mean 8.127 × π = 25.53 cm" became the four shown
  averages ÷ 4 × π ≈ 25.53, with no intermediate mean. The 06 bars and
  "−1.1%" are not used; "about 1% short" is.
- The draft's 11_5 ("with every direction, not just four, ... the true
  length, to every digit shown") became 11_6 to 11_9: the sweep drawing
  the curve, grids of 4 to 32 directions, the flattening, and 3,600
  directions named as the computer's average, computed and not proved.
  The draft bound `crofton.mean_reach_3dp` (in the oracle, 2L/π). 11_8
  binds the new `grid.3600.*` keys instead.
- The draft's 11_6 ("So the bet survives") became 11_9 ("It shows the bet
  working on this coast, but it doesn't prove that it must"). The
  envelope's "25.8" callback is dropped, since 06_4 already opened it.

## Proposed bindings

### Numbers (`main.rs`, the number guard)

These rows let every number word in 10 and 11 pass the guard (checked; see
below). SPOKEN, after 09_8's:
`("10_2", "here that's seven", "open.crossings", "7")`,
`("10_4", "casts no shadow", "beach.shadow_0", "0")`,
`("10_5", "crosses it twice", "open.bay_crossings", "2")`,
`("10_6", "Our first count was seven", "open.crossings", "7")`,
`("10_6", "nine and eight as well", "slide.sequence", "7,9,8,7")`,
`("10_6", "above seven, or below", "open.crossings", "7")`,
`("10_8", "fifteen point two six centimetres", "turn.0.reach", "15.26")`,
`("10_8", "seven point six three crossings", "turn.0.slide_mean", "7.63")`,
`("10_9", "So seven was a low reading", "open.crossings", "7")`,
`("11_1", "eight point seven four crossings", "turn.45.slide_mean", "8.74")`,
`("11_2", "it cast no shadow", "beach.shadow_0", "0")`,
`("11_2", "three and a half centimetres", "beach.shadow_90", "3.5")`,
`("11_2", "eight point two three", "turn.90.slide_mean", "8.23")`,
`("11_3", "seven point nine one", "turn.135.slide_mean", "7.91")`,
`("11_4", "twenty-two point zero centimetres", "open.estimate", "22.0")`,
`("11_5", "Twenty-five point five three centimetres", "turn.slide_estimate", "25.53")`,
`("11_5", "about one percent short", "turn.slide_short_percent", "1")`,
`("11_7", "twenty-five point seven four", "grid.8.estimate", "25.74")`,
`("11_7", "twenty-five point eight one", "grid.32.estimate", "25.81")`,
`("11_8", "twenty-five point eight one centimetres", "grid.3600.estimate", "25.81")`,
`("11_9", "three thousand six hundred directions", "grid.most", "3600")`.
DEFINITIONAL, after 09_8's (a choice, a unit or a definition, not a count
the facts hold): `("10_2", "one of these sticks")`,
`("10_3", "runs one way across the lines")`,
`("10_5", "once on the way down and once on the way back up")`,
`("10_9", "this one way")`, `("11_1", "all run one way")`,
`("11_3", "And one more eighth")`, `("11_3", "three eighths of a turn")`,
`("11_4", "Average those four")`, `("11_4", "multiply by pi")`,
`("11_6", "But four directions are only four")`, `("11_6", "half a turn")`,
`("11_7", "Our four directions are these four points")`,
`("11_7", "Eight give")`, `("11_7", "thirty-two give")`,
`("11_8", "two decimal places")`.

"casts no shadow" and "it cast no shadow" hold no number word. They are
SPOKEN so that the zero they claim is the beach's fact. "Eight give" and
"thirty-two give" name a grid, a choice, and their estimates are SPOKEN.
"the true length to two decimal places" is held to `coast.length_2dp` by
`words()`, which refuses unless 8.2165 × π, the 3,600 estimate and the
length all round to 25.81.

### Pauses (text-only intentions)

These are authored overrides with the same fields as root's
`docs/pause-overrides.json`, for root to append to its `overrides` list
after 09_8's (once 08/09's nineteen are in) and to extend its `_comment`
to 10/11. They are editorial hypotheses, not measured pacing, and nothing
has been listened to. The countdowns, the lays, the turns, the sweep and
the flattening stay protected visual time and are not to be trimmed to
these ranges. The check below resolves each one with
`tools/narration_director.py`'s own `directions()` on the script these
rows make, together with the integrated file's twenty-five and 08/09's
nineteen, and no two land on the same boundary. Each phrase occurs once in
its cue.

```json
[
  {"video": "crofton", "cue": "10_1", "after": "Back to the coast", "role": "thought", "range_s": [0.6, 1.0], "reason": "09's bench and plot fade and the coast comes back at full strength; let the eye find it again."},
  {"video": "crofton", "cue": "10_1", "after": "it's almost straight", "role": "thought", "range_s": [0.6, 1.0], "reason": "The inset's magnified bay and its two chords are inspected before the sticks are named."},
  {"video": "crofton", "cue": "10_2", "after": "a crossing of one of these sticks", "role": "thought", "range_s": [0.6, 1.0], "reason": "Seven sticks light up; match each gold dot to its stick before they are added up."},
  {"video": "crofton", "cue": "10_3", "after": "so the shadow rule holds for it", "role": "thought", "range_s": [0.5, 0.9], "reason": "08's rule returns on the console; read it before it is applied to one stick."},
  {"video": "crofton", "cue": "10_3", "after": "then divided by the gap", "role": "cue_transition", "range_s": [0.7, 1.2], "reason": "Both rules stand on the console; read the coast's under the stick's before the shadows are laid."},
  {"video": "crofton", "cue": "10_4", "after": "starting at the cove", "role": "thought", "range_s": [0.8, 1.1], "reason": "The cove's 2.5 s lay onto the bar finishes before the beach is named."},
  {"video": "crofton", "cue": "10_4", "after": "nothing is added", "role": "thought", "range_s": [0.5, 0.9], "reason": "The trace runs along the beach while the bar stands still; see it before the reason."},
  {"video": "crofton", "cue": "10_5", "after": "so both of its shadows go in", "role": "thought", "range_s": [0.6, 1.0], "reason": "Both of the bay's halos and both of its spans on the bar are compared before the reason."},
  {"video": "crofton", "cue": "10_5", "after": "a line through the bay crosses it twice", "role": "clause", "range_s": [0.35, 0.65], "reason": "The lower line's halo lands before its two crossings are ringed one at a time."},
  {"video": "crofton", "cue": "10_6", "after": "Our first count was seven", "role": "clause", "range_s": [0.35, 0.65], "reason": "The bay's marks clear; seven is looked at before the slide's readings come in."},
  {"video": "crofton", "cue": "10_7", "after": "Above", "role": "thought", "range_s": [0.5, 0.9], "reason": "The answer lands and the countdown's ? goes before the bar is laid past the mark."},
  {"video": "crofton", "cue": "10_8", "after": "fifteen point two six centimetres", "role": "thought", "range_s": [0.5, 0.9], "reason": "The trace fades and 15.26 cm stands over the bar's end; read it before it is halved."},
  {"video": "crofton", "cue": "10_9", "after": "So seven was a low reading", "role": "thought", "range_s": [0.5, 0.9], "reason": "Compare the mark at seven and 04's readings with the bar's 7.63."},
  {"video": "crofton", "cue": "10_9", "after": "just the shadows", "role": "thought", "range_s": [0.4, 0.8], "reason": "A breath before the caveat that this average holds for one direction."},
  {"video": "crofton", "cue": "11_1", "after": "So far the lines have all run one way", "role": "thought", "range_s": [0.5, 0.9], "reason": "10's count, readings and rows clear and the table comes up with its 0° row; read it before the turn."},
  {"video": "crofton", "cue": "11_1", "after": "and every shadow changes", "role": "thought", "range_s": [0.6, 1.0], "reason": "The 3 s eighth of a turn ends; watch the bar settle before the average is named."},
  {"video": "crofton", "cue": "11_2", "after": "and watch the beach", "role": "thought", "range_s": [0.5, 0.9], "reason": "The beach lights on the coast and on the bar as the turn begins."},
  {"video": "crofton", "cue": "11_2", "after": "three and a half centimetres", "role": "thought", "range_s": [0.5, 0.9], "reason": "'beach 3.5 cm' stands over its spans; compare it with 10's 'beach 0 cm'."},
  {"video": "crofton", "cue": "11_4", "after": "Each direction sees the coast differently", "role": "thought", "range_s": [0.5, 0.9], "reason": "The table's four rows, 7.63 to 8.74, are compared before they are averaged."},
  {"video": "crofton", "cue": "11_4", "after": "and multiply by pi", "role": "thought", "range_s": [0.4, 0.8], "reason": "The four-direction row is read before the comparison with one placement."},
  {"video": "crofton", "cue": "11_5", "after": "Easily", "role": "thought", "range_s": [0.5, 0.9], "reason": "The answer lands and the countdown goes before the number."},
  {"video": "crofton", "cue": "11_5", "after": "Twenty-five point five three centimetres", "role": "clause", "range_s": [0.35, 0.65], "reason": "The green 25.53 cm is read before how short it falls."},
  {"video": "crofton", "cue": "11_7", "after": "Our four directions are these four points", "role": "thought", "range_s": [0.6, 1.0], "reason": "The four gold points on the curve and the list's first column are matched with the table before more directions."},
  {"video": "crofton", "cue": "11_7", "after": "Eight give twenty-five point seven four", "role": "clause", "range_s": [0.35, 0.65], "reason": "Eight directions' points and column land before thirty-two."},
  {"video": "crofton", "cue": "11_8", "after": "as we flattened the stick's arch", "role": "clause", "range_s": [0.35, 0.65], "reason": "The 2.5 s flattening settles before its height is named."},
  {"video": "crofton", "cue": "11_8", "after": "twenty-five point eight one centimetres", "role": "clause", "range_s": [0.35, 0.65], "reason": "The green 25.81 cm is read beside the halving before the comparison with the true length."},
  {"video": "crofton", "cue": "11_9", "after": "but it doesn't prove that it must", "role": "thought", "range_s": [0.6, 1.1], "reason": "'computed, not proved' appears; the difference between a computed average and a proof is the chapter's last point."}
]
```

Visual holds between cues (the picture runs on while the voice waits).
These need render evidence before they enter a map, so here they are only
the scene's own seconds:

- after 10_4: the lay over the climb, the rise, the headland and the fall,
  7 s, after the line and its breath (`Kit::hold`).
- after 10_6 and after 11_4: the countdown, 4 s (`think`).
- after 10_7: the last lay is 6 s from "The last stretch". By the
  silent-render estimate it runs about 0.8 s past the line's end, in
  silence, until it is done.
- after 10_9 and after 11_9: the chapter's end, 2.6 s.
- after 11_6: the half turn, 0.3 s, 12 s and 0.3 s, after the line and its
  breath.
- inside 10_1: the ticks, lagged over 2.5 s; inside 10_4 and 10_5: the
  lays, 2.5 s, 3 s, 2.5 s and 2.5 s; inside 11_1 to 11_3 and 11_6: the
  turns, 3 s each; inside 11_8: the flattening, 0.4 s and 2.5 s. Each
  blocks the scene until it is done, so a phrase after it waits for it.

## Integration steps for root

Rebased against the shared files as they stood at 14:05Z, before root's
08/09 integration. Their sha256 prefixes: `mod.rs` b1bfe30ea99ec85a,
`main.rs` dff1a9534fbdff59, `facts.rs` 27be92d39bfcb699, `palette.rs`
167ee126372a2749, `oracle.py` 290d05d0b43a4a11, `test_oracle.py`
d5f3a90018fb5d4e, `model.rs` ce81892a6dbdf281, `placement.rs`
bfd93f4b571a797c, `sheet.rs` 89150645f8724438, `draw.rs`
197b87764d3ff38a, `script.tsv` ae2027918938452e (ends at 07_7),
`docs/storyboard.md` 26f70b3098e10444, `docs/pause-overrides.json`
f218a01902d855b4 (25 overrides), `the_bet.rs` 092be143c42826fa (root's
repaired 06), and 08/09's frozen `the_shadow_rule.rs` a3db91ca331bf45f and
`turning_the_stick.rs` fa0ae74e8cbb1aac. Do steps 1 to 9 of
`candidate-08-09.md` first. Until steps 1 to 4 here are in, the two
chapters do not compile (the new modules) or their `words()` refuse
(missing facts).

1. `src/chapters/mod.rs`. Declare the two modules, keeping the list in
   order: `pub mod a_crowd_of_sticks;` after `pub mod a_coin_that_never_changes;`,
   and `pub mod turning_the_sheet;` before `pub mod turning_the_stick;`.
   Add the registry entries after 09. Proposed intensities are 0.4 and 0.5
   (11 is the chapter's payoff):
   `Chapter { key: "10_a_crowd_of_sticks", make: |kit| Box::new(a_crowd_of_sticks::ACrowdOfSticks { kit }), intensity: 0.4 }`
   and
   `Chapter { key: "11_turning_the_sheet", make: |kit| Box::new(turning_the_sheet::TurningTheSheet { kit }), intensity: 0.5 }`.
   The module doc's "the draft's chapters 10 to 28" (after 08/09) becomes
   "12 to 28".
2. No palette change: both chapters use 07's `palette::stick()` for the
   bar, the trace, the chords and the plot, and 01 to 09's roles for the
   coast, the gold crossings and points, the green length and the teal
   coin.
3. `src/facts.rs`. After the `for m in LADDER` loop and before "the
   draft's 19_1", using the `coast` and `length` already bound in
   `facts()`:

   ```rust
   // Chapters 10 and 11: the coast cut into sticks, each one way across
   // the first lines; the beach's shadow along those lines and across
   // them; how far the four directions fall short; and evenly spread sets
   // of directions from 0°, the last the computer's (numerical averages,
   // which agree with the identity's 2L/π only to the places shown)
   let sticks = crate::chapters::a_crowd_of_sticks::sticks(&coast);
   f.add("sticks.count", sticks.len())?;
   let beach = crate::chapters::a_crowd_of_sticks::piece(
       &coast,
       coast.parts[model::BEACH].start,
       coast.parts[model::BEACH].end(),
   );
   f.add("beach.shadow_0", fixed(beach.travel(0.0), 0))?;
   f.add("beach.shadow_90", fixed(beach.travel(90.0), 1))?;
   let four_cm = coast.fixed_directions_estimate(SHOWN_DIRECTIONS.len(), GAP);
   f.add(
       "turn.slide_short_percent",
       fixed((length - four_cm) / length * 100.0, 0),
   )?;
   for m in crate::chapters::turning_the_sheet::GRID {
       f.add(
           &format!("grid.{m}.estimate"),
           fixed(coast.fixed_directions_estimate(m, GAP), 2),
       )?;
   }
   let most = crate::chapters::turning_the_sheet::MOST;
   f.add("grid.most", most)?;
   let height = crate::chapters::turning_the_sheet::mean_total(&coast, most);
   f.add(&format!("grid.{most}.mean_reach"), fixed(height, 3))?;
   f.add(&format!("grid.{most}.mean_crossings"), fixed(height / GAP, 4))?;
   ```

   Expected values: `sticks.count` 67, `beach.shadow_0` 0,
   `beach.shadow_90` 3.5, `turn.slide_short_percent` 1,
   `grid.4.estimate` 25.53, `grid.8.estimate` 25.74, `grid.16.estimate`
   25.80, `grid.32.estimate` 25.81, `grid.3600.estimate` 25.81,
   `grid.most` 3600, `grid.3600.mean_reach` 16.433,
   `grid.3600.mean_crossings` 8.2165. All twelve are new keys. The other
   keys the chapters read (`open.*`, `slide.sequence`, `turn.*`,
   `sheet.gap`, `coast.length_2dp`, `crofton.mean_reach_3dp`) are already
   printed. The block, set inside a function body, passes `rustfmt
   --check` at the repo's width. None of the names it binds (`sticks`,
   `beach`, `four_cm`, `most`, `height`) is bound or read later in root's
   `facts()`. `model`, `fixed`, `GAP` and `SHOWN_DIRECTIONS` are already in
   scope there.
4. `oracle.py`. Two constants after `SHOWN_DIRECTIONS = [0, 45, 90, 135]`
   (line 104 before 08/09's constants go in):

   ```python
   # Chapters 10 and 11: the longest stick the coast is cut into (cm), and the
   # sets of evenly spread directions from 0 degrees averaged (the last is the
   # computer's)
   STICK_LONGEST = 0.45
   DIRECTION_GRID = [4, 8, 16, 32, 3600]
   ```

   Then block 4 inside `main()`, 4-space indent, before the line
   `f["steinhaus.directions"] = str(STEINHAUS_DIRECTIONS)` (after the
   `for m in (4, 6):` ladder loop). It reads `geom`, `length`, `first`,
   `first_slide` and `slid_four` from `main()`:

   ```python
       # Chapter 10: the coast as a crowd of sticks. Each piece cut where it
       # lies level with the lines at 0 degrees (an arc at pi/2 + k pi) runs one
       # way across them; each such stretch split into the fewest equal sticks
       # no longer than STICK_LONGEST. A one-way stick's average count over
       # every slide is its shadow over the gap (08), so the coast's is all
       # the shadows added up, over the gap
       n0 = normal(0)
       stretches = []
       for g in geom:
           if g[0] == "S":
               stretches.append((g, math.dist(g[1], g[2])))
               continue
           (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
           lo, hi = min(a0, a1), max(a0, a1)
           k = math.ceil((lo - math.pi / 2) / math.pi)
           level = []
           while math.pi / 2 + k * math.pi < hi:
               if math.pi / 2 + k * math.pi > lo:
                   level.append(math.pi / 2 + k * math.pi)
               k += 1
           angles = [lo] + level + [hi]
           stretches.extend((("A", (cx, cy), r, a, b), r * (b - a)) for a, b in zip(angles, angles[1:]))
       check(all(len(one_way_pieces([g], n0)) == 1 for g, _ in stretches), f"cut where it lies level with the lines at 0 degrees, the coast runs one way in each of its {len(stretches)} stretches")
       check(any(len(one_way_pieces([g], n0)) > 1 for g in geom), "negative control: cut only at its joints, some piece of the coast turns back across the lines")
       check(abs(sum(w for _, w in stretches) - length) < 1e-9, "the stretches add up to the coast's length")
       sticks = []
       for g, w in stretches:
           k = math.ceil(w / STICK_LONGEST - 1e-9)
           if g[0] == "S":
               p, q = g[1], g[2]
               pts = [(p[0] + (q[0] - p[0]) * i / k, p[1] + (q[1] - p[1]) * i / k) for i in range(k + 1)]
               sticks += [("S", a, b) for a, b in zip(pts, pts[1:])]
           else:
               c, r, a, b = g[1], g[2], g[3], g[4]
               sticks += [("A", c, r, a + (b - a) * i / k, a + (b - a) * (i + 1) / k) for i in range(k)]
       f["sticks.count"] = str(len(sticks))
       check(all(len(one_way_pieces([s], n0)) == 1 for s in sticks), f"each of the {len(sticks)} sticks runs one way across the lines at 0 degrees")
       for beta in SHOWN_DIRECTIONS:
           laid = sum(travel([s], beta) for s in sticks)
           check(abs(laid - travel(geom, beta)) < 1e-9, f"at {beta} degrees the sticks' shadows add up to the coast's travel {laid:.6f}")
       zero = travel(geom, 0)
       check(fixed(float(f["turn.0.reach"]) / GAP, 2) == f["turn.0.slide_mean"], f"10_8: the shown {f['turn.0.reach']} / {GAP:g} rounds to the shown {f['turn.0.slide_mean']}")
       check(zero / GAP > first == min(slide_counts(geom, 0)), f"10_6 to 10_9: the average {zero / GAP:.4f} is above the first count {first}, the slide's lowest")
       # 10_5: the bay at 0 degrees is two one-way pieces, down then back up;
       # the line through it crosses each once
       bay = one_way_pieces([geom[BAY]], n0)
       check(len(bay) == 2 and bay[0][1] < bay[0][0] and bay[1][1] > bay[1][0], "10_5: at 0 degrees the bay goes down and comes back up, two shadows")
       check(line_count(bay, first_slide - GAP) == 2 == int(f["open.bay_crossings"]), "10_5: the line through the bay crosses it twice, once on each way")
       check(abs(sum(abs(b - a) for a, b in bay) - travel([geom[BAY]], 0)) < 1e-12 and abs(abs(bay[0][1] - bay[0][0]) - geom[BAY][2]) < 1e-12, "10_5: both of the bay's shadows go in, the way down its radius")
       # 10_4 and 11_2: the beach along the lines, then across them
       beach = [geom[BEACH]]
       f["beach.shadow_0"] = fixed(travel(beach, 0), 0)
       f["beach.shadow_90"] = fixed(travel(beach, 90), 1)
       check(travel(beach, 0) < 1e-12 and abs(travel(beach, 90) - math.dist(geom[BEACH][1], geom[BEACH][2])) < 1e-12, "10_4 and 11_2: the beach casts no shadow at 0 degrees and its whole length at 90")

       # Chapter 11: one placement, the four slid directions, and grids of
       # evenly spread directions (numerical averages, not the identity)
       check(fixed(first * math.pi, 1) == f["open.estimate"], f"11_4: the shown {first} x pi rounds to the shown {f['open.estimate']}")
       shown_four = sum(float(f[f"turn.{b}.slide_mean"]) for b in SHOWN_DIRECTIONS) / len(SHOWN_DIRECTIONS)
       check(fixed(shown_four * math.pi, 2) == f["turn.slide_estimate"], f"11_5: the four shown averages, averaged and times pi, round to the shown {f['turn.slide_estimate']}")
       check(fixed(float(fixed(shown_four, 2)) * math.pi, 2) != f["turn.slide_estimate"], f"negative control: a shown mean {fixed(shown_four, 2)} x pi would give {fixed(float(fixed(shown_four, 2)) * math.pi, 2)}, so 11_5 shows none")
       four_est = slid_four * worth()
       f["turn.slide_short_percent"] = fixed((length - four_est) / length * 100, 0)
       check(four_est < length and abs(length - first * worth()) > abs(length - four_est), f"11_5: four directions give {four_est:.4f}, short of {length:.4f} and nearer it than one placement's {first * worth():.4f}")
       grid = {}
       for m in DIRECTION_GRID:
           grid[m] = sum(travel(geom, 180 * k / m) for k in range(m)) / m
           f[f"grid.{m}.estimate"] = fixed(grid[m] / GAP * worth(), 2)
       est = [grid[m] / GAP * worth() for m in DIRECTION_GRID]
       check(all(a < b for a, b in zip(est, est[1:])) and est[-1] < length, "11_7: more directions come nearer the length, from below: " + ", ".join(f"{e:.4f}" for e in est))
       check(f[f"grid.{DIRECTION_GRID[0]}.estimate"] == f["turn.slide_estimate"] == f["ladder.4.coast_estimate"], "11_7's four directions are 11_5's")
       check(f[f"grid.{DIRECTION_GRID[3]}.estimate"] == f["coast.length_2dp"] != f[f"grid.{DIRECTION_GRID[2]}.estimate"], "11_7: thirty-two directions match the length to two places, sixteen do not")
       most = DIRECTION_GRID[-1]
       f["grid.most"] = str(most)
       f[f"grid.{most}.mean_reach"] = fixed(grid[most], 3)
       f[f"grid.{most}.mean_crossings"] = fixed(grid[most] / GAP, 4)
       gap_to_identity = abs(grid[most] - 2 * length / math.pi)
       check(f[f"grid.{most}.mean_reach"] == f["crofton.mean_reach_3dp"] and 0 < gap_to_identity < 1e-5, f"11_8: {most} directions average {grid[most]:.9f}, {gap_to_identity:.1e} from 2L/pi: the same to three places, computed and not exact")
       check(fixed(float(f[f"grid.{most}.mean_reach"]) / GAP, 4) == f[f"grid.{most}.mean_crossings"], f"11_8: the shown {f[f'grid.{most}.mean_reach']} / {GAP:g} gives the shown {f[f'grid.{most}.mean_crossings']}")
       check(fixed(float(f[f"grid.{most}.mean_crossings"]) * math.pi, 2) == f[f"grid.{most}.estimate"] == f["coast.length_2dp"], f"11_8: the shown {f[f'grid.{most}.mean_crossings']} x pi gives {f['coast.length_2dp']}, the true length to two places")
       shadows = [travel(geom, 180 * k / most) for k in range(most)]
       check(14.5 <= min(shadows) and max(shadows) <= 17.7, f"11_6: the total shadow runs {min(shadows):.4f} to {max(shadows):.4f} cm, inside the plot's 14 to 18")
   ```

   Block 4 is independent of the Rust. It cuts the oracle's own geometry
   at the level angles, splits the stretches into sticks itself and reads
   their shadows with the oracle's `one_way_pieces` and `travel`. Applied
   as written here to a scratch copy of the current `oracle.py` with
   08/09's step 4 already in (246 facts), the copy prints 258 facts with
   0 failed cross-checks: the 246 it printed before, every one unchanged,
   and the twelve keys above. Its negative controls are inside it: the
   coast cut only at its joints turns back across the lines, and a rounded
   mean 8.13 would give 25.54. Three changed grids fail it, as they should:
   32 and 3,600 swapped (the estimates no longer rise, and 32 directions
   are 2.8e−3 from 2L/π), 24 in place of 32 (no match with the length to
   two places), and 360 in place of 3,600 (3.0e−5 from 2L/π, outside the
   1e−5 the block allows). Of the names it binds in `main()`, the
   loop variables (`g`, `k`, `a`, `b`, `beta`, `m`) and `lo`, `hi`, `r`,
   `cx`, `cy`, `p`, `q`, `w`, `c` are assigned again before any later read,
   and the rest (`n0`, `stretches`, `sticks`, `bay`, `beach`, `grid`,
   `est`, `most`, `shadows` and so on) are not read later, so no later line
   changes. That is checked by the unchanged 246.
5. `src/main.rs`. The SPOKEN and DEFINITIONAL rows above, after 08/09's.
6. `docs/storyboard.md`. Move the two chapter sections above in after
   08/09's "### Notes for 09", at the end of the file, add their bullets to
   "Differences from the full draft", and change the status line's
   "native chapters 01 to 09" to "01 to 11". Then regenerate the script with
   `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected. `test_oracle.py`'s displayed-arithmetic test reads the moved
   pictures: their quoted equations, "15.260 ÷ 2 = 7.63" and the others,
   follow from their shown operands (checked below with its own
   `equations()`).
7. `docs/pause-overrides.json`. Append the twenty-seven overrides under
   "Pauses" to its `overrides` list, after 09_8's (71 in all with 08/09's),
   and change its `_comment`'s "through chapters01–09" to
   "through chapters01–11".
8. Nothing in `glyphs.rs`: every string 10 and 11 set goes through the kit,
   so the probe collects it. The TeX uses `\div`, `\times`, `\approx`,
   `\pi`, `\quad`, `\text` and `^\circ` as 06 to 09 set them, plus "(", ")"
   and "+" in 11_4's row (TeX math, no new font). The Text "about 1% short"
   needs "%", as 06's "about 15%" and 09's "about 64%" do, and "computed,
   not proved" a comma. If "%" fails the probe, "about 1 percent short" is
   the fallback, with the same SPOKEN row.
9. Then Cargo fmt, Clippy and the tests, `facts-check`, the glyph probe and
   a first render of 10 and 11 for the frame risks below.

Clippy lessons from 01 to 09 are applied, though Clippy has not run.
Every `pub` item and field is read in non-test code: `Shore`'s fields by
10 and 11, `Marks`' and both `Words`' by their scenes, and `sticks`,
`piece`, `GRID`, `MOST` and `mean_total` also by step 3. A cheap check
(below) confirms every imported name is used. All-constant asserts are in
`const { }` blocks. Range tests are written `(a..=b).contains(&x)`. No
function takes more than seven arguments. Neither file calls `.rt()` or
`.lag()` on an animation, or `set_opacity` on a stroked line.

## Frame risks to reread at the first render

- 10_2: a crossing can sit 0.031 cm (0.025 frame) from its stick's end, at
  the cove. Its halo still holds the dot, but only just. No stick length
  from 0.30 to 0.60 cm keeps every crossing more than 0.073 cm clear.
- 10_1's inset: the chords lie within 0.025 and 0.026 frame units of the
  magnified bay (chord width 3, coast width 4). That is the point ("almost
  straight"), but check the two chords and three ticks still read at 4K.
  The inset clears the coast by 1.04 and stays on the card.
- The bar, its halos and the trace are redrawn every frame from 67 pieces
  (`piece` and `travel` per stick), and in 11 the curve and the fill from
  361 samples each. Watch render speed.
- 10_4: the beach's eight sticks are eight spans of zero width while the
  trace runs along the beach. Check that no stub or dot shows at the bar's
  end.
- "beach 0 cm" is centred over the bar at frame x −4.86, 0.67 inside the
  console's left; "15.26 cm" over its end at 2.03. Both are over the bar,
  clear of the numbers under it.
- 10_1 rebuilds 09's last frame from 09's own `words()`, its private sizes
  copied with `_09` suffixes and its panel backings (`BACKS_09`). If root
  changes 09, 10's rebuild must follow. Compare 10's first frame with 09's
  last.
- 10_1: 09's panels fade with 09, so the lines behind them reappear; in
  11_1 the main panel comes back for the table, the strip with it.
- 10_5: the down lay starts on "Round the bay" with the halos. By the
  silent-render estimate "comes back up" follows about 3.6 s later, after
  the 2.5 s lay. Close phrases can make a visual land late. `wait_for`
  never errors when late; it only continues. In a silent render the only
  late visual is 10_4's beach lay, 0.8 s after "Along the beach", because
  the cove's 2.5 s lay is still running. The authored 0.8 to 1.1 s pause
  after "starting at the cove" is there to absorb it.
- 11_1 to 11_3 and 11_6: the sheet turns about the card's centre, then
  half a turn about a point on one of its lines. Check the lines' ends and
  the panel edges during the turns, as for 04's turns.
- 11_4's backing: an opaque 2.7 × 2.0 panel in the band over the coast at
  (−2.6, 2.75), where the count stood. The turned lines pass behind it;
  the coast stays at least 2.9 below its padded bottom.
- 11_6: the plot uses 09's panel, the curve between frame y 1.54 and 2.84.
  The point (radius 0.06) rides it.
- 11_7: the thirty-two points are 0.106 apart at radius 0.04, so 0.026
  clear. The list fades in over the bar's place while the bar, its axis
  and the rows fade out, in the same 0.4 s. Check the cross-fade.
- 11_8: "16.433 cm" runs from frame x 4.95 to 6.15, across the panel and
  its strip; the flattened top at frame y 2.21.
- Chained rows (11_5, 11_8) are placed with `next_to(RIGHT, gap 0.25,
  ORIGIN)`, so a TeX row and a Text row are centred on one height, not
  set on one baseline. Check they read as one row.
- Text extents are estimated, not measured: size/150 frame units a
  character and size/150 half-height.

## Timing, estimated and not measured

The tool's estimates count words, a breath, the prediction pause and a
2.5 s hold: 118.5 s for 10 and 102.2 s for 11, both above the former 85 s
target, which is now an advisory, not a cap. The check's silent-render
timeline, with the scenes' own animations and no narrator or authored
pauses, comes to about 120 s for 10 and 110 s for 11 before each 2.6 s
stand. Neither chapter is split, as Mails 6881 and 6929 allow. If root
wants shorter scenes, 10 has a conceptual boundary between 10_3 (the
rule for the coast) and 10_4 (laying the shadows), and 11 between 11_5
(four directions) and 11_6 (the sweep). Neither split needs a change to
any words. Scene seconds below are what the code plays.

- 10: the fades 0.4 s, the ticks 2.5 s (10_1); the cove 2.5 s, the beach
  3 s, then the climb to the fall 7 s after the line (10_4); the bay's two
  ways 2.5 s each (10_5); the countdown 4 s (10_6); the last stretch 6 s
  (10_7); the stand 2.6 s (10_9).
- 11: three eighths of a turn, 3 s each (11_1 to 11_3); the countdown 4 s
  (11_4); the turn back 3 s and the sweep 12.6 s after the line (11_6); the
  flattening 2.9 s (11_8); the stand 2.6 s (11_9).
- Phrase moments come from each sentence's share of characters, so with
  no narrator they are estimates.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --check` (the repo's `rustfmt.toml`) on both new
  files exits 0, and on step 3's block set in a scratch function.
- The fmn and kit calls were read against the integrated chapters and the
  kit as they stand: `placement.rs` (`at_once`, `glide`, `tracked`,
  `faded_line`, `halo_along`, `live_halo`, `live_lines`, `live_dots`,
  `live_count`, `live_word`, `half_turn_pin`, `Placement::begin`, `run`,
  `pose`), `draw.rs` (`card_panel`, `think`, `stand`, `say`, `fade_to`,
  `dim_line`, `together`, `fade_in_lagged`), `sheet.rs` and 08/09's
  frozen public items.
- `oracle.py` as it stands: 230 facts, 0 failed cross-checks. With
  08/09's step 4 applied to a scratch copy (their `apply_block3`): 246
  facts, 0 failed, the copy identical to the one checked for Mail 6928.
- `apply_block4` (session scratch) applies step 4 as written here to that
  copy: 258 facts, 0 failed, the 246 old keys unchanged, 12 new.
- `mirror_1011` (session scratch): a Python transliteration of the new
  Rust (`piece`, `cuts`, `sticks`, `shadows`, `bar_spans`,
  `sticks_within`, `Marks::new`, `Lens::new`, both `words()`,
  `mean_total`) and every test assertion, on the applied oracle's facts
  and the script these rows make. Result: 289 ok, 0 failed, including
  `sheet_mirror`'s own checks, run first. It includes eight negative
  controls on the two `words()` (a wrong stick count, a slide sequence
  under seven, a beach shadow, a sixteen-direction estimate, a short
  percent, a 3,600 halving, an identity value, a beach length) and the
  stick end moved onto a crossing. It also runs step 3's block, every key
  equal to the oracle's. Its INDEPENDENT parts:
  - every printed rounding is at least 4e−5 from an edge;
  - 2L/π = 16.432981626 rounds to the shown 16.433 and 8.2165;
  - a midpoint grid of 3,600 directions and endpoint grids of 7,200 and
    14,400 round the same;
  - dense arc-length samples keep the coast off the bay at least 1.7 from
    the inset's centre;
  - each scene's waited phrases come in order in their lines.
- `imports_check` (session scratch): every name either file imports is
  used in its non-test code (or, for the tests' imports, its tests), and
  every `pub` item and field is read in non-test code. 229 ok, 0 failed.
- `candidate1011_check` (session scratch) runs `tools/storyboard.py`'s
  own `parse` and `check` on this file, writing nothing, and checks:
  - the words rule on the script these rows make;
  - every line said once, in order, by its scene, and the phrases each
    scene waits for, read from its source and its test, each once in its
    line;
  - `main.rs`'s number guard (its own word list and matching rule) with
    the rows above, against the applied oracle's facts, plus negative
    controls;
  - every quoted equation in the two tables, with `test_oracle.py`'s own
    `equations()`, and controls;
  - the doc's facts block against the one rustfmt checked, and its oracle
    blocks against the ones run;
  - the pause overrides, with `narration_director.directions()`, on
    `script.tsv` with 08/09's and these lines added, alone and with the
    integrated file's twenty-five and 08/09's nineteen;
  - the handed-off files' hashes against the table below.

  Result: 324 ok, 0 failed (the two hash rows included). The storyboard
  tool's advisories (118.5 s and 102.2 s, above its 85 s hint) are
  advisory only, as the tool itself says.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact.

## Files handed off

| file | sha256 | bytes |
|---|---|---|
| `src/chapters/a_crowd_of_sticks.rs` | 6941464509c8cbbc77ef463199e02c6eb95410ea68baa3421f1f2ae2be00ffa2 | 61247 |
| `src/chapters/turning_the_sheet.rs` | 386e9c07c183c28830ac8f876fa78987a59710cb7b740586846e50e5689337bf | 45537 |

This file's own hash is in the handoff mail.
