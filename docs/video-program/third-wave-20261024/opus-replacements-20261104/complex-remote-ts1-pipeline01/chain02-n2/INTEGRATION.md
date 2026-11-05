# ts1 recovery on a derived narration folder

WhiteSwan, for root (Mails 5426, 5428, 5453, 5460, 5461, 5466, 5490). The design is Ruby's (Mail 5456), approved by root. I launched nothing and used no SSH. `O` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/remote-ts1-01`.

`chain02/` beside this folder is my superseded shim attempt. It is unsealed and unused, so don't transfer it. `chain01/` stays sealed and is loaded by hash.

## Why

`coherent_narration.py` writes `str(wav_path)` into each expected row, after `.resolve()`, then compares whole rows with `report.json`. On ts1 that gives `/data` spellings against stored `/Users` spellings, and `narration_delivery.py` calls the same verifier. The tools and binary are frozen, so `--narration` is the one input the chain can change.

## Start

Transfer this folder. Then, by the `/Users` path:

```
C=<this folder, by its logical path>
PATH="$HOME/.local/bin:$HOME/.cargo/bin:/usr/bin:/bin" python3 -I -B $C/ts1_chain02_n2.py --check derive
bash $C/start-chain02-n2.sh derive
```

`--check derive` is the real test. It spends no job name and does all of this on the real host:
1. Recomputes every row with the unmodified `unit_spans` and checks them against `report.json`.
2. Builds a dry `N2` in `/tmp`, with symlinks for the 108 unchanged files.
3. Confirms unmodified `coherent_narration.py verify` still refuses `N`, with the exact message, and passes the dry `N2`.
4. Runs the real binary to its first narrated call with a refusing `python3` stub, then gives that exact call to the unmodified `narration_delivery.py`.

It leaves the stub and dry folder in `/tmp`, because nothing here deletes files.

Then run `--check timeline` and start `timeline`, `score`, `preview` and `pictures` the same way, each once. Each waits for its upstream exit files. Run `--check` for the later stages before you start them.

| Stage | Job in `O/jobs` | Output |
|---|---|---|
| derive | `complex-remote-narration-derive01` | `narration-n2`, `derive-summary.json` |
| timeline | `complex-remote-timeline02` | `timeline-output`, `prior-table-comparison.json` |
| score | `complex-remote-score02` | `scored` |
| preview | `complex-remote-preview02` | `preview` |
| pictures | `complex-remote-pictures02` | `pictures` |

Controller files are `O/chain-<stage>-controller02.log` and `.exit`. Manifests are `O/chain-<stage>-inputs02.json`.

For an early look at a finished chapter, this read-only command applies the same mapping:

```
python3 -I -B $C/ts1_chain02_n2.py --compare 01_the_bump
```

## What N2 is

`N2` is `O/jobs/complex-remote-narration-derive01/narration-n2`. Of `N`'s 110 files:

| File | Change |
|---|---|
| `report.json` | The `source` value in each of the 414 rows, to what the unmodified `unit_spans` returns on that host. No other byte changes. |
| `assembly.json` | The value of `report_sha256`. No other byte changes. |
| The other 108 | Byte-identical copies: 105 WAVs, `external-units.json`, `native-install-receipt.json`, `phrase-timings.json`. |

`report.json` has its own compact encoding, so the writer substitutes exact quoted strings and doesn't re-serialize. The parsed result must then equal the rows the verifier will compute.

**The writer stops unless:**
- all 105 cues and 414 rows are present, with the same row count per cue and the same keys per row;
- `source` is the only field that differs, in every row;
- each stored and fresh pair is `samefile`, and the fresh file hashes to `source_sha256`;
- the 105 takes map one to one;
- each old string appears only in its rows, and the old report hash appears once in `assembly.json`.

Then both unmodified tools must pass `N2`, and `N` must still be refused. `N`, the takes, the tools and the binary aren't touched. Every later stage rebuilds the two files from `N` and refuses any difference in `N2`.

