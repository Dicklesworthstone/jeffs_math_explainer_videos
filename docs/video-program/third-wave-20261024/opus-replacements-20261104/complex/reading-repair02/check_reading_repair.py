"""Checks on reading-repair02. No Rust runs here: `cargo test` is the real check, and root runs it.

    python3 -I -B check_reading_repair.py REPO EMPTY_SCRATCH

Run from anywhere; paths in the output are relative. It writes only under EMPTY_SCRATCH.

It checks that:
- the base files, the dependencies, test-repair01's patch and the frozen reading-repair01 still have
  the recorded hashes;
- the patch regenerates byte for byte, applies to canonical, gives the candidate files, and
  reverses back to canonical;
- undoing the listed edits gives canonical back, so each chapter changes only where this says;
  every existing test is still there byte for byte, and reading-repair01's tests survive except
  the one the assignment replaced; the script and the storyboard's words do not change;
- every storyboard picture note that quotes a card now quotes its text;
- rustfmt, by file name on the whole module tree, accepts the result;
- a Python mirror of the changed and new Rust tests passes on the candidate and fails on each
  defect planted in a scratch copy of the text;
- the binder is in step, the validator passes, and the planted defects are all refused;
- the score facts recorded for root are still what the source says.

Reading text is not running code. The mirror checks what the Rust tests assert about the source,
not that the Rust compiles, that a card stays up in a rendered frame, or that anyone can read it.
chars/15 is a rule of thumb used as a lower bound, not a measurement of any reader.
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
PATCH = "reading-repair02.patch"
MAP = f"{CRATE}/docs/narration-direction-map-oliver09.json"
WINDOW = "reading_window(&kit, stage, &line)?;"
ANY_SAY = "let line = say(&kit, stage, "
VOICE_END = "wait_until(stage, line.end())?;"
SITES = [
    ("04_4", "04_5", "exact", "reading_color"),
    ("05_8", "05_9", "four", "gallery"),
    ("07_2", "07_3", "exact", "one_point"),
    ("08_5", None, "exact", "loop_sums"),
    ("09_5", "09_6", "needs", "boundary"),
    ("10_5", "10_6", "exact", "rigidity"),
    ("11_6", "11_7", "squashed", "counting"),
    ("11_8", None, "panel", "counting"),
    ("13_11", None, "note", "real_integral"),
    ("14_3", "14_4", "exact", "continuing"),
]
BREATH = 0.6
CARD_COMES_IN = 0.4
RULE_OF_THUMB = 15.0
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


def undo(text, edits):
    """Reverse listed edits, each of which must be there exactly once."""
    for new, old in edits:
        if text.count(new) != 1:
            raise SystemExit(f"listed edit is there {text.count(new)} times: {new[:80]!r}")
        text = text.replace(new, old)
    return text


# ---------------------------------------------------------------- the Rust tests, in Python


def rust_const(files, ref):
    module, name = ref.split("::")
    source = files[module]
    one = re.search(rf'const {name}: &str =\s*"((?:[^"\\]|\\.)*)"', source)
    if one:
        return [one.group(1)]
    many = re.search(rf"const {name}: \[&str; \d+\] = \[(.*?)\];", source, re.S)
    return re.findall(r'"((?:[^"\\]|\\.)*)"', many.group(1))


def to_read(mod, files):
    """`TO_READ`, from mod.rs's tests, its constants resolved from the chapter files."""
    block = mod[mod.index("const TO_READ:") : mod.index("const READING_RULE_OF_THUMB")]

    def texts(raw):
        out = []
        for literal, ref in re.findall(r'"((?:[^"\\]|\\.)*)"|(\w+::\w+)', raw):
            out.extend([literal] if not ref else rust_const(files, ref))
        return out

    rows = []
    for m in re.finditer(r"ToRead \{\n(.*?)\n        \},", block, re.S):
        body = m.group(1)
        rows.append(
            {
                "cue": re.search(r'cue: "(\d\d_\d+)"', body).group(1),
                "shows": texts(re.search(r"shows: (&.*?),\n\s+unspoken", body, re.S).group(1)),
                "unspoken": texts(re.search(r"unspoken: (&.*?),\n\s+after_voice", body, re.S).group(1)),
                "after_voice": re.search(r"after_voice: (true|false)", body).group(1) == "true",
                "closing_breath": float(re.search(r"closing_breath: ([\d.]+)", body).group(1)),
            }
        )
    return rows


def rule_of_thumb_s(parts):
    return sum(len(p) for p in parts) / RULE_OF_THUMB


def clean_s(seconds, read):
    return BREATH - (CARD_COMES_IN if read["after_voice"] else 0.0) + seconds + read["closing_breath"]


