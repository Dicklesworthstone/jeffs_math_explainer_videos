"""complex-stage04-audio-reuse01: which complex ElevenLabs Oliver recordings the stage04 script can reuse, by exact text.

    python3 -I -B checks/audio_reuse.py build <fresh-dir>
    python3 -I -B checks/audio_reuse.py verify <fresh-scratch>

Run both from the repo root. `build` writes the inventory into <fresh-dir>: recordings.json, cues.tsv,
derived-audio.tsv and summary.txt. `verify` builds again into the scratch folder and says whether each
file is byte-equal to the sealed inventory/.

The run is read-only: every lab file is opened "rb", nothing is written outside <fresh-dir>, and every
file read is stat-ed before and after the run, so a change by anything would show. It decodes no compressed
audio and runs no subprocess. PCM is read from the WAV data chunk with Python's `wave` module.

"current" means the take's submitted text is byte-equal to the stage04 cue text. It is not a judgement of
the take: nothing here is heard, transcribed, aligned or selected.
"""
import array
import difflib
import hashlib
import json
import pathlib
import re
import sys
import wave

REPO = pathlib.Path.cwd()
HERE = pathlib.Path(__file__).resolve().parent
OUT = HERE.parent
R = "narration_lab/third-wave-20261024/root"
EC = f"{R}/elevenlabs-complex-rebuild-20261104"
SIB = f"{R}/elevenlabs-complex-analysis-20261101"
FFR = f"{R}/five-film-release-20261027"
CR = f"{EC}/combined-repair-stage04/explainer_corpus/complex_analysis_visual"
RECEIPT04 = f"{EC}/combined-repair-stage04-integration01.json"
RECEIPT04_SHA = "295c27e3cd17f84f39bde8f7a191e7ec3a29035d6fcbe62d0f1a2bd6c94fb00b"
SCRIPT_SHA = "12442480d7e74ca49e31ca9f6f437715e3d7151b706ff4a3815139bfa67c0fc3"
PREP = f"{EC}/narration-request-preparation01"
AUDIO = (".wav", ".mp3", ".flac", ".m4a", ".ogg", ".opus", ".aac")

READ = {}  # rel -> (size, mtime_ns) when first read


def rel(p):
    return str(pathlib.Path(p).relative_to(REPO))


def note(p):
    p = pathlib.Path(p)
    st = p.stat()
    READ.setdefault(rel(p), (st.st_size, st.st_mtime_ns))


def raw(p):
    note(p)
    with open(p, "rb") as f:
        return f.read()


def text(p):
    return raw(p).decode("utf-8")


def sha(b):
    return hashlib.sha256(b).hexdigest()


def repo_rel(s):
    """A repo-relative form of a path a receipt wrote, absolute or not."""
    i = s.find("narration_lab/")
    return s[i:] if i >= 0 else s


def wav_info(p, peak=True):
    note(p)
    with open(p, "rb") as f, wave.open(f, "rb") as w:
        n = w.getnframes()
        pcm = w.readframes(n)
        info = {"channels": w.getnchannels(), "sample_width_bytes": w.getsampwidth(), "sample_rate": w.getframerate(),
                "frames": n, "seconds": round(n / w.getframerate(), 6), "pcm_sha256": sha(pcm)}
    if peak and info["sample_width_bytes"] == 2 and pcm:
        assert sys.byteorder == "little"  # WAV PCM is little-endian; array reads native order
        samples = array.array("h")
        samples.frombytes(pcm)
        info["peak_abs_sample"] = max(abs(min(samples)), abs(max(samples)))
    return info


def cue_rows(script):
    return {m.group(1): m.group(2) for m in re.finditer(r"^(\d\d_\d+)\t(.*)$", script, flags=re.M)}


def words(t):
    return len(t.split())


def sentences(t):
    return [s for s in re.split(r"(?<=[.?!]) ", t) if s]


