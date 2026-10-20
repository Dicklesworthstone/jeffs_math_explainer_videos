# Storyboard — Check a Giant Matrix Product Without Doing the Product Again

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py freivalds` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact computed in
`src/facts.rs` and recomputed another way by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One tiny example runs through the whole video: A = (1 2 1 / 0 3 1 /
3 2 0), B = (2 1 0 / 1 0 2 / 2 1 3), and the claim C, their product with
one slip (row 2, column 2 says 2; the product has 1). The true product's
nine entries are the digits 1 to 9, each once. It is chapter 01's opening
picture; chapters 02 and 03 work on its mistake, C − AB, and its tosses
(chapter 03 sets it aside for the tosses seen first); chapter 04 leaves it
for the cost at scale and two 2 × 2 examples with N = 10¹⁶ in them, and its
last beat brings the opening picture back.

| Chapter | Spine | Status |
|---|---|---|
| `01_check_with_coins` | someone hands over C; redoing it is nine entries of three multiplications, a billion at a thousand rows; one spot check misses a single wrong entry eight times in nine; a coin for each column: add C's heads columns, add B's and multiply by A (nine multiplications); they must match if C is right, and do not; of the eight tosses, how many miss? Four, exactly the tails ones over the wrong entry's column; no wrong answer hides from more than half, so toss again: twenty fresh tosses all miss less than once in a million | source and silent visual checkpoint accepted by root at `94988fd`, on root's evidence, not this lane's: central06 ran fmt, strict release clippy, 32 tests, the release build and the 92 native/oracle facts with 17 oracle checks; root read all six glyph pages and the changed and prediction frames of draft02 (2,524 frames, empty layout lint); central07 reran the changed SPOKEN test. Narration not generated; no audio accepted |
| `02_never_more_than_half` | the mistake on its own, C − AB: the check misses exactly when its heads columns add up to zero in every row (C·r − A(B·r) = (C − AB)·r); on the same A and B (`model::SHAPES`), two in one row, both one too big, hide only from tails on both coins (2 of 8); one too big and one too small by the same amount hide whenever their coins agree (4 of 8, a second equality case); one, one and minus one in a row hide from three (a prediction; not a power of two); one wrong entry per row and column hides only from all tails (1 of 8); the proof: pair the tosses that differ only in the coin of a column where C is wrong (with three coins, coin 1 pairs each toss with the one under it); across a pair that row's sum moves by the wrong entry, so at most one of each pair hides it; the pairs cover every toss once, at any size. The 19,682 and 624 small mistakes are a finite check, labelled as one | source, native checks and sampled silent visuals accepted by root for `.23.2`, on root's evidence, not this lane's: central14 passed fmt, strict release all-target clippy, 42 tests and the build, with 150 native/oracle fact rows equal, 20 oracle checks and 81 glyph sources with zero failures; root read the final changed source, all eight glyph pages, nine cue-end frames and twelve pairing and prediction frames of draft04 (chapter 02: 2,483 frames, 82.766667 s, silent 960 × 540 at 30 fps, empty layout lint, SHA-256 `8dd755a7…d5bd8fd`); EmeraldLeopard's independent image review (Mail 3894) found no blocker. After `.23.3`'s 02_3 wording change ("by the same amount"), root's central22 re-rendered it: 2,529 frames, 84.30 s, SHA-256 `12136612…d72d0b`. Narration not generated; no audio, continuous playback or full-video acceptance |
| `03_toss_again` | two fresh tosses: 16 of the 64 ways they can land both miss the slip, exactly a quarter for this slip (four misses of the first toss times four of the second), and for any wrong C at most a quarter (the theorem's bound squared, kept apart from the example); one toss reused for both checks is the grid's diagonal, still half; a toss whoever wrote C saw first can be beaten (each of the eight has a wrong C it lets through, `model::blind_spot`: one wrong entry under a tails coin, or under all heads a cancelling pair in one row), while fresh tosses catch each of those wrong Cs half the time; so fresh means new coins for every check, tossed after C is fixed | source, native checks and sampled visuals accepted by root for `.23.3`, on root's evidence, not this lane's: central22 passed fmt, strict release all-target clippy, 48 native tests and the release build, with 177 native/oracle fact rows equal and 108 glyph sources with zero failures; chapter 03 is 2,371 frames, 79.033333 s, SHA-256 `2911f66e…84d9d9d`; root read the actual cue, reveal and card frames and the new glyph pages; FuchsiaHawk (Mail 3997) independently verified the final source, decoded pixels and card dwell. Narration not generated; no audio, continuous motion, full-video or publication acceptance |
| `04_exact_and_rounded` | the cost at scale, counted in multiplications and not in seconds (twenty checks at a thousand rows: twenty million against a billion, a fiftieth); the theorem is about exact arithmetic: in doubles 10^16 + 1 is stored as 10^16 (a prediction; computed, the doubles there two apart), so a right answer is caught on one toss and a wrong one by one passes all four (`model::ROUNDED_*`, every sum left to right as the loops run); a tolerance lets the right one through, but then a wrong C inside it passes every toss, a different test with no half guarantee; the ending: the opening picture with the exact promise, fair independent coins tossed after C is fixed, in exact arithmetic | source, native checks and sampled silent visuals accepted by root for `.23.4`, on root's and FuchsiaHawk's evidence, not this lane's: central36 passed fmt, strict Clippy, 55 tests with script coverage, 146 glyph sources with zero failures, build and render without input drift (207 native/oracle fact rows equal at central32, facts unchanged since); draft11's chapter 04 is 2,372 frames, 79.066667 s, 960 × 540 at 30 fps, empty layout lint, no audio stream, SHA-256 `7faf2967…5636b9`; root read six final frames, and FuchsiaHawk (Mail 4089) measured the decoded pixels: the 04_3 answer readable 6.0 s, the prediction hold and the dot's move kept, the schoolbook label fitting, the equality rows dimmed. Narration not generated; no audio, continuous motion, full-video or publication acceptance |

## Vocabulary (one word per idea)

**matrix** (a grid of numbers: A, B and the claim C; "grid" only on first
sight); **entry**; **product** (A times B, row times column; "the schoolbook
method" in the exact version); **answer** or **claim** (C, what someone
handed over); **coin** (one per column); **heads** keeps a column, **tails**
leaves it out; **toss** (one landing of every coin: on screen the coins, and
`r` in the formulas, never "vector" in the voice); **check** (add B's heads
columns, multiply by A, compare with C's heads columns added); **match**;
**miss** (a toss that lets a wrong C through; never "false positive" in the
voice); **caught** (the columns disagree, so C is wrong for certain); **spot
check** (recompute one entry); **fresh** (tossed independently, after C is
fixed; "independently" in the exact version); **the mistake** (C minus the true
product, entry by entry; "C − AB" on screen); **hide** (the mistake's side
of a miss: a mistake hides from a toss that misses it); **pair** (two
tosses that differ only in one coin; each toss's **partner**); **way** (one
landing of the coins: "the eight ways three coins can land", and in
chapter 03 "the sixty-four ways two tosses can land"; never "pair", which
is chapter 02's); **reuse** (one toss used for more than one check);
**slip** (the claim's one wrong entry, row 2, column 2, and so its
mistake: C − AB is a single 1 there; chapter 01's voice calls it "a single
wrong entry" and "the wrong entry", the screen calls it "the slip" from
chapter 02, and chapter 03's voice does too); **double** (the 64-bit
numbers many programs compute with; "IEEE double"
in the exact version; the voice never says "floating point" or
"rounding", the screen shows it); **tolerance** (a gap the check lets
pass instead of exact equality).

## Color roles

| Idea | Color |
|---|---|
| matrix A's name, and A·(B·r) | blue `#58C4DD` |
| matrix B's name, and B·r | violet `#C792EA` |
| the claim C's name, and C·r | gold `#F0AC5F` |
| a heads coin, and the columns it keeps | teal `#5CD0B3` |
| a tails coin | `faint`, its face `dim` |
| a toss that catches the mistake, and the verdict | green `#83C167` |
| the mistake: the wrong entry's column, its true value, a toss that misses it, the ladder's chance bars | red `#FC6255` |
| the think ring and "?"; in chapter 02 a pair's link, its change, and the ring on the paired coin; in chapter 03 the diagonal of one toss reused, and the row where all heads cancels | `hero` `#FFD866` |
| entries, brackets, signs | `ink`, `dim` |

