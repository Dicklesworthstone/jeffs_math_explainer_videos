"""Complex Frederick take intake, stage07: the stage06 intake (a28166cb) with stage07 pins and root's 09_5 repair takes.

Run from the repository root with python3 -I -B. Stage07 is stage06 with one cue reworded, so one cue needs new takes.
Root submitted that cue once and kept three receipts in EC/remote-ts1-01/cauchy-cue-repair01. They predate this script
and have their own shape, so they are read as they are and never rewritten or re-issued: see check_repair.

  tree    --out NEW    root's new native tree: the stage06 workspace's bytes with the five sealed repair files over
                       them. Writes NEW/workspace/ and NEW/tree-receipt.json; the stage06 tree is only read.
  plan    --lane DIR   validate root's receipts and both takes, then write DIR/plan-NN/ (plan.json, inputs.txt,
                       run-asr.py, frozen-inputs.json) for takes no ingest has processed and no open plan holds. If
                       nothing is new it writes nothing.
  ingest  --lane DIR --plan plan-NN --job JOBDIR
                       bind root's guard job (resource-receipt.json, command.log) to that plan, re-hash every WAV,
                       and write DIR/ingest-NN/ (ingest.json, transcripts.json). The ingest closes the plan:
                       WAVs with an ok record become processed; every other WAV becomes eligible again.
  release --lane DIR --plan plan-NN --reason TEXT
                       close a plan that will not be run, so its WAVs become eligible again.
  status  --lane DIR   print what plan would do now; writes nothing.

The other 104 cues keep their takes: the bridge reads them from the sealed stage06 lane and, through root's transfer02,
from the sealed stage05 lane, each take by the hash of the text it was recorded from. DIR must be a new lane.

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
import importlib.util
import io
import json
from pathlib import Path
import re
import shutil
import struct
import sys
import wave

SCHEMA = "complex-frederick-intake-stage07-v1"
REPO = Path.cwd()
HERE = Path(__file__).resolve()
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
TOPIC = "complex_analysis_visual"
NATIVE = f"{EC}/native-stage07-root01"  # root's new tree, written by the tree subcommand
CRATE = f"{NATIVE}/workspace/explainer_corpus/{TOPIC}"
TAKES = f"{EC}/remote-ts1-01/cauchy-cue-repair01"
# Stage07 is stage06 with cue 09_5 and its card corrected (complex-cauchy-wording-repair01, seal eed378da).
MAP_REL = "docs/narration-direction-map-frederick-stage07.json"
# Exact candidate bindings. These checks do not establish independent or audiovisual acceptance.
PINS = {
    "script": "946a9c07a462eb066fc8d20c6a092d5b7564d38752d26177bdbbcf15c6027cdc",
    "units": "c937fca1f0d7507117462600f0f478472bd2b8dab93075aba6494bd99ce81141",
    "map": "4db4c4d5a433f3212e6af739d71625dd0931ac86bc68c4cfc859212925faff77",
}
# The sealed stage05 side. Its intake is imported by pin and runs its own checks on its own crate; nothing here
# rewrites a stage05 file.
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
STAGE05 = {
    "intake": (f"{P}/complex-frederick-intake-resolution01/frederick_intake.py",
               "8760be91d1df2207faddb4bacca173d7351c14ebdd50e86e4ac295eff7fe4d32"),
    "crate": f"{EC}/combined-repair-stage05/explainer_corpus/{TOPIC}",
    "takes": f"{EC}/frederick-stage05-01",
    "lane": f"{EC}/frederick-stage05-recognition01",
}
TRANSFER_SCHEMA = "complex-frederick-stage05-to-stage06-transfer-v1"
# The sealed stage06 side: its lane, root's transfer02 and root's decisions are read and never rewritten.
STAGE06 = {
    "pins": {"script": "0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee",
             "units": "e8d2d91dcb0e6d444923185ceaf400cdaed7fa54f9843a80930ae2b81b4189e3",
             "map": "fa2b976b6e916c3a45652f55753744eb802e1bc95edbad4d29c912ce3722c967"},
    "lane": f"{EC}/recognition-stage06-root01",
    "transfer": (f"{EC}/stage06-runtime-transfer02/transfer.json",
                 "d28701efddb62730a3c447e3c7eb3a77579007667632f09a587af80b405e9a54"),
    "selection": (f"{EC}/stage06-timing-decisions01/selection01.json",
                  "40641da8a02e9275b8de64ac5484ef4d9c34481a06a745e52280bc5b6a323511"),
    "answers": (f"{EC}/stage06-timing-decisions01/answers06.json",
                "a0ceb84607deb65632002814f9d299e1ac1bb610dc58dc7dc05bc290fee7b962"),
    "workspace": f"{EC}/native-stage06-root03/workspace",
    "narration": f"{EC}/production-audio-stage06-01/native-narration01",
}
TAKE_ASR = {"helper": "mini_take_asr01.py", "helper_sha256": "edd34a1ae9852d9f18c66d23424bbd6583ad2c21d91004de61b72566e463c091",
            "job": r"complex-cauchy-take-asr\d\d", "fw_name": "fw-0.11.0", "model_name": "ggml-large-v3-turbo.bin",
            # Root's fetched evidence (Mails 5671, 5683). The guard hashes inputs only, so these name the results.
            "inputs_sha256": "fae416a681b9c17539f4a1fa172813d9ce89a6afeab78564ad12968a644fe213",
            "results": {"resource-receipt.json": "08d231f044a8977f4d4a91e45b2bceab4d84dd4dae1d12970e81d459693e93b8",
                        "take-comparison.json": "b3873b2e5bd634a3102a3da5744889f3843619aac928879cd1b0b4a1fb984c61",
                        "take01-full-large.json": "e8543ba1d7d037b91b191616433b87e902ffe2bfab5296a4d74044b74cc034bd",
                        "take02-full-large.json": "087c954ee8de13590bde5b29d0f00b38cf93919c3159a47491261d03799439c4"}}
# Root's one submission of the corrected cue, and the five sealed files that make the stage07 tree.
REPAIR = {
    "cue": "09_5",
    "request": ("request01.json", "e9b0aef22537f507fcf968b42065c442f536cd82e0c10a76344c9771e2f9a84a"),
    "submission": ("submission01.json", "cf0b5c049f17abe6178507b808b1d424f8c7b0f31127e9f8390113ed64b8619d"),
    "downloads": ("download-completion01.json", "57ec25437d212f5197db6e106aa4782cf884c5cd9d26ec3ee2ef7bede75d8f9f"),
    "takes": ("09_5-take01.wav", "09_5-take02.wav"),
    "staged": f"{P}/complex-cauchy-wording-repair01/staged/explainer_corpus/{TOPIC}",
    "files": {"script.tsv": ("0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee",
                             PINS["script"]),
              "narration_units.json": ("e8d2d91dcb0e6d444923185ceaf400cdaed7fa54f9843a80930ae2b81b4189e3",
                                       PINS["units"]),
              "docs/storyboard.md": ("9ea68add5ac48474ba430ec14174df504fcdc5250efd29010cbbf9e60e571c9e",
                                     "ad1ccd44893116b9acf01999e2a919a28233c57c775368b927d2b6387f3880bc"),
              "src/chapters/boundary.rs": ("69e41733fd0e4d4e24f4ca1eb59cdf8c718ce07a8507e75a4e0a469331432c64",
                                           "1ebe0331ba47144b0968dc553fc2237139d10f2d8f26a207cd0b12f3c5dfdca7"),
              MAP_REL: (None, PINS["map"])},
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
NAMES = {"wav": re.compile(r"(\d\d_\d+)-take(\d\d)\.wav")}
GENERATION = ("Root generated this take once from this exact cue text while the live script was still stage06; the "
              "request binds that prior script. Stage07 adopted the same text afterwards. No receipt says the stage07 "
              "script was live at generation, and this record does not either.")
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
    """The stage07 script, units and map, by pin and by meaning. Returns (script rows, {role: path})."""
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
        why.append(f"take name {name!r} is not a {cue}-takeNN.wav")
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
    if rec["bytes"] != take.get("bytes"):
        why.append(f"{name} has {rec['bytes']} bytes, the receipt says {take.get('bytes')}")
    if info["frames"] != take.get("frames"):
        why.append(f"{name} has {info['frames']} frames, the receipt says {take.get('frames')}")
    format_said = (take.get("sample_rate"), take.get("channels"), take.get("sample_width"))
    if format_said != (info["sample_rate"], info["channels"], info["sample_bits"] // 8):
        why.append(f"{name} is not the format the receipt says: {format_said}")
    seconds = take.get("duration")
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


def instant(value):
    """An ISO 8601 time that names its zone, or None."""
    try:
        moment = datetime.datetime.fromisoformat(value)
    except (TypeError, ValueError):
        return None
    return moment if moment.tzinfo is not None else None


def check_repair(takes_dir, script, other):
    """Root's three receipts for the corrected cue, read as they are. Their shape is not the stage06 receipts': the
    request binds the prior script, the submission binds the text by hash, and the download record has no cue field.
    Returns reasons, take records and the receipts they rest on. A receipt at another hash is a moved pin."""
    cue, reasons, takes, read, rests = REPAIR["cue"], [], [], {}, {}
    for kind in ("request", "submission", "downloads"):
        name, pin = REPAIR[kind]
        read[kind], digest = read_receipt(takes_dir / name)
        if digest != pin:
            raise PinMoved(f"root's {kind} receipt {name} is {str(digest)[:12]}, not the pinned {pin[:12]}")
        rests[kind] = {"name": name, "sha256": digest}
    request, submission, listing = read["request"], read["submission"], read["downloads"]
    text = request.get("text")
    if request.get("cue") != cue or cue not in script:
        reasons.append(f"request cue {request.get('cue')!r} is not {cue} in the stage07 script")
    elif text != script[cue]:
        reasons.append("stale cue text: the request text is not the stage07 script row")
    if not isinstance(text, str) or request.get("text_sha256") != sha_bytes(text.encode("utf-8")):
        reasons.append("request text_sha256 does not hash its text")
    if request.get("prior_script_sha256") != STAGE06["pins"]["script"] or "script_sha256" in request:
        reasons.append("the request must bind the stage06 script as its prior script, and claim no other")
    if request.get("voice") != VOICE:
        reasons.append(f"mixed voice: request voice {request.get('voice')!r}")
    for key, want in SETTINGS.items():
        if request.get(key) != want or type(request.get(key)) is bool:
            reasons.append(f"request {key} {request.get(key)!r}, expected {want!r}")
    if submission.get("cue") != cue or submission.get("text_sha256") != request.get("text_sha256"):
        reasons.append("the submission does not name this cue and text")
    if submission.get("submission_clicked_once") is not True or submission.get("do_not_duplicate") is not True:
        reasons.append("the submission is not one click, marked do-not-duplicate")
    if listing.get("request") != REPAIR["request"][0] or listing.get("submission") != REPAIR["submission"][0]:
        reasons.append("the download record names other receipts")
    if listing.get("selected_take") is not None or any(listing.get(key) is not False
                                                       for key in ("asr_review", "listening", "fine_sync")):
        reasons.append("the download record claims a selection, recognition review, listening or fine sync")
    times = [instant(request.get("utc")), instant(submission.get("utc")), instant(listing.get("recorded_at"))]
    if None in times or sorted(times) != times:
        reasons.append("the request, submission and download times are missing or out of order")
    rows = listing.get("takes")
    if not isinstance(rows, list) or [Path(str(row.get("path", ""))).name for row in rows if isinstance(row, dict)] \
            != list(REPAIR["takes"]):
        reasons.append(f"incomplete take set: one generation must list exactly {list(REPAIR['takes'])}")
        return reasons, takes, rests
    generation = {"statement": GENERATION, "script_at_generation": request.get("prior_script_sha256"),
                  "text_sha256": request.get("text_sha256"), "request": rests["request"],
                  "submission": rests["submission"], "downloads": rests["downloads"]}
    seen = set()
    for row in rows:
        rec, why = check_take(takes_dir, cue, row, other)
        if rec["name"] in seen:
            why.append(f"duplicate input: {rec['name']} is listed twice")
        seen.add(rec["name"])
        reasons += why
        takes.append(dict(rec, generation=generation))
    return reasons, takes, rests


def scan(takes_dir, script, other):
    """Validate root's receipts and takes; reads only."""
    groups, loose = name_groups(takes_dir)
    reasons, takes, rests = check_repair(takes_dir, script, other)
    entries = [{"cue": REPAIR["cue"], "receipt": REPAIR["downloads"][0], "reasons": reasons, "takes": takes,
                "rests": rests}]
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
    return {
        "ready": [e for e in entries if not e["reasons"]],
        "refused": [{"cue": e["cue"], "receipt": e["receipt"], "reasons": list(dict.fromkeys(e["reasons"]))}
                    for e in entries if e["reasons"]],
        "awaiting_download": [],
        "not_submitted": [],
        "unreceipted_wavs": sorted(set(groups["wav"]) - named) + loose,
        "receipts": len(entries),
    }


