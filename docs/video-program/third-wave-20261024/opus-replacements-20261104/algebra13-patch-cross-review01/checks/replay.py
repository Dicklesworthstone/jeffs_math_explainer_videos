"""Replay a candidate's 13_1 to 13_8 and 13_12 statements from its source text, on Oliver's sentence starts.

Independent of BronzeJaguar's desk_mirror.py: it imports nothing from that bundle and keeps its own tile model.
It reads the Rust as text. It cannot see types or borrows, and it does not render. Times are sums of the literal
run times, so they are predictions. Oliver's starts are root's ASR estimates, not hearing.

For each candidate it reports:
  POSE/NAME  each interval where a tile's name is not that tile's pose, or its tile is moving under it
  LIFECYCLE  a fade of something not on stage, a glide of something not on stage, a bad wait index, a missing phrase
  WAITS      slack at every sentence or phrase wait; a negative slack means the picture overran
  HOLDS      how long each caption is fully shown, against 15 characters per second

Usage (repo root): python3 -I -B <this file> <arithmetic.rs> [label]
"""
import json
import math
import pathlib
import re
import sys

LAB = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103")
CRATE = pathlib.Path("explainer_corpus/abstract_algebra_expanded")
LENGTHS = {
    "13_1": 16.00, "13_2": 24.64, "13_3": 20.08, "13_4": 19.92, "13_5": 19.60, "13_6": 22.56,
    "13_7": 24.08, "13_8": 21.68, "13_9": 25.28, "13_10": 22.72, "13_11": 27.84, "13_12": 26.24,
}
BREATH = 0.6
READ_RATE = 15.0


def mat(a, b):
    return [[sum(a[i][k] * b[k][j] for k in range(3)) for j in range(3)] for i in range(3)]


def turn(t):
    c, s = math.cos(math.pi / 2 * t), math.sin(math.pi / 2 * t)
    return [[c, -s, 0], [s, c, 0], [0, 0, 1]]


def flip(t):
    c, s = math.cos(math.pi * t), math.sin(math.pi * t)
    return [[1, 0, 0], [0, c, -s], [0, s, c]]


IDENT = turn(0)
R, S = turn(1), flip(1)


def word(*ms):
    out = IDENT
    for m in ms:
        out = mat(out, m)
    return out


NAMES = {
    "start": IDENT, "e": IDENT, "r": R, "s": S, "r s": word(R, S), "s r": word(S, R), "s r s": word(S, R, S),
    "r²": word(R, R), "r³": word(R, R, R), "r⁴ = e": word(R, R, R, R), "s² = e": word(S, S),
    "r⁻¹": turn(-1), "r³ r = e": word(R, R, R, R),
}


def same(a, b):
    return all(abs(a[i][j] - b[i][j]) < 1e-9 for i in range(3) for j in range(3))


def strip_comments(text):
    return "\n".join(re.sub(r"(^|\s)//.*$", "", line) for line in text.splitlines())


def split_statements(text):
    out, cur, depth, in_str, i = [], [], 0, False, 0
    while i < len(text):
        c = text[i]
        cur.append(c)
        if in_str:
            if c == "\\" and i + 1 < len(text) and text[i + 1] == '"':
                cur.append(text[i + 1])
                i += 1
            elif c == '"':
                in_str = False
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
            if depth == 0 and c == "}" and "".join(cur).lstrip().startswith("for "):
                out.append("".join(cur).strip())
                cur = []
        elif c == ";" and depth == 0:
            out.append("".join(cur).strip())
            cur = []
        i += 1
    return [s for s in out if s and s != ";"]


def split_args(text):
    out, cur, depth, in_str = [], [], 0, False
    for c in text:
        if in_str:
            in_str = c != '"'
        elif c == '"':
            in_str = True
        elif c in "([{":
            depth += 1
        elif c in ")]}":
            depth -= 1
        elif c == "," and depth == 0:
            out.append("".join(cur).strip())
            cur = []
            continue
        cur.append(c)
    last = "".join(cur).strip()
    if last:
        out.append(last)
    return out


