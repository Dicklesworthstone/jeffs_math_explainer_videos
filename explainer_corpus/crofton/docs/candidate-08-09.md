# Candidate chapters 08 and 09 (unregistered)

Written by GentleLantern for root's 08/09 assignment (Mail 6913,
2026-10-24, marker GERMAN1133-C08), continuing
`mx-video-program-program-87do.22.3`. Two new files, neither registered nor
compiled:

- `src/chapters/the_shadow_rule.rs`: chapter 08, `TheShadowRule`. It
  rebuilds 07's end and runs the slide experiments, the cut and the rule's
  test. It also holds the model the facts are printed from (`slide_runs`,
  `share`, `mean`, `lines_in`, `pieces`, `cut_counts`, `reach`,
  `tilted_to`), and the `Bench` that both chapters draw the experiment
  with.
- `src/chapters/turning_the_stick.rs`: chapter 09, `TurningTheStick`, with
  the half turn's average shadow (`mean_shadow`). It starts on 08's
  `Bench` at 08's end.

No helper file was needed: 09 uses 08's public `Bench` and plot helpers.
Both chapters use `placement.rs`, `sheet.rs`, `draw.rs` and 07's stick
model (`straight_sticks::{ends, shadow, crossings}`) as they stand. The
rows below are in `docs/storyboard.md`'s format, ready to move there when
root registers the scenes. `tools/storyboard.py`'s own `parse` and `check`
read this file as it stands. Nothing shared was edited: the registry,
`main.rs`, `model.rs`, `facts.rs`, `palette.rs`, `oracle.py`, the scripts,
the storyboards, chapters 01 to 07 and the pause overrides are as root
left them. Both chapters build on root's 06/07 integration, which was in
the shared files by 13:05Z: they use `palette::stick`, 07's module and the
stick's facts. Nothing here has been compiled, rendered, narrated or
listened to.

What the two chapters keep apart, as Mail 6913 asks:

- one placement: the count at rest, 2 at the tilt and 2 at the test, and
  07's 9.4 cm or 0 cm;
- the slide average, one direction at a time: the step plot's two traces,
  1.5 and 2.25, and the rule, shadow ÷ gap;
- the direction average: the arch over half a turn, its flattened height
  3.82 cm, 1.91 crossings, 6.0 cm.

## 08_the_shadow_rule: The shadow rule (~138 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Now tilt the stick, so it lies thirty degrees from the lines. Its shadow shrinks to three centimetres. | Where 07 ended. The stick lies across the first placement's lines, with its three gold dots and "3 crossings". The ruler stands at the card's left with the purple shadow on it, the light from the stick's ends, and "shadow 6 cm" over its top. 07's two rows sit under the coast. On "Now tilt the stick" the rows and "shadow 6 cm" fade (0.4 s), and the stick turns 60° about its centre in 2.5 s, its upper end going left. Its count, dots, shadow and light are solved every frame from its direction; the count goes from 3 to 2 at 113.6°, once. On "thirty degrees from the lines" a dashed reference along the lines from the stick's centre, an arc and "30°" in the wedge (`stick.tilt`). On "shrinks to three centimetres" "shadow 3 cm" fades in over the ruler's top, on its own backing (`stick.reach30`) | live stick, live counter from the model, TeX | |
| 08_2 | At this placement it crosses two lines. Now hold the stick still, and slide the sheet up through one whole gap. Will the count stay at two? ⟨predict⟩ | The two dots and "2 crossings" (`stick.tilted_first`). On "slide the sheet up" the angle goes. An opaque console fades in under the coast, from the card's bottom to frame −1.55, with a backing behind the count and the faint footprints of the lines at rest. On it an empty step plot: the slide from "0" to "2 cm" across, the count "1" to "3" up. Countdown at the card's upper right. Then, after the countdown and in silence, the sheet slides up 2 cm in 8 s at a steady pace. The count, its dots and the plot's gold trace follow: 2 for 1 s, 1 for 4 s, 2 for 3 s (`stick.tilted_counts`). The lines land on their footprints | think countdown, live counter, live plot from the model | hold |
| 08_3 | No. It went from two to one, and back to two. Sliding through a whole gap tries every slide once, and it spent half of them at one crossing and half at two. So on average, over every slide, it crosses one and a half lines. | The trace whole. On "half of them at one crossing" a halo along its run at 1 (`stick.tilted_one_share`); on "and half at two" the halo moves to its two runs at 2 (`stick.tilted_two_share`). On "one and a half lines" a dashed `ink` line across the plot at 1.5 and "average 1.5" at its right (`stick.at30_mean`) | highlight | |
| 08_4 | Why one and a half? Cut the shadow into one whole gap, two centimetres, and the one centimetre left over. Slide the sheet through the same gap again, and watch where the lines meet the shadow. | On "Cut the shadow" an `ink` tick across the ruler, one gap above the shadow's lower end. On "one whole gap, two centimetres" "2 cm" beside the lower piece, on its own backing (`sheet.gap`). On "the one centimetre left over" "1 cm" beside the upper piece (`stick.tilted_leftover`). On "watch where the lines meet the shadow" a gold point on the ruler at each line's height inside the shadow. After the line, the same 8 s slide in silence: the points ride up the ruler with the lines. The lower piece holds one at every moment, and the upper one for 1 cm of the 2 (`stick.gap_piece_counts`, `stick.leftover_share`) | live points from the model | hold |
| 08_5 | The whole gap always held one line. As each one left at the top, the next came in at the bottom. The leftover centimetre held a line only while one passed through, half the time. One, plus a half. | On "The whole gap always held one line" a halo along the ruler's lower piece. On "The leftover centimetre" it moves to the upper piece. On "One, plus a half" it goes | highlight | |
| 08_6 | And one and a half is just the shadow, three centimetres, divided by the gap, two. That works for any shadow. Each whole gap of it holds one line, and the part left over holds one for its share of the slide. So for a straight stick at any angle, the average count over every slide is its shadow divided by the gap. | On "divided by the gap, two" `3 ÷ 2 = 1.5` on the console (`stick.reach30`, `sheet.gap`, `stick.at30_mean`). On "That works for any shadow" the tick, the pieces' words, their backings and the ruler's points fade. On "the average count over every slide" the rule, `average over every slide = shadow ÷ gap`, over it | TeX | |
| 08_7 | Let's test the rule. Turn the stick part of the way back, until its shadow is four and a half centimetres. What should its average count be now? ⟨predict⟩ | On "Turn the stick part of the way back" the first trace, its average and "shadow 3 cm" fade, and the stick turns back 18.6° in 2 s to 131.41°, against 08_1's tilt, its upper end rising: 48.6° from the lines. Its count stays at 2 all the way. On "four and a half centimetres" "shadow 4.5 cm" (`stick.test_shadow`). On "What should its average count be now" a second trace starts empty; countdown. Then, in silence, the same 8 s slide: 2 for 2 s, 3 for 2 s, 2 for 4 s (`stick.test_first`, `stick.test_counts`) | think countdown, live counter, live plot from the model | hold |
| 08_8 | Two and a quarter. It crossed three lines for a quarter of the slide, and two for the rest, just as four and a half divided by two says. | On "Two and a quarter" a dashed line at 2.25 and "average 2.25" (`stick.test_mean`). On "three lines for a quarter of the slide" a halo along the run at 3 (`stick.test_three_share`). On "four and a half divided by two" the halo goes and `4.5 ÷ 2 = 2.25` appears under the first use | highlight, TeX | |
| 08_9 | The rule agrees with what we saw before, too. Lying across, the shadow is six centimetres, three whole gaps, and six over two is three. Lying along, there's no shadow, and zero over two is zero. | On "six over two is three" `across 6 ÷ 2 = 3` at the console's right (`stick.shadow_across`, `stick.across_rule`). On "zero over two is zero" `along 0 ÷ 2 = 0` under it (`stick.shadow_along`, `stick.along_rule`). The stick stays at 131.41°. The picture stands at least 2.6 s | TeX, callback | hold |