def check_lane(lane, takes_dir, crate):
    if not re.fullmatch(r"[a-z0-9][a-z0-9_-]*", lane.name):
        raise Refused("lane folder name must be lowercase letters, digits, - or _ (it names guard jobs)")
    for owner, label in ((takes_dir, "take folder"), (crate.parents[1], "stage07 tree"), (HERE.parent, "tool bundle"),
                         (REPO / WORK, "guard work folder"), (REPO / STAGE05["lane"], "sealed stage05 lane"),
                         (REPO / STAGE05["takes"], "stage05 take folder"),
                         (REPO / STAGE06["lane"], "sealed stage06 lane")):
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


def classify(found, state, transferred, takes_dir):
    processed = state["processed"]
    processed_pcm = {take["pcm_sha256"]: take for take in processed.values()}
    flight = {take["sha256"]: (plan, take) for plan, data in state["plans"].items()
              if plan not in state["closed"] for take in data["takes"]}
    flight_pcm = {take["pcm_sha256"]: (plan, take) for plan, take in flight.values()}
    eligible, excluded, refused = [], [], []
    stage05_folder = takes_dir.resolve() == (REPO / transferred["takes"]).resolve()
    for row in found["refused"]:
        moved = transferred["cues"].get(row["cue"])
        if stage05_folder and moved and row["receipt"] == moved["receipts"]["downloads"]["name"] \
                and sha_file(takes_dir / row["receipt"]) == moved["receipts"]["downloads"]["sha256"]:
            excluded += [{"cue": row["cue"], "name": t["name"], "why": "transferred from stage05"}
                         for t in moved["takes"]]
        else:
            refused.append(row)
    for entry in found["ready"]:
        if entry["cue"] in transferred["cues"]:
            refused.append({"cue": entry["cue"], "receipt": entry["receipt"], "reasons": [
                f"cue {entry['cue']} has transferred stage05 takes; its retained takes stay in use"]})
            continue
        dup = []
        for take in entry["takes"]:
            twin = transferred["audio"].get(take["sha256"]) or transferred["audio"].get(take["pcm_sha256"])
            if twin:
                dup.append(f"duplicate input: {take['name']} has the same audio as retained {twin}")
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
    transfer = check_transfer(Path(args.transfer), script)
    for take in lane_state(REPO / STAGE06["lane"])["processed"].values():
        transfer["audio"][take["sha256"]] = transfer["audio"][take["pcm_sha256"]] = f"{take['cue']} {take['name']}"
    tools = check_tools(args)
    inventory, other = other_voice_hashes(args.other_voice or [REPO / folder for folder in OTHER_VOICE])
    found = scan(takes_dir, script, other)
    state = lane_state(lane)
    eligible, excluded, refused = classify(found, state, transfer, takes_dir)
    return {"takes_dir": takes_dir, "crate_files": crate_files, "tools": tools, "lane": lane, "script": script,
            "found": found, "state": state, "eligible": eligible, "excluded": excluded, "refused": refused,
            "other_voice": inventory, "transfer": transfer}


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
        "transferred_cues": len(ctx["transfer"]["cues"]),
        "unreceipted_wavs": found["unreceipted_wavs"],
        "open_plans": sorted(set(state["plans"]) - set(state["closed"])),
        "closed_plans": {plan: closer for plan, closer in sorted(state["closed"].items())},
        "processed_wavs": len(state["processed"]),
        "other_voice_inventory": ctx["other_voice"],
    }


