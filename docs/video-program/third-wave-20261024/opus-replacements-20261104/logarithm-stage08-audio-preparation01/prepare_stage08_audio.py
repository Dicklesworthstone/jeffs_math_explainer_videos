#!/usr/bin/env python3
"""Logarithm stage08 audio preparation (bead mx-video-program-program-87do.63), for root's guarded use.

Run from the repository root with `python3 -I -B`. Every subcommand writes only into a NEW
folder named by --out, refuses an existing one, and never edits a recording, the candidate,
canonical, a receipt or a review. Nothing here selects a take, synthesizes, renders, builds
or listens.

    report   Read-only. For all 162 retained recordings of combined-repair-stage08's script:
             recognition coverage, each sentence onset and each of the 33 phrase anchors as
             the recognizer placed them, every normalized recognizer disagreement, and the
             explicit questions that block a recording until root answers them.
             `--check OUT` reruns it and compares the bytes.
    decode   Root's run, under root's guard. Converts each SELECTED complete recording to
             24 kHz mono PCM16 with root's earlier ffmpeg arguments (no cut, time or pitch
             change) and writes the external receipt tools/coherent_narration.py reads.
    prepare  Root's run. From root's selection, root's answers and the decode receipt: the
             span manifest for coherent_narration.py assemble (whole recordings, sentence
             ends at the recognizer's onsets, zero inserted silence), the proposed native
             hold dispositions, a workspace holding byte copies of the assembler and the
             candidate's map, and the exact assemble command.
    timings  Root's run, after root's assemble. Verifies the assembly with the same tool,
             checks each assembled cue is the decoded PCM sample for sample, mirrors the
             kit's report and phrase-timing loader checks, and writes phrase-timings.json
             (the 33 anchors) into --out for root to install.

Recognizer times are estimates. whisper-cpp word segments usually start where the previous
word ended, so an onset can sit at the start of the pause before the word. Nothing here is a
measured boundary, a listening judgment, fine sync or acceptance.

Exit codes: 0 done or in step; 1 refused (each problem listed) or a stale report; 2 usage or
OUT refused; 3 a pinned input changed.
"""

import argparse
import collections
import difflib
import hashlib
import json
import os
import re
import shutil
import subprocess
import sys
import wave

P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage08-audio-preparation01"
EL = "narration_lab/third-wave-20261024/root/elevenlabs-logarithm-rebuild-20261104"
W = "narration_lab/third-wave-20261024/root/five-film-release-20261027"
TAKES = f"{EL}/ui-takes01"
CAND = f"{EL}/combined-repair-stage08/candidate"
TOPIC = "logarithm_mysteries"
CRATE_IN = f"explainer_corpus/{TOPIC}"  # inside a tree whose root holds tools/ and explainer_corpus/
CRATE = f"{CAND}/{CRATE_IN}"
SCRIPT = f"{CRATE}/script.tsv"
UNITS = f"{CRATE}/narration_units.json"
MAP_IN = f"{CRATE_IN}/docs/narration-direction-map-oliver15.json"
MAP = f"{CAND}/{MAP_IN}"
BP3 = f"{EL}/combined-repair-stage07/bound-plan03"
ANCHORS = f"{BP3}/phrase-anchors.json"
TEMPLATE = f"{BP3}/phrase-timings-unmeasured-template.json"
DISPOSITIONS = f"{BP3}/direction-dispositions-proposed.json"
HOLD_AUDIT = f"{BP3}/hold-audit.json"
INVENTORY = f"{EL}/full80-retained-inventory01.json"
INTEGRATION = f"{EL}/combined-repair-stage08/integration01.json"
ASSEMBLER = "tools/coherent_narration.py"
CACHE = "tools/narration_cache.py"
KIT = "crates/explainer_kit/src/narration.rs"

# The recognition runs. Only the large-v3-turbo runs place anchors; small.en corroborates.
RUNS = [
    {"name": "opening", "job": f"{W}/logarithm-opening-recognition01", "frozen": f"{EL}/opening-recognition01/frozen-inputs.json",
     "review": f"{EL}/opening-recognition01/recognition-comparison01.json", "role": "anchors"},
    {"name": "middle", "job": f"{W}/logarithm-middle-recognition01", "frozen": f"{EL}/middle-recognition01/frozen-inputs02.json",
     "review": None, "role": "anchors"},
    {"name": "turns", "job": f"{W}/logarithm-turns-recognition01", "frozen": f"{EL}/turns-recognition01/frozen-inputs.json",
     "review": f"{EL}/turns-recognition01/transcript-review01.json", "role": "anchors"},
    {"name": "remaining", "job": f"{W}/logarithm-remaining-recognition01", "frozen": f"{EL}/remaining-recognition01/frozen-inputs.json",
     "review": f"{EL}/remaining-recognition01/transcript-review01.json", "role": "anchors"},
    {"name": "targeted-small", "job": f"{W}/logarithm-targeted-small01", "frozen": f"{EL}/targeted-recognition-small01/inputs.json",
     "review": f"{EL}/targeted-recognition-small01/review01.json", "role": "corroborates"},
]
ANCHOR_MODEL = "ggml-large-v3-turbo.bin"
CORROBORATING_MODEL = "ggml-small.en.bin"

