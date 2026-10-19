# Voice models and analysis tools — primary-source matrix

Research snapshot: 2026-10-16, SageBasin/Codex. Consumer: Jeff and BronzeRidge selecting bounded narration experiments. This is a source/metadata audit, not a listening comparison, installation qualification or legal clearance. Retire/supersede it when the selected checkpoints or licenses change. Only documentation, code and small metadata were fetched; no weights, model execution, training, services or production edits.

The linked X item concerns **Fish Audio Drama 3**, per the parent's logged-in browser observation, including its paid-partnership label. That observation identifies the lead; technical claims below come from primary sources. I did not independently inspect X or infer quality from its demo. The separate [idea slate](../VOICE_IDEA_WIZARD.md) proposes experiments; this file records compatibility and evidence limits.

## What Michael currently is

T = `<FTTS_REPO>`, inspected HEAD `7718a5f6a14f0d7f42b72933af73c9e66aec0dde`. P = `<VIDEO_REPO>`, read-only evidence. Governing instructions and READMEs were read before inspection.

| Local observation | Source and implication |
|---|---|
| FTTS implements Qwen3-TTS-12Hz-0.6B-Base, rather than the whole Qwen TTS family. | T/`README.md:5`. Other checkpoint sizes/heads are not qualified drop-in replacements. |
| `michael` is a bundled `.spk` vector, described as warm/dynamic/masculine. | T/`crates/ftts-cli/src/lib.rs:124`. The preset file is 4,096 bytes; SHA-256 `e43d6f26f8608be6d28a4d9911c8b7e37adbf3f9a33456838d586fc3faa2ef3a`. This is not a reference recording or proof of a particular English accent. |
| Presets/cards are embedding-only; a QUALITY `.ftvoice` can carry transcript and 16 codec codes/frame. | T/`crates/ftts-cli/src/synth.rs:391`, `synth.rs:399`, `synth.rs:542`. A voice card does not contain the waveform needed by another architecture. |
| QUALITY enrollment encodes the same cleaned reference PCM used for speaker enrollment. | T/`crates/ftts-cli/src/lib.rs:3143`, `lib.rs:3156`. An approved synthetic Michael WAV plus its exact transcript can create a new reference pack; improvement remains unmeasured. |
| `say` supports richer ICL; the resident vector protocol does not. `talk` explicitly supplies only the embedding. | T/`crates/ftts-cli/src/synth.rs:393`; `talk.rs:499`; T/`README.md:222`. Model warmth, persistent dialogue context and reference conditioning are different properties. |
| Teacher-forced logit comparisons already exist as conformance tests. | T/`crates/ftts-conformance/tests/pq_distributional_tf.rs:1`, `:107`, `:277`. These compare captured oracle cells; they are not an arbitrary-audio emotion-scoring CLI. Missing fixtures can skip the test (`:280`). |
| FTTS code and its weights have different licenses. | T/`LICENSE`: code is MIT **with the OpenAI/Anthropic rider**; Qwen weights and converted derivatives are Apache-2.0. Do not summarize the combined product as plain MIT. This task is owner-directed research. |

Compatibility rule: **approved audio plus verbatim transcript is the practical bridge**. Each destination computes its own speaker features, codec tokens and prompt representation. FTTS's 1,024-float vector, voice-card pixels and Qwen codec IDs are not portable to Parler/Fish/F5/Chatterbox/CosyVoice/Pocket by renaming or resizing them. Within the exact Qwen architecture, a component-level adapter might be possible after encoder/tokenizer/checkpoint and prompt-layout equivalence tests; that is narrower than `.ftvoice` format support and is not demonstrated here. A same-named “Michael” preset in another project is not evidence of the same speaker.

## Synthesis capability and license matrix

“Ready upstream” means a documented released path, not installed/qualified here. “Test” denotes a proposed sidecar experiment. License names below distinguish the named code revision from the named weights. Dependency/assets/reference licenses still travel separately.

