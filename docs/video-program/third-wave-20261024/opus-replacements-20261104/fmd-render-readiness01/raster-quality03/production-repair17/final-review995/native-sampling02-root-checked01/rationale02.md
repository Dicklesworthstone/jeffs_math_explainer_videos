# Combined-repair native sampling plan 02 (CC1, 2026-11-10)

**Status.** This is a sampling plan for root's guarded native 4K render of bundle02. Its frames are predicted from source, the selected timelines and the narration timing files. Nothing here has been built, compiled, test-run, rendered, decoded or viewed. It accepts nothing.

## Binding

| Input | sha256 |
|---|---|
| bundle02 `staged/src/chapters/the_bump.rs` (arrow candidate03) | `90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7` |
| bundle02 `staged/src/chapters/loop_sums.rs` (square, gatefix) | `a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef` |
| bundle02 `staged/src/chapters/mod.rs` | `4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578` |
| bundle02 `staged/src/chapters/boundary.rs` (candidate03) | `4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932` |
| bases: the_bump / loop_sums / mod / boundary | `8e93012e…` / `8a13e8a1…` / `e1f4d1b8…` / `cd1900aa…` |
| bundle02 manifest / root receipt | `9c29672e…` / `2d9fbc03…` |
| reference master993 | `80244e88…` (133662 frames, 60 fps) |
| selected timelines 01 / 08 / 09 | `150147fe…` / `43c8bab2…` / `5a91b8f9…` |
| phrase timings (all six cues: WAV and text hashes match) | `b025eaa7…` |
| narration report (rebind313) | `69cb32ee…` |
| script.tsv (root-boundary760-source) | `234f7257…` |

How the hashes were checked:
- I hashed all eight bundle02 files myself.
- I diffed `loop_sums` a8d62f56 against the reviewed 977c77df. They differ only at lines 656–657, 773 and 1755–1756, the `centre_name_at` → `c_name_at` rename. None of those lines is a timing call, so the 08 frames apply unchanged.
- I did not redo the geometry, as root directed.

## How the frames were derived (`frame-windows01.py`)

