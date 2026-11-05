"""Independent synthetic scenarios for the ElevenLabs external-voice repair (WhiteSwan's review).

Usage: python3 -I -B scenarios.py REPO
    REPO is a fresh fixture root that already holds tools/coherent_narration.py,
    tools/narration_cache.py and tools/narration_delivery.py (one tool variant).

Prints one JSON object {scenario: outcome}. An outcome is
    "ok ..."                     every step completed (details follow)
    "refused <Type>: <message>"  the tool raised an error its CLI reports (ValueError, OSError, KeyError, TypeError, wave.Error)
    "crash <Type>: <message>"    any other exception (the CLI would print a traceback)
    "exit <n>: ..."              a narration_delivery CLI run

Everything is written under REPO. The PCM is synthetic integers; no audio is decoded, played or heard.
None of this code is shared with the repair bundle: the voice strings below are copied from the real
request files (frederick-stage05-01/*-request01.json and ui-takes01/04_4-request01.json), not from the patch.
"""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
from pathlib import Path
import struct
import subprocess
import sys
import types
import wave

REPO = Path(sys.argv[1]).resolve()
os.chdir(REPO)
FRED_LABEL, FRED_VOICE = "ElevenLabs UI, Frederick Surrey", "Frederick Surrey - Smooth and Velvety"
OLIVER_LABEL, OLIVER_VOICE = "ElevenLabs UI, Oliver", "Oliver - Clean, British and Steady"
V2 = "elevenlabs-ui-receipt-v2"
TOPIC = "demo"
TEXT = {"01_1": "A smooth curve rises. It fades on both sides.", "01_2": "Nothing breaks on the real line."}
FRAMES = {"01_1": 2400, "01_2": 1800}
ENDS = {"01_1": [1000, 2400], "01_2": [1800]}
GAPS = {"01_1": [120, 0], "01_2": [0]}
REFUSED = (ValueError, OSError, KeyError, TypeError, wave.Error)


def load(path, name):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


coherent = load(REPO / "tools/coherent_narration.py", "coherent_under_test")


def sha(path):
    return hashlib.sha256(Path(path).read_bytes()).hexdigest()


def put(path, value):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(value if isinstance(value, str) else json.dumps(value))
    return path


# ---------------------------------------------------------------- the fixture topic (written once)
TREE = REPO / "explainer_corpus" / TOPIC
SCRIPT = put(TREE / "script.tsv", "".join(f"{cue}\t{text}\n" for cue, text in TEXT.items()))
MAP = put(TREE / "directions.json", {"overrides": [{"video": TOPIC, "cue": "01_1", "after": "smooth curve"}]})
EVIDENCE = put(TREE / "disposition-evidence.txt", "Synthetic fixture; no listening.\n")
UNITS = put(TREE / "narration_units.json", {
    "schema_version": 1, "script_sha256": sha(SCRIPT),
    "units": [{"id": "u" + cue, "line_ids": [cue], "text": text} for cue, text in TEXT.items()],
    "direction_contract": {"schema_version": 1, "topic": TOPIC, "script_sha256": sha(SCRIPT),
                           "map": {"path": f"explainer_corpus/{TOPIC}/directions.json", "sha256": sha(MAP)}}})
UNIT = {u["id"][1:]: u for u in json.loads(UNITS.read_text())["units"]}
SCRIPT_ROWS = coherent.read_script(SCRIPT)


def write_pcm(path, frames, seed):
    data = b"".join(struct.pack("<h", ((i * 37 + seed * 101) % 2000) - 1000) for i in range(frames))
    with wave.open(str(path), "wb") as w:
        w.setparams((1, 2, 24000, 0, "NONE", "not compressed"))
        w.writeframes(data)


def frames_of(path):
    with wave.open(str(path), "rb") as w:
        return w.getparams()[:3], w.readframes(w.getnframes())


def take(S, cue, seed):
    """A decoded 24 kHz WAV and a stand-in for the retained download it came from."""
    wav = S / f"{cue}.wav"
    write_pcm(wav, FRAMES[cue], seed)
    src = put(S / f"{cue}-download01.wav", f"synthetic stand-in for a retained 48 kHz download, {cue}, seed {seed}")
    return {"cue": cue, "text": TEXT[cue], "wav": str(wav), "wav_sha256": sha(wav), "samples": FRAMES[cue],
            "rate": 24000, "source": str(src), "source_sha256": sha(src)}


