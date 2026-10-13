# Decisions: fair_coin (Video E)

Each approved choice, why it was made, where it lives, and whether it is in
force. A revision starts from the version these describe, reads this file
first, and changes only what its request needs; a change to a row below is
made on purpose and recorded here. The words every revision keeps are in
`protected.tsv` (check: `protected_check.py docs/protected.tsv script.tsv
docs/description.md`).

| # | Decision | Why | Where | Status |
|---|---|---|---|---|
| 1 | The owner's idea is the spine: von Neumann's two-toss trick, then its growth to unfair dice; those two get most of the screen; memory, Peres, Elias, entropy and the hardware note are short beats | the owner's request; orchestrator's steer, 2026-10-12 | brief.md, storyboard.md §2 | in force |
| 2 | One picture: the unit square cut at the coin's chance on both sides; heads then tails and tails then heads are mirror images across the diagonal; the cube and the six-way grid are the same picture one step on | orchestrator's steer, 2026-10-12 | every chapter; `draw.rs` (`Square`, `Cube`) | in force |
| 3 | The coin is a model, heads four times in five, labelled "a model coin (one we made up)"; the loaded die is a model too, 5, 5, 10, 10, 20 and 50 percent | an ordinary tossed coin cannot be this unfair (Gelman and Nolan); the numbers must be easy to see in the square | `model.rs` (`bent`, `LOADED`), 01_1, 08_1 | in force |
| 4 | Every seeded number comes from seeds 195101 to 195109 (NumPy PCG64DXSM), and no seed or window is chosen because it looks better: 01_1 shows the whole thousand tosses with their count; 10_2 says the strip's own count (fifteen) and names sixteen as the average | owner's rule via the orchestrator, 2026-10-13 | `facts.rs`, `oracle.py`, 01_1, 10_2 | in force |
| 5 | Colour roles: heads blue, tails pink, Alice green and always above the diagonal, Bob violet below, thrown-away pairs grey, gold for strokes and words only; dice grids are laid six down to one from the corner so "first roll bigger" stays above the diagonal | five R1 reviewers; ΔE2000 test in `draw.rs` | storyboard.md §3, `draw.rs` | in force |
| 6 | "Order never changes a chance" is said only for tosses with the same chance every time and no memory, after the coin with memory has been shown | orchestrator, 2026-10-12 (R1 R4, M12) | 06_4, edge test `edge_same_counts_same_chance_without_memory` | in force |
| 7 | Voice: michael. Names: von Neumann spoken in its plain spelling; Pascal respelled; Peres and Elias on screen only (their probes failed) | owner's voice for this lane; narration skill | `x`, `lexicon.json` | in force |
| 8 | Every ⟨predict⟩ beat gives 4 s of silent think time with a draining gold ring and "pause for more time"; the answer comes only after the ring empties | owner's quiz rule, 2026-10-13 | `draw.rs` `predict_pause`, `THINK_S` | in force |
| 9 | The picture stays through the cut where two chapters share it: the resting square at 02→03, 03→04, 04→05 and 05→06, the four-toss row at 06→07; the other joins keep a fade | R2 C10; the subject changes at the others | chapter ends and starts; `draw.rs` `fade_all_but`, `FourRow` | in force |
| 10 | Length: 9 to 12 minutes, chapters at most 85 s. The scratch drafts total 13.0 min (02 86.0 s, 03 and 06 85.5 s); no words are cut before narration, because michael ran about 7 percent shorter than scratch timing in Video C; the narrated draft decides, and the first cuts would be 04_6, 01_5 and 09_6 | orchestrator's 9 to 12; owner: shortening is the riskiest revision | storyboard.md, this row | in force until the narrated draft |
| 11 | The video ends on the square beside the receipts card, not on a text card | R2 C6, M29 | 11_3 | in force |
| 12 | Nothing is published or uploaded from this lane; renders, audio and sources stay out of the repository | AGENTS.md | — | in force |
