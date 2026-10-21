# Storyboard — Check a Giant Matrix Product Without Doing the Product Again

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py freivalds` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact computed in
`src/facts.rs` and recomputed another way by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it. Nothing that answers a question
is on screen before its answer beat starts.

## The arc

The complete episode (revision of 2026-10-20, eleven chapters; the editorial
candidate root accepted for source implementation is preserved with its
audit in the narration lab). One tiny example runs through the whole video:
A = (1 2 1 / 0 3 1 / 3 2 0), B = (2 1 0 / 1 0 2 / 2 1 3), and the claim C,
their product with one slip (row 2, column 2 says 2; the product has 1). The
true product's nine entries are the digits 1 to 9, each once.

- Act I (01 to 04): someone hands over C; redoing it costs as much as doing
  it; two cheap checks fail honestly (spot checks at scale, and a fixed sum of
  every column fooled by a cancelling pair); coins choose the columns, and one
  toss catches the slip for certain; all eight tosses, half miss, and why; the
  half needs fair coins.
- Act II (05, 06): mistakes of other shapes, a finite check that is not a
  proof, and the pairing proof on one concrete pair, with a nonzero entry of
  the mistake D = C − AB, and its failure over a column where D is all zero.
- Act III (07, 08): fresh tosses multiply and reuse does not; what the chance
  means and what it does not; the name; a toss seen first can be beaten.
- Act IV (09 to 11): the cost counted honestly (per check, twenty checks, the
  break-even at three by three, the loop a programmer writes); doubles break
  both promises; the exact promise on the opening picture, how many checks a
  billion needs, and why the check works.

| Chapter | Spine | Status |
|---|---|---|
| `01_handed_an_answer` | cold open with the payoff, the title; the tiny example; row times column said aloud; a billion multiplications at a thousand rows (a prediction); why anyone checks | source written 2026-10-20 (`handed_an_answer.rs`), uncompiled, no native check, no render, no narration |
| `02_cheap_checks` | one random entry misses a single slip eight times in nine; at a million entries, half a million picks for a coin flip's chance (a prediction); the sum of every column costs nine multiplications and catches this slip; the slip named | source written 2026-10-20 (`cheap_checks.rs`), uncompiled, no native check, no render, no narration |
| `03_coins_choose` | the sum of every column passes a C with a cancelling pair; with three columns every fixed choice can be fooled, so coins choose; heads, heads, tails (a prediction: do the columns match?); a mismatch proves C wrong and names a wrong row | source written 2026-10-20 (`check_with_coins.rs`, from the accepted chapter 01 scenes), uncompiled, no native check |
| `04_eight_tosses` | all eight tosses (a prediction), half miss, the misses leave out the slip's column; the mistake on its own; a count becomes a chance with fair coins; coins heads one time in ten (a prediction) | source written 2026-10-20 (`check_with_coins.rs` and the accepted chapter 02 opening), uncompiled, no native check |
| `05_shapes_of_mistakes` | two of one sign, a cancelling pair (the sum of every column again), one, one and minus one (a prediction), one per row and column; two wrong entries can hide as often as one; 19,682 nonzero small mistakes checked, not a proof | source written 2026-10-20 (`never_more_than_half.rs`, from the accepted chapter 02 scenes), uncompiled, no native check |
| `06_never_more_than_half` | pick a nonzero entry of the mistake; pair the tosses by its column's coin; one pair walked through (a prediction); every pair, at most one miss; the pairing fails over a column where the mistake is all zero; any size | source written 2026-10-20 (`never_more_than_half.rs`), uncompiled, no native check |
| `07_toss_again` | two fresh tosses (a prediction), a quarter; reuse gains nothing; twenty fresh tosses; what the chance means; one-sided; the name | source written 2026-10-20 (`toss_again.rs`, from the accepted chapter 03 scenes and the chapter 01 ladder), uncompiled, no native check |
| `08_seen_first` | a toss seen first can be beaten (a prediction: where to hide a wrong entry from heads, heads, tails); all heads is the sum of every column; fresh tosses still catch each of those wrong Cs half the time; what a programmer must do | source written 2026-10-20 (`toss_again.rs`), uncompiled, no native check |
| `09_what_it_costs` | one check a thousandth of the product, twenty a fiftieth, counted in multiplications; at three by three three checks cost as much (a prediction); the loop | source written 2026-10-20 (`exact_and_rounded.rs`), uncompiled, no native check |
| `10_exact_and_rounded` | the proof needs exact sums; ten to the sixteen plus one in doubles (a prediction); a right C caught, a wrong C passing every toss; a tolerance has no half guarantee | source written 2026-10-20 (`exact_and_rounded.rs`, the accepted chapter 04 scenes), uncompiled, no native check |
| `11_the_promise` | the exact promise and its assumptions on the opening picture; back to the start; thirty fresh checks for one in a billion (a prediction); why a mistake cannot hide; names to search and receipts | source written 2026-10-20 (`exact_and_rounded.rs`), uncompiled, no native check |

Every chapter's narration, render, native timing, listening and audiovisual
acceptance is open. The earlier four-chapter cut (288.5 s) and its accepted
source are preserved; nothing here claims they still describe the video.

## Vocabulary (one word per idea)

**matrix** (a grid of numbers: A, B and the claim C; "grid" only on first
sight); **entry**; **product** (A times B, row times column; "row by column"
names the counting convention, "the schoolbook method" only in an exact
version); **answer** or **claim** (C, what someone handed over); **spot
check** (recompute one entry); **the sum of every column** (all three columns
of a matrix added into one column, the all-heads toss); **coin** (one per
column); **heads** keeps a column, **tails** leaves it out; **toss** (one
landing of every coin: on screen the coins, and `r` in the formulas, never
"vector" in the voice); **heads columns** (the columns a toss keeps);
**check** (add B's heads columns, multiply by A, compare with C's heads
columns added); **match**; **miss** (a toss that lets a wrong C through;
never "false positive" in the voice); **caught** (the columns disagree, so C
is wrong for certain); **slip** (the claim's one wrong entry, row 2, column 2,
named in 02_10); **the mistake** (C minus the true product, entry by entry;
"C − AB" on screen, always red and drawn apart from C); **a nonzero entry of
the mistake** (what the proof picks; never "a wrong entry is not zero", since
C itself may wrongly hold a zero); **hide** (the mistake's side of a miss);
**pair** (two tosses that differ only in one coin); **way** (one landing of
the coins; "the sixty-four ways two tosses can land"); **fresh** (tossed
independently, after C is fixed); **reuse**; **double** (the 64-bit numbers
many programs compute with; the voice never says "floating point",
"rounding", "decimal", "precision" or "approximate"); **tolerance**.

## Color roles

| Idea | Color |
|---|---|
| matrix A's name, and A·(B·r) | blue `#58C4DD` |
| matrix B's name, and B·r | violet `#C792EA` |
| the claim C's name, and C·r | gold `#F0AC5F` |
| a heads coin, and the columns it keeps | teal `#5CD0B3` |
| a tails coin | `faint`, its face `dim` |
| a toss that catches the mistake, and the verdict | green `#83C167` |
| the mistake: C − AB, its nonzero entries, a toss that misses it, the ladder's chance bars | red `#FC6255` |
| the think ring and "?"; a picked column, a pair's link and its change; the diagonal of one toss reused; the row where all heads cancels | `hero` `#FFD866` |
| entries, brackets, signs | `ink`, `dim` |

