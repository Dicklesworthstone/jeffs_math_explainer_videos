"""Text checks on compile-repair01. This is not a compiler and proves nothing rustc would.

    python3 -I -B check_compile_repair.py REPO EMPTY_SCRATCH

Run from anywhere; paths in the output are relative. It writes only under EMPTY_SCRATCH,
except that it never touches canonical, the build log, or the delivered candidate files.

It checks that:
- the nine baseline files still have the hashes this patch was made against;
- the patch regenerates byte for byte, applies to canonical, gives the candidate files, and
  reverses back to canonical;
- nothing but those nine files is touched, and only in the lines the patch names;
- every site in root's build02 log is repaired, and the old forms are gone everywhere;
- no test, assertion, cue or fact was removed or changed;
- rustfmt, by file name on the whole module tree, accepts the result;
- each repair, undone on its own in a scratch copy, is caught by these same site checks.
"""

import hashlib
import json
import re
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/complex_analysis_visual"
CHAPTERS = f"{CRATE}/src/chapters"
HERE = Path(__file__).resolve().parent
FILES = ("draw", "closing", "the_bump", "one_derivative", "counting", "two_planes", "turning", "gallery", "loop_sums")
LOG = "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-opus-rebuild-build02/command.log"

GET_OLD = "crate::facts::facts().unwrap().get(key).unwrap()"
GET_NEW = "fact(key).unwrap()"
GUIDE_OLD = ("            DashedLine::new(g.p(x, g.y.0), g.p(x, g.y.1))\n", "                .build()\n        };")
GUIDE_NEW = ("            Ok(DashedLine::new(g.p(x, g.y.0), g.p(x, g.y.1))\n", "                .build()?)\n        };")

# One row per repair: log line of the error (None: not in the log, see the README), file, the
# text the candidate must hold, and the old text it must not.
REPAIRS = [
    (8, "one_derivative", "let body = polyline(pts.iter().map(|&z| p.p(z)))", "let body = polyline(pts.iter().map(|&z| p.p(z)).collect())"),
    (39, "one_derivative", "polyline(square_edge(Z0, sv, 30).iter().map(|&z| inp.p(z)))", "polyline(square_edge(Z0, sv, 30).iter().map(|&z| inp.p(z)).collect())"),
    (None, "one_derivative", "polyline(kept_size(sv).iter().map(|&w| out.p(w)))", "polyline(kept_size(sv).iter().map(|&w| out.p(w)).collect())"),
    (70, "closing", GUIDE_NEW[0], GUIDE_OLD[0]),
    (70, "closing", GUIDE_NEW[1], GUIDE_OLD[1]),
    (91, "draw", "stage.play(tween_linear(self.walked, 1.0, rt)?)?;\n        Ok(())", "stage.play(tween_linear(self.walked, 1.0, rt)?)\n"),
    (102, "draw", "stage.play(tween(self.blend, 1.0, rt)?)?;\n        Ok(())", "stage.play(tween(self.blend, 1.0, rt)?)\n"),
    (114, "draw", "stage.play(tween(self.enhance, e, rt)?)?;\n        Ok(())", "stage.play(tween(self.enhance, e, rt)?)\n"),
    (125, "draw", "use explainer_kit::{Facts, Frame2, Kit, Palette, Script, Timed, hex, tween, v};", "use explainer_kit::{Facts, Frame2, Kit, Palette, Script, hex, tween, v};"),
    (142, "draw", "use explainer_kit::{Facts, Frame2, Kit, Palette, Script, Timed, hex, tween, v};", "use explainer_kit::{Facts, Frame2, Kit, Palette, Script, hex, tween, v};"),
    (159, "the_bump", GUIDE_NEW[0], GUIDE_OLD[0]),
    (159, "the_bump", GUIDE_NEW[1], GUIDE_OLD[1]),
    (180, "closing", GET_NEW, GET_OLD),
    (189, "counting", "let at = move |t: f64| C::polar(r, TAU * t);", "let at = |t: f64| C::polar(r, TAU * t);"),
    (213, "two_planes", GET_NEW, GET_OLD),
    (222, "turning", GET_NEW, GET_OLD),
    (231, "gallery", GET_NEW, GET_OLD),
    (240, "loop_sums", GET_NEW, GET_OLD),
]
# The log's 15 errors, as `line: code, file:line` from its own text.
LOGGED = {
    7: ("E0283", "one_derivative.rs:431"),
    38: ("E0283", "one_derivative.rs:249"),
    69: ("E0308", "closing.rs:199"),
    90: ("E0308", "draw.rs:800"),
    101: ("E0308", "draw.rs:633"),
    113: ("E0308", "draw.rs:638"),
    124: ("E0599", "draw.rs:272"),
    141: ("E0599", "draw.rs:283"),
    158: ("E0308", "the_bump.rs:199"),
    179: ("E0515", "closing.rs:411"),
    188: ("E0373", "counting.rs:666"),
    212: ("E0515", "two_planes.rs:652"),
    221: ("E0515", "turning.rs:440"),
    230: ("E0515", "gallery.rs:849"),
    239: ("E0515", "loop_sums.rs:596"),
}
failures = []


