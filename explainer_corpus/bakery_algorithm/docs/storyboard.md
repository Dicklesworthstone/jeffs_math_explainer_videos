# Storyboard — The Algorithm Lamport Is Proudest Of

The hand-edited source of this video's words and pictures (from R1, 2026-10-11;
it started as plan §11.4, revision 2). `python3 -I tools/storyboard.py
bakery_algorithm` checks it (gate G2) and writes `script.tsv` from the words
column. Facts: `docs/fact-sheet.md` and `oracle.py`; brief: `docs/brief.md`;
R1 reviews and their dispositions: `docs/reviews/bakery_algorithm/`.

Words are spoken exactly as written, one sentence at a time: no colons,
semicolons, dashes or parentheses in them (voice rules, plan §5.4).
`*(…)*` is an annotation, never spoken. `⟨predict⟩` opens a prediction
question; the beat ends with the question and the picture holds before the
next beat answers it.

## 2 The arc

| Act | Chapters | Spine |
|---|---|---|
| I. The problem | 01 one plus one is one, 02 the room | a step is a read then a write; check-then-set fails; Dijkstra's problem and Knuth's addition; the harder question |
| II. Building the bakery | 03 take a number, 04 same number, 05 the unwritten ticket, 06 three customers, 07 one run | slates and tickets; ties → (ticket, number); unwritten tickets → the flag |
| III. Why it works | 08 why never two, 09 reading garbage safely, 10 the old zero | three places a customer can be; the proof never trusts a read that overlaps a write |
| IV. Costs and legacy | 11 fair at a price, 12 your CPU, 13 proudest | fairness, unbounded tickets, costs; outboxes and fences; firmware, clocks and Paxos |

## 3 Color roles

| Idea | Color |
|---|---|
| customers 1–5 | five fixed hues, named aloud by colour: **Blue** (1), **Green** (2), **Purple** (3), **Teal** (4), **Pink** (5); each wears its number on a badge. Hex values are chosen so every pair, and every customer against every role colour, is at least ΔE2000 20 apart (R1 creative C-3; checked by a lint before drafts) |
| the room (critical section) | warm red |
| the flag (choosing) raised | amber `#F0AC5F` |
| tickets / slates | chalk white |
| the order (ticket, number) | gold `#FFD866` (hero) scanline |
| a glance that overlaps a write | white static with colour fringing (not magenta, which sits too close to Pink; C-3) |
| store buffers ("outboxes") / hardware | cool grey |

## Vocabulary (one word per idea)

