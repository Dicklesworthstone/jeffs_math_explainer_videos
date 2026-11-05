"""The stage03 facts this review rests on, re-derived from the bytes. Reads only, except a fresh scratch folder.

    python3 -I -B stage03_checks.py <fresh-scratch-dir>

Run from the repo root. Standard library and the system `patch`. Each section prints PASS or FAIL
rows; a FAIL row is a failed premise of the review, not a finding. Nothing is compiled, rendered,
synthesized or heard.
"""
import hashlib
import json
import pathlib
import re
import shutil
import subprocess
import sys

REPO = pathlib.Path.cwd()
CL = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
ST = f"{CL}/combined-repair-stage03"
CR = f"{ST}/explainer_corpus/complex_analysis_visual"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
SCRATCH = pathlib.Path(sys.argv[1])
assert not SCRATCH.exists(), "the scratch folder must be fresh"
failures = []


def check(name, ok, detail=""):
    print(("PASS " if ok else "FAIL ") + name + (f": {detail}" if detail else ""))
    if not ok:
        failures.append(name)


def sha(rel):
    return hashlib.sha256((REPO / rel).read_bytes()).hexdigest()


def text(rel):
    return (REPO / rel).read_text(encoding="utf-8")


def film(rel):
    return re.split(r"#\[cfg\(test\)\]\s*mod tests", text(rel))[0]


def script(rel=f"{CR}/script.tsv"):
    return dict(l.split("\t", 1) for l in text(rel).splitlines() if "\t" in l and not l.startswith("#"))


def sentences(t):
    opens = [0] + [i + 2 for i in range(len(t) - 1) if t[i] in ".?!" and t[i + 1] == " "]
    return [t[o:(opens[k + 1] if k + 1 < len(opens) else len(t))].strip() for k, o in enumerate(opens)]


print("== A. the stage03 receipt still names these bytes")
receipt = json.loads(text(f"{CL}/combined-repair-stage03-integration01.json"))
print("   receipt sha256", sha(f"{CL}/combined-repair-stage03-integration01.json"))
hashed = {row["path"]: row["sha256"] for row in receipt["files"]}
print("   receipt changed:", receipt["changed"])
moved = [k for k, v in hashed.items() if not (REPO / ST / k).is_file() or sha(f"{ST}/{k}") != v]
check("every file the receipt hashes is still those bytes", hashed and not moved, f"{len(hashed)} files, moved {moved}")
for flag in ("native_build", "hold_rebind", "actual_picture_review", "listening"):
    found = re.findall(rf'"{flag}"\s*:\s*(true|false)', json.dumps(receipt))
    print(f"   receipt {flag}: {found}")

print("\n== B. stray files in the candidate tree (F2)")
p6 = json.loads(text(f"{CL}/combined-repair-stage02/integration-partial06.json"))
before = {k.rsplit("/", 1)[-1]: v for k, v in p6["before"].items()}
for name in ("counting.rs", "mod.rs", "one_derivative.rs"):
    rel = f"{CR}/src/chapters/{name}.orig"
    check(f"{name}.orig is partial06's before-bytes of {name}", (REPO / rel).is_file() and sha(rel) == before[name], sha(rel)[:16])
    check(f"partial06 applied {name} at an offset, which is when patch keeps a .orig", f"{name}\nHunk #" in p6["dry_run"])
hook = f"{CR}/.ee/hook-state/session_start_orient.last.json"
check(".ee/hook-state is in the crate", (REPO / hook).is_file(), sha(hook)[:16])
listed = [k for k in hashed if k.endswith(".orig") or "/.ee/" in k]
check("the receipt hashes all four as candidate files", len(listed) == 4, ", ".join(sorted(listed)))
mods = re.findall(r"^\s*(?:pub )?mod (\w+);", text(f"{CR}/src/chapters/mod.rs"), re.M)
check("no module is named after a .orig file", not any(m.endswith("orig") for m in mods))

print("\n== C. the 11_7 turning field and the plate's samples (F4)")
draw = film(f"{CR}/src/chapters/draw.rs")
cnt = film(f"{CR}/src/chapters/counting.rs")
check("PLATE is PLATE_CELLS4", "pub const PLATE: PlateStyle = PLATE_CELLS4;" in draw)
chain = {"PLATE_CELLS4": "PLATE_BLEND", "PLATE_BLEND": "PLATE_TILE", "PLATE_TILE": "PLATE_NOW"}
for child, parent in chain.items():
    body = re.search(rf"pub const {child}: PlateStyle = PlateStyle \{{(.*?)\}};", draw, re.S).group(1)
    check(f"{child} takes the rest of {parent} and sets no samples", f"..{parent}" in body and "samples" not in body)
