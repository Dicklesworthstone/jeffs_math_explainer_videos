# Storyboard — Measure a Curve Without Following It

The hand-edited source of this video's words and pictures, for the chapters
that have a real native scene. `python3 -I tools/storyboard.py crofton`
checks it (gate G2) and writes `script.tsv` from the words column; the Rust
tests check that the chapter registry and `script.tsv` follow it. Every
number below is a fact printed by `oracle.py` (its key in `code` where it is
not obvious); the Rust `src/facts.rs` prints the same keys and values,
computed another way (`src/model.rs`).

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it. Nothing that answers a question
is on screen before its answer beat starts.

**Status (2026-10-26): all 28 native chapters implemented; independent review repairs and final audiovisual qualification remain in progress.** Initial draft written by GentleLantern (Bead
`mx-video-program-program-87do.22.1`). The whole film, chapters 01 to 28
with every spoken line, picture, prediction and pause intention, is the full
draft in `docs/storyboard-draft.md`. Root's rule (Mail 6790): this file, the
active `script.tsv` and the production registry hold only chapters with a
real native scene, so a chapter moves here from the draft when its scene is
assigned and written. Chapter 01's scene is `src/chapters/the_question.rs`.
Opening qualification is recorded in the third-wave README; new chapters require their own native/frame qualification. No narration or listening acceptance is implied. The
tool's estimate counts words; it is not measured narration.

## The film, in one paragraph

One example runs through the whole film: **the coast**, a blue line drawn on
a 16 × 9 cm card from four straight runs and seven circular arcs (a cove, a
long beach, a climb, a knobbly headland, a round bay, a point, a short run
and a hook), exactly 15/2 + 1399π/240 = 25.81 cm long
(`coast.length_exact`). Chapter 01 asks how long it is and shows why the
obvious tools are awkward: a ruler can't bend, and 2 cm dividers always come
out short (22.6 cm). The draft's later chapters lay a clear **sheet** ruled
with lines 2 cm apart over the card, count crossings, calibrate one crossing
with a coin as wide as the gap (worth π × gap ÷ 2 = π cm), find the bias of
a single placement with straight sticks, average over slides and turns, earn
Crofton's formula, and then separate the exact identity from finite random
estimates and from the drawing's scale.

## Vocabulary (one word per idea)

Film-wide; chapter 01 uses **coast**, **card**, **ruler**, **dividers**,
**step** and **length**. **coast** (the drawn curve; "coastline" only in
01_1 and the draft's chapters 26 and 27); **card**; **length** (of the line
itself, every bend included; never the distance between the ends);
**dividers** (two points a fixed **step** apart, walked along the coast,
each step landing back on it); **sheet**, **line**, **gap**, **crossing**,
**count**, **placement**, **slide**, **turn**, **average**, **worth**,
**coin**, **stick**, **shadow**, **drop**, **chart**, **identity** against
**estimate**: defined where the draft first uses them.

## Color roles

| Idea | Color |
|---|---|
| the coast | blue `#58C4DD` |
| the coin of known length, its rim laid out | teal `#5CD0B3` |
| a straight stick and its shadow (07) | purple `#9A72AC` |
| the card | `faint` panel |
| the true length (the tracer in 01_2) | green `#83C167` |
| the dividers, their landings and their chords | red `#FC6255` |
| a point where a straight edge meets the coast (01_3), crossings later | gold `#F0AC5F` |
| the ruler, labels, TeX | `ink`, `dim` |
| the think ring and "?" | `hero` `#FFD866` |

The coast never changes color. The draft's later roles ( the sheet's `dim` lines) are in `docs/storyboard-draft.md`.

## Geometry, fixed for every chapter

- Card: 16 × 9 cm, centred at `model::CARD_CENTRE`; one card centimetre is
  `model::CARD_UNIT` frame units.
- The coast starts at (−7.3, −0.5) heading −75°, pieces as `model::COAST`;
  it stays at least 0.25 cm inside the card (reaching |x| 7.30, |y| 2.02),
  never crosses itself, and its ends are 14.7 cm apart.
- The beach is a straight 3.5 cm run at height −1.98 cm. The headland is a
  195° arc of radius 0.75 cm; the line tangent to it at the middle of its
  arc has the whole coast on one side, touching only there.
- Dividers walk by Richardson's rule: each step lands where the coast,
  followed onward from the last landing, first comes exactly one step away
  in a straight line. At 2 cm: 11 steps and a last straight bit of 0.64 cm,
  22.6 cm in all. Step 6 leaves the headland and lands on the fall, cutting
  straight across 3.08 cm of coast.

## Sources and scope

The statements the video makes, where each was checked, and what was not
read or watched, are in `brief.md`. Chapter 01 claims only that a straight
step is never longer than the stretch of coast it cuts across, so dividers
can never come out long, and that a smaller step cuts less off each bend
it crosses. It does not yet say how short, and it does not say that a
smaller step always walks further: on this coast 1.9 cm dividers walk
21.6 cm, less than the 2 cm walk's 22.6 (`dividers.1_9.length`; the
draft's 25_6 shows it).

## 01_the_question: How long is it? (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Here is a coastline, drawn on a card sixteen centimetres wide. A cove, a long beach, a knobbly headland, a round bay and a hook at the far end. How long is it? | The card fades in, a slightly lighter panel with "16 cm" under its lower edge. On "Here is a coastline" the coast draws itself in blue from its start, by arc length. As each part is named its label appears in `dim` beside it ("cove", "beach", "headland", "bay", "hook"); on "How long" a `hero` ? in the card's empty upper right, and the labels fade | coast from `model::COAST`, drawn by arc length from a tracker, labels at the model's piece middles | |
| 01_2 | I mean the length of the line itself, every bend included, not the distance between its ends. | A green dot runs along the whole coast in 4 s, leaving a thin green trail over the blue: the length we are after. Then a dashed `dim` segment joins the two ends, "14.7 cm end to end" (`coast.ends_apart`); the trail, the dot and the segment fade | dot moving by arc length from a tracker | |
| 01_3 | A ruler won't do it. It's straight, so it fits the beach, but on the headland it touches at a single point. | An `ink` ruler 5 cm long lies under the beach and fits along it. It lifts, tilts 7.5° and settles on the headland's top, touching it at one point, a small gold dot; then the ruler and the dot fade | ruler placed from the model's beach and headland tangent | |
| 01_4 | Here's a quicker tool. Dividers, two points a fixed step apart. Open them to two centimetres and walk them along the coast, each step landing back on the line. | Red dividers, two legs from a hinge, stand closed at the coast's start; on "Open them" they open to 2 cm, "2 cm" beside them. On "walk them" the "2 cm" fades and a live count of steps and the word "steps" appear together on the card under the coast, at 1; the dividers walk: each step pivots on its last landing and swings the other leg over to the next, each landing a red dot and each step a red chord, while the count ticks to 11. A short red chord runs from the last landing to the end | divider walk from the model, live count from a tracker | |
| 01_5 | Eleven steps and a short bit at the end. Will the total come out too long, too short, or about right? ⟨predict⟩ | The dividers fade; the chords stay. The count gives way, on the card under the coast, to "11 × 2 cm + 0.64 cm = 22.6 cm" (`dividers.2_0.steps`, `dividers.2_0.last`, `dividers.2_0.length`). Countdown in the card's upper right | TeX, think countdown | |
| 01_6 | Too short, and it can never come out long. Each step goes straight between its two points, and a straight line is shorter than the bend it cuts across. | The sum fades. A zoom: the card, the coast and the chords scale 2.2 times about the middle of step 6, the step that leaves the headland. That chord thickens and the stretch of coast under it brightens; beside them "2 cm straight < 3.08 cm of coast" in TeX (`dividers.2_0.headland_coast`) | zoom (scale about a point), TeX | hold |
| 01_7 | Each smaller step cuts less off its bend, but it needs more steps, and every step still has to follow the coast. | The TeX fades and the zoom comes back out; the 2 cm chords and dots dim. Dividers closed to 1 cm take three quick steps from the start, then stop; dividers and chords fade, leaving the card and the coast alone | divider walk from the model | |

### Notes for 01

- The opening is motivation, not measurement: the viewer meets the coast,
  the goal (length, every bend) and why the obvious tools are awkward,
  before any sheet or formula (owner direction, 2026-10-24).
- Prediction (01_5): answerable from the picture (a chord is shorter than
  its arc). It is also the film's first small payoff: the dividers' error
  has a sign, and the draft's 25_4 returns to it.
- The dividers' 22.6 cm is shown, not the true length: the true length stays
  hidden until the draft's 06_4, so the viewer cannot yet say how short.
- The zoom in 01_6 lands on the headland, where the ruler fitted at one
  point in 01_3: the same place defeats both tools.
- 01_7 is about one step at a time: on any one arc a shorter chord cuts
  less off than a longer one. It promises nothing about the whole walk,
  which on this coast does not grow steadily as the step shrinks (from
  0.25 to 4 cm in 0.05 cm steps, a longer step walks further at 46 of 75
  increments; `oracle.py`, with a square of side 2 as a control). Nothing in
  01_7 answers or reopens 01_5's question.
- Timing, estimated and not measured: from "walk them" the walk takes
  5.4 s (the "2 cm" fade, ten swings of 0.36 s each with a 0.12 s landing,
  the last chord's fade), against about 4.1 s for the line's last eleven
  words at the tool's 2.68 words a second (4.7 s with the tool's 0.6 s
  breath). So about a second of silent walking ends the beat before 01_5
  says "Eleven steps"; at the first handoff's 0.42 s swings it was about
  two. A first render should confirm it reads as watching, not as dead air.
- The live count is rebuilt every frame, so it cannot fade in; its word
  appears with it, so the pair arrives together on the walk's first beat.
- Semantic pauses: a thought gap after 01_1's question; the 01_6 hold, the
  close-up still, before the method changes in 01_7.
- Continuity: 01 ends on the card and the coast alone, which is where the
  draft's chapter 02 begins.
- Pronunciation: no names. "Centimetres" (British spelling, as the owner's
  other films).

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
| 03_3 | Now swap in seven small half circles instead. That adds just as much length, but here they cross no line at all. So at one placement, more length doesn't have to mean more crossings. The hope is that it does on average, over many placements. | On "seven small half circles" the big half circle, its dots and "9 crossings" fade, and seven half circles of radius 0.25 cm draw along the beach in 1 s; their tops stay 0.18 cm under the line at −1.55 cm. On "just as much length", under the TeX in `dim`, "both about 1.998 cm longer" (`swap.extra_3dp`). On "no line at all" "7 crossings" in gold beside the TeX. On "The hope is" the half circles and the words fade, the straight beach returns, and "hope: on average, more length means more crossings" appears in gold under the coast | computed illustration, precisely card | |
| 03_4 | How many centimetres, then, is one crossing worth? The lines are two centimetres apart. Is each crossing worth two centimetres? ⟨predict⟩ | The hope fades. On "two centimetres apart" 02_3's bracket and "2 cm" return. On "Is each crossing worth" "1 crossing = 2 cm ?" in gold under the coast. Countdown | think countdown | |
| 03_5 | It can't be. Seven times two is fourteen centimetres, and the two ends of the coast are already fourteen point seven apart. No curve joining them can be shorter than that. | The bracket and the "?" go. On "Seven times two" the guess gives way to "7 × 2 cm = 14 cm" in red (`open.gap_guess`). On "fourteen centimetres" a thick red bar 14 cm long grows in 1 s from the coast's start, along the straight line toward its end. On "the two ends of the coast" 01_2's dashed `dim` segment joins the ends, with "14.7 cm end to end" (`coast.ends_apart`); the bar stops 0.73 cm short of the end. On "No curve joining them" the whole coast thickens and settles back | bars to scale (the bar against the end-to-end segment), TeX | |
| 03_6 | The exchange rate has to come from somewhere else, from a curve whose length we already know. | The sum, the bar, the segment and "14.7 cm end to end" fade; "7 crossings → ? cm" stays | | |
| 03_7 | Since I built this coast myself, from four straight runs and seven pieces of circles, I know its true length exactly. It goes in this envelope until counting has had its chance. | On "four straight runs" the four runs thicken (`coast.straight_runs`). On "seven pieces of circles" the runs settle and the seven arcs thicken (`coast.arcs`), and on "I know its true length" the arcs settle. The coast keeps its colour throughout. On "this envelope" a green envelope with its flap and "true length" under it appears in the band over the coast at the card's left; no number is shown. Then the card, the coast, the sheet's lines, the dots and the count dim, and the title "Measure a Curve Without Following It" appears over them while the envelope stays bright | title card | hold |

### Notes for 03

- Root frame repair: hide the original "7 crossings → ? cm" during both
  replacement beaches, restoring it only when the original beach returns.
  After the two extra dots and "9 crossings" are fully visible in 03_2,
  retain the complete picture for at least 2.6 seconds before 03_3 begins.
  This is deliberate inspection time, not silence to trim. The three-decimal
  added-length label says "both about 1.998 cm longer" because it is rounded.

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
- Root folded these revised rows into the full draft
  (`docs/storyboard-draft.md`, `docs/script-draft.tsv`), preserving all later
  chapters. The frozen candidate retains the earlier proposal verbatim.

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
| 05_3 | Always two. The lines are exactly a coin's width apart. At a typical landing, one line runs through the coin, and only one. Its rim crosses that line twice. Turning changes nothing, because a circle looks the same turned any way. | On "Always two" a live counter, "2 crossings", fades in where the count was (`coin.crossings`). On "Turning changes nothing" the coin turns once on the spot in 2.4 s, its mark going round while the dots and the halo stay put. Then, in silence, it is set down at four more places, sliding and turning on the way, 1.6 s each with half a second's rest: (2.6, 0.75), (−1.5, −3.2), (5.2, −2.3) and (4.5, 1.85) cm. Every frame the meeting points and the line through it are solved again. The halo jumps to the next line the moment the coin's centre passes midway between two lines, and the counter stays at 2 | live meeting points, halo and counter from the model | hold |
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

