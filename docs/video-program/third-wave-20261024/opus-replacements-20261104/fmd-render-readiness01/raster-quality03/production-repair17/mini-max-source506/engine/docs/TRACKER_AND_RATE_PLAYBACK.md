# Tracker-driven animation and authored easing

The installed Python wheel keeps callback-driven tracker values synchronized
with Marionette and evaluates authored Transform/composition rate functions
at the existing animation lifecycle's actual alpha. It does not add a second
frame clock or replace Choreo, the native geometry operations, or Lumen.

```python
from manimlib import Scene, ValueTracker, Dot, RIGHT, linear

class ParameterSweep(Scene):
    def construct(self):
        parameter = ValueTracker(0)
        dot = Dot()
        dot.add_updater(lambda mob: mob.move_to(parameter.get_value() * RIGHT))
        self.add(parameter, dot)
        self.play(
            parameter.animate(run_time=2, rate_func=lambda t: t * t).set_value(3)
        )

receipt = ParameterSweep().render(
    "parameter-sweep.y4m", resolution=(960, 540), fps=60, threads=4
)
```

## Native values, not an independent Python mirror

An ordinary native Transform already interpolates typed tracker state.
Previously, a Python-callback Transform changed the `uniforms["value"]`
mirror without changing the native state read by `get_value()`. A scene could
therefore finish an animation while dependent geometry remained unchanged.

The callback `Mobject.interpolate` adapter reads both native endpoints before
writing, interpolates the tracker, executes the existing record/style/path
operation, writes through the existing scalar/complex native setters, and
refreshes the mirror. This covers `ValueTracker`, `ComplexValueTracker`, and
`ExponentialValueTracker`, including trackers inside transformed families.
It reads native values even when endpoint mirrors are stale after a native
animation. Scalar and complex values do not pass through the float32 record
plane. A `locked_uniform_keys` entry of `"value"` prevents the tracker write;
locking point data alone does not.

Interpolation does not call an authored `set_value` override. In particular,
`ControlMobject.set_value` performs discrete validation and rebuilds controls;
those side effects do not belong in numeric state interpolation. Existing
geometry interpolation and authored `interpolate(...); super()` behavior
remain available. This change does **not** make arbitrary in-place writes to
`uniforms["value"]` into write-through native tracker views.

Exponential callbacks interpolate geometrically in the logarithmic domain,
using decoded native float64 values and NumPy. This avoids multiplying powers
that can overflow even when the answer is representable. It is not
bit-identical to native Choreo's encoded-lane/dmath implementation: extra
log/exp round trips can introduce rounding differences. Nonpositive or
nonfinite decoded endpoints, and a nonrepresentable interpolated result,
are refused before record or tracker writes. Finite native logarithmic states
that decode to zero or infinity require the native Transform route; this
adapter cannot reconstruct lost encoded information. These are standard-mode
semantics, not new certification evidence.

## Curves execute at the alpha that the animation uses

The older frontend precomputed authored rate functions on a fixed 30 Hz table
and interpolated that table during native execution. This could change the
curve at a different output FPS and at lagged/time-windowed sub-alphas. It also
invoked a global rate before begin even when every animation was a callback.

Authored leaf rates on the existing Transform protocol and regular
`AnimationGroup`, `LaggedStart`, and `Succession` timelines now select their
existing callback lifecycles. The callable is evaluated at the true
lifecycle/subobject alpha, including final-alpha interpolation. Catalog
functions and names retain their native route when no other authored behavior
requires callbacks. A catalog spelling such as `"linear"` is resolved to the
catalog callable when an authored Transform path needs the callback route.
Callable objects need not be hashable, and classification does not call their
`__call__` or equality methods.

A play-level custom rate is assigned to the top-level animations without a
redundant global lookup table when **all** top-level animations support the
callback path. A group's curve controls its timeline; it is not assigned to
its children again. Nested custom-rate groups acquire their root and Scene
context through the existing composition helpers and restore prior context
after success or failure. Builders are prepared once in argument order.

Mixed plays containing unsupported native kinds retain the prior global-rate
lowering, including its table approximation. Target-less `CyclicReplace` and
`Swap`, for example, are not silently converted into generic Transforms.
Their independent native-only leaf-rate behavior is not repaired here. This
is not a claim that every animation kind's arbitrary easing is complete.

## Installation and execution boundaries

`fmn_python.playback.install_scene_playback` installs both adapters after the
wheel's existing shared animation definitions. Existing exported classes and
qualified aliases retain their identities. Reinstallation preserves later
authored replacements. Consumers loading the native extension directly can
explicitly call that same installer; the independent embedded initialization
and the native Rust API are unchanged.

The new protocol tests extract production definitions and use explicit
substitute storage, lifecycle setup, and a modeled execution clock. They
exercise tracker synchronization, fixed-table aliasing, exact family sub-alpha
calculation, builder preparation, group context ownership, mixed-native
fallbacks, and the combined playback installer. They are not native renderer
or full-workspace acceptance.

The installed-wheel runner also registers eight tracker cases and six live
rate cases. The latter includes real 60 FPS Y4M output and observations of
native tracker-driven geometry, along with nested composition, error recovery,
and catalog/callable cases. These require a built `manimlib` extension; a
missing import is not a skipped success. No throughput, memory, cross-platform
bit-equality, or certification improvement is claimed. Callback-selected
animations can have additional interpreter overhead that remains unmeasured.
