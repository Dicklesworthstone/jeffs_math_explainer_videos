# Preview 274 helper: notes and handoff for review

cc3, 2026-11-08. These notes go with one prepared helper, `cc3-preview-fade274.py`. It is meant for
a guarded native 540p60 preview of the nine chapters whose raster plates now fade.

- **What I did not do.** I did not launch, deploy or run it on the host. I ran no render, no
  ffmpeg and no build. I edited no candidate, canonical file, TODO, Beads entry or memory.
- **Who runs it.** Root reviews it, and CC2 reviews it independently. Root alone runs it, under
  root's own exclusive guard, as job `csd-review-repair-preview274`.
- **Narration.** ElevenLabs Frederick Surrey, unchanged.
- **What it decides.** Nothing. It picks frames for people to look at. It makes no picture,
  motion, 4K, listening, fine-sync, publication or final-AV claim.

| File (in `production-repair17/`) | sha256 |
|---|---|
| `cc3-preview-fade274.py` (908 lines) | `d80cf7b6ffba56968f7c6dfd7a2cca50e65769dce1c97deb5d34683ed3fc9669` |
| base it adapts: `review-repair-preview243.py` | `3e5dd06c48b888e4a947a71f57929121fdb76590b0a6e5dfc3e21bd5f1e23c17` |

## 1. What the helper does, in order

1. **Admission and inputs.** It stops unless every input checks out:
   - root's `resource-receipt.json` in the job directory is admitted with no exit, and uses the
     same policy as preview243 (`stop_free_gib` 8, `phase_growth_gib` 4);
   - `work/bin/review-repair268` hashes to build268's `binary_sha256`;
   - build268's `qualification.json` reports 233 tests, the native plate-fade probe and Clippy
     all passed, and lists all nine chapters;
   - the nine chapter files plus `draw.rs` and `mod.rs` in `work/review-repair268/workspace`
     hash to the values embedded in the helper. Those values must also match build268's
     `source-copy.json`.
2. **Source check, before any render.** It checks its site table against the compiled source
   text, line by line (section 3). Any drift stops it.
3. **Render.** It renders the nine chapters exactly as preview243 rendered two: `render` with
   `--res 960x540 --fps 60 --threads 16 --narration <rebind269> --lint --crf 18 --preset
   veryfast`. The working directory is `work/production-repo07`, and the process is pinned to
   16 CPUs.
4. **Exact schedule comparison, per chapter.**
   - The preview's `timeline`, `cues` and `cuelog` tables must equal timeline269's output byte
     for byte. Both use the rebind269 narration path.
   - They must also equal production timeline21 after timeline269's named path mapping.
   - Timeline269's `comparison.json` row must say equal, with the same sha256.
   - The frame total must equal timeline269's frame count and ffprobe's `nb_frames`.
   - The video must be 960×540 at 60/1.
5. **Chroma.** It makes one ffmpeg `signalstats` pass per chapter and keeps one number per frame
   (SATAVG, mean chroma) in `chroma.txt`. No pictures are written in this pass.
6. **Placement.** It places every plate fade (section 4).
7. **Picking and output.** It picks frames, each with every reason it was picked (section 5). It
   then extracts them with preview243's select filter and builds sheets of eight, as preview243
   did. Each sheet label also carries the first reason.
8. **Records.**
   - Per chapter, `sampling.json` holds the schedule rows, every fade's placement with its
     fit or cross-check numbers, the fade flags, the samples with their sha256s, the sheets
     and the scope.
   - `summary.json` holds the counts.
   - All JSON is strict, with no `Infinity`.

Every output is created exclusively: `open('x')` and `mkdir()` without `exist_ok`. A second run
into the same job directory therefore stops instead of overwriting.

## 2. Inputs it reads on the host

The paths are the ones root's scripts use:

| Input | Path |
|---|---|
| binary | `work/bin/review-repair268` |
| narration | `jobs/csd-review-repair-rebind269/narration` |
| build268 qualification and source copy | `jobs/csd-review-repair-build268/qualification.json` and `.../source-copy.json` |
| timeline269 | `jobs/csd-review-repair-timeline269/comparison.json` and `.../timeline-output/` |
| production baseline | `jobs/csd-production-timeline21/timeline-output`, with assembly19's narration path mapped |
| compiled source | `work/review-repair268/workspace/explainer_corpus/complex_analysis_visual/src/chapters/` |

