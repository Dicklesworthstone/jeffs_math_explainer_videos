# R3 review: bakery_algorithm, narrated 540p draft. Reviewer: Maya, 14 (maya_14)

**Verdict:** I'd watch this to the end and tell a friend about the bakery. Chapters 01 to 09 mostly work for me. I can predict things, check the numbers in my head, and the tunes are clever. Chapters 10 to 12 get dense: there are fast pictures, logic I have to rewind, and words nobody explains. I found no blockers. There are four majors (01_7, 07_5, 10_2, 10_4). Each one has a cheap fix in the wording or the picture.

## What I checked and how

- I read the persona prompt and the whole packet: the stage rules, the script and the description.
- I opened all 13 cue sheets once each, with their legends. Each tile is the frame 1.5 s into a line. Sheets 03, 04, 05, 06, 07, 08, 09, 10 and 11 end with padding tiles that repeat the last frame, and I ignored those.
- I read the chapter sources (on-screen strings, `wait_for` cues, beat comments) to see what appears later in a line than its tile shows. Examples: the 01_7 lower third, the 09_3 table, the 10_4 code, the 11_3/11_5 odometer, the 12_1 "points lost" card.
- **Numbers I recomputed:**
  - 01: 6 orders, 4 that lose a point (listed by hand).
  - 02_4: 14 orders, 8 with both inside. A quick in-memory enumeration of the 20 interleavings, where a check that finds "occupied" waits; no files written.
  - 03_4: 3, 0, 0, 2 gives four. 03_5 order: Pink 2, Green 3, Blue 4.
  - 04_2: both see 4 and write 5. 04_6 grid order: (2, 5), (3, 2), (4, 1), (5, 3), (5, 4), which is Pink, Green, Blue, Purple, Teal.
  - 07: Teal sees 1, 1 and takes 2. Blue takes 3. Green takes 4. 41 + 5 = 46.
  - 09_1: 09 becomes 19 (tens digit first) or 00 (ones digit first). 1974 + 12 = 1986. 09_6: 7 gives 8.
  - 11_5: 1000 × 86,400 × 365.25 = 31,557,600,000, which is under 2^35 = 34,359,738,368. 2^32 ms is 49.7 days, "under two months". 11_6: 2^64 ns is about 584.5 years. 11_7: 3 reads × 4 others = 12.
- **Numbers left unchecked:** 28 and 322 (05_7), 2,126,816,238 (06_2), 3,027,633 (08_9), 269,550 (09_6), 64 steps (07). These are model counts that I only listened to.

## Comprehension check

