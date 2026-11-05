"""Complex Frederick take intake: plan root's guarded recognition of new, complete, distinct WAVs only.

Run from the repository root with python3 -I -B. Every subcommand reads the retained Frederick take folder
(default EC/frederick-stage05-01) against the pinned combined-repair-stage05 script, units and map.

  plan    --lane DIR   validate every downloads receipt and WAV, then write the next DIR/plan-NN/ (plan.json,
                       inputs.txt, run-asr.py, frozen-inputs.json) for WAVs that are complete, distinct and
                       Frederick-only, and that no ingest has processed and no open plan holds. If nothing is
                       new it writes nothing.
  ingest  --lane DIR --plan plan-NN --job JOBDIR
                       bind root's guard job (resource-receipt.json, command.log) to that plan, re-hash every WAV,
                       and write DIR/ingest-NN/ (ingest.json, transcripts.json). The ingest closes the plan:
                       WAVs with an ok record become processed; every other WAV becomes eligible again.
  release --lane DIR --plan plan-NN --reason TEXT
                       close a plan that will not be run, so its WAVs become eligible again.
  status  --lane DIR   print what plan would do now; writes nothing.

The tool never runs fw, the guard, synthesis, builds or renders. It never reads Oliver timings: retained Oliver
WAVs and receipts are read only for hashes, so that an Oliver recording under a Frederick label is refused. It never edits
a receipt, a WAV, or an earlier plan, ingest or release. Recognized word times are provisional DTW marks, not
fine sync, phrase timings or listening. Exit codes: 0 done, 2 refused, 3 a pinned binding or tool moved.
"""
import argparse
from array import array
import datetime
import difflib
import hashlib
import io
import json
from pathlib import Path
import re
import struct
import sys
import wave

SCHEMA = "complex-frederick-intake-v1"
REPO = Path.cwd()
HERE = Path(__file__).resolve()
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
TOPIC = "complex_analysis_visual"
CRATE = f"{EC}/combined-repair-stage05/explainer_corpus/{TOPIC}"
TAKES = f"{EC}/frederick-stage05-01"
# The map keeps its oliver12 file name as reviewed provenance only (combined-repair-stage05-integration01 limits).
MAP_REL = "docs/narration-direction-map-oliver12.json"
PINS = {
    "script": "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3",
    "units": "e776c860dcfa66c277ef0e188437cade034f0427d19efa4c975dde313d9b51c6",
    "map": "70888624edd6e930b383b277b50f8da5e39ac94f1a661c71b7f0e9575362f20c",
}
GUARD = ("docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/"
         "guard_owner56g_target20g.py")
GUARD_SOURCE = ("narration_lab/production-20261020/second-wave-central/duration-and-storage-20261020/root/"
                "run_bounded_phase_oliver_owner48g_target14g_20261102.py")
WORK = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
FW_HOME = ".local/bin/fw"
MODEL_HOME = ".cache/franken_whisper/models/whisper/whisper-large-v3-turbo-f16-v1/ggml-large-v3-turbo.bin"
TOOL_PINS = {
    "guard": "1c19a9b8f817c3bca06e18fd0785ac21c3cb935be715d67ecc41e4e39d1f5097",
    "guard_source": "819746fb33110d6c133e184812c80a00562d848816f623078ad86f0fb1a316da",
    "fw": "7b570cf44d1f413ead01340c45e4070ec71009b7c2482d71a2aebf49e8fda47f",
    "model": "1fc70f774d38eb169993ac391eea357ef47c88757ef72ee5943879b7e8e2bc69",
}
# Retained Oliver take folders: their WAVs and receipts are read for hashes only.
OTHER_VOICE = (f"{EC}/ui-takes-stage04-01", f"{EC}/counting-oliver02", f"{EL}/ui-takes01")
VOICE = "Frederick Surrey - Smooth and Velvety"
SOURCE_TAG = VOICE + "_pvc_s50_sim75_v4"
SETTINGS = {"model": "Eleven v4", "stability": 0.5, "similarity": 0.75, "clone_strength": 0,
            "output": "WAV 48 kHz Lossless"}
FMT = (1, 1, 48000, 96000, 2, 16)  # PCM, mono, 48 kHz, byte rate, block align, 16-bit
RATE = 48000
FW_RECORD = "franken-whisper-batch-result-v1"
NAMES = {
    "request": re.compile(r"(\d\d_\d+)-request(\d\d)\.json"),
    "submission": re.compile(r"(\d\d_\d+)-submission(\d\d)\.json"),
    "downloads": re.compile(r"(\d\d_\d+)-downloads(\d\d)\.json"),
    "wav": re.compile(r"(\d\d_\d+)-download(\d\d)\.wav"),
}
LIMITS = ("Intake and recognition bookkeeping only. No listening, take selection, fine sync or acceptance. "
          "Recognized word times are provisional DTW marks; Oliver timings are never read or reused.")


class Refused(Exception):
    """Exit 2: the operation is refused and nothing was written."""


class PinMoved(Exception):
    """Exit 3: a pinned binding or tool moved; review before running again."""


def utc():
    return datetime.datetime.now(datetime.timezone.utc).isoformat()


