# Apportionment requests05 review 210 (cc1)

Written 2026-11-08 at about 04:11 UTC by the cc1 pane. This is a read-only check of requests05 and the take bindings after root applied corrections 202 and build209 passed. I edited nothing outside this file. There was no synthesis, build, render, Bead, TODO or `progress.json` edit. The checks ran on the bytes listed below. The only process I ran, besides reading files, was `tools/storyboard.py --check` on a scratch copy.

Not done: no listening, no fine sync, and no independent final AV acceptance. Nothing here says a take sounds right; it checks only which text each take was bound to.

## Result

**One concrete mismatch, M1. Everything else checked agrees.**

### M1: `progress.json` skips 10_6 and 10_7

`progress.json` (sha256 `fb1482f4359895418027accb8d124def2e2bc71a747a8d4ed63ea5d3d6414ced`, last written 04:02:13Z) says:

- `cues_generated` and `no_regenerate`: 56 cues, 01_1 through 10_5;
- `next_cue`: **`11_1`**.

But the live script has **10_6** ("Each pair of neighbouring whole numbers gets its own threshold…") and **10_7** ("So whenever there are enough seats to go round…") between 10_5 and 11_1. Neither has any receipt in `frederick-generation89/`, and neither is in `synthesis_holds` (empty) or `held_cues` (06_3 only).

The earlier snapshots step one cue at a time: `before-probe211` through `before-probe216` have next cues 10_2, 10_3, 10_4 and 10_5. Only the write after probe216 jumps to 11_1.