| ch | what I'd tell a friend |
|---|---|
| 01 | Two programs that both add one to a shared score can lose a point if both read before either writes. That happens in 4 of the 6 possible orders, so any fix has to block that "both have read, nobody has written" middle. |
| 02 | A vacant/occupied sign on the door fails, because both can check it before either flips it. Dijkstra set the rules a real fix must meet: never two inside, someone idle holds up nobody, it works at any speed, and (Knuth's addition) everyone eventually gets in. The hard version is doing this with signs that can be caught mid-flip. |
| 03 | In Lamport's bakery each customer has her own slate. To take a ticket she reads everyone else's slate, writes one more than the biggest, and waits for anyone with a smaller ticket. There is no shared ticket machine, and nobody writes on anyone else's slate. |
| 04 | Two customers who read at the same moment can get the same ticket. So the bakery compares (ticket, customer number) pairs like words in a dictionary, ticket first and number only to break a tie, which gives one clear order. |
| 05 | Blue can decide on her ticket before writing it, so Green sees her zero and walks in, and then Blue walks in too. An "I'm choosing" flag makes others wait until the ticket is written, and with flags not one of the 322 orders puts two inside. |
| 06 | With three customers the square of orders becomes a cube with over two billion orders, and not one of them puts two in the room. |
| 07 | In one real run, equal tickets happen, flags make an early customer wait, ties go to the lower customer number, and five visits add five points (41 to 46) in exactly the grid's order. |
| 08 | Whatever Green was doing when Blue last checked her flag, Green's pair ends up bigger than Blue's. So if both were inside, each would be smaller than the other, which is as impossible as each being taller than the other. |
| 09 | A glance at a slate that's being rewritten can return nonsense, but the proof never relied on such a glance (it trusts only glances at finished tickets), so garbage can only make someone wait longer. |
| 10 | Even when Green catches Blue's old zero and walks straight in, Blue started choosing after Green finished, so Blue's ticket comes out bigger and she waits. So yes, computers can take turns safely with flickery memory, if each slate is read only once when finding the biggest. |
| 11 | The bakery is fair: whoever finishes choosing first goes first, and nobody gets overtaken more than once. The price is that tickets keep growing while the line never empties (31.5 billion in a year, too big for a 32-bit slate), and each entry costs a lot of reads. |
| 12 | On a real laptop each core parks its writes in an outbox, so two customers can both see empty slates and walk in together. A fence (wait until your outbox is delivered) fixes it on Intel and AMD, and Arm shuffles even more. |
| 13 | Real computers mostly use the hardware's one-motion "magic sign" instead. The bakery still runs inside Arm's firmware while a core wakes up, and its "bigger ticket, break ties by number" trick shows up again in Lamport's later work. That's why he says he discovered it rather than invented it, and he's proudest of it. |

## Rubric

Correctness here means "nothing sounded wrong to me". Ruth checks the maths.

| ch | clarity | correctness | motivation | pacing | delight |
|---|---|---|---|---|---|
| 01 | 3 | 5 | 5 | 4 | 4 |
| 02 | 4 | 4 | 4 | 4 | 3 |
| 03 | 5 | 5 | 5 | 4 | 5 |
| 04 | 4 | 5 | 5 | 4 | 5 |
| 05 | 4 | 5 | 5 | 4 | 4 |
| 06 | 4 | 5 | 4 | 5 | 5 |
| 07 | 3 | 5 | 4 | 2 | 4 |
| 08 | 4 | 5 | 4 | 4 | 5 |
| 09 | 4 | 5 | 4 | 4 | 4 |
| 10 | 2 | 4 | 4 | 3 | 4 |
| 11 | 3 | 5 | 4 | 3 | 3 |
| 12 | 3 | 5 | 5 | 3 | 4 |
| 13 | 4 | 5 | 4 | 4 | 5 |

## Findings

| id | persona | chapter/line | severity | what confused, bored, misled or delighted me (in my words) | suggested fix |
|---|---|---|---|---|---|
| maya-01 | maya_14 | 01_7 (and 01_8) | major | "Draw every order at once, as staircases." Nobody tells me that a step across is Blue's and a step up is Green's, or that the unlucky order I just watched is one of the staircases. The lanes swing into the square's sides in about 1.6 s, six staircases appear, and then I'm asked "How many lose a point?" I can't answer, because I don't know how to read a staircase. The lower third "6 = C(4, 2)" uses a symbol I've never seen. This square comes back in 02_4 and 05_7 and as the 06 cube, so if I miss it here, those are just pretty pictures with numbers. | Before "There are six", add about 5 s: "Each step across is one of Blue's, each step up is one of Green's. The order we just saw is this staircase." Pulse the first staircase while the voice says it. Change the lower third to "6 ways to pick which 2 of the 4 steps are Blue's" and drop C(4, 2). |
| maya-02 | maya_14 | 07_5 (also 07_2) | major | 07_5 is about 18 s with four events: Green in first, Blue arrives late and takes three, Green is out before Blue checks her, Green comes back and takes four. Meanwhile the run races from step 23 to step 64. In the tiles the lanes are rows of tiny dots, the read values ("0 1", ":=2") are a few pixels tall on a phone, and the customers are little numbered circles now, not the blobs I've known since chapter 02. I followed the voice and not the picture, and I lost why "Green is out before Blue checks her" matters. | Split 07_5 into two lines with a hold between them. (a) "Green goes in first. Blue, customer one, arrives late and takes three." Hold the run while Blue's slate 3 pulses. (b) "Green is out before Blue checks her, so Blue waits only for Purple, then Teal. Green comes back and takes four, behind Blue." Hide the per-glance digit labels in the fast stretches (the slates carry the numbers), make the slates and badges bigger, and keep the blob faces at the lane starts. |
| maya-03 | maya_14 | 10_2 | major | Three "so"s in 14 s: "Green had already seen it down. So Blue raised it after Green finished choosing. That's fact one the other way round." I don't know when Green saw the flag down, and nobody says why that puts Blue's raising after Green's choosing (Green only checks flags after she's done choosing). The screen has the answer as step 14, 17 and 21 and Green's 8 against Blue's 9, but those labels are tiny on my phone and the voice never says them. I'd rewind this twice. | Say the numbers so I can put them in order in my head: "Green finished choosing at step fourteen and saw Blue's flag down at step seventeen. Blue only raised it at twenty-one, after Green was done. That's fact one the other way round: Green holds eight, so Blue's ticket comes out nine, and she waits." Make the step labels and the closing pair comparison about twice as big. |
| maya-04 | maya_14 | 10_4 | major | "The obvious way reads a slate twice." To me the obvious way is to look once and remember the number. The screen stays empty for the first couple of seconds, then shows code (`for k in others:`, `if number[k] > t:`). It's the first code since x = x + 1, and it calls my slate `number[k]`, a second name nobody gave me. Then a failure flies by on three lanes and I hear two tunes at once, but I can't say why reading twice broke it. That's about 19 s where I'm lost, right before the big "Yes." | Option (a): start with "For programmers, one caution:" like 12_7 and cut it to about 8 s. Option (b): show it on slates with small numbers and no code, for example: "Green is already inside with five, and Blue has seen it. Then she glances at Purple's slate mid-write: nine, bigger! She looks again to copy it: zero. Now her biggest is zero, so she takes one, smaller than Green's five, and walks in." Ruth should confirm this example is a real failure. If the code stays, label `number[k]` as "slate k". |
| maya-05 | maya_14 | 09_3 | minor | A three-row table (atomic / regular / safe), each with a definition, fades in over about 9 s while the voice says one sentence. Nobody says "regular". "Safe" is never used again, and it clashes with "take turns safely" in 01_2 and 10_5. I tried to read the rows and missed the joke. | Show only the "safe" row: "safe: a glance mid-write may return any number". Move regular and atomic to the description. Keep the "funny name" line. |
| maya-06 | maya_14 | 11_5 (11_3, 11_6) | minor | "Too big for a thirty-two bit slate. Like the odometer, it could roll over." I don't know what 32-bit means. "The odometer" sounds like one I was shown, but 11_3's counter was only labelled "biggest ticket". The wheels showing 4294967295 rolling to 0 help, if I can read ten small digits. | "A thirty-two bit slate only counts to about four billion. Then, like a car's odometer, it rolls back to zero, in under two months." In 11_6: "A sixty-four bit slate counts so high it lasts centuries." |
| maya-07 | maya_14 | 02_4 | minor | The 01_7 lower third taught me "pick which steps are Blue's". With three steps each that gives 20 orders, but the voice says fourteen, so it sounded wrong until I worked out that a check that finds "occupied" has to wait. | Add to 02_4's "counting:" lower third: "a check that finds 'occupied' waits, so 6 of the 20 ways to mix the steps can't happen." I checked this: 20 mixes, 14 possible, 8 with both inside. |
| maya-08 | maya_14 | 04_4 (03_1, 04_3) | minor | Customer numbers are only tiny digits on the blobs' bellies. In the 03 and 04 tiles I can barely read them, and the voice first mentions them at "Purple is three and Teal is four". Also, Pink, Green and Blue were served and wiped to zero in 04_3, then silently get their tickets back at the start of 04_4. | When the badges appear in 03_1, say: "Each customer also wears a number: Blue one, Green two, Purple three, Teal four, Pink five." Make the badges about 1.5× bigger. In 04_4, reuse 04_1's rewind slide, or add "Back to before anyone was served." |
| maya-09 | maya_14 | 08_5 (08_4 to 08_7) | minor | "Not yet choosing. Blue had already finished choosing." Why had she? Because she only checks flags after she's chosen. I can see it on the timeline, but the voice skips it. Also, the screen labels say "done choosing" while the voice and the fact cards say "finished choosing". | "Blue only checks flags after she's chosen, so she'd already finished." Change the label to "finished choosing". |
| maya-10 | maya_14 | 08_8 | minor | The formula (n_B, Blue) < (n_G, Green) < (n_B, Blue) uses n_B and n_G, letters nobody named. | Write "(Blue's ticket, 1) < (Green's ticket, 2) < (Blue's ticket, 1)", or plug in example tickets. |
| maya-11 | maya_14 | 05_8 | minor | "Lamport calls this the doorway" is a second name for choosing time, and it's never used again. Two names make me think there are two things. | Drop it from the screen and keep it in the description. |
| maya-12 | maya_14 | 12_6, 12_5, 12_7 | minor | "Chapter five's bug": I don't remember chapter numbers, I remember "the unwritten ticket". 12_7 says it's for programmers, which is fine, but its code card is unreadable at phone size even if I wanted to try. The 12_5 lower third (MFENCE, XCHG, x86-TSO, Abdulla et al.) is a wall of words. | Say "which is the unwritten ticket again". Show 12_7's code at a size a phone can read (fewer, shorter lines) or leave it to the description. |
| maya-13 | maya_14 | 06_2 (06_1) | minor | The key "red: two in the room at once" next to big red beams made me think for a second that some orders do put two in. | Change the key to "red: where two would be in the room". The voice and the "0 put two in the room" count already say that none do. |
| maya-14 | maya_14 | 11_2 | minor | The first sentence is a long "As long as… and nobody… whoever…" before I know what the claim even is. | Lead with the claim: "Nobody waits forever. Once you've taken your ticket, each other customer can get ahead of you at most once. That holds as long as nobody gets stuck in the room or frozen mid-choice." Ruth should check the reworded condition. |
| maya-15 | maya_14 | 08_1 | minor | The "the proof counts on:" lower third has three promises in small text, and it sits under the question the voice is asking. I can't read three clauses and listen at the same time. It comes back in 12_1, where it matters. | Drop it from 08_1. Or voice one line in 08_9: "The proof counts on a few promises. Remember this one: a finished write is seen by every later read." |
| maya-16 | maya_14 | 13_2 | minor | The voice says "fast memory" and the screen says "cache: off". Is that one thing or two? | Label it "fast memory (cache): off", or say "its cache, the fast memory," once. |
| maya-17 | maya_14 | 01_4 | delight | "It isn't an equation. It means make x one bigger." That's exactly the sentence I needed, the moment I needed it. | Keep it. |
| maya-18 | maya_14 | 03_4, 07_3, 12_3 (also 02_3, 04_5, 05_5) | delight | Pauses where I could actually answer: 3, 0, 0, 2 gives four; two ones gives two; the outboxes mean she sees nothing. 04_5's "Blue is number one, does she go first?" is a fair trick question. I felt clever every time. | Keep them. Fix maya-01 so 01_7's pause is answerable too. |
| maya-19 | maya_14 | 03_6, 04_3, 05_3 | delight | One tune at a time means only one customer is in the room, and two tunes at once is the bug you can hear. "After you. No, after you." made me laugh, and cat before cow made the tie-breaker obvious. | Keep them. |
| maya-20 | maya_14 | 06_1 to 06_2, 07_6 | delight | The same square grows into a cube while the counter races past two billion. Then the scoreboard from the first minute goes 41 to 46: five visits, five points, none lost. | Keep them. |
| maya-21 | maya_14 | 08_8 to 08_9, 09_1 | delight | "Like me being taller than you, and you taller than me" is the whole proof in one line. "Only the proof covers every case" told me why the proof beats the computer search. 09 becoming 19 or 00 is a glitch I could check myself. | Keep them. |
| maya-22 | maya_14 | 10_5, 13_4 | delight | The opening question comes back and gets a one-word "Yes." Then the ending explains the title: he discovered it, and that's why he's proudest. The coda with everyone's tune in order is a great last picture. | Keep them. |
