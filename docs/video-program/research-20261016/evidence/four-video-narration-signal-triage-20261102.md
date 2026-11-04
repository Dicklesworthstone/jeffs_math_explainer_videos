# Four-video narration signal triage

Author-run, 2026-11-02. This measures the complete 24 kHz mono PCM cues in
the current review assemblies for five paradoxes, abstract algebra,
exponential, and logarithm. Complex analysis is not included: its retained
draft is largely grouped audio, not a complete per-cue assembly.

The guarded `four-video-narration-signal-triage01` job exited 0, with 84
frozen inputs, unchanged input-manifest hashes and no resource stop. It
checked each PCM sample hash against the corresponding successful full-cue
reconstruction receipt before measuring. No audio was cut or modified.

| Candidate assembly | Cues | Samples exactly at digital full scale | Highest sample peak |
| --- | ---: | ---: | ---: |
| Five paradoxes | 21 | 0 | −0.52 dBFS |
| Abstract algebra | 20 | 0 | −0.43 dBFS |
| Exponential | 16 | 0 | −0.45 dBFS |
| Logarithm | 14 | 0 | −0.50 dBFS |

The largest measured RMS over the first or last 30 ms of any cue was
−35.05 dBFS. The longest run of complete 50 ms windows below −50 dBFS was
0.90 seconds. These are fixed-window signal descriptions; quiet windows may
be intentional, and energetic edges may also be natural.

This is **not listening**, speech detection, a take decision, proof against
audible clipping, an intersample true-peak measurement, or an encoded-final
loudness check. It cannot resolve the five-paradoxes 04_2 fraction or 05_2
condition disagreements. Listen to the full takes, review pacing and sync,
and measure the encoded final mix when a final master is authorized.
Independent mathematical, pedagogical, audiovisual and accessibility review
remain open.

- [Measurement script](four-video-narration-signal-triage-20261102.py)
  SHA-256 `a512c1dc196276adf5dd7e76303f6eb08d50ac29a4a863d15df6157c9e6ae439`.
- Detailed JSON: `narration_lab/third-wave-20261024/root/five-film-release-20261027/four-video-narration-signal-triage01/signal-triage.json`, SHA-256 `28d0e4a08977a79ec9498511ef1e1a0cfd15f1a579ac12e7b85fb7b6b97af022`.
- Resource receipt SHA-256 `98081b7a803a116c9fcb11995174734ae1cee624113395fad5755dcbcf97e493`.
