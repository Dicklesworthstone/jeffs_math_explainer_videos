# Storyboard — Mental Math Tricks That Actually Work (and Why)

The hand-edited source of this video's words and pictures. `python3 -I
tools/storyboard.py mental_math` checks it (gate G2) and writes `script.tsv`
from the words column. Facts: `docs/fact-sheet.md` and `oracle.py` (every
number below names its fact key); brief: `docs/brief.md`; reviews and their
dispositions: `docs/reviews/mental_math/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a question for the
viewer; the beat ends with the question and the picture holds before the
next beat answers it.

**The "your turn" device.** A quiz is a `⟨predict⟩` beat whose last column
says `listen N s`: a violet card with the problem, a ring that drains over N
seconds with the seconds left in its middle, the line "pause for more time"
under it, the narration silent and the music alone (one soft mallet note a
second). The next beat works the answer with the trick, in the same place on
screen. The owner asked for "lots of quiz questions", so the quizzes are this
video's prediction pauses and there are more than the usual two a chapter
(plan rule 10). Every quiz is a row of `model::QUIZZES`, and its numbers are
the `F-Q.<line>.*` facts.

## 2 The arc

The owner asked for the reason behind each trick and for many quizzes; his
four tricks carry the video, and the two chapters of everyday tricks are
short.

| Act | Chapters | Spine |
|---|---|---|
| I. The centrepiece | 01 eighty-five squared, 02 ends in five, 03 your turn | a quiz before anything is taught; the paper method fails in the head; the trick on 35; the square whose strip moves; three quizzes |
| II. Any product | 04 by parts, 05 running total | a rectangle cut into four pieces; the zeros come back by size; the hard part is holding four numbers, so hold one |
| III. Spot the structure | 06 the middle, 07 Bethe, 08 near fifty, 09 your turn | two numbers the same distance from a middle; Feynman's story of Bethe and 48 squared; the same strips, taken off or put on |
| IV. Everyday | 10 percents, 11 round numbers | the ten-percent block and the swap; halving, doubling, and round then fix |
| V. Which trick? | 12 which trick, 13 nothing to grab, 14 near something you know | a mixed round; a product no trick helps, and paper; the one move behind three of the tricks |

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| the big block (tens times tens: the hundreds) | blue `#58C4DD` |
| the strips (tens times ones) | teal `#5CD0B3` |
| the corner (ones times ones; the small square) | amber `#F0AC5F` |
| the strip that moves, and the trick's one move wherever it appears | `hero` gold `#FFD866` |
| a piece taken away | `warn` red `#FC6255` |
| a confirmed answer | `good` green `#83C167` |
| the "your turn" card and its countdown | violet `#B189D6` |
| the running total, and every number being held in the head | `ink` white, large |

The six hues are checked pairwise for contrast (ΔE2000 ≥ 20) by a test before
drafts, as Video C's customers were.

## Vocabulary (one word per idea)

**trick** (never "hack", "shortcut", "secret" or "method"); **times** (never
"multiplied by"); **squared**; **the number in front** (the digits left of
the 5; the letter n appears once, at 03_7); **parts** (the tens and the ones
a number breaks into) and **pieces** (the smaller rectangles); **block**,
**strip** and **corner** (the three kinds of piece); **running total**;
**how big** (for the zeros); **the middle** and **the distance** (never "a"
and "b" aloud); **your turn** (every quiz); **round, then fix**; **halve** and
**double**; **percent**; **paper** (the written method). Never "genius",
"human calculator", "instantly" or "lightning".

## 01_eighty_five_squared (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ Quick. What's eighty-five squared? | The quiz card, cold: 85² in large type, the ring draining from 4, "pause for more time" (`F-Q.01_1.*`) | **the quiz card**; countdown ring on a tracker | listen 4 s (the countdown: the music alone, one mallet note a second) |
| 01_2 | Seven thousand two hundred twenty-five. Eight times nine is seventy-two. Then write twenty-five. | The answer builds where the ring was: 8 × 9 = 72, then 25 slides in behind it: 7225 (`F-H1.trick.*`) | TeX equation; highlight | |
| 01_3 | *(title)* | **Mental Math Tricks That Actually Work (and Why)** | title card | listen 3 s (the motif: a rising question, then its answer) |
| 01_4 | If that took you longer than four seconds, here's why. School teaches multiplying on paper. For this one, that's two rows of digits and three carries, built up from the small end. | 85 × 85 set out as on paper: the rows 425 and 6800 written in, small carry digits above; two counters tick to "2 rows" and "3 carries" (`F-H1.paper.rows`, `.carries`) | a table of digits; lagged reveal; count-up | |
| 01_5 | On paper that's fine, because the paper remembers the rows. In your head, you have to hold them all at once. | The sheet slides away and the rows hang in the air inside a head outline with three slots; they overflow, and the first row fades out: lost | morph; highlight | |
| 01_6 | The trick holds one number. Seventy-two. | The head again, one slot filled: 72; the 25 lands after it | highlight | |
| 01_7 | And it rests on a picture. If you ever forget the trick, the picture gives it back. So every trick in this video comes with its picture, and with a few problems for you to try. | A square whose top strip lifts and slides round to its side (the picture of chapter 02, in outline), then small icons for what is coming: a cut rectangle, a number line with a middle, a bill; the quiz card icon last | lagged reveal; morph | hold |

