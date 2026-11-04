"""Root's pulse hunk, its six map rows, the 13 call sites, and the proposed test patch: fidelity and type-checks.

Usage from the repo root:  python3 -I -B pulse_check.py SCRATCH_DIR
Everything is built in SCRATCH_DIR. Rust runs as `rustc --emit=metadata` and `clippy-driver` against the existing rmeta
in target/debug/deps (fmn pin a399d62) and fresh score and Kit metadata from the current tree. There is no Cargo, no
codegen and no render, so **the proposed test is type-checked here and never executed**. Nothing is written in the repo.
"""
import difflib
import glob
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

ROOT = os.getcwd()
R = "narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/permutations-repair-stage01/"
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
O = P + "algebra11-pulse-cross-review01/"
CRATE = "explainer_corpus/abstract_algebra_expanded/"
DEPS = ROOT + "/target/debug/deps"
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
fails = 0
EXPECT = {
    "candidate/permutations.rs": "4f861c3b6704dfd89774909f88168bc7252a66cc54a4c5d62f1f547060c28473",
    "candidate02/permutations.rs": "0ba22ebd8903fe8c856776398b836cd50397ee44840518f6c28ef146aebfb958",
    "candidate/narration-direction-map-oliver25.json": "1cd112090153321b11244fbd895fe179cdafbcf058291dc3a06cfc383caa3a48",
    "candidate02/narration-direction-map-oliver25.json": "129b3ef924a7d3546c2040531733415a54362d7c25a13257c6c79160f1f8bf62",
}


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(text):
    return text.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(S, "<scratch>")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def run(args, cwd=ROOT, **kw):
    return subprocess.run(args, capture_output=True, text=True, cwd=cwd, **kw)


def fresh(name):
    path = os.path.join(S, name)
    if os.path.isdir(path):
        shutil.move(path, path + f"-old-{len(glob.glob(path + '-old-*'))}")
    os.makedirs(path)
    return path


def read(path):
    return open(path, encoding="utf-8").read()


print("--- A. root's files and the hunk")
for name, want in EXPECT.items():
    ck(sha(R + name) == want, f"{name} is the bound file", want[:12])
ck(sha(R + "baseline/permutations.rs") == sha(CRATE + "src/permutations.rs"), "baseline permutations.rs equals canonical")
ck(sha(R + "baseline/script.tsv") == sha(CRATE + "script.tsv"), "baseline script.tsv equals canonical")

work = fresh("root-patch")
shutil.copyfile(R + "candidate/permutations.rs", work + "/permutations.rs")
r = run(["patch", "-p1", "--fuzz=0", "-i", os.path.abspath(R + "pulse-color-preservation.patch")], cwd=work)
ck(r.returncode == 0 and "offset" not in r.stdout.lower(), "root's patch applies to candidate at fuzz 0, with no offset", rel(r.stdout.strip()))
ck(sha(work + "/permutations.rs") == EXPECT["candidate02/permutations.rs"], "patched candidate has candidate02's hash")
r = run(["patch", "-R", "-p1", "--fuzz=0", "-i", os.path.abspath(R + "pulse-color-preservation.patch")], cwd=work)
ck(r.returncode == 0 and sha(work + "/permutations.rs") == EXPECT["candidate/permutations.rs"], "and it reverses to candidate")

before = read(R + "candidate/permutations.rs").split("\n")
after = read(R + "candidate02/permutations.rs").split("\n")
ops = [op for op in difflib.SequenceMatcher(None, before, after, autojunk=False).get_opcodes() if op[0] != "equal"]
ck(len(ops) == 1 and ops[0][0] == "replace", "the two files differ in one contiguous place", str(ops))
tag, i1, i2, j1, j2 = ops[0]
print("removed:", [line.strip() for line in before[i1:i2]])
print("added:")
for line in after[j1:j2]:
    print("   ", line)
ck((j2 - j1) - (i2 - i1) == 9, "net +9 lines", f"{j2 - j1} for {i2 - i1}")
fn_start = max(n for n, line in enumerate(after[: j1 + 1]) if line.startswith("fn "))
ck(after[fn_start].startswith("fn pulse("), "the hunk sits in fn pulse", f"line {fn_start + 1}")
src = "\n".join(after)
for word, label in (("indicate", "indicate"), ("yellow", "yellow"), ("add_updater", "add_updater")):
    ck(word not in src, f"candidate02 has no `{label}`")

