# cc2 plate-fade investigation 246 (bead .39)

Bounded investigation of review241's M1 (colour plates stay full strength through authored
fades). It covers the cause, the affected chapters, one minimal downstream candidate and a
focused native probe. This is a source investigation only: nothing was compiled, built,
rendered, run, committed or deleted. No canonical or shared source, Beads entry, TODO or memory
file was edited. Narration stays ElevenLabs Frederick Surrey. I make no claim about listening,
runtime, pixels or final AV.

Paths below are relative to this folder (`production-repair17/`) unless they start with
`crates/`, which means franken_manim at pin `a399d626c43ecc852a5c795288979f52b446bc5c`.
`P` is `production-source-preserved214/workspace/explainer_corpus/complex_analysis_visual`.

## Deliverables

| File | sha256 | Install as |
|---|---|---|
| `cc2-plate-fade-candidate246-draw.rs` | `aab61199e7a0dea9c5482ee2aa127f94c0c5bb06e2edbb5202350db964a1ea56` | `src/chapters/draw.rs` |
| `cc2-plate-fade-probe246.rs` | `c0c56084be04c9ff3605e0cb1d05fe989510c35d043ce6215688e72be909155f` | `src/chapters/plate_fade_probe.rs` |

The candidate is production19's `P/src/chapters/draw.rs` (sha256 `cd394e5a…`) with additions
only. `diff` shows three hunks (around production lines 266, 282 and 1825) and no removed
production line. It is rustfmt-clean with the repo's `rustfmt.toml` (edition 2024, width 120).
Formatting was checked together with the probe under the install names in a scratch copy, so
both files parse. Neither file has been compiled.

## Verdict

- **Cause (source-proven):** an opacity-lane mismatch. Production plates are raster image
  quads. Their visible opacity is a separate one-lane `opacity` field. The engine's
  `fade_in`/`fade_out` write only the alpha of `*rgba` fields, so they never change a plate.
- **Review241's attribution needs a correction.** Production does not draw plates with
  RowStrips: `PLATE = PLATE_RASTER` at `P/src/chapters/draw.rs:636`. The RowStrips/Blend styles
  use `stroke_rgba` and would fade, but the film does not use them.
- **Effect:**
  - On a fade in, the plate appears at full strength on the first fade frame.
  - On a fade out, it stays full on every frame, the last one included, and disappears only when
    the remover takes it off after the play.
  - Vector overlays in the same play fade normally.
  - This matches review241's measurements and root's fade-root-observation251 (final10 frames
    3737/3767/7441 at full strength).
- **Repair:** downstream only and schedule-neutral. `fade_in_all`/`fade_out_all` (draw.rs) add,
  inside the same `play`, a ramp of each image record's `opacity` lane. The ramp uses the fade's
  own run time and rate. It adds no play and no frame, and moves no cue.

## Source evidence

### Production plate path

All in `P/src/chapters/draw.rs`:

- `PLATE_RASTER` (`:629`) and `PLATE = PLATE_RASTER` (`:636`).
- `Picture::build`, raster branch:
  - `ImageMobject::from_resource(resource)` (`:683`), added to the arena as one image mob;
  - `Painted::Raster { mobs: [mob], .. }` (`:704`);
  - `group(stage, &[mob])` wraps it in an empty VMobject parent (`group`, `:261`).
- `Picture::rows()` returns that image mob (`:727–729`).
- `recolor` replaces only the texture: `arena.set_image_resource(mobs[0], Some(resource))`
  (`:744`).
- Painter:
  - `show` (`:1007`): `add_to_scene(anchor)`, then `fade_in_all(&[group], rt)`.
  - `leave` (`:1028`): clear and remove the anchor, then `fade_out_all(&[group], rt)`.
- `fade_in_all` (`:269`) and `fade_out_all` (`:279`) are bare `fade_in`/`fade_out` per mob in one
  `play_prepared`.
