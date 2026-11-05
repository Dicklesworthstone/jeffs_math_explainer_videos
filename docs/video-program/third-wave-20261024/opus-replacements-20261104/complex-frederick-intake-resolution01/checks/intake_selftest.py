"""Exercise frederick_intake.py on synthetic fixtures. From the repository root:

    python3 -I -B .../complex-frederick-audio-preparation01/checks/intake_selftest.py FRESH_SCRATCH

Reads the pinned stage05 crate, root's guard wrapper (only its check_inputs function, which hashes files),
tools/coherent_narration.py and one retained Oliver WAV, all read only. Writes only under FRESH_SCRATCH.
Fixture WAVs are seeded noise written here (test fixtures, not narration); the fw logs and guard receipts are
synthetic too. Nothing here runs fw, the guard's launch path, synthesis, a build or a render.
"""
from array import array
import ast
import hashlib
import importlib.util
import json
from pathlib import Path
import random
import shutil
import struct
import subprocess
import sys

REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
TOOL = HERE.parent / "frederick_intake.py"
FIXED = "2026-11-05T00:00:00+00:00"
OLD_SCRIPT = "2968b0fd76540cbcd4f50290925e8acfec1aa995bea76a04934dac12f959929e"  # canonical, not stage05


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


fi = load("frederick_intake", TOOL)
SCRIPT = fi.read_script(REPO / fi.CRATE / "script.tsv")
EC = REPO / fi.EC
OLIVER_WAV = EC / "ui-takes-stage04-01/01_3-download01.wav"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def dump(path, data):
    path.write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")


def edit(path, change):
    data = json.loads(path.read_text(encoding="utf-8"))
    change(data)
    dump(path, data)


def noise(seed, seconds=0.25, trailer=b"C2PA", fmt=fi.FMT, pcm=None):
    """A synthetic RIFF/WAVE: seeded noise, root's chunk order (fmt, data, trailing chunk)."""
    if pcm is None:
        rng = random.Random(seed)
        samples = array("h", (rng.randint(-900, 900) for _ in range(int(48000 * seconds))))
        if sys.byteorder != "little":
            samples.byteswap()
        pcm = samples.tobytes()
    body = b"WAVE" + b"fmt " + struct.pack("<I", 16) + struct.pack("<HHIIHH", *fmt)
    body += b"data" + struct.pack("<I", len(pcm)) + pcm
    if trailer:
        body += trailer + struct.pack("<I", 8) + b"FIXTURE!"
    return b"RIFF" + struct.pack("<I", len(body)) + body


def pcm_of(data):
    return fi.probe_wav(data)


