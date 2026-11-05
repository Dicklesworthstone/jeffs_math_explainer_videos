"""Independent checks of BronzeJaguar's sealed logarithm-stage09-audio-preparation01 (root's Mail 3974).

From the repository root:

    python3 -I -B .../logarithm-stage09-audio-independent01/checks/independent_checks.py FRESH_SCRATCH

Written by LavenderPike without running or copying Bronze's run_checks.py. Read-only on the repository; writes only
under FRESH_SCRATCH. Real ffmpeg runs only on synthetic clicks. The end-to-end assembly uses synthetic 24 kHz noise
under the stage09 script, units, map and proposed dispositions, through a byte copy of canonical
tools/coherent_narration.py. Bronze's helper is imported only to call its selection, answer and point functions
in-process on the real binding with fixture files written here; nothing is decoded, assembled or timed from
production audio. No recognition is run: the clarity job's command.log is parsed as it stands. Nothing here is a
listening check, and no take is selected.
"""
import ast
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
from array import array

REPO = Path.cwd()
P = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
UNDER = f"{P}/logarithm-stage09-audio-preparation01"
UNDER_SEAL = "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13"
BINDING = f"{UNDER}/bindings/logarithm-stage09-oliver.json"
OLD_BINDING = f"{P}/logarithm-stage08-audio-preparation01/bindings/logarithm-stage08-oliver.json"
CANONICAL = {"tools/coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
             "tools/narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
             "crates/explainer_kit/src/narration.rs": "8072720d8ff86b91ad35d6904e148d330fdcf736b931f393f12b8bc9e4f823b8"}
CHANGED = ("04_4", "08_5")
# The reviewed clarity rewording, as Bronze's HANDOFF and root's anchor inventory state it.
DELTA = {"04_4": ("which is why this base is called natural", "which is why base e is called natural"),
         "08_5": ("Halving the lost lap leaves a half turn", "Taking half of the lost lap leaves a half turn")}
TEST_SPLIT = "\n#[cfg(test)]\nmod tests {"
UI_OLIVER = "Oliver - Clean, British and Steady"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def sha_file(path):
    return sha(Path(path).read_bytes())


