#!/usr/bin/env python3
"""Assemble a per-video release bundle and YouTube package (plan §13, bead mx-e-w8-4ol.1).

Dry-run by default: assembles `<slug>_4k_vN.mp4`, `<slug>_1080p_vN.mp4` (x264 CRF 18,
audio copied), `<slug>.en.srt`, `<slug>.en.vtt`, `contact_sheet.png`, the thumbnail youtube.md names,
`release_notes.md` and `SHA256SUMS` under the bundle directory (default `release_vN/` beside the cut's
`final/`, under the ignored `renders/`: a 4K MP4 never sits in the tracked tree), once `publish/youtube.md`
and `publish/michael/*.txt` pass tools/youtube_package.py's check (against the master's timelines when it
has them) and docs/publish/MANIFEST.md names no other md5 for the master.
`--upload` creates the GitHub Release `<slug>-vN` with the bundle as its assets (`gh release create`,
the title from title.txt, the notes from release_notes.md), refusing a tag that exists; it refuses to
run at all unless `ALLOW_PUBLISH_UPLOAD=1` is set in the environment (G10: the owner's approval).
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def fmn_commit(root: Path) -> str:
    """The franken_manim commit the workspace builds against: Cargo.lock's resolved source (`...franken_manim
    ?rev=R#SHA`), else Cargo.toml's `rev = "..."` on the franken_manim git dependency."""
    lock = root / "Cargo.lock"
    if lock.is_file():
        m = re.search(r'franken_manim\?rev=[0-9a-f]*#([0-9a-f]{7,40})', lock.read_text(encoding="utf-8"))
        if m:
            return m.group(1)[:12]
    cargo_toml = root / "Cargo.toml"
    if cargo_toml.is_file():
        m = re.search(r'franken_manim"\s*,\s*rev\s*=\s*"([0-9a-f]{7,40})"', cargo_toml.read_text(encoding="utf-8"))
        if m:
            return m.group(1)[:12]
    return "unknown"


def probe_video(mp4: Path) -> dict[str, str]:
    cmd = [
        "ffprobe",
        "-v",
        "error",
        "-select_streams",
        "v:0",
        "-show_entries",
        "stream=width,height,r_frame_rate,codec_name:format=duration",
        "-of",
        "json",
        str(mp4),
    ]
    res = subprocess.run(cmd, capture_output=True, text=True, check=True, timeout=60)
    data = json.loads(res.stdout)
    stream = (data.get("streams") or [{}])[0]
    fmt = data.get("format") or {}
    return {
        "width": str(stream.get("width", "")),
        "height": str(stream.get("height", "")),
        "r_frame_rate": str(stream.get("r_frame_rate", "")),
        "codec_name": str(stream.get("codec_name", "")),
        "duration": f"{float(fmt.get('duration', 0.0)):.2f}s",
    }


def build_1080p_proxy(master_mp4: Path, out_mp4: Path) -> None:
    cmd = [
        "ffmpeg",
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-i",
        str(master_mp4),
        "-vf",
        "scale=1920:1080:flags=lanczos",
        "-c:v",
        "libx264",
        "-crf",
        "18",
        "-preset",
        "fast",
        "-pix_fmt",
        "yuv420p",
        "-c:a",
        "copy",
        str(out_mp4),
    ]
    subprocess.run(cmd, check=True, timeout=600)


def build_contact_sheet(master_mp4: Path, out_png: Path, tiles: int = 8) -> None:
    """Eight stills spread over the whole cut (the middle of each eighth), 4x2 at 480x270. Tiling the first
    frames instead gave a sheet of the opening's dark first eighth of a second."""
    duration = float(probe_video(master_mp4)["duration"].rstrip("s"))
    step = duration / tiles
    cmd = [
        "ffmpeg",
        "-hide_banner",
        "-loglevel",
        "error",
        "-y",
        "-ss",
        f"{step / 2:.3f}",
        "-i",
        str(master_mp4),
        "-vf",
        f"fps=1/{step:.6f},scale=480:270:flags=lanczos,tile=4x2",
        "-frames:v",
        "1",
        str(out_png),
    ]
    subprocess.run(cmd, check=True, timeout=300)


def format_srt_ts(seconds: float) -> str:
    ms = round(max(0.0, seconds) * 1000)
    h, rem = divmod(ms, 3_600_000)
    m, rem = divmod(rem, 60_000)
    s, ms = divmod(rem, 1000)
    return f"{h:02d}:{m:02d}:{s:02d},{ms:03d}"


def format_vtt_ts(seconds: float) -> str:
    return format_srt_ts(seconds).replace(",", ".")


def wrap_caption(text: str, max_chars: int = 42) -> str:
    words = text.split()
    lines: list[str] = []
    cur: list[str] = []
    for w in words:
        cand = " ".join(cur + [w])
        if cur and len(cand) > max_chars and len(lines) < 1:
            lines.append(" ".join(cur))
            cur = [w]
        else:
            cur.append(w)
    if cur:
        lines.append(" ".join(cur))
    return "\n".join(lines[:2])


