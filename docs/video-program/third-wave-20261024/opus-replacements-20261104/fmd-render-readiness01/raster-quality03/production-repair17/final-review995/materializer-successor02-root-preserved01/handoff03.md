# scope02 review, F1 fix and the four 09 instants (CC0, final4k-impl02)

CC0 (LavenderPike), 2026-11-11T00:13Z.

**Scope:**
- Text reads and local static checks only.
- Nothing launched, built, rendered, decoded or extracted, and no production or CSD file touched.
- No audio heard or checked.
- No picture, motion, AV, caption-playback or publication acceptance. No visual verdict on the 08 anchors.

## 1. Root's scope02 helper and controller: no concrete defect

**Files:**
- `csd-combined01-pictures-scope02.py` (`baab097a…`)
- `csd-combined01-pictures-after-preview-scope02.py` (`e7ebe2bd…`)

Both are in `final-review995/combined-pictures-root01`, beside the unchanged originals (`43508444…` and `77163ab9…`).

**Helper:**
- The diff adds `if chapter=='08_loop_sums': chosen.update(range(10200,10380))` and changes the sampling string. Nothing else changes.
- By AST, every other function and every module statement is identical. 01 and 09 select exactly as before.
- The selection only grows: the union is taken before the layout loop, and `set(oldframes)<=chosen` still holds.

**Controller:** it changes only the helper name, both in the bound rows and in the guard command. JOB, INPUTS, PRIOR and the `combined-pictures01` binding are unchanged.

**08 selection, recomputed from the frozen plan file** (`6b70dab9…`, 407 frames):
- The range adds exactly 37 frames, as two runs: 10200–10215 and 10359–10379.
- 10380 comes from plan02, so all 181 frames of 995's inclusive window 10200–10380 (`range(a, b+1)` at 995:765) are selected.
- With layout anchors at 10788 and 12336, the old and new selections are 447 and 484. These match root's counts.
  - Anchors at 10788 alone would give 428/465; anchors at 12336 alone would give 426/463.
  - CC2 read the live layout (`c5347cb9`) and found anchors 10788, 12336, 12336. So both lint anchors are in the selection.
- 484 frames is far below the 4000 cap.
- No job-name knock-on: the gate draft's `pictures` job name (`csd-native-combined-pictures01`) stays right.

**L1 (wording only; no change needed now).** The sampling string says `10200:10380`. Read as a Python slice, that excludes 10380; frame 10380 comes from plan02. If root copies the string into notes, write "10200–10380 inclusive (10380 from plan02)".

**Still needed after extraction.** The 08 admission's `review_scope` can name an interval containing [10200, 10380] only once root has viewed all 181 of those frames. Then the gate's `REQUIRED_SCOPE` and 995:1102–1104 pass.

## 2. CC2 stage-review01 F1 fixed: `cc0-native-materialize-repaired02.py`

| File | sha256 |
|---|---|
| cc0-native-materialize-repaired02.py | `e5abe22e…` (full hash in the Mail) |
| cc0-native-materialize-repaired02-from-impl01.diff | `91c48c0e…`: +7 −5, the F1 change only |
| cc0-native-materialize-repaired02-from-991.diff | `dec85ebd…`: +95 −32 |
| checks02.py / checks02.out | all checks passed |

**The change.** `part = 'stems/'+CHAPTER+'.music.f32.wav.part'` becomes `prefix = 'stems/'+CHAPTER+'.music.f32.wav'`; the filter stays `endswith('.wav')`. The docstring is updated to match.

**Unchanged:**
- 991's 09 literal assertion (by AST);
- the cuelog binding loop, including `set(stems) == stem_names`;
- every other statement (by AST);
- the gate draft (`2a855393…`), with every CHAPTERS field equal and 5 pins.

**Checked on the actual score21 sets.** I ran the exact selection statements, cut from each file by AST, on the score21 summary object in root's terminal record `score21-terminal01.json` (`746ca65c…`, 44 files):

| Chapter | impl01 | impl02 |
|---|---|---|
| 01 | refuses (F1 reproduced) | `01_the_bump.music.f32.wav` |
| 08 | part0000–0002 | part0000–0002 |
| 09 | part0000–0002 | part0000–0002, equal to 991's literals |
| 11 | part0000–0001 | part0000–0001 |

- No `.parts.tsv` and no `stems/qa/*.json` is taken.
- CC2 read the CSD `score21-summary.json` itself for 01. I read only root's local terminal record.

**Hygiene.** The 991 diff quotes 991's removed scope line, which contains a flagged word. That is pre-existing text; the new file says "hearing".

**F3 and F4 are root's.** I have made no change for them.
- F3: root authors each admission, `review_scope` included, before that chapter's final, then freezes the exact names. Whenever root pins `ADMISSIONS[ch]`, the names must equal CC3's runner names.
- F4: both binary pins are set before the runner is frozen.

## 3. The four 09 playback instants (for the 995 successor)

**Source.** `completed-master993-summary.json` (`38b52105…`), `rows[09_boundary].review_scope.playback_seconds`. That row's disposition lists `root-preview313-playback343.json` (`d9f250ec…`).

**Where they are.** 09 starts at master frame 67065. 995's rule (lines 755–762) samples centre `round(s*60)` and 2 frames either side, with a native 4K frame at the centre.

| s (exact) | 09 centre | samples | master frame | master time |
|---|---|---|---|---|
| 27.68726914660832 | 1661 | 1659–1663 | 68726 | 19:05.433 |
| 31.90054923413567 | 1914 | 1912–1916 | 68979 | 19:09.650 |
| 35.51193216630197 | 2131 | 2129–2133 | 69196 | 19:13.267 |
| 39.12331509846828 | 2347 | 2345–2349 | 69412 | 19:16.867 |

**Why the 995 successor must list them itself.** Under M1 the new 09 row's scope has `playback_seconds == []`, so these instants no longer arrive through the row. The successor should keep these four values as a constant, cite 343 as the source, and feed the existing loop unchanged.

**The same moments in the narration.** 995's `schedules()` binds 09's tables to timeline269. CC2 found the timeline01 09 tables equal to them byte for byte. So each centre falls at the same point in the narration as before.

**Not in the preview extraction.** These frames are not in plan02's 09 list. Of prior775, only 2129 and 2130 are present. So the preview pictures do not cover them; only the final sampling in 995 does.