- Test gap: `fades_keep_every_vertex_colour` (`:1630`) exercises only `PLATE_NOW`, `PLATE_TILE`
  and `PLATE_BLEND` (`:1631`), never the production `PLATE_RASTER`. It reads `stroke_rgba`, which
  the raster quad does not have.

### Engine at the pin

- `crates/fmn-library/src/image.rs:301,320`: an image mobject's record schema is
  `("point",3),("im_coords",2),("opacity",1)`, and its six records get `opacity` written from
  `ImageMobject::opacity` (1.0).
- `crates/fmn-mobject/src/align.rs:192` `set_family_opacity`: writes the alpha lane of every
  field whose name ends in `rgba`, across the family. `set_family_opacity_zero` (`:250`) calls it
  with 0.
- `crates/fmn-anim/src/fading.rs`:
  - `fade_in` (`:52`) is a Transform whose start copy is prepared with `opacity_zero`;
  - `fade_out` (`:80`) is a remover with `final_alpha_value = 0.0` (`:95`).
- `crates/fmn-anim/src/transform.rs:291,358`: both preparations call `set_family_opacity_zero`
  (rgba only). The image's `opacity` lane is equal in the start and target copies, so the
  Transform interpolates it from 1 to 1.
- The renderer honours the lane:
  - `crates/fmn-render/src/retained.rs` `image_mesh` reads `opacity` for each of the six records
    into `SurfaceVertex::textured`;
  - `crates/fmn-render/src/three_d.rs:4075/4109/4112` multiplies it into the colour's alpha.
  - So a downstream write to the lane does reach pixels.
- `crates/fmn-mobject/src/record.rs:667` `write_scalar_lanes` (behind `Stage::set_scalar_lanes`)
  writes a one-lane field through `write_range`, which advances the field's revision.
- `crates/fmn-mobject/src/stage.rs:1024` `set_image_resource` replaces the resource and its
  revision only. The Painter's recolouring updater therefore cannot reset the ramp.
- Order and finish semantics:
  - `crates/fmn-anim/src/frame.rs:520–562` `prepare_play_animation` interpolates the play's
    animations in list order on every frame, before scene updaters.
  - `crates/fmn-anim/src/frame.rs:298` `finish_animations` runs `finish` and then
    `clean_up_from_scene` (remover) per animation in list order, then one zero-dt updater pass.
  - `crates/fmn-anim/src/animation.rs:786` `finish` calls `interpolate(final_alpha_value)`.
  - `crates/fmn-anim/src/update.rs:59,100` `UpdateFromFunc::new_alpha`: the closure receives
    `rate_func(alpha)`, with smooth as the default config.
  - `crates/fmn-anim/src/clock.rs` `FrameSample`: frames are 1-based (no alpha-0 frame), and the
    last frame has alpha exactly 1.

### Production engine is the pin plus an overlay that does not touch this path

The build19 frozen engine hashes are recorded in
`production-overlays-preserved220/qualification01/production-repair17/build19-inputs.json`.

- These files are byte-identical to the pin: `fading.rs`, `transform.rs`, `update.rs`,
  `animation.rs`, `frame.rs`, `image.rs`, `align.rs`, `stage.rs`, `three_d.rs`, fmn `lib.rs` and
  scene `runtime.rs`.
- Three files differ: `retained.rs` (`c2124bcc…`), `plan.rs` and fmn `rendering.rs`. I read a
  local copy with matching hashes. Its `retained.rs` differs from the pin by 25 lines, all about
  plan limits (`RenderPlanLimits`, `set_plan_limits`, `sync_with_limits`). None touch `opacity`,
  images or textured surfaces; `image_mesh` sits at overlay line 810.
- `P/src/main.rs` `declare_plan_budget` sets `opts.camera` (the retained camera route that native
  images need) and `opts.plan_limits`. The probe uses it.

## Affected chapters (registered in `P/src/chapters/mod.rs`)

