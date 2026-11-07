# Unchanged chapter early-final reuse review 17

## Decision

**Proceed with a separate, explicitly provisional production16 final-render lane for 01, 02, 04, 06, 07, 10, 12 and 15 once the launch-time identities below are frozen. Do not wait for 08/09 implementation to render these candidates. Promotion into the production17 master remains conditional on exact per-chapter equivalence after17 exists.** This can save elapsed time without pretending future equivalence is already proven. No job was launched by this review.

**Remove chapter14 from the proposed unchanged set.** Q/continuing.rs changes DENSE[1] to “all these roots together are dense; drawn: through order 64”, SHA-256 `9cb4fc0bb87e48d9ce9793d09c0d31b4bfbfbfab7057074db204cf8d6b164f96`. Exact source equivalence fails for14. Chapters03/05/08/09/11/13 likewise wait17. Rendering14 with16 can only produce a knowingly superseded wording candidate.

## Before launching the eight candidate chapters

1. Pin actual TRJ production16 bytes to `8dc5e8c4f640b73d9e4e6c4575cb492ebcc0deb0c2de0a658bbccf40d46082d1`; preserve its terminal native qualification and source/dependency receipt. Freeze every runtime narration WAV/report, exact per-cue text and sentence/phrase timing, score stem and parts table, engine/kit assets and launch helper in the resource manifest. Do not use the changing production17 working tree as runtime input.
2. Require the applicable existing selected-frame/motion review for each candidate chapter, with no unresolved defect for that chapter; do not invent a global all15-clean permit. Existing final-review-gate16.py and final-chapter16.py are all15/CSD/global-permit-bound. Create a separately reviewed partial-TRJ helper/permit naming exactly these eight. Keep the earlier gates unchanged.
3. Preserve original resource admission/monitor/stop policy and fresh paths. Pin 3840x2160, 60 fps, x264 CRF16, veryfast, yuv420p, the same color tagging and negotiated encoder configuration intended for17. Include host encoder identity. The diagnostic used CRF0/1fps/silent; those settings are not final settings. Verify descendant stop coverage for this actual worker path. Limit concurrency according to measured host resource headroom.
4. On completion retain terminal exit0, invariant input hashes, exact frame count and all emitted timeline/cue/cuelog/layout tables; keep the native 48kHz stereo PCM16 capture with frames*800 samples, stem identity, and every movie hash. Perform decode and final selected-picture inspection. A completed render is not listening acceptance.

## Sufficient later promotion check for each individual chapter

- Compare the complete source/dependency closure between16 and17: chapter implementation; shared draw/model/palette/facts; narrator/kit/render/score integration; Cargo.lock, engine git/build overrides, assets, build features and relevant globals. `build17.py` is designed to copy16 and reject changes outside candidate-receipt paths, but its terminal source-copy.json must establish the actual change set. Inspect cross-chapter imports: an unchanged file can consume changed exported constants. Confirm changed sibling modules have no runtime effects on selected chapters. Different executable hashes are expected and must remain explicit in provenance.
- Compare the chapter's exact script cue set/order/text, decoded WAV SHA-256 and PCM samples, phrase/sentence boundaries, inserted gaps and direction dispositions. Compare per-cue report payloads, not whole-report equality alone: unrelated08/09 edits legitimately change global hashes.
- Generate17's native **60fps** timeline/cues/cuelog and compare byte-for-byte after only an enumerated old-root/new-root mapping for absolute filenames. Validate each rewritten file's digest before mapping. Preserve all hashes, timestamps, integer frame/sample positions, binary float offset representations and row order. Never normalize away numeric differences or arbitrary path differences. Compare duration and frame count exactly.
- Require identical score stem bytes, split-part samples/order/offsets and QA applicability for that chapter. A score regenerated globally may change unmodified chapters through seeds or duration context. Match frames*800 coverage with no gaps/overlaps.
- Confirm17's master/replay accepts each retained16 PCM capture **bit for bit** against17's expected soundtrack. Verify equivalent video stream parameters/extradata/timebase and actual stream-copy joins. Recompute all later chapter offsets, captions, total frames and expected duration using17 timelines; the old fixed total117697 is invalid after lengthening08/09.
- Record per-chapter `render_binary=16` and `master_binary=17`, both hashes, equivalence evidence and review scope. If any condition fails, preserve the candidate and rerender only that chapter with17; never relabel16 media as rendered by17.

## Concrete helper pitfalls and evidence limits

- final-master16.py hardcodes all15, host=csd, production16 binary/permit identity and total117697 frames. It cannot serve as the mixed17 master unchanged. A new explicit mixed-provenance importer must validate chapter contracts and compute actual totals.
- final-chapter16.verify_tables compares exact narration rows and appends exact score rows. Keep that strength in the new helper. Path normalization must be a named mapping only, not stripping path columns.
- prepare-final-inputs16.py freezes the entire existing preview/picture input tree; copying it blindly forces unrelated failed/repaired chapters into a partial render gate.
- TRJ parity result proves **111/111 decoded YUV frames equal for one silent chapter04 4K/1fps diagnostic**, elapsed242.938418s. This is useful host-transfer evidence, not narrated mixing,60fps motion, all-scene equivalence or final acceptance. Its files have different MP4 hashes despite identical decoded frames, so container-byte identity across hosts is not a required criterion.
- The rough diagnostic throughput is about0.457fps; extrapolation to long60fps chapters is only a scheduling estimate because narration durations, frame-to-frame caching, encoding and animation workload differ. Start useful frozen candidates rather than claiming a measured full-film ETA.

This decision is conditional scheduling advice from local source/receipt inspection. No remote launch or live TRJ filesystem verification was performed here. Full listening, fine sync and final audiovisual/release acceptance remain open.

## Inspected helper hashes

- `trj-picture-parity16cores01.py`: `90ed1af57b5d2069756159f1081f31efc5f941d7a4c7c18d832f61d9e81acbbd`
- `trj-picture-parity16cores01-result.json`: `68e2d17141e16a9fac7d7f6919af97172ae3b1a7e250c45e102e84f6ab5aea92`
- `build17.py`: `b2f37ee35184ca348505d6200ddee11157fbf066f4905f2d453ca7e9ea4270e6`
- `final-chapter16.py`: `63fa91806fa427d0f4f5e344f19169a325c9b48b3eafb5e4521b092f84050e79`
- `final-master16.py`: `53f384e68239f8eaf61ea03a6f5f81162c83bb5064a8851ec2fd37a800c5df60`
- `prepare-final-inputs16.py`: `ef61d758061fd03eefd036f2624fffa6d26bf8eb83396f921f6974c0e5faaca9`
- `preview16.py`: `dc185dea97521971fcb844820c52fd80603d10b749736c8a82ac6c62d9225a3e`
- `final-review-gate16.py`: `6d14516f5323ffa3aa0d8ec0d1c454e1ae276c59ee598d5c0a8b3b4bebd08ea6`
