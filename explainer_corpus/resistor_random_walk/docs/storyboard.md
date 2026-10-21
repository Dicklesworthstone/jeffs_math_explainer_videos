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

The long form (owner direction 2026-10-20: complete videos of 10 to 15
minutes). It starts where Doyle and Snell start, a walker on a line of
identical wires and the same line as a circuit, where both answers come from
equal steps; then the one tiny network the rest of the video keeps: four
junctions, identical wires, a pair side by side from the top to the left and
from the right to the bottom (a Wheatstone bridge, which neither school
shortcut can reduce). The top is held at one volt and the bottom at zero;
the left junction sits at 3/5 and the right at 2/5, and a walker from the
left reaches the top first with chance 3/5. Chapter 04 previews the payoff;
05 to 07 derive both junctions' balances in both pictures, solve them and
check the walker by its routes; 08 finds the values by repeated averaging,
09 proves there is only one solution and where that fails; 10 and 11 carry
the rule to unequal wires and to a program that knows when to stop.

| Chapter | Spine | Status |
|---|---|---|
| `01_which_end` | a walker on a line of five identical wires, one wire above the bottom; one route; a chance is the fraction of very many walkers; its routes never run out, so one step ahead, half the time up and half down, and its chance is the average of the two far ends', halfway between them; prediction; equal steps, so 1/5; its chance of walking for ever keeps the same rule with zero at both ends, so it is zero | new for the long form (`mx-video-program-program-87do.18`, root's implementation grant of 2026-10-20): source only, nothing compiled or rendered in this lane |
| `02_the_line_as_a_circuit` | the same line with a battery: current in equals current out, so every wire carries one current and identical wires drop equal voltages; prediction; 1/5 of a volt, the walker's chance, junction by junction; equal drops are the halfway rule, and on a line it allows only equal steps, so the two answers had to agree | new, as 01 |
| `03_a_bridge` | the bridge network and its averaging rule (production 01_1 to 01_3, the values previewed); the two school shortcuts; prediction: which two wires merge next; none, so the voltages must come from the rule at both junctions at once | 03_1 and 03_2 are production 01_1 and 01_2 (source at 38e52c0, the frozen repairs of f136332 and 7a43336); 03_3's words replace 01_3's; 03_4 to 03_8 new |
| `04_the_same_number` | the walker on the bridge, one step ahead, prediction, three fifths again, only one solution, the model's scope (production 01_4 to 01_8, the circuit rebuilt as chapter 03 left it) | words unchanged from production; the uniqueness card shortened (cut X1); the opening state rebuilt for the split |
| `05_why_an_average` | the left junction's drops with the previewed numbers (production 02_1 to 02_3), then both voltages unknown: the left's balance, 4L = 1 + 1 + R + 0, and the right's (prediction), 4R = 1 + L + 0 + 0 | 05_1 to 05_3 production words (their Ohm card dropped, their Kirchhoff card cut to its last sentence); 05_4 replaces 02_4; 05_5 to 05_7 new |
| `06_the_walkers_rule` | the walker's first step, one half not one third (production 02_5, 02_6), the left's first-step equation in letters (02_7's words), the right's, and times four the circuit's two balances, letter for letter | 06_1 to 06_3 production words (06_3's rows now in letters, its fresh-start card dropped); 06_4 to 06_6 new |
| `07_one_answer_two_ways` | solve the pair once: 16R = 4 + 2 + R, 15R = 6, so 2/5, then (2 + 2/5)/4 = 3/5; fifteen is not zero, so no other pair keeps both; the walker by its routes, 9/16 plus 1/16 of itself, 3/5 again, and why that worked; prediction: the bottom first, 2/5 | new |
| `08_round_after_round` | repeated averaging, every junction at once from the round before; the real rounds; the gap a quarter a round; prediction: never exactly; within a thousandth after five rounds; the ends must stay fixed | production chapter 03 (39eed75 and the frozen repairs), its line ids renumbered 03 → 08 only |
| `09_only_one` | the maximum principle on the difference of two solutions; prediction: an island; the hypothesis; the island's walker has chance 0 while its voltage is free | production chapter 04 (1cb1cbe and the frozen repairs), its line ids renumbered 04 → 09 only |
| `10_any_wires` | the weighted rule for unequal wires (production 02_8), then Doyle and Snell's unequal example on the same shape: prediction, across with chance 1/2; the two equations; 7/16 and 3/8, as voltages and as chances | 10_1 is production 02_8, moved; 10_2 to 10_5 new |
| `11_when_to_stop` | for programmers: the weighted averaging; two runs from all zeros and all ones, the top and the bottom held in both, bracket the answer; within a thousandth of each other after nine rounds; prediction: beside an island; they never move there; on any finite network whose middle junctions reach an end, the voltage is the chance | new |

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
in the voice); **pair** (two wires side by side); **route** (the walk a
walker takes, from chapter 01 on); **step** (one wire walked, or one of the
equal differences up a line); **balance** (current in equals current out at
one junction, as an equation); **bridge** (the wire from the left junction to
the right); **in a row** and **side by side** (the two school shortcuts);
**detour** (a route that comes back to where it started); **difference**
(chapter 09: two solutions subtracted, junction by junction); **peak** (a
difference's largest value); **island** (junctions wired only to each other,
with no path of wires to the top or the bottom); **conductance** (from
chapter 03's cards and chapter 10 in the voice: how easily a wire carries
current, one over its resistance); **run** (chapter 11: one sequence of
rounds from a starting guess).

## Color roles

| Idea | Color |
|---|---|
| the top: one volt, a chance of one | gold `#F0AC5F` |
| the bottom: zero volts, a chance of zero | blue `#58C4DD` |
| a middle junction once its value is known, and a value travelling along a wire | between blue and gold in proportion to the value (`palette::value`) |
| a middle junction whose value is not yet shown | `faint` |
| the walker | green `#83C167` |
| wires and the battery | `dim` |
| the unknown in a prediction | `hero` `#FFD866`, the "?" only |
| a current: a wire's drop and the arrow of its flow (chapters 02 and 05) | teal `#5CD0B3`, text and strokes only |
| a wire not under discussion | `faint` |
| the pictures' names, units and notes | `dim`; values in `ink` |

## 01_which_end: Which end first? (~81 s)

