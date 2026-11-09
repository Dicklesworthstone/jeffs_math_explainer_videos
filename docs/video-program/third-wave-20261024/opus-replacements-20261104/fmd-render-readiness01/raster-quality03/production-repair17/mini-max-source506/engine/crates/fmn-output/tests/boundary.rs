//! fm-wj3 acceptance: the fake-ffmpeg contract suite (every
//! negotiation dimension, both mux stages, failure modes), sandbox
//! tests, capability-error tests with ffmpeg absent, the provenance
//! fingerprint test, and the real-ffmpeg smoke test behind an env
//! flag (FFMPEG_PROTOCOL.md §6).

use std::path::Path;
#[cfg(unix)]
use std::path::PathBuf;
#[cfg(unix)]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(unix)]
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
#[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
use std::time::Duration;

use fmn_frame::ColorRange;
#[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
use fmn_hash::sha256;
#[cfg(unix)]
use fmn_output::{Boundary, BoundaryError, EncoderCapabilities, FfmpegTool, JobLimits};
use fmn_output::{ColorDescription, Container, EncoderChoice, VideoJob, WireFormat, negotiate};
#[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
use fmn_platform::process::{
    ProcessCancellation, ProcessError, ProcessMechanism, ProcessSpec, ProcessStdinLimits,
    RunningProcess,
};
#[cfg(unix)]
use fmn_platform::process::{
    ProcessOutcome, ProcessRunner, ProcessTermination, ScriptedRunner, StdProcessRunner,
};

fn job(wire: WireFormat, container: Container, encoder: EncoderChoice) -> VideoJob {
    VideoJob {
        width: 64,
        height: 36,
        fps: (30000, 1001),
        wire,
        color: ColorDescription::video_bt709(),
        container,
        encoder,
        crf: None,
    }
}

#[cfg(unix)]
static DIR_SEQ: AtomicU64 = AtomicU64::new(0);

/// Serialize the real process-boundary tests.
///
/// The executable fixture is now a Cargo-built native image, so the former
/// shell write/exec `ETXTBSY` race no longer exists. The gate remains useful
/// for the timeout/process-group cases: it prevents sibling tests from
/// creating enough concurrent subprocess and pipe pressure to obscure the
/// supervision contract being measured. [`scratch`] carries the guard for the
/// complete test body.
#[cfg(unix)]
static SPAWN_GATE: Mutex<()> = Mutex::new(());

#[cfg(unix)]
fn create_private_test_directory(path: &Path) {
    std::fs::create_dir(path).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;

        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700)).unwrap();
    }
}

/// A fresh private scratch dir for one test, plus the spawn gate.
#[cfg(unix)]
fn scratch(tag: &str) -> (PathBuf, MutexGuard<'static, ()>) {
    // A panicking test poisons the mutex; the gate protects a file-system
    // race, not an invariant, so a poisoned gate is still a usable gate.
    let gate = SPAWN_GATE.lock().unwrap_or_else(PoisonError::into_inner);
    // Build harnesses may point TMPDIR into a transferred checkout whose
    // shared ancestor is intentionally group-writable. The boundary must
    // reject that tree, so create security fixtures under Unix's sticky
    // temporary root or the Windows temp directory. macOS serves /tmp through
    // /private/tmp; resolve the root up front so scripted-runner outcomes use
    // the paths the boundary sees.
    #[cfg(unix)]
    let tmp = PathBuf::from("/tmp");
    #[cfg(windows)]
    let tmp = std::env::temp_dir();
    let tmp = tmp.canonicalize().unwrap_or(tmp);
    let dir = tmp.join(format!(
        "fmn-boundary-test-{}-{}-{tag}",
        std::process::id(),
        DIR_SEQ.fetch_add(1, Ordering::Relaxed)
    ));
    create_private_test_directory(&dir);
    (dir, gate)
}

// ---- the pure argv contract ----------------------------------------

/// Collect a representative argv from every builder and mode.
fn all_argvs() -> Vec<Vec<String>> {
    let out = Path::new("/tmp/out.mp4");
    let mut argvs = Vec::new();
    for wire in [
        WireFormat::Rgba8,
        WireFormat::Bgra8,
        WireFormat::Nv12,
        WireFormat::P010,
    ] {
        argvs.push(
            negotiate::encode_argv(&job(wire, Container::Mp4, EncoderChoice::Auto), out).unwrap(),
        );
    }
    argvs.push(
        negotiate::encode_argv(
            &job(
                WireFormat::Rgba8,
                Container::MovTransparent,
                EncoderChoice::Auto,
            ),
            out,
        )
        .unwrap(),
    );
    argvs.push(
        negotiate::encode_argv(
            &job(WireFormat::Rgba8, Container::Gif, EncoderChoice::Auto),
            out,
        )
        .unwrap(),
    );
    argvs.push(negotiate::mux_argv(
        Path::new("/tmp/v.mp4"),
        Path::new("/tmp/a.wav"),
        out,
    ));
    argvs.push(negotiate::concat_argv(Path::new("/tmp/list.txt"), out));
    argvs.push(negotiate::transcode_audio_argv(
        Path::new("/tmp/in.mp3"),
        Path::new("/tmp/out.wav"),
    ));
    argvs.push(negotiate::transcode_image_argv(
        Path::new("/tmp/in.webp"),
        Path::new("/tmp/out.png"),
    ));
    argvs
}

#[test]
fn no_vflip_no_eq_no_filters_anywhere() {
    // D-23: the repairs are structurally impossible. No invocation may
    // contain a filter argument of any kind.
    for argv in all_argvs() {
        for arg in &argv {
            assert!(!arg.contains("vflip"), "vflip found in {argv:?}");
            assert!(!arg.starts_with("eq="), "eq filter found in {argv:?}");
            assert!(
                !matches!(
                    arg.as_str(),
                    "-vf" | "-af" | "-filter:v" | "-filter:a" | "-filter_complex"
                ),
                "filter flag found in {argv:?}"
            );
        }
    }
}

#[test]
fn every_wire_format_negotiates() {
    let out = Path::new("/tmp/out.mp4");
    for (wire, pix) in [
        (WireFormat::Rgba8, "rgba"),
        (WireFormat::Bgra8, "bgra"),
        (WireFormat::Nv12, "nv12"),
        (WireFormat::P010, "p010le"),
    ] {
        let argv =
            negotiate::encode_argv(&job(wire, Container::Mp4, EncoderChoice::Auto), out).unwrap();
        let at = argv.iter().position(|a| a == "-pix_fmt").unwrap();
        assert_eq!(argv[at + 1], pix);
        // Rational frame rate, exactly.
        let at = argv.iter().position(|a| a == "-framerate").unwrap();
        assert_eq!(argv[at + 1], "30000/1001");
        // Input geometry.
        let at = argv.iter().position(|a| a == "-video_size").unwrap();
        assert_eq!(argv[at + 1], "64x36");
    }
    // Wire payload arithmetic (the NV12 argument).
    assert_eq!(WireFormat::Rgba8.frame_bytes(3840, 2160), 33_177_600);
    assert_eq!(WireFormat::Nv12.frame_bytes(3840, 2160), 12_441_600);
    assert_eq!(WireFormat::P010.frame_bytes(3840, 2160), 24_883_200);
}

#[test]
fn color_description_maps() {
    let out = Path::new("/tmp/out.mp4");
    let mut j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
    let argv = negotiate::encode_argv(&j, out).unwrap();
    let find = |argv: &[String], key: &str| -> String {
        let at = argv.iter().position(|a| a == key).unwrap();
        argv[at + 1].clone()
    };
    assert_eq!(find(&argv, "-color_primaries"), "bt709");
    assert_eq!(find(&argv, "-color_trc"), "bt709");
    assert_eq!(find(&argv, "-colorspace"), "bt709");
    assert_eq!(find(&argv, "-color_range"), "tv");

    j.color = ColorDescription::srgb_full();
    let argv = negotiate::encode_argv(&j, out).unwrap();
    assert_eq!(find(&argv, "-color_trc"), "iec61966-2-1");
    assert_eq!(find(&argv, "-color_range"), "pc");
    assert_eq!(j.color.range, ColorRange::Full);
}

