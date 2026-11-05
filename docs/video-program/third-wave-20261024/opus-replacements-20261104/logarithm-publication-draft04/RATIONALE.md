# Logarithm publication draft04

LavenderPike, 2026-11-05. `metadata.diff` is the exact change from `publication-stage09-draft03`, which stays as it is.
Nothing was uploaded. The times are provisional until the final master and real seeks confirm them.

## Chapter times

All 13 starts are the same as draft03. They come from `frames_after` in the remote `schedule-comparison01.json` (4c4e42be).
Only chapter 12 changed: it gains 420 frames, and as the last chapter it moves no start. Chapter 04 stayed at 11725 frames,
not the predicted +24, and the times use the real number. The film now ends at 32:24, 7 s later. Chapter 12's
start stays 29:33. `chapter-times02.json` holds the frames.

## Changes

| Change | Why |
|---|---|
| New title | The old title had no em dash, but "Infinite Answers" tells a new viewer little and hides the matrix chapters, a third of the film. The new one names the real span and follows the algebra film's title pattern. It's a proposal; root decides. |
| Higham chapter added, linked to eprint 310 | The film's screens cite "Higham (Handbook of Linear Algebra)" and "Higham §4 Fact 1". Draft03's link, eprint 1067, is the 2008 SIAM book, where those references don't apply. I checked the eprint 310 PDF by eye: chapter 11 of the Handbook, section 4 "Matrix Logarithm", Facts 1 and 3 as the film uses them. |
| Book kept, marked "For more depth" | `sources.md` says the book wasn't read for the film, so it shouldn't read as a source. |
| Culver added | The narration names Culver at 11_20 and the screen cites the paper, but draft03 had no entry. The wording matches the screen: as Higham and Gallier state it. |
| Natural logarithm added | Chapter 04 runs over 3 minutes and draft03 skipped it. |
| Depth sentences added | Draft03 didn't say what the film assumes. Some general results are also cited, not proved: the principal matrix logarithm (10_4), Culver's rule (11_20) and complex logarithms of invertible matrices (11_8). |
| "a synthetic voice" added | Matches the channel's earlier disclosure. The decode receipt names Oliver and has no Frederick. |
| Music credit added | The film has a score from `explainer_score`, and draft03 didn't credit it. |

Unchanged: the 13 chapter labels, the opening and matrix paragraphs, Gallier, DLMF, the FrankenManim credit and `tags.txt`.
The script is still 559afa36, the bytes draft01 read in full; I read all 80 cues again for this draft.

## For root

- **Companion film.** Cue 04_1 says "The companion video builds it from scratch." `catalog.json` records no publication
  for `exponential_magic`, so I added no link. If it isn't public at release, that line points at nothing.
- **`sources.md` has the wrong record.** It links the Handbook chapter to eprint 1067, which is probably how draft03's
  correction went wrong. I didn't edit it.
- **Culver's DOI.** It redirects to the AMS page for this paper, but that page refuses automated reads, so the paper and the
  page are still unread. The pages and DOI agree with the film's secondary sources. Open the link once in a browser before
  upload.

## Not checked

The final master, seeks, captions and the thumbnail against the new title. The thumbnail was last viewed as a static image
on 2026-11-05. No listening, AV review or fine sync.
