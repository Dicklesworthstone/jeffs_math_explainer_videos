# Logarithm stage08 audio preparation (bead mx-video-program-program-87do.63)

BronzeJaguar, 2026-11-05, for root (BlueSalmon). Answers Mail 3651, the owner's two voice messages, and Mail 3725.

This bundle holds a voice-agnostic helper for root's guarded use, a production binding for the retained Oliver logarithm takes, the read-only report it produced, and checks.

**What I did not do:**
- select or accept a take, or listen to anything;
- decode, assemble or time any production audio;
- synthesize, render, build, compile, or edit the candidate, canonical, receipts or reviews;
- touch Beads, the TODO or any commit.

**Recognition is not listening.** Every time in this bundle is a whisper-cpp estimate unless root replaces it with a sample.

## Voice status

The owner's refinement (canonical NARRATION-CONSISTENCY.md, 2026-11-05T03:22Z) keeps **Oliver** for logarithm. The 162 retained Oliver recordings are therefore production input. `bindings/logarithm-stage08-oliver.json` says `"use": "production"`.

04_4 and 08_5 stay open until root answers their questions or binds targeted Oliver replacements (see "After the clarity patch").

The helper names no voice (check F1). A binding names one voice, its inventories, its recognition runs and every pin. The same code serves a Frederick film through its own binding.

Two guards keep one voice per binding:
- **Downloads receipts.** Every take's downloads receipt must list it. Its download name must contain `voice.download_source_contains`; otherwise a voice question is raised.
- **Fresh timing per take.** Timings come only from each take's own recognition, or from a byte-identical twin, which raises a provenance question. Nothing carries timings between takes, chapters or voices.

## Files

| File | sha256 | What |
|---|---|---|
| `prepare_audio.py` | see SHA256SUMS | the helper: `report`, `decode`, `prepare`, `timings` |
| `bindings/logarithm-stage08-oliver.json` | `9e569c0e66ec995ae514c3d7e8489299cb3f159a8c0375efcf655d2b1973910b` | stage08 candidate, inventory, 5 recognition runs, bound-plan03 anchors/template/dispositions, assembler, kit; 30 pins |
| `report/*.json` | see SHA256SUMS | `report --out` on the real inputs (copied from a mirror run; `report --check` proves it in step) |
| `checks/run_checks.py`, `checks/outputs/run_checks-01.txt` | see SHA256SUMS | 45 checks, transcript |

Root's commits 6ef777a and e4882d0 swept intermediate copies of this folder while I was working, including `prepare_audio.py` `24fd6ff0…`. The sealed files here supersede them. My first, Oliver-specific draft `prepare_stage08_audio.py` was never delivered; I moved it to my scratch.

## Report (real inputs, `report/`)

**Coverage**
- 162 recordings of 80 cues, all 48 kHz mono PCM16.
- 160 recordings have their own large-v3-turbo recognition.
- 00_1 download03 and download04 have none. Their samples equal download02's, so they use its recognition and each carries a provenance question.
- The targeted small.en run corroborates; it does not place times.

**Sentence onsets: 568 of 570 supported.** "Supported" means the word agrees with the script inside a run of at least 3 agreeing words, is first in its one-word segment, and is in range and in order.

**The 2 unsupported onsets are 08_5 sentence 4, both takes.** The recognizer heard "Having" for "Halving", so there is no agreeing word. Each has a candidate: the recognizer's one word in that slot, between agreeing runs:

| Take | Candidate time | 24 kHz sample |
|---|---|---|
| download01 | 20.380 s | 489120 |
| download02 | 20.200 s | 484800 |

**Phrase anchors: 66 of 66 supported** (33 anchors × 2 takes).

**Abutting-onset caveat.** 429 of the 568 supported onsets sit in a word segment that begins exactly where the previous word ended. Such an onset may lie up to the segment's length before the word, at the start of the pause.
- The median segment is 0.24 s.
- 46 are at least 0.6 s and 5 are at least 1.0 s. The longest is 04_7 sentence 1, "Zero …", at 1.64 s in both takes.
- `coverage.json` lists all 46 by id.
- Root can replace any onset or mark with a measured sample in the answers file (see "Answers").

**Questions: 10 total, 8 blocking.**

| Id | Kind | Blocks | Decisions |
|---|---|---|---|
| `provenance:00_1:00_1-download03.wav`, `…download04.wav` | twin recognition | yes | `use_pcm_identity` |
| `voice:04_3:04_3-download01.wav`, `…download02.wav` | voice | **no (advisory)** | `confirmed_voice`, `reject_take` |
| `content:04_4:base:04_4-download01.wav`, `…download02.wav` | content | yes | `heard_as_script`, `reject_take` |
| `content:08_5:halving:08_5-download01.wav`, `…download02.wav` | content | yes | `heard_as_script`, `reject_take` |
| `onset:08_5:s4:08_5-download01.wav`, `…download02.wav` | sentence onset | yes | `use_candidate`, `sample` |

