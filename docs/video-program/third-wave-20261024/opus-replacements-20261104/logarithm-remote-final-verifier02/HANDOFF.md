# Remote final verifier, revision 02

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. Source and helper preparation only.

**This replaces `logarithm-remote-final-verifier01`, so use this folder.** Verifier01 required the failed
`logarithm-remote-rebind01` and would have refused the good film. It was also **never sealed**: any note calling it sealed
is wrong. It stays byte for byte, and `docs/verifier01-as-left.sha256` records it.

**Nothing here has run on mac-mini-max or on a repaired master.** No SSH, no job, no render, decode, ASR, build or
synthesis. No Beads, TODO, memory or production files were touched. All check groups pass, but they run on this local
host against stage09 records and stand-ins, so they validate helper behaviour only. **The helper accepts nothing; real
final media review remains open.**

## Paths

| Short | Path |
|---|---|
| R | repository root, the working directory |
| P | `docs/video-program/third-wave-20261024/opus-replacements-20261104` |
| O | `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01` |
| S | `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/final-picture-repair01` |

## What changed from verifier01

`docs/remote01-to-remote02.diff` is the whole change, and G1b proves it matches the helper. All 636 anchors, media checks,
tool pins, the hold and closing gates, and `receipt_ok`, `ran`, `check_binding` and `write_tools` are unchanged.

| What | Verifier01 | Now |
|---|---|---|
| Rebind job | fixed name from `--series`, so the failed job refuses | read by hash from the binding: see below |
| Rebind gates | `receipt_ok` and order | the same, plus the receipt must show `rebind_audio.py --apply` in phase `native`, and the master must have frozen that receipt |
| Binary | binding against master | also against the assembly manifest's frozen row |
| Master retry | not possible | optional `--master-job`, limited to `logarithm-remote-master01(-r[2-9])?` |
| `verification.json` | | adds `rebind_job`, `rebind_receipt_sha256`; every earlier key stays |
| Verify job inputs | | adds the rebind receipt and the assembly manifest |

### How the rebind is identified

There is no CLI argument and no hardcoded job name. The binding already passes the identity by hash:

1. The binding's `files` must hold exactly one row whose hash equals the assembly receipt's `inputs_sha256`.
2. That manifest must hold exactly one row named `<lane>/logarithm-remote-rebindNN/resource-receipt.json`.
3. That row's hash must equal the receipt on disk.
4. That receipt must pass every gate above, and build, rebind, assembly must run in that order.

The failed `rebind01` stays on the lane and is never read. G8 proves this by making its receipt unreadable. Ruby confirmed
no binding key or schema change (Mail 5355), and Lavender's driver derives the same job the same way.

### Root's cross-check of the path-string heuristic (Mail 5359)

The verifier has no such test, and neither has Lavender's driver or worker. `mentions` appears only in
`remote_pipeline01.py:91` and the sealed `prepare_final_picture_pipeline.py:270`, which that first file reuses. The
verifier only hashes the binary. Its source link is the build's guard manifest: every changed source must be a row there
at its integrated hash, found through the hash in the build receipt.

## What root does on the worker

**The verifier can't run until a binding exists.** Root reports rebind02 and assemble01 passed, and `bind` then refused on
that heuristic, so no binding, timeline or score exists yet. Check 2 below needs them, plus the master.

**1. Transfer** this folder to the same repository-relative path. The two extra files are already there:
`O/verifier-support-transfer01.json` records both with matching hashes.

**2. Measure the tools, once,** before any render, since Lavender's master reads the same file. `tools` is the only
writer of `O/tools01.json`. If verifier01's `tools` already wrote it, keep that file: the code is identical (G1b).

```sh
/opt/homebrew/bin/python3 -I -B P/logarithm-remote-final-verifier02/verify_remote_final.py tools \
  --out "$PWD/O/tools01.json" \
  --ffmpeg /opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffmpeg --ffprobe /opt/homebrew/Cellar/ffmpeg/8.1.2_1/bin/ffprobe
```

