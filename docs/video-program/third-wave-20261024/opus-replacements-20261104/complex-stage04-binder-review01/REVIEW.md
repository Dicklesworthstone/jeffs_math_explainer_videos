# complex-stage04-binder-review01: review of complex-stage03-binder01 against stage04

LavenderPike, for BlueSalmon; cc BronzeJaguar. Bead `mx-video-program-program-87do.39`. Date 2026-11-05 (UTC). Narration stays ElevenLabs Oliver. Root's request: Mail 3650.

**State: source-text review, frozen for root.**
- Nothing compiled, rendered, synthesized or heard, and no timing measured.
- No candidate, stage, canonical, TODO or Beads edits. No take made or selected.
- I did not rerun BronzeJaguar's 64 checks. Root is replaying them.
- I ported the kit's and the crate's rules from their Rust source and re-derived what I checked from the stage04 files, without importing either binder.

**Independence, stated plainly.**
- I did not write the binder, the bound plans or the scenes.
- I did write review-fixes01, whose F3–F6 patches made stage04.
- I also wrote the cross-review findings F1/F2 that this binder answers.
- On those deltas this review is not independent.

P = `docs/video-program/third-wave-20261024/opus-replacements-20261104`. EC = `narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104`. S4 = `EC/combined-repair-stage04/explainer_corpus/complex_analysis_visual`.

## Verdict

**The bound plan is correct for stage04 and has no blocker.**

| File | sha256 |
|---|---|
| map `narration-direction-map-oliver12.json` | `70888624…` |
| units `narration_units.json` | `e776c860…` |

- Every hook I could check independently lands on words its cue says.
- The map rows point at the right source lines.
- The units carry the exact stage04 text.

One finding affects how root integrates (R1). Two correct statements in the handoff (R2, R3). One is a hardening note (R4).

## Findings

### R1 (Medium, integration): the sealed binder cannot check the plan once it is installed

HANDOFF §"The plan for stage04" (line 30) says: "If the bound plan is installed in stage04 and named in a new receipt, the tool accepts it only if it equals this binding byte for byte (check d21)." With the sealed tool, that path cannot pass:

- **The receipt is pinned by hash and path.** `STAGE04_RECEIPT` is fixed at `bind_complex_stage03.py:101`, and its hash `295c27e3…` is a pin. So:
  - Rewriting the receipt to name the installed files makes the run stop with exit 3 before any check (`:985–990`).
  - Keeping it fails the receipt check (`check_receipt`, `:294–311`). The new map is "on disk but not in the receipt", and the units "differ from its receipt hash".
- **d21 passes only because the test rewrites the pin.** `checks/run_checks.py:639–644` installs the plan in a mirror, then calls `forge` (`:490–507`). `forge` rewrites the mirror's receipt and edits the wrapper's pinned hash to match (`edit(m, WRAPPER, old_sha, sha(new))`, `:504`). The test is a fair test of the comparison at `:1183–1186`, but no sealed command reproduces it.
- **A new stage path is not supported either.** `STAGE04` is a constant (`:100`).

**What root can do:**
- Install into a new stage, or into stage04, with a new receipt.
- Check it with `checks/post_install_check.py STAGE_ROOT [RECEIPT]` from this bundle. It needs no pin change, and it passes only if both hold:
  - the stage equals stage04's receipt except for two things: the units (now `e776c860…`) and the one new map (`70888624…`); the four F2 strays may be absent;
  - script.tsv is still `12442480…`, and the units name that map by path and hash. A receipt, if given, lists exactly the files on disk.
- Its self-test, `checks/post_install_selftest.py`, ran in fresh scratch copies of stage04 and passed 7/7:

| Case | Result |
|---|---|
| Not installed | refused |
| Installed | passes |
| Installed, strays moved aside | passes |
| Stage03's units | refused |
| One byte added to `the_bump.rs` | refused |
| Receipt missing the map | refused |
| An extra file | refused |

- The alternative is a new declaration in a new bundle, as the handoff says for script changes.

### R2 (Low–Medium, timing statement): 15_2's strip is held 1.2 s after the voice, not 1.2 s plus a 0.6 s breath

HANDOFF finding 6 says: "After the voice ends come the 1.2 s beat (`closing.rs:349`) and the kit's 0.6 s breath (`BREATH`) before 15_3." The breath does not add to the beat. In the source:

