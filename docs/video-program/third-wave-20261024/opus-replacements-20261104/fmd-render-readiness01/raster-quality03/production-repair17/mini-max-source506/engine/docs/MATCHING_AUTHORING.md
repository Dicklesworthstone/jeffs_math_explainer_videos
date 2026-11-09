# Authoring matching transitions

The optional Python portal plans `TransformMatchingParts`,
`TransformMatchingShapes`, `TransformMatchingStrings`, and `TransformMatchingTex`
as real `AnimationGroup` instances. Public class/qualified-import identities
are retained. Geometry comparisons still use Marionette/Chisel's native
`has_same_shape_as`; text uses Scribe's native UTF-8 source-span parts.

```python
from manimlib import *

class ArcMatch(Transform):
    def __init__(self, source, target, **kwargs):
        kwargs.setdefault("path_arc", PI / 2)
        super().__init__(source, target, **kwargs)

class Rename(Scene):
    def construct(self):
        source = Text("foo + foo").shift(LEFT)
        target = Text("bar + bar").shift(RIGHT)
        self.add(source)
        self.play(TransformMatchingStrings(
            source, target, key_map={"foo": "bar"},
            match_animation=ArcMatch,
            mismatch_animation=ArcMatch,
        ))
```

`add_transform`, `find_pairs_with_matching_shapes`, and `matching_blocks` are
real planning hooks. Factories return `Animation` objects; they are not probed
then replaced with native stock transforms. Interpolation/lifecycle overrides
execute on the shared scene callback boundary. Unchanged leaf animations keep
their existing native kernels. Child easing is not applied again by the outer
matching group.

Explicit pairs claim first. Shape matching then considers remaining pieces;
unmatched pieces fade toward/from the opposite group's center. A prior whole
family claim prevents a second transform of its descendants. Text renames can
cover several glyphs, repeated occurrences, and Unicode. Claimed positions stay
masked rather than being deleted: two separated strings never become adjacent
merely because the intervening part was matched elsewhere. Tex keeps semantic
key matching and never falls back to pairing unrelated equal-looking glyphs.

The planner rejects foreign-family pairs, cross-scene operands, invalid UTF-8
spans, duplicated explicit string claims, and keys splitting native grouped
parts. Planning is bounded to 65,536 parts/pairs and 1,048,576 default shape
comparisons; explicit pairs reduce the remaining shape-search space. These are
resource refusals, not silent partial results.

Successful cleanup delegates child remover/replacement cleanup, removes the
source and temporary group, then publishes the target. A child cleanup/removal
failure propagates and does not publish the target or retry a partial cleanup.
The existing shared execution owner handles aborted native/Python leaves and
transient state; no scene rollback or arbitrary callback transaction is claimed.

`_native_kind` and existing `_native_params` remain available as legacy
inspection metadata. Playback of these matching classes follows the authored
child plan rather than independently generating another native matching plan.
Inferred string blocks are not inserted into public `matched_pairs`, preserving
the distinction between explicit claims and derived `source_keys`/`target_keys`.

## Validation boundary

The protocol suites test planning/ownership with explicit native-object and
Scribe doubles. The string suite AST-selects the actual shared block planner.
They are not native-render evidence. `matching_transform_semantics.py` contains
eight existing installed-native cases. `matching_authoring.py` adds four cases
for authored block dispatch, native key metadata, invalid-span refusal, callback
recovery, and actual changing PNG frames compared at one/four threads; both are
registered in `scripts/check_portal_runtime.sh`.

Native and installed-wheel execution require a newly built portal. No global
parity, certification, or performance gate is closed by this feature alone.
