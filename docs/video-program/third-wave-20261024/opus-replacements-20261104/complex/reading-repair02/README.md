# reading-repair02: clean reading time for the complex-analysis cards

Bead `87do.39`, assigned by Mail 1303. This is a proposed pacing change. **Nothing here has been
compiled, run, rendered or listened to, and no reader has been timed.** The script does not change
(`script.tsv` `2968b0fd7654`). No Oliver take is touched and no synthesis is requested.

`reading-repair01` stays frozen. This patch **replaces it and does not stack on it**: both are
written against the same canonical bytes. If 01 has been applied, reverse it first.

## What changed from 01

01 kept every window short enough to stay under the 4 s cue-gap warning. Root ruled that the warning
is not an editorial limit, so 02 chooses each window from what the card says, how long it is, and
what is competing with it.

Totals are now +57.0 s, up from +16.5 s: chapter 04 +7.5, 05 +3.5, 07 +4.5, 08 +5.0, 09 +3.0,
10 +8.5, 11 +14.5, 13 +5.0, 14 +5.5.

Three kinds of change, all native scene code:

- **Longer windows** at all ten sites.
- **Three new windows on a chapter's last cue**: 08_5, 11_8 and 13_11 had none. Each window comes
  before the existing music breath, and the closing fade still clears the card.
- **Moved reveals**: three cards now wait for the voice to finish because the voice is talking about
  something else (04_4, 08_5, 11_6). 13_11 is split in two steps, described below.

## About chars/15

**chars/15 is a rule of thumb, not a human measurement.** Nobody timed a reader. It knows nothing of
Oliver's pace, and mathematics usually reads slower than prose. It fits code poorly. It is used here
only as a floor for what the voice does not carry, never as the reason a window has its length.

## The windows

*Clean* means the card is fully in, nobody is speaking and nothing moves, until it clears. In source
seconds it is `BREATH (0.6) − fade (0.4, if the card arrives after the voice) + window + closing
breath`. It assumes a narrated render: with no narrator `kit.hold` does nothing, so clean is lower.

| Cue → next | Window | Card arrives | Clean | Unspoken at 15 c/s | Whole card at 15 c/s |
|---|---|---|---|---|---|
| 04_4 → 04_5 | 7.5 | after voice | 7.70 | 7.40 | 7.40 |
| 05_8 → 05_9 | 3.5 | under voice | 4.10 | 3.60 | 3.60 |
| 07_2 → 07_3 | 4.5 | under voice | 5.10 | 4.67 | 6.93 |
| 08_5 → 09_1 | 5.0 | after voice | 7.20 | 6.47 | 6.47 |
| 09_5 → 09_6 | 3.0 | under voice | 3.60 | 2.53 | 7.13 |
| 10_5 → 10_6 | 8.5 | under voice | 9.10 | 8.67 | 8.67 |
| 11_6 → 11_7 | 7.0 | after voice | 7.20 | 7.13 | 7.13 |
| 11_8 → 12_1 | 7.5 | under voice (panel) | 10.10 | 6.60 | 15.60 |
| 13_11 → 14_1 | 5.0 | after voice (clause) | 7.70 | 5.27 | 12.80 |
| 14_3 → 14_4 | 5.5 | under voice | 6.10 | 5.93 | 5.93 |

Why each one:

- **04_4** The voice asks the eye to count colours around the dot, so a sentence about order would
  compete with that. The card now waits, and all of it is read clean.
- **05_8** Keeps its reveal at "A picture cannot settle it", which is related speech and only a few
  words of overlap. All of it is read clean.
- **07_2** The voice carries the conclusion. Only the condition on the first m − 1 derivatives is
  card-only, and it needs the clean time.
- **08_5** Goursat is named nowhere in the voice. From "Neighbouring squares" on, voice and picture
  are both on the shared edges, so the card moves to the voice's end.
- **09_5** The voice paraphrases the card as it appears. Only "an open set containing the closed
  disk" needs clean time.
