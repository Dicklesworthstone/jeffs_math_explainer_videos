"""Prepare algebra repair02 inside the original guard's measured WORK tree.

Run under a distinct algebra-fixes-prepare02 guarded job; launch nothing here.
The original algebra-fixes-build02 job must not exist yet. Output is restricted
exactly to WORK/algebra-fixes-inputs02. Root then admits build02 separately with
the generated build02-inputs.json. All prior source/input pins remain unchanged.
"""
import argparse
import copy
import hashlib
import importlib.util
import json
import re
import shlex
import shutil
import sys
from pathlib import Path

sys.dont_write_bytecode = True

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[7]
N = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-expanded-algebra-20261103/native-stage08-root01"
OLD = N / "corner-label-contrast-repair01"
JOBS = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
OLD_JOB = JOBS / "algebra-corner-contrast-build01"
JOB = "algebra-fixes-build02"
OUTPUT = JOBS / "algebra-fixes-inputs02"
TOPIC = "abstract_algebra_expanded"
CRATE = f"explainer_corpus/{TOPIC}"
OLD_MAP = f"{CRATE}/docs/narration-direction-map-frederick-stage13.json"
NEW_MAP = f"{CRATE}/docs/narration-direction-map-frederick-stage14.json"
NEW_TEST = "cosets::tests::right_coset_links_touch_their_tiles_and_cross_no_tile_or_name"
POLICY = {"admission_gib": 10, "stop_free_gib": 8, "phase_growth_gib": 4, "work_cap_gib": 56,
          "target_growth_cap_gib": 20, "sample_seconds": 10}

PINS = {
    OLD / "build01-inputs.json": "810c6773b2502f3669de79720e345b8b605f2d0b80f1258ce6343d8687729aca",
    OLD / "build01.sh": "5aabe03e980b24c570472d10ecb487937ae96a0438c4d11f7f1e057023181e6c",
    OLD / "native-facts01.tsv": "15e93cc2b3efd60683d3d08121fea50ccc5bb70a0be13fbb8fd0b847f5e263c4",
    OLD / "oracle-facts01.tsv": "0e18012a018fcb4590044ebc8659f4e3691a637e7c72c221ab32f2e9482ff74b",
    OLD / "native-chapters01.txt": "2014cc7830e069368fb511d582ff35f5785b65bec7fae36b0ee3dad5fee095e5",
    OLD_JOB / "command.log": "75f868db68cd17aba914b6282eb622f7f6a323272ad35d16336ac63dc1877c56",
    OLD_JOB / "resource-receipt.json": "ae2391806ecc036889bd56ae26808ed72eaae947e2ea044791639460fd27a4b9",
    HERE / "before-cosets.rs": "ab0e0300537f163d4ea6e8b7a5da07949c7b2dd20f88af122aa4cb4555150b06",
    HERE / "before-permutations.rs": "092256d823983e8a36267c833b16eaf6d1e767d86d254a7253490906505a32f2",
    HERE / "cosets.rs": "63a62ffdf4a2bf8d7aac6e27b4ecb161e0d8440ff1b7ccdad919bccade441187",
    HERE / "permutations.rs": "a5472cd6a0ee583a97cc1986809a82ec32dcf9fe7a5d8901b47a3b05d7c3255c",
    HERE / "11_17-frederick.txt": "ed410300d6b2a1bea36f719d9f8da3eb6ef0837235cf338efff79f6c6e3ef3da",
}
INSTALL = {f"{CRATE}/src/cosets.rs": ("before-cosets.rs", "cosets.rs"),
           f"{CRATE}/src/permutations.rs": ("before-permutations.rs", "permutations.rs")}
CHANGED = set(INSTALL) | {f"{CRATE}/script.tsv", f"{CRATE}/narration_units.json"}
# The build files this folder contributes; build02.sh and expect_build01.py run inside the guard.
OWN = ["prepare_build02.py", "build02.sh", "expect_build01.py", *sorted(p.name for p in PINS if p.parent == HERE)]
SAY = re.compile(r'k\.say\(stage, "(\d\d_\d+)"\)')


def refuse(message):
    raise SystemExit(f"prepare_build02 refused: {message}")


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def tree(root):
    """Every file under root by relative path, refusing symlinks."""
    files = {}
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            refuse(f"symlink in workspace: {path}")
        if path.is_file():
            files[path.relative_to(root).as_posix()] = sha(path)
    return files


