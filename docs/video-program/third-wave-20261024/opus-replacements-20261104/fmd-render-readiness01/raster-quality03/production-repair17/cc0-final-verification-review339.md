# Review 339: the 330 → 339 delta (CC2)

cc0, 2026-11-08. This is an independent review of CC2's frozen 339, which fixes review 332's M1 (overview outputs not checked against their recorded sha256). It covers only the 330 → 339 delta. It is checked against 332's M1 cases and against root's probe342 for 332's L1.

I edited no CC2, CC1 or root file, and did not edit 330, 339, 332 or any earlier review. I ran nothing on CSD, repeated no probe, launched no guarded job, wrote no permit and made no Beads, TODO or canonical edit. All local work ran on fixtures outside the repo. **No audio heard or checked.**

D is the CSD root and Q is `D/qualification01/production-repair17` (locally, this folder). `:N` is a line of `csd-final-verify339.py` at `1e200220`.

## Files reviewed (frozen; hashes confirmed with CC2's handoff and on disk)

| File | sha256 |
|---|---|
| `Q/csd-final-verify339.py` (979 lines) | `1e200220a1a7fd794abe318c0a1d815c188976f57c0707902c71915345dc05af` |
| `Q/cc2-final-verify339-notes.md` (132 lines) | `60d97ad6cf3d6aaa5319d202fd83a13f2ce568e57dd366fb163e3ee32fa35e4f` |
| `Q/csd-final-verify330.py`, the delta base, unchanged | `4fc4594d4dccd58e407824fd7ccb59a5b786ba9c8c85dd38603916b669749437` |
| `Q/cc2-final-verify330-notes.md`, unchanged | `9e11d3964897079a7b86f32425208c11df427eddbb9c723c8a0d24ad0513e781` |
| `Q/cc0-final-verification-review332.md`, mine, unchanged | `75cc0fa1b370ece5a50d0c9531b31a6b205c9a4bba5b831da8b5992a503fef88` |
| `Q/root-csd-framemd5-probe342.json`, root's | `c3e833dcbc9f613b997f58b93a946eec288833b7bee559e7e83c6181d3cbaa1b` |
| `Q/csd-final-verify321.py`, unchanged | `0a956ec8c0613971ad33e90a0a9eb4a496ba42bd2cb06d8044105397bfc5b7fb` |
| `Q/cc0-combined-master318.py`, the contract, unchanged | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` |

## Result

**No High or Medium. One Low (L1) and three notes. 332 M1 is fixed for every case where the folder is changed but `sampling.json` is not (C1–C6, plus C6b and C13, added here). 332 L1 is resolved by root's probe342.**

- **What 339 changes.**
  - The AST diff against 330 changes only `part_done`, `pictures` and `main`. No function was added or removed.
  - Only five constants changed: `VERSION`, `VERIFY`, `VERIFY_INPUTS`, `PICTURES` and `PICTURES_INPUTS`.
  - The 16 functions that read 318's records and the guard records are still AST-identical to 321's. The 318 contract is unchanged.
- **Regression.** My 48 regression cases give 47/48 on 339, the same as on 330. The 48th is W3, still a note (332 N1).
- **The C7 anchor.** A consistent forgery (C7) is still accepted inside the folder, as CC2 states, and the anchor detects it. But the overview job's printed line, which 339 calls "the anchor outside this folder", lands inside the overview job folder, and the native launch binds it at its current bytes (L1).

## 1. 332 M1: run through 339's own launch and job code

**Method.** The harness loads frozen 339, and frozen 330 as the control, and points both at a fixture root.
- It drives `main()` in `launch-pictures` mode twice: overview, then native.
- It uses each file's own `launch`, `part_done` and `pictures` code.
- The guard is replaced by a stand-in with the behaviour the CSD receipts show:
  - an admission re-hash of every bound row;
  - the child run, with its stdout written to `<job>/native.stdout`;
  - a re-hash of every row at the end.
- Pillow, the JPEG and PNG writes and the sheets are real. The ffmpeg extraction runs locally on a 24-frame fixture master.

Each change below is made after the overview job's receipt is final and before the native launch, unless stated otherwise.

**Columns.** "Spent" lists what existed afterwards: the native job folder and the native inputs file. "Anchor" lists which of the four values below differ from root's ledger:
- **ledger:** the overview `sampling.json` sha256, taken by the harness at overview success and kept outside the folder;
- **line:** the overview job's last line in its `native.stdout` (:907–909);
- **printed:** the native launch's printed `overview_sampling_sha256` (:972);
- **row:** the native inputs file's row for the overview `sampling.json`;
- **union:** `union.overview_sampling_sha256` in the native `sampling.json` (:860).

| Case | Change | 330 (control) | 339 |
|---|---|---|---|
| C0 | none | accepts | **accepts**; union exact; anchor: none differs |
| C1 | sample 5 picture: one byte XOR 0xFF | accepts | **refused** at native launch, `changed since the overview job recorded it: frame-000005.jpg`; spent nothing |
| C2 | sample 3 overwritten with sample 4's JPG | accepts | **refused** (`frame-000003.jpg`); spent nothing |
| C3 | sample 2 truncated to 500 bytes | accepts | **refused** (`frame-000002.jpg`); spent nothing |
| C4 | sample 7 deleted | accepts | **refused**, `the overview pictures are not the ones it recorded`; spent nothing |
| C5 | `sheet-0002.jpg`: one byte flipped | accepts | **refused** (`sheet-0002.jpg`); spent nothing |
| C6 | extra `pictures/frame-000099.jpg` | accepts | **refused**, `the overview pictures are not the ones it recorded`; spent nothing |
| C6b | extra `sheet-9999.jpg` | accepts | **refused**, `the overview sheets are not the ones it recorded`; spent nothing |
| C13 | an empty folder `pictures/extra` | accepts | **refused**, `the overview pictures are not the ones it recorded`; spent nothing |
| C10 | C1, then the exact bytes restored and a relaunch | accepts | **refused first (spent nothing), accepted after the restore**; union exact |
| C7 | sample 1 flipped and its `sha256` rewritten in `sampling.json` | accepts | **accepted**; anchor: printed, row and union differ, line matches |
| C8 | sample 5 flipped after the native inputs are written | refused | **refused** at guard admission (the stand-in's bound-row re-hash), naming the overview picture |
| C9 | one sample's reasons changed in `sampling.json` | refused | **refused** by the union check (`overview samples differ from the plan`); native folder and inputs spent, as in 330 |
| C11 | read-only `part_done('overview')` and `part_done('native')` after both jobs, then one native PNG flipped | n/a | **both re-hash clean; then refused**, `changed since the native job recorded it: frame-000001.png` |
| C14 | C7, plus the overview job's last line rewritten to the forged hash | accepts | **accepted**; anchor: line, printed, row and union all differ from the ledger, but agree with each other |
| C15 | C7, made after `main()` prints the anchor and before `launch()` binds | n/a | **accepted**; anchor: row and union differ, line and printed match |

**The tally.** 29 runs: 16 on 339 and 13 on 330.
- On 339, every case behaves as 339's design and notes state. That includes C7: accepted and detected.
- On 330, C1–C7, C6b and C13 are all accepted, which reproduces 332 M1.

**What this shows.**
- `part_done()` (:808–827) now refuses any change, deletion or addition to the overview pictures or sheets that is not reflected in `sampling.json`. It names the changed file.
- It runs before the native inputs are written (:971, then `launch()` at :974–975), so a refusal spends nothing (C1–C6b, C13). It also runs again inside the native job.
- Restoring the exact bytes is enough to relaunch (C10), as the 339 notes §5 retry rule says.
- The recorded paths are written from the same `PICTURES % part` path that `part_done` reads (:875, :880, :891, :794), so the exact-set check is consistent on CSD too. A path-form mismatch could only make it refuse, which is the safe direction.
- C8 and C9, the 330 protections, are unchanged.

## 2. Findings

### L1 (Low). The anchor line is inside the folder it anchors

**Where.**
- :907 comments the overview job's last line as "the anchor outside this folder (CC0 332 C7)".
- Notes §1 and §5 step 3 tell root to keep that line.

**What happens.** A guarded child's stdout is written to `<job>/native.stdout`, inside the job folder.
- **Evidence.** The CSD guarded job folder fetched into Q, `preview274-review305/`, holds `native.stdout` with the job's printed lines. Its `resource-receipt.json` has the keys admission, admitted, command, exit, final_inputs_sha256, finished, inputs_sha256, phase, pid, policy, preflight_only, samples, sbh and started. None is an output hash.
- **Not read.** I did not read `D/guard_csd02.py` itself, which is not in the repo. Root can confirm read-only on CSD where the guard writes the child's stdout.

**Why it matters.**
- The native launch binds every file under the overview folder at its current bytes (`job.rglob('*')`, :827). That includes `native.stdout`.
- So if `sampling.json` and a picture are forged together (C7), and that line is also rewritten (C14), the line and the printed anchor agree. Only a hash that root copied out of the folder at overview success still differs.
- A record that root reads back from `native.stdout` later is therefore not an anchor. Neither is anything else stored in that folder.

**Severity.** Low. It needs a consistent forgery of three files, and the fix is procedural.

**Recommendations.** These are recommendations only; root chooses.
1. **The ledger (procedure; no code change).** As soon as the overview receipt is final, and before anything else touches the folder, root copies the `sampling.json sha256` from the overview job's last line into its own record outside the job folder. This record is the ledger. Root also hashes `sampling.json` read-only at that moment and checks it equals the line.
2. **Prevention (procedure).** Immediately before the second `launch-pictures`, root hashes the overview `sampling.json` read-only and compares it with the ledger. This is 339 notes §1's prevention step. It is the only step that keeps the native job from starting on a forged record.
3. **Prevention (code, optional).** The native launch could require the ledger value as an argument and refuse on a mismatch before writing inputs. That is CC2's deferred optional item. It turns detection into prevention without relying on root's timing.

### N1. Compare the ledger with the bound value, not only the printed one

- **How the printed anchor is made.** `main()` hashes the overview `sampling.json` for the printed anchor (:972). `launch()` then hashes every bound file again when it writes the inputs (:925–926).
- **Why the union is the right value.** Inside the native job, the union's `overview_sampling_sha256` is computed after `require(bound, files)` (:851–852, :860). Every bound row is re-checked before the job writes its `sampling.json` (:895–896). So the union value is the one the guard enforces, and in any job that succeeds it equals the inputs row.
- **What C15 shows.** In C15 a forgery lands between those two reads. The printed anchor still matches the ledger, while the row and the union differ. The window is milliseconds inside one process, so C15 is artificial. It still shows which comparison is binding.
- **Recommendation.** Root should compare the ledger with `union.overview_sampling_sha256` in the native `sampling.json`, or with the native inputs row for the overview `sampling.json`. The printed line alone is not enough.
- **Where the notes differ.** Notes §1 say "the native launch's line *or* `union.overview_sampling_sha256`", but §5 step 4 names only the printed one.
- The cost is one more comparison.

### N2. Fix 3 (re-hash before viewing) already works with 339's code

- **What C11 shows.** C11 calls 339's own `part_done()` read-only on both finished jobs. It accepts both, then refuses a flipped native PNG and names it.
- **What is missing.** No mode exposes it, so a later `check-pictures` mode would be a small addition, as notes §4 say.
- **Until then.** Root's read-only hashing of the files listed in both `sampling.json` files, before viewing, is the interim step (notes §4).
- Whether to add the mode is root's call.

### N3. The folder checks are exact on purpose

- **Strict for `pictures/`.** `iterdir()` on `pictures/` refuses any extra entry, even an empty folder (C13). A tool that writes thumbnails or index files into `pictures/` on CSD before the native launch would block the launch. Nothing would be spent, and removing the entry restores the launch (the C10 pattern). Copying files out for viewing writes nothing there.
- **Looser elsewhere.** Files elsewhere in the overview folder, outside `pictures/` and `sheet-*`, are still bound at their current bytes, not refused. They are then re-hashed by the guard. They carry no pictures, so this is a note only.

## 3. 332 carry-overs

- **332 L1 (CSD framemd5 timebase) is resolved by probe342.**
  - Root's probe ran `/usr/bin/ffmpeg -nostdin -hide_banner -v error -xerror -threads 16 -i INPUT -map 0:v:0 -an -frames:v 3 -f framemd5 pipe:1` on CSD (Lavf62.3.100) on a retained 3840x2160 chapter. It exited 0, with timebase `1/60`, pts 0, 1, 2 and durations 1, 1, 1.
  - 339's decode argv is the same apart from `-frames:v 3` and the output target. It is `ffmpeg -nostdin -hide_banner -v error -xerror -threads 16 … -i MOVIE -map 0:v:0 -an -f framemd5 -n FILE` (:707, :711, :716), with `ffmpeg` resolved on `/usr/bin:/bin` by `runtime()`.
  - `frame_hashes()` (:564–586) requires exactly `#tb 0: 1/60`, pts 0..n−1 and duration 1. That is the shape probe342 printed.
  - As probe342 states, this resolves the runtime timebase concern only. The full decodes of the 318 master and of the 15 chapters remain the verify job's, and each refuses on any mismatch.
