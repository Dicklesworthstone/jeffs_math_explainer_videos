#!/usr/bin/env python3
"""Checks for prepare_audio.py. Run from the repository root:

    python3 -I -B <bundle>/checks/run_checks.py FRESH_SCRATCH_FOLDER > transcript.txt

Everything is written under FRESH_SCRATCH_FOLDER (it must not exist): a mirror of the repository
holding every input as a read-only symlink, test-only bindings, fixtures and outputs. Nothing is
written to the repository. Real recordings are never decoded: the end-to-end runs use a labelled
stand-in decoder that keeps every other sample. Real ffmpeg is run only on synthetic tones, with
the helper's own arguments. The selections and answers here are TEST FIXTURES: nobody chose or
heard anything. The real assembler copy assembles in the mirror.

A  pins: the delivered binding is in step; the bundled report is in step with the real inputs.
B  the report regenerates byte for byte, twice, and equals the bundled report.
C  an independent recomputation: coverage, text hashes, voice names, trigram onsets and marks,
   04_4 'space' and 08_5 'having' in every transcript, and the exact question set.
D  end to end in the mirror: full film (test binding), chapter 01 (production binding),
   chapter 04 with a fixture replacement take and a superseded-wording take, plus planted faults.
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
HERE = "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage08-audio-preparation01"
HELPER = f"{HERE}/prepare_audio.py"
BINDING = f"{HERE}/bindings/logarithm-stage08-oliver.json"
REPORT = f"{HERE}/report"
VOICE_MARK = "Oliver - Clean, British and Steady"
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


def script_rows(path):
    return {c: t for c, t in (r.split("\t", 1) for r in open(path, encoding="utf8").read().splitlines() if r.strip() and not r.startswith("#"))}


# ----------------------------------------------------------------------------- mirror

def link(mroot, rel):
    dst = os.path.join(mroot, rel)
    os.makedirs(os.path.dirname(dst), exist_ok=True)
    if not os.path.lexists(dst):
        os.symlink(os.path.join(REPO, rel), dst)


def build_mirror(mroot):
    b = json.load(open(BINDING))
    paths = set(b["pins"]) | {BINDING, HELPER}
    for inv in b["inventories"]:
        for t in json.load(open(inv["path"]))["takes"]:
            paths.add(f"{inv['takes_dir']}/{os.path.basename(t['wav'])}")
            paths.add(inv["take_receipt"].format(takes_dir=inv["takes_dir"], cue=t["cue"]))
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
                    out.setdefault(os.path.basename(r["input"]), {})[run_["name"]] = r["report"]["result"]["segments"]
    return out


def trigram_onset(segments, three):
    obs = []
    for seg in segments:
        for k, w in enumerate(words2(seg["text"])):
            obs.append((w, seg["start_sec"], k))
    hits = [obs[i][1] for i in range(len(obs) - 2) if [obs[i][0], obs[i + 1][0], obs[i + 2][0]] == three and obs[i][2] == 0]
    return hits


def section_c(b, anchors_doc, questions_doc):
    script = script_rows(f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv")
    inv = json.load(open(b["inventories"][0]["path"]))
    takes = {os.path.basename(t["wav"]): t for t in inv["takes"]}
    check("C1 inventory: 162 recordings of 80 cues, every requested text is the script's",
          len(takes) == 162 and {t["cue"] for t in takes.values()} == set(script)
          and all(t["text_sha256"] == sha_bytes(script[t["cue"]].encode()) for t in takes.values()))
    recs = raw_records(b)
    anchor_runs = {r["name"] for r in b["recognition"] if r["role"] == "anchors"}
    own = {n for n, v in recs.items() if set(v) & anchor_runs}
    missing = sorted(set(takes) - own)
    twins = all(any(takes[o]["pcm_sha256"] == takes[m]["pcm_sha256"] for o in own) for m in missing)
    check("C2 coverage: every recording has a large-v3-turbo recognition except two with a recognized sample twin",
          missing == ["00_1-download03.wav", "00_1-download04.wav"] and twins, f"unrecognized: {missing}")
    named, unnamed = 0, []
    for cue in script:
        rec = json.load(open(f"{b['inventories'][0]['takes_dir']}/{cue}-downloads01.json"))
        for d in rec["downloads"]:
            if VOICE_MARK in os.path.basename(d["source"]):
                named += 1
            else:
                unnamed.append(os.path.basename(d["retained"]))
    check("C3 download names: 160 name the voice; the two 04_3 downloads do not", named == 160 and sorted(unnamed) == ["04_3-download01.wav", "04_3-download02.wav"],
          f"named {named}, unnamed {sorted(unnamed)}")
    # trigram onsets, independent of the helper's alignment
    agree = ambiguous = skipped = 0
    disagree = []
    for e in anchors_doc["recordings"]:
        segs = recs.get(e["recognition"]["recognized_file"], {}).get(e["recognition"]["run"])
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
    s4 = [s for e in anchors_doc["recordings"] if e["cue"] == "08_5" for s in e["sentences"] if s["index"] == 4]
    check("C5 08_5 sentence 4 ('Halving the lost lap') has no supported onset in either take",
          len(s4) == 2 and all(s["status"] != "supported" for s in s4) and all(not trigram_onset(
              recs[f"08_5-download0{k}.wav"]["remaining"], ["halving", "the", "lost"]) for k in (1, 2)))
    space = [(f, run_) for f in ("04_4-download01.wav", "04_4-download02.wav") for run_, segs in recs[f].items()
             if "space is called natural" in " ".join(words2(" ".join(s["text"] for s in segs)))]
    having = [(f, run_) for f in ("08_5-download01.wav", "08_5-download02.wav") for run_, segs in recs[f].items()
              if "having the lost lap" in " ".join(words2(" ".join(s["text"] for s in segs)))]
    check("C6 every 04_4 transcript (2 takes x 2 models) says 'space is called natural'", len(space) == 4, str(sorted(space)))
    check("C7 every 08_5 transcript (2 takes x 2 models) says 'having the lost lap'", len(having) == 4, str(sorted(having)))
    want = {"onset:08_5:s4:08_5-download01.wav", "onset:08_5:s4:08_5-download02.wav",
            "content:04_4:base:04_4-download01.wav", "content:04_4:base:04_4-download02.wav",
            "content:08_5:halving:08_5-download01.wav", "content:08_5:halving:08_5-download02.wav",
            "voice:04_3:04_3-download01.wav", "voice:04_3:04_3-download02.wav",
            "provenance:00_1:00_1-download03.wav", "provenance:00_1:00_1-download04.wav"}
    got = {q["id"] for q in questions_doc["questions"]}
    check("C8 the question set is exactly the ten expected questions", got == want, f"extra {sorted(got - want)}, missing {sorted(want - got)}")


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
    """A second inventory: one take of older wording (superseded) and one replacement of the current
    04_4 text, recognized by its own fixture run whose times are the real 04_4-download02 record's + 0.01 s."""
    takes_dir = "fixture/takes"
    real = f"{b['inventories'][0]['takes_dir']}"
    os.makedirs(os.path.join(mroot, takes_dir))
    os.symlink(os.path.join(REPO, real, "04_4-download01.wav"), os.path.join(mroot, takes_dir, "04_4-oldtext-fixture01.wav"))
    os.symlink(os.path.join(REPO, real, "04_4-download02.wav"), os.path.join(mroot, takes_dir, "04_4-replacement-fixture01.wav"))
    script = script_rows(f"{b['tree']}/explainer_corpus/{b['topic']}/script.tsv")
    names = ["04_4-oldtext-fixture01.wav", "04_4-replacement-fixture01.wav"]
    write_json(os.path.join(mroot, takes_dir, "04_4-downloads01.json"), {
        "note": "TEST FIXTURE, scratch only", "downloads": [{"source": f"ElevenLabs_fixture_{VOICE_MARK}_pvc.wav", "retained": f"{takes_dir}/{n}"} for n in names]})
    shas = [sha_file(os.path.join(mroot, takes_dir, n)) for n in names]
    write_json(os.path.join(mroot, "fixture/inventory.json"), {"note": "TEST FIXTURE", "script_sha256": "fixture", "selection": None, "takes": [
        {"cue": "04_4", "wav": f"{takes_dir}/{names[0]}", "sha256": shas[0], "text_sha256": sha_bytes(b"An older wording of 04_4 (fixture).")},
        {"cue": "04_4", "wav": f"{takes_dir}/{names[1]}", "sha256": shas[1], "text_sha256": sha_bytes(script["04_4"].encode())}]})
    source_run = next(r for r in b["recognition"] if r["role"] == "anchors" and any(
        "04_4-download02.wav" in line for line in open(f"{r['job']}/command.log", encoding="utf8")))
    rec = next(json.loads(line) for line in open(f"{source_run['job']}/command.log", encoding="utf8")
               if line.startswith("{") and "04_4-download02.wav" in line)
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
    data["why"] = "TEST FIXTURE binding, scratch only: adds a replacement take and a superseded-wording take."
    data["inventories"].append({"path": "fixture/inventory.json", "takes_dir": takes_dir, "take_receipt": "{takes_dir}/{cue}-downloads01.json"})
    data["recognition"].append({"name": "fixture-replacement", "role": "anchors", "model": "ggml-large-v3-turbo.bin", "job": job,
                                "frozen": "fixture/recognition/frozen-inputs.json", "review": None})
    for p in ("fixture/inventory.json", f"{job}/command.log", f"{job}/resource-receipt.json", "fixture/recognition/frozen-inputs.json"):
        data["pins"][p] = sha_file(os.path.join(mroot, p))
    rel = f"{HERE}/bindings/fixture-replacement.json"
    write_json(os.path.join(mroot, rel), data)
    return rel


