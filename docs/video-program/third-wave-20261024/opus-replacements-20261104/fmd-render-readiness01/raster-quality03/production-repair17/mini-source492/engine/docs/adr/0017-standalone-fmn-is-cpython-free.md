# ADR-0017 — Keep standalone `fmn` CPython-free

**Status:** Accepted
**Date:** 2026-08-09
**Bead:** fm-7wm.3 (the one-binary CLI and Python runtime boundary)
**Amends:** adds D-25 and clarifies policy under D-01, D-02, and D-12

## Context

The project promised both a one-binary, CPython-free `fmn` installation and
direct `fmn scene.py` execution. Those promises cannot coexist without either
embedding a Python runtime or locating and spawning one. Embedding would put
CPython, PyO3, and NumPy into the native distribution closure; spawning would
make Python a second external tool and violate D-02. The existing crate graph
already points to the coherent split: `fmn-cli` has no Python dependency, while
ADR-0015 defines `fmn-python` as a CPython-pinned extension boundary.

This is also a security boundary. Searching `PATH` for an interpreter makes
execution depend on mutable host state and permits interpreter substitution.
Running an untrusted Python scene is arbitrary code execution with the host
interpreter's authority; it must not be smuggled into the standalone CLI's
capability model or one-binary claim.

## Decision

1. The native Rust library, standalone `fmn` binary, and Studio distribution
   never embed, link, locate, download, or spawn CPython. They contain no
   `fmn-python`, PyO3, or NumPy runtime dependency. Under D-02, ffmpeg remains
   the engine's only possible subprocess.
2. Python source files (`.py` and `.pyw`, case-insensitive) belong exclusively
   to the optional `fmn-python` portal. Passing one to standalone `fmn`,
   `fmn batch`, or `fmn studio` fails before file access or process launch with
   exit identity `capability` and names the `fmn-python` entry point.
3. W11's wheel installs the `manimlib` import surface and a `fmn-python`
   console entry point. It requires the supported host CPython ABI and NumPy
   policy from ADR-0015. The portal calls the same Rust engine; it is not a
   second renderer or compatibility shim.
4. Native scene registrations and compiled native scene artifacts remain the
   standalone CLI's composition input. The production adapter and exact
   artifact contract remain owned by fm-ffj.67; this ADR does not claim that
   composition is already wired.
5. Certified Python-portal runs add the host CPython implementation/version,
   ABI tag, wheel hash, NumPy identity, and transitively imported Python module
   bytes to the input closure. Native `fmn` manifests encode the portal as
   absent; CPython cannot influence their bits.
6. Python scenes execute with the host interpreter's authority inside the
   isolated scene-worker boundary. The portal documentation must state that
   arbitrary scene code can perform arbitrary user-authorized Python I/O. The
   standalone CLI never probes host Python state, so interpreter injection is
   outside its attack surface.

## Consequences

- The one-binary native claim is literal and testable, and D-02 keeps one
  subprocess boundary instead of gaining an implicit Python exception.
- Existing manim programs use `fmn-python scene.py SceneName`; README examples
  and generated CLI help no longer advertise `fmn scene.py`.
- fm-ffj.67 can implement one native composition seam without also becoming a
  Python runtime manager. fm-vsq owns the wheel, console entry point, clean-venv
  execution test, ABI matrix, and `manimlib` namespace policy.
- A future decision to bundle or launch an interpreter must supersede this ADR,
  amend D-02/D-25, and re-audit dependency closure, licensing, provenance, and
  the threat model. Mere convenience or interpreter availability is not a
  revisit trigger.
