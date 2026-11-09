# Persistent native animations

The wheel's animation-to-updater adapter can retain Choreo's existing native
animation driver for an animation with no Python interpolation
implementation. `Swap` and `CyclicReplace` no longer stop at the previous
native-driver refusal. No new renderer, geometry kernel, segment or clock is
introduced: the Scene updater's `dt` drives the retained leaf lifecycle.

```python
scene.add(left, right)
cycle_animation(Swap(left, right, path_arc=0, run_time=2, rate_func=linear))
scene.wait(5)
left.clear_updaters()
```

The helper preserves its pre-increment elapsed-time convention and its
interpolate-then-helper-update order. Noncycling effects finish once; cycling
ones wrap the existing alpha without repeatedly allocating native drivers.
Native finishing is distinct from scene cleanup: persistent effects do not
remove drawable roots or publish replacement targets. Targets and additional
participants are adopted by the existing lowering path, not added to the draw
list. With the current bridge's multi-object native specifications, the first
mobject remains the returned updater anchor; add the other participants to the
Scene as in the example.

Removing or clearing the installed updater aborts its retained driver without
jumping to the final pose. Clearing a copied updater does not cancel the original
controller. Copies and reentrant helper updates cannot double-advance its time.
Failures detach only the owned updater, release native animation state, and
preserve the original exception when cleanup also fails. Mobjects from different
Scenes are rejected before driver lowering.

Existing callback implementations remain the preferred route, preserving
live authored hooks and endpoint options. Native-only animations use the same
specification lowering as Scene.play. Their configuration is frozen when the
native driver is constructed; custom rates retain that lowering's current
sample-table semantics. Native-only nondefault final alpha and authored lifecycle
hooks without a Python interpolator are refused rather than silently discarded.
Native configuration freezes at activation, not pending registration. Edits to
native-only hooks/endpoints are checked again immediately before activation.

The installed wheel installs these helpers through the existing playback
adapter. The independent embedded initialization and native Rust API are
unchanged. Existing function aliases in compatibility modules are refreshed;
public Animation class objects retain their identities.

`test_native_animation_updater_protocol.py` extracts the production driver
factory and native-leaf wrapper, substitutes native storage/execution, and
exercises the real persistent controller. It is not native geometry or rendering
acceptance. The installed-wheel gate registers six additional real-extension
cases in `native_animation_updaters.py`. Execution requires a built extension;
no native-rendering, certification or performance claim follows from protocol
results.

## Deferred activation and mixed compositions

Native-only effects and AnimationGroup/LaggedStart/Succession protocols can now
be registered while their anchors are detached. The helper returns the original
anchor immediately, installs a pending updater, and does not construct a private
Scene or consume elapsed time. Normal Scene adoption followed by its first
updater boundary activates the same existing drivers. Zero-dt updates while
pending remain harmless. A nonzero detached update raises a clear adoption error
without discarding the pending registration; explicit removal cancels it.

```python
left = Square().shift(2 * LEFT)
right = Square().shift(2 * RIGHT)
group = Succession(
    Swap(left, right, path_arc=0, run_time=1, rate_func=linear),
    left.animate(run_time=1, rate_func=linear).shift(RIGHT),
)
background = turn_animation_into_updater(group)
self.add(background)
self.wait(3)
```

Composition timing, coarse interval crossings, and just-in-time child begin are
owned by the existing composition driver. Native and Python children are not
split into independent updater clocks. Authored group lifecycle hooks execute
with scoped Scene context, including nested groups with explicit roots. Prior
context is restored after every invocation. Ownership validation includes child
sources, targets, paths and additional native participants, not just draw roots.
The active group set is captured at activation, matching the driver's child
snapshot. Cancellation requested inside a callback detaches immediately but
unwinds drivers only when the current invocation returns, preventing reentrant
teardown of a group still running on the stack. Completed group driver references
are released without invoking scene cleanup or aborting already-finished children.

The ordinary callback-only detached leaf path remains immediate. A native
animation's geometry snapshots are taken on activation; constructing its pending
updater is not an early snapshot. Cycles retain the existing modulo-alpha and
child-driver semantics; this change does not implement arbitrary backwards
seeking or reset stateful Succession/PhaseFlow timelines on each cycle.

The additional protocol suite executes production group lifecycle, composition
driver and native-leaf/factory definitions with explicit arena/native-core and
constructor/interval doubles. It covers deferred start, both mixed same-object
succession directions, explicit roots, nested context and failure cleanup. Seven
additional installed-extension cases include a real Y4M background-animation
render whose luminance frames are independently decoded and checked for linear
motion. These cases require the built extension; registration is not a passing
native/rendering receipt. Throughput, full-workspace acceptance and certification
remain separate obligations.
