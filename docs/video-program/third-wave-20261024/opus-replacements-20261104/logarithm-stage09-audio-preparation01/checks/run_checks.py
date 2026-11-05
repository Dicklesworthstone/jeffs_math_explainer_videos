#!/usr/bin/env python3
"""Checks for prepare_audio.py with the stage09 binding. Run from the repository root:

    python3 -I -B <bundle>/checks/run_checks.py FRESH_SCRATCH_FOLDER > transcript.txt

Everything is written under FRESH_SCRATCH_FOLDER (it must not exist): mirrors of the repository
holding every input as a read-only symlink (where a test says so, one tampered copy stands in for
one link), test-only bindings, fixtures and outputs. Nothing is written to the repository. Real
recordings are never decoded: the end-to-end runs use a labelled stand-in decoder that keeps every
other sample. Real ffmpeg is run only on synthetic tones, with the helper's own arguments. The
selections and answers here are TEST FIXTURES: nobody chose or heard anything. The real assembler
copy assembles in the mirror.

A  pins: the bundled report is in step with the real inputs.
B  the report regenerates byte for byte, twice, and equals the bundled report.
C  an independent recomputation: the stage08 to stage09 text change, inventories, superseded and
   selectable recordings, requests and submissions, voice names, recognition coverage, trigram
   onsets and marks, the four new transcripts against the exact stage09 texts, the question set.
D  end to end in the mirror: full film (test binding), chapter 01 (production binding), chapter 04
   with a fixture replacement take, a superseded-wording take and a content question bound to the
   current text, plus planted faults.
G  stale stage08 selections, answers, receipts and pins; the original 04_4/08_5 takes by path and
   by name; forged inventories and requests; changed inventory, request, downloads receipt and
   recording; missing pins; full coverage, and the 78 unchanged cues against the stage08 report.
E  root's ffmpeg arguments on synthetic tones: decoded frame counts.
F  hygiene: no voice name in the helper; no absolute home paths in the bundle.
"""

import difflib
import hashlib
import importlib.util
import json
import os
import re
import shutil
import struct
import subprocess
import sys
import wave

REPO = os.getcwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
HERE = f"{P}/logarithm-stage09-audio-preparation01"
HELPER = f"{HERE}/prepare_audio.py"
BINDING = f"{HERE}/bindings/logarithm-stage09-oliver.json"
REPORT = f"{HERE}/report"
OLD_BINDING = f"{P}/logarithm-stage08-audio-preparation01/bindings/logarithm-stage08-oliver.json"
OLD_REPORT = f"{P}/logarithm-stage08-audio-preparation01/report"
VOICE_MARK = "Oliver - Clean, British and Steady"
CHANGED = ("04_4", "08_5")
NUMBER_WORDS = set("zero one two three four five six seven eight nine ten point".split())
SHARED = set()  # file names a selection must give as paths (from the bundled report)
RESULTS = []


def sha_bytes(b):
    return hashlib.sha256(b).hexdigest()


def sha_file(p):
    with open(p, "rb") as f:
        return sha_bytes(f.read())


def check(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}" + (f" — {detail}" if detail else ""))


def run(args, cwd=None, env=None):
    r = subprocess.run([sys.executable, "-I", "-B", os.path.join(REPO, HELPER), *args], cwd=cwd or REPO,
                       capture_output=True, text=True, env=env)
    return r.returncode, (r.stdout + r.stderr)


def clean(text):
    return text.replace(SCRATCH, "<SCRATCH>").replace(REPO, "<REPO>")


def last(out):
    return clean(out.strip().splitlines()[-1] if out.strip() else "")


def script_rows(path):
    return {c: t for c, t in (r.split("\t", 1) for r in open(path, encoding="utf8").read().splitlines() if r.strip() and not r.startswith("#"))}


def key(path):
    """A recording as takes-folder/file name: the clarity takes reuse the superseded takes' file names."""
    return "/".join(path.replace("\\", "/").split("/")[-2:])


# ----------------------------------------------------------------------------- mirror

def link(mroot, rel):
    dst = os.path.join(mroot, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if not os.path.lexists(dst):
        os.symlink(os.path.join(REPO, rel), dst)


def build_mirror(mroot):
    """Links every input; a file already present (a tampered copy) is left in place."""
    b = json.load(open(BINDING))
    paths = set(b["pins"]) | {BINDING, HELPER}
    for inv in b["inventories"]:
        for t in json.load(open(inv["path"]))["takes"]:
            paths.add(f"{inv['takes_dir']}/{os.path.basename(t['wav'])}")
            for k in ("take_receipt", "receipt_hash_of", "request"):
                if inv.get(k):
                    paths.add(inv[k].format(takes_dir=inv["takes_dir"], cue=t["cue"]))
    for p in sorted(paths):
        link(mroot, p)
    return b, len(paths)


def write_json(path, value):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "x", encoding="utf8") as f:
        f.write(json.dumps(value, indent=1, ensure_ascii=False) + "\n")


def variant(mroot, name, b, **changes):
    data = json.loads(json.dumps(b))
    data.update(changes)
    rel = f"{HERE}/bindings/{name}"
    write_json(os.path.join(mroot, rel), data)
    return rel


# ----------------------------------------------------------------------------- independent recomputation

def words2(text):
    """A second tokenizer: letters and apostrophes only."""
    return re.findall(r"[a-z]+(?:'[a-z]+)*", text.lower().replace("’", "'"))


def raw_records(b):
    out = {}
    for run_ in b["recognition"]:
        for line in open(f"{run_['job']}/command.log", encoding="utf8"):
            if line.startswith("{"):
                r = json.loads(line)
                if r.get("schema_version") == "franken-whisper-batch-result-v1":
                    out.setdefault(key(r["input"]), {})[run_["name"]] = r["report"]["result"]["segments"]
    return out


def trigram_onset(segments, three):
    obs = []
    for seg in segments:
        for k, w in enumerate(words2(seg["text"])):
            obs.append((w, seg["start_sec"], k))
    hits = [obs[i][1] for i in range(len(obs) - 2) if [obs[i][0], obs[i + 1][0], obs[i + 2][0]] == three and obs[i][2] == 0]
    return hits


def heard_words(segs):
    return words2(" ".join(s["text"] for s in segs))


def contains(words, phrase):
    p = phrase.split()
    return any(words[i:i + len(p)] == p for i in range(len(words) - len(p) + 1))


