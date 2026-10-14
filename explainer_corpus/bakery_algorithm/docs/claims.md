<!-- Claim walk (gate G3) of script.tsv at 49ef758, 2026-10-11: the walker's final answer, verbatim (scratch paths shortened to <scratch>/). Fixes are applied in the storyboard; see the commit that lands this file. -->

# Claim walk — Video C (script at HEAD)

## Summary

I walked 130 claims across the 82 spoken lines, plus the factual lower-thirds, cards and code panel. **90 OK · 5 MISMATCH · 14 UNSOURCED · 19 EDGE (2 fail as worded, 1 has a continuity gap, 16 hold) · 2 unchecked** (general knowledge, no source consulted).

**Problems, worst first**
1. **MISMATCH, 10_1–10_2, colours reversed against trace e3.** The storyboard shows trace e3 here. In e3, Green reads Blue's slate while Blue is writing 9, gets 0 and walks in, and Blue waits. The voice says Blue reads Green's old zero and goes in, and Green waits. The logic is right, but the roles are swapped. I replayed the trace with the two customers swapped and it works: 30 steps, Blue enters at step 25, Green waits, never two in the room. Either add that swapped trace to the oracle (and swap e1 too) or swap the colour names in the words.
2. **MISMATCH, 11_7, "reads every other flag and slate, three times over".** The real count, 3(N−1), is three reads per other customer: her slate twice and her flag once. Heard literally, the line means six. Fix: "makes three reads for every other customer: her slate twice and her flag once."
3. **MISMATCH, 07_5, "She waits behind ticket two".** In trace d, Blue first waits on Purple's ticket one (step 49), then on Teal's ticket two (step 54). Fix: "She waits behind tickets one and two."
4. **EDGE fails as worded, 09_4–09_5, "Never. It relies on just two glances."** 09_4 lets the flags flicker too, and the proof does rely on Blue's last flag check, which can catch a flag mid-flip (memo §4 handles that case). "Never" is true only for ticket reads. The algorithm itself is fine: the model check with flickering flags and slates finds no violation. Fix: ask about *ticket* reads only, or add a clause about the flag check.
5. **EDGE fails for signed numbers, 11_6, "over five hundred years".** That holds only for an unsigned 64-bit number (584.5 years). A signed one lasts 292.3 years, and the costs card in the same chapter recommends Java `AtomicLongArray`, which is signed. Likewise, 11_5's "about fifty days" is the unsigned figure; a signed 32-bit number wraps in 24.9 days.
6. **MISMATCH (on screen, if set in quotation marks), 10_3 quote card "noticed after writing the proof".** Those aren't Lamport's words. His are "I discovered that the bakery algorithm had this property after writing a proof of its correctness".
7. **EDGE continuity gap, 04_2.** "Both see four" needs Blue, Green and Pink still holding 4, 3 and 2. But 03_5–03_6 serve all three, which would leave their slates at 0, and Purple and Teal would both take ticket one.
8. **UNSOURCED:**
   - 05_9: the claim about Lamport's intent ("didn't try to close the gap").
   - 12_6: "Arm chips, like the ones in recent Macs" and "the fences go in more places". There is no Arm model in the repo.
   - 12_7: "sequentially consistent atomics are correct everywhere" and "acquire and release are not enough". The second can be backed by the 12-step x86 failure plus the standard compiler mapping (acquire/release become plain moves on x86).
   - 12_5: AMD is not cited.
   - 13_2: "inside many Arm chips". It is firmware, not silicon; it uses the bakery lock only on platforms without hardware coherency; and nothing counts how many.
   - 01_9: "almost always back to back" and "billions of steps a second".
   - On screen: the `add [x], 1` lock-prefix lower-third; Java `AtomicLongArray`/`volatile long[]`; Rust `volatile`; "SeqCst on x86 fences every store"; the x86 half of "today's chips don't tear an aligned word".
9. **Note, 12_3–12_4.** The recorded shortest x86 failure has customer 1 walk in before customer 2 starts. The voiced order (both write into their outboxes, then both read) is a different interleaving. I replayed it and it also fails in 12 steps with memory at zeros, so animate the voiced order.

**Colour-named scenarios**
- **03_4–03_5:** consistent. I model-checked Blue as the reader: she reads 3, 0, 0, 2, writes 4, and every interleaving serves Pink, Green, Blue. The oracle's own model check still uses customer 4, so it should get keys for customer 1.
- **04_2–04_6:** consistent, apart from the continuity gap in item 7. My replay serves Purple before Teal, and trace d agrees that ticket two (Teal) goes before ticket three (Blue).
- **05_1–05_6:** match traces a and a′ step for step, including Green stopped by Blue's flag at step 7.
- **07_2–07_5:** match trace d, except "behind ticket two" (item 3).
- **10_1–10_2:** reversed (item 1). These lines weren't in your list, but they are colour-named.

**Numbers spoken as words:** all 82 spoken lines contain no digits or symbols; I scanned for them. `script.tsv` matches the storyboard's words column exactly.

**OK, but verified only by me today against the primary source** (no row in the repo yet; add one):
- 02_1 and 02_5: Dijkstra 1965's wording and rules. The storyboard itself asks for this row before narration. His rule (d), no "After you"–"After you" blocking, already required that *somebody* gets in, so Knuth's "everyone" really is "more".
- 09_3: "safe" was named in the 1986 paper.
- 10_3: Holt was "a former colleague at Massachusetts Computer Associates".
- 13_3: the *Time, Clocks* tie-break sentence, and the two *Paxos Made Simple* passages (numbers bigger than any used; replicated copies).

