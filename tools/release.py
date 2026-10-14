#!/usr/bin/env python3
"""tools/release.py: a video's verified final onto the upload machine, and the one check before its upload.

    ./x SLUG release-fetch --from HOST:CLONE    copy the final from a finals clone on the build host
    ./x SLUG release-check                      everything an upload needs, checked; the upload sheet

(`python3 -I tools/release.py fetch|check SLUG --finals DIR [--voice V] ...`; ./x passes the finals directory
of the video's voice.) The finals are rendered and verified on the build host in a clone of their own; the
upload lane uploads from this checkout, from the paths docs/publish/MANIFEST.md names. Nothing here uploads,
changes visibility, edits MANIFEST or deletes a file.

`fetch` copies from CLONE's same finals directory the master, the SRT and VTT and every timeline, and the
thumbnail youtube.md names, each md5-checked against the host's copy before it takes its place. A local
final whose master differs is first moved aside whole to final_vN (the next free N): a cut is preserved
before another replaces it.

`check` exits 1 with one line per problem, or prints PASS and the upload sheet:
- the files: one master, one SRT, one VTT, the timelines, the thumbnail youtube.md names;
- the master: 3840x2160 at 60 fps, as long as its timelines (0.1 s);
- the upload text: tools/youtube_package.py check against these timelines (YouTube's limits, the
  byte-for-byte copy, chapter times of this cut, protected description and title phrases);
- the captions: the SRT parses, its cues rise without overlap and end inside the master, the VTT carries
  the same texts, and docs/protected.tsv's captions rows stand in it; with SKILLS set, the skill's
  protected_check.py over the script, the SRT and the upload text (`./x SLUG protected`);
- the thumbnail: a PNG, 1280x720, at most 2 MB;
- docs/publish/MANIFEST.md: a row naming this master's path carries its md5.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))

import youtube_package  # noqa: E402

ROOT = Path(__file__).resolve().parents[1]
THUMB_SIZE = (1280, 720)
THUMB_MAX_BYTES = 2 * 1024 * 1024
DURATION_TOLERANCE_S = 0.1
STAMP = re.compile(r"^(\d\d):(\d\d):(\d\d)[,.](\d\d\d) --> (\d\d):(\d\d):(\d\d)[,.](\d\d\d)")


def md5(path: Path) -> str:
    h = hashlib.md5()
    with path.open("rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            h.update(block)
    return h.hexdigest()


def thumbnail_rel(crate: Path) -> str | None:
    """The thumbnail's crate-relative path, from youtube.md's '- Thumbnail:' line."""
    yt = crate / "publish" / "youtube.md"
    if not yt.is_file():
        return None
    m = re.search(r"^- Thumbnail: `([^`]+)`", yt.read_text(encoding="utf-8"), re.M)
    return m.group(1) if m else None


def final_files(names: list[str]) -> list[str]:
    """The files of a final the upload needs, by name: the master, captions and timelines."""
    keep = (lambda n: n.endswith("_master.mp4") or n.endswith(".srt") or n.endswith(".vtt")
            or n.endswith(".timeline.tsv"))
    return sorted(n for n in names if keep(n))


# ------------------------------------------------------------------ fetch


class Host:
    """The build host over ssh/scp. `fetch` takes any object with list, md5s, copy and sha (the tests pass a local one)."""

    def __init__(self, host: str):
        self.host = host

    def _ssh(self, command: str) -> str:
        return subprocess.run(["ssh", self.host, command], capture_output=True, text=True, check=True).stdout

    def list(self, directory: str) -> list[str]:
        out = self._ssh(f"ls -1 {shlex.quote(directory)} 2>/dev/null || true")
        return [line for line in out.splitlines() if line]

    def md5s(self, directory: str, names: list[str]) -> dict[str, str]:
        """{name: md5} of the named files in `directory` that exist there (one ssh call)."""
        if not names:
            return {}
        files = " ".join(shlex.quote(n) for n in names)
        out = self._ssh(f"cd {shlex.quote(directory)} 2>/dev/null && "
                        f"for f in {files}; do test -f \"$f\" && (md5sum \"$f\" 2>/dev/null || md5 -r \"$f\"); done; true")
        found = {}
        for line in out.splitlines():
            digest, _, name = line.strip().partition(" ")
            found[name.strip().lstrip("*")] = digest
        return found

    def copy(self, path: str, local: Path) -> None:
        subprocess.run(["scp", "-q", f"{self.host}:{path}", str(local)], check=True)

    def sha(self, clone: str) -> str:
        return self._ssh(f"git -C {shlex.quote(clone)} log -1 --format=%h 2>/dev/null || true").strip() or "?"


def expand_remote(clone: str) -> str:
    """A clone path as ssh and scp take it: absolute, or relative to the host's home (a quoted ~ never expands)."""
    return clone[2:] if clone.startswith("~/") else clone


