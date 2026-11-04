# Can Adding a Seat Take One Away?

**Exploratory narration draft, not a production script.** One continuing fictional example; the proposed visuals are scene ideas, not rendered footage. Numerical checks are author checks. Independent mathematical and pedagogical review, complete competitor viewing, owner selection, and eventual ElevenLabs listening and fine synchronization remain open. The [research brief](apportionment-brief-20261031.md) contains sources, competing treatments, exact calculations and the current disposition.

## 1. A seat disappears

**Picture:** Three population bars labeled A, B and C; fifteen seat tokens below them. The bars have relative sizes 4, 9 and 26. An outcome appears: 2 / 3 / 10.

Three regions need to share fifteen seats. Their populations are in the ratio four to nine to twenty-six. Nobody moves in or out, and those populations will stay fixed throughout this story.

One allocation rule gives A two seats, B three, and C ten. Now add a sixteenth seat. What can happen to A?

**Hold for a prediction.** Change only the house size. Reveal 1 / 4 / 11.

Under that same rule, A gets one seat instead of two. The other two regions each gain one. We are comparing two fresh calculations, not watching a particular physical chair travel between real legislatures. How can the arithmetic do that?

## 2. The fractions compete

**Picture:** Split each exact share into whole blocks and a fractional tail; keep the three tails aligned.

The rule is called Hamilton's method, or largest remainders. First work out each region's exact proportional share. A has four of the thirty-nine population units, so with fifteen seats its share is fifteen times four over thirty-nine: one and seven thirteenths. B's is three and six thirteenths. C's is exactly ten.

Give each region the whole number first. That uses one plus three plus ten, or fourteen seats. One remains. Give it to the largest fractional tail. Seven thirteenths beats six thirteenths, so A gets it. The result is two, three, ten.

Now use sixteen seats. Every exact share increases, because the populations stay fixed. The whole-number parts are still one, three, and ten; this time two seats remain. Written with the same denominator, the tails are twenty-five thirty-ninths for A, twenty-seven thirty-ninths for B, and twenty-six thirty-ninths for C.

**Picture:** Rank the three tails without rounding their values.

B and C have the two biggest tails, so they receive the two remaining seats. A falls to one. Adding one seat changed which fractional pieces win the rounding contest; the rule recalculated the whole allocation.

## 3. Does it always happen?

**Picture:** Add a seventeenth token. Hide the answer briefly.

What if we add one more? This time the whole-number parts are one, three, eleven. The two largest tails belong to B and A. The result is two, four, eleven. A gains a seat and nobody loses one.

So the odd outcome is possible under this rule; it is not guaranteed every time the house grows.

## 4. A different promise

**Picture:** Return to the same three populations. Start a separate seat-by-seat list; do not reuse the Hamilton seat icons as if they had identities.

What if the rule never took back a seat as the house grows? Jefferson's method can be written as a contest for the next seat. Divide each region's population by one more than the number of seats it already has. The highest result wins the next seat. Then recalculate the winner's number and repeat.

We start from zero seats for this mathematical example. Fast-forward the first fourteen awards, leaving one, three, ten. The next priorities are four over two for A, nine over four for B, and twenty-six over eleven for C. C's number is largest, so C gets seat fifteen. We now have one, three, eleven.

For seat sixteen, only C's denominator changes: its priority becomes twenty-six over twelve. B's nine over four is now largest. B wins, making one, four, eleven.

For fixed populations and a fixed way to resolve ties, a sixteen-seat allocation is the first sixteen awards on the same list whose first fifteen awards made the fifteen-seat allocation. Extending that list cannot erase an earlier award. In this example none of the winning priorities through seat seventeen is tied.

## 5. Where did the divisor go?

**Picture:** Lay the last winning and next losing priorities on a number line; put a movable divider between them.

You may also see Jefferson's method described using a divisor. These are two views of the same mechanism, when no tie straddles the cutoff.

At fifteen seats, the last winning priority is twenty-six elevenths. The next is nine quarters. Put a divisor between them, say two point three. Divide each population by it and keep the whole part. You get one, three, eleven.

Why? A region receives one seat for each of its priorities at or above that divider. Dividing its population by the divider and rounding down counts exactly those priorities. We need the next unawarded priority strictly below the divider and the last awarded priority at or above it. If priorities tie at the boundary, this simple choice of divider needs extra care; our example has no such tie.

## 6. The price of the promise

**Picture:** Restore the 15-seat standard share for C, exactly 10, beside its Jefferson total of 11.

The seat-by-seat rule stopped the take-back. But look at C with fifteen seats: its exact proportional share is ten, while Jefferson gives it eleven. The quota rule says an allocation must be the floor or the ceiling of the exact share. When the share is exactly ten, both bounds are ten. Eleven violates that rule.

Be careful which divisor is which. The standard share uses the total population and the actual house size. The modified divisor is a tool for finding Jefferson's allocation. They do not mean the same thing.

## 7. Is a tradeoff inevitable?

**Picture:** Three rows, for fifteen, sixteen and seventeen seats. Mark allowable quota ranges alongside the proposed allocations.

Does that mean quota and no take-backs must conflict? Not in these three rows. Give the regions two, three, ten at fifteen seats. At sixteen, use two, four, ten. At seventeen, use two, four, eleven. Each row has the right total, each entry lies within its quota bounds, and no column goes down.

We hand-picked those three rows. They are not an algorithm for every population and every house size. But they do show that the two examples we just saw do not force a conflict here. Mathematicians have constructed methods that meet quota and avoid house-size take-backs more generally; other fairness requirements raise further questions.

## 8. Back to Alabama

**Picture:** Label two hypothetical alternatives, 299 seats and 300 seats; do not animate an enacted change.

After the 1880 census, a calculation under a largest-remainder rule gave Alabama eight seats in a 299-seat house and seven in a 300-seat house. Those were alternative calculations, not a claim that an enacted increase removed a seat.

The lesson is not that a number can lie. The exact shares increased in our example. The surprise came from a rule for turning all those fractions into whole seats, and from asking what stays fair when the total changes.

## Author-check notes

- The first fourteen Jefferson winners are C, C, B, C, C, C, B, C, A, C, C, B, C, C. Exact `Fraction` calculations give the 15th priority 26/11 and the next 9/4; no winning-priority ties occur through seat 17.
- At 15, the modified divisor 23/10 yields floors 1/3/11. At 16, 11/5 yields 1/4/11. These check the example; the cutoff argument states the required strictness.
- The hand-picked three-row quota and monotonicity check covers only those rows. Its general existence sentence rests separately on Balinski and Young, *Quotatone Apportionment Methods* (1979), linked from the brief.
- Estimated duration has not been measured with a narrator. Prediction holds, copy clarity, historical wording, numerical claims, visual pacing, independent proof and review, and ElevenLabs voice/listening/fine synchronization remain open.
