"""Derive exact Frederick Surrey requests from the three native companion builds."""

import argparse
from hashlib import file_digest
import json
from pathlib import Path
import subprocess


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
FILMS = {
    "conformal": (
        "split-conformal-native11", "split_conformal_native", "split-conformal-review-repairs01",
        11, 80,
    ),
    "queues": (
        "power-of-two-native07", "power_of_two_native", "power-of-two-review-repair02",
        10, 37,
    ),
    "robotics": (
        "frankenrobotics-native-integrated17", "frankenrobotics_native_representation_maps",
        "frankenrobotics-representation03", 12, 133,
    ),
}


def sha(path):
    with path.open("rb") as stream:
        return file_digest(stream, "sha256").hexdigest()


def native_output(binary, command):
    return subprocess.check_output([str(binary), command], text=True).splitlines()


def recording_plan_path(film):
    return HERE / ("companion-recording-plan04.json" if film == "robotics" else "companion-recording-plan02.json")


def reviewed_robotics():
    source, name, job, _, _ = FILMS["robotics"]
    opening = "01_three_ways_to_multiply"
    closing = "12_algebra_at_work"
    current = reviewed_robotics_preview(source, name, job, (opening,))
    previous = reviewed_robotics_preview(
        "frankenrobotics-native-integrated16", "frankenrobotics_native_representation",
        "frankenrobotics-representation02", (opening, closing),
    )
    current_source = HERE / source
    previous_source = HERE / "frankenrobotics-native-integrated16"
    # Reuse the closing review only when the actual script, pinned dependencies
    # and all scene code besides the opening source match the older build.
    previous_files = {path.relative_to(previous_source) for path in previous if path.is_relative_to(previous_source)}
    current_files = {path.relative_to(current_source) for path in current if path.is_relative_to(current_source)}
    assert current_files == previous_files
    for relative in sorted(previous_files):
        if relative.as_posix() not in ("Cargo.toml", "src/group_foundations.rs"):
            assert sha(current_source / relative) == sha(previous_source / relative), relative
    old_cargo = (previous_source / "Cargo.toml").read_text()
    assert (current_source / "Cargo.toml").read_text() == old_cargo.replace(
        'name = "frankenrobotics_native_representation"', f'name = "{name}"',
    )
    # Chapter 12 imports shared helpers from this file. Verify the full source
    # outside Chapter 1, including those helpers, byte by byte.
    def other_scenes(root):
        source = (root / "src/group_foundations.rs").read_bytes()
        start = source.index(b"pub fn opening(")
        rest = source.index(b"\nfn pair_pose(", start)
        return source[:start], source[rest:]

    assert other_scenes(current_source) == other_scenes(previous_source)
    return current | previous


def reviewed_robotics_preview(source, name, job, keys):
    output = WORK / job
    review_path = output / "picture-review01/viewed-representation01.json"
    if not review_path.is_file():
        raise ValueError("FrankenRobotics recording is waiting for the revised opening and closing picture review.")
    receipt_path = output / "resource-receipt.json"
    manifest = WORK / "navier-production-20261120" / f"{job}-inputs.json"
    receipt, review = (json.loads(path.read_text()) for path in (receipt_path, review_path))
    assert receipt["admitted"] and receipt["finished"] and receipt["exit"] == 0, receipt_path
    assert not receipt.get("stop_reason") and not receipt.get("stopped_for_resource_limit"), receipt_path
    assert sha(manifest) == receipt["inputs_sha256"] == receipt["final_inputs_sha256"], manifest
    assert sha(receipt_path) == review["guard_receipt_sha256"], review_path
    binary = REPO / "target/debug" / name
    assert sha(binary) == review["binary_sha256"] and review["blocking_findings"] == [], review_path
    compiled = HERE / source
    frozen = {Path(row["path"]): row["sha256"] for row in json.loads(manifest.read_text())["files"]}
    build = {compiled / part for part in ("Cargo.toml", "Cargo.lock", "script.tsv")}
    build.update(path for path in (compiled / "src").rglob("*") if path.is_file())
    assert all(frozen.get(path) == sha(path) for path in build), compiled
    bound = {receipt_path, manifest, binary, review_path, *build}
    assert set(review["chapters"]) == set(keys), review_path
    for key in keys:
        plan_path = review_path.parent / f"sample-plan-{key}.json"
        checked = review["chapters"][key]
        sample = json.loads(plan_path.read_text())
        assert sha(plan_path) == checked["sample_plan_sha256"] and sample["chapter"] == key, plan_path
        assert checked["viewed_frames"] == sample["selected_frames"] > 0, key
        assert checked["viewed_sheets"] == [row["file"] for row in sample["sheets"]], key
        for row in sample["sheets"]:
            sheet = plan_path.parent / row["file"]
            assert sha(sheet) == row["sha256"], sheet
            bound.add(sheet)
        for suffix, digest in sample["source_sha256"].items():
            artifact = output / "pictures" / f"{key}.{suffix}"
            assert sha(artifact) == digest, artifact
            bound.add(artifact)
        bound.add(plan_path)
    return bound