| Exact model/path | Released controls; Michael bridge | Code license | Weight license / material limitation | Practical disposition |
|---|---|---|---|---|
| **Current FTTS / Qwen 0.6B Base** | Existing x-vector preset; QUALITY ICL uses reference audio + transcript. No demonstrated free-form emotion-instruction interface. | Local MIT + rider | Qwen Apache-2.0 ([Q06]) | First test: approved Michael synthetic reference → existing QUALITY `say`. No engine port needed. |
| **Qwen3-TTS-12Hz-1.7B-Base** | Audio/transcript cloning; x-vector-only alternative. | Apache-2.0 ([QC]) | Apache-2.0 ([Q17]) | Upstream Python sidecar for model-size headroom. Not a promise that FTTS can load its weights or that it sounds better. |
| **Qwen 1.7B CustomVoice / VoiceDesign** | CustomVoice has named trained speakers plus instructions; VoiceDesign describes a new voice. Base's clone function is a separate path. | Apache-2.0 ([QC]) | Both Apache-2.0 ([QCV], [QVD]) | Neither supplied stock Michael identity nor arbitrary reference-clone+instruction is established by the CustomVoice API. Design→audio→Base clone is documented, but redesigning Michael is not preserving him. Upstream explicitly suppresses `instruct` for 0.6B CustomVoice ([QAPI]). |
| **Parler-TTS Mini v1** | Text descriptions control delivery; 34 named training speakers help consistency. No documented arbitrary reference-audio clone argument in the published example. | Apache-2.0 ([PC]) | Apache-2.0 ([PM]) | Useful prompt-control baseline and possible later fine-tuning platform. “Mike” is not FTTS Michael. Native Michael requires separately tested adaptation, not a prompt claiming his name. |
| **IndexTTS-2** | Speaker audio separately from emotion audio; emotion vector/intensity and text-derived emotion paths ([IC]). | Current repository uses **bilibili Model Use License**, not assumed Apache ([ILC]) | Named IndexTTS-2 file has bilibili Model Use License; §3.4(c) restricts using it/derivatives to improve other AI models except Index derivatives or non-commercial models; large-user/revenue conditions also apply ([ILW]). | Research comparison only after license disposition. Do not use it as an assumed permissive FTTS distillation teacher. Precise duration control described for v2 was not enabled in that release; current v2.5 speed control is a separate path ([IC]). |
| **Original Chatterbox / Chatterbox-Turbo** | Original supports reference cloning, CFG and exaggeration; Turbo separately advertises cloning and paralinguistic tags. Original's exaggeration/pacing coupling is documented ([CC], [CT]). | MIT ([CLC]) | Both named cards MIT ([CW], [CT]) | Practical English clone challenger. Pin original for its two-knob experiment; do not transfer those control semantics to Turbo without checking its API. Preserve Perth watermark behavior. Target Michael/English accent remains a listening question. |
| **Fun-CosyVoice3-0.5B-2512** | Released zero-shot audio+transcript path and `inference_instruct2(text, instruction, reference_audio)` examples; card advertises emotion/rate control ([CM], [CE]). | Apache-2.0 ([CLC3]) | Apache-2.0 ([CM]) | **Preferred expressive-clone sidecar test**: meaningful speaker+instruction interface, modest named model size, permissive named releases. English Michael retention and actual host cost are unmeasured. |
| **Kyutai Pocket TTS** | Audio-reference cloning and reusable voice state; English path and CPU-oriented implementation ([KM], [KC]). | MIT ([KLC]) | Current card **CC-BY-4.0**, gated access, plus usage terms ([KM]); not “MIT weights.” | Strong CPU challenger, already named in T/`AGENTS.md` as a challenger. Re-enroll Michael from approved WAV; its embeddings are not FTTS vectors. No verified free-form actor-direction control. Do not equate its named Michael with ours. |
| **Microsoft VibeVoice-1.5B** | Expressive, long-form multi-speaker generation reported by its card. | MIT ([VLC]) | Metadata MIT, but card body says research-purpose use and cautions against commercial/real-world applications ([VM]). | Material inconsistency needs disposition before production. Higher-cost long-form research reserve, not the cheap first trial. Realtime-0.5B is a different model/path, not inferred equivalent cloning support. |
| **F5-TTS / F5TTS_v1_Base in SWivid/F5-TTS** | Reference audio + transcript, flow matching, released inference and fine-tuning tooling ([FC]). | MIT ([FLC]) | **CC-BY-NC-4.0** for the inspected official weights ([FM]); MIT code does not erase that. | Research candidate; not an assumed commercially usable narration replacement. A separately licensed checkpoint would need its own evidence. |
| **EmoSteer-TTS method** | Paper demonstrates activation steering on F5/E2/CosyVoice2; emotion conversion/interpolation/erasure, with appendix code ([ES]). | No separately licensed official implementation established in this pass | No standalone EmoSteer voice-model weight release established; backbone and data licenses remain applicable | Research method, not an off-the-shelf FTTS flag. Flow-model DiT directions do not transfer to Qwen's autoregressive residual-code architecture. No “training-free = free setup” claim. |
| **Fish Audio Drama 3** | Released reference cloning and expressive directions; authors report 35B total/3B active MoE ([DR], [DB]). | **Fish Audio Community License v1.0** at current code pin ([DCL]) | Exact weight `LICENSE` is **Community v1.0** ([DWL]); card metadata still incorrectly names Research License / `LICENSE.md` ([DM]). | Candidate only after license and capacity disposition. Official installer requires ≥48 GB FP8, NVIDIA only, no CPU/MPS/ROCm and no `--compile` ([DI]). This is not S2-Pro and is not a small FTTS model swap. |
| **fishaudio/drama-3-int4** | Card credits AK and calls it an **unofficial quantization**, despite placement in Fish's namespace ([D4]). | Runtime/dependencies need their own pins; Fish code remains Community v1.0 | Same Community license bytes as Drama 3 ([D4L]) | Card distinguishes 22.1 GB download / 20.8 GB engine weights from total memory; tests were on A100-80GB/H200 and **32/48 GB serving remains unverified**. No local fit claim. Treat quantization quality/capacity as unqualified. |
| **Fish S2-Pro** | Different checkpoint using the shared installer; own inference path. | Current shared Fish source is Community v1.0 ([DCL]) | S2 weights remain **Fish Audio Research License**, commercial use separately licensed ([S2L]) | Installer says ≥24 GB GPU or CPU support for this path ([DI]); neither applies to Drama 3. Codec supplied for Drama 3 has specific coverage in Community license definitions. |

