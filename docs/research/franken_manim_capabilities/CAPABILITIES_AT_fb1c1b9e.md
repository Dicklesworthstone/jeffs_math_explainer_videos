> Surveyed read-only at franken_manim HEAD `fb1c1b9e` on 2026-10-09. The LIVE/PARTIAL/DESIGN/BROKEN classifications below must be re-verified against the code before relying on them after later commits.

I surveyed franken_manim at **HEAD `fb1c1b9e`** (main, 2026-10-09 19:02 −0400) without modifying anything; nothing was built or rendered. The short answer: the native facade can carry an ambitious 4K explainer today. The main limits are that 3D and camera moves run on a slower CPU-only route, frames of one render are captured one at a time, and a few math glyphs still draw wrong.

# franken_manim native-facade capability report (HEAD `fb1c1b9e`)

**How each claim was checked.** I read the skill reference docs and then checked them against the code at HEAD. Five parallel read-only passes covered: 3D and camera, fields and paint, text and TeX, plots/graphs/SVG, and the workflow-named features. I re-checked their key claims in the code myself and corrected one: `AnimatedStreamLines` is native, not Python-only (`crates/fmn-library/src/fields.rs:1650-1820`). Paths below are relative to the franken_manim repo root.

**Proof levels.**
- **R**: a test or e2e run renders it through `fmn::render`, `render_with_fs` or `render_camera_with_fs`.
- **E**: proven only at renderer or library level (same code, but not through the facade), or through the CLI or portal.
- **T**: the type exists, but nothing renders it.

## 0. What changed since the skill docs were verified (29233ca6 → fb1c1b9e, 51 commits)

**The authoring API did not change.**
- `git diff --stat 29233ca6 HEAD` touches nothing in `fmn-anim`, `fmn-mobject`, `fmn-scene`, `fmn-render` or `fmn-output`.
- In `fmn-library` only a test comment changed (`tex.rs:1044`).
- In `fmn/src` only doc links (`rendering.rs:104,277`) and one typesetting counter (`lib.rs:1309`) changed.
- So NATIVE-API.md's surface is valid at HEAD.

**TeX output changed visibly.** The franken_markdown pin moved to `bd00a247` (`SUITE.lock:33`) in three steps:

| Commit | Upstream rev | Visible effect |
|---|---|---|
| ad38cb64 | e4e8e2ee | TeX null delimiters around `\frac` (fractions about 0.24 em wider); `\vdots`/`\ddots`; a trailing `\\` no longer adds an empty row |
| 50395b40 | 798de11b | **Multi-row `Tex` is now `align*`.** A row without `&` is right-aligned, and `a &= b \\ &= c` aligns on `=`. Stacked one-per-line formulas now sit flush right. |
| 385f1742 | bd00a247 | `\vec` is a small arrow above the letter (it used to strike through); `\big` family at 1.2/1.8/2.4/3.0 em; array struts make matrices and `cases` taller |

The TeX box-oracle floors are now 44/111 formulas within 5% of real LaTeX and 73/111 within 10% (`crates/fmn-library/tests/tex_reference_boxes.rs:20,22`).

**Other changes:**
- **macOS CPU topology:** 90e2f256 adds `HardwareTopology::detect_macos` (`fmn-platform/src/topology.rs:813`), tested only on a recorded fixture. `fmn::render` still uses the flat fallback off Linux (`crates/fmn/src/rendering.rs:714-718`). The live reader is bead fm-macos-live-sysctl-reader-akab (open).
- **Arc fix:** ea463832 (tip-aware Arc angles) is portal-only.

**Still relevant from the earlier window (7ef2eac2…c155fe02):**
- 8ead3151: 24 h ffmpeg timeout in the facade.
- 8d5bcd06: `VideoQuality` (CRF, preset, tune, bitrate).
- 0eec605f: frame-concurrent PNG-sequence encoding.
- cdfb9221: the 2.2× 4K render speedup.
- d8d58fb9 + 33e2ba17: `Axes` placement with `c2p` following.
- 487690b2: the Hoeffding's D explainer as an in-repo example.

Beads fm-video-encode-quality-knob-fm85 and fm-render-ffmpeg-job-limits-laeo still read "open" but are **stale** for the facade.

## 1. The route model (this decides everything)

**Default route** (`RenderOptions::camera = None`, `rendering.rs:257`): the fast retained 2D vector route.
- It refuses any non-vector draw item with `CameraRequired` (`crates/fmn-render/src/vector_frame.rs:56-65`). That means Surface, DotCloud/GlowDot, **ImageMobject**, and textured meshes.
- It also refuses vector content with any z ≠ 0, or with 3D uniforms (fixed-in-frame, shading, depth test, clip planes, scale-stroke-with-zoom), via `CameraProjectionRequired` (`fmn-render/src/engine.rs:1523-1577`).
- **Consequence:** a "card flip" (rotating a 2D object about RIGHT or UP) aborts the render on this route.

**Fixed camera** (`opts.camera = Some(opts.camera_config()?)`, `rendering.rs:342`):
- The **whole** render moves to the camera route, forced to the certified CPU engine (`rendering.rs:735-739`).
- It has no tile cache and clones the render plan every frame.
- On planar 1080p content it measured 2.38–2.55× slower than the retained route (`docs/ratchet/portal_planar_route_ab.md`; portal runs). **No 4K camera-route timing exists.**

**Animated camera** (`fmn::rendering::render_camera(factory, opts)`, `rendering/camera_animation.rs:35`):
- It does **not** fill in ffmpeg. Only `render()` does (`rendering.rs:543-546`).
- Set `opts.ffmpeg = Some(FfmpegCapability::host())` yourself, or MP4 is refused (`rendering.rs:686-690`).
- `render_bundle` behaves the same way.

**Plan implication:** keep 2D chapters on the retained route; render 3D and camera-move segments as separate scenes; join everything by stream copy.

## 2. Feature-by-feature

### 2.1 3D (item 1)

