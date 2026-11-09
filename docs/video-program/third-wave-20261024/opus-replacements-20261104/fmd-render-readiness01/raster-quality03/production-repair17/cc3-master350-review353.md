# cc3 review 353: master350 with final344 (09) and the ten 352 chapters

cc3, 2026-11-08, independent of the author (CC0) and of the 352/344 author (CC2).

**Notation.**
- Q is this folder locally, or `D/qualification01/production-repair17` on the build host.
- D is the build host's job root (350's `D`). `CSS` is 350's constant for the root of the second render host.
- "The D worker" is `csd-reviewed-final352.py`. "The second-host worker" is `css-final-repair352.py`.

**What I did not do.**
- I launched, staged, transferred and rendered nothing, and wrote no permit.
- I edited no source, candidate, TODO, Beads or memory file. This file is the only one I wrote.
- I viewed no picture for this review, and I heard and checked no audio.
- Every host step was a read-only Python run over ssh. It wrote no file on the host.

## 1. Verdict

`cc0-combined-master350.py` at `60575944…` integrates the actual final344 09 row and the actual frozen 352 workers for the ten other changed chapters. It keeps every 318 gate:
- source, audio105, schedule, table, score, import, retention, replay, caption and resource gates;
- the master body, unchanged.

It adds an encoder gate for the 09 and 352 rows.

For the two real permits that exist (364 for 01, 351 for 09), 350's own code accepts each one at exactly the scope root stated, and every mutation I tried was refused at its intended check.

**One Medium finding (F1).** 350's refusal of the known 352 self-view compares path strings. A second spelling of the same file, or a byte copy of the sampling.json, gets past it. I reproduced this on the host with the real permit 364 (§6). No actual permit is affected: 364 and 351 both pass the explicit check below. So F1 does not block 364 or 351, but every further 352 permit needs either a successor fix or root's explicit per-permit check.

Not yet checkable end to end:
- 344 was still running at 21:44Z;
- no 352 job, second-host import or row manifest exists yet;
- nobody has run `check --manifest`, `film()`, `contract_reads()`, the real preflight, `launch` or `master` on the host.

## 2. Bytes reviewed

| File | sha256 | Role |
|---|---|---|
| `cc0-combined-master350.py` | `60575944736159139b05733c299b984ab6627ada3d88a5aff65ecde1bbde28d6` | reviewed, 1381 lines, read in full |
| `cc0-master350-notes.md` | `e30db95759918745131e1b53a31e6bc033d20a1125ea835707ff095d6123b151` | read; claims checked against the bytes, not relied on |
| `cc0-combined-master318.py` | `478b44a5787e12fd495ed6312624a66783f90863cb88774775d1bf0c31a55f57` | base, unchanged |
| `csd-reviewed-final352.py` | `23d55c286aba2fc58b77a5af8dbd539293592daa3a4e0e8584f18f324d868760` | CC2, pinned whole by 350 |
| `css-final-repair352.py` | `1074051b6063f1cd400de58422f7491d2d011f764884b63c4968b03feaf01a8a` | CC2, pinned whole by 350 |
| `csd-launch-final352.py` | `06451123b958db9d4806e2645fd84d8e0b37d604867dae9f25d4cb3c5155859e` | CC2, argv read |
| `cc2-final352-notes.md` | `4479363724325af8b0f53417dfe6822df9998634ea968e4213d531d1c6a72fee` | CC2 |
| `csd-reviewed-boundary-final344.py` | `b8458dbcecf3f7a207266dd20195eb3961d82d15a3fc1cc8aa0452db5a2e290b` | CC2, frozen (reviewed in 346) |
| `final-01-css-permit364.json` = `01_the_bump-css-permit364.json` | `299b8fe0db20ca2559ea97845df8a3300c078e92b7d05ab2483f205e43dbff40` | root, the 01 permit (two names, same bytes) |
| `final-boundary-permit351.json` | `873c33bde99650c7f471c690852dff8860c03ef44b9ff8c695ce57788ba7b419` | root, the 09 permit |
| `final-01-stage364.json` | `79b1c15b301d44fc83e27e5717e5b5274a2c25d2d62a425eb6d8775b95fa9f0c` | root staging record |
| `css-permit-transfer364-preparation.json` | `8d873c7c012e0e40da82f2195804c11122e44c7a727d17f347ed8970ae510b3d` | root, manifest `507333c5…` landed only |
| `root-caption347-review354.json` | `749aeba8350d46e339a517af971086bd2d6e2fe039e6501f16e480edd3ad3cb4` | root caption decision |
| `root-preview06-disposition362.json` | `1822a5908199c1cf8ae935e50c803e6a419c0109487c4051bef31d0ef0bf0b86` | root 06 R1/R2 |

