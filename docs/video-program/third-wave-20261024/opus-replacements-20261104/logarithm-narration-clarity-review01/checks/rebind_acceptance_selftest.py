"""Exercise rebind_acceptance.py on scratch copies. Writes only under FRESH_SCRATCH; reads the repo. Repo root:

    python3 -I -B .../logarithm-narration-clarity-review01/checks/rebind_acceptance_selftest.py FRESH_SCRATCH

Each case copies root's stage08 crate, applies the clarity patch (2463334a) with git apply, and rebinds by the plain field
substitutions the review lists (a positive control, not a proposed integration). Then it plants one defect and runs the check.
"""
import hashlib
import json
import shutil
import subprocess
import sys
from pathlib import Path

REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
P = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
PATCH = P / "logarithm-narration-clarity-repair01/logarithm-narration-clarity.patch"
EL = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
REL = "explainer_corpus/logarithm_mysteries"
STAGE08 = EL / "combined-repair-stage08/candidate" / REL
PLAN = EL / "combined-repair-stage07/bound-plan03"
OLD_PRODUCT = "f2c8684acd8319b5f518c59a6ae68da5488377ad9758f9c539f57867312210db"
sys.path.insert(0, str(HERE))
import rebind_acceptance as ra  # noqa: E402  (the check under test; its helpers build the control)


def dump(path, data):
    path.write_text(json.dumps(data, indent=1, ensure_ascii=False) + "\n", encoding="utf-8")


def build(root, patch=True, rebind=True):
    crate = root / REL
    shutil.copytree(STAGE08, crate)
    if not patch:
        return crate
    r = subprocess.run(["git", "apply", str(PATCH)], cwd=root, capture_output=True, text=True)
    assert r.returncode == 0, r.stderr
    if not rebind:
        return crate
    new = ra.rows(crate / "script.tsv")
    script_sha = ra.sha_b((crate / "script.tsv").read_bytes())
    product_sha = ra.sha_b((crate / "src/chapters/product_rule.rs").read_bytes())
    m = json.loads((crate / ra.MAP15).read_text(encoding="utf-8"))
    m["scripts"]["logarithm_mysteries"] = script_sha
    for r in m["visual_holds"]:
        if r["path"].endswith("/product_rule.rs"):
            r["sha256"] = r["evidence"]["sha256"] = product_sha
    map_rel = "docs/narration-direction-map-oliver16.json"
    dump(crate / map_rel, m)
    u = json.loads((crate / "narration_units.json").read_text(encoding="utf-8"))
    for unit in u["units"]:
        unit["text"] = " ".join(new[i] for i in unit["line_ids"])
    u["script_sha256"] = u["direction_contract"]["script_sha256"] = script_sha
    u["direction_contract"]["map"] = {"path": f"{REL}/{map_rel}", "sha256": ra.sha_b((crate / map_rel).read_bytes())}
    dump(crate / "narration_units.json", u)
    a = json.loads((PLAN / "phrase-anchors.json").read_text(encoding="utf-8"))
    a["script_sha256"] = script_sha
    for x in a["anchors"]:
        if x["cue"] == "08_5":
            x.update(ra.anchor(new["08_5"], x["phrase"]))
            x["cue_text_sha256"] = ra.sha_t(new["08_5"])
            x["source_sha256"] = product_sha
    dump(root / "anchors.json", a)
    t = json.loads((PLAN / "phrase-timings-unmeasured-template.json").read_text(encoding="utf-8"))
    t["script_sha256"] = script_sha
    for cue in ("04_4", "08_5"):
        t["cues"][cue]["text_sha256"] = ra.sha_t(new[cue])
    for mark in t["cues"]["08_5"]["marks"]:
        mark["byte"] = ra.anchor(new["08_5"], mark["phrase"])["byte"]
    dump(root / "template.json", t)
    h = json.loads((PLAN / "hold-audit.json").read_text(encoding="utf-8"))
    h["script_sha256"] = script_sha
    h["counts"]["words_total"] = sum(len(x.split()) for x in new.values())
    for r in h["cues"]:
        r["words"] = len(new[r["cue"]].split())
    for r in h["prediction_holds"] + h["reading_holds"]:
        if r["file"] == "product_rule.rs":
            r["source_sha256"] = product_sha
    dump(root / "hold-audit.json", h)
    for cue in ("04_4", "08_5"):
        rec = json.loads((ra.TAKES / f"{cue}-request01.json").read_text(encoding="utf-8"))
        rec["text"], rec["text_sha256"] = new[cue], ra.sha_t(new[cue])
        dump(root / f"receipt-{cue}.json", rec)
    return crate


