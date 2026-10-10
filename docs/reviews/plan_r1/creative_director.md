Plan review round 1 (2026-10-09), creative direction, against plan revision 1. Findings are being dispositioned in the plan's revision 2.

# Creative-direction memo: the best explanation for videos A, B and C

**Verdict.** All three storyboards are sound and well researched. Each one already has a good chapter-by-chapter pace. What none of them has yet is one idea that makes the central result feel *inevitable* rather than merely verified. I found one such idea per video. Each is mathematically exact, I checked each by script, and each turns the video's surprise into something the viewer could have predicted:

- **A:** Feller's leftover term Mn − kN is the cross-product of the two catches' 2×2 table, so maximum likelihood "fills the one cell nobody saw" until the table looks independent.
- **B:** random coefficients mod 256 fail *exactly* as often as XOR, 71%. The danger in the lottery lives entirely in the last few equations, and that predicts both RaptorQ's design and its ablation result.
- **C:** every read the proof trusts happens while nothing is being written. Garbage reads are therefore harmless *by construction*, and that becomes visible if chapter 08 is drawn on top of chapter 07's proof picture.

I also found five factual or framing slips in the draft narration. They are listed in the tables.

---

## Video A — How Many Fish Are in the Lake?

### (a) Highest-leverage changes, ranked

**A1. Make the hero picture the lake's four-cell table, with one dark cell. Open on it.**
The dye picture explains *why* the method is legitimate: mixing makes the catches independent. It does not show *what* is being estimated. The four-cell table does, and it carries the whole video:
- the cold open;
- Feller's algebra (A3);
- the failure modes (A8);
- the code-review Venn;
- Kosovo, which is the same table with four lists: 16 cells, one dark.

*Cold open (replaces 01_1–01_2; the 10,000 now arrives within 40 s).* The second net comes up. First-catch fish glow gold, second-catch fish green, recaptures both. Every other fish fades to a dark silhouette. The lake then reorganizes into a 2×2 table: **100 | 900 / 900 | ?**

> "A thousand fish are caught, marked with a red spot, and let go. A week later, another thousand are caught. A hundred of them have spots. Between the two nets, we've seen nineteen hundred different fish. Now look at the fish no net ever touched. We can say there are about eight thousand one hundred of them, and we can say how sure we are. This is how you count what you can't see."

Title card. Feller's 1950 story and footnote 13 move to 02.

**A2. Recast the hook's failure. The marks are a ruler for the net.**
The line "a method that depends on where the fish are is doomed" (01_5) is overstated. Many random quadrats *do* work, and Ruth and Dev will both say so. The honest failure is that one net cannot tell you how much of the lake it searched. The marks fix exactly that. The symmetric reading k/M ≈ n/N is missing from the plan, and it answers the hook directly.

*02 (new), after the three dips:*
> "Why not count one patch and scale up? Fish crowd into some places and avoid others, and a net can't tell you how much of the lake it really searched. Here, three honest dips disagree by a factor of thirty. *(P1 value)* The marks fix exactly this. Our net caught a hundred of the thousand marked fish. One in ten. So it probably caught about one in ten of all the fish. The marked fish measured our net for us. Read the same fact the other way. One fish in ten in our catch is marked, so about one in ten in the lake is. Same equation, two readings."

*Picture:* the dip estimates on a log axis. Then the k/M bar ("net caught 10% of the marked fish") morphs into n/N ("…so ~10% of the lake").

**A3. Feller's ratio is the determinant of the table. This is the goosebumps moment.**
Write the cells as overlap k, marked-only a = M−k, caught-only b = n−k, and unseen u = N−M−n+k. Then **Mn − kN = ab − k·u** exactly. The likelihood rises in N exactly while k·u < ab, so the MLE is a + b + k + ⌊ab/k⌋. That is the table whose cross-products balance, which is what independent catches produce in expectation.

Checks: for Feller's numbers, 900·900/100 = 8,100, and at N = 10,000 the ratio is 81,000,000/81,000,000. For the toy, 49/3 → u = 16 → N = 33. Both agree with the facts ledger.

