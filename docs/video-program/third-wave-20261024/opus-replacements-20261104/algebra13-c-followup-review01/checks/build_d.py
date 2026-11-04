"""Build correction D from the exact ABC bytes: two exact replacements, a repo-relative patch and the hash files.

python3 -I -B build_d.py ABC_ARITHMETIC_RS OUT_DIR   (from the repo root)

It writes only under OUT_DIR, which must not exist yet. Each replacement must match exactly once, and the input must
be ABC d3385e17, or nothing is written.
"""
import difflib
import hashlib
import sys
from pathlib import Path

RS = "explainer_corpus/abstract_algebra_expanded/src/arithmetic.rs"
ABC_SHA = "d3385e17fc8fe9ed727b94c34dbe4994d0e48fdd5b8834e3eadb228e13ed0205"
ORACLE = "explainer_corpus/abstract_algebra_expanded/oracle.py"
ORACLE_SHA = "899d6f15d33470ef99dd353389ca0971ec6574667df288366a73af088eb0457d"
D1_OLD = """        kit.until_sentence(stage, "13_3", 4)?;
        left_label = step(stage, &kit, left, 3., left_label, "r³", LEFT_LABEL)?;
        stage.wait(0.6)?;
"""
D1_NEW = """        kit.until_sentence(stage, "13_3", 4)?;
        fade(stage, &[left_label], false)?;
        perform(stage, left, 3., LONE_RT, &[])?;
        left_label = caption(stage, &kit, "r³", LEFT_LABEL.0, LEFT_LABEL.1)?;
        kit.until_sentence(stage, "13_3", 5)?;
"""
D2_OLD = """        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        kit.until_sentence(stage, "13_8", 3)?;
"""
D2_NEW = """        let mut undo_label = caption(stage, &kit, "r³ r = e", 0., -2.)?;
        // The sum stays readable even if this take's sentence window is shorter than measured.
        stage.wait(0.35)?;
        kit.until_sentence(stage, "13_8", 3)?;
"""


def sha(data):
    return hashlib.sha256(data).hexdigest()


def main():
    abc_path, out = Path(sys.argv[1]), Path(sys.argv[2])
    if out.exists():
        raise SystemExit("refusing: OUT_DIR exists")
    raw = abc_path.read_bytes()
    if sha(raw) != ABC_SHA:
        raise SystemExit(f"input is not ABC d3385e17: {sha(raw)}")
    old = raw.decode("utf-8")
    new = old
    for label, before, after in (("D1", D1_OLD, D1_NEW), ("D2", D2_OLD, D2_NEW)):
        if new.count(before) != 1:
            raise SystemExit(f"{label}: expected one match, found {new.count(before)}")
        new = new.replace(before, after)
    target = out / "candidate-ABCD" / RS
    target.parent.mkdir(parents=True)
    target.write_text(new, encoding="utf-8")
    patch = "".join(
        difflib.unified_diff(
            old.splitlines(keepends=True), new.splitlines(keepends=True), f"a/{RS}", f"b/{RS}", n=3
        )
    )
    (out / "patches").mkdir()
    (out / "patches/D-followup.patch").write_text(patch, encoding="utf-8")
    oracle = abc_path.parent.parent / "oracle.py"
    data = oracle.read_bytes()
    if sha(data) != ORACLE_SHA:
        raise SystemExit(f"oracle is not 899d6f15: {sha(data)}")
    (out / "candidate-ABCD" / ORACLE).write_bytes(data)
    (out / "base.sha256").write_text(f"{ABC_SHA}  {RS}\n{ORACLE_SHA}  {ORACLE}\n", encoding="utf-8")
    (out / "result.sha256").write_text(f"{sha(new.encode('utf-8'))}  {RS}\n{ORACLE_SHA}  {ORACLE}\n", encoding="utf-8")
    added = sum(1 for ln in patch.splitlines() if ln.startswith("+") and not ln.startswith("+++"))
    removed = sum(1 for ln in patch.splitlines() if ln.startswith("-") and not ln.startswith("---"))
    print(f"base {ABC_SHA}")
    print(f"result {sha(new.encode('utf-8'))}")
    print(f"patch +{added} -{removed}; lines {len(old.splitlines())} to {len(new.splitlines())}")


if __name__ == "__main__":
    main()
