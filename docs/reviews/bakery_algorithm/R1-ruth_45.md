<!-- R1 storyboard review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; storyboards at 2ad2579) -->

# R1 review: Video C, "The Algorithm Lamport Is Proudest Of", as Ruth Adler (ruth_45)

Most of the counts check out. The proof chapters do not. Chapter 08 states a false lemma: "the magenta glances all land in the first case". The chapter's own next example, e1, is a counterexample. Chapter 07's case split also has a gap that a student would catch. Six other lines make claims that are false or missing their conditions: 01_8, 02_4, 02_5, 09_2, 09_4 and 10_1.

## Comprehension check

| ch | what I would tell a friend |
|---|---|
| 01 | `x = x + 1` is a read and then a write. Of the C(4,2) = 6 interleavings of two such increments, exactly the 4 that pass through "both read, neither wrote" lose a point. At billions of steps a second, rare bad interleavings happen all the time. |
| 02 | We want mutual exclusion plus some progress rules. A check-then-flip sign fails (8 of 14 orders put two inside). Indivisible reads and writes are enough to fix it. The open question is memory whose reads can catch a write halfway through. |
| 03 | Each customer writes only its own slate and takes 1 + max of the other slates, then waits for every smaller nonzero ticket. No shared ticket machine is needed. |
| 04 | Tickets can tie, so the order is on (ticket, name) pairs, compared like a dictionary. That is a strict total order. It removes both the double entry and the "after you" deadlock. |
| 05 | Without a flag, a customer can be judged while its ticket is still unwritten: 4 of 28 orders put two inside. The choosing flag forbids judging anyone mid-doorway, and then 0 of 322 orders do. |
| 06 | A four-customer run shows the flag blocking an early check, a tie settled by name, and service in (ticket, name) order. |
| 07 | Whoever is in the room holds the smallest pair among ticket holders. At Blue's *last* check of Green's flag, Green had either not started choosing (so Green's ticket will be bigger) or had finished (so Blue saw Green's real ticket). If both were inside, each pair would be smaller than the other. |
| 08 | The only reads whose values the proof uses are reads of finished slates that nobody is touching, so a read that overlaps a write may return anything. **But the chapter as drafted tells me something false:** that the bad glances all fall in the first case (R1). My sentence comes from the memo, not from the chapter. Blocker. |
| 09 | It is first-come-first-served, counted from the end of the doorway, so nobody starves under the right conditions. Tickets are unbounded: below 2³⁵ after a year at one per ms, *if reads never overshoot*. It costs 3(N−1) reads, and the max must read each slate once. |
| 10 | Real CPUs let a later read overtake an earlier buffered write, so both customers can enter while memory still shows all zeros. Fences (two per entry on x86) or sequentially consistent (SeqCst) atomics restore what the proof assumed. |
| 11 | It survives in Arm firmware where the exclusive-access instructions can't be used. Its (counter, id) trick reappears in Lamport clocks and Paxos ballots. Lamport says he discovered it, and that is why he is proudest of it. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 4 | 3 | 5 | 4 | 4 |
| 02 | 4 | 3 | 4 | 4 | 3 |
| 03 | 5 | 4 | 5 | 4 | 4 |
| 04 | 4 | 4 | 4 | 4 | 4 |
| 05 | 4 | 5 | 5 | 3 | 5 |
| 06 | 4 | 5 | 3 | 3 | 3 |
| 07 | 3 | 3 | 4 | 3 | 4 |
| 08 | 3 | 1 | 5 | 3 | 5 |
| 09 | 4 | 3 | 4 | 3 | 3 |
| 10 | 4 | 3 | 5 | 4 | 4 |
| 11 | 4 | 4 | 4 | 4 | 5 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth_45 | 08_3–08_4 | blocker | "Where do the magenta glances land? All in the first case" is false, and 08_5's own e1 is the counterexample. In e1 (`scenarios.py`), P2's garbage read of 7 happens at step 7, inside P2's doorway (steps 5–14). P1's passing check of P2's flag is at step 15. With Blue = P1 in the room and Green = P2 holding ticket 8, that is the **last** case ("finished choosing"). In the last case, Green's max-reads of Blue's slate (and Blue's of Green's) can also overlap writes. The memo (§4) states the true result: the reads *whose values the proof uses* never overlap a write. 08_4's third sentence says exactly that; its first sentence contradicts it. | 08_3: "…let every slate and every flag flicker while it's written. Every glance that crosses a write turns magenta. ⟨predict⟩ Does the proof ever rely on a magenta glance?" 08_4: "Never. It relies on just two glances: in the first case, Green reading Blue's finished ticket; in the last case, Blue reading Green's finished ticket. Nobody was writing either slate. Magenta lands in every zone, always on reads the proof never used." Picture: the two trusted reads in gold, and magenta in both zones, with e1's glance inside Green's doorway in the last zone. This adds about 4 s; to stay ≤ 85 s, trim 08_5 (see R3). |
| R2 | ruth_45 | 08_3, 08_6 | major | Flags are memory too, and 08 only lets slates flicker. A glance at a flag while it is being raised can read "down". The proof survives this: a glance during the raise falls in "not yet choosing", and one during the lowering falls in "finished". But the chapter never says so, and this is exactly where Lamport's 1974 proof slipped. A footnote there treated one-bit reads as atomic, which he later called a fallacy (memo §2.2). As drafted, the takeaway "survives memory that returns garbage" is argued only for tickets. | Flicker the amber flags in 08_3 as well. Lower-third: "A glance at a flag mid-raise counts as 'not yet choosing'; mid-lowering, as 'finished'." Description footnote: Lamport's 1974 single-bit footnote, his later retraction, and the safe-flag model check (N = 3, 269,550 states). |
| R3 | ruth_45 | 08_5 | major | "The writer is taking a brand-new ticket, so she started choosing after the reader finished" does not follow. A new ticket does not imply a later start. The real reason is that the reader had already seen the writer's flag down, and nobody writes a ticket with their flag down. The "her/she" pronouns also make it unclear who is who. | "A glance that catches Green's old zero mid-write lets Blue straight in. Dangerous? No. Blue had already seen Green's flag down, and nobody writes a ticket with the flag down. So Green started choosing after Blue had finished, and by fact one her ticket will be bigger. Green waits." Cut "and wait longer" in the e1 sentence to pay for the words. |
| R4 | ruth_45 | 07_4–07_5 | major | "When Blue checked Green's flag": Blue may check many times while spinning. "In the middle of choosing… Blue would have waited" only rules out the middle case for the check that *let Blue through*. As worded, a student will say: "Blue did see the flag up, waited, then checked again. So what?" | 07_4: "⟨predict⟩ Take the last time Blue checked Green's flag, the check that let Blue through. Where could Green have been, in taking the ticket she holds now? Three places." Picture: label the read "Blue's last flag check". |
| R5 | ruth_45 | 07 "Precisely" (description text) | major | The note says Lemmas 1 and 2 are "exactly as in the memo §4", but §4 has an edge gap. Case (b) is "Dₖ ended before r₂ ended", and the closing bullet says that a check overlapping the write that closes Dₖ falls in (b). If that write ends after the check ends, it is in neither case. In the checker's point-read model this happens every time. Also: Bᵢ runs "to the end of the exit write" while "number[i] is constant", which is false during the exit write for safe registers. And "in the bakery = holding a nonzero ticket" is not the memo's definition (end of doorway). It still works, but say which one you mean. | In the description's exact version: case (b) = "Dₖ's closing write began before r₂ ended". Bᵢ ends where the exit write *begins*. State one definition of "in the bakery" and use it in both lemmas. Fix memo §4 the same way. |
| R6 | ruth_45 | 09_2 | major | "As long as… nobody dies in the doorway" is the wrong condition in both models. In Lamport 1974 a failed customer's slates return to 0, so dying in the doorway is harmless; only failing and restarting forever deadlocks (memo §2.1). In real code with no reset, dying while *waiting* with a ticket also blocks everyone with a bigger pair, forever. The stated condition is not sufficient. | "…as long as everyone in the room eventually leaves, and nobody stalls for good between raising the flag and leaving." Lower-third: "Lamport 1974: a crashed customer's slates fall back to zero; then crashes are harmless unless one keeps crashing and restarting." |
| R7 | ruth_45 | 09_4 | major | We were just told (08) that a glance can return anything. Then "That fits easily" drops Lamport's own condition: "assuming a read of number[i] can never obtain a value larger than one which has been written there." With safe slates, a single garbage read near the top of the range overflows the next ticket at once. | Voice: "…That fits easily, as long as a glance never returns more than was ever written." Lower-third: Lamport's 1974 sentence verbatim. |
| R8 | ruth_45 | 10_1 | major | The promise shown breaking, "a write is seen in order", is the wrong one. On x86 the store buffer is first-in first-out, so writes *are* seen in order. What breaks is "my write lands before my next read". 10_4 says this correctly, so the screen contradicts the voice. | Promise card: "a write lands before my next read" flickers and breaks. Optionally keep "writes are seen in order" with a check mark to show what x86 does keep. |
| R9 | ruth_45 | 02_4 | major | Attribution. As narrated, Dijkstra's 1965 rules allow everyone to be stuck forever, and Knuth "added one more". From my memory of CACM 8(9):569, Dijkstra's condition (d) already forbade an endless "after you" stand-off (somebody gets in). Knuth strengthened that to *everybody* gets in. I did not verify this against the PDF, and the fact sheet has no row for it. | Check the 1965 text, then: "…Fast or slow, it still works. And no endless 'after you': somebody always gets in. A year later Donald Knuth asked for more: everyone who wants in gets in eventually." Add a C-F14 row citing Dijkstra's conditions verbatim. |
| R10 | ruth_45 | 02_5 | major | "Dijkstra showed an ordinary sign is enough" says one read/write sign suffices. It doesn't, and 09_5's card itself cites Burns–Lynch (N processes need at least N registers). Dijkstra's solution uses several shared variables. | "Dijkstra showed ordinary signs are enough, several of them, if nobody can ever catch one halfway through a flip." |
| R11 | ruth_45 | 01_7–01_8 | major | We count 4 bad orders out of 6, then hear "the bad orders are rare". To a careful viewer that is a contradiction, because the orders are not equally likely. The Ruth note ("a count of orders, not a probability") isn't in the voice or on screen. | 01_8: "Counting orders isn't measuring chances: a program almost always runs both steps back to back, so in a real machine the bad orders are rare. But at billions…" Lower-third: "6 orders, 4 bad: a count, not a probability." |
| R12 | ruth_45 | 07_3–07_6 | minor | 07_4–07_5 is the proof of fact two, but nothing says so, and the cases never conclude "fact two". Fact one is stated with "you/I" and then used with Blue and Green, so the viewer has to do the substitution live. | Open 07_4 with "Why is fact two true?" End 07_5 with "Either way, Blue's pair is smaller. That's fact two." In case 1, show "you = Blue, I = Green" on the fact-one card. |
| R13 | ruth_45 | 04_5 | minor | "Sweep column by column. That's the serving order" is too general. My BFS on `bakery_mc` (N = 3, tickets ≤ 3, indivisible reads and writes) finds a 24-step run that serves (1,2), then (1,1), with the slates never all zero in between. P1 read P2's slate after P2 had left. This is harmless, but the served pairs are not increasing. 06_4's claim is about one run and is correct. | "That's the order the room takes everyone holding a ticket at the same time." Description: the counterexample in one line. |
| R14 | ruth_45 | 07_7 | minor | "Every order" and "3,027,633 states" are different things. The search also only covers runs whose tickets stay ≤ 4, and N = 4 was checked with indivisible reads and writes only (safe slates only up to N = 3). | Card: "N = 4, tickets ≤ 4, indivisible reads and writes: 3,027,633 reachable states, no violation." Voice: "…over every order, up to four customers, as long as tickets stay small, agrees." |
| R15 | ruth_45 | 10_3 | minor | "Two fences per customer are enough" is stated without bounds; it is a TSO-model result for N ≤ 3. "Per customer" should be "per entry". "Arm chips… need more" has no row or source in the fact sheet. | "On an Intel or AMD chip, two fences each time you enter are enough." Lower-third: "x86-TSO model, checked to 3 customers; matches Abdulla et al., TACAS 2012." For Arm, add a source row, or say "reorder more, so the fences go in more places". |
| R16 | ruth_45 | 09_2, 09_6 | minor | "Each other customer can get ahead of you at most once" is only true once you have finished choosing; before that, you can be overtaken any number of times. "Read one twice… it breaks" should be "can break", since the model shows a failing run, not that every run fails. | "Once you've taken your ticket, each other customer can get ahead of you at most once." 09_6: "…it can break." |
| R17 | ruth_45 | 09_5 card | minor | "3(N−1) slate reads": a third of those reads are of flags, and our vocabulary says flags are not slates (see also fact sheet F5). | "3(N−1) reads of other customers' flags and slates, even alone." |
| R18 | ruth_45 | 05_7 | minor | "Made every race harmless with the tie-breaker" credits the tie-breaker with too much. What makes the remaining races harmless is the flag plus fact one; the tie-breaker settles equal tickets only. | "…made it visible with the flag, and settled the one race left, equal tickets, with the tie-breaker." |
| R19 | ruth_45 | 08_1–08_2 | minor | "Anything at all" is stronger than Lamport's own condition (7456 "assuming that 7456 is a value that could be in the memory location"; a safe register returns "one of the possible values of the register"). That condition matters for R7. "He called memory like that safe" suggests 1974, but the name is from 1986. | "…may return any number the slate could hold." "Twelve years later he named memory like that safe." |
| R20 | ruth_45 | 08_6 | minor | The quote card dated "(1974)" mixes sources. "Any arbitrary value" is from the 1974 paper. "Noticed after writing the proof" and Holt are from his later publications-page note. | One source line per quote: CACM 17(8) 1974 for the first, and "Lamport, publications page" for the other two. |
| R21 | ruth_45 | 03_1 | minor | "Lamport's idea came from a bakery" is a historical claim with no fact-sheet row. The origin story the memo does record (CHM pp. 7–8) is different: irritation at a buggy 1972 algorithm. | "Picture a bakery: take a number…", or add a sourced C-X row before keeping "came from". |
| R22 | ruth_45 | 11_1 | minor | "On chips without hardware cache coherence" overstates the TF-A source, which says "not all PSCI participants are cache coherent" (`HW_ASSISTED_COHERENCY == 0`) and refers to a waking core with its cache off. | "…on chips where a core waking up isn't yet coherent with the others, with its data cache still off, the usual spinlocks can't be used." |
| R23 | ruth_45 | 10_1 / 07_1 | minor | 10_1 calls back to "the proof's promises from 07", but 07 never lists any. | In 07_1, add a lower-third: "The proof assumes: each customer's steps happen in order; a finished write is seen by every later read; the max reads each slate once." |
| D1 | ruth_45 | 07_7 | delight | "A search only covers small cases. The proof covers all of them," with the card changing to "every N". This is the right distinction, made at the right moment. I would use it in class. | Keep. Apply R14's wording to the card. |
| D2 | ruth_45 | 05_6 | delight | The 28 → 322 path-set change with the red square emptying. Both counts recomputed (see below). | Keep. Put "one visit each; a wait counts once, when it succeeds" in the lower-third, since both counts depend on it. |
| D3 | ruth_45 | 08_1 | delight | "Oh-nine to ten, a digit at a time, reads nineteen or zero zero" is a perfect elementary torn read (both write orders checked by hand). | Keep. |
| D4 | ruth_45 | 03_2 | delight | "A ticket machine is shared. It needs a room of its own." The infinite regress in one line, and it is Lamport's own Turing-lecture argument. | Keep. Put the verbatim line ("would require a ticket-machine process that never halts") in the description. |
| D5 | ruth_45 | 08 Ruth note | delight | The fact that Lamport's own 1974 proof treated the one-bit flags as atomic, and he later called that a fallacy, is the historical footnote I didn't know. It is buried in a note. | Move it to the description and to R2's lower-third. |

