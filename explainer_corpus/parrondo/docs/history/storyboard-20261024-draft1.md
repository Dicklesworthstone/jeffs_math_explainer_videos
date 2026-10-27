# Storyboard — Two Losing Games Can Win — Here Is the Catch

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py parrondo` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact computed in
`src/facts.rs` and recomputed another way by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` closes a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it. Nothing that answers a question
is on screen before its answer beat starts.

## The arc

First draft of 2026-10-24 (WildBass), thirteen chapters. One triangle runs
through the whole video: three corners named 0, 1 and 2, the remainder when
your dollars are divided by three. A win moves one corner forward (0 → 1 →
2 → 0), a loss one corner back. Every game is drawn on it as six arrows.

The continuing example is a **crowd** on that triangle. Without the half
point of handicap, game B's crowd of **26** players settles at **10, 4, 12**
and stays there in whole numbers: every player moves each round and the
counts do not change; 13 of the 26 win. The mix moves that crowd off corner
0, and one line (B's chance of winning against its share of rounds at corner
0) turns the move into money. The published coins then restore the opening's
numbers, and the cycle-product test is the rigorous form of the triangle.

- Act I (01, 02): the rules, a cent a round lost by A in the first half
  minute, the honest-looking count that says B wins, and the claim; then
  remainders and the triangle.
- Act II (03 to 05): round numbers; a crowd sloshes and settles; 26 players
  checked in whole numbers; the trap at corner 0 and a die-chosen control.
- Act III (06 to 08): the mix shifts the crowd; one dial explains the gain;
  the half point returns and the opening's numbers are earned.
- Act IV (09, 10): net flow around the triangle, and the two products that
  decide every game.
- Act V (11 to 13): when switching fails, what "wins" promises over finite
  play, and the catch on the opening picture.

| Chapter | Spine | Status |
|---|---|---|
| `01_two_games` | both rules; A loses a cent a round; the equal-thirds count says B wins 52.8 %; does B win? (a prediction); B loses, the mix wins; title | source written 2026-10-24 (`two_games.rs`), uncompiled, no render, no narration |
| `02_three_corners` | remainders; the triangle; B and A as arrows; six dollars or six hundred; A's share at corner 0 (a prediction) | animatic |
| `03_round_numbers` | the half point put back; equal thirds would give 53.3 %; do the corners share equally? (a prediction); a crowd of 26 sloshes and settles | animatic |
| `04_ten_four_twelve` | 10, 4, 12 checked arrow by arrow; arrivals (a prediction); 13 winners; 38.5 % at the bad coin | animatic |
| `05_the_trap` | where a player at corner 0 goes next (a prediction); the trap; a die-chosen coin wins 53.3 %; weights named | animatic |
| `06_mix_the_games` | the averaged arrows; which way the crowd shifts (a prediction); 38.5 → 34.6 % at corner 0 | animatic |
| `07_one_dial` | A rounds break even; B wins 75 % minus 65 points per unit of share at corner 0; at 34.6 % (a prediction) 52.5 %; the mix +2.54 cents | animatic |
| `08_the_half_point` | the handicap restored; B's threshold 37.7 % against its 38.4 %; does the mix survive? (a prediction); the opening's numbers; the source | animatic |
| `09_net_flow` | the mix's crowd of 1,418; twelve net across each edge; dollars gained per round (a prediction); gain = 3 × flow | animatic |
| `10_two_products` | balanced edges multiply to equal products; B balanced at 9/160; the mix's products (a prediction); the sign rule | animatic |
| `11_when_switching_fails` | ordinary coins mix to their average; strict alternation (a prediction) loses; AABB wins; A's share between 7.0 % and 84.7 % | animatic |
| `12_what_winning_promises` | 100 rounds: 51.7 % ahead; 1,000 rounds (a prediction): 68.9 %; the long-run law; not advice | animatic |
| `13_the_catch` | the opening picture explained; A's products (a prediction); win-win-win against lose-lose-lose; names to search | animatic |

Every chapter's narration, render, native timing, listening and audiovisual
acceptance is open. Chapters 02 to 13 are registered as storyboard animatics
until their scenes are written.

## Vocabulary (one word per idea)

See `docs/brief.md`. In the voice: **game A**, **game B**, **the mix**,
**round**, **bad coin**, **good coin**, **your dollars**, **divides by
three**, **remainder**, **corner**, **forward**, **back**, **crowd**,
**settle**, **weight**, **average per round**, **net flow**, **forward
product**, **backward product**, **the half point**, **handicap**. Never in
the voice: drift, expected value, stationary distribution (screen only, as a
name to search), strategy, half a cent (the handicap is half a percentage
point of chance; it costs game A one cent a round).

## Color roles

