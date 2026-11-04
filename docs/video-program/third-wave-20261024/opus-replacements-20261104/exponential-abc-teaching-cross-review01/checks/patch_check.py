"""The staged correction and the wiring lines: patch fidelity, source tests by mirror, type-check, facts through facts().

Usage from the repo root:  python3 -I -B patch_check.py SCRATCH_DIR
Everything is built in SCRATCH_DIR. Rust runs as `rustc --emit=metadata` and `clippy-driver` against the existing
rmeta in target/debug/deps (fmn pin a399d62), the frozen bundle's own method, with no Cargo, no codegen for scenes
and no render. The facts harness links only the Kit's facts.rs, std only. Scene tests are NOT run in Rust: their
string walks are mirrored here. Nothing is written in the repo.
"""
import glob
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys

ROOT = os.getcwd()
R = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
B = R + "exponential-abc-teaching-repair01/"
C = B + "candidate/"
F = R + "exponential-rate-facts01/candidate/"
O = R + "exponential-abc-teaching-cross-review01/"
DEPS = ROOT + "/target/debug/deps"
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(text):
    return text.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(R, "")


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def run(args, cwd=ROOT, **kw):
    return subprocess.run(args, capture_output=True, text=True, cwd=cwd, **kw)


print("--- A0. the author's three patches, from base to candidate")
for stage, patch_name in (("rate02", "rate"), ("series01", "series"), ("product01", "product")):
    stage_dir = os.path.join(S, "author", stage)
    if os.path.isdir(stage_dir):
        shutil.move(stage_dir, stage_dir + f"-old-{len(glob.glob(stage_dir + '-old-*'))}")
    os.makedirs(stage_dir)
    for name in sorted(os.listdir(B + "base/" + stage)):
        shutil.copyfile(B + "base/" + stage + "/" + name, os.path.join(stage_dir, name))
    r = run(["patch", "-p1", "--fuzz=0", "-i", os.path.abspath(B + f"patches/{patch_name}.patch")], cwd=stage_dir)
    same = all(sha(os.path.join(stage_dir, name)) == sha(C + stage + "/" + name) for name in os.listdir(C + stage))
    ck(r.returncode == 0 and "offset" not in r.stdout.lower() and same, f"{patch_name}.patch: fuzz 0, and every {stage} candidate file matches", ",".join(sorted(os.listdir(C + stage))))

print("--- A. the patches against the frozen candidates")
repair = open(B + "REPAIR.md", encoding="utf-8").read()
for path in ("series01/factorials.rs", "product01/product_grid.rs", "product01/script-slice.tsv"):
    bound = re.search(r"`candidate/" + re.escape(path) + r"` \| `([0-9a-f]{64})`", repair)
    ck(bound is not None and sha(C + path) == bound.group(1), f"{path} is the bundle's bound file", sha(C + path)[:12])

work = os.path.join(S, "apply")
if os.path.isdir(work):
    shutil.move(work, work + f"-old-{len(glob.glob(work + '-old-*'))}")
for path in ("series01/factorials.rs", "product01/product_grid.rs", "product01/script-slice.tsv"):
    os.makedirs(os.path.dirname(os.path.join(work, path)), exist_ok=True)
    shutil.copyfile(C + path, os.path.join(work, path))
for name, files in (("phrase-timing.patch", ("series01/factorials.rs", "product01/product_grid.rs")), ("03_7-wording.patch", ("product01/script-slice.tsv",))):
    patch = os.path.abspath(O + "patches/" + name)
    before = {f: sha(os.path.join(work, f)) for f in files}
    r = run(["patch", "-p1", "--fuzz=0", "-i", patch], cwd=work)
    ck(r.returncode == 0 and "fuzz" not in r.stdout and "offset" not in r.stdout.lower(), f"{name} applies, fuzz 0, no offset")
    after = {f: sha(os.path.join(work, f)) for f in files}
    for f in files:
        print(f"    {f}: {after[f]}")
    r = run(["patch", "-p1", "-R", "--fuzz=0", "-i", patch], cwd=work)
    ck(r.returncode == 0 and {f: sha(os.path.join(work, f)) for f in files} == before, f"{name} reverses to the frozen bytes")
    r = run(["patch", "-p1", "--fuzz=0", "-i", patch], cwd=work)
    assert r.returncode == 0
for leftover in glob.glob(work + "/**/*.orig", recursive=True):
    os.rename(leftover, leftover + ".kept")

scenes = {name: open(os.path.join(work, path), encoding="utf-8").read() for name, path in (("factorials", "series01/factorials.rs"), ("product_grid", "product01/product_grid.rs"))}
for name, src in scenes.items():
    for gone in ("until_clause", "_SENTENCE", "frames_covering", "wait_frames", ".expect(\"the clause"):
        ck(gone not in src, f"{name}: no {gone} left")
    body = src.split("#[cfg(test)]")[0]
    print(f"    {name}: {len(re.findall(r'until_phrase[(]', body))} until_phrase call sites")