**customer** (a program; "programmers say process" once, at 03_1), **the
room** (named "critical section" once), **order** (one way the steps can
interleave; lower-third: counting each read, write and successful wait as one
step), **slate** (a register), **ticket** (never "number"), **customer number**
(breaks ties; customers are named by colour aloud), **flag**, **choosing**
(the time a flag is up; Lamport's "doorway" appears once, on screen),
**served** (going into the room), **outbox** (a store buffer).

## 01_one_plus_one_is_one (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A slate reads nine. Someone is rewriting it as ten. Glance at it mid-stroke, and you might read nine, or ten, or, Lamport says, two thousand four hundred ninety-six. | A chalk slate frozen mid-stroke; flickering digits; quote card (Lamport's Turing Award lecture, CACM 58(6), 2015; 2496 set in full-size digits, F8) | flicker prop |  |
| 01_2 | Could computers take turns safely with memory like that? In nineteen seventy-three, that was considered impossible. | Two lanes reach for the same slate; a dated card "1973: considered impossible" (Turing lecture, verbatim) | quote card |  |
| 01_3 | *(title)* | **The Algorithm Lamport Is Proudest Of** | title card | listen 3 s (the "take a number" motif) |
| 01_4 | Two programs, running at once, share a scoreboard that reads forty-one. Programmers write x equals x plus one. It isn't an equation. It means make x one bigger. | Scoreboard; two lanes, Blue and Green; "time runs left to right; a read is an arrow in, a write is an arrow out" | **space-time lanes** |  |
| 01_5 | That's two steps. Read the score, then write it back plus one. | Arrows on the lanes | step animation |  |
| 01_6 | Here's one unlucky order. Both read forty-one, and both write forty-two. Two points scored, one counted. | x ends at 42 | step animation |  |
| 01_7 | ⟨predict⟩ Draw every order at once, as staircases from corner to corner. There are six. How many lose a point? | The lanes morph into the progress square: Blue's lane becomes the bottom axis, Green's rotates up to become the side; the order just shown becomes the first staircase, then the other five draw in | **progress square** |  |
| 01_8 | Four. Exactly the ones through the middle, where both have read and neither has written. The fix has to wall off the middle. | The 4 paths through (1,1) turn red | highlight |  |
| 01_9 | Counting orders isn't measuring chances. Each program usually runs both steps back to back, so in a real machine the bad orders are rare. But at billions of steps a second, rare means constantly. | The six orders shrink into one long strip of steps racing past; red flecks appear along it again and again; lower-third "6 orders, 4 bad: a count, not a probability" | time strip | hold |

- **For programmers (lower-third):** even `add [x], 1` reads and writes
  separately unless it carries a lock prefix.
- **Ruth:** six orders = C(4,2) interleavings of two 2-step programs (R11).

## 02_the_room (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | The fix seems obvious. Only one program at a time may touch the scoreboard. Call that stretch of code the room. | A room around the scoreboard; lower-third "critical section; the rule is called mutual exclusion" | room prop |  |
| 02_2 | Put a sign on the door. Check that it says vacant, flip it to occupied, and go in. | Sign | sign prop |  |
| 02_3 | ⟨predict⟩ Does the sign work? | The two lanes walk up to the sign | prediction card |  |
| 02_4 | No. Both read vacant before either flips it. Of the fourteen orders, eight put both in the room at once. | Progress square with a red square "both inside"; the 14 staircases draw in and a counter ticks the 8 that cross it; lower-third "counting each read, write and successful wait as one step" | progress square |  |
| 02_5 | Edsger Dijkstra posed the problem properly in nineteen sixty-five. Never two in the room. Someone who wanders off outside can't block everyone. And fast or slow, it still works. | Three pictograms on the lanes, numbered 1–3, parked in a corner: the red square; a dimmed lane outside the room that the other passes; lanes stretching in time | pictograms |  |
| 02_6 | A year later, Donald Knuth asked for more. Everyone who wants in gets in eventually. | Pictogram 4: every lane eventually reaches the room | pictograms |  |
| 02_7 | Hardware usually cheats, with a magic sign you can check and flip in one motion. Dijkstra showed that ordinary signs are enough, several of them used cleverly, as long as nobody can catch one halfway through a flip. | The magic sign, then several ordinary signs (never the single sign that just failed) | statement card |  |
| 02_8 | Now a harder question. What if a glance can catch a sign mid-flip, as a blur? Hold that thought. | A sign blurs | flicker | hold |

- **Ruth:** the four rules as Lamport states Dijkstra's problem "as extended by
  Knuth"; "atomic" is defined once (one indivisible step) and never used
  loosely. Fact-sheet row C-S1 quotes Dijkstra 1965's conditions verbatim
  (R9: his rule (d) already required that somebody gets in, so Knuth's
  "everyone" is more); the words above make no claim either way.
- **Pictograms (creative C-12):** rule 1 is ticked at 08_8 ("never two in the
  room: done"), rule 4 at 11_2.

## 03_take_a_number (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | Picture a bakery. Take a number, and the lowest number is served next. From now on, each program is a customer, and being served means going into the room. | The two lanes get customer tags (Blue 1, Green 2); three more lanes join (Purple 3, Teal 4, Pink 5); the room morphs into the serving counter; lower-third "programmers say process" | scene morph |  |
| 03_2 | But a ticket machine is shared. Two customers pressing at once could get the same ticket, just like the lost point. It would need a room of its own. | Dispenser with a drawn red ✗ (`exmark`) | statement card |  |
| 03_3 | So each customer gets her own slate, and only she writes on it. A zero means not in line. | Five slates at the heads of the five lanes | slates prop |  |
| 03_4 | ⟨predict⟩ To take a ticket, read every other slate, one at a time, and write one more than the biggest. Blue reads three, zero, zero, two. What does she write? | Read arrows from Blue's lane; slates 0, 3, 0, 0, 2 | prediction card |  |
| 03_5 | Four. Then she waits until nobody with a smaller ticket is still in line. Here that's Pink, with two, then Green, with three, then Blue. | Slates now 4, 3, 0, 0, 2; served in turn: Pink → Green → Blue | queue |  |
| 03_6 | Whoever is in the room plays their own tune, so only one tune sounds at a time. | Each served customer's lane grows note-heads as it plays: the lanes become a stave | lanes as a score | listen 3 s ("served" in Pink's, Green's and Blue's timbres, one after another) |
| 03_7 | No shared ticket machine. Nobody ever writes on anyone else's slate. | Each lane's own slate glows in its colour; every arrow from another lane is a read, none a write | lanes, slates | hold |

- **For programmers (lower-third at 03_7):** a ticket lock is the machine
  Lamport avoided: one atomic fetch-and-add (Linux x86 FIFO ticket spinlocks,
  introduced in 2008; dev-15).
- **Facts to pin:** Blue (customer 1) reading 3, 0, 0, 2 → 4, served Pink,
  Green, Blue (M3 moved the reader off customer 4; C-F7 is recomputed for this
  example).

## 04_same_number (~75 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 04_1 | ⟨predict⟩ Can two customers get the same ticket? | Rewind to before anyone was served: 03's slates 4, 3, 0, 0, 2 | prediction card |  |
| 04_2 | Yes. Before anyone is served, Purple and Teal read the slates at the same moment. Both see four as the biggest, and both write five. | Both read, both write 5 | step animation |  |
| 04_3 | If equal tickets both go in, both end up in the room. Change the rule to wait for anyone with a smaller or equal ticket, and they both wait, bowing to each other forever. After you. No, after you. | Two traces: both in the room; then two customers bowing | traces | listen 3 s (two soft notes alternating politely, never resolving) |
| 04_4 | The fix is a tie-breaker. Compare tickets, and if they're equal, the lower customer number goes first, like a dictionary. Purple is three and Teal is four, so Purple goes first. | (5, Purple 3) before (5, Teal 4): each pair drawn as a chalk ticket and a coloured badge, never two bare numerals | ordering card |  |
| 04_5 | ⟨predict⟩ Put every ticket on a grid, ticket across and customer number up, and sweep column by column. Blue holds ticket three. Teal holds ticket two. Who goes first? | Lexicographic grid; the gold scanline waits before column 2 | **lexicographic grid sweep** |  |
| 04_6 | Teal. A customer with the lowest number still waits for every smaller ticket. The number only breaks ties. That's the order in which the room takes everyone holding a ticket. | The scanline sweeps; (2, Teal) before (3, Blue) | grid sweep | hold |

- **Ruth:** ≺ is a strict total order on ℕ × {1…N} (customer numbers
  distinct); every pair has finitely many predecessors. 04_6's claim is about
  customers holding tickets at the same time: a later run can serve a smaller
  pair after a bigger one once the first has left (R13; description).
- **Facts:** 04_2's tickets are model-checked (`C-F7.model.tie.*`: from 4,
  3, 0, 0, 2 Purple and Teal both read 4, 3, 0, 2 and write 5; every order
  then serves Pink, Green, Blue, Purple, Teal); the replayed equal-ticket
  traces c (both take 1) and c′ (both see 1, both take 2) are the small cases.

