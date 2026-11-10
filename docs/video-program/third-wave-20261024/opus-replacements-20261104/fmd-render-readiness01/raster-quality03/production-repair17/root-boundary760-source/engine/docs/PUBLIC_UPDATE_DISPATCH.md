# Public update dispatch

`Mobject.update(dt, recurse=True)` traverses children through their public
`update(dt)` methods, child-first. An override accepting only `dt` is valid.
Its `super().update(...)` call chooses the base traversal's elapsed time and
recursion scope; omitting that call omits the base updater work for its subtree.
Shared children retain path-wise visits. Each node snapshots its child list and
then, at its own turn, its updater list, so edits have a defined visibility point.

Host callbacks complete before that invocation's native updater phase. Ordinary
families retain one native recursive crossing. When a child authors the public
protocol, the native phase uses self-only calls for the visited base operations,
with their chosen time values and suspension ancestry; it does not recursively
update every descendant again at each ancestor. No Stage borrow spans a Python
callback. An explicit update inside an updater is a separate invocation. A
host-only `_update_python_family` call does not start a native phase. Failures
preserve the original exception and discard pending native work, not authored
side effects. Invocation state is thread-local and is not copied or pickled
with a mobject.

## Public scene updates

`Scene.update_mobjects(dt)` updates the actual `scene.frame` first, followed by
the drawable roots captured at the start of the call. Camera callbacks can edit
scene membership, but those changes affect the next root snapshot. The frame
is not part of the drawable Stage and is not inserted into it for this purpose.
Camera suspension does not suspend unrelated drawable roots.

`Scene.should_update_mobjects()` checks the camera and the recursive public
`has_updaters()` query on each root; callbacks on deep descendants are not lost.
`always_update_mobjects` short-circuits that query. The existing public
`update_frame(dt)` retains its order: advance scene time, update objects, then
perform its normal capture/interaction policy, including its skip-mode policy.

These rules apply to direct public updates and animation-owned helper/snapshot
updates. Native `Scene.play` and `Scene.wait` also dispatch this public phase
when an authored scene, camera or drawable update protocol requires it. Stock
objects keep their native updater route. The same rational frame driver still
owns animation ordering, time advance, event delivery, capture and emission.
The independent Studio idle/presenter loop is not changed by this dispatch.

## Regression coverage

`crates/fmn-python/tests/public_update_dispatch.py` covers recursive overrides,
selected time/recursion, native-phase ordering, snapshots, suspension, exceptions,
reentry, copying/collection, animation helpers, camera-only update detection,
public scene time, and captured frames. Animation-helper Y4M and explicit public
frame PNG sequences are compared with independent controls at 1, 4 and 16
renderer workers. The installed runtime gate invokes this suite.

## Shared scene phase

A public `Scene.update_mobjects(dt)` invocation now defers its native callbacks
until the camera and every selected drawable root have finished their host
callbacks. A native callback on the first root therefore observes edits made
by a later root. A host exception abandons the pending native phase, while
retaining authored side effects and the original exception. Root overrides
may omit `super()`, select nonrecursive updates, or change `dt`; their choices
also determine the native work. Explicit updates issued inside a callback
remain independent invocations.

The private scheduler handoff `_fmn_dispatch_public_scene_update(dt)` returns
true only when it has executed this complete phase. A native caller consuming
that result must not subsequently run the default scene updater pass. Static
protocol inspection does not execute authored getters and compares against the
completed production initializer; ordinary objects retain the native route.
The native caller consumes that result with a per-frame completion marker.
Markers are valid only after prepare and once per frame; they cannot leak into
an aborted or subsequent play. Wait's zero-dt prologue and play's final zero-dt
phase have corresponding explicit handoffs. Externally updated segments are
marked stateful and do not retain a pure-frame reconstruction snapshot.

`scene_update_phase.py` exercises real native record probes, cross-root
ordering, failure disposal, authored scope, shared paths, late overrides,
descriptor-safe admission, and collection after failure.

`native_scene_update_dispatch.py` exercises the actual native play/wait loop,
including camera-only plays, per-frame override discovery, exceptions, skip
mode, stop conditions, true native probe counts, and rendered controls at
1/4/16 workers. `external_scene_updates.rs` independently tests phase validation,
exact clock/capture values, native callback counts, abort isolation and purity.
