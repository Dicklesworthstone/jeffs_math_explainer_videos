# Storyboard — A Resistor Network Is Secretly Answering a Probability Question

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py resistor_random_walk` checks it (gate G2) and
writes `script.tsv` from the words column. Every number below is a fact in
`docs/fact-sheet.md`, computed in `src/facts.rs` and recomputed by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One tiny network runs through the whole video: four junctions, identical
wires, a pair of wires side by side from the top to the left and from the
right to the bottom (a Wheatstone bridge). The top is held at one volt and
the bottom at zero; the left junction sits at 3/5 and the right at 2/5, and a
walker from the left reaches the top first with chance 3/5. The opening
previews the whole payoff; the later chapters earn each step.

| Chapter | Spine | Status |
|---|---|---|
| `01_the_same_number` | the circuit: current in equals current out, so each middle junction is the average of its wires' far ends (3/5 and 2/5 pass); the walker on the same wires: one step ahead, its chance is the same average; prediction; the same number, because one rule with the same top and bottom has only one solution; the walker is a model of the equations, not of electrons | source at 38e52c0 (`mx-video-program-program-87do.18.1`); root's central05 batch passed fmt, strict release clippy, 32 tests, the release build, the 47-fact oracle comparison, the glyph probe and a silent scratch render of the opening, and root viewed its final and prediction frames and the three glyph pages; no audio, music or full-video acceptance; the task's closure is root's, pending its receipt |
| `02_why_an_average` | at the left junction, wire by wire: each wire's current is its voltage drop over the same resistance; the pair's two drops of 2/5 flow in, 1/5 and 3/5 flow out, and they balance; with the voltage unknown the balance is 4v = 1 + 1 + 2/5 + 0, the average; the walker's first step over the same four wires, up with chance 1/2, not 1/3 (prediction); the first-step equation gives each wire the same quarter; the general rule weights each far end by its wire's conductance, and the walker chooses wires in that proportion | source by SandyMountain for `mx-video-program-program-87do.18.2`, corrected after review 3798; root's central10 batch passed fmt, strict release all-target clippy, 38 tests, the release build and the 72/72 fact comparison with the independent oracle (12 checks, numpy included); its silent scratch render draft04 ran 2448 frames, 81.6 s, with an empty layout lint, and root viewed the corrected 02_1, 02_3, 02_7 and 02_8 frames and all eight glyph pages (59 sources); an independent Opus review (3827) accepted the corrected source; accepted as source, native checks and sampled visuals only, with no audio, music or full-video acceptance; committed at d17decc, and root closed the task |
| `03_round_after_round` | guess zero at both middle junctions and average, every junction at once from the previous round's values (Jacobi's method, chosen and kept); the real rounds 1/2 and 1/4, 9/16 and 3/8, …; the bigger gap shrinks to exactly a quarter a round because one wire at each junction leads to an unknown; prediction: no round ever reaches 3/5 (power-of-two denominators), though five rounds come within a thousandth; the limit is the pair that keeps every average; the ends must stay fixed, or everything drifts to 1/2 | source by SandyMountain for `mx-video-program-program-87do.18.3`; root's central17 batch passed fmt, strict clippy, 43 tests, the release build and the 104/104 fact comparison with the independent oracle (15 checks); draft05's short card dwell, early round-5 example and battery-held table beside the counterfactual (EmeraldLeopard, 3927) were repaired (3954); central20 passed the build, fmt, strict clippy, the 5 targeted tests and the glyph probe (100 sources), and draft06 ran 2489 frames, 82.97 s, with an empty layout lint and its cards up about 13.8, 13.0, 18.9 and 8.3 s; root viewed the revised cards and glyph pages, and EmeraldLeopard (3964) verified the repair; accepted as source and sampled visuals only, with no audio, music or full-video acceptance; committed at 39eed75, and root closed the task |
| `04_only_one` | two solutions with the same top and bottom differ by a difference that is zero at both and keeps the rule; its largest value, if above zero, sits at a middle junction averaging far ends no larger, so it spreads along wires to the top, where it is zero; likewise the lowest; so one solution. Prediction: with an island of two junctions wired only to each other, is there still one? No: any equal pair keeps its rule, so every middle junction must reach the top or the bottom; a walker there has chance zero of reaching the top, while its voltage is free | source by SandyMountain for `mx-video-program-program-87do.18.4`; an independent math review (4027) passed and root narrowed two cards (4030); central29 failed strict clippy (`eq_op` in `facts::difference_ends`), repaired (4047); central31 passed fmt, strict clippy, 51 tests, the 121-fact oracle comparison, the glyph probe (123 sources) and draft09 (2470 frames, 82.33 s), whose layout lint flagged overlapping peak tokens at 32.6 s, repaired (4058); central33 passed fmt, strict clippy, the six changed scene tests and draft10 (2470 frames, 82.33 s, empty layout lint); root viewed 11 frames and all 15 glyph pages, and TanElm (4069) found no concrete defect in the actual frames, cards up 14.63, 23.20 and 15.67 s; accepted as source, math and sampled silent visuals only, with no audio, music or full-video acceptance; root closes the task |
| later chapters (not yet boarded) | every route added up for this network (a geometric series) against the rule that needs no routes; the current as the walker's net crossings (Doyle and Snell §1.3.3) | proposals only |

## Vocabulary (one word per idea)

**junction** (never "node" or "vertex" in the voice); **wire** (one identical
resistor; never "edge", "link", "branch" or "resistor" in the voice; two
wires side by side are two wires); **far end** (the junction at a wire's other
end); **the top** and **the bottom** (the two junctions held fixed; never
"boundary"); **middle junction** (any other junction); **voltage** and
**volt**; **current**; **walker**; **chance** (never "probability" in the
voice; "chance of reaching the top first" is the walk's number); **average**
(the plain average over a junction's wires, so a doubled pair counts its far
end twice); **rule** (every middle junction is the average of its wires' far
ends); **solution** (a set of values that keeps the rule); **drop** (the
voltage difference across one wire, in volts; never "potential difference"
in the voice); **pair** (the two wires side by side from the top to the
left); **difference** (chapter 04: two solutions subtracted, junction by
junction); **peak** (a difference's largest value); **island** (junctions
wired only to each other, with no path of wires to the top or the bottom); **conductance** (chapter 02 on: how easily a wire carries current,
one over its resistance).

## Color roles

| Idea | Color |
|---|---|
| the top: one volt, a chance of one | gold `#F0AC5F` |
| the bottom: zero volts, a chance of zero | blue `#58C4DD` |
| a middle junction once its value is known, and a value travelling along a wire | between blue and gold in proportion to the value (`palette::value`) |
| a middle junction whose value is not yet shown | `faint` |
| the walker | green `#83C167` |
| wires and the battery | `dim` |
| the unknown in the prediction | `hero` `#FFD866`, the "?" only |
| a current: a wire's drop and the arrow of its flow (chapter 02) | teal `#5CD0B3`, text and strokes only |
| a wire not under discussion | `faint` |
| the pictures' names, units and notes | `dim`; values in `ink` |