def build(dest):
    dest = pathlib.Path(dest)
    assert not dest.exists(), "the output folder must be fresh"
    assert "narration_lab" not in dest.resolve().parts, "never write into the labs"
    log = []

    def say(*a):
        log.append(" ".join(str(x) for x in a))

    # ---------------------------------------------------------------- the script the inventory is for
    receipt = json.loads(text(REPO / RECEIPT04))
    hashed = {row["path"]: row["sha256"] for row in receipt["files"]}
    script_b = raw(REPO / CR / "script.tsv")
    plan_b = raw(REPO / CR / "narration_units.json")
    assert sha(raw(REPO / RECEIPT04)) == RECEIPT04_SHA, "stage04 receipt is not the one inventoried"
    assert sha(script_b) == SCRIPT_SHA == hashed["explainer_corpus/complex_analysis_visual/script.tsv"]
    assert hashed["explainer_corpus/complex_analysis_visual/narration_units.json"] == sha(plan_b)
    cues = cue_rows(script_b.decode("utf-8"))
    assert len(cues) == 105 and len(set(cues.values())) == 105
    by_text = {t: c for c, t in cues.items()}
    say(f"stage04 receipt {RECEIPT04_SHA[:8]}; script.tsv {SCRIPT_SHA[:8]}: {len(cues)} cues, all texts distinct")

    # ---------------------------------------------------------------- recordings and their receipts
    audio = sorted(p for lab in (EC, SIB) for p in (REPO / lab).rglob("*") if p.is_file() and p.suffix.lower() in AUDIO)
    claims = {}  # recording rel path -> claim from its receipt
    for p in sorted((REPO / SIB).glob("*.json")):
        r = json.loads(text(p))
        if not isinstance(r, dict) or not isinstance(r.get("records"), list) or "submitted_text" not in r:
            continue
        if not all(isinstance(x, dict) and str(x.get("path", "")).lower().endswith(AUDIO) for x in r["records"]):
            say(f"not a take receipt (its records name no audio path): {rel(p)}")
            continue
        src_ok = None
        if r.get("source_text_path"):
            src = raw(REPO / r["source_text_path"])
            src_ok = src.decode("utf-8").strip() == r["submitted_text"] and r.get("source_sha256", sha(src)) == sha(src)
        for k, rec in enumerate(r["records"]):
            claims[repo_rel(rec["path"])] = {
                "receipt": rel(p), "ordinal": k + 1, "text": r["submitted_text"], "source_text_matches": src_ok,
                "source_text": r.get("source_text_path"),
                "settings": {x: r.get(x) for x in ("voice", "model", "stability", "similarity", "clone_strength")},
                "declared": {"sha256": rec.get("sha256"), "frames": rec.get("frames"),
                             "sample_rate": rec.get("sample_rate", rec.get("rate")),
                             "sample_width_bytes": rec.get("sample_width_bytes", rec.get("sample_width")),
                             "channels": rec.get("channels"), "pcm_sha256": None},
                "download_name": pathlib.PurePosixPath(rec.get("source_download", "")).name,
            }
    for p in sorted((REPO / EC).glob("*/*-downloads*.json")):
        d = json.loads(text(p))
        req_p = next(q for q in p.parent.glob("*-request*.json") if sha(raw(q)) == d["request_sha256"])
        req = json.loads(text(req_p))
        txt = p.parent / f"{req['cue']}.txt"
        src_ok = text(txt) == req["text"] and sha(req["text"].encode()) == req["text_sha256"]
        for k, rec in enumerate(d["downloads"]):
            claims[repo_rel(rec["retained"])] = {
                "receipt": rel(p), "request": rel(req_p), "ordinal": k + 1, "text": req["text"], "source_text_matches": src_ok,
                "source_text": rel(txt),
                "request_script_sha256": req["script_sha256"], "request_cue": req["cue"],
                "settings": {x: req.get(x) for x in ("voice", "model", "stability", "similarity", "clone_strength")},
                "declared": {"sha256": rec["sha256"], "frames": rec["frames"], "sample_rate": rec["sample_rate"],
                             "sample_width_bytes": None, "channels": None, "pcm_sha256": rec["pcm_sha256"]},
                "download_name": pathlib.PurePosixPath(rec["source"]).name,
                "receipt_selection": d.get("selection"), "receipt_fine_sync": d.get("fine_sync"),
                "receipt_listening": d.get("listening"),
            }
    recs, loose = [], []
    for p in audio:
        rp = rel(p)
        claim = claims.get(rp)
        if not claim:
            # a live lab can hold audio before its receipt: hashed, never matched by name, never current
            info = wav_info(p) if p.suffix.lower() == ".wav" else {}
            loose.append({"path": rp, "lab": pathlib.PurePosixPath(rp).parts[3], "receipt": None, "sha256": sha(raw(p)), **info,
                         "stage04_cue": None, "status": "no-receipt", "why": "no receipt names this file, so its text is unknown"})
            continue
        info = wav_info(p)
        file_sha = sha(raw(p))
        dec = claim["declared"]
        # None: the receipt does not state that field
        agree = {k: (None if dec[k] is None else dec[k] == (file_sha if k == "sha256" else info[k])) for k in dec}
        t = claim["text"]
        cue = by_text.get(t)
        row = {"path": rp, "lab": pathlib.PurePosixPath(rp).parts[3], "receipt": claim["receipt"], "ordinal": claim["ordinal"],
               "download_name": claim["download_name"], "sha256": file_sha, **info,
               "receipt_fields_agree": agree, "submitted_text_sha256": sha(t.encode()), "submitted_words": words(t),
               "source_text_matches_submitted": claim["source_text_matches"], "settings": claim["settings"],
               "stage04_cue": cue, "status": "current" if cue else "superseded",
               "why": ("submitted text is byte-equal to stage04 " + cue) if cue else "submitted text equals no stage04 cue",
               "shares_whole_sentences_with": sorted({c for s in sentences(t) for c, ct in cues.items() if s in sentences(ct)}),
               "contains_whole_cue": sorted(c for c, ct in cues.items() if ct in t and ct != t),
               "submitted_text": t}
        tok = re.search(r"(?:Steady_|__)([a-z0-9_]+)\.wav$", re.sub(r" \(\d+\)(?=\.wav$)", "", claim["download_name"]))
        row["download_name_setting_tokens"] = tok and tok.group(1)
        if claim.get("source_text"):
            row["source_text"] = claim["source_text"]
        for k in ("request", "request_script_sha256", "request_cue", "receipt_selection", "receipt_fine_sync", "receipt_listening"):
            if k in claim:
                row[k] = claim[k]
        recs.append(row)
    assert set(claims) == {r["path"] for r in recs}, "a receipt names a recording that is not on disk"
    dupes = {}
    for r in recs:
        dupes.setdefault(r["pcm_sha256"], []).append(r["path"])
    for r in recs:
        r["same_pcm_as"] = [q for q in dupes[r["pcm_sha256"]] if q != r["path"]]
    say(f"recordings: {len(recs)} WAV ({sum(r['lab'] == 'elevenlabs-complex-rebuild-20261104' for r in recs)} rebuild, "
        f"{sum(r['lab'] == 'elevenlabs-complex-analysis-20261101' for r in recs)} sibling), "
        f"{len({r['receipt'] for r in recs})} receipts, {len({r['submitted_text_sha256'] for r in recs})} distinct submitted texts")
    stated = [v for r in recs for v in r["receipt_fields_agree"].values() if v is not None]
    say(f"receipt fields stated: {len(stated)}; agree with the file: {sum(stated)}; disagree: {len(stated) - sum(stated)}")
    say("pcm_sha256 method reproduces every downloads receipt in the rebuild lab:",
        all(r["receipt_fields_agree"]["pcm_sha256"] is True for r in recs if r["lab"].startswith("elevenlabs-complex-rebuild")))
    say("submitted text equals the receipt's source text file (where one is named):",
        sorted({str(r["source_text_matches_submitted"]) for r in recs}))
    tally = {}
    for r in recs:
        key = (r["download_name_setting_tokens"], r["settings"]["stability"], r["settings"]["similarity"])
        tally[key] = tally.get(key, 0) + 1
    say("download-name setting tokens vs receipt stability/similarity (recorded, not interpreted):",
        sorted((str(k[0]), k[1], k[2], n) for k, n in tally.items()))
    say("current:", sorted({(r["stage04_cue"], r["path"].rsplit("/", 1)[1]) for r in recs if r["status"] == "current"}))
    say("superseded:", sum(r["status"] == "superseded" for r in recs), "recordings")
    say("audio no receipt names (text unknown, never current):", [r["path"] for r in loose] or "none")
    say("pairs whose two takes are byte-identical PCM:", sorted(pathlib.PurePosixPath(v[0]).name + " = " + pathlib.PurePosixPath(v[1]).name
                                                         for v in dupes.values() if len(v) == 2))
    say("a stage04 cue inside a longer take:", sorted({c for r in recs for c in r["contains_whole_cue"]}) or "none")
    say("stage04 cues sharing a whole sentence with a superseded take:",
        sorted({c for r in recs if r["status"] == "superseded" for c in r["shares_whole_sentences_with"]}) or "none")

    # ---------------------------------------------------------------- every other text record in the two labs
    # prepared-only folders: a requests JSON whose own status says text preparation with no synthesis, and no request
    # in it marked as submitted. Everything under such a folder is prepared text, never a generation.
    prepared = {}
    for q in sorted((REPO / EC).glob("*/requests*.json")):
        d = json.loads(text(q))
        status = str(d.get("status", d.get("state", "")))
        flags = sorted({str(x.get("submitted_in_this_preparation")) for x in d.get("requests", [])})
        if "prepar" in status.lower() and "no synthesis" in status.lower() and "True" not in flags:
            folder = rel(q.parent)
            texts = {}
            for x in d["requests"]:
                b = raw(REPO / x["text_path"])
                assert sha(b) == x["text_sha256"] and x["text_path"].startswith(folder + "/"), x["cue"]
                texts[x["cue"]] = b.decode("utf-8")
            prepared[folder] = {"requests": rel(q), "script_sha256": d.get("script_sha256"), "status": status,
                                "submitted_flags": flags, "texts": texts, "self_check": d["requests"]}
        else:
            say(f"requests file not prepared-only by its own status: {rel(q)}")
    for folder, info in prepared.items():
        t = info["texts"]
        claims_ok = all(x.get("matches_preparation01") is None or
                        x["matches_preparation01"] == (prepared.get(PREP, {}).get("texts", {}).get(x["cue"]) == t[x["cue"]])
                        for x in info["self_check"])
        say(f"prepared-only folder {folder}: requests {info['requests'].rsplit('/', 1)[1]}, made for script "
            f"{str(info['script_sha256'])[:8]}, {len(t)} texts, {sum(t.get(c) == ct for c, ct in cues.items())} byte-equal to stage04, "
            f"differ: {sorted(c for c, ct in cues.items() if t.get(c) != ct)}; submitted flags {info['submitted_flags']}; "
            f"its matches_preparation01 claims hold: {claims_ok}")
        say(f"  status, as the folder states it: {info['status']}")

    # recording: the receipts, requests and cue text of a retained recording; prepared-only: the folders above;
    # service: any other file named like a request, submission, receipt, download or generation record (a generation
    # whose audio is not on disk would show here); plans and reviews: everything else (units, maps, reviews)
    found = {"recording": set(), "prepared-only": set(), "service": set(), "plans and reviews": set()}
    rec_files = {r["receipt"] for r in recs} | {r["request"] for r in recs if "request" in r}
    # a folder holding a retained recording of cue c: its c.txt and c-* records belong to that recording
    rec_cue_dirs = {(r["path"].rsplit("/", 1)[0], r["request_cue"]) for r in recs if r.get("request_cue")}
    service = re.compile(r"(request|submi|receipt|download|generation|tts|history|take|launch)[^/]*$", re.I)

    def walk(obj, where):
        if isinstance(obj, dict):
            for k, v in obj.items():
                walk(v, where) if not isinstance(v, str) else hit(v, where)
        elif isinstance(obj, list):
            for v in obj:
                walk(v, where) if not isinstance(v, str) else hit(v, where)

    def hit(v, where):
        c = by_text.get(v.strip())
        if not c:
            return
        p_ = REPO / where
        beside_request = any(p_.parent.glob(f"{p_.stem}-request*.json")) if p_.suffix == ".txt" else False
        folder_, name_ = where.rsplit("/", 1)
        of_recording = any(folder_ == f and (name_ == f"{c_}.txt" or name_.startswith(f"{c_}-")) for f, c_ in rec_cue_dirs)
        kind = ("recording" if where in rec_files or of_recording else
                "prepared-only" if any(where.startswith(f + "/") for f in prepared) else
                "service" if service.search(where) or beside_request else "plans and reviews")
        found[kind].add((c, where))

    for lab in (EC, SIB):
        for p in sorted((REPO / lab).rglob("*")):
            if not p.is_file() or p.suffix not in (".json", ".txt", ".tsv", ".md"):
                continue
            rp = rel(p)
            if p.name in ("script.tsv", "storyboard.md") or "script-as-reviewed" in p.name:
                continue  # scripts and storyboards are not service records
            body = text(p)
            if p.suffix == ".json":
                try:
                    walk(json.loads(body), rp)
                except json.JSONDecodeError:
                    hit(body, rp)
            else:
                hit(body, rp)
    for kind, items in found.items():
        say(f"exact stage04 cue texts in {kind} records: {len({w for _, w in items})} files, {len({c for c, _ in items})} cues")
    say("recording records with an exact stage04 cue text:", sorted(found["recording"]))
    say("service records with an exact stage04 cue text and no retained recording:", sorted(found["service"]) or "none")
    # what each such cue's request and submission records say, in their own fields
    service_state = {}
    for c, where in sorted(found["service"]):
        folder = (REPO / where).parent
        parts = []
        for q in sorted(folder.glob(f"{c}-request*.json")) + sorted(folder.glob(f"{c}-submission*.json")):
            d = json.loads(text(q))
            keep = {k: d[k] for k in ("submitted", "submitted_once", "downloads_pending", "selection", "listening") if k in d}
            if "request_sha256" in d:
                keep["names_request"] = next((r.name for r in folder.glob(f"{c}-request*.json") if sha(raw(r)) == d["request_sha256"]), None)
            parts.append(f"{rel(q).rsplit('/', 2)[-2]}/{q.name} {json.dumps(keep, sort_keys=True)}")
        service_state[c] = sorted(set(service_state.get(c, [])) | set(parts))
    for c, parts in sorted(service_state.items()):
        say(f"  {c}: " + "; ".join(parts))
    have = {r["stage04_cue"] for r in recs if r["stage04_cue"]}
    # every submission record in the rebuild lab, whatever its text: the request it names and what was kept of it
    subs = []
    for q in sorted((REPO / EC).glob("*/*-submission*.json")):
        d = json.loads(text(q))
        req_q = next((r for r in q.parent.glob("*-request*.json") if sha(raw(r)) == d.get("request_sha256")), None)
        req = json.loads(text(req_q)) if req_q else {}
        kept = sorted(pathlib.PurePosixPath(r["path"]).name for r in recs if req_q and r.get("request") == rel(req_q))
        cue_ = by_text.get(req.get("text", "").strip())
        subs.append((rel(q), req_q and req_q.name, req.get("cue"), cue_, d.get("submitted_once", d.get("submitted")), kept))
        say(f"submission record {rel(q).rsplit('/', 2)[-2]}/{q.name}: names request {req_q and req_q.name}, "
            f"its text {'is stage04 ' + cue_ if cue_ else 'is no stage04 cue'}; submitted {subs[-1][4]}; retained recordings of that request: "
            f"{kept or 'none'}")
    pending = sorted({c for c, parts in service_state.items() if c not in have
                      and any('"submitted_once": true' in x or '"submitted": true' in x for x in parts)}
                     | {s[3] for s in subs if s[3] and s[4] and not s[5]})
    say("stage04 cues submitted to the service with no retained recording (pending-download):", pending or "none")

    # ---------------------------------------------------------------- prepared texts and the stage04 plan
    assert PREP in prepared, "request-preparation01 is no longer prepared-only by its own status"
    prep = prepared[PREP]["texts"]
    stage04_prep = {f: i["texts"] for f, i in prepared.items() if i["script_sha256"] == SCRIPT_SHA}
    plan = json.loads(plan_b)
    plan_units = {}

    def units(obj):
        if isinstance(obj, dict):
            if isinstance(obj.get("id"), str) and obj["id"].startswith("oliver_") and isinstance(obj.get("text"), str):
                plan_units[obj["id"].removeprefix("oliver_")] = obj["text"]
            for v in obj.values():
                units(v)
        elif isinstance(obj, list):
            for v in obj:
                units(v)

    units(plan)
    plan_pins = sorted(set(re.findall(r'"script_sha256":\s*"([0-9a-f]{64})"', plan_b.decode("utf-8"))))
    say(f"prepared-only folders made for the stage04 script: {sorted(stage04_prep) or 'none'}")
    say(f"stage04 plan narration_units.json {sha(plan_b)[:8]} pins script {[x[:8] for x in plan_pins]}; units {len(plan_units)}; "
        f"unit text differs from the script at {sorted(c for c, t in cues.items() if plan_units.get(c) != t)}")

    # ---------------------------------------------------------------- the cue list
    rows = ["cue\tstatus\twords\ttext_sha256\tcurrent_recordings\tprepared_text_01\t"
            + "".join(f"prepared_{f.rsplit('/', 1)[1]}\t" for f in sorted(stage04_prep)) + "stage04_plan_unit\tnote"]
    for c, t in cues.items():
        cur = [r for r in recs if r["stage04_cue"] == c]
        note_ = []
        if c == "15_2":
            note_.append("changed by review-fixes01 F3 (1e24aba8 -> 12442480); no take of either text")
        if cur:
            r0 = cur[0]
            note_.append(f"{len(cur)} takes in {r0['path'].rsplit('/', 2)[-2]}, request made for script "
                         f"{str(r0.get('request_script_sha256'))[:8]}; receipt selection {json.dumps(r0.get('receipt_selection'))}, "
                         f"listening {json.dumps(r0.get('receipt_listening'))}, fine_sync {json.dumps(r0.get('receipt_fine_sync'))}")
        if c in service_state and not cur:
            note_.append("service records: " + "; ".join(service_state[c]))
        old = [r for r in recs if r["status"] == "superseded" and c in r["shares_whole_sentences_with"]]
        if old:
            note_.append("shares one whole sentence with superseded " + ", ".join(sorted({pathlib.PurePosixPath(r['path']).name for r in old}))
                         + "; not a whole-cue take")
        rows.append("\t".join([c, "current" if cur else "pending-download" if c in pending else "missing", str(words(t)), sha(t.encode()),
                               ";".join(f"{pathlib.PurePosixPath(r['path']).name}:{r['pcm_sha256'][:16]}" for r in cur) or "-",
                               "exact" if prep.get(c) == t else "differs",
                               *("exact" if stage04_prep[f].get(c) == t else "differs" for f in sorted(stage04_prep)),
                               "exact" if plan_units.get(c) == t else "differs",
                               "; ".join(note_) or "-"]))
    n_cur = sum(1 for line in rows[1:] if line.split("\t")[1] == "current")
    assert n_cur == len(have) and not set(pending) & have
    say(f"cues: {n_cur} current, {len(pending)} pending-download, {len(cues) - n_cur - len(pending)} missing; "
        f"words {sum(map(words, cues.values()))} in all, {sum(words(t) for c, t in cues.items() if c not in have)} without a retained recording")
    for ch in sorted({c.split("_")[0] for c in cues}):
        mine = [c for c in cues if c.split("_")[0] == ch]
        gone = [c for c in mine if c not in have]
        say(f"  chapter {ch}: {len(mine)} cues, {len(gone)} missing ({sum(words(cues[c]) for c in gone)} words)"
            + (f"; current: {sorted(set(mine) - set(gone))}" if len(gone) < len(mine) else ""))

    # ---------------------------------------------------------------- derived audio (not recordings)
    known_file = {r["sha256"]: r["path"] for r in recs}
    known_pcm = {r["pcm_sha256"]: r["path"] for r in recs}
    drows = ["path\tbytes\tsha256\tsample_rate\tchannels\tframes\tpcm_sha256\tequals_recording"]
    derived = sorted(p for p in (REPO / FFR).rglob("*") if p.is_file() and p.suffix.lower() in AUDIO
                     and any("complex" in part for part in p.relative_to(REPO / FFR).parts))
    same = 0
    for p in derived:
        b = raw(p)
        fs = sha(b)
        info = {}
        if p.suffix.lower() == ".wav":
            try:
                info = wav_info(p, peak=False)
            except (wave.Error, EOFError):
                info = {}
        eq = known_file.get(fs) or (known_pcm.get(info.get("pcm_sha256")) and "pcm of " + known_pcm[info["pcm_sha256"]]) or "-"
        same += eq != "-"
        drows.append("\t".join([rel(p), str(len(b)), fs, str(info.get("sample_rate", "-")), str(info.get("channels", "-")),
                                str(info.get("frames", "-")), info.get("pcm_sha256", "-"), eq]))
    say(f"derived complex audio in five-film-release: {len(derived)} files, {sum(int(r.split(chr(9))[1]) for r in drows[1:]) / 1e6:.1f} MB; "
        f"equal to a recording (file or PCM): {same}")

    # ---------------------------------------------------------------- preservation
    changed = []
    for rp, (size, mtime) in sorted(READ.items()):
        st = (REPO / rp).stat()
        if (st.st_size, st.st_mtime_ns) != (size, mtime):
            changed.append(rp)
    say(f"files read: {len(READ)}; changed during the run: {changed or 'none'}")
    assert not changed

    dest.mkdir(parents=True)
    (dest / "recordings.json").write_text(json.dumps(recs + loose, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    (dest / "cues.tsv").write_text("\n".join(rows) + "\n", encoding="utf-8")
    (dest / "derived-audio.tsv").write_text("\n".join(drows) + "\n", encoding="utf-8")
    (dest / "summary.txt").write_text("\n".join(log) + "\n", encoding="utf-8")
    print("\n".join(log))


def verify(scratch):
    scratch = pathlib.Path(scratch)
    build(scratch / "inventory")
    ok = True
    for name in ("recordings.json", "cues.tsv", "derived-audio.tsv", "summary.txt"):
        now, sealed = (scratch / "inventory" / name).read_bytes(), (OUT / "inventory" / name).read_bytes()
        ok &= now == sealed
        print(f"{name}: byte-equal to the sealed inventory: {now == sealed}")
        if now != sealed and name != "recordings.json":
            # the labs are live: show what changed since the seal, line by line
            print("".join(difflib.unified_diff(sealed.decode("utf-8").splitlines(keepends=True), now.decode("utf-8").splitlines(keepends=True),
                                               fromfile=f"sealed/{name}", tofile=f"now/{name}", n=0))[:20000])
    print("VERIFY:", "PASS" if ok else "FAIL")
    return ok


if __name__ == "__main__":
    if sys.argv[1:2] == ["build"] and len(sys.argv) == 3:
        build(sys.argv[2])
    elif sys.argv[1:2] == ["verify"] and len(sys.argv) == 3:
        sys.exit(0 if verify(sys.argv[2]) else 1)
    else:
        sys.exit(__doc__)
