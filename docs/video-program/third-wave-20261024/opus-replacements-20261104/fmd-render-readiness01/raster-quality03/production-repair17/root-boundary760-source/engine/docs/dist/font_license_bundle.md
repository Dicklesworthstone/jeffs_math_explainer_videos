# The Font + License Bundle (fm-aef)

Every artifact class FrankenManim ships — binary, Python wheel, npm
package — embeds the same ten font faces and must carry the same license
inventory. `dist/FONT_BUNDLE.json` is the single manifest of that bundle:
what the faces are, what they hash to, and which license text covers each
one. This page is the format specification.

## What ships

| Item | Path inside `dist/` | License |
|---|---|---|
| 5 Computer Modern faces (roman, bold, italic, bold-italic, typewriter) | compiled into the binary (`fmd-font` `ALL_FACES`) | `licenses/fonts/computer-modern-OFL.txt` (OFL-1.1) |
| 4 IBM Plex Sans faces (regular, bold, italic, bold-italic) | compiled into the binary | `licenses/fonts/ibm-plex-sans-OFL.txt` (OFL-1.1) |
| Noto Sans Math symbol-fallback subset | compiled into the binary | `licenses/fonts/noto-sans-math-OFL.txt` (OFL-1.1) |
| The engine itself | the binary / wheel / npm artifact | `licenses/LICENSE` (MIT with the OpenAI/Anthropic rider) |

The faces are pinned by SUITE.lock (`franken_markdown` rev) and consumed
as `fmd_font::bundled::ALL_FACES`. The license texts are copied
byte-identically from the pinned `fmd-font` checkout's `fonts/<set>/OFL.txt`
and the repository's `LICENSE`.

**License inventory: complete, no gaps.** All ten faces across the three
families have their OFL-1.1 texts present upstream and shipped. The Noto
Sans Math face is the project's curated symbol subset (its TTF name table
is stripped by construction, so its manifest `version` is `null` — the
manifest records facts, not inventions).

**Known glyph gaps** (fm-binom-delimiters-glyph-gaps-ed1k). These are measured
from the bundled bytes, not inferred from font names:

- **✗ U+2717, ✘ U+2718, ✕ U+2715 and ✔ U+2714 are in no bundled face.** The
  fallback subset's curated range U+2713–U+2718 yields only ✓ U+2713,
  because Noto Sans Math carries nothing else in it. `Text("✗")` fails with
  the precise error `character '✗' (U+2717) has no glyph in the selected
  faces` and never substitutes. Comparison tables use the drawn
  `checkmark`/`exmark` mobjects (`fmn_library::matchers`), or `×` U+00D7,
  which IBM Plex Sans and the fallback carry.
- **θ U+03B8 in Computer Modern draws as ϑ.** In all three bundled CMU
  faces (`cmunrm`, `cmunbx`, `cmunti`) the `theta` glyph is
  outline-identical to `theta1` (ϑ U+03D1), so CM text and `\theta` set the
  script form (math-mode report fm-5wq.60). The closed θ needs another
  glyph source: the CM math faces of ADR-0028
  (fm-cm-math-faces-impl-n12u). IBM Plex Sans's θ is the closed form.

## Generation

```bash
cargo run -p fmn-conformance --bin gen_font_manifest
```

regenerates `dist/licenses/` and `dist/FONT_BUNDLE.json` from the pinned
checkout (located via SUITE.lock + `$CARGO_HOME/git/checkouts`;
`--fmd-font DIR` overrides, `--repo DIR` relocates). Run it whenever
SUITE.lock's `franken_markdown` pin moves or the repository `LICENSE`
changes. `--check` verifies without writing — the release-CI form.

A Rust generator (in `fmn-conformance`), not a Python script, by
deliberate choice: the drift gate must recompute every hash from the
actual bundled bytes through fmn-hash's owned SHA-256 — the same identity
the §16.7 input closure and the typeset cache fingerprint key against.
Generating and verifying from one crate makes the byte-for-byte check a
pure function call: no parser dependency in the governed closure (D1), no
second definition of the canonical encoding to drift.

## Format: `dist/FONT_BUNDLE.json` (`fmn-font-bundle/1`)

