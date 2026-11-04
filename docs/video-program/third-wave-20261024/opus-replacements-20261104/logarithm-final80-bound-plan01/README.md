# Logarithm final 80: source-bound Oliver plan

**Author:** RubyRaven, for BlueSalmon's Mail 1599.

**This is author/source binding, not independent AV or listening acceptance.** I wrote the chapter 11 and 12 repairs bound here, so this is not independent of them. Nothing was compiled, rendered, synthesized, measured or heard. Canonical is untouched.

**Narration is ElevenLabs Oliver.** No Michael, no FrankenTTS.

## Result

Everything is generated from the frozen bytes of `combined-repair-stage04` (script sha256 `f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84`).

| Item | Result |
|---|---|
| Units | 80, one per cue, in script order; 4,592 words, longest 95 (`07_6`) |
| Directions | 15 resolved by the actual `tools/coherent_narration.py`, each at its cue's last word |
| Prediction holds | 13, from the actual `predict(` calls |
| Reading holds | 2: `10_4` (conditional) and `11_20` (17.73 s window) |
| Obsolete pairs | The 4 from `oliver11` are gone |
| Phrase anchors | 24 exact-byte anchors in 14 cues; 11 sit in split cues; 0 measured |
| Inserted silence | None. All 15 proposed dispositions are `native_visual_hold` |
| Validator | PASS, and 22 planted defects refused |

| File | What it is |
|---|---|
| `explainer_corpus/logarithm_mysteries/narration_units.json` | The 80-unit plan |
| `explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver12.json` | The map |
| `hold-audit.json` | Each hold with its exact cue text and line evidence, every other wait, helpers, consumers |
| `phrase-anchors.json` | Every anchor's cue, byte, phrase and hook lines; `measured_seconds` is always null |
| `phrase-timings-unmeasured-template.json` | All 80 cues for root's aligner to fill. Its nulls make the kit's loader refuse it |
| `direction-dispositions-proposed.json` | 15 rows for root's span manifest. A proposal |
| `cue-delta.json` | Each final cue against root's 75-cue request text |
| `inputs.sha256` | Every input's bytes |
| `bundle.sha256` | Every file in this bundle except itself |

## Run it

From the repo root, with `B` set to this folder and `EMPTY` to a new empty directory each time:

```
python3 -I -B $B/bind_final80.py --check
python3 -I -B $B/checks/validate_final80.py $EMPTY
python3 -I -B $B/checks/run_check_cues.py $EMPTY
shasum -a 256 -c $B/inputs.sha256
shasum -a 256 -c $B/bundle.sha256
```

Without `--check`, the binder writes its outputs and refuses to overwrite one that differs. It stops on a wait or phrase hook it has not classified, or on a script other than the merged final 80.

"Staged root" means `coherent.REPO` is pointed at this bundle, where the map sits at its canonical relative path. The validator repeats the read on a scratch root with the canonical layout. Until root installs the script and map together, `coherent_narration.py check` run against the real repo root can't resolve this map. Use the validator until then.

## The 15 holds

The floor is the source's own constants after the voice ends, up to frame rounding. For a prediction: 0.6 s breath + 4.0 s `THINK` + 0.25 s bar fade. It is a floor because a picture that outlasts the voice delays `kit.hold`, and source alone can't rule that out. No hold adds samples to a take.

| Cue → next | Kind | Marker | Floor after voice |
|---|---|---|---|
| 00_4 → 00_5 | prediction | `ratio_ruler.rs:371` | 4.85 s |
| 01_3 → 01_4 | prediction | `building_ruler.rs:187` | 4.85 s |
| 02_2 → 02_3 | prediction | `slide_rule.rs:204` | 4.85 s |
| 04_6 → 04_7 | prediction | `undoing_growth.rs:738` | 4.85 s |
| 05_1 → 05_2 | prediction | `complex_turns.rs:235` | 4.85 s |
| 05_5 → 05_6 | prediction | `complex_turns.rs:588` | 4.85 s |
| 06_2 → 06_3 | prediction | `lifting_loops.rs:358` | 4.85 s |
| 06_4 → 06_5 | prediction | `lifting_loops.rs:458` | 4.85 s |
| 07_3 → 07_4 | prediction | `branch_cut.rs:419` | 4.85 s |
| 08_1 → 08_2 | prediction | `product_rule.rs:221` | 4.85 s |
| 08_4 → 08_5 | prediction | `product_rule.rs:346` | 4.85 s |
| 10_4 → 11_1 | reading, conditional | `doing_nothing.rs:602` | 0.6 s, plus a window of unknown length |
| 11_3 → 11_4 | prediction | `unreachable.rs:1070` | 4.85 s |
| 11_20 → 11_8 | reading | `unreachable.rs:1590` | 18.33 s |
| 12_5 → 12_6 | prediction | `what_it_forgets.rs:1121` | 4.85 s |

All 13 predictions go through `draw::predict`, which holds, marks `predict`, waits on `music_breath(THINK)` and fades (`draw.rs:352–389`). There is no code between any `predict(` call and the next cue's `say`.

**`11_20`:** 159 characters of rule + 107 of row verdicts = 266, at 15 a second = 17.7333 s, recomputed from the source's literals and the `matrix.blocks_mixed` fact. The rule's 10.6 s is the floor if root shortens the row walk. The hold has no `predict` mark, so captions are not extended.

**`10_4`:** the window is `FINE_PRINT_S − read_for`, and only if `read_for < 13.2` (`doing_nothing.rs:600–602`). `read_for` runs from when the fine print has faded in to voice end plus breath, so the length depends on the take and can be zero. I claim no number. The next cue is across a chapter join. `expanded01` marked this row at `kit.hold`; this map marks the `music_breath` that actually holds, and the audit keeps both lines.

