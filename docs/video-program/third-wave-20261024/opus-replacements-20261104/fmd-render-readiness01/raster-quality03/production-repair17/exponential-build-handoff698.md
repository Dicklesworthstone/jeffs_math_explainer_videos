# Handoff 698: guarded native qualification of exponential expanded-stage06 (CC0)

cc0, 2026-11-09, 23:52 UTC. Source only. **Nothing has run:** no SSH, build, render, synthesis, launch, Beads change or commit. Root chose the CSD for the first build and launches it. D is `/data/projects/jmev-stroke-probe-20261106`; on the CSD, Q is `D/qualification01/production-repair17`.

| | sha256 | |
|---|---|---|
| `exponential-build697.py` | `025d107beb1541c162bac0dbc4efffbc1cfb6066e9b0c2d652abf877d42dacb1` | modes `inputs`, `run` |
| Bound crate: `expanded-stage06/native-crate` | `af10728022b856385ceb22055abb7c38dd3acbd0284470d37cb499753edf2d89` | 43 files; recomputed here, equals CC1's value |
| `handoff696.md` | `377ae52a242bcc8e0d4a43c1c5f81da4446c287b87f98c9abd697c04cbf3736b` | recomputed |
| `oracle06.tsv` | `8b0f35cfe6a32fad85c67357fed190a004c6ed247358e7e031fa02daf3fa5da0` | a fresh `oracle.py` run here has this hash |

**The only change since root read 697** is one docstring passage, which claimed a stop reaches every process. That was wrong for ffmpeg (next section). The code is AST-identical to the version root read (`90a16ab2…`, copy kept in my scratch folder).

## Cleanup: what is and is not covered

The pinned engine starts ffmpeg in **a new process group, in the same session**: asupersync `c48399f` `src/process.rs:1154` uses `POSIX_SPAWN_SETPGROUP` with `POSIX_SPAWN_SETSIGDEF`, and has no `setsid` on this path. The kit and engine have no stop-signal handler. Only the cue probe reaches ffmpeg. The glyph probe writes PNGs natively (`is_video()` is false, so no ffmpeg capability is made), and the stage06 crate spawns nothing.

| Stop | Reaches ffmpeg? | Evidence | Status |
|---|---|---|---|
| Guard resource stop, or guard exception | **Yes.** `owned-process-stop02` (`a2fe7985`, pinned by `guard_csd02`) is a subreaper, walks `/proc` descendants and uses pidfds: TERM, KILL at 20 s, raises at 30 s if any are live. | Real CSD receipts `encoder-stop02`, on a complex binary: 960x540 in 0.42 s, 3840x2160 in 20.78 s, no survivors. | Covered. Not exercised with this binary. |
| Helper's own stop during the cue probe: 7,200 s step timeout, an exception inside the step, or a signal sent to the helper alone | **No.** `end_group()` signals only its own group. The binary dies, ffmpeg's stdin closes, and ffmpeg should end at EOF. | Guard does not sweep here: it leaves its loop when the helper exits, records -9 and frees the lane. Real CSD `encoder-stop01`, group-only stop: 960x540 passed; 3840x2160 survived 10.2 s and was gone later. Stand-ins in their own group: one reading stdin ended; one not reading it stayed alive after the helper ended. | **Limitation. Unproven at 64x36 with this binary.** |
| Helper's own stop during tests, Clippy, build, facts, oracle, chapters, script or glyphs | Not applicable: no ffmpeg in those steps. `end_group()` reaches the group. | Source read; stand-in grandchild in the group was killed on SIGTERM and on timeout. | Holds for what I read. Cargo and rustc are binaries here and were not audited for groups of their own. |
| SIGKILL sent to the helper alone | No cleanup at all. | Review 530 L2, same as earlier jobs. | Known. |