def request(S, cue, voice, name="request01"):
    """Same layout as the real request files: cue, text, voice plus settings."""
    return put(S / f"{cue}-{name}.json", {"cue": cue, "text": TEXT[cue], "voice": voice, "model": "Eleven v4",
                                          "stability": 0.5, "similarity": 0.75, "clone_strength": 0, "submitted": False})


def v2_row(S, cue, voice, seed, name="request01"):
    row = take(S, cue, seed)
    path = request(S, cue, voice, name)
    row.update(voice=voice, request=str(path), request_sha256=sha(path))
    return row


def v2(label, voice, rows):
    return {"schema": V2, "source": label, "provider": "ElevenLabs", "voice": voice, "records": rows}


def legacy(rows):
    return {"source": OLIVER_LABEL, "records": rows}


def native(S, cue, seed):
    """A native FrankenTTS receipt in the canonical test fixture's shape, for this one-cue unit."""
    wav = S / f"{cue}.wav"
    write_pcm(wav, FRAMES[cue], seed)
    assets = {"talker_sha256": "model", "codec_sha256": "codec", "voice_source_sha256": "voice",
              **{n: {"sha256": n} for n in ("vocab.json", "merges.txt", "tokenizer_config.json")}}
    generation = {"schema": "ftts.generation", "schema_version": 1, "run_id": "fixture", "state": "completed",
                  "execution": "inline", "requested": {"seed": 2},
                  "effective": {"seed": 2, "normalization": {"mode": "verbatim"}, "utterance_start": "fresh",
                                "sampling": {"mode": "production"}},
                  "output": {"samples": FRAMES[cue], "frames": 10, "frame_limit": 600, "sample_rate": 24000,
                             "stop_reason": "end_of_speech"},
                  "producer": {"executable_sha256": "exe"}, "voice": {"route": "xvector"}, "assets": assets}
    context = {"settings": {"normalize": "verbatim"}, "executable": {"sha256": "exe"},
               "voice": {"kind": "file", "input": {"sha256": "voice"}}, "model": "/synthetic/model.fttsq",
               "bundle": [{"path": "/synthetic/model.fttsq", "sha256": "model"},
                          {"path": "/synthetic/speech_tokenizer/model.safetensors", "sha256": "codec"}]
               + [{"path": "/synthetic/" + n, "sha256": n} for n in ("vocab.json", "merges.txt", "tokenizer_config.json")]}
    completion = {"run_id": "fixture", "event": "run_complete", "exit_code": 0, "samples": FRAMES[cue], "frames": 10}
    return {"wav": coherent.cache.wav_identity(wav), "returncode": 0, "generation": generation,
            "completion": completion, "request": {"seed": 2, "context": context, "submitted_text": TEXT[cue]}}


def span(S, cue, receipt_path, **extra):
    wav = S / f"{cue}.wav"
    return {"id": "u" + cue, "wav": str(wav), "sha256": sha(wav), "generation_receipt": str(receipt_path),
            "sentence_ends": ENDS[cue], "gap_after_samples": GAPS[cue], **extra}


def spans(S, records):
    return put(S / "spans.json", {
        "schema_version": 1, "units_sha256": sha(UNITS),
        "boundary_review": {"reviewer": "review fixture", "basis": "synthetic sample positions", "listening": "unreviewed"},
        "units": records,
        "direction_dispositions": [{"override_index": 0, "kind": "retained_source", "reason": "Synthetic fixture.",
                                    "evidence": {"path": str(EVIDENCE), "sha256": sha(EVIDENCE)}}]})


def outcome(fn):
    try:
        with contextlib.redirect_stdout(io.StringIO()):
            detail = fn()
        return "ok" + (f" {detail}" if detail else "")
    except REFUSED as error:
        return f"refused {type(error).__name__}: {error}"
    except Exception as error:  # noqa: BLE001 - the point is to see what the CLI would not catch
        return f"crash {type(error).__name__}: {error}"


def assemble(S, records):
    path = spans(S, records)
    coherent.assemble(types.SimpleNamespace(script=SCRIPT, units=UNITS, spans=path, out=S / "out",
                                            chapter=None, require_directions=True))


def verify(S):
    coherent.verify(S / "out", SCRIPT, UNITS, require_directions=True)


