# Algebra fixes: coset links and the 11_17 first return

WhiteSwan, for root, under bead mx-video-program-program-87do.39 (dispatch-3). This repairs the two findings in `N/coset-link-overlap-review01.json` and `N/permutation-first-return-wording-review01.json`. **Nothing was compiled, rendered or synthesized.** Here N is `narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/native-stage08-root01`.

## Base
- The crate is `N/corner-label-contrast-repair01/workspace/explainer_corpus/abstract_algebra_expanded`. In its registry (`src/main.rs`), `14_cosets` maps to `cosets::Cosets` and `11_permutations` to `permutations::Permutations`.
- `cosets.rs` (`ab0e0300…`) and `permutations.rs` (`092256d8…`, the hash the review names) are byte-identical in all four stage workspaces: `workspace`, `corner-label-contrast-repair01`, `quotient-picture-repair01` and `targeted-repair12_7-stage01`. The script there is `0da89056…`.
- Exact copies are kept as `before-cosets.rs` and `before-permutations.rs`. No other workspace file was touched.

## 1. Coset links (14_14, about 115.8 s to 124.45 s)
**Before:** each right-coset link was a straight line from tile centre to tile centre. The e–s and r²–r²s links ran down through the names `e` and `r²`, and the two diagonals crossed corner numbers and met on top of `r²`.

**Now:** `link_route` draws each link as an open polyline. It leaves the bottom edge of the rotation's tile 0.25 right of the column centre and enters the top edge of the flip's tile, so no link enters a tile. A link that changes column turns into a horizontal lane in the gap between the rows: 0.1 for the link going right (r to r³s) and −0.1 for the one going left (r³ to rs). Both lanes lie below the top row's names and above the bottom row's tiles. The pairs, colour, stroke width, order and timing of the links are unchanged, and the columns (left cosets) stay as they were.

Corners in scene units, from `checks/outputs/check_fixes.txt`:

| Right coset | Route |
|---|---|
| {e, s} | (−5.15, 0.49) to (−5.15, −0.24) |
| {r, r³s} | (−3.7, 0.49), (−3.7, 0.1), (−1.3, 0.1), (−1.3, −0.24) |
| {r², r²s} | (−2.25, 0.49) to (−2.25, −0.24) |
| {r³, rs} | (−0.8, 0.49), (−0.8, −0.1), (−4.2, −0.1), (−4.2, −0.24) |

Each link's round cap just meets its own tile's edge. The closest any stroke comes to a name box is 0.07, using a box ±0.15 wide and ±0.1 tall around each name. The links still cross each other three times, and they cross column-frame borders, but they never run along a border or along each other.

**Heading:** the review also noted that the heading still says "left cosets" while the right cosets are drawn. At the start of 14_14 the heading now fades to "When order matters: left and right cosets" (34 pt, about 6.2 units wide by CMR10 widths), and 14_5 hides that new heading. This is optional: to drop it, restore the three `heading`/`both` lines from the patch.

**Timing:** sentence 0 of 14_14 now runs 0.7 s of animation (0.3 s fade-out, then 0.4 s fade-in) instead of 0.3 s. I didn't measure Frederick's sentence 0 ("Now multiply on the right instead."). If it is shorter than 0.7 s, the later beats slip by the difference.

**New test:** `right_coset_links_touch_their_tiles_and_cross_no_tile_or_name` checks:
- each right coset is a rotation and then a flip;
- the route ends meet the tile edges;
- the segments are axis-aligned and cross no tile and no top- or bottom-row name;
- no segment lies along a frame border or another link;
- the two lanes differ.

## 2. 11_17: first return two, and six is a return too
**On screen:** the card read `Home after 2, not 6`, which denies the return at six even though (1 3)⁶ = e. It now reads `First return: 2 (it also returns at 6)`, computed from `first_return` and `lcm(3, 2)` as before. It is about 4.4 units wide, centred at y = −2.2, and has clear space between the tempting `lcm(3,2)=6?` above it and the "Overlapping cycles interfere" caption below it.

**New test assertion:** `home_runs(&p, &every, 6) == [2, 4, 6]`. So two is the first return, and six is a return, the third one.

**Narration:** the full replacement is in `11_17-frederick.txt`, to be synthesized as one whole cue with ElevenLabs Frederick Surrey, not spliced. Sentence 2 changes:
- **Before:** "Its lengths looked like three and two, but it returns after two runs, not six."
- **After:** "Its lengths looked like three and two, but it first returns after two runs, and six is only its third return."

