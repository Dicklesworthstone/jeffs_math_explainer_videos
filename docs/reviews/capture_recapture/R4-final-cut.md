# R4 final-cut review — capture_recapture (Video A)

Reviewer: pane 5 (MaroonIbis, built Video B; A is new to me). 2026-10-12. Reviewed the 4K master
`explainer_corpus/capture_recapture/renders/final/capture_recapture_master.mp4` (1,052.2 s, built at 300410d) and the
15 chapter finals beside it.

**Verdict: 0 blockers, 2 majors.** Both majors are overlaps a viewer sees on a paused or even a running frame. Each
needs one chapter re-rendered (09 and 14). Nothing else I found is worth holding the upload for.

## What I checked and how

- **Cue check.** `cue_sheets.py renders/final --script script.tsv --sheets renders/final/cue_sheets`: PASS, 0 errors.
  Every line is spoken once, in its chapter, with no overlap and none past the end. The 13 warnings are silences of
  4 s or more (predict pauses and listen windows, which hold the score) and two long tails (13: 5.5 s, 15: 9.5 s).
- **Cue sheets.** I read all 15 sheets against `script.tsv`, one frame per line at 1.5 s in. Every line's frame shows
  what the line says, or the last moment of the transition into it.
- **Dense contact sheets.** One frame every 6 s of the master (175 frames, 11 sheets of 16). I read them all for
  overlaps, clipped or illegible text, typos, glitches and dead frames.
- **4K stills.** For each suspect frame I pulled the still from the master and read it at full size.
- **Numbers.** Where a number is both spoken and shown, I compared them. All agree. Examples: 8,100; 451 × 100 =
  45,100; 4,845 and its five piles; 31.8% at 33; 16.67; 529/528 and 576/578; 810,000; 11,111 and 9,091; 8,475 –
  12,048; 2,600; 4.43% and 7.96%; 8,400 – 12,073; 7,750; 28.35 → 28.4 million; 47 – 94; 10,356 and 9,790.

Not checked: the audio (I read pictures, not sound), the captions files, and anything between my 6 s samples that
lasts under a few seconds.

## Majors

| id | where | what a viewer sees | fix |
|---|---|---|---|
| A-R4-1 | 09_6, 10:19 – 10:36 (the code panel) | The highlight boxes are drawn through the code. The gold box (about 10:20) puts its top edge through `marked = np.arange(N) < M` and its bottom edge through `p = [comb(M, j) * …`. The teal box (from about 10:22 to the chapter's end) puts its top edge through `est = M * n / k`. The struck lines read as crossed out, in the one panel a programmer will pause on. | Size each box from the lines it covers: top edge half a line gap above its first line, bottom edge half a gap below its last, so the strokes fall between lines. The boxes are now about 0.4 of a line too tall on each side. Or drop the outline and tint the covered lines' background. Re-render 09 only. |
| A-R4-2 | 14_2's end, 15:35 – 15:39 (Laplace's quote card) | The quote card is half-transparent and sits on top of the France table. The table shows through it, and a ghost of "the rest of France" sits behind the quote's last line. The card's left edge cuts "2,037,615 people" and "not counted" in two. Its bottom edge clips the tops of the letters in "2,037,615 ÷ 71,866⅔ ≈ 28.35 people per birth" and "chosen towns across France". The map is half covered. | Give the quote the frame to itself: fade the table, its labels, the two arithmetic lines, the map and its caption out before the card comes in (the chapter moves to the bugs table straight after), or dim them to nothing while it is up. Make the card's fill opaque. Re-render 14 only. |

## Seen, and judged not worth a re-render

These are small. Fix them only if 09 or 14's lane is in the file anyway.

- **01_8, 0:53 – 0:57.** The caption "each fish drawn = 10 fish" ghosts through the lower-right of the "precisely:"
  card. It does not touch the card's words.
- **12_3, about 12:56.** For about 2 s the frame holds only the "95% confidence interval" card before the range bar
  arrives.
- **15_1, 16:29 – 16:36.** The chapter opens on "Kosovo, 1999" alone for about 6 s, until the 4,400 bar is drawn. It
  reads as deliberate.
- **End card, 17:26 – 17:31.** The lake's fish and rings show through the Receipts card. The three lines are legible,
  and the repository address matches the remote.

## Evidence

- Cue sheets: `explainer_corpus/capture_recapture/renders/final/cue_sheets/sheet_*.png` (15, with legends).
- Stills read at 4K: 10:20.5 and 10:28 (code panel), 15:37 (quote card), 0:56, 2:56, 9:22, 12:09, 15:02, 15:31,
  17:30.