**One path to confirm.** `source-copy.json` sits next to `qualification.json` in root's local
terminal copy (`review-build268-terminal272/`). I assumed it is also in the remote build268 job
directory, but I have not seen that directory.

**Timeline269 must finish first.** Root reports it still running at 06:17, with seven chapters
complete. I have seen none of its output. The helper reads its `comparison.json` and its tables
for all nine chapters before comparing, and stops if any is missing or unequal.

## 3. The plate-fade sites

Candidate 264 gives a plate its own opacity ramp in any play whose targets hold an image. That
covers Painter `show`/`leave`, and any `fade_in_all`/`fade_out_all` whose list holds a plate
group. I traced every such play in the nine files by hand.

- **Count.** There are 44 sites and 51 plays: the gallery's six panels are one site in a loop,
  and residues' three leaves are another.
- **Bytes.** The files I read are byte-equal to what build268 compiled (`source-copy.json`).
- **Not plates.** Walkers, the dog walk, the code panel and one_point's mosaic are all vector,
  so they are not sites.

**How a play maps to frames.** These rules are read from fmn `a399d626`:
- **Length.** A play of run time rt lasts ceil(rt × 60) frames, computed on the exact double
  (`clock.rs` `frames_covering`). So 0.4 s is 25 frames, 0.8 s is 49, 0.1 s is 7 and 0.2 s is 13.
- **Position.** If it starts after s frames, it fills video frames s … s+N−1.
- **Ramp.** Frame s+k−1 shows the plate at smooth(k/(60·rt)) going in, or 1 − smooth going out.
  The rate is the quintic smoothstep (`rate.rs`), and the alpha is t/run_time (`frame.rs`).

**Anchors.** Each site's anchor is one of these:

| Anchor | Start frame | Sites (plays) |
|---|---|---|
| `line` | the cue's line row, plus the fixed plays between the `say` and the site | 16 (18) |
| `breath` | the chapter's last breath row + 120, plus the closing vector fade | 7 (7) |
| `after` an exact site | that site's end | 10 (15) |
| `fit` | no exact anchor: the site follows a measured sentence start or phrase | 9 (9) |
| `after` a fitted site | that fit + the fixed plays between | 2 (2) |

So 33 sites, holding 40 of the 51 plays, are placed exactly from source and timeline.

**Why the line and breath rules hold.** I read them in the local `explainer_kit`:
- `Narrator::say` records the line at the current frame after its gap, and returns there.
- `music_breath` opens its window at the current frame, then waits `frames_covering(2.0)`.
- `Kit::finish` adds no frames.

**What the source check refuses** (`check_sources`, run before the render):
- a site, step, anchor, wait, evidence or end line whose exact text differs at its line number;
- a run time that differs from the table. `TILE_BEATS` is read from gallery line 161.
- any timed call between an anchor and its site that is not listed. The timed calls are
  `say`, the fades, show/leave, play, pause, the waits, breath, `names_now`, `predict` and
  tweens.
- any such unlisted call between a fitted site's wait and its modelled plays. The exception is
  two lines inside residues' `if k == 2` block, which loop k = 0 does not run.
- any line naming `.show(stage`, `.leave(stage`, `portrait(`, `.group`, `.field` or an image
  that is neither a site nor an allow-listed vector use with its reason.

**The site table, with exact starts.** I computed the starts below locally from production
timeline21's tables in `timeline21-terminal01/`, and from nothing else. Timeline269 is still
running, so nothing here shows it equal to timeline21. The helper does not use these numbers. It
computes the starts again from the preview's own tables, after requiring those tables equal to
timeline269's and timeline21's. "End = total" means the site ends on timeline21's frame total,
an exact check of the anchor arithmetic.

