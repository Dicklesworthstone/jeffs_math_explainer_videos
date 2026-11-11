# Recovery 1000: complex chapter 09 bindings, final 968/969 into master 993

WhiteSwan (CC3), 2026-11-10. This is a read-only audit of the local copies in `production-repair17/`. I made no SSH
connection, launch, render, synthesis, edit, TODO or Beads change. I viewed no pictures and heard no audio.

## Verdict

There is **one concrete provenance mismatch**: in the record of the independent review of render 968. It does not
change any movie or audio bytes, and it does not block 995. There are two low-severity record gaps in the master 993
summary. Everything else that binds 968 and 969 to the 09 row of master 993 agrees exactly.

## D1. The independent review of 968 binds the pre-change helpers, not the launched ones

- `independent-native968-review-and969-handoff.md` (`46651880…`) names:
  - runner `7d5658bbd0c0977b09f49c2838d94fe70c6fe8111695ba5bc1db5d36a44d7913`;
  - launcher `7af9323e1b12bd47cb5b194e1588994c2a8958cb445f4717420f7e0258055762`.

  These are the preserved files `csd-native968-before-controlled-stop-run.py` and `-launch.py`. The review's text also
  requires "native cleanup TimeoutExpired evidence", which the launched helpers no longer accept.
- The job that ran used different helpers:
  - The 968 receipt (`completed-native968-resource-receipt.json`) runs `csd-native968-run.py`, which is `2634c842…`.
    Its launcher is `47b8f22b…`.
  - Gate 992 (line 100) pins `2634c842`.
- TODO (12:41 UTC) says "Independent787 source-reviewed exact controlled-stop change". However, no `.md` or `.json`
  file in `production-repair17/` names 2634c842 or 47b8f22b as independently reviewed.
- The reviewed and launched versions differ in only two places:
  - the docstring;
  - the precondition on the failed 864 cleanup record. That precondition was `'TimeoutExpired' in reason`. It is now
    `reason == 'signal 15'`, group 782328, child_exit -15 and exit -9.

  The render argv, binary, narration, score, quality, affinity and guard are all unchanged.
- Suggested fix: root or the reviewer records a one-line addendum that binds 2634c842 and 47b8f22b, or cites the
  relayed review message that does. Nothing needs to be rerun.

## Low record gaps in `completed-master993-summary.json` (`38b52105…`)

These do not affect gating. Gate 992 requires both 969 reviews, and verifier 995 carries all 814 frames from 969.

- **L1. The 09 picture basis named in the summary is the old preview.**
  - The 09 row's `review_basis`, its `review_scope` and `disposition`, and `review_bases['09_boundary']`, all describe
    the historical 313 preview:
    - movie `5e50811b…`;
    - frame intervals 2080 to 2109 and 2316 to 2336;
    - dispositions in `cc3-preview313-pictures340.md` and `root-preview313-pictures341/343/345`.
  - The repaired evidence from 969 appears only as `native_final_samples.record`, which is the root review `61ecd664`.
  - The independent 969 review (`93479c7a…`) is never named in the summary. It is reached only through the stage
    proofs `cc4c6f46…`.
  - `open_reviews` for 09 still lists `final_picture`.
  - For release, cite `root-final969-review.json` and `independent-final969-review.json` as the picture basis for 09.
- **L2. The 09 narration named in the summary is 313, not the narration 968 rendered with.**
  - The 09 row names `csd-review-repair-rebind313/narration`. Render 968 used `csd-native-delivery849/narration`, and
    849 is never named in the summary.
  - The audio is byte-identical:
    - Materializer 991 checks the 105 WAVs, every file other than metadata, the sentence rows, the outputs and the
      units.
    - The 09 capture `29899f9e…` equals the captures of finals 586 and 158.
  - The direction-hold metadata differs:

    | Narration | Source it binds | Hold lines |
    |---|---|---|
    | 313 | `boundary.rs` bda7d931 (before repair 752) | 942, 1007, 1286 |
    | 849, map ff852ebc | cd1900aa, the rendered source | 985, 1050, 1329 |

  - To show the hold provenance for 09, cite 849, not the row's narration field.

A cosmetic point: `root-final969-cache/progress02.json` repeats progress01's timestamp (17:10:08.395785), although it
records 480 views against 240.

## What agrees, and what I checked

- **968.**
  - The receipt shows exit 0, from 12:40:47 to 16:57:43.
  - Inputs `79f859bd` are equal at start and end, with no resource stop.
  - The runner pins:
    - binary 844 `e7525873`, built from `csd-native844-boundary.rs` = `cd1900aa`;
    - the preview PCM;
    - the score stems.

    It renders 4K60 at CRF 16.
- **969.**
  - The receipt shows exit 0, from 16:57:50 to 17:03:13. Inputs `d81fd9e0` are stable.
  - Extractor `64045fcd` and controller `e8876c7f` were used. Its pins (`32bf6d9e`, `8730f875`, `2a186477`) match the
    local files.
  - The unlaunched draft `before-hash-pin-fix-…` held two corrupted pins. It is preserved and was not used.
- **969 record `83e80dab`.**
  - The local copy and the reviewer's copy are identical.
  - It records:
    - movie `167aba1f`;
    - 16504 frames, fully decoded;
    - 814 pictures on 102 sheets;
    - layout `f91bb8a1`, the same as 775. Its two lint rows (156.8 s and 223.2 s) already existed.
  - The tables equal preview 853. The cuelog equals final 586, apart from two path substitutions.
- **Reviews.**
  - Root `61ecd664` (17:19) and independent `93479c7a` (17:08) both bind source cd1900aa, movie 167aba1f and extraction
    83e80dab. Both report no defects and safe to advance to master.
  - Both viewed exactly the 814 selected frames.
  - The independent review's sheet and picture hashes equal the extraction's.
  - The independent reviewer disclosed authoring the helpers.
  - All 102 sheets in both local caches match the extraction hashes.
- **Stage and gate.** Stage 994 `f0641032` and gate 992 `0d8d83e1` pin the following, and all match local copies:
  - materializer `3402d437`;
  - master447;
  - table rebind `92407bbf`;
  - manifest 625 `6fde63a4`;
  - aliases `c6408b16`;
  - runner `2634c842`;
  - binary `e7525873`;
  - source `cd1900aa`.
- **Master 993.**
  - Helper `1955ca32` differs from 972 (and 972 from 874) only in version, job, inputs and the gate pin.
  - The receipt shows exit 0, from 17:28:09 to 17:42:48. Inputs `a610afcd` are stable.
  - The command passes the stage a2 manifest, which the summary records as `8da77ed7`.
  - The 09 row binds:
    - native844 stage a2;
    - movie `167aba1f`;
    - 16504 frames, starting at 67065;
    - a frame sequence that is contiguous to 133662;
    - a bit-identical replay.
  - The local master, SRT and VTT (`80244e88`, `5d064232`, `26e0d7e0`) equal the summary's outputs.
  - Publication check 993 binds summary 38b52105, and the 09 start (18:37) equals the row's.
- **Verifier 995.** Helper `69fa3ded` pins master 993 and worker 991. Controller `9a055c26` pins 995.

These CSD-only files are not in this folder, so I could not check them:
- the 968/969 input manifests;
- the stage a2 inputs, proofs and manifest, and `materialized.json`;
- the master 993 inputs;
- the CSD copies of both reviews.

## Pending live jobs

I did not observe these myself. They are taken from the top of TODO:
- `csd-final-verify995` and its two picture parts;
- exponential `css-exponential-master987` input preparation.

None of this audit's findings needs them stopped or repeated.
