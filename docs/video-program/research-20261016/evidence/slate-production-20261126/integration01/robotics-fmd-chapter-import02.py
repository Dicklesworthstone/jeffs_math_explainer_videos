"""Verify and adopt the FMD chapter for the original secondary Mac master."""

import argparse
import hashlib
import json
import os
from pathlib import Path
import runpy
import shutil
import socket
import sys
import tarfile
import wave


HERE = Path(__file__).resolve().parent
FMD_ROOT = Path("/data/projects/jmev-fmd-render-qualification-20261106-1111")
FMD_BUILD = "fmd-slate-robotics-build02"
CHAPTERS = {
    "04": ("04_same_group_different_motion", 10383, 143),
    "05": ("05_the_rover_bracket", 9355, 143),
    "06": ("06_floating_base_tree", 6583, 143),
    "07": ("07_symmetries_as_permutations", 13546, 145),
    "08": ("08_four_modes", 10538, 145),
}
TABLES = ("timeline.tsv", "cues.tsv", "cuelog.tsv", "layout.tsv")
MEDIA = (*TABLES, "mp4", "captured.wav")
META = (
    "resource-receipt.json", "inputs.json", "chapter-render01.json", "media-inputs.json",
    "portable-build01.json", "build-receipt.json", "build-inputs.json", "source-inputs.json",
)


def sha(path):
    with path.open("rb") as source:
        return hashlib.file_digest(source, "sha256").hexdigest()


def read(path):
    return json.loads(path.read_text())


def write(path, value):
    with path.open("x") as target:
        json.dump(value, target, indent=2)
        target.write("\n")


def successful(receipt, inputs):
    state = read(receipt)
    assert state["phase"] == "native" and state["admitted"] and state.get("finished") and state["exit"] == 0, receipt
    assert not state.get("stopped_for_resource_limit") and not state.get("stop_reason"), receipt
    assert sha(inputs) == state["inputs_sha256"] == state["final_inputs_sha256"], receipt


def bundle():
    assert socket.gethostname().split(".")[0] == "fixmydocuments"
    job = FMD_ROOT / "jobs" / FMD_JOB
    build = FMD_ROOT / "jobs" / FMD_BUILD
    runner = FMD_ROOT / "work/slate-robotics-linux01/fmd-render-remaining01.py"
    assert sha(runner) == "180bf5b5bcf985f6d246855d7a20ff507cc58e742d5c9014dda9027e8f5115d1"
    paths = {
        "resource-receipt.json": job / "resource-receipt.json",
        "inputs.json": FMD_ROOT / f"{FMD_JOB}-inputs.json",
        "chapter-render01.json": job / "chapter-render01.json",
        "media-inputs.json": FMD_ROOT / "inputs/slate-robotics-media01" / f"chapter{NUMBER}-transfer-inputs01.json",
        "portable-build01.json": build / "portable-build01.json",
        "build-receipt.json": build / "resource-receipt.json",
        "build-inputs.json": FMD_ROOT / f"{FMD_BUILD}-inputs.json",
        "source-inputs.json": runner.with_name("transfer-inputs01.json"),
    }
    successful(paths["resource-receipt.json"], paths["inputs.json"])
    successful(paths["build-receipt.json"], paths["build-inputs.json"])
    bindings = {Path(item["path"]): item["sha256"] for item in read(paths["inputs.json"])["files"]}
    assert len(bindings) >= 145 and bindings[runner] == sha(runner)
    for item, digest in bindings.items():
        assert sha(item) == digest, item
    report = read(paths["chapter-render01.json"])
    assert report["chapter"] == KEY and report["frames"] == FRAMES
    for extension in MEDIA:
        entry = report["files"][extension]
        source = (job / entry["path"]).resolve()
        assert source.is_relative_to(job / "pictures") and sha(source) == entry["sha256"]
        paths[extension] = source
    assert set(paths) == set(META) | set(MEDIA)
    with tarfile.open(mode="w|", fileobj=sys.stdout.buffer) as archive:
        for name, source in paths.items():
            assert source.is_file() and not source.is_symlink()
            archive.add(source, arcname=name, recursive=False)


def receive(raw, work):
    overhead = 32 << 20
    max_bytes = 512 << 20
    assert shutil.disk_usage(work).free >= (10 << 30) + overhead + max_bytes
    raw.mkdir(parents=True)
    expected = set(META) | set(MEDIA)
    copied = set()
    total = 0
    with tarfile.open(mode="r|", fileobj=sys.stdin.buffer) as archive:
        for member in archive:
            name = member.name
            assert member.isfile() and name in expected and name not in copied, name
            total += member.size
            assert total <= max_bytes
            with archive.extractfile(member) as source, (raw / name).open("xb") as target:
                shutil.copyfileobj(source, target, length=1 << 20)
            assert (raw / name).stat().st_size == member.size
            copied.add(name)
    assert copied == expected