def pcm_check(S):
    """Rebuild each cue from my own WAVs and sample positions, independently of the tool."""
    for cue in TEXT:
        _, raw = frames_of(S / f"{cue}.wav")
        want, start = b"", 0
        for end, gap in zip(ENDS[cue], GAPS[cue]):
            want += raw[start * 2:end * 2] + b"\0\0" * gap
            start = end
        params, got = frames_of(S / "out" / f"{cue}.wav")
        if params != (1, 2, 24000) or got != want:
            return f"pcm=DIFFERS({cue})"
    return "pcm=exact"


def normalized_digest(S):
    """Outputs with this fixture's own path removed, so two tool variants can be compared."""
    out = S / "out"
    report = json.loads((out / "report.json").read_text().replace(str(REPO), "REPO"))
    assembly = json.loads((out / "assembly.json").read_text().replace(str(REPO), "REPO"))
    # These two hashes cover files that contain this fixture's absolute paths.
    if report["meta"].pop("spans_sha256") != sha(S / "spans.json") or assembly.pop("report_sha256") != sha(out / "report.json"):
        return "hashes=DIFFER"
    wavs = {cue: sha(out / f"{cue}.wav") for cue in TEXT}
    blob = json.dumps([report, assembly, wavs], sort_keys=True).encode()
    return hashlib.sha256(blob).hexdigest()[:16]


results = {}


def scenario(fn):
    S = REPO / "s" / fn.__name__
    S.mkdir(parents=True)
    results[fn.__name__] = fn(S)
    return fn


def absent(S):
    return " [out " + ("exists" if (S / "out").exists() else "absent") + "]"


# ---------------------------------------------------------------- positive controls
@scenario
def fred_shared(S):
    rows = [v2_row(S, cue, FRED_VOICE, n) for n, cue in enumerate(TEXT, 1)]
    receipt = put(S / "receipt.json", v2(FRED_LABEL, FRED_VOICE, rows))
    return outcome(lambda: (assemble(S, [span(S, c, receipt) for c in TEXT]), verify(S), pcm_check(S))[2])


@scenario
def fred_per_cue(S):
    records = []
    for n, cue in enumerate(TEXT, 1):
        receipt = put(S / f"{cue}-receipt.json", v2(FRED_LABEL, FRED_VOICE, [v2_row(S, cue, FRED_VOICE, n)]))
        records.append(span(S, cue, receipt))
    return outcome(lambda: (assemble(S, records), verify(S), pcm_check(S))[2])


@scenario
def oliver_legacy(S):
    receipt = put(S / "receipt.json", legacy([take(S, cue, n) for n, cue in enumerate(TEXT, 1)]))
    return outcome(lambda: (assemble(S, [span(S, c, receipt) for c in TEXT]), verify(S),
                            pcm_check(S) + " digest=" + normalized_digest(S))[2])


@scenario
def oliver_v2_with_legacy(S):
    a = put(S / "a.json", v2(OLIVER_LABEL, OLIVER_VOICE, [v2_row(S, "01_1", OLIVER_VOICE, 1)]))
    b = put(S / "b.json", legacy([take(S, "01_2", 2)]))
    return outcome(lambda: (assemble(S, [span(S, "01_1", a), span(S, "01_2", b)]), verify(S), pcm_check(S))[2])


@scenario
def native_only(S):
    records = [span(S, cue, put(S / f"{cue}-native.json", native(S, cue, n))) for n, cue in enumerate(TEXT, 1)]
    return outcome(lambda: (assemble(S, records), verify(S), pcm_check(S))[2])


@scenario
def extra_keys_ignored(S):
    row = v2_row(S, "01_1", FRED_VOICE, 1)
    row["note"] = "extra record key"
    body = dict(v2(FRED_LABEL, FRED_VOICE, [row]), note="extra receipt key")
    receipt = put(S / "receipt.json", body)
    return outcome(lambda: coherent.unit_spans(UNIT["01_1"], SCRIPT_ROWS, span(S, "01_1", receipt)) and None)


# ---------------------------------------------------------------- one narrator voice per assembly
@scenario
def mix_fred_oliver(S):
    a = put(S / "a.json", v2(FRED_LABEL, FRED_VOICE, [v2_row(S, "01_1", FRED_VOICE, 1)]))
    b = put(S / "b.json", legacy([take(S, "01_2", 2)]))
    return outcome(lambda: assemble(S, [span(S, "01_1", a), span(S, "01_2", b)])) + absent(S)


