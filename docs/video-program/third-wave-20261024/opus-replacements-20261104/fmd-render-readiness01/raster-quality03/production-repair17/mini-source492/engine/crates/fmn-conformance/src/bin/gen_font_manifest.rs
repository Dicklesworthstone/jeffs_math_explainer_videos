//! Generates the font+license bundle manifest and the shipped license
//! inventory (fm-aef, §15.3/§16.7).
//!
//! ```text
//! cargo run -p fmn-conformance --bin gen_font_manifest [--check] [--repo DIR] [--fmd-font DIR]
//! ```
//!
//! Generation mode (default): locates the pinned `fmd-font` sources (the
//! SUITE.lock `franken_markdown` rev under `$CARGO_HOME/git/checkouts`,
//! overridable with `--fmd-font`), copies every license set's OFL text to
//! `dist/licenses/fonts/<slug>-OFL.txt`, copies the engine's MIT+rider
//! `LICENSE` to `dist/licenses/LICENSE`, and writes the canonical
//! `dist/FONT_BUNDLE.json`.
//!
//! `--check`: verifies without writing — the committed manifest must be
//! byte-identical to regeneration from the committed license files, the
//! shipped OFL texts must be byte-identical to the pinned upstream copies,
//! and the shipped engine license must equal the repository `LICENSE`.
//! Exit codes: 0 ok, 64 usage, 65 drift/data, 74 io.

#![forbid(unsafe_code)]

use fmn_conformance::font_bundle::{
    ENGINE_LICENSE_PATH, FontBundleError, LICENSE_SLUGS, MANIFEST_PATH, build_manifest, ofl_path,
    render_manifest, suite_lock_pin,
};
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

const EXIT_USAGE: u8 = 64;
const EXIT_DATA: u8 = 65;
const EXIT_IO: u8 = 74;
const MAX_SUITE_LOCK_BYTES: u64 = 1024 * 1024;
const MAX_LICENSE_BYTES: u64 = 256 * 1024;
const MAX_MANIFEST_BYTES: u64 = 4 * 1024 * 1024;
const MAX_CHECKOUT_CANDIDATES: usize = 64;
const MAX_DIAGNOSTIC_VALUE_BYTES: usize = 512;

struct Cli {
    check: bool,
    repo: PathBuf,
    fmd_font: Option<PathBuf>,
}

#[derive(Debug)]
enum CliError {
    Usage(String),
    Data(String),
    Io(String),
}

impl CliError {
    fn exit_code(&self) -> u8 {
        match self {
            Self::Usage(_) => EXIT_USAGE,
            Self::Data(_) => EXIT_DATA,
            Self::Io(_) => EXIT_IO,
        }
    }
}

impl std::fmt::Display for CliError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Usage(m) | Self::Data(m) | Self::Io(m) => f.write_str(m),
        }
    }
}

fn parse_args(args: &[String]) -> Result<Cli, CliError> {
    let mut check = false;
    let mut repo = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .map_err(|e| CliError::Io(format!("locating the repository root: {e}")))?;
    let mut fmd_font = None;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--check" => check = true,
            "--repo" => {
                index += 1;
                repo = args
                    .get(index)
                    .map(PathBuf::from)
                    .ok_or_else(|| CliError::Usage("--repo needs a directory".to_owned()))?;
            }
            "--fmd-font" => {
                index += 1;
                fmd_font =
                    Some(args.get(index).map(PathBuf::from).ok_or_else(|| {
                        CliError::Usage("--fmd-font needs a directory".to_owned())
                    })?);
            }
            other => {
                return Err(CliError::Usage(format!(
                    "unknown argument '{other}'; usage: gen_font_manifest [--check] [--repo DIR] \
                     [--fmd-font DIR]"
                )));
            }
        }
        index += 1;
    }
    Ok(Cli {
        check,
        repo,
        fmd_font,
    })
}

/// Reads SUITE.lock's `franken_markdown` pin.
fn suite_pin(repo: &Path) -> Result<String, CliError> {
    let text = read_utf8_bounded(&repo.join("SUITE.lock"), "SUITE.lock", MAX_SUITE_LOCK_BYTES)?;
    suite_lock_pin(&text, "franken_markdown")
        .ok_or_else(|| CliError::Data("SUITE.lock pins no franken_markdown row".to_owned()))
}

