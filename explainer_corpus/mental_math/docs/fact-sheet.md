# Fact sheet: Video F, "Mental Math Tricks That Actually Work (and Why)"

Bead `mx-e-a-f-ms4.2` (gate G1). One row per number the video may show or
say, and one row per claim that rests on a source. Every value in the tables
at the end was copied from the oracle's output by a script, not typed.

```
./x mental_math facts-check        # mental facts == oracle.py, key by key
```

- **Result, 2026-10-13** (the build host; CPython 3.14), with the script as
  it stands after review R1 and the claim walk: `facts-check mental_math:
  304 video facts against 304 oracle keys: PASS`. The oracle prints 304 keys
  and passes all of its own cross-checks. (At G1, the day before: 277 of
  277.)
- **Rust side:** `./x mental_math test`: 20 tests pass, clippy `-D warnings`
  and `fmt --check` clean. One of them, `every_spoken_number_is_a_fact`,
  spells 161 facts the way the voice reads them and requires each in its
  script line (`docs/claims.md` has the walk that preceded it).
- **Rows:** F-H the hook (the paper method); F-T the tricks; F-M the one
  move, on every square the video cuts; F-N the product with nothing to
  grab; F-D numbers only the description states; F-Q the quizzes; F-X the edges of the always sentences, each as "how
  many hold of how many checked"; F-C cited literals.

## How the two sides get their numbers

