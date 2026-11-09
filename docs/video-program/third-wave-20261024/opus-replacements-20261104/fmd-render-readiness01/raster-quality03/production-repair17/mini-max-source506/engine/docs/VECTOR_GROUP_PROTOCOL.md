# Vector groups are Groups

`VGroup` follows the pinned Reference's `VGroup -> Group -> VMobject -> Mobject`
lineage. Existing qualified aliases and library subclasses retain their class
identities. Code that uses `isinstance(obj, Group)` can now handle vector groups,
axes, planes and vector solids as collections without treating ordinary vector
leaves as collections.

Construction uses the cooperative Group and VMobject initialization protocol.
Data, points, uniforms and color hooks execute on the receiver; Group's public
`_ingest_args` calls public `add` for the actual children. Duplicate identities
are deduplicated by the same native-backed addition path used after construction.
Iterable inputs are expanded once before addition. A failing generator does not
install a prefix of its children, although authored external effects are not
rolled back.

After ingestion, a nonempty group's own uniform mapping receives the first
child's uniform values, as in the Reference. That includes a hook-created first
child. The live mapping's owner is not shared and other children are not
restyled. Empty groups retain their own uniforms.

`group + vmobject` calls `group.add(vmobject)` and returns its result. It mutates
the existing group; it does not concatenate into a new object or copy either
operand. Thus `group += vmobject` retains the group identity. Authored `add`
overrides and native duplicate, scene-ownership and cycle validation remain in
force. Non-vector operands are rejected.

```python
from manimlib import Group, VGroup, Square, Circle

shapes = VGroup(Square())
alias = shapes
shapes += Circle()
assert shapes is alias and isinstance(shapes, Group)
```

The shared initializer installs this class protocol before playback classifiers
record their shipped method baselines. It replaces the incomplete constructor;
it does not stack another wrapper around it. No renderer, geometry algorithm,
copy implementation, native storage boundary or clock is introduced.

`vector_group_protocol.py` tests actual native groups, copies, public hooks,
iterable failures, uniform inheritance, live scene addition and animated PNGs
against independently placed leaves at one and four workers. The old
`fm-16u6-vgroup-not-group` structural-diff exemption is removed; all other
exclusions remain unchanged. A retained-wheel test with the changed Python
module is not an exact-tree wheel build or a full-workspace/parity certification.
