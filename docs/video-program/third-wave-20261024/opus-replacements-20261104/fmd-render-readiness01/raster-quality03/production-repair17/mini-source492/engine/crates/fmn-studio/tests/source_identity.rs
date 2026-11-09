//! Generation identity reuses the production bounded source scanner.
use fmn_studio::project_watch::{SourceWatch, WatchFilter, WatchLimits};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;

static NEXT: AtomicU64 = AtomicU64::new(0);
struct Project(PathBuf);
impl Project {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fmn-source-identity-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn snapshot(&self, extra: &[PathBuf]) -> SourceWatch {
        let mut roots = vec![self.0.clone()];
        roots.extend_from_slice(extra);
        SourceWatch::new_filtered(
            roots,
            Vec::new(),
            Duration::from_millis(100),
            WatchLimits::default(),
            WatchFilter {
                extensions: vec!["py".into(), "pyw".into()],
                ignored_directories: vec!["__pycache__".into()],
            },
        )
        .unwrap()
    }
}
impl Drop for Project {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn same_length_helper_edits_change_generation_identity_not_mtime_touches() {
    let project = Project::new();
    let helper = project.0.join("helper.py");
    fs::write(&helper, "VALUE=1").unwrap();
    let before = project.snapshot(&[]).content_fingerprint();
    fs::write(&helper, "VALUE=1").unwrap();
    assert_eq!(before, project.snapshot(&[]).content_fingerprint());
    let modified = fs::metadata(&helper).unwrap().modified().unwrap();
    fs::write(&helper, "VALUE=2").unwrap();
    fs::File::options()
        .write(true)
        .open(&helper)
        .unwrap()
        .set_modified(modified)
        .unwrap();
    assert_ne!(before, project.snapshot(&[]).content_fingerprint());
}

#[test]
fn explicit_assets_are_hashed_but_generated_frames_are_not() {
    let project = Project::new();
    let asset = project.0.join("values.csv");
    fs::write(&asset, "old").unwrap();
    let before = project
        .snapshot(std::slice::from_ref(&asset))
        .content_fingerprint();
    fs::write(project.0.join("render.png"), "output").unwrap();
    assert_eq!(
        before,
        project
            .snapshot(std::slice::from_ref(&asset))
            .content_fingerprint()
    );
    fs::write(&asset, "new").unwrap();
    assert_ne!(before, project.snapshot(&[asset]).content_fingerprint());
}

#[test]
fn files_refer_to_exactly_the_same_frozen_scan_as_the_fingerprint() {
    let project = Project::new();
    let source = project.0.join("scene.py");
    fs::write(&source, "one").unwrap();
    let watch = project.snapshot(&[]);
    let identity = watch.content_fingerprint();
    fs::write(&source, "two").unwrap();
    assert_eq!(identity, watch.content_fingerprint());
    let rows: Vec<_> = watch.source_files().collect();
    assert_eq!(rows.len(), 1);
    assert_eq!(
        rows[0],
        (source.as_path(), fmn_studio::protocol_digest(b"one"))
    );
    assert_ne!(identity, project.snapshot(&[]).content_fingerprint());
}

#[test]
fn paths_and_missing_file_kinds_are_part_of_the_identity() {
    let project = Project::new();
    let absent = project.0.join("later.csv");
    let before = project
        .snapshot(std::slice::from_ref(&absent))
        .content_fingerprint();
    fs::write(&absent, "").unwrap();
    let present = project
        .snapshot(std::slice::from_ref(&absent))
        .content_fingerprint();
    assert_ne!(before, present);
    fs::write(project.0.join("different.csv"), "").unwrap();
    assert_ne!(
        present,
        project
            .snapshot(&[project.0.join("different.csv")])
            .content_fingerprint()
    );
}

#[test]
fn reading_identity_does_not_consume_a_debounced_reload() {
    let project = Project::new();
    let source = project.0.join("scene.py");
    fs::write(&source, "one").unwrap();
    let mut watch = project.snapshot(&[]);
    let first = watch.content_fingerprint();
    fs::write(&source, "two").unwrap();
    assert!(!watch.poll(Duration::ZERO).unwrap());
    let changed = watch.content_fingerprint();
    assert_ne!(first, changed);
    assert!(!watch.poll(Duration::from_millis(99)).unwrap());
    assert_eq!(changed, watch.content_fingerprint());
    assert!(watch.poll(Duration::from_millis(100)).unwrap());
    assert!(!watch.poll(Duration::from_millis(101)).unwrap());
}

#[test]
fn root_order_and_duplicate_roots_do_not_change_observed_content_identity() {
    let project = Project::new();
    let source = project.0.join("scene.py");
    fs::write(&source, "one").unwrap();
    let first = project.snapshot(std::slice::from_ref(&source));
    let second = SourceWatch::new_filtered(
        vec![source, project.0.clone(), project.0.clone()],
        Vec::new(),
        Duration::ZERO,
        WatchLimits::default(),
        WatchFilter {
            extensions: vec!["py".into(), "pyw".into()],
            ignored_directories: vec!["__pycache__".into()],
        },
    )
    .unwrap();
    assert_eq!(first.content_fingerprint(), second.content_fingerprint());
}
