//! A native scene owns one real engine; disk failures never substitute ink.

use std::path::{Path, PathBuf};
use std::sync::Arc;

use fmn_config::Config;
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::{FileSystem, VirtualFs};
use fmn_tex::{Mode, Style, TexError, TexSession, TypesetSessionReport};

fn root() -> &'static str {
    if cfg!(windows) {
        r"C:\typeset"
    } else {
        "/typeset"
    }
}

fn config() -> Config {
    let mut config = Config::resolve(&[], None).unwrap().config;
    config.directories.cache = root().to_owned();
    config
}

fn session(config: &Config, fs: &Arc<VirtualFs>) -> TexSession {
    TexSession::with_cache(config, fs.clone(), Arc::new(FakeClock::new()))
}

fn bytes(session: &TexSession, source: &str) -> Vec<u8> {
    session
        .engine()
        .unwrap()
        .typeset(Mode::Math(Style::Display), source)
        .unwrap()
        .to_bytes()
        .unwrap()
}

#[test]
fn construction_and_reports_are_lazy_and_the_engine_is_reused() {
    let fs = Arc::new(VirtualFs::new());
    let session = session(&config(), &fs);
    assert_eq!(session.report(), TypesetSessionReport::default());
    assert!(!fs.exists(Path::new(root())));
    let first = session.engine().unwrap();
    assert!(std::ptr::eq(first, session.engine().unwrap()));
    assert!(session.report().initialized);
    assert!(session.report().persistent);
    assert!(session.report().cache_error.is_none());
    assert!(fs.exists(Path::new(root())));
}

#[test]
fn a_fresh_session_uses_verified_disk_layouts_then_the_memory_front() {
    let fs = Arc::new(VirtualFs::new());
    let config = config();
    let source = r"\frac{x^2+1}{2}";
    let cold = session(&config, &fs);
    let expected = bytes(&cold, source);
    assert_eq!(cold.report().layout_computations, 1);
    assert_eq!(cold.report().persistent_hits, 0);
    drop(cold);

    let warm = session(&config, &fs);
    assert_eq!(bytes(&warm, source), expected);
    assert_eq!(warm.report().layout_computations, 0);
    assert_eq!(warm.report().persistent_hits, 1);
    assert_eq!(bytes(&warm, source), expected);
    assert_eq!(warm.report().memory.hits, 1);
    assert_eq!(warm.report().persistent_hits, 1);

    let memory = TexSession::default();
    assert_eq!(bytes(&memory, source), expected);
    assert!(!memory.report().persistent);
    assert_eq!(memory.report().layout_computations, 1);
}

#[test]
fn a_foreign_cache_directory_is_not_claimed_and_layout_still_succeeds() {
    let fs = Arc::new(VirtualFs::new());
    let foreign = Path::new(root()).join("unrelated.txt");
    fs.insert(&foreign, b"leave this alone".to_vec());
    let before = fs.list_dir(Path::new(root())).unwrap();
    let session = session(&config(), &fs);
    assert!(!bytes(&session, "x").is_empty());
    let report = session.report();
    assert!(report.initialized);
    assert!(!report.persistent);
    assert!(report.cache_error.is_some());
    assert_eq!(report.layout_computations, 1);
    assert_eq!(fs.read(&foreign).unwrap(), b"leave this alone");
    assert_eq!(fs.list_dir(Path::new(root())).unwrap(), before);
}

#[test]
fn injected_filesystems_never_resolve_ambient_cache_paths() {
    let escaping = PathBuf::from(root())
        .join("..")
        .join("escape")
        .to_string_lossy()
        .into_owned();
    for directory in [String::new(), "relative-cache".to_owned(), escaping] {
        let fs = Arc::new(VirtualFs::new());
        let mut config = config();
        config.directories.cache = directory;
        let session = session(&config, &fs);
        assert!(!bytes(&session, "x").is_empty());
        assert!(!session.report().persistent);
        assert!(
            session
                .report()
                .cache_error
                .unwrap()
                .contains("explicit absolute")
        );
        assert!(!fs.exists(Path::new(root())));
    }
}

#[test]
fn template_refusals_precede_cache_io_instead_of_falling_back_to_default() {
    let fs = Arc::new(VirtualFs::new());
    let mut config = config();
    config.tex.template = "not-a-native-template".to_owned();
    let session = session(&config, &fs);
    assert!(matches!(session.engine(), Err(TexError::Pack(_))));
    assert_eq!(session.report(), TypesetSessionReport::default());
    assert!(!fs.exists(Path::new(root())));
}

