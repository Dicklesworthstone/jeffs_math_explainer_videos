# Prediction marker: independent source verdict

**Reviewer:** WhiteSwan, for BlueSalmon's Mails 1542 and 1554. I did not write this patch.

**Reviewed:** root's `narration-binding-stage01/prediction-marker.patch`, sha256 `750309fe359f8984814765aef60edc52a131cc1a257a0ab17882c07902362229`.

**Evidence:** source reading of each consumer, one run of the real Python `prediction_end` on my own synthetic cases, a byte check and `rustfmt --check`. There was no native build, timeline, caption file, score, picture or audio. This is a source verdict, not acceptance.

## Verdict

**Approved as source: the one line is correct and necessary, and it belongs where it is.** It should land together with the final map's 13 declarations. It does more than extend captions, because it also adds a musical cue at each prediction, which needs listening after the fresh score.

## Why the line is right

```rust
kit.hold(stage)?;
kit.mark(stage, "predict");   // the added line, in draw::predict
```

- **It's needed.** Without a `predict` mark, both consumers see nothing: `tools/narration_delivery.py:249` and `crates/explainer_kit/src/captions.rs:533`.
- **It matches the Kit.** `props::predict_pause` uses the same hold, then mark (`props.rs:216–217`). Only logarithm's custom helper lacked it. A scan of every corpus crate's `src/chapters/draw.rs` found no other custom `predict`.
- **It moves nothing on screen.** `Kit::mark` takes `&Stage` and `Narrator::mark` only pushes to the log (`narration.rs:1258`). It adds no wait and no mobject, and it does nothing without a narrator. So frames and cue starts are unchanged, by reading.
- **The types fit.** The call form is the same as `props.rs:217`.
- **The bytes check out.** Base `draw.rs` is stage02's `93c2f95108d11af5`. Root's candidate is `3df5c4a389cd6688` and passes `rustfmt --check`. It differs by the one added line.

### Placement against the window

The consumers need `voice end <= mark < next cue start`. Root's five cases tested the boundary ("at voice end"), but the real placement sits well inside the window:

| Step | Source | Effect |
|---|---|---|
| `kit.hold` | `narration.rs:1164`, `LineGaps::Constant` in logarithm's `main.rs` | Mark lands at voice end + `BREATH`, rounded up to a frame |
| `music_breath(THINK)` | `draw.rs`, `narration.rs:1302` | 4.0 s, starting at the mark's frame |
| Bar fade, then `say` | `draw.rs`; `narration.rs:1137`, `:1318` | About 0.25 s, then the next cue |

In my cases, at 30 and 60 fps, the mark falls 0.600–0.610 s after voice end, and the next cue starts 4.25–4.27 s after the mark. So there's no float-edge risk on either side. The last caption would stay up about 4.85 s past the end of the voice.

The real `prediction_end` returned the expected value for all six realistic placements. It returned `None` for an undeclared pair, a pair declared twice, a chapter's last line and a declared pair with no mark. See `checks/prediction_marker_cases.out.txt`.

### The 13 call sites

There are 13 `predict` calls, matching root's count. Each is followed directly by the next cue's `say`, with nothing between. None is at a chapter's end, so `following` is never `None`. Root's sentence splits (05_2/05_20, 06_6/06_20) change none of these pairs.

| Chapter | Cue pair |
|---|---|
| `ratio_ruler` | 00_4 → 00_5 |
| `building_ruler` | 01_3 → 01_4 |
| `slide_rule` | 02_2 → 02_3 |
| `undoing_growth` | 04_6 → 04_7 |
| `complex_turns` | 05_1 → 05_2, 05_5 → 05_6 |
| `lifting_loops` | 06_2 → 06_3, 06_4 → 06_5 |
| `branch_cut` | 07_3 → 07_4 |
| `product_rule` | 08_1 → 08_2, 08_4 → 08_5 |
| `unreachable` | 11_3 → 11_4 |
| `what_it_forgets` | 12_5 → 12_6 |

## What root should know before landing it

### 1. Each mark can also add a musical cue (needs listening)

`CueKind::from_label("predict")` has no `:` in the label, so it falls through to `Reveal` (`crates/explainer_score/src/score.rs:755–765`). The score then places a soft reveal dyad on the mark (`:2397–2460`: `Track::Cue`, velocity 0.4, hold 1.5 s), unless the moment is silent or too dense with cues.

That means up to 13 new audible events, each at the start of a think gap. The mark sits in a gap with over 4 s clear, so the "clear second before the next sentence" rule won't skip them.

Films using `predict_pause` already behave this way, so I'm not calling it a defect, and a comment at `score.rs:1797–1799` refers to such a cue. Mail 1542 gives extending captions as the purpose and says a fresh score is required, but it doesn't mention this cue. Whether a soft "reveal" sound suits the moment a viewer is asked to think is a listening decision after the fresh score. If it doesn't suit, a label that the score ignores would need changes to every consumer, so that's a separate decision.

### 2. The marker and the map must land together

The two caption paths treat a mark differently:

| Path | Declared pair required? | Effect of a mark on an undeclared pair |
|---|---|---|
| Native, `captions.rs:526–538` | No | Extends the line's last caption to the next cue |
| Delivery, `narration_delivery.py:244` | Yes, exactly once | No hold |
| `tools/final_check.py:435–438` | Accepts a held caption only from delivery's holds | Native extended captions fail: "captions outside their audio" |

So a mark with no declaration fails closed at `final_check` if the captions come from the native path. That's the safe direction, but it makes all 13 declarations mandatory once this patch is in.

**Today none of them exists.** The newest map in stage02, `narration-direction-map-oliver11.json`, declares 00_1 → 00_2, 01_1 → 01_2, 03_1 → 03_2 and 04_1 → 04_2: 0 of the 13.

### 3. The four old declarations should go from the final map

All four old pairs are still adjacent cues in the current script, so `narration_director.py:119–129` wouldn't refuse them for adjacency. `narration_director.py:342` and `:365` would then mark those gaps as protected visual holds, although nothing is held there. They wouldn't produce caption holds, since no mark sits in those gaps.

### 4. None of the 13 can be a sounding coherent seam

`Narrator::say` (`narration.rs:1195–1214`) refuses a contiguous pair whose next cue doesn't start at the previous cue's exact end, and the think gap guarantees it won't. This limit exists with or without this patch, but it matters for the request plan being rebound now.

### 5. Smaller effects, no action needed

- **`prosody_qa`:** these gaps become class `PREDICT` rather than `WINDOW` (`tools/prosody_qa.py:345–350`). I found no threshold tied to that class, so this is a label change.
- **`narration_director`:** the mark adds a zero-length window (`:202–204`). The breath row in the same gap already protected it (`:363`), so the outcome is unchanged.
- **Timeline files:** each gains a `mark	predict	<frame>` row (`crates/explainer_score/src/timeline.rs:101`), which the delivery parser accepts (`narration_delivery.py:105`). Anything bound to a timeline's hash goes stale, which fits root's "fresh score required".

## Optional

The patch has no native test. A short source-order test in `draw.rs` could guard it: `kit.hold(stage)?` before `kit.mark(stage, "predict")` before `add_value_tracker`. I don't consider this a blocker, and I haven't staged it, since `draw.rs` isn't in my scope.

## Not checked

- Native timeline output, real caption files, the fresh score, pictures and audio. Root lists these as open too.
- `final_check` on real files. The table above comes from reading it, and only `prediction_end` was executed.
- Which caption path this film's final delivery uses.
