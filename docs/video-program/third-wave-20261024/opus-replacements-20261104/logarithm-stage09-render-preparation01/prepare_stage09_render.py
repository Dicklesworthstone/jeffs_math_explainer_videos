#!/usr/bin/env python3
"""Logarithm stage09 narrated render preparation (BlueSalmon Mail 3999, bead 87do.63). Root runs every step.

Why a workspace. Root's stage09 build01 binary (187e119d) was compiled with its topic at W/logarithm-stage09
(main.rs passes env!("CARGO_MANIFEST_DIR")). Before a narrated render, captions, score or master, the kit runs
`python3 -I -B <topic>/../../tools/narration_delivery.py`, and that tool refuses any topic that is not in its own
REPO/explainer_corpus. W's parent has no tools/ folder and W/logarithm-stage09 is not in an explainer_corpus
folder, so build01 can render only silent or scratch-voice pictures. This helper installs the same 48 crate bytes
as WS/explainer_corpus/logarithm_mysteries beside byte copies of the three canonical narration tools in
WS/tools, writes one guarded rebuild on the same W engine with the same cargo lines, and then, once BronzeJaguar's
audio preparation has produced real production receipts, binds that narration and writes the guarded narrated
jobs one step at a time. Nothing is promoted, and nothing here launches, compiles or renders.

    python3 -I -B prepare_stage09_render.py check        --w W
    python3 -I -B prepare_stage09_render.py install      --w W --ws WS
    python3 -I -B prepare_stage09_render.py build-script --w W --ws WS --attempt 01   (then root's guard line)
    python3 -I -B prepare_stage09_render.py build-verify --w W --ws WS --attempt 01
    python3 -I -B prepare_stage09_render.py bind         --w W --ws WS --build 01 --audio O --attempt 01
    python3 -I -B prepare_stage09_render.py step         --w W --ws WS --binding 01 --series 01 --step timeline
    python3 -I -B prepare_stage09_render.py verify-step  --w W --ws WS --binding 01 --series 01 --step timeline
        steps in order: timeline, preview (optional), score, render --chapter CH (x13, --retry N), master

Run from the repository root. W and WS are absolute; WS is new and outside the repository, because the
installed manifest and every generated job name absolute scratch paths. O is the audio preparation's prepared
folder (its narration/ and the sibling O-timings/). --receipts and --target-root exist for the checks' scratch
copies; root leaves them at their defaults. Exit 0 pass, 1 refused, 2 usage.
"""
import argparse
import ast
import hashlib
import importlib.util
import json
import re
import shlex
import shutil
import subprocess
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
REPO = HERE.parents[5]
P_REL = "docs/video-program/third-wave-20261024/opus-replacements-20261104"
NATIVE_REL = f"{P_REL}/logarithm-stage09-native-preparation01/prepare_stage09_native.py"
NATIVE_SHA = "05e482751739c65bf381540e021e62efc09f8e7148352888e6c1569743b0ec85"
NATIVE_SUMS_SHA = "789d0db1eda9a15c842abf95253d8d01218a2332ba653e8697c633a3d9b40879"
AUDIO_REL = f"{P_REL}/logarithm-stage09-audio-preparation01"
AUDIO_SUMS_SHA = "2bb8e2800fd4d403ebd7bc73f8f6515b53a227125dc6a418fdd74e5e4c40bb13"  # the bundle LavenderPike reviewed
AUDIO_BINDING_SHA = "2d7f4731d373b5089686f5d345834faeb1a7238dc15007abfc7b4fab420d6803"
AUDIO_HELPER_SHA = "c6c854ce62f86ecb4025a9999e883ff04e28a29492788fec05c9060238cdb1e3"
VOICE = {"provider": "ElevenLabs", "name": "Oliver"}  # owner: logarithm keeps Oliver; no mixed voice
RECEIPT_SOURCE = "ElevenLabs UI, Oliver"
# The narration tools the kit and the assembly use, as root installed them (working tree, Mail 3920 bytes).
TOOLS = {
    "narration_delivery.py": "961f17cc85dd30eadca73f950ec0bab6e52eb47ad43a7384c3710c97962b1878",
    "coherent_narration.py": "95615cfadb42fbfdddf828340b74ec66d89c975a1f399593e212bdd9840e65d5",
    "narration_cache.py": "47331efd3952af6d0e66971a1c8fddd08929f881d1f9b470bde223db7a01b0c0",
}
BUILD01_BINARY = "187e119d8e9909670bc6bfc88abfc636876ee498dde97a697a0c1a7bdec2330a"
GAIN_DB = "0"  # explainer_kit::narration::GAIN_DB = 0.0, as Rust's f64 Display prints it into the contract argv
TOPIC = "logarithm_mysteries"
STEPS = ("timeline", "preview", "score", "render", "master")
CONTINUES = rb"(?![-A-Za-z0-9_.])"  # a path mention must not continue into a longer folder name


