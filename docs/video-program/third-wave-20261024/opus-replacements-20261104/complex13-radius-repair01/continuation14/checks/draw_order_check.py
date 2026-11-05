"""Walk 14_3's draw order from source text, on canonical and on the repair, and plant faults in the repair.

It reads the scene's statements in order and applies the engine's rule, as read at the pin: a fade roots
a mob only if it is not on stage yet, and on top (fmn-anim begin_animations), while add_to_scene removes
and re-appends (fmn-mobject add_many_to_scene). After every fade it asks whether anything that reaches
the pole lies above the mark. It also mirrors the new Rust test's text checks, which are unrun.

It models z-order from source text. It does not draw, and a real fade could still surprise it.

Usage: python3 -I -B draw_order_check.py CANONICAL_RS STAGED_RS
"""

import re
import sys
from pathlib import Path

OLD, NEW = (Path(a).read_text(encoding="utf8") for a in sys.argv[1:3])
START, STOP = 'let line = say(&kit, stage, "14_3")?;', "let doubling = kit.tex("
STEP_OPEN = "for (k, &c) in centres.iter().enumerate().skip(1) {"
STEPS = 8
STATEMENT = re.compile(
    r"let (?P<mark>\w+) = pole_mark\("
    r"|let (?P<disk>\w+) = disk\("
    r"|let (?P<poly>region) = "
    r"|root_under\(stage, &\[(?P<under>[^\]]*)\], &(?P<under_mark>\w+)\)\?;"
    r"|fade_in_all\(stage, &(?P<fade_list>\[[^\]]*\]|\w+), (?P<fade_t>[\d.]+)\)\?;"
    r"|fade_out_all\(stage, &(?P<out_list>\[[^\]]*\]|\w+), (?P<out_t>[\d.]+)\)\?;"
    r"|stage\.add_to_scene\((?P<add>\w+)\)\?;"
    r"|let mut gone = vec!\[(?P<gone>[^\]]*)\];"
    r"|gone\.extend\((?P<extend>\w+)\);"
    r"|chain\.push\((?P<push>\w+)\);"
)


def names(text):
    return [n.strip() for n in text.strip("[]").split(",") if n.strip()]


def expand(block):
    """Unroll the chain's loop into its eight steps, with each step's mobs named apart."""
    at = block.index(STEP_OPEN)
    body_from = at + len(STEP_OPEN)
    close = block.index("\n        }\n", body_from)
    body = block[body_from:close]
    out = []
    for k in range(1, STEPS + 1):
        text = re.sub(r"if k == 4 \{.*?\n            \}", lambda m: m.group(0) if k == 4 else "", body, flags=re.S)
        text = re.sub(r"\bd\b", f"d{k}", text)
        text = re.sub(r"\bpiece\b", f"piece{k}", text)
        out.append(text)
    return block[:at] + "\n".join(out) + block[close:]


def walk(source):
    block = source[source.index(START) : source.index(STOP)]
    block = expand(block)
    helper = re.search(r"fn root_under.*?\n\}\n", source, flags=re.S)
    mark_first = bool(helper and "mark.iter().chain(mobs)" in helper.group(0))
    scene, marks, reach, faults, states, chain, gone = [], {}, set(), [], [], [], []

    def members(n):
        return marks.get(n, [n])

    def root(mob):
        if mob in scene:
            scene.remove(mob)
        scene.append(mob)

    def check(label):
        rings = [i for i, m in enumerate(scene) if m.endswith(".ring")]
        if not rings:
            return
        above = [m for m in scene[max(rings) + 1 :] if m in reach]
        states.append((label, list(above)))
        if above:
            faults.append(f"{label}: above the mark: {', '.join(above)}")

    for s in STATEMENT.finditer(block):
        g = s.groupdict()
        if g["mark"]:
            marks[g["mark"]] = [f"{g['mark']}.back", f"{g['mark']}.ring"]
        elif g["disk"]:
            reach.add(g["disk"])
        elif g["poly"]:
            reach.add(g["poly"])
        elif g["under"] is not None:
            mobs, mark = names(g["under"]), marks[g["under_mark"]]
            for m in mark + mobs if mark_first else mobs + mark:
                root(m)
        elif g["fade_list"]:
            listed = names(g["fade_list"]) if g["fade_list"].startswith("[") else [g["fade_list"]]
            for n in listed:
                for m in members(n):
                    if m not in scene:
                        scene.append(m)
            check(f"fade in {g['fade_list']}")
        elif g["out_list"]:
            listed = gone if g["out_list"] == "gone" else names(g["out_list"])
            for n in listed:
                for m in members(n):
                    if m in scene:
                        scene.remove(m)
        elif g["add"]:
            for m in members(g["add"]):
                root(m)
        elif g["gone"] is not None:
            gone = names(g["gone"])
        elif g["extend"]:
            gone += chain if g["extend"] == "chain" else [g["extend"]]
        elif g["push"]:
            chain.append(g["push"])

    if len(marks) != 1:
        faults.append(f"{len(marks)} marks in the scene: a second one doubles the ring and pops in")
    left = [m for m in scene if m.endswith((".back", ".ring")) or m in reach]
    if left:
        faults.append(f"left on stage after the chapter's fade out: {', '.join(left)}")
    if len(reach) != 3 + STEPS:
        faults.append(f"expected the unit disk, the region, the first disk and {STEPS} chain disks; found {sorted(reach)}")
    return faults, states, reach