## 05_the_unwritten_ticket (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | The second crack is sneakier. Blue reads Green's slate, sees zero, and decides on ticket one. But she hasn't written it yet. | Ghost "1" hovering over Blue's slate | step-through |  |
| 05_2 | Green takes ticket one, checks Blue's slate, sees zero, which means not in line, and walks in. | Green enters | step-through |  |
| 05_3 | Then Blue writes her one, sees a tie, wins it on her lower number, and walks in too. Both in the room. | Both inside | 8-step trace (a) | listen 3 s (two tunes at once: the bug) |
| 05_4 | The fix is a flag that says, I'm choosing, don't judge me yet. Raise it before reading, and lower it after writing. Before comparing with someone, wait until their flag is down. | Amber flags | flag prop |  |
| 05_5 | ⟨predict⟩ Same unlucky order, now with flags. What does Green do this time? | Scheduler slider replays the order: the same eight steps in place, the flag steps slotted in, amber (15 steps) | toggle replay |  |
| 05_6 | She sees Blue's flag up, and waits. | Green waits | step-through |  |
| 05_7 | Without flags, two customers visiting once have twenty-eight possible orders, and four put both in the room. With flags, there are three hundred twenty-two, and not one does. | The 28 staircases as strands, 4 through the red square; they collapse into a "28 orders, 4 bad" card; the lattice regrows to 7×7; the 322 draw in as edge ribbons with a lagged reveal and a counter (spike S-6); the both-inside point stays empty and pulses | **path sets (S-6 ribbons)** |  |
| 05_8 | The time your flag is up is your choosing time. The flag isn't a lock. Nobody waits to raise their own, and many can be up at once. | Choosing shaded on the lanes; on screen once, "Lamport calls this the doorway" | statement card |  |
| 05_9 | So the gap between looking and writing is still there. The flag makes it visible, and the tie-breaker settles equal tickets. | Thesis card | statement card | hold |