*06_3–06_6 rewrite:*
> "Most of the counting cancels, and what's left decides everything. M times n, minus k times N. Now write it with the four cells. It becomes the marked-only fish times the caught-only fish, minus the overlap times the dark cell. So the likelihood keeps climbing as fish are added to the dark cell, until overlap times dark cell catches up with the two side cells multiplied together. Nine hundred times nine hundred, over a hundred. Eight thousand one hundred. That balance is exactly what two independent catches look like. The careful argument and the soup argument are the same argument."

*Picture:* the table's two diagonals glow. A seesaw has k·u on one pan and a·b on the other. The dark-cell counter ticks up until the beam is level at 8,099 and 8,100 (the tie). The motif resolves here.

*Lower-third:* "precisely: MLE of the unseen cell is ⌊ab/k⌋; a tie when ab/k is whole".

**A4. Draw the crest curve on the 3D landscape (05b). One hyperbola threads the math chapters.**
On the floor of the P(k|N) landscape, draw the curve k = Mn/N, where the catch's fraction equals the lake's. Feller's ratio makes this exact: every fixed-k likelihood slice rises to the left of the curve and falls to its right. The fixed-N probability slices peak *next to* it, at the mode ⌊(n+1)(M+1)/(N+2)⌋. The same curve then becomes the funhouse mirror of the simulation chapter. So the proportion argument (03), the landscape (05), the ratio (06) and the skew (08) are visibly one curve. This also gives the orbiting camera a job: rotate until the curve lines up with each slicing plane.

> "Along the floor runs one curve, where the catch's fraction equals the lake's. Slice at what we saw, and the likelihood climbs until it meets the curve, then falls. That curve is the formula M n over k. We'll meet it again."

**A5. Swap chapters 07 and 08: simulate first, then invert.**
"Ten thousand lakes" shows the spread when N is *known*. "How sure" builds the interval when N is *unknown*. Putting the simulation first repeats chapter 05's "turn the question around" move, the intervals become an inversion of something the viewer has just watched, and the stacked-intervals picture (07_6) stops looking like a second simulation. Cut 07_2/07_3 (the 1,900-fish and million-fish lakes) to one line.
> "We knew the lake and watched the estimates scatter. Now turn it around, like before. We don't know the lake. Which sizes could plausibly have given us a hundred?"

**A6. Fix the physics of the signature picture.**
Curl-noise currents are divergence-free. They would homogenize a clumpy density, which is the opposite of what 03_2 claims. Use overdamped motion in a habitat potential U, with drift −∇U plus a circulation c·J∇U along the contours of U, plus noise. The stationary density ∝ e^(−U) stays clumpy, because J∇U ⊥ ∇U and the curl of a gradient is zero. Tagged fish relax to the *same* density, so the tagged-fraction map flattens while the density map does not.

Streamlines drawn along U's contours then *mean* "fish circling their haunts" instead of decoration. Compute both maps from a larger simulated population, or use coarse 3×3 regions. With about 100 rendered gold fish, per-region fractions would be too noisy to show the flattening.

**A7. Use the symmetric precision formula, and soften one line.**
The plan shows SD(k) = 9.000 (exact hypergeometric) next to CV ≈ √(1/k − 1/n) = 9.5% (the binomial approximation). That pair is inconsistent. Use CV ≈ √((1−M/N)(1−n/N)/k) = 9.0%. It is symmetric in the two catches, which matches the table.

Line 07_7, "Not catching a big share of the lake," should become:
> "Mostly, catching marked fish again. Catching a big share helps a little too. Catch every fish and no doubt is left."

**A8. Give chapter 09 one rule instead of a montage.**
Every failure in the chapter is the catches ceasing to be independent, which distorts the overlap cell, or the lake changing.
> "If the same fish are likelier to be caught twice, the overlap swells, the dark cell shrinks, and the estimate comes out small."

Run each scenario as a distortion of the table, with the gauge reading out the estimate.