**Smallest fix (root's):** set `next_cue` to `10_6`. If 10_6 and 10_7 are being held on purpose, record the hold; for example, review 197's optional O1 touches 10_6. Both request texts are present and correct in requests05:

- 10_6: `text/10_6.txt`, the same bytes as requests04;
- 10_7: `text/10_7.txt`, the same bytes as requests04.

Do not regenerate 10_2 or 10_3; each was generated once, under requests05.

## Inputs (sha256)

| input | sha256 |
|---|---|
| `narration-frederick-requests05/requests05.json` | `39ee6b328627b020f1920131a5816682f176c8aa7eedf7be50c4e33fbfa85c14` |
| `narration-frederick-requests04/requests04.json` (unchanged) | `1dd6c3e0ed8ce275acdcbe7f8da24f3cf5103adeb2ca86a6ab893262f2be9bbd` |
| `frederick-generation89/progress.json` | `fb1482f4359895418027accb8d124def2e2bc71a747a8d4ed63ea5d3d6414ced` |
| `explainer_corpus/apportionment/script.tsv` | `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624` |
| `explainer_corpus/apportionment/narration_units.json` | `19c4f0a5f05411a97f9f10d2c7939381f80130a058d585601d8b2f3a442c4dc5` |
| `explainer_corpus/apportionment/docs/narration-direction-map-frederick01.json` | `212ae493dfa80d0f700b736303e32505f86ba9cb73265c46b44726ac4b8d94f7` |
| `explainer_corpus/apportionment/docs/storyboard.md` | `564e86db6c12c75a651bfcb6912619e2b3219337442095b64b78f2b49e930176` |
| `src/chapters/rounding_rules.rs` / `no_rule.rs` | `7b82c0f0…c835` / `7c88a18d…805d` |
| `docs/description.md` | `16c44c2c…56a9` |

All seven live files equal the "after" column of `cc1-apportionment-corrections202.md`. Each of the seven originals (the "before" column) is preserved byte for byte under `frederick-generation89/corrections209/before/explainer_corpus/apportionment/`.

## 1. requests05 against script, units and map

All of these hold:

- `script_sha256`, `units_sha256` and `direction_map.sha256` equal the live script, units and map.
- `direction_map.path` and `script_path` are the crate paths.
- The units file binds the live script twice (`script_sha256`, `direction_contract.script_sha256`) and the live map once. The map's `scripts.apportionment` is the live script.
- The 93 units cover the 93 script cues in order, and every unit text equals its script row.
- `storyboard.py --check apportionment`, run on a scratch copy of the storyboard and script, exits 0. The script is still exactly what the storyboard speaks.
- requests05 lists 93 requests in script order. `text/` holds exactly the 93 cue files and nothing else.
- For every request:
  - `text_path` is `narration-frederick-requests05/text/<cue>.txt`;
  - the file bytes are the script row plus `\n`, and they hash to `text_sha256`;
  - `words` equals the row's word count;
  - `unit_id` is that cue's unit;
  - `voice` is "Frederick Surrey - Smooth and Velvety".
- Words sum to 4,166 = `word_count` = `progress.script_words_total`. `cue_count` is 93, and `supersedes_request_manifest` is requests04.
- Against requests04, only 10_2, 14_7 and 14_8 differ, with the hashes and word counts in corrections 202. The other 90 text files are byte-identical. requests04 and all 93 of its text files still hash to their recorded values.
- `progress.json`: `active_request_manifest` is requests05, `active_script_sha256` is the live script, `script_words_generated` 2,157 equals the 56 generated cues' words, and `synthesis_holds` is empty.
- `corrections209/build209-inputs.json` names all seven live "after" hashes. `build-qualified209.json` records 30 tests, 443 facts, 15 directions, 93 units, 4,166 script words and `stable: true`. I read that receipt; I did not rerun the build.

## 2. The 52 takes recorded before requests05 (01_1–10_1)

For each cue I took the selected intent receipt. For 02_3 that is the `take_overrides` receipt in `cue02_3-repair135/`, not the superseded "Each chair" recording. In all 52 cases:

- the bound text equals the current script row, word for word;
- the bound text file still hashes to the receipt's `text_sha256`, as does the embedded text for 01_1;
- where the intent embeds its text, that text equals the row.

**The bindings are unchanged.** One check needs care: a straight compare of the receipt `text_sha256` against requests05 `text_sha256` "fails" for 25 cues, but the text is the same in every one. The hashes differ only in byte convention:

| bound to | cues | why the hash differs from requests05 |
|---|---|---|
| requests04 | 27: 05_2–10_1 | none; hashes equal |
| requests02 | 21: 01_2–01_6, 02_1, 02_2, 02_4–02_6, 03_1–03_6, 04_1–04_5 | requests02 text files have no trailing newline; requests04/05 add one |
| requests03 | 3: 02_3 (override), 02_7, 05_1 | the same: no trailing newline |
| text embedded in the intent | 1: 01_1 | the intent hashes the text without a newline |

The bytes match once the trailing newline is ignored. Any future guard comparing these takes to requests05 should compare text, not raw hashes, or it will report 25 false mismatches.

## 3. Cues recorded under requests05 (10_2–10_5)

10_2, 10_3, 10_4 and 10_5 each bind `narration-frederick-requests05/text/<cue>.txt` with exactly the requests05 hash:

| cue | bound `text_sha256` |
|---|---|
| 10_2 | `776edb4b…f48f` |
| 10_3 | `c58759fb…6166` |
| 10_4 | `6ae749d4…e842` |
| 10_5 | `1cb65b88…f7d` |

Each has 2 takes.

## 4. No stale requests04 text for 10_2, 14_7, 14_8

- **10_2:** all four receipts (`10_2-intent`, `-submitted`, `-generated`, `-review`) are present. Wherever a receipt carries a hash, text or path, it is the requests05 one (`776edb4b…`, the "quotient" wording). The requests05 text holds none of the old phrases ("any piece of a share", "as of that paper, nobody knew", "shows a way out").
- **14_7 and 14_8:** no receipt exists yet, and neither cue is in `cues_generated`, `no_regenerate` or `take_overrides`. requests05 holds the corrected texts (`84300559…`, 48 words; `b09af8ff…`, 49 words), which differ from requests04.
- **Search:** no JSON file under `frederick-generation89/` or `narration-frederick-requests05/` names `narration-frederick-requests04/text/10_2.txt`, `14_7.txt` or `14_8.txt`, or any of their three requests04 hashes.

## 5. Takes preserved

I counted takes from the generated receipts: 56 cues × 2 = 112, plus the 2 superseded 02_3 takes, makes 114, equal to `takes_preserved`. All 114 have at least one byte-verified copy, at the download path or in the cue's `apportionment-frederick-probe*/` directory. Some review receipts in the probe-era format (for example 02_4–02_6 and 03_1) carry transcripts but no `takes` list, so counting from review receipts undercounts (96).

## Not mismatches (for completeness)

- The `scope` string in `requests05.json` still says "Guarded build209 pending; requests not yet authorized for synthesis until build verification." That was true when the file was written (03:40:34Z). Build209 has since passed, and `progress.json` has released the holds. It is a stale status sentence, not a binding error.
- Every requests05 entry carries `submitted: false`. That matches requests04's convention: submissions live in the generation receipts.

## Open gaps (unchanged)

No take has listening acceptance, fine sync or independent final AV acceptance. The 06_3 hold, the 08_4 and 09_4 recognition uncertainties and the 10_2 toward/towards ASR variant (in the TODO) are root's, and I did not assess them. I read no transcript; this review covers text bindings and take bytes only.
