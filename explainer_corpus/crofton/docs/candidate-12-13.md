# Candidate chapters 12 and 13 (unregistered)

Written by GentleLantern for root's 12/13 assignment (after root froze
Mail 6982's four files in `root/tend-1455/crofton-10-11-intake`,
2026-10-24), continuing `mx-video-program-program-87do.22.3`. Two new
files, neither registered nor compiled:

- `src/chapters/why_every_curve.rs`: chapter 12, `WhyEveryCurve`. It
  rebuilds 11's last frame over the shore and follows Barbier's route on
  the same coast, the same 6 cm stick and the same sheet. It also holds the
  model the facts are printed from (`placed`, `halves`, `moved_back`,
  `along_ends`, `nested_moves`, `chain`, `chain_points`, `coin_rim`,
  `count_at`) and the live `Wire` that 13 bends.
- `src/chapters/the_bent_stick.rs`: chapter 13, `TheBentStick`, with the
  wire's `shape`, the 4's `four_meet` and `four_move`, the half-turn
  `sweep` and the count `shares`. It starts on 12's end.

The assignment calls 13 "bent_stick". The file is `the_bent_stick.rs`
and the key is `13_the_bent_stick`, the full draft's chapter id, named
like `the_bet.rs` and `the_shadow_rule.rs`. If root prefers
`bent_stick`, the rename touches the module line, the registry entry,
step 3's `the_bent_stick as bent` and the scene's `name()`.

No helper file was needed. Both chapters use `placement.rs`, `sheet.rs`,
`draw.rs`, 07's stick (`straight_sticks`), 08's console and rows
(`the_shadow_rule`), 10's `Shore`, `cuts`, `sticks` and `stick_ticks`
(`a_crowd_of_sticks`) and 11's words and plot helpers
(`turning_the_sheet`) as they stand. They need root's 08/09 and 10/11
integrations first (Mails 6913 and 6982's frozen files). 12 imports 10's
and 11's public items, and its rebuild of 11's end calls 11's `words()`,
which reads 10/11's facts. 13 imports 12's `Wire` and helpers, so the two
go in together.

The rows below are in `docs/storyboard.md`'s format, ready to move there
when root registers the scenes. `tools/storyboard.py`'s own `parse` and
`check` read this file as it stands. Nothing shared was edited: the
registry, `main.rs`, `model.rs`, `facts.rs`, `palette.rs`, `oracle.py`, the
scripts, the storyboards, chapters 01 to 11 and the pause overrides are as
root left them. Nothing here has been compiled, rendered, narrated or
listened to.

What the two chapters keep apart, as the assignment asks:

- exact for chains of straight sticks: crossings add piece by piece
  (12_3, 12_4), moving a stick is moving the sheet back (12_5, 12_6), so a
  stick's average depends only on its length. Equal pieces then give every
  fraction (12_7 to 12_9), and nesting gives every length, squeezed between
  fractions (12_10, 12_11). So a chain's average is its length times one
  number, exactly (12_12). No step uses the shadow rule's integral, the
  3,600-direction average or the identity itself;
- a limit for a curve: chains with their corners on the coast are never
  crossed more often than the coast, at any placement, and they get longer
  and catch more as their corners crowd together (12_13). At almost every
  placement their counts rise to the coast's, so the coast's average is the
  limit of theirs (12_14). The voice says "it's a limit". The "precisely"
  lower third says it holds for a curve with a length and that lines
  through a corner, an end or a tangent point have no chance. No count is
  changed for those placements. The limit's passage under the average is
  named, not proved, as `brief.md` says;
- the constant, found once: only after 12_15 asks which curve would find it
  does the coin give 2 = 2π × ?, so ? = 1 ÷ π (12_16);
- one placement against the average: the coast's 7 (12_3, 13_7), the
  chain's 5 (12_12), the bent wire's 1 (13_2) and its half-turn counts 1, 2,
  3, 2, 1, 2, 1 (13_3) are single placements. The averages are 6 ÷ π for
  every 6 cm wire and 25.81 ÷ π for the coast;
- computed against exact: 13_4's spreads are each count's chance, from
  `Curve::count_shares` over 3,600 directions (a midpoint rule over
  direction, exact over the slide). The oracle has them in closed form. The
  two agree within 2.2e−7, and each shown tenth of a percent lies at least
  3.67e−3 percentage points from a rounding edge. Four directions only
  estimate (1.81 crossings, 5.69 cm for the 6 cm wire), a negative control
  that is not shown;
- flat, not knotted: the wire bends only in the card's plane. The 4 that
  crosses itself is counted piece by piece, so a line through its crossing
  counts twice (13_5), and its "precisely" lower third says a wire bent out
  of the card is another question.

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
| 12_16 | The coin. Wherever it lands, it meets the lines at two points, and its rim is two pi centimetres. So two is two pi times the number, and the number is one over pi. | On "The coin" the countdown's ? goes. 05's teal coin, 2 cm across, lies at rest at card (2.0, 2.4) over the line above the bay, with its two gold dots and its count "2 crossings" at the upper left (`coin.crossings`). On "Wherever it lands" the sheet slides one gap in 4 s, then turns half a turn about card (2.0, 0.45), a point on a line, in 6 s. The coin's count stays "2 crossings" while its dots run round the rim. On "So two is two pi times the number" on the low row `2 = 2π × ?` (`coin.rim`, 6.28). On "the number is one over pi" beside it, in the length's green, `? = 1 ÷ π` | live dots and counter from the model, TeX | hold |
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

## Differences from the full draft

- 12 is seventeen beats, not six. The draft's two facts become Barbier's
  steps shown on our objects: the coast cut at its joints with the bay
  crossed twice (12_3, 12_4); invariance on the stick (12_5, 12_6); halves
  through a slide (12_7); crossed halves with multiplicity (12_8);
  fractions (12_9); the squeeze for a length that is not a fraction, which
  the draft left out (12_10, 12_11); chains exact (12_12); inscribed chains
  and the limit, with a "precisely" lower third (12_13, 12_14). The draft
  said instead that "a smooth curve can be matched as closely as we like by
  sticks, so the same holds for it".
- The draft's 12_1 "The French mathematician Joseph Émile Barbier" is
  dropped. The brief's primary source names "J.-É. Barbier", and neither
  his nationality nor his given names are in the cited evidence. The voice
  says "Barbier made this argument in eighteen sixty, without a single
  integral", and the lower third gives the journal and pages.
- The draft's coin "six point two eight centimetres of rim" becomes "two pi
  centimetres", so that 2 = 2π × ? reads as the calibration. "One crossing
  per pi centimetres" becomes "each crossing is worth pi centimetres", as
  06 said it.
- The draft's 13_1 "comes for free" becomes the derivation above. 13_2 now
  turns the stick to lie along first (0), then bends it (1). The draft's
  13_3 "anywhere from one to four instead of zero to three" is shown as
  the two computed spreads (13_4).
- The draft's 13_4 history, "the bendy cousin of Buffon's needle, a puzzle
  from the seventeen hundreds about dropping a needle on floorboards", is
  dropped for want of a cited primary source. The name stays, with its
  source on screen.
- The draft's 13_5 "Tie it in a knot, and the average still doesn't change"
  is replaced. A knot needs a third dimension, and the argument counts a
  wire lying flat. 13_5's planar 4 that crosses itself, counted piece by
  piece, takes its place. 13_7's callback to the coast is new.

## Proposed bindings

### Numbers (`main.rs`, the number guard)

These rows let every number word in 12 and 13 pass the guard (checked; see
below). SPOKEN, after 11's:
`("12_2", "in eighteen sixty", "barbier.year", "1860")`,
`("12_3", "the bay is crossed twice", "open.bay_crossings", "2")`,
`("12_3", "you get the coast's seven", "open.crossings", "7")`,
`("12_5", "our six centimetre stick", "stick.length", "6")`,
`("12_5", "it crosses two lines", "move.count", "2")`,
`("12_5", "instead of three", "stick.home_across", "3")`,
`("12_5", "Two again", "move.sheet_count", "2")`,
`("12_8", "so it counts twice", "cross.through", "2")`,
`("12_8", "cross each half once", "cross.count", "2")`,
`("12_9", "four and a half centimetres", "pieces.three_quarters", "4.5")`,
`("12_10", "fourteen point seven three centimetres", "ends.apart_2dp", "14.73")`,
`("12_10", "a fourteen and a fifteen centimetre stick", "ends.bracket.0", "14,15")`,
`("12_11", "between fourteen and fifteen times", "ends.bracket.0", "14,15")`,
`("12_11", "Between fourteen point seven and fourteen point eight", "ends.bracket.1", "14.7,14.8")`,
`("12_11", "between fourteen point seven three and fourteen point seven four", "ends.bracket.2", "14.73,14.74")`,
`("12_12", "twenty-one point three nine centimetres", "chain.joints.length", "21.39")`,
`("12_12", "crossed only five times", "chain.joints.crossings", "5")`,
`("12_13", "Twenty-three point nine one centimetres", "chain.cuts.length", "23.91")`,
`("12_13", "then twenty-five point seven one", "chain.sticks.length", "25.71")`,
`("12_13", "both catch all seven", "chain.cuts.crossings", "7")`,
`("12_13", "catch all seven", "chain.sticks.crossings", "7")`,
`("12_16", "at two points", "coin.crossings", "2")`,
`("12_16", "two pi centimetres", "coin.rim", "6.28")`,
`("12_17", "worth pi centimetres", "coin.worth", "3.14")`,
`("12_17", "twenty-five point eight one over pi", "coast.length_2dp", "25.81")`,
`("12_17", "about eight point two two", "crofton.mean_crossings", "8.22")`,
`("13_1", "our six centimetre stick", "stick.length", "6")`,
`("13_1", "about one point nine one crossings", "stick.turned_mean", "1.91")`,
`("13_1", "about three point eight two centimetres", "stick.mean_reach", "3.82")`,
`("13_1", "sixty-four percent", "stick.mean_reach_percent", "64")`,
`("13_2", "and it crosses none", "stick.home_along", "0")`,
`("13_2", "Here it crosses one line", "hook.home_count", "1")`,
`("13_2", "six centimetres", "stick.length", "6")`,
`("13_3", "It stays the same, about one point nine one", "hook.mean", "1.91")`,
`("13_4", "about ten point seven percent", "stick.shares_percent", "10.7,22.9,31.2,35.3")`,
`("13_4", "Bent, it never does", "hook.counts", "1,2,3,4")`,
`("13_5", "counts both of them", "four.through", "2")`,
`("13_5", "still one point nine one", "four.mean", "1.91")`,
`("13_5", "add up to six centimetres", "stick.length", "6")`,
`("13_7", "crossed the lines seven times", "open.crossings", "7")`,
`("13_7", "about eight point two two times", "crofton.mean_crossings", "8.22")`,
`("13_7", "twenty-five point eight one centimetres", "coast.length_2dp", "25.81")`.
DEFINITIONAL, after 11's (a choice, a unit or a definition, not a count
the facts hold): `("12_1", "on one coast")`, `("12_1", "one fixed number")`,
`("12_2", "without a single integral")`, `("12_2", "two ideas")`,
`("12_3", "once on its way down and once on its way back up")`,
`("12_7", "Cut the stick in half")`,
`("12_7", "Each half counts its own crossings")`,
`("12_7", "One half loses a crossing")`,
`("12_8", "Lay one half across the other")`,
`("12_8", "meets both halves")`,
`("12_9", "since one can be moved onto the other")`,
`("12_9", "each half has half the whole stick's average")`,
`("12_9", "each third a third")`, `("12_9", "each quarter a quarter")`,
`("12_9", "Three quarters together")`, `("12_9", "have three quarters of it")`,
`("12_9", "one number, the average for one centimetre")`,
`("12_10", "the coast's two ends")`, `("12_10", "the shorter one inside it")`,
`("12_15", "One number")`, `("12_15", "just one curve")`,
`("12_16", "So two is two pi times the number")`,
`("12_16", "the number is one over pi")`,
`("12_17", "its length over pi crossings")`,
`("13_1", "Its length over pi")`, `("13_1", "twelve over pi")`,
`("13_1", "why it's two over pi")`,
`("13_2", "two arms of three centimetres")`,
`("13_3", "Each arm is a three centimetre stick")`, `("13_3", "its two arms")`,
`("13_4", "one of its arms")`, `("13_5", "Bend it once more")`,
`("13_6", "its length over pi crossings")`,
`("13_7", "its length over pi")`, `("13_7", "pi times the average count")`.

