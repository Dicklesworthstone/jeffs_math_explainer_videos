"""Text checks on D against ABC and AB: round trip, strings, waits, phrases, formatting. No Cargo, no compile.

python3 -I -B source_checks.py REPO_ROOT ABC_RS AB_RS D_RS PATCH WORK_DIR   (prints only; WORK_DIR must not exist)

Everything is written under WORK_DIR, so nothing under REPO_ROOT changes. rustfmt runs on scratch copies of the crate's
src, because rustfmt on main.rs follows every module, and a parse error anywhere fails the run. Controls: a misindented
line must draw a diff in arithmetic.rs, a stray brace must draw a parse error, and a phrase that is not in its sentence
must be reported.
"""
import hashlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

CRATE = "explainer_corpus/abstract_algebra_expanded"
RS = f"{CRATE}/src/arithmetic.rs"
ABC_SHA = "d3385e17fc8fe9ed727b94c34dbe4994d0e48fdd5b8834e3eadb228e13ed0205"
SENTENCE_END = re.compile(r"(?<=[.?!]) ")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def literals(text):
    body = text[: text.index("#[cfg(test)]\nmod tests")]
    return set(re.findall(r'"((?:[^"\\]|\\.)*)"', body))


def round_trip(abc, d, patch, work):
    top = work / "round"
    (top / Path(RS).parent).mkdir(parents=True)
    shutil.copyfile(abc, top / RS)
    done = subprocess.run(["git", "apply", "--check", str(patch)], cwd=top, capture_output=True, text=True)
    print(f"git apply --check on a scratch copy of ABC: exit {done.returncode} {done.stderr.strip()[:200]}")
    if done.returncode:
        raise SystemExit("patch does not apply")
    done = subprocess.run(["patch", "-s", "-p1", "-i", str(patch)], cwd=top, capture_output=True, text=True)
    print(f"patch -p1 forward: exit {done.returncode}; bytes equal candidate-ABCD: {(top / RS).read_bytes() == Path(d).read_bytes()}")
    done = subprocess.run(["patch", "-s", "-R", "-p1", "-i", str(patch)], cwd=top, capture_output=True, text=True)
    print(f"patch -R -p1: exit {done.returncode}; bytes equal ABC: {sha(top / RS) == ABC_SHA}")
    if sha(top / RS) != ABC_SHA:
        raise SystemExit("reverse did not restore ABC")


def wait_order(text, label):
    lines = text.splitlines()
    for cue, count in (("13_3", 7), ("13_7", 5), ("13_8", 5)):
        waits = []
        for line in lines:
            m = re.search(rf'until_sentence\(stage, "{cue}", (\d+)\)', line) or re.search(
                rf'until_phrase\(stage, &kit, "{cue}", (\d+),', line
            )
            if m:
                waits.append(int(m.group(1)))
        ok = waits == sorted(waits) and all(w < count for w in waits)
        print(f"  {label} {cue}: waits at sentences {waits}; in order and in range: {ok}")
        if not ok:
            raise SystemExit("bad wait")


def sentence_mirror(text):
    """The split axioms.rs::sentence_ranges makes: after . ? or ! followed by a space (or the end of the line)."""
    return [s.strip() for s in SENTENCE_END.split(text) if s.strip()]


def phrases(source, script, label, tamper=False):
    consts = dict(re.findall(r'const (\w+_PHRASE): &str = "([^"]+)";', source))
    laws = re.search(r"const LAW_PHRASES: \[&str; \d+\] = \[(.*?)\];", source, re.S)
    for i, phrase in enumerate(re.findall(r'"([^"]+)"', laws.group(1))):
        consts[f"LAW_PHRASES[{i}]"] = phrase
    block = re.search(r"const PHRASES: \[\(&str, usize, &str\); (\d+)\] = \[(.*?)\];", source, re.S)
    rows = re.findall(r'\("([^"]+)", (\d+), (\w+(?:\[\d+\])?)\)', block.group(2))
    declared = int(block.group(1))
    lines = {ln.split("\t")[0]: ln.split("\t")[-1] for ln in script.splitlines() if "\t" in ln}
    missing = []
    for cue, j, name in rows:
        phrase = consts[name] if not (tamper and name == "UNDO_PHRASE") else "an undo moves"
        sentences = sentence_mirror(lines[cue])
        if int(j) >= len(sentences) or phrase not in sentences[int(j)]:
            missing.append((cue, j, phrase))
    print(f"  {label}: declared {declared}, rows {len(rows)}, phrases not in their sentence: {missing if missing else 'none'}")
    return declared, rows, missing


def fmt(label, top, config, work):
    done = subprocess.run(
        ["rustfmt", "--check", "--edition", "2024", "--config-path", str(config), str(top / "src/main.rs")],
        capture_output=True, text=True,
    )
    text = (done.stdout + done.stderr).replace(str(top) + "/", "").replace(str(work) + "/", "")
    diffs = [ln for ln in text.splitlines() if ln.startswith("Diff in")]
    errors = [ln for ln in text.splitlines() if ln.startswith("error")]
    print(f"  {label}: rustfmt exit {done.returncode}; format diffs {len(diffs)}; parse errors {len(errors)}")
    for ln in diffs[:4] + errors[:2]:
        print("     ", ln[:120])
    return diffs, errors


