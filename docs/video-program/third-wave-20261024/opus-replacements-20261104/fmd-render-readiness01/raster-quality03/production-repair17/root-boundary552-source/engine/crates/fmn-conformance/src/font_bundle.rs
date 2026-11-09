//! The font+license bundle manifest (fm-aef, §15.3/§16.7): one documented,
//! content-hashed identity for every font face the engine ships, plus the
//! license inventory that must accompany every artifact class (binary,
//! wheel, npm).
//!
//! **Why a Rust generator instead of `scripts/gen_font_manifest.py`.** The
//! drift gate must recompute every hash from the *actual bundled bytes*
//! (`fmd_font::bundled::ALL_FACES`) through `fmn_hash`'s owned SHA-256 — the
//! same identity the §16.7 input closure and the typeset cache fingerprint
//! key against. Generating from the same crate that verifies makes the
//! byte-for-byte check a pure function call: no cross-language format
//! reimplementation, no JSON/TOML parser dependency in the governed
//! closure (D1), and no second definition of the canonical encoding to
//! drift. The generator is `cargo run -p fmn-conformance --bin
//! gen_font_manifest`; the CI gate is `tests/font_bundle.rs`.
//!
//! **Format: `dist/FONT_BUNDLE.json`** (not TOML). JSON is emitted
//! byte-exactly by [`render_manifest`] with fixed key order, two-space
//! indentation, and a trailing newline; the verification side never
//! *parses* it — it regenerates and byte-compares — so no parser exists to
//! disagree with the emitter. Wheel and npm packaging toolchains both read
//! JSON natively, and JSON needs no new dependency anywhere in the release
//! pipeline. The full format specification lives in
//! `docs/dist/font_license_bundle.md`.
//!
//! Per face the manifest records: stable name (`ALL_FACES` registry order),
//! family (the declared bundle family — stable across subsetting; the
//! engine's own family registry names), version (the TTF `name` table's
//! version string where the face carries one, else `null` — the curated
//! Noto Sans Math subset ships a stripped, empty name table by
//! construction, which the manifest documents rather than invents around),
//! byte length, and the content hash: lowercase hex SHA-256 over the exact
//! bundled TTF bytes via [`fmn_hash::sha256()`].
//!
//! A face's hash here **is** the digest the input closure records and the
//! typeset cache's font component keys against: one hash function
//! (fmn-hash, FIPS 180-4), one byte identity (the `ALL_FACES` slice), one
//! hex rendering ([`fmn_hash::Digest::to_hex`]). A font change without a
//! manifest regeneration is a CI block by construction.

use fmn_hash::sha256;
use std::collections::BTreeMap;
use std::fmt;

/// The manifest format tag, carried in the `format` field.
pub const MANIFEST_FORMAT: &str = "fmn-font-bundle/1";
/// The committed manifest path, relative to the repository root.
pub const MANIFEST_PATH: &str = "dist/FONT_BUNDLE.json";
/// The regeneration command, carried in the manifest for provenance.
pub const GENERATOR_COMMAND: &str = "cargo run -p fmn-conformance --bin gen_font_manifest";
/// The hash-convention statement, carried in the manifest verbatim.
pub const HASH_CONVENTION: &str = "sha256 (fmn-hash, FIPS 180-4; lowercase hex via Digest::to_hex) \
     over the exact bundled TTF bytes of fmd-font's ALL_FACES at the SUITE.lock pin — the same \
     identity the §16.7 input closure and the typeset cache fingerprint key against";
/// The shipped engine-license path inside `dist/`.
pub const ENGINE_LICENSE_PATH: &str = "licenses/LICENSE";
/// The engine license identifier as recorded in the manifest.
pub const ENGINE_LICENSE_ID: &str = "MIT WITH Engine-Rider";

/// One bundled face's declared policy: its `ALL_FACES` stable name, the
/// license-set slug its OFL text ships under, and the bundle family name
/// (the engine's own registry names — stable across subsetting, unlike the
/// TTF name table, which the symbol-fallback subset strips).
pub struct FacePolicy {
    /// The `ALL_FACES` stable name.
    pub name: &'static str,
    /// The license-set slug: the OFL text ships at
    /// `licenses/fonts/<slug>-OFL.txt` inside `dist/`.
    pub license_slug: &'static str,
    /// The declared bundle family.
    pub family: &'static str,
}

