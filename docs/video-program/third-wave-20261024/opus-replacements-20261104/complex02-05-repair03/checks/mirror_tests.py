"""Python mirror of the Rust tests this repair adds, because no Cargo may run in this lane.

Not a compile and not `cargo test`. It reads the staged sources and the canonical script as
text, evaluates the same constants, and repeats each new test's arithmetic and source-order
assertions. It then proves each mirror can fail, three ways:

  * baseline: every mirror passes on the staged sources;
  * planted: each listed one-line defect fails the mirror it names, for the reason it names,
    while the other mirrors stay green;
  * canonical control: the mirrors of the repaired behaviour fail on the canonical sources.

Half sizes marked as estimates in the Rust tests are estimates here too: they stand in for
measured glyph boxes, and the pictures decide.

Usage, from the repository root: python3 -I -B mirror_tests.py REPO STAGED_CHAPTERS
"""

import cmath
import json
import math
import re
import sys
from pathlib import Path

TAU = math.tau
PI = math.pi
STEP = TAU / 12.0
OFFSET = 0.5
NAMES = {
    "PI": PI,
    "TAU": TAU,
    "FRAC_PI_2": PI / 2.0,
    "FRAC_PI_4": PI / 4.0,
}


class World:
    def __init__(self, sources, script, model, draw):
        self.src = dict(sources)
        self.script = script
        self.model = model
        self.draw = draw

    def code(self, name):
        text = self.src[name]
        marker = "#[cfg(test)]\nmod tests"
        assert marker in text, f"{name}: no test module"
        return text[: text.index(marker)]

    def beat(self, name, start, end):
        code = self.code(name)
        assert start in code, f"{name}: {start} not found"
        at = code.index(start)
        assert end in code[at:], f"{name}: {end} not found after {start}"
        return code[at : at + code[at:].index(end)]

    def say(self, name, cue, end):
        return self.beat(name, f'say(&kit, stage, "{cue}")', end)

    def next_say(self, name, cue, next_cue):
        return self.say(name, cue, f'say(&kit, stage, "{next_cue}")')

    def const_text(self, name, const):
        m = re.search(rf"pub const {const}\s*:\s*[^=]+=\s*(.*?);", self.src[name], re.S)
        assert m, f"{name}: no const {const}"
        return m.group(1).strip()

    def num(self, name, const):
        return eval(self.const_text(name, const), {"__builtins__": {}}, NAMES)  # noqa: S307

    def nums(self, name, const):
        body = self.const_text(name, const).strip("[]() \n")
        return [eval(x, {"__builtins__": {}}, NAMES) for x in re.split(r",\s*", body) if x]  # noqa: S307

    def cs(self, name, const):
        body = self.const_text(name, const)
        found = re.findall(r"C::new\(\s*([^,]+?)\s*,\s*([^)]+?)\s*,?\s*\)", body)
        assert found, f"{name}: {const} has no C::new"
        return [
            complex(eval(a, {"__builtins__": {}}, NAMES), eval(b, {"__builtins__": {}}, NAMES))  # noqa: S307
            for a, b in found
        ]

    def strs(self, name, const):
        return re.findall(r'r?"((?:[^"\\]|\\.)*)"', self.const_text(name, const))


def in_order(section, order):
    at = 0
    for piece in order:
        found = section.find(piece, at)
        assert found >= 0, f"{piece}: missing or early"
        at = found + len(piece)


def plane(center, width, x, y):
    height = width * (y[1] - y[0]) / (x[1] - x[0])
    return dict(center=center, width=width, height=height, x=x, y=y, unit=width / (x[1] - x[0]))


def p_of(pl, z):
    u = (z.real - pl["x"][0]) / (pl["x"][1] - pl["x"][0])
    w = (z.imag - pl["y"][0]) / (pl["y"][1] - pl["y"][0])
    return (
        pl["center"][0] + (u - 0.5) * pl["width"],
        pl["center"][1] + (w - 0.5) * pl["height"],
    )


def contains(pl, z):
    return pl["x"][0] <= z.real <= pl["x"][1] and pl["y"][0] <= z.imag <= pl["y"][1]


def planes(i_half, o_half):
    return (
        plane((-3.5, -0.35), 5.4, (-i_half, i_half), (-i_half, i_half)),
        plane((3.5, -0.35), 5.4, (-o_half, o_half), (-o_half, o_half)),
    )


def wheel(world, angle):
    table = world.model["wheel"]
    t = (angle % TAU) / TAU * len(table)
    k = min(int(math.floor(t)), len(table) - 1)
    f = t - k
    a, b = table[k], table[(k + 1) % len(table)]
    return [(a[c] + (b[c] - a[c]) * f) / 255.0 for c in range(3)]


def shade(w):
    size = abs(w)
    size_band = (math.log(size) / STEP + OFFSET) % 1.0
    phase_band = ((cmath.phase(w) % TAU) / STEP + OFFSET) % 1.0
    return (0.58 + 0.42 * size_band) * (0.80 + 0.20 * phase_band)


def turns_so_far(f, r, t, samples=720):
    k = round(min(max(t, 0.0), 1.0) * samples)
    total = 0.0
    prev = f(cmath.rect(r, 0.0))
    for j in range(1, k + 1):
        w = f(cmath.rect(r, TAU * j / samples))
        total += cmath.phase(w / prev)
        prev = w
    return total / TAU


SQUARE = lambda z: z * z  # noqa: E731
RECIP = lambda z: 1.0 / z  # noqa: E731


# ---------------------------------------------------------------- turning.rs


