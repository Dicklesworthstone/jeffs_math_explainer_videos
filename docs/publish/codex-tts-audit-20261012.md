# Narration audit — 12 October 2026

All four existing private-review masters were transcribed end to end with **fw 0.9.2**, using model-backed whisper.cpp native inference and **large-v3-turbo**, without a script prompt. Four full passes and 26 targeted/short passes completed successfully. Total full-audio coverage: **63 minutes 6 seconds**. No dropped inference windows were reported.

This is an ASR comparison, not a claim of physical listening or pronunciation approval. Except for the owner-reported RaptorQ opening, repeated recognition differences remain candidates requiring listening or acoustic examination. Number/formula spellings, homophones and punctuation were separated from substantive differences. The source scripts and delivered captions match after case/punctuation normalization.

## Owner decisions and release hold

- Michael is selected for **Fish and Hoeffding**, in addition to the existing Michael RaptorQ and Bakery cuts.
- Hoeffding must sound closer to **Herfding / HERF-ding**. Preserve the correct written spelling in captions and metadata.
- The Robert cut’s nine name occurrences were recognized as Herfting/Herfding. Its lexicon tests are Robert-specific and do not certify Michael. Probe Michael in the actual sentences and possessives.
- Preserve every original master and take. New voice/fix renders need separate files and fresh final-audio checks.
- Everything remains **Private review only**. Nothing is approved for public posting.
- Michael Fish and Michael Hoeffding masters were absent from the mounted variant final directories at this audit’s final check. They have not been audited or uploaded by this pass.

## Priority findings in the selected Michael cuts

| Cut | Sentence starts | Expected | ASR, full and isolated | Disposition |
|---|---|---|---|---|
| RaptorQ Michael | 00:00.600 | phones | phone | Owner heard this; all three ASR passes agree. Correct the take. |
| RaptorQ Michael | 03:24.467 | fountain codes | fountain code | Repeatable final plural loss candidate. |
| RaptorQ Michael | 17:44.700 | ten to sixteen slots | 10 to 16 slot | Repeatable final plural loss candidate. |
| Bakery Michael | 14:11.557 | atomic reads and writes | atomic reads and write | Repeatable final plural loss candidate. |

## Additional checks

The following are lower-certainty candidates, or checks to carry into the new Michael versions.

| Cut | Sentence starts | Expected | Recheck transcription | Note |
|---|---|---|---|---|
| RaptorQ Michael | 18:48.050 | parked pieces | park pieces | Adjacent consonants can confuse ASR; inspect the actual take. |
| RaptorQ Michael | 19:18.800 | With ex-or, A met itself and cancelled | With XOR met itself and cancelled | Check that the letter A is clearly AY. |
| Bakery Michael | 05:26.800 | Green’s slate | Green Slate | Possessive /s/ can merge with the following /s/; lower certainty. |
| Bakery Michael | 09:05.840 | smaller than the other’s | smaller than the other | Possible final possessive loss. |
| Fish Matt | 01:54.800 | Marked times second catch, over caught twice | Mark Time’s second catch over caught twice | Full pass misheard times/twice; isolated pass retains twice but weakens marked. Recheck the Michael replacement. |
| Fish Matt | 11:53.920 | five in a one followed by four hundred thirty zeros | five and a one followed by 430 zeros | The mathematical relationship should be clearly audible in Michael. |
| Fish Matt | 16:17.603 | pull the guess down | pull the guest down | Possible added consonant, or ASR confusion. |
| Hoeffding Robert | 01:31.097 | found the dependence | found the dependent | Repeatable ASR substitution; recheck Michael. |
| Hoeffding Robert | 06:34.857 | in rank coordinates | in rank coordinate | Repeatable final plural loss candidate; recheck Michael. |
| Hoeffding Robert | 08:16.267 | shrugs off outliers | shrugs off outlier | Repeatable final plural loss candidate; recheck Michael. |

## Cleared on isolated recheck

RaptorQ: “Lose some” and “which pieces it mixed.” Bakery: “lost point,” “later read,” “Anatol Holt” (Anatole is a phonetic spelling), “the other way round,” the apparent added “a” before memory, and the spoken ending. Fish: “let them go there,” “one in four,” and “caught twice.” Hoeffding: the apparent repeated “half.” These illustrate why a raw full-transcript diff is not itself a defect list. Bakery’s “The End” after narration is an ASR tail hallucination, not established added speech.

## Reproduction and scope

Audio was extracted from the exact MP4s into 16 kHz mono PCM WAV, without tempo, pitch or loudness changes. Each run used `fw transcribe --input AUDIO.wav --language en --no-diarize --no-persist --json`; full runs had a 1,200-second timeout. Candidate sentences were re-extracted with surrounding padding and transcribed independently without giving ASR the expected text. Normalized segment confidence fields were not used as calibrated probabilities.

| Current cut | Master MD5 | Audio duration |
|---|---|---|
| RaptorQ Michael | `16867253aba700f7f8602ef753cb49bb` | 1261.312 seconds |
| Bakery Michael | `19f2e859c0ee1a0841c0115ac69ab656` | 941.803 seconds |
| Fish Matt | `528918af361165d6f73862d5fc5b9107` | 1057.152 seconds |
| Hoeffding Robert | `016f42a28ff497065dd9eb2a18a6f0a1` | 526.123 seconds |

## Production handoff

The owner decisions and findings were sent to the existing narration/orchestration agents, IndigoBeaver and GreenCarp, through Agent Mail thread `owner-tts-review-20261012`. Messages were persisted; acknowledgment had not arrived when the report was prepared. The Screen Sharing input attempt could not bring the explainer Ghostty session forward, so direct UI acknowledgment remains pending. No production source, narration selections, masters, channel visibility or uploads were changed during this audit.

Consumer: the narration owner and the channel owner. Decision: which takes need correction and which new voice renders may proceed to private review. Defect classes: dropped endings, omitted letters, and unclear mathematical wording/pronunciation. This audit is superseded only after corrected Michael finals have been transcribed and the owner has reviewed them. Do not hide the discrepancies with permissive phone/phones or similar ASR equivalence rules.