| Role | Color | Used for |
|---|---|---|
| game A | blue `#58C4DD` | card A, A's arrows, A rounds |
| game B | violet `#C792EA` | card B, B's arrows, B rounds |
| the mix | gold `#F0AC5F` | the mix's arrows, bars and verdicts |
| bad coin / corner 0 | red `#FC6255` | the bad coin, corner 0's tint, losses that matter |
| good coin | green `#83C167` | the good coin, wins |
| crowd | teal `#5CD0B3` | the players (dots) and their counts |
| highlight | hero `#FFD866` | questions, the countdown, the rule being pointed at |
| structure | ink / dim / faint | text, back arrows, frames |

## Sources and scope

Harmer & Abbott, *Nature* 402, 864 (1999) for the rules; Ethier & Lee, *EJP*
14 (2009) for the strong law and the exact means the oracle reproduces
(18/709; AB 0; AABB 4/163; ABB 2416/35601; AAB 32/1609). The video makes no
claim about finance, investing, biology or casinos, and no historical claim
beyond the paper's own naming sentence. See `docs/brief.md`.

## 01_two_games: Two games, two coins (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Here are two games. Every round, you win a dollar or lose one. | Two cards fade in side by side: "Game A" (blue title) left, "Game B" (violet title) right, each a rounded dim frame. Over them, dim: "each round: win 1 dollar or lose 1 dollar" | cards | |
| 01_2 | Game A is one bent coin. It lands heads forty-nine and a half percent of the time, and heads wins. | In card A: a blue coin; on "forty-nine" the line "heads 49.5 % → win 1" in green, then "tails 50.5 % → lose 1" in dim | coin, text from facts | |
| 01_3 | In a hundred rounds you expect forty-nine and a half wins and fifty and a half losses. That's a dollar lost, a cent a round. | Under card A, TeX "100 rounds: +49.5 − 50.5 = −1 dollar"; on "a cent a round" the line "−1 cent a round, on average" in red | TeX from facts | |
| 01_4 | Game B has two coins and a rule. Before each round, count your dollars. | In card B: a red coin and a green coin; on "count your dollars" the rule's first line "count your dollars", in hero | coins | |
| 01_5 | If the count divides by three, you toss the bad coin, heads only nine and a half percent of the time. Otherwise you toss the good coin, heads seventy-four and a half percent. | The rule's branches under it: "divides by 3 → bad coin: heads 9.5 %" (red), "otherwise → good coin: heads 74.5 %" (green), each beside its coin | text from facts | |
| 01_6 | Two counts in every three don't divide by three. If the good coin really got two rounds in three, game B would win fifty-two point eight percent of them. | Under card B: "if each remainder took a third of the rounds:" (dim) and TeX ⅓ · 9.5 % + ⅔ · 74.5 % = 52.8 % with "B would win" in green | TeX from facts | |
| 01_7 | So does game B win or lose in the long run? ⟨predict⟩ | The question on screen, "Does game B win or lose in the long run?"; four seconds of silence with the countdown ring and "pause for more time"; no verdict on screen | think countdown | |
| 01_8 | It loses, a little under a cent a round. Now flip a fair coin before every round to choose A or B. That mix of two losing games wins, just over one and a half cents a round. | The ? and the question go; the cards shrink upward. Three rows with bars to scale about a zero line, each with its exact rounded value: "game A: −1.00 cents a round" (blue, left), on "It loses" "game B: −0.87" (violet, left); on "Now flip" a gold coin; on "That mix" "the mix: +1.57" (gold, right). A "precisely" card (K01). Then all clear and the title card stands: "Two Losing Games Can Win" / "Here is the catch" | bars to scale from facts, lower-third, title card | hold |
| 01_9 | The catch is hidden in game B's rule, the part that looks at your money. | Card B returns to the centre; "count your dollars" is outlined in hero and flashes. It stands, then the chapter fades | highlight | |

### Notes for 01

- First payoff: 01_3 ends near 0:25 with A's average computed aloud from the
  rule (49.5 − 50.5 per hundred rounds). The second, the equal-thirds count,
  lands by 0:50 and is the setup the whole video answers.
- Prediction (01_7): the question appears on "does game B", before the
  question ends; four seconds of countdown; no bar until 01_8 starts.
- Exact version K01 (01_8): "long-run average per round, exactly: A −1/100,
  B −73443/8446300, the mix +223/14200 dollars. Coins of Harmer & Abbott
  (Nature, 1999) with ε = 0.005; the mix: a fair coin picks A or B before each
  round. These averages hold with probability one in the long run (Ethier &
  Lee, 2009)."
- The bars are drawn to scale from the facts; the scene refuses to draw if A
  and B are not both below zero or the mix is not above it.
- Pause intentions (for listening, not rules): a full stop after "lose a
  dollar" (01_1) before the rules begin; a short thinking beat after "a cent
  a round" (01_3); "Otherwise" (01_5) continues the same thought, no long gap;
  after "fifty-two point eight percent of them" (01_6) the question follows
  closely; after "It loses" (01_8) a brief beat before "a little under a
  cent"; a longer turn before "Now flip a fair coin", which starts the
  surprise.
