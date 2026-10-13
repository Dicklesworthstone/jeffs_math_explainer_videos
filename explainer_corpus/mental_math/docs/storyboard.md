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

**Three kinds of question, three looks.**

- **Your turn** (a quiz): a `⟨predict⟩` beat whose last column says
  `listen N s (countdown)`. A violet card with the problem; a ring that
  drains over N seconds with the seconds left in its middle; the line "pause
  for more time" under it (not on the cold open's card); the narration silent
  and the music alone, one mallet note a second. At zero the answer lands
  alone, large, where the ring was, on the countdown's last note. The next
  beat then works it with the trick, in the same place, and ends on the
  number already there. The first quiz after each trick replays that trick's
  picture in small beside the working; the later ones are worked in type.
  Every quiz is a row of `model::QUIZZES`, and its numbers are the
  `F-Q.<line>.*` facts.
- **Why?** (02_4, 06_6, 08_1, 09_5, 10_4): the question as a card with no
  ring, held about two seconds.
- **Which?** (04_4): three candidates side by side; two fade.

The owner asked for "lots of quiz questions", so the quizzes are this
video's prediction pauses and there are more than the usual two a chapter
(plan rule 10).

**The one move.** A strip is sliced off the top of a square, lifts, makes a
quarter turn and stands against the right side. The square has become a
rectangle, shorter by the strip's depth and wider by it, with a small square
sticking up at the end, where the strip is taller than the rectangle. It is
drawn the same way every time: the same figure position, the same turn, two
seconds, the narrator silent, one short plucked sound. Forwards at 02_6,
10_1, 10_5 and three times at 14_3; backwards, a strip from the side into a
gap along the top, at 08_4. A rectangle's sides are always said tall by
wide: thirty by forty, twenty-three by forty-six.

**Numbers on screen.** No thousands separator in any number being worked
(1058, 2304, 11025): the two-part reading (12 then 25, 23 then 04) must stay
visible. Separators only in prose lower-thirds.

## 2 The arc

The owner asked for the reason behind each trick and for many quizzes; his
four tricks carry the video, and the two chapters of everyday tricks are
short and sit in the middle, where the video needs a breath.

| Act | Chapters | Spine |
|---|---|---|
| I. The centrepiece | 01 eighty-five squared, 02 ends in five, 03 your turn | a quiz before anything is taught; the paper method fails in the head; the trick on 35; the square whose strip moves; two quizzes; every number in front, swept |
| II. Any product | 04 by parts, 05 running total | a rectangle cut into four pieces; the zeros come back by size; four numbers are too many to hold, so hold one |
| III. Everyday | 06 percents, 07 round numbers | the tenth and the swap; halving and doubling; round, then fix |
| IV. Spot the structure | 08 the middle, 09 Bethe, 10 near fifty, 11 your turn | two numbers the same distance from a middle; Feynman's story of Bethe and 48 squared; the same move again |
| V. Which trick? | 12 which trick, 13 nothing to grab, 14 near something you know | a mixed round; a product no trick helps, and paper; one square that two tricks agree on, and the one move behind three of them |

The owner fixed the first two tricks' order ("the centerpiece", "then we can
lead into") and listed the rest. The everyday tricks come before the middle
and near fifty (R1, C3): they are cuts of chapter 04's rectangle, they give
an easy stretch after the densest act, and the story then leads straight
into the mixed round and the ending that recalls its pictures.

## 3 Color roles

One colour per idea for the whole video (plan §6.1); a highlight never uses a
data colour.

| Idea | Color |
|---|---|
| hundreds: the big block, and any piece that is a whole number of hundreds | blue `#58C4DD` |
| the strips (tens times ones), and the strip that moves | green `#70CCA1` |
| the corner (ones times ones; the small square that sticks out) | orange `#CC811F` |
| the move itself | `hero` gold `#FFD866`, as an outline and a glow on the moving piece, never a fill |
| a piece taken away, a gap, a wrong reading | `warn` red `#FC6255` |
| the "your turn" card and its countdown | violet `#B189D6` |
| numbers held in the head, the running total, an answer that has landed | `ink` white, large |

Every pair of these, with the background, is at least 24 apart (CIEDE2000;
the least is gold against orange, 24.2), pinned by a test in the crate
before any draft. The corpus's `good` green is not used: an answer is
confirmed by landing, large, in ink.

## Vocabulary (one word per idea)

