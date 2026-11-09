# CSS final-render helper 309: notes and exact changes against 290

Prepared only. Nothing was deployed, transferred, launched, rendered or written on CSD or CSS. All
remote work was read-only SSH. No permit was written, issued or carried. No TODO, Beads or shared
file was edited. Narration is ElevenLabs Frederick Surrey, unchanged: the same 105 WAVs.

Terms, as in the 290 notes:
- D is the CSD workspace `/data/projects/jmev-stroke-probe-20261106`; on CSS, D is the existing alias into `R/work/frozen-runtime75`.
- R is the CSS workspace `/data/projects/jmev-css-render-worker-20261107`.
- Q is CSD's `D/qualification01/production-repair17`; CQ is `R/qualification01`.

| File | SHA-256 | Role |
|---|---|---|
| `css-final-repair309.py` | `579fe7cee38af0d9fb249621f53b631645180b5799fc323e6a1ddd157814d18a` | new; 813 lines; parsed; exercised as below |
| `css-final-repair290.py` | `720e63a66d9e67d04c1cd99d2131d76401e42a5e2d60240fb1afc9d3a93ac8f2` | base; unchanged |
| `cc2-css-worker-review300.md` | `dfc16db3de7deff440752e378cc407fb1672e1c8419945b3b857d5ea94c881ec` | read in full; L1–L5 addressed below |
| `csd-reviewed-final303.py` (CC0) | `db3ce8e424f17a604e5f2cc802db81e5dcd7b631301808625e20a7c9b658d057` | read in full; not changed; not on CSD yet |
| `csd-launch-final303.py` (CC0) | `def224f04d86cf34db3c8af27ea10109378573cce2852201da3bd74440272cb0` | read in full; not changed |

## 1. What 309 takes from CC0's 303

I diffed 303 against 287. 309 takes 303's preview contract value for value, and changes nothing in 303.

- **PREVIEW274.** The values match 303 exactly:
  - job `csd-review-repair-preview274`, manifest `review-repair-preview274-inputs.json`, `tables='preview'`;
  - worker `cc3-preview-fade274.py` at `d80cf7b6…`;
  - narration is rebind269, so the table mapping is the identity;
  - binary 268, build268, source copy `81d28272…`;
  - `inputs_sha256` `20d91bdb…`.
- **PREVIEW243.** It gains 303's pin, `inputs_sha256` `6510db78…`. My earlier local copy of that manifest has the same hash.
- **`preview_binding()`.** This is now 303's. The `None` guard is gone, and the receipt's `inputs_sha256` must equal the pin.
- **Host.** 303 requires `host == 'csd'` and 309 requires `host == 'css'`, so a single permit cannot pass on both hosts.
- **Launch.** As in 303's launcher, `--permit-sha256` is required, and launch refuses too few CPUs before any inputs file is written.

I measured these myself on CSD, read-only:
- **The preview274 job.**
  - Its receipt (`505b12ce…`) shows exit 0, finished 10:50:17Z, and inputs equal to final inputs (`20d91bdb…`). The resource-stop and error fields are empty.
  - Its command holds the worker's Q path.
- **Its manifest.**
  - It hashes to the receipt's inputs.
  - All 12,669 of its rows still match the CSD bytes.
  - It holds binary 268, the worker and build268's `source-copy.json`, `qualification.json` and receipt at their current hashes.
- **The job folder.** 1,211 files, 396,199,723 bytes, no symlinks.
- **The tables.** All 27 tables in `preview/` (nine chapters, three kinds) are byte-equal to timeline269's.
- **The summary.** `summary.json` (`b4e2cb79…`) names binary 268. Its flags are G3 (11, NOT LOCALIZED) and I5 (14).

## 2. Exact changes against 290

The diff `diff -u css-final-repair290.py css-final-repair309.py` has 52 hunks, with 153 lines added and 66 removed. These are byte-identical to 290:
- `verify_tables`, `required`, `manifest_rows`, `destination`;
- `frozen_inputs`, `write_inputs`, `admitted`, `success`, `alias_is_exact`, `absolute_permit`;
- `launch_delivery`;
- in `render`, the render command, the probe and the encoder and soundtrack checks;
- every guard and policy value.