| Feature | Status | API / evidence |
|---|---|---|
| 3D camera config (θ/φ/γ, FOV, zoom, center, light) | **LIVE (R)** | `fmn::rendering::{CameraConfig, CameraFrame}` (`rendering.rs:68`). `frame.set_euler_angles(Some(θ), Some(φ), Some(γ))`, `increment_euler_angles`, `rotate`, `set_field_of_view` in (0, π), `set_center`, `set_width` (zoom), `light_source_position`, `samples` (edge AA: 0–1 analytic, 4+ = 4×4) (`fmn-render/src/camera.rs:233-426,515-530,724-729`). There is no orthographic mode (`camera.rs:762-778`). Proof: `crates/fmn/tests/native_camera.rs:42` |
| **Moving or orbiting the camera during a render** | **LIVE (R to PNG/GIF/Y4M; MP4 by construction, untested)** | `fmn::scene::CameraRig` is 12 scalar trackers: center xyz, width, quaternion, FOV, light xyz (`fmn-scene/src/camera_rig.rs:22-137`). `rig.animate_to(scene, &target) -> Transform` (`:78`). The test `camera_orbit_zoom_fov_and_light_match_serial_snapshots_at_every_window` (`crates/fmn/tests/native_camera_pipeline.rs:147`) animates orbit, pan, zoom, FOV and light together. The camera path shares `RenderSink` with MP4. Example: `crates/fmn/examples/camera_pipeline_orbit.rs` |
| Camera plus object animation in one play | **LIVE (T)** | The rig is sampled after updaters, from the same capture as the geometry (`camera_animation.rs:17-19`). Use `play_prepared_with(vec![Box::new(cam), prepared_anim], …)` |
| Ambient rotation (`begin_ambient_camera_rotation`) | **PARTIAL** | `fmn::scene::ThreeDScene::add_ambient_rotation` exists (`fmn-scene/src/runtime.rs:2313`) but owns its own Scene, so it is unreachable from `render`/`render_camera`. Working pattern: a dt-updater on `rig.root()` that rewrites the four quaternion trackers (the updater-on-rig pattern is tested at `native_camera_pipeline.rs:199` for a pan). `animate_to` uses normalized linear quaternion interpolation, so split rotations of π or more into several segments |
| Surface / ParametricSurface | **LIVE (R: Sphere)** | `fmn::library::ParametricSurface::new(\|u,v\| -> Vec3)` with `.u_range(a,b) .v_range(a,b) .resolution(nu,nv) .color() .opacity() .shading([refl,gloss,shadow]) .depth_test()` (`solids.rs:534-605,950-980`); `Surface::color_by_uv_function` (Gouraud shading). Proof: `crates/fmn/tests/native_render.rs:447`. Creation animation: `fmn::animation::show_surface_creation(m, res, axis)` (`fmn-anim/src/creation.rs:299`). Non-finite samples, e.g. at poles, are refused (fm-8lci, open P3) |
| Solids | **LIVE types; Cube and Sphere proven (R)** | `Sphere`, `Torus`, `Cylinder`, `Cone`, `Line3D`, `Disk3D`, `Square3D`, `Cube`, `Prism`, `SGroup`, `VCube`, `VPrism`, `Dodecahedron`, `Prismify`, `SurfaceMesh` (wireframe) (`solids.rs:1012-2333`). There is no `Arrow3D` or `Dot3D`: compose `Line3D` + `Cone`, or a small `Sphere` |
| ThreeDAxes | **PARTIAL** | `ThreeDAxes::new()…build(&book)` plus `c2p` works (`planes.rs:240,404,481`). `get_graph`, `get_parametric_surface`, labels and placement live in `planes/placement.rs`, which is **not declared or compiled** (only `coords.rs:50` has `mod placement`). Workaround: `ParametricSurface::new(move \|u,v\| axes.c2p(&[u,v,f(u,v)]))` |
| Lighting and shading | **LIVE** | One movable point light (animatable via `rig.light()`), plus a per-object `[reflectiveness, gloss, shadow]` (`stage.set_shading`, `fmn-mobject/src/positional.rs:1190`). No cast shadows, no multiple lights |
| Depth sorting | **LIVE, with caveats** | A per-fragment z-buffer that each draw opts into. Surfaces default to `depth_test: true`; VMobjects default to off. Turn it on per member via `uniforms_mut` (`fmn-mobject/src/uniforms.rs:89-101`). Back-to-front face sorting is not ported (`solids.rs:85-87`), so self-overlapping translucent surfaces depend on mesh order |
| HUD labels over 3D (`fix_in_frame`) | **LIVE (E)** | `stage.set_fixed_in_frame(m, 1.0, true)` (`positional.rs:1211`). Fixed objects use the default 8 × 14.22 frame regardless of zoom (`camera.rs:504-510`). Put them last in draw order |
| Textured surfaces (day/night globe) | **BROKEN builder, LIVE renderer** | `TexturedSurface::new(surface, path)` **drops the path**: `From<TexturedSurface> for Mobject` sets `image: None` (`solids.rs:2725-2752`), so the render fails. Workaround: convert to `Mobject`, then set `m.image = Some(ImageResource::rgba8(..)?.with_dark_image(..)?)` (`fmn-mobject/src/mobject.rs:194,275`) or call `stage.set_image_resource` (`stage.rs:954`). Renderer proof: `fmn-render/tests/texture_pairs.rs`. **No bead filed** |
| OBJ models | **PARTIAL (E)** | `ThreeDModel::from_obj(bytes)` gives a plain-colored lit mesh (`obj_model.rs:527-580`). MTL and textures go through `ObjDocument::parse` + `parse_mtl` + `.to_mobject()` (`obj_materials.rs:144,249,442`), with structural tests only |
| DotCloud / GlowDot | **LIVE on the camera route (E)** | `fmn::library::{DotCloud, glow_dot, glow_dots, true_dot}` with `.with_radius .with_glow_factor .colored .make_3d` (`pointcloud.rs:506-772`); real `(1−r)^glow` falloff (`fmn-render/src/three_d.rs:225-232`). Proven only through the CLI builtin `dot_cloud_depth.v1` |

**3D performance:** CPU-only. Metal is refused by `fmn::render` (`rendering/pipeline.rs:367-373`; fm-metal-route-native-render-pank, open P2, blocked on fm-sq8.6). PG-6 (`lifecycle.pg6_peak_gallery_4k_3d.v1`) measures memory, not time, and bypasses `fmn::render`.

**Sketch: animated 4K orbit to MP4** (each call verified to exist; the program as a whole is not compiled):
```rust
use fmn::prelude::*;
use fmn::library::ParametricSurface;
use fmn::rendering::{render_camera, CameraConfig};
use fmn::scene::CameraRig;
struct Orbit { rig: CameraRig, target: CameraConfig }
impl SceneConstruct for Orbit {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        stage.add(ParametricSurface::new(|u, v| [u, v, 0.4 * (u * v).sin()])
            .u_range(-3.0, 3.0).v_range(-3.0, 3.0).resolution(64, 64).color(BLUE))?;
        let label = stage.add(Tex::new(r"z=\sin(xy)").build(stage.tex_engine()?)?)?;
        stage.set_fixed_in_frame(label, 1.0, true); stage.to_edge(label, UP, 0.4);
        let cam = self.rig.animate_to(stage.scene_mut(), &self.target)?;
        stage.play_prepared_with(vec![Box::new(cam)], PlayOverrides { run_time: Some(6.0), ..Default::default() })?;
        Ok(())
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut o = RenderOptions::with_format("orbit.mp4", RenderFormat::Mp4)?;
    o.config.camera.resolution = (3840, 2160); o.config.camera.fps = 60;
    o.ffmpeg = Some(FfmpegCapability::host());           // render_camera does NOT add it
    let mut base = o.camera_config()?; base.samples = 4;
    base.frame.set_euler_angles(Some(-0.5), Some(1.1), None)?; o.camera = Some(base);
    render_camera(|scene, base| {
        let rig = CameraRig::new(scene, base)?;
        let mut t = base.clone();
        t.frame.set_euler_angles(Some(0.9), Some(0.9), None).map_err(SceneError::from)?;
        t.frame.set_width(base.frame.width() * 0.7).map_err(SceneError::from)?;
        Ok((Orbit { rig, target: t }, rig))
    }, o)?;
    Ok(())
}
```