Drama 3, int4 and current Fish source license bytes share SHA-256 `733a32442087acbda12ecafe8e71eb8ee5890c89b50ce7ac4ea3777faa3c823e`. Community §III's commercial exception requires licensee-plus-affiliate annual revenue **below USD 500,000**; hosted-service rules/exclusions are separate. **§IV(b) bars materials, outputs or results used to create/train/distill/improve other foundational generative or speech/audio/voice-generation models, except Fish models/derivatives; its detection/watermark/spoofing exception is narrow.** No Drama3→FTTS teacher/critique-training recommendation follows. Plain audit inference differs from model improvement; the broad improvement clause needs disposition before downstream optimization. FTTS→Fish reference input is separate and retains upstream provenance. The owner's applicable category is not established. [Exact weight license][DWL], [current code license][DCL].

## Analysis tools: what they can establish

| Tool/checkpoint | Primary-source capability and license | Useful role; required limitation |
|---|---|---|
| **Existing `tools/prosody_qa.py`** | P/`tools/prosody_qa.py:14` defines PASS/FAIL/FLAG/ADVISORY; `:37`, `:126`, `:158` use two Praat trackers and 150ms voiced windows; `:318` separates thought/quiz/window gaps; `:458` produces video-level verdicts. Source hash `8e74710bc8d771072ffe20e2abac2d9bdec221c8ab385f4de21352dd5595932d`. | **Reuse first.** Human reference counts/bands are source-reported historical measurements, not reproduced here. Reset share with fewer than 20 valid joins stays advisory (`:479`); do not hard-gate short pilots using that aggregate. Tracker disagreement is explicit (`:204`). |
| **Existing thought-based spacing / joint chooser** | P/`crates/explainer_kit/src/narration.rs:52`, `:125`, `:147`, `:887` implement thought gap classification and spacing. Newer narration skill `scripts/narrate.py:1045`, `:1054`, `:1153` already uses joint reset/register costs; `:815` includes multiple features, not only ASR. | Parent reports closed mx-hjti/mx-uuao; source inspection independently confirms these mechanisms exist, not that every historical render enabled them. Extend calibration/receipts rather than rebuild. |
| **CommonAccent XLSR English** | MIT model card and MIT author repository; 16 broad accent labels including `england`, `us`, `scotland`, `wales`; 16kHz mono path and custom SpeechBrain interface ([AC], [ACL]). | Advisory England-vs-alternatives drift signal. Not a Received Pronunciation detector or Michael identity detector. Card warns about other datasets and interface incompatibility. Pin/review custom Python before later execution. Do not assume scores are calibrated posteriors. |
| **SpeechBrain ECAPA VoxCeleb** | Apache-2.0 weights; SpeechBrain Apache-2.0 code. Speaker verification/embedding, 16kHz mono, cosine comparison ([EC], [SB]). | Independent reference similarity and within-take change signal. Speaker benchmark thresholds do not qualify synthetic Michael or accent. Use different references/texts and human hard negatives. |
| **WavLM Base Plus SV** | Card points to UniSpeech license: current linked file is **CC-BY-SA-3.0**, despite no HF license tag; card supplies speaker-verification use ([WL], [WLL]). Library code is a separate dependency. | Optional second representation for disagreement, not automatic plain-MIT clearance. More setup than ECAPA; no reason to require both before the first listening pilot. |
| **emotion2vec_plus_large** | Released utterance/frame SER features and nine classes; weight card says custom `model-license`, linking FunASR generally ([EM]). Official FunASR distinguishes MIT code from model terms; current MODEL_LICENSE is v1.1 ([EML]). | Candidate advisory affect signal. No checkpoint-contained license text resolved here; commercial use is **not cleared**. Its pseudo-labeled training lineage is not proof of neutral mathematical narration accuracy. |
| **audEERING Wav2vec2 MSP dimensional model** | Arousal/dominance/valence and pooled states; model license CC-BY-NC-SA-4.0, body research-purpose only ([AD]). | Research comparator only under applicable terms. Not a production dependency or an English-accent gate. |
| **Qwen2-Audio-7B-Instruct** | Apache-2.0 weight card with audio-analysis examples ([QA]); proposed inference implementation is Transformers, whose code is Apache-2.0 ([QAC]). No license is inferred for other scripts merely from that. | Concrete **audio-language analysis sidecar** alternative to backwards TTS. Ask timestamped, evidence-based acoustic questions with transcript withheld/mismatched controls. Its answers are fallible judgments, not calibrated emotion/identity labels. |

