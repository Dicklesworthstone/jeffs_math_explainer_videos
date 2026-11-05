"""Patch round trips, candidate equality, holds, timeline, strings, hooks and other patches' assumptions.

Run from the repo root:
  python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/complex13-14-cross-review01/checks/bindings.py

Writes only to a new temporary directory. Reads root's candidate; changes nothing.
"""
import glob
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

OPUS = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
STAGE = OPUS + "complex13-radius-repair01/"
CHAPTERS = "explainer_corpus/complex_analysis_visual/src/chapters/"
RI, CO = CHAPTERS + "real_integral.rs", CHAPTERS + "continuing.rs"
RI_S, CO_S = STAGE + RI, STAGE + "continuation14/" + CO
P13, P14 = STAGE + "complex13-radius-repair01.patch", STAGE + "continuation14/complex14-pole-order.patch"
ROOT = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/combined-repair-stage02/candidate/"
WANT = {"ri": "a5ef573dfa68325272a5d7f588fb910b2d99ea6538752616a51e92c3904d61ed", "co": "c24f264b98377aa323a714588d3e1d8b46201693592886d62138ef2a0e4ec96b",
        "ri_base": "906d4bd803b12031555ae79ff3062b0d65044aa275b5cd9f55507dddce2d82bb", "co_base": "088034785bc2588f9517d15218889f061d0b2de10fd64b363af50f1c7480b2be"}
fails = 0


def check(ok, what):
    global fails
    fails += 0 if ok else 1
    print(("PASS " if ok else "FAIL ") + what)


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def read(path):
    return open(path, encoding="utf-8").read()


print("== hashes")
check(sha(RI_S) == WANT["ri"] and sha(CO_S) == WANT["co"], "staged files are the sealed bytes")
check(sha(RI) == WANT["ri_base"] and sha(CO) == WANT["co_base"], "canonical is still the bases the patches name")
check(sha(ROOT + RI) == WANT["ri"] and sha(ROOT + CO) == WANT["co"], "root's combined-repair-stage02 candidate holds the same two files")

print("== round trips on temporary copies")
with tempfile.TemporaryDirectory(prefix="complex13-14-bind-") as work:
    for name, patch, want in ((RI, P13, "ri"), (CO, P14, "co")):
        os.makedirs(os.path.dirname(os.path.join(work, name)), exist_ok=True)
        shutil.copy(name, os.path.join(work, name))
        full = os.path.abspath(patch)
        ok = subprocess.run(["git", "apply", full], cwd=work, capture_output=True, text=True, check=False)
        check(ok.returncode == 0 and sha(os.path.join(work, name)) == WANT[want], f"{os.path.basename(patch)} applies and gives the staged bytes" + (" " + ok.stderr[:200] if ok.returncode else ""))
        back = subprocess.run(["git", "apply", "-R", full], cwd=work, capture_output=True, text=True, check=False)
        check(back.returncode == 0 and sha(os.path.join(work, name)) == WANT[want + "_base"], "and -R gives the base back")

print("== rustfmt, parse and format only")
for path in (RI_S, CO_S):
    fmt = subprocess.run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", path], capture_output=True, text=True, check=False)
    check(fmt.returncode == 0, f"rustfmt --check {os.path.basename(path)}")

print("== the hold rows, under root's rule: marker on the line, last say before = cue, first say after = next cue")
SAY = re.compile(r'\bsay\(&kit, stage, "(\d\d_\d+)"\)')
MAP = "explainer_corpus/complex_analysis_visual/docs/narration-direction-map-oliver10.json"
check(sha(MAP).startswith("341356cda34a"), "the map is oliver10 341356cda34a")
direction = json.load(open(MAP, encoding="utf-8"))
delta13 = json.load(open(STAGE + "bindings/visual-holds-delta.json", encoding="utf-8"))
delta14 = json.load(open(STAGE + "continuation14/bindings/visual-holds-delta.json", encoding="utf-8"))
CLAIMS = {("13_5", "real_integral.rs"): (593, 631), ("13_11", "real_integral.rs"): (775, 813), ("14_3", "continuing.rs"): (512, 522), ("14_4", "continuing.rs"): (522, 531)}
SOURCES = {"real_integral.rs": (read(RI), read(RI_S)), "continuing.rs": (read(CO), read(CO_S))}
rows = [h for h in direction["visual_holds"] if h["path"].endswith(("/real_integral.rs", "/continuing.rs"))]
check(len(rows) == 4, f"the map has {len(rows)} rows on these two files, the four the handoffs move")
for h in rows:
    name = os.path.basename(h["path"])
    old, new = CLAIMS[(h["cue"], name)] if "cue" in h else (None, None)
    base, staged = [s.split("\n") for s in SOURCES[name]]
    check(h["line"] == old and base[old - 1].strip() == h["marker"], f"{h['cue']}: the map's line {old} holds its marker in canonical")
    check(staged[new - 1].strip() == h["marker"], f"{h['cue']}: line {new} holds it in the staged file")
    before = SAY.findall("\n".join(staged[: new - 1]))
    after = SAY.findall("\n".join(staged[new:]))
    check(before[-1] == h["cue"], f"{h['cue']}: last say before is {before[-1]}")
    if after:
        check(after[0] == h["next_cue"], f"{h['cue']}: first say after is {after[0]}, the map's next cue")
    else:
        check(h["next_cue"] not in SAY.findall(SOURCES[name][1]) and h["next_cue"][:2] != h["cue"][:2], f"{h['cue']}: chapter-final, its next cue {h['next_cue']} opens the next chapter's file")
