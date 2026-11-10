#![forbid(unsafe_code)]
#![cfg(all(feature = "cli", feature = "batch"))]

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "fmn-cpu-pipeline-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir(&path).unwrap();
    path
}

fn render(root: &Path, format: &str, threads: &str) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
    // GIF is deliberately not a certified artifact. Preserve that product
    // refusal; its standard-mode deterministic bytes still test scheduling.
    if format != "gif" {
        command.arg("--reproducible");
    }
    let output = command
        .args([
            "--robot",
            "--format",
            format,
            "--resolution",
            "32x24",
            "--fps",
            "8",
            "--threads",
            threads,
            "--video_dir",
            root.to_str().unwrap(),
            "@builtin",
            "circle_shift.v1",
        ])
        .env("PATH", "")
        .env_remove("PYTHONPATH")
        .env_remove("PYTHONHOME")
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        output.status.success(),
        "{stdout}\n{}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains("\"frame_pipeline\":{\"submitted\":"),
        "{stdout}"
    );
    assert!(stdout.contains("\"outstanding_slots\":0"), "{stdout}");
    assert!(stdout.contains("\"render_team_frames\":["), "{stdout}");
    stdout
}

fn artifacts(root: &Path, format: &str) -> Vec<Vec<u8>> {
    if format == "png_sequence" {
        let mut paths: Vec<_> = std::fs::read_dir(root.join("circle_shift.v1"))
            .unwrap()
            .map(|entry| entry.unwrap().path())
            .filter(|path| path.extension().is_some_and(|ext| ext == "png"))
            .collect();
        paths.sort();
        assert!(!paths.is_empty());
        paths
            .iter()
            .map(|path| std::fs::read(path).unwrap())
            .collect()
    } else {
        vec![std::fs::read(root.join(format!("circle_shift.{format}"))).unwrap()]
    }
}

#[test]
fn shipping_cli_uses_pipeline_and_preserves_native_bytes_at_1_4_16_threads() {
    for format in ["png_sequence", "gif", "y4m"] {
        let mut expected = None;
        for threads in ["1", "4", "16"] {
            let root = root();
            render(&root, format, threads);
            let bytes = artifacts(&root, format);
            if let Some(expected) = &expected {
                assert_eq!(&bytes, expected, "{format} / {threads}");
            } else {
                expected = Some(bytes);
            }
        }
    }
}

#[test]
fn certified_gif_remains_a_capability_refusal_without_output_side_effects() {
    let root = root();
    let output = Command::new(env!("CARGO_BIN_EXE_fmn"))
        .args([
            "--robot",
            "--reproducible",
            "--format",
            "gif",
            "--resolution",
            "32x24",
            "--fps",
            "8",
            "--threads",
            "4",
            "--video_dir",
            root.to_str().unwrap(),
            "@builtin",
            "circle_shift.v1",
        ])
        .env("PATH", "")
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(4));
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("native GIF is outside the certified artifact set"),
        "{stdout}"
    );
    assert_eq!(std::fs::read_dir(root).unwrap().count(), 0);
}
