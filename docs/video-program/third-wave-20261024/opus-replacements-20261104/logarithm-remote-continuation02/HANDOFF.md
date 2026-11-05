# Logarithm remote continuation 02, mac-mini-max

For root (BlueSalmon), bead 87do.63. This replaces the failed rebind and then runs the assembly, binding, timeline and score, as continuation01 would have. Render, master, verify and ASR aren't here.

O = `EL/remote-mac-mini-max01`, S = `EL/final-picture-repair01`, sealed01 = `P/logarithm-remote-mac-mini-max01`.

## Why rebind01 failed

`logarithm-remote-rebind01` exited 1 on `ui-takes01/00_1-download01.wav` before writing any output. The coherent tool opens each selected take through the decode receipt, at `cache.digest(row['source'])` in `unit_spans`.

My sealed01 check missed this. It compared the transfer lists with the rows the helper froze, not with the files the tools open. The takes were in neither list, and the local tree has them, so every local run passed. Section B of `checks/run_checks.py` now traces real opens. Its positive control shows that the sealed01 rows alone miss exactly those 80 takes.

## Run it

1. Transfer this folder and `O/raw-source-transfer01.json` to the same paths on the remote. The 161 files that manifest lists are already there, verified by root.
2. Launch once, in tmux, from anywhere:

```sh
/bin/bash P/logarithm-remote-continuation02/remote_continuation02.sh <this bundle's SHA256SUMS hash> 8f7c4a9f45c4dd256536d8d5992bd14ebef44ac4fc80e5563f84c9de2bce5952 ff1ece5acfa8fe5f2fff52efb683ed76ac1b010330c526d46b64cdc2bfaa808c
```

Exit 0 means every step passed. Any other exit means stop, and no later step ran. There is no restart logic.

## What changes

| Item | Sealed01 | This bundle |
|---|---|---|
| Rebind job | `logarithm-remote-rebind01`, failed, preserved | `logarithm-remote-rebind02` |
| Rebind manifest | `O/rebind-inputs01.json`, preserved | `O/rebind-inputs02.json` |
| Freeze | 4,483 rows | those rows plus 166 |

The 166 rows are:

- the 161 files in `O/raw-source-transfer01.json`: 80 takes, 80 download receipts and the voice binding;
- that manifest itself;
- `remote_audio_after_build02.py` and this bundle's `SHA256SUMS`;
- the built binary, pinned to root's `ae54bc17…`;
- the repository's `.cargo/config.toml`, pinned to `79fb8766…`, as WhiteSwan asked.

`freeze()` hashes every raw row and requires the manifest to be exactly what the decode receipt names, with each take hash equal to the receipt. Downloads originals stay out, so they remain provenance only.

## What stays the same

- The names `logarithm-remote-assemble01`, `-timeline01`, `-score01` and `O/assemble-inputs01.json`, all still unused.
- `remote_pipeline01.py` and `remote_assemble01.py` run from sealed01, checked by its seal. `rebind_audio.py` is checked by its hash.
- Narration path, `O/remote-binding01.json`, `O/job-scripts` and the comparison paths.
- Root's `O/guard01.py` runs every job, so all limits are root's. ASR's 12 GiB is untouched.

`O/assemble-inputs01.json` keeps exactly one `resource-receipt.json` row for a rebind job, now rebind02's. Lavender's driver and Bronze's verifier can derive the rebind identity from that row.

## New stops before any job name is used

The helper stops, with nothing launched, if:

- this bundle or `rebind_audio.py` isn't at its hash;
- the build receipt doesn't pass;
- a job name, a manifest or a rebind output already exists;
- the real `--check` doesn't predict the four pinned hashes;
- any frozen file differs.

After the rebind, the four outputs must equal the prediction before the assembly launches.

## Evidence and limits

`checks/run_checks.py` passes under Python 3.14.6, the remote's version; see `checks/outputs/run_checks.out`. These ran on real local files: `rebind_audio.py --check`, coherent `check` and `verify`, and the render delivery contract. Each read only frozen files, plus the unit plan the rebind rewrites. The narration read was stage09's, standing in for the new assembly's byte-identical WAVs.

Root's remote `--check` already passed on the real remote tree. rebind01 also exercised the real guard and launch path as far as the admitted job's failure.

**Still unproved:** the tail of `--apply`, `--verify`, assembly, `bind`, job writing and the Rust kit have never run on this tree. The stand-ins in section C prove order and refusals only. The Rust kit's own file reads can't be traced this way. As a partial check, stage09's timeline and score manifests had 16 rows off the transfer lists, all stage09-specific, and the sealed job writer freezes none of those paths. A file the kit needs but no manifest names would first show up at runtime.

This is technical only: no listening, picture review, fine sync or independent AV acceptance. As in sealed01, a move other than the predicted +24 and +420 frames appears in `not_as_predicted` and doesn't stop the score.