def t_recalled_turn(w):
    beat = w.say("turning", "02_5", "let asked = predict(")
    hook = beat.find('line.wait_for(stage, "an eighth of a turn")?;')
    sweep = beat.find("sweep_arc(stage, tp, ANGLE_ARC_R, TURN_BY.arg(), SWEEP_S)?;")
    assert hook >= 0, "no hook on the recalled turn"
    assert sweep >= 0, "no sweep"
    assert hook < sweep, "sweep before its words"
    for answer in ["TURN_BY * TURN_BY", "grid.multiply", "w2_arrow", "total_", "SQUARE_TEX"]:
        assert answer not in beat, f"{answer} answers the question early"
    pulse = w.nums("turning", "PULSE")
    total = w.num("turning", "SWEEP_S") + pulse[1] + pulse[0] + pulse[1]
    assert total < 2.0, f"the fixed beat runs {total} s"


def t_last_beat(w):
    beat = w.say("turning", "02_6", "kit.finish(stage)")
    in_order(
        beat,
        [
            "fade_in_all(stage, &[rule], 0.4)?;",
            "underline_pulse(stage, &[length, total_stretch])?;",
            'line.wait_for(stage, "and angles add")?;',
            "underline_pulse(stage, &[angle_says, total_turn])?;",
            'line.wait_for(stage, "Every complex derivative")?;',
            "fade_out_all(stage, &grid.mobs(), 0.5)?;",
            "kit.music_breath(stage, 2.0)?;",
        ],
    )
    ending = beat[beat.index("kit.music_breath(stage, 2.0)?;") :]
    for twice in ["grid.mobs()", "stretch_turn"]:
        assert twice not in ending, f"{twice} fades twice"
    assert "w_arrow" in ending and "w2_arrow" in ending, "the arrows do not close the chapter"
    head = beat[: beat.index('let rt = line.until(stage, "So one plus i", 2.5)?;')]
    assert "gone.extend(stretch_turn);" in head, "the caption stays up"
    assert "fade_out_all(stage, &gone, 0.3)?;" in head, "the caption stays up"


def sentence_bytes(text):
    raw = text.encode("utf8")
    out = [0]
    for i, c in enumerate(raw):
        if c in b".?!" and raw[i + 1 : i + 2] == b" ":
            out.append(i + 2)
    return out


def t_hooks(w):
    for name in ("turning", "two_planes", "reading_color", "gallery"):
        code = w.code(name)
        says = [m.start() for m in re.finditer(re.escape('let line = say(&kit, stage, "'), code)]
        assert says, f"{name}: no cues"
        for k, at in enumerate(says):
            end = says[k + 1] if k + 1 < len(says) else len(code)
            body = code[at:end]
            cue = body.split('"')[1]
            assert cue in w.script, f"{cue}: not a cue"
            text = w.script[cue]
            raw = text.encode("utf8")
            starts = sentence_bytes(text)
            hooks = []
            for form in ('line.wait_for(stage, "', 'line.until(stage, "'):
                for m in re.finditer(re.escape(form), body):
                    phrase = body[m.end() :].split('"')[0]
                    byte = raw.find(phrase.encode("utf8"))
                    assert byte >= 0, f"{cue}: the voice never says {phrase!r}"
                    hooks.append((m.start(), byte, phrase))
            form = "line.wait_sentence(stage, "
            for m in re.finditer(re.escape(form), body):
                n = body[m.end() :].split(")")[0]
                if n.isdigit():
                    assert int(n) < len(starts), f"{cue}: no sentence {n}"
                    hooks.append((m.start(), starts[int(n)], f"sentence {n}"))
            hooks.sort()
            for a, b in zip(hooks, hooks[1:]):
                assert a[1] <= b[1], f"{cue}: out of the voice's order: {a[2]} then {b[2]}"


# ------------------------------------------------------------- two_planes.rs


def t_radii(w):
    size = w.num("two_planes", "RADIUS_SIZE")
    assert size >= 24.0, "the spoken radii are written too small"
    at = w.cs("two_planes", "RADIUS_AT")
    unit_name = w.cs("two_planes", "UNIT_AT")[0]
    r = w.num("two_planes", "WALK_R")
    half_h = 0.12
    for z, half, radius in [(at[0], 0.438, r), (at[1], 0.49, r * r)]:
        assert z.real - half > 0.1, f"{z}: reaches the axis"
        assert z.imag + half_h < -0.1, f"{z}: reaches the axis"
        near = abs(complex(z.real - half, z.imag + half_h))
        assert near > radius + 0.1, f"{z}: reaches its circle"
        assert z.imag + half_h < unit_name.imag - 0.06 - 0.05, f"{z}: reaches the 1"