def plan(film):
    source, name, preview, chapter_count, cue_count = FILMS[film]
    script = HERE / source / "script.tsv"
    binary = REPO / "target" / "debug" / name
    script_sha, binary_sha = sha(script), sha(binary)
    lines = [line for line in script.read_text().splitlines() if line and not line.startswith("#")]
    native = native_output(binary, "script")
    chapters = native_output(binary, "chapters")
    if native != lines or len(native) != cue_count:
        raise ValueError(f"{film}: built cue text differs from {script}")
    if len(chapters) != chapter_count or len(set(chapters)) != chapter_count:
        raise ValueError(f"{film}: unexpected native chapter registry")
    chapter_by_number = {chapter.partition("_")[0]: chapter for chapter in chapters}
    if list(chapter_by_number) != [f"{n:02d}" for n in range(1, chapter_count + 1)]:
        raise ValueError(f"{film}: chapter numbering changed")

    cues = []
    for line in lines:
        cue, speech = line.split("\t", 1)
        prefix = cue.partition("_")[0]
        if not speech.strip() or prefix not in chapter_by_number:
            raise ValueError(f"{film}: invalid cue {cue}")
        cues.append({"cue": cue, "chapter": chapter_by_number[prefix], "characters": len(speech),
                     "text": speech})
    if len({row["cue"] for row in cues}) != cue_count:
        raise ValueError(f"{film}: duplicate cue IDs")
    if list(dict.fromkeys(row["chapter"] for row in cues)) != chapters:
        raise ValueError(f"{film}: native chapter and narration order differ")
    if script_sha != sha(script) or binary_sha != sha(binary):
        raise ValueError(f"{film}: source changed during readback")

    receipt = WORK / preview / "resource-receipt.json"
    return {
        "film": film,
        "script": str(script.relative_to(REPO)),
        "script_sha256": script_sha,
        "native_executable": str(binary.relative_to(REPO)),
        "native_executable_sha256": binary_sha,
        "preview_receipt": str(receipt.relative_to(REPO)),
        "chapters": chapters,
        "characters": sum(row["characters"] for row in cues),
        "cues": cues,
    }


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=(*FILMS, "all"))
    args = parser.parse_args()
    selected = FILMS if args.film == "all" else (args.film,)
    if "robotics" in selected:
        reviewed_robotics()
    plans = [plan(film) for film in selected]
    print(json.dumps({
        "voice": "Frederick Surrey - Smooth and Velvety",
        "model": "Eleven v3",
        "stability": 0.5,
        "retained_format": "48 kHz mono signed 16-bit PCM WAV, whole takes",
        "scope": "Exact requests after the flagship. Preview clearance, quota, capture and listening checked separately.",
        "total_cues": sum(len(film["cues"]) for film in plans),
        "total_characters": sum(film["characters"] for film in plans),
        "films": plans,
    }, indent=2, ensure_ascii=False))


if __name__ == "__main__":
    main()
