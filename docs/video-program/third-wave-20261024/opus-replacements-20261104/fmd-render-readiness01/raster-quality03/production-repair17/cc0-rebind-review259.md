# rebind review 259: review-repair-rebind258 and timeline258 against production19 and recipe256

Dispatch-0 (cc0), 2026-11-08. This is a read-only audit. I ran no rebind, timeline, helper, build or render, and nothing on the GPU host. I wrote only this file. Source and Beads are untouched.

## Result

**No implementation defect found.** Against the production19 direction map, the `prepare-assembly19.py` relocation contract and the eight recipe256 candidates, rebind258 would rebind every changed hold and helper marker uniquely. Its assertions, together with timeline258's, cover all 105 audio and timing records.

**One open dependency, not a defect.** Build256 was stopped before admission. Rebind258 and timeline258 still name build256's workspace, binary and job, which are listed below. They are unbound until root names the corrected build. CC2's corrected260 `draw.rs` also needs the short pre-check below before that build.

## Inputs (sha256)

| File | sha256 |
|---|---|
| `review-repair-rebind258.py` | `7cecf50712374d45fcefcb0271f76bebb790b7f5df87a048238019bcfe1c7482` |
| `review-repair-timeline258.py` | `e36f17e7c13b3b3e8160541735623fea5a56660b4be097ba8173154e3c8c04da` |
| `review-repair-recipe256.json` | `21e24bdf802f00bb31229efc7273713dc26edac0b33536a17eba25dc1526dbb3` |
| `review-repair-build256.py` | `b784c0bbd042785ae20b0f14ff671d610a2162f532c89937c957f1e607b528f3` |
| `prepare-assembly19.py` (helper; read, not run) | `2dec9a7a817f6940f819a39b9e403d7c7bde779fa551e71e504f6659e3e2269e` |
| `review-repair-rebind242.py` (the predecessor that completed) | `efbe7d406dfe61ae1ca0216db388c64c5593bf2e84ea6b25b37dc874d77eec87` |
| production19 map, `narration-direction-map-frederick-production19.json` | `0d5bec55f58d40ad9b8ef5d81163606f9a48408786d4f82804ee403b327bdc29` |
| production19 `narration_units.json` (its contract binds the map above) | `995e25c7788943deff65c4d84050d8a17edc3c2c3498be0d665d7ac5b661357a` |
| `timeline21-terminal01/audit.json` (local copy of the base tables) | `7a843816b3fca94163c0c8f97dd7e9a341ff8a650905188e87766c2981f9d927` |

- **Base tree.** `production-source-preserved214/workspace`. Its `receipt.json` names `work/production-repair19/workspace` as its source, which is build256's `S`.
- **Candidates.** The recipe256 candidates match the recipe's hashes:
  - `boundary-label-repair52.rs` `dc80c541…`;
  - `loop-sums-review-repair238.rs` `8a13e8a1…`;
  - `one-derivative-review-repair238.rs` `7a4cc01a…`;
  - `mod-review-repair256.rs` `e1f4d1b8…`;
  - `rigidity-review-repair247.rs` `0bcfbc7d…`;
  - `gallery-review-repair249.rs` `d57e003a…`;
  - `cc2-plate-fade-candidate246-draw.rs` `aab61199…`;
  - `cc2-plate-fade-probe246.rs` `c0c56084…`.

## Method

- **Delta from the proven predecessor.** Rebind258 differs from rebind242 only in:
  - the workspace, binary, build and job names;
  - the `allowed` file list, which grows from 4 files to 8;
  - one new qualification assertion (`explicit_native_plate_fade_probe_passed`).

  Rebind242 completed with exit 0 (the Beads record: 23 holds and 5 helpers rebound, 105 WAVs byte-identical, timing and generation records retained). So the copy, verify and delivery mechanics are proven. This audit concentrates on what the new candidates change.