def each_card_outlasts(mod, files):
    """`each_card_outlasts_its_reading_window`. Returns the first failure, or None."""
    if [c for c, _ in table(mod, "READING_WINDOWS")] != [s[0] for s in SITES]:
        return "the table and the sites differ"
    for (cue, next_cue, card, name), read in zip(SITES, to_read(mod, files)):
        source = files[name]
        here = source.find(f'{ANY_SAY}"{cue}")?;')
        if here < 0:
            return f"{cue}: no say"
        at = source[here:].find(f"let {card} = ")
        window = source[here:].find(WINDOW)
        if at < 0 or window < 0:
            return f"{cue}: no card or no window after the say"
        shown, held = here + at, here + window
        if not shown < held:
            return f"{cue}: out of order"
        if read["after_voice"]:
            arrives = shown
            if cue == "13_11":
                over = source[here:].find("precisely_over(stage, said, note)?;")
                if over < 0:
                    return f"{cue}: the whole card is never brought in"
                arrives = here + over
            if VOICE_END not in source[here:arrives]:
                return f"{cue}: comes under the voice"
            if not arrives < held:
                return f"{cue}: the window starts before the card is up"
        between = source[shown:held]
        if "fade_out" in between or "remove_from_scene" in between:
            return f"{cue}: cleared early"
        after = source[held + len(WINDOW) :]
        if next_cue is not None:
            if cue[:2] != next_cue[:2]:
                return f"{cue}: next cue in another chapter"
            there = source[here:].find(f'{ANY_SAY}"{next_cue}")?;')
            if there < 0:
                return f"{cue}: {next_cue} is not said after it"
            then = here + there
            if not held < then:
                return f"{cue}: out of order"
            if ANY_SAY in source[here + 1 : then]:
                return f"{cue}: {next_cue} is not next"
            if source[held + len(WINDOW) : then].strip():
                return f"{cue}: {next_cue} waits on more"
            rest = source[then + 1 :]
            at = rest.find(ANY_SAY)
            cleared_by = rest if at < 0 else rest[:at]
            if "fade_out_all" not in cleared_by:
                return f"{cue}: {next_cue} clears nothing"
            if card not in re.split(r"[^\w]+", cleared_by):
                return f"{cue}: {next_cue} does not clear {card}"
        else:
            breath = f"kit.music_breath(stage, {read['closing_breath']:.1f})?;"
            if not after.lstrip().startswith(breath):
                return f"{cue}: the window is not right before {breath}"
            closing = after[: after.index("kit.finish(stage)")]
            if ANY_SAY in closing:
                return f"{cue}: is not its chapter's last cue"
            if "fade_out_all" not in closing:
                return f"{cue}: the closing clears nothing"
            if card not in re.split(r"[^\w]+", closing):
                return f"{cue}: the closing does not clear {card}"
    return None


def leave_time(mod, files, script):
    """`reading_windows_leave_time_for_what_the_voice_leaves_out`."""
    rows = table(mod, "READING_WINDOWS")
    reads = to_read(mod, files)
    asked = {c for c, _ in table(mod, "PREDICTS")}
    if [c for c, _ in rows] != [r["cue"] for r in reads]:
        return "the table and TO_READ differ"
    for (cue, seconds), read in zip(rows, reads):
        if cue not in script:
            return f"{cue} is not a cue"
        if cue in asked:
            return f"{cue}: a window after a question delays its answer"
        for piece in read["unspoken"]:
            if not any(piece in s for s in read["shows"]):
                return f"{cue}: {piece!r} is not on screen"
        if read["after_voice"] and read["unspoken"] != read["shows"] and cue != "13_11":
            return f"{cue}: comes after the voice, so all of it is unspoken"
        clean = clean_s(seconds, read)
        if clean < rule_of_thumb_s(read["unspoken"]):
            return f"{cue}: {clean:.2f} s clean is under {rule_of_thumb_s(read['unspoken']):.2f} s for what the voice leaves out"
        if clean > 2.0 * rule_of_thumb_s(read["shows"]):
            return f"{cue}: {clean:.2f} s is past twice the rule of thumb for the whole text"
    if any(not a[0] < b[0] for a, b in zip(rows, rows[1:])):
        return "not sorted"
    return None


def honest_adds_one_clause(mod, files):
    """`the_honest_note_only_adds_the_clause_nobody_says`."""
    whole = rust_const(files, "real_integral::HONEST")[0]
    said = rust_const(files, "real_integral::HONEST_SAID")[0]
    if not whole.startswith(said):
        return "the part said is not the start of the whole"
    reads = to_read(mod, files)
    if len(reads[8]["unspoken"]) != 1:
        return "13_11 leaves out more than one clause"
    return None if whole[len(said) :] == "; " + reads[8]["unspoken"][0] else "the whole adds something else"


