# Candidate chapters 06 and 07 (unregistered)

Written by GentleLantern for root's 06/07 assignment (Mail 6899,
2026-10-24), continuing `mx-video-program-program-87do.22.3`. Two new
files, neither registered nor compiled:

- `src/chapters/the_bet.rs`: chapter 06, `TheBet`. It rebuilds 05's end
  (the coin at rest after its roll, its rows), then the bet, the reveal, a
  chart to scale, a second placement and the plan.
- `src/chapters/straight_sticks.rs`: chapter 07, `StraightSticks`, with
  the stick's model (`ends`, `shadow`, `crossings`, `turn_counts`) that
  `facts.rs` would print the stick's facts from.

No new helper was needed. Both chapters use `placement.rs`, `sheet.rs`,
`draw.rs` and 05's meeting rule (`meetings`) as root integrated them. The
rows below are in `docs/storyboard.md`'s format, ready to move there when
root registers the scenes. `tools/storyboard.py`'s own `parse` and `check`
read this file as it stands. Nothing shared was edited: the registry,
`main.rs`, `model.rs`, `facts.rs`, `palette.rs`, `oracle.py`, the scripts,
the storyboards and chapters 01 to 05 are as root left them. Nothing here
has been compiled, rendered, narrated or listened to.

## 06_the_bet: The bet (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now a bet. Suppose every curve earns crossings at the same rate as the coin's rim, centimetre for centimetre. Then a crossing on any curve is worth pi centimetres. | Where 05 ended. The card and coast are dimmed to a quarter, and the sheet is bright at the first placement beside the sealed envelope. The coin rests at (6.28, 3.45) cm after its roll, touching the lines at 2.45 and 4.45 at one point each, with its two gold dots and "2 crossings". Its rim is laid out along the line at 2.45 with "6.28 cm" over it, and 05's two rows, "2 crossings ↔ 6.28 cm" and "1 crossing ↔ π cm ≈ 3.14 cm", sit under the coast. On "Now a bet" the coin, its dots, its count, the laid-out rim with its label and the first row fade in 0.4 s; the second row stays. On "every curve" the card and coast come back to full in 1 s. On "a crossing on any curve" an `ink` tag, "the bet", fades in to the left of that row | TeX row, label | |
| 06_2 | Our coast had seven crossings at the first placement. What length does the bet give? ⟨predict⟩ | On "seven crossings" the seven gold dots and a live counter, "7 crossings", fade in where 04's counter stood. The count is solved from the model at the placement shown, every frame (`open.crossings`). On "What length" "→ ? cm" follows the count, the ? in gold. The envelope stays shut. Countdown | live counter from the model, think countdown | |
| 06_3 | Seven times pi, about twenty-two point zero centimetres. | On "Seven times pi" the countdown's ? and "→ ? cm" give way to "→ 7 × π cm ≈ 22.0 cm" (`open.estimate`; 7π is 21.99). The envelope is still shut | TeX | |
| 06_4 | Now open the envelope. The true length is twenty-five point eight centimetres. | On "open the envelope" its flap swings up in 1 s, its point rising 0.3 over the envelope's top edge and staying under the line at 4.45. On "twenty-five point eight" "25.8 cm" appears in green beside "true length" (`coast.length`). This is the first time the true length is on screen | reveal | |
| 06_5 | Side by side and to scale, the bet comes up three point eight centimetres short. That's about fifteen percent. | On "Side by side" the envelope's body and flap and the bet's row fade. In 1.2 s "true length" and "25.8 cm" move down into a chart under the coast as its top row, while a green bar grows from the chart's left end to 25.8 cm, at 0.2 frame units a centimetre. In the next 1.2 s the tag "the bet" moves into the row under it while a gold bar grows. The gold bar is π cm for each crossing counted at the placement shown, 21.99 cm, and "22.0 cm" fades in at its end (`open.estimate`). On "three point eight centimetres short" an `ink` bracket under the gold bar runs from its end to the green bar's end, with an upright there, and "3.8 cm short" appears beside the rows (`open.short_by`). On "about fifteen percent" "about 15%" follows it (`open.error_percent`; 3.8 of 25.8 is 14.7%) | bars to scale from the model, live bar, bracket highlight | |
| 06_6 | But seven was one count, at one placement. Slide the sheet up a little, and the count is nine. Nine times pi is about twenty-eight point three, and now the bet is two point five centimetres too long. | On "But seven" the estimate, "22.0 cm", the bracket and its words fade; the bars stay. On "Slide the sheet up" the sheet slides 0.5 cm up the card in 2.5 s to a slide of 0.95 cm (`bet.nine_slide`), the dots and the counter solved every frame. The count goes from 7 to 9 with no other count on the way. The gold bar jumps with it to 28.27 cm, past the green one. On "Nine times pi" "→ 9 × π cm ≈ 28.3 cm" follows the count and "28.3 cm" the gold bar (`bet.nine_crossings`, `bet.nine_estimate`). On "two point five centimetres too long" the bracket returns, now running back from the gold bar's end to the green's, and "2.5 cm too long" appears beside the rows (`bet.nine_long_by`). The picture stands at least 2.6 s | live counter, live bar, bracket highlight | hold |
| 06_7 | Same coast, same bet. A single placement can come out short or long. | On "Same coast" the nine's words and the bracket fade, and the sheet slides back 0.5 cm in 2.5 s. The count goes from 9 to 7 with nothing between, and the gold bar falls back to 21.99 cm. On "short or long" "22.0 cm", the bracket and "3.8 cm short" return. The picture stands at least 2.6 s | live counter, live bar | hold |
| 06_8 | The plan was never one count. It was the average count, over every slide and every direction. The real question is whether pi times that average is the length. | On "never one count" the chart's words, bars and bracket fade. On "the average count" 04's "average count = ?" takes the counter's place, the ? in gold. On "whether pi times" "π × average count = 25.8 cm ?" appears under the coast, its ? in gold | TeX | |
| 06_9 | For the coast, we can't work that average out yet. So try the bet first on something much simpler, something whose average we can find. | On "So try the bet first" the question, the equation and the dots fade, and the card and coast dim to a quarter in 1 s. The sheet stays bright at the first placement. The picture stands at least 2.6 s | dim | hold |

