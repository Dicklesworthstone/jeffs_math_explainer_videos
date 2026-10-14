# R3 review: bakery_algorithm, narrated draft. Ruth (ruth_45)

Reviewer: Prof. Ruth Adler, 45, mathematician (concurrency theory). I judged correctness first.
Pacing and vocabulary belong to Maya, and which mechanisms make the cut belongs to Dev.

**Verdict.** The mathematics is right.

- Every spoken count I recomputed matches.
- The mutual-exclusion argument in 08–10 is the corrected one. It survives a flag caught
  mid-flip, which Lamport's own 1974 proof did not.
- First-come-first-served is stated from the end of the doorway, and "nobody waits forever" is
  given its real hypotheses.
- The 2³⁵, 2³² and 2⁶⁴ arithmetic is exact.
- Every always/never sentence has its edge test.

One thing would change the video. Chapter 12 ends on a measured table whose green **0** for
Release/Acquire on Arm contradicts the voice's "not enough". Nothing on screen or in the
description says what a zero does and does not prove.

The other findings are small:

- a rounding on screen ("≈ 31.5 billion");
- a fact-one card that drops its hypothesis;
- a TF-A lower-third that goes beyond its source;
- the bounded-slate caveat missing from "survives garbage";
- an unmeasured "usually".

I would show this to my students once the major is fixed.

## What I checked, and how

- **Sheets.** I opened all 13 cue sheets, enlarged every tile, and matched each tile to its
  legend. Tile i is line i, and trailing tiles repeat the chapter's last frame, which is how the
  cue-sheet tool pads.
  - Anything drawn only after a line's 1.5 s frame is **unchecked on the render**.
  - For six such items I read the chapter source to learn what is drawn and when, and I say so
    wherever I rely on it:
    - the 06_2 counter, which climbs to the C-F4 total (the frame catches it at 26,032,966);
    - the 09_5 gold glances and flag lower-third, which are synced to "Green reading", "and Blue
      reading" and "Nobody was writing";
    - the 10_4 code and its POPL 1983 caption;
    - the 12_1 measured card;
    - the 12_7 measured table;
    - the 13_2 lower-third.
- **Recomputation.** I ran `python3 -I` inline, with no files written. I wrote my own enumerator
  of the two-customer programs: one step per read, write and successful wait, one visit each.
  - **C-F1:** 6 orders, 4 lose a point, C(4, 2) = 6.
  - **C-F2:** 14 orders, 8 with both inside.
  - **C-F3:** 28 / 4 without flags, 322 / 0 with flags.
  - **C-F5:** the "<" rule gives 898 / 432; the "≤" rule gives 250 orders, 72 ending deadlocked.
  - **Arithmetic:**
    - 365.25 × 86,400,000 = 31,557,600,000, which is less than 2³⁵ = 34,359,738,368;
    - 2³² ms = 49.71 days and 2³¹ ms = 24.86 days;
    - 2⁶⁴ ns = 584.54 years and 2⁶³ ns = 292.27 years.

  All of these match the oracle.
- **Oracle.** I imported `oracle.py` read-only (no bytecode written) and ran only `row_cf10`,
  `row_cf11`, `row_r2_misc` and `row_c10_13`.
  - The C-F10 and C-F11 keys equal the 11 cards.
  - The `C-M1.*` measured keys equal what the 12_1 card and the 12_7 table print. Each mode is
    60,000,000 visits:

    | mode | x86 | Arm |
    |---|---|---|
    | Relaxed | 1,006,846 | 2,314,899 |
    | Release/Acquire | 1,173,616 | 0 |
    | SeqCst | 0 | 0 |

    The long Arm run is 0 of 600,000,000.
  - I re-summed the records' `total` lines with awk.
- **Proof.** I worked 08_2–08_8, 09_4–09_5 and 10_1–10_2 against the description's Lemma 1,
  Lemma 2 (cases a and b) and Assumption M.
  - The narration is the corrected argument.
  - A flag glance caught mid-raise lands in case (a), and one caught mid-lowering lands in (b).
  - The two trusted ticket glances overlap no write.
  - 10_2's "So Blue raised it after Green finished choosing" follows from steps 14 < 17 < 21
    and holds even if Green's flag glance overlapped the raise.
  - For 12_5, I convinced myself that under x86-TSO, F1 and F2 carry Lemma 1 and both cases of
    Lemma 2 through for **every** N. The outbox is FIFO, loads are not reordered with loads,
    F2 puts i's ticket in memory before her flag checks, and F1 puts k's raise in memory before
    her reads. So "two fences are enough" is true beyond the checked bounds. That is my
    argument, not a recorded one.