def t_rule(w):
    n = "two_planes"
    window = w.num(n, "WINDOW")
    inp, out = planes(window, window)
    length, angle = w.num(n, "RULE_LENGTH"), w.num(n, "RULE_ANGLE")
    z = cmath.rect(length, angle)
    sq = z * z
    assert abs(abs(sq) - abs(z) ** 2) < 1e-12, "length not squared"
    assert abs(cmath.phase(sq) - 2 * cmath.phase(z)) < 1e-12, "angle not doubled"
    assert contains(inp, z) and contains(out, sq), "the point leaves its window"
    assert length > 2 * w.num(n, "WALK_R"), "it sits on the walk's circle"
    assert abs(sq) - abs(z) > 0.5, "the length barely grows"
    assert cmath.phase(sq) - cmath.phase(z) > 0.5, "the angle barely grows"
    length_at = w.cs(n, "RULE_LENGTH_AT")
    angle_at = w.cs(n, "RULE_ANGLE_AT")
    for pl, a, b in zip((inp, out), length_at, angle_at):
        assert contains(pl, a) and contains(pl, b), "a label leaves its window"
    arc_r = w.num(n, "RULE_ARC_R")
    for at, half, arrow in [(angle_at[0], 0.2, z), (angle_at[1], 0.26, sq)]:
        corner = complex(at.real - half, at.imag - 0.1)
        assert abs(corner) > arc_r + 0.05 and corner.imag > 0.1, f"{at}: on its arc or axis"
        top_left = complex(at.real - half, at.imag + 0.1)
        assert cmath.phase(top_left) < cmath.phase(arrow) - 0.05, f"{at}: reaches its arrow"
    names = w.cs(n, "UNIT_AT")
    for at, tip in [(length_at[0], z), (length_at[1], sq)]:
        assert abs(at - tip) > 0.2 and abs(at) > abs(tip), f"{at}: not past the tip"
        for name in names:
            assert abs(at - name) > 0.4, f"{at}: on the plane's name"
    rule = w.strs(n, "SQUARE_RULE_TEX")[0]
    assert r"\arg z" in rule and r"2 \arg z" in rule and "|z|^2" in rule, "the rule changed"
    assert w.strs(n, "RULE_LENGTH_TEX") == ["|z|", "|z|^2"], "labels are not the rule's pieces"


def t_not_empty(w):
    in_order(
        w.next_say("two_planes", "03_2", "03_3"),
        [
            "fade_in_all(stage, &[rule], 0.4)?;",
            'line.wait_for(stage, "squares each length")?;',
            "fade_in_all(stage, &shown_rule, 0.4)?;",
            'line.wait_for(stage, "doubles each angle")?;',
            "fade_in_all(stage, &angles, 0.4)?;",
            "line.wait_sentence(stage, 2)?;",
            "fade_out_all(stage, &shown_rule, 0.3)?;",
            "let ring_in = ",
        ],
    )


def image_laps(r, t):
    t = min(max(t, 0.0), 1.0)

    def share(start, end):
        a, b = start, max(end, start + 1e-6)
        k = max(math.ceil((b - a) * 240.0), 1)
        out = []
        for j in range(k + 1):
            z = cmath.rect(r, TAU * (a + (b - a) * j / k))
            out.append(z * z)
        return out

    return share(0.0, min(t, 0.5)), share(0.5, max(t, 0.5))


def t_laps(w):
    n = "two_planes"
    r = w.num(n, "WALK_R")
    for t in (0.0, 0.1, 0.25, 0.5, 0.75, 1.0):
        first, second = image_laps(r, t)
        assert len(first) >= 2 and len(second) >= 2, "a lap has under two points"
        for z in first + second:
            assert abs(abs(z) - r * r) < 1e-12, "off the image circle"
    first, second = image_laps(r, 0.5)
    assert abs(first[0] - first[-1]) < 1e-5, "the first half does not close a lap"
    assert abs(second[0] - second[-1]) < 1e-3, "the second lap started early"
    first, second = image_laps(r, 1.0)
    assert abs(first[-1] - second[0]) < 1e-12, "the laps do not join"
    assert abs(second[0] - second[-1]) < 1e-5, "the second half does not close a lap"
    two = w.next_say(n, "03_2", "03_3")
    in_order(
        two,
        [
            "tween_linear(walked, 0.5, rt / 2.0)",
            "stage.add_to_scene(second_lap)?;",
            "stage.add_to_scene(w_out)?;",
            "tween_linear(walked, 1.0, rt / 2.0)",
        ],
    )
    assert "trail, second_lap]" in w.next_say(n, "03_3", "03_4"), "the second lap never fades"


def t_flip(w):
    n = "two_planes"
    window = w.num(n, "WINDOW")
    _, out = planes(window, window)
    buff = w.num(n, "FLIP_BUFF")
    for th in w.nums(n, "TICKS"):
        a = cmath.rect(1.0, th)
        b = 1.0 / a
        assert abs(a.real - b.real) < 1e-12 and abs(a.imag + b.imag) < 1e-12, "not a flip"
        assert contains(out, a) and contains(out, b), "leaves the plane"
        length = abs(a - b) * out["unit"]
        assert length > 2 * buff + 0.5, f"{th}: only {length} long"
    in_order(
        w.next_say(n, "03_5", "03_6"),
        [
            "ticks.extend([a, b]);",
            'line.wait_for(stage, "each point flips")?;',
            "flips.extend([ghost, chord]);",
            "line.wait_sentence(stage, 3)?;",
        ],
    )
    assert "gone.extend(flips);" in w.next_say(n, "03_6", "03_7"), "the flips never fade"


def t_ask(w):
    n = "two_planes"
    window = w.num(n, "WINDOW")
    _, out = planes(window, window)
    ask = w.cs(n, "ASK_AT")[0]
    at, zero = p_of(out, ask), p_of(out, 0j)
    card_h = 28.0 / 48.0 * 0.62 + 0.3
    ring_y = at[1] - card_h / 2.0 - 0.45
    ring_r = 0.26
    bottom = out["center"][1] - out["height"] / 2.0
    assert at[0] - ring_r > zero[0] + 0.1, "the ring reaches the imaginary axis"
    assert ring_y - ring_r > bottom, "the ring leaves the plane"
    name = p_of(out, w.cs(n, "UNIT_AT")[0])
    assert at[1] + card_h / 2.0 < name[1] - 0.09, "the card covers the 1"
    assert at[1] - card_h / 2.0 > p_of(out, complex(0, -1))[1], "the card covers a tick"
    half_w = (8.2 * 28.0 * 0.01 + 0.5) / 2.0
    left, right = out["center"][0] - 2.7, out["center"][0] + 2.7
    assert at[0] - half_w > left and at[0] + half_w < right, "the card leaves the plane"
    assert "out.p(ASK_AT)" in w.next_say(n, "03_3", "03_4"), "the card ignores ASK_AT"