- **Cold open** (plan rule 20): a puzzle the viewer cannot yet do in four
  seconds, answered in three short sentences, then the title, then the
  honest failure of the obvious approach (01_4, 01_5).
- **Misconception defused:** "people who do this fast have better memories".
  They hold less: one number against two rows and three carries.
- **Ruth:** "two rows and three carries" is computed for 85 × 85, not typed
  (`model::paper`, and a second long multiplication in the oracle): the rows
  425 and 6800, one carry inside each row and one in adding them. Nothing is
  claimed about how long anyone takes.
- **Dev:** the paper method is not called wrong; it is built for a medium
  that stores intermediate results.

## 02_ends_in_five (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Here's that trick on a smaller number. Thirty-five squared. Look at the number in front of the five. Three. | 35² in large type; the 3 lifts a little and takes the label "the number in front" | highlight | |
| 02_2 | Multiply it by the next number up. Three times four is twelve. | 3 × 4 = 12 beside it (`F-T1.n35.front`, `.next`, `.hundreds`) | TeX equation | |
| 02_3 | Then write twenty-five after it. One thousand two hundred twenty-five. | The 12 and a 25 slide together: 1225 (`F-T1.n35.square`); under it, small, 35 × 35 = 1225 | TeX morph | |
| 02_4 | ⟨predict⟩ But why the next number up? Why three times four, and not three times three? | A square, 35 on each side, drawn over a faint grid with its side lengths marked; the question as a card | **the square on a grid**; prediction card | hold |
| 02_5 | Thirty-five squared is the area of a square, thirty-five on each side. Cut each side into thirty and five. | Both sides are cut at 30: a blue block, two teal strips and an amber corner, the cuts labelled 30 and 5 (`F-T1.cut.parts`) | grow; lagged colouring | |
| 02_6 | The big block is thirty by thirty. The little corner is five by five. That's the twenty-five. | The corner reads 25 and the 25 in the answer above pulses with it (`F-T1.cut.corner`); the block reads 30 × 30 | live labels; highlight | |
| 02_7 | That leaves two strips, each thirty long and five wide. Slide one of them round, next to the other. | The top strip lifts, turns a quarter turn and settles beside the right strip, outlined in gold while it moves: two strips side by side, ten wide (`F-T1.cut.strip`) | **the strip that moves**; morph that keeps identity | |
| 02_8 | Side by side, they're ten wide. So the block has grown from thirty by thirty to thirty by forty. Three times four, in hundreds. | Block and strips merge into one blue rectangle, 30 by 40; it is ruled into twelve squares of a hundred, three rows of four, and a counter runs 1 to 12 (`F-T1.cut.grown`, `.grown_area`) | grid of hundreds; count-up | |
| 02_9 | That's where the next number comes from. The two fives make one more ten. | The line 35² = 30 × 40 + 25 = 1225 under the picture (`F-T1.cut.total`) | TeX equation | hold |

- **The centrepiece's why, as the owner asked:** the trick is stated on
  numbers (02_1 to 02_3), questioned (02_4), then earned by the picture.
- **Misconception defused:** "it should be three times three" (02_4): the
  block alone is 30 by 30, and the strips are what add the extra ten.
- **Ruth:** the areas are counted cell by cell in the oracle (900, 150, 150,
  25); the regrouping is 30 × 30 + 2 × (30 × 5) = 30 × 40.
- **Engine:** five fixed rectangles moved by placement; the grid is faint
  lines, not 1,225 cells.

## 03_your_turn (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | ⟨predict⟩ Your turn. Whenever you want more time, pause the video. What's twenty-five squared? | The quiz card: 25², ring from 4 (`F-Q.03_1.*`) | the quiz card | listen 4 s (countdown) |
| 03_2 | Two times three is six. Then twenty-five. Six hundred twenty-five. | 2 × 3 = 6, then 25: 625 | TeX equation; highlight | |
| 03_3 | ⟨predict⟩ Sixty-five squared? | The quiz card: 65², ring from 4 (`F-Q.03_3.*`) | the quiz card | listen 4 s (countdown) |
| 03_4 | Six times seven is forty-two. Then twenty-five. Four thousand two hundred twenty-five. | 6 × 7 = 42, then 25: 4225 | TeX equation; highlight | |
| 03_5 | ⟨predict⟩ Now past a hundred, where the number in front is ten. What's a hundred and five squared? | The quiz card: 105², ring from 6; the 10 in front of the 5 underlined (`F-Q.03_5.*`) | the quiz card | listen 6 s (countdown) |
| 03_6 | Ten times eleven is a hundred and ten. Then twenty-five. Eleven thousand and twenty-five. | 10 × 11 = 110, then 25: 11025; beside it the square again, now 100 by 110 and a corner | TeX equation; the square on a grid | |
| 03_7 | It works for every whole number that ends in five. If the number in front is n, the square is n, times n plus one, in hundreds. Plus twenty-five. | The numbers in 35² = 3 × 4 hundreds + 25 turn into letters: (10n + 5)² = 100 · n(n + 1) + 25, with "the number in front" under the n; lower-third "checked for every number from 5 to 99,995" (`F-X1.ends_in_five.*`); small: "7.5² = 56.25: the same trick, with the point put back" (`F-T1.half.squared`) | **TransformMatchingTex**: numbers → letters | |
| 03_8 | It is only as easy as that one multiplication. A hundred thirty-five squared needs thirteen times fourteen. For that, we need a way to multiply any two numbers. | 135²: 13 × 14 = ?, the product a question mark; a faint rectangle cut in four waits behind it | highlight | hold |

