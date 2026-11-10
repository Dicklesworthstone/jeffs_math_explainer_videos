# Upstream Issue Packets for `franken_manim` (`U-1`..`U-17` + Candidate `U-18`)

> **Policy (AGENTS.md & Plan §14, Q9, D-17):** Never edit `franken_manim` from this repository, and **do NOT file beads in `franken_manim`'s tracker without the owner's explicit go-ahead**. Every gap below has a live, executable minimal reproduction in [`main.rs`](main.rs), verified against the workspace's pinned `franken_manim` revision (`a399d626c43ecc852a5c795288979f52b446bc5c`), together with the workaround used across this corpus and the read-only lookup from `franken_manim`'s `.beads/issues.jsonl`.

## Running the Reproductions & Evidence Generator

```bash
# Run all 18 live minimal reproductions as a unit test:
cargo test -p explainer_kit --example upstream

# Generate packets.md, evidence.json, and the 4 diagnostic PNG still sets:
cargo run --release -p explainer_kit --example upstream -- /tmp/upstream_packets
```

---

## Summary Matrix

| ID | Gap | Existing `franken_manim` Bead(s) | Corpus Workaround | Repro Function |
|---|---|---|---|---|
| **U-1** | `TexturedSurface` silently drops its image (`From<TexturedSurface> for Mobject` sets `image: None`) | `none` (`fmn-library/src/solids.rs:2725-2753`) | `Surface::color_by_uv_function` or set `Mobject.image` manually | `repro_u01` |
| **U-2** | Chapter soundtrack quantized to `s16` **without dither** (`DitherPolicy::None`) before ffmpeg | `none` (`fmn-output` chapter mux) | `explainer_kit::master` (`D-20`): float replay + byte-exact null test against captured `soundtrack.wav` | `repro_u02` |
| **U-3** | No AAC bitrate knob in the facade (ffmpeg default ≈ 129 kb/s) | `fm-video-encode-quality-knob-fm85` (`[open]`, video CRF only) | `QualityRunner` injects `-b:a 256k`; `master::join_and_mux` encodes AAC 256k once | `repro_u03` |
| **U-4** | MP4 colour tags probe as `unknown` on ffmpeg 8.1.2 after video-only remux/concat | `none` (related: `fm-p010-precision-transfer-tag-8n7u`, `fm-nv12-passthrough-encoders-4s3b`) | Lossless `-bsf:v h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1:video_full_range_flag=0` in `master::join_and_mux` | `repro_u04` |
| **U-5** | No 2D pan/zoom camera on the retained route | `none` | Root group `stage.scale` / `stage.shift` on the retained route, or `render_camera` segment | `repro_u05` |
| **U-6** | No post-process bloom, Gaussian blur, or radial shader gradients | `none` | `v_highlight` glow and concentric translucent halo discs (`props::glow_dot`) | `repro_u06` |
| **U-7** | Greek glyph variants: math `\alpha` looks like italic Latin `a`; `\theta`/`\rho`/`\epsilon` variants | `fm-adr0028-cm-math-faces-decision-2xyy` (`[open]`), `fm-cm-math-faces-impl-n12u` (`[open]`), `fm-5wq.60` (`[in_progress]`) | Use `\text{α}` or `\mathsf{\alpha}` per Plan §6.5; verify with `./x <slug> glyphs` (`glyph_probe_s7`) | `repro_u07` |
| **U-8** | `\binom` delimiters and drawn big delimiters ~2× heavier than `cmex10` | `fm-binom-delimiters-glyph-gaps-ed1k` (`[open]`), `fm-u5sb` (`[open]`) | Use `C(n, k)` where compact notation is needed; verify `\binom` spacing on glyph probe sheets | `repro_u08` |
| **U-9** | Many-particle performance at 4K60 and ~12× slowdown over long closed background strokes | `none` (Spike S-2 / `mx-e-w7-y6w.1`) | 1,000 procedural fish driven by one group `dt` updater (`spike_s2_fish.rs`); no long closed strokes under moving objects | `repro_u09` |
| **U-10** | Camera route has no tile cache, linear quaternion interpolation, and Metal is refused | `fm-metal-route-native-render-pank` (`[open]`), `fm-sq8.6` (`[open]`) | Dedicated short camera segments, orbits split into arcs `< π` (`120°`), encoded via `encode::render_camera` (`spike_s5_camera.rs`) | `repro_u10` |
| **U-11** | Arena slot reclamation under `always_redraw` and per-frame rebuild cost | `fm-92gu` (`[in_progress]`; native unpinned slots freed at `a399d626`, pinned portal handles deferred) | Recolour/shift in place (`props::recolor_row_strip`, `dt` updaters) and call `stage.clear_updaters` once settled (`spike_s2_fish.rs` S-8) | `repro_u11` |
| **U-12** | `NumberPlane` / `ComplexPlane` / `ThreeDAxes` `c2p` stays in the unmoved frame after `stage.shift` | `fm-native-axes-reposition-n1yd` (`[open]`) | `explainer_kit::Frame2` builds axes in place at `center` with exact `c2p` / `p2c` round-trip | `repro_u12` |
| **U-13** | `TableMobject` and `BarChart` hard-code `BLACK` (`#000000`) rules and axes | `none` (`fmn-library/src/data_mobjects.rs:292,313,326,553`) | Build tables and charts from kit primitives (`rect`, `line`, `Frame2`, `props::row_strip`) with `Palette::ink` / `Palette::muted` | `repro_u13` |
| **U-14** | Every frame snapshots every arena slot serially; multi-subpath `fill_rgba` bleeds across cells | `fm-sq8.5` (`[open]`); Spike S-3 (`mx-e-w7-y6w.2`) | Pack dense grids into `ShapeTag::Polyline` row/column strips (`props::row_strip`, `spike_s3_table.rs`), cutting 255×255 from 65,025 slots to 255 | `repro_u14` |
| **U-15** | Group positional ops (`apply_affine_many`) deduplicate the family with `Vec::contains` in $O(M^2)$ | `none` for `fmn-mobject/src/positional.rs:707` (`fm-5wq.31` `[in_progress]` fixed `stage.rs:1161` `Stage::family`) | Keep transformed groups under ~2,000 members per frame, or shift leaf `Mob`s directly in a `dt` updater (`props::swarm_step`, `spike_s4_s6.rs`) | `repro_u15` |
| **U-16** | Camera-route MP4+audio installs no default delivery `FfmpegCapability` | `none` (by design in `fmn`; proven in Spike S-5 / `mx-e-w7-y6w.4`) | `explainer_kit::encode::render_camera` refuses bare MP4/MOV options and requires `delivery_options` (`spike_s5_camera.rs`) | `repro_u16` |
| **U-17** | No `\cancel` in TeX; `\preceq` and `✗` (`U+2717`) missing from bundled faces | `fm-binom-delimiters-glyph-gaps-ed1k` (`[open]`) | Draw diagonal strike `Line`s over the target bounding box; use `explainer_kit::exmark` for `✗` | `repro_u17` |
| **U-18** | *(Candidate from S-5 / `mx-e-w7-y6w.8`)* `fmn-output` `encode_argv` tags output BT.709 but leaves rawvideo input untagged | `none` (`fmn-output/src/negotiate.rs` `encode_argv`) | `QualityRunner::adjust` splices `-colorspace bt709 -color_range <range>` before `-i -` (`tests/exact_colour.rs`) | `repro_u18` |