| What | `src/model.rs` (the video) | `oracle.py` (independent) |
|---|---|---|
| every answer | by the trick, the way the video works it in the head (the number in front times the next; pieces as a running total; the middle squared less the distance squared; 25 plus the distance; a tenth and half of it; half of ten times; round up, then take the extra off) | plain multiplication of Python integers and Fractions, never the trick |
| the one move (a strip sliced off a square's top and stood against its side) | sides and areas as sums and products | the moved figure built as a set of unit cells; the cells inside the rectangle and the cells that stand above it are counted |
| the other pictures (the 23 by 46 rectangle, the 47 by 53 rectangle over the square of 50, the grid of cents) | areas as products of the sides | every unit cell put in its piece and the pieces counted |
| the two-digit reading of a square near 50 | 25 plus the distance, and the distance squared padded to two digits | the decimal string of the plain square, sliced before its last two digits |
| the paper method (rows, held carries) | long multiplication on digits | a second long multiplication, written on digit strings |
| the numbers a worked quiz shows | from the same trick functions | written from the trick's description, then cross-checked against the plain answer |
| the edges of the always sentences | each trick against `x * y` over its whole stated range | the same ranges, the identities written out directly |
| cited numbers | constants in `facts::cited`, each also pinned against the video's own arithmetic where there is any (Bethe's 2300 and 2304, Chase's three examples) | the same literals, typed from the sources |

A carry counts as "held" when another digit product of the same row, or
another column of the sum that still has digits in it, comes after it. For
85 × 85 that is one in each row and one in the sum: three. They are not
held together, and the script does not say so.

## Findings (uncomfortable first)

1. **The book never says Bethe was surprised.** The owner remembers "Hans
   Bethe being surprised he didn't know that trick". The text gives
   Feynman's reaction in his own words and then Bethe's question, which the
   video quotes; any surprise is in the question. The script does not say
   "surprised".
2. **Feynman told this story twice, differently.** The book (1985) and his
   1975 talk "Los Alamos From Below" (printed 1976) disagree in wording and
   in detail: the talk sets it on Bethe's first day, makes the number a
   pressure, has Bethe say "about" 2300, works 47 as the example, and has
   Feynman losing races "maybe one out of four". None of that is in the
   book, and the book's own chapter with the talk's title leaves the story
   out. The video uses the book alone and mixes in nothing from the talk.
   Popular retellings (Gleick's *Genius* as quoted online, forum posts)
   follow the talk or add details neither text has.
3. **A running total does not have "the right digits early".** A first
   draft of a programmers' note said so; review R1 caught it. The first
   total of 23 × 46 is 800 and the answer 1058; for 19 × 19 it is 100
   against 361, and over all pairs of two-digit numbers the first piece has
   the answer's leading digit and length in fewer than half. What is true
   for every pair: each total is a floor, the totals never go down, and
   once the tens are known the answer lies from the tens multiplied up to
   (not at) the next tens multiplied (800 to 1500 here). That is what the
   script says, and the edge test pins 19 × 19.
4. **"25 plus the distance, then the distance squared" reads off the answer
   only from 41 to 59.** At ten away the square is 100 and carries into the
   hundreds (62² is 37 hundreds plus 144, 3844). The formula is exact for
   every number; the two-digit reading is not, and the script says where it
   stops (`F-X4.digit_pairs.*`). Spoken without "as two digits" the rule
   gives 289 for 53²; the words now say it.
5. **Each of the everyday tricks pays only sometimes.** Halving and
   doubling is always true and helps when doubling lands on a round number
   (14 × 37 becomes 7 × 74, no easier). Round then fix helps when the fix is
   small: a round number close by, and few things to fix (7 × 34 as 280 − 42
   is no easier than 210 + 28; 67 × 78 as 67 × 80 − 134 is no easier than
   parts). A tenth
   stops being one piece off 5, 10, 15 and 20 percent (17% of $48 is four
   pieces). The swap helps when one side is a percent known as a fraction.
   Each limit is on screen where the trick is taught.
6. **The rule of 72 was researched and left out.** Against interest added
   once a year the rule gives 36 years at 2% (exact 35.0), 9 at 8% (9.01),
   3 at 24% (3.22) and 2 at 36% (2.25): good from about 2% to 12% (within
   3%), poor beyond. Explaining why 72, and where it drifts, took about
   105 s in the storyboard estimate. Its history is better than usually
   told: Pacioli's *Summa* (1494), fol. 181 recto, item 44, read in
   facsimile, states the rule, says to keep it in mind, gives two examples
   (at 6 percent 12 years, at 8 percent 9) and no reason; the transcription
   everyone copies (Vacca, 1915) drops the second example. If the owner
   wants the trick it replaces a chapter or becomes a short video of its
   own.
7. **Nothing found tests the owner's central claim directly.** "The hardest
   part is remembering the 4 numbers" matches what the literature says about
   mental addition (holding partial results loads working memory: Fürst and
   Hitch 2000; Hitch 1978 as reported by others), but no experiment was
   found that compares a running total with four stored partial products.
   The script counts the numbers to hold (four, against one between steps)
   and claims no study and nothing about what people get wrong.
8. **The tricks are old school-book material, and no older origin is
   claimed.** Chase's *Elements of Arithmetic*, Part Second (Philadelphia,
   1844), p. 60, prints the rule for squares ending in 5 with the owner's
   own example (35 × 35 = 1225), the sum-and-difference rule (87 × 73 = 80²
   − 7² = 6351) and squares between 50 and 60 (53 × 53 = 2809); its rule is
   for 50 to 60 only, not for numbers below fifty. That is a latest date,
   not a first appearance. Claims that these are "Vedic" rest on a book of
   1965; the video says nothing about origins.
9. **Feynman's dates at Los Alamos are not agreed.** The book's own
   biographical page (seen only in e-text copies) has him there until 1946;
   other sources have him leaving in the autumn of 1945. The script says
   "in the nineteen forties" and nothing narrower. That he led a group at
   the time of the story is not established, so his card says only that he
   was a physicist in Bethe's division.
10. **Bethe's name cannot be verified by speech recognition.** Wikipedia's
    lead gives an English reading with "th", uncited; two dictionaries and
    the German give BAY-tuh, which is the target. `michael` reads the plain
    spelling as "Beeth", so the name goes to the voice respelled ("Bayta").
    The recogniser writes "Beta" for that and, equally, for a deliberately
    wrong "Beeta", so it cannot tell the two apart; the vowel was checked by
    measuring formants in one carrier sentence (`lexicon.json` has the
    numbers). Nobody has listened to it. The owner's pronunciation
    overrides. Feynman, Los Alamos and the spoken letter n read correctly
    from their plain spellings.
11. **One citation was seen only as a summary.** Karatsuba and Ofman's paper
    is Doklady Akademii Nauk SSSR 145(2), 1962, 293–294 by the Math-Net.Ru
    record as a search engine reported it; the record itself would not load,
    and one encyclopedia page gives volume 146. The lower-third names the
    algorithm and gives no year. Its first draft was circular (it used the
    product of the two whole numbers, which is the problem itself); the
    claim walk caught it. It now works on the digits, (2 + 3) × (4 + 6) − 8
    − 18 = 24, in tens, and the oracle checks that none of the three
    products is 23 × 46 (`F-C4.karatsuba.*`).
12. **The page images of two sources were not seen.** Feynman's passage was
    read through the full-text search of two separate scans of the 1985
    hardcover and two e-text copies, which agree on every sentence used
    here; Chase's page 60 through the OCR text of one scan. Both are to be
    looked at as images before the script is final (bead `.5`).
13. **What a Marchant calculator was is not sourced here.** The book names
    it and no more. The script says "the calculator", the label on screen is
    the book's word, and the drawing is a generic desk machine.

## Sources

Downloaded copies and text extractions live outside the repository and are
never committed. "Seen" means read on the page (or its image, or its
extracted text as stated) by this lane or by its research agent.

| id | Source | Seen | Used for |
|---|---|---|---|
| S1 | Richard P. Feynman, *"Surely You're Joking, Mr. Feynman!": Adventures of a Curious Character*, as told to Ralph Leighton, edited by Edward Hutchings (New York: W. W. Norton, 1985; ISBN 0-393-01921-7), Part 4, "Lucky Numbers", pp. 192–198; the Bethe passage is on pp. 193–94 | the passage, through full-text search of two scans of the 1985 hardcover and two e-text copies (finding 12); the contents page and the index; the library catalogue record (LCCN 84014703) | chapter 09, the quote at 11_4 |
| S2 | Richard P. Feynman, "Los Alamos From Below: Reminiscences 1943–1945", *Engineering and Science* 39(2), Caltech, January–February 1976 (adapted from a 1975 lecture at Santa Barbara) | the journal's own web page | finding 2 only; nothing in the video |
| S3 | Los Alamos context: Bethe's biography on the Nobel Prize site (from *Nobel Lectures, Physics 1963–1970*), which has him at "the Los Alamos Scientific Laboratory which was engaged in assembling the first atomic bomb"; David Hawkins, *Manhattan District History, Project Y, The Los Alamos Project*, vol. 1 (LAMS-2532), which lists the Theoretical Division's groups under H. A. Bethe as Division Leader, among them "T-4 Diffusion Problems R. P. Feynman"; *Physics Today*, "Hans in war and peace": "Soon after the government created the Los Alamos Laboratory in March 1943 … Hans was appointed head of the lab's theoretical division"; and Feynman's own sentence in S1 that he became a group leader under Bethe | the passages named | 09_1's words and its two name cards |
| S4 | Pliny E. Chase, *The Elements of Arithmetic, for Schools and Academies … Part Second* (Philadelphia: Uriah Hunt and Son, 1844), pp. 59–60 | title page and pp. 59–60, as OCR text of a Google scan | the line on the receipts card; the description; finding 8 |
| S5 | Department for Education (UK), *Teaching Children to Calculate Mentally* (2010; ref. 00365-2010PDF-EN-01) | the passages on multiplying by 5 ("multiplying by 10 then halving"), on 50 and 25, and on finding 15% from 10% and 5% | that the everyday tricks are mainstream teaching, not curiosities |
| S6 | Arthur Benjamin and Michael Shermer, *Secrets of Mental Math* (2006) | full-text search snippets only | background: left-to-right working; "take 10% of the bill" |
| S7 | Ansgar J. Fürst and Graham J. Hitch, "Separate roles for executive and phonological components of working memory in mental arithmetic", *Memory & Cognition* 28(5), 2000, 774–782 | whole paper | finding 7 |
| S8 | Graham J. Hitch, "The role of short-term working memory in mental arithmetic", *Cognitive Psychology* 10, 1978, 302–323 | **not seen**; cited in S7 and summarised in a 2012 review | finding 7 |
| S9 | A. Karatsuba and Yu. Ofman, "Multiplication of many-digital numbers by automatic computers", *Doklady Akademii Nauk SSSR* 145(2), 1962, 293–294 | **catalogue record as a search summary only** (finding 11) | the lower-third at 04_8 |
| S10 | Luca Pacioli, *Summa de arithmetica, geometria, proportioni et proportionalita* (Venice: Paganino de Paganini, 1494), fol. 181 recto, item 44; and G. Vacca's 1915 transcription in the *Napier Tercentenary Memorial Volume*, pp. 163–164 | the facsimile page (read by this lane) and Vacca's text | finding 6 only; nothing in the video |

### Passages quoted or leaned on

Feynman's book is in copyright. Two of its sentences are quoted, on screen
and nowhere else in this repository; everything else from it is paraphrase.

- **S1, p. 193** (Bethe, as Feynman tells it): "Don't you know how to square
  numbers near 50?"
- **S1, p. 194** (Feynman on Bethe): "It was easy for him—every number was
  near something he knew."
- **S1, pp. 193–94, in paraphrase:** at Los Alamos the two were putting
  numbers into a formula and came to 48 squared; Feynman reached for the
  Marchant calculator; Bethe gave 2300; Feynman began to push the buttons
  and Bethe added that the exact value is 2304; the machine agreed and
  Feynman called it remarkable; Bethe asked his question and explained:
  square 50, which is 2500, subtract 100 times the difference of the number
  from 50, which gives 2300, and for the correction square the difference
  and add it on. A few minutes later Bethe gave the cube root of two and a
  half as about 1.35 from logarithms he knew. Feynman says he then tried to
  do such things himself, that he had fun doing arithmetic fast by tricks
  with Bethe, and that it was very rare for him to see something Bethe did
  not and beat him to an answer; that Bethe could nearly always get the
  answer to any problem within a percent; and then the sentence quoted
  above, whose "It" is that getting of answers.
- **S4, p. 60** (public domain), rule (10): "To square a number ending in 5,
  multiply the number of tens by one more than itself, and place 25 at the
  right of the product." Rule (12): "The sum of two numbers multiplied by
  their difference, is equal to the difference of their squares." Rule (13):
  "To square any number between 50 and 60, add the units of the given number
  to 25 for the hundreds, and annex the square of the units for the tens and
  units." Page 59 has multiplying by 5 and by 25 (as dividing by .2 and by
  .04) and by a row of 9s (add one to the multiplier, then subtract the
  multiplicand).
