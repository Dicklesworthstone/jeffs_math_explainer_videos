# Checkpoint resume identities

`render_scenes(..., checkpoint=..., resume_key=..., resume=True)` reuses a
completed publication only when its input plan and artifact inventory match.
This is explicit, uncertified recovery, not a render cache or a source-code
identity claim. Continue to change `resume_key` whenever source, assets or
other authored inputs change.

## Version 2

A checkpoint's scene identity contains **separate module and qualified class
name fields**. Two `Demo` classes from different modules are not interchangeable;
module `a.b` / class `C` is also distinct from module `a` / nested class `b.C`.
Classes without a usable module/name are rejected before scene execution.

Plan comparison uses canonical JSON rather than Python container equality.
Constructor parameters `true`, `1` and `1.0` are distinct, as are `0.0` and
`-0.0`, including inside nested lists or mappings. Mapping key order does not
invalidate an otherwise identical plan. Matching native receipts and completed
artifacts continue to be reused without invoking a constructor or rewriting
its output.

Version-1 journals cannot establish these identities and are refused rather
than silently upgraded. Keep the existing journal and completed artifacts;
start a fresh batch with a new checkpoint path and fresh output destinations.
The rejection does not remove or overwrite old outputs. Do not simply change
a version field in an old journal: that does not supply the missing identity.

## Frozen constructor inputs

Checkpointed constructors receive independent copies of their admitted JSON
values. Mutating a nested parameter inside one scene or in `on_result` cannot
change a later scene while the journal still identifies its original inputs.
The journal is never handed to a constructor. Repeated references to the same
list or mapping have JSON **value** semantics, not shared Python identity.
Batches without a checkpoint retain ordinary Python reference semantics.

Checkpoint arguments accept `None`, booleans, integers, finite floats, strings,
lists, and dictionaries with string keys, using those exact built-in types.
Tuples, non-string nested keys, custom subclasses and circular structures are
rejected before execution rather than silently serialized as different inputs.
Use lists for array-like inputs such as nested camera resolutions. This is an
opt-in checkpoint restriction, not a restriction on ordinary scene construction.

The plan carries `constructor_inputs: "json-values-v1"` to identify this
execution contract. Earlier plans without that marker are refused: their
receipts might describe inputs changed through mutable aliases. Preserve them
and use fresh checkpoint/output destinations just as for a version-1 journal.

## Regression coverage

`test_batch_checkpoint_identity.py` exercises admission, canonical numeric
types, module/name collisions, legacy rejection and no-write failure behavior.
`checkpoint_identity.py` exercises actual native PNG publication and resume,
including a scene whose output changes with the sign of zero and palette
mutations across real renders. `test_batch_checkpoint_inputs.py` checks frozen
inputs, type admission and preserved non-checkpoint behavior. All three are
part of `scripts/check_portal_runtime.sh`.
