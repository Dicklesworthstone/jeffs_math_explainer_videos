# Stage09 decoded02 edges: the four swapped recordings (87do.63; root Mail 4619)

**BronzeJaguar, 2026-11-05.** Path abbreviations:
- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- O = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/production-audio-stage09-01`

This was read-only on every production file. I ran no decode, prepare, assembly, render or job. Nothing was heard:
levels and correlations are measurements.

## Result: the four decoded recordings pass

| Cue | Decoded file (from) | Samples (= predicted) | Last 10 ms | Bus end step (was) | Bus start step | Continuity |
|---|---|---|---|---|---|---|
| 01_2 | `decoded/01_2.wav` (`01_2-download02.wav`) | 476160 | −85.5 dB, no flag | **−81.8 dBFS** (−45.5) | −90.3 dBFS | lag 0 in 38/38 windows, r ≥ 0.9989 |
| 07_6 | `decoded/07_6.wav` (`07_6-download02.wav`) | 929280 | −87.8 dB, no flag | **−88.5 dBFS** (−38.1) | −78.3 dBFS | lag 0 in 75/75, r ≥ 0.9971 |
| 11_5 | `decoded/11_5.wav` (`11_5-download01.wav`) | 785280 | −87.0 dB, no flag | **−87.0 dBFS** (−39.5) | −74.7 dBFS | lag 0 in 63/63, r ≥ 0.9980 |
| 12_2 | `decoded/12_2.wav` (`12_2-download02.wav`) | 364800 | −85.0 dB, no flag | **−89.1 dBFS** (−53.9) | −90.3 dBFS | lag 0 in 29/29, r ≥ 0.9984 |

- **Edges.**
  - **End flags:** none of LavenderPike's three flags fires on any of the four decoded files. decoded01's four files
    each carried `sound_in_last_10ms` and `rise_in_last_10ms`.
  - **Last samples:** the last decoded samples are 0 to 2 LSB.
  - **Bus end:** with fmn's resampler mirrored, the voice bus's last sample is 1.2–2.7 LSB. Every value is inside
    edge-take-review01's predicted bound of −75.3 dBFS.
  - **Bus start:** the starts stay small. The largest is 11_5 at −74.7 dBFS: its first bus sample is 6 LSB, and the
    source opens 7, 5, 1.
- **Length:** each decode is exactly ceil(n/2) of its 48 kHz source, which is edge-take-review01's predicted
  length. The durations change by −0.08, −0.16, −0.64 and −0.08 s.
- **Sample continuity.**
  - **Alignment:** I compared every 0.5 s speech window against every second source sample, at lags −2 to +2. Lag 0
    is best in every window, so no sample was dropped or repeated and the alignment never shifts. Correlation at lag 0
    is 0.997 or more.
  - **Levels:** speech-frame 10 ms levels match the source within 0.51–1.13 dB. The largest difference in each take
    falls on a frame with a high zero-crossing rate (0.16–0.40), a fricative whose content above 12 kHz a 24 kHz file
    cannot hold. Quiet frames match (median +0.0 dB).
  - **Zeros and peaks:** no decode has a run of zeros longer than its source's, and the peaks are equal within 0.1 dB.
  - **prepared02:** root's prepared02 narration WAV for each cue holds exactly the decoded PCM.

## What binds it

The guard receipt was checked first.
- **Guard:** `five-film-release-20261027/logarithm-stage09-audio-decode02` shows exit 0, admitted and not preflight only.
  Its inputs hash 6a4d4caa equals the final hash and `O/decode-inputs02.json` (439 files). That list's rows for the
  four sources, selection02, the helper and the binding are still at their current hashes. The command is the installed
  helper's decode of selection02 into decoded02, and its log reports 80 decoded recordings.
- **Selection:** selection02 (871e7d0b) differs from selection01 in exactly 01_2, 07_6, 11_5 and 12_2. Each change is to
  edge-take-review01's recommended recording, and the helper's own load_selection accepts it.
- **Receipt:** decoded02's receipt (f9f49524) shows the same ffmpeg (8.1.2) and arguments as decoded01. Its 76 other rows
  equal decoded01's apart from the WAV path; I compared receipt rows only and did not re-read those 76 files.
- **Root's measurements:** I recomputed root's own values from the files they name: selection02-decision's last 10 ms
  and last samples, and prepared02-actual-endings01's frames, RMS, last sample and peak. All 16 agree.

## Run

`python3 -I -B P/logarithm-stage09-decoded-edges02/decoded_edges.py FRESH_OUT --fmn-sound SOUND_RS`

- **SOUND_RS:** fmn-output `sound.rs` at a399d626, sha256 7bb983c0….
- **Pins:** 3 seals file by file (edge-take-review01, timing-corrections01, production-join-inspection01) and 11 inputs.
- **Determinism:** two runs into fresh folders were byte-equal; `outputs/` is a byte copy of the first.

Output files:
- `edges.tsv`
- `continuity.tsv`
- `windows.tsv` (every scored window)
- `decoded-edges.json` (everything, with the per-cue verdict)
- `stdout.txt`
