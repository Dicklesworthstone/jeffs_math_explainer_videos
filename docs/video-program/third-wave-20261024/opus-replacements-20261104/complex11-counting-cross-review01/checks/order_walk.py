"""A walk of 11_7's draw order from the source text, with the engine's rule, and planted faults.

The rule, read at pin a399d626: adding a mob removes its family and re-appends it, so a re-add
promotes and takes no film time (fmn-mobject/src/stage.rs:811); an animation roots a mob only if it
is not in the scene already (fmn-anim/src/frame.rs:195); a fade-out removes it at its end. Nothing
in this crate sets a z-index.

This walks the statements in order and models a list. It sees no pixels and no types. It stops at
DogWalk::build, since root's code runs unchanged from there.

Usage: python3 -I -B order_walk.py COUNTING_RS
Exit 0 when the staged order is clean and every planted fault is caught.
"""

import re
import sys

SOURCE = open(sys.argv[1], encoding="utf8").read()
START, STOP = 'let line = say(&kit, stage, "11_7")?;', "let dog = DogWalk::build("
ROOTS = [f"root{k}" for k in range(5)]
INSIDE = ["inside_back", "inside_text"]
ONE = ["ring", "shown", "trail", "arrow", "in_dot", "out_dot"]
# what 11_1 to 11_6 leave on screen as 11_7 starts, bottom first, by the source's order
OPENING = ["painter", "out_axes", "out_border", "zero_label", "walker_anchor", "unit", "reference", *ONE, "leash", "follows", *ROOTS, *INSIDE, "squashed"]
# on the input plane: under the field they would vanish
MUST_STAY_OVER_FIELD = ["ring", *ROOTS]
TEXT_ROWS = [("before", "after"), ("every_output", "zero_kept"), ("after", "degree_n")]
LEFT_FOR_THE_WIDE_PLANE = {"out_axes", "out_border", "zero_label", "walker_anchor"}


def statements(cue):
    cue = "\n".join(re.sub(r"//.*$", "", row) for row in cue.splitlines())
    cue = re.sub(r"for &mark in &roots \{\s*stage\.add_to_scene\(mark\)\?;\s*\}", "ADD_MARKS;", cue)
    return [" ".join(s.split()) for s in cue.split(";") if s.strip()]


def names(arg, lists):
    arg = arg.strip()
    if arg in lists:
        return list(lists[arg])
    inner = re.fullmatch(r"\[(.*)\]", arg)
    if inner:
        out = []
        for name in inner.group(1).split(","):
            name = name.strip()
            if name:
                out.extend(lists.get(name, [name]))
        return out
    return [arg]