## Sources and scope

The statements the video makes, where they were checked, and what was not
read or watched, are in the crate's [README](../README.md). The video claims
only the exact theorem; the floating-point boundary is chapter 04's, built on
the F-X4 facts.

## 01_check_with_coins: A check with coins (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Someone multiplied two matrices, A and B, and handed you the answer, C. Is C really A times B? You could multiply them again. | A and B appear as grids of numbers between brackets, a dim × between them, their names in blue and violet under them; on "and handed you" a dim = and the claim C, its name in gold. On "Is C really" a gold ? over the =. On "You could multiply" row 1 of A, column 3 of B and entry (1, 3) of C are outlined, and the right-hand panel works that entry by hand: 1·0 + 2·2 + 1·3 = 7 | grids built from the model, highlight boxes, TeX built from the drawn entries | |
| 01_2 | Nine entries, three multiplications each, twenty-seven in all. With a thousand rows and columns, a billion. | The panel adds 9 × 3 = 27, "multiplications to redo the product"; on "With a thousand" 1000³ = 1,000,000,000, "for a thousand rows and columns", and a "precisely" lower-third: the schoolbook count, and that faster methods exist but none is known to need only about n² steps (it stays until the coins arrive in 01_4) | TeX built from facts, lower-third | |
| 01_3 | Checking one entry at random is cheap, but it misses a single wrong entry eight times in nine. | The counts go; the worked entry stays: "one entry: 3 multiplications". On "but it misses": "an entry picked at random", then in red "misses a single wrong one", "8 times in 9" (a fixed entry would always or never catch it) | panel from facts | |
| 01_4 | Here is a better check. Toss a coin for each column of C, and add up the columns that land heads. | The panel and outlines clear. On "a coin for each column" three coins land over C's columns: heads, heads, tails (teal discs marked 1, a faint disc marked 0), "heads keeps a column". On "add up the columns" C's two heads columns are outlined in teal and its tails column dims; copies of the kept entries fly down side by side with plus signs, and become their sums, C·r = (8, 7, 11), named in gold | coins, highlight, copies fly, column addition | |
| 01_5 | Do the same to B, then multiply by A. That takes nine multiplications, not twenty-seven. | The same toss lands over B's columns; its heads columns add into B·r = (3, 1, 3), named in violet, quicker than the first time. On "then multiply by A" a "for programmers" lower-third (B·r and C·r only add columns, A·(B·r) is n² multiplications against n³), and each row of A flashes with B·r while A·(B·r) = (8, 6, 11) fills in under A, row by row, named in blue. On "That takes" the panel: "this check: 9 multiplications", "redoing it: 27" | coins, column addition, lagged reveal, lower-third | |
| 01_6 | Both columns must match if C is right. They don't, so C is wrong, for certain. | B's coins and B·r clear. A·(B·r) slides to stand beside C·r; the panel gives the reason, A(B r) = (AB) r, "so a right C gives two equal columns". On "They don't" =, ≠, = appear between the rows; the middle pair, 6 and 7, is outlined in red. On "so C is wrong" a green verdict box: "C is wrong" | TeX identity, signs from the model, verdict box | |
| 01_7 | But that was one toss. Of the eight ways three coins can land, how many miss the wrong entry? ⟨predict⟩ | The columns, reason and verdict clear; the toss's coins stay over C on "But that was one toss". On "Of the eight ways" they go, and all eight tosses appear in two rows of four, in counting order (000 to 111), no verdicts; the question on screen, "How many of these 8 tosses miss the wrong entry?". Four seconds of silence with the gold countdown ring and "pause for more time" | lagged reveal, think countdown | |
| 01_8 | Check all eight. Four of them miss, exactly half. | The ? goes; each toss gets its verdict in turn, "miss" in red or "caught" in green (the left two of each row miss). On "Four of them" the question becomes "4 of 8 miss: exactly half" | lagged reveal from the model's checks | |
| 01_9 | Because every miss leaves out the column that holds the wrong entry. | C's column 2 is outlined in gold, the wrong cell alone in red, and "2 should be 1" beside its row. On "leaves out the column" every toss's middle coin is ringed: red around the tails coin of each miss, green around the heads coin of each catch | highlight, rings | |
| 01_10 | But no wrong answer hides from more than half the tosses. So toss again. Twenty fresh tosses all miss less than once in a million. | The tosses clear. A "precisely" lower-third, with the line's first sentence: with fair coins, all independent and tossed after C is fixed, each toss misses any fixed wrong C with chance at most half, in exact arithmetic. A ladder of the theorem's bound, one half a toss, a faint track for a chance of one with a red bar to scale: "1 toss" ≤ 1/2; on "So toss again" "2 tosses" ≤ 1/4; on "Twenty fresh tosses" "20 tosses" ≤ 1/1,048,576 < 1/1,000,000, and no bar: its track says "too small to see" (to scale it would be about a thousandth of a pixel). It stands until the card has had time to be read, then the chapter fades | bars from the bound, TeX fractions, lower-third | |

