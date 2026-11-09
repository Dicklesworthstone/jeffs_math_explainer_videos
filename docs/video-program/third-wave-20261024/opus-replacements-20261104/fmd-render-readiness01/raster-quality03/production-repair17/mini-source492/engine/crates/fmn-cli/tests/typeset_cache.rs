//! The shipped `fmn` binary's persistent typeset cache and pre-play preflight,
//! each run in a fresh process (fm-typeset-cache-preflight-wiring-1sn0).
//!
//! Every render here uses the `formula_sheet.v1` builtin: twenty static
//! display formulas declared as the scene's preflight manifest. Each test owns
//! its cache root, so nothing touches the user's real cache, and every claim
//! is read back from the render's own robot record and published frames.
#![cfg(feature = "batch")]
#![forbid(unsafe_code)]
#![allow(clippy::expect_used, clippy::panic)]

use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};

/// The formula sheet's twenty formulas plus the Tex scale-calibration probe
/// the preflight warms with them.
const SHEET_LAYOUTS: u64 = 21;
const SHEET_FORMULAS: u64 = 20;

static NEXT_FIXTURE: AtomicU64 = AtomicU64::new(0);

struct Fixture {
    root: PathBuf,
}

impl Fixture {
    fn new() -> Self {
        for _ in 0..1_024 {
            let sequence = NEXT_FIXTURE.fetch_add(1, Ordering::Relaxed);
            let root = std::env::temp_dir().join(format!(
                "fmn-typeset-cache-{}-{sequence}",
                std::process::id()
            ));
            match fs::create_dir(&root) {
                Ok(()) => return Self { root },
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => panic!("could not create typeset-cache fixture {root:?}: {error}"),
            }
        }
        panic!("could not allocate a unique typeset-cache fixture");
    }

    /// The fixture's home directory, spelled as the temp dir is. On macOS
    /// that runs through the `/var` -> `/private/var` link, and the
    /// platform-convention cache base below it must still be persistent
    /// (fm-macos-var-symlink-gate-aqr1).
    fn home(&self) -> PathBuf {
        let home = self.root.join("home");
        fs::create_dir_all(&home).expect("fixture home");
        home
    }

