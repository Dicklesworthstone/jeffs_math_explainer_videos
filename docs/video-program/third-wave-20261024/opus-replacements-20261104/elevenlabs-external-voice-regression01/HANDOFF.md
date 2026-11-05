# elevenlabs-external-voice-regression01

WhiteSwan, for root Mail 3917, with the pin qualification Mail 3920 asks for. Frozen once sealed.

**Nothing here was compiled, rendered, synthesized, transcribed or heard. No production audio was opened or decoded.** The tests write synthetic PCM only, so a PASS says nothing about whose voice a real take holds. No canonical, root, Bead, TODO or sealed file was edited.

## What to apply

`patch/regression-tests.diff` adds 194 lines and removes none. Every original test is kept byte for byte.

| File | Base | Patched | Added |
|---|---|---|---|
| `tools/test_coherent_narration.py` | `074bf1b2` | `19487bd0` | 164 |
| `tools/test_narration_delivery.py` | `bd45cb3d` | `cd4d8dfa` | 30 |

- Apply both files together: the delivery tests use the new coherent fixture helpers.
- From the repository root, run `git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/elevenlabs-external-voice-regression01/patch/regression-tests.diff`. `git apply --check` passes against the canonical files today.
- They join the normal suite, not a separate gate. `python3 -I -B -m unittest discover -s tools -p "test_*.py"` then runs 52 tests (44 + 8), all passing on the installed tools (coherent `95615cfa`, delivery `961f17cc`).

## The eight tests

