"""Narrow integration tests for the algebra-fixes build helper. Runs no cargo and no guard.

Runs prepare_build01.py for real into an empty scratch folder (--out), checks what it wrote
independently of its own checks, checks that a second run refuses and changes nothing, feeds
expect_build01.py root's real build log and facts plus planted faults, and checks that build01.sh
parses and keeps root's four cargo commands with the same flags.

    python3 -I -B checks/test_build_helper.py <empty scratch folder>      (from the repository root)
"""
import hashlib
import json
import re
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parents[1]
REPO = HERE.parents[7]
SCRATCH = Path(sys.argv[1]).resolve()
OUT = SCRATCH / "build01"
N = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/native-stage08-root01"
OLD = N / "corner-label-contrast-repair01"
OLD_JOB = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027/algebra-corner-contrast-build01"
CRATE = "explainer_corpus/abstract_algebra_expanded"
NEW_TEST = "cosets::tests::right_coset_links_touch_their_tiles_and_cross_no_tile_or_name"
FAILED = []


def check(ok, what):
    print("ok  " if ok else "FAIL", what)
    if not ok:
        FAILED.append(what)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def tree(root):
    return {p.relative_to(root).as_posix(): sha(p) for p in sorted(root.rglob("*")) if p.is_file()}


def run(*argv):
    return subprocess.run([sys.executable, "-I", "-B", *map(str, argv)], cwd=REPO, capture_output=True, text=True)


# 1. Prepare for real, into scratch.
assert not any(SCRATCH.iterdir()), "scratch folder must be empty"
first = run(HERE / "prepare_build01.py", "--out", OUT)
check(first.returncode == 0, f"prepare exits 0 ({first.returncode}) {first.stderr.strip()[-300:]}")
if first.returncode:
    sys.exit(1)

old_ws, new_ws = tree(OLD / "workspace"), tree(OUT / "workspace")
changed = sorted(k for k in old_ws if new_ws.get(k) != old_ws[k])
check(changed == sorted(f"{CRATE}/{p}" for p in ("narration_units.json", "script.tsv", "src/cosets.rs", "src/permutations.rs")),
      f"exactly four workspace files change: {changed}")
check(sorted(set(new_ws) - set(old_ws)) == [f"{CRATE}/docs/narration-direction-map-frederick-stage14.json"],
      "one file is added, the stage14 map, and none is lost")
check(new_ws[f"{CRATE}/src/cosets.rs"] == sha(HERE / "cosets.rs")
      and new_ws[f"{CRATE}/src/permutations.rs"] == sha(HERE / "permutations.rs"), "both candidate sources installed")

old_rows = (OLD / f"workspace/{CRATE}/script.tsv").read_text().splitlines()
new_rows = (OUT / f"workspace/{CRATE}/script.tsv").read_text().splitlines()
text = (HERE / "11_17-frederick.txt").read_text().strip()
diff = [(a, b) for a, b in zip(old_rows, new_rows) if a != b]
check(len(old_rows) == len(new_rows) and len(diff) == 1 and diff[0][1] == f"11_17\t{text}",
      "script.tsv: only the 11_17 row changes, to the whole new text")

stage13 = json.loads((OLD / f"workspace/{CRATE}/docs/narration-direction-map-frederick-stage13.json").read_text())
stage14 = json.loads((OUT / f"workspace/{CRATE}/docs/narration-direction-map-frederick-stage14.json").read_text())
script_sha = sha(OUT / f"workspace/{CRATE}/script.tsv")
check(stage14["scripts"] == {"abstract_algebra_expanded": script_sha} and stage14["overrides"] == stage13["overrides"],
      "stage14 binds the new script; overrides unchanged")
moved = []
for a, b in zip(stage13["visual_holds"], stage14["visual_holds"]):
    if a == b:
        continue
    same_rest = {k: v for k, v in a.items() if k not in ("sha256", "line")} == {k: v for k, v in b.items() if k not in ("sha256", "line")}
    lines = (OUT / "workspace" / b["path"]).read_text().splitlines()
    says = [m.group(1) for line in lines[:b["line"] - 1] if (m := re.search(r'k\.say\(stage, "(\d\d_\d+)"\)', line))]
    ok = same_rest and b["sha256"] == sha(OUT / "workspace" / b["path"]) and lines[b["line"] - 1].strip() == b["marker"] and says[-1] == b["cue"]
    moved.append((b["cue"], a["line"], b["line"], ok))
check(len(stage13["visual_holds"]) == len(stage14["visual_holds"]) and moved and all(m[3] for m in moved),
      f"changed holds point at their marker after their cue's say: {[(c, f'{x}->{y}') for c, x, y, _ in moved]}")

old_plan = json.loads((OLD / f"workspace/{CRATE}/narration_units.json").read_text())
plan = json.loads((OUT / f"workspace/{CRATE}/narration_units.json").read_text())
units_diff = [(a["id"], b["text"]) for a, b in zip(old_plan["units"], plan["units"]) if a != b]
contract = plan["direction_contract"]
check(units_diff == [("oliver_11_17", text)] and plan["script_sha256"] == contract["script_sha256"] == script_sha
      and contract["map"] == {"path": f"{CRATE}/docs/narration-direction-map-frederick-stage14.json",
                              "sha256": sha(OUT / "workspace" / contract["map"]["path"])},
      "plan: only the 11_17 text, the script hashes and the map binding change")

