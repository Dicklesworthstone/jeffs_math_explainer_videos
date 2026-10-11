> 2026-10-09. All model-checking results below come from the scripts in `scripts/` (see `scripts/README.md`).

# Research memo: Lamport's Bakery Algorithm, for a 3Blue1Brown-style explainer (franken_manim)

**Tags used throughout:**
- **[V]** = a verified fact, with its source.
- **[O]** = how the owner frames it in his article.
- **[P]** = my creative proposal.
- **"Model-checked"** = checked by my exhaustive state-space explorer (Appendix A), within the bounds stated.

**Where the files are:** all scripts and raw outputs are here:
- Scripts: `docs/research/bakery_algorithm/scripts/`
- Outputs: `docs/research/bakery_algorithm/scripts/outputs/` (`sweep_*.txt`, `scenarios_out.txt`, `tso3.txt`)

---

## 0. Ten things the script must get right

1. **What "the first" actually means.** The bakery algorithm was the first mutual-exclusion algorithm that doesn't assume lower-level mutual exclusion. That means not even atomic reads and writes. Dekker's and Dijkstra's earlier solutions didn't use test-and-set either. What they assumed was *indivisible reads and writes* [V].
2. **The magic property.** A read that overlaps a write may return *any* value, and the algorithm is still correct. Lamport found this after the fact: his proof never used those values [V].
3. **Equal tickets are normal.** They happen in the correct algorithm. What is always unique is the pair (ticket, id). Ids break ties only; they are not a priority [V, model-checked].
4. **`choosing[]` is not a lock and does not prevent torn reads.** Its job is to stop you judging someone whose ticket isn't written yet [V, model-checked].
5. **FCFS is relative to finishing the "doorway".** It is not relative to "calling lock()". Starvation-freedom follows from FCFS plus deadlock-freedom [V].
6. **Tickets are unbounded.** Lamport's paper does the arithmetic: under 2^35 after a year at one entry per ms, *assuming reads never return values larger than were written* [V].
7. **It breaks on real x86/ARM unless fences are added.** In my x86-TSO model it needs two MFENCEs per process [V, model-checked].
8. **Computing the max needs care.** It must read each `number[k]` exactly once. Re-reading inside the max loop breaks the algorithm with non-atomic registers for N ≥ 3. This is Lamport's 1983 hidden assumption, and I reproduced the failure [V, model-checked].
9. **The owner's TypeScript snippet has a real bug.** `(this.tickets[j], j) < (this.tickets[i], i)` is JavaScript's comma operator, so it evaluates `j < i`. I checked with node, and tsc 5.9.3 does not flag it. The video's code panel must use an explicit lexicographic compare.
10. **"It works even if reading … yields any value whatsoever" is not a verbatim Lamport quote.** I couldn't find that wording in the 1974 paper, the Turing lecture, his publications page, or the oral history. Section 1 gives verbatim replacements.

---

## 1. The owner's article: framing to inherit, and fixes (b)

**Source:** https://jeffreyemanuel.com/writing/bakery_algorithm, "The Bakery Algorithm: Concurrency without Atomicity", Nov 22 2025. I read the full text and the bundled glossary and visualization code.

**[O] Framing**
- Subtitle: "Mutual exclusion without atomic hardware primitives. A masterclass in fair concurrency."
- Bakery metaphor, with emphasis that "unlike a physical bakery where a shopkeeper hands out tickets… there is no central authority".
- The headline claim: it works with plain reads and writes "even when those operations themselves are not atomic".
- Sections, in order:
  - "The First Correct Solution"
  - "How It Works": four phases — Number Assignment, Waiting for Turn, Critical Section, Exiting & Release
  - "The Tie-Breaker": "If both Process 1 and Process 2 see that 5 is the current max, they might both take number 6… If numbers are equal, the lower ID wins"
  - "The Process Nexus"
  - "The Magic of Non-Atomicity"
  - "The Implementation" (TypeScript)
  - Takeaways: Hardware Agnostic, Perfect Fairness (FCFS), Total Ordering, Busy Waiting
- Glossary analogies: a single-person bathroom lock (mutual exclusion), a deli-counter ticket, and "Are we there yet?" every 2 seconds (busy waiting).