"Two again" and "it crosses two lines" are counts, so they are SPOKEN, not
DEFINITIONAL. "Bent, it never does" holds no number word: it is SPOKEN so
that the never it claims is bound to `hook.counts` starting at 1. "and it
crosses none" binds the zero to `stick.home_along`. "catch all seven"
binds the sticks' chain; "both catch all seven" covers the cuts' chain
too, and each value is its own fact. "about ten point seven percent" is
bound to the full spread whose first share is 10.7. "So two is two pi
times the number" is DEFINITIONAL: the 2 is `coin.crossings`, already
SPOKEN two words earlier as "at two points", and the 2π is the rim.

### Pauses (text-only intentions)

These are authored overrides with the same fields as root's
`docs/pause-overrides.json`, for root to append to its `overrides` list
after 11's, once 08/09's and 10/11's are in, and to extend its `_comment`
to 12/13. They are editorial hypotheses, not measured pacing, and nothing
has been listened to. The countdowns, the slides, the turns, the swings,
the sweep and the coin's half turn stay protected visual time and are not
to be trimmed to these ranges. There are 44 overrides, 115 in all with
the integrated file's twenty-five, 08/09's nineteen and 10/11's
twenty-seven. Four of them, in 12_4, 12_5 (two) and 12_16, are longer
than their role's default because a picture must finish before the next
words name it (marked "sync" in their reasons). The silent-render
timeline below finds those four latenesses with no pauses and none with
the pauses at their least. The check resolves each override with
`tools/narration_director.py`'s own `directions()` on the script these
rows make, alone and with the other seventy-one, and no two land on the
same boundary. Each phrase occurs once in its cue.

