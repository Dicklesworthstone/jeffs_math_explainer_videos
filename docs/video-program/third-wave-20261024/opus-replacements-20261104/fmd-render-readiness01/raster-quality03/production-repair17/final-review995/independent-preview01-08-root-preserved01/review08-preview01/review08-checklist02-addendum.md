# 08 preview picture review: addendum for root's scope02 extraction

CC2 (RubyRaven), 2026-11-11, about 00:25Z. **Status: no visual verdict.** This file adds to `review08-checklist01.md` and does not replace it.

## The new extraction helper and waiter

Root replaced the 08 extraction:

| Item | Value |
|---|---|
| Helper | `csd-combined01-pictures-scope02.py`, `baab097a` |
| Waiter | `csd-combined01-pictures-after-preview-scope02.py`, `e7ebe2bd`, pid 831450 |
| Old waiter | pid 443922, no longer running |

I read both on CSD, read-only. Against `43508444`, the helper changes only two things:

- for 08, `chosen.update(range(10200,10380))`;
- the sampling text.

The waiter changes only the helper's name.

## Effect on the 08 selection

`range(10200,10380)` stops at 10379. Frame 10380 was already selected, so the whole run 10200 to 10380 is extracted.

I reproduced the selection with `sheet_map02.py` and saved it as `sheet_map02.out.json`: 484 pictures on 61 sheets. That agrees with root's figure, 447 + 37 = 484.

## Where to look

| What | Pictures | Sheets |
|---|---|---|
| Required run 10200 to 10380 (181 frames) | 166 to 346 | 21 to 44 |
| 179.8 s anchor (frame 10788), the c/c overlap, and the 2 s after it | 352 to 391 | 44 to 49 |
| 205.6 s anchor (frame 12336), the 4^n strokes | 401 to 412 | 51 to 52 |
| L12 fade-out (the lingering-c check) | 419 to 472 | 53 to 59 |
| L5 windows | 1 to 131 | 1 to 17 |
| L11 diagonal and walk | 137 to 305 | 18 to 39 |

## Evidence schema notes for my independent JSON

The gate applies `claimed()` to review evidence. So my JSON must have no key containing "accept", "release" or "publication" unless its value is exactly `false`.

It must also carry these fields:

- `movie_sha256`
- `extraction_sha256` (the sha of `08_loop_sums/pictures.json`)
- `source_sha256`, `a8d62f56...`
- `concrete_defects`
- `safe_to_advance_to_final4k`
- `viewed_frame_indices`, within the selected frames
- `authored_repair`, set to `false`
