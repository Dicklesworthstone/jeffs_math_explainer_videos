# Live surface wireframes

`SurfaceMesh.init_points()` now refreshes a wireframe from its source surface's
**current native samples and unit normals**. Use it explicitly or from an
updater. It does not re-evaluate `uv_func`: regenerate the surface first when its
authored recipe has changed. `demo/python/live_wireframe.py` is a runnable example.

```python
mesh = SurfaceMesh(surface, resolution=(9, 7), normal_nudge=0.01)
mesh.add_updater(lambda wireframe: wireframe.init_points(), call=False)
self.add(surface, mesh)
self.play(surface.animate.rotate(PI / 4, axis=RIGHT))
```

The source may be detached, scene-bound, textured, or an ordinary native solid.
A wireframe in one scene may read a source in another without adopting it.
Authored `get_points` and `get_unit_normals` overrides still execute through the
existing native builder. Atlas remains the sole owner of UV interpolation,
normal nudging, smoothing and quadratic path construction.

## Stable scene objects

Generated lines are identified by their axis and ordinal within that axis.
Repeated refreshes update those same line objects rather than replacing or
appending duplicate families. Existing line styles, updaters, annotations,
scene membership and parent links are retained. Ordinary copies own independent
wire families; refreshing a copy never edits the original's lines.

The ordinary record-view protocol applies: a same-size update remains visible
through current live point views; a changed path size detaches old views safely.
Per-record style fields use the native preserve-order resize convention when
the source grid changes. A new native mesh supplies only replacement geometry,
not replacement styling for surviving wires.

Changing `mesh.resolution` and refreshing changes the number of wires. Surviving
axis/ordinal identities and their order are retained. Added wires inherit a
current same-axis style, or another existing line/root style when that axis is
new. Removed wires detach from the mesh; references held by callers remain
usable. Authored non-wire children stay in their existing relative order.
Both line counts may be zero, including an initially empty mesh that is populated
later. The native source grid can change via `become` or a surface morph.

This differs deliberately from the Reference's repeated `init_points`, which
accumulates extra lines. It also completes the former portal no-op initializer.
The mesh's constructor retains `resolution` and `normal_nudge` as editable
controls, and a native root style seed supports later growth from zero density.

## Publication and limits

The request is checked against the native 65,536-sample budget before source
callbacks or mesh construction. The complete native candidate is built and
validated before any existing wire geometry is changed. A sampling exception,
non-finite result, reentrant rebuild, or detected callback-time source/mesh edit
refuses publication and leaves the previous generated mesh in place. Effects
made by authored callbacks themselves are not rolled back; errors can be fixed
and the refresh retried. Active/locked wireframe animations must finish before
regeneration; animating the separate source surface is supported.

As with surface regeneration, refresh samples the source's current world-space
geometry. Independent transforms previously applied only to the wireframe are
not reapplied. No automatic updater is installed: explicit refresh or authored
updater order determines the relationship between surface, wireframe and clock.
This feature does not add arbitrary indexed-mesh remeshing, texture-image
crossfades, or a second surface/curve implementation.

Validation lives in `crates/fmn-python/tests/live_surface_mesh.py`, exercised by the
installed-wheel and embedded portal paths. Its analytic planar oracles check
station placement and normal offsets, while native animation output verifies
moving pixels, stable line identities and thread-count-independent Y4M bytes.