def load(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def script_rows(path):
    rows = {}
    for line in Path(path).read_text(encoding="utf-8").splitlines():
        if line.strip() and not line.startswith("#"):
            cue, text = line.split("\t", 1)
            rows[cue] = text
    return rows


def my_sentences(text):
    """My own splitter: a sentence ends at . ? or ! followed by whitespace."""
    return [s for s in re.split(r"(?<=[.?!])\s+", text.strip()) if s]


def words(text):
    return re.sub(r"[^a-z0-9 ]+", " ", text.lower()).split()


def write_wav(path, rate, pcm):
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(rate)
        w.writeframes(pcm)


def read_pcm(path):
    with wave.open(str(path), "rb") as w:
        return (w.getnchannels(), w.getsampwidth(), w.getframerate()), w.readframes(w.getnframes())


def main():
    scratch = Path(sys.argv[1])
    assert not scratch.exists(), "fresh scratch please"
    scratch.mkdir(parents=True)
    results = []

    def check(name, ok, detail=""):
        results.append(bool(ok))
        for path in sorted({str(scratch.resolve()), str(scratch)}, key=len, reverse=True):
            detail = detail.replace(path, "SCRATCH")
        print(f"{'PASS' if ok else 'FAIL'} {name}" + (f": {detail}" if detail else ""))

    # ------------------------------------------------------------------ K01 seal and pins
    sums = (REPO / UNDER / "SHA256SUMS").read_bytes()
    listed = [line.split("  ", 1) for line in sums.decode().splitlines()]
    bad = [n for h, n in listed if sha_file(REPO / UNDER / n) != h]
    check("K01-seal", sha(sums) == UNDER_SEAL and not bad and len(listed) == 14,
          f"SHA256SUMS {UNDER_SEAL[:12]}, {len(listed)} files verify; differing {bad}")
    raw = (REPO / BINDING).read_bytes()
    b = json.loads(raw)
    stale = [p for p, h in b["pins"].items() if not (REPO / p).is_file() or sha_file(REPO / p) != h]
    canon = {p: b["pins"].get(p) for p in CANONICAL}
    check("K02-pins", not stale and len(b["pins"]) == 36 and canon == CANONICAL and b["assembler"] == "tools/coherent_narration.py"
          and all(sha_file(REPO / p) == h for p, h in CANONICAL.items()),
          f"binding {sha(raw)[:12]}: {len(b['pins'])} pins re-hashed, stale {stale}; assembler, cache and kit pinned at the "
          f"canonical bytes root installed in Mail 3920 ({CANONICAL['tools/coherent_narration.py'][:8]})")

    # ------------------------------------------------------------------ K03 wording delta
    crate = f"{b['tree']}/explainer_corpus/{b['topic']}"
    new = script_rows(REPO / crate / "script.tsv")
    ob = json.loads((REPO / OLD_BINDING).read_bytes())
    old = script_rows(REPO / f"{ob['tree']}/explainer_corpus/{ob['topic']}/script.tsv")
    ws = scratch / "workspace"
    units_path = REPO / crate / "narration_units.json"
    units_doc = json.loads(units_path.read_text())
    map_rel = units_doc["direction_contract"]["map"]["path"]
    for src, dst in (("tools/coherent_narration.py", "tools/coherent_narration.py"),
                     ("tools/narration_cache.py", "tools/narration_cache.py"), (f"{b['tree']}/{map_rel}", map_rel)):
        (ws / dst).parent.mkdir(parents=True, exist_ok=True)
        shutil.copyfile(REPO / src, ws / dst)
    asm = load("canonical_assembler_copy", ws / "tools/coherent_narration.py")
    changed = sorted(c for c in new if old.get(c) != new[c])
    same_counts = all(len(my_sentences(t)) == len(asm.sentences(t)) for t in new.values())
    delta_ok = changed == list(CHANGED) and list(old) == list(new) and all(
        old[c].count(DELTA[c][0]) == 1 and old[c].replace(DELTA[c][0], DELTA[c][1]) == new[c] for c in CHANGED)
    diffs = {c: [(op, old[c].split()[i1:i2], new[c].split()[j1:j2]) for op, i1, i2, j1, j2
                 in difflib.SequenceMatcher(None, old[c].split(), new[c].split()).get_opcodes() if op != "equal"]
             for c in CHANGED}
    check("K03-wording-delta", delta_ok and same_counts
          and all(len(my_sentences(old[c])) == len(my_sentences(new[c])) == 6 for c in CHANGED)
          and len(new["08_5"].encode()) - len(old["08_5"].encode()) == 7,
          f"stage08 -> stage09 script: only {changed} differ, each by exactly the reviewed phrase ({diffs}); 6 sentences "
          f"before and after in both; 08_5 is 7 bytes longer; my splitter equals the canonical sentences() on all "
          f"{len(new)} cues")

    # ------------------------------------------------------------------ K04 chapter hooks for the changed cues
    consts, calls, others = {}, [], []
    for path in sorted((REPO / crate / "src/chapters").glob("*.rs")):
        body = path.read_text(encoding="utf-8").split(TEST_SPLIT)[0]
        for m in re.finditer(r'const\s+([A-Z_][A-Z0-9_]*)\s*:\s*&str\s*=\s*"([^"]*)"\s*;', body):
            consts[m.group(1)] = m.group(2)
        for line_no, line in enumerate(body.splitlines(), 1):
            code = line.split("//")[0]
            for cue in CHANGED:
                if f'"{cue}"' not in code:
                    continue
                m = (re.search(rf'kit\.say\(stage, "{cue}"\)', code)
                     or re.search(rf'kit\.until_sentence\(stage, "{cue}", (\d+)\)', code)
                     or re.search(rf'until_phrase\(&kit, stage, "{cue}", ([A-Z_][A-Z0-9_]*)\)', code))
                (calls if m else others).append((path.name, line_no, cue, m.group(0) if m else code.strip()))
    sentence_hooks = {c: sorted(int(re.search(r"(\d+)\)$", x[3]).group(1)) for x in calls if x[2] == c and "until_sentence" in x[3])
                      for c in CHANGED}
    phrase_names = [re.search(r", ([A-Z_][A-Z0-9_]*)\)$", x[3]).group(1) for x in calls if "until_phrase" in x[3]]
    found = {n: (new["08_5"].encode().find(consts[n].encode()), new["08_5"].encode().count(consts[n].encode()))
             for n in phrase_names}
    plan = json.loads((REPO / b["phrase_anchors"]).read_text())["anchors"]
    plan_bytes = {c: sorted(a["byte"] for a in plan if a["cue"] == c) for c in CHANGED}
    report = json.loads((REPO / UNDER / "report/anchors.json").read_text())["recordings"]
    clar = [e for e in report if e["cue"] in CHANGED]
    rep_bytes = {e["label"]: [m["byte"] for m in e["phrase_anchors"]] for e in clar}
    check("K04-changed-cue-hooks", not others and sentence_hooks == {"04_4": [1, 2, 3, 4, 5], "08_5": [1, 2, 3, 4, 5]}
          and sorted(v for v in found.values()) == [(267, 1), (306, 1)] and plan_bytes == {"04_4": [], "08_5": [267, 306]}
          and len(clar) == 4 and all(v == ([267, 306] if "08_5" in k else []) for k, v in rep_bytes.items()),
          f"non-test chapter code names 04_4/08_5 in {len(calls)} calls: until_sentence {sentence_hooks} (6 sentences each); "
          f"until_phrase {', '.join(f'{n} {consts[n]!r} at byte {found[n][0]} (x{found[n][1]})' for n in phrase_names)}; "
          f"root's anchor plan and Bronze's report give the same bytes for the 4 clarity recordings; unclassified uses {others}")

    # ------------------------------------------------------------------ K05 directions and dispositions
    contract = asm.direction_contract(Path(REPO / crate / "script.tsv"), units_doc)
    resolved = contract["resolved"]
    at_end = all(r["end_word_index"] == len(asm.direction_words(new[r["cue"]])) - 1 for r in resolved)
    disp = json.loads((REPO / b["dispositions"]).read_text())["direction_dispositions"]
    reviewed = asm.review_directions(contract, units_doc["units"], {"direction_dispositions": disp})
    check("K05-directions", len(resolved) == 15 and at_end and not any(r["cue"] in CHANGED for r in resolved)
          and len(disp) == 15 and {d["kind"] for d in disp} == {"native_visual_hold"}
          and len(reviewed["dispositions"]) == 15
          and all(sha_file(REPO / d["evidence"]["path"]) == d["evidence"]["sha256"] for d in disp),
          f"map {map_rel.split('/')[-1]}: {len(resolved)} visual directions, each at its cue's last word, none in 04_4/08_5; "
          f"the 15 proposed dispositions are native_visual_hold with a current evidence hash, and the canonical "
          f"review_directions accepts them for all {len(units_doc['units'])} units")

    # ------------------------------------------------------------------ K06 clarity provenance
    inv_spec = b["inventories"][1]
    inv = json.loads((REPO / inv_spec["path"]).read_text())
    prov = []
    for t in inv["takes"]:
        wav_rel = str(Path(t["wav"]).relative_to(REPO)) if Path(t["wav"]).is_absolute() else t["wav"]
        req = REPO / inv_spec["takes_dir"] / f"{t['cue']}-request01.json"
        r = json.loads(req.read_text())
        (_ch, _w, rate), pcm = read_pcm(REPO / wav_rel)
        prov.append(sha_file(REPO / wav_rel) == t["sha256"] and sha(pcm) == t["pcm_sha256"] and len(pcm) // 2 == t["frames"]
                    and rate == 48000 and t["receipt_sha256"] == sha_file(req) and r["cue"] == t["cue"]
                    and r["text"] == new[t["cue"]] and r["text_sha256"] == sha(new[t["cue"]].encode())
                    and t["text_sha256"] == sha(new[t["cue"]].encode()) and r["voice"] == UI_OLIVER)
    superseded = sorted(json.loads((REPO / UNDER / "report/coverage.json").read_text())["superseded_text_recordings"])
    check("K06-clarity-provenance", len(prov) == 4 and all(prov) and len(superseded) == 4
          and all("/ui-takes01/" in s and Path(s).name.split("-")[0] in CHANGED for s in superseded),
          f"4 clarity WAVs (48 kHz) at their inventoried file and PCM hashes and frame counts; each request names its cue, "
          f"the exact stage09 text and {UI_OLIVER!r}, and its hash is the inventory's receipt_sha256; the report keeps "
          f"the 4 ui-takes01 04_4/08_5 recordings as superseded")

    # ------------------------------------------------------------------ K07 the clarity recognition, parsed here
    run = next(r for r in b["recognition"] if r["name"] == "clarity")
    records = {}
    for line in (REPO / run["job"] / "command.log").read_text(encoding="utf-8").splitlines():
        if line.startswith("{"):
            rec = json.loads(line)
            records[Path(rec["input"]).name] = rec
    by_label = {e["label"]: e for e in clar}
    rows, ok7 = [], len(records) == 4
    for name, rec in sorted(records.items()):
        segs = rec["report"]["result"]["segments"]
        said = words(" ".join(s["text"] for s in segs))
        cue = name[:4]
        new_w, old_w = words(DELTA[cue][1]), words(DELTA[cue][0])
        has_new = any(said[i:i + len(new_w)] == new_w for i in range(len(said)))
        has_old = any(said[i:i + len(old_w)] == old_w for i in range(len(said)))
        e = next(v for k, v in by_label.items() if k.endswith("/" + name))
        # My own alignment: script words against recognized tokens (each token keeps its segment); a sentence onset
        # is the start of the segment holding the token matched to that sentence's first word.
        tokens = [(w, i) for i, s in enumerate(segs) for w in words(s["text"])]
        script_w = words(new[cue])
        firsts, count = [], 0
        for sentence in my_sentences(new[cue]):
            firsts.append(count)
            count += len(words(sentence))
        match = {}
        for a, z, size in difflib.SequenceMatcher(None, script_w, [w for w, _ in tokens], autojunk=False).get_matching_blocks():
            for j in range(size):
                match[a + j] = z + j
        onsets = [(segs[tokens[match[f]][1]]["start_sec"], segs[tokens[match[f]][1]]["end_sec"]) if f in match else (None, None)
                  for f in firsts[1:]]
        mine = [round(s * 24000) if s is not None else None for s, _e in onsets]
        theirs = [s["sample_24k"] for s in e["sentences"][1:]]
        ok7 = ok7 and rec["status"] == "ok" and has_new and not has_old and mine == theirs
        rows.append(f"{name}: new phrase {has_new}, old phrase {has_old}, onsets {mine} vs report {theirs}"
                    + (f", 'Here' segment [{onsets[0][0]:.2f}, {onsets[0][1]:.2f}] = {onsets[0][1] - onsets[0][0]:.2f} s"
                       if cue == "08_5" else ""))
    check("K07-clarity-recognition", ok7,
          "parsed from the clarity job's command.log with my own tokenizer and word alignment (no recognition run; "
          "the same recognizer output the report used, so this corroborates the placement, not the speech): "
          + "; ".join(rows))

    # ------------------------------------------------------------------ K08 decoder timeline (synthetic clicks, real ffmpeg)
    pa = load("prepare_audio_under_review", REPO / UNDER / "prepare_audio.py")
    version = subprocess.run(["ffmpeg", "-version"], capture_output=True, text=True, check=True).stdout.splitlines()[0]
    probe = scratch / "clicks"
    probe.mkdir()
    rows8, ok8 = [], True
    lengths = sorted({t["frames"] for t in inv["takes"]} | {96001, 96000})
    for n in lengths:
        clicks = (2, n // 3 * 2, (n - 12) // 2 * 2)  # even 48 kHz positions, so c/2 is a whole 24 kHz sample
        pcm = array("h", [0]) * n
        for c in clicks:
            pcm[c] = 20000
        write_wav(probe / f"src-{n}.wav", 48000, pcm.tobytes())
        subprocess.run(["ffmpeg", *pa.FFMPEG_ARGS, "-i", str(probe / f"src-{n}.wav"), *pa.FFMPEG_OUT, str(probe / f"dst-{n}.wav")],
                       check=True)
        params, out = read_pcm(probe / f"dst-{n}.wav")
        got = array("h", out)
        peaks = [max(range(max(0, c // 2 - 40), min(len(got), c // 2 + 40)), key=lambda i: abs(got[i])) for c in clicks]
        want, slack = pa.expected_frames({"rate": 48000, "frames": n})
        ok8 = ok8 and params == (1, 2, 24000) and len(got) == want == (n + 1) // 2 and slack == 0 \
            and peaks == [c // 2 for c in clicks]
        rows8.append(f"{n} -> {len(got)}")
    check("K08-decoder-timeline", ok8,
          f"{version}; root's arguments {pa.FFMPEG_ARGS + ['-i', 'SRC'] + pa.FFMPEG_OUT}: frames = ceil(n/2) = the "
          f"helper's expected_frames ({', '.join(rows8)}); clicks at 48 kHz sample c land at exactly c/2, so recognizer "
          f"seconds on the 48 kHz file and sentence samples on the 24 kHz decode share one timeline")

    # ------------------------------------------------------------------ K09 decode receipt shape vs canonical F2
    tree = ast.parse((REPO / UNDER / "prepare_audio.py").read_text(encoding="utf-8"))
    decode = next(n for n in tree.body if isinstance(n, ast.FunctionDef) and n.name == "cmd_decode")
    dicts = [n for n in ast.walk(decode) if isinstance(n, ast.Dict)]
    keysets = [{k.value for k in d.keys if isinstance(k, ast.Constant)} for d in dicts]
    record_keys = next(k for k in keysets if "wav_sha256" in k)
    receipt_keys = next(k for k in keysets if "records" in k)
    v = b["voice"]
    shaped = {k: None for k in receipt_keys}
    shaped.update(source=v["receipt_source"], voice=v["name"], provider=v["provider"], records=[{k: "x" for k in record_keys}])
    voice = asm.external_ui_voice(shaped)
    try:
        asm.external_ui_voice(dict(shaped, records=[dict(shaped["records"][0], request="x", request_sha256="y")]))
        f2 = "accepted"
    except ValueError as error:
        f2 = str(error)
    check("K09-receipt-shape", set(asm.EXTERNAL_RECORD_KEYS) <= record_keys and not record_keys & {"request", "request_sha256", "voice"}
          and "schema" not in receipt_keys and voice == UI_OLIVER and f2.startswith("legacy Oliver receipt states"),
          f"cmd_decode writes receipt keys {sorted(receipt_keys)} and record keys {sorted(record_keys)}: every canonical "
          f"EXTERNAL_RECORD_KEY, no request; source {v['receipt_source']!r}, voice {v['name']!r} -> {voice!r}. The same "
          f"receipt with a request on its record is refused by F2 ({f2!r}), so a request binding can only arrive as v2")

    # ------------------------------------------------------------------ K10 synthetic end to end through the canonical bytes
    rng = random.Random(20261105)
    dec = scratch / "decoded"
    dec.mkdir()
    (scratch / "source-stand-in.bin").write_bytes(b"synthetic source stand-in, not audio")
    source = scratch / "source-stand-in.bin"
    recs, span_units, pcms, ends_by_cue = [], [], {}, {}
    for unit in units_doc["units"]:
        cue = unit["line_ids"][0]
        n_sent = len(asm.sentences(new[cue]))
        length = 24000 + 7919 * n_sent + rng.randrange(1, 997)
        pcm = rng.randbytes(2 * length)
        wav = dec / f"{cue}.wav"
        write_wav(wav, 24000, pcm)
        pcms[cue] = pcm
        ends = [length * (k + 1) // n_sent for k in range(n_sent)]
        ends_by_cue[cue] = ends
        row = {k: None for k in record_keys}
        row.update(cue=cue, text=new[cue], source=str(source), source_sha256=sha_file(source), wav=str(wav),
                   wav_sha256=sha_file(wav), samples=length, rate=24000)
        recs.append(row)
        span_units.append({"id": unit["id"], "wav": str(wav), "sha256": sha_file(wav),
                           "generation_receipt": str(scratch / "decode-receipt.json"), "sentence_ends": ends,
                           "gap_after_samples": [0] * n_sent})
    receipt = {k: None for k in receipt_keys}
    receipt.update(source=v["receipt_source"], voice=v["name"], provider=v["provider"], records=recs,
                   claim="TEST FIXTURE: synthetic noise, LavenderPike independent check")
    (scratch / "decode-receipt.json").write_text(json.dumps(receipt, indent=1), encoding="utf-8")
    spans = {"schema_version": 1, "units_sha256": sha_file(units_path), "units": span_units, "direction_dispositions": disp,
             "boundary_review": {"reviewer": "LavenderPike TEST", "basis": "TEST FIXTURE: synthetic noise; whole takes, no "
                                 "samples removed, no silence inserted", "listening": "unreviewed"}}
    (scratch / "spans.json").write_text(json.dumps(spans, indent=1), encoding="utf-8")
    narration = scratch / "narration"
    r = subprocess.run([sys.executable, "-I", "-B", str(ws / "tools/coherent_narration.py"), "assemble", "--script",
                        str(REPO / crate / "script.tsv"), "--units", str(units_path), "--require-directions", "--spans",
                        str(scratch / "spans.json"), "--out", str(narration)], cwd=REPO, capture_output=True, text=True)
    same, onsets_ok = [], True
    if r.returncode == 0:
        asm.verify(narration, Path(REPO / crate / "script.tsv"), units_path, require_directions=True)
        rep = json.loads((narration / "report.json").read_text())
        for cue in new:
            same.append(read_pcm(narration / f"{cue}.wav") == ((1, 2, 24000), pcms[cue]))
            t, starts = 0, []
            for row in rep["lines"][cue]:
                starts.append(t)
                t += row["out_samples"] + row["gap_after_samples"]
            onsets_ok = onsets_ok and starts == [0] + ends_by_cue[cue][:-1] and t == len(pcms[cue]) // 2
    check("K10-whole-take-assembly", r.returncode == 0 and len(same) == 80 and all(same) and onsets_ok,
          f"assemble (byte copy of canonical {CANONICAL['tools/coherent_narration.py'][:8]}, --require-directions) and "
          f"verify on 80 synthetic takes with Bronze's receipt and span shapes and the 15 proposed native holds: every "
          f"assembled cue equals its decoded take sample for sample ({sum(same)}/80), report onsets equal the span "
          f"ends, nothing removed or inserted; " + (r.stderr.strip()[-300:] if r.returncode else r.stdout.strip()))

    # Negatives on the same world, in process.
    def assembler_verdict(manifest):
        try:
            lines = {}
            for u, rec in zip(units_doc["units"], manifest["units"]):
                for cue, row, _pcm in asm.unit_spans(u, new, rec):
                    lines.setdefault(cue, []).append(row)
            directions = asm.review_directions(contract, units_doc["units"], manifest)
            asm.check_dispositions(directions, lines)
            return "accepted"
        except ValueError as error:
            return str(error)

    trimmed = json.loads(json.dumps(spans))
    trimmed["units"][5]["sentence_ends"][-1] -= 1
    flipped = json.loads(json.dumps(spans))
    flipped["direction_dispositions"][0]["kind"] = "authored_silence"
    gap = json.loads(json.dumps(spans))
    multi = next(i for i, u in enumerate(gap["units"]) if len(u["gap_after_samples"]) >= 2
                 and units_doc["units"][i]["line_ids"][0] not in {r_["cue"] for r_ in resolved})
    gap["units"][multi]["gap_after_samples"][0] = 2400
    verdicts = [assembler_verdict(m) for m in (spans, trimmed, flipped, gap)]
    gap_cue = units_doc["units"][multi]["line_ids"][0]
    rebuilt = b"".join(pcm for u, rec in zip(units_doc["units"][multi:multi + 1], gap["units"][multi:multi + 1])
                       for _c, _row, pcm in asm.unit_spans(u, new, rec))
    check("K11-pause-contract-negatives", verdicts[0] == "accepted"
          and verdicts[1].startswith("source spans must cover every selected target sample")
          and verdicts[2].startswith("direction disposition contradicts assembled samples")
          and verdicts[3] == "accepted" and rebuilt != pcms[gap_cue] and len(rebuilt) == len(pcms[gap_cue]) + 4800,
          f"canonical assembler: a one-sample trim -> {verdicts[1]!r}; a hold relabelled authored_silence with zero gaps "
          f"-> {verdicts[2]!r}; 0.1 s of silence inserted after {gap_cue} sentence 0 (no direction there) -> "
          f"{verdicts[3]}: the assembler alone does not forbid invented silence. The guards are the helper's all-zero "
          f"gap_after_samples and its timings step, which refuses an assembled cue whose PCM is not the decoded take "
          f"(that cue's rebuilt PCM is 4800 bytes longer than its take)")

    # ------------------------------------------------------------------ K12 the helper's selection and answer contract, in process
    pb = pa.load_binding(BINDING)
    _files, questions, recordings, inputs = pa.build_report(pb)
    clar_dir = inv_spec["takes_dir"]
    first = {}
    for e in recordings:
        first.setdefault(e["cue"], e)
    takes = {c: (f"{clar_dir}/{c}-download01.wav" if c in CHANGED else first[c]["file"]) for c in new}
    sel_base = {"schema": "audio-preparation-selection-v1", "binding_sha256": pb["sha256"],
                "script_sha256": pb["pins"][pb["script"]], "by": "LavenderPike TEST FIXTURE (not root)",
                "basis": "TEST FIXTURE ONLY: first recording of each cue; nobody chose or heard anything"}

    def try_selection(tag, takes_, **extra):
        path = scratch / f"selection-{tag}.json"
        path.write_text(json.dumps(dict(sel_base, takes=takes_, **extra)), encoding="utf-8")
        try:
            return pa.load_selection(pb, str(path), inputs, None)[1]
        except pa.Refused as error:
            return str(error)

    chosen = try_selection("full", takes)
    bare = try_selection("bare", dict(takes, **{"04_4": "04_4-download01.wav"}))
    old_path = next(s for s in superseded if "/04_4-" in s)
    stale_sel = try_selection("superseded", dict(takes, **{"04_4": old_path}))
    other = try_selection("stage08", takes, binding_sha256=sha_file(REPO / OLD_BINDING))
    check("K12-selection-contract", isinstance(chosen, dict) and chosen["04_4"]["rel"] == takes["04_4"]
          and chosen["08_5"]["rel"] == takes["08_5"] and "name the recording by its path" in bare
          and "superseded wording" in stale_sel and "not audio-preparation-selection-v1 for this binding" in other,
          f"80-cue fixture selection with 04_4/08_5 by clarity path -> chosen from oliver-clarity-takes01; a bare "
          f"04_4-download01.wav -> refused ({bare.splitlines()[0][:90]}...); the ui-takes01 path -> refused (superseded "
          f"wording); the stage08 binding hash -> refused")

    e85 = next(e for e in recordings if e["path"] == takes["08_5"])
    s4, mark = e85["sentences"][4], e85["phrase_anchors"][0]
    fixture = {"by": "LavenderPike TEST FIXTURE (not root)", "basis": "TEST FIXTURE ONLY: nothing measured or heard"}

    def try_answers(tag, answers):
        path = scratch / f"answers-{tag}.json"
        path.write_text(json.dumps({"schema": "audio-preparation-answers-v1", "binding_sha256": pb["sha256"],
                                    "answers": answers}), encoding="utf-8")
        try:
            return pa.check_answers(pb, str(path), questions, recordings, chosen)
        except pa.Refused as error:
            return str(error)

    carried = try_answers("stage08-id", {"onset:08_5:s4:08_5-download01.wav": dict(fixture, decision="sample", sample_24k=480000)})
    moved = round(mark["asr_start_s"] * 24000) + 2400
    applied = try_answers("moved-s4", {s4["id"]: dict(fixture, decision="sample", sample_24k=moved)})
    frames24 = (e85["frames"] + 1) // 2
    starts, ends, marks = pa.resolved_points(e85, applied, frames24) if isinstance(applied, dict) else (None, None, [])
    kit_ok = all(0 <= m["seconds"] < frames24 / 24000 for m in marks) and all(
        a["byte"] < z["byte"] and a["seconds"] <= z["seconds"] for a, z in zip(marks, marks[1:]))
    check("K13-answers-and-mark-coupling", "answer to an unknown question or point onset:08_5:s4:08_5-download01.wav" in carried
          and isinstance(applied, dict) and ends is not None and marks[0]["byte"] == 267
          and marks[0]["seconds"] < ends[3] / 24000 and kit_ok,
          f"a stage08 answer id for 08_5 is refused as unknown; finding L1 reproduced: an answer moving "
          f"{s4['id'].split(':')[2]} of 08_5-download01 to sample {moved} ({moved / 24000:.3f} s, 0.1 s after the "
          f"'leaves a half turn' mark at {marks[0]['seconds'] if marks else None} s) is accepted by check_answers and "
          f"resolved_points, and the marks still pass the kit's own order and range rules, so the mark would precede "
          f"the start of its own sentence with no refusal")

    # Real-data margin for L1: how far each supported mark sits after its sentence's onset, against that onset's window.
    leads = []
    for e in report:
        text = new[e["cue"]]
        bounds = [len(text[:text.index(s)].encode()) for s in my_sentences(text)]
        for m in e["phrase_anchors"]:
            k = max(i for i, s in enumerate(bounds) if s <= m["byte"])
            s = e["sentences"][k]
            if k and m["status"] == "supported" and s["status"] == "supported":
                leads.append((round(m["asr_start_s"] - s["sample_24k"] / 24000, 3), round(s["asr_end_s"] - s["asr_start_s"], 3),
                              e["cue"], m["byte"]))
    leads.sort()
    check("K14-mark-margin", leads and all(lead > win for lead, win, _c, _b in leads),
          f"{len(leads)} supported marks inside sentences 1..n of Bronze's report: smallest lead over the sentence onset "
          f"{leads[0][0]} s ({leads[0][2]} byte {leads[0][3]}), while the largest onset word segment among them is "
          f"{max(w for _l, w, _c, _b in leads)} s; L1 needs a correction well beyond the stated uncertainty on these takes")

    print(f"INDEPENDENT: {'PASS' if all(results) else 'FAIL'} ({sum(results)}/{len(results)})")
    return 0 if all(results) else 1


if __name__ == "__main__":
    sys.exit(main())
