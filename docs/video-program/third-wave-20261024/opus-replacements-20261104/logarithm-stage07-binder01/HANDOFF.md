# Logarithm stage07 binder

BronzeJaguar, for BlueSalmon, answering the stage07 binder assignment. Narration is ElevenLabs Oliver and no audio is involved here.

**State: a proposed plan, frozen for root, and not installed. Nothing here has been compiled, rendered, synthesized, measured or heard, and this is not a native, listening or independent acceptance.** Canonical and every earlier output are unchanged; the checks hash them before and after.

**This bundle is superseded for the current candidate, and its binder now refuses it. That is the stale-input refusal working.** It binds `folded_decade.rs` `c4f023ce…`. At 00:54:31Z, during my final check run, root integrated White's `digits-boundary.patch` into the candidate (`acfd77f4…`, `integration-partial06.json`). Run on today's candidate, this binder exits 3 and names that file. **Use `logarithm-stage07-binder02` for the current candidate.** This folder stays as the record of the state before that patch.

Root has since accepted the name oliver14 and written a request list for the new script, so decisions 1 and 2 below are closed. Binder02 binds the new list.

**The result:** at those inputs, the binder reads exact stage07, binds all 80 cues and all 15 holds, and refuses stale input.
- **Holds:** no marker line, say line or marker text moved. Seven holds take a new file hash, and three of those also take a new interval. Eight are untouched.
- **Waits:** the only changes are the opening handoff's two.
- **Hooks:** literal sentence hooks go from 282 to 285, which are the opening handoff's three. The 24 phrase anchors keep every cue, phrase and byte.
- **Open:** the request list and the map's name need your decision.

## Run it

From the repo root, where `<F>` is this folder. On today's candidate these all exit 3, except `--print-pins`. They ran as described on the bundle's own inputs, per the evidence below:

```
python3 -I -B <F>/bind_final80-stage07.py --check --out <F>/bound-plan      # validate the plan here
python3 -I -B <F>/bind_final80-stage07.py --out <DIR> --final-out <DIR>     # your own copy
python3 -I -B <F>/bind_final80-stage07.py --print-pins                      # to rebind on purpose
python3 -I -B <F>/checks/run_checks.py <fresh-scratch>                      # all the checks below
```

| Exit | Meaning |
|---|---|
| 0 | Bound, or `--check` passed |
| 1 | A semantic stop, or an existing output differs and was not overwritten |
| 3 | **Stale input: refused before anything is computed or written** |

`--final-out` is repo-relative. It changes only the evidence path in `direction-dispositions-proposed.json`, so that file's hash depends on where you put the plan. Without the flag it names this folder. `--out` refuses canonical and the candidate.

| `bound-plan/` file | SHA-256 |
|---|---|
| `explainer_corpus/logarithm_mysteries/narration_units.json` | `61a1f622d083a700e3749bb6a4b92c61625231033ab5aef6dcfacc16c71d7b8f` |
| `explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver14.json` | `6a326cdee5ab9e8b2055bcb2aa77ffeb16c493df16f4d0e464e448175a97cf66` |
| `hold-audit.json` | `151c4e6d81aad1a31bca8fdec30184b69d1d180546534bdfda51890a77628d1a` |
| `phrase-anchors.json` | `31a3af5097ad2a2a005058e11d1dedceb19f0089b7a0a6002011f9435f7da244` |
| `phrase-timings-unmeasured-template.json` | `fd1e12d9e5ba1b8b4fc7dc296ffa388937c2b853edc81e7d2910b9f66d706760` |
| `direction-dispositions-proposed.json` | `986609984d8325809c516937dcf97acfe963d5f8bae0ad3f151246dcc7b3a299` |
| `cue-delta.json` | `88da78a400ac8cef325de8db94483f70e263ddd8b2ba2923df6d905ceba00514` |
| `stage07-delta.json`, new: every hold, wait and hook against stage06 | `a1f0ea092079c0c0f48b6e366c415ac3a2fa532f0642b98e7c55cae44a996f33` |
| `inputs.sha256` | `6ed20e477482a2703b2c73b27e00f2f10ec97eb87a7b13fb6738dac8f99aea67` |

The binder pins 86 input files by hash, plus the candidate's chapter listing. The script pin is `b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e`.

## Decisions for you

1. **The map's name.** I named it `narration-direction-map-oliver14.json`. Canonical already holds an oliver13, bound to script `f201d2dc`, so reusing that name would overwrite a live file with different bytes. Nothing in the tools or kit hard-codes a map's name. If you prefer another, change `MAP_NAME`.
2. **No request list binds the new script.** `narration-request-final80-01/requests.json` still binds `f201d2dc`, and 01_5's new text is only in `narration-request-rounding01/01_5.txt` (`f0a6d133…`). Each row of `cue-delta.json` now gives `current_request`, the file with that cue's exact current bytes. You will need a list for `b89eab60` before synthesis. I did not write one, since that is your narration lane.
3. **Do not install two files.**
   - The candidate's own `narration_units.json` is stale: 14 units, script `15eb9ae9…`. Use the plan's.
   - `src/chapters/doing_nothing.rs.orig` has been carried along since before stage06. `mod.rs` does not declare it. The binder lists it as ignored and pins it, so moving it aside makes the binder refuse until you rebind.