# ----------------------------------------------------------------------------- the retained transfer

def rel(path):
    """Repository-relative when inside the repository, else absolute (scratch fixtures)."""
    path = Path(path).resolve()
    return str(path.relative_to(REPO.resolve())) if path.is_relative_to(REPO.resolve()) else str(path)


def stage05_intake():
    """The sealed stage05 intake, imported by pin. It keeps its own stage05 PINS and rules."""
    path, pin = STAGE05["intake"]
    if not (REPO / path).is_file() or sha_file(REPO / path) != pin:
        raise PinMoved(f"the sealed stage05 intake {path} is not {pin[:12]}")
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("frederick_intake_stage05_sealed", REPO / path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def lane_files(lane):
    """Every plan, ingest and release file of a lane, by name and hash."""
    names = [p for pattern in ("plan-[0-9][0-9]/*", "ingest-[0-9][0-9]/*", "release-[0-9][0-9].json")
             for p in lane.glob(pattern) if p.is_file()]
    return {str(p.relative_to(lane)): sha_file(p) for p in sorted(names)}


def check_transfer(path, script6):
    """Root's transfer02 at today's bytes: every file it names unchanged, the stage05 lane closed as recorded, and each
    transferred cue the same text in stage05 and stage07. Returns what plan, status and the bridge need."""
    if not path.is_file():
        raise Refused(f"no transfer file {path}")
    if sha_file(path) != STAGE06["transfer"][1]:
        raise PinMoved(f"{path.name} is not root's transfer02 at {STAGE06['transfer'][1][:12]}")
    data = load_json(path)
    s5 = stage05_intake()
    problems = []
    if data.get("schema") != TRANSFER_SCHEMA or data.get("kind") != "transfer":
        raise Refused(f"{path.name} is not a {TRANSFER_SCHEMA} file")
    if not str(data.get("by") or "").strip() or not str(data.get("basis") or "").strip() \
            or data.get("listening") is not False:
        problems.append("the transfer needs by and basis, and listening false")
    old, new = data.get("stage05") or {}, data.get("stage06") or {}
    if old.get("intake") != {"path": STAGE05["intake"][0], "sha256": STAGE05["intake"][1]} \
            or old.get("crate_pins") != s5.PINS:
        problems.append("the transfer was not derived by the sealed stage05 intake with the stage05 pins")
    if new.get("crate_pins") != STAGE06["pins"]:
        problems.append("the transfer names other stage06 pins than the sealed stage06 side's")
    if problems:
        raise Refused("transfer refused: " + "; ".join(problems))
    script5, _ = s5.check_binding(REPO / old["crate"])
    lane5, takes5 = REPO / old["lane"], REPO / old["takes"]
    if lane_files(lane5) != old["lane_files"]:
        problems.append("the stage05 lane gained, lost or changed a plan, ingest or release file since the transfer")
    audio, seen = {}, set()
    for c in data.get("cues") or []:
        cue = c["cue"]
        if cue in seen or cue not in script6 or script5.get(cue) != script6[cue] \
                or sha_bytes(script6[cue].encode("utf-8")) != c["text_sha256"]:
            problems.append(f"{cue}: not one cue with the same text in the stage05 and stage07 scripts")
        seen.add(cue)
        for kind, held in c["receipts"].items():
            receipt = takes5 / held["name"]
            if Path(held["name"]).name != held["name"] or not receipt.is_file() or sha_file(receipt) != held["sha256"]:
                problems.append(f"{cue}: its {kind} receipt {held['name']} changed or is missing")
        for take in c["takes"]:
            wav = REPO / take["path"]
            if wav.parent.resolve() != takes5.resolve() or not wav.is_file() or sha_file(wav) != take["sha256"]:
                problems.append(f"{cue}: {take['name']} changed, moved or is missing")
            audio[take["sha256"]] = audio[take["pcm_sha256"]] = f"{cue} {take['name']}"
    if problems:
        raise Refused("transfer refused:\n" + "\n".join(problems))
    return {"cues": {c["cue"]: c for c in data["cues"]}, "audio": audio, "takes": old["takes"], "lane": old["lane"],
            "sha256": sha_file(path), "data": data}


def cmd_tree(args):
    """Root's new native tree. The stage06 workspace's files are copied, leaving out the session-hook folder .ee, then
    the five sealed repair files go over them, the stage06 map staying beside the stage07 one."""
    out, source, staged = Path(args.out), REPO / STAGE06["workspace"], REPO / REPAIR["staged"]
    if not (REPO / "tools/coherent_narration.py").is_file():
        raise Refused("run from the repository root")
    if out.exists():
        raise Refused("--out must be a new folder")
    crate_old = source / "explainer_corpus" / TOPIC
    for name, (old, new) in REPAIR["files"].items():
        if old is not None and sha_file(crate_old / name) != old:
            raise PinMoved(f"stage06 {name} is not the pinned {old[:12]}")
        if sha_file(staged / name) != new:
            raise PinMoved(f"staged {name} is not the pinned {new[:12]}")
    workspace, crate = out / "workspace", out / "workspace/explainer_corpus" / TOPIC
    for path in sorted(p for p in source.rglob("*") if p.is_file() and ".ee" not in p.relative_to(source).parts):
        target = workspace / path.relative_to(source)
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(path, target)
    for name in REPAIR["files"]:
        target = crate / name
        target.parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(staged / name, target)
    check_binding(crate)
    rows = [{"path": rel(p), "sha256": sha_file(p)} for p in sorted(workspace.rglob("*")) if p.is_file()]
    dump_new(out / "tree-receipt.json", {
        "schema": SCHEMA, "kind": "tree", "at": utc(), "tool": {"name": HERE.name, "sha256": sha_file(HERE)},
        "from": STAGE06["workspace"], "repair_files": sorted(REPAIR["files"]), "left_out": [".ee"], "files": rows,
        "limits": "Source files only. Nothing is built, rendered, heard or accepted here."})
    print(json.dumps({"tree": rel(out), "files": len(rows), "receipt_sha256": sha_file(out / "tree-receipt.json")}))
    return 0


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
    write_plan(ctx, args, report)
    print(json.dumps(report, indent=1, ensure_ascii=False))
    return 0


def write_plan(ctx, args, report, adopted=None):
    """A plan and its frozen inputs. An adopted plan records the job that already ran and has nothing to launch."""
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
                                          "sample_rate", "channels", "sample_bits", "peak", "chunks", "source_name",
                                          "generation")}
        row["receipts"] = dict(sorted(take["receipts"].items()))
        row["text_sha256"] = sha_bytes(ctx["script"][take["cue"]].encode("utf-8"))
        takes.append(row)
    plan = {
        "schema": SCHEMA, "kind": "plan", "plan": name, "at": utc(), "lane": lane.name,
        "tool": {"name": HERE.name, "sha256": sha_file(HERE)},
        "binding": {"script_sha256": PINS["script"], "units_sha256": PINS["units"], "map_sha256": PINS["map"],
                    "map_name": MAP_REL,
                    "voice": VOICE, "settings": SETTINGS,
                    "transfer": {"path": str(Path(args.transfer).resolve()), "sha256": ctx["transfer"]["sha256"]}},
        "recognition": {"model_path": str(tools["model"]), "tool_sha256": dict(TOOL_PINS),
                        **({"adopted_job": adopted} if adopted else {"job_prefix": prefix, "fw_command": fw_command})},
        "takes": takes,
        **{key: value for key, value in summary(ctx).items() if key not in ("eligible_wavs",)},
        "limits": LIMITS,
    }
    dump_new(plan_dir / "plan.json", plan)
    if not adopted:
        write_new(plan_dir / "inputs.txt", "".join(take["path"] + "\n" for take in takes))
        write_new(plan_dir / "run-asr.py", RUN_TEMPLATE.format(
            plan=name, run_rel=f"{args.lane}/{name}/run-asr.py", prefix=prefix, repo=str(REPO.resolve()),
            plan_dir=str(plan_dir.resolve()), guard=str((REPO / GUARD).resolve()), fw=fw_command))
    # Pinned tools and checked bytes are frozen at the hash that was checked: a later change fails the guard.
    rows = [(role, tools[role], TOOL_PINS[role]) for role in ("guard", "guard_source", "fw", "model")]
    rows.append(("intake_tool", HERE, plan["tool"]["sha256"]))
    rows.append(("transfer", Path(args.transfer), ctx["transfer"]["sha256"]))
    rows += [(f"crate_{role}", path, PINS[role]) for role, path in ctx["crate_files"].items()]
    rows += [("receipt", takes_dir / name, digest) for name, digest in sorted(receipts.items())]
    rows += [("wav", Path(take["path"]), take["sha256"]) for take in takes]
    rows += [(role, plan_dir / file, None) for role, file in
             (("plan", "plan.json"), *(() if adopted else (("inputs", "inputs.txt"), ("run", "run-asr.py"))))]
    frozen = [{"path": str(Path(path).resolve()), "sha256": digest or sha_file(path), "role": role}
              for role, path, digest in rows]
    dump_new(plan_dir / "frozen-inputs.json", {"files": frozen})
    report["plan"] = name
    report["frozen_inputs_sha256"] = sha_file(plan_dir / "frozen-inputs.json")
    if not adopted:
        report["next"] = f"python3 -I -B {args.lane}/{name}/run-asr.py --attempt 1"
    return plan


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