---

## Detailed Packets

### U-1 — `TexturedSurface` silently drops its image on `Mobject` conversion
- **Upstream location:** `crates/fmn-library/src/solids.rs:2725-2753` (`impl From<TexturedSurface> for Mobject`)
- **Existing `franken_manim` bead:** `none`
- **Expected:** Converting a `TexturedSurface` into a `Mobject` preserves or loads `light_texture` / `dark_texture` into `Mobject.image`.
- **Observed:** `TexturedSurface::with_dark(sphere, "day.png", Some("night.png"))` records `light_texture() == "day.png"` and `num_textures() == 2`, but `Mobject::from(ts)` constructs `Mobject { image: None, .. }`.
- **Workaround:** Use `Surface::color_by_uv_function` for procedural UV surfaces or populate `Mobject.image` manually after conversion.

### U-2 — Chapter soundtrack quantized to `s16` without dither before ffmpeg
- **Upstream location:** `crates/fmn-output` chapter soundtrack mux path (`SampleFormat::S16`, `DitherPolicy::None`)
- **Existing `franken_manim` bead:** `none`
- **Expected:** Either float/24-bit PCM handoff or `DitherPolicy::Tpdf` when quantizing to 16-bit PCM.
- **Observed:** `MixReport::wav_bytes(SampleFormat::S16, DitherPolicy::None)` truncates without dither, and per-chapter AAC encoding introduces a second lossy generation when chapters are concatenated.
- **Workaround:** `explainer_kit::master` (`crates/explainer_kit/src/master.rs`, D-20) replays `Narrator::cue_log` through `fmn_output::SoundMixer`, verifies byte-for-byte identity against fmn's captured `soundtrack.wav` via `null_test`, and masters the entire programme in `f64` (`to_stereo`) before a single 256 kb/s AAC encode.

