<!--
Draft video description for Video C (R2, 2026-10-11; R2 Ruth R2). Every
statement below rests on a row of docs/fact-sheet.md or an oracle key; the
bracketed tags name them. The release package (bead mx-e-c-7v0.16) adds the
chapter timestamps and the repository link, and trims this text to what the
upload page accepts, linking this file for the rest. Edit it whenever the
storyboard's notes send something to "the description".
-->

# The Algorithm Lamport Is Proudest Of

In 1974 Leslie Lamport published a way for programs to take turns using only
memory that each of them writes alone: take a ticket bigger than any you see,
break ties by customer number, and raise a flag while you choose. Its proof
never trusts a read that overlaps a write, so the algorithm survives memory
that returns garbage; this video builds it, proves it, and shows what it costs
on a real laptop.

## Chapters

(Timestamps come with the finished cut.)

1. One plus one is one
2. The room
3. Take a number
4. Same number
5. The unwritten ticket
6. Three customers
7. One run
8. Why never two
9. Reading garbage safely
10. The old zero
11. Fair at a price
12. Your CPU
13. Proudest

## Precisely

Where the video simplifies, here is the exact statement, by beat (chapter_beat).
Bracketed tags name the fact-sheet row or oracle key in the repository.
Model-checking results hold only within their bounds: N customers, tickets at
most K, and either indivisible ("atomic") reads and writes or flickering
("safe") ones.

**01_7–01_8.** Two programs that each read and then write can interleave in
C(4, 2) = 6 orders; the 4 that lose a point are exactly the 4 whose staircase
passes through the middle, where both have read and neither has written.
[C-F1]

**01_9.** These are counts of orders, not probabilities: nothing says a real
machine picks each order equally often, and the video claims no rate for a
real machine.

**02_4.** An order counts each read, each write and each successful wait as one
step, and each customer visits once. "Eight put both in the room at once" means
at some moment; every one of the 14 orders ends with both gone. [C-F2]

