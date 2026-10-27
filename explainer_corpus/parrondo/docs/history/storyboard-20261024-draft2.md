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

Second draft of 2026-10-24 (WildBass), eighteen chapters, after the owner's
direction of the same day: more setting, motivation and intuition before any
machinery, no first-minute derivation deadline. Draft 1 (thirteen chapters,
the rules and B's verdict inside the first minute) is kept unchanged at
`docs/history/storyboard-20261024-draft1.md`, with its script beside it.

**One player** carries Act I: a teal token with a dollar counter that starts
at **7 dollars**. The viewer learns each rule by watching that player toss,
count dollars in rows of three, and see the remainder pick B's coin. Three
computed experiments of the same player (100,000 rounds each, one seed fixed
in advance) make the puzzle concrete before any explanation: game A drifts
down; game B drifts down more slowly while the player stands on multiples of
three more than 38 % of the time; the fair-coin mix climbs, and the player
stands on multiples of three less often.

The player's remainder becomes **the triangle** (corners 0, 1, 2; a win moves
one corner forward, 0 → 1 → 2 → 0, a loss one back): the same teal token now
sits on a corner. Then **26 copies of the player** become **the crowd**, the
same teal dots, whose flows are conserved (the total stays 26) while the
share at corner 0, the bad coin's exposure, swings and steadies before it is
ever called a weight. The explanation uses the **round-number coins** (no
handicap; badge "round-number coins") where the crowd of 26 settles at **10,
4, 12** in whole numbers; the headline uses the **published coins** (badge
"published coins") and returns in chapter 13. The two are never on screen
unlabelled.

- Act I (01 to 05): the puzzle; one player learns game A, game B and the
  remainder; B looks like a winner and loses; the mix looks hopeless and
  wins; the player's exposure to the bad coin is the clue.
- Act II (06 to 10): the remainder as a place on a triangle; where you stand
  decides B; round-number coins; one player becomes a crowd; 26 players
  checked in whole numbers; the trap at corner 0 and a die-chosen control.
- Act III (11 to 13): the mix shifts the crowd; one dial explains the gain;
  the published coins return and the opening's numbers are earned.
- Act IV (14, 15): net flow around the triangle, and the two products that
  decide every game.
- Act V (16 to 18): when switching fails, what "wins" promises over finite
  play, and the catch, ending with a switching rule the viewer judges from
  the picture, which loses.

| Chapter | Spine | Status |
|---|---|---|
| `01_the_puzzle` | the puzzle; the player and game A; a few rounds climb; a cent a round (a prediction); 100,000 rounds drift down | source written 2026-10-24 (`the_puzzle.rs`), uncompiled, no render, no narration |
| `02_the_rule` | B's rule; rows of three and the remainder; which coin at ten dollars? (a prediction); seven, eight, nine | animatic |
| `03_does_b_win` | two counts in three; equal thirds say 52.8 %; does B win? (a prediction); the player's B run loses, 38 % at multiples of three | animatic |
| `04_looks_hopeless` | the mix; two losing ordinary coins mix to a loser; the averages say lose; does the mix win? (a prediction); the player's mix run climbs | animatic |
| `05_the_clue` | more or less time at multiples of three? (a prediction); 34.5 %; the puzzle stated exactly; title; the plan | animatic |
| `06_three_corners` | the player's remainder becomes a corner; where a win from corner 2 goes (a prediction); B and A as arrows | animatic |
| `07_where_you_stand` | six dollars or six hundred; A's share at corner 0 (a prediction); B's run says otherwise; round-number coins, labelled; equal thirds 53.3 % | animatic |
| `08_one_player_to_a_crowd` | 26 copies of the player; where the crowd is after one round (a prediction); conserved flows; the share at corner 0 swings and settles at 10, 4, 12 | animatic |
| `09_ten_four_twelve` | 10, 4, 12 checked arrow by arrow; arrivals (a prediction); 13 winners; 38.5 % at the bad coin | animatic |
| `10_the_trap` | where a player at corner 0 goes next (a prediction); the trap; a die-chosen coin wins 53.3 %; weights named | animatic |
| `11_mix_the_games` | the averaged arrows; which way the crowd shifts (a prediction); 38.5 → 34.6 % at corner 0 | animatic |
| `12_one_dial` | A rounds break even; B wins 75 % minus 65 points per unit of share at corner 0; at 34.6 % (a prediction) 52.5 %; the mix +2.54 cents | animatic |
| `13_the_half_point` | the published coins restored; B's threshold 37.7 % against its 38.4 %; does the mix survive? (a prediction); the opening's numbers; the source | animatic |
| `14_net_flow` | the mix's crowd of 1,418; twelve net across each edge; dollars gained per round (a prediction); gain = 3 × flow | animatic |
| `15_two_products` | balanced edges multiply to equal products; B balanced at 9/160; the mix's products (a prediction); the sign rule | animatic |
| `16_when_switching_fails` | ordinary coins mix to their average; strict alternation (a prediction) loses; AABB wins; A's share between 7.0 % and 84.7 % | animatic |
| `17_what_winning_promises` | the player's B run was ahead at 1,000 rounds; 100 rounds: 51.7 % ahead; 1,000 rounds (a prediction): 68.9 %; the long-run law; not advice | animatic |
| `18_the_catch` | the puzzle explained; a switching rule that reads the money the other way (a prediction) loses 27.6 cents a round; swapped, it wins; win-win-win against lose-lose-lose; names to search | animatic |

Every chapter's narration, render, native timing, listening and audiovisual
acceptance is open. Chapters 02 to 18 are registered as storyboard animatics
until their scenes are written.

## Vocabulary (one word per idea)

See `docs/brief.md`. In the voice: **the player**, **game A**, **game B**,
**the mix**, **round**, **bad coin**, **good coin**, **your dollars** / **the
count**, **divides by three**, **rows of three**, **remainder**, **corner**,
**forward**, **back**, **crowd**, **settle**, **weight**, **average per
round**, **net flow**, **forward product**, **backward product**, **the half
point**, **handicap**, **round-number coins** (the voice says "round
coins"), **published coins** (the voice says "the coins Harmer and Abbott
published"). Never in the voice: drift, expected value, stationary
distribution (screen only, as a name to search), strategy, half a cent (the
handicap is half a percentage point of chance; it costs game A one cent a
round).

## Color roles and stable objects

| Role | Color | Used for |
|---|---|---|
| game A | blue `#58C4DD` | card A, A's coin, A's arrows, A's runs and rounds |
| game B | violet `#C792EA` | card B, B's arrows, B's runs and rounds |
| the mix | gold `#F0AC5F` | the fair deciding coin, the mix's arrows, runs, bars and verdicts |
| bad coin / corner 0 | red `#FC6255` | the bad coin, corner 0's tint, the count with nothing left over |
| good coin | green `#83C167` | the good coin, wins, forward steps |
| the player and the crowd | teal `#5CD0B3` | the player's token and dollar chips, then the 26 (and 1,418) copies |
| highlight | hero `#FFD866` | questions, the countdown, the rule being pointed at |
| structure | ink / dim / faint | text, back arrows, frames, axes |

Stable objects: the player's token keeps its teal and its place on the left
third until it steps onto the triangle in 06; the triangle keeps corner 0 at
the top, 1 lower right, 2 lower left in every chapter; card A is always left
and card B right. **Coin badge**: from 01_2 on, a small badge at the top
right names the active coins. "published coins · 49.5 · 9.5 · 74.5 %" is
filled (ink on dim); "round-number coins · 50 · 10 · 75 % · no handicap" is
outlined in hero. The badge changes only on the voice's own cue (07_5,
13_1, 15_7) and with the round coins again at 14_1.

