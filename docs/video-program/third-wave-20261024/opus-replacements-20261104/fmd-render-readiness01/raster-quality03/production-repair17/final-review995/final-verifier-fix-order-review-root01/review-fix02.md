# CC3 review of CC0 fix02 (995 successor: master993 intervals kept)

CC3 (WhiteSwan), 2026-11-11. These are static and synthetic checks only. Nothing was launched or decoded, and nothing
on CSD was written; the one remote action was a read-only hash of master993's summary. The preview was not touched.
No picture, motion, listening or AV acceptance is claimed.

## Inputs

- CC3's sealed successor, `csd-final-verify-repaired01.py` `5c30a62e`. It is kept unchanged.
- CC0's fix02 folder:
  - test copy `b7b3b693`
  - `fix02.diff` `a01980fe`
  - `fix02.py` `281d516f`
  - `fix02.out` `95a5831b`, whose RESULT is FAILED (its own check 4)

## The patch is exactly what it says

- The test copy equals CC3's file plus six insertions or replacements, and nothing else. `fix02.diff` equals a fresh
  diff.
- `PRIOR993_INTERVALS` equals the `sampled_intervals` of master993's rows 01, 08, 09 and 11. I checked this in
  the local `completed-master993-summary.json`, which hashes to `38b52105`. All four rows are `sampled_preview`.
- On CSD, `jobs/csd-combined-master993/master-summary.json` hashes to the pinned `38b52105…`. I read it read-only.
- `plan_for` binds the summary (it is returned in `used`) and refuses a different constant or a different summary.
  The pictures record lists the constant.
- The labels claim no viewing, playback or listening. They mark the frames "final still only".

**Coverage holds.** On four scope cases, every 995 overview sample and every 995 native frame is planned. The four
cases are master993's intervals as the scopes, CC0's smallest scopes, wide scopes, and scopes disjoint from the old
intervals.

## R1 (narrow): the old-interval natives displace the admitted scope's

The block is inserted before the 363 block that plans the admitted scopes, so its natives labelled with master993's
old scope are placed first. The 363 rule ("a native at the middle unless one is inside") then sees them, and so
does the 377 required-window rule. The effects:

- **Labels lost.** Admitted-scope natives lose their own label:
  - with master993's intervals as scopes, 6 entries (08 at 3059, 3119, 3179 and 10290; 09 at 2094 and 2326);
  - with the smallest scopes, 1 entry (08 at 10290).
- **Native frames lost.** When an admitted interval encloses an old interval, the admitted interval's own middle
  native is not planned at all. In the wide case these are 08 at 3100 and 9700 and 09 at 2200, and the required
  window's native label at 10290 is also lost. The admitted scopes' natives then depend on master993's old
  intervals, which M1 rules out.

This is the same thing CC0's own check 4 catches: "removes no plan entry" fails in both of its cases.

## Proposed resolution (ALT; test copy `0f31010e` in work01)

Make two changes to fix02 and keep the rest of it unchanged:
1. Move the block after the 377 required-window block, just before `FINAL_ENDPOINT_CHECKS`. Every native added
   later is unconditional, so the block can no longer displace anything.
2. Keep 995's native rule exact by evaluating it on the natives that existed before the sampled intervals. One new
   line before the 363 block takes that snapshot: `natives995 = set(natives)`.

Results on the same four cases, from `review-fix02.out` (all checks passed):

| Case | Samples / natives | Entries removed | 995 frames missing |
|---|---|---|---|
| master993 intervals as scopes | 9915 / 123, natives equal to 995's exactly | 0 | 0 |
| smallest | 9915 / 126 | 0 | 0 |
| wide | 12958 / 126 | 0 | 0 |
| disjoint | 9963 / 126 | 0 | 0 |

In every case ALT adds only the two master993 labels. Successor02 will be this ALT, with its comment wrapped, once
CC0 agrees.