Outlines enclose their cells and stay off the brackets: a row, column or
entry box spans exactly its cells, and the brackets stand a little farther
out (`draw::BRACKET_GAP`), so no box crosses a bracket's stroke or serif
(review F5). Every N near 10^16 is set upright, in the text face of TeX,
the same N the cards print (review F8).

## Sources and scope

The statements the video makes, where they were checked, and what was not
read or watched, are in the crate's [README](../README.md). The video claims
only the exact theorem; the floating-point boundary is chapter 10's, built
on the F-X4 facts. Counts of work are multiplications in the row-by-column
convention, never times, and no claim is made about every known method of
multiplying.

## 01_handed_an_answer: Someone hands you C (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Someone hands you the product of two giant matrices. Can you check it without redoing all that work? | Cold open: two large faint grids standing for a thousand by a thousand, A in blue and B in violet outline, a dim × between them, "1000 × 1000" under them; on "hands you" a dim = and a gold grid, C. On "Can you check it" a gold ? over the = | grids drawn as texture, TeX labels | |
| 01_2 | With coin tosses, you can. For a thousand rows and columns, twenty checks need a fiftieth of the multiplications that redoing it takes. Right answers always pass, and a wrong one gets through less than once in a million. | A row of coins lands over C. On "twenty checks" two bars to scale under the grids: "redo the product, row by column: 1,000,000,000" full, "20 checks: 20,000,000" a fiftieth of it. On "Right answers" two lines, "a right C: always passes" and "a wrong C passes all 20 with chance ≤ 1/1,048,576"; a "precisely" card (K01). Then everything clears and the title card stands | bars to scale from facts, lower-third, title card | hold |
| 01_3 | Start small. Here are two matrices, A and B, and the answer someone handed you, C. Is C really A times B? | The opening picture: A and B as grids of numbers between brackets, names in blue and violet under them, a dim ×; on "the answer someone handed you" a dim = and C, its name in gold. On "Is C really" a gold ? over the = | grids built from the model | |
| 01_4 | Each entry of the product is a row of A times a column of B. Multiply them in pairs, then add. Row one times column three gives zero, plus four, plus three. Seven. | On "a row of A" row 1 of A, column 3 of B and entry (1, 3) of C are outlined. On "Multiply them in pairs" the panel shows 1·0 + 2·2 + 1·3, built from the drawn entries; on "gives zero" the line = 0 + 4 + 3 = 7 | highlight boxes, TeX from the model | |
| 01_5 | C has seven there too, so that entry is right. But the product has nine entries, three multiplications each, twenty-seven in all. | C's 7 is outlined in green. On "But the product" the panel adds 9 × 3 = 27, "multiplications to redo the product" | TeX from facts | |
| 01_6 | Now make it a thousand rows and a thousand columns. How many multiplications does that take, row by column? ⟨predict⟩ | The outlines and the 3 × 3 sums clear; the panel shows "n = 1000". On "How many" the question on screen, "How many multiplications, row by column, for a 1000 × 1000 product?"; four seconds of silence with the gold countdown ring and "pause for more time"; no count on screen | think countdown | |
| 01_7 | One billion. That's a million entries, a thousand multiplications each. And checking it by redoing it costs another billion. | The ? and the question go; the answer 1,000,000 entries × 1,000 = 1,000,000,000 (TeX from facts) and a "precisely" card (K02): the baseline is row-by-column multiplication, n cubed multiplications, and the video compares the check with that count only. On "another billion" the line "redo it to check: another 1,000,000,000" | TeX from facts, lower-third | |
| 01_8 | It might come from a fast library, a rented machine or a student. You'd like to trust it without paying twice. | The panel's answer stays; nothing new is drawn for the three sources. The card stands its reading time from its completed fade-in, then the chapter fades | lower-third read | |

### Notes for 01

- Prediction (01_6): the question is on screen from "How many", before the
  question's last word; four seconds of silence with the countdown; no count
  until 01_7 starts, and the count is written as it is said.
- Exact versions: K01 (01_2) and K02 (01_7). K02 replaces the earlier
  SCHOOLBOOK card, which claimed that no known method needs only about n
  squared steps; the video makes no claim about every known method. The
  cold open's "a fiftieth" names its example (a thousand rows and columns)
  and its unit (multiplications, row by column).
- The bars are drawn to scale from `model::product_multiplications` and
  `check_multiplications`; the scene refuses to draw if twenty checks are not
  a fiftieth or 2^−20 does not beat a million.
- Pause targets, for listening, not rules: after "Can you check it without
  redoing all that work?" about one second before its answer; before "It
  might come", a turn of about 0.85 s.