now = re.search(r"pub const PLATE_NOW: PlateStyle = PlateStyle \{(.*?)\};", draw, re.S).group(1)
check("PLATE_NOW has samples: 1", re.search(r"samples:\s*1,", now) is not None)
check("portrait builds through plate_colors with the style's samples", "portrait_styled(stage, f, plane, enhance, &PLATE)" in draw and "plate_colors(|z| f.eval(z), plane, cols, rows, enhance, style.samples)" in draw)
check("plate_colors is portrait_colors at one sample", "if samples <= 1 {\n        return portrait_colors(f, plane, cols, rows, enhance);" in draw)
check("Turning builds with portrait and repaints with portrait_colors", "let strips = portrait(stage, Func::Quintic, &inp, 1.0)?;" in cnt and "portrait_colors(|z| Func::Quintic.eval(z).map(|w| by * w), &inp, cols, rows, 1.0)" in cnt)
tests = text(f"{CR}/src/chapters/counting.rs").split("#[cfg(test)]\nmod tests", 1)[1]
check("no counting test reads the plate's style or its samples (the gap F4 names)", not re.search(r"\bPLATE\b|\.samples\b|PlateStyle", tests))
check("turned_by(0) is exactly one, so the turn back repaints 11_6's colours", "C::polar(LEADING.abs().powf(t), LEADING.arg() * t)" in cnt)

print("\n== D. the widest pictures under PLATE_CELLS4 (F5)")
cell4 = 0.035 / 4
painted = {
    "gallery.rs trig_plane": ("gallery.rs", "pub fn trig_plane"),
    "gallery.rs gamma_plane": ("gallery.rs", "pub fn gamma_plane"),
    "continuing.rs gamma_plane": ("continuing.rs", "pub fn gamma_plane"),
    "reading_color.rs plane": ("reading_color.rs", "pub fn plane"),
}
widest = 0
for label, (name, fn) in painted.items():
    src = film(f"{CR}/src/chapters/{name}")
    body = src[src.index(fn):]
    w, x0, x1, y0, y1 = map(float, re.search(r"Plane::new\(\s*v\([^)]*\),\s*([0-9.]+),\s*\(([-0-9.]+),\s*([-0-9.]+)\),\s*\(([-0-9.]+),\s*([-0-9.]+)\)", body).groups())
    cols, rows = round(w / cell4), round(w * (y1 - y0) / (x1 - x0) / cell4)
    widest = max(widest, cols)
    print(f"   {label}: width {w}, {cols} cols × {rows} rows")
uses = {"gallery.rs": ["let trig = Painter::build(stage, Some(Func::Sin), tp, 1.0)?;", "let gamma = Painter::build(stage, Some(Func::Gamma), gp, 1.0)?;"],
        "continuing.rs": ["let gamma = Painter::build(stage, None, gp, 1.0)?;"]}
for name, calls in uses.items():
    src = film(f"{CR}/src/chapters/{name}")
    check(f"{name} paints those planes", all(c in src for c in calls))
test_src = text(f"{CR}/src/chapters/draw.rs")
check("draw.rs's budget test assumes a 6.4 × 6.4 widest picture with cols ≤ 800", "the widest picture the chapters draw: 6.4 × 6.4 scene units" in test_src and "rows <= 800 && cols <= 800" in test_src)
check("the widest painted picture has more than 800 cols", widest > 800, f"{widest}")
demand = text(f"{P}/engine-geometry-budget01/checks/demand.py")
check("the geometry budget did count the 8.4 and 7.5 plates", '("05 trig", [size(8.4' in demand and '("05 gamma", [size(8.4' in demand and '("continuing gamma", [size(7.5' in demand)

print("\n== E. the narration plan against the stage's script (F1)")
units_rel = f"{CR}/narration_units.json"
units = json.loads(text(units_rel))
sc = script()
print("   narration_units.json sha256", sha(units_rel)[:16], "script.tsv", sha(f"{CR}/script.tsv")[:16])
check("the plan pins a different script than the stage's (the gap)", units["script_sha256"] != sha(f"{CR}/script.tsv"), units["script_sha256"][:16])
stale = [u["id"] for u in units["units"] if " ".join(sc[l] for l in u["line_ids"]) != u["text"]]
check("only oliver_11_7 has other words than the script", stale == ["oliver_11_7"], ", ".join(stale))
old = [u for u in units["units"] if u["id"] == "oliver_11_7"][0]["text"]
check("its words are the six-sentence wording before root's wording repair", old.startswith("The same trick proves that every polynomial has roots. Take one of degree n") and len(sentences(old)) == 6)
patch = f"{P}/complex11-counting-teaching02/patches/narration-units.patch"
trial = SCRATCH / "units-trial" / "explainer_corpus/complex_analysis_visual"
trial.mkdir(parents=True)
shutil.copyfile(REPO / units_rel, trial / "narration_units.json")
r = subprocess.run(["patch", "-p1", "--no-backup-if-mismatch", "-s", "-i", str(REPO / patch)], cwd=SCRATCH / "units-trial", capture_output=True, text=True)
check("teaching02's narration-units.patch applies to the stage's plan", r.returncode == 0, (r.stdout + r.stderr).strip()[:200])
patched = json.loads((trial / "narration_units.json").read_text(encoding="utf-8"))
got = hashlib.sha256((trial / "narration_units.json").read_bytes()).hexdigest()
check("and gives the plan teaching02's handoff names", got.startswith("3989e1855f617734"), got[:16])
check("whose script hash is the stage's", patched["script_sha256"] == sha(f"{CR}/script.tsv") == patched["direction_contract"]["script_sha256"])
check("and every unit is the script's words", not [u["id"] for u in patched["units"] if " ".join(sc[l] for l in u["line_ids"]) != u["text"]])
print("   its map pointer:", patched["direction_contract"]["map"]["path"].rsplit("/", 1)[-1], patched["direction_contract"]["map"]["sha256"][:16])