- Callbacks: 01_6's equal-thirds count returns in 03_3, 05_6 and 07_8; the
  three bars return in 08_7.
- Pronunciation to probe: "game A" (the letter, not the article),
  "forty-nine and a half", "seventy-four and a half", "fifty-two point eight".

## 02_three_corners: Only the remainder matters (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Game B asks one question about your money. When you divide your dollars by three, what's left over? | Card B's rule shrinks into the top-left corner. A row of seven teal dots, grouped in threes with lagged outlines, one left over | lagged reveal, dots | |
| 02_2 | Six leaves nothing over. Seven leaves one, eight leaves two, and nine leaves nothing again. That leftover is the remainder, and it's always zero, one or two. | The dots regroup for 6, 7, 8 and 9; under each count its remainder 0, 1, 2, 0 appears; the word "remainder" under the row | lagged reveal | |
| 02_3 | So draw three corners, one for each remainder. A win moves you one corner forward, and from two that means around to zero. A loss moves you one corner back. | A large triangle: corner 0 at the top, 1 lower right, 2 lower left. Forward arrows clockwise (0→1→2→0) in green, back arrows inside in dim. A teal token at 7 dollars sits on corner 1; it steps forward to 8 (corner 2), forward to 9 (corner 0), back to 8 | arrows, token moves | |
| 02_4 | Write game B on the triangle. From corner zero, the bad coin, forward nine and a half percent of the time. From corners one and two, the good coin, forward seventy-four and a half percent. | Corner 0 tints red. Arrow labels from facts: 0→1 "9.5 %", 0→2 "90.5 %"; 1→2 and 2→0 "74.5 %", 1→0 and 2→1 "25.5 %" | labels from facts | |
| 02_5 | Game A is the same from every corner, forward forty-nine and a half percent, back fifty and a half. | A small blue copy of the triangle beside it, all forward arrows "49.5 %", all back arrows "50.5 %" | small multiple | |
| 02_6 | Whether you hold six dollars or six hundred, game B plays the same way at corner zero. So what decides game B is how often you stand at each corner. | "6" and "600" both slide into corner 0. A "precisely" card (K02) | lower-third | |
| 02_7 | If you play only game A for a long time, what share of the rounds do you spend at corner zero? ⟨predict⟩ | The question on screen over A's triangle; countdown; no shares shown | think countdown | |
| 02_8 | One third. Game A treats every corner alike, so in the long run the corners share the rounds equally. Game B does not. | A's three corners each fill a bar to "1/3". B's corner 0 pulses red | bars, flash | |

### Notes for 02

- K02: "the capital is any whole number, negatives included, and play never
  stops; the remainder is 0, 1 or 2, so −1 leaves 2 and −3 leaves 0".
- Prediction (02_7) answered at 02_8's first words; the bars grow with them.
- Pause intentions: after the question in 02_1, a short gap (it is answered
  at once); a completed-thought pause after "zero, one or two" (02_2); hold the
  triangle a beat after "one corner back" (02_3) so the token's last step is
  seen; "So what decides game B" (02_6) follows without a long gap.
- Pronunciation: "remainder", "ninety and a half" (screen only).

## 03_round_numbers: Round numbers and a crowd (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | The percentages in these games are round numbers with half a percentage point shaved off. For now, put the half point back. | Beside each coin: 49.5 % → 50 %, 9.5 % → 10 %, 74.5 % → 75 %, each with a small "+0.5" that rises and fades | count-up from facts | |
| 03_2 | Game A becomes a fair coin, heads exactly half the time, so on average it neither wins nor loses. Game B gets a ten percent coin at corner zero and a seventy-five percent coin everywhere else. | Both triangles relabel: A's arrows "50 %"; B's 0→1 "10 %", 0→2 "90 %", the others "75 %" and "25 %" | labels from facts | |
| 03_3 | With equal thirds at every corner, B would now win fifty-three point three percent of its rounds. | TeX ⅓ · 10 % + ⅔ · 75 % = 53.3 % (callback to 01_6) | TeX from facts | |
| 03_4 | But do the corners really get a third each under game B? ⟨predict⟩ | The question on screen; countdown | think countdown | |
| 03_5 | Watch a crowd find out. Twenty-six players start at corner zero, and every one of them plays game B. | A's triangle leaves; B's triangle grows to centre. 26 teal dots gather at corner 0 with "26" beside it; a round counter "round 0" | dots, live count | |
| 03_6 | Let the crowd split exactly by the coins' chances. After one round, a tenth of it moves forward and nine tenths fall back to corner two. | The dots become a bar at each corner (height ∝ players). Round 1: corner 0 empties, corner 1 rises to 2.6, corner 2 to 23.4, flowing along the arrows. A "precisely" card (K03) | bars, live numbers | |
| 03_7 | The next round, three quarters of corner two win their way around to zero. The crowd sloshes back and forth between corners zero and two. | Rounds 2 to 6 play at one a second: (18.2, 5.9, 2.0), (2.9, 2.3, 20.8), (16.2, 5.5, 4.4), (4.6, 2.7, 18.7), (14.7, 5.1, 6.2); the counter ticks | bars, live numbers | |
| 03_8 | Round after round the sloshing dies down, and the crowd settles at ten players on corner zero, four on corner one and twelve on corner two. | Rounds 7 to 30 quicken; the bars converge; "10", "4", "12" settle in teal; small dim "within half a player from round 24" | bars, live numbers | hold |

