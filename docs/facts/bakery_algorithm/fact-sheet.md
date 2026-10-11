# Fact sheet: Video C, "The Algorithm Lamport Is Proudest Of"

Bead `mx-e-c-7v0.2`. This sheet covers every row C-F1 … C-F14 of the facts
ledger (plan §11.5), the numbers spoken in plan §11.4 that belong to those
rows, and three supplementary rows (C-X1 … C-X3) for narrated claims the
ledger does not list. Every value in the table was copied from the oracle's
actual output by a script, not typed by hand.

```
python3 -I docs/facts/bakery_algorithm/oracle.py        # from the repo root
```

- **Output:** 168 `KEY<TAB>VALUE` lines on stdout. Provenance and 53
  cross-checks go to stderr. The exit status is 1 if any cross-check fails.
- **Last runs** (2026-10-10, Apple M4 Pro, CPython 3.14.6): 34.4 s and 34.0 s
  wall in two runs with byte-identical stdout; 168 keys; 53/53 cross-checks
  passed; exit 0. Peak memory is about 1.8 GB, almost
  all of it the live N = 4 sweep. The run writes nothing outside stdout and
  stderr (`sys.dont_write_bytecode` keeps `__pycache__` out of the scripts
  folder).
- **Result against the plan:** 111 MATCH, 0 MISMATCH. 53 rows are n/a because
  the plan states no value for them (supporting values). 1 WORDING and 3 NOTE
  rows are explained under Findings.

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
    2 s. The live counts equal the recordings, pruned counts included
    (204,059 and 12,306). So none of the ledger's model-checker numbers rests
    on a recording alone.
  - `C-F13.tsc_version.recorded` and
    `C-F13.tsc_strict_diagnostics_on_owner_snippet.recorded` are tool outputs
    Python cannot reproduce. I re-ran `tsc --noEmit --strict
    owner_snippet_lock.ts` on 2026-10-10: version 5.9.3, no output, exit 0.
    `node comma_operator_check.js` (node v26.8.1) printed `(t[j], j) < (t[i],
    i) evaluates to true | j < i = true | lexicographic would be false`.
- **Cited.** C-F14 values are pinned from the primary sources quoted below.
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
    both take 2).
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
1974), section "The Algorithm", p. 453:
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

## The table

One row per oracle key, in output order. Match is MATCH when the oracle value
equals the plan's stated value. It is n/a when the plan states no value for
that key. WORDING and NOTE rows are explained under Findings. "same" in the
last column means the method of the row above it.

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
| C-F2 | orders that END with both inside | `C-F2.orders_ending_with_both_inside` = `0` | "eight end with both in the room" (02_3 words) | WORDING | `row_cf2`: every order ends with both gone; the 8 have both inside at some point. See finding F1. |
| C-F2 | orders that deadlock | `C-F2.orders_deadlocked` = `0` | — | n/a | `row_cf2` |
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
| C-F7 | same, in the model: customer 4's reads | `C-F7.model.customer4_reads` = `0,3,0,2` | 0, 3, 0, 2 | MATCH | `row_cf7`: `bakery_mc` with N = 5, customers 2 and 5 pre-seated with 3 and 2; customer 4 runs its doorway |
| C-F7 | model: customer 4's ticket | `C-F7.model.customer4_ticket` = `4` | 4 | MATCH | same |
| C-F7 | model: distinct serving orders over ALL interleavings | `C-F7.model.distinct_service_orders` = `1` | one order ("Here that's customer five, then two, then four") | MATCH | `row_cf7`: BFS over every interleaving of 2, 4, 5 (no new arrivals) |
| C-F7 | model: that serving order | `C-F7.model.service_order` = `5,2,4` | 5, 2, 4 | MATCH | same |
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
| C-F11 | 2^64 tickets at 10^9/s, in years (365.25 d) | `C-F11.u64_years_at_1e9_per_s` = `584.5420460906264` | ≈584.5 (ledger §11.5) | n/a | `row_cf11`: arithmetic; full precision for the 1e-9 gate |
| C-F11 | same, 1 decimal | `C-F11.u64_years_at_1e9_per_s_1dp` = `584.5` | ≈584.5 (ledger §11.5) | MATCH | same |
| C-F11 | 2^63 (signed 64-bit), 1 decimal | `C-F11.i64_years_at_1e9_per_s_1dp` = `292.3` | — | n/a | same; the Rust panel uses AtomicU64, so 584.5 applies |
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