### Notes for 01

- Prediction (01_7): the question is on screen by the narrator's "Of the
  eight ways", well before the question's last word; four seconds of silence
  with the countdown; no verdict, count or ring is on screen until 01_8
  starts. The answer lands on "Four of them", after the verdicts.
- Exact versions kept on screen: 01_2 (the schoolbook count; faster methods
  exist, none is known to need only about n² steps), 01_5 (for programmers:
  only additions for B·r and C·r, n² multiplications against n³), 01_6 (why
  a right C must match: A(Br) = (AB)r, exactly), 01_10 (fair coins, all
  independent and tossed after C is fixed; at most half a toss for any
  fixed wrong C; exact arithmetic; the ladder is the theorem's bound, and
  the example meeting it exactly is the separate 01_8 fact). 01_3 says the
  spot check picks its entry at random. All three are two lines. The closing
  card is held for its length at fifteen characters a second; the other two
  stay through the following line, which the draft's cue sheets should
  confirm is long enough to read them.
- The scene refuses to render if a spoken claim stops being true: the
  example must have exactly one wrong entry, exactly half the tosses must
  miss, and twenty tosses must beat a million (`facts::wrong_entry`, the
  01_8 guard, `ladder_tex`). The bound itself is checked on every small
  mistake each time the facts are built.
