<!-- Claim walk (gate G3) of script.tsv at a06b1e8 plus the de-slopify pass, 2026-10-12. The first part is this lane's dispositions; below the rule is the walker's answer, verbatim (scratch paths shortened to <scratch>/). Fixes are applied in the storyboard; see the commit that lands this file. -->

# Claim walk — Video E

## Dispositions

The walk covered all 61 lines and the on-screen strings: 122 rows, 82 OK, 23
edges that hold, 8 edges that fail as worded, 8 mismatches, 1 unsourced. No
spoken number was wrong and every seeded count replayed on the walker's own
loop. Each problem, in the walker's order:

| # | line | disposition |
|---|---|---|
| 1 | 01_1, 01_4, 01_6 | **fixed.** The hook's logic was wrong twice over (a blind call by Bob is an even bet). It is now: whoever gets the favoured side wins four bets in five, so "a call is fair only if the caller has no idea which side the coin favours. And neither of them can check what the other knows." 01_1 says neither trusts what the other knows; 03_6 pays it off: "it makes no difference if one of them does know." |
| 2 | 05_3 | **fixed.** "So heads then tails is four times as likely, and after two tails it's the other way round"; the picture shows both paths and both counted cases (6,306 against 1,615; 1,681 against 6,442). |
| 3 | 04_6 | **fixed by cutting.** "No fair rule can promise an end" is false for a fair coin. The spoken line now stops at "sixty tosses"; the statement that is true ("no rule that always names a winner within a fixed number of tosses is fair for every coin") is the lower-third, with its reason, and the description has the proof. This also bought back 17 s. |
| 4 | 01_5 | **fixed.** Computer noise is no longer called "this unfair" (Jun and Kocher: within about a hundredth of one half); it is a captioned third icon, and 11_1 says "a little unfair". |
| 5 | 10_4 | **fixed.** One beat: "run the trick again and again on the two things it ignored", the two things named on the picture; "however far you go, they stop just short of it" (true of every row and every number of rounds; only a rule whose rows grow without end attains the ceiling, in the limit). The fact sheet's "always rising" now says "along the doublings shown". |
| 6 | 07_3 | **fixed.** "Each face came up about as often as a fair die's would." |
| 7 | 09_4 | **fixed.** "Low, high, middle is one order. High, low, middle is another", matching the dice shown (four, six, five; then six, four, five). |
| 8 | 11_1 | **fixed.** "Many get them from electrical noise". |
| 9 | 11_3 card, description | **fixed.** "seeds 195101 to 195109, NumPy PCG64DXSM" (`E-S0.seeds`); "every count and chance in this video was computed by code; quoted numbers are cited". |
| 10 | 02_8 | **fixed by cutting.** The sentence is gone from the voice; the band's caption says "a perfectly fair game rarely comes out 500 each" (one time in forty; an oracle cross-check). |
| 11 | 09_6 | **fixed.** "It keeps going, up to six rolls." |
| 12 | 10_1 card | **fixed.** "50–50" with the printed en dash, in the storyboard and the fact sheet; the source line adds "From a symposium held June 29 to July 1, 1949". |
| 13 | 05_4, 11_3 | **fixed.** `E-C2.blum.focs_year` = 1984 is pinned to the paper that was read; `E-C2.blum.year` stays 1986 for the journal version cited in the description. |
| 14 | 05_4 | **fixed.** "Each of those has the same chance, whatever came before it. So they behave like a plain unfair coin". |
| 15 | 01_5 | **fixed.** "An ordinary coin, tossed and caught, can't be weighted to do this"; the lower-third says "by much". |
| 16 | 02_4 | **fixed.** "cuts it again, four fifths of the way up, at the same mark". |
| 17 | 11_3 | **fixed.** "So an unfair coin like ours still has a fair one inside it, as long as it can land both ways." |

