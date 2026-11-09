# ADR-0025 — A certified build names its exact sources; unidentifiable builds are refused

**Status:** Accepted (implemented with this ADR under the owner's standing instruction to decide on expert judgment; open to owner revision)
**Date:** 2026-09-26
**Bead:** fm-certified-closure-integrity-4fei (item 1)
**Amends:** policy under §16.7 (the certified input closure, C2) as implemented by `crates/fmn-cli/build.rs`, `crates/fmn-python/build.rs` and `docs/INPUT_CLOSURE.md`

## Context

C2 records the engine identity `FMN_BUILD_ID`. Until now the build scripts read `.git/HEAD` and embedded `git:<commit>`, with no notion of the working tree. Many agents edit one shared checkout concurrently, so a binary compiled from uncommitted edits still called itself `git:<commit>`. A certified manifest could therefore name a clean commit that the binary was not built from, which is exactly the lie the closure exists to prevent. Without `.git` and `FMN_BUILD_ID`, the fallback was `release:<package>:<version>`, which impersonates an official release.

The bead proposed refusing certified output from any dirty build. About twenty test files across `fmn`, `fmn-cli`, `fmn-conformance` and the portal gate render with `--reproducible` from development builds, and the shared tree is dirty almost continuously. Refusal would make every one of those gates depend on other agents' momentary edits. It would also add no integrity beyond naming the dirty state exactly.

## Decision

1. **Identity forms.** One implementation (`crates/fmn-cli/src/build_identity.rs`) is `include!`d by the `fmn-cli` and `fmn-python` build scripts:
   - `<FMN_BUILD_ID>`: an explicit, printable, quote-free identity from the build host. Release pipelines set this.
   - `git:<commit>`: `git status` reports no change to a compiled input.
   - `git:<commit>+dirty:<sha256>`: otherwise. The digest is over every compiled input that differs from HEAD, staged or not, tracked or not. For each, in sorted path order, it hashes the path, then the content or a deletion marker. It depends only on what would be compiled, not on git's index state.
   - `git:<commit>+unverified`: git could not report the tree state.
   - `unidentified:<package>:<version>`: neither `.git` nor `FMN_BUILD_ID`.
2. **Compiled inputs** are `crates/`, `Cargo.toml`, `Cargo.lock`, `SUITE.lock`, `rust-toolchain.toml` and `.cargo/`. Edits elsewhere (docs, beads, scratch) never dirty a build. The build scripts rerun when any compiled input, the index, HEAD or its ref changes.
3. **Refusal.** Certified output is refused, as a named capability error, for the two forms that cannot name their sources: `unidentified:` and `+unverified`. `fmn_output::certified_build_refusal` is the one predicate, and three places enforce it:
   - the CLI, before any rendering;
   - the portal's `begin_portal_render(reproducible=True)` and its certified publication route;
   - `ProvenanceManifest::new` in certified mode, as defence in depth.

   Dirty builds are identified, not refused. Their manifests carry the dirty identity in C2, so the semantic digest of a dirty build never equals a clean one, and two dirty builds compare equal only from identical sources: "equal semantic digest ⇒ equal bits" still holds.
4. **Tool boundary.** The build scripts spawn `git` (`status`, `diff --name-only`, `ls-files`) with `GIT_OPTIONAL_LOCKS=0`, so a build never takes `index.lock` from under a concurrent git operation. D-02 governs subprocesses of the engine at run time, and a build script is not the engine. A host without git yields `+unverified` or `unidentified:`, and certified output fails closed.

## Consequences

- A certified manifest no longer names a commit its binary was not built from. Anyone holding the same source state recomputes the same dirty digest. Anyone else can treat a `+dirty` identity as not release-grade. `fmn-manifest-compare` reports differing semantic digests as not comparable, unchanged.
- `unidentified:` replaces the `release:` fallback, so a build without identity cannot pass for a release. Release builds set `FMN_BUILD_ID` explicitly or build from a checkout.
- Every edit to a compiled input changes `FMN_BUILD_ID`, so `fmn-cli` and `fmn-python` recompile their own crate after such an edit. Their dependency graphs already did.
- `fmn-hash` becomes a build-dependency of `fmn-cli` and `fmn-python`, and a dev-dependency of `fmn-cli` for the module's unit tests. That is one workspace edge in `Cargo.lock`; the wasm graph is unchanged, and `docs/wasm_audit.md` records the re-run.
- The bead's acceptance line "dirty build refused" is replaced by "dirty build identified by its exact source digest; unidentifiable build refused", with the negative controls in `build_identity`'s and `provenance`'s unit tests and the CLI's `certified_renders_require_a_build_that_names_its_sources`.