PINS = {
    INVENTORY: "d4ee2762f067ca7e1a78ec3d950dae5fce9ba7b8d39be6425a61127ae8eaf44c",
    INTEGRATION: "9ba49565e975a74b404876c4904778450821ee2b65387e96cd220c0fe0526987",
    SCRIPT: "b89eab600a6c99cdb737025d3cdb6d317597b42f20b4339479407c84ad2ea00e",
    UNITS: "0e27709b2db1dad6df0c0db49264ec9b69a021f7eae7323e62cd7f2ac853771c",
    MAP: "c94f0077f37974a12bba098dcf65d6afa4ed6082686ef41d15608897ca0e9cd4",
    ANCHORS: "a5930e689359c846e215863b394334c186b1bb0994047b9dd75e879fb6ea6970",
    TEMPLATE: "9169b6004507f6a1a735aa457beb28a5a71b2f9a4d759dc5721ff778b6ef6d28",
    DISPOSITIONS: "5372e362098144a3f701fbf032fd098e3b3dab53482ee843e4d577c86eafc2f0",
    HOLD_AUDIT: "4040d484c48e930348726fca61a6ee531cdd58f08211e6c6a237c2ad138f1c9a",
    f"{W}/logarithm-opening-recognition01/command.log": "4113e65f9c4e91a3aa5fbdacc6de6d7311fc9acdd9318ed5cda9d44eda46a8e8",
    f"{W}/logarithm-opening-recognition01/resource-receipt.json": "ead195c3635ccfa3514f4c401b52e32f2d7fe0a91faf158b6604d6625262cbbf",
    f"{W}/logarithm-middle-recognition01/command.log": "1c7fb420b88a6557d49e6ad395926b28c0199620679a80521180fb0e428957a8",
    f"{W}/logarithm-middle-recognition01/resource-receipt.json": "391fbcb37b7cecd24f162ed1b9a73e9a0c9e631da15cbd4696aca0e7795b9f88",
    f"{W}/logarithm-turns-recognition01/command.log": "8d20c0079989fd2d652a9e39c49301d6eb9092d9c959669f7696c1aa60557333",
    f"{W}/logarithm-turns-recognition01/resource-receipt.json": "d65dcf76ad9e639ecb3e355368017dd71873d5b90264f65e4c1276d73cd7bada",
    f"{W}/logarithm-remaining-recognition01/command.log": "3e479160bebb7daf0759caab2f7a538df6527cd53d34b3cf218c5a0295a442b7",
    f"{W}/logarithm-remaining-recognition01/resource-receipt.json": "ebcfe80f7d4eca6b5794ec30efe597f08d817b28deeac1fccb191a690ce34dbe",
    f"{W}/logarithm-targeted-small01/command.log": "3e9d2d265480ea7dc7ca83eb0efc102a8192c6a93508de9f89fc93e90985455d",
    f"{W}/logarithm-targeted-small01/resource-receipt.json": "ebfb8c0a98ef45479429e3b1e02468ca246fa9bd111559b0c7fd608407133426",
    f"{EL}/opening-recognition01/frozen-inputs.json": "4cf1425c517e9eb88a58c044eadf9ff0e88d4efb7a8af12b29eb1b13eda6e2f2",
    f"{EL}/middle-recognition01/frozen-inputs02.json": "0b7df805517c6abd810292f65a9d9c636c633a226312c26a2e165ce2e19334b5",
    f"{EL}/turns-recognition01/frozen-inputs.json": "a5ac5862f7a2ef505ec2ed576c4c47d2a33fc7d4a13e26c91d809dfff1714f10",
    f"{EL}/remaining-recognition01/frozen-inputs.json": "ec997c899b5ad46204329efb7a1826dbc2926eb5edee9aef3f7b3d3808e99881",
    f"{EL}/targeted-recognition-small01/inputs.json": "9483d891ae12d5e3d5b0c49413c20598a0d9fc338ebda501694f62865a1d6d04",
    f"{EL}/opening-recognition01/recognition-comparison01.json": "6e894b9aacad3e2ea26ae17f219bbcb6b505709197126586177a329a5be45d78",
    f"{EL}/turns-recognition01/transcript-review01.json": "edd60bdcd002f584690845921368cfa5da876718f5d7274b435040cce48ed07f",
    f"{EL}/remaining-recognition01/transcript-review01.json": "bf3727e27c28967c6442d3143470c21779f8951506f32e363ce902f85275c8ef",
    f"{EL}/targeted-recognition-small01/review01.json": "2c3779042bf55d728cc0c0c1ee42973ca38c1716e15c90e6175c38529ba2b35a",
    ASSEMBLER: "e0aa81f352cb2c847f53dacf1d37b7d1b6a44526102df7f79329278c278c48d0",
    CACHE: "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
    KIT: "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8",
}

# Root's two unresolved words (Mail 3651), carried as content questions on every take of the cue.
CONTENT = {
    "04_4": {"script_word": "base", "heard": "space",
             "note": "Mail 3651: 04_4 base/space unresolved. Both takes, in both models, transcribe 'space' where the script says 'base'."},
    "08_5": {"script_word": "halving", "heard": "having",
             "note": "Mail 3651: 08_5 halving/having unresolved. Both takes, in both models, transcribe 'Having' where the script says 'Halving'."},
}
MIN_BLOCK = 3         # a supported word lies in a run of at least this many agreeing words
RATE_OUT = 24000      # what tools/coherent_narration.py assembles at
FFMPEG_ARGS = ["-nostdin", "-n", "-v", "error"]  # root's earlier decode, word for word
FFMPEG_OUT = ["-ac", "1", "-ar", "24000", "-c:a", "pcm_s16le"]
ONES = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
TENS = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()


class Refused(Exception):
    pass


# ----------------------------------------------------------------------------- basics

def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(path):
    h = hashlib.sha256()
    with open(path, "rb") as f:
        while block := f.read(1 << 20):
            h.update(block)
    return h.hexdigest()


def dumps(value):
    return json.dumps(value, indent=1, ensure_ascii=False) + "\n"


def stale_pins(repo):
    out = []
    for rel, want in PINS.items():
        path = os.path.join(repo, rel)
        got = sha_file(path) if os.path.isfile(path) else None
        if got != want:
            out.append(f"{rel} is {got}, pinned {want}")
    return out


def read_script(repo):
    rows = {}
    for row in open(os.path.join(repo, SCRIPT), encoding="utf8").read().splitlines():
        if row.strip() and not row.startswith("#"):
            cue, text = row.split("\t", 1)
            rows[cue] = text
    return rows


def kit_sentences(text):
    """(start, end) character spans, the kit's rule: split after . ? ! before a space or the end."""
    out, start = [], 0
    for i, c in enumerate(text):
        if c in ".?!" and (i + 1 == len(text) or text[i + 1] == " "):
            if text[start : i + 1].strip():
                out.append((start, i + 1))
            start = i + 1
            while start < len(text) and text[start] == " ":
                start += 1
    if text[start:].strip():
        out.append((start, len(text)))
    return out


