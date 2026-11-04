#!/usr/bin/env python3
"""Rehearse root's whole non-Cargo path on a temp copy of the repo. Run from the repo root:

    python3 -I -B rehearse_integration.py

It runs the real integrate_stage06.py, then the real tools/coherent_narration.py and
tools/narration_delivery.py CLIs on the installed plan, with all 80 cues.

The WAVs here are STAND-INS: near-silent samples of arbitrary length, labelled in their receipts.
No voice, no TTS. A pass means the plan, the receipt shape and the dispositions get through the real
assembler and verifier. It says nothing about any take, any timing or how anything sounds. The Rust
loader and the crate are not compiled or run here; the phrase file goes through this folder's Python
mirror of the loader's rules. The real tree is read, hashed before and after, and never written.
"""
import importlib.util
import json
import os
import re
import shutil
import stat
import subprocess
import sys
import tempfile
import wave
from pathlib import Path

sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location("stage06_sandbox", HERE / "sandbox.py")
sandbox = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = sandbox
spec.loader.exec_module(sandbox)
pf, sha = sandbox.pf, sandbox.sha
PY = [sys.executable, "-I", "-B"]
CRATE = pf.REL.as_posix()
SCRIPT, UNITS = f"{CRATE}/script.tsv", f"{CRATE}/narration_units.json"
KEYS = ["00_ratio_ruler", "01_building_ruler", "02_slide_rule", "03_folded_decade", "04_undoing_growth", "05_complex_turns", "06_lifting_loops", "07_branch_cut", "08_product_rule", "09_matrix_logs", "10_doing_nothing", "11_unreachable", "12_what_it_forgets"]
results = []


def note(label, good, detail=""):
    results.append((label, good))
    print(f"{'pass' if good else 'MISS'} {label}" + (f": {detail}" if detail else ""))


def run(root, *argv):
    return subprocess.run([*PY, *map(str, argv)], cwd=root, capture_output=True, text=True)


def integrate(root, *extra):
    return run(root, HERE / "integrate_stage06.py", "--repo", root, *extra)


def preflight(root, *extra):
    done = run(root, HERE / "preflight.py", "--repo", root, "--quiet", *extra)
    return done.returncode, re.findall(r"^FAIL (\S+)", done.stdout, re.M), done


def tree_hash(path):
    rows = []
    for p in sorted(Path(path).rglob("*")):
        if p.is_file() and "target" not in p.parts:
            rows.append(f"{p.relative_to(path)} {sha(p.read_bytes())}")
    return sha("\n".join(rows).encode())


def stand_in_narration(root, work, rate=24000, tamper=None, drop_disposition=False):
    """Stand-in takes and a span manifest in the shape root's UI lane writes for the assembler."""
    coherent = sandbox_tool(root)
    script, units = coherent.read_units(root / SCRIPT, root / UNITS, require_directions=True)
    work.mkdir(parents=True)
    records, spans = [], []
    for at, unit in enumerate(units):
        cue = unit["line_ids"][0]
        count = len(coherent.sentences(script[cue]))
        frames = 2400 * count
        wav = work / f"{cue}.wav"
        with wave.open(str(wav), "wb") as w:
            w.setparams((1, 2, rate, 0, "NONE", "not compressed"))
            # One sample differs per cue, so no two stand-ins share a hash.
            w.writeframes((at + 1).to_bytes(2, "little", signed=True) + b"\0\0" * (frames - 1))
        source = work / f"{cue}.stand-in"
        source.write_bytes(f"stand-in for {cue}: not audio, not a provider file".encode())
        text = script[cue]
        if tamper == cue:
            text += " And more."
        records.append({"cue": cue, "text": text, "wav": str(wav), "wav_sha256": sha(wav.read_bytes()), "samples": frames, "rate": rate, "source": str(source), "source_sha256": sha(source.read_bytes())})
        spans.append({"id": unit["id"], "wav": str(wav), "sha256": sha(wav.read_bytes()), "sentence_ends": [2400 * (i + 1) for i in range(count)], "gap_after_samples": [0] * count})
    receipt = work / "receipt.json"
    receipt.write_text(json.dumps({"source": "ElevenLabs UI, Oliver", "records": records, "claim": "STAND-IN receipt for a rehearsal. No provider, no voice."}))
    for row in spans:
        row["generation_receipt"] = str(receipt)
    dispositions = json.loads((root / pf.STAGE / "bound-plan/direction-dispositions-proposed.json").read_bytes())["direction_dispositions"]
    if drop_disposition:
        dispositions = dispositions[1:]
    manifest = {"schema_version": 1, "units_sha256": sha((root / UNITS).read_bytes()), "units": spans,
                "boundary_review": {"reviewer": "rehearsal", "basis": "stand-in samples; no boundary exists", "listening": "unreviewed"},
                "direction_dispositions": dispositions}
    path = work / "spans.json"
    path.write_text(json.dumps(manifest))
    return path