def check_adopted(job, inputs, eligible, script):
    """Bronze's finished job, bound from its receipt to its manifest, then to the helper, the tools and root's takes."""
    if not re.fullmatch(TAKE_ASR["job"], job.name):
        raise Refused(f"{job.name!r} is not a two-take recognition job")
    moved = sorted(name for name, value in TAKE_ASR["results"].items()
                   if not (job / name).is_file() or sha_file(job / name) != value)
    if moved or sha_file(inputs) != TAKE_ASR["inputs_sha256"]:
        raise Refused(f"not the evidence root fetched: {moved or 'the manifest'}")
    receipt = load_json(job / "resource-receipt.json")
    command = receipt.get("command") or []
    if receipt.get("phase") != "asr" or not receipt.get("admitted") or receipt.get("preflight_only") \
            or "finished" not in receipt or receipt.get("exit") != 0 or receipt.get("stopped_for_resource_limit") \
            or "error" in receipt:
        raise Refused("the job did not finish a clean guarded run")
    if receipt.get("inputs_sha256") != sha_file(inputs) or receipt.get("final_inputs_sha256") != sha_file(inputs):
        raise Refused("the job's inputs hashes are not this manifest")
    if command[-2:] != ["--run", job.name[-2:]] or Path(command[-3]).name != TAKE_ASR["helper"]:
        raise Refused("the guard ran another command")
    frozen = {Path(row["path"]).name: row for row in load_json(inputs)["files"]}
    want = {TAKE_ASR["helper"]: TAKE_ASR["helper_sha256"], TAKE_ASR["fw_name"]: TOOL_PINS["fw"],
            TAKE_ASR["model_name"]: TOOL_PINS["model"], **{take["name"]: take["sha256"] for take in eligible},
            **{REPAIR[key][0]: REPAIR[key][1] for key in ("request", "submission", "downloads")}}
    moved = sorted(name for name, value in want.items() if frozen.get(name, {}).get("sha256") != value)
    if moved:
        raise Refused(f"the job's manifest does not bind {moved}")
    compared = load_json(job / "take-comparison.json")
    tops = {"cue": REPAIR["cue"], "selected_take": None, "listening": False, "fine_sync": False,
            "text_sha256": sha_bytes(script[REPAIR["cue"]].encode("utf-8")), "fw_sha256": TOOL_PINS["fw"],
            "model_sha256": TOOL_PINS["model"], "request_sha256": REPAIR["request"][1],
            "download_completion_sha256": REPAIR["downloads"][1]}
    moved = sorted(key for key, value in tops.items() if compared.get(key, "absent") != value)
    if moved:
        raise Refused(f"take-comparison.json differs at {moved}")
    if sorted((row.get("take"), row.get("sha256")) for row in compared["takes"]) \
            != sorted((take["name"], take["sha256"]) for take in eligible):
        raise Refused("take-comparison.json does not hold exactly root's two takes")
    plan = {"recognition": {"model_path": frozen[TAKE_ASR["model_name"]]["path"]}}
    records = {}
    for number, row in enumerate(compared["takes"], 1):
        take = next(t for t in eligible if t["name"] == row["take"])
        path = job / f"take{number:02d}-full-large.json"
        report = load_json(path)
        if Path(str(report.get("input_path"))).name != take["name"]:
            raise Refused(f"{path.name} recognized another input")
        problems = record_problems({"status": "ok", "report": report}, plan)
        if problems:
            raise Refused(f"{path.name}: {'; '.join(problems)}")
        records[take["path"]] = {"file": path.name, "sha256": sha_file(path), "report": report}
    return records