@scenario
def mix_external_native(S):
    a = put(S / "a.json", legacy([take(S, "01_1", 1)]))
    b = put(S / "b.json", native(S, "01_2", 2))
    return outcome(lambda: assemble(S, [span(S, "01_1", a), span(S, "01_2", b)])) + absent(S)


@scenario
def swap_after_assembly(S):
    """All-Frederick assembly; then 01_2's receipt becomes a self-consistent Oliver v2 receipt.
    unit_spans alone accepts the new receipt, so only the guard inside verify can see the mix."""
    receipts = {}
    for n, cue in enumerate(TEXT, 1):
        receipts[cue] = put(S / f"{cue}-receipt.json", v2(FRED_LABEL, FRED_VOICE, [v2_row(S, cue, FRED_VOICE, n)]))
    first = outcome(lambda: assemble(S, [span(S, c, receipts[c]) for c in TEXT]))
    body = json.loads(receipts["01_2"].read_text())
    row = body["records"][0]
    oliver_request = request(S, "01_2", OLIVER_VOICE, "oliver-request01")
    row.update(voice=OLIVER_VOICE, request=str(oliver_request), request_sha256=sha(oliver_request))
    put(receipts["01_2"], v2(OLIVER_LABEL, OLIVER_VOICE, [row]))
    alone = outcome(lambda: coherent.unit_spans(UNIT["01_2"], SCRIPT_ROWS, span(S, "01_2", receipts["01_2"])) and None)
    return f"assemble={first.split(' ')[0]}; unit_spans_alone={alone.split(' ')[0]}; verify={outcome(lambda: verify(S))}"


# ---------------------------------------------------------------- one record checked through unit_spans
def fred_case(S, change):
    """A valid one-cue Frederick v2 take for 01_1, changed by `change`, then read through unit_spans."""
    row = v2_row(S, "01_1", FRED_VOICE, 1)
    ctx = types.SimpleNamespace(S=S, row=row, body=v2(FRED_LABEL, FRED_VOICE, [row]), extra={},
                                request=Path(row["request"]))
    change(ctx)
    receipt = put(S / "receipt.json", ctx.body)
    return outcome(lambda: coherent.unit_spans(UNIT["01_1"], SCRIPT_ROWS, span(S, "01_1", receipt, **ctx.extra)) and None)


def rewrite_request(ctx, drop=(), **fields):
    """Change the bound request file and re-record its hash, so only the content check can refuse it."""
    body = json.loads(ctx.request.read_text())
    body.update(fields)
    put(ctx.request, {k: v for k, v in body.items() if k not in drop})
    ctx.row["request_sha256"] = sha(ctx.request)


def replace_request(ctx, value, rehash):
    put(ctx.request, value)
    if rehash:
        ctx.row["request_sha256"] = sha(ctx.request)


def unversioned_lowercase(ctx):
    for key in ("schema", "provider", "voice"):
        ctx.body.pop(key)
    ctx.body["source"] = FRED_LABEL.lower()