### Notes for 03

- K03: "the crowd splits by expected numbers: 26 times each corner's chance.
  Real players scatter around these numbers."
- The bars show one decimal; 26 dots return in 04 for the whole-number check.
- Prediction (03_4): no bar heights or crowd before 03_5.
- Pause intentions: a turn after "put the half point back" (03_1); a
  thinking beat after "fifty-three point three percent of its rounds" (03_3)
  before the question; a held breath on "Watch a crowd find out" (03_5) as
  the dots gather; a listen-quiet hold after "twelve on corner two" (03_8) so
  the settling is seen (hold).

## 04_ten_four_twelve: A crowd that stays put (~71 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Check that this crowd really stays put. At corner zero, ten players toss the bad coin. One wins and moves forward. Nine lose and drop back to corner two. | The bars become 26 dots again: 10 at corner 0, 4 at 1, 12 at 2. Corner 0's dots split: one beside the forward arrow (green "1"), nine beside the back arrow (dim "9"); they wait at the arrow's start | dots, labels from facts | |
| 04_2 | At corner one, four players toss the good coin. Three win and move on to corner two, and one falls back to zero. | Corner 1's dots split: "3" forward, "1" back | dots | |
| 04_3 | At corner two, twelve players. Nine win and wrap around to zero, and three fall back to one. | Corner 2's dots split: "9" forward, "3" back | dots | |
| 04_4 | Now let everyone move at once. How many players land on each corner? ⟨predict⟩ | The question on screen; countdown; the dots still wait | think countdown | |
| 04_5 | Corner zero gets one plus nine, ten. Corner one gets one plus three, four. Corner two gets nine plus three, twelve. Every player moved, and the crowd looks exactly the same. | All 26 dots move along their arrows together; arrival sums appear at each corner "1 + 9 = 10", "1 + 3 = 4", "9 + 3 = 12" | swarm move, TeX from facts | |
| 04_6 | Count the winners. One, three and nine make thirteen, exactly half of twenty-six. Without the half point, game B is perfectly fair. | The 13 forward movers flash green; "13 of 26 win = 1/2" | flash | |
| 04_7 | Now look at where the crowd sits. Ten of twenty-six is thirty-eight and a half percent, more than a third. | Corner 0's 10 dots outlined red; a bar "38.5 %" beside a dim marker at "33.3 %" | bar from facts | |
| 04_8 | Those extra rounds at corner zero, with the bad coin, cancel the good coin's advantage exactly. | The equal-thirds count from 03_3 (53.3 %) slides down to "50 %" as the share moves from 33.3 % to 38.5 % | tween | |

### Notes for 04

- Prediction (04_4): the arrivals are not summed on screen before 04_5.
- Facts: the flows are whole players (26 is the smallest crowd for which
  they are; the oracle searches every smaller crowd).
- Lint disposition: 04_5's three sentences opening "Corner …" are the count
  itself, said in the order the arrivals appear; kept on purpose.
- Pause intentions: each corner's split is its own thought (04_1 to 04_3),
  with short completed-thought gaps; after "looks exactly the same" (04_5) a
  longer hold so the unchanged counts register; "perfectly fair" (04_6) ends
  a section.

## 05_the_trap: Why corner zero collects players (~81 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Why does corner zero collect extra players? Follow one who has just arrived there. Where is that player most likely to be one round later? ⟨predict⟩ | The crowd dims; one teal dot at corner 0, outlined. The question on screen; countdown | think countdown | |
| 05_2 | At corner two, nine times in ten. The bad coin knocks them back. And from corner two, the good coin usually wins, which carries them forward, around to zero again. | The dot drops to corner 2 ("90 %"), then wins back to 0 ("75 %"), tracing a red-and-green path | path | |
| 05_3 | So most of the good coin's wins at corner two just bring the player back to zero, where the bad coin waits. | The 0 ↔ 2 path repeats three times, quicker, its trace thickening | path | |
| 05_4 | That's the trap in game B. The good coin wins often, but its wins keep getting spent on climbing back to a multiple of three. | The thick 0 ↔ 2 trace glows; corner 0's red tint deepens | glow | |
| 05_5 | One change removes the trap. Let a die choose B's coin instead of your money. A one or a two means the bad coin, anything else the good coin. | A die beside the two coins; faces 1, 2 point to the red coin, 3 to 6 to the green coin. The triangle dims: the corners no longer matter | die, arrows | |
| 05_6 | Now the bad coin really does get one round in three, wherever you stand, and this game wins fifty-three point three percent of the time. | TeX ⅓ · 10 % + ⅔ · 75 % = 53.3 % (callback), "wins" in green | TeX from facts | |
| 05_7 | Same two coins. Only the rule that picks between them changed. Reading your money is what drags game B from fifty-three point three percent down to exactly half. | The die fades; the triangle returns, "count your dollars" outlined; "53.3 % → 50 %" beside it | highlight | |
| 05_8 | Each corner's long-run share of the rounds is called its weight. For game B, the weights are five thirteenths, two thirteenths and six thirteenths. | At the corners: 5/13, 2/13, 6/13 (= 10, 4, 12 of 26). Lower-third: "names to search: stationary distribution · Markov chain" | lower-third, labels from facts | |

