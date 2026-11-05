"""Patch round trip, the hold bindings and every source pin these edits could disturb. Writes source-markers.json only.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-timing-repair01/checks/source_checks.py

The interval hash and the census follow the root's binder, combined-repair-stage06/bind_final80-stage06.py: scene code
is the text before #[cfg(test)], comment lines are skipped, and interval = lines[say_line - 1 : hold_line] joined by
newlines, sha256 of the UTF-8 bytes. Reproducing map13's stored values on the baseline is the control for that reading.
The binder itself is read, not run.
"""
import hashlib
import json
import re
import subprocess
import sys
import tempfile
from pathlib import Path

O = Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-opening-timing-repair01")
CRATE = "explainer_corpus/logarithm_mysteries"
CH = f"{CRATE}/src/chapters"
FILES = ("ratio_ruler.rs", "folded_decade.rs", "slide_rule.rs")
MAP = f"{CRATE}/docs/narration-direction-map-oliver13.json"
SCRIPT_SHA = "f201d2dc57fa9ccac30d475255690a90d6749449014b542d4dea62164f31bf84"
SAY = re.compile(r'\bkit\.say\(stage, "(\d\d_\d+)"\)')
TIMING = re.compile(r"\b(predict_pause|predict|kit\.hold|kit\.music_breath|kit\.listen|kit\.sonify|kit\.mark|kit\.finish|stage\.wait|wait_frames|wait_until)\(")
HOOKS = [
    r'kit\.until_sentence\(stage, "(\d\d_\d+)", (\d+)\)',
    r'sentence_time\(&kit, "(\d\d_\d+)", (\d+)\)',
    r'\.sentence_at\("(\d\d_\d+)", (\d+)\)',
    r'sentence_len\(&kit\.sentence_starts\("(\d\d_\d+)"\), (\d+)\)',
    r'room\(&kit\.sentence_starts\("(\d\d_\d+)"\), (\d+),',
    r'until_share\(&kit, stage, "(\d\d_\d+)", (\d+),',
]
PHRASE = re.compile(r"until_phrase\(|phrase_time\(|phrase_marks\(")
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(data):
    return hashlib.sha256(data).hexdigest()


def census(data):
    text = data.decode("utf-8")
    scene = text[: text.find("#[cfg(test)]")]
    says, sites, hooks = {}, [], []
    current = None
    for n, line in enumerate(scene.split("\n"), 1):
        if line.strip().startswith("//"):
            continue
        m = SAY.search(line)
        if m:
            current = m.group(1)
            says[current] = n
        t = TIMING.search(line)
        if t and not re.search(r"\bfn (predict|wait_until)\(", line):
            sites.append((n, t.group(1), line.strip(), current))
        for pattern in HOOKS:
            for cue, j in re.findall(pattern, line):
                hooks.append((n, cue, int(j), current))
    return text.split("\n"), scene, says, sites, hooks


base = {f: Path(CH, f).read_bytes() for f in FILES}
new = {f: Path(O, CH, f).read_bytes() for f in FILES}
info = {f: (census(base[f]), census(new[f])) for f in FILES}

print("== script and neighbours")
check(sha(Path(f"{CRATE}/script.tsv").read_bytes()) == SCRIPT_SHA, f"script.tsv {SCRIPT_SHA[:16]}: not touched, every take's text stands")
tree = sorted(p.relative_to(O).as_posix() for p in (O / "explainer_corpus").rglob("*") if p.is_file())
check(tree == [f"{CH}/{f}" for f in sorted(FILES)], f"the candidate tree holds exactly these: {[t.rsplit('/', 1)[1] for t in tree]}")

print("\n== each cue's say, and the order of all says")
for f in FILES:
    (_, _, bs, _, _), (_, _, ns, _, _) = info[f]
    check(list(bs) == list(ns), f"{f}: says in the same order: {list(ns)}")
    moved = {c: (bs[c], ns[c]) for c in bs if bs[c] != ns[c]}
    print(f"  say lines moved: {moved}")

print("\n== ratio_ruler.rs: what stays on its line")
(bl, bscene, bs, bsites, bhooks), (nl, nscene, ns, nsites, nhooks) = info["ratio_ruler.rs"]
scene_end = bl.index("#[cfg(test)]")
same_from = 213
check(bl[same_from - 1 : scene_end - 1 - 1] == nl[same_from - 1 : scene_end - 1 - 1], f"lines {same_from} to {scene_end - 2} are byte-identical and keep their numbers")
check(bl[212] == nl[212] == '        kit.until_sentence(stage, "00_2", 1)?;', "line 213 is still the first hook of 00_2")
for line, marker in ((371, "predict(&kit, stage, v(3.6, -2.75), 3.0)?;"), (509, None)):
    if marker:
        check(nl[line - 1].strip() == marker, f"line {line}: {marker}")
