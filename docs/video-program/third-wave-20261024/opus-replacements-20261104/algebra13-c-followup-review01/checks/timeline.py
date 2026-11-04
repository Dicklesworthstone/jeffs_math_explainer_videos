"""Cue-relative timeline of the sites patch C changed, read from a candidate's Rust text, on Oliver take01.

python3 -I -B timeline.py LABEL=PATH/arithmetic.rs [LABEL=PATH ...]   (from the repo root; prints only)

A label ending in :13_8 reports only that cue. It is the positive control: AB must give RubyRaven's F1 hold of 0.04 s.

Written for this follow-up. It imports nothing from RubyRaven's replay.py or from desk_mirror.py. It does share their
idea (add up literal run times), and I read replay.py's output first, so it is a cross-check rather than a blind one.

Word and sentence times are root's take01 recognition files, large and small models. They are recognition, not
hearing, and not a statement about the take root finally selects. Sums of literal run times are predictions: no frame
of ABC exists. An unknown statement that touches the stage stops the script, so nothing is silently skipped.
"""
import json
import re
import sys
from pathlib import Path

ASR = Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
SCRIPT = Path("explainer_corpus/abstract_algebra_expanded/script.tsv")
FADE = 0.4
SETTLE = 0.25

# (cue, first statement of the site, first statement after it). None means start at the cue's say.
SITES = {
    "13_3": ('kit.say(stage, "13_3")?;', 'kit.say(stage, "13_4")?;'),
    "13_7": ('kit.until_sentence(stage, "13_7", 1)?;', 'kit.until_sentence(stage, "13_7", 4)?;'),
    "13_8": ('kit.say(stage, "13_8")?;', 'kit.until_sentence(stage, "13_8", 4)?;'),
}


def words(cue, model):
    tag = cue.replace("_", "-")
    data = json.loads((ASR / f"expanded-algebra-{tag}-asr01/take01-{model}.json").read_text(encoding="utf-8"))
    return [(s["text"].strip(), float(s["start_sec"]), float(s["end_sec"])) for s in data["result"]["segments"]]


def sentence_starts(rows):
    starts = [rows[0][1]]
    for (text, _, _), (_, start, _) in zip(rows, rows[1:]):
        if text.endswith("."):
            starts.append(start)
    return starts


def spoken(cue):
    for line in SCRIPT.read_text(encoding="utf-8").splitlines():
        cells = line.split("\t")
        if cells[0] == cue:
            return cells[-1]
    raise SystemExit(f"{cue} is not in the script")


def word_at(rows, text, after=0.0):
    for word, start, end in rows:
        if start >= after and word.strip(".,").lower() == text.lower():
            return start, end
    raise SystemExit(f"no word {text!r} after {after}")


