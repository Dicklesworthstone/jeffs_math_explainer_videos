<!-- R2 script review, 2026-10-11, from the reviewer's final answer (packet tools/review.py; script at 85fbff2) -->

# Maya, 14: R2 script review of bakery_algorithm (Video C)

I read the whole R2 packet as Maya: the brief, `script.tsv`, the storyboard and the fact sheet. I also re-read my R1 review and the R1 dispositions so I could see what landed. I didn't edit anything in the repo. My sums and two small enumerations ran in a scratchpad file outside it.

**Verdict:** no blockers, because I can say every chapter in one sentence. There are nine majors. The three that would change the video most:

- **M1:** the opening question is never answered out loud.
- **M2:** 11_6 sounds like it takes back chapter 09.
- **M3:** "number" means two different things.

Most of my R1 fixes landed well: colour names, "choosing" everywhere, "why is fact two true", the outbox in plain words, the tunes. Three slid back:
- bare "number" for the customer number (R1 M2/M3)
- "order" used for the queue (R1 m20)
- jargon at 13_2 (R1 M16)

## Comprehension check

| ch | what I'd tell a friend | where it wobbled |
|---|---|---|
| 01 | Two programs adding one to the same score lose a point if both read it before either writes it back. That's 4 of the 6 staircases, and at billions of steps a second it happens all the time, so the fix has to wall off the middle. | Felt clever at 01_7. Briefly wondered what x is (M21). |
| 02 | A vacant/occupied sign fails the same way, because both read "vacant" before either flips it. Dijkstra and Knuth wrote the rules a real fix must keep, Dijkstra did it with several ordinary signs, and the hard question is what happens if a glance catches a sign mid-flip. | Predicted "no" at 02_3 and felt clever. |
| 03 | Every customer has her own slate. To take a ticket you read everyone else's slate and write one more than the biggest, and the smallest ticket goes in next, so nobody writes on anyone else's slate. | 03_1: heard "lowest number is served next" while badges 1–5 appeared (M3). |
| 04 | Two customers can read the same biggest ticket and both write five, and then they'd both go in or bow forever. So ties go to the lower customer number. | Rewound at 04_5: Blue had four, not three (M4). Lost at "like a dictionary" (M5). |
| 05 | If Blue has picked a ticket but not written it, Green sees zero and walks in, and then Blue walks in too. A "choosing" flag fixes it, because nobody compares with you while your flag is up. | "Second crack", "lock" and "the gap" were never set up (M11–M13). |
| 06 | With three customers the square becomes a cube with over two billion orders, and not one puts two in the room. | Fun guess. |
| 07 | In a five-customer run the flag stops Purple jumping ahead of Green, the tie goes to Green, and everyone is served in the grid's order. | Stopped at 07_5: "lowest number … takes three"? And why doesn't Blue wait for Green? (M3, M14) |
| 08 | Two customers can't both be inside, because whoever's inside has a smaller pair than everyone else holding a ticket, wherever Green was when Blue last saw her flag down. Two people can't each be smaller than the other. | "Pair" was never defined (M5). At 08_5 I didn't see why fact one applies (M15). |
| 09 | Even if a glance during a write gives junk, even 2496, the proof only trusted two reads made when nobody was writing, so junk just makes you take a bigger ticket and wait. | 09_4: "crosses a write" made me miss the question (M16). |
| 10 | Catching Blue's old zero can let Green walk in, but Blue started choosing after Green finished, so Blue's ticket ends up bigger and she waits. Lamport only noticed this after his proof, and you must read each slate once. | Rewound at 10_2: fact one said *Green's* ticket is bigger (M6). I never heard "yes" to the opening question (M1). |
| 11 | It's fair (finish choosing first and you're served first, and nobody gets ahead of you twice), but tickets keep growing and every entry costs three reads per other customer. | Felt stupid at 11_5 (bits, "wrap", 2³⁵; M7). 11_6 sounded like it contradicted 09 (M2). |
| 12 | Laptops let each core park its writes in an outbox, so both customers see no flag and walk in. A fence waits for your outbox to be delivered before you read. | Clever at 12_3. Lost at 12_7 (M9). |
| 13 | The bakery still runs in Arm chips when a core wakes up, its "bigger ticket, numbers break ties" trick came back in Lamport's later work and in Paxos, and he's proudest of it because it came out of thin air. | Jargon at 13_2 (M8). Loved 13_4. |