### 2.2 2D camera (item 2)

- **Pan and zoom on the retained route: still absent.** The skill doc is correct at HEAD:
  - The screen map is fixed at the frame center with `scale = height / frame_height` (`rendering.rs:797-805`).
  - `config.camera` has no center or zoom field (`fmn-config/src/config.rs:135-144`).
- **MovingCameraScene equivalent: LIVE via the camera route (R to PNG).**
  - Use `render_camera` with the default (identity) orientation and animate `frame.set_center` and `set_width` through `rig.animate_to`. Center and width are both animated in the `native_camera_pipeline.rs:147` test.
  - Per-channel tweens (`rig.width().animate().set_value(4.0)?`) type-check but have no test.
  - For z = 0 content the projection reduces to exactly orthographic (`camera.rs:774-777`).
  - Strokes keep constant screen width while zooming unless you call `stage.set_scale_stroke_with_zoom(m, true, true)` (`positional.rs:1198`).
  - Cost: everything moves to the about-2.4×-slower CPU camera route.
- **ZoomedScene / picture-in-picture inset: absent.** There is no type, no render-to-texture and no bead. Composite insets in post, or fake one with a scaled copy of a group plus a frame rectangle.
- **"Frame" mobject:** none. The `CameraRig` tracker family is the closest equivalent.

### 2.3 Vector fields, streamlines, particles (item 3)

**Fields.** Everything below is on the retained route, imported via `fmn::library`; VectorField and StreamLines are also in the prelude.

| Feature | Status | API |
|---|---|---|
| VectorField | **LIVE (E)** | `VectorField::new(f: Fn(&[[f64;3]])->Vec<[f64;3]>, cs)` with `.with_density .with_color .with_magnitude_range .with_max_vect_len .with_style_config` → `.build()?` → `.add_to_stage(stage.arena_mut())` (`fields.rs:643-843`). Colors by magnitude (3b1b colormap). There is no `ArrowVectorField` name |
| TimeVaryingVectorField | **LIVE (T)** | `fields.rs:865-985`; a dt updater rewrites the points every frame |
| StreamLines | **LIVE (E)** | `StreamLines::new(f, cs, &RngRoot::from_seed(s))` with `.with_density .with_n_repeats .with_noise_factor .with_sample_coords .with_style_config(StreamLineStyle{..})` (`fields.rs:1183-1320`). RK45 integration; geometry is self-golden locked |
| **AnimatedStreamLines (flowing)** | **LIVE (E, unit test only)** | `AnimatedStreamLines::new(lines).with_lag_range .with_rate_multiple .with_time_width .add_to_stage(arena)?` (`fields.rs:1650-1820`). One dt updater sweeps a Gaussian flash along each line for as long as you `wait` |
| Moving along a field | **LIVE / one helper BROKEN (inferred from code)** | `move_along_vector_field` and `move_submobjects_along_vector_field` (`fields.rs:2290,2312`) are fine. `move_points_along_vector_field` (`:2342-2366`) reads object-space points and writes them back as world space, so a pre-moved mobject probably jumps |
| TracedPath / TracingTail | **LIVE (T)** | `TracedPath::new().with_stroke_width(StrokeProfile::Taper(..)).add_to_stage(arena, \|s\| s.get_center(dot))?` (`fields.rs:1846-1935`); true-arc-length width and opacity tapers |

**Field gotchas:**
- Every `add_to_stage` helper adds **without rooting**, so follow it with `stage.add_to_scene(m)?`.
- `NumberPlane` used as a coordinate system must be built first, or `c2p` panics (`planes.rs:442-446`).
- A trivial custom `impl CoordinateSystem` is the safer choice.
- Certified libm leak (fm-certified-libm-leak-3aja, open P1): it is effectively closed by the frankenscipy pin and only affects certified cross-platform bits, not MP4.

**Particles (500–2,000 dots).**
- **Pattern:** one rooted `v_group` of `Dot`s built **at the origin and then shifted**, plus **one** `add_dt_updater` on the group that calls `shift` on each child.
  - Translations are placement updates with no re-tessellation (`fmn-render/src/plan.rs:43-47`).
  - Shapes are interned without their position (`table.rs:974-990`), so identical dots should share one compiled shape (inferred, not measured).
- **Avoid `always_redraw` for particles.** It deletes and recreates the family every frame, paying a SHA-256 digest and a shape compile per frame, and the orphaned rows pile up (`fmn-mobject/src/dynamics.rs:224-247`; fm-92gu in progress: the arena never reclaims removed mobjects).
- **Avoid shifting the whole group.** `apply_affine_many` is O(M²) (`positional.rs:693`).
- **No many-object benchmark exists.** The only data point: Hoeffding chapter 07 (48 dots plus `always_redraw` rugs and a heat map) at about 60 fps per process at 4K60 on the Threadripper host after cdfb9221 (README:483-490).
- **Upper limit:** the retained plan is capped at 2²⁰ input curves (fm-sq8.10, open P2).

```rust
let n = 1500; let cols = fmn::core::color::color_gradient(&[BLUE, YELLOW], n);
let swarm = stage.add(v_group(cols.iter().map(|&c| Dot::new().radius(0.03).color(c).build())))?;
let kids = stage.get(swarm).map(|e| e.submobjects().to_vec()).unwrap_or_default();
for (k, &d) in kids.iter().enumerate() { stage.shift(d, seed_point(k)); }
stage.add_dt_updater(swarm, move |s, me, dt| {
    for d in s.get(me).map(|e| e.submobjects().to_vec()).unwrap_or_default() {
        let p = s.get_center(d); s.shift(d, [(-p[1]) * dt, p[0] * dt, 0.0]);
    }
}, false)?;
stage.wait(10.0)?;
```

### 2.4 Glow, gradients, opacity, z-index, blend (item 4)

