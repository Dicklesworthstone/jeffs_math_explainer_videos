<!--
Draft delivery description for the apportionment rebuild candidate (bead
mx-video-program-program-87do.30, 2026-11-04). Every number below is a fact
printed by oracle.py (and by src/facts.rs, which the Rust mirror check matches
key for key), a year, or a cited number with its source named;
docs/description_numbers.tsv lists the ones that are not facts. The file is
not tag-bound yet (`#! legacy`). The chapter list carries no timestamps: no
cut exists. Nothing in the film has been rendered, synthesized or listened to.
-->

# Adding a Seat

A council of fifteen seats is shared fairly among three towns, Ash, Birch and
Cedar. Then the council grows by one seat, and Ash loses a seat. This video
shows why that happens under the oldest fair rule. It follows the same three
towns through every rule that fixes it, and every rule that breaks something
else. It ends with the theorem that says no rule can keep every promise at
once: a rule that keeps each town within its fair share, rounded down or up,
must sometimes punish a town for growing.

## Chapters

1. One more seat
2. Seats do not come in fractions
3. Whole parts first, then the largest tails
4. The sixteenth seat
5. Tails move at different speeds
6. It comes and goes
7. Change the yardstick, not the rounding
8. One list for every house size
9. The cost of never taking back
10. Round up, round to the nearest
11. Can Webster break quota?
12. Both promises at once
13. When populations change
14. No rule keeps both
15. The whole map

## Precisely

What the narration says in plain words, stated exactly. The towns are a
model: Ash has 4,000 people, Birch 9,000 and Cedar 26,000, so 39,000 in all.

**Shares and the quota promise.** A town's share of a council of h seats is
its population times h divided by the total. At fifteen seats the shares are
1.54, 3.46 and 10.00. At sixteen they are 1.64, 3.69 and 10.67. A rule keeps
*the quota promise* when every town gets its share rounded down or rounded up.

**Hamilton's rule and the paradox.** Hamilton's rule gives each town the whole
part of its share, then hands the leftover seats to the largest tails. At
fifteen seats that gives 2, 3, 10. At sixteen it gives 1, 4, 11, so Ash loses
a seat although its share grew. Each added seat moves every tail by the
town's fraction of the people, 4/39, 9/39 and 26/39 of a seat, and a tail
wraps back to zero when it passes a whole seat. A slow tail can be overtaken
from one house size to the next. From 1 to 100 seats, 5 of the 99 one-seat
additions take a seat from somebody, and every time it is Ash (at 15 to 16,
23 to 24, 54 to 55, 62 to 63 and 93 to 94).

**Seaton's letter.** After the 1880 census, C. W. Seaton, chief clerk of the
Census Office, computed Hamilton apportionments for every House size from 275
to 350. Young quotes his letter to Congress: "While making these calculations
I met with the so-called 'Alabama' paradox where Alabama was allotted 8
Representatives out of a total of 299, receiving but 7 when the total became
300. Such a result as this is to me conclusive proof that the process employed
in obtaining it is defective." These were alternative calculations, not a
House that was enacted and then shrank Alabama's delegation. The film speaks
and shows only the last sentence.

**Divisor rules.** Jefferson's rule divides every population by one yardstick
(people per seat) and rounds every quotient down, choosing the yardstick so the
whole parts add up to the house. At fifteen seats any yardstick above 2,250 and
up to about 2,363.6 gives 1, 3, 11. A town's k-th seat opens when the
yardstick falls to its population divided by k, its *signpost*. All signposts
form one list, largest first, and h seats are the first h signposts, so a
larger house never takes a seat back. Ash's second signpost and Cedar's
thirteenth are both exactly 2,000, so a house of 18 needs a tie rule; no
example in the film depends on it. Adams rounds up (2, 4, 9 at fifteen seats)
and Webster rounds to the nearest (2, 3, 10 at fifteen, 2, 4, 10 at sixteen,
2, 4, 11 at seventeen).

