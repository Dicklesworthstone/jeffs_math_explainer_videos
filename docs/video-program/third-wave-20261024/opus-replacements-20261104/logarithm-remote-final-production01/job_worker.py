#!/usr/bin/env python3
"""One logarithm render or master job on root's remote lane (bead 87do.63). LavenderPike, 2026-11-05.

render_master_remote.py copies this file byte for byte to each job's script and writes that job's facts beside it in
<job>.config.json. Launch only through O/guard01.py from the repository root; the guard binds every input by hash.
Built on the worker prepare_stage09_render.py wrote for stage09. Every check of that worker stays, as do root's full
timeline, cue and lint checks. This version also reads what the files really are and what the kit really reported.
Technical checks only: no listening, picture review, fine sync or acceptance is claimed here.
"""
import hashlib
import json
import os
import pathlib
import re
import shutil
import socket
import subprocess
import sys
import wave

CONFIG = json.loads(pathlib.Path(__file__).with_suffix(".config.json").read_text())
LANE = pathlib.Path(CONFIG["lane"])
JOB = LANE / CONFIG["job"]
TOOLS = CONFIG["tools"]
# x264 writes its settings into the stream. They sit at byte 73 of a stage09 chapter and 2.8 MB into its master.
HEAD_BYTES = 16 << 20
PROBE = ("stream=codec_type,codec_name,profile,width,height,r_frame_rate,nb_frames,pix_fmt,color_space,sample_rate,"
         "channels,bit_rate:format=duration")
REPLAY = re.compile(r"^(\S+)\s+replay = fmn's soundtrack, bit for bit \((\d+) cues, (\d+) samples\)$")
MASTERED = re.compile(r"^master -> (.+): gain (-?[\d.]+) dB, (-?[\d.]+) LUFS, (-?[\d.]+) dBTP; (\d+) frames "
                      r"\(chapters (\d+)\); audio ([\d.]+) s \(picture ([\d.]+) s\)$")
CAPTIONED = re.compile(r"^captions: (\d+) sentences as (\d+) captions -> (.+)$")


def refuse(message):
    print("REFUSED: " + message, flush=True)
    sys.exit(1)


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def same(a, b):
    return pathlib.Path(a).resolve() == pathlib.Path(b).resolve()


def finished_ok(job):
    path = LANE / job / "resource-receipt.json"
    if not path.is_file():
        return False
    r = json.loads(path.read_text())
    return (r.get("admitted") is True and r.get("exit") == 0 and bool(r.get("finished"))
            and r.get("inputs_sha256") == r.get("final_inputs_sha256")
            and not r.get("stopped_for_resource_limit") and "error" not in r)


def rows(path):
    return [line.split("\t") for line in pathlib.Path(path).read_text().splitlines() if line and not line.startswith("#")]


def first(table, key):
    return next(r[1] for r in table if r[0] == key)


def findings(path):
    return len([line for line in pathlib.Path(path).read_text().splitlines() if line.strip()]) if pathlib.Path(path).is_file() else None


def capture(work, chapter):
    """The kit master's choice: the last of soundtrack_0.wav, _1, ... before the first gap."""
    folder = pathlib.Path(work) / "soundtrack" / chapter
    n = 0
    while (folder / ("soundtrack_%d.wav" % n)).exists():
        n += 1
    return folder / ("soundtrack_%d.wav" % (n - 1)) if n else None


def frame_to_sample(frames, fps):
    """explainer_score::units::frame_to_sample at 48 kHz: frames * 48000 / fps, rounded half away from zero."""
    return (2 * frames * 48000 + fps) // (2 * fps)


def incomplete(track, frames, fps):
    """Why the capture is not the whole soundtrack the master's null test needs (48 kHz, 16-bit, stereo, as long as
    the chapter in its header and in its data), or None when it is. The kit only warns when its copy fails, so a
    partial capture would otherwise surface at the master, after every render."""
    want = frame_to_sample(frames, fps)
    try:
        with wave.open(str(track)) as w:
            found = (w.getframerate(), w.getnchannels(), 8 * w.getsampwidth(), w.getnframes())
            present = len(w.readframes(want)) // (w.getnchannels() * w.getsampwidth())
    except Exception as error:  # an unreadable file is refused, never trusted
        return "not a readable WAV (%s: %s)" % (type(error).__name__, error)
    if found != (48000, 2, 16, want) or present != want:
        return ("%d Hz, %d channels, %d bits, %d frames in the header and %d present; the chapter needs 48000 Hz, "
                "2 channels, 16 bits and %d frames" % (found + (present, want)))
    return None