Vocabulary: the coin's chance is no longer spoken in percent (03_3 "heads
nine times in ten", 04_5 "ninety-nine times in a hundred"); "cuts" from
02_3 on; "changes" for the memory coin; "edge" for the cube; "settle their
bet" for the dice; "unfair" everywhere for the coin and the noise ("worse",
"loves heads", "lopsided", "the lucky one" are gone); 08_4 "every block has
an equal twin"; 10_4's screen says "rows for 4, 8, … tosses"; the gold words
"same counts, same chance" wait for 06_5, where "counts" is defined, and the
dice get words of their own ("the same rolls, any order: the same chance").
"Sixteen" no longer means two things six seconds apart (02_6 "Four in
twenty-five is sixteen percent").

Shown and said: 05_3's picture has the two paths; 08's counted pairs sit on
08_4; the 10_1 card carries 1949; 11_2's card drops the unexplained "one bit
for every 6"; 04_4's programmers' note is in the picture column; 07_5's
cheaper rule is pointed to in the description.

Speech: 06's three patterns are three short sentences in a beat of their
own; "With four tosses"; "None of them"; "The trick comes from…" (no "two
toss"); 02_2 lists the matching pairs first. The letter p in 03_5 and the
flat list in 09_4 are for the narration pass to listen to; the lexicon says
what the probes could and could not verify for "von Neumann".

Fact sheet and description: the stale quotations are replaced with the
script's sentences; the 64.7% is marked as "starting straight after a head";
the description warns, with Peres's Remark 3, against stopping his procedure
or Elias's the moment enough fair tosses have come out; RFC 4086 is in the
source table for "passwords".

Not changed: "the mathematician John von Neumann" (general knowledge, no
source line); 02_9's reversal (kept on purpose at the de-slopify pass, the
one use of that device in the script).

A narration-truth test now ties 51 spoken phrases to the facts they speak
for (`every_spoken_number_is_the_fact_it_speaks_for` in `src/main.rs`), and
fails if a line that says a number has no row.

---

# Claim walk (G3): `fair_coin` script, 61 lines plus on-screen strings

## Summary

Every spoken number recomputes exactly, and every seeded count replays. Seventeen problems of wording, logic, sourcing or picture match follow, none of them a wrong number.

**Counts by status (122 rows):**

| status | rows |
|---|---|
| OK | 82 |
| EDGE, holds as worded (some need a fence that is present) | 23 |
| EDGE, fails as worded | 8 |
| MISMATCH | 8 |
| UNSOURCED | 1 |

### Problems, worst first

1. **01_4 (with 01_1): the hook's reasoning does not follow.** "Whoever knows the coin best wins" is said about the case where Bob calls, and there Alice's knowledge gives her no move. Bob's blind call wins 80% or 20%, and a call made at random wins exactly one half (0.5 × 0.8 + 0.5 × 0.2). 01_1 also says Bob "suspects it" (heads four in five), in which case he calls heads and wins.
   - 01_4 → "Letting Bob call doesn't fix it. He doesn't know which side is the lucky one, so he's only guessing. Once he's guessed, the bet is just as unfair as before."
   - 01_1 → "Alice knows that, and Bob only suspects the coin is unfair."

2. **05_3: "So it's four times as likely" points at the wrong pair.** The nearest noun is "tails then heads", the less likely one (1/25 against 4/25). The line also gives only the after-two-heads case, so a viewer fresh from 05_2 infers this coin favours Alice overall. It does not: after two tails the counts are 1,681 against 6,442, and the plain trick's whole tally in the run is 9,980 to 10,057.
   - → "…and tails then heads needs two. So heads then tails is four times as likely. After two tails, it's the other way round."

3. **04_6: "No fair rule can promise an end" is false for a fair coin.** One toss of a fair coin is fair and ends at toss one. The reason given (the unbroken run of heads) is the reason for "fair for every coin", which only the lower-third says. For the 80% coin it happens to be true anyway, for a different reason (a bounded rule gives Alice k/5ⁿ, never one half).
   - → "No rule that's fair for every coin can promise one, because a rule that always finishes by some toss must hand an unbroken run of heads to somebody."

4. **01_5: electrical noise "this unfair" is unsourced and contradicts 11_1.** Jun and Kocher report raw bit frequencies "on the order of .5±10-2" under extreme conditions. RFC 4086 §3.3 says only that such bits "will have to be heavily de-skewed". 11_1 itself says "a little lopsided".
   - → "A coin tossed and caught can't really be this unfair, but a spun coin can, and so can a loaded die. The electrical noise inside a computer leans too, though far less."

5. **10_4: four separate issues.**
   - "run the trick again on those" is one more round, which gives 28.8 per hundred. The climb needs it repeated.
   - "ignored, which pairs matched" reads as a relative clause by ear.
   - "never quite reach it" holds for every fixed block size and depth. But Elias's Theorem 2 says a growing-block procedure "realizes the bound" in the limit, and Peres's Remark 2 says a rate "precisely h(p)" can be constructed.
   - Elias's rate does not rise row by row: row 3 gives 10.7 against row 2's 16.0, and block 64 gives 64.62 against block 63's 64.71. The bars shown (4, 8, 16, 32, 64) do rise.
   - → "…Or keep two things the trick ignored. Which pairs matched, and what the matching pairs showed. Run the trick on each of those, again and again. Both climb toward the ceiling, and at any size you choose they stop just short of it."

6. **07_3: "Each face came up about one time in six" against a bar at 136 of 949.** That face came up one time in 6.98, and sits exactly on the edge of the band drawn (136 to 181).
   - → "Each face came up about as often as a fair die's would."

7. **09_4: the spoken orders do not match the dice shown.** The voice names "low, middle, high" and "high, low, middle"; the dice that land are four, six, five (low, high, middle).
   - → "Low, high, middle, or high, low, middle, and four more." (or land four, five, six)

8. **11_1: "They get them from electrical noise" is a universal the source denies.** Jun and Kocher §3: "On computers without a hardware RNG, programmers typically try to obtain entropy… timing user processes".
   - → "Many get them from electrical noise, and raw noise comes out a little lopsided."

9. **11_3 receipts card and description: "the seed, 1951" does not replay.** The runs use seeds 195101 to 195109 on PCG64DXSM. For run 1, `PCG64DXSM(1951)` gives 790 heads, `default_rng(1951)` gives 818, and only `PCG64DXSM(195101)` gives 791.
   - Card → "seeds 195101 to 195109 (NumPy PCG64DXSM)"; same fix in the description's "Every number".
   - "every number in this video was computed by code" is also untrue of the cited literals (1949, 0.508, 350,757, 6, the years), which are pinned constants in `src/facts.rs`. → "every count and chance… computed by code; quoted numbers are cited".

10. **02_8: "A perfectly fair coin wouldn't split five hundred each."** It does 2.52% of the time.
    - → "A perfectly fair coin would rarely split five hundred each, either."

11. **09_6: "It keeps going."** A six-sided die stops at six different rolls.
    - → "It keeps going, up to six rolls."

12. **10_1 quote card: one character differs from print.** The page has "50–50" with an en dash (visibly longer than the hyphens in "heads-heads" on the page image); the card has a hyphen. The hyphen of "coin-tossing" falls at a line break in print, so the page cannot settle it; the fact sheet says the Collected Works reprint was seen and could. The voice says 1949 while the card shows only 1951.
    - → Use "50–50"; add "talk given at a symposium, June 29 to July 1, 1949" to the source line.

13. **05_4 and 11_3: "Blum, 1984" on screen against `E-C2.blum.year` = 1986.** The key is pinned to the Combinatorica version, which the fact sheet says was not seen. Add a key for the FOCS year or change the pin.

14. **05_4: the stated reason is incomplete.** "Those all share one chance" does not by itself make them "a plain unfair coin". They are also independent of each other, because the only thing this coin remembers is the head just seen. Elias p. 866 argues the same short way. It fails for a longer memory.
    - → "Each of those comes after a head, so each has the same chance whatever came before. They behave like a plain unfair coin, and the trick handles that."

15. **01_5 lower-third drops Gelman and Nolan's "substantially".** They also quote a slight lean of 0.503 or 0.497. The spoken "can't really be this unfair" fails for a two-headed coin, which they exclude by name.
    - → "weighting can't make a tossed-and-caught coin favour a side by much"

16. **02_4: "splits it again, bottom to top"** can be heard as a cut that runs bottom to top, while the picture's cut "runs straight across".
    - → "The second toss splits it again, four fifths of the way up, at the same mark, because it's the same coin."

17. **11_3: "as long as it can land both ways"** is the right edge for a coin with one chance and no memory. A coin that strictly alternates lands both ways and holds no fair coin.
    - Optional → "So an unfair coin like ours still has…"

### The six continuity points

1. **Alice green, above the diagonal: consistent everywhere.**
   - Coin: heads then tails is x in [0, 4/5], y in [4/5, 1], the top-left block, above y = x.
   - Dice: with faces laid six down to one from the corner, I checked all 30 off-diagonal blocks. "First roll bigger" is exactly the set above the diagonal (area 137/400 each side).
   - A six then a five is x in [0, 1/2], y in [1/2, 7/10], on Alice's side.
   - The "green triangle" of 08_4 is a staircase of 15 blocks.
2. **02_8 dots: consistent** with `model.rs:717` (`d.unit() < p` is heads), with the first draw across and the second up. One caution: the die rule (first face whose running total exceeds the draw, faces one to six) runs the opposite way to the dice grid. Dots must never be plotted there by raw draw.
3. **Cube: correct.**
   - Three tosses give eight pieces in groups of 1, 3, 3, 1, with volumes 64/125, 16/125, 4/125 and 1/125.
   - The three two-heads slabs are each 4/5 × 4/5 × 1/5.
   - The bricks for four, five, six are six arrangements of 1/10 × 1/5 × 1/2, each 1/100.
4. **Screen counts against `facts_p3.tsv`:** 93 on-screen numbers checked by key; the only two mismatches are the two "Blum, 1984".
5. **Quote card:** starts and ends where it should; every word matches the page image except the en dash in "50–50". "In a talk in 1949", "the printed summary" and "rigorously unbiased" are all supported by the title page and p. 36.
6. **Intel:**
   - "Passwords and secret codes" is in RFC 4086's abstract, not in the Intel paper; RFC 4086 is saved but absent from the fact sheet's source table.
   - "A little lopsided", "one chip maker", "based on" and the card's two fragments are verbatim or supported.
   - The year is the white paper's date (April 22, 1999).

### Vocabulary: one word, two ideas, or two words, one idea

- **Coin's chance in percent** at 03_3 ("Ninety percent heads") and 04_5 ("ninety-nine percent heads"). The storyboard rule reserves percent for a block's share and puts the coin's chance in fifths.
- **"pairs"** as a verb for a second idea at 08_4 ("the diagonal pairs every block with its equal").
- **"group"** on screen at 10_4 ("groups of 4, 8, 16, 32 and 64 tosses") means a block of tosses, not patterns with the same counts.
- **"counts"** appears in gold at 06_3 one beat before 06_4 defines it. It is reused for dice at 09_3 ("Same counts, same chance"), where it was never defined for faces.
- **"sixteen"** is 16 small squares for two heads at 02_5 and 16 percent for each thin block at 02_6, six seconds apart. → "Four each. Exactly equal. Four in twenty-five is sixteen percent."
- **"splits", "mark", "cut"** for one thing: 02_3 and 02_4 say "splits… at the same mark"; "the cut" is first spoken at 03_4. 03_4 says "the cut" where two cuts move together.
- **"split"** for two ideas: cutting the square (02_3) and dividing the wins (02_8).
- **"side"**: a coin's side (01_4, 03_6), "change of side" (05_3) and the cube's side (09_1).
- **"bet", "game", "decision"** for one idea: "four bets in five" (01_3), "four games in five" (05_2), "a fair decision" (08_2).
- **Unfair by other names:** "lopsided" (11_1), "loves heads" (07_4), "favours" (03_4, 03_6), "the lucky one" (01_4), "worse" (03_3). 10_2's "does worse" means a lower yield.
- **"stop", "end", "finishes"** inside 04_6.

### Said but not shown, shown but not said

- **05_3:** the voice argues by changes of side; the picture argues by a stepped cut with products. Nothing shows H→H→T against H→T→H.
- **05_3:** the after-two-tails case is neither said nor shown (problem 2).
- **07_5:** "There are cheaper rules" is said; none is shown.
- **08_5:** the caption "counted in 3,000 pairs of rolls: 1,021 for Alice…" sits over the beat where the grid has become the coin's square. It belongs at 08_3 or 08_4.
- **09_4:** dice shown against orders spoken (problem 7).
- **10_1:** 1949 is spoken; the card shows only 1951.
- **11_2:** the card's "one bit for every 6 raw binary samples" sits oddly beside 10_1's "one fair toss for every four". The reason (Intel "enhanced" the corrector) is nowhere on screen, and "bit" is never explained.
- **04_4:** the programmers' lower-third (rejection sampling) exists only in the chapter note, not in the picture column.
- **06_3:** "same counts, same chance" is shown before the word is defined.
- **02_4:** "bottom to top" against "runs straight across".

### Likely speech-synthesis stumbles

- **03_5:** the letter "p" five times, twice sentence-final ("one minus p.", "Equal for every p."). No lexicon entry or probe exists for it.
- **06_2:** "heads tails heads" is a list without commas inside a comma list. → "Here are three patterns. Two heads then a tail. Heads, tails, heads. And a tail then two heads."
- **06_5:** "With four, one, four, six, four, one" runs the toss count into the list. → "With four tosses, one, four, six, four, one."
- **09_4:** "Low, middle, high, or high, low, middle, and four more" will be read as one flat list.
- **10_4:** the garden path at "ignored, which pairs matched".
- **10_1:** "The two toss trick" may be heard as "to toss"; "von Neumann" is unverified by the probes (the lexicon says so).
- **02_2:** "Two heads, heads then tails" puts "heads, heads" together.
- **06_3:** a one-word sentence, "None.", after a held question.

### Notes on the fact sheet and description

- **Stale quotes in the fact sheet.** It quotes sentences the script no longer says: finding 3 ("neither toss affects the other"), finding 5 ("this is the simplest way, not the cheapest"), and the rows "sooner or later a pair decides", "at most 72", "At least half the pairs are thrown away".
- **"Always rising" is too strong.** The edge table's "blocks 2–64: always below, always rising" is true only along doublings.
- **Description's 64.7%** (games decided within 8 tosses) is right for a game that starts straight after a head. From a long-run start it is 67.3%.
- **Missing caution in "Peres's procedure, as code".** Peres's Remark 3 says stopping when you have m fair tosses is not fair for his procedure or Elias's.
- **Samuelson attribution is sound.** His abstract, saved in `openalex_records`, says "concentrating only on tosses that follow a common outcome".

## What I ran

All scratch files are in `<scratch>/`. Nothing in the repository was touched.

- **`exact.py`** (`python3 -I`, `fractions`):
  - blocks, 32 and 68 percent, 25/8 pairs and 25/4 tosses, costs at 1/2, 9/10 and 99/100;
  - the fair band and P(500–500);
  - two coins; the memory coin (stationary share, 4/25 against 1/25, the plain rule's 8/13 and 5/13) and the after-a-head tosses by exact enumeration;
  - slabs, Pascal rows, the six patterns, 625/24, the cheaper die's 12.5;
  - the loaded die (137/400 each way, the brick, all-differ chances for 2 to 6 rolls, the n! orders equal), with a zero-chance face and a two-faced die;
  - the geometry of both squares;
  - entropy, Peres's recursion to depth 20, Elias's rate for blocks 1 to 129, each leftover alone (17.9 and 39.3), and both bands.
- **`replay.py`** (`uv run --no-project --with numpy python3 -I`): my own loops, one draw at a time, on `Generator(PCG64DXSM(195100 + i))`. Runs 1, 2, 4, 5, 6, 7, 8 and 9 reproduce every spoken and on-screen count.
- **`seedcheck.py`**: shows seed 1951 alone does not reproduce run 1.
- **`desc.py`**:
  - the deadline lean (64.7, 53.9, 50.5 percent);
  - Blum at once (15, 15, 35, 35) and on return (25 each);
  - a single run of 100 tosses beating 72 (7 of 200,000 Monte Carlo runs; mean yield about 52);
  - the sliding-window misuse returning the first toss.
- **`screen_vs_facts.py`**: 93 on-screen numbers typed from the picture column, compared by key with `facts_p3.tsv`.
- **Read, not recomputed:**
  - the page image `gb_PA36.png`, cropped and enlarged, plus the OCR and the LANL text;
  - Jun and Kocher's raw text; Peres pp. 590–595; Elias pp. 865–869; Samuelson's abstract; Blum's FOCS text;
  - Gelman and Nolan; Diaconis, Holmes and Montgomery pp. 218 and 231; Bartoš (arXiv); Chance News 11.02; RFC 4086;
  - Crossref and OpenAlex years; `model.rs` lines 704–717; `facts.rs` lines 631–730.
- **Not checked:** Samuelson's own two pages (not saved), and "the mathematician" as a description of von Neumann (no source line; common knowledge).

## Table

| line | claim | source or computation | status | note / fix |
|---|---|---|---|---|
| 01_1 | "lands heads four times in five" | model coin; 4/5 = 80% = `E-F1.heads_percent` | OK | |
| 01_1 | "Alice knows that, and Bob only suspects it" | against 01_4 "doesn't know which side is the lucky one" and 01_6 | MISMATCH | if he suspects heads he calls heads; "…Bob only suspects the coin is unfair" |
| 01_1 | "can still settle a bet… perfectly fairly, using nothing but this coin" | 4/25 against 4/25 exactly | EDGE holds | for this coin; not at chance 0 or 1, not with memory |
| 01_1 | screen: tag "a model coin (one we made up): heads 80%", second bar | storyboard, key | OK | |
| 01_2 | title card | matches the title | OK | |
| 01_3 | "seven hundred ninety-one"; screen 791, 209, "1,000 tosses" | replay run 1: 791 and 209 | OK | |
| 01_3 | "Whoever gets heads wins about four bets in five" | 79.1% counted, 80% model | OK | |
| 01_4 | "Letting Bob call doesn't fix it… Whoever knows the coin best wins" | Bob calling at random wins exactly 1/2; Alice has no move | EDGE fails | problem 1 |
| 01_5 | "A coin tossed and caught can't really be this unfair" | Gelman and Nolan abstract: cannot "weight a coin so that it is substantially more likely…" | EDGE fails | a two-headed coin can ("unless it is double-headed"); true of weighting |
| 01_5 | "a spun coin can" | DHM p. 218 "fewer than 10% heads"; G&N checker "tails 77 times in 100 spins" | OK | |
| 01_5 | "so can a loaded die" | G&N: "dramatically unequal"; "only one 6 in 120 throws" | OK | |
| 01_5 | "or the electrical noise inside a computer" (this unfair) | Jun and Kocher: ".5±10-2"; RFC 4086: "heavily de-skewed" only | UNSOURCED | contradicts 11_1; problem 4 |
| 01_5 | lower-third "weighting can't make a tossed-and-caught coin favour a side" | G&N say "substantially" | EDGE fails | add "by much" |
| 01_6 | "Alice knows its secret while Bob doesn't" | setup; agrees with 01_4 | OK | |
| 02_1 | "look at the two results as a pair" | definition | OK | |
| 02_2 | "A pair can be four things" | 2 × 2 | OK | "heads, heads" adjacency for speech |
| 02_3 | "Heads gets four fifths of the width" | cut at 4/5 from the corner | OK | |
| 02_4 | "at the same mark, because it's the same coin" | holds without memory; 05_3 shows the fence | EDGE holds | "bottom to top" ambiguous; problem 16 |
| 02_5 | "a block's share of the square is its pair's chance" | product of sides; needs 03_7 | OK | |
| 02_5 | "sixteen of the twenty-five… one" | 16/25, 1/25 | OK | |
| 02_6 | "Four each. Exactly equal. Sixteen percent" | 4/25 = 16% both | EDGE holds | "sixteen" collides with 02_5 |
| 02_7 | Alice heads then tails, Bob tails then heads; colours | top-left block above y = x | OK | |
| 02_7 | lower-third "Reuse a toss… plain heads-or-tails back" | output equals the first toss; simulated 0.7986 | OK | |
| 02_8 | "494… 506"; screen 3,128 pairs, 2,128 grey, first eight games | replay run 2, all identical | OK | 2,128/3,128 = 68.0% |
| 02_8 | "A perfectly fair coin wouldn't split five hundred each" | P(500–500) = 2.52% | EDGE fails | "would rarely split" |
| 02_8 | screen "a fair game: 469 to 531, about 19 times in 20" | exact binomial: 95.4% | OK | |
| 02_8 | picture: heads when the draw is left of, or below, the cut | `model.rs:717` `unit() < p` | OK | |
| 02_9 | "The coin is as unfair as ever. The game isn't." | 791/209 against exact 1/2 | EDGE holds | |
| 03_1 | 4/5 wide, 1/5 tall; 1/5 wide, 4/5 tall | block coordinates | OK | |
| 03_2 | flip across the diagonal lands on the other | reflection of [0,4/5]×[4/5,1] in y = x | OK | |
| 03_3 | "Ninety percent heads" | slider stop | OK | chance in percent, against the vocabulary rule |
| 03_4 | "Nine percent each" | 9/10 × 1/10 | OK | |
| 03_4 | "One block is always the mirror image of the other" | p(1−p) = (1−p)p for every p | EDGE holds | at 0 and 1 both blocks are empty |
| 03_5 | "Equal for every p" | identity | EDGE holds | letter "p" for speech |
| 03_6 | "doesn't need to know… doesn't even need to trust Alice" | the rule never uses p | EDGE holds | fails if Alice can make the two tosses differ; 03_7 fences it |
| 03_6 | lower-third "nothing to measure first, and no stored constant" | same | OK | |
| 03_7 | "lean on two things… independent" | sufficient | OK | "can land both ways" only at 11_3 |
| 03_7 | lower-third "enough: independent tosses… strictly between 0 and 1" | exact statement | OK | |
| 04_1 | "most pairs get thrown away… thirty-two times in a hundred" | 17/25 and 8/25 | OK | |
| 04_2 | question | none | OK | |
| 04_3 | "about one time in three… three pairs… six tosses. Exactly, six and a quarter" | 0.32; 25/8; 25/4 | EDGE holds | exact as an average |
| 04_3 | screen "2 tosses ÷ 0.32 = 6.25 tosses a game (model)" | 25/4 | OK | |
| 04_4 | "six point two one"; histogram | replay run 4: 6.2076; 3209, 2170, 1562, 926, 701, 461, 320, 205, 446 | OK | |
| 04_5 | "A fair coin still averages four tosses" | 1/(1/4) = 4, the least possible | EDGE holds | |
| 04_5 | "ninety-nine percent… about a hundred and one" | 10000/99 = 101.01 | OK | |
| 04_6 | "nobody can promise when a game will stop" | 0.68ⁿ > 0 | EDGE holds | |
| 04_6 | "longest… sixty tosses" | replay: one game of 60, next 58 | OK | |
| 04_6 | "No fair rule can promise an end, because…" | one toss of a fair coin is a counterexample | EDGE fails | problem 3 |
| 04_6 | screen "after 20 tosses: 2.1%", 446 ticks, lower-third | 0.68¹⁰ = 2.11%; replay; P(1) is 0 or 1 | OK | |
| 05_1 | setup | none | OK | |
| 05_2 | "One cut moved… aren't mirror images" | green 0.8 × 0.5, violet 0.2 × 0.5 | OK | |
| 05_2 | "Forty percent against ten" | 2/5, 1/10 | OK | |
| 05_2 | "four games in five, as if they'd never bothered"; "807 for Alice" | 0.4/0.5 = 4/5; replay 807 | OK | |
| 05_3 | "Count this one's heads and it looks fair" | stationary share 1/2; replay 50.04% | OK | |
| 05_3 | "copies its last result four times in five" | replay 79.9% | OK | |
| 05_3 | "one change of side… two. So it's four times as likely" | 4/25 against 1/25; replay 6,306 against 1,615 | MISMATCH | "it" reads as tails then heads; problem 2 |
| 05_3 | screen: tag, 16% "4/5 × 1/5", 4% "1/5 × 1/5", caption | exact; replay | OK | "half heads overall" is keyed to a counted 50.0, not a model fact |
| 05_3 | lower-third: 51 in 100; 350,757; 0.508; "memory when each toss starts the way the last one landed" | DHM abstract; Bartoš abstract and "starting from the same side it landed in the previous trial" | OK | |
| 05_4 | "use only the tosses that come straight after a head" | Samuelson abstract; Blum p. 426; enumeration 0.64/0.16/0.16/0.04; replay 7,847 against 8,043 | OK | |
| 05_4 | "Those all share one chance, so they behave like a plain unfair coin" | true for one-toss memory | EDGE holds | reason incomplete; problem 14 |
| 05_4 | caption "200,000 tosses, 15,890 games: 7,847 against 8,043" | replay | OK | |
| 05_4 | lower-third "Samuelson, 1968. Fair when played to the end… Blum, 1984" | deadline computation; key says 1986 | MISMATCH | problem 13 |
| 06_1 | "same results… in a different order" | one head, one tail | OK | |
| 06_2 | "three need a cube"; three patterns | geometry | OK | list needs commas |
| 06_3 | "Each is four fifths, times four fifths, times one fifth"; 12.8% | 16/125 each | OK | |
| 06_3 | "order never changes a chance" (fenced) | pᵏ(1−p)ⁿ⁻ᵏ | EDGE holds | fails for the memory coin, which the sentence excludes |
| 06_3 | screen: gold "same counts, same chance" | storyboard | OK | shown before "counts" is defined |
| 06_4 | "one, two and one. The middle group is our trick" | row 2 | OK | |
| 06_5 | rows 1, 3, 3, 1 and 1, 4, 6, 4, 1; Pascal's triangle | binomials | OK | add "tosses" after "With four" |
| 06_6 | "inside a group, every pattern is equally likely, however unfair" | same formula | EDGE holds | |
| 06_6 | screen bars 0.16, 2.56, 15.36, 40.96, 40.96 | recomputed | OK | |
| 06_7 | "The order is perfectly fair" | given the counts | EDGE holds | |
| 07_1 | "Six patterns, all equally likely" | 16/625 each | OK | |
| 07_2 | "If exactly two are heads, the pattern picks the face" | rule | OK | |
| 07_3 | "Twenty-four thousand… nine hundred forty-nine" | replay run 7: 949 of 6,000 blocks | OK | |
| 07_3 | "Each face came up about one time in six" | 136/949 is one in 6.98 | MISMATCH | problem 6 |
| 07_3 | screen bars 166, 161, 136, 160, 167, 159; band 136 to 181 | replay; band recomputed | OK | chi-squared 4.06 on 5 |
| 07_4 | "rarely… about twenty-six tosses"; 15.4%, 26.0, 25.3 | 96/625; 625/24 = 26.04; 24000/949 | OK | |
| 07_5 | "There are cheaper rules" | 3/0.48 + 6.25 = 12.5 | OK | none shown |
| 07_5 | "Any group of patterns with the same counts is a fair choice" | same formula | EDGE holds | |
| 08_1 | "a six half the time"; caption counts | 10/20; replay 318, 283, 579, 607, 1237, 2976 | OK | |
| 08_2 | question | none | OK | |
| 08_3 | rule and colouring | 137/400 each; 30 blocks checked | OK | |
| 08_4 | "A six then a five is exactly as likely as a five then a six" | 1/10 each | EDGE holds | |
| 08_4 | "the diagonal pairs every block with its equal" | mirror blocks equal; zero-chance face tested | EDGE holds | "pairs" as a verb; "triangle" is a staircase |
| 08_5 | "a coin is just a die with two faces" | two-faced die: 4/25 each | OK | |
| 08_5 | caption 1,021, 1,020, 959 | replay run 8 | OK | on the wrong beat |
| 09_1 | "three need the cube again" | geometry | OK | |
| 09_2 | "Each order lands in one of these six bricks" | six arrangements | OK | |
| 09_3 | "A tenth, by a fifth, by a half"; 1/100 | faces 4, 5, 6 | OK | |
| 09_4 | "the order they came in is a fair choice among six" | full listing: six equal, 5.43% each | EDGE holds | needs three faces of non-zero chance |
| 09_4 | "Low, middle, high, or high, low, middle" | dice shown are 4, 6, 5 | MISMATCH | problem 7 |
| 09_5 | "six hundred fifty"; bars; band 90 to 127 | replay; recomputed | OK | |
| 09_6 | "It keeps going" | seven rolls cannot all differ | EDGE fails | "up to six rolls" |
| 09_6 | "twenty-four orders" | 4! | OK | |
| 09_6 | "rolls that all differ get rarer, so each step costs more" | 68.5, 32.6, 10.4, 2.0, 0.18%; 2.9, 9.2, 38.5, 249, 3,333 rolls | EDGE holds | nested events |
| 09_6 | lower-third 33, 9, 10, 39 | 32.55%, 9.22, 10.38%, 38.54 | OK | |
| 10_1 | "comes from… John von Neumann" | Stout and Warren p. 212; Hoeffding and Simons p. 341 | OK | "mathematician" has no source line |
| 10_1 | "in a talk in nineteen forty-nine" | title page: symposium "June 29, 30, and July 1, 1949"; p. 36 "earlier in this symposium" | OK | |
| 10_1 | "The printed summary calls it rigorously unbiased" | p. 36: "Summary written by George E. Forsythe"; "rigorously unbiased" | OK | |
| 10_1 | "At most twenty-five percent as efficient as ordinary coin tossing" | p. 36; max p(1−p) = 1/4 | OK | attributed to the summary, correctly |
| 10_1 | quote card, character by character | page image | MISMATCH | "50–50" is an en dash in print; problem 12 |
| 10_1 | source line; "a fair coin: one fair toss for every four" | title; 1/4 | OK | 1949 not shown |
| 10_2 | "about sixteen fair ones" | 50 pairs × 0.32 = 16 | OK | |
| 10_3 | "right four times in five… holds less than a fair one" | 4/5 | OK | |
| 10_3 | "The most any rule can average is just over seventy-two" | h(0.8) = 72.19; Elias Theorem 1 | EDGE holds | long-run; on 100 tosses alone Peres averages about 52 |
| 10_3 | "the coin's entropy"; lower-third "at most 72.2" | 72.19 | OK | |
| 10_4 | "Use the bigger groups in deeper rows" | Elias p. 868; 28.2, 37.1, 49.4, 58.4, 64.6 | EDGE holds | not row by row |
| 10_4 | "keep what the trick ignored… run the trick again on those" | Peres p. 591: u is every pair's sum mod 2, v the matching pairs' value | OK | one round only; garden path |
| 10_4 | "Both climb toward the ceiling, and never quite reach it" | depths to 20, blocks to 129 all below | EDGE holds | Elias Theorem 2 attains it in the limit |
| 10_4 | screen bars, dots 16.1, 28.7, 38.8, 69.6, lower-third | replay run 9: 16114, 28734, 38754, 69624 | OK | "groups of 4… tosses" misuses "group" |
| 11_1 | "for passwords and secret codes" | RFC 4086 abstract; Jun and Kocher §1 | OK | RFC not in the source table |
| 11_1 | "They get them from electrical noise" | Jun and Kocher §3 | EDGE fails | "Many get them" |
| 11_1 | "raw noise comes out a little lopsided" | "significant biases… .5±10-2" | OK | |
| 11_2 | "Pair the readings up, and throw away the pairs that match" | §4.3: "outputting nothing for [0,0] or [1,1]" | OK | |
| 11_2 | "In nineteen ninety-nine… Intel built a corrector based on this trick" | white paper, April 22, 1999 | OK | year is the paper's |
| 11_2 | card | both fragments verbatim | OK | "1 for 6" unexplained |
| 11_3 | "still has a fair one inside it, as long as it can land both ways" | true for one chance, no memory | EDGE holds | optional "like ours" |
| 11_3 | "setting aside how many heads came up, and looking only at the order" | 06_7 | OK | |
| 11_3 | card "every number in this video was computed by code" | cited literals are pinned constants | EDGE fails | problem 9 |
| 11_3 | card "the seed, 1951" | seed 1951 gives 790, not 791 | MISMATCH | problem 9 |
| 11_3 | card names and years | OpenAlex, Crossref; Blum key 1986 | MISMATCH | only "Blum, 1984" |
