# Candidate chapters 02 and 03 (unregistered)

Written by GentleLantern for root's 02/03 assignment (Mail 6861,
2026-10-24), Bead `mx-video-program-program-87do.22.1`. Three new files,
none registered, none compiled:

- `src/chapters/sheet.rs`: the one helper both chapters need. The shown
  placement (direction 0, slide 0.45 cm), the crossings solved from the
  model in walking order, the redrawn beach (`rippled`), the drawn sheet,
  and the places the two chapters share.
- `src/chapters/a_sheet_of_lines.rs`: chapter 02, `ASheetOfLines`.
- `src/chapters/an_exchange_rate.rs`: chapter 03, `AnExchangeRate`.

The rows below are in `docs/storyboard.md`'s format, ready to move there
when root registers the scenes. `tools/storyboard.py`'s own `parse` and
`check` read this file as it stands (results in "Checks run" below). The
registry, `main.rs`, `model.rs`, `facts.rs`, `oracle.py`, the scripts, the
storyboard and the full draft are unchanged. Nothing here has been
compiled, rendered, narrated or listened to.

## 02_a_sheet_of_lines: A sheet of lines (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | A string does better. Press it into every bend, then pull it straight and measure the string. That works, but it's slow and fiddly, and a real map has thousands of bends. | Where 01 ended, the card and the coast. From "Press it" a thin `ink` string is pressed along the coast piece by piece, the cove, the beach, the climb and the rise, pausing a quarter second at each joint, and stops halfway round the headland. On "slow and fiddly" it fades. It is never straightened, so no length is shown | string traced by arc length from a tracker, stops at the model's joints | |
| 02_2 | Ruler, dividers and string all follow the coast, bend by bend. Is there a way to measure it without following it at all? | Three small icons in the band over the coast, a ruler, red dividers' legs and a loose string, each named under it in `dim`. On "follow the coast" small `ink` beads appear one after another along the whole coast, one every half centimetre from its start, over 2.4 s. On "without following it at all" the icons, names and beads fade | lagged group reveal | |
| 02_3 | Here's a stranger idea. Take a clear sheet, ruled with straight lines two centimetres apart, and lay it over the card. | On "Take a clear sheet" a clear panel the card's size, five `dim` lines across it, slides in from beyond the right edge over 1.6 s and settles on the card, its lines at −3.55, −1.55, 0.45, 2.45 and 4.45 cm (`open.slide`). On "two centimetres apart" an `ink` bracket between the two lowest lines near the card's right edge, "2 cm" beside it (`sheet.gap`) | sheet lines from the shown slide, slide-in | |
| 02_4 | Wherever the coast meets a line, call that a crossing. Here's one, in the cove. And here's another, where the coast climbs away from the beach. | The bracket fades. On "call that a crossing" the definition "crossing: where the coast meets a line" in the band over the coast, "crossing" in gold. On "Here's one" the first crossing the coast meets, in the cove on the lower line, gets a gold dot and a gold "1" beside it, and the dot pulses once. On "here's another" the second, on the climb, "2" (`open.crossing_pieces`). Only those two are marked | crossings solved from the model, highlight | |
| 02_5 | Count them all, and take as long as you like. How many crossings are there? ⟨predict⟩ | On "How many crossings" the definition gives way to "How many crossings?" in the same place. The two dots stay and nothing else is marked. Countdown in the band's right half | think countdown | |
| 02_6 | Seven. Here they are, in the order the coast meets them. | The question and the "?" give way to "7 crossings" in gold (`open.crossings`). From "Here they are" the other five crossings light one at a time in walking order, each a gold dot and its number, 3 to 7, each with its own pulse and 0.75 s to itself | lagged highlights in walking order | |
| 02_7 | Only two of the lines meet the coast at all. The lower one crosses it four times, in the cove, on the climb, and twice round the bay. The upper one crosses it three times, either side of the headland, and once near the end. | The numbers fade. On "Only two of the lines" the three lines that meet nothing dim. On "The lower one" the line at −1.55 cm thickens in `ink`; on "four times" a gold "4" past its right end (`open.per_line`); its dots pulse as each place is named, the cove, the climb, then the bay's two together. On "The upper one" the lower line returns to `dim` and the line at 0.45 cm thickens, "3"; the rise and the fall pulse together, then the end. Then every line returns and the tallies fade, leaving the card, the coast, the sheet, the seven dots and "7 crossings" | highlights, tallies from the model | hold |