### Notes for 05

- The die-chosen game is the negative control for "B loses because of where
  you stand": the same coins with the choice made independently of the money
  give an ordinary coin that wins 8/15 = 53.3 % with the round-number coins
  (317/600 = 52.8 % with the published ones), so it wins.
- Pause intentions: after "nine times in ten" (05_2) a short beat; the trap
  sentence (05_4) is a completed thought with a pause after it; "One change
  removes the trap" (05_5) starts a new idea after a turn.
- Pronunciation: "thirteenths" (watch for "thirtieths").

## 06_mix_the_games: Mix the games (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now mix the games. Before every round, a fair coin decides whether you play A or B. | A gold coin above the triangle, faces "A" and "B" | coin | |
| 06_2 | On the triangle, every arrow becomes an average. At corner zero, half the time it's A's fifty percent, half the time B's ten percent. So forward thirty percent in all. | Corner 0's forward arrow: blue "50 %" and violet "10 %" merge into gold "30 %"; the back arrow "70 %" | label morph from facts | |
| 06_3 | At corners one and two, halfway between fifty and seventy-five, forward sixty-two and a half percent. | The other forward arrows morph to gold "62.5 %", back "37.5 %" | label morph | |
| 06_4 | Start the mix from game B's settled crowd, ten, four and twelve. Which way will the crowd shift? ⟨predict⟩ | B's settled bars (10, 4, 12) return; question; countdown | think countdown | |
| 06_5 | Off corner zero. Round by round, corner zero's share falls from thirty-eight point five percent to thirty-four point six, and corner one's climbs from fifteen point four to twenty-five point four. | The bars flow under the mix's arrows; live shares: corner 0 38.5 % → 34.6 %, corner 1 15.4 % → 25.4 %, corner 2 46.2 % → 40.1 %; a ghost outline keeps B's old heights | bars, live numbers | |
| 06_6 | Game A's coin ignores the corners, so its rounds keep stirring the crowd toward equal thirds. That shift of weight off corner zero is where the winning comes from. | A dim dashed line at 33.3 % on each bar; corner 0's lost weight glows | glow | hold |

### Notes for 06

- The shift is the reveal: keep B's ghost heights visible beside the mix's
  bars through 06_6.
- Pause intentions: after "every arrow becomes an average" (06_2) a short
  beat; after the shares (06_5) a held pause so the numbers can be read; a
  hold after 06_6 (the reveal).

## 07_one_dial: One dial decides game B (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Split the mix's rounds into A rounds and B rounds. The A rounds win exactly half the time, at every corner, so they can't be where the winning comes from. | The mix's rounds as a strip of blue and violet tiles; the blue tiles show "50 %" | tiles | |
| 07_2 | Game B's chance of winning depends on just one number, its share of rounds at corner zero. Every round it spends there swaps a seventy-five percent coin for a ten percent one. | A plot: horizontal axis "B's share of rounds at corner 0" from 30 % to 40 %, vertical "B wins" from 48 % to 56 %; the falling line 75 % − 65 % × share | plot, axis | |
| 07_3 | At a third, B wins fifty-three point three percent. At game B's own thirty-eight point five, exactly half. | Two points on the line: (33.3 %, 53.3 %) green, (38.5 %, 50 %) violet | plot points from facts | |
| 07_4 | Inside the mix, B's rounds spend thirty-four point six percent at corner zero. So what do they win? ⟨predict⟩ | A gold vertical marker at 34.6 %; question; countdown; no value on the line | think countdown | |
| 07_5 | Fifty-two point five percent. Averaged with the A rounds' fifty, the mix wins fifty-one point three percent of all its rounds. | The marker meets the line at 52.5 %; TeX ½ · 50 % + ½ · 52.5 % = 51.3 % | TeX from facts | |
| 07_6 | On average, that's a gain of just over two and a half cents a round. | "+2.54 cents a round" in gold; a "precisely" card (K05) | lower-third | |
| 07_7 | So game A's rounds only break even. They move the crowd off the bad corner, and game B's good coin does the winning. | The blue tiles dim; the violet tiles glow | glow | |
| 07_8 | Remember the first count, with a third of the rounds at each corner? The mix pushes the crowd most of the way back to it. | On the plot, an arrow from 38.5 % toward 33.3 % stopping at 34.6 %; "76 % of the way" | arrow from facts | |

