"""Run the repaired fact models through the real Facts type, against each oracle, with no Cargo.

Usage from the repo root:  python3 -I -B native_facts.py SCRATCH_DIR
SCRATCH_DIR receives the generated harnesses and binaries. Nothing is written in the repo.
The kit shim is crates/explainer_kit/src/facts.rs alone, as a crate called explainer_kit (std only).
"""
import hashlib
import os
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
os.makedirs(S, exist_ok=True)
R = ROOT + "/docs/video-program/third-wave-20261024/opus-replacements-20261104/"
C = R + "exponential-abc-teaching-repair01/candidate/"
F = R + "exponential-rate-facts01/candidate/"
KIT_FACTS = ROOT + "/crates/explainer_kit/src/facts.rs"
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/").replace(
        "docs/video-program/third-wave-20261024/opus-replacements-20261104/", ""
    )


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
    for line in [l for l in r.stderr.splitlines() if l.startswith(("warning", "error"))][:30]:
        print("   ", rel(line))
    if r.returncode:
        print(rel(r.stderr)[-3000:])
    return r


print("rustc:", run(["rustc", "--version"]).stdout.strip())
shim = write("kit_shim.rs", f'#[path = "{KIT_FACTS}"]\npub mod facts;\npub use facts::{{Fact, Facts, choose, oracle_diff}};\n')
kit_rlib = os.path.join(S, "libexplainer_kit.rlib")
rustc(["--crate-type", "rlib", "--crate-name", "explainer_kit", "-o", kit_rlib, shim], "kit facts shim compiles")
EXT = ["--extern", f"explainer_kit={kit_rlib}"]

main_a = f'''
#[path = "{F}src/rate_model.rs"]
mod rate_model;
#[path = "{F}src/facts.rs"]
mod facts;
#[path = "{C}series01/series_model.rs"]
mod series_model;
#[path = "{C}product01/product_model.rs"]
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
rustc(["-A", "dead_code"] + EXT + ["-o", ba, pa], "whole registry compiles (rate + series + product)")
native = run([ba]).stdout
rows = [l.split("\t", 1) for l in native.splitlines()]
ck(all(len(r) == 2 for r in rows), "every native row has a tab")
prefixes = {p: sum(1 for k, _ in rows if k.split(".")[0] == p) for p in sorted({k.split(".")[0] for k, _ in rows})}
print(f"{len(rows)} native rows; prefixes:", prefixes)

orc = {}
for name, path in (("rate", F + "oracle.py"), ("series", C + "series01/oracle.py"), ("product", C + "product01/oracle.py")):
    r = run(["python3", "-I", "-B", path])
    ck(r.returncode == 0, f"oracle {name} runs", f"{len(r.stdout.splitlines())} rows")
    orc[name] = r.stdout
combined = orc["rate"] + orc["series"] + orc["product"]
oracle_path = write("oracle_combined.tsv", combined)
print("sha256 oracle_combined:", hashlib.sha256(combined.encode()).hexdigest())
r = run([ba, oracle_path])
print(r.stdout.strip())
ck(r.stdout.strip().splitlines()[0] == "oracle_diff lines: 0", "the kit's own oracle_diff over all three oracles is empty")
d_nat = dict(rows)
d_orc = dict(l.split("\t", 1) for l in combined.splitlines())
ck(len(d_nat) == len(rows) and len(d_orc) == len(combined.splitlines()), "no duplicate keys on either side")
ck(d_nat == d_orc, "dict equality, independent of oracle_diff", f"{len(d_nat)} rows")
ck(len(rows) == 835 + 41 + 7, "row count: old 835, plus series 41, plus product 7", str(len(rows)))
for key in ("factorials.tail.4.bound", "factorials.tail.8.bound", "factorials.halving.from", "factorials.match.4.equation",
            "factorials.match.4.value", "factorials.running.4", "factorials.envelope.9", "product.e", "product.ladder.fractions"):
    print(f"  {key}: native {d_nat.get(key)} | oracle {d_orc.get(key)}")
print("  old tail rows gone:", all(k not in d_nat for k in ("factorials.tail.order", "factorials.tail.ratio", "factorials.tail.bound")))

# Negative control: one changed native value must make oracle_diff speak.
tampered = combined.replace("factorials.tail.4.bound\t0.12657", "factorials.tail.4.bound\t0.08438")
ck(tampered != combined, "control: the tampered oracle differs")
r = run([ba, write("oracle_tampered.tsv", tampered)])
ck(r.stdout.strip().splitlines()[0] != "oracle_diff lines: 0", "control: oracle_diff catches the old printed bound")

for label, path in (
    ("facts01 rate_model.rs", F + "src/rate_model.rs"),
    ("candidate series_model.rs", C + "series01/series_model.rs"),
    ("candidate product_model.rs", C + "product01/product_model.rs"),
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

out = os.path.join(S, "t_facts_all")
r = run(["rustc", "--edition", "2024", "--test", "-A", "dead_code"] + EXT + ["-o", out, pa])
ck(r.returncode == 0, "registry crate compiles in test cfg")
if r.returncode == 0:
    t = run([out])
    tail = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, "tests: whole registry crate", tail[-1] if tail else "")

for cfg, extra in (("normal", []), ("test", ["--test"])):
    r = run(["clippy-driver", "--edition", "2024", "--emit=metadata", "-D", "warnings", "-A", "dead_code"] + extra + EXT + ["-o", os.path.join(S, f"clippy_{cfg}.rmeta"), pa])
    ck(r.returncode == 0, f"clippy-driver -D warnings on the models ({cfg} cfg)")
    if r.returncode:
        print(rel(r.stderr)[-3000:])

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
