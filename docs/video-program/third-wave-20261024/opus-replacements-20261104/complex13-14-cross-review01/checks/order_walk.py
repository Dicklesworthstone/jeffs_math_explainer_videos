"""Walk 13_3 and 14_3 with the pinned engine's rooting rule, and report what stands over what on every frame.

Run from the repo root, with no arguments:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-14-cross-review01/checks/order_walk.py

Written for this review without reading Bronze's draw_order_check.py or sweep_check.py (only their outputs'
one-line results, which the handoffs quote). First it checks that the pin still says what the model assumes,
by quoting the lines. Then it interprets the film's rooting statements in source order: add_to_scene removes
and re-appends; a fade roots only what is off stage, at begin; a fade-out removes at its end. A "frame" is any
call that can capture. The model is of the draw list only: it knows nothing of pixels, and a statement form it
does not recognise is invisible to it, which the planted defects and the canonical controls are there to catch.
"""
import os
import re
import subprocess
import sys

CHAPTERS = "explainer_corpus/complex_analysis_visual/src/chapters/"
STAGE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-radius-repair01/"
RI_STAGED = STAGE + CHAPTERS + "real_integral.rs"
CO_STAGED = STAGE + "continuation14/" + CHAPTERS + "continuing.rs"
RI_BASE, CO_BASE = CHAPTERS + "real_integral.rs", CHAPTERS + "continuing.rs"
PIN = os.path.expanduser("~/.cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62/crates/")

fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def read(path):
    return open(path, encoding="utf-8").read()


print("== the pin says what this model assumes (rev a399d626)")
stage_rs, frame_rs, fading_rs = read(PIN + "fmn-mobject/src/stage.rs"), read(PIN + "fmn-anim/src/frame.rs"), read(PIN + "fmn-anim/src/fading.rs")
for text, needle, why in (
    (stage_rs, "self.remove_many_from_scene(mobs);\n        self.roots.extend_from_slice(mobs);\n        self.sort_scene_by_z_index();", "add_to_scene removes, appends, sorts by z_index"),
    (frame_rs, "if !in_scene_family(stage, mobject) {\n                stage\n                    .add_to_scene(mobject)", "a fade's begin roots only what is off stage"),
    (frame_rs, "open.current_sample = open.segment.sample(open.stepped + 1);", "the first captured sample of a play is its second, after interpolation: nothing is captured at begin"),
    (fading_rs, "opacity_zero: true,", "fade_in starts its copy at opacity zero"),
    (fading_rs, "t.state_mut().config.final_alpha_value = 0.0;", "fade_out finishes at alpha zero, its begin-time state"),
):
    check(needle in text, why)
check(re.search(r"fn sort_scene_by_z_index[^}]*sort_by", stage_rs, re.S) is not None and "sort_unstable" not in stage_rs[stage_rs.index("fn sort_scene_by_z_index") :][:600], "the z_index sort is a stable sort_by, as root read")
chapters_and_kit = subprocess.run(["rg", "-c", r"z_index|set_z", CHAPTERS, "crates/explainer_kit/src"], capture_output=True, text=True, check=False)
check(chapters_and_kit.stdout == "", "no z_index in this crate's chapters or the kit, so the sort changes nothing")
draw = read(CHAPTERS + "draw.rs")
until = draw[draw.index("pub fn until(") :]
until = until[: until.index("\n    }\n")]
check("stage.wait" not in until and "wait_until" not in until, "line.until only computes a run time: it captures nothing")
updater = read(PIN + "fmn-mobject/src/stage.rs")
check("updater: impl FnMut(&mut Stage, Mob) + 'static," in updater, "an updater receives the mobject Stage, on which VStyle is implemented")
style = read(PIN + "fmn-library/src/style.rs")
check("impl VStyle for Stage {" in style and "VStyle" in read(PIN + "fmn/src/prelude.rs"), "and VStyle is in the prelude the chapter imports")
transform_rs = read(PIN + "fmn-anim/src/transform.rs")
check("with_replace_flag" not in fading_rs and "replace_in_scene: false," in transform_rs, "a fade never sets the replace flag, so its clean-up re-adds nothing; only a fade-out's remover rule touches the scene")
check(frame_rs.count("fn in_scene_family") == 1 and ".any(|root| stage.family(root).contains(&mob))" in frame_rs, "and being on stage means being in any rooted family")
marks = read("explainer_corpus/eigen_engineering/src/chapters/marks.rs")
check("stage.add_to_scene(m)?;\n            fade_in_all(stage, &[m], 0.8)?;" in marks, "corpus precedent for rooting and then fading: eigen_engineering marks.rs, same workspace pin; its pixels were not inspected here")
bakery = read("explainer_corpus/bakery_algorithm/src/chapters/scoreboard.rs")
check(re.search(r"add_dt_updater\(.*?s\.set_opacity\(m, [^;]*true\)", bakery, re.S) is not None, "precedent at this same pin: bakery_algorithm's scoreboard calls s.set_opacity inside a dt updater")

