# Combined impact audit 261 (cc1): recipe256/262 against the existing final chapters

This audit asks what the combined visual repair does to each of the 15 existing complex final chapters, and what evidence is still needed before the four candidates for retention (02, 03, 13 and 15) can keep their existing final outputs.

**It grants no reuse and claims no pixel or audio identity.** Root decides whether to retain any final.

The work was static: I read the source and existing records. Nothing was compiled, run, rendered, synthesized or heard. No source, Beads or TODO file was changed. This file is the only one I wrote. Paths are relative to this folder.

## Bound inputs

| Input | sha256 |
|---|---|
| `review-repair-recipe256.json` (superseded) | `21e24bdf802f00bb31229efc7273713dc26edac0b33536a17eba25dc1526dbb3` |
| `review-repair-recipe262.json` (current) | `24be73c5b00e8cfd0d131462aefbf5ab739b451e491afa4efb4be1ef0d980c76` |
| `cc2-plate-fade-candidate246-draw.rs` | `aab61199e7a0dea9c5482ee2aa127f94c0c5bb06e2edbb5202350db964a1ea56` |
| `cc2-plate-fade-candidate260-draw.rs` | `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064` |
| `review-repair-build262.py`, `launch-review-repair262.py` | `120ac607…c5`, `dc84ab3d…ae` |
| `review-repair-timeline263.py`, `review-repair-after-build263.py` | `63be48df…f1`, `d4b22146…17` |
| Production19 source: `production-source-preserved214/receipt.json` | `6a4f5bdca860367e93f212390c86243639da7025281a3c803b8214e6ac7796fc` |
| Production19 binary (from `csd-production-repair-build19`) | `f4836eeeb4f2465c0d27ca0497f123f0af6bc0db7bc65096098bb20c5abb0f22` |

**What recipe262 contains.**
- Recipe262 is recipe256 with one change: `draw.rs` now points to the 260 candidate.
- The other seven entries are unchanged: boundary52, loop-sums238, one-derivative238, mod256, rigidity247, gallery249 and the 246 probe.
- The build262, timeline263 and rebind263 controllers differ from their 256/258 versions only in job and binary names, the recipe name, and the expected test count (232 → 233).

**Draw 260 changes tests only.**
- The production code of 260 and 246, meaning everything before `\n#[cfg(test)]\nmod tests {`, is byte-identical.
- The only item after the tests module is the `#[cfg(test)]` declaration of `plate_fade_probe`.
- So the runtime under recipe262 is 246's runtime. Every runtime statement below applies to both.

## Which chapters change

Only two things in recipe262 can change a chapter's output: a replaced scene file, and the change to the shared `fade_in_all` / `fade_out_all` in `draw.rs`.

The `mod.rs` change (256 against production19 `4f4b17c3…`) does not reach rendering:
- It edits only the `TEXT` glyph supplement list: three `one_derivative` entries, `gallery::UNLESS_NOTE`, and the caption "Selected square, magnified".
- `TEXT` reaches the binary only as `Explainer::glyph_text`. The kit reads that only in the `glyphs` probe subcommand (`explainer_kit/src/glyphs.rs`), not in a chapter render.

**Where image records come from.** In production19 `draw.rs`, only `Picture::build` with `style.raster` creates an image record. It calls `ImageMobject::from_resource`, and `PLATE = PLATE_RASTER`.
- Only two functions call `Picture::build`: `portrait_styled` (through `portrait`) and `Painter::build_styled` (through `Painter::build`).
- No other `draw.rs` helper (cards, predict, walker, marks and so on) calls them.
- `Painter::show` and `Painter::leave` go through `fade_in_all` and `fade_out_all`.
- The kit (`explainer_kit`, `explainer_score`) has no `ImageMobject` or `ImageResource` anywhere in its local sources.

| Chapter | Scene source under recipe262 | Raster entry point (in code outside `#[cfg(test)]`) | Output can change? |
|---|---|---|---|
| 01 the_bump | unchanged | `portrait(` in `previews` | **yes**: plate fade |
| 02 turning_numbers | unchanged `24bffa50…` | none | see E1–E6 |
| 03 two_planes | unchanged `0f22d63f…` | none | see E1–E6 |
| 04 reading_color | unchanged | `Painter` | **yes**: plate fade |
| 05 gallery | **gallery249** | `Painter` | **yes**: source and plate fade |
| 06 one_derivative | **one-derivative238** | none | **yes**: source (repaired final needed anyway) |
| 07 one_point | unchanged | `Painter` | **yes**: plate fade |
| 08 loop_sums | **loop-sums238** | none | **yes**: source (repaired final needed anyway) |
| 09 boundary | **boundary52** | `Painter` | **yes**: source and plate fade |
| 10 rigidity | **rigidity247** | `Painter` | **yes**: source and plate fade |
| 11 counting | unchanged | `Painter`, `portrait(`, `PLATE` | **yes**: plate fade |
| 12 residues | unchanged | `Painter` | **yes**: plate fade |
| 13 real_integral | unchanged `83eadb53…` | none | see E1–E6 |
| 14 continuation | unchanged | `Painter` | **yes**: plate fade |
| 15 closing | unchanged `0c217b6b…` | none | see E1–E6 |

