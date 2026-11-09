//! The W10 fuzzing campaign (§16.5 plane 4, fm-t1v): the concrete targets,
//! their resource-budget assertions, and the corpus/manifest gate.
//!
//! Targets (each: error precisely OR succeed within budget — never hang,
//! never overallocate, never panic):
//!
//! - `ttf_font_parse` — TTF bytes → `fmd_font::Font::parse`, with
//!   structure-aware mutators (sfnt table-directory corruptions, 32-bit
//!   length/offset field rewrites, splices) plus raw random noise;
//! - `yaml_config` — YAML-subset strings → `fmn_config::yaml` under its
//!   declared `Limits` (byte and nesting-depth budgets);
//! - `tex_math` — TeX strings → fmd-math via `fmn_tex::TexEngine`
//!   (arbitrary token streams error precisely per the chaos contract —
//!   this campaign builds on fmd-math's chaos suite's fragment pool), with
//!   the never-garble check and the typeset re-encode determinism check;
//! - `codec_png` / `codec_jpeg` — bytes → fmn-codec's decoders under tight
//!   pixel/chunk budgets (decompression-bomb awareness: accepted outputs
//!   are asserted `width × height × 4`, inside the declared budget);
//! - `canon_deser` — the canonical deserializer → `fmn_hash::fuzz_probe`,
//!   whose contract IS never-panic, asserted on structured mutations;
//! - `obj_model` — OBJ text → `fmn_library::obj_model::ObjMesh` under its
//!   declared `ObjLimits`, with grammar-biased token splices and
//!   digit/index/line structure-aware mutators;
//! - `svg_document_processor` — SVG text → `fmn_geom::SvgDocument` under its
//!   declared `SvgLimits` (fm-6nm, the W2 document processor), with
//!   XML/path-grammar token splices, entity and DOCTYPE staples, digit
//!   perturbation, and chunk-duplication budget-bomb steering;
//! - `path_boolean` — bytes → `fmn_geom::boolean::fuzz_probe` (fm-qjy):
//!   arbitrary quadratic paths through every admitted boolean route and the
//!   certified fallback under tight budgets; the contract IS never-panic
//!   and totality, asserted on structured mutations.
//!
//! Modes:
//! - CI (default): the reduced case counts; classes must be a subset of
//!   the manifest's and the regenerated interesting-input corpus a subset
//!   of the committed files. Every committed input, including historical
//!   class representatives retained across capability promotions, is replayed
//!   against the current target contract. Any violation fails.
//! - `FMN_FUZZ_FULL=1` (the scheduled campaign): the full case counts;
//!   classes must equal the manifest's exactly and every newly generated
//!   representative must be committed. Historical inputs stay in the replay
//!   corpus rather than being deleted when their old refusal becomes support.
//! - `FMN_FUZZ_FULL=1 FMN_FUZZ_BLESS=1`: regenerate the manifest and
//!   corpus for human review and commit (the rig never commits and never
//!   deletes; stale files are reported).

use std::fs::File;
use std::io::Read;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use fmn_conformance::fuzz::{
    self, Budgets, CampaignSpec, ManifestRow, Target, Verdict, XorShift64, mutate,
};

// ---------------------------------------------------------------- shared

const MAX_MANIFEST_BYTES: u64 = 1 << 20;
const MAX_CODEC_FIXTURE_BYTES: u64 = 1 << 21;

/// The committed corpus root.
fn corpus_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("fixtures/fuzz_corpus")
}

fn read_bytes_bounded(reader: impl Read, label: &str, max_bytes: u64) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::new();
    reader
        .take(max_bytes.saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("reading {label}: {error}"))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(format!(
            "reading {label}: file exceeds the {max_bytes}-byte resource limit"
        ));
    }
    Ok(bytes)
}

fn read_utf8_bounded(reader: impl Read, label: &str, max_bytes: u64) -> Result<String, String> {
    String::from_utf8(read_bytes_bounded(reader, label, max_bytes)?)
        .map_err(|error| format!("reading {label}: not UTF-8: {error}"))
}

fn read_manifest(path: &Path) -> String {
    let label = path.display().to_string();
    let file = File::open(path)
        .unwrap_or_else(|error| std::panic::panic_any(format!("opening {label}: {error}")));
    read_utf8_bounded(file, &label, MAX_MANIFEST_BYTES)
        .unwrap_or_else(|error| std::panic::panic_any(error))
}

fn replay_committed_corpus(target: &dyn Target) {
    let root = corpus_root().join(target.name());
    let mut entries = std::fs::read_dir(&root)
        .unwrap_or_else(|error| {
            std::panic::panic_any(format!("reading {}: {error}", root.display()))
        })
        .collect::<Result<Vec<_>, _>>()
        .unwrap_or_else(|error| {
            std::panic::panic_any(format!("reading {} entry: {error}", root.display()))
        });
    entries.sort_by_key(std::fs::DirEntry::file_name);

    for entry in entries {
        let file_type = entry.file_type().unwrap_or_else(|error| {
            std::panic::panic_any(format!("reading {} type: {error}", entry.path().display()))
        });
        assert!(
            file_type.is_file(),
            "{}: corpus entry is not a regular file",
            entry.path().display()
        );
        let name = entry
            .file_name()
            .into_string()
            .unwrap_or_else(|_| std::panic::panic_any("corpus file name is not UTF-8"));
        let encoded_digest = name
            .strip_suffix(".bin")
            .and_then(|stem| stem.rsplit_once("__"))
            .map(|(_, digest)| digest)
            .filter(|digest| {
                digest.len() == 12 && digest.bytes().all(|byte| byte.is_ascii_hexdigit())
            })
            .unwrap_or_else(|| {
                std::panic::panic_any(format!("{name}: noncanonical corpus file name"))
            });
        let path = entry.path();
        let file = File::open(&path).unwrap_or_else(|error| {
            std::panic::panic_any(format!("opening {}: {error}", path.display()))
        });
        let bytes = read_bytes_bounded(
            file,
            &path.display().to_string(),
            target.budgets().max_input_bytes,
        )
        .unwrap_or_else(|error| std::panic::panic_any(error));
        assert_eq!(
            &fmn_hash::sha256(&bytes).to_hex()[..12],
            encoded_digest,
            "{name}: content address disagrees with corpus bytes"
        );

        match catch_unwind(AssertUnwindSafe(|| target.run(&bytes))) {
            Ok(Verdict::Accepted { output_bytes }) => {
                if let Some(max) = target.budgets().max_output_bytes {
                    assert!(
                        output_bytes <= max,
                        "{name}: accepted output {output_bytes} exceeds {max} bytes"
                    );
                }
            }
            Ok(Verdict::Refused { message, .. }) => assert!(
                target.refusal_is_precise(&message),
                "{name}: imprecise refusal while replaying retained corpus: {message}"
            ),
            Ok(Verdict::Fault { message }) => {
                panic!("{name}: target fault while replaying retained corpus: {message}")
            }
            Err(_) => panic!("{name}: target panicked while replaying retained corpus"),
        }
    }
}