P/`x:727` swallows the prosody command's error and `x:730` only promotes literal `FAIL`; `x:731` warns on absent verdict text. That observed wrapper gap is already covered by A3/A4's missing-report handling. Preserve advisory semantics while requiring the report to exist when the workflow requires it. Do not turn every pitch flag into a blocking failure.

Cadence analysis should reuse these measurements and add only demonstrated missing features: phrase-local articulation rate with uncertainty, pause location versus semantic beat, emphasis timing, and beginning/middle/end changes on matched text. Full-line/EDL sample spans are authoritative where available. Forced alignment is a proposal, not ground truth; the existing QA deliberately detects silence acoustically because ASR boundaries can smear (P/`tools/prosody_qa.py:42`). Analysis-only resampling never changes the production master.

SER domain risks are hypotheses to test: actor emotion versus explanatory engagement, lexical sentiment versus sound, speaker/accent/channel shortcuts, class imbalance, and generator fingerprints. A “happy” prediction need not mean clear teaching, nor “neutral” mean flat narration. Audit with crossed text×delivery and speaker×delivery panels, held-out source takes and texts, and human labels for perceived delivery. Never interpret SER as the speaker's inner emotional state. The [CosyVoice3 paper][CP] itself reports much worse emotion accuracy when text and intended emotion conflict; its results do not qualify this channel.

## Primary-source pins and locators

Live HF API GETs supplied the full commits below; small license/code reads supplied license names. No automatic model loader was invoked. Moving-branch links used for discovery should be replaced by these pins at implementation time; pin all subordinate codecs/vocoders/tokenizers then, not merely the headline model. This is a research locator list, **not the parent's canonical registry**.

