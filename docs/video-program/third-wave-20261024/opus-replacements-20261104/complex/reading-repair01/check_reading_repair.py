"""Checks on reading-repair01. No Rust runs here: `cargo test` is the real check, and root runs it.

    python3 -I -B check_reading_repair.py REPO EMPTY_SCRATCH

Run from anywhere; paths in the output are relative. It writes only under EMPTY_SCRATCH.

It checks that:
- the base files, the dependencies and test-repair01's patch still have the recorded hashes;
- the patch regenerates byte for byte, applies to canonical, gives the candidate files, and
  reverses back to canonical;
- each chapter changes only in its import list and in one call, outside its tests; every existing
  test is still there byte for byte; the script and the storyboard's words do not change;
- rustfmt, by file name on the whole module tree, accepts the result;
- a Python mirror of the three new hold-before-clear tests and the storyboard test passes on the
  candidate and fails on each defect planted in a scratch copy of the text;
- the binder is in step, the validator passes, and the planted defects are all refused.

Reading text is not running code. The mirror checks what the Rust tests assert about the source,
not that the Rust compiles, that a card stays up in a rendered frame, or that anyone can read it.
"""

import hashlib
import importlib.util
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/complex_analysis_visual"
CHAPTERS = f"{CRATE}/src/chapters"
HERE = Path(__file__).resolve().parent
PATCH = "reading-repair01.patch"
MAP = f"{CRATE}/docs/narration-direction-map-oliver08.json"
WINDOW = "reading_window(&kit, stage, &line)?;"
ANY_SAY = "let line = say(&kit, stage, "
SITES = [
    ("04_4", "04_5", "exact", "reading_color"),
    ("05_8", "05_9", "four", "gallery"),
    ("07_2", "07_3", "exact", "one_point"),
    ("09_5", "09_6", "needs", "boundary"),
    ("10_5", "10_6", "exact", "rigidity"),
    ("11_6", "11_7", "squashed", "counting"),
    ("14_3", "14_4", "exact", "continuing"),
]
BREATH = 0.6
FAILURES = []


def report(ok, text):
    print(f"{'ok  ' if ok else 'FAIL'} {text}")
    if not ok:
        FAILURES.append(text)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def table(mod, name):
    block = re.search(rf"^pub const {name}\b.*?^\];", mod, re.M | re.S).group(0)
    return [(c, float(s)) for c, s in re.findall(r'^\s*\("(\d\d_\d+)",\s*([\d.]+)\),\s*$', block, re.M)]


def before_tests(text):
    return text[: text.index("#[cfg(test)]\nmod ")] if "#[cfg(test)]\nmod " in text else text


def test_functions(text):
    """Each `#[test]` function of a file, name to its bytes."""
    out = {}
    for m in re.finditer(r"#\[test\]\n\s*fn (\w+)\(\)", text):
        out[m.group(1)] = text[m.start() : m.start() + text[m.start() :].index("\n    }\n")]
    return out


# ---------------------------------------------------------------- the Rust tests, in Python


def each_card_outlasts(mod, files):
    """`each_card_outlasts_its_reading_window`. Returns the first failure, or None."""
    if [c for c, _ in table(mod, "READING_WINDOWS")] != [s[0] for s in SITES]:
        return "the table and the sites differ"
    for cue, next_cue, card, name in SITES:
        source = files[name]
        if cue[:2] != next_cue[:2]:
            return f"{cue}: next cue in another chapter"
        if source.count(WINDOW) != 1:
            return f"{cue}: {source.count(WINDOW)} windows in the file"
        here = source.find(f'{ANY_SAY}"{cue}")?;')
        then = source.find(f'{ANY_SAY}"{next_cue}")?;')
        shown = source.find(f"let {card} = precisely_now(")
        held = source.find(WINDOW)
        if min(here, then, shown, held) < 0 or not here < shown < held < then:
            return f"{cue}: out of order"
        if ANY_SAY in source[here + 1 : then]:
            return f"{cue}: {next_cue} is not next"
        if source[held + len(WINDOW) : then].strip():
            return f"{cue}: {next_cue} waits on more"
        between = source[shown:held]
        if "fade_out" in between or "remove_from_scene" in between:
            return f"{cue}: cleared early"
        rest = source[then + 1 :]
        at = rest.find(ANY_SAY)
        cleared_by = rest if at < 0 else rest[:at]
        if "fade_out_all" not in cleared_by:
            return f"{cue}: {next_cue} clears nothing"
        if card not in re.split(r"[^\w]+", cleared_by):
            return f"{cue}: {next_cue} does not clear {card}"
    return None