- Pronunciation to probe: the letter names A, B and C as matrix names (A must
  not be read as the article).

## 02_cheap_checks: Two cheap checks (~83 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | The obvious shortcut is to check a few entries. Pick one at random, and redo just that one. Three multiplications. | The opening picture returns (A, B and C, names, × and =). On "Pick one at random" entry (1, 3) of C is outlined in gold, with row 1 of A and column 3 of B; the panel: "one entry: 3 multiplications" | highlight boxes | |
| 02_2 | Suppose C has a single wrong entry. One random pick finds it one time in nine. Eight times in nine, it misses. | The panel adds "an entry picked at random", then in red "misses a single wrong one" and "8 times in 9". Where the wrong entry is, is not shown | panel from facts | |
| 02_3 | At a thousand rows there are a million entries, and one pick misses a single wrong one almost every time. | The 3 × 3 picture clears. A large square of faint lines, "1,000,000 entries", holds one small red cell; a gold dot, the pick, lands elsewhere, and "one pick misses it 999,999 times in 1,000,000" | texture grid, moves | |
| 02_4 | How many different entries would you have to check to catch it at least half the time? ⟨predict⟩ | The question on screen, "How many different entries for at least half the time?"; four seconds of silence with the countdown; no shading | think countdown | |
| 02_5 | Half a million. At a thousand multiplications each, that's five hundred million, half the cost of redoing the product. | The left half of the square shades gold; "500,000 entries × 1,000 = 500,000,000", then "half of 1,000,000,000"; a "precisely" card (K03): k different entries picked at random catch one wrong entry with chance k / n squared | TeX from facts, lower-third | |
| 02_6 | Spot checks only see the entries they pick, and one wrong entry can be anywhere. | The red cell, outside the shaded half, flashes | flash | |
| 02_7 | Here is a cheaper idea. Add up all three columns of C into one column. That takes only additions. | The square clears and the opening picture returns. C's three columns are outlined in teal; copies of their entries fly down side by side with plus signs and become one column, 15, 16, 15, named "C's columns added" in gold | copies fly, column addition | |
| 02_8 | If C is right, each of its columns is A times the same column of B. So the sum of C's columns must be A times the sum of B's. | The panel: "if C = AB:", "each column of C = A × that column of B", "C's column sum = A × B's column sum". B's three columns add into 3, 3, 6 in violet, quicker; each row of A flashes with it while 15, 15, 15 fills in under A, named "A × B's column sum" in blue | column addition, lagged reveal | |
| 02_9 | That takes nine multiplications, not twenty-seven. And in row two, C's sum says sixteen, while the other side says fifteen. So C is wrong. | The panel: "this check: 9 multiplications", "redoing it: 27". A's column slides beside C's; =, ≠, = appear between the rows, and 16 and 15 are outlined in red; on "So C is wrong" a green verdict box | signs from the model, verdict box | |
| 02_10 | In fact, the two in the middle of that row should be one. Call it the slip. | C's entry (2, 2) gets its own red box, "2 should be 1" beside its row (built by `correction()` from the model), and "the slip" under it. It stands, then the chapter fades | highlight box, label from the model | |

### Notes for 02

- Prediction (02_4): the question is on screen from "How many different
  entries" before its last word; four seconds of silence; no shading until
  02_5, which says the count as the square shades.
- The scale-up is computed (`facts::spot_half`): with a million entries, k
  different entries picked at random catch one wrong entry with chance
  k / 1,000,000, and k = 500,000 is the least that reaches one half; at a
  thousand multiplications each, that is half the product's billion. K03 is
  the exact version.
- The sum of every column is the all-heads toss, before any coin is named:
  C's sums 15, 16, 15, B's 3, 3, 6 and A times that 15, 15, 15
  (`facts::all_columns_check`); it catches this slip in row 2.
- The label "2 should be 1" names the boxed entry's own value (review F2).
- Pause targets: before "Here is a cheaper idea", a turn of about 0.85 s.

## 03_coins_choose: Coins choose the columns (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | But suppose the nine beside the slip were written as eight. Now row two adds up to fifteen, the same as the other side. | The opening picture with the slip boxed in red. On "were written as eight" C's 9 beside the slip turns into an 8 (gold flash) and gets a red box too. On "Now row two" C's column sum, 15, 15, 15, stands beside A × B's column sum, 15, 15, 15, with = in every row | grid edit, column sums from the model | |
| 03_2 | The extra one and the missing one cancel, and this check passes a wrong answer. | "+1" under the slip and "−1" under the 8, and "+1 + (−1) = 0"; a red verdict box, "passes a wrong C" | labels from the model, verdict box | |
| 03_3 | With three columns, every fixed choice of columns has wrong answers it can't see. So don't fix it in advance. Let coins choose the columns, after C is handed over. | A "precisely" card (K04): from two by two up, every fixed choice of columns misses some wrong C; with a single column, heads checks the product exactly. On "Let coins choose" three blank coins, each marked ?, appear over C's columns | lower-third, coins | |
| 03_4 | Back to the C with one slip. Toss a coin for each column. Heads keeps a column, and tails leaves it out. | The 8 turns back into 9 and its box, the sums and the cancelling labels go. On "Toss a coin" the blank coins land: heads, heads, tails; "heads keeps a column" | coins, highlight | |
| 03_5 | Say they land heads, heads, tails. Add up C's heads columns, then B's, and multiply B's by A, still nine multiplications. Will the two columns match? ⟨predict⟩ | C's heads columns are outlined in teal, its tails column dims, and copies fly down and add: C r = (8, 7, 11) in gold. The same toss over B, quicker: B r = (3, 1, 3) in violet; a "for programmers" card: B r and C r only add columns, A(B r) takes n squared multiplications. Each row of A flashes with B r while A(B r) = (8, 6, 11) fills in under A, in blue; "still 9 multiplications". On "Will the two columns" the question on screen; four seconds of silence; no sign, outline or verdict between the columns | copies fly, column addition, lagged reveal, think countdown | |
| 03_6 | No. If C were right, they would, for the same reason as before. But in row two, C says seven and the other side says six. | A(B r) slides to stand beside C r; the reason, A(B r) = (AB) r, "so a right C gives two equal columns". On "But in row two" =, ≠, = appear between the rows, and 6 and 7 are outlined in red | TeX identity, signs from the model | |
| 03_7 | So C is wrong, for certain, and row two holds a wrong entry. Show anyone this toss, and nine multiplications let them see it too. | A green verdict box, "C is wrong", and "row 2 holds a wrong entry". On "Show anyone this toss" a small card: "toss 110: row 2 gives 6 ≠ 7", "anyone can recheck: 9 multiplications". It stands, then the chapter fades | verdict box, text from facts | |