CASES = {
    # the request file the record binds
    "request_voice_oliver": lambda c: rewrite_request(c, voice=OLIVER_VOICE),
    "request_voice_short": lambda c: rewrite_request(c, voice="Frederick Surrey"),
    "request_text_neighbour": lambda c: rewrite_request(c, text=TEXT["01_2"]),
    "request_text_one_word": lambda c: rewrite_request(c, text=TEXT["01_1"].replace("rises", "rose")),
    "request_cue_neighbour": lambda c: rewrite_request(c, cue="01_2"),
    "request_of_neighbour_file": lambda c: c.row.update(request=str(request(c.S, "01_2", FRED_VOICE)),
                                                       request_sha256=sha(c.S / "01_2-request01.json")),
    "request_bytes_changed": lambda c: replace_request(c, json.dumps(json.loads(c.request.read_text()), indent=2), rehash=False),
    "request_voice_missing": lambda c: rewrite_request(c, drop=("voice",)),
    "request_not_object": lambda c: replace_request(c, [TEXT["01_1"]], rehash=True),
    "request_bare_name": lambda c: c.row.update(request=c.request.name),
    # receipt identity
    "receipt_voice_short": lambda c: c.body.update(voice="Frederick Surrey"),
    "receipt_voice_oliver_on_fred_label": lambda c: (c.body.update(voice=OLIVER_VOICE), c.row.update(voice=OLIVER_VOICE))[1],
    "record_voice_missing": lambda c: c.row.pop("voice"),
    "record_voice_case": lambda c: c.row.update(voice=FRED_VOICE.lower()),
    "label_near_miss": lambda c: c.body.update(source="ElevenLabs UI, Frederick"),
    "label_trailing_space": lambda c: c.body.update(source=FRED_LABEL + " "),
    "label_lowercase_v2": lambda c: c.body.update(source=FRED_LABEL.lower()),
    "label_lowercase_unversioned": unversioned_lowercase,
    "label_as_list": lambda c: c.body.update(source=[FRED_LABEL]),
    "provider_wrong": lambda c: c.body.update(provider="Elevenlabs"),
    "provider_missing": lambda c: c.body.pop("provider"),
    "schema_unknown": lambda c: c.body.update(schema="elevenlabs-ui-receipt-v3"),
    "schema_unknown_on_oliver_label": lambda c: c.body.update(schema="elevenlabs-ui-receipt-v3", source=OLIVER_LABEL, voice=OLIVER_VOICE),
    "unversioned_frederick": lambda c: c.body.pop("schema"),
    "receipt_is_list": lambda c: setattr(c, "body", [c.body]),
    # record content
    "record_missing_request_sha256": lambda c: c.row.pop("request_sha256"),
    "record_missing_samples": lambda c: c.row.pop("samples"),
    "wav_sha256_wrong": lambda c: c.row.update(wav_sha256="0" * 64),
    "samples_off_by_one": lambda c: c.row.update(samples=FRAMES["01_1"] + 1),
    "rate_48000": lambda c: c.row.update(rate=48000),
    "source_sha256_wrong": lambda c: c.row.update(source_sha256="0" * 64),
    "source_changed": lambda c: (put(Path(c.row["source"]), "changed download"), None)[1],
    "wav_copy_at_other_path": lambda c: (Path(c.S / "copy.wav").write_bytes(Path(c.row["wav"]).read_bytes()),
                                         c.row.update(wav=str(c.S / "copy.wav")))[1],
    "record_text_neighbour": lambda c: (rewrite_request(c, text=TEXT["01_2"]), c.row.update(text=TEXT["01_2"]))[1],
    "record_duplicated": lambda c: c.body.update(records=[c.row, dict(c.row)]),
    "record_for_other_cue_only": lambda c: c.body.update(records=[v2_row(c.S, "01_2", FRED_VOICE, 2)]),
    "records_not_list": lambda c: c.body.update(records={"01_1": c.row}),
    "context_crop": lambda c: c.extra.update(context_after="01_2", target_end_sample=2000),
}
for _name, _change in CASES.items():
    _S = REPO / "s" / _name
    _S.mkdir(parents=True)
    results[_name] = fred_case(_S, _change)


# ---------------------------------------------------------------- receipts in prepare_audio.py's decode shape
def decode_receipt(S, label, voice, cue="01_1", seed=1, extra_row=None):
    """The shape logarithm-stage08-audio-preparation01/prepare_audio.py cmd_decode writes (lines 789-797)."""
    row = take(S, cue, seed)
    row = {"cue": cue, "text": row["text"], "source": row["source"], "source_sha256": row["source_sha256"],
           "source_pcm_sha256": "0" * 64, "source_frames": FRAMES[cue] * 2, "source_rate": 48000,
           "downloads_receipt": str(S / f"{cue}-downloads01.json"), "wav": row["wav"], "wav_sha256": row["wav_sha256"],
           "samples": row["samples"], "rate": 24000, **(extra_row or {})}
    return {"source": label, "voice": voice, "provider": "ElevenLabs", "production": True,
            "binding": {"path": "bindings/example.json", "sha256": "0" * 64, "use": "production",
                        "voice": {"provider": "ElevenLabs", "name": voice}},
            "tool": "prepare_audio.py decode", "conversion": "format only", "records": [row],
            "claim": "Format conversion only."}


def through_unit_spans(S, body, cue="01_1"):
    receipt = put(S / f"{cue}-decode-receipt.json", body)
    return outcome(lambda: coherent.unit_spans(UNIT[cue], SCRIPT_ROWS, span(S, cue, receipt)) and None)


@scenario
def decode_shape_oliver(S):
    return through_unit_spans(S, decode_receipt(S, OLIVER_LABEL, "Oliver"))


