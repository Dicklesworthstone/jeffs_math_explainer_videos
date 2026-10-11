<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

**Reviewer: Dev, 28. Round R2 (script), capture_recapture**

I read the brief, the script, the storyboard and the fact sheet in full. I recomputed every spoken number and ran the 09_6 panel exactly as written. The method and results are in "What I checked" at the end.

### Comprehension check

| ch | What I'd tell a colleague |
|---|---|
| 01 | One net dip times the area gives answers from 2,800 to 88,000 (31× apart), because a net knows how much water it covered but not what share of the fish it caught. |
| 02 | The marks calibrate the net: 100 of the 1,000 marked fish came back, so the net caught about a tenth of all fish, and 1,000 is a tenth of 10,000. So N̂ = Mn/k, the "soup guess". |
| 03 | It only works because the marks mix. Fish density can stay clumpy, but once being marked is independent of position, any net's catch has the lake's marked share. Equal catchability is a separate assumption. |
| 04 | With the lake known, counting netfuls (4,845; split by marked count as C(M,k)·C(N−M,n−k)) gives the exact distribution of k, the hypergeometric. That is the forward direction only. |
| 05 | Fix the observed k and score each lake size by P_N(k). The score peaks at 33, which is the soup guess rounded down, and it is not a distribution over N (it sums to 16⅔). |
| 06 | One grid, two readings: a row at fixed lake size is a probability distribution that sums to 1, and a column at k = 3 is the score curve, which sums to 16⅔. |
| 07 | The score keeps rising while Mn − kN > 0, which in table cells is ab − k·u > 0. I could only say *why* (the neighbouring-score ratio) because of the 07_1 lower-third, not the voice. See dev-04. |
| 08 | The climb stops when ab = k·u, meaning both columns show the same second-catch share (independence). So the best guess is ⌊Mn/k⌋, with a tie at 9,999 and 10,000 when Mn/k is whole. |
| 09 | At a known lake of 10,000, k forms a bell around 100 (SD 9), but dividing by k skews the estimate: 95% fall in 8,475–12,048 and the mean is about 10,080, slightly high. |
| 10 | With small catches k = 0 happens 7.2% of the time. Chapman's +1/−1 removes the divide-by-zero and is nearly unbiased once you expect 4 or more caught twice. |
| 11 | Absurd lake sizes make 100 caught twice impossibly rare. Even the best size gives that exact count only 4.4%, which is normal (50 heads in 100 flips is 8%). |
| 12 | Keep every lake size whose middle 95% of k contains 100. That gives [8,400, 12,073], the procedure covers the truth at least 95% of the time, and the width shrinks roughly like 1/√k. |
| 13 | Every bias comes from one rule breaking: the second catch's marked share differs from the lake's. Bold or trap-happy fish push the estimate low; trap-shy fish and lost marks push it high; newcomers make you count the later lake. Here I'd also say "deaths do nothing", which is incomplete in a way that matters (dev-02). |
| 14 | Same table elsewhere. Laplace: births give 28.4 M people in France. Code review: 30, 24 and 12 shared give 60 bugs, so 18 hidden, range 47–94. Bugs of uneven difficulty bias it low. |
| 15 | Kosovo: lists naming about 4,400 unique victims gave an estimate of about 10,400 killed (9,002–12,122), and the later name-by-name count of 9,790 landed inside. Every case is a table with one unseen cell. |

### Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 4 |
| 02 | 5 | 4 | 5 | 4 | 5 |
| 03 | 4 | 4 | 4 | 3 | 4 |
| 04 | 5 | 5 | 4 | 4 | 4 |
| 05 | 5 | 5 | 5 | 4 | 4 |
| 06 | 4 | 5 | 3 | 4 | 4 |
| 07 | 3 | 5 | 4 | 3 | 4 |
| 08 | 4 | 5 | 4 | 4 | 5 |
| 09 | 4 | 3 | 5 | 4 | 4 |
| 10 | 4 | 4 | 5 | 4 | 4 |
| 11 | 4 | 5 | 4 | 4 | 5 |
| 12 | 5 | 5 | 5 | 4 | 5 |
| 13 | 4 | 3 | 5 | 4 | 4 |
| 14 | 4 | 4 | 5 | 4 | 5 |
| 15 | 4 | 4 | 4 | 4 | 3 |

### Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| dev-01 | Dev | 09_1, 09_6 (and the 12_2 keys) | major | The voice says "Build a lake of exactly ten thousand fish, and run Feller's experiment", then "Always check a simulation against something you can compute exactly". But the panel never nets a fish. `rng.hypergeometric` samples *from* the hypergeometric, and the exact line is the hypergeometric pmf, so the check only tests numpy's sampler against its own formula. It never tests chapter 04's claim that netting fish produces that law. The panel also never performs the check: `p` is computed and then not used. In code review I'd call this a test that asserts its own fixture. | Show the experiment. Setup line (greyed): `marked = np.arange(N) < M`. Simulation: `k = np.array([marked[rng.choice(N, n, replace=False)].sum() for _ in range(10_000)])` and `est = M * n / k`. Then the check line: `print(est.mean(), sum(M * n / j * p[j] for j in range(1, n + 1)))`. My scratch run with the same seed: 0.17 s; quantiles unchanged (8,474.58 / 10,000 / 12,048.19); sim mean 10,088.6 against exact 10,082.5; SD of k 8.96; ring toss 2 misses in the first 100 (runs 13 and 71) and 464 of 10,000. Re-pin A-F19.sim_* and A-CH12.ring_toss.* through the oracle, and update 09_3 ("8.9"), 09_5 ("ten thousand and eighty"; I'd pin it to the exact 10,082.5) and the 12_2 picture ("97 / 3", "447"). Fallback if the stream must stay: keep the line, comment it `# same law as netting n of N fish (chapter 04)`, and put the fish-level cross-check in the description. |
| dev-02 | Dev | 13_7–13_8 | major | With 30% of fish dying the needle "stays at 10,000" and the gauge says "truth: 10,000", but 7,000 fish are alive. My independent sim agrees: median 10,000 with 6,996 alive. The estimate is right about the lake **on marking day**, because M counts fish marked back then. 13_6 says the newcomers case counts the lake "as it is at the second catch"; 13_8 doesn't state the matching fact. A fisheries manager who took "the estimate doesn't move" as "deaths are harmless" would overcount by 43%. | Voice 13_8: "The estimate doesn't move. Death doesn't care about marks, so the rule still holds. But it counts the lake as it was on marking day, before the deaths." Add a second gauge mark "lake at the second catch: 7,000", matching 13_6's "12,000" mark. |
| dev-02 note | | | | Voice rules are respected in all suggested lines below: no colons, semicolons, dashes or parentheses. | |
| dev-03 | Dev | 10_3 | major | I hear "once you expect four or more caught twice, it's nearly exact on average" while the gauge shows Chapman at −6.5% for this lake. Here E[k] = 50·50/1,000 = 2.5, but the voice never says so, so the needle seems to contradict the sentence. "Nearly exact" also gives me nothing to test against. | Voice: "Here we only expect two and a half caught twice, so it still runs about six percent low. Once you expect four or more, it's within two percent on average." Checked: Chapman's exact mean is 935.22 (−6.48%), and the bound e^(−4) = 1.8%. |
| dev-04 | Dev | 07_2–07_3 | major | "Comes down to two numbers, five hundred twenty-nine against five hundred twenty-eight": the voice never says these are a ratio or where they come from. 07_3's "the top minus the bottom" only makes sense if I've read the 07_1 lower-third. The cancellation is the actual mechanism, and it's what I'd need to rebuild Feller's step or to walk a score curve in code without factorials. | Add: "Divide one score by the other, and almost everything cancels. Twenty-three times twenty-three, over thirty-three times sixteen." Label each factor in table words: 23 "not marked", 23 "missed by the second net", 33 "lake size", 16 "dark cell". Lower-third: "code walks the curve the same way, one multiply per step, in logs" (ties to 11_2). |
| dev-05 | Dev | 14_5–14_6 | minor | "An honest range is forty-seven to ninety-four" assumes every bug is equally findable, which is the assumption 14_6 says fails. So the range is low too, not just the 60. | 14_6, last sentence: "So read sixty, and the range, as probably too low." |
| dev-06 | Dev | 03_5, 03_6, 15_3 | minor | The 03_6 insets are real but were picked to match the voice: only 8 of 25 fifty-fish squares hold exactly 5 marked, and 62 of 185 ten-fish squares hold exactly 1. "Both are one in ten" presents them as typical. If I zoom on a random small square I usually won't see it. The 03_5 trays are an illustration, yet the coda card says "Every number in this video was computed by code or copied from a cited source". | Option 1: 03_6 uses squares about three times wider (side 0.12, roughly 450 and 90 fish), picked only by position (nearest the weed-bed centre and the lake centre), and says "about one in ten" with whatever they hold. Option 2: keep these squares and label them "picked to match; squares this small wobble". In both cases, 15_3's card adds "…or marked as an illustration". |
| dev-07 | Dev | 02_1 | minor | "His book never mentions them" is verified for the 1950 edition only. The fact sheet says the 3rd edition (1968), the copy most people own, has footnote 13 citing Bailey and Chapman (UNVERIFIED). Someone checking their own copy would find the line wrong. | "…and his first edition never mentions them." This adds no extra verification work. |
| dev-08 | Dev | 14_4 (lower-third) | minor | The Mills quote ("'lake' is synonymous with 'system'") is about error seeding, where planted bugs are the marked fish. That is a different design from two independent reviewers, and the lower-third reads as if Mills did the two-reviewer version. The voice's "it counts bugs too" implies the two-reviewer version is used in practice, but nothing cites that. | Lower-third: "Mills, 1970: plant known bugs as marked fish. 'lake' is synonymous with 'system'…". In the description, cite the two-inspector use: Eick et al., ICSE 1992 (I'm recalling this; verify before citing), plus Briand et al. 2000 with its own qualifier ("when the number of inspectors is too small… underestimation may be substantial"). |
| dev-09 | Dev | 01_8 | minor | The first thing I'd ask is "why not average many random dips?" Ruth's storyboard note (many random quadrats work; one net can't measure its own coverage) doesn't reach the voice, a card or a lower-third, which R2 requires. | Lower-third at 01_8: "Averaging many random dips works. All 100 here average exactly 10,000, but only 22 single dips land within 2× of it." I recomputed both from the cell grid. |
| dev-10 | Dev | 15_1 | minor | The four-list, sixteen-cell picture implies a different method from the two-catch formula, but nothing names it. I'd go looking for the two-list arithmetic and not find it. | Lower-third: "four lists, log-linear models (multiple-systems estimation): Ball et al. 2002, report to the ICTY". |
| dev-11 | Dev | 12_5 | minor | The practical planning rule (typical error about 1/√(caught twice): ±30% at 10, ±10% at 100, with a 95% range about twice that) lives only in the description, per the 10 notes. That is the one number I'd use to size a real experiment. | 12_5 slider caption: "typical error ≈ 1 ÷ √(caught twice)" next to the existing "±9%". |
| dev-12 | Dev | 09_6 code | minor | `for k in range(n + 1)` reuses `k`, the simulated array's name. Python 3 comprehension scope leaves the array alone (I checked), but it reads as clobbering it, and the same code pasted as a `for` loop in a REPL would clobber it. | Rename the loop variable to `j`: `comb(M, j) * comb(N - M, n - j)`. |
| dev-13 | Dev | 09_6 / description | minor | The panel's digits depend on numpy's Generator stream. I reproduced them on numpy 2.4.1 (Python 3.14.6); the oracle used 2.5.4. I believe NumPy doesn't guarantee identical Generator streams across releases (NEP 19; I haven't re-read it). | Description: "digits reproduced with NumPy 2.4.1 and 2.5.4; other versions may differ in the last digits". |
| dev-14 | Dev | whole script; storyboard §2 | minor | The script is 2,035 words, about 14 minutes at 145 wpm. Add about 15 two-second prediction holds, about 45 s of listen windows, act cards and the title, and it comes to roughly 15–16 minutes against the brief's "~13 min, 10 chapters". The storyboard's own chapter estimates sum to 990 s. The arc table still uses the old 10-chapter numbering ("07 check it by cheating, 08 how sure…"). | Measure after TTS. Then either update the brief's target or trim. Renumber the arc table to 09 / 11 / 13. |
| dev-15 | Dev | 11_2 | delight | The scrolling zeros with "a float64 rounds to zero past here" at the 324th zero, plus "so code works in logarithms", is exactly the connection I wanted. Checked: 5.3515e-324 stays nonzero and 5.3515e-325 becomes 0.0. | Keep. |
| dev-16 | Dev | 12_1 | delight | I implemented the range from the spoken rule alone and got [8,400, 12,073], the toy's [19, 138] and the bugs' [47, 94]. The 12_5 widths 8,274 / 3,673 / 1,597 and coverage 0.95453 also matched. That is "enough precision to implement it". | Keep the wording exactly. |
| dev-17 | Dev | 14_6 | delight | "Reviewers of different skill are fine. Bugs of different difficulty are not." This is the most useful sentence in the video for my day job, and the derivation E[a]E[b]/E[k] = N holds. | Keep. |
| dev-18 | Dev | 08_5 (lower-third) | delight | Exact fractions tie, and an lgamma scan picks 10,000 by rounding. I reproduced logL(9,999) − logL(10,000) = −7.276e−12 and argmax 10,000. It's an honest note about how implementations break ties. | Keep. |

