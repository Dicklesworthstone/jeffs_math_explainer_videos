# Logarithm stage06 preflight

Author: RubyRaven, for BlueSalmon's Mail 2526 and 2596. Narration is ElevenLabs Oliver, not Michael.

## Verdict

No source or narration text blocker. The bound plan and the staged source agree on every check here, and the plan gets through the real assembler and verifier. Stage06 is ready for root's integration and native build.

This is not a build result. Nothing has been compiled, rendered or heard. No patch is staged, because no concrete defect in stage06 turned up.

**Oliver: 0 of 80 exact takes exist locally. Provider history is unknown for 78 of 80 and incomplete for the other 2.** All 80 stay MISSING until root's per-cue history check.

## What ran

| Check | Result | Output |
|---|---|---|
| Preflight on the real stage06 | 323 ok, 0 FAIL, 4 WARN | `preflight-stage.out` |
| Same, system Python without numpy | 322 ok, 0 FAIL, 4 WARN (director check skipped) | `preflight-stage-system-python.out` |
| `integrate_stage06.py --check` on the real tree | all 7 can install, no live guarded job at that moment | `integrate-check.out` |
| Mutations on temp copies | 71 of 71 behaved | `test_preflight.out` |
| Rehearsal on a temp copy | 27 of 27 behaved | `rehearsal.out` |
| Inventory scan | 19,158 JSON files read, 9 unparseable, none over 40 MB | `oliver-cue-inventory.json` |
| rustfmt `--check` on the 5 changed chapters | exit 0 for each | not saved |

Preflight covers:
- exact script and unit text, and the request files
- all 15 holds: source hash, marker line, interval hash
- registry and cue coverage: 13 chapters, every cue said once, in order, by its own chapter
- 24 phrase anchors and their source destinations
- patch replay at zero fuzz from baseline to candidate
- stale bindings

The rehearsal ran the real `coherent_narration.py` check, assemble and verify, and `narration_delivery.py --consumer render`, on all 80 cues after a real install on the copy. The takes were **stand-in near-silent samples**. That proves the plan's shape and the receipt shape, not any take.

## Findings for root

None blocks. Numbered so you can answer by number.

1. **Stale units in stage06's crate.** `$S/explainer_corpus/logarithm_mysteries/narration_units.json` has 14 units, script 15eb9ae9, bound to oliver11. Copying the crate wholesale would regress 80 units to 14. Install from `bound-plan` only. `integrate_stage06.py` does; the rehearsal shows the real CLI and the preflight both catch the wrong copy.
2. **Stray `doing_nothing.rs.orig` in stage06's chapters.** Carried from stage04 and stage05, undeclared, not compiled. Do not install it.
3. **Complex jobs bind logarithm paths.** Complex's picture inputs hold 34 logarithm paths. Integrate with no such job running, or that job's input guard trips. Any later recheck of those inputs reports the changed chapters. The newest complex receipt I read is `complex-combined-picture-frames02`, finished 17:30:02Z, exit 0. If you are about to run complex again, rebuild its inputs after integrating.
4. **8 of 15 holds name bytes canonical does not have yet:** 04_6, 05_1, 05_5, 06_2, 06_4, 10_4, 11_3, 11_20. By design, until integration. For the same reason, no voiced consumer can run on the staged path: `narration_delivery` refuses a topic outside `explainer_corpus/`. Integration comes first.
5. **The phrase fallback is silent.** If `<narration>/phrase-timings.json` is absent, or the aligner misses one anchor word, the kit loads anyway and that hook falls back to a character share. Run the gate in step 10 below. Its test includes a file the kit would accept that the gate refuses.
6. **`narration_director.validate_map` refuses map13 as written:** `KeyError('policy')`. oliver12 fails the same way, so this is not a regression. With `policy` and `range_s` added in memory, all 15 holds validate. It only matters if you run a director audit.
7. **`00_1-tts-lane01.py` reads `narration-request-text01`.** Bytes match for all 80 under their older ids, but the 9 split cues have different ids there (`05_2-part1` for 05_2, and so on). Use `narration-request-final80-01` for final ids. `requests.json` also names stage04 as its `script_path`; the hash matches, so that is only a label.
8. **The provider's 48 kHz WAV is refused by the assembler.** It needs a bound 24 kHz mono PCM16 decode first, with its receipt row. The rehearsal shows the refusal.
9. **Assemble from the repo root.** `review_directions` reads evidence paths from the cwd, and they point at `bound-plan/hold-audit.json`, so that file must stay where it is.
10. **Shared files are pinned.** Map13 pins `crates/explainer_kit/src/kit.rs`, `narration.rs`, `draw.rs` and `main.rs`, and those kit files are dirty in the worktree. If you edit them before integrating, the preflight flags it and the map needs rebinding.