/// Locates the pinned `fmd-font` crate directory inside the cargo git
/// checkouts (the checkout directory is named by a short prefix of the
/// pinned revision).
fn discover_fmd_font(rev: &str) -> Result<PathBuf, CliError> {
    let cargo_home = std::env::var_os("CARGO_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cargo")))
        .ok_or_else(|| {
            CliError::Usage("neither CARGO_HOME nor HOME is set; pass --fmd-font".to_owned())
        })?;
    let checkouts = cargo_home.join("git").join("checkouts");
    let entries = fs::read_dir(&checkouts)
        .map_err(|e| CliError::Io(format!("reading {}: {e}", checkouts.display())))?;
    let mut candidates = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|error| {
            CliError::Io(format!(
                "reading checkout entry under {}: {error}",
                bounded_diagnostic(&checkouts.to_string_lossy())
            ))
        })?;
        let name = entry.file_name();
        if !name.to_string_lossy().starts_with("franken_markdown-") {
            continue;
        }
        let checkout = entry.path();
        let revs = fs::read_dir(&checkout).map_err(|error| {
            CliError::Io(format!(
                "reading franken_markdown checkout {}: {error}",
                bounded_diagnostic(&checkout.to_string_lossy())
            ))
        })?;
        for rev_dir in revs {
            let rev_dir = rev_dir.map_err(|error| {
                CliError::Io(format!(
                    "reading revision entry under {}: {error}",
                    bounded_diagnostic(&checkout.to_string_lossy())
                ))
            })?;
            let short = rev_dir.file_name().to_string_lossy().into_owned();
            if short.len() >= 7 && rev.starts_with(&short) {
                let candidate = rev_dir.path().join("fmd-font");
                if candidate.is_dir() {
                    candidates.push(candidate);
                    if candidates.len() > MAX_CHECKOUT_CANDIDATES {
                        return Err(CliError::Data(format!(
                            "more than {MAX_CHECKOUT_CANDIDATES} fmd-font checkouts match rev {} \
                             under {}; pass --fmd-font",
                            bounded_diagnostic(rev),
                            bounded_diagnostic(&checkouts.to_string_lossy())
                        )));
                    }
                }
            }
        }
    }
    match candidates.len() {
        1 => candidates
            .into_iter()
            .next()
            .ok_or_else(|| CliError::Data("no fmd-font candidate".to_owned())),
        0 => Err(CliError::Data(format!(
            "no fmd-font checkout for franken_markdown rev {} under {}; run `cargo fetch` or \
             pass --fmd-font",
            bounded_diagnostic(rev),
            bounded_diagnostic(&checkouts.to_string_lossy())
        ))),
        _ => Err(CliError::Data(format!(
            "ambiguous fmd-font checkouts for rev {}: {} candidates under {}; pass --fmd-font",
            bounded_diagnostic(rev),
            candidates.len(),
            bounded_diagnostic(&checkouts.to_string_lossy())
        ))),
    }
}

fn bounded_diagnostic(value: &str) -> String {
    if value.len() <= MAX_DIAGNOSTIC_VALUE_BYTES {
        return value.to_owned();
    }
    let mut end = MAX_DIAGNOSTIC_VALUE_BYTES.saturating_sub(3);
    while !value.is_char_boundary(end) {
        end = end.saturating_sub(1);
    }
    format!("{}...", &value[..end])
}

fn read_bytes_bounded(reader: impl Read, what: &str, max_bytes: u64) -> Result<Vec<u8>, CliError> {
    let read_limit = max_bytes
        .checked_add(1)
        .ok_or_else(|| CliError::Data(format!("invalid byte limit for {what}")))?;
    let mut bytes = Vec::new();
    reader
        .take(read_limit)
        .read_to_end(&mut bytes)
        .map_err(|error| CliError::Io(format!("reading {what}: {error}")))?;
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(CliError::Data(format!(
            "{what} exceeds the {max_bytes}-byte input limit"
        )));
    }
    Ok(bytes)
}

fn read_bounded(path: &Path, what: &str, max_bytes: u64) -> Result<Vec<u8>, CliError> {
    let file = fs::File::open(path)
        .map_err(|error| CliError::Io(format!("opening {what} ({}): {error}", path.display())))?;
    read_bytes_bounded(file, what, max_bytes)
}

fn read_utf8_bounded(path: &Path, what: &str, max_bytes: u64) -> Result<String, CliError> {
    decode_utf8(read_bounded(path, what, max_bytes)?, what)
}

fn decode_utf8(bytes: Vec<u8>, what: &str) -> Result<String, CliError> {
    String::from_utf8(bytes).map_err(|error| {
        CliError::Data(format!(
            "{what} is not UTF-8 at byte {}",
            error.utf8_error().valid_up_to()
        ))
    })
}