**A9. The likelihood-area identity is now proved. Upgrade it from trivia.**
ΣN L(N) = Mn/(k(k−1)) for k ≥ 2. Proof: write 1/C(N,n) = n∫t^(n−1)(1−t)^(N−n)dt and sum the negative-binomial series; this gives n·C(M,k)·B(k−1, M−k+1). Memorable form: **the area equals the estimate divided by k − 1** (33.3/2 = 16.67; 10,000/99 = 101.01). Keep "not 1" in narration and put the identity and proof in the description.

**A10. Coda on the dark cell.**
> "France, the bugs, Kosovo. Each is the same table, with one cell no one can see, filled in by the cells we can."

Use silence, not a quiet bed, under the Kosovo numbers.

**Title:** *How to Count Fish You've Never Seen* (alternative: *Counting the Fish Nobody Caught*).
**Thumbnail:** a 2×2 grid floating on dark water reading **100 | 900 / 900 | ?**, with faint silhouettes in the "?" cell. It is a puzzle viewers can solve, so it is curiosity without bait.

**Sound.** The F-Lydian 6/8 identity fits. Changes:
- Resolve the motif at the table balance (A3), not at the 10,000 in chapter 02, where it is unearned.
- Teaching sonification for chapter 03: nets of different sizes come up, and the *rhythm* of vibraphone (tagged) against kalimba (untagged) plinks keeps the same one-in-ten density. You hear a proportion stay constant.
- During the likelihood slider, a soft tone whose brightness tracks L(N), only in a ≥3 s narration gap.

### (b) Findings — Video A

| id | chapter | severity | issue | proposed change |
|---|---|---|---|---|
| A-1 | brief/01 | major | Hero picture shows mechanism (mixing), not the estimand | Four-cell table with dark cell as hero; cold open on 8,100 unseen (A1) |
| A-2 | 01_5 | major | "doomed" overclaims; random quadrats work | "a net can't tell you how much of the lake it searched"; marks as ruler (A2) |
| A-3 | 06 | major | Algebra lands on proportion but not on a picture | Mn − kN = ab − ku; seesaw on the table's diagonals (A3) |
| A-4 | 05b | major | 3D orbit risks decoration | Crest hyperbola on the floor; slices rise/fall across it (A4) |
| A-5 | 07/08 | major | Two uncertainty chapters overlap; inversion comes before the forward picture | Swap; cut 1,900/10⁶ beats to one line (A5) |
| A-6 | 01/03 | major | Divergence-free currents would flatten density, contradicting 03_2 | Habitat-potential dynamics + contour circulation; larger simulated population (A6) |
| A-7 | 07_7 | minor | CV 9.5% (binomial) beside SD 9.000 (exact); "not a big share" too strong | √((1−M/N)(1−n/N)/k) = 9.0%; "mostly" (A7) |
| A-8 | 08_6 | minor | "together cover the lake" suggests every fish was caught | "when the two catches add up to at least the size of the lake" |
| A-9 | 05_6 / A-F7 | minor | Identity marked unproved | Proved; area = N̂/(k−1); proof in description (A9) |
| A-10 | 09 | minor | Montage of five failures risks a list | One rule: dependence distorts the overlap cell (A8) |
| A-11 | 10_2 | minor | "thirty sample districts" | Communes spread across thirty départements (memo) |
| A-12 | 02_1 | minor | Quote card wording is 3rd ed.; narration says 1950 | Cite the edition quoted |
| A-13 | 10 | idea | Kosovo bed | Silence under the numbers; four-list Venn, 16 cells, one dark |

---

## Video B — Liquid Data

### (a) Highest-leverage changes, ranked

**B1. Chapter 04's punchline: mod 256 is no better than XOR. Exactly.**
Z/256 is a local ring. A matrix over it is invertible iff its determinant is odd, iff the matrix is invertible mod 2. So uniformly random K×K coefficients mod 256 succeed with probability exactly ∏(1−2^−i), which tends to 28.9%. That is the GF(2) number; my Monte Carlo gives 0.288 at K = 8, against an exact 0.290. This is a far stronger chapter hook than "nothing times two gives one". It also sets up chapter 05 visually: the bit that *falls off and is lost* here is the bit that gets *caught and folded back* there.