| Feature | Status | Notes |
|---|---|---|
| Bloom / blur / post-processing | **absent** | Source-over compositing only, in linear RGBA16F (`engine.rs:3274`). No bead |
| Glow (fake, 2D) | **LIVE** | `v_highlight(&src, n_layers, max_stroke_addition, [inner, outer])` (prelude; `vmobject.rs:1187-1219`): stacked widening translucent strokes. Give the source a low stroke opacity and draw a bright core on top |
| GlowDot | **LIVE on the camera route** | See 2.1 |
| Gradient along a stroke | **LIVE (E), no builder** | Per-point `stroke_rgba` and `stroke_width` give interpolated stroke profiles (`fmn-render/src/stroke_profile.rs`; pixel tests `fmn-render/tests/stroke_profiles.rs:87-279`). Write the column yourself: `stage.get_mut(m).buffer.write_range("stroke_rgba", 0, &rgba)`. There is no `set_color_by_gradient` |
| Tapered / variable width | **LIVE** | `VMobject::with_stroke_profile(vec![0.0, 6.0, 0.0])` (`vmobject.rs:284`; by point index). For a taper by true arc length use `taper_by_true_length` |
| Fill gradients | **LIVE (E), boundary-driven** | Per-point `fill_rgba` is interpolated into the interior (`fmn-render/tests/fill_profiles.rs:86`). Not CSS-style linear or radial |
| Radial gradients, sheen | **absent** | Use concentric translucent discs, GlowDot, or an RGBA sprite (camera route) |
| Gradient across submobjects | **LIVE (R)** | `fmn::core::color::color_gradient(&[..], n)`; panics with fewer than 2 anchors (`fmn-core/src/color.rs:280,292`). Also `interpolate_color_oklab` |
| Opacity | **LIVE** | `Style::fill(c, op).stroke(c, w, op)`; `stage.set_opacity(m, op, recurse)`. Group opacity is per member (no isolated layer). `set_opacity` flattens per-point alpha tapers |
| z-index | **LIVE** | `set_z_index(m, z, recurse)` takes effect only on re-add or `scene_mut().bring_to_front(&[..])` (`stage.rs:926-950`) |
| Blend modes, drop shadows | **absent** | — |
| Joints, caps, dashes | **LIVE** | `JointType::{Auto, Bevel, Miter, NoJoint}`; caps always round; `DashedLine`, `dashed_vmobject` (by arc length) |
| AA | **LIVE** | `opts.config.render.aa = AaPolicy::{Adaptive, Ssaa2x, Ssaa4x}`. Camera content is always Adaptive (use `samples`) |
| Banding | — | Use `file_writer.pixel_format = "p010le"` with a 10-bit encoder; P010 converts straight from binary16 (`rendering/pipeline.rs:282-307`) |

### 2.5 Text and math (item 5)

| Feature | Status | API |
|---|---|---|
| `Tex` | **LIVE (R)** | `Tex::new(src).font_size(f).style(Style).t2c(&[("x", BLUE)]).preamble(r"\newcommand..").math_style(..)` → `.build(stage.tex_engine()?)?` (`fmn-library/src/tex.rs:176-249`). Display style by default. No `.color()`: use `.style` |
| `TexText` | **LIVE (R)** | `tex.rs:288-346`, `.line_align(LineAlign::Left)`. Ligatures (`'` → ’, `--` → en dash) |
| `Text` / `MarkupText` | **LIVE (R / E)** | `Text::new(s).font("IBM Plex Sans").bold().italic().t2c/t2f/t2g/t2s/t2w .width .align .justify` (`text.rs:259-447`). Markup tags: `<b> <i> <u> <s> <tt> <big> <small> <sub> <sup> <span foreground weight style font_family>` (`fmn-text/src/markup.rs:209-276`) |
| `Code` | **PARTIAL (T)** | `Code::new(src).language("rust").theme(CodeTheme::dark()).line_numbers(true)` (`code.rs:44-201`). Only light/dark themes (dracula/monokai etc. map to dark); always CM Typewriter, no bold; about 25 languages |
| `Markdown` | **PARTIAL (T)** | `Markdown::new(md).build_with_math(&book, engine)` typesets `$…$`, `$$` and fenced math (`markdown_document.rs:881-920`). Plain `build` leaves math literal; no images. `MathDocumentError` needs `.map_err` |
| Tables | **PARTIAL (T)** | Only `TableMobject::from_csv / from_grid / from_frame` → `.build(&book) -> VMobject` (`data_mobjects.rs:148-214`). There is no Table, MathTable, DecimalTable, IntegerTable or MobjectTable; no cell highlighting. **Rules are black** (restyle on dark backgrounds). Children are rules first, then cells row-major |
| Matrix family | **LIVE (golden)** | `Matrix`, `TexMatrix`, `DecimalMatrix`, `IntegerMatrix`, `MobjectMatrix` with `get_row`, `get_column`, `set_column_colors` (`matrix.rs:430-840`). **Square brackets only**: use `Tex(r"\begin{pmatrix}..")` for other delimiters. Brackets are about 2× too heavy (fm-u5sb) |
| Brace / BraceLabel / between points | **LIVE (golden)** | `Brace::around(&vmob, DOWN)`, `BraceLabel::new(&vmob, label, dir)`, `line_brace(a, b, dir)` (`brace.rs:98-450`). Takes a **detached** VMobject; for a mobject on stage use `line_brace(stage.get_corner(m, DL), stage.get_corner(m, DR), DOWN)` |
| DecimalNumber / Integer / live counters | **LIVE (R)** | Builders at `numbers.rs:127-349`. The live pattern is tracker + `always_redraw` + `move_to(m, at, LEFT)` (Hoeffding `kit.rs:277-331`). **Avoid** `change_decimal_to_value`: it resets placement every frame |
| TransformMatchingTex / Strings | **LIVE (R / E)** | `fmn::library::TransformMatchingTex::new(arena, src, tgt, &src_spans, &tgt_spans)?`; take span maps with `.span_map()` **before** adding (`text_matching.rs:35-52,407`). Rendered in Hoeffding `formula.rs:112-135`. Bead fm-native-front-door-gaps-0gwp is stale on this |
| TransformMatchingShapes | **PARTIAL** | `fmn::animation::transform_matching_parts(arena, a, b, &pairs)` (`fmn-anim/src/transform_matching.rs:100`); you pair the parts yourself |
| `t2c` / isolate | **LIVE** | Matches raw source substrings; `"a"` also hits the a inside `\text{Rank}`, so use distinctive needles. `TexMobject::occurrences(needle)` gives child ordinals; `OldTex::isolate(..)` (`tex.rs:397-460`). **Tex child order will change** (fm-aia1, in progress), so don't hard-code indices |
| Title, BulletedList, SurroundingRectangle, underline, cross, background_rectangle | **LIVE** | Title and BulletedList are plain `Text` (no math) |