### Notes for 02

- Root's direction, as built. The same coast, card and solved crossings. A
  crossing is defined in words and on two real examples before the viewer
  counts. The count is the viewer's, because nothing new is marked during
  02_5. All seven light only in 02_6, and the true length stays sealed.
- Identity is kept through the sheet's placement and the count. The sheet
  is built once at its shown place, shifted off the frame and slid back.
  Its lines are picked out by restroking and dimmed by stroke opacity
  only, so the lines never fill in (`dim_line`, fmn pin a399d626). Dots
  keep their place and their numbers stay in walking order.
- Each indicated crossing has time to itself: one phrase each in 02_4,
  0.75 s each in 02_6, and a pulse as each place is named in 02_7.
- `sheet.rs` checks the seven crossings in walking order. It runs 8
  directions × 10 slides against the model's `meetings` and against sign
  changes along the drawn polyline. It also checks the per-line tally of 4
  and 3. `a_sheet_of_lines.rs` refuses to build if the crossings stop
  being the ones its words name, and its test shows the refusal at
  slide 1.05.
- Semantic pauses: the countdown after 02_5's question, and the hold after
  02_7, where the counted sheet stands before 03 changes the subject.
- The string in 02_1 is never straightened, to keep the true length sealed.
- Pronunciation: no names. "Fiddly", "headland" and "crossing" are
  ordinary words, unverified until qualification.

## 03_an_exchange_rate: An exchange rate (~81 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Seven crossings, and we never followed the coast. But seven is a count, not a length. | Where 02 ended. On "a count, not a length" "→ ? cm" appears beside "7 crossings", the ? in gold | TeX | |
| 03_2 | Still, crossings ought to know something about length. Swap the straight beach for one big half circle. The coast gets about two centimetres longer, and it crosses the lower line twice more. | On "Swap the straight beach" the beach dims and a blue half circle of radius 1.75 cm, standing on the beach as its diameter and bulging up, draws itself from the beach's start to its end in 1 s. On "about two centimetres longer" "≈ 2 cm longer" in TeX under the coast. On "twice more" two new gold dots appear where the half circle crosses the line at −1.55 cm, with "9 crossings" in gold beside the TeX (`swap.big_more`, `swap.big_crossings`) | computed illustration, crossings solved from the model | |
| 03_3 | Now swap in seven small half circles instead. That adds just as much length, but here they cross no line at all. So at one placement, more length doesn't have to mean more crossings. The hope is that it does on average, over many placements. | On "seven small half circles" the big half circle, its dots and "9 crossings" fade, and seven half circles of radius 0.25 cm draw along the beach in 1 s; their tops stay 0.18 cm under the line at −1.55 cm. On "just as much length", under the TeX in `dim`, "precisely: both 1.998 cm longer" (`swap.extra_3dp`). On "no line at all" "7 crossings" in gold beside the TeX. On "The hope is" the half circles and the words fade, the straight beach returns, and "hope: on average, more length means more crossings" appears in gold under the coast | computed illustration, precisely card | |
| 03_4 | How many centimetres, then, is one crossing worth? The lines are two centimetres apart. Is each crossing worth two centimetres? ⟨predict⟩ | The hope fades. On "two centimetres apart" 02_3's bracket and "2 cm" return. On "Is each crossing worth" "1 crossing = 2 cm ?" in gold under the coast. Countdown | think countdown | |
| 03_5 | It can't be. Seven times two is fourteen centimetres, and the two ends of the coast are already fourteen point seven apart. No curve joining them can be shorter than that. | The bracket and the "?" go. On "Seven times two" the guess gives way to "7 × 2 cm = 14 cm" in red (`open.gap_guess`). On "fourteen centimetres" a thick red bar 14 cm long grows in 1 s from the coast's start, along the straight line toward its end. On "the two ends of the coast" 01_2's dashed `dim` segment joins the ends, with "14.7 cm end to end" (`coast.ends_apart`); the bar stops 0.73 cm short of the end. On "No curve joining them" the whole coast thickens and settles back | bars to scale (the bar against the end-to-end segment), TeX | |
| 03_6 | The exchange rate has to come from somewhere else, from a curve whose length we already know. | The sum, the bar, the segment and "14.7 cm end to end" fade; "7 crossings → ? cm" stays | | |
| 03_7 | Since I built this coast myself, from four straight runs and seven pieces of circles, I know its true length exactly. It goes in this envelope until counting has had its chance. | On "four straight runs" the four runs thicken (`coast.straight_runs`). On "seven pieces of circles" the runs settle and the seven arcs thicken (`coast.arcs`), and on "I know its true length" the arcs settle. The coast keeps its colour throughout. On "this envelope" a green envelope with its flap and "true length" under it appears in the band over the coast at the card's left; no number is shown. Then the card, the coast, the sheet's lines, the dots and the count dim, and the title "Measure a Curve Without Following It" appears over them while the envelope stays bright | title card | hold |

