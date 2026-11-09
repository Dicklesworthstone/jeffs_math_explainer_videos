# Source-derived curve authoring

`CurvesAsSubmobjects` and `DashedVMobject` keep the actual subclass instance,
its native record schema, hook-created root geometry and decorations. Their
constructors run the normal data, point, uniform and color hooks before adding
source-derived children, preserving the Reference's construction order.

## Native geometry and authored sources

Ordinary curve decomposition uses Atlas's native shared-anchor splitter.
Overriding `get_points`, `get_bezier_tuples_from_points`, or
`get_bezier_tuples` selects the corresponding public Python protocol instead;
each returned quadratic triple is copied into native VMobject records. The
qualified `manimlib.mobject.types.vectorized_mobject.VMobject` factory remains
live. A generator may reuse its array: each yield is captured before requesting
the next. Each resulting part receives the source's style.

Dashes retain Atlas/Chisel's true-arclength placement (BN-03). The native interval
query chooses the cuts, and each public `source.get_subcurve(a, b)` result keeps
its actual object identity, custom record columns, children and paints. No
second curve evaluator, arclength solver or animation clock is introduced.

## Rebuilding a live annotation

```python
source = Circle(radius=1, color=BLUE)
dashes = DashedVMobject(source, num_dashes=12)
marker = Dot(2 * RIGHT)
dashes.add(marker)
self.add(dashes)

source.stretch(2, 0)
dashes.num_dashes = 18
dashes.init_points()           # Replaces generated dashes, not marker.
self.play(dashes.animate.shift(UP))
```

Explicit `init_points()` reads the current source and, for dashes, the current
`num_dashes` and `positive_space_ratio`. It replaces the generated run in place
without clearing the receiver's records, decorations, saved state or scene
identity. New children sample current source paints; this does not restyle the
receiver's own root records. `animate.init_points()` uses the existing target
copy and native transform path. No automatic source updater is installed.

Generated-child references are copy-remapped and cycle-collectable. Copies made
by native-family alignment remain generated, so a later rebuild removes padding
rather than accumulating it as decoration. Copies of generated parts which are
added back to that same owner carry this generated role too; independently
created decoration objects retain their own identities. `become` changes data,
not the receiver's construction recipe; rebuilding then uses that original
recipe unless the caller changes its fields.

## Refusals and failure ownership

Factory outputs must be independent, detached VMobject families. Returning the
receiver, its existing children, a source-family member, a scene-owned object,
duplicate roots or a cycle is refused before linking the generated batch. Curve
triples and final geometry must be finite. Output is bounded to 65,536 family
members and 16,777,216 records; native dash counts remain limited to 4,096 and
positive counts require a drawn ratio in `(0, 1]`. Non-positive dash counts keep
the existing empty-pattern behavior.

Sampling failures retain the previous generated children. Reentry, active
animation locks and receiver/recipe changes made by callbacks refuse publication.
Authored callback effects are not rolled back; only the prepared replacement is
withheld. There is no promise of arbitrary concurrent scene mutation.

`tests/native_derived_curves.py` exercises the actual native record, geometry and
rendering surfaces, including source factories, reused generator arrays, copy and
alignment roles, failure atomicity and three-frame output comparisons at 1/4/16
workers. The tests are registered in the installed runtime script and embedded
native harness. A test registration is not a claim that a fresh wheel, complete
workspace or cross-platform certification has passed.