- Narration units (`narration_units.json`): an **unqualified proposal, not
  frozen**. The coherent route is under independent review after the
  owner's 2026-10-19 rejection, and no take has been requested or generated
  for this video. Six units of 21 to 36 words. Two boundaries are open
  questions, not decisions: the think question ends 01u4 and its answer
  opens 01u5, which is the question-to-answer request boundary root's
  current experiment is testing; and the in-unit line splits (01_2|01_3,
  01_4|01_5, 01_6|01_7, 01_8|01_9) need their seams measured once takes
  exist. Regroup the units by that experiment's disposition, not by this
  note. What the grouping was written for: shorter requests (speech sped up
  inside longer ones), the rhetorical question in the same line as its
  answer (01_1), line splits before stop consonants ("Checking", "Do",
  "But", "Because"), and unit ends on a vowel or a nasal ("again", "nine",
  "twenty-seven", "entry", "entry", "million").
- Pause targets, for listening, not rules: after "Is C really A times B?"
  about one second before the answer; before "Here is a better check" and
  before each "But", a turn of about 0.85 s; after "exactly half", about
  0.7 s before its reason. The think window is the scene's own countdown.
- Pronunciation to probe before any synthesis: the letter names A, B and C
  as matrix names (A must not be read as the article; "by A" ends a
  sentence). No proper name is spoken in this chapter; "Freivalds" waits
  for a later chapter and its own lookup.