These are the kit rules, read from explainer_kit 8072720d/ef035faf (identical to the native build's copy), draw.rs, and fmn a399d626's `RationalFrameClock`:
- A phrase's time is the line frame / 60, plus its byte-offset mark, or else its character share between the report's sentence onsets.
- `wait_until` costs `ceil(60·Δt − 1e-6)` frames.
- A play costs `ceil(exact(rt)·60)` frames (BN-02). So 0.2 s fades are 13 frames, 0.1 s is 7 and the 1.8 s slide is 109.
- An event that starts at clock frame F with n frames shows in local frames F..F+n−1.
- A linear tween is at α = k/(60·rt) on its k-th frame.
- Each chapter's master-equivalent frame is its start (01: 0, 08: 52106, 09: 67065) plus the local frame.

## Validation

The model reproduces every recorded frame I could find. It needed no tuning.

| Check | Predicted | Recorded |
|---|---|---|
| 01 breath row; chapter end | 6155; clock 6311 | timeline 6155; 6311 frames |
| 01 chunk-1 fade | 5498–5527 | root transition window 5496–5531 |
| 09_1 end + breath; 09_7 hold; 09_8 predict | 8297+36 = 8333; 14815; 15202 | timeline 8333; 14815; mark 15202 |
| 09 shrink fades (base) | out g74118–74130 and 74131–74143; bar in 74144–74161; tween from 74162 | CC2 sheet 680: dim 74125, gone 74139, dim 74145–47, full 74157; ρ jump between 74157 and 74163 |
| ARC / SUM / SHRINK fade-ins | g72745–72762 / 73242–73259 / 74294–74311 | sheets 650 / 662 / 683: full 72757 / by 73253 / 74308 |
| shrink 2; 09_7; 09_9 growths | g74666–74725; 80722–80901; 82678–82822 | crossings 74689–74717; 80757–80822; 82702 and 82742–82761 |
| 08 L5; caption dip; δ-circle; nest out | 6988–7008; 8301–8333; 9141–9165; 9767–9781 | L5 from 6996; deviation 13 at 8313, back by 8340; full by 9156; static to 9768 |
| 08 walk, P_n; quarter fade | 10218–10337, 10338–10355; 10787–10816 | L11 10339–10353; L12 10794 / 10821 |

The base sources have the same timing as the candidates. That rests on 228/228 identical events for boundary and 184/184 for square, with both skeleton checks frozen.

## Windows (local frames; master-equivalent = start + local)

| Window | Finding | Local | Master-equivalent | Required + extended |
|---|---|---|---|---|
| 01-control-unchanged | control | 36–5563 | 36–5563 | 6 + 0 |
| 01-arrow-fade-in | preview arrow (sheet 23/24 collapse) | 5562–5597 | 5562–5597 | 36 + 0 |
| 01-arrow-held | preview arrow | 5600–6274 | 5600–6274 | 11 + 0 |
| 01-arrow-fade-out | preview arrow | 6273–6310 | 6273–6310 | 21 + 17 |
| 08-control-unchanged | control | 3748–6987 | 55854–59093 | 5 + 0 |
| 08-L5-fade-in | L5 c, leader, rim junction | 6986–7012 | 59092–59118 | 17 + 10 |
| 08-L5-held | L5 | 7026–9139 | 59132–61245 | 8 + 0 |
| 08-L5-delta-circle | L5 × δ-circle | 9139–9450 | 61245–61556 | 18 + 14 |
| 08-L5-fade-out | L5 (no orphan leader) | 9740–9800 | 61846–61906 | 31 + 30 |
| 08-inset | c_name_at rename (binding only) | 9797–9900 | 61903–62006 | 6 + 0 |
| 08-L11-diagonal-in | L11 | 10024–10052 | 62130–62158 | 15 + 14 |
| 08-L11-walk | L11 motion (corners at 10247/10277/10307/10337) | 10216–10339 | 62322–62445 | 68 + 56 |
| 08-L11-perimeter-F5 | F5 glyph baseline | 10336–10483 | 62442–62589 | 26 + 0 |
| 08-L12-quarter-in | L12 tint/doubling of the redrawn c | 10785–10818 | 62891–62924 | 34 + 0 |
| 08-L12-held | L12 | 10846–13053 | 62952–65159 | 9 + 0 |
| 08-L12-fade-out | L12 fade-out lag | 13052–13105 | 65158–65211 | 28 + 26 |
| 09-control-unchanged | control | 36–5678 | 67101–72743 | 4 + 0 |
| 09-M1-arc | ARC_TEX closing bar | 5665–6163 | 72730–73228 | 22 + 17 |
| 09-M1-sum | SUM_TEX bar, L5 width | 6162–6487 | 73227–73552 | 21 + 17 |
| 09-M2-names-leader-in | M2, leader fade | 7050–7100 | 74115–74165 | 34 + 17 |
| 09-shrink1 | r 0.18→0.12 (0.17 at 7103 … 0.12 at 7138) | 7097–7140 | 74162–74205 | 26 + 18 |
| 09-hold-mid | leader as pointer vs radius (CC0 P1) | 7150–7215 | 74215–74280 | 3 + 0 |
| 09-M1-shrink | SHRINK_TEX closing bar | 7214–7600 | 74279–74665 | 22 + 17 |
| 09-shrink2-rest | r 0.12→0.06 (0.06 at 7660), L14 rest gap, leader ends | 7599–7665 | 74664–74730 | 67 + 0 |
| 09-rest-zero | rest state | 7680–7971 | 74745–75036 | 15 + 0 |
| 09-proof-gone | leader leaves with the proof | 7970–7998 | 75035–75063 | 15 + 14 |
| 09_7-growth | M4/L2: a out r 0.01–0.11 (13658–13676), hidden to 0.43 (13734), back by 0.53 (13752) | 13648–13837 | 80713–80902 | 70 + 120 |
| 09_7-after-control | a's opacity restored (expected identical) | 13900–14814 | 80965–81879 | 5 + 0 |
| 09_8-slide-card | slide and "1" card (expected identical) | 14815–15576 | 81880–82641 | 34 + 0 |
| 09_9-growth-card | a hidden 15624–15656, back by 15666; disk under the "1" card r ≈ 0.38–0.91 (15651–15705) | 15575–16123 | 82640–83188 | 84 + 103 |

The required tier is 750 stills, or 94 sheets of eight. With the extended tier, which fills every frame of the stepped motion and fade windows, it is 1231 stills (154 sheets). CC2's suggested windows (f74150–75060, f80750–80830, f82660–82770) are covered.

## Run order (`sampling-plan-check01.py`)

1. Before the build, run P1: `sources PLAN BUILD_CRATE_ROOT`. It passes on bundle02's staged files and fails on its bases (negative control).
2. Per chapter, run P2: `timeline PLAN CHAPTER RENDERED.timeline.tsv`. It substitutes the selected narration directory and requires the selected hash. The self-test passes on the three selected timelines and refuses a planted one-frame move of 09_9.
3. Run P3: `expect` gives 6311 / 14959 / 16504 frames for comparison with the decode.
4. Extract the stills from `frames-CHAPTER-required.txt`, or use `select` to get an ffmpeg expression.
5. Compare the controls with master993 before reading any repair window.

## Uncertainties

- **delivery309's `report.json` is not local.** The 01_6 and 09_8 phrases have no marks, so they depend on sentence starts. I took those from rebind313's report. Four local reports agree exactly on all six cues, and P2 plus the timeline's own rows pin the timing.
- **Updater timing.** Opacity and name updaters are assumed to act on the same frame as their tween value (±1 frame).
- **Duplicated end frames.** The 109-frame slide and the 145-frame 09_9 growth end with one frame that repeats the end state.
- **Chain starts assumed idle.** Four chains start at a phrase I assumed idle: "Keep that square", "The linear term", "So only the remainder" and "Now keep the small circle fixed". The base anchors above confirm each one. Every later anchor has at least 58 frames of slack. `lay_down`'s arrow count is not modelled.
- **Rate curves.** Fade rate curves were not evaluated. "First faint" and "full" mean the two ends of the run.
- **Box-model figures.** The pixel figures in the expectations come from CC2's and CC0's box models and are for measurement: the 7.6 px rest gap, a 12.05 px above `A_AT`, 3.4 px leader gaps, and 2.3 px (CC2) or 4.3 px (CC0) of leader inside the circle.
- **Controls.** "Expected identical" assumes the native engine, fonts and environment match master993's binaries.

## Not run

I did not build, run fmt, Clippy or tests, render, decode, extract stills or read any candidate frame. I did no listening, fine-sync or AV check, and I published nothing. Narration (ElevenLabs Frederick Surrey), phrase timings and every cue schedule are unchanged.

## Authorship limits

I am not independent of any of the three sources:
- I wrote arrow candidate03 (90b51c2b): root's `.buff(0.0)` moved into `preview_arrow()`, plus a native test.
- I wrote square candidate01, from which CC0's candidate02 and bundle02 a8d62f56 descend.
- I wrote the boundary base 752 (cd1900aa), which CC2's candidate03 changes.

My reviews of all three were source-only. Someone other than me should read the native stills.