### Notes for 03

- Prediction (03_5): the question is on screen from "Will the two columns",
  before its last word; no =, ≠, outline or verdict until 03_6, which answers
  "No" as the signs come in.
- The cancelling C is the slip plus the 9 beside it written as 8
  (`facts::cancelling_claim`, F-F10.cancelling rows): it differs from C only
  at row 2, column 3, its column sums are 15, 15, 15, and it passes the sum of
  every column. Against all eight tosses it hides from four (000, 011, 100,
  111; F-F10.cancelling.passing) and is caught by the other four.
- K04 keeps 03_3's claim scoped: the voice says "With three columns"; the
  card gives the edges (F-X3.n2.fooled 4 of 4, F-X3.n1.heads_fooled false).
- A caught toss names a wrong row: every disagreeing row of A(B r) and C r is
  a nonzero row of the mistake (F-X6, over all 19,682 small mistakes and
  every toss). The converse is not claimed.
- Pause targets: before "But suppose" and "Back to the C", about 0.85 s.

## 04_eight_tosses: All eight tosses (~74 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | But that was one toss. Of the eight ways three coins can land, how many miss the slip? ⟨predict⟩ | A, B and C at the top, the slip boxed. On "Of the eight ways" all eight tosses appear under them in two rows of four, in counting order (000 to 111), no verdicts; the question, "How many of these 8 tosses miss the slip?"; four seconds of silence with the countdown | lagged reveal, think countdown | |
| 04_2 | Check all eight. Four of them miss, exactly half. | The ? goes; each toss gets its verdict in turn, "miss" in red or "caught" in green. On "Four of them" the question becomes "4 of 8 miss: exactly half" | lagged reveal from the model's checks | |
| 04_3 | Every miss leaves out the middle column, the one that holds the slip. When that coin lands heads, the slip's extra one lands in row two, and nothing there cancels it. | C's column 2 is outlined in gold, the slip alone in red, "2 should be 1" beside its row. On "leaves out" every toss's middle coin is ringed: red around the tails coin of each miss, green around the heads coin of each catch. On "When that coin lands heads" the green rings flash with the slip | highlight, rings, flash | |
| 04_4 | So look at the mistake on its own, C minus the true product. For the slip, it's a single one. | Everything clears. The mistake grid, its zeros dim and its 1 outlined in red, "C − AB" in red under it, alone on the left | grid from the model | |
| 04_5 | The two columns the check compares always differ by the mistake's heads columns, added up. So the check misses exactly when those add up to zero in every row. | Heads, heads, tails lands over the mistake's columns, its heads columns outlined in teal, and their sum (0, 1, 0) appears beside it, with C r − A(B r) = (C − AB) r above and "not all zero: caught" in green | grid, highlight, TeX identity | |
| 04_6 | Four of eight becomes a chance of one half when every toss is equally likely. Fair coins, each tossed on its own, give exactly that. | The eight tosses return in two rows of four, each with its chance under it, 1/8, red under the four misses; "misses: 4 × 1/8 = 1/2". A "precisely" card (K06): this half guarantee assumes fair coins, independent of each other, drawn after C is fixed | TeX fractions from facts, lower-third | |
| 04_7 | Suppose each coin landed heads only one time in ten. How often would the slip get past? ⟨predict⟩ | The individual chances and their old fair-coin total go blank; "heads: 1 time in 10" over the tosses; the question, "Heads 1 time in 10: how often does the slip get past?"; four seconds of silence | think countdown | |
| 04_8 | Nine times in ten, whenever the middle coin lands tails. Our half guarantee uses fair coins. | Each toss's chance appears, computed: 729/1000, 81/1000 and so on, red under the misses; "misses: (729 + 81 + 81 + 9) / 1000 = 9/10". On "Our half guarantee" the line "fair coins: 1/2" | TeX fractions from facts | |

### Notes for 04

- Predictions (04_1, 04_7): each question on screen before its last word;
  four seconds of silence; no verdict, ring or chance until the answer beat.
  At 04_7 the fair-coin total leaves with the individual chances (root
  review, 2026-10-20); the biased total comes in only with 04_8's answer.
- The misses are the tosses with tails over the slip's column
  (F-F4.misses_are_tails_on_wrong_column); the scene refuses to mark them
  otherwise.
- The mistake is drawn alone, red and named "C − AB", apart from C (D stays
  distinct from C).
- Chances are exact fractions (`facts::toss_weights`, F-F12): with heads one
  time in ten, the slip passes 9/10; with only the slip's column's coin fair,
  still 1/2 (F-F12.only_slip_coin_fair), which is why 04_6 says "when" and
  04_8 says the guarantee "uses" fair coins, not that fairness is needed for
  every construction.
- Pause targets: before "So look at the mistake" and "Four of eight
  becomes", about 0.85 s.

