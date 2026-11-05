# Remote final verifier, revision 03

BronzeJaguar, 2026-11-05, for BlueSalmon, bead 87do.63. This answers Mails 5452 and 5477.

**Use this helper instead of verifier02's, and keep the verifier02 folder in place on the worker.** Verifier02 is unchanged,
and verifier03 reads `anchors.json` from it.

**Nothing here has run on mac-mini-max or on a repaired master.** No SSH, job or guard launch. The checks used tiny receipt
files on this host. **The helper accepts nothing:** picture viewing, listening, fine sync and final ASR remain open.

- P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`
- O = `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01`

## The defect

The guard rewrites a job's receipt in place just after starting it. `run_job` read that receipt once, so a read landing inside
the write raised an uncaught `JSONDecodeError` and failed the job for nothing. Lavender's render rehearsal hit this race.

## The whole change

`docs/remote02-to-remote03.diff` is all of it: 3 lines out, 15 code lines in, plus docstring lines.

| Change | Detail |
|---|---|
| `live_receipt()` | Rereads a file that doesn't parse, 100 times at 0.1 s, then refuses. Same bound as Lavender's worker |
| `run_job` | The start read uses it. The semantic gate on the next line is untouched: exact script, no `exit`, `admitted` true |
| `ANCHORS` | Points at verifier02's sealed `anchors.json`, so nothing sealed is copied |
| `load_anchors()` | Pins that file's hash. It sits outside this folder's seal, so the seal now covers it through this helper |
| `import time` | Needed by the reread |

**Only a parse failure rereads.** A missing receipt still refuses at once, and a parsed receipt goes straight to the gate. A
partial write can't pass as a good receipt, because the guard's JSON object only parses once its closing brace is there.

**Unchanged:** `tools`, `check`, `write-job`, `verify-job`, every gate, `verification.json`, `VERIFY_SUCCESS`, and `verify-job`'s
stdout. That stdout is the record ASR02 reads, so the postcheck contract in Mail 5477 holds.

## Running it

Use verifier02's `HANDOFF.md` step 3 with this helper, on the worker, after the complete master:

```sh
V="$PWD/P/logarithm-remote-final-verifier03/verify_remote_final.py"
```

- **Transfer:** this folder, to the same path.
- **`O/tools01.json`:** stays as it is. Don't run `tools` again.
- **PATH:** the guard needs `sbh`, so first run
  `export PATH="$HOME/.local/bin:$HOME/.cargo/bin:/opt/homebrew/bin:/usr/bin:/bin:/usr/sbin:/sbin"`.
- **Physical paths:** run from `pwd -P`. ASR02 compares the receipt's script path exactly.
- **`--job-dir`:** `O/job-scripts`. ASR02 reads that folder.
- **Postcheck:** after the guarded run, save `verify-job`'s stdout alone to `O/verify-postcheck01.json`.
- **Job script:** `write-job` embeds this helper's hash, and the job refuses if the helper later moves. The guard's manifest
  lists this helper and verifier02's `anchors.json`.

**Root's `O/verify-after-master03.py`, as read at `1e650a5a9680`:** its interpreter, paths, `--job-dir`, `write-job` keys, guard
command and stdout-only post-check all match this helper and ASR02. **One launch need:** it doesn't set PATH, and the guard
calls `sbh` by name after making the job folder. Started without `sbh` on PATH, it uses up `verify01`, so export the PATH line
first. There is also no `--phase native` preflight, so a guard refusal needs `--verify 02`. ASR02 accepts that.

## Checks

From the repository root, with a fresh scratch path:
`python3 -I -B P/logarithm-remote-final-verifier03/checks/run_checks.py SCRATCH`. It takes about 12 seconds. The transcript is
`checks/outputs/run_checks.txt`.

| Case | Result |
|---|---|
| Verifier02 and its 10 sealed files | unchanged, by hash |
| The diff against verifier02 | exactly the listed lines |
| `anchors.json` | loads from verifier02; any other bytes refuse |
| Control: verifier02 on a partial receipt | dies with `JSONDecodeError`, so the test reproduces the defect |
| Whole receipt | no delay, passes this gate |
| Partial or empty, then valid | rereads, then passes this gate |
| Partial, then wrong script, form, `admitted` or `exit` | the semantic gate refuses |
| Permanently malformed | `REFUSED`, exit 1, after the bound |
| Missing receipt | refused at once |

"Passes this gate" means `run_job` reached its next gate, which refuses this host. Off the worker it can go no further.

## Limits

- **Verifier02's full check suite wasn't rerun.** Its code is unchanged apart from the listed lines, and its evidence keeps
  all of its stand-in limits.
- **Simulated race:** a file written partial, then replaced. The real guard wasn't run here.
- **Real worker jobs haven't run** with this helper, or with verifier02.
- **Not independent.** I wrote verifier02 and this.