A step that exits normally is not checked for leftovers (521's rule). The engine kills ffmpeg's group on its own timeouts and errors.

**If `cue-probe.cleanup.json` exists after a run**, check before admitting another job: `ps -eo pid,pgid,sid,ppid,lstart,args | grep fmn-bound-ffmpeg`. A leftover from this job shows `-video_size 64x36`. A temp folder `D/work/tmp/fmn-ffmpeg-session-<binary pid>-0` may remain; preserve it. I would expect a 64x36 encoder to be gone within seconds, but that is inference.

I did not add a fix, because root asked for the limit to be stated and any new stop code needs review. If root wants one, a successor could have `end_group()` also signal `/proc` entries whose session equals the helper's. The stand-in confirmed ffmpeg's session stays the same.

## Launch on the CSD (not run)

1. Copy `exponential-build697.py` to Q at the hash above.
2. Place the 43-file crate at any real absolute path; 697 checks the digest. Kit, score and engine need no transfer. `oracle06.tsv` is pinned by hash and need not be copied.
3. Run these. A retry needs a new job name.

```
D=/data/projects/jmev-stroke-probe-20261106; Q=$D/qualification01/production-repair17; SRC=<real path>/native-crate
/usr/bin/python3 -I -B $Q/exponential-build697.py inputs --host csd --job csd-exponential-build697 --source $SRC
/usr/bin/python3 -I -B $D/guard_csd02.py --phase native --job csd-exponential-build697 --inputs $Q/csd-exponential-build697-inputs.json -- /usr/bin/python3 -I -B $Q/exponential-build697.py run --host csd --job csd-exponential-build697 --source $SRC
```

The guard's exclusive lane means this waits for any running guarded job. Policy is 10/8/4/56/20/10, unchanged.

**Dependency closure.** Kit, score and engine are used in place and read only from finished `csd-apportionment-build656`, each at the hash its `qualification.json` recorded, with 656's receipt required to be a clean success. Six kit and score files that 697's parsing relies on, plus the workspace `Cargo.toml`, `.cargo/config.toml` and `rust-toolchain.toml`, are pinned to hashes I took from the repo; the kit and score pins also equal export682. Nothing in 656 is written, and all of it is re-hashed at the end. Only the job's copy of the crate is edited: two path lines in `Cargo.toml` and the 19 engine source lines in `Cargo.lock`, each edit asserted to change nothing else.

**Checks, in order:** 95 tests; Clippy `--all-targets -D warnings`; build; 1,104 facts equal to `oracle.py` by key, oracle output hash pinned; 7 chapters in order; `script` output equal to `script.tsv`, 51 cues, 3,153 words; each cue said once by literal in a compiled module; the cue probe's said order per chapter equals the script; glyph probe exit 0, empty `failures.txt`, registry and page hashes.

## What was checked

`compile350` and the banned-string scan. On the real files, read only: stage pins, fresh oracle hash, 51 cues in `script.tsv` matched statically, 95 authored tests, the lock edit (19 lines removed) and the patch text (identical for the exponential and apportionment locks).

End to end with **stub** `cargo`, Clippy and binary, nothing compiled, and a stand-in 656 made from local files:

- The CSD path passes, with the Cargo arguments as intended.
- Eleven refusals fire: wrong test count, changed or duplicated fact, missing chapter, changed script, unsaid or misordered cue, glyph failure, wrong policy, non-fresh job, tampered source.
- SIGTERM and timeout end the helper with -9 and write the cleanup record.

`sha`, `own_group`, `end_group`, `on_stop`, `cargo_home`, `env_problems`, `text` and `read_receipt` are AST-identical to mini 521 after normalizing job numbers.

## Limits and open evidence

- **Never executed with real Cargo, a real binary or real CSD paths.** handoff696 says stage05 and stage06 Rust have never compiled, so a compile, test or Clippy stop is a plausible first result. It goes to CC1, whose successor needs a new helper file (the `STAGE` block) and new job names.
- **Not seen from here:** 656's `qualification.json`, its receipt and `pristine-engine.toml`. I read the code that wrote them; `inputs` stops on any difference.
- **Toolchain:** `csd-toolchain-check01.json` matches the wrapper and rustc pins. `cargo-rch-real` was not observed on the CSD. The Mac wrapper has the same hash and names it. If it is absent, 697 refuses before Cargo.
- **Reuse of 656's compiled output is expected, not proven.** Time and target growth are unmeasured; the guard's 20 GiB target cap applies, and the first step may take up to 14,400 s.
- **Clippy `--all-targets` is stricter than 656's form** and follows the earlier exponential builds, so it may flag test code.
- **The cue probe uses estimated timing at 64x36, 1 fps.** It is not picture evidence. The layout lint on CC1's list is not run here; it belongs with a draft render.
- **Zero glyph failures can hide fallback faces.** Root must still read the pages.
- **A later narrated render** from this binary needs `tools/` beside the topic, as 667 did.
- **Mac profile:** left as root read it and exercised with stubs only; not chosen, and no further work is planned.

No audio heard or checked. No narration was generated: ElevenLabs Frederick Surrey remains current, and handoff696 says no takes exist for this text. This accepts no picture, motion, sound, fine-sync, AV or release result and authorises nothing. All prior files are preserved; 686, 687, 689 and root's 700 to 702 are untouched.