- **Start easy and climb** (the order): 2 × 3, then 6 × 7, then past a
  hundred, where the number in front has two digits.
- **Ruth:** "every whole number that ends in five" is tested from 5 to
  99,995 (`edge_the_square_of_every_number_ending_in_five…`); the edge is 5
  itself, where nothing is in front and zero times one gives 25. The
  identity is the general proof. The honest limit is 03_8.
- **Maya:** n is said once, with its label on screen, after three worked
  quizzes (numbers before letters).
- **The thread:** 03_8's 13 × 14 is answered as a quiz at 05_6.

## 04_by_parts (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Twenty-three times forty-six. No fives to help. Before anything else, ask how big the answer should be. Twenty times fifty is a thousand. So, somewhere near a thousand. | 23 × 46; each number slides to its nearest ten, 20 × 50 = 1000, tagged "about" (`F-T2.estimate`) | highlight; morph | |
| 04_2 | Now draw it. A rectangle, twenty-three by forty-six. Its area is the answer. | A rectangle 23 tall and 46 wide over the faint grid, its sides labelled | grow | |
| 04_3 | Break each number into parts. Twenty and three. Forty and six. The rectangle falls into four pieces. | Cuts at 20 and at 40; the side labels split into 20, 3 and 40, 6; the pieces take their colours: blue block, two teal strips, amber corner (`F-T2.parts.*`, `F-T2.pieces.sides`) | **the rectangle cut into parts**; lagged colouring | |
| 04_4 | ⟨predict⟩ Take the big piece, twenty by forty. Two times four is eight. But eight what? Eighty, eight hundred, or eight thousand? | The block lights; 2 × 4 = 8 over it; three choices under it: 80, 800, 8,000 (`F-T2.size.*`) | prediction card; highlight | hold |
| 04_5 | Eight hundred. Twenty times four is eighty already, so twenty times forty has to be ten times that. Tens times tens are hundreds. | 20 × 4 = 80 beside the choices; the 80 and the 8,000 fade; the block is ruled into eight squares of a hundred, two rows of four | grid of hundreds; highlight | |
| 04_6 | So leave the zeros out while you multiply. Then put them back by asking how big the piece has to be. | 2 × 4 = 8, then two zeros slide back in: 800 (`F-T2.size.lead`, `.zeros`) | TeX morph | |
| 04_7 | The other three go the same way. Twenty times six, a hundred twenty. Three times forty, a hundred twenty. Three times six, eighteen. | Each piece takes its number in turn: 120, 120, 18 (`F-T2.pieces`); lower-third "the distributive law"; lower-third, for programmers: "these four pieces are schoolbook multiplication; Karatsuba's 1962 algorithm gets the two middle ones from a single extra multiplication" | lagged reveal; live labels | |
| 04_8 | Four easy multiplications. What's left is adding them up. And that's the part that goes wrong. | The four numbers lift off the rectangle into a column, 800, 120, 120, 18, and start to waver | morph | hold |

- **The owner's words:** "ignoring zeros and then adding them back … think
  about how many zeros it must have to make sense" is 04_4 to 04_6.
- **Misconception defused:** "count the zeros" (04_4): the size of the piece
  tells you, and one wrong candidate is ruled out by a product already known.
- **Ruth:** the estimate is each number rounded to its nearest ten
  (`model::estimate`); the four pieces are counted cell by cell in the
  oracle. The law is named once, in a lower-third.
- **Dev:** the Karatsuba line is a lower-third, labelled "for programmers".