## What I ran (all scratch files are in `/tmp/r1_ruth_c/`; nothing in the repo was changed)

- **My own interleaving counter** (`enum2.py`, `enumN.py`, `sign.py`). Convention: one step per read, write and successful wait; one visit each; the max reads the other customers' slates. I read `tree_count.py`'s docstring for these conventions, but the code is mine. Every count matches the fact sheet:
  - lost update: 6 orders, 4 lose a point (C(4,2) = 6)
  - sign: 14 orders, 8 with both inside
  - two customers, no flag: 28 orders, 4 bad, 0 deadlocked
  - two customers, with flag: 322 orders, 0 bad
  - comparing numbers with `<`: 898 orders, 432 bad
  - comparing numbers with `≤`: 250 orders, 0 bad, 72 deadlocked
  - three customers, no flag: 792,987 orders, 312,492 with two or more inside
  - three customers, with flag: 2,126,816,238 orders, 0 bad
- **Reran the research checker `bakery_mc.explore`.** This uses their code, so it is not an independent check.
  - N = 3, tickets ≤ 6, safe slates: 269,550 states (12,306 pruned), no violation, 7.5 s.
  - N = 4, tickets ≤ 4, indivisible reads and writes: 3,027,633 states (204,059 pruned), no violation, 87.5 s on this machine (the fact sheet says about 30 s).