### Notes for 08

- Prediction (08_2): "two", since the stick does not move. The slide
  shows that a single count depends on where the sheet is. That is why the
  slide average is needed, and the plot keeps the whole run of counts.
- The reason is the cut (08_4, 08_5), not "a longer shadow catches more"
  (a single placement need not). With lines at the slide's heights, the
  whole gap (lo, lo + 2] holds one line at every step (the floor rule),
  and the part left over holds one for its length's share of the slide.
  08 tests that exactly and on a grid of shadows and slides.
- The test (08_7) is a prediction on a shadow the viewer has not seen: 4.5
  cm, so 2.25, and three crossings for a quarter of the slide.
- Every still pose keeps the stick's ends at least 0.2 cm from the lines
  (`words()` refuses otherwise, with controls).

## 09_turning_the_stick: Turning the stick (~110 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | That settles the slide, one direction at a time. But the shadow, and so the average, still depends on which way the stick points. So now the other choice, the direction. | Where 08 ended: the stick at 131.41° with its two dots and "2 crossings", "shadow 4.5 cm", the step plot with the test's trace and "average 2.25", and the rule and its four uses on the console. On "That settles the slide" the plot, its trace, the four uses and the footprints fade; the rule stays. On "which way the stick points" the count, its dots, "shadow 4.5 cm" and their backings fade, and the stick turns on 48.6° in 2.5 s to lie along the lines, its shadow shrinking to a point | live stick | |
| 09_2 | Turning the stick is just like turning the sheet the other way, and half a turn tries every direction once. Watch the shadow as the stick turns. | On "Watch the shadow" an opaque panel fades in at the card's upper right, clear of the coast, with an empty plot: the stick's direction "0°", "90°", "180°" across, its shadow "0", "3", "6" up, under "shadow in cm". After the line, in silence: the stick, lying along and now named 0°, turns half a turn about its centre in 12 s at a steady pace. A purple column at its direction rises from the axis to its shadow, and behind it the arch is drawn from 0°. The ruler's shadow and light follow every frame | live stick, live plot from the model | hold |
| 09_3 | Lying along the lines, no shadow. Straight across, the full six centimetres. In between, the shadow traces this arch. Each point of it, divided by the gap, is the average count over every slide at that direction, like the two we measured. | The arch whole, the stick along again (`stick.shadow_along`, `stick.shadow_across`). On "like the two we measured" two gold points on it, 3 cm at 150° and 4.5 cm at 131.41°, 08's tilt and test (`stick.reach30`, `stick.test_shadow`) | callback | |
| 09_4 | So the average over every slide and every direction is the arch's average height, divided by the gap. Is that average height more than half the stick, or less? ⟨predict⟩ | On "more than half the stick" a dashed `dim` line across the plot at 3 cm, "half" at its right. Countdown where the count stood | think countdown | |
| 09_5 | More. Join the two ends of the arch to its top with straight lines. The triangle's area is half its width times the full height. So its average height is half the stick. The arch lies above that tent, so its average is larger. Flatten it into a rectangle with the same area, and its height is three point eight two centimetres, about sixty-four percent of the stick. | On "More" the countdown goes. On "Join the two ends of the arch" a white tent joins the endpoints to the peak. On "So its average height is half the stick" the half-height line briefly brightens. The triangular area is half the enclosing rectangle; the arch is strictly above the tent between endpoints and peak. On "Flatten it" the tent and "half" label fade while the region under the arch fills in purple behind the arch and measured points. The top flattens over 2.5 s, retaining area. On "three point eight two centimetres" the new height and "3.82 cm" appear; on "about sixty-four percent" show "about 64%". All geometry derives from the same LENGTH, angle and shadow model. | geometric lower bound, area-preserving morph | hold |
| 09_6 | Divide by the gap, and the stick's average count, over every slide and every direction, is one point nine one crossings. | On "one point nine one crossings" `and over every direction = 3.82 ÷ 2 = 1.91` under the rule (`stick.mean_reach`, `sheet.gap`, `stick.turned_mean`) | TeX | |
| 09_7 | Times pi centimetres a crossing, that's six point zero centimetres, the stick's own length. A single placement said nine point four or zero. Averaged over both choices, the slide and the direction, the bet gets the stick right. | On "six point zero centimetres" `1.91 × π cm ≈ 6.0 cm` in green under it (`stick.turned_mean`, `stick.turned_estimate`). On "the stick's own length" "6 cm" in green over the stick (`stick.length`). On "A single placement said" `one placement 9.4 cm or 0 cm` at the console's left (`stick.across_estimate`, `stick.along_estimate`) | TeX, callback | |
| 09_8 | That sixty-four percent is really two over pi. Keep it in mind. Later, a coin will explain it, with no calculus at all. | On "two over pi" `3.82/6 ≈ 0.64 ≈ 2/π` at the card's upper left (`stick.mean_reach`, `stick.length`, `stick.mean_reach_share`). On "a coin will explain it" a teal coin ring and a gold "?" beside it. The picture stands at least 2.6 s | TeX | hold |