**Glyph gaps at the `bd00a247` pin:**
- **Greek variants:** `\theta`, `\vartheta` and `Text("θ")` in CM all draw the curly ϑ. `\varrho` equals `\rho`, and `\epsilon` equals `\varepsilon`. Lowercase Greek comes from CM *text* italic (no cmmi/cmsy/cmex bundled).
  - Blocked on an owner decision, fm-adr0028-cm-math-faces-decision-2xyy (open P1), and its implementation, fm-cm-math-faces-impl-n12u (open P1).
  - fm-5wq.60 is in progress, with only `\root` fixed so far.
- **Symbols:** relations and operators come from Noto Sans Math, which gives a heavier sans look (fm-tex-metrics-glyphs-ru72).
- **Colors:** `\color` is **silently inert**. `\textcolor`, `\mathfrak`, `\mathscr`, `\bm`, `\textit`, `\textrf`/`\texttt`, `\hspace` and `\not` are build errors.
- **Environments:** `gather`, `split`, `equation` and `\hline` are refused.
- **Symbols:** ✗ has no glyph; use `exmark` or ×.
- **Works:** `\mathbb`, `\mathcal` (script forms), `\mathbf`, `\boldsymbol`, `\operatorname`, `cases`, `pmatrix`/`bmatrix`/`vmatrix`, `array`, `\overbrace`/`\underbrace`, `\sqrt[n]`.
- Probe every formula with a glyph sheet before you rely on it.

**Fonts:**
- **Bundled faces** (all from fmd-font; no system fonts): CMU Serif in 4 styles ("Computer Modern"), CM Typewriter (regular only), IBM Plex Sans in 4 styles, and a Noto Sans Math subset used as fallback.
- **Custom fonts: LIVE (T)** via `FontBook::add_family(name, bytes)` (`fmn-text/src/font.rs:346`).
  - It registers a **regular face only** and accepts TTF only (no WOFF or TTC; CFF-based OTF probably fails).
  - Custom fonts never reach `Tex`.
  - IBM Plex has a closed θ, which is a workaround for Text.

### 2.6 Plots (item 6)

**Axes: LIVE (R)** (`crates/fmn/tests/native_render.rs:338-391` renders axes, graph, axis labels, graph label, Riemann rectangles and a vertical line):
- **Build:** `Axes::new().x_range([a,b,s]).y_range(..).width(w).height(h).axis_config(AxisConfig{..}).build(&book)?` (`coords.rs:1171-1311`).
- **Labels and graphs:** `add_coordinate_labels(&book, x, y, excluding, font_size)` (`:1353`), `get_axis_labels(engine, "x", "y")` (`:1580`), `get_graph(f)` (`:1378`).
- **Calculus helpers:** `get_area_under_graph` and `area_under_graph_styled` (`:1826,1837`), `get_riemann_rectangles(&f, Some([a,b]), Some(dx), "left")` (`:1724`), `get_tangent_line`, `i2gp`, `get_v_line_to_graph`, `get_graph_label_tex`.

**Axes placement (the old bead): fixed for `Axes`.** A *built* Axes supports `.shifted .moved_to .to_edge .to_corner .scaled .scaled_about .rotated_about .aligned_on_border` (`coords/placement.rs:79-139`, compiled via `coords.rs:50`), and `c2p` follows. Caveats:
- Place it **before** building graphs and adding it.
- Moving the added `Mob` later is not tracked, and there is no `next_to`.
- Bead fm-native-axes-reposition-n1yd stays open for the planes.

**Other plot types:**

| Feature | Status | Notes |
|---|---|---|
| NumberPlane, ComplexPlane | **LIVE centered only (T)** | `background_line_style`, `faded_line_style`, `n2p`/`p2n`. Their placement (`planes/placement.rs`) does not compile (signature mismatch at `:57`; missing macro import at `:4`) |
| NumberLine | **LIVE** | `n2p`, `add_numbers`, `.shifted .rotated_about .scaled_about` |
| FunctionGraph, ParametricCurve, ImplicitFunction | **LIVE (golden)** | — |
| BarChart | **PARTIAL** | `max_value` defaults to `Some(1.0)`; axes hard-coded black; bars hard-coded blue→yellow; no animated values. Build bars from `Rectangle` + `always_redraw` instead |
| Histogram | **absent** | Use `riemann_rectangles_styled` or `Rectangle`s |
| `get_scatterplot` | returns a camera-only `DotCloud` | Plot `Dot`s at `axes.c2p` instead |

**Animating a changing graph:** use tracker + `always_redraw(|s| s.add(axes.get_graph(..).build()))`, or `Transform` between Riemann sets as dx shrinks.

### 2.7 Network graphs (item 7)

**PARTIAL (T).**
- **Construction:** `NetworkGraph::from_edge_list(&nodes, &edges).laid_out(&GraphLayout::{Circular, Shell{..}, BreadthFirst{root}, Spring{seed, iterations, ideal_edge}})?.build()?` (`network_graph.rs:112-231`).
- **Limits:**
  - Four layouts only (no Kamada-Kawai, random or planar).
  - **Undirected only**: no DiGraph arrows.
  - Fixed grey edges and blue dots, with no style knobs and no labels.
  - Children are edges then nodes.
  - Layout changes animate by `Transform` between two builds.
- **Recommendation:** use it for `position(label)` only, and draw your own `Dot`, `Arrow` and `Text` for explainer-quality graphs.
- **NeuralNetworkMobject:** PARTIAL. Synapse lines are colored by weight sign. A layer with more than 12 units likely stacks its interior neurons on one dot (`neural_network.rs:585-593`; inferred from code).

### 2.8 SVG and images (item 8)

**SVG: LIVE (E), vector route.**
- Use `fmn::library::svg::svg_mobject_with_paints(&bytes, SvgPaintOverrides::default())` (`svg_paint.rs:172`), not plain `svg_mobject`. Only the former handles even-odd holes and dashes (`svg.rs:20-23`).
- **Supported:** paths, basic shapes, groups/defs/`use`, transforms, solid fills and strokes.
- **Refused:** gradients, clipPath, mask, pattern, filter, `<text>`, `<style>` CSS, and probably Inkscape's `sodipodi:namedview` (unknown elements are refused) (`fmn-geom/src/svg.rs:16-50,2585`).
- **Not handled for you:** there is no file loader (`std::fs::read`), and the result is **y-down and not centered**, so flip it with `.stretched_about(-1.0, 1, center)`.
- **Recoloring** works via `map_style_deep` or `stage.set_color(m, c, None, true)`; `write` and `show_creation` work on the paths.
- **For a custom fish drawing:** export Plain SVG, convert text to paths, flatten gradients, then import.

