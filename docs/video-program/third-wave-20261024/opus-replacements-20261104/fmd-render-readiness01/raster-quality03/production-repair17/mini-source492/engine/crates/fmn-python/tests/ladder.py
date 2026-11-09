"""fm-zoi acceleration-ladder acceptance tests (§15.2 Rev 4).

Proves, per rung, on each rung's declared class:

- bit-equality vs rung 0: every rung produces bit-identical RecordBuffer
  state on every frame for the corpus (position shifters, scale pulsers,
  color ramps);
- crossing reduction: the batched group replaces per-updater and
  per-field crossings with one updater crossing and one batched dirty
  crossing per frame;
- explicit opt-in: rung objects are distinct inspectable types with loud
  construction errors; nothing substitutes silently.

The arithmetic contract (fmn-anim's ladder module): Python reference
updaters compute in float (binary64) and store through set_field; the
rung-1 callbacks compute in float64 NumPy and store into the f32 views.
Both are the same correctly rounded operations plus exact floor/abs, so
the bits match. No transcendental functions appear anywhere.
"""

import gc
import importlib
import math
import weakref

import numpy as np

import manimlib
from manimlib import _ArrayUpdater as ArrayUpdater
from manimlib import _BatchedUpdater as BatchedUpdater
from manimlib import _NativeUpdater as NativeUpdater
from manimlib import Mobject, Scene

bridge_errors = importlib.import_module("manimlib.exceptions")

DT = 0.25
FRAMES = 6
N_POINTS = 4
VELOCITY = (0.1, -0.2, 0.3)
PULSE_CENTER = (0.0, 0.0, 0.0)
PULSE_AMPLITUDE = 0.5
PULSE_PERIOD = 2.0
RAMP_FROM = (0.0, 0.0, 0.0, 1.0)
RAMP_TO = (1.0, 0.5, 0.25, 1.0)
RAMP_PERIOD = 1.0


def triangle(u):
    frac = u - math.floor(u)
    return 1.0 - abs(2.0 * frac - 1.0)


def fresh_mobject():
    mob = Mobject()
    mob.resize(N_POINTS)
    for i in range(N_POINTS):
        mob.set_field("point", i, [i * 1.0, -i * 0.5, 0.25])
    return mob


def batched_cycle_observers(callback_owns_anchor):
    anchor = fresh_mobject()
    member = fresh_mobject() if callback_owns_anchor else anchor
    if callback_owns_anchor:
        def callback(views, dt, owner=anchor):
            assert owner is not None
    else:
        def callback(views, dt):
            pass
    anchor.add_updater(BatchedUpdater([member], callback), call=False)
    return weakref.ref(anchor), weakref.ref(member), weakref.ref(callback)


# Ordinary attachment can form a cycle through either Rust-owned field.
# Dropping the user's last references must collect it without clear_updaters
# or a test-worker teardown reaching into production objects.
for callback_owns_anchor in (False, True):
    cycle_observers = batched_cycle_observers(callback_owns_anchor)
    gc.collect()
    gc.collect()
    assert all(observer() is None for observer in cycle_observers), (
        "BatchedUpdater retained an unreachable owner",
        callback_owns_anchor,
    )


def scene_state(scene):
    return [m._data_array(False).tobytes() for m in scene.mobjects]


# ---------------------------------------------------------------- rung 0
# Reference Python updaters against the declared arithmetic contract.
def make_shift():
    def update(m, dt):
        for i in range(N_POINTS):
            x, y, z = m.get_field("point", i)
            m.set_field("point", i, [x + 0.1 * dt, y - 0.2 * dt, z + 0.3 * dt])

    return update


def make_pulse():
    state = {"t": 0.0}

    def update(m, dt):
        s = 1.0 + PULSE_AMPLITUDE * triangle(state["t"] / PULSE_PERIOD)
        for i in range(N_POINTS):
            x, y, z = m.get_field("point", i)
            m.set_field("point", i, [0.0 + (x - 0.0) * s, 0.0 + (y - 0.0) * s, 0.0 + (z - 0.0) * s])
        state["t"] += dt

    return update


def make_ramp():
    state = {"t": 0.0}

    def update(m, dt):
        u = state["t"] / RAMP_PERIOD
        alpha = u - math.floor(u)
        row = [a + (b - a) * alpha for a, b in zip(RAMP_FROM, RAMP_TO)]
        for i in range(N_POINTS):
            m.set_field("rgba", i, row)
        state["t"] += dt

    return update


# ---------------------------------------------------------------- rung 1
# The same arithmetic expressed over the group's writable views.
def make_shift_views():
    velocity = np.array(VELOCITY, dtype=np.float64)

    def update(views, dt):
        delta = velocity * dt
        for view in views:
            points = view["point"]
            points[:] = points.astype(np.float64) + delta

    return update


def make_pulse_views():
    state = {"t": 0.0}

    def update(views, dt):
        s = 1.0 + PULSE_AMPLITUDE * triangle(state["t"] / PULSE_PERIOD)
        for view in views:
            points = view["point"]
            points[:] = 0.0 + (points.astype(np.float64) - 0.0) * s
        state["t"] += dt

    return update