### Notes for 09

- Prediction (09_4): "less", or "half", for a stick that spends time
  tilted. The arch's shape answers it before the number: above half from
  30° to 150°, two thirds of the turn.
- The arch, its column and the flattening are drawn every frame from the
  stick's direction through 07's model. The flattening moves each point of
  the top straight up or down by the same share, so its area never changes
  (a test, with the shoelace formula), and it ends at the average.
- 3.82 is a midpoint average over 3,600 directions, as `Curve::mean_travel`
  reads one. The exact value is 12/π = 3.8197; both round to every number
  shown (independent check).
- 09_8 plants the callback that the draft's 13_1 pays ("The coin told us
  that, without an integral"). 13_1's picture names "the 2/π from 09_7"; it
  is now 09_8.

## Differences from the full draft

- 08_1 to 08_2: the draft hid the count and asked how many crossings the
  tilted stick has. Here the count at rest is shown (2), and the prediction
  is whether it stays at two while the sheet slides. The slide answers it,
  and shows a single count depends on the slide.
- 08_2 to 08_3: the draft's tally bar became a step plot drawn from the
  model's runs. It keeps the order of the counts (2, 1, 2) and their
  shares; the slide is slower (8 s) so each count is seen.
- 08_3: the draft's "a longer shadow catches more of them" became the cut
  (08_4, 08_5), an argument the viewer watches: one whole gap always holds
  one line, and the part left over holds one for its share of the slide.
- 08_7 and 08_8 are new: the rule tested on a 4.5 cm shadow, a prediction,
  before the general statement is used.
- The draft's 08_4 callbacks are kept as 08_9.
- 09_1 is new: it closes the slide, one direction at a time, and names the
  direction as the other choice.
- 09_2: the half turn is silent, after the line, so the arch can be
  watched being drawn. Turning the stick is presented as turning the sheet
  the other way (a test).
- 09_3 adds 08's two measurements as points on the arch.
- 09_5: "spends a long time near its top" became "stays above half the
  stick for most of the turn, and falls to zero only at its two ends", both
  tested.
- 09_7: the draft's "the bet is exactly right" became "the bet gets the
  stick right". 6.0 is rounded, and 01 to 07 keep "exactly" for what is
  exact. The draft's flash of 04's controls became the one placement row,
  9.4 cm or 0 cm, from 07.
- The draft's 09_7 is 09_8.

## Proposed bindings

### Numbers (`main.rs`, the number guard)

These rows let every number word in 08 and 09 pass the guard (checked; see
below). SPOKEN, after 07_7's:
`("08_1", "thirty degrees", "stick.tilt", "30")`,
`("08_1", "shrinks to three centimetres", "stick.reach30", "3")`,
`("08_2", "crosses two lines", "stick.tilted_first", "2")`,
`("08_2", "stay at two", "stick.tilted_first", "2")`,
`("08_3", "from two to one, and back to two", "stick.tilted_counts", "2,1,2")`,
`("08_3", "half of them at one crossing", "stick.tilted_one_share", "0.50")`,
`("08_3", "and half at two", "stick.tilted_two_share", "0.50")`,
`("08_3", "one and a half lines", "stick.at30_mean", "1.5")`,
`("08_4", "Why one and a half", "stick.at30_mean", "1.5")`,
`("08_4", "one whole gap, two centimetres", "sheet.gap", "2")`,
`("08_4", "the one centimetre left over", "stick.tilted_leftover", "1")`,
`("08_5", "always held one line", "stick.gap_piece_counts", "1")`,
`("08_5", "half the time", "stick.leftover_share", "0.50")`,
`("08_5", "One, plus a half", "stick.at30_mean", "1.5")`,
`("08_6", "And one and a half", "stick.at30_mean", "1.5")`,
`("08_6", "the shadow, three centimetres", "stick.reach30", "3")`,
`("08_6", "divided by the gap, two", "sheet.gap", "2")`,
`("08_7", "four and a half centimetres", "stick.test_shadow", "4.5")`,
`("08_8", "Two and a quarter", "stick.test_mean", "2.25")`,
`("08_8", "three lines for a quarter of the slide", "stick.test_three_share", "0.25")`,
`("08_8", "and two for the rest", "stick.test_first", "2")`,
`("08_8", "four and a half divided by two", "stick.test_shadow", "4.5")`,
`("08_9", "the shadow is six centimetres", "stick.shadow_across", "6")`,
`("08_9", "three whole gaps", "stick.across_rule", "3")`,
`("08_9", "six over two is three", "stick.across_rule", "3")`,
`("08_9", "zero over two is zero", "stick.along_rule", "0")`,
`("09_3", "the full six centimetres", "stick.shadow_across", "6")`,
`("09_5", "three point eight two centimetres", "stick.mean_reach", "3.82")`,
`("09_5", "about sixty-four percent", "stick.mean_reach_percent", "64")`,
`("09_6", "one point nine one crossings", "stick.turned_mean", "1.91")`,
`("09_7", "pi centimetres a crossing", "coin.worth", "3.14")`,
`("09_7", "six point zero centimetres", "stick.turned_estimate", "6.0")`,
`("09_7", "nine point four", "stick.across_estimate", "9.4")`,
`("09_7", "or zero", "stick.along_estimate", "0")`,
`("09_8", "sixty-four percent", "stick.mean_reach_percent", "64")`.
DEFINITIONAL, after 07_4's (a choice, a unit or a definition, not a count
the facts hold): `("08_2", "one whole gap")`,
`("08_3", "every slide once")`, `("08_5", "each one left")`,
`("08_5", "while one passed through")`, `("08_6", "holds one line")`,
`("08_6", "holds one for its share")`, `("09_1", "one direction at a time")`,
`("09_2", "half a turn tries every direction once")`,
`("09_3", "the two we measured")`, `("09_4", "more than half the stick")`,
`("09_5", "above half the stick")`,
`("09_5", "falls to zero only at its two ends")`,
`("09_7", "A single placement")`, `("09_8", "two over pi")`.