/// Extract a stable outcome-class label from a typed error's `Debug`
/// variant name (`TooLarge { .. }` → `too-large`).
fn debug_variant_class(e: &impl std::fmt::Debug) -> String {
    let debug = format!("{e:?}");
    let variant: String = debug
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric())
        .collect();
    let mut class = String::with_capacity(variant.len() + 4);
    for (i, ch) in variant.chars().enumerate() {
        if ch.is_ascii_uppercase() && i > 0 {
            class.push('-');
        }
        class.push(ch.to_ascii_lowercase());
    }
    class
}

/// Read a committed fmn-codec test fixture as a mutation seed (the codec
/// has no JPEG encoder; its own pinned fixtures are the canonical small
/// real-world inputs).
fn codec_fixture(rel: &str) -> Vec<u8> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../fmn-codec/tests/fixtures")
        .join(rel);
    let label = path.display().to_string();
    let file = File::open(&path)
        .unwrap_or_else(|error| std::panic::panic_any(format!("opening {label}: {error}")));
    read_bytes_bounded(file, &label, MAX_CODEC_FIXTURE_BYTES)
        .unwrap_or_else(|error| std::panic::panic_any(error))
}

// ---------------------------------------------------------------- (a) TTF

/// TTF bytes → `fmd_font::Font::parse`.
struct TtfFontParse;

impl Target for TtfFontParse {
    fn name(&self) -> &'static str {
        "ttf_font_parse"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 1 << 20,
            // The parsed Font owns the input bytes; its size is bounded by
            // the input by construction.
            max_output_bytes: None,
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            fmd_font::bundled::CM_REGULAR.to_vec(),
            fmd_font::bundled::PLEX_REGULAR.to_vec(),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(10) {
            // Structure-aware: the table directory steers every lookup.
            0..=4 => mutate::corrupt_sfnt_directory(rng, input),
            5 => mutate::overwrite_u32be(rng, input),
            6 => mutate::flip_bit(rng, input),
            7 => mutate::splice_chunk(rng, input),
            8 => mutate::duplicate_chunk(rng, input, self.budgets().max_input_bytes),
            _ => mutate::truncate(rng, input),
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        match fmd_font::Font::parse(input.to_vec()) {
            Ok(_font) => Verdict::Accepted {
                output_bytes: input.len() as u64,
            },
            Err(e) => {
                let class = match &e {
                    fmd_font::FontError::BadMagic => "bad-magic".to_owned(),
                    fmd_font::FontError::MissingTable(table) => {
                        format!("missing-table-{}", table.to_lowercase())
                    }
                    fmd_font::FontError::Truncated => "truncated".to_owned(),
                    fmd_font::FontError::NoUnicodeCmap => "no-unicode-cmap".to_owned(),
                };
                Verdict::Refused {
                    class,
                    message: e.to_string(),
                }
            }
        }
    }
}

// ---------------------------------------------------------------- (b) YAML

/// YAML-subset strings → `fmn_config::yaml`, under the parser's declared
/// budgets.
struct YamlConfig;

/// Grammar-biased tokens: mapping/list structure, indentation, quoting,
/// scalars, and comment starts.
const YAML_TOKENS: &[&str] = &[
    ":",
    ": ",
    "- ",
    "  ",
    "    ",
    "\n",
    "\n\n",
    "#",
    "# note\n",
    "\"",
    "'",
    "key",
    "value",
    "fps",
    "pixel_width",
    "tex",
    "template",
    "a",
    "z9",
    "0",
    "144",
    "1.5",
    "-0.5",
    "1e3",
    "true",
    "False",
    "yes",
    "~",
    "[",
    "]",
    "{",
    "}",
    ",",
    "(",
    ")",
    "\"#ff00aa\"",
];

impl Target for YamlConfig {
    fn name(&self) -> &'static str {
        "yaml_config"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 65_536,
            // The Value tree is bounded by the input document by
            // construction (the parser's own Limits do the refusing).
            max_output_bytes: None,
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        [
            "pixel_width: 1920\npixel_height: 1080\nfps: 60\n",
            "tex:\n  template: default\nquality:\n  mode: high\n",
            "colors:\n  primary: \"#333333\"\n  enabled: true\n  ratio: 0.5\n  nothing: ~\n",
            "items:\n  - a\n  - b\nnested:\n  deep:\n    key: 1\n",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect()
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        if rng.chance(1, 24) {
            // Targeted depth-bomb: indentation nesting past the parser's
            // 32-level budget, exercising the depth refusal directly.
            let mut doc = String::new();
            let depth = 30 + rng.below(8);
            for level in 0..depth {
                for _ in 0..level {
                    doc.push_str("  ");
                }
                doc.push_str("k:\n");
            }
            *input = doc.into_bytes();
            return;
        }
        if rng.chance(1, 8) {
            // Whole-document replacement with grammar-biased soup.
            *input = mutate::token_soup(rng, YAML_TOKENS, 24).into_bytes();
            return;
        }
        if let Ok(text) = std::str::from_utf8(input) {
            let mut text = text.to_owned();
            mutate::token_splice(rng, &mut text, YAML_TOKENS);
            *input = text.into_bytes();
        } else {
            mutate::flip_bit(rng, input);
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        let Ok(src) = std::str::from_utf8(input) else {
            return Verdict::Refused {
                class: "non-utf8".to_owned(),
                message: "input is not UTF-8".to_owned(),
            };
        };
        match fmn_config::yaml::parse(src) {
            Ok((_value, _warnings)) => Verdict::Accepted {
                output_bytes: src.len() as u64,
            },
            Err(e) => {
                if e.line == 0 || e.col == 0 || e.message.is_empty() {
                    return Verdict::Fault {
                        message: format!("position-less or empty parse error: {e}"),
                    };
                }
                let class = if e.message.contains("byte budget") {
                    "over-byte-budget"
                } else if e.message.contains("level budget") {
                    "over-depth-budget"
                } else {
                    "syntax"
                };
                Verdict::Refused {
                    class: class.to_owned(),
                    message: e.to_string(),
                }
            }
        }
    }
}

// ---------------------------------------------------------------- (c) TeX

/// TeX strings → fmd-math through the real engine (preamble pack, layout,
/// the works).
struct TexMath;

/// The engine is expensive to construct (the fingerprint typesets a probe
/// set); one per process, shared by every case.
static TEX_ENGINE: LazyLock<fmn_tex::TexEngine> = LazyLock::new(|| {
    fmn_tex::TexEngine::new("fmd-math/pack/default", None).expect("bundled engine constructs")
});

fn tex_engine() -> &'static fmn_tex::TexEngine {
    &TEX_ENGINE
}

