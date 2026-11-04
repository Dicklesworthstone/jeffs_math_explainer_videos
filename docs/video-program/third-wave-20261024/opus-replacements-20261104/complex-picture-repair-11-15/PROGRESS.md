# Progress: complex chapters 11–15 native repair

WhiteSwan, Mail 1655, bead `.39`. Times are UTC on 2026-11-04.

- 13:44 Reservation 751 granted. Seeded `baseline/` from canonical and `candidate/` from the frozen review's candidate. Mail 1747 to LavenderPike about `chain_arrows`.
- 13:51 LavenderPike confirmed: `complex-picture-repair-06-10/patches/00-draw-helpers.patch` owns the line. Mine stays a labelled fallback.
- Applied the report-only findings to chapter 11, then facts and oracle, 12, 13, 14, 15, registry.
- `registry_check.py` failed on a duplicate "radius ". Reused `the_bump::RADIUS` instead of a second const.
- `geometry.py` failed once on my own check: the derivative bound is tight at z = −1.6, so `truth <= bound` tripped on rounding. Added a tolerance. The source was right.
- `beat_times.py` showed the three condition cards of 11_2 all landing within about 2.3 s of a sentence about 10 s long. Moved them onto their words and recorded it as 11-Z.
- `cue_phrases.py` failed on a tool limit: helpers later in the file read the wrong cue. Mapped the two helpers to their cues.
- Added `16-storyboard.patch`, since the storyboard's picture column would otherwise contradict the source.
- `dead_code.py` passed first, which was not credible: a crate-wide name count let another chapter's `contour` hide this one. Tightened it. It then found `contour()` test-only; marked it `#[cfg(test)]`.
- Stacked all three lanes in the replay. Text applies cleanly. The duplicate-value test finds `\sin z` registered by both other lanes, reported in `HANDOFF.md`.
- Handed off. Frozen for root; nothing compiled or rendered.