def sha_bytes(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def load_json(path):
    return json.loads(Path(path).read_text(encoding="utf-8"))


def write_new(path, text):
    with open(path, "x", encoding="utf-8") as handle:
        handle.write(text)


def dump_new(path, data):
    write_new(path, json.dumps(data, indent=1, ensure_ascii=False) + "\n")


# The next three mirror tools/coherent_narration.py (read_script, sentences, direction_words); the self-test
# compares them on every stage05 cue, so a later edit of that file cannot silently change this tool.
def read_script(path):
    result = {}
    for number, row in enumerate(Path(path).read_text(encoding="utf-8").splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        if len(cells) != 2 or not re.fullmatch(r"\d\d_\d+", cells[0]) or not cells[1].strip():
            raise PinMoved(f"invalid script row {number}")
        if cells[0] in result:
            raise PinMoved(f"duplicate script cue {cells[0]}")
        result[cells[0]] = cells[1]
    return result


def sentences(text):
    return [s.strip() for s in re.findall(r".*?[.?!](?= |$)|.+$", text) if s.strip()]


def direction_words(text):
    return [m.lower().replace("’", "'") for m in re.findall(r"[A-Za-z0-9]+(?:['’][A-Za-z]+)?", text)]


def check_binding(crate):
    """The stage05 script, units and map, by pin and by meaning. Returns (script rows, {role: path})."""
    files = {"script": crate / "script.tsv", "units": crate / "narration_units.json", "map": crate / MAP_REL}
    moved = []
    for role, path in files.items():
        got = sha_file(path) if path.is_file() else "missing"
        if got != PINS[role]:
            moved.append(f"script drift: {role} {path.name} is {got[:12]}, not the pinned {PINS[role][:12]}")
    if moved:
        raise PinMoved("; ".join(moved))
    script = read_script(files["script"])
    units = load_json(files["units"])
    contract = units.get("direction_contract") or {}
    problems = []
    if units.get("script_sha256") != PINS["script"] or contract.get("script_sha256") != PINS["script"]:
        problems.append("units do not bind the script")
    if contract.get("topic") != TOPIC or contract.get("map") != {
            "path": f"explainer_corpus/{TOPIC}/{MAP_REL}", "sha256": PINS["map"]}:
        problems.append("units do not bind the map")
    texts = {}
    for unit in units.get("units", []):
        ids = unit.get("line_ids")
        if not isinstance(ids, list) or len(ids) != 1 or ids[0] in texts:
            problems.append(f"unit {unit.get('id')} is not one new cue")
        else:
            texts[ids[0]] = unit.get("text")
    if texts != script:
        problems.append("unit texts are not the script rows one to one")
    direction = load_json(files["map"])
    if direction.get("scripts", {}).get(TOPIC) != PINS["script"]:
        problems.append("map does not bind the script")
    if any(item.get("cue") not in script for item in direction.get("overrides", [])):
        problems.append("map names a cue outside the script")
    if problems:
        raise PinMoved("; ".join(problems))
    return script, files


def tool_paths(args):
    return {"guard": REPO / GUARD, "guard_source": REPO / GUARD_SOURCE, "fw": Path(args.fw), "model": Path(args.model)}


def check_tools(args):
    paths = tool_paths(args)
    moved = [f"{role} {path.name}: {(sha_file(path) if path.is_file() else 'missing')[:12]} is not the pinned "
             f"{TOOL_PINS[role][:12]}" for role, path in paths.items()
             if not path.is_file() or sha_file(path) != TOOL_PINS[role]]
    if moved:
        raise PinMoved("; ".join(moved))
    return paths


def data_chunk(data):
    """The data chunk body of a RIFF file, or None; lenient, for the other-voice inventory only."""
    at = 12
    while data[:4] == b"RIFF" and at + 8 <= len(data):
        size = struct.unpack_from("<I", data, at + 4)[0]
        if data[at:at + 4] == b"data":
            return data[at + 8:at + 8 + size]
        at += 8 + size + (size & 1)
    return None


def other_voice_hashes(folders):
    """Hashes of retained Oliver recordings: every WAV in each folder (file and PCM sha256), plus every sha256 or
    pcm_sha256 field in its JSON receipts, whatever their layout (downloads, recordings or takes)."""
    receipts, wavs, hashes = 0, 0, set()

    def walk(value):
        if isinstance(value, dict):
            for key, item in value.items():
                if key in ("sha256", "pcm_sha256") and isinstance(item, str) and re.fullmatch(r"[0-9a-f]{64}", item):
                    hashes.add(item)
                else:
                    walk(item)
        elif isinstance(value, list):
            for item in value:
                walk(item)

    for folder in map(Path, folders):
        found = sorted(folder.glob("*.wav"))
        if not found:
            raise PinMoved(f"other-voice inventory {folder.name} has no WAV")
        for path in found:
            data = path.read_bytes()
            wavs += 1
            hashes.add(sha_bytes(data))
            pcm = data_chunk(data)
            if pcm:
                hashes.add(sha_bytes(pcm))
        for path in sorted(folder.glob("*.json")):
            data, _ = read_receipt(path)
            if data is not None:
                receipts += 1
                walk(data)
    return {"folders": len(folders), "wavs": wavs, "receipts": receipts, "hashes": len(hashes)}, hashes


def probe_wav(data):
    """Walk every RIFF chunk of a whole file; raise ValueError naming the first defect."""
    if len(data) < 12 or data[:4] != b"RIFF" or data[8:12] != b"WAVE":
        raise ValueError("not a RIFF/WAVE file")
    declared = struct.unpack_from("<I", data, 4)[0] + 8
    if declared != len(data):
        raise ValueError(f"RIFF declares {declared} bytes but the file has {len(data)}")
    at, chunks, fmt, pcm = 12, [], None, None
    while at < len(data):
        if at + 8 > len(data):
            raise ValueError("truncated chunk header")
        cid, size = data[at:at + 4], struct.unpack_from("<I", data, at + 4)[0]
        body, end = at + 8, at + 8 + size
        if end > len(data):
            raise ValueError(f"chunk {cid.decode('latin-1')!r} runs past the end of the file")
        chunks.append({"id": cid.decode("latin-1"), "bytes": size})
        if cid == b"fmt ":
            if fmt is not None or size < 16:
                raise ValueError("second or short fmt chunk")
            fmt = struct.unpack_from("<HHIIHH", data, body)
        elif cid == b"data":
            if pcm is not None:
                raise ValueError("second data chunk")
            pcm = data[body:end]
        at = end + (size & 1)
    if at != len(data):
        raise ValueError("missing pad byte after the last chunk")
    if fmt is None or pcm is None:
        raise ValueError("no fmt or no data chunk")
    if fmt != FMT:
        raise ValueError(f"format {fmt} is not 16-bit mono 48 kHz PCM")
    if not pcm or len(pcm) % 2:
        raise ValueError("empty or odd-length PCM data")
    with wave.open(io.BytesIO(data)) as reader:
        frames = reader.getnframes()
        if (reader.getnchannels(), reader.getsampwidth(), reader.getframerate()) != (1, 2, RATE) or \
                frames != len(pcm) // 2 or sha_bytes(reader.readframes(frames)) != sha_bytes(pcm):
            raise ValueError("the wave module reads different samples")
    samples = array("h", pcm)
    if sys.byteorder != "little":
        samples.byteswap()
    peak = max(max(samples), -min(samples))
    if peak == 0:
        raise ValueError("every sample is zero")
    return {"frames": frames, "seconds": frames / RATE, "sample_rate": RATE, "channels": 1, "sample_bits": 16,
            "pcm_sha256": sha_bytes(pcm), "peak": peak, "chunks": chunks}


def name_groups(takes_dir):
    groups = {kind: {} for kind in NAMES}
    loose = []
    for path in sorted(takes_dir.iterdir()):
        if not path.is_file():
            continue
        for kind, pattern in NAMES.items():
            match = pattern.fullmatch(path.name)
            if match:
                groups[kind][path.name] = match.group(1)
                break
        else:
            if path.suffix.lower() == ".wav":
                loose.append(path.name)
    return groups, loose


def check_take(takes_dir, cue, take, other):
    why = []
    raw = str(take.get("path", ""))
    name = Path(raw).name
    rec = {"cue": cue, "name": name}
    match = NAMES["wav"].fullmatch(name)
    if not match or match.group(1) != cue:
        why.append(f"take name {name!r} is not a {cue}-downloadNN.wav")
    if not Path(raw).is_absolute() or Path(raw).parent.resolve() != takes_dir.resolve():
        why.append(f"take {name} is not in the take folder")
    source = Path(str(take.get("source", ""))).name
    rec["source_name"] = source
    if not (source.startswith("ElevenLabs_") and SOURCE_TAG in source and source.endswith(".wav")) \
            or "oliver" in source.lower():
        why.append(f"mixed voice: {name} source {source!r} is not a Frederick v4 download")
    local = takes_dir / name
    if not name or not local.is_file():
        why.append(f"missing WAV: {name or raw!r}")
        return rec, why
    data = local.read_bytes()
    rec.update(path=str(local.resolve()), sha256=sha_bytes(data), bytes=len(data))
    if rec["sha256"] != take.get("sha256"):
        why.append(f"{name} bytes do not hash to the receipt sha256 (partial, replaced or still copying)")
    try:
        info = probe_wav(data)
    except ValueError as error:
        why.append(f"partial or malformed WAV {name}: {error}")
        return rec, why
    rec.update(info)
    if info["pcm_sha256"] != take.get("pcm_sha256"):
        why.append(f"{name} PCM does not hash to the receipt pcm_sha256")
    if info["frames"] != take.get("frames"):
        why.append(f"{name} has {info['frames']} frames, the receipt says {take.get('frames')}")
    seconds = take.get("seconds")
    if not isinstance(seconds, (int, float)) or abs(seconds - info["seconds"]) > 0.005:
        why.append(f"{name} lasts {info['seconds']:.4f} s, the receipt says {seconds}")
    if rec["sha256"] in other or info["pcm_sha256"] in other:
        why.append(f"mixed voice: {name} audio matches a retained Oliver recording")
    return rec, why


def read_receipt(path):
    """Parse a receipt from one read, so the hash a plan freezes is the hash of the bytes that were checked."""
    try:
        data = Path(path).read_bytes()
        return json.loads(data.decode("utf-8")), sha_bytes(data)
    except (OSError, ValueError):
        return None, None


def binds(submission, request_name, request_sha):
    """Root's two submission layouts: the request by name (submitted, generation_requests), or by the sha256 of
    the request file (submitted_once). A receipt that carries both must match both."""
    by_name, by_hash = submission.get("request"), submission.get("request_sha256")
    if by_name is None and by_hash is None:
        return False
    return (by_name is None or by_name == request_name) and (by_hash is None or by_hash == request_sha)


def check_receipt(takes_dir, receipt, cue, script, other, groups):
    """Reasons to refuse one downloads receipt, its take records, and the receipts it rests on."""
    reasons, takes = [], []
    data, digest = read_receipt(takes_dir / receipt)
    rests = {"downloads": {"name": receipt, "sha256": digest}}
    if not isinstance(data, dict):
        return ["unreadable receipt"], takes, rests
    if data.get("cue") != cue:
        reasons.append(f"receipt cue {data.get('cue')!r} is not {cue}")
    if cue not in script:
        reasons.append("cue is not in the stage05 script")
    if not isinstance(data.get("takes"), list) or not data["takes"]:
        reasons.append("not a Frederick take receipt: no takes list"
                       + (" (an Oliver-style downloads list)" if "downloads" in data else ""))
        return reasons, takes, rests
    request_name = data.get("request")
    if not isinstance(request_name, str) or groups["request"].get(request_name) != cue:
        reasons.append(f"request {request_name!r} is not a {cue} request in the take folder")
    else:
        request, digest = read_receipt(takes_dir / request_name)
        rests["request"] = {"name": request_name, "sha256": digest}
        if not isinstance(request, dict):
            reasons.append(f"unreadable request {request_name}")
            request = {"cue": cue, "text": script.get(cue), "script_sha256": PINS["script"], "voice": VOICE,
                       "text_sha256": sha_bytes(str(script.get(cue)).encode("utf-8")), **SETTINGS}
        text = request.get("text")
        if request.get("cue") != cue:
            reasons.append(f"request cue {request.get('cue')!r} is not {cue}")
        if cue in script and text != script[cue]:
            reasons.append("stale cue text: the request text is not the stage05 script row")
        if not isinstance(text, str) or request.get("text_sha256") != sha_bytes(text.encode("utf-8")):
            reasons.append("request text_sha256 does not hash its text")
        if request.get("script_sha256") != PINS["script"]:
            reasons.append(f"script drift: request binds script {str(request.get('script_sha256'))[:12]}, "
                           f"not stage05 {PINS['script'][:12]}")
        if request.get("voice") != VOICE:
            reasons.append(f"mixed voice: request voice {request.get('voice')!r}")
        for key, want in SETTINGS.items():
            if request.get(key) != want or type(request.get(key)) is bool:
                reasons.append(f"request {key} {request.get(key)!r}, expected {want!r}")
    request_sha = rests.get("request", {}).get("sha256")
    submissions, others = [], []
    for name, owner in sorted(groups["submission"].items()):
        if owner != cue:
            continue
        submission, digest = read_receipt(takes_dir / name)
        if isinstance(submission, dict) and binds(submission, request_name, request_sha):
            submissions.append((name, submission, digest))
        else:
            others.append(name)
    generations = None
    if len(submissions) != 1:
        reasons.append(f"{len(submissions)} submission receipts bind {request_name!r}; expected one"
                       + (f" ({', '.join(others)} bind another request or are unreadable)" if others else ""))
    else:
        name, submission, digest = submissions[0]
        rests["submission"] = {"name": name, "sha256": digest}
        once = submission.get("submitted_once") is True
        generations = submission.get("generation_requests", 1 if once else None)
        if not (once or submission.get("submitted") is True) or submission.get("submitted", True) is not True:
            reasons.append("submission is not marked submitted")
        if submission.get("cue", cue) != cue:
            reasons.append(f"submission cue {submission.get('cue')!r} is not {cue}")
        if type(generations) is not int or generations < 1 or (once and generations != 1):
            reasons.append(f"submission generation_requests {generations!r} is not a positive count")
            generations = None
    if generations and len(data["takes"]) != 2 * generations:
        reasons.append(f"incomplete take set: {len(data['takes'])} takes for {generations} generation "
                       f"request(s); expected {2 * generations}")
    seen = set()
    for take in data["takes"]:
        rec, why = check_take(takes_dir, cue, take if isinstance(take, dict) else {}, other)
        if rec["name"] in seen:
            why.append(f"duplicate input: {rec['name']} is listed twice")
        seen.add(rec["name"])
        reasons += why
        takes.append(rec)
    return reasons, takes, rests


def scan(takes_dir, script, other):
    """Validate every downloads receipt in the take folder; reads only."""
    groups, loose = name_groups(takes_dir)
    entries = []
    for receipt, cue in sorted(groups["downloads"].items()):
        reasons, takes, rests = check_receipt(takes_dir, receipt, cue, script, other, groups)
        entries.append({"cue": cue, "receipt": receipt, "reasons": reasons, "takes": takes, "rests": rests})
    by_name, by_audio = {}, {}
    for entry in entries:
        for take in entry["takes"]:
            by_name.setdefault(take["name"], set()).add(entry["receipt"])
            for key in ("sha256", "pcm_sha256"):
                if take.get(key):
                    by_audio.setdefault(take[key], set()).add((entry["receipt"], take["name"]))
    for entry in entries:
        for take in entry["takes"]:
            if len(by_name[take["name"]]) > 1:
                entry["reasons"].append(f"duplicate input: {take['name']} is named by "
                                        f"{', '.join(sorted(by_name[take['name']]))}")
            twins = sorted({name for key in ("sha256", "pcm_sha256") if take.get(key)
                            for _, name in by_audio[take[key]]} - {take["name"]})
            if twins:
                entry["reasons"].append(f"duplicate input: {take['name']} has the same audio as {', '.join(twins)}")
    named = {take["name"] for entry in entries for take in entry["takes"]}
    downloaded = {entry["cue"] for entry in entries}
    submitted = set(groups["submission"].values())
    return {
        "ready": [e for e in entries if not e["reasons"]],
        "refused": [{"cue": e["cue"], "receipt": e["receipt"], "reasons": list(dict.fromkeys(e["reasons"]))}
                    for e in entries if e["reasons"]],
        "awaiting_download": sorted(submitted - downloaded),
        "not_submitted": sorted(set(script) - submitted - downloaded),
        "unreceipted_wavs": sorted(set(groups["wav"]) - named) + loose,
        "receipts": len(entries),
    }


def check_lane(lane, takes_dir, crate):
    if not re.fullmatch(r"[a-z0-9][a-z0-9_-]*", lane.name):
        raise Refused("lane folder name must be lowercase letters, digits, - or _ (it names guard jobs)")
    for owner, label in ((takes_dir, "take folder"), (crate.parents[1], "stage05 crate"), (HERE.parent, "tool bundle"),
                         (REPO / WORK, "guard work folder")):
        if lane.resolve() == owner.resolve() or lane.resolve().is_relative_to(owner.resolve()):
            raise Refused(f"lane must not be inside the {label}")
    if not lane.parent.is_dir():
        raise Refused("lane parent folder does not exist")


def lane_state(lane):
    """Plans, closures and processed WAVs, from the lane's own immutable files."""
    plans, closed, processed = {}, {}, {}
    if not lane.is_dir():
        return {"plans": plans, "closed": closed, "processed": processed}
    for folder in sorted(lane.glob("plan-[0-9][0-9]")):
        if not (folder / "plan.json").is_file():
            raise Refused(f"{folder.name} has no plan.json; move the folder aside before planning again")
        plans[folder.name] = load_json(folder / "plan.json")
    closers = [(p.parent.name, load_json(p)) for p in sorted(lane.glob("ingest-[0-9][0-9]/ingest.json"))]
    closers += [(p.name, load_json(p)) for p in sorted(lane.glob("release-[0-9][0-9].json"))]
    for name, data in closers:
        if data.get("plan") not in plans or data["plan"] in closed:
            raise Refused(f"{name} closes {data.get('plan')!r}, which is unknown or already closed")
        closed[data["plan"]] = name
        for take in data.get("processed", []):
            processed[take["sha256"]] = dict(take, ingest=name)
    return {"plans": plans, "closed": closed, "processed": processed}


def next_name(lane, prefix, suffix=""):
    numbers = [int(p.name[len(prefix) + 1:len(prefix) + 3]) for p in lane.glob(f"{prefix}-[0-9][0-9]{suffix}")]
    number = max(numbers, default=0) + 1
    if number > 99:
        raise Refused(f"no {prefix} number left")
    return f"{prefix}-{number:02d}{suffix}"


def classify(found, state):
    processed = state["processed"]
    processed_pcm = {take["pcm_sha256"]: take for take in processed.values()}
    flight = {take["sha256"]: (plan, take) for plan, data in state["plans"].items()
              if plan not in state["closed"] for take in data["takes"]}
    flight_pcm = {take["pcm_sha256"]: (plan, take) for plan, take in flight.values()}
    eligible, excluded, refused = [], [], list(found["refused"])
    for entry in found["ready"]:
        dup = []
        for take in entry["takes"]:
            key = (take["cue"], take["name"], take["sha256"])
            old = processed.get(take["sha256"]) or processed_pcm.get(take["pcm_sha256"])
            if old and (old["cue"], old["name"], old["sha256"]) != key:
                dup.append(f"duplicate input: {take['name']} has the same audio as processed {old['name']}")
            plan, held = flight.get(take["sha256"]) or flight_pcm.get(take["pcm_sha256"]) or (None, None)
            if held and (held["cue"], held["name"], held["sha256"]) != key:
                dup.append(f"duplicate input: {take['name']} has the same audio as {held['name']} in {plan}")
        if dup:
            refused.append({"cue": entry["cue"], "receipt": entry["receipt"], "reasons": dup})
            continue
        for take in entry["takes"]:
            if take["sha256"] in processed:
                excluded.append({"cue": take["cue"], "name": take["name"],
                                 "why": f"processed in {processed[take['sha256']]['ingest']}"})
            elif take["sha256"] in flight:
                excluded.append({"cue": take["cue"], "name": take["name"],
                                 "why": f"in flight in {flight[take['sha256']][0]}"})
            else:
                eligible.append(dict(take, receipts=entry["rests"]))
    refused.sort(key=lambda r: r["receipt"])
    return eligible, excluded, refused


def gather(args):
    takes_dir, crate, lane = Path(args.takes), Path(args.crate), Path(args.lane)
    if not (REPO / GUARD).is_file() or not (REPO / "tools/coherent_narration.py").is_file():
        raise Refused("run from the repository root")
    check_lane(lane, takes_dir, crate)
    script, crate_files = check_binding(crate)
    tools = check_tools(args)
    inventory, other = other_voice_hashes(args.other_voice or [REPO / folder for folder in OTHER_VOICE])
    found = scan(takes_dir, script, other)
    state = lane_state(lane)
    eligible, excluded, refused = classify(found, state)
    return {"takes_dir": takes_dir, "crate_files": crate_files, "tools": tools, "lane": lane, "script": script,
            "found": found, "state": state, "eligible": eligible, "excluded": excluded, "refused": refused,
            "other_voice": inventory}


def summary(ctx):
    found, state = ctx["found"], ctx["state"]
    return {
        "script_cues": len(ctx["script"]),
        "downloads_receipts": found["receipts"],
        "eligible_wavs": [f"{t['cue']} {t['name']}" for t in ctx["eligible"]],
        "excluded_wavs": [f"{x['cue']} {x['name']}: {x['why']}" for x in ctx["excluded"]],
        "refused": ctx["refused"],
        "awaiting_download": found["awaiting_download"],
        "not_submitted": len(found["not_submitted"]),
        "unreceipted_wavs": found["unreceipted_wavs"],
        "open_plans": sorted(set(state["plans"]) - set(state["closed"])),
        "closed_plans": {plan: closer for plan, closer in sorted(state["closed"].items())},
        "processed_wavs": len(state["processed"]),
        "other_voice_inventory": ctx["other_voice"],
    }


RUN_TEMPLATE = '''"""Root's guarded fw recognition for {plan}. Generated by frederick_intake.py; do not edit.

From the repository root:  python3 -I -B {run_rel} --attempt N [--dry-run]
Every attempt is a new guard job ({prefix}-aN); the guard hashes frozen-inputs.json before and after fw.
Recognized word times are provisional DTW marks: not fine sync, not phrase timings, not listening.
"""
import argparse
import json
from pathlib import Path
import subprocess
import sys

REPO = Path({repo!r})
PLAN = Path({plan_dir!r})
PREFIX = {prefix!r}
GUARD = {guard!r}
FW = {fw!r}


def closers():
    lane = PLAN.parent
    paths = sorted(lane.glob("ingest-[0-9][0-9]/ingest.json")) + sorted(lane.glob("release-[0-9][0-9].json"))
    return [str(p.relative_to(lane)) for p in paths if json.loads(p.read_text(encoding="utf-8")).get("plan") == PLAN.name]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--attempt", type=int, required=True)
    parser.add_argument("--dry-run", action="store_true", help="print the guard command and exit")
    args = parser.parse_args()
    if args.attempt < 1:
        parser.error("attempts count from 1")
    command = ["python3", "-I", "-B", GUARD, "--phase", "asr", "--job", f"{{PREFIX}}-a{{args.attempt}}",
               "--inputs", str(PLAN / "frozen-inputs.json"), "--", *FW]
    if args.dry_run:
        print(json.dumps(command, indent=1))
        return 0
    if Path.cwd().resolve() != REPO.resolve():
        parser.error("run from the repository root")
    if closers():
        parser.error(f"{{PLAN.name}} is closed by {{closers()}}; plan again instead")
    return subprocess.run(command, cwd=REPO).returncode


if __name__ == "__main__":
    sys.exit(main())
'''


def cmd_plan(args):
    ctx = gather(args)
    report = summary(ctx)
    if args.command == "status" or not ctx["eligible"]:
        report["plan"] = None if args.command == "plan" else "status only; nothing written"
        print(json.dumps(report, indent=1, ensure_ascii=False))
        return 0
    lane = ctx["lane"]
    lane.mkdir(exist_ok=True)
    name = next_name(lane, "plan")
    plan_dir = lane / name
    plan_dir.mkdir()
    prefix = f"complex-{lane.name}-{name}"
    tools = ctx["tools"]
    fw_command = [str(tools["fw"]), "transcribe", "--inputs-from", str(plan_dir.resolve() / "inputs.txt"),
                  "--backend", "whisper-cpp", "--model", str(tools["model"]), "--language", "en", "--no-diarize",
                  "--no-persist", "--split-on-word", "--max-segment-length", "1", "--threads", "4", "--timeout",
                  "180", "--json"]
    takes_dir = ctx["takes_dir"]
    receipts = {r["name"]: r["sha256"] for take in ctx["eligible"] for r in take["receipts"].values()}
    takes = []
    for take in ctx["eligible"]:
        row = {key: take[key] for key in ("cue", "name", "path", "sha256", "pcm_sha256", "bytes", "frames", "seconds",
                                          "sample_rate", "channels", "sample_bits", "peak", "chunks", "source_name")}
        row["receipts"] = dict(sorted(take["receipts"].items()))
        row["text_sha256"] = sha_bytes(ctx["script"][take["cue"]].encode("utf-8"))
        takes.append(row)
    plan = {
        "schema": SCHEMA, "kind": "plan", "plan": name, "at": utc(), "lane": lane.name,
        "tool": {"name": HERE.name, "sha256": sha_file(HERE)},
        "binding": {"script_sha256": PINS["script"], "units_sha256": PINS["units"], "map_sha256": PINS["map"],
                    "map_name": "oliver12 is reviewed provenance only, not voice authorization",
                    "voice": VOICE, "settings": SETTINGS},
        "recognition": {"job_prefix": prefix, "fw_command": fw_command, "model_path": str(tools["model"]),
                        "tool_sha256": dict(TOOL_PINS)},
        "takes": takes,
        **{key: value for key, value in summary(ctx).items() if key not in ("eligible_wavs",)},
        "limits": LIMITS,
    }
    dump_new(plan_dir / "plan.json", plan)
    write_new(plan_dir / "inputs.txt", "".join(take["path"] + "\n" for take in takes))
    write_new(plan_dir / "run-asr.py", RUN_TEMPLATE.format(
        plan=name, run_rel=f"{args.lane}/{name}/run-asr.py", prefix=prefix, repo=str(REPO.resolve()),
        plan_dir=str(plan_dir.resolve()), guard=str((REPO / GUARD).resolve()), fw=fw_command))
    # Pinned tools and checked bytes are frozen at the hash that was checked: a later change fails the guard.
    rows = [(role, tools[role], TOOL_PINS[role]) for role in ("guard", "guard_source", "fw", "model")]
    rows.append(("intake_tool", HERE, plan["tool"]["sha256"]))
    rows += [(f"crate_{role}", path, PINS[role]) for role, path in ctx["crate_files"].items()]
    rows += [("receipt", takes_dir / name, digest) for name, digest in sorted(receipts.items())]
    rows += [("wav", Path(take["path"]), take["sha256"]) for take in takes]
    rows += [(role, plan_dir / file, None) for role, file in
             (("plan", "plan.json"), ("inputs", "inputs.txt"), ("run", "run-asr.py"))]
    frozen = [{"path": str(Path(path).resolve()), "sha256": digest or sha_file(path), "role": role}
              for role, path, digest in rows]
    dump_new(plan_dir / "frozen-inputs.json", {"files": frozen})
    report["plan"] = name
    report["frozen_inputs_sha256"] = sha_file(plan_dir / "frozen-inputs.json")
    report["next"] = f"python3 -I -B {args.lane}/{name}/run-asr.py --attempt 1"
    print(json.dumps(report, indent=1, ensure_ascii=False))
    return 0


def align(text, segments):
    """Provisional word alignment of the script row to recognized words (difflib on direction_words tokens)."""
    words = direction_words(text)
    heard = []
    for segment in segments:
        for token in direction_words(str(segment.get("text", ""))):
            heard.append({"word": token, "start_s": segment.get("start_sec"), "end_s": segment.get("end_sec"),
                          "confidence": segment.get("confidence")})
    timing = [None] * len(words)
    differences = []
    matcher = difflib.SequenceMatcher(None, words, [h["word"] for h in heard], autojunk=False)
    for op, i1, i2, j1, j2 in matcher.get_opcodes():
        if op == "equal":
            for k in range(i2 - i1):
                timing[i1 + k] = heard[j1 + k]
        else:
            differences.append({"script": " ".join(words[i1:i2]), "recognized": " ".join(h["word"] for h in heard[j1:j2]),
                                "script_word_index": i1})
    rows, at = [], 0
    for number, sentence in enumerate(sentences(text)):
        count = len(direction_words(sentence))
        span = range(at, at + count)
        aligned = [k for k in span if timing[k]]
        rows.append({"sentence": number, "text": sentence, "first_word_index": at, "words": count,
                     "onset_s": timing[at]["start_s"] if count and timing[at] else None,
                     "first_aligned_word_index": aligned[0] if aligned else None,
                     "first_aligned_onset_s": timing[aligned[0]]["start_s"] if aligned else None,
                     "last_aligned_end_s": timing[aligned[-1]]["end_s"] if aligned else None})
        at += count
    if at != len(words):
        raise ValueError("sentence words do not add up to the cue words")
    return {"script_words": len(words), "recognized_words": len(heard),
            "aligned_words": sum(1 for t in timing if t), "differences": differences, "sentences": rows,
            "words": [{"word": w, "start_s": t["start_s"] if t else None, "end_s": t["end_s"] if t else None}
                      for w, t in zip(words, timing)],
            "recognized": heard}


def record_problems(record, plan):
    report = record.get("report") or {}
    request, result = report.get("request") or {}, report.get("result") or {}
    params = request.get("backend_params") or {}
    raw = result.get("raw_output") or {}
    checks = [
        (record.get("status") == "ok", f"fw status {record.get('status')!r}"),
        (request.get("model") == plan["recognition"]["model_path"], "record names a different model"),
        (request.get("backend") == "whisper_cpp" and result.get("backend") == "whisper_cpp", "backend is not whisper_cpp"),
        (request.get("language") == "en" and request.get("diarize") is False, "language or diarization differs"),
        (params.get("split_on_word") is True and (params.get("decoding") or {}).get("max_segment_length") == 1,
         "not word-split recognition"),
        (f"sha256:{TOOL_PINS['model'][:12]}" in str(raw.get("model_version_tag", "")), "model tag is not the pinned model"),
        (isinstance(result.get("segments"), list) and isinstance(result.get("transcript"), str), "no transcript"),
    ]
    return [message for ok, message in checks if not ok]


def cmd_ingest(args):
    lane, takes_dir = Path(args.lane), Path(args.takes)
    state = lane_state(lane)
    if args.plan not in state["plans"]:
        raise Refused(f"no plan {args.plan!r} in the lane")
    if args.plan in state["closed"]:
        raise Refused(f"{args.plan} is already closed by {state['closed'][args.plan]}")
    plan = state["plans"][args.plan]
    script, _ = check_binding(Path(args.crate))
    job = Path(args.job)
    if not re.fullmatch(re.escape(plan["recognition"]["job_prefix"]) + r"-a[1-9]\d*", job.name):
        raise Refused(f"job {job.name!r} is not an attempt of {args.plan}")
    receipt_path, log_path = job / "resource-receipt.json", job / "command.log"
    if not receipt_path.is_file():
        raise Refused("job has no resource-receipt.json")
    receipt = load_json(receipt_path)
    frozen_sha = sha_file(lane / args.plan / "frozen-inputs.json")
    if receipt.get("phase") != "asr" or receipt.get("command") != plan["recognition"]["fw_command"]:
        raise Refused("the job did not run this plan's fw command")
    if not receipt.get("admitted") or receipt.get("preflight_only"):
        raise Refused("the job was never launched; run a new attempt or release the plan")
    if "finished" not in receipt:
        raise Refused("the job has not finished")
    if receipt.get("inputs_sha256") != frozen_sha:
        raise Refused("the job's inputs_sha256 is not this plan's frozen-inputs.json")
    run_ok = (receipt.get("exit") == 0 and not receipt.get("stopped_for_resource_limit") and "error" not in receipt
              and receipt.get("final_inputs_sha256") == frozen_sha and log_path.is_file())
    by_path = {take["path"]: take for take in plan["takes"]}
    records, noise = {}, 0
    if run_ok:
        for number, line in enumerate(log_path.read_text(encoding="utf-8").splitlines(), 1):
            if not line.startswith("{"):
                noise += 1
                continue
            try:
                record = json.loads(line)
            except ValueError:
                raise Refused(f"command.log line {number} is not a JSON record") from None
            if record.get("schema_version") != FW_RECORD or record.get("total") != len(plan["takes"]):
                raise Refused(f"command.log line {number} is not a {FW_RECORD} record of this plan")
            if record.get("input") not in by_path:
                raise Refused(f"command.log line {number} names an input outside {args.plan}")
            if record["input"] in records:
                raise Refused(f"command.log names {Path(record['input']).name} twice")
            index = record.get("index")
            if type(index) is not int or not 0 <= index < len(plan["takes"]) \
                    or plan["takes"][index]["path"] != record["input"]:
                raise Refused(f"command.log line {number} index does not match its inputs.txt line")
            records[record["input"]] = record
    processed, failed, rows = [], [], []
    for path, take in by_path.items():
        record = records.get(path)
        local = Path(path)
        if not run_ok:
            why = [f"run did not complete (exit {receipt.get('exit')!r}"
                   + (", resource stop" if receipt.get("stopped_for_resource_limit") else "")
                   + (", guard error" if "error" in receipt else "") + ")"]
        elif record is None:
            why = ["no fw record"]
        else:
            why = record_problems(record, plan)
        if not local.is_file() or sha_file(local) != take["sha256"]:
            why.append("WAV changed or missing since the plan")
        brief = {key: take[key] for key in ("cue", "name", "sha256", "pcm_sha256")}
        if why:
            failed.append(dict(brief, reasons=why))
            continue
        result = record["report"]["result"]
        processed.append(brief)
        rows.append(dict(brief, seconds=take["seconds"], transcript=result["transcript"],
                         alignment=align(script[take["cue"]], result["segments"]),
                         word_timestamps=str(result.get("raw_output", {}).get("word_timestamps")),
                         timing="provisional DTW word marks; not fine sync, phrase timings or listening",
                         listening=False, fine_sync=False))
    name = next_name(lane, "ingest")
    folder = lane / name
    folder.mkdir()
    dump_new(folder / "transcripts.json", {"schema": SCHEMA, "kind": "transcripts", "plan": args.plan, "rows": rows,
                                           "limits": LIMITS})
    dump_new(folder / "ingest.json", {
        "schema": SCHEMA, "kind": "ingest", "ingest": name, "plan": args.plan, "at": utc(), "job": job.name,
        "resource_receipt_sha256": sha_file(receipt_path),
        "command_log_sha256": sha_file(log_path) if log_path.is_file() else None,
        "frozen_inputs_sha256": frozen_sha, "run_complete": run_ok, "non_json_log_lines": noise,
        "transcripts_sha256": sha_file(folder / "transcripts.json"),
        "processed": processed, "failed": failed, "limits": LIMITS})
    print(json.dumps({"ingest": name, "plan": args.plan, "run_complete": run_ok, "processed": len(processed),
                      "failed": [f"{f['cue']} {f['name']}: {'; '.join(f['reasons'])}" for f in failed],
                      "differences": {f"{r['cue']} {r['name']}": len(r["alignment"]["differences"]) for r in rows}},
                     indent=1, ensure_ascii=False))
    return 0


def cmd_release(args):
    lane = Path(args.lane)
    state = lane_state(lane)
    if args.plan not in state["plans"] or args.plan in state["closed"]:
        raise Refused(f"{args.plan!r} is not an open plan of this lane")
    if not args.reason.strip():
        raise Refused("a release needs a reason")
    name = next_name(lane, "release", ".json")
    dump_new(lane / name, {"schema": SCHEMA, "kind": "release", "plan": args.plan, "at": utc(),
                           "reason": args.reason.strip(), "processed": [], "limits": LIMITS})
    print(json.dumps({"release": name, "plan": args.plan}))
    return 0


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=("plan", "status", "ingest", "release"))
    parser.add_argument("--lane", required=True, help="root's recognition lane folder (created by the first plan)")
    parser.add_argument("--takes", default=str(REPO / TAKES), help="retained Frederick take folder")
    parser.add_argument("--crate", default=str(REPO / CRATE), help="stage05 crate (pinned; drift exits 3)")
    parser.add_argument("--fw", default=str(Path.home() / FW_HOME))
    parser.add_argument("--model", default=str(Path.home() / MODEL_HOME))
    parser.add_argument("--other-voice", action="append", help="retained Oliver take folder (repeatable)")
    parser.add_argument("--plan", help="plan-NN for ingest or release")
    parser.add_argument("--job", help="root's guard job folder for ingest")
    parser.add_argument("--reason", default="", help="why a plan is released unrun")
    args = parser.parse_args(argv)
    if args.command in ("ingest", "release") and not args.plan:
        parser.error(f"{args.command} needs --plan")
    if args.command == "ingest" and not args.job:
        parser.error("ingest needs --job")
    try:
        if args.command in ("plan", "status"):
            return cmd_plan(args)
        return cmd_ingest(args) if args.command == "ingest" else cmd_release(args)
    except Refused as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 2
    except PinMoved as error:
        print(f"PIN MOVED: {error}", file=sys.stderr)
        return 3


if __name__ == "__main__":
    sys.exit(main())
