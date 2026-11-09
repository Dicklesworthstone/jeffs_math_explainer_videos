# Recovery 516: apportionment 540p preview after preview458 (CC2)

CC2 (RubyRaven), 2026-11-09, about 06:35 UTC, task 516 (root, through the owner's session).

**What this is.**
- A reuse proof for preview458's chapters 01–08, plus one new guarded helper that renders only 09–15 into a new folder.
- Prepared only. I launched no guard, build or render, and edited no Beads or TODO file.
- preview458, timeline444, score445, 462, 468 and every file they wrote are unchanged.
- Root runs the recovery. CC0 reviews it (524 reviewed the earlier draft; the corrected helper needs a re-read).
- The narration is unchanged: narration437, bound by hash.

## Files

| File | sha256 | |
|---|---|---|
| `cc2-apportionment-preview-recovery516.py` | `81f39578c44b676c626c466bc7a84421693278183da24af3976b3c388763a231` | new, 352 lines, sealed |
| the draft root read | `7547515c183cebb28a8ef49f9832dda1bc3a222ed93bdb6579f834f0398a4cd6` | 233 lines. Superseded; a copy is kept outside Q |
| G/`preview458.py` / G/`preview-after-score458.py` | `7ab178d1…` / `3a7c4b0c…` | root's helper and controller, read, unchanged |
| G/`preview458-inputs.json` | `5817cb642dbe308bf2c838f64784b62ea613bdcf4678061ee34f0a2457e26024` | 3,820 rows, 1,573,623,430 bytes |
| B/`apportionment-frederick-preview458/resource-receipt.json` | `a8d25c31a921545220e6911d73fc870c827dede9311d4a3cf00cb59e0fe4d96d` | the failed receipt |
| `guard_owner80g_target20g.py` | `4f34e0d1e9a8f9d164860828f87a734e99b697d2d3048c7b65508e5d84b8b4ad` | 458's guard, unchanged |
| the binary `apportionment-native209` | `c77303562bc062943796a55462a84f61af19262de760c0230e8318bd873b2df6` | |
| `cc0-apportionment-eight-chapters520.json` / `.md` | `8995aa7d…` / `22d1f906…` | CC0's independent metadata check of 01–08 |
| `cc0-preview516-review524.md` | `cf2e72ab…` | CC0's review of the draft: no defect above Low. L1 and N1 are folded in here |

- G = `narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89`.
- B = `narration_lab/third-wave-20261024/root/five-film-release-20261027`.
- Q = this folder.

## Why preview458 stopped

- The guard admitted the job at 05:13:43Z. The receipt shows:
  - exit 1 at 05:48:55Z;
  - no error, no resource stop, 53 samples;
  - inputs equal before and after (5817cb64).
- `command.log` gives the 93 cues, then success lines for 01–08 only:
  - 01 rendered at attempt 2, 06 at attempt 5 and 07 at attempt 2, after the kit's transient-EPERM retries;
  - then: `ffmpeg boundary: process mechanism: cannot spawn …/fmn-bound-ffmpeg: … Resource temporarily unavailable (os error 35)`, and preview458.py's `CalledProcessError`.
- fmn hashes the resolved ffmpeg, copies it into a private probe folder and runs `-version` before each encode (franken_manim a399d62, `crates/fmn-output/src/ffmpeg.rs:430`). That spawn failed at 09, before any 09 file existed.
- **Cause.** os error 35 is EAGAIN. This Mac's limits are `kern.maxproc` 16,000 and `kern.maxprocperuid` 10,666. Root counted 10,971 processes at the failure (3,776 of them bash), above the per-user limit.
  - At 06:18Z my checker counted 3,200 processes for this user and 3,500 in all.
- The kit retries only Darwin EPERM (`crates/explainer_kit/src/encode.rs:268`, `cli.rs:672`), not EAGAIN.
- Then 458's own end checks never ran: the 15-timeline check and `preview458-summary.json`.
- 468's controller then stopped without launching ("a prerequisite finished without success"). PID 55014 is no longer running.

## Proof that 01–08 can be reused (read only, finished 06:18:43Z)

My checker ran on this Mac and wrote only outside the repository (`check516.py` 1aeff388, output 175077ac). Every item holds; there are no problems.

- **Pins.**
  - The binary, preview458.py, the controller, the guard and the inputs file are all at their hashes.
  - The inputs file is the one both receipt fields name.
  - timeline444's and score445's receipts are successful: finished, admitted, not a preflight, exit 0, inputs stable.
- **Inputs now.** All 3,820 rows are at their hashes, with none missing (1,573,623,430 bytes).
- **458's folder.**
  - It holds exactly 52 files: the receipt, command.log, five files for each of 01–08, the eight captured soundtracks, and `soundtrack/previous/06…v1` and `07…v1`.
  - No 09–15 file exists, and there is no summary.
  - The helper pins this listing as **fc5100da** (`fc5100dab6c12d9fbb320be155872e556501512b0719f57a6513bb7fa1a30ed3`): the sha256 of `sha256  bytes  path` lines, sorted by path. CC0 rebuilt the same digest from its own 520 hashes.
- **Per chapter, against timeline444** (bound rows of 5817cb64):
  - `timeline.tsv`, `cues.tsv` and `layout.tsv` are byte-equal. 08 keeps its anchors at 0.8, 53.4 and 93.2 s.
  - The cue table matches the timeline's cue order and start frames.
  - `cuelog.tsv` is timeline444's, line for line, plus one row: the chapter's score stem. That row is at its current and bound hash and size and starts at sample 0 (gain 0 dB, as CC0's 520 also reports). The only difference is that 458 used `--score` and 444 did not.
  - Every narration row in the cue log is at its narration437 hash.
- **Movies.** Each has exactly two streams:
  - video: H.264 High yuv420p, 960×540, 60/1, with as many packets as timeline frames, colour range tv and matrix bt709;
  - audio: AAC, 48 kHz, stereo, within one AAC frame of the picture.
  - Each decodes with `ffmpeg -xerror` and prints nothing.
  - Encoder tags: `Lavc62.28.102 libx264` / `Lavf62.12.102`.
- **Encoder.**
  - The x264 SEI string is identical in all eight. It includes `crf=16.0 ref=10 subme=8 trellis=2 bframes=5 threads=17`.
  - The kit passes `-crf 16 -preset slow -tune animation` (`encode.rs:101`), which asks for 10 reference frames (`encode.rs:148`). `threads=17 lookahead_threads=2` is what x264 chose for these 540p encodes.
  - Homebrew's ffmpeg 8.1.2_1 is the only installed version, and its install receipt dates from 2026-07-13. I did not trace which ffmpeg fmn resolves, so the helper requires these exact tags and this SEI string for 09–15.
- **Soundtracks.** Each `soundtrack/<ch>/soundtrack_0.wav` is 16-bit stereo 48 kHz with exactly frames × 800 samples. The two `previous/…v1` captures are byte-identical to the final 06 and 07 captures.
- **The movies are the last attempts' output.** Kit publication is atomic, and the tables are written after the loop (`cli.rs:610–735`).

| Chapter | Frames | Attempt | Movie sha256 |
|---|---|---|---|
| 01_one_more_seat | 4,806 | 2 | `0d30be36…` |
| 02_shares | 6,647 | 1 | `921d635c…` |
| 03_largest_tails | 5,527 | 1 | `23a9cc1a…` |
| 04_sixteenth | 4,238 | 1 | `d6e69f37…` |
| 05_dials | 6,316 | 1 | `5499f041…` |
| 06_not_every_time | 4,947 | 5 | `06a69e22…` |
| 07_yardstick | 5,049 | 2 | `eaf0b190…` |
| 08_signposts | 6,448 | 1 | `cbec602a…` |

01–08 come to 43,978 frames. 09–15 still need 56,908, so the film has 100,886 in all.

**Why a partial render yields the same chapters as `render all`.** From the kit source, `cli.rs` a63970e0, which is a bound row:
- Each chapter gets a fresh `Kit`, its own linter and its own stem, and the narrator is reset to zero (`cli.rs:590–628`).
- The selected script keeps only the wanted chapters (`cli.rs:571`).
- The narrator pairs contiguous cues only within one chapter, in full-script order, so a partial selection cannot create new neighbours (`narration.rs:743–752`).
- A grep of the apportionment crate, explainer_kit and explainer_score for `thread_local!` and for static atomics, mutexes and lazy cells finds only `glyphs.rs:17`. That is an opt-in glyph capture, which ordinary renders do not use.
- The helper checks the result anyway. For every new chapter, the timeline, cues and layout must again equal timeline444's.

## The helper (why one is needed, and what it does)

preview458.py cannot finish the job:
- it hard-codes `render all`;
- it asserts that `preview/` does not exist;
- it writes into 458's folder.

A bare manual command would lose the pins and the end checks. 516 is the narrow successor.

**What it renders.**
- Chapters 09–15, one `render <chapter>` call each. The flags are 458's: `--res 960x540 --fps 60 --lint --narration G/narration437 --score B/…score445/score/stems --crf 16 --preset slow --threads 4`.
- The output goes to the new `B/apportionment-frederick-preview516/preview/`, with cwd at the repository root.
- 01–08 stay where they are: no copy, link, move or edit.

**Root's commands.** Run them from the repository root, with `PY=/opt/homebrew/opt/python@3.15/bin/python3.15`:

```
PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview516-check \
   --inputs MANIFEST516 -- PY -I -B Q/cc2-apportionment-preview-recovery516.py --check-only
PY -I -B Q/guard_owner80g_target20g.py --phase native --job apportionment-frederick-preview516 \
   --inputs MANIFEST516 -- PY -I -B Q/cc2-apportionment-preview-recovery516.py
```

**How the guard runs it.** I read the original, 819746fb:
- `--phase/--job/--inputs`, then `--` and the command (lines 89–93);
- it writes `command.log` and the receipt, then starts the command with `start_new_session=True` and cwd at the repository (lines 146–148);
- a resource stop sends SIGTERM to that group and, after 20 s, SIGKILL (lines 76–83).

**What the helper does, in order.** Every step is a fail-closed assertion.
1. **It runs only as the guard's own command.**
   - Its pid must lead its process group and its session; otherwise it refuses (any wrapper, or an unguarded run).
   - The job folder must be the mode's job under B and fresh: only `command.log` and `resource-receipt.json`.
   - The receipt must show admitted, native, not a preflight, no exit, and the 80g policy 10/8/4/80/20/10.
   - Its command must name this file, with `--check-only` only in the check job.
   - The check-only mode is guarded too, since it decodes the eight movies.
2. **It prints its own hash, the binary's and the receipt's inputs hash** (CC0 524 L1), and repeats them in the summary.
3. **Pins and preview458.** It runs everything in the proof above:
   - the 3,820 rows;
   - the 444 and 445 receipts;
   - 458's receipt a8d25c31 and the absent summary;
   - the 52-file listing fc5100da;
   - the binary's 15 chapters;
   - every score QA passed;
   - `chapter()` on 01–08;
   - and `preview/` must not exist.

   The check-only mode stops here and writes nothing beyond the guard's own files.
4. **Before each new chapter**, it requires:
   - at most half of `kern.maxprocperuid` processes for this user (5,333 of 10,666);
   - render time left: the seven renders share 458's single 14,400 s budget, and each timeout is what remains.
5. **Each render** stays in the guard's process group, so the guard's stops reach the binary and its ffmpeg.
   - On a timeout, an exception or a stop signal, `end_group()` sends SIGTERM to the whole group and waits up to 6 s. Then it writes `<chapter>.cleanup.json` in the job and sends SIGKILL to the group, itself included, so the receipt shows -9.
   - This is CC3's 521 pattern (9af15207). It fixes the draft's `subprocess.run(timeout=)`, which would have killed only the binary and could orphan its ffmpeg.
6. **After each render**, `chapter()` checks the new chapter exactly as it checked 01–08, and prints its file hashes (CC0 524 N1).
   - For the split stem of 12 (two parts, at samples 0 and 5,760,000), it expects one row per part. That follows the kit: `kit.rs:204–224`, `score_stem.rs:54`, `narration.rs:539`.
7. **At the end**, it checks 458's listing again and writes `preview516-summary.json` (mode `x`) with:
   - the 15 chapters and each chapter's folder, frames and file hashes;
   - the hashes from step 2 and the guard binding;
   - the reuse binding (job, receipt and listing);
   - the x264 settings;
   - a scope that accepts nothing.

**Changes after root's reading of the draft.**
- Root freezes MANIFEST516: 458's 3,820 rows, this helper, the 52 preview458 files and the original bindings. The helper does not read it; it records its hash from the receipt.
- `--check-only` is run guarded and refuses otherwise.
- The owned-group stop replaces the bare timeout.
- The guard command is verified, as above.
- The shared 14,400 s budget replaces 7 × 3,600 s.
- 524's L1 and N1 are folded in.
- `chapter()`, `stems()`, `wav()`, `listing()`, `success()` and `pressure()` are unchanged from the draft (AST-equal, test K1).

## Evidence and what I ran

**Read only, before the corrections:**
- `check516.py` run02 (above). It ran ffprobe and eight null decodes of the 540p movies on this Mac.
- `test516.py` run01 (8d52a733, output 6cfed89a): 12/12 on the draft 7547515c. It ran the draft's pre-render checks unguarded three times:
  - `--check-only` once;
  - two `main()` refusal cases with `render()` replaced.

  That means 24 null decodes and three passes over the 1.57 GB of inputs.

  This was before root's direction that check-only must be guarded. I have not repeated it, and I ran no heavy check after that direction.

**On the sealed helper, light only:** `test516b.py` (d6279d79, child b78a9cf7, output 023741ae) passed 8 of 8. It never runs `main()`, a render, a decode or the input hashing. Children run only `/bin/sleep`.

| Case | Shows |
|---|---|
| K1 | The checks run01 exercised are AST-equal to the draft's |
| K2 | `own_group()` refuses in the caller's group and passes under `start_new_session` |
| K3 | A fresh, admitted, native 80g job naming this helper and its mode passes. Seven departures are refused: an extra file, finished, another policy, a preflight, another command, the wrong mode, no folder |
| K4 | A 2 s stand-in timeout ends the sleeper and its grandchild, records `TimeoutExpired` and ends with -9 within 2.2 s, and leaves no process in the group |
| K5 | SIGTERM to the helper stand-in alone ends its whole group (`signal 15`), with nothing left |
| K6 | With 5,000 s stand-in renders, the timeouts are 14,400, 9,400 and 4,400 s, and it stops before 12 with 09–11 finished |
| K7 | At 5,334 processes it stops before 09 without rendering |
| K8 | The helper and the draft copy are unchanged |

## Open points for root

1. **MANIFEST516 is root's.** 516 binds 458's rows itself and records the manifest hash, but does not check what the manifest contains.
2. **Process pressure.** The check counts only this user and runs only between chapters. A burst during a chapter can still cause EAGAIN, and the kit does not retry it.
   - That chapter then fails with exit 1 and no summary.
   - Chapters already passed have printed their hashes, so a further successor can reuse them the same way.
3. **Partial leftovers.** A render that exits non-zero is not swept for leftover processes (as in 521).
   - A spawn failure inside `run_owned` ends the group with -9 and a record, not exit 1.
   - A failed chapter can leave partial files (for example a soundtrack capture). Only chapters that passed `chapter()` are reusable.
4. **Chapter 12** is the lane's first scored split-stem render. If its rows differ from the kit-derived expectation, the helper stops before the summary and keeps the files.
5. **Layouts.** Equality with timeline444's 64×36 layouts is required for 09–15, as it held for all of 01–08. A difference stops the run for review, not for repair.
6. **462 and 468** are frozen and pinned to preview458's folder and summary, so they cannot read this split output. A successor that reads `preview516-summary.json`'s chapter map is root's and CC0's call. I wrote none.
7. **Budget.** 458's log gives 2,056 s of chapter rendering, and its whole guarded child ran 2,112 s. If root wants only what 458 left of its 14,400 s (about 12,290 s), `RENDER_SECONDS` is the one constant to change.
8. **Expected size and time.**
   - Size: 56,908 frames, about 182 MB of soundtracks and about 40 MB of movies (458's 685 bytes per frame), well under the 4 GiB per-job cap.
   - Time: at 458's 21–35 fps, about 30–45 minutes of rendering, plus the checks.
9. **Lane.** The guard holds its single lane lock. Root decides when this runs relative to the other queued jobs.

## Limits

- **This is technical evidence, not acceptance.** It covers hashes, headers, packet counts, null decodes and stand-in process tests.
- **No claims.** I make no claim about pictures, motion, listening, loudness, fine sync, captions or independent AV acceptance. Nothing here authorizes a render beyond root's guarded run, or any publication.
- **Unverified by me.** I heard no audio and looked at no frame. I have no result for 09–15.