## Sources and scope

Harmer & Abbott, *Nature* 402, 864 (1999) for the rules; Ethier & Lee, *EJP*
14 (2009) for the strong law and the exact means the oracle reproduces
(18/709; AB 0; AABB 4/163; ABB 2416/35601; AAB 32/1609). The video makes no
claim about finance, investing, biology or casinos, and no historical claim
beyond the paper's own naming sentence. See `docs/brief.md`.

The one-player runs (01, 03, 04, 05, 07, 17) are computed experiments, not
data: a SplitMix64 generator with seed **1999**, fixed before any run was
looked at, start 7 dollars, 100,000 rounds; the mix draws its deciding coin
before each toss from the same generator. `src/model.rs` and `oracle.py`
each run them and must agree to the dollar. A different seed gives a
different path; the voice says only what the exact long-run numbers also
say, plus the luck it shows on purpose (01_5, 17_1).

## 01_the_puzzle: One player, one bent coin (~84 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Here's a puzzle about two games of chance. Each one, played on its own, slowly loses money. Can switching between them ever make money? | Two cards fade in, "Game A" (blue) left and "Game B" (violet) right, each with a small falling dim arrow "loses slowly". A gold curved arrow swings between them with a hero "?" | cards, question | |
| 01_2 | Before guessing, let's learn the games properly, with one player and real rounds. The player starts with seven dollars. | The cards shrink to the top. The player: a teal token on the left third, a row of seven teal dollar chips beside it, a live counter "7 dollars". The badge "published coins" appears top right | token, chips, live count | |
| 01_3 | Every round, the player tosses a coin. Heads wins a dollar, and tails loses one. | A neutral coin beside the token. Demonstration, labelled "for example": heads, "+1" in green, a chip slides in, "8 dollars"; tails, "−1" in dim, a chip slides out, "7 dollars" | coin, chips, live count | |
| 01_4 | Game A uses a single bent coin. It lands heads forty-nine and a half percent of the time, just under half. | The coin turns blue (card A glows). Under it "heads 49.5 % → +1" (green) and "tails 50.5 % → −1" (dim), from facts | text from facts | |
| 01_5 | Watch a few rounds. The money wanders up and down, and here it even climbs. A handful of tosses can't tell you whether a game wins or loses. | A small step plot to the right, "rounds" 0 to 8 against dollars: the first eight rounds of the player's game A run, drawn one toss at a time (7, 8, 9, 8, 9, 10, 11, 12, 11); the counter and chips follow | step plot from facts, live count | |
| 01_6 | So think in averages instead. In a hundred rounds you expect forty-nine and a half wins and fifty and a half losses. | The step plot dims. "100 rounds: 49.5 wins, 50.5 losses" | text from facts | |
| 01_7 | On average, how much does the player lose per round? ⟨predict⟩ | The question; countdown; no answer on screen | think countdown | |
| 01_8 | One dollar per hundred rounds, which is a cent a round. Tiny, but it never stops. | TeX "49.5 − 50.5 = −1 dollar per 100 rounds" then, in red, "−1 cent a round, on average" | TeX from facts | |
| 01_9 | Here's one simulated player, playing game A for a hundred thousand rounds. The path wobbles, but drifts down, about a thousand dollars below where it started. | Axes: rounds 0 to 100,000, dollars −1,200 to +200. The player's game A run in blue, drawn left to right; a dim dashed line falling one cent a round from 7; the end readout "−1,062 dollars after 100,000 rounds". A "precisely" card (K01) | plot from facts, live readout | |
| 01_10 | That's losing slowly. Luck decides any short stretch, and the cent a round decides the long run. | The first 1,000 rounds outlined, then the dashed line glows. Hold, then the plot slides to a thumbnail at the top left (blue, kept for 03 and 04) | highlight, thumbnail | hold |

### Notes for 01

- The opening puzzle is a question, not a derivation; nothing technical
  comes before the player exists. The first payoff, the cent a round, is the
  viewer's own computation at 01_7 and 01_8.
- K01 (01_9): "one simulated player: SplitMix64 seed 1999, fixed before any
  run was looked at; start 7 dollars; game A's exact long-run average is −1
  cent a round; this run ends 1,062 dollars down".
- 01_5's words are true of this run (its first eight rounds end above 7); the
  tests assert it, and the climb is the point: short runs mislead.
- 01_9: "about a thousand dollars below" is exact long-run arithmetic (one
  cent times 100,000), and this run is within 100 dollars of it; the tests
  assert both.
- Prediction (01_7): the TeX of 01_8 is not on screen before 01_8 starts.
- Pause intentions (for listening, not rules): after the question in 01_1 a
  real pause, long enough to guess; a short turn after "seven dollars"
  (01_2); a completed-thought pause after "wins or loses" (01_5); a breath
  before "Tiny, but it never stops" (01_8); the path draws in silence for a
  beat after "a hundred thousand rounds" (01_9) before "The path wobbles";
  hold at the end.
- Callbacks: A's thumbnail returns in 03_4 and 07_3; the cent a round in
  04_4, 13_2 and 18_1.
- Pronunciation to probe: "game A" (the letter, not the article),
  "forty-nine and a half", "fifty and a half", "a hundred thousand".