- **Sources.** I re-read Trusted Firmware-A at the fact sheet's pinned commit (`510355478850…`):
  `lib/locks/bakery/bakery_lock_coherent.c` and `include/plat/arm/common/plat_arm.h`.
  - Everything else rests on the fact sheet's verbatim quotes, which I read.
  - The narrated historical claims match those quotes:
    - Dijkstra, CACM 8(9):569, 1965;
    - Knuth, 1966;
    - CACM 17(8), 1974, p. 454, "any arbitrary value";
    - "safe", named in print in 1986;
    - Holt;
    - POPL 1983;
    - the Turing lecture's 2496 and "considered impossible";
    - the CHM oral history, pp. 9–10, "proudest".
- **Unchecked.** These I took from the fact sheet, which records the oracle's cross-checks:
  - 2,126,816,238 (C-F4);
  - 3,027,633 and 269,550 (C-F8);
  - every C-F9 x86-TSO result;
  - C-X4 through C-X10;
  - trace d's 64 steps;
  - the 33-step double-fetch trace.
- **Pacing.** I judged pacing from single frames only, so those scores are provisional.

## Comprehension check

1. **One plus one is one.** Two programs doing x = x + 1 can lose a point because reading and
   writing are separate steps. Of the 6 interleavings, exactly the 4 through the "both read,
   neither wrote" corner lose it, so a fix must exclude that corner in every order, not just the
   likely ones.
2. **The room.** A vacant/occupied sign fails (8 of 14 orders put both in) because the check and
   the flip are separate. Dijkstra (1965) fixed the rules: never two in the room, an idle stop
   holds up nobody, any speeds. Knuth (1966) added that everyone eventually gets in. Dijkstra's
   solution needs reads that can't catch a write halfway, which leaves the "blur" question open.
3. **Take a number.** Lamport's 1974 bakery drops the shared ticket machine. Each customer writes
   only her own slate, takes one more than the biggest ticket she reads, and waits for smaller
   tickets, so no lower-level exclusion is needed.
4. **Same number.** Concurrent choosers can take equal tickets: "wait for smaller" lets both in,
   and "wait for smaller or equal" deadlocks. So tickets become (ticket, customer number) pairs
   in dictionary order, a strict total order. Whoever enters holds the smallest pair among the
   current holders.
5. **The unwritten ticket.** Without a "choosing" flag, a reader can see a not-yet-written ticket
   as 0 and walk in beside its owner (4 of 28 orders). Waiting until the flag is down removes
   this (0 of 322), though choosing can still overlap and ties still need the tie-breaker.
6. **Three customers.** Three customers visiting once have 2,126,816,238 orders and none puts two
   in the room. That check is exhaustive, but only for one visit each.
7. **One run.** A four-customer run of 64 steps serves in exact pair order. The flag stops Purple
   from reading Green's unwritten ticket, and no point is lost.
8. **Why never two.** Fact one: if Blue finished choosing before Green started and still holds
   her ticket, Green's ticket is bigger. Fact two: whoever is in the room holds a smaller pair
   than every other holder; it follows from a three-way split on where Green was at Blue's last
   flag check. So two in the room would each be smaller than the other.
9. **Reading garbage safely.** Re-read with flickering ("safe") memory, the proof trusts only two
   ticket glances, and neither overlaps a write. Garbage can only delay someone, and a flag
   caught mid-flip falls into the case split.
10. **The old zero.** Even a glance that returns Blue's old 0 mid-write is harmless: Blue raised
    her flag after Green finished choosing, so fact one with the roles swapped makes Blue's
    ticket bigger. Lamport noticed this robustness only after writing the proof, and in 1983 he
    added that the maximum must read each slate once.
11. **Fair at a price.** The algorithm is first-come-first-served from the end of choosing, and
    each other customer can pass you at most once after your ticket write, so nobody starves.
    The price:
    - tickets grow while the bakery never empties, so 32 bits overflow in under two months at
      one entry a millisecond;
    - 64 bits last centuries, provided no glance returns more than was written;
    - each entry costs 3(N − 1) reads.
12. **Your CPU.** Store buffers on x86, and more reordering on Arm, break the proof's "a finished
    write is seen by every later read". The code therefore needs fences (two per entry on x86)
    or, simplest, sequentially consistent atomics. Release/Acquire falls short because the
    doorway needs a store to stay ahead of a later load.