COMMENT = re.compile(r"//[^\n]*")
FRAME = r"stage\.play\(|stage\.play_prepared\(|stage\.wait\(|line\.wait_\w+\(|wait_until\(|kit\.hold\(|reading_window\(|pause\(|precisely_\w+\(|names_now\(|predict\("
TOKEN = re.compile(
    r"for m in \[(?P<list>(?:[^\[\]]|\[[^\]]*\])*)\] \{\s*stage\.add_to_scene\(m\)\?;"
    r"|stage\.add_to_scene\((?P<add>[\w\[\]]+)\)\?;"
    r"|stage\.remove_from_scene\((?P<rem>[\w\[\]]+)\);"
    r"|root_under\(stage, &\[(?P<rlist>[^\]]*)\], &(?P<mark>\w+)\)\?;"
    r"|fade_in_all\(stage, &(?P<fin>[^;]*?), [\d.]+\)\?;"
    r"|fade_out_all\(stage, &(?P<fout>[^;]*?), [\d.]+\)\?;"
    r"|(?P<frame>" + FRAME + r")"
)
EXPAND = {
    "inside": ["inside[0]", "inside[1]"],
    "outside": ["outside[0]", "outside[1]"],
    "pole_names": ["pole_names[0]", "pole_names[1]"],
    "pole1": ["pole1[0]", "pole1[1]"],
    "pole_top": ["pole_top[0]", "pole_top[1]"],
    "arrows[..1]": ["arrows[0]"],
    "arrows[1..2]": ["arrows[1]"],
    "arrows[2..]": ["arrows[2]", "arrows[3]"],
    "gone": [],
}


def names(text):
    text = text.strip()
    if text.startswith("["):
        out = []
        for part in text[1:-1].split(","):
            if part.strip():
                out.extend(EXPAND.get(part.strip(), [part.strip()]))
        return out
    return EXPAND.get(text, [text])


class Walk:
    def __init__(self):
        self.roots, self.frames, self.rooted_unseen = [], [], []

    def add(self, m):
        if m in self.roots:
            self.roots.remove(m)
        self.roots.append(m)

    def frame(self, label):
        self.frames.append((label, list(self.roots), list(self.rooted_unseen)))

    def run(self, code, suffix=""):
        for t in TOKEN.finditer(code):
            if t["list"] is not None:
                for m in names("[" + t["list"] + "]"):
                    self.add(m + suffix if m in ("d", "piece") else m)
                    self.rooted_unseen.append(m + suffix if m in ("d", "piece") else m)
            elif t["add"]:
                m = t["add"]
                self.add(m)
                self.rooted_unseen.append(m)
            elif t["rem"]:
                if t["rem"] in self.roots:
                    self.roots.remove(t["rem"])
            elif t["rlist"] is not None:
                for m in names("[" + t["rlist"] + "]") + names(t["mark"]):
                    m = m + suffix if m in ("d", "piece") else m
                    self.add(m)
                    self.rooted_unseen.append(m)
            elif t["fin"]:
                listed = [m + suffix if m in ("d", "piece") else m for m in names(t["fin"])]
                for m in listed:
                    if m not in self.roots:
                        self.roots.append(m)
                self.rooted_unseen = [m for m in self.rooted_unseen if m not in listed]
                self.frame("fade_in " + ",".join(listed))
            elif t["fout"]:
                listed = names(t["fout"])
                self.frame("fade_out " + ",".join(listed))
                self.roots = [m for m in self.roots if m not in listed]
            else:
                self.frame(t["frame"])


def cut(text, a, b):
    code = text[: text.index("#[cfg(test)]\nmod tests")]
    code = COMMENT.sub("", code)
    return code[code.index(a) : code.index(b, code.index(a))]


def walk_13(text):
    w = Walk()
    w.run(cut(text, 'say(&kit, stage, "13_2")', 'say(&kit, stage, "13_4")'))
    return w


def walk_14(text):
    code = cut(text, 'say(&kit, stage, "14_3")', 'say(&kit, stage, "14_4")')
    head, rest = code.split("for (k, &c) in centres.iter().enumerate().skip(1) {", 1)
    depth, at = 1, 0
    while depth:
        depth += {"{": 1, "}": -1}.get(rest[at], 0)
        at += 1
    body, tail = rest[: at - 1], rest[at:]
    w = Walk()
    w.run(head)
    for k in range(1, 9):
        piece = body if k == 4 else re.sub(r"if k == 4 \{.*?\n            \}", "", body, flags=re.S)
        w.run(piece, f"#{k}")
    w.run(tail)
    return w