def tool_env():
    """The pinned tools and no others. The kit finds ffmpeg, ffprobe and python3 by name, so PATH decides what runs."""
    names = ("python3", "ffmpeg", "ffprobe")
    folders = [str(pathlib.Path(TOOLS[name]["path"]).parent) for name in names]
    path = os.pathsep.join(dict.fromkeys(folders + ["/usr/bin", "/bin"]))
    for name in names:
        found = shutil.which(name, path=path)
        if found is None or not same(found, TOOLS[name]["path"]):
            refuse("PATH %s finds %s at %s, not the pinned %s" % (path, name, found, TOOLS[name]["path"]))
        if sha(TOOLS[name]["path"]) != TOOLS[name]["sha256"]:
            refuse("this host's %s is not the pinned file" % name)
    return dict(os.environ, PATH=path)


def run(argv, timeout, env):
    print("RUN " + " ".join(argv), flush=True)
    subprocess.run(argv, check=True, timeout=timeout, env=env)


def movie_facts(path, frames, env):
    """What the file is, read from the file: never what the command asked for."""
    command = [TOOLS["ffprobe"]["path"], "-v", "error", "-show_entries", PROBE, "-of", "json", str(path)]
    probe = json.loads(subprocess.check_output(command, text=True, timeout=300, env=env))
    video = [s for s in probe.get("streams", []) if s.get("codec_type") == "video"]
    audio = [s for s in probe.get("streams", []) if s.get("codec_type") == "audio"]
    if len(video) != 1 or len(audio) != 1:
        refuse("%s has %d video and %d audio streams" % (path.name, len(video), len(audio)))
    v, a = video[0], audio[0]
    picture = {key: v.get(key) for key in ("codec_name", "profile", "width", "height", "r_frame_rate", "pix_fmt", "color_space")}
    sound = {key: a.get(key) for key in ("codec_name", "sample_rate", "channels")}
    want = {"codec_name": "h264", "profile": "High", "width": 3840, "height": 2160, "r_frame_rate": "60/1",
            "pix_fmt": "yuv420p", "color_space": "bt709"}
    if picture != want:
        refuse("%s is %s, not the 4K60 picture the film needs" % (path.name, picture))
    if str(v.get("nb_frames")) != str(frames):
        refuse("%s holds %s frames, the timeline says %d" % (path.name, v.get("nb_frames"), frames))
    if sound != {"codec_name": "aac", "sample_rate": "48000", "channels": 2}:
        refuse("%s has audio %s, not 48 kHz stereo AAC" % (path.name, sound))
    with open(path, "rb") as handle:
        head = handle.read(HEAD_BYTES)
    start = head.find(b"x264 - core")
    settings = head[start:start + 2048] if start >= 0 else b""
    if b" rc=crf" not in settings or b" crf=16.0" not in settings:
        refuse("%s does not carry x264's own record of CRF 16" % path.name)
    return dict(picture, frames=frames, audio=sound, x264=settings[:settings.find(b" Copy")].decode("ascii", "replace"),
                rate_control="crf 16.0", duration=probe.get("format", {}).get("duration"))


def contracts(lines):
    """The kit's own plan line for the master and for the captions, one of each, for this whole film."""
    found = {}
    for line in lines:
        if line.startswith('{"') and '"consumer"' in line:
            row = json.loads(line)
            if row.get("consumer") in found:
                refuse("the log has two %s plan lines" % row.get("consumer"))
            found[row.get("consumer")] = row
    want = {"topic": CONFIG["topic"], "chapter_scope": None, "promotion": False, "listening_acceptance": False,
            "selected_cues": len(CONFIG["cues"]), "verified_cues": len(CONFIG["cues"])}
    for name in ("master", "captions"):
        if name not in found:
            refuse("the log has no %s plan line" % name)
        wrong = {key: found[name].get(key) for key, value in want.items() if found[name].get(key) != value}
        if wrong:
            refuse("the %s plan line differs: %s" % (name, wrong))
    return found


def holds_of(contract):
    """Every prediction hold of the bound map, in order. The kit returns None for a hold it can't place, silently."""
    holds = contract.get("prediction_holds")
    found = [[h.get("cue"), h.get("next_cue")] for h in holds or []]
    if found != CONFIG["prediction_cues"]:
        refuse("the master planned %d prediction holds, %s; the bound map has %d" % (len(found), found, len(CONFIG["prediction_cues"])))
    ends = [h.get("end_ms") for h in holds]
    if not all(isinstance(end, int) for end in ends) or ends != sorted(set(ends)):
        refuse("the prediction holds do not end at rising times: %s" % ends)
    return holds


def replays_of(lines):
    """One bit-for-bit replay line per chapter, in order, as long as the chapter."""
    found = [(m[1], int(m[2]), int(m[3])) for m in map(REPLAY.match, lines) if m]
    want = [(chapter, CONFIG["frames"][chapter] * 800) for chapter in CONFIG["chapters"]]
    if [(name, samples) for name, _, samples in found] != want:
        refuse("the master's replay lines are %s, not the %d chapters' lengths" % (found, len(want)))
    return {name: {"cue_rows": cues, "samples": samples} for name, cues, samples in found}