## Exact path mapping

`compare_prior` is the only comparison replaced. Before equality, it changes only these cells:

| Table | Rows | Column | Must be exactly | Compared as |
|---|---|---|---|---|
| `<chapter>.timeline.tsv` | `line` | 4 | `N2/<cue>.wav`, cue from column 1 | `N/<cue>.wav` |
| `<chapter>.cuelog.tsv` | every data row | 0 | `N2/<cue>.wav`, cue from that name | `N/<cue>.wav` |
| `<chapter>.cues.tsv` | none | none | | |

A cell that doesn't match exactly stays as it is and fails. All other cells must be equal, including cue-log hash, bytes, frame, offset and timing cells. The 12 tables must map exactly 52 cells. I confirmed the macOS tables hold exactly those 52 path cells, 26 of each kind, and no others.

The sealed `timeline-summary.json` scope sentence still says "line for line". Read it with `prior-table-comparison.json`, which records the mapping and the count per table.

## What changes from chain01

All stage code is sealed chain01, loaded by hash. This file replaces only:
- job, manifest and controller names, with `derive` added upstream of `timeline`;
- `N` to `N2` for every stage, while `base_rows` keeps freezing `N`;
- `compare_prior`, as above;
- `check_qualified`, which also requires timeline01's failed receipt and its exact refusal line, once;
- `check_upstream` and `stage_files`, for the new stage;
- `child_gate`, which retries only a half-written receipt parse (Mail 5490). The guard rewrites that file just after `Popen`. A failed gate isn't retried.

**Manifests:** the derive manifest freezes `N`, the qualifier's files, chain01, and the failed job with its manifest and controller files. The timeline manifest adds all 110 `N2` rows and `derive-summary.json`, and later manifests inherit them.

**The 4K stage needs `N2` too,** because the binary's narration gate runs there as well.

## What was tested

`checks/rehearse_n2.py` passes all 37 checks in a tree whose repository path is a symlink, with a byte-identical replay. Real parts:
- all 105 cues, their takes and receipts;
- both unmodified tools;
- the sealed qualifier and chain01;
- `guard01.py` logic and the preserved guard;
- ffmpeg, ffprobe and PIL.

Fake parts are the binary, the macOS receipts and `sbh`. The fake binary makes the kit's exact narration call, then writes tiny tables. Takes are APFS clones, so production bytes can't be reached.

- Sealed chain01 first reproduced the real refusal under the guard.
- All five stages then ran under the guard.
- Damaged reports, takes and hashes, an edited `N2`, relabelled failure evidence and an edited chain01 were all refused.
- Three wrong table cases stopped the timeline, each with its comparison kept.

I also ran the macOS release binary with a refusing stub: 41 arguments, `--gain-db 0`, 15 chapter keys, and no output folder written. The kit source shows the ts1 binary makes the same call.

## Limits

- **Nothing here has run on ts1.** `--check derive` is the first real test of the derivation and the binary's narration gate.
- **The real binary hasn't run past that gate on `N2`.** `Narrator::load` was read, not run. It uses `source` only to compare spans between cues, and the mapping is one to one.
- **The stub probe needs `/tmp` to allow execution.** Root's Mail 5461 shows no `noexec`.
- **A table difference still costs the whole timeline run,** so use `--compare` when chapter 01's tables appear. x86_64 against macOS table equality is still untested.
- **The refusal for a child's ffmpeg outside the manifest still hasn't been rehearsed.**
- **The 4 GiB per-job growth cap wasn't measured for preview or pictures** (Ruby, Mail 5456).
- **Not independently reviewed yet.** Ruby will check the writer and mapping. I wrote this code, and the fix design is Ruby's. Nothing here is listening, picture, fine sync or AV acceptance.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-remote-ts1-pipeline01/chain02-n2
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/checks/rehearse_n2.py
```

Run these from the repository root on the local host, on APFS. The rehearsal needs 10 GiB free, because it uses the real guard's admission.