**Hill's thresholds.** The Huntington Hill method, the law for the House since
1941, rounds a quotient between n and n + 1 up once it passes √(n(n + 1)).
Because n(n + 1) = n² + n is less than (n + ½)², every threshold sits below
Webster's halfway mark, by ¼ ÷ (n + ½ + √(n(n + 1))), a gap that shrinks as n
grows: about 1.41, 2.45 and 3.46 for n = 1, 2, 3. Between 0 and 1 the
threshold is √0 = 0, so any town with people rounds up to a first seat, and
whenever the house has at least one seat per town every town gets one. At
three seats Hill gives 1, 1, 1 although Cedar's share is exactly 2.00.

**Webster keeps quota with three towns.** Suppose Webster gives town i a seats
with a ≤ q_i − 1, where q_i is its share, at yardstick d. Rounding to the
nearest means p_i/d ≤ a + ½ ≤ q_i − ½ < q_i, so d is larger than the exact
yardstick P/h. Then every quotient p_j/d is below its share q_j, and each other
town gets at most p_j/d + ½ < q_j + ½. Adding up, h < (q_i − 1) + Σ_{j≠i}(q_j +
½) = h − 1 + (n − 1)/2, which is h when n = 3: a contradiction. The same
argument, turned around, rules out a seat or more above the share rounded up.
This is Balinski and Young's result (1982, Proposition 6.2, as cited by Gölz,
Peters and Procaccia); the argument above is the film's. With four towns the
inequality leaves half a seat of room, and the film's example uses it: towns
of 2,000, 5,000, 8,000 and 27,000 sharing 14 seats have shares 0.67, 1.67,
2.67 and 9.00. Rounding to the nearest lifts each small share by a third of a
seat and hands out 15 seats, and Webster's yardstick must rise until the big
town's quotient falls below 8½, giving 1, 2, 3, 8.