- **The beat.** `closing.rs:347` waits for `line.end()`, then `:349` pauses 1.2 s.
- **`kit.hold`, then `say(&kit, stage, "15_3")`.** These are `closing.rs:352–353`. Both wait from the end of the line, not from the current time:
  - `wait_gap` targets `busy_until + gap` (`narration.rs:1148–1152`, `1164–1172`, `1135–1146`);
  - `busy_until` is the 15_2 start plus its length (`:1229`).
- **The gap is under 1.2 s in both modes.**

  | Mode | Gap |
  |---|---|
  | `LineGaps::Constant` (stage04's mode, `main.rs:39`) | `BREATH` = 0.6 s |
  | `LineGaps::Thought` | at most 0.6 s for the hold, and Beat 0.81 s less the takes' quiet edges for the say (15_3 opens "One", which is not a topic opener or a connective) |

  So both targets are already past, and neither waits.
- **The result.** The finished strip stands 1.2 s (rounded up to a frame) after the voice. Then 15_3 starts, and the strip fades over 0.5 s under it (`:357`).

**What the strip actually gets.** These are source estimates (`checks/outputs/binder_review.txt`; draw.rs character shares). The two Oliver rates come from WAV file lengths of the four existing takes (01_1, 11_7): 0.375 and 0.400 s a word. They are not aligned.

| Rate | 15_2 length | Last panel lit | Last caption complete for | of which under the voice |
|---|---|---|---|---|
| scratch, 2.6 words/s | 32.3 s | 22.1 s | 10.6 s | 9.4 s |
| Oliver 0.375 s/word | 31.1 s | 21.6 s | 9.8 s | 8.6 s |
| Oliver 0.400 s/word | 33.2 s | 23.1 s | 10.4 s | 9.2 s |

- No panel's 0.9 s fade runs into the next panel's hook at any of the three rates.
- The two-line captions are 27–38 characters, about 1.8–2.5 s at the map's own chars/15 floor.
- Each caption appears as its sentence starts, and that sentence then says the caption's words. mod.rs's reading rule (`READING_WINDOWS` comment, `mod.rs:47–57`) lets such speech share the reading.

**Recommendation.** On source evidence, 1.2 s needs no lengthening. Judge F6 on 1.2 s, not 1.8 s, and confirm it on the narrated draft's cue sheet. The handoff's sentence should read "the 1.2 s beat, which the kit's breath does not extend".

### R3 (Low, factual): 15_2 has no Oliver take of either wording

HANDOFF finding 7 says: "The current take's words no longer equal the script." The sealed audio inventory (`P/complex-stage04-audio-reuse01`, SHA256SUMS `6336b565…`) finds:
- no recording of stage04 15_2;
- no recording whose text holds either clause, "though not always everywhere" or "though there may be none".

15_2 is missing, like 102 other cues, not stale. The 15_2 recordings elsewhere in `narration_lab` belong to other films (stokes, crofton, parrondo). The action is the same: 15_2 needs a take from the exact stage04 text. Root's `EC/narration-request-stage04-01/text/15_2.txt` holds that text.

### R4 (Low, hardening; no stage04 defect): S1's word-start test lets a phrase start after an apostrophe

`bind_complex_stage03.py:677` refuses a phrase only when the character before it is alphanumeric. So a hook on "s recipe" inside "the bump's recipe" would pass as a word start. My scan also counts `'` and `’` as inside a word. It passes every phrase hook in the strict audit (94) and all 315 literal call-site hooks. Nothing to change for stage04. A later binder could add the apostrophe.

## Checked and confirmed (source text, no defect)

Every item below is a line of `checks/outputs/binder_review.txt`.

- **Pins.**
  - stage04 receipt `295c27e3…`, script `12442480…`, installed units `3c4f53d7…`
  - Bronze's SHA256SUMS `c66742a0…`, and every file it lists verifies
  - bound map `70888624…` and units `e776c860…`
  - kit `narration.rs` `8072720d…`
- **Stage delta.**
  - stage04 differs from stage03 in exactly the four files its receipt names.
  - The film code of `counting.rs` (1,313 lines) and `draw.rs` (1,285 lines), split at `#[cfg(test)] mod tests {`, is byte-equal in both stages.
- **Sentences.** Three splitters split all 105 cues alike, sentence for sentence:
  - the kit's `sentences()` (`narration.rs:621–641`)
  - the crate's `sentence_offsets()` (`draw.rs:82–91`)
  - `tools/coherent_narration.py`'s `sentences`

  This matters because `Spoken` uses measured sentence starts only when the counts agree, and otherwise falls back silently to character shares (`draw.rs:109–121`). No cue will fall back once a take's `report.json` exists.
- **Hooks.**
  - Each of the 105 cues is said exactly once in the 17 compiled modules.
  - The scan found 315 literal hook calls. 305 sit at the cue's own `say` binding and 10 sit in helpers.
  - Every phrase occurs once in its cue at a word start, and every sentence index is in range.
  - Every hook is in Bronze's strict audit at the same cue, file, line, call and value.
  - The audit expands each of the 10 helper hooks to a cue whose text holds the literal:
    - nine sit in 11_8's helpers, `counting.rs:792–1002`
    - `counting.rs:1148` `until "There the count"` sits in 11_4's pole walk
  - The audit's character and sentence for all 94 phrase hooks equal first-occurrence `find()` on the stage04 text.
- **11_7 (seven sentences), against `counting.rs:525–575`.** Each picture arrives on its words:

  | Hook | Words |
  |---|---|
  | "complex polynomial" | s0 |
  | `wait_sentence(1)` and `until "which leaves"` | the division, s1 |
  | "with z" | z to the n, s2 |
  | `wait_sentence(3)` | the huge circle |
  | `wait_sentence(4)` | the dog |

  The walk runs to `sentence(6)`, floored at 5 s. The floor does not bind at any of the three rates above.
- **15_2.**
  - F3 leaves sentences 0–4 byte-equal. The clause sits inside sentence 5, which goes from 143 to 139 characters.
  - The six hooks (`closing.rs:321`, `:329–333`) are unchanged in meaning: sentences 1–5 and "and it turns a real integral", all in sentence 4.
  - The five captions in `CAPTIONS` (`closing.rs:29–35`) match what each sentence says. The last, "at most one extension; paths can branch", agrees with "though there may be none".
- **Map.**
  - All 28 rows (23 holds, 5 helpers) carry stage04's file hash, and each marker is the text of its line.
  - The holds are exactly mod.rs `PREDICTS` (13, prediction) and `READING_WINDOWS` (10, reading).
  - Each `next_cue` is the script's next cue.
  - The 23 overrides quote their cue's stage04 text byte for byte.
- **Units.**
  - 105 one-cue units in script order, each text the stage04 cue.
  - The script and map hashes are right.
  - Against the installed `3c4f53d7…`, only 11_7, 15_2, `script_sha256` (`2968b0fd` → `12442480`) and the map (oliver10 → oliver12) differ.

## Estimated against measured timing

Every moment in this review is a source estimate. The only measured numbers are the WAV lengths of four existing takes. Once takes exist, the hooks in Bronze's audit split like this:

| Hooks | Count | Once takes exist | Source |
|---|---|---|---|
| Sentence hooks (`wait_sentence` 234, `sentence` 6) | 240 | measured sentence onsets from each take's `report.json` | `narration.rs:884–905` |
| Phrase hooks (`wait_for` 68, `until` 26) | 94 | still character-share estimates inside their sentence | `draw.rs:142–157` |
| `line.end()` | — | the WAV length | — |

A phrase hook becomes measured only with a `phrase-timings.json`, which must match the WAV and text hashes (`narration.rs:906–951`). Once sentence onsets are measured, a phrase hook's error stays within its own sentence.

## Rerun

From the repo root:

```sh
python3 -I -B P/complex-stage04-binder-review01/checks/binder_review.py
python3 -I -B P/complex-stage04-binder-review01/checks/post_install_selftest.py FRESH_SCRATCH
python3 -I -B P/complex-stage04-binder-review01/checks/post_install_check.py STAGE_ROOT [RECEIPT]   # after root installs
```

`binder_review.py` writes nothing and exits 1 on any FAIL. `post_install_selftest.py` writes only into its scratch folder.

`checks/outputs/` holds the two sealed outputs. Each was run twice into separate scratch folders, and the two runs were byte-equal.

## Files

- `REVIEW.md`: this file.
- `checks/binder_review.py`: the focused checks.
- `checks/post_install_check.py`: for root after installation.
- `checks/post_install_selftest.py`: its plants.
- `checks/outputs/binder_review.txt`: the sealed output of `binder_review.py`.
- `checks/outputs/post_install_selftest.txt`: the sealed output of `post_install_selftest.py`.
- `inputs.sha256`: the inputs read, repo-relative.
- `SHA256SUMS`: this bundle.
