"""Exercise frederick_timing_bridge.py end to end on synthetic fixtures. From the repository root:

    python3 -I -B .../complex-frederick-timing-bridge01/checks/bridge_selftest.py FRESH_SCRATCH

Builds a synthetic chapter-01 take world with the sealed intake self-test's fixture writer (seeded-noise WAVs,
receipts, a synthetic guard job and fw records), runs the sealed intake's plan and ingest on it, then the bridge's
report, decode (ffmpeg on the noise fixtures only), prepare, the workspace assembler's assemble on those fixtures,
and timings. Every expected onset, mark and sample is computed here from the synthetic segments. Negative cases
check that each refusal names its cause. Read-only checks cover the real stage05 sources: hook census, anchor
byte offsets, dispositions against the pinned assembler's own direction contract, and the sentence-rule mirrors.
Writes only under FRESH_SCRATCH; no production audio, recognition, synthesis, build or render.
"""
from array import array
import difflib
import hashlib
import importlib.util
import json
from pathlib import Path
import random
import re
import shutil
import subprocess
import sys
import wave

REPO = Path.cwd()
HERE = Path(__file__).resolve().parent
TOOL = HERE.parent / "frederick_timing_bridge.py"
P = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104"
INTAKE_SELFTEST = P / "complex-frederick-audio-preparation01/checks/intake_selftest.py"
INTAKE_SELFTEST_SHA = "0fca8e1649dab182974b64e20f8248a90e2330ab4ae3a07b26e9fad7adc610f5"


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def sha(data):
    return hashlib.sha256(data).hexdigest()


def dump(path, data):
    Path(path).write_text(json.dumps(data, indent=1) + "\n", encoding="utf-8")


