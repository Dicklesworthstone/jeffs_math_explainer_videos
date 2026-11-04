"""Cheap source check for the logarithm rebuild candidate (no Cargo).

Run from the crate root:  python3 -I -B docs/check_cues.py

Checks, against script.tsv and the kit's sentence rule (split after . ? !
followed by a space or the end, as in explainer_kit narration.rs):
  * every cue is said exactly once, by exactly one chapter, in script order;
  * every until_sentence(id, j) names a said cue and 0 < j < sentences(id),
    and within a cue the indices never go backwards;
  * the chapter registry in src/chapters/mod.rs lists chapters in cue order;
  * spoken text holds no digits (numbers are spelled for the narrator);
  * string literals in chapter code that contain digits are listed, so a
    reviewer can confirm each is a label, an index or a facts.rs value.
Exit status 1 on any failure; the literal listing is informational.
"""

import pathlib
import re
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def sentences(text):
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            part = text[start : i + 1].strip()
            if part:
                out.append(part)
            start = i + 1
    rest = text[start:].strip()
    if rest:
        out.append(rest)
    return out


def script():
    cues = []
    for line in (ROOT / "script.tsv").read_text(encoding="utf-8").splitlines():
        if not line or line.startswith("#"):
            continue
        cid, text = line.split("\t", 1)
        cues.append((cid, text))
    return cues


SAY = re.compile(r'kit\.say\(\s*stage\s*,\s*"([0-9]{2}_[0-9]+)"\s*\)')
UNTIL = re.compile(r'kit\.until_sentence\(\s*stage\s*,\s*"([0-9]{2}_[0-9]+)"\s*,\s*([0-9]+)\s*\)')
STRING = re.compile(r'"((?:[^"\\]|\\.)*)"')


def main():
    failures = []
    cues = script()
    order = {cid: k for k, (cid, _) in enumerate(cues)}
    count = {cid: len(sentences(text)) for cid, text in cues}
    if len(order) != len(cues):
        failures.append("duplicate cue ids in script.tsv")
    for cid, text in cues:
        if re.search(r"[0-9]", text):
            failures.append(f"{cid}: digits in spoken text")

    said_by = {}
    chapter_first = {}
    for rs in sorted((ROOT / "src" / "chapters").glob("*.rs")):
        src = rs.read_text(encoding="utf-8")
        events = []
        for m in SAY.finditer(src):
            events.append((m.start(), "say", m.group(1), None))
        for m in UNTIL.finditer(src):
            events.append((m.start(), "until", m.group(1), int(m.group(2))))
        events.sort()
        last_said, last_j = None, {}
        for _, kind, cid, j in events:
            if cid not in order:
                failures.append(f"{rs.name}: unknown cue {cid}")
                continue
            if kind == "say":
                said_by.setdefault(cid, []).append(rs.name)
                if last_said is not None and order[cid] <= order[last_said]:
                    failures.append(f"{rs.name}: {cid} said after {last_said}")
                last_said = cid
                chapter_first.setdefault(rs.stem, order[cid])
            else:
                if cid != last_said:
                    failures.append(f"{rs.name}: until_sentence({cid}, {j}) while {last_said} is the current line")
                if not 0 < j < count[cid]:
                    failures.append(f"{rs.name}: until_sentence({cid}, {j}) but {cid} has {count[cid]} sentences")
                if j < last_j.get(cid, 0):
                    failures.append(f"{rs.name}: until_sentence({cid}, {j}) goes backwards")
                last_j[cid] = j
    for cid, _ in cues:
        who = said_by.get(cid, [])
        if len(who) != 1:
            failures.append(f"{cid}: said {len(who)} times ({', '.join(who) or 'never'})")

    mod_rs = ROOT / "src" / "chapters" / "mod.rs"
    if mod_rs.exists():
        keys = re.findall(r'key:\s*"([0-9]{2})_', mod_rs.read_text(encoding="utf-8"))
        if keys != sorted(keys) or len(set(keys)) != len(keys):
            failures.append(f"mod.rs registry out of order: {keys}")
        prefixes = sorted({cid[:2] for cid, _ in cues})
        if keys and keys != prefixes:
            failures.append(f"mod.rs registry {keys} does not cover cue prefixes {prefixes}")
    else:
        failures.append("src/chapters/mod.rs missing")

    print(f"{len(cues)} cues, {sum(count.values())} sentences, {len(said_by)} said")
    print("string literals with digits (review: label, index or facts value):")
    for rs in sorted((ROOT / "src" / "chapters").glob("*.rs")):
        body = rs.read_text(encoding="utf-8").split("#[cfg(test)]")[0]
        for n, line in enumerate(body.splitlines(), 1):
            if line.strip().startswith("//"):
                continue
            for m in STRING.finditer(line):
                lit = m.group(1)
                if re.search(r"[0-9]", lit) and not re.fullmatch(r"[0-9]{2}_[0-9]+|[0-9]{2}_[a-z_]+|[a-z_]+\.[a-z0-9_]+", lit):
                    print(f"  {rs.name}:{n}: \"{lit}\"")
    if failures:
        print("FAIL")
        for f in failures:
            print("  " + f)
        return 1
    print("OK")
    return 0


if __name__ == "__main__":
    sys.exit(main())