Every compiled engine fade goes through draw.rs `fade_in_all`/`fade_out_all`. In the registered
files' non-test code, `fade_in(` and `fade_out(` occur once each, both in draw.rs, and the kit's
`fade_all(` never occurs. The unregistered drafts beside them (cauchy, argument, bookends,
continuation, domain_color, derivative, integral) are not compiled. So these are all the plate
fade sites:

| Chapter | Plate fade sites (file:line) |
|---|---|
| 01_the_bump | `the_bump.rs:367` preview `portrait`: in at `:330` (first `fade_in_all` chunk, 0.5), out at `:349` (`fade_out_all(&all, 0.6)`) |
| 04_reading_color | `reading_color.rs:219` Painter: show `:221` (1.2), leave `:438` (0.5) |
| 05_gallery | `gallery.rs`: painter `:397`/`:483`; exp `:487`/`:554`; trig `:558`/`:672`; gamma `:676`/`:740`; zeta `:747`/`:886`; strip `:797`/`:887`; nine tiles show `:893` (0.25 each); eight tile groups out together in `fade_out_all(&leaving, TILE_BEATS[1])`; zoom show `TILE_BEATS[2]`; `painters[0].leave(0.5)`, `zoom.leave(0.5)` at the end of the tile cue |
| 07_one_point | `one_point.rs:419` show (1.0), `:585` leave (0.5) |
| 09_boundary | `boundary.rs:474` show (1.0), `:1398` leave (0.5) |
| 10_rigidity | `rigidity.rs:428`/`:507` (1.0/0.5); `:533`/`:619` (1.0/0.5). The last is the chapter end, review241's frames 7440/7441 |
| 11_counting | `counting.rs:196` show (0.8), `:519` leave (0.4, under the identical Turning field, so unseen either way); Turning field (`:653`, a plate added by `add_to_scene` by design) out at `:567` (`fade_out_all(&acted, 0.3)`) |
| 12_residues | `residues.rs:282`/`:324` (0.6/0.4, per function in the loop); bump `:329`/`:362` (0.8/0.5) |
| 14_continuation | `continuing.rs:275`/`:321` (1.0/0.5); lac `:686`/`:738` (1.0/0.4); gamma (blank) `:747`/`:799` (0.1/0.5) |

These have no plates (no Painter, portrait, Picture or image): 02_turning_numbers,
03_two_planes, 06_one_derivative, 08_loop_sums, 13_real_integral, 15_closing. thumbnail.rs is a
still and is not affected.

The Painter crossfades (`Painter::to`, `enhance_to`) recolour the texture and do not involve
opacity, so they are unaffected and unchanged.

## The candidate

All changes are in `cc2-plate-fade-candidate246-draw.rs`:

- `image_records(stage, m)` (`:274`): the members of `m`'s family that carry an image resource.
  For a Picture or Painter group, that is its one image quad.
- `plate_fade(owner, images, rising)` (`:287`):
  - an `fmn::animation::UpdateFromFunc::new_alpha` on the same owner mob;
  - its closure writes `opacity = alpha` (in) or `1 − alpha` (out), clamped to [0, 1], to every
    listed image via `set_scalar_lanes(k, "opacity", shown, false)`;
  - config: `AnimConfig::default()` (smooth, as the fades), `name: "PlateFade"`, and
    `final_alpha_value` 1 for in, 0 for out.
- `fade_in_all` (`:301`) and `fade_out_all` (`:315`): after each mob's fade, if the mob has image
  records, also `prepare(plate_fade(..).rt(rt))` into the same play. Everything else is
  unchanged.

Why this is correct and preserves schedules:

- **Same play, same `rt`, same rate.** A segment's frame count comes from the longest run time
  in the play, so it does not change. No play is added, so cue waits and stage time are
  untouched. The ramp is listed after its fade, so it writes last on every frame.
- **Fade in.**
  - Frame k has opacity `smooth(k/(fps·rt))`. For 1 s at 60 fps, the first frame is about
    5·10⁻⁵, i.e. effectively 0.
  - The last frame is exactly 1, and `finish` leaves it at 1.