The other four sentences are byte-identical, so the cue still has 5 sentences (as `repair_tests` pins) and every `until_sentence` index still binds. It is 70 words, up from 64 (limit 100), and contains no digits. No phrase anchor names 11_17.

## Integration (root)
1. Copy `cosets.rs` and `permutations.rs` into the crate's `src/`. `cosets.patch` and `permutations.patch` show the exact changes against the `before-*` copies.
2. Replace the 11_17 row of `script.tsv` and the `oliver_11_17` text in `narration_units.json` with the text of `11_17-frederick.txt`. Because `include_str!` reads `script.tsv`, the tests need the new row too.
3. Synthesize 11_17 as one whole Frederick cue, then build, run the tests and Clippy, and re-read the pictures for 14_14 and 11_17.

## Checks run
- `rustfmt --check --edition 2024 --config-path rustfmt.toml` on a scratch mirror of the crate's `src/` with both candidates: exit 0. This parses every module and checks format only, not types. The base passes too, and `git diff --no-index --stat` against the base shows only these two files.
- `python3 -I -B checks/check_fixes.py <this folder> <current script.tsv>`: PASS. It checks the 11_17 sentences, words and digits, the returns of (1 3), and a Python mirror of `link_route` against every tile and name box. Its output is in `checks/outputs/`.

## Not established
- Compilation, the Rust tests, Clippy and any rendered frame. The name boxes are estimates, not measured glyphs.
- Any audio, timing or listening result for the new 11_17 take.
- The review's other note, the empty transposition scaffold at 265–270 s, is outside this dispatch and untouched.

## Guarded build helper (prepared, not run)
Root launches it. I ran `prepare_build01.py` only into a scratch `--out` folder for the tests, and I ran no cargo and no guard.

It adapts root's `N/corner-label-contrast-repair01/build01.sh`, which built successfully as job `algebra-corner-contrast-build01` (exit 0, admitted, inputs `810c6773…`). The new output folder is `build01/` beside these files, and the new guard job is `algebra-fixes-build01`.