My watcher saw the six 350/352 files unchanged for 180 s, ending at 21:49:58Z. The 350 bytes equal the snapshot I began reading at 21:35Z.

On the build host, the staged Q copies of 344, its launcher, the 352 workers, the 352 launcher, permit 351 and permit 364 are at the hashes above. No 352 file is present under the host's view of `CSS/qualification01` yet.

## 3. How I checked

**Source.**
- I read every line of 350.
- From the D worker I read the header, `sampled_review_scope`, `independent`, `review_basis`, `preview_binding`, `check` and `run`.
- From the second-host worker I read the header, helpers, `alias_is_exact`, `independent`, `review_basis`, `preview_binding`, `check`, `verify_tables`, `required`, `launch` and `render`.
- From 344 I read the summary keys; from the 352 launcher, its child argv.

**Delta against 318.**
- A function-level AST comparison, with docstrings set aside.
- A unified diff of every changed function.

**Host.**
- I executed 350's frozen text in memory against the real staged 352 and 344 workers, the real permits 364 and 351, and the real preview movies, samplings, sheets and Q records.
- Every mutation was made to an in-memory copy of a permit.
- The scripts are in my session scratch, not in the repo: `host353b.py`, `host353d.py`, `host353e.py`, and `host353a.py`/`host353c.py` for host state.

## 4. Gate preservation against 318

**AST-identical functions (19).** With docstrings set aside, these are identical:
- `absolute`, `capture_check09`, `completed313`, `contract_reads`, `film`, `flag`, `gates`, `header`, `imported`, `last_capture`, `load`;
- `permit_gate`, `provenance313`, `read`, `replays_of`, `retention`, `schedule`, `sha`, `success`.

CC0 counts 18 because `replays_of` differs in its docstring only.

**Changed functions.** I diffed each one line by line:
- **`master`.** Only the summary dict changes. It gains `review_bases`, four more acceptance flags stated `False`, and a scope sentence. A `claimed()` walk then refuses before the exclusive write. Everything else is the same lines as 318:
  - the hard-link staging;
  - the native `master` then `captions all` loop: argv, cwd `REPO07`, `PATH`/`TMPDIR`/`XDG_CACHE_HOME`, timeouts, log names;
  - `replays_of`, the ffprobe checks and the re-hashing.
- **`preflight`**: the attempt folder name only.
- **`launch`**: two messages only.
- **`check`**: prints the review basis and intervals.
- **`chapter_gate`.** The removed lines are exactly:
  - the 317 pins;
  - 318's prefix match for combined workers, now replaced by whole-byte pins of the two 352 workers;
  - the unconditional `permit_gate()` call, which now runs on retained rows only.

  Every non-motion assertion of `permit_gate()` reappears in `sampled_scope()`: binary, chapter, pictures reviewed, no defects, reviewer, observations, timeline hash, evidence present and at its hashes, and `frames` a positive int. Only `actually_reviewed_motion` and `motion_intervals` are replaced, by the review scope.

**Kept for every row.**
- the receipt's success and its command flags;
- frozen inputs at their hashes, including the worker and the permit;
- render outputs at their hashes;
- `module.verify_tables`;
- `schedule()`: timeline269 tables after the one narration mapping, each line WAV equal to rebind269's, cue-log rows, and the score21 stems;
- the 48 kHz, 2-channel, 16-bit capture of `frames×800` samples;
- the staging device;
- `film()`'s whole-film gates and `imported()` for foreign rows.

