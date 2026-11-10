# Handoff 717: guarded native qualification of exponential expanded-stage08 (CC0)

cc0, 2026-11-10, 00:37 UTC. **Sealed for root to read and launch. Source only: nothing has run.** No SSH, Cargo, build, render, synthesis, launch, Beads change or commit. Native results are pending. D is `/data/projects/jmev-stroke-probe-20261106`; on the CSD, Q is `D/qualification01/production-repair17`.

**Scope: stage08 only, chapters 01 to 09.** CC1's ending integration (task 720) is a later stage. 716 is not retargeted to it: tree, binary name, chapters and counts are all bound to stage08. A later stage needs its own helper file and job names.

| File | sha256 | State |
|---|---|---|
| `exponential-build716.py` | `f9c25c382f8859efe1bf338aad57a12686c1bbb7922dbf5716ed428a797d3bfd` | Sealed. Modes `inputs`, `run`. |
| `exponential-build697.py` | `025d107beb1541c162bac0dbc4efffbc1cfb6066e9b0c2d652abf877d42dacb1` | Sealed, unchanged |
| `exponential-build-handoff698.md` | `060a8007ae11ad526bbe50a52851214e5f4a00c6ef12f7056305ac3080a2ade5` | Sealed, unchanged |

Job `csd-exponential-build697` stays as it is. Root reports that it ended exit 1 with stable inputs, 95 tests passed, and a constant assertion was the only Clippy finding. I have not seen its receipt or logs.

## Pins

Every value was recomputed here from the sealed files, not copied on trust. All of CC2's stated values agree. All 17 per-file hash rows in handoff712 that name a file in stage08 or its crate match the files.

| Pin | Value | How it was recomputed |
|---|---|---|
| `tree_sha256` | `5d6294a3a4634292c5a9cef568a4b896b8b44abe5f5a4c8575383db1a5398baf` | 716's own `listing`, on the real crate and on a separate scratch copy; stable across each measure |
| `files` | 47 | same |
| `handoff_sha256` | `f93be1cd3e353440ed301cd89b71582a5eca231356458330791c2c9dcff67e76` | sha256 of `expanded-stage08/handoff712.md`. Recorded in `qualification.json` only; that file need not be installed. |
| `lock_sha256` | `60338289fe9224d5b8321557cce27d075326fa8d467bf4ed2b12dcb23aa9139c` | sha256 of the crate's `Cargo.lock`, same as stage06 |
| `oracle_sha256` | `5143c1445e283d072c9557cd7cfb2d7f4c0e34fb40ef3d7a7fa18e7d67ed4254` | Fresh `oracle.py` run, equal to `oracle08.tsv` |
| `facts` | 1,558 | Rows and unique keys of that output: 1,104 in `oracle06.tsv` plus handoff693's 454 |
| `tests` | 155 | Static count of `#[test]` in compiled modules: 95 inherited, 4 `applications_model`, 17 `decay_scene`, 15 `circuit_scene`, 24 `applications_checks`. All 95 originals are present by name. |
| `cues`, `words` | 67, 4,128 | From `script.tsv`. Cues per chapter: 5, 6, 7, 6, 8, 9, 10, 8, 8. |

`package`, `binary` (`exponential_magic_expanded_stage08`) and the nine chapter keys are also bound. CC2's `check712.py`, which writes nothing, reports `ALL PASS` on the sealed bytes.

## What changed from 697

35 of the 38 functions in 697 are AST-identical after renumbering. That includes `own_group`, `end_group`, `on_stop`, `run_step`, `read_receipt` and `guard_binding`. Only `csd_closure`, `run` and `main` changed, and three functions are new. The Mac profile, kit and toolchain pins, and every refusal in 697 are retained.

| Change | Purpose |
|---|---|
| `STAGE`: nine chapters, stage08 binary name, handoff712's pins | Stage08 instead of stage06. |
| New `check_pins()`, called in `main` right after the job-name check | Refuses both modes if any pin is `None`. |
| New `ORIGINAL_TESTS`: the 95 test names | Derived from stage06's compiled modules (tree `af107280…`). |
| New `passed_tests()`; 4 added lines in the tests step | The count must equal the pin, and **each of the 95 must appear as passed**. Dropping or renaming one stops the job. |
| New `build697_evidence()`, called from `csd_closure` | On the CSD, requires 697's receipt to be admitted, finished, exit 1, stable and without an error or resource stop, and requires `ORIGINAL_TESTS` to equal what `tests.stdout` lists as passed. Both files are read only and bound in the manifest. |
| `qualification.json` | Adds `original_tests_passed` and `tests_sha256`; `handoff696_sha256` becomes `handoff_sha256`. |
| Names | `preflight716.json`, binary suffix `-716`, schemas and refusal messages. |