class Replay:
    def __init__(self, source, cue, model, marks=False):
        self.source, self.cue, self.model, self.marks = source, cue, model, marks
        self.rows = words(cue, model)
        self.starts = sentence_starts(self.rows)
        self.text = spoken(cue)
        self.t = 0.0
        self.events = []
        self.slack = {}
        self.times = {m.group(1): float(m.group(2)) for m in re.finditer(r"const (\w+_RT): f64 = ([\d.]+);", source)}
        self.phrases = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)";', source))

    def log(self, what, **extra):
        self.events.append(dict(what=what, at=round(self.t, 2), **extra))

    def run_time(self, token):
        return self.times[token] if token in self.times else float(token)

    def phrase_time(self, j, phrase):
        sentences = re.split(r"(?<=[.!?])\s+", self.text)
        sentence = sentences[j]
        if self.marks:
            # What a phrase mark bound to measured audio would give: the phrase's first recognised word.
            return word_at(self.rows, phrase.split()[0], after=self.starts[j])[0]
        offset = sentence.index(phrase)
        start = self.starts[j]
        end = self.starts[j + 1] if j + 1 < len(self.starts) else self.rows[-1][2]
        return start + (end - start) * offset / len(sentence)

    def statement(self, line):
        s = line.strip()
        if not s or s.startswith("//"):
            return
        m = re.fullmatch(r'kit\.until_sentence\(stage, "([^"]+)", (\d+)\)\?;', s)
        if m:
            target = self.starts[int(m.group(2))]
            self.slack[f"s{m.group(2)}"] = round(target - self.t, 2)
            self.log(f"wait s{m.group(2)}", slack=round(target - self.t, 2))
            self.t = max(self.t, target)
            return
        m = re.fullmatch(r'until_phrase\(stage, &kit, "([^"]+)", (\d+), (\w+)\)\?;', s)
        if m:
            j = int(m.group(2))
            self.t = max(self.t, self.starts[j])
            target = self.phrase_time(j, self.phrases[m.group(3)])
            self.log(f"wait phrase {m.group(3)}", target=round(target, 2), slack=round(target - self.t, 2))
            self.t = max(self.t, target)
            return
        m = re.fullmatch(r"fade\(stage, &(\[.*\]|\w+), false\)\?;", s)
        if m:
            self.log("fade", of=m.group(1), until=round(self.t + FADE, 2))
            self.t += FADE
            return
        m = re.fullmatch(r"perform\(stage, (\w+), ([\d.]+), (\w+|[\d.]+), &\[[^\]]*\]\)\?;", s)
        if m:
            rt = self.run_time(m.group(3))
            self.log("move", tile=m.group(1), to=float(m.group(2)), lands=round(self.t + rt, 2))
            self.t += rt + SETTLE
            return
        m = re.fullmatch(r"advance\(stage, (\w+), ([\d.]+)\)\?;", s)
        if m:
            self.log("move", tile=m.group(1), to=float(m.group(2)), lands=round(self.t + 1.7, 2))
            self.t += 1.7 + 0.5
            return
        m = re.fullmatch(r'(?:\w+ = |let mut \w+ = |let \w+ = )?step\(stage, &kit, (\w+), ([\d.]+), (\w+), "([^"]+)", \w+\)\?;', s)
        if m:
            self.log("fade", of=m.group(3), until=round(self.t + FADE, 2))
            self.t += FADE
            self.log("move", tile=m.group(1), to=float(m.group(2)), lands=round(self.t + 1.7, 2))
            self.t += 2.2
            self.log("caption", text=m.group(4), whole=round(self.t + FADE, 2))
            self.t += FADE
            return
        m = re.fullmatch(r'(?:\w+ = |let mut \w+ = |let \w+ = )?caption\(stage, &kit, "([^"]+)", [^;]*\)\?;', s)
        if m:
            self.log("caption", text=m.group(1), whole=round(self.t + FADE, 2))
            self.t += FADE
            return
        m = re.fullmatch(r'(?:\w+ = |let mut \w+ = )?replace\(stage, &kit, (\w+), "([^"]+)", [^;]*\)\?;', s)
        if m:
            self.log("fade", of=m.group(1), until=round(self.t + FADE, 2))
            self.t += FADE
            self.log("caption", text=m.group(2), whole=round(self.t + FADE, 2))
            self.t += FADE
            return
        m = re.fullmatch(r"let \w+ = grouping\(stage, [^;]*\)\?;", s)
        if m:
            self.log("box", whole=round(self.t + FADE, 2))
            self.t += FADE
            return
        m = re.fullmatch(r"d4g::glide\(stage, \w+, \([^)]*\), ([\d.]+)\)\?;", s)
        if m:
            self.log("glide", until=round(self.t + float(m.group(1)), 2))
            self.t += float(m.group(1))
            return
        m = re.fullmatch(r"d4g::hide\(stage, [^,]+, ([\d.]+)\)\?;", s)
        if m:
            self.log("rack leaves", until=round(self.t + float(m.group(1)), 2))
            self.t += float(m.group(1))
            return
        m = re.fullmatch(r"stage\.wait\(([\d.]+)\)\?;", s)
        if m:
            self.t += float(m.group(1))
            return
        if "add_value_tracker" in s or "add_tile" in s:
            if "add_tile" in s:
                self.log("tile appears at once")
            return
        if s.startswith('kit.say(stage, "'):
            self.t = 0.0
            return
        if "stage" in s:
            raise SystemExit(f"unknown statement in {self.cue}: {s}")

    def run(self):
        first, last = SITES[self.cue]
        lines = self.source.splitlines()
        i = next(n for n, line in enumerate(lines) if line.strip() == first)
        j = next(n for n, line in enumerate(lines) if n > i and line.strip() == last)
        for line in lines[i:j]:
            self.statement(line)
        if self.cue == "13_8":
            target = self.starts[4]
            self.slack["s4"] = round(target - self.t, 2)
            self.log("wait s4", slack=self.slack["s4"])
        return self


def caption_rows(events):
    """Each caption with when it is whole and when the next fade after it starts."""
    rows = []
    for i, event in enumerate(events):
        if event["what"] != "caption":
            continue
        leaves = next((e["at"] for e in events[i + 1:] if e["what"] == "fade"), None)
        rows.append((event["text"], event["at"], event["whole"], leaves))
    return rows


def only_13_8(source, model):
    r8 = Replay(source, "13_8", model).run()
    name = next(c for c in caption_rows(r8.events) if c[0] == "r³ r = e")
    print(f"  {model}: r³ r = e whole {name[2]:.2f}, leaves {name[3]:.2f}: hold {name[3] - name[2]:.2f}; s4 slack {r8.slack['s4']:+.2f}")


