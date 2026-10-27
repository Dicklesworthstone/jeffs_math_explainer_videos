# Storyboard, full draft — Two Losing Games Can Win — Here Is the Catch

The whole planned film: every chapter's words and pictures. Production
follows `docs/storyboard.md`, which holds only the chapters whose native
scenes are written (today 01, identical there) and from which
`python3 -I tools/storyboard.py parrondo` writes `script.tsv`. A chapter
moves there, words unchanged, when root assigns and its scene is written.
The tools and the crate never read this file. Every number below is a fact
computed in `src/facts.rs` and recomputed another way by `oracle.py`.

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
`docs/history/storyboard-20261024-draft1.md`, with its script beside it;
this draft as it stood before root's scope correction (Mail 6790), when
02 to 18 were registered as animatics, is kept unchanged at
`docs/history/storyboard-20261024-draft2.md`.

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
| `01_the_puzzle` | the puzzle; the player and game A; a few rounds climb; a cent a round (a prediction); 100,000 rounds drift down | native source written 2026-10-24 (`the_puzzle.rs`), uncompiled, no render, no narration |
| `02_the_rule` | B's rule; rows of three and the remainder; which coin at ten dollars? (a prediction); seven, eight, nine | planned (words and pictures only) |
| `03_does_b_win` | two counts in three; equal thirds say 52.8 %; does B win? (a prediction); the player's B run loses, 38 % at multiples of three | planned (words and pictures only) |
| `04_looks_hopeless` | the mix; two losing ordinary coins mix to a loser; the averages say lose; does the mix win? (a prediction); the player's mix run climbs | integrated source; native/frame qualification pending |
| `05_the_clue` | more or less time at multiples of three? (a prediction); 34.5 %; the puzzle stated exactly; title; the plan | integrated source; native/frame qualification pending |
| `06_three_corners` | the player's remainder becomes a corner; where a win from corner 2 goes (a prediction); B and A as arrows | planned (words and pictures only) |
| `07_where_you_stand` | six dollars or six hundred; A's share at corner 0 (a prediction); B's run says otherwise; round-number coins, labelled; equal thirds 53.3 % | planned (words and pictures only) |
| `08_one_player_to_a_crowd` | 26 copies of the player; where the crowd is after one round (a prediction); conserved flows; the share at corner 0 swings and settles at 10, 4, 12; a seeded real crowd jumps around those numbers | planned (words and pictures only) |
| `09_ten_four_twelve` | 10, 4, 12 checked arrow by arrow; arrivals (a prediction); 13 winners; 38.5 % at the bad coin; weights named | planned (words and pictures only) |
| `10_the_trap` | where a player at corner 0 goes next (a prediction); the trap; a die-chosen coin wins 53.3 % | planned (words and pictures only) |
| `11_mix_the_games` | the averaged arrows; which way the crowd shifts (a prediction); 38.5 → 34.6 % at corner 0 | planned (words and pictures only) |
| `12_one_dial` | A rounds break even; B wins 75 % minus 65 points per unit of share at corner 0; at 34.6 % (a prediction) 52.5 %; the mix +2.54 cents | planned (words and pictures only) |
| `13_the_half_point` | the published coins restored; B's threshold 37.7 % against its 38.4 %; does the mix survive? (a prediction); the opening's numbers; the source | planned (words and pictures only) |
| `14_net_flow` | a test from the coins alone?; the mix's crowd of 1,418 and its paired flows; the viewer counts each edge's surplus, twelve; dollars gained per round (a prediction); why a settled corner forces the same twelve; one seeded real round against the expected numbers; B's balances multiply to equal products | native source written 2026-10-24 (`net_flow.rs`), uncompiled, no render, no narration |
| `15_two_products` | the converse built on B's 10, 4, 12, then in letters; the sign from one line with both other factors positive; B fair, not a winner; the mix's products (a prediction); the one line on the 1,418; the published A, B and mix | native source written 2026-10-24 (`two_products.rs`), uncompiled, no render, no narration |
| `16_when_switching_fails` | does the order matter?; strict alternation (a prediction) loses; the crowd played round by round from B's settled crowd, the dots climbing past B's break-even line; one cycle's refill of corner 0; the averaged-chances shortcut fails; AABB stays below the line; the table; ordinary coins in any order; A's share: the curve, rounded crossings against the exact strict bounds | native source written 2026-10-24 (`when_switching_fails.rs`), uncompiled, no render, no narration |
| `17_what_winning_promises` | the player's +16 after 1,000 rounds of B, −548 after 100,000; what the exact 100-round distribution is (start at 0, negative dollars, even totals, one bar per total); ahead, behind, level; the crowd's average and the start's cost; 1,000 rounds (a prediction): 68.9 %, the same bars on the same axis; the spread and the square root as a long-run law; B from 7: lucky, not rare; A and B alone; the law with probability one; three meanings; not advice | native source written 2026-10-24 (`what_winning_promises.rs`), uncompiled, no render, no narration |
| `18_the_catch` | the puzzle explained; a switching rule that reads the money the other way (a prediction) loses 27.6 cents a round; swapped, it wins; win-win-win against lose-lose-lose; names to search | planned (words and pictures only) |

