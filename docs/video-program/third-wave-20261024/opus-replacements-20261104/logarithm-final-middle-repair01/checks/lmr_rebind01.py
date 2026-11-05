"""Scratch: mirror the binder's checks on the staged bytes; write the rebind table root needs.

Positive control: the recorded interval hashes must be reproduced from the baseline.
Negative control: the superseded draft of draw.rs must fail the binder's parse.
Run from the repo root. Writes one file, with mode "x".
Usage: lmr_rebind01.py BASE STAGED SUPERSEDED HOLD_AUDIT MAP WS STAGE09 OUT_JSON
"""
import hashlib
import json
import os
import re
import sys

BASE, STAGED, DRAFT, AUDIT, MAP, WS, STAGE09, OUT = sys.argv[1:9]
CH = "src/chapters"
FILES = ["draw.rs", "complex_turns.rs", "lifting_loops.rs"]


def sha(b):
    return hashlib.sha256(b).hexdigest()


def read(root, f):
    return open(os.path.join(root, CH, f), "rb").read()


def interval(lines, say, marker):
    return sha("\n".join(lines[say - 1:marker]).encode("utf-8"))


def binder_parse(text):
    """bind_final80.py lines 306-315, as written there."""
    m = re.search(r"^pub fn predict\(.*?^}", text, re.M | re.S)
    if not m:
        return "predict not found"
    body = m.group(0)
    order = ["kit.hold(stage)?;", 'kit.mark(stage, "predict");', "kit.music_breath(stage, THINK)?;",
             "fade_out_all(stage, &[bar, probe], "]
    places = [body.find(x) for x in order]
    if -1 in places or places != sorted(places):
        return "order broken: %s" % places
    think = re.search(r"^pub const THINK: f64 = ([0-9.]+);", text, re.M)
    fade = re.search(r"fade_out_all\(stage, &\[bar, probe\], ([0-9.]+)\)", body)
    if not think or not fade:
        return "constant missing"
    return "ok, think %s, fade %s" % (think.group(1), fade.group(1))


print("== binder parse of draw.rs")
print("baseline  :", binder_parse(read(BASE, "draw.rs").decode("utf-8")))
print("staged    :", binder_parse(read(STAGED, "draw.rs").decode("utf-8")))
print("draft (negative control):", binder_parse(read(DRAFT, "draw.rs").decode("utf-8")))

audit = json.load(open(AUDIT, encoding="utf-8"))
print("== hold-audit helper rows for draw.rs, in the staged bytes")
staged_draw = read(STAGED, "draw.rs").decode("utf-8").split("\n")
base_draw = read(BASE, "draw.rs").decode("utf-8").split("\n")
for row in audit["helpers"]:
    if row["path"].endswith("draw.rs"):
        line, marker = row["line"], row["marker"]
        print("  line %d: base %s, staged %s  %s" % (
            line, marker in base_draw[line - 1], marker in staged_draw[line - 1], marker[:50]))

print("== line moves")
table = {"files": {}, "holds": []}
for f in FILES:
    a = read(BASE, f).decode("utf-8").split("\n")
    b = read(STAGED, f).decode("utf-8").split("\n")
    same = len(a) == len(b)
    if same:
        changed = [i + 1 for i in range(len(a)) if a[i] != b[i]]
    else:
        k = 0
        while a[k] == b[k]:
            k += 1
        changed = "first difference at line %d" % (k + 1)
    print("  %-18s lines %d to %d; %s" % (f, len(a), len(b), changed))
    table["files"][f] = {"sha256_base": sha(read(BASE, f)), "sha256_staged": sha(read(STAGED, f)),
                         "lines_base": len(a), "lines_staged": len(b), "changed_lines": changed}

print("== holds in the changed chapter files")
holds = [r for r in audit["prediction_holds"] + audit["reading_holds"] if r["file"] in FILES]
for r in holds:
    f = r["file"]
    a = read(BASE, f).decode("utf-8").split("\n")
    b = read(STAGED, f).decode("utf-8").split("\n")
    say, mark, nxt = r["say_line"], r["marker_line"], r["next_say_line"]
    control = interval(a, say, mark) == r["interval_sha256"]
    assert control, r["cue"]
    assert r["source_sha256"] == sha(read(BASE, f)), r["cue"]
    assert '"%s"' % r["cue"] in b[say - 1] and '"%s"' % r["next_cue"] in b[nxt - 1], r["cue"]
    marker_new = b[mark - 1].strip()
    new_interval = interval(b, say, mark)
    row = {"cue": r["cue"], "file": f, "say_line": say, "marker_line": mark, "next_say_line": nxt,
           "marker_old": r["marker"], "marker_new": marker_new,
           "interval_old": r["interval_sha256"], "interval_new": new_interval,
           "interval_changed": new_interval != r["interval_sha256"],
           "baseline_reproduces_recorded_interval": control}
    table["holds"].append(row)
    print("  %s %s:%d interval reproduced from baseline: %s; changed: %s; marker now %s" % (
        r["cue"], f, mark, control, row["interval_changed"], marker_new))

print("== map rows")
m = json.load(open(MAP, encoding="utf-8"))
for row in m["visual_holds"]:
    if os.path.basename(row["path"]) in FILES:
        h = [x for x in table["holds"] if x["cue"] == row["cue"]][0]
        print("  map %s line %d equal to audit: %s; interval equal: %s" % (
            row["cue"], row["line"], row["line"] == h["marker_line"], row["interval_sha256"] == h["interval_old"]))

print("== where the three old hashes are named")
olds = {f: sha(read(BASE, f)) for f in FILES}
hits = {}
for root in (WS, STAGE09):
    for dirpath, _, names in os.walk(root):
        for n in names:
            p = os.path.join(dirpath, n)
            if os.path.getsize(p) > 8 * 1024 * 1024:
                continue
            try:
                s = open(p, encoding="utf-8").read()
            except (UnicodeDecodeError, OSError):
                continue
            for f, h in olds.items():
                c = s.count(h)
                if c:
                    key = ("WS" if root == WS else "stage09") + ":" + os.path.relpath(p, root)
                    hits.setdefault(key, {})[f] = c
for k in sorted(hits):
    print("  ", k, hits[k])
table["old_hash_named_in"] = hits

with open(OUT, "x", encoding="utf-8") as fh:
    json.dump(table, fh, indent=1, ensure_ascii=False)
    fh.write("\n")
print("wrote", os.path.basename(OUT))