class Replay:
    def __init__(self, source, starts, script):
        self.src = source
        self.starts = starts
        self.script = script
        self.t = 0.0
        self.said = {}
        self.busy = 0.0
        self.cue = None
        self.env = {}
        self.trackers = {}
        self.tiles = []
        self.stage = {}
        self.next_id = 0
        self.log = []
        self.waits = []
        self.problems = []
        self.mismatch = []
        self.holds = []
        self.consts = {
            "LONE_RT": self.const_f("LONE_RT"),
            "PACKAGE_RT": self.const_f("PACKAGE_RT"),
        }
        for name in ("BELOW_TURNING",):
            if f"const {name}" in source:
                self.consts[name] = self.const_f(name)
        self.moves = {}
        for name in ("LEFT", "RIGHT", "LONG_ROUTE", "SEQUENCE"):
            m = re.search(rf"const {name}: \[Move; \d+\] = \[(.*?)\];", source, re.S)
            if m:
                self.moves[name] = re.findall(r"Move::(Turn\(-?[\d.]+\)|Flip)", m.group(1))

    def const_f(self, name):
        return float(re.search(rf"const {name}: f64 = (-?[\d.]+);", self.src).group(1))

    def rel(self, t=None):
        t = self.t if t is None else t
        return t - self.said.get(self.cue, 0.0)

    def say_log(self, text):
        self.log.append(f"{self.cue or '-':>6} {self.rel():7.2f}  {text}")

    def pose_fn(self, text):
        m = re.search(r"pose\(&(\w+), t\)", text)
        if m:
            moves = self.moves[m.group(1)]

            def f(t):
                state = IDENT
                for i, mv in enumerate(moves):
                    frac = min(max(t - i, 0.0), 1.0)
                    if mv == "Flip":
                        nxt = flip(frac)
                    else:
                        nxt = turn(float(mv[5:-1].rstrip(".") or 0) * frac)
                    state = mat(nxt, state)
                return state

            return f
        if "turn(-t)" in text:
            return lambda t: turn(-t)
        if "if t <= 1." in text and "flip(t - 1.)" in text:
            return lambda t: flip(t) if t <= 1 else mat(flip(t - 1), flip(1))
        if "if t <= 1." in text and "turn(t - 1.)" in text:
            return lambda t: flip(t) if t <= 1 else mat(turn(t - 1), flip(1))
        if text.strip() == "tile_model::turn":
            return turn
        raise ValueError("unknown pose function: " + text)

    def num(self, text):
        text = text.strip().rstrip(".") if re.fullmatch(r"-?[\d.]+", text.strip()) else text.strip()
        if text in self.consts:
            return self.consts[text]
        if text in self.env and isinstance(self.env[text], (int, float)):
            return float(self.env[text])
        m = re.fullmatch(r"(\w+)\.len\(\) as f64", text)
        if m:
            return float(len(self.moves[m.group(1)]))
        m = re.fullmatch(r"f64::from\((\w+)\)", text)
        if m:
            return float(self.env[m.group(1)])
        m = re.fullmatch(r"\((\w+) \+ 1\) as f64", text)
        if m:
            return float(self.env[m.group(1)] + 1)
        return float(text)

    def new_mob(self, name, text="", x=None, y=None, kind="thing", on=False):
        self.next_id += 1
        mob = {"id": self.next_id, "name": name, "text": text, "x": x, "y": y, "kind": kind}
        if on:
            self.stage[mob["id"]] = mob
            mob["whole_from"] = self.t
        return mob

    def list_of(self, expr):
        expr = expr.strip().lstrip("&").strip()
        if expr.startswith("[") and expr.endswith("]"):
            out = []
            for item in split_args(expr[1:-1]):
                out.extend(self.item(item))
            return out
        m = re.fullmatch(r"(\w+)\[\.\.(\d+)\]", expr)
        if m:
            return self.env[m.group(1)][: int(m.group(2))]
        return self.item(expr)

    def item(self, expr):
        expr = expr.strip()
        m = re.fullmatch(r"(\w+)\[(\d+)\]", expr)
        if m:
            return [self.env[m.group(1)][int(m.group(2))]]
        m = re.fullmatch(r"(\w+)\.slots", expr)
        if m:
            return self.env[m.group(1)]["slots"]
        got = self.env[expr]
        return got if isinstance(got, list) else [got]

    def elapse(self, dt, moving=None):
        """Advance the clock. `moving` is (tile, from, to) while a tile's tracker runs."""
        t0, t1 = self.t, self.t + dt
        for mob in list(self.stage.values()):
            if mob["kind"] != "name":
                continue
            tile = next((p for p in self.tiles if p["on"] and abs(p["x"] - mob["x"]) < 1e-6), None)
            if tile is None:
                continue
            value = self.trackers[tile["tracker"]]
            want = NAMES[mob["text"]]
            if moving and moving[0] is tile:
                bad = not (same(tile["fn"](moving[1]), want) and same(tile["fn"](moving[2]), want))
            else:
                bad = not same(tile["fn"](value), want)
            if bad and dt > 0:
                self.note_mismatch(mob, t0, t1)
        self.t = t1

    def note_mismatch(self, mob, t0, t1):
        cue = self.cue
        base = self.said.get(cue, 0.0)
        if self.mismatch and self.mismatch[-1]["id"] == mob["id"] and abs(self.mismatch[-1]["end"] - (t0 - base)) < 1e-6:
            self.mismatch[-1]["end"] = t1 - base
        else:
            self.mismatch.append({"id": mob["id"], "cue": cue, "text": mob["text"], "x": mob["x"], "start": t0 - base, "end": t1 - base})

    def take_off(self, mobs, run, how):
        names = []
        for mob in mobs:
            if mob["id"] not in self.stage:
                self.problems.append(f"{self.cue} {self.rel():.2f}: {how} of {mob['name']} ({mob['text']}), not on stage")
            names.append(mob["text"] or mob["name"])
        self.say_log(f"{how} {run:.2f}: " + ", ".join(names))
        for mob in mobs:
            if mob["kind"] in ("name", "caption") and "whole_from" in mob:
                self.record_hold(mob, self.t)
        self.elapse(run)
        for mob in mobs:
            self.stage.pop(mob["id"], None)
        for tile in self.tiles:
            if tile["on"] and tile["mob"]["id"] not in self.stage:
                tile["on"] = False

    def record_hold(self, mob, end):
        chars = len(mob["text"].replace(" ", ""))
        self.holds.append({
            "cue": mob["cue"], "text": mob["text"], "from": mob["whole_from"] - self.said[mob["cue"]],
            "hold": end - mob["whole_from"], "need": chars / READ_RATE, "open": False,
        })

    def put_on(self, mobs, run, how="show"):
        names = [m["text"] or m["name"] for m in mobs]
        self.say_log(f"{how} {run:.2f}: " + ", ".join(names))
        for mob in mobs:
            if mob["id"] in self.stage:
                self.problems.append(f"{self.cue} {self.rel():.2f}: {how} of {mob['name']}, already on stage")
            self.stage[mob["id"]] = mob
            mob["cue"] = self.cue
        self.elapse(run)
        for mob in mobs:
            mob["whole_from"] = self.t

    def caption(self, text, x, y, name=""):
        kind = "name" if (text in NAMES and y <= -1.9 and abs(y + 2.0) < 1e-6) else "caption"
        mob = self.new_mob(name or text, text, x, y, kind)
        self.put_on([mob], 0.4, "caption")
        return mob

    def pos(self, x_expr, y_expr):
        def one(e):
            m = re.fullmatch(r"(\w+)\.([01])", e.strip())
            if m and m.group(1) in ("LEFT_LABEL", "RIGHT_LABEL"):
                raw = re.search(rf"const {m.group(1)}: \(f64, f64\) = \((-?[\d.]+), (-?[\d.]+)\);", self.src)
                return float(raw.group(int(m.group(2)) + 1).rstrip("."))
            return self.num(e)

        return one(x_expr), one(y_expr)

    def at_pair(self, expr):
        raw = re.search(rf"const {expr}: \(f64, f64\) = \((-?[\d.]+), (-?[\d.]+)\);", self.src)
        return float(raw.group(1).rstrip(".")), float(raw.group(2).rstrip("."))

    def text_arg(self, expr):
        expr = expr.strip()
        if expr.startswith('"'):
            return expr[1:-1]
        return self.env[expr]

    def tile_for(self, tracker):
        return next((p for p in self.tiles if p["tracker"] == tracker and p["on"]), None)

    def motion(self, tracker, to, run, tail):
        tile = self.tile_for(tracker)
        if tile is None:
            self.problems.append(f"{self.cue} {self.rel():.2f}: motion of {tracker} with no tile on stage")
            self.trackers[tracker] = to
            self.elapse(run + tail)
            return
        frm = self.trackers[tracker]
        self.say_log(f"motion {tracker} {frm:g} to {to:g} in {run:.2f} (+{tail:.2f})")
        self.elapse(run, (tile, frm, to))
        self.trackers[tracker] = to
        self.elapse(tail)

    def wait_until(self, cue, target, what):
        slack = target - self.t
        self.waits.append({"cue": cue, "what": what, "at": target - self.said[cue], "slack": slack})
        self.say_log(f"wait {what}: slack {slack:+.2f}")
        if slack > 0:
            self.elapse(slack)

    def phrase_at(self, cue, j, phrase):
        text = self.script[cue]
        parts = sentence_ranges(text)
        if j >= len(parts):
            self.problems.append(f"{cue}: sentence {j} does not exist")
            return None
        a, b = parts[j]
        at = text[a:b].find(phrase)
        if at < 0:
            self.problems.append(f"{cue} sentence {j}: phrase {phrase!r} missing")
            return None
        s = self.starts[cue]
        start, end = s[j], (s[j + 1] if j + 1 < len(s) else LENGTHS[cue])
        share = len(text[a:a + at]) / (b - a)
        return self.said[cue] + start + (end - start) * share

    def run(self, statements):
        for st in statements:
            self.one(st)

    def one(self, st):
        st = re.sub(r"\s+", " ", st).strip()
        if st.endswith(";"):
            st = st[:-1]
        if st.startswith("for "):
            return self.loop(st)
        m = re.fullmatch(r'kit\.say\(stage, "([\d_]+)"\)\?', st)
        if m:
            cue = m.group(1)
            at = max(self.t, self.busy + BREATH) if self.cue else self.t
            late = self.t - (self.busy + BREATH) if self.cue else 0.0
            if at > self.t:
                self.elapse(at - self.t)
            self.cue = cue
            self.said[cue] = self.t
            self.busy = self.t + LENGTHS[cue]
            self.say_log(f"SAY {cue}, gap slack {-late:+.2f}, length {LENGTHS[cue]:.2f}")
            return
        m = re.fullmatch(r'kit\.until_sentence\(stage, "([\d_]+)", (\d+)\)\?', st)
        if m:
            cue, j = m.group(1), int(m.group(2))
            if j >= len(self.starts[cue]):
                self.problems.append(f"{cue}: until_sentence {j} out of range: silently no wait")
                return
            return self.wait_until(cue, self.said[cue] + self.starts[cue][j], f"{cue} sentence {j}")
        m = re.fullmatch(r'until_phrase\(stage, &kit, "([\d_]+)", (\d+), (\w+)\)\?', st)
        if m:
            cue, j, name = m.group(1), int(m.group(2)), m.group(3)
            phrase = re.search(rf'const {name}: &str = "(.*?)";', self.src).group(1)
            target = self.phrase_at(cue, j, phrase)
            if target is None:
                return
            return self.wait_until(cue, target, f"{cue} phrase {phrase!r} (share)")
        if st == "kit.hold(stage)?":
            target = self.busy + BREATH
            self.say_log("hold")
            if target > self.t:
                self.elapse(target - self.t)
            return
        m = re.fullmatch(r"stage\.wait\(([\d.]+)\)\?", st)
        if m:
            self.say_log(f"wait {m.group(1)}")
            return self.elapse(float(m.group(1)))
        m = re.fullmatch(r"let (\w+) = stage\.add_value_tracker\(0\.\)", st)
        if m:
            self.trackers[m.group(1)] = 0.0
            return
        m = re.fullmatch(r"let (mut )?(\w+) = tile_actor::add_tile\((.*)\)\?", st)
        if m:
            self.env[m.group(2)] = [self.add_tile(m.group(2), m.group(3))]
            return
        m = re.fullmatch(r"(\w+)\.extend\(tile_actor::add_tile\((.*)\)\?,?\s*\)", st)
        if m:
            self.env[m.group(1)].append(self.add_tile(m.group(1) + "+", m.group(2)))
            return
        m = re.fullmatch(r"(\w+)\.extend\((.*)\)", st)
        if m:
            self.env[m.group(1)] = self.env[m.group(1)] + self.list_of(m.group(2))
            return
        m = re.fullmatch(r"(\w+)\.push\((\w+)\)", st)
        if m:
            self.env[m.group(1)] = self.env[m.group(1)] + self.item(m.group(2))
            return
        m = re.fullmatch(r"let mut (\w+) = (\w+)", st)
        if m and m.group(2) in self.env:
            self.env[m.group(1)] = list(self.item(m.group(2)))
            return
        m = re.fullmatch(r"(?:let mut )?(\w+) = vec!\[(.*)\]", st)
        if m:
            self.env[m.group(1)] = self.list_of("[" + m.group(2) + "]")
            return
        m = re.fullmatch(r"let mut (\w+) = Vec::new\(\)", st)
        if m:
            self.env[m.group(1)] = []
            return
        m = re.fullmatch(r"fade\(stage, (.*), false\)\?", st)
        if m:
            return self.take_off(self.list_of(m.group(1)), 0.4, "fade")
        m = re.fullmatch(r"d4g::hide\(stage, (.*), ([\d.]+)\)\?", st)
        if m:
            return self.take_off(self.list_of(m.group(1)), float(m.group(2)), "hide")
        m = re.fullmatch(r"d4g::show\(stage, (.*), ([\d.]+)\)\?", st)
        if m:
            return self.put_on(self.list_of(m.group(1)), float(m.group(2)))
        m = re.fullmatch(r"(let (?:mut )?)?(\w+) = caption\(stage, &kit, (.*)\)\?", st)
        if m:
            a = split_args(m.group(3))
            x, y = self.pos(a[1], a[2])
            self.env[m.group(2)] = self.caption(self.text_arg(a[0]), x, y)
            return
        m = re.fullmatch(r"(\w+)\.push\(caption\(stage, &kit, (.*)\)\?\)", st)
        if m:
            a = split_args(m.group(2))
            x, y = self.pos(a[1], a[2])
            self.env[m.group(1)] = self.env[m.group(1)] + [self.caption(self.text_arg(a[0]), x, y)]
            return
        m = re.fullmatch(r"(let (?:mut )?)?(\w+) = replace\(stage, &kit, (\w+), (.*)\)\?", st)
        if m:
            a = split_args(m.group(4))
            self.take_off(self.item(m.group(3)), 0.4, "fade")
            x, y = self.pos(a[1], a[2])
            self.env[m.group(2)] = self.caption(self.text_arg(a[0]), x, y)
            return
        m = re.fullmatch(r"(let (?:mut )?)?(\w+) = step\(stage, &kit, (\w+), (.*?), (\w+), (.*?), (\w+)\)\?", st)
        if m:
            self.take_off(self.item(m.group(5)), 0.4, "fade")
            self.motion(m.group(3), self.num(m.group(4)), 1.7, 0.5)
            x, y = self.at_pair(m.group(7))
            self.env[m.group(2)] = self.caption(self.text_arg(m.group(6)), x, y)
            return
        m = re.fullmatch(r"advance\(stage, (\w+), (.*)\)\?", st)
        if m:
            return self.motion(m.group(1), self.num(m.group(2)), 1.7, 0.5)
        m = re.fullmatch(r"perform\(stage, (\w+), (.*), (.*), &\[.*\]\)\?", st)
        if m:
            return self.motion(m.group(1), self.num(m.group(2)), self.num(m.group(3)), 0.25)
        m = re.fullmatch(r"let (\w+) = grouping\(stage, (.*)\)\?", st)
        if m:
            mob = self.new_mob(m.group(1))
            self.env[m.group(1)] = mob
            return self.put_on([mob], 0.4, "grouping")
        m = re.fullmatch(r"d4g::glide\(stage, (\w+), \(.*\), ([\d.]+)\)\?", st)
        if m:
            for mob in self.item(m.group(1)):
                if mob["id"] not in self.stage:
                    self.problems.append(f"{self.cue} {self.rel():.2f}: glide of {mob['name']}, not on stage")
            self.say_log(f"glide {m.group(1)} {m.group(2)}")
            return self.elapse(float(m.group(2)))
        m = re.fullmatch(r"let (\w+) = bare_rack\(stage, &kit\)\?", st)
        if m:
            self.env[m.group(1)] = [self.new_mob(f"{m.group(1)}{i}") for i in range(8)]
            return
        m = re.fullmatch(r"let (\w+) = rack\(.*\)\?", st)
        if m:
            self.env[m.group(1)] = {"slots": [self.new_mob(f"{m.group(1)}{i}") for i in range(8)]}
            return
        m = re.fullmatch(r"appear\(stage, (.*), ([\d.]+), ([\d.]+)\)\?", st)
        if m:
            mobs = self.list_of(m.group(1))
            each, lag = float(m.group(2)), float(m.group(3))
            return self.put_on(mobs, each + (len(mobs) - 1) * lag * each, "appear")
        m = re.fullmatch(r"let (\w+) = \[row_frame\(stage, 0\)\?, row_frame\(stage, 1\)\?\]", st)
        if m:
            self.env[m.group(1)] = [self.new_mob(m.group(1) + "0"), self.new_mob(m.group(1) + "1")]
            return
        m = re.fullmatch(r"let (\w+) = relight\(stage, &cards, (.*), (\d), (.*)\)\?", st)
        if m:
            leaving = [] if m.group(4).strip() == "&[]" else self.list_of(m.group(4))
            lit = self.new_mob("lit" + m.group(3))
            self.say_log(f"relight card {m.group(3)}, 0.40: leaving " + ", ".join(x["text"] or x["name"] for x in leaving))
            for mob in leaving:
                if mob["id"] not in self.stage:
                    self.problems.append(f"{self.cue} {self.rel():.2f}: relight fades {mob['name']}, not on stage")
                if mob["kind"] == "caption" and "whole_from" in mob:
                    self.record_hold(mob, self.t)
            self.elapse(0.4)
            for mob in leaving:
                self.stage.pop(mob["id"], None)
            self.stage[lit["id"]] = lit
            self.env[m.group(1)] = lit
            return
        m = re.fullmatch(r"let (\w+) = \[ side_rack\.frame\(.*\)\?,? \]", st)
        if m:
            self.env[m.group(1)] = [self.new_mob(m.group(1) + "0"), self.new_mob(m.group(1) + "1")]
            return
        m = re.fullmatch(r"let (\w+) = side_rack\.frame\(.*\)\?", st)
        if m:
            self.env[m.group(1)] = self.new_mob(m.group(1))
            return
        m = re.fullmatch(r"let (\w+) = eq_tex\(.*\)\?", st)
        if m:
            self.env[m.group(1)] = self.new_mob(m.group(1), self.spoken_eq(m.group(1)), None, None, "caption")
            return
        m = re.fullmatch(r'let (\w+) = purpose\(stage, &kit, "(.*)"\)\?', st)
        if m:
            self.env[m.group(1)] = self.new_mob(m.group(1), m.group(2), None, None, "caption")
            return
        m = re.fullmatch(r"let (\w+) = mark\(.*\)\?", st)
        if m:
            self.env[m.group(1)] = self.new_mob(m.group(1))
            return
        if re.fullmatch(r"let lines = \[.*\]", st):
            self.line_set = self.eq_lines.pop(0)
            return
        if re.match(r"let \(?\w+", st):
            return
        if re.fullmatch(r"shown\.extend\(side_rack\.slots\)", st):
            return
        self.problems.append("UNHANDLED: " + st[:120])

    eq_lines = [["s r s r²s", "=(s r s) r²s = r⁻¹ r²s", "=(r⁻¹ r²) s = rs"], ["r x = s", "x = r⁻¹ s", "x = r³s"]]
    eq_named = {"closed": "s r = r³s", "unchanged": "e r = r = r e", "check": "r (r³s) = s"}

    def spoken_eq(self, name):
        if name == "step":
            return self.line_set[self.loop_k]
        return self.eq_named[name]

    def loop(self, st):
        m = re.fullmatch(r"for (.*?) in (.*?) \{ (.*) \}", st)
        head, source, body = m.group(1), m.group(2), split_statements(m.group(3))
        if source == "lines.iter().enumerate()":
            for k in range(3):
                self.loop_k = k
                self.run(body)
            return
        m2 = re.fullmatch(r"\[(.*)\]\.into_iter\(\)\.enumerate\(\)", source)
        if m2:
            for k, item in enumerate(split_args(m2.group(1))):
                self.env["index"], self.env["text"] = k, item[1:-1]
                self.run(body)
            return
        m2 = re.fullmatch(r"(\d+)\.\.=(\d+)", source)
        if m2:
            for n in range(int(m2.group(1)), int(m2.group(2)) + 1):
                self.env[head] = n
                self.run(body)
            return
        m2 = re.fullmatch(r"\[(.*)\]", source)
        names = split_args(head.strip("()"))
        for item in split_args(m2.group(1)):
            parts = split_args(item.strip()[1:-1])
            for n, p in zip(names, parts):
                self.env[n] = p[1:-1] if p.startswith('"') else float(p.rstrip("."))
            self.run(body)

    def add_tile(self, name, args):
        a = split_args(args)
        tracker, x, fn = a[2], self.num(a[3]), a[6]
        mob = self.new_mob(name, "", x, None, "tile", on=True)
        self.tiles.append({"tracker": tracker, "x": x, "fn": self.pose_fn(fn), "mob": mob, "on": True})
        self.say_log(f"tile {name} at x {x:g} appears at once")
        return mob

    def close_open_holds(self):
        for mob in self.stage.values():
            if mob["kind"] in ("name", "caption") and "whole_from" in mob:
                self.holds.append({
                    "cue": mob["cue"], "text": mob["text"], "from": mob["whole_from"] - self.said[mob["cue"]],
                    "hold": self.t - mob["whole_from"], "need": len(mob["text"].replace(" ", "")) / READ_RATE, "open": True,
                })


