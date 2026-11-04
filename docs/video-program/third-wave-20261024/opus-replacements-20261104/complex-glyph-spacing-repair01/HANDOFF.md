# Complex glyph spacing repair 01: F1 `z_0` and F3 unary minus

For BlueSalmon, Mail 1952. From BronzeJaguar, 2026-11-04.

**State: staged, not applied. Uncompiled, no native probe, no render.** Canonical sources were only read. The evidence below is source reading, font-table arithmetic and syntax-only checks. Your guarded probe is the first time either new form will run.

## The answer

**Grouped thin space is supported by the pinned parser and layout, on reading.** `z_{\,0}` should move the subscript right by 0.1167 em with no vertical change. `|{-3z}|` should drop both binary spaces around the minus.

This exact form has no upstream test behind it. A search of the pinned `fmd-math`, `fmn-tex` and `fmn-library` finds no `_{\,` or `^{\,`. The nearest is a parse-only test of the fragment `\sqrt{\,` (`fmd-math/tests/parse_constructs.rs:751`). `\,` on its own is in the supported-constructs test (`commands.rs:953`). See `checks/outputs/upstream_search.txt`.

**The braces are required.** `z_\,0` would not work: the space alone becomes the subscript, it lays out at zero width, and the 0 falls to the baseline at full size.

## Old and new strings

Ten distinct strings, on 12 source lines in 6 files. `inventory.tsv` has every line, with symbol, call site, size and registry row. `probe-strings.tsv` has the strings alone.

| Fix | Where | Old | New |
|---|---|---|---|
| F1 | `one_derivative.rs:26` QUOTIENT_TEX | `\frac{f(z_0 + h) - f(z_0)}{h}` | `\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}` |
| F1 | `one_derivative.rs:44` SETTLES_TEX | `\to\, f'(z_0)` | `\to\, f'(z_{\,0})` |
| F1 | `one_derivative.rs:47` TIMES_TEX | `\times\, f'(z_0)` | `\times\, f'(z_{\,0})` |
| F1 | `one_derivative.rs:211`, `mod.rs:407` | `z_0 = 1 + i` | `z_{\,0} = 1 + i` |
| F1 | `one_derivative.rs:217`, `mod.rs:408` | `z_0^2 = 2i` | `z_{\,0}^2 = 2i` |
| F1 | `loop_sums.rs:50` LINEAR_TEX | `f(z) \approx a + b\,(z - z_0)` | `f(z) \approx a + b\,(z - z_{\,0})` |
| F1 | `loop_sums.rs:51` EACH_TEX | `\oint (a + b\,(z - z_0))\,dz = 0` | `\oint (a + b\,(z - z_{\,0}))\,dz = 0` |
| F1 | `residues.rs:31` ZERO_TEX | `\frac{f'}{f} \approx \frac{m}{z - z_0}` | `\frac{f'}{f} \approx \frac{m}{z - z_{\,0}}` |
| F1 | `closing.rs:28` LIMIT_TEX | `\lim_{h \to 0}\frac{f(z_0 + h) - f(z_0)}{h}` | `\lim_{h \to 0}\frac{f(z_{\,0} + h) - f(z_{\,0})}{h}` |
| F3 | `counting.rs:50` LEASH_TEX | `|z^5 + 1| \le 2 < 3 = |-3z|` | `|z^5 + 1| \le 2 < 3 = |{-3z}|` |

Three things about the edits:

- `mod.rs:407-408` must move with `one_derivative.rs:211,217`. Those two supplement entries are literals, so leaving them would keep the old overlapping rows on the probe beside the new ones.
- `one_derivative.rs:217` is also reflowed to one argument per line. The longer literal takes the call past rustfmt's call width, and the no-reflow control below draws a format diff. Lines after it in that file move down by 7, so the `used_at_baseline_line` column in `inventory.tsv` refers to the baseline.
- `boundary::QUOTIENT_TEX` is a different constant with no `z_0`, and it is not touched.

## How `z_{\,0}` runs through the pinned code

All citations are `franken_markdown@126a530 fmd-math/src/`. `checks/outputs/pinned_excerpts.txt` prints each cited line with its file hash.

