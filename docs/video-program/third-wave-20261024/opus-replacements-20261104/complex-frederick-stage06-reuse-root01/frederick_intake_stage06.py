"""Complex Frederick take intake, stage06: the sealed resolution intake (8760be91) with stage06 pins and one transfer.

Run from the repository root with python3 -I -B. Every subcommand reads a retained Frederick take folder (default
EC/frederick-stage06-01) against the pinned combined-repair-stage06 script, units and map.

  transfer --out NEW   root's explicit stage05 -> stage06 transfer. The sealed stage05 intake (8760be91, imported by
                       pin) re-checks every downloads receipt of the stage05 take folder against the stage05 crate,
                       exactly as it planned them. A cue transfers when its stage05 and stage06 rows are the same
                       text, its receipt is valid at today's bytes, and every take it lists was processed by a closed
                       stage05 plan at the receipts that plan froze. Writes NEW/transfer.json only; no stage05 file,
                       receipt, WAV, plan or ingest is rewritten, and a later change to any of them refuses the file.
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

plan and status need --transfer: a stage06 request for a transferred cue is refused (new stage06 requests are for
the remaining cues only), audio equal to a transferred take is a duplicate, and the transferred stage05 receipts
themselves are reported as transferred, not refused, if the stage05 take folder is scanned. DIR must be a new lane,
not the stage05 one.

Each downloads receipt rests on exactly one submission receipt that binds its request. When two or more bind it
(a failed attempt and its retry), root may author one CUE-submission-resolutionNN.json that names, at their current
hashes, the request, the downloads receipt, the selected submission with its submitted state, and each superseded
submission with the CUE-submission-observationNN.json that supersedes it; see check_resolution. Nothing else, and
no free text, decides which submission counts. The plan rests the takes on the resolution and every file it names.

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
import struct
import sys
import wave

SCHEMA = "complex-frederick-intake-stage06-v1"
REPO = Path.cwd()
HERE = Path(__file__).resolve()
EC = "narration_lab/third-wave-20261024/root/elevenlabs-complex-rebuild-20261104"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
TOPIC = "complex_analysis_visual"
CRATE = f"{EC}/combined-repair-stage06-root01/explainer_corpus/{TOPIC}"
TAKES = f"{EC}/frederick-stage06-01"
# Root's corrected stage06 candidate; the preserved stage05 side keeps its original map and pins.
MAP_REL = "docs/narration-direction-map-frederick-stage06.json"
# Exact candidate bindings. These checks do not establish independent or audiovisual acceptance.
PINS = {
    "script": "0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee",
    "units": "e8d2d91dcb0e6d444923185ceaf400cdaed7fa54f9843a80930ae2b81b4189e3",
    "map": "fa2b976b6e916c3a45652f55753744eb802e1bc95edbad4d29c912ce3722c967",
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
    "resolution": re.compile(r"(\d\d_\d+)-submission-resolution(\d\d)\.json"),
    "observation": re.compile(r"(\d\d_\d+)-submission-observation(\d\d)\.json"),
}
RESOLUTION_SCHEMA = "frederick-submission-resolution-v1"
RESOLUTION_KEYS = {"schema", "cue", "request", "downloads", "selected", "superseded", "by", "basis", "at", "listening"}
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
    """The stage06 script, units and map, by pin and by meaning. Returns (script rows, {role: path})."""
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


def instant(value):
    """An ISO 8601 time that names its zone, or None."""
    try:
        moment = datetime.datetime.fromisoformat(value)
    except (TypeError, ValueError):
        return None
    return moment if moment.tzinfo is not None else None


def check_resolution(takes_dir, cue, receipt, listing, digests, binders, groups):
    """Root's explicit choice among two or more submission receipts that bind one request (a failed attempt and its
    retry). digests holds the hashes of the request and downloads bytes already checked; binders the
    (name, submission, digest) of every submission receipt of the cue that binds the request.

    Returns None when the cue has no resolution of this request, so the one-submission rule stands. Otherwise
    returns (reasons, generations, rests). The resolution must be the cue's only one; name this request and this
    downloads receipt at those hashes; name as `selected` and `superseded` exactly the receipts that bind the
    request, at their hashes; give each superseded receipt a CUE-submission-observationNN.json, at its hash, whose
    `supersedes` begins with that receipt's name; and state the selected attempt's submitted state, which must agree
    with anything that receipt states itself. Its times must run in order. Every named file stays in the folder."""
    names = sorted(n for n, owner in groups["resolution"].items() if owner == cue)
    if not names:
        return None
    if len(names) > 1:
        return [f"{len(names)} submission resolutions for {cue} ({', '.join(names)}); expected at most one"], None, {}
    name = names[0]
    res, digest = read_receipt(takes_dir / name)
    rests = {"resolution": {"name": name, "sha256": digest}}
    if not isinstance(res, dict) or not isinstance(res.get("request"), dict):
        return [f"unreadable submission resolution {name}"], None, rests
    if res["request"].get("name") != listing.get("request"):
        return None  # a resolution of another request of this cue
    said = f"resolution {name}"
    if set(res) != RESOLUTION_KEYS or res.get("schema") != RESOLUTION_SCHEMA or res.get("cue") != cue \
            or not all(isinstance(res.get(k), str) and res[k].strip() for k in ("by", "basis")) \
            or res.get("listening") is not False or instant(res.get("at")) is None:
        return [f"{said}: not a {RESOLUTION_SCHEMA} for {cue} with exactly the keys {sorted(RESOLUTION_KEYS)}, "
                "non-empty by and basis, an ISO time with its zone, and listening false"], None, rests
    reasons, bound = [], {n: (s, d) for n, s, d in binders}

    def entry_of(value, keys, label):
        if isinstance(value, dict) and set(value) == keys and isinstance(value.get("name"), str):
            return value
        reasons.append(f"{said}: {label} must be an object with exactly the keys {sorted(keys)}")
        return None

    request = entry_of(res["request"], {"name", "sha256"}, "request")
    if request and request["sha256"] != digests["request"]:
        reasons.append(f"{said}: names {request['name']} at {str(request['sha256'])[:12]}, but it is "
                       f"{str(digests['request'])[:12]} now")
    downloads = entry_of(res["downloads"], {"name", "sha256"}, "downloads")
    if downloads and (downloads["name"], downloads["sha256"]) != (receipt, digests["downloads"]):
        reasons.append(f"{said}: names the downloads receipt {downloads['name']} at {str(downloads['sha256'])[:12]}, "
                       f"not {receipt} at {str(digests['downloads'])[:12]}")
    selected = entry_of(res["selected"], {"name", "sha256", "submitted", "generation_requests"}, "selected")
    superseded = res["superseded"] if isinstance(res["superseded"], list) and res["superseded"] else []
    if not superseded:
        reasons.append(f"{said}: superseded must list at least one submission receipt")
    olds = [entry_of(x, {"name", "sha256", "evidence"}, f"superseded entry {i}") for i, x in enumerate(superseded, 1)]
    named = [x["name"] for x in [selected, *olds] if x]
    for x in (x for x in [selected, *olds] if x):
        if x["name"] in bound and bound[x["name"]][1] != x["sha256"]:
            reasons.append(f"{said}: {x['name']} is {bound[x['name']][1][:12]} now, not the stated "
                           f"{str(x['sha256'])[:12]}")
    if len(set(named)) != len(named):
        reasons.append(f"{said}: names one submission receipt twice ({', '.join(named)})")
    if len(bound) < 2:
        reasons.append(f"{said}: only {len(bound)} submission receipt binds {listing.get('request')}; there is nothing "
                       "to resolve")
    elif set(named) != set(bound):
        reasons.append(f"{said}: the submission receipts binding {listing.get('request')} are {sorted(bound)}, but it "
                       f"names {named}")
    evidence = []
    for old in (x for x in olds if x):
        proof = entry_of(old["evidence"], {"name", "sha256"}, f"the evidence for {old['name']}")
        data, sha = (read_receipt(takes_dir / proof["name"]) if proof and groups["observation"].get(proof["name"]) == cue
                     else (None, None))
        if proof and (not isinstance(data, dict) or sha != proof["sha256"]):
            reasons.append(f"{said}: the evidence for {old['name']} must be a {cue}-submission-observationNN.json in "
                           f"the take folder at the stated hash; {proof['name']!r} is not")
        elif proof and not (isinstance(data.get("supersedes"), str) and (
                data["supersedes"] == old["name"] or data["supersedes"].startswith(old["name"] + " "))):
            reasons.append(f"{said}: {proof['name']} supersedes {data.get('supersedes')!r}, not {old['name']}")
        elif proof:
            evidence.append((old, proof, data))
    generations = selected.get("generation_requests") if selected else None
    if not selected or selected["submitted"] is not True or type(generations) is not int or generations < 1:
        reasons.append(f"{said}: the selected state must be submitted true with a positive integer "
                       "generation_requests")
        generations = None
    elif selected["name"] in bound:
        own, contrary = bound[selected["name"]][0], []
        if own.get("submitted", True) is not True:
            contrary.append("submitted")
        if "submitted_once" in own and (own["submitted_once"] is not True or generations != 1):
            contrary.append("submitted_once")
        if "generation_requests" in own and (type(own["generation_requests"]) is not int
                                             or own["generation_requests"] != generations):
            contrary.append("generation_requests")
        if own.get("cue", cue) != cue:
            contrary.append("cue")
        if contrary:
            reasons.append(f"{said}: the selected state contradicts {selected['name']}'s own {', '.join(contrary)}")
    if reasons:
        return reasons, None, rests
    times = {"resolution": instant(res["at"]), "downloads": instant(listing.get("at")),
             "selected": instant(bound[selected["name"]][0].get("at"))}
    late = [f"{k} has no ISO time with its zone" for k in ("downloads", "selected") if times[k] is None]
    if not late:
        if times["selected"] > times["downloads"]:
            late.append(f"{selected['name']} is later than {receipt}")
        if times["resolution"] < times["downloads"]:
            late.append(f"the resolution is earlier than {receipt}")
    for old, proof, data in evidence:
        tried, seen = instant(bound[old["name"]][0].get("at")), instant(data.get("at"))
        if tried is None or seen is None:
            late.append(f"{old['name']} or {proof['name']} has no ISO time with its zone")
        elif not tried <= seen <= times["resolution"]:
            late.append(f"{proof['name']} must come after {old['name']} and before the resolution")
    if late:
        return [f"{said}: times out of order: {'; '.join(late)}"], None, rests
    rests["submission"] = {"name": selected["name"], "sha256": selected["sha256"]}
    for number, (old, proof, _data) in enumerate(evidence, 1):
        rests[f"superseded{number:02d}"] = {"name": old["name"], "sha256": old["sha256"]}
        rests[f"superseded{number:02d}_evidence"] = {"name": proof["name"], "sha256": proof["sha256"]}
    return [], generations, rests


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
        reasons.append("cue is not in the stage06 script")
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
            reasons.append("stale cue text: the request text is not the stage06 script row")
        if not isinstance(text, str) or request.get("text_sha256") != sha_bytes(text.encode("utf-8")):
            reasons.append("request text_sha256 does not hash its text")
        if request.get("script_sha256") != PINS["script"]:
            reasons.append(f"script drift: request binds script {str(request.get('script_sha256'))[:12]}, "
                           f"not stage06 {PINS['script'][:12]}")
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
    digests = {"request": request_sha, "downloads": rests["downloads"]["sha256"]}
    resolution = check_resolution(takes_dir, cue, receipt, data, digests, submissions, groups)
    if resolution is not None:
        why, generations, chosen = resolution
        reasons += why
        rests.update(chosen)
    elif len(submissions) != 1:
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
    for owner, label in ((takes_dir, "take folder"), (crate.parents[1], "stage06 crate"), (HERE.parent, "tool bundle"),
                         (REPO / WORK, "guard work folder"), (REPO / STAGE05["lane"], "sealed stage05 lane"),
                         (REPO / STAGE05["takes"], "stage05 take folder")):
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
                f"cue {entry['cue']} has transferred stage05 takes; new stage06 requests are for the remaining cues "
                "only"]})
            continue
        dup = []
        for take in entry["takes"]:
            twin = transferred["audio"].get(take["sha256"]) or transferred["audio"].get(take["pcm_sha256"])
            if twin:
                dup.append(f"duplicate input: {take['name']} has the same audio as transferred stage05 {twin}")
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


# ----------------------------------------------------------------------------- stage05 -> stage06 transfer

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


def derive_transfer(script6, s5, crate5, takes5, lane5, other_folders):
    """What may transfer, derived only from the stage05 side's own files and the sealed stage05 checks."""
    script5, _ = s5.check_binding(crate5)
    state = s5.lane_state(lane5)
    still_open = sorted(set(state["plans"]) - set(state["closed"]))
    if still_open:
        raise Refused(f"the stage05 lane has open plans {still_open}; ingest or release them before the transfer")
    _inventory, other = s5.other_voice_hashes(other_folders)
    found = s5.scan(takes5, script5, other)  # the stage05 rules, at today's bytes
    held = {take["sha256"]: (entry, take) for entry in found["ready"] for take in entry["takes"]}
    refused = {row["receipt"]: row["reasons"] for row in found["refused"]}
    closed_by = {plan: closer for plan, closer in state["closed"].items()}
    per_cue, why = {}, {}
    for sha, brief in sorted(state["processed"].items(), key=lambda kv: (kv[1]["cue"], kv[1]["name"])):
        cue, plan_name = brief["cue"], next(p for p, c in closed_by.items() if c == brief["ingest"])
        row = next((t for t in state["plans"][plan_name]["takes"] if t["sha256"] == sha), None)
        problems = []
        if row is None or (row["cue"], row["name"]) != (cue, brief["name"]):
            problems.append(f"{brief['name']} is not a take of {plan_name}, which {brief['ingest']} closes")
        entry, take = held.get(sha, (None, None))
        if entry is None:
            receipts = sorted(r for r, reasons in refused.items() if r.startswith(cue + "-"))
            problems.append(f"{brief['name']}: no downloads receipt the sealed stage05 intake accepts today lists it"
                            + (f" ({'; '.join(r + ': ' + ', '.join(refused[r]) for r in receipts)})" if receipts else ""))
        elif row is not None and dict(sorted(entry["rests"].items())) != row["receipts"]:
            problems.append(f"{brief['name']}: the receipts it rests on today are not the ones {plan_name} froze")
        if row is not None and row["text_sha256"] != sha_bytes(script5[cue].encode("utf-8")):
            problems.append(f"{brief['name']}: {plan_name} bound other text than the stage05 row")
        if cue not in script6:
            problems.append(f"{cue} is not a stage06 cue")
        elif script6[cue] != script5[cue]:
            problems.append(f"{cue}: the stage06 row is not the stage05 text")
        if problems:
            why.setdefault(cue, []).extend(problems)
            continue
        slot = per_cue.setdefault(cue, {"entries": {}, "takes": []})
        slot["entries"][entry["receipt"]] = entry
        slot["takes"].append({"name": brief["name"], "path": rel(take["path"]), "sha256": sha,
                              "pcm_sha256": take["pcm_sha256"], "frames": take["frames"], "plan": plan_name,
                              "ingest": brief["ingest"]})
    cues = []
    for cue in [c for c in script5 if c in per_cue and c not in why]:
        slot = per_cue[cue]
        if len(slot["entries"]) != 1:
            why[cue] = [f"takes from {len(slot['entries'])} downloads receipts; transfer one receipt per cue"]
            continue
        entry = next(iter(slot["entries"].values()))
        if sorted(t["name"] for t in entry["takes"]) != sorted(t["name"] for t in slot["takes"]):
            why[cue] = [f"{entry['receipt']} lists takes that no closed stage05 plan processed"]
            continue
        cues.append({"cue": cue, "text_sha256": sha_bytes(script6[cue].encode("utf-8")), "same_text": True,
                     "receipts": dict(sorted(entry["rests"].items())), "takes": slot["takes"]})
    moved = {c["cue"] for c in cues}
    return {
        "stage05": {"intake": {"path": STAGE05["intake"][0], "sha256": STAGE05["intake"][1]},
                    "crate": rel(crate5), "crate_pins": dict(s5.PINS), "takes": rel(takes5), "lane": rel(lane5),
                    "lane_files": lane_files(lane5)},
        "stage06": {"crate_pins": dict(PINS)},
        "same_cue_ids_and_order": list(script5) == list(script6),
        "cues": cues,
        "not_transferred": {
            "takes_refused": {cue: list(dict.fromkeys(reasons)) for cue, reasons in sorted(why.items())},
            "changed_text": [{"cue": c, "stage05_text_sha256": sha_bytes(script5[c].encode("utf-8")),
                              "stage06_text_sha256": sha_bytes(script6[c].encode("utf-8"))}
                             for c in script5 if c in script6 and script5[c] != script6[c]],
            "only_in_stage05": [c for c in script5 if c not in script6],
            "only_in_stage06": [c for c in script6 if c not in script5],
            "need_stage06_requests": [c for c in script6 if c not in moved],
            "stage05_scan": {"refused": found["refused"], "awaiting_download": found["awaiting_download"],
                             "unreceipted_wavs": found["unreceipted_wavs"]},
        },
    }


