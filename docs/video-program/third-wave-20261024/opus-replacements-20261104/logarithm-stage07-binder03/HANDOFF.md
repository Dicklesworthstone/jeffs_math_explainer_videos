# Logarithm stage07 binder, revision 3: the chapter 05–08 teaching repair

BronzeJaguar, for BlueSalmon, on Bead `mx-video-program-program-87do.63`. This binds LavenderPike's sealed `logarithm05-08-teaching-repair01` as you integrated it, using your real receipt `integration-partial07.json`. Narration is ElevenLabs Oliver; no audio is involved here.

**State: a proposed plan, frozen for root, not installed. Nothing here has been compiled, rendered, synthesized, measured or heard. These source checks are not native acceptance, picture acceptance, listening or fine sync.** Canonical, the stage07 candidate, your `stage07/bound-plan`, RubyRaven's preflight and the sealed binder01, binder02, White and Lavender bundles are all untouched.

**The result:**
- **Receipt:** `integration-partial07.json` `4b1751d5…` names patch `1aafaaa8…` and the before and after hashes of the four chapters. All of them agree with Lavender's `SHA256SUMS`, your preserved originals, stage06 and the candidate.
- **Anchors:** 25 → 33. The eight added are Lavender's rows, field for field. All seconds are null.
- **Holds:** the seven 05–08 holds equal Lavender's `hold-table.json`. Only `05_5` moves, by 13 lines. No interval text changed.
- **Waits:** 13 → 16. The three new ones are word-sync waits inside each file's single `until_phrase`.
- **Map:** renamed to **oliver15**, because its content differs from the oliver14 in your `stage07/bound-plan`. You decide the name, and it is one constant (see below).
- **Cue delta:** byte-identical to binder02's. No cue, request or state changed.
- **Takes:** none is selected. Selection for all 80 cues remains pending.
- **Preflight:** binder02's five-line patch serves unchanged. The new helpers needed no new line.

## Run it

From the repo root, where `<F>` is this folder:

```
python3 -I -B <F>/bind_final80-stage07-r3.py --check --out <F>/bound-plan      # validate the plan here
python3 -I -B <F>/bind_final80-stage07-r3.py --out <DIR> --final-out <DIR>     # your own copy
python3 -I -B <F>/bind_final80-stage07-r3.py --print-pins                      # to rebind on purpose
python3 -I -B <F>/checks/run_checks.py <fresh-scratch>                         # all the checks below
```

Exit 3 means an input moved, and nothing was computed. The binder pins 108 files plus the chapter folder's listing. Among them are partial07, Lavender's five sealed files, your four originals in `before-teaching05-08-01/`, and binder02's plan.

**Give `<DIR>` a new folder.** The binder never overwrites an output that differs, so `--out stage07/bound-plan` would print `DIFFERS, not overwritten` and leave your binder02 copy as it is. `--final-out` changes only the dispositions' evidence path and, since it lists their hash, `increment-delta.json`.

## What changed against binder02

`bound-plan/increment-delta.json` lists every change by JSON path. `binder02-to-03.patch` is the whole binder change. The base is binder02's sealed plan, not your `stage07/bound-plan`. The checks show those two differ only in the two `--final-out` files, and binder03 does not read yours.

| File | Hash | Change |
|---|---|---|
| `narration_units.json` | `0e27709b…` | only the map's path and hash |
| `docs/narration-direction-map-oliver15.json` | `c94f0077…` | scope note; the seven 05–08 holds' file hashes; `05_5` line 623 → 636 |
| `hold-audit.json` | `4040d484…` | the seven holds' source hashes and `05_5`'s lines; three waits; three finishes; four registered chapter hashes; open items; new `teaching_repair_05_08` record |
| `phrase-anchors.json` | `a5930e68…` | eight anchors added; counts; `by_chapter_file` for the three files |
| `phrase-timings-unmeasured-template.json` | `9169b600…` | `05_20` gains 4 marks, `06_6` 2 and `08_5` 2; seconds null |
| `direction-dispositions-proposed.json` | `d55e8ddb…` | evidence path and hash; `05_5`'s reason names line 636; map hash |
| `cue-delta.json` | `2263bb51…` | none, byte-identical |
| `stage07-delta.json` | `9fe53557…` | the four chapters, finishes, holds, waits, anchors, the helpers' sentence reads, the map's name |
| `increment-delta.json` | `2976a2b1…` | rebuilt against binder02 |
| `inputs.sha256` | `1443a00a…` | 249 lines: 23 added, 13 removed |