| Key | Weight revision |
|---|---|
| Q06 | `5d83992436eae1d760afd27aff78a71d676296fc` |
| Q17 | `fd4b254389122332181a7c3db7f27e918eec64e3` |
| QCV | `0c0e3051f131929182e2c023b9537f8b1c68adfe` |
| QVD | `5ecdb67327fd37bb2e042aab12ff7391903235d3` |
| PM | `0392b9451a601e528fd863bbb0598431fee810d9` |
| IndexTTS-2 | `740dcaff396282ffb241903d150ac011cd4b1ede` |
| CW / CT | `5bb1f6ee58e50c3b8d408bc82a6d3740c2db6e18` / `749d1c1a46eb10492095d68fbcf55691ccf137cd` |
| CM | `29e01c4e8d000f4bcd70751be16fa94bf3d85a18` |
| KM | `7f364c05d1be3faf2b234cdbe4cbb68a026d4218` |
| VM | `c00898d257e6b46004e3e2866a47534085fb685a` |
| FM | `84e5a410d9cead4de2f847e7c9369a6440bdfaca` |
| DM / D4 | `e0a9218aa48888270f06ef1ed9ea67622c0e17d2` / `a6f588691a13b6607cb09f9800556487ab7aa0fe` |
| S2-Pro | `1de9996b6be38b745688de084d87a5633f714e4e` |
| AC / EC | `cc5dc6a56db647149d9e52856d6e55114c1045a8` / `0f99f2d0ebe89ac095bcc5903c4dd8f72b367286` |
| WL / EM | `feb593a6c23c1cc3d9510425c29b0a14d2b07b1e` / `6c303ba987b86b93193de93e34bb2b077a6bedc4` |
| AD / QA | `6eba34a2485ea31cb03600241787c3a5edab8626` / `0a095220c30b7b31434169c3086508ef3ea5bf0a` |

