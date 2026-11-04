"""Does each changed beat fit its sentence? Scratch timing only.

Mirrors `draw::Spoken` without a narrator: length = words / 2.6 + 0.4, onsets by character share.
Oliver's takes will differ, and the kit uses measured sentence starts when it has them, so this is
a plausibility check on the blocks' seconds, not fine-sync. "late" = how far past the voice's moment
the picture is when that beat starts. Run from the repo root: python3 -I -B <this file>
"""
import re

SCRIPT = "explainer_corpus/complex_analysis_visual/script.tsv"
CUES = {}
for row in open(SCRIPT, encoding="utf-8"):
    parts = row.rstrip("\n").split("\t")
    if len(parts) >= 2:
        CUES[parts[0]] = parts[1]


class Cue:
    def __init__(self, key):
        self.key, self.text = key, CUES[key]
        self.len = len(self.text.split()) / 2.6 + 0.4
        self.starts = [0] + [m.end() for m in re.finditer(r"[.?!]\s+", self.text)]
        self.t = 0.0
        self.rows = []
        print(f"\n{key}: scratch length {self.len:.2f} s, {len(self.starts)} sentences")

    def share(self, at):
        return self.len * at / len(self.text)

    def s(self, j):
        return self.share(self.starts[j])

    def at(self, phrase):
        return self.share(self.text.index(phrase))

    def go(self, seconds):
        self.t += seconds

    def until(self, moment, name):
        late = self.t - moment
        print(f"  {name:<38} voice {moment:6.2f}  picture ready {self.t:6.2f}  late {max(late, 0.0):5.2f}")
        self.t = max(self.t, moment)
        return late

    def wait(self, j):
        return self.until(self.s(j), f"sentence {j}")

    def wait_for(self, phrase):
        return self.until(self.at(phrase), repr(phrase))

    def finish(self):
        print(f"  ends at {self.t:6.2f}; voice ends {self.len:6.2f}; over by {max(self.t - self.len, 0.0):5.2f}")
        return self.t - self.len


c = Cue("11_1")
c.go(0.8 + 0.5)
for i in range(3):
    c.wait(i + 2)
    if i > 0:
        c.go(0.2 + 0.5)
    c.go(0.2)
    until = c.s(i + 3) if i < 2 else c.len
    room = until - c.t - 0.3 - 0.2 - 0.8
    rt = min(max(room, 1.8), 2.6)
    c.go(0.3 + 0.2 + rt + 0.8)
    print(f"    walk {i}: room {room:.2f}, walked {rt:.2f}, count held until {c.t:.2f}, sentence over at {until:.2f}")
c.finish()

c = Cue("11_2")
c.go(0.3 + 0.6 + 0.2)
room = c.s(1) - c.t - 0.3 - 0.2
rt = max(room, 2.6)
c.go(0.3 + 0.2 + rt)
print(f"    walk: room {room:.2f}, walked {rt:.2f}")
c.wait(1)
c.go(0.5)
c.wait_for("each counted")
c.go(0.3)
c.wait(2)
c.go(0.4)
c.wait(3)
c.go(0.2 + 3 * 0.7)
c.finish()

c = Cue("11_3")
c.go(0.4 + 1.0 + 0.3 + 0.3)
c.wait(1)
c.go(0.3 + 0.3 + 0.2 + 2.6)
c.wait(2)
c.go(0.2 + 0.9 + 0.2 + 0.2 + 2.8)
c.wait(3)
c.wait(4)
c.go(0.5)
c.finish()

c = Cue("11_8")
c.t = c.s(4)
c.go(0.2 + 0.3 + 0.4)
c.wait_for("take steps")
c.go(0.3 + 0.4 + 0.4)
c.wait_for("cannot move")
c.go(0.2)
rt = max(c.s(5) - c.t, 1.6)
print(f"    the reach is carried round in {rt:.2f} s")
c.go(rt)
c.wait(5)
c.go(0.4)
c.finish()

c = Cue("12_2")
c.go(0.4 + 0.3 + 0.4 + 0.5 + 12 * 0.1 + 0.4 + 0.4 + 0.4)
c.wait(1)
c.go(0.4)
c.wait(2)
c.go(0.4)
c.wait_for("the residue is")
c.go(0.4)
c.finish()

c = Cue("13_2")
c.go(0.4 + 0.3 + 0.2)
rt = max(c.at("then come back") - c.t, 1.4)
print(f"    the segment is walked in {rt:.2f} s")
c.go(rt + 0.2)
rt = max(c.at("Inside this loop") - c.t, 1.6)
print(f"    the half circle is walked in {rt:.2f} s")
c.go(rt + 0.2 + 0.2)
c.wait(1)
c.go(0.9)
c.wait(2)
c.finish()

c = Cue("13_8")
c.go(0.3)
c.wait(1)
c.go(1.2)
c.wait_for("each has imaginary part")
c.go(0.4)
c.wait(2)
c.go(0.8)
c.finish()

c = Cue("14_2")
c.go(0.4 + 0.5 + 0.5)
c.wait(1)
c.go(0.4)
c.wait(2)
c.go(0.3 + 0.3 + 11 * 0.12 + 0.4)
c.wait(3)
c.go(0.3 + 11 * 0.12 + 0.4)
c.wait(4)
c.go(0.5)
c.wait(5)
c.go(0.3)
c.finish()

c = Cue("14_3")
c.go(0.4 + 0.3 + 0.4 + 0.3 + 0.3 + 0.4)
c.wait(1)
c.go(0.5)
c.wait(3)
c.go(0.3 + 0.6)
c.wait(4)
c.go(0.3 + 0.3 + 0.2)
for k in range(1, 9):
    c.go(0.3 + 0.35)
    if k == 4:
        c.wait(5)
        c.go(0.3)
c.go(0.2)
c.wait(6)
c.go(0.4 + 0.4)
c.finish()