print("--- B. the map rows")
map0 = json.loads(read(R + "candidate/narration-direction-map-oliver25.json"))
map2 = json.loads(read(R + "candidate02/narration-direction-map-oliver25.json"))
rows0, rows2 = map0["visual_holds"], map2["visual_holds"]
ck(len(rows0) == len(rows2), "same number of visual holds", str(len(rows2)))
changed, marker_ok, cue_ok = [], True, True
for a, b in zip(rows0, rows2):
    if a != b:
        changed.append((b["cue"], a["line"], b["line"]))
    if "permutations.rs" in b.get("path", ""):
        marker_ok &= after[b["line"] - 1].strip() == b["marker"]
        said = [line for line in after[: b["line"]] if re.search(r'\bsay\(', line) or 'say(stage, "' in line]
        cue_ok &= bool(said) and f'"{b["cue"]}"' in said[-1]
        if b["sha256"] != EXPECT["candidate02/permutations.rs"]:
            changed.append(("wrong sha", b["cue"], b["sha256"][:12]))
print("changed rows (cue, candidate line, candidate02 line):", changed)
ck(len(changed) == 6 and all(c[2] - c[1] == 9 for c in changed), "exactly six rows changed, each by +9, each naming candidate02's hash")
others = {k: v for k, v in map2.items() if k != "visual_holds"}
ck(others == {k: v for k, v in map0.items() if k != "visual_holds"}, "nothing else in the map changed")
ck(marker_ok, "every permutations row's line holds its marker in candidate02")
ck(cue_ok, "the nearest say above every permutations row is that row's cue")
units = read(R + "candidate02/narration_units.json")
ck(EXPECT["candidate02/narration-direction-map-oliver25.json"] in units, "candidate02's units name candidate02's map hash")
ck(EXPECT["candidate/narration-direction-map-oliver25.json"] not in units, "and no longer name the prior map")

print("--- C. the call sites")
sites = []
cue = sentence = fn_name = None
for n, line in enumerate(after, 1):
    m = re.match(r"\s*(?:pub(?:\(\w+\))? )?fn (\w+)", line)
    if m:
        fn_name = m.group(1)
    m = re.search(r'\bsay\(stage, "(\d+_\d+)"', line)
    if m:
        cue, sentence = m.group(1), None
    m = re.search(r'until_sentence\(stage, "(\d+_\d+)", (\d+)\)', line)
    if m:
        cue, sentence = m.group(1), m.group(2)
    if re.search(r"\bpulse\(stage,", line) and not line.startswith("fn "):
        sites.append((n, fn_name, cue, sentence, line.strip()))
script = {row.split("\t")[0]: row.split("\t", 1)[1] for row in read(CRATE + "script.tsv").split("\n") if re.match(r"11_\d+\t", row)}
ck(len(script) >= 25, "chapter 11 script rows read", str(len(script)))


def sentences(text):
    """The Kit's own split (explainer_kit narration.rs): after . ? or ! followed by a space or the end."""
    return [part.strip() for part in re.findall(r".+?[.?!](?= |$)|.+$", text) if part.strip()]


for site in sites:
    spoken = sentences(script[site[2]])[int(site[3])] if site[3] is not None else "(no anchor in the cue)"
    print(f"{site[0]:>5} {site[1]:<22} {site[2]} sentence index {site[3]}: {site[4]}\n        voice from there: {spoken[:150]}")
ck(len(sites) == 13, "13 call sites", str(len(sites)))
colour_words = re.compile(r"\b(yellow|flash\w*|highlight\w*|lit|lights? up|glow\w*)\b", re.I)
ck(not [cue for cue, text in script.items() if colour_words.search(text)], "no chapter 11 line says yellow, flash, highlight, lit, lights up or glow")
r = run(["rustfmt", "--edition", "2024", "--config", "max_width=120", "--color", "never", "--check", R + "candidate02/permutations.rs"])
ck(r.returncode == 0, "rustfmt --check on candidate02 (max_width 120)", rel(r.stdout)[:800] if r.returncode else "")

print("--- D. the test patch")
patch = O + "patches/pulse-regression-test.patch"
work = fresh("test-patch")
shutil.copyfile(R + "candidate02/permutations.rs", work + "/permutations.rs")
r = run(["patch", "-p1", "--fuzz=0", "-i", os.path.abspath(patch)], cwd=work)
ck(r.returncode == 0 and "offset" not in r.stdout.lower(), "the test patch applies to candidate02 at fuzz 0, with no offset", rel(r.stdout.strip()))
patched = work + "/permutations.rs"
new_hash = sha(patched)
print("patched permutations.rs sha256", new_hash)
print("patch sha256", sha(patch))
new_lines = read(patched).split("\n")
if new_hash == EXPECT["candidate02/permutations.rs"] or "mod pulse_tests" not in read(patched):
    print("RESULT stopped: the test module is not in the patched file, so nothing below would test it")
    sys.exit(1)