- **Scratch tree.** I built a scratch copy of the production19 crate source and overlaid the eight candidates under their target names, with `plate_fade_probe.rs` beside `draw.rs` for its `#[path]` module.
- **Formatting.** `rustfmt --check --edition 2024` with the repository `rustfmt.toml`, run from `main.rs` (which parses every module), passes on the whole tree. Build256 runs `cargo fmt` before hashing its source copy, so the files relocation would see equal the candidate bytes. This assumes the host's nightly-2026-08-31 rustfmt and `repo-view/rustfmt.toml` match the local nightly 2026-08-30 build and root config. The single-file failures for `mod.rs` and `draw.rs` were only unresolved child modules.
- **Relocation rule.** I re-implemented `relocate_map`'s marker rule independently; the helper was not imported or run. A marker matches a line only when the line, stripped, equals the marker exactly. A hold must match exactly once strictly between its own `let line = say(&kit, stage, "<cue>")?;` and the next such call (or the end of the file). A helper must match exactly once in its file. I then applied rebind258's own per-row checks: every field except `line` and `sha256` unchanged, and any hash change confined to the allowed files.

## Changed holds and helpers (all 13 rows in changed files)

| Row | File | Line before → after | File sha | Matches in file / in cue interval | Interval |
|---|---|---|---|---|---|
| hold 05_8 | gallery.rs | 866 → 876 | 7d1cf20f → d57e003a | 1 / 1 | 853–877 |
| hold 06_8 | one_derivative.rs | 663 → 734 | 4f0154ec → 7a4cc01a | 1 / 1 | 685–737 |
| hold 08_5 | loop_sums.rs | 1177 → 1186 | 69042b14 → 8a13e8a1 | 1 / 1 | 1180–end of file |
| hold 09_4 | boundary.rs | 938 → 938 | c9e6b5ba → dc80c541 | 1 / 1 | 920–941 |
| hold 09_5 | boundary.rs | 1003 → 1003 | c9e6b5ba → dc80c541 | 1 / 1 | 941–1004 |
| hold 09_8 | boundary.rs | 1282 → 1282 | c9e6b5ba → dc80c541 | 1 / 1 | 1253–1285 |
| hold 10_4 | rigidity.rs | 473 → 502 | c577e2c0 → 0bcfbc7d | 1 / 1 | 427–505 |
| hold 10_5 | rigidity.rs | 499 → 528 | c577e2c0 → 0bcfbc7d | 1 / 1 | 505–529 |
| helper `predict` | draw.rs | 1211 → 1251 | cd394e5a → aab61199 | 1 | — |
| helper `reading_window` | draw.rs | 1265 → 1305 | cd394e5a → aab61199 | 1 | — |
| helper `precisely_over` | draw.rs | 1312 → 1352 | cd394e5a → aab61199 | 1 | — |
| helper `PREDICTS` | mod.rs | 31 → 31 | 4f4b17c3 → e1f4d1b8 | 1 | — |
| helper `READING_WINDOWS` | mod.rs | 58 → 58 | 4f4b17c3 → e1f4d1b8 | 1 | — |

- **All 13 relocate uniquely.** Each hold stays inside its own cue's interval, and every changed hash lies in a file on rebind258's `allowed` list.
- **Untouched files.** The other 15 hold rows are unique and unchanged.
- **The cause of rebind239's failure is covered.** The `mod.rs` helper hash change (from the TEXT registry) is now on the allowed list.
- **Test strings cannot collide.** Quoted test lines (for example `"kit.music_breath(stage, 2.0)?;",`) do not equal a stripped marker, so tests that quote source cannot create false matches. This holds even where an interval runs to the end of the file (08_5).
- **Hold durations are unchanged.** These live in `mod.rs`, not in the map. The `PREDICTS` and `READING_WINDOWS` arrays in `mod-review-repair256.rs` are byte-identical to production19's. The registry differences are only the TEXT additions.
- **The other map keys stay equal, as line 61 requires.** The `overrides` texts all equal the bound `script.tsv` rows (sha `234f7257…`, unchanged by the recipe), so `data[key] == original[key]` holds for every key except holds, helpers and scope.

## Audio and timing records (105)

Rebind258 itself asserts:

- every N1 file except `external-units.json`, `report.json` and `assembly.json` is byte-identical in N2 (line 112), and there are 105 WAVs (line 113);
- `report['lines']`, `assembly['outputs']` and `manifest['units']` are unchanged (lines 106–107);
- the direction binding resolves as before (line 104);
- `coherent_narration.py verify --require-directions` passes;
- delivery reports `verified_cues == selected_cues == 105` (line 120).

Timeline258 then requires all 45 tables of the 15 chapters (timeline, cues and cuelog) to equal timeline21's byte for byte, once the narration directory is mapped. I checked its constants against the local copy `timeline21-terminal01`:

- there are 15 chapters, totalling exactly 133,662 frames, as line 45 asserts;
- all 105 cuelog rows name `jobs/csd-production-assembly19/narration`, which is timeline258's `OLD`. That path also appears in the line rows of the 15 timeline tables, and the byte replacement covers those too. `OLD` is not a substring of `N`.
- the tables carry no units, map or source hash that the rebind would change. The cuelog's WAV hashes stay fixed because the WAVs are byte-identical.

The binary embeds only `script.tsv` and `docs/storyboard.md` (`main.rs`), so rewriting `narration_units.json` and adding the new map after the build cannot make the binary stale.

## Unbound to the corrected build (not implementation defects)

Build256 was stopped before admission, and root will name a new build around CC2's corrected260 `draw.rs`. Before rebind258 and timeline258 can run, these build256 references must point at the new build:

| Script | Line | Now |
|---|---|---|
| `review-repair-rebind258.py` | 15 | `TREE = ROOT/'work/review-repair256/workspace'` |
| `review-repair-rebind258.py` | 21 | `BIN = ROOT/'work/bin/review-repair256'` |
| `review-repair-rebind258.py` | 40 | `build = ROOT/'jobs/csd-review-repair-build256'` |
| `review-repair-rebind258.py` | 48–50 | the `allowed` tuple and the exact `source-copy.json['changed']` equality, plus the `explicit_native_plate_fade_probe_passed` key. Both must match the new build's file set and its qualification key. |
| `review-repair-timeline258.py` | 12 | `B = R / 'work/bin/review-repair256'` |
| `review-repair-timeline258.py` | 23 | `jobs/csd-review-repair-build256/qualification.json` |
| `review-repair-timeline258.py` | 10 | `N = …/csd-review-repair-rebind258/narration`. This only needs a change if the rebind job is renamed with the build. |

**Pre-check for corrected260 `draw.rs` and the new recipe.**

1. **The three helper markers stay unique whole lines in `draw.rs`.** Each must count exactly 1 with `grep -c -x -F '<line>' draw.rs`; all three do in 246. Their lengths are 116, 91 and 88 characters, so a signature change could make rustfmt reflow one onto several lines and lose the match.

   ```
   pub fn predict(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken, lines: &[&str], at: Vec3) -> fmn::Result<Vec<Mob>> {
   pub fn reading_window(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<()> {
   pub fn precisely_over(stage: &mut Stage<'_>, said: Mob, whole: Mob) -> fmn::Result<()> {
   ```
2. **The new tree stays rustfmt-stable** as a whole (from `main.rs`), so the build's `cargo fmt` leaves the reviewed bytes unchanged.
3. **The file set stays the eight names above**, or `allowed` and the `changed` equality change with it. This includes `plate_fade_probe.rs` as a new file under `src/chapters/`.
4. **The new build writes the same qualification key.** Its probe and test counts are the build script's business. Rebind258 reads only `binary_sha256`, `chapters` and the probe flag.

The mirror can be re-run on the new tree in seconds; it is held outside the repository.

## Notes (no change required)