### Notes for 06

- Every count on screen is solved from the model at the placement shown,
  every frame. The gold bar is π cm for each crossing counted, so it is
  the bet at that placement and moves when the count does. Both bars start
  at the same x and run 0.2 frame units a centimetre, so the gap between
  their ends is the difference itself (a test).
- The prediction comes before the reveal. The envelope stays shut through
  06_2 and 06_3, and nothing on screen before 06_4 contains 25.8, 25.81 or
  25.813 (a test). 06_2 can be worked out from 05's row (1 crossing ↔ π cm)
  and the count.
- What is said is what is shown, rounded the same way. The shown
  differences are differences of the shown values: 25.8 − 22.0 = 3.8 and
  28.3 − 25.8 = 2.5. 3.8 of 25.8 is 14.7%, shown as "about 15%". The facts
  hold the signed error, −15 (`open.error_percent`); the screen and voice
  carry the sign as "short". `words()` refuses the chapter if any of these
  stop holding.
- One placement is not the averaged estimator. 06_6 and 06_7 show it: the
  same coast and bet give 7 crossings, 22.0 cm, short, at one placement,
  and 9, 28.3 cm, long, at another. 06_8 then says what the plan was:
  π times the average over every slide and every direction.
- The slide up, 0.5 cm (`the_bet::NINE_BY`), lands inside 04's run of
  nine, which lasts from 0.397 to 0.994 cm of 04's slide. Read every
  1/10000 cm, it passes 7 then 9 only, and back 9 then 7. At the slid
  placement every line keeps 0.072 cm from every turning height of the
  coast (the nearest are joints) and 0.103 cm from every tangent point. The
  nine dots are at least 0.673 cm apart. 04's whole slide still reads 7, 9,
  8, 7 (a test); 06 does not slide there.
- Refusal controls in the chapter's tests, each refused by its own check:
  0.3 cm up stays at 7; 1.1 cm down passes 7, 9, 7, 8, 9; 0.6 cm up passes
  only 7 and 9 but is not the slide the facts print.
- 05's coin is rebuilt where its roll ended, from 05's own rule
  (`meetings`). It touches the lines at 2.45 and 4.45 at one point each,
  which makes 05's count of two (a test). 05's items are private, so its
  constants are copied, as 04 copies 03's.