1. **Parse.** `script_cluster` takes `z` as the base (`parse.rs:1347-1351`) and calls `argument("subscript")` (`:1386`). A `{` goes to `math_group_body` (`:861-863`), which returns `NodeKind::List` (`:348`). Inside it `\,` is `Space(Thin)` (`:361`). The subscript is `List[Space(Thin), Symbol 0]`.
2. **Dispatch.** `scripts()` lays the subscript out in script style (`layout.rs:1150-1151`). The single-character group shortcut (`:473-486`) needs exactly one Ord symbol, so two items take the general arm to `hlist` (`:487-488`) and `line` (`:309`).
3. **Glue.** `line` turns the space into glue of `mu / 18 * ctx.size()` (`:337-343`). `size()` is `size_factor * decl_size` (`:203-205`) and Script is 0.7 (`style.rs:44`). Thin is 3 mu (`node.rs:236`). So 3/18 × 0.7 = 0.1167 em.
4. **Assembly.** `x += glue.natural` runs before the digit takes `dx: x` (`layout.rs:377-383`), so the digit sits past the glue and the box width includes it. Glue draws nothing (`mbox.rs:313`).
5. **Placement.** Rule 18b puts the box at `dx: base_w` (`layout.rs:1172-1183`), and its shift reads only height and depth, which glue does not change. Rule 18e, for `z_{\,0}^2`, keeps the subscript at `base_w` and the superscript at `base_w + delta` (`:1198-1227`), and its clash test also reads only heights.

The surface adds nothing between the film's string and this path. `Tex` is display math (`franken_manim@a399d62 crates/fmn-library/src/tex.rs:176-180`) and the engine passes the source straight through (`crates/fmn-tex/src/engine.rs:534-540`). Every call site here passes an empty `t2c`, and none uses `tex_spans`.

**Why the braces matter.** For `z_\,0`, the control-symbol arm of `argument` returns the bare space as the whole subscript (`parse.rs:886-895`), and a lone `Space` lays out as `kern_box(0.0)` (`layout.rs:826-833`). `z_{\,}0` would fail the same way. The space and the digit must share one group, space first.

**Available spaces.** `\,` is the smallest single positive space. The parser has `\,` 3 mu, `\:` 4, `\;` 5, `\ ` 6 and `\!` −3 (`parse.rs:361-365`), and the command table adds only `\quad` and `\qquad` (`commands.rs:680-681`). It has no `\kern`, `\mkern` or `\hspace`. A narrower gap is possible as `z_{\;\!0}`, 5 − 3 = 2 mu. That form is not patched and appears only as two comparison rows in `probe-strings.tsv`.

## How `|{-3z}|` works

`char_class` does not list ASCII `|`, so it takes the default arm and is Ord (`atom.rs:205-218`). U+2223 `∣` is listed as Rel, but these strings use the ASCII bar. `-` maps to `−` (`parse.rs:170`), which is Bin (`atom.rs:207`).

- **Old form.** The atom before the minus is an Ord bar, which is not in the degrade list (`atom.rs:287-299`). The minus stays Bin and takes a medium space each side. That is F3.
- **New form.** The minus opens its own list, so `prev_atom` is `None` and it degrades to Ord (`atom.rs:287-288, 300-302`). The group itself is Ord (`atom.rs:238, 244`), so it takes no automatic space beside the bars. Font kerning cannot cross it, because a list carries `char_glyph: None` (`layout.rs:487-490`).

## Predicted effect

Computed from the bundled fonts and the pinned values, not rendered. See `checks/outputs/predict_spacing.txt`. A box gap is the horizontal distance between ink bounding boxes. A positive one means the two inks cannot touch horizontally, whatever their shapes. A negative one only says they may.

**F1, `z` to subscript `0`.** `z` overhangs its advance by 0.0562 em. The script `0` has 0.0273 em of left bearing.

| Form | Box gap | Size 28 | Size 30 | Size 32 | Size 36 |
|---|---|---|---|---|---|
| `z_0` now | −0.0288 em | −1.1 px | −1.2 px | −1.2 px | −1.4 px |
| `z_{\,0}` patch, 3 mu | +0.0879 em | +3.3 px | +3.6 px | +3.8 px | +4.3 px |
| `z_{\;\!0}` not patched, 2 mu | +0.0490 em | +1.9 px | +2.0 px | +2.1 px | +2.4 px |

