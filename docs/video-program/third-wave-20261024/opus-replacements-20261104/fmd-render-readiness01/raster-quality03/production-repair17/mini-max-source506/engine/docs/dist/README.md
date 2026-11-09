# Distribution Documentation (W11, fm-aef)

The user-facing home for everything a release ships and promises. One home —
these documents publish with the release pipeline; crate-level documentation
stays in the crates and is linked, not duplicated.

| Document | Content |
|---|---|
| [install_matrix.md](install_matrix.md) | Install matrix (platform × SIMD build tier), the tier explanation, and artifact selection |
| [ci_coverage.md](ci_coverage.md) | CI lanes per platform, the exact Windows promise (functional now; bit-cert ruled out by ADR-0019), the PG-5 cadences, and the R22 time budgets |
| [determinism.md](determinism.md) | The certified-determinism user guide: what `--reproducible` promises, requires, and excludes |
| [ffmpeg_optional.md](ffmpeg_optional.md) | The ffmpeg-optional posture: native sinks, the capability error, what needs ffmpeg |
| [cache_config_conventions.md](cache_config_conventions.md) | Per-platform cache/config directory conventions (replacing appdirs/diskcache), `--clear-cache` and `fmn doctor` behavior, the migration-note policy |
| [studio_ui.md](studio_ui.md) | Embedded Studio UI assets: compiled-in, versioned with the binary, no runtime file serving (§13.5) |
| [font_license_bundle.md](font_license_bundle.md) | The font + license bundle manifest: bundled faces, OFL texts, content hashes, the no-corpus-leak check |
| [python_wheel.md](python_wheel.md) | CPython ABI, exact `manimlib` namespace ownership, console capability boundary, clean-venv ritual, and current matrix gaps |

## The Behavior Notes' distribution home

The Behavior Notes — deliberate, evidence-backed differences from classic
manim, written as migration guidance — live at
[../behavior_notes/](../behavior_notes/README.md) and ship with every release
unchanged. That register is the authoritative index (§16.8); it is linked
here, never duplicated.

## Related normative documents

- [../INPUT_CLOSURE.md](../INPUT_CLOSURE.md) — the input-closure specification
  (§16.7): what "the same content-hashed input closure" enumerates.
- [../FFMPEG_PROTOCOL.md](../FFMPEG_PROTOCOL.md) — the negotiated ffmpeg
  boundary: sandboxing, hashing, audit trail.
- [../GOVERNANCE.md](../GOVERNANCE.md) — gate G5 (Distribution & Leapfrogs)
  and the policy-ruling discipline these documents follow.