Interval hashes use root's `expanded01` convention, which is not written down: the `\n`-joined lines from the cue's `say` through the marker. It reproduces 11 of root's 14 rows exactly. The other 3 changed because their source or marker changed.

## The four removed pairs

They are stale rows in `oliver11`, naming the retired seven-chapter draft. None of their old words are in the final cues, and no final interval has a window there.

| Pair | Old row | Status today |
|---|---|---|
| 00_1 → 00_2 | `bookends.rs:33` | File not registered |
| 01_1 → 01_2 | `real_inverse.rs:55` | File not registered |
| 03_1 → 03_2 | `branch_cut.rs:52` | Same name, now chapter 07 with other bytes; that line no longer holds the marker |
| 04_1 → 04_2 | `product_law.rs:86` | File not registered |

## Phrase anchors

`phrase-anchors.json` has all 24. The bytes are UTF-8 offsets where the source's `text.find(phrase)` lands, and the kit looks a mark up at exactly that byte. Without a mark, each helper falls back to a provisional share of a measured sentence; the file names which one.

The 11 in split cues get a new `text_sha256`, so any timing bound to the old text can't load:

| Cue | Byte: phrase | Moved? |
|---|---|---|
| 11_7 | 174 `so nothing forces a logarithm to keep the axes`; 226 `the spin, which turns every axis`; 315 `a matrix that sends some direction`; 404 `only when it does that to every direction alike`; 453 `a half turn combined with an equal stretch` | Same byte |
| 11_20 | 91 `the matrix's blocks for each negative stretch`; 137 `come in matching pairs` | New cue; were 11_7 bytes 588, 634 |
| 12_6 | 89 `also ends here`; 105 `and so does two and a half` | Same byte |
| 12_21 | 192 `a convention broke the same tie`; 230 `the honest answer is the whole family` | New cue; were 12_6 bytes 681, 719 |

The other 13 keep their cue and byte: `02_1`, `02_2`, `04_2`, `10_3` (2), `10_4`, `11_1` (2), `11_2`, `11_5` (2), `11_6`, `12_1`. They still need measured marks, because a mark binds a WAV and no take is bound. `05_2`, `05_20`, `06_6`, `06_20` and `12_20` have no byte anchors.

Three `until_share` hooks (`12_20` twice, `12_21`) are listed apart: they read sentence onsets, not word marks.

## Findings for root

I changed none of these. Stage04 is frozen.

1. **`docs/check_cues.py` fails on stage04 as it stands (27 failures), and these look like tool artefacts.** It reads every `*.rs` whole, so 14 failures name the 5 retired chapter files and 13 come from strings in test modules. On registered scene code alone it passes (`checks/check_cues.out`). If the retired files are installed, or the checker gates integration, it will fail.
2. **`src/chapters/doing_nothing.rs.orig` is in stage04**, apparently left by a patch run. It is not compiled, but it shouldn't be installed.
3. **The storyboard's "Authored reading holds" table lists only `11_20`.** `10_4` is a real reading window in source.
4. **Two prediction cues don't end on their question.** `00_4` ends "Halfway by difference would be six hundred and sixty." and `12_5` ends "Think about the laps, the direction of spin, and the axis." The storyboard says the question ends the cue. Both read as deliberate, and I changed no wording; the words are for listening to judge.
5. **Take naming.** All 80 final cues match a file in `narration-request-text01` byte for byte, so no new wording is needed. But 9 of them are named as parts there: `05_2-part2` is `05_20`, `06_6-part2` is `06_20`, `11_7-part2` is `11_20`, `12_6-part2` is `12_20`, `12_6-part3` is `12_21`, and the four `-part1` files are `05_2`, `06_6`, `11_7` and `12_6` (`cue-delta.json`). External receipts must use the final cue ids, because `unit_spans` matches on the cue. `narration-request-plan01.json` still binds the 75-cue script.
6. **The score changes.** Each `predict` mark becomes a Reveal cue (`score.rs:755`), so up to 13 new audible events, as WhiteSwan's marker review noted. The `11_20` window also needs a fresh score.
7. **Director map format.** `narration_director.py` `validate_map` also wants a `policy` block, which neither this map nor `oliver01`–`11` has. I added `reason` and `evidence` to each hold row so the rows themselves fit.

## Remaining gaps

- **Not built.** Root's assembly records no native build of stage04. Cargo, tests and Clippy are root's, and hold rows must be rebound if any bound source changes.
- **No takes.** I ran no synthesis. Root's inventory found no exact local record of this text and requires a provider history check before new synthesis. I did not repeat either.
- **No measured timing.** All 24 anchors run on provisional shares until words are aligned on real takes. `phrase-timings.json` must cover all 80 cues and bind each WAV, so it can't exist until every take does.
- **No fine sync.** Hold lengths aren't checked against takes. `THINK` is still a proposal, and `11_20`'s 18.33 s hasn't been heard.
- **Holds against pictures.** The floors don't exclude a picture overrun, and this audit doesn't verify that each answer stays hidden until its hold ends.
- **Captions.** `prediction_end` ran only on synthetic numbers, never a real timeline.
- **Kit and tools bound at working-tree bytes.** `shasum -a 256 -c inputs.sha256` will show if they move. If root builds from different bytes, rebind.
- **No acceptance.** No picture review, listening or independent AV review is claimed. Someone other than me should check the source binding.

## Installing

Install as one set: `script.tsv`, the plan, the map, `draw.rs` and all registered chapters. The plan binds the script's bytes, and each hold row binds its chapter's bytes. Leave out the 5 retired files and the `.orig`. Any later source edit means rerunning the binder into a new version.