**Checks, in order:** 155 tests, including the 95 by name; Clippy `--all-targets -D warnings`; build; 1,558 facts equal to `oracle.py` by key, oracle output hash pinned; 9 chapters in order; `script` output equal to `script.tsv`, 67 cues, 4,128 words; each cue said once by literal in a compiled module; the cue probe's said order per chapter equals the script; glyph probe exit 0, empty `failures.txt`, registry and page hashes.

## Cleanup: what is and is not covered, kept from 698

The stop code is AST-identical to 697's, so 698's limits carry over unchanged. Root reports 697 stopped at Clippy, before the cue probe, so **ffmpeg has still not run under this helper line and the limit is still unproven**.

The pinned engine starts ffmpeg in **a new process group, in the same session**: asupersync `c48399f` `src/process.rs:1154` uses `POSIX_SPAWN_SETPGROUP` with `POSIX_SPAWN_SETSIGDEF`, and has no `setsid` on this path. The kit and engine have no stop-signal handler. Only the cue probe reaches ffmpeg. The glyph probe writes PNGs natively (`is_video()` is false, so no ffmpeg capability is made). The stage08 crate spawns nothing: a search of its `src` for `Command::new`, `std::process`, `.spawn(`, `posix_spawn` and `ffmpeg` found only `std::process::exit` at `main.rs:34`.

| Stop | Reaches ffmpeg? | Evidence | Status |
|---|---|---|---|
| Guard resource stop, or guard exception | **Yes.** `owned-process-stop02` (`a2fe7985`, pinned by `guard_csd02`) is a subreaper, walks `/proc` descendants and uses pidfds: TERM, KILL at 20 s, raises at 30 s if any are live. | Real CSD receipts `encoder-stop02`, on a complex binary: 960x540 in 0.42 s, 3840x2160 in 20.78 s, no survivors. | Covered. Not exercised with this binary. |
| Helper's own stop during the cue probe: 7,200 s step timeout, an exception inside the step, or a signal sent to the helper alone | **No.** `end_group()` signals only its own group. The binary dies, ffmpeg's stdin closes, and ffmpeg should end at EOF. | Guard does not sweep here: it leaves its loop when the helper exits, records -9 and frees the lane. Real CSD `encoder-stop01`, group-only stop: 960x540 passed; 3840x2160 survived 10.2 s and was gone later. Stand-ins in their own group: one reading stdin ended; one not reading it stayed alive after the helper ended. | **Limitation. Unproven at 64x36 with this binary.** |
| Helper's own stop during tests, Clippy, build, facts, oracle, chapters, script or glyphs | Not applicable: no ffmpeg in those steps. `end_group()` reaches the group. | Source read; stand-in grandchild in the group was killed on SIGTERM and on timeout, rerun for 716. | Holds for what I read. Cargo and rustc are binaries here and were not audited for groups of their own. |
| SIGKILL sent to the helper alone | No cleanup at all. | Review 530 L2, same as earlier jobs. | Known. |

