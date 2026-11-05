# Independent review of the four stage09 take swaps (87do.63, root Mail 4614)

LavenderPike, 2026-11-05. `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`, `EL` =
`narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`, `O` = `EL/production-audio-stage09-01`.

**Subject.** BronzeJaguar's sealed `P/logarithm-stage09-edge-take-review01` (SHA256SUMS `95030197`, verified 10/10).
It recommends swapping all four cues. Root has since adopted that recommendation as `O/selection02.json`
(`871e7d0b`; Mail 4619), and decode02 and prepare02 have run from it.

**Late.** Mail 4614 asked for this review before decode02 and prepare02. My provider usage limit delayed it until
after both had run. It is therefore a check of what production now consumes.

## Decision: keep all four swaps

I found no concrete reason any proposed swap is worse or unsafe.

| Cue | Selected now (was) | Decision | Basis (checks) | Left for root's ear |
|---|---|---|---|---|
| 01_2 | `01_2-download02.wav` (`download01`) | **keep swap** | V, T, L (−0.07 dB), W (no differences), P | no pause at "is two, the" (download01: 0.27 s) |
| 07_6 | `07_6-download02.wav` (`download01`) | **keep swap** | V, T, L (+0.05 dB), W (shared "non zero"), A1, A2 (s1–s5 each in quiet, speech 0.01–0.04 s after), P | tighter comma and sentence pauses |
| 11_5 | `11_5-download01.wav` (`download02`) | **keep swap** | V, T, L (+0.04 dB), W (shared "non zero"), P | soft sound before "If" (12.10–12.18 s); b178 at a junction without a pause; take 0.64 s shorter |
| 12_2 | `12_2-download02.wav` (`download01`) | **keep swap** | V, T, L (−0.11 dB), W (shared "laps→lapse" ×2), P | none |

The "left for root's ear" column repeats Bronze's hearing items. They are cadence questions that energy cannot settle;
they are not defects and not blockers.

## What I checked, and that Bronze's review does not list

All results are from `checks/independent_checks.py`: **25 PASS, 0 FAIL**. Two runs gave byte-equal output, which is
sealed as `checks/outputs/independent_checks.txt`.

- **V: voice per file.**
  - Each of the eight files matches its download receipt and root's decision, in both the file hash and the PCM hash.
  - The provider's own download name for each file says "Oliver - Clean, British and Steady", with s50, sim75 and v4.
  - Each pair of takes comes from one generation (one provider timestamp), between −11 and +295 s from its request
    receipt.
  - Logarithm keeps Oliver, so no swap changes the voice.
- **T: request.** Each request's text hash equals the current script line's hash. The script is `559afa36`, the one
  selection02 names. Every request is Oliver on Eleven v4, with stability 0.5 and similarity 0.75.
- **L: whole-take audio.** Each alternative has the same format as the take it replaces: 48 kHz, mono, 16-bit. Its
  speech-frame RMS is within 0.11 dB of that take's (about −21.5 dBFS for all eight). It has no clipped sample and no DC
  offset. A swap therefore adds no level step against the neighbouring cues.
- **W: words, my own comparison.** I compared each take's single recognizer transcript, taken from root's opening and
  remaining reviews, with the script using my own tokeniser. No alternative has a difference that the take it
  replaces lacks.
- **A: timing answers.**
  - answers03 is answers02 plus exactly one entry, Bronze's optional `onset:07_6:s5:07_6-download02.wav` = 625920
    (24 kHz). No other answer changed.
  - I read each of the five 07_6 alternative onset answers against the alternative's own 48 kHz samples. In each, the
    0.10 s before the onset is quiet (every 10 ms frame below −55 dBFS), and speech at −40 dBFS or more starts 0.01–0.04 s
    after it.
  - The other three alternatives need no answer. Prepare02 ran with answers03, so no question blocked it.
- **P: production.**
  - selection02 differs from selection01 in exactly these four cues, and each now names the reviewed alternative.
  - decode02 and prepare02 both bind selection02, and prepare02 binds answers03 (`cab7c534`).
  - Root's `prepared02-comparison01.json` finds the other 76 cues' PCM unchanged.

I did not repeat Bronze's edge, pause, rule or point analyses, and I did not repeat root's end-window measurements.
Bronze owns decoded-edges02 (Mail 4619); I did not measure decoded02 or prepared02 audio.

## Limits

- No listening, viewing, fine sync or acceptance is claimed. The cadence of the tighter reads is for root's ear.
- The words come from the existing recognizer pass, which is not proof of speech.
- The level check compares whole takes. It does not judge how the joins sound.
- I synthesised, decoded, prepared and launched nothing, and edited no production input, Beads, TODO or memory file.
- This review is independent of Bronze's code and analysis. It reuses root's recognizer reviews and receipts as inputs.

## Files

| File | What it is |
|---|---|
| `REVIEW.md` | This note |
| `checks/independent_checks.py` | The 25 checks: read-only, standard library only |
| `checks/outputs/independent_checks.txt` | Their output at seal |

**Replay.** Run from the repository root; it takes about 1 s. OUT must not exist.

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-edge-take-independent01/checks/independent_checks.py OUT
```
