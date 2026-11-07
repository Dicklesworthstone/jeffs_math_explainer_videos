"""The checks build01.sh runs after cargo, frozen from root's algebra-corner-contrast-build01.

    expect_build01.py tests EXPECTED TEST_LOG
        Every expected test passed, no test failed or was ignored, no other test ran, and the one
        summary line counts exactly the expected tests.
    expect_build01.py facts NATIVE ORACLE OLD_NATIVE OLD_ORACLE
        Root's check (two columns, unique keys, native equals oracle), and both files byte-identical
        to root's build, since neither repair touches a fact. Not independent review.

Exit 0 pass, 1 fail.
"""
import re
import sys
from pathlib import Path


def fail(message):
    print(f"expect_build01 FAIL: {message}")
    return 1


def tests(expected_path, log_path):
    expected = Path(expected_path).read_text().splitlines()
    log = Path(log_path).read_text()
    results = re.findall(r"^test (\S+) \.\.\. (\S+)$", log, re.M)
    passed = [name for name, status in results if status == "ok"]
    others = [(name, status) for name, status in results if status != "ok"]
    summaries = re.findall(r"^test result: (\S+)\. (\d+) passed; (\d+) failed; (\d+) ignored; (\d+) measured; (\d+) filtered out",
                           log, re.M)
    if len(expected) != len(set(expected)) or not expected:
        return fail("the expected list is empty or repeats a name")
    if others:
        return fail(f"tests not ok: {others[:5]}")
    if sorted(passed) != sorted(expected):
        missing = sorted(set(expected) - set(passed))
        extra = sorted(set(passed) - set(expected))
        return fail(f"missing {missing[:5]}, unexpected {extra[:5]}, repeated {len(passed) - len(set(passed))}")
    if summaries != [("ok", str(len(expected)), "0", "0", "0", "0")]:
        return fail(f"summary lines {summaries}")
    print(f"All {len(expected)} expected tests passed: root's build01 set plus the new coset test.")
    return 0


def facts(path):
    rows = [line.split("\t", 1) for line in Path(path).read_text().splitlines() if line.strip()]
    if not rows or any(len(row) != 2 for row in rows):
        raise ValueError(f"{path}: not two tab-separated columns")
    result = dict(rows)
    if len(result) != len(rows):
        raise ValueError(f"{path}: repeated key")
    return result


def facts_check(native, oracle, old_native, old_oracle):
    try:
        mine, theirs = facts(native), facts(oracle)
    except ValueError as error:
        return fail(str(error))
    if mine != theirs:
        return fail(f"native and oracle facts differ on {sorted(k for k in set(mine) | set(theirs) if mine.get(k) != theirs.get(k))[:5]}")
    for new, old in ((native, old_native), (oracle, old_oracle)):
        if Path(new).read_bytes() != Path(old).read_bytes():
            return fail(f"{Path(new).name} differs from root's build01")
    print(f"Native and Python facts agree: {len(mine)} unique keys, byte-identical to root's build01; not independent review")
    return 0


def main(argv):
    if len(argv) == 3 and argv[0] == "tests":
        return tests(*argv[1:])
    if len(argv) == 5 and argv[0] == "facts":
        return facts_check(*argv[1:])
    print(__doc__)
    return 2


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
