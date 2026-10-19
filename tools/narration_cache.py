#!/usr/bin/env python3
"""Content-addressed FTTS takes for x; leaves the installed planner/editor untouched.

Only verbatim FTTS normalization (NFC) is supported. The submitted text and NFC
text are both recorded. Old tags/WAVs remain historical; explicit old director
picks must be reconsidered, never silently retargeted to freshly generated audio.
Receipts require observed model EOS and matching generation provenance, not audible quality.
"""
import argparse
from concurrent.futures import ThreadPoolExecutor
import fcntl
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import re
import shutil
import subprocess
import sys
import time
import unicodedata
import uuid
import wave

SCHEMA = 2


def encoded(value):
    return json.dumps(value, sort_keys=True, ensure_ascii=False, separators=(",", ":")).encode()


def digest(path):
    hasher = hashlib.sha256()
    with Path(path).open("rb") as handle:
        while block := handle.read(1024 * 1024):
            hasher.update(block)
    return hasher.hexdigest()


def identity(path):
    path = Path(path).resolve(strict=True)
    before = path.stat()
    result = {"path": str(path), "sha256": digest(path), "bytes": before.st_size}
    after = path.stat()
    if (before.st_ino, before.st_size, before.st_mtime_ns, before.st_ctime_ns) != (
            after.st_ino, after.st_size, after.st_mtime_ns, after.st_ctime_ns):
        raise ValueError(f"input changed while hashing: {path}")
    return result


def write_new(path, value):
    """Exclusive creation, fsync; incomplete files never imply a valid cache hit."""
    with Path(path).open("xb") as handle:
        handle.write(encoded(value) + b"\n")
        handle.flush()
        os.fsync(handle.fileno())


def events(stdout):
    return [json.loads(line) for line in stdout.splitlines() if line.strip()]


def child_environment(environ):
    env = dict(environ)
    # Preserve the deployed batch's balanced-profile policy. Now explicit and keyed.
    env.pop("FTTS_PROFILE", None)
    # Derived ICL caches are not inputs to this independent, recorded generation.
    env["FTTS_VOICE_CACHE"] = "0"
    return env


def make_context(ftts, voice, model, planner, script, lexicon, env, options):
    executable = shutil.which(ftts)
    if not executable:
        raise ValueError(f"FTTS executable not found: {ftts}")
    executable = str(Path(executable).resolve(strict=True))
    command = [executable, "say", "--no-resident", "--robot", "--check", "--voice", voice,
               "--normalize", "verbatim", "--profile", options["profile"]]
    if model:
        command += ["--model", str(Path(model).resolve(strict=True))]
    # The executable, not a duplicated search algorithm, resolves the actual bundle.
    checked = subprocess.run(command + ["--", "Cache preflight."], env=env, text=True,
                             capture_output=True, timeout=30, check=True)
    preflight_events = events(checked.stdout)
    check = next(e for e in preflight_events if e.get("event") == "check_complete")
    normalization = next(e for e in preflight_events if e.get("event") == "text_prepared")
    if check["normalize"] != "verbatim":
        raise ValueError("producer did not honor verbatim normalization")
    resolved_model = Path(check["model"]).resolve(strict=True)
    root = resolved_model if resolved_model.is_dir() else resolved_model.parent
    required = ("vocab.json", "merges.txt", "tokenizer_config.json", "speech_tokenizer/model.safetensors")
    if any(not (root / name).is_file() for name in required):
        raise ValueError(f"not a complete FTTS bundle: {root}")
    # Conservatively hash the whole model bundle, including optional denoising and
    # enrollment weights/configs. Adding an asset also changes identity.
    bundle = [identity(p) for p in sorted(root.rglob("*")) if p.is_file()]
    supplied_voice = Path(voice)
    if supplied_voice.is_file():
        voice_id = {"kind": "file", "input": identity(supplied_voice)}
        voice_arg = str(supplied_voice.resolve(strict=True))
    else:
        # A built-in's materialized filename includes PID: hash bytes, not that name.
        materialized = identity(check["voice"])
        voice_id = {"kind": "builtin", "name": voice, "sha256": materialized["sha256"],
                    "bytes": materialized["bytes"]}
        voice_arg = voice
    # Script contents are guarded per batch below; per-take raw/submitted text
    # provides their cache identity. Editing one line must not regenerate a video.
    sources = [identity(planner), identity(__file__)]
    for candidate in (Path(script).parent / "alts.tsv", Path(script).parent / "reads.tsv", Path(lexicon)):
        sources.append(identity(candidate) if candidate.is_file() else {"path": str(candidate.resolve()), "absent": True})
    relevant = {k: v for k, v in env.items() if k.startswith(("FTTS_", "RAYON_", "OMP_", "MKL_", "OPENBLAS_", "VECLIB_", "LC_"))
                or k == "LANG"}
    # Values can contain private paths: receipts store digests rather than arbitrary env text.
    env_hashes = {k: hashlib.sha256(v.encode()).hexdigest() for k, v in relevant.items()}
    return {"schema": SCHEMA, "executable": identity(executable), "model": str(resolved_model),
            "bundle": bundle, "voice": voice_id, "voice_argument": voice_arg, "sources": sources,
            "script_path": str(Path(script).resolve()),
            "settings": {key: check[key] for key in ("profile", "packet_frames", "math_mode", "normalize")},
            "options": options, "environment_sha256": env_hashes,
            "platform": {"system": platform.system(), "release": platform.release(), "machine": platform.machine()},
            "normalization": {"policy": "NFC", "engine_unicode": normalization["unicode_version"],
                              "local_unicode": unicodedata.unidata_version,
                              "version": "planner source + FTTS executable SHA256"}}


