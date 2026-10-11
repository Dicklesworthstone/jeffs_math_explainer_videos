<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# R2 review: Video C, "The Algorithm Lamport Is Proudest Of", as Ruth Adler (ruth_45)

**No blockers.** I recomputed every number the script speaks and every count it shows, and none is wrong. I re-fetched the primary sources for every spoken quote, and each one checks out verbatim. The claim walk's fixes have landed. The open problems are a term clash ("atomic"), the exact versions I was promised but cannot find yet (there is no description, and memo §4 still has its gap), always/never tests that don't appear in the fact sheet, and a handful of quantifier and vocabulary slips.

## Claim-walk fixes: did they land?

| walk item | now | verdict |
|---|---|---|
| 10_1–10_2 colours vs e3 | Green reads Blue's slate during Blue's write, gets 0 and walks in; Blue waits. | **Landed.** I replayed e3 on `bakery_mc`. At step 17 Green checks Blue's flag and finds it down (Blue is in the room). Blue starts choosing at 21. At step 25 Green reads Blue's slate mid 0→9 as 0 and enters. At step 30 Blue waits. The 10_2 reasoning is case (a), applied correctly. |
| 09_4–09_5 "never" | Now asks only about ticket reads. | **Landed.** The flag reads are answered only by a chapter-10 lower-third that has no line id (R13). |
| 11_5–11_6 signed/unsigned | "under two months" covers 49.7 d (u32) and 24.9 d (i32); "centuries" covers 584.5 y (u64) and 292.3 y (i64); the screen shows both. | **Landed.** The direction of the 32-bit bound is still wrong in the voice (R6). |
| 11_7 count | "three reads for every other customer" | **Landed**: 3(N−1). |
| 07_5 waits | "She waits for Purple, then Teal" | **Landed**: `C-F6.d.customer1_waits_behind` = (1,3)(2,4). |
| 10_3 quote card | verbatim "I discovered that the bakery algorithm had this property after writing a proof of its correctness" | **Landed**: checked against the publications page today. |
| 04_2 continuity | "Before anyone is served" | **Landed.** My recomputation gives 5, 5 and service order 5, 2, 1, 3, 4. |
| 12_3–12_4 wording and animated order | | **Not landed** (R11). |

## Comprehension check

