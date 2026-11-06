# Cue 09_5 wording repair

WhiteSwan, for root (Mails 5588, 5598). It also answers the two chain traps in Ruby's Mail 5600, because I wrote that chain.

Nothing live changed. I built, rendered and synthesized nothing. No Rust was compiled, and nothing here is listening, picture or sync evidence. I wrote this repair, so it isn't an independent review.

`EC` is `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`. `Q` is `EC/remote-ts1-01`.

## Ready text

Cue 09_5, in full, sha256 `c84c83bd91b69948de77e1d21089e71c70abe71fcde6c49510c81568be104bcb`:

> It is zero. The bigger circle has swallowed the poles at i and minus i, where the bump is not holomorphic. Each pole pulls the average down by one half. For this formula, a sufficient condition is that the function is holomorphic throughout the disk and a little beyond its edge.

This is byte-identical to the text root submitted in `Q/cauchy-cue-repair01/request01.json`, with the same six settings.

Why the old sentence was false: √(1 − z) can't be extended holomorphically past z = 1, yet its average on the unit circle is 1.00000, its centre value (`checks/check_repair.py`). DLMF §1.9 needs only analytic inside and continuous on the contour.

## The card has to change too

The "precisely:" card shown from the cue's last sentence through its reading window (`src/chapters/boundary.rs:56-57`, shown at `:403`) repeats the claim. Ruby and Bronze found the same thing independently.

| | Card text |
|---|---|
| Live | Cauchy's formula needs f holomorphic on an open set containing the closed disk: on the circle and inside it |
| Staged | Cauchy's formula holds if f is holomorphic on an open set containing the closed disk: on the circle and inside it |

**Root's call.** Ruby suggested "holds when f is holomorphic". I staged "holds if" for two reasons: it can only mean sufficiency, while "when" echoes the removed "only holds when", and it keeps line 57 at the 120-character limit, where "holds when" would run to 122. Any other wording moves the hashes of `boundary.rs`, the map and the units, so tell me and I'll reissue.

## Staged files

`staged/explainer_corpus/complex_analysis_visual/` holds the five full files. `repair.diff` shows every change, and `bindings.json` has the full hashes.

| File | Lines changed | What | Old | New |
|---|---|---|---|---|
| `script.tsv` | 1 | Line 61 | `0bae7acb15f2` | `946a9c07a462` |
| `src/chapters/boundary.rs` | 1 | Line 57, the card; no line moves | `69e41733fd0e` | `1ebe0331ba47` |
| `docs/storyboard.md` | 1 | Line 198, cue and card; `main.rs:76` tests the words against the script | `9ea68add5ac4` | `ad1ccd448931` |
| `docs/narration-direction-map-frederick-stage07.json` | 6 | Script hash, `overrides[11].after`, three boundary hold hashes, scope | `fa2b976b6e91` | `4db4c4d5a433` |
| `narration_units.json` | 6 | Two script hashes, map path, map hash, unit text, scope | `e8d2d91dcb0e` | `c937fca1f0d7` |

- **Three hold rows change,** 09_4, 09_5 and 09_8, because all three hash `boundary.rs`. Their line numbers (342, 407, 686) don't move.
- **The map is a new file.** The stage06 map stays beside it. The units point at stage07, and no source file names either map.
- **Apply to a new tree only,** never to `native-stage06-root03`. Copy all five files as a set, because each hash binds the next.

## Anchors and neighbours

Checked with no change needed:

- **Sentences:** four before and after, by the unmodified tool's own split. The first three are identical text.
- **Scene:** 09_5 uses `wait_sentence` 1, 2 and 3, with no `wait_for`, and `phrase-timings.json` has no marks for it. The card still arrives on "For this formula".
- **Reading time:** `mod.rs:69` stays 3.0 s. The test's bounds hold, 3.60 s against 2.53 and 15.07. The `unspoken` clause at `mod.rs:723` is still on the card, so that test is unchanged. A longer last sentence only adds card time.
- **Card layout:** no new character, so no new glyph. It wraps to two lines by a mirror of `animatic::wrap`, at 92 and at every narrower step to 60. No pixels were seen.
- **Format:** `rustfmt --check` passes on the staged `boundary.rs`.
- **Lint:** `lint_script.py` gives 0 errors and 35 warnings, before and after.
- **Captions:** built from `report.json`, so nothing is authored. The sentence wraps to 4 lines at 42 characters, against 3 before. I didn't mirror `balanced_captions`; the kit's `validate` applies when captions are built.
- **09_1:** it already says "a little beyond its edge", so the new sentence repeats its wording.
- **Takes for 09_4 and 09_6:** their requests carry no context fields, so neither binds the old 09_5.

**One thing to listen for:** 09_6 opens "The same circle". The old sentence ended on "the circle", while the new one says "disk" and "edge". "The bigger circle" earlier in 09_5 still gives it something to refer to, so this isn't a defect, but it's a changed join.

## Take bindings

| | Old take | New take |
|---|---|---|
| Request | `frederick-stage06-01/09_5-request02.json`, `c94d916e…` | `Q/cauchy-cue-repair01/request01.json`, `e9b0aef2…` |
| Submission | `09_5-submission02.json` | `Q/cauchy-cue-repair01/submission01.json`, `cf0b5c04…`, Loading when read |
| Selected source | `09_5-download01.wav`, `5882db72…` | Pending; root retrieves |
| Decoded | `decoded/09_5.wav`, `f3f5b9bd…`, 405120 samples | Pending |
| Narration folder | `09_5.wav`, `2e3e1419…` | Pending |
| Sentence ends | 35520, 185280, 269760, 405120 | Pending; a full new take, so all four move |