Pixels are at 1080p; double them for 4K. The model agrees with the frozen glyphs02 measurements at probe size 36: `p_1` predicts +3.1 px and measured 4, `c_0` predicts +0.8 px and measured 1, and `z_0` on row 716 predicts −1.4 px where the ink measured as one component at every level.

So 3 mu should open a gap about like `p_1`, and wider than `c_0`. That is the one thing here I would judge by eye.

**F1, `z_{\,0}^2`.** The subscript now starts 0.0605 em to the right of the superscript. In `z_0^2` it starts 0.0562 em to the left. On the frozen crop of row 717 the `2` ends at x90 and the merged `z` and `0` end at x88. A 5.7 px move puts the `0` about 4 px past the `2` on the probe. This is a style question, not a collision.

**F3 at film size 34.** Bar to minus goes from +18.0 px to +7.8 px. Minus to `3` goes from +14.5 px to +4.3 px. The string narrows by 0.151 units.

**Widths in the film.** Ink widths come from the glyphs02 probe rows, scaled to each call's size.

| String | Size | Width, units | New edges |
|---|---|---|---|
| QUOTIENT_TEX | 36 | 2.496 → 2.580 | −1.290 to +1.290 |
| SETTLES_TEX | 32 | 1.172 → 1.209 | +2.145 to +3.355 |
| TIMES_TEX | 30 | 0.877 → 0.912 | +2.844 to +3.756 |
| `z_0 = 1 + i` | 28 | 1.170 → 1.202 | plane-relative |
| `z_0^2 = 2i` | 28 | 0.835 → 0.868 | plane-relative |
| LIMIT_TEX | 32 | 2.746 → 2.820 | −3.810 to −0.990 |
| LINEAR_TEX | 28 | 2.316 → 2.349 | +2.426 to +4.774 |
| EACH_TEX | 28 | 3.048 → 3.080 | +2.060 to +5.140 |
| ZERO_TEX | 32 | 1.422 → 1.460 | −4.030 to −2.570 |
| LEASH_TEX | 34 | 3.512 → 3.361 | −5.130 to −1.770 |

The tightest pair I checked is QUOTIENT_TEX to SETTLES_TEX on one line in chapter 06, whose gap goes from 0.916 to 0.855 units. No placement reads these widths: each string is centred by `kit.tex`, and later code only fades them or, for `names[1]`, shifts by a fixed vector (`one_derivative.rs:389`).

No duration or cue word changes. `untouched.sha256` holds `script.tsv` and `narration_units.json` among others. The frames that show these strings change, in chapters 06, 08, 11, 12 and 15.

## Tests affected

I expect no test to need editing. These are mirrors over source text, not test runs, so your Cargo gate decides. See `checks/outputs/tests_affected.txt`.

- **One test names a changed constant,** of 43 test functions in the six files. `the_leash_formula_holds_on_the_unit_circle` (`counting.rs:1176`) asserts `LEASH_TEX.contains(r"\le 2 < 3 =")` at `:1178`. The new string still contains it.
- **`no_supplement_is_empty_or_duplicated`** (`mod.rs:656`) reads the new strings through `TEX`. The mirror resolves 98 entries with no duplicates. It does not resolve the other 65 (array elements and similar), but the new forms occur nowhere in the baseline crate, so they cannot collide with those.
- **Two tests scan changed files as text:** `the_quintic_is_not_painted_before_its_question_is_over` (`counting.rs:1257`) and `each_card_outlasts_its_reading_window` (`mod.rs:830`, which reads `counting.rs` and `loop_sums.rs`). Counts of the patterns they look for are unchanged.
- **Nothing found outside the crate.** I searched `crates`, `tools` and `docs/video-program`, leaving out the `opus-replacements-20261104` staging, for four distinctive forms: `z_0 = 1 + i`, `z_0^2 = 2i`, `|-3z|` and `f(z_0 + h)`. There were 0 hits. Inside the crate, `oracle.py:355` has `|-3z|` in a comment, which stays correct mathematics. `counting.rs:1177` and `model.rs:1002` are comments too.
- **The probe's `check_groups`** (`crates/explainer_kit/src/glyphs.rs:178`) passes on every new string, by a port of it in `checks/substitutions.py`.

