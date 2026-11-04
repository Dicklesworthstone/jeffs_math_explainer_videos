# The extra seat that makes a seat disappear

**Status: candidate for comparison, not a selected or approved production.** Existing catalog identity `apportionment`, Bead `mx-video-program-program-87do.30`. Consumer: Jeff choosing a next-video subject. Proposed payoff: use one fixed set of three populations to show both a disappearing seat under Hamilton's largest-remainder method and a quota violation under Jefferson's divisor method. The same moving picture would make the tradeoff visible. This is a teaching proposal, not a claim that the paradox, examples or comparison are novel. A complete exploratory [narration and picture draft](apportionment-narration-draft-20261101.md) connects those scenes; it has not been independently reviewed, timed with a narrator or approved for production.

## Opening and continuing example

Three fictional regions have populations in the ratio **2 : 7 : 27**, total 36 units. Their populations do not change. Distribute 28 seats in proportion to population: each exact quota is population × 28/36. Hamilton's rule gives everyone the floor of its quota, then gives the remaining seats to the largest fractional remainders. There is no remainder tie in either case below.

| House | A, population 2 | B, population 7 | C, population 27 | Hamilton seats |
|---|---|---|---|---|
| 28 | 14/9 = 1 + 5/9 | 49/9 = 5 + 4/9 | 21 | **2, 5, 21** |
| 29 | 29/18 = 1 + 11/18 | 203/36 = 5 + 23/36 | 87/4 = 21 + 3/4 | **1, 6, 22** |

At 28 seats there is one remainder seat; A's 5/9 beats B's 4/9. At 29 seats there are two remainder seats, but B's 23/36 and C's 3/4 both exceed A's 11/18. A loses its remainder seat even though its exact quota increased. The outcome is a recalculation for a larger house with identical populations, not a state losing population or a historical election.

### Smaller continuing example for a visual treatment

A local search also found a **4 : 9 : 26** population ratio where the same two contrasts fit into 15 and 16 seats. This is a different synthetic example, not a historical population or a claim of a globally smallest construction.

| House | A quota / remainder | B quota / remainder | C quota / remainder | Hamilton | Jefferson |
|---|---|---|---|---|---|
| 15 | 20/13 = 1 + 7/13 | 45/13 = 3 + 6/13 | 10 + 0 | **2, 3, 10** | **1, 3, 11** |
| 16 | 64/39 = 1 + 25/39 | 48/13 = 3 + 27/39 | 32/3 = 10 + 26/39 | **1, 4, 11** | **1, 4, 11** |

At 15 seats A takes the one remainder seat because 7/13 > 6/13 > 0. At 16, B and C take two remainder seats: 27/39 > 26/39 > 25/39. The extra seat goes to C and A's formerly awarded seat shifts to B. Under Jefferson, the 15th priority is 26/11, giving C eleven seats although its quota is exactly ten. The next highest priority is 9/4, so B gains the 16th seat. A separate exact `Fraction` run reproduced all quotas and allocations and found no tie for the winning priority among the first 16 awards. This is an author calculation, not an independent review.

For reproducibility, the Python check below in **A different rule** can also check this example: change `population` to `(4, 9, 26)`, the first loop to `(15, 16)`, the second loop to `range(1, 17)`, and its final comparison to `(15, 16)`. Do not interpret those substitutions as a check of all possible populations or apportionment rules.

**Proposed picture sequence:** start with 15 physical seat tokens and show each quota as a filled bar plus a fractional tail. Give the single remaining token to A; ask the viewer to predict what adding one seat does. Recompute the three growing bars for 16 and reorder their tails, explicitly showing that an already awarded token moves from A to B while the new one goes to C. Replay the same population and house with Jefferson's sequential priority ladder, freeze C at 11 against its exact quota of 10, then return to the historical 299/300 example. Show why the sequential list is house-monotone with fixed populations and consistent tie order; do not imply it solves all fairness questions. A later claim about *all* allocation methods needs a separately reviewed theorem.

### A worked opening and payoff, for a draft storyboard