## 02_the_rule: The rule that reads your money (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Game B has two coins and a rule for choosing between them. Before every round, the player counts their dollars and asks one question. | Card B grows beside the player. Two coins inside it, red and green. The line "count your dollars" in hero under the token | coins, highlight | |
| 02_2 | Does the count divide by three? Arrange the dollars in rows of three and look at what's left over. | The player's seven chips slide into rows of three: two full rows and one chip alone, outlined | chips regroup | |
| 02_3 | Seven dollars make two rows with one left over. Eight leave two over. Nine fill three rows exactly, with nothing left over. That leftover is the remainder, and it's always zero, one or two. | Under the rows, "7 → remainder 1"; a chip is added, "8 → 2"; another, three full rows, "9 → 0" with the empty leftover slot in red; the word "remainder" | chips, lagged reveal | |
| 02_4 | When the remainder is zero, the player must toss the bad coin. It lands heads only nine and a half percent of the time. | "remainder 0 → bad coin: heads 9.5 %" in red, beside the red coin | text from facts | |
| 02_5 | Any other remainder, and the player tosses the good coin. It lands heads seventy-four and a half percent of the time. | "remainder 1 or 2 → good coin: heads 74.5 %" in green | text from facts | |
| 02_6 | Try one yourself. With ten dollars, which coin does the player toss? ⟨predict⟩ | Ten loose chips beside the token, not yet in rows; both coins dim; countdown | think countdown | |
| 02_7 | The good coin. Ten dollars make three rows with one left over, so the remainder is one. | The ten chips slide into three rows and one; "10 → remainder 1"; the green coin lights | chips regroup | |
| 02_8 | Now say the good coin wins twice in a row from seven. Seven becomes eight, and eight becomes nine. The count divides by three, and out comes the bad coin. | Demonstration, labelled "for example": from 7 the green coin lands heads, a chip joins (8, remainder 2); again (9, remainder 0); the rows close and the red coin slides to the token | coin, chips, live count | |

### Notes for 02

- The remainder is taught on the player's own chips before any triangle or
  notation; "remainder" is named only after three examples, then the viewer
  uses it once (02_6, 02_7).
- 02_8 is a demonstration (the voice says "say"); the coin results are not
  random draws and the picture labels them "for example".
- Pause intentions: "Does the count divide by three?" (02_2) is asked and
  answered at once, a short gap; a completed-thought pause after "zero, one or
  two" (02_3); "The good coin." (02_7) then a short beat while the chips
  regroup; after "out comes the bad coin" (02_8) a beat.
- Pronunciation: "nine and a half", "seventy-four and a half", "remainder",
  "their" (one player).

## 03_does_b_win: Does game B win? (~52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | So how often does the player meet the bad coin? Two counts in every three don't divide by three. | A strip of counts 6 to 14, every third one red (6, 9, 12), the others green | strip, lagged reveal | |
| 03_2 | If the good coin really got two rounds in three, game B would win fifty-two point eight percent of them. | Under card B: "if each remainder took a third of the rounds:" (dim) and TeX ⅓ · 9.5 % + ⅔ · 74.5 % = 52.8 %, "B would win" in green | TeX from facts | |
| 03_3 | So does game B win or lose in the long run? ⟨predict⟩ | The question; countdown; no verdict on screen | think countdown | |
| 03_4 | It loses. Here's the same player, a hundred thousand rounds of game B. The money drifts down again, though more slowly than in game A. | The axes of 01_9 return; A's run stays faint; B's run draws in violet; end readout "−548 dollars" | plot from facts, live readout | |
| 03_5 | And look at how often the bad coin came out. Not a third of the rounds, but more than thirty-eight percent. | A live readout grows with the run, "rounds at a multiple of 3", ending at "38.4 %" beside a dim marker "a third: 33.3 %" | live readout from facts | |
| 03_6 | Those extra rounds with the bad coin are what sink game B. Why they happen is the heart of this video. | The 38.4 % readout and card B's "count your dollars" both pulse hero; B's run slides to a thumbnail beside A's | highlight, thumbnail | hold |

### Notes for 03