print("\n== F. the retained 11_7 pair")
take_text = text(f"{CL}/counting-oliver02/11_7.txt").rstrip("\n")
check("the retained request text is the stage's 11_7, word for word", take_text == sc["11_7"])
dl = json.loads(text(f"{CL}/counting-oliver02/11_7-downloads01.json"))
flat = json.dumps(dl)
for key in ("listening", "fine_sync", "selection"):
    print(f"   downloads01 {key}:", re.findall(rf'"{key}"\s*:\s*([^,}}]+)', flat)[:2])
print("   sha256 11_7.txt", sha(f"{CL}/counting-oliver02/11_7.txt")[:16], "downloads01.json", sha(f"{CL}/counting-oliver02/11_7-downloads01.json")[:16])

print("\n== G. 15_2's last sentence and what a narrow rewording would touch (F3)")
s152 = sentences(sc["15_2"])
print("   15_2 sentence 5:", s152[5])
check("15_2 has six sentences", len(s152) == 6)
closing = film(f"{CR}/src/chapters/closing.rs")
block = closing[closing.index('let line = say(&kit, stage, "15_2")?;'):closing.index('let line = say(&kit, stage, "15_3")?;')]
hooks = re.findall(r"line\.(wait_sentence\(stage, \d\)|wait_for\(stage, \"[^\"]+\"\))", block)
print("   15_2 hooks:", hooks)
check("no 15_2 hook reads sentence 5's words", all("wait_sentence(stage, 5)" == h or "\"" not in h or h.split('"')[1] not in s152[5] for h in hooks))
proposed = s152[5].replace("though not always everywhere", "though there may be none")
check("the proposed wording changes only that clause", proposed != s152[5] and len(sentences(" ".join(s152[:5] + [proposed]))) == 6 and not re.search(r"\d", proposed), proposed)
check("14_7 is the film's case with no continuation at all", "Sometimes there is no way across at all." in sc["14_7"])

print("\n== H. storyboard picture cells that quote strings the film does not draw (F6)")
sb = text(f"{CR}/docs/storyboard.md")
for cue, quoted, const_file, const in (
    ("04_2", "band: size × 1.69", "reading_color.rs", 'pub const BAND: &str = "next band: size × ";'),
    ("04_6", "bump: where are the poles? how many turns?", "reading_color.rs", 'pub const ASK: [&str; 2] = ["where are the bump\'s poles?", "how many turns at each?"];'),
    ("05_7", "series converges to the right", "gallery.rs", 'pub const CONVERGES: &str = "series converges: Re s > 1";'),
):
    row = next(l for l in sb.splitlines() if l.startswith(f"| {cue} |"))
    src = film(f"{CR}/src/chapters/{const_file}")
    check(f"{cue}: the row quotes {quoted!r}, the film draws {const.split('= ', 1)[1]}", f'"{quoted}"' in row and const in src and quoted not in src)

print("\n== I. the hook scan's one problem row is the scanner's, not the film's")
calls = [m.start() for m in re.finditer(r"approach_pole\(&kit, stage, &line, inp, out\)", cnt)]
said_11_4 = cnt.index('let line = say(&kit, stage, "11_4")?;')
said_11_5 = cnt.index('let line = say(&kit, stage, "11_5")?;')
check("approach_pole is called once, inside 11_4's code", len(calls) == 1 and said_11_4 < calls[0] < said_11_5)
check("its hook is in 11_4 once, where a sentence starts", sc["11_4"].count("There the count") == 1 and "There the count has no meaning." in sentences(sc["11_4"]))
check("the scanner reads the helper's `line` as the file's last cue, 11_8", cnt.index("fn approach_pole(") > cnt.index('let line = say(&kit, stage, "11_8")?;'))

print("\nFAILURES: " + (", ".join(failures) if failures else "none"))
sys.exit(1 if failures else 0)
