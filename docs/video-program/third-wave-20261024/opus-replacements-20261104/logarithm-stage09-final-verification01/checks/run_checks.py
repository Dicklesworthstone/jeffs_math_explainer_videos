"""Checks for verify_stage09_final.py (bead 87do.63, BlueSalmon Mail 4124). From the repository root:

    python3 -I -B .../logarithm-stage09-final-verification01/checks/run_checks.py FRESH_SCRATCH --w W

LavenderPike. Read-only on the repository and on W (root's geometry stage, for the stage09 crate); writes only under
FRESH_SCRATCH. The parsers run on real kit outputs already in root's release lane: the Nov 2 logarithm machine render
(7 chapters, 64 captions, its real master probed with ffprobe and one frame decoded), the logarithm and apportionment
rebuild previews' 79 non-empty layout files, and a real ebur128 log. The planted faults are byte edits of copies in
scratch. The command-binding fixture uses RubyRaven's sealed render helper's own JOB_TEMPLATE and step_config, with
stand-in media files that are labelled as such. Nothing here renders, assembles, launches a guard job, listens or
views a picture.
"""
import argparse
import contextlib
import copy
import glob
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import re
import shutil
import subprocess
import sys
import types
import wave

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-final-verification01"
HELPER = REPO / HERE / "verify_stage09_final.py"
RENDER_HELPER = REPO / P / "logarithm-stage09-render-preparation01/prepare_stage09_render.py"
RENDER_HELPER_SHA = "9ddc2003aa666844214aace166343bb37c1d5baeb6b62abb252f517c634b5388"
LANE = REPO / "narration_lab/third-wave-20261024/root/five-film-release-20261027"
OLD_WORK = LANE / "logarithm-machine-render-work01"
OLD_TIMELINE = "logarithm-machine-timeline01"
OLD_NARRATION = LANE / "logarithm-branch-hold-assemble11/narration"
OLD_HOLDS = REPO / "explainer_corpus/logarithm_mysteries/docs/narration-direction-map-oliver11.json"
OLD_VERIFY = LANE / "logarithm-machine-verify01/verification.json"
OLD_LOUDNESS = LANE / "apportionment-final-encoded-loudness01/command.log"


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("scratch")
    parser.add_argument("--w", required=True)
    a = parser.parse_args()
    if os.path.exists(a.scratch):
        print("FRESH_SCRATCH must not exist", file=sys.stderr)
        return 2
    scratch = Path(a.scratch)
    scratch.mkdir(parents=True)
    crate = Path(a.w) / "logarithm-stage09"
    V = load("verify_stage09_final", HELPER)
    results = []

    def clean(text):
        """Shorten local paths before any truncation, so no scratch or home path reaches the transcript."""
        for path, label in ((str(scratch.resolve()), "SCRATCH"), (str(scratch), "SCRATCH"), (str(Path(a.w)), "W"), (str(REPO), "REPO")):
            text = text.replace(path, label)
        return text

    def check(name, ok, detail=""):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {clean(detail)}" if detail else ""), flush=True)

    def refusal(fn, *args):
        try:
            fn(*args)
        except V.Refused as error:
            return clean(str(error))
        return None

    print(f"helper sha256 {sha(HELPER)}; checks sha256 {sha(__file__)}")

    # ------------------------------------------------------------------ F1 the stage09 crate and root's pinned tools
    script = V.read_script(crate / "script.tsv")
    sys.path.insert(0, str(REPO / "tools"))
    try:
        assembler = load("coherent_narration_for_check", REPO / "tools/coherent_narration.py")
    finally:
        sys.path.remove(str(REPO / "tools"))
    native = assembler.read_script(crate / "script.tsv")
    holds = json.loads((crate / V.DIRECTION_MAP).read_text())["visual_holds"]
    order = list(script)
    successors = {cue: (order[i + 1] if i + 1 < len(order) else None) for i, cue in enumerate(order)}
    pins = {rel: sha(REPO / rel) == digest for rel, digest in V.PINS.items()}
    check("F1-stage09-crate", V.registry(crate) == V.CHAPTERS and script == native and len(script) == V.CUES
          and len(holds) == 15 and all(successors[h["cue"]] == h["next_cue"] for h in holds) and all(pins.values()),
          f"W/logarithm-stage09 registers the 13 chapters in order ({V.CHAPTERS[0]} .. {V.CHAPTERS[-1]}); its script reads as "
          f"{len(script)} cues, identical to the canonical assembler's read_script; oliver16's {len(holds)} visual holds each "
          f"name the script's next cue; root's strict verify, caption screen, guard and guard source match their pins "
          f"({sum(pins.values())}/{len(pins)})")

    # ------------------------------------------------------------------ F2 real kit outputs: timelines, schedule, captions
    summary = json.loads((LANE / OLD_TIMELINE / "timeline-summary.json").read_text())
    old_chapters = [x["chapter"] for x in summary["chapters"]]
    report = json.loads((OLD_NARRATION / "report.json").read_text())
    old_script = {cue: " ".join(s["sentence"] for s in rows) for cue, rows in report["lines"].items()}
    saved = V.CHAPTERS
    V.CHAPTERS = old_chapters
    try:
        ctx = types.SimpleNamespace(work=OLD_WORK, lane=LANE, timeline_job=OLD_TIMELINE, narration=OLD_NARRATION)
        bound = {(LANE / OLD_TIMELINE / "timeline-output" / f"{ch}.timeline.tsv").resolve() for ch in old_chapters}
        chapters, placements, timelines, total = V.read_schedule(ctx, old_script, summary, bound)
    finally:
        V.CHAPTERS = saved
    srt_text = (OLD_WORK / "logarithm_mysteries.en.srt").read_text()
    vtt_text = (OLD_WORK / "logarithm_mysteries.en.vtt").read_text()
    facts = V.check_captions(srt_text, vtt_text, old_script, report, placements, total)
    check("F2-real-schedule-and-captions", total == 17783 and len(placements) == 14 and facts["caption_blocks"] == 64,
          f"the Nov 2 logarithm machine render (real kit output): {len(chapters)} chapter timelines parse and equal the timeline "
          f"job's, {len(placements)} cues play the bound narration WAVs, {total} frames; {facts['caption_blocks']} captions, SRT = VTT, "
          f"text equal to the narration report's sentences as a whole and cue by cue (the script stand-in is that report's "
          f"sentences, which the assembler takes from the script); last caption ends {facts['last_caption_end_ms']} ms of "
          f"{facts['video_ms']} ms")

    # ------------------------------------------------------------------ F3 planted bad captions
    blocks = srt_text[:-2].split("\n\n")

    def srt_of(rows):
        return "".join(f"{k}\n{body}\n\n" for k, body in enumerate(rows, 1))

    bodies = ["\n".join(b.split("\n")[1:]) for b in blocks]
    vtt_bodies = vtt_text[len("WEBVTT\n\n"):-2].split("\n\n")
    assert [b.split("\n", 1)[1] for b in bodies] == [b.split("\n", 1)[1] for b in vtt_bodies]
    k22 = next(i for i, b in enumerate(bodies) if "For a nonzero complex number" in b)
    word = bodies[3].split("\n")[1].split()[-1]

    def swap(rows, i, old, new):
        rows = list(rows)
        assert old in rows[i]
        rows[i] = rows[i].replace(old, new, 1)
        return rows

    moved = swap(bodies, k22, bodies[k22].split(" --> ")[0], "00:02:09,100")
    moved_vtt = swap(vtt_bodies, k22, vtt_bodies[k22].split(" --> ")[0], "00:02:09.100")
    s_end = bodies[k22 - 1].split(" --> ")[1].split("\n")[0]
    overlap = swap(bodies, k22, bodies[k22].split(" --> ")[0], "00:02:08,000")
    overlap_vtt = swap(vtt_bodies, k22, vtt_bodies[k22].split(" --> ")[0], "00:02:08.000")
    last_end = bodies[-1].split(" --> ")[1].split("\n")[0]
    past = swap(bodies, len(bodies) - 1, last_end, "00:04:57,000")
    past_vtt = swap(vtt_bodies, len(bodies) - 1, last_end.replace(",", "."), "00:04:57.000")
    three = swap(bodies, 5, bodies[5], bodies[5] + "\nextra")
    three_vtt = swap(vtt_bodies, 5, vtt_bodies[5], vtt_bodies[5] + "\nextra")
    plants = {
        "VTT word changed only": (srt_text, "WEBVTT\n\n" + "\n\n".join(swap(vtt_bodies, 3, word, word + "zz")) + "\n\n",
                                  "SRT and VTT differ at caption 4"),
        "same word changed in both": (srt_of(swap(bodies, 3, word, word + "zz")),
                                      "WEBVTT\n\n" + "\n\n".join(swap(vtt_bodies, 3, word, word + "zz")) + "\n\n",
                                      "the captions' text differs from the script"),
        "02_2's first caption moved before 02_2 starts": (srt_of(moved), "WEBVTT\n\n" + "\n\n".join(moved_vtt) + "\n\n",
                                                          "02_1: the captions shown during this cue read"),
        "caption overlaps the one before": (srt_of(overlap), "WEBVTT\n\n" + "\n\n".join(overlap_vtt) + "\n\n",
                                            f"caption {k22 + 1} starts at 128000 ms, before caption {k22} ends"),
        "last caption past the video": (srt_of(past), "WEBVTT\n\n" + "\n\n".join(past_vtt) + "\n\n", "outside the video's"),
        "a caption of three lines": (srt_of(three), "WEBVTT\n\n" + "\n\n".join(three_vtt) + "\n\n", "caption 6: needs one or two text lines"),
        "SRT numbering skips": (srt_text.replace("\n\n5\n", "\n\n6\n", 1), vtt_text, "SRT caption 5: numbered '6'"),
        "VTT without its header": (srt_text, vtt_text[len("WEBVTT\n\n"):], "VTT: no WEBVTT header"),
        "SRT comma stamps in the VTT": (srt_text, "WEBVTT\n\n" + "\n\n".join(bodies) + "\n\n", "VTT caption 1: malformed time line"),
    }
    got = {name: refusal(V.check_captions, s, v, old_script, report, placements, total) for name, (s, v, _w) in plants.items()}
    check("F3-planted-captions", all(got[name] and plants[name][2] in got[name] for name in plants),
          "; ".join(f"{name} -> {got[name]!r}"[:150] for name in plants) + f" (the moved caption still starts after {s_end})")

    # ------------------------------------------------------------------ F4 the real master: frames and A/V, planted counts
    master = OLD_WORK / "logarithm_mysteries_master.mp4"
    probe = json.loads(subprocess.run(["ffprobe", "-v", "error", "-show_streams", "-show_format", "-of", "json", str(master)],
                                      check=True, capture_output=True, text=True).stdout)
    packets = int(subprocess.run(["ffprobe", "-v", "error", "-select_streams", "V:0", "-count_packets", "-show_entries",
                                  "stream=nb_read_packets", "-of", "csv=p=0", str(master)], check=True, capture_output=True, text=True).stdout)
    media = V.media_facts(probe, packets, total)
    late = copy.deepcopy(probe)
    next(s for s in late["streams"] if s["codec_type"] == "audio")["duration"] = f"{total / 60 + 0.05:.6f}"
    small = copy.deepcopy(probe)
    next(s for s in small["streams"] if s["codec_type"] == "video")["width"] = 1920
    extra = copy.deepcopy(probe)
    extra["streams"].append({"codec_type": "subtitle", "codec_name": "mov_text"})
    bad = {"timelines one frame longer": refusal(V.media_facts, probe, packets, total + 1),
           "a dropped frame": refusal(V.media_facts, probe, packets - 1, total),
           "audio 50 ms long": refusal(V.media_facts, late, packets, total),
           "1920 wide": refusal(V.media_facts, small, packets, total),
           "a third stream": refusal(V.media_facts, extra, packets, total)}
    check("F4-real-master-frames-and-av", media["video_frames"] == 17783 and all(bad.values())
          and "17783 video frames; the chapters' timelines sum to 17784" in bad["timelines one frame longer"]
          and "17782 video frames" in bad["a dropped frame"] and "the audio lasts" in bad["audio 50 ms long"],
          f"the real master's {media['video_frames']} video packets equal the 7 timelines' {total} frames; video "
          f"{media['video_seconds']} s, audio {media['audio_seconds']} s (within one AAC frame); planted: "
          + "; ".join(f"{k} -> {v!r}"[:110] for k, v in bad.items()))

    # ------------------------------------------------------------------ F5 every real non-empty layout file in the lane
    layouts = sorted(p for p in glob.glob(str(LANE / "*/*.layout.tsv")) + glob.glob(str(LANE / "*/*/*.layout.tsv")) if os.path.getsize(p))
    rows = 0
    for path in layouts:
        rows += len(V.layout_anchors(Path(path).read_text(), Path(path).name))
    sample = (LANE / "logarithm-opus-rebuild-preview01/render/11_unreachable.layout.tsv").read_text().split("\n")[0]
    anchor = V.layout_anchors(sample, "x")[0][0]
    broken = refusal(V.layout_anchors, sample.replace(" @29.8s", ""), "11_unreachable.layout.tsv")
    detail_only = refusal(V.layout_anchors, "hold\toverlap\t\"a\" and \"b\" @3.0s overlap", "x")
    check("F5-real-layout-rows", len(layouts) >= 79 and rows > 0 and anchor == 1788 and broken and detail_only,
          f"{len(layouts)} real non-empty layout files ({rows} finding rows) in the release lane all parse; "
          f"'{sample[:40]}' anchors at frame {anchor}; a checkpoint without its time is refused ({broken!r}), and a time "
          f"inside the detail does not count ({detail_only!r})")

    # ------------------------------------------------------------------ F6 the picture plan on the real render
    holds11 = {h["cue"]: h for h in json.loads(OLD_HOLDS.read_text())["visual_holds"]}
    holds11 = {cue: h for cue, h in holds11.items() if cue in old_script}
    cue = "02_2"
    wav = OLD_NARRATION / f"{cue}.wav"
    text = old_script[cue]
    at = text.encode().index(b"every logarithm")
    phrases = {"cues": {cue: {"wav_sha256": sha(wav), "text_sha256": hashlib.sha256(text.encode()).hexdigest(),
                              "marks": [{"byte": at, "seconds": 3.5, "basis": "TEST FIXTURE: the shape cmd_timings writes"}]}}}
    marks = V.check_phrases(phrases, old_script, OLD_NARRATION)
    wrong_wav = copy.deepcopy(phrases)
    wrong_wav["cues"][cue]["wav_sha256"] = "0" * 64
    backwards = copy.deepcopy(phrases)
    backwards["cues"][cue]["marks"].append({"byte": at - 1, "seconds": 4.0, "basis": "x"})
    onsets = V.sentence_onsets(report)
    samples =V.plan_samples(chapters, placements, timelines, old_script, onsets, holds11, marks,
                             {ch: [] for ch in old_chapters})
    start22 = next(p["start_frame"] for p in placements if p["cue"] == cue)
    phrase_frames = [s["frame"] for s in samples if any("phrase byte" in r for r in s["reasons"])]
    frames = [s["frame"] for s in samples]
    precedent = {p["frame"] for p in json.loads(OLD_VERIFY.read_text())["pictures"]}
    covered = sum(1 for p in placements for s in samples if s["frame"] == p["start_frame"] and f"{p['cue']} cue start" in s["reasons"])
    sentence_rows = sum(len(v) for v in onsets.values())
    hold_cues = sorted(c for c in holds11 if any(f"{c} " in r and "hold" in r for s in samples for r in s["reasons"]))
    bad_wav, bad_order = refusal(V.check_phrases, wrong_wav, old_script, OLD_NARRATION), refusal(V.check_phrases, backwards, old_script, OLD_NARRATION)
    bad_next = refusal(V.plan_samples, chapters, placements, timelines, old_script, onsets,
                       {c: dict(h, next_cue="06_1") for c, h in holds11.items()}, {}, {ch: [] for ch in old_chapters})
    check("F6-real-picture-plan", frames == sorted(set(frames)) and frames[-1] < total and covered == len(placements)
          and phrase_frames == [start22 + 210, start22 + 240, start22 + 270, start22 + 330] and hold_cues == sorted(holds11)
          and bad_wav and bad_order and bad_next,
          f"{len(samples)} distinct frames on the real render (root's Nov 2 verifier drew {len(precedent)}; "
          f"{len(precedent & set(frames))} of them recur); every cue start, {sentence_rows} sentence onsets by the kit's "
          f"report_starts rule, the {len(hold_cues)} oliver11 holds that fall in this cut ({', '.join(hold_cues)}), and one phrase mark in "
          f"cmd_timings' shape at 3.5 s -> frames {phrase_frames}; refused: another WAV ({bad_wav!r}), marks out of byte order "
          f"({bad_order!r}), a hold map naming another next cue ({bad_next!r})")

    # ------------------------------------------------------------------ F7 command binding, on RubyRaven's own master script
    assert sha(RENDER_HELPER) == RENDER_HELPER_SHA
    with contextlib.redirect_stdout(io.StringIO()):
        R = load("prepare_stage09_render_for_check", RENDER_HELPER)
    lane, jobs, work = scratch / "lane", scratch / "ws/jobs", scratch / "lane" / f"{V.STAGE}-render-work01"
    narration, binary = scratch / "narration", scratch / "ws/stand-in-binary"
    for folder in (lane, jobs, work, narration):
        folder.mkdir(parents=True)
    binary.write_bytes(b"STAND-IN binary: never run\n")
    # TEST FIXTURE narration: one second of silence per cue, never a recording; report and phrase timings in the shapes
    # coherent_narration.py and prepare_audio.py cmd_timings write.
    for cue in script:
        with wave.open(str(narration / f"{cue}.wav"), "wb") as w:
            w.setnchannels(1)
            w.setsampwidth(2)
            w.setframerate(24000)
            w.writeframes(b"\0\0" * 24000)
    (narration / "report.json").write_text(json.dumps({"meta": {"rate": 24000, "claim": "TEST FIXTURE"}, "lines": {
        cue: [{"sentence": text, "out_samples": 24000, "gap_after_samples": 0}] for cue, text in script.items()}}))
    fixture_marks = {"08_5": [{"byte": 267, "seconds": 0.5, "basis": "TEST FIXTURE"}]}
    (narration / "phrase-timings.json").write_text(json.dumps({"voice": "TEST FIXTURE", "cues": {cue: {
        "wav_sha256": sha(narration / f"{cue}.wav"), "text_sha256": hashlib.sha256(text.encode()).hexdigest(),
        "marks": fixture_marks.get(cue, [])} for cue, text in script.items()}}))

    def guard_receipt(command, inputs):
        return {"admitted": True, "phase": "native", "command": command, "exit": 0, "finished": "fixture",
                "inputs_sha256": sha(inputs), "final_inputs_sha256": sha(inputs)}

    def fixture_job(name, files, log, rows):
        (lane / name).mkdir()
        for rel, body in files.items():
            (lane / name / rel).parent.mkdir(parents=True, exist_ok=True)
            (lane / name / rel).write_text(body)
        job_script = jobs / f"{name}.py"
        if not job_script.exists():
            job_script.write_text(f"# STAND-IN job script {name}\n")
        inputs = jobs / f"{name}-inputs.json"
        inputs.write_text(json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in [job_script] + rows]}, indent=2) + "\n")
        (lane / name / "command.log").write_text(f"RUN fixture\n{log}\n")
        (lane / name / "resource-receipt.json").write_text(json.dumps(guard_receipt(["python3", "-I", "-B", str(job_script)], inputs)) + "\n")

    tl = f"{V.STAGE}-timeline01"
    summary_rows, srt_rows, offset = [], [], 0

    def stamp(ms, mark):
        return f"{ms // 3600000:02d}:{ms // 60000 % 60:02d}:{ms // 1000 % 60:02d}{mark}{ms % 1000:03d}"

    for k, ch in enumerate(V.CHAPTERS):
        chapter_cues = [cue for cue in script if cue.startswith(f"{k:02d}_")]
        frames = 60 + 120 * len(chapter_cues)
        body = f"# explainer timeline v1\nchapter\t{ch}\nfps\t60\nframes\t{frames}\n"
        for i, cue in enumerate(chapter_cues):
            body += f"line\t{cue}\t{60 + 120 * i}\t0\t{narration / (cue + '.wav')}\n"
            at = round((offset + 60 + 120 * i) * 1000 / 60)
            srt_rows.append((at, at + 1000, script[cue]))
        (work / f"{ch}.timeline.tsv").write_text(body)
        (work / f"{ch}.layout.tsv").write_text("")
        (work / f"{ch}.mp4").write_bytes(f"STAND-IN chapter {ch}\n".encode())
        summary_rows.append({"chapter": ch, "frames": frames, "cues": chapter_cues, "timeline_sha256": sha(work / f"{ch}.timeline.tsv")})
        offset += frames
    fixture_job(tl, {"timeline-summary.json": json.dumps({"fps": 60, "chapters": summary_rows, "cues": 80, "total_frames": offset}),
                     **{f"timeline-output/{ch}.timeline.tsv": (work / f"{ch}.timeline.tsv").read_text() for ch in V.CHAPTERS}},
                V.TIMELINE_SUCCESS, [])
    for ch in V.CHAPTERS:
        fixture_job(f"{V.STAGE}-render-{ch}-01", {"render-summary.json": json.dumps({"chapter": ch, "mp4_sha256": sha(work / f"{ch}.mp4")})},
                    V.RENDER_SUCCESS, [])
    c_ruby = types.SimpleNamespace(lane=lane, target=scratch / "target", jobs=jobs, crate=crate)
    b_ruby = {"audio": {"narration": str(narration)}, "build": {"binary": str(binary), "binary_sha256": sha(binary)}}
    mjob = R.job_name("master", "01")
    files = R.job_files(c_ruby, mjob)
    cfg = R.step_config(c_ruby, b_ruby, "master", "01", mjob, files, None)
    text = R.JOB_TEMPLATE.replace("@@JOB@@", mjob).replace("@@STEP@@", "master").replace("@@HELPER@@", RENDER_HELPER_SHA)
    (jobs / f"{mjob}.py").write_text(text.replace("@@CONFIG@@", repr(json.dumps(cfg, indent=1))))
    (work / "logarithm_mysteries_master.mp4").write_bytes(b"STAND-IN master: not a video\n")
    (work / "logarithm_mysteries.en.srt").write_text("".join(f"{k}\n{stamp(s, ',')} --> {stamp(e, ',')}\n{t}\n\n" for k, (s, e, t) in enumerate(srt_rows, 1)))
    (work / "logarithm_mysteries.en.vtt").write_text("WEBVTT\n\n" + "".join(f"{stamp(s, '.')} --> {stamp(e, '.')}\n{t}\n\n" for s, e, t in srt_rows))
    bound_rows = [binary] + [work / f"{ch}.{ext}" for ch in V.CHAPTERS for ext in ("mp4", "timeline.tsv", "layout.tsv")]
    bound_rows += [lane / tl / "timeline-summary.json"] + [lane / tl / "timeline-output" / f"{ch}.timeline.tsv" for ch in V.CHAPTERS]
    bound_rows += [narration / "report.json", narration / "phrase-timings.json"] + [narration / f"{cue}.wav" for cue in script]
    bound_rows += [crate / rel for rel in ("script.tsv", V.DIRECTION_MAP, "src/chapters/mod.rs")]
    fixture_job(mjob,{"master-summary.json": json.dumps({k: {"path": str(work / n), "sha256": sha(work / n)} for k, n in
                                                         (("master", "logarithm_mysteries_master.mp4"), ("srt", "logarithm_mysteries.en.srt"),
                                                          ("vtt", "logarithm_mysteries.en.vtt"))})}, V.MASTER_SUCCESS, bound_rows)
    mine = types.SimpleNamespace(lane=lane, jobs=jobs, crate=crate, narration=narration, series="01",
                                 master_job=mjob, timeline_job=tl, score_job=f"{V.STAGE}-score01", work=work)
    cues = list(script)
    positive = V.bind_master(mine, cues)

    @contextlib.contextmanager
    def planted(*edits):
        saved_bytes = [(p, p.read_bytes()) for p, _ in edits]
        try:
            for p, fn in edits:
                p.write_bytes(fn(p.read_bytes()))
            yield
        finally:
            for p, data in saved_bytes:
                p.write_bytes(data)

    script_path, inputs_path, receipt_path = jobs / f"{mjob}.py", jobs / f"{mjob}-inputs.json", lane / mjob / "resource-receipt.json"

    def forged_config(change):
        """A consistent forgery: the script's CONFIG changed, its inputs row and the receipt re-hashed to match."""
        def script_edit(data):
            t = data.decode()
            line = re.search(r"^CONFIG = json\.loads\((.*)\)$", t, re.M)
            c2 = json.loads(__import__("ast").literal_eval(line.group(1)))
            change(c2)
            return t.replace(line.group(0), "CONFIG = json.loads(" + repr(json.dumps(c2, indent=1)) + ")").encode()

        def inputs_edit(data):
            d = json.loads(data)
            d["files"][0]["sha256"] = hashlib.sha256(script_edit(script_path.read_bytes())).hexdigest()
            return (json.dumps(d, indent=2) + "\n").encode()

        def receipt_edit(data):
            d = json.loads(data)
            d["inputs_sha256"] = d["final_inputs_sha256"] = hashlib.sha256(inputs_edit(inputs_path.read_bytes())).hexdigest()
            return json.dumps(d).encode()
        return [(receipt_path, receipt_edit), (inputs_path, inputs_edit), (script_path, script_edit)]

    def receipt_with(**fields):
        return [(receipt_path, lambda d: json.dumps({**json.loads(d), **fields}).encode())]

    cases = {
        "the receipt ran another script": (receipt_with(command=["python3", "-I", "-B", str(jobs / "other.py")]), "the guard's receipt ran"),
        "the master exited 1": (receipt_with(exit=1), "did not pass: exit 1"),
        "inputs file edited after the run": ([(inputs_path, lambda d: d + b" ")], "the guard's input hash is not"),
        "a chapter mp4 changed after the master": ([(work / "05_complex_turns.mp4", lambda d: d + b"x")], "bound inputs moved since it ran"),
        "master command writes another folder": (forged_config(lambda c2: c2["argv"].__setitem__(-1, str(scratch / "elsewhere"))),
                                                 "the master command is"),
        "captions read another narration": (forged_config(lambda c2: c2["captions_argv"].__setitem__(4, str(scratch / "other"))),
                                            "the captions command is"),
        "the script names another narration": (forged_config(lambda c2: c2.__setitem__("narration", str(scratch / "other"))),
                                               "the master used narration"),
        "the script is a render step": (forged_config(lambda c2: c2.__setitem__("step", "render")), "another step, job or lane"),
        "the log lacks the success line": ([(lane / mjob / "command.log", lambda d: d + b"Traceback\n")], "does not end with the master's success line"),
        "the master changed after its job": ([(work / "logarithm_mysteries_master.mp4", lambda d: d + b"x")], "the master changed since the master job"),
        "the timeline job failed": ([(lane / tl / "resource-receipt.json", lambda d: json.dumps({**json.loads(d), "exit": 2}).encode())],
                                    f"{tl} did not pass"),
    }
    outcome = {}
    for name, (edits, _want) in cases.items():
        with planted(*edits):
            outcome[name] = refusal(V.bind_master, mine, cues)
    unchanged = refusal(V.bind_master, mine, cues)
    check("F7-command-binding", positive["cfg"]["argv"][1] == "master" and unchanged is None
          and all(outcome[n] and cases[n][1] in outcome[n] for n in cases),
          f"the master script is RubyRaven's sealed JOB_TEMPLATE with her step_config (argv {positive['cfg']['argv'][1:2]} ... "
          f"--out render-work01; captions all), stand-in media; bind_master passes it; planted: "
          + "; ".join(f"{n} -> {outcome[n]!r}"[:120] for n in cases) + "; after every plant is restored it passes again")

    # ------------------------------------------------------------------ F10 the command line, composed, on that fixture
    common = ["--lane", str(lane), "--jobs", str(jobs), "--crate", str(crate), "--narration", str(narration), "--series", "01"]

    def cli(*args):
        r = subprocess.run([sys.executable, "-I", "-B", str(HELPER), *args, *common], cwd=REPO, capture_output=True, text=True)
        return r.returncode, clean(r.stdout.strip())

    vdir = scratch / "ws/verify-jobs"
    code_c, out_c = cli("check")
    facts_c = json.loads(out_c) if code_c == 0 else {}
    code_w, out_w = cli("write-job", "--verify", "01", "--job-dir", str(vdir))
    written = json.loads(out_w) if code_w == 0 else {}
    vscript, vinputs = vdir / f"{V.STAGE}-final-verify01.py", vdir / f"{V.STAGE}-final-verify01-inputs.json"
    rows = json.loads(vinputs.read_text())["files"] if vinputs.is_file() else []
    rows_ok = all(Path(r["path"]).is_file() and sha(r["path"]) == r["sha256"] for r in rows) and rows and rows[0]["path"] == str(vscript)
    planned = len(V.read_config(vscript)["samples"]) if vscript.is_file() else None
    again = cli("write-job", "--verify", "01", "--job-dir", str(vdir))
    early = cli("verify-job", "--verify", "01", "--job-dir", str(vdir))
    srt_path = work / "logarithm_mysteries.en.srt"
    with planted((srt_path, lambda d: d.replace(b"\n2\n", b"\n3\n", 1))):
        stale = cli("check")
    check("F10-composed-command-line", code_c == 0 and facts_c.get("cues") == 80 and facts_c.get("chapters") == 13
          and facts_c.get("holds") == 15 and facts_c.get("phrase_marks") == 1 and code_w == 0 and rows_ok
          and len(rows) == written.get("input_rows") and planned == facts_c.get("pictures_planned")
          and written.get("launch_from_repo_root", "").startswith(f"python3 -I -B {V.GUARD_REL} --phase native --job {V.STAGE}-final-verify01 --inputs ")
          and again[0] == 1 and "exists: use a new --verify number" in again[1] and early[0] == 1 and "resource-receipt.json is missing" in early[1]
          and stale[0] == 1 and "the srt changed since the master job" in stale[1],
          f"on the fixture above (80 one-second silent stand-in WAVs, report, captions and phrase timings in their writers' shapes, "
          f"the real oliver16 holds): `check` passes with {facts_c.get('chapters')} chapters, {facts_c.get('cues')} cues, "
          f"{facts_c.get('holds')} holds, {facts_c.get('phrase_marks')} phrase mark and {facts_c.get('pictures_planned')} planned "
          f"pictures; `write-job` writes the job and {len(rows)} input rows, all current and the job script first, and prints the "
          f"guard line; writing it again is refused ({again[1]!r}); `verify-job` before any launch is refused ({early[1]!r}); "
          f"an SRT edited after the master is refused by `check` ({stale[1]!r})")

    # ------------------------------------------------------------------ F8 the job runs only under its own guard receipt
    jlane, jdir = scratch / "job-lane", scratch / "job-dir"
    jlane.mkdir()
    jdir.mkdir()
    jcfg = {"job": f"{V.STAGE}-final-verify01", "lane": str(jlane), "script": str(jdir / f"{V.STAGE}-final-verify01.py"),
            "helper": str(HELPER), "helper_sha256": sha(HELPER),
            "master": {"path": str(work / "logarithm_mysteries_master.mp4"), "sha256": "0" * 64}}
    jscript = Path(jcfg["script"])

    def run_script(cfg_now):
        body = V.JOB_TEMPLATE.replace("@@JOB@@", cfg_now["job"]).replace("@@HELPER@@", sha(HELPER))
        jscript.write_text(body.replace("@@CONFIG@@", repr(json.dumps(cfg_now, indent=1))))
        r = subprocess.run([sys.executable, "-I", "-B", str(jscript)], cwd=REPO, capture_output=True, text=True)
        return r.returncode, clean(r.stdout.strip())

    no_receipt = run_script(jcfg)
    (jlane / jcfg["job"]).mkdir()
    (jlane / jcfg["job"] / "resource-receipt.json").write_text(json.dumps({"admitted": True, "command": ["python3", "-I", "-B", str(jdir / "x.py")]}))
    other = run_script(jcfg)
    (jlane / jcfg["job"] / "resource-receipt.json").write_text(json.dumps({"admitted": True, "command": ["python3", "-I", "-B", str(jscript)]}))
    moved_master = run_script(jcfg)
    moved_helper = run_script(dict(jcfg, helper_sha256="f" * 64))
    check("F8-job-guard-binding", no_receipt[0] == other[0] == moved_master[0] == moved_helper[0] == 1
          and "launch only through the guard" in no_receipt[1] and "does not show this script running" in other[1]
          and moved_master[1] == "REFUSED: the master moved since the job was written"
          and moved_helper[1] == "REFUSED: the verification helper moved"
          and not any((jlane / jcfg["job"] / n).exists() for n in V.OUTPUTS),
          f"the written job script, run directly: {no_receipt[1][:60]!r}...; with a receipt for another script: {other[1][:70]!r}; "
          f"under its own receipt but with the master's bytes changed: {moved_master[1]!r}; with the helper's hash changed: "
          f"{moved_helper[1]!r}; no output was written")

    # ------------------------------------------------------------------ F9 measurement parsers, sheets, one real frame
    real_loud = V.loudness_summary(OLD_LOUDNESS.read_text(errors="replace"))
    cut_loud = refusal(V.loudness_summary, OLD_LOUDNESS.read_text(errors="replace").split("Summary:")[0])
    out = scratch / "pictures"
    out.mkdir()
    frame = out / "frame-0001.png"
    subprocess.run(["ffmpeg", "-nostdin", "-n", "-v", "error", "-xerror", "-ss", f"{600 / 60:.9f}", "-i", str(master), "-frames:v", "1",
                    "-vf", "scale=960:540", str(frame)], check=True, timeout=180)
    from PIL import Image
    with Image.open(frame) as img:
        size = img.size
    pictures = [{"frame": 600 + k, "seconds": (600 + k) / 60, "reasons": ["F9 sheet test", "second reason"], "path": str(frame)} for k in range(5)]
    sheets = V.make_sheets(pictures, out)
    with Image.open(sheets[0]) as s1, Image.open(sheets[1]) as s2:
        sheet_sizes = (s1.size, s2.size)
    screen_out = scratch / "caption-screen.json"
    screen = subprocess.run([sys.executable, "-I", "-B", str(REPO / V.SCREEN_REL), str(OLD_WORK / "logarithm_mysteries.en.srt"), str(total), "60",
                             str(screen_out)], capture_output=True, text=True)
    check("F9-measurement-parsers", real_loud["integrated_lufs"] and cut_loud and size == (960, 540)
          and sheet_sizes == ((1920, 1160), (1920, 580)) and screen.returncode == 0
          and json.loads(screen_out.read_text())["caption_count"] == 64,
          f"a real ebur128 log reads I {real_loud['integrated_lufs']} LUFS, LRA {real_loud['lra_lu']} LU, true peak "
          f"{real_loud['true_peak_dbtp']} dBTP, and a log cut before its Summary is refused ({cut_loud!r}); the job's frame command "
          f"decodes the real master's frame 600 to {size[0]}x{size[1]}; five pictures make sheets of {sheet_sizes}; root's caption "
          f"screen reads the real SRT: {screen.stdout.strip()}")

    print(f"FINAL-VERIFICATION CHECKS: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
