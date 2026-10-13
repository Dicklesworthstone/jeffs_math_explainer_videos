# Decisions — What Does It Mean to Raise e to a Matrix?

One row per choice in force. A revision starts from the version these rows
describe, reads them first, and changes only what its request needs; a row
changes on purpose and is written down here. Status: **owner** (the owner's
own words), **order** (the orchestrator's order for this lane), **agreed**
(settled with a sibling lane by mail), **R1** (a review disposition,
`docs/reviews/matrix_exponential/R1.md`), **default** (chosen in the brief;
the owner has not yet seen it). Nothing below has the owner's approval yet:
he sees the storyboard and drafts after P5.

| what | why | where it lives | status |
|---|---|---|---|
| The topics, in his order: exp's properties and definitions, ODEs, the series, the matrix series and its convergence, what e^A means (systems, graphs), what it computes, the road to Lie theory | his request | brief, top row; storyboard §2 | owner |
| Quizzes with a visible countdown and a silent pause before each answer | his request for every video | eleven `listen N s (countdown)` beats; `model::QUIZZES`; test `every_countdown_is_a_quiz_and_every_quiz_has_its_countdown` | owner |
| Voice michael; 13–17 min; chapters ≤ 85 s | the order | brief | order |
| Working title "What Does It Mean to Raise e to a Matrix?" | the order's working title, kept | brief; `glyphs::TITLE` | default |
| One picture: a rule for tiny steps, followed (dots carried along their arrows) | carries every act; returns in 01, 09, 10, 16 | brief "The one picture"; storyboard §4 | default, R1 (C2) |
| A matrix is a machine, used as a move or as a rule; the exponential takes a rule and returns a move | two readings were conflated in the first storyboard | storyboard Vocabulary; 01_4, 09_1 | R1 (R5) |
| Never "only when they commute": "when they commute, the rule holds", with the exact statement and the 3-4-5 turns on screen | the order's wording is false for a single pair | 14_5; protected.tsv; fact sheet finding 1 | R1; departs from the order on purpose |
| No θ: the angle of a turn is the time t | the engine draws θ as ϑ at the pin | brief "Notation" | agreed (Video G) |
| Borders: G owns robotics (shown once at 16_1 in G's notation, e^{[S]q} … M); K owns eigendecomposition (one pointer at 13_5 in K's letters, V e^{tΛ} V⁻¹) | the three videos read as one family and do not repeat each other | brief "Siblings"; 13_5, 16_1 | agreed (G, K) |
| The opening question has no countdown; it returns as the last quiz (16_3) | nobody can answer it at 0:09 | 01_1, 16_3 | R1 (C10, C4) |
| The hopper quiz is cut; the strip at 12_3 names the hopper and gives both answers | its answer depended on a rule the card did not state | 12_3; fact sheet finding 8 | R1 (blocker D1) |
| Both commutator loops are multiplied in the order played (later motion on the left) and leave minus the commutator | the first facts used the other order on both sides of facts-check | 14_6, 15_3; test `the_loops_leave_minus_the_commutator_in_the_order_played` | R1 (D6, R9, C5) |
| The 32-bit series run shown is this video's own, labelled with its loop; Moler and Van Loan's IBM digits only in the description | their digits cannot be reproduced in IEEE arithmetic, and other loop orders give other wrong answers | 13_3; protected.tsv; fact sheet finding 2 | R1 (D8, R12) |
| The Standard Model in one line: 8, 3 and 1 tiny steps, twelve carriers; the fine print in the description | honestly scoped, as the order asks | 16_2; protected.tsv | order |
| Countdowns 4 or 5 s | the owner's quiz rule asks for 4 to 6 s; shortened from 5 or 6 for length | storyboard; `model::QUIZZES` | default |
| Length at the top of the range (17.4 min by the gate, 17.0 without its per-countdown allowance; the finished michael cuts ran 3 to 5% over the gate) | every topic of the request has its chapter | brief "Length target" | default; owner to decide whether 17 must hold |
| Moler said "Mohler" (long o); eigenvalue as written, never sentence-initial | probed in michael; partial | `lexicon.json`; claims.md "Names" | pending the owner's ear |
| Four role colours nudged, the rest as the storyboard sets them: substance amber `#F0AC5F` to `#E8903B`, quiz violet `#B189D6` to `#9985E6`, good green `#83C167` to `#83B943`, turning blue `#58C4DD` to `#54C4E5` | the storyboard's own gate (every pair of roles at least ΔE2000 20 apart, by a test before drafts) failed as drawn: result gold and substance amber 14.9 (and they share the frame in 03), second rose and quiz violet 15.7 (and share it in 14), rule teal and good green 17.3, rule teal and turning blue 19.6. These are the smallest nudges found (ΔE2000 2 to 8 from the old hues) that put every pair, ink and background included, at 22.0 or more | storyboard §3; `chapters/draw.rs` role functions; test `the_role_colours_are_told_apart` | default (P4 lane, pane 5); the owner has not yet seen it |
| A quiz's ring drains from the scene's own clock inside the countdown's listen window (`Kit::listen`, no sonification file) | the storyboard asks for the music alone while the ring runs, the motif left unresolved until the answer lands: that is the score's own listen window, so the card has no sound of its own to author or re-level, and the ring and the window cannot drift apart | `chapters/draw.rs` `QuizCard::count` | default (P4 lane, pane 5) |