## 05_shapes_of_mistakes: Mistakes of other shapes (~78 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | Is half special to this slip? Try mistakes of other shapes, with the same A and B. | The slip's mistake grid, "C − AB" in red under it; the tally, top right, starts with "the slip (1): 4 of 8" | grid from the model, tally | |
| 05_2 | Two in one row, both one too big, hide only when both coins land tails. Two of eight. | The grid becomes 0, 1, 1 in row 2; all eight tosses come in, two rows of four, and under each its row-2 sum, red where it is zero (000 and 100). On "Two of eight" the tally adds "1, 1 in one row: 2 of 8" | grid swap, lagged reveal, sums from the model | |
| 05_3 | But one too big and one too small by the same amount hide whenever their coins agree. Four of eight, half again. | The grid becomes 1, −1 in row 2 and the sums change with it; the zeros (000, 011, 100, 111) turn red. On "Four of eight" the tally adds "1, −1 in one row: 4 of 8" | grid swap, sums from the model | |
| 05_4 | That's the pair that fooled the sum of every column, which is just the toss with all heads. | Toss 111's zero is ringed in gold, and "all heads = the sum of every column" stands under the tosses | ring, text | |
| 05_5 | Now one, one and minus one in a row. How many tosses does it hide from? ⟨predict⟩ | The ring and its words go; the grid becomes 1, 1, −1 in row 2; the sums go. On "How many tosses" the question, "How many of these 8 tosses does it hide from?"; four seconds of silence with the countdown; no sum on screen | think countdown | |
| 05_6 | Three. Not half, and not a power of two. | The ? and the question go; the sums come in (0, −1, 1, 0 over 1, 0, 2, 1), three of them red; the tally adds "1, 1, −1 in one row: 3 of 8" | lagged reveal from the model | |
| 05_7 | Put one wrong entry in each row and column, and it hides only from all tails. One of eight. | The grid becomes one 1 in each row and each column; under each toss its verdict, "miss" only under 000; the tally adds "1 per row and column: 1 of 8" | grid swap, verdicts from the model | |
| 05_8 | So two wrong entries can hide as often as one. What matters is whether they can cancel. | The tally's two "4 of 8" lines flash together, then its "2 of 8" and "1 of 8" lines | flash | |
| 05_9 | Try every nonzero three by three mistake with entries of minus one, zero or one. All nineteen thousand, six hundred eighty-two of them hide from at most half the tosses. | A "checked" card: all 19,682 nonzero three by three mistakes with entries −1, 0 or 1 hide from at most half the tosses; a finite check, the pairing proves every size. Under the tosses "19,682 checked: most misses 4 of 8" | lower-third, text from facts | |
| 05_10 | But trying cases is not a proof. Mistakes can hold any whole numbers, at any size. We need a reason. | The count dims; a gold ? and "any size, any whole numbers" beside it. It stands until the card has been read, then the chapter fades | text | |

### Notes for 05

- Prediction (05_5): the question is on screen from "How many tosses"; four
  seconds of silence; no sum, ring or tally line for the new mistake until
  05_6, which says "Three" first.
- The count 19,682 is of nonzero mistakes (the zero matrix is no mistake):
  3^9 − 1 (F-X1.n3.wrong); the card says "nonzero", as the voice does.
- Edge tests behind the always-sentences (F-X5): 18 sign and size cases, 6
  cancelling cases, 384 one-per-row-and-column patterns.
- "Two wrong entries can hide as often as one": the cancelling pair and the
  slip each hide from 4 of 8 (F-X2.cancelling.misses, F-X2.one_entry.misses).
- Pause targets: before "But one too big", "Try every" and "But trying
  cases", about 0.85 s.

## 06_never_more_than_half: Never more than half (~81 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | Why never more than half? Pick any entry where the mistake isn't zero. In the one, one, minus one mistake, take row two, column one. | The 1, 1, −1 mistake, red "C − AB" under it, its nonzero entries outlined in red; the eight tosses below, with no sums. On "take row two, column one" that entry's box flashes and column 1 is outlined in gold | grid from the model, highlight | |
| 06_2 | Pair up the tosses that differ only in that column's coin. With three coins, that makes four pairs. | The first coin of every toss is ringed in gold; a gold link joins each top toss to the one under it, its partner; "4 pairs" | rings, links | |
| 06_3 | Fix the other two coins, say tails, then heads. With the first coin tails, row two's heads columns add up to minus one. | Every pair but 001 and 101 dims. Coins 2 and 3 of both are ringed in gold, "fixed: tails, heads". On "With the first coin tails" the top toss's row-2 sum, −1, appears under it | dim, rings, label from the model | |
| 06_4 | Flip the first coin to heads. How much does row two's sum change? ⟨predict⟩ | The question, "Coin 1 flips to heads: how much does row 2's sum change?"; four seconds of silence with the countdown; the bottom toss's sum is not shown | think countdown | |
| 06_5 | By the mistake's entry there, one, exactly. Minus one becomes zero. Flipping that coin adds just that one entry of the mistake to row two's sum. | The ? and the question go; "+1" beside the link; the bottom toss's sum appears, 0, in red; the chosen entry's box flashes with it | labels from the model, flash | |
| 06_6 | In every pair, row two's sums differ by the chosen entry of the mistake. That entry isn't zero, so both sums can't be zero. At most one toss of each pair misses. | All pairs light again; every sum comes in, and "+1" beside every link. The entry boxes in row 2 give way to one red row outline, the gold column outline distinct. On "At most one" the three zero sums are ringed in red and flash, one in each of three pairs and none in the fourth | lagged reveal, highlight, labels from the model | |
| 06_7 | The column matters. Pair the slip's tosses by the first coin, a column where its mistake is all zeros, and its row sum never moves. Two of those pairs miss twice. | The slip's mistake replaces it (one red 1 at row 2, column 2); its column 1 is outlined dim, "all zeros"; beside every link "+0"; the sums 0, 0, 1, 1 over 0, 0, 1, 1 come in, and pairs 000 and 100, 001 and 101 have both their sums ringed in red | grid swap, labels and rings from the model | |
| 06_8 | Pairs cover every toss exactly once. So at most half of all tosses miss, for matrices of any size. | Clear the zero-column counterexample, its +0 labels and double-miss rings. Restore the 1, 1, −1 mistake, gold nonzero column, red row, its eight sums, +1 labels and three single-miss rings before the summary. A "precisely" card (K08): pick an entry where the mistake is not 0; flipping its column's coin moves its row's sum by that entry, so a pair holds at most 1 miss. At the right, one after another: "8 tosses, 4 pairs", "at most 1 miss in each pair", "at most 4 of 8"; on "for matrices of any size" 2ⁿ tosses, 2ⁿ⁻¹ pairs. It stands until the card has been read, then the chapter fades | lagged reveal, TeX, lower-third | |