**02_5–02_6.** Dijkstra's problem (CACM 8(9):569, 1965), in his words: "at any
moment only one of these N cyclic processes is in its critical section", and
"(a) The solution must be symmetrical between the N computers; as a result we
are not allowed to introduce a static priority." "(b) Nothing may be assumed
about the relative speeds of the N computers; we may not even assume their
speeds to be constant in time." "(c) If any of the computers is stopped well
outside its critical section, this is not allowed to lead to potential
blocking of the others." "(d) If more than one computer is about to enter its
critical section, it must be impossible to devise for them such finite speeds,
that the decision to determine which one of them will enter its critical
section first is postponed until eternity." "Stopped well outside its
critical section" means stopped where it isn't trying to get in, Lamport's
noncritical section (1974, condition 3, p. 454): a customer frozen while
choosing, or paused in line, does hold up the others (chapter 11 and
"Costs" below). Rule (d) already asks that
somebody gets in; Knuth's 1966 addition is that everyone who wants in
eventually gets in (Lamport 1986: "This requirement was first stated and
satisfied by Knuth"). [C-S1, C-F14]

**02_7.** Dijkstra's solution assumes that "Writing a word into or
nondestructively reading a word from this store are undividable operations":
a step nobody can catch halfway, which is what "atomic" means in this video.
[C-S1]

**03 (for programmers).** The shared ticket machine Lamport avoided is a ticket
lock: one atomic fetch-and-add on a shared counter. Linux adopted it for x86
spinlocks in 2008 ("x86: FIFO ticket spinlocks", merged for 2.6.25). Lamport's
own reason a literal ticket machine fails: "A literal implementation of this
approach would require a ticket-machine process that never halts, violating
Dijkstra’s requirements." [fact sheet open item 6, C-S26]

**04_3.** With two customers visiting once and the flag in place, the rule
"wait only for smaller tickets" puts both in the room in 432 of its 898
orders, so they can, not must; the rule "wait for smaller or equal tickets"
never puts both in, but 72 of its 250 orders end with both waiting forever.
[C-F5, C-F6.b2, C-X3]

**04_4–04_6.** The pair order is Lamport's: "(a,b) < (c,d) if a < c, or if a =
c and b < d" (1974, p. 454). Customer numbers are distinct, so this is a
strict total order on pairs, and each pair has only finitely many pairs ahead
of it. "The room always takes the smallest pair of everyone holding a ticket"
is about customers holding tickets at the same moment: whoever is in the room
holds a smaller pair than every other customer holding a ticket, including one
whose flag is still up. [C-S19, C-F7.model.tie, C-X4] Across different moments the
pairs need not line up: after the bakery empties, tickets start again from
one, so a newcomer can be served after earlier customers whose pairs were
bigger. [C-X7]

**05_7.** One visit each, steps counted as in 02_4: without flags, 4 of the 28
orders put both customers in the room; with flags, none of the 322 do. [C-F3]
The replayed traces counted the same way: the unwritten-ticket collision takes
6 steps, the same order with flags 13. [C-F6.a, C-F6.a_prime]

**05_8–05_9.** Lamport calls the time a customer's flag is up the doorway. The
flag is not a lock: nobody waits to raise their own, and all of them can be up
at once. [C-X6] It does not stop two customers choosing at the same time (in
272 of the 322 orders their choosing overlaps), and equal tickets still happen;
the customer number settles them. [C-X10, C-S7, C-F6.c]

**06_2.** Three customers visiting once have 2,126,816,238 orders, and none puts
two in the room. The glowing trails are orders drawn uniformly at random from
all of them, not runs of any real scheduler. [C-F4]

**07.** The run is trace d, with four customers (Pink stays home). Counted as in
02_4 it takes 64 steps. Customers are served (1, Green), (1, Purple), (2, Teal),
(3, Blue), (4, Green), exactly the pair order, and never two at once. Green
leaves before Blue checks her, so Blue waits only for Purple, then Teal.
[C-F6.d]

**08 (the proof), in full.** Customers 1…N; customer i alone writes choosing[i]
and number[i]; ≺ is the pair order above.

- *Doorway D_i:* from the start of the write choosing[i] := 1 to the end of
  choosing[i] := 0. It contains i's ticket write.
- *Ticket held, T_i:* from the end of i's ticket write number[i] := n_i to the
  start of its exit write number[i] := 0. Throughout T_i, number[i] = n_i ≥ 1,
  and no write to number[i] overlaps T_i.
- *In the bakery, B_i:* from the end of D_i to the start of the exit write. Both
  lemmas use only this definition.
- *Assumption M* (Lamport 1983): if number[j] is not written while customer i
  computes her maximum, she chooses a value bigger than number[j]. Reading
  each slate exactly once meets it. [C-S20]
- *Lemma 1 (fact one).* If every read that k makes of number[i] while choosing
  her current ticket lies inside T_i, then n_i < n_k. In particular this holds
  if D_i ended before D_k began and i has not begun its exit write by the end
  of D_k. (Those reads overlap no write and return n_i, so M gives
  n_k ≥ n_i + 1.)
- *Lemma 2 (fact two).* If i is in the critical section and k ≠ i is in the
  bakery, then (n_i, i) ≺ (n_k, k). Proof: let r_2 be i's last read of
  choosing[k], which returned 0, and r_3 its later last read of number[k].
  (a) If k's opening write choosing[k] := 1 ended after r_2 began, all of k's
  reads of number[i] come after it, hence after i's ticket write, and before
  i's exit write, so Lemma 1 gives n_i < n_k, whatever r_3 returned.
  (b) Otherwise the opening write ended before r_2 began, and then k's closing
  write choosing[k] := 0 began before r_2 ended (if r_2 overlapped no write it
  read 0 only because the closing write was over; if it overlapped a write, that
  was the closing one). k's ticket write came before that, so r_3 lies inside
  T_k, overlapped no write, and returned the true n_k; i passed it, so
  (n_i, i) ≺ (n_k, k). A flag read caught mid-raise lands in (a), one caught
  mid-lowering in (b).
- *Theorem.* Two customers in the room would each hold a smaller pair than the
  other. ∎
- The proof uses the values of only two kinds of ticket read, k's reads of
  number[i] in Lemma 1 and i's read r_3 in case (b), and no write overlaps
  either. That is why garbage is harmless.
- The computer search: four customers, tickets at most 4, atomic reads and
  writes: 3,027,633 reachable states, none with two in the room (runs that
  would take a ticket above 4 are cut). [C-F8] Fact two checked for every
  holder: no violation with three customers, tickets at most 5 (atomic) or 4
  (flickering slates and flags). [C-X4]

**09_2–09_3.** Lamport named three kinds of memory in "On Interprocess
Communication" (Distributed Computing, 1986): in a safe register "a read not
concurrent with any write obtains the correct value—that is, the most recently
written one. No assumption is made about the value obtained by a read that
overlaps a write, except that it must obtain one of the possible values of the
register." Regular and atomic registers are the stronger classes. The paper
first appeared as a research report dated December 1985, and Lamport writes
that he "had obtained all the results presented here in the late 70s". [C-S2]

**09_5.** A glance at a flag that catches it mid-raise counts as not yet
choosing; mid-lowering, as finished (the case split in 08). The model agrees:
three customers, tickets at most 6, flickering slates and flags: 269,550
states, none with two in the room. [C-F8]

**09–10, the 1974 footnote.** Lamport's original proof treated a single bit as
if it could not be caught mid-flip; he later wrote: "The paper contains one
small but significant error. In a footnote, it claims that we can consider
reads and writes of a single bit to be atomic. … It was only later, with the
work eventually described in [70], that I realized the fallacy in this
reasoning." (In the printed paper the argument sits in the proof of Assertion
2.) The algorithm stands: the case split above handles a flag caught
mid-flip, and the model checks with flickering flags find no violation.
[C-S18] He also wrote: "Several books have included emasculated versions of the
algorithm in which reading and writing are atomic operations, and called those
versions "the bakery algorithm". I find that deplorable. There's nothing wrong
with publishing a simplified version, as long as it's called a simplified
version." Chapter 08's search is that simplified version, and chapters 09–10
lift it. [C-S28]

