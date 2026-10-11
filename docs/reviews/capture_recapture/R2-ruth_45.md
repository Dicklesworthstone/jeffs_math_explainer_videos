<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# R2 script review: *How to Count Fish You've Never Seen* (capture_recapture)

**Reviewer:** Ruth Adler, 45, with `ruth_audit.md`. I read the brief, `script.tsv` (it matches the packet byte for byte), the storyboard, the fact sheet, `docs/claims.md`, my R1 review and the R1 dispositions, all in full. I made no edits to the repo. Scratch work is in `<scratch>/`.

## Blocker first

**R1, at 13_1 and 13_8: chapter 13's rule is still wrong, and the wrong wording is mine from R1.**

The 13_1 "precisely" lower-third says "biased exactly when that share differs from the marked share of the lake you mean". That is false in both directions:
- **Deaths.** After 30% die, the second catch's marked share (10%) equals the share of the lake as it now is (700 of 7,000). Yet Mn/k aims at 10,000, not 7,000. I checked this in `oracle.py`'s `deaths()`, which catches 1,000 from about 7,000 survivors.
- **The ideal case.** The shares match, and Mn/K is still biased: the mean given K > 0 is 10,082.5, which is chapter 09's "a little high".

The 13_8 line "The estimate doesn't move … the rule still holds" then tells the viewer that the estimate survives deaths. It does not: it counts the lake as it was at marking, and 30% fewer fish now remain.

## Comprehension check

