# logarithm-clarity-integration01: stage09 candidate and rebound plan for the reviewed clarity wording

WhiteSwan, 2026-11-05, for root (BlueSalmon), answering Mail 3724 under .63. Frozen for root's independent replay
and install. Folder below: `P` = `docs/video-program/third-wave-20261024/opus-replacements-20261104`, `EL` =
`narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`.

**Nothing here has been compiled, rendered, synthesized, measured or heard.** No audio was decoded; WAV files were
only hashed. No take is selected. I ran no Cargo command. Voice: ElevenLabs Oliver for logarithm (owner refinement
2026-11-05T03:22:06Z); only 04_4 and 08_5 need new Oliver takes, and the four original takes stay where they are.

## What it is

This is integration code, not prose. `integrate_clarity.py` takes root's stage08 candidate and bound-plan03,
applies the sealed clarity patch (`2463334a`, accepted by LavenderPike's review `e78a99b4`), and rebinds everything
that patch moves. It refuses stale inputs with exit 3 and checks a future `phrase-timings.json` and the new request
receipts before the kit or root uses them.

| Path | What |
|---|---|
| `integrate_clarity.py` | The helper (`1c4186d9e7100e437eab3ab443993a414c1c644f7f77de368f0901ce35538502`). Modes: `--print-pins`, `--out NEW_DIR`, `--check`, `--verify DIR`, `--validate-timings FILE [--template]`, `--validate-receipt FILE...` |
| `pins.sha256` | Its 158 pinned inputs (`534016fd42c6a43430a5fd2795c089c03fa86e77ba5da4e35080eacc349eedf6`, also `PINS_SHA256` in the helper): stage08's 47 files by listing, bound-plan03's 10, the clarity bundle's 8, binder03, the 80 Oliver request receipts, the four 04_4/08_5 takes and their two download records, the retained inventory, root's final80-02 request list, the kit's `kit.rs` and `narration.rs`, and `tools/coherent_narration.py` and `tools/narration_cache.py` |
| `stage09/candidate/` | 48 files: stage08's 47 with the patch applied and the units rebound, plus `docs/narration-direction-map-oliver16.json`. oliver15 and the older maps are unchanged |
| `stage09/bound-plan/` | bound-plan03's files, rebound, plus `take-binding.json`, `clarity-delta.json` and `inputs.sha256` |
| `checks/run_checks.py`, `checks/run_checks.out` | The checks (`daaa1becb52c5eb217fd21b9c789c608b76b672190dbfa33252700390812e2ef`) and their transcript: RESULT PASS, 118 checks |

`SHA256SUMS` lists every file in the folder.

## Commands

Run all of these from the repo root. Each reads only, apart from the folder it is told to write.

```
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-clarity-integration01
(cd $H && shasum -a 256 -c SHA256SUMS)
python3 -I -B $H/integrate_clarity.py --print-pins          # 158 inputs, no "differs" marker
python3 -I -B $H/integrate_clarity.py --check               # VERIFY ...stage09: PASS
python3 -I -B $H/checks/run_checks.py NEW_SCRATCH_DIR       # RESULT PASS, 118 checks; prints checks/run_checks.out byte for byte
```

To install, root chooses the folder. The embedded paths assume `EL/combined-repair-stage09`:

```
python3 -I -B $H/integrate_clarity.py --out narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09
python3 -I -B $H/integrate_clarity.py --verify narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104/combined-repair-stage09
```

- `--out` writes `candidate/` and `bound-plan/` under a folder that must not exist yet.
- It refuses canonical `explainer_corpus/`, stage08, bound-plan03, `ui-takes01` and the sealed bundles.
- The bytes depend only on the pinned inputs, not on the folder, the clock or the machine. A run in a scratch copy holding only the 158 pinned inputs writes the same 58 files.
- `--verify` compares only `candidate/` and `bound-plan/`. Anything else in the folder, such as root's own receipt, is listed and not compared.

Once the new takes exist:

```
python3 -I -B $H/integrate_clarity.py --validate-receipt NEW_04_4_RECEIPT NEW_08_5_RECEIPT
python3 -I -B $H/integrate_clarity.py --validate-timings STAGE09_PHRASE_TIMINGS_JSON
```

`--validate-receipt` checks a receipt in ui-takes01's layout:
- the exact stage09 text and its hash;
- script `559afa36`, when the receipt names a script;
- an Oliver label;
- Eleven v4, stability 0.5, similarity 0.75, clone strength 0;
- one of the three WAV 48 kHz labels the 80 receipts use;
- no context field, and no other voice or engine named anywhere.

`--validate-timings` requires:
- all 80 cues and the stage09 text hashes;
- marks at exactly the anchors' bytes, with their phrases (08_5: 267 and 306);
- finite, non-decreasing seconds;
- for the 78 unchanged cues, a WAV hash that is a retained Oliver take of that exact text;
- for 04_4 and 08_5, a WAV that is no retained take.

The kit's loader (`narration.rs` 906–950) checks hashes and order but not anchor bytes. That is why marks left at
260/299 would load and fall back to character shares. `--template` accepts the unmeasured template.

LavenderPike's sealed acceptance check, on the same tree:

```
R=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-narration-clarity-review01/checks
S=$H/stage09
python3 -I -B $R/rebind_acceptance.py $S/candidate/explainer_corpus/logarithm_mysteries \
  --anchors $S/bound-plan/phrase-anchors.json --template $S/bound-plan/phrase-timings-unmeasured-template.json \
  --hold-audit $S/bound-plan/hold-audit.json
```

It gives 44 PASS and 1 FAIL, "the units' other fields are stage08's". That FAIL is the units' `scope` sentence,
which the helper rewrites on purpose (decision 2). With stage08's sentence put back in a scratch copy, all 45 pass.
`run_checks.py` shows both results.

## Hash manifest of the stage09 outputs

| File (under `stage09/`) | sha256 |
|---|---|
| `candidate/…/script.tsv` | `559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a` (the sealed expected hash) |
| `candidate/…/src/chapters/product_rule.rs` | `bc9fbe1dfad255ff2e93e59fa205ebd95435216ed98bbd7b81946f0e039c1635` (the sealed expected hash) |
| `candidate/…/narration_units.json` and `bound-plan/…/narration_units.json` | `1132a5a452b339dd4d6fb340668c525b2e10dfe892d27035ce11957e80cafce9` |
| `candidate/…/docs/narration-direction-map-oliver16.json` and its `bound-plan/` copy | `14e87893ea389ef2cdf827f0a7e59a8e2b65ac537b726d796ffceff5117e4787` |
| `bound-plan/hold-audit.json` | `f6f315d628d4f862ae9b372c0c44aea67dd4ff2e71f5c26c6b2513c9193cb1ac` |
| `bound-plan/phrase-anchors.json` | `557d74a065260303689df29fedefafa54b084a691e912844b3f8e9171b1b7c84` |
| `bound-plan/phrase-timings-unmeasured-template.json` | `94f6e564e2178188c3f6867edef7fb26f7ce5dec110348e67b0fefee48fccd2a` |
| `bound-plan/direction-dispositions-proposed.json` | `8f95fb565b9a11c53889d1bb3701cc62a1221609deca74c7ebf7e7390f21cd94` |
| `bound-plan/cue-delta.json` | `db037aeda49fd056d2ec6ec71e35f5067a328cfa1c9c2b71b0602374ead10728` |
| `bound-plan/take-binding.json` | `6140e5328267e8f0ce2169bd5dea5070982e6aa744fd3da6338e8a96bdd3dad1` |
| `bound-plan/clarity-delta.json` | `fd74a9575e002f441cd2606f661bc824d7985f0ed380e23294410a13e91d9e9a` |
| `bound-plan/inputs.sha256` | `534016fd42c6a43430a5fd2795c089c03fa86e77ba5da4e35080eacc349eedf6` (equal to `pins.sha256`) |

The other 44 candidate files are stage08's bytes.

## What moves

`clarity-delta.json` lists every change by JSON path, against stage08 for the units and map and against
bound-plan03 for the rest. In short:

- **Script and source.** Script rows 28 (04_4) and 58 (08_5) change. In `product_rule.rs`, line 432 (a comment) and
  lines 722 and 723 (the test's opener) change. No line moves. The 78 other cue texts are stage08's bytes.
- **Units.** The texts of `oliver_04_4` and `oliver_08_5` change. Both script hashes become `559afa36`. The
  contract names oliver16 by path and hash, and `scope` is rewritten.
- **Map oliver16.** The changes are `scripts`, the four `product_rule.rs` hash fields of holds 08_1 and 08_4, and
  `scope`. Overrides, hold kinds, shared helpers, lines, markers and intervals are oliver15's. No override or hold
  sits on 04_4 or 08_5.
- **Anchors.**
  - 08_5 `HALF_TURN_PHRASE` moves from byte 260 to 267 (word 51 to 53). `FACTOR_PHRASE` moves from 299 to 306
    (word 60 to 62). Both stay in sentence 4.
  - Their text and source hashes are new, and `byte_or_cue_moved` becomes true, so the count goes from 8 to 10.
  - The other 31 anchors are unchanged.
  - The rule is binder03's: `text.find` bytes, and `direction_words` for `word_index`. The helper first reproduces
    all 33 bound-plan03 rows from stage08, then rebinds.
- **Template.** It binds script `559afa36` and the new text hashes for 04_4 and 08_5. The 08_5 marks are at 267
  and 306. The 78 other entries are bound-plan03's, and every WAV and second is still null.
- **Hold audit.** It binds script `559afa36` and map oliver16. Words go from 4592 to 4594 (08_5 from 81 to 83), and
  the maximum stays at 95. Holds 08_1 and 08_4 and the registered `product_rule.rs` hash become `bc9fbe1d`; their
  intervals (`1d498942`, `23b0206f`) are re-read and unchanged. The claim is rewritten, and one `open_for_root`
  item is added for the two takes.
- **Dispositions.** They bind oliver16's hash. Each evidence is the stage09 hold audit at
  `EL/combined-repair-stage09/bound-plan/hold-audit.json`.
- **Cue delta.**
  - 04_4 and 08_5 carry their new hashes and words.
  - Each has `text_changed_from_baseline` and `reworded_since_stage06` true, and no request file.
  - Each keeps its bound-plan03 request row as `superseded_request`.
  - The counts are recomputed (reworded 1 to 3, exact request bytes 79 to 77, current request files 80 to 78).
- **Take binding.**
  - It is built from root's `full80-retained-inventory01.json`.
  - 78 cues have retained Oliver takes of their exact stage09 text: 158 takes, at least two per cue.
  - 04_4 and 08_5 have none, and their four original takes are listed as preserved.
  - `run_checks.py` re-hashes all 162 WAVs. The helper re-hashes only the four.

## Decisions for root

1. **The map is named oliver16.** It is a new file, and stage08's oliver15 is kept as history. The name is your
   call.
2. **The units' `scope` sentence is rewritten.**
   - The old sentence named `combined-repair-stage07/candidate, script b89eab60`. It also said that only 01_5's
     old text could not bind.
   - The new one names stage09 and `559afa36`, and says that no take of 04_4's or 08_5's stage08 text can bind.
   - No consumer reads `scope`. That is the reason for Lavender's single FAIL.
   - If you want stage08's sentence verbatim, change that one field. Only the units hash and `clarity-delta.json`
     would follow.
3. **Install location.** `combined-repair-stage09/{candidate,bound-plan}` appears in the dispositions' evidence
   path and in the scope sentences. Another location needs a new helper revision, which means changing the
   `INSTALL` constant.
4. **Not carried.** `stage07-delta.json` and `increment-delta.json` are history. `clarity-delta.json` records this
   increment instead.
5. **Request snapshot not refreshed.** For the 78 cues, the cue delta's submission states are bound-plan03's
   snapshot of final80-02: 35 "full pair retained", 44 "not established", and 1 pending. That snapshot is older
   than root's retained inventory, which has two takes for every cue. `take-binding.json` reports the inventory.
6. **Two "halving" mentions are left as they are** (Lavender's R6).
   - Storyboard row 141 says "halving the lost lap leaves a factor of −1".
   - The 08_5 test's `expect` message says "08_5 has the halving".
   - Neither quotes the narration, and changing either would go beyond the reviewed two-cue patch.

## binder03

binder03 cannot be rerun with revised pins, and it stays sealed. Three refusals in its own code stop it on this
script:

- line 967 refuses any script other than `b89eab60`;
- line 1348 refuses a script that differs from stage06's in more than 01_5;
- line 1454 refuses a reworded cue that carries an anchor, which 08_5 now does.

The helper therefore pins stage08 and bound-plan03 and rebinds only what the patch moves.
`clarity-delta.json` `binder03.pins_revised_here` lists the revised pins:

| Pin | binder03 | stage09 |
|---|---|---|
| script | `b89eab60` | `559afa36` |
| `product_rule.rs` | `f2c8684a` | `bc9fbe1d` |
| units | `0e27709b` | `1132a5a4` |
| `MAP_NAME` | oliver15 | oliver16 |
| candidate | stage07 | stage09 |

## How Lavender's findings are handled (review `e78a99b4`, Mails 3744 and 3762)

- **R1 (voice premise):** this bundle is Oliver only. Mail 3710 corrects the repair's Frederick framing.
- **R2 (template at 260/299):** the template is rebound to 267/306, and `--validate-timings` refuses the old bytes,
  which the kit would accept.
- **R3 (rebind route):** this helper. The cue delta no longer maps 04_4 and 08_5 to the final80-01 texts.
- **R4 (`word_index`):** the helper uses binder03's `direction_words`. 11_20 is checked: "sixty-six" counts as two
  words. The checks key anchors by (cue, phrase), because 10_3's consts are null.
- **R5:** no change.
- **R6:** decision 6.

BronzeJaguar's Mail 3778 says `prepare_audio.py` needs no edit for stage09. Its new binding pins:
- the anchors' `cue`, `byte`, `byte_end`, `phrase` and `cue_text_sha256`;
- the template's marks;
- the dispositions.

All of these are in `stage09/bound-plan/` at the hashes above.

## What `run_checks.py` shows (118 checks, all PASS)

- **pins:** every input matches its pin.
- **replay:**
  - `--check` passes.
  - Two fresh `--out` runs are byte-identical to `stage09/` and to each other.
  - Two runs of the checks print the same transcript.
- **mirror:**
  - A scratch copy of only the pinned inputs writes the same bytes.
  - Exit 3 with nothing written for each of these: a changed source, template, patch or pins file, a removed
    original take, and a file added to stage08.
  - `--out` refuses the guarded folders.
- **patch:** `git apply` on a scratch copy of stage08 gives the helper's script and source, and the other 44 files
  are stage08's.
- **recompute:** this is independent code. It recomputes:
  - all 33 anchors;
  - the template marks;
  - the 15 hold intervals and their file hashes;
  - word counts;
  - unit texts;
  - all 44 crate (path, sha256) pairs the plan names;
  - the 78 untouched cues in the script, units, template, cue delta and take binding.

  It also confirms that no old hash is left outside the superseded rows.
- **consumers:**
  - `tools/coherent_narration.py` `read_units`, `direction_contract` and `review_directions` accept the plan laid
    out as root would install it.
  - They refuse stage08's units, a stale map hash, and bound-plan03's evidence.
  - `docs/check_cues.py` passes on stage09 and on stage08.
  - `rustfmt +nightly-2026-08-31 --check` passes.
  - The 08_5 Rust test, mirrored, passes old/old and new/new and fails crossed.
  - `phrase_seconds`, mirrored, reads marks at 267/306 and falls back to shares at 260/299.
- **rejections:**
  - Timings: 13 template cases and 9 measured cases. These cover old bytes, old hashes, missing, extra, renamed or
    swapped marks, a changed neighbour, an original take, another cue's take, and bad seconds.
  - Receipts: 11 cases, including Frederick Surrey, the stage08 wording, a neighbour cue, other settings, context,
    and the old script. Lavender's receipt check agrees on the control and on the Frederick case.
  - `--verify`: 11 cases, covering stale units, map, template, anchors, hold audit and evidence, a reworded
    neighbour, a stray `.orig` file and a missing map.
- **lavender:** the result above.
- **hygiene:** no machine paths, names, addresses or gendered pronouns, and no `.rej` or `.orig` files of mine. The
  candidate's only `.orig` file is stage08's own `doing_nothing.rs.orig`.

## Open for root

- The native build and tests of these exact bytes. Lines 722 and 723 of `product_rule.rs` are in a test.
- Two targeted Oliver takes of the reviewed wording, checked with `--validate-receipt`.
- Fresh sentence onsets for 04_4 and 08_5, and the 08_5 marks at 267 and 306, checked with `--validate-timings`.
- The joins 04_3/04_4/04_5 and 08_4/08_5/09_1.
  - If 04_4's length changes, 04_5 to 04_7 shift within chapter 04.
  - 08_5 ends chapter 08.
- Selection, listening, and AV acceptance.

## Limits

- This is source text and hashes only. A PASS says nothing about how a take sounds or when a word is spoken.
- `rustfmt --check` is the only Rust tool I ran. The Rust test is mirrored in Python, not run.
- The consumer checks call the tools' Python functions. They point `coherent_narration`'s `REPO` at the candidate,
  because the tool resolves maps against its own tree.
- The helper and the checks share no code. The checks rewrite binder03's word and sentence rules from scratch.
- I also wrote the clarity patch, so this binding is not independent of it. Lavender's check is independent but
  shares the kit and binder rules.
- No Beads, TODO, commits, synthesis or root publication. Root's stage08, bound-plan03, ui-takes01 and every sealed
  bundle were only read.
