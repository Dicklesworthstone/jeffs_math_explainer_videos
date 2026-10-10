Plan review round 1 (2026-10-09), persona Dev (28), against plan revision 1. Findings are being dispositioned in the plan's revision 2.

# Dev (28, backend/systems) reviews the three storyboards

I read §1.2, §5 and §9–§11 of the plan, plus the three research memos. I also re-ran a few numbers myself: the uncertainty on the bug example, a carousel baseline for the RaptorQ hook, and a GF(256) multiply built from the drafted xtime. I did not edit anything.

**Bottom line:** The fish video is ready to share once a few fixes land. The bakery video is close, but ch.10 doesn't yet give me something I could ship. The RaptorQ video has the best material and the biggest honesty risks. As storyboarded, it would leave me believing something about RaptorQ that is false, and it calls a single observed failure a "measured one in a million."

---

## Video A: How Many Fish Are in the Lake?

**My takeaways, one per chapter**

| Ch | What I'd tell a colleague |
|---|---|
| 01 | If you sample one spot of a skewed population and scale up, you get garbage. You need a measurement that doesn't care where the fish are. |
| 02 | Tag 1,000, catch 1,000, find 100 tagged, so about 10,000. Suspiciously easy. |
| 03 | Mixing spreads the tags evenly even though the fish stay clumpy, so k/n ≈ M/N. |
| 04 | P(k) is just counting netfuls: C(M,k)·C(N−M,n−k)/C(N,n). |
| 05 | Likelihood is P(what I saw \| N) read as a function of N. Take the argmax. It's not a distribution over N. |
| 06 | Comparing neighbouring lake sizes reduces to the sign of Mn − kN, so the MLE *is* the proportion guess (rounded down, with ties). |
| 07 | The error goes like 1/√k. The exact 95% interval is 8,400–12,073 and it's lopsided. |
| 08 | Mn/k runs high and goes infinite at k = 0. Chapman's +1 fixes both. |
| 09 | Every broken assumption pushes the estimate a predictable way. Fish that are easier to catch push it low. |
| 10 | Same formula for France, for bugs and for war deaths. Bug counts from it are floors. |