def json_text(data):
    return json.dumps(data, indent=2, ensure_ascii=False) + "\n"


def hold_line(lines, cue, marker):
    """The one line holding `marker` whose nearest earlier say names `cue`, or a refusal."""
    found = []
    for number, line in enumerate(lines, 1):
        if line.strip() != marker:
            continue
        said = next((m.group(1) for back in reversed(lines[:number - 1]) if (m := SAY.search(back))), None)
        if said == cue:
            found.append(number)
    if len(found) != 1:
        refuse(f"hold for {cue} is not one line: {found}")
    return found[0]


def check_root_build():
    for path, digest in PINS.items():
        if sha(path) != digest:
            refuse(f"pinned file changed: {path}")
    receipt = json.loads((OLD_JOB / "resource-receipt.json").read_text())
    manifest = OLD / "build01-inputs.json"
    if not (receipt.get("admitted") is True and receipt.get("exit") == 0
            and not receipt.get("stopped_for_resource_limit") and "error" not in receipt
            and receipt.get("policy") == POLICY and receipt.get("phase") == "native"
            and receipt.get("command") == ["sh", str(OLD / "build01.sh")]
            and receipt.get("inputs_sha256") == receipt.get("final_inputs_sha256") == sha(manifest)):
        refuse("root's algebra-corner-contrast-build01 receipt is not a clean admitted exit 0 of build01.sh")
    rows = json.loads(manifest.read_text())["files"]
    changed = [row["path"] for row in rows if sha(row["path"]) != row["sha256"]]
    if changed:
        refuse(f"{len(changed)} inputs of root's build changed since it passed, first {changed[:3]}")
    log = (OLD_JOB / "command.log").read_text()
    passed = re.findall(r"^test (\S+) \.\.\. ok$", log, re.M)
    if (len(passed) != len(set(passed)) or re.findall(r"^test \S+ \.\.\. (?!ok$)", log, re.M)
            or f"test result: ok. {len(passed)} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out" not in log
            or NEW_TEST in passed):
        refuse("root's build log does not show one clean test run")
    return rows, sorted(passed)


def budget_path():
    """The one --config file of root's build01.sh: the engine patch table, a manifest row."""
    found = set(re.findall(r"--config (\S+)", (OLD / "build01.sh").read_text()))
    if len(found) != 1:
        refuse(f"root's build01.sh names {len(found)} --config files")
    return Path(found.pop())


def rebind(work, passed):
    crate = work / CRATE
    for target, (before, after) in INSTALL.items():
        if sha(work / target) != PINS[HERE / before]:
            refuse(f"{target} in root's workspace is not the reviewed base")
        if (HERE / after).read_text().count("#[test]") - (HERE / before).read_text().count("#[test]") != (
                1 if target.endswith("cosets.rs") else 0):
            refuse(f"{after}: unexpected change in the number of tests")
        shutil.copyfile(HERE / after, work / target)
    if f"fn {NEW_TEST.rsplit('::', 1)[1]}()" not in (HERE / "cosets.rs").read_text():
        refuse("the new coset test is not in cosets.rs")

    text = (HERE / "11_17-frederick.txt").read_text().strip()
    if "\t" in text or "\n" in text:
        refuse("11_17 text must be one line")
    script_path = crate / "script.tsv"
    lines = script_path.read_text().splitlines(keepends=True)
    rows = [i for i, line in enumerate(lines) if line.startswith("11_17\t")]
    if len(rows) != 1:
        refuse("script.tsv must have one 11_17 row")
    old_text = lines[rows[0]].rstrip("\n").split("\t", 1)[1]
    lines[rows[0]] = f"11_17\t{text}\n"
    script_path.write_text("".join(lines))
    script_sha = sha(script_path)

    map_path = crate / "docs/narration-direction-map-frederick-stage13.json"
    stage13 = json.loads(map_path.read_text())
    if json_text(stage13) != map_path.read_text() or (crate / "docs" / Path(NEW_MAP).name).exists():
        refuse("stage13 map is not in the expected format, or stage14 already exists")
    stage14 = copy.deepcopy(stage13)
    stage14["scripts"][TOPIC] = script_sha
    moved = []
    for hold in stage14["visual_holds"]:
        if hold["path"] not in INSTALL:
            continue
        before, after = INSTALL[hold["path"]]
        old_lines = (HERE / before).read_text().splitlines()
        if hold["sha256"] != PINS[HERE / before] or hold_line(old_lines, hold["cue"], hold["marker"]) != hold["line"]:
            refuse(f"stage13 hold for {hold['cue']} does not match the reviewed base")
        line = hold_line((HERE / after).read_text().splitlines(), hold["cue"], hold["marker"])
        moved.append({"cue": hold["cue"], "path": hold["path"], "line": [hold["line"], line]})
        hold["sha256"], hold["line"] = PINS[HERE / after], line
    (work / NEW_MAP).write_text(json_text(stage14))

    units_path = crate / "narration_units.json"
    plan = json.loads(units_path.read_text())
    if json_text(plan) != units_path.read_text():
        refuse("narration_units.json is not in the expected format")
    units = [unit for unit in plan["units"] if unit["line_ids"] == ["11_17"]]
    contract = plan["direction_contract"]
    if len(units) != 1 or units[0]["text"] != old_text or contract["map"]["path"] != OLD_MAP:
        refuse("the plan does not bind the current 11_17 text and stage13 map")
    units[0]["text"] = text
    plan["script_sha256"] = contract["script_sha256"] = script_sha
    contract["map"] = {"path": NEW_MAP, "sha256": sha(work / NEW_MAP)}
    units_path.write_text(json_text(plan))

    # The workspace's own validator: exact unit texts, cue coverage, script hash, direction map.
    spec = importlib.util.spec_from_file_location("coherent_narration_algebra_fixes", work / "tools/coherent_narration.py")
    tool = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tool)
    tool.read_units(script_path, units_path, require_directions=True)
    return {"old_11_17": old_text, "new_11_17": text, "script_sha256": script_sha, "holds_moved": moved,
            "expected_tests": len(passed) + 1}