print("timing sites, baseline:", [(s[0], s[2], s[3]) for s in bsites])
print("timing sites, candidate:", [(s[0], s[2], s[3]) for s in nsites])
old_keys = {s[0] for s in bsites if s[1] != "predict" and s[1] != "kit.finish"}
new_keys = {s[0] for s in nsites if s[1] != "predict" and s[1] != "kit.finish"}
print("binder SITES keys, baseline:", sorted(old_keys), "candidate:", sorted(new_keys))
check(old_keys == {181, 509}, "control: the baseline's non-predict waits are the binder's two rows, 181 and 509")
wait_dots = next(s for s in nsites if s[2] == "stage.wait(0.18)?;")
wait_read = next(s for s in nsites if s[2] == "super::draw::wait_until(stage, at)?;")
check(509 in new_keys and new_keys == {509, wait_dots[0], wait_read[0]}, f"candidate's are 509 (unchanged), {wait_dots[0]} (the dots) and {wait_read[0]} (new)")
check(nscene.count("phrase_marks(") == 0, "ratio_ruler.rs still has no phrase_marks use, as the binder pins")

print("\n== folded_decade.rs and slide_rule.rs")
(fbl, _, _, fbs, _), (fnl, fnscene, _, fns, _) = info["folded_decade.rs"]
check([s[1:3] for s in fbs] == [s[1:3] for s in fns] and fns == [s for s in fns if s[1] in ("kit.finish",)] or [s[1:3] for s in fbs] == [s[1:3] for s in fns], f"folded_decade.rs: no timing site added: {[s[1:3] for s in fns]}")
check(not PHRASE.search(fnscene), "folded_decade.rs: no phrase hook")
(sbl, sbscene, sbs, sbsites, _), (snl, snscene, sns, snsites, _) = info["slide_rule.rs"]
check(sbsites == snsites, f"slide_rule.rs: timing sites identical: {[s[0] for s in snsites]}")
check(sbs == sns, "slide_rule.rs: all says on the same lines")
check(snl[101].strip() == sbl[101].strip() == "wait_frames(stage, frames_covering(at - now, fps))?;", f"binder SITES row (slide_rule.rs, 102) still holds: {snl[101].strip()}")
changed = [i + 1 for i in range(sbl.index("#[cfg(test)]")) if sbl[i] != snl[i]]
print("slide_rule.rs scene lines changed:", changed)
check(len(changed) == 3 and sbl.index("#[cfg(test)]") == snl.index("#[cfg(test)]"), "three scene lines, none moved")
anchors = lambda lines: [(i + 1, l.strip()) for i, l in enumerate(lines[: lines.index("#[cfg(test)]")]) if PHRASE.search(l)]
check(anchors(sbl) == anchors(snl), f"slide_rule.rs: phrase hooks on the same lines: {[a[0] for a in anchors(snl)]}")

print("\n== sentence hooks the binder counts")
total = 0
for f in FILES:
    (_, _, _, _, bh), (_, _, _, _, nh) = info[f]
    extra = len(nh) - len(bh)
    total += extra
    print(f"{f}: {len(bh)} to {len(nh)}; new off the current cue: {[h for h in nh if h[1] != h[3] and h not in bh]}")
print("literal sentence hooks, net change:", total)
check(total == 3, "three more literal sentence hooks: one in read_out, two in 03_3")
check(sorted((h[1], h[2]) for h in info["folded_decade.rs"][1][4] if h[3] == "03_3" and h[2] in (0, 1)) == [("03_3", 0), ("03_3", 1), ("03_3", 1)], "03_3's hooks sit on their own cue")

print("\n== hold rows in map13")
the_map = json.loads(Path(MAP).read_bytes())
rows = {}
for f in FILES:
    rel = f"{CH}/{f}"
    rows[f] = [r for r in the_map["visual_holds"] if r["path"] == rel]