- Pronunciation risks: "twenty-two point zero" (may come out as "point
  oh", or lose the "zero"), "twenty-five point eight", "twenty-eight point
  three", "fifteen percent" (heard as "fifty"?), "centimetre for
  centimetre", "coin's rim", "can't". Nothing has been listened to.

## 07_straight_sticks: Straight sticks (~72 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Take a straight stick, six centimetres long, and lay it straight across the lines. | Where 06 ended: the card and coast dimmed to a quarter, the sheet bright at the first placement. On "Take a straight stick" a purple stick 6 cm long fades in straight across the lines. Its centre is at (−4.5, 1.2) cm, over the left end of the dimmed beach. On "six centimetres long" "6 cm" appears in green to its right (`stick.length`) | stick, label | |
| 07_2 | It crosses three lines. At pi centimetres a crossing, the bet says nine point four. Far too long. | On "It crosses three lines" three gold dots appear where it crosses the lines at −1.55, 0.45 and 2.45 cm, solved from the stick at its direction. A live counter, "3 crossings", stands where the coast's stood (`stick.home_across`). The stick's ends lie a quarter centimetre from the lines beyond them. On "the bet says nine point four" "across 3 × π cm ≈ 9.4 cm" appears under the coast (`stick.across_estimate`) | live counter from the model, TeX | |
| 07_3 | Now lay it along the lines, between two of them. What does the bet say now? ⟨predict⟩ | On "Now lay it along" "6 cm", the dots and the counter fade first, so nothing on screen answers the question. Then the stick turns 90° about its centre in 3 s, its upper end swinging left and down. It comes to lie along the lines between those at 0.45 and 2.45 cm, 0.75 cm from the nearer. Countdown | rotation, think countdown | |
| 07_4 | Zero. It crosses no lines at all. Same stick, same sheet, and the bet says nine point four or zero. For a straight stick, the count depends on which way it points. The coin hid this, because a circle runs in every direction at once. | On "Zero" the countdown's ? goes and the counter returns, reading "0 crossings", with no dots (`stick.home_along`). On "nine point four or zero" "along 0 × π cm = 0 cm" appears under the across row (`stick.along_estimate`). The coin is not drawn again; only the voice recalls it | live counter, TeX | |
| 07_5 | What matters is how far the stick reaches across the lines. Picture its shadow, cast on a ruler standing at right angles to them, with the light running along the lines. Lying along, the stick casts no shadow at all. | On "Picture its shadow" an `ink` ruler stands upright at the card's left edge, just beyond the lines' ends. It runs from −2.0 to 4.4 cm, with a tick every centimetre from the stick's height. On "with the light running along" `dim` dashed light runs from each end of the stick along the lines to the ruler. The shadow is purple on the ruler, from the lower end's height to the upper end's, and lying along it is a single point. On "casts no shadow at all" "shadow 0 cm" appears over the ruler's top (`stick.shadow_along`) | live shadow from the model, ruler | |
| 07_6 | Now turn it slowly back across, and watch the count. | After the words and a breath, "shadow 0 cm" goes. Then, in silence, the stick turns back across in 10 s at a steady pace. Every frame the dots, the counter, the light and the shadow are solved again from the stick's direction. The count climbs 0, 1, 2, 3 (`stick.turn_counts`) as the ends' heights reach the lines at 0.45, 2.45 and −1.55 cm, 14.5°, 24.6° and 66.4° from along. The four counts show for 1.61, 1.13, 4.65 and 2.62 s, and the counter's word reads "crossing" while the count is 1. The shadow grows from a point to the whole stick | rotation, live counter, live shadow | hold |
| 07_7 | It climbed as the shadow grew. Lying across again, the shadow is the full six centimetres, and the count is three. | Lying across again: three dots and "3 crossings". On "the full six centimetres" "shadow 6 cm" appears over the ruler's top (`stick.shadow_across`). The two rows stay for 08. The picture stands at least 2.6 s | live shadow, live counter | hold |

### Notes for 07

- One tracker, the stick's direction, drives everything: its ends, its
  crossings (a line counts when it lies strictly inside the shadow), the
  count and its word, the light and the shadow, all drawn every frame
  through `straight_sticks.rs`'s model. At every hundredth of a degree of
  the turn, the crossings agree with `sheet::crossings` on the stick as a
  one-piece curve, and the count with the oracle's rule, the difference of
  the two ends' line numbers (a test). The stick is 6 cm long at every
  direction (a test).
- The zero is held back. The counter and the dots go before the stick
  turns, and come back only on "Zero".
- No count shown is a near miss. Lying across, the ends lie exactly a
  quarter centimetre from the lines; lying along, the stick is 0.75 cm from
  the nearest line. Homes closer than a fifth of a centimetre are refused,
  both by `words()` (the test's controls: ends on lines at 1.45, ends
  0.15 cm off at 1.3, lying on a line at 0.45) and by the oracle block's
  controls. At an exact end-on-a-line contact the chapter's strict rule
  gives 2 and the oracle's floor rule 3. That is why both refuse such homes
  rather than pick a rule.
- The turn runs from along (180°) back to across (90°) through the upper
  left. Turned through 0° to 90° instead, the stick's end runs through the
  counter (measured gap 0.000), which is the mirror's control. The oracle
  turns its stick from 0° to 90°. Because sin(180° − φ) = sin φ, the ends
  reach the same heights, so the counts are the same, both read every
  hundredth of a degree.
- "It climbed as the shadow grew" is tested at its edges. Along the turn
  each shadow holds every one before it, so the count only climbs.
  Turned on past across it falls again (a test, and the oracle block's
  control to 180°). 07_7 claims no more than that; 08's rule (average
  count = shadow ÷ gap) stays 08's.
- The counts change at sin φ = 0.75/3, 1.25/3 and 2.75/3. At 10 s for the
  quarter turn, every count shows for at least a second (a test).
- The shadow's length appears only at the two ends of the turn, never live:
  a rounded "shadow N cm" changing during the turn would draw the eye from
  the count.
- The two rows stay on screen at the end, as the draft's notes want: 08
  resolves both readings with one rule.
- Prediction (07_3): read off the picture, since the stick lies between two
  lines.
- Pronunciation risks: "Zero." as the line's first word, a one-word
  sentence that may come out clipped; "nine point four or zero";
  "centimetres"; "pi" (as "pie"). Nothing has been listened to.

## Differences from the full draft

- 06_1: no coin icon at the frame's edge. 05's own row "1 crossing ↔ π cm
  ≈ 3.14 cm" stays, tagged "the bet".
- 06_2 adds "at the first placement". 06_3 says "Seven times pi, about
  twenty-two point zero centimetres" where the draft says "Seven times pi.
  Twenty-two point zero centimetres." 7π is 21.99, so the voice says
  "about" and the TeX shows ≈.
- The draft's 06_4 is split into 06_4 (the reveal) and 06_5 (the chart to
  scale, short, percent). Its "3.8 cm short, −15%" becomes "3.8 cm short",
  "about 15%". The gap is an `ink` bracket rather than red, since red is the
  dividers' role, and the dividers' faint 22.6 bar is left out. It belongs
  to a different estimate, and beside 22.0 it would invite a comparison the
  chapter does not make.