- **Changed outputs.** Eleven chapters change: 01, 04, 05, 06, 07, 08, 09, 10, 11, 12 and 14. The nine raster chapters are 01, 04, 05, 07, 09, 10, 11, 12 and 14.
- **Whether every plate gets a ramp** is CC2 and CC3's question (246/257/260). It is not checked here.
- **Direct engine fades.** In code outside `#[cfg(test)]`, `fade_in(`, `fade_out(` and `fade_all(` appear in only three registered files:
  - `draw` (once each, inside the two helpers);
  - `one_derivative` 238 (one each);
  - `loop_sums` 238 (`fade_out(` twice).

  That matches CC3's finding behind 260. Chapters 02, 03, 13 and 15 have none, so 260's vector-only exceptions should never need to name them. If they do, this audit is wrong.

## 02, 03, 13, 15: source facts (static)

**No raster families, including indirect calls.** None of the four names `Picture`, `Painter`, `portrait`, `portrait_styled`, `PLATE*` or any `Image*` type, or calls a kit image API (there is none).

Their imports from outside `draw` are:

| Chapter | Items imported | Status |
|---|---|---|
| 02 | `model` only | `model.rs` unchanged, `118172af…` |
| 03 | `gallery::TITLES_TEX` | gallery.rs is replaced by 249, but `TITLES_TEX` is byte-equal in 249: seven TeX titles, from `f(z) = z^3 - 1` to the ζ series. |
| 03 | `the_bump::RADIUS` | `the_bump.rs` unchanged, `8e93012e…` |
| 13 | `the_bump::{POLE_I, POLE_MINUS_I}` | constants; unchanged |
| 15 | `the_bump::{AT_X, BUMP_TEX, POLE_I, POLE_MINUS_I, graph, recipe_points}` | `graph` (10 lines) and `recipe_points` (17 lines) contain no portrait or plate code. The_bump's only `portrait(` is in its own `previews`. |

**Scene source unchanged.**
- The four chapter files, `the_bump.rs`, `model.rs`, `main.rs` (`19a9e105…`), `score.rs` (`0563256d…`) and `thumbnail.rs` (`5d90834b…`) all equal the production19 receipt. That receipt matches the `production-repair19/workspace` files listed in `hz2-runtime-manifest47.json`, the workspace binary `f4836eee` was built from.
- `review-repair-build262.py` copies that same workspace. It asserts that, after `cargo fmt`, the changed files are exactly the eight recipe targets and the only new file is `plate_fade_probe.rs`. If the build passes, these files are therefore the production19 bytes.

**Existing finals.** All four were rendered by `f4836eee`, the binary recipe262 starts from:

| Final | Folder | MP4 sha256 | Frames |
|---|---|---|---|
| 02 | `final02-terminal167` | `bf0891a409f589bd4fc125c038e03dd027ec334436e25cdc129463c659984f07` | 5,372 |
| 03 | `final03-terminal115` | `20353d01ee564ef101ecad80bdfa8ec1db87ac2188b2e4642969b371f2e82cf3` | 7,249 |
| 13 | `final13-terminal200` | `49934034d4611b81155925e667dab463072b21ba0240467447b9ddfb52d32f51` | 10,641 |
| 15 | `final15-terminal81` | `3e2440a8785319cf9477e73303e6391c8a4165c7fbfd3fa6f4ad97e81f874fd4` | 4,379 |

**Their tables against timeline21.** I compared each final's tables with the local `timeline21-terminal01` copy, the base that timeline263 asserts against:
- **timeline:** equal after mapping the narration directory, for all four;
- **cues:** byte-equal for 02, 03 and 13;
- **cuelog:** every narration row equal after mapping. The finals carry one or two extra rows, which are the music stems from `csd-production-score21` (see E4).
- **15:** the local final folder holds no cues or cuelog copy, so for those only the hashes in its render summary are available.

## What the added helper reads at runtime

