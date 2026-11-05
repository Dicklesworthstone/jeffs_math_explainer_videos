# Independent review: root's stage06 reuse integration

**Reviewer:** RubyRaven, for bead 87do.39 (BlueSalmon Mails 4620, 4622, 4629 and 4631).

**Scope.** I reviewed:
- root's pins and the print fix in `P/complex-frederick-stage06-reuse-root01`;
- root's regression copy `P/complex-frederick-stage06-reuse-root-checks01`;
- the real transfer and reports in `EC/stage06-runtime-transfer01`;
- the evidence in `EC/stage06-narration-transfer01`;
- the real stage06 take folder, `EC/frederick-stage06-01`, read through `status` only.

**Not independent of the code.** root01 is root's pinned copy of my own unfinished helpers. This review is independent of root's integration and root's runs, not of the code I wrote.

**What I did not do.** I made no transfer, plan, ingest, job, synthesis, recognition, decode or assembly, and heard nothing.

## Result

Root's integration is correct and narrow. The transfer, the report and the 50 new stage06 takes all check out at today's bytes. Two findings concern what happened after the transfer:
- **F1:** the tools cannot take the 02_5 replacement pair.
- **F2:** the 09_5 generation also landed in the stage05 take folder.

Neither changes any retained take or receipt.

## Confirmed (`checks/root_reuse_review.py`: 27/27 PASS, two fresh runs byte-equal)

- **R0: named hashes.** Every hash root named verifies:
  - candidate seal `44cb2a3f`, 58/58 rows;
  - script `0bae7acb`;
  - transfer `60880cbe`;
  - retained-takes `c1a93026`;
  - drafts `a967aece`;
  - regression script `6da7f02b`.
  
  root01's SHA256SUMS (`48ae1dff`) verifies 4/4 rows.
- **R1: root's edits are narrow.** Comparing top-level code with my unsealed staging, root changed only:
  - in the intake: `CRATE`, `MAP_REL` (now `docs/narration-direction-map-frederick-stage06.json`), `PINS`, and `cmd_plan`'s `map_name`;
  - in the bridge: `DEPENDENCIES` (the intake pin), `CHAPTER_SOURCES_SHA256`, and `cmd_report` (the print fix).
- **R2: the pins hold at today's bytes.**
  - The intake's script, units and map are the candidate's, and `check_binding` accepts the crate.
  - The units' contract names the new map.
  - The bridge pins the root01 intake at its bytes (`a28166cb`) and the 25 chapter sources (`f710eaae`; `source_side` accepts them).
  - Every other dependency equals the sealed stage05 bridge's and hashes to its pin.
  - The prepare workspace takes its map name from `intake.MAP_REL`, so the rename carries through.
- **R3: the script delta.**
  - 105 cue ids, in the same order.
  - Exactly 05_8, 10_3, 14_1, 14_3 and 15_2 changed, and none of them was ever downloaded. (09_8 was not changed.)
  - All 55 transferred cues keep their text.
- **R4: the transfer.**
  - `check_transfer` accepts it today.
  - Re-deriving it in memory from the sealed stage05 side gives the same pins, lane files, 55 cues with their receipts and takes, and the same not-transferred lists. Only its informational stage05 scan is now historical (F2).
  - **Walked without the tools:** each transferred request still binds the stage05 script `12442480`, so it was not rewritten, and it names the cue, the unchanged text and Frederick. Each WAV is the one its receipt, its plan and its ingest name.
- **R5: the report.**
  - My in-process report on the real inputs is byte-equal to root's report02, and report01 equals report02.
  - Against the sealed stage05 bridge, all 110 take entries are equal except the transfer field, and all 29 question objects are identical.
- **R6: the print fix.** The report CLI exits 0 both with the transfer (`stage05=read, stage06=no recognition lane: 110 takes`) and with no lane at all. Only the print read the old flat `state` key.
- **Decode and assembly accept transferred requests (code read).**
  - `cmd_decode` checks the request's cue, text and voice.
  - `tools/coherent_narration.py:368–373` checks the request's hash, cue, text and voice.
  - Neither reads the request's `script_sha256`. So the stage05-bound requests pass under the stage06 units with no receipt rewritten.
- **R7: the real stage06 take folder.** The pinned intake's `status` (it writes nothing) finds:
  - all **50 remaining cues with two eligible takes each**, 100 WAVs, with nothing excluded, unreceipted or awaiting;
  - **one refusal: 02_5** (F1).
  
  Each accepted request equals root's draft in every provider field: cue, text, hashes, script, voice, model, stability, similarity, clone strength and output.
