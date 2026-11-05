"""Independent review checks for LavenderPike's logarithm-stage09-final-verification01 (bead 87do.63, root Mail 4246).

BronzeJaguar, 2026-11-05. From the repository root:

    python3 -I -B P/logarithm-stage09-final-verification-review01/checks/review_checks.py NEW_SCRATCH --w W

W is root's geometry stage (holds repo-view/ and logarithm-stage09/). Read-only on the repository, W and root's lanes;
writes only under NEW_SCRATCH: decoded 960x540 frames of the existing Nov 2 master, and a stand-in finished-job folder
whose media files are labelled as stand-ins. The sealed helper is loaded in process and never edited. Nothing here
launches a guard job, renders, assembles, decodes production audio, listens or views a picture.

  K01  seals and pins: Lavender's bundle, Ruby's two bundles, root's tools, the W kit sources; mirrored lines present
  K02  the helper's timeline parser against the kit's ChapterTimeline::parse on all 560 real timelines in the lane
  K03  cue matching against the kit's caption writer: on the real Nov 2 render, each cue's first caption starts at
       its placement within half a millisecond
  K04  root's real 80-cue production narration through the helper's own parsers (report, sentences, phrase
       timings, WAVs, holds), and a lower bound on the stage09 picture plan
  K05  the helper's report-order gate (gather), its own statement, on that real report: refused (finding F1)
  K06  frame times of the real Nov 2 master: every video packet at n/60, each chapter's packets equal to its
       timeline, and the job's -ss extraction equal to an exact frame-number pick on frames where the picture moves
  K07  what verify-job proves after the run, on a stand-in finished job: replaced sheets, pictures and SRT (F2)
  K08  root's strict check keeps one decode log, rewritten by its second decode (F3)
  K09  the staged fix (verify_stage09_final_fix01.py = sealed helper + .patch): the real report accepted, a missing or
       extra cue refused, the K07 tampers refused; Lavender's own checks pass on it (lavender_checks_on_fix.py, run
       first, writes checks/outputs/lavender-run_checks-on-fix01.txt, which K09 reads)
"""
import argparse
import hashlib
import importlib.util
import json
import math
import os
from pathlib import Path
import re
import subprocess
import sys
import types
import wave

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
LAV = REPO / P / "logarithm-stage09-final-verification01"
LAV_SUMS = "a0dc3d50c5841bfeb85e41766914b3c3308821e913e83433a77d9d9086edb2fc"
HELPER_SHA = "cca858048738e2c1d314a81f0644000bc0e1a20fdf3119755bcd4ca987933ce6"
RUBY = REPO / P / "logarithm-stage09-render-preparation01"
RUBY_SUMS = "fa654b83"  # as Ruby's recovery bundle pins it
RUBY_HELPER = "9ddc2003aa666844214aace166343bb37c1d5baeb6b62abb252f517c634b5388"
RECOVERY = REPO / P / "logarithm-stage09-ws-build-recovery01"
LANE = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
EL = REPO / "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
PROD = EL / "production-audio-stage09-01"
NARRATION = PROD / "prepared01/narration"
OLD_WORK = LANE / "logarithm-machine-render-work01"
OLD_TIMELINE = LANE / "logarithm-machine-timeline01"
OLD_NARRATION = LANE / "logarithm-branch-hold-assemble11/narration"
KIT = {  # W/repo-view sources the stage09 crate builds against (its Cargo.toml path dependencies)
    "crates/explainer_kit/src/captions.rs": "28e1aa51370b8c3b2fa1f711a388d9233909503684dc4c1d77a2d5f1043b26bd",
    "crates/explainer_kit/src/cli.rs": "3c5221febc3e3d92d37aa97e2f8d38c319121215a67a1dc68d24534d41d862c9",
    "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
    "crates/explainer_score/src/timeline.rs": "7c3bf30fc8ff813f789b0bd152117ffd48513201b947075b397c1bcec460655b",
}
# Source lines this review relies on; each must still be present in its pinned file, or the run stops (exit 3).
MIRRORED = [
    ("kit", "crates/explainer_score/src/timeline.rs", r'let _ = writeln!(s, "mark\t{}\t{}", m.label, m.frame);'),
    ("kit", "crates/explainer_score/src/timeline.rs", '"mark" => t.marks.push(Mark {'),
    ("kit", "crates/explainer_kit/src/cli.rs", 'std::fs::write(out_of("layout.tsv"), l.report())?;'),
    ("kit", "crates/explainer_kit/src/captions.rs", "let span = (cursor as f64 / rate, end as f64 / rate, after as f64 / rate);"),
    ("kit", "crates/explainer_kit/src/captions.rs", "let at = offset + line.start_frame as f64 / fps;"),
    ("kit", "crates/explainer_kit/src/captions.rs", "offset += ch.frames as f64 / fps;"),
    ("kit", "crates/explainer_kit/src/captions.rs", "let ms = (t.max(0.0) * 1000.0).round() as u64;"),
    ("kit", "crates/explainer_kit/src/captions.rs", "last.end = offset + end;"),
    ("repo", f"{P}/logarithm-stage09-render-preparation01/prepare_stage09_render.py",
     'files.update({str(n / rel): h for rel, h in tree(n).items()})'),
    ("repo", f"{P}/logarithm-stage09-render-preparation01/prepare_stage09_render.py",
     'paths += [c.crate / rel for rel in sorted(c.stage_rows())] + [c.tools / name for name in TOOLS]'),
    ("repo", f"{P}/logarithm-stage09-render-preparation01/prepare_stage09_render.py", "paths += output_rows(c, prior, series)"),
    ("repo", "narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/verify_video_preserve_logs01.sh",
     'log="$(mktemp "${TMPDIR:-/tmp}/verify_decode.XXXXXX")"'),
    ("repo", "narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/verify_video_preserve_logs01.sh",
     """ffmpeg -nostdin -v error "$@" -i "$F" -map 0:v -map '0:a?' -f null - >"$log" 2>&1"""),
    ("repo", "narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/verify_video_preserve_logs01.sh",
     'decode "hwaccel $hwaccel, second opinion" -hwaccel "$hwaccel"'),
]


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def seal(folder):
    """(SHA256SUMS hash, rows that verify, rows listed, files present but unlisted, ignoring __pycache__)."""
    rows = [line.split("  ", 1) for line in (folder / "SHA256SUMS").read_text().splitlines() if line.strip()]
    ok = sum(1 for digest, rel in rows if (folder / rel).is_file() and sha(folder / rel) == digest)
    present = {os.path.relpath(os.path.join(d, f), folder) for d, dirs, fs in os.walk(folder) for f in fs
               if "__pycache__" not in Path(d).parts}
    return sha(folder / "SHA256SUMS"), ok, len(rows), sorted(present - {rel for _, rel in rows} - {"SHA256SUMS"})


