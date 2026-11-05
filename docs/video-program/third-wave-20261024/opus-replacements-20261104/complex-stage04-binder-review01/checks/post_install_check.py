"""After root installs the bound oliver12 plan: is the installed stage exactly stage04 plus that plan? Read-only.

    python3 -I -B .../complex-stage04-binder-review01/checks/post_install_check.py STAGE_ROOT [RECEIPT]

STAGE_ROOT is the stage folder holding explainer_corpus/ (a new stage, or stage04 itself after an in-place install).
RECEIPT, if given, is that stage's new receipt (JSON with a `files` list of {path, sha256} and a `changed` list).

Why this exists: BronzeJaguar's sealed binder pins stage04's receipt (295c27e3...) and stage04's path, so after a real
installation it cannot re-check: rewriting the receipt is a pin change (exit 3), and keeping it fails its receipt
check (exit 1). Its d21 passes only because the test rewrites the pin in a mirror. This check needs no pin change.

Pass means, against stage04's receipt (pinned):
  - every file stage04 lists is present with stage04's hash, except
      narration_units.json, which must be the bound units e776c860..., and
      the four F2 strays (three .orig files and .ee/hook-state/...), which may be absent;
  - the only other file on disk is docs/narration-direction-map-oliver12.json, the bound map 70888624...;
  - script.tsv is still 12442480... and the installed units point at that map by path and hash;
  - with RECEIPT: it lists exactly the files on disk with their hashes, and `changed` names the units and the map.
Exit 0 pass, 1 fail, 2 usage.
"""
import hashlib
import json
import pathlib
import sys

REPO = pathlib.Path.cwd()
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
RECEIPT04 = (f"{EC}/combined-repair-stage04-integration01.json", "295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b")
CR = "explainer_corpus/complex_analysis_visual"
UNITS = (f"{CR}/narration_units.json", "e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6")
MAP = (f"{CR}/docs/narration-direction-map-oliver12.json", "70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c")
SCRIPT = (f"{CR}/script.tsv", "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3")
STRAYS = {f"{CR}/src/chapters/counting.rs.orig", f"{CR}/src/chapters/mod.rs.orig", f"{CR}/src/chapters/one_derivative.rs.orig",
          f"{CR}/.ee/hook-state/session_start_orient.last.json"}


def sha(p):
    return hashlib.sha256(p.read_bytes()).hexdigest()


def main(argv):
    if len(argv) not in (1, 2):
        print(__doc__.split("\n\n")[1], file=sys.stderr)
        return 2
    root = pathlib.Path(argv[0])
    if not (root / CR).is_dir():
        print(f"usage: {argv[0]} has no {CR}", file=sys.stderr)
        return 2
    fails = []

    def check(ok, label):
        print("PASS" if ok else "FAIL", label)
        if not ok:
            fails.append(label)

    r04 = REPO / RECEIPT04[0]
    check(sha(r04) == RECEIPT04[1], f"stage04 receipt is the pinned {RECEIPT04[1][:8]}")
    base = {f["path"]: f["sha256"] for f in json.loads(r04.read_text(encoding="utf-8"))["files"]}
    disk = {str(p.relative_to(root)).replace("\\", "/"): sha(p) for p in sorted(root.rglob("*")) if p.is_file()}

    wrong = sorted(p for p, h in base.items() if p not in STRAYS | {UNITS[0]} and disk.get(p) != h)
    check(not wrong, f"every other stage04 file is present with stage04's hash ({len(base) - len(STRAYS) - 1} files): {wrong or 'all'}")
    check(disk.get(UNITS[0]) == UNITS[1], f"narration_units.json is the bound units {UNITS[1][:8]} (now {str(disk.get(UNITS[0]))[:8]})")
    check(disk.get(MAP[0]) == MAP[1], f"docs/narration-direction-map-oliver12.json is the bound map {MAP[1][:8]} (now {str(disk.get(MAP[0]))[:8]})")
    extra = sorted(set(disk) - set(base) - {MAP[0]})
    check(not extra, f"no other new file: {extra or 'none'}")
    gone = sorted(STRAYS - set(disk))
    kept = sorted(p for p in STRAYS & set(disk) if disk[p] != base[p])
    check(not kept, f"F2 strays absent or unchanged (absent: {len(gone)} of 4)")
    check(disk.get(SCRIPT[0]) == SCRIPT[1], f"script.tsv is still {SCRIPT[1][:8]}")
    if disk.get(UNITS[0]) == UNITS[1]:
        u = json.loads((root / UNITS[0]).read_text(encoding="utf-8"))
        dc = u["direction_contract"]
        check(dc["map"]["path"] == MAP[0] and dc["map"]["sha256"] == MAP[1] == disk.get(MAP[0]) and u["script_sha256"] == SCRIPT[1],
              "the installed units name the installed map by path and hash, and the script")
    if len(argv) == 2:
        rec = json.loads(pathlib.Path(argv[1]).read_text(encoding="utf-8"))
        listed = {f["path"]: f["sha256"] for f in rec.get("files", [])}
        check(listed == disk, f"the new receipt lists exactly the {len(disk)} files on disk with their hashes")
        ch = set(rec.get("changed", []))
        check({UNITS[0], MAP[0]} <= ch, "the new receipt's `changed` names the units and the map")
    print("POST-INSTALL:", "PASS" if not fails else f"FAIL ({len(fails)})")
    return 1 if fails else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