13. **Proudest.** Real locks use atomic read-modify-write instructions. Arm's own firmware still
    uses a bakery lock where those aren't guaranteed (a waking core with its cache off). The
    "bigger number, tie broken by ID" idea recurs in Lamport clocks and Paxos ballots. Lamport
    feels he discovered the algorithm rather than invented it, and says that is perhaps why he
    is proudest of it.

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight | note |
|---|---|---|---|---|---|---|
| 01 one plus one is one | 5 | 4 | 5 | 4 | 5 | 01_9's "usually" (R3-ruth-06) |
| 02 the room | 4 | 5 | 4 | 4 | 4 | Dijkstra's (d) unvoiced; handled in Precisely |
| 03 take a number | 5 | 5 | 4 | 4 | 4 | |
| 04 same number | 5 | 5 | 5 | 4 | 4 | the "≤" deadlock is the right counterexample |
| 05 the unwritten ticket | 5 | 5 | 5 | 4 | 4 | |
| 06 three customers | 4 | 5 | 4 | 4 | 4 | trails labelled as uniform samples, correctly |
| 07 one run | 4 | 5 | 4 | 3 | 3 | 64 steps fast; lanes dense at draft size |
| 08 why never two | 5 | 4 | 5 | 4 | 5 | the fact-one card drops its hypothesis (R3-ruth-03) |
| 09 reading garbage safely | 5 | 5 | 5 | 4 | 5 | |
| 10 the old zero | 4 | 4 | 4 | 4 | 4 | "Yes" needs unbounded slates (R3-ruth-05); 10_4 opens on an empty frame |
| 11 fair at a price | 4 | 4 | 4 | 4 | 4 | "≈ 31.5 billion" (R3-ruth-02) |
| 12 your CPU | 4 | **3** | 5 | 4 | 5 | the closing table's zeros (R3-ruth-01) |
| 13 proudest | 4 | 4 | 4 | 4 | 5 | the TF-A lower-third (R3-ruth-04) |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| R3-ruth-01 | Ruth | 12 / 12_7 closing table (and the 12_1 card) | major | Chapter 12 ends, silently, on "measured: points lost in 60,000,000 visits, 2 customers". Release/Acquire shows 1,173,616 on x86 but a green **0** on Arm, and the note adds "there Release/Acquire lost 0 of 600,000,000 in a longer run". The voice has just said "Acquire and release are not enough here", so a programmer on an Arm Mac will conclude the opposite for her machine. Nothing on screen or in the description says what a zero means. **(a)** A zero only means nothing was observed. Arm allows exactly this reordering: a release store followed by an acquire load of another slate. Per C-S14's (unrecorded) compiler report, Rust's Acquire loads compile to LDAPR on Apple targets, and LDAPR may complete before an earlier STLR. C++ [atomics.order] promises nothing here either. **(b)** A point is lost only when two customers' read-then-write of the scoreboard interleave, so zero lost points never shows "never two in the room". For SeqCst that guarantee comes from the proof plus the sequential-consistency guarantee (C-S12), and the zeros only corroborate it. A positive count, by contrast, does prove failure. **(c)** The numbers on screen are oracle keys (`C-M1.*`; I ran that row and they match), but the fact sheet has no C-M1 rows. Its C-S12 still says the stress run is "still to be recorded", claims.md's 12_7 row is still UNSOURCED, and the description's Precisely has no measured entry. | Add one line under the table: "0 = none seen in these runs, not none possible: Arm allows this reordering, and no language promises Release/Acquire works here. Zero lost points never proves 'never two in the room'; the proof does." Add a description Precisely entry "12_1, 12_7 (measured)": the command, the two CPUs, each mode's lost and visits (C-M1 keys), and that caveat. Add C-M1 rows to the fact sheet and update C-S12 and the claims.md 12_7 row. If Dev's R1 run (storyboard 12, "Honest data": 11 points lost with Release/Acquire on Arm) is recorded under c7v0.17, show it as the honest Arm counterexample. |
| R3-ruth-02 | Ruth | 11 / 11_5 (picture) | minor | The card reads "= 31,557,600,000 tickets ≈ 31.5 billion". To one decimal that is **31.6** billion (31.5576). The voice's "about thirty-one and a half billion" rounds to the nearest half (`C-F10.ms_per_year_billions_to_nearest_half` = 31.5). But "31.5" printed with one decimal reads as a one-decimal rounding, so voice and screen round differently and the screen is off at its own precision. | Print "≈ 31½ billion", which matches the voice and the key. |
| R3-ruth-03 | Ruth | 08 / 08_2 card (on screen through 08_9) | minor | The fact-one card reads "Blue finished choosing before Green started: Green's ticket is bigger than Blue's". The voice adds "and still holds that ticket", and Lemma 1 needs it, because Blue's slate must not be reset before Green reads it. As printed, the card is false: Blue takes 5 and leaves (her slate back to 0), then Green starts and takes 1. This card is the reference statement for the whole proof, and 10_2 calls back to it ("fact one the other way round"). | Card: "Blue finished choosing before Green started, and still holds her ticket: Green's ticket is bigger than Blue's." |
| R3-ruth-04 | Ruth | 13 / 13_2 Precisely lower-third | minor | I read TF-A at the fact sheet's pinned commit, and two clauses go beyond it. **(1)** "only on platforms without hardware coherency (HW_ASSISTED_COHERENCY off)" is true for PSCI's locks (`psci_private.h`). But `plat_arm.h` makes `arm_lock` a bakery lock in BL31 with no HW_ASSISTED_COHERENCY condition: `#define ARM_INSTANTIATE_LOCK static DEFINE_BAKERY_LOCK(arm_lock)` under `#if defined(IMAGE_BL31) \|\| …`. The firmware-design.rst line that the fact sheet itself quotes names that "power controller driver (`arm_lock`)" as a bakery-lock user. **(2)** "the bakery lock needs only plain loads and stores": `bakery_lock_coherent.c` also issues `dmbish()`, `dsb()`, `wfe()` and `sev()`. What the source claims is that it "does not use mutual exclusion primitives". Chapter 12 has just taught that Arm needs ordering, so "only plain loads and stores" in an Arm lock's Precisely reads as the opposite lesson. | Lower-third: "…TF-A's PSCI uses bakery locks only when HW_ASSISTED_COHERENCY is 0; exclusive-access spinlocks need cacheable memory; the bakery lock needs no exclusive-access instructions, only aligned loads and stores plus barriers." Mirror this in description 13_2 and fact-sheet C-S17. A footnote I enjoyed, worth a description line: TF-A packs the choosing flag and the ticket into one word (`lock_data[me] = make_bakery_data(CHOOSING_TICKET, …)`) and writes both with one store. |
| R3-ruth-05 | Ruth | 10, 11 / 10_5, 11_6 | minor | The 10_5 "Yes" holds for slates that can hold any integer, which is Lamport's setting. On a fixed-width slate, 09_2's "any number the slate could hold" includes the top value. A glance that returns 2⁶⁴ − 1 makes 1 + max overflow, and in a Rust release build it wraps to 0, which reads as "not in line". So "survives garbage" and "64 bits last centuries" hold under different hypotheses, and the video never says a single 64-bit slate can't have both. 11_6 comes close ("One huge glance makes the tickets after it huge too"), but its picture uses 2⁶³, chosen so that 2⁶³ + 1 still fits. | Description Precisely (10_5 and 11_4–11_6): "Lamport's garbage tolerance assumes slates that hold any integer. On a b-bit slate a glance may return 2^b − 1 and the next ticket overflows, so in practice it is paired with reads that never exceed what was written, which untorn aligned words give." Optionally end the 11_6 picture on 2⁶⁴ − 1 and a ticket that wraps. |
| R3-ruth-06 | Ruth | 01 / 01_9 | minor | "Each program usually runs both steps back to back" is a frequency claim that nobody measured. It contradicts the description's Precisely for 01_9: "the video claims no rate for a real machine". How often the unlucky orders occur depends entirely on the machine and the load. | Cut the sentence, or say "A real machine doesn't pick orders evenly; the unlucky ones may be rare or common. But a fix has to work in every order." |
| R3-ruth-07 | Ruth | 01 / 01_7–01_8 | delight | Six staircases, "6 = C(4, 2)" on the card, and "exactly the ones through the middle": a lost update turned into a lattice-path fact. 01_9 then refuses to turn counts into chances, with "each tick is one step (an illustration, not a measurement)". | Keep as is (with R3-ruth-06's wording). |
| R3-ruth-08 | Ruth | 08–09 / 08_4–08_8, 09_5 | delight | "Three places" at Blue's last flag check is the cleanest presentation of Lemma 2 I have seen at this level. "It trusts only two glances at slates … Nobody was writing either slate" says exactly why safe registers suffice, and the 09_5 flags lower-third puts mid-raise and mid-lowering in the right cases. | Keep as is. |
| R3-ruth-09 | Ruth | 12 / 12_1 | delight | 08_1's card "the proof counts on" comes back, and its middle clause, "a finished write is seen by every later read", cracks over the chip. That is the right way to say "the proof is right; the machine isn't the model". | Keep as is. |
| R3-ruth-10 | Ruth | 10 / 10_3 (description 09–10) | delight | A historical footnote I didn't know: Lamport's 1974 proof treated a one-bit flag as uncatchable mid-flip, which he later called "one small but significant error", and the video's case split is the repair. It lives only in the description. | If Maya's pacing allows, add a lower-third at 10_3: "Lamport's 1974 proof wrongly assumed a one-bit flag can't be caught mid-flip; the case split here doesn't need that." Otherwise keep it in the description. |