def mastered(lines, path):
    found = [m for m in map(MASTERED.match, lines) if m]
    if len(found) != 1:
        refuse("the log has %d master result lines" % len(found))
    m = found[0]
    total = sum(CONFIG["frames"].values())
    if m[1] != str(path) or int(m[5]) != total or int(m[6]) != total:
        refuse("the master reports %s with %s frames (chapters %s); the film has %d" % (m[1], m[5], m[6], total))
    return {"gain_db": float(m[2]), "lufs": float(m[3]), "true_peak_dbtp": float(m[4]), "frames": total,
            "audio_seconds": float(m[7]), "picture_seconds": float(m[8])}


def captioned(lines, path):
    found = [m for m in map(CAPTIONED.match, lines) if m]
    if len(found) != 1:
        refuse("the log has %d caption result lines" % len(found))
    m = found[0]
    if m[3] != str(path) or int(m[1]) != CONFIG["sentences"]:
        refuse("the captions report %s sentences in %s; the film needs %d" % (m[1], m[3], CONFIG["sentences"]))
    return {"sentences": int(m[1]), "captions": int(m[2])}


def stamps(text, mark):
    rule = re.compile(r"^(\d\d):(\d\d):(\d\d)%s(\d{3}) --> (\d\d):(\d\d):(\d\d)%s(\d{3})" % (mark, mark), re.M)
    pairs = []
    for m in rule.finditer(text):
        h1, m1, s1, ms1, h2, m2, s2, ms2 = map(int, m.groups())
        pairs.append(((h1 * 60 + m1) * 60000 + s1 * 1000 + ms1, (h2 * 60 + m2) * 60000 + s2 * 1000 + ms2))
    return pairs


def caption_files(srt, vtt, count, holds):
    """The caption files themselves: the promised count, the same times twice, in order, inside the film, and every
    prediction hold's end present as a caption's end (true of all 13 on stage09)."""
    pairs = stamps(srt, ",")
    if len(pairs) != count or stamps(vtt, r"\.") != pairs:
        refuse("the SRT has %d cues and the VTT %d; the kit reported %d" % (len(pairs), len(stamps(vtt, r"\.")), count))
    last = 0
    for start, end in pairs:
        if start < last or end <= start:
            refuse("the captions overlap or run backwards at %d ms" % start)
        last = end
    if last > sum(CONFIG["frames"].values()) * 1000 / 60 + 1:
        refuse("the last caption ends at %d ms, after the film" % last)
    ends = {end for _, end in pairs}
    lost = [h["cue"] for h in holds if h["end_ms"] not in ends]
    if lost:
        refuse("the captions lost the prediction holds of %s" % lost)
    return {"last_end_ms": last}


def evidence():
    return {"binary_sha256": CONFIG["binary_sha256"], "narration": CONFIG["narration"], "tools_sha256": CONFIG["tools_sha256"],
            "binding_sha256": CONFIG["binding_sha256"],
            "worker": {"host": CONFIG["host"], "python3": TOOLS["python3"], "ffmpeg": TOOLS["ffmpeg"], "ffprobe": TOOLS["ffprobe"]}}


def summary(name, data):
    data["scope"] = CONFIG["scope"]
    with (JOB / name).open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")


def render(env):
    chapter = CONFIG["chapter"]
    work = pathlib.Path(CONFIG["render_work"])
    work.mkdir(exist_ok=True)
    if (work / (chapter + ".mp4")).exists():
        refuse(chapter + ".mp4 already exists in the render work folder")
    if capture(work, chapter) is not None:
        refuse(chapter + ": an earlier soundtrack capture is still in the render work folder")
    run(CONFIG["argv"], 43200, env)
    reference = LANE / CONFIG["timeline_job"] / "timeline-output" / (chapter + ".timeline.tsv")
    if rows(reference) != rows(work / (chapter + ".timeline.tsv")):
        refuse(chapter + ": the frame and cue schedule differs from the 60 fps timeline")
    if not (work / (chapter + ".layout.tsv")).is_file():
        refuse(chapter + ": rendered lint is missing")
    if rows(reference.with_name(chapter + ".cues.tsv")) != rows(work / (chapter + ".cues.tsv")):
        refuse(chapter + ": full cue table differs")
    track = capture(work, chapter)
    if track is None or not track.is_file():
        refuse(chapter + ": the render left no soundtrack capture for the master's null test")
    table = rows(work / (chapter + ".timeline.tsv"))
    frames = int(first(table, "frames"))
    if frames != CONFIG["frames"][chapter]:
        refuse(chapter + ": the render's frame count differs from the timeline job's")
    problem = incomplete(track, frames, int(first(table, "fps")))
    if problem:
        refuse(chapter + ": the soundtrack capture " + track.name + " is not the whole soundtrack the master's null "
               "test needs: " + problem)
    media = movie_facts(work / (chapter + ".mp4"), frames, env)
    summary("render-summary.json", dict(evidence(), chapter=chapter, mp4_sha256=sha(work / (chapter + ".mp4")),
                                        timeline_sha256=sha(work / (chapter + ".timeline.tsv")),
                                        cues_sha256=sha(work / (chapter + ".cues.tsv")),
                                        layout_sha256=sha(work / (chapter + ".layout.tsv")),
                                        layout_findings=findings(work / (chapter + ".layout.tsv")), media=media,
                                        soundtrack={"path": str(track), "sha256": sha(track),
                                                    "captures": int(track.stem.rsplit("_", 1)[1]) + 1}))