`carried_rows` changes only in its docstring.

| # | Where | 290 | 309 | Why |
|---|---|---|---|---|
| 1 | names | `…290` jobs, inputs and `repair290-permits` | `css-repair-transfer309`, `css-repair-delivery309` (+ its `-inputs.json`), `repair309-permits`, `css-reviewed-final-<ch>309`, `final-<ch>-inputs309.json`, `css-repair-permit309-<ch>…` | Disjoint from 290 and from root's outputs. None of these exist on CSS today. |
| 2 | `PREVIEW274`, `PREVIEW243` | `None`; no pin | 303's values and pins (§1) | Review300 L5; the 303 contract |
| 3 | `JOB_DIRS`, `CSD_MANIFESTS`, `runtime_sources` | preview243 only | Adds the preview274 job folder, its manifest and its worker | L5. 303's launcher binds the whole preview job folder and every manifest row. |
| 4 | `runtime_problems` | preview243 checks only | One loop over both previews: pin, manifest equals receipt, worker in command and at its hash, binary, build source copy, and that preview's chapters' tables against timeline269. Carried rows for preview274: binary 268, the worker and build268. | L5. Fails before any manifest is written. |
| 5 | `permit_sources` (CSD) | Refuses CSD 244/278/287 jobs | Requires `host == 'css'` and a permit name that starts with `<chapter>-` and ends in `.json`. Refuses any `csd-reviewed-final-<chapter>*` job, which covers 303 and later. | L4, L1, and no duplicate jobs |
| 6 | `manifest` | Prints to stdout | `--out` is required: a new `.json` directly in Q, opened with `x`. It prints the file's sha256. | L2 noclobber |
| 7 | `land-manifest` (new, CSS) | — | Reads the manifest from stdin, up to 64 MiB. It writes nothing unless the whole stream hashes to `--manifest-sha` and has version 309. Then it writes once into CQ with `xb`. | L2: a clean stdout and an equal copy are proved before any job name is used |
| 8 | `send` / `receive` | Raw concatenation | `send` writes a header, `css-final-repair309 manifest <sha>\n`, first. `receive` requires exactly that header for its `--manifest-sha` before it stages anything. | L2: the CSS copy and the stream must both be the manifest CSD hashed |
| 9 | `freeze-transfer` / `receive` | No sha argument | `--manifest-sha` is required. The CSS copy must hash to it, and so must the frozen inputs entry. The printed receive command carries it. | L2 manifest equality |
| 10 | `receive`, `manifest_kind` | An existing permit destination with equal bytes is accepted | A permit destination must not exist, so each permit file is written once. The name rule is checked again. | L1: a re-issue needs a new file name. Same bytes cannot be silently reused. |
| 11 | `delivery` | `iterdir()` hashes every entry | It requires a flat folder of regular files, no folders or links, before `copytree`. It checks the copy the same way. | L3 |
| 12 | `check()` | — | Requires `host == 'css'`. | L4 |
| 13 | `launch` | Permit must be under PERMITS and in its transfer's result | `--permit-sha256` must match. The name rule applies. The transfer result row must say `written: true`. CPUs ≥ `--threads` is checked before inputs are written. | 303 launcher parity; L1 |
| 14 | `preflight` | — | Preview274 rows with pinned hashes. Optional `--manifest` and `--manifest-sha` add a "manifest copy" row. | L2 |
| 15 | `render-summary.json` | 290 hashes | Adds `worker309_sha256`, the 290, 287 and 303 hashes and `preview_inputs_sha256`. `host` is `HOST`. | Provenance |

**Why permits are written once.** On CSS, FROZEN mirrors CSD paths, so changed evidence at an old CSD path is still refused, as review300 said. Root must write new evidence at new paths.

## 3. Copy set, measured now

The runtime copy is one job, `css-repair-transfer309`: 2,616 files, 727,159,338 bytes. This is `runtime_sources()` hashed by its own `manifest_rows()` on CSD.

