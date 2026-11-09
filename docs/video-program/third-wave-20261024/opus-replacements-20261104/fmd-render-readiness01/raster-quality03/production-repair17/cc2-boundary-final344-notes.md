# CC2 boundary final 344: the 09_boundary 4K60 worker on a sampled-preview disposition

cc2, 2026-11-08. Files:

| File | sha256 | Lines | Role |
|---|---|---|---|
| `Q/csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | 354 | guard-run worker: `check()`, `run()` |
| `Q/csd-launch-boundary-final344.py` | `70278078c8fd5e1cb414afcf02c3ff107a9dd956f0b2551a1ca0bdca360d75b4` | 110 | root-only launcher |
| `Q/cc2-boundary-final344-notes.md` | reported in the pane | | these notes |

D is the CSD project root and Q is `D/qualification01/production-repair17`. Line references are to 344.

**Status.**
- **Prepared only.** Nothing was launched, deployed, permitted, imported or rendered. No remote command was run, and nothing was written outside the three files above and my scratch area.
- **No acceptance claimed.** 344 claims no continuous motion, listening, fine-sync, final-picture, 4K, caption or AV acceptance.
- **Not publication authorization.** 344 authorizes one technical 4K60 render of 09 and nothing beyond it.

**Preserved unchanged** (hashes checked when 344 was frozen):
- `csd-reviewed-boundary-final317.py` (`19abeee9…`), `csd-launch-boundary-final317.py` (`47e0f30a…`) and their notes;
- `cc0-combined-master318.py` (`478b44a5…`);
- 339 (`1e200220…`, notes `60d97ad6…`);
- 330, 321, 311, 309 and 303;
- root's 341, 343 and 345, and CC3's 340.

## 1. Why 344 exists

**The 317 gate.** 317's `check()` required `actually_reviewed_motion: true` (317:166), plus `motion_intervals` (:169, :178), one of which had to contain the zoom exit frame (:145).

**What the reviews establish.** The actual preview313 reviews do not support that boolean:
- **CC3 340** (independent) read all 40 sheets and 14 full-size pictures, and checked a set of frames it extracted itself. It accepts no motion between samples (§6c).
- **Root 341** viewed all 313 sampled pictures on the 40 sheets: `motion_acceptance: false`.
- **Root 343** made QuickTime playback screenshots at four instants: `continuous_motion_acceptance: false`.
- **Root 345** is a disposition for a technical 4K render only. It cites 340, 341 and 343 by hash and accepts L1 and L2 for the render, with native-4K checks still to do.

**What 344 does.** 344 states exactly what those records establish, and authorizes the render from that. The motion boolean and `motion_intervals` are gone, replaced by `review_scope`.

## 2. The diff against 317

**Diffs.** The unified diffs are 129 changed lines for the worker and 31 for the launcher. Most of that is docstring and comments. I kept them in scratch (worker diff `494a9b7b…`, launcher diff `ee90fc7c…`). By AST:

- **Worker functions changed:** only `check`, `preview_binding` and `run`.
- **Worker functions added:** `sampled_review_scope` and `disposition`. None was removed.
- **Unchanged and AST-identical to 317:** `sha`, `read`, `success`, `job_name`, `absolute_permit`, `completed`, `build313`, `narration313`, `schedule313`, `independent` and `verify_tables`.
- **Worker constants changed:**
  - `VERSION` `'317'` → `'344'`;
  - `EARLIER` gains `'317'`, so an existing `csd-reviewed-final-09-boundary317` job is refused (root decides, as for 303).
- **Worker constants added:** `INDEPENDENT340`, `PICTURES341`, `PLAYBACK343`, `DISPOSITION345`, `RECORDS` (their pins) and `PERMIT_KEYS`.
- **Launcher.** Only `main` differs. Two statements change, and only `317`→`344`: the worker file name (:38) and the `VERSION` assertion (:46). The rest is comments and docstring. The guard command, single native lane, 16 CPUs, `'x'` inputs and closure are 317's.

**`check()`:**
- New first permit gate: `set(review) == PERMIT_KEYS` (:266).
- `actually_reviewed_pictures is True` is kept. The motion half of 317:166 is removed.
- `review['evidence']` must be non-empty (was `… and review['motion_intervals']`).
- 317:178 (motion bounds) is removed. `sampled_review_scope` checks the bounds instead.
- Every other line is 317's.

**`preview_binding()`.** Everything up to the sampling and zoom-probe evidence check is 317's. Then:
1. `sampled` is the set of frame numbers in preview313's own `09_boundary/sampling.json` (313 unique frames).
2. `sampled_review_scope(review, FRAMES, sha(movie), sampled)` runs (§3).
3. One of the returned intervals must contain `zoom_probe_exit_frame`. This replaces 317:145, and the exit frame is 2094 in the real summary.
4. `disposition(review, movie, sampling)` runs (§4), and its four record paths are added to the returned binding. The launcher therefore freezes them as inputs rows, and `run()` requires them bound.

**`run()`.** Only `result` changes. It gains:
- `review_scope` (the permit's);
- `disposition` (the four records with their pins);
- a new `scope` sentence: technical 4K60 only, from a sampled-preview disposition, with final-picture, motion and audio review open, as are full decode, master replay, captions and publication; not publication authorization.

Every key 317 wrote is still written with the same value. The render command, encoder, ffprobe, soundtrack and re-check gates are 317's byte for byte.

## 3. `sampled_review_scope(review, frames, movie_sha256, sampled)` (:164–208): the shared check

**What it is.** One function that uses only builtins, its own names and one nested generator. A later combined worker or master copies it unchanged and proves the copy by `ast.dump` equality. Check F6 below shows its free names are all builtins. It refuses:

1. **The motion boolean.** A permit containing `actually_reviewed_motion` or `motion_intervals`, whatever their values.
2. **Any acceptance claim.** Any key, at any depth, whose name contains `accept`, `release` or `publication` and whose value is not exactly `False`. The refusal names the key path, for example `evidence.2.final_acceptance`.
3. **Unstated flags.** A permit that does not state all six flags `False`: `continuous_motion_acceptance`, `listening_acceptance`, `fine_sync_acceptance`, `final_4k_acceptance`, `release_acceptance` and `publication_acceptance`.
4. **A wrong `review_scope`.** It must have exactly these keys:
   - `basis: "sampled_preview"` and `purpose: "technical_4k_render"`;
   - `preview_movie_sha256`, equal to the movie the caller binds;
   - `open_reviews`: unique strings that include `final_picture`, `motion` and `audio`;
   - `playback_seconds`: finite numbers in `[0, frames/60)`;
   - `sampled_intervals`: a non-empty list of `[a, b]` integer pairs (bool refused), sorted and disjoint, with `0 ≤ a < b < frames`.

   **Every frame `a..b` of every interval must be a sampled frame** of the preview: one of the pictures the reviews viewed. The function returns the intervals.

**What it does not do.** It reads no files and binds no hashes. Callers supply the movie hash and the sampled frames from records they bind.

## 4. `disposition()` (:209–227): the exact evidence, 09 only

**Pins.** The four records must be in Q, free of symlinks, at these hashes:

| Record | sha256 |
|---|---|
| `cc3-preview313-pictures340.md` | `7a0c1f9fd38a62c437b66cf1d356ef3d479008070c9ff9af06a51e1d3b6b9c14` |
| `root-preview313-pictures341.json` | `7350b30a4bcc595a1fffd31c7670c18c97ef4dac00045f46ec916c225132dca9` |
| `root-preview313-playback343.json` | `d9f250ec00007fe0687c7bf2a3155652b30df45956c5dfe53e9922dd0ec67e81` |
| `root-preview313-disposition345.json` | `cf42fd8ebd43d4ae30fc24dee9826fcc630d7d9f50aeba9a2290bee489a4f08e` |

**What the permit must bind.** Its `evidence` must bind 341, 343 and 345 at these paths and hashes, and its `independent_review` must bind 340. 317's `independent()` rule still applies: a reviewer other than the permit's, and files in Q.

**What the records must say.** Read on the host:
- 340 names the bound preview movie's hash and its `sampling.json` hash.
- 341 and 343 name the bound movie, and 341 names the bound sampling.
- 341 viewed every sampled picture (313) on every sheet (1–40), and its `concrete_picture_defects_observed` is `[]`.
- 345 says `independent_review_read_completely: true` and has the scope "Technical 4K render disposition only; not release acceptance". Its evidence cites 340, 341 and 343 by hash.
- Every `*accept*` key in 341, 343 and 345 is `false`.
- The permit's `playback_seconds` equal 343's `playback_observed_seconds`.

## 5. The permit root writes (after its own decision; 344 writes and approves nothing)

All paths are absolute CSD paths; here they are written with D and Q. Keys exactly `PERMIT_KEYS`:

| Key | Value |
|---|---|
| `chapter`, `host`, `frames` | `09_boundary`, `csd`, `16504` |
| `binary_sha256` | build313's, from `D/jobs/csd-review-repair-build313/qualification.json`; preview313's summary records `1a82f72c53dba2d5db8a3478bac252e46bf711afd59ec9d425ee6639708611db` |
| `preview_job` | `csd-review-repair-preview313` |
| `timeline_sha256` | the timeline313 09 table; preview313's `sampling.json` records `5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e` |
| `actually_reviewed_pictures`, `unresolved_defects` | `true`, `[]` (345 accepts L1 and L2 for the technical render) |
| `reviewer`, `observations` | root's; the native-4K checks 340 §6(d) and 345 name belong here |
| `evidence` | `D/jobs/csd-review-repair-preview313/09_boundary/sampling.json` (`4e7ab1d8…c0a5`) and `…/zoom-probe.json` (`e2d05d81…2c99`), plus `Q/root-preview313-pictures341.json`, `Q/root-preview313-playback343.json` and `Q/root-preview313-disposition345.json` at the §4 hashes |
| `source_binding_evidence` | as for 317 |
| `independent_reviewer`, `independent_review` | `cc3`, `[{path: Q/cc3-preview313-pictures340.md, sha256: 7a0c1f9f…9c14}]` |
| `review_scope` | the object below |
| the six flags of §3 | `false` |

The `review_scope` object:

```json
{"basis": "sampled_preview", "purpose": "technical_4k_render",
 "preview_movie_sha256": "5e50811b77c075ae638c3d539775ccb29134504152a60f5e327ca28940cb3a1c",
 "sampled_intervals": [[2080, 2109], [2316, 2336]],
 "playback_seconds": [27.68726914660832, 31.90054923413567, 35.51193216630197, 39.12331509846828],
 "open_reviews": ["final_picture", "motion", "audio", "fine_sync", "captions", "publication"]}
