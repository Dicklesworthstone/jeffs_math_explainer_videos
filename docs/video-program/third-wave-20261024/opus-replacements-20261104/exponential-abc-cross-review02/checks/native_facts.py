"""Run the staged fact models through the real Facts type, against each oracle, with no Cargo.

Usage from the repo root:  python3 -I -B native_facts.py SCRATCH_DIR
SCRATCH_DIR receives the generated harnesses and binaries. Nothing is written in the repo.

What it builds, all with single rustc calls on the pinned toolchain:
  kit shim   crates/explainer_kit/src/facts.rs as a crate called explainer_kit (std only)
  A          candidate facts.rs + facts01 rate_model + series and product candidates, one registry
  B          the unpatched series model, to test the review's prediction of 11 differing rows
  tests      every #[test] in each model and in the candidate facts.rs
  clippy     clippy-driver -D warnings on A, in both normal and test cfg (dead code allowed here;
             the scene check is what can see dead code)
"""
import hashlib
import os
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
W = R + "exponential-expanded-review01/"
F = R + "exponential-rate-facts01/candidate/"
KIT_FACTS = ROOT + "/crates/explainer_kit/src/facts.rs"
RELEVANT = "docs/video-program/third-wave-20261024/opus-replacements-20261104/"
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(RELEVANT, "")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, cwd=ROOT, **kw)


def write(name, text):
    p = os.path.join(S, name)
    with open(p, "w", encoding="utf-8") as out:
        out.write(text)
    return p


def rustc(args, label):
    r = run(["rustc", "--edition", "2024"] + args)
    ck(r.returncode == 0, label)
    warns = [l for l in r.stderr.splitlines() if l.startswith(("warning", "error"))]
    for line in warns[:30]:
        print("   ", rel(line))
    if r.returncode:
        print(rel(r.stderr)[-3000:])
    return r


print("rustc:", run(["rustc", "--version"]).stdout.strip())

# kit shim: the real facts.rs, nothing else
shim = write("kit_shim.rs", f'#[path = "{KIT_FACTS}"]\npub mod facts;\npub use facts::{{Fact, Facts, choose, oracle_diff}};\n')
kit_rlib = os.path.join(S, "libexplainer_kit.rlib")
rustc(["--crate-type", "rlib", "--crate-name", "explainer_kit", "-o", kit_rlib, shim], "kit facts shim compiles")
EXT = ["--extern", f"explainer_kit={kit_rlib}"]

# A: the whole registry as integration would build it
main_a = f'''
#[path = "{F}src/rate_model.rs"]
mod rate_model;
#[path = "{F}src/facts.rs"]
mod facts;
#[path = "{W}candidate/series01/series_model.rs"]
mod series_model;
#[path = "{W}candidate/product01/product_model.rs"]
mod product_model;
fn main() -> Result<(), String> {{
    let mut f = facts::facts()?;
    series_model::append_facts(&mut f)?;
    product_model::append_facts(&mut f)?;
    f.self_check()?;
    let ours = f.print();
    match std::env::args().nth(1) {{
        Some(p) => {{
            let oracle = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
            let d = explainer_kit::oracle_diff(&ours, &oracle);
            println!("oracle_diff lines: {{}}", d.len());
            for l in d.iter().take(40) {{ println!("  {{l}}"); }}
        }}
        None => print!("{{ours}}"),
    }}
    Ok(())
}}
'''
pa = write("facts_all.rs", main_a)
ba = os.path.join(S, "facts_all")
rustc(["-A", "dead_code"] + EXT + ["-o", ba, pa], "A: candidate registry compiles (rate + series + product)")
native = run([ba]).stdout
rows = [l.split("\t", 1) for l in native.splitlines()]
ck(all(len(r) == 2 for r in rows), "A: every native row has a tab")
print(f"A: {len(rows)} native rows; prefixes:", {p: sum(1 for k, _ in rows if k.split('.')[0] == p) for p in sorted({k.split('.')[0] for k, _ in rows})})

# the three candidate oracles, stdout only
orc = {}
for name, path in (("rate", F + "oracle.py"), ("series", W + "candidate/series01/oracle.py"), ("product", W + "candidate/product01/oracle.py")):
    r = run(["python3", "-I", "-B", path])
    ck(r.returncode == 0, f"oracle {name} runs")
    orc[name] = r.stdout
    print(f"   {name}: {len(r.stdout.splitlines())} rows, stderr {len(r.stderr)} bytes")