### U-3 — No AAC bitrate knob in the `fmn` facade
- **Upstream location:** `crates/fmn-output/src/negotiate.rs` (`-c:a aac` without `-b:a`)
- **Existing `franken_manim` bead:** `fm-video-encode-quality-knob-fm85` (`[open]`, covers video CRF knob)
- **Expected:** `RenderOptions` allows specifying audio bitrate (e.g. `256k`) for chapter renders.
- **Observed:** Bare `RenderOptions::with_format("out.mp4", RenderFormat::Mp4)` emits `-c:a aac` with ffmpeg's default (~129 kb/s).
- **Workaround:** `explainer_kit::encode::QualityRunner` splices `-b:a 256k` after `-c:a aac`, and `explainer_kit::master::join_and_mux` encodes the final master at `-c:a aac -b:a 256k`.

### U-4 — MP4 colour tags probe as `unknown` on ffmpeg 8.1.2 after stream-copy
- **Upstream location:** `crates/fmn-output/src/negotiate.rs`
- **Existing `franken_manim` bead:** `none` (related open colour beads: `fm-p010-precision-transfer-tag-8n7u`, `fm-nv12-passthrough-encoders-4s3b`)
- **Expected:** Stream-copied / concatenated MP4 masters preserve `bt709` colour primaries, transfer characteristics, matrix coefficients, and TV range in `ffprobe`.
- **Observed:** Without bitstream VUI metadata, video-only remux and concat demuxing leave colour tags probing as `unknown` on ffmpeg 8.1.2.
- **Workaround:** `explainer_kit::master::join_and_mux` passes `-bsf:v h264_metadata=colour_primaries=1:transfer_characteristics=1:matrix_coefficients=1:video_full_range_flag=0` plus `-color_primaries bt709 -color_trc bt709 -colorspace bt709 -color_range tv`.

### U-5 — No 2D pan/zoom camera on the retained route
- **Upstream location:** `crates/fmn` retained 2D render pipeline
- **Existing `franken_manim` bead:** `none`
- **Expected:** `Stage` on the retained route supports animated 2D camera frame pan and zoom (`MovingCameraScene` equivalent).
- **Observed:** Retained route uses fixed `[-7.111, 7.111] × [-4.0, 4.0]` coordinates; `CameraRig` is only available on `render_camera`.
- **Workaround:** Group the scene elements under a root `Mob` and animate `stage.scale(root, ...)` / `stage.shift(root, ...)`, or render a camera-route segment via `explainer_kit::encode::render_camera`.

### U-6 — No post-process bloom, Gaussian blur, or radial shader gradients
- **Upstream location:** `crates/fmn-render`
- **Existing `franken_manim` bead:** `none`
- **Expected:** Native bloom/blur or radial gradient fills for glowing highlights.
- **Observed:** Vector fills are flat or per-point linear contour interpolants without post-process blur.
- **Workaround:** Use `fmn::prelude::v_highlight` or `explainer_kit::props::glow_dot` (concentric translucent discs with monotone decreasing alpha).

### U-7 — Greek glyph variants (`\alpha` in math reads as italic Latin `a`)
- **Upstream location:** `fmd-math` font role mapping prior to `cmmi10` bundling
- **Existing `franken_manim` beads:** `fm-adr0028-cm-math-faces-decision-2xyy` (`[open]`), `fm-cm-math-faces-impl-n12u` (`[open]`), `fm-5wq.60` (`[in_progress]`)
- **Expected:** Math-mode `\alpha`, `\theta`, `\rho`, `\epsilon` render with distinct Computer Modern math italic shapes.
- **Observed:** Spike S-7 (`examples/glyph_probe_s7.rs`) showed math `\alpha^8` is visually indistinguishable from italic `a^8`.
- **Workaround:** Use `\text{α}` (upright serif alpha) or `\mathsf{\alpha}` (sans alpha) per Plan §6.5 and inspect `./x <slug> glyphs` sheets for every video.

### U-8 — `\binom` delimiters and heavy drawn big delimiters
- **Upstream location:** `fmd-math` drawn delimiter strokes
- **Existing `franken_manim` beads:** `fm-binom-delimiters-glyph-gaps-ed1k` (`[open]`), `fm-u5sb` (`[open]`)
- **Expected:** Tall parentheses and brackets match `cmex10` weight and spacing.
- **Observed:** Drawn big delimiters in `\binom{M}{k}` and matrices are ~2× heavier than `cmex10` and crowd the top entry at small point sizes.
- **Workaround:** Use `C(n, k)` where compact notation is needed, or verify `\binom` spacing on `./x <slug> glyphs` probe sheets.