## 02_never_more_than_half: Never more than half (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Look at the mistake on its own, C minus the true product. The check misses exactly when its heads columns add up to zero in every row. | The mistake grid of chapter 01's slip (zeros, and a 1 outlined in red), "C − AB" under it in red. On "The check misses" chapter 01's first toss lands over its columns (heads, heads, tails), the heads columns are outlined in teal, and their sum appears beside it, (0, 1, 0), with the identity C r − A(B r) = (C − AB) r above and "not all zero: caught" in green. The tally, top right, starts with "the slip (1): 4 of 8" | grid from the model, highlight boxes, TeX identity | |
| 02_2 | Two in one row, both one too big, hide only when both coins land tails. Two of eight. | The aids clear; the grid becomes 0, 1, 1 in row 2, its two wrong entries outlined. All eight tosses come in, in counting order, two rows of four, and under each its row-2 sum, red where it is zero (000 and 100). On "Two of eight" the tally adds "1, 1 in one row: 2 of 8" | grid swap, lagged reveal, sums from the model | |
| 02_3 | But one too big and one too small by the same amount hide whenever their coins agree. Four of eight, half again. | The grid becomes 1, −1 in row 2 and the sums change with it; the zeros (000, 011, 100, 111) turn red. On "Four of eight" the tally adds "1, −1 in one row: 4 of 8" | grid swap, sums from the model | |
| 02_4 | Now one, one and minus one in a row. How many tosses does it hide from? ⟨predict⟩ | The grid becomes 1, 1, −1 in row 2; the sums go. On "How many tosses" the question on screen, "How many of these 8 tosses does it hide from?"; four seconds of silence with the gold countdown ring and "pause for more time", no sum on screen | think countdown | |
| 02_5 | Three. Not half, and not a power of two. | The ? and the question go; the sums come in (0, −1, 1, 0 over 1, 0, 2, 1), three of them red; the tally adds "1, 1, −1 in one row: 3 of 8" | lagged reveal from the model | |
| 02_6 | Put one wrong entry in each row and column, and it hides only from all tails. | The grid becomes one 1 in each row and each column; with more than one wrong row there is no single row sum, so under each toss its verdict, "miss" only under 000 and "caught" under the rest. On "only from all tails" the tally adds "1 per row and column: 1 of 8" | grid swap, verdicts from the model | |
| 02_7 | Why never more than half? Pick a column where C is wrong. Pair the tosses that differ only in its coin. | The grid returns to 1, 1, −1 and the sums with it. A "checked" lower-third: all 19,682 three by three mistakes with entries −1, 0 or 1 hide from at most half the tosses; a finite check, and the pairing proves every size. On "Pick a column" column 1 of the grid is outlined in gold and the first coin of every toss is ringed in gold. On "Pair the tosses" a gold link joins each top toss to the one under it, its partner | lower-third, highlight, rings, links | |
| 02_8 | Because that row's sum changes across a pair by the wrong entry, both can't be zero. At most one of each pair misses. | The individual entry boxes in row 2 give way to one red row outline, with the gold column outline distinct; beside each link its change, "+1", the wrong entry. On "At most one" the three zero sums are ringed in red and flash: one in each of three pairs, none in the fourth | highlight, labels from the model | |
| 02_9 | Pairs cover every toss once, so at most half of all tosses miss, for matrices of any size. | The "checked" card gives way to a "precisely" lower-third: exact arithmetic, flips move row sums by the wrong entry; with fair coins, all independent and tossed after C is fixed, a toss misses with chance at most half. At the right, one after another: "8 tosses, 4 pairs", "at most 1 miss in each pair", "at most 4 of 8"; on "for matrices of any size" the TeX 2ⁿ tosses, 2ⁿ⁻¹ pairs. It stands until the card has had time to be read, then the chapter fades | lagged reveal, TeX, lower-third | |

### Notes for 02

- Prediction (02_4): the question is on screen from "How many tosses", before
  the question's last word; four seconds of silence with the countdown; no
  sum, ring or tally line for the new mistake until 02_5 starts. The answer
  is spoken first ("Three.") and the sums come in with it.
- The proof and the checks are kept apart. The pairing (02_7 to 02_9) is the
  argument for every size: with coin j of a column where C is wrong, the
  tosses fall into pairs that differ only in coin j, and across a pair the
  row's sum moves by exactly that wrong entry, so at most one of the pair
  can be zero. The 19,682 and 624 small mistakes are said on screen to be a
  finite check (`facts::check_bound` and the F-X1 rows), not the proof.
  The 02_9 count's "at most 1 miss in each pair" is the proof's constant
  (`facts::MISSES_PER_PAIR`); the example's pairs are checked against it
  and the scene refuses to draw if one holds more. The column must be one
  where C is wrong: paired by coin 1, over a column the slip is right in,
  its row sum never moves and two pairs miss twice (F-X5.right_column).
- Counts and chances are kept apart too: the voice gives counts of tosses
  ("Two of eight", "at most half of all tosses"); the chance, and the fair,
  independent, after-C and exact-arithmetic conditions it needs, is the
  02_9 lower-third. The equality cases (the slip, and one too big with one
  too small) are separate facts from the bound.
- Edge tests behind the always-sentences (F-X5): two of one sign in a row
  hide only from tails on both their coins (18 sign and size cases from −3
  to 3); one too big and one too small by the same amount hide exactly when
  their coins agree (6 cases); one wrong entry per row and column hides only from all tails
  (384 patterns, entries ±1 and ±2). "Wrong in every column" alone is not
  enough: the 1, 1, −1 row is wrong in every column and hides from three
  (`F-X5.three_in_a_row.wrong_in_every_column`), so 02_6 says per row and
  column.
- With three coins the first coin is the grid row, so coin 1's pairs stand
  one above the other (the scene refuses to draw them otherwise).