### Notes for 06

- Prediction (06_4): the question is on screen from "How much", before its
  last word; the bottom toss's sum, the change and every other sum stay off
  until 06_5 and 06_6.
- The proof picks a nonzero entry of the mistake (review correction of
  2026-10-20): "a wrong entry" is not used for this step, since a wrong entry
  of C could itself be 0. The entry is `facts::proof_entry()`, row 2, column 1
  of the 1, 1, −1 mistake; its pairs and sums are the F-F8 rows, the
  walk-through pair the F-F14 rows.
- The count's "at most 1 miss in each pair" is the proof's constant
  (`facts::MISSES_PER_PAIR`); the scene refuses a pair above it.
- The column matters (06_7): paired by coin 1, over a column where the slip's
  mistake is all zero, the row sum never moves (+0) and two pairs miss twice
  (F-X5.right_column, 2, 2, 0, 0). 06_8 clears it and brings the proof's
  picture back (misses per pair 1, 1, 0, 1) before the bound is stated
  (root review, 2026-10-20).
- Pause targets: after "Why never more than half?" about one second;
  before "The column matters", about 0.85 s.

## 07_toss_again: Toss again (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | One toss misses the slip half the time. So toss again, with fresh coins. | The slip's mistake, C − AB, top left. The eight tosses written as their coins, 000 to 111, in a column, red if it misses the slip and green if it catches it; "one toss: 4 of 8 miss" with the key "toss labels: miss caught". On "So toss again" the column sorts, misses first, and a copy turns to run along the top: "first toss", "second toss" | labels from the model, moves | |
| 07_2 | Two tosses can land sixty-four ways. How many of them miss both times? ⟨predict⟩ | The grid of the 64 ways fills in, a faint square for each first and second toss. On "How many of them" the question at the right; four seconds of silence with the countdown; no square is red | grid of squares, think countdown | |
| 07_3 | Sixteen, exactly a quarter. Both must miss, four ways for the first toss times four for the second. | The 16 squares where both miss turn red, filling the top-left corner; "16 of 64 ways miss both times". On "four ways for the first toss" the first toss's four miss rows are outlined; on "times four for the second" the four miss columns; "4 × 4 = 16" | lagged reveal from the model, highlight | |
| 07_4 | For this slip, that quarter is exact. Any wrong C gets past two fresh tosses at most a quarter of the time. | "this slip: 16/64 = 1/4"; on "Any wrong C" "any wrong C: ≤ 1/2 · 1/2 = 1/4", with extra clearance between the stacked fractions, and a "precisely" card: in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C with chance at most 1/2 to the k | TeX from facts, lower-third | |
| 07_5 | But reuse one toss for both checks, and nothing changes. Four of the eight still miss, half. | The diagonal is outlined in gold and the rest of the grid dims; on "Four of the eight" its four red squares flash; "same toss twice: 4 of 8, still 1/2" | highlight, dim | |
| 07_6 | Each fresh toss at least halves the chance again. Twenty fresh tosses all miss less than once in a million. | The grid and the panel clear. A ladder of the theorem's bound, a faint track for a chance of one and a red bar to scale: "1 toss" ≤ 1/2, "2 tosses" ≤ 1/4, and on "Twenty fresh tosses" "20 tosses" ≤ 1/1,048,576 < 1/1,000,000, no bar, "too small to see" | bars from the bound, TeX fractions | |
| 07_7 | That's a chance about your coins, for a C that's already fixed. It doesn't say how likely C was to be wrong before you checked. | The ladder dims. Two lines: "a chance over the coins, for this fixed C" in ink, and "not: the chance that C is wrong after passing", dim and struck through. A "precisely" card (K10) replaces the promise card once that has been read | text, lower-third | |
| 07_8 | And the check only errs one way. Once a toss catches C, it's wrong for certain, and a right C matches every toss. | Two lines under them: "C right: every toss matches", "C wrong: a caught toss proves it; a miss proves nothing" | text | |
| 07_9 | This check is known as Freivalds' algorithm. | "Freivalds' algorithm" large at the centre, and a "names to search" card: Freivalds' algorithm, randomized algorithm, one-sided error. It stands until read, then the chapter fades | names to search card | |

### Notes for 07

- Prediction (07_2): as accepted in the earlier chapter 03.
- The example and the theorem are kept apart (F-F5.two, F-F9.two.bound).
- 07_7 defuses the misreading of the ladder as a chance that C is right after
  passing: the bound is over the coins, for a fixed C. 07_8 is one-sided
  error, the formal term only on the card.
- Pronunciation obligation (07_9): "Freivalds" (Latvian; Rūsiņš Mārtiņš
  Freivalds) must be looked up, respelled, probed against a control and a
  decoy, and locked into `lexicon.json` with its sources before this line is
  synthesized; the owner's pronunciation overrides every source. Root
  performs the bounded route probe. No date or biography is spoken; the 1977
  paper has not been read.
- Pause targets: before "But reuse" and "That's a chance about your coins",
  about 0.85 s.