def cmd_adopt(args):
    """Open a new lane from Bronze's finished job, so the same takes are not recognized twice."""
    job, inputs = Path(args.job), Path(args.job_inputs)
    ctx = gather(args)
    if lane_state(ctx["lane"])["plans"]:
        raise Refused("adopt opens a new lane; this one already has a plan")
    if sorted(take["name"] for take in ctx["eligible"]) != sorted(REPAIR["takes"]):
        raise Refused("adopt needs exactly root's two eligible takes")
    records = check_adopted(job, inputs, ctx["eligible"], ctx["script"])
    report = summary(ctx)
    plan = write_plan(ctx, args, report, adopted=job.name)
    lane = ctx["lane"]
    rows, processed = [], []
    for take in plan["takes"]:
        result = records[take["path"]]["report"]["result"]
        brief = {key: take[key] for key in ("cue", "name", "sha256", "pcm_sha256")}
        processed.append(brief)
        rows.append(dict(brief, seconds=take["seconds"], transcript=result["transcript"],
                         alignment=align(ctx["script"][take["cue"]], result["segments"]),
                         word_timestamps=str(result.get("raw_output", {}).get("word_timestamps")),
                         timing="provisional DTW word marks; not fine sync, phrase timings or listening",
                         listening=False, fine_sync=False))
    folder = lane / next_name(lane, "ingest")
    folder.mkdir()
    dump_new(folder / "transcripts.json", {"schema": SCHEMA, "kind": "transcripts", "plan": plan["plan"], "rows": rows,
                                           "limits": LIMITS})
    dump_new(folder / "ingest.json", {
        "schema": SCHEMA, "kind": "ingest", "ingest": folder.name, "plan": plan["plan"], "at": utc(), "job": job.name,
        "adopted": True, "job_folder": rel(job), "job_inputs": rel(inputs), "job_inputs_sha256": sha_file(inputs),
        "take_comparison_sha256": sha_file(job / "take-comparison.json"),
        "records": {path: {"file": held["file"], "sha256": held["sha256"]} for path, held in records.items()},
        "resource_receipt_sha256": sha_file(job / "resource-receipt.json"),
        "command_log_sha256": sha_file(job / "command.log"),
        "frozen_inputs_sha256": sha_file(lane / plan["plan"] / "frozen-inputs.json"), "run_complete": True,
        "non_json_log_lines": sum(not line.startswith("{") for line in
                                  (job / "command.log").read_text(encoding="utf-8").splitlines()),
        "transcripts_sha256": sha_file(folder / "transcripts.json"),
        "processed": processed, "failed": [], "limits": LIMITS})
    print(json.dumps({"ingest": folder.name, "plan": plan["plan"], "adopted_job": job.name, "processed": len(processed),
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
    parser.add_argument("command", choices=("tree", "plan", "status", "adopt", "ingest", "release"))
    parser.add_argument("--lane", help="root's NEW stage07 recognition lane folder (created by the first plan)")
    parser.add_argument("--takes", default=str(REPO / TAKES), help="root's repair take folder")
    parser.add_argument("--crate", default=str(REPO / CRATE), help="stage07 crate (pinned; drift exits 3)")
    parser.add_argument("--transfer", default=str(REPO / STAGE06["transfer"][0]), help="root's transfer02 (pinned)")
    parser.add_argument("--out", help="a NEW folder for tree")
    parser.add_argument("--fw", default=str(Path.home() / FW_HOME))
    parser.add_argument("--model", default=str(Path.home() / MODEL_HOME))
    parser.add_argument("--other-voice", action="append", help="retained Oliver take folder (repeatable)")
    parser.add_argument("--plan", help="plan-NN for ingest or release")
    parser.add_argument("--job", help="root's guard job folder for ingest or adopt")
    parser.add_argument("--job-inputs", help="adopt: the frozen manifest that job's guard hashed")
    parser.add_argument("--reason", default="", help="why a plan is released unrun")
    args = parser.parse_args(argv)
    if args.command != "tree" and not args.lane:
        parser.error(f"{args.command} needs --lane")
    if args.command in ("ingest", "release") and not args.plan:
        parser.error(f"{args.command} needs --plan")
    if args.command == "ingest" and not args.job:
        parser.error("ingest needs --job")
    if args.command == "adopt" and not (args.job and args.job_inputs):
        parser.error("adopt needs --job and --job-inputs")
    if args.command == "tree" and not args.out:
        parser.error("tree needs --out")
    try:
        if args.command == "tree":
            return cmd_tree(args)
        if args.command in ("plan", "status"):
            return cmd_plan(args)
        if args.command == "adopt":
            return cmd_adopt(args)
        return cmd_ingest(args) if args.command == "ingest" else cmd_release(args)
    except Refused as error:
        print(f"REFUSED: {error}", file=sys.stderr)
        return 2
    except PinMoved as error:
        print(f"PIN MOVED: {error}", file=sys.stderr)
        return 3


if __name__ == "__main__":
    sys.exit(main())