**04_4 base/space and 08_5 halving/having are preserved unresolved,** as Mail 3651 asked.
- Both models transcribe "space" in both 04_4 takes (check C6).
- Both models transcribe "having" in both 08_5 takes (check C7).
- These are transcripts, not hearing. Only root's answer or a replacement take clears them.

**04_3.** Its two browser downloads lack the voice name (`ElevenLabs_2026-11-04T22_56_43__s50_sim75_v4.wav`, `…22_56_38__…`). All other 160 downloads contain "Oliver - Clean, British and Steady" (check C3).
- Following Mail 3725, these questions are advisory, not blocking.
- Each binds the downloads receipt (path and hash) and quotes its contemporaneous provider observation: "Two matching Oliver provider history entries directly observed…".
- That is provider provenance, not a listening confirmation.
- `prepare` carries it into `prepare-receipt.json` as `voice_provenance`, and records root's answer if one is given.
- A take whose download name **and** receipt both omit the voice still blocks (check D13, the "unnamed" plant).

**`asr-differences.json`** lists 44 transcripts with normalized disagreements. It is diagnostic only. Apart from the two questions above, they are:
- numbers and notation: "a hundred" → 100, "thousand", "3/4" → "over four", "2 by 2" → "2x2", 1966, 1760, 3520;
- "and" omissions, "non zero", too/two, laps/lapse;
- possessives: three's, four's, five's;
- British and American spelling: centres, spiralling;
- in 08_5's small.en transcripts only, "roots" dropped or heard as "routes".

## Root's workflow

Run every command from the repository root with `python3 -I -B`. Each one writes only into a new folder named by `--out`.

```bash
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage08-audio-preparation01
B=$H/bindings/logarithm-stage08-oliver.json
python3 -I -B $H/prepare_audio.py --binding $B report --check $H/report        # in step, 5 files, 10 questions
python3 -I -B $H/prepare_audio.py --binding $B report --out NEW/report         # optional fresh copy
```