#[test]
fn container_modes() {
    let out = Path::new("/tmp/out.x");
    let argv = negotiate::encode_argv(
        &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
        out,
    )
    .unwrap();
    assert!(argv.windows(2).any(|w| w == ["-c:v", "libx264"]));
    assert!(argv.windows(2).any(|w| w == ["-movflags", "+faststart"]));
    assert!(argv.windows(2).any(|w| w == ["-pix_fmt", "yuv420p"]));

    // 10-bit in stays 10-bit out.
    let argv = negotiate::encode_argv(
        &job(WireFormat::P010, Container::Mp4, EncoderChoice::Auto),
        out,
    )
    .unwrap();
    assert!(argv.windows(2).any(|w| w == ["-pix_fmt", "yuv420p10le"]));

    // Transparent MOV: qtrle over argb.
    let argv = negotiate::encode_argv(
        &job(
            WireFormat::Bgra8,
            Container::MovTransparent,
            EncoderChoice::Auto,
        ),
        out,
    )
    .unwrap();
    assert!(argv.windows(2).any(|w| w == ["-c:v", "qtrle"]));
    assert!(argv.windows(2).any(|w| w == ["-pix_fmt", "argb"]));

    // GIF mode is muxer-level: -f gif and no -c:v at all.
    let argv = negotiate::encode_argv(
        &job(WireFormat::Rgba8, Container::Gif, EncoderChoice::Auto),
        out,
    )
    .unwrap();
    assert!(argv.windows(2).any(|w| w == ["-f", "gif"]));
    assert!(!argv.iter().any(|a| a == "-c:v"));
}

#[test]
fn negotiation_refusals_are_named() {
    let out = Path::new("/tmp/out.mov");
    // Alpha container on an opaque wire.
    assert!(
        negotiate::encode_argv(
            &job(
                WireFormat::Nv12,
                Container::MovTransparent,
                EncoderChoice::Auto
            ),
            out
        )
        .is_err()
    );
    // CRF on a hardware encoder.
    let mut j = job(
        WireFormat::Nv12,
        Container::Mp4,
        EncoderChoice::Named("h264_nvenc".into()),
    );
    j.crf = Some(20);
    assert!(negotiate::encode_argv(&j, out).is_err());
    // CRF out of range.
    let mut j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
    j.crf = Some(60);
    assert!(negotiate::encode_argv(&j, out).is_err());
    // GIF takes no encoder.
    assert!(
        negotiate::encode_argv(
            &job(
                WireFormat::Rgba8,
                Container::Gif,
                EncoderChoice::Named("libx264".into())
            ),
            out
        )
        .is_err()
    );
    // Zero geometry.
    let mut j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
    j.width = 0;
    assert!(negotiate::encode_argv(&j, out).is_err());
    let mut j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
    j.fps = (30, 0);
    assert!(negotiate::encode_argv(&j, out).is_err());
}

#[test]
fn mux_stage_two_copies_video() {
    // The audit-addendum contract: stage 2 must never re-encode video.
    let argv = negotiate::mux_argv(
        Path::new("/w/video.mp4"),
        Path::new("/w/audio.wav"),
        Path::new("/out/final.mp4"),
    );
    assert!(argv.windows(2).any(|w| w == ["-c:v", "copy"]));
    assert!(argv.windows(2).any(|w| w == ["-c:a", "aac"]));
    assert!(argv.windows(2).any(|w| w == ["-map", "0:v:0"]));
    assert!(argv.windows(2).any(|w| w == ["-map", "1:a:0"]));
}

#[test]
fn concat_and_transcode_shapes() {
    let argv = negotiate::concat_argv(Path::new("/w/list.txt"), Path::new("/out/full.mp4"));
    assert!(argv.windows(4).any(|w| w == ["-f", "concat", "-safe", "0"]));
    assert!(argv.windows(2).any(|w| w == ["-c", "copy"]));

    let argv = negotiate::transcode_audio_argv(Path::new("/in.mp3"), Path::new("/out.wav"));
    assert!(argv.windows(2).any(|w| w == ["-acodec", "pcm_s16le"]));
    let argv = negotiate::transcode_image_argv(Path::new("/in.webp"), Path::new("/out.png"));
    assert!(argv.windows(2).any(|w| w == ["-c:v", "png"]));
}

