#!/usr/bin/env python3
"""Did the patch change only what it should? Compare the lab copy's lock before and after.

Compares whole parsed package records. Passes only if each of the 19 franken_manim packages appears
once and differs from its old record by the loss of `source` alone, and every other package's record
is identical, its checksum and dependency list included. Cargo qualifies a dependency only when two
packages share a name, and the patch leaves one of each, so nothing else should move. If Cargo does
requalify one, this reports it and root judges it. Run after the first resolve and before any compile.

--subset is for the kit copy, whose workspace holds fewer members than the lock it starts from.
Packages may drop out; none may be added or change in any field.

    python3 -I -B verify_patched_lock.py BEFORE AFTER           exit 0 pass, 1 fail
    python3 -I -B verify_patched_lock.py --subset BEFORE AFTER  exit 0 pass, 1 fail
    python3 -I -B verify_patched_lock.py --self-test LOCK       the checker against planted faults
"""
import sys
import tomllib
from pathlib import Path


def records(text):
    return tomllib.loads(text)["package"]


def key(record):
    return (record["name"], record["version"], record.get("source"))


def faults(before_text, after_text, subset=False):
    before, after = records(before_text), records(after_text)
    pinned = [r for r in before if "franken_manim" in r.get("source", "")]
    names = {r["name"] for r in pinned}
    found = []
    if len(pinned) != 19:
        found.append(f"expected 19 pinned packages before, found {len(pinned)}")
    for old in pinned:
        now = [r for r in after if r["name"] == old["name"]]
        if len(now) != 1:
            found.append(f"{old['name']}: {len(now)} copies after the patch")
            continue
        expected = {field: value for field, value in old.items() if field != "source"}
        if now[0] != expected:
            changed = sorted(f for f in set(expected) | set(now[0]) if expected.get(f) != now[0].get(f))
            found.append(f"{old['name']}: changed beyond losing its source: {changed}")
    rest_before = {key(r): r for r in before if r["name"] not in names}
    rest_after = {key(r): r for r in after if r["name"] not in names}
    if not subset:
        for entry in sorted(set(rest_before) - set(rest_after), key=str):
            found.append(f"lost or changed: {entry}")
    for entry in sorted(set(rest_after) - set(rest_before), key=str):
        found.append(f"added or changed: {entry}")
    for entry in sorted(set(rest_after) & set(rest_before), key=str):
        if rest_after[entry] != rest_before[entry]:
            changed = sorted(
                f
                for f in set(rest_after[entry]) | set(rest_before[entry])
                if rest_after[entry].get(f) != rest_before[entry].get(f)
            )
            found.append(f"{entry[0]} {entry[1]}: same identity, different record: {changed}")
    return found


def blocks(text):
    head, *rest = text.split("[[package]]\n")
    return head, rest


def join(head, parts):
    return head + "".join("[[package]]\n" + p for p in parts)


def patched(text):
    """What the patch should do to a lock: the pinned packages lose their source line."""
    head, parts = blocks(text)
    out = []
    for part in parts:
        lines = part.splitlines(keepends=True)
        out.append("".join(line for line in lines if not (line.startswith("source = ") and "franken_manim" in line)))
    return join(head, out)


def name_of(part):
    return part.split('"')[1]


def self_test(path):
    text = Path(path).read_text()
    good = patched(text)
    assert not faults(text, good), faults(text, good)
    print("pass: the 19 lose their source and nothing else moves")
    head, parts = blocks(good)
    original = {name_of(p): p for p in blocks(text)[1]}
    with_checksum = next(p for p in parts if "\nchecksum = " in p)
    with_dependencies = next(p for p in parts if "\nsource = " in p and "\ndependencies = [\n" in p)
    pinned_with_dependencies = next(p for p in parts if name_of(p) == "fmn-render" and "\ndependencies = [\n" in p)

    def swapped(old, new):
        assert old != new
        return join(head, [new if p is old else p for p in parts])

    planted = {
        "a git copy beside the path copy": join(head, parts + [original["fmn-output"]]),
        "one package left on git": join(head, [original["fmn-text"] if name_of(p) == "fmn-text" else p for p in parts]),
        "one package dropped": join(head, [p for p in parts if name_of(p) != "fmn-hash"]),
        "a pinned version moved": join(head, [p.replace('version = "0.5.0"', 'version = "0.5.1"') if name_of(p) == "fmn-core" else p for p in parts]),
        "an unrelated package added": join(head, parts + ['name = "planted"\nversion = "1.0.0"\nsource = "registry+https://github.com/rust-lang/crates.io-index"\n\n']),
        "an unrelated package's version changed": join(head, [parts[0].replace('version = "', 'version = "9', 1)] + parts[1:]),
        "an unrelated package's checksum changed": swapped(with_checksum, with_checksum.replace('checksum = "', 'checksum = "0', 1)),
        "an unrelated package gained a dependency": swapped(with_dependencies, with_dependencies.replace("dependencies = [\n", 'dependencies = [\n "planted",\n', 1)),
        "an unrelated package lost a dependency": swapped(with_dependencies, with_dependencies[: with_dependencies.index("dependencies = [\n") + len("dependencies = [\n")] + with_dependencies[with_dependencies.index("\n", with_dependencies.index("dependencies = [\n") + len("dependencies = [\n")) + 1 :]),
        "a pinned package's dependencies changed": swapped(pinned_with_dependencies, pinned_with_dependencies.replace("dependencies = [\n", 'dependencies = [\n "planted",\n', 1)),
    }
    for label, fault in planted.items():
        result = faults(text, fault)
        assert result, label
        print(f"caught: {label}: {result[0]}")
    unrelated = next(p for p in parts if "\nsource = " in p and name_of(p) != "fmn")
    shrunk = join(head, [p for p in parts if p is not unrelated])
    assert faults(text, shrunk), "without --subset a package may not drop out"
    assert not faults(text, shrunk, subset=True), faults(text, shrunk, subset=True)
    print("--subset pass: an unrelated package drops out")
    for label, fault in planted.items():
        result = faults(text, fault, subset=True)
        assert result, label
        print(f"--subset caught: {label}: {result[0]}")


if sys.argv[1] == "--self-test":
    self_test(sys.argv[2])
    sys.exit(0)
subset = sys.argv[1] == "--subset"
paths = sys.argv[2:] if subset else sys.argv[1:]
before_text, after_text = Path(paths[0]).read_text(), Path(paths[1]).read_text()
found = faults(before_text, after_text, subset)
for fault in found:
    print("FAULT", fault)
if found:
    sys.exit(1)
extra = tomllib.loads(after_text).get("patch", {}).get("unused", [])
kept = "nothing else added or changed in any field" if subset else "every other record is identical"
print(f"PASS: the 19 packages are path copies, {kept}; patch.unused entries: {len(extra)}")