def bind_plan(plan, context):
    jobs = []
    for line_id, sentences in plan.items():
        for sentence in sentences:
            for read in sentence["reads"]:
                original = read.get("legacy_tag", read["tag"])
                if not re.fullmatch(r"[A-Za-z0-9_.-]{1,140}", original):
                    raise ValueError(f"unsafe take tag: {original!r}")
                seed = read.get("seed", 0)
                if type(seed) is not int or not 0 <= seed < 2**64:
                    raise ValueError(f"invalid seed: {seed}")
                engine_unicode = context["normalization"]["engine_unicode"]
                if not read["tts"].isascii() and engine_unicode != unicodedata.unidata_version:
                    # UAX15 section 3 guarantees NFC stability for characters already
                    # assigned in the older Unicode version. Refuse newer/unknown
                    # characters rather than pretending our older tables know them.
                    # https://www.unicode.org/reports/tr15/#Versioning_and_Stability
                    if (not re.fullmatch(r"\d+\.\d+\.\d+", engine_unicode)
                            or tuple(map(int, engine_unicode.split("."))) < tuple(map(int, unicodedata.unidata_version.split(".")))
                            or any(unicodedata.category(c) == "Cn" for c in read["tts"])):
                        raise ValueError("Unicode versions cannot establish exact NFC for this input")
                request = {"context": context, "line_id": line_id, "sentence": sentence["j"],
                           "raw_text": read["text"], "submitted_text": read["tts"],
                           "normalized_text": unicodedata.normalize("NFC", read["tts"]), "seed": seed}
                key = hashlib.sha256(encoded(request)).hexdigest()
                tag = f"{original}__v{SCHEMA}_{key}"
                read.update(tag=tag, legacy_tag=original, cache_key=key)
                jobs.append({"tag": tag, "key": key, "request": request})
    return jobs


def check_picks(plan, picks):
    if not picks.is_file():
        return
    for number, row in enumerate(picks.read_text().splitlines(), 1):
        if not row.strip() or row.startswith("#"):
            continue
        cells = row.split("\t")
        if len(cells) != 4:
            raise ValueError(f"{picks}:{number}: malformed director pick")
        lid, index, selected, _ = cells
        reads = plan[lid][int(index)]["reads"]
        if not any(selected in (r["tag"], r["tts"]) for r in reads):
            raise ValueError(f"{picks}:{number}: stale/unqualified director tag {selected!r}; "
                             "review a current content-addressed take; old approval cannot follow new audio")


def wav_identity(path):
    with wave.open(str(path), "rb") as handle:
        frames, channels, width, rate = handle.getnframes(), handle.getnchannels(), handle.getsampwidth(), handle.getframerate()
        if not frames or (channels, width, rate) != (1, 2, 24000):
            raise ValueError("expected nonempty 24 kHz mono PCM16 FTTS WAV")
        size = 0
        while chunk := handle.readframes(65536):
            size += len(chunk)
        if size != frames * channels * width:
            raise ValueError("truncated WAV sample data")
    return {"sha256": digest(path), "bytes": path.stat().st_size, "frames": frames,
            "sample_rate": rate, "channels": channels, "sample_width": width}


