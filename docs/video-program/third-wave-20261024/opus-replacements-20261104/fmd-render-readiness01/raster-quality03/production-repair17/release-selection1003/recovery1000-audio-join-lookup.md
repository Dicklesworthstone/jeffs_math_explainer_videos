# recovery1000 addendum: audio-join evidence lookup for master993

RubyRaven (CC2), 2026-11-10. Read-only lookup. Nothing was launched, decoded, rebuilt or synthesized, and no file was
edited. This is not listening evidence.

Master: complex-analysis-master993.mp4, sha256 80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90.

## Result

No audio-join check exists for master993. No existing check can be reused for it unchanged.

- **The older tools target a different master and never ran.** trj-final-joins83.py and
  complex-join-recognition88.py are bound to production19 master01 on TRJ.
  - TRJ has no trj-production19-join-extract83 job directory and no joins.json.
  - join-recognition88-controller.log is 0 bytes.
- **No job exists on either host.** No CSD or TRJ job name mentions join, seam, audio, PCM, click or recognition.
- **No record ties audio joins to this master.** No file in Q or release-selection1003 names 80244e88 together with
  audio joins.
- **The PCM audits cover a different question.** native-linux-pcm-audit858 and
  root-boundary772-preview-pcm-identity cover the 09 soundtrack before encoding, not the master's joins.

## Existing audio evidence for master993 (none of it is listening)

- **master-native.log.**
  - All 15 chapter soundtracks match their replays bit for bit, each exactly frames x 800 samples long.
  - One gain for the whole film: -1.39 dB, -18.98 LUFS, -1.50 dBTP.
  - Audio length equals picture length, 2227.700 s.
- **995 strict decode.** Full decode with 0 errors and an A/V delta of 0.000 s.
- **Caption schedule.** The last caption before each join ends at least 3.2 s early. The next chapter's first caption
  starts 0.600 s after the join.

## Missing evidence

- **M1.** Nothing compares the encoded master's audio, on either side of each join, with the chapter soundtracks. The
  audio is lossy AAC, so a comparison would need a codec tolerance, not exact equality.
- **M2.** None of the 14 joins has a measurement of level or of a sample jump at the boundary.
- **M3.** No word-aligned transcript of the master's audio exists at the joins.

Join boundary samples (48 kHz), with seconds:

| Join | Sample | Seconds |
|---|---|---|
| 01/02 | 5048800 | 105.183 |
| 02/03 | 9346400 | 194.717 |
| 03/04 | 15145600 | 315.533 |
| 04/05 | 20564800 | 428.433 |
| 05/06 | 28468000 | 593.083 |
| 06/07 | 36492000 | 760.250 |
| 07/08 | 41684800 | 868.433 |
| 08/09 | 53652000 | 1117.750 |
| 09/10 | 66855200 | 1392.817 |
| 10/11 | 72808800 | 1516.850 |
| 11/12 | 82354400 | 1715.717 |
| 12/13 | 85508800 | 1781.433 |
| 13/14 | 94021600 | 1958.783 |
| 14/15 | 103426400 | 2154.717 |
