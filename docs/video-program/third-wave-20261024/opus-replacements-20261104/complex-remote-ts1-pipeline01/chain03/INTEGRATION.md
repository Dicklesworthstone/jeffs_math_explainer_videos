# Stage07 chain on ts1

WhiteSwan, for root (Mails 5612, 5635, 5721, 5731, 5742). `O` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01`. I launched nothing and used no SSH.

`ts1_chain03.py` is one flat file, generated from sealed `chain01` and `chain02-n2` by exact substitution, with every check kept. `diff-from-chain01.diff` shows the result. It runs derive, timeline, score, preview and pictures for the stage07 binary. It synthesizes nothing, and viewing stays a separate human step.

## Start

Needs `qualify02` to have ended with exit 0, `timeline02` to have ended with exit 0, and the stage07 audio transferred. Run by the `/Users` path, once the build has ended:

```
C=<this folder, by its logical path>
PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin" python3 -I -B $C/ts1_chain03.py --check derive
bash $C/start-chain03.sh derive
```

`--check` is read-only and spends no name, but it reads the finished build, so it can't run before then. For derive it also does what chain02's did on the real host: a dry `N2`, the unmodified `coherent_narration.py verify` refusing `N1` and passing the dry folder, and the real binary's first narrated call given to the unmodified `narration_delivery.py`.

Then run `--check` and `start-chain03.sh` for `timeline`, `score`, `preview` and `pictures`, each once. Each start waits for the build, for `timeline02` and for its own upstream exit files, and stops if one of them failed.

| Stage | Job in `O/jobs` | Output |
|---|---|---|
| derive | `complex-remote-narration-derive02` | `narration-n2`, `derive-summary.json` |
| timeline | `complex-remote-timeline03` | `timeline-output`, `baseline-table-comparison.json` |
| score | `complex-remote-score03` | `scored` |
| preview | `complex-remote-preview03` | `preview` |
| pictures | `complex-remote-pictures03` | `pictures`; `pictures_viewed` stays false |

Controller files are `O/chain-<stage>-controller03.{log,exit}`. Manifests are `O/chain-<stage>-inputs03.json`, and the last one, `chain-pictures-inputs03.json`, is what `frozen_base` inherits.

`O/complex_analysis_visual-release02` is a row in every manifest from derive on, so the guard re-hashes the binary at each job's end. `release01` stays preserved and is read only by the qualifier's check.

## timeline02 as the baseline

`timeline02` is the stage06 wording on the old binary. It supplies expected values here and is never changed. The chain checks its receipt, inputs, exit file, summary and every table's hash before use.

**The 14 unaffected chapters** must equal it table by table, after one mapping: `N2/<cue>.wav` to `BASE_N2/<cue>.wav`, in timeline `line` column 4 and cue-log column 0. Anything else stays as written and fails. That's 96 cues, so 192 path cells, derived from the baseline's own tables and asserted, with 42 tables compared.

**`09_boundary`** follows rules, decided by frame because timeline tables list rows by kind:
- Rows at or before 09_5's baseline frame stay equal, and 09_5 itself must start on the same frame.
- 09_5's row may change its take: hash and size must be this `N2`'s, and samples must be the WAV's.
- Every later row, and the `frames` header, moves by the same whole-frame shift, with every other cell equal.
- Cue seconds are recomputed as `f'{frame / 60:.3f}'`, never by subtraction. For example 0.083 plus 8 frames would give 0.216, but frame 13 is 0.217.
- Cue-log `start_sample` moves by `shift * 800`.

`09_boundary` adds 18 more path cells, 210 in total.

`baseline-table-comparison.json` is written before it is judged, so a mismatch leaves its evidence. `--compare CHAPTER` is the same read-only rule for one finished chapter:

```
python3 -I -B $C/ts1_chain03.py --compare 09_boundary
```

`full_baseline_equivalence_established` stays false in every summary.

## The rows file

The chain has no literal hash for `O/stage07-rows01.json`, because that file doesn't exist until root's audio job writes it, and a literal would force a reseal after audio.

Instead `check_pins` binds it by three things: `tool_sha256` must equal the sealed bridge's hash, `script_sha256` must equal the stage07 script, and its comparison must say 104 identical cues, 09_5 changed and 414 report rows. Its own hash is printed by `--check` and frozen into every manifest from derive on. `base_rows` then refuses any file added to `N1` or the workspace, and any file that differs.

The trade: root should compare the hash `--check` prints with the file from the Mac.

## Changes from Ruby's review (Mail 5721) and root's rulings (Mail 5731)

- **C2.** `release02` is in derive's manifest and inherited from there.
- **C5.** A moved 09_5 start is a problem. Nothing before it changed, so it can't move.
- **C1** and the qualifier's other changes are in `qualify02/INTEGRATION.md`. `QUALIFIER_SHA256` pins that file.

## Known risks for root

- **Timeline time.** `timeline03` pays the same chapter costs as `timeline02`, under the same `TIMELINE_SECONDS` of 48 h, and no full cells4 timeline has finished yet. Ruby's Mail 5740 has the numbers and the option of parallel per-chapter runs. I haven't changed this, so tell me if you want a different limit or that option.
- **Disk.** chain02's and chain03's outputs share `O/jobs` against the 56 GiB work cap, and a guard refusal spends the job name.

## What the rehearsal shows

`checks/rehearse_chain03.py` runs the chain after the jobs that really come first: the sealed qualifier, chain01's refusal, all of chain02 (so `timeline02` is a baseline those tools made), the stage07 audio from a `rehearse_stage07.py` root, then `qualify02` and chain03 through their start scripts. `checks/outputs/rehearse-chain03.txt` is its output: 50 checks, 0 failed, the same bytes on a second run.

- **Real:** all 105 cues of the rehearsal's stage07 narration, root's receipts, the stage07 tree, both unmodified narration tools, the sealed chains, the real guard, ffmpeg, ffprobe and PIL.
- **Fake:** the binary, which writes tiny tables in the kit's row order and arithmetic, plus rustup, cargo, the lock tool, the oracle and sbh.

The fake tables aren't real frames. The rehearsal proves the rules run and refuse, and it can't show a real timeline meets them. `compare_boundary` has never run on real tables, because none are local.

**For Lavender:** `Real` keeps `finish`, `qualify`, `start2`, `text2`, `n2`, `script` and `N`. `stage('timeline')` still returns chain01's refusal, and now also runs chain02, the audio install and the build, so `start2` means chain03 afterwards. The three `C.FAKE` strings you replace are unchanged, and `shift = 0` keeps every chapter's length.

## Not claimed

No compile, real timeline, listening, fine sync, picture or AV acceptance.
