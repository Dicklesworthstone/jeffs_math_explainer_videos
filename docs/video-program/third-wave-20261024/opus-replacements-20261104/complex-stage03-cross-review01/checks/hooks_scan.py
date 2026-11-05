"""Scan a complex crate's scene code for its own Spoken API: cue order, sentence and phrase hooks. Reads only.

    python3 -I -B hooks_scan.py <crate> [<older-script.tsv>] [--only file.rs,...]
"""
import difflib
import pathlib
import re
import sys

args = [a for a in sys.argv[1:] if not a.startswith("--")]
CRATE = pathlib.Path(args[0])
OLD_SCRIPT = pathlib.Path(args[1]) if len(args) > 1 else None
ONLY = next((a.split("=", 1)[1].split(",") for a in sys.argv[1:] if a.startswith("--only=")), None)
script_rows = [l.split("\t", 1) for l in (CRATE / "script.tsv").read_text(encoding="utf-8").splitlines() if "\t" in l and not l.startswith("#")]
script = dict(script_rows)
order = [k for k, _ in script_rows]


def offsets(text):
    out = [0]
    for i in range(len(text) - 2):
        if text[i] in ".?!" and text[i + 1] == " ":
            out.append(i + 2)
    return out


mod = (CRATE / "src/chapters/mod.rs").read_text(encoding="utf-8")
files = re.findall(r"Box::new\((\w+)::", mod)
said_all, problems, phrase_rows = [], [], []
SAY = re.compile(r'let (?:mut )?(\w+) = say\(&kit, stage, "(\d\d_\d+)"\)')
SENT = re.compile(r'\b(\w+)\.(wait_sentence|sentence)\((?:stage, )?(\d+)\)')
SENT_COMPUTED = re.compile(r'\b(\w+)\.(wait_sentence|sentence)\((?:stage, )?([^)\d][^)]*)\)')
PHR = re.compile(r'\b(\w+)\.(wait_for|at|until)\((?:&?\*?stage, )?"([^"]+)"')
PHR_COMPUTED = re.compile(r'\b(\w+)\.(wait_for|at|until)\((?:&?\*?stage, )?([A-Za-z_][\w:]*)[,)]')
for name in files:
    if ONLY and f"{name}.rs" not in ONLY:
        continue
    text = (CRATE / "src/chapters" / f"{name}.rs").read_text(encoding="utf-8")
    film = re.split(r"#\[cfg\(test\)\]\s*mod tests", text)[0]
    var, last = {}, {}
    for no, line in enumerate(film.split("\n"), 1):
        where = f"{name}.rs:{no}"
        for m in SAY.finditer(line):
            var[m.group(1)] = m.group(2)
            said_all.append((m.group(2), where))
        for m in SENT.finditer(line):
            v, kind, j = m.group(1), m.group(2), int(m.group(3))
            if v not in var:
                continue
            cue = var[v]
            n = len(offsets(script[cue]))
            if not 0 <= j < n:
                problems.append(f"{where} {cue}.{kind}({j}): only {n} sentences")
            if kind == "wait_sentence":
                prev = last.get(cue, (-1, -1))
                at = offsets(script[cue])[min(j, n - 1)]
                if at < prev[0]:
                    problems.append(f"{where} {cue}.wait_sentence({j}) after a later wait ({prev[1]})")
                last[cue] = (max(prev[0], at), where)
        for m in SENT_COMPUTED.finditer(line):
            if m.group(1) in var:
                phrase_rows.append(f"COMPUTED-SENTENCE {where} {var[m.group(1)]} {m.group(0)}")
        for m in PHR.finditer(line):
            v, kind, phrase = m.groups()
            if v not in var:
                continue
            cue = var[v]
            t = script[cue]
            count = t.count(phrase)
            byte = t.find(phrase)
            starts_word = byte == 0 or (byte > 0 and not t[byte - 1].isalnum())
            row = f"{where} {cue} {kind}({phrase!r}) byte {byte} count {count}"
            if count == 0:
                problems.append(row + ": NOT IN THE CUE")
            elif count > 1:
                problems.append(row + ": occurs more than once; the hook takes the first")
            elif not starts_word:
                problems.append(row + ": does not begin a word")
            if kind == "wait_for" and count:
                prev = last.get(cue, (-1, -1))
                if byte < prev[0]:
                    problems.append(f"{row}: waits for words already passed ({prev[1]})")
                last[cue] = (max(prev[0], byte), where)
            phrase_rows.append(row)
        for m in PHR_COMPUTED.finditer(line):
            if m.group(1) in var:
                phrase_rows.append(f"COMPUTED-PHRASE {where} {var[m.group(1)]} {m.group(0)}")

said = [c for c, _ in said_all]
if not ONLY:
    print("cues said", len(said), "script cues", len(order), "same order", said == order)
    if said != order:
        for row in difflib.unified_diff(order, said, "script", "said", lineterm="", n=1):
            print("  ", row)
    print("said twice:", sorted({c for c in said if said.count(c) > 1}))
    for cue, text in script_rows:
        if re.search(r"\d", text):
            print("DIGIT in spoken text", cue)
print("== problems")
print("\n".join(problems) or "none")
print("== phrase hooks")
print("\n".join(phrase_rows))
if OLD_SCRIPT:
    old = dict(l.split("\t", 1) for l in OLD_SCRIPT.read_text(encoding="utf-8").splitlines() if "\t" in l and not l.startswith("#"))
    print("\n== cues changed since", OLD_SCRIPT.name, [c for c in order if old.get(c) != script[c]])
