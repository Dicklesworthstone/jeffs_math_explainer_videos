# Oliver source plan for complex_analysis_visual, candidate 01

Bound to frozen increment06, script `2968b0fd7654`. This is a source plan and nothing
more. No take exists, nothing was compiled or rendered, nobody has listened, and no
timing is proposed. It says which words ElevenLabs Oliver must speak and which waits
the source already authors.

The canonical plan cannot be reused: it binds the old 18-cue script `515f5d8f6c4e`
(18 units, map oliver06). Canonical already holds maps oliver01 to oliver06, so this
one is oliver07. The binder refuses a version canonical already holds with other bytes.

## Files

| File | What it is |
|---|---|
| `explainer_corpus/complex_analysis_visual/narration_units.json` | 105 units, one whole cue each, exact script text, ids `oliver_<cue>` |
| `explainer_corpus/complex_analysis_visual/docs/narration-direction-map-oliver07.json` | 13 overrides and 13 prediction holds, each with hash, line, marker, next cue |
| `source-audit.json` | Every source hash, and each cue's say, next cue, word count and timing calls |
| `bind_oliver_plan.py` | Writes the three files above from a crate. Reads the Rust as text |
| `validate_oliver_plan.py` | Runs the repo's own `read_units` and `direction_contract`, plus independent line scans |
| `planted_defects.py` | Plants 36 defects in scratch copies and expects a refusal for each, plus 2 controls |
| `check-run01.txt` | The output of all of these on this plan |

The JSON paths mirror the canonical tree, so root can lay them over the candidate.

## Rebinding after compiler fixes

Any change to a chapter file holding a prediction, to `draw.rs` or to `mod.rs` changes a
hash in the map, so the plan goes stale and `--check` says so. A change anywhere else in
the chapters stales only `source-audit.json`. Rebind into a new folder:

```
python3 -I -B bind_oliver_plan.py REPO CRATE NEW_OUT [--map-version oliver07]
python3 -I -B validate_oliver_plan.py REPO CRATE NEW_OUT EMPTY_SCRATCH [--map-version oliver07]
```

Existing files with other bytes are refused, never overwritten. Pass `--map-version
oliver08` once oliver07 has reached canonical. `--check` writes nothing and exits 1 if
the folder is out of step with the crate. The units file changes only if the script or
the map does.

## Coverage and helper evidence

- **Say:** 105 calls for 105 cues, each once, all literal, all `let line = say(&kit,
  stage, "NN_M")?;` at the top level of `construct`, none in a loop, branch or helper.
  Source order equals script order in all 15 chapters. The only forwarding call is
  `draw::say` to `kit.say` (`draw.rs:94`).
- **Predict:** 13 calls, 13 `PREDICTS` rows and 13 storyboard `⟨predict⟩` rows agree both
  ways, windows included. Each passes the current `&line`, ends its cue on a question and
  has its next cue in the same chapter, with no timing call before that cue.
- **Words:** the longest cue is 14_3 at 98 words, the only one over 90, against the
  bound of 100. No digits and no non-ASCII characters. 27 of the 104 adjacent cue pairs
  exceed 100 words together, which matters only if a take is requested with its following
  cue as context (`coherent.request_text` refuses those).
- **Sentences:** `draw.rs`'s sentence rule and the narration report's agree on all 105 cues,
  so measured sentence starts can be used instead of the fallback estimate.
- **Non-literal wait arguments:** four, each resolved from the source, and every phrase and
  sentence checked against its owning cue. A new or changed one is refused.

| Call | Owner | Resolves to |
|---|---|---|
| `the_bump.rs:179` `line.wait_for(stage, phrase)` | 01_2 | "plus x to the fourth", "minus x to the sixth" |
| `gallery.rs:616` `line.wait_for(stage, phrase)` | 05_6 | "a pole at zero", "at minus one", "at minus two" |
| `gallery.rs:280` `line.until(stage, p, 2.5)` | 05_1 | "Three zeros inside"; the 05_2 call passes `None` and walks to `line.end()` |
| `counting.rs:151` `line.wait_sentence(stage, i + 2)` | 11_1 | sentences 2, 3, 4 |

- **Helpers that wait:** `coarse_loop` (11_8), `walk_and_count` in `gallery.rs` (05_1, 05_2)
  and `sample_round` (09_3, twice). Each is attributed to the cue at its call site.
- **Not in the film:** `thumbnail.rs:43` `stage.wait(1.0)`.

## What each hold is

Nothing here asks for silence in a take. Every wait below is on the native picture
timeline, and a take stays one complete cue with nothing inserted.

| Source | Count | What it is | In the map? |
|---|---|---|---|
| `predict` | 13 | After the cue and the kit breath, a window with no narration while the ring drains. `kit.listen` releases the music's duck, so the score can be heard | Yes, `kind: prediction` |
| Reading hold | 0 | No call waits past the voice to let a viewer read | No rows to bind |
| `kit.music_breath` | 16 on 15 cues | The chapter's last cue, then music alone (2.0 s; 13_11 2.5 s; 15_4 1.5 s, then 3.0 s over the title card). These are the storyboard's 15 `hold` rows | No: audit only |
| `kit.hold` | 77 | Waits for the voice to end plus the kit's breath, 0.6 s under `LineGaps::Constant` | No |
| `wait_until(line.end())` | 54 | The picture waits for the voice | No |
| `pause` | 9, of which 2 are in helpers | A picture beat before the cue's voice-end wait | No |

The binder refuses a `pause`, `stage.wait` or `wait_frames` after a cue's voice-end wait,
and a music breath inside a chapter, because either would be a hold nobody has classified.