def t_labels(w):
    n = "two_planes"
    said = w.script["03_4"]
    for label, word in zip(w.strs(n, "RAYS"), ("horizontal axis", "vertical axis")):
        assert label.startswith(word) and word in said, f"{label}: not the voice's words"
    six = w.next_say(n, "03_6", "03_7")
    in_order(six, ["line.wait_sentence(stage, 2)?;", "fade_out_all(stage, &[rescaled], 0.3)?;"])
    assert "rescaled" not in w.say(n, "03_7", "kit.music_breath"), "the note stays for the climb"
    assert "vec![rescaled]" not in six, "the note fades twice"


# ---------------------------------------------------------- reading_color.rs


def needle(f, r, t):
    return cmath.phase(f(cmath.rect(r, TAU * t)))


def t_needle(w):
    n = "reading_color"
    r = w.num(n, "LOOP_R")
    body = w.const_fn(n, "needle_angle")
    assert "f.eval(loop_point(t))" in body, "the needle does not read f's output"
    assert "w.arg()" in body, "the needle does not read the output's direction"
    for f, turns in ((SQUARE, 2.0), (RECIP, -1.0)):
        for k in range(97):
            t = k / 96.0 + 0.003
            off = (needle(f, r, t) - turns * TAU * t) % TAU
            assert min(off, TAU - off) < 1e-9, f"at {t}: {off}"
    assert abs(cmath.rect(r, 0.0) - cmath.rect(r, TAU)) < 1e-12, "a lap does not close"
    # the wedge under the needle is the colour under the dot
    for f in (SQUARE, RECIP):
        for k in range(24):
            t = k / 24.0 + 0.003
            a, b = wheel(w, cmath.phase(f(cmath.rect(r, TAU * t)))), wheel(w, needle(f, r, t))
            assert all(abs(x - y) < 1e-12 for x, y in zip(a, b)), "the needle leaves the colour"


def t_counts(w):
    n = "reading_color"
    r = w.num(n, "LOOP_R")
    for f, stated in ((SQUARE, 2.0), (RECIP, -1.0)):
        assert abs(turns_so_far(f, r, 1.0) - stated) < 1e-9, "the count disagrees with the card"
        assert abs(turns_so_far(f, r, 0.5) - stated / 2.0) < 1e-9, "the count is not steady"
    below = w.num(n, "COUNT_BELOW")
    y = w.num(n, "WHEEL_Y") - w.num(n, "WHEEL_R") - below
    assert below > 0.2, "the count touches the wheel"
    assert y - 0.15 > w.num(n, "STATED_Y") + 0.15, "the count reaches the stated row"


def t_cycle(w):
    n = "reading_color"
    in_order(
        w.next_say(n, "04_3", "04_4"),
        [
            "painter.to(stage, Some(Func::Square), 1.2)?;",
            "let squared = ",
            "fade_in_all(stage, &ring, 0.5)?;",
            "line.wait_sentence(stage, 2)?;",
            "LoopWalk::start(&kit, stage, Func::Square, p, wheel_at)?;",
            "walk.lap(stage, rt)?;",
        ],
    )
    in_order(
        w.next_say(n, "04_4", "04_5"),
        [
            "fade_in_all(stage, &zero, 0.3)?;",
            'line.until(stage, "Once for a simple zero", 2.0)?;',
            "walk.lap(stage, rt)?;",
            'line.wait_for(stage, "twice for a zero of order two")?;',
            "fade_in_all(stage, &order, 0.3)?;",
            "line.wait_sentence(stage, 2)?;",
            "walk.stop(stage);",
            "fade_out_all(stage, &walked, 0.3)?;",
            "wait_until(stage, line.end())?;",
            "let exact = ",
            "reading_window(&kit, stage, &line)?;",
        ],
    )
    in_order(
        w.next_say(n, "04_5", "04_6"),
        [
            "painter.to(stage, Some(Func::Recip), 1.2)?;",
            "let inverse = ",
            "line.wait_sentence(stage, 1)?;",
            "fade_in_all(stage, &named, 0.3)?;",
            "LoopWalk::start(&kit, stage, Func::Recip, p, wheel_at)?;",
            "walk.lap(stage, rt)?;",
            "walk.stop(stage);",
        ],
    )
    in_order(
        w.next_say(n, "04_6", "04_7"),
        [
            "clearing.extend(&walked);",
            "fade_out_all(stage, &clearing, 0.4)?;",
            "painter.to(stage, None, 0.8)?;",
            "let bump = ",
            "let asked = predict",
        ],
    )


def t_gold(w):
    in_order(
        w.say("reading_color", "04_7", "kit.music_breath"),
        [
            "stage.add_to_scene(touching)?;",
            "stage.add_to_scene(segment)?;",
            "for &m in &bump_marks {",
            "stage.add_to_scene(m)?;",
            "fade_in_all(stage, &[touching, segment, range_says], 0.6)?;",
            "fade_out_all(stage, &[unit], 0.2)?;",
        ],
    )


