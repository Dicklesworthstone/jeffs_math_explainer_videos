#!/usr/bin/env python3
"""Voice-agnostic narration assembly preparation for root's guarded use (bead mx-video-program-program-87do.63).

Run from the repository root with `python3 -I -B`. A BINDING file names one voice, its take
inventories, their recognition runs, the candidate tree and every pinned hash; this file names no
voice. Every subcommand writes only into a NEW folder given by --out, refuses an existing one,
and never edits a recording, candidate, canonical file, receipt or review. Nothing here selects
a take, synthesizes, renders, builds or listens.

    report   Read-only. For every recording in the binding's inventory: recognition coverage,
             each sentence onset and each phrase anchor as the recognizer placed it, every
             normalized recognizer disagreement, and the explicit questions that block a
             recording until root answers them. `report --check DIR` reruns and compares bytes.
    decode   Root's run. Converts each SELECTED complete recording to 24 kHz mono PCM16 with
             root's earlier ffmpeg arguments (no cut, time or pitch change) and writes the
             external receipt tools/coherent_narration.py reads.
    prepare  Root's run. From root's selection, root's answers and the decode receipt: the span
             manifest (whole recordings, sentence ends at the recognizer's onsets or root's
             samples, zero inserted silence), the binding's proposed direction dispositions, a
             workspace holding byte copies of the assembler and the candidate's direction map,
             and the exact assemble command. The spans are dry-run through that same assembler
             code before prepare reports success.
    timings  Root's run, after root's assemble. Verifies the assembly with the same tool, checks
             each assembled cue is its decoded recording sample for sample, mirrors the kit's
             report and phrase-timing checks, and writes phrase-timings.json for root to install.

One binding holds one voice. A binding may list several inventories, for example the full
retained set plus later targeted replacements of the same voice. An inventory states which receipt
its receipt_sha256 hashes (`receipt_hash_of`, default the downloads receipt; a downloads receipt it
does not hash must be pinned) and may name each recording's request (`request`), which must carry
the recording's cue and exactly its inventoried text. A recording is selectable only
while its requested text is the binding script's current text for its cue. Recordings of older
wording stay listed as superseded and are never deleted or selected. A selection names each
recording by its file name or its repository-relative path; a file name shared with a superseded
recording must be given as the path, so a choice made about one file cannot land on another. Each selectable recording
is timed from its own recognition, so a replaced take gets fresh timings and an unchanged take
keeps the recognition of its unchanged bytes. A binding whose `use` is "historical-test" is test
input, not production audio: decode, prepare and timings then require --test-only and stamp every
output "production": false. Nothing here carries one recording's or voice's timings to another.

Recognizer times are estimates. whisper-cpp word segments usually start where the previous word
ended, so an onset can sit at the start of the pause before its word. Root may replace any onset
or mark with a measured sample in the answers file. Nothing here is a heard boundary, fine sync
or acceptance.

Exit codes: 0 done or in step; 1 refused (each problem listed) or a stale report; 2 usage or
OUT refused; 3 a pinned input changed.
"""

import argparse
import collections
import difflib
import hashlib
import importlib.util
import json
import os
import re
import subprocess
import sys
import wave

sys.dont_write_bytecode = True  # the dry run imports the workspace assembler; leave no __pycache__

HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-audio-preparation01"
TOOL = f"{HERE}/prepare_audio.py"
RATE_OUT = 24000  # what tools/coherent_narration.py assembles and narration_cache accepts
FFMPEG_ARGS = ["-nostdin", "-n", "-v", "error"]  # root's earlier decode, argument for argument
FFMPEG_OUT = ["-ac", "1", "-ar", "24000", "-c:a", "pcm_s16le"]
MIN_RUN = 3  # a supported word lies in a run of at least this many agreeing words
BATCH_SCHEMA = "franken-whisper-batch-result-v1"
ONES = "zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen seventeen eighteen nineteen".split()
TENS = "zero ten twenty thirty forty fifty sixty seventy eighty ninety".split()


class Refused(Exception):
    pass


class Stale(Exception):
    pass


def usage(message):
    print(f"refused: {message}", file=sys.stderr)
    raise SystemExit(2)


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


def load_json(path):
    with open(path, encoding="utf8") as f:
        return json.load(f)


def write_new(path, data):
    os.makedirs(os.path.dirname(path) or ".", exist_ok=True)
    with open(path, "xb") as h:
        h.write(data if isinstance(data, bytes) else data.encode("utf8"))


def norm(path):
    return path.replace(os.sep, "/")


def read_script(path):
    rows = {}
    with open(path, encoding="utf8") as f:
        for row in f.read().splitlines():
            if row.strip() and not row.startswith("#"):
                cue, text = row.split("\t", 1)
                rows[cue] = text
    return rows


def kit_sentences(text):
    """(start, end) character spans by the kit's rule: split after . ? ! before a space or the end."""
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


def wav_info(path):
    """((channels, width, rate, comptype), frames, pcm bytes) of a WAV file."""
    try:
        with wave.open(path, "rb") as w:
            params = (w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getcomptype())
            frames = w.getnframes()
            pcm = w.readframes(frames)
    except (wave.Error, EOFError) as e:
        raise Refused(f"{os.path.basename(path)}: not a readable PCM WAV ({e}); this helper takes PCM16 WAV recordings only")
    if len(pcm) != frames * params[0] * params[1]:
        raise Refused(f"{os.path.basename(path)}: truncated sample data")
    return params, frames, pcm


# ----------------------------------------------------------------------------- binding

BINDING_KEYS = ("schema", "use", "voice", "why", "tree", "topic", "inventories", "recognition", "phrase_anchors",
                "anchor_template", "dispositions", "assembler", "assembler_cache", "kit", "content_questions", "pins")


def load_binding(path):
    """The binding names every input and pins each by hash. Exit 2 when malformed, 3 when stale."""
    if os.path.isabs(path) or ".." in norm(path).split("/") or not os.path.isfile(path):
        usage(f"--binding must name a repository-relative file: {path}")
    raw = open(path, "rb").read()
    b = json.loads(raw)
    missing = [k for k in BINDING_KEYS if k not in b]
    if b.get("schema") != "audio-preparation-binding-v1" or missing:
        usage(f"{path} is not an audio-preparation-binding-v1 file (missing {missing})")
    if b["use"] not in ("historical-test", "production"):
        usage("binding use must be historical-test or production")
    voice = b["voice"]
    if not all(str(voice.get(k) or "").strip() for k in ("provider", "name", "receipt_source", "download_source_contains")):
        usage("binding voice needs provider, name, receipt_source and download_source_contains (the one-voice guard)")
    if not b["inventories"] or any(not inv.get("take_receipt") for inv in b["inventories"]):
        usage("every inventory needs takes_dir and a take_receipt pattern; the downloads receipts carry the voice guard")
    crate = f"{b['tree']}/explainer_corpus/{b['topic']}"
    b["script"], b["units"] = f"{crate}/script.tsv", f"{crate}/narration_units.json"
    needed = [b["script"], b["units"], b["phrase_anchors"], b["anchor_template"], b["dispositions"],
              b["assembler"], b["assembler_cache"], b["kit"]] + [inv["path"] for inv in b["inventories"]]
    for run in b["recognition"]:
        needed += [f"{run['job']}/command.log", f"{run['job']}/resource-receipt.json", run["frozen"]]
        if run.get("review"):
            needed.append(run["review"])
    if voice.get("identity_record"):
        needed.append(voice["identity_record"])
    unpinned = [p for p in needed if p not in b["pins"]]
    if unpinned:
        usage("the binding does not pin " + ", ".join(unpinned))
    stale = []
    for rel, want in sorted(b["pins"].items()):
        got = sha_file(rel) if os.path.isfile(rel) else None
        if got != want:
            stale.append(f"{rel} is {got}, pinned {want}")
    if stale:
        raise Stale("\n".join(stale))
    units = load_json(b["units"])
    contract = units.get("direction_contract") or {}
    b["map_in_tree"] = contract.get("map", {}).get("path")
    b["map"] = f"{b['tree']}/{b['map_in_tree']}"
    if contract.get("topic") != b["topic"] or b["pins"].get(b["map"]) != contract.get("map", {}).get("sha256"):
        usage("the units' direction contract must name this topic and a map the binding pins at the same hash")
    b["evidence"] = []
    for d in load_json(b["dispositions"])["direction_dispositions"]:
        if b["pins"].get(d["evidence"]["path"]) != d["evidence"]["sha256"]:
            usage(f"disposition evidence {d['evidence']['path']} is not pinned at its stated hash")
        b["evidence"].append(d["evidence"]["path"])
    b["path"], b["sha256"] = norm(path), sha_bytes(raw)
    return b