def report(ok, text):
    print(("ok   " if ok else "FAIL ") + text)
    if not ok:
        failures.append(text)


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def run(cmd, cwd=None):
    return subprocess.run(cmd, cwd=cwd, capture_output=True, text=True)


def site_failures(texts):
    """The repairs that do not hold in `texts` (file stem to source)."""
    bad = []
    for at, stem, new, old in REPAIRS:
        if new not in texts[stem] or old in texts[stem]:
            bad.append((at, stem, new))
    return bad


def tree(path):
    return {p.relative_to(path).as_posix(): p.read_bytes() for p in sorted(Path(path).rglob("*")) if p.is_file()}


def counts(text):
    return {
        "#[test]": text.count("#[test]"),
        "assert": len(re.findall(r"\b(?:debug_)?assert(?:_eq|_ne)?!", text)),
        "should_panic": text.count("should_panic"),
        "fn": len(re.findall(r"\bfn \w+", text)),
    }


def scans(sources):
    """Where the four repaired forms occur in `sources` (file name to text), anywhere at all."""
    hits = {"collect": [], "temporary": [], "tail": [], "untimed": []}
    for name, text in sources.items():
        if re.search(r"polyline\([^;]*?\.collect\(\)\s*\)", text):
            hits["collect"].append(name)
        if re.search(r"-> &'static str \{\s*crate::facts::facts\(\)", text):
            hits["temporary"].append(name)
        for m in re.finditer(r"^[ \t]*stage\.play\([^\n]*\)\n[ \t]*\}", text, re.M):
            hits["tail"].append((name, text[: m.start()].count("\n") + 1))
        if re.search(r"\.rt\(", text) and "Timed" not in text:
            hits["untimed"].append(name)
    return hits