fn validate_output(bytes: &[u8], what: &str, max_bytes: u64) -> Result<(), CliError> {
    if u64::try_from(bytes.len()).unwrap_or(u64::MAX) > max_bytes {
        return Err(CliError::Data(format!(
            "{what} exceeds the {max_bytes}-byte output limit"
        )));
    }
    Ok(())
}

fn write_bounded(path: &Path, bytes: &[u8], what: &str, max_bytes: u64) -> Result<(), CliError> {
    validate_output(bytes, what, max_bytes)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| CliError::Io(format!("creating {}: {e}", parent.display())))?;
    }
    fs::write(path, bytes).map_err(|e| CliError::Io(format!("writing {}: {e}", path.display())))
}

/// Gathers the OFL texts `(slug, bytes)` from a directory of upstream
/// `fonts/<slug>/OFL.txt` files.
fn gather_upstream_ofl(fmd_font: &Path) -> Result<Vec<(&'static str, Vec<u8>)>, CliError> {
    LICENSE_SLUGS
        .iter()
        .map(|slug| {
            let path = fmd_font.join("fonts").join(slug).join("OFL.txt");
            read_bounded(&path, "upstream OFL text", MAX_LICENSE_BYTES).map(|bytes| (*slug, bytes))
        })
        .collect()
}