"half the time" and the two shares are `0.50` because the facts print
shares to two places; "two over pi" is the exact value whose rounding
`stick.mean_reach_share` (0.64) is, so it is definitional.

### Pauses (text-only intentions)

These are authored overrides with the same fields as root's
`docs/pause-overrides.json` (Mail 6908), for root to append to its
`overrides` list after 07_7's and to extend its `_comment` to 08/09. They
are editorial hypotheses, not measured pacing, and nothing has been
listened to. The countdowns, slides, the half turn and the flattening stay
protected visual time and are not to be trimmed to these ranges. The check
below resolves each one with `tools/narration_director.py`'s own
`directions()` on the script these rows make, together with the integrated
file's twenty-five (root's ten and 06/07's fifteen), and no two land on the
same boundary. Each phrase occurs once in its cue.

```json
[
  {
    "video": "crofton",
    "cue": "08_1",
    "after": "Now tilt the stick",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "The 2.5 s tilt is under way; let the count fall from three to two before the angle is named."
  },
  {
    "video": "crofton",
    "cue": "08_2",
    "after": "At this placement it crosses two lines",
    "role": "thought",
    "range_s": [
      0.4,
      0.8
    ],
    "reason": "Time to count the two dots before the experiment is set up."
  },
  {
    "video": "crofton",
    "cue": "08_3",
    "after": "and back to two",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The answer is read off the finished trace before its shares are named."
  },
  {
    "video": "crofton",
    "cue": "08_4",
    "after": "the one centimetre left over",
    "role": "thought",
    "range_s": [
      0.4,
      0.8
    ],
    "reason": "Both pieces and their words are on screen before the second slide is announced."
  },
  {
    "video": "crofton",
    "cue": "08_5",
    "after": "The whole gap always held one line",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The halo on the lower piece is inspected before the reason it always holds one."
  },
  {
    "video": "crofton",
    "cue": "08_5",
    "after": "half the time",
    "role": "thought",
    "range_s": [
      0.4,
      0.8
    ],
    "reason": "The second piece's share lands before the two are added."
  },
  {
    "video": "crofton",
    "cue": "08_6",
    "after": "divided by the gap, two",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The first use of the rule appears on the console before it is generalised."
  },
  {
    "video": "crofton",
    "cue": "08_6",
    "after": "That works for any shadow",
    "role": "thought",
    "range_s": [
      0.4,
      0.8
    ],
    "reason": "The cut's marks clear before the general argument."
  },
  {
    "video": "crofton",
    "cue": "08_8",
    "after": "Two and a quarter",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The prediction's answer and its dashed average register before the check."
  },
  {
    "video": "crofton",
    "cue": "08_9",
    "after": "The rule agrees with what we saw before, too",
    "role": "thought",
    "range_s": [
      0.35,
      0.7
    ],
    "reason": "A breath before the two callbacks to 07."
  },
  {
    "video": "crofton",
    "cue": "09_1",
    "after": "one direction at a time",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The step plot and its uses fade; the slide is settled before the direction is raised."
  },
  {
    "video": "crofton",
    "cue": "09_1",
    "after": "which way the stick points",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "The count and labels clear while the stick turns to lie along the lines."
  },
  {
    "video": "crofton",
    "cue": "09_3",
    "after": "In between, the shadow traces this arch",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "Time to look at the finished arch before what each point means."
  },
  {
    "video": "crofton",
    "cue": "09_5",
    "after": "More",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The answer lands before the visible triangle supplies its geometric reason."
  },
  {
    "video": "crofton",
    "cue": "09_5",
    "after": "with the same area",
    "role": "clause",
    "range_s": [
      0.35,
      0.65
    ],
    "reason": "The 2.5 s flattening runs; its height is named once it has settled."
  },
  {
    "video": "crofton",
    "cue": "09_7",
    "after": "the stick's own length",
    "role": "thought",
    "range_s": [
      0.6,
      1.1
    ],
    "reason": "The payoff: the green estimate and the green 6 cm over the stick are compared."
  },
  {
    "video": "crofton",
    "cue": "09_7",
    "after": "nine point four or zero",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The one placement row is read against the averaged one before the conclusion."
  },
  {
    "video": "crofton",
    "cue": "09_8",
    "after": "two over pi",
    "role": "thought",
    "range_s": [
      0.5,
      0.9
    ],
    "reason": "The ratio's TeX is read before the promise of the coin."
  },
  {
    "video": "crofton",
    "cue": "09_5",
    "after": "with straight lines",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "Inspect the tent under the same arch before discussing its area."
  },
  {
    "video": "crofton",
    "cue": "09_5",
    "after": "So its average height is half the stick",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "Compare the triangle with the half-height line before comparing curve areas."
  },
  {
    "video": "crofton",
    "cue": "09_5",
    "after": "so its average is larger",
    "role": "thought",
    "range_s": [
      0.5,
      1.0
    ],
    "reason": "Let the geometric lower bound land before measuring the exact average by flattening."
  }
]
```