def t_exact_card(w):
    text = w.strs("reading_color", "NEAR_ZERO")[0]
    assert "zero z = a" in text and "zero a " not in text, "the card says zero a"
    assert len(text) <= 115, f"{len(text)} characters"


def t_windows(w):
    """The two windows mod.rs guards on these chapters, repeated on the new card text."""
    breath, comes_in, per_s = 0.6, 0.4, 15.0
    near = w.strs("reading_color", "NEAR_ZERO")[0]
    four = w.strs("gallery", "FOUR")[0]
    clean = breath - comes_in + 7.5
    assert clean >= len(near) / per_s, f"04_4: {clean} s for {len(near)} characters"
    assert clean <= 2 * len(near) / per_s, "04_4: held for nothing"
    clean = breath + 3.5
    assert clean >= len(four) / per_s, f"05_8: {clean} s for {len(four)} characters"
    assert clean <= 2 * len(four) / per_s, "05_8: held for nothing"
    for name, cue, card, next_cue in (
        ("reading_color", "04_4", "exact", "04_5"),
        ("gallery", "05_8", "four", "05_9"),
    ):
        source = w.src[name]
        here = source.index(f'let line = say(&kit, stage, "{cue}")?;')
        shown = source.index(f"let {card} = ", here)
        window = "reading_window(&kit, stage, &line)?;"
        held = source.index(window, here)
        assert shown < held, f"{cue}: out of order"
        before = source[shown:held]
        assert "fade_out" not in before and "remove_from_scene" not in before, f"{cue}: cleared early"
        then = source.index(f'let line = say(&kit, stage, "{next_cue}")?;', here)
        assert not source[held + len(window) : then].strip(), f"{cue}: {next_cue} waits on more"
        rest = source[then + 1 :]
        stop = rest.find("let line = say(&kit, stage, ")
        cleared = rest[: stop if stop >= 0 else len(rest)]
        assert "fade_out_all" in cleared, f"{cue}: nothing cleared"
        assert card in re.split(r"[^0-9A-Za-z_]+", cleared), f"{cue}: {card} not cleared"
    four_at = w.src["reading_color"]
    here = four_at.index('let line = say(&kit, stage, "04_4")?;')
    assert "wait_until(stage, line.end())?;" in four_at[here : four_at.index("let exact = ", here)], (
        "04_4: the card comes under the voice"
    )


# ---------------------------------------------------------------- gallery.rs


def t_zero_pole(w):
    assert w.model["zdp"] == (["ONE"], ["(-ONE, 2)"]), f"the function's marks changed: {w.model['zdp']}"
    in_order(
        w.next_say("gallery", "05_2", "05_3"),
        [
            "line.wait_sentence(stage, 2)?;",
            "zero_mark(stage, sp.p(ONE))",
            "fade_in_all(stage, &labels, 0.3)?;",
            'line.wait_for(stage, "and at minus one")?;',
            "pole_mark(stage, sp.p(-ONE))",
            "fade_in_all(stage, &arriving, 0.3)?;",
            "Self::walk_and_count(",
        ],
    )


def t_exp_tile(w):
    n = "gallery"
    bx, by = w.nums(n, "EXP_TILE_BAND")
    x0, y0 = STEP * (bx - OFFSET), STEP * (by - OFFSET)
    tile = [
        complex(x0, y0),
        complex(x0 + STEP, y0),
        complex(x0 + STEP, y0 + STEP),
        complex(x0, y0 + STEP),
        complex(x0, y0),
    ]
    body = w.const_fn(n, "exp_tile")
    for corner in ("C::new(x, y)", "C::new(x + s, y)", "C::new(x + s, y + s)", "C::new(x, y + s)"):
        assert corner in body, f"exp_tile lost {corner}"
    assert "model::BAND_OFFSET" in body and "model::band_step()" in body, "not on the band lines"
    for a, b in zip(tile, tile[1:]):
        assert abs(abs(b - a) - STEP) < 1e-12, "unequal sides"
    at = lambda x, y: shade(cmath.exp(complex(x, y)))  # noqa: E731
    xm, ym, e = x0 + STEP / 2.0, y0 + STEP / 2.0, 1e-6
    for a, b in (
        (at(x0 - e, ym), at(x0 + e, ym)),
        (at(x0 + STEP - e, ym), at(x0 + STEP + e, ym)),
        (at(xm, y0 - e), at(xm, y0 + e)),
        (at(xm, y0 + STEP - e), at(xm, y0 + STEP + e)),
    ):
        assert abs(a - b) > 0.1, f"a side is no step in the picture: {a} {b}"
    inset = lambda u: 1e-6 + (STEP - 2e-6) * u  # noqa: E731
    for k in range(1, 64):
        a, b = (k - 1) / 63.0, k / 63.0
        assert at(x0 + inset(a), ym) < at(x0 + inset(b), ym), "a step inside the tile"
        assert at(xm, y0 + inset(a)) < at(xm, y0 + inset(b)), "a step inside the tile"
    ep = plane((-2.6, -0.25), 3.2, (-3.0, 3.0), (-4.0, 8.0))
    for z in tile:
        assert contains(ep, z), f"{z}: out of the window"
        assert z.real > -2.55 and z.imag > -3.5, f"{z}: reaches an arrow"
    in_order(
        w.next_say(n, "05_3", "05_4"),
        [
            'line.wait_for(stage, "exact squares")?;',
            "fade_in_all(stage, &[one_square], 0.3)?;",
            'line.wait_for(stage, "repeating every two pi")?;',
            "fade_in_all(stage, &[bracket, period], 0.5)?;",
        ],
    )
    assert "one_square," in w.next_say(n, "05_4", "05_5"), "the outline never fades"