In `inputs.sha256`, the four chapters' old lines go and their new hashes come in. Binder02's nine plan files replace binder01's, and partial07, Lavender's five files and your four originals are added.

`increment-delta.json` records the map's scope change by the two notes' hashes, not their text. Both notes name oliver11, and RubyRaven's `STALE.oliver11_only_as_history` allows that name only in the map and in `inputs.sha256`. My first trial quoted the text and failed that census. Each plan's map still holds its own note.

**The map's name.** Binder02 wrote oliver14, and your `stage07/bound-plan` holds that oliver14 with the pre-repair 05–08 hashes. Reusing the name would give one file name two contents, so this plan calls it oliver15. Keeping oliver14 means changing `MAP_NAME` in the binder and `MAP` in `checks/preflight_view.py`, then regenerating. No pin depends on it. Neither name exists in the candidate or canonical.

## The teaching repair, as integrated

| Chapter | Before (your original = stage06 = binder02's pin) | After (candidate = Lavender's copy and `SHA256SUMS` = partial07) |
|---|---|---|
| `complex_turns.rs` | `2cc2ba77…` | `93f663d372ab79a77139287dc38b94c148909f7ea4a6ba549884c91f342eb19a` |
| `lifting_loops.rs` | `ca34aef0…` | `01885b9f558c36bc8af425bfdf427e4b30d74d575819121e52988b5d06db7346` |
| `branch_cut.rs` | `193c4733…` | `c1159c3727a1f1d83351d2c837027c563455df6ebc2de32a2db5973d56cbc401` |
| `product_rule.rs` | `e9019c82…` | `f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db` |

In scratch, `patch -p1 -F0` of `logarithm05-08-teaching.patch` on your four originals gives the after bytes exactly. Reversed on the candidate's files, it gives the originals. Your `candidate-hashes.json` records the before bytes and script `b89eab60…`, which is unchanged. The candidate now differs from stage06's crate in exactly 13 files. Every one of them is named by a receipt.

**Eight anchors**, all on cues unchanged since stage06:

| Cue | Phrase | Bytes | Sentence | Const | Hook |
|---|---|---|---|---|---|
| `05_20` | the logarithm of two | 180–200 | 4 | `TAKE_X_PHRASE` | `complex_turns.rs:369` |
| `05_20` | a half pi | 214–223 | 4 | `TAKE_Y_PHRASE` | `:371` |
| `05_20` | the length is two | 225–242 | 4 | `LENGTH_PHRASE` | `:373` |
| `05_20` | land on two i | 276–289 | 4 | `LAND_PHRASE` | `:375` |
| `06_6` | passes minus one | 267–283 | 3 | `PASSES_PHRASE` | `lifting_loops.rs:587` |
| `06_6` | leaps from pi to minus pi | 338–363 | 3 | `LEAPS_PHRASE` | `:593` |
| `08_5` | leaves a half turn | 260–278 | 4 | `HALF_TURN_PHRASE` | `product_rule.rs:438` |
| `08_5` | a factor of minus one | 299–320 | 4 | `FACTOR_PHRASE` | `:456` |

Each phrase occurs once in its cue and starts a word, and each hook comes after its cue's say. Within each cue the hooks run in spoken order. Each file reads its marks once, through `&n.phrase_marks(id)` in its local `phrase_time`. Without a mark, the hook falls back to the phrase's character share of its measured sentence. That fallback is provisional.

**Holds, waits, finishes:**
- `05_5`: say 611 → 624, marker 623 → 636, next say 626 → 639. Its interval text is unchanged.
- Since stage06, 14 of the 15 holds have a new file hash; only `01_3` is untouched. The interval changes are still only `02_2`, `10_4` and `11_20`.
- The new waits are at `complex_turns.rs:845`, `lifting_loops.rs:769` and `product_rule.rs:606`, classed `word_sync_helper`.
- Finishes: `complex_turns` 736 → 749, `lifting_loops` 692 → 699 and `product_rule` 449 → 468. `branch_cut` stays at 607. All four agree with Lavender's table.

## Your 01_5 pair

You said the corrected `01_5` pair now exists. The checks read it apart from the binder. `ui-takes01/01_5-request01.json` is for text `f0a6d133…` and script `b89eab60…`. `01_5-downloads01.json` names list `c533f4ad…` and two WAVs, both on disk at their hashes. Listening, fine sync and selection are false, false and null. **This is an observation only.** The plan's cue delta copies `narration-request-final80-02/requests.json`, which has not changed, so the states it records are still as written there: 35 full pairs, 1 pending, and 44 not established. `01_5` is among the 44. The transcript's count of receipts in `ui-takes01` is live and is printed, not checked. It was 59 at the final run.

