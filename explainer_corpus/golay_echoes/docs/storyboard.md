# Storyboard — Two Bad Echoes Can Make One Perfect Measurement

The hand-edited source of this video's words and pictures.
`python3 -I tools/storyboard.py golay_echoes` checks it (gate G2) and writes
`script.tsv` from the words column. Every number below is a fact in
`docs/fact-sheet.md`, computed in `src/facts.rs` and recomputed by `oracle.py`.

Words are spoken exactly as written, one thought at a time: no colons,
semicolons, dashes or parentheses in them. `*(…)*` is an annotation, never
spoken. `⟨predict⟩` opens a prediction question; the beat ends with the
question and the picture holds (a visible four-second countdown, "pause for
more time") before the next beat answers it.

## The arc

One tiny example runs through the whole video: pattern A is up, up, up, down;
pattern B is up, up, down, up. Every chapter keeps those two rows of four
pulses on screen.

| Chapter | Spine | Status |
|---|---|---|
| `01_two_bad_echoes` | send a pattern, score it against its echo at every shift; every single pattern leaves ghosts; a second pattern's ghosts are the same size with the opposite sign; added, only the echo is left, twice as tall; the guarantee's assumption: the same scene for both sends | storyboarded; scene in progress, not yet compiled or rendered |
| 02 what the ghosts hide *(planned)* | two things in the scene, one loud and one quiet, three steps apart: pattern A hides the quiet one completely, pattern B doubles it, the sum shows both at eight times their strength (`model::two_measurements` with the scene 4, 0, 0, 1) | not storyboarded |
| 03 why they cancel *(planned)* | each ghost as its pairs of pulses; why B's pairs are A's with one sign changed; the doubling that builds the pair from a single pulse (`model::double`), proved for every length | not storyboarded; the doubling proof is written in `model.rs` and tested to length 64, not yet staged |
| 04 when it breaks *(planned)* | the scene changes between the sends: moved one step, or the echo returns upside down (the echo itself cancels and every ghost doubles; the radar case of a half-turn of phase, Chi, Pezeshki and Calderbank eq. 8); noise picked up by the receiver is scored like any recording, and the identity does not remove it | not storyboarded |

## Vocabulary (one word per idea)

**pulse** (one up or down blip; never "chip", "bit" or "symbol"); **pattern**
(a row of pulses; never "code" or "sequence" in the voice; "pattern A",
"pattern B" on screen); **send** (never "transmit"); **echo** (what comes
back, and the place it really comes from); **copy** (the receiver's own
pattern, the one that slides); **line up** (two pulses at the same place);
**agree** and **disagree** (same or opposite directions); **block** (one
agreement; a disagreement takes one away); **score** (the blocks left at one
shift; never "correlation" in the voice); **shift** (where the copy sits) and
**step** (one place along); **ghost** (a score where there is no echo; the
lower-third names "sidelobe" in a later chapter); **row** (the scores at
every shift, side by side).

## Color roles

| Idea | Color |
|---|---|
| pattern A's pulses | blue `#58C4DD` |
| pattern B's pulses | violet `#C792EA` |
| an agreement, and a block above the line | green `#83C167` |
| a disagreement, and a block below the line | red `#FC6255` |
| the echo: where the thing really is; "echo" and its column marker | `hero` gold `#FFD866`, strokes and words only, never a block |
| a ghost's word | `dim` |
| baselines, shift numbers, the sender and the thing | `faint` and `dim` |

## 01_two_bad_echoes: Two bad echoes (~82 s)

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | Send out four pulses. Up, up, up, down. Something bounces them back. | A sender on the left. Pattern A's four blue pulses appear at it one by one on "Up, up, up, down"; a dot runs right to a dim block on the far right and back, and the pattern returns as the echo row | pulses built from the pattern in facts, motion | |
| 01_2 | Slide a copy along what came back, and multiply the pulses that line up. | The sent pattern is the copy: it comes down under the echo, three steps to the left, and slides right one step at a time until it sits exactly under it. A "for programmers" lower-third gives the score as code | slide, lower-third (code) | |
| 01_3 | Pulses pointing the same way agree, and add a block. Opposite ways disagree, and take one away. | A key on the right: two up pulses over a green block, an up over a down over a red block, each with its product (+1, −1) | legend built from the products | |
| 01_4 | Lined up exactly, all four agree. A score of four, at the echo. | At shift 0 the four lined-up pairs light, four green blocks appear between the rows and drop into the score column under the copy's start; the gold word "echo" above it, the score 4 beside it | blocks from the products, highlight, chart | |
| 01_5 | One step off, three pairs line up. Two agree, one disagrees. A score of one, where nothing is. | The copy steps right by one; three pairs light: two green blocks and one red. In the column the red block lands on a green one and both vanish; one green block is left, and the word "ghost" | blocks cancel, highlight | |
| 01_6 | Slide all the way, and ghosts appear on both sides, a quarter the echo's size. | The copy steps quickly through every other shift, scoring each; the full row stands: one block up at one step either side, one block hanging down at three steps either side, nothing at two; "ghost" by each | fast slide, chart, lagged labels | |
| 01_7 | With more than one pulse, every pattern leaves ghosts. So send a second. Up, up, down, up. | The two far columns flash: only the end pulses meet there. Pattern A's row shrinks to a shelf at the upper right; the echo and copy become pattern B's violet pulses (up, up, down, up). A "precisely" lower-third states the far-shift rule | highlight, morph rows, lower-third | |
| 01_8 | Its ghosts are the same size, at the same shifts. What if we add the two rows? ⟨predict⟩ | B's row is scored quickly: the same echo of four, ghosts of one at the same shifts but on the other side of the line. A's row comes down from the shelf and hovers over B's, column above column | fast slide, chart | |
| 01_9 | Every ghost meets its opposite and cancels. Only the echo is left, twice as tall. Two bad echoes, and not a single ghost. | A's row drops onto B's: at every ghost a green and a red block meet and vanish together; at the echo A's four blocks stack on B's four. The score 8, the gold "echo" above it. On "not a single ghost" a two-line "precisely" lower-third keeps the exact claim (same still scene for both sends; receiver noise is scored too and is not guaranteed to cancel) | blocks cancel, stack, highlight, lower-third | |
| 01_10 | But the guarantee needs the same scene for both sends. Move the echo one step between them, and ghosts return. | The added row clears; A's row returns over B's, as the two sends were. On "Move the echo" B's row steps one shift right and A's drops onto it: this time ghosts are left at six shifts and the echo is split three and five | chart, blocks cancel, highlight | |

### Notes for 01

- The copy's first pulse sits over the column its score lands in, so the
  score row is drawn where the copy was.
- Prediction (01_8): the question is on screen by its last word; four seconds
  of silence with the countdown; the answer starts with 01_9.
- Exact versions kept on screen: 01_7 (every pattern of two or more pulses
  scores one up or one down at its last shift; a single pulse has no other
  shift) and 01_9 (exactly zero away from the echo when both sends meet the
  same still scene; noise picked up by the receiver is scored too and is not
  guaranteed to cancel). Both cards are two lines, so they clear the shift
  numbers.
- 01_10 states the guarantee's assumption and one tested way to break it (the
  echo moved one step); it does not claim that every change breaks the
  cancellation (`model::tests::a_changed_scene_leaves_ghosts` keeps the
  general identity: what is left is the change, scored by pattern A).
- Budget (2026-10-18 scratch draft at 8f154b5: 92.2 s): the kit's scratch
  voice reads 2.6 words a second plus 0.4 s a line, thought gaps measured
  0.81 to 0.83 s, the think window 5.2 s, the tail 2.8 s. This revision cuts
  15 words, one line and the 2 s hold, and ends its last animation before
  the last line does: about 82 s by the same measures.
