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
| `01_the_same_number` | the circuit: current in equals current out, so each middle junction is the average of its wires' far ends (3/5 and 2/5 pass); the walker on the same wires: one step ahead, its chance is the same average; prediction; the same number, because one rule with the same top and bottom has only one solution; the walker is a model of the equations, not of electrons | source written by SandyMountain for `mx-video-program-program-87do.18.1`; not compiled, rendered or narrated in the lane (central batch pending) |
| later chapters (not yet boarded) | why the averaging rule holds in each picture (Ohm and Kirchhoff; first-step analysis); finding the values by repeated averaging; why there is only one solution (the maximum principle, and the island where it fails); every route added up for this network (a geometric series) against the rule that needs no routes; the current as the walker's net crossings (Doyle and Snell §1.3.3) | proposals only |

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
ends); **solution** (a set of values that keeps the rule).

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