#[test]
fn the_selected_template_reaches_the_engine_and_empty_means_default() {
    for (template, pack) in [
        ("basic", "fmd-math/pack/basic"),
        ("", "fmd-math/pack/default"),
    ] {
        let session = TexSession::memory(template);
        let reference = fmn_tex::TexEngine::new(pack, None).unwrap();
        let expected = reference
            .typeset(Mode::Math(Style::Display), "x+1")
            .unwrap()
            .to_bytes()
            .unwrap();
        assert_eq!(bytes(&session, "x+1"), expected);
        assert!(!session.report().persistent);
    }
}

fn object_files(fs: &VirtualFs, dir: &Path, out: &mut Vec<PathBuf>) {
    if let Ok(children) = fs.list_dir(dir) {
        for child in children {
            if fs.read(&child).is_ok() {
                if child.components().any(|part| part.as_os_str() == "objects") {
                    out.push(child);
                }
            } else {
                object_files(fs, &child, out);
            }
        }
    }
}

#[test]
fn reports_count_disk_bytes_and_recover_a_corrupt_entry_by_recomputing_it() {
    let fs = Arc::new(VirtualFs::new());
    let config = config();
    let sources = [r"\frac{a}{b}", r"\sqrt{x+1}", r"\sum_{n=1}^{N} n"];
    let cold = session(&config, &fs);
    let expected: Vec<Vec<u8>> = sources.iter().map(|s| bytes(&cold, s)).collect();
    let report = cold.report();
    assert_eq!(report.layout_computations, 3);
    assert_eq!(
        (report.persistent_hits, report.persistent_bytes_read),
        (0, 0)
    );
    assert_eq!(report.cache_misses(), 3);
    let written = report.persistent_bytes_written;
    assert_eq!(
        written,
        expected.iter().map(|b| b.len() as u64).sum::<u64>(),
        "every fresh layout's encoded payload is published"
    );
    assert_eq!(report.persistent_rejected, 0);
    drop(cold);

    // Flip one byte in the middle of one published entry.
    let mut objects = Vec::new();
    object_files(&fs, Path::new(root()), &mut objects);
    objects.sort();
    assert_eq!(objects.len(), 3, "one object per formula: {objects:?}");
    let victim = &objects[1];
    let mut tampered = fs.read(victim).unwrap();
    let middle = tampered.len() / 2;
    tampered[middle] ^= 0x20;
    fs.write_atomic(victim, &tampered).unwrap();

    // Detected and recomputed: identical bytes, two hits, one layout.
    let recovering = session(&config, &fs);
    let recovered: Vec<Vec<u8>> = sources.iter().map(|s| bytes(&recovering, s)).collect();
    assert_eq!(
        recovered, expected,
        "a corrupt entry never changes a layout"
    );
    let report = recovering.report();
    assert_eq!(report.persistent_rejected, 1, "the corruption is reported");
    assert_eq!(report.persistent_hits, 2);
    assert_eq!(report.layout_computations, 1);
    assert_eq!(report.cache_hits(), 2);
    assert!(report.persistent_bytes_read > 0);
    assert!(
        report.persistent_bytes_written > 0,
        "the entry is republished"
    );
    drop(recovering);

    // Healed: the next fresh session hits all three.
    let healed = session(&config, &fs);
    let again: Vec<Vec<u8>> = sources.iter().map(|s| bytes(&healed, s)).collect();
    assert_eq!(again, expected);
    let report = healed.report();
    assert_eq!((report.persistent_hits, report.layout_computations), (3, 0));
    assert_eq!(report.persistent_rejected, 0);
    assert_eq!(report.persistent_bytes_read, written);
    assert_eq!(report.persistent_bytes_written, 0);
}

#[test]
fn segment_accounting_separates_layouts_before_the_first_frame_from_play() {
    let session = TexSession::default();
    // A lazy session stays lazy: segment notes alone report nothing.
    session.note_first_frame();
    session.note_segment_begin();
    session.note_segment_end();
    assert_eq!(session.report(), TypesetSessionReport::default());

    let session = TexSession::default();
    bytes(&session, "a+b");
    session.note_construct_begin();
    bytes(&session, "c+d");
    // Only the first construct boundary is recorded.
    session.note_construct_begin();
    session.note_first_frame();
    // Only the first frame's boundary is recorded.
    bytes(&session, "k+l");
    session.note_first_frame();
    session.note_segment_begin();
    session.note_segment_end();
    // Typesetting between segments (construct code) is not inside play.
    bytes(&session, "e+f");
    session.note_segment_begin();
    bytes(&session, "g+h");
    session.note_segment_end();
    session.note_segment_begin();
    bytes(&session, "i+j");
    // An open segment counts too; a later begin closes it.
    assert_eq!(session.report().layouts_inside_segments, 2);
    session.note_segment_begin();
    session.note_segment_end();
    let report = session.report();
    assert_eq!(report.layouts_before_construct, Some(1));
    assert_eq!(report.layouts_before_first_frame, Some(2));
    assert_eq!(report.layouts_inside_segments, 2);
    assert_eq!(report.layout_computations, 6);
}