**10_1–10_2.** In the trace shown, Green's glance returns 0, which is exactly
Blue's slate before her write began, so the same thing could happen with atomic
memory; the point is that the proof never needs the value. Green finishes
choosing (step 14), then sees Blue's flag down (step 17), and only then does
Blue raise it again (step 21); Blue's ticket ends bigger and she waits.
[C-F6.e3, fact sheet F3]

**10_3.** Verbatim: "The read may return any arbitrary value!" (CACM 17(8),
1974, p. 454); "I discovered that the bakery algorithm had this property after
writing a proof of its correctness" and, of Anatol Holt, "He refused to believe
it could be true." (Lamport's publications page). [open items 3–4, C-S3, C-S4]

**10_4.** Lamport 1983: "For n > 2 we need the following additional assumption
… If number[j] does not change during execution of the statement, then
number[i] is chosen to be greater than number[j] – regardless of the concurrent
activity of other processes. Thus, the following is not a valid implementation
if the two instances of number[k] represent separate fetches." In the model the
double-read maximum puts two in the room with three customers and flickering
reads (a 33-step trace), but not with two customers, and not with atomic reads.
[C-S20, C-F6.double_fetch]

**11_1.** First come, first served, in Lamport's 1986 words: "if process i
finishes executing its doorway statement before process j begins executing its
doorway statement, then i must execute its critical section before j does."
When two customers' choosing overlaps, either may go first. [C-S24, C-X2,
C-X10]

**11_2.** First-come-first-served alone does not keep anyone from waiting
forever: "In particular, FCFS does not imply deadlock freedom." Lamport's
Theorem 1: "FCFS and deadlock freedom imply lockout freedom." Deadlock
freedom is his Assertion 3 (1974): with finitely many failures, "some processor
must eventually enter its critical section", the one holding the smallest pair.
The video's conditions are his: "the system can only be deadlocked by a
processor halting in its critical section, or by an unbounded sequence of
processor failures and re-entries." The bound "at most once" counts from the end
of your ticket write. [C-S24, C-S19, C-X3, C-X5]

**11_3.** Lamport 1974: "If there is always at least one processor in the
bakery, then the value of number [i] can become arbitrarily large." The largest
ticket held only ever drops to zero, when nobody holds a ticket. [C-S19, C-X7,
C-F6.leapfrog]