def section_c(b, anchors_doc, questions_doc, coverage_doc, diffs_doc):
    script = script_rows(f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv")
    old = json.load(open(OLD_BINDING))
    old_script = script_rows(f"{old['tree']}/explainer_corpus/{old['topic']}/script.tsv")
    changed = sorted(c for c in script if script[c] != old_script.get(c))
    check("C0 the stage09 script changes exactly 04_4 and 08_5 of the stage08 script's 80 cues",
          changed == list(CHANGED) and list(script) == list(old_script) and len(script) == 80, f"changed {changed}")
    full, clar = b["inventories"]
    inv = {}
    for spec in b["inventories"]:
        for t in json.load(open(spec["path"]))["takes"]:
            inv[f"{spec['takes_dir']}/{os.path.basename(t['wav'])}"] = dict(t, spec=spec)
    current = {p for p, t in inv.items() if t["text_sha256"] == sha_bytes(script[t["cue"]].encode())}
    stale = set(inv) - current
    want_stale = {f"{full['takes_dir']}/{c}-download0{k}.wav" for c in CHANGED for k in (1, 2)}
    clarity = {p for p in inv if inv[p]["spec"] is clar}
    check("C1 inventories: 166 recordings; the four original 04_4/08_5 takes record the stage08 wording; 162 of the current text cover all 80 cues",
          len(inv) == 166 and stale == want_stale and all(inv[p]["text_sha256"] == sha_bytes(old_script[inv[p]["cue"]].encode()) for p in stale)
          and len(current) == 162 and {inv[p]["cue"] for p in current} == set(script)
          and coverage_doc["superseded_text_recordings"] == sorted(want_stale),
          f"{len(inv)} inventoried, {len(current)} current, superseded {sorted(key(p) for p in stale)}")
    check("C1b the four clarity recordings record the exact stage09 04_4/08_5 texts",
          len(clarity) == 4 and clarity <= current and sorted(inv[p]["cue"] for p in clarity) == ["04_4", "04_4", "08_5", "08_5"])
    recs = raw_records(b)
    anchor_runs = {r["name"] for r in b["recognition"] if r["role"] == "anchors"}
    own = {p for p in current if set(recs.get(key(p), {})) & anchor_runs}
    missing = sorted(key(p) for p in current - own)
    twins = all(any(inv[o]["pcm_sha256"] == inv[m]["pcm_sha256"] for o in own) for m in current - own)
    check("C2 coverage: every current recording has a large-v3-turbo recognition except two with a recognized sample twin",
          missing == ["ui-takes01/00_1-download03.wav", "ui-takes01/00_1-download04.wav"] and twins, f"unrecognized: {missing}")
    names = [r["name"] for r in b["recognition"]]
    by_run = {n: {k for k, v in recs.items() if n in v} for n in names}
    ck, sk = {key(p) for p in clarity}, {key(p) for p in stale}
    check("C2b the clarity run recognized exactly the four clarity recordings, and no other run recognized any of them",
          by_run["clarity"] == ck and not any(by_run[n] & ck for n in names if n != "clarity"), f"clarity run: {sorted(by_run['clarity'])}")
    runs = {r["name"]: r for r in coverage_doc["runs"]}
    mine = [len(by_run[n] & sk) for n in names]
    check("C2c the original takes' records count as superseded (independently: " + ", ".join(f"{n} {k}" for n, k in zip(names, mine)) + ")",
          mine == [runs[n]["superseded_records"] for n in names] and sum(mine) == 8 and runs["clarity"]["records"] == 4,
          json.dumps({n: [r["records"], r["superseded_records"]] for n, r in runs.items()}))
    named, unnamed = 0, []
    for cue in script:
        rec = json.load(open(f"{full['takes_dir']}/{cue}-downloads01.json"))
        for d in rec["downloads"]:
            if VOICE_MARK in os.path.basename(d["source"]):
                named += 1
            else:
                unnamed.append(os.path.basename(d["retained"]))
    check("C3 ui-takes01 download names: 160 name the voice; the two 04_3 downloads do not",
          named == 160 and sorted(unnamed) == ["04_3-download01.wav", "04_3-download02.wav"], f"named {named}, unnamed {sorted(unnamed)}")
    ok, detail = True, []
    for cue in CHANGED:
        d = f"{clar['takes_dir']}/{cue}"
        rec, req, sub = (json.load(open(f"{d}-{n}01.json")) for n in ("downloads", "request", "submission"))
        ok &= len(rec["downloads"]) == 2 and all(VOICE_MARK in os.path.basename(x["source"]) for x in rec["downloads"])
        ok &= req.get("voice") == VOICE_MARK and req.get("cue") == cue and req.get("text") == script[cue]
        ok &= sub.get("cue") == cue and sub.get("submitted_once") is True and sub.get("request_sha256") == sha_file(f"{d}-request01.json") \
            and VOICE_MARK in sub.get("observation", "") and "visibly selected" in sub.get("observation", "")
        detail.append(f"{cue}: model {req.get('model')}, stability {req.get('stability')}, similarity {req.get('similarity')}, format {req.get('format')}")
    check("C3b clarity: all four downloads name the voice; each request names it and the exact text; each submission was sent once, "
          "binds its request by hash and records the voice visibly selected (receipts, not a provider check)", ok, "; ".join(detail))
    bad = []
    for p, t in inv.items():
        spec = t["spec"]
        r = json.load(open(spec["request"].format(takes_dir=spec["takes_dir"], cue=t["cue"])))
        if r.get("cue") != t["cue"] or sha_bytes(r.get("text", "").encode()) != t["text_sha256"]:
            bad.append(key(p))
    check("C3c every inventoried recording's request names its cue and exactly its inventoried text (166 recordings)", not bad and len(inv) == 166, str(bad))
    hashes = all(t["receipt_sha256"] == sha_file((t["spec"].get("receipt_hash_of") or t["spec"]["take_receipt"]).format(
        takes_dir=t["spec"]["takes_dir"], cue=t["cue"])) for t in inv.values())
    which = {spec["path"].split("/")[-1]: sorted({"request" if t["receipt_sha256"] == sha_file(spec["request"].format(takes_dir=spec["takes_dir"], cue=t["cue"]))
                                                   else "downloads" if t["receipt_sha256"] == sha_file(spec["take_receipt"].format(takes_dir=spec["takes_dir"], cue=t["cue"]))
                                                   else "neither" for t in inv.values() if t["spec"] is spec}) for spec in b["inventories"]}
    check("C3d receipt_sha256 hashes the downloads receipt in full80 and the request in the clarity inventory, as the binding declares",
          hashes and which == {full["path"].split("/")[-1]: ["downloads"], clar["path"].split("/")[-1]: ["request"]}, json.dumps(which))
    agree = ambiguous = skipped = 0
    disagree = []
    for e in anchors_doc["recordings"]:
        segs = recs.get(key(os.path.join(os.path.dirname(e["path"]), e["recognition"]["recognized_file"])), {}).get(e["recognition"]["run"])
        text = script[e["cue"]]
        rows = [(s, text[s["char"]:]) for s in e["sentences"][1:]]
        btext = text.encode()
        rows += [(m, btext[m["byte"]:].decode()) for m in e["phrase_anchors"]]
        for row, tail in rows:
            first = re.match(r"[\s\"“‘(]*([A-Za-z’']+)[ ,;:]+([A-Za-z’']+)[ ,;:]+([A-Za-z’']+)", tail)
            if not first:
                skipped += 1
                continue
            three = [w.lower().replace("’", "'") for w in first.groups()]
            hits = trigram_onset(segs, three)
            if len(hits) != 1:
                if row["status"] == "supported":
                    ambiguous += 1
                continue
            if row["status"] == "supported" and abs(hits[0] - row["asr_start_s"]) < 1e-6:
                agree += 1
            elif row["status"] == "supported":
                disagree.append(f"{row.get('id')}: helper {row['asr_start_s']} trigram {hits[0]}")
            else:
                disagree.append(f"{row.get('id')}: helper {row['status']}, trigram found {hits[0]}")
    check("C4 trigram onsets and marks agree with every supported helper value they can check",
          not disagree and agree > 500, f"agree {agree}, no unique trigram {ambiguous}, not three plain words {skipped}, disagree {disagree[:4]}")
    s4 = [(e, s) for e in anchors_doc["recordings"] if e["cue"] == "08_5" for s in e["sentences"] if s["index"] == 4]
    s4_ok = len(s4) == 2 and all(e["recognition"]["run"] == "clarity" and s["status"] == "supported" and s["word"].lower() == "taking"
                                 and [round(h, 6) for h in trigram_onset(recs[key(e["path"])]["clarity"], ["taking", "half", "of"])] == [round(s["asr_start_s"], 6)]
                                 for e, s in s4)
    check("C5 08_5 sentence 4 ('Taking half of the lost lap') has a supported onset in both new takes, equal to an independent trigram",
          s4_ok, "; ".join(f"{e['file']} {s['asr_start_s']} s" for e, s in s4))
    new_words = {k: heard_words(recs[k]["clarity"]) for k in sorted(ck)}
    old_words = {(k, n): heard_words(v) for k in sorted(sk) for n, v in recs.get(k, {}).items()}
    base = [k for k, w in new_words.items() if "04_4" in k and contains(w, "which is why base e is called natural") and "space" not in w]
    half = [k for k, w in new_words.items() if "08_5" in k and contains(w, "taking half of the lost lap leaves a half turn") and "having" not in w]
    check("C6 both new 04_4 transcripts say 'which is why base e is called natural' (written 'base E'); neither says 'space'", len(base) == 2, str(base))
    check("C7 both new 08_5 transcripts say 'taking half of the lost lap leaves a half turn'; neither says 'having'", len(half) == 2, str(half))
    space = [k for k, w in old_words.items() if "04_4" in k[0] and contains(w, "space is called natural")]
    having = [k for k, w in old_words.items() if "08_5" in k[0] and contains(w, "having the lost lap")]
    check("C7a the superseded takes' transcripts are unchanged and kept apart: 'space is called natural' 4 times, 'having the lost lap' 4 times (2 takes x 2 models)",
          len(space) == 4 and len(having) == 4, f"{sorted(space)} {sorted(having)}")
    odd = []
    for k, heard in new_words.items():
        cue = k.split("/")[1].split("-")[0]
        want = words2(script[cue])
        raw = " ".join(s["text"] for s in recs[k]["clarity"])
        for op, a, b_, c, d in difflib.SequenceMatcher(None, want, heard, autojunk=False).get_opcodes():
            if op == "equal" or (op == "delete" and set(want[a:b_]) <= NUMBER_WORDS and re.search(r"\d", raw)):
                continue
            odd.append(f"{k}: {op} {want[a:b_]} -> {heard[c:d]}")
    helper_diffs = [d for d in diffs_doc["transcripts"] if d["run"] == "clarity"]
    check("C7b the four new transcripts match the exact stage09 texts word for word apart from numbers written as digits; the helper lists no difference for them",
          not odd and not helper_diffs, f"{odd[:4]} {helper_diffs[:2]}")
    want = {"voice:04_3:04_3-download01.wav", "voice:04_3:04_3-download02.wav",
            "provenance:00_1:00_1-download03.wav", "provenance:00_1:00_1-download04.wav"}
    got = {q["id"] for q in questions_doc["questions"]}
    check("C8 the question set is exactly the four expected questions (no 04_4/08_5 question remains)", got == want,
          f"extra {sorted(got - want)}, missing {sorted(want - got)}")
    advisory = sorted(q["id"] for q in questions_doc["questions"] if not q["blocking"])
    observed = all("Two matching Oliver provider history entries" in " ".join(q["provenance"]["observation"])
                   for q in questions_doc["questions"] if q["kind"] == "voice")
    check("C9 only the two 04_3 voice questions are advisory, each binding root's receipt observation (Mail 3725)",
          advisory == ["voice:04_3:04_3-download01.wav", "voice:04_3:04_3-download02.wav"] and observed, str(advisory))
    cq = b["content_questions"]
    check("C10 the stage08 content questions are bound to the stage08 texts and reported as not applying to stage09",
          coverage_doc["content_questions_for_other_text"] == ["04_4: base/space", "08_5: halving/having"]
          and all(c["text_sha256"] == sha_bytes(old_script[c["cue"]].encode()) for c in cq) and len(cq) == 2,
          str(coverage_doc["content_questions_for_other_text"]))


# ----------------------------------------------------------------------------- stand-in decoder and fixtures

FAKE = '''#!/usr/bin/env python3
# TEST STAND-IN for ffmpeg, scratch only: keeps every other sample of a 48 kHz mono PCM16 WAV.
import os, sys, wave
a = sys.argv[1:]
if a == ["-version"]:
    print("TEST STAND-IN decimator, not ffmpeg"); sys.exit(0)
assert a[:4] == ["-nostdin", "-n", "-v", "error"] and a[4] == "-i" and a[6:12] == ["-ac", "1", "-ar", "24000", "-c:a", "pcm_s16le"], a
src, dst = a[5], a[12]
if os.path.exists(dst):
    sys.exit(1)
with wave.open(src, "rb") as w:
    assert (w.getnchannels(), w.getsampwidth(), w.getframerate()) == (1, 2, 48000)
    pcm = w.readframes(w.getnframes())
out = b"".join(pcm[i:i + 2] for i in range(0, len(pcm), 4))
if os.environ.get("FAKE_SHORT") == os.path.basename(dst):
    out = out[:-2]
with wave.open(dst, "wb") as w:
    w.setparams((1, 2, 24000, 0, "NONE", "not compressed"))
    w.writeframes(out)
'''


def selection(b_rel, mroot, takes, by="run_checks TEST FIXTURE (not root)"):
    b = json.load(open(os.path.join(mroot, b_rel), "rb"))
    raw = open(os.path.join(mroot, b_rel), "rb").read()
    return {"schema": "audio-preparation-selection-v1", "binding_sha256": sha_bytes(raw), "script_sha256": b["pins"][f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv"],
            "by": by, "basis": "TEST FIXTURE ONLY: first file of each cue, not a choice; nothing was heard", "takes": takes}


def answers(b_rel, mroot, entries):
    raw = open(os.path.join(mroot, b_rel), "rb").read()
    fixture = {"by": "run_checks TEST FIXTURE (not root)", "basis": "TEST FIXTURE ONLY: nobody listened; exercises the answer path"}
    return {"schema": "audio-preparation-answers-v1", "binding_sha256": sha_bytes(raw),
            "answers": {k: dict(fixture, **v) for k, v in entries.items()}}


def fixture_replacement(mroot, b, test_b):
    """A third inventory: one take of older wording (superseded), one replacement of the current 04_4
    text recognized by its own fixture run (the real clarity 04_4-download02 record's times + 0.01 s),
    and one whose download name never names the voice. A content question is bound to the current
    04_4 text so that it applies to every recording of that text."""
    takes_dir = "fixture/takes"
    real = b["inventories"][1]["takes_dir"]
    os.makedirs(os.path.join(mroot, takes_dir))
    os.symlink(os.path.join(REPO, real, "04_4-download01.wav"), os.path.join(mroot, takes_dir, "04_4-oldtext-fixture01.wav"))
    os.symlink(os.path.join(REPO, real, "04_4-download02.wav"), os.path.join(mroot, takes_dir, "04_4-replacement-fixture01.wav"))
    os.symlink(os.path.join(REPO, real, "04_4-download01.wav"), os.path.join(mroot, takes_dir, "04_4-unnamed-fixture01.wav"))
    script = script_rows(f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv")
    names = ["04_4-oldtext-fixture01.wav", "04_4-replacement-fixture01.wav", "04_4-unnamed-fixture01.wav"]
    write_json(os.path.join(mroot, takes_dir, "04_4-downloads01.json"), {
        "note": "TEST FIXTURE, scratch only", "downloads": [{"source": f"ElevenLabs_fixture_{VOICE_MARK if n != names[2] else ''}_s50.wav",
                                                              "retained": f"{takes_dir}/{n}"} for n in names]})
    shas = [sha_file(os.path.join(mroot, takes_dir, n)) for n in names]
    write_json(os.path.join(mroot, "fixture/inventory.json"), {"note": "TEST FIXTURE", "script_sha256": "fixture", "selection": None, "takes": [
        {"cue": "04_4", "wav": f"{takes_dir}/{names[0]}", "sha256": shas[0], "text_sha256": sha_bytes(b"An older wording of 04_4 (fixture).")},
        {"cue": "04_4", "wav": f"{takes_dir}/{names[1]}", "sha256": shas[1], "text_sha256": sha_bytes(script["04_4"].encode())},
        {"cue": "04_4", "wav": f"{takes_dir}/{names[2]}", "sha256": shas[2], "text_sha256": sha_bytes(script["04_4"].encode())}]})
    source_run = next(r for r in b["recognition"] if r["name"] == "clarity")
    rec = next(json.loads(line) for line in open(f"{source_run['job']}/command.log", encoding="utf8")
               if line.startswith("{") and key(json.loads(line).get("input", "")) == key(f"{real}/04_4-download02.wav"))
    rec["input"] = f"{takes_dir}/{names[1]}"
    for seg in rec["report"]["result"]["segments"]:
        seg["start_sec"] = round(seg["start_sec"] + 0.01, 6)
        seg["end_sec"] = round(seg["end_sec"] + 0.01, 6)
    job = "fixture/recognition-job"
    os.makedirs(os.path.join(mroot, job))
    with open(os.path.join(mroot, job, "command.log"), "x", encoding="utf8") as f:
        f.write(json.dumps(rec) + "\n")
    receipt = json.load(open(f"{source_run['job']}/resource-receipt.json"))
    cmd = receipt["command"]
    cmd[cmd.index("--inputs-from") + 1] = "fixture/recognition/inputs.txt"
    write_json(os.path.join(mroot, job, "resource-receipt.json"), {k: receipt[k] for k in ("command", "exit", "admitted", "preflight_only", "inputs_sha256", "final_inputs_sha256")})
    write_json(os.path.join(mroot, "fixture/recognition/frozen-inputs.json"), {"files": [{"path": f"{takes_dir}/{names[1]}", "sha256": shas[1]}]})
    data = json.load(open(os.path.join(mroot, test_b)))
    data["why"] = "TEST FIXTURE binding, scratch only: adds a replacement take, a superseded-wording take and a current-text content question."
    data["inventories"].append({"path": "fixture/inventory.json", "takes_dir": takes_dir, "take_receipt": "{takes_dir}/{cue}-downloads01.json"})
    data["recognition"].append({"name": "fixture-replacement", "role": "anchors", "model": "ggml-large-v3-turbo.bin", "job": job,
                                "frozen": "fixture/recognition/frozen-inputs.json", "review": None})
    data["content_questions"].append({"cue": "04_4", "text_sha256": sha_bytes(script["04_4"].encode()), "script_word": "base", "heard": "space",
                                      "note": "TEST FIXTURE: a content question bound to the current 04_4 text."})
    for p in ("fixture/inventory.json", f"{job}/command.log", f"{job}/resource-receipt.json", "fixture/recognition/frozen-inputs.json"):
        data["pins"][p] = sha_file(os.path.join(mroot, p))
    rel = f"{HERE}/bindings/fixture-replacement.json"
    write_json(os.path.join(mroot, rel), data)
    return rel


def first_takes(anchors_doc, chapter=None, overrides=None):
    """The first recording of each cue, by file name, or by path where the name is shared with a superseded recording."""
    takes = {}
    for e in anchors_doc["recordings"]:
        if (chapter is None or e["cue"].startswith(chapter + "_")) and e["cue"] not in takes:
            takes[e["cue"]] = e["path"] if e["file"] in SHARED else e["file"]
    takes.update(overrides or {})
    return takes


def entry_for(anchors_doc, cue, name):
    return next(e for e in anchors_doc["recordings"] if e["cue"] == cue and name in (e["file"], e["path"]))


# ----------------------------------------------------------------------------- D: end to end

def section_d(b, mroot, test_b, anchors_doc, env):
    M = lambda *a: run(["--repo", mroot, *a], env=env)
    sel_dir = os.path.join(mroot, "fixtures")
    takes = first_takes(anchors_doc)
    clar = b["inventories"][1]["takes_dir"]
    write_json(os.path.join(sel_dir, "selection-full.json"), selection(test_b, mroot, takes))
    s1 = next(s for s in entry_for(anchors_doc, "01_1", takes["01_1"])["sentences"] if s["index"] == 1)
    s85 = next(s for s in entry_for(anchors_doc, "08_5", takes["08_5"])["sentences"] if s["index"] == 1)
    omit = entry_for(anchors_doc, "12_21", takes["12_21"])["phrase_anchors"][0]
    full_answers = {
        s1["id"]: {"decision": "sample", "sample_24k": s1["sample_24k"] + 240},
        s85["id"]: {"decision": "sample", "sample_24k": s85["sample_24k"] + 12000},
        omit["id"]: {"decision": "omit_mark"},
    }
    write_json(os.path.join(sel_dir, "answers-full.json"), answers(test_b, mroot, full_answers))
    check("D0 the fixture selection names 04_4 and 08_5 by their clarity paths (shared file names) and every other cue by file name",
          takes["04_4"] == f"{clar}/04_4-download01.wav" and takes["08_5"] == f"{clar}/08_5-download01.wav"
          and all("/" not in v for c, v in takes.items() if c not in CHANGED) and len(takes) == 80, f"04_4 {key(takes['04_4'])}, 08_5 {key(takes['08_5'])}")
    # gates
    code, out = M("--binding", test_b, "decode", "--selection", "fixtures/selection-full.json", "--out", "runs/gate1", "--ffmpeg", "ffmpeg-standin")
    check("D1 a historical-test binding refuses decode without --test-only (exit 2)", code == 2, last(out))
    code, out = M("--binding", BINDING, "decode", "--selection", "fixtures/selection-full.json", "--out", "runs/gate2", "--ffmpeg", "ffmpeg-standin", "--test-only")
    check("D2 a production binding refuses --test-only (exit 2)", code == 2, last(out))
    # full film, test binding
    code, out = M("--binding", test_b, "decode", "--selection", "fixtures/selection-full.json", "--out", "runs/full-decoded", "--ffmpeg", "ffmpeg-standin", "--test-only")
    check("D3 decode (stand-in) of 80 selected recordings", code == 0, clean(out.strip()))
    code, out = M("--binding", test_b, "prepare", "--selection", "fixtures/selection-full.json", "--answers", "fixtures/answers-full.json",
                  "--decoded", "runs/full-decoded", "--reviewer", "run_checks TEST", "--out", "runs/full-prepared", "--test-only")
    check("D4 prepare: answers applied, spans dry-run through the workspace assembler", code == 0, clean(out.strip().split("; next")[0]))
    if code != 0:
        print(clean(out))
        return
    cmd = json.load(open(os.path.join(mroot, "runs/full-prepared/assemble-command.json")))
    r = subprocess.run([sys.executable, *cmd["argv"][1:]], cwd=mroot, capture_output=True, text=True)
    check("D5 the real assembler copy assembles all 80 cues from the prepared command", r.returncode == 0 and '"assembled_cues": 80' in r.stdout, clean((r.stdout + r.stderr).strip()[-300:]))
    code, out = M(*[a for a in cmd["then"][4:]])
    check("D6 timings: verify, sample-for-sample, kit mirror, phrase-timings.json", code == 0, clean(out.strip()))
    tool_path = os.path.join(mroot, "runs/full-prepared/workspace/tools/coherent_narration.py")
    spec = importlib.util.spec_from_file_location("workspace_assembler", tool_path)
    mod = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(mod)
    receipt = json.load(open(os.path.join(mroot, "runs/full-decoded/decode-receipt.json")))
    voice = mod.external_ui_voice(receipt)
    try:
        mod.external_ui_voice(dict(receipt, voice="Frederick Surrey"))
        refused = False
    except ValueError as e:
        refused = "legacy Oliver receipt states another voice" in str(e)
    check("D6b the pinned assembler (root's Mail 3920 install) reads the decode receipt as legacy Oliver, exactly the bound voice, "
          "and refuses the same receipt stating another voice",
          sha_file(tool_path) == b["pins"][b["assembler"]] and voice == VOICE_MARK and refused,
          f"assembler {b['pins'][b['assembler']][:8]}…, receipt source {receipt['source']!r}, voice {receipt['voice']!r} → {voice!r}")
    spans = json.load(open(os.path.join(mroot, "runs/full-prepared/spans.json")))
    pt = json.load(open(os.path.join(mroot, "runs/full-prepared-timings/phrase-timings.json")))
    dec = {r_["cue"]: r_ for r_ in json.load(open(os.path.join(mroot, "runs/full-decoded/decode-receipt.json")))["records"]}
    plan = json.load(open(os.path.join(mroot, "runs/full-prepared/prepare-receipt.json")))["cues"]
    u11 = next(u for u in spans["units"] if u["id"] == "oliver_01_1")
    u85 = next(u for u in spans["units"] if u["id"] == "oliver_08_5")
    check("D7 spans: 80 units, zero inserted silence, root's samples applied (01_1 s1, 08_5 s1); 04_4/08_5 decoded and timed from the clarity recordings",
          len(spans["units"]) == 80 and all(set(u["gap_after_samples"]) == {0} for u in spans["units"])
          and u11["sentence_ends"][0] == s1["sample_24k"] + 240 and u85["sentence_ends"][0] == s85["sample_24k"] + 12000
          and all(dec[c]["source"] == takes[c] and plan[c]["recording_path"] == takes[c] and plan[c]["recognition"]["run"] == "clarity" for c in CHANGED)
          and spans["boundary_review"]["listening"] == "unreviewed" and spans["boundary_review"]["basis"].startswith("TEST ONLY"))
    marks = sum(len(e["marks"]) for e in pt["cues"].values())
    check("D8 phrase-timings: 80 cues, 32 marks (33 less one omitted), test-only label", len(pt["cues"]) == 80 and marks == 32 and pt["production"] is False,
          f"{len(pt['cues'])} cues, {marks} marks")
    vp = plan["04_3"].get("voice_provenance", {})
    check("D8b 04_3 prepared without a voice answer; the receipt observation is carried as provenance",
          vp.get("blocking") is False and vp.get("root_answer") is None and vp.get("receipt", {}).get("receipt_sha256"))
    # chapter 01 with the production binding
    write_json(os.path.join(sel_dir, "selection-ch01.json"), selection(BINDING, mroot, first_takes(anchors_doc, "01")))
    code, out = M("--binding", BINDING, "decode", "--selection", "fixtures/selection-ch01.json", "--chapter", "01", "--out", "runs/ch01-decoded", "--ffmpeg", "ffmpeg-standin")
    code2, out2 = M("--binding", BINDING, "prepare", "--selection", "fixtures/selection-ch01.json", "--chapter", "01", "--decoded", "runs/ch01-decoded",
                    "--reviewer", "run_checks TEST", "--out", "runs/ch01-prepared")
    ok = code == 0 and code2 == 0
    if ok:
        cmd = json.load(open(os.path.join(mroot, "runs/ch01-prepared/assemble-command.json")))
        r = subprocess.run([sys.executable, *cmd["argv"][1:]], cwd=mroot, capture_output=True, text=True)
        code3, out3 = M(*cmd["then"][4:])
        frag = os.path.join(mroot, "runs/ch01-prepared-timings/phrase-timings-fragment-01.json")
        ok = r.returncode == 0 and code3 == 0 and os.path.isfile(frag) and json.load(open(frag))["production"] is True
    check("D9 chapter 01 with the production binding (no --test-only) assembles and writes a labelled fragment", ok, clean(out2.strip().split("; next")[0]))
    # chapter 04 with the fixture replacement
    fx = fixture_replacement(mroot, b, test_b)
    code, out = M("--binding", fx, "report", "--out", "runs/fixture-report")
    cov = json.load(open(os.path.join(mroot, "runs/fixture-report/coverage.json"))) if code == 0 else {}
    fx_doc = json.load(open(os.path.join(mroot, "runs/fixture-report/anchors.json"))) if code == 0 else {"recordings": []}
    rep = next((e for e in fx_doc["recordings"] if e["file"] == "04_4-replacement-fixture01.wav"), None)
    base = next((e for e in fx_doc["recordings"] if e["path"] == f"{clar}/04_4-download02.wav"), None)
    shifted = rep and base and all(abs(x["asr_start_s"] - y["asr_start_s"] - 0.01) < 1e-6
                                   for x, y in zip(rep["sentences"][1:], base["sentences"][1:]) if x["status"] == y["status"] == "supported")
    want_old = sorted(json.load(open(os.path.join(REPORT, "coverage.json")))["superseded_text_recordings"] + ["fixture/takes/04_4-oldtext-fixture01.wav"])
    check("D10 a replacement take is timed from its own recognition run (fixture times = its twin's + 0.01 s); older wording is listed as superseded",
          code == 0 and rep is not None and rep["recognition"]["run"] == "fixture-replacement" and shifted
          and cov.get("superseded_text_recordings") == want_old, clean(out.strip()))
    fq = {q["id"]: q for q in json.load(open(os.path.join(mroot, "runs/fixture-report/questions.json")))["questions"]} if code == 0 else {}
    current44 = [f"{clar}/04_4-download01.wav", f"{clar}/04_4-download02.wav", "04_4-replacement-fixture01.wav", "04_4-unnamed-fixture01.wav"]
    check("D11 a content question bound to the current 04_4 text applies to every recording of that text, and records that no recognizer heard 'space'",
          all(f"content:04_4:base:{n}" in fq and fq[f"content:04_4:base:{n}"]["detail"].endswith("none. A transcript is not listening.") for n in current44)
          and sum(q["kind"] == "content" for q in fq.values()) == 4, str(sorted(k for k in fq if k.startswith("content"))))
    write_json(os.path.join(sel_dir, "selection-ch04-fixture.json"), selection(fx, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-replacement-fixture01.wav"})))
    write_json(os.path.join(sel_dir, "answers-ch04-fixture.json"), answers(fx, mroot, {
        "content:04_4:base:04_4-replacement-fixture01.wav": {"decision": "heard_as_script"}}))
    code, out = M("--binding", fx, "decode", "--selection", "fixtures/selection-ch04-fixture.json", "--chapter", "04", "--out", "runs/ch04-decoded", "--ffmpeg", "ffmpeg-standin", "--test-only")
    code2, out2 = M("--binding", fx, "prepare", "--selection", "fixtures/selection-ch04-fixture.json", "--answers", "fixtures/answers-ch04-fixture.json", "--chapter", "04",
                    "--decoded", "runs/ch04-decoded", "--reviewer", "run_checks TEST", "--out", "runs/ch04-prepared", "--test-only")
    ok = code == 0 and code2 == 0
    if ok:
        cmd = json.load(open(os.path.join(mroot, "runs/ch04-prepared/assemble-command.json")))
        r = subprocess.run([sys.executable, *cmd["argv"][1:]], cwd=mroot, capture_output=True, text=True)
        code3, out3 = M(*cmd["then"][4:])
        p44 = json.load(open(os.path.join(mroot, "runs/ch04-prepared/prepare-receipt.json")))["cues"]["04_4"]
        ok = r.returncode == 0 and code3 == 0 and p44["recording"] == "04_4-replacement-fixture01.wav" and p44["recognition"]["run"] == "fixture-replacement"
    check("D12 chapter 04 assembles with the replacement, timed from its own run", ok, clean(out2.strip().split("; next")[0]))
    # plants
    plants = []

    def plant(name, want_code, want_text, args, **kw):
        code, out = M(*args) if not kw else run(["--repo", mroot, *args], env=dict(env, **kw))
        ok = code == want_code and want_text in out
        plants.append(ok)
        check(f"D13 plant: {name} → exit {want_code}", ok, last(out))

    def sel_variant(name, takes=None, **changes):
        s = selection(test_b, mroot, takes or dict(first_takes(anchors_doc)))
        s.update(changes)
        write_json(os.path.join(sel_dir, name), s)
        return f"fixtures/{name}"

    def ans_variant(name, drop=None, add=None):
        e = {k: v for k, v in full_answers.items() if k != drop}
        e.update(add or {})
        write_json(os.path.join(sel_dir, name), answers(test_b, mroot, e))
        return f"fixtures/{name}"

    prep = lambda sel, ans, out, decoded="runs/full-decoded": ["--binding", test_b, "prepare", "--selection", sel, "--answers", ans, "--decoded", decoded,
                                                               "--reviewer", "t", "--out", out, "--test-only"]
    plant("selection for another binding", 1, "not audio-preparation-selection-v1",
          prep(sel_variant("p-wrong-binding.json", binding_sha256="0" * 64), "fixtures/answers-full.json", "runs/p1"))
    short = dict(first_takes(anchors_doc))
    short.pop("12_21")
    plant("selection missing a cue", 1, "must name exactly these 80 cues", prep(sel_variant("p-missing.json", short), "fixtures/answers-full.json", "runs/p2"))
    plant("a selected take rejected by root's answer", 1, "root rejected 04_3-download01.wav",
          prep("fixtures/selection-full.json", ans_variant("p-reject.json", add={"voice:04_3:04_3-download01.wav": {"decision": "reject_take"}}), "runs/p4"))
    plant("an answer naming an unknown question", 1, "unknown question or point",
          prep("fixtures/selection-full.json", ans_variant("p-unknown.json", add={"onset:99_9:s1:x.wav": {"decision": "sample", "sample_24k": 5}}), "runs/p5"))
    plant("a stage08 answer id carried over (08_5 s4 of the original 08_5-download01, by bare name)", 1,
          "answer to an unknown question or point onset:08_5:s4:08_5-download01.wav",
          prep("fixtures/selection-full.json", ans_variant("p-old-question.json", add={"onset:08_5:s4:08_5-download01.wav": {"decision": "sample", "sample_24k": 400000}}), "runs/p5b"))
    write_json(os.path.join(sel_dir, "selection-ch02.json"), selection(test_b, mroot, first_takes(anchors_doc, "02")))
    plant("decoded audio one sample short", 1, "decoded (1, 2, 24000) with",
          ["--binding", test_b, "decode", "--selection", "fixtures/selection-ch02.json", "--chapter", "02", "--out", "runs/p6", "--ffmpeg", "ffmpeg-standin", "--test-only"],
          FAKE_SHORT="02_1.wav")
    plant("an existing OUT", 2, "exists; name a new folder", prep("fixtures/selection-full.json", "fixtures/answers-full.json", "runs/full-prepared"))
    plant("OUT inside the bundle", 2, "lies inside the input folder", prep("fixtures/selection-full.json", "fixtures/answers-full.json", f"{HERE}/x"))
    data = json.load(open(os.path.join(mroot, test_b)))
    k_script = next(k for k in data["pins"] if k.endswith("script.tsv"))
    data["pins"][k_script] = "0" * 64
    stale_b = f"{HERE}/bindings/p-stale.json"
    write_json(os.path.join(mroot, stale_b), data)
    plant("a stale pin", 3, "STALE", ["--binding", stale_b, "report", "--out", "runs/p8"])
    plant("00_1-download03 selected without the provenance answer", 1, "provenance:00_1:00_1-download03.wav",
          prep(sel_variant("p-twin.json", dict(first_takes(anchors_doc), **{"00_1": "00_1-download03.wav"})), "fixtures/answers-full.json", "runs/p9"))
    write_json(os.path.join(sel_dir, "p-unnamed.json"), selection(fx, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-unnamed-fixture01.wav"})))
    plant("a take whose download and receipt never name the voice, unanswered", 1, "open question blocks the selection: voice:04_4:04_4-unnamed-fixture01.wav",
          ["--binding", fx, "prepare", "--selection", "fixtures/p-unnamed.json", "--answers", "fixtures/answers-ch04-fixture.json", "--chapter", "04",
           "--decoded", "runs/ch04-decoded", "--reviewer", "t", "--out", "runs/p10", "--test-only"])
    write_json(os.path.join(sel_dir, "p-oldtext.json"), selection(fx, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-oldtext-fixture01.wav"})))
    plant("a recording of superseded wording selected", 1, "records superseded wording",
          ["--binding", fx, "decode", "--selection", "fixtures/p-oldtext.json", "--chapter", "04", "--out", "runs/p11", "--ffmpeg", "ffmpeg-standin", "--test-only"])
    relabel = variant(mroot, "p-relabel.json", data | {"pins": json.load(open(os.path.join(mroot, test_b)))["pins"]},
                      voice=dict(data["voice"], receipt_source="ElevenLabs UI, Test Label"))
    write_json(os.path.join(sel_dir, "selection-relabel-ch01.json"), selection(relabel, mroot, first_takes(anchors_doc, "01")))
    code, out = M("--binding", relabel, "decode", "--selection", "fixtures/selection-relabel-ch01.json", "--chapter", "01", "--out", "runs/p12-decoded", "--ffmpeg", "ffmpeg-standin", "--test-only")
    plant("a receipt source the pinned assembler does not name", 1, "does not name the receipt source",
          ["--binding", relabel, "prepare", "--selection", "fixtures/selection-relabel-ch01.json", "--chapter", "01", "--decoded", "runs/p12-decoded",
           "--reviewer", "t", "--out", "runs/p12", "--test-only"])
    refused = os.path.join(mroot, "runs/p12/REFUSED.txt")
    print("     REFUSED.txt: " + (clean(open(refused).read().strip().replace("\n", " | ")) if os.path.isfile(refused) else "missing"))


# ----------------------------------------------------------------------------- G: stale bindings, old wording, inventories, coverage

def section_g(b, mroot, test_b, anchors_doc, env):
    M = lambda *a: run(["--repo", mroot, *a], env=env)
    sel_dir = os.path.join(mroot, "fixtures")
    full, clar = b["inventories"]
    old = json.load(open(OLD_BINDING))
    old_raw = open(OLD_BINDING, "rb").read()
    old_script_key = next(k for k in old["pins"] if k.endswith("/script.tsv"))
    script = script_rows(f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv")
    old_script = script_rows(old_script_key)
    old_anchors = json.load(open(f"{OLD_REPORT}/anchors.json"))

    def expect(name, want_code, want_text, args, runner=M):
        code, out = runner(*args)
        line = next((x for x in out.splitlines() if want_text in x), None)
        check(f"{name} → exit {want_code}", code == want_code and line is not None, clean(line) if line else clean(out.strip()[-400:]))
        return out

    def decode(b_rel, sel, chapter, out):
        return ["--binding", b_rel, "decode", "--selection", sel, *(["--chapter", chapter] if chapter else []), "--out", out, "--ffmpeg", "ffmpeg-standin"]

    def fixture(name, value):
        write_json(os.path.join(sel_dir, name), value)
        return f"fixtures/{name}"

    # stage08 selections, answers, receipts and pins
    old_first = {}
    for e in old_anchors["recordings"]:
        old_first.setdefault(e["cue"], e["file"])
    g1 = fixture("g1-stage08-selection.json", {"schema": "audio-preparation-selection-v1", "binding_sha256": sha_bytes(old_raw),
                                               "script_sha256": old["pins"][old_script_key], "by": "run_checks TEST FIXTURE (not root)",
                                               "basis": "TEST FIXTURE ONLY: a stage08-bound selection", "takes": old_first})
    expect("G1 a selection made for the stage08 binding (stage08 binding and script hashes)", 1,
           "the selection is not audio-preparation-selection-v1 for this binding and script", decode(BINDING, g1, None, "runs/g1"))
    g2 = fixture("g2-stage08-script.json", dict(selection(BINDING, mroot, first_takes(anchors_doc)), script_sha256=old["pins"][old_script_key]))
    expect("G2 a selection bound to the stage09 binding but the stage08 script", 1,
           "the selection is not audio-preparation-selection-v1 for this binding and script", decode(BINDING, g2, None, "runs/g2"))
    g3 = fixture("g3-stage08-answers.json", {"schema": "audio-preparation-answers-v1", "binding_sha256": sha_bytes(old_raw), "answers": {}})
    expect("G3 answers made for the stage08 binding", 1, "the answers are not audio-preparation-answers-v1 for this binding",
           ["--binding", BINDING, "prepare", "--selection", "fixtures/selection-ch01.json", "--chapter", "01", "--answers", g3,
            "--decoded", "runs/ch01-decoded", "--reviewer", "t", "--out", "runs/g3"])
    g4 = fixture("g4-selection-full.json", selection(BINDING, mroot, first_takes(anchors_doc)))
    expect("G4 a decode receipt made under another binding (the test binding's full decode)", 1,
           "the decode receipt is not this binding's decode of this selection",
           ["--binding", BINDING, "prepare", "--selection", g4, "--decoded", "runs/full-decoded", "--reviewer", "t", "--out", "runs/g4"])
    data = json.load(open(os.path.join(mroot, BINDING)))
    counts = {}
    for k in old["pins"]:
        counts[os.path.basename(k)] = counts.get(os.path.basename(k), 0) + 1
    old_by_base = {os.path.basename(k): v for k, v in old["pins"].items() if counts[os.path.basename(k)] == 1}
    swapped = []
    for k in data["pins"]:
        v = old_by_base.get(os.path.basename(k))
        if v and v != data["pins"][k]:
            data["pins"][k] = v
            swapped.append(os.path.basename(k))
    g5 = variant(mroot, "g5-stage08-pins.json", data)
    out = expect("G5 the stage09 binding carrying stage08's pins for the inputs that changed", 3, "STALE", ["--binding", g5, "report", "--out", "runs/g5"])
    print(f"     stage08 pins carried: {', '.join(sorted(swapped))}; stale lines: {sum(' is ' in x and 'pinned' in x for x in out.splitlines())}")
    # the original 04_4/08_5 takes
    old44, old85 = f"{full['takes_dir']}/04_4-download01.wav", f"{full['takes_dir']}/08_5-download02.wav"
    g6 = fixture("g6-old-0404-path.json", selection(BINDING, mroot, first_takes(anchors_doc, "04", {"04_4": old44})))
    expect("G6 the original 04_4 take named by its path", 1, f"04_4: {old44} cannot be selected; it records superseded wording", decode(BINDING, g6, "04", "runs/g6"))
    g7 = fixture("g7-old-0805-path.json", selection(BINDING, mroot, first_takes(anchors_doc, "08", {"08_5": old85})))
    expect("G7 the original 08_5 take named by its path", 1, f"08_5: {old85} cannot be selected; it records superseded wording", decode(BINDING, g7, "08", "runs/g7"))
    g8 = fixture("g8-bare-shared-name.json", selection(BINDING, mroot, first_takes(anchors_doc, "08", {"08_5": "08_5-download01.wav"})))
    expect("G8 a bare file name shared by a clarity take and a superseded take", 1,
           f"08_5: 08_5-download01.wav is also the name of a recording of superseded wording; name the recording by its path ({clar['takes_dir']}/08_5-download01.wav)",
           decode(BINDING, g8, "08", "runs/g8"))
    g8b = fixture("g8b-stage08-names.json", selection(BINDING, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-download01.wav"})))
    expect("G8b a stage08 selection re-hashed to stage09 still naming 04_4-download01.wav", 1,
           "04_4: 04_4-download01.wav is also the name of a recording of superseded wording", decode(BINDING, g8b, "04", "runs/g8b"))

    # forged inventories and requests (re-pinned, so only the content checks can refuse them)
    def forged_inventory(name, index, edit):
        inv = json.load(open(b["inventories"][index]["path"]))
        edit(inv["takes"])
        rel = f"fixture/forged/{name}"
        write_json(os.path.join(mroot, rel), inv)
        d = json.load(open(os.path.join(mroot, BINDING)))
        d["inventories"][index]["path"] = rel
        d["pins"][rel] = sha_file(os.path.join(mroot, rel))
        return variant(mroot, f"g-{name}", d)

    def set_text(name_end, text):
        def edit(takes):
            for t in takes:
                if t["wav"].endswith(name_end):
                    t["text_sha256"] = sha_bytes(text.encode())
        return edit

    g9 = forged_inventory("clarity-claims-old-text.json", 1, set_text("oliver-clarity-takes01/04_4-download01.wav", old_script["04_4"]))
    expect("G9 a re-pinned clarity inventory claiming a new 04_4 take is of the old text (its request says otherwise)", 1,
           f"request {clar['takes_dir']}/04_4-request01.json does not name its cue and inventoried text", ["--binding", g9, "report", "--out", "runs/g9"])
    g10 = forged_inventory("full80-claims-new-text.json", 0, set_text("ui-takes01/08_5-download01.wav", script["08_5"]))
    out = expect("G10 a re-pinned full80 inventory claiming an original 08_5 take is of the new text (its request says otherwise)", 1,
                 f"request {full['takes_dir']}/08_5-request01.json does not name its cue and inventoried text", ["--binding", g10, "report", "--out", "runs/g10"])
    print("     also refused: " + ("duplicate selectable file name" if "must have distinct file names" in out else "no duplicate-name line"))

    cue = next(c for c in script if c not in CHANGED and c.startswith("06_"))

    def request_set(name, mutate):
        d = f"fixture/{name}"
        for c in script:
            r = json.load(open(full["request"].format(takes_dir=full["takes_dir"], cue=c)))
            if c == cue:
                mutate(r)
            write_json(os.path.join(mroot, d, f"{c}-request01.json"), r)
        data = json.load(open(os.path.join(mroot, BINDING)))
        data["inventories"][0]["request"] = d + "/{cue}-request01.json"
        return variant(mroot, f"g-{name}.json", data)

    g11 = request_set("requests-text", lambda r: r.update(text=r["text"] + " Changed."))
    expect(f"G11 a request whose text is not its recording's inventoried text ({cue})", 1,
           f"request fixture/requests-text/{cue}-request01.json does not name its cue and inventoried text", ["--binding", g11, "report", "--out", "runs/g11"])
    g12 = request_set("requests-cue", lambda r: r.update(cue="01_1"))
    expect(f"G12 a request naming another cue ({cue} names 01_1)", 1,
           f"request fixture/requests-cue/{cue}-request01.json does not name its cue and inventoried text", ["--binding", g12, "report", "--out", "runs/g12"])

    # changed bytes under the real binding, each in its own fresh mirror
    def tampered(name, rel, mutate):
        mt = os.path.join(SCRATCH, name)
        dst = os.path.join(mt, rel)
        os.makedirs(os.path.dirname(dst))
        with open(dst, "xb") as f:
            f.write(mutate(open(rel, "rb").read()))
        build_mirror(mt)
        return lambda *a: run(["--repo", mt, *a])

    def json_edit(**fields):
        return lambda raw: (json.dumps(dict(json.loads(raw), **fields), indent=1, ensure_ascii=False) + "\n").encode()

    rep = ["--binding", BINDING, "report", "--out", "runs/r"]
    expect("G13 the clarity inventory changed by one byte (real binding)", 3, f"{clar['path']} is ",
           rep, tampered("mirror-g13", clar["path"], lambda raw: raw + b"\n"))
    req85 = f"{clar['takes_dir']}/08_5-request01.json"
    expect("G14 a clarity request changed in a field other than its text (the inventory hashes the request)", 1,
           f"receipt {req85} is missing or changed", rep, tampered("mirror-g14", req85, json_edit(state="TEST FIXTURE change")))
    dl44 = f"{clar['takes_dir']}/04_4-downloads01.json"
    expect("G15 a clarity downloads receipt changed by one byte (the binding pins it)", 3, f"{dl44} is ",
           rep, tampered("mirror-g15", dl44, lambda raw: raw + b"\n"))
    wav = f"{clar['takes_dir']}/04_4-download02.wav"
    expect("G16 a clarity recording changed in its last sample byte", 1, f"{wav} changed on disk",
           rep, tampered("mirror-g16", wav, lambda raw: raw[:-1] + bytes([raw[-1] ^ 1])))
    data = json.load(open(os.path.join(mroot, BINDING)))
    for c in CHANGED:
        data["pins"].pop(f"{clar['takes_dir']}/{c}-downloads01.json")
    g17 = variant(mroot, "g17-unpinned-downloads.json", data)
    expect("G17 the clarity downloads receipts neither hashed by their inventory nor pinned", 1, "is neither hashed by its inventory nor pinned",
           ["--binding", g17, "report", "--out", "runs/g17"])
    data = json.load(open(os.path.join(mroot, BINDING)))
    data["inventories"][1].pop("receipt_hash_of")
    g18 = variant(mroot, "g18-no-receipt-hash-of.json", data)
    expect("G18 the clarity inventory read as hashing its downloads receipts (it hashes the requests)", 1,
           f"receipt {dl44} is missing or changed", ["--binding", g18, "report", "--out", "runs/g18"])
    data = json.load(open(os.path.join(mroot, BINDING)))
    data["recognition"] = [r for r in data["recognition"] if r["name"] != "clarity"]
    g19 = variant(mroot, "g19-no-clarity-run.json", data)
    expect("G19 the binding without the clarity recognition run (the new takes would have no timing source)", 1,
           f"{clar['takes_dir']}/04_4-download01.wav: no recognition and no recognized recording with the same samples",
           ["--binding", g19, "report", "--out", "runs/g19"])

    # coverage
    rec = anchors_doc["recordings"]
    per_cue = {}
    for e in rec:
        per_cue.setdefault(e["cue"], set()).add(e["path"])
    clarity_paths = {f"{clar['takes_dir']}/{c}-download0{k}.wav" for c in CHANGED for k in (1, 2)}
    full_inv = json.load(open(full["path"]))["takes"]
    unchanged = [c for c in script if c not in CHANGED]
    check("G20 full coverage: 80 cues; 04_4 and 08_5 selectable exactly as the four clarity recordings, timed from the clarity run; "
          "each of the 78 unchanged cues keeps exactly its full80 recordings, none timed from the clarity run",
          set(per_cue) == set(script) and len(per_cue) == 80 and per_cue["04_4"] | per_cue["08_5"] == clarity_paths
          and all(e["recognition"]["run"] == "clarity" for e in rec if e["cue"] in CHANGED)
          and all(per_cue[c] == {f"{full['takes_dir']}/{os.path.basename(t['wav'])}" for t in full_inv if t["cue"] == c} for c in unchanged)
          and not any(e["recognition"]["run"] == "clarity" for e in rec if e["cue"] not in CHANGED) and len(unchanged) == 78,
          f"{len(rec)} selectable recordings")
    old_by = {(e["cue"], e["file"]): e for e in old_anchors["recordings"]}
    differ = [e["file"] for e in rec if e["cue"] not in CHANGED
              and (e["label"] != e["file"] or old_by.get((e["cue"], e["file"])) != {k: v for k, v in e.items() if k not in ("path", "label")})]
    n_same = sum(1 for e in rec if e["cue"] not in CHANGED)
    check("G21 the 158 recordings of the 78 unchanged cues have the same bytes, recognition, sentence onsets and marks as in the stage08 report",
          not differ and n_same == 158 and n_same == sum(1 for (c, _f) in old_by if c not in CHANGED), f"{n_same} compared, differ {differ[:4]}")
    labels = sorted(key(e["label"]) for e in rec if e["label"] != e["file"])
    check("G22 only the four clarity recordings carry their path in question and point ids (their file names are shared with superseded takes)",
          labels == sorted(key(p) for p in clarity_paths) and all(s.get("id", "").endswith(":" + e["path"]) for e in rec if e["cue"] in CHANGED
                                                                    for s in e["sentences"][1:] + e["phrase_anchors"]), str(labels))


# ----------------------------------------------------------------------------- E: real ffmpeg on synthetic tones

def section_e(helper):
    if not shutil.which("ffmpeg"):
        check("E0 ffmpeg present", False)
        return
    tones = os.path.join(SCRATCH, "tones")
    os.makedirs(tones)
    print("     ffmpeg: " + subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True).stdout.splitlines()[0].split(" Copyright")[0])
    for rate, n in ((48000, 96000), (48000, 1002240), (48000, 936961), (44100, 88200), (44100, 920123)):
        src = os.path.join(tones, f"tone-{rate}-{n}.wav")
        with wave.open(src, "wb") as w:
            w.setparams((1, 2, rate, 0, "NONE", "not compressed"))
            w.writeframes(b"".join(struct.pack("<h", int(8000 * ((i * 440 * 2 // rate) % 2 * 2 - 1))) for i in range(n)))
        dst = os.path.join(tones, f"tone-{rate}-{n}-24k.wav")
        r = subprocess.run(["ffmpeg", *helper.FFMPEG_ARGS, "-i", src, *helper.FFMPEG_OUT, dst], capture_output=True, text=True)
        with wave.open(dst, "rb") as w:
            got = (w.getnchannels(), w.getsampwidth(), w.getframerate(), w.getnframes())
        want, slack = helper.expected_frames({"rate": rate, "frames": n})
        check(f"E1 root's arguments, {rate} Hz × {n} frames → {got[3]} frames at 24 kHz (rule: {want} ± {slack})",
              r.returncode == 0 and got[:3] == (1, 2, 24000) and abs(got[3] - want) <= slack)


# ----------------------------------------------------------------------------- main

def main():
    global SCRATCH
    if len(sys.argv) != 2 or os.path.exists(sys.argv[1]) or not os.path.isfile(HELPER):
        print("usage: from the repository root, run_checks.py FRESH_SCRATCH_FOLDER (must not exist)", file=sys.stderr)
        return 2
    SCRATCH = os.path.realpath(os.path.dirname(os.path.abspath(sys.argv[1]))) + os.sep + os.path.basename(sys.argv[1])
    os.makedirs(SCRATCH)
    sys.dont_write_bytecode = True
    spec = importlib.util.spec_from_file_location("helper_under_test", HELPER)
    helper = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(helper)
    print(f"prepare_audio.py {sha_file(HELPER)}")
    print(f"binding {sha_file(BINDING)}")
    # A
    code, out = run(["--binding", BINDING, "report", "--check", REPORT])
    check("A1 the bundled report is in step with the real inputs (all pins current)", code == 0, clean(out.strip()))
    # B
    mroot = os.path.join(SCRATCH, "mirror")
    b, n = build_mirror(mroot)
    print(f"     mirror: {n} read-only links")
    outs = []
    for k in (1, 2):
        code, out = run(["--repo", mroot, "--binding", BINDING, "report", "--out", f"runs/report{k}"])
        outs.append(code)
        if k == 1:
            print("     " + clean(out.strip()))
    same = all(open(os.path.join(mroot, "runs/report1", f), "rb").read() == open(os.path.join(mroot, "runs/report2", f), "rb").read()
               == open(os.path.join(REPORT, f), "rb").read() for f in sorted(os.listdir(os.path.join(mroot, "runs/report1"))))
    check("B1 two fresh reports are byte-equal to each other and to the bundled report", outs == [0, 0] and same)
    anchors_doc = json.load(open(os.path.join(REPORT, "anchors.json")))
    questions_doc = json.load(open(os.path.join(REPORT, "questions.json")))
    coverage_doc = json.load(open(os.path.join(REPORT, "coverage.json")))
    diffs_doc = json.load(open(os.path.join(REPORT, "asr-differences.json")))
    SHARED.update(os.path.basename(p) for p in coverage_doc["superseded_text_recordings"])
    # C
    section_c(b, anchors_doc, questions_doc, coverage_doc, diffs_doc)
    # D
    env = dict(os.environ, PATH=os.path.join(SCRATCH, "bin") + os.pathsep + os.environ.get("PATH", ""))
    fake = os.path.join(SCRATCH, "bin", "ffmpeg-standin")
    os.makedirs(os.path.dirname(fake))
    with open(fake, "x") as f:
        f.write(FAKE)
    os.chmod(fake, 0o755)
    test_b = variant(mroot, "test-historical.json", b, use="historical-test",
                     why="TEST binding, scratch only: the delivered binding with use historical-test.")
    section_d(b, mroot, test_b, anchors_doc, env)
    # G
    section_g(b, mroot, test_b, anchors_doc, env)
    # E
    section_e(helper)
    # F
    text = open(HELPER, encoding="utf8").read()
    check("F1 the helper names no voice", not re.search(r"oliver|frederick|surrey", text, re.I))
    hits = []
    for root, _dirs, files in os.walk(HERE):
        for f in files:
            data = open(os.path.join(root, f), "rb").read()
            if any(n in data for n in (b"/Us" + b"ers/", b"/priv" + b"ate/", b"/ho" + b"me/")):
                hits.append(os.path.join(root, f))
    check("F2 no absolute home or scratch path in any bundle file", not hits, str(hits))
    print(f"{sum(RESULTS)} PASS, {len(RESULTS) - sum(RESULTS)} FAIL")
    return 0 if all(RESULTS) else 1


if __name__ == "__main__":
    sys.exit(main())