## 05_running_total (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Four numbers is a lot to hold while you're still multiplying. So don't hold them. Keep one running total, and add each piece the moment you have it. | The column of four numbers with the tag "hold 4"; it gives way to one large readout, "running total", beside the cut rectangle (`F-T2.held.pieces`) | live number on a tracker | |
| 05_2 | Start with the big piece. Eight hundred. Add a hundred twenty. Nine hundred twenty. Another hundred twenty. One thousand forty. And eighteen. One thousand fifty-eight. | Each piece lights in turn and pours into the total, which runs 800, 920, 1040, 1058 (`F-T2.running`); a piece dims once it is in | **live count-up**; highlight | |
| 05_3 | You only ever held one number. And the first thing you said, eight hundred, was already in the right neighbourhood. | A number line from 0 to 1,200: the estimate 1,000, the first total 800 and the answer 1,058 marked on it (`F-T2.product`) | number line | |
| 05_4 | ⟨predict⟩ Your turn. Six times forty-three? | The quiz card: 6 × 43, ring from 5 (`F-Q.05_4.*`) | the quiz card | listen 5 s (countdown) |
| 05_5 | Six forties are two hundred forty. Six threes are eighteen. Two hundred fifty-eight. | A rectangle 6 by 43 in two pieces; the total runs 240, 258 | the rectangle cut into parts; live count-up | |
| 05_6 | ⟨predict⟩ And the one we left. Thirteen times fourteen? | The quiz card: 13 × 14, ring from 6 (`F-Q.05_6.*`) | the quiz card | listen 6 s (countdown) |
| 05_7 | A hundred. A hundred forty. A hundred seventy. A hundred eighty-two. So a hundred thirty-five squared is eighteen thousand two hundred twenty-five. | Four pieces, 100, 40, 30, 12; the total runs 100, 140, 170, 182; then 182 and 25 slide together: 135² = 18225 (`F-T1.n135.*`) | live count-up; TeX morph | hold |

- **The owner's point** ("the hardest part is remembering the 4 numbers and
  adding them up") is a beat of its own, 05_1, and the cure is 05_2: one
  number held, pieces added in the order block, strip, strip, corner.
- **Misconception defused:** "the multiplying is the hard part".
- **Dev (lower-third at 05_3, for programmers):** "biggest piece first is an
  anytime answer: stop after any step and you have an estimate, with the
  digits that matter most already right".
- **Ruth:** "in the right neighbourhood" is 800 against 1,058, shown on a
  line, not claimed as a bound; the running total never goes down
  (`edge_the_pieces_always_add_up_to_the_product`, all products up to
  99 × 99).

## 06_the_middle (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ Forty-seven times fifty-three. Before you start on the parts, look at the two numbers. What do they have in common? | 47 × 53; a number line fades in beneath, 47 and 53 marked, nothing between them yet | number line; prediction card | |
| 06_2 | They're the same distance from fifty. Three below, and three above. | 50 appears between them; two arcs of length 3 (`F-T3.middle`, `.distance`) | number line; highlight | |
| 06_3 | So draw fifty squared, and lay the rectangle over it. It sticks out by three on one side, and falls short by three on the other. | A 50 by 50 square in outline; the 47 by 53 rectangle over it: a strip 3 wide stands out past the right edge, and a gap 3 tall is left along the top (`F-T3.strip`, `.gap`) | overlay; highlight | |
| 06_4 | Move the part that sticks out into the gap. It nearly fills it. What's missing is one small square, three by three. | The strip lifts, turns and settles into the gap, gold while it moves; a 3 by 3 hole is left in the corner, outlined red (`F-T3.strip_area`, `.gap_area`) | **the strip that moves**; morph | |
| 06_5 | So the answer is fifty squared, minus three squared. Twenty-five hundred, minus nine. Two thousand four hundred ninety-one. | 47 × 53 = 50² − 3² = 2500 − 9 = 2491 (`F-T3.*`); lower-third "(a + b)(a − b) = a² − b²: the difference of squares", a labelled "the middle", b "the distance" | TeX equation; lower-third | |
| 06_6 | ⟨predict⟩ Your turn. Nineteen times twenty-one? | The quiz card: 19 × 21, ring from 4 (`F-Q.06_6.*`) | the quiz card | listen 4 s (countdown) |
| 06_7 | The middle is twenty, and they're one away. Four hundred, minus one. Three hundred ninety-nine. | Number line 19, 20, 21; 20² − 1² = 399 | number line; TeX equation | |
| 06_8 | ⟨predict⟩ Sixty-five times seventy-five? | The quiz card: 65 × 75, ring from 6 (`F-Q.06_8.*`) | the quiz card | listen 6 s (countdown) |
| 06_9 | The middle is seventy. Five away. Forty-nine hundred, minus twenty-five. Four thousand eight hundred seventy-five. | Number line 65, 70, 75; 70² − 5² = 4875 | number line; TeX equation | |
| 06_10 | It needs a middle that's easy to square. Forty-seven times fifty-six has its middle at fifty-one and a half. Back to parts. | 47 × 56 on the number line, its middle at 51½ marked red; the cut rectangle returns beside it (`F-T3.no_middle.*`) | number line; highlight | hold |

- **The owner's "(a + b)*(a − b)"**, spoken as the middle and the distance so
  that the letter a is never read aloud; the letters are on screen once.
- **Misconception defused:** "this is a fact about fifty" (06_7 and 06_9
  use 20 and 70).
- **Ruth:** exact for any two numbers with a whole middle, tested for every
  such pair up to 200 (`edge_equally_far_from_a_middle…`); 06_10 is the
  limit (a half-integer middle still works, and is not easier).