**Proposed title:** Can Adding a Seat Take One Away? This is a candidate title, not a novelty or production decision. Keep the three regions and the same 15 seats visible through both methods; the viewer should be able to track the changing allocation rather than memorize method names.

1. **The wager.** Place 15 seats under three population bars labeled 4, 9 and 26. The bars are ratios, not actual state populations. Reveal the Hamilton result 2 / 3 / 10 and ask: if we add a seat and nobody moves, can any region lose one? Hold the question before showing 1 / 4 / 11. Make clear that this is a fresh recalculation, not a claim that a real representative was removed.
2. **Open the machine.** At 15 seats show the exact quotas 20/13, 45/13, 10. Their floors use 14 seats; A's remainder 7/13 beats B's 6/13. At 16, the floors still use 14. Show the remainders as equal-width strips with a common denominator: A 25/39, B 27/39, C 26/39. B and C take the two remaining seats. Label the animation as a *comparison of allocations*: A's old award is reassigned to B, while the newly available seat is assigned to C. The arithmetic does not identify physical chairs across separate allocations.
   **A check against overgeneralizing:** add one more seat, still with the same populations. At 17 seats the floors are 1 / 3 / 11 and the remainders are 29/39, 36/39 and 13/39. B and A win the two remaining seats: Hamilton gives **2 / 4 / 11**. Comparing 16 → 17, A gains one and nobody loses. The paradox is a possible result of the rule, not something that occurs at every house-size increase. The adjacent 16 → 17 comparison also gives a second prediction question before the reveal. This example's calculated remainder winners have no tie.