```json
[
  {"video": "crofton", "cue": "12_1", "after": "on one coast", "role": "thought", "range_s": [0.5, 0.9], "reason": "11's computed result is restated as one coast's; let it land before the question why."},
  {"video": "crofton", "cue": "12_1", "after": "Here's the claim", "role": "thought", "range_s": [0.4, 0.8], "reason": "The claim row fades in; let the eye reach it before it is read out."},
  {"video": "crofton", "cue": "12_2", "after": "without a single integral", "role": "thought", "range_s": [0.5, 0.9], "reason": "Barbier's source is on screen; no integral is the frame for the whole chapter."},
  {"video": "crofton", "cue": "12_3", "after": "look at this placement", "role": "thought", "range_s": [0.6, 1.0], "reason": "The joints' ticks and the two tones come in; the eye needs to find the eleven pieces."},
  {"video": "crofton", "cue": "12_3", "after": "once on its way back up", "role": "thought", "range_s": [0.5, 0.9], "reason": "The bay's two halos are lit; let the viewer find both crossings on it before the sum."},
  {"video": "crofton", "cue": "12_4", "after": "always add up to the coast's count", "role": "thought", "range_s": [1.8, 2.2], "reason": "sync: the 8 s slide of a whole gap finishes, the row's digits changing and the sum always equal to the count, before the step from counts to averages."},
  {"video": "crofton", "cue": "12_5", "after": "instead of three", "role": "thought", "range_s": [0.5, 0.9], "reason": "Two against three: the moved stick's count has changed, the first fact to compare."},
  {"video": "crofton", "cue": "12_5", "after": "moving the sheet the other way", "role": "thought", "range_s": [0.5, 0.9], "reason": "The equivalence the proof rests on; let it register before it is shown."},
  {"video": "crofton", "cue": "12_5", "after": "Put the stick back", "role": "clause", "range_s": [0.8, 1.1], "reason": "sync: the stick's 2.5 s glide home finishes, three again, before the sheet is named."},
  {"video": "crofton", "cue": "12_5", "after": "move the sheet instead", "role": "thought", "range_s": [1.4, 1.8], "reason": "sync: the sheet's 3 s turn must finish, and the count read 2, before 'Two again'."},
  {"video": "crofton", "cue": "12_6", "after": "which way it points", "role": "thought", "range_s": [0.5, 0.9], "reason": "The conclusion of invariance; the sheet has turned back and the count reads three."},
  {"video": "crofton", "cue": "12_7", "after": "together they make the whole stick's count", "role": "thought", "range_s": [0.4, 0.8], "reason": "The halves' counts 2 and 1 stand beside them; let the viewer add them to the 3."},
  {"video": "crofton", "cue": "12_7", "after": "as a line passes it", "role": "thought", "range_s": [0.8, 1.1], "reason": "The slide reaches the cut about 3.75 s after 'Watch the cut'; the next sentence should land as the counts swap."},
  {"video": "crofton", "cue": "12_8", "after": "Lay one half across the other", "role": "thought", "range_s": [0.6, 1.0], "reason": "The lower half swings across the upper; let the viewer watch it settle on the middle before the meeting point is named."},
  {"video": "crofton", "cue": "12_8", "after": "so it counts twice", "role": "clause", "range_s": [0.3, 0.6], "reason": "One point, two dots, two crossings: the multiplicity the rest of the proof needs."},
  {"video": "crofton", "cue": "12_9", "after": "since one can be moved onto the other", "role": "thought", "range_s": [0.5, 0.9], "reason": "The swing back and the slide back run; the reason equal pieces agree is 12_5's."},
  {"video": "crofton", "cue": "12_9", "after": "have three quarters of it", "role": "thought", "range_s": [0.5, 0.9], "reason": "The three-quarters row is on screen; let it be read before the general rule."},
  {"video": "crofton", "cue": "12_10", "after": "and a bit", "role": "thought", "range_s": [0.5, 0.9], "reason": "14.73 cm and a bit is not a fraction we can build; the line is drawn."},
  {"video": "crofton", "cue": "12_10", "after": "and watch", "role": "thought", "range_s": [0.8, 1.2], "reason": "The quarter turn is under way; give the eye the three counts before the rule is said."},
  {"video": "crofton", "cue": "12_11", "after": "fifteen times our number", "role": "thought", "range_s": [0.4, 0.8], "reason": "The first bracket appears; let it be read before it tightens."},
  {"video": "crofton", "cue": "12_11", "after": "fourteen point seven four", "role": "thought", "range_s": [0.5, 0.9], "reason": "The hundredths bracket is up; the squeeze is the step from fractions to every length."},
  {"video": "crofton", "cue": "12_12", "after": "exactly its total length times that number", "role": "thought", "range_s": [0.5, 0.9], "reason": "The exact result for chains; the sheet is returning to the first placement under it."},
  {"video": "crofton", "cue": "12_12", "after": "crossed only five times", "role": "thought", "range_s": [0.5, 0.9], "reason": "Five against the coast's seven: let the viewer look for the two that are missing."},
  {"video": "crofton", "cue": "12_13", "after": "never crossed more often than the coast", "role": "thought", "range_s": [0.6, 1.0], "reason": "The one-sided bound is the key to the limit; let it settle before the finer chains."},
  {"video": "crofton", "cue": "12_13", "after": "catch all seven", "role": "cue_transition", "range_s": [0.6, 1.0], "reason": "The finest chain's purple dots sit on the gold ones; let it be seen before the limit is stated."},
  {"video": "crofton", "cue": "12_14", "after": "rise to the coast's count", "role": "thought", "range_s": [0.5, 0.9], "reason": "The limit's two halves, lengths and counts, both said; the lower third is up."},
  {"video": "crofton", "cue": "12_15", "after": "the same for every curve", "role": "thought", "range_s": [0.5, 0.9], "reason": "The claim returns with its ?; the question of what calibrates it follows."},
  {"video": "crofton", "cue": "12_16", "after": "The coin", "role": "thought", "range_s": [0.5, 0.9], "reason": "The answer to the prediction: the coin fades in with its two dots."},
  {"video": "crofton", "cue": "12_16", "after": "two pi centimetres", "role": "thought", "range_s": [2.6, 3.2], "reason": "sync: the sheet's 4 s slide and 6 s half turn under the coin, its count steady at 2, finish before the calibration is said."},
  {"video": "crofton", "cue": "12_17", "after": "worth pi centimetres", "role": "thought", "range_s": [0.5, 0.9], "reason": "The rule for every curve and 06's worth meet; let them join."},
  {"video": "crofton", "cue": "12_17", "after": "It was forced", "role": "thought", "range_s": [0.6, 1.0], "reason": "The chapter's point, before the coast's own numbers."},
  {"video": "crofton", "cue": "13_1", "after": "Back to our six centimetre stick", "role": "thought", "range_s": [0.5, 0.9], "reason": "12's rows and the coin go and the stick returns across the lines."},
  {"video": "crofton", "cue": "13_1", "after": "about three point eight two centimetres", "role": "thought", "range_s": [0.5, 0.9], "reason": "The shadow row is up; let it be read before its share is drawn."},
  {"video": "crofton", "cue": "13_2", "after": "and it crosses none", "role": "thought", "range_s": [0.5, 0.9], "reason": "Along the lines, 0: the count's first change, seen before the bend."},
  {"video": "crofton", "cue": "13_2", "after": "Here it crosses one line", "role": "thought", "range_s": [0.5, 0.9], "reason": "Bent, one crossing: the count has changed; the length comes next."},
  {"video": "crofton", "cue": "13_2", "after": "six centimetres", "role": "thought", "range_s": [0.5, 0.9], "reason": "The arms' labels and 3 + 3 = 6 cm are up before the prediction."},
  {"video": "crofton", "cue": "13_3", "after": "about one point nine one", "role": "thought", "range_s": [0.5, 0.9], "reason": "The prediction's answer, with its row; the reason follows."},
  {"video": "crofton", "cue": "13_3", "after": "which is what the straight stick does", "role": "thought", "range_s": [0.5, 0.9], "reason": "The additivity argument is complete before the sheet turns."},
  {"video": "crofton", "cue": "13_4", "after": "how the count is spread", "role": "thought", "range_s": [0.6, 1.0], "reason": "The straight spread's bars fade in; let the eye find the 0 bar."},
  {"video": "crofton", "cue": "13_4", "after": "more than a gap across the lines", "role": "thought", "range_s": [0.8, 1.1], "reason": "Both spreads are up; compare them before the summary line."},
  {"video": "crofton", "cue": "13_5", "after": "Where its pieces cross", "role": "clause", "range_s": [0.4, 0.65], "reason": "The ring marks the crossing as the sheet starts to move a line onto it."},
  {"video": "crofton", "cue": "13_6", "after": "Buffon's noodle", "role": "thought", "range_s": [0.5, 0.9], "reason": "The name and its source on screen; the rule for every flat wire follows."},
  {"video": "crofton", "cue": "13_7", "after": "a long bent wire", "role": "thought", "range_s": [0.5, 0.9], "reason": "The wire goes and the sheet returns to the first placement; the coast takes its place."},
  {"video": "crofton", "cue": "13_7", "after": "about eight point two two times", "role": "thought", "range_s": [0.5, 0.9], "reason": "One placement's seven against the average 8.22, before the turn around."}
]
```

Visual holds between cues (the picture runs on while the voice waits).
These need render evidence before they enter a map, so here they are only
the scene's own seconds:

- after 12_4: the slide runs on, 8 s from "Slide the sheet", past the
  line's end.
- after 12_7: the slide past the cut, 10 s from "Watch the cut", then the
  lower third.
- after 12_10: the quarter turn and the slide, 3 s each, from "Turn the
  sheet, slide it".
- after 12_15 and 13_2: the countdown, 4 s (`think`).
- after 13_3: the half turn, 12 s from "Turn the sheet, and its count
  moves around".
- after 12_17 and 13_7: the chapter's end, 2.6 s.
- inside 12_4, 12_5 and 12_16: the slide, the stick's return, the sheet's
  turn, and the coin's slide and half turn that the "sync" overrides wait
  for. Inside 12_8, 13_2 and 13_5 the swing, the turn and the move fit
  their words in the silent estimate, so their pauses are ordinary ones.
  Each animation blocks the scene until it is done, so a phrase after it
  waits for it, and a real narrator faster than the estimate would show
  those three late too. That needs the render's evidence.

## Integration steps for root

Rebased against the shared files as they stood at 16:27Z, before root's
08/09 and 10/11 integrations. Their sha256 prefixes: `mod.rs`
b1bfe30ea99ec85a, `main.rs` dff1a9534fbdff59, `facts.rs`
27be92d39bfcb699, `oracle.py` 290d05d0b43a4a11, `script.tsv`
ae2027918938452e (ends at 07_7), `docs/storyboard.md` 26f70b3098e10444,
`docs/pause-overrides.json` f218a01902d855b4 (25 overrides). Do steps 1 to
9 of `candidate-08-09.md` and then of `candidate-10-11.md` first. Until
steps 1 to 4 here are in, the two chapters do not compile (the new
modules) or their `words()` refuse (missing facts).

1. `src/chapters/mod.rs`. Declare the two modules, keeping the list in
   order: `pub mod the_bent_stick;` before `pub mod the_bet;`, and
   `pub mod why_every_curve;` last, after `pub mod turning_the_stick;`.
   Add the registry entries after 11. Proposed intensities are 0.5 and 0.4
   (12 is the argument):
   `Chapter { key: "12_why_every_curve", make: |kit| Box::new(why_every_curve::WhyEveryCurve { kit }), intensity: 0.5 }`
   and
   `Chapter { key: "13_the_bent_stick", make: |kit| Box::new(the_bent_stick::TheBentStick { kit }), intensity: 0.4 }`.
   The module doc's "the draft's chapters 12 to 28" (after 10/11) becomes
   "14 to 28".
