# Native no-score PCM audit858

Independent Codex BlackThrush. I freshly read and hashed original CSD313 and transferred Mac766 soundtrack captures. Both are exactly identical: WAV SHA `f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51`;52,812,844bytes;48,000Hz stereo16-bit PCM;13,203,200sample frames. Raw PCM payload SHA is `b919e1629234e9c6e83e1d8bc5b1ccbae727c4cc5e2bb4e466106a582e7e0b16` for both. Length equals16,504video frames×800audio samples/frame. Exact paths and source hashes are in JSON.

**The new no-score853 capture should match these PCM bytes and expected WAV hash.** The existing cross-platform313/766 identity is direct precedent, not merely an expectation from source. No new853 output was checked here.

In frozen explainer_kit/cli.rs651–653, external score is installed only when --score is supplied.853 and766 omit it. At666–680 CRF output enables the capture directory. In encode.rs187–203, the wrapper locates the mux input named soundtrack.wav and copies its original bytes before running the mux process. Thus the captured file is pre-AAC PCM, not decoded compressed audio. Capture failure only warns, so native render success alone is insufficient: check presence/count/format/hash explicitly.

Any changed PCM payload or sample count is a concrete deterministic audio/schedule discrepancy to investigate before advancing. Equal PCM with a changed WAV container hash would require explicit RIFF-header explanation and a new evidence binding; do not silently accept against the old pin. MP4 or AAC byte differences do not imply PCM capture differences: encoding/container verification is separate, and lossy decodedAAC cannot be null-tested against originalPCM as exact equality.

The eventual score-inclusive final must null-test against its own score-inclusive replay; f7f8… is specifically the no-score baseline. Frozen cli.rs987–1007 performs captured/replay null testing and frame-derived sample-length enforcement during mastering. Preserve those checks.

No listening, pronunciation, subjective audio quality, fine sync or final AV acceptance is claimed.