def t_climb(w):
    n = "gallery"
    stop = w.num(n, "CLIMB_STOP")
    tp = plane((-2.4, -0.1), 8.4, (-6.6, 6.6), (-3.3, 3.3))
    assert stop < 3.0, "the arrow reaches the dot"
    assert (3.0 - stop) * tp["unit"] > 0.08 + 0.03, "the tip is on the dot"
    assert "C::new(0.0, CLIMB_STOP)" in w.next_say(n, "05_4", "05_5"), "the arrow ignores CLIMB_STOP"


def t_strip(w):
    n = "gallery"
    in_order(
        w.next_say(n, "05_7", "05_8"),
        [
            "line.wait_sentence(stage, 5)?;",
            "fade_out_all(stage, &[converges, pole_says, zedge], 0.3)?;",
            "strip.show(stage, 0.8)?;",
            "stage.add_to_scene(start_box)?;",
            "for &m in &pole1 {",
            "stage.add_to_scene(m)?;",
            "fade_in_all(stage, &[start_box, sborder, band, half, squashed, critical], 0.4)?;",
        ],
    )
    in_order(
        w.say(n, "05_8", "reading_window"),
        [
            "fade_in_all(stage, &rings, 0.5)?;",
            'line.wait_for(stage, "is the Riemann hypothesis")?;',
            "fade_in_all(stage, &rh, 0.4)?;",
            "line.wait_sentence(stage, 2)?;",
            "fade_out_all(stage, &[squashed], 0.3)?;",
            "let four = ",
            "wait_until(stage, line.end())?;",
        ],
    )
    closing = w.say(n, "05_9", "kit.finish")
    for gone in ("zedge", "converges", "pole_says", "squashed"):
        assert gone not in closing, f"{gone} would fade twice"


def t_tile_ready(w):
    n = "gallery"
    beats = w.nums(n, "TILE_BEATS")
    assert sum(beats) < 1.8, f"{sum(beats)} s before the tile"
    nine = w.say(n, "05_9", "kit.finish")
    in_order(
        nine,
        [
            'line.wait_for(stage, "in little tiles")?;',
            "fade_in_all(stage, &[boxed], TILE_BEATS[0])?;",
            "leaving.push(p.group);",
            "fade_out_all(stage, &leaving, TILE_BEATS[1])?;",
            "zoom.show(stage, TILE_BEATS[2])?;",
            "fade_in_all(stage, &framing, TILE_BEATS[3])?;",
            'line.wait_for(stage, "nearly square")?;',
            "fade_in_all(stage, &[outline, tile_says], 0.5)?;",
        ],
    )
    a, b = nine.index("in little tiles"), nine.index('"nearly square"')
    assert ".leave(stage" not in nine[a:b], "a panel leaves on its own clock"
    four = w.strs(n, "FOUR")[0]
    assert four.startswith("this strip"), "the card does not say which picture"
    assert len(four) / 15.0 < 4.1, "the card outruns its window"


TESTS = {
    "turning::the_recalled_turn_moves_and_does_not_answer_the_question": t_recalled_turn,
    "turning::the_last_beat_reads_the_rule_off_the_rows_and_ends_on_the_arrows": t_last_beat,
    "turning::every_hook_is_said_and_they_run_in_the_voices_order": t_hooks,
    "two_planes::the_walk_radii_stand_off_their_circles_and_the_axes": t_radii,
    "two_planes::the_rule_is_shown_on_one_point_that_squaring_really_sends_there": t_rule,
    "two_planes::the_planes_are_not_empty_under_the_rule": t_not_empty,
    "two_planes::each_half_of_the_walk_is_one_lap_of_the_image": t_laps,
    "two_planes::each_flip_goes_straight_across_the_horizontal_axis": t_flip,
    "two_planes::the_question_and_its_ring_stand_clear_of_the_axes": t_ask,
    "two_planes::the_labels_say_what_the_voice_says": t_labels,
    "reading_color::the_needle_stands_on_the_colour_under_the_dot": t_needle,
    "reading_color::the_walk_counts_what_the_cards_state": t_counts,
    "reading_color::the_cycle_is_walked_on_the_words_that_name_it": t_cycle,
    "reading_color::the_gold_never_crosses_the_marks_and_never_leaves_a_gap": t_gold,
    "reading_color::the_exact_card_names_its_point_and_fits_its_window": t_exact_card,
    "mod::reading_windows_and_each_card_outlasts (existing tests, repeated)": t_windows,
    "gallery::the_zero_and_the_pole_come_on_their_own_words": t_zero_pole,
    "gallery::the_exponential_tile_is_an_exact_square_on_the_band_lines": t_exp_tile,
    "gallery::the_climb_stops_short_of_the_dot": t_climb,
    "gallery::the_strip_arrives_on_a_clear_column_and_under_the_pole": t_strip,
    "gallery::the_tile_is_ready_for_the_words_that_name_it": t_tile_ready,
}