- 03_4 to 03_5 are the seeded run (K01's generator). The voice claims only
  what is true of it and of the exact numbers: it ends below its start
  (−548), less far than A's run (−1,062), and its share of rounds at a
  multiple of three is above 38 % (38.37 %; the exact long-run share is
  38.4 %). The exact average, −0.87 cents a round, is shown in 04_4.
- Prediction (03_3): no path or share on screen before 03_4.
- Pause intentions: a thinking beat after the question in 03_1, though it is
  answered at once; after "fifty-two point eight percent of them" (03_2) the
  question follows closely; "It loses." (03_4) then a short beat before the
  run draws; hold at the end.
- Pronunciation: "fifty-two point eight", "thirty-eight".

## 04_looks_hopeless: Why switching looks hopeless (~64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Now the switching. Before every round, the player flips a fair coin. Heads means play game A, and tails means play game B. Call this the mix. | A gold coin above the token, faces "A" (blue) and "B" (violet); "the mix" in gold | coin | |
| 04_2 | Here's why the mix looks hopeless. Take two ordinary coins that both lose, one landing heads forty-nine percent of the time and one forty-eight. | Two plain coins, "49 %" and "48 %", both with a red "loses" | coins from facts | |
| 04_3 | Choosing between them with a fair coin works exactly like one coin that lands heads halfway between, forty-eight and a half percent. Two losers mix into a loser. | A gold coin picks one at random; they merge into one coin "48.5 %", red "loses" | coin merge from facts | |
| 04_4 | Averaging the games seems to agree. Game A loses a cent a round and game B a little under a cent, so halfway between should still be a loss. | Bars to scale about a zero line: "game A −1.00" (blue), "game B −0.87" (violet) cents a round; a dashed gold bar "halfway: −0.93 ?" | bars to scale from facts | |
| 04_5 | So when the player plays the mix, do they win or lose in the long run? ⟨predict⟩ | The question; countdown; the dashed bar waits | think countdown | |
| 04_6 | They win. Here's the same player, a hundred thousand rounds of the mix. The money climbs, ending well over a thousand dollars above where it started. | The axes return with A's and B's runs faint; the mix's run draws in gold, rising; end readout "+1,748 dollars" | plot from facts, live readout | hold |

### Notes for 04

- Negative controls, the attractive wrong answers: two ordinary losing coins
  always mix into a losing coin (oracle sweep), and the average of the two
  games' averages, −0.93 cents, is not the mix's average (+1.57). Both are
  shown before the reveal and both are computed.
- 04_6: the seeded run ends at +1,748 (exact long-run +1,570 over 100,000
  rounds). "Well over a thousand" is asserted by the tests.
- The bars are drawn to scale from the facts; the scene refuses to draw if A
  and B are not both below zero.
- Pause intentions: after "Call this the mix" (04_1) a beat; a completed
  thought after "Two losers mix into a loser" (04_3); "They win." (04_6) then
  a short beat; hold on the rising run.
- Pronunciation: "forty-eight and a half".

## 05_the_clue: The clue in the counts (~53 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Nothing about the coins changed, so something else did. In game B, the player stood on a multiple of three more than thirty-eight percent of the time. In the mix, more often or less? ⟨predict⟩ | B's readout "38.4 %" (violet) and the dim "33.3 %"; an empty gold readout; countdown | think countdown | |
| 05_2 | Less. In the mix, the player stood on a multiple of three only about thirty-four and a half percent of the time, much closer to a third. | The gold readout fills, run by run, to "34.5 %" between B's 38.4 % and the dim 33.3 % | live readout from facts | |
| 05_3 | That's the puzzle, and it's real. Two games that each lose, mixed by a fair coin, make a game that wins just over one and a half cents a round. | The dashed "−0.93 ?" bar of 04_4 is struck through; the mix's bar grows up to "+1.57". A "precisely" card (K02). Then all clear and the title card stands, "Two Losing Games Can Win" / "Here is the catch" | bars from facts, lower-third, title card | hold |
| 05_4 | The catch is hidden in game B's rule, the part that reads your money. To find it, we need a better picture of where the player stands, and then a crowd of players. | Card B returns to the centre; "count your dollars" outlined in hero and flashes. The player's token reappears beside it | highlight | |

### Notes for 05

- The clue is the exposure to the bad coin, measured on the player's own runs
  before any picture of it: 38.37 % in B, 34.52 % in the mix (exact 38.4 %
  and 34.5 %). "About thirty-four and a half" is asserted by the tests.
- K02 (05_3): "long-run average per round, exactly: A −1/100, B
  −73443/8446300, the mix +223/14200 dollars. Coins of Harmer & Abbott
  (Nature, 1999); the mix: a fair coin picks A or B before each round. These
  averages hold with probability one in the long run (Ethier & Lee, 2009)."
- Prediction (05_1): the mix's readout is empty until 05_2.
- Pause intentions: a longer turn before "That's the puzzle" (05_3); the title
  holds; 05_4 starts the explanation after a breath.

## 06_three_corners: The remainder becomes a place (~59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Game B only ever looks at the remainder, so let's track just that. Draw three corners, one for each remainder, zero, one and two. | The chips fold into a small "7 dollars" counter. A large triangle: corner 0 at the top, 1 lower right, 2 lower left, each labelled; corner 0 tinted red | triangle | |
| 06_2 | The player's seven dollars leave one over, so the player stands at corner one. A win moves them one corner forward, to corner two. | The teal token slides from the counter onto corner 1 (identity kept); a green forward arrow 1 → 2; the token steps, counter "8" | token moves | |
| 06_3 | From corner two, where does one more win take the player? ⟨predict⟩ | The token waits on corner 2; countdown; no arrow out of corner 2 yet | think countdown | |
| 06_4 | Around to corner zero. Eight dollars plus one is nine, and nine leaves nothing over. A loss always moves one corner back. | Green arrow 2 → 0, the token steps, counter "9"; then a dim back arrow 0 → 2, the token steps back, counter "8". All six arrows drawn: forward clockwise in green, back inside in dim | arrows, token moves | |
| 06_5 | Now write game B on the triangle. From corner zero, the bad coin, forward nine and a half percent of the time. From corners one and two, the good coin, forward seventy-four and a half percent. | Arrow labels from facts: 0→1 "9.5 %", 0→2 "90.5 %"; 1→2 and 2→0 "74.5 %", 1→0 and 2→1 "25.5 %" | labels from facts | |
| 06_6 | Game A is the same from every corner, forward forty-nine and a half percent, back fifty and a half. | A small blue copy of the triangle beside it, all forward arrows "49.5 %", all back arrows "50.5 %" | small multiple | hold |

### Notes for 06

- The token that steps onto corner 1 is the player of 01 to 05: same teal,
  same counter; the triangle is the player's remainder drawn as a place.
- Prediction (06_3) teaches the wrap: the arrow 2 → 0 is not drawn before
  06_4.
- Pause intentions: a short gap after "zero, one and two" (06_1); hold the
  triangle a beat after "one corner back" (06_4) so the token's last step is
  seen; hold at the end so both triangles can be compared.
- Pronunciation: "remainder", "ninety and a half" (screen only).

## 07_where_you_stand: Where you stand decides game B (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Whether you hold six dollars or six hundred, game B plays the same way at corner zero. So what decides game B is how often you stand at each corner. | "6" and "600" both slide into corner 0. A "precisely" card (K03) | lower-third | |
| 07_2 | If you play only game A for a long time, what share of the rounds do you spend at corner zero? ⟨predict⟩ | The question over A's triangle; countdown; no shares shown | think countdown | |
| 07_3 | One third. Game A treats every corner alike, so in the long run the corners share the rounds equally. The player's game A run spent thirty-three and a half percent there. | A's three corners each fill a bar to "1/3"; A's thumbnail with its readout "33.5 %" | bars, readout from facts | |
| 07_4 | Game B does not treat its corners alike. Its run spent more than thirty-eight percent at corner zero, and that's what we need to explain. | B's thumbnail with "38.4 %"; B's corner 0 pulses red | flash, readout | |
| 07_5 | To see why, make the numbers rounder for a while. Each coin in these games is a round percentage with half a point shaved off. Put the half point back. | The badge flips from "published coins" to the outlined "round-number coins · no handicap". Beside each coin 49.5 % → 50 %, 9.5 % → 10 %, 74.5 % → 75 %, each with a small "+0.5" that rises and fades | badge, count-up from facts | |
| 07_6 | Game A becomes a fair coin, so on average it neither wins nor loses. Game B gets a ten percent coin at corner zero and a seventy-five percent coin everywhere else. | Both triangles relabel: A's arrows "50 %"; B's 0→1 "10 %", 0→2 "90 %", the others "75 %" and "25 %" | labels from facts | |
| 07_7 | These round coins are our worked example, not the published games. With equal thirds at every corner, B would now win fifty-three point three percent of its rounds. | TeX ⅓ · 10 % + ⅔ · 75 % = 53.3 % (callback to 03_2), beside the badge | TeX from facts | hold |

### Notes for 07

- K03: "the capital is any whole number, negatives included, and play never
  stops; the remainder is 0, 1 or 2, so −1 leaves 2 and −3 leaves 0".
- Prediction (07_2) answered at 07_3's first words; the bars grow with them.
- The badge is the label of the explanatory example: outlined "round-number
  coins · no handicap" from 07_5 to 12_8 and again in 14; the published
  coins return, filled, at 13_1.
- Pause intentions: "So what decides game B" (07_1) follows without a long
  gap; a turn after "Put the half point back" (07_5); a thinking beat after
  "fifty-three point three percent of its rounds" (07_7), held.
- Pronunciation: "thirty-three and a half", "fifty-three point three".

## 08_one_player_to_a_crowd: One player becomes a crowd (~64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | So is it really a third each? One player's path is full of luck. To see past it, imagine twenty-six copies of the player, all starting at corner zero and all playing game B. | A's triangle leaves; the token on B's triangle splits into 26 teal dots gathered at corner 0, "26" beside it; a counter "round 0"; a fixed readout "players: 26" | dots, live count | |
| 08_2 | Let the crowd split exactly by the coins' chances. After one round, where is most of the crowd? ⟨predict⟩ | The dots become a bar at corner 0 (height ∝ players); countdown; nothing has moved | think countdown | |
| 08_3 | At corner two. A tenth of the crowd moves forward, and nine tenths fall back to corner two. | Round 1: 2.6 flow along 0 → 1, 23.4 along 0 → 2, drawn as moving streams on the arrows. A "precisely" card (K04) | bars, flows, live numbers | |
| 08_4 | The next round, three quarters of corner two win their way around to zero. Every player is always somewhere, so the total stays twenty-six while the crowd sloshes between corners. | Rounds 2 to 6 at one a second: (18.2, 5.9, 2.0), (2.9, 2.3, 20.8), (16.2, 5.5, 4.4), (4.6, 2.7, 18.7), (14.7, 5.1, 6.2); streams on the arrows each round; "players: 26" never changes | bars, flows, live numbers | |
| 08_5 | Watch the share at corner zero, where the bad coin waits. It swings wildly at first, then steadies above a third. | A live readout "share at corner 0" and a small trace of it against rounds, with a dashed line at 33.3 % | live readout, trace | |
| 08_6 | Round after round the sloshing dies down, and the crowd settles at ten players on corner zero, four on corner one and twelve on corner two. Not a third each. | Rounds 7 to 30 quicken; the bars converge; "10", "4", "12" settle in teal; the share trace flattens at 38.5 %; small dim "within half a player from round 24" | bars, live numbers | hold |

### Notes for 08

- K04: "the crowd splits by expected numbers: 26 times each corner's chance.
  Real players scatter around these numbers."
- Conserved flows before weights: the total stays 26 every round (the facts
  check it); the corner 0 share is shown as a changing quantity here and is
  named "weight" only at 10_8.
- The bars show one decimal; 26 dots return in 09 for the whole-number check.
- Prediction (08_2): no flow before 08_3.
- Pause intentions: a held breath on "imagine twenty-six copies" (08_1) as the
  token splits; "At corner two." (08_3) then the streams; a listen-quiet hold
  after "Not a third each" (08_6) so the settling is seen (hold).

## 09_ten_four_twelve: A crowd that stays put (~71 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Check that this crowd really stays put. At corner zero, ten players toss the bad coin. One wins and moves forward. Nine lose and drop back to corner two. | The bars become 26 dots again: 10 at corner 0, 4 at 1, 12 at 2. Corner 0's dots split: one beside the forward arrow (green "1"), nine beside the back arrow (dim "9"); they wait at the arrow's start | dots, labels from facts | |
| 09_2 | At corner one, four players toss the good coin. Three win and move on to corner two, and one falls back to zero. | Corner 1's dots split: "3" forward, "1" back | dots | |
| 09_3 | At corner two, twelve players. Nine win and wrap around to zero, and three fall back to one. | Corner 2's dots split: "9" forward, "3" back | dots | |
| 09_4 | Now let everyone move at once. How many players land on each corner? ⟨predict⟩ | The question on screen; countdown; the dots still wait | think countdown | |
| 09_5 | Corner zero gets one plus nine, ten. Corner one gets one plus three, four. Corner two gets nine plus three, twelve. Every player moved, and the crowd looks exactly the same. | All 26 dots move along their arrows together; arrival sums appear at each corner "1 + 9 = 10", "1 + 3 = 4", "9 + 3 = 12" | swarm move, TeX from facts | |
| 09_6 | Count the winners. One, three and nine make thirteen, exactly half of twenty-six. Without the half point, game B is perfectly fair. | The 13 forward movers flash green; "13 of 26 win = 1/2" | flash | |
| 09_7 | Now look at where the crowd sits. Ten of twenty-six is thirty-eight and a half percent, more than a third. | Corner 0's 10 dots outlined red; a bar "38.5 %" beside a dim marker at "33.3 %" | bar from facts | |
| 09_8 | Those extra rounds at corner zero, with the bad coin, cancel the good coin's advantage exactly. | The equal-thirds count from 07_7 (53.3 %) slides down to "50 %" as the share moves from 33.3 % to 38.5 % | tween | |

### Notes for 09

- Prediction (09_4): the arrivals are not summed on screen before 09_5.
- Facts: the flows are whole players (26 is the smallest crowd for which
  they are; the oracle searches every smaller crowd).
- Arrival sums are listed by the corner they come from, lowest first.
- Lint disposition: 09_5's three sentences opening "Corner …" are the count
  itself, said in the order the arrivals appear; kept on purpose.
- Pause intentions: each corner's split is its own thought (09_1 to 09_3),
  with short completed-thought gaps; after "looks exactly the same" (09_5) a
  longer hold so the unchanged counts register; "perfectly fair" (09_6) ends
  a section.

## 10_the_trap: Why corner zero collects players (~81 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Why does corner zero collect extra players? Follow one who has just arrived there. Where is that player most likely to be one round later? ⟨predict⟩ | The crowd dims; one teal dot at corner 0, outlined. The question on screen; countdown | think countdown | |
| 10_2 | At corner two, nine times in ten. The bad coin knocks them back. And from corner two, the good coin usually wins, which carries them forward, around to zero again. | The dot drops to corner 2 ("90 %"), then wins back to 0 ("75 %"), tracing a red-and-green path | path | |
| 10_3 | So most of the good coin's wins at corner two just bring the player back to zero, where the bad coin waits. | The 0 ↔ 2 path repeats three times, quicker, its trace thickening | path | |
| 10_4 | That's the trap in game B. The good coin wins often, but its wins keep getting spent on climbing back to a multiple of three. | The thick 0 ↔ 2 trace glows; corner 0's red tint deepens | glow | |
| 10_5 | One change removes the trap. Let a die choose B's coin instead of your money. A one or a two means the bad coin, anything else the good coin. | A die beside the two coins; faces 1, 2 point to the red coin, 3 to 6 to the green coin. The triangle dims: the corners no longer matter | die, arrows | |
| 10_6 | Now the bad coin really does get one round in three, wherever you stand, and this game wins fifty-three point three percent of the time. | TeX ⅓ · 10 % + ⅔ · 75 % = 53.3 % (callback), "wins" in green | TeX from facts | |
| 10_7 | Same two coins. Only the rule that picks between them changed. Reading your money is what drags game B from fifty-three point three percent down to exactly half. | The die fades; the triangle returns, "count your dollars" outlined; "53.3 % → 50 %" beside it | highlight | |
| 10_8 | Each corner's long-run share of the rounds is called its weight. For game B, the weights are five thirteenths, two thirteenths and six thirteenths. | At the corners: 5/13, 2/13, 6/13 (= 10, 4, 12 of 26). Lower-third: "names to search: stationary distribution · Markov chain" | lower-third, labels from facts | |

### Notes for 10

- The die-chosen game is the negative control for "B loses because of where
  you stand": the same coins with the choice made independently of the money
  give an ordinary coin that wins 8/15 = 53.3 % with the round-number coins
  (317/600 = 52.8 % with the published ones), so it wins.
- Pause intentions: after "nine times in ten" (10_2) a short beat; the trap
  sentence (10_4) is a completed thought with a pause after it; "One change
  removes the trap" (10_5) starts a new idea after a turn.
- Pronunciation: "thirteenths" (watch for "thirtieths").

## 11_mix_the_games: Mix the games (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Now mix the games, as the player did. Before every round, a fair coin decides whether you play A or B. | The gold coin of 04_1 above the triangle, faces "A" and "B"; the badge still "round-number coins" | coin | |
| 11_2 | On the triangle, every arrow becomes an average. At corner zero, half the time it's A's fifty percent, half the time B's ten percent. So forward thirty percent in all. | Corner 0's forward arrow: blue "50 %" and violet "10 %" merge into gold "30 %"; the back arrow "70 %" | label morph from facts | |
| 11_3 | At corners one and two, halfway between fifty and seventy-five, forward sixty-two and a half percent. | The other forward arrows morph to gold "62.5 %", back "37.5 %" | label morph | |
| 11_4 | Start the mix from game B's settled crowd, ten, four and twelve. Which way will the crowd shift? ⟨predict⟩ | B's settled bars (10, 4, 12) return; question; countdown | think countdown | |
| 11_5 | Off corner zero. Round by round, corner zero's share falls from thirty-eight point five percent to thirty-four point six, and corner one's climbs from fifteen point four to twenty-five point four. | The bars flow under the mix's arrows; live shares: corner 0 38.5 % → 34.6 %, corner 1 15.4 % → 25.4 %, corner 2 46.2 % → 40.1 %; a ghost outline keeps B's old heights | bars, live numbers | |
| 11_6 | Game A's coin ignores the corners, so its rounds keep stirring the crowd toward equal thirds. That shift of weight off corner zero is where the winning comes from. | A dim dashed line at 33.3 % on each bar; corner 0's lost weight glows | glow | hold |

### Notes for 11

- The shift is the reveal: keep B's ghost heights visible beside the mix's
  bars through 11_6. The player's own mix run (05_2, 34.5 % with the
  published coins) is the same effect seen in one player.