def take(host, remote: str, want: str, local: Path, log: list[str]) -> str:
    """Copy `remote` (md5 `want` on the host) to `local` through a temporary name beside it, md5-checked; an
    older different `local` is kept as <stem>.vN<suffix>. Returns the md5."""
    if local.is_file() and md5(local) == want:
        log.append(f"  same    {local.name} {want}")
        return want
    part = local.with_name(local.name + ".part")
    host.copy(remote, part)
    got = md5(part)
    if got != want:
        part.rename(part.with_name(part.name + ".bad"))
        raise RuntimeError(f"{remote}: md5 {got} after the copy, the host has {want} (kept as {part.name}.bad)")
    if local.exists():
        aside = next(local.with_name(f"{local.stem}.v{n}{local.suffix}") for n in range(1, 10_000)
                     if not local.with_name(f"{local.stem}.v{n}{local.suffix}").exists())
        local.rename(aside)
        log.append(f"  aside   {local.name} -> {aside.name}")
    part.rename(local)
    log.append(f"  fetched {local.name} {want}")
    return want


def preserve(finals: Path) -> Path | None:
    """Move a final aside whole to the next free final_vN beside it; returns where it went."""
    if not finals.is_dir() or not any(finals.iterdir()):
        return None
    aside = next(finals.with_name(f"{finals.name}_v{n}") for n in range(1, 10_000)
                 if not finals.with_name(f"{finals.name}_v{n}").exists())
    finals.rename(aside)
    return aside


def fetch(slug: str, finals: Path, source: str, host=None, root: Path = ROOT) -> list[str]:
    """Bring the final at HOST:CLONE onto this checkout; returns the log lines (raises on a failed copy)."""
    if ":" not in source:
        raise ValueError(f"--from wants HOST:CLONE, got {source!r}")
    name, clone = source.split(":", 1)
    host = host or Host(name)
    clone = expand_remote(clone)
    rel = finals.resolve().relative_to(root.resolve()).as_posix()
    remote_final = f"{clone}/{rel}"
    names = final_files(host.list(remote_final))
    masters = [n for n in names if n.endswith("_master.mp4")]
    if len(masters) != 1 or not any(n.endswith(".timeline.tsv") for n in names):
        raise RuntimeError(f"{remote_final}: no single master with timelines (found {names})")
    want = host.md5s(remote_final, names)
    if sorted(want) != names:
        raise RuntimeError(f"{remote_final}: no md5 for {sorted(set(names) - set(want))}")
    log = [f"{slug}: {source} at {host.sha(clone)}, {remote_final}"]
    # anything of the upload's in the local final that is not exactly the host's: the whole cut goes aside
    local = final_files([p.name for p in finals.iterdir() if p.is_file()]) if finals.is_dir() else []
    if any(n not in want or md5(finals / n) != want[n] for n in local):
        log.append(f"  preserved the old final as {preserve(finals).name}")
    finals.mkdir(parents=True, exist_ok=True)
    for n in names:
        take(host, f"{remote_final}/{n}", want[n], finals / n, log)
    crate = root / "explainer_corpus" / slug
    thumb = thumbnail_rel(crate)
    if thumb:
        remote_dir, _, thumb_name = f"{clone}/explainer_corpus/{slug}/{thumb}".rpartition("/")
        thumb_md5 = host.md5s(remote_dir, [thumb_name]).get(thumb_name)
        if thumb_md5 is None:
            log.append(f"  NO THUMBNAIL on the host: render it there with ./x {slug} thumbnail in {clone}, then fetch again")
        else:
            (crate / thumb).parent.mkdir(parents=True, exist_ok=True)
            take(host, f"{remote_dir}/{thumb_name}", thumb_md5, crate / thumb, log)
    return log