> "Bytes already multiply. Keep the remainder after two hundred fifty-six. Watch doubling one twenty-eight. The top bit slides off the end, and it's gone. Two times one twenty-eight is zero. So a packet that says two times A is six can't tell you A. It could be three. It could be one thirty-one. Worse, flip the top bit of one piece of the file. Any packet that multiplied that piece by an even number can't notice. Run the lottery with random bytes and this arithmetic, and it fails seventy-one percent of the time. Exactly as often as plain ex-or. Only the last bit of each coefficient was doing anything."

*Picture:* bits slide left and the top bit drops into a dark gutter. Two lottery counters run side by side (XOR and mod 256) and converge to the same 71%.
*Lower-third:* "precisely: invertible over Z/256 ⇔ det odd ⇔ invertible mod 2".
*Chapter 05 bridge:* "So catch the falling bit. Fold it back in."

**B2. "The danger lives at the end." Say it in chapter 03; cash it in chapter 09.**
With i independent equations, the next random one is wasted with probability q^(i−K). Over GF(2) with K = 1,000, the chance that any of the first 990 packets is wasted is under 1/1,000. All the risk sits in the last few slots: 1/8, 1/4, 1/2. That is why the failure rate barely depends on K (GF(2): 50%, 62.5%, 67%, then about 71% from K = 10 onward). Dev will not expect that.

*03, after 03_5, with a "danger meter" showing the lit fraction as the bucket fills (flat near zero, spiking at the end):*
> "Watch the danger as the bucket fills. For almost the whole fill, a random packet is nearly certain to help. All the danger is packed into the last few slots. One in eight. One in four. One in two. So ten pieces or ten million, exactly enough ex-or packets fail about seventy-one percent of the time."

*09 rewrite. This makes the ablation predictable and gives the video its best prediction pause:*
> "Remember where the danger lives? RaptorQ's cheap ex-or equations fill almost the whole space, and by design they leave between ten and sixteen directions open. A few dense byte rows close them. If those rows were ex-or rows, what would you expect? *(pause)* The ex-or lottery all over again. Seventy-one percent. With byte rows, about six failures in a thousand."

Why "by design" is safe: there are S + K′ binary rows for L = K′ + S + H unknowns, so at least H directions stay open. The measured ablation (70.7% against 0.632%) is the evidence. The narration says "behaves like", not "is".

**B3. Explain "each extra packet is worth ×256" with the change nobody notices.**
Decoding fails iff some nonzero change d to the file is invisible to every packet held (Ad = 0). A random GF(256)-weighted packet misses a fixed d with probability exactly 1/256, because c ↦ c·d is uniform on a field. A union bound over directions gives P(fail) < 256^(−h)/255. That bound matches the exact values to three digits (0.3922% / 1.532×10⁻⁵ / 5.984×10⁻⁸). This is what earns the closing line "because a polynomial refuses to factor".

*07, after 07_4:*
> "Decoding fails only if some change to the file would slip past every packet you hold. Take one such change. A random byte-weighted packet misses it only one time in two fifty-six. There are about two fifty-six to the K possible changes. Each must slip past K plus h packets. The two huge numbers cancel, and one over two fifty-six to the h is left. Every extra packet divides the risk by two fifty-six. And that one-in-two fifty-six holds only because no two nonzero bytes multiply to zero."

*Lower-third:* "grouping multiples of a change sharpens this to 256^−h/255". For XOR, the narration adds: "each packet catches a change only half the time, so each extra packet only halves the risk."

**B4. Make a circular slide rule, a byte clock with 255 hours, the Act II hero, and demote the torus.**
This rhymes with chapter 04: *the wrong clock had 256 hours and added; the right one has 255 hours and multiplies.* That is GF(256)* cyclic of order 255, set against the additive group of Z/256. The new set piece is a 1D precursor of the table reorder. The doubling path drawn in byte order is a tangled 255-point star. Each byte then slides to its log position, the star untangles into a circle, and doubling becomes one click.

> "In byte order, doubling makes a tangled star. Move every byte to where doubling first reached it, and the star untangles into a circle. Doubling is one click around. A clock with two hundred fifty-five hours. To multiply, add clicks. One fifty is a hundred eighty clicks. Three is twenty-five. Two hundred five clicks. One sixty-seven."