| Mail 3917 item | Test | What it checks |
|---|---|---|
| Frederick assemble + verify | `test_frederick_ui_takes_assemble_and_verify_sample_for_sample` | Two one-cue Frederick v2 takes assemble, then pass strict `verify` with the script, units and directions. Each output cue equals its take sample for sample. `assembly_voice` names Frederick. |
| Frederick render consumer, legacy Oliver positive | `test_frederick_and_legacy_oliver_ui_assemblies_reach_render` (delivery) | Frederick v2, all-legacy Oliver, and v2 Oliver beside legacy Oliver each go through the real CLI, `--consumer render`, with `external-units.json`. Each exits 0 with `plan_mode` `external-ui` and `listening_acceptance` false. |
| Legacy Oliver positive | `test_legacy_and_v2_oliver_receipts_assemble_as_one_voice` | All-legacy, and v2 Oliver beside legacy, assemble and verify as one voice. |
| Mixed refusal | `test_one_assembly_refuses_two_narrator_voices` | Frederick + legacy Oliver, both orders; v2 Oliver + Frederick; native + Frederick; native + legacy Oliver. `assemble` refuses each with "one narrator voice" and leaves no output folder. |
| Post-assembly swap | `test_verify_refuses_a_voice_swapped_in_after_assembly` | After an all-Frederick assembly, one receipt is rewritten as a self-consistent Oliver v2 receipt. `unit_spans` alone accepts it; `verify` refuses it. |
| Swap and contradiction at the consumer | `test_render_refuses_swapped_voice_and_legacy_contradiction_cleanly` (delivery) | `render` refuses the swap with one `narration delivery refused:` line (no traceback), and passes again once the receipt is restored. It also refuses a legacy receipt rewritten after assembly to state Frederick. |
| Legacy contradiction refusal | `test_legacy_oliver_receipt_cannot_state_another_voice_or_request` | A legacy Oliver receipt stating Frederick (receipt or record voice), or carrying a request, is refused by `unit_spans` and `assemble`. `Oliver` (the short name Bronze's helper writes) and the full Oliver voice stay accepted. |
| **Beyond the list** | `test_frederick_receipt_binds_provider_voice_and_request` | 11 refusals on one Frederick receipt: provider; receipt voice; record voice; missing `request_sha256`; request digest; request cue, text and voice (each rewritten and rehashed); a near-miss label; Frederick without the schema; an unknown label without the schema. |

**Why the extra test.** Without it, 8 of the 18 weakened copies below (M5–M12) pass the whole canonical suite. Root can drop it: the other seven do not depend on it.

**Fixture choices**
- Labels and voices are written out in the test, not read from `EXTERNAL_UI_VOICES`, so a changed table entry fails.
- The request keys follow the real `frederick-stage05-01` requests (`cue`, `text`, `voice`, `model`).
- The existing fixture's direction contract is reused, with one-cue units as external plans require.

## Replay

```
B=docs/video-program/third-wave-20261024/opus-replacements-20261104/elevenlabs-external-voice-regression01
(cd $B && shasum -a 256 -c SHA256SUMS)
python3 -I -B $B/checks/regression_checks.py NEW_SCRATCH_DIR
```

- It writes only under `NEW_SCRATCH_DIR` and takes about 1.5 minutes, running the suites in parallel.
- It expects `RESULT PASS, 0 failed`, which is `checks/regression_checks.out`, byte-identical over two runs.
- It exits 3 if a pinned input moved.
- The canonical test files may be at their base or already patched; the transcript names which.

**Sections**
1. **Pins:**
   - the installed tools;
   - the test files;
   - root's preserved pre-install bytes (`before-*.py`, `receipt01.json`);
   - the seals of repair01 `3df152e4`, review01 `0da3fa50` and clarity-integration01 `5fcb5ea6`.
2. **The diff:** it rebuilds the patched hashes, removes 0 lines, passes `git apply --check` on canonical, and both files compile.
3. **Suites on the installed tools:** the original suite passes 44 of 44, the patched suite 52 of 52.
4. **Controls:**
   - **Pre-install tools:** the 44 original tests pass, and all 8 new tests fail.
   - **The reviewed repair without the F2 fix:** pre-install bytes plus Bronze's two diffs, via `git apply`. Adding review01's fix to them gives the installed bytes exactly. The 44 pass, and exactly the 2 legacy-contradiction tests fail.
   - **18 weakened copies of the installed tools:** the patched suite fails on all 18. Sixteen of them pass all 44 original tests, and each of those sixteen fails a new test.

| Copy | Change | Caught by |
|---|---|---|
| M1 | no one-voice guard in `verify` | swap (coherent + render) |
| M2 | no guard in `assemble` | mixed refusal |
| M3 | the guard folds every UI voice into one | 5 new tests |
| M4 | the Frederick label is missing from the table | 6 new tests |
| M5–M12 | provider; record voice; v2 record keys; request digest; request cue; request text; request voice; the native-key check | the binding test |
| M13–M15 | the legacy check ignores record voices; allows a request; refuses the short name `Oliver` | the legacy-contradiction test |
| M16 | delivery accepts any receipt | the original `ExternalPlanTests` (no new test needed) |
| M17 | delivery diff not applied | both delivery tests |
| M18 | only the delivery diff applied | all 8, plus 1 original |

5. **Mail 3920:** the qualification below.
6. **Hygiene:** no machine path, user name, address or gendered pronoun.

## Mail 3920: the old pins of logarithm-clarity-integration01

My sealed `logarithm-clarity-integration01` (`5fcb5ea6`, which root installed at `EL/combined-repair-stage09`) pins coherent `e0aa81f3` in `pins.sha256` and `stage09/bound-plan/inputs.sha256`. Its hold audit also anchors coherent `e0aa81f3` and delivery `c91914b8`. I did not edit it, and section 5 checks it read-only.

- Of its 158 pinned files, only `tools/coherent_narration.py` moved, so its `--check` now exits 3 at that pin, as designed. The old hash is root's preserved pre-install coherent.
- Top-level code that differs:
  - **coherent:** `assemble`, `unit_spans` and `verify`, plus the new voice names and constants;
  - **delivery:** `external_plan` only.
- `integrate_clarity.py` calls only `coherent.read_script` and `coherent.direction_words`. Its consumer check calls `read_units`, `direction_contract` and `review_directions`. Neither reaches any changed code (2 and 9 names).
- The consumer replay on the sealed stage09 prints the same lines with the pre-install and the installed coherent, equal to the sealed transcript:
  - stage09 units: `ACCEPTED 80 15`;
  - stage08 units: `REFUSED ValueError unit plan does not bind current script bytes`.
- **The four hold-audit anchors** each occur once in the installed bytes, inside unchanged functions. New lines:

| File | Old line | New line |
|---|---|---|
| coherent | 74 | 141 |
| coherent | 123 | 190 |
| delivery | 242 | 242 |
| delivery | 318 | 320 |

- **Exception: delivery anchor 318 sits in `verify`, which reaches changed code** (`external_plan`, and coherent's guard via `coherent.verify`).
  - For legacy Oliver receipts like logarithm's, that code adds only two refusals: a mixed voice, and a receipt stating another voice or a request. A receipt that is not a JSON object is also refused now. v2 receipts get the full v2 binding.
  - review01's scan found neither in the 91 retained Oliver-label receipts.
  - Bronze's Mail 3994 helper writes `voice: Oliver`, which is accepted.
  - It was not run on a production assembly, because that would decode production audio.

**Rebinding.** These are the edits root can make in its installed copy, if wanted; changing them in my sealed folder would break its seal:
- `pins.sha256` and `inputs.sha256`: `e0aa81f3… tools/coherent_narration.py` becomes `95615cfa… tools/coherent_narration.py`.
- Hold-audit consumers: coherent `95615cfa`, lines 141 and 190; delivery `961f17cc`, lines 242 and 320.

**Replay effect.** review01's replay now exits 3 by design, because it pins the pre-install tools as its base. Its sealed transcript stands, and this bundle replays from root's preserved bytes instead.

## Limits
- Synthetic and source checks only: no Rust, production audio, real-take assembly, ASR or listening.
- The tests port my own review01 scenarios, as Mail 3917 asked, so they are not independent of that review.
- F1 (a Frederick v2 producer) is not covered here. Lavender's Mail 3978 reports one for complex, which I have not reviewed.