- **S5:** "multiply by 5 by multiplying by 10 then halving, e.g. 18 × 5 =
  180 ÷ 2 = 90".

## Terms (the jargon ledger)

| Term | First said at | As spoken | On screen |
|---|---|---|---|
| squared | 01_1 | (assumed: Maya knows it) | 85² |
| the number in front | 02_1 | "the number in front of the five" | label under the digit; `n` once, at 03_6 |
| strip, small square | 02_5, 02_7 | "Slice a strip off the top"; "A small square, five by five" | green and orange (storyboard §3) |
| round first | 04_1 | "Before anything else, round first." | "about" beside the estimate |
| parts | 04_3 | "Break each number into parts. Twenty and three." | 20 + 3 on the rectangle's side |
| pieces; block, strips, corner | 04_3 | "four pieces. A big block, two strips and a small corner." | the colours of storyboard §3 |
| running total | 05_1 | "Keep one running total, and add each piece the moment you have it." | one large readout |
| floor | 05_3 | "each total was a floor, so the answer could only go up" | a shaded stretch of the number line |
| a tenth | 06_1 | "Ten percent is one tenth." | one of ten slices of the bill |
| percent | 06_8 | "Percent means cents on the dollar." | a grid of cents |
| the swap; friendly percent | 06_7, 06_8 | "Swap the two numbers."; "like a tenth, a quarter or a half" | 10%, 25%, 50% with their fractions |
| round, then fix | 07_5 | "Call it four times twenty, eighty dollars. Then give back four cents. … That's round, then fix." | a rectangle with a red sliver off its end |
| the middle, the distance | 08_2 | "It's fifty, the middle. … A distance of three." | a number line; `a`, `b` in lower-thirds |
| the distributive law, difference of squares, the area model | never spoken | | lower-thirds at 04_7 and 08_5; the receipts card |

