# Logarithm stage07 binder, incremental update

BronzeJaguar, for BlueSalmon, answering the two relays on White's source markers and the final80-02 request list. Narration is ElevenLabs Oliver and no audio is involved here.

**State: a proposed plan and a preflight patch, frozen for root, not installed. Nothing here has been compiled, rendered, synthesized, measured or heard. These source checks are not native acceptance, listening or independent acceptance.** Canonical, stage07, RubyRaven's preflight and the sealed `logarithm-stage07-binder01` are untouched.

**The result:**
- **Map and units:** byte-identical to binder01's, `6a326cde…` (oliver14) and `61a1f622…`. No hold, wait, interval or sentence hook changed.
- **White's mark:** `03_3` byte 35, `forgets`, is bound as a direct reader of `folded_decade.rs` `acfd77f4…`. Anchors go from 24 to 25. Its seconds are null.
- **Root's list:** every cue's current request now comes from `narration-request-final80-02/requests.json` `c533f4ad…`. Its submission states are copied as recorded.
- **No take is selected. Selection for all 80 cues remains pending.** No timing was invented.
- **Chapters 05–08 are not here.** Their pending source changes will be a separate update, and until then this binder refuses them.

## Run it

From the repo root, where `<F>` is this folder:

```
python3 -I -B <F>/bind_final80-stage07-r2.py --check --out <F>/bound-plan      # validate the plan here
python3 -I -B <F>/bind_final80-stage07-r2.py --out <DIR> --final-out <DIR>     # your own copy
python3 -I -B <F>/bind_final80-stage07-r2.py --print-pins                      # to rebind on purpose
python3 -I -B <F>/checks/run_checks.py <fresh-scratch>                         # all the checks below
```

Exit 3 means an input moved, and nothing was computed. It pins 98 files plus the chapter folder's listing. `--final-out` changes the dispositions' evidence path and, because it lists their hash, `increment-delta.json`. Nothing else changes.

## What changed against binder01

`bound-plan/increment-delta.json` lists every change by JSON path. `binder01-to-02.patch` is the whole binder change.

| File | Hash | Change |
|---|---|---|
| `narration_units.json` | `61a1f622…` | none |
| `docs/narration-direction-map-oliver14.json` | `6a326cde…` | none |
| `hold-audit.json` | `15aa3a4b…` | `folded_decade.rs` hash, anchor count, two open items reworded |
| `phrase-anchors.json` | `603ba46d…` | one anchor added, counts, new `direct_mark_readers`, read-out note |
| `phrase-timings-unmeasured-template.json` | `901392b4…` | `03_3` gains one mark, seconds null |
| `direction-dispositions-proposed.json` | `8b4fe095…` | only each row's evidence path and hash |
| `cue-delta.json` | `2263bb51…` | current request rows gain the list's state; `final80_request_state` is replaced by `current_request_list` |
| `stage07-delta.json` | `08f106ac…` | `folded_decade.rs` hash, the added anchor |
| `increment-delta.json` | `d4d0cd2e…` | new |
| `inputs.sha256` | `cafb2585…` | 239 lines; 13 added, and the old `folded_decade.rs` line removed |

## White's mark

Bound from `logarithm-opening-oliver-timing01/source-markers.json` `4b523a5e…`, and root's `integration-partial06.json`, `c4f023ce…` to `acfd77f4…`.

| Field | Value |
|---|---|
| Cue and text hash | `03_3`, `4ce29b09…`, unchanged since stage06 |
| Phrase | `forgets`, bytes 35 to 42, sentence 0, const `FORGETS_PHRASE` |
| Reader | `folded_decade.rs:474`, `phrase_marks("03_3")` in `forgets_mark` |
| Use | `folded_decade.rs:345`, passed to `lay_beats` |
| Source hash | `acfd77f46c2378c98ec77d1b5e7d0f2cb9c15f2bc3d209d80ba690c8fcc26e5e` |
| Without a mark | `DIGITS_SHARE` of the span from sentence 0 to sentence 1, provisional |
| Measured seconds | null |

The binder stops if any of these differs from White's file, or if that file starts proposing a change to cue text, holds, sites or sentence hooks. It does not read White's `recognition-candidates.json`, so no timing comes from it. For the opening read-out, White's audit proposes no source change, and the plan says so with its limits: neither take is selected and nothing was heard.

## Root's request list

The cue delta copies these from the list and judges none of them:

| State, as recorded | Cues |
|---|---|
| `full pair retained` | 35 |
| `submitted once; generation/download pending` | 1, `06_1` |
| `not established by this snapshot` | 44, including `01_5` |

`provider_history_checked` is false on all 80. Only `01_5` points outside final80-01, at `narration-request-rounding01/01_5.txt`. The other 79 rows name the same file and hash as before. The list names final80-01 `f93a6f1e…` as the list it supersedes, which is that file's exact hash.

I also read the 35 retained-pair receipts. Each has its WAVs on disk at the recorded hashes, with listening and fine sync false.

**One thing for selection:** `00_1`'s receipt holds four downloads, not two. Your `00_1-pcm-identity01.json` finds two distinct PCMs among them, and the receipt's note says download-to-generation numbering is not yet mapped. "Full pair" is still right. Selection will need that mapping for `00_1`.

## Preflight update

`preflight-stage07.patch` applies to a copy of RubyRaven's `preflight.py` `056d58d5…`, which stays frozen. The result is `10b23ddd…`. Five lines change and none move:

| Line | Change |
|---|---|
| `:383` | read `narration-request-final80-02/requests.json` |
| `:576` | cue-delta and the list name the same older request, except `01_5`, which must be null |
| `:579` | the older request's text equals the final cue, except `01_5`, and only `01_5` |
| `:734` | the expected literal `phrase_marks` readers gain `("folded_decade.rs", "03_3")` |
| `:824` | read each cue's request file from the row's `text_path` |

Lines 576 and 579 are the ones to look at. They were exceptionless at stage06, and `01_5` is a real exception: its older request holds the old wording. The new `:579` is exact, so a second differing cue fails it.

`checks/preflight_view.py` runs it in a mirror, with `STAGE` and `MAP_NAME` set in memory. **Its pins, `REPAIRS` and assembly group still name stage06's bytes. They are not rebased and were not run.**

The result on this plan is 289 ok, 2 FAIL and 2 WARN:
- **FAIL `STALE.canonical_still_binds_map12` and `STALE.canonical_map12_matches_canonical`:** canonical has moved on by your integration, so both are expected. Binder01 had the same two.
- **WARN `CON.director_map_shape`:** Oliver maps carry no `policy`, as the preflight's own text says.
- **WARN `TXT.requests_name_older_stage`:** now only a label. The list names stage07, the check expects stage06, and the hash matches.

Two controls show the lines matter. Without `:734`, `PHR.direct_mark_readers` fails. Without the four request lines, the six request checks fail. Nothing else changes in either run.

## Chapters 05–08: a future, separate update

This bundle includes none of it. It pins `complex_turns.rs`, `lifting_loops.rs`, `branch_cut.rs` and `product_rule.rs` at their current bytes, so integrating any of them makes the binder exit 3. Even with pins bypassed, a change without a receipt stops the build. The checks plant both cases for all four files.

I read `logarithm05-08-teaching-repair01`, which I take to be LavenderPike's from your message. It holds `binder-rows.json`, `hold-table.json` and `phrase-anchors-proposed.json`, and I found no HANDOFF. I asked Lavender to name before and after hashes in their handoff, and I told them not to edit this bundle.

**A scratch dry run, which is not a binding.** I took their four files as they stood (`93f663d3…`, `01885b9f…`, `c1159c37…`, `f2c8684a…`) and their rows, with a stand-in receipt, pins recomputed and the binder01 comparison left out. This binder then builds:
- 33 anchors in 18 cues
- 14 holds with a new file hash
- `05_5`'s say and marker lines move, and no interval changes beyond the existing three

Those bytes may still change, and this run is not in the checks below. It did find one thing, which I fixed here: White's anchor and the general added-anchor set are now separate constants, so a later update can add anchors without tripping White's guard.

The later update will need those rows, new pins, a comparison rebased on this plan, and its own preflight pass. I have not tested the preflight on those files.

## Evidence

`checks/outputs/run_checks.txt` records 263 of 263 passing on the sealed bytes. Sections B to G recompute from bytes without importing the binder.

| Checked | How |
|---|---|
| The plan is reproducible | Two fresh runs and the delivered plan are byte-equal; `--check` passes; `--print-pins` reproduces the table |
| White's anchor | Recomputed from the script and source: byte, sentence, both lines, one reader in the file, template row, fallback text; White's `SHA256SUMS` holds, and their candidate copy equals stage07's file |
| The added anchor is the only anchor change | Binder01's 24 rows are identical field for field; an own scan finds exactly the literal readers `03_3` and `04_2` |
| Root's list | Exact bytes for all 80, supersession hash, states counted, receipts read |
| Nothing else moved since binder01 | A diff written apart from the binder, against a list of allowed JSON paths per file; holds, waits, finishes, hooks, helpers and consumers are equal |
| Stale input | One byte added to each of 37 inputs, including the list, White's file, binder01's plan and all four pending chapters; plus real older files, including the pre-mark `folded_decade.rs` |
| Semantic stops | 44 planted defects with pins bypassed, plus a positive control that builds; a planted `forgets` swap exposed a crash in my own new guard, now a clean stop |
| Second census | RubyRaven's own lexer, with the patch above; the patch applied by `patch` equals the text the view runs |

A pass says the text and hashes agree. It does not say the film works.

The checks hash the plan's inputs before and after. As with binder01, a concurrent integration by root would fail the final `unchanged` rows and `--check` would exit 3. That would be the refusal working, not a defect.

## Not established

- **Native:** no build or test of these bytes; `forgets_mark` has never been compiled by me.
- **Takes:** selection for all 80 cues; `01_5` generation; `06_1` generation and download; the 44 cues whose state the snapshot does not establish.
- **Timing:** `phrase-timings.json`, including `03_3`'s mark; sentence starts on real takes; the reading windows on real lengths.
- **Listening and AV:** nothing was heard or watched, and the opening's read-out needs a person.
- **Independence:** none. I wrote chapters 09–12's repairs and this binder, and White wrote the mark. RubyRaven's preflight disagrees in method, but shares one model and one memory with us. An independent reader should take this before it counts as review.

Oliver14 stays the map's name, as you accepted.

## Files

```
bind_final80-stage07-r2.py     the binder, revision 2
binder01-to-02.patch           its whole change against binder01's
preflight-stage07.patch        five lines against RubyRaven's preflight
bound-plan/                    the ten outputs
checks/run_checks.py           all the checks
checks/build_unpinned.py       lets a plant reach the semantic stops, in a mirror only
checks/preflight_view.py       the patched preflight, in a mirror only
checks/outputs/run_checks.txt  the transcript
SHA256SUMS
```