| CSD source | Files | Bytes |
|---|---:|---:|
| `D/jobs/csd-review-repair-preview274/**` | 1,211 | 396,199,723 |
| `D/jobs/csd-review-repair-preview243/**` | 1,064 | 149,790,898 |
| `D/jobs/csd-review-repair-rebind269/**` | 117 | 95,930,289 |
| `D/jobs/csd-review-repair-timeline269/**` | 65 | 37,662,407 |
| `D/work/bin/` (268, 238) | 2 | 36,917,280 |
| Q: timeline269, preview243 and preview274 manifests, the two preview workers, binding282 | 6 | 8,500,826 |
| `D/work/review-repair268/workspace/**` | 72 | 2,035,988 |
| `D/jobs/csd-production-timeline21/` tables | 45 | 42,081 |
| `D/jobs/csd-review-repair-build268/**`, `…build238/**` | 18 + 16 | 41,475 + 38,371 |

**Disk use.**
- **Expected growth.** About 1.35 GiB: the files are staged in the job, then placed. That is within the 4 GiB per-job cap.
- **Free now.** 35.4 GiB on CSS.
- **Not counted here.** The guard's own 20 GiB target-growth check.

**Preview274 rows already on CSS (read-only).**
- 2,815 of its 12,669 rows are already on CSS, and 0 differ. They include numpy, scipy, python3.14, ffmpeg, ffprobe, `production-repo07`, assembly19 and timeline21.
- The 9,854 absent rows are the same kind of non-runtime rows that 290 §3 describes.

## 4. Order of operations for root

None of this has been run. Every step is root's decision.

1. **Place the helper.** Put `css-final-repair309.py` with identical bytes in Q and in CQ.
2. **Runtime manifest.**
   - **On CSD:** `… css-final-repair309.py manifest --out Q/css-repair-runtime309-manifest.json`. It prints the file's sha, `<S>`.
   - **Land it on CSS:** `ssh csd 'cat Q/css-repair-runtime309-manifest.json' | ssh css '/usr/bin/python3 -I -B CQ/css-final-repair309.py land-manifest --out CQ/css-repair-runtime309-manifest.json --manifest-sha <S>'`. A banner on stdout makes this refuse without writing.
3. **Freeze the transfer.** On CSS: `… freeze-transfer --job css-repair-transfer309 --manifest CQ/… --manifest-sha <S>`. It prints the guarded receive command.
4. **Relay.** `ssh csd '… send --manifest Q/… --manifest-sha <S>' | ssh css '<printed command>'`.
5. **Delivery.** On CSS, run `… preflight --manifest CQ/… --manifest-sha <S>`. Only the delivery job and permits should still be missing. Then run `launch-delivery`.
6. **Each chapter root assigns to CSS:**
   - **Write the permit.** Root writes it on CSD with `host: "css"`, named `<chapter>-<anything>.json`, for example `07_one_point-css-permit01.json`.
   - **Its manifest.** `manifest --chapter <ch> --permit <it> --out Q/css-permit309-<ch>-<n>.json`, then `land-manifest`.
   - **Transfer.** `freeze-transfer --job css-repair-permit309-<ch-with-hyphens>[suffix]`, then the relay.
   - **Launch.** `launch --chapter <ch> --permit CQ/repair309-permits/<name> --permit-sha256 <its sha> --permit-transfer <that job> [--after <job>]`.

**Which chapters can launch.** Only `06`/`08` (preview243) and the nine 274 chapters, and only after root writes their actual picture and motion permits. The CSD permits for 303 must name `host: "csd"`.

**Chapter 09 (root, 2026-11-08).** Actual 09 needs CC0's narrow connector correction 310, so root will issue no 09 permit on binary 268.
- 309 has no code-level 09 refusal. The only gate is that no permit is written for it, so nothing should be written.
- A 310-corrected 09 means a new binary. 309 pins binary 268, so 09 then needs a new helper version.
- The other chapters stay on 268, pending coverage and motion review.

**Frozen.** 309 is frozen at `579fe7ce…`. Root reviewed this exact delta, so any change will be a new version with a new file.