**What I ran**
- **Oracle:** your `uv run … oracle.py`. It finished with 173 keys and 53 of 53 cross-checks passed, but it took **397.7 s, not about a minute**: the four-customer sweep alone took 392.7 s.
- **Trace replays** on the research model checker (`bakery_mc`): traces d, e3 and e3 with the customers swapped.
- **Colour examples in the model:** Blue's 03 example; the 04_2 example with every serving order.
- **x86 store-buffer model:** replayed the voiced 12_3 order.
- **Edge checks, three customers** (atomic with tickets up to 5; flickering registers with tickets up to 4): all flags up at once (05_8); "everyone else holding a ticket has a bigger pair" (08_3); the largest ticket drops only to zero (11_3); at most one bypass after taking a ticket (11_2); a customer frozen mid-choosing stalls everyone after at most one more entry each (11_7).
- **Small checks:** the 02_4 sign orders; 11_1 "either may go first"; the signed 32-bit wrap.
- **Rust panel:** compiled the 12_7 panel with rustc.
- **Sources fetched:** Lamport's publications page, `time-clocks.pdf`, `paxos-simple.pdf`, `mutual2.pdf`, and a scan of Dijkstra 1965.
- **Not run:** `./x bakery_algorithm facts-check` and `cargo test`. I compared the 13 values pinned in `facts.rs` with the oracle output by eye; all match.

**Housekeeping**
- `fact-sheet.md` is out of date: it says 168 keys; it has no rows for the `C-F7.customer1_*` keys or `C-F10.u32_wrap_days_at_1_per_ms_1dp`; it cites old line ids; and its C-F7 rows still describe customer 4.
- I edited nothing in the repo; `git status` is clean. Scratch files are in `<scratch>/`: `oracle.tsv`, `oracle.err`, `checks*.py`, `table.md`, and the downloads in `web/`.

## Table