## Simplification → exact

| Said | Exact | Where the exact form is |
|---|---|---|
| "the number in front, times the next number, then twenty-five" | (10n + 5)² = 100 × n × (n + 1) + 25 for every whole n ≥ 0 | on screen at 03_6 |
| "five shorter … five wider … a small square, five by five" | a² = (a − b)(a + b) + b² with a = 35, b = 5 | the line at 02_9; in letters at 14_4 |
| "tens times tens are hundreds" | (a × 10)(b × 10) = ab × 100 | the tiling at 04_5 |
| "each total was a floor, so the answer could only go up" | the partial sums are non-decreasing, and for two-digit numbers 100ac ≤ xy < 100(a + 1)(c + 1), a and c the tens digits | the shaded number line at 05_3 |
| "swap the two numbers" | x% of y = x × y ÷ 100 = y% of x | lower-third at 06_8 |
| "half of one, double the other" | (a ÷ 2)(2b) = ab; whole numbers only when a is even | 07_2's picture |
| "round, then fix" | n(r − e) = nr − ne | 07_5's picture |
| "fifty squared, minus three squared" | (a + b)(a − b) = a² − b² | lower-third at 08_5 |
| "start from twenty-five. Above fifty, add the distance. Below fifty, take it off. … then write the distance squared, as two digits" | (50 ± b)² = 2500 ± 100b + b²; reads off as two pairs of digits only for b ≤ 9 | lower-thirds at 10_7 and 11_3 |
| "the same move" | a² = (a − b)(a + b) + b² | on screen at 14_4 |
| "two rows of digits and three carries" | for 85 × 85, with the convention above | `F-H1.paper.*` |