4. **The next repair needs a rebind, by design.**
   - Run `--print-pins`, read the difference and edit `PINS`.
   - Add the new file's receipt to `RECEIPTS`.
   - If a wait moved, the binder stops until `SITES` agrees.
   - A wording change needs `ROUNDING` extended, because any second changed cue stops the run.

## All 15 holds

Every say line, marker line and marker is the same as in stage06. Full values, and the diff of each changed interval, are in `stage07-delta.json`.

| Hold | Kind | File and marker line | File hash | Interval, stage06 → stage07 |
|---|---|---|---|---|
| 00_4 → 00_5 | prediction | `ratio_ruler.rs:371` | **new** | `86bf44bb7022` unchanged |
| 01_3 → 01_4 | prediction | `building_ruler.rs:187` | same | `c91208d7f0b1` unchanged |
| 02_2 → 02_3 | prediction | `slide_rule.rs:204` | **new** | `e45bdd98eeb6` → **`20094536ae21`** |
| 04_6 → 04_7 | prediction | `undoing_growth.rs:749` | **new** | `b2d1d76b8f9f` unchanged |
| 05_1 → 05_2 | prediction | `complex_turns.rs:245` | same | `5256ae12dfc7` unchanged |
| 05_5 → 05_6 | prediction | `complex_turns.rs:623` | same | `93a300682e33` unchanged |
| 06_2 → 06_3 | prediction | `lifting_loops.rs:374` | same | `9c1ce6b4868e` unchanged |
| 06_4 → 06_5 | prediction | `lifting_loops.rs:475` | same | `7651918d6c2c` unchanged |
| 07_3 → 07_4 | prediction | `branch_cut.rs:419` | same | `8b3000fdb4e3` unchanged |
| 08_1 → 08_2 | prediction | `product_rule.rs:221` | same | `1d498942e2bc` unchanged |
| 08_4 → 08_5 | prediction | `product_rule.rs:346` | same | `23b0206f29d4` unchanged |
| 10_4 → 11_1 | reading | `doing_nothing.rs:602` | **new** | `eb79bd6b505c` → **`9fe875942e62`** |
| 11_3 → 11_4 | prediction | `unreachable.rs:1070` | **new** | `e34e65ad69a7` unchanged |
| 11_20 → 11_8 | reading | `unreachable.rs:1590` | **new** | `36386b6b24e1` → **`4a4e6c72d1f9`** |
| 12_5 → 12_6 | prediction | `what_it_forgets.rs:1123` | **new** | `b238a9296828` unchanged |

**What changed inside the three intervals:**
- 02_2 changes two lines, the labels for `a` and `b`.
- 10_4 changes two lines, the fine print's size, colour and row.
- 11_20 changes one line, `precisely(` to `footer(`.

**The readings' windows are unchanged:**
- 10_4 stays conditional at 197 characters, floor 13.2 s, 14.924 characters a second.
- 11_20 stays 159 + 107 characters, 17.7333 s.

| Chapter file | Stage06 | Stage07, equal to root's integration receipt |
|---|---|---|
| `ratio_ruler.rs` | `92cf028b124f8a37…` | `198d4a0586369258d845a21ee857eeb51e34357b61fe40398e5789379a6b5fd1` |
| `slide_rule.rs` | `eea4d01d96fd4853…` | `68d4b011fb4d90033ef1ae69aedef1639e12c11ce3624dbb5c378152d39b1ecd` |
| `folded_decade.rs`, no hold | `acfc7f9c198b3d3d…` | `c4f023ce5284dd768767b28c9d68a4da1e05c40385eb750dc9d4fbe583b6347e` |
| `undoing_growth.rs` | `754ea3125c2f7e89…` | `4dc4c555122d9cd5423d7094336c1d5e6f7f9afb77d6850a38bc0148c20180ef` |
| `matrix_logs.rs`, no hold | `c82b2a4d10f3c5fd…` | `7df6f8681099c76dcd1f32494e2edcbb07b427c54178d5c557d5801052f481b0` |
| `doing_nothing.rs` | `eeee764f0fe5fd3c…` | `7f445be212c586093d2092113b086d809dcc189619985baf1c8f0c407c709b47` |
| `unreachable.rs` | `a676dac78580ef73…` | `c822e8b46796e79e2f8461872fe538de711a4fde755913ec57696710d0cc28a7` |
| `what_it_forgets.rs` | `e938b750ce579fae…` | `6951d3b2f157387683efaa3ec7c9d9d12a23c8448df247531cd5f0081bab1e28` |

