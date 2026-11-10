# Handoff 732: guarded CSD build 731 for apportionment chapter 06

cc0 (LavenderPike), 2026-11-10, 01:21 UTC. Source preparation only. I used no SSH, Cargo, render or synthesis, made no Beads change or commit, and edited no canonical file. Helper 731 has not run on any host. All prior files, including build656 and its helper, are preserved.

## Result

- **Ready for root's audit and one launch.** Helper 731 builds the four exact 722 candidates on build656's retained closure. It writes into a new job folder only and keeps a copy of any older apportionment binary that Cargo will replace.
- **All of 656's guards are kept,** along with the 716 guard and stop functions that ran on the CSD from 00:44:57 to 00:47:01 UTC.
- **Five files must be transferred.** Finished 656's job folder supplies everything else, and the helper binds it by hash.
- **This is not a reuse decision.** The 06 hold rebind, 93-PCM equality and the 45 normalized tables are later steps (see "Later, before any retained render is reused").

| File | sha256 |
|---|---|
| `apportionment-ch06-build731.py` | `8963f280589090c524137465694fc82aabe8d6608f382c6d6e4155bb412d4b00` |
| `apportionment-build656.py` (base, unchanged) | `b5990c28d25b4daeb7e1247151acc390ccdaae81ee31f74ac911f0ee9670f040` |
| `cc0-apportionment-repair-review725.md` (accepted) | `20991b7ad7b88a61c1d4c0ac6d951127b8d9af401c2ba2b19a2f4f3c464795d9` |
| `apportionment-ch06-repair722/handoff722.md` | `0516ec6c1b32b25dc219c027d1b8246c76b46092bccfafc884cdacfdd377fa1e` |

Review 725's three Lows stay open: F1, the narration check is not limited to 06_4; F2, the 06_3 tag starts move inside the fixed 1.5 s group; F3, Cedar's wedge touches the top tick.

## What 731 builds

The job folder's `workspace/` holds the 95 workspace files that finished 656 recorded, copied at its hashes. The only change is that the four candidates replace the four canonical files:

| Path under `explainer_corpus/apportionment/` | 656 and canonical | Candidate |
|---|---|---|
| `src/chapters/not_every_time.rs` | `d3e88f7b…` | `04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9` |
| `src/facts.rs` | `b26bf1e4…` | `e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d` |
| `oracle.py` | `d9a5a6ab…` | `8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f` |
| `docs/storyboard.md` | `564e86db…` | `17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd` |

Frozen and checked in 656's record: `script.tsv` `b9344e11…` and `docs/description_numbers.tsv` `f8e46262…`. The second is now a compile input of `facts.rs`, as review 725 asked. Also pinned:

- `Cargo.toml` `c74357f0…`.
- `Cargo.lock` after 656's edit `a4187255…`.
- `src/main.rs`, `src/chapters/mod.rs` and `src/chapters/draw.rs`, the last of which holds the hold helpers.

The engine is used in place, read only, through 656's own `pristine-engine.toml` (`a96a5777…`), as 716 did. The kit and score are copies, so Cargo compiles them again at the new path. No Cargo.toml path is rewritten.

Pins:

| Pin | Value | Source |
|---|---|---|
| `SOURCE656` | 4,152 files, `77fd892a…` | 656's whole recorded map |
| `WORKSPACE731` | 95 files, `eb860846…` | this job's workspace |
| `CRATE731` | 49 files, `6a63d40f…` | this job's crate |

I derived `SOURCE656` from exports 652/655 plus 656's two edits. The parts 716 already checked on the CSD agree with it: the root `Cargo.toml` `2fa23046…`, all eight kit, config and toolchain-file pins, and the patch text byte for byte. If `inputs` refuses on it, nothing has been created. Send me the printed listing.

## Steps and what each must show