The storyboard's hold column has only three kinds of value: blank (77), `hold` (15) and
`listen N s` (13). It authors no reading hold either.

**What "0 reading" means here.** It means no wait is authored past the voice for reading, and
the map's `scope` sentence should be read that way. It is stricter than the repo's older use:
81 direction maps have `reading` rows, including this topic's oliver01 to 06 (1 to 4 rows
each), and several of those rows mark a plain `kit.hold` where the cue stages something to
read. Under that use the clearest candidates are the seven card boundaries listed under
teaching risks below. They are left out on purpose. A `reading` row does nothing in
`narration_delivery`, which counts only `prediction` rows, but it marks the boundary as
protected in `narration_director.py audit`, and those short gaps are what that audit should
flag. This is root's call; adding rows would be a small binder change and a new map hash.

`pause` lines are placed before the voice-end wait in source order. Whether the voice is
still speaking at that moment needs measured takes.

## The 13 prediction holds

| Cue | Next | Window in source | Marker at |
|---|---|---|---|
| 01_4 | 01_5 | 5 s | `the_bump.rs:245` |
| 02_5 | 02_6 | 5 s | `turning.rs:376` |
| 03_3 | 03_4 | 4 s | `two_planes.rs:381` |
| 04_6 | 04_7 | 5 s | `reading_color.rs:154` |
| 06_8 | 06_9 | 5 s | `one_derivative.rs:346` |
| 07_3 | 07_4 | 5 s | `one_point.rs:392` |
| 09_4 | 09_5 | 5 s | `boundary.rs:241` |
| 09_8 | 09_9 | 6 s | `boundary.rs:458` |
| 10_4 | 10_5 | 5 s | `rigidity.rs:398` |
| 11_5 | 11_6 | 6 s | `counting.rs:266` |
| 13_5 | 13_6 | 5 s | `real_integral.rs:487` |
| 14_4 | 14_5 | 5 s | `continuing.rs:397` |
| 15_3 | 15_4 | 6 s | `closing.rs:325` |

The windows are the storyboard's own numbers in `chapters::PREDICTS`, not proposals made
here. The overrides carry no `range_s`.

## Remaining gaps

1. **All 105 Oliver takes.** None exists, and the assembly's 13 `native_visual_hold`
   dispositions need those takes.
2. **Compilation.** All Rust is uncompiled, so rebind after root's fixes.
3. **Listening, fine sync and word marks.** The phrase moments in `wait_for` and `until` are
   estimates until the takes are aligned.
4. **Pronunciation.** Neither the crate nor the canonical topic has a `lexicon.json`. The
   spoken names are Cauchy (06_6, 08_5, 09_2, 09_7), Riemann (05_8, 06_6), Liouville (10_6),
   plus zeta and gamma. Goursat, Rouché and Lanczos are not spoken in any cue. Letters need
   the same attention: `z` is spoken in 25 cues and can be read "zee" or "zed", and `i` (27),
   `x` (12), `R` (4), `f` (4) and `h` (4) must come out as letters. Any respelling belongs
   in the TTS input only, never in these units.
5. **Director audit.** `narration_director.py audit` needs `policy`, `range_s` and each
   hold's `reason` and `evidence`. Like the apportionment precedent this map omits them,
   because ranges would be invented timing. `read_units`, `direction_contract` and
   `narration_delivery` do not read those fields.
6. **Line gaps.** The crate uses `LineGaps::Constant`, left for root to decide with the takes.
7. **The title card.** The 3.0 s over the title follows 15_4, the last cue, so it cannot be a
   `visual_hold`: the schema needs an adjacent next cue. It is in the audit only.

## Strongest unresolved teaching risks

These can only be judged with measured takes or frames, so nothing here adds a hold.

1. **Seven `precisely:` cards have no reading window.** Each clears at the start of the next
   cue, with no wait before the fade (read in the source). Five arrive in their cue's last
   sentence. Their screen time is the remaining voice plus the 0.6 s breath. Words left is an
   upper bound, because a card arrives at or after its last sync:

   | Cue | Card at | Fades at | Card chars | Words left, at most |
   |---|---|---|---|---|
   | 05_8 | `gallery.rs:751` | `gallery.rs:777` | 54 | 5 |
   | 07_2 | `one_point.rs:367` | `one_point.rs:380` | 104 | 16 |
   | 14_3 | `continuing.rs:384` | `continuing.rs:394` | 89 | 16 |
   | 09_5 | `boundary.rs:287` | `boundary.rs:297` | 107 | 17 |
   | 10_5 | `rigidity.rs:411` | `rigidity.rs:421` | 130 | 22 |
   | 11_6 | `counting.rs:313` | `counting.rs:322` | 107 | 27 |
   | 04_4 | `reading_color.rs:121` | `reading_color.rs:126` | 111 | 29 |

   If a card proves too short, the fix is a source change and a rebind with a `reading` row.
   The binder refuses that new wait until someone classifies it. The other two `precisely:`
   cards, 13_11 (192 chars) and 08_5 (97 chars), stay through their chapter's closing music
   breath. The 11_8 code panel arrives before the cue's first sync and stays through its 84
   words and a 2.0 s breath, though a moving picture competes with it.
2. **Whether each answer stays hidden through its window.** The source draws nothing between
   `predict` returning and the next `say`, but what is already on screen during the window
   can only be judged on frames. The cards also paraphrase their questions, and only a person
   reading the frames can say that they ask the same thing.
3. **Long single takes.** 14_3 is 98 words and seven sentences in one request. The next longest
   are 11_8 (84), 14_5 (83), and 11_3, 11_7 and 15_2 (82 each). Rushing or drift across a take
   that long can only be found by listening.