- **R8: retained-takes.json** names the same 55 cues, texts, request and downloads receipts, and 110 WAVs as the transfer.
- **R9: root's regression.** It now replays 35/37: T8 and I7 fail. Every other line of its output equals root's recorded log, apart from S1's file count (460, was 455) and the total. The cause is F2, not a tool change.
- **S1: snapshot.** All 868 real files read are unchanged: size, mtime and sha256.

## Findings

### F1 (Medium; root's decision): the tools cannot take the 02_5 replacement pair

Root made a targeted 02_5 pair in the stage06 folder at 10:07Z, because both recognizers hear "turn" as "term", and kept the old pair. As pinned, neither tool can take the new pair in:
- **The intake refuses it.** 02_5 is a transferred cue: "new stage06 requests are for the remaining cues only".
- **The bridge refuses it.** It rejects any cue with takes in both lanes.
- **Names collide.** The new files are again `02_5-download01/02.wav`, and the bridge keys takes by (cue, file).

So the transferred stage05 pair is the only 02_5 candidate the bridge can report.

Options:
- **(a) Keep the stage05 pair.** The new pair stays retained evidence only. No change needed.
- **(b) Withhold 02_5 from the transfer (recommended if the new pair is wanted).** A narrow next intake version would take root's explicit `--withhold CUE` with a reason, written into a new transfer02. 02_5 would then be a remaining cue, its stage06 pair would plan normally, and the bridge would report only that pair. The old pair stays on disk untouched but is no longer selectable.
- **(c) Make all four takes selectable.** This needs stage-qualified take keys in the bridge, selection and answers. It is larger and I do not recommend it now.

I have not implemented any of these. That needs an assignment.

### F2 (Low): 09_5's stage06 generation was also written into the stage05 take folder

**What was added.** At 08:33–08:36Z, after the transfer, these appeared in the stage05 take folder:
- `09_5-request02.json` (bound to stage06 `0bae7acb`);
- `09_5-submission02.json`;
- both WAVs;
- `09_5-downloads01.json`.

The stage06 folder has byte-equal copies. Only its downloads receipt differs, because it names stage06 paths.

**Consequences:**
- No transferred file changed, so the transfer stays valid. Its recorded `stage05_scan` (09_5 awaiting download) is now historical: under stage05 rules, the new receipt is refused there for script drift.
- Root's regression no longer replays 37/37 (R9).
- **The hazard:** pointed at the stage05 folder, the pinned intake offers 09_5's two WAVs as eligible (R7).
  - The default `--takes` is the stage06 folder, so this happens only if someone points `plan` at the stage05 folder.
  - Recognition provenance would then point into the sealed stage05 folder, and the stage06 copies would be refused as duplicates.

**Root's call:** either move the five stage05-folder 09_5 files aside (the stage06 folder holds the same audio), or record them as misplaced and always plan from the stage06 folder.

**My own design gap:** the stage06 intake should refuse `plan --takes` when it names the stage05 take folder. `status` can still show it. This is a one-line hardening for any next version.

### Nits

- **N1 (cosmetic).** The bridge's prepare `why_a_workspace` text (`:1325`) still says "no oliver12 map" and "the stage05 map".
- **N2 (hygiene).** root01 holds two `__pycache__/*.cpython-315.pyc` files that are not in its SHA256SUMS, left by a run without `-B`.
- **N3 (pre-existing lineage).** The candidate seal binds `.ee/hook-state/session_start_orient.last.json`, inherited since stage03, and so does `native-stage06-root01`'s inputs. A session started with its cwd in that crate would rewrite the file and break the seal and the frozen inputs. Because it is bound by hash, a rewrite is caught, not silently used.

## Not checked or claimed

- **Not exercised:** the two-lane bridge path with a real stage06 lane, because none exists yet. My helper regression covered it only on fixtures.
- **Not run:** plan, ingest, decode, prepare and assembly.
- **Not claimed:** listening, voice or content acceptance, selection, timing or fine sync. 09_5's provider history and the 02_5 recognizer ambiguity are root's evidence.
- **My own staging:** my unsealed `P/complex-frederick-stage06-reuse01/` is left as it is, superseded by root01.

## Replay

```
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-frederick-stage06-reuse-root-review01/checks/root_reuse_review.py SCRATCH
```

SCRATCH must be new and outside the repository. The run takes a few minutes, because it builds both bridges' reports and replays root's regression.
