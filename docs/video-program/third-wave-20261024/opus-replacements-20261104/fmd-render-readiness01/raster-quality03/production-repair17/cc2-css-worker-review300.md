# CC2 review 300: CSS final-render helper 290

cc2, 2026-11-08. This is an independent review of CC1's CSS adaptation of 287. As root asked, it covers only runtime, transfer, path and resource issues. Nothing was run on CSD or CSS: no source edits, no jobs, no permits, no Beads and no TODO. It makes no picture, motion, audio or AV claim.

Terms, as in CC1's notes:
- D is the CSD workspace path. On CSS it is the existing alias into `R/work/frozen-runtime75`.
- R is the CSS workspace.
- Q is CSD's `production-repair17` folder.

| File | sha256 |
|---|---|
| `css-final-repair290.py` (720 lines) | `720e63a66d9e67d04c1cd99d2131d76401e42a5e2d60240fb1afc9d3a93ac8f2` (root's named hash; unchanged since 06:28:17Z) |
| `css-final-repair290-notes.md` (189 lines) | `9bed252cb42669fcc4a5d9168e91a10592fa797bc43d9a0070ae6809fbe80384` |

## Result

**0 High, 0 Medium, 5 Low, 4 notes.**

No finding stops CSS290 being used for what it can do today: stage the runtime, run the narration delivery, and run 06/08 once a real permit exists. Every Low fails closed. The cost is a spent fixed job name or a refused permit, never a wrong render.

The Lows are about not spending job names, and about permit naming. Timeline269 is still running, and the helper's `manifest` mode refuses until it finishes cleanly. That was the right result at review time.

## Lows

**L1. Re-issued and same-named permits.** The notes' "new job suffix" does not cover a re-issued permit.
- **Why.** `destination()` (`:118–123`) places every permit at `PERMITS/<file name>`. Evidence goes to its fixed path under FROZEN. `receive` (`:311–316`) refuses any existing destination whose bytes differ. So two cases are refused even under a new job suffix:
  - a re-issued permit that keeps its file name;
  - two chapters' permits with the same file name.

  The same applies to evidence whose file at a given CSD path has changed since an earlier transfer.
- **Fix.** For every chapter and every issue, give the permit file a **new, unique name** (chapter plus issue number, for example). Never cite an evidence path whose bytes may change.
- **Notes to correct.** In §3B and §4 step 8, say that a re-issue needs a new file name as well as a new job suffix.

**L2. The runtime transfer is single-shot.**
- **Why.**
  - `css-repair-transfer290` and `css-repair-delivery290` are fixed names, and `manifest_kind()` (`:284`) accepts only that one transfer name.
  - A broken relay therefore spends this helper version. Nothing in FROZEN is damaged.
  - CSD has never been the sending end of a CSS stream: 75's sender ran on TRJ.
- **Cheap checks before `freeze-transfer`, none of them jobs:**
  - **Clean stdout.** `ssh csd true | wc -c` must print `0`. Any shell banner or rc output on stdout would corrupt the stream. Check the CSS side's non-interactive `ssh css true` the same way.
  - **Identical manifest copy.** Confirm the CSS copy of the runtime manifest has the same sha256 that `send --manifest-sha` will use. `freeze-transfer` and `receive` never compare it with the CSD file. A differing copy fails only at the first file-hash mismatch, after the name is already spent.
  - **No overwrite.** Write `manifest` output with the shell's noclobber (`set -C`), so an existing manifest is never overwritten. Each run embeds a new `utc`, so a re-run is never byte-equal.
- **Optional.** Land the stream as one file on CSS first. Check its byte count against the manifest's `bytes`, then feed the guarded `receive` with `< file`. A network failure then costs nothing.
  - **Cost.** About 310 MB of transient space, outside the guard's accounting. CSS has 24 GiB free.

**L3. `delivery` hashes every narration entry, with no file test.**
- **Why.** `:399–403` hashes every name in the rebind269 narration folder. Delivery76 filtered with `is_file()`. A subfolder would raise there and spend `css-repair-delivery290`.
- **Likely flat.** Assembly19's folder is flat: 105 WAVs, plus `assembly.json`, `external-units.json`, `phrase-timings.json` and `report.json`, which is 109 entries (runtime75's manifest). CC1 counts 109 files for rebind269. I have not seen its listing.
- **Before launch-delivery (root).** Confirm the folder has no subfolder with one read-only listing on CSD.

**L4. One chapter, one host, by assignment only.**
- **Why.** 287 on CSD never sees CSS jobs. 290 sees CSD's 244/278/287 jobs only at `manifest --permit` time (`:198–200`), and one permit file serves both hosts. A chapter could therefore render on both.
- **Fix (root).** Before issuing a chapter's permit, record in writing which host renders it, and do not run that chapter's permit on the other host.
- **Optional hardening for a later version.** Require a `host` field in the permit: 290 accepts only `css`, and the next 287 accepts only `csd`.