- **Engine:** the square is an outline and the rectangle a fill; the moving
  strip is one rectangle, turned by placement.

## 07_bethe (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | The next trick comes with a story. Los Alamos, in the nineteen forties. Richard Feynman is working under Hans Bethe, and they're putting numbers into a formula. They get to forty-eight squared. | Two name cards, "Hans Bethe, head of the Theoretical Division" and "Richard Feynman, a group leader in it"; "Los Alamos"; between them 48² | lagged cards | |
| 07_2 | Feynman reaches for the calculator, a mechanical one. Bethe says, that's twenty-three hundred. | A desk calculator in outline, labelled "a Marchant calculator"; above Bethe's card, 2300 (`F-C1.bethe.rough`) | lagged reveal | |
| 07_3 | Feynman starts pushing the buttons anyway. And Bethe says that if he wants it exactly, it's twenty-three oh four. | The 2300 becomes 2304; the calculator's dials turn and stop on 2304 (`F-C1.bethe.exact`) | live digits; highlight | |
| 07_4 | Feynman is impressed. And Bethe asks him a question. Don't you know how to square numbers near fifty? | Quote card, verbatim: "Don't you know how to square numbers near 50?"; source line "Hans Bethe, as told by Richard P. Feynman, "Surely You're Joking, Mr. Feynman!" (1985), "Lucky Numbers"" | quote card | hold |
| 07_5 | ⟨predict⟩ Fifty squared is twenty-five hundred. How would you get from there to forty-eight squared? | 50² = 2500 on the left, 48² = ? on the right, an arrow between them; a 50 by 50 square in outline with a 48 by 48 square inside it | prediction card; the square on a grid | |
| 07_6 | Bethe's way. Forty-eight is two below fifty, so take two hundred off. That's his twenty-three hundred. Then add two squared. Twenty-three oh four. | Three steps in a row: 2500, − 200 = 2300, + 4 = 2304 (`F-T4.below.*`, `F-T4.fifty_squared`) | TeX equation; lagged reveal | hold |

- **Every detail is from the book's text** (fact sheet, source S1, pp.
  193–94 of the 1985 Norton edition): the number, the calculator, the two
  answers in that order, Feynman's reaction, Bethe's question and his
  method. Two sentences are quoted verbatim in the whole video: Bethe's
  question here and Feynman's last line at 09_8. Everything else is
  paraphrase.
- **What is not said:** what the formula was (the book does not say); that
  Bethe was surprised (the book gives his question and leaves it there; the
  owner's "surprised he didn't know that trick" is carried by the question
  itself); anything from Feynman's 1975 talk, which tells it differently.
- **Ruth:** "working under" rests on Feynman's own sentence in the book and
  on the laboratory's history (fact sheet, S2); the calculator's brand is on
  screen only, so no pronunciation is at stake.

## 08_near_fifty (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Here's why that works. Forty-eight squared sits inside fifty squared. To cut down to it, take a strip off the top and a strip off the side. Each strip is fifty by two. A hundred each. | The 50 by 50 square, the 48 by 48 square inside it in blue; the two strips along the top and the right edge turn red, each labelled 50 × 2 = 100 (`F-T4.below.strip`, `.two_strips`) | the square on a grid; highlight | |
| 08_2 | But the two strips cross in the corner. That small square, two by two, got taken off twice. So one of them goes back. Plus four. | The corner where the strips cross flashes; a small amber square drops back in: + 4 (`F-T4.below.corner`); the line 2500 − 200 + 4 = 2304 | glow; TeX equation | |
| 08_3 | ⟨predict⟩ Now a number above fifty. What changes for fifty-three squared? | 53² = ?; the 50 by 50 block with a larger square outlined round it | prediction card | |
| 08_4 | The same picture, with the strips put on. Two strips, fifty by three. Lay them end to end and they're a hundred by three. Three hundreds. And this time the corner is new. Three by three. Nine. | The block gains two teal strips and an amber corner; the top strip slides round, gold, to lie end to end with the other: one strip 100 by 3 (`F-T4.above.long_strip`, `.two_strips`, `.corner`) | **the strip that moves**; morph | |
| 08_5 | So fifty-three squared is twenty-five hundred, plus three hundred, plus nine. Two thousand eight hundred nine. | 2500 + 300 + 9 = 2809 (`F-T4.above.total`) | TeX equation | |
| 08_6 | Which gives a quick way to say it. Start from twenty-five. Add the distance from fifty, or take it off. That's the hundreds. Then write the distance squared. | Two rows build: 25 + 3 = 28, 3² = 09, giving 28 09; 25 − 2 = 23, 2² = 04, giving 23 04 (`F-T4.above.digit_pairs`, `F-T4.below.digit_pairs`); lower-third "precisely: (50 + d)² = 2500 + 100d + d², d = how far from 50" | TeX morph; lower-third | hold |

- **One picture for both directions:** strips taken off (below fifty) or put
  on (above); the two strips of fifty make "that many hundreds", which is why
  the distance goes straight onto 25.