**1. Selection** (root's decision; `by` and `basis` required). It names every cue in script order, or exactly one chapter's cues with `--chapter NN`:

```json
{"schema": "audio-preparation-selection-v1", "binding_sha256": "<sha of the binding file>",
 "script_sha256": "b89eab60…", "by": "…", "basis": "…", "takes": {"00_1": "00_1-download01.wav", "…": "…"}}
```

**2. Answers.** Start from `report/answers-template.json`.
- Each answer needs `decision`, `by` and `basis`; `sample` also needs `sample_24k`.
- Blocking questions on selected files must be answered. Advisory ones may be.
- Any onset or mark id in `anchors.json` (`onset:<cue>:s<k>:<file>`, `anchor:<cue>:b<byte>:<file>`) may also be given `sample`, and an anchor may be given `omit_mark` (the kit then falls back to its estimate). This is how measured timings replace recognizer estimates.
- `reject_take` refuses that selection.
- An unknown id refuses the whole file.

**3. Decode** (root's guard). Each selected complete recording goes through root's earlier ffmpeg arguments: `-nostdin -n -v error -i SRC -ac 1 -ar 24000 -c:a pcm_s16le DST`.
- The output must be exactly ⌈n/2⌉ frames.
- `decode-receipt.json` is in coherent_narration's external format, with source `"ElevenLabs UI, Oliver"` and repository-relative paths.

```bash
python3 -I -B $H/prepare_audio.py --binding $B decode --selection SEL.json --out NEW/decoded [--chapter NN]
```

**4. Prepare.**

```bash
python3 -I -B $H/prepare_audio.py --binding $B prepare --selection SEL.json --answers ANS.json \
  --decoded NEW/decoded --reviewer NAME --out NEW/prepared [--chapter NN]
```

It writes:
- `spans.json`: whole recordings; sentence ends at the onsets, root's samples, or accepted candidates; every `gap_after_samples` zero; the binding's native-hold dispositions for the selected cues; `boundary_review.listening: "unreviewed"`.
- `workspace/` with byte copies of `tools/coherent_narration.py`, `tools/narration_cache.py` and the candidate's oliver15 map.
- `assemble-command.json`, `phrase-marks-plan.json` and `prepare-receipt.json`.

Before reporting success, prepare runs the spans through the workspace assembler's own `read_units`, direction checks, `unit_spans` and `check_dispositions`, writing nothing. If those refuse, it writes `REFUSED.txt` and exits 1.

**5. Assemble.** Run `argv` from `assemble-command.json` from the repository root. It is the workspace copy of `coherent_narration.py assemble --require-directions`.

**6. Timings.** Run `then` from `assemble-command.json` (`timings --prepared … --narration … --out …-timings`). It:
- reruns `coherent_narration verify --require-directions`;
- checks the manifest equals `spans.json`;
- checks each assembled cue equals its decoded PCM sample for sample;
- mirrors the kit's sentence count, normalized text and 0.05 s total checks;
- writes `phrase-timings.json` (every mark carries its basis);
- for a chapter assembly, writes `phrase-timings-fragment-NN.json` instead, because the kit needs every script cue.

Install with the `cp -n` line in `timings-receipt.json`.

## Blockers found and how the helper handles them

1. **Assembler anchoring.** `coherent_narration.py` resolves the units' direction map against its own tree (`REPO = parents[1]`, line 23; `direction_map_path`, lines 55–71). Canonical has a different script and no oliver15 map, so the repository's copy cannot assemble this candidate. Byte copies in `workspace/` fix that. Every other path resolves from the cwd, which is the repository root.
2. **24 kHz only.** `narration_cache.wav_identity` refuses anything but 24 kHz mono PCM16 (line 179), so decoding is required. It changes the format only.
3. **ffmpeg odd counts.** With root's arguments, ffmpeg 8.1.2 returns ⌈n/2⌉ frames for an odd 48 kHz count (check E1). Root's earlier `n // 2` agrees for every retained take, since all have even counts. The helper uses ⌈n/2⌉.
4. **External-take label (Frederick films, not logarithm).** `coherent_narration.py:275` accepts external UI receipts only when `source == 'ElevenLabs UI, Oliver'`. A Frederick receipt would fall into the FrankenTTS branch and fail (`KeyError: 'wav'`). The helper's dry run catches this and names it in `REFUSED.txt` (check D13, relabel plant). Extending that branch is root's decision; I did not edit the tool.
5. **Phrase timings must be complete.** The kit loads `phrase-timings.json` only if it holds every script cue (`narration.rs:914–917`), hence the chapter fragments.
6. **Stray file.** The stage08 candidate holds `src/chapters/doing_nothing.rs.orig`. The helper does not read it.

## After the clarity patch (Mail 3724: stage09, targeted Oliver replacements)

The helper needs no edit. Root writes a new binding:
- `tree` → the installed stage09 candidate; pin its `script.tsv`, `narration_units.json` and map.
- `phrase_anchors` and `anchor_template` → the rebound stage09 files (08_5 anchors at bytes 267/306). Pin them.
- `dispositions` → bound-plan03's file if the map's overrides are unchanged; otherwise a new proposal with its evidence pinned.
- `inventories` → keep `full80-retained-inventory01.json` and add the replacements' inventory. Each take needs `cue`, `wav`, `sha256` and `text_sha256`. Its `take_receipt` pattern points at downloads receipts that name the voice.
- `recognition` → keep the five runs and add the replacements' run: large-v3-turbo, `--split-on-word --max-segment-length 1`, frozen inputs pinned.
- `content_questions` → drop or replace the two; old-text entries are reported under `content_questions_for_other_text`.

The report will then show:
- the old 04_4/08_5 takes under `superseded_text_recordings`; they are kept, not selectable;
- the 78 unchanged cues with exactly their current onsets, since the bytes and recognition are the same;
- the replacements with onsets from their own run.

Check D10 shows this with a fixture: the replacement's onsets are its own run's (its twin's + 0.01 s), and the older-wording take is listed as superseded. D12 assembles chapter 04 with that replacement.

## Checks (`checks/run_checks.py`, 45 PASS, 0 FAIL)

Run from the repository root:
`python3 -I -B $H/checks/run_checks.py FRESH_SCRATCH > out.txt`

Two fresh runs gave byte-identical transcripts; `checks/outputs/run_checks-01.txt` is one of them.
- **A, B.** The bundled report is in step with the real inputs. Two fresh mirror reports equal it byte for byte.
- **C.** An independent recompute with a second tokenizer and unique-trigram search, not the helper's alignment:
  - 569 onsets and marks agree, none disagree; 63 have no unique trigram and 2 don't start with three plain words;
  - coverage, text hashes and voice names;
  - 08_5 sentence 4 unsupported;
  - "space" and "having" in all 4 transcripts each;
  - the exact 10 question ids;
  - only the 04_3 voice questions advisory.
- **D.** End to end in a scratch mirror, with every input a read-only symlink:
  - the full film through a test copy of the binding (`use: historical-test`, `--test-only`);
  - chapter 01 through the production binding;
  - chapter 04 with a fixture replacement.
  - Each run decodes, prepares, assembles with the real assembler copy and writes timings.
  - Plants, each refused with the right exit code: another binding's selection, a missing cue, an unanswered 08_5 onset, a rejected take, an unknown answer id, one sample short, an existing OUT, OUT inside the bundle, a stale pin (exit 3), an unanswered twin, an unnamed voice, a superseded-wording take, and an assembler that does not name the receipt source.
- **E.** Root's ffmpeg arguments on synthetic tones only: 48 kHz even and odd counts, and 44.1 kHz.
- **F.** The helper names no voice, and no bundle file contains an absolute home or scratch path.

**Test fixtures.** The selections and answers in D are fixtures: nobody chose or heard anything. Production audio was never decoded. D uses a labelled stand-in that keeps every other sample, in scratch only. Real ffmpeg ran only on synthetic tones.

## Limits

- There are no listening, fine-sync, render or native-kit results. The kit checks are a Python mirror of `narration.rs`, and I compiled nothing.
- Onsets and marks are recognizer estimates. The abutting caveat applies to 429 of them.
- The binding's voice guard checks receipts and download names, not the provider. Root verifies provider identity.
- Root owns selection, answers, decode, assembly, installation, TODO and Beads.
