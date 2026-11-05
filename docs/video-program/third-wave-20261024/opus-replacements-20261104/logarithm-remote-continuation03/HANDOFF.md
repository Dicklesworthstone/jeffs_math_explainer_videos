# Logarithm remote continuation 03, mac-mini-max

For root (BlueSalmon), bead 87do.63. This starts at bind, after `logarithm-remote-rebind02` and `logarithm-remote-assemble01` passed, and then runs timeline and score. It never runs a rebind, assembly or build. Render, master, verify and ASR aren't here.

O = `EL/remote-mac-mini-max01`, S = `EL/final-picture-repair01`, sealed01 = `P/logarithm-remote-mac-mini-max01`.

## Run it

1. Transfer this folder to the same path on the remote. Nothing else is needed.
2. Launch once, in tmux, through your start wrapper:

```sh
/bin/bash P/logarithm-remote-continuation03/remote_continuation03.sh <this bundle's SHA256SUMS hash> 8f7c4a9f45c4dd256536d8d5992bd14ebef44ac4fc80e5563f84c9de2bce5952 ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c
```

Exit 0 means every step passed. Any other exit means stop, and no later step ran. `bind` writes only after every gate passes, so a refusal there spends no name and writes nothing.

## Why bind refused

`gates()` searched the binary for the crate path, followed by a lookahead meant to reject longer folder names. Rust string literals have no terminator, so the path can sit directly against the next literal.

In the stage09 binary it does: `…/logarithm_mysteries` is followed by `πi0−πi loop`. That passed only because `π` isn't ASCII. An ASCII neighbour would make the search refuse a correct binary. **This is the likely cause, not a confirmed one**, because the remote binary isn't here. A plain byte count of the crate path in that binary would settle it, and nothing below depends on the answer.

## The path is a real input

`main.rs:30` passes `env!("CARGO_MANIFEST_DIR")` to the kit as the topic. The kit reads `script.tsv` from it and finds `tools/narration_delivery.py` from it. A binary compiled for another path would refuse every narrated job, so paperwork alone wouldn't replace this gate.

## What replaces the search

`remote_pipeline03.py` loads sealed `remote_pipeline01.py` by hash. Sealed `gates()` runs unchanged except for that one call, which is replaced in memory.

The replacement refuses unless all of these hold:

1. The binary's bytes are `ae54bc17…6136`.
2. `O/target/release/logarithm_mysteries` has the same hash.
3. The build receipt passes, as phase `native`, with the command exactly `/bin/bash O/build01.sh`.
4. `build01.sh` is at its pinned hash, with `set -euo pipefail`, both clean-start tests, the exact `--manifest-path "$C/Cargo.toml"` build line, the copy and the `cmp`. Its `C` and `O` resolve to this crate and lane.
5. The build's frozen manifest binds that script, `Cargo.toml`, `Cargo.lock` and the engine budget. Sealed `gates()` has already required every crate source in it, unmoved.
6. The assembly's frozen manifest binds this binary.
7. **The real binary, run to its first narrated call.** `python3` is a stub that records the call and refuses. The call must equal the sealed delivery contract exactly: tool path, `--topic`, narration, gain and all 13 chapters. It's reached only if `script.tsv` at the compiled topic matches the compiled script byte for byte. Nothing renders or is written.

Every other gate is the sealed code: tools, integration, audio identity, holds, facts, oracle, chapters, coherent verify, delivery contract and prior jobs.

## Binding

The file, the schema `logarithm-remote-binding-v1` and every consumer path stay the same. Three things differ:

- `remote_helper_sha256` is now this helper's.
- `files` gains this helper and this bundle's `SHA256SUMS`.
- `checks` gains `compiled_build`, the proof above.

## Cross-check

Only sealed `remote_pipeline01.py:91` runs this search on the remote path. Lavender's `render_master_remote.py`, Bronze's `verify_remote_final.py` and root's `O/asr01.py` don't use it. V4 `check()` has a similar search, but nothing on this path calls it.

## Evidence and limits

`checks/run_checks.py` passes under Python 3.14.6; see `checks/outputs/run_checks.out`. It uses root's real `build01.sh` and manifests. It runs the probe on a real Rust binary, stage09's: accepted for its own crate, refused for the repaired crate.

**Not run:** the remote binary, and every gate after this one. Those gates, `step`, the job template and the kit have never run on this tree. The remaining sealed gates were read for path traps, and none was found, but reading is not a run. Section C uses tiny files for the receipt, target and binary.

This is technical only: no listening, picture review, fine sync or independent AV acceptance. As before, a move other than the predicted +24 and +420 frames appears in `not_as_predicted` and doesn't stop the score.