| ch | what I'd tell a friend |
|---|---|
| 01 | `x = x + 1` is a read then a write. Of the C(4,2) = 6 interleavings of two increments, exactly the 4 that pass through "both read, neither wrote" lose an update. That's a count, not a probability, but at 10⁹ steps/s the rare timing happens constantly. |
| 02 | A check-then-flip sign fails (8 of 14 orders put both inside). Dijkstra's 1965 problem, plus Knuth's "everyone gets in", can be solved with several indivisible reads and writes. The open question is memory whose reads can catch a write halfway. |
| 03 | Each customer writes only her own slate and takes 1 + the largest ticket she reads (Blue: 3, 0, 0, 2 → 4), then waits for every smaller nonzero ticket. No shared ticket machine is needed. |
| 04 | Concurrent choosers can tie (Purple and Teal both read 4, both write 5). Comparing (ticket, customer number) like a dictionary, a strict total order, removes both the double entry (`<`) and the deadlock (`≤`). |
| 05 | Without a flag, a customer can be judged on an unwritten ticket: 4 of 28 orders put two inside. Waiting for others' flags to drop before comparing gives 0 of 322. The flag isn't a lock and doesn't close the gap; it makes the gap visible. |
| 06 | Three customers, one visit each: 2,126,816,238 orders, and none puts two inside. |
| 07 | In a four-customer run the flag stops Purple from judging Green's unwritten ticket, Teal takes 2, the tie at 1 goes to Green, and service follows the (ticket, number) order exactly. |
| 08 | Fact one plus a three-way split on where Green was at Blue's last successful flag check gives fact two: the customer inside holds the least pair. Asymmetry of < then forbids two inside. A bounded search (N = 4, tickets ≤ 4, indivisible reads and writes) agrees, but only the proof covers every N. |
| 09 | Even if a read that overlaps a write returns any value in range (a "safe" register), the proof never uses such a ticket read. It trusts only two reads of finished, unchanging slates, so high garbage just pushes someone back. |
| 10 | Reading Blue's pre-write 0 lets Green in, and that is harmless: Green had seen Blue's flag down, so Blue started choosing later and fact one makes her wait. Lamport noticed the property after the proof. The max must read each slate once. |
| 11 | It is FCFS from the end of choosing, and with each other customer bypassing you at most once after you write your ticket, nobody starves under the stated conditions. Tickets are unbounded: under 2³⁵ after a year at 1 per ms if reads never overshoot, so 32 bits can wrap in about 50 days and 64 bits last at least 292 years at 10⁹/s. It costs 3(N−1) reads, and a frozen chooser stalls everyone. |
| 12 | The store buffer lets a later read overtake an earlier write, breaking "a finished write is seen by every later read", so both can enter. Two fences per entry on x86, or SeqCst atomics everywhere, restore the order. Acquire/release don't order a store before a later load. |
| 13 | Arm's reference firmware uses a bakery lock where exclusive-access spinlocks can't work: a core waking with its cache off, on non-coherent platforms. Bigger-than-any-seen plus a tie-break reappears in Lamport clocks and Paxos ballots. Lamport says he discovered it, and is proudest of it. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 5 | 5 | 5 | 4 | 4 |
| 02 | 4 | 4 | 4 | 4 | 3 |
| 03 | 5 | 5 | 5 | 4 | 4 |
| 04 | 4 | 4 | 4 | 4 | 4 |
| 05 | 5 | 5 | 5 | 4 | 5 |
| 06 | 4 | 5 | 3 | 4 | 4 |
| 07 | 4 | 5 | 4 | 3 | 4 |
| 08 | 4 | 5 | 5 | 3 | 5 |
| 09 | 4 | 4 | 5 | 4 | 5 |
| 10 | 4 | 5 | 4 | 4 | 5 |
| 11 | 4 | 4 | 4 | 3 | 4 |
| 12 | 4 | 4 | 4 | 4 | 4 |
| 13 | 4 | 4 | 4 | 4 | 5 |

