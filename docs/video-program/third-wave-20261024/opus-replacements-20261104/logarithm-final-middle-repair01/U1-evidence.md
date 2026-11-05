# U1: evidence on the 06_2 hold

**Status: uncertain authored timing.** Root's instruction, relayed 2026-11-05, is to retain it as
that, with no change to physics or narration. Nothing is staged for U1. This records what the source
and the master show.

**Correction to my review.** Mail 5203 said "Stopping nearer 1.5 would remove the concern", and
`logarithm-final-review-pane168-01/REVIEW.md` line 110 offers the same stop as a next step. That went
past the evidence. U1 is not a demonstrated answer leak, and no test shows that a different stop
would change what a viewer does.

## What the frozen picture shows

Picture 0598 (master frame 50777, 846.28 s), opened as an image. The hold has 0 changed pixels
outside the bar across its 6 recorded pictures.

| Item | Value |
|---|---|
| Walk position | 0.93 of a lap, set by `lifting_loops.rs:371`, `tween(turn, 0.93, 1.6)` |
| Readout | "carried logarithm: 1.86 πi" |
| Number on the circle | Green dot 25.2° short of the yellow dot at 1, so it has not arrived |
| Column | Blue dot 0.14π, about 11 cell pixels, under the purple dot at the tick "2πi" |
| Marked on the column | Purple dots at 0, 2πi and 4πi, with tick labels 0, πi, 2πi, 3πi, 4πi |
| Card | "back at 1: logarithm = ?" |
| Answer text | None. The gold "2πi" only replaces the card later, at `lifting_loops.rs:380–381` |

## Timing

"Source" rows are derived from tween durations, not sampled. The rest are recorded pictures.

| Time (s) | Master frame | Basis | Event |
|---|---|---|---|
| 841.48 | 50489 | Picture 0592 | 06_2 sentence 4 starts, "When the number arrives back at one, what is the logarithm?"; `:370` waits for this sentence and `:371` starts the last climb |
| 841.98 | 50519 | Picture 0593 | Readout about 1.17 |
| 843.08 | | Source | The 1.6 s climb ends at 0.93 of a lap |
| 843.48 | | Source | The card's 0.4 s fade ends |
| 844.97 | 50698 | Picture 0594 | Readout 1.86, card up |
| 845.17 | 50710 | Placement | The 06_2 cue ends |
| 845.78 | 50747 | Picture 0596 | Mark; bar starts |
| 849.83 | 50990 | Picture 0601 | Last recorded hold frame |
| 850.03 | 51002 | Picture 0602 | 06_3 starts, "Two pi times i."; `:378` starts the 0.8 s climb to one lap |
| 850.53 | 51032 | Picture 0603 | Readout 1.97 |
| 850.83 | | Source | Climb ends at 1.00 lap; the 0.4 s swap to gold "2πi" starts |
| 852.45 | 51147 | Picture 0604 | Readout 2.00, gold 2πi shown |

The number stops short for about 2.1 s of the voice's last sentence, then through the whole hold.
It reaches 1 only after the answer cue has begun.

## Evidence on each side

For treating it as a hint:

- The marked purple dot at 2πi is the nearest candidate above the blue dot, and the source has
  taken the walk up in stages: 0.25 of a lap at `:348`, 0.5 at `:361`, 0.93 at `:371`.
- The authors treat picture giveaways as real elsewhere in this chapter pair:
  `complex_turns.rs:627`, "the blue x = 0 column leaves too: it would give away 05_6's first step".
- The standard says no hint on screen may give the answer away during the countdown
  (`REVIEW-AND-CONTINUITY.md` lines 76–77).

Against treating it as a leak:

- No text states the answer, and the number is deliberately held short of 1.
- Three candidates are marked. Which one the walk lands on is the inference the chapter wants, by
  continuity. That is the teaching point, and 06_3 answers it.
- Other holds in this film also show a position and ask the viewer to read it, in pictures I
  opened: 02_2 has the cursor on the D scale at 8 with "?" under it (picture 0202), 04_6 has the
  dashed ×2 line crossing the curve (picture 0414), and 01_3 has the marker at 3's place
  (picture 0121).

## What would settle it

Only a person's judgment can settle this: the owner's, or a viewer's, at playback. Source and
frames cannot, and I have not watched it in motion.

The stop is set by the `0.93` at `lifting_loops.rs:371`, which stays as authored.

The O1 patch moves this hold's bar from over the z plane to under the card. That doesn't touch the
walk or the readout, so it neither adds to nor removes this concern.