2. No palette change: both chapters use 07's `palette::stick()` for the
   wire, the chains and the bars, and 01 to 11's roles for the coast, the
   gold crossings and rings, the green length and the teal coin.
3. `src/facts.rs`. After 10/11's block (ending with
   `f.add(&format!("grid.{most}.mean_crossings"), fixed(height / GAP, 4))?;`)
   and before "the draft's 19_1", using the `coast` already bound in
   `facts()`:

   ```rust
   // Chapters 12 and 13: Barbier's route on the card, each number from the
   // model the chapters draw. 12: the pieces' counts at the first placement,
   // the moved stick and the sheet moved back, the halves, the nested
   // sticks along the ends' line, the chains on the coast. 13: the bent
   // wire at home, the half turn about its corner's line, each wire's count
   // shares over 3,600 directions (a midpoint rule over direction), the 4
   // and the line through its crossing. And the two years the voice names
   {
       use crate::chapters::placement::Pose;
       use crate::chapters::straight_sticks::{ACROSS, ALONG, HOME, LENGTH};
       use crate::chapters::{sheet, the_bent_stick as bent, why_every_curve as every};
       let first = Pose::new(0.0, sheet::shown_slide(&coast, 0.0));
       let joined = |v: &[usize]| v.iter().map(usize::to_string).collect::<Vec<_>>().join(",");
       f.add("barbier.year", every::BARBIER_YEAR)?;
       let crossings = sheet::crossings(&coast, first.beta, first.slide);
       let by_piece: Vec<usize> = (0..coast.parts.len())
           .map(|i| crossings.iter().filter(|c| c.piece == i).count())
           .collect();
       f.add("open.piece_counts", joined(&by_piece))?;
       let home = every::placed(every::halves, HOME, ACROSS, 0.0);
       let moved_to = [HOME[0] + every::MOVE_RIGHT, HOME[1]];
       let moved = every::placed(every::halves, moved_to, every::MOVE_TO, 0.0);
       f.add("move.count", every::count_at(&moved, first))?;
       let moved_back = every::moved_back(first).end();
       f.add("move.sheet_count", every::count_at(&home, moved_back))?;
       let split = sheet::crossings(&home, first.beta, first.slide);
       let halves: Vec<usize> = (0..2).map(|i| split.iter().filter(|c| c.piece == i).count()).collect();
       f.add("halves.counts", joined(&halves))?;
       let crossed = every::placed(every::halves, HOME, ACROSS, 1.0);
       f.add("cross.count", every::count_at(&crossed, first))?;
       let meet = (HOME[1] + LENGTH / 4.0).rem_euclid(GAP);
       f.add("cross.through", sheet::crossings(&crossed, 0.0, meet).len())?;
       f.add("pieces.third", fixed(LENGTH / 3.0, 0))?;
       f.add("pieces.quarter", fixed(LENGTH / 4.0, 1))?;
       f.add("pieces.three_quarters", fixed(3.0 * LENGTH / 4.0, 1))?;
       let (a, b) = coast.ends();
       let apart = every::dist(a, b);
       f.add("ends.apart_2dp", fixed(apart, 2))?;
       for k in 0..3 {
           let scale = 10f64.powi(k);
           let lo = (apart * scale).floor() / scale;
           let (low, high) = (fixed(lo, k as usize), fixed(lo + 1.0 / scale, k as usize));
           f.add(&format!("ends.bracket.{k}"), format!("{low},{high}"))?;
       }
       let nested = [every::NESTED[0], apart, every::NESTED[1]].map(|l| every::along_ends(&coast, l));
       for (i, mv) in every::nested_moves(first).iter().enumerate() {
           let counts: Vec<usize> = nested.iter().map(|c| every::count_at(c, mv.end())).collect();
           f.add(&format!("ends.nested.{i}"), joined(&counts))?;
       }
       for (name, at) in ["joints", "cuts", "sticks"].iter().zip(every::chain_points(&coast)) {
           let chain = every::chain(&coast, &at);
           f.add(&format!("chain.{name}.chords"), chain.parts.len())?;
           f.add(&format!("chain.{name}.length"), fixed(chain.length, 2))?;
           f.add(&format!("chain.{name}.crossings"), every::count_at(&chain, first))?;
       }
       let wire = every::placed(bent::shape, HOME, ALONG, bent::BENT);
       f.add("hook.home_count", every::count_at(&wire, first))?;
       let [half_turn] = bent::sweep(first);
       let mut seen: Vec<usize> = Vec::new();
       for i in 0..=3600 {
           let n = every::count_at(&wire, half_turn.at(f64::from(i) / 3600.0));
           if seen.last() != Some(&n) {
               seen.push(n);
           }
       }
       f.add("hook.sweep", joined(&seen))?;
       for (key, counts_key, u) in [
           ("stick", "stick.all_counts", bent::STRAIGHT),
           ("hook", "hook.counts", bent::BENT),
       ] {
           let shares = bent::shares(u);
           let low = shares.iter().position(|&p| p > 1e-9).unwrap_or(0);
           let shown: Vec<String> = shares[low..].iter().map(|&p| fixed(p * 100.0, 1)).collect();
           f.add(&format!("{key}.shares_percent"), shown.join(","))?;
           f.add(counts_key, joined(&(low..shares.len()).collect::<Vec<_>>()))?;
       }
       f.add("hook.mean", fixed(model::moments(&bent::shares(bent::BENT)).0, 2))?;
       let four = every::placed(bent::shape, HOME, ALONG, bent::FOUR);
       let at = bent::four_move(first).end();
       let here = sheet::crossings(&four, at.beta, at.slide);
       f.add("four.count", here.len())?;
       let meet = bent::four_meet();
       let through = here.iter().filter(|c| every::dist(c.at, meet) < 1e-9).count();
       f.add("four.through", through)?;
       f.add("four.mean", fixed(model::moments(&bent::shares(bent::FOUR)).0, 2))?;
       f.add("noodle.year", bent::NOODLE_YEAR)?;
   }
   ```

   Expected values: `barbier.year` 1860, `open.piece_counts`
   1,0,1,1,0,1,2,0,0,0,1, `move.count` 2, `move.sheet_count` 2,
   `halves.counts` 2,1, `cross.count` 2, `cross.through` 2,
   `pieces.third` 2, `pieces.quarter` 1.5, `pieces.three_quarters` 4.5,
   `ends.apart_2dp` 14.73, `ends.bracket.0` 14,15, `ends.bracket.1`
   14.7,14.8, `ends.bracket.2` 14.73,14.74, `ends.nested.0` 7,7,7,
   `ends.nested.1` 7,8,8, `chain.joints.chords` 11, `chain.joints.length`
   21.39, `chain.joints.crossings` 5, `chain.cuts.chords` 15,
   `chain.cuts.length` 23.91, `chain.cuts.crossings` 7,
   `chain.sticks.chords` 67, `chain.sticks.length` 25.71,
   `chain.sticks.crossings` 7, `hook.home_count` 1, `hook.sweep`
   1,2,3,2,1,2,1, `stick.shares_percent` 10.7,22.9,31.2,35.3,
   `stick.all_counts` 0,1,2,3, `hook.shares_percent` 23.5,62.1,14.3,0.1,
   `hook.counts` 1,2,3,4, `hook.mean` 1.91, `four.count` 2,
   `four.through` 2, `four.mean` 1.91, `noodle.year` 1969. Thirty-three
   are new keys. `hook.counts`, `hook.mean` and `stick.all_counts` are
   already printed by the oracle, and `facts.rs` must print them now
   because 13 reads them (`facts-check` wants every key the video prints
   in the oracle, with the same value). The other keys the chapters read
   (`open.*`, `stick.*`, `sheet.gap`, `coin.*`, `coast.*`, `crofton.*`) are
   already printed. The block, set inside a function body, passes `rustfmt
   --check` at the repo's width. It is a block of its own, so none of its
   names (`first`, `joined`, `crossings`, `home`, `moved`, `nested`,
   `wire`, `four`, `meet` and so on) reaches root's later lines. `model`,
   `fixed` and `GAP` are already in scope there.