Chapter 08's correctness went from 1 to 5 since R1.

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R1 | ruth_45 | 12_7, 13_1 (02 Ruth note) | major | "Atomic" appears first at 12_7, where "sequentially consistent atomics" means the language's atomic loads and stores. At 13_1, "magic signs, called atomic instructions" means read-modify-write. The word is never defined, though the 02 Ruth note promised it would be defined once, as one indivisible step, and never used loosely. Meanwhile the chapter-10 lower-third says aligned plain loads and stores don't tear, which makes them atomic too. Heard back to back, 12_7 → 13_1 suggests the recommended fix *is* the magic sign the bakery avoids, and that undercuts 13_1's question. There is a real irony worth a footnote: on x86 a SeqCst store is a locked XCHG (or MOV + MFENCE) under the recommended mapping (C-S14), used only as a fence, never for exclusion. | 02_7 lower-third: "a step nobody can catch halfway is called atomic". 12_7: "…is sequentially consistent atomic reads and writes." 13_1: "…the hardware's magic signs, instructions that check and flip in one indivisible step", with a lower-third "atomic read-modify-write: test-and-set, compare-and-swap, fetch-and-add". Put the XCHG-as-fence remark in the description. |
| R2 | ruth_45 | 04 (R13), after 07_5, 08 "Precisely", 10 (D5), 11 costs card and "Precisely"; D4 | major | The R2 rule says every persona note reaches the narration, a card, a lower-third or the description. There is no description draft, and no fact-sheet "simplification → exact" list (plan §5.1 rule 11). So the items routed to "the description" cannot be checked: the R13 counterexample, the exact Lemmas 1–2 and Assumption M, the 1974 single-bit footnote, and D4's Turing line. (I verified D4 today: "A literal implementation of this approach would require a ticket-machine process that never halts, violating Dijkstra's requirements.") Two notes still say "X or the description": the TS comma-operator aside after 07_5 and the 11 costs card. Chapter 11's "Precisely" bullet names no destination at all. I cannot find the exact versions I was promised. | Before narration, add a "Precisely / description" ledger (a storyboard section or a `publish/youtube.md` draft), one row per item: line id → exact text → where it shows. Pick one destination for each "or". Give 11's "Precisely" a home (a lower-third at 11_1–11_2, or the description). |
| R3 | ruth_45 | 08 "Precisely (description)"; MEMO.md §4 | major | The 08 note sends the description to "Lemma 1 and Lemma 2 exactly as in the memo §4", with the corrected case (b). The memo is still uncorrected (lines 326–339). It says Bᵢ runs "…to the end of the exit write", that "Throughout Bᵢ, number[i] is constant" (false during a safe exit write), and gives case (b) as "Dₖ ended before r₂ ended", which misses r₂ overlapping a closing write that ends after r₂. R5 was deferred to bead mx-e-c-7v0.5, to be done "with the script's claim walk". The walk is done, the memo is unchanged, and the bead is open. A description copied from the memo will publish the gap. | Fix memo §4 now: case (b) becomes "Dₖ's closing write began before r₂ ended"; Bᵢ ends where the exit write *begins*; use one definition of "in the bakery" in both lemmas. State the bypass corollary from the end of the ticket write, which is what 11_2 says and what I checked. Copy the corrected lemmas into the R2 ledger. |
| R4 | ruth_45 | 05_8, 08_3, 04_6, 11_1, 11_2, 11_3, 11_7; 11 "Precisely" | major | The audit requires each always/never sentence to be tested at its edges, with the test named in the fact sheet. These tests exist, run by the walker (G1–G5, in uncommitted scratch) and by me, but none is a fact-sheet row or an oracle key. They cover: many flags up at once; fact two for *every* holder, including one whose flag is still up (C-X2's Assertion-2 ghost checks only holders at L2/L3); "at most once" counted from the ticket write; tickets reset only when the bakery empties; "stalls everyone"; "either of us may go first"; 04_6 for simultaneous holders; and "the double fetch needs garbage", which only memo §3.2's table backs. | Add bounded oracle keys, for example `C-X4.fact_two_all_holders.n3_k5_atomic.violations = 0`, `C-X5.bypass_from_ticket_write.n3_k4_atomic.worst = 1`, `C-F6.double_fetch.n3_k5_atomic.violation_found = 0`, plus G1, G3 and G5. Add a fact-sheet row per line id that names its key. My versions are in scratch `mc_checks.py` (about 60 s in all). |
| R5 | ruth_45 | 04_3 | minor | "If equal tickets both go in, both end up in the room" states a certainty. From the 04_2 state under `<` with no new arrivals, there are runs with Purple and Teal inside together and runs where they never are (BFS on `bakery_mc`). The `≤` half ("forever") is certain. | "If the rule lets equal tickets both go in, both can end up in the room at once." |
| R6 | ruth_45 | 11_4–11_5 | minor | The voice gets the direction of the bound wrong. 2³² ms = 49.7 d is the *earliest* a 32-bit ticket can wrap, and only if the line never empties at the full rate, so "would wrap" overstates it. "Below about thirty-four billion" hedges a strict bound. The 11_4 card's 31,557,600,000 quietly assumes a 365.25-day year. | "Below two to the thirty-five, about thirty-four billion. Too big for a thirty-two bit number, which could wrap in under two months." Card: "ms in a 365.25-day year" (365 days gives 31,536,000,000, also < 2³⁵). |
| R7 | ruth_45 | 11_1–11_2 | minor | "So nobody waits forever" derives starvation-freedom from 11_1's FCFS. FCFS holds vacuously for the `≤` variant, which deadlocks. The step also needs deadlock-freedom: whoever holds the smallest pair gets through (memo Assertion 3; fact-sheet F9). This is the FCFS vs starvation-freedom distinction the audit lists. | "And since whoever holds the smallest pair always gets through, nobody waits forever, as long as…", or just drop "So". Add "FCFS + deadlock-freedom ⇒ no starvation (Lamport 1986)" to 11's "Precisely". |
| R8 | ruth_45 | 03_1, 04_6, 07_5 (also 05_3, 07_4) | minor | "Number" means three things: the ticket (03_1 "the lowest number is served next"), the customer number (04_6 "a customer with the lowest number still waits", 07_5 "with the lowest number") and a machine integer (11_5–11_6). The same phrase plays opposite roles at 03_1 and 04_6, so a careful listener hears a contradiction. | 03_1: "Take a number, and the lowest ticket is served next." 04_6 and 07_5: "the lowest customer number". 05_3 and 07_4: "customer number". |
| R9 | ruth_45 | 04_6 | minor | "That's the order in which the room takes everyone holding a ticket." Everywhere else, "order" means an interleaving (01, 02, 05, 06, 08_9), and Maya's m20 reserved "who's served first" for the queue. | "That's who the room takes first, among everyone holding a ticket." |
| R10 | ruth_45 | 08_3 (04_4) | minor | "Pair" is first spoken at 08_3 ("a bigger pair") and never defined aloud. At 04_4 the pair is only drawn. | Append to 04_4: "Ticket and customer number together make a pair." |
| R11 | ruth_45 | 12_3–12_4 | minor | "Each reads the other's slate … No flag" and "the real slates still say nobody is even choosing" call flags slates, though 09_4 keeps them apart. The recorded 12-step trace (`bakery_tso_out.txt` lines 2–13) has P1 enter before P2 starts. That is not the voiced order, where both buffer and then both read. The voiced order is also a 12-step violation with memory at zero, but nothing pins it. | "Then each checks the other's flag and ticket." / "…while the real flags and slates still say nobody is even choosing." Add the voiced schedule as `C-F9.voiced.*` and cite it in 12_3's picture. |
| R12 | ruth_45 | 02_5 | minor | "Can't block everyone" can be heard as not-all (so some may be blocked) or as not-anyone. Dijkstra's (c) is "not allowed to lead to potential blocking of the others". | "Someone who stops outside the room can't hold up the others." |
| R13 | ruth_45 | 09_4–09_5 (lower-third filed under 10) | minor | 09_4 makes the flags flicker too, but 09_5's "Never" covers only ticket reads, and the proof does trust Blue's last flag read. The answer, "a glance at a flag mid-raise counts as not yet choosing; mid-lowering, as finished", is filed under chapter 10 without a line id. It could appear after the question has closed. | Pin that lower-third to 09_5. |
| R14 | ruth_45 | 01, 04 and 10 Ruth notes | minor | Three notes have no destination: "six orders = C(4,2)"; "≺ is a strict total order with finitely many predecessors"; and the safe-register search (N = 3, tickets ≤ 6, 269,550 states, no violation). Because of the last one, the viewer hears "a computer search agrees" (08_9) only for indivisible reads, never for flickering memory. | 01_7 picture: "6 = C(4, 2): pick which 2 of the 4 steps are Blue's". 04_6 lower-third: "precisely: pairs are totally ordered; each has finitely many ahead of it". 09_5 or 10_3 card: "flickering slates and flags, N = 3, tickets ≤ 6: 269,550 states, no violation (model)". |
| R15 | ruth_45 | 06_2 (picture) | minor | The trails are "sampled uniformly over orders" (E-C2), but nothing on screen says what is random. After 01_9's "counting orders isn't measuring chances", unlabelled samples will be read as typical runs. | Caption: "200 of 2,126,816,238 orders, drawn uniformly, not as a real machine would". |
| R16 | ruth_45 | 10 (D5), 11 costs card, 12 lower-third | minor | Three sourced claims rely only on the memo, with no fact-sheet row. (a) The 1974 footnote: I verified it on the publications page today ("In a footnote, it claims that we can consider reads and writes of a single bit to be atomic… It was only later… that I realized the fallacy in this reasoning."). (b) Burns & Lynch 1993: not checked by me. (c) Abdulla et al., TACAS 2012: checked. Table 1 row 4 gives "2 per process", 4 minimal sets, and the text says "one at line 3 and the other at either line 10 or line 12" (2 processes, numbers ≤ 2). So 12_5's placement is one of the minimal sets, and "matches" holds. | Add C-S18 (the footnote passage), C-S19 (Burns & Lynch, Inf. & Comp. 107(2):171–184, the theorem verbatim with page) and C-S20 (Abdulla, row 4 and the quoted sentence). |
| R17 | ruth_45 | 02_4, 05_7, 06_2, 08_9, 09_6, 12_4 | minor | `facts.rs` pins 15 keys; all 15 equal today's TSV. None of them is a count that is both spoken and drawn on a counter: 14/8, 28/4, 322/0, 2,126,816,238, 3,027,633, 7→8, 12 steps (plan rule 12). The two new pins (24.9 and 292.3) are uncommitted in the working tree. | Pin C-F2, C-F3, `C-F4.flag.orders`, `C-F8.n4_k4_atomic.states`, `C-F6.e1.*` and `C-F9.no_fence.shortest_violation_steps` before any scene draws a counter. |
| R18 | ruth_45 | brief.md | minor | The brief contradicts the storyboard: hues "blue, green, orange, purple, pink" (now Teal); "garbage reads flickering magenta" (now white static); "customer 4 takes 4" and "both see 5, both take 6"; 11 chapters at about 13 min (now 13 chapters, about 14). | Update the brief's colour roles, worked examples and length. |
| D1 | ruth_45 | 10_1–10_2 | delight | Now exactly trace e3, with the real reason: the flag was seen down, so the writer started later, so fact one applies. It is the proof's case (a) running live. | Keep. |
| D2 | ruth_45 | 09_4–09_5 | delight | "Does the proof ever trust a ticket read that crosses a write? Never. It trusts just two." That is Lamport's observation at the right generality. I'd use it in class. | Keep, with R13. |
| D3 | ruth_45 | 13_4 | delight | A footnote I didn't know, verified on CHM p. 9. Levin asks why it's the one he's proudest of. Lamport: "No, I can't." (laughter) He answers on p. 10. On discovered vs invented: "I'm not sure I understand the distinction either." | Use one as a 13_4 caption, or in the description (CHM pp. 9–10). |
| D4 | ruth_45 | 09_2 / description | delight | From the publications page, verified: "Several books have included emasculated versions of the algorithm in which reading and writing are atomic operations, and called those versions 'the bakery algorithm'. I find that deplorable. There's nothing wrong with publishing a simplified version, as long as it's called a simplified version." The video does exactly this: Act II assumes indivisible reads (08_9's card says so), and 09–10 lift the assumption. | Quote it in the description or as a 09_2 lower-third. |
| D5 | ruth_45 | 13_1 | delight | Turing lecture §2.2, verified: "Although of little if any practical use, the bakery algorithm…". That is Lamport's own dismissal, and 13_2 answers it. | Show it on a 13_1 card, then 13_2's "Yes." |

## Mathematician audit

1. **Quantifiers explicit:** fails, R5, R6, R12. These pass: 01_8 "exactly", 05_7 "not one", 06_2, 08_9's bounds, 09_5 "never" (for ticket reads), 11_2 "at most once".
2. **Always/never tested at edges and named in the fact sheet:** fails, R4. The claims themselves hold in my re-runs.
3. **Probability space stated:** the voice passes (01_9 and its lower-third). Fails for the 06_2 picture, R15.
4. **No probability attached to a fixed unknown:** pass; there is no instance in this video.
5. **Terms in their standard sense:** fails. Atomic instruction vs atomic register is R1; FCFS vs starvation-freedom is R7. The doorway passes: it is shown once on screen at 05_8 and equals "choosing".
6. **Exact version recorded for every simplification:** fails, R2, R3, R14.
7. **Historical claims verified:**
   - The spoken quotes and dates pass. I re-fetched the Turing lecture (2496; "In 1973, this was considered impossible [4, page 88]"), the 1974 paper ("The read may return any arbitrary value!"; the 2³⁵ sentence; "Received September 1973"), the publications page (7456, "did not invent… discovered", "after writing a proof of its correctness", Holt, "called safe", the footnote) and CHM pp. 9–10 (with the hedge "not quite in the same way" on Paxos).
   - Rows are missing for three claims: R16.
   - 09_3's "twelve years" holds for 1986 in print. The SRC report is dated Dec 1985, and Lamport says he had the results "in the late 70s"; keep C-S2's caveat.
8. **Numbers equal the oracle; voice and screen round alike:** pass, all recomputed (see below). R6 is about the direction of a bound, not a value; R17 is about pins.
9. **Edge cases acknowledged:** pass. Ties (04), zero meaning "not in line", overflow with Lamport's condition (11_6), high garbage vs the old value (09_6 / 10_1), N = 1 ("even alone").
10. **Notation consistent, no symbol used for two things:** fails, R1, R8, R9, R10, R11, R18.
11. **Nothing true but misleading:** R1 is borderline, plus R5 and R6. Hardware honesty (12) and the TF-A narrowing (13_2 plus its lower-third) pass.
12. **Every persona note reached the video or was dropped with a reason:** fails, R2, R14. All other notes are routed: the 01, 03 and 12 lower-thirds, 05_7, 05_8, 05_9, the 09_3 register names, 11_6's verbatim condition, 13_2, the 13 quotes, and the deferred ESLint check with its reason.

## What I recomputed, and how

All scratch is in `…/scratchpad/r2_ruth_c/`, run with `python3 -I`. Nothing in the repo was changed, and I compiled nothing.

- **`counts.py`, my own memoised interleaving counter.** It uses the fact sheet's conventions but none of the oracle's code. Results: lost update 6 / 4 (equal to C(4,2) and to the middle set); sign 14 / 8 / 0; two customers with no flag 28 / 4 / 0; with flag 322 / 0; `<` 898 / 432; `≤` 250 / 0 / 72 deadlocked; three customers with no flag 792,987 / 312,492; with flag 2,126,816,238 / 0. All equal the oracle.
- **`arith.py`, exact fractions.**
  - 31,557,600,000 < 2³⁵ = 34,359,738,368, and the bound fails at 2³⁴.
  - 2³² ms = 49.71 d and 2³¹ ms = 24.86 d (both under two months).
  - 2⁶⁴ / 10⁹ s = 584.54 y and 2⁶³ / 10⁹ s = 292.27 y (in 365-day years, 584.94 and 292.47).
  - 3(N−1) for N = 2…8.
  - Blue: 3, 0, 0, 2 → 4, served (2, Pink), (3, Green), (4, Blue). The tie gives 5, 5 and service 5, 2, 1, 3, 4. Rewriting 09 → 10 a digit at a time shows 19 or 00.
- **`mc_checks.py`.** It uses `bakery_mc`'s transition relation; the checks are mine.
  - Replayed e3 (the D1 row).
  - Double-fetch max with atomic reads: no violation at N = 3, K = 4 (30,762 states); N = 3, K = 5 (45,073); N = 4, K = 3 (5,014,648).
  - 04_3 under `<`: Purple and Teal together is reachable and also avoidable.
  - Fact two over every holder of a nonzero slate: atomic N = 3, K = 5 gives 27,778 states, 2,993 checks, 0 violations; safe N = 3, K = 4, excluding customers mid-exit-write, gives 92,716 states, 5,611 checks, 0 violations.
  - Bypass counted from the ticket write: worst case 1 for atomic N = 3, K = 4 (45,868 ghost states) and for safe N = 3, K = 3 (102,364).
- **Other checks.**
  - Diffed the packet's script against `script.tsv`: identical.
  - Compared the 15 `facts.rs` pins with today's TSV: all equal.
  - Read the de-slopify word diff since the claim walk; every re-worded line is covered above.
  - Read the Abdulla et al. PDF (Table 1, §4) and the recorded TSO trace.

## Unchecked

- The TSO counts and fence results. I took them from the TSV; I only reasoned that the voiced 12_3 order is a 12-step store-buffer violation with memory at zero.
- 3,027,633 and 269,550 today. They come from the oracle's live recount in the TSV; I re-ran both myself at R1.
- The step counts of traces a′, b1, b2, d and the double-fetch trace, and the leapfrog trace. I read the d and e1 schedules but did not re-run their keys.
- The fact sheet's C-S rows, which I did not re-fetch: Dijkstra 1965, Knuth 1966, mutual2, interprocess, Time-Clocks, Paxos, TF-A, Intel, AMD, Apple, C++, Rust and Java.
- Burns & Lynch 1993.
- rustc's output for the panel, and ESLint (deferred).