ck(len(new_lines) > len(after) and new_lines[: len(after) - 1] == after[:-1], "only appends: every candidate02 line keeps its number", f"{len(after) - 1} then {len(new_lines) - 1}")
ck(all(new_lines[b["line"] - 1].strip() == b["marker"] for b in rows2 if "permutations.rs" in b.get("path", "")), "every permutations row's line still holds its marker")
print("rows whose sha256 would follow:", sum("permutations.rs" in b.get("path", "") for b in rows2))
r = run(["rustfmt", "--edition", "2024", "--config", "max_width=120", "--color", "never", "--check", patched])
ck(r.returncode == 0, "rustfmt --check on the patched file (max_width 120)", rel(r.stdout)[:1500] if r.returncode else "")


def one(pattern):
    hits = [h for h in glob.glob(f"{DEPS}/lib{pattern}-*.rmeta") if os.path.getsize(h) > 0]
    assert len(hits) == 1, (pattern, len(hits))
    return hits[0]


def metadata(driver, args, label, expect_ok=True, show=6000):
    r = run([driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + args,
            env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": ROOT})
    ck((r.returncode == 0) == expect_ok, label)
    if expect_ok and r.returncode:
        print(rel(r.stderr)[-show:])
    return r


print("--- E. type-check, on a scratch copy of the crate with the patched file")
fmn = one("fmn")
dep_info = read(fmn.replace("/libfmn-", "/fmn-").replace(".rmeta", ".d"))
ck("franken_manim-6641d0e09642f63e/a399d62/crates/fmn/src/lib.rs" in dep_info, "fmn metadata was built from the a399d62 checkout")
score = os.path.join(S, "libexplainer_score.rmeta")
metadata("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_score", "-A", "warnings", "-o", score, ROOT + "/crates/explainer_score/src/lib.rs"], "explainer_score metadata from the current tree")
kit = os.path.join(S, "libexplainer_kit.rmeta")
metadata("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_kit", "-A", "warnings", "--extern", f"fmn={fmn}", "--extern", f"fmn_output={one('fmn_output')}",
                   "--extern", f"fmn_codec={one('fmn_codec')}", "--extern", f"serde_json={one('serde_json')}", "--extern", f"explainer_score={score}", "-o", kit,
                   ROOT + "/crates/explainer_kit/src/lib.rs"], "explainer_kit metadata from the current tree")
EXT = ["--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}", "--extern", f"explainer_score={score}"]


def crate_with(name, text):
    base = fresh(name)
    shutil.copytree(CRATE + "src", base + "/src")
    shutil.copyfile(CRATE + "script.tsv", base + "/script.tsv")
    with open(base + "/src/permutations.rs", "w", encoding="utf-8") as out:
        out.write(text)
    return base + "/src/main.rs", base


patched_text = read(patched)
main_path, base = crate_with("crate-patched", patched_text)
metadata("rustc", EXT + ["-D", "warnings", "--crate-name", "alg", "-o", base + "/a.rmeta", main_path], "rustc -D warnings, normal cfg")
metadata("rustc", EXT + ["-D", "warnings", "--test", "-A", "dead_code", "--crate-name", "alg_t", "-o", base + "/b.rmeta", main_path], "rustc -D warnings, test cfg (the new module compiles)")
metadata("clippy-driver", EXT + ["-D", "warnings", "--crate-name", "alg_c", "-o", base + "/c.rmeta", main_path], "clippy-driver -D warnings, normal cfg")
metadata("clippy-driver", EXT + ["-D", "warnings", "-A", "dead_code", "--test", "--crate-name", "alg_ct", "-o", base + "/d.rmeta", main_path], "clippy-driver -D warnings, test cfg")

