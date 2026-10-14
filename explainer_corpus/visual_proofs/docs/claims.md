<!-- Claim walk (gate G3) of script.tsv at 52a1a1d (plus one de-slopify edit at 15_3), 2026-10-13. The first part is this lane's dispositions; below the rule is the walker's answer, verbatim (the repository path written <repo>, scratch scripts in claims_walk/). Fixes are applied in the storyboard and script.tsv at b6e8b50; the new facts have tests and oracle rows. -->

# Claim walk: Video I

## Dispositions

The walk covered all 89 spoken lines and the on-screen strings: 89 rows, 42 OK,
22 edges that hold, 2 edges that fail as worded, 8 mismatches, 7 unsourced, 8
misleading. No spoken number was wrong and every number recomputed on the
walker's own scripts. The problems were in the words around the numbers. Each
one, in the walker's order (the numbers are the walker's):

| # | line | disposition |
|---|---|---|
| 1 | 15_5 | **fixed.** "A check up to some size does not prove the next size", on screen and in the misconception row; "It also checked the sums up to a thousand" is said (`F-K.sum.through`). Decision 25 |
| 2 | 09_2, 09_3 | **fixed.** 09_2 says the step's extra column and "every full slab"; 09_3 says the gold pieces make three slabs of five by five, one under, one behind, one beside |
| 3 | 03_6 | **fixed.** "A picture that shows why, and checks what it took for granted, is a proof"; the lie says what it took for granted and is not one. Decision 24 |
| 4 | 02_7 | **fixed.** "and some of them you can see" |
| 5 | 13_1 | **fixed.** "One more." The owner's opinion that it is famous is no longer in the voice |
| 6 | 08_5 | **fixed.** "published this picture in nineteen eighty-four, under the title Proof Without Words, Sum of Squares"; nothing dates the picture itself, and the fact sheet's S8 now says "partly verified" |
| 7 | 02_4 | **fixed.** "Place the points so that no three lines cross at one place inside the circle"; the walker's counterexample (the video's own three-through-one set is uneven) is in the fact sheet. Decision 27 |
| 8 | 06_2 | **fixed in part.** "The square is now twenty-one on a side" is said; "four hundred and forty-one" stays on screen (`F-K.sixth.total`, new) because the chapter was at its 85 s cap |
| 9 | 12_7 | **fixed in part.** "So add the row to itself, shifted one place" is said; the volumes and 625 left the screen for the description (`F-B.fourth.by_kind`, `F-B.fourth.total` stay facts) |
| 10 | 14_7 | **fixed.** "Martin Gardner says the trick began with Paul Curry, a New York amateur magician, in nineteen fifty-three" |
| 11 | 06_6 | **fixed.** "A version of the picture with the slabs cut in half was published in nineteen eighty-five"; Aryabhata is off the screen's date line (the description carries him) |
| 12 | 12_6 | **fixed.** "Slide the cut anywhere inside"; `a_cut_at_the_end_of_a_side_leaves_one_block` |
| 13 | 14_3 | **fixed.** "The same pieces have room for one more tile" |
| 14 | 13_6 | **fixed.** "With shorter sides two and five, the tilted square is twenty-nine, and the long side is no whole number. This is Pythagoras's theorem." Three spoken-number rows |
| 15 | 07_2 | **fixed.** "You may slice in half the cubes that end up on top. What is the fewest number of these stacks, turned to fit, that make a solid box?" Walker note 3 found the first draft's reason wrong once halves are allowed; its own argument rests on an asserted premise ("extents 4, 4 and 3"), so the fact sheet's `F-S.fewest` is derived instead from the box's size (a stack is 4 by 4 by 4 in every direction), computed in the model and again in the oracle, with its evidence level stated: necessary conditions on the cube lattice, not a search over every packing. Decision 28 |
| 16 | 07_3 | **fixed.** "the second and third are turned to fit" |
| 17 | 05_1 | **fixed.** "Across all five cubes, that is fifteen slabs" |
| 18 | 13_7 lower-third | **fixed.** "No source we found says who first drew this picture" |
| 19 | 10_4 tag | **fixed.** `F-H.float.reaches_one` = 54 in the model and the oracle (Python floats are the same binary64) |
| 20 | 14_6 lower-third | **fixed.** "so 2 × 8 − 3 × 5 = 1 and the slopes 2/5 and 3/8 differ by only 1/40" (`F-L.slopes.differ`) |
| 21 | 15_6 | **fixed.** "Looking at a picture is not checking it." |
| 22 | 11_5 | **fixed.** "the corner that is left" |
| 23 | 13_5 | **fixed.** "The big square is forty-nine and the four triangles are twenty-four, and neither changed, so the space left over cannot change" (Maya's keep-this sentence, with its numbers) |
| 24 | 03_5 | **fixed.** One hundred and ninety-nine tiles, a hundred by a hundred, ten thousand |
| 25 | 04_5 | **fixed.** Five thousand and fifty tiles |
| 26 | 11_6 | **fixed.** "in ancient Greece": a number with no key is gone rather than keyed |
| 27 | 05_2 | **fixed.** "the corner of the layer" |

Notes of the walk: (1) `protected.tsv` cited 06_5 for the al-Karaji lines; it is
06_6 now, "no three lines" is in the script too, and six phrases that carry
this walk's and the audit's fixes are protected (33 in all). (2) **Not
resolved:** whether Nicomachus's passage generalises past the third cube; S2
records only what was seen, the spoken "every cube is a run of odd numbers" is
true (tested to k = 200) and is the protected phrase, and the passage is for
the owner's library copy of D'Ooge. (3) The 07_2 note is rewritten (above).
(4) The fact sheet's stale quotes, the orphaned `F-X.al_karaji.after_nicomachus`
(kept for the description) and the 02_6 row's wrong key are fixed; the
storyboard header lists the card's pass at 10_6; S8 says "partly verified".
(5) The numbers with no key are presentation parameters (the 12-column sweep,
the lens factor, levels two to eight, the cut sliding to 4 and 1).

Length: every chapter is at most 85 s by the G2 estimate (17.7 min over 15
chapters, against the order's 12 to 16). The fixes that say a step the screen
shows added words; the words that carried no step went (06_5's "cut every
cube's layer", 13_1's opener, 14_3's repeat of the outline, 12_2's "Four
pieces"). The owner's named cuts, chapter 10 folded into 11 and 12_7 to the
description, bring it to about 16.4.

---

# Claim walk (G3): `visual_proofs` script, 89 lines plus on-screen strings

Walked against the files as they stood at 2026-10-13 20:50 (sha256 prefixes: `explainer_corpus/visual_proofs/script.tsv` bd7418836c6a, `docs/storyboard.md` aab275ed4353, `docs/fact-sheet.md` b1d26b623ed1, `oracle.py` 3475bc93286d, `src/model.rs` 6a1392b13233).

What was read: all 89 spoken line ids of `explainer_corpus/visual_proofs/script.tsv` (verified equal to the storyboard words column, same order; 01_4 is the silent title), the whole storyboard (picture column, cards, lower-thirds, notes), the whole fact sheet (sources S1 to S17, simplification table, edge table, every key), `facts_printed.txt`, `oracle.py`, `src/model.rs`, and, because a revision must keep them, `docs/protected.tsv`, `docs/decisions.md` and `docs/brief.md`.

How: every number, step, quantifier, superlative and historical verb was walked against the picture column and the sources table, and recomputed in exact integers or Fractions by scripts in `claims_walk/` (list in appendix A). `oracle.py` was run to a scratch file and its 182 keys equal `facts_printed.txt` key for key except one: `F-C.evenly_spaced.regions` (30) is printed by the oracle only. Nothing under `<repo>` was edited, created or deleted.

Short verdict: every number the voice speaks is right and matches a fact key (one exception, 11_6's "two thousand years", has no key). The defects are in words around the numbers: two quantifiers that fail at an edge, three unsourced superlatives or age claims, a definition of proof and a stated condition that are looser than the card and the facts, and eight places where the screen shows a step or a number the voice never says.

---

## (a) Summary

Counts by status over all 89 rows (one status per row, the worst finding on the row; other findings on the same row are in the problem list):

| status | rows |
|---|---|
| OK | 42 |
| EDGE (holds as worded) | 22 |
| EDGE (fails as worded) | 2 |
| MISMATCH | 8 |
| UNSOURCED | 7 |
| MISLEADING | 8 |
| total | 89 |

Problems found: 27 (7 must fix before narration, 9 should fix, 11 low). The 22 "EDGE (holds)" rows are the always/never/every/any/nothing sentences that were tried at their edges and survived; each edge is listed in appendix B.

Must fix before narration (the voice would say something false or unsupported):
1. 15_5 "says nothing about the next size" fails at the cubes' own identity (a polynomial edge) and as evidence.
2. 09_2 skips the step's extra column and the regrouping into three 5 by 5 slabs, the step the proof turns on.
3. 03_6 defines a proof as a picture that "says what it took for granted"; the lie says it too.
4. 02_7 "the best ones you can see": unsourced superlative.
5. 13_1 "maybe the most famous picture in mathematics": unsourced superlative.
6. 08_5 "This picture is not ancient": no source dates the picture, only Siu's printing.
7. 02_4 "spread unevenly" is not the condition for thirty-one, and the video's own thirty-region set is unevenly spaced.

---

## (b) Problems, worst first

Replacement wordings obey the voice rules (one sentence at a time; no colons, semicolons, dashes or parentheses; numbers spelled out; no single letters; "L-shaped" kept only where the line already has it) and keep the line id and every protected phrase of `docs/protected.tsv` on the line. On-screen strings are marked as such; the voice rules do not apply to them.

### 1. 15_5, EDGE (fails as worded), must fix
Spoken: "A check up to some size says nothing about the next size." The same string is on screen in the receipts strip and is quoted in the fact sheet's Misconceptions row 15.

Edge tried: the cubes' own identity. 1³ + … + n³ and (n(n + 1) ÷ 2)² are both polynomials of degree four in n. They agree at n = 0 to 4, and the fifth finite difference of the left side over n = 0 to 11 is zero, so five checks together with the degree bound settle every n (`70_checks_edge.py`). Without the degree bound the same five checks prove nothing (a function that differs by n(n − 1)(n − 2)(n − 3)(n − 4) agrees at all five). So a check alone does not prove the next size, but "says nothing" is false as evidence (a pass to forty rules out every counterexample below forty) and false once a reason is added. 02_7's "prove nothing" is the defensible form; this sentence should match it.

Also on this line: the screen adds "every sum up to 1,000" (`F-K.sum.through`), which the voice does not say, and which is not "piece by piece".

Replacement:
> A program rebuilt these pictures piece by piece, the cubes up to forty, and Siu's box up to twenty-four levels. It also checked the sums up to a thousand. That is still only checking. A check up to some size does not prove the next size.

On screen and in `docs/fact-sheet.md` Misconceptions row 15, change "says nothing about" to "does not prove".

### 2. 09_2, MISMATCH, must fix
Spoken: "Give each stack one more level, a five by five slab at its base. Every slab of the block, four by five, becomes five by six, with an L-shaped layer of two arms and a corner. And one more slab goes underneath."

The picture column for 09_2 adds two steps the voice never says: the step's slab "gains a column of 5" (the step goes from ten cubes to fifteen, `F-S.grown.top`), and the gold pieces regroup as "under blue, behind copper, beside pink", which is how the 5 by 6 slab and the Ls become the three 5 by 5 slabs of 09_3 (`F-S.grown.slab`). Recomputed (`30_stacks_box.py`): new cells per height 30, 10, 10, 10, 10, 5, which is 75 = 3 times 25; the four full slabs gain ten each (arms 5 and 4, corner 1) and the step gains five. As spoken, "every slab of the block, four by five" can be heard to include the step, which is not four by five and does not become five by six. The first sentence (a five by five slab at each base) also runs ahead of the picture, which shows it last.

Replacement for 09_2:
> Give each stack one more level, a five by five slab at its base. Every full slab of the block, four by five, becomes five by six, with an L-shaped layer of two arms and a corner. The step on top gains one more column. And one more slab goes underneath.

Replacement for 09_3 (so the regrouping is said where it is shown):
> Those gold pieces make three new slabs of five by five, one under, one behind and one beside, and nothing overlaps. Like the L-shaped layer, they are what you add to get the next size.

### 3. 03_6, MISLEADING, must fix
Spoken: "A picture that shows why, and says what it took for granted, is a proof."

Taken literally this is false by the video's own 14_7: that card says what the picture took for granted ("the long edge is straight") and is not a proof. What separates 03_6 from 14_7 is the tick, that the assumption is checked, which is what the card draws as a close-up with a tick. 15_3's three criteria say it correctly ("every fit has a reason, and a look is not a reason"); 03_6 is the first and most repeated statement of the definition and drops the check.

Replacement:
> A picture that shows why, and checks what it took for granted, is a proof.

### 4. 02_7, UNSOURCED, must fix
Spoken: "Such a reason is called a proof, and the best ones you can see." A superlative with no source, in the exact form rule 3 names ("the best ones"). Nothing in the fact sheet ranks proofs. (The first sentence, "Five checks in a row prove nothing about the sixth", is a protected phrase and holds for checks alone; see appendix B.)

Replacement:
> Such a reason is called a proof, and some of them you can see.

### 5. 13_1, UNSOURCED, must fix
Spoken: "One more, and maybe the most famous picture in mathematics." Hedged with "maybe" but still a superlative with no source; the storyboard note shows it is the owner's own opinion ("the owner finds it famous"), which is not an attribution a viewer can see.

Replacement:
> One more, and one you may already know.

### 6. 08_5, UNSOURCED, must fix
Spoken: "This picture is not ancient. Man-Keung Siu, of the University of Hong Kong, published it in nineteen eighty-four, under the title Proof Without Words."

Two issues. First, S8 supports "Siu published it in 1984" (Crossref and Nelsen p. 77); no row dates the picture itself, so "not ancient" is an age claim the sources do not carry (compare the fact sheet's own care with Page, S9: "no source was found that dates the halved square earlier"). Second, the title is "Proof without Words: Sum of Squares"; "Proof Without Words" alone is the series name. S8's Seen cell also lacks the "Partly verified" mark that S5 and S10 carry although no printed figure was looked at.

Replacement:
> Man-Keung Siu, of the University of Hong Kong, published this picture in nineteen eighty-four, under the title Proof Without Words, Sum of Squares.

### 7. 02_4, MISLEADING, must fix
Spoken: "Put points on a circle, spread unevenly, and join every pair with a straight line."

"Spread unevenly" is given as the condition for the doubling to break at thirty-one, and it is neither necessary nor sufficient (`10_circle.py`). Not sufficient: the video's own `F-C.three_through_one` set has angles 28.07, 53.13, 67.38, 208.07, 233.13 and 247.38 degrees, gaps 25.1, 14.3 and 140.7 twice, which is plainly uneven, and still gives thirty regions, not thirty-one. Not necessary: evenly spaced points give the generic count at n = 3, 4, 5, 7 and 9 (4, 8, 16, 57, 163) and break only at even n of six or more (30 at six, 88 at eight). The true condition is the one the 02_6 lower-third states: no three lines cross at one point inside the circle.

Replacement:
> Put points on a circle and join every pair with a straight line. Place the points so that no three lines cross at one place inside the circle. The lines cut the circle into pieces, and each piece is a region. One point, one region. Two points, two. Three points, four. Four points, eight. Five points, sixteen.

### 8. 06_2, MISMATCH, should fix
Spoken numbers all hold (arm six by fifteen, fifteen is two and a half sixes, five slabs, six). The picture column adds the payoff of the hook: "The square is 21 on a side" (`F-K.sixth.side`) and the question mark of 02_7 becomes "441 = 21 × 21" (225 + 216, `F-K.unit_squares`, `F-K.sixth.cube`). The voice stops at "Exactly one cube", so the payoff of the "five for five" setup plays silent.

Replacement tail:
> Five and one make six. Exactly one cube. The square is now twenty-one on a side. The sixth total is four hundred and forty-one, and that is twenty-one times twenty-one.

### 9. 12_7, MISMATCH, should fix
Spoken: "Every block of the cube appears twice ... One, four, six, four, one." The sentence "appears twice" holds (recomputed: the sixteen blocks equal the eight cube blocks times the long and the short fourth length). The picture column adds that each number is the sum of the two above it (`F-B.fourth.from_cube`, shown as Pascal's triangle rows) and the line 81 + 216 + 216 + 96 + 16 = 625 (`F-B.fourth.by_kind`, `F-B.fourth.total`). The voice says neither, though the neighbour sum is the reason the counts are what they are.

Replacement:
> And the fourth power? Every block of the cube appears twice, once with a long fourth length and once with a short one. So each middle count is the sum of two neighbours in the row above. One, four, six, four, one. And the blocks add up to six hundred and twenty-five, five to the fourth power. We can count its blocks, even though no one can build it in our space.

### 10. 14_7, MISLEADING, should fix
Spoken: "The trick goes back to Paul Curry, a New York magician, in nineteen fifty-three. Martin Gardner found triangle forms like this one and published them three years later."

S14 is Gardner's own sentence: "The inventor of this delightful paradox is Paul Curry, a New York City amateur magician. In 1953 he conceived the brilliant notion ..." The voice states Gardner's attribution as fact ("goes back to") and drops "amateur", which changes what the viewer pictures. The fact sheet's "never said" list is otherwise respected: 1953 is the idea, not a publication; "forms like this one" is hedged because Gardner's figures were not seen. "Neither picture was ever a triangle" holds (cross product of each bend against the corner-to-corner line is 1 and −1, not 0).

Replacement:
> Neither picture was ever a triangle. Martin Gardner says the trick began with Paul Curry, a New York amateur magician, in nineteen fifty-three. Gardner found triangle forms like this one and published them three years later.

### 11. 06_6, MISLEADING, should fix
Spoken: "The picture with the slabs cut in half was published in nineteen eighty-five."

"The picture ... was published" reads as its first appearance. S5 is marked "Partly verified": the printed page was not seen and the cut slabs rest on Blinder's description citing Fry. S6 lists earlier pictures of the same L-shaped layers (Golomb 1965, Love 1977); Golomb's has no halves (Gardner's description, a snippet) but Love's content was not seen. So the sources support "a version ... was published in 1985", not "the picture". Fry is named on screen, not in the voice; decision 13 already says "the slab picture Fry's (1985)".

The screen's date line also shows Nicomachus (about 100) and Aryabhata (499, "states the rule", S4) which the voice never says; either drop them from the line or say one of them.

Replacement:
> Al-Karaji proved the rule with L-shaped layers around the year one thousand. A version of the picture with the slabs cut in half was published by Alan Fry in nineteen eighty-five.

(If "Alan Fry" is not to be spoken, end with "was published in nineteen eighty-five" after "A version of".)

### 12. 12_6, EDGE (fails as worded), should fix
Spoken: "And nothing depended on three and two. Slide the cut, and it is still eight blocks."

Edge tried (`40_sums_quarters_cut.py`): (a + b)³ equals the sum of the eight blocks for every a and b from 0 to 8, but for a or b equal to 0 seven or eight of the blocks have no volume, so "still eight blocks" is false at the end of the slide. For a and b from 1 to 8, including a = b and a < b (where "long" and "short" swap names), it holds. The picture slides only between 4 and 1, so the screen is fine; the sentence is the one that overreaches. The protected phrase "nothing depended on three and two" stays.

Replacement:
> The blocks add up to one hundred and twenty-five. Five cubed. And nothing depended on three and two. Slide the cut anywhere inside the cube, and it is still eight blocks.

### 13. 14_3, MISLEADING, should fix
Spoken: "Same pieces, an outline that looks the same, and a tile has vanished."

Taken literally nothing vanished: the pieces total 12 + 5 + 8 + 7 = 32 in both arrangements (`F-L.pieces.total`), and every tile is still there. What changed is that the second outline encloses 33 (`F-L.outline.second`) and one tile of it is empty. 14_2 says this correctly ("now one tile is empty"); 14_3 then says the tile "vanished" while the screen reads "32 + 1 = 33".

Replacement:
> Same pieces, an outline that looks the same, and room for one more tile. If this were a proof, it would prove that thirty-two equals thirty-three.

### 14. 13_6, MISMATCH, should fix
Spoken: "It holds for every right triangle ... Change the shape, and both arrangements follow." The "every" holds (`50_pythagoras.py`: integer legs 1 to 100 each way, equal legs, 1 and 100, fractional legs, all exact; a leg of 0 is not a triangle). The picture column adds a step the voice skips: at legs 2 and 5 the readout is 4 + 25 = 29 (`F-P.other.tilted`), "no whole long side" (`F-P.other.whole_side`), "and the argument does not care". That is the scope test of the sentence being spoken.

Replacement:
> It holds for every right triangle. This is the theorem that carries Pythagoras's name. Even with shorter sides two and five, where the tilted square is twenty-nine and the long side is no whole number, both arrangements follow.

### 15. 07_2, MISLEADING, should fix
Spoken: "If the cubes that stick out on top may be sliced in half, what is the fewest number of these stacks, turned to fit, that make a solid box?"

At this beat the picture is "the stack and a ghost copy"; there is no block yet, so "the cubes that stick out on top" has no referent and can be heard as the single apex cube of the one stack. The answer three is right and is the fewest (note 3), but the rule the question sets cannot be read before the block exists.

Replacement:
> Two staircases made a rectangle. Your turn. Guess. You may slice cubes in half, but only the ones that end up sticking out on top. What is the fewest number of these stacks, turned to fit, that make a solid box?

### 16. 07_3, MISMATCH, should fix
Spoken: "Here are three copies of the stack, each turned to fit." In `model::stacks::TURNS` and in the picture column the first stack stays where it is; only the second and third turn (`F-S.turns` is the determinant of each turn, 1 for all three, the first being the identity).

Replacement:
> Three. Here are three copies of the stack, and the second and third are turned to fit.

### 17. 05_1, MISLEADING, low
Spoken: "A cube five tiles on a side is five slabs, each five by five. A cube of side four is four slabs. Fifteen slabs in all." Two cubes have been named, so "in all" reads as 5 + 4 = 9. Fifteen is 1 + 2 + 3 + 4 + 5 over all five cubes (`F-K.slabs`).

Replacement tail:
> A cube of side four is four slabs. Across all five cubes, that is fifteen slabs.

### 18. 13_7, UNSOURCED, low (on-screen string)
Lower-third: "Nobody knows who first drew this picture." A universal negative the sources cannot carry. S13 says only that Heath found "no positive evidence" for Pythagoras and that Loomis credits the figure to A. D. Wheeler, 1859, "the earliest printing found, not a claimed first". The spoken part of 13_7 is correct and matches the card step for step.

Replacement (on screen):
> No source we found says who first drew this picture. A close cousin is in a third-century Chinese commentary on the Zhoubi suanjing

### 19. 10_4 tag, UNSOURCED, low (on-screen string)
The "for programmers" tag says "in 64-bit floats this sum reaches exactly 1.0 after 54 pieces". The number has no fact key in `docs/fact-sheet.md`, `src/facts.rs` or `oracle.py`. Recomputed (`40_sums_quarters_cut.py`): in binary64, adding 2^−1, 2^−2, … left to right, the sum after 53 pieces is 0.9999999999999999 (exact) and after 54 pieces it is 1.0 (a tie that rounds to even). So 54 is right; it needs a key (for example `F-H.float.reaches_one`, value 54) in the video's facts, the oracle and the fact sheet, or the number should be dropped from the tag.

### 20. 14_6 lower-third, UNSOURCED, low (on-screen string)
Lower-third: "2, 3, 5, 8, 13: neighbours in the Fibonacci numbers. That is why the slopes are so close." "So close" is unquantified and "that is why" has no key. The true statement and its numbers (`60_lie.py`): 2 × 8 − 3 × 5 = 1, which is also why the sliver has area exactly one, and 2/5 − 3/8 = 1/40.

Replacement (on screen):
> 2, 3, 5, 8, 13: neighbours in the Fibonacci numbers, so 2 × 8 − 3 × 5 = 1 and the slopes 2/5 and 3/8 differ by only 1/40

### 21. 15_6, MISLEADING, low
Spoken: "A picture cannot check itself." The video's own cards check fits by close-up with a tick, and 14_5's lens exposes the bend, so a picture with a lens does check a fit. What the line means is the 15_3 point that looking is not checking.

Replacement:
> Looking at a picture is not checking it. But when you can say why every piece fits, it does what checking cases never does. It shows you why.

### 22. 11_5, wording, low (status stays EDGE holds)
Spoken: "Share the last corner three ways ..." In an endless process there is no last corner; the line means the corner left after however many cuts.

Replacement:
> Three colours, and each one gets a square of every size. Share the corner that is left three ways, and each colour, with its share of the corner, is exactly one third. And the corner shrinks below any size you can name.

### 23. 13_5, MISMATCH, low
Spoken: "Nine plus sixteen. Twenty-five. The same pile, counted two ways." The picture column shows both ways: "49 − 24 = 25, both ways" (`F-P.big`, `F-P.triangles`). The voice gives only the second way and then the reason ("neither did the big square"), never the numbers forty-nine and twenty-four.

Replacement (insert after the second sentence):
> Forty-nine for the big square, take away twenty-four for the four triangles, and twenty-five is left the other way too.

### 24. 03_5, MISMATCH, low
The screen's readout computes the hundredth layer, 199 (`F-O.hundredth`), and the square, 100 × 100 = 10,000 (`F-O.sum.hundred`). The voice stops at "one corner". Both numbers hold (recomputed).

Replacement tail:
> Ninety-nine rows, ninety-nine columns, one corner. That is one hundred and ninety-nine tiles, and the square is a hundred by a hundred, ten thousand.

### 25. 04_5, MISMATCH, low
The screen ends on the hundred-step staircase, 5,050 (`F-T.count.hundred`), and the formula in letters; the voice has neither. 5,050 recomputed.

Replacement tail (after "So the staircase is half of that rectangle."):
> A staircase of a hundred steps has five thousand and fifty tiles.

### 26. 11_6, UNSOURCED, low
Spoken: "over two thousand years ago". A spoken number with no fact key; S11 gives Heath's 1897 edition and the proposition but no date for Archimedes. It is true (he died in 212 BC, so more than 2,200 years). Add the date to S11 and an `F-X` key, or accept as common knowledge and say so in the fact sheet. The rest of the line holds: Proposition 23 is a finite sum with a remainder, "quarters of quarters like these" and "a picture of this sum" are as strong as S11 and the "partly verified" S10 allow.

### 27. 05_2, voice rule, low (status stays EDGE holds)
Spoken: "One slab fits the corner of the L." A single letter spoken, against "no single letters". Replacement: "One slab fits the corner of the layer." ("L-shaped" appears in eight lines and in `docs/storyboard.md` is accepted by the G2 checker; I did not flag it.)

---

## Notes that are not problems

**Note 1, protected phrases.** All 26 protected phrases marked "script" in `docs/protected.tsv` are present in `script.tsv` and every replacement above keeps the ones on its line. "no three lines" is marked "description" only; replacement 7 puts the idea in the script (as "no three lines cross"), which is consistent with its stated reason. `docs/protected.tsv` cites 06_5 for the al-Karaji line; it is 06_6.

**Note 2, 06_5.** "Nicomachus noticed that every cube is a run of odd numbers" is a protected phrase and consistent with the brief ("wrote that each cube is a run of consecutive odd numbers"). S2's Seen cell quotes the rule only as far as "the next three, the third". If the "and so on" is in the passage, quote it in S2; if not, the voice has generalised from three cubes. The verb "noticed" is not stronger than the source, and he is correctly not credited with the sum-of-cubes rule (S3).

**Note 3, 07_2 "the fewest".** The answer three holds, but the storyboard's reason ("one or two stacks have 30 or 60 cubes, under the 64 of the least box round one stack") is incomplete once halves are allowed: a box 4 by 4 by 3.5 has 56 cubes, under 60. The argument that works (`30_stacks_box.py`): a box with h full layers and one sliced half layer has volume W × D × (h + ½), and one stack's whole cubes have extents 4, 4 and 3 in some order. For 30 and 60 cubes there is no W, D, h meeting both (none at all), and for 90 there are two, one of them 4 by 5 by 4½. Also confirmed: three rotated stacks tile the block plus step by exact cover for n = 1 to 5 (unique for n of 3 or more), and 90 cubes have no whole-cube box with every side at least 4.

**Note 4, fact-sheet and storyboard drift (not spoken).**
1. Simplification-table quotes no longer match the script: 03_5 ("a hundred layers on" vs "all the way to the hundredth layer"), 06_4 ("whose side is the staircase up to that number" vs "Its side is one plus two and so on, up to that number"), 08_4 ("With any number of levels" vs "If this goes on working for any number of levels").
2. `F-X.al_karaji.after_nicomachus` (900) is described as "the 'nine centuries' of 06_5"; 06_5 no longer says nine centuries and the screen's date line shows no span. The key is orphaned.
3. The fact sheet row for 02_6 gives `F-C.three_through_one.regions` as the key for "six evenly spaced points give 30"; that key is a different (uneven) set with the same count. The evenly spaced claim rests on `F-C.evenly_spaced.regions`, printed by `oracle.py` only (the regular hexagon, by an affine map, gives 30 and 13 crossings). List it in the fact sheet and print it from `src/facts.rs`.
4. The storyboard header lists the card's pass at 10_5; the card is in 10_6's picture.
5. S8's Seen cell has no "Partly verified" although no printed figure was seen (see problem 6).
6. Numbers on screen with no key, all presentation parameters: the 12-column sweep (04_5), "eight times" lens (14_5), levels two to eight (11_2), the cut sliding to 4 and 1 (12_6), and the 10_4 tag (problem 19).

**Note 5, small things checked and left alone.** 02_3 "a worse problem" is the video's own comparison, argued at 02_4 to 02_6. 04_1 "adding them one at a time is slow" is the premise of the next beat. 15_1 "a famous theorem" is neither a superlative nor a comparison and 13_6 names the theorem. 07_5 "covers exactly half of it": "it" is the top face (10 of 20). 07_4 "the other two stacks take it" holds collectively; at the last handover the layer is one cube and goes to one stack. 01_2 puts the cut before the lay-flat while the picture splits the two slabs as they land; the order difference is not material.

---

## Appendix A: recomputation scripts (`claims_walk/`, all `python3 -I -B`)

| script | what it recomputes | result |
|---|---|---|
| `00_script_vs_storyboard.py` | script.tsv equals the storyboard words column; voice-rule scan | 89 ids equal; only "L-shaped" and one spoken "L" (05_2) |
| `01_quantifier_scan.py` | every spoken sentence with a quantifier, superlative, comparison, historical verb | list used for the walk |
| `10_circle.py` | regions for 1 to 7 drawn points by Euler on exact crossings and by a 900 by 900 sample; three-through-one set; regular n-gons | 1, 2, 4, 8, 16, 31, 57; smallest region of six 0.97 percent; uneven set gives 30; regular hexagon 30, octagon 88 |
| `20_layers_stairs_cubes.py` | chapters 01 to 06: layers, staircases, arms, slabs, odd runs | all as spoken; 'always' sentences hold to n = 300 to 5000 |
| `30_stacks_box.py` | three rotated stacks by exact cover; slice fills the box; grown block; fewest stacks | covers exist and are unique for n of 3 or more; 75 = 3 times 25 new cells; 1 and 2 stacks impossible |
| `40_sums_quarters_cut.py` | halves, quarters, cut square, cube, fourth power, binary64 sum | 10 and 20 pieces; 54 in floats; edge a = 0 |
| `50_pythagoras.py` | four triangles two ways, exact | holds for every pair tried |
| `60_lie.py` | missing square cell by cell with Fractions | 32 and 33, hole at (7, 1), sliver 1, thickest 1/8, 17 pieces fold in |
| `70_checks_edge.py` | the polynomial edge for 02_7 and 15_5 | degree-4 identity settled by five values plus a bound |
| `80_block_slabs.py` | 07_4 handover of the L from blue to the other stacks | 7, 5, 3, 1 cells; copper top arm, pink right arm and corner |
| `90_rows.py`, `95_protected.py` | the table below; protected phrases | 42 / 22 / 2 / 8 / 7 / 8; all present |

## Appendix B: always / never / every / any / nothing sentences and the edge tried

| line | sentence | edge tried | result |
|---|---|---|---|
| 01_3 | No gap, and nothing left over | 225 cells in 15 by 15, coverage count per cell; reach 40 | holds |
| 02_2 | Every total is a perfect square | n = 1 to 2000; n = 1 gives 1 | holds |
| 02_3 | Checking one case at a time never finishes | a finite family would finish; the family here is every n | holds in context |
| 02_7 | Five checks in a row prove nothing about the sixth | circle: 1, 2, 4, 8, 16 then 31; cubes: five checks plus a degree bound do prove it | holds for checks alone (the bound is the "reason") |
| 03_4 | one at the end of every row, one on top of every column; each layer is two longer | n = 1 (1 to 3 tiles) to 300 | holds |
| 03_5 | the same picture works all the way to the hundredth layer | layer 100 is 199 = 99 + 99 + 1; first 100 odd numbers 10,000 | holds |
| 03_6 | each arm is as long as the side it wraps, at every size | n = 1 (no arm; wraps the empty square) to 300 | holds |
| 04_4 | every pair comes to six | all five pairs at n = 5 | holds |
| 04_5 | works at any height; always a rectangle one taller than wide | n = 1 (1 by 2) to 300 | holds |
| 05_2 | the other must be cut in half | cube 2: arm 2 wide, 1 long; a 2 by 2 slab cannot lie in it | holds |
| 05_6 | the five cube wraps with no cut at all | arm 5 by 10 is two whole slabs | holds |
| 06_3 | the arms always take one slab fewer than the cube has | k = 1 (0 = 1 − 1) to 2000; k = 2 first cut | holds |
| 06_4 | every cube fits, at every size; any number | sum n = 1 to 5000; tiling to 40 | holds |
| 06_5 | every cube is a run of odd numbers | k = 1 to 200, each odd number used once | holds (note 2) |
| 07_4 | each time the blue square gives up an L | 7, 5, 3, 1 cells; last goes to one stack | holds collectively |
| 08_2 | slice every one in half | 10 cubes to 20 half cubes to 20 places | holds |
| 08_4 | any number of levels (after "if this goes on") | n = 1 (1 by 2 by 1½) to 30 filled; identity to 2000 | holds |
| 09_3 | nothing overlaps | n = 1 to 24 | holds |
| 09_4 | that covers every height | base n = 1; step n = 1 to 24 | holds |
| 10_4 | not after any number of steps | n = 0 to 300 | holds |
| 10_5 | gap halves every time; below any size you name; no number smaller than one is big enough; nothing ever passes one | n to 300; 1/m to 5000; 1 − 1/m to 3000 | holds |
| 10_6 | exactly what it means | positive pieces; "this endless sum" scopes it | holds |
| 11_4 | at every size there are three equal squares | levels 1 to 39 | holds |
| 11_5 | exactly one third at each stage; corner below any size | n = 1 to 300 | holds |
| 12_6 | nothing depended on three and two; still eight blocks | a, b = 1 to 8 incl. a = b, a < b; a = 0 | fails at a = 0 (problem 12) |
| 12_7 | every block appears twice; no one can build it | sixteen blocks equal two shifted copies of eight | holds |
| 13_3, 14_2 | nothing turns, nothing flips | slides are pure translations in both chapters | holds |
| 13_6 | every right triangle | legs to 100, equal, 1 and 100, fractions; leg 0 | holds (leg 0 is no triangle) |
| 14_7 | neither picture was ever a triangle | bend against corner-to-corner line: cross product 1 and −1 | holds |
| 15_3 | every piece accounted for; every fit has a reason | the lie passes the first and fails the second | holds as criteria |
| 15_4 | the reason covered every cube | arms are k − 1 half slabs, k to 2000 | holds |
| 15_5 | says nothing about the next size | polynomial edge; as evidence | fails (problem 1) |
| 15_6 | what checking cases never does | rhetorical; "cannot check itself" flagged | problem 21 |

## Appendix C: historical claims against the sources table

| claim (line) | row | verdict |
|---|---|---|
| Moser, 1949, thirty-one at six points (02_6 lower-third) | S1 | supported (OCR of the note); not claimed as proof or naming |
| Nicomachus noticed runs of odd numbers (06_5) | S2, S3 | supported, scope see note 2; sum-of-cubes rule not credited |
| Al-Karaji proved it with gnomons, about 1000 (06_6) | S3 | supported (Heath: "has a proof ... figure with gnomons"; tenth to eleventh century) |
| Aryabhata, 499, "states the rule" (06_6 screen) | S4 | supported; "first" is not said |
| Fry, 1985, slabs cut in half (06_6) | S5, S6 | date and author supported, partly verified; "the picture" overreaches (problem 11) |
| Siu, 1984, University of Hong Kong (08_5) | S8 | year and place supported; "not ancient" not (problem 6) |
| gnomon, Aristotle, Physics III.4 (03_5 lower-third) | S17, S3 | supported; the L-shape reading is Heath's, which the row says |
| Archimedes, Proposition 23, finite sum plus a third of the last piece (11_6) | S11 | supported; "over two thousand years ago" has no key (problem 26) |
| Ajose and Nelsen, 1994, a picture of this sum (11_6) | S10 | supported, partly verified, hedged by "a picture" |
| Euclid II.4 in words (12_2 lower-third) | S12 | supported; no letters claimed |
| Pythagoras's name; Zhoubi commentary, third century (13_6, 13_7) | S13 | supported; "Nobody knows" not (problem 18) |
| Curry 1953, Gardner 1956, "three years later" (14_7) | S14 | supported as Gardner's account; attribution and "amateur" (problem 10) |
| Nelsen 1993 and 2000 (15_6) | S15 | supported |
| halved square, nobody named (10 chapter) | S9 | consistent: nobody is named in the voice |

---

## (c) Every line id: status and the fact keys it rests on

Keys are `F-` keys of `docs/fact-sheet.md`; a note with "problem N" points to section (b).

| line | status | fact keys it rests on | what was tried or found |
|---|---|---|---|
| 01_1 | OK | F-K.cubes, F-K.slabs | five cubes of side 1 to 5; 1+2+3+4+5 = 15 slabs on screen later (05_1) |
| 01_2 | OK | F-K.cut_slabs, F-K.pieces, F-K.side | two cut slabs (cubes 2 and 4), 17 pieces, side 15; recomputed |
| 01_3 | EDGE (holds as worded) | F-K.unit_squares, F-K.side, F-K.tiled.through | 'no gap, nothing left over': 225 cells = 15 x 15, no cell covered twice; tried to cube 40 |
| 02_1 | OK | F-K.cubes, F-C.cube_totals | totals 1, 9, 36, 100, 225 on screen only |
| 02_2 | EDGE (holds as worded) | F-C.cube_totals, F-C.cube_roots | 'every total': the five shown, and n = 1 to 2000; roots 1, 3, 6, 10, 15 |
| 02_3 | EDGE (holds as worded) | (none; context F-C.regions) | 'never finishes': true for the infinite family of cube counts; 'a worse problem' is the video's own comparison, argued at 02_4 to 02_6 |
| 02_4 | MISLEADING | F-C.regions | counts 1, 2, 4, 8, 16 recomputed; 'spread unevenly' is neither necessary nor sufficient (problem 7) |
| 02_5 | OK | F-Q.regions.line, F-Q.regions.seconds |  |
| 02_6 | OK | F-Q.regions.answer, F-C.regions, F-C.doubling, F-C.three_through_one.regions, F-X.moser | 31 vs 32; lower-third 'six evenly spaced points give 30' checked (regular hexagon 30); key F-C.evenly_spaced.regions exists only in oracle.py |
| 02_7 | UNSOURCED | F-C.regions, F-C.cube_roots | 'the best ones you can see' is a superlative with no source (problem 4); 'prove nothing' holds for checks alone (polynomial edge in 70_checks_edge.py) |
| 03_1 | OK | F-O.layers | 2^2 - 1 = 3 |
| 03_2 | OK | F-O.layers, F-O.square | 5, 7, 9; 5 x 5 |
| 03_3 | OK | F-Q.next_layer.line, F-Q.next_layer.seconds, F-O.next |  |
| 03_4 | EDGE (holds as worded) | F-O.next, F-Q.next_layer.answer, F-O.tiled.through | 'every row/column', 'two longer': n = 1 to 300; 5 + 5 + 1 = 11 |
| 03_5 | MISMATCH | F-O.sum, F-O.hundredth, F-O.sum.hundred, F-O.sum.through | 99 + 99 + 1 = 199 and 100 x 100 = 10,000 are on screen and not said (problem 24); gnomon lower-third rests on S3 and S17 (Heath's reading) |
| 03_6 | MISLEADING | F-O.tiled.through, F-O.sum.through | 'says what it took for granted, is a proof' omits that the assumption is checked (problem 3); 'at every size' holds n = 1 to 300, n = 1 has no arms |
| 04_1 | OK | F-T.columns, F-T.counts, F-T.count | 1+2+3+4+5 = 15; 'slow' is the premise of the next beat, not a factual comparison |
| 04_2 | OK | (none) |  |
| 04_3 | OK | F-Q.rectangle.line, F-Q.rectangle.seconds |  |
| 04_4 | EDGE (holds as worded) | F-T.rectangle, F-T.rectangle.cells, F-T.count | every pair 1+5, 2+4, 3+3, 4+2, 5+1 = 6; 30 / 2 = 15 |
| 04_5 | MISMATCH | F-T.tiled.through, F-T.sum.through, F-T.count.hundred | 'always': n = 1 (1 by 2) to 300 holds; the 5,050 readout and the letters formula are on screen and not said (problem 25) |
| 04_6 | OK | F-K.side, F-T.count |  |
| 05_1 | MISLEADING | F-K.slabs | 'Fifteen slabs in all' follows only the five and the four cube; reads as 5 + 4 = 9 (problem 17) |
| 05_2 | EDGE (holds as worded) | F-K.cut_slabs, F-K.sides | 'must be cut': a 2 by 2 slab cannot lie in an arm 1 long; spoken 'the L' is a single letter (voice rule, problem 27) |
| 05_3 | OK | F-K.sides | arm 3 x 3 = one slab; side 6 |
| 05_4 | OK | F-K.arm.4, F-Q.cut_slabs.line, F-Q.cut_slabs.seconds |  |
| 05_5 | OK | F-K.arm.4.slabs, F-Q.cut_slabs.answer, F-K.sides | 6 / 4 = 3/2; side 10 |
| 05_6 | OK | F-K.arm.5, F-K.arm.5.slabs, F-K.side |  |
| 05_7 | EDGE (holds as worded) | F-K.unit_squares, F-K.layer_areas, F-K.side | 'the same 225 tiles': unit cubes 225 and unit squares 225, one to one through the slabs |
| 06_1 | OK | F-K.sixth.cube |  |
| 06_2 | MISMATCH | F-K.arm.6, F-K.arm.6.slabs, F-K.sixth.cube, F-K.sixth.side, F-K.unit_squares | numbers spoken all hold; side 21 and 441 = 21 x 21 (the payoff of 02_7) are on screen and not said (problem 8) |
| 06_3 | EDGE (holds as worded) | F-K.arm.7, F-K.arm.7.slabs, F-K.arm.8, F-K.arm.8.slabs, F-K.tiled.through | 'always one slab fewer': k = 1 (0 = 1 - 1) to 2000; the first cut is k = 2 |
| 06_4 | EDGE (holds as worded) | F-K.tiled.through, F-K.sum.through | n = 1 to 5000 exact; structural for all n |
| 06_5 | EDGE (holds as worded) | F-K.odd_runs, F-X.nicomachus | runs checked k = 1 to 200; protected phrase; S2 'Seen' quotes the rule only to 'the third' (note 2) |
| 06_6 | MISLEADING | F-X.al_karaji, F-X.fry, F-X.nicomachus, F-X.aryabhata | 'the picture ... was published in 1985' reads as first appearance; S5 partly verified, S6 (Love 1977) unseen (problem 11); screen adds Aryabhata and Nicomachus dates unsaid |
| 07_1 | OK | F-S.layers, F-S.stack | 16 + 9 + 4 + 1 = 30 |
| 07_2 | MISLEADING | F-Q.stacks.line, F-Q.stacks.seconds, F-S.stack | 'the cubes that stick out on top' has no referent on screen (problem 15); 'fewest' = 3 holds (note 3) |
| 07_3 | MISMATCH | F-Q.stacks.answer, F-S.turns | 'each turned to fit': the first stack stays put, only the second and third turn (problem 16) |
| 07_4 | EDGE (holds as worded) | F-S.top.layer, F-S.top, F-S.layers, F-S.stack | 'each time the blue square gives up an L': 7, 5, 3, 1 cells, taken by the other stacks; at the last time one cube goes to one stack |
| 07_5 | OK | F-S.block, F-S.top, F-S.top.layer | 'half of it' = half of the top face (10 of 20) |
| 08_1 | OK | F-S.top, F-S.top.layer |  |
| 08_2 | OK | F-S.top.halves, F-S.top, F-S.top.layer | empty half = step turned half way round (exact); 10 halves lift, 20 half cubes |
| 08_3 | OK | F-S.box, F-S.box.volume, F-S.three_stacks, F-S.stack |  |
| 08_4 | EDGE (holds as worded) | F-S.filled.through, F-S.sum.through, F-S.one.box, F-S.sums | 'any number of levels': n = 1 to 30 filled, n = 1 to 2000 volume identity; hedged by 'if this goes on working' |
| 08_5 | UNSOURCED | F-X.siu | 'This picture is not ancient' has no source; title is 'Sum of Squares', not 'Proof Without Words' alone (problem 6) |
| 09_1 | OK | F-S.layers, F-S.grown.box | four levels, five slabs |
| 09_2 | MISMATCH | F-S.grown.box, F-S.grown.top, F-S.grown.slab | screen: step gains a column of 5 and the gold regroups as under, behind, beside; voice says neither (problem 2) |
| 09_3 | EDGE (holds as worded) | F-S.grown.slab, F-S.grows.through | 'nothing overlaps': n = 1 to 24; 3 x 25 = 75 new cells |
| 09_4 | EDGE (holds as worded) | F-S.one.box, F-S.one.cubes, F-S.grows.through | base case 1 x 2 x 1.5; the step R(n) lifted sits in R(n+1) leaving 3(n+1)^2, n = 1 to 24 |
| 09_5 | OK | F-S.filled.through, F-S.grows.through, F-S.one.box |  |
| 10_1 | OK | F-H.pieces |  |
| 10_2 | OK | F-H.filled, F-H.gap, F-H.pieces | 31/32, 1/32 = last piece |
| 10_3 | OK | F-H.gap |  |
| 10_4 | UNSOURCED | F-H.pieces_to.thousandth, F-H.gap_at.thousandth, F-H.pieces_to.millionth, F-H.gap_at.millionth, F-H.checked.through | spoken numbers hold (10, 20); the tag '54 pieces' has no fact key (recomputed: 54 is right; problem 19); 'not after any number of steps': n = 0 to 300 |
| 10_5 | EDGE (holds as worded) | F-H.gap, F-H.checked.through | gap halves n = 1 to 300; 'below any size': 1/m, m to 5000; 'no number below one is big enough': 1 - 1/m, m to 3000 |
| 10_6 | EDGE (holds as worded) | F-H.checked.through | 'exactly what it means' holds for positive pieces (fact sheet row) |
| 11_1 | OK | (none) |  |
| 11_2 | OK | F-R.pieces |  |
| 11_3 | OK | F-Q.share.line, F-Q.share.seconds, F-R.pieces |  |
| 11_4 | EDGE (holds as worded) | F-Q.share.answer, F-R.shares, F-R.checked.through | three equal squares at every level, 1 to 39 exact |
| 11_5 | EDGE (holds as worded) | F-R.share, F-R.left, F-R.short, F-R.limit, F-R.checked.through | 'exactly one third' at every n = 1 to 300; 'the last corner' reads oddly in an endless process (problem 22) |
| 11_6 | UNSOURCED | F-X.ajose_nelsen | 'over two thousand years ago' has no fact key (problem 26; S11 gives no date; true, Archimedes d. 212 BC); S10 partly verified, wording 'a picture' is hedged enough |
| 12_1 | OK | F-B.parts |  |
| 12_2 | OK | F-B.square.each, F-B.square.total, F-X.euclid |  |
| 12_3 | OK | (none) |  |
| 12_4 | OK | F-Q.kinds.line, F-Q.kinds.seconds |  |
| 12_5 | OK | F-B.cube.blocks, F-B.cube.kinds, F-Q.kinds.answer |  |
| 12_6 | EDGE (fails as worded) | F-B.cube.each, F-B.cube.by_kind, F-B.cube.total, F-B.cube.power, F-B.filled.cuts | 'Slide the cut ... still eight blocks' fails at the end of the slide (a = 0 or b = 0: seven blocks have no thickness; problem 12); holds for a, b = 1 to 8 |
| 12_7 | MISMATCH | F-B.fourth.blocks, F-B.fourth.kinds, F-B.fourth.from_cube, F-B.fourth.by_kind, F-B.fourth.total | 'appears twice' holds; screen adds the neighbour-sum rule and 625 = 5^4, voice stops at the counts (problem 9) |
| 13_1 | UNSOURCED | F-P.legs, F-P.big | 'maybe the most famous picture in mathematics' has no source (problem 5); square 7 on a side is on screen only |
| 13_2 | OK | F-P.tilted |  |
| 13_3 | EDGE (holds as worded) | F-P.slides, F-P.squares | slides (0,0) (0,4) (-4,-3) (3,0): translations only; first stays; squares 9 and 16 |
| 13_4 | OK | F-Q.tilted.line, F-Q.tilted.seconds |  |
| 13_5 | MISMATCH | F-P.big, F-P.triangles, F-P.tilted, F-P.squares, F-P.long_side | 'counted two ways': screen shows 49 - 24 = 25 and 9 + 16, voice says only the second (problem 23) |
| 13_6 | MISMATCH | F-P.checked.through, F-P.other.legs, F-P.other.tilted, F-P.other.whole_side | 'every right triangle': legs 1 to 100, equal, 1 and 100, fractions: holds; screen's 2 and 5, 29, 'no whole long side' unsaid (problem 14) |
| 13_7 | UNSOURCED | F-P.checked.through | spoken steps hold and match the card; lower-third 'Nobody knows who first drew this picture' is a universal negative (problem 18) |
| 14_1 | OK | F-L.frame, F-L.pieces |  |
| 14_2 | EDGE (holds as worded) | F-L.slides, F-L.holes.first, F-L.holes.second, F-L.hole.at | 'nothing turns, nothing flips': four translations; hole at (7, 1) |
| 14_3 | MISLEADING | F-L.pieces, F-L.pieces.total, F-L.outline.first, F-L.outline.second | 'a tile has vanished': every piece is still there, 32 = 32; the second outline has room for 33 (problem 13) |
| 14_4 | OK | F-Q.slopes.line, F-Q.slopes.seconds, F-L.slopes |  |
| 14_5 | OK | F-L.slopes, F-L.slopes.thousandths, F-L.straight_slope, F-L.dip, F-Q.slopes.answer | 2/5 = 0.400 > 3/8 = 0.375; 3/8 < 5/13 < 2/5 |
| 14_6 | UNSOURCED | F-L.dip, F-L.bulge, F-L.sliver, F-L.sliver.pieces, F-L.sliver.fold, F-L.sliver.thickest, F-L.outline.first, F-L.outline.second, F-L.true_triangle | spoken words hold; lower-third 'That is why the slopes are so close' has no key (problem 20) |
| 14_7 | MISLEADING | F-X.curry, F-X.gardner, F-X.gardner.after_curry, F-L.bends.on_the_edge | S14 says 'amateur magician' and 'inventor' on Gardner's say-so; 'Neither picture was ever a triangle': cross products 1 and -1, holds (problem 10) |
| 15_1 | OK | (none) |  |
| 15_2 | OK | (none) |  |
| 15_3 | EDGE (holds as worded) | (none; the criteria are the card) | applied to ch 13 and ch 14: the lie passes 'accounted for' (12+5+8+7 = 32) and fails 'every fit has a reason' |
| 15_4 | EDGE (holds as worded) | F-K.tiled.through, F-K.sum.through | 'the reason covered every cube': k = 1 to 2000 arms |
| 15_5 | EDGE (fails as worded) | F-K.tiled.through, F-S.filled.through, F-K.sum.through | 'says nothing about the next size' fails at the polynomial edge and as evidence (problem 1); 40 and 24 hold; screen's 'every sum up to 1,000' unsaid |
| 15_6 | MISLEADING | F-X.nelsen | 'A picture cannot check itself' against the video's own close-ups and lens (problem 21) |