## 06_the_bet: The bet (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now a bet. Suppose every curve earns crossings at the same rate as the coin's rim, centimetre for centimetre. Then a crossing on any curve is worth pi centimetres. | Where 05 ended. The card and coast are dimmed to a quarter, and the sheet is bright at the first placement beside the sealed envelope. The coin rests at (6.28, 3.45) cm after its roll, touching the lines at 2.45 and 4.45 at one point each, with its two gold dots and "2 crossings". Its rim is laid out along the line at 2.45 with "6.28 cm" over it, and 05's two rows, "2 crossings ↔ 6.28 cm" and "1 crossing ↔ π cm ≈ 3.14 cm", sit under the coast. On "Now a bet" the coin, its dots, its count, the laid-out rim with its label and the first row fade in 0.4 s; the second row stays. On "every curve" the card and coast come back to full in 1 s. On "a crossing on any curve" an `ink` tag, "the bet", fades in to the left of that row | TeX row, label | |
| 06_2 | Our coast had seven crossings at the first placement. What length does the bet give? ⟨predict⟩ | On "seven crossings" the seven gold dots and a live counter, "7 crossings", fade in where 04's counter stood. The count is solved from the model at the placement shown, every frame (`open.crossings`). On "What length" "→ ? cm" follows the count, the ? in gold. The envelope stays shut. Countdown | live counter from the model, think countdown | |
| 06_3 | Seven times pi, about twenty-two point zero centimetres. | On "Seven times pi" the countdown's ? and "→ ? cm" give way to "→ 7 × π cm ≈ 22.0 cm" (`open.estimate`; 7π is 21.99). The envelope is still shut | TeX | |
| 06_4 | Now open the envelope. The true length is twenty-five point eight centimetres. | On "open the envelope" its flap swings up in 1 s, its point rising 0.3 over the envelope's top edge and staying under the line at 4.45. On "twenty-five point eight" "25.8 cm" appears in green beside "true length" (`coast.length`). This is the first time the true length is on screen | reveal | |
| 06_5 | Side by side and to scale, the bet comes up three point eight centimetres short. That's about fifteen percent. | On "Side by side" the envelope's body and flap and the bet's row fade, while the coast, sheet and crossing dots dim to 0.12 opacity. In 1.2 s "true length" and "25.8 cm" move down into the chart's top row. Over the next 1.2 s "the bet" moves into the lower row and "22.0 cm" appears at the future gold endpoint. Only after both labels land, the geometry returns and both bars grow together for 1.2 s from the same left origin, at 0.2 frame units a centimetre. The green bar represents 25.8 cm; the live gold bar is π cm per crossing, initially 21.99 cm (`open.estimate`). On "three point eight centimetres short" an `ink` bracket under the gold bar runs from its end to the green bar's end, with an upright there, and "3.8 cm short" appears beside the rows (`open.short_by`). On "about fifteen percent" "about 15%" follows it (`open.error_percent`; 3.8 of 25.8 is 14.7%) | persistent labels, sequential label transfer before bar growth, bars to scale from the model, live bar, bracket highlight | |
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

## 10_a_crowd_of_sticks: A crowd of sticks (~118 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Back to the coast. Look closely at any smooth curve, and it's almost straight. So think of the coast as a crowd of tiny sticks, laid end to end. | Where 09 ended, rebuilt over the shore in 09's drawing order: the stick lying along the lines with its light and its point of shadow on the ruler, the ruler, 08's rule, 09's plot with the arch, its two measured points, the dashed half line, the flattened fill and its words, 09's rows, "6 cm" over the stick, and the ratio, the coin and its "?". The card and the coast are dimmed as 07 left them, 08's console and 09's panels are up. On "Back to the coast" all of 09 fades, with the console and the panels (0.4 s), and the card and the coast come back to full strength. On "Look closely" an inset fades in at the card's upper right: a disc of radius 1.2 showing the bay three times over round its lowest point, with a small ring on the bay where it looks. Inside it the magnified bay is almost straight, and the two whole sticks there are purple chords with their three end ticks. On "a crowd of tiny sticks" ink ticks across the coast at both ends of every stick, lagged over 2.5 s. There are 67 sticks, none longer than 0.45 cm, cut at the joints and where the coast lies level with the lines (`sticks.count`) | inset magnifier from the model, lagged ticks | |
| 10_2 | Every crossing of the coast is a crossing of one of these sticks. So the coast's count is just its sticks' counts, added up, and here that's seven. | On "Every crossing of the coast" the inset goes and the first placement's seven gold dots fade in. On "one of these sticks" a halo lights each of the seven sticks that hold them, each dot inside its own stick. On "here that's seven" "7 crossings" fades in at the card's upper left, counted live from the model (`open.crossings`) | highlight, live counter from the model | |
| 10_3 | Each stick runs one way across the lines, as the straight stick did, so the shadow rule holds for it. Its average count over every slide is its shadow divided by the gap. So the coast's average count is all its sticks' shadows, added up, then divided by the gap. | On "the shadow rule holds for it" 08's console fades in under the coast with 08's rule on its top row, `average over every slide = shadow ÷ gap`, and the seven halos go. On "all its sticks' shadows" the coast's rule under it, `average for the coast = all its shadows ÷ gap` | TeX | |
| 10_4 | Lay their shadows end to end along this bar, starting at the cove. Along the beach, nothing is added. It lies along the lines, so it casts no shadow. | On "Lay their shadows end to end" the top rule goes. On the console an axis under the bar's place, ticked at every gap of shadow, numbered "0" to "9" with "gaps" after the last: one frame unit a gap, 2 cm of shadow. A purple trace along the coast and the bar at the console's left both start empty. On "starting at the cove" the trace runs along the cove in 2.5 s and the bar grows with it, each stick's shadow a span, alternating full and lighter purple. On "Along the beach" the trace runs along the beach in 3 s and the bar does not grow. On "casts no shadow" "beach 0 cm" over the bar's end (`beach.shadow_0`). After the line, in silence, the trace and the bar run on over the climb, the rise, the headland and the fall in 7 s, to 3.85 gaps, unlabelled | live bar from the model | hold |
| 10_5 | Round the bay, the coast goes down and then comes back up, so both of its shadows go in. That's right, because this line through the bay crosses it twice, once on the way down and once on the way back up. | On "Round the bay" "beach 0 cm" goes, a halo lights the bay's way down, from its left end to its lowest point, and the trace and the bar run on over it in 2.5 s, its span haloed on the bar as well. On "comes back up" the way back up is haloed on the coast and on the bar, and laid in 2.5 s. On "this line through the bay crosses it twice" a halo on the lower line through the bay (`open.bay_crossings`). On "once on the way down" a gold ring round its crossing left of the lowest point. On "once on the way back up" a ring round the one right of it | live bar from the model, highlight | |
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
  no longer than 0.45 cm: 67 sticks, 0.236 to 0.438 cm. A search from 0.30 to 0.60 cm in 0.001 cm steps found no minimum
  clearance above 0.074 cm. Keep 0.45 for this 67-piece demonstration;
  its minimum clearance of 0.031 cm still requires inspection in actual frames.
- The live bar, its halos and the trace are drawn every frame from the
  sticks' shadows at the sheet's direction (`bar_spans`), so 11 turns them
  with the sheet without a second model.

## 11_turning_the_sheet: Turning the sheet (~102 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So far the lines have all run one way. Turn the sheet an eighth of a turn, and every shadow changes. Where a stick now turns back, both of its shadows go in, as in the bay. Now the average comes to eight point seven four crossings. | Where 10 ended. On "So far the lines have all run one way" the count, its dots, 04's readings, the mark, "15.26 cm" and the result go. 09's panel comes back at the card's upper right, and on it a table under "average over every slide", with its first row "0° 15.260 ÷ 2 = 7.63" (`turn.0.reach_3dp`, `sheet.gap`, `turn.0.slide_mean`). On "Turn the sheet an eighth of a turn" "lines at 0°" goes and the sheet turns 45° about the card's centre in 3 s. The bar is redrawn every frame from the same sticks' shadows at the sheet's direction, and grows. On "eight point seven four crossings" the row "45° 17.485 ÷ 2 = 8.74" (`turn.45.reach_3dp`, `turn.45.slide_mean`) | live bar from the model, table built from the model | |
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
- The sticks are cut for 0°, so on the turned sheet a few of them turn
  back across the lines: 6 at 45°, 4 at 90° and 3 at 135°. `shadows` lays
  such a stick's travel, both of its shadows, as 10_5 laid the bay's, and
  that is what makes the bar the coast's travel. End-to-end shadows alone
  would give 8.70, 8.20 and 7.87, not 8.74, 8.23 and 7.91; at 45° that is
  0.089 cm of bar, 0.045 frame units. Nothing spoken says the sticks
  still run one way: 10_3 says it of 0°, and 10_9 limits 10's average to
  it.
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