**09 only.** `provenance313` and `capture_check09`.

**Retained rows (02, 03, 13, 15).** These are unchanged:
- `RETAINED` pins, `permit_gate()`, `retention()`, `module.check() == (review, sha)` and the movie pin;
- plus one new refusal: a permit carrying `review_scope`.

**New for the 09 and 352 rows.** The x264 settings read from the movie must equal the summary's `encoder` and contain ` rc=crf` and ` crf=16.0`.

**318 review history.**
- 318 and its review records stay as they were: CC0's 318 notes, root's 305 and cc1's 322.
- 350 does not address cc1 322 F1. The delivery contract is preflighted for both consumers, but the native `captions all` still runs only after the native master. That finding is still open.
- 322 N2 is also unchanged. 350's encoder gate is per row, with no cross-row codec-parameter comparison before the join, and it now applies to eleven re-rendered rows.

## 5. The two row kinds against the real interfaces

### 5.1 The 352 workers and 350 agree on every shared name

From host run T0:
- **`sampled_review_scope`.** Its source-segment sha256 is `0b9d0ca5…` in 344 and in both 352 workers. This is 350's `SAMPLED_SCOPE_SOURCE_SHA`.
- **`review_basis`.** Its source sha is `a6c92320…` in both 352 workers.
- **Shared constants.** `MOVIES`, `CHAPTERS` (equal to 350's `CHAPTERS352`), the `PREVIEWS` job names (equal to 350's preview243/preview274 split) and `PERMIT_KEYS` (equal to 344's, and to the key-set pin `b051136c…`) agree.
- **Preview movies.** For all ten chapters, the preview movie on the host hashes to its own sampling.json's `movie_sha256` and to `MOVIES`.
- **`pinned_scope`.** It accepts all three workers. It refuses a worker whose globals shadow a name the function uses (`set`), and a function compiled from another file.

**The second-host worker on the build host.** Its `review_basis` and `preview_binding` read only `D` paths, and its `DQ` is Q, so they need no alias. 350 runs them on the build host and gets the same results as the D worker. CC0 notes N1 is correct.

For second-host rows, 350 does not call that worker's `check()` or `independent()`, which assert the alias. That row rests on four things:
- its job's success receipt (that worker's `render()` calls `check()` before and after rendering);
- every file that job froze being present on the build host at its hash. This means the build-host import must carry the job's whole input closure, as 318 required for the retained 13 row;
- 350's own `independent_files()`;
- the summary gates.

### 5.2 Permit 364 (01, second host), real records

`combined_scope` through the second-host worker accepts it:

| Item | Value |
|---|---|
| Intervals | `[[5493, 5531], [6273, 6310]]`, 77 sampled frames |
| Six flags | stated false |
| Preview274 01 sampling.json (74 frames) | viewed by root's `picture-review-progress305.json`, cited by its sheets |
| Bump transition manifest (74 frames) | viewed by `bump-transition-review305.json`, cited directly |
| Grid sampling (88 frames) | viewed by `cc3-current-grid-review315.json`, cited directly |
| Independent confirmation | cc3 315 |
| Runs / unviewed / sheets / files | 2 runs, none unviewed, 10 sheets re-hashed, 17 files frozen |

Further results:
- `preview_binding` gives 23 paths, the same list from both workers.
- The second-host worker's `review_basis` equals the D worker's, which equals 350's `frame_basis`.
- The D worker on this second-host permit is refused, as one chapter must have one host.

**Mutations of 364, all refused by 350 at the intended check.** Each case notes how 352 alone behaves:

