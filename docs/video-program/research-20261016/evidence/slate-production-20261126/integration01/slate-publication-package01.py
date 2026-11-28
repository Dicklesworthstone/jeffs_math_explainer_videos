"""Prepare one local delivery package from a successful native master."""

import argparse
import errno
import hashlib
from itertools import count
import json
import os
from pathlib import Path
import re
import runpy
import shutil


HERE = Path(__file__).resolve().parent
REPO = HERE.parents[5]
WORK = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
COPY = HERE / "publication-copy01.md"
COPY_REVIEW = WORK / "companion-thumbnails02/thumbnail-and-copy-readback01.json"
FLAGSHIP_THUMB = WORK / "anytime-valid-thumbnail01"
YOUTUBE = runpy.run_path(str(REPO / "tools/youtube_package.py"))
# The original reviewed copy is preserved. This opening scene shows weather forecasts.
CHAPTER_LABEL_CORRECTIONS = {
    ("conformal", "01_compare_the_forecasts"): "Two forecasts, one temperature",
}
FILMS = {
    "anytime": ("How to Catch a Change Without Crying Wolf",
                "e-process, e-values, optional stopping, conformal martingale, sequential testing",
                "FrankenTUI controller code\n"
                "https://github.com/Dicklesworthstone/frankentui/blob/63f3ab756f7e8830c25c2d9a2d6c870af1a8be0b/crates/ftui-render/src/budget.rs\n"
                "Vovk et al., Retrain or not retrain: Conformal test martingales for change-point detection\n"
                "https://proceedings.mlr.press/v152/vovk21b.html\n"
                "Wine chemistry data from Cortez et al., UCI Wine Quality"),
    "conformal": ("How Wide Should an Honest Prediction Be?",
                  "split conformal prediction, prediction intervals, uncertainty, exchangeability, statistics",
                  "Lei et al., Distribution-Free Predictive Inference for Regression\n"
                  "https://www.stat.berkeley.edu/~ryantibs/papers/conformal-jasa.pdf\n"
                  "Tibshirani et al., Conformal Prediction Under Covariate Shift\n"
                  "https://proceedings.neurips.cc/paper_files/paper/2019/file/8fb21ee7a2207526da55a679f0332de2-Paper.pdf\n"
                  "Wine chemistry data from Cortez et al., UCI Wine Quality"),
    "queues": ("Why Looking at Two Queues Changes Everything",
               "power of two choices, load balancing, queueing theory, probability, randomized algorithms",
               "Azar et al., Balanced Allocations\n"
               "https://cs.brown.edu/people/eupfal/papers/SICOMP29.pdf\n"
               "Mitzenmacher, The Power of Two Choices in Randomized Load Balancing\n"
               "https://www.eecs.harvard.edu/~michaelm/postscripts/mythesis.pdf"),
    "robotics": ("The Math Behind FrankenRobotics",
                 "FrankenRobotics, abstract algebra, group theory, robotics, Lie groups, kinematics",
                 "Body models from FrankenRobotics. For rigid transforms and wheeled robots, see Lynch and Park, Modern Robotics."),
}


