#!/usr/bin/env python3
"""Does the budget patch's new public surface trip anything at the pinned revision? Text scan only.

    python3 -I -B surface.py
"""
import re
from pathlib import Path

PIN = Path.home() / ".cargo/git/checkouts/franken_manim-6641d0e09642f63e/a399d62"
WORDS = re.compile(r"RenderOptions|NativeFramePipeline|NativeFrameStages|RenderPlanLimits|plan_limits|max_resident_bytes")


def rust_files():
    return [p for p in (PIN / "crates").rglob("*.rs") if "/target/" not in str(p)]


for name in ("API_SCHEMA.tsv", "API_OVERLAY.tsv"):
    text = (PIN / name).read_text(errors="replace")
    first = text.splitlines()[0][:70] if text else ""
    print(f"{name}: {len(WORDS.findall(text))} mentions of the facade's render types; {text.count(chr(10))} rows; the first column is manim's own API: {'manimlib' in text}")

literal = re.compile(r"\bRenderOptions\s*\{")
sites = []
for path in rust_files():
    for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
        stripped = line.strip()
        if literal.search(line) and not stripped.startswith(("pub struct", "impl", "//", "fn ", "pub fn ", "-> ")) and ") -> RenderOptions {" not in line:
            sites.append(f"{path.relative_to(PIN)}:{number}: {stripped[:70]}")
print(f"struct literals of RenderOptions, which a new field would break: {len(sites)}")
for site in sites:
    print(" ", site)

for needle in ("NativeFrameStages::new", "NativeFramePipeline::new"):
    found = sorted(str(p.relative_to(PIN)) for p in rust_files() if needle in p.read_text(errors="replace"))
    print(f"{needle}: {found}")

source = (PIN / "crates/fmn-cli/src/offline_cpu.rs").read_text()
print(f"fmn CLI route builds its own pipeline with the default budget: {'NativeFramePipeline::new' in source}; it has no way to declare one")
scenarios = (PIN / "crates/fmn-conformance/tests/e2e_scenarios.rs").read_text()
print(f"e2e_scenarios.rs mentions plan_limits: {'plan_limits' in scenarios}; budget01 and this patch register no scenario")
compiled = (PIN / "crates/fmn/src/rendering/compiled.rs").read_text()
print(f"compiled.rs reaches workers through RenderSink::new, which the budget patch changed: {'RenderSink::new(options, fs)' in compiled}")