def first_takes(anchors_doc, chapter=None, overrides=None):
    takes = {}
    for e in anchors_doc["recordings"]:
        if (chapter is None or e["cue"].startswith(chapter + "_")) and e["cue"] not in takes:
            takes[e["cue"]] = e["file"]
    takes.update(overrides or {})
    return takes


# ----------------------------------------------------------------------------- D: end to end

def section_d(b, mroot, test_b, anchors_doc):
    env = dict(os.environ, PATH=os.path.join(SCRATCH, "bin") + os.pathsep + os.environ.get("PATH", ""))
    fake = os.path.join(SCRATCH, "bin", "ffmpeg-standin")
    os.makedirs(os.path.dirname(fake))
    with open(fake, "x") as f:
        f.write(FAKE)
    os.chmod(fake, 0o755)
    M = lambda *a: run(["--repo", mroot, *a], env=env)
    sel_dir = os.path.join(mroot, "fixtures")
    takes = first_takes(anchors_doc)
    write_json(os.path.join(sel_dir, "selection-full.json"), selection(test_b, mroot, takes))
    by_id = {e["file"]: e for e in anchors_doc["recordings"]}
    s4 = next(s for s in by_id["08_5-download01.wav"]["sentences"] if s["index"] == 4)
    s1 = next(s for s in by_id["01_1-download01.wav"]["sentences"] if s["index"] == 1)
    omit = by_id["12_21-download01.wav"]["phrase_anchors"][0]
    full_answers = {
        "onset:08_5:s4:08_5-download01.wav": {"decision": "use_candidate"} if s4.get("candidate") else {"decision": "sample", "sample_24k": 1},
        "content:04_4:base:04_4-download01.wav": {"decision": "heard_as_script"},
        "content:08_5:halving:08_5-download01.wav": {"decision": "heard_as_script"},
        "voice:04_3:04_3-download01.wav": {"decision": "confirmed_voice"},
        s1["id"]: {"decision": "sample", "sample_24k": s1["sample_24k"] + 240},
        omit["id"]: {"decision": "omit_mark"},
    }
    write_json(os.path.join(sel_dir, "answers-full.json"), answers(test_b, mroot, full_answers))
    check("D0 08_5 s4 has a recognizer candidate between agreeing runs", bool(s4.get("candidate")), json.dumps(s4.get("candidate")))
    # gates
    code, out = M("--binding", test_b, "decode", "--selection", "fixtures/selection-full.json", "--out", "runs/gate1", "--ffmpeg", "ffmpeg-standin")
    check("D1 a historical-test binding refuses decode without --test-only (exit 2)", code == 2, clean(out.strip().splitlines()[-1]))
    code, out = M("--binding", BINDING, "decode", "--selection", "fixtures/selection-full.json", "--out", "runs/gate2", "--ffmpeg", "ffmpeg-standin", "--test-only")
    check("D2 a production binding refuses --test-only (exit 2)", code == 2, clean(out.strip().splitlines()[-1]))
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
    spans = json.load(open(os.path.join(mroot, "runs/full-prepared/spans.json")))
    pt = json.load(open(os.path.join(mroot, "runs/full-prepared-timings/phrase-timings.json")))
    u11 = next(u for u in spans["units"] if u["id"] == "oliver_01_1")
    u85 = next(u for u in spans["units"] if u["id"] == "oliver_08_5")
    check("D7 spans: 80 units, zero inserted silence, last end = 24 kHz frames, root's sample and candidate applied",
          len(spans["units"]) == 80 and all(set(u["gap_after_samples"]) == {0} for u in spans["units"])
          and u11["sentence_ends"][0] == s1["sample_24k"] + 240 and u85["sentence_ends"][3] == s4["candidate"]["sample_24k"]
          and spans["boundary_review"]["listening"] == "unreviewed" and spans["boundary_review"]["basis"].startswith("TEST ONLY"))
    marks = sum(len(e["marks"]) for e in pt["cues"].values())
    check("D8 phrase-timings: 80 cues, 32 marks (33 less one omitted), test-only label", len(pt["cues"]) == 80 and marks == 32 and pt["production"] is False,
          f"{len(pt['cues'])} cues, {marks} marks")
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
    check("D9 chapter 01 with the production binding (no --test-only) assembles and writes a labelled fragment", ok, clean((out + out2).strip()[-200:]))
    # chapter 04 with the fixture replacement
    fx = fixture_replacement(mroot, b, test_b)
    code, out = M("--binding", fx, "report", "--out", "runs/fixture-report")
    cov = json.load(open(os.path.join(mroot, "runs/fixture-report/coverage.json"))) if code == 0 else {}
    fx_doc = json.load(open(os.path.join(mroot, "runs/fixture-report/anchors.json"))) if code == 0 else {"recordings": []}
    rep = next((e for e in fx_doc["recordings"] if e["file"] == "04_4-replacement-fixture01.wav"), None)
    base = next((e for e in fx_doc["recordings"] if e["file"] == "04_4-download02.wav"), None)
    shifted = rep and base and all(abs(x["asr_start_s"] - y["asr_start_s"] - 0.01) < 1e-6
                                   for x, y in zip(rep["sentences"][1:], base["sentences"][1:]) if x["status"] == y["status"] == "supported")
    check("D10 a replacement take is timed from its own recognition run (fixture times = its twin's + 0.01 s); older wording is listed as superseded",
          code == 0 and rep is not None and rep["recognition"]["run"] == "fixture-replacement" and shifted
          and cov.get("superseded_text_recordings") == ["04_4-oldtext-fixture01.wav"], clean(out.strip()))
    fq = json.load(open(os.path.join(mroot, "runs/fixture-report/questions.json")))["questions"] if code == 0 else []
    check("D11 the 04_4 content question applies to the replacement too (its transcript still says 'space')",
          any(q["id"] == "content:04_4:base:04_4-replacement-fixture01.wav" for q in fq))
    write_json(os.path.join(sel_dir, "selection-ch04-fixture.json"), selection(fx, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-replacement-fixture01.wav"})))
    write_json(os.path.join(sel_dir, "answers-ch04-fixture.json"), answers(fx, mroot, {
        "content:04_4:base:04_4-replacement-fixture01.wav": {"decision": "heard_as_script"},
        "voice:04_3:04_3-download01.wav": {"decision": "confirmed_voice"}}))
    code, out = M("--binding", fx, "decode", "--selection", "fixtures/selection-ch04-fixture.json", "--chapter", "04", "--out", "runs/ch04-decoded", "--ffmpeg", "ffmpeg-standin", "--test-only")
    code2, out2 = M("--binding", fx, "prepare", "--selection", "fixtures/selection-ch04-fixture.json", "--answers", "fixtures/answers-ch04-fixture.json", "--chapter", "04",
                    "--decoded", "runs/ch04-decoded", "--reviewer", "run_checks TEST", "--out", "runs/ch04-prepared", "--test-only")
    ok = code == 0 and code2 == 0
    if ok:
        cmd = json.load(open(os.path.join(mroot, "runs/ch04-prepared/assemble-command.json")))
        r = subprocess.run([sys.executable, *cmd["argv"][1:]], cwd=mroot, capture_output=True, text=True)
        code3, out3 = M(*cmd["then"][4:])
        plan = json.load(open(os.path.join(mroot, "runs/ch04-prepared/prepare-receipt.json")))["cues"]["04_4"]
        ok = r.returncode == 0 and code3 == 0 and plan["recording"] == "04_4-replacement-fixture01.wav" and plan["recognition"]["run"] == "fixture-replacement"
    check("D12 chapter 04 assembles with the replacement, timed from its own run", ok, clean((out + out2).strip()[-200:]))
    # plants
    plants = []

    def plant(name, want_code, want_text, args, **kw):
        code, out = M(*args, **kw) if not kw else run(["--repo", mroot, *args], env=dict(env, **kw))
        ok = code == want_code and want_text in out
        plants.append(ok)
        check(f"D13 plant: {name} → exit {want_code}", ok, clean(out.strip().splitlines()[-1] if out.strip() else ""))

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
    plant("08_5 onset left unanswered", 1, "open question blocks the selection: onset:08_5:s4:08_5-download01.wav",
          prep("fixtures/selection-full.json", ans_variant("p-no-0805.json", drop="onset:08_5:s4:08_5-download01.wav"), "runs/p3"))
    plant("a selected take rejected by root's answer", 1, "root rejected 04_4-download01.wav",
          prep("fixtures/selection-full.json", ans_variant("p-reject.json", add={"content:04_4:base:04_4-download01.wav": {"decision": "reject_take"}}), "runs/p4"))
    plant("an answer naming an unknown question", 1, "unknown question or point",
          prep("fixtures/selection-full.json", ans_variant("p-unknown.json", add={"onset:99_9:s1:x.wav": {"decision": "sample", "sample_24k": 5}}), "runs/p5"))
    write_json(os.path.join(sel_dir, "selection-ch02.json"), selection(test_b, mroot, first_takes(anchors_doc, "02")))
    plant("decoded audio one sample short", 1, "decoded (1, 2, 24000) with",
          ["--binding", test_b, "decode", "--selection", "fixtures/selection-ch02.json", "--chapter", "02", "--out", "runs/p6", "--ffmpeg", "ffmpeg-standin", "--test-only"],
          FAKE_SHORT="02_1.wav")
    plant("an existing OUT", 2, "exists; name a new folder", prep("fixtures/selection-full.json", "fixtures/answers-full.json", "runs/full-prepared"))
    plant("OUT inside the bundle", 2, "lies inside the input folder", prep("fixtures/selection-full.json", "fixtures/answers-full.json", f"{HERE}/x"))
    data = json.load(open(os.path.join(mroot, test_b)))
    key = next(k for k in data["pins"] if k.endswith("script.tsv"))
    data["pins"][key] = "0" * 64
    stale_b = f"{HERE}/bindings/p-stale.json"
    write_json(os.path.join(mroot, stale_b), data)
    plant("a stale pin", 3, "STALE", ["--binding", stale_b, "report", "--out", "runs/p8"])
    plant("00_1-download03 selected without the provenance answer", 1, "provenance:00_1:00_1-download03.wav",
          prep(sel_variant("p-twin.json", dict(first_takes(anchors_doc), **{"00_1": "00_1-download03.wav"})), "fixtures/answers-full.json", "runs/p9"))
    plant("04_3 selected without the voice answer", 1, "voice:04_3:04_3-download01.wav",
          prep("fixtures/selection-full.json", ans_variant("p-novoice.json", drop="voice:04_3:04_3-download01.wav"), "runs/p10"))
    fx_sel = selection(fx, mroot, first_takes(anchors_doc, "04", {"04_4": "04_4-oldtext-fixture01.wav"}))
    write_json(os.path.join(sel_dir, "p-oldtext.json"), fx_sel)
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
    # C
    section_c(b, anchors_doc, questions_doc)
    # D
    test_b = variant(mroot, "test-historical.json", b, use="historical-test",
                     why="TEST binding, scratch only: the delivered binding with use historical-test.")
    section_d(b, mroot, test_b, anchors_doc)
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
