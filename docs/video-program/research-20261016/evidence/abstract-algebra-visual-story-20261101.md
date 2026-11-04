# A visual introduction to abstract algebra

Owner request, 2026-11-01. Working title: **The Secret Structure of Symmetry**. Inspired by the progression of Joseph A. Gallian's *Contemporary Abstract Algebra*, but written and drawn as an original explanation, not a chapter-by-chapter adaptation of protected text or artwork. In the publisher's [ninth-edition contents](https://prod.cengageasia.com/title/default/detail?isbn=9781305657960), equivalence relations are preliminary, groups and subgroups precede cyclic and permutation groups, cosets are chapter 7, and normal subgroups and factor groups are chapter 9. The requested journey therefore reaches beyond literally the first few chapters. Edition not specified by owner; don't claim the ninth edition is their copy.

**Promise:** start with a square you can turn and flip. Discover a group as a system of reversible actions, find which moves cooperate with a chosen move, partition the moves into equivalence classes, and see why some partitions can be multiplied while others cannot. Use one labeled square throughout, with a clock as an extra comparison. Link individual visual claims to the algebra; no assumed novelty, video production, independent review or listening acceptance.

## One precise convention

Let `D₄` mean the eight symmetries of a square; other books use `D₈` for this same eight-element group. Let `r` be a counterclockwise quarter turn and `s` reflection across the horizontal axis. Write a product `ab` for “do b first, then a.” Every move is uniquely `rᵃsᵇ`, `a ∈ {0,1,2,3}`, `b ∈ {0,1}`; `r⁴=e`, `s²=e`, `sr=r⁻¹s`. All examples below use this convention. Label each corner so flipping a symmetric outline does not hide what moved.

## Draft narration and scene sequence

### 1. The puzzle: a square has eight ways to look unchanged

Picture: a square with differently colored numbered corners. Demonstrate rotation, flip, identity, and return to original position. Keep a ghost outline so “unchanged” means the unmarked square occupies the same region; the labels track which symmetry occurred.

> “Turn a square a quarter turn and its outline still fits the same frame. Flip it across a line through its center and it fits again. Mark the corners and you can tell the moves apart. There are eight ways to take the square onto itself: four rotations and four reflections.
>
> “What if we stop asking what the square is made of and instead ask how its moves combine? That change of question opens the door to abstract algebra.”

Picture sequence shows all eight, derived from the actual transforms. A square with distinguished colored corners is used as a tracking device; its color pattern itself is not claimed invariant under every symmetry.

### 2. The rules of a group are rules for combining actions

> “Do one symmetry and then another. The result is another symmetry. There is a do-nothing move. Every move can be undone by another move. And the way we group a sequence of three moves does not change their order or their result. Those are the group rules: closure, identity, inverses and associativity.
>
> “Associativity doesn't mean we may swap moves. Try a quarter turn followed by a flip. Now undo that and try the flip followed by the turn. The square lands in two different labeled arrangements. This group is noncommutative.”

Show `sr=r³s` while `rs` is different. In the displayed `ab` convention, first animation for `sr` executes `r`, then `s`; second for `rs` executes `s`, then `r`. A table or a computed permutation demonstrates the actual final corner positions. Function composition is associative; finite closure, identity and inverse are checked on all eight. A group need not be finite, consist of transformations, or be commutative.

### 3. Subgroups, cycles and an order that means two things

> “Keep just the rotations. The do-nothing move, a quarter turn, a half turn and a three-quarter turn still form a group under the same operation. That smaller group is a subgroup.
>
> “Starting from the identity and repeatedly applying a quarter turn visits all four rotations before returning home. One move generates the whole rotation subgroup, so it is cyclic. The order of that move is four: the first positive number of repetitions that returns us to identity. The order of the full square group is eight: the number of its moves. Same word, two related but different counts.”

Display `<r>={e,r,r²,r³}` and the square group size 8. Mention an infinite group need not have finite order and an element need not have finite order. A subgroup must contain the identity and be closed under products and inverses, not merely be any subset.

### 4. Centralizer: which moves can change places in a sequence?

> “Now fix the horizontal flip. Which square symmetries give the same result whether we do them before or after that flip? Those moves form its centralizer.
>
> “The identity works. So do the half turn, the horizontal flip itself, and the vertical flip. But a quarter turn does not: it changes which reflection line we are using. The centralizer gathers exactly the moves that commute with this chosen move.”

Show `C_D₄(s)={e,r²,s,r²s}`; `r²s` is the vertical-axis reflection under our convention. Distinguish the centralizer of an **element** from the center `Z(G)`, which contains elements commuting with *every* group element. Here the center is `{e,r²}`. Show before/after for `r` and `r²`. Real use: if two operations commute, either order yields the same transformation; do not assert unrelated physical operations commute without modeling their action.

### 5. Equivalence classes and cosets: decide what differences to ignore

Picture: clock hands at 1, 5 and 9 on a four-position dial; then return to the eight square moves. Highlight that equivalence has reflexive, symmetric and transitive requirements.

> “A clock identifies numbers that differ by a whole number of turns. One, five and nine all land on the same position on a four-step clock. That is an equivalence class: a collection of things we have decided to treat as the same, according to a rule that is reflexive, symmetric and transitive.
>
> “A subgroup gives us a way to make classes of group elements. Say two moves are equivalent when one differs from the other by a move inside the subgroup. The classes are called cosets. If the group is finite, every coset has the same size as the subgroup, and the cosets partition the group.”

Use `g∼h` iff `g⁻¹h∈H`, which produces left cosets `gH`. For `H=<s>={e,s}`, the four left cosets are `H`, `rH`, `r²H`, `r³H`, two elements each. Do not call this a quotient group yet. Explain `|G|=number of cosets × |H|` only for finite G, with its counting reason. A left coset `gH` is a set, generally not a subgroup; do not say all cosets themselves are subgroups.

### 6. The quotient test: does the answer depend on the representative?

Picture: first demonstrate failure using the actual eight-move model, then switch to a normal subgroup. Color one class at a time; show two choices of representative and resulting class, not just a table with unexplained colors.

> “We might try to multiply two classes by choosing one representative from each, multiplying those moves, and asking which class contains the result. But that only makes sense if the answer does not depend on our choices.
>
> “For the subgroup containing just the identity and our flip, it can depend on the choice. The identity and the flip lie in the same class. Multiply the class of the quarter turn using the identity, and you get the class of the quarter turn. Use the flip instead, and you get a different class. So this proposed class multiplication is ambiguous.
>
> “A normal subgroup is exactly the kind for which this multiplication is well defined. For the square, take all four rotations. There are two classes: rotations and reflections. Two rotations give a rotation; a rotation and a reflection give a reflection; two reflections give a rotation. These two classes form a quotient group.”

Explicit failure using left cosets and right-to-left composition: `H=eH=sH`, `e·r=r∈rH`, but `s·r=r³s∈r³H`, and `rH≠r³H`. Both e and s are legitimate representatives of H; r is a representative of rH. Therefore naive `(aH)(bH)=(ab)H` is not well-defined for H. For `N=<r>`, `N` is normal (index two, or verify `gNg⁻¹=N`); `D₄/N={N,sN}≅C₂`. Orientation sign `det:D₄→{+1,−1}` has kernel N; `+1` for rotations, `−1` for reflections. The quotient keeps orientation while discarding which particular rotation occurred.

**Bridge the missing why, in pictures:** Show the sequence `r`, then `s`, then `r⁻¹`: the result is `r⁻¹sr=r²s`, the vertical reflection. The proposed subgroup `H={e,s}` contains the horizontal reflection but not this vertical reflection: changing viewpoint has carried an allowed difference outside H. For the four rotations N, by contrast, `g⁻¹ng` is always a rotation whenever g is any square symmetry and n is a rotation. That is the normality test in this example, not a claim that the individual rotations stay unchanged. Visually carry a small highlighted “ignored difference” through the middle move, showing it may change but must remain within the set of ignored moves.

> “The problem is not that the representative changed. The problem is that, after another move changes our point of view, the difference we agreed to ignore can turn into a difference we did not agree to ignore. A normal subgroup keeps every one of those transported differences inside the same subgroup. Then all choices of representative lead to the same class.”

**Precise version for review, not mandatory on-screen notation:** If `a'=ah` and `b'=bk` with `h,k∈N`, then `a'b'=ab(b⁻¹hb)k`; normality puts `b⁻¹hb` in N, so both products lie in `abN`. Conversely, if class multiplication is independent of representatives, `H=hH` implies `(hH)(gH)=(eH)(gH)`, so `hgH=gH` and `g⁻¹hg∈H` for every h and g. Applying this inclusion to g and its inverse gives equality and normality. This explains both directions of the claim; [Judson's quotient construction](https://math.libretexts.org/Bookshelves/Abstract_and_Geometric_Algebra/Abstract_Algebra%3A_Theory_and_Applications_%28Judson%29/10%3A_Normal_Subgroups_and_Factor_Groups/10.01%3A_Factor_Groups_and_Normal_Subgroups) supplies an external check for the normality equivalences and sufficiency. Independent review remains open.

**Interactive visual now built:** `abstract-algebra-symmetry-prototype-20261101.html` lets the viewer choose representatives in `H × rH`, then switch to `N × sN` without changing the underlying square. It highlights the resulting class and identifies each box as a set of transformations. Browser author checks observed `e·r=r` and `s·r=r³s` in different classes, and `e·s=s` and `r·s=rs` in the same class. A separate author calculation composed explicit corner permutations: all four choices from `H × rH` occupy two result cosets, while all 16 choices from `N × sN` land in `sN`. This latter single-class-pair demonstration illustrates, but does not by itself prove, the general normal-subgroup theorem. The prototype still needs fresh-reader, accessibility and independent mathematics review; it is not native video footage.

The prototype now also offers both classes in each input, with the complete two-by-two quotient table: rotations followed by rotations and reflections followed by reflections give rotations; mixed pairs give reflections. The author checked all four class selections in the desktop browser and composed all 64 representative pairs using corner permutations: each of the four class pairs produces exactly the four transformations in its stated result class. This checks `D₄/N`, not all possible quotient groups. A reader-facing script should say **quarter-turn angle** rather than simply “orientation”: determinant orientation or handedness is what the quotient retains.

### 7. What does any of this buy us?

> “Suppose a vision system needs to recognize a mark regardless of how many quarter turns it has been given. Rotations of the same image can be grouped into an equivalence class. But a mirror image may matter: an arrow or a letter can change meaning when flipped. The rotation subgroup lets us ignore quarter-turn angle, while the two-class quotient distinguishes rotations from reflections.
>
> “The same language studies molecular symmetries, error-detecting codes, and transformations in engineering. Those applications have different groups and assumptions; the square is our model, not a proof that every real object has these eight symmetries. Gallian's book itself includes a check-digit application based on a different dihedral group.”

Clarify the vision model: orbit equivalence of an image under rotations is an action on images, not itself the quotient group `D₄/N`. `D₄/N` is a two-element group of **transformation classes**; it can act on rotation-orbits because N is normal. A mark can have additional internal symmetry; then fewer than four distinct rotated images may occur. This is an illustrative model, not a benchmark or claim that a particular deployed system uses it. [Publisher's contents](https://prod.cengageasia.com/title/default/detail?isbn=9781305657960) lists a D₅-based check-digit scheme, but no implementation or specific error-detection guarantee is inferred.

### Close

> “We began with moves of a square. A group let us compose them. A centralizer asked which moves can trade order. An equivalence relation let us group moves into classes. A normal subgroup made those classes into a new group without ambiguity. Abstract algebra gives us a precise way to decide which differences matter, and which ones we can safely forget.”

## Sources and comparison

- [Publisher's ninth-edition contents](https://prod.cengageasia.com/title/default/detail?isbn=9781305657960); this is a description/contents check, not a complete reading of the book.
- [Judson, normal subgroups and factor groups](https://math.libretexts.org/Bookshelves/Abstract_and_Geometric_Algebra/Abstract_Algebra%3A_Theory_and_Applications_%28Judson%29/10%3A_Normal_Subgroups_and_Factor_Groups/10.01%3A_Factor_Groups_and_Normal_Subgroups) explains why representative-independent multiplication needs normality. [Centralizers and cosets](https://math.libretexts.org/Bookshelves/Abstract_and_Geometric_Algebra/Introduction_to_Groups_and_Geometries_%28Lyons%29/02%3A_Groups/2.03%3A_Subgroups_and_cosets) defines a centralizer.
- [Mathemaniac, “Chapter 5: Quotient groups”](https://www.youtube.com/watch?v=N11vOq3h9XQ), 12:36, is directly relevant. Four muted frames and their visible captions were inspected: at 2:31 a colored coset diagram already discusses choosing different representatives; at 5:02 a diagram names the quotient as a group of cosets; at 7:34 the video shows an integer-line composition; at 10:05 it presents normal subgroups of the integers and integer quotients. The first observation directly overlaps our representative-choice explanation, so do not claim that idea is novel. The proposed distinction is a single physical square carried from initial actions through an explicit failed product and a normal quotient, plus the same-square normality bridge above; its distinctiveness and teaching value remain hypotheses. The transcript panel remained loading. Four sampled frames are not a full visual review, a continuous viewing, or listening.
- [Oxford Mathematical Institute's square-symmetry introduction](https://www.maths.ox.ac.uk/outreach/oxford-online-maths-club/season-3-episode-2) already uses square moves, compositions, the eight-element group and group rules; this observation is from its associated written page, not a claim of full video viewing. [Nathan Carter's Visual Group Theory](https://nathancarter.github.io/vgt.html) already presents groups, subgroups and quotients visually in an illustrated textbook. An interactive comparison of valid and invalid representative choices on one square is our proposed execution; neither a unique idea nor superior learning is established by these limited checks. Compare exact examples and visual sequencing after actual competitor audiovisual review.

## Open work

- [ ] Independently verify every multiplication, orientation, centralizer, coset and quotient condition and audit the applications.
- [ ] Review existing treatments audiovisually, check differentiation, and test the narrative with a fresh reader.
- [ ] Translate the labeled-square and representative-choice prototype and its complete quotient multiplication into native footage; make a genuine side-by-side noncommuting comparison. Avoid confusing a group action's orbits with a quotient group.
- [ ] Prepare complete narration using ElevenLabs; preserve exact takes and settings, verify pronunciation and obtain real listening and fine-sync review.
- [ ] Follow existing guarded resource admission and perform actual picture, captions, technical and independent checks before final acceptance. Preserve existing files and work on the other requested videos; no publication is claimed.