def mirror_of_rust_test(source):
    """The new Rust test's text checks, in Python, so that its strings are at least known to match."""
    problems = []
    start = source.find('say(&kit, stage, "14_3")')
    scene = source[start : source.find("let doubling = kit.tex(")]
    for mobs, fade in (
        ("&[region]", "fade_in_all(stage, &[region, region_says], 0.5)?;"),
        ("&[first]", "fade_in_all(stage, &[first], 0.6)?;"),
        ("&[d, piece]", "fade_in_all(stage, &[d, piece], 0.3)?;"),
    ):
        at, then = scene.find(f"root_under(stage, {mobs}, &pole1)?;"), scene.find(fade)
        if at < 0 or then < 0 or at > then:
            problems.append(f"root_under {mobs} before its fade")
    if scene.count(" = disk(") != 3:
        problems.append("three disks in the scene")
    if not 0 <= scene.find("let unit = disk(") < scene.find("fade_in_all(stage, &pole1, 0.3)?;"):
        problems.append("unit disk before the mark")
    helper = source.find("fn root_under")
    if helper < 0 or "for &m in mobs.iter().chain(mark) {" not in source[helper:start]:
        problems.append("helper puts the mark last")
    if scene.count("pole_mark(") != 1:
        problems.append("one mark")
    if "gone.extend(pole1);" not in scene:
        problems.append("the mark fades out with the rest")
    return problems


def replace_once(text, a, b):
    if text.count(a) != 1:
        raise SystemExit(f"plant does not apply, {text.count(a)} matches: {a!r}")
    return text.replace(a, b)


def reorder(text, first, second):
    return replace_once(text, f"{first}\n{second}", f"{second}\n{first}")


old_faults, old_states, _ = walk(OLD)
new_faults, new_states, reach = walk(NEW)
print(f"canonical: {len(old_faults)} faults")
for f in old_faults:
    print("  ", f)
print(f"repair: {len(new_faults)} faults; {len(new_states)} fades checked with the mark on stage; {len(reach)} things reach the pole")
for f in new_faults:
    print("  ", f)

R_REGION = "        root_under(stage, &[region], &pole1)?;\n"
R_FIRST = "        root_under(stage, &[first], &pole1)?;\n"
R_CHAIN = "            root_under(stage, &[d, piece], &pole1)?;\n"
CHAIN_END = "        line.wait_sentence(stage, 6)?;\n        let same = "
PLANTS = {
    "no re-add at the region": lambda t: replace_once(t, R_REGION, ""),
    "no re-add at the first disk": lambda t: replace_once(t, R_FIRST, ""),
    "no re-add in the chain": lambda t: replace_once(t, R_CHAIN, ""),
    "chain's re-add after its fade": lambda t: reorder(t, R_CHAIN.rstrip("\n"), "            fade_in_all(stage, &[d, piece], 0.3)?;"),
    "first disk's re-add after its fade": lambda t: reorder(t, R_FIRST.rstrip("\n"), "        fade_in_all(stage, &[first], 0.6)?;"),
    "helper puts the mark first": lambda t: replace_once(t, "for &m in mobs.iter().chain(mark) {\n        stage", "for &m in mark.iter().chain(mobs) {\n        stage"),
    "a second mark back at the chain's end": lambda t: replace_once(
        t,
        CHAIN_END,
        "        let pole_top = pole_mark(stage, cp.p(ONE));\n        fade_in_all(stage, &pole_top, 0.2)?;\n" + CHAIN_END,
    ),
    "the mark never fades out": lambda t: replace_once(t, "        gone.extend(pole1);\n", ""),
    "a new disk over the mark": lambda t: replace_once(
        t,
        CHAIN_END,
        "        let extra = disk(stage, &cp, ZERO, 2.0, output(), 0.10)?;\n        fade_in_all(stage, &[extra], 0.3)?;\n        chain.push(extra);\n" + CHAIN_END,
    ),
}
caught = 0
for name, plant in PLANTS.items():
    faults, _states, _reach = walk(plant(NEW))
    mirror = mirror_of_rust_test(plant(NEW))
    verdict = "CAUGHT" if faults else "MISSED"
    caught += bool(faults)
    print(f"plant: {name}: {verdict} by the walk ({len(faults)}); Rust-test mirror {'fails' if mirror else 'passes'}")
print(f"plants caught by the walk: {caught}/{len(PLANTS)}")

mirror_new, mirror_old = mirror_of_rust_test(NEW), mirror_of_rust_test(OLD)
print(f"Rust-test mirror on the repair: {mirror_new or 'all hold'}")
print(f"Rust-test mirror on canonical (must fail): {mirror_old}")

# The anchor formula in native-review-anchors.tsv rests on these fades and moves being what they are.
FORMULA = [
    "fade_out_all(stage, &[first, first_c, region, small_name, pole_name], 0.3)?;",
    "fade_in_all(stage, &[disk_stops], 0.3)?;",
    "fade_in_all(stage, &[walker], 0.2)?;",
    "fade_in_all(stage, &[d, piece], 0.3)?;",
    "run_time: Some(0.35),",
    "if k == 4 {",
    "fade_in_all(stage, &[forced], 0.3)?;",
    "fade_in_all(stage, &[region, region_says], 0.5)?;",
    "fade_in_all(stage, &[step_says, first_c], 0.3)?;",
    "fade_in_all(stage, &[first], 0.6)?;",
]
scene_new = NEW[NEW.index(START) : NEW.index(STOP)]
missing = [s for s in FORMULA if s not in scene_new]
print(f"anchor formula's constants in the staged scene: {'all there' if not missing else missing}")

ok = old_faults and not new_faults and caught == len(PLANTS) and not mirror_new and mirror_old and not missing
print("RESULT", "pass" if ok else "FAIL")
sys.exit(0 if ok else 1)