/// The license policy for every bundled face, in `ALL_FACES` registry
/// order. A new face without a policy row fails the manifest build loudly —
/// that *is* the "no unlicensed asset ships" gate: the set of shippable
/// faces is closed and each one names its license at rest.
pub const FACE_POLICY: &[FacePolicy] = &[
    FacePolicy {
        name: "cm-regular",
        license_slug: "computer-modern",
        family: "Computer Modern",
    },
    FacePolicy {
        name: "cm-bold",
        license_slug: "computer-modern",
        family: "Computer Modern",
    },
    FacePolicy {
        name: "cm-italic",
        license_slug: "computer-modern",
        family: "Computer Modern",
    },
    FacePolicy {
        name: "cm-bold-italic",
        license_slug: "computer-modern",
        family: "Computer Modern",
    },
    FacePolicy {
        name: "cm-typewriter",
        license_slug: "computer-modern",
        family: "CM Typewriter",
    },
    FacePolicy {
        name: "plex-regular",
        license_slug: "ibm-plex-sans",
        family: "IBM Plex Sans",
    },
    FacePolicy {
        name: "plex-bold",
        license_slug: "ibm-plex-sans",
        family: "IBM Plex Sans",
    },
    FacePolicy {
        name: "plex-italic",
        license_slug: "ibm-plex-sans",
        family: "IBM Plex Sans",
    },
    FacePolicy {
        name: "plex-bold-italic",
        license_slug: "ibm-plex-sans",
        family: "IBM Plex Sans",
    },
    FacePolicy {
        name: "noto-sans-math-symbols",
        license_slug: "noto-sans-math",
        family: "Noto Sans Math",
    },
];

/// Every license-set slug that must ship an OFL text, in canonical order.
pub const LICENSE_SLUGS: &[&str] = &["computer-modern", "ibm-plex-sans", "noto-sans-math"];

/// The `dist/`-relative path a license set's OFL text ships at.
#[must_use]
pub fn ofl_path(slug: &str) -> String {
    format!("licenses/fonts/{slug}-OFL.txt")
}

/// A manifest face row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FaceEntry {
    /// The `ALL_FACES` stable name.
    pub name: String,
    /// The declared bundle family.
    pub family: String,
    /// The TTF name-table version string, trimmed; `None` when the face
    /// carries no version record (the curated symbol subset).
    pub version: Option<String>,
    /// The exact bundled byte length.
    pub byte_len: u64,
    /// Lowercase hex SHA-256 over the bundled bytes.
    pub sha256_hex: String,
    /// The `dist/`-relative path of the OFL text covering this face.
    pub license: String,
}

/// A manifest license row.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LicenseEntry {
    /// The `dist/`-relative path.
    pub path: String,
    /// The license identifier (`OFL-1.1` or the engine license id).
    pub license_id: String,
    /// What the license covers: face names, or `engine` for the engine's
    /// own license.
    pub covers: Vec<String>,
    /// Byte length of the shipped text.
    pub byte_len: u64,
    /// Lowercase hex SHA-256 over the shipped bytes.
    pub sha256_hex: String,
}

/// The whole bundle manifest, pre-render.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BundleManifest {
    /// The SUITE.lock repo name the faces pin to (`franken_markdown`).
    pub pin_repo: String,
    /// The full SUITE.lock revision the faces pin to.
    pub pin_rev: String,
    /// Face rows, in `ALL_FACES` registry order.
    pub faces: Vec<FaceEntry>,
    /// License rows: every OFL set, then the engine license.
    pub licenses: Vec<LicenseEntry>,
}

/// A manifest-construction failure. Every variant is a named gap — the
/// release pipeline must never invent around one.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FontBundleError {
    /// An `ALL_FACES` face has no [`FACE_POLICY`] row (a new face landed
    /// without a license assignment).
    FaceWithoutPolicy { name: String },
    /// A [`FACE_POLICY`] row names a face `ALL_FACES` does not carry.
    PolicyWithoutFace { name: String },
    /// A required license set's OFL bytes were not supplied.
    MissingLicense { slug: String },
    /// License bytes were supplied for a slug that covers no face.
    UnexpectedLicense { slug: String },
}

impl fmt::Display for FontBundleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FaceWithoutPolicy { name } => write!(
                f,
                "bundled face '{name}' has no FACE_POLICY row: every shippable face must name \
                 its license set (no unlicensed asset ships)"
            ),
            Self::PolicyWithoutFace { name } => {
                write!(
                    f,
                    "FACE_POLICY row '{name}' names a face ALL_FACES does not carry"
                )
            }
            Self::MissingLicense { slug } => {
                write!(f, "license set '{slug}': no OFL text supplied")
            }
            Self::UnexpectedLicense { slug } => {
                write!(f, "license set '{slug}' covers no bundled face")
            }
        }
    }
}

