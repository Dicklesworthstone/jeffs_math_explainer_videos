# reading-repair01 for complex_analysis_visual

Explicit reading windows after the seven `precisely:` cards that were competing with the voice.
Each card now stays still on screen for an authored time after the voice ends, before the next
cue can clear it.

**This is a proposed teaching and pacing change. It is not measured reading or listening
acceptance.** Nothing here has been compiled or rendered, no take exists, and nobody has listened
to anything. The seconds are authored from each card's text. They are not derived from any
recording, and the map carries no `range_s`.

The script, the takes and all earlier outputs are untouched: script.tsv stays `2968b0fd7654`, and
the narrator stays ElevenLabs Oliver.

## Files

| File | What it is |
|---|---|
| `reading-repair01.patch` | Unified patch of 11 files. From the repo root: `patch -p1 -i .../reading-repair01.patch` |
| `explainer_corpus/complex_analysis_visual/...` | The 11 patched files whole, plus the bound `narration_units.json` and `docs/narration-direction-map-oliver08.json` |
| `source-audit.json` | The binder's audit, with a `reading_windows` section and the unregistered files it ignored |
| `bind_oliver_plan.py`, `validate_oliver_plan.py`, `planted_defects.py` | The oliver-plan01 tools extended for reading windows. The originals are untouched |
| `baseline.json` | sha256 of the base files, the files read, test-repair01's patch and the script |
| `check_reading_repair.py` | The checks. `python3 -I -B check_reading_repair.py REPO EMPTY_SCRATCH` |
| `check-run01.txt` | Their output |

## Patch order

The base is current canonical, which already holds compile-repair01 and test-repair01. The patch
applies directly to it and shares no file with either.

1. Apply `reading-repair01.patch` after the build in progress finishes.
2. Lay over `narration_units.json`, the oliver08 map and `source-audit.json`.
3. Run `bind_oliver_plan.py REPO CRATE <this folder> --check`, then `validate_oliver_plan.py`.

**Any later source repair stales the map's hashes.** Rebind with this binder into a new folder.
Existing files with other bytes are refused, never overwritten.

## The windows

The change is one helper, `draw::reading_window` (`draw.rs:884`), one table,
`chapters::READING_WINDOWS` (`mod.rs:53`), and one line at each site.

| Cue → next | Site | Card | Chars | Window | Why this long |
|---|---|---|---|---|---|
| 04_4 → 04_5 | `reading_color.rs:124` | `NEAR_ZERO` | 111 | 2.5 s | A relation in words with three letters; the voice is counting turns, not reading it |
| 05_8 → 05_9 | `gallery.rs:755` | `FOUR` | 54 | 2.0 s | A note on how the plot was made; only a five-word sentence overlaps it |
| 07_2 → 07_3 | `one_point.rs:371` | `CRITICAL` | 104 | 3.0 s | A condition on m − 1 derivatives and its result, arriving as a new heading does |
| 09_5 → 09_6 | `boundary.rs:291` | `NEEDS` | 107 | 2.0 s | One hypothesis in exact terms; the voice says nearly the same thing |
| 10_5 → 10_6 | `rigidity.rs:415` | `EXACT` | 130 | 3.0 s | A whole theorem, hypotheses and conclusion, beside the condition card; capped by the 4 s limit below |
| 11_6 → 11_7 | `counting.rs:316` | `SQUASHED` | 107 | 1.5 s | A note on how the output is drawn; no argument to follow, long voice overlap |
| 14_3 → 14_4 | `continuing.rs:388` | `PATH_EXACT` | 89 | 2.5 s | Two uniqueness claims told apart, at the end of the longest cue, before a question |

The total is +16.5 s: chapter 04 +2.5, 05 +2.0, 07 +3.0, 09 +2.0, 10 +3.0, 11 +1.5, 14 +2.5.

**Why a plain `pause`, not `music_breath`.** A music breath tells the score the music should be
heard, is refused mid-chapter by the binder, and is tied to the storyboard's `hold`. These
windows are for reading.

**Why at most 3.3 s.** With the kit's 0.6 s breath the gap stays under 4 s, where the score still
treats it as a hold (`HOLD_MERGE`, `score.rs:1138`) and the cue sheets do not warn of dead air
(`--max-gap`). A longer window would need a decision on both. The crate's `steady_bed: true` is
why the bed itself does not react; the limit depends on that setting.

## Helper ordering, hold before clear

```rust
pub fn reading_window(kit: &Kit, stage: &mut Stage<'_>, line: &Spoken) -> fmn::Result<()> {
    let seconds = /* the cue's row in READING_WINDOWS, or fail */;
    wait_until(stage, line.end())?;  // the voice ends (this is what holds a scratch render)
    kit.hold(stage)?;                // the kit's breath; does nothing without a narrator
    pause(stage, seconds)            // the authored window
}
```

It uses only the pinned native calls the crate already uses. It draws and clears nothing, and the
next cue's `fade_out_all` clears the card. The lookup is the same form `draw::predict` uses, which
rustc has accepted.

I read all seven sites. Each clears its card only after the next `say`, and no fade or removal
falls between the card and the window. Windows sit before the question at 07_3 and 14_4, and after
the answer at 09_5, 10_5 and 11_6. None sits between a prediction and its answer.