- **10_5** The whole theorem, three hypotheses and a conclusion, while the voice's last sentence is
  about sine of one over z. All of it is read clean.
- **11_6** A note about how the output is drawn, which has nothing to do with the minus three z the
  voice is on. It moves to the voice's end.
- **11_8** The panel shares the screen with the running check for the whole cue. The check now stands
  still for 10.1 s so the code can be read against what the voice just described. Pieces the voice
  never says: `if L * step >= |a|: refuse`, `# L bounds |f'| on the loop`, `angle(b / a)`,
  `# in (−π, π]`, `# zeros − poles inside`.
- **13_11** The voice carries the first clauses, so `HONEST_SAID` shows those while it speaks. The
  warning nobody says arrives alone when the voice stops ("for slowly decaying f that symmetric limit
  can exist when the integral does not"), and the first line should not move.
- **14_3** Two uniqueness claims right before a question. All of it is read clean.

The windows live in one table, `chapters::READING_WINDOWS`. To change one after listening, change its
row, the matching hold cell in the storyboard, and rebind. The tests will not stop a longer window
until clean passes twice the rule of thumb for the whole text, and that bound is only there to catch
a picture held for nothing. Raise it if listening asks for more.

**Thin margins.** Clean is close to the floor at 11_6 (7.20 against 7.13), 14_3 (6.10 against 5.93),
04_4 (7.70 against 7.40), 10_5 (9.10 against 8.67) and 07_2 (5.10 against 4.67). These are the first
places to lengthen if a reader runs out of time.

## The replaced test

`reading_windows_stay_short_and_never_delay_an_answer` was 01's test. It asserted
`seconds + BREATH < 4.0`, which is the arbitrary 3.3 s limit of Mail 1303 and exists only to stay under
the warning. It is replaced by `reading_windows_leave_time_for_what_the_voice_leaves_out`, which
requires:

- every row is a real cue, in script order, and never sits between a question and its answer;
- every piece listed as unspoken is actually on screen;
- a card arriving after the voice lists all of itself as unspoken (13_11 is the one exception, since
  its earlier words were on screen while the voice said them);
- clean ≥ the rule of thumb for the unspoken text;
- clean ≤ twice the rule of thumb for the whole text.

Neither `HOLD_MERGE` nor `--max-gap` limits anything here.

Every test in canonical is byte-identical. New tests: `the_honest_note_only_adds_the_clause_nobody_says`
and `a_growing_card_covers_before_it_clears`. Extended for chapter-end windows, keeping their earlier
assertions apart from one per-file count that became per-cue: `each_card_outlasts_its_reading_window`
and `every_reading_window_is_in_the_storyboard`.

## Helpers added to `draw.rs`

- `reading_window`: waits for the voice's end, then the kit's breath, then the table's seconds. It
  draws and clears nothing.
- `precisely_lifted`: a lower third raised so that its first line sits where the taller card's first
  line will sit.
- `precisely_over`: the whole card fades in over the part (0.3 s), then the part goes (0.1 s). The
  order matters because the band is 0.94 opaque, so the part leaves behind the whole.

## Storyboard

Ten hold cells and their picture notes now state the card's text and when it arrives.

`z₀`/`a` notation: 04_4's note said `c(z − z₀)ᵐ`, but its card says `a`, so the note now matches the
card. 07_5's "At z₀ = 1" had no such label in the scene and the voice says "the point one", so it now
reads "At the point 1". 06_1, 06_3 and 12_3 keep `z₀` because their scenes draw it. The film's own
mix of `a` and `z₀` is listed under teaching defects below.

## Applying it

From the repository root, against the canonical hashes in `baseline.json`:

```
patch -p1 -i <this folder>/reading-repair02.patch
```

**With `clippy-repair05`** (WhiteSwan, handed off in Mail 1385). I tested its patch
`e6d4405bf25b6979`, the hash in that handoff, on scratch copies of canonical. Repeat this if either
patch's hash differs. Its `HANDOFF.md` rehearses the same merge with `git merge-file` and reports the
same result. With both patches, its expected test count of 97 rises by the 6 tests this patch adds.

- Either order gives identical bytes, with no rejects.
- Four files are touched by both: `counting.rs`, `draw.rs`, `one_point.rs`, `real_integral.rs`.
- One hunk needs fuzz 1 either way: `HONEST_SAID` sits right above the `#[cfg(test)]` on
  `antiderivative`. `patch -p1` accepts it. After the Clippy patch, `git apply` refuses it and
  `git apply -C2` accepts it.
- On the merged tree: rustfmt exit 0, the binder and validator pass, and the Python mirrors of the
  reading tests pass. It has not been compiled, so it has not been shown to clear strict Clippy.

**Rebind after any merge.** Map rows pin each source file's `sha256` and `line`, so a merge stales
them. Bind into a new empty folder, using the merged crate:

```
python3 -I -B bind_oliver_plan.py REPO MERGED_CRATE NEW_FOLDER
python3 -I -B validate_oliver_plan.py REPO MERGED_CRATE NEW_FOLDER EMPTY_SCRATCH
```

On the test merge above, only `sha256` and `line` moved in the map's rows, in the same four files.
`narration_units.json` changed only in its reference to the map's hash.

## The map

`docs/narration-direction-map-oliver09.json`: 23 overrides, made up of 13 prediction and 10 reading
holds, all authored and none measured. It is a proposal, not acceptance.

- 08_5, 11_8 and 13_11 have a `next_cue` in the next chapter. `validate_map` takes adjacency from the
  whole cue list, so these pass. The validator allows this only on a chapter's last cue, and refuses
  it for a prediction.
- `review_directions` will need one `native_visual_hold` disposition per selected override, so 23.
- The rows omit `reason`, `evidence`, `policy` and `range_s` on purpose: with no recording there is
  nothing to take them from. `narration_director.validate_map` will ask for them once takes exist.

## For root: score and cue warnings

Nothing shared is changed, and I recommend weakening no gate.

- **Bed.** With `steady_bed: true`, this crate's setting, `HOLD_MERGE` and `hold_duck` do nothing to
  the bed, so crossing 4 s does not change its level. This depends on that setting staying true. None
  of the ten sites' chapter files carries a sonification, the one thing that still follows the duck
  curve.
- **Music in the windows.** Every gap of at least 2.4 s (`ROLL_GAP`) gets a slow chord roll and then
  the varied leitmotif (`MIN_PHRASE_GAP` 1.6 s), capped by the steady-bed ceiling. So the score now
  writes music under theorem reading. Nobody has heard this; whether it helps or distracts is for
  root and the owner to decide.
- **Authored windows.** A plain `pause` is not logged as a breath window, so `settle`/`recede` do not
  apply to the mid-chapter windows.
- **Cue sheets.** Nine expected warnings at the defaults, none of which should be silenced by
  changing `--max-gap` or `--max-tail`:
  - gaps: 04_4 8.1 s, 05_8 4.1 s, 07_2 5.1 s, 10_5 9.1 s, 11_6 7.6 s, 14_3 6.1 s;
  - tails: 08_5 8.2 s, 11_8 10.6 s, 13_11 8.7 s.

  09_5's 3.6 s gap is simply under 4 s, not exempt. Any other warning at these cues is a real finding.
- **`narration_director`** reports `protected_visual_hold` for a mapped row. Without a row, a long
  cross-cue gap becomes `scene_wait_review`.
- **`tools/storyboard.py`** estimates ignore `read N s`, so they undercount by 57.0 s. Its showcase
  row "morphs" now also lists 09_5, because its `morph` pattern matches "holomorphic" in the quoted
  card. That row was already ticked.
- **Length.** Chapter 11 grows most (+14.5 s). Nothing has been checked against the chapter or
  raw-frame limits, because there are no takes.

## Checks

```
python3 -I -B check_reading_repair.py REPO EMPTY_SCRATCH
```

`check-run01.txt` is the passing run. It covers:

- base, dependency and frozen-01 hashes;
- the patch regenerating byte for byte, applying and reversing;
- scope by reversal, chapter by chapter;
- rustfmt on the whole module tree;
- Python mirrors of the new and changed tests, with 31 planted defects each failing for its own reason;
- the binder and validator, with 55 binder and 21 validator planted defects, plus 3 controls;
- the score facts above, read from source.

**A Python mirror is not the Rust test.** It reads the same text and asserts the same things, but
`cargo test` is the real check and root runs it.

## What would accept this

On a narrated draft with real Oliver takes:

1. At each of the ten cues, frames at the start and end of the clean time show the whole card, opaque
   and still, with no voice.
2. 13_11's cross-fade: the first line holds still, with no flash of two cards or of a gap.
3. Someone actually reads each card in its window, and says where they ran out of time or waited too
   long.
4. Listening: do the long silent holds drag, and does the score under them help?

## Remaining gaps

- Not compiled, not run, not rendered. Clippy has never passed on this crate; build04 stopped there
  on findings unrelated to this patch. From reading the source, my new code has none of the lint kinds
  in that log, but only Clippy can say. The oracle has not run in this lane.
- No takes, listening or frames. Every number above is in source seconds.
- `precisely_lifted`'s lift and the cross-fade have never been seen on a frame. While the voice runs,
  the shorter 13_11 card sits above the frame foot with picture visible below it, inside the area the
  whole card then covers. First-line stability also assumes `lower_third` does not narrow the wrap
  for width; that was computed in Python at the 92-character wrap.
- A scratch render with no narrator has no breath, and 04_4, 08_5 and 11_6 wait there on estimated
  voice. Clean time cannot be judged from such a render.
- Earlier findings still open: `--check` reports no prediction pause in `05_gallery`, `08_loop_sums`
  and `12_residues`, and `script.tsv` is "stale" in its header comment only (the cue rows match).
- Build04's log is what I read for the Clippy stop. Any further canonical change rebases this patch:
  the check fails on a moved base hash.

## Strongest unresolved teaching defects

1. **Long silences are authored but unheard.** Up to 9.1 s mid-chapter (10_5) and 10.6 s after the last
   line (11_8). They may drag, and only listening can tell.
2. **Under-voice cards still overlap speech**: 05_8, 07_2, 09_5, 10_5, 14_3. The overlap is related
   speech, but a viewer starting the card under the voice is still splitting attention.
3. **11_8's panel still competes with the demonstration while the voice runs.** Highlighting code line
   by line would need interleaving inside the timed sample loop of `coarse_loop`, so I did not add it.
   The clean window afterwards is the remedy, not a cure.
4. **Goursat arrives far from "almost"**, the word it explains, and nothing highlights the `each` row.
5. **13_11's warning has no example and no voice**, though it is the honest caveat of the chapter.
6. **Notation mixes `a` (04_4 and the cards) with `z₀` (06, 08, 12).** These are correct picture by
   picture, but a viewer sees both. Fixing it is a scene and card change, so I have not done it here.
7. **Twelve `names_now` cards have no window**: 04_7, 06_6, 07_2, 07_6, 09_2, 10_1, 10_6, 11_2, 11_7,
   12_1, 12_2, 14_1. They are search terms rather than theorems, but they are still read under speech.

## Files

- `reading-repair02.patch`, `baseline.json`: the patch and what it was made against.
- `from-reading-repair01.diff`: for review only, what changed since 01. Do not apply it.
- `explainer_corpus/complex_analysis_visual/...`: the candidate files, mirroring canonical paths,
  plus `narration_units.json` and `docs/narration-direction-map-oliver09.json`.
- `source-audit.json`: per-cue facts from the binder, including clean time and expected warnings.
- `bind_oliver_plan.py`, `validate_oliver_plan.py`, `planted_defects.py`, `check_reading_repair.py`,
  `check-run01.txt`: tools and the passing run.
