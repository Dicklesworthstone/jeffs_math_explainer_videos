# logarithm-narration-clarity-repair01: 04_4 "base" and 08_5 "halving", reworded so the ear has one reading

WhiteSwan, 2026-11-05. Answers root's Mail 3668. This is a source proposal against `combined-repair-stage08/candidate`, frozen for root's independent review and integration.

**Nothing here has been compiled, rendered, synthesized, measured or heard.** I have listened to no take. The candidate, canonical `explainer_corpus/`, every take and receipt, and every sealed bundle are untouched. The only checks are on source text and formatting: `checks/source-checks.out` is RESULT PASS.

**Voice.** The owner's direction of 2026-11-05T03:14:34Z (`docs/video-program/NARRATION-CONSISTENCY.md`) makes ElevenLabs Frederick Surrey the voice for new production, including this rebuild.
- Every Oliver take, including the four the recognition review read, stays as historical and test input.
- Every cue of this film needs a Frederick take anyway, so this rewording adds no synthesis. It changes only the text root sends for 2 of the 80 cues.
- Root verifies the provider voice and does all synthesis. Phrase marks must be measured fresh on the Frederick takes.

## What the evidence says

The evidence is `targeted-recognition-small01/review01.json`: root's whisper.cpp `small.en` run over the two Oliver takes of each cue.

| Cue | Script | Both transcripts |
|---|---|---|
| 04_4 | "which is why **this base** is called natural" | "which is why **this space** is called natural" |
| 08_5 | "**Halving** the lost lap leaves a half turn" | "**Having** the lost lap leaves a half turn" |

Root's note on this: it corroborates an uncertainty and does not establish a spoken error. I agree, and I add nothing about how Oliver said the words.

What makes the evidence worth acting on is that both confusions sit in the words, not in one voice. A reroll, a direction note or a new voice reading the same text leaves the same ambiguity:

- **"this base" / "this space".** The /s/ of "this" runs straight into "base". English /p/ after /s/ is unaspirated, so "this space" carries nearly the same sounds as "this base". What separates them is whether the stop is voiced, and that is a weak cue in this position. That holds in any accent, for a listener as well as for a recognizer.
- **"halving" / "having".** In General American English, "halve" and "have" are pronounced the same, so "halving" and "having" are one sound. In Southern British English they differ only in the vowel, and neither has an /l/. The Oliver receipts name a British voice. I have not checked Frederick Surrey's accent. In either case the word rests on at most one vowel.

## Proposal

Each cue gets one replacement, inside one sentence. Sentence counts, every other sentence and all the mathematics stay as they are.

| Cue | Sentence | Now | Proposed |
|---|---|---|---|
| 04_4 | 6th (index 5) | A small percentage change becomes the same small step, which is why **this base** is called natural. | A small percentage change becomes the same small step, which is why **base e** is called natural. |
| 08_5 | 5th (index 4) | **Halving** the lost lap leaves a half turn, and a half turn is a factor of minus one. | **Taking half of** the lost lap leaves a half turn, and a half turn is a factor of minus one. |

### 04_4: "which is why base e is called natural"

- **No /s/ before the /b/.** "Base" now follows the vowel of "why".
- **It says what "this base" left unsaid: the base is e.** 04_3 has just said "The natural logarithm of e is one". 04_5 goes on to "Other bases are the same ruler stretched" and "the logarithm base two", and "base e" sets up both.
- **The voice now says the card's words.** The line on screen in that sentence is "that is why base e is called natural" (`undoing_growth.rs:554`, shown at `until_sentence(stage, "04_4", 5)`, line 550). Today the voice says "this base" while the card says "base e".
- **Size.** The cue stays at 71 words and gets 3 characters shorter. No source change is needed: 04_4 has no phrase anchor, and its six sentence hooks keep their sentences.
- **Considered and not chosen.**
  - "which is why e is called the natural base" is also clear, but it no longer matches the card.
  - "which is why the base e is called natural" adds a word and gains nothing.

### 08_5: "Taking half of the lost lap leaves a half turn"

