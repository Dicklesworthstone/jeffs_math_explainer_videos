# Preview-to-final binding 282 (cc1): preview243 chapters 06 and 08 to the combined 268 final

## Summary

This file binds what preview243 showed for 06 one_derivative and 08 loop_sums to what the combined final (final278: binary 268, narration 269) will render. It names what reviews 270 and 271 carry forward and what still needs checking on the final.

**Behaviour, by source: no change found for 06 or 08.**
- Both chapter files are byte-identical in build238, which made preview243, and build268, the final's binary. So is every file they import, except `draw.rs`.
- `draw.rs` gains three insertions and loses nothing. They add a plate fade inside `fade_in_all` and `fade_out_all` only when a faded mob's family holds an image record. Nothing 06 or 08 builds holds one. For these chapters the insertions do a read-only family walk per faded mob and add no animation, no `prepare` and no time.
- The other differences between the two builds cannot be reached from a 06/08 render: `gallery.rs`, `rigidity.rs`, one glyph-list line in `mod.rs`, and the test-only `plate_fade_probe.rs`.
- Both builds compiled only the film crate. They linked the same dependency rlib files: none of the 216 was modified after 2026-11-08 00:59:07Z, and build238 started at 04:45:38Z.

**Bytes: not claimed and not needed.** The binaries differ (`f31690e6…` and `983954c5…`), and so will the frames' bytes. Following disposition 283, nothing here asks for a rebuild to prove deterministic compilation.

**Narration.** All 105 WAVs, the 14 of chapters 06 and 08 among them, are byte-identical in assembly19, rebind242 and rebind269. So are the 105 sentence rows and the assembly outputs and units. The 242 → 269 differences are direction-contract metadata only.

**Schedule.** One piece is still open: timeline269. When I read it, it had written only chapters 01–03 and had not finished. Preview243's 06/08 tables equal timeline242's byte for byte, and those equal timeline21's after the narration-path mapping. The final gate asserts the same equality for timeline269.

**What carries forward.** Reviews 270 and 271 carry forward for content, layout in scene units and timing at frame numbers. They do not cover:
- anything that depends on 540p pixels or the preview encode;
- motion, which neither reviewed;
- audio, since preview243 had no score;
- final AV.

The final needs those checked fresh (section 6).

**How this was done.**
- Read only: local files in this folder, and read-only SSH to csd using Python and `sha256sum`.
- Nothing was written on csd; no build, render, extraction or synthesis.
- No Beads, TODO or source edits. This file is the only file I wrote. Timeline269 was not touched.
- Paths below are relative to `R = /data/projects/jmev-stroke-probe-20261106` on csd unless they name a file in this folder.

## Bound records (csd)