3. **Offer a repair.** Return to the unchanged populations and award seats one at a time by the largest next priority, population / (current seats + 1). Fast-forward earlier awards only if the order remains inspectable. At seat 15, compare C's 26/11 with B's 9/4 and A's 2: C wins, leaving 1 / 3 / 11. At seat 16, B's 9/4 exceeds C's 13/6 and A's 2: B wins, leaving 1 / 4 / 11. Every larger house continues the same ordered award list for fixed populations and a consistent tie rule, so this rule cannot take back an earlier award. The original 15/16 comparisons themselves have no winning-priority ties.
4. **The cost of that repair.** Bring back the *standard* 15-seat quota, 10 for C, beside its 11 allocated seats. C has more than its ceiling, so this Jefferson example violates the quota rule. Distinguish the standard quota from a modified divisor quotient. A transparent cross-check uses modified divisors 23/10 and 11/5: flooring population/divisor produces 1 / 3 / 11 and 1 / 4 / 11 respectively. These two numerical cross-checks support this example, not a general proof that all methods face this particular tradeoff. The [University of Kentucky review](https://www.ms.uky.edu/~guillou/S09/181review3.pdf) gives the corresponding next critical divisor population/(current seats+1); [OpenStax](https://openstax.org/books/contemporary-mathematics/pages/11-4-apportionment-methods) explains adjusted-divisor flooring and gives a separate Jefferson quota-violation example.
5. **Refuse the false dilemma.** Ask whether the *two failures we just saw* mean that quota and no take-backs must conflict. For this same example, set out the following hand-picked allocations and allowed quota ranges:

   | House | A: seats (quota range) | B: seats (quota range) | C: seats (quota range) |
   |---|---|---|---|
   | 15 | 2 (1–2) | 3 (3–4) | 10 (10 exactly) |
   | 16 | 2 (1–2) | 4 (3–4) | 10 (10–11) |
   | 17 | 2 (1–2) | 4 (3–4) | 11 (11–12) |

   All three rows use the correct number of seats; each entry lies between the exact quota's floor and ceiling, and each column stays level or rises. This demonstrates coexistence for **these three rows**, not an algorithm or a guarantee for other inputs. The stronger general existence statement is separately sourced: [Balinski and Young's 1979 paper](https://pubsonline.informs.org/doi/pdf/10.1287/moor.4.1.31) studies house-monotone methods satisfying quota. Do not call this particular hand-picked sequence their method or assert it follows any specific general method without checking.
6. **Close the loop.** Return to the historical Alabama comparison, 299 versus 300 hypothetical seats, and say what was learned: Hamilton protects quota but can lose a seat as the house grows; Jefferson's fixed priority sequence prevents that particular reversal but can exceed quota. Those examples alone do not establish an impossibility. Avoid promising a solution to every fairness criterion or giving a policy endorsement.

**Draft narration for the false-dilemma turn, not approved production copy:**

> We tried keeping each region close to its exact share, and an extra seat made A lose one. We tried giving out seats one at a time, and no one had to give a seat back, but C ended up with eleven when its exact share was ten. Is that tradeoff unavoidable?
>
> Let's keep the same populations and try three particular house sizes. At fifteen seats, use two, three, and ten. At sixteen, give the new seat to B: two, four, and ten. At seventeen, give the new seat to C: two, four, and eleven. In these three rows, nobody loses a seat. And each count stays between the whole numbers immediately below and above its exact share. When the share is exactly ten, both bounds are ten.
>
> That's not a new voting rule. We picked three rows for one example; we haven't said how to allocate every other house size or population. But it does show that the two failures we saw don't force a conflict here. Mathematicians have also constructed methods that satisfy both quota and house monotonicity more generally. Their rules and other fairness properties deserve their own careful explanation.

Author check on 2026-11-01 used exact `Fraction` quotas, floor/ceiling bounds, row sums and componentwise comparisons for these three rows. No ties or future-allocation claims are involved. The check was performed by the author of the draft, not an independent reviewer.

The research sketch includes a separate expandable table for this example, explicitly labeled as hand-picked and limited to three house sizes. An author desktop-browser check opened the table and confirmed all three displayed rows and quota ranges; switching the existing Hamilton and Jefferson controls and revealing the priority panel still worked. Mobile and full keyboard/screen-reader checks were not performed.

**Reproducibility of this opening:** an exact `Fraction` author calculation on 2026-10-31 yielded modified quotients (40/23, 90/23, 260/23) and (20/11, 45/11, 130/11), with the asserted floors. The same calculation gave the last 15th and 16th priorities 26/11 and 9/4, and next priorities 9/4 and 13/6. This is not an independent implementation, proof review, audience test, audiovisual review or owner approval.

**Try the idea:** open the local [apportionment visual sketch](apportionment-visual-sketch-20261031.html). It starts at Hamilton's 15-seat result with the comparison hidden, asks for a prediction, and allows 15, 16 or 17 seats and either method to be selected. The same seat icons and exact standard-quota remainder bars persist across the choices; Jefferson at 15 shows C's exact quota of 10 against 11 seats. Changing house size adds explicit per-region differences, labeled against the previous house size; switching method clears those labels. After the comparison is revealed, the Jefferson panel displays the three candidate priorities before each of seats 15 and 16 and highlights each winner. The allocation is recomputed using integer comparisons. Author desktop-browser checks exercised the original four method/house combinations, prediction reveal, quota display, change labels and priority panel. On the expanded sketch, a further author check saw Hamilton 15→16: 2/3/10 → 1/4/11, then 16→17: 1/4/11 → 2/4/11 with no loss; Jefferson 17 displayed 1/4/12 and its 16→17 comparison 1/4/11 → 1/4/12. An exact `Fraction` author calculation reproduced these results and found no winning-priority tie in the first 17 Jefferson awards. Earlier checks saw the panel hidden before reveal and for Hamilton, and Space activated the focused reveal button. This is a research-only sketch, not a produced video or an independent calculation. It changes states rather than animating a seat transfer and only displays the last two priority rounds; a fresh reader may not infer the full mechanism without the accompanying text. Mobile-device, complete keyboard-only and accessibility review and fresh-reader testing remain open.

## A different rule, a different cost

Under Jefferson's divisor method (equivalently the sequential highest-averages version with priority population / (current seats + 1)), begin with zero seats and award each new seat to the largest priority. Use a consistent tie rule if needed; **no priority ties occur in these 29 awards**. The same populations give **1, 5, 22** at 28 seats and **1, 5, 23** at 29. Since the 29-seat result simply continues the same priority list, no allocation can decrease when one seat is added, with fixed populations and tie ordering.

But at 28 seats C's exact quota is exactly 21, so receiving 22 violates quota: in general, a quota-respecting allocation must lie between the floor and ceiling of the exact quota, inclusive. The last Jefferson priority at 28 is 27/22; the next is 27/23, ahead of B's 7/6. This example demonstrates a cost of **this** method, not that every house-monotone method must violate quota. It also starts from zero seats for a clean abstract allocation example; do not describe it as a full specification of the current U.S. House's constitutional minimum or present Jefferson as the current statutory method.

**Exact local check:** standard-library `Fraction` calculations reproduced both quota tables, both Hamilton results, all 29 distinct winning Jefferson priorities, and the two Jefferson seat vectors. This is an author check, not an independent proof or audience test. The primary [Census Bureau methods explanation](https://www.census.gov/about/history/historical-censuses-and-surveys/census-programs-surveys/decennial-census/methods.html) describes the largest-remainder and adjusted-divisor procedures; equivalence of the priority implementation and divisor formulation should be independently checked before a script claims it.

Reproduce the exact arithmetic with `python3 -I`:

```python
from fractions import Fraction as F

population = (2, 7, 27)
for house in (28, 29):
    quota = [F(house * p, sum(population)) for p in population]
    seats = [int(q) for q in quota]
    remainder = [q - s for q, s in zip(quota, seats)]
    assert len(set(remainder)) == len(remainder)
    for i in sorted(range(3), key=remainder.__getitem__, reverse=True)[:house - sum(seats)]:
        seats[i] += 1
    print('Hamilton', house, 'quota', quota, 'remainder', remainder, 'seats', seats)

seats = [0, 0, 0]
for house in range(1, 30):
    priority = [F(p, seats[i] + 1) for i, p in enumerate(population)]
    winning = max(priority)
    assert priority.count(winning) == 1
    seats[priority.index(winning)] += 1
    if house in (28, 29):
        print('Jefferson', house, 'seats', seats, 'last priority', winning)
```

## Historical anchor and scope

A searchable excerpt from an original [1881 Census Office letter preserved by GovInfo](https://www.govinfo.gov/content/pkg/SERIALSET-01968_00_00-053-0065-0000/pdf/SERIALSET-01968_00_00-053-0065-0000.pdf), printed page 19 headed “The Alabama Paradox,” explains that with the same 1880-census populations Alabama would receive eight seats in a 299-seat House and seven in a 300-seat House. The full GovInfo PDF did not open in this pass; the excerpt and [H. Peyton Young's Census symposium paper](https://www2.census.gov/about/history/historical-censuses-surveys/survey-census-history/decennial/fairness-in-apportionment-young.pdf), PDF pages 6 and 12–14, support the historical calculation. These were **hypothetical alternative house sizes**, not an assertion that Alabama actually lost a seat when Congress enacted a 300-seat House. Young also develops a three-region comparison. The present 2:7:27 example is synthetic and is not claimed to be the historical data.

Do not frame the whole theorem as “quota and house monotonicity are incompatible.” [Balinski and Young's original quota-method paper](https://pubsonline.informs.org/doi/pdf/10.1287/moor.4.1.31) explicitly studies methods satisfying both. A stronger population-monotonicity incompatibility has additional definitions and assumptions; it is outside this candidate's proved scope until its precise theorem and an independently reviewed exposition are included. Avoid presenting a toy mathematical allocation as legal advice or a policy endorsement.

## Competition and disposition

- [Mathispower4u's *Apportionment: The Alabama Paradox*](https://www.youtube.com/watch?v=-5gMG408H74) is a direct competing video. On 2026-10-31 its complete English transcript (YouTube export: authored or unspecified) was read, but the recording was not continuously viewed or listened to. At 1:31–4:28 it works through populations 6:6:2 and 10 versus 11 seats, giving 4/4/2 then 5/5/1; at 4:33–5:07 it explains the faster-growing larger-state remainders overtaking the smaller state's; at 5:11–5:29 it briefly raises divisor methods and quota. Visual samples at 1:40, 4:40 and 5:15 were actually inspected: the first already displays both final allocations in a side-by-side table, the second retains that table above an explanatory paragraph, and the last states the divisor/quota caveat in text. These sampled frames support a candidate contrast in presentation, not a claim about every frame. Thus a three-region reveal, remainder-ranking mechanism and mention of the cost already exist in a single competing video. Our continuing 4:9:26 example works the two methods and the quota failure on the same allocation; whether its visual payoff materially improves the existing treatment remains untested. Transcript provenance and sampled pictures are not full audiovisual viewing or listening.
- [The Moening Group's *The Alabama Paradox: Apportionment*](https://www.youtube.com/watch?v=OZZNixt2lpA) supplies a useful negative comparison. On 2026-11-01 its full English **auto-generated** transcript was read, and one frame at approximately 3:43 was inspected; the full recording was not continuously watched or listened to. The transcript works a fictional three-state Hamilton example at 70 and 71 seats, 26/20/24 → 26/21/24, concluding that no state loses a seat and the paradox does not occur. The sampled 3:43 frame shows the 70-seat quota/allocation table. This treatment already uses a no-paradox example; our proposed 16→17 counterexample follows the same teaching purpose on the *same* populations as the earlier 15→16 paradox. That is a proposed continuity benefit, not a novelty or audience-effect finding. Auto-captions can misrecognize words and numbers; no spoken-content verdict is implied.
- [Mathispower4u's *Apportionment: Jefferson's Method*](https://www.youtube.com/watch?v=weGGVmy9yLc), 10:11, provides another direct comparison. Its complete English transcript, exported with caption provenance authored or unspecified, was read on 2026-11-01; the video was not continuously viewed or listened to. At 1:29–2:16 it describes reducing a divisor and recalculating; at 2:18–6:09 it applies that to tutors; at 6:17–10:00 a second example tries several divisors, including ones that allocate too many seats. The proposed draft explains the related one-seat priority ladder and, when the cutoff has no tie, uses the gap between the last awarded and next priority to select a divisor. The priority formulation is not an originality claim, and the transcript alone cannot establish the lesson's complete visual or auditory presentation.
- Young's Census paper already gives a three-region Hamilton example and discusses apportionment principles. This is direct written competition, not merely a source. The proposed same-populations, two-method reveal is an unproven presentation distinction.
- A further direct comparison is H. Peyton Young's [Brookings article, *Dividing the House*](https://www.brookings.edu/articles/dividing-the-house-why-congress-should-reinstate-an-old-reapportionment-formula/). Its written explanation contrasts Jefferson's large-state bias and a quota exceeding its ceiling, walks through a 21-to-22-seat three-state Hamilton paradox, and discusses divisor methods and their immunity to the paradox. These are the same substantive ideas, although this pass did not establish whether its illustrative figures use one set of populations for both failures. The article argues for a policy choice, whereas the proposed sketch would explain the two rules on one small object without endorsing a policy. That difference and the 15/16-seat example are proposed presentation differences, not proof of an unserved audience.
- [Pearson's Alabama paradox lesson page](https://www.pearson.com/channels/quantitative-reasoning/learn/colleen/10-voting-and-apportionment/alabama-paradox) presents the paradox, describes Hamilton's remainders, and names Jefferson and Webster as methods that avoid it; the page lists a 5-minute lesson and a 7-minute example. Only the visible lesson text and listings were read, not either video or their audio. Full video comparison remains a release-selection gate.
- Full competing-video viewing, independent numerical and proof review, a fresh-reader test, comparison against the other candidate briefs, and owner selection remain open. Park or reframe if the two-method story fails to supply a material payoff beyond existing treatments. No narration, render, publication, listening or fine-sync acceptance is claimed.
