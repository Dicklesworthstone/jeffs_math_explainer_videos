# Logarithm 05–08: teaching repair of C1, L1, R10, C2 and B8

LavenderPike, for BlueSalmon, answering root's assignment after review05-08 (Mail 3288, thread 3079) and root's recognition note on 05_20. Narration is ElevenLabs Oliver. Every retained take still applies, because no cue's words change.

**State: staged and frozen for root. Nothing here has been compiled, tested by Cargo, checked by Clippy, rendered or heard.** The candidate and canonical are untouched. The baseline is `combined-repair-stage07/candidate` under `narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104`. The four chapter files there still carried the hashes below when this folder was sealed; `run_checks.py` checks them first.

## The repair

| Item | Cue | Baseline | Was | Now (staged lines) |
|---|---|---|---|---|
| **C1** | 05_20 | `complex_turns.rs:349-366` | The whole line `e^{ln 2 + iπ/2} = 2i`, its arrow and the `2i` label arrive together as "For instance" starts, about 9 to 10 s before "so we land on two i". | `:349-377`. The line is built in three stages, each on its own words. `e^{ln 2 + iy}` comes on "the logarithm of two". `iy` becomes `iπ/2` on "a half pi". The words card comes on "the length is two". `= 2i` comes on "land on two i", then the arrow and label. The stages stand on the whole line's lower-left corner (`example_part`, `:770-775`), so shared glyphs stay put. The finished picture is unchanged. |
| **L1** | 06_6 | `lifting_loops.rs:585-594` | The dot crosses −1 and the gauge drops during "The gap sits at zero until". The inset is up about 5.7 s before "leaps". | `:584-600`. The walk holds at turn 0.45 until "passes minus one". The ring at −1 then fades in, and the dot crosses about 1.0 s into the phrase. The inset waits for "leaps from pi to minus pi". The final walk is 1.2 s (was 1.8 s), so it is done when "So that candidate breaks" starts. |
| **R10** | 08_5 | `product_rule.rs:394-449` | The ring never changes after 125.0 s. No half turn is drawn. | `:428-457`. A gold dot at +1 comes with the gold row. "Halving the lost lap" shows `½(2πi) = πi` alone. On "leaves a half turn" a purple arc with a head sweeps over the top from +1 to −1 in 1.6 s, then `half turn` fades in. On "a factor of minus one" the line becomes the old `½(2πi) = πi, e^{πi} = −1`, with the left glyphs still. Helpers are at `:488-543`. |
| **C2** | 05_3 | `complex_turns.rs:471-473`, `:515`, `:1001` | The input dot waits on the labelled 3π tick while the output winds on. | `:491-500`, `:529`, `:755-764`. Past the strip's top the dot runs up the goes-on arrow, fading, and is gone at its tip (`strip_dot`). The arrow's length is the constant `GOES_ON_TIP` that `PAST_STRIP` comes from. The comments that said "waits" are rewritten. |
| **B8** | 07_5 | `branch_cut.rs:227-241`, `:496` | The logarithmic spiral's two innermost turns merge into a red knot at zero. The least gap between turns is 0.063 for a 0.04 stroke. | `:224-241`, `:496`. An even spiral: three turns from zero itself to the panel's edge, ending at angle 3π. Its turns are 0.900 apart and it never crosses itself. Stroke, colour and end radius are unchanged. |

The script, its explanation and every other row stay. Each staged file says the same cues in the same order and keeps every old `until_sentence` hook. Check: `run_checks.txt`, "what the patch changes".