for delta, staged_sha in ((delta13, WANT["ri"]), (delta14, WANT["co"])):
    for row in delta["rows"]:
        old, new = CLAIMS[(row["cue"], os.path.basename(row["path"]))]
        check(row["old"]["line"] == old and row["new"]["line"] == new and row["new"]["sha256"] == staged_sha and row["new"]["marker"] == row["old"]["marker"], f"Bronze's delta: {row['cue']} {old} -> {new}, same marker, bound to the staged hash")
check(delta13["map_sha256"] == sha(MAP) and delta14["map_sha256"] == sha(MAP), "both deltas name this map")

print("== timed calls, film code only")
TIMED = re.compile(r"(fade_in_all\(stage, &[^;]*?, [\d.]+\)\?;|fade_out_all\(stage, &[^;]*?, [\d.]+\)\?;|shift_all\([^;]*\)\?;|stage\.wait\([^;]*\)\?;|line\.wait_\w+\([^;]*\)\?;|line\.until\([^;]*\)\?;|wait_until\([^;]*\)\?;|run_time: Some\([\d.]+\)|tween_linear\([^;]*\)\?)")


def film(text):
    return text[: text.index("#[cfg(test)]\nmod tests")]


def durations(block):
    out = []
    for hit in TIMED.findall(block):
        m = re.search(r", ([\d.]+)\)\?;$", hit)
        names = re.sub(r"\s+", " ", hit)
        out.append((re.split(r"[(&]", names)[0], m[1] if m else names))
    return out


def block(text, a, b):
    t = film(text)
    return t[t.index(a) : t.index(b, t.index(a))]


base13, new13 = block(read(RI), 'say(&kit, stage, "13_3")', 'say(&kit, stage, "13_4")'), block(read(RI_S), 'say(&kit, stage, "13_3")', 'say(&kit, stage, "13_4")')
base14, new14 = block(read(CO), 'say(&kit, stage, "14_3")', 'say(&kit, stage, "14_4")'), block(read(CO_S), 'say(&kit, stage, "14_3")', 'say(&kit, stage, "14_4")')
check(durations(base13) == durations(new13), f"13_3: {len(durations(new13))} timed calls, same kinds and durations in the same order: nothing shifts")
d_old, d_new = durations(base14), durations(new14)
missing = [x for x in d_old if x not in d_new]
extra = d_new.copy()
for x in d_old:
    if x in extra:
        extra.remove(x)
check(len(d_old) - len(d_new) == 1 and not extra and ("fade_in_all", "0.2") in d_old, f"14_3: {len(d_old)} -> {len(d_new)} timed calls; the one gone is a 0.2 s fade, the second mark's")
check("fade_in_all(stage, &pole_top, 0.2)?;" in base14 and "pole_top" not in new14, "that fade is the second mark, which is the only thing removed")
after_old = base14[base14.index("fade_in_all(stage, &pole_top, 0.2)?;") :]
check("line.wait_sentence(stage, 6)?;" in after_old[:200], "and the next timed call is the sentence hook: the card follows the voice, nothing later moves, and it can only arrive 0.2 s nearer if the chain was running late")
other = [h for h in range(1, 10) if f'say(&kit, stage, "14_{h}")' in film(read(CO))]
rest_old = film(read(CO)).replace(base14, "")
rest_new = film(read(CO_S)).replace(new14, "")
check(durations(rest_old) == durations(rest_new), "no other timed call moves in continuing.rs")
rest_old = film(read(RI)).replace(base13, "")
rest_new = film(read(RI_S)).replace(new13, "")
check(durations(rest_old) == durations(rest_new), "no other timed call moves in real_integral.rs")