Visual holds between cues (the picture runs on while the voice waits).
These need render evidence before they enter a map, so here they are only
the scene's own seconds:

- after 08_2 and after 08_7: the countdown, 4 s (`think`), then the slide,
  8 s.
- after 08_4: the slide, 8 s, after the line and its breath (`Kit::hold`).
- after 08_9: the chapter's end, 2.6 s.
- after 09_2: the half turn, 0.3 s, 12 s and 0.3 s, after the line and its
  breath.
- after 09_4: the countdown, 4 s.
- after 09_8: the chapter's end, 2.6 s.
- inside 09_1: the turn to lie along, 2.5 s; inside 09_5: the flattening,
  0.4 s and 2.5 s; inside 08_1 and 08_7: the tilts, 2.5 s and 2 s. Each
  blocks the scene until it is done, so a phrase after it waits for it.

## Integration steps for root

Rebased against the files with root's 06/07 integration in, as they stood
at 13:05Z and again at 13:10Z. Their sha256 prefixes: `mod.rs`
b1bfe30ea99ec85a, `main.rs` dff1a9534fbdff59, `facts.rs`
27be92d39bfcb699, `palette.rs` 167ee126372a2749, `oracle.py`
290d05d0b43a4a11, `test_oracle.py` d5f3a90018fb5d4e, `model.rs`
ce81892a6dbdf281, `placement.rs` bfd93f4b571a797c, `sheet.rs`
89150645f8724438, `draw.rs` 197b87764d3ff38a, `script.tsv`
ae2027918938452e (ends at 07_7), `docs/storyboard.md` 59bb6563063acb31,
`docs/pause-overrides.json` f218a01902d855b4 (25 overrides), the
integrated `the_bet.rs` 799634f9176f6ae1 and `straight_sticks.rs`
20feb335521c4d0c (root changed only their module doc comments from what I
delivered), and `candidate-06-07.md` 412ff85f0e94deef. The 06/07 lines in
`script.tsv` are the candidate's words, none reworded. Until steps 1 to 4
here are in, the two chapters do not compile (the new modules) or their
`words()` refuse (missing facts).

1. `src/chapters/mod.rs`. Declare the two modules, keeping the list in
   order: `pub mod the_shadow_rule;` and `pub mod turning_the_stick;` after
   `pub mod the_question;`. Add the registry entries after 07. Proposed
   intensities are 0.4 and 0.5 (the payoff):
   `Chapter { key: "08_the_shadow_rule", make: |kit| Box::new(the_shadow_rule::TheShadowRule { kit }), intensity: 0.4 }`
   and
   `Chapter { key: "09_turning_the_stick", make: |kit| Box::new(turning_the_stick::TurningTheStick { kit }), intensity: 0.5 }`.
   The module doc's "the draft's chapters 08 to 28" becomes "10 to 28".
2. No palette change: both chapters use 07's `palette::stick()`, and 01 to
   07's roles for the coast, the gold crossings, the green length and the
   teal coin.
3. `src/facts.rs`. After the chapter 07 block (its `stick.shadow_*` loop)
   and before "the film's identity", using the `slide`, `worth` and `home`
   already bound there:

   ```rust
   // Chapters 08 and 09: the stick on screen tilted 08's angle from the
   // lines, then on until its shadow is 08_7's test, each held still while
   // the sheet slides one gap up from the first placement (the counts it
   // passes, the share of the slide at each, the average); the tilt's
   // shadow cut one gap above its lower end, and what each piece holds;
   // the rule on 07's two readings; and the average shadow over 09's half
   // turn
   let tilted = crate::chapters::the_shadow_rule::TILTED;
   let tested = crate::chapters::the_shadow_rule::tilted_to(crate::chapters::the_shadow_rule::TEST_SHADOW);
   let slides = [tilted, tested].map(|theta| crate::chapters::the_shadow_rule::slide_runs(theta, slide));
   f.add(
       "stick.tilt",
       fixed(crate::chapters::the_shadow_rule::TILT.to_degrees(), 0),
   )?;
   for (theta, runs, [first, counts]) in [
       (tilted, &slides[0], ["stick.tilted_first", "stick.tilted_counts"]),
       (tested, &slides[1], ["stick.test_first", "stick.test_counts"]),
   ] {
       f.add(
           first,
           crate::chapters::straight_sticks::crossings(home, theta, slide).len(),
       )?;
       let passed: Vec<String> = runs.iter().map(|r| r.count.to_string()).collect();
       f.add(counts, passed.join(","))?;
   }
   let share = crate::chapters::the_shadow_rule::share;
   let reach = crate::chapters::the_shadow_rule::reach;
   f.add("stick.tilted_one_share", fixed(share(&slides[0], 1), 2))?;
   f.add("stick.tilted_two_share", fixed(share(&slides[0], 2), 2))?;
   f.add(
       "stick.at30_mean",
       fixed(crate::chapters::the_shadow_rule::mean(&slides[0]), 1),
   )?;
   f.add("stick.reach30", fixed(reach(tilted), 0))?;
   f.add("stick.tilted_leftover", fixed(reach(tilted) - GAP, 0))?;
   let (whole, held) = crate::chapters::the_shadow_rule::cut_counts(tilted, slide);
   let whole: Vec<String> = whole.iter().map(i64::to_string).collect();
   f.add("stick.gap_piece_counts", whole.join(","))?;
   f.add("stick.leftover_share", fixed(held, 2))?;
   f.add("stick.test_shadow", fixed(reach(tested), 1))?;
   f.add("stick.test_three_share", fixed(share(&slides[1], 3), 2))?;
   f.add(
       "stick.test_mean",
       fixed(crate::chapters::the_shadow_rule::mean(&slides[1]), 2),
   )?;
   f.add(
       "stick.across_rule",
       fixed(reach(crate::chapters::straight_sticks::ACROSS) / GAP, 0),
   )?;
   f.add(
       "stick.along_rule",
       fixed(reach(crate::chapters::straight_sticks::ALONG) / GAP, 0),
   )?;
   let shadow_mean = crate::chapters::turning_the_stick::mean_shadow();
   let stick = crate::chapters::straight_sticks::LENGTH;
   f.add("stick.mean_reach", fixed(shadow_mean, 2))?;
   f.add("stick.mean_reach_share", fixed(shadow_mean / stick, 2))?;
   f.add("stick.mean_reach_percent", fixed(shadow_mean / stick * 100.0, 0))?;
   f.add("stick.turned_mean", fixed(shadow_mean / GAP, 2))?;
   f.add("stick.turned_estimate", fixed(shadow_mean / GAP * worth, 1))?;
   ```

   Expected values: `stick.tilt` 30, `stick.tilted_first` 2,
   `stick.tilted_counts` 2,1,2, `stick.tilted_one_share` 0.50,
   `stick.tilted_two_share` 0.50, `stick.at30_mean` 1.5, `stick.reach30` 3,
   `stick.tilted_leftover` 1, `stick.gap_piece_counts` 1,
   `stick.leftover_share` 0.50, `stick.test_shadow` 4.5,
   `stick.test_first` 2, `stick.test_counts` 2,3,2,
   `stick.test_three_share` 0.25, `stick.test_mean` 2.25,
   `stick.across_rule` 3, `stick.along_rule` 0, `stick.mean_reach` 3.82,
   `stick.mean_reach_share` 0.64, `stick.mean_reach_percent` 64,
   `stick.turned_mean` 1.91, `stick.turned_estimate` 6.0. The oracle
   already prints `stick.at30_mean`, `stick.reach30`, `stick.mean_reach`,
   `stick.mean_reach_share`, `stick.turned_mean` and
   `stick.turned_estimate` from its own placements; the other sixteen come
   from step 4. The block, set inside a function body, passes
   `rustfmt --check` at the repo's width. None of the names it binds
   (`tilted`, `tested`, `slides`, `share`, `reach`, `whole`, `held`,
   `shadow_mean`, `stick`) is bound elsewhere in root's integrated
   `facts()`. It avoids `runs`, `turned` and `mean`, which `facts()` binds
   for 04 and the film's identity.