*Picture:* two concentric rings; the inner ring rotates 25 clicks; read 167 opposite 150.

The 65,536-cell reorder stays as the 2D climax ("every pair at once"). The torus either gets a true sentence ("the clock wraps both ways, so each stripe, all pairs with one product, winds around a doughnut") or is cut to a 5 s flourish.

**B5. Answer Dev's "why not just rotate the bit back?" (10 s, in 05).**
Folding the overflow into bit 0 means reducing mod x⁸+1, which is rotation. Doubling becomes invertible, but **3·255 = 0** there, because (x+1)(x⁷+…+1) = x⁸+1.
> "The simplest fold drops the fallen bit into the bottom slot. A rotation. Doubling is fine now. But three times two fifty-five comes out zero. The fold has to follow a rule that can't be factored."

This makes irreducibility *necessary* on screen, not just asserted.

**B6. Say once that pieces are chunks.**
In 02 or 07, add 5 s: each piece is a whole chunk, perhaps a thousand bytes, and one equation's coefficients apply to every byte position at once. This is why XOR runs 32–64 bytes per instruction and why byte multiplication is a table lookup. It also matches the owner's AVX2/NEON screenshot.

**B7. Cold open: the liquid demonstration first.**
> "I'm sending this picture to a million phones. Each loses a different tenth of what I send, and none can ask twice. Yet almost every phone that catches just two more packets than the picture has pieces rebuilds it. The standard demands failures no more often than one in a million. The secret is an arithmetic where three times seven is nine."

Then go straight to feedback implosion and the coupon collector (01_3–01_5 unchanged).

**B8. Small fix in 02_7.** The redundant fifth card should be a real combination of cards already held: B⊕C = 4, which is (A⊕C)⊕(A⊕B). The draft's "A⊕C⊕A⊕B…" is not a real card.

**Title:** *The Arithmetic Where 3 × 7 = 9* (alternative: *Liquid Data: How Bytes Became a Field*). 3·7 = 9 holds under every degree-8 representation (no reduction occurs), so the thumbnail is honest.
**Thumbnail:** chalky **3 × 7 = 9** with 011 × 111 = 1001 beneath, and the log-striped table glowing behind.

**Sound.** The glassy E-Lydian identity, with its motif taken from 0x1D's set bits, is exactly right. Changes:
- Play the 0x1D motif at *every* fold, so the viewer learns "that sound means reduction".
- Add a **rank chord**: each useful packet adds a note to a growing chord, and a wasted one is a muted thud. Near the end of the GF(2) run the thuds crowd in; over GF(256) they almost never come.
- Idea: a Shepard-style glide of 255 tiny steps (≈4.7 cents each) for the byte clock. It climbs for one octave and lands home: α²⁵⁵ = 1, heard.

### (b) Findings — Video B

| id | chapter | severity | issue | proposed change |
|---|---|---|---|---|
| B-1 | 04 | major | Chapter shows Z/256 is inconvenient, not that it is useless here | Exact theorem: mod-256 coefficients fail at the GF(2) rate; 2A = 6 → {3, 131} (B1) |
| B-2 | 03/09 | major | Ablation is announced, not understood; why 10–16 rows suffice is missing | Danger-at-the-end meter; predict 71% before revealing it (B2) |
| B-3 | 07/10 | major | "×256 per packet" and the closing line are asserted | Change-nobody-notices union bound, tight to 3 digits (B3) |
| B-4 | 06 | major | Torus is near-decoration; slide rule is linear but logs wrap | Circular slide rule; star → circle untangling; rhymes with 04's clock (B4) |
| B-5 | 05 | minor | Irreducibility asserted, not motivated | Rotation fold gives 3·255 = 0 (B5) |
| B-6 | 02/07 | minor | Pieces shown as single bytes only | One beat: chunks, bytewise coefficients (B6) |
| B-7 | 02_7 | minor | Redundant card not a real combination | B⊕C = 4 (B8) |
| B-8 | 04–06 | minor | ~4 min of algebra risks a sag | Every beat answers "do we have 1/256 blind spots yet?"; keep the two-clock rhyme |
| B-9 | 01 | idea | Hook previews payoff only at 01_6 | Liquid demo cold open (B7) |
| B-10 | §7.5 | idea | Byte chords teach XOR; nothing teaches rank | Rank-chord sonification in an authored gap |