- **Ruth:** C-F3 (no flag 28 / 4 bad; flag 322 / 0), C-F4 (three customers
  2,126,816,238 / 0, chapter 06); one visit each, a wait counted once, when it succeeds
  (lower-third at 05_7; D2). The tie-breaker settles equal tickets only; the
  flag and fact one make the other races harmless (R18).
- **Dev:** the flag is not a lock: nobody waits to raise their own, many can
  be up at once, and a flag stays up only for the steps it takes to write a
  ticket (dev-03).
- **Engine:** blocked moves depend on history, so paths are drawn, not walls.

## 06_three_customers (~30 s; the whole chapter on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ Now take three customers. The square becomes a cube. How many orders are there now? | The 7×7 square extrudes into a 10×10×10 cube; "two inside" beams, translucent, drawn last | **3D cube, orbit < π** |  |
| 06_2 | Over two billion. And not one of them puts two in the room. | Sampled staircases thread the cube as glowing trails (sampled uniformly over orders, E-C2) while a counter climbs to 2,126,816,238; the beams stay dark | **3D cube, orbit < π** | hold |

- **Engine:** about 30 s on the camera route (≈ 5 min at S-5's 6.1 fps); about
  200 sampled polylines on the 10×10×10 lattice; beams are depth-tested
  translucent prisms drawn last; arcs under π (E-C2). Cut it first if the
  video runs long: 05_7's ribbons carry the claim.

## 07_one_run (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Here's one full run, fast. | The five lanes with their slates at the heads; a step clock (no code panel during the run, C-7) | lanes + step clock |  |
| 07_2 | Purple and Green arrive together, and both choose ticket one. Purple finishes first and finds Green's flag still up, so she waits. Without the flag, she'd have read Green's unwritten ticket as zero. | Fast-forward, slowing at the beat | step clock |  |
| 07_3 | ⟨predict⟩ Teal sees one and one. What ticket does she take? | Teal reading | prediction card |  |
| 07_4 | Two. And the tie between Purple and Green goes to Green, the lower number. | (1, Green 2) before (1, Purple 3) | ordering card |  |
| 07_5 | Blue arrives late, with the lowest number, and takes three. She waits for Purple, then Teal. Green leaves, gets back in line, and takes four. Who's served first follows the grid exactly. | (1, Green) → (1, Purple) → (2, Teal) → (3, Blue) → (4, Green), the grid sweep beside the lanes; each entry plays its customer's "served" motif (C-2); chapter 01's scoreboard sits inside the room and ticks once per visit, 41 → 46, no point lost (Maya m6) | callback | hold |

- **Engine:** slate values swap by replacement transforms at their step events, driven by the shared step clock, never redrawn every frame (E-C4).
- **For programmers (a code panel after 07_5, or the description):** the
  owner's TypeScript `(t[j], j) < (t[i], i)` is JavaScript's comma operator: it
  evaluates `j < i`; `tsc --strict` is silent (C-F13); tuple comparison in
  Python and Rust is correct. ESLint's `no-sequences` claim is unverified
  (dev-07: its default `allowInParentheses: true` may let it pass): run ESLint
  on the snippet and record the result before saying anything about it.

## 08_why_never_two (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Why can two customers never be in the room at once? Picture Blue inside, and Green holding a ticket. | Lanes; Blue and Green; lower-third "the proof assumes: each customer's steps happen in order; a finished write is seen by every later read; the biggest ticket is found by reading each slate once" | lanes |  |
| 08_2 | Fact one. If Blue finished choosing before Green started, and still holds that ticket, then Green's ticket is bigger. Green saw Blue's. | Fact one card, in colours | card |  |
| 08_3 | Fact two. If Blue is in the room, everyone else holding a ticket has a bigger pair than hers. Why is that true? | Fact two card | card |  |
| 08_4 | ⟨predict⟩ Take the last time Blue checked Green's flag, the check that let her through. Where could Green have been? | Green's choosing interval slides along the time axis against Blue's last flag check (labelled) | **sliding interval with zones** |  |
| 08_5 | Three places. Not yet choosing. Then by fact one, Green's ticket will be bigger. | Zone "not yet" | zones |  |
| 08_6 | In the middle of choosing. Then Green's flag was up, and that check wouldn't have let Blue through. | Zone "choosing", forbidden | zones |  |
| 08_7 | Finished choosing. Then Blue read Green's real ticket, and went in only because her pair was smaller. Either way, Green's pair is bigger. That's fact two. | Zone "done" | zones |  |
| 08_8 | Now suppose both were in the room. Then fact two says each one's pair is smaller than the other's. Like being taller than you, and you taller than me. Impossible. | 04's grid returns: the scanline tries to reach both pairs first and tears; then (n_B, Blue) < (n_G, Green) < (n_B, Blue); pictogram rule 1 ticks | TransformMatchingTex | hold |
| 08_9 | A computer search agrees, over every order, up to four customers, as long as tickets stay small. But a search only covers small cases. The proof covers all of them. | Card "N = 4, tickets ≤ 4 (runs that would pass 4 are cut), indivisible reads and writes: 3,027,633 reachable states, no violation", then "every N" | card |  |

- **Precisely (description):** Lemma 1 and Lemma 2 exactly as in the memo §4,
  with case (b) "the write that ends choosing began before the check ended",
  Bᵢ ending where the exit write *begins*, and one definition of "in the
  bakery" used in both lemmas (R5; fix memo §4 the same way); Assumption M (the
  max reads each slate once).