**The quota method.** Balinski and Young defined it in 1974 (PNAS, p. 4604,
Theorem 2: "There exists a unique consistent, house-monotone method Q = Q(p)
satisfying quota."). Seats are handed out one at a time. With a_i seats held
in a house of h, the next seat may go only to a town in E(h + 1) = {i : a_i <
p_i(h + 1)/P}, one that stays within its share rounded up at the new house
size, and among those to the largest p_i/(a_i + 1): the first eligible
signpost on Jefferson's list. The set is never empty, since the a_i add up to
h and the new shares add up to h + 1. For our towns it gives 1, 3, 10 at
fourteen seats, 1, 4, 10 at fifteen (Cedar's eleventh signpost would lift it
above its share of exactly 10) and 1, 4, 11 at sixteen. It also skips once
earlier, at six seats, where Cedar's share is exactly 4.

**Why the quota method keeps quota (the film's argument, chapter 12).** Lay
town i's seats along the house sizes as tiles: its k-th tile runs from
(k − 1)P/p_i to kP/p_i, the house sizes over which its share goes from k − 1
to k (for Ash 9 3/4 long, Birch 4 1/3 and Cedar 1 1/2). At house size h a tile
is *passed* if it ends at or before h and *entered* if it starts strictly before
h. The share rounded down counts the passed tiles and the share rounded up the
entered ones, so the quota promise says every passed tile is held and no
unentered tile is. The rule's eligibility test is exactly "the town's next tile
is entered", so the upper half holds by construction. Jefferson's priority
p_i/(a_i + 1) is P divided by the end of that next tile, so the rule hands out
the entered unheld tile that ends soonest. Now suppose a passed tile T (town
i's next tile) is unheld at house H. Let t0 be the last house size up to H at
which the rule handed out a tile ending strictly later than T, or 0 if there is
none. At t0, every town's next tile that ends no later than T, town i's
included, had not been entered, or the rule would have taken it instead. Each
tile handed out from t0 + 1 to H ends no later than T (that is the choice of
t0) and is its town's next tile at t0 or a later one, and so is T; so all of
them start at or after t0 and end by H. That is H − t0 + 1 different whole tiles inside the stretch from t0
to H. But town j's tiles have length P/p_j, so at most (H − t0)p_j/P of them
fit inside, and summed over the towns that is H − t0: a contradiction. Ties in
the priority do not matter, since every comparison used is strict. The film
draws this on the stretch from 6 to 13 seats, 7 seats long, which holds 0, 1
and 4 whole tiles of Ash, Birch and Cedar, 5 in all. The film proves only that
the method keeps quota at every house size; it does not prove Balinski and
Young's uniqueness claim. Its own scan agrees through 35 seats (at 36 two
signposts tie).

**Divisor rules and growth.** A signpost is a population divided by a fixed
number, so when a town grows all of its signposts get bigger and can only pass
the signposts of towns that did not grow. In the census of chapter 13 (Ash
3,900, Birch 9,500, Cedar 26,100, 16 seats) Hamilton's seats go from 1, 4, 11
to 2, 4, 10, so Cedar, which grew, loses a seat to Ash, which shrank. Jefferson
gives 1, 4, 11 before and after. In the puzzle of chapter 15 (Webster, 15
seats, Cedar grows to 30,000) the cutoff signpost rises from 2,667 to 2,857,
Ash's next signpost (1,600) stays below it, and the seats go from 2, 3, 10 to
1, 3, 11.

**The impossibility (chapter 14).** Balinski and Young proved in 1982 that
no rule keeps both the quota promise and the population promise on every map,
with an extra assumption. Gölz, Peters and Procaccia report that the proof
and its later variants "all assume what Robinson and Ullman (2010) call the
order-preserving property, i.e., if state i has strictly larger population
than state j, then i must receive at least as many seats as j" (§3.1). Their
Theorem 1 removes the assumption for any deterministic method. Its proof uses
the three maps of their Table 1 at 10 seats: a city of 824 people and four
villages of 44 (shares 8.24 and 0.44); the same map with the fifth village
grown to 222 (shares 6.99 and 1.88); and the first map with villages two and
three shrunk to one person each (city share 9.02). With three towns Webster
keeps both promises, and four towns were open as of that paper. The paper's
lottery rule keeps the quota promise in every draw, never takes a seat back,
and gives every town exactly its share on average: Ash at fifteen seats gets 2
seats in 54% of draws and 1 in 46%.

## Sources

- M. L. Balinski and H. P. Young, "A new method for congressional apportionment," Proceedings of the National Academy of Sciences 71(11): 4602–4606, November 1974, doi:10.1073/pnas.71.11.4602 (PMC433936). Read by eye from the page scans: p. 4604, the definition of E(h + 1), Theorem 2 and the recursion that defines the quota method. The paper states the theorem without a proof.
- P. Gölz, D. Peters and A. D. Procaccia, "In This Apportionment Lottery, the House Always Wins," arXiv:2202.11061v2, accepted by Operations Research in 2024. §3.1 (the order-preserving quote, Balinski and Young 1982 Proposition 6.2 on three states), Theorem 1 and Table 1 (maps A, B and C), and the randomized method.
- H. P. Young, "Fairness in Apportionment," U.S. Census Bureau, https://www2.census.gov/about/history/historical-censuses-surveys/survey-census-history/decennial/fairness-in-apportionment-young.pdf. PDF p. 10, Table 4 (New York after 1830: quota 38.593, 40 seats); PDF p. 13, printed p. 12 (Seaton's letter, verbatim above); PDF p. 16 (Huntington Hill adopted in 1941).
- M. L. Balinski and H. P. Young, Fair Representation (1982), Proposition 6.2 and the impossibility theorem: cited through Gölz, Peters and Procaccia only, not read.
- M. L. Balinski and H. P. Young, "The Quota Method of Apportionment," American Mathematical Monthly 82 (1975): not read; the film cites the 1974 definition instead.
