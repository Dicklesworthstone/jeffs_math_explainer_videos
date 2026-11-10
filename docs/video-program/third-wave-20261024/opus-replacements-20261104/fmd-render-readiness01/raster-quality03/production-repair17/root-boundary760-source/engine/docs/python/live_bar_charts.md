# Live signed bar charts

`BarChart.change_bar_values(values)` now updates bars through zero and negative
values without losing their baseline. The existing method and class identities
are retained. It returns `None`, updates the supplied prefix, and leaves bars
beyond that prefix unchanged. Supplying more values than bars is an error rather
than silently ignoring data.

```python
chart = BarChart([0, 0.5, 0.25], label_y_axis=False)
self.add(chart)
self.play(chart.animate.change_bar_values([0.75, 0, -0.5]))
self.play(chart.animate.change_bar_values([-0.25, 0.75, 0.5]))
```

The original extent-stretch implementation could not grow a collapsed bar and
used bounding boxes that displaced signed bars. The replacement derives each
rectangle from its retained bottom edge and the live chart y-axis, using the
native Rectangle's existing point layout and native fixed-size record writes.
There is no epsilon-height workaround, saved last-nonzero height, or alternate
rendering engine.

A value of zero collapses exactly onto the baseline; the next update can grow
in either direction. Negative values extend opposite the chart's positive y
axis. Whole-chart rotation, reflection, shear, and nonuniform scaling are
respected. Individual bar baseline translations and widths are retained.
Changing `max_value` changes the data-to-height scale at the next update. Height
comes from the actual axis geometry, not the chart's screen-space bounding box
or a later assignment to its informational `height` attribute.

## Construction and ownership

Initial signed values use the same baseline convention. With `max_value=None`,
the initial scale is the largest absolute value, or one for all-zero data. This
is intentionally different from the old signed `max(values)` normalization.
Explicit `max_value`, width and height must be finite and positive. Bar values
and generated point records must be finite and f32-representable. At most
4,096 bars are admitted; iterable values are bounded before allocation.

Updates preserve bar identities, their children, scene membership, updaters,
record styles, and scene time. The nine-point native rectangle topology does
not resize, so scene-bound live views keep following the same records. Binding
an initially detached chart to a scene is still a separate native ownership
transition with its ordinary view rules. Copies and saved states do not share
mutable point records. Ordinary `.animate.change_bar_values(...)` uses the
existing record interpolation lifecycle, including zero endpoints. Tracker
updaters may also call the setter directly.

All supplied values and all target point arrays are prepared before the first
bar is written. Conversion errors, nonfinite values, unsupported point topology,
and overflow do not partially publish the input prefix. Recursive calls to the
same setter are refused. Arbitrary effects performed by custom conversion or
subclass methods are not rolled back.

## Scope

This updates the chart's original rectangular bars, not arbitrary replacement
shapes or schemas with extra pointlike fields. It does not insert/delete bars,
change category count, automatically move bar-name labels, or regenerate axis
ticks/labels when `max_value` changes. Those remain independent annotations and
retain their authored locations. It does not preempt a nonreturning iterator or
callback. A deliberately collapsed y-axis remains a collapsed coordinate chart;
restore its geometry before expecting nonzero chart heights.

`crates/fmn-python/tests/live_bar_chart.py` covers native zero/sign transitions,
affine geometry, live views, ownership, copying, restore, validation, builders,
and independent signed-rectangle Y4M comparisons at one and four threads.
