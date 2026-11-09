# Python vector output

`Scene.render("diagram.svg")`, `Scene.render_session("diagram.svg")`, and
`fmn-python --format svg` export the final native scene as an SVG document.
Text and mathematics are native glyph outlines: the SVG does not need fonts
installed in its viewer, invoke a typesetter, or contain a raster screenshot.

```python
from manimlib import Scene, Square, Tex, RIGHT, UP

class Diagram(Scene):
    def construct(self):
        box = Square(fill_opacity=1)
        self.add(box, Tex(r"x^2 + y^2").shift(UP))
        self.play(box.animate.shift(RIGHT), run_time=1)

result = Diagram().render("diagram.svg", resolution=(1280, 720))
```

The console uses the same native generation owner and supports explicit scene
names and `--write_all`. With multiple names, the output directory contains one
`NAME.svg` per selected scene; selectors and destinations are checked before
rendering. `--transparent` omits the background rather than rasterizing alpha.

```sh
fmn-python lesson.py Diagram --format svg --video_dir diagram.svg
fmn-python --robot lesson.py First Second --format svg --video_dir diagrams
fmn-python lesson.py --write_all --format svg --transparent --video_dir diagrams
```

## What the document represents

SVG is **final-state vector output**, not an animated SVG or a frame sequence.
The native skip/final-state timeline runs the scene to its semantic endpoint,
then serializes its final quadratic paths. Camera pan, zoom, in-plane rotation,
and fixed-in-frame attachment use the actual native camera. Families retain
native painter order and shared-object placements. `stroke_behind` emits two
adjacent vector paint passes in the appropriate order. Camera backgrounds are
ordinary vector rectangles; fully transparent backgrounds add no geometry.

Flat fill/stroke colors and native text/TeX outlines are supported. The current
route refuses image/texture/surface/point-cloud primitives, lighting, depth tests,
user clip planes, per-vertex paint or variable-width strokes, and paths whose
perspective projection has varying homogeneous weight. Those are explicit
capability errors, not silently flattened approximations; PNG remains available
for Lumen raster output. A nonzero camera tilt may therefore refuse world-space
curves while a fixed-frame overlay remains representable.

Stroke widths follow the existing native SVG export convention (SVG pixel
widths). The SVG viewer owns its rasterization and compositing. This is not a
promise to match Lumen antialiasing, curve-distance strokes, or linear-light
blending pixel for pixel. Python SVG output is standard-mode, not a certified
C1-C10 render; `--reproducible` remains an explicit refusal.

## Publication and limits

One completed generation publishes one document and a `RenderResult` with
`format="svg"`, `engine="native-svg"`, `frame_count=1`, byte length and SHA-256.
The destination must not exist. Reel's atomic create-new capability arbitrates
concurrent writers at publication, so an earlier artifact is never overwritten.
An exception, unsupported scene, invalid geometry or exceeded budget publishes
nothing. The source scene's geometry is never replaced by projected coordinates:
projection is applied to a private native copy-on-write snapshot.

The vector route admits at most 32,768 drawn point-carrying entries, 262,144
point records, and 64 MiB of encoded SVG, with dimensions 1–16,384 and thread
budgets 1–96. Thread count does not select an alternative SVG serializer.
Video encoder, pixel-format and ffmpeg options do not apply to SVG. The existing
`-s` switch specifically requests a final PNG; use `--format svg` on its own for
a vector still.