impl std::error::Error for FontBundleError {}

/// Reads one TTF `name`-table string (name IDs 1 = family, 5 = version).
///
/// Preference order: Windows platform English (3, *, 0x0409), then Unicode
/// platform (0), then Macintosh (1), then any record. Returns `None` for a
/// missing/malformed name table or a missing record — absence is
/// representable, never an error (the symbol subset's table is empty by
/// construction). Platform 0/3 strings decode UTF-16BE; platform 1 strings
/// are taken as ASCII (every bundled face's records are).
fn ttf_name_string(bytes: &[u8], want_name_id: u16) -> Option<String> {
    let be_u16 = |at: usize| -> Option<u16> {
        bytes
            .get(at..at + 2)
            .map(|w| u16::from_be_bytes([w[0], w[1]]))
    };
    let be_u32 = |at: usize| -> Option<u32> {
        bytes
            .get(at..at + 4)
            .map(|w| u32::from_be_bytes([w[0], w[1], w[2], w[3]]))
    };
    let num_tables = usize::from(be_u16(4)?);
    let mut name_range = None;
    for i in 0..num_tables {
        let rec = 12 + i * 16;
        if bytes.get(rec..rec + 4)? == b"name" {
            let off = usize::try_from(be_u32(rec + 8)?).ok()?;
            let len = usize::try_from(be_u32(rec + 12)?).ok()?;
            name_range = Some((off, off.checked_add(len)?));
            break;
        }
    }
    let (table_off, table_end) = name_range?;
    let count = usize::from(be_u16(table_off + 2)?);
    let string_off = table_off + usize::from(be_u16(table_off + 4)?);
    // (rank, value); lower rank wins.
    let mut best: Option<(u8, String)> = None;
    for i in 0..count {
        let rec = table_off + 6 + i * 12;
        if rec + 12 > table_end {
            break;
        }
        let (Some(platform), Some(name_id), Some(lang), Some(len), Some(off)) = (
            be_u16(rec),
            be_u16(rec + 6),
            be_u16(rec + 4),
            be_u16(rec + 8),
            be_u16(rec + 10),
        ) else {
            continue;
        };
        if name_id != want_name_id {
            continue;
        }
        let start = string_off.checked_add(usize::from(off))?;
        let end = start.checked_add(usize::from(len))?;
        let raw = bytes.get(start..end)?;
        let (rank, text) = match platform {
            3 | 0 => {
                let rank = if platform == 3 && lang == 0x0409 {
                    0
                } else {
                    1
                };
                let decoded: String = char::decode_utf16(
                    raw.as_chunks::<2>()
                        .0
                        .iter()
                        .map(|w| u16::from_be_bytes(*w)),
                )
                .map(|r| r.unwrap_or(char::REPLACEMENT_CHARACTER))
                .collect();
                (rank, decoded)
            }
            1 => (
                2,
                raw.iter()
                    .filter(|b| b.is_ascii())
                    .map(|b| char::from(*b))
                    .collect(),
            ),
            _ => (3, String::from_utf8_lossy(raw).into_owned()),
        };
        let better = match &best {
            Some((best_rank, _)) => rank < *best_rank,
            None => true,
        };
        if better {
            best = Some((rank, text));
        }
    }
    best.map(|(_, text)| text)
}

/// Computes one face row from its policy and bundled bytes.
#[must_use]
pub fn face_entry(policy: &FacePolicy, bytes: &[u8]) -> FaceEntry {
    let version = ttf_name_string(bytes, 5).map(|v| v.trim().to_owned());
    FaceEntry {
        name: policy.name.to_owned(),
        family: policy.family.to_owned(),
        version,
        byte_len: bytes.len() as u64,
        sha256_hex: sha256(bytes).to_hex(),
        license: ofl_path(policy.license_slug),
    }
}