    fn cache(&self) -> PathBuf {
        self.root.join("cache")
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

/// One fresh `fmn` process whose home is the fixture's own directory, so no
/// user config or cache is read. With `home` false there is no home at all.
fn fmn_with_home(fixture: &Fixture, args: &[OsString], home: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    command
        .args(args)
        .current_dir(&fixture.root)
        .env_remove("XDG_CONFIG_HOME")
        .env_remove("XDG_CACHE_HOME")
        .env_remove("HOME")
        .env_remove("APPDATA")
        .env_remove("LOCALAPPDATA")
        .env_remove("USERPROFILE")
        .stdin(Stdio::null());
    if home {
        // The store's protected-path check needs a home to compare against.
        command.env(
            if cfg!(windows) { "USERPROFILE" } else { "HOME" },
            fixture.home(),
        );
    }
    command.output().expect("the shipped fmn binary runs")
}

fn fmn(fixture: &Fixture, args: &[OsString]) -> Output {
    fmn_with_home(fixture, args, true)
}

/// What one render reported and published.
struct Render {
    record: String,
    frames: Vec<Vec<u8>>,
}

impl Render {
    fn typesetting(&self) -> &str {
        let start = self
            .record
            .find("\"typesetting\":{")
            .unwrap_or_else(|| panic!("render record has no typesetting object: {}", self.record));
        &self.record[start..]
    }

    fn count(&self, field: &str) -> u64 {
        number(self.typesetting(), field)
            .unwrap_or_else(|| panic!("typesetting.{field} missing: {}", self.typesetting()))
    }

    fn flag(&self, field: &str) -> bool {
        let prefix = format!("\"{field}\":");
        let tail = self
            .typesetting()
            .split_once(&prefix)
            .unwrap_or_else(|| panic!("typesetting.{field} missing"))
            .1;
        tail.starts_with("true")
    }

    fn closure_digest(&self) -> &str {
        let tail = self
            .record
            .split_once("\"closure_digest\":\"")
            .expect("certified render names its closure digest")
            .1;
        &tail[..tail.find('"').expect("closed digest string")]
    }
}

fn number(record: &str, field: &str) -> Option<u64> {
    let prefix = format!("\"{field}\":");
    let tail = record.split_once(&prefix)?.1;
    let digits = tail.bytes().take_while(u8::is_ascii_digit).count();
    (digits != 0).then(|| tail[..digits].parse().ok()).flatten()
}

/// Render the formula sheet as a certified PNG sequence in a fresh process.
fn render(fixture: &Fixture, output: &str, threads: &str, extra: &[OsString]) -> Render {
    render_with_home(fixture, output, threads, extra, true)
}

fn render_with_home(
    fixture: &Fixture,
    output: &str,
    threads: &str,
    extra: &[OsString],
    home: bool,
) -> Render {
    let video_dir = fixture.root.join(output);
    let mut args: Vec<OsString> = [
        "--robot",
        "--reproducible",
        "--format",
        "png_sequence",
        "--resolution",
        "160x90",
        "--fps",
        "8",
        "--threads",
        threads,
    ]
    .into_iter()
    .map(OsString::from)
    .collect();
    args.extend(extra.iter().cloned());
    args.push("--video_dir".into());
    args.push(video_dir.clone().into_os_string());
    args.push("@builtin".into());
    args.push("formula_sheet.v1".into());
    let output = fmn_with_home(fixture, &args, home);
    let stdout = String::from_utf8(output.stdout).expect("robot stdout is UTF-8");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "formula sheet render failed: {stdout} {stderr}"
    );
    let record = stdout
        .lines()
        .find(|line| line.contains("\"kind\":\"render\""))
        .unwrap_or_else(|| panic!("no render record in {stdout}"))
        .to_owned();
    let sequence = video_dir.join("formula_sheet.v1");
    let mut paths: Vec<PathBuf> = fs::read_dir(&sequence)
        .expect("the PNG sequence directory exists")
        .map(|entry| entry.expect("directory entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
        .collect();
    paths.sort();
    assert!(!paths.is_empty(), "the formula sheet published frames");
    let frames = paths
        .iter()
        .map(|path| fs::read(path).expect("read published frame"))
        .collect();
    Render { record, frames }
}

fn cache_dir_args(path: &Path) -> Vec<OsString> {
    vec!["--cache-dir".into(), path.as_os_str().to_owned()]
}

/// Every published typeset object under a cache root.
fn typeset_objects(root: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries {
            let path = entry.expect("cache entry").path();
            if path.is_dir() {
                walk(&path, out);
            } else if path.components().any(|part| part.as_os_str() == "objects") {
                out.push(path);
            }
        }
    }
    let mut out = Vec::new();
    walk(&root.join("ns").join("typeset"), &mut out);
    out.sort();
    out
}

#[test]
fn a_warm_second_process_serves_every_formula_from_disk_with_identical_certified_bits() {
    let fixture = Fixture::new();
    let cache = cache_dir_args(&fixture.cache());

    let cold = render(&fixture, "cold", "2", &cache);
    assert!(cold.flag("persistent"), "{}", cold.typesetting());
    assert_eq!(cold.count("disk_hits"), 0);
    assert_eq!(cold.count("misses"), SHEET_LAYOUTS);
    let written = cold.count("bytes_written");
    assert!(written > 0, "the cold run publishes its layouts");
    assert_eq!(
        typeset_objects(&fixture.cache()).len() as u64,
        SHEET_LAYOUTS
    );

    // A fresh process: nothing survives but the persistent store.
    let warm = render(&fixture, "warm", "2", &cache);
    assert_eq!(
        warm.count("disk_hits"),
        SHEET_LAYOUTS,
        "{}",
        warm.typesetting()
    );
    assert_eq!(warm.count("misses"), 0, "no formula is laid out again");
    assert_eq!(warm.count("bytes_read"), written);
    assert_eq!(warm.count("bytes_written"), 0);
    assert_eq!(warm.count("rejected"), 0);
    assert_eq!(warm.count("store_errors"), 0);

    // Under --reproducible a hit is bit-identical to a miss, and warming the
    // cache never changes the certified closure.
    assert_eq!(warm.frames.len(), cold.frames.len());
    assert!(warm.frames == cold.frames, "hit and miss frames differ");
    assert_eq!(warm.closure_digest(), cold.closure_digest());

    // Nor does where the cache lives: another root (cold there) renders the
    // same frames under the same closure, because the resolved config (C4)
    // carries `directories.cache` only as an inert marker.
    let elsewhere = render(
        &fixture,
        "elsewhere",
        "2",
        &cache_dir_args(&fixture.root.join("another-cache")),
    );
    assert_eq!(elsewhere.count("misses"), SHEET_LAYOUTS);
    assert!(
        elsewhere.frames == cold.frames,
        "a cache root changed the frames"
    );
    assert_eq!(
        elsewhere.closure_digest(),
        cold.closure_digest(),
        "a cache root changed the certified closure"
    );
}

#[test]
fn twenty_static_formulas_are_typeset_on_several_workers_before_the_first_frame() {
    let fixture = Fixture::new();
    let render = render(&fixture, "out", "4", &cache_dir_args(&fixture.cache()));
    assert_eq!(
        render.count("requests"),
        SHEET_FORMULAS,
        "{}",
        render.typesetting()
    );
    assert_eq!(render.count("batches"), 1);
    // The preflight did every layout of the run before construct() began:
    // construction laid out nothing, and nothing was laid out in play. (The
    // sheet builds every formula before its first play, so "none in play"
    // alone would hold without a preflight; "none in construct" would not.)
    assert_eq!(render.count("misses"), SHEET_LAYOUTS);
    assert_eq!(render.count("layouts_before_construct"), SHEET_LAYOUTS);
    assert_eq!(render.count("layouts_before_first_frame"), SHEET_LAYOUTS);
    assert_eq!(render.count("layouts_inside_play"), 0);
    let parallelism = std::thread::available_parallelism().map_or(1, usize::from);
    if parallelism >= 2 {
        assert!(render.count("workers") >= 2, "{}", render.typesetting());
        assert!(
            render.count("active_workers") >= 2,
            "{}",
            render.typesetting()
        );
    }
    assert!(render.count("wall_ns") > 0);

    // The preflight ceiling follows the render's thread budget.
    let serial = self::render(
        &fixture,
        "serial",
        "1",
        &cache_dir_args(&fixture.root.join("c1")),
    );
    assert_eq!(serial.count("workers"), 1);
    assert_eq!(serial.count("layouts_inside_play"), 0);
    assert!(
        serial.frames == render.frames,
        "worker count changed the frames"
    );
}

#[test]
fn a_corrupt_entry_is_detected_recomputed_and_republished_across_processes() {
    let fixture = Fixture::new();
    let cache = cache_dir_args(&fixture.cache());
    let cold = render(&fixture, "cold", "2", &cache);

    let objects = typeset_objects(&fixture.cache());
    assert_eq!(objects.len() as u64, SHEET_LAYOUTS);
    let victim = &objects[objects.len() / 2];
    let mut bytes = fs::read(victim).expect("read a cache object");
    let middle = bytes.len() / 2;
    bytes[middle] ^= 0x5a;
    fs::write(victim, &bytes).expect("tamper with a cache object");

    let recovering = render(&fixture, "recovering", "2", &cache);
    assert_eq!(
        recovering.count("rejected"),
        1,
        "{}",
        recovering.typesetting()
    );
    // A corrupt entry is evicted and republished: no read or write failed.
    assert_eq!(recovering.count("store_errors"), 0);
    assert_eq!(recovering.count("disk_hits"), SHEET_LAYOUTS - 1);
    assert_eq!(recovering.count("misses"), 1);
    assert!(
        recovering.count("bytes_written") > 0,
        "the entry is republished"
    );
    assert!(
        recovering.frames == cold.frames,
        "a corrupt entry changed a frame"
    );

    let healed = render(&fixture, "healed", "2", &cache);
    assert_eq!(healed.count("rejected"), 0);
    assert_eq!(healed.count("disk_hits"), SHEET_LAYOUTS);
    assert_eq!(healed.count("misses"), 0);
    assert!(healed.frames == cold.frames);
}

#[test]
fn an_attached_store_that_cannot_read_or_write_is_reported_and_changes_no_frame() {
    let fixture = Fixture::new();
    let cache = cache_dir_args(&fixture.cache());
    let cold = render(&fixture, "cold", "2", &cache);
    assert_eq!(cold.count("store_errors"), 0, "{}", cold.typesetting());

    // The typeset namespace's object directory becomes a plain file. The root
    // is still owned and opens, so the store attaches, but no entry can be
    // read or published: what a cache directory that turned read-only or was
    // replaced looks like (and, unlike a permission bit, also for root).
    let objects = fixture
        .cache()
        .join("ns")
        .join("typeset")
        .join(format!("v{}", fmn::tex::TYPESET_FORMAT_VERSION))
        .join("objects");
    fs::rename(&objects, objects.with_file_name("objects.moved")).expect("move objects aside");
    fs::write(&objects, b"not a directory").expect("plant a file where objects were");

    let broken = render(&fixture, "broken", "2", &cache);
    // Attached and no open error, yet not caching: every one of the 21
    // requests failed one read and one write, and the record says so.
    assert!(broken.flag("persistent"), "{}", broken.typesetting());
    assert!(
        broken.typesetting().contains("\"cache_error\":null"),
        "{}",
        broken.typesetting()
    );
    assert_eq!(broken.count("store_errors"), 2 * SHEET_LAYOUTS);
    assert_eq!(broken.count("disk_hits"), 0);
    assert_eq!(broken.count("misses"), SHEET_LAYOUTS);
    assert_eq!(broken.count("bytes_written"), 0);
    assert_eq!(broken.count("rejected"), 0);
    assert!(
        broken.frames == cold.frames,
        "a store that cannot cache changed a frame"
    );
}

#[test]
fn clear_cache_empties_the_store_and_the_next_render_is_cold_with_the_same_bits() {
    let fixture = Fixture::new();
    let cache = cache_dir_args(&fixture.cache());
    let before = render(&fixture, "before", "2", &cache);
    assert_eq!(
        typeset_objects(&fixture.cache()).len() as u64,
        SHEET_LAYOUTS
    );

    let mut clear: Vec<OsString> = vec!["--clear-cache".into()];
    clear.extend(cache.iter().cloned());
    clear.push("--robot".into());
    let cleared = fmn(&fixture, &clear);
    let stdout = String::from_utf8_lossy(&cleared.stdout);
    assert!(cleared.status.success(), "{stdout}");
    assert!(stdout.contains("\"kind\":\"cache_clear\""), "{stdout}");
    assert!(stdout.contains("\"outcome\":\"cleared\""), "{stdout}");
    assert!(
        typeset_objects(&fixture.cache()).is_empty(),
        "the store is empty"
    );

    let after = render(&fixture, "after", "2", &cache);
    assert_eq!(after.count("disk_hits"), 0, "{}", after.typesetting());
    assert_eq!(after.count("misses"), SHEET_LAYOUTS);
    assert!(
        after.frames == before.frames,
        "--clear-cache changed a render"
    );
    assert_eq!(
        typeset_objects(&fixture.cache()).len() as u64,
        SHEET_LAYOUTS
    );
}

#[test]
fn the_config_files_cache_directory_is_the_store_the_render_uses() {
    let fixture = Fixture::new();
    let configured = fixture.root.join("configured-cache");
    let config = fixture.root.join("fmn.yml");
    let text = format!(
        "directories:\n  cache: \"{}\"\n",
        configured.to_str().expect("UTF-8 fixture path")
    );
    fs::write(&config, text).expect("write config file");
    let args = vec!["--config_file".into(), config.into_os_string()];

    let cold = render(&fixture, "cold", "2", &args);
    assert!(cold.flag("persistent"), "{}", cold.typesetting());
    assert_eq!(typeset_objects(&configured).len() as u64, SHEET_LAYOUTS);
    let warm = render(&fixture, "warm", "2", &args);
    assert_eq!(warm.count("disk_hits"), SHEET_LAYOUTS);
    assert!(warm.frames == cold.frames);
}

#[test]
fn an_unavailable_platform_cache_degrades_to_memory_and_says_why() {
    let fixture = Fixture::new();
    // No --cache-dir and no HOME/XDG/LOCALAPPDATA: the platform convention
    // has no trustworthy base, so the store refuses to guess.
    let render = render_with_home(&fixture, "out", "2", &[], false);
    assert!(!render.flag("persistent"), "{}", render.typesetting());
    assert!(
        !render.typesetting().contains("\"cache_error\":null"),
        "the unavailable cache is reported: {}",
        render.typesetting()
    );
    assert_eq!(render.count("misses"), SHEET_LAYOUTS);
    assert!(!fixture.root.join(".cache").exists());
}

#[test]
fn without_a_cache_dir_the_render_uses_the_platform_convention_under_home() {
    let fixture = Fixture::new();
    let cold = render(&fixture, "cold", "2", &[]);
    assert!(cold.flag("persistent"), "{}", cold.typesetting());
    let home = fixture.home();
    let root = if cfg!(target_os = "macos") {
        home.join("Library").join("Caches").join("franken-manim")
    } else if cfg!(windows) {
        home.join("AppData").join("Local").join("franken-manim")
    } else {
        home.join(".cache").join("franken-manim")
    };
    assert_eq!(
        typeset_objects(&root).len() as u64,
        SHEET_LAYOUTS,
        "{root:?}"
    );
    let warm = render(&fixture, "warm", "2", &[]);
    assert_eq!(warm.count("disk_hits"), SHEET_LAYOUTS);
    assert!(warm.frames == cold.frames);
}