- **It uses the cue's own word for the operation.** The cue's second sentence defines the square root as "e to the power of half a logarithm". The purple line on screen at this sentence is ½(2πi) = πi (`HALVED_LAP`, `product_rule.rs:490`). "Taking half of" names that same ½ with the word the cue and the picture already use. "Taking half of the lost lap" has no second reading.
- **Both hooks keep their exact words.** `HALF_TURN_PHRASE` ("leaves a half turn") and `FACTOR_PHRASE` ("a factor of minus one") stay verbatim, once each, in the same sentence and the same order. So both `until_phrase` waits (lines 438 and 456) and the swap between them still hold.
- **Either reading gives the right mathematics.** "Taking half of … leaves" can be heard as "half of it" or as "what remains after removing half". Both give π, a half turn: half of 2π is π, and so is 2π minus half of it.
- **Size.** The cue grows from 81 to 83 words and by 7 bytes.
- **Considered and not chosen.**
  - "Half of the lost lap is a half turn" drops `HALF_TURN_PHRASE`, so the hook, its anchor and the Rust test would all have to move.
  - Keeping "Halving" and rerolling, or adding a direction note, leaves the same words.

## The patch

`logarithm-narration-clarity.patch` is git-style with `a/`/`b/` headers. It goes against the candidate root, where paths start at `explainer_corpus/logarithm_mysteries/`. It touches two files:

**`script.tsv`:** rows 04_4 and 08_5 only.

**`src/chapters/product_rule.rs`:** three lines. No line moves; the file stays 830 lines long.
- Line 432: the scene comment that quotes the narration.
- Lines 722–723: the opener in `the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays`, plus its comment. The test looked for "Halving the lost lap" and would panic on the new text, so this edit is required.
- The identifier `halving` stays. It is a name, not narration, and the test's `found(...)` order list uses it.

**Not changed:**
- `undoing_growth.rs`: its card already says "base e".
- `docs/storyboard.md:141` ("halving the lost lap leaves a factor of −1"): it is a written description of the picture and reads unambiguously. Root may align it.
- `narration_units.json` and the direction map: their rebinding is listed below and left to root, because the map's name and the unit ids say "oliver" and naming them under the new voice is root's call.

| File | Baseline (stage08 candidate) | Expected |
|---|---|---|
| `script.tsv` | `b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e` | `559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a` |
| `src/chapters/product_rule.rs` | `f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db` | `bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635` |
| `src/chapters/undoing_growth.rs` | `4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef` | unchanged |
| `narration_units.json` | `0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c` | unchanged by the patch; rebind below |
| `docs/narration-direction-map-oliver15.json` | `c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4` | unchanged by the patch; rebind below |

## Binding impact

`binding-inventory.json` holds every value exactly, with old and new. The checks compute it from source and compare it byte for byte. Each formula first reproduces root's recorded value on the unpatched copy before it computes a new one.

1. **Script.** `b89eab60…` becomes `559afa36…`.
2. **Cue texts.**

   | Cue | Old text sha256 | New text sha256 |
   |---|---|---|
   | 04_4 | `6a85a6bd…` | `7e4e9612…` |
   | 08_5 | `f91292cc…` | `8e26bcb0…` |

3. **Direction map `oliver15`.**
   - `scripts.logarithm_mysteries` gets the new script hash.
   - The 08_1 and 08_4 visual holds change `sha256` and `evidence.sha256` from `f2c8684a…` to `bc9fbe1d…`. These are the map's only 4 mentions of the file.
   - Their lines (221, 346), markers and interval hashes (`1d498942…`, `23b0206f…`) are unchanged. Both intervals were recomputed with root's formula on both copies.
   - The other 13 holds bind files the patch does not touch.
4. **`narration_units.json`.**
   - Two unit texts change: `oliver_04_4` and `oliver_08_5`.
   - So do `script_sha256` and `direction_contract.script_sha256`.
   - `direction_contract.map` must change too, once root has rebound and named the map.
5. **Phrase anchors** (`bound-plan03/phrase-anchors.json`). Both 08_5 anchors stay in sentence 4, and their hooks stay at lines 438 and 456. Each one's `cue_text_sha256` and `source_sha256` also change. 04_4 has no anchor.

   | Anchor | Byte (end) | Word index |
   |---|---|---|
   | `HALF_TURN_PHRASE` | 260 (278) → 267 (285) | 51 → 53 |
   | `FACTOR_PHRASE` | 299 (320) → 306 (327) | 60 → 62 |

6. **Hold audit.**
   - The 08_1 and 08_4 rows change `source_sha256`.
   - 08_5 goes from 81 to 83 words; the film total goes from 4592 to 4594. The maximum stays 95.
   - Line numbers do not move, so 08_5's phrase wait (line 606) and its chapter finish (line 468) keep theirs.
