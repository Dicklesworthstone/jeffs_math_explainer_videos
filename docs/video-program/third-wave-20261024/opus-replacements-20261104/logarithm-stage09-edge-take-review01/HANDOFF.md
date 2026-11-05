# Stage09 edge-take review: 01_2, 07_6, 11_5, 12_2 (87do.63; root Mails 4534 and 4595)

**BronzeJaguar, 2026-11-05.** Path abbreviations:
- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- EL = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`

Nothing here was heard. Transcripts and levels are provisional evidence, not listening. I selected, answered, decoded and
changed nothing. Root adjudicates and runs guarded production.

## Recommendation: swap all four

| Cue | Keep or swap | Retained file to select | Onset answers needed | Unresolved content or edge issue | Hearing items (energy only) |
|---|---|---|---|---|---|
| 01_2 | **swap** | `EL/ui-takes01/01_2-download02.wav` | none (both sentence starts are supported recognizer onsets) | none | The comma in "itself is two, the square root" has no pause of 0.10 s or more; download01 pauses 0.27 s there. |
| 07_6 | **swap** | `EL/ui-takes01/07_6-download02.wav` | answers02 (`P/logarithm-stage09-timing-corrections01/inputs/answers02.json`, 075c83bd) already holds this take's s1 44160, s2 240720, s3 376560 and s4 492240. s6 is the recognizer onset, at leave. One optional answer: s5 `onset:07_6:s5:07_6-download02.wav` sample **625920** (leave 26.08 s). The recognizer's 26.12 s is 0.03 s after speech level; download01's s5 was moved by 0.04 s for the same reason. | none | "the angle pi, so" has no comma pause (download01: 0.26 s). The alternative adds 0.11 s after "w only". The sentence pauses after "side only." and "at most pi." shrink, 0.51 to 0.40 s and 0.59 to 0.41 s. |
| 11_5 | **swap** | `EL/ui-takes01/11_5-download01.wav` | none | none | A soft sound before "If" (10 ms frames up to −51.6 dB, 12.10–12.18 s). The s2 boundary at 12.2 s lies after it, in quiet (−66 dB). Mark b178 ("the two routes") at 14.9 s sits at a −66.8 dB junction; the comma pause at "diagonal," is under 0.05 s (download02: 0.11 s). The pause after "then A." shrinks from 0.63 to 0.49 s. The take is 0.64 s shorter. |
| 12_2 | **swap** | `EL/ui-takes01/12_2-download02.wav` | none | none | No pause differs by 0.10 s or more. The shortest boundary pause is 0.49 s (download01: 0.58 s). |

Each swap removes the measured end defect, adds no word difference, and leaves no point unresolved in the installed
helper. Each alternative is a slightly tighter read than the take it replaces, with fewer comma pauses. Only listening
can weigh that, so the cadence items above are for root's ear. They are not blockers.

## Evidence

All of it is in `outputs/` and reproduces byte for byte.

- **Edge.**
  - **Selected takes:** in every one, the speech decays to the floor (−82 to −89 dB). After 0.07–0.15 s of quiet, the
    last 10 ms frame alone rises to between −45 and −58 dB. In 1 ms windows the last 3–6 ms climb to between −37 and
    −52 dB. This is LavenderPike's `rise_in_last_10ms` flag on the production files. The mirrored bus ends are −38.1
    (07_6), −39.5 (11_5), −45.5 (01_2) and −53.9 dBFS (12_2).
  - **Alternatives:** no rise. The last 10 ms are −85 to −88 dB; the −60 dB tail is 0.10–0.16 s; the first 1 ms are −78 to
    −86 dB. Their estimated bus end is at most **−75.3 dBFS**.
  - **The estimate:** the fmn last-frame gain (2.807) times the largest decoded/source tail ratio in root's 80 decodes
    (1.0) times the source's largest last-256 sample. The same estimate covers the true mirrored value on all four
    selected takes. It is an estimate: these files have no decode yet.
- **Words.**
  - **Request:** each pair comes from one request whose text equals the script.
  - **Recognizer:** each take has one recognizer pass (opening or remaining run, large-v3-turbo), and root's transcript
    review of that same log agrees with it word for word.
  - **Differences:** the differences are shared by both takes (07_6 and 11_5 "nonzero" → "non zero"; 12_2 "laps" →
    "lapse" twice) or absent (01_2). No take has an unshared difference.
  - **Last word:** the last script word is recognized in all eight takes.
  - **Name collision:** the complex-film logs naming `01_2-download02.wav` and `07_6-download0x.wav` are other films' files
    with the same names.
- **Timing.**
  - **Check selection:** `outputs/check-selection-swap4.json` is an input for the helper's own load_selection, not a
    selection. The helper accepts it.
  - **Answers:** check_answers raises no blocking question under answers01 or answers02. resolved_points resolves every
    sentence end and mark.
  - **Control:** LavenderPike's place() rule, replayed on the 48 kHz sources, reproduces her 24 kHz rows for the
    selected takes, 16 of 16 points.
  - **New join flags:** only the 07_6 s5 `late` (the optional answer above), and the 11_5 s2 `blip` and b178 `no_pause`
    (hearing items above).
- **Rule.**
  - **Replay:** production-choices01's R1–R5, replayed on its sealed evidence, gives choices.tsv's choice and rule for
    all four cues.
  - **Edge at −60 dB:** with R2's edge test moved to −60 dBFS sound, R2 picks the alternative in every cue (0 flags
    against 1).
  - **Other cues, not audited:** the same test also flags the selected start of 04_3 and 09_4 (bus start steps −63.5 and
    −84.3 dBFS). I propose nothing for them.

## Route for root (if adopted)

1. **Selection:** write selection02 as selection01 with the four paths above, with your own by and basis.
2. **Decode and prepare:** decode, then prepare with answers02, plus the optional 07_6 s5 answer if you want it.
3. **Edge check:** after assembly, rerun the bus-edge check on the new narration. All four should show no final rise.

## Run and files

`python3 -I -B P/logarithm-stage09-edge-take-review01/edge_take_review.py FRESH_OUT --fmn-sound SOUND_RS`

- **SOUND_RS:** fmn-output `sound.rs` at a399d626, sha256 7bb983c0….
- **Read-only:** the script pins 4 seals file by file and 16 inputs, and writes only FRESH_OUT.
- **Determinism:** two runs into fresh folders were byte-equal; `outputs/` is a byte copy of the first.

Output files:
- `takes.tsv` and `review.json`: one record per take, with the verdicts.
- `points.tsv`: every resolved point under both answer sets, with its pause placement.
- `pauses.tsv`: every internal quiet run of at least 0.10 s.
- `rules.tsv`: the rule replays.
- `edge-calibration.tsv`: the 80-decode tail ratios.
- `stdout.txt`: the run log.