| Record | sha256 | State |
|---|---|---|
| `R/jobs/csd-review-repair-build238/resource-receipt.json` | `0dddf8e168870e12374df9ea5c652a944052daacdcd0d1d25318bbc6ae5dc8de` | 04:45:38–04:46:29Z, exit 0, inputs = final (`bc68e60d…`) |
| `…build238/source-copy.json` | `6d4c7e93e89a29930cf5b4c2f3101fba023a3e09b99888f3e9db083bdb1eb65c` | 69 files; changed: boundary, loop_sums, mod, one_derivative |
| `…build238/qualification.json` | `fe167a8135ac855e82f3af87d538db9e8e7eb24406e9db79cc63108b20035e9b` | binary `f31690e648393923c373922b513a77e65dc44fef10dbd39f8b88b561e283e9da`, 226 tests, facts `7d528509…` |
| `R/qualification01/production-repair17/review-repair-build238-inputs.json` | `bc68e60d79f95eafc8f1ade1903e163c72e8ef385dd1296068614f81ee2a49fa` | 12,313 files |
| `R/jobs/csd-review-repair-rebind242/resource-receipt.json` | `49baed5691eef69ee86b5b6ec588d4a6bf8457a1a99cc93d7ba1e616a5f953aa` | exit 0, inputs = final (`04728a68…`) |
| `…rebind242/rebind-summary.json` | `6ea26fed7276424f5c13f959377cf3c8962ab37e75e097a02b44808895ce8ba6` | 105 WAVs byte-identical; 105 cues verified |
| `R/jobs/csd-review-repair-timeline242/resource-receipt.json` | `a19cba5d077700c8a0cba1ac9c9597bf46611f48266f532a98f256fd078d2535` | exit 0, inputs = final (`8557a40f…`) |
| `…timeline242/comparison.json` | `b3bdeb8b31adb5bc9d43550cc15c904c7ef38231568fd1929f9ec9535d606e4d` | 06/08 tables equal timeline21 after mapping |
| `R/jobs/csd-review-repair-preview243/resource-receipt.json` | `f6fc94fc79c7e44eae4723a68c33417e5037974c5e1460f77cecf61aa09c06e2` | 04:57:50–05:26:50Z, exit 0, inputs = final (`6510db78…`) |
| `R/qualification01/production-repair17/review-repair-preview243-inputs.json` | `6510db7856deec2abfea54d585000ec09d0cf5334ff32b558adb71172a8dcf91` | 12,607 files |
| `R/jobs/csd-review-repair-build268/resource-receipt.json` | `3504e1ab843b7e951e37ef06904b6eddd247cf9dd7ea061828552c4affc19ad2` | exit 0, inputs = final (`53f1360e…`) |
| `…build268/source-copy.json` | `81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c` | 70 files; changed: the 8 recipe268 targets |
| `…build268/qualification.json` | `2f3430a82dc1561eecbc1485aa00c18070606c4d8728fc6811b62b1f141dc857` | binary `983954c508e42b90d443ad6ca09f2baba7565a4f609d09d2ea821640534d6ec7`, 233 tests + probe, facts `7d528509…` |
| `R/jobs/csd-review-repair-rebind269/resource-receipt.json` | `32fee5d44ab9d18067593bccaeb6f0ae50dc784f084a94f37e03ef804f351ebd` | exit 0, inputs = final (`f1eac452…`) |
| `…rebind269/rebind-summary.json` | `fa263d8a6c14c21ef037525fd316e2d64019c0f506e934a4e2d38886b5167da6` | 105 WAVs byte-identical; 105 cues verified |
| `R/jobs/csd-review-repair-timeline269/resource-receipt.json` | not final | started 05:36:46Z; not finished when read; inputs `2c4e86e5abbcf5ca86622328e16bb3b528e96c0d398fcfb5f71b64af7343f5d2` |

**The two films reviewed.** Both files on csd match the local copies that reviews 270 and 271 read:
- `preview/06_one_derivative.mp4` is `5e45d72da91701c9c7c346b98ded8491c13ba1befed6be1b6301c1c6d33ed62d`. Its `sampling.json` is `34ee8c6a…` and its `select.filter` is `87e6a88c4bba37cf4485ebe9d407dbef346d111a781170085386641ca13d604a`.
- `preview/08_loop_sums.mp4` is `1ac1ee716f6aa383a9154d6305ffa571bd555df818565d28a07084eb4b5364d6`. Its `sampling.json` is `951eab4a…` and its `select.filter` is `445d7121…`.

**The reviews.** `cc0-preview06-review270.md` is `d4ad3533de6a6034285973a2a921a235d319fc569073d36c1b41e999c4d6b531`. `cc2-preview08-review271.md` is `d70fb0deec2bfc562c0815e57678019769cc84544c940bbfe192f5ac5b242478`.

## 1. Source: build238 against build268

**From the two `source-copy.json` records.** The live workspaces still equal them, apart from the rebind rewrites in section 3.

