#!/usr/bin/env python3
"""Checks for prepare_stage09_render.py. Writes only inside SCRATCH (new, outside the repository).

    python3 -I -B checks/run_checks.py SCRATCH --w W        exit 0 when every check passes

Real inputs are read, never written: root's W stage and build01 receipt, the EL stage09 candidate, the
canonical narration tools and the audio preparation bundle. The real cargo, guard, binary and kit never run.

- Guarded jobs run under an emulated guard in SCRATCH: it binds the inputs file before and after the job as
  root's guard does, without admission, lane lock or resource sampling. Builds use a stub `cargo` first on PATH;
  narrated steps use a stand-in binary (a Python script) that writes timeline, score, chapter and master files.
- Narration is a synthetic fixture. 80 noise takes in the audio preparation's decode-receipt and span shapes are
  assembled by the workspace's byte copy of the canonical assembler, with the bound plan's 15 proposed holds.
  This is LavenderPike's K10 recipe, rewritten here. The fixture is then wrapped in production-shaped receipts
  labelled TEST FIXTURE: a consistent forgery, so only the check under test can refuse.
- The coherent verifier and the delivery contract run for real, on the workspace copies, against that fixture.

Printed paths are shortened to SCRATCH, W, REPO and ~.
"""
import argparse
import contextlib
import hashlib
import importlib.util
import json
import os
import random
import shutil
import subprocess
import sys
import wave
from pathlib import Path

HERE = Path(__file__).resolve()
HELPER = HERE.parents[1] / "prepare_stage09_render.py"
REPO = HERE.parents[6]
spec = importlib.util.spec_from_file_location("prepare_stage09_render", HELPER)
H = importlib.util.module_from_spec(spec)
spec.loader.exec_module(H)
N = H.N

parser = argparse.ArgumentParser()
parser.add_argument("scratch")
parser.add_argument("--w", required=True)
ARGS = parser.parse_args()
SCR, W = Path(ARGS.scratch), Path(ARGS.w)
if not SCR.is_absolute() or (SCR.exists() and any(SCR.iterdir())) or SCR.resolve().is_relative_to(REPO.resolve()):
    sys.exit("SCRATCH must be a new or empty absolute folder outside the repository")
SCR.mkdir(parents=True, exist_ok=True)
WS = SCR / "ws"
LANE = SCR / "lane"
TARGET = SCR / "target"
STUB = SCR / "stub"
REAL_LANE = REPO / N.RECEIPTS_REL
S9 = REPO / N.EL_REL / "combined-repair-stage09"
RESULTS = []


def clean(text):
    text = str(text)
    for path, name in ((SCR.resolve(), "SCRATCH"), (SCR, "SCRATCH"), (W, "W"), (REPO, "REPO"), (Path.home(), "~")):
        text = text.replace(str(path), name)
    return text


def tail(text, n):
    """The end of a long output, shortened after the paths are, so no partial path is printed."""
    return clean(str(text).strip())[-n:]


def record(name, ok, detail=""):
    RESULTS.append(bool(ok))
    print(f"{'PASS' if ok else 'FAIL'} {name}{': ' + clean(detail) if detail else ''}", flush=True)


def helper(*args, scratch=True):
    extra = ["--receipts", str(LANE), "--target-root", str(TARGET)] if scratch else []
    run = subprocess.run([sys.executable, "-I", "-B", str(HELPER), *map(str, args), *extra], cwd=REPO,
                         capture_output=True, text=True, timeout=900)
    return run.returncode, run.stdout + run.stderr


def result(out):
    try:
        return json.loads(out)
    except json.JSONDecodeError:
        return {}


def ctx(ws=WS, lane=LANE, target=TARGET):
    return H.Ctx(argparse.Namespace(w=str(W), ws=ws and str(ws), receipts=lane and str(lane), target_root=target and str(target)))


def expect_refusal(name, call, fragment):
    try:
        call()
    except H.Refused as refusal:
        record(name, fragment in str(refusal), f"refused: {clean(refusal)[:300]}")
        return
    record(name, False, "not refused")


@contextlib.contextmanager
def edited(path, change):
    """Mutate one file in place for one check, then put its exact bytes back."""
    path = Path(path)
    saved = path.read_bytes()
    try:
        change(path)
        yield
    finally:
        path.write_bytes(saved)


def edit_json(path, change):
    def apply(p):
        data = json.loads(p.read_text())
        change(data)
        p.write_text(json.dumps(data, indent=2) + "\n")
    return edited(path, apply)


