# Chapter 11 storyboard, cue by cue

Follow-up02 candidate. Every line under Words is copied from the candidate
script.tsv, and `checks/script_and_captions.py` rejoins them and compares them
byte for byte. Picture is what `src/permutations.rs` is written to draw, not what
anyone has seen: nothing here has been rendered. Quoted captions are on-screen text.

## 11_1

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | A permutation is a rearrangement that sends each item to exactly one position, with no collisions and no missing destinations. | Title. Three labels send arrows into three slots. On "no collisions" two red arrows meet one slot; on "no missing destinations" a gold ring marks the empty slot. | map board |  |
| 1 | Imagine four numbered cards. | The map clears. Four numbered cards appear on their trail. | cards |  |
| 2 | We can describe a rearrangement by tracking where each label goes. | The cards stay; the labels are what the next line tracks. | cards |  |
| 3 | Let's follow card one to its new position, then the card it displaced, until the trail closes. | Card 1 pulses. | pulse |  |

## 11_2

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Suppose one goes to three, three goes to four, and four goes back to one. | Arrows draw 1 to 3, 3 to 4, 4 to 1. | traced arrows |  |
| 1 | We write the cycle one, three, four. | (1 3 4) is written. | TeX |  |
| 2 | The label two stays where it is, and we can omit that one-element cycle. | Card 2 pulses: it stays, so it is omitted. | pulse |  |
| 3 | The list is a loop, not a row of destinations: starting at three gives the same cycle, three, four, one. | A ring walks the same arrows from 3. (1 3 4)=(3 4 1). | ring walk |  |
| 4 | Reversing the direction usually gives a different permutation. | The arrows reverse. 1 maps to 3 against 1 maps to 4. "A two-label swap is its own reverse." | reversed arrows | +1.0 s |

## 11_3

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Now compute a composition. | Heading (1 2 3)(1 2), rightmost first, over a three-column route board. | route board |  |
| 1 | As before, the rightmost operation acts first. | The board stays. | route board |  |
| 2 | Take the cycle one, two, three after the swap one, two. | The board stays. | route board |  |
| 3 | Follow one: the swap sends it to two, then the three-cycle sends two to three. | Label 1 is traced through both columns, ending on 3. | traced route |  |
| 4 | Follow three: it stays put under the swap, then moves to one. | Label 3 is traced, ending on 1. | traced route |  |
| 5 | Follow two: it goes to one, then back to two. | Label 2 is traced, ending on 2. | traced route |  |
| 6 | The composition swaps one and three and fixes two. | (1 2 3)(1 2)=(1 3). | TeX |  |
| 7 | Reversing the order gives a different swap. | A second board in the reverse order, all routes at once. (1 2)(1 2 3)=(2 3). | route board | +1.0 s |

## 11_4

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | To undo a cycle, run its arrows backwards. | The cards run along the arrows, then along the same arrows reversed, and land where they began. | moving cards |  |
| 1 | The inverse of one, three, four is one, four, three. | (1 3 4) inverse = (1 4 3) is written. | TeX |  |
| 2 | If two cycles affect separate labels, they commute: each changes only positions the other leaves alone. | Two six-slot rows with separate loops run in both orders at once. On "they commute" the equation and "Overlapping: order matters. Separate: it does not." | moving cards |  |
| 3 | Every trail in a finite permutation eventually returns to its starting label. | One trail closes on the fixed slots of the top row. | ring walk |  |
| 4 | It cannot first enter an earlier label in the middle, because that would give one destination two different inputs. | A wrong arrow bends into slot 3. On "because that would": "Two cards into slot 3: impossible." | wrong arrow |  |
| 5 | So the trails form disjoint cycles. | The trail closes properly. (1 3 5)(2 6) is written. | traced arrows |  |
| 6 | Their order and the starting point chosen within each loop do not change the permutation. | (1 3 5)(2 6)=(2 6)(3 5 1). | TeX |  |

## 11_12

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Those two separate loops answer a new question. | The two-loop row returns, its loops drawn as arrows, title "Repeat it until every card is home". | six-card row |  |
| 1 | Keep repeating the same permutation, the cycles one, three, five and two, six. | Two empty strips of twelve numbered runs appear, one per loop, with a run counter at 0. | return strips |  |
| 2 | On our clock, the first return was the order of the move. | "On the clock: first return = order." | caption |  |
| 3 | Here, return has a strict meaning, every card back in its own slot at the same moment. | Gold rings on all six slots. "Here: every card in its own slot, at the same moment." | rings on slots |  |
| 4 | How many repetitions does that take? | "How many runs? Predict first." appears; nothing moves. | caption |  |
| 5 | Predict before we run it. | The board holds with the answer hidden. | hold | prediction, +3.0 s of silence |