def helper_in_order(draw):
    """`the_reading_window_waits_in_order_and_clears_nothing`."""
    head = draw.find("pub fn reading_window(")
    if head < 0:
        return "no helper"
    body = draw[head : head + draw[head:].index("\n}\n")]
    at = [body.find(x) for x in (VOICE_END, "kit.hold(stage)?;", "pause(stage, seconds)")]
    if min(at) < 0 or not at[0] < at[1] < at[2]:
        return "out of order"
    if not body.rstrip().endswith("pause(stage, seconds)"):
        return "the pause is not last"
    for touch in ("fade", "remove_from_scene", "clear_updaters", "kit.mark", "kit.listen", "music_breath"):
        if touch in body:
            return f"touches {touch}"
    return None


def growing_card_covers(draw):
    """`a_growing_card_covers_before_it_clears`."""
    head = draw.find("pub fn precisely_over(")
    if head < 0:
        return "no helper"
    body = draw[head : head + draw[head:].index("\n}\n")]
    comes, goes = body.find("fade_in_all(stage, &[whole], 0.3)?;"), body.find("fade_out_all(stage, &[said], 0.1)")
    return None if 0 <= comes < goes else "it clears before it covers"


def storyboard_agrees(mod, board):
    """`every_reading_window_is_in_the_storyboard`, on the repository's own parser."""
    planned = []
    for chapter in board:
        for beat in chapter["rows"]:
            m = re.fullmatch(r"read (.+?) s(?:, hold)?", beat["hold"].strip())
            if m:
                planned.append((beat["id"], float(m.group(1))))
    built = table(mod, "READING_WINDOWS")
    return None if planned == built else f"{built} against {planned}"


def beats_note(board, cue):
    return next(b["picture"] for c in board for b in c["rows"] if b["id"] == cue)


# ---------------------------------------------------------------- the checks


