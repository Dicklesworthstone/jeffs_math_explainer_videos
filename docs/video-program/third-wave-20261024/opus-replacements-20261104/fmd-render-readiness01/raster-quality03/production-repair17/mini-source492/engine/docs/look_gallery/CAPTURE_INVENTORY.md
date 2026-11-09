# Look Gallery — Reference Capture Inventory

> The committed record of the one-time Reference imagery capture (fm-xb3,
> §16.3, D-16). The captures themselves live in `gallery/reference_captures/`
> (gitignored — private fixtures per the §15.3 policy) together with their
> `PROVENANCE.json`; this file is what the repository remembers about them.

## Doctrine

- **Capture once, keep forever.** The Reference render environment (GL stack,
  Pango, LaTeX-adjacent fonts) is *recorded* in the provenance manifest at
  capture time and never maintained afterward. There is no certified
  Pango/llvmpipe environment in CI, ever.
- **Imagery, never a pixel warden.** These captures feed the human-judged
  Look Gallery (verdicts: at-least-as-good / different-but-fine, Behavior-
  Noted / regression) and G0-2's calibration study (fm-k77). No bit- or
  pixel-comparison gate consumes them.
- **Fixture policy (§15.3).** Gallery fixtures are private (not committed,
  not redistributed); the public corpus is our own permissively-licensed
  primitive scenes. The calibration scenes below are our own definitions,
  rendered *by* the Reference engine for aesthetic comparison; per-scene
  attribution applies to any future capture derived from 3b1b video scenes
  (CC BY-NC-SA), which this set deliberately avoids.

## The calibration set (§20.1 spike 2)