---

## Video C — Taking a Number Without a Ticket Machine

### (a) Highest-leverage changes, ranked

**C1. Fix five slips before anything else.**
- **03_5:** the narration reads "three, zero, zero, four, two" with service order P5, P2, P4. The memo's slates are **0, 3, 0, 4, 2**. As written, the narration contradicts its own order.
- **02_4:** "Everyone who wants in eventually gets in" is Knuth's 1966 addition. Dijkstra 1965 was not starvation-free. Rewrite: "Edsger Dijkstra posed the problem in nineteen sixty-five, and Donald Knuth sharpened it a year later."
- **02_6:** "Leslie Lamport asked something harder" contradicts 08_6. He found the garbage-read property *after* writing the proof. Make it the video's question: "Here's a harder question… hold that thought." Then 08 reveals that Lamport noticed afterward that his proof already answered it.
- **10_1:** "it fails" should be "it can fail".
- **07_6:** "every schedule with up to four customers" needs "with small tickets", and the safe-register check covers only N ≤ 3.

**C2. Make the hero the space-time lanes, a score, and demote the grid.**
The lexicographic grid explains one fix (ties). The lanes carry ten chapters:
- 01 and 02: the read and write arrows;
- 05: the gap between reading and writing a ticket;
- 07: the proof zones;
- 08: reads that land inside a write;
- 09: doorway-ordered FCFS;
- 10: store buffers, which are simply writes whose arrows *land late*;
- the 1978 callback in C7.

Introduce the lanes in 01: "Draw each worker as a lane, time running left to right. A read is an arrow in, a write is an arrow out." Keep the grid for 04 and its callback.

The lanes are also literally a musical score, which unifies the §7.5 sonification: one staff per process, the critical section as the motif, mutual exclusion as "no two staves play their melody at once".

**C3. Replace the schedule trees with the progress square.**
A schedule of two processes is a staircase path, with process 1's progress across and process 2's up. For the lost update there are C(4,2) = 6 paths in a 2×2 grid. The four bad ones pass through the centre point (both have read, neither has written). That is exactly the plan's 4 of 6, now with the *reason* visible.
> "Here's every order at once. Worker one's progress runs across, worker two's runs up. Every schedule is a staircase from corner to corner. Six of them. The four that cut through the middle, where both have read and neither has written, lose a point. A lock's job is to wall off the middle."

- In 02 and 05 the red square is "both inside". With no flag: 28 paths, 4 enter it. With the flag: 322 paths, all detour around it.
- Blocked moves depend on history, so draw paths, not static walls. This is accurate.
- Optional 3D set piece (camera route, needs a spike): for N = 3 the square becomes a cube and "two inside" becomes three crossing square beams. Sampled staircases thread through as the counter reaches **2,126,816,238 schedules, 0 touching a beam**. This is a 3D orbit that carries an idea.
- Footnote for Ruth: these are Dijkstra-style progress graphs, the seed of directed algebraic topology *(verify attribution before narrating)*.

**C4. Draw chapter 08 on top of chapter 07's proof zones. This makes the astonishing fact inevitable.**
In the memo's Lemma 2, case (a) never uses the value i read. Case (b) reads a ticket that provably is not being written. Lemma 1's reads happen after the write ended. So *every read the proof trusts overlaps no write.*

Start with an honest torn read: rewrite "09" as "10" one digit at a time, and a reader mid-write sees 19 or 00. Then generalize to "anything at all, even 2496".
> "Go back to the proof picture, and let every slate flicker while it's written. When you saw my flag down, either I hadn't started choosing, or I'd finished. In the first case you might read my ticket mid-write and get garbage. The proof never used that value. I'm the one who waits, because I'm sure to see your ticket. In the second case my ticket was finished and nobody was touching it. Every read the proof trusts happens while nothing is being written. Every read that might be garbage is one it never trusts."