## Always sentences and their edge tests

| Sentence (line) | Test in `src/facts.rs` | Range, and the edge |
|---|---|---|
| "It works for every whole number that ends in five" (03_5) | `edge_the_square_of_every_number_ending_in_five_is_the_front_times_the_next_then_25` | 5 to 99,995; 5 itself (nothing in front); 95 and 995 (the front ends in 9); no other last digit is accepted |
| the pieces add up, zeros come back by size, "each total was a floor" (04, 05_3) | `edge_the_pieces_always_add_up_to_the_product` | every product up to 99 × 99; round numbers and single digits have fewer pieces; 19 × 19 starts at 100, so nothing is said about leading digits |
| "fifty squared, minus three squared" (08_5) | `edge_equally_far_from_a_middle_is_the_middle_squared_less_the_distance_squared` | every pair up to 200 with a whole middle; distance 0; an odd sum has no whole middle |
| "That reads straight off while you're within nine of fifty." (11_3) | `edge_near_fifty_is_exact_and_the_two_digit_reading_stops_at_ten_away` | the formula for 0 to 100; the reading for 41 to 59 and no other number |
| the swap, halving and doubling, round then fix, the one move (06, 07, 14_3) | `edge_the_small_tricks_hold_over_their_whole_ranges` | every x, y up to 200; every even number up to 200; every count up to 99 and price up to 999; every square up to 200 and every strip no wider than it |
| "Any trick that fits gives the same answer." (14_2) | each trick's own edge test (every trick is an identity, so two that fit must agree), and `F-T4.twist.same_two_numbers` for the square the line is said about | 55²: the fives trick and near fifty both give 30, then 25 |
| every countdown is a quiz with the same seconds | `every_countdown_is_a_quiz_and_every_quiz_has_its_countdown` (`src/main.rs`) | all 14 |

## The numbers

Copied from `oracle.py`'s output by a script; `mental facts` prints the same keys and values.

### Quizzes

| line | asks | countdown | trick | the numbers the answer shows | answer |
|---|---|---|---|---|---|
| 01_1 | 85^2 | 4 s | ends_in_five | 8, 9, 72 | 7225 |
| 03_1 | 25^2 | 5 s | ends_in_five | 2, 3, 6 | 625 |
| 05_4 | 6x43 | 5 s | by_parts | 240, 258 | 258 |
| 05_6 | 13x14 | 6 s | by_parts | 100, 140, 170, 182 | 182 |
| 06_4 | 15% of $60.00 | 5 s | percent_blocks | 6.00, 3.00 | 9.00 |
| 07_3 | 16x45 | 5 s | halve_and_double | 8, 90 | 720 |
| 07_6 | 3x$4.99 | 5 s | round_then_fix | 5.00, 15.00, 0.03 | 14.97 |
| 08_6 | 19x21 | 5 s | pair | 20, 1, 400, 1 | 399 |
| 11_1 | 46^2 | 5 s | near_fifty | -4, 21, 16 | 2116 |
| 12_1 | 20% of $45.00 | 5 s | percent_blocks | 4.50 | 9.00 |
| 12_3 | 49x51 | 5 s | pair | 50, 1, 2500, 1 | 2499 |
| 13_1 | 67x78 | 6 s | by_parts | 4200, 4680, 5170, 5226 | 5226 |
| 14_1 | 55^2 | 5 s | ends_in_five | 5, 6, 30 | 3025 |