- **Music:** swing drops out for 08–10 (rubato, a pedal under the lemmas).

## 09_reading_garbage_safely (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Remember the slate caught mid-stroke? Rewrite oh-nine as ten one digit at a time, and a glance in between reads nineteen, or zero zero. | Two-digit rewrite, both write orders | flicker |  |
| 09_2 | Lamport went further. A glance during a write may return any number the slate could hold. Even two thousand four hundred ninety-six. | Wild values; the 01_1 slate's 2496 | flicker |  |
| 09_3 | Twelve years later he named memory like that safe. A funny name, since it's the least safe kind. | "safe" card; "regular" and "atomic" on screen for Ruth | register zoo (screen) |  |
| 09_4 | ⟨predict⟩ Back to the proof picture. Let every slate and every flag flicker while it's written. Does the proof ever trust a ticket read that crosses a write? | 07's zones, same layout and scale (C-16); glances that cross a write become white static | **overlay on the proof zones** | listen 3 s (a read that crosses a write smears its note with a slow tape bend) |
| 09_5 | Never. It trusts just two ticket reads. Green reading Blue's finished ticket, and Blue reading Green's finished ticket. Nobody was writing either slate. | The two trusted reads in gold; static lands in every zone, always on reads the proof never used | zones |  |
| 09_6 | A glance that catches garbage, say seven, just makes a customer take eight and wait longer. | Trace e1 inside the zones | trace |  |

## 10_the_old_zero (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | ⟨predict⟩ And a glance that catches Blue's old zero mid-write lets Green straight in. Is that dangerous? | Trace e3 | prediction card |  |
| 10_2 | No. Green had already seen Blue's flag down, and nobody writes a ticket with the flag down. So Blue started choosing after Green finished. By fact one, Blue's ticket will be bigger, so Blue waits. | Blue's choosing begins after Green's ends | trace |  |
| 10_3 | Lamport noticed this only after writing the proof. In his words, the read may return any arbitrary value. His colleague Anatol Holt refused to believe it. | Quote cards, one source each: "The read may return any arbitrary value!" (CACM 17(8), 1974); "I discovered that the bakery algorithm had this property after writing a proof of its correctness" and "He refused to believe it could be true" (Lamport, publications page); 2496 flickers beside a calmly waiting customer: "Can computers take turns safely with memory like that? Yes." | quote card |  |
| 10_4 | One caution, from Lamport himself. Find the biggest ticket by reading each slate exactly once. The obvious loop reads a slate twice, once to compare and once to copy, and a flicker between them can break it. | The two-line max loop that reads twice; then only the outcome: two lanes in the room, two tunes clashing (C-13) | trace | hold |

