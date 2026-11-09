# Live function graphs

`axes.bind_graph_to_func(graph, func, jagged=False, get_discontinuities=None)`
updates a VMobject graph on the existing scene updater clock. Direct binding
accepts a vectorized callable: it receives one detached 1-D array of x samples
and returns either one real value per sample or a scalar constant. Binding
installs the updater without evaluating the function; the first ordinary
updater tick computes the new geometry.

```python
pole = ValueTracker(0)
graph = axes.get_graph(lambda x: x)  # Establish the full sampling interval.
axes.bind_graph_to_func(
    graph,
    lambda xs: 1 / (xs - pole.get_value()),
    get_discontinuities=lambda: [pole.get_value()],
)
self.add(graph)
self.play(pole.animate.set_value(1))
```

## Behavior correction: dynamic discontinuities

The pinned Reference and earlier portal binding insert discontinuity neighbors
into a fixed-length array and discard its rightmost samples. Repeated updates
therefore shrink the represented domain and draw connecting segments across
breaks. That behavior is deliberately corrected, not reproduced.

A binding freezes its baseline x samples once. Each update subtracts the
current discontinuity exclusion bands, adds their boundary samples without
truncation, and builds independent native subpaths. Exclusion half-width is
`max(graph.epsilon, 1e-6)`, or `1e-6` when absent. Overlapping bands merge; bands
can touch the endpoints or hide the entire domain, which can recover on a
later tick. Smoothing runs separately on each surviving segment. It cannot
smooth across a discontinuity.

The function and public coordinate conversions run before native corner-path
construction, smoothing and joining. Only then does the ordinary `graph.set_points` / native record-resize protocol
replace geometry. Callable, shape, nonfinite-value and native construction
errors preserve the last complete graph. A failure in an authored override of
the final `set_points` operation retains that override's normal partial-effect
semantics. The adapter does not roll back arbitrary external callback effects.

The current axes' public `p2c`/`c2p` methods define coordinates. Moving or
rotating the axes does not silently turn them into fixed original pixel
coordinates. The graph keeps its proxy, styles, uniforms, children, and author
updaters; only its owned path changes. Vertex styles follow the existing native
order-preserving resize policy when the number of samples changes.

Rebinding replaces only the previously installed graph callback, preserving
other updaters and the baseline sample interval. Explicitly call
`axes.unbind_graph_from_func(graph)` to stop following the function without
changing the last geometry. Copied graph updaters retain their callable/axes
closure dependencies, but receive and mutate the copied graph, not the original.

At most 65,536 evaluation samples and 1,024 declared discontinuities are
admitted per update. Nonfinite output is an error, not an implicit clipping or
silent hole; declare singularities explicitly. Generated coordinates must fit
finite f32 records. Native geometry, smoothing, buffer generations and rendering
remain the authorities. This is not a new renderer or certification claim.

## Scalar construction and array binding

`axes.get_graph(function, bind=True)` retains `get_graph`'s scalar contract,
including `math.sin` and functions with scalar `if` branches. The initial graph
still comes from the existing native `ParametricCurve` sampler. Later updater
ticks call that same scalar function for each sample. The public binding hook
receives the original callable; subclass overrides are not silently bypassed.
Direct `bind_graph_to_func(graph, func)` retains its vectorized contract.
Neither path tests a callable in both modes or retries a failed callback.

Ranges accept two or three values, including NumPy arrays. The endpoints must
increase; step and sampling density must be positive and finite. Excessive
sampling and invalid discontinuities fail before initial function evaluation.

```python
import math
amplitude = ValueTracker(1)
wave = axes.get_graph(lambda x: amplitude.get_value() * math.sin(x), bind=True)
self.add(wave)
self.play(amplitude.animate.set_value(2))
```

## Queries after unbinding or direct path editing

`axes.i2gp(x, graph)` and `input_to_graph_point` continue to use an attached
`underlying_function` when present. A direct vectorized binding supplies the
scalar-query adapter explicitly, so its array-only function still receives an
array even when queried at one x coordinate.

Without that function metadata, lookup searches the graph's current native
quadratic curves, not its original formula or an out-of-range global alpha.
Curve parameters always stay in `[0, 1]`; axis values never become curve
parameters. Null subpath joins are skipped, so a missing x interval returns
`None` instead of an invented point across a discontinuity. Endpoints, decreasing
x direction, translated/rotated axes, and live record edits are supported.

This lookup targets function graphs: each searched quadratic is assumed
continuous and x-monotone in the coordinate system. When multiple curves cover
the same x, the first in path order is selected. A vertical curve at exactly
the requested x returns its first endpoint. This is not a general curve/line
intersection solver for loops or nonlinear coordinate maps. Query results are
bounded by native f32 precision; malformed/nonfinite data or a nonconvergent
bracket raises rather than returning a fabricated success.

The runnable example is `demo/python/live_graphs.py`. Native acceptance lives in
`crates/fmn-python/tests/live_graphing.py` and is registered in the installed
portal gate. The separate protocol suites explicitly double native geometry;
they do not qualify native rendering or cross-platform reproducibility.