| line | claim | source | status | note / fix |
|---|---|---|---|---|
| 01_1 | A slate going from nine to ten, glanced at mid-stroke, might read nine, ten or, "Lamport says", 2496 | `C-F14.turing_lecture_garbage.from/to/value` = 9 / 10 / 2496; fact sheet open item 2 (turing.pdf p. 4, verbatim) | OK | Lamport's sentence is conditional ("correct even if reading … obtains the value 2496"). "You might read … Lamport says" is a fair reading of his model, not a claim about real chips. Card digits are full size (F8). |
| 01_2 | In 1973, taking turns safely with memory like that was considered impossible | `C-F14.considered_impossible.year` = 1973; open item 2 (verbatim, citing Brinch Hansen p. 88); finding F6 | OK | F6 says this paraphrase is fair; don't widen it. |
| 01_4 | The scoreboard reads forty-one | `C-F1.score_start` = 41 | OK | |
| 01_4 | `x = x + 1` isn't an equation; it means make x one bigger | general knowledge (assignment) | unchecked | No source needed or consulted. |
| 01_5 | That's two steps: read, then write back plus one | C-F1 model (`row_cf1`: R then W per program) | OK | |
| 01_5 (screen) | Even `add [x], 1` reads and writes separately unless it has a lock prefix | memo §5 Ch1, a proposal only | UNSOURCED | True as far as I know. Needs an Intel SDM citation (LOCK prefix and atomicity of locked RMW). |
| 01_6 | Both read 41, both write 42; two points scored, one counted | `C-F1.score_end_when_a_point_is_lost` = 42; `score_end_otherwise` = 43 | OK | |
| 01_7 | There are six orders | `C-F1.orders` = 6 (= C(4,2)) | OK | |
| 01_8 | Four lose a point | `C-F1.orders_lose_a_point` = 4 | OK | |
| 01_8 | "Exactly" the ones through the middle (both read, neither written) | `C-F1.losing_orders_are_exactly_the_middle_orders` = 1; `orders_through_middle` = 4 | OK | The two sets are equal, so "exactly" holds. |
| 01_9 | Counting orders isn't measuring chances; lower-third "6 orders, 4 bad: a count, not a probability" | C-F1; definitional | OK | |
| 01_9 | A program almost always runs both steps back to back, so on a real machine the bad orders are rare | none | UNSOURCED | Plausible, but nothing in the repo measures it. Cite a measurement or say "usually". |
| 01_9 | At billions of steps a second, rare means constantly | memo §5 Ch1 ("rare × 10⁹ per second"), a proposal only | UNSOURCED | General knowledge (GHz clocks). Fine as rhetoric; add a one-line source if a row is wanted. |
| 02_1 | Only one program at a time may touch it (lower-third: "critical section", "mutual exclusion") | Dijkstra 1965, CACM 8(9):569: "critical section" … "this mutual exclusion of critical-section execution" (I read a scan of the article) | OK | |
| 02_4 | Both read vacant before either flips it | naive-sign enumeration (`oracle.enumerate_paths`), re-run by me | OK | In all 8 both-inside orders, both reads come before either flip (12 of the 14 orders have both reads first). |
| 02_4 | Of the fourteen orders, eight put both in the room at once | `C-F2.orders` = 14; `C-F2.orders_both_inside` = 8 | OK | F1's fix ("at once") is applied: `orders_ending_with_both_inside` = 0. |
| 02_5 | Dijkstra posed the problem in 1965 | `C-F14.dijkstra_problem.year` = 1965 (CACM 8(9):569) | OK | Dijkstra calls it an open question "since at least 1962", so "posed properly" is the right verb. |
| 02_5 | Rules: never two in the room; someone who wanders off outside can't block everyone; fast or slow, it still works | Dijkstra 1965 (scan): "only one … in its critical section"; (b) "Nothing may be assumed about the relative speeds"; (c) "stopped well outside its critical section … not allowed to lead to potential blocking of the others" | OK | **No repo row.** The storyboard (R9) requires a verbatim row, so add these. R9's question is answered: rule (d), "After you"-"After you" blocking, already required that *somebody* gets in, and Lamport 1986 Part II confirms it. So 02_6's "asked for more" (everyone) is consistent. |
| 02_6 | A year later Knuth asked for more: everyone who wants in gets in eventually | `C-F14.knuth_eventual_entry.year` = 1966; Lamport 1986 Part II (I fetched mutual2.pdf): "Lockout Freedom Property … This requirement was first stated and satisfied by Knuth in [6]" | OK | |
| 02_7 | Hardware usually has a magic sign you can check and flip in one motion | memo §8 (atomic RMW: CAS, test-and-set); fact sheet open item 6 (x86 `xadd`) | OK | |
| 02_7 | Dijkstra showed ordinary signs are enough, several used cleverly, as long as nobody catches one mid-flip | memo §0.1 / §2.3 [V]; Dijkstra 1965 scan: reads and writes "are undividable operations"; the solution uses arrays b, c and k | OK | Dekker solved two processes first (EWD123). Dijkstra's solution is for N, so "showed" is fine. |
| 03_2 | A shared ticket machine could hand two customers the same ticket, like the lost point, and would need a room of its own | C-F1 (read-then-write loses updates) | OK | Lamport gives a different reason (Turing lecture: "a ticket-machine process that never halts, violating Dijkstra's requirements"). The script's reason is still valid. |
| 03_3 | Each customer has her own slate and only she writes on it | 1974 paper (memo §2.1); publications page: "Each memory location is written by only one process" | OK | |
| 03_3 | A zero means not in line | 1974 program (memo §2.1) | OK | |
| 03_4 | Take a ticket by reading every other slate one at a time and writing one more than the biggest | 1974 program (memo §2.1: maximum's arguments read separately) | OK | |
| 03_4 | Blue reads three, zero, zero, two | `C-F7.slates_before` = 0,3,0,0,2; `C-F7.customer1_reads` = 3,0,0,2 | OK | Blue = customer 1, so she reads customers 2–5. |
| 03_5 | She writes four | `C-F7.customer1_ticket` = 4; `slates_after_customer1` = 4,3,0,0,2 | OK | |
| 03_5 | Served Pink (2), then Green (3), then Blue | `C-F7.service_order_customer1` = 5,2,1; my bakery_mc BFS (N = 5, Green 3 and Pink 2 waiting, Blue's doorway, then every interleaving of 1, 2 and 5) gives the single order (5,2,1) | OK | The oracle's `C-F7.model.*` keys still model-check customer 4 only. Add customer-1 model keys (the check above is all it needs). |
| 03_6 | Only one tune sounds at a time (only one in the room) | C-F3, C-F4, C-F8 (mutual exclusion) | OK | |
| 03_7 | No shared machine; "nobody ever" writes on anyone else's slate | 1974 single-writer design; bakery_mc writes only index i | EDGE (holds) | Edge: crash recovery. Lamport has a failed process's memory read zero; if code resets a dead customer's slate from another thread, that is a second writer (costs card: "in code that's your job"). |
| 03_7 (screen) | A ticket lock is the machine Lamport avoided: one atomic fetch-and-add (Linux x86 FIFO ticket spinlocks, 2008) | fact sheet open item 6 (commit 314cdbef, v2.6.25, LWN) | OK | |
| 04_2 | Yes, two customers can get the same ticket | `C-F6.c.tickets` = 1,1; `C-F6.c_three_customers.tickets` = 2,2,1 | OK | |
| 04_2 | Purple and Teal read at the same moment, both see four, both write five | illustrative (F9). I replayed it in bakery_mc (N = 5, slates 4,3,0,0,2): Purple reads [4,3,0,2], Teal reads [4,3,0,2], both write 5. Every interleaving then serves Pink, Green, Blue, Purple, Teal | EDGE (holds, with a continuity gap) | It holds only while Blue, Green and Pink still hold 4, 3 and 2. But 03_5–03_6 serve all three, and their slates would then be 0, so Purple and Teal would both take **1**. Fix: present 04_1 as "back to Blue's slates, before anyone is served", or keep 03_5–03_6 from playing the service out. |
| 04_3 | If equal tickets both go in, both end up in the room | `C-F6.b1.in_room_at_end` = 2 (12 steps); `C-F5.strict_lt_with_flag.orders_both_inside` = 432 / 898 | OK | |
| 04_3 | Wait on smaller-or-equal and they both wait forever | `C-F6.b2.deadlocked` = 1; `C-F5.le_with_flag.orders_deadlocked` = 72 / 250; `C-X3.liveness.*.le_numbers_only.no_progress_state_found` = 1 | EDGE (holds) | "Forever" holds: in b2 the only enabled moves are self-loops. |
| 04_4 | Tie-breaker: equal tickets go to the lower customer number, like a dictionary | 1974 order on pairs (memo §2.1) | OK | |
| 04_4 | Purple (3) goes before Teal (4) | (5,3) ≺ (5,4); my 04_2 replay serves Purple before Teal in every interleaving | OK | |
| 04_5–04_6 | Blue holds 3, Teal holds 2, so Teal goes first | (2,4) ≺ (3,1); trace d step 41 (Teal passes Blue's 3); `C-F6.d.service_order` puts (2,4) before (3,1) | OK | This previews trace d. Note that Blue held 4 in 03, so this is a fresh example. |
| 04_6 | The lowest customer number still waits for every smaller ticket; the number only breaks ties | pair order; memo §0.3 [V, model-checked] | EDGE (holds) | |
| 04_6 | That's the order in which the room takes everyone holding a ticket | `C-F6.d.service_order_is_lexicographic` = 1; my check G2 (below) | EDGE (holds) | Holds for customers holding tickets at the same time (Ruth R13). A newcomer whose choosing overlapped can still slot in ahead (see 11_3). |
| 05_1 | Blue reads Green's slate as 0 and plans ticket 1, unwritten | trace a (SCHED_A steps 1–2); `C-F6.a.steps` = 8 | OK | |
| 05_2 | Green takes 1, reads Blue's 0, walks in | trace a steps 3–6 | OK | |
| 05_3 | Blue writes 1, wins the tie on her lower number, walks in: both in the room | trace a steps 7–8; `C-F6.a.in_room_at_end` = 2 | OK | |
| 05_4 | Flag up before reading, down after writing; wait until their flag is down before comparing | 1974 program (choosing, L2) | OK | |
| 05_5–05_6 | Same order with flags: Green sees Blue's flag up and waits | `C-F6.a_prime.steps` = 15; `C-F6.a_prime.customer2_stopped_by_flag_at_step` = 7 | OK | The picture's "same eight steps in place" can't be literal. Trace a's step 6 (Green walks in on 0) never happens in a′: Green waits at step 7 and only enters at step 15. |
| 05_7 | Without flags: 28 orders, 4 put both in | `C-F3.no_flag.orders` = 28; `orders_both_inside` = 4 | OK | |
| 05_7 | With flags: 322 orders, not one puts both in | `C-F3.flag.orders` = 322; `orders_both_inside` = 0 | OK | Exhaustive, so "not one" holds. |
| 05_8 | The flag isn't a lock; nobody waits to raise their own; many can be up at once | memo §0.4 [V]; no guard on `choosing:=1` in bakery_mc; my G1: a state with all three flags up is reachable (N = 3, atomic and safe) | EDGE (holds) | |
| 05_9 | "Lamport didn't try to close the gap between looking and writing. He made it visible…" | none for the intent | UNSOURCED | The mechanics are sourced (memo §0.3–0.4, R18); the claim about Lamport's intent isn't. Fix: "The flag doesn't close the gap between looking and writing. It makes the gap visible, and the tie-breaker settles equal tickets." |
| 06_1 (screen) | The square becomes a cube (7×7 → 10×10×10) | `oracle.bakery_ops`: 3N+1 steps per customer (7 for N = 2, 10 for N = 3) | OK | |
| 06_2 | Over two billion orders, and not one puts two in the room | `C-F4.flag.orders` = 2,126,816,238; `orders_two_or_more_inside` = 0 | OK | |
| 07_2 | Purple and Green arrive together; both choose ticket one | trace d steps 1–2, 8, 14; `C-F6.d.tickets_taken` = P3:1, P2:1, … | OK | |
| 07_2 | Purple finishes first, finds Green's flag still up, and waits | Purple's flag down at step 9; `C-F6.d.customer3_stopped_by_customer2_flag_at_step` = 12 | OK | |
| 07_2 | Without the flag she'd have read Green's unwritten ticket as 0 | trace d: number[2] = 0 at step 12; Green writes 1 at step 14 | OK | |
| 07_3–07_4 | Teal sees one and one and takes two | trace d steps 17–20 (reads Blue 0, Green 1, Purple 1); P4:2 | OK | |
| 07_4 | The Purple/Green tie goes to Green, the lower number | step 23 (Purple waits on (1,2)); Green enters at step 29 | OK | |
| 07_5 | Blue arrives late with the lowest number and takes three | steps 30–34 (reads 1,1,2); P1:3 | OK | |
| 07_5 | "She waits behind ticket two" | trace d step 49: Blue waits on **Purple's ticket one** (Purple in the room); step 54: Blue waits on Teal's ticket two | MISMATCH | Incomplete against the trace it stands for. Fix: "She waits behind tickets one and two" or "She waits for Purple, then Teal." |
| 07_5 | Green leaves, gets back in line, and takes four | Green exits at step 36; rechooses at steps 55–59 (reads 3,0,2); P2:4 | OK | The voice order isn't chronological: Green leaves (step 36) before Blue's waits (49, 54). Fine if the picture follows the step clock. |
| 07_5 | Who's served first follows the grid exactly | `C-F6.d.service_order` = (1,2)(1,3)(2,4)(3,1)(4,2); `service_order_is_lexicographic` = 1 | OK | The picture's scoreboard 41 → 46 = `C-F6.d.room_entries` = 5. The model has N = 4 (Pink idle), which doesn't change the run. |
| after 07_5 (screen/description) | The owner's TS `(t[j], j) < (t[i], i)` evaluates `j < i`; `tsc --strict` is silent | `C-F13.example.as_written` = 1; `as_written_equals_j_lt_i_on_every_grid_case` = 1; `tsc_strict_diagnostics_on_owner_snippet.recorded` = 0 | OK | The storyboard already holds back the ESLint claim. |
| 08_1 (screen) | The proof assumes in-order steps, finished writes seen by later reads, and a max that reads each slate once | memo §4 (Assumption M) | OK | |
| 08_2 | Fact one: if Blue finished choosing before Green started and still holds that ticket, Green's is bigger | memo §4 Lemma 1; `C-X2.*.violation_found` = 0 (FCFS ghost and Assertion 2; N ≤ 3, atomic and safe) | OK | Needs Assumption M (see 10_4). |
| 08_3 | Fact two: if Blue is in the room, everyone else holding a ticket has a bigger pair | memo §4 Lemma 2 ("in the bakery"); my G2 check | EDGE (holds) | The voice widens Lemma 2 to anyone holding a written ticket, including those whose flag is still up. G2 (N = 3): atomic K = 5 has 0 violations over 27,778 states, 464 of the cases with the holder's flag up. Safe K = 4 has 0 once a customer whose exit write is still in progress is excluded; all 1,070 raw hits are such customers, who have already left the room. |
| 08_5 | Not yet choosing → by fact one, Green's ticket will be bigger | memo §4 Lemma 2 case (a) | OK | |
| 08_6 | Mid-choosing → Green's flag was up, so the check wouldn't pass | memo §4 Lemma 2 (forbidden middle) | OK | A glance at a flag mid-raise or mid-lowering is handled by the chapter-10 lower-third and memo §4's last bullet. |
| 08_7 | Finished → Blue read Green's real ticket and went in only because her pair was smaller | memo §4 Lemma 2 case (b) | OK | |
| 08_8 | Both inside would make each pair smaller than the other's: impossible | memo §4 Theorem | OK | |
| 08_9 | A search agrees over every order, up to four customers, with small tickets | `C-F8.n4_k4_atomic.states` = 3,027,633 (recomputed live in my run); `violation_found` = 0 | OK | Atomic reads; with safe registers only N ≤ 3 (`C-F8.n3_k6_safe`). The card says so. |
| 08_9 | A search covers only small cases; the proof covers all | memo §3.1 | OK | |
| 09_1 | Rewriting 09 as 10 a digit at a time, a glance reads 19 or 00 | `C-X1.rewrite_09_as_10.reads_in_between` = 00,19 | OK | |
| 09_2 | A glance during a write may return any number the slate could hold, even 2496 | 1974 p. 453: "The read may return any arbitrary value!"; publications page: "(assuming that 7456 is a value that could be in the memory location)"; `C-F14.turing_lecture_garbage.value` = 2496 | OK | |
| 09_3 | Twelve years later he named such memory "safe" | `C-F14.interprocess_communication.year` = 1986 (1986 − 1974 = 12); publications page (fetched): "This paper introduced three classes of registers. The weakest … is called safe." | OK | It appeared first as SRC Research Report 8 (1985), and Lamport says he had the results "in the late 70s". "Named", meaning in print, in 1986 is right. |
| 09_3 | A funny name, since it's the least safe kind | same page ("The weakest class"); memo §2.3 | OK | |
| 09_4–09_5 | "Does the proof ever rely on a glance that crosses a write? Never. It relies on just two glances." | memo §4 Lemma 2 and "Why garbage is harmless" | EDGE (fails as worded) | 09_4 lets "every slate **and every flag**" flicker. The proof also relies on Blue's last flag check (r₂ returning "down"), and that check *can* overlap Green's flag write. Memo §4 handles it: mid-raise → case (a), mid-lowering → case (b). "Never" is true only for ticket (slate) reads. The algorithm is fine: safe flags and slates give no violation (`C-F8.n3_k6_safe`, `C-X2.*_safe`). Fix: ask "Does the proof ever trust a *ticket* read that crosses a write?", or add "…and a flag check, which counts as before or after if it catches a flip". |
| 09_5 | Green read Blue's finished ticket and Blue read Green's; nobody was writing either slate | memo §4 Lemma 1/2 (number constant through the bakery) | OK | |
| 09_6 | Garbage of seven makes a customer take eight | `C-F6.e1.overlapping_read_value` = 7; `ticket_taken_after_garbage` = 8; `max_in_room` = 1 | OK | |
| 09_6 | Garbage "just" makes her "wait longer" | memo §1 / §8 pitfall; trace e1 | EDGE (holds for garbage above the true value) | In e1 she'd have waited for Blue anyway (a true read of 0 or 1 gives ticket 1 or 2, which still loses to (1, Blue)), so "longer" means "never sooner". Garbage *below* the true value can let her in sooner, which 10_1 covers. Acceptable as split. |
| 10_1 | A glance catching **Green's** old zero mid-write lets **Blue** straight in | trace e3 (the storyboard's picture): `C-F6.e3.customer2_enters_on_that_read` = 1; replay step 25: **Green** reads Blue's slate mid-write (Blue writing 9) as 0 and enters; **Blue** then waits (step 30) | MISMATCH | The colours are reversed relative to the recorded trace. Fix: mirror e1/e3. I replayed the mirrored schedule on bakery_mc (safe, K = 9): 30 steps, Blue reads Green's slate during Green's 0 → 9 write as 0 and enters at step 25, Green finishes 9 and waits behind (8, Blue), max in room 1, final tickets Blue 8, Green 9. Add it to the oracle as e3′. Alternatively, swap the colours in 10_1–10_2. |
| 10_1 | "Old zero" (the value before the write) | `C-F6.e3.overlapping_read_equals_pre_write_value` = 1 (F3) | OK | |
| 10_2 | Blue had seen Green's flag down → Green started choosing after Blue finished → by fact one Green's ticket is bigger, so Green waits | memo §4 Lemma 2 case (a); mirrored e3 replay (Blue's flag check at step 17, Green's doorway at 21–28, Green waits at 30) | MISMATCH | The logic is correct, but the roles don't match recorded e3 (same fix as 10_1). |
| 10_2 | Nobody writes a ticket with the flag down | 1974 program (the ticket write is inside the doorway) | EDGE (holds) | The exit write of 0 happens with the flag down, but 03_3 says 0 means "not in line", so it isn't a ticket. |
| 10 (screen lower-third) | Today's chips don't tear an aligned word | fact sheet open item 5 (TF-A: "the ARM architecture guarantees single-copy atomicity for aligned accesses") | UNSOURCED (x86 half) | Sourced for Arm only. Add an Intel SDM citation (aligned accesses are atomic) or say "Arm guarantees…". |
| 10_3 | Lamport noticed this only after writing the proof | publications page (open item 4; I fetched it): "I discovered that the bakery algorithm had this property after writing a proof of its correctness" | OK | |
| 10_3 (screen) | Quote card "noticed after writing the proof" | same page | MISMATCH (if set as a quotation) | That isn't Lamport's wording. Use "I discovered that the bakery algorithm had this property after writing a proof of its correctness", or drop the quotation marks. |
| 10_3 | "In his words, the read may return any arbitrary value" | 1974 p. 453, verbatim: "The read may return any arbitrary value!" (open item 3) | OK | |
| 10_3 | His colleague Anatol Holt refused to believe it | publications page (fetched): "Anatol Holt, a former colleague at Massachusetts Computer Associates … He refused to believe it could be true." | OK | The fact sheet has the refusal but not "colleague". Add the clause to the row. |
| 10_4 | Lamport's own caution: find the biggest ticket by reading each slate exactly once | Lamport, POPL 1983 (memo §2.2): "not a valid implementation if the two instances of number[k] represent separate fetches"; `C-F14.nonatomic_operations.year` = 1983 | OK | |
| 10_4 | The obvious loop reads a slate twice, and a flicker between can break it | `C-F6.double_fetch.n3_k3_safe.violation_found` = 1 (33 steps); `n2_k6_safe` / `n2_k9_safe` = 0 | OK | "Can" is right. It needs three or more customers and reads that can return garbage. |
| 11_1 | Finish choosing before I start and you're served first | Lamport 1986 Part II FCFS (memo §2.3); `C-X2.fcfs_and_assertion2.*.violation_found` = 0 (4 configs) | OK | |
| 11_1 | Choosing at the same time, either may go first | my bakery_mc BFS (N = 2, K = 4, atomic): with both flags up at once before anyone entered, both customer 1 and customer 2 occur as first entrant | EDGE (holds) | |
| 11_2 | Nobody waits forever, if everyone in the room leaves and nobody stalls with a raised flag or a ticket | memo §4 (FCFS + deadlock-freedom ⇒ lockout freedom, Lamport 1986 Thm 1); `C-X3.liveness.*.correct` = 0 | EDGE (holds) | Rests on the proof. The model checks deadlock-freedom only (F9). The caveats match Lamport's. |
| 11_2 | Once you've taken your ticket, each other customer gets ahead of you at most once | memo §4 corollary (from the end of the doorway); my G4 ghost check from the moment the ticket write *completes* | EDGE (holds) | G4: worst case 1 bypass (N = 3, K = 5 atomic, 67,827 ghost states; N = 3, K = 4 safe, 225,215). |
| 11_3 | Tickets keep growing as long as somebody is always in line | 1974: "If there is always at least one processor in the bakery, then the value of number[i] can become arbitrarily large"; `C-F6.leapfrog.tickets` = 1..6; my G3 | EDGE (holds for the largest ticket) | G3 (N = 3): the largest ticket held never drops to a nonzero value (atomic K = 5: 0 cases; safe K = 4: 0, ignoring customers still mid-exit-write). A single new ticket *can* be lower than one still held: found, customer 2 writes 1 while customer 3 holds 2. Lamport says "can become arbitrarily large". |
| 11_3 | They reset only when the bakery empties | my G3 | EDGE (holds) | The high mark drops only to zero, that is, when nobody holds a ticket. Customers may still be mid-choosing then. |
| 11_4 | Lamport did the sum at one customer a millisecond | 1974 p. 455, verbatim (open item 3) | OK | |
| 11_5 | Below about 34 billion, two to the thirty-five | `C-F10.ms_per_year` = 31,557,600,000 < `C-F10.two_pow_35` = 34,359,738,368; `ms_per_year_below_two_pow_35` = 1 | OK | |
| 11_5 | Too big for a 32-bit number, which would wrap in about fifty days | `C-F10.u32_wrap_days_at_1_per_ms_1dp` = 49.7 | EDGE (holds for unsigned) | A signed 32-bit number (Java `int`, the `AtomicIntegerArray` the card warns against) wraps in 24.9 days (2³¹ ms, computed). |
| 11_6 | A 64-bit number lasts over five hundred years at a billion tickets a second | `C-F11.u64_years_at_1e9_per_s_1dp` = 584.5; `C-F11.i64_years_at_1e9_per_s_1dp` = 292.3 | EDGE (fails for signed) | Holds only for unsigned (the Rust panel's `AtomicU64`). The same chapter's costs card recommends Java `AtomicLongArray`, which is signed: 292 years. Fix: "lasts centuries", or "an unsigned sixty-four bit number", or change the card. |
| 11_6 | As long as a glance never returns more than was ever written | 1974 p. 455, verbatim (open item 3; lower-third) | OK | |
| 11_7 | "Even alone, a customer reads every other flag and slate, three times over" | `C-F12.other_slate_reads_alone.model.n5` = 12 = 3(N−1); `…equal_3_n_minus_1_for_n_2_to_8` = 1 | MISMATCH | 3(N−1) is three reads *per other customer*: her slate twice (max, L3) and her flag once (L2). Read literally, "every other flag and slate, three times over" means 6(N−1). Fix: "Even alone, a customer makes three reads for every other customer: her slate twice and her flag once." |
| 11_7 | One customer frozen while choosing stalls everyone | memo §8 costs; my G5 | EDGE (holds, eventually) | G5 (N = 3): with customer 1 frozen mid-choosing (14,185 atomic / 36,604 safe start states), each other customer can still enter at most once (if already past her flag check), then everyone stalls. |
| 11 (screen costs card) | Any read/write lock needs ≥ N shared variables (Burns & Lynch 1993); the bakery uses 2N, half unbounded; 3(N−1) reads per entry | memo §2.3 † (sub-agent-verified); C-F12 | OK | |
| 11 (screen costs card) | Use `AtomicLongArray`, never `AtomicIntegerArray` | none | UNSOURCED | True in my understanding (Java), but see 11_6: `long` is signed, so 292 years, not 584. |
| 12_1 | Run as written on your laptop, it can fail | `C-F9.no_fence.violation_found` = 1 (12 steps, x86-TSO); memo §2.3 (Intel SDM §11.2.3.4; ARM/POWER allow SB) | OK | |
| 12_1 | Your laptop breaks one of the proof's promises (a finished write is seen by every later read) | memo §3.6 | OK | |
| 12_2 | Your laptop has several processors, called cores | general knowledge | unchecked | |
| 12_2 | Each core drops its writes in an outbox and keeps going (lower-third: "it's not the cache … it's the queue in front of them") | x86-TSO (memo §2.3 †, Sewell et al. 2010); `bakery_tso.py` | OK | Exact for x86. Arm reorders for more reasons than store buffers (12_6). |
| 12_3–12_4 | Both raise flags and write tickets into outboxes, then each reads the other's slate: no flag, both walk in, real slates still zero | `C-F9.no_fence.memory_choosing_at_violation` = 0,0; `memory_tickets_at_violation` = 0,0; `in_room_at_violation` = 2; 12 steps | OK | The **recorded** BFS trace (`outputs/bakery_tso_out.txt`, the 12 lines after line 1) is a different interleaving: P1 walks in before P2 starts. I replayed the **voiced** order on `TSOBakery(2,4,())` and it is also a 12-step violation with memory at zeros, so animate that one. Wording: "reads the other's slate … No flag" mixes the two words; consider "each checks the other's flag and slate". |
| 12_5 | The fix is a fence: wait until your outbox is delivered before reading others | `bakery_tso` MFENCE semantics; memo §2.3 (MFENCE forbids SB) | OK | |
| 12_5 | On Intel, two fences per entry (after raising, after lowering the flag) are enough | `C-F9.f1_f2.violation_found` = 0 (N = 2 K = 4 B = 4; `n2_k6_b4`; `n3_k3_b3`); `f1_only` / `f2_only` fail (18 / 20 steps); Abdulla et al. TACAS 2012 (memo †) | EDGE (holds within bounds) | "Enough" rests on a bounded model plus the literature, and both fences are necessary. Also: in C, C++ or Rust a fence around plain variables is still a data race. |
| 12_5 | …and on AMD | x86-TSO paper (memo †) covers x86 generally; no AMD citation | UNSOURCED | Add the AMD64 APM Vol. 2 memory-ordering section, or a statement that x86-TSO covers AMD. |
| 12_6 | Arm chips, like the ones in recent Macs | none (the storyboard's Dev note mentions M4 Pro / `ldapr`) | UNSOURCED | True as far as I know. Add a one-line source. |
| 12_6 | Arm reorders even more | memo §2.3 / §8 (ARM/POWER weaker; Maranget et al. †) | OK | |
| 12_6 | So there the fences go in more places | none: no Arm model or fence analysis in the repo (TSO only) | UNSOURCED | Needs an Arm check (e.g., herd7 aarch64 on a bakery litmus) or a citation. Otherwise say "need more care". |
| 12_7 | Sequentially consistent atomics are the easy, correct way everywhere | memo §5 Ch10 / §8 (proposal-level); DRF-SC theory | UNSOURCED | True by the language models (SeqCst-only code has SC executions, which is the proof's model). Not model-checked. Add a row citing C++ [atomics.order] / Rust std docs, or record the c7v0.17 run. |
| 12_7 | Acquire/release aren't enough, because your write must land before your next read | none recorded (Dev's 11-lost-update stress run is unrecorded) | UNSOURCED | True. It is backable from what's here: on x86, acquire loads and release stores compile to plain MOV, so `C-F9.no_fence`'s 12-step failure applies. Add that mapping as a row, or record c7v0.17. |
| 12_7 (screen panel) | The SeqCst Rust lock/unlock as shown | I compiled the panel with rustc 1.100.0-nightly (`--crate-type lib`): compiles | OK | `(nj, j) > (t, i)` is correct because j ≠ i (dev-14). The max also loads its own slate (`C-F12.own_slate_loads_alone.rust_panel.n5` = 1), which is 0 then and harmless. |
| 12_7 (screen card) | C, C++ and Rust plain shared variables are UB and `volatile` doesn't help; Java `AtomicLongArray` works, `volatile long[]` doesn't | memo §8 (C/C++ UB, C/C++ `volatile`) | UNSOURCED (Rust `volatile`, Java parts) | True as far as I know. Cite the JLS / `java.util.concurrent.atomic` docs and the Rust reference. |
| 12 (screen lower-third) | SeqCst on x86 fences every store | none | UNSOURCED | Needs a C++→x86 mapping citation (SC store = `xchg` or `mov` + `mfence`). |
| 12 (screen lower-third) | x86-TSO model checked to 3 customers; matches Abdulla et al. 2012 | `C-F9.f1_f2.n3_k3_b3.violation_found` = 0; memo §2.3 † | OK | |
| 13_1 | In practice people use atomic instructions | memo §8; Turing lecture ("of little if any practical use"); open item 6 (`xadd`) | OK | |
| 13_2 | "Inside many Arm chips" | fact sheet open item 5 + F2 | UNSOURCED ("many") | TF-A is firmware, not silicon. It uses bakery locks only when `HW_ASSISTED_COHERENCY == 0` (`psci_private.h`), and nothing counts platforms. Fix: "In Arm's own reference firmware…" (firmware-design.rst: "On Arm Platforms, bakery locks are used in psci (`psci_locks`) and power controller driver (`arm_lock`)"). |
| 13_2 | …in the code that runs while a core wakes up with its data cache still off | firmware-design.rst L539–542: "immediately after reset and before the data cache is enabled in the warm boot path" | OK | |
| 13_2 | Until it rejoins the others, the usual spinlocks can't be used, so it takes a ticket | same: "not currently possible to use 'exclusive' based spinlocks, therefore BL31 uses locks based on Lamport's Bakery algorithm"; `psci_private.h` | OK | The F2 narrowing is applied. |
| 13_3 | The bigger-than-any-seen ticket, with numbers to break ties, reappears in Lamport's ordering of events across networks | CHM (memo §2.2: "a sequence of numbers that can in fact increase forever … in the 'Time, Clocks' paper… in Paxos"); Time, Clocks 1978 p. 560 (fetched): "To break ties, we use any arbitrary total ordering < of the processes"; `C-F14.time_clocks.year` = 1978 | OK | The tie-break quote isn't in the repo; add it. Keep "reappears": the publications page credits the timestamp idea to Johnson & Thomas. |
| 13_3 | It's in Paxos's ballot numbers too | CHM (memo); Paxos Made Simple (fetched): "a proposal with number greater than any already used" … "Different proposers choose their numbers from disjoint sets of numbers" | OK | "Ballot" is the Part-Time Parliament term; Paxos Made Simple says "proposal number". Not in the repo; add it. |
| 13_3 | Paxos keeps copies of a database agreeing | Paxos Made Simple §3 (fetched): replicated deterministic state machine (banking example) | OK | Not in the repo. |
| 13_4 | Lamport says he didn't invent the bakery algorithm; he discovered it | publications page, verbatim (open item 4; fetched): "I feel that I did not invent the bakery algorithm, I discovered it." | OK | The voice uses indirect speech; the caption is verbatim. |
| 13_4 | It just seemed to come out of thin air; perhaps that's why he's proudest of it | CHM pp. 9–10 (F4): "…it just seemed to come out of thin air to me. There was nothing like it that preceded it, so perhaps that's why I'm proudest of it." | OK | The paraphrase drops "nothing like it preceded it", which is what "so perhaps" follows. Acceptable with attribution. |

Lines with no checkable claim of their own (questions, transitions or the title): 01_3, 02_2, 02_3, 02_8, 03_1, 04_1, 06_1 (voice), 07_1.

## Since the walk (R3, 2026-10-11)

The rows above are the walker's verbatim answer. These are their status now.

| line | claim | source | status | note |
|---|---|---|---|---|
| 12_7 | Sequentially consistent atomics are the easy, correct way everywhere | fact sheet C-S12 (C++ [intro.races], Rust `std::sync::atomic`, JLS §17.4.5); C-M1 (recorded stress runs, SeqCst 0 lost on both machines) | OK | The zeros corroborate; the guarantee is C-S12 plus the proof. |
| 12_7 | Acquire/release aren't enough, because your write must land before your next read | C-S13 (x86 mapping: acquire/release are plain MOVs; `C-F9.no_fence` fails in 12 steps); C-M1 (x86 Release/Acquire lost 1,173,616 of 60,000,000) | OK | Arm saw none in its runs; the 12_7 table captions that zero "none seen; not promised" (R3 dev-01, ruth-01). |
| 13_1 (lower-third) | A ticket lock takes its number with one fetch-and-add; on x86 the SeqCst bakery fences every store | fact sheet open item 6 (Linux commit 314cdbe, "x86: FIFO ticket spinlocks": "a 16-bit xadd"); C-S14 (Cambridge mapping page; SDM: XCHG always asserts LOCK#) | OK | R3 dev-03 (5d56abf). |
| 13_2 (lower-third) | TF-A's PSCI code takes a bakery lock only when HW_ASSISTED_COHERENCY is 0; a bakery lock needs only aligned loads, stores and barriers | C-S17 (psci_private.h, plat_arm.h, bakery_lock_coherent.c at TF-A 5103554) | OK | R3 ruth-04 (135eeb0): `arm_lock` is a bakery lock in BL31 on every platform. |
| 11_6 (caption) | A real huge glance: a 64-bit ticket written in two 32-bit halves (Java's plain long) | C-S22 (JLS §17.7, non-atomic treatment of `long`) | OK | R3 dev-07. |