def ensure_captions(crate: Path, finals: Path, bundle: Path, slug: str) -> None:
    srt_dst = bundle / f"{slug}.en.srt"
    vtt_dst = bundle / f"{slug}.en.vtt"
    existing_srt = list(finals.glob("*.en.srt")) + list((crate / "publish").glob("**/*.en.srt"))
    existing_vtt = list(finals.glob("*.en.vtt")) + list((crate / "publish").glob("**/*.en.vtt"))
    if existing_srt and existing_vtt:
        shutil.copy2(existing_srt[0], srt_dst)
        shutil.copy2(existing_vtt[0], vtt_dst)
        return

    # Fall back to building captions from narration_v5/report.json or script.tsv
    report_candidates = [
        finals / "report.json",
        crate / "narration" / "out" / "report.json",
        crate / "narration_v5" / "report.json",
    ]
    entries: list[tuple[float, float, str]] = []
    for rp in report_candidates:
        if rp.is_file():
            data = json.loads(rp.read_text(encoding="utf-8"))
            rate = float((data.get("meta") or {}).get("rate", 24000))
            t = 0.0
            lines = data.get("lines") or {}
            if isinstance(lines, dict):
                for sents in lines.values():
                    if not isinstance(sents, list):
                        continue
                    for s in sents:
                        dur = float(s.get("out_samples", 24000)) / rate
                        gap = float(s.get("gap_after_samples", 0)) / rate
                        txt = str(s.get("sentence") or s.get("text") or "").strip()
                        if txt:
                            entries.append((t, t + dur, txt))
                        t += dur + gap
            elif isinstance(lines, list):
                for item in lines:
                    dur = float(item.get("duration_s", 2.5))
                    txt = str(item.get("text") or item.get("sentence") or "").strip()
                    if txt:
                        entries.append((t, t + dur, txt))
                    t += dur + 0.3
            if entries:
                break

    if not entries and (crate / "script.tsv").is_file():
        t = 0.0
        for raw in (crate / "script.tsv").read_text(encoding="utf-8").splitlines():
            if not raw.strip() or raw.startswith(("#", "id\t")):
                continue
            parts = raw.split("\t")
            if len(parts) >= 2:
                txt = parts[1].strip()
                dur = max(1.5, len(txt.split()) / 2.6)
                entries.append((t, t + dur, txt))
                t += dur + 0.35

    srt_blocks = []
    vtt_blocks = ["WEBVTT", ""]
    for idx, (t0, t1, txt) in enumerate(entries, start=1):
        wrapped = wrap_caption(txt)
        srt_blocks.append(f"{idx}\n{format_srt_ts(t0)} --> {format_srt_ts(t1)}\n{wrapped}\n")
        vtt_blocks.append(f"{format_vtt_ts(t0)} --> {format_vtt_ts(t1)}\n{wrapped}\n")
    srt_dst.write_text("\n".join(srt_blocks), encoding="utf-8")
    vtt_dst.write_text("\n".join(vtt_blocks), encoding="utf-8")


