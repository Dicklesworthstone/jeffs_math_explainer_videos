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

One tiny example runs through the whole video, and stays on screen at the
top of every chapter: A = (1 2 1 / 0 3 1 / 3 2 0), B = (2 1 0 / 1 0 2 /
2 1 3), and the claim C, their product with one slip (row 2, column 2 says
2; the product has 1). The true product's nine entries are the digits 1 to
9, each once.

| Chapter | Spine | Status |
|---|---|---|
| `01_check_with_coins` | someone hands over C; redoing it is nine entries of three multiplications, a billion at a thousand rows; one spot check misses a single wrong entry eight times in nine; a coin for each column: add C's heads columns, add B's and multiply by A (nine multiplications); they must match if C is right, and do not; of the eight tosses, how many miss? Four, exactly the tails ones over the wrong entry's column; no wrong answer hides from more than half, so toss again: twenty fresh tosses all miss less than once in a million | written 2026-10-19 (scene, facts, oracle); not yet compiled, rendered or narrated: root's native batch |
| `02_never_more_than_half` | other mistakes on the same A and B (`model::SHAPES`): two wrong entries in a row pass two tosses of eight, a cancelling pair passes four, three in a row pass three (not a power of two), one in every column passes only all tails; the proof: pair the tosses by the coin of a column where C is wrong, and in each pair at most one passes; checked on every small mistake (19,682 at three by three, 624 at two by two) | named, not storyboarded |
| `03_toss_again` | two independent tosses: 16 of the 64 pairs both miss, a quarter exactly; the same toss twice is still half; a toss whoever wrote C saw first can be beaten (every one of the eight has a wrong answer it lets through, `model::blind_spot`); so the coins must be fresh: independent, and tossed after C is fixed | named, not storyboarded |
| `04_exact_and_rounded` | the cost at scale (twenty checks at a thousand rows: a fiftieth of the multiplications of redoing it); the theorem is about exact arithmetic: in doubles a right answer fails a toss and a wrong one passes all four (`model::ROUNDED_*`), so a tolerance turns it into a different question; the ending: the rigorous form of the opening picture | named, not storyboarded |

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
fixed; "independently" in the exact version).

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
| the think ring and "?" | `hero` `#FFD866` |
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
| 01_9 | Because every miss leaves out the column that holds the wrong entry. | C's column 2 is outlined in red, "should be 1" beside its slip. On "leaves out the column" every toss's middle coin is ringed: red around the tails coin of each miss, green around the heads coin of each catch | highlight, rings | |
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
