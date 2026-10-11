# Fact sheet: Video C, "The Algorithm Lamport Is Proudest Of"

Bead `mx-e-c-7v0.2`. This sheet covers every row C-F1 … C-F14 of the facts
ledger (plan §11.5), the numbers spoken in plan §11.4 that belong to those
rows, and three supplementary rows (C-X1 … C-X3) for narrated claims the
ledger does not list. Every value in the table was copied from the oracle's
actual output by a script, not typed by hand. The claim walk (gate G3,
`docs/claims.md`, 2026-10-11) added 11 keys (C-F6, C-F7, C-F10) and the
cited rows C-S1 … C-S17 below, for claims that rest on a source rather than
on a computation. Review round R2 (2026-10-11, `docs/reviews/bakery_algorithm/R2.md`)
added 52 keys (C-F6, C-F9, C-F10, C-F13 and the edge tests C-X4 … C-X10), the
cited rows C-S18 … C-S29, the edge-test index and the simplification → exact
list below.

Line ids: the Findings, the open items and the plan-value column use the ids
of plan §11.4 (revision 2), which the storyboard has since renumbered (plan
02_3, for example, is storyboard 02_4). Rows added at the claim walk cite
storyboard ids and say so. The video's slates example is read by customer 1
(Blue; R1, M3): the `C-F7.customer4_*` and customer-4 `C-F7.model.*` rows are
the plan's original example, kept for the record.

```
python3 -I explainer_corpus/bakery_algorithm/oracle.py   # from the repo root
```

(`./x bakery_algorithm facts-check` runs it from the crate folder with
`uv run --no-project --with numpy python3 -I -B oracle.py`.)

- **Output:** 236 `KEY<TAB>VALUE` lines on stdout (184 before R2). Provenance
  and 64 cross-checks (53 before R2) go to stderr. The exit status is 1 if any
  cross-check fails.
- **Runs on 2026-10-10** (Apple M4 Pro, CPython 3.14.6): 34.4 s and 34.0 s
  wall in two runs with byte-identical stdout; 168 keys; 53/53 cross-checks
  passed; exit 0. Peak memory is about 1.8 GB, almost
  all of it the live N = 4 sweep. The run writes nothing outside stdout and
  stderr (`sys.dont_write_bytecode` keeps `__pycache__` out of the scripts
  folder).
- **Runs on 2026-10-11** (same machine, the `uv` command above, which runs
  CPython 3.14.0): the claim walker's, 397.7 s, 173 keys; after the G3
  additions, 247.6 s, 184 keys, under a load average between about 20 and
  100. Both: 53/53 cross-checks passed, exit 0, no key printed
  twice. The live N = 4 sweep took 392.7 s and 239.3 s of those. The second
  run's stdout differs from the first only by the 11 added lines: every
  earlier key has the same value.
- **Runs at R2, 2026-10-11** (same command, `nice -n 10`): 167.5 s and 201.3 s,
  each 236 keys, 64/64 cross-checks passed, exit 0, no key printed twice. The
  first 184 lines are byte-identical to that morning's run; the 52 R2 keys
  follow them. The table below was copied from the first run and every value
  equals the second's. The R2 rows themselves take about 4 s.
- **Result against the plan:** 126 MATCH, 0 MISMATCH. 54 rows are n/a because
  the plan (or, for the rows added at the claim walk, the storyboard) states
  no value for them (supporting values). 1 WORDING and 3 NOTE rows are
  explained under Findings. The 52 R2 rows compare with the R2 storyboard:
  32 MATCH, 20 n/a (supporting values), 0 MISMATCH.

## How the oracle gets its numbers

- **Recomputed from scratch.** The interleaving counts (C-F1–C-F5) come from
  my own interleaving engine: explicit path enumeration for two customers, and
  for three customers an "avoidance" DP (all leaves minus the leaves reachable
  without ever putting two in the room). Both share no code with
  `tree_count.py` and are checked against it. The slates example (C-F7), the
  arithmetic (C-F10, C-F11), the code transliterations (C-F12) and the
  comma-operator emulation (C-F13) are also computed here.
- **Computed with the research scripts.** The oracle imports the scripts'
  side-effect-free modules (`bakery_mc.py`, `tree_count.py`, `bakery_tso.py`).
  `scenarios.py`, `leapfrog.py` and `df_trace.py` run on import, so their
  schedules are copied into `oracle.py` and replayed step by step against
  `bakery_mc`'s transition relation. Every step must match exactly one enabled
  transition.
- **Recorded.** Keys ending in `.recorded` are constants pinned from a cited
  file in `docs/research/bakery_algorithm/scripts/outputs/`. The oracle also
  re-reads that file to confirm it still says so.
  - `C-F8.n4_k4_atomic.states.recorded` = 3027633 comes from
    `outputs/sweep_big.txt:3` (that run took 711.7 s).
  - `C-F8.n3_k6_safe.states.recorded` = 269550 comes from
    `outputs/sweep_big.txt:2`.
  - Both sweeps are also **recomputed live** in every run, in about 30 s and
    2 s on an idle machine (239.3 s and 3.8 s in the loaded run of
    2026-10-11). The live counts equal the recordings, pruned counts included
    (204,059 and 12,306). So none of the ledger's model-checker numbers rests
    on a recording alone.
  - `C-F13.tsc_version.recorded` and
    `C-F13.tsc_strict_diagnostics_on_owner_snippet.recorded` are tool outputs
    Python cannot reproduce. I re-ran `tsc --noEmit --strict
    owner_snippet_lock.ts` on 2026-10-10: version 5.9.3, no output, exit 0.
    `node comma_operator_check.js` (node v26.8.1) printed `(t[j], j) < (t[i],
    i) evaluates to true | j < i = true | lexicographic would be false`.
- **Cited.** C-F14 values are pinned from the primary sources quoted below.
  Claims with no number to compute rest on the cited rows C-S1 … C-S17.
- **Checked against recordings.** Every other live value from the reused
  scripts is compared with the matching line of `outputs/` (file:line on
  stderr): `tree_count_out.txt`, `crosscheck_out.txt`, `scenarios_out.txt`,
  `leapfrog_out.txt`, `df_trace_out.txt`, `bakery_tso_out.txt`, `tso3.txt`,
  `sweep_fcfs.txt`, `sweep_live.txt`. All agree.

Conventions: an "order" counts each read, each write and each successful wait
as one step (plan 02 lower-third). Each customer visits once. The max reads the
other customers' slates in index order. "Both inside" means at some moment.
Model-checker results hold only within their bounds: N customers, tickets ≤ K,
store buffers ≤ B.

## Findings (uncomfortable first)

No oracle value contradicts the ledger. The findings below are about wording
and citations in the plan's draft script.

- **F1, wording (02_3).** "Of the fourteen orders, eight end with both in the
  room." Read literally, this is false: no order ends with both inside
  (`C-F2.orders_ending_with_both_inside` = 0). Each of the 14 one-visit orders
  ends with both customers gone. The 8 are orders in which both are inside *at
  some moment*. Suggest "eight put both in the room at once", the phrasing 05_6
  already uses.
- **F2, TF-A narration (11_1, was 10_5): partly supported.** The narration says the
  bakery lock is used "for the moments at boot when caches are off and
  compare-and-swap isn't guaranteed to work". The source supports a narrower
  claim:
  - the lock is used "immediately after reset and before the data cache is
    enabled in the warm boot path", when a core powers up or resumes, and not
    only at first boot;
  - what can't be used is "'exclusive' based spinlocks" (load- and
    store-exclusive), not a compare-and-swap instruction as such;
  - PSCI uses bakery locks only when `HW_ASSISTED_COHERENCY` is 0. On
    hardware-coherent platforms TF-A uses ordinary spinlocks.

  Suggested wording: "…for the moments when a core is waking up with its
  caches off, and the chip's usual atomic instructions aren't guaranteed to
  work." Quotes are under the open items below.
- **F3, trace e3 (08_5): "a garbage zero".** The overlapping read returns 0,
  and 0 is exactly the slate's value before the write began
  (`C-F6.e3.overlapping_read_equals_pre_write_value` = 1). So the same outcome
  can happen with atomic or regular registers too: the read simply lands
  before the write. It still illustrates case (a) of the proof correctly. e1's
  7 is the true garbage value: it overlaps a 0 → 1 write and is neither 0
  nor 1. If the narration needs garbage that only safe registers allow, say so
  for e1 only.