- Pause intentions: after "every arrow becomes an average" (11_2) a short
  beat; after the shares (11_5) a held pause so the numbers can be read; a
  hold after 11_6 (the reveal).

## 12_one_dial: One dial decides game B (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Split the mix's rounds into A rounds and B rounds. The A rounds win exactly half the time, at every corner, so they can't be where the winning comes from. | The mix's rounds as a strip of blue and violet tiles; the blue tiles show "50 %" | tiles | |
| 12_2 | Game B's chance of winning depends on just one number, its share of rounds at corner zero. Every round it spends there swaps a seventy-five percent coin for a ten percent one. | A plot: horizontal axis "B's share of rounds at corner 0" from 30 % to 40 %, vertical "B wins" from 48 % to 56 %; the falling line 75 % − 65 % × share | plot, axis | |
| 12_3 | At a third, B wins fifty-three point three percent. At game B's own thirty-eight point five, exactly half. | Two points on the line: (33.3 %, 53.3 %) green, (38.5 %, 50 %) violet | plot points from facts | |
| 12_4 | Inside the mix, B's rounds spend thirty-four point six percent at corner zero. So what do they win? ⟨predict⟩ | A gold vertical marker at 34.6 %; question; countdown; no value on the line | think countdown | |
| 12_5 | Fifty-two point five percent. Averaged with the A rounds' fifty, the mix wins fifty-one point three percent of all its rounds. | The marker meets the line at 52.5 %; TeX ½ · 50 % + ½ · 52.54 % = 51.27 % (two decimals, so the sum checks) | TeX from facts | |
| 12_6 | On average, that's a gain of just over two and a half cents a round. | "+2.54 cents a round" in gold; a "precisely" card (K05) | lower-third | |
| 12_7 | So game A's rounds only break even. They move the crowd off the bad corner, and game B's good coin does the winning. | The blue tiles dim; the violet tiles glow | glow | |
| 12_8 | Remember the first count, with a third of the rounds at each corner? The mix pushes the crowd most of the way back to it. | On the plot, an arrow from 38.5 % toward 33.3 % stopping at 34.6 %; "76 % of the way" | arrow from facts | |