4. `oracle.py`. Two constants after `STICK_TURN = 90` (line 103):

   ```python
   # chapters 08 and 09: the stick tilted this far from the lines, degrees,
   # then on until its shadow is this long, cm
   STICK_TILT = 30
   TEST_SHADOW = 4.5
   ```

   Then block 3 inside `main()`, 4-space indent, after the existing
   `check(abs(at30 - STICK * 0.5 / GAP) < 1e-3, …)` (line 895). It reads
   `first_slide`, `mean_reach` and `f["stick.at30_mean"]` from `main()` and
   07's `home`:

   ```python
       # chapters 08 and 09 on screen: chapter 07's stick tilted STICK_TILT
       # degrees from the lines (direction 180 - STICK_TILT, its upper end to
       # the left as 07 turned it), then until its shadow is TEST_SHADOW cm.
       # At each tilt the sheet slides one gap up from the first placement,
       # read at 20000 midpoints: the counts it passes in order, the share of
       # the slide at each, and the average. At the first tilt the shadow, cut
       # one gap above its lower end, holds exactly one line in that whole gap
       # at every step, and a line in the part left over for that part's
       # share of the slide. Both still poses keep the stick's ends a fifth of
       # a centimetre from the lines. Chapter 09 turns the stick half a turn
       # instead of the sheet: its average shadow, read at 3600 midpoint
       # directions, is the sheet-turned mean_reach above
       slide_steps = 20000

       def slid(angle):
           seen, shares, total = [], {}, 0
           for i in range(slide_steps):
               n = crossings(home(angle), 0, first_slide + GAP * (i + 0.5) / slide_steps)
               if not seen or seen[-1] != n:
                   seen.append(n)
               shares[n] = shares.get(n, 0) + 1
               total += n
           return seen, {n: k / slide_steps for n, k in shares.items()}, total / slide_steps

       def shadow_of(angle):
           [(_, (_, y0), (_, y1))] = home(angle)
           return min(y0, y1), max(y0, y1)

       def off_lines(h):
           t = (h - first_slide) / GAP
           return abs(t - round(t)) * GAP

       tilted = 180 - STICK_TILT
       tested = 180 - math.degrees(math.asin(TEST_SHADOW / STICK))
       seen, shares, mean = slid(tilted)
       lo, hi = shadow_of(tilted)
       first_count = crossings(home(tilted), 0, first_slide)
       f["stick.tilt"] = fixed(STICK_TILT, 0)
       f["stick.tilted_first"] = str(first_count)
       f["stick.tilted_counts"] = ",".join(map(str, seen))
       f["stick.tilted_one_share"] = fixed(shares.get(1, 0), 2)
       f["stick.tilted_two_share"] = fixed(shares.get(2, 0), 2)
       f["stick.tilted_leftover"] = fixed(hi - lo - GAP, 0)
       whole, held = set(), 0
       for i in range(slide_steps):
           s = first_slide + GAP * (i + 0.5) / slide_steps
           whole.add(math.floor((lo + GAP - s) / GAP) - math.floor((lo - s) / GAP))
           held += math.floor((hi - s) / GAP) - math.floor((lo + GAP - s) / GAP)
       f["stick.gap_piece_counts"] = ",".join(map(str, sorted(whole)))
       f["stick.leftover_share"] = fixed(held / slide_steps, 2)
       check(seen == [first_count, first_count - 1, first_count],
             f"tilted {STICK_TILT} degrees, the slide passes {seen}: two, one, two")
       check(fixed(mean, 1) == f["stick.at30_mean"] and abs(mean - (hi - lo) / GAP) < 1e-3,
             f"the slide on screen averages {mean:.4f}, its shadow over the gap")
       check(abs(held / slide_steps - (hi - lo - GAP) / GAP) < 1e-3, "the part left over holds a line for its share of the slide")
       t_seen, t_shares, t_mean = slid(tested)
       t_lo, t_hi = shadow_of(tested)
       t_first = crossings(home(tested), 0, first_slide)
       f["stick.test_shadow"] = fixed(TEST_SHADOW, 1)
       f["stick.test_first"] = str(t_first)
       f["stick.test_counts"] = ",".join(map(str, t_seen))
       f["stick.test_three_share"] = fixed(t_shares.get(3, 0), 2)
       f["stick.test_mean"] = fixed(t_mean, 2)
       check(abs(t_hi - t_lo - TEST_SHADOW) < 1e-9, f"tilted to {tested:.2f} degrees, the shadow is {t_hi - t_lo:.6f} cm")
       check(t_seen == [t_first, t_first + 1, t_first], f"the test slide passes {t_seen}: two, three, two")
       check(abs(t_mean - TEST_SHADOW / GAP) < 1e-3, f"the test slide averages {t_mean:.4f}, its shadow over the gap")
       near = min(off_lines(h) for h in (lo, hi, t_lo, t_hi))
       check(near >= 0.2, f"tilted and tested, the stick's ends keep {near:.3f} cm from the lines")
       f["stick.across_rule"] = fixed(travel(home(90), 0) / GAP, 0)
       f["stick.along_rule"] = fixed(travel(home(0), 0) / GAP, 0)
       turned_shadows = [travel(home(180 * (i + 0.5) / 3600), 0) for i in range(3600)]
       turned_shadow = sum(turned_shadows) / 3600
       f["stick.mean_reach_percent"] = fixed(turned_shadow / STICK * 100, 0)
       check(abs(turned_shadow - mean_reach) < 1e-6, "the stick turned half a turn averages the shadow the sheet turned does")
       check(sum(s > STICK / 2 for s in turned_shadows) / 3600 > 0.5, "the arch stays above half the stick for most directions")
   ```

   Applied as written here to a scratch copy of root's integrated
   `oracle.py` (the two anchors each occur once), the copy prints 246
   facts with 0 failed cross-checks: the 230 it printed before, every one
   unchanged, and the 16 new keys, equal to the block's own run. Four
   changed inputs fail it, as they should: a
   25° tilt (an end 0.018 cm from a line), a 45° tilt (the slide passes 2,
   3, 2), a 4.0 cm test (two whole gaps, the count never changes) and a
   5.4 cm test (an end 0.05 cm from a line). Of the names it binds in
   `main()`, `seen`, `shares`, `lo`, `hi`, `s` and `i` are assigned again
   before any later read, and the rest (`mean`, `tilted`, `tested`, `near`
   and so on) are not read later, so no later line changes. A
   `test_oracle.py` case for these controls is optional.
