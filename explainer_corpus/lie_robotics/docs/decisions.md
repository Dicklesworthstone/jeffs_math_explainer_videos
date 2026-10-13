# Decisions — Video G (lie_robotics)

One row per choice in force. A revision starts from the version these rows
describe, reads them first, and changes only what its request needs; a row is
changed on purpose and the change is written here. "Owner" means the owner has
not yet seen or approved the choice; everything so far is the director's
default (`docs/brief.md`), reviewed in R1 (`docs/reviews/lie_robotics/R1.md`)
and the claim walk (`docs/claims.md`). The words every revision keeps are in
`docs/protected.tsv`.

| # | What | Why | Where it lives | Status |
|---|---|---|---|---|
| 1 | The arm is the KUKA LBR iiwa 14 R820, numbers from the owner's `vendor/iiwa14.xacro`, cited by path, never copied | the only iiwa in the owner's project with every link offset in a source file; the xacro is GPL-3.0 upstream | brief "The arm chosen"; `model::JOINT_ORIGINS`; fact sheet §Sources | in force (owner to see) |
| 2 | `vendor/iiwa14_kinematics.json` is not used | it drops every ±π/2 roll, making all seven axes parallel | fact sheet finding 1 | in force |
| 3 | The mobile platform's speeds and paths are the video's own numbers, labelled so; no chassis dimension shown | the owner's `kmrGeometry.ts` numbers are an older brochure's and its wheel citation could not be found | storyboard 06_4 lower third; fact sheet finding 3 | in force |
| 4 | Working title "Why Robot Arms Think in Lie Groups" | the order's working title | brief; `main.rs`; title card 01_3 | owner |
| 5 | Voice `michael` | the owner's choice for the corpus | brief | in force |
| 6 | One picture (the screw) and one move (the flat piece, then wrap back), with a corner badge whenever the move is made | R1 (C1, M11): the move must stay on screen | storyboard preamble and §2 | in force |
| 7 | A quiz in every chapter (15): violet card beside the picture, 4 to 6 s countdown, "pause for more time", readouts that would give the answer hidden | the owner: "quizzes with pauses before the answers"; the same card as Videos F and J | storyboard "The quiz device"; `model::QUIZZES`; `G-Q.*` | in force; timing to be checked on the draft (frames at cue times) |
| 8 | 3D by the video's own fixed projection (front right, about 32° round and 22° up) on the retained route; no camera route | quiz cards, matrices and readouts share every 3D frame; the layout lint reads it | storyboard preamble; R1 E1 to E4 | in force |
| 9 | Notation: `e^{[ω]t}` with ω a unit axis and t the angle (no θ); `T(q) = e^{[S1]q1} ⋯ e^{[S7]q7} M`; twists spin first | Lynch and Park's conventions; agreed with Video J, which hands off to this video | brief "Notation"; mail thread notation-gjk | in force |
| 10 | Vocabulary: tilt and roll (the book), tip (the gripper's), wrap back (the move), platform (mobile) against base (the arm's foot), step (Newton's only) | each word one idea (R1 M35, claim walk 14 to 17) | storyboard "Vocabulary" | in force |
| 11 | Palette: blue `#4A8DF8`, teal `#3FE0E0`, orange `#D48F3F`, pink `#EC4E97`, steel `#CFD7E5` with the corpus's hero, warn, good and quiz violet; every pair ≥ ΔE2000 20 | the first palette failed its own claim (R1 C7) | `src/palette.rs` and its test | in force |
| 12 | Chapter 11's quiz pose has only the elbow straight (rank 5, one lost direction) | the stretched arm loses two directions while the words explained one (R1) | storyboard 11_2, 11_3; `G-J.rank.elbow_straight` | in force |
| 13 | 11_6 says a six-joint arm "with two joints at the shoulder" would lose a direction | removing joint 7 loses nothing; removing any of joints 1 to 3 does (claim walk 1) | storyboard 11_6; `G-J.rank.ell.without_*` | in force |
| 14 | No claim that this arm has a closed-form inverse kinematics | not checked against a source (claim walk 10) | storyboard 13_6; fact sheet finding 8 | in force until a source is opened |
| 15 | Mozzi is credited with "such an axis" in 1763, not the theorem | his book and Ceccarelli's paper were not opened (R1 R8) | storyboard 07_6 | in force; deferred to bead .5: open Ceccarelli |
| 16 | Rodrigues: "named after", not "found by" | where the formula sits in his 1840 paper was not verified | storyboard 05_6 | in force |
| 17 | Length 16.5 min by the G2 estimate, over the 12 to 16 min default | the R1 fixes added words; the next cuts would be the Apollo beats and 14_5 | brief "Length target" | owner's call |
| 18 | The chapter keys `06_the_platform` and `14_the_steady_path` | the words "platform" and "steady" (R1) | `chapters/mod.rs` | in force |