### Notes for 12

- K05: "exactly: a crowd of 1,418 at 490, 360, 568 under the mix sends 147,
  225, 355 forward and 343, 135, 213 back; every corner gets back what it
  sends; 727 win, 691 lose: 18/709 dollars a round (Ethier & Lee 2009 give
  18/709)".
- The line is exact: B wins 3/4 − (13/20)·share, so B is fair at share 5/13.
- Prediction (12_4): the answer is between 50 % and 53.3 %, nearer 53.3 %.
- Pause intentions: completed-thought pause after "can't be where the
  winning comes from" (12_1); a thinking beat after the two points (12_3);
  the answer (12_5) begins promptly after the countdown.

## 13_the_half_point: The half point returns (~76 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Now shave the half point off every coin again, back to the coins Harmer and Abbott published, the ones the player used. | The badge flips back to the filled "published coins". 50 % → 49.5 %, 10 % → 9.5 %, 75 % → 74.5 %, each with "−0.5" | badge, count-down from facts | |
| 13_2 | Game A slips from fair to losing a cent a round. | Row "game A: 0 → −1.00 cents a round" | live number | |
| 13_3 | Game B's crowd barely moves. Corner zero still holds thirty-eight point four percent of its rounds, and with the weaker coins that's now too many. B loses a little under a cent a round. | The dial's line drops by half a point; its fair point moves from 38.5 % to 37.7 %; B's point at 38.4 % sits past it; "game B: −0.87" | plot from facts | |
| 13_4 | The mix had an edge of just over two and a half cents a round. Does it survive the handicap? ⟨predict⟩ | "the mix: +2.54" with a question mark; countdown | think countdown | |
| 13_5 | It does. The half point costs the mix about one cent a round, and the edge was bigger than that. The mix still wins just over one and a half cents a round. | "+2.54 − 0.97 = +1.57 cents a round" in gold | TeX from facts | |
| 13_6 | Inside the mix, corner zero still holds only thirty-four and a half percent, so B's rounds still win fifty-two point one percent. | The gold marker on the lowered line at 34.5 % meets it at 52.1 %; the player's mix-run readout "34.5 %" (05_2) beside it | plot from facts | |
| 13_7 | That's every number from the opening. A loses a cent a round, B a little under a cent, and the coin-flip mix of the two wins. | The bars of 05_3 return exactly (callback), and the three run thumbnails beside them | bars from facts | |
| 13_8 | Harmer and Abbott named the effect after its discoverer, Juan Parrondo. Their paper, in Nature in nineteen ninety-nine, is a single page. | Citation card: "G. P. Harmer & D. Abbott, Losing strategies can win by Parrondo's paradox, Nature 402, 864 (1999)" | citation card | hold |

