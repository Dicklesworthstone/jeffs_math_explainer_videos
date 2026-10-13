# Decisions — Proofs You Can See

One row per choice that a revision must start from. **Status:** *order* = set by the owner's
request or the orchestrator's order for this video; *proposed* = the director's choice, in the
brief or storyboard, awaiting the owner's look at the storyboard and drafts; *approved* = the
owner said yes (none yet); *retired* = no longer in force (kept, with why). A revision reads this
file first and changes only what its request needs; a change to a row is written here.

| # | What | Why | Where it lives | Status |
|---|---|---|---|---|
| 1 | The owner's four pictures are in: the cubes as slabs tiling a 15 by 15 square; 1/4 + 1/16 + … = 1/3 in a quartered square; (a + b)², ³, ⁴ as blocks; Siu's three stacks | his request and thread | chapters 01, 05–09, 11, 12; brief "Proofs, and why these" | order |
| 2 | About 8–10 proofs, judged by surprise, clarity and what motion adds; each a real proof, then what was shown and what the picture assumed | the order | brief; the card (storyboard header) | order |
| 3 | An honest coda: a picture can lie (Curry's missing square), and what turns a picture into a proof | the order, and the owner's own first post ("it's easy to fool yourself") | chapters 14, 15 | order |
| 4 | Quizzes with a visible countdown and silent think time | the order; owner workflow of 2026-10-13 (4–6 s, answer only after the countdown) | `model::QUIZZES`, nine `⟨predict⟩` rows marked `countdown` | order |
| 5 | Voice `michael`; 12–16 min; chapters ≤ 85 s; viewers Maya, Dev, Ruth; working title "Proofs You Can See" | the order's defaults | brief; `./x` voice table (not yet added) | order |
| 6 | Every identity checked in facts.rs and oracle.py for many sizes; every piece count computed | the order | `src/model.rs`, `src/facts.rs`, `oracle.py`; `./x visual_proofs facts-check` | order |
| 7 | Solids drawn by the scene itself with a fixed viewpoint and projected cubes, no camera route | the order's rule for partial 3D; the engine's VMobjects have no depth test | brief "Three dimensions"; `chapters/iso.rs` (in progress) | order |
| 8 | Eight proofs and one lie: the owner's four plus odd layers, two staircases, the halved square and Pythagoras by two arrangements | each classic sets up one of the owner's four; Pythagoras is the honest twin of the lie | brief; storyboard §2 | proposed |
| 9 | Left out: circle by rings or wedges, Cavalieri's sphere (limits of curved pieces), AM-GM from four rectangles (dropped to fit the length), difference of cubes, (a + b + c)³, subsets on a cube, the Gauss story | brief "Proofs, and why these" gives each reason | brief | proposed |
| 10 | The hook: the cube totals are squares five times; Moser's circle breaks at six points (31, not 32) | an honest failure of checking cases (plan rule 2) | chapter 02 | proposed |
| 11 | "The card" (Shown / Assumed) after each proof, failing once at 14_7; "the next size" asked of each proof | the coda's three criteria are earned by repetition | storyboard header; chapters 03–15 | proposed |
| 12 | Colour roles: five size colours pink, copper, green, blue, violet (#EA87B3, #CD853F, #83C167, #58C4DD, #994DE5); hero gold for the moving piece and answers; warn red for what does not fit | the owner's picture's order (red to purple); every pair ≥ CIEDE2000 21 | storyboard §3; brief | proposed |
| 13 | History said only as far as the sources were seen: Nicomachus noticed cubes are runs of odd numbers (did not state the rule); al-Karaji proved it about 1000; the slab picture Fry's (1985); Siu 1984; Ajose and Nelsen 1994; Curry 1953, Gardner's triangle 1956; nobody named for the halved square or the two-arrangement Pythagoras | the sources table and its "never said" list | fact sheet §Sources; `docs/protected.tsv` | proposed |
| 14 | Spoken vocabulary: picture, proof, piece, fit, tile, layer, staircase, stack, box, slab, block, card ("took for granted"); never "obviously", "clearly" | one word per idea (plan rule 21) | storyboard "Vocabulary" | proposed |
| 15 | Music: F major, 88 BPM, pad bed, vibraphone melody, plucked answer; motif three notes then their chord | pieces, then the whole | `src/score.rs` | proposed |