| id | ch/beat | sev | what happened to me | suggested fix |
|---|---|---|---|---|
| A-1 | 10_3 | major | Ch.07 spent 85 s teaching me "count recaptures". Then the one example I'd actually use comes with no error bar. With 12 overlaps the exact 95% interval for total bugs is **47–94**, so the "18 nobody found" could be anywhere from **5 to 52**. Chapman gives 58.6 ± 8.4. | Add: "Twelve overlaps is not many. The honest range is about fifty to ninety bugs, so somewhere between five and fifty are still hiding." Show the interval bar. |
| A-2 | 10_3 | major | It never says the reviewers have to work *independently*. On our team the second reviewer sees the first reviewer's comments, which wrecks the overlap. It also doesn't say *which* kind of unevenness hurts. | Add "working separately, without seeing each other's notes." Then the precise point: "It's fine if one reviewer is sharper. What breaks it is bugs that are easier for *everyone*." (Reviewer skill varying is the M_t model, where the formula still holds. Bug difficulty varying is M_h, which is the bias.) |
| A-3 | 10_4 | minor | "Usually an underestimate" is where it stops. Okay, so what do I do? | Show a card: "Treat it as a floor. With three or more reviewers there are estimators built for uneven bugs." Name them in the description (verify which one Briand et al. 2000 recommend). Also mention Mills' 1972 error seeding: plant bugs on purpose, and they're your painted fish. Mutation-testing people will get it instantly. |
| A-4 | 08_6 | major | "When the two catches together cover the lake" reads to me as "between them they caught every fish". And either way I can't check it, because I don't know N. That's circular. | "…it is exactly unbiased whenever the two catches add up to at least the size of the lake, and within a percent or two once you expect four or more recaptures." Keep the 08_7 joke, but give the *reason*: "It's the formula you'd get if you added one imaginary fish that was caught both times." (The memo's proof sketch supports this.) That's add-one smoothing, and I'll never forget it. |
| A-5 | 08 | minor | The simulation is presented as a visual. The idea that would actually change how I work is that simulation and exact calculation agree, so one tests the other. That line is buried in the facts ledger. | Add a 4-line code panel: `k = rng.hypergeometric(M, N-M, n, size=10_000)`; `est = M*n/k`. Narrate: "Simulation and the exact calculation agree to the fish. Test your simulation against something you can compute." Add a planning card: "Expect ~10 recaptures for ±30%, ~100 for ±10%." |
| A-6 | 07_5 | minor | "A careful 95% interval" doesn't tell me how to compute one. | Lower-third: "Keep every N where what you saw isn't in the extreme 2.5% tail." Put a scipy snippet in the description, plus the trap: `scipy.stats.hypergeom(M, n, N)` means (total, tagged, draws). The letters are the opposite of the video's. |
| A-7 | 06 / 07_2 | delight→minor | "Comparing neighbours is a familiar optimization" is a stretch. The real programmer connection is better: C(10000,1000) overflows a double, and 10⁻⁴²⁹ underflows it (float64 bottoms out near 10⁻³⁰⁸). The ratio trick works on four small numbers. | Replace the persona connection: "That probability doesn't even fit in a float64, which is why real code works in logarithms or in ratios like this one." |
| A-8 | 02_6 | minor | "And they're right" is a sales beat. We don't know N, and ch.06 says 9,999 ties with 10,000. | "And that's exactly what the careful math says too." |
| A-9 | 07_7 | minor | `CV` appears on screen and is never defined. | Label it "relative error ≈ √(1/k − 1/n)". |
| A-10 | 07_2–07_3 | minor | One marker shows an absolute probability (10⁻⁴²⁹) and the next shows a relative likelihood (4×10⁻¹⁶²) on the same axis. I noticed. | Put both on the L/L_max scale, or label which is which. |
| A-11 | 05_6 | minor | My immediate reaction to "it's not a probability of N" is "then give me one." | One line: "Want a probability for the lake size? You need a starting belief. With a flat one, you get 8,536 to 12,186: almost the same, but a different question." |
| A-12 | 05_7–05_8, 06_3, 09 | delight | Slicing the 3D bars one way gives a distribution and the other way gives a likelihood. That finally made the distinction click for me. The N² terms cancelling down to Mn − kN is great. The table of failure directions in ch.09 is the most practically useful minute of all three videos. | Keep them. |
| A-13 | 03_4, 04_1–04_3 | minor (pace) | Soup and C(20,4) are slow for me, but Maya needs them. | Keep them brisk. Fine. |

**Verdict.** Yes, I'd share it, after A-1, A-2 and A-4. **Tweet:** "Finally get why the 'tag-and-recatch' estimate is maximum likelihood: the ratio of neighbouring likelihoods literally simplifies to Mn − kN. Also why our two-reviewer bug counts are floors, not estimates."

---

## Video B: Liquid Data (RaptorQ / GF(256))

**My takeaways, one per chapter**

| Ch | What I'd tell a colleague |
|---|---|
| 01 | One-to-many broadcast can't rely on retransmits, and naive repetition wastes a lot. |
| 02 | Packets are XOR equations over chunks. Any K independent ones solve the system. |
| 03 | Random XOR equations are often redundant: at exactly K, 71% of random systems can't be solved. |
| 04 | Arithmetic mod 256 has zero divisors (2·128 = 0), so you can't divide. You need a field. |
| 05 | GF(256) means treating bytes as polynomials mod 0x11D. xtime is three lines. |
| 06 | Log/exp tables turn multiplication into adding logs. |
| 07 | With 256 choices per coefficient, about 0.4% of systems fail at exactly K, and each extra packet cuts that about 256× (in the model). |
| 08 | Dense elimination costs about K³. Sparse peeling is fast but stalls around 98%. |
| 09 | Mostly XOR, plus 10–16 dense byte rows. Inactivation parks stuck unknowns and solves a small dense core at the end. |
| 10 | About 0.6% failure at K and ~2×10⁻⁵ at K+1 on small blocks. The standard promises 10⁻⁶ at K+2. |