**L5. The nine 274 chapters need more than the `PREVIEW274` names on CSS.**
- **Why.** `JOB_DIRS`, `CSD_MANIFESTS` and `runtime_sources()` (`:79–81`, `:136–143`) carry only preview243.
- **What a version for any of the nine must add:**
  - **`PREVIEW274` values**, from review 284's final287 table: job `csd-review-repair-preview274`, manifest `review-repair-preview274-inputs.json`, worker `cc3-preview-fade274.py` at `d80cf7b6…`, tables `preview`, build268, binary 268, identity narration mapping;
  - **in the transfer and the carried rows:** that job folder, that manifest and that worker.

  Until then, CSS can render only 06/08.

## Notes

**N1. Binary 268 first runs on CSS inside `delivery`.**
- **Where.** `delivery` runs `facts` and `chapters` (`:408–411`) before `narration_delivery.py`.
- **What that catches.** A shared-library or toolchain mismatch would show there, before any render. Production19 came from the same build host and ran on CSS in 78/96.
- **Also caught there.** If any of the 174 uncopied `production-repo07` files turns out to be read, the delivery gate stops the run there too, before a render.

**N2. Carried rows bind CSS's system tools to CSD's.**
- **What is compared.** `carried_rows()` (`:426–439`) compares every timeline269 and preview243 row present on CSS with CSD's hash. That includes Python 3.14, numpy, scipy, ffmpeg and ffprobe. CC1 found 0 differences today.
- **Consequences.**
  - A package update on CSS before launch refuses every launch. That is intended, but CSS's owner should know.
  - Matching rows also show that CSS's ffprobe and ffmpeg are the same builds as CSD's, at the same paths.

**N3. Resources are adequate.**

| Item | Value |
|---|---|
| Free space | 24.25 GiB (CC1, today; fleet286 showed 26.7 GB at 05:54Z) |
| Guard | admits at 10 GiB, stops at 8 GiB, 4 GiB per job, one native lane |
| Transfer | about 0.6 GiB; the 310 MB staging copy stays in the transfer job folder by design |
| Delivery | about 0.1 GiB |
| 4K chapters | small (78/96 grew 0.04–0.06 GiB) |
| Target growth | about 1.6 of 20 GiB after the transfer |
| Threads | 16, the first 16 of the affinity, beside other processes at a load of 15–17 on 64 CPUs |
| Render timeout | 12 h, as in 78 |

**N4. FROZEN now gains post-75 files.**
- **What changes.** `frozen-runtime75` will hold files placed after 75. Existing files are never changed: every write is exclusive, and an existing destination must already match. The alias stays exact.
- **The record.** `transfer-result.json`'s `written` flags record which files 290 added. Cite those flags, not the folder name, as the record of what is in FROZEN.

## Checked and found sound

- **Paths.**
  - Every destination is FROZEN plus the CSD absolute path, under the D-only assertion, with no `..`.
  - The alias check (`:114–115`) is the same one delivery76 passed. That shows R has no symlink in its path, so `PERMITS` resolves strictly and `absolute_permit()` can pass on CSS.
  - Binary 268's only embedded runtime path, the workspace topic, resolves through the alias into the transferred workspace.
- **Transfer protocol.**
  - `send` re-hashes every file before the first byte and again while streaming.
  - `receive`:
    - checks every destination before reading;
    - stages new files inside the job folder;
    - refuses a truncated stream and trailing bytes;
    - places files with `'xb'` and keeps their modes;
    - writes `transfer-result.json` with `'x'`.
- **Gates.**
  - Every job named is admitted with policy 10/8/4. CSS receipts record exactly `admission_gib` 10, `stop_free_gib` 8 and `phase_growth_gib` 4 (local `css-evidence78/` receipts).
  - `success()` uses the 287/78 semantics.
  - Inputs are frozen with `'x'`, and every bound file is re-hashed before and after the render.
  - The no-duplicate check refuses the job's own name and any other CSS final job for the chapter.
- **Delivery.** It keeps delivery76's narration copy and source rebinding, with stricter equality. The 105 WAVs must equal assembly19's, and the native gate must report 105/105 `external-ui`.
- **Render.**
  - The command line equals 287's.
  - `verify_tables` is 78's logic, with the CSS narration copy in place of rebind269's folder.
  - It checks with ffprobe that there is one MP4, plus the x264 `crf=16.0`, the soundtrack length and the repeated `check()`.

## Limits

This review read the helper, the notes and the completed CSS records (75/76/78/96/97, the CSS guard and fleet286). None of 290's modes has run anywhere.

Nothing here establishes a compiled, rendered, pixel, motion, audio, listening or host equivalence. A CSS chapter would still be only a technical scored 4K60 chapter. Export, master, final pictures, motion, audio, captions, fine sync and the independent final AV check all remain.