It refuses off mac-mini-max, refuses an existing file, and refuses a Pillow older than 10.1.

**3. After the master passes,** from R on the worker. Replace P, O and S with the paths above; `set --` works in both
zsh and bash. If Lavender's driver used a master retry, add `--master-job logarithm-remote-master01-rN` to the `set --`
line, and see the first trap below.

```sh
V="$PWD/P/logarithm-remote-final-verifier02/verify_remote_final.py"
PY=/opt/homebrew/bin/python3
set -- --lane "$PWD/O/jobs" --jobs "$PWD/O/render-jobs01" --series 01 \
  --crate "$PWD/S/workspace/explainer_corpus/logarithm_mysteries" \
  --narration "$PWD/O/jobs/logarithm-remote-assemble01/narration01" \
  --integration "$PWD/S/integration.json" --binding "$PWD/O/remote-binding01.json" --tools "$PWD/O/tools01.json"

$PY -I -B "$V" check "$@"                                                     # read only, every gate
$PY -I -B "$V" write-job "$@" --verify 01 --job-dir "$PWD/O/job-scripts"      # prints the guard line
# root launches the printed line, from R, on the worker
$PY -I -B "$V" verify-job "$@" --verify 01 --job-dir "$PWD/O/job-scripts"     # after the guarded run
```

`check` prints the `rebind_job` it derived. Expect `logarithm-remote-rebind02`. `write-job` refuses an existing script,
inputs file or job folder, so a second attempt needs `--verify 02`. `verification.json` lands at
`O/jobs/logarithm-remote-verify01/verification.json`, where `asr01.py` reads it.

**4. Fetch for review.** Viewing happens off the worker. `verify-job` prints `verification_sha256`, and
`verification.json` holds `sheet_sha256` for every sheet, so compare the fetched files against those. The sheets are
`sheet-NNN.jpg` in the job folder.

## Contract

| Item | Value | Confirmed by |
|---|---|---|
| Lane, guard | `O/jobs`, `O/guard01.py` | root, Mail 5282 |
| Build | `logarithm-remote-build01`, `O/logarithm_mysteries-release01`, `O/build-inputs01.json` | root; real manifest read |
| Rebind | `logarithm-remote-rebind02`, derived, never named here | Ruby, Mail 5355; root, Mail 5359 reports it passed |
| Assembly | `logarithm-remote-assemble01`, `O/assemble-inputs01.json` | Ruby, Mail 5355; root, Mail 5359 reports it passed |
| Narration | `O/jobs/logarithm-remote-assemble01/narration01` | Ruby, Mail 5297; root, Mail 5295 |
| Binding | `O/remote-binding01.json`, `logarithm-remote-binding-v1`, keys as read from Ruby's `gates()` | Ruby, Mail 5355, for continuation02 |
| Timeline, score | `logarithm-remote-timeline01`, `-score01`, success lines unchanged | Ruby, Mail 5297; G12 |
| Comparisons | beside the binding, in `O` | Ruby, sealed source read |
| Render, master | `logarithm-remote-render-<chapter>-01` and `logarithm-remote-master01`, each with `(-r[2-9])?` | Lavender, Mails 5296 and 5314 |
| Render work | `O/jobs/logarithm-remote-render-work01` | Lavender, Mail 5296 |
| Job scripts | `O/render-jobs01`, sidecar config line unchanged | Lavender, Mail 5296; G12 |
| Tools file | `O/tools01.json`, written only by `tools` | root, Mail 5290; Lavender, Mail 5296 |
| Verify job | `logarithm-remote-verify01` | root; matches `asr01.py` |

**If Ruby's bind repair changes the binding,** this needs revisiting. The helper reads `schema`, `approved`, `build`,
`audio`, `cues`, `chapters`, `crate`, `direction` and `files`, and `files` must hold both guard manifests.