## Exact path for root

Run from the repo root. Steps 1 to 3 are the only ones I wrote code for.

1. Confirm no job binds the logarithm paths (finding 3), then `python3 -I -B $D/integrate_stage06.py --check`, where `$D` is this folder.
2. `python3 -I -B $D/integrate_stage06.py --apply --out narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-integration02`. This installs exactly 7 files and keeps the 6 it replaces under `before/`. oliver12 stays in `docs/`, unbound.
3. In your guarded lane, with `CARGO_TARGET_DIR` set: `sh $D/build_stage06.sh`. It runs preflight `--mode canonical`, `check_cues`, `cargo test`, `build`, `clippy -D warnings`, then 80/80/15, facts against oracle, and the chapter list.
4. `logarithm_mysteries glyphs <dir>`, then read the pages listed under "Look at first".
5. Scratch-voice preview with the argv in `five-film-release-20261027/logarithm-combined-repair-preview01/resource-receipt.json`, then frames at the passages listed below.
6. Provider history, per cue, for exact text and voice (`required_before_synthesis`).
7. Guarded serial Oliver lane from `narration-request-final80-01`. Keep every take.
8. Decode to 24 kHz with a receipt row per cue.
9. `coherent_narration.py assemble --require-directions`, using `bound-plan/direction-dispositions-proposed.json` for `direction_dispositions` and a truthful `listening` value.
10. Write `<narration>/phrase-timings.json`, then `python3 -I -B $D/preflight.py --mode canonical --phrase-timings <file> --narration <narration>`.
11. Narrated preview. Listening and final AV review are yours.

## Dependencies

- Toolchain: `rust-toolchain.toml`, nightly-2026-08-31, with rustfmt, clippy, rust-src.
- `fmn`: git rev `a399d626c43ecc852a5c795288979f52b446bc5c`, features `ffmpeg`. The checkout is present.
- `explainer_kit` and `explainer_score`: path deps at `../../crates/`. **So the crate cannot build in place in stage06**, where that path does not exist. Integrate first, or overlay.
- The crate is its own workspace, excluded at root `Cargo.toml:16`. `Cargo.toml` and `Cargo.lock` are byte-identical stage to canonical, so `--locked --offline` needs nothing new.
- `CARGO_INCREMENTAL=0`, as in `combined-build-repair02/build.sh`.
- `encode.rs:248` sets 256 GiB raw and 32 GiB artifact caps, so long chapters clear the older 64 GiB limit.
- Last native build inputs still match: `build-inputs03.json` 183 files and `combined-preview01/inputs.json` 188 files, 0 moved. Stage06's delta from that build is 5 chapters, units and map13.

## Oliver cues

`oliver-cue-inventory.tsv` has one row per cue. Totals: 80 cues, 4,592 words, 24,250 characters, 364 sentences, longest cue 95 words.