Every chapter's narration, render, native timing, listening and audiovisual
acceptance is open. Chapters 02 to 18 are planned: words and pictures only,
no scene and no registry entry.

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
right names the active coins: "published coins", filled (ink on dim). Their
chances join it at 02_5, once all three coins have been introduced:
"published coins · 49.5 · 9.5 · 74.5 %". "round-number coins · 50 · 10 ·
75 % · no handicap" is outlined in hero. The badge changes only on the
voice's own cue (02_5, 07_5, 13_1, 15_14) and with the round coins again at
14_2.

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
| 01_2 | Before guessing, let's learn the games properly, with one player and real rounds. The player starts with seven dollars. | The cards and their arrows shrink together to the top left. The player: a teal token on the left third labelled "player", a row of seven teal dollar chips beside it, a live counter "7 dollars". The badge "published coins" appears top right | token, chips, live count | |
| 01_3 | Every round, the player tosses a coin. Heads wins a dollar, and tails loses one. | A neutral coin marked "?" under the token. Demonstration, labelled "for example": heads, the coin shows H, "+1" in green, a chip fades in, "8 dollars"; tails, T, "−1" in dim, a chip fades out, "7 dollars" | coin, chips, live count | |
| 01_4 | Game A uses a single bent coin. It lands heads forty-nine and a half percent of the time, just under half. | The example labels go and the coin turns blue (card A's title flashes). Beside it "heads: 49.5 %, +1 dollar" (green) and "tails: 50.5 %, −1 dollar" (dim), from facts | text from facts | |
| 01_5 | Watch a few rounds. The money wanders up and down, and here it even climbs. A handful of tosses can't tell you whether a game wins or loses. | A small step plot to the right, "rounds" 0 to 8 against dollars: the first eight rounds of the player's game A run, drawn one toss at a time (7, 8, 9, 8, 9, 10, 11, 12, 11); the counter and chips follow | step plot from facts, live count | |
| 01_6 | So think in averages instead. In a hundred rounds you expect forty-nine and a half wins and fifty and a half losses. | The step plot clears (its rounds are the start of 01_9's run; counter and chips stay at 11). "100 rounds: 49.5 wins, 50.5 losses" | text from facts | |
| 01_7 | On average, how much does the player lose per round? ⟨predict⟩ | Hero "average loss per round?"; countdown; no answer on screen | think countdown | |
| 01_8 | One dollar per hundred rounds, which is a cent a round. Tiny, but it never stops. | TeX "49.5 − 50.5 = −1 dollar per 100 rounds" then, in red, "−1 cent a round, on average" | TeX from facts | |
| 01_9 | Here's one simulated player, playing game A for a hundred thousand rounds. The path wobbles, but sinks, ending about a thousand dollars below where it started. | The panel, coin, chips and counter leave; the token moves to round 0 at 7 dollars. Axes: rounds 0 to 100,000, dollars −1,200 to +200, ticks 0 and −1,000; a dim dashed line falling one cent a round from 7. The player's game A run in blue, drawn left to right at an even pace, the token riding its tip; the readout "after 100,000 rounds: 1,062 dollars down" (the change since the start; the plotted count ends at −1,055). A "precisely" card (K01) | plot from facts, live readout | |
| 01_10 | That's losing slowly. Luck decides any short stretch, and the cent a round decides the long run. | At "Luck decides" the first 1,000 rounds, a sliver at this scale, are outlined in hero, and a dashed hero line leads to an inset in the window's empty lower left, "first 1,000 rounds, zoomed in", at its own scale (dollars −62 to +17, labelled "13" and "−58"): the same rounds drawn again in blue, swinging 71 dollars, beside a dim dashed line falling the ten dollars a cent a round costs over them. At "the cent a round decides" the long run's dashed line glows. Hold until K01 is read; the inset, outline and K01 leave, then the plot shrinks to a thumbnail at the top left (blue; the full draft returns to it in 03 and 04) and the token fades | highlight, zoom inset from facts, thumbnail | hold |

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
- 01_10's inset is the same seeded run's first 1,000 rounds (low −58, high
  13, from the run; the tests assert both, that their swing is more than
  twice the ten dollars a cent a round costs over them, and that the drawn
  long run never comes down into the inset or its title).
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
| 02_1 | Game B has two coins and a rule for choosing between them. Before every round, the player counts their dollars and asks one question. | The player as 01 left them (teal token, "player", live count "7 dollars", seven chips in a row) and the badge "published coins". Card B appears on the right; at "two coins" a red coin "bad" and a green coin "good" inside it; at "counts their dollars" the line "count your dollars" in hero under card B's title, flashing | coins, highlight, live count | |
| 02_2 | Does the count divide by three? Arrange the dollars in rows of three and look at what's left over. | The same seven chips slide into rows of three: two full rows and one chip in the next row, which is outlined in hero and labelled "left over" | chips regroup | |
| 02_3 | Seven dollars make two rows with one left over. Eight leave two over. Nine fill three rows exactly, with nothing left over. That leftover is the remainder, and it's always zero, one or two. | Under the rows, "7 → remainder 1"; at "Eight" a chip joins (counter 8), the leftover row now holds two, "8 → remainder 2"; at "Nine" another (counter 9), three full rows, the outlined leftover row moves to the empty next row and turns red, "9 → remainder 0" in red; at "That leftover is the remainder" the line "the remainder: always 0, 1 or 2" in hero | chips, lagged reveal, live count | |
| 02_4 | When the remainder is zero, the player must toss the bad coin. It lands heads only nine and a half percent of the time. | Beside the red coin, "remainder 0 → bad coin" in red (the coin flashes), then under it "heads 9.5 %" | text from facts | |
| 02_5 | Any other remainder, and the player tosses the good coin. It lands heads seventy-four and a half percent of the time. | Beside the green coin, "remainder 1 or 2 → good coin" in green (the coin flashes), then under it "heads 74.5 %". With all three coins now introduced, as the line ends the badge gains their chances: "published coins · 49.5 · 9.5 · 74.5 %" | text from facts, badge | |
| 02_6 | Try one yourself. With ten dollars, which coin does the player toss? ⟨predict⟩ | The lines and the leftover row clear; both coins and their rules dim. At "With ten dollars" a tenth chip joins (counter 10) and all ten loosen into a heap beside the token, not in rows; countdown | think countdown | |
| 02_7 | The good coin. Ten dollars make three rows with one left over, so the remainder is one. | The green coin and its rule light again at once (flash); at "Ten dollars make" the heap slides into three rows and one, the leftover row outlined; "10 → remainder 1" | chips regroup | |
| 02_8 | Now say the good coin wins twice in a row from seven. Seven becomes eight, and eight becomes nine. The count divides by three, and out comes the bad coin. | Demonstration, labelled "for example": the last three chips leave, counter back to 7. At "Seven becomes eight" a green "+1" beside the token, the green coin flashes, a chip joins (8), "8 → remainder 2"; at "eight becomes nine" again (9), the leftover row empties and turns red, "9 → remainder 0" in red; at "out comes the bad coin" the red coin and its rule light again and the coin comes to rest under the token, where 01's coin was | coin, chips, live count | |

### Notes for 02

- The remainder is taught on the player's own chips before any triangle or
  notation; "remainder" is named only after three examples, then the viewer
  uses it once (02_6, 02_7).
- 02_8 is a demonstration (the voice says "say"); the coin results are not
  random draws and the picture labels them "for example".
- The chips are the count: the scene's tests check that for 7, 8, 9 and 10
  chips the full rows hold three each and the outlined leftover row holds
  exactly the remainder's chips (none at 9), against the facts'
  `remainder.examples`, and that 02_6's heap has no two chips touching and
  no rows.
- Pause intentions: "Does the count divide by three?" (02_2) is asked and
  answered at once, a short gap; a completed-thought pause after "zero, one or
  two" (02_3); "The good coin." (02_7) then a short beat while the chips
  regroup; after "out comes the bad coin" (02_8) a beat.
- Pronunciation: "nine and a half", "seventy-four and a half", "remainder",
  "their" (one player).

## 03_does_b_win: Does game B win? (~52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | So how often does the player meet the bad coin? Two counts in every three don't divide by three. | Card B as 02 left it on the right (coins, rules, "count your dollars"; the badge with its chances). On the left a strip under the caption "dollars": the counts 6 to 14, one by one, every third one red (6, 9, 12), the others green, with their remainders under them (caption "remainder"); at "Two counts in every three" the green counts flash | strip, lagged reveal | |
| 03_2 | If the good coin really got two rounds in three, game B would win fifty-two point eight percent of them. | Under the strip, dim "if the good coin got two rounds in three:" and TeX ⅓ · 9.5 % + ⅔ · 74.5 % ≈ 52.8 % (317/600, not exact, so ≈); at "would win", "B would win" in green | TeX from facts | |
| 03_3 | So does game B win or lose in the long run? ⟨predict⟩ | The question; countdown; no verdict on screen | think countdown | |
| 03_4 | It loses. Here's the same player, a hundred thousand rounds of game B. The money sinks again, though more slowly than in game A. | At "It loses." the countdown's "?" goes and "B would win" is struck through in red; the hypothetical dims. At "Here's the same player" the strip and the hypothetical leave, card B shrinks into card B's top slot of 01 with "count your dollars" readable beside it, and the axes of 01_9 return with A's whole run faint in blue, labelled "A" at its end. At "a hundred thousand rounds of game B" B's run draws in violet at an even pace, the token riding its tip; then "B" at its end and the readout "after 100,000 rounds: 548 dollars down"; at "more slowly" both end labels flash | plot from facts, live readout | |
| 03_5 | And look at how often the bad coin came out. Not a third of the rounds, but more than thirty-eight percent. | B's first drawing dims and the token returns to round 0. "rounds at a multiple of 3:" with no share at round 0; the number and its % come in with the first 250 rounds, and under it dim "a third of the rounds would be 33.3 %". The run draws again from its start in bright violet over the dim one, the token riding it, and the share is computed from exactly the rounds drawn so far (one tracker picks both): 36.4 % after 250 rounds, swinging a little, ending at 38.4 % (38,371 of 100,000); at "but more than thirty-eight percent" the readout flashes | live readout from facts | |
| 03_6 | Those extra rounds with the bad coin are what sink game B. Why they happen is the heart of this video. | At "what sink game B" the readout and "count your dollars" flash; hold; the readout leaves and the window, with A's faint run and B's, shrinks to a thumbnail at the top left where 01's was, the token fading | highlight, thumbnail | hold |

### Notes for 03

- 03_4 to 03_5 are the seeded run (K01's generator). The voice claims only
  what is true of it and of the exact numbers: it ends below its start
  (−548), less far than A's run (−1,062), and its share of rounds at a
  multiple of three is above 38 % (38.37 %; the exact long-run share is
  38.4 %). The exact average, −0.87 cents a round, is shown in 04_4.
- Prediction (03_3): no path or share on screen before 03_4.
- The hypothetical against the run (root's direction, 2026-10-24): 03_2 is
  labelled "if" and written ≈; 03_5's share is the run's own count over
  the rounds drawn so far. The tests check that the readout at every drawn
  point is that prefix share, that it ends at the run's 38,371 of 100,000,
  and, as a negative control, that weighting the two coins by the run's
  share instead of a third gives a win rate below a half.
- Pause intentions: a thinking beat after the question in 03_1, though it is
  answered at once; after "fifty-two point eight percent of them" (03_2) the
  question follows closely; "It loses." (03_4) then a short beat before the
  run draws; hold at the end.
- Pronunciation: "fifty-two point eight", "thirty-eight".

## 04_looks_hopeless: Why switching looks hopeless (~64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Now the switching. Before every round, the player flips a fair coin. Heads means play game A, and tails means play game B. Call this the mix. | The same teal player with "7 dollars" and the full published-coins badge. A gold coin above the token has background-coloured A/B faces; blue "heads → game A" and violet "tails → game B" rules arrive with the voice; "the mix" appears in gold | coin, stable player, rules | |
| 04_2 | Here's why the mix looks hopeless. Take two ordinary coins that both lose, one landing heads forty-nine percent of the time and one forty-eight. | Two plain coins, "49 %" and "48 %", both with a red "loses" | coins from facts | |
| 04_3 | Choosing between them with a fair coin works exactly like one coin that lands heads halfway between, forty-eight and a half percent. Two losers mix into a loser. | A gold chooser has an arrow and "1/2" to each ordinary coin. Faces and verdicts clear before the discs merge; the result reads "48.5 %", red "loses", with ½·49 % + ½·48 % = 48.5 % below. A separate hero frame points at "loses" without transforming text | coin merge from facts, outline emphasis | |
| 04_4 | Averaging the games seems to agree. Game A loses a cent a round and game B a little under a cent, so halfway between should still be a loss. | Bars to scale about zero: "game A −1.00" (blue), "game B −0.87" (violet); dashed gold "halfway: −0.93 ?". Caption: "long-run average per round, cents" | bars to scale from facts | |
| 04_5 | So when the player plays the mix, do they win or lose in the long run? ⟨predict⟩ | The question; countdown; the dashed bar waits | think countdown | |
| 04_6 | They win. Here's the same player, a hundred thousand rounds of the mix. The money climbs, ending well over a thousand dollars above where it started. | Axes return with ticks −1,000/0/+1,000, faint A/B paths and end labels. The same token rides the gold mix path; the gold coin stays. The completed readout "after 100,000 rounds: 1,748 dollars up" is pointed at by a separate hero frame, then held | plot from facts, live readout, outline emphasis | hold |

### Notes for 04

- Negative controls, the attractive wrong answers: two ordinary losing coins
  always mix into a losing coin (oracle sweep), and the average of the two
  games' averages, −0.93 cents, is not the mix's average (+1.57). Both are
  shown before the reveal and both are computed.
- 04_6: the seeded run gains 1,748 dollars, ending at a balance of 1,755
  from its initial seven. The asymptotic mean times 100,000 rounds is about
  1,570.42 dollars; that is not this finite run's expected gain from its
  initial state. "Well over a thousand" is asserted by the tests.
- The bars are drawn to scale from the facts; the scene refuses to draw if A
  and B are not both below zero.
- Pause intentions: after "Call this the mix" (04_1) a beat; a completed
  thought after "Two losers mix into a loser" (04_3); "They win." (04_6) then
  a short beat; hold on the rising run.
- Pronunciation: "forty-eight and a half".

## 05_the_clue: The clue in the counts (~53 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Nothing about the coins changed, so something else did. In game B, the player stood on a multiple of three more than thirty-eight percent of the time. In the mix, more often or less? ⟨predict⟩ | Same three runs in a lower window under a 32–40 % scale. The unchanged published badge's own edge briefly brightens in ink. Violet "game B 38.4 %" and dim "a third 33.3 %" are marked; a separate frame points at B's label while its tick pulses. Gold "the mix:" has no number through the countdown | think countdown, scale, outline emphasis | |
| 05_2 | Less. In the mix, the player stood on a multiple of three only about thirty-four and a half percent of the time, much closer to a third. | The mix path dims by stroke only and replays with the token at its tip. One tracker drives the drawn prefix, gold scale mark and live share: none at round zero, then 250 rounds through 100,000, ending at34.5 %. Separate frames point at the readout row and the third's label; only ticks pulse | live readout from facts, outline emphasis | |
| 05_3 | That's the puzzle, and it's real. Two games that each lose, mixed by a fair coin, make a game that wins just over one and a half cents a round. | The to-scale bars return; a separate frame points at A/B labels. The dashed "halfway: −0.93 ?" is struck and dimmed; the mix bar grows rightward to "+1.57". K02 holds for its reading time. Everything except the unchanged badge clears, then "Two Losing Games Can Win" / "Here is the catch" stands at least2.5seconds | bars from facts, lower-third, title card | hold |
| 05_4 | The catch is hidden in game B's rule, the part that reads your money. To find it, we need a better picture of where the player stands, and then a crowd of players. | Card B returns at (0.5,0.2), with both coins, rules and chances. The hero outline around "count your dollars" thickens and settles without transforming words. The same player's token and label return beside it | outline emphasis, callback | |

### Notes for 05

- The clue is the exposure to the bad coin, measured on the player's own runs
  before any picture of it: 38.371 % in B and 34.520 % in the mix, rounded
  on screen to38.4 % and34.5 %. These are empirical shares from the seeded
  runs. "About thirty-four and a half" is asserted by the tests.
- K02 (05_3): "exact long-run averages, dollars a round: game A −1/100,
  game B −73443/8446300, the mix +223/14200; a run's average settles on
  these with probability one (Ethier & Lee, 2009)". It holds for reading;
  the coin attribution remains in the brief and the persistent badge.
- Prediction (05_1): the mix's readout is empty until 05_2.
- Pause intentions: a longer turn before "That's the puzzle" (05_3); the title
  holds; 05_4 starts the explanation after a breath.

## 06_three_corners: The remainder becomes a place (~59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Game B only ever looks at the remainder, so let's track just that. Draw three corners, one for each remainder, zero, one and two. | The player as 02 left them: token, "player", "7 dollars", the seven chips in rows of three with the leftover row outlined, the published badge. The full rows dim; every chip and the big count flow into a small "7 dollars" counter at the top left. Three corner discs: 0 at the top, tinted red, 1 lower right, 2 lower left; then the names "0", "1", "2" | triangle | |
| 06_2 | The player's seven dollars leave one over, so the player stands at corner one. A win moves them one corner forward, to corner two. | "7 → remainder 1" under the counter; the teal token itself steps onto corner 1 (identity kept); a green forward arrow 1 → 2; the token steps, the counter "8" and "8 → remainder 2" | token moves | |
| 06_3 | From corner two, where does one more win take the player? ⟨predict⟩ | The token waits on corner 2; countdown where game A's triangle will stand; no arrow out of corner 2 yet | think countdown | |
| 06_4 | Around to corner zero. Eight dollars plus one is nine, and nine leaves nothing over. A loss always moves one corner back. | Green arrow 2 → 0, the token steps, the counter "9"; "9 → remainder 0" in red and corner 0 flashes; then a dim back arrow 0 → 2, the token steps back, "8". All six arrows drawn: forward clockwise outside in green, back inside in dim; a beat | arrows, token moves | |
| 06_5 | Now write game B on the triangle. From corner zero, the bad coin, forward nine and a half percent of the time. From corners one and two, the good coin, forward seventy-four and a half percent. | "game B" under the triangle. Arrow labels from facts, beyond each arrow's bow: corner 0's in red, 0→1 "9.5 %", 0→2 "90.5 %"; 1→2 and 2→0 "74.5 %" in green, 1→0 and 2→1 "25.5 %" dim | labels from facts | |
| 06_6 | Game A is the same from every corner, forward forty-nine and a half percent, back fifty and a half. | Game A's triangle on the left, the same size, as card A was: forward "49.5 %" in blue, back "50.5 %" dim, "game A" under it | small multiple | hold |

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
| 07_1 | Whether you hold six dollars or six hundred, game B plays the same way at corner zero. So what decides game B is how often you stand at each corner. | The picture 06 left. "6 → remainder 0" and "600 → remainder 0" appear between the top corners and both slide into B's corner 0. A "precisely" card (K03), held for its reading time | lower-third | |
| 07_2 | If you play only game A for a long time, what share of the rounds do you spend at corner zero? ⟨predict⟩ | A's corners flash; countdown between the triangles; no shares shown | think countdown | |
| 07_3 | One third. Game A treats every corner alike, so in the long run the corners share the rounds equally. The player's game A run spent thirty-three and a half percent there. | "1/3" in each of A's corners; under A, "the player's 100,000 rounds" / "33.5 % at corner 0" | bars, readout from facts | |
| 07_4 | Game B does not treat its corners alike. Its run spent more than thirty-eight percent at corner zero, and that's what we need to explain. | Outlines round B's corner-0 labels; under B, "the player's 100,000 rounds" / "38.4 % at corner 0"; B's corner 0 flashes; both readouts held side by side | flash, readout | |
| 07_5 | To see why, make the numbers rounder for a while. Each coin in these games is a round percentage with half a point shaved off. Put the half point back. | The coins at the right, A's, the bad and the good, with "49.5 %", "9.5 %", "74.5 %"; an outline round the chances; a "+0.5" rises and fades over each as they become "50 %", "10 %", "75 %", and the badge becomes the outlined "round-number coins · 50 · 10 · 75 % · no handicap" | badge, count-up from facts | |
| 07_6 | Game A becomes a fair coin, so on average it neither wins nor loses. Game B gets a ten percent coin at corner zero and a seventy-five percent coin everywhere else. | Both triangles relabel, each label replaced: A's arrows "50 %"; B's 0→1 "10 %", 0→2 "90 %", the others "75 %" and "25 %"; each coin flashes with its words | labels from facts | |
| 07_7 | These round coins are our worked example, not the published games. With equal thirds at every corner, B would now win fifty-three point three percent of its rounds. | "with equal thirds at every corner:" over TeX ⅓ · 10 % + ⅔ · 75 % ≈ 53.3 % (callback to 03_2, ≈ as there, 8/15 not a whole tenth), then "B would win", between the top corners under the badge | TeX from facts | hold |

### Notes for 07

- K03: "the capital is any whole number, negatives included, and play never
  stops; the remainder is 0, 1 or 2, so −1 leaves 2 and −3 leaves 0".
- Prediction (07_2) answered at 07_3's first words; the thirds appear with them.
- The badge is the label of the explanatory example: outlined "round-number
  coins · no handicap" from 07_5 to 12_12 and again from 14_2 to 15_13;
  the published coins return, filled, at 13_1 and at 15_14.
- Pause intentions: "So what decides game B" (07_1) follows without a long
  gap; a turn after "Put the half point back" (07_5); a thinking beat after
  "fifty-three point three percent of its rounds" (07_7), held.
- Pronunciation: "thirty-three and a half", "fifty-three point three".

## 08_one_player_to_a_crowd: One player becomes a crowd (~95 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | So is it really a third each? One player's path is full of luck. To see past it, imagine twenty-six copies of the player, all starting at corner zero and all playing game B. | 07's end: both triangles with the round-number coins' chances, the badge "round-number coins · 50 · 10 · 75 % · no handicap", the counter "8" with "8 → remainder 2", the token still on B's corner 2, and "with equal thirds at every corner:" over 07_7's sum and "B would win". The sum is outlined (really a third each); the token pulses (full of luck). A's triangle, its labels and the hypothetical leave. The same token takes one more win, 2 → 0, onto corner 0 while the counter becomes "9" with "9 → remainder 0"; at "twenty-six copies" it splits into 26 teal dots gathered on corner 0, "26" beside them; the counter and remainder give way to "round 0" and "players: 26", both live from the crowd's trackers; corner 0 gets a hollow ring (all starting at corner zero) and B's title an outline (all playing game B) | token move, dots, live count | |
| 08_2 | Let the crowd split exactly by the coins' chances. After one round, where is most of the crowd? ⟨predict⟩ | The dots merge into a teal puddle filling corner 0 (each corner's puddle has area ∝ its players) and, in the left panel, three rows "corner 0", "corner 1", "corner 2" with live bars (length ∝ players) and live one-decimal numbers "26.0", "0.0", "0.0"; under "players: 26" a dim label "split by the chances (expected numbers)", which stays through 08 and 09 (except 08_7's real crowd, which has its own label) and returns on 11's bars and 14's crowd of 1,418; a small dim note "shown to one decimal; the total is exactly 26"; countdown; nothing has moved | think countdown | |
| 08_3 | At corner two. A tenth of the crowd moves forward, and nine tenths fall back to corner two. These are not fractions of people. Two point six and twenty-three point four are expected numbers: twenty-six times each chance. | Round 1, slowly (2.4 s): teal streams on the 0 → 1 and 0 → 2 arrows, widths ∝ their flows, as the puddles and bars move; "2.6" (green) appears past the 10 % label and "23.4" (dim) past the 90 % label, each label outlined in turn. At "not fractions of people" a "precisely" card (K04); at "are expected numbers" the dim label is outlined | bars, flows, live numbers | |
| 08_4 | The next round, three quarters of corner two win their way around to zero. Every player is always somewhere, so the total stays twenty-six while the crowd sloshes between corners. | Round 1's streams and numbers fade. Round 2 (1.6 s) with its streams: three quarters of corner 2 win around to 0, (18.2, 5.9, 2.0); "players: 26" is outlined; rounds 3 to 6 at 0.7 s each, streams on the arrows each round: (2.9, 2.3, 20.8), (16.2, 5.5, 4.4), (4.6, 2.7, 18.7), (14.7, 5.1, 6.2); "players: 26" never changes | bars, flows, live numbers | |
| 08_5 | Watch the share at corner zero, where the bad coin waits. It swings wildly at first, then steadies above a third. | K04 leaves once read. A live readout "share at corner 0 … %" and a small trace of it against rounds 0 to 60, with a dashed line labelled "33.3 %"; corner 0 gets a hollow ring (where the bad coin waits); the trace is outlined (swings wildly at first); rounds 7 to 12 at half a second each | live readout, trace | |
| 08_6 | Round after round the sloshing dies down, and the crowd settles at ten players on corner zero, four on corner one and twelve on corner two. Not a third each. | Rounds 13 to 60 at a tenth of a second each; the bars and puddles converge and the trace flattens; at each phrase that row's live number gives way to a fixed teal "10", "4", "12", the row outlined; at "Not a third each" the share's readout becomes a fixed "38.5" and a small dim "within half a player of 10, 4, 12 from round 24" appears; the share row, then "33.3 %", are outlined | bars, live numbers | hold |
| 08_7 | Real players don't split this neatly. Toss real coins for these twenty-six, and the counts jump from round to round. | The share row, its trace, the precision note and the round counter leave; the dim label is outlined (real players don't split). At "Toss real coins" a second, smaller triangle at the lower left, labelled "real tosses, seeded illustration", with its own "round 0" and 26 teal dots starting at 10, 4, 12, a live count beside each corner; the split bars and puddles stay at 10, 4, 12 beside it. After the line, in silence, its first eight computed rounds hop at one a second, the dots moving and the counts jumping: (9, 6, 11), (7, 5, 14), (9, 6, 11), (8, 8, 10), (9, 2, 15), (14, 2, 10), (6, 5, 15), (10, 5, 11); the split crowd never moves off 10, 4, 12 | dots, live counts | hold |
| 08_8 | In this run of a thousand rounds, they land on exactly ten, four and twelve only forty times, but on average, that's where they sit. | "exactly 10, 4, 12 so far:" with a live tally (0 after the eight rounds shown); at "In this run of a thousand rounds" the real counter runs to "1,000" in 3 s, the dots snapping to each round's places and the tally climbing; at "only forty times" the tally line gives way to "this run: exactly 10, 4, 12 in 40 of 1,000 rounds", and at "on average" "average over the 1,000 rounds: 10.0, 4.0, 12.0" appears (held). The real triangle, its dots, counts and lines leave; the split crowd stays at 10, 4, 12 | live counts, lines from facts | hold |

### Notes for 08

- K04: "expected numbers: 26 times one player's chance of being at that
  corner after that round; real players come whole and scatter around them".
- The split crowd runs to round 60: the first round from which every
  readout (the three counts and the share at corner 0, at one decimal)
  shows its settled value for good through round 200 (the counts from
  round 41, the share from round 60; from round 41 to 59 the share still
  alternates between 38.5 and its neighbours, 38.3 to 38.6). The shown
  one-decimal counts can add to 26.1 (rounds 2 and 4, for example) or 25.9
  (round 7), hence the precision note; the live total adds the unrounded
  counts and reads 26 in every frame.
- Conserved flows before weights: the total stays 26 every round (the facts
  check it); the corner 0 share is shown as a changing quantity here and is
  named "weight" only at 09_9.
- The bars show one decimal; 26 dots return in 09 for the whole-number check.
- Prediction (08_2): no flow before 08_3.
- 08_7's real crowd is computed (`realcrowd.*`, `model::real_crowd`): the
  same 26 players from 10, 4, 12, the round-coin B, each player's toss in
  turn from the film's SplitMix64 seed 1999 (the eight rounds shown are the
  first eight). The tests assert that it leaves 10, 4, 12 in its first
  round while the split crowd never does, that it lands on exactly 10, 4, 12
  in 40 of 1,000 rounds, and that its averages are 10.0, 4.0, 12.0. It is an
  illustration of one seed, labelled so; "on average" means over these
  1,000 rounds, and successive rounds of one crowd are not independent, so
  40 of 1,000 is this seed's count, not a probability. It answers the
  review's M1 (GoldGate) before 09's exact check, and calls back to
  01_10: luck decides any short stretch, the chances the long run, now for
  a crowd.
- Pause intentions: a held breath on "imagine twenty-six copies" (08_1) as the
  token splits; "At corner two." (08_3) then the streams; a listen-quiet hold
  after "Not a third each" (08_6) so the settling is seen (hold); after
  "jump from round to round" (08_7) the real dots hop in silence for eight
  seconds (hold) before 08_8 counts the thousand rounds, and the two result
  lines hold at its end (hold). The words of the old 08_7 are split at that
  silence, so the first real rounds are seen before they are summarized.
- Pronunciation: "a tenth", "nine tenths" (the final -nths cluster) and
  "twenty-three point four" (08_3), "sloshes" (08_4).

## 09_ten_four_twelve: A crowd that stays put (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Check that this crowd really stays put, still split exactly by the chances. At corner zero, ten players toss the bad coin, so one wins and moves forward, and nine drop back to corner two. | 08's end: B's triangle with the round-number chances and the badge, "players: 26", the dim "split by the chances (expected numbers)", the panel's bars and the corners' puddles at 10, 4, 12. The bars and puddles become 26 dots again: 10 at corner 0, 4 at 1, 12 at 2; the dim label is outlined (still split exactly by the chances). Corner 0 gets a hollow ring as its dots pulse; they split: one waits past the start of the forward arrow 0 → 1, outside the triangle (green "1"), nine past the start of the back arrow 0 → 2, inside it (dim "9") | dots, labels from facts | |
| 09_2 | At corner one, the good coin splits four players. Three move on to corner two, and one falls back to zero. | A hollow ring at corner 1 as its dots pulse; they split and wait: "3" forward (green), "1" back (dim) | dots | |
| 09_3 | At corner two, it splits twelve. Nine wrap around to zero, and three fall back to one. | A hollow ring at corner 2 as its dots pulse; they split and wait: "9" forward, around to 0 (green), "3" back (dim) | dots | |
| 09_4 | Now let everyone move at once. How many players land on each corner? ⟨predict⟩ | The question "how many land on each corner?" in the panel; countdown; the dots still wait and nothing is summed | think countdown | |
| 09_5 | Corner zero gets one plus nine, ten. Corner one gets one plus three, four. Corner two gets nine plus three, twelve. Everyone moved, and the counts come back exactly as they were. | All 26 dots move at once (2 s), straight to the corners their arrows point at, while the groups' counts and the question fade; at each "Corner … gets" its arrival sum appears beside it, each summand in its arrow's colour (green forward, dim back), listed by the corner it comes from: "1 + 9 = 10", "1 + 3 = 4", "9 + 3 = 12"; at "exactly as they were" each total is outlined in turn (held) | swarm move, text from facts | hold |
| 09_6 | Count the winners. One, three and nine make thirteen, exactly half of twenty-six. Without the half point, game B is perfectly fair. | The 13 forward movers pulse; "winners: 1 + 3 + 9 = 13", then, green, "13 of 26 = 1/2"; the badge is outlined (without the half point) | pulse | |
| 09_7 | Now look at where the crowd sits. Ten of twenty-six is thirty-eight and a half percent, more than a third. | A red ring around corner 0's 10 dots, pulsing; "share at corner 0: 10 of 26" and a red bar growing to a live "38.5 %"; a dim mark at "33.3 %" across the bar, pulsing (more than a third) | bar from facts | |
| 09_8 | Those extra rounds at corner zero, with the bad coin, cancel the good coin's advantage exactly. | The bar falls back to the mark, 33.3 %, and "B wins 53.3 %" appears (07_7's equal-thirds count, live from the same share); at "with the bad coin" the bar grows back to 38.5 % while B's wins slide to "50.0 %"; the B-wins row is outlined (cancel the good coin's advantage exactly) | tween | |
| 09_9 | Each corner's long-run share of the rounds is called its weight. For game B, the weights are five thirteenths, two thirteenths and six thirteenths. | The line "weight: a corner's long-run share of the rounds"; at each fraction that corner's sum gives way to its weight, "10/26 = 5/13", "4/26 = 2/13", "12/26 = 6/13". After the line, lower-third: "names to search: stationary distribution · Markov chain" | lower-third, labels from facts | |