**New Rust tests** (3 in `chapters`, 1 in `main`; none has run):

- `each_card_outlasts_its_reading_window`: card, window and next cue are in order, nothing lies
  between the window and the next `say`, nothing clears the card early, and the next cue clears it.
- `reading_windows_stay_short_and_never_delay_an_answer`: the bounds above, and no window on a
  prediction.
- `the_reading_window_waits_in_order_and_clears_nothing`: the helper's order.
- `every_reading_window_is_in_the_storyboard`: table and storyboard agree, both ways.

These read the source, because no render can run here. They cannot show a card on a frame.

## Direction map oliver08

It has 20 overrides and 20 holds in script order: 13 predictions and 7 reading. A reading row
exists only where the source adds a wait, and both tools refuse any other. No row has `range_s` or
`word_index`.

- **Dispositions.** `review_directions` will need 20 `native_visual_hold` dispositions.
- **Director fields.** As in oliver07, the rows omit `policy`, `range_s` and `evidence`, which
  `narration_director.validate_map` wants, because there is no recording to take them from.
- **The binder reads only modules registered in `mod.rs`.** The oliver-plan01 binder globs every
  file, and run on canonical it refuses with 57 problems, because canonical still holds seven old
  unregistered chapter files (`argument`, `bookends`, `cauchy`, `continuation`, `derivative`,
  `domain_color`, `integral`). This one names them in the audit, and a planted control shows they
  cannot change the map.

## Checks

`check_reading_repair.py` passes; the output is in `check-run01.txt`.

- The patch regenerates, applies and reverses.
- Each chapter changes only in its import list and one call. Outside their tests, `draw.rs` gains
  only the helper, `mod.rs` only the table, and `main.rs` one doc comment.
- All existing tests are byte-identical, and rustfmt accepts the whole module tree.
- Every beat's words are identical, and the script the storyboard tool would generate is
  byte-identical.
- A Python mirror of the 4 new tests passes, and 16 defects planted in the text all fail it.
- The binder is in step and the validator passes, using the repo's own `read_units` and
  `direction_contract`. 45 binder defects and 18 validator defects are refused, with 3 controls.

## Measurable acceptance, for when takes exist

On the real render, the time from the card fully in to the next cue's start should be at least
`chars / 15`, the program's card floor elsewhere (`read_s`). Whether that holds depends on how long
the voice continues after the card's last sync. It must run at least this long:

| Cue | `chars / 15` | Window | Voice after last sync must reach | So the remaining words must run under |
|---|---|---|---|---|
| 04_4 | 7.40 s | 2.5 s | 5.00 s | 5.8 words/s |
| 05_8 | 3.60 s | 2.0 s | 1.40 s | 3.6 words/s |
| 07_2 | 6.93 s | 3.0 s | 4.43 s | 3.6 words/s |
| 09_5 | 7.13 s | 2.0 s | 4.93 s | 3.45 words/s |
| 10_5 | 8.67 s | 3.0 s | 5.97 s | 3.7 words/s |
| 11_6 | 7.13 s | 1.5 s | 6.53 s | 4.1 words/s |
| 14_3 | 5.93 s | 2.5 s | 3.63 s | 4.4 words/s |

The word counts behind the last column are upper bounds, so recheck against real takes. This is
arithmetic, not a measurement, and it predicts nothing about Oliver's pace. `chars / 15` is itself
a floor, and it does not model reading while a voice talks.

## Gaps

- **Not compiled, not run, not rendered.** The nine bodies never borrow-checked before build03 are
  now checked, but these lines are new. Clippy and the oracle have still not run.
- **No takes, no listening, no frames.** Whether these windows feel right is only answerable with
  those.
- **Scratch renders change more than the seconds.** 04_4 and 11_6 had no `line.end()` wait, so a
  render with no narrator now also waits for the estimated voice there.
- **The storyboard tool's estimate ignores `read N s`.** `tools/storyboard.py` counts 0 for it, so
  its advisory 32.0 min undercounts by 16.5 s. I did not change the tool.
- **Findings that predate this patch, unchanged by it:** `tools/storyboard.py --check` reports no
  prediction pause in 05_gallery, 08_loop_sums and 12_residues, and calls `script.tsv` stale. The
  staleness is only the header comment; all cue rows match.

## Strongest unresolved teaching defects

1. **The cards are still read under a competing voice for most of their length.** The windows add
   clean time at the end but do not make card and voice cooperate. At 11_6 and 04_4 the voice is
   saying something else for longest.
2. **The 10_5 theorem card is capped by the 4 s limit, not by what it needs.** It has the most to
   read (130 characters) and depends most on the voice overlap.
3. **Other cards are unchanged:** 13_11 (192 characters, followed by a 2.5 s music breath, on a
   chapter's last cue), 08_5 (97 characters) and the 11_8 code panel. The assignment named seven
   boundaries, so these are left for your judgment.
4. **Stale storyboard text at 04_4:** its picture column says `c(z − z₀)ᵐ` where the card says `a`.
   The card is what renders; I left the documentation text alone.