### Notes for 13

- The published numbers: B's weights 38.4 %, 15.4 %, 46.2 %; B's fair
  share threshold 49/130 = 37.7 %; the mix's weights 34.5 %, 25.4 %, 40.1 %;
  B rounds inside the mix 52.1 %; the mix's win 50.8 %.
- History: only the paper's own sentence ("called Parrondo's paradox, after
  its discoverer, Juan Parrondo") and its date and length. No year of
  discovery, no affiliation.
- Pause intentions: "It does." (13_5) is followed by a short beat; the
  citation (13_8) holds.
- Pronunciation: Juan Parrondo ("hwahn pah-RON-doh"), Harmer, Abbott,
  "nineteen ninety-nine".

## 14_net_flow: The crowd circulates (~52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | There's a test that decides any game like these, without finding where the crowd settles. It starts from how a settled crowd moves around the triangle. | The triangle alone, the mix's gold arrows; the badge flips to "round-number coins" | arrows, badge | |
| 14_2 | Go back to the round numbers, and take the mix's crowd, scaled up to one thousand four hundred eighteen players so every count is whole. Across each edge, twelve more players go forward than come back. | Corners 490, 360, 568; edge labels forward/back: 147/135, 225/213, 355/343; each edge's net "12" in gold | labels from facts | |
| 14_3 | Every forward step wins a dollar and every backward step loses one. So how many dollars does this crowd gain each round? ⟨predict⟩ | Question; countdown | think countdown | |
| 14_4 | Thirty-six, twelve for each of the three edges. That's seven hundred twenty-seven winners against six hundred ninety-one losers. | TeX 3 × 12 = 36 = 727 − 691 | TeX from facts | |
| 14_5 | In any settled crowd, every edge carries the same net flow, and the gain per round is three times it. A game is fair exactly when that flow is zero. | "gain per round = 3 × net flow"; B's settled 26 return with net 0 on each edge (1 − 1, 3 − 3, 9 − 9) | TeX, labels from facts | |

### Notes for 14

- "Every edge carries the same net flow" holds for any settled crowd on the
  triangle (what enters a corner leaves it); the oracle checks it on 729
  games. 14_5's B check: 0→1 carries 1 forward and 1 back, 1→2 3 and 3,
  2→0 9 and 9.
- Pause intentions: after "come back" (14_2) a beat for the labels.

## 15_two_products: Two products decide (~79 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | When is the net flow zero? Then every edge balances on its own, the players going forward across it equal to the players coming back. | Three balance lines, one per edge, in words then symbols: weight₀ · p₀ = weight₁ · q₁, weight₁ · p₁ = weight₂ · q₂, weight₂ · p₂ = weight₀ · q₀ (p forward, q back) | TeX derivation | |
| 15_2 | Multiply the three balances together. The weights appear once on each side and cancel. | The three lines multiply into one; each weight is struck out on both sides (TransformMatchingTex) | TransformMatchingTex | |
| 15_3 | What's left compares the two ways around. Winning three times in a row, all the way around the triangle, is exactly as likely as losing three times in a row the other way. | p₀ p₁ p₂ = q₀ q₁ q₂ with "win, win, win" and "lose, lose, lose" under each side | TeX | |
| 15_4 | For game B without the half point, a tenth times three quarters times three quarters, against nine tenths times a quarter times a quarter. Both come to nine over one hundred sixty. Balanced, so fair. | TeX 1/10 · 3/4 · 3/4 = 9/160 and 9/10 · 1/4 · 1/4 = 9/160 | TeX from facts | |
| 15_5 | For the mix, the forward product is thirty percent times sixty-two and a half percent twice over. The backward product is seventy percent times thirty-seven and a half percent twice over. Which is bigger? ⟨predict⟩ | TeX 0.3 · 0.625 · 0.625 vs 0.7 · 0.375 · 0.375, no values; countdown | think countdown | |
| 15_6 | Forward, seventy-five over six hundred forty against sixty-three. When the forward product is bigger, the crowd turns forward and the game wins. When it's smaller, the game loses. | 75/640 vs 63/640 in gold; a "precisely" card (K06) | lower-third | |
| 15_7 | With the half point shaved off, game B's forward product drops below its backward one, and the mix's stays above. Two products per game decide A, B and the mix. | The badge flips to "published coins". A table, three rows (A, B, the mix): forward and backward products to four digits and the verdict | badge, table from facts | |

### Notes for 15

- K06: "precisely: average gain per round = 3 (p₀p₁p₂ − q₀q₁q₂) / W, where W
  is the sum over the corners of their spanning-tree weights, which is
  positive; every chance strictly between 0 and 1". The proof in 15_1 to 15_3
  shows that zero flow forces equal products; the formula gives the sign.
- The table (published coins): A 0.1213 vs 0.1288 (loses), B 0.0527 vs 0.0588
  (loses), the mix 0.1134 vs 0.1018 (wins); values from facts.
- Pronunciation: "nine over one hundred sixty", "seventy-five over six
  hundred forty", "sixty-two and a half".

## 16_when_switching_fails: When switching fails (~69 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | Switching helped here for one reason. Game B's coin reads your money, and game A's rounds move your money around. | Card B's "count your dollars" and A's triangle, side by side; the badge "published coins" | cards | |
| 16_2 | Remember the two ordinary coins. Mix two losing ordinary coins and you always get a losing coin, with the average chance. | The 49 % and 48 % coins of 04_2 merge again into 48.5 %, red "loses" (callback) | coins from facts | |
| 16_3 | Even these two games don't win under every schedule. What happens if you strictly alternate, A, then B, then A, then B? ⟨predict⟩ | A strip "A B A B A B …"; question; countdown | think countdown | |
| 16_4 | It loses, about two thirds of a cent a round. The random mix wins, and the rigid alternation of the same two games loses. Two A's, then two B's, the pattern in the original paper, wins about one and a half cents. | Schedule table (published coins): "random, half and half: +1.57", "A B A B …: −0.67", "A A B B …: +1.47" cents a round | table from facts | |
| 16_5 | The share of A matters too. With A on only one round in twenty, the mix loses. With A on nine rounds in ten, it loses again. The mix wins only when A's share is between about seven and eighty-five percent. | A curve: average per round against A's share from 0 to 100 %, zero line, crossings marked 7.0 % and 84.7 %, points at 5 % (−0.23) and 90 % (−0.34). A "precisely" card (K07) | curve from facts | |
| 16_6 | So there is no general rule that switching between losing games wins. It works here, with these coins and these schedules, because one game reads the money and the other moves it. | The curve dims; the triangle with B's red corner 0 returns | callback | |