def make_ramp_views():
    state = {"t": 0.0}
    start = np.array(RAMP_FROM, dtype=np.float64)
    stop = np.array(RAMP_TO, dtype=np.float64)

    def update(views, dt):
        u = state["t"] / RAMP_PERIOD
        alpha = u - math.floor(u)
        row = start + (stop - start) * alpha
        for view in views:
            view["rgba"][:] = row
        state["t"] += dt

    return update


def build_rung0():
    scene = Scene()
    shifter, pulser, ramper = fresh_mobject(), fresh_mobject(), fresh_mobject()
    scene.add(shifter, pulser, ramper)
    shifter.add_updater(make_shift(), call=False)
    pulser.add_updater(make_pulse(), call=False)
    ramper.add_updater(make_ramp(), call=False)
    scene._keep = (shifter, pulser, ramper)
    return scene


def build_rung1():
    scene = Scene()
    shifter, pulser, ramper = fresh_mobject(), fresh_mobject(), fresh_mobject()
    scene.add(shifter, pulser, ramper)
    shifter.add_updater(BatchedUpdater([shifter], make_shift_views()), call=False)
    pulser.add_updater(BatchedUpdater([pulser], make_pulse_views()), call=False)
    ramper.add_updater(BatchedUpdater([ramper], make_ramp_views()), call=False)
    scene._keep = (shifter, pulser, ramper)
    return scene


def build_rung2():
    scene = Scene()
    shifter, pulser, ramper = fresh_mobject(), fresh_mobject(), fresh_mobject()
    scene.add(shifter, pulser, ramper)
    shifter.add_updater(
        ArrayUpdater.shift(shifter, "point", list(VELOCITY)), call=False
    )
    pulser.add_updater(
        ArrayUpdater.scale_pulse(
            pulser, "point", list(PULSE_CENTER), PULSE_AMPLITUDE, PULSE_PERIOD
        ),
        call=False,
    )
    ramper.add_updater(
        ArrayUpdater.color_ramp(
            ramper, "rgba", list(RAMP_FROM), list(RAMP_TO), RAMP_PERIOD
        ),
        call=False,
    )
    scene._keep = (shifter, pulser, ramper)
    return scene


def build_rung3():
    scene = Scene()
    shifter, pulser, ramper = fresh_mobject(), fresh_mobject(), fresh_mobject()
    scene.add(shifter, pulser, ramper)
    handles = [
        NativeUpdater.shift(shifter, "point", list(VELOCITY)),
        NativeUpdater.scale_pulse(
            pulser, "point", list(PULSE_CENTER), PULSE_AMPLITUDE, PULSE_PERIOD
        ),
        NativeUpdater.color_ramp(
            ramper, "rgba", list(RAMP_FROM), list(RAMP_TO), RAMP_PERIOD
        ),
    ]
    scene._keep = (shifter, pulser, ramper, handles)
    return scene, handles


rung0 = build_rung0()
rung1 = build_rung1()
rung2 = build_rung2()
rung3, rung3_handles = build_rung3()

# Bit-equality per frame: every rung against rung 0, on the corpus.
for frame in range(FRAMES):
    rung0.update(DT)
    rung1.update(DT)
    rung2.update(DT)
    rung3.update(DT)
    reference = scene_state(rung0)
    assert scene_state(rung1) == reference, f"rung 1 drifted from rung 0 at frame {frame}"
    assert scene_state(rung2) == reference, f"rung 2 drifted from rung 0 at frame {frame}"
    assert scene_state(rung3) == reference, f"rung 3 drifted from rung 0 at frame {frame}"

# The corpus actually moved (bit-equality is not vacuous identity).
final = scene_state(rung0)
initial_probe = build_rung0()
assert scene_state(initial_probe) != final

# Rung 1 under the batched dispatch loop is bit-identical to rung 1 under
# the per-updater loop (the two dispatch paths share declared semantics).
batched = build_rung1()
looped = build_rung1()
for frame in range(FRAMES):
    batched.update_batched(DT)
    looped.update(DT)
    assert scene_state(batched) == scene_state(looped), f"batched dispatch drifted at frame {frame}"

# Rung time bases advance exactly one dt per frame.
probe = build_rung2()
array_updaters = [u for m in probe.mobjects for u in m.updaters]
probe.update(DT)
probe.update(DT)
assert all(u.time() == 2 * DT for u in array_updaters)
assert all(u.time() == 2 * DT for u in array_updaters)
assert sorted(u.class_tag() for u in array_updaters) == ["color-ramp", "scale-pulse", "shift"]
native_probe, native_handles = build_rung3()
native_probe.update(DT)
assert all(h.time() == DT for h in native_handles)
assert sorted(h.class_tag() for h in native_handles) == ["color-ramp", "scale-pulse", "shift"]
assert all(isinstance(h.updater_id(), int) for h in native_handles)