def segments_for(text, style, edits=None, t0=10):
    """One segment per whitespace word, in centiseconds. style 'gap': 24 cs word, 6 cs gap, 50 cs more after a
    sentence. style 'abut': each segment ends where the next begins. edits: word index -> replacement text, or
    'merge' (this word and the next in one segment). Returns (segments, wav seconds, first start of each word)."""
    edits = edits or {}
    words = text.split()
    out, starts, t, i = [], {}, t0, 0
    while i < len(words):
        n = 2 if edits.get(i) == "merge" else 1
        said = " ".join(words[i:i + n]) if n == 2 else edits.get(i, words[i])
        dur = 24 * n + 6 * (n - 1)
        step = dur + 6 + (50 if words[i + n - 1][-1] in ".?!" else 0)
        end = t + (step if style == "abut" else dur)
        out.append({"start_sec": t / 100, "end_sec": end / 100, "text": " " + said, "speaker": None,
                    "confidence": 0.9})
        for k in range(n):
            starts[i + k] = (t, end)
        t += step
        i += n
    return out, (t + 40) / 100, starts


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    assert "narration_lab" not in scratch.resolve().parts
    scratch.mkdir(parents=True)
    assert sha(INTAKE_SELFTEST.read_bytes()) == INTAKE_SELFTEST_SHA, "sealed intake self-test moved"
    it = load("intake_selftest", INTAKE_SELFTEST)
    fi = it.fi
    br = load("frederick_timing_bridge", TOOL)
    bron = br.dep("log_preparation")
    SCRIPT = it.SCRIPT
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        for path in sorted({str(scratch.resolve()), str(scratch)}, key=len, reverse=True):
            detail = detail.replace(path, "SCRATCH")
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    def last(err):
        return err.strip().splitlines()[-1] if err.strip() else ""

    # ------------------------------------------------------------------ real stage05 sources (read only)
    side, script, files = br.source_side(REPO / fi.CRATE)
    chapters = REPO / fi.CRATE / "src/chapters"
    literal = indirect = 0
    for path in sorted(chapters.glob("*.rs")):
        body = path.read_text(encoding="utf-8").split(br.TEST_SPLIT)[0]
        for line in body.splitlines():
            code = line.split("//")[0]
            literal += len(re.findall(r'\.(?:wait_for|until)\(stage, "', code)) + len(re.findall(r'\.at\("', code))
            indirect += len(re.findall(r"\.(?:wait_for|until)\(stage, [A-Za-z_]", code))
    check("r01-hook-census", (literal, indirect, side["hook_sites"]) == (80, 5, 85) and not side["anchor_problems"],
          f"{side['hook_sites']} hook sites = {literal} literal + {indirect} indirect (independent line grep); "
          f"{len(side['anchors'])} anchors in {len({a['cue'] for a in side['anchors']})} cues; no unbound phrase")
    bad = [a for a in side["anchors"] if script[a["cue"]].encode()[a["byte"]:a["byte_end"]].decode() != a["phrase"]
           or script[a["cue"]].encode().find(a["phrase"].encode()) != a["byte"]
           or len(script[a["cue"]].encode()[:a["byte"]].decode()) != a["char"]]
    unique = sum(1 for a in side["anchors"] for h in a["hooks"] if h["binding"].startswith("receiver"))
    check("r02-anchor-offsets", not bad and sum(len(a["hooks"]) for a in side["anchors"]) == 94,
          f"every byte, byte_end and char equals Rust str::find semantics (first occurrence); 94 phrase uses, "
          f"{unique} bound by unique text (helper receivers), the rest by a say() in the same function")
    ws = scratch / "real-workspace"
    for src, dst, _pin in br.workspace_copies(files):
        (ws / dst).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(src, ws / dst)
    mod = load("real_workspace_assembler", ws / "tools/coherent_narration.py")
    _script, units = mod.read_units(Path(files["script"]), Path(files["units"]), True)
    contract = mod.direction_contract(Path(files["script"]), json.loads(Path(files["units"]).read_text()))
    ends = {r["override_index"]: r["end_word_index"] for r in contract["resolved"]}
    disp = [{k: d[k] for k in ("override_index", "cue", "kind", "reason", "evidence")} for d in side["dispositions"]]
    reviewed = mod.review_directions(contract, units, {"direction_dispositions": disp})
    check("r03-dispositions", len(disp) == 23 and all(d["end_word_index"] == ends[d["override_index"]]
                                                      for d in side["dispositions"])
          and len(reviewed["dispositions"]) == 23 and not side["disposition_problems"],
          "23 native_visual_hold rows; end words equal the pinned assembler's direction_contract; its "
          "review_directions accepts them with every evidence hash for all 105 cues")
    coherent_same = all([s for s in mod.sentences(t)] == [t[a:z].strip() for a, z in bron.kit_sentences(t)]
                        for t in script.values())
    film_same = all(len(br.film_sentence_offsets(t)) == len(bron.kit_sentences(t)) for t in script.values())
    check("r04-sentence-mirrors", coherent_same and film_same and not side["film_ignores_measured_sentence_starts"],
          "coherent sentences = kit_sentences spans on 105 cues; draw.rs sentence_offsets count equals them, so the "
          "film uses measured sentence starts for every cue")
    # Rebind qualification (root's Mail 3920): the pinned canonical tool is Bronze's sealed patched copy plus
    # insertions only, namely the legacy Oliver constant and a check inside the legacy branch, which a v2 receipt
    # never reaches (the v2 branch returns first). Every synthetic test below runs the canonical bytes.
    sealed = (P / "elevenlabs-external-voice-repair01/patched/tools/coherent_narration.py").read_bytes()
    canonical = (REPO / br.DEPENDENCIES["assembler"][0]).read_bytes()
    old, new = sealed.decode().splitlines(), canonical.decode().splitlines()
    ops = [op for op in difflib.SequenceMatcher(None, old, new, autojunk=False).get_opcodes() if op[0] != "equal"]
    v2_return = next(i for i, line in enumerate(new) if line.strip() == "return voice")
    legacy_if = next(i for i, line in enumerate(new) if line.startswith("    if source == LEGACY_EXTERNAL_UI_SOURCE"))
    first, second = (ops + [None, None])[:2]
    added = [new[j] for op in ops for j in range(op[3], op[4])]
    check("r05-assembler-rebind", sha(sealed) == "18c4e75e740943463a7ccecd22f7aaf01e08e19b45c30d0960d6a5e12490ba7d"
          and sha(canonical) == br.DEPENDENCIES["assembler"][1] and len(ops) == 2
          and all(op[0] == "insert" for op in ops)
          and all(line.startswith(("#", "LEGACY_OLIVER_VOICES = ")) for line in new[first[3]:first[4]])
          and v2_return < legacy_if < second[3] and len(added) == 9,
          f"canonical tools/coherent_narration.py {sha(canonical)[:12]} = sealed patched copy 18c4e75e + "
          f"{len(added)} inserted lines in {len(ops)} places (the LEGACY_OLIVER_VOICES constant; a check inside the "
          f"legacy branch, after the v2 branch's return); nothing removed or changed")

    # ------------------------------------------------------------------ synthetic world, chapter 01
    fx = it.Fixture(scratch / "world")
    cues = [c for c in SCRIPT if c.startswith("01_")]
    words = {c: SCRIPT[c].split() for c in cues}
    edits = {("01_2", 1): {words["01_2"].index("plus"): "bus"},
             ("01_5", 1): {words["01_5"].index("They") - 1: "merge"},
             ("01_3", 1): {words["01_3"].index("fortieth"): "40"},
             ("01_6", 1): {words["01_6"].index("roots"): "routes"}}
    planned = {}
    for k, cue in enumerate(cues):
        wavs = []
        for n, style in ((1, "gap"), (2, "abut")):
            segs, seconds, starts = segments_for(SCRIPT[cue], style, edits.get((cue, n)))
            frames = int(48000 * seconds) | (1 if (cue, n) == ("01_1", 1) else 0)
            rng = random.Random(100 * k + n)
            pcm = array("h", (rng.randint(-900, 900) for _ in range(frames)))
            if sys.byteorder != "little":
                pcm.byteswap()
            wavs.append(it.noise(0, pcm=pcm.tobytes()))
            planned[(cue, f"{cue}-download{n:02d}.wav")] = {"segments": segs, "seconds": seconds, "starts": starts,
                                                             "frames": frames}
        fx.add(cue, (1, 2), layout="name" if k % 2 else "hash", wavs=wavs)
    code, out, err = fx.run("plan")
    plan = fx.plan_json("plan-01")
    records = [{"schema_version": fi.FW_RECORD, "index": i, "total": len(plan["takes"]), "input": take["path"],
                "status": "ok", "synthetic_fixture": True,
                "report": {"request": {"model": plan["recognition"]["model_path"], "backend": "whisper_cpp",
                                       "language": "en", "diarize": False,
                                       "backend_params": {"split_on_word": True, "decoding": {"max_segment_length": 1}}},
                           "result": {"backend": "whisper_cpp",
                                      "transcript": "".join(s["text"] for s in
                                                            planned[(take["cue"], take["name"])]["segments"]).strip(),
                                      "segments": planned[(take["cue"], take["name"])]["segments"],
                                      "raw_output": {"model_version_tag": f"fw-native-v1+sha256:{fi.TOOL_PINS['model'][:12]}",
                                                     "word_timestamps": "dtw"}}}}
               for i, take in enumerate(plan["takes"])]
    job = fx.job("plan-01", 1, records=records)
    code2, out2, err2 = fx.run("ingest", extra=("--plan", "plan-01", "--job", str(job)))
    check("w01-intake-plan-and-ingest", code == 0 and code2 == 0 and json.loads(out2)["processed"] == 12,
          f"sealed intake planned and ingested 12 synthetic takes of chapter 01 ({last(err) or 'ok'}; {last(err2) or 'ok'})")

    # A targeted second-model job for 01_5 download01, as root ran for 02_5: guard receipt, manifest, records.
    target = fx.takes / "01_5-download01.wav"
    small = scratch / "world/targeted"
    small.mkdir()
    model, fw = small / "fixture-small.bin", small / "fw"
    model.write_bytes(b"synthetic model fixture")
    (small / "inputs.txt").write_text(f"{target}\n", encoding="utf-8")
    manifest = small / "inputs.json"
    dump(manifest, {"files": [{"path": str(target), "sha256": sha(target.read_bytes())},
                              {"path": str(small / "inputs.txt"), "sha256": sha((small / "inputs.txt").read_bytes())},
                              {"path": str(model), "sha256": sha(model.read_bytes())},
                              {"path": str(fw), "sha256": fi.TOOL_PINS["fw"]}]})
    sjob = fx.work / "complex-fixture-targeted-small01"
    sjob.mkdir()
    command = [str(fw), "transcribe", "--inputs-from", str(small / "inputs.txt"), "--backend", "whisper-cpp", "--model",
               str(model), "--language", "en", "--no-diarize", "--no-persist", "--split-on-word",
               "--max-segment-length", "1", "--threads", "4", "--timeout", "180", "--json"]
    msha = sha(manifest.read_bytes())
    dump(sjob / "resource-receipt.json", {"phase": "asr", "command": command, "preflight_only": False, "admitted": True,
                                          "inputs_sha256": msha, "exit": 0, "final_inputs_sha256": msha,
                                          "finished": it.FIXED, "synthetic_fixture": True})
    ssegs, _s, sstarts = segments_for(SCRIPT["01_5"], "gap", t0=12)
    srecord = {"schema_version": fi.FW_RECORD, "index": 0, "total": 1, "input": str(target), "status": "ok",
               "report": {"request": {"model": str(model), "backend": "whisper_cpp", "language": "en", "diarize": False,
                                      "backend_params": {"split_on_word": True, "decoding": {"max_segment_length": 1}}},
                          "result": {"backend": "whisper_cpp", "transcript": "", "segments": ssegs,
                                     "raw_output": {"model_version_tag": f"fw-native-v1+sha256:{sha(model.read_bytes())[:12]}"}}}}
    (sjob / "command.log").write_text(json.dumps(srecord) + "\n", encoding="utf-8")

    base = ("--takes", str(fx.takes), "--lane", str(fx.lane), "--jobs", str(fx.work))
    supp = ("--supplement", str(sjob), str(manifest))

    def bridge(*args):
        r = subprocess.run([sys.executable, "-I", "-B", str(TOOL), *args], cwd=REPO, capture_output=True, text=True)
        return r.returncode, r.stdout, r.stderr

    code, out, err = bridge("report", *base, *supp, "--out", str(scratch / "report-a"))
    code_b, _o, _e = bridge("report", *base, *supp, "--out", str(scratch / "report-b"))
    same = code_b == 0 and all((scratch / "report-a" / n).read_bytes() == (scratch / "report-b" / n).read_bytes()
                               for n in ("source.json", "takes.json", "questions.json", "answers-template.json",
                                         "coverage.json"))
    takes = {(t["cue"], t["file"]): t for t in json.loads((scratch / "report-a/takes.json").read_text())["takes"]}
    questions = {q["id"]: q for q in json.loads((scratch / "report-a/questions.json").read_text())["questions"]}
    check("w02-report-deterministic", code == 0 and same and len(takes) == 12, f"{out.strip()[:160]}")

    # Every supported onset and mark equals the synthetic segment start; windows and abutment are explicit.
    wrong = []
    for (cue, name), t in takes.items():
        p = planned[(cue, name)]
        word_of_char = {}
        at = 0
        for i, w in enumerate(SCRIPT[cue].split()):
            at = SCRIPT[cue].index(w, at)
            word_of_char[at] = i
            at += len(w)
        for row in t["sentences"][1:] + [m for m in t["anchors"] if m["status"] != "sentence_onset"]:
            char = row.get("char", None)
            if char is None:
                char = len(SCRIPT[cue].encode()[:row["byte"]].decode())
            start, end = p["starts"][word_of_char[char]]
            if row["status"] == "supported" and (row["sample_24k"] != start * 240 or row["window_s"] != [start / 100, end / 100]
                                                 or row["abuts_previous_word"] != name.endswith("02.wav")):
                wrong.append(row.get("id"))
    statuses = {q: questions[q]["kind"] for q in questions}
    onsets_total = sum(len(t["sentences"]) - 1 for t in takes.values())
    check("w03-points-exact", not wrong and onsets_total == 2 * sum(len(bron.kit_sentences(SCRIPT[c])) - 1 for c in cues)
          and sum(s["status"] == "supported" for t in takes.values() for s in t["sentences"][1:]) == onsets_total - 1,
          f"all supported points equal round(start x 24000) of their synthetic segment, with window [start, end]; "
          f"abuts_previous_word true exactly for the abutting takes; wrong {wrong}")
    expect_q = {"anchor:01_2:b65:01_2-download01.wav": "phrase_anchor", "onset:01_5:s2:01_5-download01.wav":
                "sentence_onset", "content:01_2:01_2-download01.wav": "content",
                "content:01_3:01_3-download01.wav": "content", "content:01_6:01_6-download01.wav": "content"}
    q1, q2 = questions.get("anchor:01_2:b65:01_2-download01.wav", {}), questions.get("onset:01_5:s2:01_5-download01.wav", {})
    check("w04-questions", statuses == expect_q and all(q["blocking"] for q in questions.values())
          and q1.get("candidate", {}).get("heard") == "bus" and q1["decisions"] == ["use_candidate", "sample", "omit_mark"]
          and q2["decisions"] == ["use_supplementary:complex-fixture-targeted-small01", "sample"],
          f"{len(questions)} blocking questions: a misheard anchor word offers its candidate; a merged segment "
          "leaves sentence 2 of 01_5 inside_a_segment and offers the targeted second model; three content "
          "differences ('proceed' or 'reject_take')")
    s2 = next(s for s in takes[("01_5", "01_5-download01.wav")]["sentences"] if s["index"] == 2)
    word = words["01_5"].index("They")
    cross = takes[("01_5", "01_5-download01.wav")]["sentences"][1]["supplementary"][0]
    check("w05-supplement-cross-check", s2["status"] == "inside_a_segment"
          and s2["supplementary"][0]["sample_24k"] == sstarts[word][0] * 240
          and cross["delta_s"] == round(sstarts[words["01_5"].index("Let")][0] / 100
                                        - planned[("01_5", "01_5-download01.wav")]["starts"][words["01_5"].index("Let")][0] / 100, 6),
          f"supplement placement {s2['supplementary'][0]['sample_24k']} for 'They'; delta_s {cross['delta_s']} on a "
          "point both models support")

    # Bronze's own points_for gives the same statuses and samples on every fixture take.
    anchors = [a for a in side["anchors"] if a["cue"] in cues]
    differ = []
    for (cue, name), t in takes.items():
        b_sent, b_marks = bron.points_for(cue, name, SCRIPT[cue], {"segments": planned[(cue, name)]["segments"]},
                                          planned[(cue, name)]["frames"] / 48000, [a for a in anchors if a["cue"] == cue])
        for mine, theirs in zip(t["sentences"][1:], b_sent[1:]):
            if (mine["status"], mine.get("sample_24k")) != (theirs["status"], theirs.get("sample_24k")):
                differ.append(mine["id"])
        for mine, theirs in zip(t["anchors"], b_marks):
            if mine["status"] == "sentence_onset":
                onset = t["sentences"][mine["sentence"]]
                if theirs["status"] == "supported" and onset.get("sample_24k") != theirs["sample_24k"]:
                    differ.append(mine["id"])
            elif (mine["status"], mine.get("sample_24k")) != (theirs["status"], theirs.get("sample_24k")):
                differ.append(mine["id"])
    check("w06-log-preparation-agreement", not differ,
          f"statuses and samples equal logarithm-stage08 prepare_audio.points_for on 12 takes; differing {differ}")

    # ------------------------------------------------------------------ selection, decode, answers, prepare
    source_sha = json.loads((scratch / "report-a/source.json").read_text())["sha256"]
    selection = scratch / "selection.json"
    sel = {"schema": br.SELECTION_SCHEMA, "source_sha256": source_sha, "script_sha256": fi.PINS["script"],
           "by": "fixture root", "basis": "synthetic test selection", "chapter": "01",
           "takes": {c: f"{c}-download01.wav" for c in cues}}
    dump(selection, sel)

    def variant(name, change):
        data = json.loads(json.dumps(sel))
        change(data)
        path = scratch / f"{name}.json"
        dump(path, data)
        return path

    for name, change, needle in (
            ("n01-selection-needs-basis", lambda d: d.update(basis=" "), "needs `by` and `basis`"),
            ("n02-selection-unknown-take", lambda d: d["takes"].update({"01_3": "01_3-download03.wav"}),
             "is not an ingested take"),
            ("n03-selection-cue-set", lambda d: d["takes"].pop("01_6"), "must name exactly these 6 cues"),
            ("n04-selection-other-source", lambda d: d.update(source_sha256="0" * 64), "stage05 source side")):
        code, out, err = bridge("decode", *base, *supp, "--chapter", "01", "--selection", str(variant(name, change)),
                                "--out", str(scratch / name))
        check(name, code == 2 and needle in err and not (scratch / name).exists(), last(err))

    decoded = scratch / "decoded"
    code, out, err = bridge("decode", *base, *supp, "--chapter", "01", "--selection", str(selection), "--out", str(decoded))
    receipt = json.loads((decoded / "decode-receipt.json").read_text()) if code == 0 else {}
    frames_ok = all(r["samples"] == (planned[(r["cue"], f"{r['cue']}-download01.wav")]["frames"] + 1) // 2
                    and wave.open(r["wav"]).getparams()[:3] == (1, 2, 24000) for r in receipt.get("records", []))
    mod_voice = mod.external_ui_voice(receipt) if receipt else None
    requests_ok = all(Path(r["request"]) == (fx.takes / f"{r['cue']}-request01.json").resolve()
                      and sha(Path(r["request"]).read_bytes()) == r["request_sha256"] and r["voice"] == fi.VOICE
                      and json.loads(Path(r["request"]).read_text())["text"] == r["text"] == SCRIPT[r["cue"]]
                      for r in receipt.get("records", []))
    check("w07-decode-v2-receipt", code == 0 and frames_ok and mod_voice == fi.VOICE and requests_ok
          and (receipt["schema"], receipt["source"], receipt["provider"]) == (
              "elevenlabs-ui-receipt-v2", "ElevenLabs UI, Frederick Surrey", "ElevenLabs")
          and set(receipt["records"][0]) >= set(mod.EXTERNAL_RECORD_KEYS + mod.EXTERNAL_V2_RECORD_KEYS),
          f"6 takes decoded with root's ffmpeg arguments, ceil(n/2) frames (01_1 has an odd count); every record "
          f"names the full request path (the downloads receipt's folder joined to its bare name), its sha256 and "
          f"the exact voice; the pinned assembler's external_ui_voice reads {mod_voice!r}")

    answers = scratch / "answers.json"
    first = takes[("01_1", "01_1-download01.wav")]["sentences"][1]
    ans = {"schema": br.ANSWERS_SCHEMA, "source_sha256": source_sha, "answers": {
        "anchor:01_2:b65:01_2-download01.wav": {"decision": "use_candidate", "by": "fixture root", "basis": "test"},
        "anchor:01_2:b87:01_2-download01.wav": {"decision": "omit_mark", "by": "fixture root", "basis": "test"},
        "onset:01_5:s2:01_5-download01.wav": {"decision": "use_supplementary:complex-fixture-targeted-small01",
                                              "by": "fixture root", "basis": "test"},
        first["id"]: {"decision": "sample", "by": "fixture root", "basis": "test", "sample_24k": first["sample_24k"] - 240},
        **{q: {"decision": "proceed", "by": "fixture root", "basis": "test"} for q in questions if q.startswith("content:")}}}
    dump(answers, ans)

    def answers_variant(name, change):
        data = json.loads(json.dumps(ans))
        change(data["answers"])
        path = scratch / f"{name}.json"
        dump(path, data)
        return path

    prep = ("prepare", *base, *supp, "--chapter", "01", "--selection", str(selection), "--decoded", str(decoded),
            "--reviewer", "fixture root")
    cases = (
        ("n05-unanswered-blocks", lambda a: a.pop("content:01_3:01_3-download01.wav"), "open question blocks"),
        ("n06-unknown-answer", lambda a: a.update({"onset:01_9:s1:x.wav": {"decision": "sample"}}), "unknown question"),
        ("n07-sample-needs-int", lambda a: a[first["id"]].update(sample_24k=1.5), "positive integer sample_24k"),
        ("n08-reject-take", lambda a: a["content:01_6:01_6-download01.wav"].update(decision="reject_take"),
         "root rejected 01_6-download01.wav"),
        ("n09-answer-needs-basis", lambda a: a["anchor:01_2:b65:01_2-download01.wav"].update(by=""), "needs `by`"),
        ("n10-order", lambda a: a[first["id"]].update(sample_24k=10 ** 7), "not strictly increasing"),
        ("n11-supplement-absent", lambda a: a["onset:01_5:s2:01_5-download01.wav"].update(
            decision="use_supplementary:other-job"), "is not one of"))
    for name, change, needle in cases:
        code, out, err = bridge(*prep, "--answers", str(answers_variant(name, change)), "--out", str(scratch / name))
        check(name, code == 2 and needle in err and not (scratch / name).exists(), last(err))
    code, out, err = bridge("prepare", *base, "--chapter", "01", "--selection", str(selection), "--decoded", str(decoded),
                            "--reviewer", "fixture root", "--answers", str(answers), "--out", str(scratch / "n12"))
    check("n12-supplement-must-be-given", code == 2 and "is not one of" in err, last(err))

    # A decoded file that no longer matches its receipt row.
    bad_decode = scratch / "decoded-bad"
    shutil.copytree(decoded, bad_decode)
    data = bytearray((bad_decode / "decoded/01_4.wav").read_bytes())
    data[-2] ^= 1
    (bad_decode / "decoded/01_4.wav").write_bytes(bytes(data))
    rec = json.loads((bad_decode / "decode-receipt.json").read_text())
    rec["records"][3]["wav"] = str(bad_decode / "decoded/01_4.wav")
    dump(bad_decode / "decode-receipt.json", rec)
    code, out, err = bridge(*prep[:-4], "--decoded", str(bad_decode), "--reviewer", "fixture root", "--answers",
                            str(answers), "--out", str(scratch / "n13"))
    check("n13-decoded-file-changed", code == 2 and "01_4: the decoded file, its receipt row" in err, last(err))
    rec = json.loads((decoded / "decode-receipt.json").read_text())
    rec["voice"] = "Oliver - Clean, British and Steady"
    other = scratch / "decoded-voice"
    other.mkdir()
    dump(other / "decode-receipt.json", rec)
    code, out, err = bridge(*prep[:-4], "--decoded", str(other), "--reviewer", "fixture root", "--answers",
                            str(answers), "--out", str(scratch / "n14"))
    check("n14-other-voice-receipt", code == 2 and "is not this tool's Frederick v2 decode" in err, last(err))

    prepared = scratch / "prepared"
    code, out, err = bridge(*prep, "--answers", str(answers), "--out", str(prepared))
    spans = json.loads((prepared / "spans.json").read_text()) if code == 0 else {"units": []}
    marks_plan = json.loads((prepared / "phrase-marks-plan.json").read_text())["cues"] if code == 0 else {}
    pr = json.loads((prepared / "prepare-receipt.json").read_text()) if code == 0 else {}
    want_ends = {cue: [s.get("sample_24k") for s in takes[(cue, f"{cue}-download01.wav")]["sentences"][1:]]
                 for cue in cues}
    want_ends["01_1"][0] -= 240
    want_ends["01_5"][1] = sstarts[word][0] * 240
    got = {u["id"]: u["sentence_ends"][:-1] for u in spans["units"]}
    uid = side["unit_ids"]
    disp_ok = [d["cue"] for d in spans.get("direction_dispositions", [])] == ["01_4"] and \
        spans["direction_dispositions"][0]["evidence"]["path"].endswith("chapters/the_bump.rs")
    m12 = [m["byte"] for m in marks_plan.get("01_2", {}).get("marks", [])]
    check("w08-prepare", code == 0 and all(got[uid[c]] == want_ends[c] for c in cues) and disp_ok
          and all(u["gap_after_samples"] == [0] * len(u["sentence_ends"]) for u in spans["units"])
          and spans["boundary_review"]["listening"] == "unreviewed" and m12 == [65]
          and pr["voice"] == fi.VOICE and pr["recognition"]["supplements"][0]["records"] == 1,
          f"{out.strip()[:200]}; sentence ends are the recognizer onsets except root's sample (01_1 s1) and the "
          "accepted second-model onset (01_5 s2); 01_2 keeps the candidate mark and omits the other; one "
          "native_visual_hold (01_4) with the_bump.rs evidence")

    narration = scratch / "narration"
    asm = json.loads((prepared / "assemble-command.json").read_text())["argv"] if code == 0 else []
    asm = [a if a != (str(prepared) + "-narration") else str(narration) for a in asm]
    r = subprocess.run([sys.executable] + asm[1:], cwd=REPO, capture_output=True, text=True)
    check("w09-workspace-assemble", r.returncode == 0 and (narration / "report.json").is_file(),
          f"the prepared argv ran the pinned workspace assembler on the fixtures: "
          f"{json.loads(r.stdout).get('assembled_cues') if r.returncode == 0 else last(r.stderr)} cues")

    timings_out = scratch / "timings"
    code, out, err = bridge("timings", "--prepared", str(prepared), "--narration", str(narration), "--out", str(timings_out))
    frag = json.loads((timings_out / "phrase-timings-fragment-01.json").read_text()) if code == 0 else {"cues": {}}
    report = json.loads((narration / "report.json").read_text()) if (narration / "report.json").is_file() else {"lines": {}}
    onsets_ok = True
    for cue in cues:
        at, onsets = 0, []
        for row in report["lines"][cue]:
            onsets.append(at)
            at += row["out_samples"] + row["gap_after_samples"]
        onsets_ok &= onsets[1:] == want_ends[cue]
    c12 = frag["cues"].get("01_2", {}).get("marks", [])
    cand = takes[("01_2", "01_2-download01.wav")]["anchors"][0]["candidate"]
    c15 = frag["cues"].get("01_5", {}).get("marks", [])
    check("w10-timings", code == 0 and onsets_ok and len(frag["cues"]) == 6 and "fragment" in frag
          and [(m["byte"], m["seconds"]) for m in c12] == [(65, cand["sample_24k"] / 24000)]
          and [m["seconds"] for m in c15] == [s / 24000 for s in [want_ends["01_5"][0], want_ends["01_5"][1],
                                                                 want_ends["01_5"][2], want_ends["01_5"][3]]]
          and all(e["wav_sha256"] == sha((narration / f"{c}.wav").read_bytes()) for c, e in frag["cues"].items())
          and all("film_without_mark_s" in m and "window_s" in m for e in frag["cues"].values() for m in e["marks"]),
          f"{out.strip()}; report.json onsets equal the prepared sentence ends; marks carry byte, seconds, basis, "
          "window and the film's estimate without the mark; anchors at sentence starts reuse the onset")

    # ------------------------------------------------------------------ late tampering (after the positive path)
    bad_narr = scratch / "narration-bad"
    shutil.copytree(narration, bad_narr)
    data = bytearray((bad_narr / "01_3.wav").read_bytes())
    data[-4] ^= 1
    (bad_narr / "01_3.wav").write_bytes(bytes(data))
    code, out, err = bridge("timings", "--prepared", str(prepared), "--narration", str(bad_narr), "--out", str(scratch / "n15"))
    check("n15-narration-changed", code == 2 and "coherent_narration verify failed" in err and not (scratch / "n15").exists(),
          last(err))
    lane2 = scratch / "lane-tampered"
    shutil.copytree(fx.lane, lane2)
    t2 = json.loads((lane2 / "ingest-01/transcripts.json").read_text())
    t2["rows"][0]["transcript"] += " extra"
    dump(lane2 / "ingest-01/transcripts.json", t2)
    code, out, err = bridge("report", "--takes", str(fx.takes), "--lane", str(lane2), "--jobs", str(fx.work), "--out",
                            str(scratch / "n16"))
    check("n16-transcripts-changed", code == 2 and "transcripts.json changed since ingest" in err, last(err))
    work2 = scratch / "work-tampered"
    shutil.copytree(fx.work, work2)
    with open(work2 / job.name / "command.log", "a", encoding="utf-8") as handle:
        handle.write("\n")
    code, out, err = bridge("report", *base[:4], "--jobs", str(work2), "--out", str(scratch / "n17"))
    check("n17-command-log-changed", code == 2 and "command.log is missing under the jobs folder or changed" in err, last(err))
    m2 = small / "inputs-other.json"
    dump(m2, json.loads(manifest.read_text()) | {"note": "another manifest"})
    code, out, err = bridge("report", *base, "--supplement", str(sjob), str(m2), "--out", str(scratch / "n18"))
    check("n18-supplement-manifest", code == 2 and "not a completed guarded recognition" in err, last(err))
    code, out, err = bridge("report", *base, *supp, "--out", str(scratch / "report-a"))
    check("n19-out-exists", code == 2 and "exists; give a new folder" in err, last(err))
    crate = scratch / "crate-copy"
    shutil.copytree(REPO / fi.CRATE, crate, ignore=shutil.ignore_patterns("target", "renders"))
    with open(crate / "script.tsv", "a", encoding="utf-8") as handle:
        handle.write("\n")
    code, out, err = bridge("report", "--crate", str(crate), "--out", str(scratch / "n20"))
    check("n20-script-drift", code == 3 and "PIN MOVED" in err and "script" in err, last(err))

    # The source scanner refuses an unbound variable hook and reports an unbound literal.
    syn = scratch / "chapters"
    syn.mkdir()
    (syn / "probe.rs").write_text('fn a(kit: &Kit, stage: &mut Stage<\'_>) {\n    let line = say(&kit, stage, "01_4")?;\n'
                                  '    line.wait_for(stage, "no such words")?;\n    line.wait_for(stage, words)?;\n}\n',
                                  encoding="utf-8")
    try:
        br.chapter_hooks(syn / "probe.rs")
        refused = ""
    except br.PinMoved as error:
        refused = str(error)
    (syn / "probe2.rs").write_text('fn b(line: &Spoken, stage: &mut Stage<\'_>) {\n    line.wait_for(stage, "and so on")?;\n'
                                   '    // line.wait_for(stage, "commented out")?;\n'
                                   '    line.wait_for(stage, "no such words")?;\n}\n', encoding="utf-8")
    found, masked, lit_at, _used = br.chapter_hooks(syn / "probe2.rs")
    bound = [br.bind(script, h, p[0], masked, lit_at) for h, p in found]
    check("n21-source-scanner", "unrecognised phrase hook" in refused and len(found) == 2
          and bound[0][0] is None and "cues contain 'and so on'" in bound[0][1]
          and bound[1][0] is None and "0 script cues" in bound[1][1],
          f"variable hook: {refused[:70]}; ambiguous and missing literals become problems; comments are skipped")

    # The request file root's take binds changes after decode: the pinned assembler's own check refuses.
    req = fx.takes / "01_2-request01.json"
    req.write_text(req.read_text(encoding="utf-8") + " ", encoding="utf-8")
    code, out, err = bridge(*prep, "--answers", str(answers), "--out", str(scratch / "n22"))
    check("n22-request-changed", code == 2 and (("request" in err and "changed" in err) or "external request" in err),
          last(err))

    print(f"SELFTEST: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