def binding_summary(b):
    return {"path": b["path"], "sha256": b["sha256"], "use": b["use"], "voice": {k: b["voice"][k] for k in ("provider", "name")}}


def take_rel(path, takes_dirs):
    """An inventory or recognition path, absolute or relative, as its repository-relative form."""
    name = os.path.basename(path)
    for d in takes_dirs:
        if norm(path).endswith(f"{d}/{name}"):
            return f"{d}/{name}"
    raise Refused(f"{name}: not a recording under {', '.join(takes_dirs)}")


def guard_out(b, out):
    if os.path.isabs(out) or ".." in norm(out).split("/") or not out.strip("/"):
        usage(f"--out {out} must be repository-relative")
    full = os.path.realpath(out)
    if os.path.lexists(out):
        usage(f"OUT {out} exists; name a new folder")
    guards = ["explainer_corpus", "tools", "crates", HERE, b["tree"]] + [inv["takes_dir"] for inv in b["inventories"]]
    for run in b["recognition"]:
        guards += [run["job"], os.path.dirname(run["frozen"])] + ([os.path.dirname(run["review"])] if run.get("review") else [])
    guards += [os.path.dirname(p) for p in [b["phrase_anchors"], b["anchor_template"], b["dispositions"]] + b["evidence"]]
    for g in guards:
        gf = os.path.realpath(g)
        if full == gf or full.startswith(gf + os.sep):
            usage(f"OUT {out} lies inside the input folder {g}")
    return out


def make_out(b, out):
    guard_out(b, out)
    os.makedirs(out)
    return out


# ----------------------------------------------------------------------------- inputs

def load_inputs(b):
    """Everything the subcommands read, each piece checked against the others."""
    problems = []
    script = read_script(b["script"])
    dirs = [inv["takes_dir"] for inv in b["inventories"]]
    takes, superseded, receipts, inventories = {}, {}, {}, []
    for spec in b["inventories"]:
        inv = load_json(spec["path"])
        if inv.get("selection") is not None:
            problems.append(f"{spec['path']} claims a selection; an inventory only lists recordings")
        inventories.append({"path": spec["path"], "sha256": b["pins"][spec["path"]], "script_sha256": inv.get("script_sha256"), "takes": len(inv["takes"])})
        for t in inv["takes"]:
            try:
                rel = take_rel(t["wav"], [spec["takes_dir"]])
            except Refused as e:
                problems.append(str(e))
                continue
            if rel in takes or rel in superseded:
                problems.append(f"{rel} is inventoried twice")
            receipt = spec["take_receipt"].format(takes_dir=spec["takes_dir"], cue=t["cue"])
            # Inventories differ in which receipt their receipt_sha256 hashes (the downloads receipt in
            # full80-retained-inventory01, the request in clarity-retained-inventory01). A downloads
            # receipt the inventory does not hash must be pinned by the binding instead.
            hashed = spec.get("receipt_hash_of", spec["take_receipt"]).format(takes_dir=spec["takes_dir"], cue=t["cue"])
            voice_gap = None
            if hashed != receipt and b["pins"].get(receipt) is None:
                problems.append(f"{rel}: downloads receipt {receipt} is neither hashed by its inventory nor pinned")
            if not os.path.isfile(hashed) or ("receipt_sha256" in t and sha_file(hashed) != t["receipt_sha256"]):
                problems.append(f"{rel}: receipt {hashed} is missing or changed")
            if spec.get("request"):
                # The request is the UI-side record of the submitted words: it must name this cue and
                # exactly the text the inventory says this recording is of.
                request_path = spec["request"].format(takes_dir=spec["takes_dir"], cue=t["cue"])
                request = load_json(request_path) if os.path.isfile(request_path) else {}
                text = request.get("text")
                if request.get("cue") != t["cue"] or not isinstance(text, str) \
                        or sha_bytes(text.encode()) != t["text_sha256"] or request.get("text_sha256") != t["text_sha256"]:
                    problems.append(f"{rel}: request {request_path} does not name its cue and inventoried text")
            if not os.path.isfile(receipt):
                problems.append(f"{rel}: downloads receipt {receipt} is missing")
            else:
                if receipt not in receipts:
                    receipts[receipt] = load_json(receipt)
                want = b["voice"]["download_source_contains"]
                rows = [d for d in receipts[receipt].get("downloads", []) if os.path.basename(d.get("retained", "")) == os.path.basename(rel)]
                if len(rows) != 1:
                    problems.append(f"{rel}: its downloads receipt does not list it exactly once")
                elif want not in os.path.basename(rows[0].get("source", "")):
                    said = [f"{k}: {v}" for k, v in sorted(receipts[receipt].items()) if isinstance(v, str) and b["voice"]["name"] in v]
                    voice_gap = {"file_name": os.path.basename(rows[0].get("source", "")), "receipt_says": said}
            t = dict(t, rel=rel, receipt=receipt, inventory=spec["path"], voice_gap=voice_gap)
            if t["cue"] in script and t["text_sha256"] == sha_bytes(script[t["cue"]].encode()):
                takes[rel] = t
            else:
                superseded[rel] = t
    names = collections.Counter(os.path.basename(r) for r in takes)
    if any(n > 1 for n in names.values()):
        problems.append("selectable recordings must have distinct file names: " + ", ".join(n for n, k in names.items() if k > 1))
    missing = [c for c in script if c not in {t["cue"] for t in takes.values()}]
    if missing:
        problems.append("no recording of the current text for: " + ", ".join(missing))
    units = load_json(b["units"])
    if units.get("script_sha256") != b["pins"][b["script"]] or any(len(u["line_ids"]) != 1 for u in units["units"]) \
            or [u["line_ids"][0] for u in units["units"]] != list(script):
        problems.append("units must be one unit per cue, in script order, bound to this script (external UI takes are whole cues)")
    anchors = load_json(b["phrase_anchors"])["anchors"]
    template = load_json(b["anchor_template"])
    if {(c, m["byte"]) for c, e in template["cues"].items() for m in e["marks"]} != {(a["cue"], a["byte"]) for a in anchors}:
        problems.append("the phrase anchors and the unmeasured template disagree")
    for a in anchors:
        text = script.get(a["cue"], "").encode()
        if text[a["byte"] : a["byte_end"]].decode(errors="replace") != a["phrase"] or a.get("cue_text_sha256", sha_bytes(text)) != sha_bytes(text):
            problems.append(f"anchor {a['cue']}@{a['byte']} no longer lands on '{a['phrase']}'")
    runs = []
    for run in b["recognition"]:
        rc = load_json(f"{run['job']}/resource-receipt.json")
        cmd = rc.get("command", [])
        arg = lambda flag: cmd[cmd.index(flag) + 1] if flag in cmd else None
        if rc.get("exit") != 0 or rc.get("admitted") is not True or rc.get("preflight_only") is not False \
                or rc.get("inputs_sha256") != rc.get("final_inputs_sha256"):
            problems.append(f"{run['name']}: the run did not finish cleanly with stable inputs")
        if os.path.basename(arg("--model") or "") != run["model"]:
            problems.append(f"{run['name']}: model {os.path.basename(arg('--model') or '')} is not {run['model']}")
        if not norm(os.path.dirname(arg("--inputs-from") or "")).endswith(os.path.dirname(run["frozen"])):
            problems.append(f"{run['name']}: its inputs list is not beside the pinned frozen inputs")
        if "--split-on-word" not in cmd or arg("--max-segment-length") != "1":
            problems.append(f"{run['name']}: not one word per segment")
        frozen = {}
        for f in load_json(run["frozen"])["files"]:
            if norm(f["path"]).endswith(".wav"):
                try:
                    frozen[take_rel(f["path"], dirs)] = f["sha256"]
                except Refused:
                    pass
        records, other_text = {}, 0
        with open(f"{run['job']}/command.log", encoding="utf8") as log:
            for line in log:
                if not line.startswith("{"):
                    continue
                r = json.loads(line)
                if r.get("schema_version") != BATCH_SCHEMA:
                    continue
                try:
                    rel = take_rel(r["input"], dirs)
                except Refused as e:
                    problems.append(f"{run['name']}: {e}")
                    continue
                known = takes.get(rel) or superseded.get(rel)
                if r["status"] != "ok":
                    problems.append(f"{run['name']}: {rel} status {r['status']}")
                elif known is None or frozen.get(rel) != known["sha256"]:
                    problems.append(f"{run['name']}: {rel} was not frozen with its inventoried hash")
                elif rel in records:
                    problems.append(f"{run['name']}: {rel} recognized twice")
                elif rel in superseded:
                    other_text += 1
                else:
                    records[rel] = r["report"]["result"]
        runs.append(dict(run, records=records, superseded_records=other_text, log_sha256=b["pins"][f"{run['job']}/command.log"]))
    if problems:
        raise Refused("\n".join(problems))
    return {"script": script, "takes": takes, "superseded": superseded, "anchors": anchors, "runs": runs, "inventories": inventories}