The candidate differs from stage06's crate in these eight files and `script.tsv`, and nowhere else. `oracle.py` and `facts.rs` have the same bytes as stage06's, and the candidate's `oracle.py` reproduces `oracle04.tsv`, so the binder still reads that table from stage06.

## SITES, waits and finishes

| Item | Stage06 | Stage07 | Class |
|---|---|---|---|
| The dots' `stage.wait(0.18)` | `ratio_ruler.rs:181`, census 00_1 | **`:210`, census 00_2** | `picture_pacing` |
| `read_out`'s `wait_until` | not present | **`ratio_ruler.rs:555`, census 00_6, runs in 00_1** | `word_sync_helper` |
| `preview()`'s tail | `ratio_ruler.rs:509` | `:509`, same | `helper_tail` |
| `kit.finish` in chapter 03 | `folded_decade.rs:441` | **`:448`** | finish |

- The other 10 waits and 12 finishes keep their lines, so there are 13 waits, not 12.
- My own scan, separate from the binder's census, finds the same 13 waits, 13 predict calls and 2 reading windows.
- The stale pair 00_1 → 00_2 now lists no wait in its final interval. That is the census's line order: `read_out`'s waits run during 00_1, but inside the voice, so it is still not a hold. `stage07-delta.json` says this too.
- All 22 helper and consumer rows keep their hash and line.

## Hooks

| Item | Stage06 | Stage07 |
|---|---|---|
| Phrase anchors | 24 in 14 cues | 24, same cue, phrase, byte, sentence and word; no hook line moved; all 24 take their file's new hash |
| Measured seconds | 0 | 0 |
| Sentence-share hooks | 3 | 3, same lines and shares: `what_it_forgets.rs:1199`, `:1202`, `:1280` |
| Literal sentence hooks | 282 | **285**: `sentence_at("00_1", 1)` at `ratio_ruler.rs:541`; `sentence_at("03_3", 0)` and `("03_3", 1)` at `folded_decade.rs:344` |
| Off-cue sentence hooks | 2, both `undoing_growth.rs:112` | 3, adding `ratio_ruler.rs:541` |
| Computed sentence hooks | `product_rule.rs:339` | Same |

**The opening's anchors are marked current baseline, and I added no hook.** `read_out` places 00_1's six numbers by character share. `phrase-anchors.json` lists them in a new section, `opening_read_out_estimates`, with status `CURRENT BASELINE`, every `measured_seconds` null and no entry in the template. The shares run from 0.0918 to 0.8937 of the cue's second sentence. They are not phrase marks, and nothing in the source reads a mark for these words. White's audit has since proposed no source change here, and one mark on 03_3's "forgets", which is the patch above and binder02's subject.

## 01_5

Only "its true position is one point" → "its position is about one point" changed, and the binder stops on anything else in the script.

- **Unchanged:** 58 words and 4 sentences; hooks 1, 2 and 3 stay in range.
- **Untouched by it:** 01_5 is not a held cue and carries no anchor. `building_ruler.rs` has the same bytes as stage06's.
- **The picture:** it already prints `pos(3) ≈ 1.585` (`building_ruler.rs:288`), so voice and picture agree in source.
- **The take:** root's review records the cue as not generated, and a take of the old text cannot bind the new hash.
- **Left as it is:** the comment at `building_ruler.rs:267` still says "true position". It is neither spoken nor drawn.

## What the adaptation changed

- **A count that would have gone wrong.** Chapter 10's repair put `precisely: ` inside `SOURCE`. Stage06's `reading_10_4` adds the prefix itself, so run unchanged it would have recorded 208 characters instead of 197. It now counts once and stops if the prefix goes.
- **The baseline walk** reads stage06's text, since the 75-cue request's words no longer equal the script's. The rewording is checked separately, and an anchor on a reworded cue stops the run.
- **Paths:** the crate is under `candidate/`. The oracle table is read from stage06, guarded by the equal `oracle.py` and `facts.rs`. The five integration receipts replace `assembly01.json` as inputs, and each must name its file's current hash.
- **New:** the pins, `--print-pins`, `--final-out`, the refusal of canonical as a target, `stage07-delta.json`, and a list of lines that read sentences in other ways.

## Evidence

`run_checks.py` runs from the repo root, with a scratch path that does not exist yet. It uses the first `python3` on `PATH` with numpy for section J, because `tools/narration_director.py` needs it. There are two transcripts:

| Transcript | Result | Why |
|---|---|---|
| `checks/outputs/run_checks.txt`, the live run | **161 of 162** | The one FAIL is `unchanged: combined-repair-stage07`, because root's integration landed during the run. All the other checks passed, including the unchanged mirror, which had been copied before the change. Kept as it was |
| `checks/outputs/run_checks-at-bundle-inputs.txt` | **162 of 162** | The same script, unchanged, on a snapshot of the repo's relevant files with root's preserved `before-oliver-digits-mark01-folded_decade.rs` in place of that chapter |

The table below is what those runs checked. A `--check` right after the live run exited 3, naming `folded_decade.rs`, pinned `c4f023ce…`, found `acfd77f4…`.

| Check | Result | What it cannot show |
|---|---|---|
| Two runs, and `--check` on this plan | Same bytes; the plan equals a fresh run | Anything about the film |
| Candidate against root's receipts | All 9 files equal what partial01–05 record | That the integration was right |
| All 15 holds recomputed without the binder | Hash, marker, say, interval, next cue, override | That the code does what the marker suggests |
| The same formula on stage06's bytes | Reproduces all 15 of stage06's values | |
| Six intervals against the repair handoffs | 00_4, 02_2, 10_4, 11_3, 11_20, 12_5 agree | Only those handoffs' own values |
| All 80 cues | Units, template, cue delta and each current request file are exact | That any take exists |
| **Stale input, in a mirror of the inputs** | 26 one-byte changes across every input class, 3 real older files (stage06's script, pre-repair `what_it_forgets.rs` and `ratio_ruler.rs`), a missing file and a stray file: exit 3, nothing written. The unchanged mirror gives identical bytes | Inputs outside the pins, such as the request texts, which root's own files pin |
| Write guards | `--out` into canonical or the candidate is refused; a differing output is kept | |
| 17 planted defects, pins bypassed in the mirror | Each reaches its own stop: unclassified wait, moved wait, lost prediction, third window, changed rate, lost prefix, sentence out of range, unreadable hook, wrong read phrase, cue order, a reworded 01_5, a second reworded cue, an unreceipted file, and others | Defects no stop looks for |
| **RubyRaven's preflight, unmodified, on this plan** | 278 ok, using its own lexer and the real `read_units`, `direction_contract`, `review_directions`, `check_dispositions`, `prediction_end`, `validate_map` and `check_cues.py` | Types, pixels, sound |
| The preflight's 8 FAILs | All listed with reasons in `checks/preflight_view.py`: 6 are the request gap above, and 2 say canonical has moved on from oliver12. With 01_5's request swapped in the mirror only, all 6 go away | That list was written by me after seeing them |
| `docs/check_cues.py` on a scratch copy | Passes; refuses a removed `say` | Cue order only |
| Canonical, stage07 and earlier outputs | Hashes equal before and after | Between the two hashes |

Two WARNs remain in the preflight, and neither is new: `requests.json` names an older stage as its path label, and Oliver maps carry no `policy` for the director's raw validation. The real `validate_map` accepts all 15 holds once those are added in memory.

The preflight's pins and assembly group did not run, because they name stage06's own repairs.

**My failed checks on the way, all fixed in the delivered scripts:**
- Two plants on the script stopped at the hash check, before the wording stops they were meant to reach. The driver now sets the expected script too.
- The derived-request control failed on its own mirror's moved inputs. That is now listed as expected in that mode only.

**What I read:** the whole stage06 binder and the opening handoff; `ratio_ruler.rs:130`–`:219` and `:500`–`:569`; `doing_nothing.rs:60`–`:79` and `:560`–`:614`; `building_ruler.rs:264`–`:307`; `folded_decade.rs:338`–`:356`; and RubyRaven's preflight from `:322` to `:950`. This is a binding audit, not a chapter review.

## Still open

- **Native:** Cargo, tests and Clippy on these exact bytes. Your last integration receipt lists the native build as pending.
- **Timing:**
  - No take is bound, and the 24 anchors have no measured marks.
  - 01_5 has no take.
  - `read_out`'s shares are estimates against real Oliver word timing.
  - In 00_2 the ticks and dots take about 2.46 s by source before `until_sentence("00_2", 1)`, and whether the first sentence is that long is unmeasured.
  - 10_4's window can be zero.
  - 11_20's 17.73 s and the 13 prediction floors are source constants.
- **Listening:** none. Nothing here says how any of this sounds.
- **Independence:** not independent.
  - I wrote chapters 09–12's repairs that this binds, as well as this binder and these checks.
  - RubyRaven's preflight is the nearest thing to a second opinion. It has another author and its own lexer, but it was written for stage06, we share one model and one auto-memory directory, and I chose how to run it and which FAILs to expect.
  - I did not trace the first author of the stage06 binder or the opening repair.
- **Provider history:** `requests.json` records 80 unsubmitted and 80 unchecked as of its own date. I did not look further.

I changed nothing in canonical, stage07, Beads or any sealed folder, and started no agents.