def sentence_ranges(text):
    out, start, n = [], 0, len(text)
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == n or text[i + 1] == " "):
            a = start
            while a <= i and text[a] == " ":
                a += 1
            out.append((a, i + 1))
            start = i + 1
    return out


def load_script():
    out = {}
    for row in (CRATE / "script.tsv").read_text(encoding="utf-8").splitlines():
        cols = row.split("\t")
        if len(cols) >= 2 and cols[0].startswith("13_"):
            out[cols[0]] = cols[1]
    return out


def load_starts(script):
    a = json.loads((LAB / "arithmetic-sentence-audit02.json").read_text())
    large = {r["cue"]: [g["start"] for g in r["punctuation_groups"]] for r in a["records"] if r.get("model") == "large"}
    small = {r["cue"]: [g["start"] for g in r["punctuation_groups"]] for r in a["records"] if r.get("model") == "small"}
    # The large model merges 13_6's first two sentences and splits its last at the colon.
    large["13_6"] = [large["13_6"][0], small["13_6"][1]] + large["13_6"][1:4]
    for cue, s in large.items():
        n = len(sentence_ranges(script[cue]))
        if cue in ("13_1", "13_2", "13_3", "13_4", "13_5", "13_6", "13_7", "13_8", "13_12") and len(s) != n:
            raise SystemExit(f"{cue}: {len(s)} starts for {n} sentences")
    return large


