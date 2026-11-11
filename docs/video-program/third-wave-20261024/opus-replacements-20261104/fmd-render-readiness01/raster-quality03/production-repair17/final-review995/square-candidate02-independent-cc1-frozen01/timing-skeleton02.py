"""Whole-film timing skeleton of two loop_sums.rs files (read-only, no compile). CC1, square candidate02 review.

Film part = text before "\\n#[cfg(test)]\\nmod tests {"; comments stripped. Extracts, in source order, every call
that can take scene time, bind narration or start/stop motion: say, line.wait_for / wait_sentence / until,
wait_until, kit.hold, kit.music_breath, pause, stage.wait, fade_in_all, fade_out_all, bare fade_in / fade_out,
stage.play / play_prepared / prepare, tween_linear, tween, any `.rt(` run-time override, predict, names_now,
stage.add_updater / clear_updaters. For each: the kind, the first string literal (phrases) and EVERY numeric
literal in its argument text. Mob lists are ignored only in the sense that identifiers are not compared.

usage: python3 -I -B timing-skeleton02.py A.rs B.rs
       python3 -I -B timing-skeleton02.py --variant SRC.rs OUT.rs   (control: c_aside.rt(0.3) -> 0.35, refuses to overwrite)
"""
import re
import sys

NAMES = ["say", r"line\.wait_for", r"line\.wait_sentence", r"line\.until", "wait_until", r"kit\.hold",
         r"kit\.music_breath", "pause", r"stage\.wait", "fade_in_all", "fade_out_all", "fade_out", "fade_in",
         r"stage\.play_prepared", r"stage\.play", r"stage\.prepare", "tween_linear", "tween", "predict",
         "names_now", r"stage\.add_updater", r"stage\.clear_updaters"]
CALL = re.compile(r"(?<![\w.])(" + "|".join(NAMES) + r")\(|\.(rt)\(")
SPLIT = "\n#[cfg(test)]\nmod tests {"


def strip_comments(s):
    out, i = [], 0
    while i < len(s):
        if s.startswith("//", i):
            i = s.index("\n", i)
            continue
        m = re.match(r'r(#*)"', s[i:]) if s[i] == "r" and (i == 0 or not (s[i - 1].isalnum() or s[i - 1] == "_")) else None
        if m:
            close = '"' + m.group(1)
            j = s.index(close, i + len(m.group(0))) + len(close)
            out.append(s[i:j])
            i = j
            continue
        if s[i] == '"':
            j = i + 1
            while s[j] != '"':
                j += 2 if s[j] == "\\" else 1
            out.append(s[i:j + 1])
            i = j + 1
            continue
        out.append(s[i])
        i += 1
    return "".join(out)


def args_end(s, i):
    depth, j = 1, i
    while j < len(s):
        ch = s[j]
        if ch == '"':
            j += 1
            while s[j] != '"':
                j += 2 if s[j] == "\\" else 1
        elif ch == "r" and re.match(r'r#+"', s[j:]) and not (s[j - 1].isalnum() or s[j - 1] == "_"):
            m = re.match(r'r(#+)"', s[j:])
            j = s.index('"' + m.group(1), j + len(m.group(0))) + len(m.group(1))
        elif ch == "(":
            depth += 1
        elif ch == ")":
            depth -= 1
            if depth == 0:
                return j
        j += 1
    raise ValueError("unbalanced")


def skeleton(path):
    film = strip_comments(open(path, encoding="utf-8").read().split(SPLIT)[0])
    out = []
    for m in CALL.finditer(film):
        kind = m.group(1) or ".rt"
        start = m.end()
        args = film[start:args_end(film, start)]
        lit = re.findall(r'"((?:\\.|[^"\\])*)"', args)
        nums = re.findall(r"(?<![\w.])(\d+\.\d+|\d+)(?![\w.])", re.sub(r'"(?:\\.|[^"\\])*"', '""', args))
        out.append((kind, lit[0] if lit and kind in ("say", "line.wait_for", "line.wait_sentence", "line.until",
                                                      "wait_until") else "", tuple(nums)))
    return out


if sys.argv[1] == "--variant":
    src = open(sys.argv[2], encoding="utf-8").read()
    film, sep, tests = src.partition(SPLIT)
    old = "c_aside.rt(0.3)"
    assert sep and film.count(old) == 1, film.count(old)
    with open(sys.argv[3], "x", encoding="utf-8", newline="") as f:
        f.write(film.replace(old, "c_aside.rt(0.35)") + sep + tests)
    print("wrote control variant", sys.argv[3])
    sys.exit(0)

a, b = skeleton(sys.argv[1]), skeleton(sys.argv[2])
kinds = {}
for k, _, _ in a:
    kinds[k] = kinds.get(k, 0) + 1
print("events:", len(a), "vs", len(b))
print("kinds (first file):", dict(sorted(kinds.items())))
diff = next((i for i, (x, y) in enumerate(zip(a, b)) if x != y), None)
if diff is not None:
    print("first difference at", diff, a[diff], "vs", b[diff])
print("TIMING SKELETON IDENTICAL:", a == b)
