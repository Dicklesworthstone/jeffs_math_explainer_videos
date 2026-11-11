# 08_loop_sums preview01 picture review: preparation

CC2 (RubyRaven), 2026-11-10 23:49Z. **Status: no visual verdict.**

At 23:49Z on CSD, preview01 is still running (09). `csd-native-combined-pictures01` does not exist yet, so no 08 picture or sheet exists. This file is preparation only.

## Independence

I wrote no part of the 08 source. My session transcript has no Write or Edit on any loop_sums file.

| Version | sha | Authors |
|---|---|---|
| candidate01 | `9a91ed2a` (L12, L11, L5) | CC1 |
| candidate02 | `977c77df` (c name and leader redraw) | CC0 |
| bundle02 rename | `a8d62f56` (`centre_name_at` to `c_name_at`) | root |

combined01 builds `loop_sums.rs` `a8d62f56` over build268 `8a13e8a1`.

## Inputs

- **Preview.** preview01 `08_loop_sums.mp4` `63984b44`: 960x540, 14959 frames.
- **Layout.** preview01 `08_loop_sums.layout.tsv` `c5347cb9`, copied here. It has 3 rows.
- **Selection.** `csd-combined01-pictures.py` takes plan02's 407 frames (`frames-08_loop_sums-all.txt` `6b70dab9`). For each layout row at time t it adds frames t, t+12, ..., t+120 and one every 300 frames to the end.
- **Result.** `sheet_map01.py` reproduces the selection: 447 pictures on 56 sheets. Sheets are 1280x1536 with 8 thumbnails at 640x360, labelled `NNN | frame F | t s`.

## The two lint anchors

| Anchor | Lint row | Source | Pictures / sheets |
|---|---|---|---|
| 179.8 s, frame 10788 | "c" and "c" overlap 100%, at 185 later looks | The L12 quarter picture redraws the dot and `kit.tex("c", 26.0, output(), c_name_at)` on top of `centre_name`. The size, colour and position are identical (combined01 lines 653–657 and 768–783, comment "candidate02, review F4"). Both fade out together in `square_picture`. | 315–354, sheets 40–45. The hold runs to the fade-out, pictures 382–435, sheets 48–55. |
| 205.6 s, frame 12336 | two strokes of unlabeled shapes cross "4^n", at 4 later looks | `cancel_top` and `cancel_bottom` strike through `four_top` (2.45, 1.0) and `four_bottom` (4.5, 0.5). The code is unchanged from build268 (base lines 700–722), so it is in master993 too. | 364–375, sheets 46–47 |

**What to look for:**
- **c/c.** Doubled or offset strokes on the c glyph or the dot. Heavier edges than the other labels. A c that lingers after the rest in the 0.4 s fade-out (CC1's earlier Low).
- **4^n.** Whether each stroke reads as a single strike-through cancelling its own 4^n, and the base and exponent stay legible. Also whether the strokes touch the bar or the other terms.

## plan02 windows and their sheets

| Window | Frames | Pictures | Sheets |
|---|---|---|---|
| 08-control-unchanged | 3748–6987 | 1–6 | 1 |
| 08-L5-fade-in | 6986–7012 | 5–31 | 1–4 |
| 08-L5-held | 7026–9139 | 32–39 | 4–5 |
| 08-L5-delta-circle | 9139–9450 | 39–70 | 5–9 |
| 08-L5-fade-out | 9740–9800 | 71–131 | 9–17 |
| 08-inset | 9797–9900 | 128–136 | 16–17 |
| 08-L11-diagonal-in | 10024–10052 | 137–165 | 18–21 |
| 08-L11-walk | 10216–10339 | 166–289 | 21–37 |
| 08-L11-perimeter-F5 | 10336–10483 | 286–311 | 36–39 |
| 08-L12-quarter-in | 10785–10818 | 312–345 | 39–44 |
| 08-L12-held | 10846–13053 | 348–383 | 44–48 |
| 08-L12-fade-out | 13052–13105 | 382–435 | 48–55 |

Sheets 55–56 are lint persistence, every 300 frames to 14958. Frames 0–3747 are outside plan02 and are not sampled.

## Plan once the sheets exist

1. Fetch pictures01 `08_loop_sums/pictures.json` and the sheets read-only, and verify their hashes against the record.
2. Read all 56 sheets as images. Open single 960x540 pictures where a sheet thumbnail is too small, especially for the c glyph and the 4^n strokes.
3. Report concrete findings only, with frame numbers.

Root accepted CC0's M1: no old master993 intervals carried over under the new preview hash. So the review states its own sampled intervals and does not inherit master993's.