def windows_short(mod, script):
    """`reading_windows_stay_short_and_never_delay_an_answer`."""
    rows = table(mod, "READING_WINDOWS")
    asked = {c for c, _ in table(mod, "PREDICTS")}
    for cue, seconds in rows:
        if cue not in script:
            return f"{cue} is not a cue"
        if not (seconds >= 1.0 and seconds + BREATH < 4.0):
            return f"{cue}: {seconds} s"
        if cue in asked:
            return f"{cue}: a window after a question delays its answer"
    if any(not a[0] < b[0] for a, b in zip(rows, rows[1:])):
        return "not sorted"
    return None


def helper_in_order(draw):
    """`the_reading_window_waits_in_order_and_clears_nothing`."""
    head = draw.find("pub fn reading_window(")
    if head < 0:
        return "no helper"
    body = draw[head : head + draw[head:].index("\n}\n")]
    at = [body.find(x) for x in ("wait_until(stage, line.end())?;", "kit.hold(stage)?;", "pause(stage, seconds)")]
    if min(at) < 0 or not at[0] < at[1] < at[2]:
        return "out of order"
    if not body.rstrip().endswith("pause(stage, seconds)"):
        return "the pause is not last"
    for touch in ("fade", "remove_from_scene", "clear_updaters", "kit.mark", "kit.listen", "music_breath"):
        if touch in body:
            return f"touches {touch}"
    return None


def storyboard_agrees(mod, board, parse):
    """`every_reading_window_is_in_the_storyboard`, on the repository's own parser."""
    planned = []
    for chapter in board:
        for beat in chapter["rows"]:
            m = re.fullmatch(r"read (.+) s", beat["hold"].strip())
            if m:
                planned.append((beat["id"], float(m.group(1))))
    built = table(mod, "READING_WINDOWS")
    return None if planned == built else f"{built} against {planned}"


# ---------------------------------------------------------------- the checks