def validate_generation(generation, request, completion):
    """Require the producer's effective evidence, not only a successful process exit."""
    context = request["context"]
    checks = [generation["schema"] == "ftts.generation", generation["schema_version"] == 1,
              generation["run_id"] == completion["run_id"], generation["state"] == "completed",
              generation["execution"] == "inline", generation["output"]["stop_reason"] == "end_of_speech",
              generation["requested"]["seed"] == request["seed"],
              generation["effective"]["seed"] == request["seed"],
              generation["effective"]["normalization"]["mode"] == context["settings"]["normalize"],
              generation["effective"]["utterance_start"] == "fresh",
              generation["effective"]["sampling"]["mode"] == "production",
              generation["producer"]["executable_sha256"] == context["executable"]["sha256"],
              generation["voice"]["route"] in ("xvector", "icl"),
              generation["output"]["samples"] == completion["samples"],
              generation["output"]["frames"] == completion["frames"],
              generation["output"]["sample_rate"] == 24000,
              0 < generation["output"]["frames"] < generation["output"]["frame_limit"]]
    voice = context["voice"]
    voice_hash = voice["input"]["sha256"] if voice["kind"] == "file" else voice["sha256"]
    checks.append(generation["assets"]["voice_source_sha256"] == voice_hash)
    bundle = {item["path"]: item["sha256"] for item in context["bundle"]}
    model = Path(context["model"])
    root = model if model.is_dir() else model.parent
    if model.is_dir():
        canonical = root / "qwen3-tts-12hz-0.6b-base.fttsq"
        model = canonical if str(canonical) in bundle else root / "model.safetensors"
    checks.append(generation["assets"]["talker_sha256"] == bundle[str(model)])
    checks.append(generation["assets"]["codec_sha256"] == bundle[str(root / "speech_tokenizer/model.safetensors")])
    for name in ("vocab.json", "merges.txt", "tokenizer_config.json"):
        checks.append(generation["assets"][name]["sha256"] == bundle[str(root / name)])
    if not all(checks):
        raise ValueError("generation provenance does not match the request or clean EOS completion")


def valid_hit(job, takes):
    try:
        receipt = json.loads((takes / f"{job['tag']}.receipt.json").read_text())
        validate_generation(receipt["generation"], job["request"], receipt["completion"])
        return (receipt["schema"] == SCHEMA and receipt["key"] == job["key"]
                and receipt["request"] == job["request"] and receipt["returncode"] == 0
                and receipt["completion"]["event"] == "run_complete"
                and receipt["completion"]["exit_code"] == 0
                and receipt["completion"]["audio_bytes"] == receipt["wav"]["frames"] * 2
                and receipt["wav"] == wav_identity(takes / f"{job['tag']}.wav"))
    except (OSError, ValueError, KeyError, TypeError, wave.Error, EOFError):
        return False


def stage(job, takes, env, timeout):
    if valid_hit(job, takes):
        return {"job": job, "state": "hit"}
    attempt = takes / "attempts" / uuid.uuid4().hex
    attempt.mkdir(parents=True)
    output = attempt / "audio.wav"
    request = job["request"]
    context = request["context"]
    settings = context["settings"]
    command = [context["executable"]["path"], "say", "--no-resident", "--robot", "--voice", context["voice_argument"],
               "--model", context["model"], "--seed", str(request["seed"]), "--trace", str(attempt / "trace")]
    for key, flag in (("profile", "--profile"), ("packet_frames", "--packet-frames"),
                      ("math_mode", "--math-mode"), ("normalize", "--normalize")):
        command += [flag, settings[key]]
    command += ["--", request["submitted_text"], str(output)]
    write_new(attempt / "request.json", {"key": job["key"], "request": request, "command": command})
    started = time.monotonic()
    try:
        with (attempt / "stdout.ndjson").open("xb") as stdout, (attempt / "stderr.txt").open("xb") as stderr:
            result = subprocess.run(command, env=env, stdout=stdout, stderr=stderr, timeout=timeout, check=False)
        emitted = events((attempt / "stdout.ndjson").read_text())
        completions = [e for e in emitted if e.get("event") == "run_complete"]
        if result.returncode != 0 or any(e.get("event") == "run_error" for e in emitted) or len(completions) != 1:
            raise ValueError(f"producer did not complete successfully (exit {result.returncode})")
        completed = completions[0]
        if completed.get("exit_code") != 0 or completed.get("audio_bytes", 0) <= 0:
            raise ValueError("producer has no successful audio completion")
        traces = list((attempt / "trace").glob("*.generation.json"))
        if len(traces) != 1:
            raise ValueError("producer must emit exactly one generation receipt; use the N10-qualified FTTS build")
        generation = json.loads(traces[0].read_text())
        validate_generation(generation, request, completed)
        wav = wav_identity(output)
        if completed["audio_bytes"] != wav["frames"] * wav["channels"] * wav["sample_width"]:
            raise ValueError("completion bytes disagree with WAV")
        receipt = {"schema": SCHEMA, "key": job["key"], "request": request, "returncode": 0,
                   "wav": wav, "completion": completed, "generation": generation,
                   "command": command, "elapsed_s": time.monotonic() - started,
                   "claim": "observed model EOS and matching process/assets/bytes; text accuracy and voice quality unqualified"}
        write_new(attempt / "receipt.json", receipt)
        return {"job": job, "state": "generated", "attempt": attempt}
    except (OSError, ValueError, KeyError, TypeError, subprocess.SubprocessError, wave.Error, EOFError) as error:
        write_new(attempt / "failure.json", {"error": str(error), "elapsed_s": time.monotonic() - started})
        return {"job": job, "state": "failed", "attempt": attempt, "error": str(error)}