| Case | 350 | 352 alone |
|---|---|---|
| a. preview sampling item views itself (root's view kept) | refused: `a record cannot view itself` | accepted; the sampling is listed as its own viewer |
| b. as a, with root's 305 view removed | refused, same message | `preview_binding` **accepted** (23 paths): its "root evidence views the sampling" rule is met by the sampling itself |
| c. root's view removed, no self-view | refused: root evidence must view the sampling | refused |
| d. grid sampling views itself | refused: self-view | refused: not cited |
| e. same path bound twice, wrong hash first | refused: not at the bound hash | `review_basis` accepts (last entry wins); its `check()` would refuse |
| f. views an unbound digest | refused | — |
| g. `playback_seconds` non-empty | refused | — |
| h. interval past the viewed frames | refused | — |
| i. nested `release_ok: true` in an evidence item | refused: `evidence.0.release_ok` | — |
| j. audio review closed | refused | — |
| k. `actually_reviewed_motion` added | refused: not 344's keys | — |

Case b is the known 352 possibility, and 350 refuses it as written. §6 shows a variant that 350 does not refuse.

**Root's statement checked.** On the host, permit 364 has:
- no self-view;
- no item whose `views` contains its own sha256;
- no duplicate path;
- all ten bound paths canonical, with each path equal to its resolved form.

### 5.3 Permit 351 (09)

`sampled_permit_gate` with the real 344 module accepts it:

| Item | Value |
|---|---|
| Intervals | `[[2080, 2109], [2316, 2336]]`, 51 sampled frames |
| Zoom-line exit | frame 2094, inside the first interval |
| Disposition records | cc3 340 and root 341/343/345, at the pins |
| Playback | exactly root 343's four instants (27.687, 31.901, 35.512, 39.123 s) |
| Files frozen | 8 |

The preview313 09 sampling has 313 samples, all at distinct frames, so 350's uniqueness assert holds.

These are refused:
- playback `[]`;
- 340 removed;
- the exit-frame interval removed;
- 351 on a 352 row;
- 364 on the 09 gate;
- 351 through 318's historical `permit_gate()` (KeyError on the motion flag).

`claimed()` finds nothing in 364, 351, 341, 343 or 345.

**The 09 `worker_review`.** My `cc3-final-render-review346.md` contains 344's full sha256 `b8458dbc…`, so it satisfies `independent344`'s citation rule. It is not in the host's Q yet; the host run reported it missing. Root must stage it unchanged before `check --manifest` (CC0 §9.4).

### 5.4 What could not be checked yet

**Jobs and rows.**
- The 344 job was admitted and not finished at 21:44Z.
- No `csd-reviewed-final-*352` or `css-reviewed-final-*352` job exists.
- There is no second-host import for any 352 row, and no 350 row manifest.

So none of these has met a real 344 or 352 render summary: 350's summary gates, encoder gate, `verify_tables`/`schedule` on 352 tables, the frozen-inputs loop over a second-host job's closure, or the staging device check. I checked the summary keys 350 reads against the summary dicts in 344 `:351`, the D worker's `run()` and the second-host worker's `render()`. Every key exists, including the second-host summary's `worker352_sha256` and `narration`.

**Independent records for the other permits.** The independent records that contain each chapter's preview movie hash today are:

| Chapters | Record |
|---|---|
| 01, 04, 05, 07, 10 | cc3 315 |
| 06 | cc0 270 |
| 08 | cc2 271 |
| 11, 12, 14 | cc1 357 |

Whether each one passes depends on the permit binding it with `views`. Root has not written those permits yet.

## 6. Findings

### F1 (Medium, permit hygiene): the self-view refusal can be bypassed by an alias path or a byte copy

**Where.** `frame_basis` refuses `path == item['path']` (:637). Both are raw strings from the permit. `by_digest` maps a digest to the last bound path with those bytes (:628), the same choice 352 makes.

**Reproduced on the host with the real permit 364.**
- In memory, I removed root's 305 view.
- I added, before the real sampling item, an evidence item for `D/jobs/csd-review-repair-preview274/01_the_bump/./sampling.json` with `views: [the sampling's own sha256]`.
- **Result.** 350's `combined_scope` accepts the permit. The preview sampling is "viewed by" its own alias, so the rule "root evidence must view the sampling" is met by the sampling itself.
- Both 352 workers' `preview_binding` accept it too (23 paths).
- With the alias item placed after the real one, 350 refuses it (`a record cannot view itself`), so the outcome depends on the order of the items.

**Paths that reach this.** The same holds for any second path with the same bytes: a `//` spelling, a symlink, or a byte copy.

Byte copies already exist locally for every chapter: root's review folders.
- `preview274-review305/<chapter>/sampling.json`; for 01 it is `35854fb5…`, the same bytes as the original.
- `preview243-review265/06_one_derivative/sampling.json` (`34ee8c6a…`) and `…/08_loop_sums/sampling.json`.

These are natural files to stage and bind as "root's review", and binding one with `views: [sampling sha]` would pass 350. Today none is on the build host: a search under D's `qualification01` and `jobs` found 11 original preview samplings and no copy.

**Effect.** A 352 permit could reach the master with no root view of the preview's own sampled pictures behind it. That breaks the requirement that 350 refuse the 352 self-view.

**Fix, in a successor (350 is frozen).** Refuse when the viewing item's bound sha256 equals the digest it views. One line beside :637:

```python
assert digest != item['sha256'], 'a record cannot view itself or a copy of itself: '+item['path']
```

This catches aliases, symlinks and copies alike, because a file cannot record viewing a record that has its own bytes. Optionally, also require `absolute(item['path'])` (symlink-free, no `..`) for every bound item, as 350 already requires for independent files.

CC0 confirmed F1 against the frozen bytes (cross-session reply, after this review's first version). CC0 also proposes refusing two bound frame records that share a sha256: `assert len(by_digest) == len(records)`. I agree. That refuses an alias or copy even when nothing views it. 350 stays frozen; any successor is root's call.

This does not change any accepted result, because `frame_basis` runs before the comparison with the worker's `review_basis`.

**Until then: root's explicit check for each 352 permit.**
- No bound item's `views` contains its own `sha256`.
- Every bound `path` equals its resolved form.
- No two bound items share a sha256 unless they are the same path.

364 and 351 pass all three. Points 1 and 2 come from the host run at 21:51Z. Point 3 comes from a second host run at 21:55Z, added after this review's first version (`50185f20…`) claimed it untested: each permit has 10 bound items across evidence, independent_review and source_binding_evidence, with 10 distinct sha256s, each at its stated hash.

### N1 (note): one host per chapter is not enforced across hosts

- The D worker refuses the chapter's earlier finals under D.
- The second-host worker refuses the chapter's other finals on its host.
- 350 gates the row's job fully, but does not refuse a second 352 final of the same chapter on the other host.

This does not change the master's bytes. It would leave two technical finals for one chapter. Before `check --manifest`, root should confirm that each chapter has a 352 job on one host only (CC0 §9.3).

### N2 (note): native captions

- 350 adds no caption code and runs binary 268's `captions all` exactly as 318 did.
- Its srt/vtt are a new native export from 350's staged tables. They are not a copy of native327, and byte equality with native327 is untested (CC0 §7; CC2's 363).
- Root's 354 adopted caption347 as the delivery candidate. That adoption is contingent on equality with the final master's schedule and text, and on final caption playback. That binding is outside 350 and still open.

## 7. Still open after this review

- **Final 4K pictures.** No final frames exist. Still owed:
  - root's 06 checks at frames 600, 900, 1200, 1290, 1320, 1680 and 1890 in native final output (root 362);
  - root's 09 L1/L2 final checks;
  - every chapter's exact final-picture checks.
- **Motion.** Every 352 and 09 basis is sampled stills; 09 adds root 343's four playback instants. Nobody has reviewed continuous motion.
- **Listening and audio.** I heard and checked no audio. 350 states `audio_heard` false.
- **Fine sync.** Open.
- **Captions.** Caption347's binding to the final master, and caption playback.
- **Full decode** of the master (350 states `full_decode` false).
- **Independent final AV.** Open.
- **Release and publication.** Open.
- **On the host, not yet run:**
  - 344 and the 352 renders finishing;
  - the second-host imports;
  - staging 346 (as the 09 `worker_review`) and the second-host worker at `CSS/qualification01`;
  - the 350 row manifest;
  - `check --manifest`, read line by line, then the real preflight, `launch` and `master`.

Nothing in this review is acceptance of a picture, motion, audio, fine-sync, caption, final-AV, 4K, release or publication review.