Scene definitions live in `scripts/capture_reference_imagery.py` (ids are
kept in lockstep by the script's inventory check). One still per scene.

| id | exercises | status |
|---|---|---|
| `gradient_fills` | fill + stroke color gradients, opacity compositing | **captured** |
| `self_intersections` | nonzero-winding fill of a self-intersecting star | **captured** |
| `joints_and_caps` | every joint type (auto/bevel/miter/no_joint) on wide zig-zag strokes, with caps | **captured** |
| `glow` | GlowDot falloff at three radii/colors | **captured** |
| `lighting_3d` | 3D sphere under the Reference lighting model, oblique camera | **captured** |
| `text_sample` | Pango text rendering (regular + italic) for the native-text comparison | **captured** |
| `math_formula` | Euler's identity for the native fmd-math comparison | **captured** (supplement session 2026-08-23, below) |

## Capture record

| field | value |
|---|---|
| Reference pin | `3b1b/manim @ 6199a00d4c1b1127ebe45cb629c3f22538b10e13` |
| capture machine | `sensedemobox` — Linux 6.17.0-41-generic x86-64, glibc 2.42, dual-socket AMD EPYC 7282 |
| GL identity | Mesa **llvmpipe** (LLVM 20.1.8, 256-bit), GL 4.5 Core Profile, Mesa 25.2.8 — software rasterization under Xvfb; no GPU involved |
| Python / closure | CPython 3.13.7; moderngl 5.12.0, PyOpenGL 3.1.10, manimpango 0.6.1, numpy 2.5.1, scipy 1.18.0, pillow 12.3.0, fonttools 4.63.0 |
| capture date | 2026-07-25 |
| per-image sha256 | `gallery/reference_captures/PROVENANCE.json` |

**On llvmpipe.** The doctrine above anticipates exactly this ("no certified
Pango/llvmpipe environment in CI, ever") — software rasterization is a fine
*capture* environment because these files are never compared bit-for-bit
against anything. The Reference draws through its own GLSL shaders, so a
conformant GL 4.5 implementation produces the Reference's look; what a
software rasterizer changes is speed, not the shading math. Recorded, not
maintained (D-16).

**Two harness defects the capture itself exposed**, both fixed in
`scripts/capture_reference_imagery.py` before the set above was accepted:

1. **The blank-capture bug (the serious one).** The harness called
   `scene.camera.get_image()`, which only *reads* the framebuffer. Nothing
   ever drew into it, so the first run wrote six byte-identical fully
   transparent PNGs — and reported six successful captures. The draw is now
   explicit (`scene.update_frame(force_draw=True)`), and `check_liveness()`
   makes the failure unrepresentable: it refuses to publish any set in which
   a frame is a single flat color or two frames are pixel-identical, and the
   whole set is now written all-or-nothing. A blank Look Gallery would have
   calibrated G0-2 against nothing while looking entirely healthy.
2. **`joints_and_caps` was clipped.** Four 2.4-unit rows overflowed the
   8-unit frame, cutting `auto` and `no_joint` off-screen in a scene whose
   entire purpose is showing every joint type. The stack is now fitted to
   the frame height.

**Supplement session — `math_formula` (fm-gjl7), 2026-08-23.** The panel
postdated the 2026-07-25 set; the capture harness now supports supplement
semantics (PNGs already on disk are never re-rendered or overwritten, and
`PROVENANCE.json` is merged with the session appended under `supplements` —
the original record stays byte-identical). Same machine and same llvmpipe GL
identity as the base session; per-session closure recorded in the manifest:
CPython 3.13.1 (uv-managed), numpy 2.5.2, scipy 1.18.1, moderngl 5.12.0,
PyOpenGL 3.1.10, manimpango 0.6.1, pillow 12.3.0, fonttools 4.63.0. The
scene renders `Tex(r"e^{i\pi} + 1 = 0", font_size=96)` centred — the exact
string and scale of the native panel in
`spikes/g0-8-accelerator/src/bin/g0_2_look.rs`. sha256
`c2eb1124091572f076c6badb10ba8716aff7d84a6f05751df5b0b7534d20807c`.
Environment note: the run needed `texlive-fonts-extra` (`dsfont.sty`) and
`texlive-science` (`physics.sty`) installed for the Reference's default TeX
template; this is the recorded capture machine's dependency closure, not a
maintained environment (D-16).

## Regenerated gallery (fm-5wq.50, 2026-10-05)

The gallery is now regenerated from **installed release artefacts**. It no
longer uses the G0-2 spike harness, which flipped y itself and so never ran
the production 2D route.

- **Scenes.** Every panel is a committed scene class under `gallery/scenes/`:
  - `primitives.py`: the seven calibration scenes above, unchanged in content;
  - `math_sheet.py`: fifty authored tier-1 formulas in two sheets. The
    corpus formulas stay private under §15.3;
  - `readme.py`: twenty README-class scenes.
- **Command.**
  `scripts/regenerate_look_gallery.py --fmn-python PATH [--reference-python PATH]`.
  - The installed `fmn-python` renders each panel as a certified final still
    into `gallery/renders/` (committed). The build id comes from that render's
    provenance sidecar.
  - The pinned Reference renders the same unedited source (`-s -w`, xvfb, a
    private config directory) into `gallery/reference_captures/panels/`. That
    directory is private and never touches the original capture set above.
- **Manifest** (`crates/fmn-conformance/fixtures/look_gallery.tsv`, v2).
  - It names the release under review once. Each row records its source,
    both digests and its build id.
  - `fmn_conformance::gallery::render_pairs` refuses a panel from another
    build, or a render or capture whose bytes changed.
  - Its verdict column is **advisory** (agent review) and resets to
    `unreviewed` whenever a render's pixels change.
- **Owner verdicts** (`crates/fmn-conformance/fixtures/look_gallery_owner_verdicts.tsv`).
  - Each verdict is bound to the render digest it judged. The vocabulary is
    enforced by `tests/look_gallery.rs`: at-least-as-good,
    different-but-fine with a named BN, or regression.
  - G2 and G4a cite only these.

**Incident, recorded in `PROVENANCE.json` supplements.** The first version of
the regeneration script wrote its captures to the paths of the 2026-07-25 set
and overwrote them.
- `lighting_3d` was restored byte-identically by re-running
  `capture_reference_imagery.py`.
- `joints_and_caps`, `math_formula`, `self_intersections` and `text_sample`
  re-captured byte-identical.
- `glow` and `gradient_fills` no longer reproduce on this host, so the July
  bytes are lost. Their recorded digests remain in `captures`, and the files
  now hold this host's capture of the same scenes.
- The script now writes only to `panels/` and refuses to replace a capture it
  did not record.