| File | build238 | build268 | Reached by a 06/08 render |
|---|---|---|---|
| `chapters/one_derivative.rs` | `7a4cc01a…` | `7a4cc01a…` | yes: chapter 06 |
| `chapters/loop_sums.rs` | `8a13e8a1…` | `8a13e8a1…` | yes: chapter 08 |
| `chapters/one_point.rs` (`RATIO`) | `76a8bf1d…` | `76a8bf1d…` | yes: a `&str` constant |
| `chapters/two_planes.rs` (`FN_TEX`) | `0f22d63f…` | `0f22d63f…` | yes: a constant |
| `chapters/residues.rs` (`TOTALS`) | `7d06f69b…` | `7d06f69b…` | yes: a constant (08) |
| `chapters/turning.rs` (`RULE`) | `24bffa50…` | `24bffa50…` | yes: a constant (08) |
| `src/model.rs` | `118172af…` | `118172af…` | yes |
| `src/main.rs` | `19a9e105…` | `19a9e105…` | yes |
| `script.tsv` (compiled in, and compared at run time) | `234f7257…` | `234f7257…` | yes |
| `Cargo.lock` | `6a080058…` | `6a080058…` | yes |
| `tools/narration_delivery.py`, `tools/coherent_narration.py` (the delivery gate) | `961f17cc…`, `a7fc6705…` | same | yes: a gate only |
| `chapters/draw.rs` | `cd394e5a…` (production19) | `9bb07b3f…` (candidate264) | yes: section 2 |
| `chapters/mod.rs` | `b5817c50…` (236) | `e1f4d1b8…` (256) | registry: one added line, in the glyph list |
| `chapters/gallery.rs` | `7d1cf20f…` | `d57e003a…` | no |
| `chapters/rigidity.rs` | `c577e2c0…` | `0bcfbc7d…` | no |
| `chapters/plate_fade_probe.rs` | absent | `c0c56084…` | no: declared `#[cfg(test)]` in `draw.rs` |

**Why `gallery.rs`, `rigidity.rs` and the `mod.rs` line are unreachable.**
- **Imports.** 06 imports only from `draw`, `one_point`, `two_planes` and `model`. 08 imports only from `draw`, `residues`, `turning`, `two_planes` and `model`. Neither imports `gallery` or `rigidity`.
- **`draw.rs`.** Its only uses of `gallery` (`trig_plane`, `gamma_plane`, `exp_plane`) are inside its test module, below the runtime prefix (line 1398 in production19).
- **`mod.rs`.** Mod 236 and mod 256 differ by one line, `gallery::UNLESS_NOTE,`, in the `TEXT` glyph list. As 261 and 273 show on the compiled kit, `TEXT` is read only by the `glyphs` probe subcommand. `PREDICTS` (line 31) and `READING_WINDOWS` (line 58), which 06_8 and 08_5 use, sit at the same lines in both versions with unchanged contents.

**Chapter registry.** The registry lists are identical in both qualification records, the same 15 keys in the same order.

**The shared build inputs are the same.** The build238 and build268 input manifests have identical rows for:
- `work/tree/engine` (4,057);
- `work/tree/repo-view` (68: kit, score crate, rustfmt and tools);
- `engine-budget.toml` and `work/.cargo/config.toml`;
- the copied `production-repair19/workspace` (69).

The two controllers use the same asserted cargo and rustc binaries, the same environment and the same flags. They differ only in metadata (`jmev_review_repair238` against `jmev_review_repair268`), test count and the probe step.

**Both builds compiled only `complex_analysis_visual`,** in their tests, Clippy and build logs. None of the 216 dependency rlibs in the shared target was modified after 00:59:07Z, before build238 began, so both binaries link the same dependency files (timestamp evidence; not re-hashed). So the machine-code doubt that 273 raised as U1, between production19 and 268, does not arise between 238 and 268 for the engine, kit and score. There the difference is confined to the film crate.

## 2. `draw.rs`: what the change does in 06 and 08

**The diff.** A line diff of the runtime part (everything before `\n#[cfg(test)]\nmod tests {`) between csd's live `review-repair238` and `review-repair268` copies gives three insertions and no replacements or deletions:
- 32 lines after production19 line 268: `image_records` and `plate_fade`;
- 4 lines in `fade_in_all`;
- 4 lines in `fade_out_all`.

Each pair of 4 lines is:

```rust
let images = image_records(stage, m);
if !images.is_empty() {
    anims.push(stage.prepare(plate_fade(m, images, true).rt(rt))?);   // false in fade_out_all
}
```

