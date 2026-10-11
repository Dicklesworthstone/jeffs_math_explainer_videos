<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# R2 review: Video C, "The Algorithm Lamport Is Proudest Of" (script). Reviewer: Dev, 28

## Comprehension check

| ch | the one sentence I'd tell a colleague |
|---|---|
| 01 | `x = x + 1` is a read followed by a write. Of the 6 ways two increments can interleave, exactly the 4 that pass through "both have read, neither has written" lose a point, and at billions of steps a second that happens all the time. |
| 02 | A check-then-flip "occupied" sign fails the same way (8 of 14 orders let both in). Dijkstra's 1965 rules plus Knuth's 1966 "everyone eventually gets in" define the problem, and Dijkstra solved it with several plain signs, provided nobody can catch one mid-flip. The cliffhanger: what if a read can catch a write halfway? |
| 03 | A shared ticket machine would itself need a lock. Instead each customer writes only her own slate: she reads the others, writes 1 + max, and waits for every smaller nonzero ticket (Blue reads 3, 0, 0, 2, writes 4, and is served after Pink and Green). |
| 04 | Customers who read at the same moment can take equal tickets. Comparing tickets alone either lets both in (`<`) or deadlocks them (`≤`), so you compare (ticket, customer number) like words in a dictionary. |
| 05 | A ticket that has been chosen but not yet written still reads 0, so a rival walks in and then she walks in too. A choosing flag, raised before reading and lowered after writing and checked before comparing, takes it from 4 bad orders in 28 to 0 in 322. The flag isn't a lock. |
| 06 | Three customers have 2,126,816,238 orders, and none of them puts two in the room. |
| 07 | In a four-customer run the flag stops Purple reading Green's unwritten ticket, ties go to the lower customer number, and the serving order is a sweep of the (ticket, number) grid. |
| 08 | Fact one says whoever finished choosing before you started has a smaller ticket. Fact two says whoever is in the room has the smallest pair of everyone holding a ticket, proved by cases on where the other customer was at the last flag check. Together they make two in the room a contradiction. |
| 09 | The proof trusts only two ticket reads, both of slates nobody is writing, so a read that overlaps a write may return anything ("safe" memory). Garbage just means a bigger ticket and a longer wait. |
| 10 | A garbage zero that lets Green in early is harmless: Green had already seen Blue's flag down, so Blue started choosing later and will take a bigger ticket. Lamport noticed this only after writing the proof. The max must read each slate exactly once. |
| 11 | It is first-come-first-served from the end of choosing. So nobody waits forever unless someone dies holding a flag or a ticket, and each rival can overtake you at most once. Tickets grow while the line never empties (32 bits wrap in under two months at one per ms; 64 bits last centuries), each entry costs 3 reads per other customer even alone, and a frozen chooser stalls everyone. |
| 12 | CPUs park writes in store buffers, so both customers read stale zeros and both walk in. On x86, two fences fix it (after raising the flag and after lowering it). Arm "needs more care", which I can't turn into code (dev-02). SeqCst atomics are the easy fix everywhere, and acquire/release isn't enough because it doesn't order a store before a later load. |
| 13 | Real locks use atomic instructions, but Arm's reference firmware uses a bakery lock while a waking core has its cache off and can't use exclusive-based spinlocks. The idea of a bigger ticket with ties broken by number reappears in Lamport clocks and in Paxos ballots. |

