# Abstract algebra: the full visual build-up

Status: **development draft, not a finished script, render, or release**. The
published 6:32 narration is an overview, not the requested treatment. Preserve
that published cut and its assets. Develop a substantially longer successor
using native FrankenManim scenes and ElevenLabs narration. Do not call it a
complete treatment until its worked examples, arguments and final production
have actually been delivered and reviewed.

## Scope and source

The owner asked for a visual introduction inspired by the early chapters of
Joseph Gallian's *Contemporary Abstract Algebra*, gradually reaching
centralizers, equivalence classes and quotient groups, with applications. The
[publisher's ninth-edition contents](https://www.cengageasia.com/title/default/detail?isbn=9781305657960)
place preliminaries before chapters 1–9: groups, subgroups, cyclic groups,
permutations, isomorphisms, cosets and Lagrange, direct products, then normal
subgroups and factor groups. The owner did not specify an edition; this is a
source-bound organizing reference, not a claim that a whole edition has been
read or that every exercise has been covered. Write original examples and
explanations rather than reproducing the textbook.

The first release collapsed this journey into a sequence of definitions. Its
actual script spends a few sentences on each of groups, cyclicity, centralizers,
equivalence and cosets, then moves directly to a quotient. It does not work
through subgroup tests, permutations in cycle notation, isomorphism, a proof
of Lagrange's theorem or a direct product. It gives an application name-check
and one small parity check, not a developed account of how the machinery helps.
The new work needs these missing bridges, not simply a longer recap.

## Teaching contract

Make one coherent long-form video with chapters; determine its runtime from the
finished script and actual pacing, not the skill's generic 6–9 minute default.
The following are production targets, not measured durations or a reason to
pad. The viewer should be able to stop and work each example with the screen
before the answer is revealed. Explain a definition through a concrete failure
or operation, compute a second example, then prove or qualify the general fact.
The conventions for composition, notation and left cosets must remain visible
when they matter.

1. **Arithmetic and equivalence** (target 7–10 min). Clock arithmetic, integer
   congruence and why remainders make partitions; reflexive, symmetric and
   transitive checked, including a counterexample that fails transitivity.
   Distinguish equality of classes from equality of representatives. Introduce
   functions and composition with arrows, and state that composition order is
   rightmost first throughout.
2. **Symmetry and groups** (8–12 min). Mark a square's corners. Find all four
   rotations and four reflections; build an operation table from moves. Compare
   a rotation then reflection with the opposite order. Explain closure,
   associativity of function composition, identity, inverse, and why
   commutativity is optional. Contrast with an operation that fails an axiom.
   Show uniqueness of identity and inverse, and cancellation, with short proofs.
3. **Subgroups and centralizers** (8–12 min). Nonempty subgroup test
   `a,b ∈ H ⇒ ab⁻¹ ∈ H`, with both directions proved; rotation subgroup and
   a nonexample. Define the centralizer of one chosen move; compute all eight
   candidate commutators for a reflection in the square group, then prove the
   centralizer is a subgroup. Distinguish it from the center of the group.
4. **Cyclic groups and order** (7–10 min). Powers, negative powers and the
   distinction between group order and element order. On a twelve-position
   clock, `⟨8⟩ = {0,8,4}` under addition. Derive `ord(k) = n/gcd(n,k)` in
   `Z/nZ` and explain why subgroups of a cyclic group are cyclic. Exhibit
   `U(12)={1,5,7,11}` where every nonidentity element has order two: order four
   need not imply cyclic.
5. **Permutations** (8–12 min). Follow labeled objects through two permutations;
   write and read cycles, compose rightmost first, find inverses and show
   disjoint cycles commute. Build a transposition decomposition and show parity
   is well defined, with a clear argument rather than inference from examples.
   Relate square symmetries to actual corner permutations, then use a worked
   check-digit or coding example only with its detection guarantee qualified.
6. **Isomorphism** (6–9 min). Two different pictures with the same operation
   structure; construct the map `Z/4Z →` square rotations and verify bijection
   and operation preservation. Contrast `Z/4Z` with `U(12)` via element orders.
   Explain Cayley's theorem with the injective left-translation action, and
   define automorphism as an isomorphism from a group to itself.
7. **Cosets and Lagrange** (8–12 min). Construct left cosets `gH` as actual sets;
   verify the induced equivalence relation and the partition. Show the
   bijection `h ↦ gh`, so every coset has `|H|` elements, and derive
   `|G|=[G:H]|H|` for a finite group. Derive that element order divides group
   order. Explain that the converse does not hold in general.
8. **Direct products** (5–8 min). Pair a 4-step and a 3-step clock, compute
   componentwise, and show the 12-step orbit of `(1,1)`. Contrast `Z/2 × Z/2`
   with `Z/4` using element orders. Separate an external product construction
   from an internal decomposition, and do not assert one without its conditions.
9. **Normality and quotient groups** (10–15 min). Use the established square
   moves, not a new unexplained table. For `H={e,s}`, `e` and `s` are in `H`;
   `er=r` and `sr=r⁻¹s` lie in different left cosets of H, so naïve
   representative multiplication fails. Show conjugation moves the horizontal
   reflection to a different reflection. Prove normality is equivalent to
   well-defined coset multiplication, then construct `D4/⟨r⟩ ≅ C2`. Keep
   orientation distinction separate from whether an image's markings reveal it.
10. **Applications and synthesis** (6–10 min). Work one check-digit example
    completely with an explicit error model, plus one symmetry application in
    physics or graphics. Explain the operation, group, question answered, and
    limits. Revisit the opening square: which distinctions did the quotient
    keep, and which did it discard? Offer several exercises and answers.

The targets total 73–110 minutes; they are a planning estimate, not a promise
of runtime or evidence of completed content. Split native rendering into short
chapters to respect renderer limits, then assemble the long-form video. If a
concept needs more time, give it time.

## First teaching sequence: original narration and picture beats

**Picture:** A blank 12-hour clock. A hand moves from 9 to 11, then a second
hand moves forward another five positions. Do not show the answer yet.

**Voice:** It is eleven o'clock. Five hours pass. Where does the hour hand
point? If you said four, you just did arithmetic in a system where twelve
steps bring you back to the start. On an ordinary number line eleven plus five
is sixteen. On this clock, sixteen and four describe the same position.

**Picture:** A number line rolls onto the clock. Illuminate 4, 16, 28 and -8
one at a time, all landing at the same marked position; maintain the different
integer labels.

**Voice:** Those numbers have not become equal as integers. They are equivalent
for this particular purpose: each differs from four by a whole number of
twelve-hour trips. The distinction matters. If we were counting hours actually
worked, four and sixteen would not be interchangeable. The rule for deciding
what counts as the same depends on the question.

**Picture:** Place integers into twelve vertical lanes, one per clock position.
Track 4 and 16 into one lane, and 5 into a different lane. No integer belongs
to two lanes.

**Voice:** We can put every integer in the lane of its clock position. A lane
is an equivalence class. To make that idea work in general, a relation has three
jobs. Every object must be related to itself. If the first is related to the
second, the second must be related to the first. And if the first is related to
the second, and the second to the third, then the first must be related to the
third. We call these reflexive, symmetric and transitive.

**Picture:** On separate mini-panels, show `a-a=0`, reverse the arrow between
`a` and `b`, then concatenate two 12-step-multiple arrows. Translate the last
panel to `(a-b)+(b-c)=a-c`.

**Voice:** For our clock, each condition has a reason. Zero is a multiple of
twelve. Reversing a multiple gives another multiple. And joining two whole
numbers of trips gives a whole number of trips. That is why the lanes form a
partition: every integer enters one lane, and any two lanes are either exactly
the same or completely separate.

**Prediction:** Are two integers related when they are at most one apart an
equivalence relation? Show `0 ~ 1` and `1 ~ 2`; hold, then reveal `0 ≁ 2`.

**Voice:** Being at most one apart passes the first two tests, but fails the
third. Zero is close to one, and one is close to two, but zero is not close to
two. Without transitivity, chains of pairwise similarity need not form the
clean classes we want.

**Picture:** Return to the clock. Highlight two different representatives of
the same lane, 4 and 16; add 5 to each, yielding 9 and 21, which meet in the
same lane.

**Voice:** Can we add classes without worrying which representative we choose?
Here we can. If `a` differs from `a prime` by a multiple of twelve, and `b`
differs from `b prime` by another multiple of twelve, then the sums differ by
the sum of those two multiples. So the result is independent of our choices.
That sentence is a proof that clock addition is well defined. Later, when we
try to multiply classes of symmetries, exactly this question will become much
more interesting.

**Picture:** A marked square turns and reflects; retain the labels. A
right-to-left composition legend is visible. Cut from clock addition to moves.

**Voice:** The clock gave us a way to combine positions and a way to forget
whole turns. Now let's replace positions by actions. Turn a square, or flip it,
and look at how one move followed by another behaves. The pictures change, but
the questions become remarkably familiar: what is the do-nothing action, can
we undo a move, and what happens if we group actions into classes?

### The square's moves: second narration sequence

**Picture:** Label the square's corners 1, 2, 3, 4 in clockwise order, starting
at the top right. Overlay a fixed, faint outline so the labeled square's final
position can be compared with its start. Introduce the rotation `r`, a
counterclockwise quarter-turn, and the reflection `s` across the horizontal
axis. Display the convention `ab = do b first, then a` beside the picture.

**Voice:** Let's give our square labels, because without them we could turn it
and mistake the result for doing nothing. We will call a counterclockwise
quarter-turn `r`, and a reflection across the horizontal line `s`. When we
write two symbols side by side, the right-hand move happens first. So `rs`
means flip, then rotate. The convention is a choice; keeping it consistent is
not optional.

**Picture:** Beginning from the same labeled square each time, show `e`, `r`,
`r²`, `r³`; then `s`, `rs`, `r²s`, `r³s`. Each becomes a labeled card. Use
actual transformations, not eight prearranged lookalikes. Hold all eight.

**Voice:** First come the four rotations, including zero turns. Then come four
reflections. These are all the symmetries of the square. Why only eight? Pick
where corner one goes: four choices. An adjacent corner must go to one of the
two corners adjacent to that destination. Once those two choices are made,
the other corners are forced. That gives at most eight possibilities, and the
eight moves on screen show that all eight are attained.

**Prediction:** Show `r²` followed by `r³`; pause with the two move cards
visible. Then reveal a full turn plus one quarter-turn, `r²r³=r`.

**Voice:** Combining two of these moves gives another move in the same list.
For example, a half-turn followed by three quarter-turns is five quarter-turns,
which has the same effect as one. This property is closure. It is a statement
about the selected collection and the selected operation. Multiplying two odd
integers is closed among odd integers; adding them is not. One collection can
behave differently under different operations.

**Picture:** Show three move cards `a`, `b`, `c`. The rightmost card acts first;
brackets `(ab)c` and `a(bc)` rearrange around the same ordered sequence. The
three pictured transformations land on exactly the same labeled square.

**Voice:** When we combine three moves, the parentheses may change without
changing their order. That is associativity. We do not have to check hundreds
of separate triples here: our moves are functions, and composing functions is
associative. To see why, start with any point. Both parenthesizations send it
through `c`, then `b`, then `a`. Every point ends in the same place.

**Picture:** Hold `(rs)` and `(sr)` side by side. Animate the labeled square
from the same start; highlight a corner that finishes in different positions.

**Voice:** Parentheses are not permission to exchange moves. Flip then turn,
and turn then flip, put the labels in different places. Thus `rs` and `sr`
are different. Associativity keeps the order fixed; commutativity would allow
us to swap it. Our square symmetries have the first property, not the second.

**Picture:** Put `e` next to each of the eight cards from either side; animate
the square remaining unchanged. Then show `r` paired with `r³`, `r²` with
itself, and each reflection paired with itself.

**Voice:** The identity move, `e`, leaves every point where it started. Do it
before or after any symmetry and the result is unchanged. Every move can also
be undone by a move in the list. A quarter-turn is undone by three
quarter-turns. A half-turn undoes itself. So does each reflection.

**Picture:** Four boxes appear only after the examples: closure,
associativity, identity, inverse. Attach each box to the picture just used.

**Voice:** A group is a set with an operation satisfying these requirements.
The operation must be closed and associative. There must be an identity, and
every element must have an inverse in the set. The square gives us a finite,
noncommutative group, often called `D4` when the subscript refers to the four
vertices. Some authors use a different subscript convention, so we will always
say which group we mean.

**Picture:** Two proposed identity cards `e` and `f` flank the same operation
slot. Move from `ef=e` to `ef=f` with the property used written below each step.

**Voice:** The rules already force useful facts. Could the same group have two
different identities, `e` and `f`? Because `f` is an identity, `ef` equals
`e`. Because `e` is an identity, that same product equals `f`. So `e=f`.
There is only one identity.

**Picture:** Given `ba=e` and `ac=e`, display `b=b(ac)=(ba)c=c`, with each
equality animated. Label the middle step associativity.

**Voice:** Similarly, suppose `b` and `c` are both inverses of `a`. Put the
identity after `b`, replace it by `ac`, regroup, and replace `ba` by the
identity. What remains is `c`. Each element has one inverse. The grouping step
is exactly where associativity does work; none of this lets us swap symbols.

**Prediction:** Present integers with ordinary subtraction as the proposed
operation. Ask which group rule fails. Reveal `(5−3)−1=1` and
`5−(3−1)=3`, with identical integers and differently placed brackets.

**Voice:** Not every familiar operation produces a group. On the integers,
subtraction is closed, but it is not associative. Here the two groupings give
one and three. A group is not just any collection of arithmetic-looking
symbols. Its rules make certain kinds of reasoning reliable.

**Transition:** Fade all reflection cards while retaining the four rotations.
Do not yet label them a subgroup or reveal the test.

**Voice:** Now suppose we keep only the rotations. Do they still have enough
structure to stand on their own? And is there a quick way to decide this for
other collections, without rebuilding every operation table?

### Smaller groups and centralizers: third narration sequence

**Picture:** The four rotation cards stand in a ring, with `r` moving one step
around it. Place each reflection in a separate tray. Compose two rotations and
trace the resulting pointer around the ring.

**Voice:** The rotations do form a group under the same composition. Combining
two rotations stays among rotations. The do-nothing move is included, and
each turn can be undone by another turn in the ring. Associativity comes for
free from the larger group because we did not change the operation. A group
inside a group, using the same operation, is called a subgroup.

**Picture:** Place `e`, `r`, and `r²` in a candidate box; leave `r³` outside.
Ask the viewer to test it. Reveal `r·r²=r³` crossing the boundary.

**Voice:** Not every selection works. If we keep the identity, a quarter-turn,
and a half-turn but leave out the three-quarter-turn, the first two turns
combine to give the missing one. This set is not closed, so it cannot be a
subgroup. The picture is a counterexample: one failed pair is enough.

**Picture:** Put an arbitrary nonempty subset H inside G. A magnifying glass
selects arbitrary `a` and `b` from H. Show `ab⁻¹` and ask whether it stays in
H. Do not mark H as a subgroup yet.

**Voice:** There is a compact subgroup test. Take a nonempty subset `H` of a
group. If, for every two elements `a` and `b` in `H`, the product `a` times
the inverse of `b` is also in `H`, then `H` is a subgroup. Why is one test
enough to recover the identity, inverses, and closure?

**Picture:** Select any `h` in H, guaranteed by nonemptiness. First show
`hh⁻¹=e ∈ H`. Next use the pair `e,h` to show `eh⁻¹=h⁻¹ ∈ H`. Finally, for
`a,b ∈ H`, select `a` and `b⁻¹` and display `a(b⁻¹)⁻¹=ab ∈ H`.

**Voice:** Because the set is nonempty, choose an element `h`. The test
applied to `h` and itself puts the identity in the set. Now apply it to the
identity and `h`: the inverse of `h` is in the set. Finally, given `a` and
`b`, we already know `b` inverse is in the set. Apply the test to `a` and
`b` inverse: the result is `ab`. We have closure. Associativity is inherited
from the original group. Conversely, a subgroup already contains inverses and
is closed, so it necessarily passes the test. Both directions matter.

**Picture:** Return the reflection cards to the screen. Choose the horizontal
reflection `s`. For each move `g`, compare two animated paths: `g` followed by
`s`, and `s` followed by `g`. Record a match or mismatch next to the actual
result, not merely a colored guess.

**Voice:** Now ask a more selective question. Which symmetries can trade places
with this one particular reflection without changing the result? The set of
all such moves is called the centralizer of `s`. It contains the identity and
`s` itself. A half-turn and the vertical reflection work too. The two quarter
turns and the two diagonal reflections do not.

**Picture:** Keep the four matching cards `e`, `r²`, `s`, `r²s` visible. Show
the two ordered paths for `r` and `s` finishing differently. Then move to an
algebra panel reading `as=sa`, `bs=sb`.

**Voice:** Is this collection just an accident of the square? No. The moves
that commute with any fixed element form a subgroup. If `b` commutes with
`s`, then its inverse does too: from `bs=sb`, multiply by `b` inverse on the
appropriate sides. Thus `b⁻¹s=sb⁻¹`. If `a` also commutes with `s`, then
`(ab⁻¹)s=a(b⁻¹s)=a(sb⁻¹)=(as)b⁻¹=s(ab⁻¹)`. The product required by our
subgroup test stays in the centralizer. The identity is there, so the set is
nonempty. The test finishes the proof.

**Picture:** On the left, highlight the centralizer of the horizontal
reflection: four cards. On the right, highlight only cards that commute with
every symmetry: `e` and `r²`.

**Voice:** The centralizer depends on the chosen move. The center asks a
stronger question: which moves commute with every move in the group? For the
square, only the identity and the half-turn do. Notice how a precise question
about reversing order produces a useful new group inside the old one.

**Transition:** Isolate `r`, repeatedly apply it to a marked square and track
the sequence `e, r, r², r³, e`; hide the “cyclic” label until after the loop.

**Voice:** One quarter-turn, repeated, generated every rotation we kept. How
many moves can a single element generate in other groups? That question leads
to cyclic groups and to a powerful way of counting.

### Repeated moves and cyclic groups: fourth narration sequence

**Picture:** On a twelve-position clock labeled 0 through 11, start at zero and
repeatedly add eight. Leave the traveled arrows visible: `0 → 8 → 4 → 0`.
Keep the unvisited positions gray. Next to the clock, write
`⟨8⟩ = {0, 8, 4} ⊂ Z/12Z`.

**Voice:** Here our group operation is addition modulo twelve. Start at zero
and repeatedly add eight. We reach eight, then sixteen, which is four on the
clock, then twenty-four, which is zero. Repeating only cycles through those
three positions. The angle brackets mean the subgroup generated by eight: all
integer multiples of eight, read modulo twelve. It has three elements.

**Picture:** Show a second clock stepping by five. Count all twelve positions
before returning to zero. Then display a small counterexample to an intuitive
shortcut: stepping by eight has order three even though eight is an integer
larger than three.

**Voice:** Starting from zero, repeated addition by five reaches every clock
position before returning. Its order is twelve. The order of an element is the
least positive number of repetitions needed to reach the identity, when such a
number exists. The order of a finite group is instead the number of its
elements. We must keep those two uses of order distinct. Some elements in
infinite groups never return: adding one repeatedly on the ordinary integers
has infinite order.

**Picture:** Write `mk ≡ 0 (mod n)` and then `n | mk`. Let `d=gcd(n,k)`,
`n=dn'`, `k=dk'`. Cancel the common positive `d`, giving `n' | mk'`.
Highlight `gcd(n',k')=1`.

**Voice:** We can calculate the order of any clock step without tracing the
whole orbit. Adding `k` a total of `m` times returns us to zero exactly when
`n` divides `mk`. Let `d` be the greatest common divisor of `n` and `k`, and
write `n=d n prime`, `k=d k prime`. The return condition says that `n prime`
divides `m` times `k prime`. These two primed numbers are relatively prime, so
Euclid's lemma says `n prime` must divide `m`. The first positive choice is
`m=n prime`, or `n/gcd(n,k)`.

**Picture:** Substitute the two checked examples: `12/gcd(12,8)=3` and
`12/gcd(12,5)=12`. Then show step zero: `12/gcd(12,0)=1`.

**Voice:** Eight has order three; five has order twelve. Even the zero step
fits: it is already the identity, so its order is one. A step generates the
whole clock precisely when its greatest common divisor with the clock size is
one.

**Picture:** Replace the clock by an infinite row of integers. Shade a
nonzero subgroup H. Mark its least positive element `d`; divide another
positive member `h=qd+r` with `0≤r<d`. Animate `r=h−qd` remaining in H.

**Voice:** Why are all subgroups of the additive integers generated by a
single number? Apart from the subgroup containing only zero, choose the least
positive member and call it `d`. Divide any member `h` by `d`: it is `q`
copies of `d` plus a remainder smaller than `d`. Both `h` and `q` copies of
`d` are in the subgroup, so their difference, the remainder, is in it too.
But no positive remainder smaller than `d` can be in the subgroup. The
remainder is zero. Every member is a multiple of `d`, and every such multiple
is in the subgroup. Thus the subgroup is exactly the multiples of `d`.

**Picture:** Wrap the integer line back to a clock. Show a subgroup of the
clock and highlight all integers that land in it. The highlighted preimage is
a subgroup of the integers; mark its least positive generator and map its
multiples back to the clock.

**Voice:** The same idea classifies subgroups of a finite cyclic group. Lift
a subgroup of the clock to all integers that land in it. Those integers form
a subgroup of the integers, so they are multiples of one number. Mapping back
shows that the original subgroup is generated by the image of that number.
In a cyclic group of order `n`, there is exactly one subgroup of each order
dividing `n`. We have shown why subgroups are cyclic; the exact count and
divisor correspondence will follow from the orbit formula and the counting
argument we develop with cosets. We are not assuming the result for arbitrary
groups.

**Picture:** List multiplication modulo twelve on the four invertible
positions `1,5,7,11`. For each nonidentity element show its square reaching
`1`. Compare with addition on a four-position clock, where `1` visits all
four positions.

**Voice:** A group having four elements does not automatically make it cyclic.
The invertible residues modulo twelve are one, five, seven and eleven, under
multiplication. Every nonidentity element squares to one. No element visits
all four, unlike the four-position additive clock. Same number of elements,
different internal structure. Soon we will make “same structure” precise.

**Transition:** Relabel the four square corners. Move from repeated rotations
to arbitrary reorderings of the labels, without implying all reorderings are
rigid symmetries of the square.

**Voice:** Our square moves reorder four corners, but most ways to reorder four
labels do not preserve the square's geometry. What happens when we study all
reorderings themselves? That larger group is the world of permutations.

### Permutations and their structure: fifth narration sequence

**Picture:** Four labeled dots occupy four slots. Offer the viewer all
rearrangements; show `4 × 3 × 2 × 1 = 24`. Keep the subset of eight square
symmetries highlighted inside the full set of 24 permutations.

**Voice:** A permutation is a bijection from a set to itself: every label has
one destination and every destination receives exactly one label. With four
labels there are four choices for the first destination, three for the next,
then two and one: twenty-four permutations. The eight square symmetries are
among them. But swapping only two neighboring corners while leaving the others
fixed is a perfectly good permutation and not a rigid symmetry of the square.

**Picture:** Use the earlier clockwise corner labels: 1 top right, 2 bottom
right, 3 bottom left, 4 top left. Animate the counterclockwise quarter-turn
`r`: `1→4→3→2→1`, then display `r=(1 4 3 2)`. Mark the horizontal
reflection `s=(1 2)(3 4)`. Beside both, show `rightmost transformation first`.

**Voice:** Cycle notation follows where a label goes. Our quarter-turn sends
one to four, four to three, three to two, and two back to one, so we write
`(1 4 3 2)`. The horizontal reflection swaps one with two and, separately,
three with four. That is `(1 2)(3 4)`. Labels missing from a cycle are fixed.
These two swaps act on disjoint labels, so either order gives the same result.

**Prediction:** Compute `rs` for these actual corner labels. Keep both cycle
expressions and the convention visible. Let the viewer follow each label;
show the answer only after the pause.

**Picture:** Follow 2 through `s` to 1, then through `r` to 4; follow 4 to 3
then 2. Show that 1 and 3 are fixed. Reveal `rs=(2 4)`. Separately follow
`sr=(1 3)` and display the two results side by side.

**Voice:** To calculate `rs`, apply the reflection first. Two goes to one,
then the quarter-turn takes one to four. Four goes to three, then to two.
One and three return to themselves. The result is the swap `(2 4)`. Reversing
the order gives `(1 3)`. Our earlier picture of noncommuting moves now has a
symbolic calculation behind it.

**Picture:** Starting with `(1 4 3 2)`, animate the same arrows backward.
Reveal `(1 2 3 4)` as the inverse. Show a disjoint three-cycle and two-cycle,
then a clock with periods three and two meeting again after six steps.

**Voice:** To invert a cycle, reverse its arrows. For disjoint cycles, each
collection of labels moves independently. A permutation made of a three-cycle
and a disjoint two-cycle first returns to the identity after six repetitions,
the least common multiple of three and two. It cannot return earlier: both
cycles have to return simultaneously.

**Picture:** Break `(1 4 3 2)` into swaps, displaying
`(1 2)(1 3)(1 4)` under the rightmost-first convention. Verify the action on
each of the four labels using arrows.

**Voice:** Any finite cycle can be built from swaps, also called
transpositions. Here three swaps give our quarter-turn. A different list of
swaps might describe the same permutation. But one remarkable fact survives
every such rewriting: the number of swaps always has the same parity. A
permutation is either even or odd; it cannot be both.

**Picture:** Place four distinct numbers `x1<x2<x3<x4` on the labels. Display
the product of all six differences `xj−xi` for `i<j` as a compact bracketed
product, not six moving formulas. Swap two labels. Highlight the factor for
that pair reversing sign; pair the remaining factors into unchanged products.

**Voice:** Here is a reason the parity is well defined. Give every label a
different number and multiply all differences `xj−xi` with `i` less than `j`.
This product is nonzero. Swapping any two labels reverses its sign: the
difference for the swapped pair changes sign, while the other affected
differences pair up without a net sign change. So a sequence of swaps changes
the sign once per swap. The final permutation fixes the final product, no
matter how we described it. An even-length and an odd-length swap description
would force the same nonzero product to have opposite signs. Impossible.

**Picture:** Show the sign rule on a two-cell board:
`sign(στ)=sign(σ)sign(τ)`, then a small even/odd multiplication table. Display
the identity's plus sign and two negatives multiplying to plus.

**Voice:** Because swap counts add under composition, the sign of a product
is the product of the signs. We have distilled a complicated rearrangement
into one bit of information. That bit loses many details, but it faithfully
tracks whether a rearrangement is even or odd. Later we will see this as a map
between groups, with the even permutations forming a subgroup.

**Transition:** Lay the four rotations beside a four-position clock, aligned
`0↔e`, `1↔r`, `2↔r²`, `3↔r³`. Do not assert that all groups of order four
are the same.

**Voice:** These two groups look different: one is made of numbers on a
clock, the other of movements of a square. Yet their operation tables match.
When can two different-looking groups be considered the same structure?

### Isomorphism: sixth narration sequence

**Picture:** Place the operation table for addition modulo four next to the
table of rotation composition. Corresponding entries illuminate together as
`0,1,2,3` are relabeled `e,r,r²,r³`. Verify several entries with moving cards,
not merely with a global color change.

**Voice:** Map zero on the clock to the identity move, one to the quarter-turn,
two to the half-turn, and three to the three-quarter-turn. Each clock position
has exactly one partner and every rotation has a partner. If we add on the
clock and then translate the answer, we get the same rotation as if we
translated both inputs first and composed the moves. In symbols, our map takes
`a+b` to `r` raised to `a+b`, which is `rᵃrᵇ`. We can work either problem and
get the corresponding answer in the other picture.

**Picture:** Draw two paths between input pair and output: combine then map,
or map both then combine. Display `φ(a+b)=φ(a)φ(b)` and a one-to-one matching
of all four elements.

**Voice:** An isomorphism is a bijection that preserves the operation. The
bijection matters: if two distinct elements collapse into one, we have thrown
information away. Preservation matters too: an arbitrary relabeling may match
the objects while scrambling how they combine. An isomorphism does both jobs,
so these two groups are the same as groups even though a clock position is not
literally a physical rotation.

**Picture:** Contrast the additive four-clock with `U(12)`. Highlight the
clock's element of order four, and then show that every nonidentity element of
`U(12)` has order two. Cross out a proposed structure-preserving bijection.

**Voice:** Equal size is not enough. An isomorphism preserves element orders:
if repeated application first reaches the identity after `m` steps before the
map, the same is true afterward, because the map is bijective and preserves
products. Our four-clock has an element of order four. The units modulo
twelve do not. Therefore these two groups cannot be isomorphic. A mismatch in
an invariant is an efficient way to rule out an alleged equivalence.

**Picture:** For each element `g` of a finite group, draw a machine `L_g`
that sends any element `x` to `gx`. Demonstrate that `L_g` is reversible via
`L_(g⁻¹)`. Then compose `L_h` followed by `L_g`, and reveal `L_(gh)`.

**Voice:** In fact every group can be viewed as a group of permutations of its
own elements. For each `g`, let `L_g` send `x` to `gx`. This is a permutation:
left multiplication by `g` inverse reverses it. Composing the machines for
`h` and then `g` gives the machine for `gh`. And different elements give
different machines, because applying `L_g` to the identity reveals `g`.
So `g ↦ L_g` is an isomorphism from the original group onto a subgroup of a
permutation group. This is Cayley's theorem. A group need not start as a set of
motions; its own multiplication can always be represented as permutations.

**Picture:** Show the map `x↦−x` on the four-clock, pair `1` with `3`, and
leave `0` and `2` fixed. Illustrate its preservation of addition.

**Voice:** An isomorphism from a group to itself is an automorphism. Reversing
the four-clock, sending each position to its negative, is one example. It
changes labels while preserving the group's structure. We will need this
distinction between a change of label and a change of structure when we
return to symmetries and their quotients.

### Cosets and the counting theorem: seventh narration sequence

**Picture:** Return to the eight square symmetries. Let `H={e,r²}`. Show the
four left cosets as separate cards:
`H={e,r²}`, `rH={r,r³}`, `sH={s,r²s}`,
`rsH={rs,r³s}`. Generate every card by multiplying both members of H on the
left by the label, using the declared rightmost-first convention.

**Voice:** Choose a subgroup `H`, here the identity and the half-turn. For a
move `g`, take every member of `H` and multiply it on the left by `g`. The
resulting set is the left coset `gH`. With this subgroup, the eight square
symmetries fall into four pairs. Some different labels give the same coset:
`eH` and `r²H` are the same pair. But two cosets cannot overlap in only one
element. Why?

**Picture:** Two translucent coset cards `gH` and `kH` overlap at `x`. Reveal
`x=gh₁=kh₂`, so `k⁻¹g=h₂h₁⁻¹ ∈ H`. Then take an arbitrary `gh` and rewrite it
as `k(k⁻¹g)h`, which belongs to `kH`. Reverse the roles for the other inclusion.

**Voice:** Suppose the two cosets share `x`. Then `x=gh₁=kh₂` for members
`h₁` and `h₂` of `H`. Rearranging tells us `k` inverse times `g` belongs to
`H`. Take any element `gh` of the first coset. It is `k` times the product of
`k⁻¹g` and `h`, and that product is in `H`. So every element of the first
coset lies in the second. Reversing the same argument gives the other
inclusion. Overlap forces equality. Every group element lies in its own coset,
so the distinct cosets partition the entire group.

**Picture:** Connect each `h∈H` to `gh∈gH` with a line. Show the inverse
arrow labeled `g⁻¹`. Duplicate the size marker `|H|` across every coset,
and count the coset cards.

**Voice:** Multiplication by `g` gives a bijection from `H` to `gH`; multiply
by `g` inverse to undo it. Every coset therefore has exactly the size of `H`.
For a finite group, count the whole group one coset at a time: the group order
is the number of cosets times the subgroup order. This is Lagrange's theorem.
In the picture, eight equals four pairs times two moves per pair.

**Picture:** Select one group element `g` and draw the cycle generated by its
powers. Apply the same partition count to `⟨g⟩`. Write
`ord(g) divides |G|`. Then show the implication with its arrow pointing only
one way: `subgroup of order d ⇒ d divides |G|`.

**Voice:** The powers of an element form a subgroup, so an element's order
divides the order of a finite group. In particular, repeatedly applying any
symmetry eight times must return the square to its starting arrangement. But
Lagrange's theorem does not promise a subgroup for every divisor of every
finite group order. For cyclic groups that stronger statement holds, as our
clock calculation can show; for arbitrary groups it need not.

**Picture:** Put four square symmetries in one coset label, and four in the
other, using the rotation subgroup `N={e,r,r²,r³}`. Show the four elements
visibly beneath each label; leave any multiplication of coset labels for the
later quotient chapter.

**Voice:** Cosets let us count, but they do not automatically form a new group
under representative multiplication. Before we try that operation, we need to
know whether changing a representative can change the answer. First, though,
let's see how groups combine when we keep two independent motions at once.

### Direct products: eighth narration sequence

**Picture:** Two clocks sit side by side, one with four positions and the
other with three. Each state is labeled by a pair `(a,b)`. Lay out the twelve
pairs in a four-by-three grid. Add `(1,1)` repeatedly, tracing all twelve
states before returning to `(0,0)`.

**Voice:** Suppose one device has four settings arranged in a cycle and
another has three. A state of the combined device needs two coordinates. We
can change the first setting and the second independently, so we add pairs
component by component. This gives the direct product `Z/4Z × Z/3Z`, with
twelve elements. Starting at `(0,0)`, keep adding `(1,1)`. The first clock
returns every four steps and the second every three. Both first return
together after twelve. One element visits all twelve states.

**Picture:** Construct a four-by-three correspondence between the twelve
positions of a single clock and the pairs `(k mod 4,k mod 3)`. Point out that
the 12-step orbit already contains all pairs. Animate addition in both views.

**Voice:** Map a twelve-clock position `k` to its two remainders, one modulo
four and one modulo three. This preserves addition. Its values run through all
twelve pairs without repetition, so it is an isomorphism. The two small
clocks together have the same group structure as one clock of size twelve.
This example is a window into the Chinese remainder theorem, not a proof of
every version of that theorem.

**Picture:** Replace the clocks by two two-position switches. Show their four
states and double each nonzero state to get `(0,0)`. Compare with the
four-clock, whose step one has order four.

**Voice:** But combining two clocks does not always make the one large clock
of their total size. Two two-position switches give four states, yet every
nonidentity state returns after two repetitions. The four-clock has an element
of order four. They are different groups. The distinction comes from the
actual periods, not just the number of states.

**Picture:** The external product starts as pairs chosen from two separate
groups. Contrast with two subgroups drawn within a larger group; label the
second picture “additional conditions needed” until worked out elsewhere.

**Voice:** This construction starts with two groups and makes a new one from
pairs. That is an external direct product. A separate question asks whether
an existing group can be decomposed internally in this way. The answer needs
conditions on its subgroups and their interaction; simply finding two
subgroups of the right sizes is not a proof.

### Normality and quotient groups: ninth narration sequence

**Picture:** Restore the square group and `H={e,s}`, where `s` is horizontal
reflection. Show `H` and `rH={r,rs}`. Use the earlier corner permutations and
rightmost-first convention throughout. Group `e` and `s` under the label `H`.

**Voice:** Let's try to combine cosets the obvious way. To multiply `gH` by
`kH`, choose one move from each, multiply the moves, and use the coset that
contains the result. For this to define a real operation on classes, the
answer cannot depend on which members we chose.

**Prediction:** In the first class choose either `e` or `s`; in the second
choose `r`. Compare `er` and `sr`. Hold before showing coset membership.

**Picture:** Reveal `er=r∈rH`. Show `sr=r³s`, derived from the actual corner
permutations, in `r³sH={r³s,r³}`. The two answer cards are different; do not
show an equal sign between `rH` and `r³sH`.

**Voice:** The identity and horizontal reflection lie in the same class.
Using the identity gives `r`, in `rH`. Using the reflection gives `sr`, in
a different left coset. The class label alone does not determine the answer.
The proposed multiplication is not well defined.

**Picture:** Show a rotation changing the horizontal reflection axis to the
vertical axis under `rsr⁻¹`; display `rsr⁻¹=r²s`. The latter is not in
`H={e,s}`.

**Voice:** The geometric issue is visible. When we change viewpoint by a
quarter-turn, the horizontal reflection becomes a vertical one, which this
subgroup did not include. A subgroup is normal when every conjugated move
`g h g⁻¹`, for `g` in the group and `h` in the subgroup, stays in the
subgroup. Equivalently, every left coset `gH` equals the corresponding right
coset `Hg`. Our reflection subgroup fails this test.

**Picture:** Draw two representative changes `g' = gn₁` and `k' = kn₂`,
where `n₁,n₂∈N`. Starting from `g'k'`, visibly move `n₁` across `k` using
`k⁻¹n₁k∈N`: `gn₁kn₂ = gk(k⁻¹n₁k)n₂ ∈ gkN`.

**Voice:** Why does normality fix the representative problem? Suppose `N` is
normal, and replace `g` by `gn₁` and `k` by `kn₂`, where both added factors
are in `N`. Their product is `gn₁kn₂`. Insert `kk⁻¹` to move the first added
factor past `k`: the product becomes `gk` followed by `k⁻¹n₁k` and `n₂`.
Normality puts the conjugated factor in `N`, and closure keeps their product
in `N`. So every choice gives the same coset `gkN`.

**Picture:** Reverse the argument. Multiply the identity coset N by `gN` using
two representatives of N: `e` and `n`. Well-definedness requires `ngN=gN`;
therefore `g⁻¹ng∈N`. Label this “for every g and n”.

**Voice:** The condition is also necessary. If multiplying cosets is
well defined, choosing `e` or any `n` from the identity coset before `g` must
lead to the same output coset. Thus `ngN=gN`, which says `g⁻¹ng` is in `N`.
That must hold for every `n` and `g`, which is normality. We have not just
found a useful trick; we have identified exactly the condition needed for this
operation to be unambiguous.

**Picture:** Choose the rotation subgroup `N={e,r,r²,r³}`. Show its two
cosets, rotations N and reflections `sN`, as groups of four. Animate the
two-by-two table: `N·N=N`, `N·sN=sN`, `sN·N=sN`, `sN·sN=N`.

**Voice:** All rotations form a normal subgroup of the square group. There
are two cosets: rotations and reflections. Because the subgroup is normal,
we can multiply the cosets. Two rotations give a rotation; composing a
rotation and reflection gives a reflection; composing two reflections gives a
rotation. The resulting quotient group has two elements and is isomorphic to
the two-position clock.

**Picture:** A determinant sign display shows +1 for the rotation cards and
−1 for the reflection cards. A two-step arrow maps each square symmetry to
its sign and shows multiplication preserved; its positive-sign kernel is N.

**Voice:** The quotient has forgotten which rotation or which reflection we
used. It retains only orientation: rotation has sign plus, reflection sign
minus. Multiplying transformations multiplies those signs. The moves sent to
plus are exactly the rotation subgroup. This is a concrete bridge to
homomorphisms and kernels; the full general isomorphism theorem is a further
result, not something established by this one example.

**Picture:** Show the original square with asymmetric markings and its mirror.
Rotate one through all four orientations. Separately show an unmarked square,
whose reflection looks identical; label the difference between a
transformation and an observed image.

**Voice:** This distinction can matter in graphics or vision: a mark may be
the same up to rotation yet different from its mirror. But a completely
unmarked square looks the same after either operation. The quotient is a
statement about transformations; what an image reveals depends on its marks.

### An error-detection application and synthesis: tenth narration sequence

**Picture:** Start with data `101`. Add one parity bit, giving `1010`, so the
number of ones is even. Send it across a channel. Let the last data bit flip,
giving `1000`. The receiver counts one one and raises a warning.

**Voice:** Let's use a group to understand a small error-detecting code. To
send three bits, add one more bit so the total number of ones is even. For
example, `101` becomes `1010`. Suppose the third bit changes on the way. The
receiver sees `1000`, whose number of ones is odd, so it knows an error has
occurred. This check detects every single-bit flip. It does not tell us which
bit flipped, and by itself it cannot repair the message.

**Picture:** Show another transmission of `1010`, but flip the first and
third bits. The receiver gets `0000`, still even. Display “two errors can
escape this check.”

**Voice:** If two bits flip, parity changes twice and returns to even. For
instance, `1010` can become `0000`, another valid even-parity word. The
receiver cannot detect that change with this test alone. No claim about
correcting two errors follows from detecting one.

**Picture:** Represent four-bit strings as elements of `(Z/2Z)⁴`, adding
coordinate by coordinate. Color the eight even-parity strings blue and the
eight odd-parity strings gold. The parity map adds the four coordinates modulo
two and sends each color to 0 or 1.

**Voice:** Now the abstract structure explains the calculation. Four-bit
strings form a group under coordinatewise addition modulo two. The parity
map sends each string to the sum of its four bits modulo two. It preserves
addition. The strings sent to zero form its kernel: the even-parity subgroup.
There are two cosets, even and odd, and they form a two-element quotient. A
single bit flip adds an odd-parity error, moving a valid word to the other
coset. A two-bit flip adds an even-parity error and stays in the same coset.
The quotient captures exactly the one distinction this check can see.

**Picture:** Return to the square, clock, permutation arrows, coset partition,
and parity strings. Connect each to a question: combine, undo, generate,
relabel, count, forget, detect. Bring back the opening equivalence lanes.

**Voice:** The pictures may look unrelated: clock positions, moves of a
square, permutations, and bit strings. Group theory lets us ask the same
questions in each setting. Which moves combine? Which can be undone? Which
elements generate a smaller group? When do two descriptions have the same
structure? Cosets let us count. Normal subgroups tell us when we can forget a
chosen class of differences and still combine the classes without ambiguity.
The parity code makes that last idea practical: we deliberately keep one bit
of structure, because it is the bit that can reveal a single error.

**Picture:** Offer three exercises, leaving blank answer space. After separate
pauses, reveal short solutions with the relevant visual, not just a check mark.

**Voice:** Try three questions. First: in the additive twelve-clock, what is
the order of nine? Second: can a subgroup of order three exist in a group of
order eight? Third: does an even-parity check detect every error that flips
exactly three bits? Pause and work them out.

**Picture:** Reveal in turn: `12/gcd(12,9)=4`; `3` does not divide `8`;
three flips change parity once modulo two, so they are detected.

**Voice:** Nine has order four. A subgroup of order three cannot exist in a
group of order eight, by Lagrange's theorem. And three bit flips do change
parity, so the even-parity check detects them, though it still cannot say
which bits changed. The same structure that began with an ordinary clock now
lets us make a precise prediction about a communication system.

## Completion requirements

- Finish and review an original full narration and timed visual storyboard for
  every chapter above, with worked exercises, proofs, and consistent notation.
- Independently check finite computations and verify each mathematical claim;
  independent human mathematics/pedagogy review remains open.
- Implement native FrankenManim scenes, keep existing public cut reproducible,
  produce complete ElevenLabs takes and align pictures to those takes.
- Run guarded 4K mastering, caption and technical checks; inspect actual final
  pictures and verify public 4K/manual captions only after the treatment meets
  this scope. Continuous listening and fine-sync acceptance remain unverified;
  ASR is a screening aid, not proof of either.