class Fixture:
    def __init__(self, root):
        self.root = root
        self.takes = root / "takes"
        self.lane = root / "lane"
        self.work = root / "work"
        self.takes.mkdir(parents=True)
        self.work.mkdir()

    def take_entry(self, path, data, source):
        info = pcm_of(data)
        return {"path": str(path), "source": source, "sha256": sha(data), "pcm_sha256": info["pcm_sha256"],
                "frames": info["frames"], "seconds": round(info["seconds"], 2)}

    def add(self, cue, seeds, layout="name", skip=(), wavs=None):
        """Request, submission, downloads receipt and two WAVs in root's layout, all synthetic."""
        text = SCRIPT[cue]
        request = {"at": FIXED, "cue": cue, "text": text, "text_sha256": sha(text.encode()),
                   "script_sha256": fi.PINS["script"], "voice": fi.VOICE, **fi.SETTINGS, "submitted": False,
                   "limits": "synthetic test fixture"}
        dump(self.takes / f"{cue}-request01.json", request)
        if layout == "name":
            submission = {"at": FIXED, "cue": cue, "request": f"{cue}-request01.json", "submitted": True,
                          "generation_requests": 1, "ui": "synthetic test fixture"}
        else:
            submission = {"at": FIXED, "cue": cue, "submitted_once": True, "listening": False,
                          "request_sha256": sha((self.takes / f"{cue}-request01.json").read_bytes())}
        if layout != "none":
            dump(self.takes / f"{cue}-submission01.json", submission)
        takes = []
        for n, seed in enumerate(seeds, 1):
            data = wavs[n - 1] if wavs else noise(seed)
            path = self.takes / f"{cue}-download{n:02d}.wav"
            if n not in skip:
                path.write_bytes(data)
            source = f"ElevenLabs_{FIXED[:19].replace(':', '_')}_{fi.SOURCE_TAG}{' (1)' if n == 1 else ''}.wav"
            takes.append(self.take_entry(path, data, source))
        dump(self.takes / f"{cue}-downloads01.json",
             {"cue": cue, "request": f"{cue}-request01.json", "takes": takes, "distinct_pcm": len(takes),
              "listening": False, "fine_sync": False, "synthetic_fixture": True})

    def run(self, *args, lane=None, takes=None, extra=()):
        command = [sys.executable, "-I", "-B", str(TOOL), *args, "--lane", str(lane or self.lane),
                   "--takes", str(takes or self.takes), *extra]
        r = subprocess.run(command, cwd=REPO, capture_output=True, text=True)
        return r.returncode, r.stdout, r.stderr

    def plan_json(self, name):
        return json.loads((self.lane / name / "plan.json").read_text(encoding="utf-8"))

    def inputs(self, name):
        path = self.lane / name / "inputs.txt"
        return [Path(line).name for line in path.read_text(encoding="utf-8").splitlines()] if path.exists() else []

    def job(self, plan, attempt, exit_code=0, records=None, **over):
        """A synthetic guard job folder in the guard's layout (resource-receipt.json, command.log)."""
        p = self.plan_json(plan)
        frozen = sha((self.lane / plan / "frozen-inputs.json").read_bytes())
        job = self.work / f"{p['recognition']['job_prefix']}-a{attempt}"
        job.mkdir()
        receipt = {"started": FIXED, "phase": "asr", "command": p["recognition"]["fw_command"],
                   "preflight_only": False, "samples": [], "admitted": True, "inputs_sha256": frozen, "pid": 0,
                   "exit": exit_code, "final_inputs_sha256": frozen, "finished": FIXED, "synthetic_fixture": True}
        receipt.update(over)
        dump(job / "resource-receipt.json", receipt)
        if records is None:
            records = [fw_record(i, p, take) for i, take in enumerate(p["takes"])]
        (job / "command.log").write_text("".join(json.dumps(r) + "\n" for r in records), encoding="utf-8")
        return job


