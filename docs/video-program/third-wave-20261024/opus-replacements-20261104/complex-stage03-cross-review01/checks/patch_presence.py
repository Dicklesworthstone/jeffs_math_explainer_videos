"""Is each integrated patch's post-image in the stage03 tree? Reads only.

    python3 -I -B patch_presence.py <stage-root> <patch>...

<stage-root> is the folder holding explainer_corpus/. For each hunk of each patch, the hunk's
new side (context and added lines, in order) is looked for verbatim in the stage file. A hunk
not found is printed with the first new-side line that breaks the match, so a later patch that
rewrote those lines can be named by hand. Removed lines that are unique in the old side and
still present in the stage file are flagged as well.
"""
import hashlib
import pathlib
import re
import sys

ROOT = pathlib.Path(sys.argv[1])


def hunks(text):
    files, cur, h = [], None, None
    for line in text.splitlines():
        if line.startswith("+++ "):
            path = line[4:].split("\t")[0]
            path = path[2:] if path.startswith("b/") else path
            cur = {"path": path, "hunks": []}
            files.append(cur)
            h = None
        elif line.startswith("--- "):
            continue
        elif line.startswith("@@") and cur is not None:
            h = {"head": line, "new": [], "old": [], "added": [], "removed": []}
            cur["hunks"].append(h)
        elif h is not None and line[:1] in (" ", "+", "-"):
            body = line[1:]
            if line[0] in " +":
                h["new"].append(body)
            if line[0] in " -":
                h["old"].append(body)
            if line[0] == "+":
                h["added"].append(body)
            if line[0] == "-":
                h["removed"].append(body)
        elif h is not None and line == "":
            h["new"].append("")
            h["old"].append("")
        elif line.startswith("\\"):
            continue
    return files


total_bad = 0
for patch in sys.argv[2:]:
    p = pathlib.Path(patch)
    data = p.read_bytes()
    print(f"== {p.name} sha256 {hashlib.sha256(data).hexdigest()[:16]}")
    for f in hunks(data.decode("utf-8")):
        target = ROOT / f["path"]
        if not target.is_file():
            print(f"   {f['path']}: NOT IN STAGE")
            total_bad += 1
            continue
        text = target.read_text(encoding="utf-8")
        found = 0
        for h in f["hunks"]:
            block = "\n".join(h["new"])
            if block in text:
                found += 1
                continue
            # where does it break
            prefix = []
            brk = None
            for row in h["new"]:
                trial = "\n".join(prefix + [row])
                if trial in text:
                    prefix.append(row)
                else:
                    brk = row
                    break
            added_missing = [a for a in h["added"] if a.strip() and a not in text]
            print(f"   {f['path']} {h['head']}: new side NOT verbatim; breaks at {brk!r}; added lines missing {len(added_missing)}/{len([a for a in h['added'] if a.strip()])}")
            for a in added_missing[:4]:
                print(f"      missing + {a.strip()[:110]}")
            total_bad += 1
        stale = []
        for h in f["hunks"]:
            for r in h["removed"]:
                if len(r.strip()) > 25 and r not in "\n".join(h["new"]) and text.count(r) and "\n".join(h["old"]).count(r) == 1:
                    stale.append(r.strip()[:110])
        print(f"   {f['path']}: {found}/{len(f['hunks'])} hunks verbatim" + (f"; removed lines still present: {len(stale)}" if stale else ""))
        for s in stale[:4]:
            print(f"      still - {s}")
print("hunks not verbatim:", total_bad)