**Invocation, from the repository root (Q is this folder's parent):**
```
python3 -I -B Q/algebra-fixes/prepare_build01.py
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py \
  --phase native --job algebra-fixes-build01 --inputs "$PWD/Q/algebra-fixes/build01/build01-inputs.json" \
  -- sh "$PWD/Q/algebra-fixes/build01.sh" > Q/algebra-fixes/build01/build01-controller.log
```
The guard wrapper is the one root's build01 ran under. It loads root's own guard, pinned by hash, so this run gets:
- the same exclusive native lane lock;
- 10 GiB admission, an 8 GiB free-space stop and 4 GiB growth per phase;
- a 56 GiB work cap and a 20 GiB target-growth cap;
- the original `allocation-baseline.json` in `five-film-release-20261027`;
- the input manifest checked before and after the run.

**`prepare_build01.py` (light; no cargo).** It refuses before writing anything unless all of these hold:
- root's build01 receipt is a clean, admitted exit 0 of `build01.sh` under that exact policy, and both its input hashes equal the manifest's bytes;
- every one of root's 4,271 inputs still has its recorded hash;
- root's facts, chapters, `build01.sh` and log, and the five files here have their pinned hashes;
- the output folder and the guard job do not exist yet.

It then:
- copies root's workspace exactly and installs the two Rust files;
- replaces the 11_17 row with the whole text of `11_17-frederick.txt`;
- rebinds the plan: the `oliver_11_17` text, both script hashes, and direction map `stage14`, which is stage13 with the new script hash and the hashes and lines of the seven holds in the two changed files (the 14_2 hold moves from line 277 to 304; the six in `permutations.rs` keep their lines);
- validates the plan with the copied workspace's own `tools/coherent_narration.read_units(…, require_directions=True)`;
- writes `expected-tests.txt` (root's 122 passing tests plus the new coset test), `build.env` (root's engine patch table, read from root's `build01.sh`), `build01-inputs.json` (root's 4,271 rows unchanged, plus 86 for this folder's build files, the new workspace and root's outputs) and `prepare-receipt.json`.

**`build01.sh` (run by the guard).** It is root's script with these changes:
- The output folder is `build01/` here.
- The lock check, `cargo fmt --check`, and the `cargo test`, `cargo build` and `cargo clippy … -D warnings` commands are root's own, with the same flags.
- After the tests, `expect_build01.py tests` requires all 123 expected tests to pass, none to fail or be ignored, no other test to run, and the one summary line to count 123.
- After the build, `expect_build01.py facts` runs root's facts comparison, which requires 240 unique keys with native equal to oracle. It also requires both files to be byte-identical to root's build01, since neither repair touches a fact; no fact key reads the script.
- `native-chapters01.txt` must `cmp` equal to root's.

The script writes only inside `build01/` and the shared `target/geometry-kit01`. Like root's script, it keeps any existing release binary as `build01/before-build01-abstract_algebra_expanded-release`.

**Prerequisites:**
- Run from the repository root with Python 3.11 or later, because `hashlib.file_digest` is used here and in root's guard.
- `Q/algebra-fixes/build01` and the guard job `algebra-fixes-build01` must not exist.
- Root's corner-label workspace, its build inputs and the scratch engine tree must be unchanged since that build. Prepare checks this and refuses otherwise.
- The guard admits the job: `sbh check` must pass, at least 10 GiB must be free, the work folder must be under 56 GiB (about 51.4 GiB at root's build01), and target growth over the original baseline must be at most 20 GiB.
- The build needs no audio. Synthesizing the 11_17 take, then timeline and pictures, comes after the build.
- `raw11_17-ui01/` is not mine and is untouched. Nothing here reads it.

**Tests (`checks/test_build_helper.py`, output in `checks/outputs/`):** 24 checks PASS, in 2.8 s on a scratch folder. They run prepare for real into scratch and then verify its output independently:
- only four workspace files change and one is added;
- the script row, the stage14 diff and every moved hold land on their marker;
- the plan diff;
- every manifest hash is current, with root's rows kept first;
- the expected test list and `build.env`.

They also cover:
- a second prepare refuses and writes nothing;
- `expect_build01.py` passes root's real log with the new test added, and fails root's log alone, a failed test, an ignored test and a stale count;
- the facts check passes root's facts and fails a changed value and a repeated key;
- `build01.sh` parses (`sh -n`), resolves the repository root, and contains root's four cargo commands unchanged except for `--config "$BUDGET"`.

**Not established:** any cargo step, and the guard's admission today. `build01.sh` itself has not been run end to end, even with fake tools, because its output path is fixed to `build01/` here, which is root's to create.

## Files (sha256)
| File | sha256 |
|---|---|
| `before-cosets.rs` | `ab0e0300537f163d4ea6e8b7a5da07949c7b2dd20f88af122aa4cb4555150b06` |
| `before-permutations.rs` | `092256d823983e8a36267c833b16eaf6d1e767d86d254a7253490906505a32f2` |
| `cosets.rs` | `63a62ffdf4a2bf8d7aac6e27b4ecb161e0d8440ff1b7ccdad919bccade441187` |
| `permutations.rs` | `a5472cd6a0ee583a97cc1986809a82ec32dcf9fe7a5d8901b47a3b05d7c3255c` |
| `cosets.patch` | `247cba9ba48c6934f2598121586d2de25b2fcd5109986b0c9e3290b61e871758` |
| `permutations.patch` | `77062f97b89a3d779a36ae651a05b8721ce2e8a21fe4100a6301d28886d01861` |
| `11_17-frederick.txt` | `ed410300d6b2a1bea36f719d9f8da3eb6ef0837235cf338efff79f6c6e3ef3da` |
| `checks/check_fixes.py` | `0aac660dc90625f9f74b00004408b893ab35d22fe01336f6d5fd759a7f9b17b7` |
| `checks/outputs/check_fixes.txt` | `6de5dd48696512926652bbadd7612cf87aee380dee5a7734e55bbe51eb2eb9f0` |
| `prepare_build01.py` | `058826d2d6973954b358a856969da05cd70d8a668df195f434bf186d5155d9ae` |
| `build01.sh` | `5701e56c990c53a22b022491b0e8943545bb6e07f24a9ca4d193238b1afef493` |
| `expect_build01.py` | `3bdf70cdc15eae903d2db23e2da9c59ee76b0a8f347eda40c9d2c2c84c21a001` |
| `checks/test_build_helper.py` | `5dd5a3fe3abd43dbcfd3d0f79dd47fb33ac690aba2d2e71f5d8c589be63211e0` |
| `checks/outputs/test_build_helper.txt` | `1079e8246451e033b8e51914ef256ee8b08862d25b5b7cdf644a119e4ecfed60` |