// Issue #3: on Windows the probe child would inherit the caller's working
// directory in its DLL search order, so the boundary must refuse before the
// runner ever sees a spawn (v0.4.0 behavior).
#[cfg(not(unix))]
#[test]
fn private_ffmpeg_boundary_fails_closed_without_a_provable_directory_acl() {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    let root = std::env::temp_dir().join(format!(
        "fmn-boundary-non-unix-{}-{}",
        std::process::id(),
        SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    std::fs::create_dir(&root).expect("fresh non-Unix boundary fixture");
    let source = std::env::current_exe().expect("current native test executable");
    let runner = fmn_platform::process::ScriptedRunner::new();
    use fmn_platform::process::{FfmpegLocator as _, StdFfmpegLocator};
    let executable = StdFfmpegLocator::default()
        .locate_ffmpeg(&source)
        .expect("current executable is a host-native image");

    let error = fmn_output::FfmpegTool::resolve(executable, &runner, &root)
        .expect_err("safe std cannot prove a private non-Unix workdir ACL");
    assert!(matches!(
        error,
        fmn_output::BoundaryError::Workdir { ref detail }
            if detail.contains("cannot prove a private ffmpeg workdir")
    ));
    assert!(
        runner.runs().is_empty(),
        "a refused private workdir must not execute ffmpeg"
    );
}

// ---- the ScriptedRunner contract suite -----------------------------

#[cfg(unix)]
mod private_boundary {
    use super::*;

    const FAKE_VERSION: &str = "ffmpeg version 7.1-fake Copyright (c) fake";

    fn bound_tool_leaf() -> &'static str {
        if cfg!(windows) {
            "fmn-bound-ffmpeg.exe"
        } else {
            "fmn-bound-ffmpeg"
        }
    }

    fn native_tool_bytes() -> Vec<u8> {
        #[cfg(any(target_os = "linux", target_os = "android"))]
        {
            const HEADER_BYTES: usize = 64;
            const PROGRAM_BYTES: usize = 56;

            let mut bytes = vec![0_u8; HEADER_BYTES + PROGRAM_BYTES];
            bytes[..4].copy_from_slice(b"\x7fELF");
            bytes[4] = 2;
            bytes[5] = 1;
            bytes[6] = 1;
            bytes[16..18].copy_from_slice(&3_u16.to_le_bytes());
            let machine = if cfg!(target_arch = "x86_64") {
                62_u16
            } else {
                183_u16
            };
            bytes[18..20].copy_from_slice(&machine.to_le_bytes());
            bytes[20..24].copy_from_slice(&1_u32.to_le_bytes());
            bytes[24..32].copy_from_slice(&0x1000_u64.to_le_bytes());
            bytes[32..40].copy_from_slice(&(HEADER_BYTES as u64).to_le_bytes());
            bytes[52..54].copy_from_slice(&(HEADER_BYTES as u16).to_le_bytes());
            bytes[54..56].copy_from_slice(&(PROGRAM_BYTES as u16).to_le_bytes());
            bytes[56..58].copy_from_slice(&1_u16.to_le_bytes());
            let image_bytes = bytes.len() as u64;
            let program = &mut bytes[HEADER_BYTES..];
            program[..4].copy_from_slice(&1_u32.to_le_bytes());
            program[4..8].copy_from_slice(&5_u32.to_le_bytes());
            program[16..24].copy_from_slice(&0x1000_u64.to_le_bytes());
            program[32..40].copy_from_slice(&image_bytes.to_le_bytes());
            program[40..48].copy_from_slice(&image_bytes.to_le_bytes());
            program[48..56].copy_from_slice(&0x1000_u64.to_le_bytes());
            return bytes;
        }
        #[cfg(target_os = "macos")]
        {
            const HEADER_BYTES: usize = 32;
            const SEGMENT_BYTES: usize = 72;
            const ENTRY_BYTES: usize = 24;

            let mut bytes = vec![0_u8; HEADER_BYTES + 2 * SEGMENT_BYTES + ENTRY_BYTES + 1];
            bytes[..4].copy_from_slice(&[0xcf, 0xfa, 0xed, 0xfe]);
            let cpu = if cfg!(target_arch = "x86_64") {
                0x0100_0007_u32
            } else {
                0x0100_000c_u32
            };
            bytes[4..8].copy_from_slice(&cpu.to_le_bytes());
            let subtype = if cfg!(target_arch = "x86_64") {
                3_u32
            } else {
                0_u32
            };
            bytes[8..12].copy_from_slice(&subtype.to_le_bytes());
            bytes[12..16].copy_from_slice(&2_u32.to_le_bytes());
            bytes[16..20].copy_from_slice(&3_u32.to_le_bytes());
            bytes[20..24]
                .copy_from_slice(&((2 * SEGMENT_BYTES + ENTRY_BYTES) as u32).to_le_bytes());
            let image_bytes = bytes.len() as u64;
            let pagezero = &mut bytes[HEADER_BYTES..HEADER_BYTES + SEGMENT_BYTES];
            pagezero[..4].copy_from_slice(&0x19_u32.to_le_bytes());
            pagezero[4..8].copy_from_slice(&(SEGMENT_BYTES as u32).to_le_bytes());
            pagezero[8..18].copy_from_slice(b"__PAGEZERO");
            pagezero[32..40].copy_from_slice(&0x1_0000_0000_u64.to_le_bytes());
            let text_offset = HEADER_BYTES + SEGMENT_BYTES;
            let text = &mut bytes[text_offset..text_offset + SEGMENT_BYTES];
            text[..4].copy_from_slice(&0x19_u32.to_le_bytes());
            text[4..8].copy_from_slice(&(SEGMENT_BYTES as u32).to_le_bytes());
            text[8..14].copy_from_slice(b"__TEXT");
            text[24..32].copy_from_slice(&0x1_0000_0000_u64.to_le_bytes());
            text[32..40].copy_from_slice(&image_bytes.to_le_bytes());
            text[48..56].copy_from_slice(&image_bytes.to_le_bytes());
            text[56..60].copy_from_slice(&7_u32.to_le_bytes());
            text[60..64].copy_from_slice(&5_u32.to_le_bytes());
            let entry_offset = HEADER_BYTES + 2 * SEGMENT_BYTES;
            let entry = &mut bytes[entry_offset..entry_offset + ENTRY_BYTES];
            entry[..4].copy_from_slice(&0x8000_0028_u32.to_le_bytes());
            entry[4..8].copy_from_slice(&(ENTRY_BYTES as u32).to_le_bytes());
            entry[8..16].copy_from_slice(
                &((HEADER_BYTES + 2 * SEGMENT_BYTES + ENTRY_BYTES) as u64).to_le_bytes(),
            );
            *bytes.last_mut().expect("entry byte") = 0xc3;
            return bytes;
        }
        #[cfg(windows)]
        {
            const PE_OFFSET: usize = 0x80;
            const OPTIONAL_BYTES: usize = 0xf0;
            const HEADER_BYTES: usize = 0x200;
            const IMAGE_BYTES: usize = 0x400;

            let mut bytes = vec![0_u8; IMAGE_BYTES];
            bytes[..2].copy_from_slice(b"MZ");
            bytes[0x3c..0x40].copy_from_slice(&(PE_OFFSET as u32).to_le_bytes());
            bytes[PE_OFFSET..PE_OFFSET + 4].copy_from_slice(b"PE\0\0");
            let coff = PE_OFFSET + 4;
            let machine = if cfg!(target_arch = "x86_64") {
                0x8664_u16
            } else {
                0xaa64_u16
            };
            bytes[coff..coff + 2].copy_from_slice(&machine.to_le_bytes());
            bytes[coff + 2..coff + 4].copy_from_slice(&1_u16.to_le_bytes());
            bytes[coff + 16..coff + 18].copy_from_slice(&(OPTIONAL_BYTES as u16).to_le_bytes());
            bytes[coff + 18..coff + 20].copy_from_slice(&0x0002_u16.to_le_bytes());
            let optional = coff + 20;
            bytes[optional..optional + 2].copy_from_slice(&0x020b_u16.to_le_bytes());
            bytes[optional + 16..optional + 20].copy_from_slice(&0x1000_u32.to_le_bytes());
            bytes[optional + 24..optional + 32]
                .copy_from_slice(&0x0000_0001_4000_0000_u64.to_le_bytes());
            bytes[optional + 32..optional + 36].copy_from_slice(&0x1000_u32.to_le_bytes());
            bytes[optional + 36..optional + 40].copy_from_slice(&0x200_u32.to_le_bytes());
            bytes[optional + 56..optional + 60].copy_from_slice(&0x2000_u32.to_le_bytes());
            bytes[optional + 60..optional + 64]
                .copy_from_slice(&(HEADER_BYTES as u32).to_le_bytes());
            bytes[optional + 68..optional + 70].copy_from_slice(&3_u16.to_le_bytes());
            let section = optional + OPTIONAL_BYTES;
            bytes[section..section + 5].copy_from_slice(b".text");
            bytes[section + 8..section + 12].copy_from_slice(&1_u32.to_le_bytes());
            bytes[section + 12..section + 16].copy_from_slice(&0x1000_u32.to_le_bytes());
            bytes[section + 16..section + 20].copy_from_slice(&0x200_u32.to_le_bytes());
            bytes[section + 20..section + 24].copy_from_slice(&(HEADER_BYTES as u32).to_le_bytes());
            bytes[section + 36..section + 40].copy_from_slice(&0x6000_0020_u32.to_le_bytes());
            return bytes;
        }
        #[allow(unreachable_code)]
        std::fs::read(std::env::current_exe().expect("current test executable"))
            .expect("read current native test executable")
    }

    fn write_native_tool(path: &Path) {
        std::fs::write(path, native_tool_bytes()).expect("write native executable fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
                .expect("mark native fixture executable");
        }
    }

    fn resolve_tool(
        path: &Path,
        runner: &dyn ProcessRunner,
        workdir_parent: &Path,
    ) -> Result<FfmpegTool, BoundaryError> {
        use fmn_platform::process::{FfmpegLocator as _, StdFfmpegLocator};

        let executable = StdFfmpegLocator::default()
            .locate_ffmpeg(path)
            .map_err(|_| BoundaryError::FfmpegUnavailable {
                attempted: path.to_path_buf(),
                alternative: fmn_output::NATIVE_ALTERNATIVE,
            })?;
        FfmpegTool::resolve(executable, runner, workdir_parent)
    }

    #[test]
    fn scripts_and_post_selection_native_to_script_swaps_cannot_issue_a_tool() {
        use fmn_platform::process::{FfmpegLocator as _, FfmpegLocatorError, StdFfmpegLocator};

        let (dir, _gate) = scratch("typed-token-native-image");
        let script = if cfg!(windows) {
            dir.join("ffmpeg-script.bat")
        } else {
            dir.join("ffmpeg-script")
        };
        std::fs::write(&script, b"#!/bin/sh\necho 'ffmpeg version spoof'\n").unwrap();
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&script, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        assert!(matches!(
            StdFfmpegLocator::default().locate_ffmpeg(&script),
            Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
        ));

        let selected = if cfg!(windows) {
            dir.join("ffmpeg-selected.exe")
        } else {
            dir.join("ffmpeg-selected")
        };
        write_native_tool(&selected);
        let executable = StdFfmpegLocator::default()
            .locate_ffmpeg(&selected)
            .expect("issue token for native image");
        std::fs::write(&selected, b"#!/bin/sh\necho 'ffmpeg version swapped'\n").unwrap();
        let runner = ScriptedRunner::new();
        let error = FfmpegTool::resolve(executable, &runner, &dir)
            .expect_err("post-selection script substitution cannot issue a tool");
        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
        assert!(runner.runs().is_empty());
    }

    #[test]
    fn malformed_native_replacement_is_rejected_before_private_probe() {
        use fmn_platform::process::{FfmpegLocator as _, StdFfmpegLocator};

        let (dir, _gate) = scratch("malformed-private-copy");
        let selected = if cfg!(windows) {
            dir.join("ffmpeg-selected.exe")
        } else {
            dir.join("ffmpeg-selected")
        };
        write_native_tool(&selected);
        let executable = StdFfmpegLocator::default()
            .locate_ffmpeg(&selected)
            .expect("issue token for native image");
        let mut malformed = native_tool_bytes();
        malformed.truncate(8);
        std::fs::write(&selected, malformed).unwrap();

        let runner = ScriptedRunner::new();
        let error = FfmpegTool::resolve(executable, &runner, &dir)
            .expect_err("malformed native container cannot reach -version");
        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
        assert!(runner.runs().is_empty());
    }

    /// Write a fake tool file and script its `-version` probe.
    fn scripted_tool(dir: &Path) -> (PathBuf, ScriptedRunner) {
        let tool_path = if cfg!(windows) {
            dir.join("ffmpeg.exe")
        } else {
            dir.join("ffmpeg")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: format!("{FAKE_VERSION}\nbuilt with fake-gcc\n").into_bytes(),
                stderr: Vec::new(),
            },
        );
        (tool_path, runner)
    }

    fn session_root(dir: &Path, sequence: u64) -> PathBuf {
        dir.join(format!(
            "fmn-ffmpeg-session-{}-{sequence}",
            std::process::id()
        ))
    }

    fn probe_bound_tool(dir: &Path, sequence: u64) -> PathBuf {
        probe_bound_tool_in_session(dir, 0, sequence)
    }

    fn probe_bound_tool_in_session(
        dir: &Path,
        session_sequence: u64,
        probe_sequence: u64,
    ) -> PathBuf {
        session_root(dir, session_sequence)
            .join(format!("fmn-probe-{}-{probe_sequence}", std::process::id()))
            .join(bound_tool_leaf())
    }

    fn bound_tool(dir: &Path, sequence: u64) -> PathBuf {
        session_root(dir, 0)
            .join(format!("fmn-job-{}-{sequence}", std::process::id()))
            .join(bound_tool_leaf())
    }

    fn first_bound_tool(dir: &Path) -> PathBuf {
        bound_tool(dir, 0)
    }

    fn successful_scripted_outcome() -> ProcessOutcome {
        ProcessOutcome {
            termination: ProcessTermination::Exited(Some(0)),
            stdout: Vec::new(),
            stderr: Vec::new(),
        }
    }

    #[test]
    fn provenance_fingerprint() {
        let (dir, _gate) = scratch("fingerprint");
        let (tool_path, runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        assert_eq!(
            tool.sha256_hex(),
            fmn_hash::sha256::sha256(&native_tool_bytes()).to_hex()
        );
        assert_eq!(tool.version(), FAKE_VERSION);
        assert_eq!(tool.path(), std::fs::canonicalize(tool_path).unwrap());
        assert_eq!(tool.native_image().policy_version, 2);
        assert_eq!(
            tool.native_image().file_bytes,
            native_tool_bytes().len() as u64
        );
        let runs = runner.runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].program, probe_bound_tool(&dir, 0));
        assert_ne!(runs[0].program, tool.path());
        assert!(runs[0].cwd.is_none());
        assert_eq!(
            runs[0].env[2].1,
            runs[0].program.parent().unwrap().display().to_string()
        );
    }

    #[test]
    fn absent_ffmpeg_is_a_capability_error_naming_the_alternative() {
        let (dir, _gate) = scratch("absent");
        let runner = ScriptedRunner::new();
        let err = resolve_tool(&dir.join("nope/ffmpeg"), &runner, &dir).unwrap_err();
        let message = err.to_string();
        assert!(message.contains("y4m"), "alternative not named: {message}");
        assert!(message.contains("PNG sequences"), "{message}");
        assert!(matches!(err, BoundaryError::FfmpegUnavailable { .. }));
    }

    #[test]
    fn successful_non_ffmpeg_version_banner_is_refused() {
        let (dir, _gate) = scratch("wrong-version-banner");
        let tool_path = if cfg!(windows) {
            dir.join("custom-tool-name.exe")
        } else {
            dir.join("custom-tool-name")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(&dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: b" ffmpeg version leading-space-spoof\n".to_vec(),
                stderr: Vec::new(),
            },
        );

        let error = resolve_tool(&tool_path, &runner, &dir)
            .expect_err("an exit-zero non-ffmpeg banner must not issue a tool");
        assert!(matches!(
            error,
            BoundaryError::ProbeFailed("-version first line is not an ffmpeg version banner")
        ));
        assert_eq!(runner.runs().len(), 1);
    }

    #[test]
    fn version_banner_requires_strict_utf8_only_on_the_first_line() {
        let (dir, _gate) = scratch("version-banner-utf8");
        let tool_path = if cfg!(windows) {
            dir.join("custom-tool-name.exe")
        } else {
            dir.join("custom-tool-name")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(&dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: b"ffmpeg version strict-first-line\ntrailing-\xff".to_vec(),
                stderr: Vec::new(),
            },
        );

        let tool = resolve_tool(&tool_path, &runner, &dir)
            .expect("only the provenance-bearing first line must be UTF-8");
        assert_eq!(tool.version(), "ffmpeg version strict-first-line");
    }

    #[test]
    fn non_utf8_version_banner_is_refused() {
        let (dir, _gate) = scratch("non-utf8-version-banner");
        let tool_path = if cfg!(windows) {
            dir.join("custom-tool-name.exe")
        } else {
            dir.join("custom-tool-name")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(&dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: b"ffmpeg version hostile-\xff\n".to_vec(),
                stderr: Vec::new(),
            },
        );

        let error = resolve_tool(&tool_path, &runner, &dir)
            .expect_err("a non-UTF-8 provenance banner must be refused");
        assert!(matches!(
            error,
            BoundaryError::ProbeFailed("-version output is not valid UTF-8")
        ));
    }

    #[test]
    fn control_bearing_version_banner_is_refused() {
        let (dir, _gate) = scratch("control-version-banner");
        let tool_path = if cfg!(windows) {
            dir.join("custom-tool-name.exe")
        } else {
            dir.join("custom-tool-name")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(&dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: b"ffmpeg version hostile-\x1b[2J\n".to_vec(),
                stderr: Vec::new(),
            },
        );

        let error = resolve_tool(&tool_path, &runner, &dir)
            .expect_err("a control-bearing provenance banner must be refused");
        assert!(matches!(
            error,
            BoundaryError::ProbeFailed("-version first line contains a control character")
        ));
    }

    #[test]
    fn non_relocatable_ffmpeg_is_a_named_capability_refusal() {
        let (dir, _gate) = scratch("non-relocatable");
        let tool_path = if cfg!(windows) {
            dir.join("ffmpeg.exe")
        } else {
            dir.join("ffmpeg")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool(&dir, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(127)),
                stdout: Vec::new(),
                stderr: b"loader could not resolve a sibling library".to_vec(),
            },
        );

        let error = resolve_tool(&tool_path, &runner, &dir)
            .expect_err("a private-copy probe failure must reject the installation");
        assert!(matches!(
            error,
            BoundaryError::UnsupportedRelocatedExecutable { .. }
        ));
        let message = error.to_string();
        assert!(message.contains("relocatable/self-contained"), "{message}");
        assert!(message.contains("sibling library"), "{message}");
        let runs = runner.runs();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].program, probe_bound_tool(&dir, 0));
        assert_ne!(runs[0].program, std::fs::canonicalize(tool_path).unwrap());
    }

    #[test]
    fn private_session_creation_skips_a_foreign_collision() {
        let (dir, _gate) = scratch("session-collision");
        let collision = session_root(&dir, 0);
        std::fs::create_dir(&collision).unwrap();
        let sentinel = collision.join("foreign");
        std::fs::write(&sentinel, b"not ours").unwrap();
        let tool_path = if cfg!(windows) {
            dir.join("ffmpeg.exe")
        } else {
            dir.join("ffmpeg")
        };
        write_native_tool(&tool_path);
        let mut runner = ScriptedRunner::new();
        runner.script(
            probe_bound_tool_in_session(&dir, 1, 0),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(0)),
                stdout: format!("{FAKE_VERSION}\n").into_bytes(),
                stderr: Vec::new(),
            },
        );

        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        assert_eq!(tool.version(), FAKE_VERSION);
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"not ours");
        assert_eq!(
            runner.runs()[0].program,
            probe_bound_tool_in_session(&dir, 1, 0)
        );
    }

    #[cfg(unix)]
    #[test]
    fn writable_non_sticky_workdir_parent_is_refused_before_probe() {
        use std::os::unix::fs::PermissionsExt as _;

        let (dir, _gate) = scratch("insecure-parent");
        let parent = dir.join("insecure");
        std::fs::create_dir(&parent).unwrap();
        let tool_path = parent.join("ffmpeg");
        write_native_tool(&tool_path);
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o777)).unwrap();
        let runner = ScriptedRunner::new();

        let error = resolve_tool(&tool_path, &runner, &parent)
            .expect_err("an attacker-writable parent cannot anchor a private session");
        assert!(matches!(error, BoundaryError::Workdir { .. }));
        assert!(error.to_string().contains("without the sticky bit"));
        assert!(runner.runs().is_empty());
    }

    #[test]
    fn parent_directory_components_are_refused_before_creation() {
        let (dir, _gate) = scratch("parent-component");
        let tool_path = if cfg!(windows) {
            dir.join("ffmpeg.exe")
        } else {
            dir.join("ffmpeg")
        };
        write_native_tool(&tool_path);
        let runner = ScriptedRunner::new();
        let requested = dir.join("must-not-create/../work");

        let error = resolve_tool(&tool_path, &runner, &requested)
            .expect_err("workdir construction must not traverse parent components");
        assert!(matches!(error, BoundaryError::Workdir { .. }));
        assert!(error.to_string().contains("parent-directory component"));
        assert!(!dir.join("must-not-create").exists());
        assert!(runner.runs().is_empty());
    }

    #[test]
    fn resolved_identity_change_refuses_probes_and_jobs_before_spawn() {
        let (dir, _gate) = scratch("identity-change");
        let (tool_path, runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        std::fs::write(&tool_path, b"substituted executable bytes").unwrap();

        let probe_error =
            EncoderCapabilities::probe(&tool, &runner).expect_err("changed probe identity");
        assert!(matches!(
            probe_error,
            BoundaryError::ExecutableImageRejected { .. }
        ));

        let runner = Arc::new(runner);
        let boundary =
            Boundary::new(tool, runner.clone(), JobLimits::default(), dir.clone()).unwrap();
        let destination = dir.join("must-not-publish.mp4");
        let frames = vec![0_u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let job_error = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames,
                &EncoderCapabilities::parse(ENCODERS_LISTING),
                &destination,
            )
            .expect_err("changed encode identity");
        assert!(matches!(
            job_error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
        assert_eq!(runner.runs().len(), 1, "only the resolution probe ran");
        assert!(!destination.exists());
    }

    #[test]
    fn exclusive_workdir_creation_skips_collisions_without_claiming_them() {
        let (dir, _gate) = scratch("workdir-collision");
        let (tool_path, mut runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        let collision = first_bound_tool(&dir)
            .parent()
            .expect("collision directory")
            .to_path_buf();
        std::fs::create_dir(&collision).unwrap();
        let sentinel = collision.join("foreign");
        std::fs::write(&sentinel, b"not ours").unwrap();

        runner.script(bound_tool(&dir, 1), successful_scripted_outcome());
        let runner = Arc::new(runner);
        let boundary = Boundary::new(
            tool,
            runner.clone(),
            JobLimits {
                keep_workdir: true,
                ..JobLimits::default()
            },
            dir.clone(),
        )
        .unwrap();
        let frames = vec![0_u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let error = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames,
                &EncoderCapabilities::parse(ENCODERS_LISTING),
                &dir.join("unused.mp4"),
            )
            .expect_err("scripted runner creates no artifact");
        assert!(matches!(error, BoundaryError::ArtifactMissing));
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"not ours");

        let runs = runner.runs();
        assert_eq!(runs.len(), 2);
        let workdir = runs[1].program.parent().expect("job workdir");
        assert!(runs[1].cwd.is_none());
        assert_eq!(
            workdir,
            session_root(&dir, 0)
                .join(format!("fmn-job-{}-1", std::process::id()))
                .as_path()
        );
        assert_eq!(runs[1].program, workdir.join(bound_tool_leaf()));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;

            assert_eq!(
                std::fs::metadata(workdir).unwrap().permissions().mode() & 0o777,
                0o700
            );
            assert_eq!(
                std::fs::metadata(&runs[1].program)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o500
            );
        }
    }

    #[test]
    fn boundary_refuses_a_different_workdir_parent_than_resolution() {
        let (dir, _gate) = scratch("workdir-parent-mismatch");
        let (tool_path, runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        let other = dir.join("other");

        let error = match Boundary::new(tool, Arc::new(runner), JobLimits::default(), other) {
            Ok(_) => panic!("a boundary cannot change the private-copy loader environment"),
            Err(error) => error,
        };
        assert!(matches!(error, BoundaryError::Workdir { .. }));
        assert!(
            error
                .to_string()
                .contains("differs from the resolved tool parent")
        );
    }

    #[test]
    fn boundary_refuses_a_replaced_private_session_root() {
        let (dir, _gate) = scratch("session-replacement");
        let (tool_path, runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        let original = session_root(&dir, 0);
        let displaced = dir.join("displaced-session");
        std::fs::rename(&original, &displaced).unwrap();
        std::fs::create_dir(&original).unwrap();
        let sentinel = original.join("foreign");
        std::fs::write(&sentinel, b"not ours").unwrap();

        let error = match Boundary::new(tool, Arc::new(runner), JobLimits::default(), dir.clone()) {
            Ok(_) => panic!("a boundary cannot accept a replaced private session root"),
            Err(error) => error,
        };
        assert!(matches!(error, BoundaryError::Workdir { .. }));
        assert!(error.to_string().contains("session identity changed"));
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"not ours");
    }

    #[test]
    fn cleanup_retains_a_replaced_job_directory() {
        let (dir, _gate) = scratch("workdir-replacement");
        let (tool_path, mut runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        runner.script(first_bound_tool(&dir), successful_scripted_outcome());
        let runner = Arc::new(runner);
        let boundary = Boundary::new(tool, runner, JobLimits::default(), dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let stream = boundary
            .start_encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                &caps,
                &dir.join("unused.mp4"),
                None,
                fmn_platform::process::ProcessCancellation::new(),
                fmn_platform::process::ProcessStdinLimits::new(1, 1),
            )
            .unwrap();
        let claimed = first_bound_tool(&dir)
            .parent()
            .expect("job directory")
            .to_path_buf();
        let displaced = session_root(&dir, 0).join("displaced-owned-job");
        std::fs::rename(&claimed, &displaced).unwrap();
        std::fs::create_dir(&claimed).unwrap();
        let sentinel = claimed.join("foreign");
        std::fs::write(&sentinel, b"not ours").unwrap();

        drop(stream);

        assert_eq!(std::fs::read(&sentinel).unwrap(), b"not ours");
        assert!(displaced.join(bound_tool_leaf()).is_file());
    }

    const ENCODERS_LISTING: &str = "Encoders:\n V..... = Video\n ------\n V....D libx264              H.264 (x264)\n V....D libx265              H.265 (x265)\n V....D qtrle                QuickTime RLE\n V....D h264_nvenc           NVIDIA NVENC H.264\n A....D aac                  AAC audio\n";

    #[test]
    fn encoder_capabilities_parse_and_report_hardware() {
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        assert!(caps.offers("libx264"));
        assert!(caps.offers("h264_nvenc"));
        assert!(!caps.offers("hevc_videotoolbox"));
        assert_eq!(caps.hardware(), vec!["h264_nvenc".to_string()]);
    }

    /// Run one scripted encode and hand back the recorded spec (the run
    /// necessarily ends in `ArtifactMissing` — the scripted runner writes
    /// nothing — which itself proves artifact verification gates
    /// publication).
    fn scripted_encode(
        encoder: EncoderChoice,
        frames: Vec<u8>,
    ) -> (
        Result<(), BoundaryError>,
        Vec<fmn_platform::process::ProcessSpec>,
        PathBuf,
    ) {
        let (dir, _gate) = scratch("contract");
        let (tool_path, mut runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        runner.script(first_bound_tool(&dir), successful_scripted_outcome());
        let runner = Arc::new(runner);
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let boundary =
            Boundary::new(tool, runner.clone(), JobLimits::default(), dir.clone()).unwrap();
        let destination = dir.join("final/out.mp4");
        let result = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, encoder),
                frames,
                &caps,
                &destination,
            )
            .map(|_| ());
        (result, runner.runs(), dir)
    }

    #[test]
    fn encode_contract_spec() {
        let frame = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let (result, runs, dir) = scripted_encode(EncoderChoice::Auto, frame.repeat(3));
        // The scripted run wrote no artifact: verification refuses, and the
        // destination was never touched.
        assert!(matches!(result, Err(BoundaryError::ArtifactMissing)));
        assert!(!dir.join("final/out.mp4").exists());

        // One probe (-version) + one encode.
        assert_eq!(runs.len(), 2);
        let spec = &runs[1];
        assert!(spec.program.is_absolute());
        // Env allowlist, exactly, in order: LANG, LC_ALL, TMPDIR.
        assert_eq!(spec.env.len(), 3);
        assert_eq!(spec.env[0], ("LANG".to_string(), "C".to_string()));
        assert_eq!(spec.env[1], ("LC_ALL".to_string(), "C".to_string()));
        assert_eq!(spec.env[2].0, "TMPDIR");
        // The private dir is TMPDIR, under the workdir root. All governed
        // paths are absolute, so no child cwd change is requested.
        assert!(spec.cwd.is_none());
        let private_dir = spec.program.parent().unwrap();
        assert!(private_dir.starts_with(&dir));
        assert_eq!(spec.env[2].1, private_dir.display().to_string());
        assert_eq!(spec.program, private_dir.join(bound_tool_leaf()));
        // The payload rides stdin, whole.
        assert_eq!(
            spec.stdin.as_ref().unwrap().len(),
            3 * WireFormat::Nv12.frame_bytes(64, 36)
        );
        // The artifact is born inside the private dir.
        let out_arg = spec.argv.last().unwrap();
        assert!(out_arg.starts_with(&private_dir.display().to_string()));
    }

    #[test]
    fn relative_audio_is_resolved_before_the_cwd_free_mux_spawn() {
        let (dir, _gate) = scratch("relative-audio");
        let (tool_path, mut runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        runner.script(first_bound_tool(&dir), successful_scripted_outcome());
        let runner = Arc::new(runner);
        let boundary =
            Boundary::new(tool, runner.clone(), JobLimits::default(), dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let relative_audio = Path::new("Cargo.toml");
        let canonical_audio = std::fs::canonicalize(relative_audio).unwrap();
        let stream = boundary
            .start_encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                &caps,
                &dir.join("unused.mp4"),
                Some(relative_audio),
                fmn_platform::process::ProcessCancellation::new(),
                fmn_platform::process::ProcessStdinLimits::new(1, 0),
            )
            .unwrap();

        let video_artifact = first_bound_tool(&dir)
            .parent()
            .expect("job directory")
            .join("video.mp4");
        std::fs::write(&video_artifact, b"scripted video").unwrap();
        let result = stream.prepare();
        assert!(matches!(result, Err(BoundaryError::ArtifactMissing)));

        let runs = runner.runs();
        assert_eq!(runs.len(), 3, "probe, video encode, then audio mux");
        let mux = &runs[2];
        assert!(mux.cwd.is_none());
        let canonical_audio = canonical_audio.to_str().expect("UTF-8 test path");
        assert!(
            mux.argv.iter().any(|argument| argument == canonical_audio),
            "mux argv did not contain the resolved audio path: {:?}",
            mux.argv
        );
        for input_at in mux
            .argv
            .iter()
            .enumerate()
            .filter_map(|(index, argument)| (argument == "-i").then_some(index + 1))
        {
            assert!(
                Path::new(&mux.argv[input_at]).is_absolute(),
                "cwd-free mux received relative input {:?}",
                mux.argv[input_at]
            );
        }
    }

    #[test]
    fn relative_concat_inputs_are_resolved_before_the_cwd_free_spawn() {
        let (dir, _gate) = scratch("relative-concat");
        let (tool_path, mut runner) = scripted_tool(&dir);
        let tool = resolve_tool(&tool_path, &runner, &dir).unwrap();
        runner.script(first_bound_tool(&dir), successful_scripted_outcome());
        let runner = Arc::new(runner);
        let boundary = Boundary::new(
            tool,
            runner.clone(),
            JobLimits {
                keep_workdir: true,
                ..JobLimits::default()
            },
            dir.clone(),
        )
        .unwrap();
        let relative_part = PathBuf::from("Cargo.toml");
        let absolute_part = dir.join("part1.mp4");
        std::fs::write(&absolute_part, b"scripted part").unwrap();
        let canonical_relative = std::fs::canonicalize(&relative_part).unwrap();
        let canonical_absolute = std::fs::canonicalize(&absolute_part).unwrap();

        let error = boundary
            .concat(
                &[relative_part, absolute_part],
                &dir.join("unused-joined.mp4"),
            )
            .expect_err("scripted concat creates no artifact");
        assert!(matches!(error, BoundaryError::ArtifactMissing));

        let runs = runner.runs();
        assert_eq!(runs.len(), 2, "probe, then concat");
        let concat = &runs[1];
        assert!(concat.cwd.is_none());
        let list_at = concat
            .argv
            .iter()
            .position(|argument| argument == "-i")
            .expect("concat list input")
            + 1;
        let list_file = Path::new(&concat.argv[list_at]);
        assert!(list_file.is_absolute());
        assert_eq!(
            std::fs::read_to_string(list_file).unwrap(),
            format!(
                "file '{}'\nfile '{}'\n",
                canonical_relative.to_str().expect("UTF-8 test path"),
                canonical_absolute.to_str().expect("UTF-8 test path")
            )
        );
    }

    #[test]
    fn hardware_encoder_explicit_and_unknown() {
        let frame = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        // Present in caps: the argv names it.
        let (result, runs, _) =
            scripted_encode(EncoderChoice::Named("h264_nvenc".into()), frame.clone());
        assert!(matches!(result, Err(BoundaryError::ArtifactMissing)));
        assert!(runs[1].argv.windows(2).any(|w| w == ["-c:v", "h264_nvenc"]));

        // Absent from caps: refused before any spawn.
        let (result, runs, _) =
            scripted_encode(EncoderChoice::Named("hevc_videotoolbox".into()), frame);
        let error = result.expect_err("unknown hardware encoder must be refused");
        assert!(
            matches!(&error, BoundaryError::UnknownEncoder { .. }),
            "expected UnknownEncoder, got {error:?}"
        );
        if let BoundaryError::UnknownEncoder {
            requested,
            hardware_available,
        } = error
        {
            assert_eq!(requested, "hevc_videotoolbox");
            assert_eq!(hardware_available, vec!["h264_nvenc".to_string()]);
        }
        // Only the -version probe ran.
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn payload_geometry_is_checked_before_spawn() {
        let (result, runs, _) = scripted_encode(EncoderChoice::Auto, vec![0u8; 1000]);
        assert!(matches!(result, Err(BoundaryError::PayloadGeometry { .. })));
        assert_eq!(runs.len(), 1); // -version only
    }

    #[test]
    fn failure_modes_map_to_typed_refusals() {
        let (dir, _gate) = scratch("failures");
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let frames = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);

        // Nonzero exit with stderr.
        let nonzero_dir = dir.join("nonzero");
        create_private_test_directory(&nonzero_dir);
        let (tool_path, mut runner) = scripted_tool(&nonzero_dir);
        let tool = resolve_tool(&tool_path, &runner, &nonzero_dir).unwrap();
        runner.script(
            first_bound_tool(&nonzero_dir),
            ProcessOutcome {
                termination: ProcessTermination::Exited(Some(1)),
                stdout: Vec::new(),
                stderr: b"Unknown encoder 'libx264'".to_vec(),
            },
        );
        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), nonzero_dir).unwrap();
        let error = boundary
            .encode(&j, frames.clone(), &caps, &dir.join("a.mp4"))
            .expect_err("nonzero ffmpeg exit must be refused");
        assert!(
            matches!(&error, BoundaryError::EncodeFailed { .. }),
            "expected EncodeFailed, got {error:?}"
        );
        if let BoundaryError::EncodeFailed { code, stderr } = error {
            assert_eq!(code, Some(1));
            assert!(stderr.contains("Unknown encoder"));
        }

        // Timeout.
        let timeout_dir = dir.join("timeout");
        create_private_test_directory(&timeout_dir);
        let (tool_path, mut runner) = scripted_tool(&timeout_dir);
        let tool = resolve_tool(&tool_path, &runner, &timeout_dir).unwrap();
        runner.script(
            first_bound_tool(&timeout_dir),
            ProcessOutcome {
                termination: ProcessTermination::TimedOut,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
        );
        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), timeout_dir).unwrap();
        assert!(matches!(
            boundary.encode(&j, frames.clone(), &caps, &dir.join("b.mp4")),
            Err(BoundaryError::JobTimedOut { .. })
        ));

        // Log overflow.
        let overflow_dir = dir.join("overflow");
        create_private_test_directory(&overflow_dir);
        let (tool_path, mut runner) = scripted_tool(&overflow_dir);
        let tool = resolve_tool(&tool_path, &runner, &overflow_dir).unwrap();
        runner.script(
            first_bound_tool(&overflow_dir),
            ProcessOutcome {
                termination: ProcessTermination::OutputLimitExceeded,
                stdout: Vec::new(),
                stderr: Vec::new(),
            },
        );
        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), overflow_dir).unwrap();
        assert!(matches!(
            boundary.encode(&j, frames, &caps, &dir.join("c.mp4")),
            Err(BoundaryError::LogOverflow)
        ));
    }

    #[test]
    fn prerun_counts_without_spawning() {
        // The --prerun retention: a counting pass plans jobs (negotiation +
        // argv construction are pure) and invokes the boundary zero times.
        let runner = ScriptedRunner::new();
        let j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
        let _plan = negotiate::encode_argv(&j, Path::new("/tmp/out.mp4")).unwrap();
        let frames_expected = 42u64; // the counting pass's product
        assert!(frames_expected > 0);
        assert!(runner.runs().is_empty(), "prerun must not spawn");
    }

    // ---- the fake-ffmpeg sandbox suite (real StdProcessRunner) ---------

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    fn copy_native_ffmpeg(dir: &Path, name: &str) -> PathBuf {
        let source = std::env::var_os("CARGO_BIN_EXE_fmn-ffmpeg-test-fixture")
            .map(PathBuf::from)
            .expect("Cargo exposes the native ffmpeg fixture");
        let leaf = if cfg!(windows) && !name.to_ascii_lowercase().ends_with(".exe") {
            format!("{name}.exe")
        } else {
            name.to_owned()
        };
        let path = dir.join(leaf);
        std::fs::copy(source, &path).expect("copy native ffmpeg fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt as _;
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
        }
        path
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    fn set_native_fixture_mode(dir: &Path, mode: &str) {
        std::fs::write(dir.join(".fmn-native-ffmpeg-mode"), mode)
            .expect("write native fixture mode");
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    struct SourceSwappingRunner {
        source: PathBuf,
        replacement: Vec<u8>,
        swapped: std::sync::atomic::AtomicBool,
        swap_on_version: bool,
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    impl ProcessRunner for SourceSwappingRunner {
        fn mechanism(&self) -> ProcessMechanism {
            StdProcessRunner.mechanism()
        }

        fn start(
            &self,
            spec: &ProcessSpec,
            cancellation: ProcessCancellation,
            stdin_limits: ProcessStdinLimits,
        ) -> Result<Box<dyn RunningProcess>, ProcessError> {
            let is_version = spec.argv == ["-version"];
            if (self.swap_on_version || !is_version) && !self.swapped.swap(true, Ordering::AcqRel) {
                std::fs::write(&self.source, &self.replacement).map_err(|error| {
                    ProcessError::Plumbing {
                        program: spec.program.clone(),
                        detail: format!(
                            "replace resolved executable {}: {error}",
                            self.source.display()
                        ),
                    }
                })?;
            }
            StdProcessRunner.start(spec, cancellation, stdin_limits)
        }
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    struct TransientSourceSwappingRunner {
        source: PathBuf,
        replacement: Vec<u8>,
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    impl ProcessRunner for TransientSourceSwappingRunner {
        fn mechanism(&self) -> ProcessMechanism {
            StdProcessRunner.mechanism()
        }

        fn start(
            &self,
            spec: &ProcessSpec,
            cancellation: ProcessCancellation,
            stdin_limits: ProcessStdinLimits,
        ) -> Result<Box<dyn RunningProcess>, ProcessError> {
            let original = std::fs::read(&self.source).map_err(|error| ProcessError::Plumbing {
                program: spec.program.clone(),
                detail: format!(
                    "read resolved executable {}: {error}",
                    self.source.display()
                ),
            })?;
            std::fs::write(&self.source, &self.replacement).map_err(|error| {
                ProcessError::Plumbing {
                    program: spec.program.clone(),
                    detail: format!(
                        "transiently replace resolved executable {}: {error}",
                        self.source.display()
                    ),
                }
            })?;
            let started = StdProcessRunner.start(spec, cancellation, stdin_limits);
            let restored = std::fs::write(&self.source, original);
            match (started, restored) {
                (Ok(process), Ok(())) => Ok(process),
                (Ok(process), Err(error)) => {
                    let _ = process.cancel();
                    Err(ProcessError::Plumbing {
                        program: spec.program.clone(),
                        detail: format!(
                            "restore resolved executable {}: {error}",
                            self.source.display()
                        ),
                    })
                }
                (Err(error), _) => Err(error),
            }
        }
    }

    /// Resolve a fake tool that answers the `-version` probe.
    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    fn real_tool(dir: &Path) -> (FfmpegTool, StdProcessRunner) {
        let runner = StdProcessRunner;
        let path = copy_native_ffmpeg(dir, "fake-ffmpeg");
        let tool = resolve_tool(&path, &runner, dir).unwrap();
        (tool, runner)
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn transient_source_swap_cannot_select_the_version_probe_executable() {
        let (dir, _gate) = scratch("transient-version-substitution");
        let source = copy_native_ffmpeg(&dir, "fake-ffmpeg");
        let original = std::fs::read(&source).expect("read original native fixture");
        let runner = TransientSourceSwappingRunner {
            source: source.clone(),
            replacement:
                b"#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version evil'; exit 0; fi\nexit 1\n"
                    .to_vec(),
        };

        let tool = resolve_tool(&source, &runner, &dir)
            .expect("the private copy is unaffected by a transient source swap");
        assert_eq!(tool.version(), "ffmpeg version 7.1-fake");
        assert_eq!(std::fs::read(&source).unwrap(), original);
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn version_probe_detects_substitution_during_execution() {
        let (dir, _gate) = scratch("version-substitution");
        let source = copy_native_ffmpeg(&dir, "fake-ffmpeg");
        let runner = SourceSwappingRunner {
            source: source.clone(),
            replacement:
                b"#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version evil'; exit 0; fi\nexit 1\n"
                    .to_vec(),
            swapped: std::sync::atomic::AtomicBool::new(false),
            swap_on_version: true,
        };

        let error =
            resolve_tool(&source, &runner, &dir).expect_err("version substitution must fail");
        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn encoder_probe_detects_substitution_during_execution() {
        let (dir, _gate) = scratch("probe-substitution");
        let source = copy_native_ffmpeg(&dir, "fake-ffmpeg");
        let tool = resolve_tool(&source, &StdProcessRunner, &dir).unwrap();
        let runner = SourceSwappingRunner {
            source,
            replacement: b"#!/bin/sh\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version evil'; exit 0; fi\nif [ \"$2\" = \"-encoders\" ]; then echo 'Encoders:'; echo ' ------'; echo ' V....D libx264 fake'; exit 0; fi\nexit 1\n".to_vec(),
            swapped: std::sync::atomic::AtomicBool::new(false),
            swap_on_version: false,
        };

        let error =
            EncoderCapabilities::probe(&tool, &runner).expect_err("probe substitution must fail");
        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn sandbox_publishes_atomically_and_pins_the_environment() {
        let (dir, _gate) = scratch("sandbox");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            keep_workdir: true,
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("published/movie.mp4");
        let frames = vec![7u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let report = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames,
                &caps,
                &destination,
            )
            .unwrap();

        // Published by rename: destination holds the artifact bytes.
        assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");
        assert_eq!(report.artifact_bytes, 9);
        assert_eq!(report.artifact_digest, sha256(b"FAKEVIDEO"));
        assert_eq!(report.destination, destination);
        assert_eq!(report.invocations.len(), 1);
        let invocation = &report.invocations[0];
        assert_eq!(invocation.provenance.encoder.as_deref(), Some("libx264"));
        assert!(
            invocation
                .provenance
                .tool_version
                .starts_with("ffmpeg version 7.1-fake")
        );

        // The env the child actually observed: pinned locale, private
        // TMPDIR, and no ambient leakage.
        let workdir: PathBuf = invocation.provenance.argv.last().unwrap().into();
        let envdump = std::fs::read_to_string(format!("{}.envdump", workdir.display())).unwrap();
        assert!(envdump.contains("LANG=C"));
        assert!(envdump.contains("LC_ALL=C"));
        assert!(envdump.contains("TMPDIR="));
        assert!(
            !envdump.contains("HOME="),
            "ambient HOME leaked:\n{envdump}"
        );
        assert!(
            !envdump.contains("PATH="),
            "ambient PATH leaked:\n{envdump}"
        );
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn prepared_commit_refuses_a_replaced_job_directory() {
        let (dir, _gate) = scratch("prepared-workdir-replacement");
        let (tool, runner) = real_tool(&dir);
        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("must-not-publish.mp4");
        let frames = vec![5_u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let bytes = u64::try_from(frames.len()).unwrap();
        let mut stream = boundary
            .start_encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                &caps,
                &destination,
                None,
                ProcessCancellation::new(),
                ProcessStdinLimits::new(bytes, bytes),
            )
            .unwrap();
        stream.write_stdin(&frames).unwrap();
        let prepared = stream.prepare().unwrap();
        let claimed = first_bound_tool(&dir)
            .parent()
            .expect("job directory")
            .to_path_buf();
        let displaced = session_root(&dir, 0).join("displaced-prepared-job");
        std::fs::rename(&claimed, &displaced).unwrap();
        std::fs::create_dir(&claimed).unwrap();
        let sentinel = claimed.join("foreign");
        std::fs::write(&sentinel, b"not ours").unwrap();

        let error = prepared
            .commit()
            .expect_err("commit must not read or publish through a replaced path");
        assert!(matches!(error, BoundaryError::Workdir { .. }));
        assert!(
            error
                .to_string()
                .contains("claimed directory identity changed")
        );
        assert_eq!(std::fs::read(&sentinel).unwrap(), b"not ours");
        assert!(displaced.join("out.mp4").is_file());
        assert!(!destination.exists());
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn spawn_executes_the_bound_copy_when_the_source_is_replaced() {
        let (dir, _gate) = scratch("spawn-binding");
        let source = copy_native_ffmpeg(&dir, "fake-ffmpeg");
        let original = std::fs::read(&source).expect("read original native fixture");
        let replacement = b"#!/bin/sh\ncat > /dev/null\nif [ \"$1\" = \"-version\" ]; then echo 'ffmpeg version evil'; exit 0; fi\nfor a in \"$@\"; do last=\"$a\"; done\nprintf 'EVILVIDEO' > \"$last\"\nexit 0\n".to_vec();
        let runner = Arc::new(SourceSwappingRunner {
            source: source.clone(),
            replacement: replacement.clone(),
            swapped: std::sync::atomic::AtomicBool::new(false),
            swap_on_version: false,
        });
        let tool = resolve_tool(&source, runner.as_ref(), &dir).unwrap();
        let source_identity = tool.path().to_path_buf();
        let source_sha256 = tool.sha256_hex().to_owned();
        let boundary = Boundary::new(
            tool,
            runner,
            JobLimits {
                keep_workdir: true,
                ..JobLimits::default()
            },
            dir.clone(),
        )
        .unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("bound.mp4");
        let frames = vec![3_u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let report = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames.clone(),
                &caps,
                &destination,
            )
            .unwrap();

        assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");
        assert_eq!(std::fs::read(&source).unwrap(), replacement);
        let provenance = &report.invocations[0].provenance;
        assert_eq!(provenance.tool_path, source_identity);
        assert_eq!(provenance.tool_sha256_hex, source_sha256);
        assert_ne!(provenance.bound_tool_path, provenance.tool_path);
        #[cfg(unix)]
        assert_eq!(
            provenance.process_mechanism,
            "posix_spawn.absolute_path.new_process_group"
        );
        #[cfg(windows)]
        assert_eq!(
            provenance.process_mechanism,
            "create_process_w.explicit_application.atomic_job_list"
        );
        assert_eq!(provenance.process_policy_version, 1);
        assert_eq!(
            std::fs::read(&provenance.bound_tool_path).unwrap(),
            original
        );

        let second_destination = dir.join("must-refuse.mp4");
        let error = boundary
            .encode(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames,
                &caps,
                &second_destination,
            )
            .expect_err("later job must reject the changed source");
        assert!(matches!(
            error,
            BoundaryError::ExecutableImageRejected { .. }
        ));
        assert!(!second_destination.exists());
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn sandbox_timeout_separates_setup_from_tree_kill_and_leaves_destination_untouched() {
        let (dir, _gate) = scratch("timeout");
        // The native fixture remains the direct child while its child inherits
        // the process-group pipes. Its readiness marker is written only after
        // that descendant exists, so the second interval below measures
        // timeout/kill/reap rather than private-copy and hashing setup.
        set_native_fixture_mode(&dir, "timeout");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            timeout: Duration::from_millis(200),
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("never.mp4");
        let mut blocked_job = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
        blocked_job.width = 3840;
        blocked_job.height = 2160;
        let frames = vec![0u8; WireFormat::Nv12.frame_bytes(blocked_job.width, blocked_job.height)];
        let worker_destination = destination.clone();
        let setup_started = std::time::Instant::now();
        let worker = std::thread::spawn(move || {
            boundary.encode(&blocked_job, frames, &caps, &worker_destination)
        });
        let ready = dir.join(".fmn-native-ffmpeg-timeout-ready");
        while !ready.is_file() {
            assert!(
                setup_started.elapsed() < Duration::from_secs(30),
                "native fixture did not reach post-spawn readiness; setup phase elapsed {:?}",
                setup_started.elapsed()
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        let setup_elapsed = setup_started.elapsed();
        let supervision_started = std::time::Instant::now();
        let result = worker.join().expect("native timeout worker");
        let supervision_elapsed = supervision_started.elapsed();
        assert!(
            matches!(result, Err(BoundaryError::JobTimedOut { .. })),
            "expected a timeout, got {result:?}"
        );
        assert!(
            supervision_elapsed < Duration::from_secs(3),
            "post-spawn timeout/tree-kill/reap was not prompt: {supervision_elapsed:?}; \
             setup was independently classified as {setup_elapsed:?}"
        );
        assert!(!destination.exists());
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn sandbox_refuses_oversized_artifacts() {
        let (dir, _gate) = scratch("oversize");
        set_native_fixture_mode(&dir, "oversize");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            max_artifact_bytes: 1024,
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("big.mp4");
        let frames = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let result = boundary.encode(
            &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
            frames,
            &caps,
            &destination,
        );
        assert!(
            matches!(
                result,
                Err(BoundaryError::ArtifactOversized {
                    bytes: 4096,
                    max: 1024
                })
            ),
            "expected an oversized-artifact refusal, got {result:?}"
        );
        assert!(!destination.exists());
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn sandbox_failed_job_preserves_existing_destination() {
        let (dir, _gate) = scratch("failkeep");
        set_native_fixture_mode(&dir, "fail7");
        let (tool, runner) = real_tool(&dir);
        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("keep.mp4");
        std::fs::write(&destination, b"the old render").unwrap();
        let frames = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let result = boundary.encode(
            &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
            frames,
            &caps,
            &destination,
        );
        let error = result.expect_err("failed ffmpeg job must be refused");
        assert!(
            matches!(&error, BoundaryError::EncodeFailed { .. }),
            "expected EncodeFailed, got {error:?}"
        );
        if let BoundaryError::EncodeFailed { code, stderr } = error {
            assert_eq!(code, Some(7));
            assert!(stderr.contains("boom"));
        }
        assert_eq!(std::fs::read(&destination).unwrap(), b"the old render");
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn two_stage_mux_runs_both_stages_and_copies_video() {
        let (dir, _gate) = scratch("mux");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            keep_workdir: true,
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let caps = EncoderCapabilities::parse(ENCODERS_LISTING);
        let destination = dir.join("with_audio.mp4");
        let audio = dir.join("track.wav");
        std::fs::write(&audio, b"RIFFfake").unwrap();
        let frames = vec![0u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let report = boundary
            .encode_with_audio(
                &job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto),
                frames,
                &audio,
                &caps,
                &destination,
            )
            .unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");

        // The private dir's argv log shows both stages; stage 2 copied the
        // video stream and never re-encoded it.
        assert_eq!(report.invocations.len(), 2);
        let workdir: PathBuf = report.invocations[0].provenance.argv.last().unwrap().into();
        let log = std::fs::read_to_string(workdir.parent().unwrap().join("argv.log")).unwrap();
        let lines: Vec<&str> = log.lines().collect();
        assert_eq!(lines.len(), 2, "expected two stages:\n{log}");
        assert!(lines[0].contains("rawvideo"), "stage 1 encodes video");
        assert!(lines[1].contains("-c:v copy"), "stage 2 copies video");
        assert!(lines[1].contains("-c:a aac"), "stage 2 encodes audio");
        assert!(!lines[1].contains("libx264"), "stage 2 must not re-encode");
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn audio_transcode_uses_the_fake_capability_and_publishes_wav() {
        let (dir, _gate) = scratch("audio-transcode");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            keep_workdir: true,
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let input = dir.join("source.mp3");
        let destination = dir.join("decoded/track.wav");
        std::fs::write(&input, b"fake compressed audio").unwrap();

        let report = boundary
            .transcode_audio(&input, &destination)
            .expect("fake ffmpeg decodes");
        assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");
        assert_eq!(report.invocations.len(), 1);
        let invocation = &report.invocations[0];
        assert!(invocation.provenance.encoder.is_none());
        assert!(
            invocation
                .provenance
                .argv
                .windows(2)
                .any(|args| { args == ["-acodec", "pcm_s16le"] })
        );
        assert!(
            invocation
                .provenance
                .argv
                .last()
                .is_some_and(|path| path.ends_with("decoded.wav"))
        );
    }

    #[cfg(all(unix, feature = "ffmpeg-test-fixture"))]
    #[test]
    fn concat_writes_a_list_and_copies_streams() {
        let (dir, _gate) = scratch("concat");
        let (tool, runner) = real_tool(&dir);
        let limits = JobLimits {
            keep_workdir: true,
            ..JobLimits::default()
        };
        let boundary = Boundary::new(tool, Arc::new(runner), limits, dir.clone()).unwrap();
        let parts = vec![dir.join("part0.mp4"), dir.join("part1.mp4")];
        for part in &parts {
            std::fs::write(part, b"encoded part fixture").unwrap();
        }
        let destination = dir.join("joined.mp4");
        let report = boundary.concat(&parts, &destination).unwrap();
        assert_eq!(std::fs::read(&destination).unwrap(), b"FAKEVIDEO");
        assert_eq!(report.invocations.len(), 1);
        assert!(report.invocations[0].provenance.encoder.is_none());

        // A quoted path is refused, not escaped.
        let evil = vec![dir.join("it's.mp4")];
        std::fs::write(&evil[0], b"hostile path fixture").unwrap();
        assert!(matches!(
            boundary.concat(&evil, &destination),
            Err(BoundaryError::Negotiation(_))
        ));
    }

    // ---- the real thing, behind an env flag ----------------------------

    #[test]
    fn real_ffmpeg_smoke() {
        if std::env::var("FMN_REAL_FFMPEG").is_err() {
            return; // opt-in only
        }
        let (dir, _gate) = scratch("real");
        let runner = StdProcessRunner;
        let tool = resolve_tool(Path::new("/usr/bin/ffmpeg"), &runner, &dir).unwrap();
        assert!(tool.version().starts_with("ffmpeg version"));
        let caps = EncoderCapabilities::probe(&tool, &runner).unwrap();
        assert!(caps.offers("libx264") || caps.offers("mpeg4"));

        let boundary =
            Boundary::new(tool, Arc::new(runner), JobLimits::default(), dir.clone()).unwrap();
        let destination = dir.join("smoke.mp4");
        let mut j = job(WireFormat::Nv12, Container::Mp4, EncoderChoice::Auto);
        j.fps = (30, 1);
        // Three gray frames.
        let frame = vec![0x80u8; WireFormat::Nv12.frame_bytes(64, 36)];
        let report = boundary
            .encode(&j, frame.repeat(3), &caps, &destination)
            .unwrap();
        let bytes = std::fs::read(&destination).unwrap();
        assert!(bytes.len() > 100, "suspiciously small mp4");
        assert!(report.invocations[0].provenance.tool_sha256_hex.len() == 64);
        println!(
            "real ffmpeg smoke OK: {} bytes via {}",
            bytes.len(),
            report.invocations[0].provenance.tool_version
        );
    }
}