def sha(path):
    with path.open("rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def file_checksums(path, expected=None):
    source = fingerprint(path)
    md5 = hashlib.md5(usedforsecurity=False)
    sha256 = hashlib.sha256()
    with path.open("rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            md5.update(block)
            sha256.update(block)
    if fingerprint(path) != source or (expected is not None and sha256.hexdigest() != expected):
        raise ValueError(f"Asset changed: {path}")
    return {"bytes": source[2], "md5": md5.hexdigest(), "sha256": sha256.hexdigest()}, source


def fingerprint(path):
    info = path.stat()
    return info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns


def read_json(path):
    return json.loads(path.read_text(encoding="utf-8"))


def stable_success(path):
    receipt = read_json(path)
    if (not receipt["admitted"] or not receipt["finished"] or receipt["exit"] != 0
            or receipt.get("stopped_for_resource_limit") or receipt.get("stop_reason")
            or receipt["inputs_sha256"] != receipt["final_inputs_sha256"]):
        raise ValueError(f"Unsuccessful or unstable guard: {path}")
    return receipt


def verified_starts(delivery, chapters, keys, total):
    pairs = list(chapters.items()) if isinstance(chapters, dict) else chapters
    if tuple(key for key, _ in pairs) != tuple(keys) or len(set(keys)) != len(keys):
        raise ValueError("Master chapters differ from the reviewed native registry.")
    paths = {p.name: p for p in delivery.glob("*.timeline.tsv")}
    if set(paths) != {f"{key}.timeline.tsv" for key in keys}:
        raise ValueError("The master directory has a missing or extra chapter timeline.")
    starts, frame = [], 0
    for key, count in pairs:
        fields = [line.split("\t") for line in paths[f"{key}.timeline.tsv"].read_text().splitlines()]
        headers = [line for line in fields if line[0] in ("chapter", "fps", "frames")]
        if (type(count) is not int or count <= 0 or len(headers) != 3
                or dict(headers) != {"chapter": key, "fps": "60", "frames": str(count)}):
            raise ValueError(f"Timeline and receipt disagree for {key}.")
        starts.append((key, frame, frame // 60))
        frame += count
    if frame != total:
        raise ValueError("Chapter frames do not sum to the verified master.")
    return starts


def publication_text(film, labels, starts):
    title, tags, sources = FILMS[film]
    sections = re.split(r"^## ", COPY.read_text(), flags=re.M)[1:]
    section, = [section for section in sections if section.splitlines()[0] == title]
    _, _, body = section.partition("\n")
    paragraphs, sep, chapters = body.strip().partition("\n\nChapters in order\n\n")
    if not sep or not paragraphs:
        raise ValueError(f"Missing description or chapters for {film}.")
    chapter_labels = re.findall(r"^\d+\. (.+)$", chapters, re.M)
    if (chapter_labels != list(labels.values())
            or [key for key, _, _ in starts] != list(labels)):
        raise ValueError("Reviewed chapter labels and master chapter order differ.")
    chapter_labels = [
        CHAPTER_LABEL_CORRECTIONS.get((film, key), label) for key, label in labels.items()
    ]
    draft = f"{paragraphs}\n\nChapters\n" + "".join(f"?:?? {label}\n" for label in chapter_labels)
    draft += f"\nSources\n{sources}\n\nAnimation made with FrankenManim. Narration uses a synthetic ElevenLabs voice.\n"
    desc = YOUTUBE["fill_text"](draft, [(key, seconds) for key, _, seconds in starts])
    parsed = [YOUTUBE["chapter_seconds"](match) for _, match in YOUTUBE["chapter_lines"](desc)]
    if (len(parsed) < 3 or parsed[0] != 0 or parsed != [s for _, _, s in starts]
            or any(b - a < YOUTUBE["CHAPTER_MIN_S"] for a, b in zip(parsed, parsed[1:]))
            or len(title) > YOUTUBE["TITLE_MAX"] or len(desc) > YOUTUBE["DESCRIPTION_MAX"]
            or YOUTUBE["tags_length"](tags.split(", ")) > YOUTUBE["TAGS_MAX"]
            or re.search(r"[<>]", title + desc) or "—" in title):
        raise ValueError("Metadata exceeds a YouTube limit or has invalid chapter times.")
    return {"title.txt": title + "\n", "description.txt": desc, "tags.txt": tags + "\n"}


def film_inputs(film):
    if film == "anytime":
        return (WORK / "anytime-valid-native-master01", "probability-native-4k60.mp4",
                "probability_native", FLAGSHIP_THUMB / "thumbnail-preview/probability_native_thumbnail.png",
                read_json(FLAGSHIP_THUMB / "thumbnail-viewed01.json")["image_sha256"])
    reader = "companion-native-delivery02.py" if film == "robotics" else "companion-native-delivery01.py"
    delivery = runpy.run_path(str(HERE / reader))
    plan = read_json(delivery["audio"]["registry"]["recording_plan_path"](film))
    entry, = [row for row in plan["films"] if row["film"] == film]
    thumbnail, = [row["original"] for row in read_json(COPY_REVIEW)["thumbnails"] if row["film"] == film]
    name = delivery["stage_job"](f"{film}-native-master01")
    stem = delivery["prepared"]["native_artifact_stem"](Path(entry["native_executable"]))
    return (WORK / name, f"{name}-4k60.mp4", stem,
            REPO / thumbnail["path"], thumbnail["sha256"])


def materialize_package(job, binding, assets, metadata):
    fingerprints = {name: file_checksums(path, expected) for name, (path, expected) in assets.items()}
    content = {name: text.encode("utf-8") for name, text in metadata.items()}
    manifest = json.loads(json.dumps({
        **binding,
        "assets": {
            **{name: data for name, (data, _) in fingerprints.items()},
            **{name: {"bytes": len(data), "md5": hashlib.md5(data, usedforsecurity=False).hexdigest(),
                      "sha256": hashlib.sha256(data).hexdigest()} for name, data in content.items()},
        },
        "scope": "Technical delivery package. Final picture, listening, fine-sync and independent "
                 "review require separate evidence.",
    }))
    for attempt in count(1):
        out = job / f"publication-package{attempt:02d}"
        record = out / "manifest01.json"
        if not out.exists():
            break
        try:
            previous = read_json(record)
        except (FileNotFoundError, json.JSONDecodeError):
            continue
        if previous != manifest:
            continue
        for name, hashes in manifest["assets"].items():
            asset = out / name
            if name in fingerprints and fingerprint(asset) == fingerprints[name][1]:
                continue  # The original was just checked. This is its hard link.
            if file_checksums(asset, hashes["sha256"])[0] != hashes:
                raise ValueError(f"Changed package asset: {asset}")
        return out

    out.mkdir()
    for name, (path, _) in assets.items():
        target = out / name
        checksums, original = fingerprints[name]
        try:
            os.link(path, target)
        except OSError as error:
            if error.errno != errno.EXDEV:
                raise
            with path.open("rb") as source, target.open("xb") as destination:
                shutil.copyfileobj(source, destination, length=1 << 20)
            if file_checksums(target, checksums["sha256"])[0] != checksums or fingerprint(path) != original:
                raise ValueError(f"Asset changed while copying: {path}")
        else:
            if fingerprint(target) != original:
                raise ValueError(f"Asset changed while linking: {path}")
    for name, data in content.items():
        with (out / name).open("xb") as handle:
            handle.write(data)
    with record.open("x", encoding="utf-8") as handle:
        json.dump(manifest, handle, indent=2)
        handle.write("\n")
    return out


def package(film):
    reviewed = read_json(COPY_REVIEW)["publication_copy"]
    if sha(COPY) != reviewed["sha256"]:
        raise ValueError("The reviewed publication copy changed.")
    labels = reviewed["chapter_labels"][film]
    job, film_name, binary, thumb, thumb_sha = film_inputs(film)
    guard, delivery = job / "resource-receipt.json", job / "delivery"
    if stable_success(guard)["phase"] != "native":
        raise ValueError("Expected a guarded native master.")
    record_path = delivery / "delivery01.json"
    record = read_json(record_path)
    frames = record["media"]["frames"]
    clock = record["remux_clock"]
    if (record["full_av_decode"] is not True or clock["video_packets"] != frames
            or clock["video_packet_payloads_unchanged"] is not True
            or clock["aac_packets_and_clocks_unchanged"] is not True):
        raise ValueError("The master is missing a technical verification.")
    starts = verified_starts(delivery, record["chapters"], labels, frames)
    metadata = publication_text(film, labels, starts)
    master = delivery / film_name
    if master.stat().st_size != record["media"]["bytes"]:
        raise ValueError("Master byte count differs from the verifier.")
    assets = {master.name: (master, record["sha256"][master.name]), "thumbnail.png": (thumb, thumb_sha)}
    captions = record.get("captions", {}).get("sha256", record["sha256"])
    for ext in ("srt", "vtt"):
        source = delivery / f"{binary}.en.{ext}"
        assets[f"captions.en.{ext}"] = (source, captions[source.name])
    out = materialize_package(job, {
        "film": film, "master_guard_sha256": sha(guard), "master_receipt_sha256": sha(record_path),
        "frames": frames, "chapter_starts": starts,
    }, assets, metadata)
    print(out)
    return out


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("film", choices=tuple(FILMS))
    package(parser.parse_args().film)