# detach() unregisters; the next frame leaves the buffer untouched.
detached_scene, detached_handles = build_rung3()
before = scene_state(detached_scene)
assert detached_handles[0].detach() is True
assert detached_handles[0].detach() is False
detached_scene.update(DT)
detached_scene.update(DT)
after = scene_state(detached_scene)
assert after[0] == before[0], "a detached native updater still ran"
# Two frames in, the pulse (identity at t = 0 by design) and the ramp have
# both moved; the detached shifter stays frozen.
assert after[1] != before[1] and after[2] != before[2]
assert detached_handles[0].is_attached() is False
assert detached_handles[1].is_attached() is True

# ------------------------------------------------- crossing reduction
# Six mobjects; rung 0 dispatches six Python updaters writing four fields
# each; rung 1 is ONE group callback over six views.
REDUCTION_MOBS = 6


def build_reduction_rung0():
    scene = Scene()
    mobs = [fresh_mobject() for _ in range(REDUCTION_MOBS)]
    scene.add(*mobs)
    for mob in mobs:
        mob.add_updater(make_shift(), call=False)
    scene._keep = mobs
    return scene


def build_reduction_rung1():
    scene = Scene()
    mobs = [fresh_mobject() for _ in range(REDUCTION_MOBS)]
    scene.add(*mobs)
    mobs[0].add_updater(BatchedUpdater(mobs, make_shift_views()), call=False)
    scene._keep = mobs
    return scene


reduction0 = build_reduction_rung0()
manimlib._crossing_stats_reset()
reduction0.update(DT)
rung0_counts = manimlib._crossing_stats_snapshot()

reduction1 = build_reduction_rung1()
manimlib._crossing_stats_reset()
reduction1.update(DT)
rung1_counts = manimlib._crossing_stats_snapshot()

assert rung0_counts["updater_call"] == REDUCTION_MOBS, rung0_counts
assert rung0_counts["field_write"] == REDUCTION_MOBS * N_POINTS, rung0_counts
# One dispatch crossing (the update loop's per-updater record) plus one
# group-callback crossing — versus six dispatches and 24 field writes.
assert rung1_counts["updater_call"] == 2, rung1_counts
assert rung1_counts["field_write"] == 0, rung1_counts
assert rung1_counts["dirty_propagation"] == 1, rung1_counts
assert rung1_counts["total"] < rung0_counts["total"], (rung0_counts, rung1_counts)

# ...and the reduced rung is still bit-identical to rung 0 on this corpus.
for frame in range(FRAMES):
    reduction0.update(DT)
    reduction1.update(DT)
    assert scene_state(reduction1) == scene_state(reduction0), f"reduction drifted at frame {frame}"

# ------------------------------------------------------- explicit opt-in
# Rung objects are distinct, inspectable types sitting in the ordinary
# updater list; nothing rewrites a rung-0 callable.
optin = build_rung2()
kinds = [type(u).__name__ for m in optin.mobjects for u in m.updaters]
assert kinds == ["_ArrayUpdater", "_ArrayUpdater", "_ArrayUpdater"], kinds
grouped = build_reduction_rung1()
assert type(grouped.mobjects[0].updaters[0]).__name__ == "_BatchedUpdater"
assert grouped.mobjects[0].updaters[0].group_size() == REDUCTION_MOBS

# Construction validation is loud.
try:
    BatchedUpdater([], lambda views, dt: None)
except ValueError:
    pass
else:
    raise AssertionError("an empty BatchedUpdater group was accepted")
try:
    BatchedUpdater([object()], lambda views, dt: None)
except TypeError:
    pass
else:
    raise AssertionError("a non-Mobject BatchedUpdater member was accepted")
try:
    BatchedUpdater([fresh_mobject()], 42)
except TypeError:
    pass
else:
    raise AssertionError("a non-callable BatchedUpdater callback was accepted")
try:
    ArrayUpdater.shift(fresh_mobject(), "missing", [1.0, 2.0, 3.0])
except KeyError:
    pass
else:
    raise AssertionError("a declared op on a missing field was accepted")
try:
    ArrayUpdater.shift(fresh_mobject(), "point", [1.0])
except ValueError:
    pass
else:
    raise AssertionError("a declared op with the wrong lane count was accepted")
try:
    ArrayUpdater.shift(fresh_mobject(), "point", [1.0, float("nan"), 3.0])
except ValueError:
    pass
else:
    raise AssertionError("a declared op with a NaN parameter was accepted")
try:
    ArrayUpdater.color_ramp(fresh_mobject(), "rgba", [0.0] * 4, [1.0] * 4, 0.0)
except ValueError:
    pass
else:
    raise AssertionError("a declared op with a zero period was accepted")

# NativeUpdater requires a bound mobject.
detached_target = fresh_mobject()
try:
    NativeUpdater.shift(detached_target, "point", [1.0, 2.0, 3.0])
except bridge_errors.StaleHandleError:
    pass
else:
    raise AssertionError("a native updater was registered on a detached mobject")