def snapshot():
    """Every real input this run must leave alone."""
    snap = {f"W/{STAGE}/{k}": v for k, v in N.tree(W / H.STAGE).items()}
    snap["W stage09 names"] = sorted(p.name for p in W.iterdir() if "stage09" in p.name)
    snap["lane names"] = sorted(p.name for p in REAL_LANE.iterdir() if p.name.startswith(H.STAGE))
    snap["S9 names"] = sorted(p.name for p in S9.iterdir())
    snap.update({f"tools/{n}": N.sha(REPO / "tools" / n) for n in H.TOOLS})
    binary = REPO / "target/geometry-kit01/debug/logarithm_mysteries"
    snap["shared binary"] = N.sha(binary) if binary.is_file() else None
    snap["audio bundle"] = N.sha(REPO / H.AUDIO_REL / "SHA256SUMS")
    return snap


STAGE = H.STAGE


# ------------------------------------------------------------------ stand-ins

STUB_CARGO = """#!/bin/sh
printf '%s\\n' "$*" >> "$STUB_LOG"
case "$1" in
  test) echo "test result: ok. ${STUB_TESTS:-241} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s" ;;
  build) mkdir -p "$CARGO_TARGET_DIR/debug" && cp "$STUB_BINARY" "$CARGO_TARGET_DIR/debug/logarithm_mysteries" &&
         chmod 755 "$CARGO_TARGET_DIR/debug/logarithm_mysteries" ;;
  clippy) : ;;
  *) echo "stub cargo: unexpected $1" >&2; exit 2 ;;
esac
"""

STAND_IN = r'''#!/usr/bin/env python3
# Stand-in for the workspace stage09 binary in the render-preparation checks. It names its topic
# @@TOPIC@@ as the real binary embeds CARGO_MANIFEST_DIR.
import json, os, pathlib, sys
CUES = json.loads(@@CUES@@)
CHAPTERS = list(CUES)
argv = sys.argv[1:]
with open(os.environ["STUB_LOG"], "a") as log:
    log.write("binary " + " ".join(argv) + "\n")
def opt(name):
    return argv[argv.index(name) + 1]
if argv == ["chapters"]:
    print("\n".join(CHAPTERS))
    sys.exit(0)
out = pathlib.Path(opt("--out"))
out.mkdir(parents=True, exist_ok=True)
if argv[0] == "render":
    fps, narration, final = int(opt("--fps")), opt("--narration"), "--timeline-only" not in argv
    for chapter in (CHAPTERS if argv[1] == "all" else [argv[1]]):
        k, cues = CHAPTERS.index(chapter), list(CUES[chapter])
        if os.environ.get("STUB_SWAP") and k == 0:
            cues[0], cues[1] = cues[1], cues[0]
        frames = (10 + 2 * k) * fps + (int(os.environ.get("STUB_SHIFT", "0")) if final else 0)
        rows = ["# explainer timeline v1", "chapter\t" + chapter, "fps\t%d" % fps, "frames\t%d" % frames]
        rows += ["line\t%s\t%d\t0\t%s/%s.wav" % (c, j * fps // 2, narration, c) for j, c in enumerate(cues)]
        (out / (chapter + ".timeline.tsv")).write_text("\n".join(rows) + "\n")
        (out / (chapter + ".cuelog.tsv")).write_text("stand-in\n")
        (out / (chapter + ".cues.tsv")).write_text("\n".join(cues) + "\n")
        if final:
            (out / (chapter + ".mp4")).write_bytes(b"stand-in mp4 " + chapter.encode())
            (out / (chapter + ".layout.tsv")).write_text("")
elif argv[0] == "score":
    qa = out / "stems" / "qa"
    qa.mkdir(parents=True)
    for chapter in CHAPTERS:
        (qa / (chapter + ".json")).write_text(json.dumps({"chapter": chapter, "passed": not os.environ.get("STUB_QA_FAIL"), "metrics": {}}))
elif argv[0] == "master":
    (out / "logarithm_mysteries_master.mp4").write_bytes(b"stand-in master")
elif argv[0] == "captions":
    (out / "logarithm_mysteries.en.srt").write_text("1\n00:00:00,000 --> 00:00:01,000\nstand-in\n")
    (out / "logarithm_mysteries.en.vtt").write_text("WEBVTT\n\n00:00:00.000 --> 00:00:01.000\nstand-in\n")
else:
    sys.exit(2)
'''


def stand_in(name, topic_text):
    cues = {}
    for cue in H.script_cues(WS / H.CRATE / "script.tsv"):
        chapter = next(ch for ch in H.CHAPTERS if ch.split("_", 1)[0] == cue.split("_", 1)[0])
        cues.setdefault(chapter, []).append(cue)
    path = STUB / name
    path.write_text(STAND_IN.replace("@@TOPIC@@", topic_text).replace("@@CUES@@", repr(json.dumps(cues))))
    path.chmod(0o755)
    return path