@scenario
def decode_shape_frederick(S):
    return through_unit_spans(S, decode_receipt(S, FRED_LABEL, "Frederick Surrey"))


@scenario
def decode_shape_frederick_plus_v2_fields(S):
    """The smallest change to that shape that the v2 branch accepts: schema, exact voice, per-record voice and request."""
    path = request(S, "01_1", FRED_VOICE)
    body = decode_receipt(S, FRED_LABEL, FRED_VOICE,
                          extra_row={"voice": FRED_VOICE, "request": str(path), "request_sha256": sha(path)})
    body["schema"] = V2
    return through_unit_spans(S, body)


@scenario
def decode_shape_misbound(S):
    """A Frederick binding whose receipt_source was left as the Oliver label: the receipt says voice Frederick."""
    return through_unit_spans(S, decode_receipt(S, OLIVER_LABEL, "Frederick Surrey"))


@scenario
def misbound_mixed_with_oliver(S):
    a = put(S / "a.json", decode_receipt(S, OLIVER_LABEL, "Frederick Surrey"))
    b = put(S / "b.json", legacy([take(S, "01_2", 2)]))
    return outcome(lambda: (assemble(S, [span(S, "01_1", a), span(S, "01_2", b)]), verify(S), pcm_check(S))[2])


@scenario
def v2_schema_stripped_under_oliver_label(S):
    row = v2_row(S, "01_1", FRED_VOICE, 1)
    body = v2(FRED_LABEL, FRED_VOICE, [row])
    body.pop("schema")
    body["source"] = OLIVER_LABEL
    return through_unit_spans(S, body)


# ---------------------------------------------------------------- the narration_delivery CLI (render consumer)
def deliver(S):
    """Assembly built from the canonical plan; external-units.json holds the same bytes, as root's external plans do."""
    (S / "out" / "external-units.json").write_bytes(UNITS.read_bytes())
    run = subprocess.run([sys.executable, "-I", "-B", str(REPO / "tools/narration_delivery.py"), "--topic", str(TREE),
                          "--narration", str(S / "out"), "--consumer", "render", "--chapter-key", "01_demo",
                          "--gain-db", "0"], cwd=REPO, capture_output=True, text=True, timeout=120)
    if run.returncode == 0:
        return f"exit 0: plan_mode={json.loads(run.stdout)['plan_mode']}"
    tail = (run.stderr.strip().splitlines() or ["(no stderr)"])[-1]
    return f"exit {run.returncode}: {tail}"


def assembled(S, records):
    done = outcome(lambda: assemble(S, records))
    return deliver(S) if done == "ok" else f"assembly {done}"


@scenario
def delivery_frederick(S):
    receipt = put(S / "receipt.json", v2(FRED_LABEL, FRED_VOICE, [v2_row(S, c, FRED_VOICE, n) for n, c in enumerate(TEXT, 1)]))
    return assembled(S, [span(S, c, receipt) for c in TEXT])


@scenario
def delivery_oliver_legacy(S):
    receipt = put(S / "receipt.json", legacy([take(S, c, n) for n, c in enumerate(TEXT, 1)]))
    return assembled(S, [span(S, c, receipt) for c in TEXT])


@scenario
def delivery_native_under_external_plan(S):
    records = [span(S, c, put(S / f"{c}-native.json", native(S, c, n))) for n, c in enumerate(TEXT, 1)]
    return assembled(S, records)


@scenario
def delivery_swapped_mix(S):
    receipts = {c: put(S / f"{c}-receipt.json", v2(FRED_LABEL, FRED_VOICE, [v2_row(S, c, FRED_VOICE, n)]))
                for n, c in enumerate(TEXT, 1)}
    done = outcome(lambda: assemble(S, [span(S, c, receipts[c]) for c in TEXT]))
    row = json.loads(receipts["01_2"].read_text())["records"][0]
    oliver_request = request(S, "01_2", OLIVER_VOICE, "oliver-request01")
    row.update(voice=OLIVER_VOICE, request=str(oliver_request), request_sha256=sha(oliver_request))
    put(receipts["01_2"], v2(OLIVER_LABEL, OLIVER_VOICE, [row]))
    return deliver(S) if done == "ok" else f"assembly {done}"


print(json.dumps(results, indent=1, sort_keys=True).replace(str(REPO), "REPO"))