def chapters_match(work, when):
    """Each chapter's files are still the bytes its render recorded, and the kit would take the capture it recorded."""
    for chapter, recorded in CONFIG["renders"].items():
        for name, digest in recorded.items():
            if not (work / name).is_file() or sha(work / name) != digest:
                refuse("%s no longer matches its render summary %s" % (name, when))
        track, want = capture(work, chapter), CONFIG["soundtracks"][chapter]
        if track is None or str(track) != want["path"] or sha(track) != want["sha256"]:
            refuse(chapter + ": the kit would null-test another soundtrack capture than its render recorded, " + when)


def master(env):
    work = pathlib.Path(CONFIG["render_work"])
    outputs = {"master": work / (CONFIG["topic"] + "_master.mp4"), "srt": work / (CONFIG["topic"] + ".en.srt"),
               "vtt": work / (CONFIG["topic"] + ".en.vtt")}
    for path in [*outputs.values(), work / "master"]:
        if path.exists():
            refuse(str(path) + " already exists")
    chapters_match(work, "before the master")
    run(CONFIG["argv"], 14400, env)
    run(CONFIG["captions_argv"], 1200, env)
    chapters_match(work, "after the master")
    lines = (JOB / "command.log").read_text(errors="replace").splitlines()
    plans = contracts(lines)
    holds = holds_of(plans["master"])
    replays = replays_of(lines)
    report = mastered(lines, outputs["master"])
    captions = captioned(lines, outputs["srt"])
    timing = caption_files(outputs["srt"].read_text(), outputs["vtt"].read_text(), captions["captions"], holds)
    media = movie_facts(outputs["master"], sum(CONFIG["frames"].values()), env)
    data = {key: {"path": str(path), "sha256": sha(path)} for key, path in outputs.items()}
    data.update(evidence(), soundtracks=CONFIG["soundtracks"], renders=CONFIG["render_jobs"], media=media, master_report=report,
                replays=replays, prediction_holds=holds, captions=dict(captions, **timing), plan_mode=plans["master"].get("plan_mode"))
    summary("master-summary.json", data)


def main():
    if socket.gethostname().split(".")[0].lower() != CONFIG["host"]:
        refuse("this job runs only on " + CONFIG["host"])
    receipt = JOB / "resource-receipt.json"
    if not receipt.is_file():
        refuse("launch only through the guard: " + str(receipt) + " is missing")
    started = json.loads(receipt.read_text())
    command = started.get("command")
    if (not isinstance(command, list) or len(command) != 4 or command[1:] != ["-I", "-B", CONFIG["script"]]
            or "exit" in started or started.get("admitted") is not True):
        refuse("the guard's receipt for this job does not show this script running")
    if not same(command[0], TOOLS["python3"]["path"]) or not same(sys.executable, TOOLS["python3"]["path"]):
        refuse("this is not the pinned python3")
    if os.environ.get("CARGO_TARGET_DIR") != CONFIG["target"]:
        refuse("CARGO_TARGET_DIR is not the guard's target")
    if sha(CONFIG["binary"]) != CONFIG["binary_sha256"]:
        refuse("the binary moved")
    binding = json.loads(pathlib.Path(CONFIG["binding"]).read_text())
    if (binding["build"]["binary"], binding["build"]["binary_sha256"]) != (CONFIG["binary"], CONFIG["binary_sha256"]):
        refuse("the binding names another binary")
    if binding["audio"]["narration"] != CONFIG["narration"]:
        refuse("the binding names another narration")
    for path in CONFIG["comparisons"]:
        if json.loads(pathlib.Path(path).read_text()).get("hard_failures") != []:
            refuse(pathlib.Path(path).name + " does not record a clean comparison")
    for job in CONFIG["after"]:
        if not finished_ok(job):
            refuse(job + " did not finish cleanly")
    env = tool_env()
    {"render": render, "master": master}[CONFIG["step"]](env)
    print(CONFIG["success"], flush=True)


if __name__ == "__main__":
    main()
