<!-- R1 storyboard review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; storyboards at 2ad2579) -->

# R1 review: Video C, "The Algorithm Lamport Is Proudest Of" (storyboard). Reviewer: Dev, 28

## What I checked and how

- **Read:** `tools/review_prompts/dev_28.md` and all of `docs/reviews/bakery_algorithm/R1-packet.md` (brief, storyboard, fact sheet with its findings and table). For three specific points I grepped `docs/research/bakery_algorithm/MEMO.md`: fence placement, the failure model, and the double-fetch variant.
- **Recomputed by hand:**
  - 01: 6 orders, 4 lose a point, and those 4 are exactly the paths through (1,1).
  - 02: 14 orders, 8 put both inside. I built a 3×3 lattice with two blocked edges: 20 − 3 − 3 = 14 orders, and 4 × 2 = 8 pass through (2,2).
  - 03: slates 0,3,0,0,2 → customer 4 reads 0,3,0,2 and writes 4; service order 5 → 2 → 4.
  - 08_1: the rewrite 09 → 10 can be read mid-write as 19 or 00.
  - 09_4: 31,557,600,000 < 34,359,738,368.
  - 2³² ms = 49.71 days and 2³¹ ms = 24.86 days.
- **Recomputed with my own model:** `/tmp/r1_dev_c/paths.py` (`python3 -I`), two customers, one visit each, a step = read, write or successful wait. It gives no flag: 28 orders, 4 both inside; flag: 322 orders, 0 both inside; 0 deadlocks. This matches 05_6.
- **Rust panel (10_4):**
  - I copied it verbatim to `/tmp/r1_dev_c/panel.rs` and ran `rustc --edition {2021,2024} --crate-type lib -D warnings panel.rs` (rustc 1.100.0-nightly, aarch64-apple-darwin). It compiles clean with no warnings.
  - `--emit asm`: on x86_64-apple-darwin, `lock` + `unlock` use 4 `xchg` (2 `xchgb`, 2 `xchgq`) and no `mfence`. On aarch64 it is `stlr`/`ldar` with no `dmb`.
- **Stress test** (`/tmp/r1_dev_c/stress.rs`, `rustc -O`, `./stress 3000000 300000`, M4 Pro):
  - Setup: threads start on a barrier. The critical section increments a counter as a separate load and store (the `count++` race from chapter 01). "Overlaps" counts entries where a Relaxed occupancy `fetch_add` found someone already inside, which is a lower bound. "Lost" counts lost increments.
  - These are single runs, not a benchmark. Results:

| variant (only the orderings differ from the panel) | N=2, 6,000,000 entries: overlaps / lost | N=4, 1,200,000 entries: overlaps / lost |
|---|---|---|
| panel verbatim (all SeqCst) | 0 / 0 | 0 / 0 |
| Release stores, Acquire loads | 2 / 11 | 66 / 1,166 |
| Relaxed | 11 / 61,213 | 2 / 36,013 |
| Release/Acquire + `fence(SeqCst)` after raising and after lowering the flag | 0 / 0 | 0 / 0 |

  - Acquire loads compile to `ldapr` (RCpc) on this target, which explains why Release/Acquire fails here.
- **Ticket growth** (`/tmp/r1_dev_c/tickets.rs`, panel verbatim): with N=2, 6,000,000 entries took 0.66 s and the largest ticket was 407,661.
- **Code panel, line by line:**
  - The `use … Ordering::SeqCst` group import is valid.
  - L3–L6 are the doorway. The max loads every slate exactly once, own slate included. The own slate is 0 at entry, so that is harmless; it makes 3N−2 loads, as fact-sheet F5 says. `unwrap` panics only when N = 0.
  - L8 is Lamport's flag wait (L2).
  - L9–L13 break when `nj == 0 || (nj, j) > (t, i)`. Rust tuples compare lexicographically, and since j ≠ i the pairs are never equal, so `>` means "not before me". That is exactly Lamport's L3.
  - Waiting on the local `t` instead of re-reading `number[i]` is fine, because only customer i writes `number[i]`.
  - Every access is SeqCst, so the SC proof applies. The data in the critical section is protected because the loads that let a customer through are acquires that read the other customer's release stores.
  - **Verdict:** it compiles, it is a correct bakery lock, and it does what the voice says ("your write must land before your next read").