def load_native():
    path = REPO / NATIVE_REL
    digest = hashlib.sha256(path.read_bytes()).hexdigest()
    if digest != NATIVE_SHA:
        raise SystemExit(f"REFUSED: the sealed native helper is {digest[:12]}, expected {NATIVE_SHA[:12]}")
    spec = importlib.util.spec_from_file_location("prepare_stage09_native", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


N = load_native()
Refused, need, sha, tree, pinned = N.Refused, N.need, N.sha, N.tree, N.pinned
CHAPTERS, STAGE, CRATE = N.CHAPTERS, N.STAGE, N.CRATE


def receipt_ok(r):
    return (r.get("admitted") is True and r.get("exit") == 0 and bool(r.get("finished"))
            and r.get("inputs_sha256") == r.get("final_inputs_sha256")
            and not r.get("stopped_for_resource_limit") and "error" not in r)


def mentions(data, path):
    return len(re.findall(re.escape(str(path).encode()) + CONTINUES, data))


def write_new(path, text):
    with open(path, "x") as handle:
        handle.write(text)


class Ctx:
    def __init__(self, a):
        self.native = N.Paths(argparse.Namespace(w=a.w, into=a.w, el=None, white=None, receipts=None, target_root=None))
        self.w = self.native.w
        self.lane = Path(a.receipts) if a.receipts else REPO / N.RECEIPTS_REL  # the guard's work folder
        self.target = Path(a.target_root) if a.target_root else REPO / "target"
        self.ws = None
        if getattr(a, "ws", None):
            self.ws = Path(a.ws)
            need(self.ws.is_absolute(), "--ws must be absolute")
            self.crate = self.ws / CRATE
            self.tools = self.ws / "tools"
            self.ws_binding = self.ws / "workspace-binding01.json"
            self.jobs = self.ws / "jobs"

    def stage_rows(self):
        binding = json.loads((self.w / f"{STAGE}-source-binding01.json").read_text())
        return {r["path"]: r["stage_sha256"] for r in binding["files"]}


# ---------------------------------------------------------------- check

def script_cues(path):
    return list(N.script_rows(Path(path).read_text(encoding="utf-8")))


def check(c):
    for name, digest in TOOLS.items():
        pinned(REPO / "tools" / name, digest, f"tools/{name}")
    pinned(REPO / NATIVE_REL.replace("prepare_stage09_native.py", "SHA256SUMS"), NATIVE_SUMS_SHA, "native seal")
    native = N.verify_run(c.native, "01")  # root's build01: receipt, 241 tests, Clippy, chapters, stage binding
    need(native["stage09_binary"] == BUILD01_BINARY, f"build01 binary is {native['stage09_binary'][:12]}, expected 187e119d")
    need(native["preserved_binary"] == N.STAGE08_BINARY, "build01 did not preserve stage08's e9233401")
    data = (c.w / f"{STAGE}-build01-binary").read_bytes()
    compiled = mentions(data, c.native.stage)
    need(compiled > 0, "build01 does not name W/logarithm-stage09: the blocker below no longer describes it")
    kit_cli = c.w / "repo-view/crates/explainer_kit/src/cli.rs"
    text = kit_cli.read_text()
    need('repo.join("tools/narration_delivery.py")' in text and "check_delivery_contract" in text,
         "the W kit no longer runs tools/narration_delivery.py before narrated consumers")
    delivery = (REPO / "tools/narration_delivery.py").read_text()
    need("require(topic.parent == REPO/'explainer_corpus', 'topic must be in this repository')" in delivery,
         "narration_delivery.py no longer requires an explainer_corpus topic")
    return {"build01": {k: native[k] for k in ("job", "tests", "chapters", "stage09_binary", "preserved_binary")},
            "blocker": {"compiled_topic": str(c.native.stage), "compiled_topic_mentions_in_build01": compiled,
                        "kit_runs": str(c.native.stage.parent.parent / "tools/narration_delivery.py"),
                        "kit_tool_present": (c.native.stage.parent.parent / "tools/narration_delivery.py").exists(),
                        "topic_parent_is_explainer_corpus": c.native.stage.parent.name == "explainer_corpus",
                        "kit_cli_sha256": sha(kit_cli),
                        "consequence": "read from the kit and tool sources, not run: every narrated render, captions, "
                                       "score and master of build01 refuses before any output"},
            "tools": TOOLS}


# ---------------------------------------------------------------- workspace

def install(c):
    facts = check(c)
    ws = c.ws
    need(not ws.exists() and not ws.is_symlink(), f"refusing to overwrite {ws}")
    need(ws.parent.is_dir(), f"{ws.parent} does not exist")
    real = ws.parent.resolve() / ws.name
    need(not real.is_relative_to(REPO.resolve()), "--ws is inside the repository: it names absolute scratch paths")
    for forbidden in (c.w / "engine", c.w / "repo-view", c.w / "logarithm-stage08", c.native.stage):
        need(not real.is_relative_to(forbidden.resolve()), f"--ws is inside {forbidden}")
    rows = c.stage_rows()
    need(tree(c.native.stage) == rows, "W/logarithm-stage09 differs from root's stage binding")
    ws.mkdir()
    for rel in sorted(rows):
        (c.crate / rel).parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(c.native.stage / rel, c.crate / rel)
    c.tools.mkdir()
    for name in TOOLS:
        shutil.copy2(REPO / "tools" / name, c.tools / name)
    binding = {
        "schema": "logarithm-stage09-render-workspace-v1",
        "helper_sha256": sha(HERE), "native_helper_sha256": NATIVE_SHA,
        "w": str(c.w), "ws": str(ws), "from_stage": str(c.native.stage),
        "from_stage_binding_sha256": sha(c.w / f"{STAGE}-source-binding01.json"),
        "crate": {rel: sha(c.crate / rel) for rel in sorted(rows)},
        "tools": {name: sha(c.tools / name) for name in TOOLS},
        "build01_binary": BUILD01_BINARY,
        "why": "The kit runs <topic>/../../tools/narration_delivery.py, which requires <its repository>/explainer_corpus/"
               "<topic>. This layout gives the same 48 crate bytes that root's build01 compiled such a repository; "
               "the tools are byte copies of the repository's. Only the crate's location changes.",
    }
    write_new(c.ws_binding, json.dumps(binding, indent=2) + "\n")
    verify_ws(c)
    return {"ws": str(ws), "crate_files": len(rows), "tools": len(TOOLS), "workspace_binding_sha256": sha(c.ws_binding),
            "build01_binary": facts["build01"]["stage09_binary"]}


def verify_ws(c):
    need(c.ws is not None and c.ws_binding.is_file(), "no workspace binding: run install first")
    b = json.loads(c.ws_binding.read_text())
    need(b.get("helper_sha256") == sha(HERE), "the workspace was installed by another version of this helper")
    need(b["w"] == str(c.w) and b["ws"] == str(c.ws), "the workspace binding names another W or WS")
    rows = c.stage_rows()
    need(tree(c.crate) == rows == b["crate"], "the workspace crate differs from root's stage09 bytes")
    need(tree(c.tools) == TOOLS == b["tools"], "the workspace tools differ from the pinned canonical bytes")
    need(sorted(p.name for p in (c.ws / "explainer_corpus").iterdir()) == [TOPIC], "explainer_corpus holds another entry")
    return b


# ---------------------------------------------------------------- guarded rebuild

def build_paths(c, nn):
    job = f"{STAGE}-ws-build{nn}"
    return {"job": job, "script": c.ws / f"{job}.sh", "inputs": c.ws / f"{job}-inputs.json",
            "before": c.ws / f"before-{job}-binary", "after": c.ws / f"{job}-binary",
            "sums": c.ws / f"{job}-binaries.sha256", "receipt": c.lane / job}


def build_text(c, j, before_sha):
    q = shlex.quote
    if before_sha is None:
        preserve = ['[ ! -e "$B" ] || refuse "a shared binary appeared after this script was written"']
    else:
        preserve = [f'[ -f "$B" ] && [ "$(sha "$B")" = {before_sha} ] || refuse "the shared binary is not the one this script preserves"',
                    'cp -p "$B" "$BEFORE"',
                    f'[ "$(sha "$BEFORE")" = {before_sha} ] || refuse "the preserved copy differs"']
    lines = [
        f"# {j['job']}: the stage09 tests, binary and strict Clippy built from the render workspace, so the binary's",
        f"# topic is WS/{CRATE}. Written by prepare_stage09_render.py (sha256 {sha(HERE)}).",
        "# Same engine (W/engine-budget.toml) and cargo lines as logarithm-stage09-build01; launch only through the guard.",
        "set -eu",
        'refuse() { echo "REFUSED: $1" >&2; exit 1; }',
        "sha() { shasum -a 256 \"$1\" | cut -d ' ' -f 1; }",
        f"W={q(str(c.w))}",
        f"WS={q(str(c.ws))}",
        f"R={q(str(REPO))}",
        f"T={q(str(c.target))}",
        '[ "${CARGO_TARGET_DIR:-}" = "$T" ] || refuse "CARGO_TARGET_DIR is not the guard\'s target"',
        "export CARGO_INCREMENTAL=0",
        "export RCH_SHIM_LOCAL_IDE=1",
        'export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"',
        'cd "$R"',
        f'M="$WS/{CRATE}/Cargo.toml"',
        'B="$CARGO_TARGET_DIR/debug/logarithm_mysteries"',
        f'BEFORE="$WS/{j["before"].name}"',
        f'AFTER="$WS/{j["after"].name}"',
        f'SUMS="$WS/{j["sums"].name}"',
        'for f in "$BEFORE" "$AFTER" "$SUMS"; do [ ! -e "$f" ] || refuse "$f exists"; done',
        *preserve,
        f'python3 -I -B "$R/{N.LOCK_CHECK_REL}" "$W/{STAGE}-original-Cargo.lock" "$WS/{CRATE}/Cargo.lock"',
        'cargo test --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets',
        'cargo build --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --bin logarithm_mysteries',
        'cp -p "$B" "$AFTER"',
        'cargo clippy --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets --no-deps -- -D warnings',
        'cmp "$B" "$AFTER" || refuse "Clippy changed the shared binary"',
        f'(cd "$WS" && shasum -a 256 {"" if before_sha is None else q(j["before"].name) + " "}{q(j["after"].name)}) > "$SUMS"',
        '"$AFTER" chapters',
    ]
    return "\n".join(lines) + "\n"


def build_rows(c, j):
    paths = [j["script"], c.ws_binding, *(c.crate / rel for rel in sorted(c.stage_rows())), *(c.tools / n for n in TOOLS)]
    stage = c.native.stage
    paths += [stage / rel for rel in sorted(tree(stage))]
    paths += [c.w / f"{STAGE}-source-binding01.json", c.w / f"{STAGE}-original-Cargo.toml", c.w / f"{STAGE}-original-Cargo.lock"]
    paths += [c.w / name for name in (f"before-{STAGE}-build01-binary", f"{STAGE}-build01-binary", f"{STAGE}-build01-binaries.sha256")]
    for sub in ("engine", "repo-view"):
        paths += [c.w / sub / rel for rel in sorted(tree(c.w / sub))]
    paths += [c.w / rel for rel in N.W_PIN if "/" not in rel]
    paths += [REPO / NATIVE_REL, REPO / NATIVE_REL.replace("prepare_stage09_native.py", "SHA256SUMS"), HERE]
    paths += [REPO / rel for rel in N.TOOL_PIN]
    shared = c.target / "geometry-kit01"
    paths += [shared / rel for rel in N.FILM_BINARIES if (shared / rel).is_file()]
    need(len(set(paths)) == len(paths), "an input row repeats")
    return [{"path": str(p), "sha256": sha(p)} for p in paths]


def build_script(c, nn):
    check(c)
    verify_ws(c)
    j = build_paths(c, nn)
    for key in ("script", "inputs", "before", "after", "sums", "receipt"):
        need(not j[key].exists() and not j[key].is_symlink(), f"{j[key]} exists: use a new attempt number")
    shared = c.target / "geometry-kit01/debug/logarithm_mysteries"
    before_sha = sha(shared) if shared.is_file() else None
    need(before_sha is not None or not shared.exists(), f"{shared} is not a regular file")
    write_new(j["script"], build_text(c, j, before_sha))
    rows = build_rows(c, j)
    write_new(j["inputs"], json.dumps({"files": rows}, indent=2) + "\n")
    return {"job": j["job"], "script_sha256": sha(j["script"]), "inputs_sha256": sha(j["inputs"]), "input_rows": len(rows),
            "preserves": before_sha or "absent (nothing to preserve)", "preserves_build01_binary": before_sha == BUILD01_BINARY,
            "launch_from_repo_root": f"python3 -I -B {N.GUARD_REL} --phase native --job {j['job']} "
                                     f"--inputs {j['inputs']} -- sh {j['script']}"}


def build_verify(c, nn):
    verify_ws(c)
    j = build_paths(c, nn)
    need((j["receipt"] / "resource-receipt.json").is_file(), f"no guard receipt for {j['job']}")
    r = json.loads((j["receipt"] / "resource-receipt.json").read_text())
    need(r.get("phase") == "native" and r.get("command") == ["sh", str(j["script"])], f"the receipt ran {r.get('command')}")
    need(receipt_ok(r), f"the job did not pass: exit {r.get('exit')}, error {r.get('error')}")
    need(r["inputs_sha256"] == sha(j["inputs"]), "the guard's input hash is not this attempt's inputs file")
    need(json.loads(j["inputs"].read_text())["files"][0]["sha256"] == sha(j["script"]), "the script moved after launch")
    log = (j["receipt"] / "command.log").read_text(errors="replace")
    need("PASS: the 19 packages are path copies, every other record is identical" in log, "lock check did not pass")
    results = re.findall(r"^test result: .*$", log, re.M)
    need(len(results) == 1 and results[0].startswith(
        f"test result: ok. {N.TESTS} passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"), f"test results: {results}")
    tail = [line for line in log.splitlines() if line.strip()][-len(CHAPTERS):]
    need(tail == CHAPTERS, f"the chapters listing ends {tail[-3:]}")
    sums = dict(reversed(line.split("  ", 1)) for line in j["sums"].read_text().splitlines())
    need(sums.get(j["after"].name) == sha(j["after"]), "the workspace binary copy moved")
    if j["before"].exists():
        need(sums.get(j["before"].name) == sha(j["before"]), "the preserved binary moved")
    data = j["after"].read_bytes()
    topic, old = mentions(data, c.crate), mentions(data, c.native.stage)
    need(topic > 0, "the new binary does not name the workspace topic")
    need(old == 0, f"the new binary still names W/logarithm-stage09 ({old} times)")
    return {"job": j["job"], "tests": f"{N.TESTS} passed", "clippy": "-D warnings, exit 0 (set -e)", "chapters": len(CHAPTERS),
            "binary": str(j["after"]), "binary_sha256": sums[j["after"].name],
            "preserved_binary": sums.get(j["before"].name, "none"), "workspace_topic_mentions": topic}


# ---------------------------------------------------------------- audio binding

def audio_paths(audio):
    o = Path(audio)
    need(o.is_absolute(), "--audio must be absolute")
    return {"prepared": o, "narration": o / "narration", "timings": Path(f"{o}-timings")}


def run_tool(argv):
    return subprocess.run(argv, cwd=REPO, capture_output=True, text=True, timeout=900)


def bind(c, build_nn, audio, mm):
    check(c)
    build = build_verify(c, build_nn)
    out = c.ws / f"narration-binding{mm}.json"
    need(not out.exists(), f"{out} exists: use a new attempt number")
    a = audio_paths(audio)
    pr_path = a["prepared"] / "prepare-receipt.json"
    need(pr_path.is_file(), f"no prepare receipt at {pr_path}: the real audio preparation has not run")
    for rel in ("SHA256SUMS", "bindings/logarithm-stage09-oliver.json", "prepare_audio.py"):
        pinned(REPO / AUDIO_REL / rel, {"SHA256SUMS": AUDIO_SUMS_SHA, "prepare_audio.py": AUDIO_HELPER_SHA}.get(rel, AUDIO_BINDING_SHA),
               f"audio preparation {rel}")
    pr = json.loads(pr_path.read_text())
    want = {"sha256": AUDIO_BINDING_SHA, "use": "production", "voice": VOICE}
    need({k: pr.get("binding", {}).get(k) for k in want} == want, f"prepare receipt binding {pr.get('binding')}")
    need(pr.get("production") is True and pr.get("chapter") is None, "the prepare receipt is not a whole-film production run")
    cues = script_cues(c.crate / "script.tsv")
    need(list(pr.get("cues", {})) == cues, f"the prepare receipt covers {len(pr.get('cues', {}))} cues, not the script's 80")
    spans = a["prepared"] / "spans.json"
    need(pr.get("spans_sha256") == sha(spans), "spans.json moved since prepare")
    need(json.loads(spans.read_text()).get("units_sha256") == N.PIN["units"], "spans were not made for units 1132a5a4")
    decode = Path(pr["decode_receipt"]["path"])
    decode = decode if decode.is_absolute() else REPO / decode
    need(sha(decode) == pr["decode_receipt"]["sha256"], "the decode receipt moved")
    dr = json.loads(decode.read_text())
    need(dr.get("source") == RECEIPT_SOURCE and dr.get("production") is True, "the decode receipt is not production Oliver")
    for name in ("coherent_narration.py", "narration_cache.py"):
        need(sha(a["prepared"] / "workspace/tools" / name) == TOOLS[name], f"the assembly workspace's {name} is not canonical")
    n = a["narration"]
    assembly = json.loads((n / "assembly.json").read_text())
    need(assembly.get("manifest") == json.loads(spans.read_text()), "the assembly was not made from this spans.json")
    tr_path = a["timings"] / "timings-receipt.json"
    need(tr_path.is_file(), f"no timings receipt at {tr_path}")
    tr = json.loads(tr_path.read_text())
    need({k: tr.get("binding", {}).get(k) for k in want} == want and tr.get("production") is True, "timings receipt binding")
    need(tr.get("cues") == len(cues) and tr.get("file") == "phrase-timings.json", "the timings receipt is not whole-film")
    need(tr.get("assembly_sha256") == sha(n / "assembly.json") and tr.get("report_sha256") == sha(n / "report.json"),
         "the timings receipt names another assembly")
    timings_file = a["timings"] / "phrase-timings.json"
    pt = json.loads(timings_file.read_text())
    need(pt.get("voice") == VOICE["name"] and pt.get("production") is True and pt.get("binding_sha256") == AUDIO_BINDING_SHA
         and pt.get("script_sha256") == N.PIN["script"] and pt.get("prepare_receipt_sha256") == sha(pr_path),
         "phrase-timings.json is not this production preparation's")
    need(list(pt.get("cues", {})) == cues, "phrase-timings.json does not cover the script's cues in order")
    texts = N.script_rows((c.crate / "script.tsv").read_text(encoding="utf-8"))
    for cue in cues:
        entry = pt["cues"][cue]
        need(entry["wav_sha256"] == sha(n / f"{cue}.wav"), f"phrase timing WAV differs for {cue}")
        need(entry["text_sha256"] == hashlib.sha256(texts[cue].encode()).hexdigest(), f"phrase timing text differs for {cue}")
    installed = n / "phrase-timings.json"
    need(installed.is_file() and installed.read_bytes() == timings_file.read_bytes(),
         "phrase-timings.json is not installed in the narration (timings-receipt 'install' line)")
    verify_argv = ["python3", "-I", "-B", str(c.tools / "coherent_narration.py"), "verify", str(n), "--script",
                   str(c.crate / "script.tsv"), "--units", str(c.crate / "narration_units.json"), "--require-directions"]
    v = run_tool(verify_argv)
    need(v.returncode == 0, f"coherent_narration verify refused: {(v.stderr or v.stdout).strip()[-500:]}")
    contract_argv = ["python3", "-I", "-B", str(c.tools / "narration_delivery.py"), "--topic", str(c.crate), "--narration",
                     str(n), "--consumer", "render", "--gain-db", GAIN_DB]
    for key in CHAPTERS:
        contract_argv += ["--chapter-key", key]
    d = run_tool(contract_argv)
    need(d.returncode == 0, f"the kit's delivery contract refused: {(d.stderr or d.stdout).strip()[-500:]}")
    contract = json.loads(d.stdout)
    need(contract.get("verified_cues") == contract.get("selected_cues") == len(cues) and contract.get("topic") == TOPIC
         and contract.get("chapter_scope") is None and contract.get("promotion") is False, f"contract result {contract}")
    files = {str(p): sha(p) for p in (pr_path, spans, decode, tr_path, timings_file)}
    files.update({str(n / rel): h for rel, h in tree(n).items()})
    for record in dr.get("records", []):
        wav = Path(record["wav"]) if Path(record["wav"]).is_absolute() else REPO / record["wav"]
        need(sha(wav) == record["wav_sha256"], f"decoded {record.get('cue')} moved")
        files[str(wav)] = record["wav_sha256"]
    bound_plan = REPO / N.EL_REL / "combined-repair-stage09/bound-plan"
    files.update({str(bound_plan / rel): h for rel, h in tree(bound_plan).items()})
    binding = {
        "schema": "logarithm-stage09-render-narration-binding-v1", "helper_sha256": sha(HERE), "ws": str(c.ws),
        "workspace_binding_sha256": sha(c.ws_binding),
        "build": {"attempt": build_nn, "job": build["job"], "binary": build["binary"], "binary_sha256": build["binary_sha256"]},
        "audio": {**{k: str(v) for k, v in a.items()}, "binding_sha256": AUDIO_BINDING_SHA, "helper_sha256": AUDIO_HELPER_SHA,
                  "sums_sha256": AUDIO_SUMS_SHA},
        "voice": VOICE, "cues": len(cues), "plan_mode": contract.get("plan_mode"),
        "checks": {"coherent_verify": verify_argv, "delivery_contract": contract_argv},
        "files": files,
        "claim": "Technical binding of real production receipts only. Nothing here was heard, rendered or accepted.",
    }
    write_new(out, json.dumps(binding, indent=2) + "\n")
    return {"binding": str(out), "binding_sha256": sha(out), "bound_files": len(files), "plan_mode": contract.get("plan_mode"),
            "binary_sha256": build["binary_sha256"], "voice": VOICE}


def load_binding(c, mm):
    verify_ws(c)
    path = c.ws / f"narration-binding{mm}.json"
    need(path.is_file(), f"no narration binding {path}: bind needs a real audio receipt first")
    b = json.loads(path.read_text())
    need(b.get("helper_sha256") == sha(HERE) and b["ws"] == str(c.ws), "the narration binding is another helper's or WS's")
    need(b["workspace_binding_sha256"] == sha(c.ws_binding), "the workspace binding moved")
    build = build_verify(c, b["build"]["attempt"])
    need(build["binary_sha256"] == b["build"]["binary_sha256"], "the bound binary moved")
    moved = [p for p, h in b["files"].items() if not Path(p).is_file() or sha(p) != h]
    need(not moved, f"bound narration files moved: {moved[:3]}")
    return b, path


# ---------------------------------------------------------------- narrated steps

def job_name(step, series, chapter=None, retry=None):
    if step == "render":
        return f"{STAGE}-render-{chapter}-{series}" + (f"-r{retry}" if retry else "")
    return f"{STAGE}-{step}{series}"


def job_files(c, job):
    return {"script": c.jobs / f"{job}.py", "inputs": c.jobs / f"{job}-inputs.json", "folder": c.lane / job}


def render_work(c, series):
    return c.lane / f"{STAGE}-render-work{series}"


SUCCESS = {
    "timeline": "TIMELINE PASS: 13 chapters at 60 fps; 80 cue placements in script order.",
    "preview": "PREVIEW RENDERED: 13 chapters at 960x540, 30 fps, real narration; pictures and sound not reviewed.",
    "score": "SCORE PASS: all 13 chapter score QA reports passed.",
    "render": "RENDER PASS: 4K60 chapter; frame and cue schedule equals the 60 fps timeline.",
    "master": "MASTER WRITTEN: candidate master and captions; strict verify, pictures, ASR, caption screen, loudness and listening remain.",
}
SCOPE = ("Technical gates only, logarithm stage09 with ElevenLabs Oliver: no listening, picture review, fine sync, "
         "independent review or final AV acceptance.")


def latest_render(c, series, chapter):
    names = [job_name("render", series, chapter)] + [job_name("render", series, chapter, r) for r in range(1, 10)]
    present = [name for name in names if (c.lane / name).exists()]
    need(present, f"{chapter}: no render job in series {series}")
    return present[-1], (None if present[-1] == names[0] else int(present[-1].rsplit("-r", 1)[1]))


def predecessors(c, step, series):
    tl, sc = job_name("timeline", series), job_name("score", series)
    if step == "timeline":
        return []
    if step in ("preview", "score"):
        return [tl]
    if step == "render":
        return [tl, sc]
    return [tl, sc] + [latest_render(c, series, ch)[0] for ch in CHAPTERS]


def output_rows(c, job, series):
    folder = c.lane / job
    paths = [folder / "resource-receipt.json", folder / "command.log"]
    if job.startswith(f"{STAGE}-render-"):
        chapter = json.loads((folder / "render-summary.json").read_text())["chapter"]
        work = render_work(c, series)
        paths += [folder / "render-summary.json"]
        paths += [work / f"{chapter}.{ext}" for ext in ("mp4", "timeline.tsv", "cuelog.tsv", "cues.tsv", "layout.tsv")
                  if (work / f"{chapter}.{ext}").is_file()]
    else:
        paths += [folder / rel for rel in sorted(tree(folder)) if rel not in ("resource-receipt.json", "command.log")]
    return paths


def step_config(c, b, step, series, job, files, chapter):
    n = b["audio"]["narration"]
    tl = c.lane / job_name("timeline", series)
    sc = c.lane / job_name("score", series)
    work = render_work(c, series)
    binary = b["build"]["binary"]
    cfg = {"job": job, "step": step, "script": str(files["script"]), "lane": str(c.lane), "target": str(c.target),
           "binary": binary, "binary_sha256": b["build"]["binary_sha256"], "topic": TOPIC, "narration": n,
           "chapters": CHAPTERS, "cues": script_cues(c.crate / "script.tsv"), "chapter": chapter,
           "timeline_job": tl.name, "render_work": str(work), "success": SUCCESS[step], "scope": SCOPE,
           "after": predecessors(c, step, series)}
    if step == "timeline":
        cfg["verify_argv"] = b["checks"]["coherent_verify"]
        cfg["argv"] = [binary, "render", "all", "--timeline-only", "--fps", "60", "--narration", n, "--out", str(c.lane / job / "timeline-output")]
    elif step == "preview":
        cfg["argv"] = [binary, "render", "all", "--res", "960x540", "--fps", "30", "--narration", n,
                       "--out", str(c.lane / job / "preview"), "--lint", "--crf", "28", "--preset", "ultrafast"]
    elif step == "score":
        cfg["argv"] = [binary, "score", "--narration", n, "--timelines", str(tl / "timeline-output"), "--out", str(c.lane / job / "scored")]
    elif step == "render":
        cfg["argv"] = [binary, "render", chapter, "--res", "3840x2160", "--fps", "60", "--narration", n,
                       "--score", str(sc / "scored/stems"), "--out", str(work), "--lint", "--crf", "16", "--preset", "veryfast"]
    else:
        cfg["render_jobs"] = dict(zip(CHAPTERS, cfg["after"][2:]))
        cfg["argv"] = [binary, "master", "--narration", n, "--score", str(sc / "scored/stems"), "--out", str(work)]
        cfg["captions_argv"] = [binary, "captions", "all", "--narration", n, "--out", str(work)]
    return cfg


def write_step(c, mm, series, step, chapter=None, retry=None):
    b, binding_path = load_binding(c, mm)
    need(step in STEPS, f"unknown step {step}")
    need((step == "render") == (chapter is not None), "--chapter goes with --step render only")
    need(chapter is None or chapter in CHAPTERS, f"unknown chapter {chapter}")
    need(retry is None or step == "render", "--retry goes with --step render only")
    job = job_name(step, series, chapter, retry)
    files = job_files(c, job)
    for key in ("script", "inputs", "folder"):
        need(not files[key].exists(), f"{files[key]} exists: use a new series (or --retry for a render)")
    if step == "render" and retry:
        earlier = job_name("render", series, chapter, retry - 1 if retry > 1 else None)
        need((c.lane / earlier).exists(), f"--retry {retry} needs {earlier} first")
        need(not (render_work(c, series) / f"{chapter}.mp4").exists(), f"{chapter}.mp4 exists in the render work folder")
    after = predecessors(c, step, series)
    facts = {}
    for prior in after:
        facts[prior] = verify_job(c, b, prior, series)
    cfg = step_config(c, b, step, series, job, files, chapter)
    c.jobs.mkdir(exist_ok=True)
    text = JOB_TEMPLATE.replace("@@JOB@@", job).replace("@@STEP@@", step).replace("@@HELPER@@", sha(HERE))
    write_new(files["script"], text.replace("@@CONFIG@@", repr(json.dumps(cfg, indent=1))))
    paths = [files["script"], HERE, binding_path, c.ws_binding, Path(b["build"]["binary"])]
    paths += [Path(p) for p in b["files"]]
    paths += [c.crate / rel for rel in sorted(c.stage_rows())] + [c.tools / name for name in TOOLS]
    paths += [REPO / N.GUARD_REL, REPO / N.GUARD_SOURCE_REL]
    for prior in after:
        paths += output_rows(c, prior, series)
    need(len(set(paths)) == len(paths), "an input row repeats")
    write_new(files["inputs"], json.dumps({"files": [{"path": str(p), "sha256": sha(p)} for p in paths]}, indent=2) + "\n")
    return {"job": job, "script": str(files["script"]), "script_sha256": sha(files["script"]),
            "inputs_sha256": sha(files["inputs"]), "input_rows": len(paths), "verified_predecessors": facts,
            "consumer_argv": [cfg["argv"]] + ([cfg["captions_argv"]] if step == "master" else []),
            "launch_from_repo_root": f"python3 -I -B {N.GUARD_REL} --phase native --job {job} "
                                     f"--inputs {files['inputs']} -- python3 -I -B {files['script']}"}


def timeline_rows(path):
    return [line.split("\t") for line in Path(path).read_text().splitlines() if line and not line.startswith("#")]


def first(table, key):
    return next(r[1] for r in table if r[0] == key)


def schedule(path):
    return [r for r in timeline_rows(path) if r and r[0] in ("chapter", "fps", "frames", "line")]


def verify_job(c, b, job, series):
    """A finished narrated job: the guard's receipt, its log's success line and the outputs it summarised."""
    files = job_files(c, job)
    folder = files["folder"]
    need((folder / "resource-receipt.json").is_file(), f"no guard receipt for {job}")
    r = json.loads((folder / "resource-receipt.json").read_text())
    need(r.get("phase") == "native" and r.get("command") == ["python3", "-I", "-B", str(files["script"])],
         f"{job}: the receipt ran {r.get('command')}")
    need(receipt_ok(r), f"{job} did not pass: exit {r.get('exit')}, error {r.get('error')}")
    need(r["inputs_sha256"] == sha(files["inputs"]), f"{job}: the guard's input hash is not this job's inputs file")
    need(json.loads(files["inputs"].read_text())["files"][0]["sha256"] == sha(files["script"]), f"{job}: the script moved")
    cfg = read_config(files["script"])
    need(cfg["job"] == job and cfg["lane"] == str(c.lane), f"{job}: the script names another job or lane")
    need(cfg["narration"] == b["audio"]["narration"] and cfg["binary_sha256"] == b["build"]["binary_sha256"],
         f"{job} used another narration or binary than this binding")
    lines = [line for line in (folder / "command.log").read_text(errors="replace").splitlines() if line.strip()]
    need(lines and lines[-1] == SUCCESS[cfg["step"]], f"{job}: the log does not end with its success line")
    cues = script_cues(c.crate / "script.tsv")
    step = cfg["step"]
    if step == "timeline":
        s = json.loads((folder / "timeline-summary.json").read_text())
        need(s["fps"] == 60 and [x["chapter"] for x in s["chapters"]] == CHAPTERS and s["cues"] == len(cues), f"{job}: summary")
        seen = []
        for item in s["chapters"]:
            table = timeline_rows(folder / "timeline-output" / f"{item['chapter']}.timeline.tsv")
            ids = [row[1] for row in table if row[0] == "line"]
            need(first(table, "fps") == "60" and int(first(table, "frames")) == item["frames"] and ids == item["cues"],
                 f"{job}: {item['chapter']} timeline differs from its summary")
            seen += ids
        need(seen == cues and s["total_frames"] == sum(x["frames"] for x in s["chapters"]), f"{job}: cue order or frames")
        return {"chapters": len(CHAPTERS), "cues": len(seen), "total_frames": s["total_frames"]}
    if step == "preview":
        s = json.loads((folder / "preview-summary.json").read_text())
        for item in s["chapters"]:
            need(sha(folder / "preview" / f"{item['chapter']}.mp4") == item["mp4_sha256"], f"{job}: {item['chapter']} preview moved")
        need([x["chapter"] for x in s["chapters"]] == CHAPTERS, f"{job}: preview chapters")
        return {"chapters": len(CHAPTERS), "layout_findings": sum(x["layout_findings"] or 0 for x in s["chapters"])}
    if step == "score":
        s = json.loads((folder / "score-summary.json").read_text())
        need(list(s["qa_sha256"]) == CHAPTERS, f"{job}: score QA chapters")
        for chapter, digest in s["qa_sha256"].items():
            qa = folder / "scored/stems/qa" / f"{chapter}.json"
            need(sha(qa) == digest and json.loads(qa.read_text()).get("passed") is True, f"{job}: {chapter} QA")
        return {"qa_passed": len(CHAPTERS)}
    if step == "render":
        s = json.loads((folder / "render-summary.json").read_text())
        work = Path(cfg["render_work"])
        need(s["chapter"] == cfg["chapter"] and sha(work / f"{s['chapter']}.mp4") == s["mp4_sha256"], f"{job}: chapter file moved")
        reference = c.lane / cfg["timeline_job"] / "timeline-output" / f"{s['chapter']}.timeline.tsv"
        need(schedule(reference) == schedule(work / f"{s['chapter']}.timeline.tsv"), f"{job}: schedule differs from the timeline")
        return {"chapter": s["chapter"], "mp4_sha256": s["mp4_sha256"], "layout_findings": s["layout_findings"]}
    s = json.loads((folder / "master-summary.json").read_text())
    for key in ("master", "srt", "vtt"):
        need(sha(s[key]["path"]) == s[key]["sha256"], f"{job}: {key} moved")
    return {k: s[k]["sha256"] for k in ("master", "srt", "vtt")}


def read_config(script):
    text = Path(script).read_text()
    match = re.search(r"^CONFIG = json\.loads\((.*)\)$", text, re.M)
    need(match is not None, f"{script} has no CONFIG line")
    return json.loads(ast.literal_eval(match.group(1)))


def verify_step(c, mm, series, step, chapter=None, retry=None):
    b, _ = load_binding(c, mm)
    need((step == "render") == (chapter is not None), "--chapter goes with --step render only")
    job = job_name(step, series, chapter, retry)
    result = verify_job(c, b, job, series)
    return {"job": job, **result, "not_claimed": SCOPE}


JOB_TEMPLATE = r'''#!/usr/bin/env python3
"""@@JOB@@: logarithm stage09 narrated @@STEP@@. Written by prepare_stage09_render.py (sha256 @@HELPER@@).

Launch only through root's guard from the repository root; the guard binds every input by hash.
Technical checks only: no listening, picture review, fine sync or acceptance is claimed here.
"""
import hashlib
import json
import os
import pathlib
import subprocess
import sys

CONFIG = json.loads(@@CONFIG@@)
LANE = pathlib.Path(CONFIG["lane"])
JOB = LANE / CONFIG["job"]


def refuse(message):
    print("REFUSED: " + message, flush=True)
    sys.exit(1)


def sha(path):
    with open(path, "rb") as handle:
        return hashlib.file_digest(handle, "sha256").hexdigest()


def finished_ok(job):
    path = LANE / job / "resource-receipt.json"
    if not path.is_file():
        return False
    r = json.loads(path.read_text())
    return (r.get("admitted") is True and r.get("exit") == 0 and bool(r.get("finished"))
            and r.get("inputs_sha256") == r.get("final_inputs_sha256")
            and not r.get("stopped_for_resource_limit") and "error" not in r)


def rows(path):
    return [line.split("\t") for line in pathlib.Path(path).read_text().splitlines() if line and not line.startswith("#")]


def first(table, key):
    return next(r[1] for r in table if r[0] == key)


def schedule(path):
    return [r for r in rows(path) if r and r[0] in ("chapter", "fps", "frames", "line")]


def findings(path):
    return len([line for line in pathlib.Path(path).read_text().splitlines() if line.strip()]) if pathlib.Path(path).is_file() else None


def run(argv, timeout):
    print("RUN " + " ".join(argv), flush=True)
    subprocess.run(argv, check=True, timeout=timeout)


def summary(name, data):
    data["scope"] = CONFIG["scope"]
    with (JOB / name).open("x") as handle:
        handle.write(json.dumps(data, indent=2) + "\n")


def timeline():
    out = JOB / "timeline-output"
    run(CONFIG["verify_argv"], 900)
    listed = subprocess.check_output([CONFIG["binary"], "chapters"], text=True).splitlines()
    if listed != CONFIG["chapters"]:
        refuse("the binary's chapter registry differs: " + " ".join(listed))
    run(CONFIG["argv"], 2400)
    seen, details = [], []
    for chapter in CONFIG["chapters"]:
        path = out / (chapter + ".timeline.tsv")
        table = rows(path)
        if first(table, "chapter") != chapter or first(table, "fps") != "60":
            refuse(chapter + ": the timeline is not this chapter at 60 fps")
        lines = [r for r in table if r[0] == "line"]
        for r in lines:
            if pathlib.Path(r[4]).resolve() != (pathlib.Path(CONFIG["narration"]) / (r[1] + ".wav")).resolve():
                refuse(r[1] + ": the line plays a file outside the bound narration")
        seen += [r[1] for r in lines]
        details.append({"chapter": chapter, "frames": int(first(table, "frames")), "cues": [r[1] for r in lines],
                        "timeline_sha256": sha(path)})
    if seen != CONFIG["cues"]:
        refuse("the cue placements differ from the script order")
    summary("timeline-summary.json", {"fps": 60, "chapters": details, "cues": len(seen),
                                      "total_frames": sum(x["frames"] for x in details)})


def preview():
    out = JOB / "preview"
    run(CONFIG["argv"], 14400)
    reference = json.loads((LANE / CONFIG["timeline_job"] / "timeline-summary.json").read_text())
    chapters = []
    for item in reference["chapters"]:
        chapter = item["chapter"]
        table = rows(out / (chapter + ".timeline.tsv"))
        if first(table, "fps") != "30" or [r[1] for r in table if r[0] == "line"] != item["cues"]:
            refuse(chapter + ": the preview's cue order differs from the 60 fps timeline")
        chapters.append({"chapter": chapter, "frames": int(first(table, "frames")), "mp4_sha256": sha(out / (chapter + ".mp4")),
                         "layout_findings": findings(out / (chapter + ".layout.tsv"))})
    summary("preview-summary.json", {"res": "960x540", "fps": 30, "chapters": chapters})


def score():
    run(CONFIG["argv"], 7200)
    qa = {}
    for chapter in CONFIG["chapters"]:
        path = JOB / "scored" / "stems" / "qa" / (chapter + ".json")
        if json.loads(path.read_text()).get("passed") is not True:
            refuse(chapter + ": score QA did not pass")
        qa[chapter] = sha(path)
    summary("score-summary.json", {"qa_sha256": qa})


def render():
    chapter = CONFIG["chapter"]
    work = pathlib.Path(CONFIG["render_work"])
    work.mkdir(exist_ok=True)
    if (work / (chapter + ".mp4")).exists():
        refuse(chapter + ".mp4 already exists in the render work folder")
    run(CONFIG["argv"], 43200)
    reference = LANE / CONFIG["timeline_job"] / "timeline-output" / (chapter + ".timeline.tsv")
    if schedule(reference) != schedule(work / (chapter + ".timeline.tsv")):
        refuse(chapter + ": the frame and cue schedule differs from the 60 fps timeline")
    summary("render-summary.json", {"chapter": chapter, "mp4_sha256": sha(work / (chapter + ".mp4")),
                                    "timeline_sha256": sha(work / (chapter + ".timeline.tsv")),
                                    "layout_findings": findings(work / (chapter + ".layout.tsv"))})


def master():
    work = pathlib.Path(CONFIG["render_work"])
    outputs = {"master": work / (CONFIG["topic"] + "_master.mp4"), "srt": work / (CONFIG["topic"] + ".en.srt"),
               "vtt": work / (CONFIG["topic"] + ".en.vtt")}
    for path in outputs.values():
        if path.exists():
            refuse(str(path) + " already exists")
    for chapter in CONFIG["render_jobs"]:
        if not (work / (chapter + ".mp4")).is_file():
            refuse(chapter + ".mp4 is missing")
    run(CONFIG["argv"], 14400)
    run(CONFIG["captions_argv"], 1200)
    summary("master-summary.json", {k: {"path": str(p), "sha256": sha(p)} for k, p in outputs.items()})


def main():
    receipt = JOB / "resource-receipt.json"
    if not receipt.is_file():
        refuse("launch only through the guard: " + str(receipt) + " is missing")
    started = json.loads(receipt.read_text())
    if started.get("command") != ["python3", "-I", "-B", CONFIG["script"]] or "exit" in started or started.get("admitted") is not True:
        refuse("the guard's receipt for this job does not show this script running")
    if os.environ.get("CARGO_TARGET_DIR") != CONFIG["target"]:
        refuse("CARGO_TARGET_DIR is not the guard's target")
    if sha(CONFIG["binary"]) != CONFIG["binary_sha256"]:
        refuse("the workspace binary moved")
    for job in CONFIG["after"]:
        if not finished_ok(job):
            refuse(job + " did not finish cleanly")
    {"timeline": timeline, "preview": preview, "score": score, "render": render, "master": master}[CONFIG["step"]]()
    print(CONFIG["success"], flush=True)


if __name__ == "__main__":
    main()
'''


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    parser.add_argument("command", choices=["check", "install", "build-script", "build-verify", "bind", "step", "verify-step"])
    parser.add_argument("--w", required=True)
    parser.add_argument("--ws")
    parser.add_argument("--attempt")
    parser.add_argument("--build")
    parser.add_argument("--audio")
    parser.add_argument("--binding")
    parser.add_argument("--series")
    parser.add_argument("--step", choices=STEPS)
    parser.add_argument("--chapter")
    parser.add_argument("--retry", type=int, choices=range(1, 10))
    for hidden in ("--receipts", "--target-root"):
        parser.add_argument(hidden, help="checks only: a scratch copy in place of the default")
    a = parser.parse_args(argv)
    two = re.compile(r"\d\d")
    if a.command != "check" and not a.ws:
        parser.error(f"{a.command} needs --ws")
    if a.command in ("build-script", "build-verify", "bind") and not two.fullmatch(a.attempt or ""):
        parser.error(f"{a.command} needs --attempt NN")
    if a.command == "bind" and not (two.fullmatch(a.build or "") and a.audio):
        parser.error("bind needs --build NN and --audio O")
    if a.command in ("step", "verify-step") and not (two.fullmatch(a.binding or "") and two.fullmatch(a.series or "") and a.step):
        parser.error(f"{a.command} needs --binding NN, --series NN and --step")
    try:
        c = Ctx(a)
        if a.command == "check":
            result = {"check": "PASS", **check(c)}
        elif a.command == "install":
            result = {"install": "PASS", **install(c)}
        elif a.command == "build-script":
            result = {"build_script": "WRITTEN", **build_script(c, a.attempt)}
        elif a.command == "build-verify":
            result = {"build_verify": "PASS", **build_verify(c, a.attempt)}
        elif a.command == "bind":
            result = {"bind": "WRITTEN", **bind(c, a.build, a.audio, a.attempt)}
        elif a.command == "step":
            result = {"step": "WRITTEN", **write_step(c, a.binding, a.series, a.step, a.chapter, a.retry)}
        else:
            result = {"verify_step": "PASS", **verify_step(c, a.binding, a.series, a.step, a.chapter, a.retry)}
    except Refused as refusal:
        print(f"REFUSED: {refusal}")
        return 1
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    sys.exit(main())
