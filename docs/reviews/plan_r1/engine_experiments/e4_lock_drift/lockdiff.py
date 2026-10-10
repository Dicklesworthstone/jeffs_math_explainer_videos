"""E4 of the engine-feasibility review: crates.io version drift between two Cargo.lock files.

Compares our workspace's Cargo.lock with franken_manim's Cargo.lock at the pinned
commit and lists every package whose resolved versions differ. A downstream workspace
resolves crates.io dependencies on its own, so it can drift from the versions
franken_manim tested; `./x doctor` can run this check.

Usage:
    git -C <franken_manim> show <pin>:Cargo.lock > <tmp>/franken_manim.lock
    python3 -I lockdiff.py <our Cargo.lock> <tmp>/franken_manim.lock

Needs Python 3.11+ (tomllib).
"""
import sys
import tomllib


def load(path):
    with open(path, "rb") as handle:
        data = tomllib.load(handle)
    out = {}
    for package in data.get("package", []):
        out.setdefault(package["name"], set()).add((package["version"], package.get("source", "path")))
    return out


def main(ours_path, theirs_path):
    ours, theirs = load(ours_path), load(theirs_path)
    common = sorted(set(ours) & set(theirs))
    drift = [
        (name, sorted(ours[name]), sorted(theirs[name]))
        for name in common
        if {v for v, _ in ours[name]} != {v for v, _ in theirs[name]}
    ]
    print(f"packages: ours={len(ours)} franken_manim={len(theirs)} common={len(common)} version-mismatch={len(drift)}")
    for name, a, b in drift:
        print(f"  {name}: ours {[v for v, _ in a]} vs franken_manim {[v for v, _ in b]}")
    print("only ours:", sorted(set(ours) - set(theirs)))
    return 1 if drift else 0


if __name__ == "__main__":
    if len(sys.argv) != 3:
        sys.exit("usage: python3 -I lockdiff.py OUR_CARGO_LOCK FRANKEN_MANIM_CARGO_LOCK")
    sys.exit(main(sys.argv[1], sys.argv[2]))
