#!/usr/bin/env python3
"""Logarithm timeline and score on mac-mini-max, for root's remote lane (bead 87do.63).

A thin binding on the sealed prepare_final_picture_pipeline.py, loaded by hash; no sealed file is edited. Its gates,
tables, stems, QA and comparisons run unchanged, as does version 4's job template. Only these change, in memory:
- the lane, target, binary, narration, binding and comparison paths, to root's remote ones;
- the job stem, so the jobs are logarithm-remote-timeline01 and -score01;
- gates(), which names the build and assembly scripts and manifests, and allows the one build row the rebind rewrites.

Run from anywhere; the controller runs it. Commands are the sealed helper's: seal, poll, ready, bind, step, verify-step,
compare. seal here checks this bundle and the sealed one. Technical gates only: nothing here was heard, viewed or accepted.
"""
import hashlib
import importlib.util
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve()
BUNDLE = HERE.parent
REPO = HERE.parents[5]
SEALED = REPO / "docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-final-picture-pipeline01"
SEALED_SHA = "797b4394d407dcf146de947cd3c3f1837e83e829b392eebcc4bf23f957143dad"
SEALED_SUMS_SHA = "3df900b02cb8f38d6df474e5808a37b6161068055204595a0338da134f67cb85"


def load_sealed():
    path = SEALED / "prepare_final_picture_pipeline.py"
    found = hashlib.sha256(path.read_bytes()).hexdigest()
    if found != SEALED_SHA:
        raise SystemExit(f"REFUSED: the sealed pipeline helper is {found[:12]}, expected {SEALED_SHA[:12]}")
    spec = importlib.util.spec_from_file_location("prepare_final_picture_pipeline", path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


M = load_sealed()
H, N = M.H, M.N
need, sha, tree, pinned = M.need, M.sha, M.tree, M.pinned
O = M.EL / "remote-mac-mini-max01"
JOBS = O / "jobs"
OLD_LANE = M.LANE
BUILD_SCRIPT, ASSEMBLE_SCRIPT = O / "build01.sh", BUNDLE / "remote_assemble01.py"
BUILD_INPUTS, ASSEMBLE_INPUTS = O / "build-inputs01.json", O / "assemble-inputs01.json"
UNITS = M.CRATE / "narration_units.json"

H.STAGE = "logarithm-remote"
M.LANE = JOBS
M.BINARY = O / "logarithm_mysteries-release01"
M.BUILD, M.ASSEMBLE = "logarithm-remote-build01", "logarithm-remote-assemble01"
M.NARRATION = JOBS / M.ASSEMBLE / "narration01"
M.BINDING = O / "remote-binding01.json"
M.COMPARISON = {"timeline": O / "schedule-comparison01.json", "score": O / "score-comparison01.json"}
# Stage09's jobs stay in the old lane. A joined absolute path replaces the left side, so M.LANE / PRIOR[step] finds them.
M.PRIOR = {step: str(OLD_LANE / job) for step, job in M.PRIOR.items()}
M.C.lane, M.C.jobs, M.C.target = JOBS, O / "job-scripts", O / "target"
M.full_tables.__defaults__ = (M.NARRATION,)  # the default was fixed when the sealed helper loaded


def frozen_build(receipt, required):
    """The sealed frozen() for the build, with one exception: root's manifest binds the unit plan, which the rebind must
    rewrite. That row must instead equal the approved bytes the rebind preserved beside the workspace."""
    need(sha(BUILD_INPUTS) == receipt["inputs_sha256"], f"{M.BUILD}: {BUILD_INPUTS.name} is not the manifest its guard froze")
    rows = {row["path"]: row["sha256"] for row in json.loads(BUILD_INPUTS.read_text())["files"]}
    need(rows.get(str(UNITS)) == sha(M.S / "narration_units-before-rebind01.json"),
         f"{M.BUILD}: the unit plan it froze is not the one the rebind preserved")
    moved = [p for p, h in rows.items() if p != str(UNITS) and (not Path(p).is_file() or sha(p) != h)]
    need(not moved, f"{M.BUILD}: its frozen inputs moved after it ran: {moved[:3]}")
    absent = [str(p) for p in required if str(p) not in rows]
    need(not absent, f"{M.BUILD}: {BUILD_INPUTS.name} does not bind {absent[:3]}")


def gates(a):
    """The sealed gates() with the remote scripts and manifests. Returns the binding's body and writes nothing."""
    for name, digest in H.TOOLS.items():
        pinned(M.TOOLS / name, digest, f"workspace tools/{name}")
    pinned(REPO / N.GUARD_REL, N.TOOL_PIN[N.GUARD_REL], "the resource guard")
    pinned(REPO / N.GUARD_SOURCE_REL, N.TOOL_PIN[N.GUARD_SOURCE_REL], "the guard's source")
    build_receipt = M.finished(M.BUILD, BUILD_SCRIPT)
    assemble_receipt = M.finished(M.ASSEMBLE, ASSEMBLE_SCRIPT)
    integrated = M.integration_gate(a.integration_sha256)
    cues = M.cues_of()
    crate = tree(M.CRATE)
    bound, spans = M.S / "holds-bound01.json", M.S / "spans01.json"
    frozen_build(build_receipt, [BUILD_SCRIPT, *M.build_sources(crate)])
    M.frozen(M.ASSEMBLE, ASSEMBLE_INPUTS, assemble_receipt,
             [ASSEMBLE_SCRIPT, M.CRATE / "script.tsv", UNITS, spans, M.TOOLS / "coherent_narration.py"])
    folder = M.LANE / M.BUILD
    need(M.BINARY.is_file(), f"no binary at {M.BINARY}")
    need(H.mentions(M.BINARY.read_bytes(), M.CRATE) > 0, "the binary was not compiled for the repaired workspace's crate")
    listed = (folder / "chapters.txt").read_text().splitlines() if (folder / "chapters.txt").is_file() else []
    need(listed == M.CHAPTERS, f"the build's chapter registry is not the 13 chapters: {listed}")
    M.live([str(M.BINARY), "chapters"], (folder / "chapters.txt").read_bytes(), "the binary's chapters")
    audio = M.audio_gate(cues)
    hold_facts = M.holds_gate(a.holds_sha256, a.integration_sha256, M.NARRATION)
    facts = M.facts_gate(folder)
    checks = M.tool_checks()
    for step, job in M.PRIOR.items():
        need(H.receipt_ok(json.loads((M.LANE / job / "resource-receipt.json").read_text())), f"the prior {step} {job} did not pass")
    files = {str(p): sha(p) for p in (spans, bound, BUILD_INPUTS, ASSEMBLE_INPUTS, BUILD_SCRIPT, ASSEMBLE_SCRIPT, M.HERE,
                                      M.HERE.parent / "SHA256SUMS", HERE, BUNDLE / "SHA256SUMS",
                                      O / "guard01.py", O / "guard-pins01.json")}
    for job in (M.BUILD, M.ASSEMBLE):
        files.update({str(M.LANE / job / rel): h for rel, h in tree(M.LANE / job).items()})
    files.update({str(M.NARRATION / rel): h for rel, h in tree(M.NARRATION).items()})
    return {"schema": "logarithm-remote-binding-v1", "helper_sha256": sha(M.HERE), "render_helper_sha256": M.V4_SHA,
            "remote_helper_sha256": sha(HERE), "ws": str(M.WS), "crate": crate,
            "approved": {"integration_sha256": a.integration_sha256, "holds_bound_sha256": a.holds_sha256,
                         "integrated_rows": integrated},
            "build": {"job": M.BUILD, "binary": str(M.BINARY), "binary_sha256": sha(M.BINARY)},
            "audio": {"narration": str(M.NARRATION), **audio}, "voice": H.VOICE, "cues": len(cues),
            "chapters": len(M.CHAPTERS), "facts_rows": facts, "direction": hold_facts, "checks": checks, "files": files,
            "claim": "Technical binding only. Nothing here was heard, rendered or accepted."}


def seal(digest):
    """This bundle against the hash root passes in, the sealed bundle against its own pin, and two facts that would
    otherwise refuse only after the assembly: root's manifest spells this tree's paths, and stage09's jobs are here."""
    sealed = SEALED / "SHA256SUMS"
    need(sealed.is_file() and sha(sealed) == SEALED_SUMS_SHA, "the sealed pipeline bundle's SHA256SUMS moved")
    for folder, sums in ((SEALED, sealed), (BUNDLE, BUNDLE / "SHA256SUMS")):
        need(sums.is_file(), f"no {sums}")
        for line in sums.read_text().splitlines():
            expected, name = line.split(maxsplit=1)
            need(sha(folder / name.lstrip("*")) == expected, f"{folder.name}/{name} differs from its seal")
    need(sha(BUNDLE / "SHA256SUMS") == digest, f"this bundle's SHA256SUMS is not the seal {digest[:12]}")
    rows = {row["path"] for row in json.loads(BUILD_INPUTS.read_text())["files"]}
    need(str(BUILD_SCRIPT) in rows, f"{BUILD_INPUTS.name} does not name {BUILD_SCRIPT}: this tree's path differs from root's")
    for step, output in (("timeline", "timeline-output"), ("score", "scored/stems")):
        prior = M.LANE / M.PRIOR[step]
        need((prior / "resource-receipt.json").is_file() and (prior / output).is_dir(),
             f"stage09's {step} job has not been transferred: {prior}")
    return {"seal": "PASS"}


M.gates, M.seal = gates, seal

if __name__ == "__main__":
    sys.exit(M.main())