def sandbox_tool(root):
    spec = importlib.util.spec_from_file_location("rehearse_coherent", root / "tools/coherent_narration.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module


def assemble(root, spans, out):
    return run(root, "tools/coherent_narration.py", "assemble", "--script", SCRIPT, "--units", UNITS, "--require-directions", "--spans", spans, "--out", out)


def main():
    real = Path.cwd()
    real_before = (tree_hash(real / pf.REL), tree_hash(real / pf.STAGE))
    tmp = Path(tempfile.mkdtemp(prefix="stage06-rehearsal-"))
    try:
        base = tmp / "base"
        print(f"sandbox: {sandbox.build_base(real, base)} files copied")

        # ---------------------------------------------------------- the positive path
        root = tmp / "repo"
        shutil.copytree(base, root)
        before = {rel: sha((root / CRATE / rel).read_bytes()) for rel, *_ in load_files() if (root / CRATE / rel).exists()}
        done = integrate(root, "--check")
        note("integrate --check on an untouched copy", done.returncode == 0, done.stdout.strip())
        out_rel = "records/integration"
        done = integrate(root, "--apply", "--out", out_rel)
        note("integrate --apply installs seven files", done.returncode == 0, done.stdout.strip() + done.stderr.strip()[-300:])
        kept = all(sha((root / out_rel / "before" / rel).read_bytes()) == digest for rel, digest in before.items())
        note("every replaced file is kept, byte for byte, under before/", kept and len(before) == 6, f"{len(before)} kept; map13 is new, so it has none")
        record = (root / out_rel / "integration.json").read_text()
        note("the records hold no absolute home path", ("/" + "Users/") not in record and ("/" + "Users/") not in (root / out_rel / "before-manifest.json").read_text())
        done = run(root, "tools/coherent_narration.py", "check", "--script", SCRIPT, "--units", UNITS, "--require-directions")
        note("real CLI: coherent_narration check --require-directions", done.returncode == 0, (done.stdout.strip() or done.stderr.strip())[:200])
        code, fails, done = preflight(root, "--mode", "canonical")
        note("preflight --mode canonical, pins on, after install", code == 0 and not fails, done.stdout.strip().splitlines()[-1])

        spans = stand_in_narration(root, tmp / "takes")
        out = tmp / "narration"
        done = assemble(root, spans, out)
        note("real CLI: assemble, 80 stand-in takes, 15 native_visual_hold dispositions", done.returncode == 0, (done.stdout.strip() or done.stderr.strip()).replace(str(tmp), "<tmp>")[:300])
        done = run(root, "tools/coherent_narration.py", "verify", out, "--script", SCRIPT, "--units", UNITS, "--require-directions")
        note("real CLI: verify --require-directions rebuilds every cue sample for sample", done.returncode == 0, (done.stdout.strip() or done.stderr.strip())[:300])
        keys = [x for k in KEYS for x in ("--chapter-key", k)]
        done = run(root, "tools/narration_delivery.py", "--topic", CRATE, "--narration", out, "--consumer", "render", "--gain-db", "0", *keys)
        reply = json.loads(done.stdout) if done.returncode == 0 else {}
        good = reply.get("verified_cues") == 80 and reply.get("selected_cues") == 80 and reply.get("plan_mode") == "canonical" and reply.get("listening_acceptance") is False
        note("real CLI: narration_delivery --consumer render accepts the whole film", good, done.stderr.strip()[:300] or f"verified {reply.get('verified_cues')}, plan_mode {reply.get('plan_mode')}, listening_acceptance {reply.get('listening_acceptance')}")

        assembly = json.loads((out / "assembly.json").read_bytes())
        template = json.loads((root / pf.STAGE / "bound-plan/phrase-timings-unmeasured-template.json").read_bytes())
        script = pf.read_script(root / SCRIPT)
        timing = pf.fill_template(template, script)
        for cue, entry in timing["cues"].items():
            entry["wav_sha256"] = assembly["outputs"][cue]["sha256"]
        destination = out / "phrase-timings.json"
        destination.write_text(json.dumps(timing))
        code, fails, done = preflight(root, "--mode", "canonical", "--phrase-timings", destination, "--narration", out)
        note("phrase gate at <narration>/phrase-timings.json, bound to the assembled WAVs", code == 0 and not fails, done.stdout.strip().splitlines()[-1])
        entry = timing["cues"]["11_20"]
        entry["marks"] = entry["marks"][:1]
        short = tmp / "short.json"
        short.write_text(json.dumps(timing))
        code, fails, done = preflight(root, "--mode", "canonical", "--phrase-timings", short, "--narration", out)
        note("phrase gate refuses a file missing one anchor, which the kit would load", code == 1 and fails == ["GATE.phrase_timings"], str(fails))

        # ---------------------------------------------------------- refusals, each on its own copy
        done = integrate(root, "--apply", "--out", "records/again")
        note("a second --apply is refused", done.returncode == 1 and not (root / "records/again").exists(), done.stdout.strip().splitlines()[0][:120])

        def fresh(name):
            path = tmp / name
            shutil.copytree(base, path)
            return path

        other = fresh("moved")
        ratio = other / CRATE / "src/chapters/ratio_ruler.rs"
        ratio.write_bytes(ratio.read_bytes() + b"// canonical moved\n")
        snapshot = tree_hash(other / CRATE)
        done = integrate(other, "--apply", "--out", "records/x")
        note("--apply refuses when canonical moved under stage06, writing nothing", done.returncode == 1 and tree_hash(other / CRATE) == snapshot and not (other / "records").exists(), done.stdout.strip().splitlines()[0][:130])

        other = fresh("exists")
        (other / "records/x").mkdir(parents=True)
        snapshot = tree_hash(other / CRATE)
        done = integrate(other, "--apply", "--out", "records/x")
        note("--apply refuses an existing record folder", done.returncode == 1 and tree_hash(other / CRATE) == snapshot)

        other = fresh("job")
        job = other / pf.RELEASE / "a-live-job"
        job.mkdir()
        (job / "resource-receipt.json").write_text(json.dumps({"started": "now", "pid": os.getpid()}))
        snapshot = tree_hash(other / CRATE)
        done = integrate(other, "--apply", "--out", "records/x")
        note("--apply refuses while a guarded job looks live", done.returncode == 1 and tree_hash(other / CRATE) == snapshot, done.stdout.strip().splitlines()[0][:110])
        done = integrate(other, "--apply", "--out", "records/x", "--open-jobs-checked")
        note("--open-jobs-checked is root's explicit override", done.returncode == 0)

        def installed(name):
            path = tmp / name
            shutil.copytree(base, path)
            assert integrate(path, "--apply", "--out", "records/i").returncode == 0
            return path

        other = installed("staleunits")
        shutil.copyfile(other / pf.STAGE / pf.REL / "narration_units.json", other / UNITS)
        done = run(other, "tools/coherent_narration.py", "check", "--script", SCRIPT, "--units", UNITS, "--require-directions")
        code, fails, _ = preflight(other, "--mode", "canonical")
        note("copying stage06's stale units is caught by the real CLI and the preflight", done.returncode != 0 and code == 1 and "ASM.installed_units_are_the_plan" in fails, f"{len(fails)} FAIL")

        other = installed("orig")
        shutil.copyfile(other / pf.STAGE / pf.REL / "src/chapters/doing_nothing.rs.orig", other / CRATE / "src/chapters/doing_nothing.rs.orig")
        code, fails, _ = preflight(other, "--mode", "canonical")
        note("installing the stray .orig is caught", code == 1 and "ASM.orig_not_installed" in fails, str(fails[:3]))

        other = installed("partial")
        shutil.copyfile(other / "records/i/before/src/chapters/complex_turns.rs", other / CRATE / "src/chapters/complex_turns.rs")
        code, fails, _ = preflight(other, "--mode", "canonical")
        note("a half-reverted install is caught by the holds", code == 1 and "HOLD.05_1.source_hash" in fails and "ASM.installed_equals_stage" in fails, f"{len(fails)} FAIL")

        other = installed("wrongtext")
        done = assemble(other, stand_in_narration(other, tmp / "takes-wrong", tamper="05_20"), tmp / "n-wrong")
        note("real assembler refuses a take whose receipt text differs from the cue", done.returncode != 0 and "external source, text or decoded audio changed" in done.stderr, done.stderr.strip().splitlines()[-1][:110])

        other = installed("dropped")
        done = assemble(other, stand_in_narration(other, tmp / "takes-drop", drop_disposition=True), tmp / "n-drop")
        note("real assembler refuses a missing disposition", done.returncode != 0 and "explicit assembly disposition" in done.stderr, done.stderr.strip().splitlines()[-1][:110])

        other = installed("rate")
        done = assemble(other, stand_in_narration(other, tmp / "takes-48", rate=48000), tmp / "n-48")
        note("real assembler refuses 48 kHz: the provider's WAV needs a bound 24 kHz decode first", done.returncode != 0 and "24 kHz" in done.stderr, done.stderr.strip().splitlines()[-1][:110])

        # ---------------------------------------------------------- post_build_check, stand-in binary
        def stub(name, facts_extra="", chapters=KEYS):
            path = tmp / name
            oracle = (root / CRATE / "oracle.py").resolve()
            listing = "\\n".join(chapters)
            path.write_text(f'#!/bin/sh\nif [ "$1" = facts ]; then "{sys.executable}" -I -B "{oracle}"; printf "{facts_extra}"; else printf "{listing}\\n"; fi\n')
            path.chmod(path.stat().st_mode | stat.S_IXUSR)
            return path

        check = lambda binary: run(root, HERE / "post_build_check.py", "--repo", root, "--binary", binary)
        done = check(stub("good"))
        note("post_build_check passes a stand-in binary that agrees", done.returncode == 0, done.stdout.strip().splitlines()[1] if done.returncode == 0 else done.stderr[-200:])
        done = check(stub("extra", facts_extra="made.up\\t1\\n"))
        note("post_build_check refuses a fact the oracle lacks", done.returncode != 0 and "facts differ" in done.stderr, done.stderr.strip()[:110])
        done = check(stub("order", chapters=KEYS[::-1]))
        note("post_build_check refuses the wrong chapter list", done.returncode != 0 and "lists" in done.stderr)
        done = subprocess.run(["sh", "-n", str(HERE / "build_stage06.sh")], capture_output=True, text=True)
        note("build_stage06.sh parses (sh -n only; it has not been run)", done.returncode == 0, done.stderr.strip())
    finally:
        shutil.rmtree(tmp, ignore_errors=True)
    real_after = (tree_hash(real / pf.REL), tree_hash(real / pf.STAGE))
    note("the real canonical crate and stage06 are unchanged", real_before == real_after, real_after[0][:16])
    miss = [label for label, good in results if not good]
    print(f"{len(results) - len(miss)} of {len(results)} steps behaved; {len(miss)} did not: {miss}")
    return 1 if miss else 0


def load_files():
    spec = importlib.util.spec_from_file_location("stage06_integrate", HERE / "integrate_stage06.py")
    module = importlib.util.module_from_spec(spec)
    sys.modules[spec.name] = module
    spec.loader.exec_module(module)
    return module.FILES


if __name__ == "__main__":
    sys.exit(main())