5. `src/main.rs`. The SPOKEN and DEFINITIONAL rows above.
6. `docs/storyboard.md`. Move the two chapter sections above in after
   "### Notes for 07", at the end of the file, add their bullets to
   "Differences from the full draft", and change the status line's
   "native chapters 01 to 07" to "01 to 09". Then regenerate the script with
   `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected.
7. `docs/pause-overrides.json`. Append the nineteen overrides under
   "Pauses" to its `overrides` list, after 07_7's (44 in all), and change
   its `_comment`'s "through chapters01–07" to "through chapters01–09".
8. Nothing in `glyphs.rs`: every string 08 and 09 set goes through the kit,
   so the probe collects it. The TeX adds `^\circ` (used by lie_robotics),
   `\div` (bayesian_search, capture_recapture, mental_math), `\tfrac`
   (bakery_algorithm, capture_recapture, epsilon_delta), and `\times`,
   `\approx`, `\quad`, `\text` as 06 and 07 set them. The Text "about 64%"
   adds "%", which 06's "about 15%" also needs. If it fails the probe,
   "64 percent" is the fallback, with the same SPOKEN row. 08's live word
   "crossing" (singular, at the count of 1) is recorded as 07's is.
9. Then Cargo fmt, Clippy and the tests, `facts-check`, the glyph probe and
   a first render of 08 and 09 for the frame risks below.

Clippy lessons from 01 to 07 are applied, though Clippy has not run. Every
`pub` item and field is read inside the crate (`Bench::dots_alpha` and
`count_alpha` by 09; `Backs::pieces` and `Bench::meets_alpha` by 08). A
range test is written `!(a..=b).contains(&y)`. Neither file calls `.rt()`
or `.lag()`. The shadow, light and stick opacity trackers are locals, not
unread fields.

## Frame risks to reread at the first render

- 08_1's tilt sweeps the stick's upper end across where the shadow label's
  backing stands (0.000 margin by the estimate). The backing and "shadow 3
  cm" appear only after the tilt, and 08_7's tilt clears the backing by
  0.27.
- The count's backing clears the stick at every tilt 08 makes by 0.38; the
  console clears the stick at every direction by 0.36 and the coast by
  0.19.
- "30°" sits in the wedge: 0.40 to 0.73 under the stick and 0.25 over the
  dashed reference.
- 08_4's "1 cm" clears the lines at rest by 0.23, and its backing clears
  the stick by 0.21. The ruler's gold points sit between the lines' left
  ends (frame −6.2) and the card's edge (−6.4): a point of radius 0.08
  centred at −6.296 leaves about 0.02 on each side. Tight.
- While the sheet slides, its lines pass behind the count's backing, the
  shadow label's backing, the console and the pieces' backings, all
  registered hidden before any word, so the words stay in front. The
  footprints stay behind the live lines.
- 08_2's and 08_7's countdowns are at sheet::THINK_AT (3.6, 2.9), where
  01 to 07 put theirs, with the sheet at rest.
- 09's panel (frame x 1.0 to 6.28) hides the right parts of the lines at
  frame 0.61 and 2.21 and runs on past the lines' right ends, so no stub of
  2.21 shows. It clears the coast by 0.16 and the line at 3.81 by 0.19,
  and no line grazes its edges (at least 0.18).
- "about 64%" stands one row over "3.82 cm", 0.05 apart by the text-size
  estimate; "half" is gone by then.
- 09_4's countdown is where the hidden count stood, (−2.6, 2.9). Its ring
  clears the line at 2.21 by 0.07, as 01 to 08's at THINK_AT do; its note
  sits over the stick lying along.
- During the half turn the stick passes 0.18 cm from the dimmed coast at
  its nearest (near 90°), as in 07.
- At 180° the stick's tracker is set to 0°, the same stick drawn the same
  way, before the half turn starts. Check that nothing jumps.
- The flattening fills under the arch in translucent purple over the
  dashed half line. Check that the half line still reads through it.
- 09_8's TeX, coin and "?" sit in the band between the lines at 2.21 and
  3.81, clear of them by at least 0.15 by the estimate.
- Text extents are estimated, not measured: size/150 frame units a
  character and size/150 half-height.

## Timing, estimated and not measured

The tool's estimates count words, a breath, the prediction pause and a
2.5 s hold: 138.1 s for 08 and 110.2 s for 09, both above the former 85 s
target, which is an advisory now, not a cap. The silent experiments add
to that. Neither chapter is split, as Mail 6881 and the repaired gate
allow. If root wants shorter scenes, 08 has a conceptual boundary between
08_6 (the rule found) and 08_7 (the rule tested), and the split needs no
change to any words. Scene seconds below are what the code plays.

- 08: the tilt 2.5 s (08_1); the countdown 4 s and the slide 8 s (08_2);
  the second slide 8 s after the line (08_4); the test tilt 2 s, the
  countdown 4 s and the slide 8 s (08_7); the stand 2.6 s (08_9).
- 09: the fades and the turn to lie along, 2.5 s (09_1); the half turn
  12.6 s after the line (09_2); the countdown 4 s (09_4); the flattening
  2.9 s (09_5); the stand 2.6 s (09_8).
- Phrase moments come from each sentence's share of characters, so with
  no narrator they are estimates. In 09_5, by that estimate, "three point
  eight two centimetres" comes about 4.9 s after "Flatten it", after the
  2.9 s flattening.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --check` (the repo's `rustfmt.toml`) on both new
  files exits 0, and on step 3's block set in a scratch function.
- The fmn and kit calls were read against the integrated chapters and the
  kit (`Kit::text`, `tex`, `hold`, `finish`), `placement.rs` (`at_once`,
  `glide`, `tracked`, `faded_line`, `halo_along`, `live_lines`,
  `footprints`, `live_count`, `Placement::make`, `settle`, `share`) and
  `draw.rs` (`think`, `stand`, `say`, `fade_to`), as they stand.
- `oracle.py` as it stood before root's 06/07 integration: 221 facts, 0
  failed cross-checks. With the integration in (13:05Z): 230 facts, 0
  failed.
- `oracle_0809_patch` (session scratch) runs block 3 above, verbatim, with
  the oracle's functions and 07's `home`: 0 failed, and the four controls
  refused.
- `apply_block3` (session scratch) applies step 4 as written here to a
  scratch copy of the integrated `oracle.py`: 246 facts, 0 failed, the 230
  old keys unchanged and the 16 new ones equal to block 3's own run.
- Python transliterations of the new Rust and of every test assertion, on
  the integrated oracle's output and block 3's keys (session scratch, not
  committed). `ch08_mirror`: 144 ok, 0 failed. `ch09_mirror`: 98 ok, 0
  failed. The same counts as on the oracle before the integration. Both counts include `sheet_mirror`'s own checks, run
  first. The INDEPENDENT parts are exact fractions for the slides' runs and
  the cut, the coset argument for the whole gap, arc-length coast samples,
  the exact average 12/π (3.8197) rounding to every number shown, the
  shown operands (3.82 ÷ 2, 1.91 × π, 3.82 ÷ 6, 2/π), a fine trapezoid of
  the arch (∫ 6|sin| = 12 over half a turn), and controls: four wrong
  facts and a 4 cm stick refused by 09's `words()`.
- `candidate0809_check` (session scratch) runs `tools/storyboard.py`'s
  own `parse` and `check` on this file, writing nothing. It also checks the
  words rule on the script these rows make, and every phrase the two scenes
  wait for (read from their sources and tests) in its line. It checks every
  SPOKEN row's value against the integrated oracle's facts and block 3's
  keys, and runs `main.rs`'s number guard (its own word list and matching
  rule) with the rows above, plus negative controls. It resolves the pause
  overrides with `narration_director.directions()` on `script.tsv` with
  these lines added, alone and with the integrated file's twenty-five, and
  compares the files' hashes with the ones below. It also checks that root's
  `facts.rs` already prints 06/07's keys and, with this step 3, every key
  the chapters read. Against the 13:10Z files: 317 ok, 0 failed.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact.

## Files handed off

| file | sha256 prefix | bytes |
|---|---|---|
| `src/chapters/the_shadow_rule.rs` | a3db91ca331bf45f | 62131 |
| `src/chapters/turning_the_stick.rs` | fa0ae74e8cbb1aac | 36091 |

This file's own hash is in the handoff mail.

## Root integration amendment 17:20 UTC

F1 reverses the wording of08_7 to match the turn. F2 puts the fill behind the arch; preserve this layering in chapter10. F3 now has an actual tent drawn under the concave sine arch. Its area is half the enclosing rectangle, and the arch lies strictly above its two sides except at endpoints/peak, so its mean is greater than half the stick. This replaces the insufficient most-of-the-turn rationale. The21 current08/09 directions above supersede the authored19; exact old/new entries are preserved in root/tend-1713/direction-supersession16.json. Earlier timing estimates and integration prose record the author handoff, not final voice timing.