## Rubric

"Correctness" for me means: does it sound right, and does it agree with itself? The real maths is Ruth's job. Pacing is my estimate at 150 words a minute, from the word counts in `script.tsv`, before holds.

| ch | clarity | correctness | motivation | pacing | delight | why |
|---|---|---|---|---|---|---|
| 01 | 4 | 5 | 5 | 4 | 5 | Best opening minute. x is never tied to the scoreboard. |
| 02 | 4 | 5 | 4 | 4 | 4 | The sign fails as I predicted. The pictograms carry 02_5's three rules. |
| 03 | 4 | 5 | 5 | 5 | 5 | Clear and checkable, apart from "lowest number" over the badges. |
| 04 | 3 | 3 | 5 | 4 | 5 | 04_5 contradicts Blue's four. "Pair" and "dictionary" are never explained. 04_3 sounds circular. |
| 05 | 4 | 5 | 5 | 3 | 4 | The trace and the flag land. The chapter is full (189 words). |
| 06 | 5 | 5 | 4 | 5 | 5 | Short and huge. |
| 07 | 3 | 4 | 4 | 4 | 4 | 07_2 is great now. 07_5 is out of order. Only ~38 s spoken of ~60, so there's room. |
| 08 | 3 | 5 | 5 | 3 | 5 | The proof works for me except "pair". Fullest chapter (201 words), so fixes here must be swaps. |
| 09 | 4 | 5 | 5 | 4 | 5 | 2496 is paid off. "Crosses a write" is a new phrase. |
| 10 | 3 | 3 | 4 | 4 | 3 | Fact one is flipped. The hook's answer is never spoken. The chapter ends on a clash. |
| 11 | 3 | 3 | 4 | 3 | 4 | Fairness is clear. Number sizes are unexplained, and 11_6 sounds like it takes back 09. 182 words. |
| 12 | 3 | 4 | 5 | 4 | 5 | The outbox predict is the best "aha" in the video. 12_7 is spoken jargon. |
| 13 | 3 | 5 | 4 | 4 | 5 | 13_2 is a jargon wall. 13_4 is a perfect last line. |

**What I checked, and how**

- **In my head:**
  - 01_6: 41 + 1 twice is 42, so one point is lost.
  - 01_7–01_8: four of the six staircases (RRUU, RURU, RUUR, URRU, URUR, UURR) go through the middle.
  - 03_4–03_5: max(3, 0, 0, 2) + 1 = 4, and they're served Pink (2), Green (3), Blue (4).
  - 04_2: max(4, 3, 0, 2) + 1 = 5 for both Purple and Teal.
  - 04_4: (5, Purple 3) comes before (5, Teal 4).
  - 04_5: (2, Teal) comes before (3, Blue).
  - 07: Teal sees 1, 1 and takes 2; Blue sees 1, 1, 2 and takes 3; Green comes back and takes 4. The serving order goes column by column, and 5 visits take the scoreboard from 41 to 46.
  - 09_1: rewriting 09 as 10 one digit at a time gives 19 or 00.
  - 09_6: 7 + 1 = 8.
  - 11_7: three reads per other customer are her slate (for the biggest), her flag and her ticket.
- **With a small enumeration in my scratchpad:**
  - 02_4: the sign (read vacant, flip, leave) gives 14 orders, 8 with both inside.
  - 01: 6 orders, 4 lose a point.
  - 11_4–11_6:
    - a year is 31,557,600,000 ms, which is less than 2³⁵ = 34,359,738,368
    - 2³² ms = 49.7 days, which is under two months
    - 2⁶⁴ tickets at 10⁹ a second last 584.5 years (2⁶³: 292.3), which is "centuries"