- **Arithmetic with exact fractions:**
  - 31,557,600,000 ms in a 365.25-day year < 2³⁵ = 34,359,738,368, and 2³⁴ is too small.
  - 2⁶⁴ tickets at 10⁹/s last 584.542 years.
- **e1 zone placement:** read the schedule in `scenarios.py` (this is R1's evidence).
- **New BFS searches with an added tracking variable on `bakery_mc`:**
  - Serving order (R13): found the 24-step counterexample.
  - 09_3 ("tickets keep growing while somebody is in line"): searched for a cycle where the bakery never empties, the room keeps serving, and tickets stay bounded. None exists for N = 2 with tickets ≤ 2, or N = 3 with tickets ≤ 4. So 09_3 holds within those bounds; nothing to report.
- **Checked by hand:** the 01_7 set equality, C-F7 (0, 3, 0, 2 → 4; served 5, 2, 4), 08_1's 19 and 00, the Rust panel's lexicographic `break` condition and its single read per slate, and the 12-step TSO count (6 steps per customer).

## Unchecked

- The step counts of traces a′, b1/b2, d, the double-fetch trace and leapfrog (15, 12/14, 70, 33, 46); I did not replay them.
- The TSO state counts.
- The tsc and node outputs.
- Every quote and citation. I fetched no sources.
- Dijkstra's 1965 condition (d), which I am going on from memory (R9).
- Burns–Lynch, Abdulla et al., and the Arm "needs more" claim.