| Step | Pass condition |
|---|---|
| `inputs` (no Cargo, writes only the manifest) | 656's receipt has exit 0 and stable inputs, and its map is at `SOURCE656` with every file re-hashed. The patch file and binary match. Its `tests.stdout` shows exactly the 30 names below. Its `oracle.stdout` is at `2d622f71…` and its facts equal the oracle at 443 keys. The 5 transferred files are at their hashes. |
| tests | `cargo test --bin apportionment --release`: one result line, 33 passed, 0 failed, 0 ignored, 0 filtered out. The passed names are exactly 656's 30 plus `chapters::not_every_time::tests::the_printed_steps_are_the_exact_shares`, `chapters::not_every_time::tests::the_question_reads_the_house_constants` and `facts::tests::the_census_year_is_the_sourced_row`. |
| clippy | `--all-targets --no-deps -- -D warnings` with `CARGO` set. 656 linted only the bin target. This is the first strict pass over this crate's test code on the CSD, so a finding in old test code is possible. `clippy.stderr` names the file and line. |
| build | Copied to `apportionment-native731`. Any older `work/target/release/apportionment` is copied first to `apportionment-previous`, and `preflight731.json` records whether it is 656's binary. |
| facts | Native facts equal `oracle.py` at 443 keys. Both outputs are byte-equal to 656's `facts.stdout` and `oracle.stdout`, and the oracle's is at `2d622f718124cc693cc298cda54784153d1e9705d377462925fce5440567b0fd`. `oracle.py --check` exits 0, prints the same facts and reports `ok census year 1880: the sourced row and the narration agree`. |
| chapters | The 15 keys, equal to 656's record. |
| script | Binary script equals `script.tsv`: 93 cues, 4,166 words, one owner each, numbered in order. |
| static | Every cue said once as `say(&kit, stage, "NN_k")` in the 17 modules `chapters/mod.rs` compiles. `not_every_time` says 06_1 to 06_6, in order. |
| cue probe | `render all --scratch-voice --res 64x36 --fps 1`. Each chapter's ids as said, joined, are the script order. The timing is estimated. |
| glyphs | All 15 chapters register strings, with 0 failures. Chapter 06 registers the five strings 722 builds at run time: the question, the three `+n/39 turn` labels and `1880 census, C. W. Seaton: … from 275 to 350`. The last is the native proof that the embedded TSV row gives 1880. |

The workspace listing and every manifest row are checked again at the end. `qualification731.json` is written only when every step passes.

## Minimum transfer to the CSD

Place exactly these five files under `Q = /data/projects/jmev-stroke-probe-20261106/qualification01/production-repair17`, as regular files, not links:

| CSD path | sha256 |
|---|---|
| `Q/apportionment-ch06-build731.py` | `8963f280589090c524137465694fc82aabe8d6608f382c6d6e4155bb412d4b00` |
| `Q/apportionment-ch06-repair722/not_every_time.rs` | `04499441ccde5be5f827b9768da26bbd0672f21e283728b298809f827d7225e9` |
| `Q/apportionment-ch06-repair722/facts.rs` | `e9b86e26f7e285ddc397a8948462492c824e77a637a4904e501466a7ece78f1d` |
| `Q/apportionment-ch06-repair722/oracle.py` | `8c1dd3af893059601f1c6a195a3292e7274d796edd4701adfefd6fb3d780c94f` |
| `Q/apportionment-ch06-repair722/storyboard.md` | `17a46b19463c07dd4713527574bcb46d56e8dc4dba6b648cc1daac4817f8ffbd` |

Nothing else is read from the transfer. The helper takes the rest from `jobs/csd-apportionment-build656/`:

- every file of its recorded map;
- `qualification.json`, `resource-receipt.json`, `pristine-engine.toml` and `apportionment-native656`;
- `tests.stdout`, `facts.stdout` and `oracle.stdout`.

It hashes each one. The 722 diffs, handoffs and `before-*` copies are not needed on the CSD.

## Exact invocation