def formatting(repo, abc, d, work):
    config = repo / "rustfmt.toml"
    print("rustfmt", subprocess.run(["rustfmt", "--version"], capture_output=True, text=True).stdout.strip())
    print("config", config.name, "=", " ".join(config.read_text(encoding="utf-8").split()))
    trees = {}
    for name, source in (("abc", abc), ("d", d), ("misindent", d), ("brace", d)):
        top = work / name
        shutil.copytree(repo / CRATE / "src", top / "src")
        shutil.copyfile(source, top / "src/arithmetic.rs")
        trees[name] = top
    # The film's other sources are canonical here. rustfmt reads them for the module walk and is not asked to write.
    path = trees["misindent"] / "src/arithmetic.rs"
    comment = "        // The sum stays readable even if this take's sentence window is shorter than measured.\n"
    needle = comment + "        stage.wait(0.35)?;\n"
    text = path.read_text(encoding="utf-8")
    if text.count(needle) != 1:
        raise SystemExit("control could not find its line")
    path.write_text(text.replace(needle, comment + "   stage.wait(0.35)?;\n"), encoding="utf-8")
    path = trees["brace"] / "src/arithmetic.rs"
    path.write_text(path.read_text(encoding="utf-8").replace(needle, needle + "        }}\n"), encoding="utf-8")
    d_abc, e_abc = fmt("ABC", trees["abc"], config, work)
    d_d, e_d = fmt("ABCD", trees["d"], config, work)
    d_m, _ = fmt("control, misindented line", trees["misindent"], config, work)
    _, e_b = fmt("control, stray brace", trees["brace"], config, work)
    chapter = [ln for ln in d_d if "arithmetic.rs" in ln]
    print(f"  ABCD: diffs in arithmetic.rs {len(chapter)}; parse errors {len(e_d)}; diff list equals ABC's: {d_d == d_abc and e_d == e_abc}")
    if chapter or e_d or d_d != d_abc:
        raise SystemExit("format check failed")
    if not [ln for ln in d_m if "arithmetic.rs" in ln] or not e_b:
        raise SystemExit("a format control did not fire")
    print("  both controls fired")


def main():
    repo, abc, ab, d, patch, work = (Path(p).resolve() for p in sys.argv[1:7])
    if work.exists():
        raise SystemExit("refusing: WORK_DIR exists")
    work.mkdir(parents=True)
    print("ABC", sha(abc))
    print("ABCD", sha(d))
    if sha(abc) != ABC_SHA:
        raise SystemExit("ABC is not d3385e17")
    round_trip(abc, d, patch, work)

    abc_text, ab_text, d_text = (Path(p).read_text(encoding="utf-8") for p in (abc, ab, d))
    print("\nString literals outside the tests")
    for label, a, b in (("AB to ABC", ab_text, abc_text), ("ABC to ABCD", abc_text, d_text)):
        la, lb = literals(a), literals(b)
        print(f"  {label}: added {sorted(lb - la)}; removed {sorted(la - lb)}")
    if literals(abc_text) != literals(d_text):
        raise SystemExit("D changed a string")

    print("\nSentence and phrase waits")
    wait_order(abc_text, "ABC")
    wait_order(d_text, "ABCD")
    script = (repo / CRATE / "script.tsv").read_text(encoding="utf-8")
    declared, rows, missing = phrases(d_text, script, "ABCD phrases")
    if declared != len(rows) or missing:
        raise SystemExit("phrase table is wrong")
    _, _, caught = phrases(d_text, script, "control, altered UNDO phrase", tamper=True)
    if not caught:
        raise SystemExit("phrase control did not fire")

    print("\nThe statements D uses")
    for name, pattern in (
        ("step keeps other callers", r"= step\(stage, &kit,"),
        ("LONE_RT constant", r"const LONE_RT: f64"),
        ("perform helper", r"fn perform\(stage: &mut Stage<'_>, tracker: Mob, to: f64, run_time: f64, marks: &\[Mob\]\)"),
    ):
        print(f"  {name}: {len(re.findall(pattern, d_text))}")
    print(f"  existing `stage.wait(..)?;` statements (the same discarded result as D2): {len(re.findall(r'stage[.]wait[(][0-9.]+[)][?];', abc_text))}")
    print(f"  arithmetic.rs named in include_str! under src: {[p.name for p in (repo / CRATE / 'src').rglob('*.rs') if 'arithmetic.rs' in ''.join(re.findall(r'include_str!.*', p.read_text(encoding='utf-8')))]}")

    print("\nFormatting")
    formatting(repo, abc, d, work)


if __name__ == "__main__":
    main()