- **Exact local takes: 0 of 80.** No normalized match in a recording, and no recording holding an exact sentence of 4 words or more. 108 real Oliver UI receipts were in the scan.
- **Provider history:** observed for 00_1 and 00_2 only, with a sentence-start query: "No matches until October 28, 2026". That is a search cutoff, not proof of absence. 78 are not checked.
- **Split cues:** 05_2, 05_20, 06_6, 06_20, 11_7, 11_20, 12_6, 12_20, 12_21. A take of an older unsplit cue cannot serve them: `unit_spans` requires the receipt text to equal the whole cue.
- **Short sentences:** 02_3 "Eight.", 07_6 "Two cautions.", 08_1 "Square it.", 11_4 "No." `unit_spans` needs a measured end per sentence. Aligners may miss these, so check them by hand.
- Settings on record: Oliver - Clean, British and Steady; Eleven v4; stability .5; similarity .75; clone strength 0; WAV 48 kHz.
- The scan sees local JSON only, and only the audio field names real receipts use. A positive control confirmed it counts an exact take and ignores a template row and a plan row.

## Look at first

Not a substitute for reading everything. Sampled pictures are not continuous AV.

- **Glyph pages:** the 7 new literals in `glyph-delta.tsv`. None adds a character canonical literals lack, but that says nothing about TeX commands. Look at `\ell` and `(\ell)` in chapter 06 and `singular S` in chapter 11. `\ell` is already in canonical and the last glyph job exited 0, but exit 0 does not prove which face drew it.
- **Tests:** 9 new or changed, with filtered commands in `new-tests.tsv`. Mine may fail; they have never compiled.
- **05_3:** last sentence budget went from 3.8 to 4.2 s.
- **06_2:** first budget went from 0.4 to 2.8 s; also the `\ell` colouring and the 0.71 s count step.
- **10_3:** add frames at the phrases (2094, 2099, 2104 at scratch timing; recompute on voice). The longer static page is 6.3 s.
- **11_8:** the two red lines need 6.20 s, which fits if the final 18-word sentence runs at or under about 2.96 words a second (median 2.56). The fallback is a letter-only string at 5.40 s if speech is under about 6.1 s. The teal line has no frame yet. Find `S` rows by exact string.
- **10_4:** its reading window depends on the take and can be zero. Measure it once takes exist.

## Not run

- `cargo` anything: compile, tests, Clippy. Text checks cannot see types or borrows. `build_stage06.sh` has only passed `sh -n`.
- `integrate_stage06.py --apply` on the real tree. It ran only on temp copies.
- `post_build_check.py` against a real binary. It ran only against a stand-in.
- The Rust phrase loader. The gate is my Python mirror of `narration.rs` lines 906 to 950.
- Glyph pages, patched pictures, any render, any Oliver take, provider history, browser, ASR, fine sync, listening, final AV.
- ASR is not listening. Sampled pictures are not continuous AV. No claim here says the film looks or sounds right.

## Authorship: this is not independent

I wrote the original 04 to 06 rebuild, the chapter 11 and 12 repairs, patches 04, 05 and 06, the bound-plan binder root adapted, and the `check_cues` repair, and I cross-reviewed the interval patch. Root wrote the interval patch, the assembly and the stage06 binder adaptation. BronzeJaguar wrote the labels patch, under shared memory lineage. The preflight has its own parser and does not import the binder, but the same author reading the same plan can share the same blind spot. Treat it as a mechanical gate, not as independent acceptance.

## My own errors, caught before handoff

- The inventory first reported 80 exact recordings: it counted the unmeasured template, which sits a text hash beside a null `wav_sha256`. The contradiction with my earlier scan exposed it. Fixed, with a control.
- `build_stage06.sh` had an apostrophe inside a `${VAR:?}` message and would have died on its first line. `sh -n` caught it.
- One baseline pin was wrong until I took it from the real file.
- Mutations found that sets hid a second call on a listed line, and that the 11_20 window used the audit's count, not the source's. Both fixed, with a new case.