**When the change takes effect.** `image_records` returns the members of `m`'s family that hold an image resource. In 06 and 08 none do:
- **Engine.** At the pinned engine (`a399d626`), outside test modules, an entry gets an image only from `ImageMobject` → `Mobject` (`fmn-library/src/image.rs:323`) or from loading a persisted scene (`fmn-mobject/src/persist.rs:1463/1490`). The setters at `fmn-render/src/retained.rs:1091` and `revision.rs:675` are inside test modules.
- **Film and kit.** In the build268 film sources and `repo-view/crates`, `ImageMobject` appears only in `draw.rs` `Picture::build`, in its raster branch (line 723; line 2031 is test code), and in a kit example that is not part of the binary. Neither the film nor the kit loads a persisted scene.
- **Who builds a picture.** Only `portrait_styled` and `Painter::build_styled` call `Picture::build`, and only `portrait` and `Painter::build` call them.
- **Chapters 06 and 08.** Neither chapter names `Picture`, `Painter`, `portrait*`, `PLATE*` or any `Image*` type. Its `image_at` is the geometric image of a square. The draw helpers they import (`Plane`, `dot`, `predict`, `chain_arrows` and the rest) build circles, arrows, cards and TeX, and call none of those four functions.

**So in 06 and 08,** every `fade_in_all` and `fade_out_all` call does the family walk, `get` and `image_resource()` for each mob, finds nothing, and pushes nothing. The list passed to `play_prepared` is the same as under build238.

**Other paths.**
- The chapters' own engine fades (`fade_in(`/`fade_out(` in `one_derivative` and `fade_out(` twice in `loop_sums`) do not go through the helpers.
- The `predict`, `reading_window` and `precisely_over` bodies that the direction map names are unchanged. They only move by 40 lines (1211/1265/1312 → 1251/1305/1352).

**Not established.** Nothing here shows equal frame bytes. Static reading can't exclude an engine effect of the extra read-only calls. I know of no mechanism for one, and the facts output is identical (`7d528509…`).

## 3. Narration: rebind242 against rebind269

**The WAVs.** `R/jobs/csd-production-assembly19/narration`, `…rebind242/narration` and `…rebind269/narration` hold the same 109 file names. All 105 WAVs are byte-identical across the three, including the 14 of chapters 06 and 08.

**The only differing files are `external-units.json`, `report.json` and `assembly.json`.** Every difference between the 242 and 269 copies falls into one of these groups:
- **Direction-map binding:** `narration-direction-map-frederick-review242.json` `c9e9ad33a9e55a1020591fe242f9b3a7e3bbdf705dfb8f563cef4110d7d26b67` became `…review269.json` `7eb0e40d6366950169eb10727b3386fa1d290eaeb60078d609d5e57a0ef344e3`.
- **Paths:** evidence paths from `work/review-repair238/workspace` to `work/review-repair268/workspace`.
- **Other chapters' holds:** gallery hold 5 (`gallery.rs:866` → `:876`, sha to `d57e003a…`) and rigidity holds 13 and 14 (`:473/:499` → `:502/:528`, sha to `0bcfbc7d…`), with their reason strings.
- **Derived hashes:** `units_sha256` (`1bc1b76d…` → `fd90880f…`) and `report_sha256` (`9aebd4c2…` → `b4a4c91e…`).

**What is equal.**
- `report.json` `lines` (105 rows) is equal.
- `assembly.json` `outputs` and `manifest.units` are equal.
- Both rebind summaries record 23 holds, 5 helpers, 105 byte-identical WAVs and 105 verified and selected cues, in `external-ui` mode.

**Chapters 06 and 08 in the direction map.** Their holds are 06_8 (`one_derivative.rs:734`, `predict`) and 08_5 (`loop_sums.rs:1186`, `reading_window`), with unchanged file hashes. Only the `draw.rs` and `mod.rs` file hashes and the `draw.rs` line numbers of their helpers changed (section 2).

**The canonical `narration_units.json`.** In both workspaces it was `995e25c7…` before the rebind. After the rebind the two copies differ only in `direction_contract.map` (path and sha), which is the same binding change as above.

