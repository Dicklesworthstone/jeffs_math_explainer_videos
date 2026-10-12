# R4 final-cut review — raptorq_gf256 (Video B)

Fresh-eyes review of the 4K final before publishing, 2026-10-12, by CloudyMink
(Video A's builder; B was new to me). Reviewed:
`explainer_corpus/raptorq_gf256/renders/final/`, the 17 chapter files and
`raptorq_gf256_master.mp4` (3840×2160 at 60 fps, 1,256.0 s = 20:56; master
built at 664aaaa).

**Result: 0 blockers, 3 majors.** The picture matches the script on every
line, and every number I recomputed is right. The three majors are small text
problems with one-line fixes. Nine smaller items are listed at the end as
optional.

Times are master time (m:ss). Chapter-local time is in brackets.

## Majors

| # | where | what a viewer sees | fix |
|---|---|---|---|
| M1 | ch 11, **11_5**, 12:00–12:11 (41–52 s) | The two guide lines that run from the clock to the little clicks/bytes table pass through the table's top row. One crosses the **9**, 10 and 11, so the answer "9" looks struck out while the voice says "nine clicks". The end of the "7 clicks" arc also sits on the grey row title "clicks". | End the guide lines at the table's left edge, or keep them clear of the digit row. Move the row title "clicks" to the left of its row, clear of the arcs. |
| M2 | ch 11, **11_7**, 12:36–12:40 (77–82 s) | The label "0: none" sits at the clock's centre, under the hand and the mirror lines. The hand cuts through "none" and the label can't be read. It is the only place this frame says zero has no partner. | Move it off the pivot: below the clock, or beside "one full turn: 255 clicks ≡ 0" on the right. |
| M3 | ch 01, **01_8**, 1:11–1:16 (71–76 s) | The hook's closing bar chart labels its middle bar "carousel ≈ 23,500". The word is never said or shown before. The voice says "send the file on a loop" and the screen says "loop 1", "loop 3", "loop 6". A viewer can't tell this bar is the loop. | Label the bar "the loop". |

## What was checked

- **Cue check** (`cue_sheets.py` over `renders/final`, sheets in
  `renders/final/cue_sheets`): PASS, 0 errors. All 104 lines play once, in
  script order, with no overlap. Every chapter is under 85 s; the longest is 10
  at 84.6 s. The five warnings are authored pauses.
- **Every cue sheet read against `script.tsv`:** each line's frame shows what
  the line says, in all 17 chapters.
- **Dense pass:** 209 frames of the master, one every 6 s, read as 18 sheets.
  Thirteen suspect spots were then pulled at full 4K resolution, several as
  0.1 s time series.
- **Numbers recomputed and correct:** 3 × 7 = 9, 2 × 128 = 29, 2 × 142 = 1,
  128 × 4 = 58, 150 × 3 = 167, 150 × 150 = 26, with their click counts (180,
  25, 205, 105; 75 − 25 = 50; 198 + 198 → 141 → 21). 0x57 · 0x83 is 0x31 with
  RaptorQ's polynomial and 0xC1 with AES's. The code panel's 32,131 of 65,025
  pairs. The seven-hour times table. Chapter 16's parked-piece equations
  (c = 7, then A = 5, B = 3, C = 7, D = 2, E = 12) and the ring of five
  (3A = 15, A = 5). 97,876; 71 → 42 → 23 → 12%; 41,666,666,666,667 and 11.6 h;
  70.7% and 0.646%; the order counts in 12_4's note.
- **Not found:** typos, clipped text, dead frames, or a line whose picture
  belongs to another line. The two empty frames in the sample are a scene
  change under half a second long (15:45) and the first frame of chapter 17
  (19:33). The closest text to a frame
  edge is the last equation of 16_5, about a quarter of a unit from the right
  edge and fully visible.
- **Not checked:** the sound. I can't listen, so narration, music and
  loudness rest on the master's own VERIFY PASS and the narration QA. I did
  not re-run verify.

## Smaller items (optional; none needs a fix before publishing)

| where | what |
|---|---|
| 01_2, 0:09 | For about 0.4 s the new caption "10,000 pieces · 10,002 packets caught → picture rebuilt" crossfades through "each phone drawn = 1,000 phones" in the same place. Fading the old one out first would clear it. |
| 03_8, 3:33–3:39 | The "precisely" note covers the fountain's basin and the foot of its pipe, and the bottom packet cards touch the note's top edge. The note's text is readable. |
| 06_5, 6:27–6:33 | In the shrunken ring, the D ⊕ E and C ⊕ D cards touch. |
| 07_4–07_5, 7:15–7:27 | The small clock's "128" label sits directly on the bit row's place label "128", so it reads as a doubled number. |
| 08_4, 8:33–8:40 | The caught bit's chip rests against the end of "256 members, one for every byte" and touches the final "e". |
| 11_3, 11:39 | The "precisely" note covers the bottom edge of the star figure. |
| 13_4, 14:33–14:37 | The danger meter already reads 1/256 while the panel beside it still shows "the 2 values of one ex-or amount" for about 4 s. |
| 16_4, 19:09 | The new byte card flashes solid yellow for about half a second and its text washes out (the highlight's default colour floods the card's fill). |
| 02_2, 1:33 | "4 + 1 = 5" stays on screen while the byte fills with ones for "zero to two fifty-five". |