*Picture:* chapter 07's zones. Reads that cross a write turn magenta; every one of them falls in zone (a), shaded "value unused". Then the Holt anecdote and the quote card land as payoff, not as assertion.

**C5. Add a thesis line and a cold open.**
Thesis, at the end of 05 and again in the coda:
> "You can't close the gap between looking and writing. Lamport didn't try. He made the gap visible, with the flag, and made the outcome of every race harmless, with the tie-breaker."

Cold open, 20 s before 01:
> "A slate reads nine. Someone is rewriting it as ten. You glance mid-stroke. You might read nine, or ten, or, Lamport says, two thousand four hundred ninety-six. Could computers take turns safely with memory like that? In nineteen seventy-three, that was considered impossible. Lamport found an algorithm that does it. The idea came from a bakery."

This draws only on the Turing lecture's 2496 quote and its "considered impossible" line.

**C6. Compress 06 to about 50 s and move the JavaScript aside there.**
A 70-step walkthrough is the biggest energy sag in the corpus. Fast-forward on the lanes and narrate only three beats: the flag saves the day, the tie is broken by name, and late customer 1 waits behind (2,4). Move 04_6's comma-operator aside to 06's code panel (or the description), so it no longer interrupts the moment the grid is introduced.

**C7. In chapter 10, bring the chapter-04 grid back, relabelled.**
Relabel its axes "(clock value, process)". *Time, Clocks* (1978) orders events by counter and breaks ties by a fixed process order, which is the same lexicographic sweep.
> "Four years later, Lamport ordered events across whole networks with counters, breaking ties by name. Same grid."

If verified on his publications page, the bonus is that the 1978 paper is drawn with space-time diagrams and Lamport credits special relativity *(verify wording before narrating)*.

**C8. Rename chapter 01.** "Two plus one is one" garbles the arithmetic. The narration says two points scored, one counted, so use **"One plus one is one"**.

**Title:** *The Algorithm Lamport Is Proudest Of* (supported by the oral history), or *Taking Turns When Memory Lies*.
**Thumbnail:** a bakery ticket reading **2496** beside a slate frozen mid-stroke between 9 and 10.

**Sound.**
- B♭ with swing at 104 BPM suits Acts I–II. Drop the swing for 07–08 (rubato, a pedal under the lemmas), because the proof deserves stillness.
- Give the "two motifs at once = bug" demonstration a ≥3 s narration gap in 05 (trace a) and again in 10 (store buffers). Under ducking, a sonification nobody can hear teaches nothing.
- Spinning as a soft repeated note is excellent. Keep it audible in 09, where it *is* the cost.

### (b) Findings — Video C

| id | chapter | severity | issue | proposed change |
|---|---|---|---|---|
| C-1 | 03_5 | major | Slate values misread (3,0,0,4,2 vs 0,3,0,4,2) | Correct values; consider showing rather than reading all five aloud |
| C-2 | 02_4 | major | Knuth's eventual-entry rule attributed to Dijkstra 1965 | "Dijkstra posed…, Knuth sharpened it a year later" |
| C-3 | 02_6 | major | Frames Lamport as setting out to tolerate garbage; contradicts 08_6 | Pose as the video's question; reveal the after-the-fact discovery in 08 |
| C-4 | brief | major | Two hero pictures | Space-time lanes as hero; grid supporting (C2) |
| C-5 | 01/02/05 | major | Trees show counts, not reasons | Progress square; optional 3D cube for N = 3 (C3) |
| C-6 | 08 | major | Safe-register robustness asserted with separate traces | Overlay on 07's zones: trusted reads never overlap writes (C4) |
| C-7 | 06 | minor | 85 s walkthrough sags | ~50 s, three narrated beats (C6) |
| C-8 | 04_6 | minor | Code aside interrupts the grid reveal | Move to 06 code panel / description |
| C-9 | 07_6, 10_1 | minor | Bounded check reads as universal; "fails" is too strong | "with small tickets"; "can fail" |
| C-10 | 01 title | minor | "Two plus one is one" garbles the arithmetic | "One plus one is one" |
| C-11 | 10 | idea | Lamport-clock link is verbal only | Grid relabelled (clock, id); 1978 space-time diagrams (verify) |
| C-12 | §7.5 | idea | Swing under the proof | Rubato + pedal in 07–08; authored gaps for the clash demo |