- **Misconception defused:** "below fifty you subtract the distance squared
  too" (08_2: the corner was removed twice, so it comes back).
- **Ruth:** 2,500 − 200 + 4 is checked by counting the cells of the inner
  square; the two-digit reading is exact for 41 to 59 and nowhere else from
  0 to 100 (`edge_near_fifty…`), and 09_7 says so.

## 09_your_turn_fifty (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | ⟨predict⟩ Your turn. Fifty-two squared? | The quiz card: 52², ring from 4 (`F-Q.09_1.*`) | the quiz card | listen 4 s (countdown) |
| 09_2 | Twenty-five plus two is twenty-seven. Two squared is four. Twenty-seven oh four. | 25 + 2 = 27, 2² = 04: 27 04 | TeX equation; highlight | |
| 09_3 | ⟨predict⟩ Forty-six squared? | The quiz card: 46², ring from 5 (`F-Q.09_3.*`) | the quiz card | listen 5 s (countdown) |
| 09_4 | Four below. Twenty-five minus four is twenty-one. Four squared is sixteen. Twenty-one sixteen. | 25 − 4 = 21, 4² = 16: 21 16 | TeX equation; highlight | |
| 09_5 | ⟨predict⟩ Fifty-seven squared? | The quiz card: 57², ring from 5 (`F-Q.09_5.*`) | the quiz card | listen 5 s (countdown) |
| 09_6 | Seven above. Thirty-two. Seven squared is forty-nine. Thirty-two forty-nine. | 25 + 7 = 32, 7² = 49: 32 49 | TeX equation; highlight | |
| 09_7 | That reads straight off while you're within nine of fifty. Further out, the square spills over into the hundreds. Sixty-two squared is thirty-seven hundred, plus a hundred forty-four. | 62²: 25 + 12 = 37, 12² = 144; the 1 of the 144 carries into the hundreds: 3844 (`F-T4.carries.*`); lower-third "reads straight off from 41 to 59" (`F-X4.digit_pairs.*`) | TeX morph; highlight | |
| 09_8 | Feynman says he took up tricks like this himself after that. He writes that he very rarely beat Bethe to an answer. And he says why. Every number was near something Bethe knew. | Quote card, verbatim: "It was easy for him—every number was near something he knew."; source line as at 07_4 | quote card | hold |

- **Climb:** two above, four below, seven above.
- **Ruth:** the limit is stated where the viewer has just used the trick
  (09_7); the formula is exact for every number, the reading only for 41 to
  59. 09_8 paraphrases two of the book's sentences and quotes one.

## 10_percents (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Now money. A forty-eight dollar bill, and you want to leave fifteen percent. Ten percent is easy. The decimal point moves one place. Four dollars eighty. | The bill as a long bar, $48.00, cut into ten strips; one strip lifts out: 10% = $4.80, and the decimal point in 48.00 hops one place left (`F-T5.bill`, `.ten`) | proportion bar; highlight | |
| 10_2 | Five percent is half of that. Two forty. Together, seven dollars twenty. | Half a strip: 5% = $2.40; the strip and the half sit side by side: 15% = $7.20 (`F-T5.bill.five`, `.fifteen`) | proportion bar; live labels | |
| 10_3 | And twenty percent is the ten percent block, doubled. | Two strips: 20% = $9.60 (`F-T5.bill.twenty`) | proportion bar | |
| 10_4 | ⟨predict⟩ Your turn. Fifteen percent on sixty dollars? | The quiz card: 15% of $60, ring from 5 (`F-Q.10_4.*`) | the quiz card | listen 5 s (countdown) |
| 10_5 | Ten percent is six dollars. Half of that is three. Nine dollars. | $6.00 + $3.00 = $9.00 on a bar of ten strips | proportion bar; TeX equation | |
| 10_6 | ⟨predict⟩ Now a stranger one. What's eight percent of twenty-five? | 8% of 25 = ? in large type | prediction card | |
| 10_7 | Turn it round. Twenty-five percent of eight is a quarter of eight. Two. And eight percent of twenty-five is also two. | The two phrases swap their numbers: 25% of 8 = 2, then back: 8% of 25 = 2 (`F-T5.swap.*`) | TeX morph | |
| 10_8 | Percent means per hundred. So both are eight times twenty-five, over a hundred. The same rectangle, turned on its side. | A rectangle 8 by 25, area 200, turns a quarter turn to 25 by 8; under it 8 × 25 ÷ 100 = 2 (`F-T5.swap.area`); lower-third "precisely: x% of y = xy ÷ 100 = y% of x" | morph that keeps identity; lower-third | |
| 10_9 | ⟨predict⟩ Your turn. Four percent of seventy-five? | The quiz card: 4% of 75, ring from 5 (`F-Q.10_9.*`) | the quiz card | listen 5 s (countdown) |
| 10_10 | Seventy-five percent of four. Three quarters of four is three. The swap only pays when one of the two is a friendly percent. | 75% of 4 = 3; then 17% of 43 and 43% of 17, both grey: no easier either way | TeX morph; highlight | hold |

