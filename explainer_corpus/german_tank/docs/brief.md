# Brief — How Much Can Five Serial Numbers Reveal?

Values I chose without the owner are marked `(default)`. Chapter numbers are
the 2026-10-24 sixteen-chapter structure (the owner's direction to set the
stage before any machinery); fact group `G-Fn` belongs to chapter `n`.

| Field | Value |
|---|---|
| Topic | catalog `german_tank`, "How many machines were built, from a handful of serial numbers?"; third-wave lane Bead `mx-video-program-program-87do.12.1`; working title "How Much Can Five Serial Numbers Reveal?" (assignment, 2026-10-24) |
| One-sentence takeaway | If things are numbered 1, 2, 3, … with no skips and every handful of five is equally likely to be the one you see, the largest number is only a lower bound, and adding one average gap to it gives a recipe that is right on average for every possible count, because the gaps can trade places; among recipes with that property none misses by less, yet one estimate can still be far off, and a handful that wasn't drawn fairly breaks the recipe. |
| The promise | Five friends want to know whether their board games are rare. Before any recipe, the viewer reads five numbered boxes, finds the one certain thing, at least sixty, and meets the unknown tail above it (01); computed fair handfuls from a thousand and from seventy show that the numbers' spacing is a clue they can't turn into proof (02); the gaps are counted one at a time and the hidden top gap is guessed from the others: about seventy-one, with "if the handful is fair" on screen when the recipe first claims anything (03). There is no first-minute deadline (owner direction, Mail 6777). No war anecdote opens the video; the name and its history get one line in chapter 16, pending a primary source |
| The hook (where the obvious answer fails) | "The largest number seen" is the tempting answer and it is always too low or exactly right, never too high; "double the average" is the other tempting answer, and it can land below a box someone is holding (`G-F10.silly_doubling` 28.6 for a handful containing 60) |
| The one picture | The line: one faint dot for every number the company could have printed, the friends' five boxes sitting on their own numbers with gold ticks, the unseen stretches between them as blue arcs with their counts, and one dashed bracket above the largest that nobody can see. The boxes keep their identity from the first shot to the last |
| Worked example | Boxes 14, 27, 33, 51, 60. Gaps 13, 12, 5, 17, 8 hold 55 boxes, 11 a gap, estimate 60 + 11 = 71 (`G-F3`). The same recipe as a formula: m(1 + 1/k) − 1 = 60 · 6/5 − 1 = 71 |
| Computed experiments | SplitMix64 from seed 20261024, one stream per experiment, each box drawn uniformly by rejection and a repeat redrawn; the rows shown are the first drawn, never chosen, and `src/model.rs` and `oracle.py` implement the generator separately: ten fair handfuls from 1,000 and ten from 70 (`G-F2`), twenty repeat handfuls from 71 (`G-F8.again_*`). Every spoken claim about them is an exact fact, not a sample statistic |
| Tiny example (the proof's picture) | A company of five boxes, two seen: ten pairs, ten estimates from 2 to 6.5, averaging exactly 5 (`G-F5`); the ten gap lists are every way to split the three unseen boxes into three gaps, once each (`G-F6.one_pair_per_list`) |
| Model (exact) | Boxes numbered 1 to N without skips, N fixed and unknown; a handful is k different boxes; every k-box handful is equally likely (a uniform draw without replacement). "Right on average" means unbiased: the estimates average to N over all handfuls, for every N ≥ k |
| Three ideas kept apart | **unbiased** is a property of the recipe over all handfuls (05 to 07); **uncertainty** is how far one estimate can land and which counts remain plausible (08 and 09: the 71-box company's estimates run from 5 to 84.2, the average miss is about 9, and the one-sided 95% range for our handful is 60 to 107, `G-F9.range`); **the one realised estimate**, 71, is a number, not a count and not a promise (03_10, 08_1, 16) |
| Counterexamples (biased sampling) | The friends' own five boxes, bought from a shop that only ever got the first 71 of the 200 made: the recipe still says 71 and averages exactly 71 over every handful that shop could sell, short by 129 (`G-F14.shop_average`, `G-F14.shop_short_by`). Shelves holding only boxes 151 to 200: it averages 230 (`G-F14.late_average`), and a first gap of 157 gives the draw away (`G-F14.late_first_gap`). Equal chances per box without equally likely handfuls (one of two deliveries, boxes 1–5 or 6–10, each box seen half the time): it averages 8, not 10 (`G-F15.deliveries_average`) |
| Audience | a curious 14-year-old with school arithmetic; a programmer who could simulate it afterwards; a statistician who finds nothing wrong (default, plan §1.2) |
| Length | By `tools/storyboard.py`'s estimate (spoken words at 2.68 words a second plus breaths, prediction pauses and holds): chapters 01 to 13, the production storyboard, 29.4 min (74.8, 84.6, 67.3, 84.7, 77.4, 136.4, 166.0, 135.7, 142.3, 149.2, 142.8, 244.6 and 256.6 s); chapters 14 to 16 (`docs/storyboard-draft.md`) 3.7 min; about 33.1 min in all. Chapters 06 to 13 are long because the swaps, each transformed line, the distribution of estimates, the coverage experiment, the two recipes' widths, the pairs behind sufficiency, the regrouped pairs with their misses as arrows, and the forced staircase with its three recipes compared are given time to be inspected (root's 06/07, 08/09, 10/11 and 12/13 assignments set no runtime cap; the 12/13 assignment says not to shorten motivation or reading time to meet the old draft duration). These are advisory script estimates, not measured narration; authored pauses are not in the estimate. Root's separately bound silent native previews have their own measured durations in the third-wave review records. No runtime target was set; the owner's direction allows a longer film |
| Scope in source | Root's scope correction (Mail 6790): only chapters with a native scene are in `docs/storyboard.md`, `script.tsv` and the chapter registry, now chapters 01 to 13 (02 and 03 integrated by root, 04 and 05 in the tree from `docs/integration-04-05.patch`; 06 and 07 from `docs/integration-06-07.patch` and 08 and 09 from `docs/integration-08-09.patch`, both integrated by root; 10 and 11 proposed in `docs/integration-10-11.patch`, and 12 and 13 in `docs/integration-12-13.patch`, which applies after it). Chapters 14 to 16 are complete words and pictures in the draft; root assigns their scenes |
| Format, voice, look | the program defaults (plan §1; AGENTS.md): 3840×2160 at 60 fps; FrankenTTS under the current narration direction, after root's qualification; 3Blue1Brown look. This lane made no audio |
| Protected facts | `docs/protected.tsv`: the phrases of chapters 01 to 13, the chapters in `script.tsv`; a drafted chapter's phrases join when it moves into `storyboard.md` |
| Oracle | `oracle.py`: brute-force enumeration (all 13,019,909 handfuls of five from 71), sequential-draw products where enumeration is too large, a triangular solve for the forced unbiased recipe, edge checks of every always/never sentence, negative controls; the Rust model (`src/model.rs`) uses closed forms instead |
| Out of scope | Bayesian estimates and priors; sampling with replacement; several factories or several numbering series; serial-number codes that skip or encode dates; confidence intervals beyond the one-sided 95% range; the variance formula; any claim about real production figures |
| Evidence boundary | Selection data (`docs/video-program/third-wave-20261024/MOR_CASE.md` and the catalog) supports editorial hypotheses about what may hold attention; it is not a forecast of views and not evidence that this topic causes any outcome |

## Sources

What was actually read, and what is known only from search results.

**Read.** George Clark, Alex Gonye and Steven J. Miller, "Lessons from the
German Tank Problem", arXiv:2101.08162v2 (dated 22 January 2021; the arXiv
listing says it was to appear in *The Mathematical Intelligencer*). Pages 1–4,
10 and 16 were read from the arXiv PDF on 2026-10-24 UTC; the PDF is not kept
in this repository.

- p. 1 (abstract) and p. 3: the model — tanks "labeled consecutively starting
  at 1", k serial numbers observed, maximum m — and the estimate
  N̂ = m(1 + 1/k) − 1. This is the formula the video derives as "the largest
  plus one average gap"; `src/facts.rs` pins `G-F3.estimate` 71 to it.
- p. 3: with the smallest value unknown and s the spread between the smallest
  and largest observed numbers, N̂ = s(1 + 2/(k − 1)) − 1, derived in their
  §3. Chapter 15's spread recipe is the same quantity (46 · 3/2 − 1 = 68;
  `G-F15.spread_estimate`), and `oracle.py` checks its unbiasedness by
  enumeration for three different starting numbers.
- p. 10, Figure 2: the often-quoted wartime table (169 / 1,000 / 122 and two
  more rows) is credited there to a WikiEducator page (their reference [6],
  p. 16). **This video quotes no historical figure.**
- p. 1, footnote: two talk videos by the third author (YouTube
  `quV-MCB8Ozs`, `I3ngtIYjw3w`). Not watched.

**Additional primary-source check, 2026-10-26.** Read §3, including §3.2,
pp. 7–10 of the authors' [21 January 2021 manuscript](https://web.williams.edu/Mathematics/sjmiller/public_html/math/papers/GTPv10.pdf).
It defines the population count as N = N₂ − N₁ + 1 and the sample spread
as S = maximum − minimum. For a uniform sample without replacement from
consecutive labels and k > 1, it derives E[S] = (N + 1)(k − 1)/(k + 1).
Thus the displayed spread recipe is unbiased for the count, not the last
label. This qualifies the Section 3 attribution in chapter 15; it does not
retroactively extend the earlier arXiv reading or establish a new audible result.
The card retains its assumptions and count warning; the bibliographic citation
is carried here and in the storyboard notes instead of on the card (F7).

**Not read (bibliographic or secondary only).**

- Leo A. Goodman, "Serial Number Analysis", *J. Amer. Statist. Assoc.* 47
  (1952) 622–634, DOI 10.1080/01621459.1952.10483442: known from MathWorld's
  bibliography via a search result. A search snippet of Mark P. Becker, "A
  Conversation with Leo Goodman", *Statistical Science* 24 (2009) 361–385,
  DOI 10.1214/08-STS276, arXiv:1010.0310, says Goodman introduced "the most
  efficient unbiased estimator"; the interview's text was not opened (only its
  arXiv abstract page, which says nothing about serial numbers). Nothing in
  the video is attributed to Goodman.
