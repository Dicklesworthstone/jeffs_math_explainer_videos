"""fm-zoi detection-report acceptance (§15.2 Rev 4, §17.1).

Exercises `manimlib._crossing_report()` end to end: the schema line, the
per-class counters, the phase breakdown, and the callback-heavy verdict
on a scene whose Python callbacks dominate the frame by construction
(busy-spinning updater bodies against a trivial native advance). The
golden byte-locked format lives in report.rs's Rust tests; this suite
pins the live getter's behavior. No wall-clock values are asserted — only
counts, format, and a verdict the workload decides by orders of magnitude.
"""

import manimlib
from manimlib import Mobject, Scene

MOBS = 6
FRAMES = 3


def parse(report):
    rows = {}
    lines = report.splitlines()
    assert lines[0] == "schema\tfmn-crossing-report/1", lines[0]
    for line in lines[1:]:
        parts = line.split("\t")
        assert len(parts) == 3, line
        key = (parts[0], parts[1])
        assert key not in rows, f"duplicate row {key}"
        rows[key] = parts[2]
    expected = {
        ("crossing", name)
        for name in (
            "updater_call",
            "method_dispatch",
            "field_write",
            "dirty_propagation",
            "other",
            "total",
        )
    } | {
        ("phase", "python_callback_ns"),
        ("phase", "native_ns"),
        ("phase", "total_ns"),
        ("share", "python_callback_ppm"),
        ("share", "updater_call_ppm"),
        ("detection", "callback_heavy"),
        ("detection", "rule"),
    }
    assert set(rows) == expected, set(rows) ^ expected
    return rows


# Empty counters: zero totals, "-" shares, not callback-heavy.
manimlib._crossing_stats_reset()
rows = parse(manimlib._crossing_report())
assert rows[("crossing", "total")] == "0"
assert rows[("phase", "total_ns")] == "0"
assert rows[("share", "python_callback_ppm")] == "-"
assert rows[("share", "updater_call_ppm")] == "-"
assert rows[("detection", "callback_heavy")] == "false"
assert rows[("detection", "rule")] == "updater_call>0&&python_callback_ppm>=500000"


# Callback-heavy scene: spinning Python updaters, trivial native advance.
def make_spinner():
    def update(mob, dt):
        acc = 0.0
        for k in range(2000):
            acc += k * dt
        mob.set_field("point", 0, [acc, 0.0, 0.0])

    return update


scene = Scene()
mobs = []
for _ in range(MOBS):
    mob = Mobject()
    mob.resize(2)
    mob.add_updater(make_spinner(), call=False)
    mobs.append(mob)
scene.add(*mobs)
scene._keep = mobs

manimlib._crossing_stats_reset()
for _ in range(FRAMES):
    scene.update(0.01)
rows = parse(manimlib._crossing_report())

assert int(rows[("crossing", "updater_call")]) == MOBS * FRAMES, rows
assert int(rows[("crossing", "field_write")]) == MOBS * FRAMES, rows
assert int(rows[("crossing", "total")]) == sum(
    int(rows[("crossing", name)])
    for name in ("updater_call", "method_dispatch", "field_write", "dirty_propagation", "other")
)
python_ns = int(rows[("phase", "python_callback_ns")])
native_ns = int(rows[("phase", "native_ns")])
assert python_ns + native_ns == int(rows[("phase", "total_ns")])
share = rows[("share", "python_callback_ppm")]
expected_share = str(python_ns * 1_000_000 // (python_ns + native_ns)) if python_ns + native_ns else "-"
assert share == expected_share, (share, expected_share)
# The workload decides the verdict by orders of magnitude, not by a margin:
# thousands of Python operations per frame against an empty native advance.
assert python_ns > native_ns * 100, (python_ns, native_ns)
assert rows[("detection", "callback_heavy")] == "true", rows

# Reset restores the empty report.
manimlib._crossing_stats_reset()
rows = parse(manimlib._crossing_report())
assert rows[("crossing", "total")] == "0"
assert rows[("detection", "callback_heavy")] == "false"
