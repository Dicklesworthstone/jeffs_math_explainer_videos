# CC3 task 676: independent source review of the complex final assembly path

Source review only. I read files and computed hashes on this machine. I ran none of the reviewed helpers, opened no remote session, and launched nothing. I viewed no picture, heard no audio, and make no claim about motion, fine sync or independent AV.

Reviewed, by sha256 of the local bytes:

| Short name | File | sha256 |
|---|---|---|
| 606 | `combined-master606.py` | `7c8269fbd291352f4ef9a7828e3539b045f0414dada4870ddc00a057a7b5b163` |
| 607 | `root-boundary607-gate.py` | `9293391ff356226af4b7689a50a97308fa6c933b5d17870afafcdf7d53c8b2a1` |
| 647 | `csd-final-verify647.py` | `11c1f97be530b3d5f94ba1d21afdb28f98335c699d0a8b3580fb098ea99a5f0f` |
| 625 | `root-master-manifest625.json` | `6fde63a4029af7765491d0f38f5371feb701f954277d2cf3728d9f0ae4220942` |

Also read: 582, 586, 587, 588, 571, 573, 557, 558, 575, 593, 594, 595, 598, 600, 629, the 344 worker's `check()`, the kit's `master.rs` and `cli.rs` join and replay, and fmn's `sound.rs`. A reference like `606:1243` is file:line. `D` is constant D of 606 (the build host root), `Q` is its qualification folder, and `M` is constant MAC of 607 (the mac-mini-max root).

## Result

- Apart from the dropped render-command binding in R3, I found no route by which a wrong chapter 09 passes the hash chain as intended. No 606, 607 or 647 code edit is required if 09 lands as attempt a1. The findings below are pre-launch checks, that one binding, and a limit on what "independent" means for 09 and 10.
- Draft 625 unchanged fails closed. Its 09 row is kind `boundary313` on job `csd-reviewed-final-09-boundary344`. TODO records that job as exit 1 after the 43,200 s timeout. `launch` stops at `606:1243` and `check` refuses at `606:987`. I did not see that receipt, so root should treat this as code plus the TODO record. It still must not be launched.
- 607 has never run. Chapter 10's sibling 582 has run. The two differ in the 344/313 evidence chain (`607:57-81`), which is the part most likely to surprise at master time. See R2.

## Findings, most important first

### R1. Medium, hard stop if absent: confirm the 553 qualification file is inside the 09 import closure

`607:52` indexes `physical[M/jobs/native-boundary553/qualification.json]`, which raises KeyError if the row is missing. 593 exports only the rows of the 586 and 587 inputs plus everything under those two job folders (`593:37`, `593:60`). 586's own additions name 553's receipt and binary, not its qualification (`586:45` onward). So the file arrives only if it is already in the ancestry 557 → 558 → 571 → 573 → 586.

557 reads the file (`557:45`), which suggests it was frozen, but `root-boundary557-inputs.json` is not in my local copy and I cannot confirm it. Chapter 10's equivalent lookup passed under an identically shaped export (575 and 593 differ only in names), so presence is likely.

Check now, read-only, on mac-mini-max, against `M/qualification/root-boundary586-final-inputs.json`: exactly one row whose path ends `jobs/native-boundary553/qualification.json`, with sha256 `aee4c6fe288ad2842b19f2b88a8f5b322c0b7bdd0246322872f4e89c02c3ac1c`. That hash equals the local terminal copy `root-boundary553-terminal-qualification.json`.

If the row is absent, the import cannot gain it, because both input manifests are frozen. The repair is then a successor gate that reads a staged copy of that terminal qualification from Q instead of `physical[...]`. That cascades as in the table under "Exact steps".

### R2. Medium: exercise 607's never-run half now

`607:57-81` calls `provenance313`, `sampled_permit_gate`, `independent344`, the 344 worker's `preview_binding`, `independent` and `check`, and `capture_check09`. None of these depends on the 09 final. Root can call them read-only on the build host today by importing 606 by path, under a module name registered in `sys.modules` as 647 does at `647:459-462`, and feeding them the 09 row of `cc1-master447-manifest-draft534.json`.

That would surface a stale or unstaged dependency hours before it can block the master. Note that `607:57` needs `cc1-master447-manifest-draft534.json` in the build host's Q, and I found no TODO record of it being staged there. The 344 `check()` passed on the build host on 2026-11-08, before the reboot, per TODO. Its refusal list is 244, 278, 287, 303 and 317 only, so the failed 344 job does not trip it.

The one 607 comparison that cannot run early is `607:81`, the native 571 capture against binary 313's capture. It needs the import.

### R3. Low-medium: native rows drop the render-command binding every other row has