- Richard Ruggles and Henry Brodie, "An Empirical Approach to Economic
  Intelligence in World War II", *J. Amer. Statist. Assoc.* 42 (1947) 72–91:
  first known from its title and from secondary descriptions (a Wikipedia
  biography, a blog post), because the publisher's page returned HTTP 403.
  Root later obtained, through a web search, the publisher's indexed abstract
  (https://www.tandfonline.com/doi/abs/10.1080/01621459.1947.10501915). It
  states that in early 1943 the Economic Warfare Division of the American
  Embassy in London used markings and serial numbers on captured German
  equipment to estimate production and strength, first tyres and later tanks
  and other weapons. **That abstract supports beat 16_1, the one historical
  sentence, in its narrow form and no further.** Opening the page directly
  still returns 403, and the full paper has not been read. The page range is
  from the secondary descriptions. No production or strength figure is used.
  The abstract names an embassy division, not a profession, so 16_1 says
  "Allied analysts" (root's choice; it said "Allied statisticians" before),
  and 16_2 opens chapter 16 cleanly if 16_1 is cut.
- The optimality results (sufficiency of the largest number, Rao–Blackwell,
  Lehmann–Scheffé) are standard textbook theorems named only on the
  "precisely" cards; no textbook was read for this draft. The script proves
  each instance it uses on its own tiny example, and `oracle.py` checks them
  by enumeration (the unique unbiased recipe of the largest number, solved
  count by count; seven unbiased rivals that all miss at least as much, plain
  and squared, for k = 2, 3, 5 and N up to 13).

## Competing treatments (bounded check)

Five web searches and three page fetches on 2026-10-24 UTC; YouTube's own
search was not used. **No video was watched**, so nothing below is a judgment
of any video's content, quality or pacing.

| Treatment | What is known | Source of that knowledge |
|---|---|---|
| Numberphile, "The Clever Way to Count Tanks" (James Grime) | a video page exists; a third-party listing gives 2024-07-31 and 17 minutes; a summary says it covers the wartime story and m(1 + 1/k) − 1 | search results only (numberphile.com, thetvdb.com); not watched |
| U.S. Census Bureau, Statistics in Schools, "Panzers" (2016) | a classroom video page about the wartime tank estimate | search result only; not watched |
| Steven J. Miller's talks | MAA 2019 slides link a YouTube talk; the paper's footnote links two more | search results and the paper's footnote; not watched |
| Blogs (Futility Closet 2016, R-bloggers, Geraint Lewis) | written treatments that open with tanks and serial numbers | search result snippets only |

Differentiation hypothesis, untested against any watched video: this version
opens on products and why their count matters, not on war; lets the viewer
find the lower bound and the unknown tail before any recipe; shows computed
fair handfuls before defining anything by formula; earns the correction from
the gaps, counted one at a time; proves unbiasedness on a table small enough
to see whole; keeps unbiased, uncertainty and the one estimate apart; and
breaks the assumption with the same five boxes. Whether a close animated
treatment already does this is **unknown**.

## Pronunciation risks (for root's qualification; nothing was synthesized or heard)

- Number words in quick runs: "fourteen, twenty-seven, thirty-three,
  fifty-one and sixty" (01_4, where each box turns over on its own number);
  "thirteen … twelve more … then five, then seventeen, then eight"
  (03_1 to 03_3). Risk: run-together hyphenated numbers, "fourteen" versus
  "forty".
- Compound numbers whose joins carry the value: "a hundred and fifty" (02_8),
  "fifty-five" (03_4) and "seventy-one" (03_8, 03_10); "thirty-seven",
  "thirty-six" and "seventy-three" in one sentence (07_9), where thirty-six
  and thirty-seven must stay distinct. Risk: a dropped "and" or a broken
  hyphen heard as two numbers.
- Decimals and halves: "about forty-five and eighty-four point two" (08_3),
  "about eleven point seven" (08_8) and the other point values in 10_6, 10_9,
  12_13, 12_15, 13_11 and 13_13; "five plus three
  halves, six and a half" (05_4); "and a half" again in 12_6, 12_8, 12_12,
  13_4, 13_6, 13_10 and 15_3.
- Chapters 12 and 13 put close values side by side, and each pair is the
  claim: the average misses "one point four" and "one point two" (12_13,
  13_11) and "two point four" (13_11); the standard deviations "about one
  point seven" and "exactly one point five" (12_15); "about eight point
  seven" against "nine point two" (13_13); "sixty-nine", two short of
  "seventy-one" (13_13). Fractions: "four and a quarter" (12_10), "two and a
  quarter" and "three quarters" (12_15), "seven sixths" (13_13, 13_14) and
  "four and five sixths" (13_14), where "sixths" blurred into "six" changes
  the recipe. Signed and listed values: "minus one, zero and plus one"
  (12_11), whose signs carry the meaning; "zero, one, two and three"
  (12_12); "five, six, seven and eight" (12_5); "Three and four average
  three and a half. Four, five and six average five." (12_6); the staircase
  "two, three and a half, five, and six and a half" (13_10), where "three
  and a half, five" must not merge into one number.
- Lists of small numbers that name boxes, not a count: "one, two, three,
  four and five" (04_6) and "one, three and four" (05_3); the gap lists
  "zero, zero and three" (06_2, 06_5), "one, two and zero" (06_2, 06_3) and
  "three, zero and zero" (06_5), where "zero" must not become "oh". Risk: a
  list read as one number or with a flat, rushed rhythm. "Five hundred"
  (04_1) and "over thirteen million" (05_1, 08_4) must keep their joins.
- Large and close numbers in chapters 08 and 09: "over nine hundred
  thousand" and "sixty-seven stacks" (08_4); "a hundred and seven" and "a
  hundred and eight" (09_3, 09_4, 09_8), one count apart and on either side
  of the cutoff, so a dropped "and" or a blurred "seven"/"eight" changes the
  claim; "ninety-six times in a hundred" and "ninety-five" (09_6, 09_7);
  "sixty through seventy-nine" (09_10).
- A read-back paced by the picture: "skip one box, take the next, skip two,
  take the next, and stop" (06_3). Each step fills one square, so a
  run-together reading leaves the squares behind the words.
- Grouping that carries meaning: "the largest, plus the largest minus five,
  over five" and "sixty plus fifty-five over five" (04_5), "half of the
  largest minus two" (05_3), "N minus five, over six" (07_3, 07_6) and "the
  largest minus the size, over the size" (07_10), "the handful size over the
  count" (13_7), "three times the smaller number, minus one" (13_9) and
  "seven sixths of the largest number, minus one" (13_13), where the one
  comes off after the multiplying; the pause rows in
  `docs/pauses.tsv` mark where the grouping breath belongs. "N" is spoken as
  the letter in chapter 07 only (07_1 to 07_3, 07_6).
- Terms: "estimator" (said once, 04_3), "unbiased" (05_7, 07_7, 08_9, 10_4,
  12_2, 12_9, 13_1, 13_2, 13_8 to 13_12; stress on the second syllable),
  "standard deviation" (08_8, 10_9, 12_15), "confidence" (09_7),
  "sufficient" (once, 11_6), "Statisticians" (04_3, 08_8, 11_6), "analysts"
  (16_1, drafted).
- Card-only names, never voiced: Rao–Blackwell, Lehmann–Scheffé, Ruggles,
  Brodie, Goodman.

## Assumptions

1. Numbering starts at 1 and never skips (chapter 15 relaxes the start).
2. The five boxes are different boxes (no box is counted twice).
3. Every handful of five is equally likely; this is the assumption chapters
   14 and 15 break.
4. "Average miss" is the mean absolute error over handfuls; the optimality
   claim holds for it and for the mean squared error (any convex loss), and is
   checked for both.
5. The 95% range is one-sided: the counts from the largest number up to the
   largest count under which a handful topped by 60 or less still has at
   least a one-in-twenty chance (`G-F9.range` 60,107; coverage at least 95%
   checked for every count from 5 to 150).