- **Fade out.**
  - The last frame (alpha 1) has opacity 0.
  - Then `finish` with `final_alpha_value = 0` gives the lane back at 1 (root's audit confirms
    `UpdateFromFunc::finish` calls `interpolate(final_alpha_value)`).
  - This happens in the same `finish_animations` pass in which the earlier-listed `fade_out`'s
    remover takes the group off the scene. No frame is captured in between (`frame.rs:298`), and
    the plate is back at 1 for any later show, exactly as `fade_out` restores vector alphas.
- **Full-strength frames are unchanged.** At hold the lane is exactly 1.0, as in production19.
  Pixels are unchanged by construction; the probe checks this.
- **Covers every site in the table.** Both Painter methods and every direct `fade_in_all` /
  `fade_out_all` of plate groups (the_bump preview, gallery tiles, Turning field) pass through
  the two helpers.

## Tests

### Source and lane tests (in the candidate's `tests` module, no render)

1. `a_raster_plate_fades_through_its_opacity_lane` (`:1875`), at 10 fps headless (the existing
   `scene` harness), with a `PLATE` Picture.
   - `image_records(group) == [image]`.
   - A frame-only updater (`dt > 0`, plate in `roots()`) logs the lane on all 20 fade frames.
   - Each frame's six records are equal, and frame k equals `smooth(k/10)` in and
     `1 − smooth(k/10)` out, within 1e-6.
   - Exactly 10 + 10 frames are logged and none after removal.
   - The lane is given back at 1, and the group is off the scene.