Ordinary rows bind the job's command (`606:988-991`). 607 and 582 do not. `594:69` only calls `terminal(state)` on the 586 receipt, and `607:55` hashes the binary file without tying it to the movie. What does constrain the movie is the crf and rc strings (`607:106-107`), the tables, cue log, layout, frame count and capture. A render made by another binary with the same schedule would pass.

The practical risk is low, since 586 is sealed and root launched it. Without re-versioning anything, root can close it with a one-off read-only check before writing the manifest: compare `command` in the imported `mini-boundary-final586/resource-receipt.json` with the list at `586:100-103`, and keep that result in Q. Do the same for chapter 10's receipt from `mini-final555`. Fold it into the gate only if 607 is being re-versioned for another reason.

### R4. Medium, evidence wording: inherited independent review covers Linux-rendered previews, not native pixels

`607:57-73` carries permit 351 and cc3 340 from preview313 (Linux binary 313). `582:60-64` carries permit 366 from preview274 (Linux binary 268). The master row then records `review_basis: sampled_preview` with `review_scope.preview_movie_sha256` naming the Linux movie, and chapter 10 lists `independent_confirmation` from cc3 315. The comment at `647:522-523` calls this "independent preview".

No file compares native pixels with those previews: 571's comparison is schedules only and says so (`571:90`). The native previews and finals have been viewed by root only (604, 548, 589, and the coming 587 record). Every acceptance flag stays false, so nothing is falsely flagged, but "independently reviewed" for 09 and 10 means the same source on another binary and architecture.

Root should state this in the new 09 record and in any release summary. Two options would convert the inference into evidence, both root's call: a numeric frame comparison of native 572 frames against preview313 frames at shared frame numbers, or one independent viewer of the native sheets.

### R5. Low, conditional: 647 pins attempt a1 while 607 accepts any attempt

`647:406` names `csd-native-stage595-09-a1`, `607:15` allows aN, and `600:5-6` and `629:7` hard-code a1. Chapter 10 already went a1 → a2, which is why 647 exists. 629 runs 600 inside a plain ssh session for up to 24 h (`629:70`). A lost session ends 629 with the remote outcome unknown, and `600:54` then refuses a re-run. Read the a1 receipt on the build host before concluding anything. A lost session is not evidence that 595 failed.

### R6. Low: what the 313/40 record can support

`607:93-95` checks counts, the empty defect list, non-empty observations and the absence of claims. It binds the movie and `pictures587.json` but does not read sheet hashes. Each sheet tile is 640x360 (`587:105`), so a sheet pass of a 4K final is a thumbnail pass. 594 already re-checks full decode, 313 pictures and 40 sheets (`594:77-78`), so 607 does not need to.

Root should record `sheet_hashes` as 589 did, checked against the `sheets` rows of `pictures587.json`, and state the thumbnail scope. Native 4K frames of the master come only from 647's native picture job.

### R7. Low: parameter sets are recorded, not compared

The kit stream-copies chapters through the concat demuxer. `647:989` records `extradata_md5` per chapter and never compares them, and `606:1099` skips encoder checks on retained rows. In the records I could read, the x264 option strings for 01, 10 and 11 match except for thread counts and the banner (`r3222 b35605a` on the Mac). No file gave me the other chapters' encoder strings.

After 647 verify, read `extradata_md5` in `verification.json`: either all 15 are equal, or the differing chapters name the joins to sample. No code change. 647's framemd5 identity and strict decode show ffmpeg decodes the join; they say nothing about other players. TODO's open "cross-host master compatibility" item stands.

### R8. Low: one 24 h deadline covers the render wait and the admission wait

`629:28` sets the deadline once, and `629:30` and `629:50` share it. Chapter 09 was live at 2h49m at TODO's last poll and the build host's guard lane is busy with apportionment. If the total exceeds 24 h, 629 stops. The Mac and build-host work is preserved, but root must then inspect state rather than restart.

## Checked, no finding

- Native 553 source: `root-boundary553-inputs.json` lists `boundary.rs` at `bda7d931498b...`, equal to `REPAIR310_SHA` (`606:273`).
- Hash pins agree with local bytes: 606 → 607 and 582, 647 → 606, 629 → 593, 598 and 600, 607 → 594 and 595.
- Closure contents other than R1: the 571 preview capture is bound (`573:52`, then `586:41`); `pictures587.json`, the final layout and the capture are in the exported job folders; the binary is in 586's additions.
- 09's three score parts: 595's names equal the 586 transfer names, and `606:schedule` checks each part against score21.
- Source trace of a `native553` row through `606:master` (`606:1344-1352`) and `647:master350` (`647:478-556`): no missing key. This is a reading, not a run.
- Layout assert `587:72` compares a 4K final with a 540p preview. Earlier 4K finals and 540p previews share a hash in 01 (`647:251`, `647:254`) and 08 (`647:252`, `647:256`), so resolution should not change it.
- Cross-architecture replay: every narration row in the local 09 and 10 cue logs has gain bits zero, stems are asserted at gain 0 (587), gains go through `fmn_dmath::pow` rather than libm (`sound.rs:791`), and mixing is multiply and add (`sound.rs:931-935`, `:1071-1074`). I see no platform-dependent step. It is still unexecuted: the first replay of a Mac capture with score against Linux happens inside the 606 master.
- 3.5 GiB transport cap (`594:44`): an earlier 4K chapter 09 final was 30,960,821 bytes (`repaired09-export162.json`, a different binary, so scale only). The true total is unknown until 593 runs.