**Gap: root's two receipts won't pass the stage06 intake as written.** The originals should stay as they are. Either the stage07 intake accepts their shape, or an intake-shaped record cites them.

| `check_receipt` needs | Root's receipt has |
|---|---|
| Name `09_5-requestNN.json` in the take folder | `request01.json` |
| `script_sha256` equal to the pinned script | `prior_script_sha256` only |
| Name `09_5-submissionNN.json`, binding the request by name or hash | `submission01.json`, with `text_sha256` only |
| `submitted` or `submitted_once`, and a generation count | `submission_clicked_once` |
| `09_5-downloadsNN.json` with two takes per generation | Pending |

`09_5-request-candidate.json` has the intake's request fields with the stage07 script hash, if you take the second route.

## Dependency order

Ruby's Mail 5600 §3 lists the pins in the qualifier, chain, final bundle and ASR. Bronze's Mail 5599 lists the ASR rows. I checked the points below and won't repeat those lists.

1. **Source.** New tree from `native-stage06-root03`, with the five staged files. Tools unchanged.
2. **Take.** Retrieve, choose, recognize this cue only. All four sentences must be found. The first three are newly voiced, so no earlier view of them carries over.
3. **Audio, new folders only.** Stage07 copies of the intake and bridge, with these values:

   | Pin | New value |
   |---|---|
   | Intake `PINS` script | `946a9c07a462eb066fc8d20c6a092d5b7564d38752d26177bdbbcf15c6027cdc` |
   | Intake `PINS` units | `c937fca1f0d7507117462600f0f478472bd2b8dab93075aba6494bd99ce81141` |
   | Intake `PINS` map | `4db4c4d5a433f3212e6af739d71625dd0931ac86bc68c4cfc859212925faff77` |
   | Intake `MAP_REL` | `docs/narration-direction-map-frederick-stage07.json` |
   | Intake `CRATE`, `TAKES` | The new tree and take folder |
   | Bridge `CHAPTER_SOURCES_SHA256` | `40ed4a20bc57c1165e269d8c712cc7bcd22b37eb00af9be35fa12a7f4dc65762` |

   Then transfer 104 cues (stage06 to stage07, same rule as stage05 to stage06), and run decode, prepare, assemble, timings and install. The selection row for 09_5 (`stage06-timing-decisions01/selection01.json`) needs replacing. The `direction_dispositions` rows for the three boundary holds must cite the new `boundary.rs`.
4. **Build, only after `pictures02` exits,** under a new `BIN` name (Mail 5600 §2). `main.rs:22` embeds the script and `check_delivery_contract` refuses a mismatch, so a new binary is required. Then fmt, Clippy, tests and facts-check, plus the glyph probe for the card.
5. **ts1.** New qualifier, then a new chain with unused names: derive, timeline, score, preview, pictures.
6. **Final and ASR.** Re-pin Lavender's and Bronze's bundles to the new chain. Ruby reports that no final-job name has been used yet.
7. **Review.** Listening and fine sync for the new take. Pictures for `09_boundary`, including the card. Other chapters' viewing carries over only where sheet hashes match.
8. **Publication.** `complex-publication-draft02` says "is used for", which stays true, so its wording needs no change. Its script hash and chapter times need the new values, and everything after chapter 09 moves.

The score can't be narrowed: it covers the whole film (`cli.rs:7`) and needs the full registry (`cli.rs:407`), so run it once and treat any stem reuse as a hash result.

## Ruby's two chain traps

**Trap 1, `prove()` at `ts1_chain02_n2.py:168-169`.** It requires ts1 to refuse `N1`. Assemble on macOS as stage06 did: `report.json` then holds `/Users` spellings again, ts1 refuses it the same way, and derive stays necessary with the same 414-row proof. If you assemble on ts1 instead, there's nothing to derive, so that stage should be dropped, not kept.

**Trap 2, `mapped()` at `:227-228`.** It maps `N2` cells to `N1`, and the macOS baseline holds the retained old `N` path, so a new `N1` would fail at the end of a full timeline. Two fixes work:

- **Better:** use `timeline02`'s ts1 tables as the baseline, if it ends with exit 0. That covers 14 chapters instead of 4 and removes the cross-platform question. Map each new folder cell to the matching `narration-n2` cell.
- **Otherwise:** keep the macOS tables and map to the retained old `N`, a constant separate from `N1`.

**Expected on the new timeline:** all three tables equal for the 14 other chapters after path mapping. In `09_boundary`, rows before 09_5 equal, 09_5 new, and each later row moved by one constant. `CUES`, `ROWS`, `FILES` and `PRIOR_PATH_CELLS` stay 105, 414, 110 and 52, as long as the take has four sentences.

## Limits

- The two-line wrap is a Python mirror. Real text width and the rendered card are unchecked.
- The new take's length is unknown, so chapter 09's growth isn't stated.
- The chain answers above come from reading my own code. Nothing has run on ts1.
- The stage07 intake, bridge and chain aren't written. I can write them on request, each in a new folder.

## Replay

```
P=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-cauchy-wording-repair01
(cd $P && shasum -a 256 -c SHA256SUMS)
python3 -I -B $P/checks/check_repair.py
```

Run from the repository root. The expected output is `checks/transcript.txt`. The check writes only to a new temporary folder and leaves it in place.