- **From the research output:** I read trace d in `docs/research/bakery_algorithm/scripts/outputs/scenarios_out.txt` to check the 07_5 timing: Green goes in at step 29, Blue arrives at 30, Green comes out at 36, Blue waits on Purple at 49 and on Teal at 54, and Green takes 4 at 59.
- **Unchecked:**
  - 28/4 and 322/0 (05_7)
  - 2,126,816,238 (06_2; I only compared "over two billion" with the card)
  - 3,027,633 (08_9)
  - the 15 steps (05_5)
  - the 12-step outbox failure and "two fences are enough" (12_4–12_5)
  - Arm reordering (12_6) and the TF-A claims (13_2)
  - dates and quotes

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| M1 | maya_14 | 10_3, 10_4 (asked at 01_2) | major | The video opens with "Could computers take turns safely with memory like that?" The "Yes" only appears as text at 10_3, on a screen already holding three quote cards and a flickering 2496, while I'm listening about Anatol Holt. Then chapter 10 ends on 10_4's two clashing tunes, so my last feeling is "it breaks". Also, "Lamport noticed this" right after the old-zero case sounds like he noticed the old zero. | Put 10_4 before 10_3. In 10_3, say "Lamport noticed that it survives garbage only after writing the proof", and end the beat with the spoken line "So, could computers take turns safely with memory like that? Yes." Hold it over the 2496 slate and the calm waiting customer. Show the quote cards one at a time. In 10_4, say "the obvious way" instead of "the obvious loop". About +4 s, and 10 is ~46 s spoken against ~55. |
| M2 | maya_14 | 11_6 (against 09_2, 09_6) | major | 09_2 said a glance can return any number, "even 2496", and 09_6 said garbage just makes you wait longer. Then 11_6 says "As long as a glance never returns more than was ever written." It sounds like the video taking chapter 09 back, and nobody says why it matters here. | Give the reason: "As long as a garbage glance is never bigger than a real ticket, because one huge glance would make every ticket after it huge." Ruth to OK; it's the Dev note under 10. Pay for the time by moving "two to the thirty-five" in 11_5 to the card only. |
| M3 | maya_14 | 03_1, 04_6, 05_3, 07_4, 07_5, 13_3 | major | "Number" means two things. At 03_1, "the lowest number is served next" means the ticket, and it plays while badges 1–5 appear, so I first thought Blue (1) goes first. Later, bare "number" means the customer number: 04_6 "lowest number" and "The number only breaks ties", 05_3 "her lower number", 07_4 "the lower number", 13_3 "numbers to break ties". At 07_5, "Blue arrives late, with the lowest number, and takes three" stopped me: lowest number, but takes three? | Say "ticket" for the ticket, and always say "customer number" in full: <br>• 03_1: "the lowest ticket is served next" (keep "Take a number") <br>• 04_6: "Even customer number one…" and "The customer number only breaks ties" <br>• 05_3: "her lower customer number" <br>• 07_4: "goes to Green, customer two, ahead of Purple, customer three" <br>• 07_5: "Blue, customer number one, arrives late and takes three" <br>• 13_3: "with customer numbers to break ties" |
| M4 | maya_14 | 04_5–04_6 | major | Blue wrote four at 03_5, and it stayed on screen through 04_2. Then 04_5 says "Blue holds ticket three. Teal holds ticket two" as if it's happening now, so I thought I'd remembered wrong. 04_6 also calls the queue "the order in which the room takes everyone", but "order" is the staircase word (my R1 m20 again). | Use the chapter's own slates (4, 3, 5, 5, 2), which are already model-checked (`C-F7.model.tie.service_order` = Pink, Green, Blue, Purple, Teal). <br>• 04_5: "…sweep column by column. Blue is customer number one. Does she go before Pink and Green?" <br>• 04_6: "No. Blue's four still waits for Pink's two and Green's three. The customer number only breaks ties. So the sweep goes Pink, Green, Blue, Purple, then Teal. That's who's served first, of everyone holding a ticket." <br>Cheaper fallback: "Now suppose Blue holds three…", with the slates visibly wiped first. |
| M5 | maya_14 | 04_4 (used at 08_3, 08_7, 08_8) | major | "Pair" is the key word of the proof (08_3 "a bigger pair", 08_7, 08_8), but nobody says it before 08_3. I had to guess that it means ticket plus customer number, and that "bigger" means "later in the sweep". And 04_4's "like a dictionary" went over my head: how is a ticket like a dictionary? | 04_4: "The fix is a tie-breaker. Pair each ticket with its customer number. Compare tickets first, and only if they're equal, compare customer numbers, the way a dictionary looks at the second letter only when the first letters match. Purple is three and Teal is four, so Purple goes first." In 04_5, say "Put every pair on a grid". In 04_6, add "The sweep reaches smaller pairs first." About +6 s in 04, and nothing added to the full chapter 08. |
| M6 | maya_14 | 10_2 (against 08_2) | major | I learned fact one at 08_2 as "…then Green's ticket is bigger". At 10_2 I hear "By fact one, Blue's ticket will be bigger", which sounds like the opposite. And "So Blue started choosing after Green finished" jumps two steps in one "So", and I had to rewind. | 10_2: "No. Blue's flag is up while she writes, and Green had already seen it down. So Blue raised it after Green finished choosing. That's fact one the other way round. Blue's ticket will be bigger, so Blue waits." Bring the fact-one card back with Blue and Green swapped. Ruth to OK the middle sentence. About +3 s. |
| M7 | maya_14 | 11_4–11_5 | major | "Still below about thirty-four billion, two to the thirty-five. That's too big for a thirty-two bit number, which would wrap in under two months." I don't know what a thirty-two bit number is, what "wrap" means, or why two to the thirty-five. "Below about" is fuzzy too, when the question was "how big?". The odometer is right there at 11_3 and isn't used. | 11_5: "About thirty-one and a half billion. A thirty-two bit number tops out near four billion, so like the odometer it would roll back to zero in under two months." Keep 2³⁵ and 34,359,738,368 on the card. (31,557,600,000 and 49.7 days are checked above.) |
| M8 | maya_14 | 13_2 | major | "Reference firmware", "data cache", "spinlocks" and "rejoins the others", all in two sentences. It's the jargon wall I said I'd quit at in R1, smaller but still there, and it's in the last chapter. 13_1 just taught me "magic signs" for atomic instructions, and 13_2 doesn't use them. | "Yes. In the code Arm publishes for its own chips, at the moment a core wakes up, before its fast memory is switched on. Until then the magic signs aren't guaranteed to work, so it takes a ticket, bakery style." This follows fact-sheet F2's wording; Ruth to OK. Leave "Trusted Firmware", "data cache" and "spinlock" to the card and the lower-third. |
| M9 | maya_14 | 12_7 | major | "Sequentially consistent atomics" and "Acquire and release are not enough" are spoken to everyone. The storyboard's "for programmers" note isn't spoken, so nothing tells me it's OK not to follow. It's the last thing I hear in chapter 12, and it's where I'd feel stupid. | Start 12_7 with "For programmers, the easy, correct way…", and put a small "for programmers" tag on the code panel. |
| M10 | maya_14 | 04_3 | minor | "If equal tickets both go in, both end up in the room" sounds like saying the same thing twice. I didn't hear why they'd both go in. | "Our rule only waits for smaller tickets, so neither waits, and both end up in the room." |
| M11 | maya_14 | 05_1 | minor | "The second crack": the first one was never called a crack. The slates were 4, 3, 5, 5, 2 a moment ago, and now Blue reads Green's slate as zero with no reset. | "The second crack, after equal tickets, is sneakier." On screen, wipe all five slates to zero (and dim Purple, Teal and Pink) before Blue reads. |
| M12 | maya_14 | 05_8 | minor | "The flag isn't a lock." Nobody ever said what a lock is; 01_8 dropped the word after R1. | "Unlike the sign on the door, the flag isn't a lock." That keeps Dev's word and ties it to 02_2. |
| M13 | maya_14 | 05_9 | minor | "So the gap between looking and writing is still there." What gap? It was never named, and "still" sounds like someone tried to close it. | "So two customers can still choose at the same moment. The flag just shows everyone who's choosing, and the tie-breaker settles equal tickets." (The C-S7 keys back the first sentence.) |
| M14 | maya_14 | 07_5 | minor | "She waits for Purple, then Teal." Why not Green, who had ticket one? "Green leaves" comes after that, so the events sound backwards. And the scoreboard ticking from 41 to 46, the fix for the video's very first problem, is never said. | Put it in time order, which matches trace d (steps 29–70): "Green goes in first. Blue, customer number one, arrives late and takes three. Green comes out, so Blue waits for Purple, then Teal. Green gets back in line and takes four, behind Blue. Five visits, five points on the scoreboard, none lost." 07 has about 20 s spare. |
| M15 | maya_14 | 08_5 | minor | "Not yet choosing. Then by fact one, Green's ticket will be bigger." Fact one needs Blue to have finished choosing first, and nobody says she had. It felt handed to me. | "Not yet choosing. Blue had already finished choosing, so by fact one, Green's ticket will be bigger." It's +5 words in the fullest chapter, so find ~2 s elsewhere in 08. |
| M16 | maya_14 | 09_4 | minor | "Does the proof ever trust a ticket read that crosses a write?" "Crosses a write" is a new phrase for what the video has called a glance mid-stroke. This prediction is the video's main point, and I nearly missed what it was asking. | "Does the proof ever trust a glance at a slate while it's being written?" |
| M17 | maya_14 | 12_1 (08_1) | minor | "Breaks one of the proof's promises": the promises were never spoken. They were a lower-third headed "the proof assumes", so "promises" is new. | "…because your laptop breaks one thing the proof counts on, that a finished write is seen by every later read." Also head 08_1's lower-third "the proof counts on". |
| M18 | maya_14 | 12_3–12_4 | minor | "Each reads the other's slate. What does it see?" I predicted "zero", and the answer is "No flag". Then "the real slates still say nobody is even choosing", but choosing is the flag, not the slate. | 12_3: "Then each checks the other's flag and slate." 12_4: "No flag, and a zero. Both walk in, while the real flags and slates still say nobody is even choosing." |
| M19 | maya_14 | 12_6 | minor | "Arm chips … reorder even more." Nobody said chips reorder anything. I only learned "outbox". | "Arm chips, like the ones in recent Macs, shuffle their reads and writes even more than an outbox does, so they need more care." Ruth to OK. |
| M20 | maya_14 | 01_1, 03_1, 09_3 | minor | Nobody says who Lamport is or that the bakery is his, so "Lamport says" at 01_1 sounds like I'm meant to know him. "Twelve years later" at 09_3 counts from a year I never heard; only 1973 was spoken. | 03_1: "Leslie Lamport answered it in nineteen seventy-four. Picture a bakery. …" (C-F14 gives 1974.) That also anchors "twelve years later" at 1986. |
| M21 | maya_14 | 01_4 | minor | The scoreboard reads 41, then "Programmers write x equals x plus one." Nobody says x is the scoreboard. | Label the scoreboard "x" on screen as the line starts, or say "Programmers call the score x, and write…" |
| M22 | maya_14 | 09_4, 12_1 (picture column) | minor | Two callbacks point at the wrong chapter. 09_4's picture says "07's zones" and 12_1's says "07's promise card", but the zones and the promise lower-third are in 08 (07 is the run). If the builder follows those notes, the same object doesn't come back. | Change both to 08: the zones of 08_4–08_7, and 08_1's lower-third. |
| d1 | maya_14 | 01_4 | delight | "It isn't an equation. It means make x one bigger." Still lets me in during the first minute. | Keep exactly. |
| d2 | maya_14 | 01_9 | delight | "Counting orders isn't measuring chances … rare means constantly." This fixes my R1 "4 of 6 isn't rare" confusion in two sentences. | Keep. |
| d3 | maya_14 | 03_4 | delight | Three, zero, zero, two makes four. I got it before the answer, and since Blue is customer one, four isn't her customer number. | Keep, with the full prediction pause. |
| d4 | maya_14 | 03_6, 05_3 | delight | "Only one tune sounds at a time", then two tunes at once at 05_3: I heard the bug before it was named. | Keep both listen windows. |
| d5 | maya_14 | 04_3 | delight | "After you. No, after you." Still the funniest moment. | Keep (M10 only changes the first sentence). |
| d6 | maya_14 | 06_2 | delight | "Over two billion. And not one of them puts two in the room." A big wow for 10 seconds of words. | Keep the counter climbing. |
| d7 | maya_14 | 07_2 | delight | Now in time order, and "Without the flag, she'd have read Green's unwritten ticket as zero" tells me exactly what the flag saved. | Keep. |
| d8 | maya_14 | 08_8 | delight | Taller than you, and you taller than me: I get the contradiction instantly. | Keep. Tiny polish: "Like me being taller than you, and you taller than me." |
| d9 | maya_14 | 09_1–09_3 | delight | Nineteen or zero zero (I checked both ways), 2496 paid off at last, and "safe, the least safe kind" made me laugh. | Keep all three. |
| d10 | maya_14 | 12_3–12_4, 13_4 | delight | I predicted "no flag" from the outboxes before the narrator said it. And the last line, "out of thin air … proudest of it", ties back to the title. | Keep the frozen-at-zero slates and the 13_4 words exactly. |

The biggest single fix: say "Yes" to the opening question out loud (M1), and make "number" mean only the ticket's number while the person's is always "customer number" (M3). Most of my other rewinds come from those two.