## 08_seen_first: A toss seen first (~64 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Fresh also means tossed after C is handed over. Someone who sees your toss first can write a wrong C it misses. | The eight tosses come in as coins, two rows of four. The exact card and its solution are withheld until after 08_2's prediction | coins | |
| 08_2 | Say they see heads, heads, tails. Where could they put a wrong entry this toss can't see? ⟨predict⟩ | Toss 110 is ringed in gold, and an empty mistake, all zeros dim, appears under it. The question, "Where can a wrong entry hide from heads, heads, tails?"; four seconds of silence; no entry is marked | ring, grid, think countdown | |
| 08_3 | Anywhere in the third column, under the tails coin. The toss never looks there. | After the prediction, with the answer: the empty mistake's third column fills red, "any nonzero entry here", the tails coin is ringed in red, and in the same fade the "precisely" card appears: from two by two up, any toss seen first can be beaten, one wrong entry under a tails coin, or under all heads a cancelling pair in one row. Its reading clock starts after that fade. Then the eight little grids fill in under the tosses, each the mistake of the wrong C written for that toss, zeros dim and wrong entries outlined in red, with "miss" under each, and "each grid shows C − AB" at the right; in the seven tosses with a tails coin, the tails coin over the wrong entry is ringed in red | lower-third, grids from the model, rings | |
| 08_4 | Against all heads, put one entry too big and one too small by the same amount in one row, and they cancel. That's the sum of every column again. | The first row of 111's grid is outlined in gold, and beside it "1 + (−1) = 0"; on "That's the sum of every column again" "all heads = the sum of every column" | highlight, text | |
| 08_5 | So every one of the eight tosses can be beaten. Yet against fresh tosses, each of these wrong answers is caught half the time. | The eight "miss" verdicts flash, "8 of 8 tosses beaten when seen first"; on "Yet against fresh tosses" a green line, "fresh tosses catch each of these wrong Cs: 4 of 8" | flash, text from facts | |
| 08_6 | The protection was never in any one toss. It's that whoever wrote C couldn't know which toss was coming. | Every toss's coins dim and a gold ? stands over each | dim, text | |
| 08_7 | For programmers, that means drawing the coins after C is fixed, from a source its maker can't predict, and new ones for every check. | A "for programmers" card (K12): the coins must not depend on C, drawn after C is fixed, from a source its maker cannot predict, fair, and new for every check. It stands until read, then the chapter fades | lower-third | |

### Notes for 08

- Prediction (08_2): the question is on screen from "Where could they";
  only toss 110's empty mistake is shown, and no grid holds a wrong entry,
  until 08_3. The "precisely" card names the answer, so it too waits for
  08_3 (root review, 2026-10-20).
- Every single nonzero entry in toss 110's third column hides from it, and
  none in its heads columns does (F-X7); the shown blind spots are
  `model::blind_spot`'s (F-X3).
- Pause targets: before "Against all heads" and "The protection", about
  0.85 s.

## 09_what_it_costs: What it costs (~59 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Now the cost. At a thousand rows, one check takes a million multiplications, a thousandth of the billion for the product. | "n = 1000" at the top. A faint track and a full bar, "redo the product, row by column: n³ = 1,000,000,000"; on "one check takes" a second track, "1 check: n² = 1,000,000", its bar to scale, a thousandth, and the ratio 1,000,000 / 1,000,000,000 = 1/1000; a "for programmers" card (K13): multiplications counted row by column, n cubed for the product and n squared per check; time also needs additions and memory traffic | bars to scale from facts, TeX, lower-third | |
| 09_2 | So twenty fresh checks take twenty million, a fiftieth. That counts work, not seconds. | A third track, "20 fresh checks: 20 × n² = 20,000,000", a blue bar to scale, a fiftieth; the ratio becomes 20,000,000 / 1,000,000,000 = 1/50 | bars to scale, TeX | |
| 09_3 | The work also includes additions, and moving numbers in and out of memory. | No new picture: the card is read with the line; it fades once its reading time has passed | lower-third read | |
| 09_4 | For three by three matrices, how many checks cost as much as redoing the product? ⟨predict⟩ | The thousand-row bars clear. "n = 3": a full bar, "redo: 27", and "1 check: 9", a third of it. The question; four seconds of silence; nothing stacked | bars from facts, think countdown | |
| 09_5 | Three. Nine multiplications each, twenty-seven in all. For matrices that small, just multiply. The bigger the matrices, the smaller each check's share. | Three bars of 9 stack end to end under the 27, "3 × 9 = 27". On "The bigger the matrices" k checks: k·n² / n³ = k / n, and 3/3 = 1, 20/1000 = 1/50 | bars, TeX from facts | |
| 09_6 | For programmers, the whole check is a short loop. Draw fresh coins. Add B's heads columns, multiply by A, and add C's heads columns. | The bars clear. A "for programmers" code panel slides in, six lines of code in the typewriter face, and the lines for the coins, B r, A(B r) and C r flash as they are said | code panel | |
| 09_7 | Compare row by row, and stop as soon as a toss catches C. Otherwise, repeat with new coins, twenty times in all. | The compare and stop lines flash, then the loop's first line. It stands, then the chapter fades | code panel, flash | |

### Notes for 09

- Prediction (09_4): the question is on screen before its last word; the
  stacked bars and "3 × 9 = 27" only in 09_5.
- K13 replaces the earlier COST_NOTE, which said every entry is read from
  memory; a selected-column check need not read the tails columns of B and
  C. The convention is stated: multiplications counted row by column.
- Break-even: k checks cost k·n² against n³, equal at k = n (F-F13). Three
  checks at three by three cost 27, the product's own count.
- The code is ASCII only (the typewriter face): no ·, × or ≠.
- Pause targets: before "For three by three" and "For programmers", about
  0.85 s.