The line stands on the left: six junctions one above another, the bottom
(junction 0, blue) at the foot and the top (junction 5, gold) at the head,
the four middle junctions faint, five identical dim wires between them.
Values are written left of the junctions; the bars and rows on the right.
Every number is `R-F10`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Five identical wires join six junctions in a line. A walker starts one wire above the bottom. | The six junctions appear from the bottom up, then the five wires; the name "chance of reaching the top first" above the line. On "A walker" the green walker appears on junction 1 | junctions and wires built from the line in facts | |
| 01_2 | At each middle junction it picks one of its two wires at random, and it stops if it reaches the top or the bottom. | On "one of its two wires" junction 1's two wires flash; on "it stops" the top and the bottom flash | highlight | |
| 01_3 | Here's one route. Up, down, up, up, up and up. | On "Up," the walker takes the drawn route 1, 2, 1, 2, 3, 4, 5, a step at a time, and the top flashes when it arrives. Then, without words, a faint ghost walker on junction 1 steps straight down to the bottom, which flashes, and the ghost fades: its first step could just as well have ended it there | motion, highlight | |
| 01_4 | So what's its chance of reaching the top first? It's the fraction of very many walkers that do. | The walker is back on junction 1 with a gold "?" beside it. On "very many walkers" a row of faint green dots stands beside junction 1, no count and no outcomes; a "precisely" lower-third (card A): walkers step independently, and as their number grows the fraction that reach the top first tends to this chance (the law of large numbers) | lower-third | |
| 01_5 | Its routes never run out, so look one step ahead. Half the time it lands one wire up and half the time one wire down, and from there it starts afresh. | The dots go. On "look one step ahead" a "?" at junctions 0 and 2, and "1/2" in the walker's green beside each of junction 1's wires. On "starts afresh" junction 2 flashes. Card A goes and card B comes: its next steps do not depend on how it got here, so its chance from where it lands is that junction's own chance | values beside wires, lower-third | |
| 01_6 | So its chance is the average of the chances at those two far ends. And an average of two numbers sits halfway between them. | The two "?" and the halves go; on the right the row p(1) = (p(0) + p(2))/2; on "halfway" a dim note under it: "halfway between its neighbours" | equation | |
| 01_7 | From the top the chance is one, and from the bottom it's zero. So what is it one wire above the bottom? ⟨predict⟩ | Card B goes. The top reads 1, the bottom 0, junctions 2 to 4 a faint "?"; the question is on screen by its last word, then the countdown on the right. Nothing shows a middle value | prediction pause, think time | |
| 01_8 | One fifth. Halfway at every middle junction means equal steps, and five equal steps climb from zero to one. | The countdown's "?" goes; junctions 1 to 4 read 1/5, 2/5, 3/5, 4/5, each turning its colour; on "equal steps" bars grow beside the junctions to lengths in proportion to the chances, a straight ramp | values, bars | |
| 01_9 | And its chance of walking for ever is zero. That chance keeps the same halfway rule, with zero at both ends, and equal steps from zero to zero stay at zero. | The name becomes "chance of walking for ever"; the row goes; the top's 1 becomes 0; on "equal steps from zero" the bars shrink away and every middle value becomes 0, the junctions blue. Card C: endless walks exist, but together they have chance 0, so it stops with chance 1; on a line of N wires, x wires up, the chance is x/N | values, lower-third | |

### Notes for 01

- The line is Doyle and Snell's five-block walk (§1.1.1, printed p. 3) and
  its averaging property (§1.1.3, pp. 4–5); the chance of never reaching an
  end is the same halfway rule with zero at both ends (§1.1.5, from p. 6),
  so it is 0. Card C says endless routes exist; only their chance is 0. No
  sentence says every walk ends.
- Prediction (01_7): the tempting answers are 1/2 (two ends) or "very
  small". The answer needs the halfway rule of 01_6 and the fixed ends;
  nothing on screen gives a middle value before the countdown ends.
- The ghost's step down (01_3) is drawn without words (cut X2): `R-F10.ghost`.
- No simulated walkers are counted: the dots of 01_4 carry no number.
- Pauses for the narration (no audio exists for this version): the one
  prediction hold after 01_7; a reveal pause after "One fifth."; a
  completed-thought pause before 01_4 (the route ends) and before 01_9.

## 02_the_line_as_a_circuit: The same line, with a battery (~68 s)