The runtime change for 02, 03, 13 and 15 is in `fade_in_all` and `fade_out_all` (pinned engine `fmn` `a399d626`, read with `git show`):

1. **For each faded mob `m`,** after the engine fade is built and `stage.prepare` boxes it, the helper calls `image_records(stage, m)`. The facade `Stage` dereferences to the mobject arena, so this makes three calls:
   - `MobjectStage::family(m)`, a depth-first family walk;
   - `get(k)` for each member;
   - `Entry::image_resource()`, which is `self.image.as_ref()`.
2. **Where no member has an image, nothing is added.**
   - No `PlateFade` is built, and there is no extra `prepare` and no extra animation in `play_prepared`.
   - The animation list and run times are those of production19, and no new error path is reachable.
   - In these four chapters no family can hold an image: entries get images only through `ImageMobject` or `set_image_resource`, and neither is reachable from them.
3. **`family()` is not a pure read.** On a cache miss it writes the entry's `family_cache` (a `RefCell`), and every later call returns that cached list.
   - The engine fade itself calls `family()` only at `begin`, inside `play`: `fade_in` and `fade_out` only build a `Transform`, and `prepare_animation` only boxes it.
   - So the helper can fill a mob's family cache **earlier** than production19 did, before the play rather than at `begin`.
   - The cached content is the same depth-first list, unless an engine path changes a family's structure without calling `invalidate_family_caches`. At the pin, attach, detach, delete and entry replacement do call it, and `fmn-mobject/tests/scenarios.rs::family_cache_invalidation` exists.
   - This is a **concrete, low-likelihood risk, not a demonstrated difference**. The interval between the new call and `begin` holds no structural edit to `m` in the helper loop. It does hold the `begin` of earlier mobs in the same play.

## Evidence still needed before retaining 02, 03, 13 and 15 (nothing here grants it)

**E1. The build's source binding.**
- Build262's `source-copy.json` must show:
  - the four chapter files, `the_bump.rs`, `model.rs`, `main.rs`, `score.rs`, `facts.rs`, `plate_layout.rs` and `thumbnail.rs` at the production19 hashes;
  - the eight targets at their recipe262 hashes.
- `cargo fmt` runs without `--check`, and the script does not re-assert the candidate hashes after it. The changed-set assertion protects the unchanged files, but it does not prove that a reformatted candidate equals its reviewed bytes.
- `draw.rs` in the build must be `923ad2ef…`.

**E2. The dependency closure.**
- The crate's path dependencies are outside the copied workspace, so build262's own hashing does not cover them: `explainer_kit` and `explainer_score` under `work/tree/repo-view`, the `fmn` git checkout at `a399d626`, and `engine-budget.toml`.
- The launcher adds every file in `timeline21-inputs.json`. That file exists only on the remote host, so I could not check whether it lists those sources.
- Needed: proof that the kit, score and engine sources build262 compiles equal build19's. Either `review-repair-build262-inputs.json` lists them at build19's hashes, or a comparison of the dependency closure in the build's Cargo dep-info gives the same result.
- Root has already bound kit `lib.rs` (`6fd69ffe…`), `props.rs`, `plot.rs` and `animatic.rs` (`rigidity-kit-binding254.json`). `glyphs.rs`, `cli.rs`, `kit.rs` and all of `explainer_score` still need binding to the bytes actually built.

**E3. The schedules.**
- `csd-review-repair-timeline263/comparison.json` must show that the timeline, cues and cuelog of 02, 03, 13 and 15, produced by binary 262, equal timeline21 after the named narration-path mapping. The controller asserts this for all 45 tables, with a total of 133,662 frames.
- The finals' own tables equal the local timeline21 copy, as above.
- If any of the 45 tables fails, E4 fails for every chapter, not only that one.

**E4. Audio: the music is one score for the whole film.**
- The finals' cuelogs mix their narration with stems from `csd-production-score21`. Bound hashes:

  | Chapter | Score21 stem | sha256 |
  |---|---|---|
  | 02 | `02_turning_numbers.music.f32.wav` | `36c1915acb3e5174f851c2f53cde5006903edc5676597eed6d0f410352835346` |
  | 03 | `03_two_planes.music.f32.wav` | `6ae59a9b782eb67dd586d87ae4f103a8276614bb0c09b154cb807287b8b49c26` |
  | 13 | `13_real_integral.music.f32.wav.part0000.wav` | `def06f7b659b209c000e89ceb17e3cf35acaa561c5e74d2231693fc13d1c594d` |
  | 13 | `13_real_integral.music.f32.wav.part0001.wav` | `1cad887b2c84916d42101f3683c260f1b52557f69a5ced03b57b97af13136488` |
  | 15 | `15_closing.music.f32.wav` (from `final15-export56.json`) | `ec9d9a4f0292b56aad2698066aa0afdefbe0536943f8190de0318f29962d720e` |
  | all | `score.fingerprint` | `09dcfea7d570df142abb3bf0e54eb69149d2ab9d7d29abc712e148c169bf6c80` |