**11_4–11_6.** Lamport's sum (1974, p. 455): "if processors enter the doorway at
the rate of at most one per msec, then after a year of operation we will have
number[i] < 2^35--assuming that a read of number[i] can never obtain a value
larger than one which has been written there." A 365.25-day year has
31,557,600,000 ms (a 365-day year 31,536,000,000), both below
2³⁵ = 34,359,738,368. Under that condition each new ticket is at most one more
than the largest ticket written so far, so after a year at one entry a
millisecond no ticket exceeds 31,557,600,000: "up to about thirty-one and a
half billion". A 32-bit slate overflows after 2³² entries, 49.7 days at one a
millisecond, or after 2³¹, 24.9 days, if signed, as Java's `int` is. A 64-bit
slate lasts 2⁶⁴ ns = 584.5 years at a billion tickets a second, or 292.3 years
if signed, as Java's `long` is. [C-F10, C-F11, C-S16] How a glance can return
more than was written: a 64-bit ticket written in two halves. In Java, "a
single write to a non-volatile long or double value is treated as two separate
writes: one to each 32-bit half. This can result in a situation where a thread
sees the first 32 bits of a 64-bit value from one write, and the second 32 bits
from another write." But "Writes and reads of volatile long and double values
are always atomic." (JLS SE 21 §17.7), and `AtomicLongArray` gives its
elements volatile access. [C-S22, C-S15] Lamport's garbage tolerance (10_5)
assumes slates that can hold any integer: on a b-bit slate a glance may return
2ᵇ − 1 and the next ticket overflows, so in practice it is paired with reads
that never exceed what was written, which untorn aligned words give.

**12_1–12_5.** The outbox is the store buffer. Intel: "Reads may be reordered
with older writes to different locations but not with older writes to the same
location." AMD: "Where sequential consistency is needed (for example in
Dekker’s algorithm for mutual exclusion), an MFENCE instruction should be used
between the store and the subsequent load, or a locked access, such as XCHG,
should be used for the store." [C-S10] In an x86-TSO model (two customers,
tickets at most 4, outboxes of at most 4 writes): with no fence, both enter in
12 steps, also in the order 12_3 describes, with memory still all zeros; a fence
only after raising the flag fails in 18 steps, one only after lowering it in 20;
both together hold, also for two customers with tickets at most 6 and for three
customers with tickets and outboxes at most 3. [C-F9] Abdulla et al. infer the
same: "two fences per process, one at line 3 and the other at either line 10 or
line 12" (after raising the flag, then after writing the ticket or after
lowering the flag), for two processes with tickets at most 2. [C-S21]

**12_6.** On Arm, another core can see two writes to different places in the
opposite order: the "message passing" outcome is "allowed in the architecture,
and it is commonly observable", and "the writes on Thread 0 are to distinct
addresses and so can be committed out of order on Thread 0; and/or they might
also be propagated to Thread 1 in either order; and/or the reads on Thread 1
(likewise to distinct addresses) can be satisfied out of order". [C-S11, C-S25]
Blue writing her ticket and then lowering her flag, with Green checking the
flag and then the ticket, is that test, so Green can see the flag down and the
ticket still zero: chapter 05's unwritten ticket. A model shows where that
leads: if an outbox may deliver writes to different slates out of order, the
two x86 fences let both customers in, in 22 steps. That model is not a model of
Arm, which reorders in more ways; nothing here gives a fence count for Arm.
[C-F9.ooo_outbox] Barriers on both sides forbid the outcome: "the sync or DMB
on Thread 0 keeps the two writes in order". [C-S25]

**12_7.** Sequentially consistent atomics are correct everywhere for this code:
"programs that correctly use mutexes and memory_order::seq_cst operations to
prevent all data races and use no other synchronization operations behave as if
the operations executed by their constituent threads were simply interleaved"
(C++ [intro.races]; Rust and Java give the same guarantee). [C-S12] Acquire and
release are not enough: on x86 they compile to plain moves ("Load Acquire: MOV
(from memory)", "Store Release: MOV (into memory)"), which is the unfenced
program that fails in 12 steps. [C-S13] The irony: under the recommended
mapping every sequentially consistent store on x86 is a locked XCHG, or a MOV
and an MFENCE, so the easy fix does use one of the hardware's magic signs, as
a fence, never to keep customers apart. [C-S14] In C, C++ and Rust, plain
shared variables race, and "Any such data race results in undefined behavior";
`volatile` does not change that. In Java, `AtomicLongArray` works and a
`volatile long[]` does not. [C-S15, C-S23]

**13_1.** Lamport himself: "Although of little if any practical use, the bakery
algorithm [11] has become a popular example of a mutual exclusion algorithm."
(Turing lecture, 2015) [C-S26]

**13_2.** Arm's Trusted Firmware-A (TF-A): "BL31 accesses the state of a CPU or
cluster immediately after reset and before the data cache is enabled in the
warm boot path. It is not currently possible to use 'exclusive' based
spinlocks, therefore BL31 uses locks based on Lamport's Bakery algorithm
instead." Its PSCI code uses them only on platforms that are not
hardware-coherent (`HW_ASSISTED_COHERENCY == 0`), while the power controller's
lock (`arm_lock`) is a bakery lock in BL31 either way; the lock code still
issues barriers (`dmbish`, `dsb`) and packs the choosing flag and the ticket
into one word written by one store; exclusive instructions "have defined behavior
only on certain memory types (cacheable and Normal memory in particular …)".
[fact sheet open item 5, F2, C-S17]

**13_3.** Time, Clocks (1978): "To break ties, we use any arbitrary total
ordering < of the processes." Paxos Made Simple (2001): "if it uses a proposal
with number greater than any already used, then it will succeed in issuing a
proposal that is accepted", and "Different proposers choose their numbers from
disjoint sets of numbers, so two different proposers never issue a proposal
with the same number." Lamport links the counters himself, with a hedge: "in
Paxos, there is also a counter that’s being used in, not quite in the same
way". [C-S5, C-S6, C-S27]

**13_4.** "I feel that I did not invent the bakery algorithm, I discovered it."
(publications page). Asked by Roy Levin why it is the one he is proudest of,
Lamport first said "No, I can’t." and later: "But the bakery algorithm, it just
seemed to come out of thin air to me. There was nothing like it that preceded
it, so perhaps that’s why I’m proudest of it." (Computer History Museum oral
history, 2016, pp. 9–10) [open item 4, F4, C-S27]