## Traps

- **A master retry passes here and then stops at ASR.** `O/asr01.py` asserts `master_job == 'logarithm-remote-master01'`.
  That fails closed, but ASR would need a new script. The ASR addendum covers this.
- **Run `tools` once, before render.** If it is rerun with different bytes, `tools_sha256` differs from the master's and
  `check` refuses, correctly.
- **A Homebrew change on the worker** refuses at `check` or job start, because tools are pinned by hash. Re-measure and
  re-master rather than patch.
- **Any crate edit after binding** refuses, because `check` compares the binding's crate tree.
- **`check` re-hashes every master row,** including the 4K chapters, so expect minutes.
- **An anchor crossing its next cue refuses by design.** Fix `anchors.json` in a new path rather than widening the check.
- **Take this folder from the working tree.** Ruby reports a commit swept an early state of their folder, so don't assume
  HEAD holds this one. Compare it against `SHA256SUMS`.

## Checks

From R, on this host, with a fresh scratch path:
`python3 -I -B P/logarithm-remote-final-verifier02/checks/run_checks.py SCRATCH`. The transcript is
`checks/outputs/run_checks.txt`.

| Group | Evidence kind | Shows |
|---|---|---|
| G1 | sealed verifier against this one, by AST | exact changes; sealed bundle intact; anchors byte-identical |
| G1b | verifier01 against this one, by AST | exact revision; verifier01 unchanged |
| G2 | real stage09 records | all 1356 sealed pictures reproduced, reasons and order included |
| G3 | real stage09 records | all 494 flagged frames come back from their bases |
| G4 | real stage09 records | every old frame and reason kept, anchor first |
| G5 | **synthetic** shifted schedule | anchors move with it; `was` changes nothing |
| G6 | planted | 15 resolver faults refused |
| G7 | real holds, then planted | 15 holds; 18 hold and closing faults refused |
| G8 | **stand-in** remote-layout fixture | a failed rebind01 beside a good rebind02: this passes, verifier01 refuses; each planted fault stops at its expected gate |
| G8b | **stand-in** | a master retry passes where the base job failed; other names refused |
| G9a | **stand-in** | `check`, `write-job`, repeat and early `verify-job`, through the real `main` |
| G9b | **stand-in** tools, 20 blank pictures | `run_job` and `verify-job` flow; root's real `bound_proof` accepts the output |
| G10 | delivered files | no local paths |
| G11 | root's real local records | build manifest pins all 6 sources; the 2 extra files arrived |
| G12 | Ruby's sealed and Lavender's unsealed source text | names, success lines, config line and keys agree; no peer code run |

The G8 plants are consistent forgeries: everything downstream is re-hashed, so only the planted fact can decide.

## Limits

- **No real rebind data was read here.** Root's report of rebind02 is a mail, and `O/assemble-inputs01.json` isn't on this
  host. The fixture copies its shape from Ruby's sealed source.
- **Lavender's bundle wasn't sealed when I checked.** G12 records the hashes I read. The fixture's master script is still
  root's older `S/render-master01.py` worker text, not Lavender's. G12 checks Lavender's config line and keys as text.
- **Stand-ins throughout.** The worker's Python 3.14, ffmpeg 8.1.2 and Pillow 12.1.1 were never exercised here; these
  checks ran under local Python 3.15. The media code is the sealed verifier's, which never ran on a repaired master, but
  the same steps did run on the stage09 master.
- **Closing timing is unmeasured:** 330 frames of hold, then exactly 90 of fade, from the kit source. A surprise refuses
  with the measured numbers.
- **Passing proves technical facts only.** No frame was viewed or sound heard. Independent picture review, listening and
  final ASR remain open.
- **Not independent.** I wrote the O1, O2, O3 repairs, reviewed sheets 113–225, and wrote this helper's parents. An
  independent reader should check it.
- **No Rust was compiled for this work.**