- **F4, CHM page (11_3 Quotes, was 10_7; memo §2.2).** Levin's question ("You've also
  said it was the one you were proudest of") is on page 9 of 40. Lamport's
  answer ("…it just seemed to come out of thin air to me … perhaps that's why
  I'm proudest of it") is on **page 10** of 40. Cite pp. 9–10.
- **F5, C-F12 depends on what is counted.** 3(N−1) counts reads of *other
  customers'* slates, and holds for the model, the plan's Rust panel and
  Lamport's 1974 text alike (`…equal_3_n_minus_1_for_n_2_to_8` = 1). But the
  plan's own Rust panel also loads its own slate once inside the max
  (`C-F12.own_slate_loads_alone.rust_panel.n5` = 1), so it makes 3N−2 loads in
  all. The memo's "about 2(N−1) reads" (§5 Ch9, §8) is stale and conflicts
  with the plan. Keep the card's "3(N−1) slate reads", meaning other customers'
  slates.
- **F6, "considered impossible" (01_2).** In Lamport's sentence, "this" is
  "to implement mutual exclusion without assuming any lower-level mutual
  exclusion". The narration's "Could computers take turns safely with memory
  like that? In 1973, that was considered impossible." is a fair paraphrase:
  memory whose reads can overlap writes means no lower-level mutual exclusion.
  Don't widen it further.
- **F7, Turing-lecture dates.** The lecture is for the 2013 ACM Turing Award.
  It was given at PODC in Paris in July 2014 and published in CACM 58(6),
  June 2015. "Turing lecture 2015" (C-F14) and "2013 Turing Award lecture"
  name the same text. A bare year on screen should say which it is: the award
  (2013), the talk (2014) or the article (2015).
- **F8, 2496 and 2^2496.** The quoted sentence's 2496 is set in full-size
  digits (I checked glyph sizes in turing.pdf). The same paragraph later asks
  about "the value 2^2496" (superscript) and mentions 2^128. Text extraction
  flattens these to "22496" and "2128", so don't copy the quote card from
  extracted text.
- **F9, minor.**
  - 04_2's "both see five, both write six" uses illustrative numbers; the
    replayed equal-ticket traces are c (both take 1) and c′ (both see 1,
    both take 2). (Storyboard 04_2 now says "both see four … both write
    five", and the claim walk added keys for it: `C-F7.model.tie.*`.)
  - In the 1974 PDF, the bakery paper's own date line is "Received September
    1973; revised January 1974". The "Received July 1973" line near the top of
    the scan belongs to the preceding article.
  - The scripts README's runtimes are stale: "big" is now about 35 s, not over
    12 min.
  - C-X3 (liveness) checks deadlock-freedom only (from every trying state
    some entry is reachable). "Nobody waits forever" (09_2) rests on the
    proof's FCFS plus deadlock-freedom, not on the model check.

## Open items from the bead (verify before narration)

**1. Dijkstra and progress graphs: NOT REQUIRED.** Plan §11.4 never connects
Dijkstra to the progress square: 01_6, 01_7, 02_3 and 05_6 show it without
attribution. Dijkstra is named only for the 1965 problem (02_4) and the
"ordinary sign" (02_5). If a later draft names him, this is still open.
UNVERIFIED: a search found secondary sources (Goubault's BRICS report NS-01-7;
the nLab "progress graph" page) saying progress graphs were attributed to
Dijkstra by Coffman, Elphick & Shoshani (1971). The nLab page itself credits
Dijkstra only with the PV language (1968). I read no primary source.

**2. Turing lecture wording: VERIFIED.**
https://lamport.azurewebsites.net/pubs/turing.pdf ("The Computer Science of
Concurrency: The Early Years", PDF dated 28 February 2015, §2.2 "The First
'Real' Solution", PDF page 4; published CACM 58(6):71–76, June 2015).
Verbatim:
- "The algorithm is correct even if reading a number while it is changing from 9 to 10 obtains the value 2496."
- "It was the first algorithm to implement mutual exclusion without assuming any lower-level mutual exclusion. In 1973, this was considered impossible [4, page 88]. Even in 1990, experts still thought it was impossible [21, question 28]."
- [4] is Per Brinch Hansen, *Operating System Principles*, 1973.

The plan's quote ("correct even if reading a number while it is changing from
9 to 10 obtains the value 2496") is an exact substring.

**3. The 1974 paper: VERIFIED.**
https://lamport.azurewebsites.net/pubs/bakery.pdf (CACM 17(8):453–455, August
1974), section "The Algorithm", p. 454 (corrected at R2 from "p. 453": the
section starts on p. 454, checked against the page images on 2026-10-11):
- "The algorithm has the remarkable property that if a read and a write operation to a single memory location occur simultaneously, then only the write operation must be performed correctly. The read may return any arbitrary value!"

p. 455 (the C-F10 bound):
- "if processors enter the doorway at the rate of at most one per msec, then after a year of operation we will have number[i] < 2^35--assuming that a read of number[i] can never obtain a value larger than one which has been written there."
- In the scan, 35 is a superscript and the OCR letter-spacing in "number" is normalized here.

**4. Publications page, the 7456 passage: VERIFIED.**
https://lamport.azurewebsites.net/pubs/pubs.html, entry "A New Solution of
Dijkstra's Concurrent Programming Problem":
- "If the write changes the value from 0 to 1, a concurrent read could obtain the value 7456 (assuming that 7456 is a value that could be in the memory location). The algorithm still works. I didn't try to devise an algorithm with this property. I discovered that the bakery algorithm had this property after writing a proof of its correctness and noticing that the proof did not depend on what value is returned by a read that overlaps a write."
- The same entry also has "I feel that I did not invent the bakery algorithm, I discovered it." (11_3) and, on Anatol Holt, "He refused to believe it could be true." (08_6).

**5. Arm TF-A bakery lock rationale: VERIFIED, with the narrowing in F2.**
The source is TF-A `master` at commit
`510355478850fb9bb4b064554d3dc2416b8b4b96` (2026-10-05); raw files fetched
by SHA matched `master`.
- https://github.com/ARM-software/arm-trusted-firmware/blob/510355478850fb9bb4b064554d3dc2416b8b4b96/lib/locks/bakery/bakery_lock_coherent.c#L19-L35
  (the same comment is in `bakery_lock_normal.c`, lines 21–35): "ARM
  architecture offers a family of exclusive access instructions to efficiently
  implement mutual exclusion with hardware support. However, as well as
  depending on external hardware, the these instructions have defined behavior
  only on certain memory types (cacheable and Normal memory in particular; see
  ARMv8 Architecture Reference Manual section B2.10). Use cases in trusted
  firmware are such that mutual exclusion implementation cannot expect that
  accesses to the lock have the specific type required by the architecture for
  these primitives to function (for example, not all contenders may have
  address translation enabled). This implementation does not use mutual
  exclusion primitives." … "Note that the ARM architecture guarantees
  single-copy atomicity for aligned accesses regardless of status of address
  translation."
- https://github.com/ARM-software/arm-trusted-firmware/blob/510355478850fb9bb4b064554d3dc2416b8b4b96/docs/design/firmware-design.rst#L539-L542:
  "BL31 accesses the state of a CPU or cluster immediately after reset and
  before the data cache is enabled in the warm boot path. It is not currently
  possible to use 'exclusive' based spinlocks, therefore BL31 uses locks based
  on Lamport's Bakery algorithm instead." The same file, line 2375: "On Arm
  Platforms, bakery locks are used in psci (`psci_locks`) and power controller
  driver (`arm_lock`)."
- https://github.com/ARM-software/arm-trusted-firmware/blob/510355478850fb9bb4b064554d3dc2416b8b4b96/lib/psci/psci_private.h#L190-L236:
  "On systems where participant CPUs are cache-coherent, we can use spinlocks
  instead of bakery locks." and, under `HW_ASSISTED_COHERENCY == 0`, "Use
  bakery locks for state coordination as not all PSCI participants are cache
  coherent."

**6. Linux ticket spinlocks, 2008: VERIFIED.**
- Commit `314cdbefd1fd0a7acf3780e9628465b77ea6a836`, "x86: FIFO ticket
  spinlocks", Nick Piggin, 2008-01-30
  (https://github.com/torvalds/linux/commit/314cdbefd1fd0a7acf3780e9628465b77ea6a836):
  "Introduce ticket lock spinlocks for x86 which are FIFO." Its code comment
  in `include/asm-x86/spinlock.h`: "This uses a 16-bit xadd to increment the
  tail and also load the position of the head". `xadd` is a fetch-and-add, as
  03 "For programmers" says.
- The GitHub compare API shows the commit is in `v2.6.25` (tagged 2008-04-17)
  and not in `v2.6.24`.
- LWN, Jonathan Corbet, "Ticket spinlocks", February 6, 2008
  (https://lwn.net/Articles/267968/): "a patch merged for 2.6.25 shows that
  there is always more which can be done." and "Nick's response was a new
  spinlock implementation which he calls 'ticket spinlocks.'"
- The patch was first posted on LKML on 1 Nov 2007
  (https://lkml.iu.edu/hypermail/linux/kernel/0711.0/0125.html). "2008" is
  right for the merge and the release.
- Queued spinlocks: plan §11 does not mention them, so they were not checked.

**Also verified (not bead items, used in 11_3 and C-F14):**
- CHM oral history (Roy Levin, recorded August 12, 2016),
  https://archive.computerhistory.org/resources/access/text/2017/07/102717182-05-01-acc.pdf,
  page 10 of 40: "But the bakery algorithm, it just seemed to come out of thin
  air to me. There was nothing like it that preceded it, so perhaps that's why
  I'm proudest of it." (See F4 for the page.)
- Lamport, "The Mutual Exclusion Problem Part II" (JACM 33(2), 1986),
  https://lamport.azurewebsites.net/pubs/mutual2.pdf: "This requirement was
  first stated and satisfied by Knuth in [6]." Its [6] is Knuth, CACM 9(5):321,
  May 1966.
- The C-F14 citations for 1978, 1983, 1986 and 2015 are as printed on
  Lamport's publications page. The 1965 and 1966 citations are as in
  references 4 and 1 of the 1974 paper.

## Cited rows from the claim walk (C-S1 … C-S17)

Storyboard claims that rest on a source, not on a computation. Line ids are
the storyboard's. Every row was verified on 2026-10-11 during the claim walk
(gate G3): "walker" means the claim walker read the source (`docs/claims.md`);
"re-read" means it was downloaded again and read for the row the same day.
Quotes are verbatim; "…" marks a cut. Third-party documents are not committed;
each location is enough to find the passage.

**C-S1. Dijkstra's conditions, 1965 (02_1, 02_5, 02_6, 02_7; R9).**
E. W. Dijkstra, "Solution of a Problem in Concurrent Programming Control",
CACM 8(9):569, September 1965 (one page; doi:10.1145/365559.365617).
- "at any moment only one of these N cyclic processes is in its critical section. In order to effectuate this mutual exclusion of critical-section execution the computers can communicate with each other via a common store. Writing a word into or nondestructively reading a word from this store are undividable operations"
- "(a) The solution must be symmetrical between the N computers; as a result we are not allowed to introduce a static priority."
- "(b) Nothing may be assumed about the relative speeds of the N computers; we may not even assume their speeds to be constant in time."
- "(c) If any of the computers is stopped well outside its critical section, this is not allowed to lead to potential blocking of the others."
- "(d) If more than one computer is about to enter its critical section, it must be impossible to devise for them such finite speeds, that the decision to determine which one of them will enter its critical section first is postponed until eternity. In other words, constructions in which "After you"-"After you"-blocking is still possible, although improbable, are not to be regarded as valid solutions."
- From his proof: "when none of the computers is in its critical section, of the computers looping … at least one—and therefore exactly one—will be allowed to enter its critical section in due time."
- Lamport, "The Mutual Exclusion Problem Part II" (JACM 33(2), 1986; mutual2.pdf p. 6): "These two properties, mutual exclusion and deadlock freedom, were the requirements for a mutual exclusion solution originally stated by Dijkstra in [2]."
- R9 answered: rule (d) already required that *somebody* gets in, so Knuth's 1966 "everyone" (`C-F14.knuth_eventual_entry`) is more, as 02_6 says. 02_5 voices mutual exclusion, (c) and (b); (a) and (d) are not voiced.
- Verified: walker read a public scan of the page; re-read (a byte-identical copy) against the page image; mutual2.pdf re-read.

**C-S2. "safe", named in 1986 (09_3).** Lamport, "On Interprocess
Communication", Distributed Computing 1(2):77–101, 1986
(`C-F14.interprocess_communication`).
- Publications page (https://lamport.azurewebsites.net/pubs/pubs.html), that entry: "This paper introduced three classes of registers. The weakest class with which arbitrary synchronization is possible is called safe. The next strongest is called regular and the strongest, generally assumed by algorithm writers, is called atomic." The same entry: "I had obtained all the results presented here in the late 70s".
- interprocess.pdf (the SRC Research Report 8 text, dated December 25, 1985), p. 19: "The weakest possibility is a safe register, in which it is assumed only that a read not concurrent with any write obtains the correct value—that is, the most recently written one. No assumption is made about the value obtained by a read that overlaps a write, except that it must obtain one of the possible values of the register."
- "Twelve years later he named" (1974 → 1986) means named in print; the report is dated December 1985.
- Verified: walker read the page; re-read; interprocess.pdf read for this row.

**C-S3. Anatol Holt (10_3).** Publications page, entry "A New Solution of
Dijkstra's Concurrent Programming Problem": "Perhaps the person who realized
it better than anyone is Anatol Holt, a former colleague at Massachusetts
Computer Associates. When I showed him the algorithm and its proof and pointed
out its amazing property, he was shocked. He refused to believe it could be
true." The page's CV lists "Massachusetts Computer Associates (1970-1977)".
Verified: walker; re-read.

**C-S4. The 10_3 quote card.** Same entry: "I didn't try to devise an
algorithm with this property. I discovered that the bakery algorithm had this
property after writing a proof of its correctness and noticing that the proof
did not depend on what value is returned by a read that overlaps a write." The
card quotes the second sentence up to "correctness", word for word; the voice's
"noticed this only after writing the proof" paraphrases it. Verified: walker;
re-read.

**C-S5. Time, Clocks: a bigger clock, and ties broken by process (13_3).**
time-clocks.pdf (CACM 21(7):558–565, July 1978).
- p. 560, rule IR2: "(b) Upon receiving a message m, process Pj sets Cj greater than or equal to its present value and greater than Tm."
- pp. 560–561: "We simply order the events by the times at which they occur. To break ties, we use any arbitrary total ordering < of the processes." p. 561: "a ⇒ b if and only if either (i) Ci(a) < Cj(b) or (ii) Ci(a) = Cj(b) and Pi < Pj."
- The tie-break sentence itself is on p. 561 (claims.md says p. 560, where its paragraph begins).
- Verified: walker; re-read against the page images (subscripts as printed).

**C-S6. Paxos (13_3).**
- Paxos Made Simple (ACM SIGACT News 32(4), December 2001, 51–58; paxos-simple.pdf, dated 01 Nov 2001), p. 7: "if it uses a proposal with number greater than any already used, then it will succeed in issuing a proposal that is accepted." p. 8: "Different proposers choose their numbers from disjoint sets of numbers, so two different proposers never issue a proposal with the same number."
- p. 8, §3: "The server can be described as a deterministic state machine that performs client commands in some sequence." … "We therefore instead use a collection of servers, each one independently implementing the state machine."
- "Ballot numbers": Paxos Made Simple says "proposal number". The Part-Time Parliament (ACM TOCS 16(2):133–169, May 1998; author's PDF lamport-paxos.pdf, p. 5, §2.1): "Ballot numbers were chosen from an unbounded ordered set of numbers."
- Verified: walker read Paxos Made Simple; re-read; The Part-Time Parliament read for this row.

**C-S7. 05_9 describes the algorithm.** "The flag doesn't close the gap
between looking and writing. It makes the gap visible, and the tie-breaker
settles equal tickets." Its subject is the flag; it says nothing about
Lamport's intent (the walker's UNSOURCED item was the earlier "Lamport didn't
try to close the gap"). Each clause has keys:
- the gap stays open: with flags, two customers can still take the same ticket (`C-F6.c.tickets` = 1,1; `C-F6.c_three_customers.tickets` = 2,2,1; `C-F7.model.tie.slates_after` = 4,3,5,5,2);
- the gap is visible: a customer who meets a raised flag waits (`C-F6.a_prime.customer2_stopped_by_flag_at_step` = 7; `C-F6.d.customer3_stopped_by_customer2_flag_at_step` = 12);
- equal tickets are settled by the customer number (`C-F6.d.service_order` = (1,2)(1,3)(2,4)(3,1)(4,2); `C-F7.model.tie.service_order` = 5,2,1,3,4).
- Verified: storyboard text and keys read for this row.

**C-S8. Chapter 10 lower-third, "today's chips don't tear an aligned word".**
Arm: open item 5 (TF-A: "the ARM architecture guarantees single-copy
atomicity for aligned accesses regardless of status of address translation").
x86:
- Intel SDM Vol. 3A (order number 253668-093US, September 2026), §11.1.1 "Guaranteed Atomic Operations", p. 11-2: "The Intel486 processor (and newer processors since) guarantees that the following basic memory operations will always be carried out atomically: … Reading or writing a doubleword aligned on a 32-bit boundary." and "The Pentium processor (and newer processors since) guarantees that the following additional memory operations will always be carried out atomically: Reading or writing a quadword aligned on a 64-bit boundary."
- AMD64 APM Vol. 2 (publication 24593, revision 3.45, July 2026), §7.3.2 "Access Atomicity", p. 196: "Cacheable, naturally-aligned single loads or stores of up to a quadword are atomic on any processor model".
- So the lower-third holds for Arm and x86 (Intel and AMD), for aligned accesses up to 64 bits; no other architecture was checked.
- Verified: read for this row.

**C-S9. Chapter 01 lower-third, `add [x], 1` and the lock prefix.**
- SDM Vol. 3A §11.1, p. 11-2: "The processor also supports bus locking for performing selected memory operations (such as a read-modify-write operation in a shared area of memory) that typically need to be handled atomically, but are not automatically handled this way."
- §11.1.2.2, p. 11-4: "To explicitly force the LOCK semantics, software can use the LOCK prefix with the following instructions when they are used to modify a memory location." … "The following two-operand arithmetic and logical instructions: ADD, ADC, SUB, SBB, AND, OR, and XOR."
- SDM Vol. 2A (253666-093US), "LOCK—Assert LOCK# Signal Prefix", p. 3-565: "Causes the processor’s LOCK# signal to be asserted during execution of the accompanying instruction (turns the instruction into an atomic instruction)."
- Maranget, Sarkar and Sewell, "A Tutorial Introduction to the ARM and POWER Relaxed Memory Models" (2012), p. 5: "many x86 instructions involve multiple memory accesses, e.g. an x86 increment INC. By default, these are not guaranteed atomic (so two parallel increments of an initially 0 location might result in it holding 1)"; p. 8: "On x86 INC is not instruction-atomic whereas LOCK;INC is."
- Verified: read for this row.

**C-S10. Outboxes and fences on Intel and AMD (12_2–12_5).**
- SDM Vol. 3A §11.2.2, p. 11-7: "Reads may be reordered with older writes to different locations but not with older writes to the same location." and "Reads cannot pass earlier LFENCE and MFENCE instructions." §11.2.3.4, p. 11-11, Example 11-3 (each processor stores to one location, then loads the other): "r1 = 0 and r2 = 0 is allowed".
- AMD64 APM Vol. 2 §7.2 "Multiprocessor Memory Access Ordering", p. 191: "stores can be delayed arbitrarily by store buffering while the processor continues operation." "Non-overlapping Loads may pass stores." "Where sequential consistency is needed (for example in Dekker’s algorithm for mutual exclusion), an MFENCE instruction should be used between the store and the subsequent load, or a locked access, such as XCHG, should be used for the store."
- So AMD documents the same store-buffer relaxation as Intel, which is the one the x86-TSO model (C-F9) has. The two fence points themselves rest on C-F9 (`f1_f2` holds, `f1_only` and `f2_only` fail), within its bounds.
- Verified: read for this row.

**C-S11. Arm reorders even more (12_6).**
- Apple, "Addressing architectural differences in your macOS code" (developer.apple.com, Apple silicon documentation), section "Synchronize Access to Shared Data in Memory": "A strong memory-ordering model, like the one in Intel-based Mac computers, adds implicit memory barriers to prevent the processor from reordering load and store instructions in a way that might introduce race conditions. A weak memory ordering model, like the one in Apple silicon, gives the processor more flexibility to reorder memory instructions and improve performance, but doesn’t add implicit memory barriers. To ensure the correctness of your code on both platforms, add explicit synchronization primitives to your code."
- That Apple silicon runs Arm code: Apple, "Building a universal macOS binary": "don’t assume that code intended for the `arm64` architecture runs only on an iOS device. That code also runs in your macOS app on an Apple silicon."
- Maranget, Sarkar and Sewell (2012), p. 4: "In broad terms x86 and SPARC are similar, with relatively strong models based on the Total Store Ordering (TSO) model … ARM and POWER are much weaker than TSO (though broadly similar to each other)". p. 9, on message passing: "On x86-TSO or SPARC TSO that final outcome of r1=1 ∧ r2=0 is also forbidden … But on ARM and POWER, this final outcome is allowed in the architecture, and it is commonly observable on current processor implementations."
- ARMv8 as revised: Pulte, Flur, Deacon, French, Sarkar and Sewell, "Simplifying ARM Concurrency" (PACMPL 2, POPL 2018, article 19), p. 19:1: "ARM has a relaxed memory model"; p. 19:7: "The remaining relaxed-memory effects (which are still subtle) are all due to thread-local out-of-order and speculative execution and thread-local buffering."
- "Need more care": Apple's last sentence above, and C-S13's mapping page, which puts ordering instructions on Arm's sequentially consistent loads ("Load Seq Cst: LDAR" for AArch64) where x86 uses a plain load ("Load Seq_Cst: MOV (from memory)"). The repository has no Arm model, so nothing bakery-specific is checked for Arm.
- Verified: read for this row.

**C-S12. Sequentially consistent atomics are correct everywhere (12_7).**
- C++ working draft (https://eel.is/c++draft/intro.races), [intro.races], note: "It can be shown that programs that correctly use mutexes and memory_order::seq_cst operations to prevent all data races and use no other synchronization operations behave as if the operations executed by their constituent threads were simply interleaved, with each value computation of an object being taken from the last side effect on that object in that interleaving. This is normally referred to as “sequential consistency”."
- Rust, `std::sync::atomic` (docs for 1.99.0): "Rust atomics currently follow the same rules as C++20 atomics, specifically the rules from the intro.races section, without the “consume” memory ordering." `Ordering::SeqCst`: "Corresponds to memory_order_seq_cst in C++20."
- Java, JLS SE 21 §17.4.5: "A program is correctly synchronized if and only if all sequentially consistent executions are free of data races. If a program is correctly synchronized, then all executions of the program will appear to be sequentially consistent (§17.4.3)."
- So the 12_7 panel, which shares only SeqCst atomics, runs as an interleaving of its steps: the model the proof assumes (08_1). The panel is not model-checked as code here, and the c7v0.17 stress run is still to be recorded (storyboard 12, "Honest data").
- Verified: read for this row.

**C-S13. Acquire and release are not enough here (12_7).**
- Cambridge, "C/C++11 mappings to processors" (https://www.cl.cam.ac.uk/~pes20/cpp/cpp0xmappings.html), x86: "Load Acquire: MOV (from memory)" and "Store Release: MOV (into memory)". The page describes itself as "collected for discussion, not as a definitive source".
- So on x86 a release/acquire bakery is the unfenced program of the x86-TSO model, which fails in 12 steps (`C-F9.no_fence.violation_found` = 1, `C-F9.no_fence.shortest_violation_steps` = 12): a release store can still sit in its outbox when the next acquire load reads another slate.
- C++ [atomics.order], note: "memory_order::seq_cst ensures sequential consistency only for a program that is free of data races and uses exclusively memory_order::seq_cst atomic operations. Any use of weaker ordering will invalidate this guarantee unless extreme care is used."
- Not checked: what rustc emits (nothing was compiled for this row).
- Verified: read for this row.

**C-S14. Chapter 12 lower-third, "on x86 it fences every store".**
- Cambridge mapping page, x86: "Store Seq Cst: (LOCK) XCHG // alternative: MOV (into memory),MFENCE" and "The parenthesised (LOCK) reflects the fact that the XCHG instruction on x86 has an implicit LOCK prefix." It also lists a mapping that fences sequentially consistent loads instead, and says "we strongly recommend to use the write-fencing mapping only."
- SDM Vol. 3A §11.2.2, p. 11-7: "Reads or writes cannot be reordered with I/O instructions, locked instructions, or serializing instructions." SDM Vol. 2A, LOCK, p. 3-565: "The XCHG instruction always asserts the LOCK# signal regardless of the presence or absence of the LOCK prefix."
- So under the recommended mapping every SeqCst store is a locked XCHG, or a MOV and an MFENCE: a full fence either way. Not checked: rustc's actual output.
- R2 (dev-09): Dev reports, from his R1 check with rustc 1.100.0-nightly, that the 12_7 panel's lock plus unlock compiles to 4 `xchg` and no `mfence` on x86-64, and to `stlr`/`ldar` with no `dmb` on aarch64, and that Acquire loads compile to `ldapr` on his Mac. That is a reviewer's report, not re-run here (nothing compiles on this machine); it is recorded with its toolchain under bead mx-e-c-7v0.17 before anything on screen says what the compiler emits.
- Verified: read for this row.

**C-S15. 12_7 card: plain shared variables and `volatile`.**
- C++ [intro.races]: "The execution of a program contains a data race if it contains two potentially concurrent conflicting actions, at least one of which is not atomic, and neither happens before the other, except for the special case for signal handlers described below. Any such data race results in undefined behavior". A `volatile` object is not an atomic one; for C, see memo §8.
- Rust, `std::sync::atomic`: "The most important aspect of this model is that data races are undefined behavior. A data race is defined as conflicting non-synchronized accesses where at least one of the accesses is non-atomic." `std::ptr::read_volatile`: "In particular, just like in C, whether an operation is volatile has no bearing whatsoever on questions involving concurrent accesses from multiple threads. Volatile accesses behave exactly like non-atomic accesses in that regard."
- Java, `java.util.concurrent.atomic` package summary (Java 8): "These classes are also notable in providing volatile access semantics for their array elements, which is not supported for ordinary arrays." (Java 21 keeps the first half.) So `AtomicLongArray` works and a `volatile long[]` (a volatile reference to plain elements) doesn't; with every shared access volatile, C-S12's JLS guarantee applies.
- Verified: read for this row.

**C-S16. Signed tickets in Java (11_5, 11_6 picture, 11 costs card).**
- JLS SE 21 §4.2.1: "The integral types are byte, short, int, and long, whose values are 8-bit, 16-bit, 32-bit and 64-bit signed two's-complement integers, respectively". `java.util.concurrent.atomic` (Java 21): "AtomicIntegerArray: An int array in which elements may be updated atomically." and "AtomicLongArray: A long array in which elements may be updated atomically."
- So an `AtomicIntegerArray` ticket overflows after 2^31 entries, 24.9 days at one a millisecond (`C-F10.i32_wrap_days_at_1_per_ms_1dp`), and an `AtomicLongArray` ticket after 2^63, 292.3 years at 10^9 a second (`C-F11.i64_years_at_1e9_per_s_1dp`): "centuries", as 11_6 says. The Rust panel's `AtomicU64` gives 584.5 (`C-F11.u64_years_at_1e9_per_s_1dp`).
- Verified: read for this row; the numbers are oracle keys.

**C-S17. Arm's reference firmware (13_2).** TF-A at commit
`510355478850fb9bb4b064554d3dc2416b8b4b96` (2026-10-05; still `master` on
2026-10-11).
- readme.rst: "Trusted Firmware-A (TF-A) is a reference implementation of secure world software for Arm A-Profile architectures (Armv8-A and Armv7-A), including an Exception Level 3 (EL3) Secure Monitor." and "Copyright (c) 2013-2025, Arm Limited and Contributors." The repository is github.com/ARM-software/arm-trusted-firmware; its documentation is published through trustedfirmware.org.
- docs/design/firmware-design.rst L539–542 (open item 5): "BL31 accesses the state of a CPU or cluster immediately after reset and before the data cache is enabled in the warm boot path. It is not currently possible to use 'exclusive' based spinlocks, therefore BL31 uses locks based on Lamport's Bakery algorithm instead."
- lib/psci/psci_private.h L190–236: "On systems where participant CPUs are cache-coherent, we can use spinlocks instead of bakery locks." and, under `HW_ASSISTED_COHERENCY == 0`, "Use bakery locks for state coordination as not all PSCI participants are cache coherent."
- docs/getting_started/build-options.rst L344–352 (`HW_ASSISTED_COHERENCY`): "On most Arm systems to-date, platform-specific software operations are required for CPUs to enter and exit coherency." … "This option defaults to 0".
- So "Arm's own reference firmware" holds (Arm's copyright on a reference implementation for Arm cores); the bakery lock runs only where `HW_ASSISTED_COHERENCY` is 0, as 13_2's "Precisely" lower-third says; nothing here counts platforms, and 13_2 no longer says "many".
- R2 (M8): 13_2 now says "Code that Arm itself publishes uses it while a core wakes up, before its fast memory is switched on. Until then, the magic signs aren't guaranteed to work". "Fast memory … switched on" is "the data cache is enabled" above; "the magic signs aren't guaranteed to work" is F2's suggested wording ("exclusive" instructions have defined behaviour only on cacheable Normal memory).
- Verified: walker (firmware-design.rst, psci_private.h); re-read; readme.rst and build-options.rst read for this row.

## Cited rows from R2 (C-S18 … C-S29)

Added at review round R2 for claims and notes the R2 reviews asked to source.
Every quote below was read on 2026-10-11 by the R2 fixer from a fresh download
(kept outside the repository), as text or, where the PDF's text layer is OCR,
as page images. "Verified 2026-10-11 (R2)" means that read.

**C-S18. The 1974 single-bit argument and Lamport's correction (09, 10;
description "Precisely" 09; R2 R16a).**
- Publications page, entry "A New Solution of Dijkstra's Concurrent Programming Problem": "The paper contains one small but significant error. In a footnote, it claims that we can consider reads and writes of a single bit to be atomic. It argues that a read overlapping a write must get one of the two possible values; if it gets the old value, we can consider the read to have preceded the write, otherwise to have followed it. It was only later, with the work eventually described in [70], that I realized the fallacy in this reasoning."
- In the CACM text the argument sits in the proof of Assertion 2, not in a footnote (bakery.pdf p. 454): "Since choosing [k] has essentially just two values—zero and nonzero—we can assume that from processor i's point of view, reading or writing it is done instantaneously, and a simultaneous read and write does not occur." The paper's only footnote (p. 455) reads "We have recently found such an algorithm, but it is quite complicated."
- So the flickering-flag case rests on memo §4's case split (corrected at R2) and on the model checks (`C-F8.n3_k6_safe.violation_found` = 0; `C-X2.*_safe`; `C-X4.*.n3_k4_safe.violations` = 0), not on the 1974 argument.
- Verified 2026-10-11 (R2): the page; bakery.pdf pp. 454–455 as page images.

**C-S19. The 1974 paper on failures, the doorway and the bakery, and
Assertion 3 (11_2; 11 costs card; dev-09).** bakery.pdf:
- p. 454: "We allow processor i to fail at any time, and then to be restarted in its noncritical section (with choosing [i] = number [i] = 0). However, if a processor keeps failing and restarting, then it can deadlock the system."
- p. 454: "A processor may fail at any time. We assume that when it fails, it immediately goes to its noncritical section and halts. There may then be a period when reading from its memory gives arbitrary values. Eventually, any read from its memory must give a value of zero."
- p. 454: "Processor i is said to be in the doorway while choosing [i] = 1. It is said to be in the bakery from the time it resets choosing (i) to zero until it either fails or leaves its critical section."
- p. 454, the pair order (04_4, description 04_4–04_6): "The relation "less than" on ordered pairs of integers is defined by (a,b) < (c,d) if a < c, or if a = c and b < d."
- p. 455 (11_3): "If there is always at least one processor in the bakery, then the value of number [i] can become arbitrarily large."
- p. 455, reference 1 (the title in the description's sources): "Knuth, D.E. Additional comments on a problem in concurrent programming control. Comm. Acm 9, 5 (May 1966), 321-322."
- p. 455, Assertion 3: "Assume that only a bounded number of processor failures may occur. If no processor is in its critical section and there is a processor in the bakery which does not fail, then some processor must eventually enter its critical section." From its proof: "assume that processor i has the minimum value of (number [i], i) among all processors in the bakery. Then processor i must eventually complete the for loop and enter its critical section." And: "Assertion 3 implies that the system can only be deadlocked by a processor halting in its critical section, or by an unbounded sequence of processor failures and re-entries."
- So 11_2's conditions and "whoever holds the smallest pair gets through" are Lamport's (with `C-X3` in the model), and the costs card's "a dead customer's slate stays set: resetting it is your job" is the code-side reading of his assumption that a failed processor's memory eventually reads zero.
- Verified 2026-10-11 (R2), page images.

**C-S20. Lamport 1983: read each slate once (10_4; 08_1 lower-third;
dev-09).** "Reasoning About Nonatomic Operations", POPL 1983
(`C-F14.nonatomic_operations.citation`; nonatomic.pdf), §5 "Discussion", p. 36:
- "Our formal approach led us to discover a previously unrecognized assumption required for the correctness of the bakery algorithm"
- "For n > 2 we need the following additional assumption about this statement. If number[j] does not change during execution of the statement, then number[i] is chosen to be greater than number[j] – regardless of the concurrent activity of other processes. Thus, the following is not a valid implementation if the two instances of number[k] represent separate fetches." (the loop `temp[i] := 0; for k := 1 to n do if number[k] > temp[i] then temp[i] := number[k] od; number[i] := 1 + temp[i]` follows).
- So 10_4's caution is Lamport's own. The failure needs three or more customers (`C-F6.double_fetch.n2_k6_safe.violation_found` = 0, `…n3_k3_safe` = 1) and reads that can return garbage (`C-F6.double_fetch.n3_k4_atomic.violation_found` and `…n3_k5_atomic.violation_found` = 0, R2).
- Verified 2026-10-11 (R2), as the page image (the PDF's text layer is scrambled OCR).

**C-S21. Abdulla et al.: two fences per process under TSO (12
lower-third; R2 R16c, dev-09).** P. A. Abdulla, M. F. Atig, Y.-F. Chen,
C. Leonardsson and A. Rezine, "Counter-Example Guided Fence Insertion under
TSO" (the authors' PDF, tacas12.pdf at user.it.uu.se/~parosha; TACAS 2012 per
memo §2.3, the venue not re-checked):
- Table 1, row "4. Lamport Bakery [20]": size "2/22/4/32", fences necessary in the smallest set "2 per process", number of minimal fence sets "4". "The placement constraint only allows fences immediately after write operations."
- §7: "We bounded possible values for the number variables by 2, and checked whether mutual exclusion is respected under TSO. Our approach automatically infers four minimal sets of fences, and proves the resulting programs respect mutual exclusion under TSO. These sets correspond to placing two fences per process, one at line 3 and the other at either line 10 or line 12." In their Fig. 4, line 3 follows `store c[i]=1`, line 10 follows `store n[i]=n[i]` and line 12 follows `store c[i]=0` ("We analyzed an instance with two processes").
- So 12_5's two fences (after raising the flag, after lowering it) are one of their minimal sets, and the lower-third's "matches" holds for two customers with tickets ≤ 2; C-F9 covers N ≤ 3 within its own bounds.
- Verified 2026-10-11 (R2).

**C-S22. Java's plain `long` may be written in two halves (description,
chapter 11; dev-13).** JLS SE 21 §17.7 "Non-Atomic Treatment of double and
long": "For the purposes of the Java programming language memory model, a
single write to a non-volatile long or double value is treated as two
separate writes: one to each 32-bit half. This can result in a situation
where a thread sees the first 32 bits of a 64-bit value from one write, and
the second 32 bits from another write. Writes and reads of volatile long and
double values are always atomic."
- So in Java a glance at a plain `long` slate can return a value that was never written, larger than any that was, which is what Lamport's condition at 11_6 rules out. `AtomicLongArray` elements have volatile access semantics (C-S15).
- Verified 2026-10-11 (R2), docs.oracle.com/javase/specs/jls/se21/html/jls-17.html.

**C-S23. C: a data race is undefined behaviour (12_7 card; dev-09).**
N1570 (the C11 committee draft of April 12, 2011, the last public draft),
§5.1.2.4 ¶25: "The execution of a program contains a data race if it contains
two conflicting actions in different threads, at least one of which is not
atomic, and neither happens before the other. Any such data race results in
undefined behavior."
- A `volatile` object is not an atomic one, so the card's C half holds as C-S15 argues for C++ and Rust.
- Verified 2026-10-11 (R2), open-std.org/jtc1/sc22/wg14/www/docs/n1570.pdf.

**C-S24. First-come-first-served, deadlock freedom and lockout freedom
(11_1–11_2; R2 R7).** Lamport, "The Mutual Exclusion Problem Part II" (JACM
33(2), 1986; mutual2.pdf), pp. 6–7:
- "We can require that if process i finishes executing its doorway statement before process j begins executing its doorway statement, then i must execute its critical section before j does."
- "The FCFS property states that processes will not execute their critical sections “out of turn”. However, it does not imply that any process ever actually executes its critical section. In particular, FCFS does not imply deadlock freedom. However, FCFS and deadlock freedom imply lockout freedom, as we now show. Theorem 1 FCFS and deadlock freedom imply lockout freedom."
- So 11_2 rests on both halves: whoever holds the smallest pair gets through (deadlock freedom: C-S19's Assertion 3, `C-X3`), and nobody gets ahead of you more than once each (`C-X5`).
- Verified 2026-10-11 (R2).

**C-S25. Arm: another core can see a later write before an earlier one
(12_6; dev-02, dev-11).** Maranget, Sarkar and Sewell, "A Tutorial
Introduction to the ARM and POWER Relaxed Memory Models" (October 10, 2012,
revision 120; the same tutorial as C-S9 and C-S11):
- p. 10: "To explain the observed behaviour for MP on ARM and POWER, there are three distinct possibilities: the writes on Thread 0 are to distinct addresses and so can be committed out of order on Thread 0; and/or they might also be propagated to Thread 1 in either order; and/or the reads on Thread 1 (likewise to distinct addresses) can be satisfied out of order, binding the values they read in the opposite order to program order".
- p. 12, on MP with barriers (forbidden): "Microarchitecturally, and in our models, the sync or DMB on Thread 0 keeps the two writes in order, both locally and in the order they propagate to other threads in the system."
- p. 21, on store buffering: "Adding a DMB or sync on both threads suffices to rule out the unintended outcome".
- In bakery terms MP is Blue writing her ticket and then lowering her flag, and Green checking Blue's flag and then her ticket. With only 12_5's two fences, no barrier separates Blue's two writes or Green's two reads, so Green can see the flag down and the ticket still zero: chapter 05's unwritten-ticket case, as 12_6 says. That is a reading of the architecture's MP result, not a run on Arm. The repository has no Arm model, and the voice gives no fence count for Arm. `C-F9.ooo_outbox.*` (R2) shows the consequence in a model where only the writes may be reordered.
- Verified 2026-10-11 (R2).

**C-S26. Turing lecture §2.2 (13_1 card; description; R2 D4, D5).**
turing.pdf, §2.2 "The First 'Real' Solution", printed pp. 3–4: "Although of
little if any practical use, the bakery algorithm [11] has become a popular
example of a mutual exclusion algorithm. It is based on a protocol sometimes
used in retail shops: customers take successively numbered tickets from a
machine, and the lowest-numbered waiting customer is served next. A literal
implementation of this approach would require a ticket-machine process that
never halts, violating Dijkstra’s requirements. Instead, an entering process
computes its own ticket number by reading the numbers of all other
synchronizing processes and choosing a number greater than any that it sees."
Verified 2026-10-11 (R2).

**C-S27. CHM oral history, pp. 9–10 (13_4; description; R2 D3).** Roy Levin
with Leslie Lamport, 2016 (the F4 source):
- p. 9, Levin: "I’ve read in one of your perspectives on it that you thought it was the one algorithm that you discovered rather than invented. I’m not sure I understand the distinction." Lamport: "I’m not sure I understand the distinction either."
- p. 9, Lamport, on the counter: "a sequence of numbers that can in fact increase forever, that you will see elsewhere in things I’ve done — in the “Time, Clocks” paper, for example, the use of timestamps or that kind of counter, and in the whole state machine approach, in Paxos, there’s-- well, in Paxos, there is also a counter that’s being used in, not quite in the same way".
- p. 9, Levin: "You’ve also said it was the one you were proudest of. Can you say why?" Lamport: "No, I can’t." (the transcript then reads "<laughter>"). On p. 10 he answers: "Let me try to answer the question of why I’m proudest of it." … (F4's quote follows).
- Verified 2026-10-11 (R2).

**C-S28. Publications page: simplified versions (description; R2 D4).**
Same entry as C-S18: "Several books have included emasculated versions of the
algorithm in which reading and writing are atomic operations, and called
those versions "the bakery algorithm". I find that deplorable. There's
nothing wrong with publishing a simplified version, as long as it's called a
simplified version."
- The video labels its simplification: 08_9's card says "indivisible reads and writes", and 09–10 lift the assumption.
- Verified 2026-10-11 (R2).

**C-S29. Tuples compare in dictionary order in Rust and Python (description,
"Code"; dev-08).**
- Rust std docs, primitive type `tuple` (std 1.99.0): "The sequential nature of the tuple applies to its implementations of various traits. For example, in PartialOrd and Ord, the elements are compared sequentially until the first non-equal set is found." Verified 2026-10-11 (R2), doc.rust-lang.org/std/primitive.tuple.html.
- Python: `C-F13.python_tuple_compare_equals_lexicographic_on_every_grid_case` = 1, computed by the oracle on the interpreter that runs it.

**Not verified at R2, so off the screen and out of the description:** Burns
and Lynch, "Bounds on Shared Memory for Mutual Exclusion", Information and
Computation 107(2):171–184, 1993 (doi:10.1006/inco.1993.1065). The title,
authors, volume, issue and pages were checked against Crossref on 2026-10-11,
but the theorem's text could not be fetched (the publisher's page needs a
browser), so memo §2.3's "n read/write registers are necessary" (†) stays
unquoted until someone reads the paper (R2 dev-09, R16b).

## Edge tests for always/never sentences (R2, Ruth R4; plan §5.3)

Each always/never sentence, the test that probes it at its edge, and the key.
Every model test holds within its bounds only (customers N, tickets ≤ K;
"atomic" = indivisible reads and writes, "safe" = flickering slates and flags).

| line | sentence | edge tested | key = value |
|---|---|---|---|
| 01_8 | "Exactly the ones through the middle" | losing set equals middle set | `C-F1.losing_orders_are_exactly_the_middle_orders` = 1 |
| 04_3 | "both can end up in the room at once" (`<`); "forever" (`≤`) | `<` both inside in some orders; `≤` deadlocks | `C-F5.strict_lt_with_flag.orders_both_inside` = 432; `C-F6.b2.deadlocked` = 1; `C-X3.liveness.*.le_numbers_only.no_progress_state_found` = 1 |
| 04_6, 08_3 | "always takes the smallest pair of everyone holding a ticket"; "everyone else holding a ticket has a bigger pair" | every holder, flag still up included, N = 3, atomic K = 5 and safe K = 4 | `C-X4.fact_two_all_holders.n3_k5_atomic.violations` = 0; `….n3_k4_safe.violations` = 0 |
| 05_7, 06_2 | "not one does" | every order | `C-F3.flag.orders_both_inside` = 0; `C-F4.flag.orders_two_or_more_inside` = 0 |
| 05_8 | "many can be up at once" | all three flags up together is reachable | `C-X6.all_flags_up_at_once_reachable.n3_k5_atomic` = 1; `….n3_k4_safe` = 1 |
| 08_9 | "agrees, over every order, up to four customers" | N = 4, K = 4, atomic | `C-F8.n4_k4_atomic.violation_found` = 0 |
| 09_5 | "Never" (ticket glances); flag glances by the 09_5 lower-third | flickering slates and flags | `C-F8.n3_k6_safe.violation_found` = 0; memo §4 (corrected at R2) |
| 10_4 | "can break it" | needs N ≥ 3 and garbage reads | `C-F6.double_fetch.n3_k3_safe.violation_found` = 1; `….n2_k6_safe` = 0; `….n3_k5_atomic.violation_found` = 0 |
| 11_1 | "you're served first"; "either of us may go first" | disjoint and overlapping choosing | `C-X2.*.violation_found` = 0; `C-X10.choosing_disjoint.n2_one_visit.first_to_finish_choosing_enters_first` = 1; `C-X10.choosing_overlaps.n2_one_visit.first_to_enter_can_be` = 1,2 |
| 11_2 | "whoever holds the smallest pair gets through"; "at most once"; "nobody waits forever" | deadlock freedom; bypass counted from the ticket write | `C-X3.liveness.*.correct.no_progress_state_found` = 0; `C-X5.bypass_from_ticket_write.n3_k4_atomic.worst` = 1; `….n3_k3_safe.worst` = 1; C-S19, C-S24 |
| 11_3 | "They reset only when the bakery empties" | the largest ticket held never drops to a nonzero value | `C-X7.largest_ticket_drops_to_nonzero.n3_k5_atomic` = 0; `….n3_k4_safe` = 0 |
| 11_7 | "stalls everyone" | customer 1 frozen with her flag up | `C-X8.frozen_while_choosing.*.worst_entries_per_other_customer` = 1 (one more visit each, then nobody) |
| 11 card | "Paused in line → everyone behind waits" | customer 1 frozen at L2/L3 | `C-X9.paused_in_line.*.entries_by_customers_behind_her` = 0 |
| 12_5 | "two fences … are enough" (Intel, AMD) | x86-TSO, N ≤ 3 within bounds; each fence alone fails | `C-F9.f1_f2.violation_found` = 0; `C-F9.f1_only.violation_found` = 1; `C-F9.f2_only.violation_found` = 1; C-S21 |

## Simplification → exact (plan §5.1 rule 11)

Where the narration simplifies, the exact statement and where the viewer finds
it. The description's "Precisely" section gives each in full.

| line | said | exact | where |
|---|---|---|---|
| 01_7 | "There are six" | C(4, 2) = 6 interleavings of two 2-step programs | 01_7 lower-third |
| 01_9 | "Counting orders isn't measuring chances" | the counts are over orders, not a probability; a real scheduler weighs them differently | 01_9 lower-third |
| 02_4 | "fourteen orders" | each read, write and successful wait is one step; each customer visits once | 02_4 lower-third |
| 02_5–02_6 | Dijkstra's rules, Knuth's addition | Dijkstra 1965's (a)–(d) verbatim (C-S1) | description |
| 04_4–04_6 | "pair", "like a dictionary" | ≺ is the lexicographic order on (ticket, customer number), a strict total order with finitely many predecessors; "everyone holding a ticket" means at the same time (R13) | 04_6 lower-third; description |
| 05_7 | "twenty-eight orders … three hundred twenty-two" | one visit each; the 02_4 step rule | 05_7 lower-third |
| 05_8 | "choosing time" | Lamport's doorway | 05_8 on screen |
| 06_2 | the trails | uniform samples over the 2,126,816,238 orders, not a scheduler | 06_2 caption |
| 08_2–08_8 | facts one and two, three zones | memo §4's Lemma 1, Lemma 2 (cases (a), (b)), Assumption M, as corrected at R2 | description |
| 08_9 | "as long as tickets stay small" | N ≤ 4, tickets ≤ 4, atomic; runs that would pass 4 are cut | 08_9 card |
| 09_2–09_3 | "any number the slate could hold", "safe" | Lamport 1986's safe register (C-S2); regular and atomic | 09_3 on screen; description |
| 09_5 | "two glances at slates" | the flag check may overlap a write: mid-raise counts as not yet choosing, mid-lowering as finished | 09_5 lower-third |
| 10_1–10_2 | "a glance that catches Blue's old zero" | e3's 0 is the slate's value before the write (F3) | description |
| 10_4 | "reading each slate exactly once" | Assumption M (C-S20), needed for N ≥ 3 with garbage reads | description |
| 11_1–11_2 | "fair", "nobody waits forever" | FCFS from the end of the doorway; FCFS + deadlock freedom ⇒ lockout freedom (C-S24); bypass counted from the ticket write | description |
| 11_5–11_6 | "up to about thirty-one and a half billion"; "under two months"; "centuries" | 31,557,600,000 < 2³⁵ (Lamport's bound, 365.25-day years); 2³² ms = 49.7 days, 2³¹ ms = 24.9 days; 584.5 / 292.3 years; all assume no glance returns more than was written (C-S22 for how one can) | 11_5–11_6 cards; description |
| 12_2–12_5 | "outbox", "fence" | x86-TSO store buffer; MFENCE or a locked instruction (C-S10); model bounds | 12_2, 12_5 lower-thirds |
| 12_6 | "Others can see your flag come down before your ticket lands" | Arm's MP behaviour (C-S25); no fence count claimed for Arm | description |
| 13_2 | "Code that Arm itself publishes … before its fast memory is switched on" | TF-A, warm boot before the data cache is enabled, only when `HW_ASSISTED_COHERENCY == 0` | 13_2 lower-third |

## The table

One row per oracle key, in output order. Match is MATCH when the oracle value
equals the plan's stated value. It is n/a when the plan states no value for
that key. WORDING and NOTE rows are explained under Findings. "same" in the
last column means the method of the row above it. For the last 52 rows (added
at R2) the stated value is the R2 storyboard's, and the values were copied
from the R2 run's output by a script.

| id | quantity | oracle key = value | plan value | match | how verified |
|---|---|---|---|---|---|
| C-F1 | scoreboard start | `C-F1.score_start` = `41` | 41 (01_4) | MATCH | `row_cf1`: simulate every order from 41 |
| C-F1 | orders of two read-then-write programs | `C-F1.orders` = `6` | 6 (ledger §11.5) | MATCH | `row_cf1`: all 4!/(2!·2!) orders enumerated; cross-checked `tree_count.lost_update`, `outputs/tree_count_out.txt:1`, `plan_r1/creative_scripts/progress_paths.py` |
| C-F1 | orders that lose a point | `C-F1.orders_lose_a_point` = `4` | 4 (ledger §11.5) | MATCH | `row_cf1`: final score 42 instead of 43 |
| C-F1 | staircases through the middle (1,1) | `C-F1.orders_through_middle` = `4` | 4 (ledger §11.5) | MATCH | `row_cf1`: lattice path visits (1,1) |
| C-F1 | losing set = middle set | `C-F1.losing_orders_are_exactly_the_middle_orders` = `1` | "Exactly the ones that pass through the middle" (01_7) | MATCH | `row_cf1`: set equality |
| C-F1 | score after a lost point | `C-F1.score_end_when_a_point_is_lost` = `42` | 42 (01_5) | MATCH | `row_cf1` |
| C-F1 | score otherwise | `C-F1.score_end_otherwise` = `43` | — | n/a | `row_cf1` |
| C-F1 | C(4,2) | `C-F1.binomial_4_choose_2` = `6` | 6 (01 Ruth) | MATCH | `math.comb` |
| C-F2 | check-then-flip sign: orders | `C-F2.orders` = `14` | 14 (ledger §11.5) | MATCH | `row_cf2`: explicit path enumeration, one step per read, write, successful wait; cross-checked: `tree_count.count` (reused) + `outputs/tree_count_out.txt`:2 |
| C-F2 | orders that ever put both inside | `C-F2.orders_both_inside` = `8` | 8 (ledger §11.5) | MATCH | `row_cf2` |
| C-F2 | staircases through the red square | `C-F2.orders_through_red_square` = `8` | 8 (02_3 picture) | MATCH | `row_cf2`: path visits the both-inside lattice point |
| C-F2 | orders that deadlock | `C-F2.orders_deadlocked` = `0` | — | n/a | `row_cf2` |
| C-F2 | orders that END with both inside | `C-F2.orders_ending_with_both_inside` = `0` | "eight end with both in the room" (02_3 words) | WORDING | `row_cf2`: every order ends with both gone; the 8 have both inside at some point. See finding F1. |
| C-F3 | 2 customers, no flag: orders | `C-F3.no_flag.orders` = `28` | 28 (ledger §11.5) | MATCH | `two_customer_counts`; cross-checked: path enumeration == avoidance DP == `tree_count.count`; `outputs/crosscheck_out.txt` |
| C-F3 | no flag: both inside | `C-F3.no_flag.orders_both_inside` = `4` | 4 (ledger §11.5) | MATCH | same |
| C-F3 | no flag: through red square | `C-F3.no_flag.orders_through_red_square` = `4` | 4 (05_6 picture) | MATCH | same |
| C-F3 | no flag: deadlocked | `C-F3.no_flag.orders_deadlocked` = `0` | — | n/a | same |
| C-F3 | 2 customers, with flag: orders | `C-F3.flag.orders` = `322` | 322 (ledger §11.5) | MATCH | same |
| C-F3 | with flag: both inside | `C-F3.flag.orders_both_inside` = `0` | 0 (ledger §11.5) | MATCH | same |
| C-F3 | with flag: through red square | `C-F3.flag.orders_through_red_square` = `0` | none (05_6 picture) | MATCH | same |
| C-F3 | with flag: deadlocked | `C-F3.flag.orders_deadlocked` = `0` | — | n/a | same |
| C-F4 | 3 customers, no flag: orders | `C-F4.no_flag.orders` = `792987` | 792,987 (ledger §11.5) | MATCH | `row_cf4`: avoidance DP (written here); cross-checked: `tree_count.count` (reused) + `outputs/tree_count_out.txt`:9 |
| C-F4 | no flag: 2+ inside | `C-F4.no_flag.orders_two_or_more_inside` = `312492` | 312,492 (ledger §11.5) | MATCH | same |
| C-F4 | no flag: deadlocked | `C-F4.no_flag.orders_deadlocked` = `0` | — | n/a | same |
| C-F4 | 3 customers, correct: orders | `C-F4.flag.orders` = `2126816238` | 2,126,816,238 (ledger §11.5; 05 engine note) | MATCH | `row_cf4`; cross-checked: `tree_count.count` (reused) + `outputs/tree_count_out.txt`:12 |
| C-F4 | correct: 2+ inside | `C-F4.flag.orders_two_or_more_inside` = `0` | 0 (ledger §11.5) | MATCH | same |
| C-F4 | correct: deadlocked | `C-F4.flag.orders_deadlocked` = `0` | — | n/a | same |
| C-F5 | `<` numbers only, with flag: orders | `C-F5.strict_lt_with_flag.orders` = `898` | 898 (ledger §11.5) | MATCH | `two_customer_counts`; cross-checked: path enumeration == avoidance DP == `tree_count.count`; `outputs/crosscheck_out.txt` |
| C-F5 | `<`: both inside | `C-F5.strict_lt_with_flag.orders_both_inside` = `432` | 432 (ledger §11.5) | MATCH | same |
| C-F5 | `<`: deadlocked | `C-F5.strict_lt_with_flag.orders_deadlocked` = `0` | — | n/a | same |
| C-F5 | `≤` numbers only, with flag: orders | `C-F5.le_with_flag.orders` = `250` | 250 (ledger §11.5) | MATCH | same |
| C-F5 | `≤`: both inside | `C-F5.le_with_flag.orders_both_inside` = `0` | 0 (ledger §11.5) | MATCH | same |
| C-F5 | `≤`: deadlocked | `C-F5.le_with_flag.orders_deadlocked` = `72` | 72 (ledger §11.5) | MATCH | same |
| C-F6 | trace a (no flag) length | `C-F6.a.steps` = `8` | 8 (ledger §11.5; 05_3) | MATCH | `row_cf6`; replayed step by step on `bakery_mc` (schedule copied from `scenarios.py`); step count == `outputs/scenarios_out.txt`; memo §3.3–3.5 |
| C-F6 | trace a: in the room at the end | `C-F6.a.in_room_at_end` = `2` | "Both in the room" (05_3) | MATCH | same |
| C-F6 | trace a′ (same order, flag) length | `C-F6.a_prime.steps` = `15` | 15 (ledger §11.5; 05_5) | MATCH | same |
| C-F6 | trace a′: most ever in the room | `C-F6.a_prime.max_in_room` = `1` | — | n/a | same |
| C-F6 | trace a′: customer 2 waits on the flag at step | `C-F6.a_prime.customer2_stopped_by_flag_at_step` = `7` | happens (05_5) | n/a | same; memo §3.3 says step 7 |
| C-F6 | trace b1 (`<`, both enter) length | `C-F6.b1.steps` = `12` | 12 (ledger §11.5) | MATCH | same |
| C-F6 | trace b1: in the room at the end | `C-F6.b1.in_room_at_end` = `2` | "both end up in the room" (04_3) | MATCH | same |
| C-F6 | trace b2 (`≤`) length | `C-F6.b2.steps` = `14` | — | n/a | same |
| C-F6 | trace b2: every enabled move is a self-loop | `C-F6.b2.deadlocked` = `1` | deadlock (ledger §11.5; 04_3) | MATCH | same |
| C-F6 | trace c (tie) length | `C-F6.c.steps` = `6` | — | n/a | same |
| C-F6 | trace c: tickets | `C-F6.c.tickets` = `1,1` | tie (ledger §11.5) | MATCH | same; equal tickets |
| C-F6 | trace c′ (3 customers) length | `C-F6.c_three_customers.steps` = `13` | — | n/a | same |
| C-F6 | trace c′: tickets | `C-F6.c_three_customers.tickets` = `2,2,1` | — (04_2 says 5 → 6, illustrative) | n/a | same; both see 1, both take 2 (memo §3.3) |
| C-F6 | trace d (4 customers) length | `C-F6.d.steps` = `70` | 70 (ledger §11.5) | MATCH | same; memo §3.4 |
| C-F6 | trace d: room entries | `C-F6.d.room_entries` = `5` | — | n/a | same |
| C-F6 | trace d: serving order (ticket, customer) | `C-F6.d.service_order` = `(1,2)(1,3)(2,4)(3,1)(4,2)` | (1,2) → (1,3) → (2,4) → (3,1) → (4,2) (06_4) | MATCH | same; ticket read from the state at each room entry |
| C-F6 | trace d: order is the grid's | `C-F6.d.service_order_is_lexicographic` = `1` | "exactly the grid's" (06_4) | MATCH | same |
| C-F6 | trace d: tickets taken, in order | `C-F6.d.tickets_taken` = `P3:1,P2:1,P4:2,P1:3,P2:4` | 3 and 2 take 1; 4 takes 2; 1 takes 3; 2 returns, takes 4 (06_2–06_4) | MATCH | same; from the ticket-write steps |
| C-F6 | trace d: customer 3 stopped by 2's flag at step | `C-F6.d.customer3_stopped_by_customer2_flag_at_step` = `12` | happens (06_2) | n/a | same |
| C-F6 | trace d: most ever in the room | `C-F6.d.max_in_room` = `1` | — | n/a | same |
| C-F6 | trace d: customer 1's waits, (ticket, customer) in order | `C-F6.d.customer1_waits_behind` = `(1,3)(2,4)` | Purple's 1, then Teal's 2 (storyboard 07_5: "She waits for Purple, then Teal") | MATCH | same; customer 1's L3 waits in step order (steps 49 and 54), repeats merged; added at the claim walk (G3) |
| C-F6 | trace e1 length | `C-F6.e1.steps` = `18` | — (ledger §11.5 lists e1 without a length) | n/a | same (safe registers, K = 9) |
| C-F6 | e1: value of the read that overlaps a write | `C-F6.e1.overlapping_read_value` = `7` | seven (08_5) | MATCH | same; checked: read overlaps a write and is neither 0 (old) nor 1 (new) |
| C-F6 | e1: ticket taken | `C-F6.e1.ticket_taken_after_garbage` = `8` | eight (08_5) | MATCH | same |
| C-F6 | e1: most ever in the room | `C-F6.e1.max_in_room` = `1` | — ("only makes her later") | n/a | same |
| C-F6 | trace e3 length | `C-F6.e3.steps` = `30` | — | n/a | same |
| C-F6 | e3: value of the overlapping read | `C-F6.e3.overlapping_read_value` = `0` | "a garbage zero" (08_5) | MATCH | same |
| C-F6 | e3: that 0 is the slate's value before the write | `C-F6.e3.overlapping_read_equals_pre_write_value` = `1` | — (plan calls it garbage) | NOTE | same; see finding F3 |
| C-F6 | e3: the read lets customer 2 in | `C-F6.e3.customer2_enters_on_that_read` = `1` | "lets a customer straight in" (08_5) | MATCH | same |
| C-F6 | e3: final tickets | `C-F6.e3.final_tickets` = `9,8` | — (memo §3.5: (8,2) ≺ (9,1)) | n/a | same |
| C-F6 | e3: most ever in the room | `C-F6.e3.max_in_room` = `1` | "so she waits" (08_5) | MATCH | same |
| C-F6 | leapfrog length | `C-F6.leapfrog.steps` = `46` | — | n/a | `row_cf6`: schedule copied from `leapfrog.py`, replayed on `bakery_mc` |
| C-F6 | leapfrog tickets | `C-F6.leapfrog.tickets` = `1,2,3,4,5,6` | 1…6 (ledger §11.5; 09_3 odometer 1, 2, 3, 4, 5, 6) | MATCH | same; == `outputs/leapfrog_out.txt:2` |
| C-F6 | leapfrog final slates | `C-F6.leapfrog.final_tickets` = `0,6` | — | n/a | same |
| C-F6 | double-fetch max, 3 customers, safe: shortest violation | `C-F6.double_fetch.n3_k3_safe.shortest_violation_steps` = `33` | 33 (ledger §11.5; 09_6) | MATCH | `explore` (BFS, reused `bakery_mc`) as in `df_trace.py`; == `outputs/df_trace_out.txt:3` and its 33 trace lines |
| C-F6 | double-fetch N=3: states explored | `C-F6.double_fetch.n3_k3_safe.states` = `47642` | — | n/a | same |
| C-F6 | double-fetch N=3: breaks | `C-F6.double_fetch.n3_k3_safe.violation_found` = `1` | "it breaks" (09_6) | MATCH | same |
| C-F6 | double-fetch N=2, K=6 safe: states | `C-F6.double_fetch.n2_k6_safe.states` = `2224` | — | n/a | same; == `outputs/df_trace_out.txt:1` |
| C-F6 | double-fetch N=2, K=6: breaks | `C-F6.double_fetch.n2_k6_safe.violation_found` = `0` | only with 3+ customers (09_6) | MATCH | same |
| C-F6 | double-fetch N=2, K=9 safe: states | `C-F6.double_fetch.n2_k9_safe.states` = `4597` | — | n/a | same; == `outputs/df_trace_out.txt:2` |
| C-F6 | double-fetch N=2, K=9: breaks | `C-F6.double_fetch.n2_k9_safe.violation_found` = `0` | only with 3+ customers (09_6) | MATCH | same |
| C-F7 | slates | `C-F7.slates_before` = `0,3,0,0,2` | 0, 3, 0, 0, 2 (ledger §11.5) | MATCH | `row_cf7`: direct computation; memo §5 Ch3 |
| C-F7 | customer 4 reads | `C-F7.customer4_reads` = `0,3,0,2` | zero, three, zero, two (03_4) | MATCH | same |
| C-F7 | customer 4 writes | `C-F7.customer4_ticket` = `4` | 4 (ledger §11.5) | MATCH | same |
| C-F7 | slates after | `C-F7.slates_after` = `0,3,0,4,2` | 0, 3, 0, 4, 2 (ledger §11.5) | MATCH | same |
| C-F7 | served | `C-F7.service_order` = `5,2,4` | 5, 2, 4 (ledger §11.5) | MATCH | same |
| C-F7 | customer 1 (Blue) reads | `C-F7.customer1_reads` = `3,0,0,2` | three, zero, zero, two (storyboard 03_4) | MATCH | `row_cf7`: direct computation; the video's reader is customer 1 (R1, M3) |
| C-F7 | customer 1 writes | `C-F7.customer1_ticket` = `4` | Four (storyboard 03_5) | MATCH | same |
| C-F7 | slates after customer 1 | `C-F7.slates_after_customer1` = `4,3,0,0,2` | 4, 3, 0, 0, 2 (storyboard 03_5 picture, 04_1) | MATCH | same |
| C-F7 | served | `C-F7.service_order_customer1` = `5,2,1` | Pink, Green, Blue (storyboard 03_5) | MATCH | same |
| C-F7 | same, in the model: customer 4's reads | `C-F7.model.customer4_reads` = `0,3,0,2` | 0, 3, 0, 2 | MATCH | `row_cf7`: `bakery_mc` with N = 5, customers 2 and 5 pre-seated with 3 and 2; customer 4 runs its doorway |
| C-F7 | model: customer 4's ticket | `C-F7.model.customer4_ticket` = `4` | 4 | MATCH | same |
| C-F7 | model: distinct serving orders over ALL interleavings | `C-F7.model.distinct_service_orders` = `1` | one order ("Here that's customer five, then two, then four") | MATCH | `row_cf7`: BFS over every interleaving of 2, 4, 5 (no new arrivals) |
| C-F7 | model: that serving order | `C-F7.model.service_order` = `5,2,4` | 5, 2, 4 | MATCH | same |
| C-F7 | model: customer 1's reads | `C-F7.model.customer1_reads` = `3,0,0,2` | 3, 0, 0, 2 (storyboard 03_4) | MATCH | `row_cf7`: `bakery_mc` with N = 5, customers 2 and 5 seated with 3 and 2; customer 1 runs her doorway; added at the claim walk (G3) |
| C-F7 | model: customer 1's ticket | `C-F7.model.customer1_ticket` = `4` | 4 (storyboard 03_5) | MATCH | same |
| C-F7 | model: distinct serving orders over ALL interleavings | `C-F7.model.customer1_distinct_service_orders` = `1` | one order (storyboard 03_5) | MATCH | `row_cf7`: BFS over every interleaving of 1, 2, 5 (no new arrivals) |
| C-F7 | model: that serving order | `C-F7.model.customer1_service_order` = `5,2,1` | Pink, Green, Blue (storyboard 03_5) | MATCH | same |
| C-F7 | tie: customer 3 (Purple) reads | `C-F7.model.tie.customer3_reads` = `4,3,0,2` | "Both see four as the biggest" (storyboard 04_2) | MATCH | `row_cf7`: from the state after customer 1's doorway (slates 4, 3, 0, 0, 2, nobody served), customers 3 and 4 run their doorways side by side, read for read; added at the claim walk (G3) |
| C-F7 | tie: customer 4 (Teal) reads | `C-F7.model.tie.customer4_reads` = `4,3,0,2` | same (storyboard 04_2) | MATCH | same |
| C-F7 | tie: slates after both write | `C-F7.model.tie.slates_after` = `4,3,5,5,2` | "both write five" (storyboard 04_2) | MATCH | same |
| C-F7 | tie: distinct serving orders over ALL interleavings | `C-F7.model.tie.distinct_service_orders` = `1` | — | n/a | `row_cf7`: BFS over every interleaving of all five (no new arrivals) |
| C-F7 | tie: that serving order | `C-F7.model.tie.service_order` = `5,2,1,3,4` | Purple (3) before Teal (4) (storyboard 04_4) | MATCH | same; Pink, Green, Blue, then Purple, then Teal |
| C-F8 | N=4, tickets ≤ 4, atomic: states (RECORDED) | `C-F8.n4_k4_atomic.states.recorded` = `3027633` | 3,027,633 (ledger §11.5; 07_7) | MATCH | RECORDED constant `REC_N4K4` from `outputs/sweep_big.txt:3` (711.7 s run); oracle checks the file still says so |
| C-F8 | same, recomputed live | `C-F8.n4_k4_atomic.states` = `3027633` | 3,027,633 | MATCH | `row_cf8`: `explore(Bakery(4, 4, 'correct'))` (reused `bakery_mc`), about 30 s; pruned count also equals the recording (204,059) |
| C-F8 | N=4 K=4: violation | `C-F8.n4_k4_atomic.violation_found` = `0` | no violation (07_7) | MATCH | same |
| C-F8 | N=3, tickets ≤ 6, safe: states (RECORDED) | `C-F8.n3_k6_safe.states.recorded` = `269550` | 269,550 (ledger §11.5; 08 Ruth) | MATCH | RECORDED constant `REC_N3K6` from `outputs/sweep_big.txt:2` |
| C-F8 | same, recomputed live | `C-F8.n3_k6_safe.states` = `269550` | 269,550 | MATCH | `row_cf8`: `explore(Bakery(3, 6, 'correct', safe=True))`, about 2 s; pruned 12,306 == recording |
| C-F8 | N=3 K=6 safe: violation | `C-F8.n3_k6_safe.violation_found` = `0` | zero violations (08 Ruth) | MATCH | same |
| C-F9 | TSO, no fence: states | `C-F9.no_fence.states` = `48646` | — | n/a | `row_cf9`: `bakery_tso.explore_tso` (reused), N=2, K=4, buffer ≤ 4; == `outputs/bakery_tso_out.txt:1` |
| C-F9 | TSO, no fence: fails | `C-F9.no_fence.violation_found` = `1` | fails (ledger §11.5) | MATCH | same |
| C-F9 | TSO, no fence: shortest failure | `C-F9.no_fence.shortest_violation_steps` = `12` | 12 (ledger §11.5; 10_2) | MATCH | same (BFS, so shortest) |
| C-F9 | memory flags when both are inside | `C-F9.no_fence.memory_choosing_at_violation` = `0,0` | "nobody is even choosing" (10_2) | MATCH | same |
| C-F9 | memory tickets when both are inside | `C-F9.no_fence.memory_tickets_at_violation` = `0,0` | memory bar frozen at zeros (10_2) | MATCH | same |
| C-F9 | in the room at that moment | `C-F9.no_fence.in_room_at_violation` = `2` | both enter (10_2) | MATCH | same |
| C-F9 | F1 only: states | `C-F9.f1_only.states` = `2298` | — | n/a | same; == `outputs/bakery_tso_out.txt:14` |
| C-F9 | F1 only: fails | `C-F9.f1_only.violation_found` = `1` | fails (ledger §11.5) | MATCH | same |
| C-F9 | F1 only: shortest failure | `C-F9.f1_only.shortest_violation_steps` = `18` | 18 (ledger §11.5) | MATCH | same |
| C-F9 | F2 only: states | `C-F9.f2_only.states` = `7484` | — | n/a | same; == `outputs/bakery_tso_out.txt:33` |
| C-F9 | F2 only: fails | `C-F9.f2_only.violation_found` = `1` | fails (ledger §11.5) | MATCH | same |
| C-F9 | F2 only: shortest failure | `C-F9.f2_only.shortest_violation_steps` = `20` | 20 (ledger §11.5) | MATCH | same |
| C-F9 | F1 + F2: states | `C-F9.f1_f2.states` = `1135` | 1,135 (ledger §11.5) | MATCH | same; == `outputs/bakery_tso_out.txt:54` |
| C-F9 | F1 + F2: fails | `C-F9.f1_f2.violation_found` = `0` | OK (ledger §11.5) | MATCH | same |
| C-F9 | F1 + F2, N=2, K=6, B=4: states | `C-F9.f1_f2.n2_k6_b4.states` = `1811` | — (memo §3.6: 1,811) | n/a | same, as `tso3.py`; == `outputs/tso3.txt:1` |
| C-F9 | F1 + F2, N=2, K=6: fails | `C-F9.f1_f2.n2_k6_b4.violation_found` = `0` | — ("N ≤ 3", 10 Ruth) | MATCH | same |
| C-F9 | F1 + F2, N=3, K=3, B=3: states | `C-F9.f1_f2.n3_k3_b3.states` = `37580` | — (memo §3.6: 37,580) | n/a | same; == `outputs/tso3.txt:2` |
| C-F9 | F1 + F2, N=3: fails | `C-F9.f1_f2.n3_k3_b3.violation_found` = `0` | OK for N ≤ 3 (10 Ruth) | MATCH | same |
| C-F10 | ms in a year (365.25 days) | `C-F10.ms_per_year` = `31557600000` | 31,557,600,000 (ledger §11.5; 09_4) | MATCH | `row_cf10`: arithmetic; Lamport 1974 p. 455 ("number[i] < 2^35") |
| C-F10 | 2^35 | `C-F10.two_pow_35` = `34359738368` | 34,359,738,368 (ledger §11.5) | MATCH | same |
| C-F10 | a year of ms < 2^35 | `C-F10.ms_per_year_below_two_pow_35` = `1` | < (ledger §11.5) | MATCH | same |
| C-F10 | 2^34 | `C-F10.two_pow_34` = `17179869184` | — | n/a | same (2^35 is the least power of two that works) |
| C-F10 | a year of ms < 2^34 | `C-F10.ms_per_year_below_two_pow_34` = `0` | — | n/a | same |
| C-F10 | ms in a 365-day year | `C-F10.ms_per_365_day_year` = `31536000000` | — | n/a | same (also < 2^35) |
| C-F10 | 2^32 ms in days, 1 decimal | `C-F10.u32_wrap_days_at_1_per_ms_1dp` = `49.7` | "2³² ms = 49.7 days" (storyboard 11_5 picture); "under two months" (11_5) | MATCH | `row_cf10`: arithmetic (R1, dev-02); unsigned 32-bit |
| C-F10 | 2^31 ms in days, 1 decimal | `C-F10.i32_wrap_days_at_1_per_ms_1dp` = `24.9` | — (Java `int`, the `AtomicIntegerArray` the 11 costs card warns against) | n/a | same; signed 32-bit; added at the claim walk (G3) |
| C-F11 | 2^64 tickets at 10^9/s, in years (365.25 d) | `C-F11.u64_years_at_1e9_per_s` = `584.5420460906264` | ≈584.5 (ledger §11.5) | n/a | `row_cf11`: arithmetic; full precision for the 1e-9 gate |
| C-F11 | same, 1 decimal | `C-F11.u64_years_at_1e9_per_s_1dp` = `584.5` | ≈584.5 (ledger §11.5) | MATCH | same |
| C-F11 | 2^63 (signed 64-bit), 1 decimal | `C-F11.i64_years_at_1e9_per_s_1dp` = `292.3` | "292.3 signed, as in Java" (storyboard 11_6 picture, from the claim walk) | MATCH | same; Java's `long` (`AtomicLongArray`, the 11 costs card) is signed, so 292.3 applies there; the Rust panel's AtomicU64 is unsigned, so 584.5 applies there |
| C-F11 | 2^64 at 10^9/s in 365-day years | `C-F11.u64_years_at_1e9_per_s_365_day_years_1dp` = `584.9` | — | n/a | same |
| C-F12 | reads of other slates, customer alone, N=2 | `C-F12.other_slate_reads_alone.model.n2` = `3` | 3(N−1) = 3 (ledger §11.5; 09_5) | MATCH | `model_reads_alone`: a lone customer stepped through `bakery_mc`; reads counted from the transition labels |
| C-F12 | same, N=3 | `C-F12.other_slate_reads_alone.model.n3` = `6` | 3(N−1) = 6 | MATCH | same |
| C-F12 | same, N=4 | `C-F12.other_slate_reads_alone.model.n4` = `9` | 3(N−1) = 9 | MATCH | same |
| C-F12 | same, N=5 | `C-F12.other_slate_reads_alone.model.n5` = `12` | 3(N−1) = 12 | MATCH | same |
| C-F12 | same, plan's Rust panel transliterated, N=5 | `C-F12.other_slate_reads_alone.rust_panel.n5` = `12` | 3(N−1) = 12 | MATCH | `rust_panel_lock` with counting slates |
| C-F12 | Rust panel: loads of its own slate | `C-F12.own_slate_loads_alone.rust_panel.n5` = `1` | — (not counted by the plan) | NOTE | same; the max reads all N slates, own included. See finding F5. |
| C-F12 | same, Lamport 1974 program transliterated, N=5 | `C-F12.other_slate_reads_alone.lamport1974.n5` = `12` | 3(N−1) = 12 | MATCH | `lamport_1974_lock` (j runs over all N, j = i included) |
| C-F12 | 3(N−1) holds for N = 2…8 in all three codes | `C-F12.other_slate_reads_equal_3_n_minus_1_for_n_2_to_8` = `1` | 3(N−1) (ledger §11.5) | MATCH | `row_cf12` |
| C-F13 | `(t[j], j) < (t[i], i)` with t=[0,5,3], i=2, j=1 | `C-F13.example.as_written` = `1` | evaluates `j < i` (ledger §11.5) | MATCH | `js_comma` emulation of the comma operator; I re-ran `node comma_operator_check.js` (node v26.8.1) on 2026-10-10: "evaluates to true \| j < i = true \| lexicographic would be false" |
| C-F13 | `j < i` for the same values | `C-F13.example.j_lt_i` = `1` | — | n/a | same |
| C-F13 | correct lexicographic comparison | `C-F13.example.lexicographic` = `0` | — (memo §1: false) | n/a | same |
| C-F13 | as written == `j < i` for every case (tickets 0–4, ids 0–3) | `C-F13.as_written_equals_j_lt_i_on_every_grid_case` = `1` | evaluates `j < i` (ledger §11.5) | MATCH | same |
| C-F13 | TypeScript version (RECORDED) | `C-F13.tsc_version.recorded` = `5.9.3` | 5.9.3 (ledger §11.5) | MATCH | RECORDED tool output (scripts README); I re-ran `tsc --noEmit --strict owner_snippet_lock.ts` on 2026-10-10: `tsc --version` = 5.9.3, no output, exit 0 |
| C-F13 | tsc diagnostics on the owner's snippet (RECORDED) | `C-F13.tsc_strict_diagnostics_on_owner_snippet.recorded` = `0` | silent (ledger §11.5) | MATCH | same |
| C-F14 | Dijkstra's problem | `C-F14.dijkstra_problem.year` = `1965` | 1965 (ledger §11.5; 02_4) | MATCH | CITED: Lamport 1974 ref. 4: "Comm. ACM 8, 9 (Sept. 1965), 569" |
| C-F14 |  | `C-F14.dijkstra_problem.citation` = `CACM 8(9):569, Sept 1965` | — | n/a | same |
| C-F14 | Knuth adds eventual entry | `C-F14.knuth_eventual_entry.year` = `1966` | 1966 (ledger §11.5; 02_4) | MATCH | CITED: Lamport 1974 ref. 1 "Comm. ACM 9, 5 (May 1966), 321-322"; Lamport 1986 Part II: "This requirement was first stated and satisfied by Knuth in [6]." |
| C-F14 |  | `C-F14.knuth_eventual_entry.citation` = `CACM 9(5):321-322, May 1966` | — | n/a | same |
| C-F14 | bakery paper | `C-F14.bakery_paper.year` = `1974` | Aug 1974 (ledger §11.5) | MATCH | CITED: bakery.pdf masthead "August 1974 Volume 17 Number 8", page numbers 453 and 455; Lamport publications page |
| C-F14 |  | `C-F14.bakery_paper.citation` = `CACM 17(8):453-455, Aug 1974` | CACM 17(8):453–455, Aug 1974 (ledger §11.5) | MATCH | same (ASCII hyphen in the key value) |
| C-F14 | received date | `C-F14.bakery_paper.received` = `September 1973` | — (memo §7: September 1973) | n/a | CITED: bakery.pdf p. 455 "Received September 1973; revised January 1974" |
| C-F14 | Reasoning About Nonatomic Operations | `C-F14.nonatomic_operations.year` = `1983` | POPL 1983 (ledger §11.5) | MATCH | CITED: Lamport publications page "(January 1983), 28-37" |
| C-F14 |  | `C-F14.nonatomic_operations.citation` = `POPL 10, Jan 1983, 28-37` | — | n/a | same |
| C-F14 | On Interprocess Communication | `C-F14.interprocess_communication.year` = `1986` | 1986 (ledger §11.5) | MATCH | CITED: Lamport publications page "Distributed Computing 1, 2 (1986), 77-101" |
| C-F14 |  | `C-F14.interprocess_communication.citation` = `Distributed Computing 1(2):77-101, 1986` | — | n/a | same |
| C-F14 | Time, Clocks | `C-F14.time_clocks.year` = `1978` | 1978 (ledger §11.5) | MATCH | CITED: Lamport publications page "Communications of the ACM 21, 7 (July 1978), 558-565" |
| C-F14 |  | `C-F14.time_clocks.citation` = `CACM 21(7):558-565, July 1978` | — | n/a | same |
| C-F14 | Turing lecture (written version) | `C-F14.turing_lecture.year` = `2015` | 2015 (ledger §11.5) | MATCH | CITED: Lamport publications page "Communications of the ACM, June 2015, Vol. 58 No. 6, Pages 71-76" |
| C-F14 |  | `C-F14.turing_lecture.citation` = `CACM 58(6):71-76, June 2015` | — | n/a | same |
| C-F14 | lecture delivered | `C-F14.turing_lecture.delivered` = `PODC, Paris, July 2014` | — | NOTE | CITED: Lamport publications page "my Turing lecture, which I gave at the PODC conference in Paris in July, 2014". See finding F7. |
| C-F14 | award year | `C-F14.turing_award.year` = `2013` | — | n/a | CITED: Lamport publications page "2013 ACM Turing Award (2014)" |
| C-F14 | "considered impossible" | `C-F14.considered_impossible.year` = `1973` | 1973 (ledger §11.5; 01_2) | MATCH | CITED: turing.pdf p. 4: "In 1973, this was considered impossible [4, page 88]." |
| C-F14 | Brinch Hansen page cited | `C-F14.considered_impossible.brinch_hansen_page` = `88` | — | n/a | same; ref. [4] = Brinch Hansen, Operating System Principles, 1973 |
| C-F14 | torn read: from | `C-F14.turing_lecture_garbage.from` = `9` | 9 (01_1; §11.1 pt 10) | MATCH | CITED: turing.pdf p. 4, verbatim sentence in §Open items |
| C-F14 | torn read: to | `C-F14.turing_lecture_garbage.to` = `10` | 10 | MATCH | same |
| C-F14 | torn read: value | `C-F14.turing_lecture_garbage.value` = `2496` | 2496 (01_1; §11.1 pt 10) | MATCH | same; glyph sizes checked: full-size digits, not 2^496 |
| C-F14 | publications page: from | `C-F14.publications_page_garbage.from` = `0` | — | n/a | CITED: Lamport publications page (bakery entry) |
| C-F14 | publications page: to | `C-F14.publications_page_garbage.to` = `1` | — | n/a | same |
| C-F14 | publications page: value | `C-F14.publications_page_garbage.value` = `7456` | 7456 (§11.1 pt 10) | MATCH | same |
| C-X1 | 09 rewritten as 10 a digit at a time: in-between reads | `C-X1.rewrite_09_as_10.reads_in_between` = `00,19` | "nineteen, or zero zero" (08_1) | MATCH | `row_cx1`: both write orders |
| C-X2 | FCFS ghost + Assertion 2, N=2 K=6 atomic: states | `C-X2.fcfs_and_assertion2.n2_k6_atomic.states` = `498` | — | n/a | `row_cx2`: `explore(..., fcfs=True, check_assert2=True)` (reused); == `outputs/sweep_fcfs.txt:2` |
| C-X2 | FCFS or Assertion 2 violation | `C-X2.fcfs_and_assertion2.n2_k6_atomic.violation_found` = `0` | FCFS holds (09_1) | MATCH | same |
| C-X2 | N=2 K=6 safe: states | `C-X2.fcfs_and_assertion2.n2_k6_safe.states` = `2304` | — | n/a | same; `:3` |
| C-X2 | FCFS or Assertion 2 violation | `C-X2.fcfs_and_assertion2.n2_k6_safe.violation_found` = `0` | FCFS holds (09_1) | MATCH | same |
| C-X2 | N=3 K=4 atomic: states | `C-X2.fcfs_and_assertion2.n3_k4_atomic.states` = `32491` | — | n/a | same; `:4` |
| C-X2 | FCFS or Assertion 2 violation | `C-X2.fcfs_and_assertion2.n3_k4_atomic.violation_found` = `0` | FCFS holds (09_1) | MATCH | same |
| C-X2 | N=3 K=4 safe: states | `C-X2.fcfs_and_assertion2.n3_k4_safe.states` = `168742` | — | n/a | same; `:5` |
| C-X2 | FCFS or Assertion 2 violation | `C-X2.fcfs_and_assertion2.n3_k4_safe.violation_found` = `0` | FCFS holds (09_1) | MATCH | same |
| C-X3 | liveness, correct, N=2 K=6 | `C-X3.liveness.n2_k6.correct.no_progress_state_found` = `0` | nobody waits forever (09_2) | MATCH | `row_cx3`: `explore(..., liveness=True)` (reused); == `outputs/sweep_live.txt:2`; deadlock-freedom only, not a fairness proof |
| C-X3 | liveness, `≤` numbers only, N=2 K=6 | `C-X3.liveness.n2_k6.le_numbers_only.no_progress_state_found` = `1` | "bow to each other forever" (04_3) | MATCH | same; `:3` |
| C-X3 | liveness, correct, N=3 K=4 | `C-X3.liveness.n3_k4.correct.no_progress_state_found` = `0` | nobody waits forever (09_2) | MATCH | same; `:4` |
| C-X3 | liveness, `≤`, N=3 K=4 | `C-X3.liveness.n3_k4.le_numbers_only.no_progress_state_found` = `1` | "bow to each other forever" (04_3) | MATCH | same; `:5` |
| C-F6 | trace a, counted the 02_4 way | `C-F6.a.steps_counted` = `6` | 6 (storyboard 05_3 features) | MATCH | `row_r2_counted_steps`: trace a's steps minus 2 `start` moves (dev-10) |
| C-F6 | trace a′, counted the 02_4 way | `C-F6.a_prime.steps_counted` = `13` | 13 (storyboard 05_5 picture) | MATCH | same; minus 2 blocked waits |
| C-F6 | trace d, counted the 02_4 way | `C-F6.d.steps_counted` = `64` | 64 (storyboard 07_1 step clock) | MATCH | same; minus 6 blocked waits |
| C-F6 | trace d: Green leaves before Blue checks her | `C-F6.d.customer2_out_before_customer1_checks_her` = `1` | "Green is out before Blue checks her" (07_5) | MATCH | same; Green's exit write precedes Blue's first L2 read of choosing[2] (M14) |
| C-F6 | trace e3: Green's flag down (end of write) | `C-F6.e3.customer2_finished_choosing_at_step` = `14` | "Green finished choosing" (10_2) | n/a | same; step numbers of the 30-step replay |
| C-F6 | trace e3: Green's check finds Blue's flag down | `C-F6.e3.customer2_saw_customer1_flag_down_at_step` = `17` | "Green had already seen it down" (10_2) | MATCH | same; after Green finished choosing (14) |
| C-F6 | trace e3: Blue starts raising her flag for the ticket Green glances at | `C-F6.e3.customer1_began_raising_flag_again_at_step` = `21` | "So Blue raised it after Green finished choosing" (10_2) | MATCH | same; 21 > 17 > 14 (M6) |
| C-F9 | 12_3's voiced order on the x86-TSO model: steps | `C-F9.voiced.steps` = `12` | 12 (12_4 picture) | MATCH | `row_r2_tso`: the schedule replayed on `bakery_tso.TSOBakery(2, 4, ())`, one matching transition per step (R11) |
| C-F9 | voiced order: in the room at the end | `C-F9.voiced.in_room_at_end` = `2` | "Both walk in" (12_4) | MATCH | same |
| C-F9 | voiced order: flags in memory | `C-F9.voiced.memory_choosing_at_end` = `0,0` | "the real flags … still say nobody is even choosing" (12_4) | MATCH | same |
| C-F9 | voiced order: tickets in memory | `C-F9.voiced.memory_tickets_at_end` = `0,0` | "… and slates" (12_4) | MATCH | same |
| C-F9 | voiced order: what sits in the outboxes | `C-F9.voiced.outboxes_at_end` = `c1=1,n1=1,c1=0;c2=1,n2=1,c2=0` | both outboxes full (12_3 picture) | MATCH | same; nothing is ever delivered |
| C-F9 | outbox may deliver writes to different slates out of order, fences F1+F2: states | `C-F9.ooo_outbox.f1_f2.states` = `1673` | — | n/a | `row_r2_tso`: `OutOfOrderOutbox(2, 4, F1+F2)` (oracle code; extra flush moves added to `bakery_tso`'s); N = 2, tickets ≤ 4, buffer ≤ 4; a model, not Arm (dev-02) |
| C-F9 | same: two in the room | `C-F9.ooo_outbox.f1_f2.violation_found` = `1` | description, 12_6 (model) | MATCH | same; cross-check: the in-order x86-TSO model with F1+F2 still holds |
| C-F9 | same: shortest violation | `C-F9.ooo_outbox.f1_f2.shortest_violation_steps` = `22` | 22 (Dev's own model, R2 dev-02) | MATCH | same (BFS, so shortest) |
| C-F9 | same: in the trace a lowered flag reaches memory while that customer's ticket is still in her outbox | `C-F9.ooo_outbox.f1_f2.flag_lands_before_ticket_in_trace` = `1` | "your flag can come down before your ticket lands" (description) | MATCH | same; scanned from the trace |
| C-X4 | fact two over every holder, N=3, tickets ≤ 5, atomic: states | `C-X4.fact_two_all_holders.n3_k5_atomic.states` = `27778` | — | n/a | `row_r2_always_never`: BFS on `bakery_mc`; == `outputs/sweep_safety.txt:46` (R4) |
| C-X4 | same: (in the room, holder) pairs checked | `C-X4.fact_two_all_holders.n3_k5_atomic.checks` = `2993` | — | n/a | same; a holder is anyone with a nonzero slate (flag still up included), not mid-exit-write |
| C-X4 | same: holder whose pair is not bigger | `C-X4.fact_two_all_holders.n3_k5_atomic.violations` = `0` | "everyone else holding a ticket has a bigger pair" (08_3); "always takes the smallest pair" (04_6) | MATCH | same; Ruth's R2 check gave 27,778 / 2,993 / 0 |
| C-X6 | all three flags up at once, reachable | `C-X6.all_flags_up_at_once_reachable.n3_k5_atomic` = `1` | "many can be up at once" (05_8) | MATCH | same BFS |
| C-X7 | transitions where the largest ticket held drops | `C-X7.largest_ticket_drops.n3_k5_atomic` = `545` | — | n/a | same BFS |
| C-X7 | … and drops to a nonzero value | `C-X7.largest_ticket_drops_to_nonzero.n3_k5_atomic` = `0` | "They reset only when the bakery empties" (11_3) | MATCH | same |
| C-X4 | fact two over every holder, N=3, tickets ≤ 4, flickering (safe) slates and flags: states | `C-X4.fact_two_all_holders.n3_k4_safe.states` = `92716` | — | n/a | same; == `outputs/sweep_safety.txt:37` |
| C-X4 | same: pairs checked | `C-X4.fact_two_all_holders.n3_k4_safe.checks` = `5611` | — | n/a | same; customers mid-exit-write have left the room and are not holders |
| C-X4 | same: violations | `C-X4.fact_two_all_holders.n3_k4_safe.violations` = `0` | (08_3, 04_6) | MATCH | same; Ruth's R2 check gave 92,716 / 5,611 / 0 |
| C-X6 | all flags up at once, safe | `C-X6.all_flags_up_at_once_reachable.n3_k4_safe` = `1` | (05_8) | MATCH | same |
| C-X7 | largest ticket drops, safe | `C-X7.largest_ticket_drops.n3_k4_safe` = `3150` | — | n/a | same |
| C-X7 | … to a nonzero value, safe | `C-X7.largest_ticket_drops_to_nonzero.n3_k4_safe` = `0` | (11_3) | MATCH | same |
| C-X5 | bypass ghost from the end of the ticket write, N=3, tickets ≤ 4, atomic: ghost states | `C-X5.bypass_from_ticket_write.n3_k4_atomic.ghost_states` = `45868` | — | n/a | `row_r2_liveness_edges`: BFS over (state, entries by each other customer since your ticket write, capped at 2) (R4) |
| C-X5 | same: most entries by one other customer before you | `C-X5.bypass_from_ticket_write.n3_k4_atomic.worst` = `1` | "at most once" (11_2) | MATCH | same; Ruth's R2 check gave 45,868 ghost states, worst 1 |
| C-X8 | customer 1 frozen with her flag up: start states | `C-X8.frozen_while_choosing.n3_k4_atomic.start_states` = `9763` | — | n/a | same row: every reachable state with customer 1 in her doorway and her flag up; she never moves again (dev-03) |
| C-X8 | same: most room entries by one other customer afterwards | `C-X8.frozen_while_choosing.n3_k4_atomic.worst_entries_per_other_customer` = `1` | "One customer frozen while choosing stalls everyone" (11_7), after at most one more visit each | MATCH | same |
| C-X9 | customer 1 paused while waiting (L2 or L3): start states | `C-X9.paused_in_line.n3_k4_atomic.start_states` = `7098` | — | n/a | same row: she never moves again (dev-03) |
| C-X9 | same: most entries by one other customer | `C-X9.paused_in_line.n3_k4_atomic.worst_entries_per_other_customer` = `1` | — | n/a | same; only someone whose pair ends up smaller can still enter, once |
| C-X9 | same: entries by anyone whose pair is bigger than hers | `C-X9.paused_in_line.n3_k4_atomic.entries_by_customers_behind_her` = `0` | "Paused in line → everyone behind waits" (11 costs card) | MATCH | same |
| C-X5 | bypass ghost, N=3, tickets ≤ 3, safe: ghost states | `C-X5.bypass_from_ticket_write.n3_k3_safe.ghost_states` = `102364` | — | n/a | same; Ruth's R2 check gave 102,364 |
| C-X5 | same: worst | `C-X5.bypass_from_ticket_write.n3_k3_safe.worst` = `1` | (11_2) | MATCH | same |
| C-X8 | frozen with her flag up, safe: start states | `C-X8.frozen_while_choosing.n3_k3_safe.start_states` = `25568` | — | n/a | same; includes freezes in the middle of a write |
| C-X8 | same: worst | `C-X8.frozen_while_choosing.n3_k3_safe.worst_entries_per_other_customer` = `1` | (11_7) | MATCH | same |
| C-X9 | paused in line, safe: start states | `C-X9.paused_in_line.n3_k3_safe.start_states` = `10158` | — | n/a | same |
| C-X9 | same: worst | `C-X9.paused_in_line.n3_k3_safe.worst_entries_per_other_customer` = `1` | — | n/a | same |
| C-X9 | same: entries by anyone behind her | `C-X9.paused_in_line.n3_k3_safe.entries_by_customers_behind_her` = `0` | (11 costs card) | MATCH | same |
| C-X10 | two customers, one visit, flags: orders in which both flags are up at once | `C-X10.choosing_overlaps.n2_one_visit.orders` = `272` | — | n/a | `row_r2_choosing_overlap`: the oracle's own engine (as C-F3); overlapping + disjoint = 322 |
| C-X10 | same: who can enter first | `C-X10.choosing_overlaps.n2_one_visit.first_to_enter_can_be` = `1,2` | "either of us may go first" (11_1) | MATCH | same |
| C-X10 | orders in which one finishes choosing before the other starts | `C-X10.choosing_disjoint.n2_one_visit.orders` = `50` | — | n/a | same |
| C-X10 | same: the first to finish choosing enters first | `C-X10.choosing_disjoint.n2_one_visit.first_to_finish_choosing_enters_first` = `1` | "Finish choosing … before I start … you're served first" (11_1) | MATCH | same; with C-X2's ghost check for repeated visits |
| C-F6 | double-fetch max, N=3, tickets ≤ 4, atomic: states | `C-F6.double_fetch.n3_k4_atomic.states` = `30762` | — | n/a | `row_r2_misc`: `explore` (reused `bakery_mc`); == `outputs/sweep_safety.txt:44` |
| C-F6 | same: breaks | `C-F6.double_fetch.n3_k4_atomic.violation_found` = `0` | it needs garbage reads (11 "Precisely") | MATCH | same |
| C-F6 | double-fetch max, N=3, tickets ≤ 5, atomic: states | `C-F6.double_fetch.n3_k5_atomic.states` = `45073` | — | n/a | same; == `outputs/sweep_safety.txt:54` |
| C-F6 | same: breaks | `C-F6.double_fetch.n3_k5_atomic.violation_found` = `0` | (11 "Precisely") | MATCH | same |
| C-F10 | ms in a 365.25-day year, in billions, to the nearest half | `C-F10.ms_per_year_billions_to_nearest_half` = `31.5` | "about thirty-one and a half billion" (11_5) | MATCH | `row_r2_misc`: arithmetic (M7) |
| C-F13 | Python's tuple `<` equals dictionary order on the C-F13 grid | `C-F13.python_tuple_compare_equals_lexicographic_on_every_grid_case` = `1` | "Python's tuples compare in dictionary order" (description) | MATCH | `row_r2_misc`: the running interpreter's own tuple comparison (dev-08) |