### Notes for 07

- K05: "exactly: a crowd of 1,418 at 490, 360, 568 under the mix sends 147,
  225, 355 forward and 343, 135, 213 back; every corner gets back what it
  sends; 727 win, 691 lose: 18/709 dollars a round (Ethier & Lee 2009 give
  18/709)".
- The line is exact: B wins 3/4 − (13/20)·share, so B is fair at share 5/13.
- Prediction (07_4): the answer is between 50 % and 53.3 %, nearer 53.3 %.
- Pause intentions: completed-thought pause after "can't be where the
  winning comes from" (07_1); a thinking beat after the two points (07_3);
  the answer (07_5) begins promptly after the countdown.

## 08_the_half_point: The half point returns (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Now shave the half point off every coin again, as Harmer and Abbott did when they published these games. | 50 % → 49.5 %, 10 % → 9.5 %, 75 % → 74.5 %, each with "−0.5" | count-down from facts | |
| 08_2 | Game A slips from fair to losing a cent a round. | Row "game A: 0 → −1.00 cents a round" | live number | |
| 08_3 | Game B's crowd barely moves. Corner zero still holds thirty-eight point four percent of its rounds, and with the weaker coins that's now too many. B loses a little under a cent a round. | The dial's line drops by half a point; its fair point moves from 38.5 % to 37.7 %; B's point at 38.4 % sits past it; "game B: −0.87" | plot from facts | |
| 08_4 | The mix had an edge of just over two and a half cents a round. Does it survive the handicap? ⟨predict⟩ | "the mix: +2.54" with a question mark; countdown | think countdown | |
| 08_5 | It does. The half point costs the mix about one cent a round, and the edge was bigger than that. The mix still wins just over one and a half cents a round. | "+2.54 − 0.97 = +1.57 cents a round" in gold | TeX from facts | |
| 08_6 | Inside the mix, corner zero still holds only thirty-four and a half percent, so B's rounds still win fifty-two point one percent. | The gold marker on the lowered line at 34.5 % meets it at 52.1 % | plot from facts | |
| 08_7 | That's every number from the opening. A loses a cent a round, B a little under a cent, and the coin-flip mix of the two wins. | The opening's three bars return exactly as in 01_8 (callback) | bars from facts | |
| 08_8 | Harmer and Abbott named the effect after its discoverer, Juan Parrondo. Their paper, in Nature in nineteen ninety-nine, is a single page. | Citation card: "G. P. Harmer & D. Abbott, Losing strategies can win by Parrondo's paradox, Nature 402, 864 (1999)" | citation card | hold |

### Notes for 08

- The handicapped numbers: B's weights 38.4 %, 15.4 %, 46.2 %; B's fair
  share threshold 49/130 = 37.7 %; the mix's weights 34.5 %, 25.4 %, 40.1 %;
  B rounds inside the mix 52.1 %; the mix's win 50.8 %.
- History: only the paper's own sentence ("called Parrondo's paradox, after
  its discoverer, Juan Parrondo") and its date and length. No year of
  discovery, no affiliation.
- Pause intentions: "It does." (08_5) is followed by a short beat; the
  citation (08_8) holds.
- Pronunciation: Juan Parrondo ("hwahn pah-RON-doh"), Harmer, Abbott,
  "nineteen ninety-nine".

## 09_net_flow: The crowd circulates (~52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | There's a test that decides any game like these, without finding where the crowd settles. It starts from how a settled crowd moves around the triangle. | The triangle alone, the mix's gold arrows (skeleton coins) | arrows | |
| 09_2 | Go back to the round numbers, and take the mix's crowd, scaled up to one thousand four hundred eighteen players so every count is whole. Across each edge, twelve more players go forward than come back. | Corners 490, 360, 568; edge labels forward/back: 147/135, 225/213, 355/343; each edge's net "12" in gold | labels from facts | |
| 09_3 | Every forward step wins a dollar and every backward step loses one. So how many dollars does this crowd gain each round? ⟨predict⟩ | Question; countdown | think countdown | |
| 09_4 | Thirty-six, twelve for each of the three edges. That's seven hundred twenty-seven winners against six hundred ninety-one losers. | TeX 3 × 12 = 36 = 727 − 691 | TeX from facts | |
| 09_5 | In any settled crowd, every edge carries the same net flow, and the gain per round is three times it. A game is fair exactly when that flow is zero. | "gain per round = 3 × net flow"; B's settled 26 return with net 0 on each edge (1 − 1, 3 − 3, 9 − 9) | TeX, labels from facts | |