| Chapter | Site | Source line | Call | Cue | Placement | Start (s) |
|---|---|---|---|---|---|---|
| 01 | A1 | the_bump:330 | chunk fade in 0.5 (holds the z² preview) | 01_6 | fit | — |
| 01 | A2 | the_bump:349 | closing fade out 0.6 (holds the preview) | 01_6 | breath; end = total 6311 | 104.583 |
| 04 | B1 | reading_color:221 | identity show 1.2 | 04_1 | fit | — |
| 04 | B2 | reading_color:438 | leave 0.5 | 04_7 | breath; end = total 6774 | 112.400 |
| 05 | C1 | gallery:407 | z³−1 show 1.0 | 05_1 | line | 0.600 |
| 05 | C2, C3 | gallery:493, 497 | leave 0.5, then e^z show 0.8 | 05_3 | line, after | 33.183, 33.683 |
| 05 | C4, C5 | gallery:564, 568 | leave 0.5, then sine show 0.8 | 05_4 | line, after | 48.267, 48.767 |
| 05 | C6, C7 | gallery:682, 686 | leave 0.5, then Γ show 0.8 | 05_6 | line, after | 79.567, 80.067 |
| 05 | C8, C9 | gallery:750, 757 | leave 0.5, then ζ show 0.8 | 05_7 | line, after | 103.050, 103.550 |
| 05 | C10 | gallery:807 | strip show 0.8 | 05_7 | fit | — |
| 05 | C11, C12 | gallery:896, 897 | ζ leave 0.4, then strip leave 0.4 | 05_9 | line, after | 145.283, 145.700 |
| 05 | C13 ×6 | gallery:903 | six panel shows, 0.25 each | 05_9 | after | 146.117 (+0.25 each) |
| 05 | C14 | gallery:942 | five panels out, `TILE_BEATS[1]` 0.3 | 05_9 | fit | — |
| 05 | C15 | gallery:960 | zoom show 0.5 | 05_9 | after fitted C14 | — |
| 05 | C16, C17 | gallery:988, 989 | panel leave, then zoom leave, 0.5 each | 05_9 | breath, after; end = total 9879 | 163.650, 164.150 |
| 07 | D1 | one_point:419 | \|z\|² show 1.0 | 07_3 | fit | — |
| 07 | D2 | one_point:585 | leave 0.5 (vector mosaic fades in next) | 07_5 | fit | — |
| 09 | E1 | boundary:474 | show 1.0 | 09_1 | line | 0.600 |
| 09 | E2 | boundary:1398 | leave 0.5 | 09_9 | breath; end = total 16504 | 274.567 |
| 10 | F1 | rigidity:457 | sin(1/z) show 1.0 | 10_4 | line | 62.283 |
| 10 | F2, F3 | rigidity:536, 562 | leave 0.5, then sin z show 1.0 | 10_6 | line, after | 104.050, 104.550 |
| 10 | F4 | rigidity:648 | leave 0.5 | 10_6 | breath; end = total 7442 | 123.533 |
| 11 | G1 | counting:196 | identity show 0.8 | 11_1 | line | 0.600 |
| 11 | G2 | counting:519 | leave 0.4 (strips field added, without a fade, just before) | 11_7 | line | 124.433 |
| 11 | G3 | counting:567 | field out with the acted picture, 0.3 | 11_7 | fit | — |
| 12 | H1a, H1b | residues:282 (k = 0, 1) | two small shows, 0.6 each | 12_3 | fit, after fitted | — |
| 12 | H1c | residues:282 (k = 2) | small show 0.6 | 12_3 | fit | — |
| 12 | H2 ×3 | residues:324 | three leaves, 0.4 each | 12_4 | line | 50.450 (+0.417 each) |
| 12 | H3 | residues:329 | bump show 0.8 | 12_4 | after | 51.700 |
| 12 | H4 | residues:362 | bump leave 0.5 | 12_4 | breath; end = total 3943 | 65.217 |
| 14 | I1 | continuing:275 | bump show 1.0 | 14_1 | line | 0.600 |
| 14 | I2 | continuing:321 | leave 0.5 | 14_2 | line | 20.417 |
| 14 | I3 | continuing:686 | lacunary show 1.0 | 14_7 | line | 147.967 |
| 14 | I4, I5 | continuing:738, 747 | leave 0.4, then blank-Γ show 0.1 | 14_8 | line, after | 170.100, 170.517 |
| 14 | I6 | continuing:799 | leave 0.5 | 14_8 | breath; end = total 11756 | 195.433 |

**Agreement with earlier records.**
- **Rigidity.** F1 at 62.283 s matches rigidity247's "62.3–104 s" plate window, and F2 at
  104.05 s ends it.
- **Gallery.** 05_9's line at frame 8692 and the gallery breath at 9669 match gallery249 and
  cc1's review255.

**Two sites to expect.**
- **G2.** The quintic strips field is added with `add_to_scene`, not faded, just before the
  identity plate's 0.4 s leave. This is existing production design, which the repair does not
  touch. The G2 frames show it.
- **I5.** This is a blank painter (`Painter::build(..., None, ...)`) fading in for 7 frames.
  Little or no visible change is expected there.

## 4. Fitted sites: how they are placed, and how much to trust them