JSON, not TOML: emitted byte-exactly with fixed key order, two-space
indentation, LF endings, and a trailing newline; the verifier regenerates
and byte-compares rather than parsing, so no JSON/TOML parser ever enters
the closure. Wheel and npm toolchains read JSON natively.

- `format` — `fmn-font-bundle/1`.
- `generated_by` — the regeneration command.
- `hash_convention` — verbatim: SHA-256 (fmn-hash, FIPS 180-4, lowercase
  hex via `Digest::to_hex`) over the exact bundled TTF bytes of
  `ALL_FACES` at the SUITE.lock pin. A face's hash here **is** the digest
  the input closure records — one hash function, one byte identity, one
  hex rendering, everywhere.
- `suite_lock` — `{repo, rev}` the faces pin to.
- `faces[]` — in `ALL_FACES` registry order: `name` (stable face name),
  `family` (the declared bundle family — the engine's own registry names,
  stable across subsetting), `version` (the TTF name-table version string,
  or `null` where the face carries none), `byte_len`, `sha256`, `license`
  (the covering OFL path, relative to `dist/`).
- `licenses[]` — every shipped license text: `path`, `license_id`
  (`OFL-1.1` or `MIT WITH Engine-Rider`), `covers` (face names, or
  `engine`), `byte_len`, `sha256`.

## The gates (CI)

`cargo test -p fmn-conformance --test font_bundle`:

1. **Drift** — the committed manifest must regenerate byte-for-byte from
   the actual bundled faces, the shipped license files, the repository
   `LICENSE`, and the SUITE.lock pin. A font change without regeneration
   fails (with the expected form written to
   `dist/FONT_BUNDLE.json.actual`). This is the input-closure coupling:
   the manifest can never describe bytes the engine does not ship.
2. **License completeness** — every face's OFL text ships, is non-empty,
   and is a genuine SIL OFL text; every file under `dist/licenses/fonts/`
   is manifest-listed (nothing unlisted ships); the engine's MIT+rider
   license ships and equals the repository `LICENSE`.
3. **Pin coupling** — the manifest's recorded rev must equal SUITE.lock's
   `franken_markdown` row.
4. **Public assets only** — the manifest lists exactly the `ALL_FACES`
   faces and `licenses/` paths; no private-fixture marker may appear.

## The no-corpus-leak gate (§15.3)

`cargo test -p fmn-conformance --test corpus_leak` — the release-CI
enforcement that no CC BY-NC-SA private fixture ever lands in the
shippable set. The private fixtures: `corpus/` (the harvested 3b1b
TeX-string corpus), `gallery/reference_captures/`, `scripts/manim_ref/`,
`scripts/videos_ref/`. Three teeth plus one hermetic oracle:

1. **Path tooth** — `git ls-files` (ground truth for what a commit, and
   therefore any package built from one, carries) plus the `dist/` and
   wheel/npm staging trees must contain no path under a private-fixture
   directory.
2. **Gitignore tooth** — every private-fixture directory must be reported
   ignored by `git check-ignore` against the real `.gitignore`.
3. **Content tooth** — the committed `docs/g0/g0-4-corpus/denominator.tsv`
   carries `sha256(mode + NUL + string)` for every corpus string (hashes
   are public; strings never ship). Every text file on the surface is
   scanned line-wise under the same convention via fmn-hash, so a copied
   corpus string is caught *without the corpus present in CI*. Lines are
   hashed raw, trimmed, and quote-stripped (the fixture-file and
   string-literal leak forms), under both harvest modes. Where the private
   fixtures exist on disk, their whole-file digests are also collected
   and any byte-identical surface file is flagged (copied Reference
   captures).

   One documented boundary: the corpus holds thousands of trivially short
   fragments (`"!"`, `" = "`, even the empty string — pieces of
   multi-argument `Tex()` call sites) whose hashes would collide with
   ordinary source lines; variants shorter than 16 bytes are not hashed.
   The NC substance — authored strings a leak would actually copy — is
   long-form, and the path/whole-file teeth are length-blind.

Negative tests plant synthetic leaks (copied strings in each embedding
form, byte-identical files, private paths) and prove each tooth bites;
the gate test then runs all teeth over the real tree and fails loudly
with every finding. The `docs/g0/*-renders/` directories are *not*
private: they carry the engine's own renders and are covered by the whole
-content tooth against the Reference-capture digests.