**Other assets:**

| Feature | Status | Notes |
|---|---|---|
| ImageMobject | **LIVE on the camera route only (E)** | `from_png`, `from_jpeg`, `from_rgba8`, `.with_height`, `.with_opacity` (`image.rs:93-200`). It **forces the whole render onto the camera route** |
| Per-frame raster | **PARTIAL (T)** | Animate pixels via `stage.set_image_resource` in an updater. Live-raster helpers are portal-only |
| Drawings | **LIVE (T)** | Clock, dartboard, speedometer, piano, laptop, speech/thought bubbles (`drawings.rs`). Lightbulb, video icon and earth need your own SVG |
| Boolean ops | **LIVE (E)** | `fmn::library::boolean_ops::{union, intersection, difference, exclusion}` → `.into_mobject()` → restyle. Output points are flattened (fm-ui0c), which changes where `show_creation` starts |

### 2.9 Animations (item 9)

All of the following are LIVE unless noted. Paths are under `crates/fmn-anim/src/`.
- **Transform family:** `Transform::new(a, b).with_path_arc(angle, axis)` (`transform.rs:405,439`) morphs arbitrary shapes; families with different child counts are aligned. Also `replacement_transform` (`:619`), `transform_from_copy`, `move_to_target`, `restore`, `cyclic_replace`, `swap`, `fade_to_color`.
- **Point maps:** `apply_matrix(arena, m, [[f64;3];3])`, `apply_matrix_2d` (`:750,769`), `apply_pointwise_function` (`:693`), `apply_complex_function` (`:791`). `Homotopy::new(|x,y,z,t| …, m)`, `complex_homotopy`, `smoothed_homotopy`, `PhaseFlow` (`movement.rs:36-177`).
  - **Gotcha:** NumberPlane grid lines are 2-point lines and `prepare_for_nonlinear_transform` is not ported (`planes.rs:53-55`). Nonlinear warps need a grid you build yourself from densely sampled `ParametricCurve`s. Linear `apply_matrix_2d` on a plane is fine.
- **Motion:** `MoveAlongPath::new(m, path_mob)` (true arc length; `movement.rs:262`), `Rotating::new(m).with_angle().with_about_point()`, `rotate(m, angle)`.
- **Composition:** `AnimationGroup::new/with_lag_ratio/in_group`, `Succession`, `lagged_start`, `lagged_start_map` (`composition.rs:200-684`).
  - `stage.play` takes one animation or a **2-tuple** only (`animation.rs:1034`); use `play_prepared_with` for more.
- **Creation:** `show_creation`, `uncreate`, `write`/`DrawBorderThenFill`, `show_increasing_subsets`, `add_text_letter_by_letter`, and `add_text_word_by_word` (needs word groups) (`creation.rs:288-821`).
- **Fades and grows:** `fade_in(arena, m, shift, scale)` (no defaults), `fade_out`, `fade_transform`, `grow_from_center/point/edge`, `grow_arrow`.
- **Indication** (`indication.rs`):
  - `indicate(arena, m, 1.2, None)` (`:72`).
  - `show_passing_flash(m, tw)` and `VShowPassingFlash` (tapered).
  - `WiggleOutThenIn::new(m)` (`:159`).
  - `show_creation_then_fade_around(arena, rect, lag, remover)` (`:566`) is the Circumscribe equivalent.
  - `flash`, `focus_on`, `circle_indicate` and `broadcast` are **PARTIAL**: you build the radial lines, dots or circles yourself (`:57-111`).
  - `apply_wave`, `turn_inside_out`, `flashy_fade_in`.
- **Rate functions** (`fmn-core/src/rate.rs`): `linear smooth rush_into rush_from slow_into double_smooth there_and_back lingering`. The two-argument ones (`there_and_back_with_pause running_start overshoot wiggle exponential_decay`) need a wrapper fn or `RateFunc::Sampled`. There are no `ease_*` functions.
- **Updaters** (`fmn-mobject/src/dynamics.rs`, `stage.rs`):
  - `stage.add_value_tracker(v)` (`dynamics.rs:79`), animated with `t.animate().set_value(x)?`.
  - `always_redraw` (`:224`; not rooted, so call `add_to_scene`).
  - `add_updater`, `add_dt_updater` (`stage.rs:1638,1656`), `f_always`.
  - Updaters keep running during plays, so `clear_updaters` before fading. There is no `ValueTracker` struct.
- **Lines and arrows:** `TracedPath` (2.3), `DashedLine`, `Arrow` (thickness in Reference units: use 4.0, not 0.04), `curved_arrow(start, end, angle, style)` (`arc.rs:255`), `curved_double_arrow`.

### 2.10 Audio (item 10)

**LIVE (R into MP4 and MOV).**
- **Call:** `stage.scene_mut().add_sound(path, time_offset_s, gain_db: Option<f64>, gain_to_background_db: Option<f64>)?` (`fmn-scene/src/runtime.rs:1027`). The start time is the call-site scene time plus the offset; negative starts are clipped to 0.
- **Mixing** (`fmn-output/src/sound.rs:1-63,104-109`):
  - Native and sample-exact, with exact rational placement.
  - Output is 48 kHz **stereo**; mono is duplicated, stereo averaged to mono when needed.
  - A 64-tap windowed-sinc resampler handles any input rate.
  - Cues are summed in `add` order, then clamped (`report.artifact.soundtrack.clipped_samples`).
  - `gain_to_background` ducks the already-mixed signal under the cue.
  - **No cue-count limit** found; the mix is capped at 1 hour.