print("== words")
STR = re.compile(r'"(?:[^"\\]|\\.)*"')
NOTE = re.compile(r"^\s*//[^\n]*$", re.M)  # the new doc comments quote "R" and "i"; they are not strings
for name, a, b in (("real_integral.rs", read(RI), read(RI_S)), ("continuing.rs", read(CO), read(CO_S))):
    sa, sb = sorted(STR.findall(NOTE.sub("", film(a)))), sorted(STR.findall(NOTE.sub("", film(b))))
    check(sa == sb, f"{name}: every string in the film code is unchanged ({len(sb)})")
    hook = re.compile(r'line\.(?:until|wait_for)\(stage, ("[^"]*")')
    check(hook.findall(film(a)) == hook.findall(film(b)), f"{name}: the same phrase hooks, in order ({len(hook.findall(film(b)))})")
check(sha("explainer_corpus/complex_analysis_visual/script.tsv").startswith("2968b0fd7654"), "script.tsv is the frozen 2968b0fd7654")

print("== other tests that read these two files")
t = read(CHAPTERS + "mod.rs")
for cue, nxt, card, staged in (("13_11", None, "note", read(RI_S)), ("14_3", "14_4", "exact", read(CO_S))):
    here = staged.index(f'let line = say(&kit, stage, "{cue}")?;')
    shown = here + staged[here:].index(f"let {card} = ")
    held = here + staged[here:].index("reading_window(&kit, stage, &line)?;")
    check(shown < held, f"{cue}: {card} comes before its reading window, as each_card_outlasts_its_reading_window needs")
    between = staged[shown:held]
    check("fade_out" not in between, f"{cue}: nothing fades {card} before the window")
check(read(CO_S).count("gammas_colour_goes_on_behind_the_cover") == 1, "gammas_colour_goes_on_behind_the_cover is unchanged in place")
for patch in ("complex-color-repair03/patches/B-plate-lab.patch", "complex-color-comparison01/patches/probe-plate-lab.patch"):
    path = OPUS + patch
    if os.path.exists(path):
        wanted = re.findall(r'continuing\.contains\("(Plane::new\(v\([^"]*)"\)', read(path))
        check(wanted and all(w in read(CO_S) for w in wanted), f"{os.path.basename(patch)}: its {len(wanted)} continuing.rs plane literal(s) are still in the staged file")

print("== other staged patches that touch either file (git apply --check on the staged bytes)")
others = []
for path in glob.glob(OPUS + "**/*.patch", recursive=True):
    if STAGE in path or "complex13-14-cross-review01" in path:
        continue
    head = "".join(re.findall(r"^(?:\+\+\+) [^\n]*\n", read(path), re.M))
    if RI in head or CO in head:
        others.append(path)
print(f"     {len(others)} patches name them")
with tempfile.TemporaryDirectory(prefix="complex13-14-bind-") as work:
    for name, staged in ((RI, RI_S), (CO, CO_S)):
        os.makedirs(os.path.dirname(os.path.join(work, name)), exist_ok=True)
        shutil.copy(staged, os.path.join(work, name))
    with tempfile.TemporaryDirectory(prefix="complex13-14-bind-") as base:
        for name in (RI, CO):
            os.makedirs(os.path.dirname(os.path.join(base, name)), exist_ok=True)
            shutil.copy(name, os.path.join(base, name))
        broken = 0
        for path in sorted(others):
            args = ["git", "apply", "--check", "--include=" + RI, "--include=" + CO, os.path.abspath(path)]
            on_base = subprocess.run(args, cwd=base, capture_output=True, text=True, check=False).returncode == 0
            on_stage = subprocess.run(args, cwd=work, capture_output=True, text=True, check=False).returncode == 0
            broken += on_base and not on_stage
            print(f"     canonical {'yes' if on_base else 'no '}  staged {'yes' if on_stage else 'no '}  {path[len(OPUS):]}")
        check(not broken, "no patch that still applies to canonical stops applying on the staged bytes; the 'no, no' rows no longer apply to canonical either")

print("RESULT " + ("PASS" if fails == 0 else f"FAIL {fails}"))
sys.exit(1 if fails else 0)