## 4. Schedules

| Table | timeline21 | timeline242 = preview243 (bytes) |
|---|---|---|
| 06 timeline | `dbc82d8481d8f511…` | `b7cfd82fe14f1c41…` |
| 06 cues | `ee8eb35a2d8621e5…` | `ee8eb35a2d8621e5…` |
| 06 cuelog | `91f5695913803d17…` | `49291e9bc414d6f8…` |
| 08 timeline | `fed5faea5359e1c1…` | `9d6425d62c5e4543…` |
| 08 cues | `1beae064b4b1a811…` | `1beae064b4b1a811…` |
| 08 cuelog | `cadf1d0cfcd98fa3…` | `6f0c12086c96387a…` |

- **What is already shown.** The preview243 tables equal the timeline242 tables byte for byte. Timeline242's controller asserted that those equal timeline21 with `csd-production-assembly19/narration` replaced by `csd-review-repair-rebind242/narration`. The frame counts are 10,030 and 14,959.
- **Layout tables.** Preview243's layout tables are 06 `e3b0c442…` (empty) and 08 `7be1c4b12ad55f624040c3217f98587c89c3f94f07807c0f8e7f211d309aa951` (2 findings at 205.6 s, the intended cancellation strokes per 271).
- **Still pending.** Timeline269's 06 and 08 tables must equal timeline21 with the path replaced by `…rebind269/narration`. That is equivalent to equalling the timeline242 tables with `rebind242` replaced by `rebind269`. `csd-reviewed-final278.py` `check()` asserts it for all 15 chapters. Until timeline269 finishes with exit 0, the schedule half of this binding is unproven. The permit's `timeline_sha256` comes from that output.

## 5. What reviews 270 and 271 carry forward

**What the binding carries.** Given sections 1–4 and a passing timeline269, the 268 render runs the same scene code against the same schedule and narration. Findings about the following therefore apply to the final without re-review:
- what is drawn and written;
- positions in scene units;
- when things appear and leave, at frame numbers.

**What it does not carry.** Anything measured in 540p pixels or attributed to the CRF 18 veryfast preview encode. Nor does it carry what neither review looked at.

| | Carries forward by this binding | Needs a fresh check on the 4K final |
|---|---|---|
| **270 (06)** | F1 shared scale (equal plane scales, both 97.0 px/unit at 540p); the F2 qualifiers' text and times (48.5, 52.5–53.0, 75.5–75.93, 111.88–128.85 s); fan geometry; the zoom labels and readouts; the quiz (ring full at 141.05 s, box fades at 146.5 s, answer at 147.5/148.0 s); end fade at 167.0 s; Lows L2, L3, F3 and note N1 as source facts | R1 (3 px from the frame stroke at 540p) and R2 (2 px from the Im axis) at native 4K; L1's margin (≥ 12 px at 540p; "about 50 px at 4K" was an estimate); the contrasts (lowest 4.6:1, the 06_7 qualifier) on the final encode; F3's glyph weight at 4K; the input pinwheel's heads at the hub |
| **271 (08)** | M1 fixed (the plain "Selected square, magnified" caption, 163.5–217.2 s); the c/Dₙ fade at 170.3–170.6 s and return at 172.3–172.57 s as a schedule fact; L3 cleared; R1 stale caption at 231.0–249.0 s; L1, L2 and L4 as source facts; O1's two lint rows at 205.6 s | N1: 271 puts the level and colour steps on small static labels (c ≈ 7×9 px at 540p) down to the preview encode, and that needs confirming on the 4K CRF 16 output; L4's dot under the width-7 edge; O4's 12 px/5 px clearances; the struck 4ⁿ; O5's apostrophe |
| **Neither** | nothing | motion (270 lists the unsampled hand-over, fan, fades, flip and zoom; 271 checked stills only); voice timing and listening; music, absent from preview243 and added in final278 by `--score`; fine sync; independent final AV |