def checked_inputs(production, repo, work, raw):
    inputs = production["score_inputs"]("robotics", work / "robotics-frederick-voice01",
                                        work / "robotics-final-clock01")
    binary, _, _, frames, _, _, _, paths = inputs
    assert frames[KEY] == FRAMES and len(frames) == 12 and sum(frames.values()) == 128_487
    successful(raw / "resource-receipt.json", raw / "inputs.json")
    successful(raw / "build-receipt.json", raw / "build-inputs.json")
    source = read(raw / "source-inputs.json")
    build = read(raw / "portable-build01.json")
    report = read(raw / "chapter-render01.json")
    assert sha(raw / "source-inputs.json") == build["source_manifest_sha256"]
    assert sha(binary) == source["mac_binary_sha256"]
    assert source["engine_revision"] == "a399d626c43ecc852a5c795288979f52b446bc5c"
    assert all(build["same_as_mac"][key] == source["files"][f"expected/{key}.txt"]["sha256"]
               for key in ("script", "chapters", "facts"))
    assert report["source_build_sha256"] == sha(raw / "portable-build01.json")
    assert report["chapter"] == KEY and report["frames"] == FRAMES
    assert report["render_options"] == list(production["RENDER_OPTIONS"])
    media = read(raw / "media-inputs.json")
    assert media["chapter"] == KEY and media["frames"] == FRAMES and Path(media["mac_workspace"]) == repo
    frozen = {Path(item["path"]): item["sha256"] for item in read(raw / "inputs.json")["files"]}
    remote = FMD_ROOT / "inputs/slate-robotics-media01"
    assert len(media["files"]) == ASSET_COUNT and frozen[remote / f"chapter{NUMBER}-transfer-inputs01.json"] == sha(raw / "media-inputs.json")
    for relative, record in media["files"].items():
        item = repo / relative
        assert item.is_relative_to(work) and not item.is_symlink(), item
        assert item.stat().st_size == record["bytes"] and sha(item) == record["sha256"], item
        assert frozen[remote / relative] == record["sha256"]
        paths.add(item)
    for extension in MEDIA:
        assert sha(raw / extension) == report["files"][extension]["sha256"], extension
    assert (raw / "layout.tsv").read_text().splitlines() == report["layout"]
    return inputs, remote


def normalize(raw, output, remote, repo):
    output.mkdir()
    for extension in MEDIA:
        target = output / f"{KEY}.{extension}"
        if extension in ("timeline.tsv", "cuelog.tsv"):
            text = (raw / extension).read_text()
            with target.open("x") as file:
                file.write(text.replace(f"{remote}/", f"{repo}/"))
        else:
            os.link(raw / extension, target)


def check_media(production, inputs, raw, normalized, remote, repo):
    _, narration, _, _, timeline, stems, score, _ = inputs
    files = {extension: normalized / f"{KEY}.{extension}" for extension in MEDIA}
    for extension, file in files.items():
        if extension in ("timeline.tsv", "cuelog.tsv"):
            assert file.read_text() == (raw / extension).read_text().replace(f"{remote}/", f"{repo}/")
        else:
            assert sha(file) == sha(raw / extension)
    for extension in ("timeline.tsv", "cues.tsv"):
        assert files[extension].read_bytes() == (timeline / f"{KEY}.{extension}").read_bytes()
    shared = production["shared"]
    shared["verify_cue_log"](files["cuelog.tsv"], timeline / f"{KEY}.cuelog.tsv", stems, score[KEY]["parts"])
    assert narration.is_dir()
    media = shared["verify_av"](files["mp4"], FRAMES)
    with wave.open(str(files["captured.wav"]), "rb") as pcm:
        assert (pcm.getnchannels(), pcm.getsampwidth(), pcm.getframerate(), pcm.getnframes()) == (2, 2, 48000, FRAMES * 800)
    return files, media