def reseal_map(crate):
    """After a planted map edit, point the units at the edited map so only the planted defect remains."""
    u = json.loads((crate / "narration_units.json").read_text(encoding="utf-8"))
    path = u["direction_contract"]["map"]["path"]
    u["direction_contract"]["map"]["sha256"] = ra.sha_b((crate / path[len(REL) + 1:]).read_bytes())
    dump(crate / "narration_units.json", u)


def edit(path, change):
    data = json.loads(path.read_text(encoding="utf-8"))
    change(data)
    dump(path, data)


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    assert "narration_lab" not in scratch.resolve().parts
    results = []

    def case(name, want, plant=None, patch=True, rebind=True):
        root = scratch / name
        crate = build(root, patch, rebind)
        if plant:
            plant(root, crate)
        args = [str(crate)]
        if rebind and patch:
            args += ["--anchors", str(root / "anchors.json"), "--template", str(root / "template.json"),
                     "--hold-audit", str(root / "hold-audit.json"),
                     "--receipt", str(root / "receipt-04_4.json"), "--receipt", str(root / "receipt-08_5.json")]
        r = subprocess.run([sys.executable, "-I", "-B", str(HERE / "rebind_acceptance.py"), *args], cwd=REPO,
                           capture_output=True, text=True)
        ok = r.returncode == want and not r.stderr
        first = next((line for line in r.stdout.splitlines() if line.startswith("FAIL")), None)
        results.append(ok)
        print(f"{'PASS' if ok else 'FAIL'} {name}: exit {r.returncode}, expected {want}" + (f"; first failure: {first}" if first else ""))
        if r.stderr:
            print("  stderr: " + r.stderr.strip().splitlines()[-1])

    def neighbour(root, crate):
        text = (crate / "script.tsv").read_text(encoding="utf-8")
        (crate / "script.tsv").write_text(text.replace("And this is the ruler again.", "And this is our ruler again.", 1),
                                          encoding="utf-8")
        new = ra.rows(crate / "script.tsv")
        edit(crate / "narration_units.json", lambda u: [x.update(text=" ".join(new[i] for i in x["line_ids"])) for x in u["units"]])

    def stale_map(root, crate):
        edit(crate / "docs/narration-direction-map-oliver16.json",
             lambda m: [r.update(sha256=OLD_PRODUCT) for r in m["visual_holds"] if r["path"].endswith("/product_rule.rs")])
        reseal_map(crate)

    case("t0-good-rebind", 0)
    case("t1-template-keeps-08_5-marks-at-260-299", 1,
         lambda root, crate: edit(root / "template.json",
                                  lambda t: [m.update(byte=b) for m, b in zip(t["cues"]["08_5"]["marks"], (260, 299))]))
    case("t2-map-keeps-old-product_rule-hash", 1, stale_map)
    case("t3-neighbour-04_5-reworded", 1, neighbour)
    case("t4-units-keep-old-08_5-text", 1,
         lambda root, crate: edit(crate / "narration_units.json", lambda u: [
             x.update(text=ra.rows(STAGE08 / "script.tsv")["08_5"]) for x in u["units"] if x["id"] == "oliver_08_5"]))
    case("t5-anchors-keep-08_5-byte-260", 1,
         lambda root, crate: edit(root / "anchors.json", lambda a: [
             x.update(byte=260) for x in a["anchors"] if x["cue"] == "08_5" and x["const"] == "HALF_TURN_PHRASE"]))
    case("t11-anchors-11_20-whitespace-word-index", 1,
         lambda root, crate: edit(root / "anchors.json", lambda a: [
             x.update(word_index=len(ra.rows(crate / "script.tsv")["11_20"].encode()[:x["byte"]].decode().split()))
             for x in a["anchors"] if x["cue"] == "11_20"]))
    case("t6-stage08-unpatched", 1, patch=False, rebind=False)
    case("t7-patched-not-rebound", 1, rebind=False)
    case("t8-extra-orig-file", 1,
         lambda root, crate: shutil.copyfile(crate / "src/chapters/product_rule.rs", crate / "src/chapters/product_rule.rs.orig"))
    case("t9-receipt-wrong-voice", 1,
         lambda root, crate: edit(root / "receipt-08_5.json", lambda r: r.update(voice="Frederick Surrey")))
    case("t10-hold-audit-keeps-old-source-hash", 1,
         lambda root, crate: edit(root / "hold-audit.json", lambda h: [
             r.update(source_sha256=OLD_PRODUCT) for r in h["prediction_holds"] if r["file"] == "product_rule.rs"]))
    print("SELFTEST:", "PASS" if all(results) else "FAIL", f"({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