### Notes for 03

- 03_2 and 03_3 are the computed illustration root asked for. They use the
  same placement and the same lines. The beach redrawn as one big half
  circle adds two crossings, both on the lower line, and as seven small
  half circles it adds none. Both add the same length, (π/2 − 1) × 3.5 =
  1.998 cm, exactly. So more length alone is not a more-crossings theorem.
  The averaged statement is only stated, as a hope, and the draft's
  chapters 10 and 12 earn it. 03_3's "at one placement" leads straight
  into the draft's 04, one count, one placement.
- Placement matters, not just length. Slid 0.17 cm, so that a line runs
  through the small half circles, the seven add 14 crossings. The film's
  later sampling and averaging start from that. It is not on screen; the
  refusal test uses that slide.
- Over every slide at direction 0, both redrawn beaches gain exactly
  1.75 crossings on average (the travel gained, 3.5 cm, ÷ 2 cm). That is a
  `sheet.rs` test, not a claim on screen.
- The prediction in 03_4 can be answered from the picture. The coast joins
  ends 14.7 cm apart, so 7 × 2 cm = 14 cm cannot be its length. The bar in
  03_5 shows it: 14 < 14.73, short by 0.73 cm.
- 03_5 says "No curve joining them", where the draft says "No line": every
  curve joining the ends is at least as long as the straight segment.
- The true length stays sealed. No word on screen contains 25.8, 25.81 or
  25.813, nor the redrawn coasts' 27.8, 27.81 or 27.811, which less 2
  would give it away. Tests check both.
- The envelope stays shut until the draft's 06_4. Later chapters must
  place it again at the same spot (`ENVELOPE_AT`, frame (−5.6, 3.05));
  `envelope()` is private to chapter 03 until chapter 04 needs it.
- Semantic pauses: the countdown after 03_4, and the hold after 03_7 (the
  envelope, then the title).
- Pronunciation risks for qualification: "envelope" (EN-vuh-lohp or
  ON-vuh-lohp; choose one and lock it), "half circle(s)", and "fourteen
  point seven", which 01_2 shows but does not say. Nothing has been
  listened to.

## Differences from the full draft