/// Grammar-biased fragments, after fmd-math's chaos-suite pool (the
/// in-crate deterministic half builds on it, per its own docs), trimmed to
/// the constructs that reach interesting parser states.
const TEX_TOKENS: &[&str] = &[
    "{",
    "}",
    "^",
    "_",
    "&",
    "~",
    "$",
    "'",
    "\\",
    " ",
    "\n",
    "%",
    "[",
    "]",
    "(",
    ")",
    "a",
    "x",
    "0",
    "9",
    "+",
    "-",
    "=",
    "|",
    "<",
    ">",
    "α",
    "→",
    "…",
    r"\frac",
    r"\sqrt",
    r"\left",
    r"\right",
    r"\over",
    r"\choose",
    r"\begin{matrix}",
    r"\end{matrix}",
    r"\text",
    r"\hat",
    r"\sum",
    r"\int",
    r"\limits",
    r"\displaystyle",
    r"\mathbb",
    r"\color",
    r"\Bigg",
    r"\not",
    r"\notarealcommand",
    r"\,",
    r"\",
    r"\sqrt[",
    r"\substack",
    r"\overbrace",
    r"\lim",
];

impl Target for TexMath {
    fn name(&self) -> &'static str {
        "tex_math"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 8192,
            // A typeset result is path bytes + span records over an ≤8 KiB
            // source; 32 MiB is a generous but real expansion bound.
            max_output_bytes: Some(1 << 25),
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        [
            r"\frac{a+b}{2}",
            r"\sqrt{x^2 + y^2}",
            r"\sum_{i=0}^{n} i = \frac{n(n+1)}{2}",
            r"\left( \frac{a}{b} \right)",
            r"\hat{x} + \vec{v} + \alpha",
        ]
        .iter()
        .map(|s| s.as_bytes().to_vec())
        .collect()
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        if rng.chance(1, 8) {
            *input = mutate::token_soup(rng, TEX_TOKENS, 20).into_bytes();
            return;
        }
        if let Ok(text) = std::str::from_utf8(input) {
            let mut text = text.to_owned();
            mutate::token_splice(rng, &mut text, TEX_TOKENS);
            *input = text.into_bytes();
        } else {
            mutate::overwrite_byte(rng, input);
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        let Ok(src) = std::str::from_utf8(input) else {
            return Verdict::Refused {
                class: "non-utf8".to_owned(),
                message: "input is not UTF-8".to_owned(),
            };
        };
        match tex_engine().typeset(fmn_tex::Mode::Math(fmn_tex::Style::Display), src) {
            Ok(typeset) => {
                // Where defined, Ok outputs re-encode deterministically:
                // the typeset codec round-trips bit-for-bit.
                let Ok(bytes) = typeset.to_bytes() else {
                    return Verdict::Refused {
                        class: "typeset-codec-resource".to_owned(),
                        message: "typeset result is not cache-encodable".to_owned(),
                    };
                };
                let deterministic = fmn_tex::Typeset::from_bytes(&bytes).is_ok_and(|again| {
                    again
                        .to_bytes()
                        .is_ok_and(|encoded_again| encoded_again == bytes)
                });
                if !deterministic {
                    return Verdict::Fault {
                        message: "typeset re-encode is not deterministic".to_owned(),
                    };
                }
                Verdict::Accepted {
                    output_bytes: bytes.len() as u64,
                }
            }
            Err(fmn_tex::TexError::Math(m)) => {
                let class = match &m {
                    // Tier-tagged and bounded to three classes — the precise
                    // construct name stays in the error message (asserted
                    // below) and the summary stats; per-name classes would
                    // bloat the committed corpus with one file per soup
                    // fragment.
                    fmd_math::MathError::UnsupportedCommand { name, .. } => {
                        match fmd_math::construct_status(name) {
                            fmd_math::ConstructStatus::UnsupportedT2 => "unsupported-t2",
                            fmd_math::ConstructStatus::Unknown => "unsupported-untiered",
                            fmd_math::ConstructStatus::Supported => "layout-pending",
                        }
                    }
                    fmd_math::MathError::Malformed { .. } => "malformed",
                    fmd_math::MathError::UnmappedChar { .. } => "unmapped-char",
                };
                Verdict::Refused {
                    class: class.to_owned(),
                    message: m.to_string(),
                }
            }
            // Pack/face/cache wiring faults are not input errors: any one
            // of them here is a campaign violation.
            Err(other) => Verdict::Fault {
                message: format!("engine fault: {other}"),
            },
        }
    }

    /// The never-garble bar: a precise fmd-math error names a construct
    /// (backtick-quoted) or a byte position (digits). All three standardized
    /// Display formats carry both; anything less is a violation.
    fn refusal_is_precise(&self, message: &str) -> bool {
        !message.is_empty()
            && (message.bytes().any(|b| b.is_ascii_digit()) || message.contains('`'))
    }
}

// ---------------------------------------------------------------- (d) PNG

/// PNG bytes → `fmn_codec::decode_png` under tight budgets.
struct CodecPng;

/// The campaign's declared decode budgets — deliberately tight so bombs
/// refuse fast and the accepted-output assertion is meaningful.
const PNG_LIMITS: fmn_codec::PngLimits = fmn_codec::PngLimits {
    max_pixels: 4096,
    max_chunks: 64,
};

impl CodecPng {
    /// Structure-aware PNG corruption: the IHDR dimensions, a chunk length
    /// field, header-territory bytes, or a generic mutation.
    fn corrupt(rng: &mut XorShift64, buf: &mut Vec<u8>) {
        const PNG_MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
        if buf.len() < 33 || !buf.starts_with(PNG_MAGIC) {
            match rng.below(3) {
                0 => mutate::flip_bit(rng, buf),
                1 => mutate::truncate(rng, buf),
                _ => mutate::splice_chunk(rng, buf),
            }
            return;
        }
        match rng.below(5) {
            0 => {
                // IHDR width/height (bytes 16..24): bomb steering.
                mutate::overwrite_u32be(rng, &mut buf[16..24]);
            }
            1 => {
                // A chunk length field: walk the chunk table, pick one.
                let mut offsets = Vec::new();
                let mut pos = 8_usize;
                while pos + 12 <= buf.len() && offsets.len() < 64 {
                    offsets.push(pos);
                    let len =
                        u32::from_be_bytes([buf[pos], buf[pos + 1], buf[pos + 2], buf[pos + 3]])
                            as usize;
                    let Some(next) = pos.checked_add(12usize.saturating_add(len)) else {
                        break;
                    };
                    if next > buf.len() {
                        break;
                    }
                    pos = next;
                }
                if offsets.is_empty() {
                    mutate::flip_bit(rng, buf);
                } else {
                    let at = offsets[rng.below(offsets.len() as u64) as usize];
                    mutate::overwrite_u32be(rng, &mut buf[at..at + 4]);
                }
            }
            2 => {
                let header_len = 64.min(buf.len());
                mutate::flip_bit(rng, &mut buf[..header_len]);
            }
            3 => mutate::truncate(rng, buf),
            _ => mutate::splice_chunk(rng, buf),
        }
    }
}

