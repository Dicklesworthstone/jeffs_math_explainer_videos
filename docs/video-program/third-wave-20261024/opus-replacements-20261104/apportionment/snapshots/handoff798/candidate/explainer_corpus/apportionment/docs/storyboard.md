# Storyboard: Adding a Seat

Candidate rebuild (bead mx-video-program-program-87do.30), authored in the
staging directory by WhiteSwan on 2026-11-04. It replaces the eight-chapter
schematic short. It mirrors `explainer_corpus/apportionment/docs/storyboard.md`
so root can copy it into place and run `python3 -I tools/storyboard.py --check
apportionment`. `script.tsv` beside it was generated from this file with that
tool's own `parse` and `script_text` functions.

Narration is **ElevenLabs Oliver** (owner instruction, Mail 595). No take
exists yet for any line below; the cue plan is `narration_units.json` (one Oliver take per cue) with `docs/narration-direction-map-oliver03.json` (the prediction holds).

Words are spoken exactly as written: no digits, colons, semicolons, dashes
or parentheses. `*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a
question; the beat ends with the question, the picture holds through a
four-second think ring (`draw::think`), and the next beat answers it. Every
number on screen or in the words names its fact key (`oracle.py`,
`src/facts.rs`); nothing is typed in by hand.

**One example all the way through.** Three towns that never move: Ash
(4,000 people, blue), Birch (9,000, gold), Cedar (26,000, green), 39,000 in
all. Every phenomenon in chapters 1 to 13 happens to these three towns,
except Webster's quota failure, which provably needs a fourth town (ch 11),
and the impossibility proof, which needs five (ch 14).

**Vocabulary, defined before use.** *share* (the quota: population × seats ÷
total), *whole part*, *tail* (the fraction after the whole part), *leftover
seats*, *the quota promise* (share rounded down or up), *yardstick* (people
per seat, the divisor), *signpost* (the yardstick at which a town's next seat
opens), *no take backs* (house monotonicity), *the population promise* (a
town that did not shrink never loses a seat to a town that did not grow:
Robinson and Ullman's form, as stated in Gölz, Peters and Procaccia).

**The look.** Three town rows at fixed heights (Ash top). A row is a share
bar on a seat grid (one cell per seat, `draw::CELL` wide): whole cells solid
in the town's color, the tail a lighter cell outlined in hero yellow. Seat
tokens are rounded squares. Leftover seats are hero yellow until awarded.
Broken promises flash red. The dial (ch 5, 6) and the signpost list (ch 8,
12, 13) are the two recurring devices.

## 01_one_more_seat: one more seat

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Three towns share a council of fifteen seats. Ash has four thousand people, Birch has nine thousand, and Cedar has twenty six thousand. | Three rows, names at left in town colors; each row a population bar (26,000 people = 8.6 units) with its count at the right end; header "15 seats". Bars grow from the left, one by one. | `pop.*`; bars, lagged reveal | hold |
| 01_2 | A careful rule, which we will rebuild together in a few minutes, splits the seats like this. Ash gets two, Birch gets three, and Cedar gets ten. | Bars dim and shrink to thin strips under each row; seat tokens drop into each row one by one: 2, 3, 10; a counter under the header counts up to 15. | `hamilton.15.*`; lagged tokens, live counter | hold |
| 01_3 | ⟨predict⟩ Now the council grows to sixteen seats. Nobody moves, nobody is born, and no population changes at all. There is simply one more seat to hand out, under exactly the same rule. What happens to Ash? | A hero yellow token appears above the rows marked "+1"; header becomes "16 seats"; think ring at the right. | quiz ring | listen 4 s |
| 01_4 | Ash drops from two seats to one. Birch and Cedar each gain a seat. The council got bigger, and the smallest town came out with less. | Ash's second token turns red and lifts away; the yellow token flies to Birch, a new token joins Cedar; Ash's row flashes; "2 → 1" beside Ash. | `hamilton.16.*`; flash | hold |
| 01_5 | This is called the Alabama paradox, after a real state that met it in eighteen eighty. Nothing in the arithmetic is wrong, and every step we take will be exact. The trouble lives in a deeper question. How should fair fractions become whole seats? | The rows compact to the left as "15 seats" and "16 seats" side by side; title "Adding a Seat" above. | card + rows | hold |
| 01_6 | We will rebuild the rule that did this, see exactly why it happens, meet the rules invented to avoid it, and end with a theorem about which promises no rule can keep together. | Three small icons fade in under the title in order: a dial, a list of signposts, a scorecard grid. | lagged reveal | hold |

## 02_shares: seats do not come in fractions

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Start with what fair would mean if seats could be cut into pieces. Together the towns hold thirty nine thousand people. Fifteen seats for thirty nine thousand people is one seat for every two thousand six hundred people. | The three population bars slide end to end into one bar labelled "39,000"; fifteen ticks divide it into equal pieces; a brace over one piece reads "2,600 people". | `pop.total`, `std_divisor.15`; lagged ticks | hold |
| 02_2 | So each town's fair share of seats is its population divided by two thousand six hundred. Ash's share is one point five four seats. Birch's is three point four six. Cedar's is exactly ten. Mathematicians call this exact number the quota. I will call it the town's share. | The long bar splits back into the three rows, each now a share bar on a seat grid, filled to its share; readouts at the right "1.54", "3.46", "10.00". | `share.15.*`; bars on a grid | hold |
| 02_3 | But a seat is a person. Nobody can sit in point five four of a chair. Each share has to become a whole number, and the whole numbers have to add up to exactly fifteen. | Ash's partial cell pulses; under the rows "? + ? + ? = 15". | highlight | hold |
| 02_4 | ⟨predict⟩ The obvious move is to round each share to the nearest whole number. At fifteen seats that works, because two plus three plus ten is fifteen. Will rounding to the nearest always give the right total? | Rounded values 2, 3, 10 appear beside the readouts; the sum "15" turns green; think ring. | `naive.15.*`; quiz | listen 4 s |
| 02_5 | No. At fourteen seats the shares are one point four four, three point two three and nine point three three. Every one rounds down, and the council is a seat short. At sixteen seats every share rounds up, and we hand out seventeen. | A table builds row by row: seats, the three shares, the three rounded values, the total. Rows 14, 15, 16; totals 13 (red), 15 (green), 17 (red). | `share.14.*`, `naive.14.*`, `share.16.*`, `naive.16.*`; table, lagged rows, flash | hold |
| 02_6 | Rounding each town on its own forgets the one thing that ties them together, the total. So every rule needs a second idea, a way to settle the seats that rounding gets wrong. | The total column of the table glows; the rest dims. | highlight | hold |
| 02_7 | There are two classic families. One keeps the shares fixed and compares what is left over. The other changes how many people count as a seat until ordinary rounding adds up. The paradox lives in the first family. | Table fades; two cards side by side, "compare what is left over" and "change the people per seat"; the first card glows. | card | hold |

## 03_largest_tails: whole parts first, then the largest tails

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Here is the first family's most natural rule. It carries Alexander Hamilton's name, and it is also called the method of largest remainders. Give every town the whole part of its share. Then hand out the seats that remain, one each, to the towns with the largest leftover fractions. | Share bars at 15 seats; whole cells fill solid in town color; the tail cells stay outlined in hero yellow. | `share.15.*`, `whole.15.*` | hold |
| 03_2 | Call the fraction after the whole part the town's tail. At fifteen seats, the whole parts are one, three and ten. That makes fourteen seats, so one seat is left over. | "1 + 3 + 10 = 14" under the rows; one yellow token appears above, "1 left over". | `whole.15.*`, `leftover.15`; counter | hold |
| 03_3 | ⟨predict⟩ The tails are point five four for Ash, point four six for Birch, and nothing at all for Cedar. Who gets the leftover seat? | Each tail lifts out of its bar and lines up at the right, with its value; think ring. | `tail.15.*`; quiz | listen 4 s |
| 03_4 | Ash has the largest tail, so Ash gets it. Two seats for Ash, three for Birch, ten for Cedar, the allocation we started with. | The yellow token flies to Ash's tail cell, which fills; "2, 3, 10". | `hamilton.15.*` | hold |
| 03_5 | This rule comes with a real guarantee. The shares add up to the house, and the whole parts are whole, so the tails always add up to a whole number, which is exactly the number of leftover seats. Every tail is less than one, so there are fewer leftover seats than towns, and no town can receive two of them. | The three tails slide end to end into one bar and measure exactly one seat, "0.54 + 0.46 + 0.00 = 1"; then the same at 16 seats measures exactly two, "0.64 + 0.69 + 0.67 = 2". | `tail.15.*`, `tail.16.*`, `leftover.15`, `leftover.16`; derivation, morph | hold |
| 03_6 | So every town ends up with its share rounded down or rounded up. Call that the quota promise. Hamilton's rule keeps it at every house size. Judged one size at a time, it looks perfectly fair. The trouble only appears when we compare two sizes. | Each row gets a bracket from its share rounded down to its share rounded up, the seat count inside; a green check. | `hamilton.within_quota_1_100`; box highlight | hold |

## 04_sixteenth: the sixteenth seat

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Now add the sixteenth seat. Every share grows, because the same people now divide more seats. Ash's share becomes one point six four, Birch's three point six nine, and Cedar's ten point six seven. | The header counter runs from 15 to 16; one house tracker drives all three share bars and their live readouts, which grow together. | `share.16.*`; live tracker, bars | hold |
| 04_2 | The whole parts are still one, three and ten. That is fourteen again, so this time two seats are left over. | Whole cells unchanged; "1 + 3 + 10 = 14"; two yellow tokens above, "2 left over". | `whole.16.*`, `leftover.16` | hold |
| 04_3 | ⟨predict⟩ Ash's tail is now point six four. Birch's is point six nine. Cedar's is point six seven. Which two towns get the two leftover seats? | Tails lift out and line up with their values; think ring. | `tail.16.*`; quiz | listen 4 s |
| 04_4 | Birch and Cedar. A moment ago Ash had the largest tail. Now it has the smallest, so the result is one, four, eleven. Ash's share went up, and Ash lost a seat. | The tails sort themselves by size (Birch, Cedar, Ash); the yellow tokens go to Birch and Cedar; Ash's row flashes red; "1, 4, 11". | `hamilton.16.*`; flash | hold |
| 04_5 | Look how close the race is. Measured in thirty ninths of a seat, the tails are twenty five, twenty seven and twenty six. Ash misses by a single thirty ninth. But a near miss does not explain anything. The real question is why the order of the tails changed at all. | A ruler of 39 ticks; the three tails as marks at 25, 26 and 27 in town colors; the one-tick gap between Ash and Cedar glows. | `tail39.16.*`; number line | hold |

## 05_dials: tails move at different speeds

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Picture each share as a hand on a dial. One full turn is one whole seat. Each time the hand passes the top, the whole part ticks up by one, and the tail starts again from zero. | Three dials in a row, one per town, each with a hand at its 15-seat tail and the swept tail shaded; the whole part under each dial: 1, 3, 10. | `tail.15.*`, `whole.15.*`; dial | hold |
| 05_2 | Adding one seat to the council adds the same fraction of the population to every share. Ash has four thousand of the thirty nine thousand people, so one more seat turns Ash's hand by four thirty ninths of a turn. Birch's hand turns nine thirty ninths. Cedar's turns twenty six thirty ninths, two thirds of the way around. | Each dial shows its step as a faint wedge ahead of its hand, labelled 4/39, 9/39, 26/39. | `speed39.*`; dial wedge | hold |
| 05_3 | ⟨predict⟩ At fifteen seats, Ash's hand is ahead of Birch's, and Cedar's points straight up at zero. Turn every hand by its own step. Which hand ends up in front? | Dials at 15 seats; think ring. | quiz | listen 4 s |
| 05_4 | Birch's. Ash's hand creeps forward. Birch's moves more than twice as far and passes it. Cedar's swings two thirds of a turn and passes it too. Nothing about Ash got worse. The other hands simply moved faster. | The house tracker runs 15 → 16; all three hands turn together at their own speeds; a ranking beside the dials reorders from Ash, Birch, Cedar to Birch, Cedar, Ash. | `tail.16.*`; live tracker | hold |
| 05_5 | So here is the mechanism. Hamilton's rule hands out leftover seats by rank, and the rank is decided by tails. Tails do not grow steadily with the house. They wrap around, each at its own speed. A slow hand, like a small town's, can be overtaken from one house size to the next while its whole part stays put. That is how a share can rise while its seats fall. | The tracker runs on from 16 to 20 seats and back to 16, the hands spinning at their speeds; Ash's dial outlined. | dial, live tracker | hold |

## 06_not_every_time: it comes and goes

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ Add one more seat, seventeen in all. Does Ash get its seat back? | Dials at 16 seats; think ring. | quiz | listen 4 s |
| 06_2 | It does. The whole parts become one, three and eleven, two seats are left over, and the largest tails now belong to Birch and Ash. Two, four, eleven. | Tracker 16 → 17; Cedar's hand passes the top and its whole part ticks to 11; the readout "2, 4, 11". | `whole.17.*`, `hamilton.17.*` | hold |
| 06_3 | So the paradox is not a slide. It comes and goes as the hands turn. Run Hamilton's rule for every council from one seat to a hundred. That is ninety nine additions of a single seat. Five of them take a seat away from somebody, and every time it is Ash. | A strip of 99 small cells fills left to right (each cell is one added seat); five cells light red, at 15→16, 23→24, 54→55, 62→63 and 93→94, each tagged "Ash". | `alabama.additions`, `alabama.count`, `alabama.houses`, `alabama.loser`; grid, lagged | hold |
| 06_4 | History found the same thing. After the eighteen eighty census, the chief clerk of the Census Office, a man named Seaton, worked out Hamilton apportionments for every House size from two hundred seventy five to three hundred fifty. Alabama received eight seats in a House of two hundred ninety nine, and seven in a House of three hundred. | Two compact columns, "House of 299: Alabama 8" and "House of 300: Alabama 7". | `history.seaton.*`; card | hold |
| 06_5 | Seaton reported it to Congress in a letter, which Peyton Young quotes. Such a result as this is to me conclusive proof that the process employed in obtaining it is defective. | The full quotation on screen, verbatim, with its source line. *(On screen: "While making these calculations I met with the so-called 'Alabama' paradox where Alabama was allotted 8 Representatives out of a total of 299, receiving but 7 when the total became 300. Such a result as this is to me conclusive proof that the process employed in obtaining it is defective." Young, Fairness in Apportionment, census PDF p. 13, printed p. 12)* | card (quote) | hold |
| 06_6 | Those were alternative calculations, not a real House that grew and took a seat away. But they showed that the danger was real. | The quote card shrinks to a corner. | card | hold |

## 07_yardstick: change the yardstick, not the rounding

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Now the second family. Go back to fifteen seats. Divide each population by two thousand six hundred, the exact number of people per seat, and drop every tail. You get one, three and ten. Fourteen seats, one short. | Rows with quotient bars at yardstick 2,600 (equal to the shares); a yardstick readout "2,600 people per seat"; whole parts counted, "14" in red. | `at2600.*`, `at2600.whole_total` | hold |
| 07_2 | ⟨predict⟩ Thomas Jefferson's rule keeps the rounding and changes the yardstick instead. It always rounds down, and it adjusts how many people count as one seat. To hand out one more seat, should that number go up or down? | A vertical slider for the yardstick beside the rows; think ring. | quiz | listen 4 s |
| 07_3 | Down. Fewer people per seat means every town has more seats' worth of people. Slide the yardstick from two thousand six hundred down to two thousand three hundred. Ash now has one point seven four seats' worth, Birch three point nine one, and Cedar eleven point three. Drop the tails, and you get one, three, eleven. Fifteen seats exactly. | The yardstick tracker slides 2,600 → 2,300; quotient bars and live readouts grow; Cedar's bar crosses the 11 mark and its eleventh cell fills; the count reads 15 in green. | `jefferson.at2300.*`, `jefferson.15.*`; live tracker, bars | hold |
| 07_4 | Any yardstick above two thousand two hundred fifty, and up to about two thousand three hundred sixty four, gives the same answer. That is Jefferson's rule. Choose the number of people per seat that makes the whole parts add up to the house. | A number line for the yardstick, 2,000 to 2,700, with the window from 2,250 (open) to 2,363.6 (closed) shaded; the slider sits inside it. | `jefferson.15.window_low`, `jefferson.15.window_high`; number line | hold |
| 07_5 | Notice what changed. Every town is measured with the same yardstick, and no town's tail is ever compared with another's. | Hold on the rows and the window. | hold | hold |

## 08_signposts: one list for every house size

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Look at the same slide another way. As the yardstick falls, each town gains its next seat at a definite moment. Cedar gains its first seat when the yardstick reaches twenty six thousand, its second at thirteen thousand, its third at about eight thousand six hundred sixty seven. Those are twenty six thousand divided by one, two and three. | Right half: an empty list column. Cedar's first three signposts arrive as cards, "Cedar seat 1  26,000", "Cedar seat 2  13,000", "Cedar seat 3  8,667". | `opening.1`, `opening.2`, `opening.4`; list, cards | hold |
| 08_2 | Ash's seats open at four thousand, then two thousand. Birch's open at nine thousand, four thousand five hundred and three thousand. Call each of these a signpost. Put every signpost on one list, largest first, and Jefferson's rule just reads the list from the top. Fifteen seats means the first fifteen signposts. | All signposts merge into one ranked column of 19 cards in town colors; a slider line drops to just below card 15; seat counters beside the list read Ash 1, Birch 3, Cedar 11. | `opening.1`–`opening.15`, `jefferson.15.*`; list, lagged, counter | hold |
| 08_3 | ⟨predict⟩ The first fifteen signposts give one, three, eleven. Whose signpost comes sixteenth? | Card 16 shown blank; think ring. | quiz | listen 4 s |
| 08_4 | Birch's fourth seat, at two thousand two hundred fifty. Sixteen seats gives one, four, eleven, and the seventeenth signpost, Cedar's twelfth, gives one, four, twelve. | Card 16 reveals "Birch seat 4  2,250"; the slider drops below 16, then 17; counters update. | `opening.16`, `opening.17`, `jefferson.16.*`, `jefferson.17.*` | hold |
| 08_5 | And this is why the family can never take a seat back. Sixteen seats are the first sixteen signposts, and the first fifteen are among them. A bigger house just reads further down the same list. | Nested braces at the left of the list: "15 seats" over cards 1 to 15, "16 seats" over 1 to 16. | box highlight | hold |
| 08_6 | Precisely, that needs a single order on the list. At two thousand people per seat, Ash's second signpost and Cedar's thirteenth are exactly tied, so a council of eighteen would need a tie rule. Every example in this film stays clear of that tie. | Cards 18 and 19 glow with "=" between their values, "2,000". | `opening.first_tie_rank`, `opening.first_tie_value`, `opening.18`, `opening.19` | hold |

## 09_cost: the cost of never taking back

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | ⟨predict⟩ Jefferson's rule never takes a seat back. But look at Cedar at fifteen seats. It gets eleven. Does that keep the quota promise? | Cedar's row: share bar at 15 seats, a bracket from 10 to 10, and eleven tokens; think ring. | quiz | listen 4 s |
| 09_2 | It does not. Cedar's share at fifteen seats is exactly ten, so rounded down or up it is still ten. Jefferson gives it eleven. And the extra seat came from Ash, which gets one seat for a share of one point five four. | Cedar's eleventh token turns red outside its bracket; Ash's row: one token against a share of 1.54. | `share.15.cedar`, `jefferson.15.cedar`, `jefferson.15.ash`; flash | hold |
| 09_3 | That is no accident. Rounding down throws away less than a seat from every town, but the same sliver is a bigger slice of a small town. At two thousand three hundred people per seat, Ash throws away forty two and a half percent of its count. Cedar throws away less than three percent. Jefferson's rule leans toward large towns. | Two bars at yardstick 2,300: Ash 1.74 with its 0.74 cut away and "42.5%" in red; Cedar 11.30 with its 0.30 cut away and "2.7%". | `jefferson.drop_pct.*`, `jefferson.at2300.*`; bars | hold |
| 09_4 | History shows the lean. In the apportionment after the eighteen thirty census, made with Jefferson's rule, New York's exact share was thirty eight point five nine three seats. Peyton Young's table records that it received forty. | Card: "New York after the 1830 census: share 38.593, seats 40", source line under it. | `history.ny1830.*`; card | hold |
| 09_5 | So the two families trade flaws. Hamilton's rule keeps the quota promise but can take a seat back. Jefferson's rule never takes a seat back but can break the quota promise. | A two-row scorecard (rule × promise), checks and crosses. | table | hold |

## 10_rounding_rules: round up, round to the nearest

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Jefferson rounds every quotient down. Change where the rounding happens and you get the rest of the family, each one built the same way, with a sliding yardstick and one list of signposts. | A ruler from a whole number n to n + 1 with a mark where a quotient starts to round up: for Jefferson, at n + 1. | number line | hold |
| 10_2 | John Quincy Adams proposed rounding up instead. Now any piece of a share earns a whole seat, so the lean flips toward small towns. At fifteen seats Adams gives two, four and nine. Cedar falls below its share of exactly ten. | The ruler mark moves to n (Adams); the rows show 2, 4, 9; Cedar's ninth token sits below its bracket, flashing red. | `adams.15.*`; flash | hold |
| 10_3 | ⟨predict⟩ Between always down and always up sits the compromise, round to the nearest. What does that give at fifteen seats? | Ruler mark at n + 1/2; think ring. | quiz | listen 4 s |
| 10_4 | Two, three, ten, the same as Hamilton. This is Daniel Webster's rule. At sixteen seats it gives two, four, ten, and at seventeen, two, four, eleven. Nobody ever loses a seat. In our three towns, Webster does what we hoped Hamilton would do. | A table, rows 15, 16, 17 seats, columns Hamilton and Webster; Ash's Hamilton column 2, 1, 2 (the 1 red) beside Webster's 2, 2, 2. | `webster.15.*`, `webster.16.*`, `webster.17.*`, `hamilton.16.*`; table | hold |
| 10_5 | One more relative is worth knowing, because it became the law for the United States House in nineteen forty one. It is called the Huntington Hill method, and it rounds up once a quotient passes the square root of the two neighbouring whole numbers multiplied together. For our towns it agrees with Webster at fifteen, sixteen and seventeen seats. | Ruler mark at √(n(n+1)), just left of the midpoint; a table row "Hill" equal to the Webster row. | `hill.15.*`, `hill.16.*`, `hill.17.*`, `history.hill_law_year`; table | hold |
| 10_6 | Careful, though. Hill's rule can break the quota promise even with three towns. At three seats it gives each town one seat, though Cedar's share is exactly two. Every rule in this family reads one list of signposts, so none of them ever takes a seat back, and every one of them can break the quota promise for some map of towns. | Rows at 3 seats: tokens 1, 1, 1; Cedar's bracket 2 to 2 in red. | `hill.3.*`, `share.3.cedar`; flash | hold |

## 11_webster_quota: can Webster break quota?

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | ⟨predict⟩ Webster's rule looks like the best of both families. Can it ever break the quota promise? | Webster's three rows with brackets; think ring. | quiz | listen 4 s |
| 11_2 | Not with three towns. Balinski and Young proved that with three towns, Webster's rule always keeps the quota promise, which is why our example can never show a failure. With four towns, it can. | A green check beside the three rows; "3 towns: Webster always keeps the quota promise", its citation and the film's own scan of our towns (1 to 100 seats, 0 breaks); the screen clears for four towns. | `webster.three_town_quota_breaks_1_100` | hold |
| 11_3 | Take four towns of two thousand, five thousand, eight thousand and twenty seven thousand people, sharing fourteen seats. The biggest town's share is exactly nine. The three small shares are point six seven, one point six seven and two point six seven, and each one rounds up. Rounding everything to the nearest hands out fifteen seats, one too many. | Four rows with share bars and readouts; rounded values 1, 2, 3, 9; total "15" in red. | `w4.pop.*`, `w4.share.14.*`; table | hold |
| 11_4 | To take one away, the yardstick must grow until some town's quotient drops below its halfway mark. The big town's halfway mark comes first. Its quotient falls below eight and a half while the small towns still round up. Webster gives one, two, three and eight. The town whose share is exactly nine gets eight. | Yardstick tracker 3,000 → 3,180; live quotients; the big town's quotient crosses 8.5 at about 3,176 and its ninth cell empties; result 1, 2, 3, 8; the big town's bracket 9 to 9 flashes red. | `w4.window_low`, `w4.webster.14.*`; live tracker, flash | hold |
| 11_5 | So even the best divisor rule can break the quota promise. Is there any rule that keeps the quota promise and never takes a seat back? | The four rows dim; the scorecard returns with an empty row. | table | hold |

## 12_both_promises: both promises at once

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | ⟨predict⟩ Jefferson broke the quota promise at fifteen seats by giving Cedar its eleventh seat. Suppose a rule refuses any seat that would lift a town above its share rounded up. Who gets the fifteenth seat instead? | The signpost list, slider below card 14; card 15 "Cedar seat 11" glows; Cedar's ceiling "10" beside it; think ring. | `quotamethod.15.cedar_ceiling`; quiz | listen 4 s |
| 12_2 | Birch. Michel Balinski and Peyton Young built a rule on exactly this idea in nineteen seventy five. Here it is in a simple form. Hand out seats one at a time, from the top of Jefferson's list. Before giving a town its next seat, check that the seat would not lift it above its share rounded up at the new house size. If it would, skip that signpost for now and go to the next one. | Card 15 is struck through; card 16 "Birch seat 4  2,250" slides up and is taken; counters read 1, 4, 10. | `quotamethod.15.*`, `quotamethod.15.birch_priority`, `history.quota_method_year` | hold |
| 12_3 | At fifteen seats Cedar already has ten, and its share is exactly ten, so it is skipped. The next signpost is Birch's fourth, at two thousand two hundred fifty, which beats Ash's second at two thousand. The result is one, four, ten. At sixteen seats Cedar's ceiling rises to eleven, and it gets its seat. | The struck card un-strikes and is taken at 16; counters 1, 4, 10 → 1, 4, 11. | `quotamethod.15.ash_priority`, `quotamethod.16.*` | hold |
| 12_4 | Each house is the previous house plus one seat, so nobody ever loses one. Balinski and Young proved that the skipping never strands a town below its share rounded down either. Both promises, at every house size. | The scorecard gains the row "quota rule", two checks; under it "proved by Balinski and Young, as cited by Gölz, Peters and Procaccia" and the film's own check (1 to 35 seats; at 36 two signposts tie). | `quotamethod.within_quota_checked`, `quotamethod.never_takes_back_checked`, `quotamethod.checked_through`, `quotamethod.first_tie_house`; table | hold |
| 12_5 | Notice that the quota rule gives one, four, ten at fifteen seats, while Hamilton gives two, three, ten. Both keep the quota promise. That promise can allow more than one answer, and different rules choose differently. | Both allocations side by side, each inside its brackets. | `quotamethod.15.*`, `hamilton.15.*` | hold |

## 13_population: when populations change

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | So far no population has changed. Real towns grow and shrink, which suggests a third promise. A town that did not shrink should never lose a seat to a town that did not grow. | The three population rows; a card with the promise. | card | hold |
| 13_2 | Take a later census. Ash shrinks to three thousand nine hundred. Birch grows to nine thousand five hundred. Cedar grows a little, to twenty six thousand one hundred. The council stays at sixteen seats. | Population bars change together (Ash shorter, Birch and Cedar longer) with live counts. | `census2.pop.*`; live counter | hold |
| 13_3 | ⟨predict⟩ Before the census, Hamilton gave one, four, eleven. What does it give now? | Seat tokens 1, 4, 11 beside the bars; think ring. | `hamilton.16.*`; quiz | listen 4 s |
| 13_4 | Two, four, ten. The new shares are one point five eight, three point eight five and ten point five seven. Ash's tail now beats Cedar's, so Cedar, which grew, loses a seat to Ash, which shrank. | Share bars at the new census; tails ranked Birch 0.85, Ash 0.58, Cedar 0.57; tokens move; Cedar flashes red. | `census2.share.16.*`, `census2.hamilton.16.*`; flash | hold |
| 13_5 | Divisor rules cannot do this. Every signpost is a population divided by a fixed number. When a town grows, all of its signposts move up the list. When a town shrinks, all of its signposts move down. So a growing town's signposts can only gain ground on a shrinking town's signposts, never lose it, and the shrinking town can never take a seat from the growing one. | Jefferson's list at the old census, cutoff under card 16, counters 1, 4, 11. At "grows" every card moves to its place in the list with only the growth applied (Birch's fourth passes Cedar's eleventh; Ash's second drops off the bottom); at "shrinks" to the census list (Ash's first falls from 4,000 to 3,900). The counters stay 1, 4, 11. | `opening.*`, `census2.grown.opening.*`, `census2.opening.*`, `census2.jefferson.16.*`, `census2.jefferson.paradox`; list, morph | hold |
| 13_6 | That is a strong reason to prefer divisor rules. But they cannot always keep the quota promise. Could some cleverer rule keep the quota promise and the population promise together? | Scorecard with a third column, "population promise". | table | hold |

## 14_no_rule: no rule keeps both

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Balinski and Young proved in nineteen eighty two that the answer is no, using one extra assumption about how rules treat bigger towns. A recent paper by Paul Gölz, Dominik Peters and Ariel Procaccia proves it without that assumption, for any rule that always gives the same answer to the same question. Their proof uses five towns and ten seats, and we can follow all of it. | A card with the two citations. | `history.by_impossibility_year`, `history.gpp_year`; card | hold |
| 14_2 | ⟨predict⟩ One city of eight hundred twenty four people and four villages of forty four people each share ten seats. The city's share is eight point two four, and each village's is point four four. So the quota promise lets the city have eight seats or nine. Which choice is safe? | Five rows, the city and villages two to five; shares 8.24 and 0.44; the city's bracket 8 to 9; think ring. | `gpp.A.*`; quiz | listen 4 s |
| 14_3 | Neither. Suppose the city gets nine. Then the last seat goes to one village. Call it the fifth. Now let only the fifth village grow, to two hundred twenty two people. The city's share falls to six point nine nine, so it can have at most seven seats. The fifth village's share is one point eight eight, so it can have at most two. That leaves at least one seat for the other three villages. | Left branch, "city gets 9": seats 9, 0, 0, 0, 1; the fifth village's bar grows to 222; brackets city ≤ 7, fifth ≤ 2; "at least 1" over villages two to four. | `gpp.B.*` | hold |
| 14_4 | So some village with an unchanged population goes from no seats to one, while the city, also unchanged, drops from nine seats to seven or fewer. The city did not shrink, and it lost seats to a village that did not grow. The population promise is broken. | A red arrow from the city to a village; a red cross on the left branch. | flash | hold |
| 14_5 | Suppose instead the city gets eight. Then two villages get one seat each. Call them the fourth and fifth. This time shrink the other two villages to a single person each. The city's share rises to nine point zero two, so it needs at least nine seats. Only one seat is left for the fourth and fifth villages, so one of them drops to zero. That village did not shrink, and it lost its seat to a city that did not grow. | Right branch, "city gets 8": seats 8, 0, 0, 1, 1; villages two and three shrink to 1; city bracket ≥ 9; one of villages four and five loses its token; red cross. | `gpp.C.*`; flash | hold |
| 14_6 | Both choices fail. So no rule that keeps the quota promise can keep the population promise for every map of five towns, and the same trick works with more. With three towns, Webster's rule keeps both. With four, as of that paper, nobody knew. | Summary card: "3 towns: Webster keeps both", "4 towns: open (as of the 2024 paper)", "5 or more: impossible". | card | hold |
| 14_7 | The same paper shows a way out. If the rule may draw lots, it can keep the quota promise, never take a seat back, and give every town exactly its share on average. | "a lottery rule (same paper)": three checks (quota promise in every draw; never takes a seat back; every town its exact share on average), then Ash at 15 seats: 2 seats in 54% of draws, 1 in 46%, on average 1.54. | `lottery.15.ash.*`, `share.15.ash`; card | hold |

## 15_recap: the whole map

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | Here is the whole map. Hamilton's rule keeps the quota promise, but it can take a seat back when the house grows, and it can punish a town for growing. | Scorecard table: rows Hamilton, Jefferson, Adams, Webster, Hill, quota rule; columns quota promise, no take backs, population promise; Hamilton's row fills first. | table | hold |
| 15_2 | Jefferson, Adams, Webster and Hill never take a seat back and never punish growth, but each can break the quota promise. The quota rule of Balinski and Young keeps the quota promise and never takes a seat back, and the theorem says it must break the population promise for some map. | The remaining rows fill one by one; Webster's quota cell carries the note "never with 3 towns". | table, lagged | hold |
| 15_3 | ⟨predict⟩ Here is one to try yourself. Under Webster's rule, suppose Cedar grows while Ash and Birch stay exactly the same, and the council stays at fifteen seats. Can Ash ever gain a seat? | Webster's list (17 cards) with the cutoff under card 15 and its value (2,667); counters 2, 3, 10; Cedar's cards glow; think ring. | `webster.opening.*`; quiz | listen 4 s |
| 15_4 | No. Cedar's signposts all move up the list, while Ash's and Birch's stay where they are. So the cutoff for fifteen seats can only rise, and a signpost that was below it stays below it. Ash can keep its seats or lose some, but it can never gain one. | "for example, Cedar 30,000 people": every card moves to its place in that list; Cedar's eleventh passes Ash's second, which drops below the cutoff; the cutoff value rises to 2,857; counters become 1, 3, 11. | `puzzle.pop.cedar`, `puzzle.webster.opening.*`, `puzzle.webster.15.*`; list, morph | hold |
| 15_5 | Back where we started. Ash lost a seat when the council grew, not because anyone slipped in the arithmetic, but because Hamilton's rule ranks tails, and tails wrap around at different speeds. Fair fractions are easy. Fair whole numbers force a choice, and now you know exactly which promises can be kept together and which cannot. | The opening rows at 15 and 16 seats return with the three dials beside them; the dials turn once. | dial | hold |
