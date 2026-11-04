# Logarithm chapter 10 interval repair: cross-review

Reviewer: RubyRaven, for BlueSalmon, Mails 1973 and 1995, bead `mx-video-program-program-87do.63`.
Reviewed: root's `logarithm10-interval-repair01` (candidate `eeee764f…`, base `535e9855…`), sheets, cues and source.
Narration stays ElevenLabs Oliver. No audio was generated or listened to.

**This is a source and baseline-picture review. No candidate picture exists yet. It is not listening, fine-sync or final AV acceptance.**

## Verdict

**The candidate is correct and narrow. I recommend taking it. I found no defect in it, so I stage no correction.**

| Question | Answer | Evidence |
|---|---|---|
| Is the defect real? | **Yes.** In the baseline the strict rule is visible while +π is still closed for 2.73 s at the scratch rate | Sheets 038 to 040; frame-by-frame measure of the baseline render |
| Are the text and endpoint swap simultaneous? | **Yes, from source.** Same call, same run time, same rate, no lag | Source and engine read; not seen in candidate pixels |
| Is timing preserved? | **Yes.** From the phrase on, the timeline lands on the same frames | Frame arithmetic, with the model checked against the baseline render |
| Do the words or maths change? | **No.** The patch only moves one fade | Patch read; `check_cues.py` listing identical |

## Disclosure

- I did not write root's candidate.
- This is not independent of lineage. As RubyRaven I wrote the original rebuild of `doing_nothing.rs`, and the early reveal may descend from my own round-1 code. WhiteSwan's repair added the closed-to-open swap (F4, `logarithm-repair08-10/REPAIR.md`). I also wrote the final80 bound plan whose anchor file names this phrase.
- Root and I are the same kind of agent, so we may share blind spots.
- In my previous assignment I logged sheets I had not seen (Mail 1992). Here I wrote each coverage note only after its image was on screen, and `checks/seal.py` refuses a header time that differs from `sampling.json`.

## Evidence and limits

| Source | Supports | Cannot show |
|---|---|---|
| 7 sheets viewed: 035, 037 to 041, 050 (`coverage/sheets-viewed.tsv`) | The baseline page at sampled instants | Candidate pictures. Sheets 000 to 034, 036, 042 to 049 and 051 to 057 are root's view, not mine |
| 2 frames viewed at full size: 159, 163 | Filled against open endpoint at 720p | Same |
| 44 baseline frames decoded and measured, not viewed (`coverage/frame-series.tsv`) | When today's swap and rule fade run, to the frame | Candidate pictures |
| Candidate and base source, engine at pin `a399d626` | Order, run times, rate, lag | Compile, types, borrows: **nothing was compiled** |
| `checks/verify.py` | Patch round trip, `rustfmt --check`, hooks, cue checker, timing model | The real Oliver phrase time, which is unmeasured |

## The defect, measured

10_4 opens: "So use the same rule: keep the turning strictly between minus pi and pi."

In the baseline render, `keep −π < turning < π` starts at the cue:

| Event | Frame | Time, s |
|---|---|---|
| Cue 10_4 starts | 2013 | 67.10 |
| Rule first visible | 2014 | 67.13 |
| Rule full | 2022 | 67.40 |
| "strictly between" begins, scratch model | | 69.81 |
| π starts to open, first visible | 2096 | 69.87 |
| π fully open | about 2103 | about 70.1 |

So the card says strictly less than π, over a filled dot at π, for **2.73 s**. Root's figures (67.3, 69.5, 70) are sampled instants and agree with this.

## Simultaneous?

The candidate's line 560 is `swap(stage, &[closed_r], &[open_r, rule], 0.3)?`.

- `swap` (`draw.rs:257`) prepares one fade-out and two fade-ins, each with `.rt(0.3)`, and passes all three to a single `play_prepared`.
- `fade_in` and `fade_out` (`fmn-anim/src/fading.rs`) are `Transform`s on the default config: `lag_ratio = 0`, `rate_func = smooth`. So the rule fades as a whole, not letter by letter, on the same rate as the ring and against the dot.
- In the baseline the same dot and ring take about 9 frames at 30 fps (frames 2095 to about 2103), and the rule fades evenly in its own 0.4 s. The frame at 70.0 (sheet 040) shows the ring over a fading dot.

The rule's fade drops from 0.4 s to 0.3 s. That does not matter for reading, because it then stays to the end of the chapter (sheet 050).