def main():
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("--out", type=Path, default=OUTPUT)
    out = parser.parse_args().out.resolve()
    if out != OUTPUT.resolve() or out.parent != JOBS.resolve():
        refuse(f"output must be the guarded WORK child {OUTPUT}")
    if OUTPUT.is_symlink() or JOBS.is_symlink():
        refuse("guarded output and WORK must not be symlinks")
    if not (REPO / "AGENTS.md").is_file() or Path.cwd().resolve() != REPO:
        refuse("run from the repository root")
    if out.exists() or (JOBS / JOB).exists():
        refuse(f"{out} or guard job {JOB} already exists; choose new names")
    rows, passed = check_root_build()
    budget = budget_path()
    if str(budget) not in {row["path"] for row in rows}:
        refuse("the engine patch table is not an input of root's build")
    base = tree(OLD / "workspace")
    listed = {row["path"]: row["sha256"] for row in rows}
    if any(listed.get(str(OLD / "workspace" / rel)) != digest for rel, digest in base.items()):
        refuse("root's workspace differs from the inputs its build recorded")

    out.mkdir(parents=True)
    work = out / "workspace"
    shutil.copytree(OLD / "workspace", work, symlinks=True)
    if tree(work) != base:
        refuse("the workspace copy is not exact")
    summary = rebind(work, passed)
    now = tree(work)
    if ({rel for rel in base if now.get(rel) != base[rel]} != CHANGED or set(now) - set(base) != {NEW_MAP}):
        refuse("the new workspace changes more than the four repaired files and stage14")

    (out / "expected-tests.txt").write_text("".join(f"{name}\n" for name in sorted([*passed, NEW_TEST])))
    (out / "build.env").write_text(f"BUDGET={shlex.quote(str(budget))}\n")
    # Root's rows stay as recorded (all re-hashed above); add only paths it did not list.
    known = {row["path"] for row in rows}
    extra = [HERE / name for name in OWN] + [path for path in PINS if path.parent != HERE]
    extra += [out / "expected-tests.txt", out / "build.env"] + [work / rel for rel in sorted(now)]
    files = rows + [{"path": str(path), "sha256": sha(path)} for path in extra if str(path) not in known]
    if len({row["path"] for row in files}) != len(files):
        refuse("duplicate manifest path")
    (out / "build02-inputs.json").write_text(json.dumps({"files": files}, indent=2) + "\n")
    summary.update(out=str(out), job=JOB, rows=len(files), root_rows=len(rows),
                   inputs_sha256=sha(out / "build02-inputs.json"))
    (out / "prepare-receipt.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(json.dumps(summary, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