combined = orc["rate"] + orc["series"] + orc["product"]
oracle_path = write("oracle_combined.tsv", combined)
print("sha256 oracle_combined:", hashlib.sha256(combined.encode()).hexdigest())
r = run([ba, oracle_path])
print(r.stdout.strip())
ck(r.stdout.strip().splitlines()[0] == "oracle_diff lines: 0", "A: the kit's own oracle_diff over all three oracles is empty")
d_nat = dict(rows)
d_orc = dict(l.split("\t", 1) for l in combined.splitlines())
ck(len(d_nat) == len(rows) and len(d_orc) == len(combined.splitlines()), "A: no duplicate keys on either side")
ck(d_nat == d_orc, "A: dict equality, independent of oracle_diff", f"{len(d_nat)} rows")
stored = open(R + "exponential-rate-facts01/oracle.tsv", encoding="utf-8").read()
ck(stored == orc["rate"], "facts01 stored oracle.tsv equals a fresh run")
stale = open(R + "exponential-expanded-series01/oracle01.tsv", encoding="utf-8").read()
print(f"series stored oracle01.tsv: {len(stale.splitlines())} rows (candidate oracle: {len(orc['series'].splitlines())}); equal: {stale == orc['series']}")
print("native rate.8.1.right_readout:", d_nat.get("rate.8.1.right_readout"), "| oracle:", d_orc.get("rate.8.1.right_readout"))
print("native tail:", d_nat["factorials.tail.ratio"], d_nat["factorials.tail.bound"], "| oracle:", d_orc["factorials.tail.ratio"], d_orc["factorials.tail.bound"])

# B: unpatched series model through the native path
main_b = f'''
#[path = "{W}base/series01/series_model.rs"]
mod series_model;
fn main() -> Result<(), String> {{
    let mut f = explainer_kit::Facts::new();
    series_model::append_facts(&mut f)?;
    print!("{{}}", f.print());
    Ok(())
}}
'''
pb = write("facts_base_series.rs", main_b)
bb = os.path.join(S, "facts_base_series")
rustc(["-A", "dead_code"] + EXT + ["-o", bb, pb], "B: base series model compiles")
base_native = dict(l.split("\t", 1) for l in run([bb]).stdout.splitlines())
base_oracle = dict(l.split("\t", 1) for l in run(["python3", "-I", "-B", W + "base/series01/oracle.py"]).stdout.splitlines())
diffs = sorted(k for k in set(base_native) | set(base_oracle) if base_native.get(k) != base_oracle.get(k))
print(f"B: base native {len(base_native)} rows, base oracle {len(base_oracle)} rows, {len(diffs)} differ")
for k in diffs:
    print(f"   {k}: native {base_native.get(k)} oracle {base_oracle.get(k)}")

# tests in each model, real rustc --test
for label, path in (
    ("rate02/rate_model.rs", R + "exponential-expanded-rate02/rate_model.rs"),
    ("facts01 rate_model.rs", F + "src/rate_model.rs"),
    ("candidate series_model.rs", W + "candidate/series01/series_model.rs"),
    ("candidate product_model.rs", W + "candidate/product01/product_model.rs"),
    ("base series_model.rs", W + "base/series01/series_model.rs"),
    ("base product_model.rs", W + "base/product01/product_model.rs"),
):
    out = os.path.join(S, "t_" + label.replace(" ", "_").replace("/", "_").replace(".", "_"))
    r = run(["rustc", "--edition", "2024", "--test", "-A", "dead_code"] + EXT + ["--crate-name", "m", "-o", out, path])
    if r.returncode:
        ck(False, f"tests compile: {label}")
        print(rel(r.stderr)[-1500:])
        continue
    t = run([out])
    tail = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, f"tests: {label}", tail[-1] if tail else "")
    for l in tail[:-1]:
        print("    ", l)

# candidate facts.rs own tests and the registry in test cfg
out = os.path.join(S, "t_facts_all")
r = run(["rustc", "--edition", "2024", "--test", "-A", "dead_code"] + EXT + ["-o", out, pa])
ck(r.returncode == 0, "A compiles in test cfg")
if r.returncode == 0:
    t = run([out])
    tail = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, "tests: whole registry crate", tail[-1] if tail else "")

# clippy on the model code
for cfg, extra in (("normal", []), ("test", ["--test"])):
    r = run(["clippy-driver", "--edition", "2024", "--emit=metadata", "-D", "warnings", "-A", "dead_code"] + extra + EXT + ["-o", os.path.join(S, f"clippy_{cfg}.rmeta"), pa])
    ck(r.returncode == 0, f"clippy-driver -D warnings on the three models and candidate facts.rs ({cfg} cfg)")
    if r.returncode:
        print(rel(r.stderr)[-3000:])

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