### U-9 — Many-particle performance at 4K60 and closed-stroke background cliff
- **Upstream location:** `crates/fmn-render` stroke/fill rasterizer
- **Existing `franken_manim` bead:** `none` (measured in Spike S-2 / `mx-e-w7-y6w.1`, `examples/spike_s2_fish.rs`)
- **Expected:** 1,000+ moving particles render quickly at 4K60 regardless of background geometry.
- **Observed:** 1,000 procedural fish at 4K60 render at 1.8–2.9 fps (~700 CPU-ms/frame), and placing long closed contour strokes underneath moving particles slows rendering by ~12×.
- **Workaround:** Move particles with a single group `dt` updater (placement-only `stage.shift`), limit `v_highlight` glow to marked particles, and never place long closed strokes under moving objects.

### U-10 — Camera route has no tile cache, linear quaternion interpolation, and Metal is refused
- **Upstream location:** `crates/fmn-render` and `crates/fmn-scene` (`CameraRig`)
- **Existing `franken_manim` beads:** `fm-metal-route-native-render-pank` (`[open]`), `fm-sq8.6` (`[open]`)
- **Expected:** Camera route caches static tiles or runs on Metal; `CameraRig` slerps great-circle arcs.
- **Observed:** Spike S-5 (`examples/spike_s5_camera.rs`) measured 484 lit 3D `Prism` bars + 2 translucent slice planes at 4K60 at 6.1 fps on CPU; `CameraRig` interpolates quaternions linearly, so single arcs `>= π` flip.
- **Workaround:** Keep 3D camera-route scenes to short dedicated segments, split orbits into arcs `< π` (e.g. `3 × 120°`), and encode via `explainer_kit::encode::render_camera`.

### U-11 — Arena slot reclamation under `always_redraw` and updater lifecycle
- **Upstream location:** `crates/fmn-mobject/src/stage.rs` (`finalize_delete`) and `dynamics.rs`
- **Existing `franken_manim` bead:** `fm-92gu` (`[in_progress]`)
- **Expected:** Replaced `always_redraw` mobjects free their arena slots immediately and frame time remains constant over 80 s chapters.
- **Observed:** At `a399d626` (verified in Spike S-8, `mx-e-w7-y6w.7`), native `always_redraw` *does* reclaim unpinned slots (`stage.len()` stays constant; 80 s grid render ran at 46 ms/frame at `t = 0` vs 36–37 ms/frame at `t = 80 s`), whereas pinned handles (such as the Python portal) defer deletes.
- **Workaround:** Prefer in-place recolouring/placement (`props::recolor_row_strip`, `dt` updaters) and call `stage.clear_updaters` once a live readout settles.

### U-12 — `NumberPlane` / `ComplexPlane` / `ThreeDAxes` `c2p` does not follow `stage.shift`
- **Upstream location:** `crates/fmn-library/src/planes.rs:874`
- **Existing `franken_manim` bead:** `fm-native-axes-reposition-n1yd` (`[open]`)
- **Expected:** Repositioning a built plane updates `c2p` / `p2c`, or planes can be constructed directly at a target center.
- **Observed:** After `stage.shift(plane_mob, [3.0, -1.5, 0.0])`, the drawn plane center moves to `[3.0, -1.5, 0.0]`, but `plane.c2p(&[1.0, 2.0])` still returns `[1.0, 2.0, 0.0]` in the unmoved origin frame.
- **Workaround:** Use `explainer_kit::Frame2` (`c2p`, `p2c`, `axis_x`, `axis_y`), which builds axes in place at `Frame2::center`.

### U-13 — `TableMobject` and `BarChart` hard-code `BLACK` (`#000000`) rules and axes
- **Upstream location:** `crates/fmn-library/src/data_mobjects.rs:292,313,326,553`
- **Existing `franken_manim` bead:** `none`
- **Expected:** `TableMobject` and `BarChart` allow configuring rule, axis, and tick colours for dark backgrounds.
- **Observed:** Both builders hard-code `Style::default().stroke(BLACK, ...)`, producing `#000000` lines that are invisible on the corpus `#0E1116` background.
- **Workaround:** Build tables and bar charts from `explainer_kit` primitives (`rect`, `line`, `Frame2`, `props::row_strip`) styled with `Palette::ink` (`#ECEFF4`) and `Palette::muted` (`#9AA5B4`).

