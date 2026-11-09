# Labels on live coordinate systems

`NumberLine.add_numbers()` places labels through the live `number_to_point`
and `get_number_mobject` protocols. Rotation, reflection, shear, three-dimensional
placement, and native point edits no longer cause labels to be laid out on a
reconstructed horizontal bounding box.

```python
line = NumberLine((-2, 2, 1)).rotate(PI / 2)
line.add_numbers([-1, 0, 1], direction=RIGHT, buff=0.2)
```

Labels remain upright at construction. `direction` and `buff` are world-space
placement arguments, just as in `get_number_mobject`; transforming the labeled
family afterward transforms the labels with the line. Ordinary `.animate`
placement uses the existing native record interpolation and scene clock.

The method returns a `VGroup` containing the actual `DecimalNumber` objects (or
an authored `get_number_mobject` override's VMobjects) and stores it as `numbers`.
Existing ticks, tips, annotations and previous label groups are not rebuilt or
removed. Repeated calls append a group, matching the Reference. Numeric values,
precision, signs, units and supported decimal text/style configuration are passed
to the existing native number/text implementation; no glyph layout is duplicated.
Default ticks, exclusions and subclass overrides retain their public dispatch.

## Axes, grids and complex planes

`Axes.add_coordinate_labels()` and `NumberPlane.add_coordinate_labels()` use
each current axis's native endpoints and numeric range. They do not infer a
chart from its bounding box. Prior labels, grid lines and unrelated annotations
therefore cannot alter the numeric scale used to place the next batch. Moving
one axis independently places its labels on that axis, not on another axis's
origin. The inherited `ThreeDAxes` method retains its existing x/y-only scope.

```python
axes = Axes((-2, 2, 1), (-2, 2, 1))
axes.apply_matrix([[1, 0.4, 0], [0.5, 1, 0], [0, 0, 1]])
axes.add_coordinate_labels([1, 2], [1, 2], font_size=24)
```

Both batches are prepared before either axis receives new labels. Each batch
lives under its actual axis and is available as `axis.numbers`; the method
returns the original chart. Existing axis classes, ticks, grids and annotations
are retained. Copied charts' label references point into their copied families.

NumberPlane's shared and per-axis configurations are merged from its actual
constructor inputs, including the y-axis settings omitted by the previous
label projection. Per-call precision, sign, direction, buffer, unit and supported
number styles override configured defaults. Native axis proxies need not be
NumberLine subclasses: an inert configuration holder delegates glyph creation
to the existing public number builder without changing proxy classes. An
authored `get_number_mobject` method on an axis retains dispatch.

Complex-plane labels retain their dominant-component rule: an input whose
imaginary component has greater magnitude labels the imaginary axis; otherwise
it labels the real axis. Complex values and real/imaginary pairs are accepted.
Imaginary labels use the native `i` unit, including the existing special handling
of `i` and `-i`. Unit settings no longer leak from an imaginary label into a later
real label in an interleaved request. The completed group is attached to the
plane and stored as `coordinate_labels`.

`skip_first=False` now honors the existing default-value helper's meaning and
includes the first tick. Previously the label method discarded this argument,
even though the helper already supported it. Default-value method overrides are
called rather than replaced. Both coordinate-label methods return the chart.

The ordinary native formatter accepts its supported decimal/style keywords;
unrouted options still fail explicitly. This change does not expand the native
axis constructor's accepted `decimal_number_config` keys.

All requested labels are prepared before attaching the group. An exception,
cancellation or detected mutation of the source axis geometry, family, range or
scene ownership leaves the candidate unattached. Authored side effects are not
rolled back. Reentrant labeling of the same line refuses; independent operations
and copies made during callbacks remain valid. Arbitrary custom `add` overrides
are outside this preparation guarantee.

For charts this check covers both axes and their owner, including changed ranges
or axis identities. Reentrant requests sharing an active axis refuse. The same
all-or-nothing preparation applies when the second axis's formatter fails after
the first batch has been prepared. It is not a transaction over arbitrary
external side effects or custom family-publication methods.

Inputs are bounded to 4,096 values/exclusions and one label batch to 1,048,576
native point records. Default tick ranges are admitted before `arange`. These
bounds do not preempt an authored callback that never returns or bound arbitrary
allocation performed by such a callback. Labels do not automatically track later
edits of the line's own points; transform their common family, or explicitly
create a new group and remove the old one in the scene's authored lifecycle.

Acceptance: `crates/fmn-python/tests/coordinate_labels.py`, including native
records, live views, callbacks, copied state and independent Y4M rendering at
one and four threads. The installed-wheel gate runs the same suite.