def recognition_for(inputs):
    """Each recording's one anchor-role recognition, or its PCM twin's (a provenance question)."""
    out, problems = {}, []
    for rel in inputs["takes"]:
        own = [(run, run["records"][rel]) for run in inputs["runs"] if run["role"] == "anchors" and rel in run["records"]]
        if len(own) > 1:
            problems.append(f"{rel}: recognized by more than one anchor run")
        elif own:
            out[rel] = {"run": own[0][0], "result": own[0][1], "via": None}
    for rel, t in inputs["takes"].items():
        if rel in out:
            continue
        twins = sorted(r for r, u in inputs["takes"].items() if r in out and out[r]["via"] is None and u["pcm"] == t["pcm"] and u["cue"] == t["cue"])
        if twins:
            out[rel] = dict(out[twins[0]], via=twins[0])
        else:
            problems.append(f"{rel}: no recognition and no recognized recording with the same samples")
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
    match, run, replaced = {}, {}, {}
    for op, i1, i2, j1, j2 in sm.get_opcodes():
        if op == "equal":
            for k in range(i2 - i1):
                match[i1 + k] = j1 + k
                run[i1 + k] = i2 - i1
        elif op == "replace" and i2 - i1 == j2 - j1:
            for k in range(i2 - i1):
                replaced[i1 + k] = j1 + k
    return canon, obs, match, run, replaced


def place(i, al, seconds, after):
    """The recognizer's onset for canonical word i: supported, or a question with any candidate."""
    canon, obs, match, run, replaced = al
    row = {"word": canon[i][0]}
    if i in match:
        j = match[i]
        o = obs[j]
        row.update(asr_start_s=round(o["start"], 6), asr_end_s=round(o["end"], 6), agreeing_run=run[i])
        if j > 0:
            row["previous_word_end_s"] = round(obs[j - 1]["end"], 6)
            row["abuts_previous_word"] = abs(obs[j - 1]["end"] - o["start"]) < 1e-6
        if run[i] < MIN_RUN:
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
    if i in replaced and i - 1 in match and i + 1 in match and run[i - 1] >= MIN_RUN and run[i + 1] >= MIN_RUN:
        o = obs[replaced[i]]
        if o["k"] == 0 and after < o["start"] < seconds:
            row["candidate"] = {"heard": o["w"], "asr_start_s": round(o["start"], 6), "sample_24k": round(o["start"] * RATE_OUT),
                                "why": "the recognizer's one different word in this slot, between agreeing runs on both sides"}
    return row


def points_for(cue, name, text, result, seconds, anchors):
    al = align(text, result)
    canon = al[0]
    sentences, after = [], 0.0
    for n, (a, _b) in enumerate(kit_sentences(text)):
        if n == 0:
            sentences.append({"index": 0, "word": canon[0][0], "status": "file_start", "sample_24k": 0})
            continue
        i = next(ix for ix, (_, off) in enumerate(canon) if off >= a)
        row = dict(id=f"onset:{cue}:s{n}:{name}", index=n, char=a, **place(i, al, seconds, after))
        if row["status"] == "supported":
            after = row["asr_start_s"]
        sentences.append(row)
    marks, after = [], -1.0
    btext = text.encode()
    for an in sorted((x for x in anchors if x["cue"] == cue), key=lambda x: x["byte"]):
        offset = len(btext[: an["byte"]].decode())
        i = next((ix for ix, (_, off) in enumerate(canon) if off == offset), None)
        pid = f"anchor:{cue}:b{an['byte']}:{name}"
        if i is None:
            marks.append({"id": pid, "byte": an["byte"], "phrase": an["phrase"], "status": "no_word_at_byte"})
            continue
        row = dict(id=pid, byte=an["byte"], phrase=an["phrase"], **place(i, al, seconds, -1.0))
        if row["status"] == "supported":
            if row["asr_start_s"] <= after:
                row["status"] = "out_of_order_or_range"
                row.pop("sample_24k", None)
            else:
                after = row["asr_start_s"]
        marks.append(row)
    return sentences, marks


def differences(text, result):
    a = [w for w, _ in tokens(text)]
    o = [x["w"] for x in observed_words(result)]
    return [{"op": op, "script": " ".join(a[i1:i2]), "heard": " ".join(o[j1:j2])}
            for op, i1, i2, j1, j2 in difflib.SequenceMatcher(a=a, b=o, autojunk=False).get_opcodes() if op != "equal"]