- Narration units: the same unqualified, unfrozen proposal as chapter 01's,
  waiting on root's question-to-answer experiment. 02_4's question ends its
  unit and 02_5's answer opens the next. Seven units of 16 to 40 words
  (02u2, with 02_3's "by the same amount", is the longest);
  the only line splits inside a unit fall before "But" (02_3) and "Put"
  (02_6), and the proof's two lines (02_7, 02_8) are separate units.
- Pause targets, for listening, not rules: after "Why never more than
  half?", about one second before its answer in the same line; before each
  "But", "Put" and "Because", a turn of about 0.85 s.
- Pronunciation to probe: "minus one" in 02_4, and the letter C as a name
  ("C minus the true product", "where C is wrong").

## 03_toss_again: Toss again (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | One toss misses the slip half the time. So toss again, with fresh coins. | The slip's mistake, C − AB, at the top left (its zeros dim, its 1 outlined in red). The eight tosses written as their coins, 000 to 111, in a column in counting order, each red if it misses the slip and green if it catches it; at the right "one toss: 4 of 8 miss", with a small "toss labels: miss caught" key above it, using the corresponding red and green colors. On "So toss again" the column sorts, the four misses moving to the top, and a copy of it turns to run along the top: "first toss" beside the column, "second toss" over the row | labels from the model, moves | |
| 03_2 | Two tosses can land sixty-four ways. How many of them miss both times? ⟨predict⟩ | The grid of the 64 ways fills in, a faint square for each first toss and second toss. On "How many of them" the question at the right, "How many of these 64 ways miss both times?"; four seconds of silence with the gold countdown ring and "pause for more time"; no square is red | grid of squares, think countdown | |
| 03_3 | Sixteen, exactly a quarter. Both must miss, four ways for the first toss times four for the second. | The ? and the question go; the 16 squares where both tosses miss turn red, filling the top-left corner, and the answer "16 of 64 ways miss both times". On "four ways for the first toss" the four rows of the first toss's misses are outlined in red; on "times four for the second" the four columns of the second toss's misses; "4 × 4 = 16" | lagged reveal from the model, highlight | |
| 03_4 | For this slip, that quarter is exact. Any wrong C gets past two fresh tosses at most a quarter of the time. | The outlines go. At the right "this slip: 16/64 = 1/4". On "Any wrong C" a second line, with extra vertical clearance between the stacked fractions, "any wrong C: ≤ 1/2 · 1/2 = 1/4", and a "precisely" lower-third: in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C with chance at most 1/2 to the k | TeX from facts, lower-third | |
| 03_5 | But reuse one toss for both checks, and nothing changes. Four of the eight still miss, half. | The diagonal, the eight ways that use one toss twice, is outlined in gold and the rest of the grid dims. On "Four of the eight" its four red squares flash, and at the right "same toss twice: 4 of 8, still 1/2" | highlight, dim | |
| 03_6 | Fresh also means tossed after C is handed over. Someone who sees your toss first can write a wrong C it misses. | The grid, its labels, the toss-label key and the slip clear, and the card gives way to a second "precisely" lower-third: from two by two up, any toss seen first can be beaten, one wrong entry under a tails coin, or under all heads a cancelling pair in one row. The eight tosses come back as coins, two rows of four. On "can write a wrong C" a little grid fills under each: the mistake of the wrong C written for that toss (`model::blind_spot`), its zeros dim and its wrong entries outlined in red, and under it "miss" in red. At the right, under where the tally will stand, "each grid shows" over C − AB in red names them as mistakes, not Cs, and stays to the end | coins, grids from the model, lower-third | |
| 03_7 | Put one wrong entry under a tails coin, and the toss never looks at it. Against all heads, put one entry too big and one too small by the same amount in one row, and they cancel. | In the seven tosses with a tails coin, the tails coin over the wrong entry is ringed in red. On "Against all heads" the first row of 111's grid is outlined in gold, and beside it "1 + (−1) = 0" | rings, highlight | |
| 03_8 | So every one of the eight tosses can be beaten. Yet against fresh tosses, each of these wrong answers is caught half the time. | The eight "miss" verdicts flash, and at the top right "8 of 8 tosses beaten when seen first". On "Yet against fresh tosses" a green line along the bottom, "fresh tosses catch each of these wrong Cs: 4 of 8". It stands until the card has had time to be read, then the chapter fades | flash, text from facts | |

### Notes for 03

- Prediction (03_2): the question is on screen from "How many of them",
  before the question's last word; four seconds of silence with the
  countdown; no red square until 03_3 starts. The answer is spoken first
  ("Sixteen") and the squares turn red with it.
- The example and the theorem are kept apart. "Exactly a quarter" is this
  slip's count, 16 of the 64 ways (F-F5.two, F-F9.two.example_meets_bound).
  "At most a quarter" for any wrong C is the theorem's per-toss bound
  squared (F-F9.two.bound, from `facts::theorem_bound`, the pairing
  proof's), and the card gives 1/2^k for k tosses. Neither is derived from
  the other.
- Fresh has two halves, and the chapter shows each failing.
  - Independent across checks (03_5): one toss reused is the grid's
    diagonal, 4 of 8, above the two fresh tosses' bound
    (F-F9.same_toss.above_two_bound).
  - Tossed after C is fixed (03_6 to 03_8): every toss seen first is
    beaten. F-X3.known_toss.fooled finds all 8 by search; the wrong Cs
    shown are `model::blind_spot`'s, each passed by its own toss in the
    check itself (`facts::blind_spots` refuses otherwise).
  - The edges in size: at two by two every toss has a blind spot too
    (F-X3.n2.fooled, found by search in the oracle); at one by one a heads
    coin has none (F-X3.n1.heads_fooled), hence the card's "from two by two
    up".
- Each of those wrong Cs is still a fixed wrong C: fresh tosses catch it 4
  of 8 times (F-X3.blind_spot.fresh_catches), so the defence is freshness,
  not the toss.
- The grid lists the tosses misses first (000, 001, 100, 101, then 010,
  011, 110, 111; F-F9.order), so the 16 squares where both miss fill one
  corner (F-F9.both_miss_fill_the_corner; the scene refuses to draw it
  otherwise). The same order on both sides keeps the reused toss on the
  diagonal.
- Narration units: the same unqualified, unfrozen proposal as chapters 01
  and 02's. 03_1 and 03_2 share a unit that ends on the question, and 03_3's
  answer opens the next. Seven units of 17 to 37 words; the longest, 03u6
  (03_7), says the two opposite errors are the same amount, which is what
  makes them cancel.
- Pause targets, for listening, not rules: before "But reuse", "Fresh
  also" and "Yet against", a turn of about 0.85 s.
- Pronunciation to probe: "sixty-four", and the letter C as a name ("Any
  wrong C", "after C is handed over").

## 04_exact_and_rounded: Exact and rounded (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | At a thousand rows, twenty fresh checks take twenty million multiplications, a fiftieth of the billion to redo the product. That counts work, not seconds. | n = 1000 at the top. A faint track and a full bar for redoing the product, "redo the product (schoolbook): n³ = 1,000,000,000", the schoolbook count of 01_2 and not a bound on every method. On "twenty fresh checks" a second track, "20 fresh checks: 20 × n² = 20,000,000", and a blue bar to scale, a fiftieth of the first. On "a fiftieth" the ratio, 20,000,000 / 1,000,000,000 = 1/50. On "That counts work" a "for programmers" lower-third: these count multiplications, not time; adding up the heads columns of B and C still takes additions, and every entry is read from memory. The card stays for its characters / 15 seconds from its completed fade-in, then fades out before 04_2 starts | bars to scale from facts, TeX, lower-third | |
| 04_2 | But many programs compute with doubles, which skip some whole numbers. What is ten to the sixteen plus one in doubles? ⟨predict⟩ | The cost clears. "N = 10¹⁶" at the top left, and a number line with the whole numbers N − 2 to N + 2. On "What is ten to the sixteen" the question, "What is N + 1 in doubles?"; four seconds of silence with the gold countdown ring and "pause for more time"; no double is marked yet | number line, think countdown | |
| 04_3 | Ten to the sixteen. Doubles there are two apart, so the one is lost. | The ? goes, and as the answer is said the question becomes "N + 1 → N" over "(N + 1) − N = 0 in doubles", both computed in doubles; they stand through the whole line. On "Doubles there are two apart" the doubles near N are marked in teal (N − 2, N, N + 2, computed), "2 apart" between two of them, and a "precisely" lower-third: a double has 53 significant bits, so above 2 to the 53, doubles are at least 2 apart; N + 1 is halfway between the doubles N and N + 2, and the tie goes to N. On "so the one is lost" a red dot at N + 1 moves to N | dots from the computed doubles, moves, lower-third | |
| 04_4 | So a right answer can get caught. This C is A times B, but with both coins heads, two lost ones make the columns differ. | The line clears. A, B and C, two by two, with N in their entries, and the mistake C − AB beside them, all zeros. The four tosses of two coins. On "This C is A times B" the exact row: "match" under every toss. On "with both coins heads" the doubles row, "match, match, match, no match" (the last in red), the toss 11 ringed in gold, and under it, both computed: exact B r = (N + 1, −N + 1) and A(B r) = (2, 0) = C r; in doubles B r = (N, −N) and A(B r) = (0, 0) ≠ (2, 0) = C r | grids from the model, verdicts computed both ways, TeX | |
| 04_5 | And a wrong answer can pass every toss. This C is too small by one, yet in doubles all four tosses miss it. | The second example replaces the first: C − AB has −1 in its corner, outlined in red. On "This C is too small by one" the exact row: match, match, then "no match" in green twice, caught. On "all four tosses miss it" the doubles row, "match" under every toss in red, the toss 10 ringed, and exact A(B r) = (N + 1, 0) ≠ (N, 0) = C r against doubles A(B r) = (N, 0) = C r | grids from the model, verdicts computed both ways, TeX | |
| 04_6 | A tolerance would let the right one through, but then a wrong C inside it passes every toss, with no half guarantee. | The computations go. On "but then a wrong C inside it" the exact and doubles rows above, the test of equality, dim, and under them a check with a tolerance, \|A(B r) − C r\| ≤ 2 in every row, the tolerance computed as the right C's largest gap in doubles (toss 11: (0, 0) against (2, 0)), and "the right C passes 4 of 4 in doubles; this wrong C, 4 of 4 in exact sums", computed; a "precisely" lower-third: tolerance 2, the right C's largest gap in doubles, lets it through, but then this wrong C passes all 4 tosses in exact sums: no half bound | TeX, text from facts, lower-third | |
| 04_7 | So the promise is for exact arithmetic. A right C matches every toss, and a wrong one gets past each fresh toss at most half the time. | Everything clears but the card. The opening picture returns, A × B = C with the slip outlined in red. On "A right C matches every toss" the first line, "a right C: every toss matches"; on "a wrong one gets past" "a wrong C: each fresh toss misses at most half the time", and k fresh tosses all miss ≤ 1/2^k, with "in doubles, both can fail" dim under them. The card gives way to the exact guarantee: in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C with chance at most 1/2 to the k. It stands until it has had time to be read, then the video fades | TeX from the bound, lower-third | |

### Notes for 04

- Prediction (04_2): the question is on screen from "What is ten to the
  sixteen", before its last word; four seconds of silence with the
  countdown; no double is marked until 04_3 starts. The answer is spoken
  first ("Ten to the sixteen") and written with it: the question gives way
  to the two answer lines at the start of 04_3, so they stand through the
  whole line (about six seconds) while the doubles and the moving dot show
  why, instead of arriving at its end.
- At 04_6 the equality rows (exact and doubles) dim when the tolerant test
  comes in under them, so their "no match" cannot be read against its
  "4 of 4": they answer a different question.
- Everything about doubles is computed, not drawn. The scene and the facts
  run the same left-to-right sums as `model::rounded_passes`
  (`model::rounded_times`), and the oracle repeats them in Python with an
  explicit loop. The doubles near N, their spacing (`f64::next_up`), N + 1
  in doubles, both examples' columns and their verdicts are all F-X4 facts.
  The scene refuses to draw if the right C does not fail exactly one toss,
  if the wrong C does not pass all four, or if the lost ones are not two.
- The cost is a count of multiplications (`model::check_multiplications`,
  `product_multiplications`), never a time. The programmers' card says that
  the column sums still take additions and memory reads.
- The tolerance is shown, not argued, and computed, not chosen. It is the
  right C's largest gap in doubles, 2 on toss 11 (`facts::right_tolerance`,
  F-X4.tolerance.right_residuals 0, 0, 0, 2), the smallest that lets the
  right one through as the voice says (a tolerance of 1 would still catch
  it). With it the wrong C passes all four tosses even in exact sums
  (F-X4.tolerance.wrong_residuals 0, 0, 1, 1). The voice says only that the
  half guarantee does not come with it.
- The voice never says "floating point", "rounding", "decimal",
  "precision" or "approximate" (the narration test in `main.rs`): it says
  "doubles" and "the one is lost". The screen and the cards show the
  rounding.
- The final card is the exact guarantee in chapter 03's words. The panel
  over it adds that a right C matches every toss and that in doubles both
  can fail, as the two examples showed.
- Narration units: the same unqualified, unfrozen proposal; 04_2's question
  ends its unit and 04_3's answer opens the next, in the same unit as 04_4.
  Six units of 21 to 39 words.
- Pause targets, for listening, not rules: before "But many programs",
  "So a right answer", "And a wrong answer" and "So the promise", a turn
  of about 0.85 s.
- Pronunciation to probe: "doubles", "ten to the sixteen", "fiftieth", and
  the letters A, B and C as names.