## 01_the_same_number: The same number, twice (~84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Identical wires join four junctions, doubled in two places. A battery holds the top at one volt and the bottom at zero. | The four junctions appear (top gold, bottom blue, the two middle ones faint), then the wires: a pair side by side from the top to the left and from the right to the bottom, one wire elsewhere. On "doubled" the two pairs flash. On "A battery" the battery's wire runs from the top round to the bottom on the right, with "1 volt" by the top and "0 volts" by the bottom | junctions and wires built from the network in facts, highlight | |
| 01_2 | Current in equals current out, so a middle junction's voltage is the average of the voltages at the far ends of its four wires. | The left junction's four wires flash, then the right's. A "precisely" lower-third: Ohm's law (each wire carries its conductance times the voltage across it), Kirchhoff's current law (they add to zero at a middle junction), so the voltage is the conductance-weighted average; wires side by side add their conductances | highlight, lower-third | |
| 01_3 | On the left, one, one, two fifths and zero average to three fifths. | The card goes. The right junction takes its value, 2/5. Four values leave the far ends of the left junction's wires (1, 1 from the doubled pair, 2/5, 0), each in its color, and slide along the wires into it; beneath the circuit the equation (1 + 1 + 2/5 + 0)/4 = 3/5 writes itself and the left junction turns its color, labelled 3/5. Then, quickly and without words, the right junction's check: 1, 3/5, 0, 0 into it, the equation swapping to (1 + 3/5 + 0 + 0)/4 = 2/5 | values travelling along wires, equation, fraction | |
| 01_4 | Now forget the battery. A walker at the left junction picks one of its four wires at random, and walks on until it reaches the top or the bottom. | The equation goes; the circuit slides left under its name, "voltage". On the right, the same junctions and wires again with no battery, under "chance of reaching the top first"; its middle junctions faint. A green walker appears at the left junction; on "at random" its four wires flash; on "walks on" it takes the drawn route, over to the right, back, over again and up to the top, which flashes | motion, highlight, a second copy of the network | |
| 01_5 | Its routes never run out, so look one step ahead. Its chance of reaching the top first is the average of its chances from the far ends of its four wires. | The walker is back at the left junction. A "precisely" lower-third: the chances it is still walking after one, two and three steps (1/4, 1/16, 1/64, from facts), never zero but shrinking towards it, so it stops with chance one. On "look one step ahead" its four wires flash, and a "?" waits at each far end | highlight, lower-third | |
| 01_6 | From the top the chance is one, and from the bottom it's zero. So what is it from the left junction? ⟨predict⟩ | The card goes. The far ends' "?" at the top become 1, the one at the bottom 0; the right junction's stays "?". The left junction carries a large gold "?": the question is on screen by its last word, then the countdown | prediction pause, think time | |
| 01_7 | Three fifths again. One rule, the same top and bottom, and only one solution. So the voltage and the chance must agree. | The "?" become 2/5 at the right junction and 3/5 at the left; the same four values slide into the left junction and the same equation forms beneath the walk. On "One rule" both pictures' equations stand side by side and both 3/5 labels flash. On "only one solution" a "precisely" lower-third, its hypothesis first: here every middle junction is wired to the top or bottom; two solutions with the same top and bottom differ by a solution that is zero at both; its largest value spreads along wires to them, so it is zero, and so is its smallest (the card is built only if the model finds no junction cut off) | equation, fraction, highlight, lower-third | |
| 01_8 | The walker is a model, not a claim about electrons. The two numbers agree because they solve the same equations. | The card goes. Under the walk two dim notes: the video's own scope statement, "a model of the equations, not of electrons", and below it the theorem's source, "voltage equals chance: Doyle and Snell, section 1.3.2" (their remark on realism is §1.3.3's, about current, and is not put on screen); the walker fades; the two pictures stand with their equal numbers, then clear | text | |

### Notes for 01

- The circuit's numbers never leave the screen while the walk is solved, so
  the prediction (01_6) is honest: will the same rule on the same wires give
  the same number? Its answer stays hidden at the walk's left junction
  through the four-second countdown and arrives with 01_7.
- Exact versions kept on screen (three lower-thirds, each at most three
  lines): 01_2 (Ohm's law and Kirchhoff's current law give the
  conductance-weighted average; wires side by side add their conductances,
  Doyle and Snell p. 43); 01_5 (the walk stops with chance one: still going
  after n steps with chance 1/4ⁿ here, `R-F3.still_walking`); 01_7 (why the
  solution is unique: the maximum principle, Doyle and Snell §1.1.5; it needs
  every middle junction joined by wires to the top or the bottom, and the
  edge is an island of two junctions joined only to each other, where the
  balance's determinant is 0, `R-X2`).
- "Only one solution" is said of this network and every network of positive
  conductances whose middle junctions all reach the top or the bottom; the
  family check (`R-X3`, 729 networks on four junctions) and the island are its
  edges. A negative conductance is refused by the model: it is no resistor,
  and the walk would have no chances.
- The walker's caveat is the slate's scope limit: the walk is a model of the
  same equations, not a description of how charge moves in a metal. Doyle and
  Snell's own probabilistic reading of current (§1.3.3) starts from "a naive
  view of the process of electrical conduction" and says "realism is not
  what we're after"; the video makes no physical claim beyond Ohm's law and
  Kirchhoff's current law.
- Pauses for the narration (no audio exists for this chapter): the one
  question is the 01_6 prediction, held with the countdown, not a rhetorical
  question answered at once; a short reveal pause after "Three fifths again."
  while the labels land; a completed-thought pause before "Now forget the
  battery." (the picture changes) and before 01_8 (the caveat is its own
  thought). These are directions for the central narration lane, which must
  follow the current owner amendment before any synthesis.

## 02_why_an_average: Why each junction is an average (~82 s; silent draft 81.6 s)

The circuit stands where chapter 01 left it, on the left; the derivation
builds on the right, row by row, every number from `R-F4` to `R-F6`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | At the left junction, each wire's current is its voltage drop divided by the wires' shared resistance. | The circuit from chapter 01 (1 volt, 0 volts, 3/5, 2/5, the battery, "voltage" over it). The three wires away from the left junction fade to faint; the left junction's four wires flash. A "precisely" lower-third: Ohm's law, for ideal resistors with steady currents, current = drop ÷ resistance; identical wires share one resistance | highlight, lower-third | |
| 02_2 | Each wire of the pair drops two fifths of a volt. Then one fifth to the right, three fifths to the bottom. | The card goes. Beside each wire of the pair, "2/5" in the current's teal, and on the wire a teal arrowhead pointing into the junction; then "1/5" and "3/5" beside the wires to the right and to the bottom, their arrowheads pointing out | values beside wires, arrows | |
| 02_3 | Current in equals current out, so the drops balance too. Two fifths and two fifths in, one fifth and three fifths out. | On the right, the first rows of the derivation, labelled as drops in volts, not currents: "drops in: 2/5 + 2/5 = 4/5 V", then "drops out: 1/5 + 3/5 = 4/5 V". A "precisely" lower-third: Kirchhoff's current law; each current is its drop over the same resistance, so the drops balance as the currents do | equation, lower-third | |
| 02_4 | With the voltage unknown, the balance says four times it equals one plus one plus two fifths plus zero. So it's their average. | The card goes. The left label 3/5 becomes v. Rows: (1 − v) + (1 − v) + (2/5 − v) + (0 − v) = 0; then 4v = 1 + 1 + 2/5 + 0; then v = (1 + 1 + 2/5 + 0)/4 = 3/5. The drops, arrows and earlier rows go; v's equation rises to the top row | equation, derivation, formula | |
| 02_5 | Now the walker picks one of the four wires, each equally likely. What's the chance its first step reaches the top? ⟨predict⟩ | The battery, its units and "voltage" go; "chance of reaching the top first" names the same picture; the top reads 1, the bottom 0, the right 2/5, the left "?". The walker appears at the left junction and its four wires flash. Nothing shows a step's chance: the countdown runs in the panel | prediction pause, think time | |
| 02_6 | One half, not one third. The pair is two of the four wires, so it counts twice, like its two currents. | "1/4" beside each of the four wires in the walker's green; the pair's two quarters slide together into "1/2" by the pair. A row: "first step up: 2/4 = 1/2". A "precisely" lower-third: counting the pair as one wire makes another network, where the first step goes up with chance 1/3 and the left junction sits at 5/11, as a voltage and as a chance | values beside wires, merge, equation, lower-third | |
| 02_7 | Up, it has reached the top. Right, its chance is the right junction's. Down, it's zero. A quarter of each, added, is the same average. | The card goes. The far ends' chances replace the quarters (1, 1, 2/5, 0). On "Right, its chance" the right junction's label flashes and a row names where its number comes from: p(right) = 2/5, found in chapter 01. Then rows under v's: p = ¼·1 + ¼·1 + ¼·2/5 + ¼·0, then p = (1 + 1 + 2/5 + 0)/4 = 3/5, the same fraction as v's, and both rows flash; the left's "?" becomes 3/5. A "precisely" lower-third: after its first step the walker starts afresh from where it landed, with no memory of how it got there, so its chance from there is that junction's | equation, highlight, lower-third | |
| 02_8 | With unequal wires, weight each far end by its wire's conductance, and the walker chooses wires in that proportion. The same rule. | The card goes. The left junction's links are labelled with their conductances (2 for the pair, 1 and 1) and its weights 2/4, 1/4, 1/4. The rows below give way to the general forms, v(x) = Σ (c(x,y) / c(x)) v(y) and p(x) = Σ (c(x,y) / c(x)) p(y), one under the other, with their domain in a dim three-line note: x is any middle junction (the top and the bottom stay fixed); y runs over the junctions wired to x, c(x, y) being their wires' conductance, positive, one over the resistance, wires side by side added; c(x) is their sum at x. *(Not displayed: at the top the same average would be 8/15, not 1; that edge is a fact and a test, `R-X5.top_average`, not on screen)* | formula, equation | |

### Notes for 02

- The derivation is the left junction's own: Ohm's law per identical wire,
  then the balance, then the unknown gathered. The right junction's value
  (2/5) is used as known; chapter 01 showed both values keep the rule, and
  why the pair of averages has only one solution. In the walk, 02_7 names
  the right's chance on screen as chapter 01's before it is substituted, so
  the step is a check of the rule, not a circular solve.
- The general rule (02_8) holds at middle junctions only, summing over the
  junctions wired to x (every c(x, y) there is positive); the top and the
  bottom are fixed, and the same average at the top would be 8/15, not 1
  (`R-X5.top_average`, a fact and a test; the screen shows only the domain,
  not this number).
- The panel's flow rows add drops in volts, labelled as drops; a current is
  a drop over the shared resistance, and 02_3 says so ("so the drops
  balance too").
- Signs: a drop is the far end's voltage minus the junction's, positive
  where current flows in (`R-F4.drops_into` is 2/5, 2/5, −1/5, −3/5, adding to
  0); the spoken drops are their sizes along the flow (`R-F4.drop.*`).
- Units and scope: drops are in volts; a current is its drop over the
  wires' common resistance, so it is never given a number without that
  resistance. Ideal resistors, steady currents, an ideal battery; Ohm's law
  and Kirchhoff's current law are the only physics claimed.
- Prediction (02_5): the tempting answer is 1/3 (three junctions); the
  answer is 1/2 because the walker picks among wires, and the pair is two
  of them (`R-F5`). Nothing on screen gives a step's chance until the
  countdown ends; the quarters arrive with 02_6.
- The pair matters (`R-X4`): counted as one wire, the network changes to
  left 5/11 and right 4/11 in both pictures, and the first step up has chance
  1/3. Parallel wires are two wires, not one; their conductances add
  (`R-F6.pair_adds`).
- Pauses for the narration (no audio exists): one prediction hold after
  02_5; a reveal pause after "One half, not one third."; completed-thought
  pauses before 02_5 (the picture changes from circuit to walk) and before
  02_8 (the general rule). Directions only; no synthesis under the current
  owner amendment.

## 03_round_after_round: Average, and average again (~83 s)

The circuit stands where chapter 02 had it, on the left; on the right a
table of rounds: round, left, right, and later the bigger gap. Every number
is `R-F7`, `R-F8` or `R-X6`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Here's a way to find the middle voltages without solving anything. Guess zero for both, then average. | The circuit (battery, 1 volt, 0 volts), its middle junctions labelled 0 and coloured as 0. On the right the table's header, round, left, right, and its first row: 0, 0, 0. On "Guess zero" the two zeros flash | table, highlight | |
| 03_2 | Each round, every middle junction takes the average of the far ends of its four wires, using the values from the round before. | Both middle junctions' wires flash. On "using the values" a "precisely" lower-third: every middle junction updates at once from the previous round (Jacobi's method); one at a time with the newest values (Gauss-Seidel), the gap shrinks to 1/16 a round from round 2 on. It stays over round one, which shows it | highlight, lower-third | |
| 03_3 | Round one. The left averages one, one, zero and zero to one half. The right averages one and three zeros to one quarter. | Both junctions' far-end values from round 0 appear at once beside their wires (left 1, 1, 0, 0; right 1, 0, 0, 0). The left's slide in and it becomes 1/2; then the right's slide in, its token from the left still reading the old 0, and it becomes 1/4. The table's row 1: 1/2, 1/4. Each junction's colour moves with its value | values travelling along wires, table | |
| 03_4 | Round two uses those. Nine sixteenths and three eighths. Then nineteen thirty-seconds and twenty-five sixty-fourths. | The card goes as round 2's far-end values (left 1, 1, 1/4, 0; right 1, 1/2, 0, 0) appear; they slide into both junctions together; 9/16 and 3/8, row 2. On "Then nineteen" round 3 lands without tokens (19/32, 25/64, row 3), and round 4 after it without words (153/256, 51/128, row 4) | values travelling along wires, table | |
| 03_5 | The bigger gap to the answer shrinks to a quarter every round, because only one wire at each junction leads to an unknown. | The table's fourth column, "bigger gap", fills down rounds 0 to 4: 3/5, 3/20, 3/80, 3/320, 3/1280, and a "precisely" lower-third comes in under it: 3 of each junction's 4 far ends never change, so its new gap is 1/4 of the other's old one: the bigger gap is exactly 1/4 of the last, 3/5, 3/20, 3/80, 3/320 and 3/1280 by round 4 (the card stops at the rows on screen, so round 5 waits for the prediction). On "only one wire" the one wire joining the two middle junctions flashes | table, highlight, lower-third | |
| 03_6 | So after how many rounds is the left exactly three fifths? ⟨predict⟩ | The card stays while the question is asked and goes as it ends. Through the countdown the table holds rounds 0 to 4 and nothing else; nothing shows round 5 or a limit. The countdown sits in the panel under the table | prediction pause, think time | |
| 03_7 | Never. Each round makes halves, quarters, eighths and so on, and three fifths is none of them. But after five rounds both are within a thousandth. | A "precisely" lower-third: each new value is the other junction's previous value plus a whole number, divided by 4, so from the guess of 0 every value is over a power of 2, like 9/16 and 153/256 (rows already shown), and never 3/5; the gaps go to 0, so the values have a limit. It stays over 03_8. On "But after five rounds" row 5 lands, 307/512, 409/1024, gap 3/5120, and the junction labels follow | lower-third, table, highlight | |
| 03_8 | The values close in on three fifths and two fifths, the only pair that keeps every average. | The 03_7 card stays. Under the table, set apart, a "limit" row: 3/5, 2/5, gap 0. The labels on the junctions stay at round 5's values: the picture shows rounds, the limit row shows where they go | table, highlight | |
| 03_9 | The top and bottom must stay fixed. Average them too, and every junction drifts to the same number. | A counterfactual, shown as one. As the line begins the whole table (its rounds and its battery-held limit row 3/5, 2/5) and the 03_7 card go, and a "precisely" lower-third comes: with the top and bottom averaged too, 1, 3/5, 2/5, 0 drift to 1/2, their mean weighted by wires 3, 4, 4, 3. On "Average them too" the battery, its two labels and the middle labels fade; all four junctions are averaged together from the solved voltages, round by round (6 rounds, to within 1/1000, `R-X6.within`), and their colours run together to one. No number on screen then belongs to the battery-held network | colour drift, lower-third | |

### Notes for 03

- The chapter chooses previous-round updates (Jacobi) and keeps them: every
  round's two new values are computed from the round before, which is why
  round 1's token from the left still reads 0 while the left already shows
  1/2. One-at-a-time updates (Gauss-Seidel) appear only in the 03_2 card,
  as a comparison (`R-F8`).
- The error bound is exact, not estimated: each middle junction's only
  wire to an unknown carries weight 1/4 (`R-F7.unknown_wires`,
  `R-F7.unknown_weight`), every other far end is fixed, so the bigger gap is
  3/5 over 4 to the n (`R-F7.gaps`, `R-F7.shrink`, checked over 40 rounds by
  the model and in closed form by the oracle). "Within a thousandth after
  five rounds" is `R-F7.within` against `R-F7.tolerance`.
- Finite against limit: no round equals 3/5 or 2/5 (`R-F7.dyadic`,
  `R-F7.never`, 40 rounds checked; the card gives the reason for every
  round). The limit row is the solved pair (`R-F7.limit`); that the limit
  keeps every average, and is the only pair that does, is chapter 01's
  claim, proved in a later chapter.
- Edges: the ends averaged too (`R-X6`: the weighted mean 1/2 is kept every
  round and, on this network, every value settles there) shows why the ends
  stay fixed; 03_9 says it as a counterfactual. A kept mean alone does not
  make values settle: two junctions joined by one wire, both averaged, swap
  1 and 0 for ever (a model test and an oracle check, not shown). An island
  of two junctions wired only to each other swaps its values for ever too
  (`R-X7`, a fact and a test, not shown). On screen the drift is claimed
  for the network shown only.
- Prediction (03_6): the tempting answers are a round count read off the
  table; the answer is never. Nothing shows round 5 or the limit before
  the countdown ends.
- Card dwell: every lower-third stays up at least its reading time at
  fifteen characters a second, counted from when it is fully in (the
  scene's `wait_until`): 03_2's over round one (03_3), 03_5's from the gap
  column to the question's end, 03_7's over the limit (03_8), 03_9's from
  the line's start. By the scratch voice each fits inside the lines it
  stays over, so no time is added; a test checks it.
- Pauses for the narration (no audio exists): the one prediction hold after
  03_6; a reveal pause after "Never."; completed-thought pauses before 03_5
  (the gaps) and before 03_9 (the edge). Directions only; no synthesis under
  the current owner amendment.

## 04_only_one: Why only one solution (~83 s)

The circuit stands on the left as chapter 03 left it, the battery holding
1 volt and 0 volts, its middle junctions at 3/5 and 2/5. On the right, a
second copy of the network for the supposed second solution, which becomes
their difference; later, in its place, an island beside the circuit. Every
number is `R-X1`, `R-F9` or `R-X8`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Here's why there is only one solution. Suppose another set of values keeps the rule, with the same top and bottom. | The circuit with its values under "one solution". On "Suppose another" a second copy of the network on the right under "a second solution?", the same 1 at its top and 0 at its bottom, a "?" at each middle junction. On "the same top and bottom" both copies' tops and bottoms flash | a second copy of the network, highlight | |
| 04_2 | Subtract them. The difference is zero at the top and bottom, and it keeps the rule too. | The right copy becomes the difference, its name "their difference": its top's 1 becomes 0 and the top turns the bottom's colour; its bottom keeps 0; the "?" stay. On "The difference is zero" the two 0s flash and a "precisely" lower-third comes: at every middle junction each solution is the conductance-weighted average of the same far ends, so their difference is the average of the far ends' differences; at the top it is 1 − 1, at the bottom 0 − 0, both 0. On "keeps the rule" the copy's middle junctions' wires flash | highlight, lower-third | |
| 04_3 | Take its largest value. If above zero, it sits at a middle junction, an average of far ends no larger. | In the difference, the left junction's "?" becomes M, its largest value, and the junction turns the hero colour. On "If above zero" the top's and bottom's 0s flash: a value above them cannot sit there. On "an average of far ends" its four far ends' values appear beside its wires: 0 and 0 from the pair to the top, at most M from the right, 0 from the bottom | highlight, values beside wires | |
| 04_4 | An average reaches its largest part only if every part equals it. So the peak spreads along wires to the top, which is zero. | The card changes: with positive weights an average is at most its largest part, and equals it only if every part does; so a peak above 0 at a middle junction passes to every far end, and on along wires; here, to the top, where it is 0; likewise for the lowest value. On "every part equals it" the four values become M. On "spreads along wires" one M per far end travels out (the pair's second goes as the first sets out) and lands clear of its dot: right of the top and of the bottom, opposite their 0; at the right junction its "?" goes first and the M becomes its value, never two M at one place. The path's wire to the top (`R-F9.path.left`, one wire) flashes. On "which is zero" each end's 0 flashes with the M beside it | values travelling along wires, highlight, lower-third | |
| 04_5 | So the peak is zero, and likewise the lowest value. Both solutions are the same. | M becomes 0 at both middle junctions: the difference is 0 everywhere (`R-F9.peak`, `R-F9.low`). On "likewise the lowest" both 0s flash. On "Both solutions are the same" the copy is the second solution again, named "the second solution", its labels the circuit's own 1, 3/5, 2/5 and 0 and its junctions their colours; both copies' 3/5 and 2/5 flash | fraction, highlight | |
| 04_6 | Now add an island, two junctions wired only to each other. Is there still only one solution? ⟨predict⟩ | The right copy goes. On "two junctions" an island beside the circuit: two junctions joined by one wire and to nothing else, a "?" at each, named "an island". The card goes as the question ends; the countdown sits under the island | prediction pause, think time | |
| 04_7 | No. The island's peak crosses its one wire and stops, never reaching the top or bottom. Any equal pair keeps the rule. | The "?" by the countdown goes. The island's first junction takes M; on "crosses its one wire" an M crosses above the wire, clear of both dots, and hands over to the second junction's M (one after the other). On "never reaching" a dim note under the island: "no wire to the top or the bottom". On "Any equal pair" the island's two values run through 0 and 0, 1/2 and 1/2, 1 and 1, the junctions recolouring, each time with the rule at its first junction under them, its one far end over 1 equal to its value; the circuit's 3/5 and 2/5 flash, unchanged | values, equation, highlight | |
| 04_8 | So the proof needs every middle junction to reach the top or the bottom along wires. | A "precisely" lower-third: with positive conductances, if every middle junction has a path of wires to the top or the bottom, the rule with those two held has at most one solution (what the maximum and minimum argument proves; the circuit's own voltages are one); the island has no such path, so any equal pair keeps its rule (built only when the model finds the island cut off and every equal pair keeping its rule). The circuit's wires flash, then the island's | lower-third, highlight | |
| 04_9 | A walker on the island never reaches the top. Its chance is zero, but the island's voltage is left free. | A green walker steps back and forth across the island's wire. On "Its chance is zero" the note becomes "chance of reaching the top first: 0": no path of wires joins the island to either end (`R-X8.island.path`), so the walker is never there after any number of steps; the 12 steps followed agree (`R-X8.island.at_ends`). On "voltage is left free" the island's values become c and c, with a second note: "voltage: any equal pair c, c" | motion, text | |

### Notes for 04

- The argument is Doyle and Snell's uniqueness principle (§1.1.5): two
  solutions with the same top and bottom differ by a solution that is 0 at
  both; with positive weights its largest value, if above 0, sits at a
  middle junction whose far ends are no larger, so all of them equal it,
  and the peak spreads along a path of wires through middle junctions to
  the top or the bottom, where it is 0. The same for the lowest value. On
  this network the path is one wire from either middle junction
  (`R-F9.path.left`, `R-F9.path.right`), and the only solution with both
  ends at 0 is 0 (`R-F9.peak`, `R-F9.low`, `R-X1.zero_ends`).
- No second solution is drawn as if it existed: the copy is labelled "a
  second solution?" and "?" until the argument shows it is the first.
- The hypothesis is explicit (04_8): every middle junction must reach the
  top or the bottom along wires, with positive conductances. Its card says
  "at most one solution", which is what the maximum and minimum argument
  proves; the circuit's own voltages are one, so here there is exactly one.
  Card B's route is this example's ("here, to the top"). The island
  (`model::with_island`, cut off by `Network::cut_off`) breaks it: its two
  junctions may hold any equal pair, shown 0, 1/2 and 1 (`R-X8.island.keeps`),
  while an unequal pair fails (`R-X8.island.unequal`) and the circuit's
  values stay 3/5 and 2/5. Across the 729 networks of the family a second
  solution with both ends at 0 exists exactly where a junction is cut off
  (`R-X8.family.free`, 57) and 0 is the only one elsewhere
  (`R-X8.family.only_zero`, 672).
- Not unique is not the same as a chance of 0: no path of wires joins the
  island to either end (`R-X8.island.path`), and a walker moves only along
  wires, so it is never at either end after any number of steps; that is
  the proof for all time, and the 12 steps followed (`R-X8.island.at_ends`)
  are a witness that agrees. So its chance of reaching the top first is 0,
  one definite number; the rule leaves the island's voltage free. Its chance of
  reaching the bottom first is 0 too, so it never stops.
- Prediction (04_6): the tempting answer is yes, since the argument was
  just made; the answer is no, because the island's peak never meets the
  top or the bottom. Nothing on screen shows an island value before the
  countdown ends.
- Card dwell, as chapter 03: every lower-third stays up at least its
  reading time at fifteen characters a second from when it is fully in:
  04_2's over 04_3, 04_4's through the 04_6 question, 04_8's to the end. A
  test checks each fits inside the lines it stays over by the scratch voice.
- Pauses for the narration (no audio exists): the one prediction hold after
  04_6; a reveal pause after "No."; completed-thought pauses before 04_6
  (the island) and before 04_9 (the walker). Directions only; no synthesis
  under the current owner amendment.
