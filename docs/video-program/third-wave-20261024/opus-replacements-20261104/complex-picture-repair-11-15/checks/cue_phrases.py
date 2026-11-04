"""Every spoken anchor in the five staged chapters resolves against the unchanged script.

`line.sentence(j)` and `line.at(phrase)` return errors when the sentence or phrase is missing, and
that only shows at render. This walks each chapter's source in order, tracks which cue `line` is
bound to, and checks every literal anchor: the phrase is in that cue, the sentence exists, and
anchors in one cue do not run backwards (a backward anchor just returns at once, so it is listed
as a warning). Anchors through a variable are listed for reading by eye.
Run from the repo root: python3 -I -B <this file>
"""
import re
import sys

STAGE = (
    "docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-picture-repair-11-15"
    "/candidate/explainer_corpus/complex_analysis_visual/src/chapters/"
)
SCRIPT = "explainer_corpus/complex_analysis_visual/script.tsv"
CUES = {}
for row in open(SCRIPT, encoding="utf-8"):
    parts = row.rstrip("\n").split("\t")
    if len(parts) >= 2:
        CUES[parts[0]] = parts[1]


def sentence_starts(text):
    return [0] + [m.end() for m in re.finditer(r"[.?!]\s+", text)]


# helpers that take `line` from a caller further up the file, and the cue that caller holds
HELPERS = {"approach_pole": "11_4", "coarse_loop": "11_8"}

ANCHOR = re.compile(
    r"\bfn (?P<fn>\w+)\b"
    r'|say\(&kit, stage, "(?P<cue>[\d_a-z]+)"\)'
    r'|line\.wait_for\(stage, "(?P<wait>[^"]+)"\)'
    r'|line\.until\(stage, "(?P<until>[^"]+)"'
    r'|line\.at\("(?P<at>[^"]+)"\)'
    r"|line\.wait_sentence\(stage, (?P<wait_s>[^)]+)\)"
    r"|line\.sentence\((?P<sent>[^)]+)\)"
    r"|line\.wait_for\(stage, (?P<wait_v>[^)\"]+)\)"
)

failed, count = False, 0
for chapter in ("counting", "residues", "real_integral", "continuing", "closing"):
    source = open(STAGE + chapter + ".rs", encoding="utf-8").read()
    # the helpers run with the cue of the call that hands them `line`; the tests read no cue
    source = source.split("#[cfg(test)]\nmod tests")[0]
    cue, last = None, -1
    print(f"--- {chapter}")
    for m in ANCHOR.finditer(source):
        if m.group("fn"):
            cue, last = HELPERS.get(m.group("fn")), -1
            continue
        if m.group("cue"):
            cue, last = m.group("cue"), -1
            if cue not in CUES:
                print("  FAIL no such cue", cue)
                failed = True
            continue
        if cue is None:
            # a helper defined before the first `say`: named with its phrases below
            where = "(helper, before any say)"
            text = None
        else:
            where = cue
            text = CUES[cue]
        phrase = m.group("wait") or m.group("until") or m.group("at")
        if phrase:
            count += 1
            texts = [text] if text else list(CUES.values())
            if not any(phrase in t for t in texts):
                print(f"  FAIL {where}: phrase not spoken: {phrase!r}")
                failed = True
                continue
            if text:
                pos = text.index(phrase)
                if pos < last:
                    print(f"  warn {where}: {phrase!r} is before the previous anchor")
                last = max(last, pos)
            continue
        expr = m.group("wait_s") or m.group("sent")
        if expr:
            count += 1
            if expr.isdigit():
                if text is None:
                    continue
                starts = sentence_starts(text)
                j = int(expr)
                if j >= len(starts):
                    print(f"  FAIL {where}: sentence {j} of {len(starts)}")
                    failed = True
                    continue
                if starts[j] < last:
                    print(f"  warn {where}: sentence {j} is before the previous anchor")
                last = max(last, starts[j])
            else:
                print(f"  read by eye {where}: sentence({expr})")
            continue
        if m.group("wait_v"):
            print(f"  read by eye {where}: wait_for({m.group('wait_v')})")

needs_said = re.search(r'const NEEDS_SAID: \[&str; 3\] = \[(.*?)\];', open(STAGE + "counting.rs", encoding="utf-8").read())
said = re.findall(r'"([^"]+)"', needs_said.group(1))
positions = [CUES["11_2"].index(p) for p in said]
print("NEEDS_SAID in 11_2, in order:", said, positions == sorted(positions), "; sentence 3 at", sentence_starts(CUES["11_2"])[3], "phrases at", positions)
if positions != sorted(positions) or positions[0] != sentence_starts(CUES["11_2"])[3]:
    failed = True

# 11_1's loop reads sentences through its index
print("11_1 sentences:", len(sentence_starts(CUES["11_1"])), "(the loop reads sentence(i + 2) and sentence(i + 3) for i < 2)")
if len(sentence_starts(CUES["11_1"])) != 5:
    failed = True
print(f"anchors checked {count}")
print("RESULT", "FAIL" if failed else "PASS")
sys.exit(1 if failed else 0)
