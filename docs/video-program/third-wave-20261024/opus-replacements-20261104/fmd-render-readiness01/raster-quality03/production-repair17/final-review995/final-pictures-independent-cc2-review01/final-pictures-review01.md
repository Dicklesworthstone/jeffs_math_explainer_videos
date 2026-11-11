# Review of CC3's repaired-final 4K extraction helpers

CC2 (RubyRaven), 2026-11-11, about 00:15Z. Independent review of CC3's sealed delivery `final-pictures-cc3`, which holds the 969 successors. I wrote no part of them. Everything on CSD was read-only, and I launched nothing.

**Verdict.** Nothing in the helpers themselves blocks them. One Medium is a sequencing conflict between this delivery's lineage check and the planned order of finals. It is cheapest to settle before root stages the runner. There are also two Lows.

## What I read

My snapshot of the delivery:

| File | sha256 |
|---|---|
| `csd-native-repair-pictures01.py` | `5fa242b8` |
| `csd-native-repair-pictures01-after-final.py` | `c27f4ae2` |
| `check-final-pictures-cc3.py` | `de57bd12` |
| `check-final-pictures-cc3.out` | `4c4939cf` |
| `pictures-vs-969.diff` | `2c9b228c` |
| `after-final-vs-969.diff` | `65a4e59d` |

**Baseline.**
- 969 pictures `64045fcd` and 969 after-final `e8876c7f`.
- `completed-pictures969.json`.
- The sealed run helper `6dceac47`, from my previous review.
- CC3's integration proposal, section 14, for the order.

**My checks.**
- I reran CC3's checker on my copy: 88 passed, 0 failed. The output is byte-identical to CC3's (`4c4939cf`).
- Read-only on CSD:
  - the 969 job's disk use and time;
  - 09's table pins against timeline01's actual 09 output.

## Preserved from 969

**Process.**
- The same base761 pin `32bf6d9e`, with `own_group`, the stop signals and `CRATE`.
- The same guard POLICY.
- The guard receipt must be admitted, native, not preflight and not finished. The job directory must hold only `command.log` and the receipt.

**Decode and extraction.** These are 969's lines, kept, with only the chapter and frame count made variables:

- the probe: one video stream at 3840x2160, 60/1, and `nb_frames` equal to the chapter's frames;
- the full `-xerror` decode;
- the exclusive `select.filter`;
- the `-n` JPEG extraction at `-q:v 2` with passthrough fps;
- the 1280x1536 sheets of eight 640x360 tiles.

**Frame selection.** The selection rule is 969's, now inside `choose()`. CC3's checker shows `choose(prior775, 969 layout rows, 16504)` reproduces the completed 969 selection exactly: 814 frames and 2 anchors.

**Tables.**
- The final's timeline and cues tables must equal the preview's byte for byte.
- The timeline must also give the run table's normalized digest under `normalized()`. That is the same line-row rule the four timeline and preview helpers use, checked by CC3 against their `EXPECTED`.
- 09 keeps 969's plain-replacement pins as well.

On CSD, timeline01's actual 09 tables already meet both of 969's pins under the delivery01-to-rebind313 replacement: `eed5a50a` for cues and `5a91b8f9` for timeline. So 09's extraction will not refuse there. preview01's 09 tables did not exist yet when I checked.

**Retained frames.** The kept set is exactly the admitted preview extraction's frames, which root viewed in full. The helper also requires that set to contain:
- the sampling list;
- for 09, all 814 prior775 frames.

The launcher additionally requires the kept set to be inside the final extraction's frames.

## Lineage and closure

**Lineage of the final.**
- The final's receipt command must be the runner with `--chapter`.
- The final's inputs file must be the one its receipt hashed.
- That inputs file must list `RUNNER` at its current hash.

**Order of the launcher.** Its steps run in this order:
1. refuse an existing JOB or INPUTS;
2. wait up to 30 h for the final, then require `success()`;
3. rehash every row the final bound;
4. add the final job's files, PIL, the retained-set files, the launcher, the helper, the runner and the pinned base761, with a drift check on each;
5. write INPUTS with `open('x')`;
6. run the guard command, which is 969's plus `--chapter`.