---

## Cross-video: series identity and recurring devices

1. **One theme, three dark regions.** Each video is about certainty from what you cannot see:
   - A: the cell no net touched;
   - B: the change no packet notices;
   - C: the red square no schedule may enter.

   Give each the same visual treatment (a region drawn as "unknown", then quantified) and end each video by showing it measured. That is the series' version of §5.1 rule 15. A possible series line, if the owner wants one: *"How do you know what you can't see?"*
2. **A grammar for the opening 30 s.** A ≤20 s cold open with a paradox that previews the payoff (A1, B7, C5), then the title, then the honest failure. Amend §5.1 rule 2 to allow this, and add a rule 19: *"Make the surprise inevitable in hindsight."* Every major change above does exactly that.
3. **Recurring devices, identical in every video:**
   - the "precisely:" lower-third (Ruth);
   - a prediction card with a fixed two-note sting (Maya);
   - the "Dev aside" code panel sliding in from the right with a soft typewriter tick;
   - a one-sentence recap card per act;
   - a closing **receipts** card ("every number in this video was computed by code: link"), which is the public face of the plan's verification discipline.
4. **Listening windows.** The §7.2 mix rules are written to keep music out of the way of speech. But every *signature* sonification (A's proportion rhythm, B's rank chord, C's motif clash) is ducked to inaudibility unless the storyboard authors a narration gap of 3 s or more for it. Add a `listen` column to the storyboard tables, separate from `hold`.
5. **Cross-links worth one line each:**
   - A's code-review Venn and C's "single-writer" discipline are both about reviewers and writers;
   - B's XOR is the same parity Dev meets in RAID, and Linux RAID-6 uses B's exact field (already in the plan).

   Don't force more than that.

## New claims in this memo, and how I checked them

I checked these with a script I wrote in the session scratchpad, not in the repo. Port them to `docs/research/*/scripts/`, plus the new pins and lower-thirds into `facts.rs` and the oracles, before any of them is narrated.

- **A:**
  - Mn − kN = (M−k)(n−k) − k·u: algebra, and exact at N = 9,999 / 10,000 / 10,001 (+100, 0, −100) and the toy (49/3 → 33).
  - ΣN L(N) = Mn/(k(k−1)): proved by a beta integral; numerically 16.667, 10.000 and 5.25 for three parameter sets.
  - CV formula and SD(k) = 9.000; the 2,000/500 split gives a CV of 8.7%.
- **B:**
  - Z/256 invertibility equals the GF(2) product (determinant-parity argument; MC 0.288 against 0.290 at K = 8).
  - 2·131 ≡ 6 (mod 256).
  - Union bound 256^−h/255 against the exact values: 0.0039216 vs 0.0039215; 1.5319×10⁻⁵ vs 1.5319×10⁻⁵; 5.984×10⁻⁸ vs 5.984×10⁻⁸.
  - GF(2) risk before the last 10 slots ≤ 2⁻¹⁰.
  - Failure for K = 1–100 nearly flat in K.
  - 3·7 = 9, 3·5 = 15, 2·3 = 6.
  - 3·255 = 0 mod x⁸+1, while doubling there is a bijection.
- **C:**
  - Lost-update paths through (1,1) number 4 of 6, by enumeration.
  - Lemma-2 case analysis taken from memo §4.
  - "09 → 10 read mid-write as 19 or 00" is digit-wise arithmetic, not a Lamport quote; use the 2496 quote verbatim for "anything at all".
- **Still to verify** (flagged inline above):
  - progress graphs credited to Dijkstra;
  - the wording of Lamport's special-relativity remark and the use of space-time diagrams in *Time, Clocks*;
  - the exact quotations behind the 2496 line and the "impossible" line, re-checked against the Turing lecture.