A step that exits normally is not checked for leftovers (521's rule). The engine kills ffmpeg's group on its own timeouts and errors.

**If `cue-probe.cleanup.json` exists after a run**, check before admitting another job: `ps -eo pid,pgid,sid,ppid,lstart,args | grep fmn-bound-ffmpeg`. A leftover from this job shows `-video_size 64x36`. A temp folder `D/work/tmp/fmn-ffmpeg-session-<binary pid>-0` may remain; preserve it. I would expect a 64x36 encoder to be gone within seconds, but that is inference.

I did not add a fix, because root asked for the limit to be stated and any new stop code needs review. If root wants one, a successor could have `end_group()` also signal `/proc` entries whose session equals the helper's. The stand-in confirmed ffmpeg's session stays the same.

## Launch on the CSD (not run)

1. Copy `exponential-build716.py` to Q at the hash above.
2. Place the 47-file crate at any real absolute path; 716 checks the digest. Kit, score and engine need no transfer. `oracle08.tsv` is pinned by hash and need not be copied. 697 and its job folder `D/jobs/csd-exponential-build697` must remain in place.
3. Run these. A retry needs a new job name.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17; SRC=<real path>/native-crate
/usr/bin/python3 -I -B $Q/exponential-build716.py inputs --host csd --job csd-exponential-build716 --source $SRC
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-exponential-build716 --inputs $Q/csd-exponential-build716-inputs.json -- /usr/bin/python3 -I -B $Q/exponential-build716.py run --host csd --job csd-exponential-build716 --source $SRC
```

`inputs` also proves the 95 names against 697's real `tests.stdout`; a difference stops there, before any compile. The guard's exclusive lane means this waits for any running guarded job. Policy is 10/8/4/56/20/10, unchanged. Inspect any cleanup record from an earlier job first, and start one launch only.

**Dependency closure, as in 698.** Kit, score and engine are used in place and read only from finished `csd-apportionment-build656`, each at the hash its `qualification.json` recorded, with 656's receipt required to be a clean success. Six kit and score files that the parsing relies on, plus the workspace `Cargo.toml`, `.cargo/config.toml` and `rust-toolchain.toml`, are pinned to hashes I took from the repo. Nothing in 656 or 697 is written, and all of it is re-hashed at the end. Only the job's copy of the crate is edited: two path lines in `Cargo.toml` and the 19 engine source lines in `Cargo.lock`, each edit asserted to change nothing else.

## What was checked

`compile350` and the banned-string scan on both files.

With **stub** `cargo`, Clippy and binary, nothing compiled, on a scratch copy of the sealed crate, using the helper's own pins, with stand-ins for 656 and 697:

- The CSD and Mac paths pass: nine chapters, 155 tests, 95 originals, 1,558 facts, 67 cues, nine glyph modules.
- 697's receipt and test log are in the manifest, and the stand-in was unchanged by every run.
- Thirteen refusals fire after launch: wrong test count, a named line missing, an original test renamed, a repeated name, changed fact, duplicate fact, missing chapter, changed script, unsaid cue, misordered cue, glyph failure, wrong policy, non-fresh job. A wrong oracle pin stops at the oracle step.
- Eleven refusals fire before launch: 697 exit 0, 697 inputs moved, 697 one test short, 697 one test extra, tampered source, three cases of a `None` pin, a wrong tree pin, a wrong file count, a wrong lock pin.
- The real file with its real pins refuses on this Mac as `csd`: wrong host.
- SIGTERM and timeout end the helper with -9, write the cleanup record and kill a stand-in grandchild.

On the real files, read only: the pins above, `ORIGINAL_TESTS` equal to the derivation from stage06's compiled modules, and the 19 lock lines. The receipt fields `build697_evidence` reads against the guard body's source: `admitted`, `inputs_sha256`, `exit`, `final_inputs_sha256` and `stopped_for_resource_limit` are real, and `error` is written only on a guard exception, so a child exit 1 alone does not trip it.

## Limits and open evidence

- **716 has never run with real Cargo, a real binary or real CSD paths.** Root reports that 697's `inputs` passed with 4,150 bindings and that its 95 tests passed on the CSD. Build, facts, oracle, chapters, script, static, cue probe and glyphs have never run for real under this helper line.
- **handoff712 says nothing was built, compiled or run on these bytes**, so a compile, test or Clippy stop is a plausible first result. It goes to CC2, whose successor needs a new helper file (the `STAGE` block) and new job names.
- **The Clippy repairs are unproven.** `limit_model.rs:326` and `solution_model.rs:206` are now `const` assertions, `circuit_scene.rs` has a `const _` assertion, and `resistor_shape` uses `i.is_multiple_of(2)` on `usize`. handoff712 did not reproduce those findings. I have not seen 697's Clippy log, so I cannot confirm which line root reported.
- **The 95 names are derived statically** from source. The count equals 697's reported 95; the name-by-name proof happens at `inputs` on the CSD.
- **The test pin is a static count.** The real number comes from Cargo, and any difference stops the job.
- **Not seen from here:** 656's `qualification.json`, its receipt and `pristine-engine.toml`, and 697's receipt and `tests.stdout`. I read the code that wrote them; `inputs` stops on any difference.
- **Toolchain:** `csd-toolchain-check01.json` matches the wrapper and rustc pins. I have not observed `cargo-rch-real` on the CSD directly. Root's report that 697's tests ran implies it was found there. If it is absent, 716 refuses before Cargo.
- **Reuse of earlier compiled output is expected, not proven.** Time and target growth are unmeasured; the guard's 20 GiB target cap applies, and the first step may take up to 14,400 s.
- **Clippy `--all-targets` is stricter than 656's form** and follows the earlier exponential builds, so it may flag test code.
- **The oracle's output hash has not been checked on the CSD.** 697 never reached that step. Here it has the pinned hash under Python 3.15, and under 3.9.6 and 3.11.15 with `LC_ALL=C`. Its sources are exact fractions and `Decimal`, and it imports only `ceil`, `factorial` and `comb` from `math`. It prints `×`, `÷` and `→`, so a non-UTF-8 locale that is neither C nor POSIX would break it. A difference on Linux would stop the job after the build, at the oracle step.
- **The cue probe uses estimated timing at 64x36, 1 fps.** It is not picture evidence. The layout lint is not run here; it belongs with a draft render.
- **Zero glyph failures can hide fallback faces.** Root must still read the pages. handoff693 asks for close reading of `%`, `/`, `7`, `8`, `9`, `V`, `j`, `q`, and of the prime after a delimiter group in `\left(\tfrac{1}{A}\right)'=k` (08_7). handoff712 adds "multiplies", "index" and "distance at 1024 cells".
- **A later narrated render** from this binary needs `tools/` beside the topic, as 667 did.
- **Mac profile:** left as root read it and exercised with stubs only; not chosen, and no further work is planned.

No audio heard or checked. No narration was generated: ElevenLabs Frederick Surrey remains current, and handoff712 says no takes exist for the changed text. This accepts no picture, motion, sound, fine-sync, AV or release result and authorizes nothing beyond the guarded job root chooses to start. All prior files are preserved.