7. **Requests.** `narration-request-final80-01/04_4.txt` and `08_5.txt`, which `narration-request-final80-02/requests.json` lists, carry the old text and are superseded. Root writes the new requests.
8. **Takes.**
   - Oliver request receipts in `ui-takes01` match the script for 80 cues before the patch and 78 after. Only 04_4 and 08_5 drop.
   - The four takes the review read are listed by hash and stay where they are.
   - None of them is Frederick production audio. Under the voice change every cue needs a Frederick take with freshly measured marks, including the two 08_5 anchors at their new bytes.
9. **Canonical.** `explainer_corpus/logarithm_mysteries` holds older bytes. This patch is bound to the stage08 candidate only.

## Checks

```
python3 -I -B checks/source_checks.py ROOT_LANE NEW_EMPTY_SCRATCH_DIR
```

`ROOT_LANE` is root's `elevenlabs-logarithm-rebuild-20261104` folder, read only. The script copies the candidate twice: one copy is the control, the other is patched.

**Baseline:**
- the candidate is root's stage08: `integration01` rows, baseline rows, and the units → map → script hash chain;
- the recognition transcripts say what the table above says;
- the four takes were requested with today's text.

**Replay:**
- `git apply --check` and `patch -p1` are clean;
- the expected rows match;
- no new `.rej` or `.orig` appears. The candidate already holds a `doing_nothing.rs.orig`, which is untouched.

**Wording:**
- only 04_4 and 08_5 change, each by exactly the replacement above;
- the sentence counts stay at 6 and 6, and exactly one sentence per cue changes;
- every listed statement of the mathematics is still said, and no digits appear;
- both cues stay under 100 words;
- 04_4 ends on its card's words.

**Hooks:**
- The candidate's `docs/check_cues.py` passes on both copies with an identical report.
- A mirror of the Rust test's text conditions:
  - passes on old text with the old opener, and on new text with the new opener;
  - fails when crossed, which shows the test edit is needed.
- `product_rule.rs` changes only lines 432, 722 and 723. Those are a scene comment and the test module.
- rustfmt (nightly-2026-08-31, edition 2024, width 120 as in the repository's `rustfmt.toml`) is clean on both copies.

**Bindings and inventory:** as listed above, and equal to `binding-inventory.json`.

I also ran the skill linter (`lint_script.py`) on both copies, from my scratch; it is not shipped:
- Both copies give 0 errors and the same 81 warnings.
- The warnings for these two rows are unchanged: "6 sentences; split the line at a picture change". The film's sentence hooks rely on those six sentences, so I left them.

## For root

1. Review the two sentences. The new text is exact in `binding-inventory.json` (`cues[].new_sentence`).
2. If you accept them, apply the patch to the candidate, or carry the two rows and three lines to canonical. Then rebind items 3–7. The map's and units' names under the new voice are your call.
3. For synthesis, use Frederick Surrey only, after you have verified the provider identity. The two changed cues need the new text; every other cue needs Frederick takes anyway.
4. After synthesis:
   - Run recognition on the new takes, with a written transcript review. I make no claim about how the new words will transcribe: this is wording with less ambiguity, not a verified result.
   - Measure the phrase marks at the new bytes.
5. In the guarded native run, `product_rule.rs`'s `the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays` must pass. Its edit is a string literal I have not compiled.

## Seen, not changed

**08_5, last sentence, "the product of the roots".** `small.en` wrote "routes" in take 2 and dropped the word in take 1.
- "Roots" and "routes" sound the same for many speakers. The sentence's own "the root of a product" fixes the meaning, and the wording is the standard statement of the rule, so I propose no change.
- Whether take 1 actually lacks the word is a question for your transcript review. I have not listened. Under the voice change, the new takes decide it.

## Limits

- Nothing was compiled, rendered, synthesized, measured or heard. No recognition was run on the new words, because no audio of them exists.
- I have not checked Frederick Surrey's accent. The argument for both rewordings does not depend on it.
- Timing:
  - The 08_5 phrase waits move with their bytes.
  - Until marks are measured, each wait's character-share estimate shifts slightly.
  - 04_4's last sentence gets 3 characters shorter.
  - None of this is a measured timing.
- Ownership: Beads, TODO, integration, synthesis and every native job are root's. Outside this folder I wrote only to my scratch and to Agent Mail.