- **Real life** (the owner's test): a tip, a discount, a tax rate. Ten
  percent is the block; five is half of it; everything else here is built
  from those.
- **Misconception defused:** "x percent of y and y percent of x are
  different questions" (10_7, 10_8).
- **Ruth:** the swap is commutativity, exact for every x and y
  (`edge_the_small_tricks…`); its usefulness is not general, and 10_10 says
  so. Money is in whole cents throughout.

## 11_round_numbers (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | Some numbers are one step from a round one. Five is half of ten. So five times eighty-six is ten times eighty-six, cut in half. Eight sixty, halved. Four hundred thirty. | A rectangle 10 by 86, area 860; it is cut across the middle and the top half fades: 5 by 86 = 430 (`F-T6.five.*`) | the rectangle; morph | |
| 11_2 | The same cut works between two numbers. Fourteen times thirty-five. Halve the fourteen, double the thirty-five. Seven times seventy. Four hundred ninety. | A rectangle 14 by 35 is cut in half across; the top half slides down beside the bottom half, gold while it moves: 7 by 70 = 490 (`F-T6.halve.*`); lower-third "25 × 36: a quarter of 3,600 is 900" (`F-T6.twenty_five.*`) | **the strip that moves**; morph that keeps identity | |
| 11_3 | Nine is one short of ten. So nine times thirty-seven is ten of them, less one. Three seventy, minus thirty-seven. Three hundred thirty-three. | A rectangle 10 by 37; one row along the top turns red and is sliced away: 370 − 37 = 333 (`F-T7.nines.*`) | the rectangle; highlight | |
| 11_4 | Prices are built for this. Four things at nineteen ninety-nine. Call it four twenties. Eighty dollars. Then give back four cents. Seventy-nine ninety-six. | Four price tags, $19.99; each rounds up to $20.00 and sheds a red cent: $80.00 − $0.04 = $79.96 (`F-T7.price.*`) | lagged reveal; TeX equation | |
| 11_5 | ⟨predict⟩ Your turn. Five times sixty-four? | The quiz card: 5 × 64, ring from 4 (`F-Q.11_5.*`) | the quiz card | listen 4 s (countdown) |
| 11_6 | Six hundred forty, halved. Three hundred twenty. | 640 ÷ 2 = 320 | TeX equation | |
| 11_7 | ⟨predict⟩ Sixteen times forty-five? | The quiz card: 16 × 45, ring from 5 (`F-Q.11_7.*`) | the quiz card | listen 5 s (countdown) |
| 11_8 | Halve and double. Eight times ninety. Seven hundred twenty. | 16 × 45 = 8 × 90 = 720; lower-third "needs an even number: 15 × 37 has nothing to halve" (`F-T6.no_halve`) | TeX morph; lower-third | |
| 11_9 | ⟨predict⟩ Three things at four ninety-nine each? | The quiz card: 3 × $4.99, ring from 5 (`F-Q.11_9.*`) | the quiz card | listen 5 s (countdown) |
| 11_10 | Three fives are fifteen dollars. Give back three cents. Fourteen ninety-seven. | $15.00 − $0.03 = $14.97 | TeX equation | hold |

- **Two moves, one idea:** get to a round number, then put right what you
  changed, by halving (11_1, 11_2) or by taking the extra back off (11_3,
  11_4). Both are the rectangle of chapter 04, cut another way.
- **Real life:** packs of five, quarters, and every price that ends in 99.
- **Ruth:** each is exact (`edge_the_small_tricks…`); halving needs an even
  number and the lower-third at 11_8 says so. "Three seventy" for 370 is
  how the voice groups it; the screen shows 370.

## 12_which_trick (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | ⟨predict⟩ Now they're mixed up, so the first job is to pick the trick. Pause whenever you like. Seventy-five squared? | Five small tiles along the top, one per trick: ends in five, parts, the middle, near fifty, percent, round numbers; the quiz card: 75², ring from 5 (`F-Q.12_1.*`) | the quiz card; lagged tiles | listen 5 s (countdown) |
| 12_2 | It ends in five. Seven times eight is fifty-six. Five thousand six hundred twenty-five. | The "ends in five" tile lights; 7 × 8 = 56, then 25: 5625 | highlight; TeX equation | |
| 12_3 | ⟨predict⟩ Forty-nine times fifty-one? | The quiz card: 49 × 51, ring from 5 (`F-Q.12_3.*`) | the quiz card | listen 5 s (countdown) |
| 12_4 | One either side of fifty. Twenty-five hundred, minus one. Two thousand four hundred ninety-nine. | The "middle" tile lights; 50² − 1² = 2499 | highlight; TeX equation | |
| 12_5 | ⟨predict⟩ Fifty-four squared? | The quiz card: 54², ring from 5 (`F-Q.12_5.*`) | the quiz card | listen 5 s (countdown) |
| 12_6 | Near fifty. Twenty-nine. Then sixteen. Twenty-nine sixteen. | The "near fifty" tile lights; 25 + 4 = 29, 4² = 16: 29 16 | highlight; TeX equation | |
| 12_7 | ⟨predict⟩ A twenty percent tip on forty-five dollars? | The quiz card: 20% of $45, ring from 5 (`F-Q.12_7.*`) | the quiz card | listen 5 s (countdown) |
| 12_8 | Ten percent is four fifty. Doubled, nine dollars. | The "percent" tile lights; $4.50 × 2 = $9.00 | highlight; TeX equation | |
| 12_9 | ⟨predict⟩ Six times forty-nine? | The quiz card: 6 × 49, ring from 5 (`F-Q.12_9.*`) | the quiz card | listen 5 s (countdown) |
| 12_10 | Six fifties, less six. Two hundred ninety-four. | The "round numbers" tile lights; 300 − 6 = 294 | highlight; TeX equation | hold |

- **The mixed round the order asks for:** the viewer must choose before
  computing; the tiles name the candidates, and only the answer lights one.
- **Dev:** 54² and 75² sit together on purpose: one is near fifty, the other
  ends in five, and each trick fails on the other number.

## 13_nothing_to_grab (~50 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | ⟨predict⟩ Last one. Sixty-seven times seventy-eight? | The quiz card: 67 × 78, ring from 6; the tiles along the top stay dark (`F-Q.13_1.*`) | the quiz card | listen 6 s (countdown) |
| 13_2 | Nothing to grab. No five, no easy middle, no round number next door. Parts still works. Forty-two hundred. Forty-six eighty. Fifty-one seventy. Fifty-two twenty-six. | Only the "parts" tile lights, dimly; the rectangle 67 by 78 in four pieces, and the total runs 4200, 4680, 5170, 5226 | the rectangle cut into parts; live count-up | |
| 13_3 | That's four pieces with carrying, and it's slow. There's no shame in reaching for paper, or a calculator. That's what they're for. | A sheet of paper and a calculator slide in beside the rectangle | lagged reveal | |
| 13_4 | But keep one habit even then. Round first. Seventy times eighty is fifty-six hundred. So if the screen says five hundred, or fifty thousand, you know something went wrong. | 70 × 80 = 5600 tagged "about" (`F-Q.13_1.estimate`); a calculator screen reading 522.6, then 52,260, each crossed red; then 5226 in green beside 5600 | highlight; number line | hold |

- **Honest framing** (the order, item 5): a product where no trick helps is
  shown and said to be one; paper and a calculator are named as the right
  tools; what the viewer keeps is the estimate.
- **Ruth:** "nothing to grab" is true of the tricks in this video: no five,
  the middle is 72½, the nearest round numbers are three and two away. Parts
  is exact as always.

## 14_near_something_you_know (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | ⟨predict⟩ Look back at three of the pictures. Thirty-five squared. Forty-eight squared. Forty-seven times fifty-three. What did they share? | The three pictures side by side, small: the square of 35 with its strip moved, the square of 48 inside 50, the 47 by 53 rectangle over the square of 50 | lagged reveal; prediction card | |
| 14_2 | One move. Slide a strip from one side of a square to the other, and you're left with an easy rectangle and a small square. Thirty times forty, plus twenty-five. Forty-six times fifty, plus four. Fifty squared, minus nine. | In each picture the strip moves once more, gold; under them 35² = 30 × 40 + 25, 48² = 46 × 50 + 4, 47 × 53 = 50² − 3²; then one line in letters: a² = (a − d)(a + d) + d² | **the strip that moves**, three times; TransformMatchingTex | |
| 14_3 | None of this is new. An American school book from eighteen forty-four has the fives trick, the middle trick and the near fifty trick, all on one page. | Card: "Pliny E. Chase, The Elements of Arithmetic, Philadelphia, 1844, p. 60" with its three examples: 35 × 35 = 1225, 87 × 73 = 6351, 53 × 53 = 2809 (`F-C3.chase.*`) | quote card | |
| 14_4 | And none of it takes a gift. It takes a few numbers you know well, and the habit of looking for the nearest one. Every number is near something you know. | A number line from 0 to 110 with the landmarks lit: the squares ending in 5, 50, 100, the round tens; then the receipts card: "every number in this video was computed by code", the repository link, and names to search: the distributive law, difference of squares, the area model | number line; glow; receipts card | listen 4 s (the motif, answered) |

- **Ends on the rigorous form of the opening picture** (plan rule 15): the
  strip that moved in chapter 02, as one identity that contains the fives
  trick (d = 5), the near-fifty trick and the middle trick.
- **Hindsight** (plan rule 19): three tricks the viewer learned separately
  were one move.
- **Ruth:** a² = (a − d)(a + d) + d² for all a and d; 48² = 46 × 50 + 4 is
  Bethe's 2,300 + 4 by another route. The 1844 page is a latest date for
  the tricks, not an origin, and the card claims no more (fact sheet, S4).
- **The last sentence** echoes Feynman's line at 09_8 in the second person;
  it is not presented as a quotation.