### Every key

| key | value |
|---|---|
| `F-H1.paper.rows` | 425, 6800 |
| `F-H1.paper.row_count` | 2 |
| `F-H1.paper.carries` | 3 |
| `F-H1.paper.product` | 7225 |
| `F-H1.trick.front` | 8 |
| `F-H1.trick.next` | 9 |
| `F-H1.trick.hundreds` | 72 |
| `F-H1.trick.square` | 7225 |
| `F-T1.n35.front` | 3 |
| `F-T1.n35.next` | 4 |
| `F-T1.n35.hundreds` | 12 |
| `F-T1.n35.square` | 1225 |
| `F-T1.n105.front` | 10 |
| `F-T1.n105.next` | 11 |
| `F-T1.n105.hundreds` | 110 |
| `F-T1.n105.square` | 11025 |
| `F-T1.n135.front` | 13 |
| `F-T1.n135.next` | 14 |
| `F-T1.n135.hundreds` | 182 |
| `F-T1.n135.square` | 18225 |
| `F-T1.n5.front` | 0 |
| `F-T1.n5.next` | 1 |
| `F-T1.n5.hundreds` | 0 |
| `F-T1.n5.square` | 25 |
| `F-T1.n55.front` | 5 |
| `F-T1.n55.next` | 6 |
| `F-T1.n55.hundreds` | 30 |
| `F-T1.n55.square` | 3025 |
| `F-T1.sweep.fronts` | 1, 2, 3, 4, 5, 6, 7, 8, 9, 10 |
| `F-T1.sweep.hundreds` | 2, 6, 12, 20, 30, 42, 56, 72, 90, 110 |
| `F-T1.sweep.squares` | 225, 625, 1225, 2025, 3025, 4225, 5625, 7225, 9025, 11025 |
| `F-T1.half.squared` | 56.25 |
| `F-M.n35.strip_wide` | 5 |
| `F-M.n35.strip_long` | 35 |
| `F-M.n35.wide` | 40 |
| `F-M.n35.tall` | 30 |
| `F-M.n35.rectangle_area` | 1200 |
| `F-M.n35.corner` | 25 |
| `F-M.n35.square` | 1225 |
| `F-M.n34.strip_wide` | 4 |
| `F-M.n34.strip_long` | 34 |
| `F-M.n34.wide` | 38 |
| `F-M.n34.tall` | 30 |
| `F-M.n34.rectangle_area` | 1140 |
| `F-M.n34.corner` | 16 |
| `F-M.n34.square` | 1156 |
| `F-M.n48.strip_wide` | 2 |
| `F-M.n48.strip_long` | 48 |
| `F-M.n48.wide` | 50 |
| `F-M.n48.tall` | 46 |
| `F-M.n48.rectangle_area` | 2300 |
| `F-M.n48.corner` | 4 |
| `F-M.n48.square` | 2304 |
| `F-M.n53.strip_wide` | 3 |
| `F-M.n53.strip_long` | 53 |
| `F-M.n53.wide` | 56 |
| `F-M.n53.tall` | 50 |
| `F-M.n53.rectangle_area` | 2800 |
| `F-M.n53.corner` | 9 |
| `F-M.n53.square` | 2809 |
| `F-M.n50.strip_wide` | 3 |
| `F-M.n50.strip_long` | 50 |
| `F-M.n50.wide` | 53 |
| `F-M.n50.tall` | 47 |
| `F-M.n50.rectangle_area` | 2491 |
| `F-M.n50.corner` | 9 |
| `F-M.n50.square` | 2500 |
| `F-M.n55.strip_wide` | 5 |
| `F-M.n55.strip_long` | 55 |
| `F-M.n55.wide` | 60 |
| `F-M.n55.tall` | 50 |
| `F-M.n55.rectangle_area` | 3000 |
| `F-M.n55.corner` | 25 |
| `F-M.n55.square` | 3025 |
| `F-M.n62.strip_wide` | 12 |
| `F-M.n62.strip_long` | 62 |
| `F-M.n62.wide` | 74 |
| `F-M.n62.tall` | 50 |
| `F-M.n62.rectangle_area` | 3700 |
| `F-M.n62.corner` | 144 |
| `F-M.n62.square` | 3844 |
| `F-M.n35.hundreds_tiles` | 3x4 |
| `F-M.n35.hundreds` | 12 |
| `F-T2.parts.first` | 20, 3 |
| `F-T2.parts.second` | 40, 6 |
| `F-T2.pieces.sides` | 20x40,20x6,3x40,3x6 |
| `F-T2.pieces` | 800, 120, 120, 18 |
| `F-T2.running` | 800, 920, 1040, 1058 |
| `F-T2.product` | 1058 |
| `F-T2.size.lead` | 8 |
| `F-T2.size.zeros` | 2 |
| `F-T2.size.too_small` | 80 |
| `F-T2.size.too_big` | 8000 |
| `F-T2.estimate` | 20, 50, 1000 |
| `F-T2.estimate.lead` | 10 |
| `F-T2.estimate.zeros` | 2 |
| `F-T2.held.pieces` | 4 |
| `F-T2.bounds` | 800, 1500 |
| `F-T2.piece_counts` | 2, 4 |
| `F-T5.bill` | 48.00 |
| `F-T5.bill.ten` | 4.80 |
| `F-T5.bill.five` | 2.40 |
| `F-T5.bill.fifteen` | 7.20 |
| `F-T5.bill.twenty` | 9.60 |
| `F-T5.seventeen.pieces` | 4.80, 2.40, 0.48, 0.48 |
| `F-T5.seventeen.total` | 8.16 |
| `F-T5.swap.across` | 25 |
| `F-T5.swap.deep` | 8 |
| `F-T5.swap.cents` | 200 |
| `F-T5.swap.one_way` | 2.00 |
| `F-T5.swap.other_way` | 2.00 |
| `F-T6.five.ten` | 860 |
| `F-T6.five.product` | 430 |
| `F-T6.halve.halved` | 7 |
| `F-T6.halve.doubled` | 70 |
| `F-T6.halve.product` | 490 |
| `F-T6.no_easier.halved` | 7 |
| `F-T6.no_easier.doubled` | 74 |
| `F-T6.no_easier.product` | 518 |
| `F-T6.halve.check` | 490 |
| `F-T7.price.each` | 19.99 |
| `F-T7.price.round` | 20.00 |
| `F-T7.price.rounded` | 80.00 |
| `F-T7.price.fix` | 0.04 |
| `F-T7.price.total` | 79.96 |
| `F-T7.no_easier.rounded` | 280 |
| `F-T7.no_easier.fix` | 42 |
| `F-T7.no_easier.product` | 238 |
| `F-T7.no_easier.by_parts` | 210, 238 |
| `F-T3.middle` | 50 |
| `F-T3.distance` | 3 |
| `F-T3.middle_squared` | 2500 |
| `F-T3.distance_squared` | 9 |
| `F-T3.product` | 2491 |
| `F-T3.strip_area` | 141 |
| `F-T3.gap_area` | 150 |
| `F-T3.hole` | 9 |
| `F-T3.no_middle` | true |
| `F-T3.no_middle.sum` | 103 |
| `F-T3.far.middle` | 50 |
| `F-T3.far.distance` | 38 |
| `F-T3.far.distance_squared` | 1444 |
| `F-T3.far.product` | 1056 |
| `F-T4.fifty_squared` | 2500 |
| `F-T4.below.number` | 48 |
| `F-T4.below.distance` | -2 |
| `F-T4.below.hundreds` | 23 |
| `F-T4.below.rough` | 2300 |
| `F-T4.below.corner` | 4 |
| `F-T4.below.corner_fits` | true |
| `F-T4.below.square` | 2304 |
| `F-T4.below.fifties` | 4 |
| `F-T4.below.fifties_area` | 200 |
| `F-T4.above.number` | 53 |
| `F-T4.above.distance` | 3 |
| `F-T4.above.hundreds` | 28 |
| `F-T4.above.rough` | 2800 |
| `F-T4.above.corner` | 9 |
| `F-T4.above.corner_fits` | true |
| `F-T4.above.square` | 2809 |
| `F-T4.above.fifties` | 6 |
| `F-T4.above.fifties_area` | 300 |
| `F-T4.carries.number` | 62 |
| `F-T4.carries.distance` | 12 |
| `F-T4.carries.hundreds` | 37 |
| `F-T4.carries.rough` | 3700 |
| `F-T4.carries.corner` | 144 |
| `F-T4.carries.corner_fits` | false |
| `F-T4.carries.square` | 3844 |
| `F-T4.carries.fifties` | 24 |
| `F-T4.carries.fifties_area` | 1200 |
| `F-T4.above.digit_pairs` | 28, 09 |
| `F-T4.below.digit_pairs` | 23, 04 |
| `F-T4.carries.hundreds_after` | 38 |
| `F-T4.carries.left` | 44 |
| `F-T4.twist.digit_pairs` | 30, 25 |
| `F-T4.twist.same_two_numbers` | true |
| `F-N.product` | 5226 |
| `F-N.estimate` | 70, 80, 5600 |
| `F-N.slipped_low` | 522.6 |
| `F-N.slipped_high` | 52260 |
| `F-N.middle_halves` | 145 |
| `F-N.has_whole_middle` | false |
| `F-N.ends_in_five` | false |
| `F-N.round_fix.rounded` | 5360 |
| `F-N.round_fix.fix` | 134 |
| `F-N.round_fix.total` | 5226 |
| `F-D.n97.tall` | 94 |
| `F-D.n97.wide` | 100 |
| `F-D.n97.corner` | 9 |
| `F-D.n97.square` | 9409 |
| `F-D.same_front.hundreds` | 6 |
| `F-D.same_front.ones` | 21 |
| `F-D.same_front.product` | 621 |
| `F-D.half_middle.middle_squared` | 2652.25 |
| `F-D.half_middle.distance_squared` | 20.25 |
| `F-D.half_middle.product` | 2632 |
| `F-Q.count` | 14 |
| `F-Q.countdown_seconds` | 72 |
| `F-X1.ends_in_five.holds` | 10000 |
| `F-X1.ends_in_five.checked` | 10000 |
| `F-X2.by_parts.holds` | 9801 |
| `F-X2.by_parts.checked` | 9801 |
| `F-X3.pair.holds` | 10100 |
| `F-X3.pair.checked` | 10100 |
| `F-X4.near_fifty.holds` | 101 |
| `F-X4.near_fifty.checked` | 101 |
| `F-X5.swap.holds` | 40401 |
| `F-X5.swap.checked` | 40401 |
| `F-X6.halve.holds` | 20000 |
| `F-X6.halve.checked` | 20000 |
| `F-X7.round_then_fix.holds` | 98901 |
| `F-X7.round_then_fix.checked` | 98901 |
| `F-X8.one_move.holds` | 20300 |
| `F-X8.one_move.checked` | 20300 |
| `F-X1.ends_in_five.largest` | 99995 |
| `F-X4.digit_pairs.first` | 41 |
| `F-X4.digit_pairs.last` | 59 |
| `F-C1.feynman.year` | 1985 |
| `F-C1.feynman.pages` | 193-94 |
| `F-C1.bethe.number` | 48 |
| `F-C1.bethe.rough` | 2300 |
| `F-C1.bethe.exact` | 2304 |
| `F-C3.chase.year` | 1844 |
| `F-C3.chase.page` | 60 |
| `F-C3.chase.square` | 1225 |
| `F-C3.chase.pair` | 6351 |
| `F-C3.chase.near_fifty` | 2809 |
| `F-C4.karatsuba.multiplications` | 3 |
| `F-C4.karatsuba.block` | 8 |
| `F-C4.karatsuba.corner` | 18 |
| `F-C4.karatsuba.sums` | 50 |
| `F-C4.karatsuba.strips_in_tens` | 24 |
| `F-C4.karatsuba.product` | 1058 |
