#!/usr/bin/env python3
"""Hold the workspace crate graph to the plan's §19 DAG.

Compares `cargo metadata`'s workspace-internal dependency edges against the
expected adjacency below (the single in-repo source of truth for the crate
map). Fails on ANY drift: a missing crate, an extra crate, a missing edge,
an extra edge, or an edge that points upward in the layering. Run by
scripts/check.sh and therefore by CI.

Stdlib only; requires cargo on PATH.
"""

import json
import subprocess
import sys

# §19 crate map. Order is the layering: a crate may depend only on crates
# that appear EARLIER in this dict (strictly-downward edges, cycle-free by
# construction). Keep in lockstep with COMPREHENSIVE_PLAN §19.
EXPECTED: dict[str, set[str]] = {
    # fmn-dmath is the root, and that is ADR-0014's ruling rather than an
    # accident: ADR-0010 makes "fmn-dmath owns every transcendental on the
    # certified path" a binding property, and a crate cannot honour it if the
    # layering puts it above the funnel. fmn-dmath consumes nothing from
    # fmn-core, so the edge it used to declare was vestigial.
    "fmn-dmath": set(),
    "fmn-core": {"fmn-dmath"},  # dmath: color transfer + rate functions (ADR-0014)
    "fmn-hash": set(),  # Canonical bytes/digests are independent of scene math.
    "fmn-config": {"fmn-core", "fmn-hash"},
    "fmn-platform": {"fmn-core"},
    "fmn-frame": {"fmn-core", "fmn-dmath"},  # dmath: deterministic transfer functions (D-17, fm-a25)
    "fmn-codec": {"fmn-core", "fmn-frame", "fmn-hash"},
    "fmn-cache": {"fmn-hash", "fmn-platform"},
    "fmn-geom": {"fmn-core", "fmn-dmath"},
    "fmn-mobject": {"fmn-core", "fmn-dmath", "fmn-geom", "fmn-hash"},  # dmath: tracker exp/ln (ADR-0014)
    # hash: canonical Timeline serialization — the Studio-scrubbing and
    # WASM-player substrate (§9.4, fm-hfe)
    "fmn-anim": {"fmn-core", "fmn-dmath", "fmn-hash", "fmn-mobject"},
    # codec: §10.6 texture/image decoding. It is a Substrate crate below
    # Lumen; exotic formats remain an upper-layer ffmpeg capability.
    "fmn-render": {"fmn-core", "fmn-dmath", "fmn-geom", "fmn-mobject", "fmn-frame", "fmn-codec", "fmn-hash", "fmn-cache"},
    "fmn-text": {"fmn-core", "fmn-geom", "fmn-mobject"},
    "fmn-tex": {"fmn-core", "fmn-config", "fmn-mobject", "fmn-text", "fmn-cache", "fmn-platform"},  # platform: capability-bound native typesetting sessions
    "fmn-library": {"fmn-core", "fmn-dmath", "fmn-geom", "fmn-mobject", "fmn-anim", "fmn-text", "fmn-tex", "fmn-codec"},  # dmath: tip/arc trigonometry (ADR-0014); codec: ImageMobject decode (§10.6, fm-2u6)
    "fmn-scene": {"fmn-core", "fmn-config", "fmn-platform", "fmn-mobject", "fmn-anim", "fmn-render", "fmn-hash"},  # hash: journal serialization + digests (§13.4, fm-y7u)
    # hash: canonical supervisor/worker IPC; cache: supervisor-owned
    # checkpoints and journals survive scene-worker replacement (§13.3,
    # D-14, fm-39s).
    "fmn-studio": {
        "fmn-core",
        "fmn-hash",
        "fmn-platform",
        "fmn-frame",
        "fmn-codec",
        "fmn-cache",
        "fmn-render",
        "fmn-scene",
    },
    # dmath: the named windowed-sinc resampler and dB gain conversion
    # affect certified WAV bits (ADR-0014).
    "fmn-output": {
        "fmn-core",
        "fmn-dmath",
        "fmn-hash",
        "fmn-platform",
        "fmn-frame",
        "fmn-codec",
        "fmn-cache",
    },
    "fmn-runtime": {"fmn-core", "fmn-platform"},
    # The native Rust front door (§15.1): a leaf facade over the real
    # subsystem APIs, never a second scene loop or animation engine.
    "fmn": {
        "fmn-core",
        "fmn-config",
        "fmn-platform",
        "fmn-geom",
        "fmn-mobject",
        "fmn-anim",
        "fmn-text",
        "fmn-tex",
        "fmn-library",
        "fmn-scene",
        "fmn-render",
        "fmn-output",
        "fmn-frame",
        "fmn-codec",
        "fmn-runtime",
    },
    # The standalone composition root consumes the native facade, Lumen,
    # frame conversion, codecs, and Reel directly; it does not duplicate any
    # subsystem implementation (fm-ffj.67, plan sections 13.6 and 15.1).
    # hash: the certified build identity digests its sources (ADR-0025,
    # fm-certified-closure-integrity-4fei).
    "fmn-cli": {
        "fmn-core",
        "fmn-config",
        "fmn-hash",
        "fmn-cache",
        "fmn-frame",
        "fmn-codec",
        "fmn-platform",
        "fmn-runtime",
        "fmn-render",
        "fmn-scene",
        "fmn-studio",
        "fmn-output",
        "fmn-library",
        "fmn",
    },
    # dmath: certified scene-corpus callbacks compute pixel-reaching graph
    # geometry through the sovereign transcendental funnel (ADR-0014,
    # fm-t1v.1). frame/runtime: the PG-5 producer consumes the real certified
    # raw-frame and scheduler surfaces rather than duplicating them
    # (fm-inr.3.2). codec: the Look Gallery's canonical panels route through
    # the owned deterministic PNG encoder (fm-qtd). cache/platform/text/tex:
    # the PG-7 producer measures the real
    # Scribe cold, cached, and 10k-glyph workloads through the governed
    # host/cache capabilities rather than a synthetic duplicate (§17.2,
    # fm-inr.2.2).
    "fmn-conformance": {
        "fmn-core",
        "fmn-dmath",
        "fmn-hash",
        "fmn-geom",
        "fmn-mobject",
        "fmn-anim",
        "fmn-render",
        "fmn-frame",
        "fmn-codec",
        "fmn-library",
        "fmn-scene",
        "fmn-output",
        "fmn-runtime",
        "fmn-cache",
        "fmn-platform",
        "fmn-text",
        "fmn-tex",
        "fmn",
    },
    # The optional CPython portal is its own composition root. It consumes the
    # same retained Lumen renderer and native Reel sinks as the standalone CLI,
    # including the optional governed ffmpeg boundary. ADR-0017 forbids the
    # standalone CLI from consuming or launching this host-CPython front door.
    "fmn-python": {
        "fmn-core",
        "fmn-hash",
        "fmn-config",
        "fmn-platform",
        "fmn-frame",
        "fmn-codec",
        "fmn-mobject",
        "fmn-anim",
        "fmn-render",
        "fmn-library",
        "fmn-scene",
        "fmn-output",
        "fmn-runtime",
        # Host-CPython Studio front door; no Python enters the native CLI.
        "fmn-studio",
    },
    # W5 tier-1 wasm surface (fm-l97, §10.7): the browser leaf. dmath is a
    # direct dependency because scene construction evaluates parametric
    # transcendentals and ADR-0014 forbids routing around the sovereign
    # funnel; frame owns the certified Rgba16F→Rgba8 transfer the canvas
    # path consumes. Neither is reimplemented here. hash: the tier-2 player
    # (fm-oee) decodes the FMTL/1 timeline bundle, a §6.7 canonical
    # container — the format layer itself, never a reimplemented parser.
    "fmn-wasm": {
        "fmn-core",
        "fmn-dmath",
        "fmn-geom",
        "fmn-mobject",
        "fmn-anim",
        "fmn-render",
        "fmn-scene",
        "fmn-frame",
        "fmn-hash",
    },
}

