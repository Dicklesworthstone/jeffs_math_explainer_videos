# Independent review: logarithm-stage09-render-preparation04

**Reviewer:** RubyRaven, for bead 87do.63 (BlueSalmon Mails 4552 and 4615).
**Reviewed:** LavenderPike's `P/logarithm-stage09-render-preparation04/`, SHA256SUMS `34b04c88` (Mail 4609).

**Scope.** This review covers the version 3 → version 4 delta and the capture-length rule. It is a source and data review plus scratch checks. It is not AV acceptance. No job, Cargo, kit or guard was run, and nothing was heard or viewed.

## Result: clean

I found no defect in the delta that would change or block production.

**What changed.** The delta is exactly the three stated repairs and their consequences:
- the timeline timeout;
- the render-job capture-completeness test;
- the master `verify-step` missing-track refusal;
- consequences: the binding schema is now `-v4`, and the writer message changed.

**Real-run evidence.** Version 4 already ran on real production work:
- root's series 02 timeline, score and renders 00–10 were all written by version 4;
- all 11 render jobs passed its capture test (exit 0, `render-summary.json` written after the test);
- I re-measured all 11 captures independently, and they agree.

## Evidence (`checks/review04_checks.py`: 15/15 PASS, two fresh runs byte-equal)

### D: the delta

- **D0. Seals.** The version 3 SHA256SUMS (`aa645803`, helper `6e5a602d`) and version 4 SHA256SUMS (`34b04c88`, helper `debd6f75`) both verify, 5/5 rows each.
- **D1. Diff.** I made my own line diff. It has the same changed lines as Lavender's `docs/v3-to-v4.diff`: −5/+57 lines in 9 regions. Two of those regions are close enough to share one unified hunk, which is why her diff shows 8.
- **D2. Template functions.** The job template gains `frame_to_sample` and `incomplete`. Of its other 16 functions, only `timeline` and `render` differ.
- **D3. Timeouts.** The only changed timeout is the timeline's kit run, 2400 → 7200 s. All the others are unchanged:

  | Run | Timeout |
  |---|---:|
  | verify | 900 s |
  | preview | 14400 s |
  | score | 7200 s |
  | render | 43200 s |
  | master | 14400 s |
  | captions | 1200 s |

### The capture rule against the kit's own code (read)

The rule is what the master enforces, so a capture it refuses would fail the master. It is a necessary condition, not a sufficient one.

- **Formula (F1).** The template's `frame_to_sample` is `explainer_score::units::frame_to_sample` (`crates/explainer_score/src/units.rs:41`), the same integer round-half-away. F1 checks it against exact rational rounding on 960,008 frame/fps cases.
- **What the master requires** (`crates/explainer_kit/src/cli.rs`, `master_video`):
  1. It reads `out/CH.timeline.tsv`, the file the render test reads.
  2. It takes the last contiguous `soundtrack_<n>.wav`, the same choice as the template's `capture()`.
  3. It needs `null_test` (`master.rs:126`) to find the capture byte-identical to the s16 replay.
  4. It needs the replay to have exactly `frame_to_sample(tl.frames, tl.fps)` samples.
  
  So a capture that is not 48 kHz s16 stereo with exactly that many frames, declared and present, cannot pass the master.
- **What the rule does not check.** It cannot see sample values. The master's byte comparison stays the sufficiency check.

### C: real and planted captures

- **C1. Real captures.** All 18 pass version 4's test: the 7 Nov 2 captures and the 11 series 02 captures.
  - An independent chunk walk finds each one with format tag 1, 48 kHz, 16-bit, stereo, `fmt ` then `data` only, and exactly `4 × frame_to_sample` data bytes, declared and present.
  - Each series 02 `render-summary.json` names its capture at today's hash, with `captures` 1.
- **C2. Planted captures** (in scratch, built from series 02's 02_slide_rule, under Python 3.15):

  | Case | Result |
  |---|---|
  | One frame cut | Refused |
  | Half-written copy (header for the whole file) | Refused |
  | Header only | Refused |
  | One frame more | Refused |
  | Mono | Refused |
  | Not a WAV | Refused |
  | Exact copy | Accepted |
  | WAVE_FORMAT_EXTENSIBLE header over the same PCM | Accepted (see N1) |
  | A trailing LIST chunk | Accepted (see N1) |

### J: the real series 02 jobs

- **J1.** The timeline, score and 11 render job scripts all name and bind helper `debd6f75` and carry its capture test.
- **J2.** No workspace job script was written by version 3: 14 are version 4's and 1 (timeline01) is version 2's. Version 4 accepts only its own jobs and version 2's, so nothing becomes unverifiable.

### Snapshot

**S1.** All 115 real files read are unchanged: size, mtime and sha256.

## Measured run times (E1, from the guard receipts and timeline02's log)

| Step | Measured | Timeout | Margin |
|---|---:|---:|---:|
| timeline02, kit run (116,245 frames) | 3,811.8 s | 7200 s | 1.89× |
| timeline02, whole job | 3,844 s | — | — |
| score02 | 1,079 s | 7200 s | 6.7× (Lavender estimated about 3,100 s) |
| renders 00–10 | 735–1,629 s each | 43200 s | ≥ 26× |

## Findings

### A1 (Low, future timelines only; no action for series 02)

The measured margin is 1.89×. The worst case in Lavender's margin table is optimistic:
- **Her assumption:** the whole film at timeline01's slowest rate, 17.9 fps (chapter 06), giving 6,470 s and a 1.11× margin.
- **What timeline02 measured:** chapter 12 ran at **12.2 fps**. The whole film at that rate would take about 9,528 s, which is more than 7200.

Series 02's timeline is finished, so nothing needs to change now. If another whole-film timeline is ever needed, for example a series 03 for new timing corrections, consider 14400 s, as preview and master already use. The guard has no wall-clock limit, so a longer timeout costs only detection time when a job hangs.

### N1 (nit, no action)

The rule accepts an EXTENSIBLE header or a trailing chunk, as long as the frame counts match. The kit's capture is a byte copy of fmn's own file, and all 18 real captures are tag 1 with only `fmt ` and `data` chunks. The master's byte comparison would still refuse either form. In practice this is not a gap.

### Production state at check time (not claimed as reviewed)

- Render 11's job script is written (by version 4); its job had not run.
- Render 12 and the master do not exist yet.
- Root's own replay of Lavender's checks (Mail 4615) is root's evidence, not mine.

## Limits

- **Not claimed:** listening, viewing, fine sync, AV acceptance or master success.
- **Not run:** Lavender's `run_checks.py`, the version 4 helper's subcommands, the kit and the guard.
- **Not checked:** the captures' sample values. Only the master's null test can check them.
- **Absolute paths:** none appear in this bundle. W is root's native folder, passed as `--w`.

## Replay

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-render-preparation04-review01/checks/review04_checks.py SCRATCH --w W
```

SCRATCH must be new and outside the repository. The run takes a few seconds and writes about 107 MB of planted WAVs into scratch.

**Caveat:** if root launches render 11 or 12 between the two runs, the outputs will differ. S1 would then also report the new capture as a changed file.