def walk(source):
    begin = source.index(START) + len(START)
    cue = source[begin : source.index(STOP, begin)]
    scene = list(OPENING)
    faults = []
    lists = {"inside": INSIDE, "roots": ROOTS, "same": ["same_back", "same_text"]}
    alias = {"walked_loop": "ring", "walked_curve": "trail", "turning.field": "field", "turning.anchor": "turn_anchor"}
    built = stopped = False
    faded = set()
    swaps = []
    turns = 0

    def real(name):
        return alias.get(name, name)

    def at_film_time(what):
        """Something plays: the state of the list now is what is on screen."""
        if built and not stopped and "field" in scene:
            bottom = scene.index("field")
            for mob in MUST_STAY_OVER_FIELD:
                if mob not in scene:
                    faults.append(f"MISSING {mob} while the field is up, at {what}")
                elif scene.index(mob) < bottom:
                    faults.append(f"UNDER {mob} is under the field at {what}")
            if "painter" in scene and scene.index("painter") > bottom:
                faults.append(f"OVER the painter is over the field at {what}")
            for mob in scene:
                if mob.startswith("same") and scene.index(mob) < bottom:
                    faults.append(f"UNDER {mob} is under the field at {what}")

    def fade_in(mobs, what):
        at_film_time(what)
        for mob in mobs:
            if mob not in scene:
                scene.append(mob)
        at_film_time(what + " (rooted)")

    def fade_out(mobs, what):
        for mob in mobs:
            if mob not in scene:
                faults.append(f"GHOST {mob} fades out but is not on screen, at {what}")
        at_film_time(what)
        for mob in mobs:
            if mob in scene:
                scene.remove(mob)
                faded.add(mob)

    def promote(mob):
        if mob in faded:
            faults.append(f"POP {mob} has faded out and comes straight back")
        if mob in scene:
            scene.remove(mob)
        scene.append(mob)

    for s in statements(cue):
        m = re.fullmatch(r"let \(walked_loop, walked_curve\) = \(one\[0\], one\[2\]\)", s)
        if m:
            continue
        m = re.fullmatch(r"let mut (\w+) = vec!\[(.*)\]", s)
        if m:
            parts = []
            for item in m.group(2).split(","):
                item = item.strip()
                k = re.fullmatch(r"one\[(\d)\]", item)
                parts.append(ONE[int(k.group(1))] if k else real(item))
            lists[m.group(1)] = parts
            continue
        m = re.fullmatch(r"gone\.extend\(one\[3\.\.\]\.iter\(\)\.copied\(\)\)", s)
        if m:
            lists["gone"].extend(ONE[3:])
            continue
        m = re.fullmatch(r"(\w+)\.extend\((.*)\)", s)
        if m:
            lists.setdefault(m.group(1), []).extend(real(n) for n in names(m.group(2), lists))
            continue
        m = re.fullmatch(r"fade_out_all\(stage, &(.*), [\d.]+\)\?", s)
        if m:
            fade_out([real(n) for n in names(m.group(1), lists)], s)
            continue
        m = re.fullmatch(r"fade_in_all\(stage, &(.*), [\d.]+\)\?", s)
        if m:
            fade_in([real(n) for n in names(m.group(1), lists)], s)
            continue
        if s.startswith("let turning = Turning::build("):
            scene += ["field", "turn_anchor"]
            built = True
            continue
        m = re.fullmatch(r"stage\.add_to_scene\((\w+)\)\?", s)
        if m:
            promote(real(m.group(1)))
            continue
        if s == "ADD_MARKS":
            for mob in ROOTS:
                promote(mob)
            continue
        if s.startswith("painter.leave("):
            if not built:
                faults.append("BARE the painter leaves before the field is on screen")
            at_film_time(s)
            if "painter" in scene:
                scene.remove("painter")
            else:
                faults.append("GHOST the painter leaves twice")
            continue
        if s.startswith("stage.play(tween(turning.turned"):
            turns += 1
            if not built or stopped:
                faults.append("DEAD the tracker moves with no updater")
            at_film_time(s)
            swaps.append(("turn", turns))
            continue
        if s == "turning.stop(stage)":
            stopped = True
            if "turn_anchor" in scene:
                scene.remove("turn_anchor")
            continue
        if s.startswith(("line.wait_", "wait_until(")):
            at_film_time(s)
            continue
        # a build or a binding: takes no film time and roots nothing
    for first, second in TEXT_ROWS:
        if first in scene and second in scene:
            faults.append(f"STACKED {first} and {second} share a place at the cue's end")
    if not built:
        faults.append("NEVER the field was never built")
    if built and not stopped:
        faults.append("LEAK the updater is still running when the wide plane comes")
    if turns != 2:
        faults.append(f"TURNS {turns} tweens of the tracker, not 2")
    left = set(scene) - LEFT_FOR_THE_WIDE_PLANE - {"wide_axes", "big_loop", "wide_all"}
    left = {mob for mob in left if not mob.startswith(("wide", "big"))}
    if left:
        faults.append(f"LEFT on screen for the wide plane: {sorted(left)}")
    return faults, scene


def stacked_rows(source):
    """Two rows that share a place must not both be fully in: the older fades before the newer."""
    faults = []
    begin = source.index(START)
    cue = source[begin : source.index(STOP, begin)]
    order = [
        ("fade_out_all(stage, &[before, every_output], 0.2)?;", "fade_in_all(stage, &result, 0.3)?;", "before + every_output, then after + zero_kept"),
        ("fade_out_all(stage, &[after], 0.3)?;", "fade_in_all(stage, &[degree_n], 0.4)?;", "after, then degree_n"),
    ]
    for gone, comes, what in order:
        if gone not in cue or comes not in cue or cue.index(gone) > cue.index(comes):
            faults.append(f"ROWS {what}: the old row is not gone first")
    return faults


def turn_order(source):
    """The result rows wait for the turn back to land; the turn back waits for 'Divide'."""
    faults = []
    begin = source.index(START)
    cue = source[begin : source.index(STOP, begin)]
    on_words = 'line.wait_for(stage, "with z")?;'
    swap_key = on_words if on_words in cue else "line.wait_sentence(stage, 2)?;"
    sequence = [
        "stage.play(tween(turning.turned, 1.0, rt)?)?;",
        "line.wait_sentence(stage, 1)?;",
        "stage.play(tween(turning.turned, 0.0, rt)?)?;",
        "fade_in_all(stage, &result, 0.3)?;",
        swap_key,
        "line.wait_sentence(stage, 3)?;",
        "turning.stop(stage);",
    ]
    at = 0
    for step in sequence:
        found = cue.find(step, at)
        if found < 0:
            faults.append(f"SEQUENCE out of order at {step}")
        else:
            at = found
    return faults