def stub_env(**extra):
    env = dict(os.environ, PATH=f"{STUB / 'bin'}:{os.environ['PATH']}", STUB_LOG=str(STUB / "calls.log"),
               CARGO_TARGET_DIR=str(TARGET))
    env.update({k: v for k, v in extra.items() if v is not None})
    for key in [k for k, v in extra.items() if v is None]:
        env.pop(key, None)
    if shutil.which("cargo", path=env["PATH"]) != str(STUB / "bin/cargo"):
        sys.exit("the stub cargo is not first on PATH; refusing to run a job script")
    return env


def calls():
    log = STUB / "calls.log"
    return log.read_text().splitlines() if log.exists() else []


def emulated_guard(job, inputs, command, **extra):
    """Root's guard as far as the inputs go: rows checked before and after, receipt and log in LANE/job."""
    folder = LANE / job
    folder.mkdir(parents=True)
    rows = json.loads(Path(inputs).read_text())["files"]

    def bound():
        moved = [r["path"] for r in rows if N.sha(r["path"]) != r["sha256"]]
        return moved, N.sha(inputs)
    receipt = {"started": "emulated", "phase": "native", "command": command, "preflight_only": False, "samples": [],
               "admitted": True, "emulated_guard": "checks only: no admission, lane lock or resource sampling"}
    moved, receipt["inputs_sha256"] = bound()
    assert not moved, ("inputs moved before launch", moved[:3])
    save = lambda: (folder / "resource-receipt.json").write_text(json.dumps(receipt, indent=2) + "\n")
    save()
    with (folder / "command.log").open("x") as log:
        run = subprocess.run(command, cwd=REPO, env=stub_env(**extra), stdout=log, stderr=subprocess.STDOUT, timeout=900)
    receipt["exit"] = run.returncode
    moved, receipt["final_inputs_sha256"] = bound()
    if moved:
        receipt["error"] = f"ValueError('reviewed input changed: {moved[0]}')"
    receipt["finished"] = "emulated"
    save()
    return run.returncode, (folder / "command.log").read_text()


def launch(written, **extra):
    """Run a written step under the emulated guard; returns (exit, log)."""
    data = result(written)
    return emulated_guard(data["job"], Path(data["launch_from_repo_root"].split(" --inputs ")[1].split(" -- ")[0]),
                          ["python3", "-I", "-B", data["script"]], **extra)


# ------------------------------------------------------------------ synthetic narration

def write_wav(path, pcm):
    with wave.open(str(path), "wb") as w:
        w.setnchannels(1)
        w.setsampwidth(2)
        w.setframerate(24000)
        w.writeframes(pcm)