def main():
    path = pathlib.Path(sys.argv[1])
    label = sys.argv[2] if len(sys.argv) > 2 else path.parent.parent.name
    src = path.read_text(encoding="utf-8")
    script = load_script()
    starts = load_starts(script)
    body = strip_comments(src)
    begin = body.index('kit.say(stage, "13_1")')
    end = body.index('kit.say(stage, "13_9")')
    a = Replay(src, starts, script)
    a.run(split_statements(body[begin:end]))
    a.one('kit.say(stage, "13_9")?')

    b = Replay(src, starts, script)
    b.cue = "13_12"
    b.said["13_12"] = 0.0
    b.busy = LENGTHS["13_12"]
    begin = body.index('kit.until_sentence(stage, "13_12", 1)')
    end = body.index("let names: Vec<Mob> = cards.iter()")
    b.env["side_rack"] = {"slots": []}
    if "let lit = relight(" not in body:
        print(f"{label}: 13_12 not replayed (sequential card changes; see the author's baseline table)")
        runs = [a]
    else:
        b.run(split_statements(body[begin:end]))
        runs = [a, b]
    for r in runs:
        r.close_open_holds()

    print(f"# {label}: replay on Oliver take01 ASR starts, sums of literal run times")
    print("\n## Timeline, cue-relative seconds")
    for r in runs:
        print("\n".join(r.log))
    print("\n## POSE/NAME: intervals when a tile's name is not its pose, or its tile moves under it")
    rows = [m for r in runs for m in r.mismatch]
    for m in rows:
        print(f"  {m['cue']:>5} x {m['x']:+.1f}  {m['start']:6.2f} to {m['end']:6.2f}  ({m['end'] - m['start']:.2f} s)  {m['text']}")
    print(f"  {len(rows)} intervals")
    print("\n## LIFECYCLE")
    problems = [p for r in runs for p in r.problems]
    for p in problems:
        print("  " + p)
    print(f"  {len(problems)} problems")
    print("\n## WAITS: negative slack means the picture is late for that word")
    for r in runs:
        for w in r.waits:
            mark = "  LATE" if w["slack"] < -0.005 else ""
            print(f"  {w['what']:<44} at {w['at']:6.2f}  slack {w['slack']:+.2f}{mark}")
    print("\n## HOLDS: seconds fully shown, against characters at 15 per second")
    for r in runs:
        for h in r.holds:
            short = "  SHORT" if h["hold"] < h["need"] - 1e-9 else ""
            kept = " (still up)" if h["open"] else ""
            print(f"  {h['cue']:>5} from {h['from']:6.2f}  hold {h['hold']:5.2f}  need {h['need']:4.2f}  {h['text']}{kept}{short}")


main()