### Notes for 09

- Prediction (09_4): the arrivals are not summed on screen before 09_5.
- Facts: the flows are whole players (26 is the smallest crowd for which
  they are; the oracle searches every smaller crowd).
- Expected numbers, said and shown: 08_2's frame, 08_7's real crowd, 09_1's
  "still split exactly by the chances" and the persistent dim label keep
  09_1 to 09_6 true of the split crowd, not of tosses (for 26 real players
  at 10, 4, 12, one round of the
  round-coin B comes back exactly 10, 4, 12 only about 7 % of the time:
  0.0707, GoldGate's review, M1, and recomputed here from the three
  binomials, wins b at corner 1 with b − 2 at corner 0 and b + 6 at
  corner 2, b = 2, 3 or 4).
- Arrival sums are listed by the corner they come from, lowest first.
- Lint disposition: 09_5's three sentences opening "Corner …" are the count
  itself, said in the order the arrivals appear; kept on purpose.
- Pause intentions: each corner's split is its own thought (09_1 to 09_3),
  with short completed-thought gaps; after "exactly as they were" (09_5) a
  longer hold so the unchanged counts register; "perfectly fair" (09_6) ends
  a section; 09_9 names the weight as the chapter closes.
- Pronunciation: "thirteenths", three times in a row (09_9), and "thirty-eight
  and a half percent" (09_7).