def faults_13(w):
    out = []
    covers = ["arrows[0]", "inside[0]", "inside[1]", "pole_names[0]"]
    shown = 0
    for label, roots, unseen in w.frames:
        if "spoke" not in roots:
            continue
        shown += 1
        spoke = roots.index("spoke")
        for m in covers + ["i_back"]:
            if m in roots and roots.index(m) < spoke:
                out.append(f"{label}: spoke over {m}")
            if m == "i_back" and m not in roots:
                out.append(f"{label}: no i_back")
        for m in ("axes", "path"):
            if m in roots and roots.index(m) > spoke:
                out.append(f"{label}: {m} over the spoke")
        if "i_back" in roots and roots.index("i_back") > roots.index("pole_names[0]"):
            out.append(f"{label}: the backing over its letter")
        if "r_name" in roots and "r_back" in roots and roots.index("r_back") > roots.index("r_name"):
            out.append(f"{label}: R's backing over R")
        if "r_name" in roots and "r_back" not in roots:
            out.append(f"{label}: R without its backing")
    if not shown:
        out.append("the spoke is never on stage")
    return out


def faults_14(w):
    out = []
    reach = re.compile(r"^(region|first|d#\d)$")
    checked = 0
    for label, roots, unseen in w.frames:
        if "pole1[0]" not in roots:
            continue
        checked += 1
        back, ring = roots.index("pole1[0]"), roots.index("pole1[1]")
        if back > ring:
            out.append(f"{label}: the backing over the ring")
        for m in roots:
            if reach.match(m) and roots.index(m) > back:
                out.append(f"{label}: {m} over the mark")
    if not checked:
        out.append("the mark is never on stage")
    return out


def flashes(w, allowed):
    """What was rooted at full paint and caught by a frame before its fade began."""
    out = []
    for label, roots, unseen in w.frames:
        for m in unseen:
            if m not in allowed and m in roots:
                out.append(f"{m} caught by {label}")
    return sorted(set(out))


# pre-existing mobs re-added on purpose are already visible; those drawn without a fade are named here and checked in pixels
# r_fact and b_fact replace the live readouts in one step, in canonical too (they swap at the same place and paint)
ALLOW_13 = {"path", "anchor", "trace", "arrows[0]", "inside[0]", "inside[1]", "pole_names[0]", "i_back", "r_back", "r_name", "r_fact", "b_fact"}
ALLOW_14 = {"pole1[0]", "pole1[1]"}

print("== 13_3 with the patch")
staged_ri, staged_co = read(RI_STAGED), read(CO_STAGED)
w13 = walk_13(staged_ri)
found = faults_13(w13)
check(not found, f"{len(w13.frames)} frame-producing calls from 13_2 to 13_4: the spoke under arrow, ring, backing and name on all that show it; R over its backing" + (" " + str(found[:4]) if found else ""))
found = flashes(w13, ALLOW_13)
check(not found, "no mob rooted early is captured at full paint before its fade" + (" " + str(found[:4]) if found else ""))
on = next(r for label, r, u in w13.frames if "spoke" in r and "r_name" in r)
print("     draw list from the spoke up, during the walk: " + " < ".join(on[on.index("spoke") :]))
check(on.index("zdot") > on.index("spoke"), "and the walker over the spoke's end")

print("== 14_3 with the patch")
w14 = walk_14(staged_co)
found = faults_14(w14)
over = [f for f in w14.frames if "pole1[0]" in f[1]]
check(not found, f"{len(over)} frame-producing calls with the mark up: no region, gold or green disk ever over it" + (" " + str(found[:4]) if found else ""))
found = flashes(w14, ALLOW_14)
check(not found, "no disk or region is captured at full paint before its fade" + (" " + str(found[:4]) if found else ""))
last = over[-1][1]
print("     draw list at the chain's end, top five: " + " < ".join(last[-5:]))
check(not any(m.startswith("pole_top") for m in last), "one mark only")

print("== controls: the model must see the faults in canonical")
base13, base14 = walk_13(read(RI_BASE)), walk_14(read(CO_BASE))
check(len(faults_13(base13)) > 0, f"canonical 13_3 faults, {len(faults_13(base13))}: {faults_13(base13)[0]}")
check(len(faults_14(base14)) > 0, f"canonical 14_3 faults, {len(faults_14(base14))}: {faults_14(base14)[0]}")