## 11_13

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Run it once, and the three cards step around their loop while the pair trades places. | First run: the three cards step, the pair trades. No lamp lights. | moving cards, counter |  |
| 1 | Run it twice, and the pair is home, but the three cards are not. | Second run: the pair is home and lamp 2 lights on its strip; the three cards are away. | moving cards, return strips |  |
| 2 | At three runs the three cards are home, but the pair has split again. | Third run: the three cards are home, lamp 3 lights on their strip; the pair has split. | moving cards, return strips |  |
| 3 | Runs four and five still leave a loop away. | Runs 4 and 5: lamp 4 lights for the pair; run 5 lights nothing. | moving cards, return strips |  |
| 4 | At six, both loops land home together, for the first time. | Sixth run: both strips light at 6, a green frame holds the column. "Both loops home, for the first time." | moving cards, return strips |  |
| 5 | So the order of this permutation is six. | order=6. | computed TeX |  |

## 11_14

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Why six? | The board stays, stopped at run 6. | return strips |  |
| 1 | The three cards are home only on multiples of three. | The three-card strip lights its later multiples, 9 and 12. | return strips |  |
| 2 | The pair is home only on multiples of two. | The pair strip lights 8, 10 and 12. | return strips |  |
| 3 | Every card home means a number on both lists, a common multiple, and the first such number is called the least common multiple. | A dim frame on 12 shows another common multiple; lcm(3,2)=6 replaces the captions. | frames, computed TeX |  |
| 4 | Card four never moves, so it adds no condition. | Card 4 pulses. "Card 4 never moves: no condition." | pulse |  |
| 5 | Adding the lengths would give five, and at five neither loop is home. | A red frame on column 5, where neither strip is lit. "3 + 2 = 5: neither loop is home." | red frame |  |

## 11_15

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Try a second one, the four-cycle one, two, three, four, with a separate swap, five, six. | A new row, (1 2 3 4)(5 6), with its loops and two empty strips. | six-card row, return strips |  |
| 1 | Multiplying the lengths says eight. | "4 × 2 = 8?" | caption |  |
| 2 | Adding them says six. | "4 + 2 = 6?" | caption |  |
| 3 | Work out the true return before the cards move. | "Work out the first return." Nothing moves. | hold | prediction, +3.0 s of silence |

## 11_16

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | The pair is home at two and again at four. | Runs 1 and 2: the pair's lamp lights at 2. | moving cards, return strips |  |
| 1 | The four-cycle first returns at four. | Runs 3 and 4: both lamps light at 4. | moving cards, return strips |  |
| 2 | So the whole row is home after four runs, the least common multiple of four and two. | A green frame on 4. lcm(4,2)=4. | frame, computed TeX |  |
| 3 | Eight is a return too, but it is not the first one, and six is not a return at all. | Both strips light their later runs. A dim frame on 8, a red frame on 6. "8 is a later return. 6 is no return." | return strips |  |

## 11_17

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | The loops have to be separate. | Title "The loops must be separate". Three cards on three slots. | cards |  |
| 1 | In our earlier composition, the cycle one, two, three after the swap one, two gave the single swap one, three. | (1 2 3)(1 2)=(1 3) from 11_3. | computed TeX |  |
| 2 | Its lengths looked like three and two, but it returns after two runs, not six. | lcm(3,2)=6? in red. The cards actually run: home after two runs. "Home after 2, not 6." | moving cards, counter |  |
| 3 | Overlapping cycles interfere with each other. | "Overlapping cycles interfere." | caption |  |
| 4 | So first rewrite a permutation as disjoint cycles, then take the least common multiple of their lengths. | "Order = lcm of the lengths of the disjoint cycles." | caption |  |

## 11_5

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | A transposition swaps just two labels. | Heading (1 3 4)=(1 4)(1 3) over a named route board. | route board |  |
| 1 | Every cycle can be built from transpositions. | The board stays. | route board |  |
| 2 | For example, the cycle one, three, four equals the swap one, four after the swap one, three, with the rightmost swap performed first. | The board stays; the heading is the example spoken. | route board |  |
| 3 | Check the path of each of the three labels. | Each of the three labels is traced through both swaps. | traced routes |  |
| 4 | A cycle of length m can be built from m minus one transpositions, so every permutation can be built from swaps. | "A length-m cycle can be built from m − 1 swaps." | caption |  |

