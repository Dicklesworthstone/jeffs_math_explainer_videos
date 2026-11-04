# Logarithm 01–04: independent source review

**Reviewer:** WhiteSwan, for BlueSalmon's Mail 1518, bead 87do.63. I did not author chapters 01–04. I did author the 08–10 stage, so its `draw.rs` impact has its own section below.

**Scope:** LavenderPike's `logarithm-repair01-04` candidate and root's `root-logarithm-fact-binding01` follow-up.

**Evidence:** source reading, patch replay in a scratch copy, one run of root's oracle, and the arithmetic in `checks/`. Nothing was compiled, rendered or heard.

## Verdict

**No blocker. Five small corrections are worth making: two medium, three low.**

Of the 56 frozen findings, 46 are addressed in source, counting the repaired part of 4.18. Four are only partly addressed, and LavenderPike's README openly leaves six unrepaired. I found no API misuse, lifecycle or ordering error, maths error or voiced-claim error in the new code. Hand layout checks found no regression.

Root's patch replays exactly and does what it says, except that one fact chain is left incomplete (R5).

None of the corrections changes a cue's words. On the scratch-rule arithmetic none adds scene time, and R2 brings the flip and the labels after it forward. All five are proposals: nothing here is applied.

## Defects, ranked

Line numbers refer to root's candidate for `building_ruler.rs` and `undoing_growth.rs`, and to LavenderPike's candidate for `slide_rule.rs` and `folded_decade.rs`.

### R1. Medium: the tangent stops as "it only ever climbs" begins (finding 4.5)

**Where:** `undoing_growth.rs:289` and `:308`; test `:936`.

**Defect:** the climb runs from 0.4 s to 4.4 s after sentence 3 starts. By character share, "it only ever climbs" starts about 4.09 s into a 10.52 s sentence on the scratch rule. So the tangent reaches the top 0.31 s into those words, then holds for about 6.1 s through "from near zero on the left to as large as you like on the right". Those are the words finding 4.5 named. With a time factor of 0.8 the motion overlaps the phrase by 1.13 s, and with 1.25 not at all.

The test at `:933–942` only asserts that the tween sits between the two `until_sentence` calls, so it passes either way.

**Correction:** replace `:308`, and add the constant beside `CLIMB_TO`.

```rust
// "it only ever climbs" starts 54 characters into this 139-character sentence.
const CLIMB_SHARE: f64 = 54.0 / 139.0;
```

```rust
let run = match kit
    .narrator
    .as_ref()
    .and_then(|n| Some((n.sentence_at("04_2", 3)?, n.sentence_at("04_2", 4)?)))
{
    Some((s3, s4)) => {
        super::draw::wait_until(stage, s3 + (s4 - s3) * CLIMB_SHARE)?;
        (s4 - stage.scene().time().to_f64() - 0.3).max(2.0)
    }
    None => 4.0,
};
play!(stage; tween(climb, CLIMB_TO, run)?);
```

Change the test's search string at `:936` to `tween(climb, CLIMB_TO, run)`. The run then covers the phrase and ends 0.30 s before sentence 4 at all three time factors. The tangent stays visible at its low start point from 0.4 s, while the voice says "since its slope is its height". The onsets are read at run time, so this follows real takes as well.

### R2. Medium: the "before" labels cannot be read (finding 4.6)

**Where:** `undoing_growth.rs:430`, `:449` and `:466`.

**Defect:** `run` and `rise` fade in over 0.4 s at `:449`. The fade-out at `:466` follows with no time passing between them, so the labels reach full strength for about one frame and are gone 0.65 s after they start. The finding asked for the legs to be labelled before the flip, and a viewer can't read that.

**Correction:** move one line, `kit.until_sentence(stage, "04_4", 1)?;`, from `:430` to between `:465` and `:466`. Then the triangle and labels arrive during sentence 0, "The mirror image shows something else", and hold until "Reflecting the graph swaps rise and run".

| Time factor | Labels at full strength | Flip, now | Flip, corrected | Sentence 2 starts |
|---|---|---|---|---|
| 0.8 | 0.85 s | ends 5.40 s | 2.40–5.00 s | 4.41 s |
| 1.0 | 1.39 s | ends 5.94 s | 2.94–5.54 s | 5.52 s |
| 1.25 | 2.06 s | ends 6.61 s | 3.61–6.21 s | 6.90 s |

