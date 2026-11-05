"""Join root's three 720p plate_lab folders into one read-only view for measure_lab.py, and record where every scene came from.

    python3 -B .../checks/consolidate.py NEW_VIEW_DIR     (from the repo root; the view belongs in scratch)

Writes nothing in the lab folders. The view is one directory of symlinks to the original scene folders, plus a plates.tsv
(checked byte-equal across all three labs) and a receipt.tsv joined from their receipts. Two labs supplying one scene is a
failure, as is a scene with no receipt line or a receipt line with no scene.

Saves provenance/originals.tsv (every original file: lab, path, bytes, sha256) and provenance/scene-sources.tsv.
"""
import hashlib
import json
import os
import pathlib
import sys

E = pathlib.Path("narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104/color-native-stage01")
JOBS = pathlib.Path("narration_lab/third-wave-20261024/root/five-film-release-20261027")
BUNDLE = pathlib.Path(__file__).resolve().parent.parent
LABS = ("lab720-01", "lab720-fades02", "lab720-controls02")
KINDS = ("stills", "fades", "repaints")
REPAIR = pathlib.Path("docs/video-program/third-wave-20261024/opus-replacements-20261104/complex-color-repair03")


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def main(view):
    view = pathlib.Path(view)
    view.mkdir(parents=True, exist_ok=False)
    problems = []

    # one geometry across the three
    plates = {lab: (E / lab / "plates.tsv").read_bytes() for lab in LABS}
    if len(set(plates.values())) != 1:
        problems.append("plates.tsv differs between labs")
    (view / "plates.tsv").write_bytes(plates[LABS[0]])
    print(f"plates.tsv byte-equal across {len(LABS)} labs: {len(set(plates.values())) == 1}, sha256 {hashlib.sha256(plates[LABS[0]]).hexdigest()}")

    # every original file
    originals = ["lab\tpath\tbytes\tsha256"]
    for lab in LABS:
        files = sorted(p for p in (E / lab).rglob("*") if p.is_file())
        for p in files:
            originals.append(f"{lab}\t{p.relative_to(E / lab).as_posix()}\t{p.stat().st_size}\t{sha(p)}")
        print(f"{lab}: {len(files)} files, {sum(p.stat().st_size for p in files)} bytes")
    (BUNDLE / "provenance").mkdir(exist_ok=True)
    (BUNDLE / "provenance/originals.tsv").write_text("\n".join(originals) + "\n")

    # the scenes
    header = None
    receipts = {}
    lines = []
    for lab in LABS:
        rows = (E / lab / "receipt.tsv").read_text().splitlines()
        if header is None:
            header = rows[0]
        elif header != rows[0]:
            problems.append(f"{lab}: receipt header differs")
        for row in rows[1:]:
            f = row.split("\t")
            key = (f[0], f[1], f[2])
            if key in receipts:
                problems.append(f"two receipt lines for {key}")
            receipts[key] = (lab, f)
            lines.append(row)
    (view / "receipt.tsv").write_text("\n".join([header] + lines) + "\n")

    seen = {}
    sources = ["kind\tsubject\tcandidate\tlab\tframes\trows\tcols\ts_per_frame\trecords\tframes_sha256"]
    for kind in KINDS:
        (view / kind).mkdir()
        for lab in LABS:
            root = E / lab / kind
            if not root.is_dir():
                continue
            for scene in sorted(x for x in root.iterdir() if x.is_dir()):
                subject, candidate = scene.name.split("__")
                key = (kind, subject, candidate)
                if key in seen:
                    problems.append(f"{key} comes from {seen[key]} and {lab}")
                seen[key] = lab
                os.symlink(scene.resolve(), view / kind / scene.name)
                pngs = sorted(scene.glob("*.png"))
                if key not in receipts:
                    problems.append(f"{key}: no receipt line")
                    continue
                f = receipts[key][1]
                if receipts[key][0] != lab:
                    problems.append(f"{key}: receipt in {receipts[key][0]}, frames in {lab}")
                if int(f[3]) != len(pngs):
                    problems.append(f"{key}: receipt says {f[3]} frames, {len(pngs)} on disk")
                joined = hashlib.sha256("".join(sha(p) for p in pngs).encode()).hexdigest()
                sources.append(f"{kind}\t{subject}\t{candidate}\t{lab}\t{len(pngs)}\t{f[4]}\t{f[5]}\t{f[9]}\t{f[10]}\t{joined}")
    for key in receipts:
        if key not in seen and key[0] in KINDS:
            problems.append(f"{key}: receipt line, no scene")
    (BUNDLE / "provenance/scene-sources.tsv").write_text("\n".join(sources) + "\n")
    print(f"scenes joined: {len(seen)}; stills {sum(k[0] == 'stills' for k in seen)}, fades {sum(k[0] == 'fades' for k in seen)}, repaints {sum(k[0] == 'repaints' for k in seen)}")

    truth = view / "truth"
    truth.mkdir()
    found = 0
    for lab in LABS:
        root = E / lab / "truth"
        if root.is_dir():
            for ppm in sorted(root.glob("*.ppm")):
                os.symlink(ppm.resolve(), truth / ppm.name)
                found += 1
                print(f"truth {ppm.name} from {lab} {sha(ppm)[:16]}")
    print(f"truth portraits: {found}")

    # what built them
    wanted = {
        "A-plate-core.patch": sha(REPAIR / "patches/A-plate-core.patch"),
        "B-plate-lab.patch": sha(REPAIR / "patches/B-plate-lab.patch"),
    }
    frozen = {
        "src/plate_layout.rs": "candidate/explainer_corpus/complex_analysis_visual/src/plate_layout.rs",
        "examples/plate_lab.rs": "candidate/explainer_corpus/complex_analysis_visual/examples/plate_lab.rs",
        "src/chapters/draw.rs": "candidate/explainer_corpus/complex_analysis_visual/src/chapters/draw.rs",
    }
    for name, rel in frozen.items():
        same = sha(E / "crate" / name) == sha(REPAIR / rel)
        print(f"lab crate {name} equals my frozen candidate: {same}")
        if not same:
            problems.append(f"{name} differs from the frozen candidate")
    binaries = {}
    for inputs in ("preview-inputs01.json", "remaining720-inputs02.json"):
        files = json.loads((E / inputs).read_text())["files"]
        named = {pathlib.Path(x["path"]).name: x["sha256"] for x in files}
        binaries[inputs] = named.get("plate_lab")
        for patch, digest in wanted.items():
            ok = named.get(patch) == digest
            print(f"{inputs}: {patch} bound to my frozen bytes: {ok}")
            if not ok:
                problems.append(f"{inputs}: {patch} not bound")
    print(f"same plate_lab binary in both jobs: {len(set(binaries.values())) == 1}, sha256 {next(iter(binaries.values()))}")
    if len(set(binaries.values())) != 1:
        problems.append("the jobs ran different binaries")

    for job in ("complex-color-plates720-01", "complex-color-remaining720-02"):
        receipt = json.loads((JOBS / job / "resource-receipt.json").read_text())
        print(f"job {job}: exit {receipt['exit']}, {receipt['started']} to {receipt['finished']}, inputs held {receipt['inputs_sha256'] == receipt['final_inputs_sha256']}")
        if receipt["inputs_sha256"] != receipt["final_inputs_sha256"]:
            problems.append(f"{job}: inputs moved")
    tail = (JOBS / "complex-color-plates720-01/command.log").read_text().strip().splitlines()[-1]
    print(f"plates720-01 last log line: {tail}")

    for problem in problems:
        print("PROBLEM", problem)
    print("CONSOLIDATED" if not problems else f"{len(problems)} PROBLEMS")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1]))