def fixture(o):
    """A production-shaped audio preparation over 80 synthetic takes (TEST FIXTURE)."""
    asm = H_load_assembler()
    crate = WS / H.CRATE
    script = N.script_rows((crate / "script.tsv").read_text(encoding="utf-8"))
    units_path = crate / "narration_units.json"
    units = json.loads(units_path.read_text())["units"]
    binding = json.loads((REPO / H.AUDIO_REL / "bindings/logarithm-stage09-oliver.json").read_text())
    summary = {"path": f"{H.AUDIO_REL}/bindings/logarithm-stage09-oliver.json", "sha256": H.AUDIO_BINDING_SHA,
               "use": "production", "voice": H.VOICE}
    rng = random.Random(20261105)
    decoded = o / "decoded"
    decoded.mkdir(parents=True)
    source = o / "source-stand-in.bin"
    source.write_bytes(b"synthetic source stand-in, not audio")
    records, span_units = [], []
    for unit in units:
        cue = unit["line_ids"][0]
        n = len(asm.sentences(script[cue]))
        length = 24000 + 7919 * n + rng.randrange(1, 997)
        wav = decoded / f"{cue}.wav"
        write_wav(wav, rng.randbytes(2 * length))
        records.append({"cue": cue, "text": script[cue], "source": str(source), "source_sha256": N.sha(source),
                        "source_pcm_sha256": None, "source_frames": None, "source_rate": None, "downloads_receipt": None,
                        "wav": str(wav), "wav_sha256": N.sha(wav), "samples": length, "rate": 24000})
        span_units.append({"id": unit["id"], "wav": str(wav), "sha256": N.sha(wav), "generation_receipt": str(o / "decode-receipt.json"),
                           "sentence_ends": [length * (k + 1) // n for k in range(n)], "gap_after_samples": [0] * n})
    decode = {"source": H.RECEIPT_SOURCE, "voice": "Oliver", "provider": "ElevenLabs", "production": True, "binding": summary,
              "tool": "TEST FIXTURE", "selection_sha256": None, "chapter": None, "records": records,
              "claim": "TEST FIXTURE: synthetic noise for the render-preparation checks; not audio, not production."}
    (o / "decode-receipt.json").write_text(json.dumps(decode, indent=1))
    dispositions = json.loads((REPO / binding["dispositions"]).read_text())["direction_dispositions"]
    spans = {"schema_version": 1, "units_sha256": N.sha(units_path), "units": span_units, "direction_dispositions": dispositions,
             "boundary_review": {"reviewer": "RubyRaven TEST", "basis": "TEST FIXTURE: synthetic noise; whole takes, no samples "
                                 "removed, no silence inserted", "listening": "unreviewed"}}
    (o / "spans.json").write_text(json.dumps(spans, indent=1))
    (o / "workspace/tools").mkdir(parents=True)
    for name in ("coherent_narration.py", "narration_cache.py"):
        shutil.copy2(WS / "tools" / name, o / "workspace/tools" / name)
    run = subprocess.run([sys.executable, "-I", "-B", str(WS / "tools/coherent_narration.py"), "assemble", "--script",
                          str(crate / "script.tsv"), "--units", str(units_path), "--require-directions", "--spans",
                          str(o / "spans.json"), "--out", str(o / "narration")], cwd=REPO, capture_output=True, text=True)
    assert run.returncode == 0, run.stderr[-800:]
    prepare = {"tool": "TEST FIXTURE prepare", "binding": summary, "production": True, "pins": {}, "chapter": None,
               "decode_receipt": {"path": str(o / "decode-receipt.json"), "sha256": N.sha(o / "decode-receipt.json")},
               "spans_sha256": N.sha(o / "spans.json"), "cues": {cue: {"recording": "synthetic"} for cue in script},
               "claim": "TEST FIXTURE: production-shaped, not production."}
    (o / "prepare-receipt.json").write_text(json.dumps(prepare, indent=1))
    t = Path(f"{o}-timings")
    t.mkdir()
    timings = {"basis": "TEST FIXTURE", "voice": "Oliver", "production": True, "binding_sha256": H.AUDIO_BINDING_SHA,
               "script_sha256": N.PIN["script"], "prepare_receipt_sha256": N.sha(o / "prepare-receipt.json"),
               "cues": {cue: {"wav_sha256": N.sha(o / "narration" / f"{cue}.wav"),
                              "text_sha256": hashlib.sha256(script[cue].encode()).hexdigest(), "marks": []} for cue in script}}
    (t / "phrase-timings.json").write_text(json.dumps(timings, indent=1))
    (t / "timings-receipt.json").write_text(json.dumps({
        "tool": "TEST FIXTURE timings", "binding": summary, "production": True, "cues": len(script), "marks": 0,
        "file": "phrase-timings.json", "assembly_sha256": N.sha(o / "narration/assembly.json"),
        "report_sha256": N.sha(o / "narration/report.json"),
        "install": f"cp -n {t}/phrase-timings.json {o}/narration/phrase-timings.json", "claim": "TEST FIXTURE"}, indent=1))
    shutil.copy2(t / "phrase-timings.json", o / "narration/phrase-timings.json")
    return o, t


def H_load_assembler():
    spec2 = importlib.util.spec_from_file_location("ws_coherent", WS / "tools/coherent_narration.py")
    module = importlib.util.module_from_spec(spec2)
    spec2.loader.exec_module(module)
    return module


# ------------------------------------------------------------------ main

def main():
    print(f"helper sha256 {N.sha(HELPER)}; checks sha256 {N.sha(HERE)}")
    before = snapshot()
    (STUB / "bin").mkdir(parents=True)
    (STUB / "bin/cargo").write_text(STUB_CARGO)
    (STUB / "bin/cargo").chmod(0o755)
    LANE.mkdir()

    # --- R: the real inputs, read only
    code, out = helper("check", "--w", W, scratch=False)
    facts = result(out)
    blocker = facts.get("blocker", {})
    record("R1 check passes on root's real build01 and names the blocker",
           code == 0 and facts.get("build01", {}).get("stage09_binary") == H.BUILD01_BINARY
           and blocker.get("compiled_topic_mentions_in_build01", 0) > 0 and blocker.get("kit_tool_present") is False
           and blocker.get("topic_parent_is_explainer_corpus") is False,
           f"build01 {facts.get('build01', {}).get('stage09_binary', '')[:12]}, preserved "
           f"{facts.get('build01', {}).get('preserved_binary', '')[:12]}; compiled topic named "
           f"{blocker.get('compiled_topic_mentions_in_build01')} times; kit tool {blocker.get('kit_runs')} present "
           f"{blocker.get('kit_tool_present')}" if code == 0 else tail(out, 300))

    # --- I: workspace install
    code, out = helper("install", "--w", W, "--ws", WS, scratch=False)
    stage = N.tree(W / H.STAGE)
    record("I1 install copies root's 48 stage09 bytes and the 3 canonical tools, nothing else",
           code == 0 and N.tree(WS / H.CRATE) == stage and N.tree(WS / "tools") == H.TOOLS
           and sorted(p.name for p in WS.iterdir()) == ["explainer_corpus", "tools", "workspace-binding01.json"],
           f"{len(stage)} crate files equal W/{H.STAGE} byte for byte; tools {sorted(H.TOOLS)}" if code == 0 else tail(out, 300))
    code, out = helper("install", "--w", W, "--ws", WS, scratch=False)
    record("I2 install refuses an existing workspace", code == 1 and "refusing to overwrite" in out, tail(out, 160))
    inside = REPO / "never-created-render-ws"
    code, out = helper("install", "--w", W, "--ws", inside, scratch=False)
    record("I3 install refuses a workspace inside the repository", code == 1 and "inside the repository" in out
           and not inside.exists(), tail(out, 160))
    in_stage = W / H.STAGE / "never-created-ws"
    code, out = helper("install", "--w", W, "--ws", in_stage, scratch=False)
    record("I4 install refuses a workspace inside W/logarithm-stage09", code == 1 and "--ws is inside" in out
           and not in_stage.exists(), tail(out, 160))
    code, out = helper("install", "--w", W, "--ws", "relative-ws", scratch=False)
    record("I5 install refuses a relative workspace", code == 1 and "must be absolute" in out, tail(out, 120))
    for tag, mutate, fragment in (
            ("a changed crate byte", lambda ws: (ws / H.CRATE / "script.tsv").write_bytes((ws / H.CRATE / "script.tsv").read_bytes() + b"\n"),
             "workspace crate differs"),
            ("an extra tool", lambda ws: (ws / "tools/extra.py").write_text("pass\n"), "workspace tools differ"),
            ("a second topic", lambda ws: (ws / "explainer_corpus/other_topic").mkdir(), "another entry")):
        ws = SCR / f"ws-{tag.split(' ', 1)[1].replace(' ', '-')}"
        assert helper("install", "--w", W, "--ws", ws, scratch=False)[0] == 0
        mutate(ws)
        expect_refusal(f"I6 the workspace check refuses {tag}", lambda: H.verify_ws(ctx(ws=ws)), fragment)

    # --- B: the guarded rebuild
    code, out = helper("build-script", "--w", W, "--ws", WS, "--attempt", "01", scratch=False)
    built = result(out)
    script01 = WS / f"{STAGE}-ws-build01.sh"
    rows = json.loads((WS / f"{STAGE}-ws-build01-inputs.json").read_text())["files"] if code == 0 else []
    text = script01.read_text() if script01.exists() else ""
    cargo = [line for line in text.splitlines() if line.startswith("cargo ")]
    record("B1 build-script writes the guarded rebuild: build01's three cargo lines on the workspace manifest",
           code == 0 and built.get("preserves") == N.sha(REPO / "target/geometry-kit01/debug/logarithm_mysteries")
           and rows[0]["path"] == str(script01) and len(cargo) == 3
           and all('--config "$W/engine-budget.toml" --manifest-path "$M"' in line for line in cargo)
           and f'M="$WS/{H.CRATE}/Cargo.toml"' in text,
           f"{built.get('input_rows')} input rows, preserves {str(built.get('preserves'))[:12]} "
           f"(build01's binary: {built.get('preserves_build01_binary')}); {built.get('launch_from_repo_root')}" if code == 0 else tail(out, 300))
    old = (W / f"{STAGE}-build01.sh").read_text()
    record("B2 the cargo lines equal root's build01 script's, line for line",
           [line for line in old.splitlines() if line.startswith("cargo ")] == cargo)
    run = subprocess.run(["sh", str(script01)], cwd=REPO, env=stub_env(CARGO_TARGET_DIR=None), capture_output=True, text=True)
    record("B3 the job refuses outside the guard, before any cargo", run.returncode == 1 and "CARGO_TARGET_DIR is not" in run.stderr
           and not calls(), run.stderr.strip())

    (TARGET / "geometry-kit01/debug").mkdir(parents=True)
    (TARGET / "geometry-kit01/debug/logarithm_mysteries").write_bytes(b"#!/bin/sh\n# an older shared binary\n")
    good = stand_in("stand-in-binary", str(WS / H.CRATE))
    code, out = helper("build-script", "--w", W, "--ws", WS, "--attempt", "02")
    stub02 = result(out)
    exit02, log02 = launch_build("02", good)
    record("B4 under the emulated guard with stub cargo the job tests, builds, copies out, lints and lists chapters",
           code == 0 and exit02 == 0 and [c.split(" --manifest-path ")[0] for c in calls() if not c.startswith("binary")]
           == ["test --locked --offline --config " + str(W / "engine-budget.toml"),
               "build --locked --offline --config " + str(W / "engine-budget.toml"),
               "clippy --locked --offline --config " + str(W / "engine-budget.toml")]
           and N.sha(WS / f"{STAGE}-ws-build02-binary") == N.sha(good)
           and N.sha(WS / f"before-{STAGE}-ws-build02-binary") == stub02.get("preserves"),
           f"exit {exit02}; " + clean(log02.strip().splitlines()[-1] if log02.strip() else ""))
    code, out = helper("build-verify", "--w", W, "--ws", WS, "--attempt", "02")
    record("B5 build-verify passes the emulated run", code == 0 and result(out).get("workspace_topic_mentions", 0) > 0, tail(out, 300) if code else
           f"binary {result(out).get('binary_sha256', '')[:12]}, workspace topic named {result(out).get('workspace_topic_mentions')} times")
    receipt = LANE / f"{STAGE}-ws-build02/resource-receipt.json"
    log = LANE / f"{STAGE}-ws-build02/command.log"
    c2 = ctx()
    with edit_json(receipt, lambda r: r.update(exit=1)):
        expect_refusal("B6 build-verify refuses a failed job", lambda: H.build_verify(c2, "02"), "did not pass")
    with edited(log, lambda p: p.write_text(p.read_text().replace("ok. 241 passed", "ok. 240 passed"))):
        expect_refusal("B7 build-verify refuses another test count", lambda: H.build_verify(c2, "02"), "test results")
    with edited(WS / f"{STAGE}-ws-build02-inputs.json", lambda p: p.write_text(p.read_text() + " ")):
        expect_refusal("B8 build-verify refuses an inputs file changed after launch", lambda: H.build_verify(c2, "02"), "input hash")
    for nn, name, topic_text, fragment in (("03", "no-topic", "(no path here)", "does not name the workspace topic"),
                                           ("04", "old-topic", f"{WS / H.CRATE} and {W / H.STAGE}/src/main.rs",
                                            "still names W/logarithm-stage09")):
        binary = stand_in(f"stand-in-{name}", topic_text)
        assert helper("build-script", "--w", W, "--ws", WS, "--attempt", nn)[0] == 0
        assert launch_build(nn, binary)[0] == 0
        expect_refusal(f"B9 build-verify refuses a binary with {name.replace('-', ' ')}", lambda: H.build_verify(c2, nn), fragment)

    # --- D: binding the narration
    real_o = S9 / "audio-preparation01"
    code, out = helper("bind", "--w", W, "--ws", WS, "--build", "02", "--audio", real_o, "--attempt", "01")
    record("D1 bind refuses the real audio folder today: no production receipt exists yet",
           code == 1 and "the real audio preparation has not run" in out and not real_o.exists(), tail(out, 200))
    o, t = fixture(SCR / "audio/prepared")
    n = o / "narration"
    code, out = helper("bind", "--w", W, "--ws", WS, "--build", "02", "--audio", o, "--attempt", "01")
    bound = result(out)
    record("D2 bind passes a production-shaped fixture: coherent verify and the kit's delivery contract on the workspace topic",
           code == 0 and bound.get("bound_files", 0) > 160, (f"{bound.get('bound_files')} files bound, plan "
           f"{bound.get('plan_mode')}, binary {bound.get('binary_sha256', '')[:12]}") if code == 0 else tail(out, 500))
    contract = ["--narration", str(n), "--consumer", "render", "--gain-db", "0"] + [x for k in H.CHAPTERS for x in ("--chapter-key", k)]
    for label, tool in (("the canonical tool", REPO / "tools/narration_delivery.py"), ("the workspace tool", WS / "tools/narration_delivery.py")):
        run = subprocess.run(["python3", "-I", "-B", str(tool), "--topic", str(W / H.STAGE), *contract], cwd=REPO,
                             capture_output=True, text=True)
        record(f"D3 the blocker, run: {label} refuses build01's topic W/logarithm-stage09 on the same narration",
               run.returncode == 1 and "topic must be in this repository" in run.stderr, run.stderr.strip())
    c3 = ctx()
    for name, path, change, fragment in (
            ("a test-only preparation", o / "prepare-receipt.json", lambda d: d.update(production=False), "whole-film production"),
            ("another voice", o / "prepare-receipt.json", lambda d: d["binding"]["voice"].update(name="Frederick"), "prepare receipt binding"),
            ("another audio binding", o / "prepare-receipt.json", lambda d: d["binding"].update(sha256="0" * 64), "prepare receipt binding"),
            ("a one-chapter preparation", o / "prepare-receipt.json", lambda d: d.update(chapter="08"), "whole-film production"),
            ("79 cues", o / "prepare-receipt.json", lambda d: d["cues"].popitem(), "covers 79 cues"),
            ("a timings receipt for another assembly", t / "timings-receipt.json", lambda d: d.update(assembly_sha256="0" * 64),
             "names another assembly"),
            ("test-only timings", t / "timings-receipt.json", lambda d: d.update(production=False), "timings receipt binding")):
        with edit_json(path, change):
            expect_refusal(f"D4 bind refuses {name}", lambda: H.bind(c3, "02", str(o), "90"), fragment)
    with edited(n / "phrase-timings.json", lambda p: p.write_bytes(p.read_bytes() + b" ")):
        expect_refusal("D5 bind refuses an installed phrase-timings.json that differs", lambda: H.bind(c3, "02", str(o), "90"),
                       "is not installed")
    aside = SCR / "aside"
    aside.mkdir()
    shutil.move(n / "phrase-timings.json", aside / "phrase-timings.json")
    expect_refusal("D6 bind refuses a narration without the installed phrase timings", lambda: H.bind(c3, "02", str(o), "90"),
                   "is not installed")
    shutil.move(aside / "phrase-timings.json", n / "phrase-timings.json")

    def wav_and_timings(p):
        data = bytearray(p.read_bytes())
        data[-1] ^= 1
        p.write_bytes(bytes(data))
    with edited(n / "03_1.wav", wav_and_timings):
        expect_refusal("D7 bind refuses an assembled WAV that the phrase timings do not name", lambda: H.bind(c3, "02", str(o), "90"),
                       "phrase timing WAV differs for 03_1")

        def follow(d):
            d["cues"]["03_1"]["wav_sha256"] = N.sha(n / "03_1.wav")
        with edit_json(t / "phrase-timings.json", follow), edit_json(n / "phrase-timings.json", follow):
            expect_refusal("D8 with the timings forged to match, the real coherent verifier refuses the changed WAV",
                           lambda: H.bind(c3, "02", str(o), "90"), "coherent_narration verify refused")
    with edited(o / "workspace/tools/narration_cache.py", lambda p: p.write_text(p.read_text() + "\n")):
        expect_refusal("D9 bind refuses an assembly workspace whose tools are not canonical", lambda: H.bind(c3, "02", str(o), "90"),
                       "not canonical")
    record("D10 no refused bind wrote a binding", not (WS / "narration-binding90.json").exists())

    # --- S: the narrated steps under the emulated guard
    def step(*args):
        return helper("step", "--w", W, "--ws", WS, "--binding", "01", *args)

    def verify(*args):
        return helper("verify-step", "--w", W, "--ws", WS, "--binding", "01", *args)
    code, written = step("--series", "01", "--step", "timeline")
    data = result(written)
    argv = data.get("consumer_argv", [[]])[0]
    record("S1 step timeline writes the job: 60 fps timeline-only render of all chapters with the bound narration",
           code == 0 and argv[1:] == ["render", "all", "--timeline-only", "--fps", "60", "--narration", str(n), "--out",
                                      str(LANE / f"{STAGE}-timeline01/timeline-output")] and argv[0] == str(WS / f"{STAGE}-ws-build02-binary"),
           data.get("launch_from_repo_root", tail(written, 300)))
    before_calls = len(calls())
    run = subprocess.run(["python3", "-I", "-B", data["script"]], cwd=REPO, env=stub_env(), capture_output=True, text=True)
    record("S2 a step script refuses outside the guard, before the binary runs",
           run.returncode == 1 and "launch only through the guard" in run.stdout and len(calls()) == before_calls, run.stdout.strip())
    code, log = launch(written)
    record("S3 the timeline job runs the real coherent verifier, then the stand-in; 13 chapters, 80 cues in order",
           code == 0 and log.strip().splitlines()[-1] == H.SUCCESS["timeline"], log.strip().splitlines()[-1] if log.strip() else "")
    code, out = verify("--series", "01", "--step", "timeline")
    record("S4 verify-step timeline", code == 0, tail(out, 300) if code else f"{result(out).get('cues')} cues, "
           f"{result(out).get('total_frames')} frames")
    code, out = step("--series", "01", "--step", "render", "--chapter", H.CHAPTERS[0])
    record("S5 a render step refuses before the score has passed", code == 1 and "no guard receipt for" in out, tail(out, 160))
    code, written = step("--series", "01", "--step", "preview")
    data = result(written)
    exit_p, log = launch(written)
    code_v, out = verify("--series", "01", "--step", "preview")
    record("S6 preview: 960x540 at 30 fps with real narration, cue order equal to the timeline",
           code == 0 and exit_p == 0 and code_v == 0 and data["consumer_argv"][0][3:7] == ["--res", "960x540", "--fps", "30"],
           " ".join(data.get("consumer_argv", [[""]])[0][1:]) if code == 0 else tail(written, 300))
    code, written = step("--series", "01", "--step", "score")
    exit_s, log = launch(written)
    code_v, out = verify("--series", "01", "--step", "score")
    record("S7 score: QA passed for every chapter", code == 0 and exit_s == 0 and code_v == 0, tail(out, 200) if code_v else
           " ".join(result(written)["consumer_argv"][0][1:]))
    first_ch = H.CHAPTERS[0]
    code, written = step("--series", "01", "--step", "render", "--chapter", first_ch)
    exit_r, log = launch(written, STUB_SHIFT="1")
    record("S8 a 4K chapter whose schedule differs from the timeline fails in the job",
           code == 0 and exit_r == 1 and "schedule differs" in log, log.strip().splitlines()[-1] if log.strip() else tail(written, 200))
    code, out = step("--series", "01", "--step", "render", "--chapter", first_ch, "--retry", "1")
    record("S9 a retry refuses while the failed chapter's mp4 is still in the work folder", code == 1 and "exists in the render work" in out,
           tail(out, 160))
    work = LANE / f"{STAGE}-render-work01"
    for ext in ("mp4", "timeline.tsv", "cuelog.tsv", "cues.tsv", "layout.tsv"):
        shutil.move(work / f"{first_ch}.{ext}", aside / f"{first_ch}.{ext}")
    ok, detail = True, []
    for chapter in H.CHAPTERS:
        extra = ["--retry", "1"] if chapter == first_ch else []
        code, written = step("--series", "01", "--step", "render", "--chapter", chapter, *extra)
        exit_r, log = launch(written)
        code_v, out = verify("--series", "01", "--step", "render", "--chapter", chapter, *extra)
        ok = ok and code == 0 and exit_r == 0 and code_v == 0
        if not ok:
            detail.append(f"{chapter}: {tail(written, 200)} {tail(log, 200)} {tail(out, 200)}")
            break
    argv = result(written).get("consumer_argv", [[]])[0]
    record("S10 13 chapters at 3840x2160, 60 fps, scored, --lint --crf 16 --preset veryfast; schedules equal the timeline",
           ok and argv[1:] == ["render", H.CHAPTERS[-1], "--res", "3840x2160", "--fps", "60", "--narration", str(n), "--score",
                               str(LANE / f"{STAGE}-score01/scored/stems"), "--out", str(work), "--lint", "--crf", "16", "--preset", "veryfast"],
           "; ".join(detail) or " ".join(argv[1:]))
    code, written = step("--series", "01", "--step", "master")
    data = result(written)
    exit_m, log = launch(written)
    code_v, out = verify("--series", "01", "--step", "master")
    record("S11 master binds the retried chapter, writes the master and captions",
           code == 0 and exit_m == 0 and code_v == 0 and f"{STAGE}-render-{first_ch}-01-r1" in data.get("verified_predecessors", {})
           and data["consumer_argv"][1][1:3] == ["captions", "all"], tail(out, 300) if code_v else
           " | ".join(" ".join(a[1:]) for a in data["consumer_argv"]))
    code, out = step("--series", "01", "--step", "master")
    record("S12 a step that exists is not rewritten", code == 1 and "exists" in out, tail(out, 160))
    code, written = step("--series", "02", "--step", "timeline")
    exit_t, log = launch(written, STUB_SWAP="1")
    code_v, out = verify("--series", "02", "--step", "timeline")
    record("S13 a timeline whose cues are out of script order fails in the job and verify-step refuses it",
           exit_t == 1 and "differ from the script order" in log and code_v == 1, tail(out, 160))
    code, written = step("--series", "03", "--step", "timeline")
    launch(written)
    code, written = step("--series", "03", "--step", "score")
    exit_s, log = launch(written, STUB_QA_FAIL="1")
    record("S14 a failed score QA fails the score job", exit_s == 1 and "score QA did not pass" in log,
           log.strip().splitlines()[-1] if log.strip() else "")
    with edited(n / "report.json", lambda p: p.write_bytes(p.read_bytes() + b" ")):
        code, out = step("--series", "04", "--step", "timeline")
        record("S15 a step refuses when a bound narration file moved", code == 1 and "bound narration files moved" in out,
               tail(out, 160))

    # --- U: real inputs untouched
    after = snapshot()
    changed = sorted(k for k in set(before) | set(after) if before.get(k) != after.get(k))
    record("U1 the real W stage, receipts lane, EL stage09, tools and shared binary are untouched", not changed, ", ".join(changed[:5]))
    passed = sum(RESULTS)
    print(f"{passed} PASS, {len(RESULTS) - passed} FAIL")
    return 0 if all(RESULTS) else 1


def launch_build(nn, binary):
    job = f"{STAGE}-ws-build{nn}"
    return emulated_guard(job, WS / f"{job}-inputs.json", ["sh", str(WS / f"{job}.sh")], STUB_BINARY=str(binary))


if __name__ == "__main__":
    sys.exit(main())