- **Not checked by me** (I rely on the fact sheet's checks only): 2,126,816,238; 3,027,633; 269,550; the 8-, 12-, 15-, 33- and 70-step traces; the TSO results; Burns–Lynch; Abdulla et al.; every quote, date and source URL; the TF-A and Linux claims.
- **ESLint:** that claim is checked against the rule's documented default only. ESLint isn't installed here and I didn't run it.

## Comprehension check

| ch | the one sentence I'd tell a colleague |
|---|---|
| 01 | `x = x + 1` is a read then a write. Of the 6 interleavings of two increments, exactly the 4 that pass through "both read, neither wrote" lose an update, and at billions of steps a second that happens constantly. |
| 02 | A check-then-flip "occupied" sign fails the same way (8 of 14 orders let both in). Hardware fixes it with an atomic test-and-set, and Dijkstra fixed it with plain reads and writes, though the video says "an ordinary sign is enough", which I can't square with the sign that just failed. The cliffhanger: what if a read can catch a write halfway? |
| 03 | A shared ticket machine would itself need a lock, so instead each customer writes only her own slate. She takes max-of-the-others + 1 and waits for every smaller nonzero ticket, so the only conflicts left are reads overlapping someone else's write. |
| 04 | Two customers who read at the same time can take the same ticket. Comparing tickets alone either lets both in (`<`) or deadlocks them (`≤`), so you compare (ticket, id) lexicographically, which is unique and totally ordered. |
| 05 | A customer who has picked her ticket but not yet written it still shows 0 ("not in line"), so a rival walks in and then she walks in too. A "choosing" flag around ticket-taking makes that window visible and everyone waits it out: from 4 bad orders of 28 to 0 of 322. |
| 06 | In a four-customer run, the flag stops one collision, a tie goes to the lower id, and a late arrival with a low id still queues behind lower tickets. The serving order is the lexicographic sort of (ticket, id). |
| 07 | Two facts make two-in-the-room a contradiction. First, whoever finished choosing before you started has a smaller ticket. Second, whoever is in the room has the smallest (ticket, id) of everyone holding one. The proof splits on where the other customer was when you read their flag. |
| 08 | The proof never relies on a read that overlaps a write, so even garbage reads leave the lock correct: garbage only makes someone take a bigger ticket or wait longer. Lamport noticed this only after writing the proof. |
| 09 | It is first-come-first-served from the end of the doorway, so nobody starves and each rival can overtake you at most once. But tickets grow while the line never empties, an entry costs 3(N−1) reads even when nobody else is in line, N is fixed in advance, and a stalled or dead customer can stall everyone. |
| 10 | The proof assumes my write is visible before my next read. CPUs park writes in a store buffer, so both customers can read stale zeros and both walk in. Fences fix it (two on x86), or most easily SeqCst atomics; acquire/release doesn't, because it doesn't order a store before a later load. |
| 11 | Ordinary locks use hardware atomics, but Arm's trusted firmware runs the bakery lock when a core wakes with its cache off and can't use its usual spinlocks. The (counter, id) tie-break reappears in Lamport clocks and Paxos ballots. |

## Rubric (1–5)

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 5 | 5 | 5 | 4 | 4 |
| 02 | 3 | 3 | 4 | 4 | 3 |
| 03 | 5 | 5 | 5 | 4 | 4 |
| 04 | 5 | 5 | 5 | 4 | 4 |
| 05 | 4 | 5 | 3 | 4 | 4 |
| 06 | 3 | 4 | 3 | 3 | 4 |
| 07 | 4 | 5 | 5 | 4 | 4 |
| 08 | 3 | 4 | 4 | 4 | 5 |
| 09 | 4 | 3 | 4 | 2 | 4 |
| 10 | 4 | 4 | 5 | 3 | 5 |
| 11 | 4 | 4 | 4 | 4 | 4 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| dev-01 | Dev | 02_5 (vs 02_3, 09_5) | major | "Dijkstra showed an ordinary sign is enough." Ten seconds earlier you showed me an ordinary sign failing (02_3). The 09_5 card then says any read/write lock needs at least N shared variables (Burns–Lynch). As written, all three can't be true, and I leave 02 thinking one plain flag can be a lock. | "Dijkstra showed that ordinary signs are enough: several of them, each read or flipped in one indivisible step, as long as nobody can catch one halfway through a flip." Keep "sign" plural. |
| dev-02 | Dev | 09_4 (and 10_4 card) | major | "Could tickets outgrow an ordinary computer number? … two to the thirty-five. That fits easily." To me an ordinary number is 32 bits, and 2³⁵ doesn't fit. At Lamport's rate of one per millisecond, a 32-bit ticket wraps in 49.7 days and a signed one in 24.9 days. The 10_4 card then recommends Java `AtomicIntegerArray`, which is a signed 32-bit int. My stress run reached ticket 407,661 in 0.66 s of nonstop two-thread contention, so an `int` ticket would wrap in roughly an hour (one run, extrapolated). Twice the video steers me toward a lock that breaks in production. | "…below two to the thirty-five: too big for 32 bits, which would wrap in about fifty days. Use 64 bits: at a billion a second that lasts 584 years" (C-F11). Change the card to `AtomicLongArray`. |
| dev-03 | Dev | 05_4 / 05_7 | major | 03_2 threw out the ticket machine because it "needs a room of its own". Then 05_4 adds a flag everybody waits on. My first objection: isn't that just a spinlock, so you've gone in a circle? The brief lists "`choosing` is not a lock" as a must-get-right, and no draft line says it. | Add to 05_7: "The flag isn't a lock. Nobody ever waits to raise their own flag, many can be up at once, and someone else's flag stays up only for the few steps it takes them to write a ticket." |
| dev-04 | Dev | 09_2 / 09_5 | major | The failure modes are incomplete. A customer who dies or stalls while holding a ticket (waiting, or in the room) blocks everyone with a bigger ticket forever, which means everyone who arrives later. The card has "dies with its flag up" but not "dies holding a ticket". The video also never says that Lamport's fault tolerance assumes a failed processor restarts with both slates at 0 (memo l.112). Threads don't do that for you: a panic in my critical section leaves `number[i]` set. | 09_2: "…as long as nobody dies holding a ticket or a raised flag." Card: add "dies holding a ticket → everyone behind waits forever (Lamport assumes a dead customer's slates reset to zero; in code that's your job)". |
| dev-05 | Dev | 08_5 | major | Pronoun soup: "lets **her** straight in … the writer … so **she** started choosing … **her** ticket will be bigger, so **she** waits". Two different customers share "she". The key step is also missing: why must the writer have started choosing after the reader finished? Because the reader had already seen the writer's flag down. Without that I can't rebuild the argument. | Use the lane colours: "Green's glance catches Blue's old zero and Green walks in. But Green had already seen Blue's flag down, so Blue started choosing after Green finished. By fact one, Blue's ticket is bigger, so Blue waits." |
| dev-06 | Dev | 10_3 | major | "On an Intel or AMD chip, two fences per customer are enough." Enough where? I can't place them. Separately, the panel I'm handed is not the two-fence version: compiled for x86-64, SeqCst gives 4 `xchg` per lock+unlock (I checked the asm), versus the 2 fences you just said suffice. | Name them on the outbox x-ray: "one right after raising your flag, one right after lowering it" (memo l.1036–1037, F1/F2). In 10_4: "SeqCst is the easy way, not the cheapest: on x86 it fences every store." |
| dev-07 | Dev | 06 "for programmers" (06_1 panel) | major | "ESLint's `no-sequences` catches it" is, I believe, false at default settings. The rule's `allowInParentheses` option defaults to true, and `(t[j], j)` is a parenthesized sequence, so the owner's snippet passes. I'm going by the documented default; ESLint isn't installed here and I didn't run it. The storyboard also doesn't show the 06 code panel, so I can't review it. | Run ESLint on `owner_snippet_lock.ts` and record the result next to the `tsc` row. If I'm right, say "`no-sequences` with `allowInParentheses: false` catches it". Put the 06 panel text in the storyboard, as 10 does. |
| dev-08 | Dev | 09_5 | major | The costs card is the payload for me: seven dense items. The voice is "And it isn't cheap." (≈2.5 s) and the row has no hold, so it's unreadable unless I pause. Burns–Lynch also has no fact-sheet row (the memo cites Inf. & Comp. 107(2), 1993). | Give it a hold of at least 10 s, or voice the top three: N fixed in advance, 3(N−1) reads even alone, a stalled customer stalls everyone. Add Burns & Lynch 1993 to the receipts. |
| dev-09 | Dev | 10_1 (honest-data note) | minor | My run produced measured failures on this M4 Pro. Release/Acquire lost 11 updates in 6,000,000 entries (N=2) and 1,166 in 1,200,000 (N=4). Relaxed lost 61,213 and 36,013. The panel and Release/Acquire + F1/F2 lost 0. Caveat: Release/Acquire fails here because Acquire compiles to `ldapr`. On an Arm core where Acquire loads use `ldar`, the hardware keeps a release store ordered before an acquire load, so it might pass a test while the language still promises nothing. | Promote `/tmp/r1_dev_c/stress.rs` (or an equivalent) into the bead as a recorded experiment, run it on the x86 host too, and only then let 10_1 say "measured". Report lost updates per million entries per variant. |
| dev-10 | Dev | 10_3 (vs 11_1) | minor | "Arm chips … reorder more and need more" has no source and no number: more what? Also, 10_3's "caches agree with each other" collides with 11_1's "chips without hardware cache coherence". | "Arm chips reorder more, so the safe recipe is the same: SeqCst atomics." Either drop "need more" or back it with a fact row. Say "on your laptop, caches agree with each other". |
| dev-11 | Dev | 08_4 | minor | "Where do the magenta glances land? All in the first case." What about Blue glancing at Green's flag while it's being lowered? That read can return 0, and Green is still mid-doorway, so it lands at the middle/finished boundary, not in case (a). It's harmless because Green's ticket is already written, but "all in the first case" is not literally true. | "Glances at a flag going up land in the first case. A glance at a flag coming down comes after the ticket is written, so it's harmless." Ruth to confirm against memo §4. |
| dev-12 | Dev | 09_6 | minor | "Read one twice" is abstract. The double fetch the memo checks (MEMO l.638) is the loop every programmer writes, `if (number[j] > m) m = number[j];`, and a C compiler may also re-read a plain variable. That would make me remember it. Be honest that it bites only when reads can tear. | Show that two-line loop as the "read twice" picture, and in the voice: "the obvious max loop reads twice; with torn reads and three customers, it breaks". |
| dev-13 | Dev | 08 Dev aside / 11_1 | minor | I'll wonder from 01_1 on whether modern RAM returns 2496 for an int. It doesn't for an aligned word, and TF-A's own comment relies on "single-copy atomicity for aligned accesses". So the real deployment uses the "plain loads and stores only" property, not the garbage tolerance. "The usual spinlocks" also hides the mechanism: Arm's exclusive load/store needs cacheable memory. | 08 aside: "today's chips don't tear an aligned word; the proof just doesn't need that". 11_1 lower-third: "load/store-exclusive needs cacheable memory; the bakery lock needs only plain loads and stores". |
| dev-14 | Dev | 10_4 panel | minor | Panel nits. (a) 09_5 says "pad each", but `&[AtomicU64]` packs 8 slates per 64-byte line (16 per 128-byte line on Apple M). (b) `(nj, j) > (t, i)` is the inverse of the voice's "wait while theirs comes before mine"; it's correct only because j ≠ i. (c) A panic in the critical section never unlocks. (d) A 16-line panel plus a card is on screen for about 12 s. | Add comments: `// pairs never tie: j != i`, `// real code: pad each slate; unlock in a Drop guard`. Highlight lines in voice order (flag up, max, ticket, flag down, two waits), or hold about 10 s longer. Call out `(nj, j) > (t, i)` as the 06 tuple-compare payoff. |
| dev-15 | Dev | 03 "for programmers" | minor | "Linux x86 FIFO ticket spinlocks, 2008" is verified for 2008. Without a qualifier, I'd assume Linux still uses them; as far as I remember, x86 moved to queued spinlocks in 2015 (unchecked; the fact sheet didn't check this either). | Check it and say "introduced in 2008" or "2008 to 2015", whichever is true. |
| dev-16 | Dev | 07_7 | minor | "Tickets ≤ 4": what does the search do with a run that would take ticket 5? Without that, I can't tell what "every order" covered. | Card: "tickets capped at 4 (runs that would go past it are cut)", using whatever the checker actually does. |
| dev-17 | Dev | 11 (chapter title) | minor | To a programmer, "out of thin air" right after a memory-model chapter means OOTA values, the C++/Java memory-model problem. I expected that topic. | Keep the phrase only inside Lamport's quote card, or give the chapter title a different name in captions and chapter markers. |
| dev-18 | Dev | 10_2, 10_4 | delight | The outbox x-ray is the store-buffering litmus test made visible: memory frozen at zeros while both walk in. "This needs your write to land before your next read" is the exact reason for SeqCst. The panel compiles, is a correct bakery lock and survived my stress test. | Keep as is (with dev-06 and dev-14). |
| dev-19 | Dev | 01_6–01_7, 01 aside | delight | The `count++` race as geometry: 4 of 6 staircases through the middle (I recomputed it). The `add [x], 1` lock-prefix aside connects to something I've actually tripped on. | Keep. |
| dev-20 | Dev | 04_3 | delight | `<` lets both in and `≤` deadlocks: exactly the off-by-one I'd write, shown as two traces. | Keep. A quick link back to the rules from 02_4 would help ("breaks rule one" / "breaks rule four"). |
| dev-21 | Dev | 06 aside | delight | The comma-operator trap, with `tsc --strict` silent on it, is the best "for programmers" moment in the video, once dev-07 is fixed. | Keep; fix the ESLint wording. |
| dev-22 | Dev | 08_6 | delight | "Noticed only after writing the proof": the proof found a property nobody designed for. That sells proofs to me better than any claim would. | Keep. |

Scratch files are in `/tmp/r1_dev_c/`: `panel.rs`, `stress.rs`, `tickets.rs`, `paths.py`, and the `panel_arm.s` / `panel_x86.s` asm. Nothing in the repository was edited.