impl Target for CodecPng {
    fn name(&self) -> &'static str {
        "codec_png"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 1 << 21,
            max_output_bytes: Some(PNG_LIMITS.max_pixels * 4),
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        // One owned-encoder seed (deterministic by construction) and two
        // pinned decoder fixtures for chunk variety.
        let rgba: Vec<u8> = (0..8 * 8 * 4).map(|i| (i * 37 % 256) as u8).collect();
        vec![
            fmn_codec::encode_rgba8(8, 8, &rgba, fmn_codec::CompressionLevel::Fast),
            codec_fixture("png/rgb8.png"),
            codec_fixture("png/gray1.png"),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        Self::corrupt(rng, input);
    }

    fn run(&self, input: &[u8]) -> Verdict {
        match fmn_codec::decode_png(input, &PNG_LIMITS) {
            Ok(img) => {
                let pixels = u64::from(img.width) * u64::from(img.height);
                let expected = pixels * 4;
                if img.rgba.len() as u64 != expected {
                    return Verdict::Fault {
                        message: format!(
                            "decoded {}x{} but rgba is {} bytes (expected {expected})",
                            img.width,
                            img.height,
                            img.rgba.len()
                        ),
                    };
                }
                Verdict::Accepted {
                    output_bytes: img.rgba.len() as u64,
                }
            }
            Err(e) => Verdict::Refused {
                class: debug_variant_class(&e),
                message: e.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------- (d) JPEG

/// JPEG bytes → `fmn_codec::decode_jpeg` under tight budgets.
struct CodecJpeg;

const JPEG_LIMITS: fmn_codec::JpegLimits = fmn_codec::JpegLimits { max_pixels: 4096 };

impl CodecJpeg {
    /// Structure-aware JPEG corruption: marker id bytes (0xFF xx), the SOF
    /// dimension fields, or a generic mutation.
    fn corrupt(rng: &mut XorShift64, buf: &mut Vec<u8>) {
        if buf.len() < 4 || buf[0] != 0xFF || buf[1] != 0xD8 {
            match rng.below(3) {
                0 => mutate::flip_bit(rng, buf),
                1 => mutate::truncate(rng, buf),
                _ => mutate::splice_chunk(rng, buf),
            }
            return;
        }
        match rng.below(5) {
            0 => {
                // Rewrite a marker id byte (skip 0xFF 00 escapes and fills).
                let markers: Vec<usize> = buf
                    .windows(2)
                    .take(4096)
                    .enumerate()
                    .filter_map(|(i, w)| {
                        (w[0] == 0xFF && w[1] != 0x00 && w[1] != 0xFF).then_some(i + 1)
                    })
                    .collect();
                if markers.is_empty() {
                    mutate::flip_bit(rng, buf);
                } else {
                    let at = markers[rng.below(markers.len() as u64) as usize];
                    buf[at] = rng.byte();
                }
            }
            1 => mutate::overwrite_u32be(rng, buf),
            2 => mutate::flip_bit(rng, buf),
            3 => mutate::truncate(rng, buf),
            _ => mutate::splice_chunk(rng, buf),
        }
    }
}

impl Target for CodecJpeg {
    fn name(&self) -> &'static str {
        "codec_jpeg"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 1 << 21,
            max_output_bytes: Some(JPEG_LIMITS.max_pixels * 4),
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            codec_fixture("jpeg/gray.jpg"),
            codec_fixture("jpeg/baseline_444.jpg"),
            codec_fixture("jpeg/progressive_420.jpg"),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        Self::corrupt(rng, input);
    }

    fn run(&self, input: &[u8]) -> Verdict {
        match fmn_codec::decode_jpeg(input, &JPEG_LIMITS) {
            Ok(img) => {
                let pixels = u64::from(img.width) * u64::from(img.height);
                let expected = pixels * 4;
                if img.rgba.len() as u64 != expected {
                    return Verdict::Fault {
                        message: format!(
                            "decoded {}x{} but rgba is {} bytes (expected {expected})",
                            img.width,
                            img.height,
                            img.rgba.len()
                        ),
                    };
                }
                Verdict::Accepted {
                    output_bytes: img.rgba.len() as u64,
                }
            }
            Err(e) => Verdict::Refused {
                class: debug_variant_class(&e),
                message: e.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------- (e) canon

/// The canonical deserializer → `fmn_hash::fuzz_probe`. The contract IS
/// never-panic (the driver's `catch_unwind` asserts it); here we assert it
/// on structured mutations of a well-framed document.
struct CanonDeser;

impl CanonDeser {
    /// A well-framed document under the probe's fixed schema.
    fn seed_doc() -> Vec<u8> {
        let schema = fmn_hash::Schema::new(*b"FMNH", 0, 1, 0);
        let mut w = fmn_hash::Writer::new(schema);
        w.put_u32(0xdead_beef)
            .put_i64(-42)
            .put_f64(1.5)
            .put_bool(true)
            .put_str("franken")
            .put_bytes(&[1, 2, 3]);
        w.finish().expect("encode")
    }
}

impl Target for CanonDeser {
    fn name(&self) -> &'static str {
        "canon_deser"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 262_144,
            // The probe drains typed reads and returns a bool: zero output.
            max_output_bytes: Some(0),
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![Self::seed_doc()]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(6) {
            0 => mutate::flip_bit(rng, input),
            1 => mutate::overwrite_u32be(rng, input),
            2 => mutate::truncate(rng, input),
            3 => mutate::splice_chunk(rng, input),
            4 => mutate::duplicate_chunk(rng, input, self.budgets().max_input_bytes),
            _ => mutate::overwrite_byte(rng, input),
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        if fmn_hash::fuzz_probe(input) {
            Verdict::Accepted { output_bytes: 0 }
        } else {
            Verdict::Refused {
                class: "unframed".to_owned(),
                message: "not a well-framed FMNH/0.1.0 document".to_owned(),
            }
        }
    }
}

// ---------------------------------------------------------------- (g) OBJ

/// OBJ text → `fmn_library::obj_model::ObjMesh` under tight declared
/// budgets — the owned Wavefront subset reader (§12.4, fm-2u6) that
/// displaces trimesh/pywavefront for `ThreeDModel`.
struct ObjModel;

/// The campaign's declared parse budgets — deliberately tight so
/// over-budget input refuses fast and the accepted-mesh assertions are
/// meaningful.
const OBJ_LIMITS: fmn_library::obj_model::ObjLimits = fmn_library::obj_model::ObjLimits {
    max_vertices: 1 << 12,
    max_tex_coords: 1 << 12,
    max_normals: 1 << 12,
    max_triangles: 1 << 13,
};

impl ObjModel {
    /// Grammar-biased tokens: statements, corner forms, float and index
    /// spellings, comment starts, and the malformed staples.
    const TOKENS: &'static [&'static str] = &[
        "v ",
        "vn ",
        "vt ",
        "f ",
        "o ",
        "g ",
        "s ",
        "usemtl ",
        "mtllib ",
        "\n",
        "\n\n",
        "# note\n",
        " ",
        "  ",
        "/",
        "//",
        "-",
        "+",
        ".",
        "e",
        "E",
        "0",
        "1",
        "9",
        "42",
        "1.5",
        "-0.5",
        "1e3",
        ".5",
        "5.",
        "nan",
        "inf",
        "1,5",
        "v 0 0 0\n",
        "f 1 2 3\n",
        "1/2/3",
        "1//3",
        "1/2",
        "-1",
        "0",
        "99999",
        "\r\n",
        "\t",
    ];

    /// A tetrahedron with per-vertex normals.
    fn tetrahedron() -> Vec<u8> {
        b"# a tetrahedron\n\
          v 0.0 1.0 0.0\n\
          v -1.0 -1.0 1.0\n\
          v 1.0 -1.0 1.0\n\
          v 0.0 -1.0 -1.0\n\
          vn 1.0 0.0 0.0\n\
          vn 0.0 1.0 0.0\n\
          vn 0.0 0.0 1.0\n\
          vn -1.0 0.0 0.0\n\
          f 1//1 2//2 3//3\n\
          f 1//1 3//3 4//4\n\
          f 1//1 4//4 2//2\n\
          f 2//2 4//4 3//3\n"
            .to_vec()
    }
}

impl Target for ObjModel {
    fn name(&self) -> &'static str {
        "obj_model"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: 262_144,
            // The mesh is an owned parse tree bounded by the input by
            // construction (the parser's own ObjLimits do the refusing).
            max_output_bytes: None,
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            Self::tetrahedron(),
            // A quad face with texture coordinates (fan triangulation).
            b"v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\n"
                .to_vec(),
            // Negative (relative) indices and skipped statements.
            b"o thing\ng grp\ns off\nusemtl m\nmtllib lib.mtl\nv 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n"
                .to_vec(),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(7) {
            0 => {
                // Grammar token splice at a random offset.
                let token = Self::TOKENS[rng.below(Self::TOKENS.len() as u64) as usize];
                let at = rng.below(input.len() as u64 + 1) as usize;
                let room = (self.budgets().max_input_bytes as usize).saturating_sub(input.len());
                let take = token.len().min(room);
                input.splice(at..at, token[..take].bytes());
            }
            1 => {
                // Digit perturbation: coordinates and indices stay
                // well-spelled but change value (index-range steering).
                let digits: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.is_ascii_digit())
                    .map(|(i, _)| i)
                    .collect();
                if digits.is_empty() {
                    mutate::flip_bit(rng, input);
                } else {
                    let at = digits[rng.below(digits.len() as u64) as usize];
                    input[at] = b'0' + (rng.below(10) as u8);
                }
            }
            2 => {
                // Truncate at a random line boundary (byte 10 is '\n').
                let lines: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|&(_, b)| *b == 10)
                    .map(|(i, _)| i)
                    .collect();
                if lines.is_empty() {
                    mutate::truncate(rng, input);
                } else {
                    input.truncate(lines[rng.below(lines.len() as u64) as usize]);
                }
            }
            3 => {
                // Duplicate a random line (budget-bomb steering).
                let text = String::from_utf8_lossy(input).into_owned();
                let lines: Vec<&str> = text.lines().collect();
                if lines.is_empty() {
                    mutate::duplicate_chunk(rng, input, self.budgets().max_input_bytes);
                } else {
                    let line = lines[rng.below(lines.len() as u64) as usize];
                    let at = rng.below(input.len() as u64) as usize;
                    let mut insertion = line.as_bytes().to_vec();
                    insertion.push(b'\n');
                    let room =
                        (self.budgets().max_input_bytes as usize).saturating_sub(input.len());
                    insertion.truncate(room);
                    input.splice(at..at, insertion);
                }
            }
            4 => mutate::flip_bit(rng, input),
            5 => mutate::truncate(rng, input),
            _ => mutate::splice_chunk(rng, input),
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        match fmn_library::obj_model::ObjMesh::parse_with_limits(input, &OBJ_LIMITS) {
            Ok(mesh) => {
                // The parser's index-safety contract: every resolved
                // corner index is in range by construction.
                for triangle in &mesh.triangles {
                    for corner in triangle {
                        let bad = corner.vertex >= mesh.vertex_count()
                            || corner.normal.is_some_and(|n| n >= mesh.normals.len())
                            || corner.tex_coord.is_some_and(|t| t >= mesh.tex_coords.len());
                        if bad {
                            return Verdict::Fault {
                                message: format!(
                                    "corner index escaped its declared list: {corner:?}"
                                ),
                            };
                        }
                    }
                }
                #[allow(clippy::cast_possible_truncation)]
                let output_bytes = (mesh.vertex_count() * 24
                    + mesh.normals.len() * 24
                    + mesh.tex_coords.len() * 16
                    + mesh.triangle_count() * 36) as u64;
                Verdict::Accepted { output_bytes }
            }
            Err(e) => Verdict::Refused {
                class: debug_variant_class(&e),
                message: e.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------- (h) SVG

/// SVG text → `fmn_geom::SvgDocument` under tight declared budgets — the
/// hardened W2 document processor (§7.6, fm-6nm) that parses untrusted
/// user SVGs with an explicit accept/reject matrix.
struct SvgDocumentProcessor;

/// The campaign's declared parse budgets — deliberately tight so
/// over-budget input refuses fast and the accepted-document assertions are
/// meaningful.
const SVG_LIMITS: fmn_geom::SvgLimits = fmn_geom::SvgLimits {
    max_bytes: 1 << 18,
    max_depth: 24,
    max_path_commands: 1 << 12,
    max_use_expansions: 128,
    max_elements: 1 << 12,
};

impl SvgDocumentProcessor {
    /// Grammar-biased tokens: element and attribute spellings, path
    /// commands, entities, the DOCTYPE staple, and the malformed classics.
    const TOKENS: &'static [&'static str] = &[
        "<svg>",
        "</svg>",
        "<svg width=\"100\" height=\"100\">",
        "<g>",
        "</g>",
        "<g transform=\"translate(10 20) scale(2)\">",
        "<defs>",
        "</defs>",
        "<use href=\"#a\"/>",
        "<use xlink:href=\"#b\" x=\"5\" y=\"6\"/>",
        "<rect x=\"1\" y=\"2\" width=\"30\" height=\"40\" rx=\"5\"/>",
        "<circle cx=\"50\" cy=\"50\" r=\"20\"/>",
        "<ellipse cx=\"10\" cy=\"20\" rx=\"5\" ry=\"8\"/>",
        "<line x1=\"0\" y1=\"0\" x2=\"9\" y2=\"9\"/>",
        "<polyline points=\"0,0 1,1 2,0\"/>",
        "<polygon points=\"0 0 4 0 2 3\"/>",
        "<path d=\"M0 0 L10 0 10 10 Z\"/>",
        "<path d=\"m1 1 c2 3 4 5 6 7 s8 9 10 11 q1 2 3 4 t5 6 a1 1 0 0 1 7 7 z\"/>",
        "<path d=\"M0 0 A30 15 45 1 0 60 60\"/>",
        "id=\"a\"",
        "id=\"b\"",
        "viewBox=\"0 0 100 100\"",
        "preserveAspectRatio=\"none\"",
        "preserveAspectRatio=\"xMaxYMin slice\"",
        "fill=\"red\"",
        "fill=\"#00ff00\"",
        "fill=\"rgb(0 0 255 / 0.5)\"",
        "fill=\"url(#g)\"",
        "fill=\"none\"",
        "stroke=\"currentColor\"",
        "style=\"fill:blue;stroke-width:2;opacity:0.5;fill-rule:evenodd\"",
        "transform=\"rotate(45 5 5) skewX(10)\"",
        "stroke-dasharray=\"4 2\"",
        "<!DOCTYPE svg>",
        "<!doctype lolz [<!ENTITY lol \"lol\">]>",
        "&amp;",
        "&#65;",
        "&#x41;",
        "&bomb;",
        "href=\"http://evil.example/x.svg\">",
        "href=\"file:///etc/passwd\"",
        "xlink:href",
        "M",
        "L",
        "H",
        "V",
        "C",
        "S",
        "Q",
        "T",
        "A",
        "Z",
        "m",
        "a",
        "z",
        "nan",
        "inf",
        "1e999",
        "-",
        "+",
        ".",
        "0",
        "1",
        "99999",
        "<linearGradient id=\"g\"/>",
        "<pattern id=\"p\"/>",
        "clip-path=\"url(#c)\"",
        "mask=\"url(#m)\"",
        "<text>x</text>",
        "<style>.a{fill:red}</style>",
        "<!-- comment -->",
        "<![CDATA[x]]>",
        "<?xml version=\"1.0\"?>",
        "\n",
        " ",
        "<",
        ">",
        "/>",
        "\"",
        "=",
    ];

    /// A small real-world document: shapes, a nested transform, defs/use.
    fn icon() -> Vec<u8> {
        b"<svg width=\"100\" height=\"100\" viewBox=\"0 0 100 100\">\
          <defs><g id=\"badge\"><circle cx=\"0\" cy=\"0\" r=\"5\"/>\
          <rect x=\"-2\" y=\"-2\" width=\"4\" height=\"4\"/></g></defs>\
          <g transform=\"translate(10 20)\"><use href=\"#badge\" x=\"5\"/>\
          <path d=\"M10 10 A30 15 0 0 1 70 10 Z\" fill=\"#ff0000\"/>\
          <rect x=\"5\" y=\"50\" width=\"20\" height=\"20\" rx=\"3\" \
          style=\"fill:blue;stroke:black;stroke-width:2\"/></g></svg>"
            .to_vec()
    }
}

impl Target for SvgDocumentProcessor {
    fn name(&self) -> &'static str {
        "svg_document_processor"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            max_input_bytes: SVG_LIMITS.max_bytes as u64,
            // The document is an owned structure bounded by the input by
            // construction (the parser's own SvgLimits do the refusing).
            max_output_bytes: None,
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            Self::icon(),
            // Compact path-heavy document: every command, relative+absolute.
            b"<svg><path d=\"M0 0 L1 1 H2 V3 C1 2 3 4 5 6 S7 8 9 10 Q11 12 13 14 \
              T15 16 A1 1 0 0 1 20 20 Z m1 1 l2 2 h3 v4 c1 1 2 2 3 3 s4 4 5 5 \
              q1 1 2 2 t3 3 a2 2 30 1 0 4 4 z\"/></svg>"
                .to_vec(),
            // Cascade + viewBox document.
            b"<svg width=\"200\" height=\"100\" viewBox=\"0 0 100 100\" \
              preserveAspectRatio=\"xMidYMid meet\" fill=\"green\">\
              <g fill-opacity=\"0.5\" opacity=\"0.8\" fill-rule=\"evenodd\">\
              <ellipse cx=\"50\" cy=\"50\" rx=\"40\" ry=\"20\"/>\
              <polygon points=\"10,10 90,10 50,90\"/></g></svg>"
                .to_vec(),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(7) {
            0 => {
                // Grammar token splice at a random offset.
                let token = Self::TOKENS[rng.below(Self::TOKENS.len() as u64) as usize];
                let at = rng.below(input.len() as u64 + 1) as usize;
                let room = (self.budgets().max_input_bytes as usize).saturating_sub(input.len());
                let take = token.len().min(room);
                input.splice(at..at, token[..take].bytes());
            }
            1 => {
                // Digit perturbation: coordinates and counts stay
                // well-spelled but change value (budget steering).
                let digits: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.is_ascii_digit())
                    .map(|(i, _)| i)
                    .collect();
                if digits.is_empty() {
                    mutate::flip_bit(rng, input);
                } else {
                    let at = digits[rng.below(digits.len() as u64) as usize];
                    input[at] = b'0' + (rng.below(10) as u8);
                }
            }
            2 => {
                // Truncate at a random tag boundary (byte 62 is '>').
                let tags: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|&(_, b)| *b == 62)
                    .map(|(i, _)| i)
                    .collect();
                if tags.is_empty() {
                    mutate::truncate(rng, input);
                } else {
                    input.truncate(tags[rng.below(tags.len() as u64) as usize]);
                }
            }
            3 => {
                // Duplicate a random element span (budget-bomb steering).
                let text = String::from_utf8_lossy(input).into_owned();
                let lines: Vec<&str> = text.lines().collect();
                if lines.is_empty() {
                    mutate::duplicate_chunk(rng, input, self.budgets().max_input_bytes);
                } else {
                    let line = lines[rng.below(lines.len() as u64) as usize];
                    let at = rng.below(input.len() as u64) as usize;
                    let mut insertion = line.as_bytes().to_vec();
                    insertion.push(b'\n');
                    let room =
                        (self.budgets().max_input_bytes as usize).saturating_sub(input.len());
                    insertion.truncate(room);
                    input.splice(at..at, insertion);
                }
            }
            4 => mutate::flip_bit(rng, input),
            5 => mutate::truncate(rng, input),
            _ => mutate::splice_chunk(rng, input),
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        match fmn_geom::SvgDocument::parse_with_limits(input, &SVG_LIMITS) {
            Ok(doc) => {
                // The processor's finiteness contract: every emitted point
                // and style scalar is finite, in the z=0 plane, by
                // construction.
                if !doc.width.is_finite() || !doc.height.is_finite() {
                    return Verdict::Fault {
                        message: "viewport dimension is not finite".to_owned(),
                    };
                }
                let mut output_bytes: u64 = 64;
                for shape in &doc.shapes {
                    for p in shape.path.points() {
                        if !p[0].is_finite() || !p[1].is_finite() || p[2] != 0.0 {
                            return Verdict::Fault {
                                message: format!("non-finite point escaped: {p:?}"),
                            };
                        }
                    }
                    if !shape.style.opacity.is_finite() || !shape.style.stroke_width.is_finite() {
                        return Verdict::Fault {
                            message: "non-finite style scalar escaped".to_owned(),
                        };
                    }
                    output_bytes += (shape.path.num_points() * 24
                        + shape.style.stroke_dasharray.len() * 8
                        + 128) as u64;
                }
                Verdict::Accepted { output_bytes }
            }
            Err(e) => Verdict::Refused {
                class: debug_variant_class(&e),
                message: e.to_string(),
            },
        }
    }
}

// ---------------------------------------------------------------- (i) boolean

/// Bytes → `fmn_geom::boolean::fuzz_probe`: two arbitrary quadratic paths
/// through every admitted boolean route and the certified fallback under
/// tight budgets. The contract IS never-panic (the driver's
/// `catch_unwind` asserts it) plus totality: every byte string is either
/// computed or refused with a typed budget/validation error inside the
/// probe — there is no input class that can hang or fault (fm-qjy).
struct PathBoolean;

impl PathBoolean {
    /// Deterministic seed of `selector` followed by `len - 1` structured
    /// body bytes (4-byte chunks become quadratic control points).
    fn seed_bytes(selector: u8, len: usize, multiplier: u8) -> Vec<u8> {
        let mut bytes = vec![selector];
        for index in 1..len {
            bytes.push(
                (index as u8)
                    .wrapping_mul(multiplier)
                    .wrapping_add(selector),
            );
        }
        bytes
    }
}

impl Target for PathBoolean {
    fn name(&self) -> &'static str {
        "path_boolean"
    }

    fn budgets(&self) -> Budgets {
        Budgets {
            // The probe reads one selector byte plus at most 258 body
            // bytes; presenting more only wastes mutations.
            max_input_bytes: 259,
            // The probe drains to a bool: zero output by construction.
            max_output_bytes: Some(0),
        }
    }

    fn seeds(&self) -> Vec<Vec<u8>> {
        vec![
            Self::seed_bytes(0, 259, 73),
            Self::seed_bytes(3, 129, 41),
            Self::seed_bytes(7, 258, 17),
        ]
    }

    fn mutate(&self, rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(7) {
            0 => mutate::flip_bit(rng, input),
            // Selector scramble: operation and fill-rule bits.
            1 => {
                if input.is_empty() {
                    input.push(rng.byte());
                } else {
                    input[0] = rng.byte();
                }
            }
            2 => mutate::overwrite_byte(rng, input),
            3 => mutate::splice_chunk(rng, input),
            4 => mutate::duplicate_chunk(rng, input, self.budgets().max_input_bytes),
            5 => mutate::truncate(rng, input),
            _ => mutate::overwrite_u32be(rng, input),
        }
    }

    fn run(&self, input: &[u8]) -> Verdict {
        if fmn_geom::boolean::fuzz_probe(input) {
            Verdict::Accepted { output_bytes: 0 }
        } else {
            Verdict::Fault {
                message: "boolean fuzz probe breached its totality contract".to_owned(),
            }
        }
    }
}

// ---------------------------------------------------------------- registry

/// The campaign registry: every target with its spec. The manifest must
/// record exactly these rows.
fn registry() -> Vec<(Box<dyn Target>, CampaignSpec)> {
    vec![
        (
            Box::new(TtfFontParse),
            CampaignSpec {
                seed: 0x7474_665f_3100_0001,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
        (
            Box::new(YamlConfig),
            CampaignSpec {
                seed: 0x9a1c_001f_3200_0002,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
        (
            Box::new(TexMath),
            CampaignSpec {
                seed: 0x7e58_0001_3300_0003,
                ci_cases: 200,
                full_cases: 5000,
            },
        ),
        (
            Box::new(CodecPng),
            CampaignSpec {
                seed: 0x00c0_de01_3400_0004,
                ci_cases: 400,
                full_cases: 10_000,
            },
        ),
        (
            Box::new(CodecJpeg),
            CampaignSpec {
                seed: 0x00c0_de02_3500_0005,
                ci_cases: 400,
                full_cases: 10_000,
            },
        ),
        (
            Box::new(CanonDeser),
            CampaignSpec {
                seed: 0xcafe_0001_3600_0006,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
        (
            Box::new(ObjModel),
            CampaignSpec {
                seed: 0x0b1e_0001_3700_0007,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
        (
            Box::new(SvgDocumentProcessor),
            CampaignSpec {
                seed: 0x5bde_0001_3800_0008,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
        (
            Box::new(PathBoolean),
            CampaignSpec {
                seed: 0xb001_0001_3900_0009,
                ci_cases: 600,
                full_cases: 20_000,
            },
        ),
    ]
}

/// The pending-note list: empty now that the SVG target has landed
/// (fm-6nm). Kept as the mechanism for future not-yet-landed targets.
const PENDING: &[(&str, &str)] = &[];

fn pending_manifest_records() -> Vec<(String, String)> {
    let mut pending: Vec<_> = PENDING
        .iter()
        .map(|(name, note)| ((*name).to_owned(), (*note).to_owned()))
        .collect();
    pending.sort_by(|a, b| a.0.cmp(&b.0));
    pending
}

fn manifest_row(
    target: &dyn Target,
    spec: &CampaignSpec,
    report: &fuzz::CampaignReport,
) -> ManifestRow {
    let budgets = target.budgets();
    ManifestRow {
        target: target.name().to_owned(),
        seed: spec.seed,
        ci_cases: spec.ci_cases,
        full_cases: spec.full_cases,
        max_input_bytes: budgets.max_input_bytes,
        max_output_bytes: budgets.max_output_bytes,
        classes: report.classes(),
    }
}

/// Run one target, asserting the campaign invariants. Returns the report.
fn run_checked(target: &dyn Target, spec: &CampaignSpec, cases: u32) -> fuzz::CampaignReport {
    let report = fuzz::run_campaign(target, spec, cases);
    assert!(
        report.violations.is_empty(),
        "{}: campaign violations (repro inputs dumped under \
         CARGO_TARGET_TMPDIR/fuzz_violations/):",
        target.name()
    );
    println!("{}", report.summary_line(target.name()));
    report
}

// ---------------------------------------------------------------- tests

#[test]
fn authority_reader_checks_limit_before_text_validation() {
    let oversized_invalid = std::io::Cursor::new(vec![0xff_u8; 9]);
    let result = read_utf8_bounded(oversized_invalid, "synthetic authority", 8);
    assert!(
        matches!(&result, Err(error) if error.contains("exceeds the 8-byte resource limit") && !error.contains("UTF-8")),
        "oversized input was not rejected before text validation: {result:?}"
    );

    assert_eq!(
        read_utf8_bounded(std::io::Cursor::new(b"12345678"), "exact authority", 8),
        Ok("12345678".to_owned()),
        "an authority exactly at the byte limit must remain readable"
    );
}

/// Driver determinism: identical (seed, case count) ⇒ identical reports,
/// down to the interesting-input bytes. This is what makes the committed
/// corpus checkable at all.
#[test]
fn campaign_is_deterministic() {
    for (target, spec) in registry() {
        let a = fuzz::run_campaign(&*target, &spec, 64);
        let b = fuzz::run_campaign(&*target, &spec, 64);
        assert_eq!(
            (a.cases_run, &a.class_counts, &a.interesting, &a.violations),
            (b.cases_run, &b.class_counts, &b.interesting, &b.violations),
            "{}: campaign is not deterministic",
            target.name()
        );
    }
}

/// Corpus persistence is monotonic: historical inputs remain live regression
/// cases even when an upstream capability promotion changes their outcome
/// class. Their boundedness, content address, and current target contract are
/// checked on every ordinary gate run.
#[test]
fn committed_corpus_replays_against_current_targets() {
    for (target, _) in registry() {
        replay_committed_corpus(&*target);
    }
}

/// The CI gate: reduced case counts, every violation fails, observed
/// classes and corpus entries must be subsets of the committed full
/// campaign's, and the manifest must agree with the registry.
#[test]
fn ci_campaign_matches_manifest_and_corpus() {
    if std::env::var_os("FMN_FUZZ_BLESS").is_some() {
        eprintln!("bless mode: CI check skipped (the full campaign is rewriting the corpus)");
        return;
    }
    let manifest_path = corpus_root().join("MANIFEST.tsv");
    let manifest_text = read_manifest(&manifest_path);
    let manifest = fuzz::parse_manifest(&manifest_text).expect("manifest parses");
    assert_eq!(
        manifest.pending,
        pending_manifest_records(),
        "manifest pending records match the declared pending-target authority"
    );

    let registry = registry();
    assert_eq!(
        manifest.rows.len(),
        registry.len(),
        "manifest rows match the target registry"
    );

    for (target, spec) in &registry {
        let row = manifest
            .rows
            .iter()
            .find(|r| r.target == target.name())
            .expect("manifest row exists (count checked above)");
        let budgets = target.budgets();
        assert_eq!(
            (
                row.seed,
                row.ci_cases,
                row.full_cases,
                row.max_input_bytes,
                row.max_output_bytes
            ),
            (
                spec.seed,
                spec.ci_cases,
                spec.full_cases,
                budgets.max_input_bytes,
                budgets.max_output_bytes
            ),
            "{}: manifest row disagrees with the registry spec",
            target.name()
        );

        let report = run_checked(&**target, spec, spec.ci_cases);
        for class in report.classes() {
            assert!(
                row.classes.contains(&class),
                "{}: class {class:?} not in the manifest's recorded classes {:?} \
                 (re-bless with FMN_FUZZ_FULL=1 FMN_FUZZ_BLESS=1)",
                target.name(),
                row.classes
            );
        }

        let expected = fuzz::expected_corpus(&report);
        let drift = fuzz::check_corpus(&corpus_root().join(target.name()), &expected, false)
            .expect("corpus readable");
        assert!(
            drift.is_empty(),
            "{}: corpus drift: {drift:?} (re-bless with FMN_FUZZ_FULL=1 FMN_FUZZ_BLESS=1)",
            target.name()
        );
    }
}

/// The scheduled full campaign (`FMN_FUZZ_FULL=1`): the authority for the
/// manifest's classes and newly generated corpus representatives. Retained
/// historical inputs are replayed by [`committed_corpus_replays_against_current_targets`]. With
/// `FMN_FUZZ_BLESS=1` it rewrites both for human review and commit.
#[test]
fn full_campaign_is_the_campaign_authority() {
    if std::env::var_os("FMN_FUZZ_FULL").is_none() {
        eprintln!("full campaign skipped (set FMN_FUZZ_FULL=1 — the scheduled campaign)");
        return;
    }
    let bless = std::env::var_os("FMN_FUZZ_BLESS").is_some();
    let registry = registry();

    let mut rows = Vec::new();
    let mut reports = Vec::new();
    for (target, spec) in &registry {
        let report = run_checked(&**target, spec, spec.full_cases);
        rows.push(manifest_row(&**target, spec, &report));
        reports.push((target.name().to_owned(), report));
    }

    if bless {
        let pending = pending_manifest_records();
        let rendered_manifest = fuzz::render_manifest(&rows, &pending)
            .expect("render bounded canonical MANIFEST.tsv before blessing files");
        for (name, report) in &reports {
            let stale =
                fuzz::bless_corpus(&corpus_root().join(name), &fuzz::expected_corpus(report))
                    .expect("bless corpus");
            if !stale.is_empty() {
                println!("{name}: retained historical corpus files: {stale:?}");
            }
        }
        std::fs::write(corpus_root().join("MANIFEST.tsv"), rendered_manifest)
            .expect("write MANIFEST.tsv");
        println!("fuzz campaign blessed — review the diff and commit it (the rig never commits)");
        return;
    }

    let manifest_path = corpus_root().join("MANIFEST.tsv");
    let manifest_text = read_manifest(&manifest_path);
    let manifest = fuzz::parse_manifest(&manifest_text).expect("manifest parses");
    assert_eq!(
        manifest.rows.len(),
        rows.len(),
        "full-campaign manifest row count matches the target registry"
    );
    assert_eq!(
        manifest.pending,
        pending_manifest_records(),
        "full-campaign manifest pending records match the declared authority"
    );
    for row in &rows {
        let committed = manifest
            .rows
            .iter()
            .find(|r| r.target == row.target)
            .expect("manifest row exists (row count checked below)");
        assert_eq!(
            row, committed,
            "{}: full campaign disagrees with the manifest (re-bless)",
            row.target
        );
    }
    for (name, report) in &reports {
        let drift = fuzz::check_corpus(
            Path::new(&corpus_root()).join(name).as_path(),
            &fuzz::expected_corpus(report),
            false,
        )
        .expect("corpus readable");
        assert!(
            drift.is_empty(),
            "{name}: full-campaign corpus drift: {drift:?} (re-bless)"
        );
    }
}