- 02_2 is reworded so that it says no number ("Ruler, dividers and string
  … bend by bend"). The picture follows the coast with beads instead of
  three dotted paths.
- 02_4 says "meets a line", the same words as its definition card.
- The draft's 02_6 is split into 02_6 (seven, revealed in walking order)
  and 02_7 (the per-line tally), so that each crossing has time to itself.
  This gives seven beats instead of six. There are no colons in 02_7's
  words.
- Draft 03_2 (an extra wiggle, more dots) is replaced by the computed pair
  03_2 and 03_3. Draft 03_3 to 03_6 become 03_4 to 03_7.
- 03_5 says "curve" where the draft says "line".
- In 03_7 the pieces thicken; they do not change colour, because the coast
  never changes colour. The envelope sits in the band over the coast at
  the card's left, not in the frame's corner.
- The full draft (`docs/storyboard-draft.md`, `docs/script-draft.tsv`) is
  unchanged. Root decides whether to fold these rows back into it.

## Integration steps for root

1. `src/chapters/mod.rs`: add `pub mod sheet;`, `pub mod a_sheet_of_lines;`
   and `pub mod an_exchange_rate;`, then the registry entries after 01.
   Proposed intensities are 0.4 and 0.5:
   `Chapter { key: "02_a_sheet_of_lines", make: |kit| Box::new(a_sheet_of_lines::ASheetOfLines { kit }), intensity: 0.4 }`
   and
   `Chapter { key: "03_an_exchange_rate", make: |kit| Box::new(an_exchange_rate::AnExchangeRate { kit }), intensity: 0.5 }`.
2. `docs/storyboard.md`: move the two chapter sections above in after
   01's notes and update its status paragraph. Then regenerate the script
   with `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then, each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected.
3. `src/main.rs`, the number guard. These rows let every number word in
   02 and 03 pass (checked; see below). SPOKEN:
   `("02_3", "two centimetres apart", "sheet.gap", "2")`,
   `("02_6", "Seven", "open.crossings", "7")`,
   `("02_7", "Only two of the lines", "open.lines_crossed", "2")`,
   `("02_7", "four times", "open.lower_crossings", "4")`,
   `("02_7", "twice round the bay", "open.bay_crossings", "2")`,
   `("02_7", "three times", "open.upper_crossings", "3")`,
   `("02_7", "once near the end", "open.end_crossings", "1")`,
   `("03_1", "Seven crossings", "open.crossings", "7")`,
   `("03_1", "seven is a count", "open.crossings", "7")`,
   `("03_2", "about two centimetres longer", "swap.extra", "2")`,
   `("03_2", "twice more", "swap.big_more", "2")`,
   `("03_3", "seven small half circles", "scale.ripples", "7")`,
   `("03_4", "two centimetres apart", "sheet.gap", "2")`,
   `("03_4", "worth two centimetres", "sheet.gap", "2")`,
   `("03_5", "Seven times", "open.crossings", "7")`,
   `("03_5", "times two", "sheet.gap", "2")`,
   `("03_5", "fourteen centimetres", "open.gap_guess", "14")`,
   `("03_5", "fourteen point seven apart", "coast.ends_apart", "14.7")`,
   `("03_7", "four straight runs", "coast.straight_runs", "4")`,
   `("03_7", "seven pieces of circles", "coast.arcs", "7")`.
   DEFINITIONAL (a pointer, a pronoun or a unit, not a count the facts
   hold): `("02_4", "Here's one")`, `("02_7", "The lower one")`,
   `("02_7", "The upper one")`, `("03_2", "one big half circle")`,
   `("03_3", "at one placement")`, `("03_4", "is one crossing worth")`,
   `("03_5", "the two ends")`.
4. `src/facts.rs` and `oracle.py`. The oracle already prints these keys:
   `open.slide` 0.45, `open.crossings` 7, `open.lines_crossed` 2,
   `open.per_line` 4,3, `open.crossing_pieces`, `open.crossing_lines`,
   `open.gap_guess` 14 and `scale.ripples` 7. `facts.rs` has none of them
   yet; port them from `sheet.rs`, which computes each one from the model.
   New keys, in both, for the SPOKEN rows and the 03_3 card:
   `open.lower_crossings` 4, `open.upper_crossings` 3,
   `open.bay_crossings` 2, `open.end_crossings` 1 (from the walking
   order); and `swap.big_crossings` 9, `swap.big_more` 2,
   `swap.small_crossings` 7, `swap.small_more` 0, `swap.extra` 2 and
   `swap.extra_3dp` 1.998. In the oracle, the big half circle is
   `rippled_coast(1)` and the small ones are `rippled_coast(7)`; its own
   `crossings` already gives 9 and 7 at the shown slide, and
   `exact_length` gives 1.997787144 more for both (checked below). The
   chapters do not read these keys; they compute the same values from the
   model and refuse to build if they differ from what their words say.
5. `src/chapters/glyphs.rs`: chapter 03 now typesets the title, so its
   comment ("which has no scene yet") goes stale. `TITLE` can stay in
   `TEXT` for the thumbnail.
6. Then Cargo fmt, Clippy and the tests, and a first render of 02 and 03
   for the frame risks below.

## Frame risks to reread at the first render

- The sheet's top line, at 4.45 cm, lies 0.05 cm inside the card's top
  edge and close to its rounded corner.
- The think note "pause for more time" sits at frame y 1.98, just under
  the line at 2.21; check that it does not read as hanging from the line.
- Text extents are estimated, not measured. Check these: the row
  "≈ 2 cm longer · 9 crossings" and the precisely row under the beach (the
  test's margins assume half-heights of 0.15 and 0.1 frame units), the
  hope text (about 7.7 units wide at size 26) and the title (about 10.5
  units at size 50) against the card's 12.8.
- The seven small half circles' tops are 0.18 cm under the lower line,
  0.146 frame units or about 39 px at 2160p. They must read as clearly not
  touching it.
- In 03 the coast is drawn as eleven piece paths, where 02 used one path.
  Every joint is tangent-continuous, but check the 02 → 03 cut for a seam
  or a change of brightness.
- In 03_5 the red bar (width 6) is brought in front of the dashed
  segment. The 0.73 cm shortfall is 0.58 frame units of dashes past the
  bar's tip.
- In 03_7 the card's fill is set to 0.25 from 0.55 (`set_opacity` is
  absolute), and the sheet's panel, at 0.06, is left as it is.

## Timing, estimated and not measured

Phrase moments come from each sentence's share of characters. There is no
narrator, so these are estimates.

- 02_1: the string's pressing is fitted between "Press it" and "slow and
  fiddly", with a 2 s minimum.
- 02_6: the five reveals take 3.75 s from "Here they are", against about
  4.6 s for the whole line.
- 03_5: the bar grows for 1.1 s after "fourteen centimetres", against
  roughly 1.2 s to "the two ends". If speech runs ahead, the segment comes
  in as soon as the bar is done.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --config-path rustfmt.toml --check` on the three
  new files exits 0.
- Every new fmn and kit call was read against the pinned source
  (a399d626): `Stage` derefs to the mobject stage for `bring_to_front(Mob)`
  (`fmn-mobject/src/stage.rs:941`), `next_to` (`positional.rs:1072`),
  `add_value_tracker`, `AnimBuilder::set_stroke`, and `Kit::tex` with
  `t2c`.
- Python transliterations of `sheet.rs` and each chapter's model code and
  test assertions (session scratch, not committed):
  - `sheet_mirror`: 44 ok, 0 failed. It includes the oracle's own shown
    slide, clearance and ripple counts.
  - `ch02_mirror`: 54 ok, 0 failed. This covers number placement, the
    bracket, the tallies and the refusal.
  - `ch03_mirror`: 47 ok, 0 failed (after sheet_mirror's own checks
    pass). It checks the words, the sealed lengths, the redrawn beaches
    and the refusal at slide 0.1676 (21 crossings). It
    counts three ways, analytic, sampled polyline and the oracle's own
    `rippled_coast` (turns on the spot), giving 7, 9 and 7 for the coast
    and the two redrawn beaches. It also checks the bar's 0.7305 cm
    shortfall and the layout.
- `tools/storyboard.py`'s own `parse` and `check`, imported and nothing
  written, report no problems for this file: 02 at 73.3 s and 03 at
  80.7 s by the tool's estimate (cap 85), with a ⟨predict⟩ in each. On the
  script these rows make, the candidate check has 108 ok and 0 failed.
  It covers every phrase the two scenes wait for, with 02_7's places in
  walking order, the words rule, every SPOKEN row's value against the
  oracle's facts (or, for the new keys, values computed from them and
  from ch03_mirror), and `main.rs`'s number guard: its own word list read
  from the file and its matching rule, line for line. That guard leaves
  no loose number word. Two negative controls show it catching the words
  when the rows are missing.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact. The independent parts are the oracle's construction and counts
  and the sampled count.