## 5. Checks run

- **Parse.** `ast` parse of 309: pass.
- **CSD, read-only harness.** This was the 309 helper with its `__main__` replaced by a tail, sent on stdin. It wrote nothing.
  - `runtime_problems()` returned **0 problems**, with nothing relaxed.
  - `manifest_rows(runtime_sources())` gave 2,616 rows and 727,159,338 bytes.
  - Every chapter's preview tables are present.
  - The rebind269 narration is flat: 109 regular files.
  - No `csd-reviewed-final-*` job exists for any of the 11 chapters.
- **CSS, read-only `preflight`.** The helper was sent on stdin, so nothing was placed.
  - It exits 1.
  - Missing is exactly the copy set in §3, plus `css-repair-transfer309`, `css-repair-delivery309` and `repair309-permits`.
  - Present: the alias, guard, runtime75, encoder stop 79, `production-repo07`, assembly19, and score21 (44 files at their hashes).
- **Clean stdout.** `ssh csd true | wc -c` = 0 and `ssh css true | wc -c` = 0.
- **Local scratch tests, 28 of 28 passed.** All host paths pointed into a new scratch folder.
  - **`permit_sources`:**
    - host `css`: accepted;
    - host `csd`, or no host: refused;
    - a name without the chapter: refused;
    - an existing `csd-reviewed-final-07-one-point303` job: refused; accepted again once that job was moved aside.
  - **`manifest --out`:** the printed sha equals the file; an existing file and a path outside Q are both refused.
  - **`send`:** a wrong sha is refused.
  - **`land-manifest`:** a stream with a login banner is refused and nothing lands; a clean stream lands; landing again is refused.
  - **`preflight`:** the manifest-copy row shows `present`, and `DIFFERENT` for a wrong sha.
  - **`freeze-transfer`:** a wrong sha is refused; the printed command carries `--manifest-sha`.
  - **`receive`, refused, with nothing placed in FROZEN:** a sha that is not the frozen one, no header, a banner before the header, and a truncated stream.
  - **`receive`, other cases:**
    - full: placed, with the file mode kept;
    - repeat: accepted, already present;
    - trailing bytes: refused;
    - tampered destination: refused.
  - **Permit transfers:**
    - the first one writes the permit;
    - the same permit again is refused;
    - in `manifest_kind`, a permit named without its chapter is refused.
  - **`launch`:** a wrong `--permit-sha256` is refused. The right one passes the permit and written checks and stops later, in `check()`, on the absent binary.
  - **`delivery`:** a narration folder with a subfolder or a symlink is refused, and nothing is copied; a flat folder passes the check.

## 6. Not established, and open gaps

**Not run anywhere.** `check()`, `launch`, `render`, `delivery` and `launch-delivery` have not run on real inputs. They need the copied runtime and a root-written permit.
- No permit exists.
- I wrote none, and none may be self-issued.

**No equivalence is claimed.** That covers compiling, rendering, pixels, motion, audio and listening between hosts.

**One chapter, one host, with one gap.**
- Today the host field binds both ways only through 303.
- 287's worker, which is on CSD, does not read `host`.
- So root should launch CSD finals only through 303, and give each chapter to one host.

**Root's open review items for these chapters.** These are from `picture-review-progress305.json`, 17:57Z: all 129 sheets (996 pictures) of the nine chapters were viewed. Motion review, listening, fine sync and final AV acceptance are all false.
- **07: HOLD.** CC3 counted 142 empty frames after the fade, before the next sentence. A source-aware disposition is pending.
- **14: I5.** A progressive-reveal disposition from CC3 is pending.
- **11: G3.** The NOT LOCALIZED fit has review308 pending. The final card is sampled only to 162.183 s of 198.85 s.
- **09.** Samples jump from 2.6 s to 138.883 s, so the Cauchy proof interval is not reviewed.

A CSS chapter that passes every gate is still only a technical scored 4K60 chapter. Still to do:
- full decode, master replay and export (80/97 pattern);
- the final picture and motion review;
- continuous listening and fine sync;
- the independent final AV review, captions and publication.