check(not rows["folded_decade.rs"], "no hold binds folded_decade.rs")
check([r["cue"] for r in rows["ratio_ruler.rs"]] == ["00_4"] and [r["cue"] for r in rows["slide_rule.rs"]] == ["02_2"], "one hold each binds ratio_ruler.rs (00_4) and slide_rule.rs (02_2)")
helpers = {h["path"] for h in the_map["shared_helpers"]}
check(not helpers & {f"{CH}/{f}" for f in FILES}, "none of the three is a shared helper")
deltas = []
for f in ("ratio_ruler.rs", "slide_rule.rs"):
    row = rows[f][0]
    (bl, _, bs, _, _), (nl, _, ns, _, _) = info[f]
    check(row["sha256"] == sha(base[f]), f"control: {row['cue']} row's source hash is the baseline")
    before = sha("\n".join(bl[bs[row["cue"]] - 1 : row["line"]]).encode("utf-8"))
    check(before == row["interval_sha256"], f"control: my interval hash reproduces map13's for {row['cue']}")
    new_line = next(i + 1 for i, l in enumerate(nl) if l.strip() == row["marker"])
    after = sha("\n".join(nl[ns[row["cue"]] - 1 : new_line]).encode("utf-8"))
    print(f"{row['cue']} -> {row['next_cue']} {f}")
    print(f"  sha256 and evidence.sha256: {row['sha256']} -> {sha(new[f])}")
    print(f"  line: {row['line']} -> {new_line}")
    print(f"  marker: {row['marker']}")
    print(f"  interval_sha256: {row['interval_sha256']} -> {after}" + (" (unchanged)" if after == row["interval_sha256"] else " (CHANGED)"))
    deltas.append({
        "cue": row["cue"],
        "next_cue": row["next_cue"],
        "path": f"{CH}/{f}",
        "sha256": {"old": row["sha256"], "new": sha(new[f])},
        "evidence_sha256": {"old": row["evidence"]["sha256"], "new": sha(new[f])},
        "line": {"old": row["line"], "new": new_line},
        "marker": {"old": row["marker"], "new": row["marker"]},
        "interval_sha256": {"old": row["interval_sha256"], "new": after},
    })
check(deltas[0]["interval_sha256"]["old"] == deltas[0]["interval_sha256"]["new"] and deltas[0]["line"]["new"] == 371, "00_4: line 371 and its interval hash survive")
check(deltas[1]["interval_sha256"]["old"] != deltas[1]["interval_sha256"]["new"] and deltas[1]["line"]["new"] == 204, "02_2: line 204 survives, its interval hash changes because two edits lie inside it")

print("\n== other pins of the old hashes in canonical's crate docs (not changed here)")
for f in FILES:
    needle = sha(base[f]).encode()
    for p in sorted(Path(f"{CRATE}/docs").rglob("*")):
        if p.is_file() and needle in p.read_bytes():
            print(f, p.as_posix(), p.read_bytes().count(needle))

print("\n== patch round trip on a scratch copy")
patch = (O / "opening-timing.patch").resolve()
with tempfile.TemporaryDirectory() as tmp:
    for f in FILES:
        dest = Path(tmp) / CH / f
        dest.parent.mkdir(parents=True, exist_ok=True)
        dest.write_bytes(base[f])
    subprocess.run(["git", "init", "-q", tmp], check=True)
    done = subprocess.run(["git", "apply", "--check", str(patch)], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0, "git apply --check on the baseline")
    done = subprocess.run(["git", "apply", str(patch)], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0 and all((Path(tmp) / CH / f).read_bytes() == new[f] for f in FILES), "applying it gives the three candidates' bytes")
    done = subprocess.run(["git", "apply", "-R", str(patch)], cwd=tmp, capture_output=True, text=True)
    check(done.returncode == 0 and all((Path(tmp) / CH / f).read_bytes() == base[f] for f in FILES), "reversing it gives the baseline's bytes")

print("\n== format")
for f in FILES:
    done = subprocess.run(["rustfmt", "--edition", "2024", "--check", "--config", "max_width=120", str(O / CH / f)], capture_output=True, text=True)
    check(done.returncode == 0 and not done.stdout.strip(), f"rustfmt --check {f}, edition 2024, width 120")

markers = {
    "topic": "logarithm_mysteries",
    "sources": [
        {
            "path": f"{CH}/{f}",
            "baseline_sha256": sha(base[f]),
            "patched_sha256": sha(new[f]),
            "line_count": {"baseline": len(info[f][0][0]) - 1, "patched": len(info[f][1][0]) - 1},
        }
        for f in FILES
    ],
    "map13_hold_delta": deltas,
    "binder_sites_delta": [
        {"key": ["ratio_ruler.rs", 181], "becomes": ["ratio_ruler.rs", wait_dots[0]], "class": "picture_pacing", "note": "unchanged: paces the note dots, 0.18 s each; now inside 00_2"},
        {"key": None, "becomes": ["ratio_ruler.rs", wait_read[0]], "class": "word_sync_helper", "note": "new: inside read_out, waits to a share of a sentence of 00_1, inside the voice. The census lists it under 00_6 because the helper follows preview()."},
        {"key": ["ratio_ruler.rs", 509], "becomes": ["ratio_ruler.rs", 509], "class": "helper_tail", "note": "unchanged"},
    ],
    "phrase_anchor_delta": [],
    "cue_text_delta": [],
}
(O / "source-markers.json").write_text(json.dumps(markers, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
print("\nRESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
