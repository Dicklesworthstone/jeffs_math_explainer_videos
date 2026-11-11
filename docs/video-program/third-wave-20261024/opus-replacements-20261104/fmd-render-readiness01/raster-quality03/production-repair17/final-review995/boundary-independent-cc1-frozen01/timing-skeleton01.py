"""Compare the timing skeleton of two boundary.rs files over the whole film part (read-only, no compile).

Film part = text before "\\n#[cfg(test)]\\nmod tests {". Extracts, in source order, every call that can take
scene time or bind narration: say(..), line.wait_for, line.wait_sentence, line.until, wait_until, kit.hold,
kit.music_breath, pause, stage.wait, fade_in_all/fade_out_all (run time), stage.play, stage.play_prepared,
tween_linear (target, run time), predict, names_now. Mob lists are ignored; kinds, phrases and the last
numeric literal (or argument tail) must match exactly. Also reports the count of add_updater/clear_updaters.

usage: python3 -I -B timing-skeleton01.py BASE.rs CANDIDATE.rs
"""
import re
import sys

KINDS = ["say(&kit, stage, ", "line.wait_for(", "line.wait_sentence(", "line.until(", "wait_until(", "kit.hold(",
         "kit.music_breath(", "pause(", "stage.wait(", "fade_in_all(", "fade_out_all(", "stage.play_prepared(",
         "stage.play(", "tween_linear(", "predict(", "names_now(", "stage.add_updater(", "stage.clear_updaters("]


def call_end(s, i):
    depth, j = 1, i
    while j < len(s):
        ch = s[j]
        if ch == 'r' and s[j + 1:j + 2] in ('"', '#') and not (s[j - 1].isalnum() or s[j - 1] == '_'):
            m = re.match(r'r(#*)"', s[j:])
            if m:
                close = '"' + m.group(1)
                k = s.index(close, j + len(m.group(0)))
                j = k + len(close)
                continue
        if ch == '"':
            j += 1
            while s[j] != '"':
                j += 2 if s[j] == '\\' else 1
            j += 1
            continue
        if ch == "'" and re.match(r"'(\\.|[^\\'])'", s[j:j + 4]):
            j += len(re.match(r"'(\\.|[^\\'])'", s[j:j + 4]).group(0))
            continue
        if ch == '/' and s[j + 1:j + 2] == '/':
            j = s.index('\n', j)
            continue
        if ch == '(':
            depth += 1
        elif ch == ')':
            depth -= 1
            if depth == 0:
                return j + 1
        j += 1
    raise ValueError("unbalanced")


def skeleton(path):
    src = open(path, encoding="utf-8").read()
    film = src.split("\n#[cfg(test)]\nmod tests {")[0]
    text = re.sub(r"//[^\n]*", "", film)
    out, pos = [], 0
    while True:
        hits = [(text.find(k, pos), k) for k in KINDS]
        hits = [(i, k) for i, k in hits if i >= 0 and (i == 0 or not (text[i - 1].isalnum() or text[i - 1] in "_."))
                or (i >= 0 and k.startswith(("line.", "kit.", "stage.")))]
        if not hits:
            break
        i, k = min(hits)
        if k == "stage.play(" and text.startswith("stage.play_prepared(", i):
            k = "stage.play_prepared("
        start = i + len(k)
        if k == "say(&kit, stage, ":
            end = call_end(text, start)
        else:
            end = call_end(text, start)
        args = text[start:end - 1]
        lit = re.findall(r'"([^"]*)"', args)
        nums = re.findall(r"(?<![\w.])(\d+\.\d+|\d+)(?![\w.])", args)
        if k in ("line.wait_for(", "say(&kit, stage, ", "line.until("):
            val = (lit[0] if lit else "VAR:" + args.strip()) + ("|" + nums[-1] if k == "line.until(" and nums else "")
        elif k in ("stage.add_updater(", "stage.clear_updaters("):
            val = args.split(",")[0].strip()
        else:
            val = nums[-1] if nums else args.strip()[-50:]
        out.append((k.rstrip("("), val))
        pos = start
    return out


a, b = skeleton(sys.argv[1]), skeleton(sys.argv[2])
print("base events:", len(a), " candidate events:", len(b))
same = a == b
if not same:
    for i, (x, y) in enumerate(zip(a, b)):
        if x != y:
            print("first difference at", i, "base", x, "candidate", y)
            break
    if len(a) != len(b):
        print("length differs")
kinds = {}
for k, _ in a:
    kinds[k] = kinds.get(k, 0) + 1
print("base kinds:", dict(sorted(kinds.items())))
print("TIMING SKELETON IDENTICAL:", same)
