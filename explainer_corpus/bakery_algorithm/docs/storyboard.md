# Storyboard — The Algorithm Lamport Is Proudest Of

The hand-edited source of this video's words and pictures (from R1, revised at
R2, 2026-10-11; it started as plan §11.4, revision 2). `python3 -I
tools/storyboard.py bakery_algorithm` checks it (gate G2) and writes
`script.tsv` from the words column. Facts: `docs/fact-sheet.md` and
`oracle.py`; brief: `docs/brief.md`; the exact versions, sources, code and
costs that the words leave out: `docs/description.md` (the draft video
description; every "description" below means that file); reviews and their
dispositions: `docs/reviews/bakery_algorithm/`.

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
step), **slate** (a register; also the spoken word for a machine integer's
storage, "a thirty-two bit slate"), **ticket** (never "number"), **customer
number** (always in full: bare "number" appears only in "take a number"; R2 M3,
R8; customers are named by colour aloud), **pair** (a ticket with its
customer number, defined aloud at 04_4), **glance** (a read), **flag**,
**choosing** (the time a flag is up; Lamport's "doorway" appears once, on
screen), **served** (going into the room), **outbox** (a store buffer; the
real name once, on screen, at 12_2). **Atomic** is never spoken loosely: the
02_7 lower-third defines it once ("a step nobody can catch halfway is called
atomic"), 12_7 says "atomic reads and writes", and the magic signs of 02_7 and
13_1 are named on screen as atomic read-modify-write instructions (R2 R1).

## 01_one_plus_one_is_one (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | A slate reads nine. Someone is rewriting it as ten. Glance at it mid-stroke, and you might read nine, or ten, or, Lamport says, two thousand four hundred ninety-six. | A chalk slate frozen mid-stroke; flickering digits; quote card (Lamport's Turing Award lecture, CACM 58(6), 2015; 2496 set in full-size digits, F8) | flicker prop |  |
| 01_2 | Could computers take turns safely with memory like that? In nineteen seventy-three, that was considered impossible. | Two lanes reach for the same slate; a dated card "1973: considered impossible" (Turing lecture, verbatim) | quote card |  |
| 01_3 | *(title)* | **The Algorithm Lamport Is Proudest Of** | title card | listen 3 s (the "take a number" motif) |
| 01_4 | Two programs, running at once, share a scoreboard that reads forty-one. Programmers call it x, and write x equals x plus one. It isn't an equation. It means make x one bigger. | Scoreboard; it takes the label "x" as the voice names it (M21); two lanes, Blue and Green; "time runs left to right; a read is an arrow in, a write is an arrow out" | **space-time lanes** |  |
| 01_5 | That's two steps. Read the score, then write it back plus one. | Arrows on the lanes | step animation |  |
| 01_6 | Here's one unlucky order. Both read forty-one, and both write forty-two. Two points scored, one counted. | x ends at 42 | step animation |  |
| 01_7 | ⟨predict⟩ Draw every order at once, as staircases from corner to corner. There are six. How many lose a point? | The lanes morph into the progress square: Blue's lane becomes the bottom axis, Green's rotates up to become the side; the order just shown becomes the first staircase, then the other five draw in; lower-third "6 = C(4, 2): pick which 2 of the 4 steps are Blue's" (`C-F1.binomial_4_choose_2`; R14) | **progress square** |  |
| 01_8 | Four. Exactly the ones through the middle, where both have read and neither has written. The fix has to wall off the middle. | The 4 paths through (1,1) turn red | highlight |  |
| 01_9 | Counting orders isn't measuring chances. Each program usually runs both steps back to back, so a bad order needs unlucky timing. But at billions of steps a second, unlucky timing happens constantly. | The six orders shrink into one long strip of steps racing past; red flecks appear along it again and again; lower-third "6 orders, 4 bad: a count, not a probability" | time strip | hold |

- **For programmers (lower-third):** even `add [x], 1` reads and writes
  separately unless it carries a lock prefix.
- **Ruth:** six orders = C(4,2) interleavings of two 2-step programs (R11; on
  screen at 01_7 since R2, R14).
- **01_9 (R2 dev-06):** no frequency claim: nothing in the repository measures
  how often a hot counter loses updates (bead mx-e-c-7v0.17 is the measurement).

## 02_the_room (~80 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | The fix seems obvious. Only one program at a time may touch the scoreboard. Call that stretch of code the room. | A room around the scoreboard; lower-third "critical section; the rule is called mutual exclusion" | room prop |  |
| 02_2 | Put a sign on the door. Check that it says vacant, flip it to occupied, and go in. | Sign | sign prop |  |
| 02_3 | ⟨predict⟩ Does the sign work? | The two lanes walk up to the sign | prediction card |  |
| 02_4 | No. Both read vacant before either flips it. Of the fourteen orders, eight put both in the room at once. | Progress square with a red square "both inside"; the 14 staircases draw in and a counter ticks the 8 that cross it; lower-third "counting each read, write and successful wait as one step" | progress square |  |
| 02_5 | Dijkstra posed the problem properly in nineteen sixty-five. Never two in the room. Someone who stops outside the room can't hold up anyone else. And fast or slow, it still works. | Name card "Edsger W. Dijkstra, 1965" (the first name only on screen: lexicon, ftts reads it 'edger'); three pictograms on the lanes, numbered 1–3, parked in a corner: the red square; a dimmed lane outside the room that the other passes; lanes stretching in time | pictograms |  |
| 02_6 | A year later, Donald Knuth asked for more. Everyone who wants in gets in eventually. | Pictogram 4: every lane eventually reaches the room | pictograms |  |
| 02_7 | Hardware usually cheats, with a magic sign you can check and flip in one motion. Dijkstra showed that ordinary signs are enough, several of them used cleverly, as long as nobody can catch one halfway through a flip. | The magic sign, then several ordinary signs (never the single sign that just failed); lower-third "a step nobody can catch halfway is called atomic" (R2 R1: the one definition of the word) | statement card |  |
| 02_8 | Now a harder question. What if a glance can catch a sign mid-flip, as a blur? Hold that thought. | A sign blurs | flicker | hold |

- **Ruth:** the four rules as Lamport states Dijkstra's problem "as extended by
  Knuth"; "atomic" is defined once (the 02_7 lower-third) and never used
  loosely. Fact-sheet row C-S1 quotes Dijkstra 1965's conditions verbatim
  (R9: his rule (d) already required that somebody gets in, so Knuth's
  "everyone" is more); the words above make no claim either way. 02_5's
  second rule is Dijkstra's (c), "not allowed to lead to potential blocking of
  the others", hence "anyone else" (R2 dev-07, R12); the description quotes
  (a)–(d).
- **Pictograms (creative C-12):** rule 1 is ticked at 08_8 ("never two in the
  room: done"), rule 4 at 11_2.

## 03_take_a_number (~70 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 03_1 | In nineteen seventy-four, Leslie Lamport found an answer. Picture a bakery. Take a number, and the lowest ticket is served next. From now on, each program is a customer, and being served means going into the room. | Dated card "Lamport, 1974" (`C-F14.bakery_paper.year`; M20); the two lanes get customer tags (Blue 1, Green 2); three more lanes join (Purple 3, Teal 4, Pink 5); the badges appear only after "the lowest ticket" is said, and the first chalk tickets are drawn beside the counter as it is said (M3); the room morphs into the serving counter; lower-third "programmers say process" | scene morph |  |
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
| 04_2 | Yes. Before anyone is served, Purple and Teal read at the same moment. Both see four as the biggest, and both write five. | Both read, both write 5 | step animation |  |
| 04_3 | Our rule only waits for smaller tickets, so both can end up in the room at once. Change the rule to wait for anyone with a smaller or equal ticket, and they both wait, bowing to each other forever. After you. No, after you. | Two traces: both in the room ("can": from 04_2's slates some orders put both inside and some never do; R5); then two customers bowing | traces | listen 3 s (two soft notes alternating politely, never resolving) |
| 04_4 | The fix is a tie-breaker. Pair each ticket with its customer number, and compare tickets first. Only if they're equal, compare customer numbers, the way a dictionary checks the second letter only when the first letters match. Purple is three and Teal is four, so Purple goes first. | (5, Purple 3) before (5, Teal 4): each pair drawn as a chalk ticket clipped to a coloured badge, never two bare numerals; the word "pair" appears under the first one as it is said (M5, R10); two dictionary words, "cat" before "cow", share a first letter beside them | ordering card |  |
| 04_5 | ⟨predict⟩ Put every pair on a grid, ticket across and customer number up, and sweep column by column. Blue is customer number one. Does she go before Pink and Green? | Lexicographic grid holding this chapter's five pairs, (4, Blue 1), (3, Green 2), (5, Purple 3), (5, Teal 4), (2, Pink 5), the slates still on screen since 04_2 (M4); the gold scanline waits before column 2 | **lexicographic grid sweep** |  |
| 04_6 | No. Blue's four still waits for Pink's two and Green's three. The customer number only breaks ties. The room always takes the smallest pair of everyone holding a ticket, so it goes Pink, Green, Blue, Purple, then Teal. | The scanline sweeps column by column: (2, Pink) → (3, Green) → (4, Blue) → (5, Purple) → (5, Teal) (`C-F7.model.tie.service_order` = 5,2,1,3,4; "always": `C-X4.fact_two_all_holders.*.violations` = 0); lower-third "precisely: pairs are totally ordered, and each has finitely many ahead of it" (R14) | grid sweep | hold |

- **Ruth:** ≺ is a strict total order on ℕ × {1…N} (customer numbers
  distinct); every pair has finitely many predecessors (04_6 lower-third). 04_6's
  claim is about customers holding tickets at the same time: a later run can
  serve a smaller pair after a bigger one once the first has left (R13;
  description, "Precisely" 04_6).
- **R2 (M4):** 04_5–04_6 use the chapter's own slates, so Blue keeps the four
  she wrote at 03_5; the earlier "Blue holds three, Teal holds two" example
  contradicted it. "The room always takes the smallest pair" previews fact two
  (08_3) and rests on the same edge test, C-X4.
- **Facts:** 04_2's tickets are model-checked (`C-F7.model.tie.*`: from 4,
  3, 0, 0, 2 Purple and Teal both read 4, 3, 0, 2 and write 5; every order
  then serves Pink, Green, Blue, Purple, Teal); the replayed equal-ticket
  traces c (both take 1) and c′ (both see 1, both take 2) are the small cases.

## 05_the_unwritten_ticket (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 05_1 | There's a sneakier crack. Blue reads Green's slate, sees zero, and decides on ticket one. But she hasn't written it yet. | A fresh start: all five slates wipe to zero and Purple, Teal and Pink dim (M11); then a ghost "1" hovers over Blue's slate | step-through |  |
| 05_2 | Green takes ticket one, checks Blue's slate, sees zero, which means not in line, and walks in. | Green enters | step-through |  |
| 05_3 | Then Blue writes her one, sees a tie, wins it on her lower customer number, and walks in too. Both in the room. | Both inside | trace (a), 6 counted steps (`C-F6.a.steps_counted`; dev-10) | listen 3 s (two tunes at once: the bug) |
| 05_4 | The fix is a flag that says, I'm choosing, don't judge me yet. Raise it before reading, and lower it after writing. Before comparing with someone, wait until their flag is down. | Amber flags | flag prop |  |
| 05_5 | ⟨predict⟩ Same unlucky order, now with flags. What does Green do this time? | Scheduler slider replays the order: trace a's six steps in place, the flag steps slotted in, amber; any step count shown is 13, counted the 02_4 way (`C-F6.a_prime.steps_counted`; dev-10) | toggle replay |  |
| 05_6 | She sees Blue's flag up, and waits. | Green waits | step-through |  |
| 05_7 | Without flags, two customers visiting once have twenty-eight orders, and four put both in the room. With flags, there are three hundred twenty-two, and not one does. | The 28 staircases as strands, 4 through the red square; they collapse into a "28 orders, 4 bad" card; the lattice regrows to 7×7; the 322 draw in as edge ribbons with a lagged reveal and a counter (spike S-6); the both-inside point stays empty and pulses | **path sets (S-6 ribbons)** |  |
| 05_8 | The time your flag is up is your choosing time. Unlike the door sign, the flag isn't a lock. Nobody waits to raise their own, and many can be up at once. | Choosing shaded on the lanes; 02_2's door sign returns beside the flag (M12); on screen once, "Lamport calls this the doorway"; "many at once": `C-X6.all_flags_up_at_once_reachable.*` = 1 | statement card |  |
| 05_9 | So two customers can still choose at once. The flag just shows everyone who's choosing, and the tie-breaker settles equal tickets. | Thesis card; two flags up together, then two equal tickets settled by their badges | statement card | hold |

- **Ruth:** C-F3 (no flag 28 / 4 bad; flag 322 / 0), C-F4 (three customers
  2,126,816,238 / 0, chapter 06); one visit each, a wait counted once, when it succeeds
  (lower-third at 05_7; D2). The tie-breaker settles equal tickets only; the
  flag and fact one make the other races harmless (R18). 05_9 (R2 M13): "two
  customers can still choose at once" is C-S7's first clause (equal tickets with
  flags, `C-F6.c.tickets` = 1,1) and `C-X10.choosing_overlaps.n2_one_visit.orders`
  = 272 of the 322.
- **Dev:** the flag is not a lock: nobody waits to raise their own, many can
  be up at once, and a flag stays up only for the steps it takes to write a
  ticket (dev-03).
- **Engine:** blocked moves depend on history, so paths are drawn, not walls.

## 06_three_customers (~30 s; the whole chapter on the camera route)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 06_1 | ⟨predict⟩ Now take three customers. The square becomes a cube. How many orders are there now? | The 7×7 square extrudes into a 10×10×10 cube; "two inside" beams, translucent, drawn last | **3D cube, orbit < π** |  |
| 06_2 | Over two billion. And not one of them puts two in the room. | Sampled staircases thread the cube as glowing trails (sampled uniformly over orders, E-C2) while a counter climbs to 2,126,816,238; the beams stay dark; caption "trails: orders drawn uniformly at random from all 2,126,816,238, not as a real machine would run them" (R15) | **3D cube, orbit < π** | hold |

- **Engine:** about 30 s on the camera route (≈ 5 min at S-5's 6.1 fps); about
  200 sampled polylines on the 10×10×10 lattice; beams are depth-tested
  translucent prisms drawn last; arcs under π (E-C2). Cut it first if the
  video runs long: 05_7's ribbons carry the claim.

## 07_one_run (~60 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 07_1 | Here's one full run, fast. | The five lanes with their slates at the heads, Pink's lane dimmed (she stays home: trace d has four customers; dev-10); a step clock counting reads, writes and successful waits the 02_4 way, ending at 64 (`C-F6.d.steps_counted`) (no code panel during the run, C-7) | lanes + step clock |  |
| 07_2 | Purple and Green arrive together, and both choose ticket one. Purple finishes first and finds Green's flag still up, so she waits. Without the flag, she'd have read Green's unwritten ticket as zero. | Fast-forward, slowing at the beat | step clock |  |
| 07_3 | ⟨predict⟩ Teal sees one and one. What ticket does she take? | Teal reading | prediction card |  |
| 07_4 | Two. And the tie between Purple and Green goes to Green, customer two, ahead of Purple, customer three. | (1, Green 2) before (1, Purple 3) | ordering card |  |
| 07_5 | Green goes in first. Blue, customer number one, arrives late and takes three. Green is out before Blue checks her, so Blue only waits for Purple, then Teal. Green gets back in line and takes four, behind Blue. | The lanes in time order, as trace d runs (M14): Green enters, Blue takes 3, Green leaves before Blue's check of her (`C-F6.d.customer2_out_before_customer1_checks_her` = 1), Blue waits on Purple's 1 and Teal's 2 (`C-F6.d.customer1_waits_behind` = (1,3)(2,4)), Green takes 4 (`C-F6.d.tickets_taken`); each entry plays its customer's "served" motif (C-2) | step clock | |
| 07_6 | Five visits, five points on the scoreboard, none lost. Who's served first follows the grid exactly. | (1, Green) → (1, Purple) → (2, Teal) → (3, Blue) → (4, Green), the grid sweep beside the lanes; chapter 01's scoreboard sits inside the room and has ticked once per visit, 41 → 46, no point lost (Maya m6; `C-F6.d.room_entries` = 5, `C-F6.d.max_in_room` = 1) | callback | hold |

- **Engine:** slate values swap by replacement transforms at their step events, driven by the shared step clock, never redrawn every frame (E-C4).
- **New beat 07_6 (R2):** 07_5 now runs in time order (M14), so its payoff
  moved to a beat of its own; 07_6 is the chapter's next free id.
- **For programmers (the description, "Code"; decided at R2, dev-08):** the
  text there, verbatim: "The owner's TypeScript lock compared pairs with
  `(t[j], j) < (t[i], i)`. In JavaScript that is the comma operator: each
  parenthesis evaluates both operands and keeps the last, so the line compares
  `j < i` and ignores the tickets (`C-F13.as_written_equals_j_lt_i_on_every_grid_case`).
  `tsc --strict` (TypeScript 5.9.3) reports nothing. Python's tuples compare in
  dictionary order (`C-F13.python_tuple_compare_equals_lexicographic_on_every_grid_case`),
  and so do Rust's (the 12_7 panel's `(nj, j) > (t, i)`)." Nothing is said
  about ESLint until it has been run on the snippet and recorded (bead
  mx-e-c-7v0.5).

## 08_why_never_two (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 08_1 | Why can two customers never be in the room at once? Picture Blue inside, and Green holding a ticket. | Lanes; Blue and Green; lower-third "the proof counts on: each customer's steps happen in order; a finished write is seen by every later read; the biggest ticket is found by reading each slate once" (M17: 12_1 calls it back in the same words) | lanes |  |
| 08_2 | Fact one. If Blue finished choosing before Green started, and still holds that ticket, then Green's ticket is bigger. Green saw Blue's. | Fact one card, in colours | card |  |
| 08_3 | Fact two. If Blue is in the room, everyone else holding a ticket has a bigger pair than hers. Why is that true? | Fact two card | card |  |
| 08_4 | ⟨predict⟩ Take the last time Blue checked Green's flag, the check that let her through. Where could Green have been? | Green's choosing interval slides along the time axis against Blue's last flag check (labelled) | **sliding interval with zones** |  |
| 08_5 | Three places. Not yet choosing. Blue had already finished choosing, so by fact one, Green's ticket will be bigger. | Zone "not yet"; Blue's finished choosing interval lights up to the left of her check (M15) | zones |  |
| 08_6 | In the middle of choosing. Then Green's flag was up, and that check wouldn't have let Blue through. | Zone "choosing", forbidden | zones |  |
| 08_7 | Finished choosing. Then Blue read Green's real ticket, and went in only because her pair was smaller. Either way, Green's pair is bigger. That's fact two. | Zone "done" | zones |  |
| 08_8 | Now suppose both were in the room. Then fact two says each one's pair is smaller than the other's. Like me being taller than you, and you taller than me. Impossible. | 04's grid returns: the scanline tries to reach both pairs first and tears; then (n_B, Blue) < (n_G, Green) < (n_B, Blue); pictogram rule 1 ticks | TransformMatchingTex | hold |
| 08_9 | A computer search agrees, over every order, up to four customers, as long as tickets stay small. Only the proof covers every case. | Card "N = 4, tickets ≤ 4 (runs that would pass 4 are cut), indivisible reads and writes: 3,027,633 reachable states, no violation", then "every N" | card |  |

- **Precisely (description, "Precisely" 08):** Lemma 1 and Lemma 2 exactly as
  in memo §4, corrected at R2 (R3): case (b) "the write that ends choosing began
  before the check ended", Bᵢ ending where the exit write *begins*, one
  definition of "in the bakery" used in both lemmas, and the bypass corollary
  counted from the end of the ticket write; Assumption M (the max reads each
  slate once). Fact two is stated for everyone holding a ticket, flag still up
  included, and tested at that edge (`C-X4`).
- **R2:** 08_5 now says why fact one applies (M15); paid for by 08_9's shorter
  last sentence, so the chapter stays under the cap.
- **Music:** swing drops out for 08–10 (rubato, a pedal under the lemmas).

## 09_reading_garbage_safely (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 09_1 | Remember the slate caught mid-stroke? Rewrite oh-nine as ten one digit at a time, and a glance in between reads nineteen, or zero zero. | Two-digit rewrite, both write orders | flicker |  |
| 09_2 | Lamport went further. A glance during a write may return any number the slate could hold. Even two thousand four hundred ninety-six. | Wild values; the 01_1 slate's 2496 | flicker |  |
| 09_3 | Twelve years later he named memory like that safe. A funny name, since it's the least safe kind. | "safe" card; "regular" and "atomic" on screen for Ruth | register zoo (screen) |  |
| 09_4 | ⟨predict⟩ Back to the proof picture. Let every slate and every flag flicker while it's written. Does the proof ever trust a glance at a slate while it's being written? | 08's zones (08_4–08_7), same layout and scale (C-16; M22); glances that cross a write become white static | **overlay on the proof zones** | listen 3 s (a read that crosses a write smears its note with a slow tape bend) |
| 09_5 | Never. It trusts just two glances at slates. Green reading Blue's finished ticket, and Blue reading Green's finished ticket. Nobody was writing either slate. | The two trusted reads in gold; static lands in every zone, always on reads the proof never used; lower-third "a glance at a flag mid-raise counts as not yet choosing; mid-lowering, as finished" (R2, dev-11; pinned here at R2, R13) | zones |  |
| 09_6 | A glance that catches garbage, say seven, just makes a customer take eight and wait longer. | Trace e1 inside the zones; corner card "model: flickering slates and flags, 3 customers, tickets ≤ 6: 269,550 states, never two in the room" (`C-F8.n3_k6_safe.states`, `.violation_found` = 0; R14) | trace |  |

## 10_the_old_zero (~55 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 10_1 | ⟨predict⟩ And a glance that catches Blue's old zero mid-write lets Green straight in. Is that dangerous? | Trace e3 | prediction card |  |
| 10_2 | No. Blue's flag is up while she takes a ticket, and Green had already seen it down. So Blue raised it after Green finished choosing. That's fact one the other way round, so Blue's ticket will be bigger, and she waits. | Blue's choosing begins after Green's ends: in trace e3 Green finishes choosing, then sees Blue's flag down, and only then does Blue raise it (`C-F6.e3.customer2_finished_choosing_at_step` = 14, `…saw_customer1_flag_down_at_step` = 17, `…customer1_began_raising_flag_again_at_step` = 21); the fact-one card from 08_2 returns with Blue and Green swapped (M6) | trace |  |
| 10_3 | Lamport noticed that the algorithm survives garbage only after writing the proof. In his words, the read may return any arbitrary value. His colleague Anatol Holt refused to believe it. | Quote cards, one at a time, one source each (M1): "The read may return any arbitrary value!" (CACM 17(8), 1974, p. 454); "I discovered that the bakery algorithm had this property after writing a proof of its correctness" and "He refused to believe it could be true" (Lamport, publications page) | quote card |  |
| 10_4 | One caution, from Lamport himself. Find the biggest ticket by reading each slate exactly once. The obvious way reads a slate twice, once to compare and once to copy, and a flicker between them can break it. | The two-line max loop that reads twice; then only the outcome: two lanes in the room, two tunes clashing (C-13); card "Lamport, Reasoning About Nonatomic Operations, POPL 1983" (C-S20) | trace |  |
| 10_5 | So, could computers take turns safely with memory like that? Yes. | The 01_1 slate flickers 2496 beside a calmly waiting customer; the 01_2 question returns as text, then "Yes." (M1) | statement card | hold |

- **Lower-thirds:** "today's chips don't tear an aligned word; the proof just
  doesn't need that" (dev-13). The flag-glance lower-third moved to 09_5 at R2
  (R13).
- **New beat 10_5 (R2, M1):** the opening question answered aloud, after the
  caution, so the chapter ends on "Yes", not on the clash; 10_5 is the
  chapter's next free id.
- **Ruth:** safe / regular / atomic per "On Interprocess Communication"
  (1986); the safe-register model check (N ≤ 3; 269,550 states at tickets ≤ 6,
  zero violations) is evidence, the proof is the reason (on screen at 09_6
  since R2). Lamport's own 1974 single-bit argument treated flags as atomic, a
  fallacy he later acknowledged; the algorithm stands (description,
  "Precisely" 09; C-S18). e3's zero is the slate's value before the write (F3):
  the point is that the proof never needed it.
- **Dev:** on a real machine garbage can be enormous (a 64-bit ticket written
  in two halves), which is why the overflow bound in 11 assumes reads never
  come back too big: in the description under chapter 11, with Java's plain
  `long` as the sourced case (C-S22, JLS §17.7).

## 11_fair_at_a_price (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 11_1 | It's also fair. Finish choosing your ticket before I start choosing mine, and you're served first. If we choose at the same time, either of us may go first. | First-come-first-served on the lanes (`C-X2`; "either": `C-X10.choosing_overlaps.n2_one_visit.first_to_enter_can_be` = 1,2) | lanes |  |
| 11_2 | As long as everyone in the room comes out, and nobody stalls for good with a raised flag or a ticket, whoever holds the smallest pair gets through. Once you've taken your ticket, each other customer can get ahead of you at most once. So nobody waits forever. | Bounded bypass (`C-X5.bypass_from_ticket_write.*.worst` = 1); the smallest pair walks in (`C-X3`); pictogram rule 4 ticks on "forever" | lanes |  |
| 11_3 | The price is that tickets keep growing while somebody is always in line. They reset only when the bakery empties. | Odometer 1, 2, 3, 4, 5, 6 (digits slide and fade; no clip primitive, E-C3); "only": `C-X7.largest_ticket_drops_to_nonzero.*` = 0 | odometer |  |
| 11_4 | ⟨predict⟩ Lamport did the sum. At one customer a millisecond, how big do the tickets get in a year? | 31,557,600,000 ms in a 365.25-day year (R6) | prediction card |  |
| 11_5 | Up to about thirty-one and a half billion, one per millisecond. That's too big for a thirty-two bit slate. Like the odometer, it could roll over in under two months. | 31,557,600,000 < 2³⁵ = 34,359,738,368, Lamport's bound (`C-F10.ms_per_year_billions_to_nearest_half` = 31.5); 2³² ms = 49.7 days, 2³¹ ms = 24.9 days (signed, as in Java's `int`); the 11_3 odometer rolls over (M7, R6) | arithmetic card |  |
| 11_6 | A sixty-four bit slate lasts centuries, even at a billion tickets a second. That sum assumes a glance never returns more than was really written. One huge glance makes the tickets after it huge too. | 584.5 years at 10⁹/s unsigned; 292.3 signed, as in Java; lower-third Lamport 1974 verbatim: "a read of number[i] can never obtain a value larger than one which has been written there"; one slate flickers to a huge value and the odometer jumps (M2) | arithmetic card |  |
| 11_7 | And it isn't cheap. Even alone, a customer makes three reads for every other customer. One customer frozen while choosing stalls everyone. | Three beats on the lanes: spinning (a waiting lane fills with soft repeated notes); a customer frozen mid-choosing, every other lane stalled behind it after at most one more visit each (`C-X8.frozen_while_choosing.*.worst_entries_per_other_customer` = 1); two slates in one grey cache-line box bouncing between cores. The costs card slides in from the right at the start of 11_7 and stays through the listen window, at least 10 s | lanes, costs card | listen 3 s (spinning) |

- **Costs card (for programmers; decided at R2, dev-03).** On screen, with a
  "for programmers" tag, about 30 words:
  "Customers fixed in advance · Paused in line → everyone behind waits · A dead
  customer's slate stays set: resetting it is your job · Pad each slate to its
  own cache line · 64-bit tickets (Java: `AtomicLongArray`)". "Everyone behind
  waits" is `C-X9.paused_in_line.*.entries_by_customers_behind_her` = 0. The
  rest is in the description, "Costs": the 3(N−1) breakdown, 2N shared
  variables, Lamport's reset assumption (C-S19), the frozen chooser and the
  paused customer with their keys. Burns & Lynch's lower bound is off the
  screen until its theorem is read (dev-09).
- **Precisely (description, "Precisely" 11):** "if i finishes choosing before
  k starts choosing, i enters first"; FCFS alone is not enough, since it holds
  vacuously for the `≤` rule that deadlocks: first-come-first-served plus
  deadlock freedom gives lockout freedom (Lamport 1986, Theorem 1; C-S24; R7);
  the bypass bound holds once a customer has taken her ticket (R16, C-X5); the
  double-fetch failure needs three or more customers and reads that can return
  garbage (C-F6, 33-step trace; atomic reads give no violation,
  `C-F6.double_fetch.n3_k5_atomic.violation_found` = 0; "can break", R16).

## 12_your_cpu (~85 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 12_1 | The proof is right, but run as written on your laptop, the algorithm can fail. A finished write isn't always seen by every later read. | 08_1's lower-third returns, "the proof counts on" (M17, M22); "a finished write is seen by every later read" flickers and breaks over a CPU outline (R8) | promise card |  |
| 12_2 | Your laptop has several processors, called cores. Picture each one running a customer. Each core drops its writes in an outbox and keeps going. | Store-buffer outboxes appear beside two lanes; lower-third "programmers say store buffer" (dev-11; C-S10) | **outbox x-ray** |  |
| 12_3 | ⟨predict⟩ Both customers raise their flags and write their tickets, into their outboxes. Then each checks the other's flag and slate. What does she see? | Both outboxes full; the real flags and slates still zero; the order animated is the voiced one (`C-F9.voiced.*`: 12 steps, nothing delivered; R11) | **outbox x-ray** |  |
| 12_4 | No flag, and no ticket. Both walk in, while the real flags and slates still say nobody is even choosing. | Both read arrows land on zeros (flag 0, then ticket 0); both enter (`C-F9.voiced.steps` = 12, `C-F9.voiced.in_room_at_end` = 2); memory frozen at zeros (`C-F9.voiced.memory_choosing_at_end`, `…memory_tickets_at_end` = 0,0) | outbox x-ray | listen 3 s (two tunes at once) |
| 12_5 | The fix is a fence. Wait until your outbox is delivered before reading anyone else's slate. On an Intel or AMD chip, two fences each time you enter are enough, one after raising your flag, and one after lowering it. | A fence drains a tray with a whoosh; the two fence points marked F1 and F2 on the lanes; lower-third "x86: MFENCE, or a locked instruction such as XCHG" (dev-11; C-S10) | fence |  |
| 12_6 | Arm chips, like the ones in recent Macs, shuffle more than an outbox does. Others can see your flag come down before your ticket lands, which is chapter five's bug again. | Arm outline; Blue's outbox delivers the flag letter before the ticket letter; Green sees no flag and no ticket and walks in, as at 05_2 (dev-02, M19; C-S11, C-S25) | statement card |  |
| 12_7 | For programmers, the easy, correct way everywhere is sequentially consistent atomic reads and writes. Acquire and release are not enough here, because this needs your write to land before your next read. | SeqCst Rust panel (below) with a "for programmers" tag (M9); highlighted while the voice says it, marked F1 and F2 like the 12_5 lanes (dev-04): F1, line 3 (`choosing[i].store(true)`) → line 5 (the max's loads); F2, lines 6–7 (the ticket store and the flag-down store) → line 9 (`choosing[j].load`); held at least 6 s; card: C, C++ and Rust plain shared variables are undefined behaviour and `volatile` doesn't help; Java `AtomicLongArray` works, a `volatile long[]` doesn't; then the panel slides away and the picture holds on the outbox draining. The panel is in the description verbatim | code panel | hold |

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
  matches Abdulla et al., TACAS 2012" (R15; C-S21: their minimal fence sets are
  one fence after raising the flag and one after either the ticket write or
  lowering the flag); "SeqCst is the easy way, not the cheapest: on x86 it
  fences every store" (dev-06).
- **12_6 (R2, dev-02):** what is spoken rests on sources: Arm lets another core
  see two writes in the opposite order (C-S11, C-S25: message passing is allowed
  on Arm, and the writes "might also be propagated to Thread 1 in either
  order"), and a flag seen down with an unwritten ticket behind it is chapter
  05's case. The voice does not claim a fence count for Arm. The description
  carries Dev's model, labelled as a model and not as Arm: an outbox that may
  deliver writes to different slates out of order breaks the two x86 fences
  (`C-F9.ooo_outbox.f1_f2.violation_found` = 1, 22 steps). R1's dev-10 entry
  ("there the fences go in more places") was never spoken; R2.md supersedes it.
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
| 13_1 | ⟨predict⟩ In practice, people use the hardware's magic signs, which check and flip in one motion. So does the bakery still run anywhere? | The magic sign from 02_7; lower-third "atomic read-modify-write instructions: test-and-set, compare-and-swap, fetch-and-add" (R1); quote card "Although of little if any practical use, the bakery algorithm…" (Lamport, Turing lecture, CACM 58(6), 2015, §2.2; C-S26; D5) | prediction card |  |
| 13_2 | Yes. Code that Arm itself publishes uses it while a core wakes up, before its fast memory is switched on. Until then, the magic signs aren't guaranteed to work, so the core takes a ticket, bakery style. | Text card: Trusted Firmware-A (TF-A): a bakery lock for a core waking with its cache off (no quotation marks, no logos; dev-12); "fast memory" is the data cache (M8, F2) | card |  |
| 13_3 | And its trick of taking a ticket bigger than any you've seen, with customer numbers to break ties, reappears in how Lamport later ordered events across whole networks. It's in the ballot numbers of Paxos too, the method that keeps copies of a database agreeing. | 04's grid relabelled (clock value, process); timeline 1974 → 1978 → Paxos | callback |  |
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
  supports the counter link, with Lamport's own hedge on Paxos, "not quite in
  the same way").
- **R2 (M8):** 13_2's words follow fact-sheet F2 ("the chip's usual atomic
  instructions aren't guaranteed to work"): the magic signs are 02_7's, and
  "fast memory" is the data cache; "Trusted Firmware", "data cache" and
  "spinlock" stay on the card and the lower-third.
- **The description** carries Levin's question and Lamport's "No, I can't."
  (CHM p. 9, C-S27; D3) and Lamport's "emasculated versions" passage (C-S28;
  D4); 13_4's picture stays uncluttered.
- **Title (dev-17):** "out of thin air" means OOTA values to a
  memory-model reader, so the chapter is named for Lamport's pride instead;
  the phrase stays inside his quote.