**The specific questions I was asked to check:**
- **xtime / GF(256) multiply:** I could write xtime from this, but not as drafted (see B-7). Full multiply and the log tables are only implied.
- **LT peeling decoder:** I'd build one that never starts (see B-8).
- **Inactivation:** I'd get the concept but not a working toy (see B-9).
- **Measured numbers:** not framed honestly yet (see B-2).

| id | ch/beat | sev | what happened to me | suggested fix |
|---|---|---|---|---|
| B-1 | brief, 07_1, 09 | **blocker** | Ch.07 says "each packet can scale each piece by any byte". The brief says RaptorQ uses "random equations whose coefficients come from a 256-element field". So I come away believing RaptorQ packets carry GF(256) coefficients. **They don't.** Every repair symbol is a plain XOR of intermediate symbols. The byte arithmetic lives only in the 10–16 dense check rows (HDPC) that the receiver adds to its own system of equations. If I build "a toy RaptorQ", I'd build random linear network coding instead. Ch.07_6 even says 0/1 coefficients gain nothing from the bigger field, so without a reconciliation the logic looks broken. | 07_1: "This design exists, and it's called random linear network coding. RaptorQ does something cheaper." New 09_1b: "Here's the twist. RaptorQ's packets never use byte coefficients. Every packet is a plain ex-or. The two hundred fifty-six choices live only in ten to sixteen check equations the receiver adds on its own." Fix the brief's takeaway to match. |
| B-2 | 10_1–10_2, title | **blocker** | "Here's how real RaptorQ does, measured… two extra, about one in a million." That number is **one failure** in 10⁶ trials, at **K = 10**. The hook promised 10,000 packets plus one or two. Nothing was measured at K ≥ 1,002 beyond 0.54% at K, and at K = 1,002 there were zero failures at K+1, but only out of 20,000 trials. The plan's own Ruth note forbids this phrasing, and the draft words still use it. | "We ran the real thing a million times, on small blocks of ten pieces. With exactly enough packets it failed about six times in a thousand. One extra: twenty-four times in a million. Two extra: once, too rare for us to measure. The standard *promises* at most one in a million." On screen: K, trial count, failure count and a CI for each bar. Also show model vs reality, 0.39% vs 0.63%, and say why: mostly sparse XOR rows. |
| B-3 | 01_4–01_5 | major | "Send random copies" is a straw man. Nobody does that. The real baseline is a carousel: send pieces 1…K on a loop. At 1% loss I computed **2.6 loops** on average and **5–6 loops** before all 10⁶ phones are done. Random copies (9.8×) makes the fountain look about 4× better than it is. That breaks your own rule 5.1.2 ("honest failure"). | Replace it with: "Send the file on a loop. Miss one packet and you wait a whole loop for it. Most phones need two or three loops. The unluckiest of a million need five or six." Keep the coupon collector as a one-line aside. In 01_3, add "and broadcast TV has no return channel at all". That's a stronger argument than "requests flood the sender". |
| B-4 | 01–02 | major | The phrases "fountain code", "rateless" and "erasure" never appear, so I can't Google what I just watched. Nothing says the sender can make *endless fresh* equations, and that is the property that matters. Nothing says what happens if a corrupted packet gets through, and in practice one bad packet silently poisons every piece it touches. | 02_1: "…as many different combinations as you like, forever. Engineers call this a fountain code." 01 lower-third: "Lost, not corrupted. Checksums throw bad packets away. One bad packet that slips through poisons the decode." |
| B-5 | 02_4 | major | "Each packet says which pieces it combined." With 10,000 pieces, that list would be bigger than the payload. | "Each packet carries a four-byte number. Sender and receiver feed it to the same pseudo-random generator, so both know which pieces it combined." |
| B-6 | 02, 07 | major | A=5 is a single byte, so I assume a piece is a byte. Then in ch.07 data bytes and coefficient bytes blur together. | Lower-third at 02_1: "In practice each piece is a whole packet's worth of bytes, about a kilobyte, and every operation runs across all of them." At 07_1, show one coefficient byte multiplying an entire row of bytes. |
| B-7 | 05_7 | major | The code panel `if a & 0x80 { … }` isn't valid Rust, because the condition must be a bool. In C, `a << 1` promotes to int, so the overflow bit survives unless you cast or XOR 0x11D. And xtime is only "times two". Nothing shows me a full multiply, or that the log route needs a zero check. | Use the panel below. Narrate "zero has no logarithm, so check it first" in 06_3, and in 06_4 say "the table has 510 entries so you never need the mod". |
| B-8 | 08_2–08_3 | major | "Each packet ex-ors about five pieces" plus "a packet with one unknown gives it away". So I build packets of degree 5, and my peeler never starts. | "A few packets carry a single piece. Half carry exactly two. A long tail carry many. That mix keeps the chain reaction going." Add: "Keep, for every piece, a list of the packets that use it." That one sentence is the data structure. |
| B-9 | 09_3–09_4 | major | Parking c is clear. Missing: (1) how to *represent* "A = y2 ⊕ c"; (2) which unknown to park; (3) where the equation for c comes from (the leftover y6, and in the real code the HDPC rows); (4) that this is just Gaussian elimination in a smart order and succeeds exactly when the matrix has full rank (RFC 6330 §5.4.2.1); (5) how big the dense core gets. | Picture: each solved piece shown as (known bytes, list of parked unknowns). Words: "Any choice of what to park is correct. Good choices keep the leftover puzzle small. The leftover packets, plus the byte rows, solve the parked ones. It's ordinary elimination in a clever order, so it works exactly when the equations are independent." Lower-third: "RaptorQ parks 158 on purpose at 10,017 pieces. Search: inactivation decoding." |
| B-10 | 07, 03_6 | major | "1.6 extra packets on average" makes plain XOR sound *fine*. What matters is the tail, and you set it up yourself with a million phones. | "With a million phones, 'fails one time in four' means 230,000 phones stuck. To get to one in a million with ex-or you need about twenty spare packets. With bytes, two." Be honest that 20 spares out of 10,000 is 0.2%, so the byte rows matter most for small, low-latency blocks. |
| B-11 | missing | major | "Why not Reed–Solomon?" I've used RS in object-storage erasure coding and in RAID-6. Not a word about it. | 08 or 10: "Reed–Solomon is perfect: any K of N packets work. But you must pick N up front, and for byte-sized symbols it tops out near 256. A fountain never runs dry." The memo has the bound n ≤ q+k−1. |
| B-12 | 02, 06_7, 10_4 | major / delight | RAID is in my persona contract but shows up only as a footnote. It's the best bridge you have. RAID-5 parity is one XOR equation. RAID-6 adds Q = Σ 2ⁱ·Dᵢ in *this exact field*, so the xtime I just wrote is what my kernel runs. And 06_7's "just a convenient generator" isn't just convenient: RAID-6 needs distinct weights 2ⁱ. Under AES's polynomial, 2 repeats after 51 steps, so arrays with more than 51 data disks would have double failures they couldn't recover. | Add RAID-5 at 02_4 ("one of these equations"). Add RAID-6 at 07_6 as the real-world reason non-binary coefficients matter. Rewrite 06_7 around the 51-disk point. |
| B-13 | 03_3 | minor | "One corner… a line's worth… four patterns" lost me. | "Before anything arrives, one corner is lit: the empty pattern. One packet: two lit. Two packets: four. Each useful packet doubles what you can make." |
| B-14 | 08_6, 09_7 | minor | "In practice peeling stalls at 98–99%" comes from 5 runs of your own simulation, and one run collapsed. The ablation is at K′ = 10 and 101 only. | Say "in our simulation" and show "1 of 5 runs collapsed", which is a great fact. Label the ablation "blocks of 10 and 101 pieces". |
| B-15 | 09_1 | minor | "Adds a few extra helper pieces." Are those transmitted? (No.) And why do they exist at all? | "The helpers are never sent. They exist to rescue the pieces no packet happened to touch," which calls back to 08_5. |
| B-16 | 08_1, 10 | minor | Missing costs: actual throughput; the fact that the *encoder* must run a decode-sized solve to send the originals unchanged; big files split into blocks (at most 56,403 symbols each), so failure scales with the number of blocks; licensing. | Put measured MB/s from the cberner crate on this Mac in the description. Add one line: "Sending the originals first costs the sender one solve." Add one licensing sentence (verify Qualcomm's IPR disclosures). |
| B-17 | 06_7, 09_1 | minor | Jargon: "generator" is undefined, "LDPC/HDPC" are labels with no words, and the "rank" gauge appears before "rank" is ever said. | Define them, or swap in plain labels and give a "search terms" lower-third: fountain code, LT code, precode, inactivation, RFC 6330. |
| B-18 | 05_3, 05_8, 03 | delight | 3·7 = 9. 2·128 = 29. The same two bytes give 0x31 in RaptorQ and 0xC1 in AES, a real interop bug. Erdős–Borwein. | Keep all of it. Optional "huh" (verify first): Intel's GFNI multiply instruction is hard-wired to AES's polynomial. |
| B-19 | 10_5 | minor | "Because a polynomial refuses to factor" is a slogan. The steep curve comes from having 256 choices; the polynomial is just how you get them. | "…because every nonzero byte has a partner to divide by." |
| B-20 | 02_2–02_3, 04_1–04_2 | minor (pace) | Slow for me, needed for Maya. | RAID-5 from B-12 gives me a reason to stay. |

```rust
fn xtime(a: u8) -> u8 { (a << 1) ^ if a & 0x80 != 0 { 0x1D } else { 0 } }
fn mul(mut a: u8, mut b: u8) -> u8 {          // shift-and-add, no tables
    let mut acc = 0;
    while b != 0 { if b & 1 != 0 { acc ^= a } a = xtime(a); b >>= 1 }
    acc                                       // mul(0x57,0x83)=0x31, mul(2,142)=1 (checked)
}
// tables: if a == 0 || b == 0 { 0 } else { EXP[LOG[a] + LOG[b]] }  // 510-entry EXP, no % 255
```

**Verdict.** Not shareable until B-1 and B-2 are fixed. As it stands, Hacker News would "well actually" it in the top comment, and they'd be right. After the fixes, it's the one I'd send to everyone. **Tweet (post-fix):** "3·7 = 9 and 2·128 = 29. RaptorQ's packets are plain XOR, and its one-in-a-million guarantee comes from 16 rows of byte math. Same field as your RAID-6."

---

## Video C: Taking a Number Without a Ticket Machine

**My takeaways, one per chapter**

| Ch | What I'd tell a colleague |
|---|---|
| 01 | x = x+1 is a load then a store. 4 of the 6 interleavings lose an update. |
| 02 | Check-then-set races. Dijkstra wrote the spec, and Lamport drops even atomic reads and writes. |
| 03 | Each thread writes only its own slot. Your ticket is max+1. |
| 04 | Equal tickets are normal. Order by (ticket, id). |
| 05 | The choosing flag stops anyone from judging you before your ticket is written. |
| 06 | One full run: the serving order is exactly lexicographic. |
| 07 | Two lemmas give a contradiction, so two threads can never both be inside. |
| 08 | The proof never uses a value read during a write, so torn reads are fine. Read each slot once. |
| 09 | First-come-first-served is counted from the doorway. Tickets grow forever. Every entry costs O(N) reads. |
| 10 | Real CPUs reorder memory, so you need fences. In practice use CAS. The idea lives on in Lamport clocks and Paxos, and in Arm firmware. |

| id | ch/beat | sev | what happened to me | suggested fix |
|---|---|---|---|---|
| C-1 | 10_3–10_4 | **blocker** | "Add a fence, two per customer." Where, exactly? Which construct in my language? I'd write it with Acquire/Release because that's my default for locks, and **that's broken**: bakery, like Dekker and Peterson, needs a store followed by a load to stay in order. The ALGOL-style panel in ch.06 doesn't map to anything I can compile. I can't write a correct toy from this. | End ch.10 on the Rust panel below. Narrate: "Make every access sequentially consistent. Acquire and release are *not* enough here, because this needs your write to land before your next read." Show the two fence points on the ch.06 panel. |
| C-2 | 03_5 | **blocker** | 03_4 says P4 reads 0, 3, 0, 2 and writes 4. Then 03_5 reads the slates as "three, zero, zero, four, two", which would make the order P5, **P1**, P4, not P2. I replayed it twice. | "Zero, three, zero, four, two." Better still, don't read a list aloud. Point at the slates. |
| C-3 | 10_1–10_3 | major | "A modern laptop… MFENCE… two per customer is enough." My laptop is an M-series Mac, and so is the machine rendering this video. MFENCE isn't an Arm instruction. Two fences is the x86 (TSO) result. Arm also reorders stores with other stores and loads with other loads, so it needs barriers in more places. | "On x86, two fences per customer suffice. Arm is looser and needs more. Easiest: sequentially consistent atomics, and let the compiler place them." Lower-third: "x86's model is called TSO." Optional delight: Apple Silicon has a hardware TSO mode for Rosetta. |
| C-4 | 10_1 | major | "Run this as written, and it fails." Did you run it? Your own rule 5.1.2 asks for honest data, and so far this is a model-checker result. | Run it with Relaxed atomics on this Mac and on an x86 box, and show "N violations per million entries". |
| C-5 | 10 (persona note only) | major | Data races are undefined behaviour in C, C++ and Rust, and the compiler may hoist the spin load so the loop never exits. That's in a persona note, not in the script. "volatile is not a fix" is true for C, C++ and C#, but **false for Java**, where volatile accesses are sequentially consistent. And `volatile int[]` only covers the array reference, not the elements. | On-screen card: "C/C++/Rust: plain variables = undefined behaviour; `volatile` doesn't help. Java: `AtomicIntegerArray` works; `volatile int[]` doesn't." |
| C-6 | 10_5 | major | "Still ships in Arm's trusted firmware." Why? This is the only "why would anyone use this today" answer in the video, and the reason is cut. | "At boot and power-down, some cores run with caches or address translation off, and Arm's compare-and-swap isn't defined on every kind of memory. Plain loads and stores are. So the bakery lock is still there." The memo verified this from the TF-A header. |
| C-7 | 08_4 | major | "Garbage only makes her later." On real hardware, what if the garbage is 2⁶⁴−1? Her ticket overflows. The 2³⁵ bound in ch.09 explicitly assumes reads never return values larger than were written. That's an oversimplification that matters in practice. | "In a real machine, garbage can be enormous, and enormous tickets run out of room. That's why the overflow bound assumes reads never come back too big." Concrete case: "a 64-bit ticket on a 32-bit chip is written in two halves." |
| C-8 | 09_5 | major | Practical costs are missing. N must be fixed in advance. A thread preempted mid-doorway leaves its flag up and stalls everyone at L2. A thread that dies with its flag up deadlocks everyone (Lamport's restart model assumes the failed processor's memory resets to 0; threads don't do that). Slots packed into one cache line ping-pong between cores. An uncontended entry costs about 2(N−1) cache misses vs one CAS. | Add one card listing all five. Add "pad each slot to its own cache line" to the code. Add a delight line: "64-bit tickets at a billion a second: 584 years." |
| C-9 | 03 (persona note) | minor | The ticket-lock connection is the best one for me, and it's only a persona note. | Narrate it: "If you've used a ticket lock, that's the machine: one atomic fetch-and-add." Description: Linux x86 ticket spinlocks (2008), later replaced by queued spinlocks (verify: 2015). |
| C-10 | 01_2 | minor | Too basic for me as is. | Add a "huh": "Even when it's one instruction, `add [x], 1`, the core reads and writes separately unless it's locked." |
| C-11 | 10_2 | minor | The first thing I'll think is "caches are coherent, though." | "This isn't the cache. Caches agree with each other. It's the queue in front of the cache." |
| C-12 | 07_6 | minor | "Every schedule with up to four customers" overstates it: four customers with tickets capped at 4, clean reads only. Safe registers were checked only up to three customers. | State the bounds on the card. Description: "Try it yourself with TLA+ (Lamport's examples include the bakery; verify the path) or Rust's `loom`." |
| C-13 | 04_2, 10_5 | minor | "Mutual exclusion" is used before it's ever named. "Paxos" and "Lamport clocks" are never explained. | Name it at 02_4. For Paxos: "the protocol that keeps replicated databases agreeing." |
| C-14 | 09_2 | minor | "So nobody starves" is missing its conditions. | "…as long as everyone eventually leaves, and nobody dies mid-doorway." |
| C-15 | 04_6 | delight | The JavaScript comma-operator trap is exactly the bug I'd write. | Add: "ESLint's `no-sequences` rule catches it; TypeScript doesn't. In Python and Rust, tuple comparison does the right thing." |
| C-16 | 05, 10_5 | delight | The flag feels like a seqlock's "writer busy" bit, except bakery waits *before* reading instead of retrying after. (counter, id) is how last-writer-wins CRDTs break ties. | Optional one-liners. Label the seqlock one as an analogy. |

```rust
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering::SeqCst};
pub fn lock(i: usize, choosing: &[AtomicBool], number: &[AtomicU64]) {
    choosing[i].store(true, SeqCst);
    let t = 1 + number.iter().map(|n| n.load(SeqCst)).max().unwrap(); // each slot read ONCE
    number[i].store(t, SeqCst);
    choosing[i].store(false, SeqCst);
    for j in (0..number.len()).filter(|&j| j != i) {
        while choosing[j].load(SeqCst) { std::hint::spin_loop() }
        loop { let nj = number[j].load(SeqCst);
               if nj == 0 || (nj, j) > (t, i) { break } std::hint::spin_loop() }
    }
}
pub fn unlock(i: usize, number: &[AtomicU64]) { number[i].store(0, SeqCst) }
```

**Verdict.** I'd share chapters 07–08 with my team today. The proof and the safe/regular/atomic comparison are excellent. The whole video I'd share once C-1, C-2 and C-6 are fixed. **Tweet:** "Lamport's bakery lock is provably correct even when reads return garbage, still breaks on your laptop without SeqCst, and still ships in Arm's boot firmware, because that's where CAS isn't guaranteed."

---

## Across all three

1. **Persona notes aren't the script.** Most of the content I'd value most is sitting in the "Dev:" persona notes: ticket locks, data races as UB, "volatile isn't a fix", the RFC's rank condition, "packets lost not corrupted". The draft words don't contain any of it. The R2 review should check that each Dev note made it into the narration or an on-screen card.
2. **Give me search terms.** Each video should end its technical act with a "names to search" lower-third.
3. **Label measured vs modelled numbers.** Every measured number needs its block size and trial count on screen. Every modelled number needs the word "model".