- **Inputs:** WAV in U8/S16/S24/S32/F32 decodes natively. FLAC, MP3, AAC, Ogg, M4A and AIFF decode through ffmpeg (`fmn-output/src/audio.rs:24-60`). **Each cue file is at most 64 MiB** (`rendering.rs:626`), about 5.8 min of 48 kHz stereo S16, so use one WAV per line.
- **Output:** a two-stage mux, `-c:v copy -c:a aac`, with **no `-b:a`** (ffmpeg's default, about 129 kb/s; `fmn-output/src/negotiate.rs:355-366`).
- **Not available:** no filters, EQ, reverb or per-cue fades (beyond gain and duck), and no audio-bitrate knob (needs a runner wrapper).
- **Formats:** PNG, GIF and Y4M outputs carry no audio. TTS is not bundled; Hoeffding uses external WAVs (`examples/hoeffding_d/narration.rs:346`).

### 2.11 Output and performance (item 11)

- **Formats:** `RenderFormat::{PngSequence, Gif, Y4m, Mp4, Mov}` (`rendering.rs:76-95`).
  - Publication is atomic and **no-clobber**: delete the old output first.
  - **Transparent output:** `Mov` plus `background_opacity < 1` gives qtrle/argb (`rendering.rs:1170-1236`). PNG sequences are RGBA.
- **Quality:** `opts.video_quality = VideoQuality{crf, preset, tune, bitrate}` (`rendering.rs:293`; `fmn-output/src/negotiate/quality.rs:12-149`).
  - CRF, preset and tune work on libx264/libx265 only.
  - NVENC and VideoToolbox accept `bitrate` only. NVENC `-cq` or `p7` needs the `FfmpegCapability` runner wrapper (OUTPUT-AND-FFMPEG.md).
  - Encoder names: `file_writer.video_codec = "h264_nvenc" | "hevc_videotoolbox" | ...`.
- **Caps:**
  - `max_output_bytes` (default 64 GiB, `rendering.rs:261,306`) caps **both** the cumulative raw input and the artifact. 64 GiB is about 5,523 NV12 4K frames, roughly 92 s at 4K60. It is **configurable**: raise it together with `ffmpeg_limits.max_artifact_bytes` (8 GiB by default, intersected; `ffmpeg_limits.rs:39-64`).
  - The ffmpeg timeout defaults to 24 h.
  - `max_resident_bytes` is 512 MiB; raise it before raising `frames_in_flight` (default 2).
- **Speed** (Hoeffding field measurement, README:483-490, not re-run):
  - 4K60 retained route at about 60 fps per process with NVENC on a 128-thread Threadripper with `--threads 16` (2.2× after cdfb9221).
  - The whole 8:27 film, 30,431 frames, in 141 s with 8 processes in parallel.
  - On an M4 Pro under heavy load, x264 CRF 16 slow at 4K60 ran 4.8–22.5 fps.
- **Parallelism:**
  - One render captures frames **serially** (fm-sq8.5, open P1). Raster and YUV conversion fan out across the render team.
  - FMTL bundle replay (`render_bundle`) reconstructs frames on workers, but native exports store full snapshots, so it gives no speedup for a single render.
  - **The real lever is one process per chapter.**
- **macOS:** the topology is a flat fallback, so E-core tiles straggle. The Darwin `killpg` EPERM race (fm-darwin-killpg-eperm-race-7aae, open P1) occasionally fails ffmpeg probes; just re-run.
- **Color tags:** MP4 color tags probe as unknown on ffmpeg 8.1.2. Fix losslessly with `-bsf:v h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1` (no bead).

### 2.12 Other features (item 12; workflow names mapped to what they delivered)

**LIVE and worth exploiting:**
- stroke-profiles: tapered and gradient strokes.
- fill-profiles: renderer support only.
- svg-paint-ingress.
- native-subset-reveals: `show_increasing_subsets`, letter-by-letter.
- live-streamlines: seeded StreamLines plus `AnimatedStreamLines`.
- native-parameter-motion and native-method-transforms: `.animate()` with `path_arc`.
- native-graph-sampling: FunctionGraph, Parametric and Implicit graphs with sample budgets.
- surface-geometry and implement-surface-grid-alignment: surface-to-surface `Transform`, library-tested (`fmn-anim/tests/surface_grid_alignment.rs:67`).
- scene-bundle-export: `export_bundle` and `export_camera_bundle` (prelude).
- native-audio-ingestion.
- native-frame-overlap: `frames_in_flight`.

**PARTIAL:**
- network-graphs (see 2.7).
- native-tables and native-markdown: they work but look plain.
- obj-materials.
- surface-textures and native-texture-pairs: the builder bug in 2.1.
- live-raster: portal-only helpers.
- mixed-camera-wasm: the browser player scrubs bundles, with no audio; the artifact is over its size budget (fm-8j70).
- Studio (`fmn studio scene.fmtl`): a scrub and inspect viewer for exported bundles. There is no hot reload for `SceneConstruct`, and live input works only for `interactive.v1` (fm-studio-general-input-juh7).

**Not native:**
- native-math-compositions: TeX in `Title`/`BulletedList` never landed (`special_tex.rs:336`).
- complex-decimal-authoring, land-saved-controls, project-autorebuild and live-clock are Python or Studio only.
- Controls (sliders, checkboxes) render only as static props.

## 3. (a) The 15 most spectacular things it can do through the native facade

Ranked by visual impact. Confidence is that it renders at 4K to MP4 via the facade today.

| # | Effect | Key API | Confidence |
|---|---|---|---|
| 1 | Lit parametric surfaces (saddles, waves, tori) with a camera that orbits, zooms and moves its light | `ParametricSurface`, `CameraRig::animate_to`, `render_camera` | High for PNG (R); medium-high for MP4 (shared sink, set ffmpeg) |
| 2 | Equation morphs where shared glyphs glide and the rest fade, with `t2c` coloring | `TransformMatchingTex` | High (R, Hoeffding e2e) |
| 3 | Flowing streamlines over a magnitude-colored vector field | `StreamLines` + `AnimatedStreamLines` + `VectorField` | Medium (E plus portal pixels) |
| 4 | 3b1b-style linear transformation of a full plane, with basis arrows | `NumberPlane` + `apply_matrix_2d` / `.animate().apply_matrix` | Medium-high (T; plain vectors) |
| 5 | 1,000+ gradient-colored dots morphing between shapes, then flowing in a field | `v_group` of `Dot`s + `Transform`, dt updaters | High for the morph (R pattern); medium for the 2,000-dot flow (unmeasured) |
| 6 | Riemann sums refining dx → 0 with shaded area on placed Axes | `get_riemann_rectangles`, `area_under_graph_styled`, `replacement_transform` | High (R) |
| 7 | Live dashboards: tracker-driven numbers, bars, curves recomputed every frame | `always_redraw` + `add_value_tracker` | High (R) |
| 8 | Neon-glow curves and comet trails | `v_highlight`, stroke profiles, `VShowPassingFlash`, tapered `TracedPath` | Medium-high (E) |
| 9 | Complex-function warps of a dense grid (conformal maps) | `apply_complex_function`, `complex_homotopy` on a self-built dense grid | Medium (T) |
| 10 | MovingCamera-style pans and zooms into detail | camera route + rig center/width | Medium-high (R to PNG); slower route |
| 11 | Glowing point clouds and starfields in 3D | `glow_dots(...).with_glow_factor` on the camera route | Medium (E, CLI builtin) |
| 12 | Surface-to-surface morphs (plane to sphere to torus) | `Transform` over aligned surface grids | Medium (E) |
| 13 | Imported SVG art drawn on stroke by stroke, then recolored | `svg_mobject_with_paints` + `write` | Medium (E; y-flip) |
| 14 | Handwriting-style write-on of TeX and text outlines, word and letter reveals | `write`, `add_text_letter_by_letter` | High for `write` (R); medium for the others |
| 15 | Set and Venn operations with boolean shapes; implicit curves and Lissajous figures | `boolean_ops::intersection`, `ImplicitFunction`, `ParametricCurve` | Medium-high (E / golden) |

Honorable mentions: syntax-highlighted `Code` and `Markdown` with math (medium); a textured day/night globe (low until the builder bug is patched by hand); OBJ models (low-medium).

## 4. (b) Gaps and bugs that limit ambitious explainers

| # | Gap | Bead (status) |
|---|---|---|
| 1 | Frames of one render are captured serially; a long chapter is wall-clock bound | fm-sq8.5 (open P1); fm-tile-key-hash-hotspot-syg8 (P2, ~12% of a 4K frame); fm-nv12-passthrough-encoders-4s3b (P2) |
| 2 | 3D, images and camera moves force the CPU-only camera route: no tile cache, about 2.4× slower, no 4K timing; Metal refused, CUDA design-only | fm-metal-route-native-render-pank (P2), fm-sq8.6 (P1), fm-ktj (P3) |
| 3 | No 2D pan/zoom on the fast route; no ZoomedScene or insets | none |
| 4 | θ/ρ/ε draw as variants; α looks like a Latin a; Noto relation glyphs; heavy big delimiters; formulas 2–18% narrow; `\color` silently ignored; `\mathfrak`/`\textcolor` errors | fm-adr0028-cm-math-faces-decision-2xyy (P1), fm-cm-math-faces-impl-n12u (P1), fm-5wq.60 (in progress), fm-tex-metrics-glyphs-ru72 (P1), fm-u5sb (P2), fm-binom-delimiters-glyph-gaps-ed1k (P2) |
| 5 | Multi-row `Tex` is now right-aligned (`align*`); matrices and cases got taller (behavior change since 29233ca6) | fm-tex-layout-oracle-bkbc (in progress) |
| 6 | Tex child order will change, breaking hard-coded indices | fm-aia1 (in progress P1) |
| 7 | NumberPlane, ComplexPlane and ThreeDAxes can't move with `c2p` following; ThreeDAxes graphs and surfaces not compiled | fm-native-axes-reposition-n1yd (open P1) |
| 8 | `TexturedSurface` silently drops its image | **none (new finding)** |
| 9 | `render_camera` and `render_bundle` don't fill in ffmpeg | none (by design; documented) |
| 10 | No bloom, blur, blend modes, shadows, radial gradients or `set_color_by_gradient` | none |
| 11 | 3D rotations of 2D objects abort the retained route (z ≠ 0) | none |
| 12 | NetworkGraph: unstyled, undirected only, 4 layouts | none; fm-native-front-door-gaps-0gwp (P3, partly stale) |
| 13 | Arena never frees removed mobjects, so `always_redraw`-heavy chapters slow down over time | fm-92gu (in progress P2) |
| 14 | 2²⁰ input-curve cap on the retained plan (fractals) | fm-sq8.10 (P2) |
| 15 | Poles in surfaces refused | fm-8lci (P3) |
| 16 | macOS: flat topology; Darwin EPERM ffmpeg-probe race | fm-macos-hardware-topology-l29x, fm-macos-live-sysctl-reader-akab (P2); fm-darwin-killpg-eperm-race-7aae (P1) |
| 17 | No AAC bitrate or NVENC CQ knob; MP4 color tags lost | fm-video-encode-quality-knob-fm85 (stale); none for the tags |
| 18 | Tables and BarChart hard-code black rules and axes; BarChart `max_value` defaults to 1 | none |
| 19 | Inferred from code, not reproduced: `move_points_along_vector_field` placement reset; `NeuralNetworkMobject` interior stacking | none |

## 5. (c) Recommended workarounds

1. **Throughput:** render one process per chapter or segment, and join with `ffmpeg -f concat -c copy` using the same encoder everywhere. Draft at 960×540 and 30 fps. On a GPU host use NVENC through the runner wrapper; on a Mac, libx264 CRF 16–18 preset medium, or VideoToolbox with `bitrate`.
2. **Long 4K renders:** set `opts.max_output_bytes = 256 << 30` and `opts.ffmpeg_limits.max_artifact_bytes = 32 << 30`. Keep `ThreadPolicy::Fixed(n)` in line with the physical cores.
3. **3D:** isolate 3D or camera shots into their own scenes on the camera route. Set `base.samples = 4` for edge anti-aliasing. Do ambient rotation with a dt-updater that writes the rig's quaternion trackers, and split large orbits into arcs of less than π. Put HUD labels on `set_fixed_in_frame` with `frame_height` left at 8.
4. **2D zoom on the fast route:** scale and shift one top-level group, plus a frame rectangle for "inset" looks. Use `render_camera` only for true camera moves.
5. **Glyphs:** run a glyph probe sheet first, as in Hoeffding's `glyphs` command. Prefer `\vartheta`-tolerant notation, or set θ through `Text(...).font("IBM Plex Sans")`. Color formula parts via `t2c` or child recoloring, never `\color`. Use separate `Tex` objects or `TexText` for centered stacks; use `&=` for aligned derivations.
6. **Planes:** build `NumberPlane` at its final spot, or use a custom data-window mapper (`Frame2`, AUTHORING-COOKBOOK §5). For nonlinear warps, build the grid from dense `ParametricCurve`s.
7. **Glow:** use `v_highlight` (8–12 layers, low source opacity) under a bright core; `VShowPassingFlash` or tapered `TracedPath` for comets; GlowDot only in camera scenes.
8. **Gradients:** compute `color_gradient` per child, or write the `stroke_rgba` column directly.
9. **Textured globe:** after converting, set `Mobject.image` to `ImageResource::rgba8(...).with_dark_image(...)` from `ImageMobject::from_png(...).pixels()`.
10. **Particles:** one `v_group` built at the origin and shifted, one group dt-updater, no `always_redraw`. Keep `always_redraw` for readouts, and clear updaters before fades.
11. **SVG:** export Plain SVG with text converted to paths and no gradients, use `svg_mobject_with_paints`, then y-flip and recenter.
12. **Graphs and tables:** draw networks from `NetworkGraph::position` with your own Dots and Arrows; restyle table and BarChart rules to white, or build bars from `Rectangle`.
13. **Delivery:** losslessly re-tag color with `h264_metadata`. Add an AAC `-b:a` through the runner wrapper if 128 kb/s is too low. Wrap renders in a 3-attempt retry on the "Operation not permitted" string, as Hoeffding does.
14. **Template:** copy `crates/fmn/examples/hoeffding_d` (its chapter registry, narration timed from real WAV lengths, and `add_sound` score). It is the only end-to-end proof of a narrated multi-chapter native render (`render_matrix.hoeffding_d_chapters.v1` / `_full_mp4.v1`).