- The draft's 06_5 ("So either the bet is wrong, or one placement isn't
  enough…") becomes 06_6 to 06_9. The viewer watches the same coast give 9
  crossings and 28.3 cm, then 7 and 22.0 cm again, before the plan is
  restated. The draft's callback to 04's log strip is replaced by this live
  slide.
- 07's stick is `purple` #9A72AC (the draft's violet) and turns about one
  centre all chapter. The draft centres it 0.3 cm above a line for 07_1 and
  midway between two for 07_3, which needs the stick to move. Here it lies
  0.75 cm above the line at 0.45, and 07_3's words ("between two of them")
  hold without "midway". The oracle ties the on-screen counts to its own
  placements' counts (block 2).
- 07_2's estimate is an `ink` row, not red beside "true 6 cm"; the stick's
  6 cm label is green.
- 07_4 says "the count depends on which way it points" where the draft says
  "direction is everything", which overstates: length matters too. The
  coin icon's flash is dropped; the voice alone recalls the coin.
- The draft's 07_5 introduces the shadow lying across and turns the stick
  to along. Here the stick is already along, so the shadow first appears as
  a point. "Lying across, the shadow is the full six centimetres" moves to
  07_7, after the new 07_6, where the viewer watches the count climb as the
  stick turns back. 09_1's half turn, its plot and the average stay 09's;
  07_6 shows only a quarter turn at one slide.
- Root decides whether to fold these rows back into the full draft.

## Proposed bindings

### Numbers (`main.rs`, the number guard)

These rows let every number word in 06 and 07 pass the guard (checked; see
below). SPOKEN, after 05_6's:
`("06_1", "worth pi centimetres", "coin.worth", "3.14")`,
`("06_2", "seven crossings", "open.crossings", "7")`,
`("06_3", "Seven times pi", "open.crossings", "7")`,
`("06_3", "twenty-two point zero centimetres", "open.estimate", "22.0")`,
`("06_4", "twenty-five point eight centimetres", "coast.length", "25.8")`,
`("06_5", "three point eight centimetres short", "open.short_by", "3.8")`,
`("06_5", "about fifteen percent", "open.error_percent", "-15")`,
`("06_6", "But seven", "open.crossings", "7")`,
`("06_6", "the count is nine", "bet.nine_crossings", "9")`,
`("06_6", "Nine times pi", "bet.nine_crossings", "9")`,
`("06_6", "twenty-eight point three", "bet.nine_estimate", "28.3")`,
`("06_6", "two point five centimetres too long", "bet.nine_long_by", "2.5")`,
`("07_1", "six centimetres long", "stick.length", "6")`,
`("07_2", "crosses three lines", "stick.home_across", "3")`,
`("07_2", "pi centimetres a crossing", "coin.worth", "3.14")`,
`("07_2", "nine point four", "stick.across_estimate", "9.4")`,
`("07_4", "Zero", "stick.home_along", "0")`,
`("07_4", "nine point four", "stick.across_estimate", "9.4")`,
`("07_4", "or zero", "stick.along_estimate", "0")`,
`("07_7", "the full six centimetres", "stick.shadow_across", "6")`,
`("07_7", "the count is three", "stick.home_across", "3")`.
DEFINITIONAL, after 05_5's (a choice, a unit or a definition, not a count
the facts hold): `("06_6", "one count, at one placement")`,
`("06_7", "A single placement")`, `("06_8", "never one count")`,
`("06_8", "pi times that average")`, `("07_3", "between two of them")`,
`("07_4", "every direction at once")`.

`"-15"` for "about fifteen percent" follows 01 to 05's rule: the value is
the fact's printed value, and the voice gives its size with "short".

### Pauses (text-only intentions)

These are authored overrides with the same fields as root's
`docs/pause-overrides.json` (Mail 6908), for root to append to its
`overrides` list and to extend its `_comment` to 06/07. They are editorial
hypotheses, not measured pacing, and nothing has been listened to. As that
comment says of 04/05, the countdowns, slides and turns stay protected
visual time and are not to be trimmed to these ranges. The check below
resolves each one with `tools/narration_director.py`'s own `directions()`
on the script these rows make, together with root's ten, and no two land
on the same boundary. Each names one boundary; 07_4's "Zero" occurs twice,
so its `word_index` picks the first.