## 10_exact_and_rounded: Exact and rounded (~58 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | Every sum so far has been exact, and the pairing proof needs that. | The pairing's step as a line: "two sums that differ by a nonzero entry are not both 0", and under it in gold, "with exact sums" | text | |
| 10_2 | But many programs compute with doubles, which skip some whole numbers. What is ten to the sixteen plus one in doubles? ⟨predict⟩ | The step clears. "N = 10¹⁶" at the top left and a number line with the whole numbers N − 2 to N + 2. On "What is ten to the sixteen" the question, "What is N + 1 in doubles?", typeset with the same upright N; four seconds of silence with the countdown; no double is marked | number line, think countdown | |
| 10_3 | Ten to the sixteen. Doubles there are two apart, so the one is lost. | The question becomes "N + 1 → N" over "(N + 1) − N = 0 in doubles", both computed, standing through the line. On "Doubles there are two apart" the doubles near N are marked in teal, "2 apart" between two of them, and a "precisely" card: a double has 53 significant bits, so above 2 to the 53, doubles are at least 2 apart; N + 1 is halfway between doubles N and N + 2, and the tie goes to N. On "so the one is lost" a red dot at N + 1 moves to N | dots from the computed doubles, moves, lower-third | |
| 10_4 | So a right answer can get caught. This C is A times B, but with both coins heads, two lost ones make the columns differ. | A, B and C two by two with N in their entries, and the mistake C − AB, all zeros. The four tosses of two coins. On "This C is A times B" the exact row, "match" under every toss; on "with both coins heads" the doubles row, the last "no match" in red, toss 11 ringed in gold, and under it both computed: exact B r = (N + 1, −N + 1) and A(B r) = (2, 0) = C r; in doubles B r = (N, −N) and A(B r) = (0, 0) ≠ (2, 0) = C r | grids from the model, verdicts computed both ways, TeX | |
| 10_5 | And a wrong answer can pass every toss. This C is too small by one, yet in doubles all four tosses miss it. | The second example replaces the first: C − AB has −1 in its corner, outlined in red. On "This C is too small by one" the exact row: match, match, then "no match" in green twice. On "all four tosses miss it" the doubles row, "match" under every toss in red, toss 10 ringed, exact A(B r) = (N + 1, 0) ≠ (N, 0) = C r against doubles A(B r) = (N, 0) = C r | grids from the model, verdicts computed both ways, TeX | |
| 10_6 | A tolerance would let the right one through, but then a wrong C inside it passes every toss, with no half guarantee. | The computations go. On "but then a wrong C inside it" the equality rows dim, and under them the check with a tolerance, A(B r) − C r at most 2 in every row, the tolerance computed as the right C's largest gap in doubles; "the right C passes 4 of 4 in doubles; this wrong C, 4 of 4 in exact sums"; a "precisely" card: tolerance 2 lets the right C through, but then this wrong C passes all 4 tosses in exact sums: no half bound. It stands until read, then the chapter fades | TeX, text from facts, lower-third | |

### Notes for 10

- Prediction (10_2): as accepted in the earlier chapter 04: the answer is
  written as it is said and stands through 10_3.
- Every N is upright (`n_tex`: `\text{N}`), the same N as the question and
  the cards (review F8).
- Everything about doubles is computed (F-X4), as accepted.
- The sentence-initial "A" of 10_6 ("A tolerance") is kept verbatim from the
  accepted line; probing it is a pronunciation obligation (A must be read as
  the article here, not as the matrix name).
- Pause targets: before "But many programs", "So a right answer" and "And a
  wrong answer", about 0.85 s.

## 11_the_promise: The promise (~73 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | So the promise is for exact arithmetic. Right answers match every toss, and a wrong one gets past each fresh toss at most half the time. | The opening picture returns, A × B = C with the slip outlined in red. On "Right answers" "a right C: every toss matches"; on "a wrong one gets past" "a wrong C: each fresh toss misses at most half the time", k fresh tosses all miss ≤ 1/2^k, and "in doubles, both can fail" dim; the promise card: in exact arithmetic, with fair coins, all independent and tossed after C is fixed, k tosses all miss any fixed wrong C with chance at most 1/2 to the k | TeX from the bound, lower-third | |
| 11_2 | It assumes fair coins, tossed after C is fixed, and new ones for every check. | At the right, one after another in gold: "fair coins", "tossed after C is fixed", "new coins for every check" | lagged reveal | |
| 11_3 | Back to the start. Redoing this product took twenty-seven multiplications. One toss took nine, and caught the slip in row two. | The promise lines and the assumptions go once the card has been read. At the right: "redoing it: 27 multiplications"; heads, heads, tails lands over C, its kept columns outlined in teal, and "one toss: 9, caught in row 2" in green | coins, highlight, text from facts | |
| 11_4 | At a thousand rows, twenty fresh checks take a fiftieth of the multiplications, and a wrong answer passes them all less than once in a million. | Under the picture: "n = 1000: 20 checks, 20,000,000 of 1,000,000,000 multiplications" and "a wrong C passes all 20: ≤ 1/1,048,576 < 1/1,000,000" | TeX from facts | |
| 11_5 | How many fresh checks would push that below one in a billion? ⟨predict⟩ | The question, "How many fresh checks for below 1 in 1,000,000,000?"; four seconds of silence with the countdown; no count | think countdown | |
| 11_6 | Thirty. Two to the thirty is just over a billion, and at a thousand rows thirty checks still take only three hundredths of the multiplications. | The ? goes; 2³⁰ = 1,073,741,824 > 1,000,000,000, then 30 × n² = 30,000,000, 3/100 of n³ | TeX from facts | |
| 11_7 | So a mistake hides only when its heads columns add up to zero in every row. Flipping the coin over any nonzero entry of the mistake breaks that. | Everything clears. The 1, 1, −1 mistake with toss 011 over it: its row sums 0, 0, 0, "hides". On "Flipping the coin" coin 1 flips to heads, row 2's sum becomes 1 in green, "caught". Then the names to search (Schwartz–Zippel lemma, polynomial identity testing) and the receipts card stand, and the video fades | coins, sums from the model, receipts card | hold |

### Notes for 11

- Prediction (11_5): the question is on screen before its last word; the
  count, 2³⁰ and the cost only in 11_6.
- Thirty is the least k with 2^k above a billion (F-F13.target: 2²⁹ =
  536,870,912 is not); thirty checks at a thousand rows take 30,000,000
  multiplications, 3/100 of the product's.
- 11_2 states the assumptions the bound uses, not a claim that nothing else
  could work (review correction of 2026-10-20).
- 11_7: toss 011 hides the 1, 1, −1 mistake, and 111, one coin away, catches
  it (F-F7.three_in_a_row.hides_from).
- The receipts card names the repository; the names to search follow UIUC
  CS 473 lecture 12's outline (Schwartz–Zippel, polynomial identity testing).
- Pause targets: before "Back to the start" and "So a mistake hides", about
  0.85 s.
