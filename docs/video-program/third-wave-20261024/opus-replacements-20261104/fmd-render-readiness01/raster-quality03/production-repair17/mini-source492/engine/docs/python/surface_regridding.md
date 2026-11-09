# Live surface resolution

`Surface.set_resolution((nu, nv))` samples the current UV recipe at a new grid
resolution without replacing the scene object. It supports refinement,
coarsening, and equal-record-count reshaping. `init_points()` remains the
fixed-topology refresh operation; assigning `surface.resolution` alone is not
an edit to native topology.

```python
surface = ParametricSurface(lambda u, v: [u, v, u*u - v*v],
                            u_range=(-1, 1), v_range=(-1, 1),
                            resolution=(5, 7))
self.add(surface)
surface.set_resolution((21, 25))
surface.set_resolution((7, 9))
```

Geometry and normal-control points are freshly sampled by Atlas, not obtained
by interpolating the old coarse geometry. For stock solids, the existing
specialized constructor remains authoritative, including sphere true normals.
Colors, opacity, texture UVs and custom record fields are regridded by
Marionette's existing two-dimensional bilinear record interpolant over the
normalized UV chart. This preserves constant colors exactly and interpolates
paint spatially rather than treating the grid as a flat list.

The target retains its native handle, material resource, uniforms, children,
scene membership and updaters. Copies and saved states retain their own grids.
When the record count changes, old exported views detach under the ordinary
RecordBuffer protocol. Equal-count updates keep their views attached. Public
resolution and triangle indices agree with native topology after success.
Same-shape refreshes retain authored triangle ordering.

## Updating a textured surface and wireframe

A `TexturedSurface` reads its `uv_surface`'s stored geometry. Set that source's
resolution first, then update the textured object with the matching shape:

```python
source.set_resolution((21, 25))
textured.set_resolution((21, 25))
wireframe.init_points()
```

The wrapper never changes or evaluates its source implicitly. Its image bytes,
light/dark material pair and retained texture UV interpolant remain intact.
Wireframe refresh uses the existing native mesh builder and retains surviving
wire identities. Source, textured wrapper and wireframe are separate edits,
not one multi-object transaction.

## Failure and animation contract

Both old and requested grids must have at least two samples along each axis;
the new grid may contain no more than 65,536 records. Empty/strip grids and
indexed meshes are not admitted by this triangle-grid operation. Invalid
controls, failed or cancelled callbacks and invalid sampled coordinates publish
no candidate. Callback changes to destination geometry, recipe, family or owner
are detected and not overwritten; the callback's own side effects are not
rolled back. The native prepared update also checks retained placement before
publication. Custom pointlike fields are baked to world coordinates when the
old placement is discarded.

Like construction and `init_points()`, regeneration evaluates the recipe in its
own coordinates. Previous affine edits to sampled geometry are not reapplied.
Put the desired mapping in the recipe (or use an axes-owned surface plot).
Regridding is discrete: use it from an updater or between animations. The
`.animate.set_resolution(...)` spelling refuses explicitly; it cannot promise a
fractional UV resolution or that Transform's common grid will shrink. Use an
explicit Transform to a separately sampled target for a geometric morph.

`Stage::resample_surface_grid` is the native stored-record counterpart: it
resamples every field without evaluating a recipe or changing placement. Use
`SurfaceGridUpdate::for_geometry` followed by `apply_surface_grid_update` when
publishing fresh world-space geometry alongside regridded destination fields.
The latter separates preparation from publication and refuses stale records or
placement. These native APIs share the existing Transform alignment resampler.