## 11_6

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | There may be many different swap descriptions of the same permutation. | The heading morphs to (1 3 4)=(1 4)(1 3)=(3 4)(1 4). | morph |  |
| 1 | Can one description use an even number of swaps and another use an odd number? | "Can the same move be both even and odd?" | caption |  |
| 2 | Predict before we prove it. | The board holds with the answer hidden. | hold | prediction, +3.0 s of silence |

## 11_7

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | If the answer is no, parity belongs to the permutation itself, not to our chosen description. | The question morphs to "If not: parity belongs to the permutation." | morph |  |
| 1 | Here's one way to see why parity cannot change. | The board clears. The product of differences is written. | TeX |  |
| 2 | Put distinct numbers x one through x n on the labels, and multiply all the differences x j minus x i for pairs with i less than j. | Numbers sit on fixed labels. On "multiply all the differences" and "for pairs with" each difference becomes a chip. | witness chips |  |

## 11_11

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Swap the numbers on any two labels, and this product changes sign. | The numbers on labels 1 and 3 change places; the labels stay. | witness chips |  |
| 1 | The difference between the two swapped labels reverses. | The factor between the swapped labels reverses in place. | witness chips |  |
| 2 | The two factors involving any other label exchange, sometimes both reversing signs, so their product stays the same. | Two factors through label 4 trade; two through label 2 trade and both reverse. Each pair keeps its product. | witness chips |  |
| 3 | Factors that avoid both labels do not change at all. | The factor that avoids both labels is left alone. | witness chips |  |
| 4 | The magnitude stays the same. | One swap: the product maps to its negative, same size. | computed TeX |  |

## 11_8

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | After a sequence of swaps, that product has been multiplied by minus one once per swap. | Two route rows. The three-swap route runs, one lamp flip per swap. | route rows, lamps |  |
| 1 | The final arrangement determines the final product, and the original product is not zero because the numbers are distinct. | The one-swap route runs to the same arrangement. Frames, and the nonzero starting product. | route rows, frames |  |
| 2 | So two swap sequences reaching the same permutation must have the same parity. | (1 3)=(1 2)(2 3)(1 2). "1 swap, 3 swaps: both odd." "2 swaps would give −1008: impossible." (signed value computed) | computed caption |  |
| 3 | An even permutation requires an even number of swaps in every decomposition; an odd one requires an odd number in every decomposition. | Final product = (−1) to the N times the initial product. "N = number of swaps." | TeX |  |
| 4 | Swap counts add, so an even followed by an even stays even, and the even permutations form a subgroup. | (−1) to the N₁+N₂ = (−1) to the N₁ times (−1) to the N₂ replaces the N caption. | TeX, new beat |  |

## 11_18

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Let's check that with cards. | Title "Swap counts add". Four cards, swap counter at 0. | four-card row, counter |  |
| 1 | Swap one, two and swap three, four, which is even. | Swaps 1,2 and 3,4 run, the counter reaches 2. "(1 2), (3 4): even." | moving cards, counter |  |
| 2 | Then swap one, three and swap two, four, also even. | Swaps 1,3 and 2,4 run, the counter reaches 4. "(1 3), (2 4): even." | moving cards, counter |  |
| 3 | That makes four swaps, and the cards land as the double swap one, four and two, three. | =(1 4)(2 3), computed from the swap lists. | computed TeX |  |
| 4 | Two swaps can also reach that arrangement, and two and four agree in parity. | A second row runs swaps 1,4 and 2,3, its counter reaching 2, on the same arrangement. "4 swaps, 2 swaps: both even." | moving cards, counter |  |

## 11_19

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | One more swap turns the count odd. | The lower row and notes clear. One more swap, counter 5. Table: even first, then even gives even, then odd gives odd. | moving cards, table |  |
| 1 | Two odd moves make an even one, just as the swap one, two then the swap two, three builds a three-cycle. | A fresh row runs swaps 1,2 and 2,3 and lands as (1 3 2): 1+1=2. Table: odd first, then even gives odd, then odd gives even. | moving cards, table |  |
| 2 | Write plus for even and minus for odd, and the signs simply multiply. | Table entries become (+)(+)=+ and so on. sign(pq)=sign(p) sign(q). | computed table, TeX |  |