print("== controls: planted defects, each must be seen")
PLANTS_13 = [
    ("spoke rooted after the re-adds", "        stage.add_to_scene(spoke)?;\n        for m in [arrows[0], i_back, inside[0], inside[1], pole_names[0]] {\n            stage.add_to_scene(m)?;\n        }\n", "        for m in [arrows[0], i_back, inside[0], inside[1], pole_names[0]] {\n            stage.add_to_scene(m)?;\n        }\n        stage.add_to_scene(spoke)?;\n", faults_13),
    ("the arrow left out of the re-adds", "for m in [arrows[0], i_back,", "for m in [i_back,", faults_13),
    ("the ring left out of the re-adds", "i_back, inside[0], inside[1], pole_names[0]]", "i_back, pole_names[0]]", faults_13),
    ("the backing over the letter", "i_back, inside[0], inside[1], pole_names[0]]", "inside[0], inside[1], pole_names[0], i_back]", faults_13),
    ("R's backing over R", "stage.add_to_scene(r_back)?;\n        stage.add_to_scene(r_name)?;", "stage.add_to_scene(r_name)?;\n        stage.add_to_scene(r_back)?;", faults_13),
    ("a wait between the spoke's rooting and its fade", "        fade_in_all(stage, &[zdot, spoke, zsq, u_axis, along], 0.4)?;", "        stage.wait(0.1)?;\n        fade_in_all(stage, &[zdot, spoke, zsq, u_axis, along], 0.4)?;", lambda w: flashes(w, ALLOW_13)),
]
TEST_MARK = "#[cfg(test)]\nmod tests"
film_ri = staged_ri[: staged_ri.index(TEST_MARK)]  # the tests quote some of these lines; plant in the film code only
for what, old, new, test in PLANTS_13:
    assert film_ri.count(old) == 1, what
    check(len(test(walk_13(film_ri.replace(old, new) + TEST_MARK))) > 0, what)
PLANTS_14 = [
    ("the region's fade before its rooting", "        root_under(stage, &[region], &pole1)?;\n        fade_in_all(stage, &[region, region_says], 0.5)?;", "        fade_in_all(stage, &[region, region_says], 0.5)?;\n        root_under(stage, &[region], &pole1)?;", faults_14),
    ("the gold disk not routed under", "        root_under(stage, &[first], &pole1)?;\n", "", faults_14),
    ("the chain's disks not routed under", "            root_under(stage, &[d, piece], &pole1)?;\n", "", faults_14),
    ("a wait between a disk's rooting and its fade", "            fade_in_all(stage, &[d, piece], 0.3)?;", "            stage.wait(0.1)?;\n            fade_in_all(stage, &[d, piece], 0.3)?;", lambda w: flashes(w, ALLOW_14)),
    ("the mark re-added under the disk", "root_under(stage, &[d, piece], &pole1)?;", "for m in pole1 { stage.add_to_scene(m)?; }\n            stage.add_to_scene(d)?;\n            stage.add_to_scene(piece)?;", faults_14),
]
film_co = staged_co[: staged_co.index(TEST_MARK)]
for what, old, new, test in PLANTS_14:
    assert film_co.count(old) == 1, what
    check(len(test(walk_14(film_co.replace(old, new) + TEST_MARK))) > 0, what)

print("== updater lifetime in 13_3")
film = staged_ri[: staged_ri.index("#[cfg(test)]\nmod tests")]
three = film[film.index('say(&kit, stage, "13_3")') : film.index('say(&kit, stage, "13_4")')]
walk_at = three.index("stage.play(tween_linear(walk, PI, rt)?)?;")
clear_at = three.index("stage.clear_updaters(anchor, true);", walk_at)
removed_at = three.index("stage.remove_from_scene(anchor);", walk_at)
fade_at = three.index("fade_out_all(stage, &[zdot, spoke, r_name, r_back, i_back], 0.3)?;")
second = three.index("let anchor = stage.arena_mut().add(VMobject::new());", fade_at)
check(walk_at < clear_at < removed_at < fade_at, "the walk's updater is cleared and its anchor removed right after the walk, well before the fade, so it cannot write opacity 1 over the fade-out")
between = three[walk_at + len("stage.play(tween_linear(walk, PI, rt)?)?;") : clear_at]
check(between.strip() == "", "nothing runs between the walk's end and the clear")
grown = three[second : three.index("stage.clear_updaters(anchor, true);", second)]
check(not re.search(r"\b(r_name|r_back|i_back|spoke|zdot)\b", grown), "the growth's updater names none of the removed mobs")
check(re.search(r"\b(i_back)\b", three[three.index("move |a, _|") : clear_at]) is None, "i_back has no updater: it stays put, as its letter does until the growth, by when it is gone")
order = [three.index(s) for s in ("stage.add_to_scene(r_back)?;", "stage.add_to_scene(r_name)?;", "stage.add_updater(\n            anchor,", "let rt = line.until", "stage.play(tween_linear(walk, PI, rt)?)?;")]
check(order == sorted(order), "R and its backing are rooted, then the updater registers with call = true and writes opacity 0, and only then can a frame be captured")
span = three[order[0] : order[2]]
check(not re.search(FRAME, span), "no frame-producing call stands between R's rooting and that first write")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