- **Why every chapter's music depends on all fifteen.** The kit's `score` command renders the whole video's music from **all fifteen** chapter timelines and slices it per chapter (`explainer_score/src/score.rs`; kit `cli.rs::score_video`). In that render:
  - tails and duck releases run continuously across cuts;
  - the progression offset depends on the chapter's index;
  - the true-peak cap is one gain, set by the film's loudest instant;
  - the bed trim is a median over the whole video;
  - the voice register is measured from all narration lines.
- So the music of a retained chapter depends on every other chapter's timeline and narration.
- **Needed:**
  - the new delivery for the combined film uses stems for 02, 03, 13 and 15 that are byte-equal to those above;
  - so does every repaired chapter, so that all cuts come from one score render;
  - all narration WAVs are byte-identical to the cuelog rows (rebind263; CC0's review259 covers the rebind).
- A matching fingerprint is enough only if it hashes the timelines' content and not narration paths. Otherwise compare the stems directly.
- If the score is re-run with any change, the retained finals' audio is no longer this film's score, and those finals need at least a new mux.

**E5. Pictures: a measured comparison, not a static argument.**
- The static facts point to identical frames, but they do not prove it:
  - the four chapters build no plates, no ramp is added, and their schedules are expected to be equal;
  - Rust release builds keep IEEE semantics.
- Against that, the binary differs (new code and `-C metadata=jmev_review_repair262`), and point 3 above is a real ordering change.
- Proof needs a full native render of each retained chapter with binary 262 at the final configuration, compared with the existing final by **decoded-frame hashes** (for example ffmpeg `framemd5` of the video streams), or by the renderer's raw-frame digests if available.
  - An MP4 byte comparison is not the test, because encoder build, settings and threads matter.
  - The finals were rendered on different workers: TRJ for 02 and 03, CSS for 13, HZ2 for 15. A comparison on another host that disagrees is inconclusive until host-specific output is ruled out, so pin the host or reproduce the original host first.
- Without such a comparison, retention is a decision root makes on E1–E4. It is not established equivalence.

**E6. What retention leaves open.**
- A retained final keeps its prior review state and nothing more:
  - 02: root's picture review 167;
  - 03: CC0's review 212, with disposition 223 (F1–F3 accepted);
  - 13: CC2's review 200, with disposition 206;
  - 15: root's 92-picture review 82.
- In all four, continuous listening, fine sync, motion and independent final AV acceptance remain open.
- Eleven chapters change, so the master joins, captions and chapter offsets must be rebuilt and checked in any case. E4 covers the music seams at 01|02, 03|04, 12|13 and 14|15.

## Risks, ranked

1. **E4 audio coupling (highest).** The retained chapters' music depends on all fifteen timelines and all narration. A changed table anywhere, or a re-scored film, invalidates the retained audio.
2. **E2 unhashed path dependencies.** The kit, score and engine sources the build compiles are not hashed by build262's own script, and I could not see whether the launcher's timeline21 input list covers them.
3. **E1 `cargo fmt` rewrites in place.** The candidates' post-format bytes are not re-asserted against the recipe.
4. **Ordering change in the family cache** (runtime point 3). It is low likelihood and tested in the engine, but it is a real ordering change and must not be assumed away.
5. **Host dependence of any pixel comparison** (E5).

## Limits

- **Sources read.** Static reading of:
  - the preserved production19 source and the local recipe candidates;
  - the local `crates/explainer_kit` and `crates/explainer_score`, which are not proven to be the bytes built on the remote host;
  - the engine at `a399d626`, via `git show` from a local clone.
- **Records read.** Existing records only: render summaries, cuelogs, the local timeline21 copy, exports and dispositions.
- **Not run.** No compile, test, Clippy, render, extraction, synthesis or listening.
- **Not read.** I did not read `cc3-plate-fade-review257.md` or `cc0-rebind-review259.md`, and read only the file table of `cc2-plate-fade-correction260.md`.
- **Pixels and audio.** This audit establishes no pixel identity and no audio identity. It accepts no motion, fine sync, listening or final AV.
- **Narration.** The narration stays ElevenLabs Frederick, unchanged.