# ------------------------------------------------------------------ check


def ffprobe(path: Path) -> dict:
    out = subprocess.run(
        ["ffprobe", "-v", "error", "-select_streams", "v:0", "-show_entries",
         "stream=width,height,r_frame_rate:format=duration", "-of", "json", str(path)],
        capture_output=True, text=True, check=True, timeout=120).stdout
    d = json.loads(out)
    s = (d.get("streams") or [{}])[0]
    num, _, den = str(s.get("r_frame_rate", "0/1")).partition("/")
    return {"width": int(s.get("width", 0)), "height": int(s.get("height", 0)),
            "fps": float(num) / float(den or 1), "duration": float(d.get("format", {}).get("duration", 0.0))}


def timelines_seconds(finals: Path) -> float:
    return sum(frames / fps for _, frames, fps in timeline_rows(finals))


def timeline_rows(finals: Path) -> list[tuple[str, float, float]]:
    rows = []
    for tl in sorted(finals.glob("*.timeline.tsv")):
        cells = dict(line.split("\t")[:2] for line in tl.read_text(encoding="utf-8").splitlines() if "\t" in line)
        rows.append((tl.name, float(cells["frames"]), float(cells["fps"])))
    return rows


def cues(text: str, kind: str) -> tuple[list[tuple[float, float, str]], list[str]]:
    """(start, end, text) of each cue of an SRT or VTT, and the problems reading it."""
    out, bad = [], []
    blocks = [b for b in re.split(r"\n\s*\n", text.replace("\r\n", "\n").strip()) if b.strip()]
    if kind == "vtt":
        if not blocks or not blocks[0].startswith("WEBVTT"):
            return [], ["the VTT does not start with WEBVTT"]
        blocks = blocks[1:]
    for n, block in enumerate(blocks, 1):
        lines = block.split("\n")
        if kind == "srt":
            if lines[0].strip() != str(n):
                bad.append(f"SRT cue {n}: index {lines[0].strip()!r}")
            lines = lines[1:]
        m = STAMP.match(lines[0].strip()) if lines else None
        if not m:
            bad.append(f"{kind.upper()} cue {n}: no 'HH:MM:SS,mmm --> HH:MM:SS,mmm' line")
            continue
        g = [int(x) for x in m.groups()]
        start = g[0] * 3600 + g[1] * 60 + g[2] + g[3] / 1000
        end = g[4] * 3600 + g[5] * 60 + g[6] + g[7] / 1000
        out.append((start, end, " ".join(s.strip() for s in lines[1:] if s.strip())))
    return out, bad


def png_size(path: Path) -> tuple[int, int] | None:
    head = path.read_bytes()[:24]
    if head[:8] != b"\x89PNG\r\n\x1a\n" or head[12:16] != b"IHDR":
        return None
    return int.from_bytes(head[16:20], "big"), int.from_bytes(head[20:24], "big")