## 11_20

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | All the permutations of our labels form a group, since they compose and each one can be undone. | "All permutations of our labels: a group." | caption |  |
| 1 | Now use our subgroup test on the even ones. | The test from 08_4: a, b in H implies a b⁻¹ in H. "H: the even ones." Four cards, counter at 0. | TeX, four-card row |  |
| 2 | They are not an empty collection, because doing nothing takes zero swaps, and zero is even. | The counter pulses at 0. "Doing nothing: 0 swaps, even." | pulse |  |
| 3 | Take two even permutations, a and b. | a=(1 2)(3 4), b=(1 3)(2 4). | TeX |  |
| 4 | To undo b, run its swaps backwards, which keeps the count. | b's swaps run in reverse, counter 2. "Undo b: same swaps, reversed." | moving cards, counter |  |
| 5 | Then do a, and the counts add to an even number. | a's swaps run, counter 4. 2+2=4. | moving cards, counter |  |
| 6 | So a times b inverse stays even, and the test passes. | "The test passes: H is a subgroup." The test pulses. | pulse |  |
| 7 | The odd permutations fail at once, because doing nothing is not odd. | "The odd ones have no identity: no subgroup." | caption |  |

## 11_21

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | How big is this subgroup on four labels? | Title "How many even permutations?" | caption |  |
| 1 | Four labels can be arranged in four times three times two times one ways, which is twenty-four. | 4×3×2×1=24. Every arrangement as a four-digit cell, twelve even in gold beside twelve odd in blue. | computed grid |  |
| 2 | Follow every arrangement by one extra swap of the first two cards. | An arrow joins each even cell to the cell with its first two cards swapped. The first pair pulses. | arrows |  |
| 3 | Each even arrangement becomes odd, each odd one becomes even, and doing it twice changes nothing. | The odd column pulses, then the even column. | pulse |  |
| 4 | So this pairing has no collisions and no missing destinations, and the two kinds are equally many, twelve even and twelve odd. | "No collisions, none missing: 12 even, 12 odd." (computed) | computed caption |  |
| 5 | The twelve even ones are doing nothing, three double swaps, and eight three-cycles. | The even cells recolour by kind: one identity, three double swaps, eight three-cycles. 1+3+8=12. | recolour, computed TeX |  |

## 11_9

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Our square symmetries are permutations of its four corners, but not every permutation of four corners preserves a square. | Square of labelled corners. On "but not every" a swap tears it. "A permutation, not a symmetry. Neighbors 2 and 3 land on opposite corners." | labelled square |  |
| 1 | All twenty-four permutations form a group, and our eight square symmetries sit inside it as a subgroup. | 4×3×2×1=24, the group S₄. On "and our eight" and "as a subgroup" the eight that keep the square light in the grid. | grid |  |
| 2 | A quarter-turn is a four-cycle, so it is odd; a half-turn swaps two pairs of corners, so it is even. | The cards turn. r=(1 4 3 2)=(1 2)(1 3)(1 4). On "a half-turn": r²=(1 3)(2 4). Swap counts computed. | moving cards |  |
| 3 | Now read the square's own moves as labels: r s is the swap two, four, and s r is the swap one, three. | Two squares perform r s and s r on the words "r s is the swap" and "and s r is the swap". | moving cards, rings |  |
| 4 | That is the same refusal to commute we saw on the tile. | The same two products as poses of the tile. rs≠sr. | tile thumbnails |  |

## 11_10

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Suppose a machine mixes a list by repeating one fixed rearrangement. | Title "Repeat one fixed rearrangement". | caption |  |
| 1 | Does the mixing go on forever, or does it eventually undo itself? | "Does the mixing go on forever?" | caption |  |
| 2 | Take the cleanest case, a perfect shuffle of ten cards. | Ten cards of one plain colour in a row, top on the left, bottom on the right. | ten-card row |  |
| 3 | Cut the deck exactly in half, then interleave the halves so the top card stays on top. | The halves split into two lanes; on "then interleave" they slide and drop into alternate slots. Shuffles: 1. | moving cards, counter |  |
| 4 | That is called an out-shuffle. | "Out-shuffle: the top card stays on top." | caption |  |
| 5 | It is one fixed permutation of the ten positions. | "Top half lands on 1, 3, 5, 7, 9. Bottom half lands on 2, 4, 6, 8, 10." | caption |  |
| 6 | How many out-shuffles bring every card home? | "How many shuffles bring every card home?" | caption |  |
| 7 | Work it out before we run it. | The board holds with the answer hidden. | hold | prediction, +3.0 s of silence |