ck(len(re.findall(r"until_phrase[(]", scenes["factorials"].split("#[cfg(test)]")[0])) == 2, "factorials keeps its two call sites")
ck(len(re.findall(r"until_phrase[(]", scenes["product_grid"].split("#[cfg(test)]")[0])) == 3, "product_grid keeps its three call sites")

print("--- A2. phrase.rs against the corpus, whitespace and trailing commas aside")


def fn_text(source, signature):
    at = source.index(signature)
    depth = 0
    for i in range(at, len(source)):
        if source[i] == "{":
            depth += 1
        elif source[i] == "}":
            depth -= 1
            if depth == 0:
                return source[at : i + 1]
    raise ValueError(signature)


def squeeze(text):
    return re.sub(r",\s*\)", ")", re.sub(r"\s+", " ", text)).replace("( ", "(").replace(" )", ")").replace(". ", ".")


corpus = open("explainer_corpus/logarithm_mysteries/src/chapters/unreachable.rs", encoding="utf-8").read()
staged = open(O + "candidate/phrase.rs", encoding="utf-8").read()
for signature in ("fn phrase_seconds(", "fn phrase_time(", "fn until_phrase("):
    ck(squeeze(fn_text(staged, signature)) == squeeze(fn_text(corpus, signature)), f"{signature}...) is the corpus function")

print("--- B. formatting")
for name, path in (("factorials.rs", "series01/factorials.rs"), ("product_grid.rs", "product01/product_grid.rs")):
    r = run(["rustfmt", "--edition", "2024", "--config", "max_width=120", "--check", os.path.join(work, path)])
    ck(r.returncode == 0, f"rustfmt --check on the patched {name}", rel(r.stdout)[:300])
r = run(["rustfmt", "--edition", "2024", "--config", "max_width=120", "--check", O + "candidate/phrase.rs"])
ck(r.returncode == 0, "rustfmt --check on phrase.rs", rel(r.stdout)[:300])

print("--- C. the scene tests' string walks and the script bindings, mirrored (not run in Rust)")


def needle_lists(src):
    out = []
    for block in re.findall(r"assert_in_order\(&\[(.*?)\]\);", src[src.index("#[cfg(test)]") :], re.S):
        needles = []
        for raw, plain in re.findall(r'r#"(.*?)"#|"((?:[^"\\]|\\.)*)"', block):
            needles.append(raw if raw else plain.replace('\\"', '"').replace("\\\\", "\\"))
        out.append(needles)
    return out


def walk(body, needles):
    start = 0
    for needle in needles:
        at = body.find(needle, start)
        if at < 0:
            return needle
        start = at + len(needle)
    return None


for name, src in scenes.items():
    body = src.split("#[cfg(test)]")[0]
    lists = needle_lists(src)
    bad = [walk(body, needles) for needles in lists]
    ck(not any(bad) and lists, f"{name}: all {len(lists)} assert_in_order walks hold", str([b for b in bad if b])[:200])
    swapped = next(needles for needles in lists if any("until_phrase" in n for n in needles))
    swapped = list(reversed(swapped))
    ck(walk(body, swapped) is not None, f"{name}: control, a reversed walk fails")
slice_rows = {}
for name in ("series01", "product01"):
    for line in open(C + name + "/script-slice.tsv", encoding="utf-8").read().splitlines():
        key, text = line.split("\t", 1)
        slice_rows[key] = text
debts = re.findall(r'\("(why [a-z ]+)", [A-Z]+_DEBT\)', scenes["factorials"])
at = [slice_rows["02_6"].find(d) for d in debts]
ck(len(debts) == 3 and -1 not in at and at == sorted(at), "every_clause_is_in_its_line_in_spoken_order", str(debts))
ck("and that differentiating" in slice_rows["02_4"], "02_4 has its phrase")
ck("collect them" in slice_rows["03_2"] and slice_rows["03_7"].find("reciprocals") < slice_rows["03_7"].find("continuity"), "every_phrase_is_in_its_line_in_spoken_order")

print("--- D. the corrected crate shape type-checks: phrase, scenes, models, facts() and Chapter rows")
crate = os.path.join(S, "crate")
src_dir = os.path.join(crate, "src")
os.makedirs(os.path.join(src_dir, "chapters"), exist_ok=True)
with open(os.path.join(crate, "script.tsv"), "w", encoding="utf-8") as out:
    for key, text in slice_rows.items():
        out.write(f"{key}\t{text}\n")