- **Lower-thirds:** "a glance at a flag mid-raise counts as not yet choosing;
  mid-lowering, as finished" (R2, dev-11); "today's chips don't tear an aligned
  word; the proof just doesn't need that" (dev-13).
- **Ruth:** safe / regular / atomic per "On Interprocess Communication"
  (1986); the safe-register model check (N ≤ 3; 269,550 states at tickets ≤ 6,
  zero violations) is evidence, the proof is the reason. Lamport's own 1974
  single-bit footnote treated flags as atomic, a fallacy he later
  acknowledged; the algorithm stands (description; R2, D5). e3's zero is the
  slate's value before the write (F3): the point is that the proof never needed
  it.
- **Dev:** on a real machine garbage can be enormous (a 64-bit ticket written
  in two halves on a 32-bit chip), which is why the overflow bound in 11
  assumes reads never come back too big.

## 11_fair_at_a_price (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | It's also fair. Finish choosing your ticket before I start choosing mine, and you're served first. If we were choosing at the same time, either of us may go first. | First-come-first-served on the lanes | lanes |  |
| 11_2 | So nobody waits forever, as long as everyone in the room eventually leaves, and nobody stalls for good with a raised flag or a ticket. Once you've taken your ticket, each other customer can get ahead of you at most once. | Bounded bypass; pictogram rule 4 ticks | lanes |  |
| 11_3 | The price is that tickets keep growing as long as somebody is always in line. They reset only when the bakery empties. | Odometer 1, 2, 3, 4, 5, 6 (digits slide and fade; no clip primitive, E-C3) | odometer |  |
| 11_4 | ⟨predict⟩ Lamport did the sum. At one customer a millisecond, how big do the tickets get in a year? | 31,557,600,000 ms in a year | prediction card |  |
| 11_5 | Still below about thirty-four billion, two to the thirty-five. That's too big for a thirty-two bit number, which would wrap in under two months. | 31,557,600,000 < 34,359,738,368; 2³² ms = 49.7 days | arithmetic card |  |
| 11_6 | A sixty-four bit number lasts centuries, even at a billion tickets a second. As long as a glance never returns more than was ever written. | 584.5 years at 10⁹/s unsigned; 292.3 signed, as in Java; lower-third Lamport 1974 verbatim: "a read of number[i] can never obtain a value larger than one which has been written there" | arithmetic card |  |
| 11_7 | And it isn't cheap. Even alone, a customer makes three reads for every other customer. One customer frozen while choosing stalls everyone. | Three beats on the lanes: spinning (a waiting lane fills with soft repeated notes); a customer frozen mid-choosing, every other lane stalled behind it; two slates in one grey cache-line box bouncing between cores | lanes | listen 3 s (spinning) |