4. `oracle.py`. The constants after 10/11's
   `DIRECTION_GRID = [4, 8, 16, 32, 3600]`:

   ```python
   # Chapters 12 and 13: where the stick is moved to (cm right of home, and
   # its direction, degrees), the halves crossed, the nested sticks along the
   # coast's ends, the sheet turned for them, the wire bent into a self-crossing
   # "4" (its pieces, cm, and the turn at each joint, degrees), and the year
   # of Barbier's note (J. Math. Pures Appl. (2) 5, 1860) and Ramaley's
   # "Buffon's noodle problem" (Amer. Math. Monthly 76, 1969)
   MOVE_RIGHT = 1.5
   MOVE_TO = 45
   CROSS_AT = 2.7
   CROSS_TO = 135
   NESTED = [14.0, 15.0]
   NESTED_SLIDES = [0.0, 1.0]
   FOUR_PIECES = [3.0, 1.5, 1.5]
   FOUR_TURN = 135
   FOUR_SHEET = 45
   BARBIER_YEAR = 1860
   NOODLE_YEAR = 1969
   ```

   Then block 5 inside `main()`, 4-space indent, before the line
   `    f["steinhaus.directions"] = str(STEINHAUS_DIRECTIONS)`:

   ```python
       # Chapters 12 and 13 (a function of its own, so its names never shadow
       # the ones main() uses after it)
       def chapters_12_13():
           # Chapter 12: why every curve, by Barbier's route (no appeal to the
           # identity: each step below is checked on its own; the identity only
           # appears at the end as a cross-check). Counts are piece by piece, as
           # one_way_pieces counts them: a line through two pieces at one point
           # counts for both
           f["barbier.year"] = str(BARBIER_YEAR)
           by_piece = [crossings([g], 0, first_slide) for g in geom]
           f["open.piece_counts"] = ",".join(map(str, by_piece))
           check(sum(by_piece) == first, f"12_3: the pieces' counts {by_piece} add up to the coast's {first}")
           check(by_piece[BAY] == 2 == int(f["open.bay_crossings"]) and by_piece[BEACH] == 0 and by_piece[HEADLAND] == 0, "12_3: the bay is crossed twice, the beach and the headland not at all")
           # crossings add, against a different count: the coast as a fine
           # polyline, each chord tested against each line directly
           fine = sample_points(geom, 512)
           worst_sum = 0
           for i in range(36):
               beta = 180 * (i + 0.5) / 36
               n = normal(beta)
               for j in range(25):
                   slide = GAP * (j + 0.5) / 25
                   hs = [p[0] * n[0] + p[1] * n[1] for p in fine]
                   direct = sum(abs(math.floor((b - slide) / GAP) - math.floor((a - slide) / GAP)) for a, b in zip(hs, hs[1:]))
                   pieces_sum = sum(crossings([g], beta, slide) for g in geom)
                   worst_sum = max(worst_sum, abs(direct - pieces_sum))
           check(worst_sum == 0, f"12_4: at 900 placements the pieces' counts add up to a fine polyline's direct count (worst difference {worst_sum})")

           # 12_5: the stick moved right and turned, against the stick at home
           # with the sheet moved the other way (turned about home)
           def home_stick(angle, centre=STICK_HOME):
               return stick_geom(STICK, angle, centre)

           moved = home_stick(MOVE_TO, (STICK_HOME[0] + MOVE_RIGHT, STICK_HOME[1]))
           f["move.count"] = str(crossings(moved, 0, first_slide))
           n0, n1 = normal(0), normal(90 - MOVE_TO)
           back = first_slide - (n0[0] * MOVE_RIGHT) + (n1[0] - n0[0]) * STICK_HOME[0] + (n1[1] - n0[1]) * STICK_HOME[1]
           f["move.sheet_count"] = str(crossings(home_stick(90), 90 - MOVE_TO, back))
           check(f["move.count"] == f["move.sheet_count"] != f["stick.home_across"], f"12_5: moved, the stick crosses {f['move.count']}, not {f['stick.home_across']}; put back with the sheet moved the other way, {f['move.sheet_count']}")
           rng = np.random.default_rng(1213)
           worst_move = 0
           for _ in range(400):
               dx, dy, turn = rng.uniform(-1.5, 1.5), rng.uniform(-1.5, 1.5), rng.uniform(-90, 90)
               beta, slide = rng.uniform(0, 180), rng.uniform(0, GAP)
               stick_now = home_stick(90 + turn, (STICK_HOME[0] + dx, STICK_HOME[1] + dy))
               nb, nt = normal(beta), normal(beta - turn)
               sheet_back = slide - (nb[0] * dx + nb[1] * dy) + (nt[0] - nb[0]) * STICK_HOME[0] + (nt[1] - nb[1]) * STICK_HOME[1]
               worst_move = max(worst_move, abs(crossings(stick_now, beta, slide) - crossings(home_stick(90), beta - turn, sheet_back)))
           check(worst_move == 0, "12_5: at 400 random moves, moving the stick and moving the sheet the other way give the same count")

           # 12_7: the halves at home, across: their counts, and through a slide
           # of one gap they always add up to the whole stick's (except where a
           # line passes through the cut, a single slide)
           low = [("S", (STICK_HOME[0], STICK_HOME[1] - STICK / 2), STICK_HOME)]
           high = [("S", STICK_HOME, (STICK_HOME[0], STICK_HOME[1] + STICK / 2))]
           f["halves.counts"] = f"{crossings(low, 0, first_slide)},{crossings(high, 0, first_slide)}"
           halves_ok = all(crossings(low, 0, s) + crossings(high, 0, s) == crossings(home_stick(90), 0, s) for s in (first_slide + GAP * (i + 0.5) / 400 for i in range(400)))
           check(halves_ok and sum(map(int, f["halves.counts"].split(","))) == int(f["stick.home_across"]), f"12_7: the halves cross {f['halves.counts']}, and through a slide they add up to the whole's")
           # 12_8: the low half laid across the high one's middle; a line through
           # their meeting point counts once for each
           a = math.radians(CROSS_TO)
           crossed = [("S", (STICK_HOME[0] - STICK / 4 * math.cos(a), CROSS_AT - STICK / 4 * math.sin(a)), (STICK_HOME[0] + STICK / 4 * math.cos(a), CROSS_AT + STICK / 4 * math.sin(a)))]
           meet = (STICK_HOME[0], CROSS_AT)
           f["cross.count"] = str(crossings(high + crossed, 0, first_slide))
           through = CROSS_AT % GAP
           f["cross.through"] = str(crossings(high + crossed, 0, through))
           check(segments_cross(high[0][1], high[0][2], crossed[0][1], crossed[0][2]), "12_8: the two halves cross")
           check(f["cross.through"] == "2" and crossings(high, 0, through) == 1 == crossings(crossed, 0, through), "12_8: the line through their meeting point counts once for each half, two")
           points = {(round(x, 9), round(y, 9)) for g in high + crossed for x, y in [(g[1][0] + (CROSS_AT - g[1][1]) / (g[2][1] - g[1][1]) * (g[2][0] - g[1][0]), CROSS_AT)]}
           check(len(points) == 1 != int(f["cross.through"]), "negative control: counted as points of the card, the line through the meeting point would count one, not two")
           check(abs(sum(math.dist(g[1], g[2]) for g in high + crossed) - STICK) < 1e-12 and math.dist(meet, ((crossed[0][1][0] + crossed[0][2][0]) / 2, (crossed[0][1][1] + crossed[0][2][1]) / 2)) < 1e-12, "12_8: still 6 cm in all, the meeting point at the crossed half's middle")

           # 12_9: equal pieces have equal averages (by moving them), so a third
           # of the stick has a third of its average; three quarters, three
           # quarters. Checked numerically, each average over every slide and
           # direction
           def stick_mean(l):
               return simpson_mean(lambda b: travel(stick_geom(l, 90), b) / GAP, 0.0, 180.0, 3600)

           f["pieces.third"] = fixed(STICK / 3, 0)
           f["pieces.quarter"] = fixed(STICK / 4, 1)
           f["pieces.three_quarters"] = fixed(3 * STICK / 4, 1)
           m6, m2, m15, m45 = stick_mean(STICK), stick_mean(STICK / 3), stick_mean(STICK / 4), stick_mean(3 * STICK / 4)
           check(abs(3 * m2 - m6) < 1e-9 and abs(4 * m15 - m6) < 1e-9 and abs(m45 - 3 * m15) < 1e-9 and abs(m45 - 0.75 * m6) < 1e-9, f"12_9: 3 x {m2:.6f} = 4 x {m15:.6f} = {m6:.6f}, and 4.5 cm averages {m45:.6f}, three quarters of it")

           # 12_10 and 12_11: the straight line between the coast's ends, and the
           # 14 and 15 cm sticks along it from the coast's start, the sheet turned
           # a quarter; a longer stick is crossed at least as often as a shorter
           # one inside it, so the average is squeezed between tenths, hundredths
           start, end = fine[0], fine[-1]
           apart = math.dist(start, end)
           f["ends.apart_2dp"] = fixed(apart, 2)
           u = ((end[0] - start[0]) / apart, (end[1] - start[1]) / apart)

           def along(l):
               return [("S", start, (start[0] + l * u[0], start[1] + l * u[1]))]

           nested = [along(NESTED[0]), along(apart), along(NESTED[1])]
           for k in range(3):
               lo = Fraction(math.floor(apart * 10**k), 10**k)
               f[f"ends.bracket.{k}"] = f"{fixed(lo, k)},{fixed(lo + Fraction(1, 10**k), k)}"
               check(lo < apart < lo + Fraction(1, 10**k), f"12_11: {apart:.6f} lies between {f[f'ends.bracket.{k}']}")
           check(f["ends.bracket.1"].split(",")[0] == f["coast.ends_apart"], "12_10: the bracket's lower tenth is 03's 14.7")
           for i, s in enumerate(NESTED_SLIDES):
               f[f"ends.nested.{i}"] = ",".join(str(crossings(g, 90, s)) for g in nested)
               worst_end = min(clearance(g, 90, s) for g in nested)
               check(worst_end > 0.25, f"12_10: at 90 degrees, slide {s}: counts {f[f'ends.nested.{i}']}, every end {worst_end:.3f} cm from a line")
           check(f["ends.nested.0"] == "7,7,7" and len(set(f["ends.nested.1"].split(","))) > 1, "12_10: first all three alike, then the longer ones crossed more")
           order_bad = sum(not (crossings(nested[0], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30) <= crossings(nested[1], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30) <= crossings(nested[2], 180 * (i + 0.5) / 120, GAP * (j + 0.5) / 30)) for i in range(120) for j in range(30))
           check(order_bad == 0, "12_10: at 3,600 placements the 14 cm stick is never crossed more than the line between the ends, nor that more than the 15 cm stick")

           # 12_12 and 12_13: chains with their corners on the coast, at its
           # joints, at chapter 10's cuts, at 10's sticks' ends. Each chord is
           # crossed only by a line that crosses the coast between its ends, so a
           # chain is never crossed more than the coast
           def point_at(s):
               for g in geom:
                   if g[0] == "S":
                       w = math.dist(g[1], g[2])
                       if s <= w + 1e-12:
                           return (g[1][0] + s / w * (g[2][0] - g[1][0]), g[1][1] + s / w * (g[2][1] - g[1][1]))
                   else:
                       (cx, cy), r, a0, a1 = g[1], g[2], g[3], g[4]
                       w = r * abs(a1 - a0)
                       if s <= w + 1e-12:
                           t = a0 + (a1 - a0) * s / w
                           return (cx + r * math.cos(t), cy + r * math.sin(t))
                   s -= w
               return end

           widths = [math.dist(g[1], g[2]) if g[0] == "S" else g[2] * abs(g[4] - g[3]) for g in geom]
           joints = [sum(widths[:i]) for i in range(len(geom) + 1)]
           cut_at = list(joints)
           for i, g in enumerate(geom):
               if g[0] == "A":
                   lo_a, hi_a = sorted((g[3], g[4]))
                   k = math.ceil((lo_a - math.pi / 2) / math.pi)
                   while math.pi / 2 + k * math.pi < hi_a:
                       lev = math.pi / 2 + k * math.pi
                       if lev - lo_a > 1e-9 and hi_a - lev > 1e-9:
                           cut_at.append(joints[i] + g[2] * abs(lev - g[3]))
                       k += 1
           cut_at.sort()
           stick_at = []
           for a, b in zip(cut_at, cut_at[1:]):
               k = max(1, math.ceil((b - a) / STICK_LONGEST - 1e-9))
               stick_at += [a + (b - a) * i / k for i in range(k)]
           stick_at.append(cut_at[-1])
           chains = {}
           for name, at in (("joints", joints), ("cuts", cut_at), ("sticks", stick_at)):
               pts = [point_at(s) for s in at]
               chain = [("S", p, q) for p, q in zip(pts, pts[1:])]
               chains[name] = chain
               f[f"chain.{name}.chords"] = str(len(chain))
               f[f"chain.{name}.length"] = fixed(sum(math.dist(p, q) for p, q in zip(pts, pts[1:])), 2)
               f[f"chain.{name}.crossings"] = str(crossings(chain, 0, first_slide))
           check(f["chain.sticks.chords"] == f["sticks.count"], f"12_13: the finest chain has a corner at each of 10's {f['sticks.count']} sticks' ends")
           lengths = [float(f[f"chain.{n}.length"]) for n in ("joints", "cuts", "sticks")]
           check(lengths == sorted(lengths) and lengths[-1] < length, f"12_13: the chains' lengths {lengths} rise towards the coast's {length:.4f} from below")
           check(crossings([chains["joints"][BAY]], 0, first_slide) == 0 and f["chain.joints.crossings"] == str(first - by_piece[BAY]), "12_12: the joints chain's chord across the bay misses both of the bay's crossings, and the chain counts the rest")
           check(f["chain.cuts.crossings"] == f["chain.sticks.crossings"] == str(first), "12_13: the finer chains catch all seven")
           more, caught = 0, {n: 0 for n in chains}
           for i in range(90):
               beta = 180 * (i + 0.5) / 90
               for j in range(40):
                   slide = GAP * (j + 0.5) / 40
                   c = crossings(geom, beta, slide)
                   for n, chain in chains.items():
                       k = crossings(chain, beta, slide)
                       more += k > c
                       caught[n] += k == c
           check(more == 0 and caught["joints"] < caught["cuts"] < caught["sticks"] < 3600, f"12_13: at 3,600 placements no chain is crossed more than the coast, and finer chains match it more often {caught}")
           # each chain is a polygon: its average is exactly its length x 1/pi on
           # this sheet (by 12_9's argument); checked numerically for the coarsest
           joints_mean = simpson_mean(lambda b: travel(chains["joints"], b) / GAP, 0.0, 180.0, 3600)
           check(abs(joints_mean - float(sum(math.dist(g[1], g[2]) for g in chains["joints"])) * 2 / (math.pi * GAP)) < 1e-6, f"12_12: the joints chain averages {joints_mean:.6f}, its length over pi")

           # 12_15 and 12_16: the coin calibrates: two crossings at every
           # placement, a rim of 2 pi; so 1/pi a centimetre, and the coast's
           # shown length over pi gives the shown average
           check(fixed(float(f["coast.length_2dp"]) / math.pi, 2) == f["crofton.mean_crossings"], f"12_16: the shown {f['coast.length_2dp']} / pi rounds to the shown {f['crofton.mean_crossings']}")
           check(abs(int(f["coin.crossings"]) / rim - 1 / math.pi) < 1e-15, "12_15: two crossings for a rim of 2 pi is 1/pi a centimetre")

           # Chapter 13: the same 6 cm wire bent at its middle into a right angle
           # at home, its right arm along the lines and its left arm turned up
           # (swung clockwise about the middle, through the open upper left)
           corner = STICK_HOME
           bent = [("S", (corner[0] + STICK / 2, corner[1]), corner), ("S", corner, (corner[0], corner[1] + STICK / 2))]
           f["hook.home_count"] = str(crossings(bent, 0, first_slide))
           check(f["stick.home_along"] == "0" != f["hook.home_count"], f"13_2: lying along, the straight stick crosses 0; bent up, {f['hook.home_count']}")
           pin = (corner[0], first_slide)
           sweep = []
           for i in range(3601):
               beta = 180 * i / 3600
               nb = normal(beta)
               k = crossings(bent, beta, first_slide + (nb[0] - n0[0]) * pin[0] + (nb[1] - n0[1]) * pin[1])
               if not sweep or sweep[-1] != k:
                   sweep.append(k)
           f["hook.sweep"] = ",".join(map(str, sweep))
           check(sweep[0] == sweep[-1] == int(f["hook.home_count"]), f"13_3: half a turn of the sheet about the corner's line, counts {f['hook.sweep']}, back where it began")
           straight_sh, bent_sh = count_shares(stick_geom(STICK, 90), GAP), count_shares(bent, GAP)
           for name, sh in (("stick", straight_sh), ("hook", bent_sh)):
               f[f"{name}.shares_percent"] = ",".join(fixed(sh.get(k, 0.0) * 100, 1) for k in range(min(sh), max(sh) + 1))
               check(abs(moments(sh)[0] - STICK * 2 / (math.pi * GAP)) < 1e-9, f"13_4: the {name}'s closed-form chances average {moments(sh)[0]:.9f}, 6 / pi")
               grid = grid_shares(stick_geom(STICK, 90) if name == "stick" else bent, 3600, GAP)
               coarse = grid_shares(stick_geom(STICK, 90) if name == "stick" else bent, 36, GAP)
               gap_fine = max(abs(grid.get(k, 0) - sh.get(k, 0)) for k in set(grid) | set(sh))
               gap_coarse = max(abs(coarse.get(k, 0) - sh.get(k, 0)) for k in set(coarse) | set(sh))
               check(gap_fine < 1e-6 < 1e-4 < gap_coarse, f"13_4: the {name}'s chances over 3,600 directions agree with the closed form within {gap_fine:.1e}; over 36 they miss by {gap_coarse:.1e} (a direction grid is an estimate, not exact)")
           check(min(bent_sh) == 1 and 0 not in bent_sh and STICK / 2 / math.sqrt(2) > GAP, f"13_4: bent, it never misses: one arm always reaches {STICK / 2 / math.sqrt(2):.3f} cm across, more than a gap")
           check(f["hook.counts"] == ",".join(map(str, sorted(bent_sh))) and f["stick.all_counts"] == ",".join(map(str, sorted(straight_sh))), "13_4: the closed form's counts are the sampled ones")
           # 13_5: bent again into a "4" that crosses itself, flat on the card
           pts = [(corner[0] + FOUR_PIECES[0], corner[1]), corner]
           heading = 180.0
           for w in FOUR_PIECES[1:]:
               heading -= FOUR_TURN
               h = math.radians(heading)
               pts.append((pts[-1][0] + w * math.cos(h), pts[-1][1] + w * math.sin(h)))
           four = [("S", p, q) for p, q in zip(pts, pts[1:])]
           check(abs(sum(math.dist(g[1], g[2]) for g in four) - STICK) < 1e-12 and segments_cross(four[0][1], four[0][2], four[2][1], four[2][2]), "13_5: the 4 is still 6 cm and its last piece crosses its first")
           x_meet = (pts[2][0], corner[1])
           nf = normal(FOUR_SHEET)
           turned = first_slide + (nf[0] - n0[0]) * x_meet[0] + (nf[1] - n0[1]) * x_meet[1]
           shift = (x_meet[0] * nf[0] + x_meet[1] * nf[1] - turned) % GAP
           f["four.count"] = str(crossings(four, FOUR_SHEET, turned + shift))
           f["four.through"] = str(crossings([four[0]], FOUR_SHEET, turned + shift) + crossings([four[2]], FOUR_SHEET, turned + shift))
           four_sh = count_shares(four, GAP)
           f["four.mean"] = fixed(moments(four_sh)[0], 2)
           check(f["four.mean"] == f["hook.mean"] == f["stick.turned_mean"] and abs(moments(four_sh)[0] - STICK / math.pi) < 1e-9, f"13_5: the 4 averages {moments(four_sh)[0]:.9f}, its length over pi")
           check(f["four.through"] == "2", f"13_5: the line through the 4's crossing counts both pieces there ({f['four.count']} in all)")
           f["noodle.year"] = str(NOODLE_YEAR)
           # negative controls for 13
           folded = [("S", (-1.5, 0.0), (1.5, 0.0)), ("S", (1.5, 0.0), (-1.5, 0.0))]
           check(abs(moments(count_shares(folded, GAP))[0] - STICK / math.pi) < 1e-9 and abs(moments(count_shares(folded[:1], GAP))[0] - STICK / 2 / math.pi) < 1e-9, "negative control: folded double, counted per layer it averages 6/pi; counted per point of the card, 3/pi, as if 3 cm long")
           short = [("S", (-2.9, 0.0), (0.0, 0.0)), ("S", (0.0, 0.0), (0.0, 3.0))]
           check(fixed(moments(count_shares(short, GAP))[0], 2) != f["hook.mean"], f"negative control: a bend that loses 1 mm averages {fixed(moments(count_shares(short, GAP))[0], 2)}, not {f['hook.mean']}")
           for g in (1.0, 4.0):
               mg = moments(count_shares(bent, g))[0]
               check(abs(mg * worth(g) - STICK) < 1e-9 and fixed(mg, 2) != f["hook.mean"] and abs(mg * worth() - STICK) > 1.0, f"spacing control: lines {g:g} cm apart, the bent wire averages {mg:.4f}; times that sheet's worth {worth(g):.4f} it is 6 cm, times this sheet's pi it is not")
           four_dirs = sum(travel(bent, b) for b in (0, 45, 90, 135)) / 4 / GAP
           check(fixed(four_dirs, 2) != f["hook.mean"] and abs(four_dirs * math.pi - STICK) > 0.1, f"negative control: four directions estimate {four_dirs:.4f}, times pi {four_dirs * math.pi:.3f}, not 6: an estimate, not exact")
           # every wire shown keeps clear of the coast on the card
           coast_pts = sample_points(geom, 256)

           def clear_of_coast(wire):
               return min(min(math.dist(p, (g[1][0] + t / 40 * (g[2][0] - g[1][0]), g[1][1] + t / 40 * (g[2][1] - g[1][1]))) for t in range(41)) for g in wire for p in coast_pts)

           worst_wire = min(clear_of_coast(w) for w in (moved, low + high, high + crossed, bent, four))
           check(worst_wire > 0.3, f"12 and 13: every wire shown keeps {worst_wire:.3f} cm from the coast")

       chapters_12_13()
   ```

   It prints the thirty-three new keys above and forty-four checks, all of
   them passing. Its negative controls are inside it: a line through the
   crossed halves' meeting point counted as points of the card (one, not
   two); a 3 cm wire folded double counted per point (3 ÷ π, not 6 ÷ π); a
   bend that loses 1 mm (it would average 1.88, not 1.91); lines 1 and 4 cm
   apart (a different average, giving 6 cm only with that sheet's worth);
   36 directions instead of 3,600 (about 1e−3 from the closed form); and
   four directions only (1.81 crossings, 5.69 cm). Everything it binds
   stays inside `chapters_12_13`, so no later line of `main()` changes:
   all 258 old keys print the same.
5. `src/main.rs`. The SPOKEN and DEFINITIONAL rows above, after 10/11's.
6. `docs/storyboard.md`. Move the two chapter sections above in after
   10/11's "### Notes for 11", at the end of the file, add their bullets to
   "Differences from the full draft", and change the status line's
   "native chapters 01 to 11" to "01 to 13". Then regenerate the script
   with `python3 -I tools/storyboard.py crofton` and check it with
   `python3 -I tools/storyboard.py --check crofton`. Until then each
   chapter's `every_phrase_the_scene_waits_for_is_in_its_line` and main's
   `the_storyboard_the_registry_and_the_script_agree` fail. That is
   expected. `test_oracle.py`'s displayed-arithmetic test reads the moved
   pictures: their quoted equations follow from their shown operands
   (checked below with its own `equations()`).
7. `docs/pause-overrides.json`. Append the forty-four overrides under
   "Pauses" to its `overrides` list, after 11's (115 in all with 08/09's
   and 10/11's), and change its `_comment`'s "through chapters01–11" to
   "through chapters01–13".
8. Nothing in `glyphs.rs`: every string 12 and 13 set goes through the kit,
   so the probe collects it. The TeX uses `\div`, `\times`, `\approx`,
   `\pi`, `\le`, `\tfrac` and `\text` as 06 to 11 set them, plus "?" and
   "+" in math. The Texts need "%" (13_4's percents, as 06's "about 15%"
   and 09's "about 64%"), "." and ",". If `\tfrac` fails the probe, the 12_9
   row can read `\text{three quarters of}` with the same SPOKEN rows.
9. Then Cargo fmt, Clippy and the tests, `facts-check`, the glyph probe and
   a first render of 12 and 13 for the frame risks below.

Clippy lessons from 01 to 11 are applied, though Clippy has not run.
Every `pub` item and field is read in non-test code: 12's `Wire` fields by
its own drawing, `Words`' fields by both scenes, and `BARBIER_YEAR`,
`NOODLE_YEAR`, `shape`, `sweep`, `shares`, `four_meet` and `four_move` by
the scenes and step 3. A cheap check (below) confirms every imported name
is used. All-constant asserts are in `const { }` blocks. Range tests are
written `(a..=b).contains(&x)`. No function takes more than seven
arguments. Neither file calls `.rt()` or `.lag()` on an animation, or
`set_opacity` on a stroked line: the coast's tones, the bay's halos and
the dimmed coast all go through `dim_line`.

## Frame risks to reread at the first render

- 12's opening rebuilds 11's last frame from 11's `words()` and plot
  helpers. It must match 11's end pixel for pixel: the plot's axes, curve,
  fill, mean line and list, 10's stick ticks and 09's panels. Any
  difference there is a visible jump at the cut.
- Lower thirds: 12_2's source, 12_7's and 12_14's "precisely", 13_5's
  "precisely" and 13_6's source. The two-line ones top out at frame
  y −2.78, and the console's low row (y −3.05) is empty while each shows.
  The test checks that row, but the render must confirm no console text
  shows through.
- 13_2's raised arm reaches card y 4.2, frame y 3.61, under the card's top
  edge (4.25 card cm) and left of the count's digits (frame x −3.6 against
  the digits' left near −3.15). Check the count's word and the arm's
  stroke do not touch.
- The moving lines pass under the wire, its dots, the rings, the coin and
  the chains in 12_4, 12_5, 12_7, 12_8, 12_10, 12_16, 13_3 and 13_5. The
  card's labels fade before every move that would sweep lines through
  them (13_3's arm labels, 12_10's counts on the console only).
- 12_8 and 13_5 place a line exactly through a crossing on purpose. Both
  of its dots sit at one point, drawn twice. Check that the double dot does
  not read as a brighter blob that needs explaining. The ring is there to
  show it.
- 12_16's slide passes the instant when the coin's centre lies midway
  between two lines, both of them tangent: strictly, a count of 0 for one
  exact slide. A frame landing on it would show "0 crossings" for one
  frame. The test samples 3,000 placements and finds 2 at each, but the
  render's frames are another sample. Read the counter through the slide.
- 13_4's 0.1% bar is drawn at the 0.01 minimum height. It should read as a
  sliver under "0.1%", not as a gap.
- `Curve::count_shares` is valid for connected chains, and every wire 13
  shares is one. A wire in separate pieces would need its own count.
- 12_12's first sentence carries 6.4 s of fades and moves before "Here's
  a chain". If the measured voice is faster than the silent-render
  estimate, the chain arrives after its words. Check against the render.

## Timing, estimated and not measured

The tool's estimates count words, a breath, the prediction pause and a
2.5 s hold: 306.3 s for 12 and 144.9 s for 13. 12 is long because it
carries the argument. The former 85 s target is an advisory, not a cap, and
the assignment allows as much time as is needed. 12 has two conceptual
boundaries if root wants shorter scenes: between 12_9 (every fraction) and
12_10 (every length), and between 12_14 (the limit) and 12_15 (the coin).
Neither split needs a change to any words, only new ids. The check's
silent-render timeline uses the scenes' own animations and no narrator
(2.6 words a second). It comes to about 306 s for 12 and 149 s for 13
with no pauses, and 325 s and 156 s with them at their least, including
the countdowns and stands. Scene seconds below are what the code plays.

- 12: the fades 0.4 s; the slide 8 s (12_4); the stick's moves 2.5 s each
  and the sheet's turns 3 s each (12_5, 12_6); the slide past the cut 10 s
  (12_7); the swing 3 s and the slide onto the meeting point 2 s (12_8);
  the swing back 3 s and the slide back 2 s (12_9); the quarter turn and
  the slide 3 s each (12_10), and back (12_12); the countdown 4 s (12_15);
  the coin's slide 4 s and half turn 6 s (12_16); the stand 2.6 s (12_17).
- 13: the turn 2.5 s and the bend 3 s (13_2); the countdown 4 s (13_2);
  the half turn 12 s (13_3); the bend into the 4 3 s and the sheet's move
  3 s (13_5); the move back 3 s (13_7); the stand 2.6 s (13_7).
- Phrase moments come from each sentence's share of characters, so with
  no narrator they are estimates.

## Checks run (2026-10-24; GentleLantern)

No Cargo, rustc, Clippy, TTS or render. Every result below is from Python
or rustfmt.

- `rustfmt --edition 2024 --check` (the repo's `rustfmt.toml`) on both new
  files exits 0, and on step 3's block set in a scratch function. The two
  lines over 120 characters in 12 are string literals (12_9's TeX and its
  test), which rustfmt leaves.
- The fmn and kit calls were read against the chapters and the kit as they
  stand: `placement.rs` (`Move::turn`, `slide`, `at`, `end`, `at_once`,
  `glide`, `tracked`, `faded_line`, `halo_along`, `live_dots`,
  `live_count`, `half_turn_pin`, `Placement::begin`, `run`, `make`,
  `settle`, `pose`), `draw.rs` (`say`, `think`, `stand`, `fade_to`,
  `dim_line`, `together`, `path`, `flat`, `line_style`), `sheet.rs`
  (`crossings`, `shown_slide`), `model.rs` (`Curve::count_shares`,
  `moments`, `worth`), and 07's to 11's frozen public items.
- `oracle.py` as it stands: 230 facts, 0 failed cross-checks. With
  08/09's and 10/11's steps 4 applied to a scratch copy (their
  `apply_block3` and `apply_block4`): 258 facts, 0 failed.
- `apply_block5` (session scratch) applies step 4 to that copy, once from
  the block files and once read from this file (`--doc`). Both give the
  same oracle, byte for byte, and the same facts: 291 facts, 0 failed, the
  258 old keys unchanged and 33 new. All 44 of block 5's own checks pass.
- `mirror_12` (session scratch): a Python transliteration of 12's model
  (`placed`, `halves`, `moved_back`, `along_ends`, `cuts`, `sticks`,
  `chain`, `chain_points`, `nested_moves`, `coin_rim`, `count_at`), of
  every refusal in its `words()` and of its model tests, on `sheet_mirror`
  (sheet.rs and the model). It runs against the applied oracle's facts:
  64 ok, 0 failed, after `sheet_mirror`'s own checks (0 failed). Every key
  12 reads equals the oracle's. Its INDEPENDENT parts:
  - a fine-chord floor count agrees with the piece-by-piece count at 900
    placements;
  - moving the sheet the same way as the stick, not back, differs at 291 of
    400 random moves (the control on 12_5);
  - lines just beside the crossed halves' meeting point cross each half
    once;
  - over 36,000 grid placements the coast's average count is 8.2160
    against L ÷ π = 8.2165, and each chain's lies within 1e−3 of its own
    length over π.
- `mirror_13` (session scratch): 13's shape, placed as 12 places it,
  counted strictly piece by piece, and its spreads by a replica of
  `Curve::count_shares`, against the applied oracle's closed forms and
  facts. It runs every number 13 shows or says, with its rounding margin,
  and the single 13_7 return move: 41 ok, 0 failed. Its controls:
  - a 5.9 cm bend gives 1.88;
  - 36 directions are off by more than 1e−4;
  - four directions give 1.81 and 5.69 cm;
  - lines 1 and 4 cm apart give 6 cm only with their own worth;
  - a fold counts twice.
- `imports_check` (session scratch): every name either file imports is
  used in its non-test code (or, for the tests' imports, its tests). Every
  `pub` item and field is read in non-test code. Every waited phrase is
  once in its line, in order, and the proposed lines keep the words rule.
  Result: 534 ok, 0 failed.
- `candidate1213_check` (session scratch) runs `tools/storyboard.py`'s own
  `parse` and `check` on this file, writing nothing, and checks:
  - the words rule, and the estimates in the headings and the timing
    section;
  - every line said once, in order, by its scene, and the phrases each
    scene waits for (literal or from a loop's list), read from its source
    and its test, each once in its line;
  - `main.rs`'s number guard (its own word list and matching rule) with
    the rows above, against the applied oracle's facts, plus five negative
    controls;
  - the keys the scenes read, against the oracle and against `facts.rs`
    with 08/09's, 10/11's and this step 3;
  - the doc's facts block against the one rustfmt checked, and its oracle
    blocks against the ones run;
  - the fourteen quoted equations, with `test_oracle.py`'s own
    `equations()`, and seven controls. It also confirms that the chain rule
    misreads the full 13_1 row, which is why that row is quoted in two
    steps;
  - the pause overrides, with `narration_director.directions()`, on
    `script.tsv` with 08/09's, 10/11's and these lines added, alone and with
    the other seventy-one, and two refusal controls;
  - the silent-render timeline;
  - the handed-off files' hashes against the table below.

  Result: 504 ok, 0 failed, the two hash rows included. The storyboard
  tool's advisories (306.3 s and 144.9 s, above its 85 s hint) are
  advisory only, as the tool itself says.
- These are transliterations of my own Rust. They test its arithmetic and
  its assertions, not that it compiles or that the transliteration is
  exact. None of the files' own Rust tests has run: twelve in 12 and
  eleven in 13, the layout tests and both
  `every_phrase_the_scene_waits_for_is_in_its_line` among them.

Not run, and needed before acceptance: Cargo fmt, Clippy (`-D warnings`),
`cargo test`, `facts-check`, the glyph probe, any render, TTS, listening,
and `tools/storyboard.py --check crofton` on the integrated storyboard.

## Files handed off

| file | sha256 | bytes |
|---|---|---|
| `src/chapters/why_every_curve.rs` | dad36a04d7e97d0e6e0e4be73c7a89c2402b37c96c490003c60044b3973de614 | 72119 |
| `src/chapters/the_bent_stick.rs` | 5fc97eceddab1941e0d20eb802142053d7c7e071277728e8daaea420068fe57f | 36556 |

This file's own hash is in the handoff mail.