## 12_why_every_curve: Why every curve (~306 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | So far we've checked the bet by computing, on one coast. Why should it work for every curve? Here's the claim. Average the count over every slide and every direction, and you get the curve's length times one fixed number, the same number for every curve. | Where 11 ended, rebuilt over the shore: 11's plot with its curve, the flattened fill and its level labelled "16.433 cm", its list of grids and estimates, its rows "16.433 ÷ 2 = 8.2165", then "8.2165 × π ≈ 25.81 cm" and "true length 25.81 cm", the caption "computed, not proved", 10's stick ticks on the coast and 09's panels. The card and the coast at full strength, 08's console up. On "Why should it work for every curve" all of 11, the ticks and the panels fade (0.4 s). On "Here's the claim" the claim on the console's top row, `average count = length × ?` | TeX | |
| 12_2 | Barbier made this argument in eighteen sixty, without a single integral. It rests on two ideas. The first is that crossings add up. | On "Barbier made this argument" a lower third, "source: Barbier, 1860, J. Math. Pures Appl., pages 275 to 277" (`barbier.year`). On "crossings add up" it goes | lower third | |
| 12_3 | Cut the coast at its joints, and look at this placement. The cove is crossed, the beach and the headland aren't, and the bay is crossed twice, once on its way down and once on its way back up. Add up every piece's crossings, and you get the coast's seven. | On "Cut the coast at its joints" short ink ticks across the coast at its eleven pieces' ends, every other piece dimmed to half its stroke (stroke only, `dim_line`) so the pieces show in two tones, and the claim goes. The sheet is at the first placement, lines at y −1.55, 0.45 and 2.45 card cm. On "The cove is crossed" the first placement's seven gold dots, each on its own piece. On "the bay is crossed twice" the bay's two halos, its way down and its way back up, both crossed by the lower line. On "Add up every piece's crossings" a live row on the console's top row, each piece's count in order with a plus after it, "1 + 0 + 1 + 1 + 0 + 1 + 2 + 0 + 0 + 0 + 1 = 7" (`open.piece_counts`), each digit counted from the model at the sheet's placement. On "the coast's seven" "7 crossings" at the card's upper left (`open.crossings`, `open.bay_crossings`) | live sum and counter from the model, highlight | |
| 12_4 | That's true at every placement. Slide the sheet, and the pieces' counts change, but they always add up to the coast's count. So the pieces' averages add up to the coast's average too. | On "Slide the sheet" the bay's halos go and the sheet slides one whole gap across in 8 s, at an even rate. The seven dots move along the coast, the row's digits change as lines pass the pieces' ends, and the sum on its right always equals the count at the upper left. After a gap the lines lie where they began, and the row reads its first values again | live sum and counter from the model | hold |
| 12_5 | Now take our six centimetre stick. Move it, and its count can change. Here it crosses two lines instead of three. But moving the stick is just like moving the sheet the other way. Put the stick back and move the sheet instead. Two again. | On "Now take our six centimetre stick" the row, its signs, the joints' ticks, the coast's dots and its count go and the coast returns to one tone. Then 07's 6 cm stick at home, lying across the lines with its middle at card (−4.5, 1.2), its three gold dots and its count "3 crossings" where the coast's stood (`stick.length`, `stick.home_across`). On "Move it" the stick glides 1.5 cm right and turns to 45° in 2.5 s. Its count becomes "2 crossings" (`move.count`). On "Put the stick back" it glides home in 2.5 s, three again. On "move the sheet instead" the sheet turns by 45° about the stick's middle in 3 s, the way that undoes the stick's turn, and the count reads "2 crossings" (`move.sheet_count`). At this first direction the 1.5 cm shift runs along the lines and moves none of them | live wire, dots and counter from the model | |
| 12_6 | The average gives every placement of the sheet an equal share, and moving them all by the same amount only reshuffles them. So a stick's average can't depend on where it lies or which way it points. It can only depend on its length. | On "only reshuffles them" the sheet turns back to the first placement in 3 s, three again | live counter from the model | |
| 12_7 | Cut the stick in half. Each half counts its own crossings, and together they make the whole stick's count. Watch the cut as a line passes it. One half loses a crossing, and the other gains it. | On "Cut the stick in half" a short ink tick across the stick at its middle. The upper half is drawn lighter (two tones), and each half's live count stands beside it, "2" left of the lower half and "1" left of the upper half (`halves.counts`). On "Watch the cut" the sheet slides one gap in 10 s at an even rate. As the line at 0.45 passes the cut at 1.2, the lower half's count drops to 1 and the upper half's rises to 2, while the whole stays "3 crossings". Then the "precisely" lower third, "a line exactly through the cut is a single slide among a whole gap of them: it has no chance, and changes no average" | live wire and counts from the model, lower third | hold |
| 12_8 | The pieces don't even have to stay in line. Lay one half across the other. Where they cross, a line through that point meets both halves, so it counts twice, just as the lines beside it cross each half once. | On "Lay one half across the other" the tick and the lower third go. In 3 s the lower half swings 45° about its own middle, to the upper left, and slides up along the stick until its middle lies on the upper half's middle, card (−4.5, 2.7). Every frame both halves keep 3 cm. At the first placement the count is "2 crossings", one on each half (`cross.count`). On "a line through that point" the sheet slides in 2 s until a line runs through the meeting point, and a gold ring goes round it. The count still reads "2 crossings", one dot for each half at the one point, and each half's own count reads "1" (`cross.through`) | live wire and counts from the model, highlight | |
| 12_9 | Equal pieces have equal averages, since one can be moved onto the other. So each half has half the whole stick's average, each third a third, and each quarter a quarter. Three quarters together, four and a half centimetres, have three quarters of it. So for every length that's a fraction, the average is the length times one number, the average for one centimetre. | On "Equal pieces have equal averages" the ring and the halves' counts go, the lower half swings back into line in 3 s, and the sheet slides back to the first placement in 2 s. On "each third a third" two ticks cut the stick into thirds of 2 cm (`pieces.third`) and the two tones go. On "each quarter a quarter" three ticks into quarters of 1.5 cm replace them (`pieces.quarter`). On "Three quarters together" a halo along the lower three quarters, 4.5 cm (`pieces.three_quarters`), and on the top row `average for 4.5 cm = 3 × average for 1.5 cm = ¾ × average for 6 cm`. On "So for every length that's a fraction" on the low row `average count = length × average for 1 cm` | live wire, highlight, TeX | |
| 12_10 | Not every length is a fraction. Take the straight line between the coast's two ends, fourteen point seven three centimetres and a bit. Lay a fourteen and a fifteen centimetre stick along it, from the same end. Turn the sheet, slide it, and watch. A longer stick is always crossed at least as often as the shorter one inside it. | On "Take the straight line between the coast's two ends" the stick, its dots and its count, the halo, the rows and the ticks go, the coast dims to 0.35 of its stroke, and the straight line from the coast's start to its end is drawn in the length's green (`ends.apart_2dp`, 14.73 cm). On "Lay a fourteen and a fifteen centimetre stick" ink ticks across the line 14 and 15 cm from the start, a purple stub from the far end out to 15 cm, and on the top row three labels, "14 cm", "ends" and "15 cm", each followed by its live count. The 15 cm stick's gold dots are drawn too. On "Turn the sheet, slide it" the sheet turns a quarter turn in 3 s about card (0, 0.45), its lines upright and the counts "7 7 7" (`ends.nested.0`), then slides 1 cm across in 3 s, the counts "7 8 8" (`ends.nested.1`). Every end keeps more than 0.25 cm from a line at both placements | live counts and dots from the model | hold |
| 12_11 | So its average lies between fourteen and fifteen times our number. Between fourteen point seven and fourteen point eight times it, and between fourteen point seven three and fourteen point seven four. Squeezed tighter and tighter, it can only be the length times the number. | On "between fourteen and fifteen times" on the low row `14 × ? ≤ average ≤ 15 × ?` (`ends.bracket.0`). On "Between fourteen point seven" it becomes `14.7 × ? ≤ average ≤ 14.8 × ?` (`ends.bracket.1`), and on "between fourteen point seven three" `14.73 × ? ≤ average ≤ 14.74 × ?` (`ends.bracket.2`). The line, the ticks and the live counts stay | TeX | |
| 12_12 | So for any chain of straight sticks, the average count is exactly its total length times that number. Here's a chain with its corners at the coast's joints. It's twenty-one point three nine centimetres long, and here it's crossed only five times. Its chord across the bay misses both of the bay's crossings. | From the line's start the marks, the labels, the line, the counts, the dots and the bracket go (0.4 s). While the first sentence is said the sheet slides back 1 cm and turns back a quarter turn, 3 s each, to the first placement. On "Here's a chain" a purple chain of eleven chords with its corners at the coast's joints (`chain.joints.chords`), its own purple dots, the coast's seven gold dots and "7 crossings", and on the top row "chain 21.39 cm, 5 crossings" (`chain.joints.length`, `chain.joints.crossings`). On "Its chord across the bay" the bay at full strength with its two halos: both gold dots on the bay, neither on its chord | chain and dots from the model, highlight | |
| 12_13 | A line can only cross a chord if it crosses the coast between the chord's ends, so a chain is never crossed more often than the coast. Put more corners on the coast, and the chain gets longer and catches more. Twenty-three point nine one centimetres, then twenty-five point seven one, and here both catch all seven. | On "Twenty-three point nine one centimetres" the joints' chain, its dots and its row give way to the chain with corners at 10's cuts, fifteen chords, "chain 23.91 cm, 7 crossings" (`chain.cuts.chords`, `chain.cuts.length`, `chain.cuts.crossings`), and the bay dims with the coast again. On "then twenty-five point seven one" the chain with corners at 10's sticks' ends, sixty-seven chords, "chain 25.71 cm, 7 crossings" (`chain.sticks.chords`, `chain.sticks.length`, `chain.sticks.crossings`), its purple dots on the gold ones | chain and dots from the model | |
| 12_14 | As the corners crowd together, the chains' lengths rise to the coast's length, and at almost every placement their counts rise to the coast's count. So the coast's average is its length times the same number. Chains give it exactly, and for a curve, it's a limit. | On "As the corners crowd together" the "precisely" lower third, "for a curve with a length: the chains' averages rise to the curve's. Lines through a corner or an end, or touching the curve, have no chance and change no average". The finest chain and its row stay | lower third | |
| 12_15 | One number, the same for every curve. To find it, we need just one curve whose average we already know. Which would you use? ⟨predict⟩ | On "One number, the same for every curve" the chain, its dots and row, the coast's dots and count and the lower third go, the coast comes back to full strength, and the claim returns to the top row, `average count = length × ?`. On "Which would you use" a countdown at the card's upper right | think countdown | hold |
| 12_16 | The coin. Away from exact tangencies, it crosses the lines at two points, and its rim is two pi centimetres. Those exceptional placements have no effect on the average. So two is two pi times the number, and the number is one over pi. | On "The coin" the countdown's ? goes. 05's teal coin, 2 cm across, lies at rest at card (2.0, 2.4) over the line above the bay, with its two gold dots and its count "2 crossings" at the upper left (`coin.crossings`). On "Away from exact tangencies" a lower third says "exact tangencies change no average"; it clears before the calibration equation. The sheet slides one gap in 4 s, then turns half a turn about card (2.0, 0.45), a point on a line, in 6 s. The coin's count stays "2 crossings" while its dots run round the rim. On "So two is two pi times the number" on the low row `2 = 2π × ?` (`coin.rim`, 6.28). On "the number is one over pi" beside it, in the length's green, `? = 1 ÷ π` | live dots and counter from the model, TeX | hold |
| 12_17 | So on this sheet, every curve averages its length over pi crossings, and each crossing is worth pi centimetres. The bet wasn't luck. It was forced. For our coast, twenty-five point eight one over pi is about eight point two two, the average the computer found. | On "every curve averages its length over pi crossings" the claim and the coin's rows give way to `average count = length ÷ π` on the top row, and the coin's dots and count go. The coin stays at rest. On "twenty-five point eight one over pi" on the low row, in the length's green, "25.81 ÷ π ≈ 8.22" (`coast.length_2dp`, `crofton.mean_crossings`, `coin.worth`). The picture stands at least 2.6 s | TeX, callback | hold |

### Notes for 12

- The proof is Barbier's, in the order of his pages 275 to 277: additivity,
  invariance, proportionality, polygons, then curves, then the coin. Each
  step is shown on the chapter's own objects before it is used. The 12_3
  row and the count come from the same crossings the dots are drawn from.
  The sum reads 7 at the first placement and keeps equal to the count
  through a whole gap (12_4, `crossings_add_piece_by_piece`, 900
  placements against a fine-chord count that does not use
  `sheet::crossings`).
- 12_5 earns invariance on the stick, not on the coast. The moved stick's
  count (2) differs from home's (3), and the sheet moved back gives 2 again.
  The test repeats that for 400 random moves and turns, every one equal.
  The mirror's negative control, the sheet moved the same way instead of
  back, differs at 291 of 400.
- 12_7 shows the cut passing a line, so additivity is seen to hold while
  the halves' own counts change: they read (2, 1), then (1, 2), and the
  whole always 3 (`the_halves_add_up_through_a_slide`, 400 slides). 12_8
  is multiplicity: the line through the meeting point counts once for
  each half, two, though it meets the card at one point. The test and the
  oracle both state that counting points of the card would give one, a
  negative control.
- 12_9 to 12_11 are the step the full draft skipped. Equal pieces give
  every fraction. For a length that is not one, the line between the
  coast's ends, a longer stick is never crossed less often than a shorter
  one inside it. That holds at 3,600 placements in the test and at each
  shown one. So the average is squeezed between the brackets' tenths and
  hundredths. The brackets' bounds come from the oracle (`ends.bracket.k`),
  and the lower tenth is 03's 14.7 (`coast.ends_apart`). The voice's "and a
  bit" says 14.73 is rounded down.
- 12_12 and 12_13 bound the chains from one side. A chord crosses a line
  only if the coast between its ends does. So a chain with its corners on
  the coast is never crossed more (3,600 placements, none more), and finer
  chains match the coast more often (1,742, 2,640, 3,542 of 3,600). The
  test's negative control lifts the joints' chain 0.3 cm off the coast,
  and then it can be crossed more. The joints' chain misses the bay's two
  crossings exactly: its count plus the bay's two is the coast's seven.
- 12_14 is a limit, and the voice and the lower third say so. It is not
  presented as a step of the exact argument. "At almost every placement"
  is the claim. A placement through a corner, an end or a tangent point is
  a single direction-and-slide among a continuum. It is not recounted or
  excluded on screen, and the coast's counts at the shown placements stay
  as sheet.rs counts them.