- **N1. The newline at the end of `narration_units.json`.** The rewrite drops it. The production19 file is ASCII only, and re-serialising it with `indent=2` is byte-identical apart from that final newline. So the rewrite changes only the map binding plus that newline, as in 242. The original is kept as `narration_units.before-review258.json`.
- **N2. Retry hazard.** The rebind starts changing `TREE` at line 67 (the new map), then lines 82–83 (backup and rewrite of `narration_units.json`), before its remaining assertions and the verify and delivery runs. A failure after line 67 leaves `TREE` different from the build's `source-copy.json`, so a corrected retry against the same workspace would stop at line 51. A failed run would need either a fresh workspace from the build or a retry that accepts those two known changes. This is the same as in 242, which completed.
- **N3. The summary's fixed counts are safe.** It writes `'holds': 23, 'helpers': 5` as constants (line 121). The helper's own requirement of 23 and 5 (`prepare-assembly19.py` line 101) guarantees them, and `zip` at line 63 cannot drop rows silently.
- **N4. The open-ended 08_5 interval.** The 08_5 hold interval runs to the end of `loop_sums.rs`, tests included. It is unique today, but any future unquoted line `reading_window(&kit, stage, &line)?;` in that file would break it.

## Limits

- This is source and record reading only. Nothing ran on the host, and I have not seen the GPU host's assembly19 narration files, `assembly.json` dispositions or the timeline21 originals beyond the local `timeline21-terminal01` copy.
- The relocation result rests on my re-implementation of the helper's rule and on the formatter-stability assumption above.
- No audio was heard or checked. This audit gives no picture, motion, fine-sync or final AV acceptance.

## Addendum, 2026-11-08T05:28Z: final260 and the 263 references

**CC2's final 260.** `cc2-plate-fade-correction260.md` (sha `2fb8c76f…`) declares `cc2-plate-fade-candidate260-draw.rs` final at `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064`, with the probe unchanged at `c0c56084…`. The file on disk matches. (An earlier read of the note named the superseded draft `c182e336…`; the note was rewritten at 00:27.)

**Marker check.** All three helper lines occur exactly once as whole lines in 260, at 1251 (`predict`), 1305 (`reading_window`) and 1352 (`precisely_over`). These are the same lines as in 246, consistent with CC2's statement that 260 changes only the test module.

**Whole tree.** This is production19 plus the seven unchanged recipe256 candidates, with 260 as `draw.rs` and the probe as `plate_fade_probe.rs`.
- `rustfmt --check` from `main.rs` passes.
- My relocation mirror again finds all 23 holds and 5 helpers unique: 13 rows in changed files, every changed hash on `allowed`.
- `PREDICTS` and `READING_WINDOWS` are unchanged.

**263 references.** I checked each script against its predecessor by diff.

| Script | sha256 | Changes from its predecessor |
|---|---|---|
| `review-repair-rebind263.py` | `8f015c4e…` | Only the build262 workspace, binary and job, and the job, map and backup names for 263. The `allowed` tuple, the `changed` equality and `explicit_native_plate_fade_probe_passed` (lines 48–50) are unchanged. |
| `review-repair-timeline263.py` | `63be48df…` | Only job 263, narration `csd-review-repair-rebind263/narration`, binary 262 and build262's qualification. |
| `review-repair-after-build263.py` | `d4b22146…` | Binds workspace and binary 262, build262's receipt, and rebind263 then timeline263, each frozen against its predecessor's inputs. Its final line now names all 45 schedules. |
| `review-repair-build262.py` | `120ac607…` | Expects 233 tests, as CC2 states. It still writes the probe key that rebind263 reads. |
| `launch-review-repair262.py` | `dc84ab3d…` | Reads `review-repair-recipe262.json`. Its build238 and timeline242 receipts are deliberate earlier-success prerequisites, unchanged from launch256, not stale names. |

No other name from 238, 242, 256 or 258 remains in these five scripts.

**Recipe262.** `review-repair-recipe262.json` (sha `24be73c5…`) arrived during this check.
- It has the same eight targets as recipe256.
- `draw.rs` is now `cc2-plate-fade-candidate260-draw.rs` (`923ad2ef…`).
- The other seven entries, including `plate_fade_probe.rs` (`c0c56084…`), are identical to recipe256's.
- All eight hashes match the files on disk.

So build262 composes exactly the tree checked above. The retry hazard (N2) is unchanged and retained by root.