On the scratch rule the flip now fills its own sentence, and at no factor does the move add time. The order test at `:878–882` still holds (live marks, then the fade, then the flip), and no test depends on the old position of the line. The triangle still appears without a fade, as it does now, just earlier.

I prefer this to adding a wait after `:449`, which would push the flip further into sentence 2.

### R3. Low: the nudge plays before its words (finding 2.6)

**Where:** `slide_rule.rs:126–128`.

**Defect:** the comment says the nudge belongs to "one that slides along it", but it plays from 1.40 s to 2.60 s and the phrase starts at about 3.86 s. By then the ruler has already moved and come back. The README lists no test for this finding.

**Correction:** insert one line before `:127`, using this file's own helper as at `:162`.

```rust
until_phrase(stage, &kit, "02_1", "one that slides")?;
```

The nudge then starts on the phrase and ends before sentence 1 at every factor checked. It ends 0.47 s early at 0.8, 0.90 s at 1.0 and 1.41 s at 1.25.

### R4. Low: "0.398" still has no visible unit (finding 2.4)

**Where:** `slide_rule.rs:155`, with the caption at `:134` and its fade at `:141`.

**Defect:** LavenderPike applied the review's suggested caption exactly. But the finding's complaint was timing: the number arrives after the caption has gone. That hasn't changed. The caption fades at the start of 02_1 sentence 2 and "0.398" arrives in 02_2.

**Correction:** carry the unit on the number. Replace `fact("slide.pos_a")?` at `:155` with:

```rust
&format!("{} of a full length", fact("slide.pos_a")?),
```

By CMR10 estimate the label is 1.99 wide and spans −5.40 to −3.41, inside its bar (−6.00 to −2.82). It is 0.59 clear of C's index line at −2.82 and clear of the later "0.505" at −0.80. `sum` and `moral` sit on lower rows. It can keep its existing fade and exit.

### R5. Low: root's `real.growth_target` does not drive the numbers beside it