**No flash from waiting.** `rule` now waits detached for about 2.7 s, as `crosses` already does across the same `until_phrase`, and as `numbers_word` does across a fade at lines 425 to 433. Detached mobjects are not drawn, and they stay valid handles.

## Timing preserved?

`until_phrase` waits from wherever the clock stands to the first frame at or after the phrase, so its landing does not depend on how the clock got there.

- **Same landing:** with or without the old 0.4 s fade, the swap starts on the same frame (2095 at scratch). This held in all 106 cases tried: voices from 0.4× to 3× the scratch rate, at 30 and 60 fps.
- **The model matches the render:** it predicts frame 2095, and the baseline's first visible change is frame 2096, where the rate's slow start would make it.
- **The exception cannot arise in practice:** the two paths differ only if the phrase starts inside the first 0.4 s of the cue. It is byte 39 of the line, about 2.7 s in at scratch rate, and would need a voice nearly 7× faster.
- **Everything later is unchanged:** only lines 556, 557 and 560 differ, and the line count is the same. The swap and crosses still take 0.7 s after the phrase, leaving 1.66 s before sentence 1 at scratch rate. That runs out only at a voice about 3.4× faster, the same as the baseline.
- **One difference:** with no narrator `until_phrase` waits for nothing, so a silent run is 0.4 s shorter. The previews and final use a narrator.

## Why it teaches better

During "So use the same rule: keep the turning", the page still shows the closed window, which is the number's rule the voice means. At "strictly between", the words and the open end arrive together, and the voice goes on to "minus pi and pi" as the crosses land on ±2π.

## Trade-off, low: a longer static page

The candidate removes the only motion between the lap marks and the swap. The page is full by 63.5 s (sheet 035) and has no pixel changes in my coarse scan until the cue.

| | Static run at scratch rate |
|---|---|
| Baseline | About 3.6 s, then the rule's fade at 67.1, then about 2.4 s |
| Candidate | About 6.3 s, 63.5 to 69.83 |

This adds to the long-static concern root already holds, and it spans a cue boundary while the voice is active. It is no reason to reject a repair of a false picture. Recheck it on Oliver timing.

## Noted, not caused by the candidate, not patched

- **N1, low:** the closed dot stands unlabelled under a flow's heading. `numbers_word` leaves at 10_3's third sentence (line 454, about 46.4 s at scratch), so for about 23 s "a flow's turning in one unit of time" sits over a window closed at π, with nothing saying that end belongs to numbers. I saw this only from 62.5 s on; the earlier start is from source. No rule is claimed during that time, so it is not a contradiction.
- **N2, low:** the open ring is not fully hollow. The 6 pt green window line runs to each ring's centre (frame 163). It stays clearly distinct from the filled dot (frame 159).
- **N3, root's open item:** the dim fine print from 90 s and the long holds. I viewed only sheet 050 here. The candidate does not touch them. The fine print's `−π < Im < π` agrees with the open ends.
- **N4, optional doc:** `docs/storyboard.md:160` stays true but does not mention the words. If root wants: "At 'strictly between' the +π end opens and the rule's words arrive with it."

## If root integrates

- `interval.patch` round-trips from canonical to the candidate, and `rustfmt --check` passes. Canonical still equals the patch's base.
- Root's `combined-repair-stage05/bound-plan/` binds the source hash **8 times in 4 files**: `phrase-anchors.json` 3, `hold-audit.json` 2, `narration-direction-map-oliver12.json` 2, `inputs.sha256` 1. These need rebinding.
- The plan's 12 `doing_nothing.rs` line references need no change: none points at a changed line, and the `until_phrase` hook stays at 558. I did not examine what the unit digest covers beyond `script.tsv` being unchanged.
- `check_cues.py` passes on both copies, with an identical listing.

## Still required

Root's list stands: native compile and scoped checks, a guarded preview, real Oliver timing, listening, independent AV.

**The half-second sampler can miss this transition.** Its samples at 69.5 and 70.0 would straddle the swap. For the first candidate render, please add frames around the phrase. At scratch rate those are:

| Frame | Time, s | Expect |
|---|---|---|
| 2094 | 69.80 | No rule; π filled |
| 2099 | 69.97 | Rule, ring and dot all part-way |
| 2104 | 70.13 | Rule full; π open; crosses just starting |

Measuring as `coverage/frame-series.tsv` does, the rule's box should stay at background until the swap, and its share should track one minus the dot's share within a frame. On a voiced render, recompute the frames from the phrase's time.