def manifest_mismatch(root: Path, master: Path) -> list[str]:
    """docs/publish/MANIFEST.md's rows that name `master`'s path with an md5 that is not the file's, one line each
    (a row with no md5, or no row at all, is not a mismatch: rows are written when a cut is recorded)."""
    manifest = root / "docs" / "publish" / "MANIFEST.md"
    if not manifest.is_file() or not master.is_relative_to(root):
        return []
    path, digest, out = master.relative_to(root).as_posix(), md5(master), []
    for n, line in enumerate(manifest.read_text(encoding="utf-8").splitlines(), 1):
        if f"`{path}`" in line:
            written = re.findall(r"\b[0-9a-f]{32}\b", line)
            if written and digest not in written:
                out.append(f"docs/publish/MANIFEST.md line {n}: names {path} with md5 {written[0]}, the file is {digest}")
    return out


def check(slug: str, finals: Path, voice: str = "michael", root: Path = ROOT, probe=ffprobe,
          skills: Path | None = None) -> tuple[list[str], list[str]]:
    """(problems, the upload sheet's lines): no problems means the cut and its upload text are ready."""
    crate = root / "explainer_corpus" / slug
    bad: list[str] = []
    rel = lambda p: p.relative_to(root).as_posix() if p.is_relative_to(root) else str(p)  # noqa: E731
    masters = sorted(finals.glob("*_master.mp4"))
    srts, vtts = sorted(finals.glob("*.srt")), sorted(finals.glob("*.vtt"))
    rows = timeline_rows(finals) if finals.is_dir() else []
    for what, found in (("master (*_master.mp4)", masters), ("SRT", srts), ("VTT", vtts)):
        if len(found) != 1:
            bad.append(f"{rel(finals)}: {len(found)} {what}, want one")
    if not rows:
        bad.append(f"{rel(finals)}: no *.timeline.tsv (fetch the whole final: ./x {slug} release-fetch)")
    thumb_name = thumbnail_rel(crate)
    thumb = crate / thumb_name if thumb_name else None
    if thumb is None:
        bad.append("publish/youtube.md: no '- Thumbnail: `...`' line")
    elif not thumb.is_file():
        bad.append(f"{rel(thumb)}: missing (render it on the build host: ./x {slug} thumbnail; then release-fetch)")
    else:
        size = png_size(thumb)
        if size != THUMB_SIZE:
            bad.append(f"{rel(thumb)}: {size or 'not a PNG'}, YouTube wants a {THUMB_SIZE[0]}x{THUMB_SIZE[1]} PNG")
        if thumb.stat().st_size > THUMB_MAX_BYTES:
            bad.append(f"{rel(thumb)}: {thumb.stat().st_size} bytes (YouTube takes 2 MB)")
    master = masters[0] if len(masters) == 1 else None
    duration = None
    p = None
    if master is not None:
        try:
            p = probe(master)
        except (subprocess.SubprocessError, OSError, ValueError) as e:
            bad.append(f"{rel(master)}: ffprobe cannot read it ({e})")
    if p is not None:
        duration = p["duration"]
        if (p["width"], p["height"]) != (3840, 2160) or abs(p["fps"] - 60) > 1e-6:
            bad.append(f"{rel(master)}: {p['width']}x{p['height']} at {p['fps']:g} fps, the delivery is 3840x2160 at 60")
        if rows and abs(duration - timelines_seconds(finals)) > DURATION_TOLERANCE_S:
            bad.append(f"{rel(master)}: {duration:.3f} s, its timelines add to {timelines_seconds(finals):.3f} s "
                       "(the master is not this final's)")
    bad.extend(youtube_package.check(slug, voice, finals if rows else None, root))
    srt_text = ""
    if len(srts) == 1:
        srt_cues, problems = cues(srts[0].read_text(encoding="utf-8"), "srt")
        bad.extend(problems)
        srt_text = " ".join(t for _, _, t in srt_cues)
        for n, (a, b, t) in enumerate(srt_cues, 1):
            if b <= a:
                bad.append(f"SRT cue {n}: ends {b:.3f} s, not after its start {a:.3f} s")
            if n > 1 and a < srt_cues[n - 2][1] - 1e-6:
                bad.append(f"SRT cue {n}: starts {a:.3f} s, before cue {n - 1} ends ({srt_cues[n - 2][1]:.3f} s)")
            if not t:
                bad.append(f"SRT cue {n}: no text")
        if duration is not None and srt_cues and srt_cues[-1][1] > duration + 0.05:
            bad.append(f"SRT: the last cue ends {srt_cues[-1][1]:.3f} s, after the master ({duration:.3f} s)")
        if len(vtts) == 1:
            vtt_cues, problems = cues(vtts[0].read_text(encoding="utf-8"), "vtt")
            bad.extend(problems)
            if [t for _, _, t in vtt_cues] != [t for _, _, t in srt_cues]:
                bad.append("VTT: its cue texts are not the SRT's")
        bad.extend(youtube_package.protected_missing(
            crate / "docs" / "protected.tsv", {"captions": srt_text}, {"captions": srts[0].name}))
        check_py = (skills / "math-explainer-videos-end-to-end-with-franken-manim-frankentts/scripts/protected_check.py"
                    if skills else None)
        if check_py and check_py.is_file() and (crate / "docs" / "protected.tsv").is_file():
            # the script's and the captions' rows (the upload text's rows are youtube_package's, above)
            r = subprocess.run([sys.executable, "-I", str(check_py), str(crate / "docs" / "protected.tsv"),
                                str(crate / "script.tsv"), str(srts[0])], capture_output=True, text=True)
            if r.returncode != 0:
                bad.extend(f"protected_check: {line.strip()}" for line in r.stdout.splitlines()
                           if line.strip().startswith("MISSING"))
    if master is not None:
        bad.extend(manifest_mismatch(root, master))
    sheet = []
    if not bad and master is not None:
        pub = crate / "publish" / voice
        yt = (crate / "publish" / "youtube.md").read_text(encoding="utf-8")
        disclosure = re.search(r"^- Altered or synthetic content: (.*)$", yt, re.M)
        sheet = [
            f"Title: {(pub / 'title.txt').read_text(encoding='utf-8').strip()}",
            f"Description: {rel(pub / 'description.txt')} (paste whole)",
            f"Tags: {rel(pub / 'tags.txt')}",
            f"Thumbnail: {rel(thumb)}",
            f"Captions: English, {rel(srts[0])}",
            "Category: Education. Audience: not made for kids. Visibility: Private (the owner's review first).",
            f"Altered or synthetic content: {disclosure.group(1) if disclosure else 'Yes (synthetic narration)'}",
            f"Master: {rel(master)} md5 {md5(master)}, {duration:.1f} s, SRT md5 {md5(srts[0])}",
        ]
    return bad, sheet


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("action", choices=["fetch", "check"])
    ap.add_argument("slug")
    ap.add_argument("--finals", type=Path, help="the video's renders/final (./x passes its voice's)")
    ap.add_argument("--voice", default="michael")
    ap.add_argument("--from", dest="source", help="fetch: HOST:CLONE, a finals clone on the build host")
    ap.add_argument("--skills", type=Path, help="check: the skills directory (protected_check.py)")
    args = ap.parse_args(argv)
    finals = args.finals or ROOT / "explainer_corpus" / args.slug / "renders" / "final"
    if args.action == "fetch":
        if not args.source:
            ap.error("fetch needs --from HOST:CLONE")
        try:
            for line in fetch(args.slug, finals, args.source):
                print(line)
        except (RuntimeError, ValueError, subprocess.CalledProcessError) as e:
            print(f"release-fetch {args.slug}: {e}", file=sys.stderr)
            return 1
        print(f"release-fetch {args.slug}: done; now ./x {args.slug} release-check")
        return 0
    bad, sheet = check(args.slug, finals, args.voice, skills=args.skills)
    for b in bad:
        print(f"FAIL {args.slug}: {b}")
    if bad:
        return 1
    print(f"PASS {args.slug}: the cut and its upload text are ready for the upload lane")
    for line in sheet:
        print(f"  {line}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