### What I checked, and how

- **Spoken numbers.** I recomputed them in exact integer or Fraction arithmetic with `python3 -I`, in a scratch folder outside the repo. All match:
  - 01: 451 × 100 and the 31.4× spread.
  - 04: 116,280 / 24 = 4,845; piles 1,365 / 2,275 / 1,050 / 150 / 5; 28.2 / 47.0 / 21.7 / 3.1 / 0.1%.
  - 05: 7.79 / 31.78 / 15.90%; argmax 33; partial sum 17–60 = 9.89; sum to 2,000 = 16.656 against 50/3.
  - 07: 529/528 and 576/578 equal the exact likelihood ratios; 49 − 48 = 100 − 99 = 1.
  - 08: 810,000 both ways; L(9,999) = L(10,000) exactly.
  - 09_4: 11,111 and 9,091.
  - 10: P(k = 0) = 7.198% (1 in 13.9); Chapman at k = 0 gives 2,600; Chapman mean −6.48%; plain guess given k > 0 is +24.1%.
  - 11: 5.35 × 10⁻⁴³⁰ (log₁₀ −429.27), 4.43%, 7.96%.
  - 01_8 grid: sum 10,000, median dip 3,700, 22 cells within 2×.
- **The 09_6 panel, run verbatim.** It runs, and building `p` takes 0.20 s. Sim mean 10,080.002; percentiles 8,474.58 / 10,000 / 12,048.19; SD of k 8.912. `p` sums to 1.0 and p[100] = 0.0442892. Ring toss: 447 misses, and the first-100 misses are runs 40, 61 and 80. The fact sheet's claims about the panel hold.
- **Algebra.** Mn − kN = ab − k·u; the ratio (N−M)(N−n)/(N·u); both absorption identities in the 10 notes; E[a]E[b]/E[k] = N for 14_6.
- **Partially checked.** My own sims (different seed) of the 13_3 bold-fish case gave a median of 7,812 against the pinned 7,752; the direction and size are consistent, but I did not reproduce the pinned number. For 13_4–13_6 and 13_9 I checked only the expected values: 7,000 / 14,500 / 12,500 / 12,000 / 6,250 / 25,000, against pinned medians 7,143 / 14,493 / 12,500 / 12,122 / 6,250 / 25,000.
- **Unchecked.** The Laplace, Kosovo, Mills, Briand and Feller quotes are recorded sources I did not re-fetch. I didn't check chapter 03's dynamics. The video description isn't in the packet, so I couldn't confirm that the "Dev (description)" items (absorption identities, the planning card, the numpy `inf` warning) actually landed.

I wrote no files in the repo; scratch scripts are in `<scratch>/`.