def fw_record(index, plan, take, status="ok", model=None, swap=("One", "1")):
    """A synthetic fw batch record: one segment per script word, 0.25 s apart, one word swapped."""
    segments, t = [], 0.0
    for word in SCRIPT[take["cue"]].split():
        if swap and word == swap[0]:
            word, swap = swap[1], None
        segments.append({"start_sec": round(t, 3), "end_sec": round(t + 0.2, 3), "text": word, "speaker": None,
                         "confidence": 0.9})
        t += 0.25
    return {"schema_version": fi.FW_RECORD, "index": index, "total": len(plan["takes"]), "input": take["path"],
            "status": status, "synthetic_fixture": True,
            "report": {"request": {"model": model or plan["recognition"]["model_path"], "backend": "whisper_cpp",
                                   "language": "en", "diarize": False,
                                   "backend_params": {"split_on_word": True, "decoding": {"max_segment_length": 1}}},
                       "result": {"backend": "whisper_cpp", "transcript": " ".join(s["text"] for s in segments),
                                  "segments": segments,
                                  "raw_output": {"model_version_tag": f"fw-native-v1+sha256:{fi.TOOL_PINS['model'][:12]}",
                                                 "word_timestamps": "dtw"}}}}


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    assert "narration_lab" not in scratch.resolve().parts
    scratch.mkdir(parents=True)
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    # Mirrors of tools/coherent_narration.py agree on every stage05 cue.
    coherent = load("coherent_narration", REPO / "tools/coherent_narration.py")
    same = coherent.read_script(REPO / fi.CRATE / "script.tsv") == SCRIPT and all(
        coherent.direction_words(t) == fi.direction_words(t) and coherent.sentences(t) == fi.sentences(t)
        for t in SCRIPT.values())
    check("mirror-coherent-narration", same, f"read_script, direction_words, sentences equal on {len(SCRIPT)} cues")

    good = ("01_1", "01_2")

    def base(name):
        fx = Fixture(scratch / name)
        fx.add("01_1", (11, 12), layout="name")
        fx.add("01_2", (21, 22), layout="hash")
        return fx

    def plan_case(name, plant, needle, cue="01_3", planned=good, want=0):
        fx = base(name)
        fx.add("01_3", (31, 32))
        plant(fx)
        code, out, err = fx.run("plan")
        reasons = []
        if code == 0:
            reasons = [r for row in json.loads(out)["refused"] if row["cue"] == cue for r in row["reasons"]]
            hit = next((r for r in reasons if needle in r), None)
            inputs = fx.inputs("plan-01")
            ok = hit and sorted({n[:4] for n in inputs}) == sorted(planned) and len(inputs) == 2 * len(planned)
            check(name, ok, f"exit 0; {cue} refused: {hit or reasons}; planned {sorted({n[:4] for n in inputs})}")
        else:
            ok = code == want and needle in err and not (fx.lane / "plan-01").exists()
            check(name, ok, f"exit {code}; {err.strip().splitlines()[-1] if err.strip() else ''}")

    # Positive control: three good cues, both submission layouts, a plan root can launch.
    fx = base("p00-good-plan")
    fx.add("01_3", (31, 32))
    code, out, err = fx.run("status")
    check("p00-status-writes-nothing", code == 0 and not fx.lane.exists() and len(json.loads(out)["eligible_wavs"]) == 6)
    code, out, err = fx.run("plan")
    p = fx.plan_json("plan-01")
    frozen = json.loads((fx.lane / "plan-01/frozen-inputs.json").read_text(encoding="utf-8"))["files"]
    roles = {}
    for row in frozen:
        roles[row["role"]] = roles.get(row["role"], 0) + 1
    check("p00-good-plan", code == 0 and fx.inputs("plan-01") == [t["name"] for t in p["takes"]]
          and len(p["takes"]) == 6 and p["not_submitted"] == len(SCRIPT) - 3 and not p["refused"],
          f"6 WAVs planned; frozen roles {dict(sorted(roles.items()))}")
    wrapper = load("guard_wrapper", REPO / fi.GUARD)
    guard_sha = wrapper.load().check_inputs(fx.lane / "plan-01/frozen-inputs.json")
    check("p00-guard-check-inputs", guard_sha == sha((fx.lane / "plan-01/frozen-inputs.json").read_bytes()),
          "root's pinned guard re-hashes all frozen inputs and accepts the manifest")
    take = p["takes"][0]
    check("p00-sample-info", take["channels"] == 1 and take["sample_rate"] == 48000 and take["sample_bits"] == 16
          and [c["id"] for c in take["chunks"]] == ["fmt ", "data", "C2PA"] and set(take["receipts"]) ==
          {"request", "submission", "downloads"}, f"{take['frames']} frames, {take['seconds']} s, chunks fmt/data/C2PA")
    dry = subprocess.run([sys.executable, "-I", "-B", str(fx.lane / "plan-01/run-asr.py"), "--attempt", "1",
                          "--dry-run"], cwd=REPO, capture_output=True, text=True)
    command = json.loads(dry.stdout)
    root_run = (REPO / fi.EL / "remaining-recognition01/run01.py").read_text(encoding="utf-8")
    root_cmd = next(ast.literal_eval(node.value) for node in ast.walk(ast.parse(root_run))
                    if isinstance(node, ast.Assign) and getattr(node.targets[0], "id", "") == "cmd")
    ours, theirs = command[command.index("--") + 1:], root_cmd[root_cmd.index("--") + 1:]
    at = theirs.index("--inputs-from") + 1
    same_fw = ours[:at] + ours[at + 1:] == theirs[:at] + theirs[at + 1:] and Path(ours[at]).name == "inputs.txt"
    check("p00-run-asr-command", dry.returncode == 0 and same_fw and command[:3] == ["python3", "-I", "-B"]
          and command[3] == root_cmd[2] and command[command.index("--job") + 1] == "complex-lane-plan-01-a1"
          and command[command.index("--inputs") + 1].endswith("plan-01/frozen-inputs.json"),
          "fw arguments equal root's run01.py except the inputs list; root's guard; new job per attempt")

    # Planted defects. Each keeps 01_1 and 01_2 good, so a refusal must stay local to the bad cue.
    req = lambda fx: fx.takes / "01_3-request01.json"
    dls = lambda fx: fx.takes / "01_3-downloads01.json"

    def stale(fx):
        def change(r):  # an earlier wording, with a text_sha256 that matches it
            r["text"] = "Earlier wording. " + r["text"]
            r["text_sha256"] = sha(r["text"].encode())
        edit(req(fx), change)

    plan_case("n01-stale-cue-text", stale, "stale cue text")
    plan_case("n02-text-hash-wrong", lambda fx: edit(req(fx), lambda r: r.update(text_sha256="0" * 64)),
              "does not hash its text")
    plan_case("n03-missing-wav", lambda fx: (fx.takes / "01_3-download02.wav").rename(
        fx.root / "moved-aside-01_3-download02.wav"), "missing WAV")

    def truncate(fx, rebind):
        path = fx.takes / "01_3-download02.wav"
        cut = path.read_bytes()[:-5000]
        path.write_bytes(cut)
        if rebind:  # a still-copying file whose receipt was written from the same partial bytes
            edit(dls(fx), lambda d: d["takes"][1].update(sha256=sha(cut)))

    plan_case("n04-partial-wav", lambda fx: truncate(fx, False), "do not hash to the receipt sha256")
    plan_case("n05-partial-wav-consistent-receipt", lambda fx: truncate(fx, True), "partial or malformed WAV")

    def silent(fx):
        data = noise(0, pcm=bytes(2 * 12000))
        (fx.takes / "01_3-download02.wav").write_bytes(data)
        edit(dls(fx), lambda d: d["takes"][1].update(sha256=sha(data)))

    plan_case("n06-silent-wav", silent, "every sample is zero")

    def stereo(fx):
        data = noise(33, fmt=(1, 2, 48000, 192000, 4, 16))
        (fx.takes / "01_3-download02.wav").write_bytes(data)
        edit(dls(fx), lambda d: d["takes"][1].update(sha256=sha(data)))

    plan_case("n07-not-mono-48k", stereo, "is not 16-bit mono 48 kHz PCM")
    plan_case("n08-mixed-voice-request", lambda fx: edit(req(fx), lambda r: r.update(
        voice="Oliver - Clean, British and Steady")), "mixed voice: request voice")
    plan_case("n09-mixed-voice-source", lambda fx: edit(dls(fx), lambda d: d["takes"][1].update(
        source=d["takes"][1]["source"].replace("Frederick Surrey - Smooth and Velvety",
                                               "Oliver - Clean, British and Steady"))), "mixed voice: 01_3-download02.wav source")

    def oliver_audio(fx):
        data = OLIVER_WAV.read_bytes()
        (fx.takes / "01_3-download01.wav").write_bytes(data)
        edit(dls(fx), lambda d: d["takes"][0].update(fx.take_entry(fx.takes / "01_3-download01.wav", data,
                                                                    d["takes"][0]["source"])))

    plan_case("n10-mixed-voice-oliver-audio-under-frederick-labels", oliver_audio,
              "audio matches a retained Oliver recording")

    def oliver_pcm(fx):  # the same Oliver samples in a new container: only the PCM hash can match
        data = noise(0, trailer=b"LIST", pcm=fi.data_chunk(OLIVER_WAV.read_bytes()))
        (fx.takes / "01_3-download01.wav").write_bytes(data)
        edit(dls(fx), lambda d: d["takes"][0].update(fx.take_entry(fx.takes / "01_3-download01.wav", data,
                                                                    d["takes"][0]["source"])))

    plan_case("n10b-mixed-voice-oliver-pcm-new-container", oliver_pcm, "audio matches a retained Oliver recording")

    def same_bytes(fx, src, dst, index):
        data = (fx.takes / src).read_bytes()
        (fx.takes / dst).write_bytes(data)
        edit(fx.takes / f"{dst[:4]}-downloads01.json", lambda d: d["takes"][index].update(sha256=sha(data),
                                                                                          pcm_sha256=pcm_of(data)["pcm_sha256"]))

    plan_case("n11-duplicate-within-cue", lambda fx: same_bytes(fx, "01_3-download01.wav", "01_3-download02.wav", 1),
              "has the same audio as 01_3-download01.wav")
    plan_case("n12-duplicate-across-cues", lambda fx: same_bytes(fx, "01_1-download01.wav", "01_3-download01.wav", 0),
              "has the same audio as 01_1-download01.wav", planned=("01_2",))

    def same_pcm(fx):
        first = (fx.takes / "01_3-download01.wav").read_bytes()
        data = noise(0, trailer=b"LIST", pcm=first[44:44 + struct.unpack_from("<I", first, 40)[0]])
        (fx.takes / "01_3-download02.wav").write_bytes(data)
        edit(dls(fx), lambda d: d["takes"][1].update(sha256=sha(data), pcm_sha256=pcm_of(data)["pcm_sha256"]))

    plan_case("n13-duplicate-pcm-other-container", same_pcm, "has the same audio as 01_3-download01.wav")

    def two_receipts(fx):
        data = json.loads(dls(fx).read_text(encoding="utf-8"))
        dump(fx.takes / "01_3-downloads02.json", data)

    plan_case("n14-wav-named-by-two-receipts", two_receipts, "is named by 01_3-downloads01.json, 01_3-downloads02.json")
    plan_case("n15-script-drift-request", lambda fx: edit(req(fx), lambda r: r.update(script_sha256=OLD_SCRIPT)),
              "script drift: request binds script 2968b0fd7654")
    plan_case("n16-incomplete-take-set", lambda fx: edit(dls(fx), lambda d: d.update(takes=d["takes"][:1])),
              "incomplete take set")
    plan_case("n17-no-submission", lambda fx: (fx.takes / "01_3-submission01.json").rename(
        fx.root / "moved-aside-01_3-submission01.json"), "0 submission receipts bind")

    def edited_request(fx):  # the request file no longer has the bytes the submission bound
        (fx.takes / "01_3-submission01.json").rename(fx.root / "moved-aside-01_3-submission01.json")
        dump(fx.takes / "01_3-submission02.json", {"at": FIXED, "cue": "01_3", "submitted_once": True,
                                                    "request_sha256": "f" * 64})

    plan_case("n18-submission-binds-another-request", edited_request, "bind another request")
    plan_case("n19-oliver-style-receipt", lambda fx: edit(dls(fx), lambda d: d.update(downloads=d.pop("takes"))),
              "not a Frederick take receipt")

    def outside(fx):
        (fx.root / "elsewhere").mkdir()
        shutil.copyfile(fx.takes / "01_3-download01.wav", fx.root / "elsewhere/01_3-download01.wav")
        edit(dls(fx), lambda d: d["takes"][0].update(path=str(fx.root / "elsewhere/01_3-download01.wav")))

    plan_case("n20-take-outside-folder", outside, "is not in the take folder")
    plan_case("n21-receipt-frames-wrong", lambda fx: edit(dls(fx), lambda d: d["takes"][0].update(frames=1)),
              "the receipt says 1")

    # Unreceipted WAVs are reported and never planned.
    fx = base("n22-unreceipted-wavs")
    (fx.takes / "01_1-download03.wav").write_bytes(noise(13))
    (fx.takes / "stray.wav").write_bytes(noise(14))
    code, out, err = fx.run("plan")
    check("n22-unreceipted-wavs", code == 0 and json.loads(out)["unreceipted_wavs"] == ["01_1-download03.wav", "stray.wav"]
          and "01_1-download03.wav" not in fx.inputs("plan-01") and len(fx.inputs("plan-01")) == 4)

    # Binding and tool pins: exit 3, nothing written.
    def crate_copy(fx, change):
        crate = fx.root / "stage/explainer_corpus" / fi.TOPIC
        (crate / "docs").mkdir(parents=True)
        for rel in ("script.tsv", "narration_units.json", fi.MAP_REL):
            shutil.copyfile(REPO / fi.CRATE / rel, crate / rel)
        change(crate)
        return crate

    fx = base("n23-script-drift-crate")
    crate = crate_copy(fx, lambda c: (c / "script.tsv").write_text((c / "script.tsv").read_text(encoding="utf-8").replace(
        "We will call it the bump.", "We call it the bump.", 1), encoding="utf-8"))
    code, out, err = fx.run("plan", extra=("--crate", str(crate)))
    check("n23-script-drift-crate", code == 3 and "script drift: script script.tsv" in err and not fx.lane.exists(),
          err.strip().split(";")[0])
    fx = base("n24-crate-copy-unchanged")
    crate = crate_copy(fx, lambda c: None)
    code, out, err = fx.run("status", extra=("--crate", str(crate)))
    check("n24-crate-copy-unchanged", code == 0, "control for n23: an exact copy of the crate is accepted")
    fx = base("n25-fw-pin-moved")
    (fx.root / "fw").write_bytes(b"not the pinned fw\n")
    code, out, err = fx.run("plan", extra=("--fw", str(fx.root / "fw")))
    check("n25-fw-pin-moved", code == 3 and "fw fw" in err and not fx.lane.exists())
    fx = base("n26-lane-inside-take-folder")
    code, out, err = fx.run("plan", lane=fx.takes / "lane")
    check("n26-lane-inside-take-folder", code == 2 and "take folder" in err and not (fx.takes / "lane").exists())
    fx = Fixture(scratch / "n27-nothing-eligible")
    fx.add("01_1", (11, 12))
    edit(fx.takes / "01_1-request01.json", lambda r: r.update(voice="Oliver - Clean, British and Steady"))
    code, out, err = fx.run("plan")
    check("n27-nothing-eligible", code == 0 and json.loads(out)["plan"] is None and not fx.lane.exists())

    # Incremental intake, in-flight exclusion, release, and ingest.
    fx = base("i-incremental")
    code, out, err = fx.run("plan")
    check("i01-plan-01", code == 0 and fx.inputs("plan-01") == [f"{c}-download0{n}.wav" for c in good for n in (1, 2)])
    code, out, err = fx.run("plan")
    report = json.loads(out)
    check("i02-in-flight-not-replanned", code == 0 and report["plan"] is None and not (fx.lane / "plan-02").exists()
          and report["excluded_wavs"][0].endswith("in flight in plan-01"))
    fx.add("01_3", (31, 32))
    code, out, err = fx.run("plan")
    check("i03-new-cue-only", code == 0 and fx.inputs("plan-02") == ["01_3-download01.wav", "01_3-download02.wav"])

    def refused_ingest(name, job, needle):
        code, out, err = fx.run("ingest", "--plan", "plan-01", "--job", str(job))
        check(name, code == 2 and needle in err and not (fx.lane / "ingest-01").exists(), err.strip())

    p1 = fx.plan_json("plan-01")
    refused_ingest("i04-ingest-job-of-another-plan", fx.job("plan-02", 1), "is not an attempt of plan-01")
    refused_ingest("i05-ingest-inputs-sha-mismatch", fx.job("plan-01", 2, inputs_sha256="0" * 64), "inputs_sha256")
    refused_ingest("i06-ingest-other-command", fx.job("plan-01", 3, command=["fw", "transcribe"]),
                   "did not run this plan's fw command")
    refused_ingest("i07-ingest-not-admitted", fx.job("plan-01", 4, admitted=False), "never launched")
    stray = dict(fw_record(0, p1, p1["takes"][0]), input=str(fx.root / "elsewhere.wav"))
    refused_ingest("i08-ingest-input-outside-plan", fx.job("plan-01", 5, records=[stray]), "names an input outside plan-01")
    twice = [fw_record(0, p1, p1["takes"][0])] * 2
    refused_ingest("i09-ingest-duplicate-record", fx.job("plan-01", 6, records=twice), "twice")
    swapped = [fw_record(1, p1, p1["takes"][0])]
    refused_ingest("i10-ingest-index-mismatch", fx.job("plan-01", 7, records=swapped), "index does not match")

    records = [fw_record(i, p1, t) for i, t in enumerate(p1["takes"])]
    records[1] = fw_record(1, p1, p1["takes"][1], status="error")
    records[3] = fw_record(3, p1, p1["takes"][3], model=str(fx.root / "other-model.bin"))
    job = fx.job("plan-01", 8, records=records)
    with (job / "command.log").open("a", encoding="utf-8") as log:
        log.write("warning: a non-JSON line from fw stderr\n")
    code, out, err = fx.run("ingest", "--plan", "plan-01", "--job", str(job))
    ingest = json.loads((fx.lane / "ingest-01/ingest.json").read_text(encoding="utf-8"))
    check("i11-ingest-partial-failures", code == 0 and [t["name"] for t in ingest["processed"]] ==
          ["01_1-download01.wav", "01_2-download01.wav"] and [f["reasons"] for f in ingest["failed"]] ==
          [["fw status 'error'"], ["record names a different model"]] and ingest["non_json_log_lines"] == 1,
          "2 processed; status error and model mismatch failed")
    rows = json.loads((fx.lane / "ingest-01/transcripts.json").read_text(encoding="utf-8"))["rows"]
    first = rows[0]["alignment"]
    check("i12-provisional-alignment", first["differences"] == [{"script": "one", "recognized": "1",
                                                                 "script_word_index": 6}]
          and all(s["onset_s"] is not None for i, s in enumerate(first["sentences"]) if i != 1)
          and first["sentences"][1]["onset_s"] is None
          and first["sentences"][1]["first_aligned_onset_s"] == 1.75 and rows[0]["listening"] is False
          and rows[0]["word_timestamps"] == "dtw", f"{first['aligned_words']}/{first['script_words']} words aligned; "
          f"sentence 1 onset unaligned at 'one', first aligned onset 1.75 s")
    code, out, err = fx.run("plan")
    check("i13-failed-wavs-eligible-again", code == 0 and fx.inputs("plan-03") == ["01_1-download02.wav",
                                                                                  "01_2-download02.wav"]
          and any(x.endswith("processed in ingest-01") for x in json.loads(out)["excluded_wavs"])
          and any(x.endswith("in flight in plan-02") for x in json.loads(out)["excluded_wavs"]))
    code, out, err = fx.run("ingest", "--plan", "plan-01", "--job", str(fx.job("plan-01", 9)))
    check("i14-ingest-closed-plan-refused", code == 2 and "already closed by ingest-01" in err)
    run_asr = load("run_asr_plan01", fx.lane / "plan-01/run-asr.py")
    check("i15-run-asr-refuses-closed-plan", run_asr.closers() == ["ingest-01/ingest.json"],
          "the generated runner sees the ingest that closed its plan (checked without launching)")
    code, out, err = fx.run("release", "--plan", "plan-02", "--reason", "synthetic test: plan not run")
    code2, out2, err2 = fx.run("release", "--plan", "plan-02", "--reason", "again")
    code3, out3, err3 = fx.run("plan")
    check("i16-release-reopens-wavs", code == 0 and code2 == 2 and code3 == 0
          and fx.inputs("plan-04") == ["01_3-download01.wav", "01_3-download02.wav"])
    code, out, err = fx.run("ingest", "--plan", "plan-03", "--job", str(fx.job("plan-03", 1, exit_code=75,
                                                                              stopped_for_resource_limit=True)))
    code2, out2, err2 = fx.run("plan")
    check("i17-resource-stop-closes-all-failed", code == 0 and not json.loads(out)["run_complete"]
          and code2 == 0 and fx.inputs("plan-05") == ["01_1-download02.wav", "01_2-download02.wav"])
    job = fx.job("plan-04", 1)
    (fx.takes / "01_3-download02.wav").write_bytes(noise(99))
    code, out, err = fx.run("ingest", "--plan", "plan-04", "--job", str(job))
    code2, out2, err2 = fx.run("plan")
    later = json.loads(out2)
    check("i18-wav-changed-after-plan", code == 0 and json.loads(out)["failed"] ==
          ["01_3 01_3-download02.wav: WAV changed or missing since the plan"] and later["plan"] is None
          and any("do not hash to the receipt" in r for row in later["refused"] for r in row["reasons"]))
    # A later take folder for the same lane: its 01_4 reuses the audio of processed 01_3-download01.wav.
    second = Fixture(fx.root / "second")
    second.add("01_4", (41, 42), wavs=[(fx.takes / "01_3-download01.wav").read_bytes(), noise(42)])
    code, out, err = fx.run("plan", takes=second.takes)
    check("i19-duplicate-of-processed", code == 0 and not (fx.lane / "plan-06").exists() and any(
        r == "duplicate input: 01_4-download01.wav has the same audio as processed 01_3-download01.wav"
        for row in json.loads(out)["refused"] for r in row["reasons"]))
    code, out, err = fx.run("status")
    report = json.loads(out)
    check("i20-status", code == 0 and report["open_plans"] == ["plan-05"] and report["processed_wavs"] == 3
          and report["closed_plans"] == {"plan-01": "ingest-01", "plan-02": "release-01.json",
                                         "plan-03": "ingest-02", "plan-04": "ingest-03"},
          "plan-05 open; 3 WAVs processed; plans 01-04 closed")

    print(f"SELFTEST: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