/// Builds the manifest from the bundled faces (as `(stable name, bytes)`
/// in registry order — pass `fmd_font::bundled::ALL_FACES`), the OFL texts
/// as `(slug, bytes)`, and the engine's own license bytes.
///
/// # Errors
///
/// Returns [`FontBundleError`] on any face/license-set mismatch between
/// the faces, [`FACE_POLICY`], and the supplied OFL texts — a named gap,
/// never an invented one.
pub fn build_manifest(
    pin_rev: &str,
    faces: &[(&str, &[u8])],
    ofl_texts: &[(&str, &[u8])],
    engine_license: &[u8],
) -> Result<BundleManifest, FontBundleError> {
    let policy_by_name: BTreeMap<&str, &FacePolicy> =
        FACE_POLICY.iter().map(|p| (p.name, p)).collect();
    let mut entries = Vec::with_capacity(faces.len());
    for (name, bytes) in faces {
        let policy =
            policy_by_name
                .get(name)
                .ok_or_else(|| FontBundleError::FaceWithoutPolicy {
                    name: (*name).to_owned(),
                })?;
        entries.push(face_entry(policy, bytes));
    }
    for policy in FACE_POLICY {
        if !faces.iter().any(|(name, _)| *name == policy.name) {
            return Err(FontBundleError::PolicyWithoutFace {
                name: policy.name.to_owned(),
            });
        }
    }
    let ofl_by_slug: BTreeMap<&str, &[u8]> = ofl_texts.iter().map(|(s, b)| (*s, *b)).collect();
    let mut licenses = Vec::with_capacity(LICENSE_SLUGS.len() + 1);
    for slug in LICENSE_SLUGS {
        let bytes = ofl_by_slug
            .get(slug)
            .ok_or_else(|| FontBundleError::MissingLicense {
                slug: (*slug).to_owned(),
            })?;
        let covers: Vec<String> = FACE_POLICY
            .iter()
            .filter(|p| p.license_slug == *slug)
            .map(|p| p.name.to_owned())
            .collect();
        if covers.is_empty() {
            return Err(FontBundleError::UnexpectedLicense {
                slug: (*slug).to_owned(),
            });
        }
        licenses.push(LicenseEntry {
            path: ofl_path(slug),
            license_id: "OFL-1.1".to_owned(),
            covers,
            byte_len: bytes.len() as u64,
            sha256_hex: sha256(bytes).to_hex(),
        });
    }
    for slug in ofl_by_slug.keys() {
        if !LICENSE_SLUGS.contains(slug) {
            return Err(FontBundleError::UnexpectedLicense {
                slug: (*slug).to_owned(),
            });
        }
    }
    licenses.push(LicenseEntry {
        path: ENGINE_LICENSE_PATH.to_owned(),
        license_id: ENGINE_LICENSE_ID.to_owned(),
        covers: vec!["engine".to_owned()],
        byte_len: engine_license.len() as u64,
        sha256_hex: sha256(engine_license).to_hex(),
    });
    Ok(BundleManifest {
        pin_repo: "franken_markdown".to_owned(),
        pin_rev: pin_rev.to_owned(),
        faces: entries,
        licenses,
    })
}

/// Escapes a string for canonical JSON emission (the manifest's strings
/// are controlled slugs and name-table text; the escaper is total anyway).
fn json_escape(text: &str, out: &mut String) {
    out.push('"');
    for ch in text.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use fmt::Write as _;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
}

fn push_kv_str(out: &mut String, indent: &str, key: &str, value: &str, comma: bool) {
    out.push_str(indent);
    json_escape(key, out);
    out.push_str(": ");
    json_escape(value, out);
    out.push_str(if comma { ",\n" } else { "\n" });
}

fn push_kv_num(out: &mut String, indent: &str, key: &str, value: u64, comma: bool) {
    use fmt::Write as _;
    out.push_str(indent);
    json_escape(key, out);
    let _ = write!(out, ": {value}{}", if comma { ",\n" } else { "\n" });
}