## 11_22

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Follow positions, just as we followed labels. | The captions clear; the numbered slots are now what we read. | ten-card row |  |
| 1 | Position one stays, and so does position ten. | Gold rings on positions 1 and 10. "Positions 1 and 10 stay." | rings on slots |  |
| 2 | Two goes to three, three to five, five to nine, nine to eight, eight to six, and six back to two, which closes a six-cycle. | A ring walks 2, 3, 5, 9, 8, 6 and back, drawing each link as it is spoken. | ring walk, arrows |  |
| 3 | Four goes to seven, and seven back to four, which is a swap. | A ring walks 4, 7 and back, below the row. | ring walk, arrows |  |
| 4 | The lengths are six and two, so the least common multiple predicts six shuffles. | (2 3 5 9 8 6)(4 7) and lcm(6,2)=6, computed from the map. The plain cards take their cycles' colours. | computed TeX, token swap |  |

## 11_23

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Now shuffle for real. | The arrows clear; two six-run strips arrive. The second shuffle runs. Lamp 2 lights for the pair. | moving cards, return strips |  |
| 1 | After two shuffles the swapped pair is home, yet most of the deck is still out of place. | Rings on the pair. On "yet most": "6 of 10 still away" (computed). Shuffles 3, 4, 5 run. | moving cards, return strips |  |
| 2 | After six, every card is home, exactly as predicted. | The sixth shuffle lands on the words. Green frame. "All ten home." | moving cards, return strips |  |
| 3 | One shuffle was enough to read the cycles, and the cycles gave us that number. | The written cycles pulse. | pulse |  |

## 11_24

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | A full deck has fifty-two cards, and more arrangements than anyone could ever list. | Fifty-two bars as a staircase, each one a card in its position. 52! is about 8×10⁶⁷ arrangements (computed). | bars |  |
| 1 | The out-shuffle sorts its positions into two fixed points, one swap, and six cycles of length eight. | A strip of segments as wide as each cycle, filling the deck's width. "2 fixed, 1 swap, 6 cycles of length 8." | cycle strip |  |
| 2 | The least common multiple is eight, so eight out-shuffles restore the whole deck. | lcm(1,2,8)=8. | computed TeX |  |
| 3 | Watch the staircase break apart and come back. | Eight out-shuffles run with a counter: the staircase breaks apart and comes back. | bars, counter | runs past the words; about 4 s of watching |

## 11_25

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | Now interleave the other way, so the top card drops to second place. | "In-shuffle: the top card drops to second place." One in-shuffle from the restored staircase. | bars, counter |  |
| 1 | That is an in-shuffle, and its fifty-two positions form one single cycle. | One segment as wide as the deck under the first strip, with 8 and 52 at the ends. | cycle strip |  |
| 2 | The return takes fifty-two shuffles, six and a half times as long, from almost the same motion. | Seven more in-shuffles. "After 8 in-shuffles: 0 cards home." (computed) | bars, counter |  |
| 3 | Watching the hands would not tell you that, but the cycle lengths do. | The 8 and the 52 pulse. | pulse |  |

## 11_26

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | So repeating one fixed rearrangement never mixes forever. | "One fixed rearrangement walks a closed loop." | caption |  |
| 1 | It walks a closed loop, and the order tells us when the loop ends. | "The order is the length of that loop." | caption |  |
| 2 | Designers of parallel computers have used this same shuffle to wire processors together. | The bars clear. Three stages of the same shuffle drawn as lines between columns; eight numbered tokens on the first column. | wires, tokens |  |
| 3 | With eight wires, repeating it puts data back on their starting wires after three stages. | The tokens follow the lines through three stages and land on their own rows. "After 3 stages: every line back where it began." | wires, tokens |  |
| 4 | Without switches to change the route, a fourth stage would only repeat the first. | A fourth stage in red; the tokens move as in stage 1. "Without switches, stage 4 repeats stage 1." | wires, tokens |  |

## 11_27

| s | Words | Picture | Device | Hold |
|---|---|---|---|---|
| 0 | A composition predicts the combined effect, and the inverse tells us how to undo it. | A fresh staircase. p⁸=e. | bars, computed TeX |  |
| 1 | Here, seven more out-shuffles undo the first. | One shuffle, then seven more, and the staircase is back. p⁻¹=p⁷ (computed). | bars |  |
| 2 | The group records what the moves do; it does not by itself guarantee that a physical device performs them without error. | "The group records the moves, not how well a device performs them." | caption |  |
| 3 | Suppose two neighboring cards trade places during one shuffle. | One shuffle, then two neighbouring bars turn red and trade places. | bars, recolour |  |
| 4 | That is one extra swap, so the eight shuffles now end on an odd permutation, and doing nothing is even, so the deck cannot be home. | The other seven run. The staircase returns with the two red bars still swapped. "One extra swap: odd, so not home." (computed) | bars |  |
| 5 | Next we'll ask when two groups with different-looking objects actually have the same structure. | "Next: when do two groups have the same structure?" | caption |  |