**trick** (never "hack", "shortcut", "secret" or "method"); **times** between
two numbers (never "multiplied by", and never a plural count such as "six
forties"; "multiply" as a verb is fine); **squared**; **the number in
front** (the digits left of the 5; the letter n is spoken once, at 03_6);
**parts** (the tens and the ones a number breaks into) and **pieces** (the
smaller rectangles); **block**, **strip** and **corner** (the three kinds of
piece, named aloud at 04_3); **strip** is also the piece that moves, and the
**small square** is what it leaves sticking up, or missing; **running
total**; **floor** (a total the answer cannot be below); **how big** (for
the zeros, and for what the estimate tells you) and **round first** (the
estimate); **a tenth** (ten percent of a bill; never "block" or "strip");
**the swap**; **the middle** and **the distance**, both named at 08_2 (never
"a" and "b" aloud); **your turn** (the first quiz of each group); **round,
then fix**, with **give back** for the fix; **half** and **double** (on
screen "halve"; the voice says "half of", which cannot be heard as "have");
**minus** in a sum, **take off** for what a picture does; **percent**;
**paper** (the written method). Numbers being worked are said in full, "one
hundred twenty", "one thousand forty"; whole hundreds as "twelve hundred",
"twenty-five hundred"; money keeps its "dollars" ("two dollars forty"); the
two-pair reading ("twenty-three oh four") is kept for squares near fifty,
where it is the trick. Never "genius", "human calculator", "instantly" or
"lightning".

## Sound plan (for the score bead)

Nothing here is auditioned yet; the drafts of P5 are silent but for the
scratch voice.

- **Countdowns.** One mallet note a second, from D, E, F sharp, A and B
  only. Every countdown ends F sharp, E, then D as the answer lands, so the
  last three seconds always sound the same and the first note tells the
  length (4 s starts on B, 5 s on the upper D, 6 s on the upper E). The D
  lands over a different chord of the progression each time, and over the
  tonic chord only for the last quiz of an act and at 14_6.
- **The title** (01_3) plays the motif's question and leaves it open; the
  answer is kept for 14_6.
- **Two silences:** before Bethe's question (09_4), and at the end of
  13_1's ring, which has no last note.
- **The move** has one short plucked sound, the same at 02_6, 08_4, 10_1,
  10_5 and, three times, at 14_3; 14_4 then holds on the music alone.
- **The machine** (09_3): dry ticks, slowing, while its digits run.

## 01_eighty_five_squared (58 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | ⟨predict⟩ Quick question. What's eighty-five squared? | The quiz card, cold, with nothing else on screen and no pause line: 85² in large type, the ring draining from 4; at zero 7225 lands (`F-Q.01_1.*`) | **the quiz card**; countdown ring on a tracker | listen 4 s (countdown: the music alone, one mallet note a second) |
| 01_2 | Seven thousand two hundred twenty-five. Eight times nine is seventy-two. Then write twenty-five. | Under the answer its working builds: 8 × 9 = 72, and the 72 and a 25 slide up into the 7225 already there (`F-H1.trick.*`) | TeX equation; highlight | |
| 01_3 | *(title)* | **Mental Math Tricks That Actually Work (and Why)** | title card | listen 3 s (the motif's question only, unanswered) |
| 01_4 | If that took you longer than four seconds, there's a likely reason. The usual way to multiply is built for paper. For this one, that's two rows of digits and three carries. | 85 × 85 set out as on a sheet of paper: the rows 425 and 6800 written in, small carry digits above; two counters tick to "2 rows" and "3 carries" (`F-H1.paper.rows`, `.row_count`, `.carries`) | a table of digits; lagged reveal; count-up | |
| 01_5 | On paper that's fine, because the paper remembers the rows. In your head, you have to hold both rows while you add them. | The sheet slides away and what is in hand at the worst moment hangs in the air under the words "in your head": 425, 6800, the sum so far, a carry; there is no shelf for them, and the first row fades: lost | morph; highlight | |
| 01_6 | The trick holds one number. Seventy-two. | "in your head" again, with one large white number: 72; the 25 lands after it | highlight | |
| 01_7 | And it rests on a picture. If you ever forget the trick, the picture gives it back. So every trick here comes with its picture. And there are problems for you to try. | A square whose top strip lifts and stands against its side (the move of chapter 02, in outline), then five more small pictures for what is coming: a cut rectangle, a bill cut in ten, a rectangle with one cut across it, a number line with a middle, a square of fifty; the quiz card last | lagged reveal; morph | hold |

- **Cold open** (plan rule 20): a puzzle the viewer cannot yet do in four
  seconds, answered in three short sentences, then the title, then the
  honest failure of the obvious approach (01_4, 01_5). The six small
  pictures of 01_7 return as the tiles of chapter 12.
- **Misconception defused:** "people who do this fast have better memories".
  They hold less: one number against two rows.
- **Ruth:** "two rows and three carries" is computed for 85 × 85
  (`model::paper`, and a second long multiplication in the oracle): the rows
  425 and 6800, one carry inside each row and one in adding them. The
  carries are counted, not said to be held together, and the picture draws no
  number of slots: nothing is claimed about how much a head holds or how
  long anyone takes.
- **Dev:** the paper method is not called wrong; it is built for a medium
  that stores intermediate results.

## 02_ends_in_five (66 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Here's that trick on a smaller number. Thirty-five squared. The number in front of the five is three. | 35² in large type; the 3 lifts a little and takes the label "the number in front" | highlight | |
| 02_2 | Multiply it by the next number up. Three times four is twelve. | 3 × 4 = 12 beside it (`F-T1.n35.front`, `.next`, `.hundreds`) | TeX equation | |
| 02_3 | Then write twenty-five after it. One thousand two hundred twenty-five. | The 12 and a 25 slide together: 1225 (`F-T1.n35.square`); under it, small, 35 × 35 = 1225 | TeX morph | |
| 02_4 | ⟨predict⟩ But why the next number up, and not three times three? | A square, 35 on each side, drawn over a faint grid of tens with its side lengths marked; the question as a card | **the square on a grid**; prediction card | |
| 02_5 | Thirty-five squared is the area of a square, thirty-five on each side. Slice a strip off the top, five deep. | A cut runs across the square 5 below its top edge; the strip turns green, the rest blue (`F-M.n35.strip_wide`, `F-M.n35.strip_long`) | grow; highlight | |
| 02_6 | Now turn the strip, and stand it against the side. | **The one move:** the strip lifts, makes a quarter turn and stands against the right side, outlined in gold while it moves; the narrator waits for it | **the strip that moves**; morph that keeps identity | listen 2 s (the move's own plucked sound) |
| 02_7 | The square has become a rectangle. Five shorter, so thirty tall. Five wider, so forty wide. And the end of the strip sticks up. A small square, five by five. | The side labels change to 30 and 40 (`F-M.n35.tall`, `F-M.n35.wide`); the end of the strip that stands above the rectangle turns orange and reads 5 × 5 (`F-M.n35.corner`) | live labels; highlight | |
| 02_8 | Thirty by forty is three times four, in hundreds. Twelve hundred. | The rectangle's tens grid brightens into twelve squares of a hundred, three rows of four, and a small numeral is written into each in turn, 1 to 12 (`F-M.n35.hundreds_tiles`, `F-M.n35.rectangle_area`) | grid of hundreds; count-up | |
| 02_9 | And the small square is the twenty-five. Twelve hundred plus twenty-five. The next number comes in because thirty-five sits halfway between thirty and forty. | The line 35² = 30 × 40 + 25 = 1225 above the caption band (`F-M.n35.square`); under it, grey: "34² = 30 × 38 + 16: true, and no easier" (`F-M.n34.*`) | TeX equation | hold |

- **The centrepiece's why, as the owner asked:** the trick is stated on
  numbers (02_1 to 02_3), questioned (02_4), then earned by one cut and one
  move. This is the order's own picture: "the strip of width 5 moves to make
  an n by (n + 1) block of hundreds plus a 5 by 5 corner".
- **Misconception defused:** "it should be three times three" (02_4): the
  strip that comes off the top goes on to the side, so the two sides are 30
  and 40, not 30 and 30.
- **Why a five** (M11): the grey line at 02_9. With 34 the same move gives
  30 by 38, which is no easier; only a five lands both sides on round tens.
- **Ruth:** the cells are counted in the oracle (the strip 5 by 35, the
  rectangle 30 by 40, the small square 25); 1200 + 25 = 1225.
- **Engine:** a body, a strip and a tens grid; the strip is one rectangle
  turned on an arc so its area never changes on the way (E7).

## 03_your_turn (48 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | ⟨predict⟩ Your turn. Whenever you want more time, pause the video. What's twenty-five squared? | The quiz card: 25², ring from 5, "pause for more time"; at zero 625 lands (`F-Q.03_1.*`) | the quiz card | listen 5 s (countdown) |
| 03_2 | Two times three is six. Then twenty-five. Six hundred twenty-five. | 2 × 3 = 6, then 25; beside it, small, the move replayed on a square of 25: 20 tall, 30 wide and a small square | TeX equation; the strip that moves | |
| 03_5 | It works for every whole number that ends in five, because two things never change. Whatever is in front, the strip is five deep and the small square is twenty-five. | **The sweep:** the moved-strip picture with the number in front on a slider that runs 1 to 10; at each stop the rectangle is that many tens tall and one more wide, ruled into its hundreds, the count of hundreds and the square beside it: 2 and 225, 6 and 625, 12 and 1225, … 110 and 11025 (`F-T1.sweep.*`); the orange small square never changes | **live recomputation from a tracker**; grid of hundreds; live numbers | |
| 03_6 | In letters, call the number in front n. Multiply it by the next number, for the hundreds. Then add twenty-five. | The numbers of 35² = 100 × 3 × 4 + 25 turn into letters: (10n + 5)² = 100 × n × (n + 1) + 25, with "the number in front" under the n; once it has settled, lower-third "the picture works at any size; a program also checked every one up to 99,995" (`F-X1.ends_in_five.*`) | **TransformMatchingTex**: numbers → letters | |
| 03_7 | It's only as easy as that one multiplication, though. One hundred thirty-five squared needs thirteen times fourteen. We'll come back for that. | 135²: 13 × 14 = ?, the product a question mark; the card shrinks into the top right corner and stays there through chapters 04 and 05 | highlight | hold |

- **Start easy and climb** (the order): 2 × 3 here; the sweep then runs the
  number in front up to 10, past a hundred (10 × 11 = 110, 11025), and
  135² at 05_7 needs 13 × 14; 5 × 6 comes back in chapter 14. The quiz on
  105² was cut for length on 2026-10-13 (docs/decisions.md): the sweep's
  last stop shows the same number.
- **Showpiece** (C7): the sweep at 03_5 earns "every" before any letter
  appears: ten squares in a row, one picture.
- **Ruth:** "every whole number that ends in five" is the identity at 03_6
  and is also tested from 5 to 99,995
  (`edge_the_square_of_every_number_ending_in_five…`); the edge is 5 itself,
  where nothing is in front and zero times one gives 25. The lower-third
  says which is the reason and which is the check. The honest limit is 03_7.
- **Maya:** n is said once, as "n times the next number", with its label on
  screen, after two worked quizzes and the sweep (numbers before letters).
- **The thread** (C23): 03_7's card waits in the corner and is filled in at
  05_7. The decimal case (7.5² = 56.25) is in the description, not here.

## 04_by_parts (73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | Most problems aren't that neat. Take twenty-three times forty-six. Before anything else, round first. Twenty times fifty is a thousand, so the answer is near a thousand. | 23 × 46; each number slides to its nearest ten, 20 × 50 = 1000, tagged "about" (`F-T2.estimate`) | highlight; morph | |
| 04_2 | Now draw it. A rectangle, twenty-three by forty-six. Its area is the answer. | A rectangle 23 tall and 46 wide over the faint grid of tens, its sides labelled | grow | |
| 04_3 | Break each number into parts. Twenty and three. Forty and six. The rectangle falls into four pieces. A big block, two strips and a small corner. | Cuts at 20 and at 40; the side labels split into 20, 3 and 40, 6; the pieces take their colours: blue block, two green strips, orange corner (`F-T2.parts.*`, `F-T2.pieces.sides`) | **the rectangle cut into parts**; lagged colouring | |
| 04_4 | ⟨predict⟩ Take the block, twenty by forty. Two times four is eight. But is that eighty, eight hundred, or eight thousand? | The block lights; 2 × 4 = 8 over it; three candidates side by side: 80, 800, 8000 (`F-T2.size.*`) | prediction card; highlight | |
| 04_5 | Eight hundred. Twenty times four is eighty already, so twenty times forty has to be ten times that. Tens times tens are hundreds. | 20 × 4 = 80 beside the candidates; the 80 and the 8000 fade; the block's tens grid brightens into eight squares of a hundred, two rows of four | grid of hundreds; highlight | |
| 04_6 | So leave the zeros out while you multiply, and put them back by asking how big the piece has to be. | 2 × 4 = 8, then two zeros slide back in: 800 (`F-T2.size.lead`, `.zeros`) | TeX morph | |
| 04_7 | The other three go the same way. Twenty times six, one hundred twenty. Three times forty, one hundred twenty again, which is rare. Three times six, eighteen. | Each piece takes its sides and its number in turn: 20 × 6 = 120, 3 × 40 = 120, 3 × 6 = 18 (`F-T2.pieces`); once the last has landed, lower-third "the distributive law" | lagged reveal | |
| 04_8 | Four easy multiplications. Now there are four numbers to hold while you add them. | The four numbers lift off the rectangle and hang under "in your head", as the rows did at 01_5, and start to waver; then, alone in the foot band, lower-third "for programmers: (2 + 3) × (4 + 6) − 8 − 18 = 24: both strips at once, in tens. Three multiplications, not four: Karatsuba's algorithm" (`F-C4.karatsuba.*`) | morph; lower-third | hold |

- **The owner's words:** "ignoring zeros and then adding them back … think
  about how many zeros it must have to make sense" is 04_4 to 04_6. The
  callback to 04_1's estimate (2 × 5 = 10, in hundreds: the one case where
  two zeros go back and three come out, M9, D13) was cut for length on
  2026-10-13; the description keeps it.
- **Misconception defused:** "count the zeros" (04_4): the size of the piece
  tells you. And "the two strips always match" (04_7: "which is rare": it
  happens here because 46 is twice 23; at 05_7 they are 40 and 30).
- **Ruth:** the estimate is each number rounded to its nearest ten
  (`model::estimate`); the four pieces are counted cell by cell in the
  oracle. 04_8 counts four numbers and claims nothing about what people get
  wrong.
- **Dev:** the Karatsuba line is worked on this rectangle's digits (the
  third multiplication is 5 × 10, not the problem itself) and has the foot
  band to itself during the hold; it is a trade for computers on long
  numbers, not for heads, and the description says so.

## 05_running_total (77 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Four numbers is a lot to hold. So don't hold them. Keep one running total, and add each piece the moment you have it. | "in your head" empties to one large white number, tagged "running total", beside the cut rectangle (`F-T2.held.pieces`) | live number on a tracker | |
| 05_2 | Block, strip, strip, corner. Twenty times forty, eight hundred. Add twenty times six, nine hundred twenty. Add three times forty, one thousand forty. And three times six, one thousand fifty-eight. | **The pour:** each piece in turn lifts off the rectangle and lands in a pile ruled in hundreds beside the total, which runs 800, 920, 1040, 1058 (`F-T2.running`); after the block the pile is already most of its final size | **live count-up**; morph that keeps identity | |
| 05_3 | Between steps, you only ever held one number. And each total was a floor, so the answer could only go up. | A number line from 0 to 1600: the totals step up it, 800, 920, 1040, 1058; the stretch from 800 to 1500 is shaded, "after the block: between 20 × 40 and 30 × 50" (`F-T2.bounds`); tags "between steps: 4" giving way to "between steps: 1"; lower-third "for programmers: an anytime answer: stop after any step and you have a floor that only rises. The price: a later piece can change a digit you already said (920 → 1040)" | number line; lower-third | |
| 05_4 | ⟨predict⟩ Your turn. Six times forty-three? | The quiz card: 6 × 43, ring from 5; at zero 258 lands (`F-Q.05_4.*`) | the quiz card | listen 5 s (countdown) |
| 05_5 | Six times forty, two hundred forty. Add six times three, eighteen. Two hundred fifty-eight. | A rectangle 6 by 43 in two pieces; the total runs 240, 258; lower-third "pieces = digits × digits: 1 × 2 here, 2 × 2 before" (`F-T2.piece_counts`) | the rectangle cut into parts; live count-up | |
| 05_6 | ⟨predict⟩ And the one we left. It has four pieces, so pause if you like. Thirteen times fourteen? | The card from the corner (135²: 13 × 14 = ?) comes back beside the quiz card: 13 × 14, ring from 6; at zero 182 lands (`F-Q.05_6.*`) | the quiz card | listen 6 s (countdown) |
| 05_7 | Ten times ten, one hundred. Add ten times four, one hundred forty. Add three times ten, one hundred seventy. And three times four, one hundred eighty-two. Then twenty-five. One hundred thirty-five squared is eighteen thousand two hundred twenty-five. | Four pieces with their own numbers, 100, 40, 30, 12, the two strips plainly different; the total runs 100, 140, 170, 182; then the 182 and a 25 fill in the waiting card: 135² = 18225 (`F-T1.n135.*`) | live count-up; TeX morph | hold |

- **The owner's point** ("the hardest part is remembering the 4 numbers and
  adding them up") is a beat of its own, 04_8 into 05_1, and the cure is
  05_2: one number carried from step to step, pieces added in a fixed order.
- **What is and is not claimed** (R1, D1): a running total is a floor that
  only rises; it is not said to have the right leading digits (800 against
  1058 does not), and the lower-third names the price of going big end
  first.
- **Ruth:** the running total never goes down, and after the block the
  answer lies between the tens multiplied and the next tens multiplied, for
  every pair of two-digit numbers
  (`edge_the_pieces_always_add_up_to_the_product`).
- **Maya** (M2): 05_7 names the multiplication behind every step, and 05_6
  says there are four pieces and offers the pause.

## 06_percents (71 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Now money. A restaurant bill of forty-eight dollars, and you want to tip fifteen percent. Ten percent is one tenth, so the decimal point moves one place to the left. Four dollars eighty. | The bill as a long bar, $48.00, cut into ten; one tenth lifts out as a copy, outlined gold: 10% = $4.80; above it the digits of 48.00 in fixed places and the point hopping one place left (`F-T5.bill`, `F-T5.bill.ten`) | proportion bar; highlight | |
| 06_2 | Five percent is half of that. Two dollars forty. Together, seven dollars twenty. | Half a tenth: 5% = $2.40; the tenth and the half side by side: 15% = $7.20 (`F-T5.bill.five`, `.fifteen`) | proportion bar | |
| 06_3 | And twenty percent is the tenth, doubled. Seventeen percent already takes four pieces. | Two tenths: 20% = $9.60 (`F-T5.bill.twenty`); under it, grey: "17% = 10% + 5% + 1% + 1%: four pieces, a running total again" (`F-T5.seventeen.*`) | proportion bar | |
| 06_4 | ⟨predict⟩ Your turn. Fifteen percent of sixty dollars? | The quiz card: 15% of $60, ring from 5; at zero $9.00 lands (`F-Q.06_4.*`) | the quiz card | listen 5 s (countdown) |
| 06_5 | A tenth is six dollars. Half of that is three dollars. Nine dollars. | $6.00 + $3.00 on a bar of ten, the tenth and the half lit | proportion bar; TeX equation | |
| 06_6 | ⟨predict⟩ What's eight percent of twenty-five dollars? | 8% of $25 = ? as a card with no ring | prediction card | |
| 06_7 | Swap the two numbers. Twenty-five percent of eight dollars is a quarter of eight. Two dollars. That's the answer to both. | The 8 and the 25 cross over on arcs between fixed places: 25% of $8 = $2.00, then back: 8% of $25 = $2.00 (`F-T5.swap.*`) | TeX morph | |
| 06_8 | Percent means cents on the dollar. Eight cents from each of twenty-five dollars. Or twenty-five cents from each of eight. The same two hundred cents. The swap pays when one side is a friendly percent, like a tenth, a quarter or a half. | A grid 25 across and 8 deep, one cell a cent: first the 25 columns light one after another ("8¢ from each dollar"), then the 8 rows ("25¢ from each"); then the 200 cells flow into two squares of a hundred: $2.00 (`F-T5.swap.across`, `F-T5.swap.deep`, `F-T5.swap.cents`); then the friendly ones in a row: 10% a tenth, 25% a quarter, 50% a half; lower-third "precisely: x% of y = x × y ÷ 100 = y% of x" | **grid**; morph that keeps identity; lower-third | hold |

- **Real life** (the owner's test): a tip, a discount, a tax rate. A tenth,
  half of it, twice it.
- **Misconception defused:** "x percent of y and y percent of x are
  different questions" (06_7, 06_8).
- **The swap's picture** (C12): each cell is a cent, so turning the grid
  swaps what the rows and the columns mean, and "over a hundred" is the two
  squares of a hundred the cells fill.
- **Ruth:** the swap is exact for every x and y
  (`edge_the_small_tricks…`); its usefulness is not general, and 06_8 says
  when it pays. The tenth stops being one piece off 5, 10, 15 and 20 percent,
  and the grey line at 06_3 shows it (D4). Money is in whole cents
  throughout.

## 07_round_numbers (80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Five is half of ten. So five times eighty-six is ten times eighty-six, cut in half. Eight hundred sixty, cut in half. Four hundred thirty. | A rectangle 10 by 86, area 860; it is cut across the middle and the top half fades: 5 by 86 = 430 (`F-T6.five.*`) | the rectangle; morph | |
| 07_2 | The same cut works between two numbers. Fourteen times thirty-five. Take half of the fourteen, and double the thirty-five. Seven times seventy. Four hundred ninety. | A rectangle 14 by 35 is cut in half across; the top half slides down beside the bottom half, outlined gold while it moves: 7 by 70 = 490 (`F-T6.halve.*`) | the rectangle; morph that keeps identity | |
| 07_3 | ⟨predict⟩ Your turn. Sixteen times forty-five? | The quiz card: 16 × 45, ring from 5; at zero 720 lands (`F-Q.07_3.*`) | the quiz card | listen 5 s (countdown) |
| 07_4 | Half of one, double the other. Eight times ninety. Seven hundred twenty. It pays when doubling makes a round number. One of them even, the other ending in five. | 16 × 45 = 8 × 90 = 720, with the restack replayed small beside it; lower-third "14 × 37 = 7 × 74: true, and no easier" (`F-T6.no_easier.*`) | TeX morph; lower-third | |
| 07_5 | Take a price just under a round number. Four things at nineteen dollars ninety-nine. Call it four times twenty, eighty dollars. Then give back four cents. Seventy-nine dollars ninety-six. That's round, then fix. | Four price tags, $19.99; each rounds up to $20.00 and sheds a red cent; a rectangle 4 by 20 with a red sliver sliced off its end: $80.00 − $0.04 = $79.96 (`F-T7.price.*`) | lagged reveal; the rectangle; TeX equation | |
| 07_6 | ⟨predict⟩ Your turn. Three things at four dollars ninety-nine each? | The quiz card: 3 × $4.99, ring from 5; at zero $14.97 lands (`F-Q.07_6.*`) | the quiz card | listen 5 s (countdown) |
| 07_7 | Three times five, fifteen dollars. Give back three cents. Fourteen dollars ninety-seven. This one pays when the fix is small. A round number close by, and only a few things to fix. | $15.00 − $0.03 = $14.97; under it, grey: "7 × 34 = 280 − 42: no easier than 210 + 28" (`F-T7.no_easier.*`) | TeX equation | hold |

- **Two moves, one idea:** get to a round number, then put right what you
  changed, by halving (07_1, 07_2) or by taking the extra back off (07_5).
  Each is taught and then tried (M17). Both are the rectangle of
  chapter 04, cut another way.
- **Real life:** packs of five, and every price that ends in 99.
- **Where each stops** (D3, D4): halving and doubling is always true and
  pays only when doubling lands on a round number (07_4 and its
  lower-third); round, then fix pays only when the fix is small: a round number close by,
  and few things to fix (07_7 and
  its grey line).
- **Ruth:** each is exact (`edge_the_small_tricks…`).

## 08_the_middle (73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | ⟨predict⟩ Forty-seven times fifty-three. Before you start on the parts, look at the two numbers. What number sits exactly halfway between them? | 47 × 53; a number line fades in beneath, 47 and 53 marked, nothing between them yet | number line; prediction card | |
| 08_2 | It's fifty, the middle. One is three below it, the other three above. A distance of three. | 50 appears between them; two arcs of length 3 (`F-T3.middle`, `.distance`) | number line; highlight | |
| 08_3 | So draw fifty squared, and lay the rectangle over it. It sticks out by three at the side, and falls short by three at the top. | The number line becomes the picture: the tick at 50 grows into a square of 50 in outline, and the rectangle, 47 tall and 53 wide, sits over it; the two arcs become the strip that stands out past the square's right side and the gap along its top (`F-T3.strip_area`, `F-T3.gap_area`) | overlay; morph that keeps identity | |
| 08_4 | Move the strip that sticks out into the gap. It nearly fills it. What's missing is one small square, three by three. | **The one move, backwards:** the strip lifts off the side, turns and lies in the gap along the top, outlined gold while it moves; a 3 by 3 hole is left in the corner, outlined red, and a framed enlargement of that corner hangs beside it (`F-T3.hole`) | **the strip that moves**; magnifier | |
| 08_5 | So the answer is fifty squared, minus three squared. Twenty-five hundred, minus nine. Two thousand four hundred ninety-one. | 47 × 53 = 50² − 3² = 2500 − 9 = 2491 (`F-T3.*`); the red hole stays on screen; lower-third "(a + b)(a − b) = a² − b²: the difference of squares", a labelled "the middle", b "the distance" | TeX equation; lower-third | |
| 08_6 | ⟨predict⟩ Your turn. Nineteen times twenty-one? | The quiz card: 19 × 21, ring from 5; at zero 399 lands (`F-Q.08_6.*`) | the quiz card | listen 5 s (countdown) |
| 08_7 | The middle is twenty, and the distance is one. Four hundred, minus one. Three hundred ninety-nine. | Number line 19, 20, 21; 20² − 1² = 399; the move replayed small on a square of 20 | number line; TeX equation | |
| 08_8 | It needs a middle and a distance that are both easy to square. Forty-seven times fifty-six has its middle at fifty-one and a half. Back to parts. | 47 × 56 on the number line, its middle at 51½ marked red; lower-third "a whole middle needs both odd or both even"; under it, grey: "12 × 88 = 50² − 38²: true, and no easier" (`F-T3.no_middle.*`, `F-T3.far.*`) | number line; lower-third | hold |

- **The owner's "(a + b)*(a − b)"**, spoken as the middle and the distance so
  that the letter a is never read aloud; his letters are on screen here and
  again at 14_3, where the same line has one term moved across.
- **Misconception defused:** "this is a fact about fifty" (08_7
  uses 20).
- **Ruth:** exact for any two numbers with a whole middle, tested for every
  such pair up to 200 (`edge_equally_far_from_a_middle…`). 08_8 gives both
  things the trick needs and the test for a whole middle. A half middle
  still works and is not easier (47 × 56 = 51.5² − 4.5²); the description
  says so.
- **Engine:** the square is an outline and the rectangle a fill; the corner
  is 3 in 50, so it gets a magnifier (E2, C17).

## 09_bethe (52 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | The next trick comes with a story. Los Alamos, in the nineteen forties, where the first atomic bomb was being built. Hans Bethe runs the Theoretical Division, and Richard Feynman works for him. They're working through a formula, and it calls for forty-eight squared. | "Los Alamos"; two name cards, "Hans Bethe, physicist, head of the Theoretical Division" and "Richard Feynman, physicist, in Bethe's division"; between them 48²; the cards then shrink to captions at the sides | lagged cards | |
| 09_2 | Feynman turns to the calculator. Bethe already has an answer. Twenty-three hundred. | A desk calculator in outline, a keyboard of round keys and a row of four result digits, labelled "a Marchant calculator"; by Bethe's caption, 2300 (`F-C1.bethe.rough`) | lagged reveal | |
| 09_3 | Feynman keys it in all the same. Bethe adds that the exact answer is twenty-three oh four. | By Bethe's caption the 00 gives way to 04: 2304, and stays still; the machine's four result digits keep running for a second and a half with the narrator silent, then stop one by one on 2, 3, 0, 4 (`F-C1.bethe.exact`) | **the machine catches up**; live digits | listen 2 s (dry ticks, slowing) |
| 09_4 | Feynman calls it remarkable. And Bethe asks him a question. Don't you know how to square numbers near fifty? | The stage clears and the music stops for the question; quote card, verbatim: "Don't you know how to square numbers near 50?"; the source fades in after the hold, on two lines: Hans Bethe, as told by Richard P. Feynman, *"Surely You're Joking, Mr. Feynman!"* (1985), "Lucky Numbers" | quote card | hold |
| 09_5 | ⟨predict⟩ Fifty squared is twenty-five hundred. How would you get from there to forty-eight squared? | 50² = 2500 on the left, 48² = ? on the right; a square of 50 in outline with a square of 48 inside it, on the same corner; the two squares stay where they are for chapter 10, which answers the question with the picture | prediction card; the square on a grid | hold |

- **Every detail is from the book's text** (fact sheet, source S1, pp.
  193–94 of the 1985 Norton edition): the number, the calculator, the two
  answers in that order, Feynman's reaction, Bethe's question and his
  method. Two sentences are quoted verbatim in the whole video: Bethe's
  question here and Feynman's line at 11_4. Everything else is paraphrase,
  and "Bethe says it's twenty-three hundred" is indirect speech on purpose.
- **Who they are** (M4): "two physicists", and where; the cards say what
  each did there (fact sheet, S3).
- **What is not said:** what the formula was (the book does not say); that
  Bethe was surprised (the book gives his question and leaves it there; the
  staging carries it: his answer stands still while the machine is still
  running); what kind of machine a Marchant was (the label is the book's
  word, and the drawing is generic); anything from Feynman's 1975 talk.
- **Sound** (C8): one of the video's two silences is spent before the
  question at 09_4.
- **Bethe's way is shown, not told twice** (cut for length on 2026-10-13,
  docs/decisions.md): 09_5 asks how to get from 2500 to 48², and chapter 10
  answers with the picture, 2500 − 200 = 2300 (Bethe's first answer) and
  + 4 = 2304, a hundred for each one below fifty. The line that told the
  same three steps before the picture (the old 09_6) is gone.

## 10_near_fifty (72 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Here's the picture, and it's the move from before. Forty-eight squared. Slice a strip off the top, two deep, and stand it against the side. | **The one move:** on the square of 48 inside the outline of 50, a strip 2 deep comes off the top, turns and stands against the right side, outlined gold (`F-M.n48.strip_wide`) | **the strip that moves** | listen 2 s (the move's sound) |
| 10_2 | Now it's as wide as fifty squared, and four rows short. That's two hundred, a hundred for each one below fifty. And that's Bethe's twenty-three hundred. | The rectangle fills the outline's width; the empty band along its top, 4 rows deep, turns red: 4 × 50 = 200; 2500 − 200 = 2300 (`F-M.n48.tall`, `F-M.n48.wide`, `F-M.n48.rectangle_area`, `F-T4.below.fifties`, `F-T4.below.fifties_area`) | highlight; TeX equation | |
| 10_3 | And the end of the strip sticks up. Two by two. Plus four. | The end of the strip that stands above the rectangle turns orange; a framed enlargement of that corner beside it: 2 × 2 = 4; 2300 + 4 = 2304 (`F-M.n48.corner`, `F-M.n48.square`) | magnifier; TeX equation | |
| 10_4 | ⟨predict⟩ Now a number above fifty. What changes for fifty-three squared? | 53² = ?; the square of 53 over the outline of 50 | prediction card | |
| 10_5 | The same move. Slice off three, and stand it against the side. | **The one move** again, on the square of 53 over the outline of 50: a strip 3 deep off the top, stood against the side, outlined gold (`F-M.n53.strip_wide`) | **the strip that moves** | listen 2 s (the move's sound) |
| 10_6 | Now it's fifty tall, and six columns wider than fifty squared. That's three hundred, a hundred for each one above fifty. And the end sticks up. Three by three is nine. Two thousand eight hundred nine. | The rectangle is 50 tall and 56 wide, the six columns past the outline lit: 6 × 50 = 300; the orange end 3 × 3 = 9 in the magnifier; 2500 + 300 + 9 = 2809 (`F-M.n53.*`, `F-T4.above.fifties`, `F-T4.above.fifties_area`) | magnifier; TeX equation | |
| 10_7 | Which gives a quick way to say it. Start from twenty-five. Above fifty, add the distance. Below fifty, take it off. That's the hundreds. Then write the distance squared, as two digits. Nine is oh nine. | Two rows build: 25 + 3 = 28, 3² = 9 → 09, giving 28 09; 25 − 2 = 23, 2² = 4 → 04, giving 23 04 (`F-T4.above.digit_pairs`, `F-T4.below.digit_pairs`); lower-third "precisely: (50 ± b)² = 2500 ± 100b + b², b = the distance from 50: + above, − below" | TeX morph; lower-third | hold |

- **One picture, both directions, and it is the centrepiece's:** below
  fifty the rectangle is short of fifty squared by rows of fifty, two for
  each one below; above, it is wider by columns of fifty, two for each one
  above; either way that is the distance in hundreds, and the end of the
  strip is the distance squared. The small
  square is always added because it always sticks out (C1, R2).
- **Misconception defused:** "below fifty you subtract the distance squared
  too" (10_3, 10_6: the end of the strip sticks out both times).
- **Ruth:** 46 × 50 + 4 and 50 × 56 + 9 are counted cell by cell; the sign
  of the distance is spoken at 10_7 ("Above fifty, add … Below fifty, take
  it off"); the
  two-digit reading is exact for 41 to 59 and nowhere else from 0 to 100
  (`edge_near_fifty…`), and 11_3 says so. The precise line at 10_7 carries
  its signs.
- **Engine:** the strip is 2 or 3 in 50; the corner gets a magnifier.

## 11_your_turn_fifty (47 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | ⟨predict⟩ Your turn. Forty-six squared? | The moved-strip picture stays on the left, on a tracker; the quiz card: 46², ring from 5; at zero 2116 lands (`F-Q.11_1.*`) | the quiz card | listen 5 s (countdown) |
| 11_2 | Four below. Twenty-five minus four is twenty-one. Four squared is sixteen. Twenty-one sixteen. | The side slides to 46: the red band and the orange end follow; 25 − 4 = 21, 4² = 16 | live recomputation from a tracker; TeX equation | |
| 11_3 | That reads straight off while you're within nine of fifty. Further out, the distance squared runs past two digits, and it spills into the hundreds. Sixty-two is twelve above. Thirty-seven hundred, plus one hundred forty-four. Thirty-eight forty-four. | The side slides to 62: the orange end is now 12 by 12 and overflows one ruled square of a hundred; a hundred's worth peels off and joins the hundreds, 37 becomes 38, and 44 is left: 3844 (`F-T4.carries.*`); lower-third "reads straight off from 41 to 59" (`F-X4.digit_pairs.*`) | live recomputation; grid of hundreds | |
| 11_4 | After that, Feynman took up tricks like these himself. He almost never got there before Bethe, who could get close to nearly any answer. And he tells us what made that easy. Every number was near something he knew. | Quote card, verbatim: "It was easy for him—every number was near something he knew."; source on two lines: Richard P. Feynman, on Hans Bethe, *"Surely You're Joking, Mr. Feynman!"* (1985), "Lucky Numbers" | quote card | hold |

- **The chapter's picture** (C5): the square of fifty stays on stage and the
  answer and the limit are read off it; at 62 the corner overflows
  its hundred, which is the carry, seen.
- **Climb:** chapter 10 works two below and three above; the quiz is four
  below; five above closes the video at 14_1.
- **Ruth:** the limit is stated where the viewer has just used the trick
  (11_3); the formula is exact for every number, the reading only for 41 to
  59. 11_4 paraphrases three of the book's sentences in its own words (what
  was easy for Bethe was getting close to nearly any answer) and quotes one
  sentence, attributed to Feynman (R6); "he says", not "he writes": the
  book is as told to Ralph Leighton.

## 12_which_trick (38 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | ⟨predict⟩ Now they're mixed up, so the first job is to pick the trick. Pause whenever you like. A twenty percent tip on forty-five dollars? | Six tiles along the top, the small pictures of 01_7: the square with its strip, the cut rectangle, the bill, the rectangle with one cut, the number line with a middle, the square of fifty; they arrive during the first sentence, the quiz card after it: 20% of $45, ring from 5; at zero $9.00 lands (`F-Q.12_1.*`) | the quiz card; lagged tiles | listen 5 s (countdown) |
| 12_2 | Start from a tenth. Four dollars fifty. Doubled, nine dollars. | The bill tile lights and plays its move once inside the tile; $4.50 × 2 | highlight; TeX equation | |
| 12_3 | ⟨predict⟩ Forty-nine times fifty-one? | The quiz card: 49 × 51, ring from 5; at zero 2499 lands (`F-Q.12_3.*`) | the quiz card | listen 5 s (countdown) |
| 12_4 | One either side of fifty. Twenty-five hundred, minus one. Two thousand four hundred ninety-nine. | The middle tile lights and plays its move; 50² − 1² = 2499 | highlight; TeX equation | hold |

- **The mixed round the order asks for** runs from here to 14_1: the viewer
  must choose before computing, four times (a tip, a middle, a product with
  nothing to grab, and a square two tricks fit). The tiles are the six
  pictures in miniature (C6); the one that fits lights and plays its move.
  That two tricks can fit, and must then agree, is 14_2 (D10).

## 13_nothing_to_grab (53 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | ⟨predict⟩ Last one of this round. Sixty-seven times seventy-eight? | The quiz card: 67 × 78, ring from 6; the tiles stay dark; the ring empties into silence, and nothing lands (`F-Q.13_1.*`) | the quiz card | listen 6 s (countdown, with no last note) |
| 13_2 | Nothing to grab. It doesn't end in five. Its middle is seventy-two and a half. And the nearest round number doesn't make it easy. | The tiles stay dark; small notes land by the problem, one for each sentence: "no 5", "middle 72½", and, grey, "67 × 80 − 134: true, and no easier" (`F-N.middle_halves`, `F-N.round_fix.*`) | lagged reveal | |
| 13_3 | Parts still works, in four slow steps. Five thousand two hundred twenty-six. | Only the parts tile lights; the rectangle 67 by 78 in four pieces, and the total runs 4200, 4680, 5170, 5226 (`F-Q.13_1.steps`) | the rectangle cut into parts; live count-up | |
| 13_4 | That's four pieces with carrying, and it's slow. There's no shame in reaching for paper, or a calculator. | The sheet of paper from 01_4 slides back in with the four pieces written on it and their sum; the calculator of chapter 09 beside it | lagged reveal | |
| 13_5 | But keep one habit even then. Round first. Seventy times eighty is fifty-six hundred. If a calculator shows five hundred and something, or fifty thousand, something went wrong. That won't catch a small slip. It tells you how big the answer has to be. | 70 × 80 = 5600, tagged "about" (`F-N.estimate`); the calculator's digits read 522.6, then 52260, each crossed with two red lines; then 5226 beside the 5600 on a number line (`F-N.slipped_low`, `.slipped_high`) | highlight; number line | hold |

- **Honest framing** (the order, item 5): a product where no trick helps is
  shown and said to be one; paper and a calculator are named as the right
  tools; what the viewer keeps is the estimate, and what the estimate can
  and cannot catch is said at 13_5 (D12).
- **Ruth:** "nothing to grab" is true of the tricks in this video, and 13_2
  gives the reason for each: no five, a middle of 72½, and a round number
  that is close but has 67 things to fix (the grey line, D11). Parts is
  exact as always. The estimate tells how big the answer is and catches no
  small slip (13_5).
- **Sound** (C8): the second of the two silences is the end of 13_1's ring.

## 14_near_something_you_know (78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 14_1 | ⟨predict⟩ One more, and it has a twist. Fifty-five squared? | The quiz card: 55², ring from 5; at zero 3025 lands (`F-Q.14_1.*`) | the quiz card | listen 5 s (countdown) |
| 14_2 | It ends in five. Five times six is thirty, then twenty-five. It's also near fifty. Twenty-five plus five is thirty, then twenty-five. Thirty twenty-five, both ways. Any trick that fits gives the same answer. | Two tiles light together, the strip and the fifty; two rows, 5 × 6 = 30 then 25, and 25 + 5 = 30 then 25, the same 30 and the same 25 lining up (`F-T1.n55.*`, `F-T4.twist.*`) | highlight; TeX equation | |
| 14_3 | They're the same move. Slice a strip off a square, stand it against the side, and you have a rectangle and a small square. Thirty times forty, plus twenty-five. Forty-six times fifty, plus four. And fifty squared is forty-seven times fifty-three, plus nine. | **The one move, three times**, one square after another in step with the three sums: 35, 48, then 50; under them three lines in one shape, 35² = 30 × 40 + 25, 48² = 46 × 50 + 4, 50² = 47 × 53 + 9 (`F-M.n35.*`, `F-M.n48.*`, `F-M.n50.*`) | **the strip that moves**, three times | listen 2 s (the move's sound, three times) |
| 14_4 | In letters, it's the line from the middle trick, with the small square moved across. | The 25, the 4 and the 9 first become 5², 2² and 3²; then the 35 line turns into letters and the other two cross-fade into it: a² = (a − b)(a + b) + b², "the middle" under a and "the distance" under b; above it, small, the line from 08_5 | **TransformMatchingTex** | hold |
| 14_5 | You don't need a gift for numbers for any of this. You need a few numbers you know well, and the habit of looking for one nearby. Every number is near something you know. Sometimes that gives you the answer. It always tells you how big the answer is. | A number line from 0 to 110 with the round tens lit, 50 and 100 brightest; the video's own numbers drop on to it, 23, 46, 47, 53, 48, and last 67 and 78, each throwing a gold arc to the round number the video used for it (20, 50, 50, 50, 50, 70, 80); 35 and 85 sit halfway between two; Feynman's sentence from 11_4 returns above the line and three words change in place, "was" to "is", "he" to "you", "knew" to "know", its quotation marks fading and its source line shortening to "after Feynman" | number line; glow; TeX morph | hold |
| 14_6 | *(receipts)* | The receipts card: "every calculation in this video was checked by two independent programs", the repository link, and names to search: the distributive law, difference of squares, the area model; small: "three of these tricks are on one page of a school arithmetic of 1844 (Chase, p. 60)" (`F-C3.chase.*`) | receipts card | listen 4 s (the motif, answered) |

- **Ends on the rigorous form of the opening picture** (plan rule 15): the
  strip that moved in chapter 02, as one identity that contains the fives
  trick, the near-fifty trick and the middle trick. It is the owner's
  (a + b)(a − b) line with one term moved across.
- **Hindsight** (plan rule 19): 14_1 and 14_2 make the viewer find it: one
  square, two tricks, the same two numbers. Every picture recalled at 14_3
  was drawn this way before (02_6, 10_1, 08_4 backwards).
- **Ruth:** a² = (a − b)(a + b) + b² for all a and b
  (`edge_the_small_tricks…`); 48² = 46 × 50 + 4 is Bethe's 2300 + 4. The
  1844 page is a latest date for three tricks, not an origin, and rule (13)
  there is for 50 to 60; it is a small line on the receipts card and a
  paragraph in the description, no longer a spoken beat. The last sentence echoes
  Feynman's line in the second person and is not presented as a quotation;
  after "nothing to grab" it says what nearness always gives (R17). The
  arcs at 14_5 go to the round number the video used for each number, not
  to the nearest landmark of any kind.