### U-14 — Serial per-frame arena snapshot and multi-subpath fill bleed
- **Upstream location:** `crates/fmn-mobject` / `crates/fmn-render`
- **Existing `franken_manim` bead:** `fm-sq8.5` (`[open]`); Spike S-3 (`mx-e-w7-y6w.2`, `examples/spike_s3_table.rs`)
- **Expected:** Dirty-slot / parallel frame capture and independent fill colours per closed sub-path.
- **Observed:** Every frame snapshots every arena slot serially, and per-point `fill_rgba` across square sub-paths bleeds across 64,221 of 65,025 cells in a 255×255 grid (Spike S-3).
- **Workaround:** Pack dense grids into `ShapeTag::Polyline` row/column strips with per-point `stroke_rgba` (`props::row_strip`, `examples/spike_s3_table.rs`), reducing 255×255 from 65,025 slots to 255 with exact pixel colours.

### U-15 — Group positional ops (`apply_affine_many`) deduplicate the family in $O(M^2)$
- **Upstream location:** `crates/fmn-mobject/src/positional.rs:703-711` (and `ancestors` at line `746`)
- **Existing `franken_manim` bead:** `none` for `positional.rs:707` (`fm-5wq.31` `[in_progress]` fixed `Stage::family` in `stage.rs:1161` to use `HashSet`, but `apply_affine_many` still loops over `self.family(mob)` with `if !members.contains(&member)` on a `Vec<Mob>`)
- **Expected:** `stage.shift(group, ...)` / `apply_affine_many` runs in $O(M)$ time.
- **Observed:** `apply_affine_many` performs $\binom{M}{2}$ comparisons on every group transform even when `mobs.len() == 1` and `self.family(mob)` is already deduplicated.
- **Workaround:** Keep transformed groups under ~2,000 members per frame, or shift leaf `Mob` handles directly inside a `dt` updater (`props::swarm_step`, `examples/spike_s4_s6.rs`).

### U-16 — Camera-route MP4+audio installs no default delivery `FfmpegCapability`
- **Upstream location:** `crates/fmn/src/rendering.rs` (`render_camera`)
- **Existing `franken_manim` bead:** `none` (by design in `fmn`; proven with audio in Spike S-5 / `mx-e-w7-y6w.4`)
- **Expected:** Camera-route segments and retained-route chapters encode with identical H.264/AAC settings so stream-copy concatenation succeeds.
- **Observed:** `RenderOptions::with_format("out.mp4", RenderFormat::Mp4)` has `ffmpeg: None`.
- **Workaround:** `explainer_kit::encode::render_camera` refuses any MP4/MOV render with `options.ffmpeg.is_none()` and requires `explainer_kit::encode::delivery_options`.

### U-17 — No `\cancel` in TeX; `\preceq` and `✗` (`U+2717`) missing from bundled faces
- **Upstream location:** `fmd-math` / bundled fonts
- **Existing `franken_manim` bead:** `fm-binom-delimiters-glyph-gaps-ed1k` (`[open]`)
- **Expected:** `\cancel{...}`, `\preceq`, and `✗` typeset directly from bundled faces.
- **Observed:** `Tex::new(r"\cancel{x}").build(&book)` errors (`UnknownControlSequence`), and `✗` has no glyph in the bundled faces.
- **Workaround:** Draw diagonal strike `Line`s across the target mobject's `extent()` and use `explainer_kit::exmark` (two crossed vector lines) for `✗`.

### Candidate U-18 — `fmn-output` `encode_argv` leaves rawvideo input colour untagged
- **Upstream location:** `crates/fmn-output/src/negotiate.rs` (`encode_argv`)
- **Existing `franken_manim` bead:** `none` (candidate `U-18` recorded on `mx-e-w7-y6w.8`)
- **Expected:** `encode_argv` places `-colorspace bt709 -color_range tv` before `-i -` on rawvideo inputs in addition to tagging the output.
- **Observed:** Because the rawvideo input is untagged, ffmpeg converts incoming BT.709 NV12 bytes using its default matrix, shifting `#0E1116` from `Cb 130.5` to `129` (green-black) and `#FFD866` from `Cr 149.7` to `146`, `Y 201.6` to `203` (verified at lossless CRF 0 in Spike S-5 / `3851db4`).
- **Workaround:** `explainer_kit::encode::QualityRunner::adjust` (`crates/explainer_kit/src/encode.rs:158-175`) splices `-colorspace bt709 -color_range <range>` before `-i -` on every rawvideo job (pinned by `crates/explainer_kit/tests/exact_colour.rs`).