### Notes for 09

- "Every edge carries the same net flow" holds for any settled crowd on the
  triangle (what enters a corner leaves it); the oracle checks it on 729
  games. 09_5's B check: 0→1 carries 1 forward and 1 back, 1→2 3 and 3,
  2→0 9 and 9.
- Pause intentions: after "come back" (09_2) a beat for the labels.

## 10_two_products: Two products decide (~79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | When is the net flow zero? Then every edge balances on its own, the players going forward across it equal to the players coming back. | Three balance lines, one per edge, in words then symbols: weight₀ · p₀ = weight₁ · q₁, weight₁ · p₁ = weight₂ · q₂, weight₂ · p₂ = weight₀ · q₀ (p forward, q back) | TeX derivation | |
| 10_2 | Multiply the three balances together. The weights appear once on each side and cancel. | The three lines multiply into one; each weight is struck out on both sides (TransformMatchingTex) | TransformMatchingTex | |
| 10_3 | What's left compares the two ways around. Winning three times in a row, all the way around the triangle, is exactly as likely as losing three times in a row the other way. | p₀ p₁ p₂ = q₀ q₁ q₂ with "win, win, win" and "lose, lose, lose" under each side | TeX | |
| 10_4 | For game B without the half point, a tenth times three quarters times three quarters, against nine tenths times a quarter times a quarter. Both come to nine over one hundred sixty. Balanced, so fair. | TeX 1/10 · 3/4 · 3/4 = 9/160 and 9/10 · 1/4 · 1/4 = 9/160 | TeX from facts | |
| 10_5 | For the mix, the forward product is thirty percent times sixty-two and a half percent twice over. The backward product is seventy percent times thirty-seven and a half percent twice over. Which is bigger? ⟨predict⟩ | TeX 0.3 · 0.625 · 0.625 vs 0.7 · 0.375 · 0.375, no values; countdown | think countdown | |
| 10_6 | Forward, seventy-five over six hundred forty against sixty-three. When the forward product is bigger, the crowd turns forward and the game wins. When it's smaller, the game loses. | 75/640 vs 63/640 in gold; a "precisely" card (K06) | lower-third | |
| 10_7 | With the half point shaved off, game B's forward product drops below its backward one, and the mix's stays above. Two products per game decide A, B and the mix. | A table, three rows (A, B, the mix, handicapped coins): forward and backward products to four digits and the verdict | table from facts | |

### Notes for 10

- K06: "precisely: average gain per round = 3 (p₀p₁p₂ − q₀q₁q₂) / W, where W
  is the sum over the corners of their spanning-tree weights, which is
  positive; every chance strictly between 0 and 1". The proof in 10_1 to 10_3
  shows that zero flow forces equal products; the formula gives the sign.
- The table (handicapped): A 0.1213 vs 0.1288 (loses), B 0.0527 vs 0.0588
  (loses), the mix 0.1134 vs 0.1018 (wins); values from facts.
- Pronunciation: "nine over one hundred sixty", "seventy-five over six
  hundred forty", "sixty-two and a half".

## 11_when_switching_fails: When switching fails (~69 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Switching helped here for one reason. Game B's coin reads your money, and game A's rounds move your money around. | Card B's "count your dollars" and A's triangle, side by side | cards | |
| 11_2 | Mix two ordinary coins and you get an ordinary coin with the average chance. Two losing ordinary coins always mix into a losing one. | A 49 % coin and a 48 % coin merge into a 48.5 % coin, red "loses" | coins from facts | |
| 11_3 | Even these two games don't win under every schedule. What happens if you strictly alternate, A, then B, then A, then B? ⟨predict⟩ | A strip "A B A B A B …"; question; countdown | think countdown | |
| 11_4 | It loses, about two thirds of a cent a round. The random mix wins, and the rigid alternation of the same two games loses. Two A's, then two B's, the pattern in the original paper, wins about one and a half cents. | Schedule table (handicapped coins): "random, half and half: +1.57", "A B A B …: −0.67", "A A B B …: +1.47" cents a round | table from facts | |
| 11_5 | The share of A matters too. With A on only one round in twenty, the mix loses. With A on nine rounds in ten, it loses again. The mix wins only when A's share is between about seven and eighty-five percent. | A curve: average per round against A's share from 0 to 100 %, zero line, crossings marked 7.0 % and 84.7 %, points at 5 % (−0.23) and 90 % (−0.34) | curve from facts | |
| 11_6 | So there is no general rule that switching between losing games wins. It works here, with these coins and these schedules, because one game reads the money and the other moves it. | The curve dims; the triangle with B's red corner 0 returns | callback | |

### Notes for 11

- The Nature caption's "in any sequence" is false; 11_4 shows the
  counterexample. At ε = 0, AB is exactly fair for every ρ (Ethier & Lee,
  Theorem 7); the handicap tips it to losing.
