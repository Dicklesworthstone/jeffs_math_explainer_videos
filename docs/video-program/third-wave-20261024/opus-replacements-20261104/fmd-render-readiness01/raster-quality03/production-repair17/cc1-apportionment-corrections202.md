# Apportionment corrections 202 (cc1): cues 10_2, 14_7, 14_8

Written 2026-11-08 (UTC) by the cc1 pane. It answers root's follow-up to review 197: root accepted F1 (14_8) and F2 (10_2) and asked for F3 (14_7) to be clarified before the remaining synthesis.

The patch is `cc1-apportionment-corrections202.patch`, in this directory (sha256 `605966dc6c749aecb861e65bd8179c76cecdd11cab57b8233a399b6aa8b28e4a`, 18,978 bytes, 7 files).

## What this is and is not

- **Not applied by me.** I edited no file under `explainer_corpus/`, no request, recording, receipt, Bead, TODO, `progress.json` or other shared file. Every candidate file was built and checked in session scratch, from the live bytes listed below. Root applied the patch after delivery; see "After delivery" at the end.
- **Not done:** no build, no `cargo`, no render, no synthesis and no listening. Nothing here is a native qualification, fine sync or independent final AV acceptance. The validation below is source and plan validation only.
- **No picture change.** The cards, layout, waits and colours are untouched. One card keeps its current text on purpose; see "Deliberately unchanged".

## F3 clarified (14_7)

The film has heard two different theorems by the time 14_7 plays. 14_1 says: "Balinski and Young proved in nineteen eighty two that the answer is no, with one extra assumption … A recent paper by Paul Gölz, Dominik Peters and Ariel Procaccia proves the answer is still no without that assumption". For four towns, those two theorems have different answers:

| question | four towns | source (arXiv:2202.11061v2, §3.1) |
|---|---|---|
| quota + population promise, **with** the extra assumption (order-preserving) | settled: impossible | "The original impossibility result of Balinski and Young (1982) (which, in addition, assumes the order-preserving property) uses only n = 4 states." |
| quota + population promise, **without** it (Theorem 1's setting) | open | "The proof of Theorem 1 uses n = 5 states … Whether impossibility holds for n = 4 remains an open question." |
| three towns | Webster keeps both | "For n = 3 states, the Sainte-Laguë method (also known as Webster's method) satisfies both properties (Balinski and Young, 1982, Proposition 6.2)." |

The current line, "With four, as of that paper, nobody knew", does not say which of the two questions it means. Right after GPP's five-town proof, the second is the natural reading, which is why review 197 rated F3 Low. But a viewer who remembers 14_1 can hear "nobody knew" as covering both. For the first question that is false: Balinski and Young's own proof used four towns.

**Proposed (+4 words, 44 → 48):** "With four, without the extra assumption, nobody knew as of that paper."

- "The extra assumption" points back to words the viewer has heard: 14_1's "one extra assumption" and "without that assumption".
- "As of that paper" moves to the end. That keeps the time scope without two interruptions in a row. I did not search later literature, so the line still makes no claim that the question is open today.
- The trigger "With four" is unchanged, and the card still appears on it.
- Alternatives, if root prefers: "With four, without that assumption, it was still open as of that paper." (+4 words); or keep the current line, which is defensible in context. I recommend the patched wording.

## Exact spoken changes

| cue | before (words) | after (words) | change |
|---|---|---|---|
| 10_2 | 41 | 41 | "any piece of a share" → "any piece of a **quotient**" (F2) |
| 14_7 | 44 | 48 | "With four, as of that paper, nobody knew." → "With four, **without the extra assumption**, nobody knew as of that paper." (F3) |
| 14_8 | 33 | 49 | "The same paper shows a way out." → "The same paper shows **what a lottery can add**." Added at the end: "**But even a lottery cannot keep the quota promise and the population promise together.**" (F1) |

The full after-texts are the `+` lines of the `script.tsv` hunk in the patch. Script total: 4,146 → 4,166 words. Chapter 10 is unchanged at 359 words; chapter 14 goes from 428 to 448.

**Triggers kept.** Every `s.wait_for` phrase in the three cue blocks still matches once, in source order. Positions are given in direction words:

| cue | trigger | word index before → after |
|---|---|---|
| 10_2 | "proposed rounding up" | 3 → 3 |
| 10_2 | "lean flips toward small towns" | 19 → 19 |
| 10_2 | "At fifteen seats Adams gives" | 24 → 24 |
| 10_2 | "Cedar falls below" | 33 → 33 |
| 14_7 | "So no rule that keeps the quota promise" | 3 → 3 |
| 14_7 | "With three towns, Webster's rule keeps both" | 29 → 29 |
| 14_7 | "With four" | 36 → 36 |
| 14_8 | "it can keep the quota promise" | 13 → 15 (once only; "cannot keep" does not match) |
| 14_8 | "exactly its share on average" | 28 → 30 |

No prediction override or visual hold is on these cues, so the map's 15 `overrides` are byte-identical.

## Files: live bytes the patch was cut against, and the result

All seven live files were re-hashed just before delivery and still matched the "before" column.

| file (under `explainer_corpus/apportionment/`) | before sha256 | after sha256 |
|---|---|---|
| `docs/storyboard.md` (authoritative) | `6324895bc5e9a821ee9efd4fa33945ab58729cd02bf9b2d2d1eca7b74f09b566` | `564e86db6c12c75a651bfcb6912619e2b3219337442095b64b78f2b49e930176` |
| `script.tsv` (generated) | `29aa2993426bb323e5723162e446f1cf70d00836b8bc91ebb6503ad3ca48cf07` | `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624` |
| `src/chapters/rounding_rules.rs` (comment only) | `d5baaecdde2dd7d50b1cc8cbef60b177f6bf2e647709c181c3b9022aa04163f5` | `7b82c0f0514fd1e65d613532e64b76ff0566e372b37c6743681a0ceae485c835` |
| `src/chapters/no_rule.rs` (comment only) | `aceba2952c63fd44692d5015958eb76e1ce96c87f63ac17cf7212c62815cbd5e` | `7c88a18d610884876403498fa5b1f6f208c633c55d122beced7a7bc6b2bc805d` |
| `docs/description.md` | `0c87d6bcdabea1120bdd0b89cc45ffe370c6f1c5de363446aa6959a5eb84cf75` | `16c44c2c890d019a5b53d574970ede3dceaf87a8ea4e94e13a789cb5660056a9` |
| `docs/narration-direction-map-frederick01.json` | `8ae86ed5e848a94c3a9f33a592d01495ee5c0eace068aa592e4fd80f97762c53` | `212ae493dfa80d0f700b736303e32505f86ba9cb73265c46b44726ac4b8d94f7` |
| `narration_units.json` | `b1f8129c67065dbed615ca2751570f73e52516acff82130a51124afba5b3d6ad` | `19c4f0a5f05411a97f9f10d2c7939381f80130a058d585601d8b2f3a442c4dc5` |

Tools named below, as hashed for this note: `tools/storyboard.py` `3faed8c1…b0b7`, root's `validate-plan135.py` `1c3a5ab1…6876`, root's `clarity139/build140.sh` `b2e3772b…8e00`, `requests04.json` `1dd6c3e0…bbd`, `frederick-generation89/progress.json` `e74878cf…ac4e`.

### What each hunk does

1. **`docs/storyboard.md`:** only the words cell of rows 10_2, 14_7 and 14_8. The picture, features and hold cells are unchanged.
2. **`script.tsv`:** the same three rows. This is exactly what `tools/storyboard.py apportionment` writes from the patched storyboard. In scratch, a copy of the repo's tool regenerated it byte for byte, and `--check` exited 0. The `#` header line is unchanged.
3. **`rounding_rules.rs` line 172 (comment):** `// 10_2 Adams rounds up: any piece of a quotient earns a seat`.
4. **`no_rule.rs` line 470 (comment):** `// 14_8 what drawing lots adds; it still cannot keep the population promise`. Both comment edits stay on their own line. No line moves, so the hold markers at `rounding_rules.rs:189` (10_3) and `no_rule.rs:289` (14_3) stay at their recorded lines. The comments contain no `say(&kit, stage, "` text, so the said-once test is unaffected.
5. **`docs/description.md`:** two changes:
   - "four towns, without the extra assumption, were open as of that paper". The paragraph has already defined the assumption ("with an extra assumption", the order-preserving quote).
   - After the 54% / 46% sentence: "It does not escape their impossibility theorem: no lottery keeps the quota promise and the population promise together."

   Neither adds a digit; the set of digit tokens is unchanged (256 tokens), so `docs/description_numbers.tsv` needs no row. The paragraph's line wrap is left uneven to keep the hunk small.
6. **Direction map (3 values):** `scripts.apportionment` gets the new script sha. The two `visual_holds` entries on the edited chapter files get their new file sha256: 10_3 (`rounding_rules.rs`) and 14_3 (`no_rule.rs`). Their line and marker are unchanged. No other key changes.
7. **`narration_units.json` (7 lines):**
   - the three unit texts (`oliver_10_2`, `oliver_14_7`, `oliver_14_8`);
   - `script_sha256` and `direction_contract.script_sha256`;
   - `direction_contract.map.sha256` (`212ae493…`);
   - `scope`, which becomes: "Frederick source plan202. Cue10_2 names quotients, not shares, as what Adams rounds up; 14_7 scopes the open four-town case to the question without the extra assumption; 14_8 says what a lottery adds and that it still cannot keep the quota and population promises together. All prior takes retained; native build and final timing review pending."

   The file stays ASCII-escaped with a trailing newline. This is the same set of fields root's plan140 changed.

**Optional: drop the two comment hunks.** This is the only optional part. Removing hunks 3 and 4 would leave both chapter files byte-identical, and the map would then change only `scripts.apportionment`. In that case the map and units "after" hashes above no longer apply and must be recomputed, so the patch would need recutting. I recommend keeping the hunks: the old comments would state the two errors this patch removes.

## Deliberately unchanged

- **The card `"4 towns: open, as of that paper"`** (`no_rule.rs`, at "With four"), and the storyboard's picture cell describing it. It is still true for the question 14_7 now names. Adding "(without the extra assumption)" would be a picture change and a layout risk, which root asked to avoid now.
- **The 14_8 card** (title, three checks, the Ash line). It is accurate. The new last sentence plays over it after the "exactly its share on average" wait. No new wait, no new row: review 197's red-cross row stays optional and is not in this patch.
- **No fact, number, oracle or `description_numbers.tsv` change**; the facts count stays 443.
- **Preserved:** `narration-frederick-requests01`…`04` and their `text/10_2.txt`, `14_7.txt` and `14_8.txt`; every generation and review receipt and take; and the historical copies of the old wording in `snapshots/`, `candidate*/`, `before-*`, `native-plan-validation/`, `source-checks/` and `chapter12-root-roundtrip01/`. They are snapshots, not inputs. The same goes for `explainer_corpus/apportionment/script-provenance*.json` and `script-before-cue-split01.tsv`: they bind older scripts (`84c83681…`, `9a342f53…`), no tool reads them, and they do not contain these cue texts.
- **Other topics' scripts in the map** are unchanged (verified).

## Takes and synthesis order

- `progress.json` lists 52 generated cues, 01_1 through 10_1 (1,990 words, 106 takes preserved), with `next_cue` 10_2. None of the three changed cues has a take, and all three are on `synthesis_holds`.
- The other 90 cue texts are byte-identical. All 93 requests04 hashes were checked: each `text_sha256` is sha256 of the script row plus `\n`, so every retained take keeps its text binding.
- 10_1 was submitted with its own text only: `10_1-intent.json` holds the 10_1 text alone, and `10_1-submitted.json` records one generate click. No neighbour context was sent, so the 10_2 change cannot break 10_1. Do not regenerate 10_1.
- Order after applying: 10_2 first, as the next cue.

## Commands for root (apply under your leases)

All commands run at the repo root.

```sh
# 0. refuse if any target changed since this patch was cut
shasum -a 256 explainer_corpus/apportionment/docs/storyboard.md explainer_corpus/apportionment/script.tsv \
  explainer_corpus/apportionment/narration_units.json explainer_corpus/apportionment/docs/narration-direction-map-frederick01.json \
  explainer_corpus/apportionment/docs/description.md explainer_corpus/apportionment/src/chapters/rounding_rules.rs \
  explainer_corpus/apportionment/src/chapters/no_rule.rs        # must equal the "before" column

# 1. apply (storyboard, script, comments, description, map + units rebind in one step)
git apply --check docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/cc1-apportionment-corrections202.patch
git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/cc1-apportionment-corrections202.patch

# 2. regenerate the script from the authoritative storyboard (always pass the slug: without it the tool rewrites every topic)
python3 -I tools/storyboard.py apportionment          # rewrites script.tsv; sha256 must stay b9344e11…a624
python3 -I tools/storyboard.py --check apportionment  # exit 0

# 3. plan validation (empty, new scratch dir; the script refuses an existing one)
python3 -I -B narration_lab/third-wave-20261024/root/elevenlabs-apportionment-rebuild-20261104/frederick-generation89/cue02_3-repair135/validate-plan135.py \
  "$PWD" "$PWD/explainer_corpus/apportionment" <new empty dir>
#   expect: 15 directions resolved / 93 units cover 93 cues in order / 15 markers at their lines, hashes current
```

Step 4 is the guarded native build. It has the same shape as `clarity139/build140.sh`, copied to a new numbered script with new output paths, because build140 opens its outputs with exclusive create:

- `cargo test`, then `cargo build`, then `cargo clippy --all-targets --no-deps -- -D warnings`, all `--locked --offline --manifest-path explainer_corpus/apportionment/Cargo.toml`;
- native `facts` against `oracle.py`: 443 facts, equal;
- `apportionment chapters`;
- `validate-plan135.py` into a new `native-plan-validation` dir;
- the binary copy and its sha.

build140 recorded 30 tests passed, 443 facts, 93 units and 15 holds, and the same counts are expected here. The build is required before any timing or preview: `src/main.rs` embeds `script.tsv` and `docs/storyboard.md`, and the chapter tests embed every chapter source.

**build139 lesson:** a guarded-input or manifest preparation that parses `script.tsv` must skip its first line, `# apportionment: generated from docs/storyboard.md by tools/storyboard.py; …`.

### Request manifest (requests05, root's)

requests04 stays as it is and is superseded. A new `narration-frederick-requests05/requests05.json` in requests04's shape needs:

- `script_sha256` `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624`
- `units_sha256` `19c4f0a5f05411a97f9f10d2c7939381f80130a058d585601d8b2f3a442c4dc5`
- `direction_map.sha256` `212ae493dfa80d0f700b736303e32505f86ba9cb73265c46b44726ac4b8d94f7` (path unchanged)
- `word_count` 4166; `cue_count` 93 and `max_words` 100 unchanged; `supersedes_request_manifest` → requests04.json
- `text/<cue>.txt` = script row + `\n`. 90 texts and their hashes equal requests04's. The three changed ones:

| cue | requests04 `text_sha256` | requests05 `text_sha256` | words |
|---|---|---|---|
| 10_2 | `08835d25b7b48fa1c42753f7c04723b1e774fba594eb2bf30fb7b0f8df36a4f7` | `776edb4b34ff5fb78216d0928400ac92107e790d5168ef1e7b4f80a93829f48f` | 41 |
| 14_7 | `f4678d2c59af68f52beb38169bdd253469bf97e3451b517f3a041cc4cb836c30` | `843005597a85e5d6ab01a94f1698b5552dd8818fabed37e5180161f9b11cd6d5` | 48 |
| 14_8 | `d355f5abfd560b1866bc1a2bddf851176576cc44499592c160a4ee15e54b2954` | `b09af8ff25160ae775242814d3b1c88260c0d11296b3b79a76d832658e507f11` | 49 |

`progress.json` (root's) then needs:

- `active_request_manifest` → requests05;
- `active_script_sha256` → `b9344e11…a624`;
- `script_words_total` 4146 → 4166;
- release the three `synthesis_holds` after build and validation.

`cues_generated`, `no_regenerate` (52) and `next_cue` 10_2 stay as they are.

## Length (estimate, not measured)

At the median rate of the 47 measured first takes (2.69 words/s, range 1.85–3.14):

- 14_7: about 16.4 → 17.8 s;
- 14_8: about 12.3 → 18.2 s;
- chapter 14 speech: about 159 → 166 s;
- 10_2: unchanged.

The extra 14_8 sentence plays over the existing final card, before `kit.finish`. No hold or wait was retimed; actual timing comes from the new takes and the native timeline.

## Validation done in scratch (source and plan only)

- The candidate files were built from the live bytes (each asserted against the "before" hash) and passed:
  - `storyboard.py` regeneration equal to the direct row edit; `--check` exit 0;
  - root's `validate-plan135.py` on the candidate topic, with all 27 live chapter files and the two edited ones swapped in: exit 0, "15 directions resolved", "93 units cover 93 cues in order", "15 markers at their lines, hashes current".
- Controls:
  - **positive:** the same validator on the live bytes, exit 0;
  - **negative:** candidate chapters with the old map, exit 1 (`direction map changed`).
- Python mirrors of the `mod.rs` tests:
  - voice rules (no digits, `: ; ( ) – — -`): none in 93 rows;
  - every line said once, in order: 15 chapters OK.
  - These are not `cargo test`.
- Only the 3 intended script rows and 3 unit texts differ. The changed top-level unit keys are exactly `direction_contract`, `scope` and `script_sha256`. The map's changed keys are `scripts.apportionment` and two `visual_holds` hashes.
- Skill `lint_script.py` with all 15 chapter keys: 0 errors and 33 warnings both before and after. No warning is on 10_2, 14_7 or 14_8.
- Patch:
  - `git apply --check` at the repo root against the live bytes: OK, run before and after copying it here;
  - applied to a copy of the originals, it reproduces all 7 candidate files byte for byte;
  - `git apply --reverse --check` on the candidate: OK.
- The mathematical basis is review 197 (exact-rational recomputation and the GPP primary text), with this note's quotes for F3. Same author, so this is not an independent second check of review 197.

Not established: native compile, test, Clippy, facts or chapters; any take, timing, listening, fine sync or independent final AV acceptance.

## After delivery (read-only checks, 2026-11-08 about 03:42 UTC)

- Root's TODO records the patch as applied at 03:41:28Z under leases 5635–5642, with the originals kept in `frederick-generation89/corrections209/before`. The seven live files now hash to the "after" column above, exactly.
- `narration-frederick-requests05/requests05.json` binds script `b9344e11…`, units `19c4f0a5…` and map `212ae493…`, with 4,166 words, 93 cues and `supersedes_request_manifest` requests04. Its text hashes differ from requests04 only for 10_2 (`776edb4b…`, 41 words), 14_7 (`84300559…`, 48) and 14_8 (`b09af8ff…`, 49). Every text file hashes to its recorded `text_sha256`.
- I did not check the outcome of guarded build209 (still running per the TODO), and I touched none of these files.