def main():
    repo, scratch = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if scratch.exists() and any(scratch.iterdir()):
        sys.exit("EMPTY_SCRATCH is not empty")
    scratch.mkdir(parents=True, exist_ok=True)
    baseline = json.loads((HERE / "baseline.json").read_text())
    patched = list(baseline["patched_files"])
    names = [s[3] for s in SITES]

    print("# baseline")
    for path, want in baseline["patched_files"].items():
        report(sha(repo / path) == want, f"canonical base hash {path}")
    for path, want in baseline["read_dependencies"].items():
        report(sha(repo / path) == want, f"dependency unchanged {path}")
    report(sha(repo / CRATE / "script.tsv") == baseline["script_tsv"], "script.tsv hash")

    print("# patch")
    diff = []
    for path in patched:
        r = run(["diff", "-u", "--label", f"a/{path}", "--label", f"b/{path}", str(repo / path), str(HERE / path)])
        report(r.returncode == 1, f"differs {path}")
        diff.append(r.stdout)
    report("".join(diff) == (HERE / PATCH).read_text(), "patch regenerates byte for byte")
    root = scratch / "patch-root"
    for path in patched:
        (root / path).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(repo / path, root / path)
    r = run(["patch", "-p1", "-s", "-i", str(HERE / PATCH)], cwd=root)
    report(r.returncode == 0, "patch -p1 applies to canonical")
    report(all((root / p).read_bytes() == (HERE / p).read_bytes() for p in patched), "applied equals the candidate files")
    r = run(["patch", "-p1", "-R", "-s", "-i", str(HERE / PATCH)], cwd=root)
    report(r.returncode == 0, "patch -R reverses")
    report(all((root / p).read_bytes() == (repo / p).read_bytes() for p in patched), "reversed equals canonical")
    copies = {p.relative_to(HERE).as_posix() for p in (HERE / CRATE).rglob("*") if p.is_file()}
    extra = copies - set(patched)
    report(
        extra == {f"{CRATE}/narration_units.json", MAP},
        f"nothing else mirrors the crate but the two bound files: {sorted(extra - {f'{CRATE}/narration_units.json', MAP})}",
    )

    print("# scope")
    canon = {p: (repo / p).read_text() for p in patched}
    cand = {p: (HERE / p).read_text() for p in patched}
    for name in names:
        path = f"{CHAPTERS}/{name}.rs"
        a, b = canon[path], cand[path]
        report(a.count("kit.hold(stage)?;") - b.count("kit.hold(stage)?;") == 1 and b.count(WINDOW) == 1, f"{name}: one kit.hold became the window")
        undone = b.replace(WINDOW, "kit.hold(stage)?;")
        squeeze = lambda t: re.sub(r"\s+", " ", t)
        without = squeeze(undone).replace("reading_window, ", "")
        report(without == squeeze(a), f"{name}: otherwise only the import list changes")
    strings = lambda t: re.findall(r'"(?:[^"\\]|\\.)*"', before_tests(t))
    for path in patched:
        if path.endswith(".rs"):
            lost = [s for s in strings(canon[path]) if s not in strings(cand[path])]
            report(not lost, f"{Path(path).name}: no string in rendered code lost or changed")
            old, new = test_functions(canon[path]), test_functions(cand[path])
            report(all(new.get(k) == v for k, v in old.items()), f"{Path(path).name}: all {len(old)} existing tests byte for byte; {len(new) - len(old)} new")
    drawn = before_tests(cand[f"{CHAPTERS}/draw.rs"])
    helper = drawn[drawn.index("/// A reading window (") : drawn.index("/// A lower third (")]
    report(drawn.replace(helper, "") == before_tests(canon[f"{CHAPTERS}/draw.rs"]), "draw.rs: outside its tests, only the helper is added")
    listed = before_tests(cand[f"{CHAPTERS}/mod.rs"])
    block = listed[listed.index("\n/// Seconds the picture holds still") : listed.index('    ("14_3", 2.5),\n];\n') + len('    ("14_3", 2.5),\n];\n')]
    report(listed.replace(block, "") == before_tests(canon[f"{CHAPTERS}/mod.rs"]), "mod.rs: outside its tests, only READING_WINDOWS is added")
    report(
        before_tests(cand[f"{CRATE}/src/main.rs"]).replace("the prediction and the reading windows", "and the prediction windows").replace("the registry, the script, and", "the registry, the script and")
        == before_tests(canon[f"{CRATE}/src/main.rs"]),
        "main.rs: outside its tests, one doc comment",
    )

    print("# the storyboard's words")
    spec = importlib.util.spec_from_file_location("storyboard_tool", repo / "tools" / "storyboard.py")
    tool = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tool)
    old_board = tool.parse(repo / f"{CRATE}/docs/storyboard.md")
    new_board = tool.parse(HERE / f"{CRATE}/docs/storyboard.md")
    words = lambda board: [(b["id"], b["words"]) for c in board for b in c["rows"]]
    report(words(old_board) == words(new_board), "every beat's words are identical")
    changed = [
        (x["id"], x["hold"], y["hold"])
        for c, d in zip(old_board, new_board)
        for x, y in zip(c["rows"], d["rows"])
        if x != y
    ]
    report([c[0] for c in changed] == [s[0] for s in SITES], f"7 beats change: {[f'{c[0]} {c[2]!r}' for c in changed]}")
    slug = "complex_analysis_visual"
    old_problems, old_report = tool.check(slug, old_board)
    new_problems, new_report = tool.check(slug, new_board)
    report(old_problems == new_problems, f"tools/storyboard.py check: same problems before and after: {old_problems}")
    report(old_report == new_report, "its estimate does not move: it counts no `read N s`, so it undercounts these by 16.5 s")
    report(tool.script_text(slug, old_board) == tool.script_text(slug, new_board), "the script it would generate is byte-identical")
    file_text = (repo / CRATE / "script.tsv").read_text()
    generated = tool.script_text(slug, new_board)
    same_rows = [r for r in file_text.splitlines() if not r.startswith("#")] == [r for r in generated.splitlines() if not r.startswith("#")]
    print(f"     note: its `script.tsv is stale` on canonical is the header comment only: cue rows equal = {same_rows}")

    print("# rustfmt, whole module tree (canonical + test-repair01 + this overlay)")
    tree = scratch / "fmt"
    shutil.copytree(repo / CRATE / "src", tree / "src")
    for overlay in (HERE.parent / "test-repair01" / CRATE, HERE / CRATE):
        for f in overlay.rglob("*.rs"):
            shutil.copyfile(f, tree / f.relative_to(overlay))
    r = run(["rustfmt", "--check", "--edition", "2024", "--config-path", str(repo / "rustfmt.toml"), str(tree / "src/main.rs")])
    report(r.returncode == 0, "rustfmt --check")
    if r.returncode:
        print(r.stdout[:3000])

    print("# the new Rust tests, mirrored on the candidate text")
    mod = cand[f"{CHAPTERS}/mod.rs"]
    draw = cand[f"{CHAPTERS}/draw.rs"]
    files = {n: cand[f"{CHAPTERS}/{n}.rs"] for n in names}
    script = dict(r.split("\t", 1) for r in file_text.splitlines() if r and not r.startswith("#"))

    def board_from(text):
        path = scratch / "board.md"
        path.write_text(text)
        return tool.parse(path)

    board_text = cand[f"{CRATE}/docs/storyboard.md"]
    report(each_card_outlasts(mod, files) is None, f"each_card_outlasts_its_reading_window: {each_card_outlasts(mod, files)}")
    report(windows_short(mod, script) is None, f"reading_windows_stay_short_and_never_delay_an_answer: {windows_short(mod, script)}")
    report(helper_in_order(draw) is None, f"the_reading_window_waits_in_order_and_clears_nothing: {helper_in_order(draw)}")
    agreed = storyboard_agrees(mod, board_from(board_text), None)
    report(agreed is None, f"every_reading_window_is_in_the_storyboard: {agreed}")
    for name in ("each_card_outlasts_its_reading_window", "reading_windows_stay_short_and_never_delay_an_answer"):
        report(f"fn {name}()" in mod, f"mod.rs holds {name}")
    report("fn the_reading_window_waits_in_order_and_clears_nothing()" in draw, "draw.rs holds its test")
    report("fn every_reading_window_is_in_the_storyboard()" in cand[f"{CRATE}/src/main.rs"], "main.rs holds its test")

    print("# planted defects, each must fail the mirror")
    say = lambda c: f'{ANY_SAY}"{c}")?;'

    def card_plant(name, old, new):
        changed_files = dict(files)
        assert changed_files[name].count(old) == 1, (name, old)
        changed_files[name] = changed_files[name].replace(old, new)
        return each_card_outlasts(mod, changed_files)

    plants = [
        ("the window after the next say", card_plant("boundary", f"{WINDOW}\n        {say('09_6')}", f"{say('09_6')}\n        {WINDOW}")),
        ("a fade between window and next say", card_plant("rigidity", f"{WINDOW}\n", f"{WINDOW}\n        fade_out_all(stage, &[exact], 0.3)?;\n")),
        ("the card cleared before the window", card_plant("rigidity", f"        {WINDOW}\n", f"        fade_out_all(stage, &[exact], 0.3)?;\n        {WINDOW}\n")),
        ("the window removed", card_plant("gallery", WINDOW, "kit.hold(stage)?;")),
        ("the card not cleared by the next cue", card_plant("rigidity", "listed[1], not_in, exact]", "listed[1], not_in]")),
        ("no such card before the window", card_plant("counting", "let squashed = precisely_now(", "let squashed_ = precisely_now(")),
        ("a table row removed", each_card_outlasts(mod.replace('    ("05_8", 2.0),\n', ""), files)),
        ("a window too short", windows_short(mod.replace('("10_5", 3.0)', '("10_5", 0.5)'), script)),
        ("a window at the 4 s limit", windows_short(mod.replace('("10_5", 3.0)', '("10_5", 3.4)'), script)),
        ("a window on a question", windows_short(mod.replace('("10_5", 3.0)', '("10_4", 3.0)'), script)),
        ("rows out of order", windows_short(mod.replace('("04_4", 2.5)', '("06_4", 2.5)'), script)),
        ("helper: the pause first", helper_in_order(draw.replace("    wait_until(stage, line.end())?;\n    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    pause(stage, seconds)?;\n    wait_until(stage, line.end())?;\n    kit.hold(stage)\n"))),
        ("helper: the breath before the voice", helper_in_order(draw.replace("    wait_until(stage, line.end())?;\n    kit.hold(stage)?;\n", "    kit.hold(stage)?;\n    wait_until(stage, line.end())?;\n"))),
        ("helper: gains a fade", helper_in_order(draw.replace("    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    kit.hold(stage)?;\n    fade_out_all(stage, &[], 0.1)?;\n    pause(stage, seconds)\n"))),
        ("storyboard: a row gone", storyboard_agrees(mod, board_from(board_text.replace("| live readout | read 2 s |", "| live readout | |")), None)),
        ("storyboard: other seconds", storyboard_agrees(mod, board_from(board_text.replace("| live readout | read 2 s |", "| live readout | read 2.5 s |")), None)),
    ]
    for what, caught in plants:
        report(caught is not None, f"caught: {what}: {caught}")

    print("# the binder, the validator and their planted defects")
    crate = scratch / "crate"
    shutil.copytree(tree / "src", crate / "src")
    (crate / "docs").mkdir()
    shutil.copyfile(HERE / CRATE / "docs/storyboard.md", crate / "docs/storyboard.md")
    shutil.copyfile(repo / CRATE / "script.tsv", crate / "script.tsv")
    python = [sys.executable, "-I", "-B"]
    r = run([*python, str(HERE / "bind_oliver_plan.py"), str(repo), str(crate), str(HERE), "--check"])
    report(r.returncode == 0, f"binder --check: {r.stdout.splitlines()[0] if r.stdout else r.stderr[:300]}")
    r = run([*python, str(HERE / "validate_oliver_plan.py"), str(repo), str(crate), str(HERE), str(scratch / "validate")])
    report(r.returncode == 0, f"validator: {r.stdout.strip().splitlines()[-2] if r.returncode == 0 else (r.stdout + r.stderr)[-300:]}")
    r = run([*python, str(HERE / "planted_defects.py"), str(repo), str(crate), str(HERE), str(scratch / "planted")])
    report(r.returncode == 0, f"planted_defects: {r.stdout.strip().splitlines()[-1] if r.stdout else r.stderr[:300]}")

    print("# the map's reading rows, and what acceptance must measure")
    direction = json.loads((HERE / MAP).read_text())
    audit = json.loads((HERE / "source-audit.json").read_text())
    rows = {r["cue"]: r for r in audit["reading_windows"]}
    report(direction["hold_kinds"] == {"prediction": 13, "reading": 7}, f"hold_kinds {direction['hold_kinds']}")
    report(all("range_s" not in x for x in direction["overrides"] + direction["visual_holds"]), "no range_s anywhere in the map")
    fades = {"04_4": 0.7, "05_8": 0.4, "07_2": 1.1, "09_5": 0.4, "10_5": 0.9, "11_6": 1.5, "14_3": 0.8}
    print("     cue   chars read_s window  voice after the card's last sync must reach, s")
    for cue, seconds in table(mod, "READING_WINDOWS"):
        chars = rows[cue]["card_chars"]
        needed = chars / 15.0 + fades[cue] - BREATH - seconds
        print(f"     {cue}  {chars:5d} {chars / 15.0:6.2f} {seconds:6.1f}  {needed:5.2f}")
    print(f"     total {audit['counts']['reading_windows_total_s']} s; by chapter {audit['counts']['reading_windows_by_chapter_s']}")

    print("# hashes")
    for rel in [PATCH, "baseline.json", "check_reading_repair.py", "bind_oliver_plan.py", "validate_oliver_plan.py", "planted_defects.py", "source-audit.json", f"{CRATE}/narration_units.json", MAP, *patched]:
        print(f"{sha(HERE / rel)}  {rel}")
    print(f"\n{'ALL PASS' if not FAILURES else f'{len(FAILURES)} FAILED'}")
    sys.exit(1 if FAILURES else 0)


if __name__ == "__main__":
    main()