def target(mode):
    assert socket.gethostname().lower().split(".")[0] == "mac-mini-max"
    host = runpy.run_path(str(HERE / "queues-secondary-clock01.py"))
    repo, work, wrapper = (host[name] for name in ("REPO", "WORK", "WRAPPER"))
    delivery = HERE / "companion-native-delivery02.py"
    production = runpy.run_path(str(delivery))
    stage = work / f"robotics-fmd-ch{NUMBER}-import01"
    raw, normalized = stage / "raw", stage / "normalized"
    if mode == "receive":
        receive(raw, work)
        mode = "review"
    inputs, remote = checked_inputs(production, repo, work, raw)
    if mode == "review" and not normalized.exists():
        normalize(raw, normalized, remote, repo)
    files, media = check_media(production, inputs, raw, normalized, remote, repo)
    plan_path = stage / "sample-plan01.json"
    sheets = stage / "picture-review01"
    if mode == "review":
        sampler = runpy.run_path(str(HERE / "slate-master-pictures01.py"))
        sheets.mkdir()
        tables = {ext: files[ext] for ext in sampler["TABLES"]}
        plan = sampler["sheets"](files["mp4"], sheets, KEY, 0, FRAMES, tables)
        write(plan_path, {"chapter": KEY, "video_sha256": sha(files["mp4"]), **plan})
        print(f"{KEY}: staged {plan['samples']} pictures for review.")
        return
    plan = read(plan_path)
    assert plan["chapter"] == KEY and plan["frames"] == FRAMES and plan["video_sha256"] == sha(files["mp4"])
    assert all(value == sha(files[key]) for key, value in plan["source_sha256"].items())
    for sheet in plan["sheets"]:
        assert sha(sheets / sheet["file"]) == sheet["sha256"]
    viewed_path = stage / "viewed01.json"
    viewed = read(viewed_path)
    assert viewed["sample_plan_sha256"] == sha(plan_path) and not viewed["blocking_findings_in_samples"]
    assert viewed["sheets_viewed"] == len(plan["sheets"]) and viewed["pictures_viewed"] == plan["samples"]
    job = production["chapter_job"]("robotics", KEY)
    out = work / job
    manifest = production["input_manifest"](job)
    if mode == "execute":
        state = read(out / "resource-receipt.json")
        assert state["admitted"] and state["phase"] == "native" and not state.get("finished")
        pictures = out / "pictures"
        pictures.mkdir()
        for extension, path in files.items():
            name = pictures / f"{KEY}.{extension}"
            if extension == "captured.wav":
                name = pictures / "soundtrack" / KEY / "soundtrack_0.wav"
                name.parent.mkdir(parents=True)
            os.link(path, name)
        write(out / "chapter-render01.json", {
            "chapter": KEY, "media": media, "render_options": list(production["RENDER_OPTIONS"]),
            "sha256": {ext: sha(path) for ext, path in files.items()}, "layout": files["layout.tsv"].read_text().splitlines(),
            "fmd_guard_sha256": sha(raw / "resource-receipt.json"), "visual_review_sha256": sha(viewed_path),
        })
        return
    assert mode == "adopt" and not out.exists() and not manifest.exists()
    baseline = work / "allocation-baseline.json"
    assert sha(baseline) == "5df0f17125e127d943c4478cc21d23439902a8e7e8ee696bf9008831fd484595"
    assert sha(wrapper) == "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097"
    layer = runpy.run_path(str(wrapper))
    guard = layer["load"]()
    assert guard.WORK == work and guard.REPO == repo and guard.TARGET == repo / "target"
    assert guard.ADMISSION_GIB["native"] == 10 and guard.STOP_FREE_GIB == 8
    assert guard.CAPS["native"] == 4 and guard.TARGET_GROWTH_CAP_GIB == 20
    guard.WORK_CAP_GIB = 200
    paths = inputs[-1] | {Path(__file__).resolve(), delivery, wrapper, layer["SOURCE"], baseline,
                          plan_path, viewed_path, *raw.iterdir(), *files.values()}
    paths.update(sheets / sheet["file"] for sheet in plan["sheets"])
    write(manifest, {"files": [{"path": str(path), "sha256": sha(path)} for path in sorted(paths)]})
    sys.argv = [__file__, "--phase", "native", "--job", job, "--inputs", str(manifest), "--",
                sys.executable, "-I", "-B", __file__, "execute", NUMBER]
    raise SystemExit(guard.main())


if __name__ == "__main__":
    assert __debug__
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("mode", choices=("bundle", "receive", "review", "adopt", "execute"))
    parser.add_argument("chapter", choices=tuple(CHAPTERS))
    args = parser.parse_args()
    stage = args.mode
    NUMBER = args.chapter
    KEY, FRAMES, ASSET_COUNT = CHAPTERS[NUMBER]
    FMD_JOB = f"fmd-slate-robotics-render{NUMBER}-01"
    if stage == "bundle":
        bundle()
    else:
        target(stage)