shutil.copyfile(O + "candidate/phrase.rs", os.path.join(src_dir, "phrase.rs"))
shutil.copyfile(os.path.join(work, "series01/factorials.rs"), os.path.join(src_dir, "chapters/factorials.rs"))
shutil.copyfile(os.path.join(work, "product01/product_grid.rs"), os.path.join(src_dir, "chapters/product_grid.rs"))
shutil.copyfile(C + "rate02/rate_law.rs", os.path.join(src_dir, "chapters/rate_law.rs"))
shutil.copyfile(F + "src/rate_model.rs", os.path.join(src_dir, "rate_model.rs"))
shutil.copyfile(C + "series01/series_model.rs", os.path.join(src_dir, "series_model.rs"))
shutil.copyfile(C + "product01/product_model.rs", os.path.join(src_dir, "product_model.rs"))
facts_src = open(F + "src/facts.rs", encoding="utf-8").read()
anchor = "    crate::rate_model::append_facts(&mut facts)?;\n"
assert facts_src.count(anchor) == 1
wired = facts_src.replace(anchor, anchor + "    crate::series_model::append_facts(&mut facts)?;\n    crate::product_model::append_facts(&mut facts)?;\n")
with open(os.path.join(src_dir, "facts.rs"), "w", encoding="utf-8") as out:
    out.write(wired)
chapters = """mod factorials;
mod product_grid;
mod rate_law;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "rate", make: |kit| Box::new(rate_law::RateLaw { kit }), intensity: 0.4 },
        Chapter { key: "series", make: |kit| Box::new(factorials::Factorials { kit }), intensity: 0.4 },
        Chapter { key: "product", make: |kit| Box::new(product_grid::ProductGrid { kit }), intensity: 0.4 },
    ]
}
"""
with open(os.path.join(src_dir, "chapters/mod.rs"), "w", encoding="utf-8") as out:
    out.write(chapters)
main = """mod chapters;
mod facts;
mod phrase;
mod product_model;
mod rate_model;
mod series_model;

fn main() {
    let f = facts::facts().map(|f| f.print());
    let _ = (f, chapters::registry());
}
"""
main_path = os.path.join(src_dir, "main.rs")
with open(main_path, "w", encoding="utf-8") as out:
    out.write(main)


def one(pattern):
    hits = [h for h in glob.glob(f"{DEPS}/lib{pattern}-*.rmeta") if os.path.getsize(h) > 0]
    assert len(hits) == 1, (pattern, len(hits))
    return hits[0]


def metadata(driver, args, label, expect_ok=True):
    r = run([driver, "--edition", "2024", "--emit=metadata", "-L", f"dependency={S}", "-L", f"dependency={DEPS}"] + args,
            env={**os.environ, "CARGO_PKG_VERSION": "0.1.0", "CARGO_PKG_NAME": "x", "CARGO_MANIFEST_DIR": ROOT})
    ck((r.returncode == 0) == expect_ok, label)
    if expect_ok and r.returncode:
        print(rel(r.stderr)[-6000:])
    return r


fmn = one("fmn")
dep_info = open(fmn.replace("/libfmn-", "/fmn-").replace(".rmeta", ".d"), encoding="utf-8").read()
ck("franken_manim-6641d0e09642f63e/a399d62/crates/fmn/src/lib.rs" in dep_info, "fmn metadata was built from the a399d62 checkout")
score = os.path.join(S, "libexplainer_score.rmeta")
metadata("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_score", "-A", "warnings", "-o", score, ROOT + "/crates/explainer_score/src/lib.rs"], "explainer_score metadata from the current tree")
kit = os.path.join(S, "libexplainer_kit.rmeta")
metadata("rustc", ["--crate-type", "rlib", "--crate-name", "explainer_kit", "-A", "warnings", "--extern", f"fmn={fmn}", "--extern", f"fmn_output={one('fmn_output')}",
                   "--extern", f"fmn_codec={one('fmn_codec')}", "--extern", f"serde_json={one('serde_json')}", "--extern", f"explainer_score={score}", "-o", kit,
                   ROOT + "/crates/explainer_kit/src/lib.rs"], "explainer_kit metadata from the current tree")
EXT = ["--extern", f"fmn={fmn}", "--extern", f"explainer_kit={kit}"]
outs = os.path.join(S, "outs")
os.makedirs(outs, exist_ok=True)
metadata("rustc", EXT + ["-D", "warnings", "--crate-name", "exp_crate", "-o", outs + "/a.rmeta", main_path], "rustc -D warnings, normal cfg")
metadata("rustc", EXT + ["--test", "-A", "dead_code", "--crate-name", "exp_crate_t", "-o", outs + "/b.rmeta", main_path], "rustc, test cfg")
metadata("clippy-driver", EXT + ["-D", "warnings", "--crate-name", "exp_clip", "-o", outs + "/c.rmeta", main_path], "clippy-driver -D warnings, normal cfg")
metadata("clippy-driver", EXT + ["-D", "warnings", "-A", "dead_code", "--test", "--crate-name", "exp_clip_t", "-o", outs + "/d.rmeta", main_path], "clippy-driver -D warnings, test cfg")
bad_main = main.replace("mod phrase;\n", "")
bad_path = os.path.join(src_dir, "main_without_phrase.rs")
with open(bad_path, "w", encoding="utf-8") as out:
    out.write(bad_main)