[Q06]: https://huggingface.co/Qwen/Qwen3-TTS-12Hz-0.6B-Base/blob/5d83992436eae1d760afd27aff78a71d676296fc/README.md
[Q17]: https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-Base/blob/fd4b254389122332181a7c3db7f27e918eec64e3/README.md
[QCV]: https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-CustomVoice/blob/0c0e3051f131929182e2c023b9537f8b1c68adfe/README.md
[QVD]: https://huggingface.co/Qwen/Qwen3-TTS-12Hz-1.7B-VoiceDesign/blob/5ecdb67327fd37bb2e042aab12ff7391903235d3/README.md
[QC]: https://github.com/QwenLM/Qwen3-TTS/blob/022e286b98fbec7e1e916cb940cdf532cd9f488e/LICENSE
[QAPI]: https://github.com/QwenLM/Qwen3-TTS/blob/022e286b98fbec7e1e916cb940cdf532cd9f488e/qwen_tts/inference/qwen3_tts_model.py#L470
[PC]: https://github.com/huggingface/parler-tts/blob/d108732cd57788ec86bc857d99a6cabd66663d68/LICENSE
[PM]: https://huggingface.co/parler-tts/parler-tts-mini-v1/blob/0392b9451a601e528fd863bbb0598431fee810d9/README.md
[IC]: https://github.com/index-tts/index-tts/blob/d9e41aac89fd00b3d71497fddb287b7f24613712/README.md
[ILC]: https://github.com/index-tts/index-tts/blob/d9e41aac89fd00b3d71497fddb287b7f24613712/LICENSE
[ILW]: https://huggingface.co/IndexTeam/IndexTTS-2/blob/740dcaff396282ffb241903d150ac011cd4b1ede/LICENSE.txt
[CC]: https://github.com/resemble-ai/chatterbox/blob/5de7a54aa4e5e2baadb0182dde554908b48b85c2/README.md
[CLC]: https://github.com/resemble-ai/chatterbox/blob/5de7a54aa4e5e2baadb0182dde554908b48b85c2/LICENSE
[CW]: https://huggingface.co/ResembleAI/chatterbox/blob/5bb1f6ee58e50c3b8d408bc82a6d3740c2db6e18/README.md
[CT]: https://huggingface.co/ResembleAI/chatterbox-turbo/blob/749d1c1a46eb10492095d68fbcf55691ccf137cd/README.md
[CM]: https://huggingface.co/FunAudioLLM/Fun-CosyVoice3-0.5B-2512/blob/29e01c4e8d000f4bcd70751be16fa94bf3d85a18/README.md
[CE]: https://github.com/QwenAudio/CosyVoice/blob/074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc/example.py#L71
[CLC3]: https://github.com/QwenAudio/CosyVoice/blob/074ca6dc9e80a2f424f1f74b48bdd7d3fea531cc/LICENSE
[CP]: https://funaudiollm.github.io/cosyvoice3/pdf/CosyVoice3_0.pdf
[KM]: https://huggingface.co/kyutai/pocket-tts/blob/7f364c05d1be3faf2b234cdbe4cbb68a026d4218/README.md
[KC]: https://github.com/kyutai-labs/pocket-tts/blob/fbea645333cd5c64e72688e0a96e54e69bbb9602/README.md
[KLC]: https://github.com/kyutai-labs/pocket-tts/blob/fbea645333cd5c64e72688e0a96e54e69bbb9602/LICENSE
[VM]: https://huggingface.co/microsoft/VibeVoice-1.5B/blob/c00898d257e6b46004e3e2866a47534085fb685a/README.md
[VLC]: https://github.com/microsoft/VibeVoice/blob/16fb2cb1217c9934a886e1948ffb06120caa2df5/LICENSE
[FC]: https://github.com/SWivid/F5-TTS/blob/2f866649b779881af8bd644e76b9a6d95053da29/README.md
[FLC]: https://github.com/SWivid/F5-TTS/blob/2f866649b779881af8bd644e76b9a6d95053da29/LICENSE
[FM]: https://huggingface.co/SWivid/F5-TTS/blob/84e5a410d9cead4de2f847e7c9369a6440bdfaca/README.md
[ES]: https://arxiv.org/html/2508.03543v1
[DM]: https://huggingface.co/fishaudio/drama-3/blob/e0a9218aa48888270f06ef1ed9ea67622c0e17d2/README.md
[DWL]: https://huggingface.co/fishaudio/drama-3/blob/e0a9218aa48888270f06ef1ed9ea67622c0e17d2/LICENSE
[DCL]: https://github.com/fishaudio/fish-speech/blob/541566932abbaebcb0a40e562a1f846738383f38/LICENSE
[DB]: https://beta.fish.audio/blog/fish-audio-drama-3/
[DR]: https://arxiv.org/abs/2610.18010
[DI]: https://speech.fish.audio/install/
[D4]: https://huggingface.co/fishaudio/drama-3-int4/blob/a6f588691a13b6607cb09f9800556487ab7aa0fe/README.md
[D4L]: https://huggingface.co/fishaudio/drama-3-int4/blob/a6f588691a13b6607cb09f9800556487ab7aa0fe/LICENSE
[S2L]: https://huggingface.co/fishaudio/s2-pro/blob/1de9996b6be38b745688de084d87a5633f714e4e/LICENSE.md
[AC]: https://huggingface.co/Jzuluaga/accent-id-commonaccent_xlsr-en-english/blob/cc5dc6a56db647149d9e52856d6e55114c1045a8/README.md
[ACL]: https://github.com/JuanPZuluaga/accent-recog-slt2022/blob/a2ff1154713270635d85bf476e5ae53b4ef54c24/LICENSE
[EC]: https://huggingface.co/speechbrain/spkrec-ecapa-voxceleb/blob/0f99f2d0ebe89ac095bcc5903c4dd8f72b367286/README.md
[SB]: https://github.com/speechbrain/speechbrain/blob/c0241278f93ef02cd5191f03caf4e5f8e1292bf6/LICENSE
[WL]: https://huggingface.co/microsoft/wavlm-base-plus-sv/blob/feb593a6c23c1cc3d9510425c29b0a14d2b07b1e/README.md
[WLL]: https://github.com/microsoft/UniSpeech/blob/6112826ac13a4327f4c9a7afa2a505e35b763514/LICENSE
[EM]: https://huggingface.co/emotion2vec/emotion2vec_plus_large/blob/6c303ba987b86b93193de93e34bb2b077a6bedc4/README.md
[EML]: https://github.com/modelscope/FunASR/blob/dddd5dc5dfdf759fc58c2a9c0c5a52ab60556922/MODEL_LICENSE
[AD]: https://huggingface.co/audeering/wav2vec2-large-robust-12-ft-emotion-msp-dim/blob/6eba34a2485ea31cb03600241787c3a5edab8626/README.md
[QA]: https://huggingface.co/Qwen/Qwen2-Audio-7B-Instruct/blob/0a095220c30b7b31434169c3086508ef3ea5bf0a/README.md
[QAC]: https://github.com/huggingface/transformers/blob/551122ae8e709cc36a4f2b14900fac8964ef22df/LICENSE