## Exact steps for root after the actual 09 final review

Before 629 reaches 593: R1. Any time: R2.

1. Let existing 588 and 629 finish. Do not duplicate them. Inspect the terminal receipts of 586, 587, `csd-mini-import594-09-a1` and `csd-native-stage595-09-a1`, and read `materialized.json`.
2. View all 40 sheets in `M/jobs/mini-boundary-pictures587/sheets`, hash-checked against `pictures587.json`. Any concrete defect stops here: 607 has no waiver (`607:94`).
3. Write one new record in Q, on both machines, by exclusive create. `607:84-95` requires the fields below. `sheets_viewed` must be the literal list 1 through 40. Any key containing `accept`, `release` or `publication` must be exactly `false` (`606:452-462`). Put only computed hashes in it.

```json
{
  "movie_sha256": "<movie_sha256 from pictures587.json>",
  "extraction_manifest_sha256": "<sha256 of pictures587.json>",
  "selected_pictures_viewed": 313,
  "sheets_viewed": [1, 2, 3, "...each integer through...", 40],
  "sheet_hashes": ["<one row per sheet, from pictures587.json>"],
  "concrete_picture_defects_observed": [],
  "observations": ["<what was actually seen>", "<thumbnail scope; native pixels viewed by root only>"],
  "continuous_listening": false,
  "continuous_motion_review": false,
  "fine_sync_accepted": false,
  "independent_final_av_accepted": false
}
```

4. Write a new manifest file in Q; 625 stays untouched. Keep the other 14 rows byte-for-byte and in registry order. `606:1133` requires the manifest to sit directly in Q. Replace only the 09 row with exactly these nine keys, dropping `permit` and `worker_review` (`607:13-14`). Write absolute build-host paths as the other rows do:

| Key | Value |
|---|---|
| `chapter` | `09_boundary` |
| `kind` | `native553` |
| `job` | `csd-native-stage595-09-a1` |
| `worker` | `Q/root-boundary595-materialize.py` |
| `worker_sha256` | `bc2b3dc320da6ea516f2baad37dcfc248f8d492eec37e84acd3447209be43550` |
| `inputs` | `Q/csd-native-stage595-09-a1-inputs.json` |
| `import_receipt` | `D/jobs/csd-mini-import594-09-a1/resource-receipt.json` |
| `import_inputs` | `Q/csd-mini-import594-09-a1-inputs.json` |
| `picture_review` | `{"path": "Q/<new record>", "sha256": "<its sha256>"}` |

5. Run the R3 command comparison.
6. Run 606 `check` with the new manifest. Require `film` ok and 15 rows ok, with 09 showing kind `native553`.
7. Run 606 `launch` once. It waits, re-gates, runs the delivery preflight and the 12 GiB free-space check before spending the job name.
8. Use 647, not 608: `check`, `launch-verify`, then `launch-pictures` for overview and, after it succeeds, `launch-pictures` for native. View every sheet and native frame. Read `extradata_md5` (R7).

What forces a successor:

| Event | Change required |
|---|---|
| Import lands as a2 or later | New 600 and 629 equivalents; manifest `import_*` keys. 647 unaffected. |
| Stage lands as a2 or later | New 600 equivalent; manifest `job` and `inputs`; new 647 with `NATIVE608['09_boundary'][3]` updated. |
| Any 607 edit | New 607 → new 606 (`606:925` pin) → new 647 (`647:124-125` pins). |
| 606 fails after admission, including at replay | `csd-combined-master606` is spent (`606:1229`), so new 606 and 647 names. |
| Picture defect or 09 re-render | New 586 and 587 and every downstream 09 pin. |

## Not established here

Whether 586 and 587 succeed, the actual movie bytes, any native gate result for 09, the 606 master, 647's runtime, and every final-picture, motion, listening, fine-sync, caption-playback, independent-AV and publication gate. The TODO top was read for status; nothing else about live jobs was observed.