- "Two losing coins always mix into a losing coin": a mix of coins that do
  not depend on the money is a coin with the averaged chance; tested over a
  grid in the oracle.
- K07 (on 11_5): "handicapped coins; the mix wins exactly when A's share is
  strictly between 7.0 % and 84.7 % (rounded); AB −0.67, AABB +1.47 cents a
  round, per round played".
- Pause intentions: after "for one reason" (11_1) a short beat; the
  alternation verdict (11_4) gets a completed-thought pause before "Two A's".

## 12_what_winning_promises: What winning promises (~62 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Just over one and a half cents a round is an average over the very long run. Over a hundred rounds, luck is much bigger than that. | A histogram of the crowd's dollars after 100 rounds of the mix, from 0 at the start (exact distribution) | histogram | |
| 12_2 | Start a crowd at zero dollars and let everyone play the mix for a hundred rounds. Just over half finish ahead, and about two in five finish behind. | The histogram's right side gold "ahead 51.7 %", left red "behind 39.6 %", the middle bar dim "level 8.7 %"; a "precisely" card (K08) | histogram, labels from facts | |
| 12_3 | After a thousand rounds, what share of the crowd is ahead? ⟨predict⟩ | Question; countdown; the histogram dims | think countdown | |
| 12_4 | A little more than two in three. The average gain grows in step with the number of rounds, while the spread of luck grows only like its square root. | The histogram widens and slides right: "ahead 68.9 %" | histogram morph | |
| 12_5 | In the long run, a player's average per round settles on just over one and a half cents, with probability one. Over a thousand rounds of game A or game B alone, most players end up behind. | Two dim rows: "game A alone, 1,000 rounds: behind 61.2 %", "game B alone: behind 63.9 %"; a "precisely" card (K09) | labels from facts, lower-third | |
| 12_6 | These are invented games. What they teach is about chances that depend on where you stand, and none of it is advice about real money. | Everything fades but the triangle | | hold |

### Notes for 12

- K08: "start at 0 dollars (corner 0); exact distribution of the dollars after
  100 rounds of the mix with the published coins: ahead 51.7 %, behind
  39.6 %, level 8.7 %".
- K09: "with probability one the average per round converges to 223/14200
  dollars (strong law of large numbers for these games: Ethier & Lee,
  Electron. J. Probab. 14, 2009)".
- "settles on that one and a half cents": the limit is 1.57 cents; the voice
  says "that", pointing at the number already on screen.
- Pause intentions: a thinking beat after "much bigger than that" (12_1);
  a hold at the end (12_6).

## 13_the_catch: The catch (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Back to the start. Game A loses a cent a round, wherever you stand. | The two cards from 01 return; A's triangle with "49.5 %" everywhere | callback | |
| 13_2 | Game B loses because its crowd piles up at corner zero, where the bad coin waits and the good coin's wins keep returning it. | B's triangle; corner 0 at 38.4 %; the 0 ↔ 2 trace from 05 | callback | |
| 13_3 | Flip a fair coin between them, and game A's rounds keep pushing the crowd off corner zero. Game B's rounds, played from better corners, now win more than half the time. | The gold coin; corner 0 falls to 34.5 %; "B rounds: 52.1 %" | bars from facts | |
| 13_4 | So losing, for game B, depends on two things, its coins and where its crowd settles. A partner game can change the second. | Two labels under B's card: "its coins" (fixed) and "where its crowd settles" (moving) | labels | |
| 13_5 | One last check. For game A, with forty-nine and a half percent at every corner, which way round the triangle is more likely, three wins or three losses? ⟨predict⟩ | A's triangle; question; countdown | think countdown | |
| 13_6 | Three losses. Fifty and a half percent, three times over, beats forty-nine and a half, three times over. Game A loses, as the first minute showed. | 0.505³ vs 0.495³, values from facts; callback "−1 cent a round" | TeX from facts | |
| 13_7 | For any game drawn on this triangle, the verdict is that comparison. Win, win, win one way round against lose, lose, lose the other. Whichever is more likely is the way the money flows. | The forward and backward cycles drawn as two loops on the triangle, gold and dim | arrows | |
| 13_8 | To look further, search for Parrondo's paradox, Markov chains, and the flashing ratchet that Harmer and Abbott compared it to. | Lower-third "names to search: Parrondo's paradox · Markov chain · flashing ratchet"; receipts card: sources and the oracle | lower-third, receipts card | hold |

### Notes for 13

- The rigorous form of the opening picture is 13_7 with K06 behind it.
- Receipts card lines: "rules: Harmer & Abbott, Nature 402, 864 (1999)";
  "exact means and the long-run law: Ethier & Lee, EJP 14, 1827–1862 (2009)";
  "every number: computed in src/facts.rs and recomputed by oracle.py".
- Pause intentions: completed-thought pauses after 13_2 and 13_3; 13_7's
  three sentences are one thought with short gaps; final hold.