def report(label, source):
    print(f"\n######## {label}")
    if label.endswith(":13_8"):
        for model in ("large", "small"):
            only_13_8(source, model)
        return
    for model in ("large", "small"):
        print(f"\n---- {model} model")
        r3 = Replay(source, "13_3", model).run()
        rows = r3.rows
        print("13_3 starts", r3.starts)
        for event in r3.events:
            print("   ", event)
        caps = {c[0]: c for c in caption_rows(r3.events)}
        r_at, _ = word_at(rows, "R")
        cubed, _ = word_at(rows, "cubed")
        r_cubed = max(s for w, s, _ in rows if w == "R" and s < cubed)
        four, _ = word_at(rows, "Four")
        lands = next(e["lands"] for e in r3.events if e["what"] == "move" and e["to"] == 3.0)
        name = caps["r³"]
        print(f"  r²: whole {caps['r²'][2]:.2f}; voice says R at {r_at:.2f}: lead {r_at - caps['r²'][2]:+.2f}")
        print(f"  r³: tile lands {lands:.2f}; fade-in {name[1]:.2f}; whole {name[2]:.2f}; leaves {name[3]:.2f}; hold {name[3] - name[2]:.2f}")
        print(f"      voice: R {r_cubed:.2f}, cubed {cubed:.2f}, Four {four:.2f}")
        print(f"      tile lands {lands - cubed:+.2f} from cubed; whole {name[2] - cubed:+.2f} from cubed; whole {name[2] - four:+.2f} from Four")
        print(f"  slack at s6 (r⁴ = e): {r3.slack['s6']:+.2f}")

        r7 = Replay(source, "13_7", model).run()
        print("13_7 starts", r7.starts)
        for event in r7.events:
            print("   ", event)
        rows = r7.rows
        rev_s, rev_e = word_at(rows, "reverse")
        if_s, _ = word_at(rows, "if")
        rule = next(c for c in caption_rows(r7.events) if c[0] == "s r = r⁻¹ s")
        print(f"  rule: fade-in {rule[1]:.2f}, whole {rule[2]:.2f}; voice: if {if_s:.2f}, reverse {rev_s:.2f} to {rev_e:.2f}")
        print(f"        fade-in starts {rule[1] - rev_s:+.2f} from reverse's start; whole {rule[2] - rev_e:+.2f} from reverse's recognised end")
        print(f"  slack: s2 {r7.slack['s2']:+.2f}, s3 {r7.slack['s3']:+.2f}")

        r8 = Replay(source, "13_8", model).run()
        print("13_8 starts", r8.starts)
        for event in r8.events:
            print("   ", event)
        rows = r8.rows
        undo_s, undo_e = word_at(rows, "undo")
        hide = next((e for e in r8.events if e["what"] == "rack leaves"), None)
        name = next(c for c in caption_rows(r8.events) if c[0] == "r³ r = e")
        print(f"  rack leaves {hide['at']:.2f} to {hide['until']:.2f}; voice: undo {undo_s:.2f} to {undo_e:.2f}")
        print(f"  r³ r = e: fade-in {name[1]:.2f}, whole {name[2]:.2f}, leaves {name[3]:.2f}: hold {name[3] - name[2]:.2f}")
        print(f"            against 5 glyphs at 15/s = 0.33 s: {name[3] - name[2] - 5 / 15:+.2f}; against 8 with spaces = 0.53 s: {name[3] - name[2] - 8 / 15:+.2f}")
        print(f"  slack: s2 {r8.slack['s2']:+.2f}, s3 {r8.slack['s3']:+.2f}, s4 {r8.slack['s4']:+.2f}")

        m7 = Replay(source, "13_7", model, marks=True).run()
        m8 = Replay(source, "13_8", model, marks=True).run()
        rule = next(c for c in caption_rows(m7.events) if c[0] == "s r = r⁻¹ s")
        back = next(c for c in caption_rows(m7.events) if c[0] == "r⁻¹")
        hide = next(e for e in m8.events if e["what"] == "rack leaves")
        name = next(c for c in caption_rows(m8.events) if c[0] == "r³ r = e")
        print("  if the two phrases are bound to their first recognised word, rather than a character share:")
        print(f"    13_7 rule fade-in {rule[1]:.2f}, whole {rule[2]:.2f}; r⁻¹ whole {back[2]:.2f}; s2 slack {m7.slack['s2']:+.2f}")
        print(f"    13_8 rack leaves {hide['at']:.2f} to {hide['until']:.2f}; s2 slack {m8.slack['s2']:+.2f}; r³ r = e hold {name[3] - name[2]:.2f}")


def main():
    for arg in sys.argv[1:]:
        label, path = arg.split("=", 1)
        report(label, Path(path).read_text(encoding="utf-8"))


if __name__ == "__main__":
    main()