- **332 N1 (W3).** Unchanged. 47/48 on 339. The window constant is still pinned through the helper's hash, not cross-checked at runtime.
- **332 N2.** `verified()` still binds the verify job's outputs at their current bytes. It is inherited from 321 and outside 339's scope, as CC2 states.
- **Correction to 332's L4 text.** Review 332 said that "the verify and picture jobs run the runtime checks again under the guard". More exactly:
  - the verify job re-runs `runtime()` only (:683);
  - the picture jobs re-run both `runtime()` and `pillow_runtime()` (:840–841);
  - every launch, including `launch-verify`, runs both before writing inputs (:920–921).
  - L4 stays fixed.

## 4. Recommended run order (root's)

This is 339 notes §5 with L1 and N1 folded in.
1. `check`.
2. `launch-verify`, then the verify job.
3. `launch-pictures` starts the overview job. **At overview success:**
   - copy the last line's `sampling.json sha256` into root's own record outside the job folder (the ledger);
   - hash `sampling.json` read-only and confirm it equals the line.
4. **Immediately before the second launch,** hash the overview `sampling.json` read-only and compare it with the ledger. Proceed only on a match.
5. The second `launch-pictures` starts the native job. **Compare the ledger with all three:**
   - the printed `overview_sampling_sha256`;
   - the native inputs row for the overview `sampling.json`;
   - `union.overview_sampling_sha256`.