**Why a fit is needed.** Nine sites follow `wait_sentence` or `wait_for`. Their frame depends on
measured sentence starts and phrase marks in the narration report, together with the kit's wait
rules. None of that is in the timeline tables: there is no play log, and these chapters emit no
marks at their fades. Reproducing it would mean porting the kit's narration timing.

**The fit.** The helper fits the exact source play sequence around the site to the film's
per-frame chroma, by least squares:
- that sequence is the contiguous plays before it (`pre`), the site, and the plays after it
  (`post`), with their fixed offsets and run times;
- every already-placed neighbouring play is held fixed in the model;
- the visibility is raised to each power in γ ∈ {1, 0.75, 0.55, 0.4}, because the renderer blends
  in linear light and the film is sRGB, so chroma is a concave function of visibility;
- the search runs inside the site's cue, between its placed neighbours.

**Acceptance.** A fit is accepted only if all three hold:
- the least plate slope t-statistic is at least 8;
- the plate slope is at least 0.25 SATAVG;
- no separate candidate scores 0.8 of the best.

**When a fit fails.** The site is marked NOT LOCALIZED. The helper then samples its search window
every 15 frames, or every 30 if the window is over 60 s, and records that as the reason. It never
guesses a position.

**Cross-checks on the exact anchors.** The same model is evaluated at each exact anchor and at
±8 frames around it, and the best shift is recorded.
- An exact anchor is never moved.
- It is flagged, and the frames at the shifted position are added, only if the best shift is
  over 3 frames, scores at least 8, and beats the anchor's score by 1.5 times.

**What I tested locally.** These were synthetic signals, not real frames:
- I placed 12_residues' sites at known frames, added distractor fades of other sizes, and used
  response powers 1.0, 0.55 and 0.45 with noise σ 0.03. Both fits and H1b landed within 0 to 2
  frames of the truth, and no exact anchor was flagged.
- **A bias I fixed.** The first version, without the γ search and the fixed neighbours, fitted
  4 frames early and raised a false flag on H2. I fixed both before writing these notes.
- **A plate with no signal** (H1c at zero amplitude) was refused as NOT LOCALIZED, but only by
  the ambiguity rule. Its best candidate still scored t = 50.9 with slope 0.75, far past both
  floors. With no plate in the signal, that fit can only have come from other fades.

**Limits of the fit.**
- **The thresholds are untuned.** They have never been tried on real SATAVG values, and I have
  not estimated any real site's amplitude. A small or low-chroma plate gives a weak signal and
  may come out NOT LOCALIZED.
- **A wrong fit could be accepted.** One that is unambiguous and latched onto a colourful vector
  fade would pass; the negative test above shows such a candidate scoring t = 50.9. One_point's
  mosaic and the colour-wheel legend are the risky neighbours, and both are modelled in D2 and
  B2.
- **What the reviewer must do.** Treat a fitted placement as a hypothesis, and confirm it on the
  sheets: the frames around a fitted start must show the plate ramping. `chroma` is recorded on
  every sample to help with that.

## 5. Which frames it picks

The set is bounded. There is no whole-film or every-frame sampling, and every reason is kept per
frame.