def main():
    repo = Path(sys.argv[1]).resolve()
    scratch = Path(sys.argv[2]).resolve()
    if scratch.exists() and any(scratch.iterdir()):
        sys.exit("scratch must be empty")
    scratch.mkdir(parents=True, exist_ok=True)

    base = json.loads((HERE / "baseline.json").read_text(encoding="utf8"))
    for rel, want in base["patched_files"].items():
        report(sha(repo / rel) == want, f"baseline unchanged: {rel}")
    for rel, want in base["read_dependencies"].items():
        path = Path(rel).expanduser() if rel.startswith("~") else repo / rel
        report(sha(path) == want, f"dependency unchanged: {rel}")

    # the log really says what this repair was made from
    log = (repo / LOG).read_text(encoding="utf8").splitlines()
    report(log[-1].startswith("error: could not compile") and "15 previous errors" in log[-1], "log ends: 15 previous errors")
    codes = {i + 1: line for i, line in enumerate(log) if re.match(r"error\[E\d+\]", line)}
    report(sorted(codes) == sorted(LOGGED), f"{len(codes)} error headers at the listed log lines")
    for at, (code, where) in LOGGED.items():
        report(code in log[at - 1] and where in log[at], f"log {at}: {code} {where}")
    covered = {r[0] for r in REPAIRS if r[0]}
    report(covered == {at + 1 for at in LOGGED}, "every logged error has a repair row")

    # the patch
    patch = HERE / "compile-repair01.patch"
    made = ""
    for stem in FILES:
        rel = f"{CHAPTERS}/{stem}.rs"
        done = run(["diff", "-u", "--label", f"a/{rel}", "--label", f"b/{rel}", str(repo / rel), str(HERE / rel)])
        report(done.returncode == 1, f"{stem}.rs differs from canonical")
        made += done.stdout
    report(made == patch.read_text(encoding="utf8"), "patch regenerates byte for byte")

    copy = scratch / "applied"
    shutil.copytree(repo / CRATE / "src", copy / CRATE / "src")
    canonical = tree(copy / CRATE / "src")
    done = run(["patch", "-p1", "--no-backup-if-mismatch", "-i", str(patch)], cwd=copy)
    report(done.returncode == 0, "patch -p1 applies")
    applied = tree(copy / CRATE / "src")
    changed = sorted(k for k in canonical if canonical[k] != applied.get(k))
    report(changed == sorted(f"chapters/{s}.rs" for s in FILES) and set(applied) == set(canonical), f"exactly {len(changed)} files change")
    for stem in FILES:
        report(applied[f"chapters/{stem}.rs"] == (HERE / CHAPTERS / f"{stem}.rs").read_bytes(), f"applied {stem}.rs equals the candidate copy")
    done = run(["patch", "-R", "-p1", "--no-backup-if-mismatch", "-i", str(patch)], cwd=copy)
    report(done.returncode == 0 and tree(copy / CRATE / "src") == canonical, "patch -R returns to canonical")
    report(not list((copy / CRATE / "src").rglob("*.orig")) and not list((copy / CRATE / "src").rglob("*.rej")), "no .orig or .rej")

    # the repairs
    texts = {s: (HERE / CHAPTERS / f"{s}.rs").read_text(encoding="utf8") for s in FILES}
    bad = site_failures(texts)
    report(not bad, f"{len(REPAIRS)} repair rows hold" + (f": {bad}" if bad else ""))

    # the old forms are gone from every chapter, not only the logged lines
    every = {p.name: p.read_text(encoding="utf8") for p in (copy / CRATE / "src/chapters").glob("*.rs")}
    registered = set(re.findall(r"^\s*(?:pub )?mod (\w+);", (repo / CRATE / "src/chapters/mod.rs").read_text(encoding="utf8"), re.M))
    before = {n: t for n, t in every.items() if n[:-3] in registered}
    live = dict(before)
    live.update({f"{s}.rs": texts[s] for s in FILES})
    report(len(registered) > 10 and len(live) == len(registered), f"{len(live)} registered chapter modules scanned")
    # control: the same scans must fire on canonical, or finding nothing below means nothing
    old_hits = scans(before)
    report(
        sorted(old_hits["collect"]) == ["one_derivative.rs"]
        and len(old_hits["temporary"]) == 5
        and {h[0] for h in old_hits["tail"]} == {"draw.rs"}
        and len(old_hits["tail"]) == 3
        and old_hits["untimed"] == ["draw.rs"],
        f"control, canonical still has the old forms: {old_hits}",
    )
    new_hits = scans(live)
    report(not any(new_hits.values()), f"none of the old forms in any registered module: {new_hits}")

    # tests and facts kept
    for stem in FILES:
        rel = f"{CHAPTERS}/{stem}.rs"
        before = (repo / rel).read_text(encoding="utf8")
        report(counts(before) == counts(texts[stem]), f"{stem}.rs keeps its counts {counts(texts[stem])}")
        pairs = lambda t: re.findall(r'"(?:[^"\\]|\\.)*"', t)  # noqa: E731
        report(pairs(before) == pairs(texts[stem]), f"{stem}.rs string literals unchanged")
    report(sha(repo / CRATE / "script.tsv") == base["script_tsv"], "script.tsv is the frozen script")

    # rustfmt over the whole module tree
    fmt = scratch / "fmt"
    shutil.copytree(repo / CRATE / "src", fmt / "src")
    for stem in FILES:
        shutil.copy(HERE / CHAPTERS / f"{stem}.rs", fmt / "src/chapters" / f"{stem}.rs")
    done = run(["rustfmt", "--check", "--edition", "2024", "--config-path", "rustfmt.toml", str(fmt / "src/main.rs")], cwd=repo)
    report(done.returncode == 0, "rustfmt --check accepts the whole module tree" + ("" if done.returncode == 0 else "\n" + done.stdout[:800]))

    # each repair undone alone is caught
    caught = 0
    for i, (at, stem, new, old) in enumerate(REPAIRS):
        planted = dict(texts)
        planted[stem] = planted[stem].replace(new, old)
        hits = site_failures(planted)
        if any(h[1] == stem and h[2] == new for h in hits):
            caught += 1
        else:
            report(False, f"plant {i} not caught: {stem} {new[:50]!r}")
    report(caught == len(REPAIRS), f"{caught}/{len(REPAIRS)} repairs undone alone are refused")

    print("\nFAILED: " + str(len(failures)) if failures else "\nall checks passed")
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