print("--- E2. positive controls: the type-check really reads the module, in test cfg only")
module_at = patched_text.index("mod pulse_tests {\n") + len("mod pulse_tests {\n")
broken = patched_text[:module_at] + '    const BROKEN: u32 = "not a number";\n' + patched_text[module_at:]
path, base = crate_with("positive-type", broken)
r = metadata("rustc", EXT + ["--test", "-A", "warnings", "--crate-name", "pos_t", "-o", base + "/x.rmeta", path], "a type error planted in pulse_tests fails the test cfg", expect_ok=False)
ck("BROKEN" in r.stderr and "permutations.rs" in r.stderr, "and the error is that one")
metadata("rustc", EXT + ["-A", "warnings", "--crate-name", "pos_n", "-o", base + "/y.rmeta", path], "the same plant does not touch the normal cfg")
linty = patched_text[:module_at] + "    #[test]\n    fn probe() {\n        let items = [1];\n        assert!(items.len() == 0);\n    }\n" + patched_text[module_at:]
path, base = crate_with("positive-clippy", linty)
metadata("rustc", EXT + ["-D", "warnings", "-A", "dead_code", "--test", "--crate-name", "pos_l", "-o", base + "/z.rmeta", path], "a Clippy-only fault planted in pulse_tests passes rustc -D warnings")
r = metadata("clippy-driver", EXT + ["-D", "warnings", "-A", "dead_code", "--test", "--crate-name", "pos_c", "-o", base + "/w.rmeta", path], "and fails clippy-driver -D warnings in test cfg", expect_ok=False)
ck("len_zero" in r.stderr or "comparison_to_empty" in r.stderr or "len()" in r.stderr, "with the planted lint")

print("--- F. controls: each mutated pulse must still compile, so that a native failure is an assertion and not a build error")
pulse_at = patched_text.index("fn pulse(stage: &mut Stage<'_>, mobs: &[Mob])")
pulse_end = patched_text.index("\n}\n", pulse_at)
body = patched_text[pulse_at:pulse_end]
new_call = body[body.index("let a = mob"):body.index("animations.push(stage.prepare(a)?);")]
old_call = "let a = fmn::animation::indicate(stage.arena_mut(), mob, 1.25, None)?.rt(0.6);\n        "
ck(old_call.strip() in "\n".join(before), "control A's line is the one root removed")
controls = [
    ("A old indicate", new_call, old_call),
    ("B no rate_func", "                rate_func: Some(fmn::core::rate::there_and_back),\n", ""),
    ("C scale 1.0", ".scale(1.25)?", ".scale(1.0)?"),
    ("D run_time 0.5", "run_time: Some(0.6),", "run_time: Some(0.5),"),
]
for label, old, new in controls:
    ck(body.count(old) == 1, f"control {label}: the edit lands once inside fn pulse")
    mutated = patched_text[:pulse_at] + body.replace(old, new) + patched_text[pulse_end:]
    tag_name = re.sub(r"\W+", "_", label).lower()
    path, base = crate_with("control-" + tag_name, mutated)
    metadata("rustc", EXT + ["--test", "-A", "warnings", "--crate-name", "ctl_" + tag_name, "-o", base + "/x.rmeta", path], f"control {label}: compiles in test cfg")

print("--- G. what this reviewer already wrote that sits in the base")
follow = read(P + "algebra11-patch-cross-review01/patches/permutations-followup.patch")
added = [line[1:] for line in follow.split("\n") if line.startswith("+") and not line.startswith("+++") and line[1:].strip()]
present = [line for line in added if line in after]
ck(len(present) == len(added), "every non-empty line my earlier follow-up added is in candidate02", f"{len(present)} of {len(added)}")
ck(not any(("set_anim_args" in line or "there_and_back" in line or "scale(1.25)" in line) for line in added), "and none of those lines is in the pulse hunk")
review = read(P + "algebra11-patch-cross-review01/REVIEW.md")
ck("Filled tokens need a scale-only pulse or a `halo`" in review, "my earlier review named the scale-only remedy")

print("--- H. outside this hunk: other chapters' flat indicate, source only")
others = []
for name in sorted(os.listdir(CRATE + "src")):
    if name == "permutations.rs" or not name.endswith(".rs"):
        continue
    for n, line in enumerate(read(CRATE + "src/" + name).split("\n"), 1):
        if "animation::indicate(" in line:
            others.append(f"{name}:{n}")
print("other files' indicate calls:", len(others), others)
ck(len(others) == 12, "12 indicate calls outside permutations.rs")
transitivity = read(CRATE + "src/transitivity.rs").split("\n")
ck('explainer_kit::hex("#0E1116")' in transitivity[684] and "group_of(stage, &[token, number])" in transitivity[685], "transitivity.rs:685-686 groups a disk with a dark number")
ck("indicate(stage.arena_mut(), mob, scale, None)" in transitivity[144] and "pulse(stage, &[hours[0], hours[4], hours[8]]" in transitivity[693], "and transitivity.rs:694 pulses three of them through the flat indicate at :145")

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