LAYER = {name: i for i, name in enumerate(EXPECTED)}
METADATA_TIMEOUT_SECONDS = 120


def main() -> int:
    try:
        completed = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            check=True,
            capture_output=True,
            text=True,
            timeout=METADATA_TIMEOUT_SECONDS,
        )
    except FileNotFoundError:
        print("crate-DAG check FAILED: cargo is not on PATH", file=sys.stderr)
        return 1
    except subprocess.TimeoutExpired:
        print(
            f"crate-DAG check FAILED: cargo metadata exceeded "
            f"{METADATA_TIMEOUT_SECONDS}s",
            file=sys.stderr,
        )
        return 1
    except subprocess.CalledProcessError as error:
        detail = error.stderr.strip() or f"exit status {error.returncode}"
        print(f"crate-DAG check FAILED: cargo metadata: {detail}", file=sys.stderr)
        return 1
    try:
        meta = json.loads(completed.stdout)
    except json.JSONDecodeError as error:
        print(
            f"crate-DAG check FAILED: cargo metadata emitted invalid JSON: {error}",
            file=sys.stderr,
        )
        return 1
    workspace_ids = set(meta["workspace_members"])
    actual: dict[str, set[str]] = {}
    spikes: set[str] = set()
    for pkg in meta["packages"]:
        if pkg["id"] not in workspace_ids:
            continue
        # §19's DAG governs the SHIPPED graph (ADR-0003: dev is a separate,
        # non-shipped tier). Dev-dependencies — e.g. a test suite borrowing
        # fmn-platform's VirtualFs/FakeClock doubles — are exempt from the
        # layering; the governed-closure allowlist still covers their
        # packages.
        deps = {
            d["name"]
            for d in pkg["dependencies"]
            if (d["name"] == "fmn" or d["name"].startswith("fmn-"))
            and d.get("kind") != "dev"
        }
        # G0 spikes (spikes/, fmn-spike-*) are sanctioned prototype crates
        # outside the §19 map (§20.1). They are exempt from the map itself,
        # but no §19 crate may ever depend on one (checked below).
        if pkg["name"].startswith("fmn-spike-"):
            spikes.add(pkg["name"])
            continue
        actual[pkg["name"]] = deps

    errors: list[str] = []
    for name, deps in sorted(actual.items()):
        for dep in sorted(deps & spikes):
            errors.append(f"{name}: production crate depends on spike {dep}")
    for missing in EXPECTED.keys() - actual.keys():
        errors.append(f"crate missing from workspace: {missing}")
    for extra in actual.keys() - EXPECTED.keys():
        errors.append(f"crate not in plan §19: {extra}")

    for name in sorted(EXPECTED.keys() & actual.keys(), key=LAYER.__getitem__):
        want, got = EXPECTED[name], actual[name]
        for e in sorted(want - got):
            errors.append(f"{name}: missing declared dependency on {e}")
        for e in sorted(got - want):
            errors.append(f"{name}: undeclared-in-plan dependency on {e}")
        for e in sorted(got & want):
            if LAYER[e] >= LAYER[name]:
                errors.append(f"{name}: edge to {e} points upward in the layering")

    if errors:
        print("crate-DAG check FAILED:", file=sys.stderr)
        for e in errors:
            print(f"  - {e}", file=sys.stderr)
        return 1
    print(f"crate-DAG check OK: {len(actual)} crates match plan §19, all edges downward")
    return 0


if __name__ == "__main__":
    sys.exit(main())