```json
[
  {"video": "crofton", "cue": "06_1", "after": "Now a bet", "role": "thought", "range_s": [0.4, 0.8], "reason": "The coin and its count fade before the bet is stated."},
  {"video": "crofton", "cue": "06_1", "after": "centimetre for centimetre", "role": "thought", "range_s": [0.5, 0.9], "reason": "The supposition stands before its consequence, while the coast brightens."},
  {"video": "crofton", "cue": "06_4", "after": "Now open the envelope", "role": "thought", "range_s": [0.7, 1.2], "reason": "The flap opens (1 s) before the true length is spoken."},
  {"video": "crofton", "cue": "06_5", "after": "Side by side and to scale", "role": "clause", "range_s": [0.3, 0.65], "reason": "The two bars grow (2.4 s) before the difference is named."},
  {"video": "crofton", "cue": "06_6", "after": "at one placement", "role": "thought", "range_s": [0.5, 1.0], "reason": "The old estimate fades; the point of the chapter lands before the slide."},
  {"video": "crofton", "cue": "06_6", "after": "the count is nine", "role": "thought", "range_s": [0.4, 0.8], "reason": "The gold bar has jumped past the green one; let it register before the arithmetic."},
  {"video": "crofton", "cue": "06_7", "after": "same bet", "role": "thought", "range_s": [0.6, 1.2], "reason": "The slide back finishes and the count settles at seven before the conclusion."},
  {"video": "crofton", "cue": "06_8", "after": "every direction", "role": "thought", "range_s": [0.4, 0.9], "reason": "The plan, restated with its question on screen, settles before the real question."},
  {"video": "crofton", "cue": "07_2", "after": "It crosses three lines", "role": "thought", "range_s": [0.5, 1.0], "reason": "Time to count the three dots."},
  {"video": "crofton", "cue": "07_2", "after": "the bet says nine point four", "role": "thought", "range_s": [0.35, 0.7], "reason": "The row lands before the verdict."},
  {"video": "crofton", "cue": "07_3", "after": "between two of them", "role": "thought", "range_s": [0.4, 0.8], "reason": "The stick finishes its 3 s turn before the question."},
  {"video": "crofton", "cue": "07_4", "after": "Zero", "word_index": 0, "role": "thought", "range_s": [0.5, 1.0], "reason": "The answer: the counter reads 0 with no dots before the voice explains."},
  {"video": "crofton", "cue": "07_4", "after": "which way it points", "role": "thought", "range_s": [0.4, 0.8], "reason": "The chapter's claim stands before the callback to the coin."},
  {"video": "crofton", "cue": "07_5", "after": "running along the lines", "role": "thought", "range_s": [0.4, 0.8], "reason": "The light and the shadow's single point appear before the claim about them."},
  {"video": "crofton", "cue": "07_7", "after": "It climbed as the shadow grew", "role": "thought", "range_s": [0.4, 0.8], "reason": "The finished turn is looked at before the summary of its end."}
]
```

Visual holds between cues (the picture runs on while the voice waits).
These need render evidence before they enter a map, so here they are only
the scene's own seconds:

- after 06_2: the countdown, 4 s (`think`).
- after 06_6 and after 06_7: at least 2.6 s from the moment the result is
  whole (`stand`).
- after 06_9: the chapter's end, 2.6 s on the dimmed card.
- after 07_3: the countdown, 4 s.
- after 07_6: the silent turn back across, 0.4 s fade and 10 s, after the
  line and its breath (`Kit::hold`).
- after 07_7: the chapter's end, 2.6 s.

## Integration steps for root

Rebased against the files as they stood at 11:20Z, after root's 04/05
integration. Their sha256 prefixes: `mod.rs` 9033f6801c735c6c, `main.rs`
90c42f6e57156f8f, `facts.rs` 5facd10185e0cbf7, `palette.rs`
762853a019697b33, `oracle.py` b86b7418c93c1e51, `test_oracle.py`
d5f3a90018fb5d4e, `model.rs` ce81892a6dbdf281, `placement.rs`
bfd93f4b571a797c, `script.tsv` 7e6e2818c8248852, `docs/storyboard.md`
ec04dad708a21f58, `docs/pause-overrides.json` 0ec18afc868906a0. Root's later
04 backing repair (`one_count_one_placement.rs`, now bc7ef9925d6aaee7) is
not used by 06 or 07. Until steps 1 to 4 are in, the two chapters do not
compile (`palette::stick`, the new modules) or their `words()` refuse
(missing facts).

1. `src/chapters/mod.rs`. Declare the two modules, keeping the list in
   order: `pub mod straight_sticks;` after `pub mod sheet;`, and
   `pub mod the_bet;` before `pub mod the_question;`. Add the registry
   entries after 05. Proposed intensities are 0.5 (the reveal and the miss)
   and 0.4:
   `Chapter { key: "06_the_bet", make: |kit| Box::new(the_bet::TheBet { kit }), intensity: 0.5 }`
   and
   `Chapter { key: "07_straight_sticks", make: |kit| Box::new(straight_sticks::StraightSticks { kit }), intensity: 0.4 }`.
   The module doc's "the draft's chapters 06 to 28" becomes "08 to 28".
2. `src/palette.rs`. A role for the stick, which 07 uses as
   `palette::stick()`, and the same role added to
   `every_role_is_its_own_color`'s list:

   ```rust
   /// A straight stick (07) and its shadow.
   pub fn stick() -> Srgb {
       Palette::purple()
   }
   ```

   In `docs/storyboard.md`'s Color roles table, add the row
   `| a straight stick and its shadow (07) | purple #9A72AC |` (the hex in
   backticks), and take "violet stick, " out of the sentence under the
   table.