Run after `csd-exponential-build716` has finished, which root records at 00:47:01 UTC, and when no other Cargo job is using `work/target`. Launch once and never duplicate it.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17
sha256sum $Q/apportionment-ch06-build731.py $Q/apportionment-ch06-repair722/not_every_time.rs $Q/apportionment-ch06-repair722/facts.rs $Q/apportionment-ch06-repair722/oracle.py $Q/apportionment-ch06-repair722/storyboard.md
/usr/bin/python3 -I -B $Q/apportionment-ch06-build731.py inputs --job csd-apportionment-ch06-build731
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-apportionment-ch06-build731 --inputs $Q/csd-apportionment-ch06-build731-inputs.json -- /usr/bin/python3 -I -B $Q/apportionment-ch06-build731.py run --job csd-apportionment-ch06-build731
```

`inputs` prints the manifest path, its sha256, the row count (4,164 expected) and the sha256 of any binary already at `work/target/release/apportionment`. The guard command must be given exactly as above. The helper checks the following:

- the receipt's command list against its own path and arguments;
- the policy 10/8/4/56/20/10;
- the manifest hash;
- a fresh job folder holding only `command.log` and `resource-receipt.json`;
- `CARGO_TARGET_DIR=work/target`, `RUSTUP_TOOLCHAIN=nightly-2026-08-31`, `CARGO_BUILD_JOBS=2` and `RAYON_NUM_THREADS=4`.

Toolchain: `cargo-rch-real` at `cargo 1.100.0-nightly (e8cb624d5 2026-08-22)`, rustc commit `90850177…`, plus 656's two pins (wrapper `b854c942…`, rustc `49f9fa06…`). The time is not measured. 716 took about 2 minutes on a warm target. 731 compiles kit, score and the crate at new paths. Tests may take up to 4 hours and each later step up to 2 hours.

## If it stops

| Event | Meaning | Action |
|---|---|---|
| `inputs` refuses | Nothing created | Read the message. A transfer or hash problem is fixed by transferring again. A `SOURCE656` or 656-evidence refusal comes to me. |
| Guard returns 75, or a resource stop | Admission or resource limit; job folder kept | Preserve it. The same sealed helper may run again as `csd-apportionment-ch06-build731-a2`, with its own `inputs` first. |
| Test, Clippy or build failure | A source finding for CC3 | Preserve the job. A source change means new candidate hashes, so a successor helper with new names is needed, not `-a2`. |
| Facts or oracle differ from 656 | Contradicts 722's "no printed value changes" | Do not proceed. Preserve everything and send the stdout files to CC3 and me. |
| Cue or glyph probe failure | A runtime finding in a scene | As for a test failure. The files of earlier steps stay in the job folder. |

Stops are build 521's and 716's: the helper's own group, cleanup record, then SIGKILL. ffmpeg runs only in the cue probe, in its own process group, and the guard's stop covers it.

## Later, before any retained render is reused

None of these is done or assumed here:

1. **Rebind the 06_1 prediction hold.** Assembly566's `report.json`, which 664 relocated by path only, binds `docs/narration-direction-map-frederick01.json` at `d6d21eb2…`. Its `visual_holds[5]` (06_1 to 06_2) names `not_every_time.rs` at `d3e88f7b…`, line 58, marker `let q = think_here(&kit, stage)?;`. In candidate `04499441…` the same marker occurs once, at line 102. `draw.rs` (`dd385ed5…`), which holds `THINK_S` and `think`, is unchanged. On the paths these jobs use, nothing compares this entry's `sha256` with the file. `coherent_narration.py` checks only the map's own hash, and `narration_delivery.py:316-320` reads only video, cue, next cue and kind. (`narration_director.py:119-125` checks an `evidence` field, which this entry does not have.) Nothing will stop on the stale record, so the rebind has to be deliberate.
2. **93 PCM equality.** Any timeline or render with the new binary must use the 93 cue recordings 664 verified, byte-equal.
3. **45 normalized tables.** For all 15 chapters, the new binary's `timeline.tsv`, `cues.tsv` and `cuelog.tsv`, after 666's path substitution, must equal the reference, as timeline666 did for 656. 725 F2 means frames inside 06_3's tag group will differ from the old 06 even if all tables match.
4. **Runtime tools.** Before any `--narration` render, the binary needs the four tools 667 installed beside 656's topic (`coherent_narration.py`, `narration_cache.py`, `narration_delivery.py`, `storyboard.py`) under this job's `workspace/tools/`, at the hashes 667 recorded. The binary resolves them from its built topic path.
5. **Then** the new 06 final, its pictures (including 725 F2 and F3), and the full master and its verification.

## Provenance kept separate

| Chapters | Render | Binary |
|---|---|---|
| 01, 02 | local final632 renders (export 682) | Mac build, not 656 |
| 03–05, 07–15 | CSD final672 renders | `apportionment-native656` |
| old 06 | CSD final672, 4,947 frames | `apportionment-native656`, preserved, superseded only if root accepts the new 06 |
| new 06 | not rendered | `apportionment-native731` only |

`qualification731.json` records the same split under `provenance`. Because `facts.rs` and `oracle.py` are shared, the 731 binary is new for every chapter. As review 725 said, claim it only for what is rendered from it.

## What I checked

I ran a scratch stub proof (`python3 -I -B`, scratch files only). Cargo, the binary and the guard receipt were stand-ins. Everything else was real:

- **Real inputs:** the 95 workspace files at the predicted hashes, the four candidates read in place, `oracle.py` run in the job crate, and `static_cues` and `check_script` run on the real crate and script.
- **Full pass:** exit 0 with 33 tests, 443 facts equal to the 656 stand-in, 93 cues and 17 modules. The census line was present, and the job workspace and crate matched `WORKSPACE731` and `CRATE731`. The previous binary was kept, and the stand-in 656 and the candidates were unchanged afterwards.
- **Refused after launch, with no `qualification731.json`:**
  - tests: 32 tests, a renamed original test, a replaced new test;
  - Clippy failure;
  - facts: a changed fact, facts reordered (equal rows, other bytes);
  - a missing chapter, a changed script;
  - cue probe: an unsaid cue, swapped cues;
  - glyphs: a glyph failure, a missing 06 string;
  - receipt: a stale policy, a job folder that is not fresh, another command.
- **Refused at `inputs`:**
  - 656 receipt: exit 1, moved inputs;
  - 656 files: a changed workspace file, a changed engine file, the patch file, the binary;
  - 656 logs and records: 29 tests, oracle output, facts, recorded chapters;
  - the real `SOURCE656` pin on the stand-in map;
  - a changed candidate.
- **Control:** an exact copy of the candidates passed.
- **Stops:** SIGTERM and a timeout each ended the group, wrote `tests.cleanup.json` and left no grandchild.
- **Pins:** `ORIGINAL_TESTS` equals the canonical sources' 30 names, and `NEW_TESTS` equals the three the candidates add.
- **compile350:** passed, with no banned text.

Also read: build656, after-build665, assembly664, after-assembly667, timeline666, csd-final672, 716 as it ran, review 685, and the kit's CLI and glyph probe. Root's TODO (line 3896) records that on 2026-11-04 an earlier apportionment crate, which then had 19 tests, passed strict Clippy and a native build on the Mac. That day's 15-chapter glyph probe (`apportionment-opus-rebuild-glyphs01`) exited 0 with an empty `failures.txt`. So building every chapter at 1 fps is not new for this film, though that crate differs from today's.

## Not established

Nothing has compiled. The tests, Clippy and both probes have not run on these sources. The run time on the CSD is unknown. `SOURCE656` is a prediction until `inputs` runs there. I did not see 656's actual `tests.stdout`, `facts.stdout` or `qualification.json`, or `guard_csd02.py`. Glyph pages will need reading, and the cue probe's timing is estimated. No audio heard or checked. No picture, motion, fine-sync, AV or release acceptance, and no reuse decision for any retained render.