## Preflight

No new line was needed. RubyRaven's preflight with binder02's `preflight-stage07.patch` (`4f24da5f…`, five lines) and `MAP_NAME` set to oliver15 in memory gives 329 ok, 2 FAIL and 2 WARN on this plan:
- **FAIL `STALE.canonical_still_binds_map12` and `STALE.canonical_map12_matches_canonical`:** both expected, as at binder01 and binder02.
- **WARN `CON.director_map_shape` and `TXT.requests_name_older_stage`:** both as before.

The same two controls still work. Without `:734`, `PHR.direct_mark_readers` fails. Without the request lines, the six request checks fail. Nothing else changes in either. The view's own pins, `REPAIRS` and assembly group still name stage06's bytes, as before. They were not run.

## Evidence

`checks/outputs/run_checks.txt` records 330 of 330 passing on the sealed bytes. Sections B to G recompute from bytes without importing the binder.

| Checked | How |
|---|---|
| The plan is reproducible | Two fresh runs and the delivered plan are byte-equal; `--check` passes; `--print-pins` reproduces the table |
| Your integration | Receipt, patch, originals, stage06, binder02's pins, Lavender's sums and copies agree; the patch applies forward and back with no fuzz |
| The eight anchors | Recomputed from script and source: bytes, uniqueness, word start, const, hook line inside its cue, sentence, spoken order, template rows, Lavender's rows |
| The seven holds | Recomputed by the interval formula, which also reproduces all 15 of stage06's own values on stage06's bytes; equal to `hold-table.json` |
| Nothing else moved since binder02 | A diff written apart from the binder, against a list of allowed JSON paths per file. Binder02's 25 anchors and 13 waits are unchanged, row for row, and the reading windows, sentence hooks, helpers and consumers are equal |
| Stale input | One byte added to each of 45 inputs, among them partial07, Lavender's five files, your original of `complex_turns.rs` and binder02's plan; seven real older files, among them the pre-repair `complex_turns.rs` and `product_rule.rs` |
| Semantic stops | 59 planted defects with pins bypassed, plus a control that builds. Most new plants hit the receipt or the after-hash check first, so three are consistent forgeries |
| Second census | RubyRaven's lexer with the patch above; the patch applied by `patch` equals the text the view runs |

**The forgeries.** Each one edits a chapter, rewrites that file's hash in partial07, `hold-table.json` and the proposed anchors, and reseals Lavender's `SHA256SUMS`, all in the mirror. Only the rows' content can catch them:
- Swapping two `05_20` hooks stops on `hook_sites`.
- Moving `LAND_PHRASE` to a longer phrase said once stops on the missing anchor.
- Editing text inside `05_5`'s interval stops on the hold table.

A forged comment after the tests builds, which shows the forging itself trips nothing.

**Fixtures are preserved, as you asked.** The missing chapter is moved into the scratch path and copied back, and the stray file is moved there and kept. Nothing in the scratch path is deleted.

The checks hash every input before and after. If you integrate something while they run, the final `unchanged` rows fail and `--check` exits 3. That is the refusal working.

## Not established

- **Native:** I have not built or tested these bytes. `phrase_time` and `until_phrase` in the three files have never been compiled by me, and partial07 says `native_validation: false`.
- **Takes:** no take is selected for any of the 80 cues. Whatever the list does not establish stays unestablished.
- **Timing:** `phrase-timings.json`, including the eight marks and `03_3`'s; sentence starts on real takes; the reading windows on real lengths. Without marks, all eight fall back to character shares.
- **Picture:** no glyph probe and no sheets of the repaired chapters.
- **Listening and AV:** nothing was heard or watched.
- **Independence:** none. I wrote this binder. Lavender wrote the repair and its rows. RubyRaven's preflight differs in method but shares one model with us.

## Files

```
bind_final80-stage07-r3.py     the binder, revision 3
binder02-to-03.patch           its whole change against binder02's
bound-plan/                    the ten outputs
checks/run_checks.py           all the checks
checks/build_unpinned.py       lets a plant reach the semantic stops, in a mirror only (unchanged)
checks/preflight_view.py       binder02's view with MAP set to oliver15, in a mirror only
checks/outputs/run_checks.txt  the transcript
HANDOFF.md
SHA256SUMS
```