# (file, old, new, the mirror that must fail, words its reason must contain)
PLANTED = [
    ("turning", 'line.wait_for(stage, "an eighth of a turn")?;\n', "", "the_recalled_turn", "no hook"),
    ("turning", "sweep_arc(stage, tp, ANGLE_ARC_R, TURN_BY.arg(), SWEEP_S)?;\n", "let _ = grid.multiply;\n", "the_recalled_turn", "no sweep"),
    ("turning", "pub const SWEEP_S: f64 = 0.7;", "pub const SWEEP_S: f64 = 1.2;", "the_recalled_turn", "fixed beat"),
    ("turning", "fade_out_all(stage, &grid.mobs(), 0.5)?;", "let _ = 0;", "the_last_beat", "grid.mobs"),
    ("turning", "gone.extend(stretch_turn);", "", "the_last_beat", "caption stays"),
    ("turning", 'line.wait_for(stage, "and angles add")?;', 'line.wait_for(stage, "and angels add")?;', "every_hook", "never says"),
    ("gallery", 'line.wait_for(stage, "exact squares")?;', 'line.wait_for(stage, "as you climb")?;', "every_hook", "out of the voice's order"),
    ("two_planes", "pub const RADIUS_AT: [C; 2] = [C::new(0.8, -0.66), C::new(0.7, -0.5)];", "pub const RADIUS_AT: [C; 2] = [C::new(0.8, -0.3), C::new(0.7, -0.5)];", "the_walk_radii", "reaches"),
    ("two_planes", "pub const RULE_ANGLE: f64 = PI / 6.0;", "pub const RULE_ANGLE: f64 = 2.0;", "the_rule_is_shown", "angle not doubled"),
    ("two_planes", "pub const RULE_LENGTH: f64 = 1.4;", "pub const RULE_LENGTH: f64 = 0.8;", "the_rule_is_shown", "walk's circle"),
    ("two_planes", "fade_out_all(stage, &shown_rule, 0.3)?;", "let _ = 0;", "the_planes_are_not_empty", "shown_rule"),
    ("two_planes", 'line.wait_for(stage, "doubles each angle")?;', 'line.wait_for(stage, "Walk a point")?;', "the_planes_are_not_empty", "doubles each angle"),
    ("two_planes", "stage.add_to_scene(second_lap)?;", "let _ = 0;", "each_half_of_the_walk", "second_lap"),
    ("two_planes", "gone = vec![rule, ring_in, ring_out, w_in, w_out, trail, second_lap]", "gone = vec![rule, ring_in, ring_out, w_in, w_out, trail]", "each_half_of_the_walk", "never fades"),
    ("two_planes", "pub const FLIP_BUFF: f64 = 0.1;", "pub const FLIP_BUFF: f64 = 0.4;", "each_flip", "long"),
    ("two_planes", "gone.extend(flips);", "", "each_flip", "never fade"),
    ("two_planes", "pub const ASK_AT: C = C::new(0.55, -0.65);", "pub const ASK_AT: C = C::new(0.0, 0.4);", "the_question_and_its_ring", "imaginary axis"),
    ("two_planes", "pub const ASK_AT: C = C::new(0.55, -0.65);", "pub const ASK_AT: C = C::new(0.55, 1.0);", "the_question_and_its_ring", "covers the 1"),
    ("two_planes", '"horizontal axis → positive ray"', '"real axis → positive ray"', "the_labels_say", "voice's words"),
    ("two_planes", "fade_out_all(stage, &[rescaled], 0.3)?;", "let _ = 0;", "the_labels_say", "rescaled"),
    ("reading_color", "f.eval(loop_point(t)).map_or(0.0, |w| w.arg())", "loop_point(t).arg()", "the_needle_stands", "f's output"),
    ("reading_color", "pub const COUNT_BELOW: f64 = 0.35;", "pub const COUNT_BELOW: f64 = 0.1;", "the_walk_counts", "touches the wheel"),
    ("reading_color", "pub const STATED_Y: f64 = 0.6;", "pub const STATED_Y: f64 = 1.0;", "the_walk_counts", "stated row"),
    ("reading_color", "let walked = walk.stop(stage);\n        fade_out_all(stage, &walked, 0.3)?;", "let walked = walk.stop(stage);", "the_cycle_is_walked", "fade_out_all(stage, &walked"),
    ("reading_color", 'LoopWalk::start(&kit, stage, Func::Recip, p, wheel_at)?', 'LoopWalk::start(&kit, stage, Func::Square, p, wheel_at)?', "the_cycle_is_walked", "Func::Recip"),
    ("reading_color", "clearing.extend(&walked);", "", "the_cycle_is_walked", "clearing.extend"),
    ("reading_color", "stage.add_to_scene(touching)?;", "let _ = 0;", "the_gold_never", "touching"),
    ("reading_color", "near a zero z = a of order m", "near a zero a of order m", "the_exact_card", "zero a"),
    ("reading_color", "near a zero z = a of order m", "near a zero z = a that has order m", "the_exact_card", "characters"),
    ("reading_color", "near a zero z = a of order m", "near a zero z = a that has order m", "mod::reading_windows", "04_4"),
    ("gallery", "zero_mark(stage, sp.p(ONE))", "pole_mark(stage, sp.p(ONE))", "the_zero_and_the_pole", "zero_mark"),
    ("gallery", "fade_in_all(stage, &arriving, 0.3)?;", "let _ = 0;", "the_zero_and_the_pole", "arriving"),
    ("gallery", "pub const EXP_TILE_BAND: (i32, i32) = (1, 3);", "pub const EXP_TILE_BAND: (i32, i32) = (20, 3);", "the_exponential_tile", "window"),
    ("gallery", "C::new(x + s, y + s)", "C::new(x + s, y + 2.0 * s)", "the_exponential_tile", "lost"),
    ("gallery", "                one_square,\n", "", "the_exponential_tile", "never fades"),
    ("gallery", "pub const CLIMB_STOP: f64 = 2.8;", "pub const CLIMB_STOP: f64 = 2.9;", "the_climb_stops", "on the dot"),
    ("gallery", "stage.add_to_scene(start_box)?;", "let _ = 0;", "the_strip_arrives", "start_box"),
    ("gallery", "fade_out_all(stage, &[converges, pole_says, zedge], 0.3)?;", "let _ = 0;", "the_strip_arrives", "converges"),
    ("gallery", "fade_out_all(stage, &[squashed], 0.3)?;", "let _ = 0;", "the_strip_arrives", "squashed"),
    ("gallery", "pub const TILE_BEATS: [f64; 4] = [0.3, 0.3, 0.5, 0.2];", "pub const TILE_BEATS: [f64; 4] = [0.4, 0.4, 0.8, 0.3];", "the_tile_is_ready", "before the tile"),
    ("gallery", 'line.wait_for(stage, "nearly square")?;', "", "the_tile_is_ready", "nearly square"),
    ("gallery", "this strip shows four zeros", "this plot shows four zeros", "the_tile_is_ready", "which picture"),
    ("gallery", "located by Newton's method", "located to high accuracy by Newton's method in the scene itself", "mod::reading_windows", "05_8"),
    ("gallery", "fade_out_all(stage, &[squashed], 0.3)?;\n        let four = precisely_now(&kit, stage, FOUR)?;", "let four = precisely_now(&kit, stage, FOUR)?;\n        fade_out_all(stage, &[squashed], 0.3)?;", "mod::reading_windows", "cleared early"),
]