## Syntax-only checks

All in `checks/outputs/`, with scripts in `checks/`. Nothing below compiles or type-checks. The edits change string contents and one call's line breaks.

| Check | Result |
|---|---|
| `substitutions.py check .`: each old line at its recorded place, no unlisted raw string | 12 ok, 0 mismatches, 0 unlisted |
| `git apply --check` from the repo root | passes on all 6 files |
| rustfmt on baseline crate, repo `rustfmt.toml` | exit 0 |
| rustfmt on patched crate | exit 0, no format diffs, no parse errors |
| Control: new literal without the reflow | exit 1, format diff at `one_derivative.rs:214` |
| Control: stray brace in `closing.rs` | exit 1, parse error, so the run reaches the chapters |
| Round trip outside the repo, `git apply` then `git apply -R` | equals the edited copies, then the baseline |

## Applying it

From the repo root:

```
D=docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-glyph-spacing-repair01
shasum -a 256 -c $D/baseline.sha256
git apply --check $D/complex-glyph-spacing.patch
git apply $D/complex-glyph-spacing.patch
shasum -a 256 -c $D/result.sha256
```

- **If canonical has moved,** `python3 -I -B $D/checks/substitutions.py check .` names any line that differs. Every edit replaces one whole line, so a moved line is easy to redo by hand.
- **Older stage trees still hold the old strings,** including `complex/reading-repair02`, `complex/clippy-repair05` and the `candidate` and `baseline` trees of the picture-repair stages. Syncing one of those over canonical after this patch would undo it.
- **Two staged patches carry these lines as context:** `complex-picture-repair-06-10/patches/12-mod-lists-06.patch` and `complex-picture-repair-11-15/patches/11-counting.patch`, plus the copy of the latter in `complex-preview01-review-11-15`. I did not check whether they are already applied. If one is still pending, apply it before this patch and use `substitutions.py check` rather than the baseline hashes.

## What the focused probe should inspect

1. **No failures** on the ten `patch` rows of `probe-strings.tsv`. The two `comparison_only` rows are optional and are not in the patch.
2. **Separation on all 11 on-screen `z_0`.** `z` and `0` should be separate components at levels 0.25, 0.5 and 0.75. On glyphs02, ten of the eleven were one component at all three levels, and row 717 (`z_0^2`) split only at 0.75. `checks/touch_test_any_pages.py PAGES_DIR page:x0,y0,x1,y1` is the frozen review's test with the pages folder as an argument. `checks/outputs/touch_adapter_control.txt` shows it reproducing the frozen results on the old pages.
3. **Same subscript height.** The `0` should sit where it did, with only `x` moved.
4. **Whether 3 mu reads as detached,** against `z_k` and `c_0` nearby. If it does, the 2 mu comparison rows are one substitution away.
5. **Row 717 stagger,** the `0` about 4 px past the `2`.
6. **Row 723,** with bar, minus and `3` separate and the bars tight like `|z^5`.
7. **Scene frames in chapters 06, 08, 11, 12 and 15,** chiefly the chapter 06 line holding QUOTIENT_TEX and SETTLES_TEX.

**Rows and crops.** Rows are keyed by source in byte order, so very little renumbers. Against the glyphs02 registry, old 716 becomes 717, old 717 becomes 718, and `z_k` moves from 718 to 716. The other changed strings keep their numbers, assuming no other string has changed since glyphs02. See `checks/outputs/registry_rows.txt`. The frozen crop boxes will not carry over for ink right of a `z_0`, because the widths change.

## Limits

- I did not compile, run the probe, render or edit canonical. Every effect above is a prediction until your probe shows it.
- The widths come from glyphs02 ink extents, and the row map from the glyphs02 registry.
- **This is not independent.** I wrote part of this crate and I found these defects, so nobody else has checked this patch yet.
- **Scope.** F2, F4 and F5 are untouched, as you asked. The engine and cue words are untouched. Narration stays ElevenLabs Oliver and nothing here touches it.

Python scripts run under `python3 -I -B`, except `touch_test_any_pages.py`, which needs PIL and so runs under `python3 -B`.