- 12_15's prediction asks which curve would find the number. The coin comes
  back because 05 established that it always meets the lines at two points
  (the two-tangency instant is 05's exceptional placement). The half turn
  in 12_16 is about a point on a line, so the lines end where they began
  and the coin's count can be checked at every frame of the turn. The test
  checks 3,000 placements, all of them 2.
- 12_17's numbers are the established 25.81 and 8.22, and the screen says
  "25.81 ÷ π ≈ 8.22" from the shown operand (it rounds the same way as the
  unrounded length). The test's negative control: 8.22 × π rounds to 25.82,
  not 25.81, so nothing multiplies the rounded mean back.
- "Barbier" is said once, with the year. The lower third cites the journal
  and pages that `brief.md` read from the scan. "without a single integral"
  is p. 277's "sans même avoir une seule intégration à effectuer".
- Pronunciation risks for qualification: "Barbier" (French bar-BYAY; a
  narrator may say BAR-bee-er), "eighteen sixty", "fourteen point seven
  three centimetres and a bit", "twenty-one point three nine", "two pi
  centimetres", "one over pi", "reshuffles". Nothing has been listened to.

## 13_the_bent_stick: The bent stick (~145 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Back to our six centimetre stick. Its length over pi says it averages about one point nine one crossings. And by the shadow rule, its average shadow is the gap times that average, twelve over pi, about three point eight two centimetres. That's the stick's sixty-four percent from before, and now we know why it's two over pi. The coin explained it. | Where 12 ended: the shore at full strength, the coin at rest at card (2.0, 2.4), and 12's rows `average count = length ÷ π` and "25.81 ÷ π ≈ 8.22". On "Back to our six centimetre stick" the coin and the rows go, and the 6 cm stick comes back at home lying across, with its three gold dots and "3 crossings" (`stick.length`, `stick.home_across`). On "about one point nine one crossings" on the top row "6 ÷ π ≈ 1.91" (`stick.turned_mean`). On "twelve over pi" on the low row `2 × 6 ÷ π = 12 ÷ π ≈ 3.82 cm`, checked as its two steps "2 × 6 ÷ π = 12 ÷ π" and "12 ÷ π ≈ 3.82 cm" (`sheet.gap`, `stick.mean_reach`). On "sixty-four percent" beside it, in the length's green, "3.82 ÷ 6 ≈ 0.64 ≈ 2 ÷ π" (`stick.mean_reach_share`, `stick.mean_reach_percent`). That is 09's 64% | live wire, dots and counter from the model, TeX, callback | |
| 13_2 | Turn it to lie along the lines, and it crosses none. Now bend the same wire at its middle, into two arms of three centimetres at a right angle. Here it crosses one line. Its length hasn't changed, six centimetres. Will its average count go up, go down, or stay the same? ⟨predict⟩ | On "Turn it to lie along the lines" the rows go and the stick turns about its middle in 2.5 s to lie along the lines, between the lines at 0.45 and 2.45. Its count reads "0 crossings" (`stick.home_along`). On "Now bend the same wire" its left arm swings about the middle in 3 s, up through the open card at the upper left, to stand upright from the corner at card (−4.5, 1.2). It crosses the line at 2.45, "1 crossing" (`hook.home_count`), and every frame keeps both arms 3 cm. On "Its length hasn't changed" "3 cm" beside each arm, and on the top row "3 + 3 = 6 cm" (`stick.length`). On "or stay the same" a countdown at the card's upper right | live wire, dots and counter from the model, think countdown | hold |
| 13_3 | It stays the same, about one point nine one. Each arm is a three centimetre stick, and crossings add up, so the bent wire averages what its two arms do, which is what the straight stick does. Turn the sheet, and its count moves around, but its average doesn't. | On "It stays the same" the countdown's ? goes and on the low row `3 ÷ π + 3 ÷ π = 6 ÷ π ≈ 1.91`, checked as "3 ÷ π + 3 ÷ π = 6 ÷ π" and "6 ÷ π ≈ 1.91" (`hook.mean`). On "Turn the sheet, and its count moves around" the arms' labels on the card go, then the sheet turns half a turn in 12 s, at an even rate, about card (−4.5, 0.45), the point under the corner on a line. The count runs 1, 2, 3, 2, 1, 2, 1 and ends where it began (`hook.sweep`) | live wire, dots and counter from the model | hold |
| 13_4 | What changes is how the count is spread. Straight, the stick misses every line about ten point seven percent of the time. Bent, it never does, because one of its arms always reaches more than a gap across the lines. Different spreads, the same average. | On "What changes is how the count is spread" the rows give way to the straight stick's spread on the console's left: purple bars over the counts 0, 1, 2 and 3 (`stick.all_counts`), each bar's height its chance, "10.7%", "22.9%", "31.2%" and "35.3%" over them (`stick.shares_percent`), "straight" and "average 1.91" at their left. On "Bent, it never does" the bent wire's spread on the console's right: bars over 1, 2, 3 and 4 (`hook.counts`), "23.5%", "62.1%", "14.3%" and "0.1%" (`hook.shares_percent`), "bent" and "average 1.91". There is no bar over 0. Each chance is from the model's count shares over 3,600 directions | computed spreads from the model | hold |
| 13_5 | Bend it once more, into a shape that crosses itself, still lying flat on the card. Where its pieces cross, a line through that point counts both of them, as before. Its average is still one point nine one, because its pieces still add up to six centimetres. | On "Bend it once more" the spreads go, and in 3 s the upright arm swings an eighth further about the corner while its outer half turns three eighths at its joint, into a 4 whose last piece crosses the arm along the lines at card (−3.44, 1.2). Every frame keeps its pieces 3, 1.5 and 1.5 cm. On "Where its pieces cross" a gold ring round the crossing, then the sheet turns an eighth about it and slides in 3 s until a line runs through it. The count reads "2 crossings", both at the ring, one on each piece (`four.count`, `four.through`). On "counts both of them" the "precisely" lower third, "the wire lies flat on the card and is counted piece by piece, so a line through its crossing counts twice. A wire bent out of the card is another question". On "still one point nine one" "average 1.91" on the top row (`four.mean`) | live wire, dots and counter from the model, highlight, lower third | |
| 13_6 | This is known as Buffon's noodle. However a wire is bent, as long as it lies flat on the card, it averages its length over pi crossings on this sheet. | On "Buffon's noodle" the lower third and the ring give way to a source lower third, "source: J. F. Ramaley, Buffon's noodle problem, Amer. Math. Monthly, 1969" (`noodle.year`). The 4 and "average 1.91" stay | lower third | |
| 13_7 | And our coast is just a long bent wire. At our first placement it crossed the lines seven times. On average it crosses its length over pi, about eight point two two times. Turn that around, and pi times the average count gives back its length, twenty-five point eight one centimetres. That's why counting crossings can measure a coast. | On "our coast is just a long bent wire" the source, "average 1.91", the wire, its dots and its count go, and the sheet goes back to the first placement in 3 s, one move. On "seven times" the coast's seven gold dots and "7 crossings" (`open.crossings`). On "gives back its length" on the low row, in the length's green, "8.2165 × π ≈ 25.81 cm" (`crofton.mean_crossings_4dp`, `coast.length_2dp`, `crofton.mean_crossings`). The picture stands at least 2.6 s | live dots and counter from the model, TeX, callback | hold |

### Notes for 13

- 13_1 derives 09's 64% instead of announcing it. 08's shadow rule (the
  average over every slide is the shadow divided by the gap) is exact at
  each direction. Averaged over directions, the average shadow is the gap
  times the average count, 2 × 6 ÷ π = 12 ÷ π, and 12 ÷ π over 6 cm is
  2 ÷ π. The row shows the operands 2, 6 and 12, so nothing multiplies
  rounded numbers. `words()` refuses unless 3.82 ÷ 6 rounds to the shown
  0.64. `test_oracle.equations` reads a chain's middle part's leading
  number as a running result ("a ÷ 2 = b × π = c"). The row
  "2 × 6 ÷ π = 12 ÷ π ≈ 3.82 cm" is an ordinary chain of equalities, so
  the picture column quotes its two steps separately, each a substring of
  the row. 13_3's row is quoted the same way.
- 13_2's prediction comes after the length is shown unchanged and the
  count has visibly changed (0 along, 1 bent), so the viewer weighs a
  changed count against a kept length. 13_3 answers by 12's additivity
  (each arm a 3 cm stick) before the sweep shows single counts moving.
- 13_4 shows the difference that matters: the same average, a different
  spread. The bent wire never misses because one arm always reaches
  3 ÷ √2 ≈ 2.12 cm across the lines, more than the 2 cm gap. That is
  checked at every direction in the mirror, and the closed form has no
  zero bin. The bars' chances come from `shares`, 3,600 directions, which
  agree with the oracle's closed form within 2.2e−7. Each shown
  tenth-percent lies at least 3.67e−3 points from an edge, about 160
  times the grid's error. 36 directions miss by about 1e−3, a control
  that a direction grid is an estimate. The 0.1% bar is 0.0013 card cm
  tall, so it is drawn at the 0.01 minimum. It is seen as a sliver with
  "0.1%" over it.
- 13_5's 4 is planar. Its pieces' lengths are kept every frame, and the
  line through its crossing counts twice, as 12_8 did. The oracle's
  negative control folds a 3 cm wire back on itself: counted per layer it
  averages 6 ÷ π, counted per point of the card 3 ÷ π, as if 3 cm long.
  Knots are not shown or claimed. The lower third says a wire bent out of
  the card is another question.
- "This is known as Buffon's noodle" is a naming claim. The lower third
  gives the source of the name, Ramaley's 1969 note, which `brief.md`
  records as bibliography checked, not read. No history of Buffon's needle
  is told: the brief cites no primary source for it.
- 13_7 closes on the coast: 7 at one placement, 8.22 on average, and
  π × the average count back to the length. The screen multiplies the
  four-place 8.2165, which rounds to the length: 8.2165 × π = 25.8129 and
  25.81 = 25.81. The two-place 8.22 × π would be 25.82, so the voice
  says the length, not a product.
- Pronunciation risks for qualification: "Buffon's" (BUFF-onz in English,
  bü-FAW(N) in French), "noodle", "sixty-four percent", "twelve over pi",
  "two over pi", "ten point seven percent". Ramaley is not spoken.
  Nothing has been listened to.


### Root review of 12/13 (2026-10-24)

The opening graph preserves chapter11's repaired fill-behind-curve order. Cue12_16 qualifies exact tangencies explicitly in voice and a lower third. The strict crossing model gives zero at the two-tangency placement; no random grid can prove otherwise. Source notes above are author history, not measured production timing. Full independent review and exact bindings are in narration_lab/third-wave-20261024/root/tend-2008.

## 14_the_same_width: The same width every way

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Think back to the coin. Wherever we dropped it, apart from the exact touching placements, it met the lines at two points. That was useful because we knew its rim. But what made its count so dependable? | Dim the same coast. Bring back the coin and its computed crossing dots. | persistent scene | |
| 14_2 | Press two straight rulers against opposite sides of the coin, keeping them parallel. The gap between them is the coin's width, two centimetres. Turn the coin. Different points of its rim touch the rulers, but the gap stays the same. | Rulers press inward, contact rings and computed 2.00 cm readout appear. A mark follows the coin through a full turn. | controlled rotation | hold |
| 14_3 | A circle has the same width whichever way you turn it. Is a circle the only shape that can do that? ⟨predict⟩ | Keep the coin and rulers visible while the viewer predicts. | think countdown | |
| 14_4 | No. Mark three points, each two centimetres from the others. They are the corners of a triangle with equal sides. Put the point of a compass on one corner, and draw an arc from the second corner to the remaining corner. Do the same from each corner, without changing the compass opening. Keep the curved outline. | Construct the three circular arcs around the same marked vertices, using a moving compass arm. | computed arcs | hold |
| 14_5 | This curve is called a Reuleaux triangle. Press the rulers against it. One touches a corner. The other touches the arc across from that corner. That arc was drawn from this corner, so the gap between them is its radius, two centimetres. | Rulers touch the actual support points. Highlight the corner and opposite arc contact; show the radius between them. | support points | |
| 14_6 | Now imagine turning this shape while keeping the rulers pressed against it. The corners will trade places with the arcs. Will the gap between them ever change? ⟨predict⟩ | Hold the constructed shape before any answer rotation. | think countdown | |
| 14_7 | It stays at two centimetres. Watch the rulers ride up and down as the shape turns. A corner touches a ruler while the opposite arc touches the other. Their separation stays equal to the compass radius. Constant width does not mean the shape stays still between fixed lines. It means the pressed rulers stay the same distance apart. | Turn through a half turn over twelve seconds. Compute both ruler positions, contacts and width at every frame. Explain the definition below. | coordinated representations | hold |
| 14_8 | The triangle inside it can't do that. Without its arcs, the gap shrinks to about one point seven three centimetres. As it turns, the gap comes back to two, and drops again. The curved sides make the difference. | Dim arcs, press rulers against the plain triangle, rotate through the wider pose and back. Readout changes from 1.73 to 2.00 to 1.73. | counterexample | hold |
| 14_9 | So the Reuleaux triangle shares the coin's constant width, despite looking very different. We have another shape whose width matches the spacing of our sheet. Can that tell us something about its rim, without following the rim? | Restore the same arcs and their rulers; keep coin and coast in view. | persistent objects | hold |

### Notes for 14

- Root completed the script around the preserved peer scene, rather than substituting the shorter planning draft. Both prediction questions precede their demonstrations.
- Width means the separation of parallel supporting rulers. Their positions are computed from arc extrema, not held fixed. The plain triangle is the negative comparison.
- The definition remains visible during the twelve-second turn, exceeding its character-based reading floor. Actual preview timing and layout require qualification.
- Reuleaux pronunciation remains a narration prerequisite; no pronunciation or listening acceptance is implied by silent pictures.

## 15_barbiers_floorboards: Barbier's floorboards

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | Now look at the sheet's lines. They're two centimetres apart, just as far apart as the rulers. One line runs between them. It has no dents, this shape, so that line enters, leaves, and crosses the rim twice. A straight line cannot leave and then enter again. That's what convexity gives us here. | Mark the sheet gap, highlight the interior line and its two computed rim crossings. Define convexity below. | computed crossings | hold |
| 15_2 | Slide it and turn it. The rulers move with the shape, but their separation is still the spacing of the sheet. The highlighted line can change. The count stays the same. | Carry the same shape through four placements, with computed support rulers, interior line, crossing dots and count. Hold at each placement. | controlled experiment | hold |
| 15_3 | As the sheet slides, a line can pass exactly through a corner. The adjoining arcs meet there, and our rule for counting their interiors needs special treatment for that one slide. And once in every gap, the lines lie along both rulers, and the rim only touches them. Each of those is one slide among a continuous range of slides. An exact exceptional slide has probability zero, so it does not change the average. | Slide a whole gap in three legs. Ring the corner, then the tangent contacts, and state the measure-zero qualification. | exceptional placements | hold |
| 15_4 | So like the coin, it averages two crossings. We already know how an average count measures length on this sheet. What does that make the length of its rim? ⟨predict⟩ | Show average count = length divided by pi, then average count = 2. Hold for prediction. | think countdown | |
| 15_5 | Two pi centimetres, about six point two eight. That is the same as the coin's rim. The corners and the curved sides look different, but the same width has led to the same length. | Solve 2 = length divided by pi. Put the resulting 6.28 cm beside the coin's rim label. | linked equations | hold |
| 15_6 | Check it another way. Each arc is centred on a corner. The radii to its ends make a triangle with equal sides, so its angles are each sixty degrees. Each arc is a sixth of a circle of radius two. Lift the three arcs off and lay them end to end. Together they make half of that circle. Its full circumference is four pi. So half of it is two pi again. | Reveal the radii and angle. Move the three intact arcs into a semicircle with a shared centre, preserving each radius and sweep. Show the arithmetic. | rigid arc rearrangement | hold |
| 15_7 | And a third way is to approach the rim with straight chains. The triangle itself is such a chain, six centimetres long. With six corners, the chain is about six point two one centimetres, and with twelve, about six point two seven. These lengths rise toward the rim from below. Each finite chain still cuts off the bends. | Return the arcs to the shape. Show inscribed chains with 3, 6 and 12 corners and computed lengths. | polygon bounds | hold |
| 15_8 | Barbier made the general point in eighteen sixty. For a convex shape of constant width, its perimeter depends only on that width. A circle of the same width gives the reference length, pi times the width. Ours is two centimetres wide. The statement concerns the boundary of a convex shape. Keeping the same supporting rulers in every direction while adding inward detours would add length. | Cite Barbier's original note. Compare with the coin and show length = pi times width. Replace the source with the convex-boundary qualification. | source and scope | hold |
| 15_9 | So counting crossings and laying out the arcs agree. The straight chains approach the same result. On our coast, the average count also recovered the length, about twenty-five point eight one centimetres. Constant width gave us an especially clear check. The counting rule reaches beyond that special shape. | Restore the coast's brightness and computed crossing dots. Show its 8.2165 average times pi giving 25.81, without mislabelling the current count as the average. | return to example | hold |

### Notes for 15

- Convexity is essential to two transverse crossings of an interior line. Tangencies and corner coincidences are isolated slide values, not a different average. Finite animation sampling does not prove an almost-everywhere theorem.
- Arc rearrangement proves the exact perimeter here; finite inscribed chains supply lower bounds. They are not an exact replacement for the arcs.
- Root rechecked Barbier's primary scan, section 8, printed pages 283–285: [original note](https://www.numdam.org/item/JMPA_1860_2_5__273_0.pdf). His equal-perimeter claim explicitly concerns convex curves with equally separated parallel tangents.
- Each definition/source card has an explicit full-visibility deadline of its body character count divided by 15 per second. Preview measurements must confirm the deadlines survive frame quantization. Prediction and final inspection holds remain separate.
- Barbier and Reuleaux require later pronunciation qualification. These are written words and silent scenes, not accepted narration.

## 16_two_ways_to_pick_a_line: Two ways to pick a line

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | So far, a random placement meant an even slide and an even turn of the sheet. That was a particular recipe for randomness. Here is another recipe that sounds fair, but changes what gets measured. | Keep the same coast on the card. Recall the even slide and even turn before replacing their labels with the new rule. | persistent card and coast | hold |
| 16_2 | Put two pins on the card. Each pin is spread uniformly over its area, independently of the other pin. Draw the full straight line through them, continuing beyond the pins to the edges of the card. Then pick fresh pins and repeat. | The same two gold pin objects and full line move through the first four retained draws. The line visibly extends past both pins. | seeded geometry | hold |
| 16_3 | To compare the recipes, replace the coast with two equal vertical sticks, each two centimetres long. Keep a stick in the middle and a stick near the corner. We will draw ten thousand independent pairs of pins and count how many full lines cross each stick. | Preserve the card; introduce the two computed equal sticks. Show the sample count, with both crossing totals still hidden. | controlled comparison | hold |
| 16_4 | Make a prediction before we draw the lines. Which stick will be crossed more often, the middle stick, the corner stick, or will they tie? ⟨predict⟩ | Both sticks and the stated sampling recipe remain visible through the countdown. No result or line stream has appeared. | think countdown | |
| 16_5 | Now run the experiment. Every pair supplies a full line, and every line contributes to the counts. The animation shows selected draws as the total advances. The calculation includes them all. | Reuse the pin and line objects for a twelve-second selected-draw stream. Count every retained draw; reveal only the number of pairs processed. | computed experiment | hold |
| 16_6 | In this run, two thousand one hundred and seventy-eight lines cross the middle stick. One thousand two hundred and seventy-three cross the corner stick. Equal lengths, in the same direction, but very different counts. Moving the stick on the card has changed its chance of being crossed. | Reveal the actual 2,178 and 1,273 totals at their phrases, with proportional bars on the same scale. Label them as this seeded run. | coordinated counts and bars | hold |
| 16_7 | Think about where the pins can go. A line with a long run across the card has more room for widely separated pin pairs than a line that clips a corner. Nearby lines inherit different amounts of room for pairs. Picking pins uniformly therefore favours some families of lines over others. | Show a long central chord and a short parallel corner chord on the same card. Place several illustrative pin pairs along each. This is intuition about the induced line distribution, not a probability calculation. | geometric motivation | hold |
| 16_8 | The line itself is infinitely long. What matters here is the part inside the card, where pins are allowed. And the result is specific to this pin recipe. Random needs a recipe. Equal sticks do not automatically receive equal average counts under every way of choosing lines. | Remove the illustrative lines while retaining the identical sticks and measured totals. Display the central lesson. | explicit scope | hold |
| 16_9 | Keep these sticks exactly where they are. Next we will change only how we choose the lines, returning to even slides and even turns. That comparison will show why specifying the randomness belongs inside the mathematics, before we average anything. | Preserve the card, sticks and totals for the next chapter. Show the finite-run and full-line conditions with a separate reading hold. | return to controlled experiment | hold |

### Notes for 16

- The retained PCG64DXSM stream has seed 20261025 and 10,000 independent uniform pin pairs on the 16 by 9 cm rectangle. The generator uses the existing NumPy oracle; Rust parses the round-trip coordinates and computes all strict interior crossings. An exact-rational checker is a separate geometric calculation, not an independent random-number generator.
- The full infinite line counts, even when the segment between its pins misses a stick. Endpoint coincidences and coincident pins are probability-zero exceptions under continuous sampling; the retained stream has no coincident pins.
- Prediction precedes the experiment, and both results remain hidden until after all draws are counted. Rapid pictures show selected draws; no claim that all 10,000 pairs were individually displayed.
- The long and short parallel chords motivate the induced bias. This finite experiment is not a proof of exact probabilities or a general rule that every line near a corner is unlikely.
- The prior scenes, original model and later draft remain intact. Silent picture timing and authored pause directions do not establish narrated synchrony or listening acceptance.

## 17_the_chart_of_lines: Give every line an address

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 17_1 | The pins favored the middle stick. What would the sheet's even slides and turns do instead? To make that question precise, let's give every line an address. Keep the same card and the same sticks. | Same rectangle and equal sticks, previous pin results at right, then the new question | persistent objects | |
| 17_2 | Start at the middle of the card. Draw a perpendicular to the line. Its direction is called the normal direction. The angle we record belongs to that perpendicular, not to the line itself. Watch them turn together. | One full line, teal perpendicular direction, angle arc and rightward reference turn together | controlled angle tracker | |
| 17_3 | The other part of the address is a signed distance. Positive means along that normal direction. Negative means the opposite way. A line through the middle has distance zero. Sliding the line changes this distance while its angle stays put. | Gold segment from center to perpendicular foot; line moves from positive to negative distance through zero, then returns | controlled distance tracker | |
| 17_4 | Put the normal angle across a chart, from zero to half a turn. Put the signed distance up and down. Now a line on the card is a dot on the chart. Turn the line, and the dot moves sideways. Slide it, and the dot moves vertically. | Chart axes, bounded half-diagonal distance range, gold dot linked to the physical line; controlled turn then slide | coordinated representations | |
| 17_5 | A sheet has parallel lines, all with the same normal angle. They become a column of dots. Equal gaps become equal vertical spacing. Slide the sheet and the dots move up their column. Turn the sheet and the column moves across. | Five selected parallel sheet lines and equally spaced chart points move together; explicit 2 cm gap label | persistent column, linked trackers | |
| 17_6 | Some addresses describe lines that miss the card entirely. This curved boundary keeps just the addresses that meet it. A dot outside the boundary is a missed card. Notice that the allowed distance range changes with the angle. | Computed plus/minus rectangular support curves bound the chart; a red outside point is rejected | computed boundary, inspection hold | |
| 17_7 | Here is our new rule. Pick a dot uniformly in the whole chart box. If its line misses the card, discard it and pick both coordinates again. Inside the boundary, equal areas have equal chances. Taller columns contain more accepted dots, so the accepted angles alone are not evenly distributed. | Card/sticks persist, chart boundary and equal-area rule remain; no experimental result yet | explicit joint law and rejection | |
| 17_8 | With that rule, count ten thousand accepted lines. Make a prediction before we run it. Will the middle stick still win, or should the equal sticks come out about even? ⟨predict⟩ | Same sticks and chart region, counters and outcomes hidden, countdown | prediction hold | |
| 17_9 | Each accepted dot gives a line, and we test that full line against both sticks. The display shows selected draws, but the count uses every accepted line. | Native 10,000-line stream drives physical line and chart dot; full counter, selected-draw disclosure | computed experiment | |
| 17_10 | Seven hundred and ninety-six cross the middle stick, and eight hundred and eleven cross the corner one. About even in this run. That small difference is compatible with chance. The chart gives us a way to explain why equal sticks have equal chances under this rule. | Measured 796/811 revealed only after experiment; first 1,000 accepted chart points shown | measured counts, finite-sample qualifier | |
| 17_11 | Compare the two recipes on the same chart. The uniform rule spreads its dots across the allowed region. The pin rule piles more dots near the middle band, where lines have longer runs through the card. These are samples from different distributions, even though both recipes sound random. | Hold white accepted scatter, replace with gold pin scatter using retained inputs; same axes/card/sticks | controlled distribution comparison | |
| 17_12 | Before averaging over random lines, say how you chose them. The sheet's even slides and turns give equal weight to equal areas of this chart. Next, we'll see what area belongs to the lines that cross a stick. | Same chart, pin scatter and sticks persist; precise joint-law and finite-sample condition | callback and next question | |

### Notes for 17

- Root expands the retained draft to define the normal rather than conflating it with the line direction. Half a turn is half-open to avoid double-counting the same unoriented line; signed distance distinguishes its two sides.
- The accepted sampler is uniform in the joint angle-distance region of lines meeting the rectangle. Its angular marginal is proportional to projected width. Drawing a uniform angle and then a uniform distance in that angle's range would be a different law.
- The native input stream continues the original PCG64DXSM seed after 40,000 uniforms consumed by the pin experiment. All 10,000 accepted lines are counted; only selected draws and the first 1,000 scatter points are displayed.
- The prediction precedes the experiment and its reveal. Empirical scatter and 796/811 are not proofs of the exact measure or equal crossing probability; the next chapter earns the area argument. No narration or listening acceptance follows from silent timing.

## 18_the_sticks_band: The stick's band

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 18_1 | Put away the short test sticks and bring back our six centimetre stick. Keep the same card and the same chart. Which addresses belong to lines that cross this stick? | Persistent card and chart; short sticks fade, earlier six-centimetre violet stick returns at centre | identity and callback | |
| 18_2 | Fix a normal angle. Each endpoint of the stick has a projection onto that normal. If a line's signed distance lies between those projections, it crosses the stick. Outside that interval, it misses. Watch the line slide through the whole interval. | Endpoint dots, physical full line and matching chart dot; highlighted interval at a fixed angle, line slides through and beyond | computed endpoint projections | hold |
| 18_3 | Now turn the normal. The endpoint projections trace the edges of a band. Every address between those edges belongs to a crossing line. The height of each column is the stick's shadow in that direction, the same shadow we measured earlier. | Live normal angle sweeps; computed violet band and boundary curves appear; coordinated physical line and chart column | coordinated representations | hold |
| 18_4 | We already added these shadow widths over a half turn. With angle measured in radians, the total is twice the stick's length. For our six centimetre stick, the band has area twelve. This is chart area, not an area on the card. | Recall shadow integral through computed area12.00; label centimetres times radians | computed identity and units | hold |
| 18_5 | Keep the length and direction fixed, and imagine sliding the stick to the right. Make a prediction before it moves. What happens to its band, and does the chart area grow, shrink, or stay the same? ⟨predict⟩ | Hide band and answer, keep stick/card/axes; clear-space prediction ring | prediction | |
| 18_6 | Each endpoint gains the same projection of the movement. So the boundaries in each column rise or fall together. The band bends as it shifts, but every column keeps its height. Add those unchanged heights. The area is still twelve. | Persistent stick translates; live endpoint boundaries/fill/readout computed from tracker geometry | controlled translation | hold |
| 18_7 | Return the stick to the centre. This time, imagine turning it about its centre, keeping its length. Predict before we turn. What happens to the band, and will its area change? ⟨predict⟩ | Return to centre then hide band/readout; separate prediction ring | prediction before rotation | |
| 18_8 | Different angles now see the different shadows. The width pattern moves across the chart, but adding it over a complete half turn gives the same total. The area stays twelve. Turning changed which lines meet the stick, not how much chart area they occupy. | Centred stick rotates while its computed band and live area move together | controlled rotation | hold |
| 18_9 | There's a catch at the chart's edge. After a half turn, the normal points the opposite way. The same physical line then has the opposite signed distance. Watch the dot wrap to the other edge while the line moves continuously. Those edges name the same lines, with reversed distance signs. | Same gold line rotates continuously through the seam; chart dot jumps from upper right to lower left; explicit sign-flip label | seam convention | hold |
| 18_10 | Now put the pin dots back. This sampling recipe crowds some chart regions more than others. Moving our stick moves its band through that fixed distribution. It does not move the pin distribution with it. Equal band areas can therefore have different chances. These sampled dots illustrate the crowding. They are not an exact probability calculation. | Fixed retained gold pin scatter, labeled displayed subset; same vertical band translates and returns; no universal strict-loss claim or new numerical pin estimate | controlled sampling-law comparison | hold |
| 18_11 | Under the uniform chart rule, equal lengths give equal band areas and equal hit probabilities, provided the whole stick stays inside the card. That explains why position need not matter. Next, replace the stick with our coast. A line may cross a curve repeatedly, so its address will need a crossing count, not just a yes or no. | Violet band remains; precise conditions with full reading hold; motivate next chapter's multiplicity | conditions and transfer | hold |

### Notes for18

- Endpoint intervals compute the band, not a decorative sine curve. Translation shifts each column by the normal projection of the displacement, leaving its height unchanged. Rotation is demonstrated about the centred stick; the edge wrap is shown separately and includes reversal of signed distance.
- The area uses radians and the earlier projection-width integral. The live readout computes twice endpoint distance and is checked against numerical integration and the independent oracle. Equal hit probability is conditioned on the whole stick remaining inside the fixed card.
- The older draft's unrestricted "always catches fewer" pin sentence is not used. The fixed nonuniform distribution is compared to the moving band; no strict-loss theorem, finite-count probability proof or rotational monotonicity is claimed.
- Scratch timing and authored pause directions do not establish narrated synchronization or listening acceptance.

## 19_the_coasts_chart: Every crossing counts

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 19_1 | Replace the stick with our coast, keeping the same card and the same chart. The coast bends back on itself, so knowing whether a line meets it is no longer enough. We need to know how many crossings belong to that address. | Same16×9 card and chart; violet stick fades and original blue coast returns | recurring example | hold |
| 19_2 | Start with a line that misses the coast. Its address gets zero. Move to a line that crosses once. Its address gets one. The gold dot on the chart names the entire line. The gold dots on the coast mark its crossings. | Selected clear lines with computed0 and1 crossings; physical full line, chart address and dots agree | coordinated geometry | hold |
| 19_3 | Here is a line with six crossings. Follow it across the coast and count the gold dots. Its address needs a stronger color because it contributes all six crossings. Replacing that count by a yes would throw most of its contribution away. | Selected six-crossing line held; actual native intersection dots and computed counter | multiplicity | hold |
| 19_4 | Repeat this calculation across the chart. This picture uses three hundred and sixty angle columns and a hundred and eighty-four distance rows. Each little patch gets the count at its centre. The colors show a finite grid approximation, with blank patches for misses. The outline still marks the lines that meet the card. | Actual360×184 midpoint grid; vertically merged same-count cells preserve exact image;0–6 legend and resolution label | computed heatmap | hold |
| 19_5 | In this grid, about thirty-five percent of the lines across the card miss the coast. Among the lines that do meet it, about seventy-one percent cross once. Keep those groups separate. And six is the largest count found on this grid. This picture alone does not prove a bound for every possible line. | Grid labels explicitly35% of all accepted midpoints versus71% conditional on hit; sampled maximum scoped | denominators and limits | hold |
| 19_6 | To total the chart, multiply each little patch's area by its crossing count, then add. A patch marked six contributes six times its area. We're adding chart area with multiplicity, not measuring the area inside the coast, and not merely counting colored patches. | Actual selected six-crossing line and chart address return; area-times-count label | weighted area | hold |
| 19_7 | The stick's band totaled twice its length. Think about what should happen when the stick becomes this coast. Make a prediction before we add the patches. What should the coast's weighted chart total be? ⟨predict⟩ | Same coast and chart; total hidden; clear-space countdown | prediction | |
| 19_8 | Each column adds its own crossing counts, multiplied by the same patch area. This is a numerical sum, with the grid's limited resolution. Sweeping through the grid gives approximately fifty-one point six three. | Sweep360 actual column sums before result phrase; live five-decimal readout rounds to51.63 on completion; approximation label | computed accumulation | hold |
| 19_9 | Twice the coast's exact length also rounds to fifty-one point six three. Look at the extra digits. The grid and the exact calculation differ slightly. Their agreement illustrates the identity. It does not turn a finite grid into a proof. | Side-by-side grid51.62530 versus twice exact length from model, both explicitly approximate displayed decimals | estimate versus identity | hold |
| 19_10 | Why does twice the length appear? Break the coast into small pieces. For each line, every piece contributes its crossings, including places where the coast doubles back. Adding over all lines adds those contributions too. Each straight piece has the band rule we already found. The precise extension to finite-length curves uses a limiting argument. The picture is its intuition. | Highlight actual11 model pieces sequentially on persistent coast; chart remains; additive statement | proof intuition with limit disclosed | hold |
| 19_11 | This is the Cauchy Crofton length formula, associated with Morgan Crofton's work in eighteen sixty-eight. It measures length by adding crossings over the space of lines. The earlier shadow argument and this chart are descriptions of the same geometry. | Name/year source caption with reading hold; original coast and multiplicity chart persist | attribution | hold |
| 19_12 | Precisely, the curve must have finite length. Count every traversal. Tracing a path again adds length and adds crossings again. Use signed distance and a full half turn of normal angles, measured in radians, with equal chart areas weighted equally. For our coast, exceptional tangencies and joints occupy zero chart area. No uniform probability over the entire unbounded plane of lines is needed. Next we'll write this rule as a formula. | Precise finite-length/path-multiplicity/dp dθ conditions, with full reading hold | exact scope | hold |

### Notes for19

- Displayed cells are midpoint samples,360×184 over normal angle[0,π) and signed distance within the actual16×9 card's half-diagonal box. The conditional denominator includes just midpoints whose full line meets the card. Counts/area/rounded percentages come from native geometry and are independently checked by the oracle's monotone-projection interval method. The grid's maximum6 is not promoted to a universal maximum theorem.
- The unchanged pre-existing `chart.miss_percent`/`chart.once_percent` use a finer direction integration, with closed-form oracle comparison. New `chart.grid_*` keys explicitly own this image's finite estimates; neither method's provenance is relabeled.
- The exact identity is one half the integral of intersection multiplicity over signed p∈R and normal angleθ∈[0,π), measure dp dθ. For a parametrized rectifiable path, count parameter traversals; for a geometric rectifiable set with no retracing, count intersection points and use its geometric length. Exceptional addresses in this piecewise line/arc example do not affect the integral. Finer cells alone are not a general proof for arbitrary rectifiable paths; the general area/coarea theorem supplies the extension.
- Source attribution: retained brief bibliography, M.W.Crofton, Philosophical Transactions158(1868),181–199, DOI10.1098/rstl.1868.0008. Royal Society archive search separately identifies the paper and30March1868 referee report. The publisher full text was unavailable in this tick; no fresh full-paper-reading claim. Modern theorem checked in Sonia Reilly's2016 MIT research paper,p.7(print), using a full-turn convention with factor1/4; this chapter uses half-turn signed distance and factor1/2. No priority claim excluding Cauchy/Barbier, and no pronunciation acceptance.
- Source links: https://doi.org/10.1098/rstl.1868.0008 ; https://math.mit.edu/documents/rsi/2016Sonia_Reilly.pdf . The limited source checks support attribution and scope, not the numerical results, which come from executable geometry.
- Scratch timing and authored directions are not narrated synchronization or listening acceptance.

## 20_the_formula: The formula

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 20_1 | Keep the coast and its chart in view. We have already done the work the formula describes. Label each address by its crossing count, multiply by the little patch's area, and add. The symbols are a compact way to ask for that total without choosing a grid size. | Re-establish the same16×9card, coast and computed360×184chart; explicitly label finite grid shading. | persistent model and chart | hold |
| 20_2 | Theta and p locate one full line, its normal angle and its signed distance. The letter n records how many times that line meets the path. Follow the gold address to the gold crossing on the coast. That is what the expression inside the integral means. | Actual selected one-crossing line and crossing linked to chart address; display n(theta,p). | coordinated representations | hold |
| 20_3 | First hold the angle fixed and move through every signed distance. That reads one column. Add the crossing counts down the column, each weighted by its small step of distance. The result is the path's total projected travel in that direction. If the path doubles back, the overlapping shadows are counted again. | Highlight a chart column with inner integral. Retain physical coast and count shading. | column and projection interpretation | hold |
| 20_4 | Now add those columns over a half turn, measuring the angle in radians. That is the outer integral. Finally multiply by one half, because the weighted chart total is twice the length. We earned that factor from the stick's band. It is still the same calculation. | Complete L=1/2 double integral with p over real values and theta0..pi. Sweep chart column across the half turn. | linked formula and scan | hold |
| 20_5 | This is an exact identity for a path of finite length, with repeated traversals counted in both length and crossings. It contains no random choice. A finite colored grid approximates the integral. Randomly placed sheets are another way to estimate it. Neither approximation becomes exact just because the identity is exact. | Exact-identity label over retained finite grid; retain full formula for inspection. | exact versus estimated | hold |
| 20_6 | Divide the total by pi instead, and you have the average total shadow over directions, two over pi times the length. Count overlapping shadows separately. Barbier reports that Cauchy's projection result appeared in eighteen thirty-two. For our coast the average is about sixteen point four three three centimetres. | Average-shadow integral formula and computed16.433. Credit Cauchy1832 explicitly as reported by Barbier1860. | scoped attribution and average | hold |
| 20_7 | Try the formula on our coin, two centimetres across. Its rim is about six point two eight centimetres. Make a prediction before we fill its chart. What should the weighted total be? ⟨predict⟩ | Same card; coast yields to centred coin. Empty chart and rim label; countdown precedes result. | prediction | |
| 20_8 | Twelve point five seven, approximately. Twice the rim. Now let's see that answer in the chart, rather than merely doubling a number. | Reveal computed weighted total12.57 while coin and chart persist. | return to prediction | hold |
| 20_9 | A line strictly less than one centimetre from the centre crosses the rim twice. Slide this line through the interior. Its gold address moves within the band, and the crossing dots move along the same circle. Turning the line changes neither the band height nor the count inside it. | Exact moving line and two computed intersections linked to signed-distance chart dot. Fill centred band across half turn. | controlled translation | hold |
| 20_10 | Right on the edge, a tangent only touches the rim. There is one contact. Farther away there is none. The tangent addresses sit on the band's boundaries, which have zero area in the chart. They do not alter this integral. | Stop at exact tangent and show one gold contact plus boundary; then outside line/address with no contact. | exceptional cases | hold |
| 20_11 | The band is two centimetres tall and pi radians wide. Multiply its area by the interior weight two to get four pi. The formula takes one half of that, giving two pi centimetres, the coin's rim, about six point two eight. | Band dimensions, symbolic2pi area, weight2 giving4pi, then1/2 returning2pi. | area and normalization | hold |
| 20_12 | Return to the earlier pictures. The coin gives two crossings per sheet. A stick's average total shadow is about sixty-four percent of its length. The coast averages about eight point two two crossings per sheet at our two-centimetre spacing. These are different quantities, connected by the same length identity. | Separate callback labels identify per-sheet counts versus mean projection fraction and sheet spacing. | return with denominators | hold |
| 20_13 | The precise scope matters. Use a finite-length plane path and count every traversal. Use signed distance across the whole real line and a half turn of normal angles in radians. Exceptional tangent lines do not carry chart area. The formula is exact. Our finite estimates still have error. | Precise lower third; hold after fully visible for complete reading. | theorem conventions | hold |

### Notes for 20

- The centred coin radius, band area and weighted total come from the same model; strict interior/tangent/exterior counts come from analytic line-circle intersections. No tolerance thickens the tangent set.
- Native continuation uses the existing finite midpoint chart as a picture, explicitly distinguished from the exact integral. Repeated traversals count separately in both projected travel and length; projection is not silhouette width.
- Primary source checked this tick: Barbier1860, printed pages276–277, https://www.numdam.org/item/JMPA_1860_2_5__273_0.pdf . Barbier states the sum of projections relation and reports a1832 lithographed publication. Cauchy's original memoir was not read; no priority or pronunciation acceptance. The source supports the scoped historical report, not the general theorem proof.
- Modern finite-length/path interpretation and half-turn versus full-turn normalization continue chapter19's explicit conventions. The displayed symbolic formula is exact; numerical labels are rounded approximations.
- New13 cues supersede only this chapter's old6-cue draft. Actual spoken synchrony and name/pronunciation qualification remain open.

## 21_ten_drops: Ten drops
| id | words | picture | features |
|---|---|---|---|
| 21_1 | The identity averages over every placement. A measurement uses a finite sample. Let's return to our coast and try ten drops of the ruled sheet. | Keep the original coast and card; recall exact length with approximation sign. | Inspect the same object before sampling. |
| 21_2 | Each time, choose a direction uniformly over a half turn. Independently, choose a slide uniformly over one gap. Make fresh independent choices for the next drop. Do not choose placements because their counts look good. | Persistent coast; separate direction and slide rules; label retained seed. | Read both independence conditions. |
| 21_3 | Watch the sheet change its direction and slide while the coast stays fixed. The gold dots mark the actual crossings. Keep each count in the row, so we can inspect the complete experiment before taking an average. | Show the exact retained ten placements, clipped sheet lines and native crossing dots; accumulate the count row. | Hold every placement for inspection; leave the full row. |
| 21_4 | Their mean is eight point two crossings. At our spacing, multiply by pi centimetres per crossing. The estimate is twenty-five point eight centimetres. That agrees with the true length to the displayed decimal place. | Compute the mean from the retained counts; show unrounded computation rounded once, beside the exact-length label. | Inspect the agreement without calling it precision. |
| 21_5 | Make a prediction. Should we expect ten drops to be this close whenever we try again? ⟨predict⟩ | Keep the count row and estimate through the think ring. | Prediction before the answer. |
| 21_6 | No. This is a fortunate sample. Across a thousand retained drops, the counts range from five to twelve. Those bars describe that sample. The gold outlines show the numerical model's probabilities. It also includes thirteen crossings, with a very small probability, about zero point zero three six percent. | Same coast remains; show frequency bars and model outlines on one common scale. Call out the tiny thirteen bin; sample has none. | Separate observed range from possible outcomes in the model. |
| 21_7 | To describe the variation, convert each drop's count into a length estimate. The model's standard deviation is about four point four centimetres. This measures spread around the true length. Square the errors, average them, then take the square root. It does not say that every error fits inside this bar. | Show a centered spread bar; label SD and explain the root-mean-square error meaning for this unbiased estimator. | A spread is neither a bound nor a confidence interval. |
| 21_8 | Averaging ten independent drops reduces that standard deviation to about one point four centimetres. Some errors are smaller, some larger. We have improved the method, but our particularly close opening result still overstates its usual precision. | Reveal the narrower second bar on the same scale; mark both as plus-or-minus one SD only. | Inspect both widths on the common scale. |
| 21_9 | Now repeat the whole ten-drop experiment a thousand times. Each square is a fresh group of ten, not an individual drop. Color it red if the resulting estimate is more than one centimetre from the true length. In this retained experiment, forty-three percent of the groups are red. | One thousand squares in retained order, computed from all ten counts in each group; red only for strict error greater than one centimetre. | Hold the group interpretation and error threshold. |
| 21_10 | The numerical probability calculation also rounds to forty-three percent. It adds independent counts. Multiply their probabilities, collect the ways to reach each total, then add the probabilities outside the allowed error window. Its input probabilities are numerically computed. Agreement here does not turn a finite sample, or a numerical calculation, into an exact probability. | Keep actual group squares and sample percentage; add separately labeled approximate model percentage and total-error window. | Distinguish sample from model and approximate from exact. |
| 21_11 | So keep our opening estimate, but remember its luck. An exact identity can justify an unbiased measuring procedure without making a small experiment precise. Next we can ask how much more repetition buys us. | Return to the first-ten estimate beside the original coast; retain precise sampling and numerical qualifications. | Full reading hold for the condition. |

The data use the existing oracle's PCG64DXSM streams: seed20261024 for1000 single drops and seed20261026 for1000 independent groups of10. Native drawings use retained round-trip coordinates, not invented illustrative counts. The single-drop distribution is a1800-direction midpoint calculation with exact interval weights within each direction. The oracle partitions angle events and integrates trigonometric widths in floating point, using a midpoint approximation for tiny angle intervals. Convolution is a numerical evaluation from those probabilities. The retained historical key `drops.ten.far_exact_percent` is not an exactness claim. Standard deviation is the population-model spread of the estimator, not the finite sample's sample standard deviation or a coverage guarantee.

## 22_a_thousand_drops: A thousand drops

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 22_1 | Keep the same coast, and keep going to a thousand drops. Each fresh drop chooses a uniform direction over a half turn, and independently a uniform slide over one gap. The choices are independent of all earlier drops. | Persistent same coast and 16 by 9 card; independent choices stated. | persistent objects | |
| 22_2 | Follow the running estimate, from the first ten to all thousand. Each new crossing count joins the total, and we divide by the number of drops used so far. The curve wiggles. More data need not improve every individual step. | All retained prefix means, n from10 to1000; persistent trace and moving pointer; exact coast length reference. | computed experiment, tracker | inspect |
| 22_3 | This retained run ends at twenty-five point nine centimetres. That is one observed average. It does not tell us how far another run might wander, and it does not certify this run's error. | Reveal final retained estimate, explicitly observed. | phrase-bound reveal | hold |
| 22_4 | Imagine starting over again and again, always averaging a thousand independent drops. The numerical population model gives those averages a standard deviation of point one four centimetres. Recall what that means. Square their deviations from their mean, average those squares, then take the square root. It is a measure of spread, not a promise that an estimate lands inside an interval. | Separate model SD0.14 from observed25.9; definition in words. | coordinated representations | inspect |
| 22_5 | Why does averaging help? First centre each drop's estimate by subtracting its mean. Independence makes the cross-products of different centred deviations average to zero. So the variance of their sum is n times the variance of one drop. Variance is the average squared deviation we just described. | Same coast persists; explain cross terms then sum variance. | derivation with motivation | hold |
| 22_6 | Now turn that sum into an average by dividing by n. Squared deviations are divided by n squared. That leaves variance divided by n. Taking the square root gives standard deviation multiplied by one over the square root of n. | nσ²/n²=σ²/n, then SD=σ/√n. | coordinated symbolic explanation | hold |
| 22_7 | That is why the spread shrinks slowly. With four times as many drops, the standard deviation is half as large. Follow the gold curve. It describes repeated independent averages under the model, not the wiggles in our retained run. | Computed SD curve n100..2200 and100/400 example, clearly model. | computed plot | inspect |
| 22_8 | Make a prediction. To bring the standard deviation down to a tenth of a centimetre or less, about how many independent drops would we need? ⟨predict⟩ | Target SD0.1 line on existing curve; answer hidden. | think countdown | predict |
| 22_9 | About two thousand. Using the unrounded population standard deviation in the numerical model, the first whole number that meets the target is nineteen sixty-five. We square the ratio of the single-drop standard deviation to the target, then round up. This is a target for spread, not a confidence guarantee. | Reveal1965 point and ceiling rule on same curve; numerical input qualification. | phrase-bound result | hold |
| 22_10 | Return to our coast. The retained estimate and the model's standard deviation answer different questions. The integral identity has no sampling variation. A finite random estimate does. Keeping those statements separate tells us what our measurement can actually promise. | Observed estimate, populationSD and Crofton identity beside same coast; precise independent-drop/SD/numerical conditions. | callback | inspect |

### Notes for 22

- Uses the exact retained seed20261024 placements from chapter21; no new draws. The running trace displays every prefix from10 through1000. Native geometry computes counts.
- SD=σ/√n holds for independent identically distributed finite-variance drops. Covariance terms vanish for centred independent deviations; independence is sufficient, not necessary. The numerical one-drop SD comes from the existing1800-direction midpoint count distribution. The oracle uses event-angle integration, also numerical. Neither supplies a rigorous interval error certificate for the threshold.
- The numerical threshold uses unroundedSD, not the displayed4.4. Check1964 fails and1965 passes. It is a minimum for the stated numerical model, not a confidence sample-size claim.
- The identity uses finite-length rectifiable paths with traversal multiplicity, signed distance, half-turn angles in radians and negligible exceptional tangent/boundary lines, as established in chapter20.
- Scratch-voice timing and authored pause directions do not establish actual speech timing or listening acceptance.

## 23_fixed_directions: Fixed directions

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 23_1 | Could we replace random turns with a small, planned set of directions? Keep our coast. Choose four directions, equally spaced over a half turn, horizontal, diagonal, vertical, and the other diagonal. There is a crucial condition. In each direction we must still average over every slide, uniformly across a full gap. | Same coast and card; directions0/45/90/135; full slide average condition. | persistent objects | inspect |
| 23_2 | Watch the sheet slide while its direction stays fixed. The crossing count changes as lines pass the coast. Average that count over the entire slide, then do the same in each remaining direction. These displayed averages come from the computed projection travel, not from counting the few animation frames you see. | Actual sheet and computed crossing dots; full-gap sweep at each fixed direction; analytical slide means. | computed geometry, controlled changes | inspect |
| 23_3 | Average those direction results and apply the same crossing calibration. Our coast comes out at twenty-five point five three centimetres, about one point one percent low. That looks encouraging. But does this particular coast hide a much worse problem for a differently oriented path? | Reveal25.53 and -1.1% against exact model length. | phrase-bound reveal | hold |
| 23_4 | Start with something simpler, the same straight stick, six centimetres long. For each direction, the slide-averaged count measures the stick's projection across the lines, divided by their spacing. The bars show those projection lengths. As the stick rotates, its true length stays fixed, but the average of these projections changes. The graph tracks the resulting percentage error. | Same stick rotates; four projection bars and computed error graph share angle tracker. | coordinated representations | inspect |
| 23_5 | When the stick lines up with a chosen direction, its projection across those lines vanishes. That is the low point. Turn halfway toward the next chosen direction, and the projections balance differently. The total reaches its high point. The pattern then repeats. Averaging slides removes the accident of where the sheet lands. It does not remove the effect of these chosen directions. | Return to aligned orientation, then22.5 degrees; low and high points on same graph. | deliberate inspection | hold |
| 23_6 | For these four equally spaced directions, the extremes are about five point two percent low and two point six percent high. Those percentages are rounded, so treat them as approximate descriptions. The exact factors come from summing the projections. These bounds hold for every orientation of the stick. | Rounded extrema from exact trigonometric expressions; same computed trace. | model-bound numbers | inspect |
| 23_7 | Why does a stick help with a coast? For a polygonal path, add the projection contributions of its straight pieces. The ratio for the whole path is a length-weighted average of the pieces' ratios, so it cannot escape their common bounds. For a finite rectifiable curved path, the corresponding integral adds the local tangent contributions. Count the whole path, including repeated traversals. That is the scope of the claim. | Coast pieces highlighted; length-weighted local ratios explained; finite rectifiable traversal scope. | geometric explanation | hold |
| 23_8 | Make a prediction. Suppose we keep only two directions, at right angles, still averaging every slide in each. How much worse could the orientation error become? ⟨predict⟩ | Same coast; two-direction question before answer. | think countdown | predict |
| 23_9 | About twenty-one point five percent low, or eleven point one percent high. The same rotating stick reaches both extremes. A diagonal stick gives the high point. A stick along either chosen direction gives the low point. Again these rounded percentages describe the full slide averages, not a guarantee for an individual placement. | Two bars and new error curve; aligned/diagonal/aligned controlled rotation. | computed comparison | inspect |
| 23_10 | With just one direction, a parallel stick is missed for almost every slide. Its estimate is a hundred percent low. At the opposite extreme, a perpendicular stick gives an estimate about fifty-seven percent high. Choosing directions matters even when we have averaged away all the slide variation. | Parallel stick with noncoincident horizontal guide lines; one-direction range. | limiting example | hold |
| 23_11 | Return to our coast and its planned directions. The estimate remains useful because we can state exactly what the method controls, orientation error after full slide averaging. If we use only a few actual slides, their sampling error is a separate issue. A small error for this coast is an example. The geometric bounds explain what survives when the path changes. | Return25.53; precise factors and measure conditions; callback. | scoped conclusion | inspect |

### Notes for 23

- For m equally spaced unoriented line directions, the slide-averaged segment estimate divided by length is (π/(2m)) times the sum of absolute sines of its relative angles. Put t=π/(2m). On a sector0≤α≤π/m the sine sum is cos(α-t)/sin(t), so the exact lower/upper factors are t cot(t) and t/sin(t), with lower0 for m1. Percentages are nearest-rounded, not outward certified bounds.
- Full slide averaging means uniform integration across one complete spacing period in each direction; all intersections of the whole finite rectifiable path are counted with traversal multiplicity. Almost-everywhere tangent/coincident exceptions do not alter the slide integral. It is not a claim about an individual placement, finite-sample error, or a nonrectifiable coast.
- The coast consists of the existing exact straight and circular-arc model. Native travel uses analytic integrals; the existing oracle uses numerical integration. Animation samples illustrate geometry but are not themselves the averaging calculation. No new random draws.
- No historical-priority or pronunciation claim is introduced. In particular the original Cauchy memoir has not been read. Scratch timing and pause coverage are not narrated synchrony or listening acceptance.

## 24_the_longimeter: The longimeter

**Picture.** Keep the same coast and card. Assemble three colored square grids, inspect projection travel and the orientation average, predict the spacing, then translate the calibrated sheet with actual crossing dots and count. Return to the analytical slide average and its residual directional bias.

| ID | Spoken words | Picture / time |
|---|---|---|
| 24_1 | Six equally spaced directions reduce the directional error to about two point three percent low and one point two percent high. After full slide averaging, our coast reads twenty-five point seven three centimetres. Can we turn that idea into a tool you could lay on a map? | Same coast; six-direction estimate and rounded bounds. |
| 24_2 | Start with a square grid. Add another turned thirty degrees, and another turned thirty degrees the other way. Together their lines run in six directions. This is the arrangement in Chris Staecker's reconstruction of the Steinhaus longimeter. Our drawing uses its directions, with our own illustrative placement. | Build three colored grids; show distinct family directions. |
| 24_3 | We want the crossing count to read the map length directly, in millimetres. But first, remember what an average means here. For a fixed direction, average over a full gap of slides. The answer is the coast's total projection travel across the lines, divided by the gap. It depends on direction. | Across-line travel divided by d; fixed-direction condition. |
| 24_4 | Only after also averaging uniformly over orientations does that projection travel become two times the length over pi. Divide by the gap, and the mean count for a line family is two L over pi d. Here L is the path length, and d is the line spacing, both measured in millimetres. | Reveal orientation average and units after projection explanation. |
| 24_5 | Our sheet has six line families. Turn the whole sheet uniformly, and average its position so each family's offset is uniform over a gap. Adding the family means gives twelve L over pi d. The families need not be independent for their expectations to add. | Add six equal orientation means; distinguish one sheet from independent drops. |
| 24_6 | Now choose the spacing. We want the expected count to have the same numerical value as the length in millimetres. Make a prediction. What gap makes that happen? ⟨predict⟩ | Inspect twelve L over pi d; five-second prediction. |
| 24_7 | Twelve over pi millimetres, approximately three point eight two. Substitute that gap into the denominator. The calibration factor cancels, leaving the numerical length in millimetres. That's why counting can act like a ruler. | Cancellation; exact gap versus rounded printed value. |
| 24_8 | The exact calibration uses twelve over pi. Printing the rounded spacing introduces a small additional scale error. And neither calibration removes the variation in an individual placement. Let's slide our exact-spacing sheet over the same coast. Watch the crossings appear and disappear. | Actual rigid translations and calculated crossing dots/count; retain each selected placement. |
| 24_9 | The coast hasn't changed. The count has. These are selected placements, not a random experiment or a guarantee about the size of that variation. Count every crossing along the whole path, including repeated traversals. A mere touch at a tangency is an exceptional placement, not a crossing through the line. | Three inspected selected positions; unchanged coast. |
| 24_10 | Keep these directions fixed and average fully over each family's slide. Our coast's calibrated reading is two hundred fifty-seven point two seven millimetres. Its actual length is about two hundred fifty-eight point one three. That remaining difference is directional bias, separate from the changing count at individual placements. | Side-by-side analytical slide mean, true length and placement count. |
| 24_11 | So the longimeter packages the geometry into a practical counting sheet. More directions control directional bias after slide averaging. Turning and shifting provide the averaging behind the calibration. A particular placement still gives an estimate, and the map's scale is a separate conversion to distance on the ground. | Same coast, grid, precise scope and source attribution; deliberate reading holds. |

**Scope and sources.** Finite rectifiable path, full traversal multiplicity, all crossings. Fixed-direction means require uniform full-gap offsets; full calibration additionally averages sheet orientation uniformly over a half turn. Offset marginals suffice for linearity of expectation; family independence is not assumed. Three-grid directions and rounded 3.82 mm spacing observed in C. Staecker's own reconstruction source, https://faculty.fairfield.edu/cstaecker/files/machines/longimeter.tex and page https://faculty.fairfield.edu/cstaecker/machines/longimeter.html. Its rotation centers differ from our illustrative common-origin grids. Original Steinhaus 1930/1931 papers have not been read; no date, priority or pronunciation acceptance. The published -2.26% lower claim is not copied; our analytic six-direction bound rounds to -2.3%.

## 25_the_gap: The gap

**Picture.** Keep the coast and its card fixed. Spread the same line family from two to four to eight centimetres, with actual crossing dots. Predict whether calibration preserves the mean. Compare retained histograms and numerical population probabilities on one length axis. Return to the original divider walk, predict a smaller step, then show the nonmonotone exception with actual computed landings.

| ID | Spoken words | Picture / time |
|---|---|---|
| 25_1 | A counting sheet also needs a spacing. Return to our original parallel lines, two centimetres apart. Widen the gap to four centimetres, then eight. The coast stays exactly where it was. There are fewer lines available to cross it. Does a cheaper sheet change what we're measuring? | Persistent coast/card; same line objects spread, computed crossing dots. |
| 25_2 | To compare the methods fairly, keep the rule for a random drop explicit. Choose the line direction uniformly over a half turn. Independently, choose the slide uniformly over a full gap. Widening the gap widens that slide range too. A fixed direction or a favorite placement would be a different experiment. | Direction and normalized slide labels; selected geometry labelled illustrative. |
| 25_3 | Each crossing becomes worth more. With the original spacing it is worth pi centimetres. With the wider sheets, two pi and four pi. We get fewer crossings, but each counts for more length. Make a prediction. Will the average calibrated estimate change? ⟨predict⟩ | Calibration expressions and five-second prediction before mean table. |
| 25_4 | The factors cancel. The mean count is two times the length divided by pi times the gap. Multiply by pi times the gap over two, and we get the original length. For this coast, the mean counts are about eight point two two, four point one one, and two point zero five. Different counts, the same expected length. | Mean-count table plus common true-length line; symbolic cancellation earned. |
| 25_5 | Here are our thousand retained drops again. Use the same directions and the same fractions of a gap for all the sheets. The filled bars count what actually happened. Their horizontal positions are length estimates, so a crossing on a wider sheet makes a bigger jump. These finite averages need not land exactly on the true length. | Three histograms share a centimetre axis; finite mean labels and retained-pair coupling explicit. |
| 25_6 | The gold outlines describe the numerical population model instead. Its standard deviations are about four point four centimetres, eight point one, and fourteen point nine. Recall what that means. Square each estimate's deviation from the mean, average those squares, then take the square root. It measures spread. It is not an error bound, and it does not specify a coverage probability. | Numerical PMF outlines; separate SD bars centered on true length, not finite sample means. |
| 25_7 | For this coast and these spacings, the wider sheets have more variable estimates. That is what this comparison establishes. It is not a theorem that every increase in spacing makes every curve's standard deviation larger. The identity fixes the mean. It does not fix the whole distribution. | Return to same three distributions and common expectation; scope the observed trend. |
| 25_8 | Now remember the dividers from the opening. Their two centimetre walk measured twenty-two point six centimetres. Each straight chord is no longer than the portion of coast it replaces. A straight portion can give equality. A bend can lose length. Our red walk cuts across bends, so its total falls short. | Draw the original actual red walk segment by segment, including its final short chord; same coast. |
| 25_9 | Close the dividers to one centimetre. Think about where their next landing will be, and how many landings they will need. Make a prediction. Will this smaller step measure more length? ⟨predict⟩ | Hold original walk and question, then five-second prediction. |
| 25_10 | For this change, yes. The one centimetre walk gives twenty-five point one centimetres. Half a centimetre gives twenty-five point six. The shorter chords follow these bends more closely. But those examples do not prove that every smaller step gives a longer total. | Compute and draw each new walk, with separate result holds and table rows. |
| 25_11 | Try one point nine centimetres instead. It still takes eleven whole steps, just like the two centimetre walk. Those steps are each shorter, and the final little chord does not make up the difference. The result drops to twenty-one point six centimetres. A smaller step has given a shorter answer. | Actual1.9cm landings against faded2cm walk; eleven-step count and final chord; full table. |
| 25_12 | Both tools have a scale choice, but they use it differently. The divider sum replaces the path by chords and can miss its bends. The sheet preserves the expected length under our random-drop rule, for any positive gap, while its distribution can change. Keep the coast fixed, and distinguish the mean, the spread, and the result of a particular trial. | Same coast, divider table and expected-length callback; precise scope with full reading hold. |

**Scope.** Finite rectifiable path; count every traversal. Orientation uniform over a half turn, slide independent and uniform over a gap. Finite samples reuse retained angles and binary64 power-of-two slide scaling; no RNG regeneration. Population probabilities and SD are numerical, not exact continuum probabilities or coverage guarantees. The three observed SD comparisons are not a general monotonicity result. Divider chords are at most their corresponding arc lengths, with equality possible; totals here include the final short chord.

## 26_ripples: Which drawing did we measure?

| id | words | picture |
|---|---|---|
| 26_1 | What exactly does the sheet measure? Keep the same card and the same coast. Change only this straight beach, three point five centimetres long. Replace it with seven semicircles, each half a circle, keeping both endpoints fixed. | Persistent coast and card; change only highlighted beach with fixed endpoints and linked enlargement. |
| 26_2 | Each ripple has radius a quarter centimetre. Its arc is pi times its radius, so the beach now totals seven pi over four centimetres, about five point five. That is pi over two times its old length. The extra distance lives in the bends. | Computed semicircle radius and exact beach length, with approximation mark. |
| 26_3 | Now use fourteen semicircles with half the radius. More pieces, each proportionally shorter, give the same total length. These are alternative replacements of the straight beach. We have not added ripples inside the earlier ripples. | Same beach object becomes fourteen semicircles; explicitly alternative replacement, not nested detail. |
| 26_4 | Return to the larger ripples and our two centimetre dividers. The landing points do change. The totals shown here differ, yet both round to twenty-two point six centimetres. Equal rounded answers do not mean the dividers detected no change. They still cut across much of the new detail. | Actual two divider walks and changed landings; four-decimal totals distinguish equal one-decimal labels. |
| 26_5 | Keep the sheet at a two centimetre gap. Make a prediction. After averaging its turns and slides, will it recover the extra arc length, or miss that detail too? ⟨predict⟩ | Prediction before mean reveal. |
| 26_6 | It recovers the length of this new drawing. Turn uniformly over a half turn, and independently slide uniformly across a full gap. The mean is about eight point eight five crossings. Multiplying the unrounded mean by pi gives about twenty-seven point eight centimetres. The identity is exact. Those decimal displays are rounded. | Exact symbolic identity and explicitly rounded mean/length; joint probability law. |
| 26_7 | Here is a selected line, first above the ripples, then cutting through them. Now it contributes fourteen crossings on the beach. Counting only whether the line hits would discard nearly all that information. Every entrance and exit counts. | Actual selected line and computed fourteen beach intersection dots in both views. |
| 26_8 | A single ripple averages a quarter of a crossing under our turn and slide rule. Seven together average one point seven five. Whenever a ripple is hit, it contributes at least one crossing, so its chance of any hit is at most a quarter. Most placements miss that ripple, but we keep the multiple crossings when they occur. | Single-ripple expectation, additivity and P(hit) bound; no family independence assumption. |
| 26_9 | The sheet expectation is about twenty-seven point eight centimetres. The two centimetre divider walk is still about twenty-two point six. We changed the drawing, and therefore its length. Averaging crossings recovers that drawn length. It cannot restore details absent from the drawing. | Original diagram compares two methods on same new curve. |
| 26_10 | This distinction has a history. Mandelbrot quotes Steinhaus imagining much more detailed measurements of the left bank of the Vistula, lengths greater than a school map suggests by factors of ten, a hundred, even a thousand. That is a historical illustration of scale sensitivity, not a measurement we have made here. | Scoped paraphrase through Mandelbrot; not a Vistula experiment. |
| 26_11 | Our model remains a finite collection of straight pieces and circular arcs. It has a definite length. The sheet measures that chosen drawing in expectation. Deciding how much real coastal detail to draw is a separate question. | Return to chosen finite drawing and precise conditions. |

**Scope and sources.** Equal semicircle replacements preserve endpoints and all other coast pieces; n=7 and14 are alternatives, not recursively nested shapes. Exact total 4 + 1819π/240 cm; 27.8 cm is rounded. Two-centimetre divider totals differ (about22.6414 versus22.6268), despite both rounding to22.6; no blindness claim. Uniform half-turn direction and independent full-gap slide, finite rectifiable path and complete traversal counts. E[N] of one radius0.25 semicircle is0.25; P(N≥1)≤E[N], with no independence among ripple counts required.

History: primary text of Mandelbrot, Science156(1967), printed636 opening and638 reference1 read at https://pdodds.w3.uvm.edu/research/papers/others/1967/mandelbrot1967a.pdf. It attributes the Vistula illustration to Steinhaus1954. Publisher metadata https://www.impan.pl/en/publishing-house/journals-and-series/colloquium-mathematicum/all/3/1/93999/length-shape-and-area confirms title/year/pages; download and DML pages returned403. Original Steinhaus1954 paper remains unread. Paraphrase is explicitly through Mandelbrot; no empirical Vistula ratios, priority or pronunciation acceptance.

## 27_how_long_is_a_coastline: How long is a coastline?

| id | words | picture |
|---|---|---|
| 27_1 | Keep our original drawing fixed. Walk along it with one centimetre dividers, adding the chords and the last short piece. Now imagine reducing the step. Are we merely measuring this same drawing more accurately, or are we discovering new coastal detail? | Persistent original coast/card; actual one-centimetre divider walk and sum. |
| 27_2 | With a half centimetre step, the divider total increases by about two percent. The drawing itself has not changed. Shorter chords follow its existing bends more closely. That is a different operation from adding the ripples we just drew. | Replace only divider walk by half-centimetre landings; computed rounded increase. |
| 27_3 | Lewis Fry Richardson studied map measurements made with different divider steps. Mandelbrot later reported the results. Make a prediction. As the steps became smaller, did the measured coastal lengths settle toward a limit like an ordinary smooth curve, or keep growing across the measured scales? ⟨predict⟩ | Historical question and prediction before empirical result. |
| 27_4 | The reported measurements kept growing across those scales. Richardson described that behavior with an empirical scaling relation. It summarizes a pattern in measurements. It does not say that every observed ratio is identical, or that the pattern must continue below the measured range. | State measured-range growth and empirical scope before formula. |
| 27_5 | Here is the relation. G is the divider step, and L is the resulting length. M sets the overall scale. The exponent describes how strongly the measured length changes with the step. For the west coast of Britain, Mandelbrot reports D as one point two five. | Decode G, L, M and reported exponent with full inspection hold. |
| 27_6 | Halving the step in that fitted relation multiplies the length by two to the power D minus one. With the reported exponent, the factor is about one point one eight nine, roughly nineteen percent longer. This percentage is a calculation from the fit, not a list of measured halving ratios. | Halving ratio derived from fitted exponent; explicitly not raw observation ratios. |
| 27_7 | Compare the shapes of the two trends. Set each starting length to one, and successively halve its starting step. The teal curve illustrates the fitted relation. These are not Richardson data points. The gold curve uses our computed divider sums, starting at one centimetre. It rises much less. | Linked normalized smooth walk versus illustrative law, both labeled; no fabricated data. |
| 27_8 | Watch our actual walk at a quarter centimetre, then an eighth. Its chords follow the same finite straight pieces and circular arcs more closely. These selected totals approach the drawn length. Smaller steps need not increase every divider sum. Remember our earlier counterexample. The finite drawn length still bounds the chord sum. | Actual quarter/eighth walks on same coast; bounded sums and nonmonotonic caveat. |
| 27_9 | Mandelbrot used Richardson's results to discuss statistical self-similarity and fractional dimension in How Long Is the Coast of Britain? That is the historical connection. Observed scaling over a range does not, by itself, prove that a physical coast has infinite length or that the same law holds at arbitrarily small scales. | Scoped primary-text attribution with the exact subtitle “Statistical Self-Similarity and Fractional Dimension” and finite-range limitation. |
| 27_10 | Return to our two drawings. The smooth version and the rippled version have different lengths, and each has its own sheet expectation. Choose the drawing first. With all crossings counted, uniform turns and independent uniform slides recover its finite length on average. The sheet cannot recover coastal features that the drawing leaves out. | Return smooth/rippled drawing distinction and precise random-sheet conditions. |

**Scope and source.** Mandelbrot, Science156(1967), printed636–637, reports Richardson's empirical relation L(G)=M G^(1−D), with D=1.25 for the west coast of Britain. The plotted law is a normalized illustration, not digitized or newly measured coastline data. About19% is 100(2^0.25−1), a consequence of that exponent, not an exact repeated observation. No extrapolation to every scale or proof of infinite physical coastline length. Original Richardson1961 paper remains unread; attribution is through Mandelbrot. Primary extracted text read: https://pdodds.w3.uvm.edu/research/papers/others/1967/mandelbrot1967a.pdf. No PDF-pixel or pronunciation acceptance. Smooth-coast curves use actual native divider sums including final short chords; same finite model and retained data.

## 28_a_square_on_the_sheet: A square on the sheet

| id | words | picture |
|---|---|---|
| 28_1 | Return to our original card. The first sheet placement gave seven crossings and an estimate of twenty two centimetres. The drawn coast is about twenty five point eight centimetres long. A particular placement can miss the length. What did averaging buy us? | Original card/coast, seven crossings and rounded estimate versus drawn length. |
| 28_2 | For lines two centimetres apart, each crossing is worth pi centimetres. That calibration works on average when the direction is uniform over a half turn and the slide is independently uniform over a full gap. Count every traversal of a finite length path. The conditions are part of the rule. | Decode calibration and exact joint orientation/slide law. |
| 28_3 | We also tried four fixed directions with complete slide averages. They gave about twenty five point five three centimetres for this drawing. Averaging over all directions removes that remaining directional bias. Neither statement says that each individual placement equals the length. | Recall four-direction slide average and all-direction identity. |
| 28_4 | Now transfer the idea to a simpler outline. Draw a square with side two centimetres, exactly the gap between our lines. Keep the small square on the same card. The enlarged copy lets us inspect it. Use the same uniform direction and independent uniform full gap slide. | Persistent physical square and linked enlargement on original card. |
| 28_5 | Watch these selected placements. There are two crossings here, then four, then two, then four. These are illustrations, not a random sample. Can you see why most placements have one of those counts? | Four computed selected poses; count labels only after movement. |
| 28_6 | Look at the width of the square across the lines. It ranges from two centimetres to two times the square root of two, about two point eight three. That is at least one gap and less than two gaps. Except at boundary placements, either one line or two lines pass through the interior. Each enters and leaves the square, contributing two crossings. | Projection width and one/two interior lines; convex entry/exit intuition. |
| 28_7 | There are exceptions to that simple count. A line can touch a corner. This placement has three distinct contact points. Or lines can coincide with entire edges, giving infinitely many contact points. Those exact coincidences have probability zero under our continuous random choices. Two or four holds almost surely, which means with probability one, not without exceptions. | Actual corner contact and coincident edges separate from transverse crossings. |
| 28_8 | Make a prediction. Under that precise random choice of direction and slide, how often will the square have four crossings? We know the perimeter, and we know what a crossing is worth on average. Can those facts answer the question without a new sampling experiment? ⟨predict⟩ | Prediction before probability result, with deliberate hold. |
| 28_9 | The rim is four sides of two centimetres, so its length is eight. Divide by pi centimetres per crossing. The mean count is eight over pi, about two point five five. The fraction is exact for this model. The decimal is rounded. | Rim to exact mean and marked rounded decimal. |
| 28_10 | Call the probability of four crossings p. The probability of two is one minus p. The mean is two times one minus p, plus four times p. That simplifies to two plus two p. Think of a baseline pair of crossings, with an extra pair in the four crossing cases. The probability zero exceptions do not change this mean. | Define p and complementary probabilities before expectation algebra. |
| 28_11 | Set that mean equal to eight over pi. Subtract two, then divide by two. The chance of four crossings is four over pi minus one, about twenty seven percent. We found a probability from a length. | Solve mean identity and reveal exact versus rounded probability. |
| 28_12 | The remaining chance, about seventy three percent, belongs to two crossings. This bar shows the calculated probability law, not frequencies from a finite sample. Four crossings means four intersections with the rim. Here they come from two sheet lines. A different rule for choosing placements could give a different probability. | Calculated probability bar; distinguish intersections from sheet lines. |
| 28_13 | Return once more to the original coast and the same sheet. A count, an average, and a carefully stated random choice connected the pictures. Choose the finite drawing, count all traversals, and average under that law. That recovers its length, including detail you drew, without inventing detail you left out. | Original card callback and precise law/exception/traversal condition. |

**Mathematical scope.** Square side and sheet gap both 2 cm. Direction is uniform over a half turn and the independent slide is uniform over a full gap. For generic placements the projected width is 2(|sin beta|+|cos beta|), between 2 and 2sqrt(2), so one or two sheet lines cross the interior and each crosses the convex rim twice. Corner tangencies/vertex coincidences and edge coincidences are exceptional, probability-zero configurations, not counterexamples to an almost-sure claim. Mean8/pi gives P(N=4)=4/pi−1, not a finite-sample frequency. Physical and enlarged squares retain identity. Original drawing and retained RNG datasets unchanged. No new historical attribution or pronunciation acceptance.
