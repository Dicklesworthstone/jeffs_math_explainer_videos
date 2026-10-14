<!--
Draft video description for Video F (P3, 2026-10-13). Every statement below
rests on a row of docs/fact-sheet.md or an oracle key; the bracketed tags
name them. It holds the exact versions of what the narration simplifies
(plan rule 11), the programmer notes that have no room on screen, and what
review round R1 sent "to the description". The chapter timestamps are the
final cut's (./x mental_math chapters). The release package adds the
repository link, and trims this to what the upload page accepts, linking this
file for the rest.
-->

# Mental Math Tricks That Actually Work (and Why)

A handful of mental arithmetic tricks that come up in real life, each with
the picture that makes it true, and thirteen "your turn" quizzes with a
countdown. Pause whenever you want more time.

A product is the area of a rectangle. Every trick here is one way to cut that
rectangle into easy pieces, or to trade it for an equal one.

## Chapters

0:00 Eighty-five squared
0:58 Squares that end in five
2:05 Your turn
2:54 Any product, by parts
4:14 A running total
5:38 Percents: a tenth, and the swap
6:55 Round numbers: half and double; round, then fix
8:22 The middle
9:39 Feynman, Bethe and forty-eight squared
10:33 Squares near fifty
11:47 Your turn
12:39 Which trick?
13:18 Nothing to grab
14:14 Every number is near something you know

## The tricks, exactly

- **Squares ending in 5.** (10n + 5)² = 100 × n × (n + 1) + 25 for every
  whole number n from 0 up: the number in front, times the next number, then
  25 [F-X1, F-T1.n5]. It is only as easy as that one product: 135² needs
  13 × 14 = 182, giving 18225 [F-T1.n135]. With a decimal point it still
  works: 7.5² = 56.25, because 7 × 8 = 56 [F-T1.half].
- **The picture.** Slice a strip off the top of a square and stand it
  against the side. A square of side a becomes a rectangle a − b tall and
  a + b wide, with a b by b square left over: a² = (a − b)(a + b) + b², for
  every a and every b up to a [F-X8]. With b = 5 and a ending in 5 both
  sides of the rectangle are round tens, which is the fives trick
  [F-M.n35]. With a near 50 and b the distance, one side is 50, which is the
  near-fifty trick [F-M.n53]. Read from right to left it is the middle
  trick.
- **By parts.** (a + b)(c + d) = ac + ad + bc + bd, the distributive law.
  Added in a fixed order, every total on the way is a floor: the totals
  never go down, and once the tens are known the answer lies from the tens
  multiplied up to, but not at, the next tens multiplied (for 23 × 46, from
  800 up to 1500) [F-X2, F-T2.factors, F-T2.bounds]. A running total does
  not have the right leading digits early: 19 × 19 starts at 100 and ends at
  361 [F-DO.n19.number, F-DO.n19.running].
- **Percents.** x% of y = x × y ÷ 100 = y% of x, always [F-X5]. It helps
  only when one of the two is a percent you know as a fraction.
- **Half and double.** (a ÷ 2) × (2b) = a × b [F-X6]. It stays in whole
  numbers when a is even, and it pays when doubling b gives a round number:
  14 × 35 is 7 × 70, but 14 × 37 is 7 × 74, true and no easier
  [F-T6.halve, F-T6.no_easier].
- **Round, then fix.** n × (r − e) = n × r − n × e [F-X7]. It pays when the
  fix is small: 7 × 34 as 280 − 42 is no easier than 210 + 28
  [F-T7.no_easier].
- **The middle.** (a + b)(a − b) = a² − b², for any two numbers [F-X3]. Their
  middle is whole when both are even or both are odd. A half middle still
  works and is no easier: 47 × 56 = 51.5² − 4.5² = 2652.25 − 20.25 = 2632
  [F-D.half_middle.*].
- **Near fifty.** (50 ± b)² = 2500 ± 100b + b² [F-T4.fifty,
  F-T4.fifty_squared]. "25 plus or minus the distance, then the distance
  squared as two digits" reads the answer straight off for 41 to 59 and for
  no other number from 0 to 100; further out the square of the distance
  carries into the hundreds (62² is 37 hundreds plus 144, 3844)
  [F-X4.digit_pairs, F-X4.near_fifty.largest, F-T4.carries]. The same move
  works near a hundred: 97² = 94 × 100 + 9 = 9409 [F-D.n97.*].
- **Two more the same picture gives.** Any two numbers with the same front
  whose last digits add to ten: 23 × 27 is 2 × 3 in hundreds, then 3 × 7:
  621 [F-D.same_front.*].
- **The estimate.** Rounding each number to its nearest ten tells you how
  big the answer is. It catches a slipped decimal point, not a wrong digit.

## The Feynman and Bethe story

The story is told by Richard Feynman in *"Surely You're Joking, Mr.
Feynman!": Adventures of a Curious Character* (as told to Ralph Leighton,
edited by Edward Hutchings; W. W. Norton, 1985), in the chapter "Lucky
Numbers", pages 193 and 194 of that edition [F-C1.feynman], about the
physicist Hans Bethe at Los Alamos. The video takes the number (48), the
calculator, Bethe's two answers (2300, then 2304) and their order
[F-C1.bethe], Feynman's reaction, Bethe's question and his method from that
text, and quotes two of its sentences. Feynman told the story again, with
different words and a different example, in a 1975 lecture printed as "Los
Alamos From Below"; nothing in the video comes from that version. The book does not say
what the formula was.

## For programmers

- The four pieces of a product by parts are schoolbook multiplication.
  Karatsuba's algorithm gets both strips from one more multiplication of
  the digit sums: for 23 × 46, (2 + 3) × (4 + 6) − 8 − 18 = 24, in tens.
  Three multiplications instead of four, and more adding: a good trade for a
  computer multiplying very long numbers, a bad one for a head
  [F-C4.karatsuba, F-T2.factors].
- Biggest piece first is an anytime answer: stop after any step and you
  have a lower bound that only rises. Its price is that a later piece can
  change a digit already said (920 becomes 1040) [F-T2.running]. Paper
  works from the small end so that a written digit never changes.
- The number of pieces is the digits of one number times the digits of the
  other: two for 6 × 43, four for 23 × 46 [F-T2.piece_counts,
  F-T2.two_pieces.factors, F-T2.factors]. Every trick in the video is a way
  to need fewer pieces.

## Not new

Three of these tricks are on one page of an American school book of 1844:
Pliny E. Chase, *The Elements of Arithmetic*, Part Second (Philadelphia:
Uriah Hunt and Son), page 60. Its rule (10) squares numbers ending in 5,
with 35 × 35 = 1225 as the example; rule (12) is the sum and difference of
two numbers, with 87 × 73 = 80² − 7² = 6351; rule (13) squares numbers
between 50 and 60, with 53 × 53 = 2809 [F-C3.chase]. That is the latest
these tricks can have been in print, not where they began.

## Receipts

Every calculation in the video was checked by two independent programs: the
Rust code that draws the pictures works each trick the way the video does,
and a Python oracle gets the same numbers by plain multiplication and by
counting unit squares. Sources, what was and was not seen, and the edge test
behind every "always" are in `docs/fact-sheet.md`.

Names to search: the distributive law; difference of squares; the area
model; Karatsuba multiplication.