def attach_fn_reader():
    def const_fn(self, name, fn):
        text = self.src[name]
        m = re.search(rf"pub fn {fn}\b[^{{]*\{{", text)
        assert m, f"{name}: no fn {fn}"
        depth, i = 1, m.end()
        while depth:
            c = text[i]
            depth += (c == "{") - (c == "}")
            i += 1
        return text[m.start() : i]

    World.const_fn = const_fn


def run_all(world):
    fails = {}
    for label, test in TESTS.items():
        try:
            test(world)
        except AssertionError as e:
            fails[label] = str(e)
        except Exception as e:  # a parse failure is reported as such, never as a pass
            fails[label] = f"ERROR {type(e).__name__}: {e}"
    return fails


def load_model(text):
    table = [
        [int(x, 16) for x in row]
        for row in re.findall(
            r"\[0x([0-9A-F]{2}), 0x([0-9A-F]{2}), 0x([0-9A-F]{2})\]", text[text.index("pub const WHEEL") :]
        )
    ][:12]
    assert len(table) == 12
    zero = re.search(r"Func::ZeroDoublePole => vec!\[(ONE)\]", text)
    pole = re.search(r"Func::ZeroDoublePole => vec!\[(\(-ONE, 2\))\]", text)
    return {"wheel": table, "zdp": ([zero.group(1)], [pole.group(1)])}


def main():
    repo, staged = Path(sys.argv[1]), Path(sys.argv[2])
    crate = repo / "explainer_corpus/complex_analysis_visual"
    script = {}
    for row in (crate / "script.tsv").read_text(encoding="utf8").splitlines():
        parts = row.split("\t")
        if len(parts) >= 2 and re.fullmatch(r"\d\d_\d+", parts[0]):
            script[parts[0]] = parts[1]
    model = load_model((crate / "src/model.rs").read_text(encoding="utf8"))
    draw = (crate / "src/chapters/draw.rs").read_text(encoding="utf8")
    names = ("turning", "two_planes", "reading_color", "gallery")
    attach_fn_reader()
    good = {n: (staged / f"{n}.rs").read_text(encoding="utf8") for n in names}
    canonical = {n: (crate / f"src/chapters/{n}.rs").read_text(encoding="utf8") for n in names}

    bad = 0
    print(f"mirrors: {len(TESTS)}; planted defects: {len(PLANTED)}")
    base = run_all(World(good, script, model, draw))
    print(f"baseline on staged sources: {len(TESTS) - len(base)} pass, {len(base)} fail")
    for label, why in base.items():
        print(f"  FAIL {label}: {why}")
        bad += 1

    print("planted defects:")
    for name, old, new, which, reason in PLANTED:
        src = dict(good)
        if src[name].count(old) < 1:
            print(f"  NOT PLANTED {name}: {old[:60]!r}")
            bad += 1
            continue
        src[name] = src[name].replace(old, new, 1)
        fails = run_all(World(src, script, model, draw))
        hit = [k for k in fails if which in k]
        ok = hit and any(reason in fails[k] for k in hit)
        other = [k for k in fails if which not in k]
        tag = "ok " if ok else "BAD"
        note = f" (also: {', '.join(k.split('::')[1][:28] for k in other)})" if other else ""
        print(f"  {tag} {name}: {old.strip()[:54]!r} -> {which}: {fails.get(hit[0], 'passed') if hit else 'passed'}{note}")
        if not ok:
            bad += 1

    print("canonical control (a mirror that passes here does not detect the original defect):")
    control = run_all(World(canonical, script, model, draw))
    # both guard what must hold before and after alike: the hooks already agree on canonical, and
    # the card windows are existing mod.rs tests repeated on the new card text
    expected_pass = {
        "turning::every_hook_is_said_and_they_run_in_the_voices_order",
        "mod::reading_windows_and_each_card_outlasts (existing tests, repeated)",
    }
    for label in TESTS:
        state = "fails" if label in control else "passes"
        why = control.get(label, "")
        print(f"  {state:6} {label}{': ' + why[:90] if why else ''}")
        if label not in control and label not in expected_pass:
            bad += 1
            print("     ^ unexpected: nothing distinguishes the repair here")
    print(json.dumps({"problems": bad}))
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
