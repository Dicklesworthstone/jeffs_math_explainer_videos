# cc2 plate-fade correction 260 (bead .39)

This corrects the source gate in candidate246 after CC3's integration finding. Nothing was
compiled, built, rendered, run, committed or deleted. The frozen 246 files are untouched. Paths
are relative to this folder. Narration stays ElevenLabs Frederick Surrey, and I make no claim
about listening, pixels or final AV.

## Files

| File | sha256 |
|---|---|
| `cc2-plate-fade-candidate260-draw.rs` (install as `src/chapters/draw.rs`) | `923ad2efb5f8524e6324f0e14a4946e10026cde902b99c1c7974524255be1064` |
| `cc2-plate-fade-probe246.rs`, unchanged (install as `src/chapters/plate_fade_probe.rs`) | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` |

The frozen `cc2-plate-fade-candidate246-draw.rs` (`aab61199…`) and
`cc2-plate-fade-investigation246.md` are untouched.

## The defect (CC3, confirmed)

The 246 gate `every_engine_fade_in_the_film_carries_the_plate_ramp` allowed engine `fade_in(` /
`fade_out(` only in draw.rs. The build256 tree (`review-repair-recipe256.json`, all eight recipe
hashes verified) contains three reviewed engine fades outside draw.rs:

- **`loop-sums-review-repair238.rs:646,648`:**
  - `fade_out(stage.arena_mut(), centre_name, …)` and `fade_out(…, diagonal_name, …)`;
  - the targets are `kit.tex(stage, "c", …)` (`:607`) and `kit.tex(stage, "D_n", …)` (`:618`),
    each bound once.
- **`one-derivative-review-repair238.rs:200,204`, the `hand_over` helper:**
  - it fades its `leaving` out and its `coming` in, generic over `m`;
  - it is called once (`:543`): `hand_over(stage, &[scale_note], &zoomed, 0.3)`;
  - `scale_note = kit.text(…)` (`:272`) and `zoomed` = two `kit.text_left(…)` (`:537–540`), each
    bound once.

The 246 gate therefore fails on exactly `loop_sums` and `one_derivative` (Python mirror below).
Neither file has a plate word outside comments: Painter, Picture, portrait, PLATE*,
ImageMobject, ImageResource, image_resource, RowStrips, blended_rows or plate_colors. Neither
imports a plate builder from draw. So the three fades are vector-only, and the plate ramp is not
needed there.

## The correction (test module only)

- **Non-test code is unchanged.** Lines 1–1439 of candidate260 are byte-identical to
  candidate246 (`1ad7b87a…` for both). So the fix, the fade helpers and every frame and cue
  schedule are unchanged.
- **Removed lines.** Compared with production19 draw.rs, the file still only adds lines. The only
  change from 246 is inside `mod tests`: the old gate (diff hunks at 246's lines 1976–2008) is
  replaced.
- **`VECTOR_ONLY_FADES` (`:1985`), typed by the test-only alias `ReviewedFades` (`:1978`):** an explicit list of the two files, not a blanket
  exemption. For each file it gives:
  - the exact engine fade lines, compared as a sorted multiset against every
    `fade_in(` / `fade_out(` / `fade_all(` line in the file's non-test code;
  - exact counts for the target bindings above, including the `hand_over(` count of 2 (its
    definition and one call) and the full `zoomed` literal.
- **`PLATE_WORDS` (`:2022`):** none may appear outside comments in a listed file.
- **`fade_gate` (`:2042`):**
  - draw.rs: exactly its two ramped fades, plus both `plate_fade(m, images, …).rt(rt)` once;
  - a listed file: exactly its listed fades, pins and no plate word;
  - every other registered file: no engine fade at all.
- **`every_engine_fade_in_the_film_carries_the_plate_ramp` (`:2098`):** runs `fade_gate` over all
  17 registered files and asserts every listed file is registered.
- **`control_the_fade_gate_refuses_unreviewed_fades` (`:2127`), on the real files.** The gate
  accepts real loop_sums, one_derivative and draw. It refuses:
  - a `painter.group` fade in an unlisted file;
  - the same fade added to a listed file;
  - a `Painter::build` in a listed file;
  - a target no longer built by `kit.text`;
  - a second binding of `centre_name`;
  - draw without its falling ramp.

Any change to the listed fade lines or targets fails the gate and needs a fresh review rather
than a wider list.

## Desk checks (not compilation)

- A Python mirror of `fade_gate` ran over the composed build256 tree: preserved214, plus the
  seven recipe replacements, with candidate260 as draw.rs. All 17 files pass and all six
  controls are refused. With the 246 rule, only loop_sums and one_derivative fail, which
  reproduces CC3.
- rustfmt (repo `rustfmt.toml`) is clean for candidate260 together with the unchanged probe
  under the install names, and the pinned string literals survived formatting.

## Untested limits

- **Uncompiled and Clippy not run.** The new code uses only std, plus `fade_gate` returning
  `Result<(), String>`. Root's `type_complexity` question is handled without allowing the lint:
  - by hand count of Clippy's scoring (default threshold 250), the first draft's const type
    `&[(&str, &[&str], &[(&str, usize)])]` scores about 306 and would fail under `-D warnings`;
  - the test-only alias `ReviewedFades` brings the const's type to about 31, and Clippy does not
    score alias definitions;
  - the other new types score at most about 150.
  - A remaining risk is that the long `assert!` / `format!` lines in the control test could trip
    some other lint.
- **The gate is a source scan.** It reads text, not types:
  - it trusts that `kit.tex`, `kit.text` and `kit.text_left` build vector text (explainer_kit),
    which was read but not re-proven here;
  - `//` stripping is per line, so a `//` inside a string literal would hide the rest of that
    line from the plate-word scan, though not from the fade-line or pin checks;
  - fade lines in comments still count, which is the conservative choice.
- **The probe is still unrun.** It is the same probe246, and its untested limits stand as in
  investigation246. A build or gate pass is not a verdict on any picture.

## For CC3

Please review candidate260 at `923ad2ef…`, with the probe at `c0c56084…`. This is the final hash.
The earlier draft `c182e336…` differs only by the `ReviewedFades` alias (4 lines added, 1 line
changed). The new control adds one normal test, so build262 expects 233 tests passed, plus the
one ignored pixel probe run explicitly. Root will use a new build name.