**Where (root's candidate):** `facts.rs:159`, `:164–165`; `oracle.py:111`, `:117–118`; `undoing_growth.rs:750`.

**Defect:** the patch binds the "×1000" label and the dot's height to the new fact. The formula beside them still uses a literal `\ln 1000`, and `real.ln_thousand` and `real.thousand_years` are still computed from a literal 1000. The dot's x position comes from `real.thousand_years`. If the fact ever changed, the label and height would move while the formula and x position would not. Nothing is wrong on screen today.

**Correction:**

```rust
// facts.rs
let growth_target = 10_u32.pow(3);
f.add("real.growth_target", growth_target)?;
// ...
f.add("real.ln_thousand", dec(f64::from(growth_target).ln(), 2))?;
f.add("real.thousand_years", (f64::from(growth_target).ln() / RATE).round() as i64)?;
```

```python
# oracle.py
growth_target = 10 ** 3
add("real.growth_target", growth_target)
# ...
add("real.ln_thousand", dec(math.log(growth_target), 2))
add("real.thousand_years", round(math.log(growth_target) / rate))
```

At `undoing_growth.rs:750`, write `\ln {}` and pass `fact("real.growth_target")?` as the first argument. The count stays at 99 and every value stays the same (`6.91`, `138`).

## Coverage of the frozen findings

"Addressed" means the source change does what the finding asks. It is not picture acceptance.

| Status | Findings | Count |
|---|---|---|
| Addressed | 1.1–1.13, 1.15; 2.1–2.3, 2.5, 2.7, 2.9; 3.1–3.7, 3.9, 3.10, 3.13, 3.14; 4.1–4.4, 4.8–4.17 | 45 |
| Partly addressed | 4.5 (R1), 4.6 (R2), 2.6 (R3), 2.4 (R4) | 4 |
| Split | 4.18: the repaired part is addressed, including `\log_2` through `tex_left`; the rest is left by decision | 1 |
| Left by stated decision | 1.14, 2.8, 3.8, 3.11, 3.12, 4.7 | 6 |

Each decision is stated in the README's "Not repaired, and why", and I have no objection to any of them. 1.14, 3.8, 3.11, 3.12 and the rest of 4.18 still need a decision from real frames or from root.

**Finding 1.13:** the marks now stay put and the unit bar lengthens, so "the same ruler, stretched" no longer contradicts a compressing picture. I accept this as repaired. Only frames can show whether it reads as a stretch.

**Finding 4.7 (hollow arrows)** is still open in this stage, but a fix is staged. See "Shared `draw.rs` impact".

## What I checked and found clean

### Pinned API (fmn a399d626 and the Kit)

Read directly, and all the new call forms match:

- **Live objects:** `always_redraw` (`fmn-mobject/src/dynamics.rs:237`), `add_value_tracker` (`dynamics.rs:79`), `add_to_scene` and `attach` (`fmn-mobject/src/stage.rs:811`, `:1045`). The new closures take the same form as canonical ones.
- **Scene clock and z-order:** `time()` (`fmn-scene/src/runtime.rs:763`), `remove` (`:960`), `bring_to_front` (`:966`). `stage.scene().time().fps()` has a precedent at Kit `narration.rs:615`.
- **Shapes:** `polyline` (`fmn-library/src/poly.rs:672`), `Arrow` (`fmn-library/src/line.rs:471`).
- **Fades:** `fade_in` and `fade_out` (`fmn-anim/src/fading.rs:52`, `:80`). Each transforms a mobject onto itself. `fade_in` starts from opacity zero and `fade_out` restores the begin-time state on completion, so on this reading the 0.55 stroke opacity from repair 1.6 is kept.
- **Kit text:** `text`, `text_left`, `tex`, `tex_left` (`kit.rs:403`, `:449`, `:480`, `:524`) and `until_sentence` (`:325`).
- **Kit timing:** `frames_covering` and `wait_frames` (`narration.rs:605`, `:613`), `sentence_at` (`:1065`), `length` (`:1076`), and `sentence_starts` and `phrase_marks`.
- **Chapter helpers:** `draw::wait_until` (`draw.rs:317`) and `Facts::add` (takes `impl Display`).

### Scene lifecycle and order

- I audited every fade list in the four chapters: no mobject fades out twice, so nothing re-roots at full strength.
- In 04_4, each live red mark starts exactly on its still original, and the graph's closing fade takes the live marks only. The test at `:878` enforces this.

### Maths and voiced claims

- The flip sends the floor to the wall and the bar to the negative x-axis.
- The flipped triangle has slope 1/x.
- The product bars land on 8, and the `two_steps` ticks align with 1, 2, 4 and 8.
- The script is unchanged (sha256 `3e4e34af7090e448` everywhere), so the existing Oliver takes still match it.

### Layout, by hand with CMR10 widths

Clear in all of these:

- **01:** rows and words. The 8/10 gap is 0.224, and the hairline ends sit 0.07 from the digits.
- **02:** `stops`, `off`, `ghost` and the bracket.
- **03:** rows, `land_word` and `keep`.
- **04:** `ln2_word`, `no_log`, `ln_name`, `rise1`, `run1`, the product rows (`landing_word` is 0.14 from `ln_word`) and the ten rows.

These are the tightest values I found. They are estimates, and frames still have to confirm them.

### Glyphs

The only non-ASCII characters in chapter code are `½ × √` (01), `×` (02, 03) and `× ÷ → ≈` (04). All are in the `TEXT` supplement in `mod.rs`. `₂` no longer appears in chapter text. `\text{}` and `\ ` follow existing TeX forms. Rendering is unseen.

### Other literals on screen

I recommend no change to `×2` (`undoing_growth.rs:659`), `one lap = ×10` (`folded_decade.rs:325`), or `= 1` and `×10` (`slide_rule.rs:134`), which describe the definition of the scale. The same goes for `70 ÷` (`undoing_growth.rs:732`), which is the rule's round number.

## Root's fact-binding patch

- **Replay:** on current canonical, LavenderPike's patch and then root's reproduce all four chapter candidates and `oracle.py` byte for byte. `facts.rs` ends at sha256 prefix `d187e97a9828d90a`, root's recorded candidate. Canonical still matches every recorded base.
- **Oracle:** I ran root's candidate `oracle.py`. Its output matches `oracle-result01.tsv` exactly (sha256 prefix `35a4d899b2c7a0f8`).
- **Rust against Python, by reading only:** the six new keys should print `4, 10, 2, 4, 8, 1000` from both sides. They are added at matching positions, and the count goes from 93 to 99. Root lists this comparison as not run, and it still needs the native run.
- **Types:** `fact(...)?` is `&'static str`, so the new `mark` call fits. `fact_f64` is already imported in both chapters.
- **Tests:** root's source-order assertion was updated. No Rust test in either changed chapter still searches for a string the patch removed.

## Shared `draw.rs` impact: my own 08–10 stage

**This section is not independent, because I wrote that stage.**

- **What it changes:** my `shared-draw-helpers.patch` (sha256 prefix `3871aa3b88a300a1`) changes only `matrix_card` sizing and `clipped_grid`'s test, and appends a test module.
- **Effect on 01–04:** none of the four candidates calls either helper (grep count 0 in each). Neither LavenderPike's nor root's patch touches `draw.rs`, and mine does not touch `facts.rs` or `oracle.py`, so root's 99 stands.
- **Replay:** applied after both patches, mine reproduces my candidate `draw.rs` (`82c7d086462eafd3`) with no conflict.

### The 05–07 arrow patch (BronzeJaguar's; closes 4.7 if root applies it)

- **What it changes:** `logarithm-repair05-07/patches/draw-arrow-solid-head.patch` (D12, Mail 1531, sha256 prefix `f30d37bd5c2e0021`) changes only the body of `draw::arrow`.
- **Effect on 01–04:** three call sites here would change: `slide_rule.rs:168`, `folded_decade.rs:179` and `undoing_growth.rs:377`. Each handle is only faded or listed (`slide_rule.rs:182`, `:194`; `folded_decade.rs:194`, `:200`; `undoing_growth.rs:386`, `:415`), so a group handle is safe on this reading.
- **Replay:** `git apply --check` passes on top of my draw patch.

Whether to apply it is root's decision. How the solid heads look is unseen.

## Not checked

- **Native build:** no compile, native tests, Clippy or Rust-against-Python fact comparison. The corrections above are uncompiled too.
- **Pictures and sound:** no frames, no audio, no fine sync. This is not audio or picture acceptance.
- **Oliver timing:** I found no measured timing for these cues in the logarithm paths I searched. All seconds here come from the Kit's scratch rule, and the factors 0.8 and 1.25 are illustrative. R1, R2 and R3 all read onsets at run time, so their logic carries over, but the exact margins will move.
- **LavenderPike's stub checks:** not rerun, as Mail 1518 rules out stand-ins. Some of their strings describe text that root's patch now builds from facts.
- **Chapters 00 and 05–12:** out of scope, apart from the shared helpers above.
- **Root's `prediction-marker.patch` (Mail 1542):** not reviewed. The request reached my inbox as this review was finishing and is outside the assignment I was given. Nothing here says anything about that patch.

## Bindings

All are sha256 prefixes. Canonical still matched this table when last checked, at 11:29 UTC on 2026-11-04.

| File | Canonical | LavenderPike candidate | Root candidate |
|---|---|---|---|
| `building_ruler.rs` | `6ed5ef5b9bf170ad` | `e9a6cd9d4d75362a` | `ae125e5ec9ec9bd6` |
| `slide_rule.rs` | `c3ec0193b7cee97e` | `6771c36b3ee96e8f` | n/a |
| `folded_decade.rs` | `c753d51b9f0cfdda` | `acfc7f9c198b3d3d` | n/a |
| `undoing_growth.rs` | `eafafef71a9c31b8` | `bb3fd7990f90189e` | `a19945fb4731925e` |
| `facts.rs` | `7bf886f9762f1904` | n/a | `d187e97a9828d90a` |
| `oracle.py` | `05a0dc9c669b5a55` | n/a | `fc8fbf0adc325700` |
| `script.tsv` | `3e4e34af7090e448` | unchanged | unchanged |
| `draw.rs` | `59ebd09d2fa5c793` | unchanged | unchanged |
| `model.rs` | `8c9e3c6abfcc48d5` | unchanged | unchanged |

Patches: LavenderPike `696e89f02bb227c3`, root `e3d55266c37fedfe`.

The arithmetic is in `checks/timing_arithmetic.py`, with its output in `checks/timing_arithmetic.out.txt`. Run it from the repo root with `python3 -I -B`.