### Notes for 16

- The Nature caption's "in any sequence" is false; 16_4 shows the
  counterexample. With the round-number coins, AB is exactly fair for every
  ρ (Ethier & Lee, Theorem 7); the handicap tips it to losing.
- "Two losing coins always mix into a losing coin": a mix of coins that do
  not depend on the money is a coin with the averaged chance; tested over a
  grid in the oracle.
- K07 (16_5): "published coins; the mix wins exactly when A's share is
  strictly between 7.0 % and 84.7 % (rounded); AB −0.67, AABB +1.47 cents a
  round, per round played".
- Pause intentions: after "for one reason" (16_1) a short beat; the
  alternation verdict (16_4) gets a completed-thought pause before "Two A's".

## 17_what_winning_promises: What winning promises (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 17_1 | Look back at the player's run of game B. After a thousand rounds it was sixteen dollars ahead, even though game B loses. | B's thumbnail grows; its first 1,000 rounds outlined; readout "+16 dollars after 1,000 rounds" | plot from facts, highlight | |
| 17_2 | Just over one and a half cents a round is an average over the very long run. Over a hundred rounds, luck is much bigger than that. | A histogram of the crowd's dollars after 100 rounds of the mix, from 0 at the start (exact distribution) | histogram | |
| 17_3 | Start a crowd at zero dollars and let everyone play the mix for a hundred rounds. Just over half finish ahead, and about two in five finish behind. | The histogram's right side gold "ahead 51.7 %", left red "behind 39.6 %", the middle bar dim "level 8.7 %"; a "precisely" card (K08) | histogram, labels from facts | |
| 17_4 | After a thousand rounds, what share of the crowd is ahead? ⟨predict⟩ | Question; countdown; the histogram dims | think countdown | |
| 17_5 | A little more than two in three. The average gain grows in step with the number of rounds, while the spread of luck grows only like its square root. | The histogram widens and slides right: "ahead 68.9 %" | histogram morph | |
| 17_6 | In the long run, a player's average per round settles on just over one and a half cents, with probability one. Over a thousand rounds of game A or game B alone, most players end up behind. | Two dim rows: "game A alone, 1,000 rounds: behind 61.2 %", "game B alone: behind 63.9 %"; a "precisely" card (K09) | labels from facts, lower-third | |
| 17_7 | These are invented games. What they teach is about chances that depend on where you stand, and none of it is advice about real money. | Everything fades but the triangle | | hold |

### Notes for 17

- 17_1 is the seeded run's own luck (+16 after 1,000 rounds of B; it ends
  −548 after 100,000). The tests assert the +16.
- K08: "start at 0 dollars (corner 0); exact distribution of the dollars after
  100 rounds of the mix with the published coins: ahead 51.7 %, behind
  39.6 %, level 8.7 %".
- K09: "with probability one the average per round converges to 223/14200
  dollars (strong law of large numbers for these games: Ethier & Lee,
  Electron. J. Probab. 14, 2009)".
- Pause intentions: a beat after "sixteen dollars ahead" (17_1) before "even
  though game B loses"; a thinking beat after "much bigger than that" (17_2);
  a hold at the end (17_7).

## 18_the_catch: The catch (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 18_1 | Back to the puzzle. Game A loses a cent a round, wherever the player stands. | The two cards of 01 return with the player's token; A's triangle with "49.5 %" everywhere | callback | |
| 18_2 | Game B loses because its crowd piles up at corner zero, where the bad coin waits. | B's triangle; corner 0 at 38.4 %; the 0 ↔ 2 trace from 10 | callback | |
| 18_3 | Flip a fair coin between them, and game A's rounds push the crowd off corner zero. Game B's rounds, played from better corners, now win more than half the time. | The gold coin; corner 0 falls to 34.5 %; "B rounds: 52.1 %" | bars from facts | |
| 18_4 | One last puzzle, to solve from the picture. Play game B only when the count divides by three, and game A at every other count. | A new card, "B when the count divides by 3, A otherwise"; the triangle relabels: 0→1 "9.5 %" (red), 1→2 and 2→0 "49.5 %" (blue) | card, labels from facts | |
| 18_5 | Does this way of switching win or lose? ⟨predict⟩ | Question; countdown | think countdown | |
| 18_6 | It loses, and badly, about twenty-eight cents a round. Every coin around the triangle now favors going back. | "−27.6 cents a round" in red; the forward and backward cycles as two loops, the backward one thick | loops, number from facts | |
| 18_7 | Swap the roles, A at corner zero and B everywhere else, and the same two games win about thirty-one cents a round. What decides it is where each coin gets tossed. | The swapped card; 0→1 "49.5 %" (blue), the others "74.5 %" (green); "+31.4 cents a round" in gold | card, labels from facts | |
| 18_8 | For any game drawn on this triangle, the verdict is that comparison. Win, win, win one way round against lose, lose, lose the other. Whichever is more likely is the way the money flows. | The forward and backward cycles drawn as two loops on the triangle, gold and dim | arrows | |
| 18_9 | To look further, search for Parrondo's paradox, Markov chains, and the flashing ratchet that Harmer and Abbott compared it to. | Lower-third "names to search: Parrondo's paradox · Markov chain · flashing ratchet"; receipts card: sources and the oracle | lower-third, receipts card | hold |

### Notes for 18

- The final transfer question (18_4 to 18_6) is answerable from the
  picture alone: every coin in that switching rule is below one half (9.5 %
  and 49.5 %), so the forward product is smaller. Exact: −2490243/9016300
  dollars a round (−27.6 cents). The swapped rule wins +2902797/9230300
  (+31.4 cents). Both switch between the same games A and B; the pair shows
  the condition, not a universal claim.
- The rigorous form of the opening picture is 18_8 with K06 behind it.
- Receipts card lines: "rules: Harmer & Abbott, Nature 402, 864 (1999)";
  "exact means and the long-run law: Ethier & Lee, EJP 14, 1827–1862 (2009)";
  "every number: computed in src/facts.rs and recomputed by oracle.py".
- Pause intentions: completed-thought pauses after 18_2 and 18_3; a turn
  before "One last puzzle" (18_4); "It loses, and badly" (18_6) then a short
  beat; 18_8's three sentences are one thought with short gaps; final hold.
- Pronunciation: "twenty-eight", "thirty-one", "favors".