The line stands on the left as chapter 01 drew it, its middle junctions
faint again; later the walk's line beside it. Every number is `R-F11`
(and `R-F10` for the walk's line).

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Now forget the walker. A battery holds the top of the same line at one volt and the bottom at zero. | The line appears with its name "voltage"; on "A battery" its wire runs from the top round to the right, through its two plates, and back to the bottom, with "1 volt" by the top and "0 volts" by the bottom | junctions and wires, battery | |
| 02_2 | Current flows down through the wires. Nothing piles up at a middle junction, so the current in equals the current out. | A teal arrowhead on every wire points down; on "the current in equals" junction 3's arrows above and below flash | arrows, highlight | |
| 02_3 | So every wire carries the same current. And a wire's current is its voltage drop divided by its resistance, so identical wires have equal drops. | All five arrows flash together; on "equal drops" all five wires flash. Card D: ideal wires, steady currents, an ideal battery; Ohm's law, current = drop ÷ resistance; Kirchhoff's current law, the currents into a middle junction add to zero | highlight, lower-third | |
| 02_4 | Five equal drops share the one volt. So what's the voltage one wire above the bottom? ⟨predict⟩ | Junction 1 carries a gold "?". Card D stays while the question is asked and goes as it ends; then the countdown. No drop is shown with a number | prediction pause, think time | |
| 02_5 | One fifth of a volt, the walker's chance from there. Junction by junction, the voltages and the chances agree. | "1/5" in teal beside every wire; the junctions read 1/5, 2/5, 3/5, 4/5 and turn their colours. On "Junction by junction" the walk's line from chapter 01 stands beside it, named "chance of reaching the top first", its chances 1/5 to 4/5, and each matching pair flashes | values beside wires, a second copy of the line | |
| 02_6 | Is that luck? At a middle junction, the current in from above equals the current out below, so the drop above equals the drop below. | Junction 3's drops above and below flash together | highlight | |
| 02_7 | Equal drops put its voltage halfway between the voltages at the far ends of its two wires. That's the walker's rule. | On the right the row 3/5 = (2/5 + 4/5)/2 for junction 3; on "the walker's rule" the walk's junction 3 and the circuit's flash together | equation, highlight | |
| 02_8 | The same rule, the same top and bottom, and on a line the rule allows only equal steps. So the two answers had to agree. | Both lines' tops and bottoms flash, then all their middle values | highlight | |

### Notes for 02

- Doyle and Snell §1.1.4 (printed pp. 4–5): equal resistors in series,
  Kirchhoff's laws and Ohm's law give v(x) = (v(x − 1) + v(x + 1))/2. The
  only physics claimed is Ohm's law and Kirchhoff's current law for ideal
  wires, steady currents and an ideal battery (card D). "Current flows down"
  is conventional current; nothing is said about charge carriers, and the
  walker's scope note comes in chapter 04.
- Prediction (02_4): the new reasoning is equal currents, so equal drops.
  The walker's 1/5 comes from a different picture; seeing it agree is the
  payoff.
- 02_8 is uniqueness on a line: the rule with fixed ends forces equal steps
  (a fifth each), so one solution; chapter 07 gives the bridge's, chapter 09
  the general argument.

## 03_a_bridge: A network with a bridge (~76 s)

The circuit stands alone in the middle (as production chapter 01 opened).
Every number is `R-F1`, `R-F2`, `R-F6` or `R-F12`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Identical wires join four junctions, doubled in two places. A battery holds the top at one volt and the bottom at zero. | The four junctions appear (top gold, bottom blue, the two middle ones faint), then the wires: a pair side by side from the top to the left and from the right to the bottom, one wire elsewhere. On "doubled" the two pairs flash. On "A battery" the battery's wire runs from the top round to the bottom on the right, with "1 volt" by the top and "0 volts" by the bottom | junctions and wires built from the network in facts, highlight | |
| 03_2 | Current in equals current out, so a middle junction's voltage is the average of the voltages at the far ends of its four wires. | The left junction's four wires flash, then the right's. A "precisely" lower-third (card E): wires side by side add their conductances, so the average counts the pair's far end twice | highlight, lower-third | |
| 03_3 | Its middle voltages will turn out to be three fifths and two fifths. On the left, one, one, two fifths and zero average to three fifths. | The card goes. The right junction takes its value, 2/5. Four values leave the far ends of the left junction's wires (1, 1 from the doubled pair, 2/5, 0), each in its color, and slide along the wires into it; beneath the circuit the equation (1 + 1 + 2/5 + 0)/4 = 3/5 writes itself and the left junction turns its color, labelled 3/5. Then, quickly and without words, the right junction's check: 1, 3/5, 0, 0 into it, the equation swapping to (1 + 3/5 + 0 + 0)/4 = 2/5, and a three-second beat | values travelling along wires, equation, fraction | |
| 03_4 | Where do those come from? Not from the school shortcuts for wires. | The equation goes. A dim heading on the right: "two school shortcuts" | text | |
| 03_5 | Two wires side by side act as one that carries current twice as easily. Two in a row, with nothing else at the junction between them, act as one twice as hard to push through. | Under the heading, two small drawings: a pair of wires side by side becomes one heavy wire, "conductance × 2"; then two wires end to end, a small junction between them, become one long wire, "resistance × 2". Card F: side by side, conductances add; in a row, with nothing else at the junction between, resistances add (Doyle and Snell, section 1.3.4) | highlight, lower-third | |
| 03_6 | Merge each pair, and five wires remain. Which two can you merge next? ⟨predict⟩ | On "Merge each pair" each pair on the circuit becomes one heavy wire: five links. Card F stays while the question is asked and goes as it ends; the countdown on the right, under the heading. Nothing marks a candidate | prediction pause, think time | |
| 03_7 | None. No two join the same two junctions, and three wires meet at each middle junction, so no two are in a row. | The five links flash one after another; on "three wires meet" the left junction's three links flash with a "3" beside it, then the right's | highlight | |
| 03_8 | The wire from the left to the right, a bridge across the middle, blocks both. The voltages must come from the rule at both junctions at once. | The left–right wire flashes, named "bridge"; on "The voltages must come" the heavy wires split back into their pairs and the drawings and heading go | highlight | |

### Notes for 03

- 03_1 and 03_2 are production 01_1 and 01_2 (protected phrases "doubled in
  two places" and "the average of the voltages at the far ends of its four
  wires"); 03_3 keeps "one, one, two fifths and zero average to three
  fifths" and announces the values as a preview (chapter 07 solves them).
  03_3's picture and its frozen timing (the 0.125 + 0.125 s rule swap, the
  three-second beat) are production 01_3's.
- Card E replaces production 01_2's longer Ohm card: chapter 02's card D
  already gives Ohm's law and Kirchhoff's current law.
- The shortcut claim is narrow: with the pairs merged, no two of the five
  links join the same two junctions (`R-F12.side_by_side` 0) and three
  links meet at each middle junction (`R-F12.meeting`), so no two are in a
  row (`R-F12.in_a_row` 0). Other tools (the star–triangle swap) are not
  claimed unavailable, and the bridge is not balanced (`R-F4`: a drop of 1/5
  across it), so it cannot simply be removed. "School shortcuts" is the
  video's framing, not a historical claim.
- Prediction (03_6): the tempting answer pairs the top–left and left–bottom
  wires "in a row"; the left also meets the bridge, so they are not.

## 04_the_same_number: The same number, twice (~62 s)

The circuit stands where chapter 03 left it: alone, the battery holding
1 volt and 0 volts, the left at 3/5 and the right at 2/5. Every number is
`R-F1` to `R-F3` or `R-X1`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Now forget the battery. A walker at the left junction picks one of its four wires at random, and walks on until it reaches the top or the bottom. | The circuit slides left under its name, "voltage". On the right, the same junctions and wires again with no battery, under "chance of reaching the top first"; its middle junctions faint. A green walker appears at the left junction; on "at random" its four wires flash; on "walks on" it takes the drawn route, over to the right, back, over again and up to the top, which flashes | motion, highlight, a second copy of the network | |
| 04_2 | Its routes never run out, so look one step ahead. Its chance of reaching the top first is the average of its chances from the far ends of its four wires. | The walker is back at the left junction. A "precisely" lower-third: the chances it is still walking after one, two and three steps (1/4, 1/16, 1/64, from facts), never zero but shrinking towards it, so it stops with chance one. On "look one step ahead" its four wires flash, and a "?" waits at each far end | highlight, lower-third | |
| 04_3 | From the top the chance is one, and from the bottom it's zero. So what is it from the left junction? ⟨predict⟩ | The card goes. The far ends' "?" at the top become 1, the one at the bottom 0; the right junction's stays "?". The left junction carries a large gold "?": the question is on screen by its last word, then the countdown | prediction pause, think time | |
| 04_4 | Three fifths again. One rule, the same top and bottom, and only one solution. So the voltage and the chance must agree. | The "?" become 2/5 at the right junction and 3/5 at the left; the same four values slide into the left junction and the same equation forms beneath the walk. On "One rule" both pictures' equations stand side by side and both 3/5 labels flash. On "only one solution" a "precisely" lower-third, its hypothesis first: here every middle junction is wired to the top or bottom, which is what makes the solution unique (proved later in the video) (the card is built only if the model finds no junction cut off) | equation, fraction, highlight, lower-third | |
| 04_5 | The walker is a model, not a claim about electrons. The two numbers agree because they solve the same equations. | The card goes. Under the walk two dim notes: the video's own scope statement, "a model of the equations, not of electrons", and below it the theorem's source, "voltage equals chance: Doyle and Snell, section 1.3.2"; the walker fades; the two pictures stand with their equal numbers, then clear | text | |

### Notes for 04

- Words, pictures and the frozen repairs are production 01_4 to 01_8's; the
  scene first rebuilds the circuit as chapter 03 leaves it (alone, its
  battery, its labels and colours), because the two are now separate
  chapters.
- Cut X1: the uniqueness card states only the hypothesis and that it is
  what makes the solution unique; the argument itself is chapter 09's
  (Doyle and Snell §1.1.5). It still fades in only after the equations fade
  out, and the notes after it.
- Prediction (04_3): will the same rule on the same wires give the same
  number? The circuit's 3/5 stays on screen; the walk's left value stays
  hidden through the countdown.

## 05_why_an_average: Why each junction is an average (~66 s)

The circuit stands on the left (`PICTURE_AT`), its values the preview's; the
derivation builds on the right, row by row, every number `R-F4` or `R-F13`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | At the left junction, each wire's current is its voltage drop divided by the wires' shared resistance. | The circuit (1 volt, 0 volts, 3/5, 2/5, the battery, "voltage" over it). The three wires away from the left junction fade to faint; the left junction's four wires flash | highlight | |
| 05_2 | Each wire of the pair drops two fifths of a volt. Then one fifth to the right, three fifths to the bottom. | Beside each wire of the pair, "2/5" in the current's teal, and on the wire a teal arrowhead pointing into the junction; then "1/5" and "3/5" beside the wires to the right and to the bottom, their arrowheads pointing out | values beside wires, arrows | |
| 05_3 | Current in equals current out, so the drops balance too. Two fifths and two fifths in, one fifth and three fifths out. | On the right, the first rows of the derivation, labelled as drops in volts, not currents: "drops in: 2/5 + 2/5 = 4/5 V", then "drops out: 1/5 + 3/5 = 4/5 V". A "precisely" lower-third (card G): each current is its drop over the same resistance, so the drops balance as the currents do | equation, lower-third | |
| 05_4 | Now hide both middle voltages and call them left and right. The balance at the left says four times the left equals one plus one plus the right plus zero. | The card goes. The left's 3/5 becomes L and the right's 2/5 becomes R. Rows: (1 − L) + (1 − L) + (R − L) + (0 − L) = 0; on "four times the left" 4L = 1 + 1 + R + 0 | equation, derivation | |
| 05_5 | So the left is the average of its far ends, whatever the right turns out to be. The pair is two wires, so the top's one counts twice. | Row L = (1 + 1 + R + 0)/4; on "The pair is two wires" the pair flashes. The drops, arrows and working rows go; L's average rises to the top row and the right junction's wires turn back from faint | formula, highlight | |
| 05_6 | Now the right junction. One wire goes up, one to the left, and two go down. What does four times the right equal? ⟨predict⟩ | The wires away from the right junction fade to faint; on "One wire goes up" its four wires flash. No far-end value is shown; the countdown in the panel | prediction pause, think time | |
| 05_7 | One plus the left plus zero plus zero. So the right is the average of its far ends too. | Row 4R = 1 + L + 0 + 0; on "So the right is the average" it becomes R = (1 + L + 0 + 0)/4 in the second row, and the faint wires return. The two averages stand: the circuit's two balances | equation, formula | |

### Notes for 05

- 05_1 to 05_3 are production 02_1 to 02_3 (numbers before letters: the
  previewed values). Their Ohm card is dropped (card D gave Ohm's law) and
  the Kirchhoff card cut to its last sentence (card G); `ROW_Y`'s spacing is
  kept.
- 05_4 replaces production 02_4, which put the right's 2/5 into the left's
  balance. Now both are unknown, and the right's balance is derived too,
  so chapter 07's solve is the derivation's own.
- Signs: each drop is the far end's value minus the junction's; the rows add
  the four drops to zero, so the unknown gathers four times.
- Prediction (05_6): the viewer applies the rule just shown to the junction
  whose pair is on the other side; the tempting error counts the bottom pair
  once.

## 06_the_walkers_rule: The walker's rule, at both junctions (~67 s)

The same network on the left as the walk, "chance of reaching the top
first", the top 1, the bottom 0, both middle junctions "?". Every number is
`R-F5`, `R-X4` or `R-F13`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now the walker picks one of the four wires, each equally likely. What's the chance its first step reaches the top? ⟨predict⟩ | The walker appears at the left junction and its four wires flash. Nothing shows a step's chance: the countdown runs in the panel | prediction pause, think time | |
| 06_2 | One half, not one third. The pair is two of the four wires, so it counts twice, like its two currents. | "1/4" beside each of the four wires in the walker's green; the pair's two quarters slide together into "1/2" by the pair. A row: "first step up: 2/4 = 1/2". A "precisely" lower-third: counting the pair as one wire makes another network, where the first step goes up with chance 1/3 and the left junction sits at 5/11, as a voltage and as a chance | values beside wires, merge, equation, lower-third | |
| 06_3 | Up, it has reached the top. Right, its chance is the right junction's. Down, it's zero. A quarter of each, added, is the same average. | The card goes. The far ends' chances replace the quarters: 1, 1, the right's chance (the letter R), 0. On "Right, its chance" the right junction's "?" flashes. Then the row p(L) = ¼·1 + ¼·1 + ¼·p(R) + ¼·0, in letters | equation, highlight | |
| 06_4 | From the right junction it steps up with chance a quarter, left with a quarter and down with a half. So its chance is a quarter of one, plus a quarter of the left's, plus a half of zero. | The left's values go. "1/4" beside each of the right junction's four wires, the down pair's two quarters sliding together into "1/2". Row p(R) = ¼·1 + ¼·p(L) + ½·0 | values beside wires, merge, equation | |
| 06_5 | Times four, the walker's equations say four times the left is two plus the right, and four times the right is one plus the left. | Rows 4p(L) = 2 + p(R) and 4p(R) = 1 + p(L) | equation | |
| 06_6 | Those are the circuit's two balances, letter for letter. Whatever solves one pair solves the other. | Beside them the circuit's 4L = 2 + R and 4R = 1 + L, in ink; matching rows flash in pairs | equation, highlight | |

### Notes for 06

- 06_1 to 06_3 are production 02_5 to 02_7; 06_3's rows are now in letters
  (the right's chance is not taken from chapter 04's preview), and its
  fresh-start card is dropped (card B in chapter 01 said it). The pair card
  (`R-X4`: the pair counted once gives 1/3 and 5/11) is kept with its
  frozen read-then-fade.
- The first-step equation is Doyle and Snell §1.1.3's "Basic Fact" on this
  network (§1.3.2, "hx is also harmonic"); the walker's chance after one
  step is that junction's own (card B, the Markov property).
- Prediction (06_1): the tempting answer is 1/3 (three junctions); the
  walker picks among wires, and the pair is two of them (`R-F5`).

## 07_one_answer_two_ways: One answer, two ways (~78 s)

The network on the left, named "voltage or chance", the top 1, the bottom 0,
both middle junctions "?"; the rows on the right. Every number is `R-F13`,
`R-X1` or `R-F14`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | So solve them once. The first says the left is two plus the right, over four. Put that into the second and multiply by four. | Rows 4L = 2 + R and 4R = 1 + L; on "the left is two plus the right" L = (2 + R)/4; on "Put that into the second" 4R = 1 + (2 + R)/4 | equation, derivation | |
| 07_2 | Sixteen times the right equals four plus two plus the right. So fifteen times the right is six, and the right is two fifths. | Rows 16R = 4 + 2 + R, then 15R = 6, then R = 6/15 = 2/5; the right junction's "?" becomes 2/5 and it turns its colour | derivation, fraction | |
| 07_3 | Then the left is two and two fifths, over four, which is three fifths. And because fifteen isn't zero, no other pair of values keeps both balances. | The working rows go; row L = (2 + 2/5)/4 = (12/5)/4 = 3/5; the left's "?" becomes 3/5. A "precisely" lower-third (card H, from the facts): 4·left − right = 2 and −left + 4·right = 1 have determinant 4·4 − 1·1 = 15, not 0, so exactly one solution | derivation, lower-third | |
| 07_4 | Now check the walker by its routes alone. From the left, it reaches the top without coming back with chance one half plus one sixteenth. | The rows go but the two values stay. The walker at the left; "1/2" by the pair to the top, "1/16" by the wire from the right to the top; row 1/2 + 1/16 = 9/16 | values beside wires, equation | |
| 07_5 | Straight up the pair, or across and then up. It comes back to the left with chance one sixteenth, and then it starts the same walk again. | On "Straight up the pair" the pair flashes; on "across and then up" the bridge and the right's wire to the top flash. On "comes back" the walker crosses the bridge and back, "1/16" by the bridge | motion, highlight | |
| 07_6 | So its chance is nine sixteenths plus a sixteenth of itself. Fifteen sixteenths of it is nine sixteenths, which makes it three fifths. | Rows S = 9/16 + (1/16)S, then (15/16)S = 9/16, then S = 9/15 = 3/5, flashing with the left's 3/5 | equation, highlight | |
| 07_7 | That worked so neatly because every detour here is the same trip across and back. Bigger networks have detours of every shape. | The bridge flashes once more | highlight | |
| 07_8 | One more. From the left, what's the walker's chance of reaching the bottom first? ⟨predict⟩ | The rows and labels by the wires go; the name becomes "chance of reaching the bottom first", the top 0, the bottom 1, the left a gold "?" and the countdown | prediction pause, think time | |
| 07_9 | Two fifths. It reaches one end or the other with chance one, so the two chances add to one. | The left reads 2/5 and the right 3/5; row 3/5 + 2/5 = 1 | fraction, equation | |

### Notes for 07

- The solve is the narration's own (`R-F13`): the left is (2 + right)/4, put
  into 4·right = 1 + left and multiplied by 4, 16·right = 4 + 2 + right,
  15·right = 6. Fifteen is the balance's determinant (`R-X1.determinant`):
  not zero, so exactly one solution (card H).
- The route check (`R-F14`): from the left, before it stops or first comes
  back, up now 1/2, across then up 1/16, across and back 1/16, down now 1/4,
  across then down 1/8, adding to 1; each return starts the same walk
  afresh (card B), so S = 9/16 + S/16. 07_7's claim is only that here every
  detour is the one trip across and back.
- Prediction (07_8): it reaches one end or the other with chance one (the
  chance of still walking after n steps is 1/4ⁿ, `R-F3.still_walking`), so
  the chances add to one; the words say chance, not every route.

## 08_round_after_round: Average, and average again (~83 s)

The circuit stands on the left, as chapter 07 had it; on the right a
table of rounds: round, left, right, and later the bigger gap. Every number
is `R-F7`, `R-F8` or `R-X6`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Here's a way to find the middle voltages without solving anything. Guess zero for both, then average. | The circuit (battery, 1 volt, 0 volts), its middle junctions labelled 0 and coloured as 0. On the right the table's header, round, left, right, and its first row: 0, 0, 0. On "Guess zero" the two zeros flash | table, highlight | |
| 08_2 | Each round, every middle junction takes the average of the far ends of its four wires, using the values from the round before. | Both middle junctions' wires flash. On "using the values" a "precisely" lower-third: every middle junction updates at once from the previous round (Jacobi's method); one at a time with the newest values (Gauss-Seidel), the gap shrinks to 1/16 a round from round 2 on. It stays over round one, which shows it | highlight, lower-third | |
| 08_3 | Round one. The left averages one, one, zero and zero to one half. The right averages one and three zeros to one quarter. | Both junctions' far-end values from round 0 appear at once beside their wires (left 1, 1, 0, 0; right 1, 0, 0, 0). The left's slide in and it becomes 1/2; then the right's slide in, its token from the left still reading the old 0, and it becomes 1/4. The table's row 1: 1/2, 1/4. Each junction's colour moves with its value | values travelling along wires, table | |
| 08_4 | Round two uses those. Nine sixteenths and three eighths. Then nineteen thirty-seconds and twenty-five sixty-fourths. | The card goes as round 2's far-end values (left 1, 1, 1/4, 0; right 1, 1/2, 0, 0) appear; they slide into both junctions together; 9/16 and 3/8, row 2. On "Then nineteen" round 3 lands without tokens (19/32, 25/64, row 3); those large values stay through the whole cue. Round 4 waits for 08_5 | values travelling along wires, table | |
| 08_5 | The bigger gap to the answer shrinks to a quarter every round, because only one wire at each junction leads to an unknown. | Round 4 first lands without words (153/256, 51/128, row 4), after 08_4 has finished saying round 3’s values. The table’s fourth column, "bigger gap", then fills down rounds 0 to 4 over 0.6 seconds: 3/5, 3/20, 3/80, 3/320, 3/1280, and a "precisely" lower-third comes in under it: 3 of each junction's 4 far ends never change, so its new gap is 1/4 of the other's old one: the bigger gap is exactly 1/4 of the last, 3/5, 3/20, 3/80, 3/320 and 3/1280 by round 4 (the card stops at the rows on screen, so round 5 waits for the prediction). On "only one wire" the one wire joining the two middle junctions flashes | table, highlight, lower-third | |
| 08_6 | So after how many rounds is the left exactly three fifths? ⟨predict⟩ | The card stays while the question is asked and goes as it ends. Through the countdown the table holds rounds 0 to 4 and nothing else; nothing shows round 5 or a limit. The countdown sits in the panel under the table | prediction pause, think time | |
| 08_7 | Never. Each round makes halves, quarters, eighths and so on, and three fifths is none of them. But after five rounds both are within a thousandth. | A "precisely" lower-third: each new value is the other junction's previous value plus a whole number, divided by 4, so from the guess of 0 every value is over a power of 2, like 9/16 and 153/256 (rows already shown), and never 3/5; the gaps go to 0, so the values have a limit. It stays over 08_8. On "But after five rounds" row 5 lands, 307/512, 409/1024, gap 3/5120, and the junction labels follow | lower-third, table, highlight | |
| 08_8 | The values close in on three fifths and two fifths, the only pair that keeps every average. | The 08_7 card stays. Under the table, set apart, a "limit" row: 3/5, 2/5, gap 0. The labels on the junctions stay at round 5's values: the picture shows rounds, the limit row shows where they go | table, highlight | |
| 08_9 | The top and bottom must stay fixed. Average them too, and every junction drifts to the same number. | A counterfactual, shown as one. As the line begins the whole table (its rounds and its battery-held limit row 3/5, 2/5) and the 08_7 card go, and a "precisely" lower-third comes: with the top and bottom averaged too, 1, 3/5, 2/5, 0 drift to 1/2, their mean weighted by wires 3, 4, 4, 3. On "Average them too" the battery, its two labels and the middle labels fade; all four junctions are averaged together from the solved voltages, round by round (6 rounds, to within 1/1000, `R-X6.within`), and their colours run together to one. No number on screen then belongs to the battery-held network | colour drift, lower-third | |

### Notes for 08

- The chapter chooses previous-round updates (Jacobi) and keeps them: every
  round's two new values are computed from the round before, which is why
  round 1's token from the left still reads 0 while the left already shows
  1/2. One-at-a-time updates (Gauss-Seidel) appear only in the 08_2 card,
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
  keeps every average, and is the only pair that does, is chapter 04's
  claim, solved for this network in chapter 07 and proved in general in
  chapter 09.
- Edges: the ends averaged too (`R-X6`: the weighted mean 1/2 is kept every
  round and, on this network, every value settles there) shows why the ends
  stay fixed; 08_9 says it as a counterfactual. A kept mean alone does not
  make values settle: two junctions joined by one wire, both averaged, swap
  1 and 0 for ever (a model test and an oracle check, not shown). An island
  of two junctions wired only to each other swaps its values for ever too
  (`R-X7`, a fact and a test, not shown). On screen the drift is claimed
  for the network shown only.
- Prediction (08_6): the tempting answers are a round count read off the
  table; the answer is never. Nothing shows round 5 or the limit before
  the countdown ends.
- Card dwell: every lower-third stays up at least its reading time at
  fifteen characters a second, counted from when it is fully in (the
  scene's `wait_until`): 08_2's over round one (08_3), 08_5's from the gap
  column to the question's end, 08_7's over the limit (08_8), 08_9's from
  the line's start. By the scratch voice each fits inside the lines it
  stays over, so no time is added; a test checks it.
- Pauses for the narration (no audio exists): the one prediction hold after
  08_6; a reveal pause after "Never."; completed-thought pauses before 08_5
  (the gaps) and before 08_9 (the edge). Directions only; no synthesis under
  the current owner amendment.

## 09_only_one: Why only one solution (~83 s)

The circuit stands on the left as chapter 08 left it, the battery holding
1 volt and 0 volts, its middle junctions at 3/5 and 2/5. On the right, a
second copy of the network for the supposed second solution, which becomes
their difference; later, in its place, an island beside the circuit. Every
number is `R-X1`, `R-F9` or `R-X8`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Here's why there is only one solution. Suppose another set of values keeps the rule, with the same top and bottom. | The circuit with its values under "one solution". On "Suppose another" a second copy of the network on the right under "a second solution?", the same 1 at its top and 0 at its bottom, a "?" at each middle junction. On "the same top and bottom" both copies' tops and bottoms flash | a second copy of the network, highlight | |
| 09_2 | Subtract them. The difference is zero at the top and bottom, and it keeps the rule too. | The right copy becomes the difference, its name "their difference": its top's 1 becomes 0 and the top turns the bottom's colour; its bottom keeps 0; the "?" stay. On "The difference is zero" the two 0s flash and a "precisely" lower-third comes: at every middle junction each solution is the conductance-weighted average of the same far ends, so their difference is the average of the far ends' differences; at the top it is 1 − 1, at the bottom 0 − 0, both 0. On "keeps the rule" the copy's middle junctions' wires flash | highlight, lower-third | |
| 09_3 | Take its largest value. If above zero, it sits at a middle junction, an average of far ends no larger. | In the difference, the left junction's "?" becomes M, its largest value, and the junction turns the hero colour. On "If above zero" the top's and bottom's 0s flash: a value above them cannot sit there. On "an average of far ends" its four far ends' values appear beside its wires: 0 and 0 from the pair to the top, at most M from the right, 0 from the bottom | highlight, values beside wires | |
| 09_4 | An average reaches its largest part only if every part equals it. So the peak spreads along wires to the top, which is zero. | The card changes: with positive weights an average is at most its largest part, and equals it only if every part does; so a peak above 0 at a middle junction passes to every far end, and on along wires; here, to the top, where it is 0; likewise for the lowest value. On "every part equals it" the four values become M. On "spreads along wires" one M per far end travels out (the pair's second goes as the first sets out) and lands clear of its dot: right of the top and of the bottom, opposite their 0; at the right junction its "?" goes first and the M becomes its value, never two M at one place. The path's wire to the top (`R-F9.path.left`, one wire) flashes. On "which is zero" each end's 0 flashes with the M beside it | values travelling along wires, highlight, lower-third | |
| 09_5 | So the peak is zero, and likewise the lowest value. Both solutions are the same. | M becomes 0 at both middle junctions: the difference is 0 everywhere (`R-F9.peak`, `R-F9.low`). On "likewise the lowest" both 0s flash. On "Both solutions are the same" the copy is the second solution again, named "the second solution", its labels the circuit's own 1, 3/5, 2/5 and 0 and its junctions their colours; both copies' 3/5 and 2/5 flash | fraction, highlight | |
| 09_6 | Now add an island, two junctions wired only to each other. Is there still only one solution? ⟨predict⟩ | The right copy goes. On "two junctions" an island beside the circuit: two junctions joined by one wire and to nothing else, a "?" at each, named "an island". The card goes as the question ends; the countdown sits under the island | prediction pause, think time | |
| 09_7 | No. The island's peak crosses its one wire and stops, never reaching the top or bottom. Any equal pair keeps the rule. | The "?" by the countdown goes. The island's first junction takes M; on "crosses its one wire" an M crosses above the wire, clear of both dots, and hands over to the second junction's M (one after the other). On "never reaching" a dim note under the island: "no wire to the top or the bottom". On "Any equal pair" the island's two values run through 0 and 0, 1/2 and 1/2, 1 and 1, the junctions recolouring, each time with the rule at its first junction under them, its one far end over 1 equal to its value; the circuit's 3/5 and 2/5 flash, unchanged | values, equation, highlight | |
| 09_8 | So the proof needs every middle junction to reach the top or the bottom along wires. | A "precisely" lower-third: with positive conductances, if every middle junction has a path of wires to the top or the bottom, the rule with those two held has at most one solution (what the maximum and minimum argument proves; the circuit's own voltages are one); the island has no such path, so any equal pair keeps its rule (built only when the model finds the island cut off and every equal pair keeping its rule). The circuit's wires flash, then the island's | lower-third, highlight | |
| 09_9 | A walker on the island never reaches the top. Its chance is zero, but the island's voltage is left free. | A green walker steps back and forth across the island's wire. On "Its chance is zero" the note becomes "chance of reaching the top first: 0": no path of wires joins the island to either end (`R-X8.island.path`), so the walker is never there after any number of steps; the 12 steps followed agree (`R-X8.island.at_ends`). On "voltage is left free" the island's values become c and c, with a second note: "voltage: any equal pair c, c" | motion, text | |

### Notes for 09

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
- The hypothesis is explicit (09_8): every middle junction must reach the
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
- Prediction (09_6): the tempting answer is yes, since the argument was
  just made; the answer is no, because the island's peak never meets the
  top or the bottom. Nothing on screen shows an island value before the
  countdown ends.
- Card dwell, as chapter 08: every lower-third stays up at least its
  reading time at fifteen characters a second from when it is fully in:
  09_2's over 09_3, 09_4's through the 09_6 question, 09_8's to the end. A
  test checks each fits inside the lines it stays over by the scratch voice.
- Pauses for the narration (no audio exists): the one prediction hold after
  09_6; a reveal pause after "No."; completed-thought pauses before 09_6
  (the island) and before 09_9 (the walker). Directions only; no synthesis
  under the current owner amendment.

## 10_any_wires: Unequal wires (~44 s)

The bridge network on the left with its values, the battery gone: the
general rule first, on its own wires; then Doyle and Snell's unequal
wires drawn on the same shape. Every number is `R-F6`, `R-F15` or `R-P1`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | With unequal wires, weight each far end by its wire's conductance, and the walker chooses wires in that proportion. The same rule. | The network with its values 3/5 and 2/5. The left junction's links are labelled with their conductances (2 for the pair, 1 and 1) and its weights 2/4, 1/4, 1/4. Then the general forms, v(x) = Σ (c(x,y) / c(x)) v(y) and p(x) = Σ (c(x,y) / c(x)) p(y), one under the other, with their domain in a dim three-line note: x is any middle junction (the top and the bottom stay fixed); y runs over the junctions wired to x, c(x, y) being their wires' conductance, positive, one over the resistance, wires side by side added; c(x) is their sum at x | formula, equation | |
| 10_2 | Here's the same shape with unequal wires, from the book this video follows. At the left, the conductances are one up, two across and one down. | The network, its labels and the weights go; the general forms and the note stay until read. The same shape again, each link one wire drawn heavier in proportion to its conductance, labelled 1, 1, 2, 1, 2; the top 1, the bottom 0, the middle junctions faint; a dim note under it: "their conductances, drawn on this shape: Doyle and Snell, section 1.3.2". On "At the left" the left's three links flash | junctions and wires, text, highlight | |
| 10_3 | So where will a walker at the left most likely step first? ⟨predict⟩ | The general forms have gone. The walker at the left; its three links flash; no step chance is shown; the countdown in the panel | prediction pause, think time | |
| 10_4 | Across, with chance two in four, one half. So the left is a quarter, plus half the right. At the right, the same reasoning gives a fifth, plus two fifths of the left. | "1/4", "1/2" and "1/4" beside the left's links, up, across and down; row L = ¼·1 + ½·R + ¼·0. On "At the right" "1/5", "2/5" and "2/5" beside the right's links; row R = ⅕·1 + ⅖·L + ⅖·0 | values beside wires, equation | |
| 10_5 | Solved, they give seven sixteenths and three eighths, as voltages and as chances. | The step chances go; the left reads 7/16 and the right 3/8, each turning its colour; row "L = 7/16, R = 3/8", and a dim "voltage = chance" under it | fraction | |

### Notes for 10

- 10_1 is production 02_8, moved after the walker's equations and before the
  unequal example; its general note keeps its read wait (it stays over
  10_2, as chapters 08 and 09 keep cards over the lines after them).
- The example is Doyle and Snell §1.3.1 (pp. 29–32: the walk on unequal
  conductances, Pxy = Cxy / Cx) and §1.3.2 (p. 37: vc = 7/16, vd = 3/8);
  their a, b, c, d are drawn as the top, the bottom, the left and the right
  (`model::unequal`, checked against `model::doyle_snell`). Their figures
  were not extracted, so the note says "drawn on this shape", not that it is
  their figure.
- Prediction (10_3): the tempting answer is "equally likely, one third
  each", the error 06_2 already met; the answer is the heavier wire across,
  2 of 4 (`R-F15.most_likely`).

## 11_when_to_stop: When to stop (~57 s)

The unequal wires stand on the left as chapter 10 left them (7/16, 3/8, the
top 1, the bottom 0). Every number is `R-F16` or `R-X9`.

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | For programmers, the method is that weighted averaging, round after round, each round from the one before. | The "for programmers" code panel slides in on the right: the top held at 1 and the bottom at 0 in both runs, lo = 0 and hi = 1 at every middle junction, then each round every middle junction's new lo and new hi as the weighted averages of the round before, all at once, until the biggest hi − lo is under the tolerance; the last line, "exact fractions; floats: round lo down, hi up" | code panel | |
| 11_2 | To know when to stop, run it twice, from all zeros and from all ones. Averaging never swaps the order of two guesses, so the true values stay between the two runs. | The panel slides away. A table on the right: round, left, right, bigger gap; its rows 0, 1 and 2 give each junction's two runs, "0 to 1", "1/4 to 3/4", "7/20 to 11/20" at the left and "0 to 1", "1/5 to 3/5", "3/10 to 1/2" at the right, gaps 1, 1/2, 1/5. Card I: positive weights keep order: in exact arithmetic the run from 0 only rises, the run from 1 only falls, and the answer lies between; stop when they differ by less than the tolerance at every middle junction | table, lower-third | |
| 11_3 | Here the two runs come within a thousandth of each other after nine rounds, and the answer lies between them. | Under a "..." row, rounds 8 and 9: "273/625 to 274/625" and "234/625 to 47/125", gap 1/625; "1093/2500 to 219/500" and "1171/3125 to 1173/3125", gap 1/1250, which flashes beside "under 1/1000". The card goes | table, highlight | |
| 11_4 | Now run both beside the island. What happens to the two runs there? ⟨predict⟩ | The table goes. An island beside the network, two junctions joined by one wire and to nothing else, named "an island", a "?" above each junction; the countdown under it | prediction pause, think time | |
| 11_5 | They never move. The island's junctions only average each other, so its zeros stay zeros and its ones stay ones, the sign that its voltage isn't unique. | Each island junction reads "0 to 1"; a round counter steps through rounds 1 to 4 and the island's runs stay "0 to 1"; a dim note: "a gap of 1, every round" | values, highlight | |
| 11_6 | So on any finite network whose middle junctions all reach the top or the bottom, the voltage is the walker's chance. One rule, the same top and bottom, one solution. | The island goes. On the right, three dim rows: "a line of five wires: 1/5, 2/5, 3/5, 4/5", "the bridge: 3/5 and 2/5", "these wires: 7/16 and 3/8", each under "voltage = chance". Card J: finite network; conductances positive and finite; a path of wires from each middle junction to the top or bottom; top 1, bottom 0 | text, lower-third | |

### Notes for 11

- The bracket (root's review, 2026-10-20): the weighted averaging round is
  monotone with positive weights, the harmonic solution lies between the
  all-zero and all-one middle starts, and their difference at a junction is
  the chance that a walker from there is still walking after that many
  steps (`R-F16.width_is_still_walking`). A finite network with positive
  finite conductances and a route from every middle junction to an end
  absorbs the walker with chance one (a uniform positive chance within
  finitely many steps), so the widths shrink. The guarantee is exact
  arithmetic's; floating point needs outward rounding of the bounds (the
  panel's last line), not a loose margin.
- The bracket's width is not chapter 08's gap to the answer: that gap needs
  the answer, the width does not. The table's values, rows 0 to 2, 8 and 9,
  and the first round under 1/1000 are exact facts (`R-F16`).
- Beside the island (`R-X9`): its two junctions see only each other, so the
  run from zeros stays at 0 and the run from ones at 1 for ever; the unequal
  wires' own junctions still close after nine rounds.
- Prediction (11_4): the tempting answer is "they close, only slower"; the
  island's junctions average only each other.
- 11_6 states the theorem with its hypotheses (card J): finite, positive
  finite conductances, a path from every middle junction to an end. On an
  infinite graph reachability alone does not give absorption or uniqueness,
  and no such claim is made.