## Code

The lock from chapter 12, as on screen, in Rust with sequentially consistent
atomics. F1 is the order from line 3 (raising the flag) to line 5 (reading the
slates); F2 is the order from lines 6–7 (writing the ticket, lowering the flag)
to line 9 (checking the others' flags), the two fence points of 12_5.

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
pub fn lock(i: usize, choosing: &[AtomicBool], number: &[AtomicU64]) {
    choosing[i].store(true, SeqCst);
    // each slate read once
    let t = 1 + number.iter().map(|n| n.load(SeqCst)).max().unwrap();
    number[i].store(t, SeqCst);
    choosing[i].store(false, SeqCst);
    for j in (0..number.len()).filter(|&j| j != i) {
        while choosing[j].load(SeqCst) { std::hint::spin_loop() }
        loop {
            let nj = number[j].load(SeqCst);
            // pairs never tie: j != i
            if nj == 0 || (nj, j) > (t, i) { break }
            std::hint::spin_loop()
        }
    }
}
// real code: pad each slate to its own cache line; unlock in a Drop guard
pub fn unlock(i: usize, number: &[AtomicU64]) { number[i].store(0, SeqCst) }
```

`(nj, j) > (t, i)` compares in dictionary order (Rust's documentation: "in
PartialOrd and Ord, the elements are compared sequentially until the first
non-equal set is found"), and the pairs never tie because j ≠ i. The maximum reads every slate once, its own included
(it holds 0 at that point), so an entry makes 3N − 2 loads, 3(N − 1) of them of
other customers' flags and slates. [C-S29, C-F12]

A trap from a TypeScript version of this lock: it compared pairs with
`(t[j], j) < (t[i], i)`. In JavaScript that is the comma operator: each
parenthesis evaluates both operands and keeps the last, so the line compares
`j < i` and ignores the tickets. `tsc --strict` (TypeScript 5.9.3) reports
nothing. Python's tuples compare in dictionary order, as Rust's do. [C-F13,
C-S29]

## Costs

- **Customers are fixed in advance:** one flag and one slate per customer, 2N
  shared variables, N of them unbounded tickets.
- **Every entry reads a lot, even alone:** each other customer's slate twice
  (for the maximum and in the wait loop) and her flag once, 3(N − 1) reads.
  [C-F12]
- **Waiting is spinning:** a waiting customer keeps re-reading.
- **A customer frozen while choosing stalls everyone:** each other customer
  gets in at most once more, then nobody does. [C-X8]
- **A customer paused in line stalls everyone behind her:** if she is
  descheduled in her wait loop, nobody whose pair is bigger than hers gets in
  until she runs again. [C-X9]
- **A dead customer's slate stays set:** Lamport's 1974 paper assumes that a
  failed processor's memory eventually reads zero ("Eventually, any read from
  its memory must give a value of zero"). In code nothing resets a dead
  thread's slate; that is your job. [C-S19]
- **Ticket width:** use 64-bit tickets; in Java, `AtomicLongArray`, not
  `AtomicIntegerArray`, whose tickets overflow after 24.9 days at one entry a
  millisecond. [C-F10, C-S16]
- **Memory order:** sequentially consistent atomics everywhere, or fences placed
  for your architecture; on x86 two per entry suffice in the model above.
  [C-F9, C-S21]

## Sources

- E. W. Dijkstra, "Solution of a Problem in Concurrent Programming Control",
  CACM 8(9):569, September 1965 (doi:10.1145/365559.365617). [C-S1]
- D. E. Knuth, "Additional comments on a problem in concurrent programming
  control", CACM 9(5):321–322, May 1966. [C-F14]
- L. Lamport, "A New Solution of Dijkstra's Concurrent Programming Problem",
  CACM 17(8):453–455, August 1974,
  https://lamport.azurewebsites.net/pubs/bakery.pdf. [open item 3, C-S18, C-S19]
- L. Lamport, "Time, Clocks and the Ordering of Events in a Distributed
  System", CACM 21(7):558–565, July 1978. [C-S5]
- L. Lamport, "Reasoning About Nonatomic Operations", POPL 10, January 1983,
  28–37, https://lamport.azurewebsites.net/pubs/nonatomic.pdf. [C-S20]
- L. Lamport, "The Mutual Exclusion Problem Part II", JACM 33(2), 1986,
  https://lamport.azurewebsites.net/pubs/mutual2.pdf. [C-S1, C-S24]
- L. Lamport, "On Interprocess Communication", Distributed Computing
  1(2):77–101, 1986. [C-S2]
- L. Lamport, "The Part-Time Parliament", ACM TOCS 16(2):133–169, May 1998, and
  "Paxos Made Simple", ACM SIGACT News 32(4), December 2001. [C-S6]
- L. Lamport, "The Computer Science of Concurrency: The Early Years" (Turing
  lecture), CACM 58(6):71–76, June 2015,
  https://lamport.azurewebsites.net/pubs/turing.pdf. [open item 2, C-S26]
- L. Lamport, publications page, entry for the 1974 paper,
  https://lamport.azurewebsites.net/pubs/pubs.html. [open item 4, C-S3, C-S4,
  C-S18, C-S28]
- Oral History of Leslie Lamport, interviewed by Roy Levin, Computer History
  Museum, 2016, pp. 9–10,
  https://archive.computerhistory.org/resources/access/text/2017/07/102717182-05-01-acc.pdf.
  [F4, C-S27]
- Trusted Firmware-A, https://github.com/ARM-software/arm-trusted-firmware
  (docs/design/firmware-design.rst; lib/locks/bakery; lib/psci/psci_private.h).
  [open item 5, C-S17]
- N. Piggin, "x86: FIFO ticket spinlocks", Linux commit 314cdbef (2008); J.
  Corbet, "Ticket spinlocks", LWN, February 6, 2008. [open item 6]
- Intel 64 and IA-32 Architectures Software Developer's Manual, Vol. 3A §11.1–11.2
  and Vol. 2A (LOCK); AMD64 Architecture Programmer's Manual Vol. 2 §7.2–7.3.
  [C-S8, C-S9, C-S10, C-S14]
- L. Maranget, S. Sarkar, P. Sewell, "A Tutorial Introduction to the ARM and
  POWER Relaxed Memory Models", 2012. [C-S9, C-S11, C-S25]
- C. Pulte, S. Flur, W. Deacon, J. French, S. Sarkar, P. Sewell, "Simplifying
  ARM Concurrency", POPL 2018. [C-S11]
- Apple, "Addressing architectural differences in your macOS code". [C-S11]
- "C/C++11 mappings to processors", University of Cambridge,
  https://www.cl.cam.ac.uk/~pes20/cpp/cpp0xmappings.html. [C-S13, C-S14]
- P. A. Abdulla, M. F. Atig, Y.-F. Chen, C. Leonardsson, A. Rezine,
  "Counter-Example Guided Fence Insertion under TSO" (TACAS 2012). [C-S21]
- C++ working draft, [intro.races] and [atomics.order],
  https://eel.is/c++draft/intro.races; ISO C committee draft N1570 §5.1.2.4.
  [C-S12, C-S13, C-S15, C-S23]
- Rust standard library documentation, `std::sync::atomic`,
  `std::ptr::read_volatile`, primitive `tuple`. [C-S12, C-S15, C-S29]
- The Java Language Specification, SE 21, §4.2.1, §17.4.5, §17.7;
  `java.util.concurrent.atomic`. [C-S12, C-S15, C-S16, C-S22]

Every number in this video was computed by code and checked against an
independent oracle; the scripts, the oracle and the fact sheet are in the
repository.