root_rows = json.loads((OLD / "build01-inputs.json").read_text())["files"]
rows = json.loads((OUT / "build01-inputs.json").read_text())["files"]
paths = [row["path"] for row in rows]
bad = [row["path"] for row in rows if sha(row["path"]) != row["sha256"]]
check(rows[:len(root_rows)] == root_rows and not bad and len(set(paths)) == len(paths),
      f"manifest keeps root's {len(root_rows)} rows first, {len(rows)} rows in all, every hash current")
needed = [str(OUT / "workspace" / rel) for rel in new_ws] + [str(HERE / n) for n in ("build01.sh", "expect_build01.py", "prepare_build01.py")]
needed += [str(OUT / "build.env"), str(OUT / "expected-tests.txt"), str(OLD / "native-facts01.tsv"), str(OLD_JOB / "command.log")]
check(all(p in paths for p in needed) and all(p.startswith("/") for p in paths),
      "manifest binds the new workspace, the build files, build.env, expected tests and root's outputs")

passed = re.findall(r"^test (\S+) \.\.\. ok$", (OLD_JOB / "command.log").read_text(), re.M)
expected = (OUT / "expected-tests.txt").read_text().splitlines()
check(expected == sorted(passed + [NEW_TEST]) and len(expected) == 123, f"expected tests: root's {len(passed)} plus the new one")
budget = set(re.findall(r"--config (\S+)", (OLD / "build01.sh").read_text()))
check((OUT / "build.env").read_text() == f"BUDGET={budget.pop()}\n", "build.env names root's engine patch table")

# 2. A second run refuses and changes nothing.
before = tree(SCRATCH)
second = run(HERE / "prepare_build01.py", "--out", OUT)
check(second.returncode != 0 and "already exists" in second.stderr and tree(SCRATCH) == before,
      "a second prepare into the same folder refuses and writes nothing")

# 3. expect_build01.py on root's real outputs and planted faults.
log = (OLD_JOB / "command.log").read_text()
grown = log.replace("test result: ok. 122 passed;", "test result: ok. 123 passed;").replace(
    "running 122 tests\n", f"running 123 tests\ntest {NEW_TEST} ... ok\n")
cases = {
    "root's log alone (new test missing)": (log, 1),
    "root's log plus the new test": (grown, 0),
    "one test failed": (grown.replace(f"test {passed[0]} ... ok", f"test {passed[0]} ... FAILED"), 1),
    "summary still counts 122": (grown.replace("123 passed;", "122 passed;"), 1),
    "one test ignored": (grown.replace(f"test {passed[1]} ... ok", f"test {passed[1]} ... ignored"), 1),
}
for name, (body, want) in cases.items():
    path = SCRATCH / f"log-{len(name)}-{want}.txt"
    path.write_text(body)
    got = run(HERE / "expect_build01.py", "tests", OUT / "expected-tests.txt", path).returncode
    check(got == want, f"tests check, {name}: exit {got}")
native, oracle = OLD / "native-facts01.tsv", OLD / "oracle-facts01.tsv"
check(run(HERE / "expect_build01.py", "facts", native, oracle, native, oracle).returncode == 0, "facts check passes on root's facts")
first_key = native.read_text().splitlines()[0].split("\t")[0]
changed_value = SCRATCH / "native-changed.tsv"
changed_value.write_text(native.read_text().replace(native.read_text().splitlines()[0], f"{first_key}\tplanted", 1))
check(run(HERE / "expect_build01.py", "facts", changed_value, oracle, native, oracle).returncode == 1, "facts check refuses a changed value")
repeated = SCRATCH / "native-repeated.tsv"
repeated.write_text(native.read_text() + native.read_text().splitlines()[0] + "\n")
check(run(HERE / "expect_build01.py", "facts", repeated, repeated, native, oracle).returncode == 1, "facts check refuses a repeated key")

# 4. build01.sh parses, finds the repository, and keeps root's cargo commands and flags.
script = (HERE / "build01.sh").read_text()
check(subprocess.run(["/bin/sh", "-n", str(HERE / "build01.sh")]).returncode == 0, "build01.sh parses (sh -n)")
up = subprocess.run(["/bin/sh", "-c", 'cd "$1/../../../../../../../.." && pwd -P', "_", str(HERE)], capture_output=True, text=True)
check(up.stdout.strip() == str(REPO), "build01.sh's eight levels up is the repository root")
root_cargo = [line.split(" --config ")[0] + ' --config "$BUDGET"' + line.split(".toml", 1)[1].split(" --config")[0]
              for line in (OLD / "build01.sh").read_text().splitlines() if line.startswith("cargo ") and "--config" in line]
root_fmt = [line for line in (OLD / "build01.sh").read_text().splitlines() if line.startswith("cargo fmt")]
check(len(root_cargo) == 3 and all(any(l.startswith(c) for l in script.splitlines()) for c in root_cargo + root_fmt),
      "root's fmt, test, build and Clippy commands appear unchanged except the --config variable")
print("RESULT", "PASS" if not FAILED else f"FAIL {FAILED}")