def check_transfer(path, script6):
    """A transfer at today's bytes: every file it names unchanged, the stage05 lane closed as recorded, and each
    transferred cue the same text in both crates. Returns what plan, status and the bridge need."""
    if not path.is_file():
        raise Refused(f"no transfer file {path}")
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
    if new.get("crate_pins") != PINS:
        problems.append("the transfer names other stage06 pins than this tool's")
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
            problems.append(f"{cue}: not one cue with the same text in the stage05 and stage06 crates")
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


def cmd_transfer(args):
    if not (REPO / "tools/coherent_narration.py").is_file():
        raise Refused("run from the repository root")
    if not args.by.strip() or not args.basis.strip():
        raise Refused("a transfer is root's statement: it needs --by and --basis")
    out = Path(args.out)
    if out.exists():
        raise Refused("--out must be a new folder")
    script6, _ = check_binding(Path(args.crate))
    s5 = stage05_intake()
    body = derive_transfer(script6, s5, Path(args.stage05_crate), Path(args.stage05_takes), Path(args.stage05_lane),
                           args.other_voice or [REPO / folder for folder in s5.OTHER_VOICE])
    data = {"schema": TRANSFER_SCHEMA, "kind": "transfer", "at": utc(), "by": args.by.strip(),
            "basis": args.basis.strip(), "listening": False, "tool": {"name": HERE.name, "sha256": sha_file(HERE)},
            **body,
            "limits": "A source-bound bookkeeping transfer: same cue text, same retained receipts and audio. No "
                      "take is heard, selected, accepted or re-recognized here, and no stage05 file is rewritten."}
    out.mkdir(parents=True)
    dump_new(out / "transfer.json", data)
    check_transfer(out / "transfer.json", script6)
    gone = body["not_transferred"]
    print(json.dumps({"transfer": rel(out / "transfer.json"), "sha256": sha_file(out / "transfer.json"),
                      "cues": len(body["cues"]), "takes": sum(len(c["takes"]) for c in body["cues"]),
                      "same_cue_ids_and_order": body["same_cue_ids_and_order"],
                      "changed_text": [c["cue"] for c in gone["changed_text"]],
                      "takes_refused": gone["takes_refused"],
                      "need_stage06_requests": len(gone["need_stage06_requests"]),
                      "stage05_awaiting_download": gone["stage05_scan"]["awaiting_download"]}, indent=1))
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
                    "map_name": MAP_REL,
                    "voice": VOICE, "settings": SETTINGS,
                    "transfer": {"path": str(Path(args.transfer).resolve()), "sha256": ctx["transfer"]["sha256"]}},
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
    rows.append(("transfer", Path(args.transfer), ctx["transfer"]["sha256"]))
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
    parser.add_argument("command", choices=("transfer", "plan", "status", "ingest", "release"))
    parser.add_argument("--lane", help="root's NEW stage06 recognition lane folder (created by the first plan)")
    parser.add_argument("--takes", default=str(REPO / TAKES), help="retained Frederick take folder")
    parser.add_argument("--crate", default=str(REPO / CRATE), help="stage06 crate (pinned; drift exits 3)")
    parser.add_argument("--transfer", help="root's transfer.json (plan and status)")
    parser.add_argument("--out", help="a NEW folder for transfer")
    parser.add_argument("--by", default="", help="who makes the transfer")
    parser.add_argument("--basis", default="", help="why the transfer is made")
    parser.add_argument("--stage05-crate", default=str(REPO / STAGE05["crate"]))
    parser.add_argument("--stage05-takes", default=str(REPO / STAGE05["takes"]))
    parser.add_argument("--stage05-lane", default=str(REPO / STAGE05["lane"]))
    parser.add_argument("--fw", default=str(Path.home() / FW_HOME))
    parser.add_argument("--model", default=str(Path.home() / MODEL_HOME))
    parser.add_argument("--other-voice", action="append", help="retained Oliver take folder (repeatable)")
    parser.add_argument("--plan", help="plan-NN for ingest or release")
    parser.add_argument("--job", help="root's guard job folder for ingest")
    parser.add_argument("--reason", default="", help="why a plan is released unrun")
    args = parser.parse_args(argv)
    if args.command != "transfer" and not args.lane:
        parser.error(f"{args.command} needs --lane")
    if args.command in ("ingest", "release") and not args.plan:
        parser.error(f"{args.command} needs --plan")
    if args.command == "ingest" and not args.job:
        parser.error("ingest needs --job")
    if args.command in ("plan", "status") and not args.transfer:
        parser.error(f"{args.command} needs --transfer")
    if args.command == "transfer" and not args.out:
        parser.error("transfer needs --out")
    try:
        if args.command == "transfer":
            return cmd_transfer(args)
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