6. **Before viewing,** re-hash read-only every file recorded in both `sampling.json` files (N2). Then root views every sheet and every native frame.

## 5. Evidence (scratch, outside the repo; local only)

| File | sha256 | Run |
|---|---|---|
| `chain339b.py`, the anchor harness | `5f3eebea74bfdcd2b642ecdcf7f9f25328d52392a2cb96478dd6afbdc1f7004c` | 29 runs (16 on 339, 13 on the 330 control). Results `b5fdb543…`. Built from `chain339.py` (`404e1e04…`) by `patch339b.py` (`cddbe356…`) |
| `regress332b.py`, the 48 regression cases | `e23b9013603da8c17b331bffd69d0555d44660381f6379126975dbc6c229e8aa` | 47/48 on 339, the same as 330. Log `ad98373a…`. Shared verifier at `f266a231…` |
| `ast339.py`, the AST comparison | `905e429eeefc89b2f22baa6277a0aec27690af4662153c1a716b9153580709a3` | 330 → 339 and 321 → 339 |

**Local tools.** ffmpeg 8.1.2 and python 3.15.0. CSD runs ffmpeg 8.0.1-3ubuntu2.

**Faked in the harness:** the guard (a stand-in that follows the CSD receipts), the verify job's outputs, `film()`, the plan (13 samples and 1 native), ldd, CPU affinity and free space.

**Real:** 339's and 330's `main`, `launch`, `part_done` and `pictures`; Pillow; the local ffmpeg extraction; and the sha256 checks.

**Not done.** I did not run anything on CSD, and no master, chapter or picture of the real film was decoded or extracted.

## 6. Limits

- This is a technical review of code and fixtures. **No picture, motion, fine-sync, caption-reading, 4K-detail or final AV acceptance is claimed or implied. No audio heard or checked.**
- Every flag 339 writes for those reviews stays False. They remain root's and the owner's.
- I did not change any file named above.