def kit_parse(text):
    """An independent mirror of explainer_score::timeline::ChapterTimeline::parse (timeline.rs 7c3bf30f, lines 118-188)."""
    t = {"chapter": "", "fps": 0, "frames": 0, "lines": [], "marks": [], "breaths": [], "listens": []}
    for n, raw in enumerate(text.splitlines(), 1):
        line = raw.rstrip("\r")
        if not line or line.startswith("#"):
            continue
        f = line.split("\t")
        num = lambda k: float(f[k])  # noqa: E731
        integer = lambda k: int(f[k]) if re.fullmatch(r"\+?[0-9]+", f[k]) else int("bad")  # noqa: E731
        if f[0] == "chapter":
            t["chapter"] = f[1]
        elif f[0] == "fps":
            t["fps"] = integer(1)
        elif f[0] == "frames":
            t["frames"] = integer(1)
        elif f[0] == "line":
            assert len(f) == 5
            t["lines"].append((f[1], integer(2), num(3), f[4]))
        elif f[0] == "mark":
            t["marks"].append((f[1] if len(f) > 1 else "", integer(2)))
        elif f[0] == "breath":
            t["breaths"].append((integer(1), num(2)))
        elif f[0] == "listen":
            assert len(f) <= 5
            t["listens"].append((integer(1), num(2)))
        else:
            raise ValueError(f"unknown record {f[0]}")
    assert t["fps"] != 0
    return t


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("scratch")
    parser.add_argument("--w", required=True)
    a = parser.parse_args()
    if os.path.exists(a.scratch):
        print("NEW_SCRATCH must not exist", file=sys.stderr)
        return 2
    scratch = Path(a.scratch)
    scratch.mkdir(parents=True)
    w = Path(a.w)
    view = w / "repo-view"
    results = []

    def clean(text):
        for path, label in ((str(scratch.resolve()), "SCRATCH"), (str(scratch), "SCRATCH"), (str(w), "W"),
                            (str(REPO), "REPO"), (str(Path.home()), "~")):
            text = text.replace(path, label)
        return text

    def check(name, ok, detail):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}: {clean(detail)}", flush=True)

    # ------------------------------------------------------------------ K01 seals, pins and mirrored lines
    missing = [(where, rel) for where, rel, line in MIRRORED
               if line not in ((view if where == "kit" else REPO) / rel).read_text()]
    if missing:
        print(f"STALE: mirrored lines missing from {missing}")
        return 3
    lav = seal(LAV)
    ruby = seal(RUBY)
    rec = seal(RECOVERY)
    V = load("verify_stage09_final_review", LAV / "verify_stage09_final.py")
    pins = {rel: sha(REPO / rel) == digest for rel, digest in V.PINS.items()}
    kit = {rel: sha(view / rel) == digest for rel, digest in KIT.items()}
    check("K01-seals-and-pins",
          lav == (LAV_SUMS, 4, 4, []) and sha(LAV / "verify_stage09_final.py") == HELPER_SHA
          and ruby[0].startswith(RUBY_SUMS) and ruby[1] == ruby[2] and not ruby[3] and sha(RUBY / "prepare_stage09_render.py") == RUBY_HELPER
          and rec[1] == rec[2] and not rec[3] and all(pins.values()) and all(kit.values()),
          f"Lavender's seal {lav[0][:8]} {lav[1]}/{lav[2]} OK, unlisted {lav[3]}, helper {HELPER_SHA[:8]}; Ruby's render "
          f"preparation seal {ruby[0][:8]} {ruby[1]}/{ruby[2]}, helper {RUBY_HELPER[:8]}; Ruby's ws-build recovery seal "
          f"{rec[0][:8]} {rec[1]}/{rec[2]}, unlisted {rec[3]}; root's 4 pinned tools {sum(pins.values())}/4; W kit sources "
          f"{sum(kit.values())}/{len(kit)}; {len(MIRRORED)} mirrored source lines present")

    # ------------------------------------------------------------------ K02 the timeline parser on every real timeline
    paths = sorted(set().union(*(LANE.glob(g) for g in ("*/*.timeline.tsv", "*/*/*.timeline.tsv", "*/*/*/*.timeline.tsv"))))
    differ, counts = [], {"mark": 0, "breath": 0, "listen": 0, "line": 0}
    for path in paths:
        text = path.read_text()
        mine, theirs = V.parse_timeline(text, path.name), kit_parse(text)
        same = (mine["chapter"], mine["fps"], mine["frames"], [(x["id"], x["start"], x["gain_db"], x["wav"]) for x in mine["lines"]],
                [(x["label"], x["frame"]) for x in mine["marks"]], [(x["frame"], x["seconds"]) for x in mine["breaths"]],
                [(x["frame"], x["seconds"]) for x in mine["listens"]]) == (
                theirs["chapter"], theirs["fps"], theirs["frames"], theirs["lines"], theirs["marks"], theirs["breaths"], theirs["listens"])
        if not same:
            differ.append(path.name)
        for key in counts:
            counts[key] += len(theirs[key + "s"]) if key != "line" else len(theirs["lines"])
    labels = sorted({x["label"] for path in paths for x in V.parse_timeline(path.read_text(), path.name)["marks"]})
    check("K02-timeline-parser-equals-kit", paths and not differ and counts["mark"] > 0,
          f"{len(paths)} real timelines in the release lane ({counts['line']} line, {counts['mark']} mark, {counts['breath']} "
          f"breath, {counts['listen']} listen records): the helper's parse_timeline equals a mirror of the kit's parse in "
          f"all of them (mark = label, then frame, as timeline.rs writes it); mark labels {labels[:6]}; differing {differ[:3]}")

    # ------------------------------------------------------------------ K03 cue matching against the kit's caption writer
    summary = json.loads((OLD_TIMELINE / "timeline-summary.json").read_text())
    report = json.loads((OLD_NARRATION / "report.json").read_text())
    offset, first_start = 0, {}
    for item in summary["chapters"]:
        t = V.parse_timeline((OLD_WORK / f"{item['chapter']}.timeline.tsv").read_text(), item["chapter"])
        for line in t["lines"]:
            first_start[line["id"]] = (offset + line["start"]) * 1000 / 60
        offset += t["frames"]
    srt = V.parse_captions((OLD_WORK / "logarithm_mysteries.en.srt").read_text(), False)
    order = list(first_start)
    owner = []  # the helper's rule (check_captions): the last cue placed at or before the caption's start + 1 ms
    for start, _end, _lines in srt:
        owner.append([cue for cue in order if first_start[cue] <= start + 1.0][-1])
    firsts = {cue: srt[owner.index(cue)][0] - first_start[cue] for cue in order if cue in owner}
    worst = max(abs(x) for x in firsts.values())
    # how far each cue's placement is from the start of the caption just before its first one
    margin = min(first_start[cue] - srt[owner.index(cue) - 1][0] for cue in order[1:] if cue in owner)
    sample_timed = all("out_samples" in s for rows in report["lines"].values() for s in rows)
    check("K03-cue-matching-equals-kit", sample_timed and len(firsts) == 14 and worst <= 0.5 and margin > 1.0,
          f"the Nov 2 report is sample-timed (out_samples on every sentence), so the kit's writer starts each cue's first "
          f"sentence at its placement (captions.rs: span from cursor 0; at = offset + start_frame / fps; stamps rounded to "
          f"the ms). On the real SRT ({len(srt)} captions), the helper's rule gives each of the {len(firsts)} cues its own "
          f"first caption, starting within {worst:.3f} ms of the placement; the previous caption starts at least "
          f"{margin / 1000:.3f} s earlier, so the 1 ms window cannot move it. Stage09's report is sample-timed too (K04)")

    # ------------------------------------------------------------------ K04 root's real 80-cue narration through the helper
    crate = w / "logarithm-stage09"
    script = V.read_script(crate / "script.tsv")
    real = json.loads((NARRATION / "report.json").read_text())
    per_cue = [cue for cue in script if V.norm(" ".join(s["sentence"] for s in real["lines"][cue])) != V.norm(script[cue])]
    onsets = V.sentence_onsets(real)
    seconds = {cue: V.wav_seconds(NARRATION / f"{cue}.wav") for cue in script}
    covered = [cue for cue, rows in real["lines"].items()
               if abs(sum(s["out_samples"] + s["gap_after_samples"] for s in rows) / real["meta"]["rate"] - seconds[cue]) > 1 / 24000]
    phrases = json.loads((NARRATION / "phrase-timings.json").read_text())
    marks = V.check_phrases(phrases, script, NARRATION)
    timings_copy = PROD / "prepared01-timings/phrase-timings.json"
    holds = {h["cue"]: h for h in json.loads((crate / V.DIRECTION_MAP).read_text())["visual_holds"]}
    lower = {}
    for cue in script:  # the cue-local frames plan_samples adds, before any chapter-level or layout sample
        length = round(seconds[cue] * 60)
        frames = {0, round(seconds[cue] * 30), max(0, length - 12)}
        for at in onsets[cue]:
            frames |= {round(at * 60), round(at * 60) + 30}
        for m in marks.get(cue, []):
            frames |= {round(m["seconds"] * 60) + d for d in (0, 30, 60, 120)}
        lower[cue] = len(frames)
    floor = sum(lower.values())
    check("K04-real-production-narration",
          sorted(real["lines"]) == sorted(script) and len(script) == 80 and not per_cue and not covered
          and sum(len(v) for v in onsets.values()) == 364 and sum(len(v) for v in marks.values()) == 33
          and timings_copy.read_bytes() == (NARRATION / "phrase-timings.json").read_bytes() and len(holds) == 15,
          f"root's real narration (EL/production-audio-stage09-01/prepared01/narration, report {sha(NARRATION / 'report.json')[:8]}, "
          f"phrase timings {sha(NARRATION / 'phrase-timings.json')[:8]}): the report's 80 cues are the W crate script's "
          f"(as a set; order in K05), and each cue's sentences read exactly its script text; sentence_onsets gives "
          f"{sum(len(v) for v in onsets.values())} onsets and each cue's out_samples + gap_after_samples equals its WAV "
          f"({sum(seconds.values()):.1f} s of narration in all); check_phrases accepts all 80 entries against the real WAVs "
          f"and script text, {sum(len(v) for v in marks.values())} marks in {len(marks)} cues; the installed file equals the "
          f"timings output; 15 holds. Picture plan: the cue-local samples alone give at least {floor} distinct frames "
          f"(>= {math.ceil(floor / 4)} sheets) before the chapter, hold and layout samples")

    # ------------------------------------------------------------------ K05 the report-order gate on the real report
    source = (LAV / "verify_stage09_final.py").read_text()
    gate = '    need(list(report["lines"]) == list(script), "report.json\'s cues differ from the script")\n'
    in_gather = source.index(gate) > source.index("def gather(c):") and source.index(gate) < source.index("def check(c):")
    try:
        exec(gate.strip(), {"need": V.need, "report": real, "script": script})  # the helper's own statement, unchanged
        outcome = "accepted"
    except V.Refused as error:
        outcome = f"Refused({str(error)!r})"
    moved = [(i, a, b) for i, (a, b) in enumerate(zip(real["lines"], script)) if a != b]
    writer = (REPO / "tools/narration_cache.py").read_text()
    sorted_writer = "json.dumps(value, sort_keys=True, ensure_ascii=False" in writer and \
        'cache.write_new(args.out / "report.json", report)' in (REPO / "tools/coherent_narration.py").read_text()
    fixed = sorted(real["lines"]) == sorted(script)
    check("K05-report-order-gate-refuses-real-report", in_gather and outcome.startswith("Refused") and sorted_writer and fixed
          and list(real["lines"]) == sorted(script),
          f"gather() runs `{gate.strip()}` before captions, holds or the plan; on root's real report it is {outcome}. The "
          f"canonical assembler writes report.json through narration_cache.encoded (sort_keys=True), so its lines are in "
          f"text order: {len(moved)} of 80 positions differ from the script, first {moved[:2]} (script: 06_20 after 06_6, "
          f"11_20 after 11_7). The kit reads the lines into a map (captions.rs read_report -> HashMap, report.get(id)); "
          f"the same cues as a set give {fixed}. F2 and F10 never met this: the Nov 2 cue ids sort in script order, and "
          f"F10's fixture report was written in script order (finding F1)")

    # ------------------------------------------------------------------ K06 frame times of the real Nov 2 master
    master = OLD_WORK / "logarithm_mysteries_master.mp4"
    stream = json.loads(subprocess.run(["ffprobe", "-v", "error", "-select_streams", "V:0", "-show_entries",
                                        "stream=time_base,start_pts,has_b_frames", "-of", "json", str(master)],
                                       check=True, capture_output=True, text=True).stdout)["streams"][0]
    num, den = (int(x) for x in stream["time_base"].split("/"))
    pts = sorted(int(x) for x in subprocess.run(["ffprobe", "-v", "error", "-select_streams", "V:0", "-show_entries", "packet=pts",
                                                 "-of", "csv=p=0", str(master)], check=True, capture_output=True,
                                                text=True).stdout.split())
    exact = all(p * num * 60 == n * den for n, p in enumerate(pts))
    per_chapter = {}
    for item in summary["chapters"]:
        packets = subprocess.run(["ffprobe", "-v", "error", "-select_streams", "V:0", "-count_packets", "-show_entries",
                                  "stream=nb_read_packets", "-of", "csv=p=0", str(OLD_WORK / f"{item['chapter']}.mp4")],
                                 check=True, capture_output=True, text=True).stdout.strip()
        per_chapter[item["chapter"]] = int(packets) == item["frames"]
    # frames in motion (f-1, f, f+1 three different small pictures, video stream only), in the first 30 s
    sums = [line.rsplit(",", 1)[1].strip() for line in subprocess.run(
        ["ffmpeg", "-nostdin", "-v", "error", "-i", str(master), "-map", "0:v:0", "-frames:v", "1800", "-vf", "scale=160:90",
         "-f", "framemd5", "-"], check=True, capture_output=True, text=True, timeout=900).stdout.splitlines()
        if line and not line.startswith("#")]
    moving = [f for f in range(1, len(sums) - 1) if len({sums[f - 1], sums[f], sums[f + 1]}) == 3]
    wanted = sorted({min(moving, key=lambda f: abs(f - target)) for target in (240, 600, 960, 1320, 1680)})
    exact_list = sorted({g for f in wanted for g in (f - 1, f, f + 1)})
    expr = "+".join(f"eq(n\\,{g})" for g in exact_list)
    subprocess.run(["ffmpeg", "-nostdin", "-n", "-v", "error", "-xerror", "-i", str(master), "-vf", f"select='{expr}',scale=960:540",
                    "-fps_mode", "passthrough", "-frames:v", str(len(exact_list)), str(scratch / "exact-%02d.png")], check=True, timeout=900)
    picked = {g: sha(scratch / f"exact-{k:02d}.png") for k, g in enumerate(exact_list, 1)}
    agree = []
    for f in wanted:
        out = scratch / f"seek-{f:04d}.png"
        subprocess.run(["ffmpeg", "-nostdin", "-n", "-v", "error", "-xerror", "-ss", f"{f / 60:.9f}", "-i", str(master), "-frames:v", "1",
                        "-vf", "scale=960:540", str(out)], check=True, timeout=180)
        agree.append(sha(out) == picked[f] != picked[f - 1] and picked[f] != picked[f + 1])
    sizes = sorted(p.stat().st_size for p in scratch.glob("seek-*.png"))
    check("K06-real-master-frame-times", exact and len(pts) == 17783 and all(per_chapter.values()) and len(sums) == 1800
          and len(wanted) == 5 and all(agree),
          f"the real Nov 2 master's {len(pts)} video packets (time base {num}/{den}, B-frame delay {stream.get('has_b_frames')}, "
          f"start pts {stream.get('start_pts')}) sit exactly at n/60 s for n = 0..{len(pts) - 1}; each of its "
          f"{len(per_chapter)} chapter mp4s has as many packets as its timeline's frames ({sum(per_chapter.values())}/"
          f"{len(per_chapter)}); at {len(wanted)} frames in motion ({wanted}, of {len(moving)} in the first 1800 whose "
          f"neighbours and own picture all differ at 160x90), the job's own "
          f"`-ss frame/60 -frames:v 1` picture equals the exact pick (select=eq(n,f)) of that frame and differs from frames "
          f"f-1 and f+1 ({sum(agree)}/{len(wanted)}): the labels name the frames shown; 960x540 PNGs "
          f"{sizes[0] // 1000}-{sizes[-1] // 1000} kB")

    # ------------------------------------------------------------------ K07 what verify-job proves after the run
    def finished_job(root, module, hashed):
        """A stand-in finished job holding every file verify-job reads, in the shape run_job writes them; with `hashed`,
        also the fields the staged fix's run_job adds. Returns (verdict(), the files a tamper edits)."""
        lane, jobs, media = root / "lane", root / "jobs", root / "media"
        job = f"{module.STAGE}-final-verify01"
        folder = lane / job
        for d in (lane, jobs, folder, folder / "decode-logs", media):
            d.mkdir(parents=True, exist_ok=True)
        (media / "master.mp4").write_bytes(b"STAND-IN master: not a video\n")
        (media / "captions.srt").write_text("1\n00:00:00,000 --> 00:00:01,000\nSTAND-IN caption\n\n")
        (media / "captions.vtt").write_text("WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nSTAND-IN caption\n\n")
        samples = [{"frame": f, "seconds": f / 60, "reasons": ["stand-in"]} for f in (0, 60, 120, 180, 240)]
        cfg = {"job": job, "lane": str(lane), "script": str(jobs / f"{job}.py"), "total_frames": 300, "samples": samples,
               **{key: {"path": str(media / name), "sha256": sha(media / name)}
                  for key, name in (("master", "master.mp4"), ("srt", "captions.srt"), ("vtt", "captions.vtt"))}}
        (jobs / f"{job}.py").write_text("# STAND-IN job script\nCONFIG = json.loads(" + repr(json.dumps(cfg)) + ")\n")
        pictures, sheets = [], []
        for i, s in enumerate(samples, 1):
            (folder / f"frame-{i:04d}.png").write_bytes(f"STAND-IN picture {i}\n".encode())
            pictures.append({**s, "path": str(folder / f"frame-{i:04d}.png")})
        for k in (1, 2):
            (folder / f"sheet-{k:03d}.jpg").write_bytes(f"STAND-IN sheet {k}\n".encode())
            sheets.append(str(folder / f"sheet-{k:03d}.jpg"))
        record = {"sha256": cfg["master"]["sha256"], "frames": 300, "media": {}, "strict_video_notes": [], "encoded_loudness": {},
                  "caption_screen": {}, "pictures": pictures, "sheets": sheets, "pictures_viewed": False}
        if hashed:
            for p in pictures:
                p["sha256"] = sha(p["path"])
            record["sheet_sha256"] = [sha(s) for s in sheets]
            record["strict_video_decodes"] = ["full decode (video + audio, software, stops at the first error): 0 error lines, "
                                              "ffmpeg exit 0, 7 s"]
        (folder / "verification.json").write_text(json.dumps(record))
        (folder / "strict-video-verification.log").write_text("STAND-IN\nVERIFY PASS\n")
        (folder / "decode-logs" / "verify_decode.STANDIN").write_text("")
        (folder / "caption-screen.json").write_text(json.dumps({"srt_sha256": cfg["srt"]["sha256"]}))
        (folder / "command.log").write_text("STAND-IN\n" + module.VERIFY_SUCCESS + "\n")
        inputs = jobs / f"{job}-inputs.json"
        inputs.write_text(json.dumps({"files": [{"path": cfg["script"], "sha256": sha(cfg["script"])}]}))
        (folder / "resource-receipt.json").write_text(json.dumps({"admitted": True, "phase": "native", "exit": 0, "finished": "stand-in",
                                                                 "command": ["python3", "-I", "-B", cfg["script"]],
                                                                 "inputs_sha256": sha(inputs), "final_inputs_sha256": sha(inputs)}))
        ctx = types.SimpleNamespace(verify_job=job, job_dir=jobs, lane=lane)

        def verdict():
            try:
                module.verify_job(ctx)
                return "passes"
            except module.Refused as error:
                return f"refused: {clean(str(error))}"
        edits = {"a sheet replaced by other bytes": folder / "sheet-001.jpg", "a picture replaced": folder / "frame-0003.png",
                 "the SRT changed after the job": media / "captions.srt"}
        return verdict, edits

    def tamper(verdict, edits):
        out = {}
        for name, path in edits.items():
            original = path.read_bytes()
            path.write_bytes(original + b"REPLACED\n")
            out[name] = verdict()
            path.write_bytes(original)
        return out

    verdict, edits = finished_job(scratch / "sealed-job", V, False)
    before = verdict()
    tampered = tamper(verdict, edits)
    check("K07-verify-job-after-the-run", before == "passes" and all(v == "passes" for v in tampered.values()),
          f"stand-in finished job with every file verify-job reads: it {before}; then, one at a time and restored: "
          + "; ".join(f"{n} -> verify-job {v}" for n, v in tampered.items())
          + ". verification.json records each picture's frame and path but no hash, and verify-job re-hashes only the "
          "master: it proves the planned files exist, not that they are the job's bytes (finding F2)")

    # ------------------------------------------------------------------ K08 root's strict check keeps one decode log
    strict = (REPO / V.STRICT_REL).read_text()
    one_log = strict.count('log="$(mktemp') == 1 and strict.count('>"$log" 2>&1') == 1 and strict.count("decode \"") == 2
    check("K08-strict-check-decode-log", one_log,
          "verify_video_preserve_logs01.sh makes one mktemp log and both decodes write it with '>' (the software decode, "
          "then the hardware second opinion when it runs, VERIFY_HWACCEL=auto gives videotoolbox on macOS): the kept "
          "decode-logs/verify_decode.* holds the last decode only. The software result survives only as the strict log's "
          "'full decode (...)' line and its first five error lines (finding F3)")

    # ------------------------------------------------------------------ K09 the staged fix
    fixed_path = REPO / P / "logarithm-stage09-final-verification-review01/verify_stage09_final_fix01.py"
    patch_path = fixed_path.with_suffix(".patch")
    X = load("verify_stage09_final_fix01", fixed_path)
    import difflib
    diff = "".join(difflib.unified_diff((LAV / "verify_stage09_final.py").read_text().splitlines(keepends=True),
                                        fixed_path.read_text().splitlines(keepends=True),
                                        "a/logarithm-stage09-final-verification01/verify_stage09_final.py",
                                        "b/logarithm-stage09-final-verification-review01/verify_stage09_final_fix01.py"))
    plus = sum(1 for x in diff.splitlines() if x.startswith("+") and not x.startswith("+++"))
    minus = sum(1 for x in diff.splitlines() if x.startswith("-") and not x.startswith("---"))
    lines = fixed_path.read_text().splitlines()
    starts = [int(h.split("+")[1].split(",")[0]) for h in re.findall(r"^@@ [^@]+ @@", diff, re.M)]
    owners = [next(x for x in reversed(lines[:s + 3]) if x.startswith("def ")).split("(")[0][4:] for s in starts]
    new_gate = next(x for x in lines if 'need(sorted(report["lines"]) == sorted(script)' in x).strip()
    gate_on = {}
    for name, rep in (("the real report", real), ("a report without 11_20", {"lines": {k: v for k, v in real["lines"].items() if k != "11_20"}}),
                      ("a report with an extra cue", {"lines": {**real["lines"], "13_1": []}})):
        try:
            exec(new_gate, {"need": X.need, "report": rep, "script": script})
            gate_on[name] = "accepted"
        except X.Refused:
            gate_on[name] = "refused"
    verdict2, edits2 = finished_job(scratch / "fixed-job", X, True)
    before2 = verdict2()
    tampered2 = tamper(verdict2, edits2)
    lavender_on_fix = (REPO / P / "logarithm-stage09-final-verification-review01/checks/outputs/lavender-run_checks-on-fix01.txt")
    lav_text = lavender_on_fix.read_text() if lavender_on_fix.is_file() else ""
    check("K09-staged-fix", diff == patch_path.read_text() and (plus, minus, len(starts)) == (20, 7, 4)
          and owners == ["gather", "run_job", "run_job", "verify_job"] and X.REPO == REPO
          and gate_on == {"the real report": "accepted", "a report without 11_20": "refused", "a report with an extra cue": "refused"}
          and before2 == "passes" and all(v.startswith("refused") for v in tampered2.values())
          and f"helper sha256 {sha(fixed_path)};" in lav_text and "FINAL-VERIFICATION CHECKS: PASS (11/11)" in lav_text,
          f"verify_stage09_final_fix01.py {sha(fixed_path)[:8]} = the sealed helper plus verify_stage09_final_fix01.patch "
          f"(+{plus}/-{minus}, {len(starts)} hunks in {owners}); its REPO resolves to this repository. The fixed gate on the "
          f"real report: " + ", ".join(f"{k} {v}" for k, v in gate_on.items()) + f". The same stand-in job in the fixed "
          f"shape: verify-job {before2}; " + "; ".join(f"{n} -> {v}" for n, v in tampered2.items())
          + ". Lavender's own run_checks.py, unchanged, passes 11/11 on the fixed helper "
          "(checks/outputs/lavender-run_checks-on-fix01.txt; its F11 run wrote the decode line and the hashes)")

    print(f"REVIEW CHECKS: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