No chapter has a wrong or empty sentence, so there are no blockers.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 5 | 4 | 5 | 4 | 4 |
| 02 | 5 | 4 | 4 | 4 | 4 |
| 03 | 5 | 5 | 5 | 4 | 4 |
| 04 | 5 | 5 | 5 | 4 | 4 |
| 05 | 5 | 5 | 5 | 4 | 4 |
| 06 | 4 | 5 | 3 | 5 | 4 |
| 07 | 4 | 5 | 4 | 3 | 4 |
| 08 | 4 | 5 | 5 | 4 | 5 |
| 09 | 4 | 5 | 4 | 4 | 5 |
| 10 | 5 | 4 | 5 | 4 | 5 |
| 11 | 4 | 5 | 4 | 3 | 4 |
| 12 | 3 | 4 | 5 | 4 | 5 |
| 13 | 5 | 5 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| dev-01 | Dev | 12_3–12_4 | major | The prediction asks what each customer sees when she "reads the other's slate". The answer is "No flag." Everywhere else a slate holds a ticket and the flag is a separate thing (05_4, 09_4). The mechanism is that both reads come back stale: the flag check sees 0 and the ticket check sees 0, because both writes are still in the outboxes. If I only hear "no flag", I think "fine, the ticket check will catch it", and nothing tells me it doesn't. The claim walk flagged this wording (claims.md, 12_3–12_4 note), but the fix never reached the script. My own store-buffer model agrees: the unfenced double entry takes 12 program steps with 0 drains, and memory is (0,0,0,0). | 12_3: "Then each checks the other's flag and slate. What does she see?" 12_4: "Zeros. No flag and no ticket. Both walk in, while the real slates still say nobody is even choosing." Picture: both read arrows land on zeros. |
| dev-02 | Dev | 12_6 (and R1.md's dev-10 line) | major | "Arm chips … reorder even more, so they need more care." More care how? This is the kind of hand-wave I'm allergic to, and it lands exactly where it matters. My laptop, the one 12_1 talks about, is an M-series Mac, and 12_5 has just given me a two-fence recipe I would copy. The claim is checkable. I extended my x86 model so stores to different addresses can leave the outbox out of order, which the Arm architecture permits (C-S11's message-passing quote shows the same effect). With x86's two fences, both customers walk in after 22 moves. Blue's lowered flag leaves her outbox before her ticket does. Green sees no flag and no ticket and walks in. Then Blue's ticket lands, she wins the tie on her lower number, and walks in too. That is chapter 05's unwritten-ticket bug brought back by the hardware, which is a far better callback than "more care". My model is not an Arm model (Arm also reorders loads), so it shows that two fences fail, not that any set of fences is enough. Also, R1.md records dev-10 as fixed with "there the fences go in more places", which the script doesn't say. | 12_6: "Arm chips, like the ones in recent Macs, reorder even more. Their outboxes can deliver out of order, so your flag can come down before your ticket lands. That's chapter five's bug again, and two fences don't stop it." Picture: the flag letter leaves the outbox before the ticket letter, and two tunes clash (05_3's listen window). Back it with a research-script row (TSO plus out-of-order draining, N = 2: F1+F2 fails) and with the writer-side message-passing test from Maranget et al. 2012 added to C-S11. Correct R1.md's dev-10 line. |
| dev-03 | Dev | 11_7 and the 11 costs card | major | Honest costs are why I'm watching, and the final script still says the card is "held at least 10 s, or in the description": the choice isn't made. As specified, the card is about 80 words over a three-beat picture, which nobody can read in 10 s. Two costs that bite in practice aren't stated plainly. First, N is fixed when you allocate the arrays. Second, a first-come-first-served lock hates preemption: if the customer holding the smallest ticket is descheduled in her wait loop, everyone behind her waits until she runs again. That follows from fact two (I reasoned it, didn't check it); the claim walk's G5 check covered only the frozen-while-choosing case. With more threads than cores, this is the common case, not an edge case. | Decide now. On screen, about 30 words for 10 s: "Customers fixed in advance · Next in line paused → everyone waits · A dead customer's slate stays set: resetting it is your job · Pad each slate to its own cache line · 64-bit tickets (Java: AtomicLongArray)". Move the rest to the description with sources: the 3(N−1) breakdown, 2N variables against Burns–Lynch's lower bound of N, and Lamport's reset assumption. Add a G5-style edge check for a customer frozen in her wait loop while holding the smallest pair. |
| dev-04 | Dev | 12_7 panel highlights | minor | 12_5 tells me the two fences go after raising the flag and after lowering it. The panel then highlights a different pair: "the ticket store and the next `load(SeqCst)` of another slate". Read literally, the next load after the ticket store is `choosing[j]`, a flag, not a slate. I'm also never shown where F1 sits in the code. My model says the pairs that matter are the flag-up store before the max's loads (F1 alone fails in 18 moves) and the ticket store before the wait-loop loads (F2 alone fails in 20). F1 plus a fence right after the ticket store holds, both on TSO and with out-of-order draining. | Highlight two pairs, labelled F1 and F2 with the same marks used on the 12_5 lanes. F1 is panel line 3 → line 5 (`choosing[i].store(true)` → the max's loads). F2 is line 6 → line 9 (`number[i].store(t)` → `choosing[j].load`). Put the panel verbatim in the description: 6 s is not enough to read 19 lines, and I'll want to copy it. |
| dev-05 | Dev | 10_2 | minor | "Nobody writes a ticket with the flag down." The panel's `unlock` does exactly that: it stores 0 to the slate with the flag down. The claim walk says it holds because 0 isn't a ticket, but two minutes later I'll have the code panel in front of me and I'll object. | "…and nobody takes a ticket with the flag down." |
| dev-06 | Dev | 01_9 | minor | "…so in a real machine the bad orders are rare." That's true for a scoreboard updated now and then. Nothing in the repo measures it for a hot counter (the claim walk marks it UNSOURCED), and after this chapter the first thing I'd do is run two threads doing `count++`. The line doesn't need a frequency claim to land "constantly". | "Each program usually runs both steps back to back, so a bad order needs unlucky timing. But at billions of steps a second, unlucky timing comes constantly." |
| dev-07 | Dev | 02_5 | minor | "Someone who wanders off outside can't block everyone." Dijkstra's rule (c) (C-S1) forbids blocking "the others", meaning any of them. If I implement against the voiced rule, a lock where an idle thread blocks some others would pass. The bakery meets the stronger rule anyway, because an idle customer has a 0 slate and a lowered flag. | "Someone who wanders off outside can't block anyone else." |
| dev-08 | Dev | after 07_5, the comma-operator "for programmers" item | minor | My favourite programmer moment from R1 is still listed as "a code panel after 07_5, or the description" in the final script. Its text isn't written, so no review round will ever read it. ESLint has still not been run: bead mx-e-c-7v0.5 is open and nothing in the repo records a result. Meanwhile plan §11 (line 2083) still says "ESLint's `no-sequences` catches it", which could leak into the description. | Pick the description and write the text into the storyboard now: the snippet, "evaluates `j < i`", "`tsc --strict` is silent (5.9.3)", and "Python and Rust tuples compare correctly". Say nothing about ESLint until it has been run and recorded. Correct the plan line. |
| dev-09 | Dev | 10_4; 11 costs card; 12 lower-third; 12_7 card | minor | Five on-screen or voiced attributions have no C-S row. They rest only on the memo's research sub-agent checks (marked †) or on a paraphrase. (a) Burns & Lynch 1993, "any read/write lock needs at least N". (b) Abdulla et al., TACAS 2012, "matches". (c) 10_4's "from Lamport himself": claims.md has the POPL 1983 quote but the fact sheet doesn't. (d) "Lamport assumes a dead customer's slates reset to zero", which is a paraphrase at memo l.112. (e) "C … plain shared variables are undefined behaviour", which cites only "see memo §8". Separately, C-S13 and C-S14 say "Not checked: rustc's actual output", but my R1 assembly check did check it (rustc 1.100.0-nightly). On x86-64, lock plus unlock is 4 `xchg` and no `mfence`. On aarch64 it is `stlr`/`ldar` with no `dmb`, and Acquire loads compile to `ldapr` on this Mac. | Add rows C-S18 to C-S22 with verbatim quotes and page numbers, re-read rather than marked †: Burns–Lynch's theorem, Abdulla's bakery fence result, Lamport 1983's "separate fetches" sentence, Lamport 1974's failure paragraph, and C11 §5.1.2.4's data-race sentence. Take off the screen anything that can't be quoted. Record the R1 assembly in C-S14 with its toolchain, or re-run it under c7v0.17. |
| dev-10 | Dev | 05_3, 05_5, 07 (step counts on screen) | minor | The 02_4 lower-third defines a step as a read, a write or a successful wait, and the 28/322 counts use that rule (I reproduced them with it). The traces the pictures replay use a different rule. Trace a's 8 steps include two "start" moves, and trace a′'s 15 include two blocked waits ("L2: choosing[1]=1, wait" and "L3: number[1]=1, wait", in `scenarios_out.txt`). Trace d's 70 come from the same model. Under the lower-third's rule, a is 6 steps and a′ is 13. If a step clock or "15 steps" appears on screen, I'll count and get a different number. Chapter 07 also shows five lanes for a four-customer trace. | Renumber every on-screen step count in the lower-third's convention (a = 6, a′ = 13, recount d), or show no counts at all. Dim Pink's lane in 07 ("Pink stays home") so the reads on screen match trace d. The 12-step x86 trace is already consistent: 12 program steps, 0 drains. |
| dev-11 | Dev | 12_2, 12_5 | minor | "Outbox" and "fence" are good spoken words, but I need the real names once so I can search for them and read the assembly. | Lower-third at 12_2: "programmers say store buffer". At 12_5: "x86: MFENCE or a locked instruction; Arm: DMB". If space is tight, fold them into the existing chapter-12 lower-thirds. |
| dev-12 | Dev | 13_2 card | minor | The card says "Arm Trusted Firmware", the lower-third says "TF-A", and the project calls itself Trusted Firmware-A (C-S17). The storyboard also puts the card's paraphrase in quotation marks, though it isn't a quote. | Card: "Trusted Firmware-A (TF-A): a bakery lock for a core waking with its cache off", with no quotation marks. |
| dev-13 | Dev | chapter-10 storyboard Dev note; brief's notation row | minor | Two leftovers. First, the chapter-10 Dev note (a 64-bit ticket written in two halves on a 32-bit chip is why 11_6 needs its caveat) has no destination: it is neither a lower-third nor in the description. Second, the brief's notation row still lists `volatile int[]` and `AtomicIntegerArray` for the glyph probe, while the screen now sets `volatile long[]` and `AtomicLongArray`. | Put the Dev note in the description under chapter 11, or drop it with a reason. If it goes in, its Java form needs a source row too: a plain `long` may be written as two halves (JLS §17.7). Update the brief's strings so the glyph probe covers what is actually typeset. |
| dev-14 | Dev | 12_3–12_5 | delight | The outbox x-ray is the store-buffering litmus test, and the fence rule is now precise enough to type in. My own model, written from scratch, gives exactly the fact sheet's numbers: no fence fails in 12 moves, F1 only in 18, F2 only in 20, and F1+F2 holds (N = 2, repeated visits; tickets/buffer bounds 4/4, 6/4 and 5/6). | Keep, with dev-01's wording. |
| dev-15 | Dev | 05_7, 06_2 | delight | "Three hundred twenty-two, and not one does." I reproduced every count with my own counter, and all match to the digit: 28/4, 322/0, 898/432, 250 orders with 72 deadlocked, 792,987/312,492, and 2,126,816,238/0. | Keep. |
| dev-16 | Dev | 10_1–10_2, 11_5–11_6 | delight | Two of my R1 complaints are fixed properly. The old-zero argument is now told in colours with the flag-down reason, and I can rebuild it myself. The bit widths are honest: "under two months" at 32 bits, "centuries" at 64, and "as long as a glance never returns more than was ever written". | Keep. |
| dev-17 | Dev | 12_7 panel | delight | The code is token-for-token the panel I compiled cleanly in R1; only the comments changed. Reading it line by line: the max loads each slate once (its own too, which is 0 at that point, so 3N−2 loads, as fact-sheet F5 says). `(nj, j) > (t, i)` is correct because j ≠ i. Waiting on the local `t` is fine because only customer i writes `number[i]`. SeqCst on every access gives the interleaving model the proof assumes. The new comments name the two traps I would hit: ties and the Drop guard. | Keep; see dev-04 for the highlights. |

## What I checked and how

- **Read:**
  - my persona and the whole R2 packet;
  - my R1 review and R1.md, to trace what happened to my findings;
  - `claims.md`, the full claim walk;
  - memo lines on sources and Assumption M;
  - `bakery_mc.py`'s step function and `outputs/scenarios_out.txt`, to settle the step convention;
  - the bead status of 7v0.5 (ESLint) and 7v0.17 (stress run): both are still open.
- **Recomputed with my own code** (scratch: `.../scratchpad/r2_dev_c/orders.py` and `sb.py`, run with `python3 -I`; neither shares code with the oracle):
  - the interleaving counts from dev-15, plus 6/4 and 14/8 for chapters 01 and 02;
  - the x86 store-buffer results (12/18/20/OK);
  - the out-of-order-draining counterexample in dev-02 (22 moves) and the alternative fence placement in dev-04.
- **Recomputed by hand or in a one-liner:**
  - 31,557,600,000 < 34,359,738,368;
  - 2³² ms = 49.71 days and 2³¹ ms = 24.86 days;
  - 2⁶⁴ at 10⁹ per second = 584.54 years, and 2⁶³ = 292.27 years;
  - 09 → 10 written a digit at a time reads 19 or 00;
  - Blue reads 3, 0, 0, 2 and writes 4; Purple and Teal read 4, 3, 0, 2 and write 5.
- **Script lint:**
  - No spoken line contains digits, colons, semicolons, dashes or parentheses.
  - 1,899 words, about 11.5–12.7 min of speech at 165–150 words a minute.
  - 08 is the longest chapter at 73–80 s of speech before its prediction pause and hold, so it is close to the 85 s cap.
- **Rust panel:** I diffed it against the R1 panel; only comments changed. I could not compile here, so the "it compiles" claim rests on token identity with my R1 compile (rustc 1.100.0-nightly, editions 2021 and 2024, `-D warnings`, clean) and the claim walk's compile.
- **Not checked by me**, relying on the fact sheet's rows:
  - 3,027,633 and 269,550;
  - the 33-step double-fetch trace and the 70-step trace d;
  - every quote, date and URL;
  - the TF-A, Linux, Apple, JLS, Rust-docs, Burns–Lynch and Abdulla claims.

Nothing in the repository was edited.