def number(n):
    if n < 20:
        return [ONES[n]]
    if n < 100:
        return [TENS[n // 10]] + ([ONES[n % 10]] if n % 10 else [])
    if n < 1000:
        return [ONES[n // 100], "hundred"] + (number(n % 100) if n % 100 else [])
    if n < 1000000 and n % 1000 == 0:
        return number(n // 1000) + ["thousand"]
    return [str(n)]


TOKEN = re.compile(r"[A-Za-z]+(?:['’][A-Za-z]+)*|[0-9]+(?:\.[0-9]+)?|π|%|/|[-−](?=[0-9])")


def tokens(text):
    """Normalized words with the character offset each came from. Digits become words, so the
    recognizer's notation ('1.01', '2π', '1/z') meets the script's words."""
    out = []
    for m in TOKEN.finditer(text):
        w = m.group().lower().replace("’", "'")
        if w == "π":
            out.append(("pi", m.start()))
        elif w == "%":
            out.append(("percent", m.start()))
        elif w == "/":
            out.append(("over", m.start()))
        elif w in ("-", "−"):
            out.append(("minus", m.start()))
        elif w[0].isdigit():
            whole, _, frac = w.partition(".")
            out.extend((x, m.start()) for x in number(int(whole)))
            if frac:
                out.append(("point", m.start()))
                out.extend((ONES[int(d)], m.start()) for d in frac)
        else:
            out.append((w, m.start()))
    return out


def take_rel(path):
    """An inventory or recognition path, absolute or relative, as its repository-relative form."""
    name = os.path.basename(path)
    if not path.replace(os.sep, "/").endswith(f"{TAKES}/{name}"):
        raise Refused(f"{name}: not a retained recording under {TAKES}")
    return f"{TAKES}/{name}"


def new_out(repo, out):
    full = os.path.abspath(os.path.join(repo, out))
    if os.path.exists(full):
        raise SystemExit(f"refused: OUT {out} exists; name a new folder")
    for guard in ("explainer_corpus", CAND, TAKES, W):
        g = os.path.abspath(os.path.join(repo, guard))
        if full == g or full.startswith(g + os.sep):
            raise SystemExit(f"refused: OUT {out} lies inside {guard}")
    os.makedirs(full)
    return full


def write_new(path, data):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "xb") as h:
        h.write(data if isinstance(data, bytes) else data.encode("utf8"))


def wav_info(path):
    with wave.open(path, "rb") as w:
        params = (w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getcomptype())
        frames = w.getnframes()
        pcm = w.readframes(frames)
    if len(pcm) != frames * params[0] * params[1]:
        raise Refused(f"{os.path.basename(path)}: truncated sample data")
    return params, frames, pcm


# ----------------------------------------------------------------------------- inputs

def load_inputs(repo):
    """Everything the subcommands read, each piece checked against the others."""
    problems = []
    script = read_script(repo)
    inv = json.load(open(os.path.join(repo, INVENTORY), encoding="utf8"))
    if inv["script_sha256"] != PINS[SCRIPT] or inv["cue_count"] != len(script) or inv.get("selection") is not None:
        problems.append("inventory does not describe this 80-cue script, or it claims a selection")
    takes = {}
    for t in inv["takes"]:
        rel = take_rel(t["wav"])
        if rel in takes:
            problems.append(f"{rel} listed twice in the inventory")
        if t["cue"] not in script:
            problems.append(f"{rel}: cue {t['cue']} is not in the script")
        elif t["text_sha256"] != sha_bytes(script[t["cue"]].encode()):
            problems.append(f"{rel}: requested text is not the script's {t['cue']}")
        receipt = f"{TAKES}/{t['cue']}-downloads01.json"
        if not os.path.isfile(os.path.join(repo, receipt)) or sha_file(os.path.join(repo, receipt)) != t["receipt_sha256"]:
            problems.append(f"{rel}: downloads receipt {receipt} is missing or changed")
        takes[rel] = dict(t, rel=rel, receipt=receipt)
    if len(takes) != inv["retained_file_count"] or {t["cue"] for t in takes.values()} != set(script):
        problems.append("inventory does not cover every cue, or its count is wrong")
    units = json.load(open(os.path.join(repo, UNITS), encoding="utf8"))
    if [u["line_ids"] for u in units["units"]] != [[c] for c in script] or any(u["id"] != "oliver_" + u["line_ids"][0] for u in units["units"]):
        problems.append("units are not one oliver_<cue> unit per cue in script order")
    if units["direction_contract"]["map"] != {"path": MAP_IN, "sha256": PINS[MAP]}:
        problems.append("units' direction contract does not name the pinned oliver15 map")
    anchors = json.load(open(os.path.join(repo, ANCHORS), encoding="utf8"))
    template = json.load(open(os.path.join(repo, TEMPLATE), encoding="utf8"))
    tmpl = {(c, m["byte"]) for c, e in template["cues"].items() for m in e["marks"]}
    if {(a["cue"], a["byte"]) for a in anchors["anchors"]} != tmpl or len(tmpl) != 33:
        problems.append("phrase anchors and the unmeasured template disagree")
    for a in anchors["anchors"]:
        b = script[a["cue"]].encode()
        if b[a["byte"] : a["byte_end"]].decode() != a["phrase"] or a["cue_text_sha256"] != sha_bytes(b):
            problems.append(f"anchor {a['cue']}@{a['byte']} no longer lands on '{a['phrase']}'")
    runs = []
    for run in RUNS:
        rc = json.load(open(os.path.join(repo, run["job"], "resource-receipt.json"), encoding="utf8"))
        cmd = rc.get("command", [])
        model = os.path.basename(cmd[cmd.index("--model") + 1]) if "--model" in cmd else None
        inputs_arg = cmd[cmd.index("--inputs") + 1] if "--inputs" in cmd else None
        want_model = ANCHOR_MODEL if run["role"] == "anchors" else CORROBORATING_MODEL
        if rc.get("exit") != 0 or rc.get("inputs_sha256") != rc.get("final_inputs_sha256") or rc.get("stopped_for_resource_limit"):
            problems.append(f"{run['name']}: the run did not finish cleanly with stable inputs")
        if model != want_model or not (inputs_arg or "").replace(os.sep, "/").endswith(run["frozen"]):
            problems.append(f"{run['name']}: model {model} or inputs {inputs_arg} is not the expected one")
        if "--split-on-word" not in cmd or cmd[cmd.index("--max-segment-length") + 1] != "1":
            problems.append(f"{run['name']}: not one word per segment")
        frozen = {}
        for f in json.load(open(os.path.join(repo, run["frozen"]), encoding="utf8"))["files"]:
            try:
                frozen[take_rel(f["path"])] = f["sha256"]
            except Refused:
                pass
        records = {}
        for line in open(os.path.join(repo, run["job"], "command.log"), encoding="utf8"):
            if not line.startswith("{"):
                continue
            r = json.loads(line)
            if r.get("schema_version") != "franken-whisper-batch-result-v1":
                continue
            rel = take_rel(r["input"])
            if r["status"] != "ok":
                problems.append(f"{run['name']}: {rel} status {r['status']}")
            if rel not in takes or frozen.get(rel) != takes[rel]["sha256"]:
                problems.append(f"{run['name']}: {rel} was not frozen with the inventory's hash")
            if rel in records:
                problems.append(f"{run['name']}: {rel} recognized twice")
            records[rel] = r["report"]["result"]
        runs.append(dict(run, model=model, records=records, log_sha256=PINS[f"{run['job']}/command.log"]))
    if problems:
        raise Refused("\n".join(problems))
    return {"script": script, "takes": takes, "anchors": anchors["anchors"], "runs": runs, "inventory": inv}


def recognition_for(inputs):
    """Each recording's one large-v3-turbo recognition, or its PCM twin's (marked)."""
    out, problems = {}, []
    for rel, t in inputs["takes"].items():
        own = [(run, run["records"][rel]) for run in inputs["runs"] if run["role"] == "anchors" and rel in run["records"]]
        if len(own) > 1:
            problems.append(f"{rel}: recognized by more than one anchor run")
        if own:
            out[rel] = {"run": own[0][0], "result": own[0][1], "via": None}
    for rel, t in inputs["takes"].items():
        if rel in out:
            continue
        twins = sorted(r for r, u in inputs["takes"].items() if r != rel and u["pcm_sha256"] == t["pcm_sha256"] and r in out and out[r]["via"] is None)
        if twins:
            out[rel] = dict(out[twins[0]], via=twins[0])
        else:
            problems.append(f"{rel}: no recognition and no recognized PCM twin")
    if problems:
        raise Refused("\n".join(problems))
    return out


# ----------------------------------------------------------------------------- anchors

def observed_words(result):
    words = []
    for si, seg in enumerate(result["segments"]):
        for k, (w, _) in enumerate(tokens(seg["text"])):
            words.append({"w": w, "start": seg["start_sec"], "end": seg["end_sec"], "seg": si, "k": k})
    return words


def align(text, result):
    canon = tokens(text)
    obs = observed_words(result)
    sm = difflib.SequenceMatcher(a=[w for w, _ in canon], b=[o["w"] for o in obs], autojunk=False)
    match, block, replaced = {}, {}, {}
    for op, i1, i2, j1, j2 in sm.get_opcodes():
        if op == "equal":
            for k in range(i2 - i1):
                match[i1 + k] = j1 + k
                block[i1 + k] = i2 - i1
        elif op == "replace" and i2 - i1 == j2 - j1:
            for k in range(i2 - i1):
                replaced[i1 + k] = j1 + k
    return canon, obs, match, block, replaced


def word_at(canon, char_offset):
    return next((i for i, (_, off) in enumerate(canon) if off == char_offset), None)


def place(i, canon, obs, match, block, replaced, seconds, after):
    """The recognizer's onset for canonical word i: supported, or a question with any candidate."""
    row = {"word": canon[i][0]}
    if i in match:
        o = obs[match[i]]
        row.update(asr_start_s=round(o["start"], 6), asr_end_s=round(o["end"], 6), run_length=block[i])
        j = match[i]
        if j > 0:
            row["previous_word_end_s"] = round(obs[j - 1]["end"], 6)
            row["abuts_previous_word"] = abs(obs[j - 1]["end"] - o["start"]) < 1e-6
        if block[i] < MIN_BLOCK:
            row["status"] = "short_agreement"
        elif o["k"] != 0:
            row["status"] = "inside_a_segment"
        elif not after < o["start"] < seconds:
            row["status"] = "out_of_order_or_range"
        else:
            row["status"] = "supported"
            row["sample_24k"] = round(o["start"] * RATE_OUT)
        return row
    row["status"] = "no_agreeing_word"
    if i in replaced and i - 1 in match and i + 1 in match and block[i - 1] >= MIN_BLOCK and block[i + 1] >= MIN_BLOCK:
        o = obs[replaced[i]]
        if o["k"] == 0 and after < o["start"] < seconds:
            row["candidate"] = {"heard": o["w"], "asr_start_s": round(o["start"], 6), "sample_24k": round(o["start"] * RATE_OUT),
                                "why": "the recognizer's one word in this slot, between agreeing runs on both sides"}
    return row


def anchors_for(text, cue, result, seconds, phrase_anchors):
    canon, obs, match, block, replaced = align(text, result)
    sentences = []
    after = 0.0
    for n, (a, _b) in enumerate(kit_sentences(text)):
        if n == 0:
            sentences.append({"index": 0, "word": canon[0][0], "status": "file_start", "sample_24k": 0})
            continue
        i = next(ix for ix, (_, off) in enumerate(canon) if off >= a)
        row = dict(index=n, char=a, **place(i, canon, obs, match, block, replaced, seconds, after))
        if row["status"] == "supported":
            after = row["asr_start_s"]
        sentences.append(row)
    marks = []
    after = 0.0
    btext = text.encode()
    for an in sorted((x for x in phrase_anchors if x["cue"] == cue), key=lambda x: x["byte"]):
        i = word_at(canon, len(btext[: an["byte"]].decode()))
        if i is None:
            marks.append({"byte": an["byte"], "phrase": an["phrase"], "status": "no_word_at_byte"})
            continue
        row = dict(byte=an["byte"], phrase=an["phrase"], **place(i, canon, obs, match, block, replaced, seconds, -1.0))
        if row["status"] == "supported":
            if row["asr_start_s"] < after:
                row["status"] = "out_of_order_or_range"
                row.pop("sample_24k", None)
            else:
                after = row["asr_start_s"]
        marks.append(row)
    return sentences, marks


def differences(text, result):
    a = [w for w, _ in tokens(text)]
    b = [o["w"] for o in observed_words(result)]
    return [{"op": op, "script": " ".join(a[i1:i2]), "heard": " ".join(b[j1:j2])}
            for op, i1, i2, j1, j2 in difflib.SequenceMatcher(a=a, b=b, autojunk=False).get_opcodes() if op != "equal"]


# ----------------------------------------------------------------------------- report

def build_report(repo):
    inputs = load_inputs(repo)
    recog = recognition_for(inputs)
    script = inputs["script"]
    recordings, questions, diffs = [], [], []
    for rel in sorted(inputs["takes"], key=lambda r: (list(script).index(inputs["takes"][r]["cue"]), r)):
        t = inputs["takes"][rel]
        data = open(os.path.join(repo, rel), "rb").read()
        if sha_bytes(data) != t["sha256"]:
            raise Refused(f"{rel} changed on disk")
        params, frames, pcm = wav_info(os.path.join(repo, rel))
        if params != (1, 2, 48000, "NONE") or frames != t["frames"] or sha_bytes(pcm) != t["pcm_sha256"] or frames % 2:
            raise Refused(f"{rel}: not the inventory's 48 kHz mono PCM16 with even frames")
        cue, seconds = t["cue"], frames / 48000
        r = recog[rel]
        sentences, marks = anchors_for(script[cue], cue, r["result"], seconds, inputs["anchors"])
        name = os.path.basename(rel)
        entry = {
            "cue": cue, "file": name, "sha256": t["sha256"], "pcm_sha256": t["pcm_sha256"], "frames_48k": frames,
            "seconds": seconds, "frames_24k_after_decode": frames // 2, "downloads_receipt": os.path.basename(t["receipt"]),
            "recognition": {"run": r["run"]["name"], "model": r["run"]["model"], "log_sha256": r["run"]["log_sha256"],
                            "recognized_file": os.path.basename(r["via"]) if r["via"] else name},
            "sentences": sentences, "phrase_anchors": marks,
        }
        recordings.append(entry)
        for run in inputs["runs"]:
            if rel in run["records"]:
                d = differences(script[cue], run["records"][rel])
                if d:
                    diffs.append({"file": name, "run": run["name"], "model": run["model"], "differences": d})
        for s in sentences:
            if s["status"] not in ("supported", "file_start"):
                questions.append({
                    "id": f"onset:{cue}:s{s['index']}:{name}", "kind": "sentence_onset", "cue": cue, "files": [name],
                    "detail": f"Sentence {s['index']} of {cue} starts with '{s['word']}'; the recognizer gives no supported onset ({s['status']}).",
                    "candidate": s.get("candidate"), "decisions": ["use_candidate", "sample"] if s.get("candidate") else ["sample"]})
        for m in marks:
            if m["status"] != "supported":
                questions.append({
                    "id": f"anchor:{cue}:b{m['byte']}:{name}", "kind": "phrase_anchor", "cue": cue, "files": [name],
                    "detail": f"Anchor '{m['phrase']}' at byte {m['byte']} has no supported onset ({m['status']}).",
                    "candidate": m.get("candidate"), "decisions": (["use_candidate"] if m.get("candidate") else []) + ["sample", "omit_mark"]})
        if r["via"]:
            questions.append({
                "id": f"provenance:{cue}:{name}", "kind": "provenance", "cue": cue, "files": [name],
                "detail": f"{name} was not recognized itself. Its PCM equals {os.path.basename(r['via'])}'s, whose recognition is used. "
                          f"The inventory warns that file count is not generation count for {cue}.",
                "candidate": None, "decisions": ["use_pcm_identity"]})
    for cue, c in CONTENT.items():
        names = [e["file"] for e in recordings if e["cue"] == cue]
        heard = sorted({(d["file"], d["model"]) for d in diffs if d["file"] in names
                        for x in d["differences"] if x["script"] == c["script_word"] and x["heard"] == c["heard"]})
        questions.append({
            "id": f"content:{cue}:{c['script_word']}", "kind": "content", "cue": cue, "files": names,
            "detail": c["note"] + f" Transcripts with '{c['heard']}': " + ", ".join(f"{f} ({m})" for f, m in heard) + ". ASR is not listening.",
            "candidate": None, "decisions": ["heard_as_script", "reject_take"]})
    questions.sort(key=lambda q: (list(script).index(q["cue"]), q["id"]))
    counts = collections.Counter()
    for e in recordings:
        for s in e["sentences"][1:]:
            counts["sentence_onsets"] += 1
            counts["sentence_onsets_supported"] += s["status"] == "supported"
            counts["sentence_onsets_abutting_previous_word"] += bool(s.get("abuts_previous_word")) and s["status"] == "supported"
        for m in e["phrase_anchors"]:
            counts["phrase_anchor_marks"] += 1
            counts["phrase_anchor_marks_supported"] += m["status"] == "supported"
    lengths = sorted(s["asr_end_s"] - s["asr_start_s"] for e in recordings for s in e["sentences"][1:] if s["status"] == "supported" and s.get("abuts_previous_word"))
    coverage = {
        "script_sha256": PINS[SCRIPT], "inventory_sha256": PINS[INVENTORY], "cues": len(script), "recordings": len(recordings),
        "runs": [{"name": r["name"], "model": r["model"], "role": r["role"], "records": len(r["records"]), "log_sha256": r["log_sha256"]} for r in inputs["runs"]],
        "recognized_by_pcm_identity": [e["file"] for e in recordings if e["recognition"]["recognized_file"] != e["file"]],
        "counts": dict(counts),
        "abutting_onset_segment_seconds": {"count": len(lengths), "median": round(lengths[len(lengths) // 2], 3) if lengths else None,
                                           "max": round(lengths[-1], 3) if lengths else None,
                                           "meaning": "how early an abutting onset can be: its word segment begins where the previous word ended"},
        "questions": len(questions), "blocking_files": sorted({f for q in questions for f in q["files"]}),
    }
    claim = ("Recognizer estimates only (whisper-cpp word segments). No take selected, heard, measured, cut or accepted. "
             "A 'supported' onset is a word the recognizer agrees on inside a run of at least 3 agreeing words, "
             "first in its segment, in range and in order.")
    files = {
        "anchors.json": dumps({"claim": claim, "min_agreeing_run": MIN_BLOCK, "recordings": recordings}),
        "questions.json": dumps({"claim": "Each question blocks the files it names until root answers it in an answers file. "
                                          "Answers are root's statements; this tool records them and checks only their form.",
                                 "questions": questions}),
        "answers-template.json": dumps({"schema": "logarithm-stage08-answers-v1", "script_sha256": PINS[SCRIPT],
                                        "answers": {q["id"]: {"decision": None, "by": None, "basis": None} for q in questions}}),
        "coverage.json": dumps(dict(coverage, claim=claim)),
        "asr-differences.json": dumps({"claim": "Normalized recognizer disagreements, every transcript. Diagnostic only; not speech evidence.",
                                       "transcripts": diffs}),
    }
    return files, questions, recordings, inputs


# ----------------------------------------------------------------------------- selection and answers

def load_selection(repo, path, inputs, chapter):
    sel = json.load(open(path, encoding="utf8"))
    problems = []
    if sel.get("schema") != "logarithm-stage08-selection-v1" or sel.get("script_sha256") != PINS[SCRIPT] or sel.get("inventory_sha256") != PINS[INVENTORY]:
        problems.append("selection is not logarithm-stage08-selection-v1 for this script and inventory")
    if not str(sel.get("by", "")).strip() or not str(sel.get("basis", "")).strip():
        problems.append("selection needs `by` and `basis`: a selection is root's decision")
    want = [c for c in inputs["script"] if chapter is None or c.startswith(chapter + "_")]
    if not want:
        problems.append(f"chapter {chapter} has no cues")
    if list(sel.get("takes", {})) != want:
        problems.append(f"selection must name exactly these cues in script order: {want[0]} … {want[-1]} ({len(want)})" if want else "no cues")
    chosen = {}
    for cue, name in sel.get("takes", {}).items():
        rel = f"{TAKES}/{name}"
        t = inputs["takes"].get(rel)
        if t is None or t["cue"] != cue:
            problems.append(f"{cue}: {name} is not a retained recording of {cue}")
            continue
        if sha_file(os.path.join(repo, rel)) != t["sha256"]:
            problems.append(f"{cue}: {name} changed on disk")
        chosen[cue] = t
    if problems:
        raise Refused("\n".join(problems))
    return sel, chosen


def check_answers(path, questions, chosen):
    data = json.load(open(path, encoding="utf8")) if path else {"schema": "logarithm-stage08-answers-v1", "script_sha256": PINS[SCRIPT], "answers": {}}
    problems = []
    if data.get("schema") != "logarithm-stage08-answers-v1" or data.get("script_sha256") != PINS[SCRIPT]:
        problems.append("answers are not logarithm-stage08-answers-v1 for this script")
    known = {q["id"]: q for q in questions}
    answers = data.get("answers", {})
    for qid in answers:
        if qid not in known:
            problems.append(f"answer to an unknown question {qid}: answers must name the report's questions")
    names = {os.path.basename(t["rel"]) for t in chosen.values()}
    applied = {}
    for q in questions:
        if not set(q["files"]) & names:
            continue
        a = answers.get(q["id"])
        if not a or a.get("decision") is None:
            problems.append(f"open question blocks the selection: {q['id']}: {q['detail']}")
            continue
        if a["decision"] not in q["decisions"]:
            problems.append(f"{q['id']}: decision {a['decision']!r} is not one of {q['decisions']}")
            continue
        if not str(a.get("by") or "").strip() or not str(a.get("basis") or "").strip():
            problems.append(f"{q['id']}: an answer needs `by` and `basis`")
            continue
        if a["decision"] == "sample" and (type(a.get("sample_24k")) is not int or a["sample_24k"] <= 0):
            problems.append(f"{q['id']}: decision 'sample' needs a positive integer sample_24k")
            continue
        if a["decision"] == "reject_take":
            problems.append(f"{q['id']}: root rejected {', '.join(sorted(set(q['files']) & names))}; select another recording")
            continue
        applied[q["id"]] = dict(a, kind=q["kind"])
    if problems:
        raise Refused("\n".join(problems))
    return data, applied


def resolved_points(entry, applied, frames24):
    """Sentence ends (24 kHz samples) and anchor marks (seconds) for one selected recording."""
    cue, name = entry["cue"], entry["file"]
    starts = []
    for s in entry["sentences"][1:]:
        qid = f"onset:{cue}:s{s['index']}:{name}"
        if s["status"] == "supported":
            starts.append((s["sample_24k"], "recognizer onset"))
        elif applied.get(qid, {}).get("decision") == "use_candidate":
            starts.append((s["candidate"]["sample_24k"], f"recognizer candidate, answered by {applied[qid]['by']}"))
        elif applied.get(qid, {}).get("decision") == "sample":
            starts.append((applied[qid]["sample_24k"], f"sample given by {applied[qid]['by']}"))
        else:
            raise Refused(f"{qid}: no onset")
    ends = [s for s, _ in starts] + [frames24]
    if any(b <= a for a, b in zip([0] + ends, ends)):
        raise Refused(f"{cue} {name}: sentence ends are not strictly increasing inside the recording: {ends}")
    marks = []
    for m in entry["phrase_anchors"]:
        qid = f"anchor:{cue}:b{m['byte']}:{name}"
        decision = applied.get(qid, {}).get("decision")
        if m["status"] == "supported":
            marks.append({"byte": m["byte"], "seconds": m["asr_start_s"], "basis": "recognizer onset"})
        elif decision == "use_candidate":
            marks.append({"byte": m["byte"], "seconds": m["candidate"]["asr_start_s"], "basis": f"recognizer candidate, answered by {applied[qid]['by']}"})
        elif decision == "sample":
            marks.append({"byte": m["byte"], "seconds": applied[qid]["sample_24k"] / RATE_OUT, "basis": f"sample given by {applied[qid]['by']}"})
        elif decision == "omit_mark":
            continue
        else:
            raise Refused(f"{qid}: no mark")
    return starts, ends, marks


# ----------------------------------------------------------------------------- decode

def cmd_decode(args):
    repo = args.repo
    inputs = load_inputs(repo)
    sel, chosen = load_selection(repo, args.selection, inputs, args.chapter)
    out = new_out(repo, args.out)
    version = subprocess.run([args.ffmpeg, "-version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    records, problems = [], []
    for cue, t in chosen.items():
        src = os.path.join(repo, t["rel"])
        dst_rel = os.path.relpath(os.path.join(out, "decoded", f"{cue}.wav"), os.path.abspath(repo)).replace(os.sep, "/")
        dst = os.path.join(repo, dst_rel)
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        if os.path.exists(dst):
            raise Refused(f"{dst_rel} exists")
        subprocess.run([args.ffmpeg, *FFMPEG_ARGS, "-i", src, *FFMPEG_OUT, dst], check=True, timeout=120)
        params, frames, _pcm = wav_info(dst)
        if params[:3] != (1, 2, RATE_OUT) or frames != t["frames"] // 2:
            problems.append(f"{cue}: decoded {params[:3]} {frames} frames; expected mono PCM16 24 kHz, {t['frames'] // 2} frames (kept, not used)")
        records.append({"cue": cue, "text": inputs["script"][cue], "source": t["rel"], "source_sha256": t["sha256"],
                        "source_pcm_sha256": t["pcm_sha256"], "source_frames": t["frames"], "source_rate": 48000,
                        "downloads_receipt": t["receipt"], "wav": dst_rel, "wav_sha256": sha_file(dst), "samples": frames, "rate": RATE_OUT})
    receipt = {"source": "ElevenLabs UI, Oliver", "tool": f"{HERE}/prepare_stage08_audio.py decode",
               "conversion": "Complete recording, 48 kHz to mono 24 kHz PCM16 by ffmpeg; no cuts, time or pitch processing.",
               "ffmpeg": {"version": version, "arguments": FFMPEG_ARGS + ["-i", "SOURCE"] + FFMPEG_OUT + ["TARGET"]},
               "selection_sha256": sha_file(args.selection), "chapter": args.chapter, "records": records,
               "claim": "Format conversion only. No take accepted, heard or timed here."}
    write_new(os.path.join(out, "decode-receipt.json"), dumps(receipt))
    if problems:
        raise Refused("\n".join(problems))
    print(f"decoded {len(records)} complete recordings into {args.out}; receipt decode-receipt.json")
    return 0


# ----------------------------------------------------------------------------- prepare

def cmd_prepare(args):
    repo = args.repo
    files, questions, recordings, inputs = build_report(repo)
    sel, chosen = load_selection(repo, args.selection, inputs, args.chapter)
    answers, applied = check_answers(args.answers, questions, chosen)
    receipt_rel = os.path.join(args.decoded, "decode-receipt.json").replace(os.sep, "/")
    receipt = json.load(open(os.path.join(repo, receipt_rel), encoding="utf8"))
    rows = {r["cue"]: r for r in receipt["records"]}
    problems = []
    if receipt.get("source") != "ElevenLabs UI, Oliver" or list(rows) != list(chosen) or receipt.get("selection_sha256") != sha_file(args.selection):
        problems.append("decode receipt is not this selection's")
    by_name = {(e["cue"], e["file"]): e for e in recordings}
    units, plan_cues, marks_plan = [], {}, {}
    for cue, t in chosen.items():
        row = rows.get(cue)
        if row is None:
            continue
        wav = os.path.join(repo, row["wav"])
        params, frames, _ = wav_info(wav)
        if (row["source"] != t["rel"] or row["source_sha256"] != t["sha256"] or row["text"] != inputs["script"][cue]
                or sha_file(wav) != row["wav_sha256"] or params[:3] != (1, 2, RATE_OUT) or frames != row["samples"] != None
                or frames != t["frames"] // 2):
            problems.append(f"{cue}: decoded file, its receipt row and the selected recording disagree")
            continue
        entry = by_name[(cue, os.path.basename(t["rel"]))]
        try:
            starts, ends, marks = resolved_points(entry, applied, frames)
        except Refused as e:
            problems.append(str(e))
            continue
        units.append({"id": f"oliver_{cue}", "wav": row["wav"], "sha256": row["wav_sha256"], "generation_receipt": receipt_rel,
                      "sentence_ends": ends, "gap_after_samples": [0] * len(ends)})
        plan_cues[cue] = {"recording": entry["file"], "recognition": entry["recognition"],
                          "sentence_starts_24k": [{"sample": s, "basis": b} for s, b in starts]}
        marks_plan[cue] = marks
    if problems:
        raise Refused("\n".join(problems))
    disp = json.load(open(os.path.join(repo, DISPOSITIONS), encoding="utf8"))["direction_dispositions"]
    disp = [d for d in disp if d["cue"] in chosen]
    for d in disp:
        if d["evidence"] != {"path": HOLD_AUDIT, "sha256": PINS[HOLD_AUDIT]} or d["kind"] != "native_visual_hold":
            raise Refused(f"disposition {d['override_index']} is not the pinned native hold proposal")
    out = new_out(repo, args.out)
    out_rel = os.path.relpath(out, os.path.abspath(repo)).replace(os.sep, "/")
    basis = ("Complete ElevenLabs Oliver recordings, converted to 24 kHz only. Sentence ends are the large-v3-turbo recognizer's "
             "word onsets (provisional; an onset can sit at the start of the pause before its word), except where root's "
             "answers say otherwise. No samples removed, no silence inserted, no gain, time or pitch processing.")
    spans = {"schema_version": 1, "units_sha256": PINS[UNITS], "units": units, "direction_dispositions": disp,
             "boundary_review": {"reviewer": args.reviewer, "basis": basis, "listening": "unreviewed"}}
    write_new(os.path.join(out, "spans.json"), json.dumps(spans, indent=2) + "\n")
    ws = os.path.join(out, "workspace")
    for src, dst in ((ASSEMBLER, "tools/coherent_narration.py"), (CACHE, "tools/narration_cache.py"), (MAP, MAP_IN)):
        data = open(os.path.join(repo, src), "rb").read()
        if sha_bytes(data) != PINS[src]:
            raise SystemExit(3)
        write_new(os.path.join(ws, dst), data)
    assemble = ["python3", "-I", f"{out_rel}/workspace/tools/coherent_narration.py", "assemble", "--script", SCRIPT, "--units", UNITS,
                "--require-directions", *(["--chapter", args.chapter] if args.chapter else []), "--spans", f"{out_rel}/spans.json",
                "--out", f"{out_rel}/narration"]
    write_new(os.path.join(out, "assemble-command.json"), dumps({
        "run_from": "repository root", "argv": assemble,
        "why_a_workspace": ("The units' direction contract names its map repository-relative, and coherent_narration.py resolves "
                            "it against its own tree. Canonical has no oliver15 map and holds another script, so the repository's "
                            "tools/ copy cannot assemble this candidate. The workspace holds byte copies of the two tools and the "
                            "candidate's map; every other path resolves from the repository root."),
        "then": ["python3", "-I", "-B", f"{HERE}/prepare_stage08_audio.py", "timings", "--prepared", out_rel, "--narration", f"{out_rel}/narration",
                 "--out", f"{out_rel}-timings"] + (["--chapter", args.chapter] if args.chapter else [])}))
    write_new(os.path.join(out, "phrase-marks-plan.json"), dumps({
        "claim": "Not a phrase-timings.json: the assembled WAV hashes do not exist yet. `timings` writes the real file after assembly.",
        "cues": marks_plan}))
    write_new(os.path.join(out, "prepare-receipt.json"), dumps({
        "tool": f"{HERE}/prepare_stage08_audio.py prepare", "pins": PINS, "chapter": args.chapter,
        "selection": {"sha256": sha_file(args.selection), "by": sel["by"], "basis": sel["basis"], "takes": sel["takes"]},
        "answers": {"sha256": sha_file(args.answers) if args.answers else None, "applied": applied},
        "decode_receipt": {"path": receipt_rel, "sha256": sha_file(os.path.join(repo, receipt_rel))},
        "spans_sha256": sha_file(os.path.join(out, "spans.json")), "cues": plan_cues,
        "claim": "Source-bound preparation. Selection and answers are root's; nothing here was heard, measured or accepted."}))
    print(f"prepared {len(units)} units into {args.out}; next: {' '.join(assemble)}")
    return 0


# ----------------------------------------------------------------------------- kit mirror and timings

KIT_NUMBERS = {w: str(i) for i, w in enumerate(ONES[:20])}


def kit_normalize(text):
    """narration.rs check_report_script's normalize, mirrored."""
    chars, prev, words = list(text), None, []
    buf = []
    for i, c in enumerate(chars):
        nxt = chars[i + 1] if i + 1 < len(chars) else None
        between = c in "-‐‑–—" and prev is not None and prev.isalpha() and nxt is not None and nxt.isalpha()
        buf.append(" " if between else c)
        prev = c
    for word in "".join(buf).split():
        w = word.strip("".join(ch for ch in set(word) if not (ch.isalnum() or ch in "-+−")))
        if w:
            words.append(KIT_NUMBERS.get(w.lower(), w))
    return " ".join(words)


def kit_mirror(narration, script, cues):
    """The kit loader's report and timing checks (narration.rs Narrator::load), mirrored in Python."""
    problems = []
    report = json.load(open(os.path.join(narration, "report.json"), encoding="utf8"))
    rate = report["meta"]["rate"]
    for cue in cues:
        rows = report["lines"].get(cue)
        spans = kit_sentences(script[cue])
        if rows is None or len(rows) != len(spans):
            problems.append(f"{cue}: report sentences {None if rows is None else len(rows)}, script {len(spans)}")
            continue
        for k, ((a, b), row) in enumerate(zip(spans, rows)):
            if kit_normalize(script[cue][a:b]) != kit_normalize(row["sentence"]):
                problems.append(f"{cue} sentence {k}: report text differs from the script")
        total = sum(r["out_samples"] + r["gap_after_samples"] for r in rows)
        _params, frames, _ = wav_info(os.path.join(narration, f"{cue}.wav"))
        if abs(total / rate - frames / _params[2]) >= 0.05:
            problems.append(f"{cue}: report accounts for {total / rate:.3f} s, WAV is {frames / _params[2]:.3f} s")
    return problems


def cmd_timings(args):
    repo = args.repo
    prepared = os.path.join(repo, args.prepared)
    pr = json.load(open(os.path.join(prepared, "prepare-receipt.json"), encoding="utf8"))
    plan = json.load(open(os.path.join(prepared, "phrase-marks-plan.json"), encoding="utf8"))["cues"]
    spans_path = os.path.join(prepared, "spans.json")
    if sha_file(spans_path) != pr["spans_sha256"] or pr["chapter"] != args.chapter:
        raise Refused("spans.json changed since prepare, or the chapter differs")
    for src, dst in ((ASSEMBLER, "tools/coherent_narration.py"), (CACHE, "tools/narration_cache.py"), (MAP, MAP_IN)):
        if sha_file(os.path.join(prepared, "workspace", dst)) != PINS[src]:
            raise Refused(f"workspace copy {dst} changed")
    tool = os.path.join(args.prepared, "workspace/tools/coherent_narration.py")
    verify = [sys.executable, "-I", tool, "verify", args.narration, "--script", SCRIPT, "--units", UNITS, "--require-directions",
              *(["--chapter", args.chapter] if args.chapter else [])]
    r = subprocess.run(verify, cwd=repo, capture_output=True, text=True)
    if r.returncode != 0:
        raise Refused(f"coherent_narration verify failed: {r.stderr.strip()[-400:]}")
    narration = os.path.join(repo, args.narration)
    assembly = json.load(open(os.path.join(narration, "assembly.json"), encoding="utf8"))
    if assembly["manifest"] != json.load(open(spans_path, encoding="utf8")):
        raise Refused("the assembly was not made from this spans.json")
    script = read_script(repo)
    cues = list(pr["cues"])
    problems = kit_mirror(narration, script, cues)
    spans = {u["id"][len("oliver_"):]: u for u in json.load(open(spans_path, encoding="utf8"))["units"]}
    entries = {}
    for cue in script:
        if cue not in cues:
            continue
        _p, frames, pcm = wav_info(os.path.join(narration, f"{cue}.wav"))
        _q, frames_src, pcm_src = wav_info(os.path.join(repo, spans[cue]["wav"]))
        if pcm != pcm_src:
            problems.append(f"{cue}: assembled PCM is not the decoded recording sample for sample")
        length = frames / RATE_OUT
        marks, prev_t, prev_b = [], -1.0, -1
        for m in plan.get(cue, []):
            if not (0 <= m["seconds"] < length) or m["seconds"] < prev_t or m["byte"] <= prev_b:
                problems.append(f"{cue}: mark at byte {m['byte']} ({m['seconds']} s) breaks the kit's order or range rule")
            prev_t, prev_b = m["seconds"], m["byte"]
            marks.append({"byte": m["byte"], "seconds": m["seconds"]})
        entries[cue] = {"wav_sha256": sha_file(os.path.join(narration, f"{cue}.wav")),
                        "text_sha256": sha_bytes(script[cue].encode()), "marks": marks}
    if problems:
        raise Refused("\n".join(problems))
    out = new_out(repo, args.out)
    timings = {"basis": "automatic recognition estimates (whisper-cpp large-v3-turbo word onsets), not heard boundaries",
               "script_sha256": PINS[SCRIPT], "prepare_receipt_sha256": sha_file(os.path.join(prepared, "prepare-receipt.json")),
               "cues": entries}
    write_new(os.path.join(out, "phrase-timings.json"), dumps(timings))
    write_new(os.path.join(out, "timings-receipt.json"), dumps({
        "verified": " ".join(verify[1:]).replace(repo, "."), "assembly_sha256": sha_file(os.path.join(narration, "assembly.json")),
        "report_sha256": sha_file(os.path.join(narration, "report.json")), "cues": len(entries),
        "marks": sum(len(e["marks"]) for e in entries.values()),
        "install": f"cp -n {args.out}/phrase-timings.json {args.narration}/phrase-timings.json",
        "claim": "Technical checks and recognizer estimates; no listening, fine sync or render."}))
    print(f"phrase-timings.json for {len(entries)} cues ({sum(len(e['marks']) for e in entries.values())} marks) in {args.out}")
    return 0


# ----------------------------------------------------------------------------- report CLI

def cmd_report(args):
    files, questions, _recordings, _inputs = build_report(args.repo)
    if args.check:
        stale = [n for n, data in files.items()
                 if not os.path.isfile(os.path.join(args.repo, args.check, n)) or open(os.path.join(args.repo, args.check, n), "rb").read() != data.encode("utf8")]
        if stale:
            print("STALE: " + ", ".join(stale))
            return 1
        print(f"in step: {args.check} ({len(files)} files, {len(questions)} questions)")
        return 0
    out = new_out(args.repo, args.out)
    for name, data in files.items():
        write_new(os.path.join(out, name), data)
    cov = json.loads(files["coverage.json"])
    c = cov["counts"]
    print(f"report: {cov['recordings']} recordings of {cov['cues']} cues; sentence onsets {c['sentence_onsets_supported']}/{c['sentence_onsets']} supported; "
          f"phrase anchor marks {c['phrase_anchor_marks_supported']}/{c['phrase_anchor_marks']} supported; {len(questions)} questions")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", default=".")
    sub = ap.add_subparsers(dest="cmd", required=True)
    r = sub.add_parser("report")
    g = r.add_mutually_exclusive_group(required=True)
    g.add_argument("--out")
    g.add_argument("--check")
    d = sub.add_parser("decode")
    d.add_argument("--selection", required=True)
    d.add_argument("--out", required=True)
    d.add_argument("--chapter")
    d.add_argument("--ffmpeg", default="ffmpeg")
    p = sub.add_parser("prepare")
    p.add_argument("--selection", required=True)
    p.add_argument("--answers")
    p.add_argument("--decoded", required=True, help="the decode OUT folder (repository-relative)")
    p.add_argument("--reviewer", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--chapter")
    t = sub.add_parser("timings")
    t.add_argument("--prepared", required=True)
    t.add_argument("--narration", required=True)
    t.add_argument("--out", required=True)
    t.add_argument("--chapter")
    args = ap.parse_args()
    if getattr(args, "chapter", None) is not None and not re.fullmatch(r"\d\d", args.chapter):
        raise SystemExit("refused: --chapter is two digits")
    for name in ("out", "check", "decoded", "prepared", "narration"):
        v = getattr(args, name, None)
        if v is not None and (os.path.isabs(v) or ".." in v.split("/")):
            raise SystemExit(f"refused: --{name} must be repository-relative")
    stale = stale_pins(args.repo)
    if stale:
        print("STALE: " + "\n".join(stale))
        return 3
    try:
        return {"report": cmd_report, "decode": cmd_decode, "prepare": cmd_prepare, "timings": cmd_timings}[args.cmd](args)
    except Refused as e:
        print("REFUSED:\n" + str(e))
        return 1


if __name__ == "__main__":
    sys.exit(main())
