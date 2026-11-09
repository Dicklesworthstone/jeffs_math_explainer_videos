# Native OBJ models with materials and textures

`ThreeDModel(obj_file, height=3)` loads OBJ geometry and its declared MTL
libraries into native, lit mesh children. The class remains a heterogeneous
`Group`; an unmaterialed model still has one plain `Surface` child. Named
material runs become colored `Surface` or textured `TexturedGeometry` children.

```python
from manimlib import ThreeDModel

model = ThreeDModel("assets/model.obj", height=3)
scene.add(model)
scene.play(model.animate.rotate(0.5))
for part in model:
    print(part.material_name)
```

OBJ parsing and triangle construction use Atlas's existing geometry parser.
The added native MTL parser reads `newmtl`, RGB `Kd`, `d`, `Tr`, `map_Kd` and
`map_Ka`. A diffuse `map_Kd` takes precedence over the ambient-map fallback,
regardless of their order within a material. Across libraries, the first
material definition in OBJ library order wins. Texture paths are relative to
the MTL file which defines them, not the working directory or OBJ directory.
Multiple material bindings to the same resolved image reuse its decoded native
resource. Unused material images are not opened.

Face order is retained as contiguous material runs. An A/B/A sequence remains
three children instead of moving the final A faces before B. UVs and normals
remain per corner, so a shared vertex may have different UVs on either side of
a seam. OBJ V coordinates are converted once to the native top-row-first image
convention. All parts use one whole-model center and height normalization;
each part is **not** independently centered or stretched.

Colored parts use `Kd` and dissolve alpha. Textured parts use their image and
dissolve alpha with the existing native lighting; `Kd` is not multiplied into
the image. This is the existing manim-facing lighting model, not a new MTL/PBR
shader. Texture sampling uses the existing native image sampler.

## Failure and ownership

The host reads local files, while all geometry, material parsing, image decoding
and rendering remain native. No trimesh, pywavefront, Pillow, external image
tool, downloader, or temporary conversion file is used. Geometry/source errors,
missing libraries, undefined selected materials, missing or invalid required
images, and textured corners lacking UVs are errors: the loader does not
silently return a gray substitute.

All required inputs and mesh children are prepared before initializing the
receiving Group. An input or native preparation failure does not replace an
existing detached model's family. Bound reinitialization and same-object
constructor reentry refuse before input conversion. Authored subclass hooks,
path conversion side effects and host filesystem races are not rolled back;
this is an ordinary trusted-host authoring API, not a sandbox.

Images are immutable native resources. Copies, pickle restoration, captures
and scene snapshots keep their resources and do not reopen the original files.
The existing pixel/material setters work on textured children; ordinary group
placement animation keeps their mesh topology and images. Triangle meshes are
not sent through UV-grid resampling merely because their class derives from
`Surface`. A mesh-to-UV-grid alignment still refuses rather than inventing grid
topology.

`model.asset_paths` lists the absolute inputs actually read. It can be passed
as explicit asset paths to a `SceneProject` watch; this importer does not install
a second watcher or claim certified closure for mutable host files.

## Admission limits and format boundary

The portal bounds each input to 64 MiB, total distinct encoded/source bytes to
128 MiB, and material/input counts to 4096. Native image decoding has the
existing 16,777,216-pixel per-raster limit. Total decoded texture bindings are
limited to 256 MiB, conservatively counting each material binding even when
multiple bindings share one allocation. OBJ material import allows at most
262,144 triangles, 64 KiB per source line and 4096 bytes per name/path; vertex,
UV and normal counts also obey the native geometry limits. Nonregular files
are refused without blocking on FIFOs. Local relative/absolute paths and file
symlinks are supported; paths are not confined to the OBJ's directory.

Sources must be UTF-8. `mtllib` accepts whitespace-separated library paths;
quoted library filenames and continued lines are not supported. Texture-map
filenames may contain spaces. Diffuse map options such as `-s`, `-o`, or
`-clamp`, non-RGB `Kd`, and out-of-range RGB/dissolve values explicitly refuse.
Other MTL reflectance properties are ignored, not emulated. OBJ polygon fan
triangulation and its geometry-subset limitations remain those of the original
parser; this feature does not add concave-polygon tessellation or free-form
surfaces.

## Native Rust composition

The original geometry-only `ThreeDModel::from_obj` behavior is unchanged.
For material-aware construction, use `fmn_library::obj_materials::ObjDocument`
and `parse_mtl`, then supply material-name-to-`ImageResource` bindings to
`ObjDocument::to_mobject`. That library layer performs no filesystem access;
asset authority stays with the host.

The independent Rust tests are `fmn-library/tests/obj_materials.rs`. Native
portal acceptance is `fmn-python/tests/obj_materials.py`; host input policy is
`test_obj_inputs.py`, whose parser doubles are explicitly not geometry evidence.