| Item | SHA-256 |
|---|---|
| `logarithm05-08-teaching.patch` (904 lines, four files, `a/` and `b/` crate paths, applies with `git apply` from the repo root to the candidate's bytes) | `1aafaaa84671c95e5a49340882efbe32aaea2ab7b74ba4ab149a1636dd90db34` |
| `complex_turns.rs`, baseline → staged | `2cc2ba7723e2733ef06c633650b135507d5d1726161fa2a277b90e00277b5ead` → `93f663d372ab79a77139287dc38b94c148909f7ea4a6ba549884c91f342eb19a` |
| `lifting_loops.rs` | `ca34aef0523ceadce22edccd7d384ac29c514653207fdf052fd5cb31cd3fa01e` → `01885b9f558c36bc8af425bfdf427e4b30d74d575819121e52988b5d06db7346` |
| `branch_cut.rs` | `193c4733de9f9081f5671700a6710dd76e989b41ab268824bbb0756313fbdbbd` → `c1159c3727a1f1d83351d2c837027c563455df6ebc2de32a2db5973d56cbc401` |
| `product_rule.rs` | `e9019c828710207b806175a040aab936ee558b102d3cb55328078d42d897f7e0` → `f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db` |
| Not touched: `draw.rs`, `script.tsv`, `facts.rs`, `oracle.py`, the kit | `draw.rs` `2a4e8450…`, `script.tsv` `b89eab60…` |

## Phrase hooks: source-bound, with an honest fallback

Each film that gains a phrase hook gets a local `until_phrase(&kit, stage, cue, PHRASE)`. It sits after the film's `impl`, so no held line above it moves. A hook names a `const` phrase from `script.tsv`. That phrase occurs once in its cue and begins a word, and the hooks run in spoken order in the source.

- **With a word mark** at the phrase's byte from `narrator.phrase_marks(cue)`, the hook waits for that mark. Root's aligner fills marks later. No mark is invented here.
- **Without one**, it waits for the phrase's character share of its measured sentence (`phrase_seconds`). That is a provisional estimate, not a heard boundary. Every `measured_seconds` in `phrase-anchors-proposed.json` is `null`.
- **Without a narrator**, a hook does nothing, as `until_sentence` does.

No take time is written in the source. `recognition_evidence.py` checks that none of the recognized times appears in it.

| Cue | Phrase | Byte | Sentence | Word | Hook |
|---|---|---|---|---|---|
| 05_20 | the logarithm of two | 180–200 | 4 | 37 | `complex_turns.rs:369` |
| 05_20 | a half pi | 214–223 | 4 | 45 | `:371` |
| 05_20 | the length is two | 225–242 | 4 | 48 | `:373` |
| 05_20 | land on two i | 276–289 | 4 | 59 | `:375` |
| 06_6 | passes minus one | 267–283 | 3 | 48 | `lifting_loops.rs:587` |
| 06_6 | leaps from pi to minus pi | 338–363 | 3 | 60 | `:593` |
| 08_5 | leaves a half turn | 260–278 | 4 | 51 | `product_rule.rs:438` |
| 08_5 | a factor of minus one | 299–320 | 4 | 60 | `:456` |

### What root's recognition run says about 05_20

Evidence: `five-film-release-20261027/logarithm-turns-recognition01/command.log` (`8d20c0079989fd2d…`, 26 records, all ok) and `turns-recognition01/takes.json` (`28983ebf376f2b26…`). Both retained 05_20 takes are bound, on the script's exact words. These are one model's provisional word rows. No take is selected, nobody heard the takes for this, and the rows accept nothing. They support the design and are the eventual word-mark evidence for C1. Root's aligner, not this folder, writes marks.

| Take | Sentence 4 starts | "the logarithm of two" | "a half pi" | "the length is two" | "land" | "two i" (written `2i`) | Voice ends |
|---|---|---|---|---|---|---|---|
| download01 | 13.50 | 15.58 | 18.02 | 19.50 | 23.70 | 24.20 | 25.27 |
| download02 | 13.44 | 15.70 | 18.22 | 19.40 | 23.56 | 24.08 | 25.11 |

This evidence changed the design. My first staging hooked the answer on "so we land on two i". On these rows that would settle `= 2i` by about 23.4 s, before "land" and 0.8 s before "two i". The hook is now "land on two i": `= 2i` begins on "land" and has settled 0.20 and 0.22 s before the row for "two i". The arrow and label settle during "two i". For the same reason the first two stages now hook on the words that name what they show: "the logarithm of two" brings `ln 2`, and "a half pi" brings `π/2`. They used to hook on "take x to be" and "and y to be". A hook on "two i" itself would fail the other way: with the fallback, its last picture would outlast the voice and move 05_3.

The fallback lands +0.04 to +0.72 s after the rows, never before them. Worst is "a half pi" in download01. Even then each picture has settled before the next hook arrives, and the last one settles 0.39 to 0.40 s before the voice ends. With marks the margin is 0.75 to 0.77 s.

The run has no 06_6, 07_5 or 08_5 records. Those hooks rest on the character share alone until a run covers them.

## Bindings, holds and the binder

**Holds** (`hold-table.json`, the binder's interval formula; it reproduces all seven stage06 values first):

| Hold | File | Say–marker lines | Next say | Interval text |
|---|---|---|---|---|
| 05_1 | `complex_turns.rs` | 185–245, unchanged | 249 | unchanged `5256ae12dfc74fb4…` |
| **05_5** | `complex_turns.rs` | **611–623 → 624–636** (+13) | **626 → 639** | unchanged `93a300682e33fcfe…` |
| 06_2 | `lifting_loops.rs` | 335–374 | 377 | unchanged |
| 06_4 | `lifting_loops.rs` | 417–475 | 478 | unchanged |
| 07_3 | `branch_cut.rs` | 366–419 | 422 | unchanged |
| 08_1 | `product_rule.rs` | 108–221 | 224 | unchanged |
| 08_4 | `product_rule.rs` | 317–346 | 349 | unchanged |

Finishes: `complex_turns.rs` 736 → 749, `lifting_loops.rs` 692 → 699, `branch_cut.rs` 607 unchanged, `product_rule.rs` 449 → 468. Every file hash changes, so each hold row's `source_sha256` must be rebound to the staged hash.

**These changes need a binder update after binder02.** BronzeJaguar's binder02 is sealed (`logarithm-stage07-binder02/SHA256SUMS` `0f007b3d8c63db7a…`). It pins these four files at their stage07 bytes, so integrating this patch makes it exit 3 (Mail 3369). The later update, built on binder02, needs the counts and row sets below. I edited neither binder.

Per file, in the film's code (`hold-table.json`, `per_file`):

| File | `say` | `until_sentence` | `until_phrase` calls | `phrase_marks` uses | `wait_frames` calls | Lines |
|---|---|---|---|---|---|---|
| `complex_turns.rs` | 8, unchanged | 25, unchanged | 0 → 4 | 0 → 1 | 0 → 1 | 1041 → 1256 |
| `lifting_loops.rs` | 8 | 26 | 0 → 2 | 0 → 1 | 0 → 1 | 913 → 1055 |
| `branch_cut.rs` | 6 | 24 | 0 | 0 | 0 | 905 → 924 |
| `product_rule.rs` | 5 | 21 | 0 → 2 | 0 → 1 | 0 → 1 | 534 → 829 |

**binder01 is stale for today's candidate.** It pins `folded_decade.rs` `c4f023ce…`, and root's partial06 made that file `acfd77f4…`. My dry run below still targets binder01, so it uses root's preserved `before-oliver-digits-mark01-folded_decade.rs`, as binder01's own handoff did. This repair touches none of that file.

The update needs **five row sets** (`binder-rows.json`):

1. `PHRASE_MARK_USES`: `complex_turns.rs`, `lifting_loops.rs` and `product_rule.rs`, 1 each.
2. `FALLBACK`: the same three, each "the phrase's character share of its measured sentence … Provisional."
3. `SITES`: `("complex_turns.rs", 845)`, `("lifting_loops.rs", 769)` and `("product_rule.rs", 606)`, class `word_sync_helper`. Each is the one `wait_frames` inside `until_phrase`.
4. `RECEIPTS`: one integration receipt naming the four chapter files, written by root when it integrates. I called it `integration-teaching-repair01.json`; any name works if it holds the four staged hashes. The dry run writes a stand-in only inside its scratch mirror.
5. Added anchors: the eight phrase anchors above, each with its cue, phrase and byte, and `measured_seconds` null. binder01 stops on any new anchor. Sealed binder02 keeps `ADDED_ANCHORS` apart from White's anchor ("a later update adds its own"), so these eight can go there without tripping White's guard (Mail 3369).

`binder_dry_run.py` runs binder01's own `build()` in four scratch mirrors. It edits no binder file or output; the rows are set in memory:

| Run | Result |
|---|---|
| baseline, binder01's pins untouched | BUILT |
| staged, binder as is | STOP: `('complex_turns.rs', 845): a wait this audit has not classified` |
| staged, with the five row sets | **BUILT** |
| plant: two 05_20 hooks swapped | STOP: `05_20: the source reaches its phrases out of spoken order` |

In the third run binder01 also passes its own stops on the hold intervals, the sentence-share hooks and the helper and consumer rows. `run_checks.py` separately checks that every old `until_sentence` stays.

I did not run binder02. BronzeJaguar reports a scratch run of it on these four files at these same hashes, with a stand-in receipt, recomputed pins and its comparison against binder01 left out. It builds, with 33 anchors in 18 cues and 14 holds whose file hash changed. 05_5's say and marker lines move, and no new interval changes. That is their scratch run, not a binding, and I have not checked it (Mail 3369).

## Timing and length

Every new wait sits inside its own cue's voice. In the mirror on the preview lengths (05_20 24.631 s, 06_6 28.477 s, 08_5 31.554 s), scaled from 0.85 to 1.15:

- No picture reaches its hook more than +0.027 s late.
- The last picture always settles before the cue's voice ends. The least room is 0.211 s, for 05_20 at ×0.85.

So no later `say` should move, and the chapter lengths should not change. Only a native run can confirm that, along with whether the music stems still fit.

The mirror's positive control reproduces the old leads. C1's answer led its words by 10.45 s. L1's drop led "passes minus one" by 2.01 s and its inset led "leaps" by 6.16 s. R10's sign led its words by 4.83 s.

## Decisions for you

1. **C1 has an extra stage, `e^{ln 2 + iy}`.** The review asked for x and y "on their words". Showing `iy` first lets "a half pi" visibly replace it. If you prefer two stages, drop `example_x` and its hook. Then `e^{ln 2 + iπ/2}` comes on "the logarithm of two".
2. **L1's last walk is 1.2 s, down from 1.8 s.** At 1.8 s, a fast take would still be walking when sentence 4 starts. At ×0.85 that was up to 0.63 s late.
3. **R10 adds an on-screen word, `half turn`** (20 pt, `lap_c`). It sits outside the ring at its upper right. Its box is estimated from the width table at x −2.95..−2.18, y 1.38..1.52. Remove it if the arc alone reads. The arc runs over the top, the way `e^{πi}` goes. Its stroke stops 0.045 clear of each dot's edge.
4. **B8's zero ring keeps its red.** The review suggested recolouring it. The even spiral leaves only 0.14 of a turn inside that ring, down from 1.30, so the knot is gone. I left the colour for you to judge on frames.
5. **R11 is not touched.** The green row's `e^{iπ}` still differs from the purple row's `e^{πi}`. The purple line's string must stay the old one so its final picture matches.
6. **New TeX and text strings** need your glyph probe: `e^{\ln 2 + iy}`, `e^{\ln 2 + i\pi/2}`, `\tfrac{1}{2}(2\pi i) = \pi i` and `half turn`. Each uses glyphs the film already sets, but only the probe proves the faces.

## Focused tests (Rust, not run)

- `complex_turns.rs`:
  - `the_worked_answer_comes_last_and_each_piece_waits_for_its_own_words`
  - `the_stages_of_the_worked_line_stand_on_its_corner_and_hold_no_answer`
  - `the_input_dot_leaves_the_labelled_ticks_along_the_arrow_and_is_gone_at_its_tip`
  - `a_phrase_lands_on_its_mark_or_on_its_share_of_the_measured_sentence`
- `lifting_loops.rs`:
  - `the_gap_drops_on_its_words_and_the_candidates_jump_waits_for_its_own`
  - the phrase test
- `branch_cut.rs`:
  - `the_spirals_turns_stand_apart_and_one_strand_leaves_the_ring_round_zero`
  - the old start-radius test, tightened to start at zero
- `product_rule.rs`:
  - `the_half_turn_and_its_sign_wait_for_their_words_and_the_whole_line_stays`
  - `the_halved_lap_stands_on_the_whole_lines_corner_and_holds_no_sign`
  - `the_half_turn_runs_between_the_two_answers_and_touches_nothing_on_the_way`
  - `the_half_turns_name_clears_the_ring_the_axis_the_definition_and_the_column`
  - the phrase test

`source_conditions.py` runs their text conditions in Python. Each row passes on the staged bytes and fails on the baseline, and four plants are caught. `mirrors.py` runs their geometry. A 12-turn spiral, the old clamp and an arc with no gap each fail it.

## Native picture checks

`source-markers.json` holds all fourteen anchors in full. Anchor on the cue, sentence and phrase. Old times are stage06's scratch voice, not Oliver. Old frames are labels in `logarithm-stage06-frames01/<chapter>`, counted from 0, and show the state being replaced.

| Anchor | When | Old frames | Expect | Fails if |
|---|---|---|---|---|
| C1-x | 05_20 "the logarithm of two" | f134, f136 | only `e^{ln 2 + iy}`; lengths, angles, ×i gone | `=`, arrow or `2i` label shows |
| C1-y | "a half pi" | f136 | `iy` → `iπ/2`, left glyphs still | glyphs jump, or answer shows |
| C1-length | "the length is two" | f150 | words card in; no answer | card early, or arrow with it |
| C1-land | "land on two i" to cue end | f158, f168 | `= 2i`, then arrow and label during "two i"; ends as f168 | any piece before "land", or unsettled at cue end |
| C1-out | 05_3 start | f168 | all leave once | a stage flashes back |
| C2 | 05_3, past the strip's top | f215–f218, f227 | dot runs up the arrow, fades, gone at its tip | dot rests on 3π while laps read 2 or 3 |
| L1-wait | 06_6 sentence 3 to "passes minus one" | f267–f269 | walk at 0.45, gauge 0, no ring, no inset | dot passes −1 early |
| L1-cross | "passes minus one" | f271, f273 | ring, then the drop about 1 s in | drop before the phrase or about 2 s after |
| L1-inset | "leaps from pi to minus pi" | f271, f283 | inset on these words; walk done by sentence 4 | inset early; walk still moving |
| B8 | 07_5 "Or cut along a spiral" | f262, f265 | separate turns, one strand from zero (compare review `zooms/07-f0264-spiral.png`) | turns merge, or don't reach zero |
| R10-dot | 08_5 sentence 3 | f276, f287 | gold dot at +1 with the gold row | absent or off +1 |
| R10-halving | sentence 4 | f298 | `½(2πi) = πi` alone | sign shows |
| R10-arc | "leaves a half turn" | f298, f302 | arc over the top +1 → −1, then `half turn` | touches a dot, arc or text; runs under |
| R10-sign | "a factor of minus one" | f310, f325 | `, e^{πi} = −1` joins; ends as f325 | early, or glyphs jump |

## Evidence

| Check | Result | What it cannot show |
|---|---|---|
| `checks/run_checks.py <fresh-scratch>` (output `checks/outputs/run_checks.txt`) | all PASS | everything below |
| Patch round trip, `git apply` and `-R`, in scratch | PASS | that it compiles |
| `rustfmt --check`, repo settings | 4/4 clean | Clippy |
| `docs/check_cues.py` on a scratch copy of the candidate with these files | PASS | timing |
| `mirrors.py`: spiral, dot, arc, label box, timing | PASS; plants fail | real glyph extents; real takes beyond 05_20 |
| `source_conditions.py` | PASS; baselines and plants fail | the Rust tests themselves |
| `bindings.py` | 7 holds, no interval text changed; per-file counts | the update after binder02 |
| `recognition_evidence.py` on root's two 05_20 row sets | PASS, including the two hooks not taken | hearing; other cues; take choice |
| `binder_dry_run.py` on binder01 | as tabled | binder02; root's real receipt |

## Overlap and gaps

- **Not independent.** I wrote review05-08's C1, L1, R10, C2 and B8, and this repair of them. Both share one model and its memory.
- **Still root's:**
  - Cargo build, the Rust tests, Clippy and the glyph probe.
  - The render, and a picture review of these anchors.
  - The binder update after binder02, the integration receipt and rebinding.
  - Oliver word marks and fine sync.
  - Listening, and the final 4K.
- No synthesis request is needed, because no cue's words change. No take is selected.
