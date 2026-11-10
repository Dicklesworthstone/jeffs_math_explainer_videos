# Handoff 717: guarded native qualification of exponential expanded-stage08 (CC0)

cc0, 2026-11-10, 00:19 UTC. **DRAFT. NOT READY TO LAUNCH.** CC2 has not sealed stage08, and it was still changing while this was written. Source only: no SSH, build, render, synthesis, launch, Beads change or commit.

`exponential-build716.py` is the preserved successor to 697. Every source pin in its `STAGE` block is `None`, and both modes refuse before reading anything until those pins are filled:

```
716 refused: expanded-stage08 is not sealed; unset pins: cues, facts, files, handoff_sha256, lock_sha256, oracle_sha256, tests, tree_sha256, words
```

| File | sha256 | State |
|---|---|---|
| `exponential-build716.py` | `e4c48afdf12c922b4556d036761e8c770eeb07893ff59a7094129862d8d7e08e` | Draft. This hash changes when the pins go in. |
| `exponential-build697.py` | `025d107beb1541c162bac0dbc4efffbc1cfb6066e9b0c2d652abf877d42dacb1` | Sealed, unchanged |
| `exponential-build-handoff698.md` | `060a8007ae11ad526bbe50a52851214e5f4a00c6ef12f7056305ac3080a2ade5` | Sealed, unchanged |

Job `csd-exponential-build697` stays as it is. Root reports that it ended exit 1 with stable inputs, 95 tests passed, and a constant assertion was the only Clippy finding. I have not seen its receipt or logs.

## What changed from 697

35 of the 38 functions in 697 are AST-identical after renumbering. That includes `own_group`, `end_group`, `on_stop`, `run_step`, `read_receipt` and `guard_binding`. The Mac profile, kit and toolchain pins, and every refusal in 697 are retained.

| Change | Purpose |
|---|---|
| `STAGE`: nine chapters, stage08 binary name, source pins `None` | Stage08 instead of stage06. Pins come only from the sealed handoff. |
| New `check_pins()`, first call in `main` | Refuses both modes while any pin is unset. |
| New `ORIGINAL_TESTS`: the 95 test names | Derived from stage06's compiled modules (tree `af107280…`). |
| New `passed_tests()`; 5 added lines in the tests step | The count must equal the pin, and **each of the 95 must appear as passed**. Dropping or renaming one stops the job. |
| New `build697_evidence()`, called from `csd_closure` | On the CSD, requires 697's receipt to be finished, exit 1, stable and without an error or resource stop, and requires `ORIGINAL_TESTS` to equal what `tests.stdout` lists as passed. Both files are read only and bound in the manifest. |
| `qualification.json` | Adds `original_tests_passed` and `tests_sha256`; `handoff696_sha256` becomes `handoff_sha256`. |
| Names | `preflight716.json`, binary suffix `-716`, schemas and refusal messages. |

Facts against `oracle.py`, chapters, script, static cue coverage, the cue probe and the glyph probe run unchanged, against the nine-chapter values.

## Expected values, not pins

Measured on the unsealed tree. The tree hash changed twice during this work, so none of these should be trusted until the seal.

| Item | Seen | Note |
|---|---|---|
| Oracle keys | **1,558 unique** | Fresh `oracle.py` output at about 00:15 UTC equalled `oracle08.tsv`, `5143c144…`. This matches the 1,558 root expected: 1,104 + 454. |
| Tests in compiled modules | 144, then 150 | Static count. All 95 originals were present. Moving. |
| Cues, words | 67, 4,128 | Words were 4,123 earlier. Cues per chapter: 5, 6, 7, 6, 8, 9, 10, 8, 8. |
| Files | 47 | |
| `Cargo.lock` | `60338289…` | Same as stage06. |
| Tree | `bbc2e837…` at 00:19 UTC | Moving. |

## Cleanup: limits kept from 698

The stop code is identical to 697's, so this table is unchanged in substance. 697 ended at Clippy, before the cue probe, so **ffmpeg has still not run under this helper line and the limit is still unproven**.

| Stop | Reaches ffmpeg? | Evidence | Status |
|---|---|---|---|
| Guard resource stop or guard exception | **Yes.** `owned-process-stop02` (`a2fe7985`, pinned by `guard_csd02`) is a subreaper, walks `/proc` descendants and uses pidfds. | Real CSD `encoder-stop02`, on a complex binary, no survivors. | Covered. Not exercised with this binary. |
| Helper's own stop during the cue probe: 7,200 s timeout, an exception inside the step, or a signal to the helper alone | **No.** `end_group()` signals only its own group. The pinned engine starts ffmpeg in a new group, same session (asupersync `c48399f` `src/process.rs:1154`). The binary dies, ffmpeg's stdin closes, and ffmpeg should end at EOF. | Real CSD `encoder-stop01`, group-only stop: 960x540 passed; 3840x2160 survived 10.2 s and was gone later. | **Limitation. Unproven at 64x36 with this binary.** |
| Helper's own stop during tests, Clippy, build, facts, oracle, chapters, script or glyphs | Not applicable: no ffmpeg in those steps. | Source read; stand-in grandchild killed on SIGTERM and on timeout, rerun for 716. | Holds for what I read. Cargo and rustc were not audited for groups of their own. |
| SIGKILL sent to the helper alone | No cleanup at all. | Review 530 L2. | Known. |