def ensure_youtube_md(crate: Path, slug: str, finals: Path) -> Path:
    """The video's checked YouTube package. It is written by hand from docs/description.md (a generated
    stand-in once said "0:00 Introduction" under a slug-cased title); a missing or failing package stops
    the release here, with every problem named (tools/youtube_package.py)."""
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import youtube_package  # noqa: PLC0415 (beside this file; -I leaves the script's directory off the path)

    with_finals = finals if list(finals.glob("*.timeline.tsv")) else None
    bad = youtube_package.check(slug, "michael", with_finals, crate.parents[1])
    if bad:
        raise SystemExit(
            "publish: the YouTube package is not ready (write publish/youtube.md and publish/michael/"
            "{title,description,tags}.txt from docs/description.md; fill the chapter times with "
            f"tools/youtube_package.py fill {slug} --finals DIR):\n  " + "\n  ".join(bad)
        )
    return crate / "publish" / "youtube.md"


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Assemble release bundle and YouTube package.")
    parser.add_argument("slug", help="Explainer slug under explainer_corpus/")
    parser.add_argument("--version", default="v1", help="Release version suffix (default: v1)")
    parser.add_argument("--master", type=Path, help="Explicit path to verified 4K master MP4")
    parser.add_argument("--out", type=Path, help="Output bundle directory")
    parser.add_argument("--upload", action="store_true", help="Upload release via gh (requires ALLOW_PUBLISH_UPLOAD=1)")
    args = parser.parse_args(argv)

    crate = ROOT / "explainer_corpus" / args.slug
    if not crate.is_dir():
        print(f"publish: no explainer at {crate}", file=sys.stderr)
        return 2

    if args.upload and os.environ.get("ALLOW_PUBLISH_UPLOAD") != "1":
        print(
            "publish: refusing automatic upload without explicit owner approval (G10: set ALLOW_PUBLISH_UPLOAD=1)",
            file=sys.stderr,
        )
        return 2

    finals = crate / "renders" / "final"
    master_mp4 = args.master
    if master_mp4 is None:
        michael_finals = crate / "variants" / "michael" / "renders" / "final"
        candidates = sorted(michael_finals.glob("*_master.mp4")) + sorted(finals.glob("*_master.mp4"))
        if candidates:
            master_mp4 = candidates[0]
    if master_mp4 is None or not master_mp4.is_file():
        print(f"publish: no master MP4 found for {args.slug} (pass --master or run ./x {args.slug} master)", file=sys.stderr)
        return 2
    finals = master_mp4.parent
    # the upload text first: a bundle with no package to go with it is no release
    yt_path = ensure_youtube_md(crate, args.slug, finals)
    import release  # noqa: PLC0415 (beside this file, on the path ensure_youtube_md set)

    # the cut MANIFEST records, and no other
    stale = release.manifest_mismatch(ROOT, master_mp4.resolve())
    if stale:
        print("publish: the master is not the cut MANIFEST records:\n  " + "\n  ".join(stale), file=sys.stderr)
        return 2

    ver = args.version if args.version.startswith("v") else f"v{args.version}"
    tag = f"{args.slug}-{ver}"
    if args.upload and subprocess.run(["gh", "release", "view", tag], capture_output=True).returncode == 0:
        print(f"publish: the release {tag} exists already (a new cut takes a new --version)", file=sys.stderr)
        return 2
    bundle = args.out or (finals.parent / f"release_{ver}")
    bundle.mkdir(parents=True, exist_ok=True)

    master_dst = bundle / f"{args.slug}_4k_{ver}.mp4"
    proxy_dst = bundle / f"{args.slug}_1080p_{ver}.mp4"
    sheet_dst = bundle / "contact_sheet.png"
    notes_dst = bundle / "release_notes.md"

    shutil.copy2(master_mp4, master_dst)
    build_1080p_proxy(master_dst, proxy_dst)

    existing_sheets = sorted(finals.glob("*contact*.png"))
    if existing_sheets:
        shutil.copy2(existing_sheets[0], sheet_dst)
    else:
        build_contact_sheet(master_dst, sheet_dst)

    ensure_captions(crate, finals, bundle, args.slug)

    thumb_rel = release.thumbnail_rel(crate)
    thumb_row = "none named in publish/youtube.md"
    if thumb_rel and (crate / thumb_rel).is_file():
        shutil.copy2(crate / thumb_rel, bundle / Path(thumb_rel).name)
        thumb_row = f"`{Path(thumb_rel).name}` (1280x720, the YouTube thumbnail)"
    elif thumb_rel:
        thumb_row = f"missing: `{thumb_rel}` (./x {args.slug} thumbnail)"

    meta_4k = probe_video(master_dst)
    meta_1080 = probe_video(proxy_dst)
    commit = fmn_commit(ROOT)
    repo_commit = subprocess.run(["git", "-C", str(ROOT), "rev-parse", "--short", "HEAD"],
                                 capture_output=True, text=True).stdout.strip() or "unknown"

    notes = f"""# Release `{args.slug}-{ver}`

| Field | Value |
|---|---|
| Explainer | `{args.slug}` |
| Version | `{ver}` |
| Repository commit | `{repo_commit}` |
| `franken_manim` commit | `{commit}` |
| 4K master | `{master_dst.name}` (`{meta_4k['width']}x{meta_4k['height']}`, `{meta_4k['r_frame_rate']}` fps, `{meta_4k['codec_name']}`, `{meta_4k['duration']}`) |
| 1080p proxy | `{proxy_dst.name}` (`{meta_1080['width']}x{meta_1080['height']}`, `{meta_1080['r_frame_rate']}` fps, `{meta_1080['codec_name']}`) |
| Captions | `{args.slug}.en.srt`, `{args.slug}.en.vtt` |
| Thumbnail | {thumb_row} |
| YouTube package | `{yt_path.relative_to(ROOT)}` |
"""
    notes_dst.write_text(notes, encoding="utf-8")

    sha_lines: list[str] = []
    for p in sorted(bundle.iterdir()):
        if not p.is_file() or p.name in ("SHA256SUMS", "SHA256SUMS.txt"):
            continue
        digest = hashlib.sha256(p.read_bytes()).hexdigest()
        sha_lines.append(f"{digest}  {p.name}")
    (bundle / "SHA256SUMS").write_text("\n".join(sha_lines) + "\n", encoding="utf-8")

    print(f"publish: assembled {tag} in {bundle} (dry-run={not args.upload})")
    if not args.upload:
        return 0
    title = (crate / "publish" / "michael" / "title.txt").read_text(encoding="utf-8").strip()
    assets = [str(p) for p in sorted(bundle.iterdir()) if p.is_file()]
    made = subprocess.run(["gh", "release", "create", tag, *assets, "--title", f"{title} ({ver})",
                           "--notes-file", str(notes_dst)])
    if made.returncode != 0:
        print(f"publish: gh release create {tag} failed ({made.returncode})", file=sys.stderr)
        return made.returncode
    print(f"publish: released {tag} with {len(assets)} assets")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
