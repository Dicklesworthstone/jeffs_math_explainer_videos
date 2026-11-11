"""Compare the timing skeleton of 08_5 between two loop_sums.rs files (read-only, no compile).

Extracts, in source order inside goursat_proof (and the 08_5 block of construct), every call that
can take scene time or bind narration: line.wait_for(phrase), fade_in_all(.., rt), fade_out_all(.., rt),
stage.play(..), stage.play_prepared(..), tween_linear(.., rt), .rt(x), wait_until, kit.hold,
reading_window, music_breath. Mob lists are ignored; kinds, phrases and run times must match exactly.

usage: python3 -I -B timing-compare03.py BASE.rs CANDIDATE.rs
"""
import re
import sys

KINDS = ["line.wait_for(", "fade_in_all(", "fade_out_all(", "stage.play_prepared(", "stage.play(",
         "tween_linear(", ".rt(", "wait_until(", "kit.hold(", "reading_window(", "kit.music_breath("]


def call_end(s, i):
    """Index just past the parenthesis that closes the call opened at s[i-1] == '('."""
    depth, j = 1, i
    while j < len(s):
        ch = s[j]
        if ch == 'r' and s[j + 1:j + 2] == '"' and not (s[j - 1].isalnum() or s[j - 1] == '_'):
            k = s.index('"', j + 2)
            j = k + 1
            continue
        if ch == '"':
            j += 1
            while s[j] != '"':
                j += 2 if s[j] == '\\' else 1
            j += 1
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
    body = src.split("fn goursat_proof(")[1].split("pub struct LoopSums")[0]
    block = src[src.index('let line = say(&kit, stage, "08_5")?;'):src.index("kit.finish(stage)", src.index('"08_5"'))]
    out = []
    for name, text in (("goursat_proof", body), ("construct 08_5", block)):
        # drop line comments so commented code is never counted
        text = re.sub(r"//[^\n]*", "", text)
        pos = 0
        while True:
            hits = [(text.find(k, pos), k) for k in KINDS]
            hits = [(i, k) for i, k in hits if i >= 0]
            if not hits:
                break
            i, k = min(hits)
            # stage.play( must not swallow stage.play_prepared(
            if k == "stage.play(" and text.startswith("stage.play_prepared(", i):
                k = "stage.play_prepared("
            start = i + len(k)
            end = call_end(text, start)
            args = text[start:end - 1]
            if k == "line.wait_for(":
                lit = re.search(r'"([^"]*)"', args)
                val = lit.group(1) if lit else "VAR:" + args.strip()
            else:
                nums = re.findall(r"(?<![\w.])(\d+\.\d+|\d+)(?![\w.])", args)
                val = nums[-1] if nums else args.strip()[-40:]
            out.append((name, k.rstrip("("), val))
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
phrases = [v for _, k, v in a if k == "line.wait_for"]
print("wait_for phrases:", len(phrases), " fades/plays/tweens:", len(a) - len(phrases))
print("TIMING SKELETON IDENTICAL:", same)