def main():
    repo, scratch = Path(sys.argv[1]).resolve(), Path(sys.argv[2]).resolve()
    if scratch.exists() and any(scratch.iterdir()):
        sys.exit("EMPTY_SCRATCH is not empty")
    scratch.mkdir(parents=True, exist_ok=True)
    baseline = json.loads((HERE / "baseline.json").read_text())
    patched = list(baseline["patched_files"])
    names = sorted({s[3] for s in SITES})

    print("# baseline")
    for path, want in baseline["patched_files"].items():
        report(sha(repo / path) == want, f"canonical base hash {path}")
    for path, want in baseline["read_dependencies"].items():
        report(sha(repo / path) == want, f"dependency unchanged {path}")
    report(sha(repo / CRATE / "script.tsv") == baseline["script_tsv"], "script.tsv hash")
    for rel, want in baseline["bound_with"].items():
        report(sha(HERE.parent / rel) == want, f"unchanged: {rel}")
    frozen = {p.relative_to(HERE.parent).as_posix(): sha(p) for p in sorted((HERE.parent / "reading-repair01").rglob("*")) if p.is_file()}
    report(frozen == baseline["reading_repair01_frozen"], f"reading-repair01 still frozen: all {len(frozen)} files have the recorded hashes")

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
    hold = "kit.hold(stage)?;"
    # the edits of each chapter, each as (what the candidate has, what canonical has)
    edits = {
        "gallery": [(WINDOW, hold)],
        "one_point": [(WINDOW, hold)],
        "boundary": [(WINDOW, hold)],
        "rigidity": [(WINDOW, hold)],
        "continuing": [(WINDOW, hold)],
        "reading_color": [
            (WINDOW, hold),
            (
                "        // the voice is still asking the eye to count colours around the dot, so the card\n"
                "        // comes only when it has finished, and all of it is read in the window below\n"
                f"        {VOICE_END}\n        let exact = precisely_now(",
                "        let exact = precisely_now(",
            ),
        ],
        "loop_sums": [
            (
                f"{VOICE_END}\n"
                '        // the lemma behind "almost": no voice names it, and from "Neighbouring squares" on\n'
                "        // the voice and the picture are both on the shared edges, so it comes when they stop\n"
                "        let exact = precisely_now(&kit, stage, GOURSAT)?;\n"
                f"        {WINDOW}\n",
                f"{VOICE_END}\n",
            ),
            (
                "fade_in_all(stage, &[each_zero, each], 0.4)?;\n        line.wait_sentence(stage, 5)?;",
                "fade_in_all(stage, &[each_zero, each], 0.4)?;\n        let exact = precisely_now(&kit, stage, GOURSAT)?;\n        line.wait_sentence(stage, 5)?;",
            ),
        ],
        "counting": [
            (
                "        // how the output is drawn has nothing to do with the minus three z the voice is on,\n"
                "        // so the card comes only when it has finished\n"
                f"        {VOICE_END}\n        let squashed = precisely_now(",
                "        let squashed = precisely_now(",
            ),
            (f"{WINDOW}\n        {ANY_SAY}\"11_7\")?;", f"{hold}\n        {ANY_SAY}\"11_7\")?;"),
            (
                "        // the panel has shared the screen with the running check for the whole cue; now the\n"
                "        // check has stopped, so the code can be read against what the voice just described\n"
                f"        {WINDOW}\n",
                "",
            ),
        ],
        "real_integral": [
            (
                "/// As much of [`HONEST`] as 13_11's voice carries: on screen while it speaks, so the warning\n"
                "/// the voice never says arrives on its own once it has stopped.\n"
                "pub const HONEST_SAID: &str =\n"
                '    "the integral of |f| over the line is finite, so the improper integral is the limit of the integral from −R to R";\n',
                "",
            ),
            (
                "let note = explainer_kit::props::precisely(&kit, stage, HONEST)?;\n"
                "        let said = precisely_lifted(&kit, stage, HONEST_SAID, note)?;\n"
                f"        {VOICE_END}\n"
                "        precisely_over(stage, said, note)?;\n"
                f"        {WINDOW}\n",
                f"let note = precisely_now(&kit, stage, HONEST)?;\n        {VOICE_END}\n",
            ),
            ("precisely_lifted, precisely_over, predict, reading_window, say,", "precisely_now, predict, say,"),
        ],
    }
    squeeze = lambda t: re.sub(r"\s+", " ", t)
    for name in names:
        path = f"{CHAPTERS}/{name}.rs"
        undone = undo(cand[path], edits[name])
        without = squeeze(undone).replace("reading_window, ", "")
        report(without == squeeze(canon[path]), f"{name}: undoing its {len(edits[name])} listed edits and the import gives canonical back")
    strings = lambda t: re.findall(r'"(?:[^"\\]|\\.)*"', before_tests(t))
    earlier = {p: (HERE.parent / "reading-repair01" / p) for p in patched}
    for path in patched:
        if path.endswith(".rs"):
            lost = [s for s in strings(canon[path]) if s not in strings(cand[path])]
            report(not lost, f"{Path(path).name}: no string in rendered code lost or changed")
            old, new = test_functions(canon[path]), test_functions(cand[path])
            report(all(new.get(k) == v for k, v in old.items()), f"{Path(path).name}: all {len(old)} existing tests byte for byte; {len(new) - len(old)} added")
    replaced = "reading_windows_stay_short_and_never_delay_an_answer"
    extended = {"each_card_outlasts_its_reading_window", "every_reading_window_is_in_the_storyboard"}
    gone, grown, same = [], [], []
    for path in patched:
        if path.endswith(".rs") and earlier[path].exists():
            mine = test_functions(earlier[path].read_text())
            new = test_functions(cand[path])
            for name, body in mine.items():
                if name in test_functions(canon[path]):
                    continue
                (same if new.get(name) == body else grown if name in new else gone).append(name)
    report(gone == [replaced], f"of reading-repair01's new tests only {replaced} is gone: {gone}")
    report(set(grown) == extended, f"two are extended for chapter-end windows: {sorted(grown)}")
    report(same == ["the_reading_window_waits_in_order_and_clears_nothing"], f"one is byte for byte: {same}")
    new_mod = test_functions(cand[f"{CHAPTERS}/mod.rs"])["each_card_outlasts_its_reading_window"]
    old_mod = test_functions((earlier[f"{CHAPTERS}/mod.rs"]).read_text())["each_card_outlasts_its_reading_window"]
    kept = re.findall(r'"\{cue\}: [^"]*"', old_mod)
    missing = [m for m in kept if m not in new_mod and m != '"{cue}: one window in the file"']
    report(not missing, f"each_card_outlasts keeps its assertions; the per-file count is now per cue: {missing}")
    drawn = before_tests(cand[f"{CHAPTERS}/draw.rs"])
    helper = drawn[drawn.index("/// A reading window (") : drawn.index('/// A lower third ("precisely:")')]
    growing = drawn[drawn.index("/// A lower third brought in now that") : drawn.index('/// "names to search:" brought in now')]
    report(drawn.replace(helper, "").replace(growing, "") == before_tests(canon[f"{CHAPTERS}/draw.rs"]), "draw.rs: outside its tests, only the three helpers are added")
    listed = before_tests(cand[f"{CHAPTERS}/mod.rs"])
    last_row = '    ("14_3", 5.5),\n];\n'
    block = listed[listed.index("\n/// Seconds the picture holds still") : listed.index(last_row) + len(last_row)]
    report(
        listed.replace(block, "").replace("    real_integral::HONEST_SAID,\n", "") == before_tests(canon[f"{CHAPTERS}/mod.rs"]),
        "mod.rs: outside its tests, only READING_WINDOWS and one TEXT row are added",
    )
    report(
        before_tests(cand[f"{CRATE}/src/main.rs"]).replace("the prediction and the reading windows", "and the prediction windows").replace("the registry, the script, and", "the registry, the script and")
        == before_tests(canon[f"{CRATE}/src/main.rs"]),
        "main.rs: outside its tests, one doc comment",
    )

    print("# the storyboard")
    spec = importlib.util.spec_from_file_location("storyboard_tool", repo / "tools" / "storyboard.py")
    tool = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(tool)
    old_board = tool.parse(repo / f"{CRATE}/docs/storyboard.md")
    new_board = tool.parse(HERE / f"{CRATE}/docs/storyboard.md")
    words = lambda board: [(b["id"], b["words"]) for c in board for b in c["rows"]]
    report(words(old_board) == words(new_board), "every beat's words are identical")
    changed = [(x["id"], y["hold"]) for c, d in zip(old_board, new_board) for x, y in zip(c["rows"], d["rows"]) if x != y]
    report(sorted(c[0] for c in changed) == sorted([s[0] for s in SITES] + ["07_5"]), f"11 beats change, the ten windows and 07_5's note: {changed}")
    held = lambda board: {b["id"]: b["hold"] for c in board for b in c["rows"]}
    report(sorted(k for k, x in held(old_board).items() if x != held(new_board)[k]) == sorted(s[0] for s in SITES), "only the ten windows change a hold cell")
    only_notes = all(
        {k for k in x if x[k] != y[k]} <= {"picture", "hold"} for c, d in zip(old_board, new_board) for x, y in zip(c["rows"], d["rows"])
    )
    report(only_notes, "only their picture and hold cells change")
    slug = "complex_analysis_visual"
    old_problems, old_report = tool.check(slug, old_board)
    new_problems, new_report = tool.check(slug, new_board)
    report(old_problems == new_problems, f"tools/storyboard.py check: same problems before and after: {old_problems}")
    moved = [(a, b) for a, b in zip(old_report, new_report) if a != b]
    report(
        len(old_report) == len(new_report)
        and len(moved) == 1
        and "keep identity" in moved[0][0]
        and moved[0][0].lstrip().startswith("[x]")
        and moved[0][1].lstrip().startswith("[x]")
        and "holomorphic" in beats_note(new_board, "09_5"),
        "its report moves on one line only, a row already ticked: its `morph` pattern matches the word holomorphic in 09_5's quoted card",
    )
    print("     note: so its estimate lines do not move: it counts no `read N s`, and undercounts these windows by 57 s")
    report(tool.script_text(slug, old_board) == tool.script_text(slug, new_board), "the script it would generate is byte-identical")
    file_text = (repo / CRATE / "script.tsv").read_text()
    generated = tool.script_text(slug, new_board)
    same_rows = [r for r in file_text.splitlines() if not r.startswith("#")] == [r for r in generated.splitlines() if not r.startswith("#")]
    print(f"     note: its `script.tsv is stale` on canonical is the header comment only: cue rows equal = {same_rows}")
    files = {n: cand[f"{CHAPTERS}/{n}.rs"] for n in names}
    beats = {b["id"]: b for c in new_board for b in c["rows"]}
    quoted = [("04_4", "reading_color::NEAR_ZERO"), ("05_8", "gallery::FOUR"), ("07_2", "one_point::CRITICAL"), ("08_5", "loop_sums::GOURSAT"), ("09_5", "boundary::NEEDS"), ("10_5", "rigidity::EXACT"), ("11_6", "counting::SQUASHED"), ("14_3", "continuing::PATH_EXACT")]
    for cue, ref in quoted:
        text = rust_const(files, ref)[0]
        report(text in beats[cue]["picture"].replace("\\|", "|"), f"{cue}: its picture note quotes {ref.split('::')[1]} exactly")
    honest = beats["13_11"]["picture"].replace("\\|", "|")
    said = rust_const(files, "real_integral::HONEST_SAID")[0]
    whole = rust_const(files, "real_integral::HONEST")[0]
    report(said in honest and whole[len(said) :] in honest, "13_11: its picture note quotes both steps of the card")
    report("z₀" not in beats["04_4"]["picture"], "04_4: its picture note no longer says z₀, which the card never did")
    one_point = cand[f"{CHAPTERS}/one_point.rs"]
    report(
        "z₀" not in beats["07_5"]["picture"] and "z_0" not in one_point and "z₀" not in one_point,
        "07_5: its picture note no longer says z₀, which chapter 07 never draws; the voice says the point one",
    )
    keep = {"06_1": ("one_derivative", "z_0 = 1 + i"), "06_3": ("one_derivative", "z_0 = 1 + i"), "12_3": ("residues", "z - z_0")}
    still = [b["id"] for c in new_board for b in c["rows"] if "z₀" in b["picture"]]
    report(sorted(still) == sorted(keep), f"z₀ stays in the notes of {still} only")
    for cue, (name, drawn) in keep.items():
        report(drawn in (repo / CHAPTERS / f"{name}.rs").read_text(), f"{cue}: keeps z₀, because {name}.rs draws {drawn}")

    print("# rustfmt, whole module tree (canonical + this overlay)")
    tree = scratch / "fmt"
    shutil.copytree(repo / CRATE / "src", tree / "src")
    overlay = HERE / CRATE
    for f in overlay.rglob("*.rs"):
        shutil.copyfile(f, tree / f.relative_to(overlay))
    r = run(["rustfmt", "--check", "--edition", "2024", "--config-path", str(repo / "rustfmt.toml"), str(tree / "src/main.rs")])
    report(r.returncode == 0, "rustfmt --check")
    if r.returncode:
        print(r.stdout[:3000])

    print("# the changed and new Rust tests, mirrored on the candidate text")
    mod = cand[f"{CHAPTERS}/mod.rs"]
    draw = cand[f"{CHAPTERS}/draw.rs"]
    script = dict(r.split("\t", 1) for r in file_text.splitlines() if r and not r.startswith("#"))

    def board_from(text):
        path = scratch / "board.md"
        path.write_text(text)
        return tool.parse(path)

    board_text = cand[f"{CRATE}/docs/storyboard.md"]
    results = [
        ("each_card_outlasts_its_reading_window", each_card_outlasts(mod, files)),
        ("reading_windows_leave_time_for_what_the_voice_leaves_out", leave_time(mod, files, script)),
        ("the_honest_note_only_adds_the_clause_nobody_says", honest_adds_one_clause(mod, files)),
        ("the_reading_window_waits_in_order_and_clears_nothing", helper_in_order(draw)),
        ("a_growing_card_covers_before_it_clears", growing_card_covers(draw)),
        ("every_reading_window_is_in_the_storyboard", storyboard_agrees(mod, board_from(board_text))),
    ]
    for name, failure in results:
        report(failure is None, f"{name}: {failure}")
    for name in ("each_card_outlasts_its_reading_window", "reading_windows_leave_time_for_what_the_voice_leaves_out", "the_honest_note_only_adds_the_clause_nobody_says"):
        report(f"fn {name}()" in mod, f"mod.rs holds {name}")
    report("3.3" not in mod and "seconds + explainer_kit::narration::BREATH < 4.0" not in mod, "no test bounds a window by the 4 s gap any more")
    for name in ("the_reading_window_waits_in_order_and_clears_nothing", "a_growing_card_covers_before_it_clears"):
        report(f"fn {name}()" in draw, f"draw.rs holds {name}")
    report("fn every_reading_window_is_in_the_storyboard()" in cand[f"{CRATE}/src/main.rs"], "main.rs holds its test")

    print("# planted defects, each must fail the mirror")
    say = lambda c: f'{ANY_SAY}"{c}")?;'

    def card_plant(name, old, new):
        changed_files = dict(files)
        assert changed_files[name].count(old) == 1, (name, old)
        changed_files[name] = changed_files[name].replace(old, new)
        return each_card_outlasts(mod, changed_files)

    def time_plant(*pairs):
        planted = mod
        for old, new in pairs:
            assert planted.count(old) == 1, old
            planted = planted.replace(old, new)
        return leave_time(planted, files, script)

    def source_plant(old, new):
        changed_files = dict(files)
        assert changed_files["real_integral"].count(old) == 1, old
        changed_files["real_integral"] = changed_files["real_integral"].replace(old, new)
        return honest_adds_one_clause(mod, changed_files)

    plants = [
        ("the window after the next say", card_plant("boundary", f"{WINDOW}\n        {say('09_6')}", f"{say('09_6')}\n        {WINDOW}")),
        ("a fade between window and next say", card_plant("rigidity", f"{WINDOW}\n", f"{WINDOW}\n        fade_out_all(stage, &[exact], 0.3)?;\n")),
        ("the card cleared before the window", card_plant("rigidity", f"        {WINDOW}\n", f"        fade_out_all(stage, &[exact], 0.3)?;\n        {WINDOW}\n")),
        ("the window removed", card_plant("gallery", WINDOW, "kit.hold(stage)?;")),
        ("the card not cleared by the next cue", card_plant("rigidity", "listed[1], not_in, exact]", "listed[1], not_in]")),
        ("no such card before the window", card_plant("counting", "let squashed = precisely_now(", "let squashed_ = precisely_now(")),
        ("a table row removed", each_card_outlasts(mod.replace('    ("05_8", 3.5),\n', ""), files)),
        ("a card promised after the voice comes under it", card_plant("reading_color", f"{VOICE_END}\n        let exact", "let exact")),
        ("13_11's clause comes under the voice", card_plant("real_integral", f"{VOICE_END}\n        precisely_over(stage, said, note)?;", f"precisely_over(stage, said, note)?;\n        {VOICE_END}")),
        ("a last cue's breath before the window", card_plant("real_integral", f"{WINDOW}\n        kit.music_breath(stage, 2.5)?;", f"kit.music_breath(stage, 2.5)?;\n        {WINDOW}")),
        ("a last cue's breath shorter than the clean time counts on", card_plant("loop_sums", "kit.music_breath(stage, 2.0)?;", "kit.music_breath(stage, 1.0)?;")),
        ("a last cue's card not cleared", card_plant("loop_sums", "each_zero, each, exact, lit_outer,", "each_zero, each, lit_outer,")),
        ("the code panel not cleared", card_plant("counting", "last.push(panel.group);\n", "")),
        ("the theorem back to a 3 s window", time_plant(('("10_5", 8.5)', '("10_5", 3.0)'))),
        ("the honest clause with no window and a short breath", time_plant(('("13_11", 5.0)', '("13_11", 0.1)'), ('cue: "13_11",\n            shows: &[real_integral::HONEST],\n            unspoken: &["for slowly decaying f that symmetric limit can exist when the integral does not"],\n            after_voice: true,\n            closing_breath: 2.5,', 'cue: "13_11",\n            shows: &[real_integral::HONEST],\n            unspoken: &["for slowly decaying f that symmetric limit can exist when the integral does not"],\n            after_voice: true,\n            closing_breath: 0.5,'))),
        ("a window far past the card's length", time_plant(('("05_8", 3.5)', '("05_8", 9.0)'))),
        ("a card after the voice claiming only part is unspoken", time_plant(('cue: "04_4",\n            shows: &[reading_color::NEAR_ZERO],\n            unspoken: &[reading_color::NEAR_ZERO],', 'cue: "04_4",\n            shows: &[reading_color::NEAR_ZERO],\n            unspoken: &["near a zero"],'))),
        ("unspoken text that is not on the card", time_plant(('unspoken: &["an open set containing the closed disk"]', 'unspoken: &["a neighbourhood of the disk"]'))),
        ("TO_READ out of step with the table", time_plant(('("09_5", 3.0)', '("09_6", 3.0)'))),
        ("a window on a question, both tables agreeing", time_plant(('("10_5", 8.5)', '("10_4", 8.5)'), ('cue: "10_5"', 'cue: "10_4"'))),
        ("rows out of order, both tables agreeing", time_plant(('("04_4", 7.5)', '("06_4", 7.5)'), ('cue: "04_4"', 'cue: "06_4"'))),
        ("a cue that does not exist, both tables agreeing", time_plant(('("05_8", 3.5)', '("05_99", 3.5)'), ('cue: "05_8"', 'cue: "05_99"'))),
        ("the honest card says more than the voice's part plus the clause", source_plant('the limit of the integral from −R to R; for', 'the limit of the integral from −R to R, always; for')),
        ("the part said is reworded away from the whole", source_plant('"the integral of |f| over the line is finite, so the improper integral is the limit of the integral from −R to R";', '"the integral of |f| is finite, so the improper integral is the limit of the integral from −R to R";')),
        ("helper: the pause first", helper_in_order(draw.replace(f"    {VOICE_END}\n    kit.hold(stage)?;\n    pause(stage, seconds)\n", f"    pause(stage, seconds)?;\n    {VOICE_END}\n    kit.hold(stage)\n"))),
        ("helper: the breath before the voice", helper_in_order(draw.replace(f"    {VOICE_END}\n    kit.hold(stage)?;\n", f"    kit.hold(stage)?;\n    {VOICE_END}\n"))),
        ("helper: gains a fade", helper_in_order(draw.replace("    kit.hold(stage)?;\n    pause(stage, seconds)\n", "    kit.hold(stage)?;\n    fade_out_all(stage, &[], 0.1)?;\n    pause(stage, seconds)\n"))),
        ("a growing card clears first", growing_card_covers(draw.replace("    fade_in_all(stage, &[whole], 0.3)?;\n    fade_out_all(stage, &[said], 0.1)\n", "    fade_out_all(stage, &[said], 0.1)?;\n    fade_in_all(stage, &[whole], 0.3)\n"))),
        ("storyboard: a row gone", storyboard_agrees(mod, board_from(board_text.replace("| live readout | read 3 s |", "| live readout | |")))),
        ("storyboard: other seconds", storyboard_agrees(mod, board_from(board_text.replace("| live readout | read 3 s |", "| live readout | read 2.5 s |")))),
        ("storyboard: a last cue's other seconds", storyboard_agrees(mod, board_from(board_text.replace("| lower third; arrows | read 5 s, hold |", "| lower third; arrows | read 4 s, hold |")))),
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

    print("# the map's reading rows, in source seconds. Nothing below is measured.")
    direction = json.loads((HERE / MAP).read_text())
    audit = json.loads((HERE / "source-audit.json").read_text())
    rows = {r["cue"]: r for r in audit["reading_windows"]}
    report(direction["hold_kinds"] == {"prediction": 13, "reading": 10}, f"hold_kinds {direction['hold_kinds']}")
    report(all("range_s" not in x for x in direction["overrides"] + direction["visual_holds"]), "no range_s anywhere in the map")
    reads = to_read(mod, files)
    print("     cue    arrives      window  breath   clean   unspoken@15  whole@15   next")
    for (cue, seconds), read in zip(table(mod, "READING_WINDOWS"), reads):
        row = rows[cue]
        clean = clean_s(seconds, read)
        report(abs(row["clean_s_in_source"] - clean) < 1e-9, f"{cue}: the binder's clean time agrees with the mirror's")
        print(f"     {cue:<6} {row['arrives']:<12} {seconds:5.1f} {read['closing_breath']:7.1f} {clean:7.2f} {rule_of_thumb_s(read['unspoken']):10.2f} {rule_of_thumb_s(read['shows']):9.2f}   {row['next_cue']}")
    print(f"     total {audit['counts']['reading_windows_total_s']} s; by chapter {audit['counts']['reading_windows_by_chapter_s']}")

    print("# for root: what the score and the cue sheets will make of these. Nothing shared is changed.")
    score = (repo / "crates/explainer_score/src/score.rs").read_text()
    identity = (repo / CRATE / "src/score.rs").read_text()
    facts = [
        ("pub const HOLD_MERGE: f64 = 4.0;", score, "score.rs HOLD_MERGE 4.0"),
        ("pub const MIN_PHRASE_GAP: f64 = 1.6;", score, "score.rs MIN_PHRASE_GAP 1.6: a gap this long gets a phrase"),
        ("const ROLL_GAP: f64 = 2.4;", score, "score.rs ROLL_GAP 2.4: a gap this long gets a chord roll first"),
        ("steady_bed: true,", identity, "this crate's identity sets steady_bed"),
    ]
    for marker, text, what in facts:
        report(marker in text, what)
    report("kit.sonify" not in "".join(files.values()), "none of the ten sites' chapter files carries a sonification, the one thing the duck curve still shapes under steady_bed")
    warned = [(c, r["expected_cue_sheet_warning"]) for c, r in rows.items() if r["expected_cue_sheet_warning"]]
    print(f"     {len(warned)} of {len(rows)} windows will warn in cue_sheets at its defaults (--max-gap 4.0, --max-tail 5.0):")
    for cue, text in warned:
        print(f"       {cue}: {text}")
    report(len(warned) == 9, "nine warnings are expected, six gaps and three tails; 09_5's 3.6 s is simply under 4 s, not exempt")

    print("# hashes")
    for rel in [PATCH, "from-reading-repair01.diff", "baseline.json", "check_reading_repair.py", "bind_oliver_plan.py", "validate_oliver_plan.py", "planted_defects.py", "source-audit.json", f"{CRATE}/narration_units.json", MAP, *patched]:
        print(f"{sha(HERE / rel)}  {rel}")
    print(f"\n{'ALL PASS' if not FAILURES else f'{len(FAILURES)} FAILED'}")
    sys.exit(1 if FAILURES else 0)


if __name__ == "__main__":
    main()