**Every chapter:**
- the first and last frames;
- every timeline `line` and `mark` row at +0, +0.5, +1 and +2 s (preview243's rule);
- every current layout-lint line at −0.2 to +2 s, and every 5 s after it while the lint says
  it persists (preview243's rule).

**Every plate-fade play** (each repeat on its own):
- the frame before the start;
- the start (ramp frame 1);
- the middle (ramp frame ⌈N/2⌉);
- the last frame (ramp frame N);
- the first hold frame;
- 0.5 s into the hold.

**Added for some plays:**
- **Fade-outs:** every frame from N−3 to N+3. This catches any reappearance at the remover
  frame.
- **Fitted and after-fitted plays:** a ±3-frame bracket at the start and at the end.

**Old anchors (05).** These are gallery249's qualifier and cross times, each sampled at −0.2, 0,
+0.2, +1 and +2 s:
- "nearly square" at 153.92 s;
- cc1's 154.0 and 155.0 s stills;
- sentence 4 between 157.0 and 157.6 s;
- the take end at 160.547 s;
- the closing fade at 163.15 s.

The start, middle and last frame of the closing 0.5 s vector fade are added too, since the
qualifier and cross leave in it.

**Old anchors (10).** These are rigidity247's windows:
- 12.0–16.7 s every 0.25 s, for q, its backing ring and the pile-up (R1, R2);
- 62.3–104 s every 2 s, for the plate top, title and "Im" (R3);
- 88.0–103.6 s every 1 s, for the plate foot against the "precisely" card (L1).

**Why the old times can be reused.** The helper stops unless the preview's schedule equals
production timeline21's. So if it runs to the end, times from earlier records point at the same
moments.

**Size.** Estimated on the timeline21 tables, with fit positions stubbed, this comes to about 784
frames on about 98 sheets, plus lint lines. If many fits fail, the fallback windows could add
several hundred frames.

**Removed from preview243.** It sampled whole films once a second. Here that would add about
1,350 frames, or 169 sheets. I left it out under "avoid whole-film explosion". Restoring it is
one `add` loop if root wants it.

## 6. Cost to expect

- **Render.** About 81,032 frames across nine chapters, 1,350.5 s of film. Preview243 averaged
  14.6 fps on its two chapters; at that rate this is roughly 1.5 h, but chapters differ.
- **Chroma pass.** One more full decode of each 540p film.
- **Fits.** Pure Python, about a minute or two in total by my estimate. It is unmeasured on the
  host.
- **Disk.** Unmeasured: nine 540p films, 1,350.5 s in all, plus about 800 JPEGs and about 100
  sheets.

Root's guard owns admission, CPU, memory and disk.

## 7. What I ran, and what is not established

**Run locally, against local files only:**
- an `ast` parse of the helper;
- `check_sources` against my local composed tree, which equals build268's compiled bytes for all
  11 files. Five drift controls were each refused:
  - a changed run time;
  - an inserted pause between a cue anchor and its site;
  - an inserted pause inside a fit model;
  - an extra plate leave;
  - a changed `TILE_BEATS`.
- the synthetic placement tests in section 4;
- exact placement and every end check on the real timeline21 tables (section 3). These ran
  against timeline21 only; nothing ran against timeline269, which is still running.

The scratch scripts are in my session scratchpad and are not delivered.

**Not established:**
- **The helper has not run** on the host or on real frames.
- **No real-host step has been exercised:** ffmpeg `signalstats`, the metadata parse, the render,
  extraction or the sheets.
- **The fit thresholds are untested** on real frames.
- **The site table is my own reading.** The completeness scan is a text guard, not a proof: a
  plate group passed through a helper under another name would slip past it.
- **The say and breath rules come from the local kit.** Root's TODO (05:12 UTC) records the
  kit's `lib.rs` equal locally and on the host. `rigidity-kit-binding254.json` records
  `props.rs`, `plot.rs` and `animatic.rs` equal to the production manifest. `narration.rs`,
  which holds these rules, is in neither record, and I did not compare it.
  - The 7 breath-anchored chains are confirmed exactly by timeline21's frame totals.
  - The 16 line-anchored sites, and the 9 exact after sites chained to them, rest on
    `Narrator::say` returning at the recorded frame. Only the run-time chroma cross-check tests
    that.
- **Timeline269 is unfinished.** Root reports it still running at 06:17, with seven chapters
  complete. Until it finishes equal, the helper cannot run, and none of my numbers bind to
  rebind269.
- **No acceptance.** Nothing here is a picture, motion, 4K, listening, fine-sync, publication or
  final-AV acceptance.

## 8. Handoff for CC2's independent review

Please review exactly `cc3-preview-fade274.py` at `d80cf7b6…`. It is source only, with no runs.
The questions are these:

1. **Sites.** Is every plate-bearing play in the nine compiled files listed? Is each site's
   direction, run time and cue right? Start from section 3, with candidate 264's
   `image_records`, `fade_in_all` and `fade_out_all`.
2. **Exact anchors.** Do the `line`, `breath` and `after` chains match the source, and is
   ceil(rt × 60) frame counting right? For the wait rules, see `frames_covering` in fmn
   `clock.rs` and the kit's `Narrator::say` and `window`.
3. **Fits.** Is each fitted site's `wait`, `pre` and `post` the full contiguous sequence? Is
   `skip` {275, 278} correct for residues k = 0? Are the acceptance and NOT LOCALIZED fallback
   honest?
4. **Picks.** Does every fade get its start, middle, last, first-hold and post-hold frames, and
   fade-outs their N±3 bracket? Are the gallery249 and rigidity247 anchors the right ones?
5. **Safety.**
   - It writes only inside `jobs/csd-review-repair-preview274`, with exclusive opens, and runs
     only after root's admission.
   - The schedule comparison is exact.
   - The `source-copy.json` path needs confirming (section 2).

Root reviews the helper before any use.
