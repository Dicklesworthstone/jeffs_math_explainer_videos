//! fm-34mh: the documented bundle must render through the shipping binary.
#![forbid(unsafe_code)]
#![cfg(all(feature = "cli", feature = "batch"))]

use std::path::Path;
use std::process::Command;

use fmn_codec::{PngLimits, decode_png};
use fmn_scene::TimelineBundle;

fn frames(directory: &Path) -> Vec<Vec<u8>> {
    let mut paths: Vec<_> = std::fs::read_dir(directory)
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "png"))
        .collect();
    paths.sort();
    paths
        .iter()
        .map(|path| std::fs::read(path).unwrap())
        .collect()
}

#[test]
fn readme_bundle_renders_decodable_frames_and_certified_threads_agree() {
    let source = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../demo/wasm/bundle.fmtl")
        .canonicalize()
        .unwrap();
    let bytes = std::fs::read(&source).unwrap();
    let bundle = TimelineBundle::from_bytes(&bytes).unwrap();
    assert_eq!(bundle.frame_count(), 45);
    let root = std::env::temp_dir().join(format!("fmn-readme-bundle-{}", std::process::id()));
    std::fs::create_dir(&root).unwrap();
    let mut certified = None;
    for (name, threads, reproducible) in [
        ("documented", "1", false),
        ("certified-1", "1", true),
        ("certified-4", "4", true),
    ] {
        let destination = root.join(name);
        let mut command = Command::new(env!("CARGO_BIN_EXE_fmn"));
        command.current_dir(&root);
        if reproducible {
            command.arg("--reproducible");
        }
        let result = command
            .args([
                "--format",
                "png_sequence",
                "--resolution",
                "320x180",
                "--threads",
                threads,
                "--video_dir",
            ])
            .arg(&destination)
            .arg(&source)
            .arg("DemoTimeline")
            .env("PATH", "")
            .env_remove("PYTHONPATH")
            .env_remove("PYTHONHOME")
            .output()
            .unwrap();
        assert!(
            result.status.success(),
            "{name}: {}\n{}",
            String::from_utf8_lossy(&result.stdout),
            String::from_utf8_lossy(&result.stderr)
        );
        let pngs = frames(&destination.join("DemoTimeline"));
        assert_eq!(pngs.len(), bundle.frame_count() as usize);
        for png in &pngs {
            let decoded = decode_png(png, &PngLimits::default()).unwrap();
            assert_eq!((decoded.width, decoded.height), (320, 180));
        }
        if reproducible {
            if let Some(expected) = &certified {
                assert_eq!(&pngs, expected);
            } else {
                certified = Some(pngs);
            }
        }
    }
    // This test consumes the existing demo. It must never repair or re-export it.
    assert_eq!(std::fs::read(source).unwrap(), bytes);
}