**If `cue-probe.cleanup.json` exists after a run**, check before admitting another job: `ps -eo pid,pgid,sid,ppid,lstart,args | grep fmn-bound-ffmpeg`. A leftover from this job shows `-video_size 64x36`. A temp folder `D/work/tmp/fmn-ffmpeg-session-<binary pid>-0` may remain; preserve it. I added no fix.

## Launch prerequisites, in order

1. **CC2 seals stage08** and names its handoff, tree hash and file count.
2. **I fill the pins and seal.** Tree, files, handoff, lock, `oracle.py` output hash, compiled-test count, fact count, cues and words, each recomputed from the sealed files and not copied on trust. I recheck that all 95 original tests are present, then give the final 716 and 717 hashes. Until then this handoff authorizes nothing.
3. **Root reads the sealed 716 and 717**, chooses the host and places the crate at a real absolute path. 697 and its job folder must remain in place on the CSD.
4. **`inputs` on the CSD** also proves the 95 names against 697's real `tests.stdout`. A difference stops there, before any compile.
5. **The guard lane is free**, and any cleanup record from an earlier job has been inspected. A retry needs a new job name.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17; SRC=<real path>/native-crate
/usr/bin/python3 -I -B $Q/exponential-build716.py inputs --host csd --job csd-exponential-build716 --source $SRC
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-exponential-build716 --inputs $Q/csd-exponential-build716-inputs.json -- /usr/bin/python3 -I -B $Q/exponential-build716.py run --host csd --job csd-exponential-build716 --source $SRC
```

## What was checked

`compile350` and the banned-string scan. Then, with **stub** `cargo`, Clippy and binary, nothing compiled, a scratch snapshot of the unsealed crate and stand-ins for 656 and 697, with stand-in pins supplied by the scratch driver:

- The CSD and Mac paths pass: nine chapters, 1,558 facts, 67 cues, 95 originals.
- 697's receipt and test log are in the manifest.
- Thirteen refusals fire after launch: wrong test count, a named line missing, an original test renamed, a repeated name, changed fact, duplicate fact, missing chapter, changed script, unsaid cue, misordered cue, glyph failure, wrong policy, non-fresh job.
- Six refusals fire before launch: 697 exit 0, 697 inputs moved, 697 one test short, 697 one test extra, tampered source, unset pins (on the real file).
- SIGTERM and timeout end the helper with -9, write the cleanup record and kill a stand-in grandchild.

Read on the real files, without changing them: script ownership, static cue coverage over the nine compiled chapter modules, the 19-line lock edit, and `ORIGINAL_TESTS` equal to the derivation from stage06's compiled modules.

## Limits and open evidence

- **716 has never run with real Cargo, a real binary or real CSD paths.** Root reports that 697 passed preflight, closure, the job copy and 95 real tests on the CSD. Build, facts, oracle, chapters, script, static, cue probe and glyphs have never run for real under this helper line.
- **The 95 names are derived statically** from source. The count equals 697's reported 95; the name-by-name proof happens at `inputs` on the CSD.
- **No stage08 Rust has been compiled**, so a compile, test or Clippy stop is a plausible first result.
- **Stage08's oracle prints non-ASCII values** (`→`, `×`). The helper compares bytes under the CSD's locale. I expect UTF-8, but that is inference.
- **Reuse of earlier compiled output is expected, not measured.** The first step may take up to 14,400 s; the guard's 20 GiB target cap applies.
- **Toolchain:** `cargo-rch-real` is expected on the CSD; 697 got past the toolchain check by root's report.
- **The cue probe uses estimated timing at 64x36, 1 fps.** It is not picture evidence.
- **Zero glyph failures can hide fallback faces.** Root must still read the pages. `%`, `/`, `7`, `8`, `9`, `V`, `j`, `q` are the handoff693 glyph worries.
- **A later narrated render** from this binary needs `tools/` beside the topic, as 667 did.

No audio heard or checked. No narration was generated: ElevenLabs Frederick Surrey remains current. This accepts no picture, motion, sound, fine-sync, AV or release result and authorizes nothing. All prior files are preserved.