fn run(cli: &Cli) -> Result<String, CliError> {
    let rev = suite_pin(&cli.repo)?;
    let fmd_font = match &cli.fmd_font {
        Some(dir) => dir.clone(),
        None => discover_fmd_font(&rev)?,
    };
    let upstream_ofl = gather_upstream_ofl(&fmd_font)?;
    let engine_license = read_bounded(
        &cli.repo.join("LICENSE"),
        "the engine LICENSE",
        MAX_LICENSE_BYTES,
    )?;
    let dist = cli.repo.join("dist");

    // The manifest is built from what will ship in generation mode or from
    // the committed copies in check mode. Check mode verifies those copies
    // against the pinned upstream bytes below.
    let committed_ofl: Vec<(&'static str, Vec<u8>)> = if cli.check {
        LICENSE_SLUGS
            .iter()
            .map(|slug| {
                read_bounded(
                    &dist.join(ofl_path(slug)),
                    "shipped OFL text",
                    MAX_LICENSE_BYTES,
                )
                .map(|bytes| (*slug, bytes))
            })
            .collect::<Result<_, _>>()?
    } else {
        Vec::new()
    };
    if cli.check {
        for ((slug, committed_bytes), (_, upstream_bytes)) in
            committed_ofl.iter().zip(upstream_ofl.iter())
        {
            if committed_bytes != upstream_bytes {
                return Err(CliError::Data(format!(
                    "shipped {} drifted from the pinned upstream copy (rev {rev}); regenerate \
                     with `cargo run -p fmn-conformance --bin gen_font_manifest`",
                    ofl_path(slug)
                )));
            }
        }
    }
    let ofl_refs: Vec<(&str, &[u8])> = if cli.check {
        committed_ofl
            .iter()
            .map(|(s, b)| (*s, b.as_slice()))
            .collect()
    } else {
        upstream_ofl
            .iter()
            .map(|(s, b)| (*s, b.as_slice()))
            .collect()
    };

    if cli.check {
        let shipped_engine = read_bounded(
            &dist.join(ENGINE_LICENSE_PATH),
            "shipped engine LICENSE",
            MAX_LICENSE_BYTES,
        )?;
        if shipped_engine != engine_license {
            return Err(CliError::Data(format!(
                "shipped {ENGINE_LICENSE_PATH} != repository LICENSE; regenerate"
            )));
        }
    }

    let faces: Vec<(&str, &[u8])> = fmd_font::bundled::ALL_FACES.to_vec();
    let manifest = build_manifest(&rev, &faces, &ofl_refs, &engine_license)
        .map_err(|e: FontBundleError| CliError::Data(e.to_string()))?;
    let rendered = render_manifest(&manifest);
    validate_output(
        rendered.as_bytes(),
        "rendered font bundle manifest",
        MAX_MANIFEST_BYTES,
    )?;

    let mut summary = String::new();
    if cli.check {
        let committed = read_bounded(
            &cli.repo.join(MANIFEST_PATH),
            "the committed manifest",
            MAX_MANIFEST_BYTES,
        )?;
        if committed != rendered.as_bytes() {
            return Err(CliError::Data(format!(
                "{MANIFEST_PATH} drifted from the bundled faces at SUITE.lock rev {rev}; \
                 regenerate with `cargo run -p fmn-conformance --bin gen_font_manifest`"
            )));
        }
        let _ = writeln!(
            summary,
            "ok: {MANIFEST_PATH} matches {} faces + {} license texts at rev {rev}",
            manifest.faces.len(),
            manifest.licenses.len()
        );
    } else {
        // Validate the complete generated document before the first filesystem
        // mutation, then ship byte-identical license sources and the manifest.
        for (slug, bytes) in &upstream_ofl {
            let dest = dist.join(ofl_path(slug));
            write_bounded(&dest, bytes, "shipped OFL text", MAX_LICENSE_BYTES)?;
            let _ = writeln!(
                summary,
                "shipped {} ({} bytes)",
                dest.display(),
                bytes.len()
            );
        }
        let engine_dest = dist.join(ENGINE_LICENSE_PATH);
        write_bounded(
            &engine_dest,
            &engine_license,
            "shipped engine LICENSE",
            MAX_LICENSE_BYTES,
        )?;
        let _ = writeln!(
            summary,
            "shipped {} ({} bytes)",
            engine_dest.display(),
            engine_license.len()
        );
        let dest = cli.repo.join(MANIFEST_PATH);
        write_bounded(
            &dest,
            rendered.as_bytes(),
            "rendered font bundle manifest",
            MAX_MANIFEST_BYTES,
        )?;
        let _ = writeln!(
            summary,
            "wrote {} ({} faces, {} license texts, rev {rev})",
            dest.display(),
            manifest.faces.len(),
            manifest.licenses.len()
        );
    }
    Ok(summary)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match parse_args(&args).and_then(|cli| run(&cli)) {
        Ok(summary) => {
            print!("{summary}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("gen_font_manifest: {error}");
            ExitCode::from(error.exit_code())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Error, ErrorKind};

    struct RefusingReader;

    impl Read for RefusingReader {
        fn read(&mut self, _buffer: &mut [u8]) -> std::io::Result<usize> {
            Err(Error::new(ErrorKind::PermissionDenied, "refused"))
        }
    }

    #[test]
    fn bounded_reader_accepts_the_limit_and_refuses_limit_plus_one() {
        let exact = read_bytes_bounded(Cursor::new([b'x'; 8]), "fixture", 8)
            .expect("exact boundary is valid");
        assert_eq!(exact, [b'x'; 8]);

        let error = read_bytes_bounded(Cursor::new([b'x'; 9]), "fixture", 8)
            .expect_err("limit-plus-one must be refused");
        assert!(matches!(error, CliError::Data(_)));
        assert_eq!(error.to_string(), "fixture exceeds the 8-byte input limit");
    }

    #[test]
    fn bounded_reader_keeps_io_failures_distinct_from_data_refusals() {
        let error = read_bytes_bounded(RefusingReader, "fixture", 8)
            .expect_err("reader failure must be preserved");
        assert!(matches!(error, CliError::Io(_)));
        assert_eq!(error.exit_code(), EXIT_IO);
    }

    #[test]
    fn invalid_utf8_is_a_data_refusal_with_a_stable_offset() {
        let error = decode_utf8(vec![b'a', 0xff, b'b'], "SUITE.lock")
            .expect_err("invalid UTF-8 must be refused as data");
        assert!(matches!(error, CliError::Data(_)));
        assert_eq!(error.exit_code(), EXIT_DATA);
        assert_eq!(error.to_string(), "SUITE.lock is not UTF-8 at byte 1");
    }

    #[test]
    fn rendered_output_is_bounded_before_any_writer_is_called() {
        validate_output(&[b'x'; 8], "manifest", 8).expect("exact boundary is valid");
        let error =
            validate_output(&[b'x'; 9], "manifest", 8).expect_err("limit-plus-one must be refused");
        assert!(matches!(error, CliError::Data(_)));
        assert_eq!(error.exit_code(), EXIT_DATA);
        assert_eq!(
            error.to_string(),
            "manifest exceeds the 8-byte output limit"
        );
    }

    #[test]
    fn ambiguity_diagnostics_have_a_fixed_text_envelope() {
        let diagnostic = bounded_diagnostic(&"é".repeat(MAX_DIAGNOSTIC_VALUE_BYTES));
        assert!(diagnostic.len() <= MAX_DIAGNOSTIC_VALUE_BYTES);
        assert!(diagnostic.ends_with("..."));
        assert!(diagnostic.is_char_boundary(diagnostic.len()));
    }
}
