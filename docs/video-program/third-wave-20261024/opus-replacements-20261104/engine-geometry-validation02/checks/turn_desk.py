#!/usr/bin/env python3
"""What the lab's 11_7 turn should ask of the budget, from reading the pinned engine. Not a measurement.

The plate is the film's: counting.rs's input plane at cells4, 640 rows of 640 cells, each row a
strip of 2 * 640 + 9 knots. Every frame that writes a colour lane brings a whole plate of new knots.
A plan keeps what it has held until the next would pass the limit, then keeps only what the last
capture still showed, and refuses if that plus the new plate still does not fit.

The model charges the painter's last fade frame in full. If a wholly transparent row costs less,
the real numbers are lower, so this is an upper bound on the peak and on where a refusal comes.

    python3 -I -B turn_desk.py
"""
ROWS, COLS = 640, 640
LIVE = ROWS * (2 * COLS + 9)
DEFAULT, CONTROL, BUDGET = 1 << 20, 1 << 21, 1 << 22
HOLD, LEAVE, REPAINTS = 1, 3, 9


def walk(limit, repaints=REPAINTS):
    """Plates kept, per capture, or the capture that was refused."""
    kept = 0
    shown_last = 0
    rows = []
    compactions = []
    for capture in range(HOLD + LEAVE + repaints):
        if capture == 0:
            new, shown = 1, 1  # the painter alone
        elif capture == 1:
            new, shown = 2, 2  # the field appears; the painter takes its first fade frame
        elif capture < HOLD + LEAVE:
            new, shown = 1, 2  # the painter's fade
        else:
            new, shown = 1, 1  # a repaint of the field, the painter gone
        if (kept + new) * LIVE > limit:
            compactions.append(capture)
            kept = shown_last
            if (kept + new) * LIVE > limit:
                return {"refused": capture, "asks": (kept + new) * LIVE, "plates": rows, "compactions": compactions}
        kept += new
        shown_last = shown
        rows.append(kept)
    return {"refused": None, "plates": rows, "compactions": compactions}


assert LIVE == 824_960
print(f"quintic cells4: {ROWS} rows of {2 * COLS + 9} knots, {LIVE} knots a plate, {ROWS * COLS} cells")
print(f"default {DEFAULT}: holds {DEFAULT // LIVE} plate; 1<<21 holds {CONTROL // LIVE}; 1<<22 holds {BUDGET // LIVE}")
print()

for name, limit in (("the default", DEFAULT), ("1<<21", CONTROL)):
    run = walk(limit)
    assert run["refused"] == 1, run
    print(f"Under {name}: refused at capture 1, the takeover, before any repaint; asks at most {run['asks']}.")
print()

run = walk(BUDGET)
assert run["refused"] is None and max(run["plates"]) == 5, run
assert run["compactions"] == [4, 7, 11], run
print(f"Lab turn under 1<<22, {HOLD} hold + {LEAVE} fade + {REPAINTS} repaints = {HOLD + LEAVE + REPAINTS} frames:")
print("  plates kept by capture:", run["plates"])
print(f"  compactions at captures {run['compactions']}")
print(f"  peak {max(run['plates']) * LIVE} knots, {BUDGET - max(run['plates']) * LIVE} under the budget ({1 - max(run['plates']) * LIVE / BUDGET:.1%})")
print()

whole = walk(BUDGET, repaints=96)
tail = whole["plates"][4:]
assert whole["refused"] is None and max(whole["plates"]) == 5
assert tail[3:] == ([2, 3, 4, 5] * 24)[: len(tail) - 3], tail
print("The same rule run through 96 repaints never passes 5 plates: after the takeover it repeats")
print("2, 3, 4, 5 plates. That is this model's reasoning, not evidence about the film. Time and memory")
print("are not modelled. The real 11_7 render has to confirm it, once one tree holds the plate core,")
print("the teaching repair and picture_cells(&inp, &super::draw::PLATE).")