/// Renders the manifest to its canonical byte form: fixed key order,
/// two-space indentation, LF line endings, trailing newline. Byte-for-byte
/// equality with the committed `dist/FONT_BUNDLE.json` is the drift gate.
#[must_use]
pub fn render_manifest(manifest: &BundleManifest) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    push_kv_str(&mut out, "  ", "format", MANIFEST_FORMAT, true);
    push_kv_str(&mut out, "  ", "generated_by", GENERATOR_COMMAND, true);
    push_kv_str(&mut out, "  ", "hash_convention", HASH_CONVENTION, true);
    out.push_str("  \"suite_lock\": {\n");
    push_kv_str(&mut out, "    ", "repo", &manifest.pin_repo, true);
    push_kv_str(&mut out, "    ", "rev", &manifest.pin_rev, false);
    out.push_str("  },\n");
    out.push_str("  \"faces\": [\n");
    for (i, face) in manifest.faces.iter().enumerate() {
        let last = i + 1 == manifest.faces.len();
        out.push_str("    {\n");
        push_kv_str(&mut out, "      ", "name", &face.name, true);
        push_kv_str(&mut out, "      ", "family", &face.family, true);
        match &face.version {
            Some(version) => push_kv_str(&mut out, "      ", "version", version, true),
            None => out.push_str("      \"version\": null,\n"),
        }
        push_kv_num(&mut out, "      ", "byte_len", face.byte_len, true);
        push_kv_str(&mut out, "      ", "sha256", &face.sha256_hex, true);
        push_kv_str(&mut out, "      ", "license", &face.license, false);
        out.push_str(if last { "    }\n" } else { "    },\n" });
    }
    out.push_str("  ],\n");
    out.push_str("  \"licenses\": [\n");
    for (i, license) in manifest.licenses.iter().enumerate() {
        let last = i + 1 == manifest.licenses.len();
        out.push_str("    {\n");
        push_kv_str(&mut out, "      ", "path", &license.path, true);
        push_kv_str(&mut out, "      ", "license_id", &license.license_id, true);
        out.push_str("      \"covers\": [");
        for (j, covered) in license.covers.iter().enumerate() {
            if j > 0 {
                out.push_str(", ");
            }
            json_escape(covered, &mut out);
        }
        out.push_str("],\n");
        push_kv_num(&mut out, "      ", "byte_len", license.byte_len, true);
        push_kv_str(&mut out, "      ", "sha256", &license.sha256_hex, false);
        out.push_str(if last { "    }\n" } else { "    },\n" });
    }
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

/// Extracts a repo's pinned revision from SUITE.lock text: the
/// tab-separated row whose first field is `repo` inside the `[repos]`
/// section.
#[must_use]
pub fn suite_lock_pin(suite_lock: &str, repo: &str) -> Option<String> {
    let mut in_repos = false;
    for line in suite_lock.lines() {
        if let Some(section) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            in_repos = section == "repos";
            continue;
        }
        if !in_repos || line.starts_with('#') {
            continue;
        }
        let mut fields = line.split('\t');
        if fields.next() == Some(repo) {
            return fields.next().map(str::to_owned);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_table_reads_family_and_version() {
        let bytes: &[u8] = fmd_font::bundled::CM_REGULAR;
        assert_eq!(ttf_name_string(bytes, 1).as_deref(), Some("CMU Serif"));
        assert_eq!(
            ttf_name_string(bytes, 5)
                .map(|v| v.trim().to_owned())
                .as_deref(),
            Some("Version 0.7.0")
        );
    }

    #[test]
    fn missing_name_table_is_none_not_an_error() {
        assert_eq!(ttf_name_string(b"\x00\x01\x00\x00", 5), None);
        assert_eq!(ttf_name_string(b"", 5), None);
    }

    #[test]
    fn json_escape_covers_controls_and_quotes() {
        let mut out = String::new();
        json_escape("a\"b\\c\n\u{1}", &mut out);
        assert_eq!(out, "\"a\\\"b\\\\c\\n\\u0001\"");
    }

    #[test]
    fn suite_lock_pin_reads_the_repos_section() {
        let text = "[toolchain]\nrustc\tnightly-x\n\n[repos]\n# comment\n\
                    franken_markdown\tabc123\tnotes\n\n[patches]\nother\tdef456\n";
        assert_eq!(
            suite_lock_pin(text, "franken_markdown").as_deref(),
            Some("abc123")
        );
        assert_eq!(suite_lock_pin(text, "other"), None);
    }

    #[test]
    fn a_face_without_policy_is_a_named_error() {
        let result = build_manifest("rev", &[("mystery-face", b"x")], &[], b"license");
        assert_eq!(
            result,
            Err(FontBundleError::FaceWithoutPolicy {
                name: "mystery-face".to_owned()
            })
        );
    }

    #[test]
    fn a_missing_ofl_text_is_a_named_error() {
        let faces: Vec<(&str, &[u8])> = fmd_font::bundled::ALL_FACES.to_vec();
        let result = build_manifest("rev", &faces, &[], b"license");
        assert_eq!(
            result,
            Err(FontBundleError::MissingLicense {
                slug: "computer-modern".to_owned()
            })
        );
    }
}