**Lows that will persist.** All of these Lows come from the 238 source, so they will be in the 268 final unless the source changes: 270 R1 and R2, L2, L3, F3 and N1, and 271 R1, L1, L2 and L4. If root changes `one_derivative.rs` or `loop_sums.rs` to repair any of them, this binding stops applying to that chapter.

## 6. For the final278 permit

**Proposed `source_binding_evidence` (sha-checked on csd as listed).** It is the same for both chapters, plus that chapter's own source file:

```json
[
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-build238/resource-receipt.json", "sha256": "0dddf8e168870e12374df9ea5c652a944052daacdcd0d1d25318bbc6ae5dc8de"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-build238/source-copy.json", "sha256": "6d4c7e93e89a29930cf5b4c2f3101fba023a3e09b99888f3e9db083bdb1eb65c"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-build238/qualification.json", "sha256": "fe167a8135ac855e82f3af87d538db9e8e7eb24406e9db79cc63108b20035e9b"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-build268/source-copy.json", "sha256": "81d28272a7a23b63199e79e6aa1086f01f131f0ddd685d4ae859d04b1de1934c"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-build268/qualification.json", "sha256": "2f3430a82dc1561eecbc1485aa00c18070606c4d8728fc6811b62b1f141dc857"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/review-repair-recipe268.json", "sha256": "edadbffc1a0740594fee11ec458bc67131016765dc5721f0157675a421b2a08b"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/cc2-plate-fade-candidate264-draw.rs", "sha256": "9bb07b3ff030d1ac0885d1382097bde596e8cae419d86b51933e12e7eeed8218"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/mod-review-repair236.rs", "sha256": "b5817c501f31d7e6b35672804f14a7f61901fdb81d4669c4ef48324df3b30bc6"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17/mod-review-repair256.rs", "sha256": "e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind242/rebind-summary.json", "sha256": "6ea26fed7276424f5c13f959377cf3c8962ab37e75e097a02b44808895ce8ba6"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind269/rebind-summary.json", "sha256": "fa263d8a6c14c21ef037525fd316e2d64019c0f506e934a4e2d38886b5167da6"},
 {"path": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-timeline242/comparison.json", "sha256": "b3bdeb8b31adb5bc9d43550cc15c904c7ef38231568fd1929f9ec9535d606e4d"}
]
```

**Additions:**
- **Per chapter:** for 06, `R/qualification01/production-repair17/one-derivative-review-repair238.rs` `7a4cc01acb497ad881c0d8478c696ab74db025494ab9381d0b414355fe3fd30d`. For 08, `…/loop-sums-review-repair238.rs` `8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5`.
- **When it exists:** `R/jobs/csd-review-repair-timeline269/comparison.json`, and this file once it is on csd.

**The rest of the permit is not source binding.**
- **`actually_reviewed_motion`.** It needs a real motion review. 270 and 271 are stills-only. Because of this binding, a motion review of preview243 applies to the 268 render at the source level.
- **`unresolved_defects == []`.** It needs root's disposition of the persisting Lows in section 5.
- **`timeline_sha256` and `frames`.** These come from timeline269.

**What the worker does and does not check.**
- **It checks:** the tables (timeline and cues against timeline269, and cuelog with the score rows); 3840×2160 at 60 fps with the frame count; x264 CRF 16; 48 kHz stereo.
- **It does not check:**
  - the layout tables. Compare the final's against preview243's (06 empty; 08 `7be1c4b1…` with its two rows at 205.6 s).
  - the 4K picture items in section 5;
  - the music mix;
  - listening, fine sync, motion or final AV.

## Limits

- **Read only.** No compile, run, render, extraction, synthesis or listening.
- **Static reasoning.** The behaviour argument rests on reading the source. It is not an equality of decoded frames or binaries.
- **Dependency rlibs.** That both builds linked the same files rests on mtimes, not hashes.
- **Timeline269.** It was incomplete when read, so the schedule binding is conditional on it.
- **What I did not do.** I did not re-derive the measurements in 270 or 271. I did not look at any picture.
- **Narration.** It stays ElevenLabs Frederick Surrey, by the bound WAVs. No audio was heard.
