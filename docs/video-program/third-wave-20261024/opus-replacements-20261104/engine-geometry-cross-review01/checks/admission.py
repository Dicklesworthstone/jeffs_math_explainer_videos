"""A second mirror of RenderPlan::sync_with_limits (plan.rs:696-721), written from the source and not from demand.py.

The rule it mirrors. Reachability is measured from the last sync's rows, at the start of the sync.
Over max_unreachable_styles: rebuild first. Otherwise append; if a knot limit trips with something
unreachable: rebuild. With nothing unreachable: refuse. A rebuild keeps only what the stage holds now.

Part 1 repeats Ruby's whole-plate repaint. Part 2 follows gallery.rs 05_9 as the film plays it: panels
enter and leave one at a time, so the six are never repainted together. Part 3 plays the lab's Kind::Fade.

    python3 -I -B admission.py
"""

UNREACHABLE_STYLES = 1 << 12


class Refused(Exception):
    pass


class Plan:
    def __init__(self, limit):
        self.limit = limit
        self.knots = 0
        self.styles = 0
        self.live_styles = 0
        self.compactions = []
        self.peak = 0
        self.frame = 0

    def sync(self, live, repainted):
        """live: {plate: (rows, knots a row)} on screen. repainted: plates whose rows have new paint."""
        unreachable = self.styles - self.live_styles
        if unreachable > UNREACHABLE_STYLES:
            self.rebuild(live, "UnreachableBudget")
        else:
            total = self.knots
            refused_at = None
            for name in repainted:
                rows, row_knots = live[name]
                for _ in range(rows):
                    total += row_knots
                    if total > self.limit:
                        refused_at = total
                        break
                if refused_at:
                    break
            if refused_at:
                if unreachable > 0:
                    self.rebuild(live, "AdmissionBoundary")
                else:
                    raise Refused(f"frame {self.frame}: requested {refused_at}, limit {self.limit}")
            else:
                self.knots = total
                self.styles += sum(live[n][0] for n in repainted)
        self.live_styles = sum(v[0] for v in live.values())
        self.peak = max(self.peak, self.knots)
        self.frame += 1

    def rebuild(self, live, reason):
        total = 0
        for rows, row_knots in live.values():
            for _ in range(rows):
                total += row_knots
                if total > self.limit:
                    raise Refused(f"frame {self.frame}: rebuild requested {total}, limit {self.limit}")
        self.knots = total
        self.styles = sum(v[0] for v in live.values())
        self.compactions.append((self.frame, reason))


LIMITS = [("1<<20", 1 << 20), ("1<<21", 1 << 21), ("1<<22", 1 << 22)]


def report(name, run):
    for label, limit in LIMITS:
        plan = Plan(limit)
        try:
            run(plan)
        except Refused as error:
            print(f"{name:<34}{label:>7}  REFUSED, {error}")
            continue
        gaps = [b[0] - a[0] for a, b in zip(plan.compactions, plan.compactions[1:])]
        cadence = f"every {gaps[0]} frames" if gaps and len(set(gaps)) == 1 else f"gaps {sorted(set(gaps))}" if gaps else "none"
        why = sorted({r for _, r in plan.compactions})
        print(f"{name:<34}{label:>7}  admitted, {plan.frame} frames, peak {plan.peak}, {len(plan.compactions)} compactions, {cadence} {why}")


def whole_plate(rows, row_knots, frames):
    def run(plan):
        live = {"plate": (rows, row_knots)}
        for _ in range(frames):
            plan.sync(live, ["plate"])
    return run


def film_05_9(plan):
    """gallery.rs:854-930 at 60 fps. Painter::show and leave are fades of the whole group."""
    row = 2 * 480 + 9
    zoom = (571, 2 * 571 + 9)
    live = {}
    for k in range(6):
        live[f"p{k}"] = (297, row)
        for _ in range(15):  # painter.show(stage, 0.25)
            plan.sync(live, [f"p{k}"])
    for _ in range(60):  # the six hold, with no new paint
        plan.sync(live, [])
    for k in range(1, 6):
        for _ in range(12):  # p.leave(stage, 0.2)
            plan.sync(live, [f"p{k}"])
        del live[f"p{k}"]
        plan.sync(live, [])
    live["zoom"] = zoom
    for _ in range(48):  # zoom.show(stage, 0.8)
        plan.sync(live, ["zoom"])
    for _ in range(60):
        plan.sync(live, [])
    for _ in range(30):  # painters[0].leave(stage, 0.5)
        plan.sync(live, ["p0"])
    del live["p0"]
    for _ in range(30):  # zoom.leave(stage, 0.5)
        plan.sync(live, ["zoom"])


def lab_fade(rows, row_knots):
    """plate_lab.rs fade() at FADE_FPS 3: show(1.0), wait 1 frame, leave(1.0) is seven frames."""
    def run(plan):
        live = {"plate": (rows, row_knots)}
        for _ in range(3):
            plan.sync(live, ["plate"])
        plan.sync(live, [])
        for _ in range(3):
            plan.sync(live, ["plate"])
    return run


def main():
    z2 = (731, 1471)
    sin = (640, 1289)
    print("Part 1. Every row repainted every frame, 24 frames")
    report("z2 cells4", whole_plate(*z2, 24))
    report("sin(1/z) cells4", whole_plate(*sin, 24))
    report("six panels, all together (bound)", whole_plate(1782, 969, 24))
    report("z2 cells2", whole_plate(366, 741, 24))
    print()
    print("Part 2. 05_9 as the film plays it, cells4, sequential fades")
    report("05_9 six, then cubic + zoom", film_05_9)
    seven = 7 * 297 * 969
    print(f"live six plus the one fading: 7 x 297 x 969 = {seven}; 1<<22 is {(1 << 22) / seven:.2f} times that")
    print()
    print("Part 3. The lab's seven-frame fade at z2 cells4")
    report("z2 cells4, Kind::Fade", lab_fade(*z2))
    print()
    print("Plate lab frame total if a z2 cells4 fade is added: 67 + (77 + 7) + 9 =", 67 + 77 + 7 + 9, "against FRAME_LIMIT 160")
    print("Plate lab frame total if a fade and a repaint are both added: 67 + 84 + 12 =", 67 + 84 + 12, "(over)")


if __name__ == "__main__":
    main()