**What 969 had that is gone.** 969 compared the scored cuelog with `reference586`. That check is dropped, and the cuelog is now only hashed into the record. That is right, because the repaired narration changes the cuelog. CC3's proposal moves the check to `schedule()` in the stage successor (991 line 182). I will confirm it is there when I review CC0's 991/992 successors.

## Resource

**969 actual, from the job and its receipt.**
- 814 pictures came to 236.7 MB, about 290 KB per 4K JPEG.
- 102 sheets came to 33.2 MB.
- The job ran 5 min 23 s.
- Work grew by 0.27 GB.

**Estimate for 09.** The kept set holds at least the retained union of 1506 frames. In proportion, that is about 0.45 GB of pictures plus about 65 MB of sheets, and roughly 10 minutes. Even at the new 4000 cap it would be about 1.2 GB. Both are well under `phase_growth_gib=4`. With 17.9 GiB free now, both also stay above the 8 GiB stop.

**Timeouts.** These are unchanged at 3600 s each for decode and extraction. 969 needed about 5 min for the whole job.

**The cap rise.** CAP goes from 1500 to 4000, and that change is necessary. 09's retained union of 1506 alone is above 969's limit, which CC3's checker confirms.

## Medium

**S1. The runner can be staged only once, but section 14's order fills its second pin later.**

`lineage()` requires `bound.get(str(RUNNER)) == sha(RUNNER)`. The runner on disk must still be the exact file the final's guard bound. The final's own `success()` rehashes the same file too, because the runner is in its INPUTS.

CC3's section 14 lets 09, 08 and 01 go first, with 11 waiting for the counting02 chain. The run helper allows this: `unresolved()` checks only the chapter's own chain. But `native-counting02.binary_sha256` can only be filled after build02 qualifies. Filling it means changing the one fixed `RUNNER` path. On CSD that is also an overwrite of a staged file.

Then the change breaks things:

| When the runner changes | Effect |
|---|---|
| A final is running | That final fails `success()`. That is 1.6 to 4.3 h lost, and the job name is used up. |
| After a combined01 final, before its extraction | The extraction refuses ("not the runner the final job bound"). |

Both fail closed, but either costs hours. Two ways out, neither needing a new gate:

- **(a) Recommended.** Run build02 to a qualified binary before staging the runner. Then fill both pins and stage the runner once. build01 took 11 min 22 s. The counting02 delivery, timeline, preview and pictures jobs can still follow later, because they do not touch the runner.
- **(b)** If the combined01 finals must not wait, stage the runner now with the combined01 pin filled, and treat it as frozen. 11 then needs a successor runner path. The pictures helper hard-codes `RUNNER`, so 11's extraction also needs a successor, as do any stage successors that read the runner.

This applies equally to my earlier render-helper review (Mail 25005), where I did not raise it.

## Lows

**L1. A mistyped chapter costs a 30 h wait.** The after-final launcher checks the chapter against the runner's table only after its 30 h wait. A wrong value such as `08-loop-sums` waits the full deadline on a final that will never exist, then exits without creating a job. One `assert chapter in run.CHAPTERS` before the loop fixes it. That is an argument check, not a gate.

**L2. The checker never runs `lineage()`, `retained()` or `main()`.** It is static: 88 assertions, with `choose()` and `normalized()` run on 969's real record and on synthetic input. That is acceptable under "no new gates". The CSD facts above stand in for the rest: the 09 pins and the disk and time figures.

## Notes

- The record keeps every `pictures969.json` field and adds `chapter`, `final_job` and `preview_extraction`. CC3's checker confirms this.
- **Interpreter.** `lineage()` compares the final's receipt command with `sys.executable`. So the after-final launcher must run under the same interpreter path as the final's launcher. 968 and 969 both used `/usr/bin/python3`.
- No review scope is carried, so root's M1 rule holds: scope comes only from viewing the actual extraction.

## Not claimed

No native, picture, motion, listening, fine-sync or AV acceptance, and no speed claim. The ElevenLabs Frederick narration is unchanged and not assessed.

## Files

In my scratch, `final-pictures-review01`:

- `final-pictures-review01.md` (this file)
- `cc2-check-run1.out`
- `snapshot01/`, holding the six delivered files