def plate_order(source):
    """11_8: the plate and name go over the line and the disk before the sweep, name above plate."""
    faults = []
    body = source[source.index("fn coarse_loop(") :]
    body = body[: body.index("\n}\n")]
    find = body.find
    sweep = find("stage.play(tween_linear(carried, 1.0, rt)?)?;")
    plate, name = find("stage.add_to_scene(zero_plate)?;"), find("stage.add_to_scene(zero_name)?;")
    for label, built_at in (("gap", find("let gap =")), ("disk", find("let disk =")), ("a_name", find("let a_name ="))):
        if not 0 <= built_at < plate:
            faults.append(f"PLATE {label} is rooted after the plate's promotion")
    if not 0 <= plate < name < sweep:
        faults.append("PLATE the promotion is not plate, name, sweep")
    # On zero the line's end meets the plate at every bearing, its first frame included. Off zero, as
    # staged, the line starts on the positive axis, clear of the plate: teaching02's test proves that.
    line_rooted, line_fade = find("stage.add_to_scene(gap)?;"), "fade_in_all(stage, &[gap], 0.2)?;"
    on_zero = "const ZERO_NAME_AT: (f64, f64) = (0.0, 0.0);" in source
    if on_zero and not 0 <= line_rooted < plate < name < find(line_fade) < sweep:
        faults.append("PLATE the line's end shows over the digit on zero, then the plate cuts it")
    after = body[plate:sweep]
    if 0 <= line_rooted < plate:
        after = after.replace(line_fade, "")
    if "fade_in_all" in after:
        faults.append("PLATE something roots between the promotion and the sweep")
    if "frame.extend([zero_plate, zero_name]);" not in body or "let mut left = frame;" not in body:
        faults.append("PLATE the plate and name do not leave with the frame")
    if not 0 <= find("frame.extend(out.axes(stage)?);") < find("frame.extend([zero_plate, zero_name]);"):
        faults.append("PLATE the plate is not over the axes")
    return faults


def everything(source):
    faults, scene = walk(source)
    return faults + stacked_rows(source) + turn_order(source) + plate_order(source), scene


faults, scene = everything(SOURCE)
print("on screen, bottom first, as the wide plane comes in:", scene)
print("faults on the staged file:", len(faults))
for fault in faults:
    print("  FAULT", fault)

PLANTS = [
    ("the loop is not put back over the field", "stage.add_to_scene(walked_loop)?;\n", "", "UNDER ring"),
    ("the marks are not put back over the field", "for &mark in &roots {\n            stage.add_to_scene(mark)?;\n        }\n", "", "UNDER root0"),
    ("the painter leaves before the field is up", "let turning = Turning::build(stage, inp, out, walked_curve)?;\n", "painter.leave(stage, 0.4)?;\n        let turning = Turning::build(stage, inp, out, walked_curve)?;\n", "BARE"),
    ("the marks fade with 11_6", "gone.extend(inside);\n", "gone.extend(inside);\n        gone.extend(roots);\n", "POP root0"),
    ("the loop fades with 11_6", "let mut gone = vec![one[1]];", "let mut gone = vec![one[0], one[1]];", "POP ring"),
    ("the updater is never stopped", "turning.stop(stage);\n", "", "LEAK"),
    ("the card stays for the wide plane", "acted.extend(same);\n", "", "LEFT"),
    ("the field stays for the wide plane", "walked_curve, turning.field];", "walked_curve];", "LEFT"),
    ("the curve stays for the wide plane", "walked_loop, walked_curve, turning.field", "walked_loop, turning.field", "LEFT"),
    ("the old row is left under the new one", "fade_out_all(stage, &[after], 0.3)?;\n", "", "LEFT"),
    ("the card goes on before the field", "let turning = Turning::build(stage, inp, out, walked_curve)?;\n", "let same_early = 0;\n        fade_in_all(stage, &[same_back], 0.1)?;\n        let turning = Turning::build(stage, inp, out, walked_curve)?;\n", "UNDER same_back"),
    ("the back turn is dropped", "stage.play(tween(turning.turned, 0.0, rt)?)?;\n", "", "TURNS"),
    ("the results land before the turn back", "stage.play(tween(turning.turned, 0.0, rt)?)?;\n        // \"which leaves its roots unchanged\": what the turn left alone, and why\n", "// moved\n", "SEQUENCE"),
    ("11_8's plate is not promoted", "stage.add_to_scene(zero_plate)?;\n", "", "PLATE"),
    ("11_8's name goes under its plate", "stage.add_to_scene(zero_plate)?;\n    stage.add_to_scene(zero_name)?;\n", "stage.add_to_scene(zero_name)?;\n    stage.add_to_scene(zero_plate)?;\n", "PLATE"),
]
if "stage.add_to_scene(gap)?;\n" in SOURCE:
    PLANTS.append(("the line is not put under the plate on zero", "stage.add_to_scene(gap)?;\n", "", "PLATE"))
else:
    PLANTS.append(("the digit goes on zero and the line fades in over it", "const ZERO_NAME_AT: (f64, f64) = (-0.25, -0.3);", "const ZERO_NAME_AT: (f64, f64) = (0.0, 0.0);", "PLATE"))
missed = 0
for name, old, new, code in PLANTS:
    if SOURCE.count(old) != 1:
        print(f"  PLANT NOT APPLICABLE {name}: found {SOURCE.count(old)} times")
        missed += 1
        continue
    try:
        found, _ = everything(SOURCE.replace(old, new))
    except ValueError as error:
        found = [f"{code} walk stopped: {error}"]
    caught = any(code in f for f in found)
    print(f"  plant {'caught' if caught else 'MISSED'}: {name} -> {[f for f in found if code in f][:1]}")
    missed += 0 if caught else 1
print(f"planted: {len(PLANTS)}; missed: {missed}")
sys.exit(1 if faults or missed else 0)