def publish(result, takes):
    job, attempt = result["job"], result["attempt"]
    # Preserve unqualified/corrupt files with unique IDs; never overwrite legacy audio.
    for suffix in ("wav", "receipt.json"):
        target = takes / f"{job['tag']}.{suffix}"
        if target.exists() or target.is_symlink():
            target.rename(attempt / f"previous.{suffix}")
    # Exclusive hard links: audio first, receipt last. A crash between them is a miss.
    os.link(attempt / "audio.wav", takes / f"{job['tag']}.wav")
    os.link(attempt / "receipt.json", takes / f"{job['tag']}.receipt.json")


def run_batch(jobs, takes, env, workers, timeout, verify_inputs):
    takes.mkdir(parents=True, exist_ok=True)
    with (takes / ".cache.lock").open("a+b") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        with ThreadPoolExecutor(max_workers=workers) as pool:
            results = list(pool.map(lambda job: stage(job, takes, env, timeout), jobs))
        # Full rehash, including newly added model files, before accepting hits OR
        # publishing any staged attempt. No mtime-only shortcut.
        verify_inputs()
        for result in results:
            if result["state"] == "generated":
                publish(result, takes)
            print(json.dumps({k: str(v) if isinstance(v, Path) else v for k, v in result.items() if k != "job"}), flush=True)
        return results


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--planner", type=Path, required=True)
    parser.add_argument("--script", type=Path, required=True)
    parser.add_argument("--work", type=Path, required=True)
    parser.add_argument("--lexicon", type=Path, required=True)
    parser.add_argument("--voice", required=True)
    parser.add_argument("--model", default=os.environ.get("FTTS_MODEL", ""))
    parser.add_argument("--ftts", default=os.environ.get("FTTS", "ftts"))
    parser.add_argument("--seeds", type=int, default=0)
    parser.add_argument("--workers", type=int, default=8)
    parser.add_argument("--timeout", type=float, default=300)
    args = parser.parse_args()
    if args.workers < 1 or args.seeds < 0 or not math.isfinite(args.timeout) or args.timeout <= 0:
        parser.error("workers/timeout must be positive and seeds nonnegative")
    args.work.mkdir(parents=True, exist_ok=True)
    env = child_environment(os.environ)
    options = {"profile": "balanced", "route": "in-process", "normalization": "verbatim", "voice_cache": False}
    def capture():
        return make_context(args.ftts, args.voice, args.model, args.planner, args.script, args.lexicon, env, options)
    # Serializes direct callers too; x also holds its checkout producer reservation.
    with (args.work / ".plan.lock").open("a+b") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        context = capture()
        script_input = identity(args.script)
        subprocess.run([sys.executable, "-I", str(args.planner.resolve()), "plan", str(args.script.resolve()), str(args.work.resolve()),
                        "--voice", args.voice, "--model", args.model, "--lexicon", str(args.lexicon.resolve()),
                        "--seeds", str(args.seeds)], cwd=args.script.resolve().parent, env=env, check=True, timeout=120)
        plan = json.loads((args.work / "plan.json").read_text())
        jobs = bind_plan(plan, context)
        # Keep the qualified plan separately before replacing the transient planner
        # view. Takes and their old names are never discarded.
        snapshot = args.work / ("qualified-plan-" + uuid.uuid4().hex + ".json")
        write_new(snapshot, plan)
        write_new(snapshot.with_suffix(".inputs.json"), {"script": script_input, "context": context})
        (args.work / "plan.json").write_bytes(snapshot.read_bytes())
        (args.work / "takes.tsv").write_text("".join(f"{j['tag']}\t{j['request']['seed']}\t{j['request']['submitted_text']}\n" for j in jobs))
        check_picks(plan, Path(env.get("NARRATION_PICKS", args.script.parent / "picks.tsv")))
        def verify():
            if capture() != context or identity(args.script) != script_input:
                raise ValueError("generation inputs changed during batch; staged outputs retained but not qualified")
        results = run_batch(jobs, args.work / "takes", env, args.workers, args.timeout, verify)
        return 1 if any(r["state"] == "failed" for r in results) else 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, ValueError, KeyError, StopIteration, subprocess.SubprocessError) as error:
        print(f"narration-cache: {error}", file=sys.stderr)
        raise SystemExit(1) from error
