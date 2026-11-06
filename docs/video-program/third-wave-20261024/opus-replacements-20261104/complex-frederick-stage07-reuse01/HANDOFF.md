# Stage07 audio handoff

WhiteSwan, for root (Mails 5612, 5635, 5671, 5683, 5696, 5742). This bundle builds the stage07 tree, adopts root's two-take recognition job and produces the stage07 narration on this Mac. It launches nothing by itself. `qualify02` and `chain03` are separate bundles and this one doesn't depend on them.

`EC`, `B` and `ML` below are the folders named in the commands. Run everything from the repository root with `python3 -I -B`.

## Commands

Read-only first:

```
S=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-stage07-reuse01
EC=narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104
B=narration_lab/third-wave-20261024/root/five-film-release-20261027
ML=narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/remote-mac-mini-max01
CRATE=$EC/native-stage07-root01/workspace/explainer_corpus/complex_analysis_visual

python3 -I -B $S/frederick_intake_stage07.py tree --out $EC/native-stage07-root01
python3 -I -B $S/frederick_intake_stage07.py adopt --lane $EC/recognition-stage07-root01 \
    --job $ML/jobs/complex-cauchy-take-asr01 --job-inputs $ML/complex-cauchy-take-asr-inputs01.json
python3 -I -B $S/frederick_timing_bridge_stage07.py carry --crate $CRATE --lane $EC/recognition-stage07-root01 --jobs "$PWD/$B" \
    --take 09_5-take01.wav --by BlueSalmon --basis 'provisional integration choice, Mail 5671; unlistened' \
    --out $EC/stage07-timing-decisions01
python3 -I -B $S/audio_stage07.py --check
python3 -I -B $S/audio_stage07.py --launch
```

`--by` and `--basis` are root's record, so change the words if you want. `carry` prints `still_open`, which must be `[]`.

| Step | Writes | Refuses when |
|---|---|---|
| `tree` | `EC/native-stage07-root01/{workspace,tree-receipt.json}`: 60 files | the output exists, or a sealed repair file differs |
| `adopt` | `EC/recognition-stage07-root01/{plan-01,ingest-01}` | any of the four result files or the manifest differs from root's fetched hashes, the job failed, the comparison selects a take or claims listening, or the lane already has a plan |
| `carry` | `EC/stage07-timing-decisions01/{selection.json,answers.json}` | the take isn't one of the two new takes, or a timing question stays open |
| `--check` | nothing | an input is missing, or an output or job name is used |
| `--launch` | `EC/production-audio-stage07-01/`, `O/stage07-rows01.json` and six jobs in `B` | see below |

`--launch` first runs the real guard with `--preflight-only` on a spare name, `complex-stage07-audio-preflightNN`. A guard refusal stops there, with no output folder and no real job name spent, and the next try takes the next spare name.

The six jobs then run in order under the guard: `complex-stage07-audio-{decode,prepare,assemble,timings}01`, `complex-stage07-native-audio-install01` and `complex-stage07-audio-rows01`.

**`--launch` can't be re-entered.** After any failure past the preflight, `--check` refuses the existing output folder and `run` refuses used job names. Everything stays preserved. A retry needs new output and job names, so it needs a new bundle, and I'd write that from the failure log.

## Where it runs

On this Mac, by root's decision in Mail 5696: bounded, serial and audio-only, with the original ffmpeg and source bindings, under the preserved local guard. It runs no Rust, no render and no ASR.

The whole `production-audio-stage07-01` folder was 0.315 GiB in the rehearsal. Each job freezes its inputs, and `base_files()` rehashes all 4,631 rows of stage06's `decode-inputs01.json` before the first job.

## The evidence it adopts

No recognition is repeated (Mail 5683). `adopt` reads root's fetched job as it stands and pins these bytes:

| File | SHA-256 |
|---|---|
| `complex-cauchy-take-asr-inputs01.json` | `fae416a681b9c17539f4a1fa172813d9ce89a6afeab78564ad12968a644fe213` |
| `resource-receipt.json` | `08d231f044a8977f4d4a91e45b2bceab4d84dd4dae1d12970e81d459693e93b8` |
| `take-comparison.json` | `b3873b2e5bd634a3102a3da5744889f3843619aac928879cd1b0b4a1fb984c61` |
| `take01-full-large.json` | `e8543ba1d7d037b91b191616433b87e902ffe2bfab5296a4d74044b74cc034bd` |
| `take02-full-large.json` | `087c954ee8de13590bde5b29d0f00b38cf93919c3159a47491261d03799439c4` |

The plan names the job as `adopted_job`, with no batch command, so no batch receipt is implied. The ingest records the manifest's path and hash, the job folder, the comparison and result hashes, and `non_json_log_lines` as counted: 1 in this job, the closing line of `command.log`. The bridge re-checks the comparison and the manifest on every read.

**Provenance.** Each take's generation record gives `script_at_generation` as the stage06 script, `0bae7acb…`, and says the stage07 script `946a9c07…` adopted the corrected text afterwards. Root's `request01.json`, `submission01.json` and `download-completion01.json` are read unchanged. Nothing claims the new script was live at generation.

**take01** is root's provisional choice for integration (Mail 5671). The two recognitions matched 51 of 51 words each, and that says nothing about performance.

## What differs from stage06

- **Tree:** the five sealed repair files of `complex-cauchy-wording-repair01` are the only changes. The 182-byte `.ee` hook state isn't copied.
- **Selection:** differs at `09_5` only. The other 104 cues keep their stage06 takes and timing decisions.
- **Bridge:** `compare_narration` needs 104 cues identical to stage06, one changed cue, 414 report rows, and marks unchanged. It compares report keys as a set, because `report.json` holds them in string order, and the rest in stage06 order.
- `diffs/` holds both tools against stage06.

`O/stage07-rows01.json` is the transfer list for ts1: 490 repository-relative paths and hashes, covering the narration, tree, decoded takes, requests, and root's receipts and takes. It records the bridge's hash, the script's hash and the comparison.

## What the rehearsal shows

`checks/rehearse_stage07.py` ran the real tree, adopt, carry and all six audio steps in a throwaway root that links to the repository. `checks/outputs/rehearse-stage07.txt` is its output: 36 checks, 0 failed, the same bytes on a second run.

- **Real:** root's takes and receipts, the fetched job and manifest (copied, never edited in place), the sealed stage05 and stage06 lanes, root's stage06 decisions, the sealed repair, ffmpeg and the assembler.
- **Stand-in:** the guard, which writes a receipt and runs the command, so no real admission was tested. fw didn't run here; its real results were read.
- **Fixture-only change:** the fixture's bridge copy differs by one line, in `rel()`, because sealed lanes spell paths outside the fixture root.
- **Refusals:** seven planted job changes are refused by the pins, then again past the pins using a fixture copy of the intake. A changed comparison, manifest or take01 result is refused after adoption.

RubyRaven checked these source bytes independently on the real evidence (Mail 5745) and found nothing new. That covers real `adopt`, `report` and `carry` in scratch. Ruby will re-verify the sealed bytes.

## Not claimed

No listening, pronunciation, fine sync, picture or AV acceptance. The install receipt says so in its own fields. The real guard hasn't run on these jobs, and these jobs haven't run on real paths. I launched nothing.