# ----------------------------------------------------------------------------- report

def build_report(b):
    inputs = load_inputs(b)
    script = inputs["script"]
    order = {c: k for k, c in enumerate(script)}
    for rel, t in sorted(inputs["superseded"].items()):
        if sha_file(rel) != t["sha256"]:
            raise Refused(f"{rel} changed on disk")
    for rel, t in inputs["takes"].items():
        if sha_file(rel) != t["sha256"]:
            raise Refused(f"{rel} changed on disk")
        params, frames, pcm = wav_info(rel)
        if params[0] != 1 or params[1] != 2 or params[3] != "NONE":
            raise Refused(f"{rel}: not mono PCM16")
        if t.get("pcm_sha256", sha_bytes(pcm)) != sha_bytes(pcm) or t.get("frames", frames) != frames:
            raise Refused(f"{rel}: the inventory's frame count or PCM hash differs from the file")
        t.update(rate=params[2], frames=frames, pcm=sha_bytes(pcm))
    recog = recognition_for(inputs)
    recordings, questions, diffs = [], [], []
    for rel in sorted(inputs["takes"], key=lambda r: (order[inputs["takes"][r]["cue"]], r)):
        t = inputs["takes"][rel]
        cue, name, seconds = t["cue"], os.path.basename(rel), t["frames"] / t["rate"]
        r = recog[rel]
        sentences, marks = points_for(cue, name, script[cue], r["result"], seconds, inputs["anchors"])
        recordings.append({
            "cue": cue, "file": name, "path": rel, "sha256": t["sha256"], "pcm_sha256": t["pcm"], "rate": t["rate"], "frames": t["frames"],
            "seconds": seconds,
            "recognition": {"run": r["run"]["name"], "model": r["run"]["model"], "log_sha256": r["run"]["log_sha256"],
                            "recognized_file": os.path.basename(r["via"]) if r["via"] else name},
            "sentences": sentences, "phrase_anchors": marks})
        for run in inputs["runs"]:
            if rel in run["records"]:
                d = differences(script[cue], run["records"][rel])
                if d:
                    diffs.append({"file": name, "run": run["name"], "model": run["model"], "differences": d})
        for s in sentences[1:]:
            if s["status"] != "supported":
                questions.append({
                    "id": s["id"], "kind": "sentence_onset", "cue": cue, "files": [name],
                    "detail": f"Sentence {s['index']} of {cue} starts with '{s['word']}'; the recognizer gives no supported onset ({s['status']}).",
                    "candidate": s.get("candidate"), "decisions": (["use_candidate"] if s.get("candidate") else []) + ["sample"]})
        for m in marks:
            if m["status"] != "supported":
                questions.append({
                    "id": m["id"], "kind": "phrase_anchor", "cue": cue, "files": [name],
                    "detail": f"Anchor '{m['phrase']}' at byte {m['byte']} has no supported onset ({m['status']}); without a mark the kit falls back to its estimate.",
                    "candidate": m.get("candidate"), "decisions": (["use_candidate"] if m.get("candidate") else []) + ["sample", "omit_mark"]})
        if t["voice_gap"]:
            gap, voice = t["voice_gap"], f"{b['voice']['provider']} {b['voice']['name']}"
            observed = bool(gap["receipt_says"])
            questions.append({
                "id": f"voice:{cue}:{name}", "kind": "voice", "cue": cue, "files": [name], "blocking": not observed,
                "detail": f"One binding holds one voice ({voice}). The downloaded file name {gap['file_name']!r} does not contain "
                          f"{b['voice']['download_source_contains']!r}. "
                          + (f"The downloads receipt records a contemporaneous provider observation naming the voice ({' | '.join(gap['receipt_says'])}). "
                             "That observation is bound as provider provenance; it is not a listening confirmation. Advisory: a missing "
                             "name is not evidence of another voice. Root may record a provider-history recheck here."
                             if observed else "The downloads receipt does not name the voice either. Root confirms the provider voice or rejects the recording."),
                "provenance": {"receipt": t["receipt"], "receipt_sha256": sha_file(t["receipt"]), "observation": gap["receipt_says"]},
                "candidate": None, "decisions": ["confirmed_voice", "reject_take"]})
        if r["via"]:
            questions.append({
                "id": f"provenance:{cue}:{name}", "kind": "provenance", "cue": cue, "files": [name],
                "detail": f"{name} was not recognized itself. Its samples equal {os.path.basename(r['via'])}'s, whose recognition is used. "
                          "Equal samples do not say how many generations produced the files.",
                "candidate": None, "decisions": ["use_pcm_identity"]})
    other_text = []
    for c in b["content_questions"]:
        if c["cue"] not in script or c["text_sha256"] != sha_bytes(script[c["cue"]].encode()):
            other_text.append(f"{c['cue']}: {c['script_word']}/{c['heard']}")
            continue
        for e in recordings:
            if e["cue"] != c["cue"]:
                continue
            heard = sorted({d["model"] for d in diffs if d["file"] == e["file"] for x in d["differences"]
                            if c["script_word"] in x["script"].split() and c["heard"] in x["heard"].split()})
            questions.append({
                "id": f"content:{c['cue']}:{c['script_word']}:{e['file']}", "kind": "content", "cue": c["cue"], "files": [e["file"]],
                "detail": f"{c['note']} Recognizers transcribing '{c['heard']}' for '{c['script_word']}' in {e['file']}: "
                          f"{', '.join(heard) or 'none'}. A transcript is not listening.",
                "candidate": None, "decisions": ["heard_as_script", "reject_take"]})
    questions.sort(key=lambda q: (order[q["cue"]], q["id"]))
    for q in questions:
        q.setdefault("blocking", True)
    counts = collections.Counter()
    gaps, long_words = [], []
    for e in recordings:
        for s in e["sentences"][1:]:
            counts["sentence_onsets"] += 1
            if s["status"] == "supported":
                counts["sentence_onsets_supported"] += 1
                if s.get("abuts_previous_word"):
                    counts["supported_onsets_abutting_previous_word"] += 1
                    gaps.append(s["asr_end_s"] - s["asr_start_s"])
                    if round(gaps[-1], 3) >= 0.6:
                        long_words.append({"id": s["id"], "word": s["word"], "word_segment_s": round(gaps[-1], 3)})
        for m in e["phrase_anchors"]:
            counts["phrase_anchor_marks"] += 1
            counts["phrase_anchor_marks_supported"] += m["status"] == "supported"
    long_words.sort(key=lambda r: (-r["word_segment_s"], r["id"]))
    gaps.sort()
    bind = binding_summary(b)
    use_note = ("TEST INPUT ONLY: this binding's recordings are historical, not production audio. " if b["use"] == "historical-test" else "")
    claim = (use_note + "Recognizer estimates only (whisper-cpp word segments). No take selected, heard, measured, cut or accepted. "
             f"A 'supported' onset is a word the recognizer agrees on inside a run of at least {MIN_RUN} agreeing words, "
             "first in its segment, in range and in order.")
    coverage = {
        "binding": bind, "script_sha256": b["pins"][b["script"]], "inventories": inputs["inventories"],
        "cues": len(script), "recordings": len(recordings), "rates": sorted({e["rate"] for e in recordings}),
        "superseded_text_recordings": sorted(inputs["superseded"]),
        "content_questions_for_other_text": other_text,
        "runs": [{"name": r["name"], "model": r["model"], "role": r["role"], "records": len(r["records"]),
                  "superseded_records": r["superseded_records"], "log_sha256": r["log_sha256"]} for r in inputs["runs"]],
        "recognized_by_pcm_identity": [e["file"] for e in recordings if e["recognition"]["recognized_file"] != e["file"]],
        "counts": dict(sorted(counts.items())),
        "abutting_onset_word_seconds": {"count": len(gaps), "median": round(gaps[len(gaps) // 2], 3) if gaps else None,
                                        "max": round(gaps[-1], 3) if gaps else None,
                                        "meaning": "an abutting onset's word segment begins where the previous word ended, so the onset may lie up to about this long before the word"},
        "abutting_onsets_with_word_segment_at_least_0_6_s": long_words,
        "questions": len(questions), "blocking_questions": sum(q["blocking"] for q in questions),
        "files_with_blocking_questions": sorted({f for q in questions if q["blocking"] for f in q["files"]}),
        "claim": claim}
    files = {
        "anchors.json": dumps({"binding": bind, "claim": claim, "min_agreeing_run": MIN_RUN, "recordings": recordings}),
        "questions.json": dumps({"binding": bind, "claim": "A blocking question stops the files it names from being prepared until root "
                                 "answers it; an advisory one stays visible and may be answered. Answers are root's statements; this "
                                 "tool checks only their form.", "questions": questions}),
        "answers-template.json": dumps({"schema": "audio-preparation-answers-v1", "binding_sha256": b["sha256"],
                                        "answers": {q["id"]: {"decision": None, "by": None, "basis": None} for q in questions}}),
        "coverage.json": dumps(coverage),
        "asr-differences.json": dumps({"binding": bind, "claim": "Normalized recognizer disagreements, every transcript. Diagnostic only; not speech evidence.",
                                       "transcripts": diffs}),
    }
    return files, questions, recordings, inputs


# ----------------------------------------------------------------------------- selection and answers

def load_selection(b, path, inputs, chapter):
    sel = load_json(path)
    problems = []
    if sel.get("schema") != "audio-preparation-selection-v1" or sel.get("binding_sha256") != b["sha256"] \
            or sel.get("script_sha256") != b["pins"][b["script"]]:
        problems.append("the selection is not audio-preparation-selection-v1 for this binding and script")
    if not str(sel.get("by") or "").strip() or not str(sel.get("basis") or "").strip():
        problems.append("the selection needs `by` and `basis`: choosing takes is root's decision")
    want = [c for c in inputs["script"] if chapter is None or c.startswith(chapter + "_")]
    if not want:
        problems.append(f"chapter {chapter} has no cues")
    elif list(sel.get("takes", {})) != want:
        problems.append(f"the selection must name exactly these {len(want)} cues in script order: {want[0]} … {want[-1]}")
    chosen = {}
    by_name = {os.path.basename(r): t for r, t in inputs["takes"].items()}
    old_rel = set(inputs["superseded"])
    old = {os.path.basename(r) for r in old_rel}
    for cue, name in sel.get("takes", {}).items():
        name = name if isinstance(name, str) else ""
        if "/" in name:
            # A repository-relative path names exactly one recording.
            t = inputs["takes"].get(name)
            stale = name in old_rel
        elif name in old and name in by_name:
            # A replacement can share its file name with a recording of the older wording; a bare name
            # would let a choice made about one file silently land on the other.
            problems.append(f"{cue}: {name} is also the name of a recording of superseded wording; "
                            f"name the recording by its path ({by_name[name]['rel']})")
            continue
        else:
            t = by_name.get(name)
            stale = name in old
        if t is None or t["cue"] != cue:
            why = "it records superseded wording" if stale else "it is not an inventoried recording of that cue"
            problems.append(f"{cue}: {name} cannot be selected; {why}")
            continue
        chosen[cue] = t
    if problems:
        raise Refused("\n".join(problems))
    return sel, chosen


def check_answers(b, path, questions, recordings, chosen):
    data = load_json(path) if path else {"schema": "audio-preparation-answers-v1", "binding_sha256": b["sha256"], "answers": {}}
    problems = []
    if data.get("schema") != "audio-preparation-answers-v1" or data.get("binding_sha256") != b["sha256"]:
        problems.append("the answers are not audio-preparation-answers-v1 for this binding")
    known = {q["id"]: q for q in questions}
    points = {}
    for e in recordings:
        for s in e["sentences"][1:]:
            points[s["id"]] = ["sample"]
        for m in e["phrase_anchors"]:
            points[m["id"]] = ["sample", "omit_mark"]
    names = {os.path.basename(t["rel"]) for t in chosen.values()}
    applied = {}
    for qid, a in sorted(data.get("answers", {}).items()):
        q = known.get(qid)
        if q is None and qid not in points:
            problems.append(f"answer to an unknown question or point {qid}")
            continue
        files = q["files"] if q else [qid.rsplit(":", 1)[1]]
        if not set(files) & names or (a or {}).get("decision") is None:
            continue
        allowed = q["decisions"] if q else points[qid]
        if a["decision"] not in allowed:
            problems.append(f"{qid}: decision {a['decision']!r} is not one of {allowed}")
        elif not str(a.get("by") or "").strip() or not str(a.get("basis") or "").strip():
            problems.append(f"{qid}: an answer needs `by` and `basis`")
        elif a["decision"] == "sample" and (type(a.get("sample_24k")) is not int or a["sample_24k"] <= 0):
            problems.append(f"{qid}: decision 'sample' needs a positive integer sample_24k")
        elif a["decision"] == "reject_take":
            problems.append(f"{qid}: root rejected {', '.join(sorted(set(files) & names))}; select another recording")
        else:
            applied[qid] = {"decision": a["decision"], "by": a["by"], "basis": a["basis"],
                            **({"sample_24k": a["sample_24k"]} if a["decision"] == "sample" else {})}
    for q in questions:
        if q["blocking"] and set(q["files"]) & names and q["id"] not in applied and not any(q["id"] in p for p in problems):
            problems.append(f"open question blocks the selection: {q['id']}: {q['detail']}")
    if problems:
        raise Refused("\n".join(problems))
    return applied


def resolved_points(entry, applied, frames24):
    """Sentence ends (24 kHz samples) and phrase marks (seconds) for one selected recording."""
    starts = []
    for s in entry["sentences"][1:]:
        a = applied.get(s["id"], {})
        if a.get("decision") == "sample":
            starts.append((a["sample_24k"], f"sample given by {a['by']}: {a['basis']}"))
        elif a.get("decision") == "use_candidate":
            starts.append((s["candidate"]["sample_24k"], f"recognizer candidate accepted by {a['by']}"))
        elif s["status"] == "supported":
            starts.append((s["sample_24k"], "recognizer onset"))
        else:
            raise Refused(f"{s['id']}: no onset")
    ends = [s for s, _ in starts] + [frames24]
    if any(y <= x for x, y in zip([0] + ends, ends)):
        raise Refused(f"{entry['cue']} {entry['file']}: sentence ends are not strictly increasing inside the recording: {ends}")
    marks = []
    for m in entry["phrase_anchors"]:
        a = applied.get(m["id"], {})
        if a.get("decision") == "omit_mark":
            continue
        if a.get("decision") == "sample":
            marks.append({"byte": m["byte"], "seconds": a["sample_24k"] / RATE_OUT, "basis": f"sample given by {a['by']}: {a['basis']}"})
        elif a.get("decision") == "use_candidate":
            marks.append({"byte": m["byte"], "seconds": m["candidate"]["asr_start_s"], "basis": f"recognizer candidate accepted by {a['by']}"})
        elif m["status"] == "supported":
            marks.append({"byte": m["byte"], "seconds": m["asr_start_s"], "basis": "recognizer onset"})
        else:
            raise Refused(f"{m['id']}: no mark")
    return starts, ends, marks


def expected_frames(t):
    """Decoded length: exactly ceil(n/2) for 48 kHz (ffmpeg with these arguments; equal to root's earlier
    n // 2 check for every even count), else within one sample of the ratio."""
    if t["rate"] == 48000:
        return (t["frames"] + 1) // 2, 0
    return round(t["frames"] * RATE_OUT / t["rate"]), 1


def test_gate(b, args):
    if b["use"] == "historical-test" and not args.test_only:
        usage(f"{b['path']} binds historical {b['voice']['name']} recordings as test input, not production audio. "
              "Pass --test-only to run a test, or use a production binding.")
    if b["use"] == "production" and args.test_only:
        usage("--test-only is for historical-test bindings")
    return b["use"] == "production"


# ----------------------------------------------------------------------------- decode

def cmd_decode(b, args):
    production = test_gate(b, args)
    _files, _questions, _recordings, inputs = build_report(b)
    _sel, chosen = load_selection(b, args.selection, inputs, args.chapter)
    out = make_out(b, args.out)
    version = subprocess.run([args.ffmpeg, "-version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    records, problems = [], []
    for cue, t in chosen.items():
        dst = f"{out}/decoded/{cue}.wav"
        os.makedirs(os.path.dirname(dst), exist_ok=True)
        subprocess.run([args.ffmpeg, *FFMPEG_ARGS, "-i", t["rel"], *FFMPEG_OUT, dst], check=True, timeout=300)
        params, frames, _pcm = wav_info(dst)
        want, slack = expected_frames(t)
        if params[:3] != (1, 2, RATE_OUT) or abs(frames - want) > slack:
            problems.append(f"{cue}: decoded {params[:3]} with {frames} frames; expected mono PCM16 24 kHz with {want} (kept, not usable)")
        records.append({"cue": cue, "text": inputs["script"][cue], "source": t["rel"], "source_sha256": t["sha256"],
                        "source_pcm_sha256": t["pcm"], "source_frames": t["frames"], "source_rate": t["rate"],
                        "downloads_receipt": t.get("receipt"), "wav": dst, "wav_sha256": sha_file(dst), "samples": frames, "rate": RATE_OUT})
    receipt = {"source": b["voice"]["receipt_source"], "voice": b["voice"]["name"], "provider": b["voice"]["provider"],
               "production": production, "binding": binding_summary(b), "tool": f"{TOOL} decode",
               "conversion": "Complete recording to mono 24 kHz PCM16 by ffmpeg; no cuts, time or pitch processing.",
               "ffmpeg": {"version": version, "arguments": FFMPEG_ARGS + ["-i", "SOURCE"] + FFMPEG_OUT + ["TARGET"]},
               "selection_sha256": sha_file(args.selection), "chapter": args.chapter, "records": records,
               "claim": ("TEST ONLY, historical recordings. " if not production else "") + "Format conversion only. No take accepted, heard or timed here."}
    write_new(f"{out}/decode-receipt.json", dumps(receipt))
    if problems:
        raise Refused("\n".join(problems))
    print(f"decoded {len(records)} complete recordings into {out}; receipt {out}/decode-receipt.json")
    return 0


# ----------------------------------------------------------------------------- prepare

def import_assembler(path, tag):
    spec = importlib.util.spec_from_file_location(f"assembler_{tag}", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def dry_run(workspace_tool, b, spans_path, chapter):
    """The spans through the same assembler code root will run, writing nothing."""
    from pathlib import Path
    mod = import_assembler(workspace_tool, "dry")
    script, units = mod.read_units(Path(b["script"]), Path(b["units"]), True)
    manifest = json.loads(Path(spans_path).read_text())
    selected = [u for u in units if not chapter or u["line_ids"][0].startswith(chapter + "_")]
    if [r["id"] for r in manifest["units"]] != [u["id"] for u in selected]:
        raise ValueError("span manifest omits, repeats or reorders selected units")
    mod.check_boundary_review(manifest.get("boundary_review"))
    directions = mod.review_directions(mod.direction_contract(Path(b["script"]), json.loads(Path(b["units"]).read_text())), selected, manifest)
    lines = {}
    for unit, record in zip(selected, manifest["units"]):
        for cue, row, _pcm in mod.unit_spans(unit, script, record):
            lines.setdefault(cue, []).append(row)
    mod.check_dispositions(directions, lines)
    return len(lines)


def cmd_prepare(b, args):
    production = test_gate(b, args)
    files, questions, recordings, inputs = build_report(b)
    sel, chosen = load_selection(b, args.selection, inputs, args.chapter)
    applied = check_answers(b, args.answers, questions, recordings, chosen)
    receipt_path = f"{norm(args.decoded).rstrip('/')}/decode-receipt.json"
    receipt = load_json(receipt_path)
    rows = {r["cue"]: r for r in receipt["records"]}
    problems = []
    if receipt.get("source") != b["voice"]["receipt_source"] or receipt.get("binding", {}).get("sha256") != b["sha256"] \
            or list(rows) != list(chosen) or receipt.get("selection_sha256") != sha_file(args.selection) \
            or receipt.get("chapter") != args.chapter or receipt.get("production") != production:
        problems.append("the decode receipt is not this binding's decode of this selection")
    by_file = {(e["cue"], e["file"]): e for e in recordings}
    unit_ids = {u["line_ids"][0]: u["id"] for u in load_json(b["units"])["units"]}
    units, plan_cues, marks_plan = [], {}, {}
    for cue, t in chosen.items():
        row = rows.get(cue)
        if row is None:
            continue
        params, frames, _ = wav_info(row["wav"])
        want, slack = expected_frames(t)
        if (row["source"] != t["rel"] or row["source_sha256"] != t["sha256"] or row["text"] != inputs["script"][cue]
                or sha_file(row["wav"]) != row["wav_sha256"] or params[:3] != (1, 2, RATE_OUT) or frames != row["samples"]
                or abs(frames - want) > slack):
            problems.append(f"{cue}: the decoded file, its receipt row and the selected recording disagree")
            continue
        entry = by_file[(cue, os.path.basename(t["rel"]))]
        try:
            starts, ends, marks = resolved_points(entry, applied, frames)
        except Refused as e:
            problems.append(str(e))
            continue
        units.append({"id": unit_ids[cue], "wav": row["wav"], "sha256": row["wav_sha256"], "generation_receipt": receipt_path,
                      "sentence_ends": ends, "gap_after_samples": [0] * len(ends)})
        plan_cues[cue] = {"recording": entry["file"], "recording_path": t["rel"], "recognition": entry["recognition"],
                          "sentence_starts_24k": [{"sample": s, "basis": why} for s, why in starts]}
        voice_q = next((q for q in questions if q["kind"] == "voice" and q["files"] == [entry["file"]]), None)
        if voice_q:
            plan_cues[cue]["voice_provenance"] = {"question": voice_q["id"], "blocking": voice_q["blocking"],
                                                  "receipt": voice_q["provenance"], "root_answer": applied.get(voice_q["id"])}
        marks_plan[cue] = marks
    if problems:
        raise Refused("\n".join(problems))
    dispositions =[d for d in load_json(b["dispositions"])["direction_dispositions"] if d["cue"] in chosen]
    out = make_out(b, args.out)
    basis = (("TEST ONLY: historical recordings, not production audio. " if not production else "")
             + f"Complete {b['voice']['provider']} {b['voice']['name']} recordings, converted to 24 kHz only. Sentence ends are the "
             "recognizer's word onsets (estimates; an onset can sit at the start of the pause before its word) except where root's "
             "answers give samples. No samples removed, no silence inserted, no gain, time or pitch processing.")
    spans = {"schema_version": 1, "units_sha256": b["pins"][b["units"]], "units": units, "direction_dispositions": dispositions,
             "boundary_review": {"reviewer": args.reviewer, "basis": basis, "listening": "unreviewed"}}
    write_new(f"{out}/spans.json", json.dumps(spans, indent=2) + "\n")
    copies = ((b["assembler"], "tools/coherent_narration.py"), (b["assembler_cache"], "tools/narration_cache.py"), (b["map"], b["map_in_tree"]))
    for src, dst in copies:
        data = open(src, "rb").read()
        if sha_bytes(data) != b["pins"][src]:
            raise Stale(f"{src} changed during prepare")
        write_new(f"{out}/workspace/{dst}", data)
    tool = f"{out}/workspace/tools/coherent_narration.py"
    try:
        cues = dry_run(tool, b, f"{out}/spans.json", args.chapter)
    except (ValueError, KeyError, TypeError, OSError) as e:
        hint = ""
        if repr(b["voice"]["receipt_source"]) not in open(tool, encoding="utf8").read():
            hint = (f"\nThe pinned assembler's external-take branch does not name the receipt source {b['voice']['receipt_source']!r}, "
                    "so it read the decode receipt as a native FrankenTTS receipt. Root decides whether to extend that branch.")
        write_new(f"{out}/REFUSED.txt", f"The assembler's own checks refused these spans: {type(e).__name__}: {e}{hint}\n"
                  "Nothing was assembled. Keep this folder as evidence and prepare again into a new one.\n")
        raise Refused(f"the assembler's own checks refused these spans ({type(e).__name__}: {e}){hint}\nSee {out}/REFUSED.txt")
    chapter_args = ["--chapter", args.chapter] if args.chapter else []
    assemble = ["python3", "-I", "-B", tool, "assemble", "--script", b["script"], "--units", b["units"], "--require-directions",
                *chapter_args, "--spans", f"{out}/spans.json", "--out", f"{out}/narration"]
    timings = ["python3", "-I", "-B", TOOL, "--binding", b["path"], "timings", "--prepared", out, "--narration", f"{out}/narration",
               "--out", f"{out}-timings"] + ([] if production else ["--test-only"])
    write_new(f"{out}/assemble-command.json", dumps({
        "run_from": "repository root", "argv": assemble, "then": timings,
        "why_a_workspace": ("The units' direction contract names its map repository-relative, and coherent_narration.py resolves it "
                            "against its own tree (REPO = the tool's parent folder). Canonical holds another script and no copy of "
                            "this map, so the repository's tools/ copy cannot assemble this candidate. The workspace holds byte "
                            "copies of the two tools and the candidate's map; every other path resolves from the repository root."),
        "dry_run": f"prepare ran these spans through the workspace copy's read_units, direction checks and unit_spans: {cues} cues accepted, nothing written"}))
    write_new(f"{out}/phrase-marks-plan.json", dumps({
        "claim": "Not a phrase-timings.json: the assembled WAV hashes do not exist yet. `timings` writes the real file after assembly.",
        "cues": marks_plan}))
    write_new(f"{out}/prepare-receipt.json", dumps({
        "tool": f"{TOOL} prepare", "binding": binding_summary(b), "production": production, "pins": b["pins"], "chapter": args.chapter,
        "selection": {"sha256": sha_file(args.selection), "by": sel["by"], "basis": sel["basis"], "takes": sel["takes"]},
        "answers": {"sha256": sha_file(args.answers) if args.answers else None, "applied": applied},
        "decode_receipt": {"path": receipt_path, "sha256": sha_file(receipt_path)},
        "spans_sha256": sha_file(f"{out}/spans.json"), "cues": plan_cues,
        "claim": ("TEST ONLY, historical recordings. " if not production else "")
                 + "Source-bound preparation. Selection and answers are root's; nothing here was heard, measured or accepted."}))
    print(f"prepared {len(units)} units into {out}; dry run accepted {cues} cues; next, from the repository root: {' '.join(assemble)}")
    return 0


# ----------------------------------------------------------------------------- kit mirror and timings

KIT_NUMBERS = {w: str(i) for i, w in enumerate(ONES)}
DASHES = "-‐‑–—"


def kit_normalize(text):
    """narration.rs check_report_script's normalize, mirrored."""
    chars, out = list(text), []
    for i, c in enumerate(chars):
        prev = chars[i - 1] if i else None
        nxt = chars[i + 1] if i + 1 < len(chars) else None
        between = c in DASHES and prev is not None and prev.isalpha() and nxt is not None and nxt.isalpha()
        out.append(" " if between else c)
    words = []
    for w in "".join(out).split():
        keep = lambda ch: ch.isalnum() or ch in "-+−"
        lo, hi = 0, len(w)
        while lo < hi and not keep(w[lo]):
            lo += 1
        while hi > lo and not keep(w[hi - 1]):
            hi -= 1
        if w[lo:hi]:
            words.append(KIT_NUMBERS.get(w[lo:hi].lower(), w[lo:hi]))
    return " ".join(words)


def kit_mirror(narration, script, cues):
    """The kit loader's report checks (narration.rs check_report_script, report_starts, load), mirrored."""
    problems = []
    report = load_json(f"{narration}/report.json")
    rate = report["meta"]["rate"]
    for cue in cues:
        rows = report["lines"].get(cue)
        spans = kit_sentences(script[cue])
        if rows is None or len(rows) != len(spans):
            problems.append(f"{cue}: report has {None if rows is None else len(rows)} sentences, the script {len(spans)}")
            continue
        for k, ((a, z), row) in enumerate(zip(spans, rows)):
            if kit_normalize(script[cue][a:z].strip()) != kit_normalize(row["sentence"]):
                problems.append(f"{cue} sentence {k}: report text differs from the script")
        total = sum(r["out_samples"] + r["gap_after_samples"] for r in rows)
        params, frames, _ = wav_info(f"{narration}/{cue}.wav")
        if abs(total / rate - frames / params[2]) >= 0.05:
            problems.append(f"{cue}: the report accounts for {total / rate:.3f} s, the WAV is {frames / params[2]:.3f} s")
    return problems


def cmd_timings(b, args):
    production = test_gate(b, args)
    prepared = norm(args.prepared).rstrip("/")
    pr = load_json(f"{prepared}/prepare-receipt.json")
    plan = load_json(f"{prepared}/phrase-marks-plan.json")["cues"]
    spans_path = f"{prepared}/spans.json"
    if pr["binding"]["sha256"] != b["sha256"] or pr["production"] != production:
        raise Refused("the prepared folder was made from another binding; timings never carry across bindings or voices")
    if sha_file(spans_path) != pr["spans_sha256"]:
        raise Refused("spans.json changed since prepare")
    for src, dst in ((b["assembler"], "tools/coherent_narration.py"), (b["assembler_cache"], "tools/narration_cache.py"), (b["map"], b["map_in_tree"])):
        if sha_file(f"{prepared}/workspace/{dst}") != b["pins"][src]:
            raise Refused(f"workspace copy {dst} changed")
    chapter = pr["chapter"]
    verify = ["python3", "-I", "-B", f"{prepared}/workspace/tools/coherent_narration.py", "verify", args.narration,
              "--script", b["script"], "--units", b["units"], "--require-directions", *(["--chapter", chapter] if chapter else [])]
    r = subprocess.run([sys.executable] + verify[1:], capture_output=True, text=True)
    if r.returncode != 0:
        raise Refused(f"coherent_narration verify failed: {(r.stderr or r.stdout).strip()[-600:]}")
    narration = norm(args.narration).rstrip("/")
    assembly = load_json(f"{narration}/assembly.json")
    if assembly["manifest"] != load_json(spans_path):
        raise Refused("the assembly was not made from this spans.json")
    script = read_script(b["script"])
    cues = [c for c in script if c in pr["cues"]]
    problems = kit_mirror(narration, script, cues)
    cue_of = {u["id"]: u["line_ids"][0] for u in load_json(b["units"])["units"]}
    decoded = {cue_of[u["id"]]: u for u in load_json(spans_path)["units"]}
    if list(decoded) != cues:
        raise Refused("the spans and the prepare receipt name different cues")
    entries = {}
    for cue in cues:
        _p, frames, pcm = wav_info(f"{narration}/{cue}.wav")
        _q, _f, pcm_src = wav_info(decoded[cue]["wav"])
        if pcm != pcm_src:
            problems.append(f"{cue}: the assembled PCM is not the decoded recording sample for sample")
        length = frames / RATE_OUT
        marks, prev_t, prev_b = [], 0.0, -1
        for m in plan.get(cue, []):
            if not (0 <= m["seconds"] < length) or m["seconds"] < prev_t or m["byte"] <= prev_b:
                problems.append(f"{cue}: the mark at byte {m['byte']} ({m['seconds']} s) breaks the kit's order or range rule")
            prev_t, prev_b = m["seconds"], m["byte"]
            marks.append({"byte": m["byte"], "seconds": m["seconds"], "basis": m["basis"]})
        entries[cue] = {"wav_sha256": sha_file(f"{narration}/{cue}.wav"), "text_sha256": sha_bytes(script[cue].encode()), "marks": marks}
    if problems:
        raise Refused("\n".join(problems))
    complete = cues == list(script)
    name = "phrase-timings.json" if complete else f"phrase-timings-fragment-{chapter}.json"
    out = make_out(b, args.out)
    timings = {"basis": "automatic recognition estimates (whisper-cpp word onsets) unless a mark's basis names root's sample; not heard boundaries",
               "voice": b["voice"]["name"], "production": production, "binding_sha256": b["sha256"], "script_sha256": b["pins"][b["script"]],
               "prepare_receipt_sha256": sha_file(f"{prepared}/prepare-receipt.json"),
               **({} if complete else {"fragment": f"chapter {chapter} only; the kit needs every script cue, so merge before installing"}),
               "cues": entries}
    write_new(f"{out}/{name}", dumps(timings))
    write_new(f"{out}/timings-receipt.json", dumps({
        "tool": f"{TOOL} timings", "binding": binding_summary(b), "production": production, "verified": " ".join(verify),
        "assembly_sha256": sha_file(f"{narration}/assembly.json"), "report_sha256": sha_file(f"{narration}/report.json"),
        "cues": len(entries), "marks": sum(len(e["marks"]) for e in entries.values()), "file": name,
        "install": f"cp -n {out}/{name} {narration}/phrase-timings.json" if complete else "merge fragments for every chapter of one voice first",
        "claim": ("TEST ONLY, historical recordings. " if not production else "") + "Technical checks and recognizer estimates; no listening, fine sync or render."}))
    print(f"{name} for {len(entries)} cues ({sum(len(e['marks']) for e in entries.values())} marks) in {out}")
    return 0


# ----------------------------------------------------------------------------- report CLI and main

def cmd_report(b, args):
    files, questions, _recordings, _inputs = build_report(b)
    if args.check:
        stale = [n for n, data in files.items()
                 if not os.path.isfile(f"{args.check}/{n}") or open(f"{args.check}/{n}", "rb").read() != data.encode("utf8")]
        if stale:
            print("STALE: " + ", ".join(stale))
            return 1
        print(f"in step: {args.check} ({len(files)} files, {len(questions)} questions)")
        return 0
    out = make_out(b, args.out)
    for name, data in files.items():
        write_new(f"{out}/{name}", data)
    cov = json.loads(files["coverage.json"])
    c = cov["counts"]
    print(f"report ({b['use']}, {b['voice']['name']}): {cov['recordings']} recordings of {cov['cues']} cues; "
          f"sentence onsets {c.get('sentence_onsets_supported', 0)}/{c.get('sentence_onsets', 0)} supported; "
          f"phrase anchor marks {c.get('phrase_anchor_marks_supported', 0)}/{c.get('phrase_anchor_marks', 0)} supported; {len(questions)} questions")
    return 0


def main():
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0], formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument("--repo", default=".", help="the repository root (default: the current folder)")
    ap.add_argument("--binding", required=True, help="repository-relative audio-preparation-binding-v1 file")
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
    d.add_argument("--test-only", action="store_true")
    p = sub.add_parser("prepare")
    p.add_argument("--selection", required=True)
    p.add_argument("--answers")
    p.add_argument("--decoded", required=True, help="the decode OUT folder")
    p.add_argument("--reviewer", required=True)
    p.add_argument("--out", required=True)
    p.add_argument("--chapter")
    p.add_argument("--test-only", action="store_true")
    t = sub.add_parser("timings")
    t.add_argument("--prepared", required=True)
    t.add_argument("--narration", required=True)
    t.add_argument("--out", required=True)
    t.add_argument("--test-only", action="store_true")
    args = ap.parse_args()
    if getattr(args, "chapter", None) is not None and not re.fullmatch(r"\d\d", args.chapter):
        usage("--chapter is two digits")
    os.chdir(args.repo)
    for name in ("check", "decoded", "prepared", "narration", "selection", "answers"):
        v = getattr(args, name, None)
        if v is not None and (os.path.isabs(v) or ".." in norm(v).split("/")):
            usage(f"--{name} must be repository-relative")
    try:
        b = load_binding(args.binding)
        return {"report": cmd_report, "decode": cmd_decode, "prepare": cmd_prepare, "timings": cmd_timings}[args.cmd](b, args)
    except Stale as e:
        print("STALE:\n" + str(e))
        return 3
    except Refused as e:
        print("REFUSED:\n" + str(e))
        return 1


if __name__ == "__main__":
    sys.exit(main())