r = metadata("rustc", EXT + ["--crate-name", "exp_bad", "-o", outs + "/e.rmeta", bad_path], "control: without `mod phrase;` the crate does not compile", expect_ok=False)

print("--- D2. the two optional recurrence lines in REVIEW.md type-check")
chapter = os.path.join(src_dir, "chapters/factorials.rs")
plain = open(chapter, encoding="utf-8").read()
trial = plain
for before, after in (
    ("    let fourth = print_equation(stage, k, table, 4)?;\n", "    let fourth = print_equation(stage, k, table, 4)?;\n    stage.wait(0.5)?;\n"),
    ("    reveal_all(stage, &chain, 0.4)?;\n", '    chain.push(k.text(stage, "coefficient", 23.0, Palette::dim(), v(LABEL_X, CHAIN_Y))?);\n    reveal_all(stage, &chain, 0.4)?;\n'),
):
    assert trial.count(before) == 1, before
    trial = trial.replace(before, after)
with open(chapter, "w", encoding="utf-8") as out:
    out.write(trial)
metadata("rustc", EXT + ["-D", "warnings", "--crate-name", "exp_opt", "-o", outs + "/f.rmeta", main_path], "rustc -D warnings with both lines")
with open(chapter, "w", encoding="utf-8") as out:
    out.write(plain)

print("--- E. the oracle and the two facts lines")
oracles = [F + "oracle.py", C + "series01/oracle.py", C + "product01/oracle.py"]
joined = "".join(open(p, encoding="utf-8").read() for p in oracles)
joined_path = os.path.join(S, "oracle_joined.py")
with open(joined_path, "w", encoding="utf-8") as out:
    out.write(joined)
r = run(["python3", "-I", "-B", joined_path])
ck(r.returncode == 0, "one file holding the three oracles in this order runs")
digest = hashlib.sha256(r.stdout.encode()).hexdigest()
ck(digest in repair, "its stdout is the combined oracle REPAIR.md binds", digest)
oracle_rows = dict(line.split("\t", 1) for line in r.stdout.splitlines())
print("    rows:", len(oracle_rows), "| imports:", sorted(set(re.findall(r"^(?:from|import) (\w+)", joined, re.M))))
shim = os.path.join(S, "kit_shim.rs")
with open(shim, "w", encoding="utf-8") as out:
    out.write(f'#[path = "{ROOT}/crates/explainer_kit/src/facts.rs"]\npub mod facts;\npub use facts::{{Fact, Facts, choose, oracle_diff}};\n')
rlib = os.path.join(S, "libexplainer_kit.rlib")
r = run(["rustc", "--edition", "2024", "--crate-type", "rlib", "--crate-name", "explainer_kit", "-o", rlib, shim])
ck(r.returncode == 0, "Kit facts shim builds", rel(r.stderr)[-300:] if r.returncode else "")
harness = f"""
#[path = "{src_dir}/rate_model.rs"]
mod rate_model;
#[path = "{src_dir}/series_model.rs"]
mod series_model;
#[path = "{src_dir}/product_model.rs"]
mod product_model;
#[path = "{{FACTS}}"]
mod facts;
fn main() -> Result<(), String> {{
    print!("{{}}", facts::facts()?.print());
    Ok(())
}}
"""


def facts_rows(path, name):
    p = os.path.join(S, name + ".rs")
    with open(p, "w", encoding="utf-8") as out:
        out.write(harness.replace("{FACTS}", path))
    binary = os.path.join(S, name)
    r = run(["rustc", "--edition", "2024", "-A", "dead_code", "--extern", f"explainer_kit={rlib}", "-o", binary, p])
    ck(r.returncode == 0, f"{name} compiles", rel(r.stderr)[-500:] if r.returncode else "")
    return dict(line.split("\t", 1) for line in run([binary]).stdout.splitlines())


native = facts_rows(os.path.join(src_dir, "facts.rs"), "facts_wired")
ck(native == oracle_rows and len(native) == 883, "facts() with the two lines before self_check prints 883 rows, all equal to the oracle", str(len(native)))
print("    factorials.tail.4.bound", native.get("factorials.tail.4.bound"), "| factorials.tail.8.bound", native.get("factorials.tail.8.bound"), "| product.e", native.get("product.e"))
native_old = facts_rows(os.path.abspath(F + "src/facts.rs"), "facts_unwired")
ck(len(native_old) == 183 and native_old != oracle_rows, "control: facts01's facts() without the lines prints only the 183 older rows", str(len(native_old)))

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