3. `src/facts.rs`. After the chapter 05 block and before the "film's
   identity" comment, using the `length`, `slide` and `crossings` already
   computed there:

   ```rust
   // Chapter 06: the bet, π cm a crossing, at the first placement and
   // slid up into 04's run of nine
   let worth = PI * COIN / 2.0;
   let bet = crossings.len() as f64 * worth;
   f.add("open.estimate", fixed(bet, 1))?;
   f.add("open.short_by", fixed(length - bet, 1))?;
   f.add("open.error_percent", fixed((bet - length) / length * 100.0, 0))?;
   let nine_slide = slide + crate::chapters::the_bet::NINE_BY;
   let nine = crate::chapters::sheet::crossings(&coast, 0.0, nine_slide).len();
   f.add("bet.nine_slide", fixed(nine_slide, 2))?;
   f.add("bet.nine_crossings", nine)?;
   f.add("bet.nine_estimate", fixed(nine as f64 * worth, 1))?;
   f.add("bet.nine_long_by", fixed(nine as f64 * worth - length, 1))?;

   // Chapter 07: the stick on screen, lying across and along the first
   // placement's lines, the counts its turn between the two passes, and
   // its shadow at both ends of the turn
   let home = crate::chapters::straight_sticks::HOME;
   let [lying_across, lying_along] = [
       crate::chapters::straight_sticks::ACROSS,
       crate::chapters::straight_sticks::ALONG,
   ]
   .map(|theta| crate::chapters::straight_sticks::crossings(home, theta, slide).len());
   f.add("stick.length", fixed(crate::chapters::straight_sticks::LENGTH, 0))?;
   f.add("stick.home_across", lying_across)?;
   f.add("stick.home_along", lying_along)?;
   f.add("stick.across_estimate", fixed(lying_across as f64 * worth, 1))?;
   f.add("stick.along_estimate", fixed(lying_along as f64 * worth, 0))?;
   f.add(
       "stick.turn_counts",
       crate::chapters::straight_sticks::turn_counts(home, slide)
           .iter()
           .map(usize::to_string)
           .collect::<Vec<_>>()
           .join(","),
   )?;
   for (key, theta) in [
       ("stick.shadow_across", crate::chapters::straight_sticks::ACROSS),
       ("stick.shadow_along", crate::chapters::straight_sticks::ALONG),
   ] {
       let [lo, hi] = crate::chapters::straight_sticks::shadow(home, theta);
       f.add(key, fixed(hi - lo, 0))?;
   }
   ```

   Expected values: `open.estimate` 22.0, `open.short_by` 3.8,
   `open.error_percent` -15, `bet.nine_slide` 0.95, `bet.nine_crossings` 9,
   `bet.nine_estimate` 28.3, `bet.nine_long_by` 2.5, `stick.length` 6,
   `stick.home_across` 3, `stick.home_along` 0, `stick.across_estimate`
   9.4, `stick.along_estimate` 0, `stick.turn_counts` 0,1,2,3,
   `stick.shadow_across` 6, `stick.shadow_along` 0. The oracle already
   prints `open.estimate`, `open.short_by`, `open.error_percent`,
   `stick.length`, `stick.across_estimate` and `stick.along_estimate`; the
   other nine come from step 4. The two `stick.*_estimate` keys are the
   oracle's from its own placements, and the native side's from the stick
   on screen; block 2's check ties the two counts together. The block, set
   inside a function body, passes `rustfmt --check` at the repo's width.
   None of the names it binds (`worth`, `bet`, `nine_slide`, `nine`,
   `home`, `lying_across`, `lying_along`, `lo`, `hi`) is bound elsewhere
   in `facts()`. (`along` is: 01's ruler, at the top.)
4. `oracle.py`. Three constants after `OTHER_COINS = [1.5, 3.0]`:

   ```python
   # chapter 06 slides the first sheet this far up, cm, into 04's run of nine
   NINE_BY = 0.5
   # chapter 07's stick on screen: its centre on the card, cm, and its turn
   # from along the lines to across, degrees
   STICK_HOME = (-4.5, 1.2)
   STICK_TURN = 90
   ```

   Then two blocks inside `main()`, 4-space indent. Block 1 goes after
   `f["open.short_by"] = …`:

   ```python
       # chapter 06: the sheet slid NINE_BY cm up from the first placement,
       # into 04's run of nine; the slide passes no other count (read every
       # 1/10000 cm); the bet short at the first placement, long there
       nine_slide = first_slide + NINE_BY
       nine = crossings(geom, 0, nine_slide)
       f["bet.nine_slide"] = fixed(nine_slide, 2)
       f["bet.nine_crossings"] = str(nine)
       f["bet.nine_estimate"] = fixed(nine * worth(), 1)
       f["bet.nine_long_by"] = fixed(nine * worth() - length, 1)
       passed = []
       for i in range(5001):
           n = crossings(geom, 0, first_slide + NINE_BY * i / 5000)
           if not passed or passed[-1] != n:
               passed.append(n)
       check(passed == [first, nine], f"chapter 06's slide up {NINE_BY} cm passes the counts {passed}")
       check(first * worth() < length < nine * worth(), f"the bet is short at {first} crossings and long at {nine}")
   ```

   Block 2 goes after `f["stick.along_estimate"] = …`:

   ```python
       # chapter 07 on screen: the stick centred at STICK_HOME on the sheet at
       # the first placement, turned about its centre from along the lines
       # (0 degrees) to across (STICK_TURN degrees), its count read every
       # 1/100 degree; its shadow across the lines at both ends of the turn.
       # Lying across, each end keeps a fifth of a centimetre from every line
       # (in exact fractions), so the count shown is no near miss
       def home(angle):
           return stick_geom(STICK, angle, STICK_HOME)

       f["stick.home_across"] = str(crossings(home(90), 0, first_slide))
       f["stick.home_along"] = str(crossings(home(0), 0, first_slide))
       turn_seen = []
       for i in range(100 * STICK_TURN + 1):
           n = crossings(home(i / 100), 0, first_slide)
           if not turn_seen or turn_seen[-1] != n:
               turn_seen.append(n)
       f["stick.turn_counts"] = ",".join(map(str, turn_seen))
       f["stick.shadow_across"] = fixed(travel(home(90), 0), 0)
       f["stick.shadow_along"] = fixed(travel(home(0), 0), 0)
       y0 = Fraction(STICK_HOME[1]).limit_denominator(1000)
       s0, gap = Fraction(first_slide).limit_denominator(1000), Fraction(GAP)
       off = min(min((e - s0) % gap, gap - (e - s0) % gap) for e in (y0 - Fraction(STICK) / 2, y0 + Fraction(STICK) / 2))
       check(off >= Fraction(1, 5), f"lying across, the stick's ends keep {off} cm from the lines, at least a fifth")
       check(f["stick.home_across"] == f["stick.across"] and f["stick.home_along"] == f["stick.along"],
             "the stick on screen crosses as many lines as the oracle's own placements")
       check(turn_seen == sorted(set(turn_seen)), f"turned from along to {STICK_TURN} degrees, the count only climbs: {turn_seen}")
   ```

   Run with the oracle's own functions (session scratch), both blocks pass
   on the current `oracle.py` and print the expected values above. Five
   changed inputs fail them, as they should: a 1.1 cm slide (passes 7, 9,
   8, 7), a 0.3 cm slide (stays at 7), a home with ends on lines (1.45), a
   home with ends 0.15 cm off (1.3), and a turn on to 180° (0, 1, 2, 3, 2,
   1, 0). Block 2's list is `turn_seen`, not `turned`, because `main()`
   assigns `turned` a few lines later. A `test_oracle.py` case for these
   controls is optional.
5. `src/main.rs`. The SPOKEN and DEFINITIONAL rows above.
6. `docs/storyboard.md`. Move the two chapter sections above in after 05's
   notes, add their bullets to "Differences from the full draft", and
   update the status paragraph. Then regenerate the script with
   `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected.
7. `docs/pause-overrides.json`. Append the fifteen overrides under
   "Pauses" to its `overrides` list, after 05_6's, and extend its
   `_comment` from 04/05 to 04 to 07.
8. Nothing in `glyphs.rs`: every string 06 and 07 set goes through the kit
   or `glyphs::record`, so the probe collects it. 06's TeX adds `\times`,
   `\rightarrow` with numbers, and `\approx` beside them; 05's `\leftrightarrow` rows are
   re-set as 05 set them. 07's TeX adds `\quad` (used by
   bakery_algorithm and bayesian_search). The Text "about 15%" adds "%",
   which other topics' kit Text has set and root's glyph probes passed
   (german-glyph06 and parrondo-glyph12, empty `failures.txt`). It has not
   been probed in crofton. If it fails, "15 percent" is the fallback, with
   the same SPOKEN row. 07's counter word "crossing" (singular) is recorded
   by `live_noun` as it is drawn.
9. Then Cargo fmt, Clippy and the tests, `facts-check`, the glyph probe and
   a first render of 06 and 07 for the frame risks below.

Clippy lessons from 01 to 05 are applied, though Clippy has not run. The
two all-constant asserts in 07's layout test are in a `const { }` block.
`turning_angles`, used only by tests, is `#[cfg(test)]`. Neither file calls
`.rt()` or `.lag()` (the `Timed` import placement.rs needed).

## Frame risks to reread at the first render

- 06's chart sits in the band under the coast. By the text-size estimate
  its top row's text clears the coast's lowest point (the bay, frame
  (1.74, −1.36)) by 0.12, and "25.8 cm" sits right under that point.
  During the slide up, the sheet's lowest line rises from −2.59 to −2.19
  and passes 0.15 under the bracket. Both are tight.
- 06's gold bar jumps when the count changes, untweened, so it can be
  read as the bet following the count. Check that it reads as a jump, not a
  glitch.
- At the slid placement the line at frame 2.61 runs 0.19 under the count
  row, where "→ 9 × π cm ≈ 28.3 cm" stands, and the top line leaves the
  card. The test asserts the margin; reread it.
- 06_5 moves "true length" (scaled by 18/20) and "25.8 cm" from the
  envelope into the chart with `animate().scale().move_to()`. The commands
  apply in order to a copy, so the edges land where the test assumes; check
  that the text does not jump at the end. On the way down, over 1.2 s, the
  two texts cross the sheet's lines at 2.21, 0.61 and −0.99 and the dimmed
  coast. This is the collision class root repaired in 04 with opaque
  backings (Mail 6908), but here it is passing motion: the texts are never
  at rest on a line. A sampling layout lint may flag those frames. If root
  wants none, fading the two out at the envelope and in at the chart is the
  fallback, at the cost of the moving identity.
- Elsewhere in 06 the sheet only slides straight up and back, 0.5 cm. At
  every moment of the slide the count row (frame y 2.8 to 3.2) stays clear
  of the lines, which run from 2.21 to 2.61 under it and from 3.81 to 4.21
  over it. The chart stays clear by 0.15 (a test), and the envelope has gone
  before the slide. So 06 needs no backings by this estimate; the first
  render decides.
- The flap is redrawn each frame by an updater (`set_points`). Its point
  rises to frame 3.66, under the line at 3.81.
- The arrow and the estimates are placed after an unseen probe text,
  "crossings" at 30 points, where the live word is drawn. It is never added
  to the scene, so the layout lint does not see it (`Arena::roots` is scene
  membership).
- 07's light from the stick's lower end (card y −1.8) runs left across the
  dimmed cove to the ruler.
- The ruler stands in the 0.13 cm strip between the lines' left ends and
  the card's edge. Its ticks end where the lines end, at heights at least
  0.2 cm from any line.
- During the turn the stick passes 0.18 cm from the dimmed coast at its
  nearest (near 90.2°), and its ends keep at least 0.34 frame units from
  the counter.
- The "1 crossing" phase lasts 1.13 s, and the word changes with the
  count. Check that it does not flicker.
- 07's rows sit in 05's band, at frame y −1.78 and −2.22, between the
  coast's lowest point and the line at −2.59. These are tight, as 05's
  were.
- Text extents are estimated, not measured: size/150 frame units a
  character and size/150 half-height.

## Timing, estimated and not measured

The tool's estimates count words, a breath, the prediction pause and a
2.5 s hold. Scene seconds below are what the code plays.

- 06: the coin's fade 0.4 s and the brighten 1 s (06_1); the countdown 4 s
  (06_2); the flap 1 s (06_4); the chart 2.4 s, then the bracket and words
  0.8 s (06_5); the slide up 2.5 s and the stand 2.6 s (06_6); the slide
  back 2.5 s and the stand 2.6 s (06_7); the dim 1 s and the stand 2.6 s
  (06_9).
- 07: the turn to along 3 s and the countdown 4 s (07_3); the silent turn
  back 10 s after a 0.4 s fade (07_6); the stand 2.6 s (07_7).
- Phrase moments come from each sentence's share of characters, so with
  no narrator they are estimates. In 06_6 the slide up (2.5 s) starts on
  "Slide the sheet up"; by that estimate the count turns to 9 about 1.7 s
  in, before "the count is nine" (about 2.4 s). In 06_5 the chart (2.4 s)
  has to fit before "three point eight centimetres short", about 2.9 s
  after "Side by side"; if speech runs ahead, the bracket waits for the
  bars.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --check` (the repo's `rustfmt.toml`) on both new
  files exits 0.
- The fmn and kit calls were read against the pinned source (a399d626) and
  the integrated chapters. `AnimBuilder::set_anim_args`, `scale` and
  `move_to` return `Result<_, AnimateError>`, which `?` converts to
  `fmn::Error` (`fmn/src/lib.rs:1391`). `Mob` is `Copy` and `PartialEq`;
  `Pose`, `Move` and `Placement` are `Copy`. `Placement::make`,
  `placement::runs`, `lines_at`, `live_count`, `live_word`, `envelope` and
  `at_once` have the signatures used, after root's edits to
  `placement.rs`. A bare `say(…)?;` is the pattern 03 and 04 use.
- `oracle.py` as it stands: 221 facts, 0 failed cross-checks.
  `test_oracle.py`: 48 tests OK.
- `oracle_0607_patch` (session scratch) runs the two blocks above,
  verbatim, with the current oracle's functions: 0 failed, and the five
  controls refused.
- Python transliterations of the new Rust and of every test assertion, on
  that oracle's output and the blocks' keys (session scratch, not
  committed). `ch06_mirror`: 86 ok, 0 failed. `ch07_mirror`: 82 ok, 0
  failed. Both counts include `sheet_mirror`'s own checks, run first. The
  INDEPENDENT parts are the oracle's own counts and slides, exact fractions
  for the stick's end clearances, arc-length coast samples, the displayed
  differences (25.8 − 22.0 = 3.8, 28.3 − 25.8 = 2.5, 3.8 / 25.8 = 14.73%)
  and the 0° to 90° counter control.
- `candidate0607_check` (session scratch) runs `tools/storyboard.py`'s
  own `parse` and `check` on this file, writing nothing. It also checks the
  words rule on the script these rows make, and every phrase the two scenes
  wait for (read from their sources and tests) in its line. It checks every
  SPOKEN row's value against the oracle's facts and the blocks' keys, and
  runs `main.rs`'s number guard (its own word list and matching rule) with
  the rows above, plus negative controls. It resolves the pause overrides
  with `narration_director.directions()`, alone and with root's ten from
  `docs/pause-overrides.json`, and compares the two files' hashes with the
  ones below. It also checks that `facts.rs` with step 3 supplies every key
  the chapters read. Results are in the handoff mail.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact.

## Files handed off

| file | sha256 prefix | bytes |
|---|---|---|
| `src/chapters/the_bet.rs` | 9dd1468f5a90bbf3 | 41996 |
| `src/chapters/straight_sticks.rs` | 85e6429ca80fb74d | 33244 |

This file's own hash is in the handoff mail.
