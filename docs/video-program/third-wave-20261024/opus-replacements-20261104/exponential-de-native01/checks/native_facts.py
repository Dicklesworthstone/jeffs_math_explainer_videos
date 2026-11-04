"""Run the integrated crate's real fact registry against its real oracle, with no Cargo.

Usage from the repo root:  python3 -I -B native_facts.py SCRATCH_DIR
SCRATCH_DIR/integrated must come from make_integration.py. The kit shim is crates/explainer_kit/src/facts.rs alone,
as a crate called explainer_kit (std only). The models and facts.rs are the integrated crate's own files.
"""
import hashlib
import os
import subprocess
import sys

ROOT = os.getcwd()
S = os.path.abspath(sys.argv[1])
CRATE = S + "/integrated"
WORK = S + "/native"
os.makedirs(WORK, exist_ok=True)
KIT_FACTS = ROOT + "/crates/explainer_kit/src/facts.rs"
MODELS = ("rate_model", "facts", "series_model", "product_model", "limit_model", "solution_model")
BASE_ROWS, LIMIT_ROWS, ONLY_ROWS = 883, 176, 20
fails = 0


def ck(ok, label, extra=""):
    global fails
    print(("PASS " if ok else "FAIL ") + label + (("  " + extra) if extra else ""))
    fails += 0 if ok else 1


def rel(s):
    return s.replace(ROOT + "/", "").replace(S + "/", "<scratch>/")


def run(args, **kw):
    return subprocess.run(args, capture_output=True, text=True, cwd=ROOT, **kw)


def write(name, text):
    p = os.path.join(WORK, name)
    with open(p, "w", encoding="utf-8") as out:
        out.write(text)
    return p


def rustc(args, label):
    r = run(["rustc", "--edition", "2024"] + args)
    ck(r.returncode == 0, label)
    if r.returncode:
        print(rel(r.stderr)[-4000:])
    return r


print("rustc:", run(["rustc", "--version"]).stdout.strip())
shim = write("kit_shim.rs", f'#[path = "{KIT_FACTS}"]\npub mod facts;\npub use facts::{{Fact, Facts, choose, oracle_diff}};\n')
kit_rlib = os.path.join(WORK, "libexplainer_kit.rlib")
rustc(["--crate-type", "rlib", "--crate-name", "explainer_kit", "-o", kit_rlib, shim], "kit facts shim compiles")
EXT = ["--extern", f"explainer_kit={kit_rlib}"]

mods = "".join(f'#[path = "{CRATE}/src/{m}.rs"]\nmod {m};\n' for m in MODELS)
main = mods + """
fn main() -> Result<(), String> {
    let f = facts::facts()?;
    let ours = f.print();
    match std::env::args().nth(1) {
        Some(p) => {
            let oracle = std::fs::read_to_string(p).map_err(|e| e.to_string())?;
            let d = explainer_kit::oracle_diff(&ours, &oracle);
            println!("oracle_diff lines: {}", d.len());
            for l in d.iter().take(40) { println!("  {l}"); }
        }
        None => print!("{ours}"),
    }
    Ok(())
}
"""
pa = write("facts_all.rs", main)
ba = os.path.join(WORK, "facts_all")
rustc(["-A", "dead_code"] + EXT + ["-o", ba, pa], "the integrated registry compiles, facts.rs included")
native = run([ba])
ck(native.returncode == 0, "the registry runs, and its self_check accepts it")
if native.returncode:
    print(native.stderr[-2000:])
rows = [l.split("\t", 1) for l in native.stdout.splitlines()]
ck(all(len(r) == 2 for r in rows), "every native row has a tab")
prefixes = {p: sum(1 for k, _ in rows if k.split(".")[0] == p) for p in sorted({k.split(".")[0] for k, _ in rows})}
print(f"{len(rows)} native rows; prefixes:", prefixes)

r = run(["python3", "-I", "-B", CRATE + "/oracle.py"])
ck(r.returncode == 0, "the concatenated oracle runs", f"{len(r.stdout.splitlines())} rows")
oracle = r.stdout
oracle_path = write("oracle.tsv", oracle)
print("sha256 oracle stdout:", hashlib.sha256(oracle.encode()).hexdigest())
print("sha256 native facts :", hashlib.sha256(native.stdout.encode()).hexdigest())
r = run([ba, oracle_path])
print(r.stdout.strip())
ck(r.stdout.strip().splitlines()[0] == "oracle_diff lines: 0", "the kit's own oracle_diff is empty")
d_nat = dict(rows)
d_orc = dict(l.split("\t", 1) for l in oracle.splitlines())
ck(len(d_nat) == len(rows) and len(d_orc) == len(oracle.splitlines()), "no duplicate keys on either side")
ck(d_nat == d_orc, "dict equality, independent of oracle_diff", f"{len(d_nat)} rows")
ck(len(rows) == BASE_ROWS + LIMIT_ROWS + ONLY_ROWS, f"row count: stage {BASE_ROWS}, plus limit {LIMIT_ROWS}, plus only {ONLY_ROWS}", str(len(rows)))
ck(prefixes.get("tiny") == LIMIT_ROWS, "every limit row carries tiny.", str(prefixes.get("tiny")))
ck(prefixes.get("only") == ONLY_ROWS, "every uniqueness row carries only.", str(prefixes.get("only")))
for key in (
    "tiny.one.1024", "tiny.total.1024", "tiny.distance.1024", "only.defect.2", "only.defect.4", "only.axes.tops", "only.rate.1/2.end",
):
    print(f"  {key}: native {d_nat.get(key)} | oracle {d_orc.get(key)}")

tampered = oracle.replace("only.defect.4\t24", "only.defect.4\t12")
ck(tampered != oracle, "control: the tampered oracle differs")
r = run([ba, write("oracle_tampered.tsv", tampered)])
ck(r.stdout.strip().splitlines()[0] != "oracle_diff lines: 0", "control: oracle_diff catches one changed denominator")

out = os.path.join(WORK, "t_facts_all")
r = run(["rustc", "--edition", "2024", "--test", "-A", "dead_code"] + EXT + ["-o", out, pa])
ck(r.returncode == 0, "the models compile in test cfg")
if r.returncode == 0:
    t = run([out])
    tail = [l for l in t.stdout.splitlines() if l.startswith("test ")]
    ck(t.returncode == 0, "tests: every model and facts.rs", tail[-1] if tail else "")
    for l in tail[:-1]:
        if "limit_model" in l or "solution_model" in l or "FAILED" in l:
            print("    ", l)
    if t.returncode:
        print(rel(t.stdout)[-3000:])

print("RESULT", "PASS" if not fails else f"{fails} FAIL")
sys.exit(1 if fails else 0)
