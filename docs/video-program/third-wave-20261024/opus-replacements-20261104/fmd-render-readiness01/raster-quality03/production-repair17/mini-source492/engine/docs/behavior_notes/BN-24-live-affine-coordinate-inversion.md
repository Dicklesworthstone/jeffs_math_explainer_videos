# BN-24: Invert live affine axes instead of projecting onto each axis

**Status:** Draft.

## Corrected behavior

`Axes.point_to_coords` and its `p2c` alias now invert the actual live affine
coordinate chart. The previous implementation independently projected a world
point onto each axis. That only inverts orthogonal axes: after the shear
`[[1, 1, 0], [0, 1, 0], [0, 0, 1]]`, `p2c(c2p(1, 2))` could return `(3, 2.5)`.
The error also changed the sample domain retained by a live graph binding and
mislocated geometry-only graph queries.

The inverse reads the existing native axis endpoints and current axis ranges.
It neither creates geometry nor owns another point buffer or renderer. A
scaled dual basis handles nonuniform scale, reflection, shear and rotation;
normalizing each axis avoids squaring very large or very small unit lengths.
Arithmetic uses a fixed scalar order without a BLAS solve or cached matrix.

For two-dimensional axes embedded in three-dimensional space, an off-plane
point maps to its orthogonal projection onto the chart plane. For three axes,
the full three-dimensional affine inverse is used. Independently translated
axes use the same effective origin as the existing forward mapping rather
than assuming the axes still intersect. Native edits and copied axes are read
on every call, so no stale inverse survives a geometry change.

Single points return a tuple of coordinates. Arrays with shape `(..., 3)` retain
the existing tuple-of-arrays convention, including empty `(0, 3)` arrays. The
results do not alias input arrays. Numeric conversion protocols remain valid;
complex/non-finite points, invalid ranges, collapsed directions and singular or
numerically degenerate charts fail explicitly rather than producing NaNs or
plausible but incorrect coordinates. A normalized triple product no larger than
64 times f64 machine epsilon is considered numerically degenerate.

## Dispatch and ownership

The original `Axes` class, import aliases, method name/signature and normal
subclass dispatch remain intact. `ThreeDAxes`, `NumberPlane` and `ComplexPlane`
inherit the correction, including complex number/point conversion aliases.
An authored inverse still overrides the inherited method. An authored forward
mapping without an inverse keeps the previous inherited inverse dispatch; it is
not silently fitted to an affine model. Authors of nonlinear charts must supply
an appropriate inverse.

Queries leave native records, exported views, styles, scene membership, updaters
and the scene clock unchanged. Existing graph binding and graph-query code uses
the corrected inverse; no alternate graph updater or sampling clock is added.

## Limits and regression coverage

This correction is for the live Python axes adapter. It does not replace native
Rust coordinate builders, add a nonlinear solver, recover precision already lost
in stored geometry, or promise a reliable inverse for an ill-conditioned chart.
Small floating-point differences on ordinary orthogonal charts are expected.

`crates/fmn-python/tests/coordinate_mapping.py` exercises sheared and tilted
charts, batched arrays, independent axis motion, extreme unit scales, copied
axes, invalid inputs, subclass dispatch, complex aliases and downstream graph
bindings/queries against the real native extension. Its animated graph is
compared with an independently authored world-coordinate path at one and four
threads; the Y4M bytes must agree exactly. Negative controls using the original
inverse reproduce the coordinate, binding-domain and graph-query failures.