- **Costs card (for programmers, held at least 10 s, or in the description):**
  the number of customers fixed in advance; 3(N−1) reads of other customers'
  flags and slates per entry, even alone (R17); spinning; a customer paused
  while choosing stalls everyone; one that dies holding a raised flag or a
  ticket blocks everyone behind it (Lamport assumes a dead customer's slates
  reset to zero; in code that's your job; dev-04, R6); slates sharing a cache
  line ping-pong (pad each); 2N shared variables, half unbounded, while any
  read/write lock needs at least N (Burns & Lynch, Inf. & Comp. 107(2), 1993;
  add to the receipts, dev-08). Use 64-bit tickets: `AtomicLongArray`, never
  `AtomicIntegerArray` (dev-02).
- **Precisely:** "if i finishes choosing before k starts choosing, i enters
  first"; the bypass bound holds once a customer has taken her ticket (R16);
  the double-fetch failure needs three or more customers and reads that can
  return garbage (C-F6, 33-step trace; "can break", R16).

## 12_your_cpu (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | The proof is right. But run this on your laptop as written, and it can fail, because your laptop breaks one of the proof's promises. | 07's promise card; "a finished write is seen by every later read" flickers and breaks over a CPU outline (R8) | promise card |  |
| 12_2 | Your laptop has several processors, called cores. Picture each one running a customer. Each core drops its writes in an outbox and keeps going. | Store-buffer outboxes appear beside two lanes | **outbox x-ray** |  |
| 12_3 | ⟨predict⟩ Both customers raise their flags and write their tickets, into their outboxes. Then each reads the other's slate. What does it see? | Both outboxes full; the real slates still zero | **outbox x-ray** |  |
| 12_4 | No flag. Both walk in, while the real slates still say nobody is even choosing. | Both enter (12 steps); memory frozen at zeros | outbox x-ray | listen 3 s (two tunes at once) |
| 12_5 | The fix is a fence. Wait until your outbox is delivered before reading anyone else's slate. On an Intel or AMD chip, two fences each time you enter are enough, one after raising your flag, and one after lowering it. | A fence drains a tray with a whoosh; the two fence points marked on the lanes | fence |  |
| 12_6 | Arm chips, like the ones in recent Macs, reorder even more, so they need more care. | Arm outline | statement card |  |
| 12_7 | *(for programmers)* The easy, correct way everywhere is sequentially consistent atomics. Acquire and release are not enough here, because this needs your write to land before your next read. | SeqCst Rust panel (below); the ticket store and the next `load(SeqCst)` of another slate highlighted while the voice says it; held at least 6 s; card: C, C++ and Rust plain shared variables are undefined behaviour and `volatile` doesn't help; Java `AtomicLongArray` works, a `volatile long[]` doesn't; then the panel slides away and the picture holds on the outbox draining | code panel | hold |

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

- **Lower-thirds:** "it's not the cache: caches agree with each other; it's
  the queue in front of them" (M15); "x86-TSO model, checked to 3 customers;
  matches Abdulla et al., TACAS 2012" (R15); "SeqCst is the easy way, not the
  cheapest: on x86 it fences every store" (dev-06).
- **Honest data:** "it can fail" is backed by the TSO model (12-step trace).
  Dev's R1 stress run on this Mac (M4 Pro; `count++` inside the lock;
  6,000,000 entries at N = 2) lost 0 updates with the SeqCst panel, 11 with
  Release/Acquire (whose Acquire loads compile to `ldapr` here) and 61,213
  with Relaxed. That becomes the c7v0.17 experiment (recorded with its
  commands, on this Mac and the x86 host) before any narration says
  "measured".
- **Panel (dev-14, E-C5):** `// each slate read once` sits on its own line;
  `(nj, j) > (t, i)` is the inverse of "wait while theirs comes before mine",
  correct because j ≠ i.

## 13_proudest (~65 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 13_1 | ⟨predict⟩ In practice, people use the hardware's magic signs, called atomic instructions. So does the bakery still run anywhere? | The magic sign from 02_7 | prediction card |  |
| 13_2 | Yes. In Arm's own reference firmware, in the code that runs while a core wakes up with its data cache still off. Until it rejoins the others, the usual spinlocks can't be used, so it takes a ticket, bakery style. | Text card: Arm Trusted Firmware, "bakery lock: a core waking with its cache off" (no logos) | card |  |
| 13_3 | And its trick of taking a ticket bigger than any you've seen, with numbers to break ties, reappears in how Lamport later ordered events across whole networks. It's in the ballot numbers of Paxos too, the method that keeps copies of a database agreeing. | 04's grid relabelled (clock value, process); timeline 1974 → 1978 → Paxos | callback |  |
| 13_4 | Lamport says he didn't invent the bakery algorithm. He discovered it. It just seemed to come out of thin air, and perhaps, he says, that's why he's proudest of it. | The five lanes taking turns in a calm loop, one "served" motif at a time in five timbres, the gold scanline sweeping; the quotes set as captions over it; receipts card | coda | listen 3 s (the motif resolves) |

- **Precisely (lower-third at 13_2):** "TF-A uses the bakery lock immediately
  after reset and before the data cache is enabled in the warm boot path, and
  only when the platform is not hardware-coherent (`HW_ASSISTED_COHERENCY ==
  0`); load/store-exclusive spinlocks need cacheable memory; the bakery lock
  needs only plain loads and stores" (F2, R22, dev-13).
- **Quotes (verbatim):** "I feel that I did not invent the bakery algorithm, I
  discovered it." (publications page); "the bakery algorithm, it just seemed to
  come out of thin air to me … perhaps that's why I'm proudest of it" (CHM oral
  history, pp. 9–10, F4).
- **Ruth:** "reappears" rather than "became" for the clock lineage (CHM p. 9
  supports the counter link).
- **Title (dev-17):** "out of thin air" means OOTA values to a
  memory-model reader, so the chapter is named for Lamport's pride instead;
  the phrase stays inside his quote.
