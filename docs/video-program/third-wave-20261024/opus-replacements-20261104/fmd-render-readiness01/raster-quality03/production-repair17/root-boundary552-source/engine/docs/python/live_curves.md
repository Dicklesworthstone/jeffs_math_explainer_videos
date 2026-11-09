# Live parametric curves and scalar graphs

`ParametricCurve.init_points()` rebuilds the current `t_func` over `t_range`,
using the current `epsilon`, `discontinuities` and `use_smoothing` controls.
`FunctionGraph` inherits this behavior. Refresh returns the same object and
**replaces** its sampled geometry; repeated calls do not append duplicate paths.
This deliberately corrects the Reference's append-on-refresh behavior.

```python
level = ValueTracker(0.25)
path = ParametricCurve(lambda t: (t, level.get_value() * t, 0),
                       t_range=(-1, 1, 0.1), use_smoothing=False)
path.add_updater(lambda current: current.init_points(), call=False)
self.add(path)
self.play(level.animate.set_value(1), run_time=2)
```

The native Atlas sampler owns point evaluation, discontinuity exclusion, sample
budgets and smoothing. The adapter does not implement a second path sampler.
A refreshed curve is usable by the ordinary path query, rendering, copying and
animation APIs. Parameter intervals may grow or shrink, sample density may
change, and discontinuity-separated paths may be introduced or removed.

## Recipe and ownership

`t_func` and `t_range` are the authoritative refresh recipe, including on a
`FunctionGraph`. The scalar constructor installs a parameter function that
captures its supplied scalar callable. Changing state read by that callable is
observed; replacing the informational `.function` or `.x_range` attribute alone
does not replace the parameter recipe. To change it explicitly, assign `t_func`
and/or `t_range` before refresh. As at construction, the recipe returns scene
coordinates. Prior shifts, rotations and other affine geometry edits are not
reapplied automatically.

Refresh retains native object identity, scene membership, children, updaters,
uniforms, record styles and saved states. Same-size point arrays retain their
live generation; resized arrays detach old live views under the existing view
protocol. Resizing uses the standard point setter's style propagation, not an
inferred correspondence between old and new parameter samples. Copies share
the authored callable just as ordinary Python function closures do, but updating
a copy's geometry does not update the original's records.

## Failure and animation boundaries

Construction completes on a detached candidate before target publication.
Invalid requests or samples, callback errors and cancellation leave target
geometry unchanged. Original exceptions propagate; the next valid refresh may
succeed. A callback that changes the target's records, recipe, schema, family or
scene binding causes publication to be refused. Its own side effects are not
rolled back. A callback that never returns cannot be preempted here.

Recursive refresh and refresh during direct animation/data locking are refused.
Animate a separate tracker and refresh from an updater instead. A graph with an
active `bind_graph_to_func` updater already has a geometry owner; call
`axes.unbind_graph_from_func(graph)` before switching to parameter refresh.
Custom schemas with additional pointlike fields are not regenerated because a
plain native path does not define those fields.

`crates/fmn-python/tests/live_curves.py` includes real native geometry, ownership,
failure recovery and independent analytic-path Y4M comparisons at 1/4 threads.