## 10_the_trap: Why corner zero collects players (~102 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Why does corner zero collect extra players? Follow one who has just arrived there. Where is that player most likely to be one round later? ⟨predict⟩ | 09's picture: B's triangle with the round-number coins' chances, the badge "round-number coins · 50 · 10 · 75 % · no handicap", "players: 26" over the dim "split by the chances (expected numbers)", and the 26 dots at 10, 4, 12. "collect extra players" gives corner 0 a hollow ring while its dots pulse. At "Follow one" the crowd dims, the panel's head leaves and the same player stands out: the teal token on corner 0 with the counter "9" and "9 → remainder 0" (08_1's last move), a hollow ring round the corner. "one round later: which corner?" on screen; countdown | think countdown | |
| 10_2 | At corner two, nine times in ten. The bad coin knocks them back. And from corner two, the good coin usually wins, which carries them forward, around to zero again. | At "At corner two" the question leaves and the token takes the back arrow 0 → 2 (1.6 s), leaving a red trace over it, while the counter becomes "8" and "8 → remainder 2"; "nine times in ten" outlines the "90 %" label as the trace pulses; "knocks them back" rings corner 0; "usually wins" outlines the "75 %" by 2 → 0; at "around to zero again" the token takes the forward arrow 2 → 0 with a green trace, "9" and "9 → remainder 0" again | token path, traces | |
| 10_3 | That was one player's likeliest two rounds, not the only way they can go. To see how common it is, go back to the whole crowd, settled at ten, four and twelve. | "illustration: one player's likeliest two rounds" (teal) under the counter, the two traces pulsing. At "go back to the whole crowd" the token, counter and remainder give way to "players: 26" and its label, the dots return to full strength and the traces dim; at "settled at ten, four and twelve" teal "10", "4", "12" beside the corners, every corner ringed and the dots pulsing | labels from facts | |
| 10_4 | Each round, the good coin wins for three players at corner one and for nine at corner two. So of its twelve wins a round, nine happen at corner two, and every one of them just brings a player back to zero, where the bad coin waits. | "the good coin's wins each round:" (dim); at each phrase its line, "corner 1: 75 % of 4 = 3", then "corner 2: 75 % of 12 = 9", that corner ringed and its winners' dots pulsing; at "nine happen at corner two" "the share of them at corner 2:" over 9/(3 + 9) = 3/4 (green TeX). At "brings a player back" a teal stream, its width ∝ 9 of 26, draws along the 2 → 0 arrow; "where the bad coin waits" rings corner 0 with its dots | TeX from facts, stream | |
| 10_5 | That's the trap in game B. The good coin wins often, but its wins keep getting spent on climbing back to a multiple of three. | "the trap: 0 → 2 → 0" in red at the foot of the panel; the two traces return to full strength and pulse; "climbing back to a multiple of three" rings corner 0 again with its dots | traces | hold |
| 10_6 | One change removes the trap. Let a die choose B's coin instead of your money. A one or a two means the bad coin, anything else the good coin. | The traces, the stream, the counts and the wins leave; "game B: your dollars pick the coin" (violet). At "Let a die choose" a dark veil dims the whole triangle (its corners no longer choose) and "die game: a die picks the coin" appears; "instead of your money" outlines B's rule. At "A one or a two" a row: die faces 1 and 2 → the red "bad" coin, "10 %"; at "anything else" a row: faces 3, 4, 5, 6 → the green "good" coin, "75 %" | die faces, coins | |
| 10_7 | Now the bad coin really does get one round in three, wherever you stand, and this game wins fifty-three point three percent of the time. | "1/3 of rounds" and "2/3 of rounds" beside the coins, the 1 and 2 faces pulsing; "wherever you stand" rings all three corners through the veil; at "this game wins" the TeX 1/3 · 10 % + 2/3 · 75 % ≈ 53.3 % (07_7's sum, now true of a real game) and "the die game wins" in green | TeX from facts | hold |
| 10_8 | Same two coins. Only the rule that picks between them changed. Reading your money is what drags game B from fifty-three point three percent down to exactly half. | Hollow rings round both coins; "Only the rule" outlines both rules; at "Reading your money" the die, coins and veil leave, the triangle returns and B's rule is outlined; at "down to exactly half" "reading your money: 53.3 % → 50 %", outlined | highlight | |

### Notes for 10

- One player's path is an illustration, not the crowd's claim (Mail 6904):
  the token's two moves are the likeliest of the four two-round paths from
  corner 0, 27 times in 40 (9/10 · 3/4), and are labelled as one player's;
  0 → 2 → 1 (9 in 40), 0 → 1 → 2 (3 in 40) and 0 → 1 → 0 (1 in 40) also
  happen. The crowd's claim comes after, from the settled split crowd's
  counts: 75 % of 4 = 3 and 75 % of 12 = 9, so 9 of the good coin's 12 wins
  a round happen at corner 2 (fact `crowd.good_wins_at2` = 3/4; it prints
  75.0 % with the published coins too, GoldGate's review, M3, though there
  it is not exactly 3/4), and every win there is the forward move 2 → 0.
  Counting the bad coin's one win too would give 9 of 13, not 3/4; the 3/4
  equals the good coin's 75 % only by these coins' accident (a 70 % good
  coin gives 97/134).
- The die-chosen game is the negative control for "B loses because of where
  you stand": the same coins with the choice made independently of the money
  give an ordinary coin that wins 8/15 = 53.3 % with the round-number coins
  (317/600 = 52.8 % with the published ones), so it wins. As a game on the
  triangle its crowd sits at equal thirds. 10_7's sum is shown with ≈:
  8/15 is 53.33… %.
- Pause intentions: after "nine times in ten" (10_2) a short beat, and a
  beat after the two rounds; 10_3 turns from one player to the crowd; the
  trap sentence (10_5) is a completed thought, held; "One change removes the
  trap" (10_6) starts a new idea after a turn; the die game's verdict (10_7)
  is held.
- Pronunciation: "likeliest" (three syllables, unhurried); "die" the cube;
  "fifty-three point three".

## 11_mix_the_games: Mix the games (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Now mix the games, as the player did. Before every round, a fair coin decides whether you play A or B. | 10's crowd: B's triangle with its chances, the badge and the 26 dots at 10, 4, 12. At "Now mix the games" 04's gold coin "A" with "the mix" above it, at 04's own place; "as the player did" rings the coin; at "a fair coin decides" "heads → game A" (blue) and "tails → game B" (violet) beside it | coin | |
| 11_2 | On the triangle, every arrow becomes an average. At corner zero, half the time it's A's fifty percent, half the time B's ten percent. So forward thirty percent in all. | "every arrow becomes an average": all six arrows pulse. At "A's fifty percent" a blue "50 %" waits past corner 0's forward label, outlined; "B's ten percent" outlines B's "10 %"; at "forward thirty percent in all" both give way to the mix's gold "30 %", the back label "90 %" to "70 %", and the panel shows 1/2 · 50 % + 1/2 · 10 % = 30 % | label swap from facts, TeX | |
| 11_3 | At corners one and two, halfway between fifty and seventy-five, forward sixty-two and a half percent. | Corners 1 and 2 ringed; at "halfway between" 1/2 · 50 % + 1/2 · 75 % = 62.5 % under the first sum; at "sixty-two and a half percent" the forward labels at corners 1 and 2 become gold "62.5 %", their back labels "37.5 %", and the title "game B" becomes "the mix" in gold | label swap, TeX | |
| 11_4 | Start the mix from game B's settled crowd, ten, four and twelve, still split exactly by the chances. Which way will the crowd shift? ⟨predict⟩ | At "Start the mix" the coin, rules and sums leave for "round 0", "players: 26" (live total), the dim "split by the chances (expected numbers)", the rows "corner 0", "corner 1", "corner 2" with live bars (length ∝ players) and live one-decimal shares "38.5 %", "15.4 %", "46.2 %", and the dim note "shares shown to one decimal; the total is exactly 26". At "ten, four and twelve" the dots merge into puddles (area ∝ players) and each bar gets a dim ghost outline at B's height, kept to the end; the corners are ringed. "still split exactly by the chances" outlines the label. "which way will the crowd shift?"; countdown. No settled value, tick or line is drawn | think countdown | |
| 11_5 | Off corner zero. Its share drops at once from thirty-eight point five percent to about thirty-four point six, then sloshes a little above and below that. | At "Off corner zero" the question leaves for a small trace, "share at corner 0, round by round" (axes and "rounds", no numbers), and corner 0 is ringed. At "Its share drops at once" round 1 plays slowly (2.2 s) from 10, 4, 12 under the mix, teal streams on the six arrows (widths ∝ flows): the shares become 34.6, 28.8, 36.5 (corner 1's bar shoots past where it will end); "to about thirty-four point six" outlines corner 0's share. At "then sloshes" rounds 2 to 6 at a second each, with streams: 33.7 / 24.1 / 42.3, 35.4 / 25.9 / 38.6, 33.9 / 25.1 / 41.0, 35.1 / 25.5 / 39.4, 34.2 / 25.3 / 40.5, the trace zigzagging. After the line, rounds 7 to 12 in silence at half a second each, then a beat | bars, live numbers, streams, trace | hold |
| 11_6 | Round after round the swings shrink, and the crowd settles. Corner zero keeps about thirty-four point six percent, and corner one ends up at twenty-five point four percent, up from fifteen point four. | Rounds 13 to 18 at 0.3 s each; at "the crowd settles" dim ticks across the bars at their settled heights and a dashed line through the trace labelled "34.6 %", the trace's crossings now readable against it. At "Corner zero keeps" corner 0's share becomes a fixed teal "38.5 → 34.6 %", outlined; at "corner one ends up" corners 1 and 2 become "15.4 → 25.4 %" and "46.2 → 40.1 %", corner 1's outlined; "up from fifteen point four" pulses corner 1's ghost | ticks, labels from facts | |
| 11_7 | Game A's coin ignores the corners, so its rounds keep stirring the crowd toward equal thirds. That shift of weight off corner zero is where the winning comes from. | "ignores the corners" rings all three; at "toward equal thirds" dashed ticks at a third on each bar and a dashed line at "33.3 %" on the trace; at "That shift of weight" a red outline fills the gap between corner 0's bar and its ghost, pulsing, and corner 0 is ringed; "where the winning comes from" outlines "38.5 → 34.6 %" | outline | hold |

### Notes for 11

- The shift is the reveal: B's ghost heights stay beside the mix's bars
  from 11_4 to the end. The player's own mix run (05_2, 34.5 % with the
  published coins) is the same effect seen in one player; it is 13_8's
  callback. 11 draws no 34.5: its numbers are the round-number coins'
  split-crowd model (34.6 %), not the published run's measurement.
- 11_5's rounds are computed (`mixshift.round.1` to `.6`, the split crowd
  of 26 from 10, 4, 12 under the round-coin mix), not tweened between end
  points: corner 0 reaches its settled share within a tenth of a point in
  the first round (34.615 % against 34.556 %; both print 34.6) and then
  crosses it five times in six rounds, while corner 1 overshoots to 28.8 %
  first (GoldGate's review, M2). Corner 0's swing shrinks every round from
  round 2, and every readout shows its settled value for good from round
  18, where the run stops. The tests assert the first drop, its nearness to
  the limit, the crossings, the overshoot and the settling round.
- Nothing settled is drawn during the prediction (11_4): the ticks, the
  dashed lines and the fixed rows wait for "the crowd settles" (11_6).
- Failed control (tested): halfway between A's equal thirds and B's 38.5 %
  would be 35.9 %, not 34.6 %.
- Pause intentions: after "every arrow becomes an average" (11_2) a short
  beat; 11_4's question gets its countdown; after 11_5 the silent rounds and
  a beat, so the sloshing can be watched; 11_6 ends a section; a hold after
  11_7 (the reveal).
- Pronunciation: "sloshes"; "thirty-four point six", "twenty-five point
  four" (each "point" clear).

## 12_one_dial: One dial decides game B (~140 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | Stay with the round coins for now. With them, game A and game B each break even when they're played alone. Inside the mix, a fair coin splits the rounds into A rounds and B rounds. | 11's panel stays: the outlined badge "round-number coins · 50 · 10 · 75 % · no handicap", the 26 players, the three corner rows with the mix's teal bars over B's dashed ghosts and their settled shares ("38.5 → 34.6 %" at corner 0). The badge is framed at "Stay with the round coins". Where the dial will stand, "game A alone: breaks even" (blue) and "game B alone: breaks even" (violet). At "a fair coin splits the rounds" every teal bar becomes two halves of its own length, blue over violet, with "A rounds" and "B rounds" beside them | bars from the model | |
| 12_2 | That coin never looks at the money, so at every corner, half the rounds are A rounds and half are B rounds. And A's coin wins exactly half the time wherever it's tossed, so the A rounds can't be where the winning comes from. | One frame round all six halves; the blue halves flash, then the violet. At "A's coin wins exactly half the time" the "A rounds" line becomes "A rounds: 50 % at every corner", framed; at "can't be where the winning comes from" the blue halves dim | shape emphasis only | beat |
| 12_3 | Everything rests on the B rounds. Their chance of winning depends on just one number: what share of them are played at corner zero. Every B round there swaps a seventy-five percent coin for a ten percent one. | The violet halves flash; at "just one number" a frame round corner 0's violet half; at the swap the "B rounds" line becomes "B rounds: 10 % at corner 0, 75 % elsewhere", framed | coin chances from facts | |
| 12_4 | Think of that share as a dial. Game B on its own sets it at thirty-eight point five percent. | The two "alone" lines give way to the dial's share axis, "32 %" to "40 %" with a tick at every point, titled "the dial: share of B's rounds at corner 0". At "Game B on its own" a violet mark at 38.5 % with "game B alone: 38.5 %", and corner 0's ghost (B's own share) flashes | dial from facts | |
| 12_5 | Inside the mix, B's rounds sit wherever the crowd sits, half of every corner's rounds. The mix moved the crowd off corner zero, and that turns the dial down to thirty-four point six percent. | A frame round the violet halves. At "moved the crowd off corner zero" a red outline marks the part of corner 0's ghost the mix's bar no longer fills, and flashes. At "turns the dial down" one tracker moves a hero marker along corner 0's bar from B's share to the mix's and, bound to the same value, a gold needle on the dial from 38.5 % to 34.6 %; "the mix: 34.6 %" under the needle | one tracker for bar and dial | beat |
| 12_6 | What is that turn worth? With a third of B's rounds at corner zero, B wins fifty-three point three percent. At game B's own thirty-eight point five, exactly half. | The win axis, "B wins", 48 % to 56 %. At "With a third" a dim mark "a third: 33.3 %" and its dashed upright; at "fifty-three point three percent" a point at (33.3 %, 53.3 %) with "53.3 %". At "At game B's own" a point slides along B's chances from the third to 38.5 %, drawing the line behind it; at "exactly half" the dashed 50 % line with "50 %" (framed) and B's violet point on it | computed slide | beat |
| 12_7 | In between, it's a straight line. B wins seventy-five percent, minus sixty-five percent of its share at corner zero. So it breaks even at exactly five thirteenths, the weight we found for corner zero. That's why game B on its own breaks even. | The whole line across the dial; the slide's trail goes. In the panel, TeX "B wins = 75 % − 65 % × share"; at "exactly five thirteenths", "50 % at share = 25/65 = 5/13 ≈ 38.5 %" while B's point flashes; at "the weight we found" corner 0's ghost flashes; at "That's why" a frame round "game B alone: 38.5 %" pointing at B's point | TeX from the model | |
| 12_8 | At the mix's thirty-four point six percent, then, what do B's rounds win? ⟨predict⟩ | A gold dashed upright at 34.6 % to the top of the dial, no point on the line; "at 34.6 %, what do B's rounds win?" under the panel; the countdown | think countdown | |
| 12_9 | Fifty-two point five percent. Averaged with the A rounds' fifty, the mix wins fifty-one point three percent of all its rounds. | The upright and the question give way to a gold point on the line at (34.6 %, 52.5 %) with "52.5 %", flashed; at "Averaged with" the TeX becomes "½ · 50 % + ½ · 52.5 % ≈ 51.3 %", framed at "of all its rounds" | TeX from the model | |
| 12_10 | On average, that's a gain of just over two and a half cents a round. | "+2.54 cents a round" in gold under the sum; the "precisely" card (K05) | lower-third card | |
| 12_11 | So game A's rounds only break even. They move the crowd off the bad corner, and game B's good coin does the winning. | A frame round "A rounds: 50 % at every corner"; the red outline flashes at "move the crowd off the bad corner"; at "game B's good coin" a frame round the B rounds line pointing at the violet halves on corners 1 and 2 | shape emphasis only | |
| 12_12 | Game A on its own keeps a third of the rounds at every corner. The mix pulls corner zero most of the way from game B's share to that third. | Dashed ticks at a third on all three bars; the dial's third mark flashes. At "most of the way" a hero arrow on the dial from 38.5 % to 34.6 %, dashed on to 33.3 %, with "76 % of the way to a third"; framed, pointing at the red outline | arrow from facts | hold |

### Notes for 12

- Order (root's direction, Mail 6914): the one changed dial and the crowd
  movement come first (12_4 and 12_5, one tracker driving both the marker on
  corner 0's bar and the dial's needle); then the two known points; then the
  straight line and its threshold (12_7); only then the prediction.
- The halves (12_1) replace the old strip of tiles: each corner's bar keeps
  its length and splits into equal A and B halves, because the fair coin
  that picks the game never reads the money, so at every corner half of the
  rounds are A rounds. A coin that preferred A at corner 0 would not split
  them evenly (checked exactly on the chain of corner and game).
- The line is exact: B wins 3/4 − (13/20) × share whatever corners 1 and 2
  split, so it is even at share 5/13, game B's own weight for corner 0.
  The published coins' line is exactly half a point lower at every share.
- The sums are shown at one decimal with "≈": ½ · 50 + ½ · 52.5 is 51.25,
  which prints as 51.3 but is not the mix's 51.27 (exactly 727/1418); "="
  appears only where every printed value is exact.
- K05: "exactly: the mix wins 727/1418 of its rounds, and every round wins
  or loses one dollar, so it gains 2 · 727/1418 − 1 = 18/709 of a dollar a
  round (Ethier and Lee also give 18/709)". The crowd of 1,418 is left to
  14, which counts it.
- Prediction (12_8): the answer lies between 50 % (B's own share) and
  53.3 % (a third), nearer 53.3 %; the scene draws no value before 12_9.
- Failed controls (tested): "the A rounds must be doing the winning" (they
  win exactly 1/2); B's rounds spread as B alone's would only break even;
  a switch that reads the corner (chapter 18) plays every B round on the
  bad coin, not at the crowd's shares.
- Pause intentions: a held beat after "can't be where the winning comes
  from" (12_2); a beat after the dial turns (12_5) and after the two points
  (12_6); 12_8's countdown; the answer (12_9) begins promptly; the card
  (12_10) stands until it can be read; 12_12 holds.
- Pronunciation: "five thirteenths" (not "thirtieths"); "thirty-four point
  six", "fifty-two point five" (each "point" clear).

## 13_the_half_point: The half point returns (~114 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | Now shave the half point off every coin again, back to the coins Harmer and Abbott published, the ones the player used. With these, both games really lose. | The dial as 12 left it: B's round line, B alone's violet point on the dashed 50 % line, the mix's gold point with "52.5 %", the marks "game B alone: 38.5 %" and "the mix: 34.6 %", the outlined round badge. New at the top left: the three coins in a row, A, bad and good, with "50 %", "10 %", "75 %"; under them a ledger "long-run average per round, in cents": game A 0.00, game B 0.00, the mix +2.54. At "shave the half point" a "−0.5" falls toward each chance, fading, as it becomes 49.5 %, 9.5 %, 74.5 %; at "published" the badge turns to the filled "published coins · 49.5 · 9.5 · 74.5 %"; at "both games really lose" a frame round "game A" and "game B" | coins, ledger, badge from facts | |
| 13_2 | Game A slips from fair to losing a cent a round. | "→ −1.00" beside game A's 0.00, framed | ledger from facts | |
| 13_3 | The crowds barely move. Game B's corner zero slips from thirty-eight point five to thirty-eight point four percent, and the mix's from thirty-four point six to thirty-four and a half. | The round coins' points dim and "52.5 %" goes. One tracker shaves all three coins from the round chances to the published ones while both marks follow their crowd's corner 0 (B's by a tenth of a point, the mix's by half that), then settle as fixed ticks; at "to thirty-eight point four percent" B's mark words become "game B alone: 38.4 %" (framed, the tick flashes); at "to thirty-four and a half" the mix's become "the mix: 34.5 %" | one tracker, computed shares | beat |
| 13_4 | But B's line drops by half a point everywhere, so the share where it breaks even falls to thirty-seven point seven percent. Game B's own crowd sits just past that, so B loses a little under a cent a round. | A second tracker lowers B's line by half a point at every share; the old line stays faint, the dimmed round points go, and a hero dot rides the line's crossing of 50 % from 38.5 % to 37.7 %. At "falls to" a hero mark "break even: 37.7 %" with its dashed upright, and in the panel "B wins = 74.5 % − 65 % × share" over "50 % at share = 24.5/65 = 49/130 ≈ 37.7 %". At "sits just past that" B's violet point at 38.4 %, just under the 50 % line, with its upright, flashed; at "B loses" game B's "→ −0.87", framed | trackers; TeX from the model | beat |
| 13_5 | The mix had an edge of just over two and a half cents a round. Does it survive the handicap? ⟨predict⟩ | A frame round the mix's +2.54; at "Does it survive" the panel's sums give way to the mix's "→ ?" in hero; the countdown | think countdown | |
| 13_6 | It does. At thirty-four and a half percent, the mix's B rounds still win fifty-two point one percent. The A rounds now lose a little, so the mix wins fifty point eight percent of its rounds. | At "It does" the "?" gives way to a gold point on the lowered line at 34.5 % with "52.1 %" and a dashed upright, flashed at "still win"; at "The A rounds now lose a little" a frame round A's "49.5 %", pointing at the A coin; at "fifty point eight percent" the TeX "½ · 49.5 % + ½ · 52.1 % ≈ 50.8 %" | TeX from the model | |
| 13_7 | The half point costs the mix about one cent a round, and the edge was bigger than that. The mix still wins just over one and a half cents a round. | TeX "+2.54 − 0.97 ≈ +1.57 cents" under the sum, framed at "the edge was bigger than that"; at "The mix still wins" the "→ ?" becomes the mix's "→ +1.57" in gold, framed | TeX from facts | |
| 13_8 | The player's own runs showed these same shares: thirty-eight point four percent in game B, and thirty-four and a half in the mix. | Across the foot, "model, published coins: game B 38.4 %, the mix 34.5 %" (dim) over "the player's runs, 100,000 rounds each: game B 38.4 %, the mix 34.5 %" (teal); a teal chip on each mark at the run's own share, each flashed at its words | runs from the model | beat |
| 13_9 | That's every number from the opening. A loses a cent a round, B a little under a cent, and the coin-flip mix of the two wins. | Everything but the badge goes; the bars of 05_3 return exactly: A −1.00, B −0.87, the halfway guess struck and faint, the mix +1.57. A frame round A's and B's labels; then round the mix's label, pointing at its bar | bars from facts | hold |
| 13_10 | Harmer and Abbott's paper calls the effect Parrondo's paradox, after its discoverer, Juan Parrondo. It appeared in Nature in nineteen ninety-nine, and it's a single page. | The bars go. A citation card: "G. P. Harmer and D. Abbott", "Losing strategies can win by Parrondo's paradox", "Nature 402, 864 (1999)"; the last line framed at "It appeared in Nature" | citation card from facts | hold |

### Notes for 13

- The explanatory coins and the real ones stay apart: the outlined round
  badge until 13_1's "published", the filled published badge after it; the
  ledger keeps the round coins' values beside the published ones
  ("0.00 → −1.00").
- The published numbers: B's weights 38.4 %, 15.4 %, 46.2 %; the even share
  49/130 = 37.7 % (5/13 less 1/130: the half point over the line's slope of
  65 points); the mix's weights 34.5 %, 25.4 %, 40.1 %; the mix's B rounds
  52.1 %; the mix wins 50.8 %; −1.00, −0.87 and +1.57 cents a round.
- The crowds' move is small on purpose (13_3 says "barely"): B's corner 0
  moves 0.10 of a point, the mix's 0.05. The words change with them.
- Failed controls (tested): "every coin is half a point weaker, so the mix
  loses a whole cent" gives 1.00, but the cost is 0.97, because the mix's
  crowd also moves off corner 0; shaving only the good coin would put B's
  even share at 49/129, not 49/130.
- 13_8's two lines are labelled: the model's long-run shares (published
  coins) and the measured shares of the player's seeded runs, 100,000
  rounds each (one seed, an illustration). They agree at the one decimal
  shown (38.37 % against 38.36 %, 34.52 % against 34.51 %), and the tests
  assert that they differ at two.
- History: only the paper's own sentence ("called Parrondo's paradox, after
  its discoverer, Juan Parrondo"), said as what the paper calls it, and its
  date and length. Not who coined the name (the page does not say;
  GoldGate's review, M4), no year of discovery, no affiliation. The card's
  volume, page and year are facts pinned to the citation (`source.volume`,
  `source.page`, `source.year`).
- The run thumbnails beside the bars (old 13_7) are dropped: the bars and
  their labels already carry every number, and the frame has no room for
  three windows beside them.
- Pause intentions: a beat after the crowds move (13_3) and after B's
  verdict (13_4); 13_5's countdown; "It does." (13_6) is followed by a
  short beat in the reading; a beat after the runs (13_8); a look at the
  bars (13_9); the citation (13_10) holds.
- Pronunciation: Juan Parrondo ("hwahn pah-RON-doh"), Harmer, Abbott,
  "nineteen ninety-nine", "thirty-seven point seven", "thirty-four and a
  half".

## 14_net_flow: The crowd circulates (~282 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | Here's a new question. Could you tell whether a game like these wins from its coins alone, without first working out where its crowd settles? There is a test, and it comes from watching a settled crowd move around the triangle. | The mix's triangle with B's corners and arrows, the gold title "the mix", and the published mix's chances forward 29.5, 62, 62 % in gold and back 70.5, 38, 38 % in dim; the matching filled "published coins · 49.5 · 9.5 · 74.5 %" that 13 left. At "There is a test" a frame round the chance labels; at "watching a settled crowd move" every arrow's shaft flashes | triangle and chances from facts | |
| 14_2 | Go back to the round coins, and take the mix's settled crowd, scaled up to one thousand four hundred eighteen players so that every count is whole. Four hundred ninety stand at corner zero, three hundred sixty at corner one, and five hundred sixty-eight at corner two. That's still thirty-four point six percent at corner zero. | At "Go back to the round coins" the badge becomes the outlined "round-number coins · 50 · 10 · 75 % · no handicap". At "scaled up" the panel: "players: 1,418" over the dim "split by the chances (expected numbers)". As each count is said, its row ("corner 0 490", "corner 1 360", "corner 2 568") and a teal puddle growing at its corner to its share. At "still thirty-four point six percent" the shares 34.6, 25.4, 40.1 % beside the counts, 34.6 % framed | puddles and counts from the model | |
| 14_3 | Look at one edge at a time. Between corners zero and one, one hundred forty-seven players step forward, thirty percent of the four hundred ninety. And one hundred thirty-five step back, thirty-seven and a half percent of the three hundred sixty. | Edge 0 → 1's two arrows flash. At "one hundred forty-seven players step forward" a teal stream along the forward arrow, as wide as the players it carries, and "30 % × 490 = 147"; at "one hundred thirty-five step back" the back stream and "37.5 % × 360 = 135" | streams from the model | beat |
| 14_4 | The other two edges work the same way. Two hundred twenty-five forward and two hundred thirteen back between corners one and two. Three hundred fifty-five forward and three hundred forty-three back between corners two and zero. | The two sums give way to a table headed "edge" and "forward − back", its first row "0 → 1 147 − 135". As each edge is said, its two streams and its row: "1 → 2 225 − 213", "2 → 0 355 − 343" | table and streams from the model | |
| 14_5 | Before going on, count the surplus on each edge, the players going forward minus the players coming back. | A frame on the "forward − back" head, then round all three pairs. Nothing is subtracted on screen: the viewer counts | counting time | hold |
| 14_6 | Twelve, every time. Twelve more players go forward than come back, on all three edges, so the crowd keeps circulating forward around the triangle. That surplus is the net flow. | "= 12" in gold on each row, one at a time. At "keeps circulating forward" the teal streams go and gold paths run forward along all three edges. At "That surplus is the net flow" the third column is headed "net flow" and the three twelves are framed | paths from the model | |
| 14_7 | Every forward step wins a dollar, and every backward step loses one. So how many dollars does this whole crowd gain in one round? ⟨predict⟩ | The gold paths flash at "wins a dollar". "how many dollars does this whole crowd gain in one round?" under the table, and the countdown; nothing is summed on screen | think countdown | |
| 14_8 | Thirty-six dollars, twelve for each of the three edges. Seven hundred twenty-seven players win, and six hundred ninety-one lose. Shared among the one thousand four hundred eighteen players, that's just over two and a half cents a round each, the mix's average once again. | The question and countdown give way to "3 × 12 = 36 = 727 − 691" in gold with "dollars a round", framed at "Seven hundred twenty-seven players win". At "Shared among" "36 dollars / 1,418 ≈ 2.54 cents" with "per player per round", framed at "the mix's average once again" | TeX from the model | hold |
| 14_9 | Why twelve on every edge? Watch corner one. Across the edge behind it, twelve more players arrive than leave. Across the edge ahead of it, twelve more leave than come back. Twelve in and twelve out, so corner one keeps its three hundred sixty. If the two edges disagreed, the count at corner one would change, and in a settled crowd no count changes. | The sums go; corner 1 is ringed. At "the edge behind it" a frame on row "0 → 1" pointing at its gold path; at "the edge ahead of it" a frame on row "1 → 2" and its path. At "keeps its three hundred sixty" "corner 1: 360 + 12 − 12 = 360" in teal; at "If the two edges disagreed" a frame on corner 1's count | shape emphasis only | beat |
| 14_10 | Remember, these are expected numbers, the crowd split exactly by the chances. Real players toss real coins. Here is one real round for all one thousand four hundred eighteen, tossed with the video's one fixed seed. Its edges carry seven, four and minus seventeen, and each corner changes by exactly the difference between its two edges. In this round the crowd even lost six dollars. | A frame on the dim "split by the chances (expected numbers)". At "Here is one real round" it gives way to "one real round: real tosses, seeded illustration" in teal, and the table's pairs, nets and gold paths to the real round's pairs "134 − 127", "233 − 229", "339 − 356". At "Its edges carry" "= 7", "= 4", "= −17" in teal. At "each corner changes" the counts become 466, 363, 589 with "−17 − 7 = −24", "7 − 4 = 3", "4 − (−17) = 21" in place of the shares, and the puddles follow. At "lost six dollars" "7 + 4 − 17 = −6 dollars" in red | real tosses from the model's seed | hold |
| 14_11 | Over a thousand real rounds, though, each edge averages eleven point nine, and the crowd gains thirty-five point eight dollars a round. Real rounds scatter, and in the long run they average out to the expected numbers. | The expected numbers return together: the dim label, the pairs, the gold twelves, the counts, the shares and the puddles. Under them in teal "1,000 real rounds, the same seeded illustration:", then, as each is said, "average net on each edge: 11.9, 11.9, 11.9" and "average gain: 35.8 dollars a round". At "Real rounds scatter" a frame on the expected-numbers label | real tosses from the model's seed | hold |
| 14_12 | So in any settled crowd, every edge carries the same net flow, and the crowd gains three times it each round. A game is fair exactly when that net flow is zero. Game B's settled twenty-six show it, one and one, three and three, nine and nine. | The thousand-round lines go; a frame on the twelves. At "three times it" "gain per round = 3 × net flow" in gold; at "A game is fair exactly when" "fair ⟺ net flow = 0". At "Game B's settled twenty-six" the mix's title, chances, counts, shares and pairs become game B's: "players: 26", 10, 4, 12 at 38.5, 15.4, 46.2 %, "1 − 1", "3 − 3", "9 − 9", each "= 0", while the puddles move to B's shares; the zeros framed | B's crowd from the model | hold |
| 14_13 | When the net flow is zero, every edge balances on its own. For game B, ten players times a tenth equals four players times a quarter, one each way. Four times three quarters equals twelve times a quarter, three each way. And twelve times three quarters equals ten times nine tenths, nine each way. | The table's head becomes "each edge balances: forward = back". As each edge is said, its balance as separate pieces, "10 × 1/10 = 4 × 1/4 = 1", "4 × 3/4 = 12 × 1/4 = 3", "12 × 3/4 = 10 × 9/10 = 9" (counts teal, forward chances green, back chances dim), while that edge's two arrows flash | pieces from the model | beat |
| 14_14 | Multiply the three balances together. Each side then holds the same three counts, ten, four and twelve, once each, so they cancel. What's left is a tenth times three quarters times three quarters, against nine tenths times a quarter times a quarter. Both come to nine over one hundred sixty. | The "= 1", "= 3" and "= 9" go; "×" signs and a rule line make the rows one product. At "the same three counts" a frame on the left counts, then on the right counts; at "so they cancel" the individual row equals signs and edge-balance heading fade first, then each count gets a red stroke and goes. The two factor columns are not separate equalities; only their full products are equal. At "What's left" copies of the forward chances fly from their rows into "1/10 · 3/4 · 3/4"; at "against nine tenths" the back chances into "= 9/10 · 1/4 · 1/4". At "Both come to" "9/160 = 9/160" in gold, framed | separate TeX pieces move as wholes | hold |
| 14_15 | The same cancelling works for any game. Write each corner's weight, its share of the settled crowd, as w, its chance forward as p, and back as q. When no coin is certain to win or lose, every corner holds some of the crowd, so no weight vanishes, and each one cancels. What's left says winning three times in a row, all the way around the triangle, is exactly as likely as losing three times in a row the other way. | B's numbers go. At "Write each corner's weight" the head becomes "w: weight · p: chance forward · q: chance back" and the rows "w₀ p₀ = w₁ q₁", "w₁ p₁ = w₂ q₂", "w₂ p₂ = w₀ q₀". At "no weight vanishes" every corner is ringed; at "each one cancels" the individual row equals signs fade before every weight gets a red stroke. Only the complete products below are equated. At "winning three times in a row" "p₀ p₁ p₂" with "win, win, win" and teal paths forward round the triangle; at "the other way" "= q₀ q₁ q₂" with "lose, lose, lose" and dim paths back | TeX pieces, paths | hold |

### Notes for 14

- Order (root's direction, ROOT1232-P14): the crowd of 1,418 and its paired
  opposing flows come before any symbol. The viewer counts the three
  surpluses (14_5, held, nothing subtracted on screen) before 14_6 names
  the net flow, and predicts the crowd's gain (14_7) before 14_8 sums it.
- 14_9 is why a settled corner forces its two edges to agree. Corner 1
  receives 147 forward from corner 0 and 213 back from corner 2 (360) and
  sends 225 forward and 135 back (360): the edge behind it carries
  147 − 135 = 12 net in and the edge ahead 225 − 213 = 12 net out.
- Expected against realized: the dim "split by the chances (expected
  numbers)" is on screen whenever expected counts are. The real round
  (14_10) has its own teal label, "one real round: real tosses, seeded
  illustration", and the thousand rounds say "the same seeded
  illustration". They come from `model::real_crossings` (seed 1999, the
  players in corner order, a toss below the chance in 200ths steps
  forward). One illustration, not a typical value or a theorem; a single
  round can lose (−6 dollars).
- Negative controls (oracle): the real round's edges disagree (7, 4, −17),
  so "every round nets twelve" is false, while each corner still changes
  by net in minus net out; B's 10, 4, 12 under the mix's coins (not
  settled) has edges that disagree, and its corners change by the same
  rule; 3 × 12 = 36 = 727 − 691.
- 14_13 to 14_15 prove one direction only: zero net flow forces equal
  products. 15 opens with the converse, so nothing is concluded from
  equal products before 15_6.
- 14_15's "no weight vanishes" is what lets each weight cancel; with every
  chance strictly between 0 and 1 every corner holds players (15_1 states
  the condition).
- The badge and chances move together: published mix 29.5/62/62 % at
  14_1, as 13 left it; at 14_2 ("Go back to the round coins") the badge
  and chance labels crossfade together to the round mix 30/62.5/62.5 %.
  The outlined round-number badge remains through 15_13.
- Pause intentions: a short beat after "Here's a new question"; the
  counting hold after "the players coming back" (14_5); the prediction
  after 14_7, answered at 14_8's first words; "Twelve, every time." then a
  short beat; an inspection hold after "the mix's average once again"
  (14_8); a beat after "no count changes" (14_9); a turn after "Real
  players toss real coins" (14_10) and an inspection hold after "lost six
  dollars".
- Pronunciation: "one thousand four hundred eighteen", "five hundred
  sixty-eight", "thirty-seven and a half", "minus seventeen", "thirty-five
  point eight", "nine over one hundred sixty".

## 15_two_products: Two products decide (~293 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 15_1 | So when the net flow is zero, the two products match. The test also runs the other way, as long as no coin is certain to win or certain to lose, and none of the coins in this video is. | B's triangle and settled crowd as 14 left them: the outlined round-number badge, B's violet title, its chances, teal puddles at 10, 4 and 12 of 26. At "when the net flow is zero" "net flow = 0 ⇒ p₀ p₁ p₂ = q₀ q₁ q₂"; at "the other way" the reverse claim "p₀ p₁ p₂ = q₀ q₁ q₂ ⇒ net flow = 0" with a "?" in hero. At "no coin is certain" the dim condition "every chance strictly between 0 and 100 %"; at "none of the coins" a frame round the chance labels | claims as TeX | |
| 15_2 | Try it on game B, whose products match, and pretend we never found its settled crowd. Put ten players at corner zero. A tenth of them step forward, so the first balance needs one player stepping back from corner one, where a quarter step back. That takes four players. Then three of those four step forward, so the second balance needs three stepping back from corner two, and that takes twelve. | A frame on the claim with its "?". At "pretend we never found" the puddles drain. At "ten players" edge 0 → 1's name, the dim "split by the chances (expected numbers)" and "10", while corner 0's puddle fills to ten players. Piece by piece as the words reach them: "× 1/10" (the forward arrow flashes), "= ? × 1/4 = 1" (the back arrow flashes), the "?" becoming "4" at "That takes four players" while corner 1's puddle fills. The second row starts from that 4, carried down from the first: "4 × 3/4 = ? × 1/4 = 3", the "?" becoming 12 and corner 2's puddle filling | each count computed from the one before | |
| 15_3 | The third balance has nothing left to choose. Twelve times three quarters is nine, and ten times nine tenths is nine, so it holds. And ten, four and twelve is exactly the crowd game B settled into. | Row three: the 12 carried down from row two and the 10 from row one, "12 × 3/4 ? 10 × 9/10"; edge 2 → 0's arrows flash at each "nine". At "so it holds" the "?" becomes "=" and "= 9" appears. At "exactly the crowd game B settled into" a frame on 10, 4 and 12, and all three corners are ringed | pieces from the model | beat |
| 15_4 | Why did the third balance come out right? Follow the same steps for any game. The first balance gives corner one's weight from corner zero's, and the second gives corner two's from corner one's. Dividing by the back chances is safe, because no coin is certain to win. | A frame on the third row. At "Follow the same steps" the numbers become letters, "w₀ p₀ = w₁ q₁", "w₁ p₁ = w₂ q₂", "w₂ p₂ = w₀ q₀", under "w: weight · p: chance forward · q: chance back". At "corner one's weight" "w₁ = w₀ p₀ / q₁" and a frame on the first balance; at "corner two's" "w₂ = w₁ p₁ / q₂ = w₀ p₀ p₁ / (q₁ q₂)" and a frame on the second; at "is safe" a frame on the condition | TeX derivation | |
| 15_5 | Put both into the third balance. Corner zero's weight appears on both sides and cancels. Multiply through by the two back chances, and what's left is exactly the two products. So the third balance holds exactly when the products match, as game B's do. | A frame on the third balance; under the steps it becomes "w₀ · p₀ p₁ p₂ / (q₁ q₂) = w₀ · q₀", and at "cancels" both w₀ get a red stroke. At "Multiply through" "p₀ p₁ p₂ = q₀ q₁ q₂" in gold, framed at "holds exactly when the products match" | TeX derivation | hold |
| 15_6 | A crowd that balances every edge stays put: each corner gets back exactly the players it sends. And only one way of spreading a crowd stays put, the settled one, because players can get from any corner to any other. So matching products mean the net flow is zero, and the game is fair. | The steps go; the products line rises. "every edge balanced → the crowd stays put"; at "gets back exactly" all three corners are ringed. At "only one way" "only the settled crowd stays put: any corner reaches any other"; at "any corner to any other" every arrow's shaft flashes. At "So matching products" "so equal products → net flow 0 → fair", and the reverse claim loses its "?", gold and framed | argument in words | hold |
| 15_7 | When the products don't match, which way does the crowd turn? Keep the net flow in each balance: forward minus back is the net flow, on every edge. The first balance gives corner one's back flow. Put it into the second, multiplied first by corner one's back chance, and because each corner's two chances add up to one, the net flow appears just once. Put that into the third, multiplied first by both back chances, and everything collects into one line. | The chain and the products line go. At "Keep the net flow" the balances become "w₀ p₀ − w₁ q₁ = net", "w₁ p₁ − w₂ q₂ = net", "w₂ p₂ − w₀ q₀ = net". At "back flow" "w₁ q₁ = w₀ p₀ − net" with a frame on the first; at "Put it into the second" "w₂ q₁ q₂ = w₀ p₀ p₁ − net (p₁ + q₁)" with a frame on the second; at "add up to one" it becomes "w₂ q₁ q₂ = w₀ p₀ p₁ − net" beside the dim "p₁ + q₁ = 1". At "one line" "net × (p₂ + q₁ q₂) = w₀ × (p₀ p₁ p₂ − q₀ q₁ q₂)" as separate pieces, framed | TeX derivation | hold |
| 15_8 | Look at the two sides. The number multiplying the net flow is a chance forward plus two chances back multiplied together, so it's positive. Corner zero's weight is positive too. So the net flow has the same sign as the forward product minus the backward product. When the forward product is bigger, the crowd turns forward and the game wins. When it's smaller, the game loses. | Frames on the line's left side, then its right. At "so it's positive" a green "positive" under "× (p₂ + q₁ q₂)", framed; at "positive too" a green "positive" under "w₀", and corner 0 ringed. At "the same sign" frames on "net", then on the products' difference. At "bigger" "forward product bigger → net flow above 0 → wins" in green, with green paths forward round the triangle; at "smaller" "forward product smaller → net flow below 0 → loses" in red, with red paths back. The two claims at the top give way to one rule, "the net flow has the sign of p₀ p₁ p₂ − q₀ q₁ q₂", framed | TeX, paths | hold |
| 15_9 | For game B with the round coins, the two products were equal, so B's net flow is zero and B is fair. Averaging B's forward chances would have said fifty-three point three percent, a winner. The products aren't fooled. | The derivation goes; the rule, the condition and the one line stay. "game B" with "1/10 · 3/4 · 3/4 = 9/160 = 9/10 · 1/4 · 1/4"; at "B is fair" "net flow = 0 fair" in gold. At "Averaging" the dim "averaging B's forward chances: 53.3 %, a winner?"; at "aren't fooled" a red stroke through it | TeX from the model | beat |
| 15_10 | For the mix, the forward product is thirty percent times sixty-two and a half percent twice over. The backward product is seventy percent times thirty-seven and a half percent twice over. Which is bigger? ⟨predict⟩ | B's lines go. B's title and chances become the mix's (forward 30, 62.5, 62.5 % in gold, back 70, 37.5, 37.5 % in dim) while the puddles move to the mix's shares. "forward: 30 % · 62.5 % · 62.5 %" in green with a frame on the forward chances; at "The backward product" "back: 70 % · 37.5 % · 37.5 %" in dim with a frame on the back chances; no values. At "Which is bigger" the countdown | think countdown | |
| 15_11 | The forward product. Seventy-five over six hundred forty, against sixty-three over six hundred forty. So the mix's crowd turns forward, and the mix wins. | The countdown goes; a frame on the forward product. At "Seventy-five" "= 75/640" in gold after it; at "against sixty-three" "= 63/640" after the backward one. At "turns forward" gold paths forward round the triangle | TeX from the model | beat |
| 15_12 | And the one line checks out on the mix's crowd of one thousand four hundred eighteen. Its positive number comes to forty-nine sixty-fourths, and its products differ by three over one hundred sixty. Twelve players of net flow times forty-nine sixty-fourths, against four hundred ninety players at corner zero times three over one hundred sixty. Both sides come to one hundred forty-seven over sixteen. | In teal, "the mix's 1,418 players: 490 at corner 0, net flow 12", and a frame on the one line. At "positive number" "p₂ + q₁ q₂ = 62.5 % + 37.5 % · 37.5 % = 49/64"; at "differ" "p₀ p₁ p₂ − q₀ q₁ q₂ = 75/640 − 63/640 = 3/160". At "Twelve players" "12 × 49/64" in teal over "net × (p₂ + q₁ q₂)"; at "against four hundred ninety" "= 490 × 3/160" over the other side. At "Both sides" "= 147/16" in gold, framed | numbers set over their letters | hold |
| 15_13 | Spread over the players and tripled, the net flow gives the mix's average exactly, just over two and a half cents a round. | The difference gives way to "3 × 12 / 1,418 = 18/709 dollars ≈ 2.54 cents" in gold with "per player per round", framed at "average exactly". At "two and a half cents" the "precisely" card (K06), held for its reading time | lower-third card | |
| 15_14 | Now shave the half point off again, back to the coins Harmer and Abbott published. Game A's forward product falls below its backward one, so A loses, a cent a round. Game B's falls below as well, so B loses, just under a cent. The mix's forward product stays above, so the mix wins, just over one and a half cents a round. Two products per game decide A, B and the mix. | The mix's numbers go. At "Harmer and Abbott published" the badge becomes the filled "published coins · 49.5 · 9.5 · 74.5 %", the mix's chances become 29.5, 62, 62 % forward and 70.5, 38, 38 % back, and the puddles barely move (34.6 to 34.5 % at corner 0). A table headed "products: forward vs back" and "average", over "average: cents per player per round"; a row as each game is said: "game A 0.1213 < 0.1288 loses −1.00", "game B 0.0527 < 0.0588 loses −0.87", "the mix 0.1134 > 0.1018 wins +1.57". At "Two products per game decide" a frame on the products column | table from facts | hold |
| 15_15 | Notice what the test never needed. No crowd and no weights, just the three chances forward and the three chances back, each set multiplied together. | A frame on the rule. At "No crowd and no weights" the puddles drain. At "the three chances forward" gold paths forward round the triangle and a frame on the forward chances; at "the three chances back" dim paths back and a frame on the back chances; at "each set multiplied together" a frame on the products column | paths | hold |

### Notes for 15

- Both directions, every chance strictly between 0 and 1 (root's
  direction, ROOT1232-P14). The converse is earned on the picture before
  it is used: 15_2 and 15_3 build game B's crowd from ten players at
  corner 0, one balance at a time (10, then 4, then 12), and the third
  balance holds with nothing left to choose. 15_4 and 15_5 do the same
  steps in letters; w₀ cancels and the third balance holds exactly when
  the products match. 15_6 says why a crowd that balances every edge is
  the settled crowd: it stays put, and the settled crowd is the only crowd
  that stays put, because every corner reaches every other.
- The sign is earned, not read off the zero case. With the net flow J kept
  in each balance (the same J on every edge, from 14): w₁q₁ = w₀p₀ − J;
  then w₂q₁q₂ = w₀p₀p₁ − J(p₁ + q₁) = w₀p₀p₁ − J; then the third balance
  gives J(p₂ + q₁q₂) = w₀(p₀p₁p₂ − q₀q₁q₂). Both p₂ + q₁q₂ and w₀ are
  positive, so J has the sign of the difference of the products. The
  oracle checks the identity, the sign and the converse on all 729 games
  with chances in tenths.
- No spanning-tree formula. The weight was named in 09 and is said again
  in 14_15 ("its share of the settled crowd"); the positive number is
  described in words in 15_8 ("a chance forward plus two chances back
  multiplied together") before 15_12 gives its value.
- K06: "if every chance is strictly between 0 and 1, the average gain per
  player per round is 3 w0 (p0 p1 p2 − q0 q1 q2) / (p2 + q1 q2), 3 times
  the net flow per player, where w0 is corner 0's share of the settled
  crowd". With the mix: 3 × (245/709) × (3/160) / (49/64) = 18/709.
- Units: the crowd's gain is in dollars a round (14_8); the one line in
  15_12 counts players on both sides (twelve players of net flow, 490
  players at corner 0, so 147/16 players); a player's average is in cents
  per player per round (15_13, the table in 15_14).
- The condition is what the argument uses: 15_4 divides by the back
  chances, and 15_6 needs every corner to reach every other. The voice
  claims the test only under it. Tested at its edge (the oracle sweeps
  every chance from 0 to 10 tenths): on this triangle the sign rule also
  survives certain coins, because whenever the products differ both other
  factors are positive. The one game shape where p₂ + q₁q₂ vanishes (a
  coin certain to win at corner 1 and one certain to lose at corner 2) has
  both products 0 and no net flow. So the video never says the test fails
  without the condition.
- Negative controls (oracle and the tests): that certain-coin game (50 %
  at corner 0, 100 % at corner 1, 0 % at corner 2) makes p₂ + q₁q₂ and w₀
  both 0, so the one line alone decides nothing there; a
  bigger difference of products need not mean a bigger gain (forward
  chances 1, 8, 9 tenths: difference 0.054, mean ≈ 0.0775; 4, 4, 8
  tenths: difference 0.056, mean ≈ 0.0764), so the products give the sign
  only; B's forward chances average 53.3 %, yet B is fair; when the
  products differ, a crowd built from the first two balances fails the
  third and moves.
- The table (published coins): A 0.1213 against 0.1288 (loses −1.00
  cents), B 0.0527 against 0.0588 (loses −0.87), the mix 0.1134 against
  0.1018 (wins +1.57); values from facts. The published mix's crowd sits
  at 34.5 % at corner 0, so its puddles barely move from the round mix's
  34.6 %.
- The badge: outlined round-number coins to 15_13; the filled published
  badge returns at 15_14, on "the coins Harmer and Abbott published".
- Pause intentions: a turn before "Try it on game B"; a beat after "so it
  holds" (15_3); a thinking beat after "Why did the third balance come out
  right?"; inspection holds after 15_5, 15_6, 15_7 and 15_8, where each
  step of the argument completes; the prediction after 15_10, answered
  at 15_11's first words; the card's reading time in 15_13; a final hold.
- Pronunciation: "nine tenths", "fifty-three point three", "seventy-five
  over six hundred forty", "sixty-three over six hundred forty",
  "forty-nine sixty-fourths", "three over one hundred sixty", "one
  hundred forty-seven over sixteen", "Harmer and Abbott".

## 16_when_switching_fails: When switching fails (~307 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 16_1 | Up to now, a fair coin chose the game before every round, and the mix won. But tossing a coin is only one way to switch between two games. Does it matter in what order the games are played, or only how often each one is? | From an empty frame: the filled badge "published coins · 49.5 · 9.5 · 74.5 %"; B's triangle with the gold title "the mix", the mix's chances 29.5, 62, 62 % forward (gold) and 70.5, 38, 38 % back (dim), and the teal puddles at the mix's settled 34.5, 25.4, 40.1 %. At "a fair coin chose the game" the panel's first line, "random: a fair coin picks A or B each round", with a small gold coin; at "Does it matter" a frame round that line while the coin pulses | badge, triangle, puddles | |
| 16_2 | Here is the simplest fixed order: A, then B, then A, then B, forever. Half the rounds are A and half are B, exactly as in the mix. Only the order has changed. | At "the simplest fixed order" the strip "A B A B A B A B A B A B …" (A blue, B in B's color) and its name "strict alternation", under the random line. The mix's six chances go and the title becomes "A B A B …". At "Half the rounds" a frame round the strip | strip | |
| 16_3 | Same two games, same coins, the same half and half. So does strict alternation win, like the coin-toss mix? ⟨predict⟩ | "does A B A B … win?" under the strip; the countdown, right of the triangle | think countdown | |
| 16_4 | It loses, about two thirds of a cent a round. The order alone has turned a win into a loss. To see why, watch the crowd. | The "?" goes; beside the strip's name, "loses: −0.67 cents a round" in red. At "The order alone" a frame round the strip | verdict from facts | |
| 16_5 | Game A's rounds cost one cent each, wherever the crowd stands, because A's coin never looks at the money. So everything rides on B's rounds, and on the dial from before: the share of the crowd on corner zero when B plays. B breaks even at about thirty-seven point seven percent. Below that line B's round wins, and above it B's round loses. | At "cost one cent each" "each A round: −1.00 cent" in blue under the strip. At "the share of the crowd on corner zero" the chart: "corner 0 when B plays" from 30 % to 40 % against "B round" 1 to 12. At "breaks even" its dashed line at 37.7 % labelled "B breaks even: 37.7 %" under the line at the right; at "Below that line" a gold "B wins" under it, at "above it" a red "B loses" above it | chart, line from facts | |
| 16_6 | Start the crowd where game B alone leaves it, with thirty-eight point four percent on corner zero, and alternate. The first B round meets only thirty point seven percent there, and wins. But each time B comes back, a little more of the crowd is waiting on corner zero. The dots climb past the line and settle at thirty-eight percent, just above it. From then on, every B round loses. | At "where game B alone leaves it" the puddles move to B's settled 38.4, 15.4, 46.2 % ("from game B's settled crowd" under the triangle). A frame steps along the strip as each round is played, the counter "round 1 of 24" counting, and the puddles move round by round (computed: A's coin on A rounds, B's on B rounds). Before every B round a teal dot lands on the chart at corner 0's share. The first, at 30.7 % ("30.7 %" beside it), stands before the next round. The later rounds run faster; the dots climb (34.3, 36.3, 37.2 … %) past the line and settle at 38.0 % ("settles at 38.0 %" above the line). At "every B round loses" a frame round the dots above the line | live counter, computed rounds, dots | |
| 16_7 | Why does alternating fill corner zero for B? Follow one cycle. B's bad coin knocks most of corner zero back to corner two, and its good coin carries most of corner one forward to corner two. So each B round leaves about sixty-nine percent of the crowd on corner two, one step behind corner zero. Then A plays, and its coin sends just under half of them forward, onto corner zero, while a few step back from corner one. Almost thirty-eight percent are waiting on corner zero when B plays again. | The settled cycle, the strip's frame on a B. At "knocks most of corner zero back" a red stream from corner 0 back to corner 2, at "carries most of corner one forward" a green stream from 1 to 2, and the puddles move to 23.5, 7.6, 68.9 %; "B leaves on corner 2: 68.95 %" in the panel. The frame moves to an A. At "sends just under half of them forward" a blue stream from 2 to 0 and "49.5 % × 68.95 % ≈ 34.13 %"; at "a few step back" a dim stream from 1 to 0 and "50.5 % × 7.59 % ≈ 3.83 %"; at "Almost thirty-eight percent" the puddles move back and "34.13 % + 3.83 % = 37.96 % on corner 0 when B plays", an arrow to the chart's settled dots | streams, sums from facts | |
| 16_8 | It's tempting to take a shortcut and average the two games' chances at each corner. That gives twenty-nine and a half percent at corner zero and sixty-two percent at the others, exactly the coin-toss mix's coins, so it predicts the mix's win of just over one and a half cents. For random switching, averaging works because a fresh choice of game is independent of where the player stands. In strict alternation the next game is already fixed. Its crowd depends on the previous rounds, so the order decides where the crowd stands when the bad coin comes up. | The streams and sums go. "averaging A's and B's chances at each corner:" "29.5 %, 62 %, 62 %: the coin-toss mix's coins", "predicts +1.57 cents a round"; at "the next game is already fixed" a red strike through the prediction and "A B A B … actually: −0.67" in red | strike, values from facts | |
| 16_9 | Now play two A's, then two B's, the pattern Harmer and Abbott simulated. Two A rounds in a row spread the crowd out again before B returns. So once the crowd settles, the first B of each pair meets about thirty-six percent on corner zero, and the second only about thirty-three, both below the line. Two and two wins, about one and a half cents a round, a little less than the coin-toss mix. | The control's lines go; the alternation's dots dim. The strip becomes "A A B B A A B B A A B B …", the title "A A B B …". From B's settled crowd again, 24 computed rounds with the frame stepping and the counter; before every B round a gold dot. They stay below the line from the first and soon alternate near 36.0 % and 33.3 % ("36.0 %" and "33.3 %" beside the settled pair). At "Two and two wins" "wins: +1.47 cents a round" in gold beside the strip | computed rounds, dots | |
| 16_10 | Put the three side by side, per round played. The coin-toss mix wins one point five seven cents. Strict alternation loses zero point six seven. Two and two wins one point four seven. Same games, same share of each: only the order changed, and with it where the crowd stands when B plays. | The chart and the strip go. A table: "schedule", "corner 0 when B plays", "cents a round"; a row as each is said: "random (a fair coin) 34.5 % +1.57", "A B A B … 38.0 % −0.67", "A A B B … 36.0 %, 33.3 % +1.47", with "per round played, A's rounds included" under it. At "where the crowd stands" a frame round the middle column | table from facts | |
| 16_11 | Now the control. Replace A and B by the two ordinary coins, forty-nine and forty-eight percent, which never look at the money. Toss for the game, alternate, or play two and two: each loses exactly three cents a round. With coins like these, only how often each is tossed can matter, never the order. | At "Now the control" the triangle and the puddles go. At "the two ordinary coins" two plain dim coins at the right, "49 %" and "48 %", and "ordinary coins (they ignore the money)" over a new column; a cell as each schedule is said: "−3.00", "−3.00", "−3.00". At "never the order" a frame round the new column | table from facts | |
| 16_12 | The share of each game matters too. Let a biased coin choose A on only one round in twenty, and the mix loses, about a quarter of a cent a round. Choose A on nine rounds in ten, and it loses again, about a third of a cent. | The table and the two coins go. The curve's axes: "A's share of the rounds (chosen at random)" from 0 to 100 %, "cents a round" from −1.2 to +2.0, the zero line; the ends "B alone −0.87" at 0 % and "A alone −1.00" at 100 %, and the coin-toss mix "+1.57" at 50 % in gold. At "one round in twenty" a red point at 5 %, "−0.23"; at "nine rounds in ten" a red point at 90 %, "−0.34" | axes, points from facts | |
| 16_13 | Compute the mix at every share in between, and the curve rises above zero and comes back down. The mix wins from about seven percent to about eighty-five percent. At each crossing the two products from before are equal, and between the crossings the forward product is the bigger one. | The curve draws from 0 % to 100 %, computed at every half percent, through all five points. At "about seven percent" a tick "≈ 7 %" where it crosses zero, at "about eighty-five" "≈ 85 %". At "between the crossings" a gold band along the zero line between the ticks, "forward product bigger: the mix wins" | curve from the model | hold |
| 16_14 | Those edges are not round numbers. The crossings sit at seven point zero three and eighty-four point seven one percent, rounded, and the mix wins only strictly between them. At exactly seven percent it still loses, by a hair, and at eighty-five percent it has already stopped winning. | At "seven point zero three" the ticks become "7.03 %" and "84.71 %". At "strictly between them", under the curve, a dim "cents a round:" and the grid's edges: "7.0 %: −0.0034 loses", "7.1 %: +0.0074 wins", "84.7 %: +0.0005 wins", "84.8 %: −0.0058 loses"; at "exactly seven percent" a frame round the first, at "eighty-five percent" round the last two, the last win and the first loss, both below 85 %. The "precisely" card (K07), held for its reading time | values from facts, lower-third card | |
| 16_15 | So no general rule says that switching between losing games wins. Here it wins for some orders and some shares, with these coins, because one game reads the money and the other moves it around. | At "no general rule" the curve and all its labels fade. At "some orders and some shares" B's triangle returns with the gold title "the mix", the mix's chances and its settled puddles (34.5, 25.4, 40.1 %); at "one game reads the money" a frame round the red corner 0 | callback | hold |

### Notes for 16

- Order (root's direction, ROOT1428-P16): the actual AB evolution (16_6)
  and its mechanism (16_7) come before the rate table (16_10), and so does
  AABB's (16_9). The prediction (16_3) is answered at 16_4's first words.
- The schedules are played round by round, never by averaged chances. The
  puddles follow `model::evolve_f64` with A's coin on A rounds and B's on B
  rounds, from B's settled crowd (38.4 % on corner 0); the dots are the
  computed corner-0 shares before each B round: 30.7, 34.3, 36.3, 37.2,
  37.7 (37.65), 37.8 (37.82) … to 38.0 % (37.96 exactly in the cycle). The
  cycle's shares come from `model::cycle` (exact): before A 23.47, 7.59,
  68.95 %, before B 37.96, 46.43, 15.61 %.
- Negative control, the tempting shortcut (16_8): averaging the two games'
  chances at each corner gives 59/200, 31/50, 31/50, which are the random
  mix's coins exactly, and they predict +1.57 cents; AB's real average is
  −0.67. Averaged coins describe a coin toss before every round, not a
  schedule.
- Edges of the sentences: "each time B comes back, a little more … is
  waiting" is checked exactly over the first 60 B rounds (strictly
  increasing). "From then on, every B round loses" holds from the sixth B
  round: its share is above 49/130, and the oracle shows that the crowd's
  total-variation distance to the cycle (0.18 points) is smaller than the
  cycle's margin over the line (0.27 points), and that distance never grows
  under a round of either game. The visible dots do not separate the fifth
  (37.65 %) from the line (37.69 %), so the voice names no round.
- A's rounds cost exactly one cent each whatever the shares (A's coin is
  the same at every corner); the B round's result is one line in corner
  0's share, wins below 49/130 = 37.7 % and loses above (12's dial).
- 16_7's sums are exact values rounded to 0.01 %, and the rounded parts add
  to the rounded total: 49.5 % × 68.95 % ≈ 34.13 %, 50.5 % × 7.59 % =
  3.83 %, 34.13 % + 3.83 % = 37.96 % (the oracle checks the printed
  arithmetic).
- AABB from B's settled crowd: its B rounds meet 34.74, 34.49 %, then
  35.78, 33.54 % … to 36.03 and 33.31 % in the cycle; all below the line,
  from the first. Harmer and Abbott's figure simulates AABB repeated and
  random switching (brief); the voice attributes nothing more.
- The table (16_10), per round played: random +1.57 (B rounds meet
  34.5 %), AB −0.67 (38.0 %), AABB +1.47 (36.0 and 33.3 %), cents a round.
- The control (16_11): with coins that ignore the money each round's
  average is 2p − 1 wherever the player stands, so any schedule with half
  of each loses exactly (−2 − 4)/2 = −3.00 cents; the oracle checks every
  schedule against its count of each coin.
- The curve (16_12 to 16_14): the mix with A chosen by an independent coin
  on a share s of the rounds, `PUBLISHED.mix_share` at every 0.5 % (exact,
  drawn as floats). The crossings are the two roots of p₀p₁p₂ = q₀q₁q₂ in
  s (the oracle counts exactly two in (0, 1) by Sturm's theorem): 7.0314 %
  and 84.7077 %. Rounded crossings are not the exact strict bounds: at the
  0.1 % grid 7.0 % loses (−0.0034 c), 7.1 % wins, 84.7 % wins (+0.0005 c),
  84.8 % loses; 85 % loses (−0.018 c). The best share on that grid is
  40.8 % (+1.66 c), not 50 %; the voice does not say where it peaks.
- K07 (16_14): "published coins, per round played: with A on a share s of
  the rounds, the mix wins exactly when s is strictly between 7.03 % and
  84.71 % (rounded); it loses at 7.0 % and wins from 7.1 % to 84.7 %".
- The badge stays the filled published badge; the ordinary coins of 16_11
  are their own labelled pair (as 04's), not the games' coins.
- Pause intentions: a turn before "But tossing a coin" (16_1); the
  prediction after 16_3, answered at 16_4's first words; a beat after "and
  wins" (16_6) while the first dot stands; a beat after "one step behind
  corner zero" (16_7) before A's round; the shortcut's verdict (16_8) after
  "an averaged coin"; a beat after each row of 16_10; inspection holds after
  16_13 (the curve) and 16_14 (the card's reading time); a final hold.
- Pronunciation: "thirty point seven", "sixty-nine", "twenty-nine and a
  half", "one point five seven", "zero point six seven", "one point four
  seven", "seven point zero three", "eighty-four point seven one",
  "Harmer and Abbott".

## 17_what_winning_promises: What winning promises (~267 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 17_1 | Back to the player. Over the first thousand rounds of its run of game B, the player finished sixteen dollars ahead, even though game B loses. The long run caught up later: by the hundred thousandth round the player was five hundred forty-eight dollars down. | From an empty frame, the filled badge; a window at its own scale, "the player's dollars: the first 1,000 rounds of game B", rounds 0 to 1,000 against dollars −10 to 35 (ticks 0 and 20), the start dashed, "start 7". From "Over the first thousand rounds" the player's teal token rides B's run (the seeded run of 03, every second round drawn) and reaches its end at "sixteen dollars ahead": "+16 dollars after 1,000 rounds" there, and at "even though game B loses" "game B: −0.87 cents a round" under it. At "The long run caught up" a thumbnail of all 100,000 rounds, bottom right, at its own scale, "all 100,000 rounds" over it; at "five hundred forty-eight" "−548 dollars after 100,000" in red under it and a dot at its end | run, token, thumbnail | |
| 17_2 | So what does a winning average promise over a fixed number of rounds? To answer exactly, start a whole crowd at zero dollars, on corner zero, and let every player play the coin-toss mix for a hundred rounds. Dollars can go below zero: a player with minus one dollar simply stands on corner two. | At "over a fixed number of rounds" the run and the thumbnail go. At "start a whole crowd at zero dollars" the axis "dollars won or lost" from −100 to +140 (ticks −100, −50, 0, +50, +100) and a teal dot at 0, "everyone starts here". At "play the coin-toss mix" "a crowd starts at 0 dollars, on corner 0, and plays the mix" at the top left and "rounds played 100" at the top right. At "Dollars can go below zero" a frame round the −100 and −50 ticks; at "minus one dollar" "−1 dollar stands on corner 2" over the axis, ending just left of the start (no tick: one dollar is a twentieth of a unit here) | axes | |
| 17_3 | After a hundred rounds the crowd has spread out. Each bar is one possible total, and its height is the share of the crowd that finishes there. Every bar sits on an even number: each round moves a dollar up or down, so after an even number of rounds every total is even. Together the bars hold the whole crowd. The heights rise and fall with the corners: add up the bars on each corner, and you get the mix's settled crowd to the precision shown, with the most on corner two. | At "spread out" the dot and the notes go and the exact distribution after 100 rounds grows from the axis (teal bars 1 dollar wide on every even total, gaps on the odd ones). At "Each bar is one possible total" "+2 dollars: 10.2 %" beside the tallest bar and a frame round it. At "its height is the share" the axis "share of the crowd" with 0, 5 and 10 %. At "Every bar sits on an even number" "odd totals: no one" and frames round the empty slots at +1 and −1. At "Together the bars hold the whole crowd" "all bars together: 100.0 %". At "rise and fall with the corners" outlines on the bars of corner-2 totals (−4, +2, +8 …); at "add up the bars" "added up by corner:" with "corner 0: 34.5 %", "corner 1: 25.4 %", "corner 2: 40.1 %"; at "the most on corner two" a frame round corner 2's line | histogram from the model, sums from the model | hold |
| 17_4 | Color them by where each player finished. Ahead of the start: fifty-one point seven percent. Behind: thirty-nine point six. Exactly level, back at zero dollars: eight point seven. | The outlines go after 17_3's hold; at "Color them" the bars' words go. As each is said, its bars take its color (an opaque layer over the teal: right of 0 gold, left red, the bar at 0 dim) and its readout comes up under "rounds played 100": "ahead 51.7 %", "behind 39.6 %", "level 8.7 %". At "Exactly level" the "precisely" card (K08), held for its reading time | colored histogram, labels from facts, lower-third card | |
| 17_5 | The crowd's average after a hundred rounds is one dollar and twenty-nine cents up. A hundred rounds at the mix's one point five seven cents would make one fifty-seven; starting on corner zero, where B's bad coin waits, costs about twenty-eight cents, once. And that average hides a lot: almost two in five players are behind. | At "The crowd's average" a white line at +1.29 dollars and the readout "average +1.29 dollars". At "would make one fifty-seven" "100 rounds at 1.57 cents = 1.57 dollars" (gold) right of the bars, and at "costs about twenty-eight cents" "starting on corner 0: −0.28 dollars, once" (red) under it (1.57 − 0.28 = 1.29). At "hides a lot" a frame round the red bars; at "almost two in five" a frame round "behind 39.6 %" | line, sums from facts | |
| 17_6 | Now let the same crowd play a thousand rounds instead of a hundred. Will more of it finish ahead, or less, and how much? ⟨predict⟩ | At "Will more of it finish ahead" the two lines go and "after 1,000 rounds: ahead?" comes up right of the bars; the countdown under the readouts | think countdown | |
| 17_7 | More. Sixty-eight point nine percent finish ahead after a thousand rounds, more than two in three. Watch the same bars, on the same dollar axis, as the rounds run from a hundred to a thousand. | At "Sixty-eight point nine" the question and the "?" give way to "after 1,000 rounds: ahead 68.9 %" in gold. After the complete invitation ending "from a hundred to a thousand", the bars run through the exact distributions after 100, 102, 104 … 1,000 rounds, every even number (one exact crowd in every frame, each bar one total on the same axes), over 10 s; "rounds played" counts with them, the readouts "ahead", "behind", "level" and "average" and the average's line follow; the full ten-second change runs without narration | computed transformation, live readouts | |
| 17_8 | Behind: twenty-eight point seven percent. Level: two point four. The crowd's average has kept growing by about one and a half cents a round, to fifteen dollars and forty-two cents. But the spread of the totals has grown more slowly, from about nine dollars to about thirty. | At "Behind" and "Level" frames round their readouts ("behind 28.7 %", "level 2.4 %"). At "kept growing" "900 more rounds: +14.13 dollars, +1.57 cents a round" right of the bars and a frame round "average +15.42 dollars". At "the spread" a bracket one spread either side of the average, over the bars, and "spread: 9.35 dollars at 100 rounds, 29.55 at 1,000" | bracket from the model | |
| 17_9 | Ten times the rounds, but only about three point one six times the spread. That's close to the square root of ten. For long runs the spread grows like the square root of the number of rounds, while the average grows in step with them. It's a law about the long run, not an exact rule at a hundred or a thousand rounds. | At "Ten times the rounds" the rate line gives way to "10 times the rounds, 3.16 times the spread"; at "close to the square root of ten" "√10 ≈ 3.16" (TeX). At "For long runs" the "precisely" card (K10: the exact 3.1620 against 3.1623, and that the square-root growth is a long-run law), held for its reading time | values from facts, lower-third card | hold |
| 17_10 | Now the player's sixteen dollars. Start a crowd at seven dollars, where the player started, and let it play game B alone for a thousand rounds. Thirty-four point five percent finish ahead, and fourteen point six percent finish at least sixteen dollars up. The player was lucky, but not rare. | At "the player's sixteen dollars" the mix's bars, the bracket, the readouts and the lines go; a teal dashed mark at +16 on the same axis and "the player: +16 dollars" over it. At "Start a crowd at seven dollars" the setup becomes "a crowd starts at 7 dollars, on corner 1, and plays game B alone". At "play game B alone" game B's exact distribution after 1,000 rounds from 7 dollars grows from the axis, colored as before (ahead gold, behind red, level dim; "rounds played 1,000" stays). At "Thirty-four point five percent" "ahead 34.5 %"; at "at least sixteen dollars up" "at least +16 14.6 %" and a frame round the bars from +16 on; at "lucky, but not rare" a frame round the mark's words | histogram from the model, mark | |
| 17_11 | Over a thousand rounds from zero dollars, game A alone leaves sixty-one point two percent of its crowd behind, and game B alone sixty-three point nine percent. The coin-toss mix leaves only twenty-eight point seven percent behind. | The histogram goes. "after 1,000 rounds from 0 dollars, behind:"; a red bar grows as each is said (6 units for the whole crowd): "game A alone 61.2 %", "game B alone 63.9 %", "the mix 28.7 %" (names in their games' colors, the mix gold) | bars from facts | |
| 17_12 | And in the very long run? With probability one, a single player's average per round settles on about the mix's one point five seven cents. Not on average over a crowd: almost every run of one player, if it goes on forever. | At "With probability one", under the rows, "one player, playing forever: average per round → +1.57 cents, with probability one" in gold and the "precisely" card (K09), held for its reading time; at "Not on average over a crowd" a frame round the line | words from facts, lower-third card | |
| 17_13 | So a winning game promises three different things. The crowd's average grows by about one and a half cents a round. The chance of being ahead is just over half after a hundred rounds and more than two in three after a thousand, but at any fixed finite number of rounds it never becomes certain. And one player's own average settles on that cent and a half only in the long run, with probability one. | At "three different things" the rows go. Three lines, each as it is said: "the crowd's average: grows 1.57 cents a round" (gold), "chance of being ahead: 51.7 % after 100 rounds, 68.9 % after 1,000, never certain" (at "never becomes certain" a frame round it), "one player's average per round: → 1.57 cents, with probability one" (teal) | summary from facts | hold |
| 17_14 | These are invented games. What they teach is about chances that depend on where you stand, and none of it is advice about real money. | At "invented games" the lines go and B's triangle returns with its chances; at "depend on where you stand" a frame round the red corner 0 while it pulses | callback | hold |

### Notes for 17

- 17_1 keeps the player's +16 after 1,000 rounds of B as a finite-run
  counterexample: B loses on average (−0.87 cents a round), yet this seeded
  run was 16 dollars ahead at round 1,000 and −548 at 100,000. The window
  is at its own scale (rounds 0 to 1,000), not 03's 100,000-round window.
- Before any long-run language, 17_2 and 17_3 say what the exact
  distributions are: the whole crowd starts at 0 dollars on corner 0 (so
  B's first round in it meets the bad coin); negative totals are allowed
  (−1 leaves remainder 2, K03); after an even number of rounds every total
  is even (odd mass exactly 0, the oracle checks); each bar is one total, 1
  dollar wide on a 2-dollar pitch, its height the share of the crowd there;
  the bars hold the whole crowd (all of it inside the window at 100
  rounds, all but less than 0.01 % at 1,000).
- One axis and one meaning for every histogram: dollars won or lost from
  the start, −100 to +140, and the share of the crowd at that total, 0 to
  11 %. The transformation (17_7) runs through the exact distributions
  after every even number of rounds from 100 to 1,000 (451 of them,
  `model::snapshots` in one pass) with no interpolation: the tracker moves
  in whole steps of two rounds, so every frame is one exact crowd on even
  totals, and the counter, the bars, the readouts and the average's line
  show the same round (a test samples 6,001 times). Every one of the 451
  holds all but at most 0.005 % of the crowd inside the window, and none
  has a bar taller than 10.23 % (the axis runs to 11 %).
- The corners in the bars (17_3): the exact bars after 100 rounds are
  jagged with a six-dollar pattern, because the corner a total stands on
  decides its next coin. Added up by corner (total mod 3), the bars give
  34.51, 25.41, 40.08 % after 100 rounds and after 1,000, the mix's
  settled crowd at one decimal (34.5, 25.4, 40.1 %, the most on corner 2).
  Only the sums are claimed: single corner-2 bars are not all taller than
  their neighbours (after 100 rounds +8, +14, +20, +26 and −22 are not), so
  the outlines mark which bars are added, not a ranking.
- Three things kept apart (17_13): the crowd's average (it grows by the
  mix's 1.57 cents a round, after a one-off cost of 0.28 dollars for
  starting on corner 0; +1.29 and +15.42 dollars); the chance of being
  ahead after a fixed number of rounds (51.7 % and 68.9 %, never certain:
  every finite run can lose every round, and after one round only 29.5 %
  are ahead, so the voice never says it is always better than even); one
  player's long-run average, which converges to 223/14200 dollars with
  probability one (K09).
- The spread is the standard deviation of the totals: 9.35 and 29.55
  dollars (shown 9.3 and 29.6). The ratio 3.1620 is close to √10 = 3.1623
  but not equal; growth like √n is the central limit theorem's long-run
  scaling (Ethier & Lee 2009), not an exact finite identity (K10).
- 17_10: from 7 dollars (corner 1, the player's start), game B alone for
  1,000 rounds: ahead 34.5 %, at least +16 dollars 14.6 % (from 0 it would
  be 12.7 %). Lucky, not rare: roughly one crowd member in seven did as
  well.
- 17_12 draws no run: the seeded mix run of 04 averages +1.748 cents a
  round over its 100,000 rounds, within its luck of 1.57 but visibly off
  it, so a run would look like a contradiction of "settles". The claim is
  K09's theorem.
- K08 (17_4): "exact distribution, not a simulation: a crowd starting at 0
  dollars, on corner 0, negative totals allowed, plays the mix with the
  published coins; after 100 rounds ahead 51.7 %, behind 39.6 %, level
  8.7 %".
- K10 (17_9): "spread = standard deviation of the totals: 9.35 dollars at
  100 rounds, 29.55 at 1,000; ratio 3.1620, square root of 10 = 3.1623;
  square-root growth is a long-run law (central limit theorem: Ethier &
  Lee, 2009)".
- K09 (17_12): "with probability one, one player's average gain per round
  converges to 223/14200 dollars, about 1.57 cents (strong law of large
  numbers for these games: Ethier & Lee, Electron. J. Probab. 14, 2009)".
- Pause intentions: a thought after "even though game B loses." (17_1),
  so the counterexample lands before "The long run caught up later"; a
  turn before "So what does a winning average promise", and a thought
  after its question; a thought after "hold the whole crowd." before the
  corners; inspection holds after 17_3 (the bars' meaning) and 17_4 (the
  card); the prediction after 17_6, answered by 17_7's first
  word; the transformation's own time, the voice silent, after "from a
  hundred to a thousand"; a hold after 17_9 (the card); a beat after "but
  not rare" (17_10); holds after 17_13 and at the end.
- Pronunciation: "fifty-one point seven", "thirty-nine point six", "one
  fifty-seven", "sixty-eight point nine", "twenty-eight point seven",
  "three point one six", "thirty-four point five", "fourteen point six",
  "sixty-one point two", "sixty-three point nine".

## 18_the_catch: The catch (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 18_1 | Back to the puzzle. Game A loses a cent a round, wherever the player stands. | The same left/right game layout of06 returns: A's triangle and player's token, "49.5 %" everywhere and "−1 cent a round"; B arrives in18_2 | callback, reused triangle builders | |
| 18_2 | Game B loses because its crowd piles up at corner zero, where the bad coin waits. | B's triangle; corner 0 at 38.4 %; the 0 ↔ 2 trace from 10 | callback | |
| 18_3 | Flip a fair coin between them, and game A's rounds push the crowd off corner zero. Game B's rounds, played from better corners, now win more than half the time. | A fades to a left explanation panel and gold deciding coin. B's triangle persists; all three crowd areas interpolate to computed mix shares, labelled "long-run corner 0:34.5 %"; "B rounds:52.1 % wins". This compares equilibria, not transient convergence | persistent crowd trackers from facts | |
| 18_4 | One last puzzle, to solve from the picture. Play game B only when the count divides by three, and game A otherwise. | Left panel "B at0; A at1 and2"; the same triangle relabels:0→1 "9.5 %" (red),1→2 and2→0 "49.5 %" (blue). Remove crowd areas before prediction so they do not reveal its result | labels from facts, controlled rule change | |
| 18_5 | Does this way of switching win or lose? ⟨predict⟩ | Question; countdown | think countdown | |
| 18_6 | It loses, and badly, about twenty-eight cents a round. Every coin around the triangle now favors going back. | "−27.6 cents a round" in red; the forward and backward cycles as two loops, the backward one thick | loops, number from facts | |
| 18_7 | Swap the roles, A at corner zero and B everywhere else, and the same two games win about thirty-one cents a round. What decides it is where each coin gets tossed. | The swapped card; 0→1 "49.5 %" (blue), the others "74.5 %" (green); "+31.4 cents a round" in gold | card, labels from facts | |
| 18_8 | For any game drawn on this triangle, the verdict is that comparison. Win, win, win one way round against lose, lose, lose the other. Whichever is more likely is the way the money flows. If the two products tie, the long-run gain is zero. | Forward/backward cycles remain on the same triangle. Left comparison and tie case; precisely card specifies fixed interior probabilities, unit steps and long-run mean. Hold the complete card for its reading budget after fade | cycle paths, exact limitation, reading hold | hold |
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
- Native18 keeps the right triangle and its crowd trackers through the rule changes. At the swap, corner0's red bad-coin tint changes to A's blue. The final names and receipts replace the whole picture and receive their combined15characters/second reading budget after fade. No sound acceptance is inferred from silent scratch cues.