2. `a_plate_fade_takes_the_frames_a_vector_fade_takes` (`:1936`), at 60 fps (the film's rate),
   for `rt` in {0.1, 0.25, 0.3, 0.4, 0.5, 0.6, 0.8, 1.0, 1.2}: plate-in, plate-out and vector-out
   frame counts all equal vector-in.
3. `every_engine_fade_in_the_film_carries_the_plate_ramp` (`:1980`):
   - over the 17 registered files (non-test code), `fade_in(` and `fade_out(` appear only in
     draw.rs, once each, and `fade_all(` never;
   - draw.rs carries exactly one rising and one falling `plate_fade(m, images, …).rt(rt)`.

The existing `fades_keep_every_vertex_colour` (vector styles) is kept unchanged.

### Focused native probe (renders; ignored by default)

`cc2-plate-fade-probe246.rs` is wired by `#[cfg(test)] #[path = "plate_fade_probe.rs"] mod
plate_fade_probe;` at the end of the candidate (`:2016`). Run it on purpose:

```
cargo test --release --bin complex_analysis_visual plate_fade_probe -- --ignored --nocapture
```

The render:

- `fmn::render_with_fs` into `fmn::platform::fs::VirtualFs`;
- Y4M, 640×360 at 10 fps;
- the film's camera route and plan limits, via `crate::declare_plan_budget`;
- 95 frames.

Phases (each fade 1 s = 10 frames, each hold 0.5 s = 5 frames):

- **A:** a static light `PLATE` Picture (left half) and a vector square (right half), faded
  together by the candidate `fade_in_all` / `fade_out_all`.
- **B:** an updater-backed Painter (`Func::Identity`) through `Painter::show` / `Painter::leave`.
- **C:** the same Picture as A through production19's fade bodies, kept verbatim in the probe.
  This is the defect control.

Measurement: per-pixel luma share against the bare background frame and each phase's own
full-strength hold, over pixels that differ by at least 24 luma steps (at least 2000 pixels
required). The assertions:

- **Timeline:**
  - each of A, B and C takes [10, 5, 10] frames (in, hold, out), so the candidate equals
    production19's C;
  - 95 clock frames, 95 Y4M frames, and `report.artifact.frame_count == 95`.
- **Opacity 0, mid and 1, for the A picture, the A square (control) and the B painter**, in and
  out:
  - first fade frame ≤ 0.05 (in) or ≥ 0.95 (out);
  - frame k = 5 (opacity 0.5) within [0.25, 0.75];
  - last fade frame ≥ 0.99 (in) or ≤ 0.01 (out);
  - monotone throughout.
- **End-of-fade frames, for A and B:**
  - the last fade-in frame and the first hold frame are byte-identical (no dip), and the hold is
    constant;
  - the last fade-out frame shows ≤ 0.01;
  - the first frame after the fade out, and the fifth, are byte-identical to the bare background.
    This rules out a full-bright flash from the lane being given back at 1.
- **Defect control:** every C fade frame, in and out, shows ≥ 0.95, and C is gone on the frame
  after its remover. If this fails, the probe cannot see the defect and its other passes mean
  nothing.
- **Full strength unchanged:** the left half of A's hold equals C's hold pixel for pixel.
- **Printed only (`--nocapture`):** every share series, and picture minus square at k = 5. The
  vector and textured paths may blend differently, so that difference is a diagnostic, not an
  assertion.

Only if every unit test and the probe pass should the affected chapters be re-rendered. Then
read the real film at review241's frames (3737–3800, 6242–6340, 7412–7441, the 10→11 join) and
root's 3737/3767/7441. Neither a build nor a probe pass is a verdict on the film's pictures.

## Explicit untested limits

- **Uncompiled.** API names and signatures come from reading the pin's source and the
  candidate's own call shapes; the compiler is the arbiter. They are:
  - `fmn::animation::UpdateFromFunc` path and `new_alpha` closure type
    `(&mut Arena, Mob, f64)`, and `with_config`;
  - `Stage::family`, `Entry::image_resource`, `set_scalar_lanes` and `roots()` through the
    facade's deref;
  - `stage.scene().time().frames()` returning `i64`;
  - updater `dt` as `f64`;
  - `RenderOptions` fields `config.camera.resolution` / `fps` with literals;
  - `VirtualFs::read` via `FileSystem`;
  - `report.artifact.frame_count` against a literal;
  - `fmn::core::rate::smooth` being the same function as `RateFunc::default()`, which unit test
    1 checks to 1e-6.
- **No Clippy run.** I expect it to pass but have not proved it. Two risks: `useless_vec` on the
  probe's `&vec![vec![LIGHT; 40]; 30]`, and pedantic casts.
- **Not rendered.** Unknowns until the probe and a real chapter render run:
  - the probe's luma bands assume a dark ground and a light 0.85 plate in 4:2:0 Y4M from the
    certified CPU camera path;
  - the colour space in which textured surfaces blend (hence the wide [0.25, 0.75] mid band);
  - whether Y4M frame k maps to clock frame k + 1 (asserted, so a mismatch fails loudly rather
    than mis-measuring).
- **Performance.** `UpdateFromFunc` keeps the engine's "Unclassified" signature, and per
  `update.rs:1–5` its segments are demoted and never frame-parallelize. Plate fade segments may
  therefore render more slowly. Frame bits and counts should not change; this is unmeasured.
- **Shared plate families.** A plate attached under some other root that is not itself faded
  would keep showing. That root would also see the lane given back at 1 at the end of a fade out.
  I found no such structure in production: Picture and Painter groups are faded as roots, and
  `group()` parents are new. This is not proven for every runtime path.
- **Visual change in production, by intent.** Plates now actually fade with their overlays. In
  11_counting the painter fades under the identical Turning field (unseen). Gallery tiles now
  take their 0.25 s and TILE_BEATS fades. Whether any such fade reads worse than a cut is a
  picture decision for root after re-render.
- **Not checked:** listening, fine sync, joined motion, captions, platform captions and
  independent final AV. They all remain open.

## Hygiene

- I wrote only the three `cc2-plate-fade-*246*` files in this folder.
- I read `bead-fade-review246.json` (root's) only to see whether it was relevant; I did not edit
  it.
- Private-path grep result and the final sha256 of this file are in the pane report.
