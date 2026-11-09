"""G0-5 crossing-cost measurement (fm-87q).

Medians over repeated batches, perf_counter_ns, no warmup tricks beyond a
throwaway first batch. These numbers seed PG-8's class-tiered binding-tax
budgets (§15.2: cached method resolution, batched crossings, GIL windows
are designed against them); they are recorded in the ratification note.

Measured crossings:
  py->rust noop     the floor: one bound-method call into the bridge
  py->rust data     get_field/set_field (engine borrow + copy in/out)
  view write        PyRecordView.write (no entry resolution, direct alias)
  rust->py updater  one stage.update(dt) driving N Python callbacks
  rust->py interp   one run_transform step dispatching a Python override
"""

from __future__ import annotations

import statistics
import time

import fmn_spike_bridge as bridge


class Mobject(bridge.BridgeMobject):
    def __init__(self, stage):
        self._engine_init(stage)


class NoopInterpolator(Mobject):
    def init_points(self):
        self.resize(1)

    def interpolate(self, start, target, alpha):
        pass


def bench(label, fn, *, batch, reps=7):
    fn()  # throwaway batch: import/JIT/alloc noise
    samples = []
    for _ in range(reps):
        t0 = time.perf_counter_ns()
        fn()
        samples.append((time.perf_counter_ns() - t0) / batch)
    med = statistics.median(samples)
    spread = max(samples) - min(samples)
    print(f"{label:<34} {med:>10.0f} ns/crossing   (±{spread / 2:.0f} over {reps} reps)")
    return med


def main():
    stage = bridge.Stage()
    mob = Mobject(stage)
    mob.resize(8)

    n = 10_000
    bench("py->rust noop method", lambda: [mob.noop() for _ in range(n)], batch=n)
    bench(
        "py->rust get_field (3 lanes)",
        lambda: [mob.get_field("point", 0) for _ in range(n)],
        batch=n,
    )
    payload = [1.0, 2.0, 3.0]
    bench(
        "py->rust set_field (3 lanes)",
        lambda: [mob.set_field("point", 0, payload) for _ in range(n)],
        batch=n,
    )

    view = mob.data_view()
    bench(
        "view write (3 lanes)",
        lambda: [view.write(0, "point", payload) for _ in range(n)],
        batch=n,
    )
    bench(
        "view read (3 lanes)",
        lambda: [view.read(0, "point") for _ in range(n)],
        batch=n,
    )

    k = 100
    ticks = []
    updated = Mobject(stage)
    updated.add_updater(lambda m, dt: ticks.append(dt))
    stage.add_to_scene(updated)
    bench(
        "rust->py updater dispatch",
        lambda: [stage.update(0.001) for _ in range(k)],
        batch=k,
    )

    a = NoopInterpolator(stage)
    b = NoopInterpolator(stage)
    steps = 200
    bench(
        "rust->py interpolate dispatch",
        lambda: stage.run_transform(a, b, steps),
        batch=steps + 1,
    )


if __name__ == "__main__":
    main()