```

**Where the intervals come from.** [2080, 2109] and [2316, 2336] are the only runs of more than three consecutive frames in preview313's sampling, so every frame in them is a sampled picture. They are also the windows that 340 §4 ("every frame 2080–2109", "every frame 2316–2336") and 341 describe. The first contains the exit frame 2094.

**The other runs.** These sampled runs are short: 35–36, 65–66, 95–96, 2009–2010, 2069–2070, 2129–2130, 13392–13393, 16473–16474 and 16501–16503. Root may add any of them; nothing outside a sampled run is accepted.

## 6. Root's steps

1. **Copy the four records.** Copy them from the local Q into Q on csd byte for byte, and check the §4 hashes there. 344 refuses any other bytes, and fails closed if a file is missing.
2. **Write the permit** (§5) in Q on csd.
3. **Launch.** Run `python3 -I -B Q/csd-launch-boundary-final344.py --chapter 09_boundary --permit <permit> --permit-sha256 <its hash> --worker-sha256 b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` once, in the one native lane.
   - **Job:** `D/jobs/csd-reviewed-final-09-boundary344`.
   - **Inputs:** `Q/final-09_boundary-inputs344.json`.
   - **Guard:** 317's unchanged `guard_csd02.py` native policy: 10 GiB admission, stop at 8 GiB free, 4 GiB phase growth.
   - **Worker:** `--threads 16`, with the 43,200 s render timeout.
4. **Outputs.** On success the job writes `render/` and `render-summary.json`. Then come root's native-4K picture checks (340 §6d, 345 L1 and L2), full decode, the master, captions, listening and fine-sync. All of these remain open.

## 7. Downstream adaptations (named only; 318, 339, 303 and 309 untouched)

**Master: a successor to 318.** CC0 is writing it as 350.
1. **Pins.** `WORKER317`/`JOB317` (318:157–158) become `Q/csd-reviewed-boundary-final344.py` and `csd-reviewed-final-09-boundary344`.
   - Check 318:561 against them.
   - 318:581 becomes `module.VERSION == '344'`. `CHAPTERS`, `HOST`, `BIN`, `NARRATION`, `TIMELINES` and `job_name` have 317's values.
   - **344 replaces 317 as the only 09 source; it is not an additional one.** 318 expects 317's job, never 303's. 344 refuses to run beside an existing 317 job, and 317 cannot pass with an honest permit.
2. **`permit_gate` (318:338–351).**
   - Add `scope=None`.
   - Keep the boolean, `motion_intervals` non-empty, and the interval bounds only when `scope is None`.
   - Otherwise call a copy of `sampled_review_scope` that is unchanged and proved by AST.

   The exact text tested is in my scratch suite (F9–F12):

   ```python
   assert review['actually_reviewed_pictures'] is True
   if scope is None:
       assert review['actually_reviewed_motion'] is True
   ...
   assert review['evidence']
   ...
   if scope is None:
       assert review['motion_intervals']
       for a, b in review['motion_intervals']:
           assert 0 <= a < b < frames
   else:
       sampled_review_scope(review, frames, *scope)
   ```

   For the boundary row only, at 318:627, pass `scope = (sha(PREVIEW313/'preview'/'09_boundary.mp4'), {x['frame'] for x in read(PREVIEW313/'09_boundary'/'sampling.json')['samples']})`. Retained and combined rows keep `scope=None`. Unadapted, 318's `permit_gate` raises `KeyError: 'actually_reviewed_motion'` on a 344 permit (F8).
3. **`independent317` (318:527–543).** No logic change. The row's `worker_review` must cite 344's sha256 (it hashes the row's worker), and the names and messages say 317.
4. **The re-runs.**
   - `module.check(key, permit)` and `module.preview_binding(key, review)` keep 317's signatures and return shapes. The binding now ends with the four record paths, and both calls return the same list, so 318:637–638 holds.
   - The summary keys 318:617–621 reads are unchanged. A master may also record `summary['review_scope']`.
5. **Docstrings.** 318:4, :19 and :22–23 name 317.

**Combined workers 303/309, if root wants them on the same terms.** `csd-reviewed-final303.py` :126, :129 and :137 and `css-final-repair309.py` :585, :588 and :599–600 hold the same boolean and `motion_intervals` gate. A successor of each would:
- take an exact permit key set;
- call the unchanged `sampled_review_scope` with its own preview's movie hash and sampled frames;
- bind its own root and independent records in place of 344's `disposition()`, which is 09-specific.

Their retained production19 permits are historical and stay on the legacy gate.

## 8. Checks (scratch, local, not in the repo; nothing real ran)

**Suite.** `fixture_test344.py` passed 70/70.
- **Real data:** the synthetic CSD tree of my 317 fixture, but with preview313's real movie (`5e50811b…`), `sampling.json` (`4e7ab1d8…`) and `zoom-probe.json` (`e2d05d81…`) from root's guarded copy. CC3 340 matched that copy to its host read. The real 09 summary row (exit frame 2094) and the four real records were at their pins.
- **Real code:** the real worker and launcher text, with only the host root and the two build268 base hashes changed.
- **Faked:** subprocess, the guard and CPU affinity. No binary, guard or render ran.

| Block | Cases |
|---|---|
| A | positive `check()` (binding holds the four records); 317 gates kept: independent reviewer equals reviewer, binary, pictures false, host css, preview274, timeline269 table, unresolved defect, missing sampling, WAV, cues, binary bytes |
| B | `actually_reviewed_motion` true or false and `motion_intervals` refused; each of the six flags true; a flag missing; `"pending"`; an extra `release_approved`; a nested `evidence.2.final_acceptance` |
| C | basis, purpose, motion or audio not open, wrong movie, interval with unsampled frame 2110, one-per-second span, no interval over 2094, unsorted, overlapping, past the end, bool frame, empty, playback not 343's, NaN, extra scope key; positive with an extra fully sampled run |
| D | evidence lacks 345 or 343, independent review not 340, each record changed with the permit re-pointed at the new hash, existing 317 job |
| E | the launcher writes the 344 inputs (423 rows, including the four records, movie, sampling and probe) and calls the guard once, in 317's form with 344 names; a second launch is refused (`File exists`); a worker at another hash is refused before loading; `run()` refuses inputs without 345; `run()` passes every pre-render gate and issues 317's exact render command |
| F | the AST delta of §2; `sampled_review_scope` has builtin-only free names and works copied alone; 318's real `permit_gate` refuses a 344 permit; the §7 adaptation accepts it with a scope, keeps legacy permits on the legacy gate, refuses a legacy permit given a scope, and refuses a 344 permit without one |

Both Python files were compiled in memory only, so no byte-code file was written in Q.

## 9. Not established

- **No acceptance of any kind.** No continuous motion, listening, fine-sync, final-picture, native-4K, caption, AV or publication acceptance.
- **The sampled intervals.** These are runs of preview313 stills at 540p, which both reviews viewed. They are not playback.
- **On CSD.** 344 has not run there. The four records are not yet in Q on csd. Whether the render fits the guard's 4 GiB growth cap is the same question as for 317, and 344 does not change it.
- **Hygiene of root's records.** Root's 341, 343 and 345 name local home paths in their values. They are root's files: 344 reads only their hashes and fields, and I did not edit them.