| ch | The sentence I'd say to a friend |
|---|---|
| 01 | One net scaled up gives 2,800 to 88,000 in the same lake, because fish clump. A net knows its area but not its share of the fish. Two marked catches instead fill the unseen cell with about 8,100. |
| 02 | The second net caught 100 of the 1,000 marked fish, so it searched about a tenth of the fish. That gives 1,000 × 1,000 / 100 = 10,000, the "soup guess", backed by E[K] = nM/N exactly. |
| 03 | Fish needn't spread evenly (they stay about 13× denser in the weeds). The marks only need to stop depending on where a fish swims. Equal catchability is a separate assumption, kept in the lower-third. |
| 04 | In a 20-fish lake, all 4,845 equally likely netfuls sorted by marked count give the hypergeometric law. Know the lake and you know the chances of every catch. |
| 05 | Fix our catch (3 of 10) and vary the lake size, and you get the score (likelihood). It peaks at 33 and sums to 50/3, not 1, because the lake size isn't random. |
| 06 | One landscape, two cuts. A fixed-size cut is a probability distribution (sums to 1). The fixed-count cut is the score curve (doesn't). |
| 07 | Comparing neighbouring sizes, the score rises exactly while Mn − kN = ab − k·u > 0: 529 vs 528 up, 576 vs 578 down. |
| 08 | The best dark cell balances the diagonals (k·u = a·b, odds ratio 1), which is the soup guess rounded down. Feller's numbers tie at 9,999 and 10,000. |
| 09 | In a known lake of 10,000, k is a bell (SD 9), but Mn/k is lopsided: 95% between about 8,475 and 12,048, averaging a little high (about 10,080). |
| 10 | Small experiments can catch zero marked fish (7.2% here). Chapman's +1 never divides by zero and is within 2% on average once you expect 4 or more caught twice. |
| 11 | Tiny lakes (1,900) and huge ones (10⁶) make our catch absurd. Even the best size gives exactly 100 only 4.4% of the time, which is normal, like 50 heads in 100 coins. |
| 12 | The careful range [8,400, 12,073] keeps every size that makes our catch unsurprising. The *procedure* covers the true size ≥ 95% of the time, and narrows mainly with more caught twice. The sentence is correct, but 12_3's "the lake moves" pushes viewers toward the wrong one (R2). |
| 13 | **Wrong as written:** "every failure breaks the share rule; deaths change nothing." Deaths leave the estimate counting the lake at marking time, not the lake you'd fish now. Blocker R1. |
| 14 | Laplace (28.4 million from an *assumed* million births) and two code reviewers (60, range 47–94) are the same table. Reviewers of different skill are harmless; bugs of different difficulty pull the estimate low. |
| 15 | Kosovo's four lists (4,400 named) gave an estimate of 10,356 (9,002–12,122). A later name-by-name count of 9,790 fell inside. Each example is a table with one unseen cell. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 4 | 5 | 4 | 4 |
| 02 | 4 | 4 | 4 | 4 | 4 |
| 03 | 4 | 3 | 4 | 4 | 4 |
| 04 | 5 | 5 | 4 | 4 | 4 |
| 05 | 5 | 5 | 5 | 4 | 5 |
| 06 | 4 | 4 | 4 | 4 | 5 |
| 07 | 4 | 5 | 4 | 3 | 4 |
| 08 | 4 | 4 | 4 | 4 | 5 |
| 09 | 5 | 5 | 4 | 4 | 4 |
| 10 | 4 | 5 | 4 | 4 | 4 |
| 11 | 4 | 5 | 4 | 4 | 4 |
| 12 | 4 | 3 | 4 | 4 | 5 |
| 13 | 4 | 2 | 5 | 4 | 4 |
| 14 | 4 | 4 | 4 | 4 | 4 |
| 15 | 4 | 4 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what misled or delighted me | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth | 13_1 lower-third, 13_1, 13_8 | **blocker** | The lower-third (my R1 wording) is false twice, as explained above: for deaths, and in the ideal case. The 13_8 voice "doesn't move … the rule still holds" lets viewers believe the estimate is still right after 30% die. It is right only for the lake *at marking*; about 7,000 fish remain. 13_6 (newcomers) counts the lake at the *second* catch, so the chapter uses two different targets and never says so. The 13_1 voice also lacks a quantifier: no real catch carries exactly the lake's share. | **Lower-third:** "precisely: Mn/k = M ÷ (marked share of the second catch). It aims at a lake of L fish exactly when that share is, on average, M/L. Even then it runs a little high (chapter 09)." **13_1:** "…The second catch must carry, on average, the same share of marked fish as the lake had when we marked it." **13_8:** "The estimate doesn't move. Death doesn't care about marks, so the share stays one in ten. But it still counts the lake as it was when we marked it." **Gauge:** add a second mark, "lake at the second catch: 7,000", mirroring 13_6. |
| R2 | ruth | 12_3 | major | "Here the catch is fixed and the lake moves" contradicts the video's own key line at 05_7, "The lake has one size". It invites exactly the misreading the chapter exists to prevent: "the lake is in [8,400, 12,073] with 95% probability". | "Here the catch is fixed, and we try every lake size." |
| R3 | ruth | R2 rule; notes for 01, 05, 09, 10, 12, 13, 14, 15 | major | The crate has no description draft and the packet has none. Yet at least nine exact versions and persona notes are routed to "the description": Feller's 93.6% (R-25, D3); the flat-prior line (R-30, "description at least"); the absorption identities and planning card (D4, D5, A-16); numpy's `inf` (D9); the full model parameters (R-07); "odds assume random communes" (R-14); the conditional mean 10,082.5 (R-23). Other notes reach nothing at all: 01's "many random quadrats work"; 05's crest/mode note (R-27); 12's delta method and split symmetry (R-32, "one line"); 13's Schnabel line; 14's Briand citation; Chapman 58.6 ± 8.4. | Write `docs/description.md` now with these items and add it to the R2 packet. Give every remaining note a destination or a "dropped: reason" line. Use the verified figures: flat prior on N ≥ 1,900 gives 8,536–12,186 (median 10,121). Feller's own p. 38 "These figures would justify a bet that the true number n of fish lies somewhere between 8500 and 12,000" belongs there, beside the 93.6% (3.47% / 2.89% tails). |
| R4 | ruth | 03_5, 03_6 | major | **03_5:** the trays show 10- and 20-fish nets sitting at 10%, tagged "illustration"; real ones hold 0–3 and 0–5 marked. **03_6:** the 50/5 and 10/1 squares were *selected* to match the voice (8 of 25 and 62 of 185 qualify), yet they are labelled "every fish". Both teach that small catches mirror the lake exactly, just before chapter 09 shows they don't. This breaks the video's own anti-cherry-picking rule (01 notes; D2). The trays are also the one remaining exception to the 15_3 receipts card. | Use the unselected regions (oracle keys `A-CH03.habitat.t400.*`): "Zoom in. The crowded weed corner holds about twenty-seven hundred fish, and two hundred seventy-one are marked. The empty north-east corner holds about two hundred, and twenty-three are marked. Both are about one in ten." Screen: all nine regions 9.3–10.7%. For 03_5, show real catches from the t = 400 frame at their true counts, with new keys. |
| R5 | ruth | 12_5 | minor | "Four times as many halves the range", but the slider shows ×0.44 then ×0.43. "A big share helps a little" is wrong at big shares, because √((1−M/N)(1−n/N)) goes to 0. "Leaves no doubt" overclaims: with M = n = k = 10,000, a lake of 10,001 still gives that catch with chance 1/10,001. | "Four times as many roughly halves it. Catching a bigger share of the lake helps more and more, and catching all of it shrinks the range to a single size." |
| R6 | ruth | 14_5, 14_6 | minor | "An honest range is forty-seven to ninety-four" states no level and gives the careful range a second name. It also assumes every bug is equally findable, which 14_6 says next is false, so "honest" overclaims. | "If every bug is equally easy to find, the careful range runs from forty-seven to ninety-four." |
| R7 | ruth | 02_4, 14_6 | minor | "probably caught about one in ten of all the fish" (n/N is fixed) and "read sixty as probably too low" put probabilities on fixed quantities, against the lesson of 05_7. | 02_4: "So guess that the second net caught about one in ten of all the fish." 14_6: "So expect sixty to undercount." |
| R8 | ruth | 08_4 | minor | "A straight line is exactly what two independent catches look like." Observed tables from independent catches are not straight (k has SD 9). The *expected* table is. | "…is what two independent catches look like on average." |
| R9 | ruth | 12_2 | minor | "rings the true size at least ninety-five times in a hundred" reads as a guarantee for each batch; a given 100 can show fewer. Also, the fact sheet tests coverage only at N = 10,000, though the voice says "whatever the true size is" (audit item 2). | "…in the long run, at least ninety-five times in a hundred…". Pin a key for minimum coverage over N; mine is 95.01% at N = 1,857, never below 95% from 1,000 to 10⁶. |
| R10 | ruth | 03_7–03_8 | minor | The voice equates mixing with independent catches ("The second net doesn't care which fish were caught before"). Bold fish (13_2) break that in a perfectly mixed lake. The lower-third keeps the exact version, so this passes the rule, but the voice overclaims. | 03_8: "…doesn't care which fish were caught before, as long as every fish is as easy to catch. That part comes later." |
| R11 | ruth | 01_7–01_8 | minor | The hook implies sampling by area is wrong. Many randomly placed nets, averaged, are unbiased: the mean of all 100 cells here is exactly 10,000. One net just can't measure its own share. This was Ruth's 01 note, and it reached nothing. | Lower-third at 01_8: "Many random nets, averaged, do work (here the 100 squares average exactly 10,000). One net can't tell what share of the fish it reached." |
| R12 | ruth | 01_3 | minor | "There are about eight thousand one hundred of them" states an estimate as a fact. | "Our best guess is eight thousand one hundred." |
| R13 | ruth | 02_1 | minor | "his book never mentions them" is true of the 1950 edition. The 1968 edition's footnote 13 reportedly cites Bailey and Chapman (UNVERIFIED). | "…and his first edition never mentions them." |
| R14 | ruth | 06_4 | minor | "Its heights add to sixteen and two thirds", but the slice on screen (17–60) sums to 9.89. | "Its heights, out to every lake size, add to sixteen and two thirds." |
| R15 | ruth | 14_4 lower-third | minor | "Mills, 1970 … (p. 73)" pairs the article's date with the reprint's page; the fact sheet asks for the 1972 IBM report date too. Also, Mills's marked fish were bugs planted on purpose, not reviewer 1's finds, which the quote's "calibration errors" shows. | "H. D. Mills (1970; IBM report 1972), repr. *Software Productivity*, p. 73: '…'. His marked fish were bugs planted on purpose." |
| R16 | ruth | 15_1 | minor | Scope. Ball et al. estimate Kosovar Albanians killed, 20 Mar–22 Jun 1999. Spagat (2014, p. 9, which I read) compares that with KMBD violent deaths, 20 Mar–20 Jun, and notes the estimate may run "a bit too high" from false reports. The voice follows Spagat. | Description: both scopes and periods, plus "comparison by Spagat (2014, p. 9)". |
| R17 | ruth | 10_3 | minor | The voice says "nearly exact on average" while the gauge shows Chapman at −6.5%. This lake expects only 2.5 caught twice, below the stated threshold of 4. | Add: "Here we expect only two and a half, so it still runs about six percent low." |
| R18 | ruth | 08_2 picture, 08 Ruth note | minor | The mosaic's orientation contradicts itself. The hero spec puts green on top. With green on top, the right divider *rises* toward the left as N grows, and it sits *below* the left while Mn − kN > 0. 08_2 ("falls toward") and the 08 note ("above … exactly when Mn − kN > 0") both assume green at the bottom. | Keep green on top and write "rises toward" and "below". |
| R19 | ruth | 01_5 picture | minor | A "live count" of drawn fish at 1 per 10 cannot show 451, which is not a multiple of 10. The claim walk raised this; it has not landed. | The counter reads the 10,000-fish lake's count (451). Drawn glyphs show about 45. |
| R20 | ruth | 09_6 code panel | minor | `k` names both the simulated array and the comprehension's dummy index. | Exact line: `… for j in range(n + 1)` with `comb(M, j) * comb(N - M, n - j)`. |
| R21 | ruth | 03 notes; 05_4 vs 05_6 | minor | The 03 note says "the left of the picture is labelled 'simulated fish density'", but 03_4 is now one panel. The score appears as 31.8% at 05_4 and as L ≈ 0.318 at 05_6. | Fix the note's location. Use percent at both beats. |
| D1 | ruth | 07_2–08_3 | delight | Mn − kN = ab − k·u, so the MLE is where the table's odds ratio crosses 1. The 07_6 check (49 − 48 = 1 = 529 − 528) is how I'd teach it. | Keep. |
| D2 | ruth | 08_5 | delight | The honest tie (9,999 and 10,000), with lgamma named as what breaks it in floats. | Keep. |
| D3 | ruth | 05_7 | delight | "The lake has one size", next to Feller's 1950 "meaningless" sentence. I re-checked it on p. 37 of the 1st-edition OCR. | Keep. Pair it with Feller's p. 38 "bet" in the description (R3). |
| D4 | ruth | 10_3 | delight | Chapman's +1 as one imaginary fish caught twice is the absorption identity in one sentence, and the e^(−Mn/N) bound makes "nearly exact" a theorem. | Keep. |
| D5 | ruth | 12_2 | delight | The ring toss uses the first 100 experiments, not a chosen window. I reproduced the misses at runs 40, 61 and 80, and 447 of 10,000. | Keep. Fixing R1 adds a second delight: newcomers mean you count the lake at the second catch, deaths mean you count it at the first. |

## Audit checklist (`ruth_audit.md`)

| item | result |
|---|---|
| Quantifiers explicit | fails: R1 (13_1), R8, R9 |
| Always/never tested at edges, test named in the fact sheet | fails: R9 (coverage across N not in the fact sheet), R5 ("no doubt"). Passes: "never divides by zero", "rounded down" (k = 0 and ties in the 08_5 lower-third), "add to one" |
| Every probability names its space | pass: 04_2 defines random; 05_4 is "for each lake size"; 10_1 and 12_2 are over repeated experiments; 11_2 is conditional on 1,900 |
| No probability on a fixed unknown; CI as coverage | fails: R2, R7. Passes: 05_6–05_7 and 12_2 |
| Standard terms | fails: R1 ("biased"), R10 ("independent") |
| Exact versions recorded | fails: R3 (description missing), R11; the 13_1 lower-third is wrong (R1). The other lower-thirds are correct; I checked each, see below |
| Historical claims verified | fails: R13, R15, R16. Pass: Feller 1950 pp. 37–38 quotes (re-read), Spagat p. 9 (re-read). Unchecked by me: Laplace text and p. 67, Mills p. 73, Ball p. 6 |
| Numbers equal the oracle; voice and screen round alike | **pass**: every spoken number recomputed, below |
| Edge cases | pass: k = 0, ties, log of zero, k ≤ 1 area. The mode tie (R-27) lives only in the notes (R3) |
| Notation consistent | fails: R18, R20, R21. Feller's n is glossed at 05_7 |
| Nothing true-but-misleading | fails: R1, R4, R5, R6, R11 |
| Persona notes reached a destination | fails: R3, R11, R21 (stale note) |

**Did the claim walk's fixes land?** Yes for:
- 11_2: the marker is at 324, "rounds to zero past here". Verified: 5.35e−324 is a float64, 5.35e−325 is 0.
- 12_5: the screen shows −16% / +21%, with ±9% labelled "typical error".
- 09_6: the panel has `from math import comb` and runs verbatim.
- 09_3: "exact SD 9.0", with 8.912 as the runs' own SD.
- The stale cross-references (03_8, 12_3, 02_8).
- 01_1 "A while later"; 02_1 "Others … for years"; 07_2 "five hundred twenty-nine"; "Douglas" dropped.
- 13_1's share wording, and 14_6's "probably too low", though both raise new issues (R1, R7).
- The receipts' unsourced items, now keyed or removed, except the 03_5 trays (R4).

Not landed: the 01_5 count (R19) and the Mills date (R15).

## What I recomputed, and how

- **The oracle**, with the given command: 451 keys in 60.7 s, compared with the fact sheet and storyboard.
- **`check_exact.py`** (`python3 -I`, exact `Fraction` and `math.comb`):
  - **Toy lake (chapter 04):** 4,845; 116,280; bins and exact fractions; 10 · 105.
  - **Toy likelihood (chapter 05):** L at 20 / 33 / 60 = 7.79 / 31.78 / 15.90%; argmax {33} over 17–5,000; sum over 17–60 = 9.891; running sum to N = 2,000 is 16.656.
  - **Likelihood area:** Mn/(k(k−1)) checked numerically on 5 cases (16.6666667 for the toy).
  - **Ratios:** 529/528 and 576/578; Feller's +100 / 0 / −100 with 80,982,001/80,981,901 and 81,018,001/81,018,101; the exact tie; L(10,000) = 0.0442892.
  - **Minimum lake:** L(1,900) = 5.3515 × 10⁻⁴³⁰, equal to (1000!)²/(100!·1900!).
  - **Law of K at N = 10,000:** SD 9.00045; P(91 ≤ K ≤ 109) = 70.9%; quantile k-values 83 / 118, giving 8,474.58 / 12,048.19; P(83 ≤ K ≤ 118) = 95.45%; mean given K > 0 = 10,082.50, SD 924.36; split symmetry 894.44 both ways.
  - **Small study:** P(K = 0) = 7.198% (1 in 13.89); Chapman 935.22 (−6.48%) and the closed form equal exactly; plain estimate +24.12%; 2,600 at k = 0.
  - **Chapman identity, iff and bound:** all 22,140 triples with N ≤ 40 hold, 0 failures, worst ratio to the bound 0.95.
  - **Other:** coin 7.959%; CI tails at 8,399 / 8,400 / 12,073 / 12,074; Feller's tails 3.468% / 2.887% (conf. 93.64%); reviewers [47, 94] and Chapman 762/13; toy [19, 138].
- **`check_cov.py`** (lgamma): coverage at 2,377 lake sizes from 1,000 to 10⁶, minimum 95.014%; the flat prior gives 8,536 / 10,121 / 12,186 and mean 10,183.68.
- **`check_mle.py`:** the 08_5 MLE and tie rule, and the ratio sign rule, on 1,240 (M, n, k) cases with 0 failures (538 ties). It also computes each failure model's target Mn/E[k]: deaths 10,000, newcomers 12,000, lost marks 12,500.
- **`lake.py`:** my own SplitMix64 rejection draw, from the fact sheet's spec only. It reproduces A-F27 exactly: 139,559 candidates, both check sums, 880 / 28 / 451, 31.43×, median 3,700, 22 cells within 2×.
- **`panel.py`:** the 09_6 panel run verbatim under uv's numpy. It gives mean 10,080.002, 8,474.58 / 10,000 / 12,048.19, sim SD 8.912, Σp = 1, p[100] = 0.0442892, and ring-toss misses at runs 40, 61, 80 (447 in all).
- **Failure models, approximately checked:** by expectation, by hand. Bold fish come to about 7,760 against the 7,752 median; trap-happy about 7,000 / trap-shy about 14,500 (the medians are 7,143 and 14,493); two halves give 160 / 40 caught twice, so 6,250 and 25,000 exactly.
- **Sources:**
  - Spagat 2014, pp. 7–9, read in the PDF.
  - Feller 1950, pp. 37–38 (archive.org OCR), through a summarizing fetch, not my own grep: the quotes for the 02_1 and 05_7 cards, the bet sentence, and the one-locality caveat.

**Unchecked:**
- The seeded A-F22 medians as printed (I read the models but did not replay them), all A-CH03 dynamics values, and the A-CH14 simulations. The expected counts 30 / 24 / 17.4 and 1200/29 I did check by hand.
- Laplace's text and p. 67, Mills p. 73, Ball et al. p. 6, and the 3rd edition of Feller.
- The `facts.rs` pins (nothing compiled).
- Anything in a description, since none exists.
