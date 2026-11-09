# ADR-0021 — Custom GLSL adopts Strategy A exclusions only; Strategy B adapters rejected

**Status:** Accepted
**Date:** 2026-08-27
**Bead:** fm-afcw (W10)
**Resolves:** OQ-9 (W10)
**Amends:** Plan §15.4, §23 (OQ-9 entry)

## Context

The Comprehensive Plan (§15.4) identified arbitrary shader directories and custom GLSL code (`set_color_by_code`, custom shaders embedded in scene files) as outside the core deterministic rendering pipeline. The plan formulated three strategies:
- **Strategy A:** Exclude custom GLSL / arbitrary shader folders from the compatibility claim with clear, typed capability refusals and explicit exclusion records in `VIDEO_CORPUS.lock` (R13).
- **Strategy B:** Maintain a short, bespoke list of known-corpus native adapters (e.g. fractal-uniform protocols) if allowlisted gallery scenes require them.
- **Strategy C:** A restricted GLSL interpreter (banked as a non-priority alternative).

With the gallery allowlist (`VIDEO_CORPUS.lock`) stabilized across the seed and expanded scene sets, the corpus curation scan mechanically confirms that the mathematical animation scenes in the gallery rely entirely on standard Vector, VMobject, 3D surface, and typography layout. Zero allowlisted scenes require custom GLSL fragment shaders or unbacked uniform protocols.

Maintaining Strategy B adapters would introduce bespoke per-scene shims and technical debt into the native engine runtime, violating the FrankenManim Engineering Doctrine (D1, D5: "Do things the RIGHT way with NO TECH DEBT").

## Decision

1. **Strategy A is the binding policy for custom GLSL.** Custom shader folders, raw OpenGL/GLSL injections, and `set_color_by_code` invocations that rely on arbitrary fragment shaders are permanently excluded from the compatibility claim.
2. **No Strategy B adapters in the engine.** FrankenManim will not ship or maintain per-scene native adapters or fractal-uniform translation layers.
3. **Precise capability refusal.** Any attempt to supply custom GLSL shaders or unbacked shader directories through the Python portal or native API yields a typed capability error (`fmn::ErrorKind::Capability` / `fmn_platform::fetch::FetchError::CapabilityAbsent` or scene-level capability refusal) naming the unsupported feature and directing callers to standard native mobjects.
4. **Corpus governance.** `VIDEO_CORPUS.lock` excludes scenes relying on out-of-band custom GLSL shaders under the documented exclusion taxonomy (R13).

## Consequences

- **Engine integrity preserved:** The native rendering pipeline (Lumen/Marionette) remains clean and sovereign without bespoke shader hacks or runtime translation layers.
- **Corpus tooling unblocked:** `VIDEO_CORPUS.lock` verification and `scripts/video_corpus.py` enforce Strategy A exclusions as a permanent invariant.
- **Plan true-up:** §15.4 and §23 (OQ-9) are trued up in the same commit to record this resolution.