**[O] Interactive pieces** (from the page's JS bundle):
- **BakeryHero:** a three.js field of floating ticket cards.
- **ProcessNexus:** processes on a radial ring; clicking a node requests entry. Status lines read like "P{m} (ticket n) is checking P{u}", "… choosing a ticket", "Holding the lock".
- **DoorwayRaceViz:** a "Temporal Scrubber" with five steps:
  1. Quiescent State
  2. The Choosing Guard: "both flags go HIGH"
  3. Concurrent Ticket Read: "both pick the same next ticket: 6"
  4. Logical Resolution: P0 wins the tie
  5. Starvation-Free Release
- **MemoryResilienceViz:** an "Inject Hardware Noise" slider and a "Torn Read Detected" monitor. Copy: "A torn read happens when a process reads 0xFFFF while the bus is halfway through writing 0x0001", and "Checking the choosing guard first essentially 'polls the voltage' until the signal stabilizes."

**Keep:** the no-central-authority point, the 5→6 tie example, and the doorway-race stepper. The noise slider is a good seed for the safe-register set piece.

**Fix in the video** (wording a professional would flag):

| Owner text | Problem | Use instead |
|---|---|---|
| "first correct published solution that didn't assume atomic hardware primitives" / "others relied on hardware tricks" | Dekker (2 processes) and Dijkstra (1965, N processes) used no test-and-set. They assumed "undividable" reads and writes. | "the first mutual-exclusion algorithm that assumes no lower-level mutual exclusion, not even atomic reads and writes" (Lamport, Turing lecture) |
| Quote "…yields any value whatsoever." | Not found verbatim. | 1974: "only the write operation must be performed correctly. The read may return any arbitrary value!"; or Turing lecture: "correct even if reading a number while it is changing from 9 to 10 obtains the value 2496." |
| "an inconsistent view only results in a process waiting slightly longer" | Garbage can also let a process *pass* early (traces e2/e3). It is safe because the other process is then guaranteed to defer. | "a garbage read can delay you, or let you through only when the other process is certain to wait for you" |
| Glossary: choosing "solves the torn read problem"; viz: choosing "polls the voltage until the signal stabilizes" | The flag doesn't stabilize anything, and torn reads of `number[]` still happen. | "choosing says: my ticket isn't decided yet, so don't compare against me" |
| "Hardware Agnostic… works on the simplest shared-memory hardware imaginable" | On modern CPUs it needs fences or sequentially consistent atomics. Lamport himself calls it "of little if any practical use". | "needs only single-writer memory cells, but real CPUs must be told not to reorder" |
| TS: `(this.tickets[j], j) < (this.tickets[i], i)` | Comma operator: this evaluates `j < i`. With tickets[j]=5, tickets[i]=3, j=1, i=2 it returns `true` (wait) where lexicographic order says `false`. | `t[j] < t[i] \|\| (t[j] === t[i] && j < i)` |
| Tooltip on that line reads "(number[i], i) < (number[j], j)" | Direction reversed. | the condition is "(number[j], j) ≺ (number[i], i)" |
| "In a distributed system…" | It is a shared-memory algorithm. Its single-writer variables are what make a message-passing version easy (Lamport). | "in a shared-memory system" |
| "Open Consensus" / "Consensus Stabilizer" labels | "Consensus" is a technical term. Read/write registers *cannot* solve wait-free consensus (Herlihy). | avoid the word |

---

## 2. Verified facts (a)

### 2.1 The algorithm as published [V]

**Source:** Lamport, "A New Solution of Dijkstra's Concurrent Programming Problem," CACM 17(8):453–455, Aug 1974. PDF: https://lamport.azurewebsites.net/pubs/bakery.pdf

**Setup**
- N computers communicate only via shared memory. Dijkstra's problem "as extended by Knuth" requires:
  1. at most one in the critical section (CS);
  2. each computer must eventually be able to enter (unless it halts);
  3. any computer may halt in its noncritical section;
  - and "no assumptions can be made about the running speeds."
- Each computer owns its words. "A processor may read from any other processor's memory, but it need only write into its own memory." This single-writer design is a first, per Lamport's publications page.

**Program for processor i**, with `choosing`, `number` initially 0:

```
L1: choosing[i] := 1;
    number[i]   := 1 + maximum(number[1],...,number[N]);   -- args read in any order
    choosing[i] := 0;
    for j = 1 step 1 until N do begin
      L2: if choosing[j] ≠ 0 then goto L2;
      L3: if number[j] ≠ 0 and (number[j], j) < (number[i], i) then goto L3;
    end;
    critical section;
    number[i] := 0;
    noncritical section; goto L1;
```

- Order on pairs: (a,b) < (c,d) if a < c, or a = c and b < d.
- Note that L3 loops back to L3, not to L2.

**Further statements in the paper**
- **Fault tolerance:** a processor may fail and restart with both variables at 0, "However, if a processor keeps failing and restarting, then it can deadlock the system." Fault tolerance is what the abstract sells.
- **The remarkable property:** "if a read and a write operation to a single memory location occur simultaneously, then only the write operation must be performed correctly. The read may return any arbitrary value!"
- **FCFS:** a processor first executes "a loop-free block of code" and is then "guaranteed to enter its critical section before any other processor which later requests service."

**Proof structure in the paper**
- *Doorway* = while choosing[i] = 1. *Bakery* = from resetting choosing to 0 until leaving the CS.
- **Assertion 1:** if i and k are in the bakery and i entered the bakery before k entered the doorway, then number[i] < number[k].
- **Assertion 2:** if i is in its CS and k ≠ i is in the bakery, then (number[i], i) < (number[k], k).
- **Assertion 3:** progress, assuming finitely many failures.
- "Assertions 1 and 2 prove that processors enter their critical sections on a first-come-first-served basis."

**Unbounded numbers**
- "If there is always at least one processor in the bakery, then the value of number[i] can become arbitrarily large."
- "if processors enter the doorway at the rate of at most one per msec, then after a year… number[i] < 2^35 — assuming that a read of number[i] can never obtain a value larger than one which has been written there."
- Arithmetic check: 31,557,600,000 ms/yr < 2^35 = 34,359,738,368.

### 2.2 Lamport's own commentary [V, verbatim]

**Writings page** (https://lamport.azurewebsites.net/pubs/pubs.html, entry for this paper):
- "I feel that I did not invent the bakery algorithm, I discovered it."
- "If the write changes the value from 0 to 1, a concurrent read could obtain the value 7456 (assuming that 7456 is a value that could be in the memory location). The algorithm still works. I didn't try to devise an algorithm with this property. I discovered that the bakery algorithm had this property after writing a proof of its correctness and noticing that the proof did not depend on what value is returned by a read that overlaps a write."
- Anatol Holt "was shocked. He refused to believe it could be true. He could find nothing wrong with my proof, but he was certain there must be a flaw."
- "Several books have included emasculated versions of the algorithm in which reading and writing are atomic operations… I find that deplorable."
- "Assuming that reads and writes of a memory location are atomic actions… is tantamount to assuming mutually exclusive access to the location… Before the bakery algorithm, people believed that the mutual exclusion problem was unsolvable… Brinch Hansen said exactly this in a 1972 paper."
- **The error:** "The paper contains one small but significant error. In a footnote, it claims that we can consider reads and writes of a single bit to be atomic…" He later realized "the fallacy in this reasoning". In the PDF this argument appears inside the proof of Assertion 2. The algorithm is still correct: later proofs and my model checking of safe `choosing` bits confirm it.
- The bakery's ever-increasing counter is linked to his later work. The bakery also introduced variables "belonging to a process" and "marked the beginning of my study of distributed algorithms."

**Turing lecture** (CACM 58(6):71–76, 2015; https://lamport.azurewebsites.net/pubs/turing.pdf):
- "Although of little if any practical use, the bakery algorithm has become a popular example…"
- "A literal implementation of this approach would require a ticket-machine process that never halts, violating Dijkstra's requirements. Instead, an entering process computes its own ticket number by reading the numbers of all other synchronizing processes and choosing a number greater than any that it sees."
- "…correct even if reading a number while it is changing from 9 to 10 obtains the value 2496."
- "the first algorithm to implement mutual exclusion without assuming any lower-level mutual exclusion. In 1973, this was considered impossible" (citing Brinch Hansen, *Operating System Principles*, p. 88). "Even in 1990, experts still thought it was impossible."

**"Proudest"** — Computer History Museum oral history (Roy Levin, 2016), p. 9: https://archive.computerhistory.org/resources/access/text/2017/07/102717182-05-01-acc.pdf
- Levin: "You've also said it was the one you were proudest of. Can you say why?" Lamport: "No, I can't." (laughter)
- Then: "the bakery algorithm, it just seemed to come out of thin air to me. There was nothing like it that preceded it, so perhaps that's why I'm proudest of it."
- Same interview, on the counter idea: "a sequence of numbers that can in fact increase forever, that you will see elsewhere… in the 'Time, Clocks' paper… in Paxos."
- **Origin story** (pp. 7–8): a 1972 CACM algorithm "seemed… awfully complicated". His own simple two-processor fix came back from the editor with "Here is the bug in your algorithm"; it "got me mad at myself for being such an idiot, and determined to solve the problem."

**Hidden assumption** — "Reasoning About Nonatomic Operations," POPL 1983 (https://lamport.azurewebsites.net/pubs/nonatomic.pdf):
- "For n > 2 we need the following additional assumption… If number[j] does not change during execution of the statement, then number[i] is chosen to be greater than number[j] — regardless of the concurrent activity of other processes."
- So `temp:=0; for k: if number[k] > temp then temp := number[k]` "is not a valid implementation if the two instances of number[k] represent separate fetches."

**Teaching Concurrency** (SIGACT News 2009) does not name the bakery algorithm. It alludes to it: "Can one implement mutual exclusion without using lower-level constructs that… assume mutually exclusive access to a resource? Quite a few people who think they are experts on concurrency can't."

### 2.3 Background [V]

Items marked † were verified by my research sub-agent from the URL shown. I personally re-read every Lamport source.

- **Dijkstra 1965**, CACM 8(9):569. N processes; assumes reading and writing a word are "undividable operations"; not starvation-free.†
  - EWD123 credits Dekker with the first correct 2-process solution: https://www.cs.utexas.edu/~EWD/transcriptions/EWD01xx/EWD123.html
  - Knuth 1966, CACM 9(5):321–322, added the "eventually enters" requirement (Lamport's 1986 Part II: "first stated and satisfied by Knuth").†
- **Peterson 1981**, "Myths about the mutual exclusion problem," IPL 12(3):115–116: flag[] plus a shared `turn` that both processes write (multi-writer, unlike the bakery).†
- **Registers** — Lamport, "On Interprocess Communication," Distributed Computing 1(2), 1986 (https://lamport.azurewebsites.net/pubs/interprocess.pdf):
  - **Safe:** "a read not concurrent with any write obtains the correct value… No assumption is made about the value obtained by a read that overlaps a write, except that it must obtain one of the possible values of the register."
  - **Regular:** an overlapping read gets the old value or one of the values being written.
  - **Atomic:** reads and writes "behave as if they occur in some definite order".
  - His Figure 5 is a ready-made visual. Values 5, 6, 27; write 5, then write 6; read1 before it, read2 and read3 overlapping the write of 6. Safe: read2/read3 ∈ {5, 6, 27}. Regular: ∈ {5, 6}, and read2=6 with read3=5 is allowed. Atomic: only (5,5), (5,6), (6,6).
- **FCFS** — Lamport, "The Mutual Exclusion Problem Part II," JACM 33(2), 1986 (https://lamport.azurewebsites.net/pubs/mutual2.pdf):
  - "if process i finishes executing its doorway statement before process j begins executing its doorway statement, then i must execute its critical section before j does."
  - "Theorem 1: FCFS and deadlock freedom imply lockout freedom."
- **Space lower bound** — Burns & Lynch, Inf. & Comp. 107(2):171–184, 1993: n read/write registers are necessary for deadlock-free mutual exclusion among n processes.† The bakery uses 2N, so it is within a factor of 2.
- **Herlihy**, "Wait-free synchronization," TOPLAS 13(1), 1991: read/write registers have consensus number 1; compare-and-swap has ∞.† There's no conflict with the bakery, because a lock is *allowed* to block.
- **Bounded variants:**
  - Taubenfeld, "The Black-White Bakery Algorithm…," DISC 2004, LNCS 3274:56–70. "Using only one additional shared bit, we bound the amount of space"; uses *atomic* registers.† https://disc.epfl.ch/abstracts/22.html
  - Jayanti, Tan, Friedland & Katz, "Bounding Lamport's Bakery Algorithm," SOFSEM 2001, LNCS 2234:261–270 (citation verified; content not read).
  - Aravind & Hesselink, "Nonatomic dual bakery algorithm with bounded tokens," Acta Informatica 48(2), 2011: bounded and nonatomic.†
  - Lamport's own route to bounding was "Concurrent Reading and Writing" (1977): write the digits one way, read them the other, so an overlapping read can't return a too-large value (publications page).
- **Hardware:**
  - x86-TSO (Sewell et al., CACM 53(7), 2010; https://www.cl.cam.ac.uk/~pes20/weakmemory/cacm.pdf): the store-buffering test can see 0/0, and MFENCE forbids it.†
  - Intel SDM Vol. 3A §11.2.3.4: "Loads May Be Reordered with Earlier Stores to Different Locations."†
  - ARM/POWER (Maranget, Sarkar, Sewell 2012 tutorial): SB/Dekker is allowed; "Adding a DMB or sync on both threads suffices."†
  - Abdulla et al., TACAS 2012 (https://user.it.uu.se/~parosha/publications/papers/tacas12.pdf): fence inference for the bakery under TSO gives two fences per process.† My model independently reproduces this (§3.6).
  - Milewski 2008: "Peterson lock on the x86 is broken!… An mfence will force the correct ordering."† https://bartoszmilewski.com/2008/11/05/who-ordered-memory-fences-on-an-x86/
- **Still used today:** Arm Trusted Firmware-A ships a bakery lock (`lib/locks/bakery`). Its header says exclusive-access instructions "have defined behavior only on certain memory types" and not all contenders "may have address translation enabled". It relies on single-copy-atomic aligned accesses plus `dmb`/`dsb` barriers. https://codex.ac6.fr/Zephyr/xref/trusted-firmware-a-latest/lib/locks/bakery/bakery_lock_normal.c
  - Ticket locks (fetch-and-add on a shared dispenser, i.e. a literal "ticket machine") became Linux x86 spinlocks via Nick Piggin's "x86: FIFO ticket spinlocks" (2008): https://lkml.iu.edu/hypermail/linux/kernel/0711.0/0125.html

---

## 3. Model checking (built and run; Appendix A)

### 3.1 Method and honest limits

- **Interleaving semantics.** Each shared read or write is a step. The max is N−1 separate reads, in *any order* (nondeterministic), as the paper permits. Local computation is free.
- **Two register models:**
  - *atomic:* each access is one step.
  - *safe:* each write is two steps (begin/end). Any read made while the register is being written returns *any* value in its domain: number ∈ 0..K, choosing ∈ {0,1}. Each overlapping read is chosen independently, which also covers new-then-old inversions.
  - Why point-reads are enough for single-writer registers: a read interval overlaps a write exactly when some point inside it does, and the reader does nothing else during its read.
- **Bounds:**
  - Tickets are capped at K; a step that would write a ticket > K is pruned.
  - So "OK" means *no violation in any execution whose tickets stay ≤ K*. It is evidence, not a proof for all N.
  - Process failures are not modeled. Liveness was checked only as "from every reachable trying state, some CS entry is reachable", in the atomic model; this is deadlock-freedom, not fair-cycle starvation analysis.
- **Ghost checks:**
  - FCFS: whenever k starts its doorway, it marks every process already in the bakery as "ahead"; a violation is k entering the CS while an ahead process hasn't entered yet.
  - Assertion 2 is checked in every newly reached CS state.

### 3.2 State counts (reachable states, tickets ≤ K)

| N, K | correct | no `choosing` | numbers only, `<` | numbers only, `≤` | double-fetch max |
|---|---|---|---|---|---|
| 2,3 atomic / safe | 190 OK / 647 OK | 107 **ME✗** / 315 **ME✗** | 193 **ME✗** / 659 **ME✗** | 188 ME-ok (deadlocks) / 638 | – |
| 2,5 | 346 / 1,543 OK | 183 / 759 ✗ | 349 / 1,563 ✗ | 344 / 1,528 | (safe, K=6: 2,224 OK) |
| 2,8 | 580 / 3,592 OK | 297 / 1,785 ✗ | 583 / 3,624 ✗ | 578 / 3,568 | (safe, K=9: 4,597 OK) |
| 3,3 | 10,917 / 44,994 OK | 9,770 / 23,120 ✗ | 12,858 / 47,778 ✗ | 10,322 / 43,932 | 18,660 OK / 81,518 **✗** |
| 3,4 | 18,576 / 92,716 OK | 17,588 / 47,725 ✗ | 21,383 / 97,240 ✗ | 17,708 / 90,942 | 30,762 OK / 168,182 **✗** |
| 3,5 | 27,778 / 165,780 OK | 27,842 / 85,500 ✗ | 31,451 / 172,440 ✗ | 26,637 / 163,118 | 45,073 OK / 300,988 **✗** |

**Additional runs**
- correct, N=3, K=6, safe: 269,550 states, OK.
- correct, N=4, K=4, atomic: 3,027,633 states, OK.
- I stopped N=4 safe (too large for Python in reasonable time).

**Liveness (atomic)**
- correct: every trying state can reach a CS entry (N=2, K=6, 424 states; N=3, K=4, 18,576 states).
- `≤` variant: a no-progress state was found (N=2 and N=3).

**FCFS + Assertion 2:** no violation.
- N=2, K=6: atomic 498 states (Assertion 2 checked in 84 CS states); safe 2,304 states (248 CS states).
- N=3, K=4: atomic 32,491 states (2,194 CS states); safe 168,742 states (8,930 CS states).
- These counts include the ghost bits.

### 3.3 Animation-ready counterexamples

Every step below was replayed against the transition relation by `scenarios.py`, so every value is exact.

**(a) No `choosing` (2 processes, 8 steps):**

| # | step | number[1], number[2] |
|---|---|---|
| 1–2 | P1 starts, reads number[2]=0 (privately plans ticket 1, not yet written) | 0, 0 |
| 3–4 | P2 starts, reads number[1]=0 | 0, 0 |
| 5 | P2 writes number[2]:=1 | 0, 1 |
| 6 | P2 checks number[1]=0 → **enters CS** | 0, 1 |
| 7 | P1 writes number[1]:=1 | 1, 1 |
| 8 | P1 checks: is (1,2) ≺ (1,1)? No → **enters CS. Both inside.** | 1, 1 |

**(a′) Same schedule with `choosing` (15 steps):**
- P2 reaches L2 and sees choosing[1]=1 → **waits** (step 7).
- P1 writes 1, lowers its flag, passes, and enters (tie broken by its lower id).
- P2 then sees (1,1) ≺ (1,2) → waits; it enters after P1 writes number[1]:=0.

**(b1) Compare numbers only, strict `<`:** both set choosing:=1, both read 0, both write 1, both clear their flags. Each sees "is 1 < 1? no" → **both enter** (12 steps).

**(b2) Compare numbers only, `≤`:** same prefix, then each sees "1 ≤ 1" → waits. In the final state the only enabled moves are self-loops: **deadlock**.

**(c) Equal tickets in the *correct* algorithm:** both raise flags, both read 0, both write 1 (6 steps). The owner's "5 → 6" example, small-number version (N=3): P3 holds 1; P1 and P2 both read max=1 and both write 2.

### 3.4 (d) Four-process walkthrough (correct algorithm; 70 verified steps)

| beat (step) | what happens | number[1..4] |
|---|---|---|
| 1–7 | P3 and P2 enter doorways together; both read only zeros | 0,0,0,0 |
| 8–12 | P3 writes 1, checks P1 (ok), then **stops at L2: choosing[2]=1** — the flag saves the day | 0,0,1,0 |
| 13–15 | P2 writes **1 (tie)** | 0,1,1,0 |
| 16–21 | P4 arrives, reads 0,1,1 → **2** | 0,1,1,2 |
| 22–23 | P3: (1,2) ≺ (1,3) → waits (tie broken by id) | |
| 24–29 | P2 passes everyone → **CS #1** | |
| 30–35 | P1 arrives (lowest id), reads 1,1,2 → **3** | 3,1,1,2 |
| 36–39 | P2 exits; P3 → **CS #2** | 3,0,1,2 |
| 40–41 | P4 checks P1: (3,1) ≺ (2,4)? **No** — the lowest id does *not* block | |
| 50–51 | P3 exits; P4 → **CS #3** | 3,0,0,2 |
| 55–60 | P2 returns, reads 3,0,2 → **4** (tickets keep climbing) | 3,4,0,2 |
| 61–62 | P4 exits; P1 → **CS #4** | 3,4,0,0 |
| 65–70 | P1 exits; P2 → **CS #5** | 0,4,0,0 |

Service order: (1,2) → (1,3) → (2,4) → (3,1) → (4,2), exactly the lexicographic order.

**Leapfrog (unbounded growth, verified):** P1 and P2 alternate so the bakery is never empty. Tickets go 1, 2, 3, 4, 5, 6, and the final state is number = [0, 6].

### 3.5 (e) Safe registers: garbage reads, still safe (verified traces, K=9)

- **e1:** P2's max-read overlaps P1's write 0→1 and returns **7**, so P2 takes **8**. P1 (ticket 1) enters; P2 waits. Garbage made P2 *later*.
- **e2:** P2's L3 read overlaps P1's *exit* write 1→0 and returns **9**, so P2 passes. Harmless: P1 is leaving.
- **e3:** P1 exits and re-enters, reading number[2]=8 and starting to write 9. P2's L3 read overlaps that write and returns **0**, so P2 **enters**. P1 finishes writing 9 and waits, because (8,2) ≺ (9,1). Garbage let P2 through, but only because P1 was certain to defer.
- **Double-fetch max (N=3, safe, 33-step BFS trace):** P1's running max t is 1. Its first fetch of number[3], which P3 is mid-writing, returns garbage 2 > t, so it re-fetches and gets **0**, and t drops to 0. P1 therefore takes ticket 1, equal to P2's, even though number[2]=1 never changed. P1 wins the tie, and P2 had already passed P1 → **both in CS**.
  - No violation in the atomic model (N=3, K ≤ 5), nor for N=2 (safe; K = 6 and 9). This matches Lamport's "for n > 2".

### 3.6 x86-TSO model (store buffers, store forwarding, MFENCE)

| fences | N=2, K=4, buffer ≤ 4 |
|---|---|
| none | **✗**, shortest trace 12 steps |
| F1 only (after choosing:=1) | ✗ (18 steps) |
| F2 only (after choosing:=0) | ✗ (20 steps) |
| F1 + F2 | OK (1,135 states). Also OK at N=2, K=6 (1,811 states) and N=3, K=3, B=3 (37,580 states) |

**The 12-step picture:** P1 performs choosing:=1, number:=1, choosing:=0 — all sitting in P1's store buffer. It reads choosing[2]=0 and number[2]=0 from memory and enters. P2 does exactly the same. At the moment both are inside, **main memory still reads choosing=[0,0], number=[0,0]**.

### 3.7 Schedule-tree counts (one attempt per process)

How these are counted: each shared read or write is a step, and a wait is one step that can be taken only when it would pass. This gives a finite tree; counts are exact via memoized DP and were cross-checked by brute force for N=2.

| program | complete schedules | bad leaves |
|---|---|---|
| lost update: `t:=x; x:=t+1` ×2 | 6 | 4 lose an update (x ends at 1, not 2) |
| naive flag lock | 14 | 8 both in CS |
| bakery without choosing, N=2 | 28 | 4 both in CS |
| correct bakery, N=2 | 322 | **0** |
| bakery without choosing, N=3 | 792,987 | 312,492 |
| correct bakery, N=3 | **2,126,816,238** | **0** |
| `<` variant with choosing, N=2 | 898 | 432 |
| `≤` variant with choosing, N=2 | 250 | 0 two-in-CS, but 72 end in deadlock |

---

## 4. The mathematical heart (narration-grade, mathematician-safe)

**Setting**
- Processes 1..N; process i alone writes choosing[i] and number[i].
- Write ≺ for lexicographic order on ℕ × {1..N}.
- Fact 0: ≺ is a strict total order. Ids are distinct, so for i ≠ k exactly one of (nᵢ, i) ≺ (nₖ, k) or (nₖ, k) ≺ (nᵢ, i) holds. (Bonus: every pair has finitely many ≺-predecessors, i.e. order type ω.)

**Definitions** (corrected at R2, 2026-10-11; R1 Ruth R5, R2 Ruth R3)
- **Doorway Dᵢ:** from the start of the write choosing[i]:=1 to the end of choosing[i]:=0. It is loop-free, so it takes a bounded number of i's steps. It contains i's ticket write.
- **Ticket held, Tᵢ:** from the end of i's ticket write number[i]:=nᵢ to the *start* of its exit write number[i]:=0. Throughout Tᵢ, number[i] = nᵢ ≥ 1, and no write to number[i] overlaps Tᵢ.
- **Bakery Bᵢ:** from the end of Dᵢ to the *start* of the exit write number[i]:=0. Bᵢ ⊂ Tᵢ, and it contains i's waiting loop and its CS. "i is in the bakery" means "the current time lies in Bᵢ"; both lemmas use only this definition. (An exit write that has begun is not "in the bakery": during it a safe read of number[i] may return anything.)

**Assumption M** (Lamport 1983): if number[j] is not written during i's max computation, then i chooses a value > number[j]. It is met by reading each number[j] once.

**Lemma 1 (doorway precedence).** If every read k makes of number[i] while choosing its current ticket lies inside Tᵢ, then nᵢ < nₖ. In particular this holds if Dᵢ ended before Dₖ began and i has not begun its exit write by the end of Dₖ (for example, i is still in the bakery then).
*Proof.* Those reads overlap no write and return nᵢ, so by M, nₖ ≥ nᵢ + 1.

**Lemma 2 (who is in front).** If i is in the CS and k ≠ i is in the bakery, then (nᵢ, i) ≺ (nₖ, k).
*Proof.* Let r₂ be i's last read of choosing[k], which returned 0, and r₃ its later last read of number[k]. Let Dₖ be k's current doorway (k is in Bₖ, so Dₖ has ended). Split on Dₖ's opening write choosing[k]:=1:
- **(a)** The opening write ended after r₂ began. All of k's reads of number[i] in Dₖ come after the opening write, hence after r₂ began, hence after i's ticket write ended (that write precedes i's waiting loop). They also end before now, and i, being in the CS now, has not begun its exit write. So they lie inside Tᵢ, and by Lemma 1, nᵢ < nₖ. *The value i read in r₃ is irrelevant.*
- **(b)** The opening write ended before r₂ began. Then **Dₖ's closing write began before r₂ ended**: if r₂ overlapped no write, it read the committed value, which is 0 only after the closing write has ended; if r₂ overlapped a write, that write is the closing one, since the opening one was over. k's ticket write precedes its closing write, so it ended before r₂ ended, hence before r₃ began; and k is in Bₖ now, so its exit write has not begun. So r₃ lies inside Tₖ, overlapped no write, and returned the true nₖ ≥ 1. i left L3, so ¬((nₖ, k) ≺ (nᵢ, i)), and by Fact 0, (nᵢ, i) ≺ (nₖ, k).
- The split is exhaustive: the opening write either ended before r₂ began or not. A flag read that overlaps the opening write (mid-raise) lands in (a); one that overlaps the closing write (mid-lowering) lands in (b). ∎

**Theorem (mutual exclusion).** If i and k were both in the CS, Lemma 2 applied both ways gives (nᵢ, i) ≺ (nₖ, k) ≺ (nᵢ, i), contradicting asymmetry. ∎

**Why garbage is harmless** (Lamport's observation, made visible): the proof uses the values of exactly two kinds of ticket read, k's max reads of number[i] in Lemma 1 and i's read r₃ in case (b), and both lie inside the reader's target's Tᵢ or Tₖ, where no write overlaps them. In case (a), whatever i read in r₃, *k* did the work. The proof never looks at an overlapped ticket value. It does use one flag read, r₂, which may overlap a write; the split above sends both possible overlaps to a case that works.

**FCFS.** If Dᵢ ends before Dₖ begins, k does not enter the CS before i.
*Proof.* Suppose k enters the CS while i has not yet entered. Then i is in Bᵢ (it has finished its doorway and not entered), so Lemma 2 (with k in the CS) gives (nₖ, k) ≺ (nᵢ, i). And k's doorway lies inside Tᵢ (it began after Dᵢ ended, and it ended before k entered, while i was still waiting), so Lemma 1 gives nᵢ < nₖ. Contradiction.

**Corollary (bounded bypass, counted from the ticket write).** Once i's ticket write has ended, each other process k enters the CS at most once before i does.
*Proof.* Suppose k enters twice while i waits. Between the entries k exits and runs a new doorway Dₖ′, which begins after k's first entry, so after i's ticket write ended; and i has not entered, so it has not begun its exit write. So Dₖ′'s reads of number[i] lie inside Tᵢ and Lemma 1 gives nᵢ < nₖ′. For k's second entry, k's last check of choosing[i] returned 0, so i's flag was down or coming down (i's ticket is already written, so its flag cannot be "not yet raised"); either way k's later read of number[i] lies inside Tᵢ and returns nᵢ, and k would pass L3 only if (nₖ′, k) ≺ (nᵢ, i), which is false. Contradiction. (This is what 11_2 says; model check `C-X5.bypass_from_ticket_write.*.worst` = 1.)

**Deadlock-freedom** (Assertion 3). Suppose some process is trying, none is in the CS, and failures are finite. Doorways are bounded, so L2 waits end. The ≺-least process in the bakery passes every L3 check, because newcomers draw larger tickets (Lemma 1).
- **Starvation-freedom:** FCFS + deadlock-freedom ⇒ lockout-freedom (Lamport 1986, Thm 1, mutual2.pdf p. 7). FCFS alone is not enough: "FCFS does not imply deadlock freedom" (same page); the `≤` variant has the same doorway, so the FCFS argument above goes through for it, yet it deadlocks (`C-F5.le_with_flag.orders_deadlocked`, `C-X3`). It assumes every process in the CS eventually leaves and processes keep taking steps.

**One-line version for the 14-year-old:** "Either you picked your ticket after you could see mine, so yours is bigger; or you'd already finished picking when I looked, so I saw your real ticket, and mine was smaller."

---

## 5. Narrative arc [P] (10 chapters, about 12.5–13.5 minutes)

**Ch1 "Two plus one is one" (~65 s)**
- **Core idea:** a "single" step is really a read and then a write.
- **Picture:** a scoreboard reading x=41; two worker lanes. "x = x+1" splits into READ and WRITE tokens that drop into one shared timeline.
- **Example:** A reads 41, B reads 41, A writes 42, B writes 42 → 42, not 43. Then fan out all 6 orders: 2 green, 4 red.
- **Misconceptions:** "rare means safe" (rare × 10⁹ per second = constant); "it's one instruction" (an ordinary increment isn't indivisible across cores).
- **Jargon:** define *process*, *interleaving*; name *race* only after showing it.
- **Personas:** 14 — scoreboard; dev — `count++`; mathematician — shuffles of two words, C(4,2)=6.

**Ch2 "The sign on the door" (~80 s)**
- **Core idea:** the critical section, and why check-then-set fails.
- **Picture:** a VACANT/OCCUPIED sign; both people read VACANT.
- **Example:** 14 orders, 8 bad. State Dijkstra's rules (from Lamport 1974): at most one inside; everyone eventually gets in; you may halt outside; no speed assumptions.
- **Stakes:** hardware's answer is test-and-set / CAS (check and set in one indivisible step). Dijkstra showed reads and writes suffice *if each is indivisible*. Lamport asks: what if they aren't?
- **Misconceptions:** "check twice"; "wait a random time" (violates no speed assumptions).
- **Jargon:** critical section, mutual exclusion, *indivisible (atomic) step*, defined carefully once and never used loosely afterwards.

**Ch3 "Take a number — there is no machine" (~70 s)**
- **Picture:** wall of personal chalk slates; a ghosted ticket dispenser with an ✗; Lamport's "ticket-machine… never halts" quote.
- **Example:** slates [0,3,0,0,2]; P4 reads 0,3,0,2 → writes 4; serving order P5, P2, P4.
- **Misconceptions:** a shared counter exists (no); the max is instant (no: N−1 separate reads — foreshadowing).
- **Jargon:** *register* (a slate), *single-writer*.
- **Personas:** dev — a ticket lock is the hardware ticket machine (fetch-and-add); mathematician — no write contention at all.

**Ch4 "Crack #1: same number" (~80 s)**
- **Example:** owner's beat, "both see 5, both take 6". Verified traces b1 (both in) and b2 (deadlock).
- **Fix:** (ticket, id).
- **Picture:** the lexicographic grid with a sweeping scanline.
- **Misconception:** "low id = priority" — show (3,1) coming after (2,4).
- **Personas:** mathematician — strict total order; dev — tuple compare, plus a one-second aside on the JS comma-operator trap.

**Ch5 "Crack #2: the unwritten ticket" (~85 s)**
- **Example:** trace (a) in 8 steps, then the scheduler slider re-runs the *same* schedule with the flag (a′).
- **Tree:** 28 leaves with 4 red → 322 leaves, 0 red.
- **Define:** *doorway*, *busy-wait*.
- **Misconceptions:** the flag makes choosing atomic or is a lock (no: two can choose at once — Ch4); it fixes torn reads (no — Ch8).

**Ch6 "The whole thing, one run" (~85 s)**
- **Picture:** code panel (Lamport's L1/L2/L3, with an explicit lexicographic compare) beside the bakery scene and a live `choosing`/`number` table.
- **Example:** walkthrough (d) beats.
- **Misconceptions:** L3 does not restart L2; you must check *all* j.

**Ch7 "Why two can never be inside" (~85 s)**
- **Picture:** a space-time diagram; k's doorway interval slides along the axis while i's L2 read tick stays fixed; zones (a), (b), and the forbidden middle (where choosing[k] would read 1).
- **On screen:** Lemma 1, Lemma 2, and the Theorem as stated in §4.
- **Misconception:** "the model checker proved it" — it only covered bounded instances; the proof covers all.

**Ch8 "Reading garbage, safely" (~85 s)**
- **Picture:** Lamport's Fig. 5 (5, 6, 27) in safe/regular/atomic columns; then slates mid-write flicker through random digits.
- **Example:** e1 (garbage → later ticket) and e3 (garbage → passes, but the other process defers); Lamport's 7456 quote; the Holt anecdote; the model-check tally (safe registers, N ≤ 3, 269,550 states at K=6, zero violations).
- **Pro footnote:** Lamport's own 1974 single-bit argument was fallacious, though the algorithm stands. The max must read each slot once (double-fetch trace).

**Ch9 "Fair, at a price" (~75 s)**
- FCFS relative to the doorway; bounded bypass; leapfrog odometer (tickets 1…6).
- Arithmetic: under 2^35 per year at 1/ms; about 585 years for 64-bit at 1/ns, with atomic reads.
- Costs: 2N registers (Burns–Lynch says ≥ N are necessary); about 2(N−1) reads even when uncontended; spinning.

**Ch10 "Your CPU isn't Lamport's memory" + legacy (~85 s)**
- **Picture:** store-buffer x-ray; memory stays all zeros while both cores enter.
- **Fixes:** two fences per process, or sequentially consistent atomics (C/C++/Rust); in practice, use CAS / fetch-add locks.
- **What the bakery teaches:** single-writer state; ordering by (counter, id) → Lamport clocks → Paxos ballots (Lamport, CHM); still shipping in Arm TF-A.
- **Close:** "discovered, not invented", then "proudest".

---

## 6. Set pieces [P]

1. **Bakery of slates:** glowing tickets that ignite as values commit. The *absence* of a dispenser is the visual joke.
2. **Space-time lanes:** one horizontal lane per process; doorway (amber), bakery wait (blue), CS (red). Reads are vertical "sampling" arrows into another lane. When an arrow lands inside a write interval, the received digit flickers (Lamport's 7456), and the proof's zones (a)/(b) shade underneath.
3. **Interleaving explorer:** a tree that grows from the root, one step per level. The no-flag tree (28 leaves) lights 4 red; morph to the flag tree (322, all green); then zoom to N=3, where the counters spin to 2,126,816,238 / 0.
4. **Scheduler slider:** scrubs one schedule; a toggle switches `choosing` on and off mid-scene (a ↔ a′).
5. **Lexicographic grid:** x = ticket, y = id. The scanline sweeps by columns, bottom-up within each column; occupied cells light in serving order. A late P1 cell at (3,1) visibly gets swept *after* (2,4).
6. **Store-buffer x-ray:** per-core FIFO trays; memory bar frozen at 0; "MFENCE" drains a tray with a whoosh.
7. **Lamport's register zoo:** the three-column Fig. 5 comparison.

---

## 7. Claims ledger (every number on screen)

| Claim | Verification |
|---|---|
| 6 orders, 4 lose an update | enumeration (`tree_count.py`) |
| naive flag: 14 / 8 | DP plus independent brute force |
| no-flag N=2: 28 / 4; correct: 322 / 0 | DP plus brute force (`crosscheck.py`) |
| N=3: 792,987 / 312,492; correct 2,126,816,238 / 0 | memoized DP (one-shot semantics) |
| traces a, a′, b1, b2, c, d, e1–e3, leapfrog | replayed step by step (`scenarios.py`, `leapfrog.py`) |
| state-count table, FCFS, Assertion 2, liveness | `run_sweep.py` outputs |
| TSO: 12-step break; F1+F2 OK | `bakery_tso.py`, `tso3.py`; consistent with Abdulla et al. 2012 |
| double-fetch breaks (N=3 safe), not N=2 | `run_sweep.py`, `df_trace.py`; Lamport 1983 |
| 2^35 / year | Lamport 1974; arithmetic 31,557,600,000 < 34,359,738,368 |
| ~585 years for 2^64 at 10⁹/s | computed: 584.5 |
| 7456, 2496, Holt, "proudest", "discovered", "of little if any practical use", "ticket-machine" | verbatim from the publications page, Turing lecture PDF, and CHM PDF |
| 1974 / CACM 17(8):453–455 / "Received September 1973" | paper PDF |
| n-register lower bound; consensus numbers | Burns–Lynch 1993; Herlihy 1991 (sub-agent-verified) |
| JS comma bug | node: `(t[j], j) < (t[i], i)` → `true` where lexicographic → `false`; tsc 5.9.3 silent |

---

## 8. Pitfalls a professional would flag (with correct phrasing)

- **"Atomic" overloaded.** Separate *atomic instruction* (read-modify-write: CAS, test-and-set), *atomic register* (linearizable reads and writes), and "the doorway". The bakery needs **no** read-modify-write and **not even** atomic registers. Safe, single-writer registers suffice. The doorway is not atomic.
- **"Unique tickets."** Wrong. Say "unique (ticket, id) pairs".
- **"Lower id has priority."** Only on ties.
- **FCFS vs starvation-freedom.** FCFS is about doorway completion. Starvation-freedom follows from FCFS plus deadlock-freedom, assuming CS holders leave and processes don't crash and restart forever (Lamport's caveat).
- **"Works on modern hardware."** No. x86 allows store→load reordering, and ARM/POWER are weaker. Say: "requires fences, or sequentially consistent atomics." In C/C++/Rust, plain shared variables in a data race are undefined behavior and compilers may hoist spin loads; C/C++ `volatile` is not a fix.
- **Ticket overflow.** Tickets are unbounded in principle. They are practically bounded only if reads can't return too-large values (atomic or regular). Bounded variants exist (Taubenfeld 2004 — atomic registers; Aravind–Hesselink 2011 — nonatomic).
- **"Only makes you wait longer."** Garbage can let you pass, but safely (e3).
- **"First solution."** First *without lower-level mutual exclusion*. Not first overall: Dekker, Dijkstra 1965, Knuth 1966, de Bruijn 1967, and Eisenberg–McGuire 1972 came earlier.
- **Max implementation.** Read each `number[k]` once (Lamport 1983).
- **Costs.** About 2(N−1) reads even when uncontended; N must be known in advance; spinning. This is why practice uses CAS / fetch-add locks (a ticket lock is the "ticket machine" Lamport avoided).
- **"Consensus."** Don't use it loosely. Read/write registers can't do wait-free consensus, yet can do (blocking) mutual exclusion.
- **Hardware honesty.** Even a safe register needs an arbiter, which can't have bounded response time (Lamport's "glitch" papers). Footnote at most.

---

## Appendix A — model-checker source (Python 3, no dependencies)

### A.1 `bakery_mc.py` (core explorer: atomic and safe registers, variants, FCFS ghost, liveness, replay)

```python
#!/usr/bin/env python3
"""
bakery_mc.py -- exhaustive interleaving explorer for Lamport's Bakery algorithm
and broken variants.  Pure Python 3, no dependencies.

Process ids are 1..N (as in Lamport 1974); internally 0..N-1.
Each shared register choosing[i], number[i] is written only by process i.

Program for process i (Lamport, CACM 17(8):453-455, 1974):
  L1: choosing[i] := 1
      number[i]   := 1 + max(number[1..N])    -- each number[k] read separately,
                                                 in any order
      choosing[i] := 0
      for j := 1..N (j != i):
        L2: while choosing[j] != 0: skip
        L3: while number[j] != 0 and (number[j], j) < (number[i], i): skip
      CRITICAL SECTION
      number[i] := 0
      (back to non-critical section)

Register semantics
  atomic : every read and every write is one indivisible step.
  safe   : every WRITE is two steps (begin, end).  A read performed while the
           register is being written may return ANY value of the register's
           domain (number: 0..K, choosing: {0,1}).  A read that does not overlap a
           write returns the last committed value.  Modelling reads as points and
           writes as intervals is equivalent to Lamport's safe-register definition
           for single-writer registers: a read interval overlaps a write iff some
           point inside it does, and the reader does nothing else during its read.

Bounding: tickets are capped at K.  A transition that would write a ticket > K
is PRUNED (and counted).  "Verified" therefore means: for every execution in
which no ticket ever exceeds K.

Variants
  correct      : Lamport's algorithm
  no_choosing  : delete choosing[] and the L2 wait
  strict_nums  : L3 compares numbers only: wait while number[j]!=0 and number[j] <  number[i]
  le_nums      : L3 compares numbers only: wait while number[j]!=0 and number[j] <= number[i]
  double_fetch : correct, except max is computed as
                     t := 0; for k: if number[k] > t then t := number[k]
                 where the two occurrences of number[k] are SEPARATE reads.
                 (Lamport, "Reasoning About Nonatomic Operations", POPL 1983:
                  "not a valid implementation" for n > 2.)
"""
from collections import deque

NCS, RD, RD2, WN, WC0, L2, L3, CS = range(8)
PCNAME = {NCS: "ncs", RD: "max-read", RD2: "max-refetch", WN: "write-num",
          WC0: "choose:=0", L2: "L2", L3: "L3", CS: "CS"}


class Bakery:
    def __init__(self, N, K, variant="correct", safe=False, any_order=True, fcfs=False):
        assert variant in ("correct", "no_choosing", "strict_nums", "le_nums", "double_fetch")
        self.N, self.K, self.variant, self.safe = N, K, variant, safe
        self.any_order, self.fcfs = any_order, fcfs
        self.use_choosing = variant != "no_choosing"
        self.pruned_states = set()

    # ---- state layout -------------------------------------------------------
    # local[i] = (pc, a, m, wp, rest)
    #   RD : a = bitmask of indices still to read, m = max so far
    #   RD2: a = index being re-fetched, m = t so far, rest = mask still to read
    #   WN : m = max read
    #   L2/L3: a = current j
    #   wp = 1 iff a (safe-mode) write has begun but not ended
    # global = (locals, choosing, number, ahead)   ahead = FCFS ghost bits
    def initial(self):
        N = self.N
        return (tuple((NCS, 0, 0, 0, 0) for _ in range(N)), (0,) * N, (0,) * N, 0)

    def others(self, i):
        return [j for j in range(self.N) if j != i]

    def _writing_choosing(self, st, j):
        pc, a, m, wp, r = st[0][j]
        return wp == 1 and pc in (NCS, WC0)

    def _writing_number(self, st, j):
        pc, a, m, wp, r = st[0][j]
        return wp == 1 and pc in (WN, CS)

    def read_choosing(self, st, j):
        if self.safe and self._writing_choosing(st, j):
            return [0, 1]
        return [st[1][j]]

    def read_number(self, st, j):
        if self.safe and self._writing_number(st, j):
            return list(range(self.K + 1))
        return [st[2][j]]

    def must_wait(self, nj, j, ni, i):
        if nj == 0:
            return False
        if self.variant == "strict_nums":
            return nj < ni
        if self.variant == "le_nums":
            return nj <= ni
        return (nj, j) < (ni, i)

    def _first_wait(self, i):
        js = self.others(i)
        return (L2 if self.use_choosing else L3, js[0]) if js else (CS, 0)

    def _next_wait(self, i, j):
        js = self.others(i)
        k = js.index(j)
        if k + 1 < len(js):
            return (L2 if self.use_choosing else L3, js[k + 1])
        return (CS, 0)

    def _mark_ahead(self, st, i, ahead):
        """i begins its doorway: every process already in the bakery is ahead of i."""
        if not self.fcfs:
            return ahead
        N = self.N
        for k in self.others(i):
            if st[0][k][0] in (L2, L3):
                ahead |= 1 << (k * N + i)
        return ahead

    def successors(self, st):
        loc, ch, nu, ahead = st
        N, K = self.N, self.K
        out = []

        def mk(i, newl, ch2=ch, nu2=nu, ah2=ahead):
            l2 = list(loc)
            l2[i] = newl
            return (tuple(l2), ch2, nu2, ah2)

        def setv(t, i, v):
            t2 = list(t)
            t2[i] = v
            return tuple(t2)

        for i in range(N):
            pc, a, m, wp, rest = loc[i]
            allmask = sum(1 << j for j in self.others(i))
            if pc == NCS:
                if self.use_choosing:
                    if self.safe and not wp:
                        out.append((i, "begin choosing:=1",
                                    mk(i, (NCS, 0, 0, 1, 0), ah2=self._mark_ahead(st, i, ahead))))
                    else:
                        ah = ahead if self.safe else self._mark_ahead(st, i, ahead)
                        lab = "end choosing:=1" if self.safe else "choosing:=1"
                        out.append((i, lab, mk(i, (RD, allmask, 0, 0, 0), ch2=setv(ch, i, 1), ah2=ah)))
                else:
                    out.append((i, "start", mk(i, (RD, allmask, 0, 0, 0),
                                                 ah2=self._mark_ahead(st, i, ahead))))
            elif pc == RD:
                if a == 0:
                    out.append((i, f"max={m}", mk(i, (WN, 0, m, 0, 0))))
                    continue
                idxs = [j for j in range(N) if a >> j & 1]
                if not self.any_order:
                    idxs = idxs[:1]
                for j in idxs:
                    for v in self.read_number(st, j):
                        if self.variant == "double_fetch" and v > m:
                            out.append((i, f"read number[{j+1}]={v} (> t: re-fetch)",
                                        mk(i, (RD2, j, m, 0, a & ~(1 << j)))))
                        else:
                            rem = a & ~(1 << j)
                            nm = max(m, v)
                            # last read of the max loop goes straight to "write number"
                            nl = (RD, rem, nm, 0, 0) if rem else (WN, 0, nm, 0, 0)
                            out.append((i, f"read number[{j+1}]={v}", mk(i, nl)))
            elif pc == RD2:
                for v in self.read_number(st, a):
                    nl = (RD, rest, v, 0, 0) if rest else (WN, 0, v, 0, 0)
                    out.append((i, f"re-fetch number[{a+1}]={v} -> t", mk(i, nl)))
            elif pc == WN:
                t = m + 1
                if t > K:
                    self.pruned_states.add(st)
                    continue
                if self.safe and not wp:
                    out.append((i, f"begin number[{i+1}]:={t}", mk(i, (WN, 0, m, 1, 0))))
                else:
                    lab = f"{'end ' if self.safe else ''}number[{i+1}]:={t}"
                    nu2 = setv(nu, i, t)
                    if self.use_choosing:
                        out.append((i, lab, mk(i, (WC0, 0, 0, 0, 0), nu2=nu2)))
                    else:
                        p2, j2 = self._first_wait(i)
                        out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), nu2=nu2)))
            elif pc == WC0:
                if self.safe and not wp:
                    out.append((i, "begin choosing:=0", mk(i, (WC0, 0, 0, 1, 0))))
                else:
                    p2, j2 = self._first_wait(i)
                    lab = "end choosing:=0" if self.safe else "choosing:=0"
                    out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), ch2=setv(ch, i, 0))))
            elif pc == L2:
                j = a
                for v in sorted(set(self.read_choosing(st, j))):
                    if v != 0:
                        out.append((i, f"L2: choosing[{j+1}]={v}, wait", st))
                    else:
                        out.append((i, f"L2: choosing[{j+1}]=0, pass", mk(i, (L3, j, 0, 0, 0))))
            elif pc == L3:
                j = a
                ni = nu[i]
                for v in sorted(set(self.read_number(st, j))):
                    if self.must_wait(v, j, ni, i):
                        out.append((i, f"L3: number[{j+1}]={v}, wait", st))
                    else:
                        p2, j2 = self._next_wait(i, j)
                        lab = f"L3: number[{j+1}]={v}, pass"
                        ah = ahead
                        if p2 == CS:
                            lab += " -> ENTER CS"
                        out.append((i, lab, mk(i, (p2, j2, 0, 0, 0), ah2=ah)))
            elif pc == CS:
                if self.safe and not wp:
                    out.append((i, f"begin number[{i+1}]:=0 (exit)", mk(i, (CS, 0, 0, 1, 0))))
                else:
                    lab = f"{'end ' if self.safe else ''}number[{i+1}]:=0 (exit)"
                    out.append((i, lab, mk(i, (NCS, 0, 0, 0, 0), nu2=setv(nu, i, 0))))
        return out

    def in_cs(self, st):
        return [i for i in range(self.N) if st[0][i][0] == CS and st[0][i][3] == 0]

    def fcfs_check(self, prev, st, i, lab):
        """Returns (violation or None, new ahead bits) when process i enters CS."""
        N = self.N
        ah = st[3]
        if "ENTER CS" not in lab:
            return None, ah
        viol = None
        for k in range(N):
            if k != i and (ah >> (k * N + i)) & 1:
                viol = (k, i)
        for k in range(N):           # i is no longer ahead of anybody
            ah &= ~(1 << (i * N + k))
        return viol, ah


def explore(model, check_assert2=False, liveness=False):
    init = model.initial()
    index = {init: 0}
    states = [init]
    parent = [None]
    edges = [] if liveness else None
    entries = set()
    q = deque([0])
    bad = bad_kind = None
    a2_checked = 0
    while q:
        sid = q.popleft()
        st = states[sid]
        succ_ids = []
        for (i, lab, ns) in model.successors(st):
            if model.fcfs:
                viol, ah = model.fcfs_check(st, ns, i, lab)
                ns = ns[:3] + (ah,)
            else:
                viol = None
            new = ns not in index
            if new:
                index[ns] = len(states)
                states.append(ns)
                parent.append((sid, i, lab))
                q.append(index[ns])
            nid = index[ns]
            succ_ids.append(nid)
            if "ENTER CS" in lab:
                entries.add(sid)
            if bad is None:
                cs = model.in_cs(ns)
                if len(cs) >= 2:
                    bad, bad_kind = nid, "MUTUAL EXCLUSION VIOLATED"
                elif viol:
                    bad, bad_kind = nid, f"FCFS VIOLATED (P{viol[0]+1} was ahead of P{viol[1]+1})"
                elif check_assert2 and new and cs:
                    a2_checked += 1
                    c = cs[0]
                    for k in model.others(c):
                        if ns[0][k][0] in (L2, L3) and not ((ns[2][c], c) < (ns[2][k], k)):
                            bad, bad_kind = nid, "ASSERTION 2 VIOLATED"
        if liveness:
            edges.append(succ_ids)
    res = {"states": len(states), "bad": bad, "bad_kind": bad_kind,
           "pruned": len(model.pruned_states), "a2_checked": a2_checked}
    if bad is not None:
        res["trace"] = _trace(parent, states, bad, model)
    if liveness:
        n = len(states)
        rev = [[] for _ in range(n)]
        for s in range(n):
            for t in edges[s]:
                rev[t].append(s)
        good = [False] * n
        dq = deque()
        for s in range(n):
            if s in entries or states[s] in model.pruned_states:
                good[s] = True
                dq.append(s)
        while dq:
            t = dq.popleft()
            for s in rev[t]:
                if not good[s]:
                    good[s] = True
                    dq.append(s)
        stuck = None
        for s in range(n):
            st = states[s]
            trying = any(st[0][i][0] not in (NCS, CS) for i in range(model.N))
            if trying and not model.in_cs(st) and not good[s]:
                stuck = s
                break
        res["stuck"] = stuck
        if stuck is not None:
            res["stuck_trace"] = _trace(parent, states, stuck, model)
    return res


def fmt_state(model, st):
    parts = []
    for i in range(model.N):
        pc, a, m, wp, rest = st[0][i]
        s = PCNAME[pc]
        if pc in (L2, L3):
            s += f"(j={a+1})"
        if pc in (RD, WN, RD2):
            s += f"(m={m})"
        if wp:
            s += "[writing]"
        parts.append(f"P{i+1}:{s}")
    return " | ".join(parts) + f" || choosing={list(st[1])} number={list(st[2])}"


def _trace(parent, states, sid, model):
    steps = []
    while parent[sid] is not None:
        p, i, lab = parent[sid]
        steps.append((i, lab, states[sid]))
        sid = p
    steps.reverse()
    return [(f"P{i+1}: {lab}", fmt_state(model, st)) for (i, lab, st) in steps]


def replay(model, schedule, verbose=True):
    """schedule: list of (proc_1based, label_substring); each step must match exactly
    one enabled transition of that process.  Raises if the step is illegal."""
    st = model.initial()
    rows = []
    for (p, sub) in schedule:
        i = p - 1
        en = [(lab, ns) for (pi, lab, ns) in model.successors(st) if pi == i]
        cands = [(lab, ns) for (lab, ns) in en if sub in lab]
        if len(cands) != 1:
            raise RuntimeError(f"step P{p} '{sub}': {len(cands)} matches; enabled={[l for l, _ in en]}")
        lab, st = cands[0]
        rows.append((f"P{p}: {lab}", fmt_state(model, st), model.in_cs(st)))
    if verbose:
        for k, (a, b, cs) in enumerate(rows, 1):
            flag = "   <== TWO PROCESSES IN CS" if len(cs) >= 2 else ""
            print(f"  {k:2d}. {a:40s} {b}{flag}")
    return st, rows
```

### A.2 `run_sweep.py`

```python
#!/usr/bin/env python3
"""Exhaustive sweeps: every variant x {atomic, safe} x N x K; plus liveness, FCFS and
Assertion-2 checks for the correct algorithm.  Prints one line per run."""
import sys, time
from bakery_mc import Bakery, explore

def run(N, K, var, safe, **kw):
    t = time.time()
    m = Bakery(N, K, var, safe=safe, fcfs=kw.pop("fcfs", False))
    r = explore(m, **kw)
    line = (f"N={N} K={K} {var:13s} {'safe  ' if safe else 'atomic'} states={r['states']:>9,} "
            f"pruned={r['pruned']:>7,} result={r['bad_kind'] or 'OK (no violation)'}")
    if "stuck" in r:
        line += f" | liveness: {'NO-PROGRESS STATE FOUND' if r['stuck'] is not None else 'every trying state can reach a CS entry'}"
    if r["a2_checked"]:
        line += f" | Assertion2 checked in {r['a2_checked']:,} CS states"
    line += f"  ({time.time()-t:.1f}s)"
    print(line, flush=True)
    return r

if __name__ == "__main__":
    which = sys.argv[1] if len(sys.argv) > 1 else "all"
    if which in ("all", "safety"):
        print("== SAFETY (mutual exclusion) sweep, any-order max reads ==")
        for N, Ks in ((2, (3, 5, 8)), (3, (3, 4, 5))):
            for K in Ks:
                for var in ("correct", "no_choosing", "strict_nums", "le_nums", "double_fetch"):
                    if N == 2 and var == "double_fetch":
                        continue
                    for safe in (False, True):
                        run(N, K, var, safe)
    if which in ("all", "liveness"):
        print("== LIVENESS (no reachable trying state without a path to a CS entry; atomic) ==")
        for N, K in ((2, 6), (3, 4)):
            for var in ("correct", "le_nums"):
                run(N, K, var, False, liveness=True)
    if which in ("all", "fcfs"):
        print("== FCFS ghost check + Assertion 2 (correct algorithm) ==")
        for N, K, safe in ((2, 6, False), (2, 6, True), (3, 4, False), (3, 4, True)):
            run(N, K, "correct", safe, fcfs=True, check_assert2=True)
    if which == "big":
        print("== larger runs ==")
        run(3, 6, "correct", True)
        run(4, 4, "correct", False)
        run(4, 3, "correct", True)   # stopped manually: too large for Python here
```

**Extra one-off checks:**
- `df_trace.py`: `explore(Bakery(2,K,"double_fetch",safe=True))` for K = 6 and 9, plus the shortest N=3 safe trace with `any_order=False`.
- `crosscheck.py`: independent brute-force DFS for the N=2 schedule counts.

### A.3 `scenarios.py` (step-validated traces) and `leapfrog.py`

```python
#!/usr/bin/env python3
"""Scripted, step-by-step scenarios for animation.  Every step is validated by
replay() against the model's transition relation (an illegal step raises)."""
from bakery_mc import Bakery, replay, explore

def hdr(s):
    print("\n" + "=" * 100 + "\n" + s + "\n" + "=" * 100)

hdr("(a) NO choosing[] : classic 2-process mutual-exclusion violation")
A = [(1, "start"), (1, "read number[2]=0"),
     (2, "start"), (2, "read number[1]=0"), (2, "number[2]:=1"),
     (2, "L3: number[1]=0, pass"),
     (1, "number[1]:=1"), (1, "L3: number[2]=1, pass")]
replay(Bakery(2, 5, "no_choosing", any_order=False), A)

hdr("(a') SAME schedule WITH choosing[] (correct algorithm): P2 is stopped at L2")
A2 = [(1, "choosing:=1"), (1, "read number[2]=0"),
      (2, "choosing:=1"), (2, "read number[1]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),
      (2, "L2: choosing[1]=1, wait"),
      (1, "number[1]:=1"), (1, "choosing:=0"),
      (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),
      (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),
      (1, "number[1]:=0"), (2, "L3: number[1]=0, pass")]
replay(Bakery(2, 5, "correct", any_order=False), A2)

hdr("(b1) numbers only, strict <  : tie -> both enter")
B = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
     (1, "number[1]:=1"), (2, "number[2]:=1"), (1, "choosing:=0"), (2, "choosing:=0"),
     (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, pass"),
     (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, pass")]
replay(Bakery(2, 5, "strict_nums", any_order=False), B)

hdr("(b2) numbers only, <= : tie -> both wait forever (deadlock)")
B2 = B[:8] + [(1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=1, wait"),
              (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait"),
              (1, "L3: number[2]=1, wait"), (2, "L3: number[1]=1, wait")]
st, _ = replay(Bakery(2, 5, "le_nums", any_order=False), B2)
m = Bakery(2, 5, "le_nums", any_order=False)
print("   enabled moves in final state:", [(f"P{i+1}", lab, ns == st) for i, lab, ns in m.successors(st)])

hdr("(c) two processes take equal tickets (normal in the real algorithm)")
C = [(1, "choosing:=1"), (2, "choosing:=1"), (1, "read number[2]=0"), (2, "read number[1]=0"),
     (1, "number[1]:=1"), (2, "number[2]:=1")]
replay(Bakery(2, 5, "correct", any_order=False), C)
hdr("(c') N=3: P3 holds ticket 1; P1 and P2 both read max=1 and both take 2")
C3 = [(3, "choosing:=1"), (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "number[3]:=1"),
      (3, "choosing:=0"), (1, "choosing:=1"), (2, "choosing:=1"),
      (1, "read number[2]=0"), (2, "read number[1]=0"),
      (1, "read number[3]=1"), (2, "read number[3]=1"),
      (1, "number[1]:=2"), (2, "number[2]:=2")]
replay(Bakery(3, 5, "correct", any_order=False), C3)

hdr("(d) correct algorithm, N=4")
D = [
    (3, "choosing:=1"), (2, "choosing:=1"),
    (3, "read number[1]=0"), (3, "read number[2]=0"), (3, "read number[4]=0"),
    (2, "read number[1]=0"), (2, "read number[3]=0"),
    (3, "number[3]:=1"), (3, "choosing:=0"),
    (3, "L2: choosing[1]=0, pass"), (3, "L3: number[1]=0, pass"),
    (3, "L2: choosing[2]=1, wait"),
    (2, "read number[4]=0"), (2, "number[2]:=1"), (2, "choosing:=0"),
    (4, "choosing:=1"), (4, "read number[1]=0"), (4, "read number[2]=1"), (4, "read number[3]=1"),
    (4, "number[4]:=2"), (4, "choosing:=0"),
    (3, "L2: choosing[2]=0, pass"), (3, "L3: number[2]=1, wait"),
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=0, pass"),
    (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=1, pass"),
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=2, pass"),
    (1, "choosing:=1"), (1, "read number[2]=1"), (1, "read number[3]=1"), (1, "read number[4]=2"),
    (1, "number[1]:=3"), (1, "choosing:=0"),
    (2, "number[2]:=0"),
    (3, "L3: number[2]=0, pass"), (3, "L2: choosing[4]=0, pass"), (3, "L3: number[4]=2, pass"),
    (4, "L2: choosing[1]=0, pass"), (4, "L3: number[1]=3, pass"),
    (4, "L2: choosing[2]=0, pass"), (4, "L3: number[2]=0, pass"),
    (4, "L2: choosing[3]=0, pass"), (4, "L3: number[3]=1, wait"),
    (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=0, pass"),
    (1, "L2: choosing[3]=0, pass"), (1, "L3: number[3]=1, wait"),
    (3, "number[3]:=0"), (4, "L3: number[3]=0, pass"),
    (1, "L3: number[3]=0, pass"), (1, "L2: choosing[4]=0, pass"), (1, "L3: number[4]=2, wait"),
    (2, "choosing:=1"), (2, "read number[1]=3"), (2, "read number[3]=0"), (2, "read number[4]=2"),
    (2, "number[2]:=4"), (2, "choosing:=0"),
    (4, "number[4]:=0"), (1, "L3: number[4]=0, pass"),
    (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=3, wait"),
    (1, "number[1]:=0"),
    (2, "L3: number[1]=0, pass"), (2, "L2: choosing[3]=0, pass"), (2, "L3: number[3]=0, pass"),
    (2, "L2: choosing[4]=0, pass"), (2, "L3: number[4]=0, pass"),
]
replay(Bakery(4, 6, "correct", any_order=True), D)

hdr("(e1) SAFE registers: garbage 7 during P1's write -> P2 takes 8; harmless")
E = [(1, "begin choosing:=1"), (1, "end choosing:=1"), (1, "read number[2]=0"),
     (1, "begin number[1]:=1"), (2, "begin choosing:=1"), (2, "end choosing:=1"),
     (2, "read number[1]=7"),
     (1, "end number[1]:=1"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
     (2, "begin number[2]:=8"), (2, "end number[2]:=8"), (2, "begin choosing:=0"), (2, "end choosing:=0"),
     (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, pass"),
     (2, "L2: choosing[1]=0, pass"), (2, "L3: number[1]=1, wait")]
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E)
hdr("(e2) SAFE: P2's L3 read overlaps P1's exit write and returns garbage 9")
E2 = E + [(1, "begin number[1]:=0"), (2, "L3: number[1]=9, pass"), (1, "end number[1]:=0")]
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E2)
hdr("(e3) SAFE: garbage 0 lets P2 in; P1's ticket is necessarily larger, so P1 waits")
E3 = E + [(1, "begin number[1]:=0"), (1, "end number[1]:=0"),
          (1, "begin choosing:=1"), (1, "end choosing:=1"),
          (1, "read number[2]=8"), (1, "begin number[1]:=9"),
          (2, "L3: number[1]=0, pass"),
          (1, "end number[1]:=9"), (1, "begin choosing:=0"), (1, "end choosing:=0"),
          (1, "L2: choosing[2]=0, pass"), (1, "L3: number[2]=8, wait")]
replay(Bakery(2, 9, "correct", safe=True, any_order=False), E3)
```

```python
# leapfrog.py -- two processes never let the bakery empty: tickets climb 1,2,3,4,5,6
from bakery_mc import Bakery, replay
S = [(1,"choosing:=1"),(1,"read number[2]=0"),(1,"number[1]:=1"),(1,"choosing:=0"),
     (1,"L2: choosing[2]=0, pass"),(1,"L3: number[2]=0, pass")]
for r in range(5):
    a, b = (2, 1) if r % 2 == 0 else (1, 2)
    nb = 1 + r; na = nb + 1
    S += [(a,"choosing:=1"),(a,f"read number[{b}]={nb}"),(a,f"number[{a}]:={na}"),(a,"choosing:=0"),
          (a,f"L2: choosing[{b}]=0, pass"),(a,f"L3: number[{b}]={nb}, wait"),
          (b,f"number[{b}]:=0"),(a,f"L3: number[{b}]=0, pass")]
st, rows = replay(Bakery(2, 7, "correct", any_order=False), S, verbose=False)
print("final state:", rows[-1][1])   # P1:ncs | P2:CS || choosing=[0, 0] number=[0, 6]
```

### A.4 `bakery_tso.py` (x86-TSO store-buffer model)

```python
#!/usr/bin/env python3
"""Bakery algorithm under an x86-TSO-style memory model (Owens/Sarkar/Sewell):
each process has a FIFO store buffer; a store is appended to the buffer; a load
returns the newest buffered store to that address in the process's OWN buffer if
any, else main memory; at any time a buffer may flush its oldest entry to memory;
MFENCE blocks until the process's own buffer is empty.

fences: subset of {"F1","F2"}:
  F1 = MFENCE right after  choosing[i] := 1   (before reading the other numbers)
  F2 = MFENCE right after  choosing[i] := 0   (before the L2/L3 wait loops)
Tickets capped at K (transitions writing a ticket > K are pruned); store buffers hold
at most B entries (a store waits while the buffer is full).  Both are
under-approximations: a violation found is real; "OK" means OK within the bounds."""
from collections import deque

NCS, F1, RD, WN, WC0, F2, L2, L3, CS = range(9)
NAMES = {NCS: "ncs", F1: "mfence#1", RD: "max-read", WN: "write-num", WC0: "choose:=0",
         F2: "mfence#2", L2: "L2", L3: "L3", CS: "CS"}


class TSOBakery:
    def __init__(self, N, K, fences=(), B=4):
        self.N, self.K, self.fences, self.B = N, K, set(fences), B   # B = store-buffer capacity
        self.pruned = 0

    def initial(self):
        N = self.N
        return (tuple((NCS, 0, 0) for _ in range(N)), (0,) * N, (0,) * N, tuple(() for _ in range(N)))

    def load(self, st, i, kind, j):
        for (k2, j2, v) in reversed(st[3][i]):      # store forwarding from own buffer
            if k2 == kind and j2 == j:
                return v
        return (st[1] if kind == "c" else st[2])[j]

    def successors(self, st):
        loc, ch, nu, buf = st
        N, K = self.N, self.K
        out = []

        def mk(i, newl, newbuf=None):
            l2 = list(loc); l2[i] = newl
            b2 = buf if newbuf is None else tuple(newbuf if k == i else buf[k] for k in range(N))
            return (tuple(l2), ch, nu, b2)

        for i in range(N):
            pc, a, m = loc[i]
            others = [j for j in range(N) if j != i]
            # 1) flush oldest buffered store of process i
            if buf[i]:
                (kind, j, v) = buf[i][0]
                ch2, nu2 = list(ch), list(nu)
                (ch2 if kind == "c" else nu2)[j] = v
                b2 = tuple(buf[i][1:] if k == i else buf[k] for k in range(N))
                out.append((i, f"flush {'choosing' if kind == 'c' else 'number'}[{j+1}]={v} to memory",
                            (loc, tuple(ch2), tuple(nu2), b2)))
            # 2) program step (a store is disabled while the buffer is full: bounded-buffer model)
            full = len(buf[i]) >= self.B
            if full and pc in (NCS, WN, WC0, CS):
                continue
            if pc == NCS:
                mask = sum(1 << j for j in others)
                out.append((i, "store choosing:=1 (buffered)",
                            mk(i, (F1 if "F1" in self.fences else RD, mask, 0), buf[i] + (("c", i, 1),))))
            elif pc in (F1, F2):
                if not buf[i]:
                    nxt = (RD, a, 0) if pc == F1 else (L2, others[0], 0)
                    out.append((i, "mfence", mk(i, nxt)))
            elif pc == RD:
                j = min(k for k in range(N) if a >> k & 1)
                v = self.load(st, i, "n", j)
                rem = a & ~(1 << j)
                nm = max(m, v)
                out.append((i, f"load number[{j+1}]={v}", mk(i, (RD, rem, nm) if rem else (WN, 0, nm))))
            elif pc == WN:
                t = m + 1
                if t > K:
                    self.pruned += 1
                    continue
                out.append((i, f"store number:={t} (buffered)", mk(i, (WC0, 0, 0), buf[i] + (("n", i, t),))))
            elif pc == WC0:
                nxt = (F2, 0, 0) if "F2" in self.fences else (L2, others[0], 0)
                out.append((i, "store choosing:=0 (buffered)", mk(i, nxt, buf[i] + (("c", i, 0),))))
            elif pc == L2:
                j = a
                v = self.load(st, i, "c", j)
                if v:
                    out.append((i, f"L2 load choosing[{j+1}]=1, wait", st))
                else:
                    out.append((i, f"L2 load choosing[{j+1}]=0, pass", mk(i, (L3, j, 0))))
            elif pc == L3:
                j = a
                nj = self.load(st, i, "n", j)
                ni = self.load(st, i, "n", i)
                if nj != 0 and (nj, j) < (ni, i):
                    out.append((i, f"L3 load number[{j+1}]={nj}, wait", st))
                else:
                    k = others.index(j)
                    if k + 1 < len(others):
                        out.append((i, f"L3 load number[{j+1}]={nj}, pass", mk(i, (L2, others[k + 1], 0))))
                    else:
                        out.append((i, f"L3 load number[{j+1}]={nj}, pass -> ENTER CS", mk(i, (CS, 0, 0))))
            elif pc == CS:
                out.append((i, "store number:=0 (exit, buffered)", mk(i, (NCS, 0, 0), buf[i] + (("n", i, 0),))))
        return out

    def in_cs(self, st):
        return [i for i in range(self.N) if st[0][i][0] == CS]


def explore_tso(model):
    init = model.initial()
    seen = {init: None}
    q = deque([init])
    bad = None
    while q:
        st = q.popleft()
        for (i, lab, ns) in model.successors(st):
            if ns not in seen:
                seen[ns] = (st, i, lab)
                q.append(ns)
                if bad is None and len(model.in_cs(ns)) >= 2:
                    bad = ns
    tr = []
    if bad is not None:
        s = bad
        while seen[s] is not None:
            p, i, lab = seen[s]
            tr.append((f"P{i+1}: {lab}", s))
            s = p
        tr.reverse()
    return len(seen), bad, tr


if __name__ == "__main__":
    for fences in ((), ("F1",), ("F2",), ("F1", "F2")):
        m = TSOBakery(2, 4, fences)
        n, bad, tr = explore_tso(m)
        print(f"TSO N=2 K=4 fences={'+'.join(fences) or 'none'}: states={n:,} -> "
              f"{'VIOLATED (%d steps)' % len(tr) if bad else 'OK'}")
    # tso3.py additionally ran: TSOBakery(2,6,("F1","F2"),B=4) and TSOBakery(3,3,("F1","F2"),B=3)
```

### A.5 `tree_count.py` (exact schedule-tree counts)

```python
#!/usr/bin/env python3
"""Count complete schedules (interleavings) of small ONE-SHOT programs: every process
runs its entry protocol, critical section and exit exactly once.
Busy-wait loops are modelled as blocking 'await' steps (a spin iteration only re-reads
and never writes, so for safety this is equivalent, and it makes the tree finite).
Counting uses memoised DP over (pcs, shared, locals, in-CS flags, seen-bad)."""
from functools import lru_cache


def make_bakery(N, with_choosing=True, cmp="lex"):
    def less(a, b):
        if cmp == "lex":
            return a < b
        if cmp == "strict":
            return a[0] < b[0]
        return a[0] <= b[0]          # "le"
    progs = []
    for i in range(N):
        js = [j for j in range(N) if j != i]
        p = []
        if with_choosing:
            p.append(("w", f"c{i}", lambda sh, m, i: 1))
        for j in js:
            p.append(("rmax", f"n{j}"))
        p.append(("w", f"n{i}", lambda sh, m, i: m + 1))
        if with_choosing:
            p.append(("w", f"c{i}", lambda sh, m, i: 0))
        for k, j in enumerate(js):
            if with_choosing:
                p.append(("await", (lambda j: lambda sh, i: sh[f"c{j}"] == 0)(j), False))
            p.append(("await", (lambda j: lambda sh, i:
                       sh[f"n{j}"] == 0 or not less((sh[f"n{j}"], j), (sh[f"n{i}"], i)))(j),
                      k == len(js) - 1))
        p.append(("exit", f"n{i}"))
        progs.append(tuple(p))
    names = sorted({f"c{i}" for i in range(N)} | {f"n{i}" for i in range(N)})
    return progs, names


def count(progs, names):
    n = len(progs)
    idx = {v: k for k, v in enumerate(names)}

    @lru_cache(maxsize=None)
    def go(pcs, sh, ms, incs, bad):
        shd = {v: sh[idx[v]] for v in names}
        total = nbad = ndead = 0
        moved = False
        for i in range(n):
            if pcs[i] >= len(progs[i]):
                continue
            op = progs[i][pcs[i]]
            sh2, ms2, inc2 = list(sh), list(ms), list(incs)
            if op[0] == "await":
                if not op[1](shd, i):
                    continue
                if op[2]:
                    inc2[i] = 1
            elif op[0] == "w":
                sh2[idx[op[1]]] = op[2](shd, ms[i], i)
                if len(op) > 3 and op[3]:
                    inc2[i] = 1
            elif op[0] == "rmax":
                ms2[i] = max(ms[i], shd[op[1]])
            elif op[0] == "exit":
                inc2[i] = 0
                sh2[idx[op[1]]] = 0
            moved = True
            p2 = list(pcs); p2[i] += 1
            t, b, d = go(tuple(p2), tuple(sh2), tuple(ms2), tuple(inc2), bad or sum(inc2) >= 2)
            total += t; nbad += b; ndead += d
        if not moved:
            done = all(pcs[i] >= len(progs[i]) for i in range(n))
            return 1, int(bad), int(not done)
        return total, nbad, ndead

    return go(tuple([0] * n), tuple([0] * len(names)), tuple([0] * n), tuple([0] * n), False)


def make_naive_flag(N=2):
    progs = []
    for i in range(N):
        progs.append((("await", lambda sh, i: sh["f"] == 0, False),   # read flag, see 0
                      ("w", "f", lambda sh, m, i: 1, True),         # flag := 1 and walk into CS
                      ("exit", "f")))
    return progs, ["f"]


def lost_update():
    from itertools import permutations
    outcomes = {}
    seqs = set(permutations("AABB"))
    for s in seqs:
        x, t, k = 0, {}, {"A": 0, "B": 0}
        for th in s:
            if k[th] == 0:
                t[th] = x
            else:
                x = t[th] + 1
            k[th] += 1
        outcomes[x] = outcomes.get(x, 0) + 1
    return len(seqs), outcomes


if __name__ == "__main__":
    print("lost update:", lost_update())                    # (6, {1: 4, 2: 2})
    print("naive flag:", count(*make_naive_flag(2)))         # (14, 8, 0)
    for N in (2, 3):
        for wc in (False, True):
            for cmp in ("lex", "strict", "le"):
                print(N, wc, cmp, count(*make_bakery(N, wc, cmp)))
```

---

## Appendix B — sources

- Owner's article: https://jeffreyemanuel.com/writing/bakery_algorithm
- Lamport 1974 PDF: https://lamport.azurewebsites.net/pubs/bakery.pdf
- Lamport, writings page: https://lamport.azurewebsites.net/pubs/pubs.html
- Turing lecture: https://lamport.azurewebsites.net/pubs/turing.pdf
- CHM oral history: https://archive.computerhistory.org/resources/access/text/2017/07/102717182-05-01-acc.pdf
- POPL 1983: https://lamport.azurewebsites.net/pubs/nonatomic.pdf
- On Interprocess Communication: https://lamport.azurewebsites.net/pubs/interprocess.pdf
- Mutual Exclusion Problem Part II: https://lamport.azurewebsites.net/pubs/mutual2.pdf
- EWD123: https://www.cs.utexas.edu/~EWD/transcriptions/EWD01xx/EWD123.html
- x86-TSO: https://www.cl.cam.ac.uk/~pes20/weakmemory/cacm.pdf
- ARM/POWER tutorial: https://www.cl.cam.ac.uk/~pes20/ppc-supplemental/test7.pdf
- Abdulla et al. TACAS'12: https://user.it.uu.se/~parosha/publications/papers/tacas12.pdf
- Milewski: https://bartoszmilewski.com/2008/11/05/who-ordered-memory-fences-on-an-x86/
- Taubenfeld DISC'04 abstract: https://disc.epfl.ch/abstracts/22.html
- TF-A bakery lock: https://codex.ac6.fr/Zephyr/xref/trusted-firmware-a-latest/lib/locks/bakery/bakery_lock_normal.c
- Linux FIFO ticket spinlocks: https://lkml.iu.edu/hypermail/linux/kernel/0711.0/0125.html
- Herlihy 1991: https://cs.brown.edu/~mph/Herlihy91/p124-herlihy.pdf
- Search hits consulted while checking the "whatsoever" quote:
  - https://arxiv.org/pdf/2007.02081
  - https://www.microsoft.com/en-us/research/uploads/prod/2016/12/Reasoning-About-Nonatomic-Operations.pdf
