//! Capability trait-suite tests (fm-x68 acceptance): the std implementations
//! against the real host, and the test doubles' contracts.
//!
//! The process tests exercise the D2 mechanism substrate for real: argv-only
//! spawning, the cleared-environment allowlist, timeout kill, output-cap
//! kill, and stdin plumbing. Unix exercises the host tools directly; Windows
//! re-enters this native test executable so the exact-image contract is tested
//! without adding another subprocess capability.

use fmn_platform::fs::{ATOMIC_DIRECTORY_COMPLETE_LEAF, FileSystem, FsNodeKind, StdFs, VirtualFs};
use fmn_platform::process::{
    FfmpegLocator, FfmpegLocatorError, MAX_FFMPEG_EXECUTABLE_BYTES, NativeExecutableArchitecture,
    NativeExecutableFormat, ProcessCancellation, ProcessOutcome, ProcessRunner, ProcessSpec,
    ProcessStdinLimits, ProcessTermination, ScriptedRunner, StdFfmpegLocator,
};
#[cfg(all(feature = "exact-process", any(unix, windows)))]
use fmn_platform::process::{ProcessError, ProcessMechanism};
use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

fn scratch(name: &str) -> PathBuf {
    let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!("caps_{name}"));
    std::fs::create_dir_all(&dir).expect("scratch dir");
    dir
}

fn locator_scratch(name: &str) -> std::io::Result<PathBuf> {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    for _ in 0..128 {
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(format!(
            "ffmpeg_locator_{name}_{}_{sequence}",
            std::process::id()
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not claim a unique ffmpeg locator scratch directory",
    ))
}

#[cfg(unix)]
fn short_locator_scratch(name: &str) -> std::io::Result<PathBuf> {
    static SEQUENCE: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

    for _ in 0..128 {
        let sequence = SEQUENCE.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        let dir = PathBuf::from("/tmp").join(format!(
            "fmn-locator-{name}-{}-{sequence}",
            std::process::id()
        ));
        match std::fs::create_dir(&dir) {
            Ok(()) => return Ok(dir),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
            Err(error) => return Err(error),
        }
    }
    Err(std::io::Error::new(
        std::io::ErrorKind::AlreadyExists,
        "could not claim a short locator scratch directory",
    ))
}

fn write_executable_bytes(path: &Path, bytes: &[u8]) {
    std::fs::write(path, bytes).expect("write executable fixture");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755))
            .expect("mark fixture executable");
    }
}

fn native_executable_fixture() -> Vec<u8> {
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
        bytes[20..24].copy_from_slice(&((2 * SEGMENT_BYTES + ENTRY_BYTES) as u32).to_le_bytes());
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
        bytes[optional + 60..optional + 64].copy_from_slice(&(HEADER_BYTES as u32).to_le_bytes());
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
    b"unsupported host executable fixture".to_vec()
}

fn write_executable(path: &Path) {
    write_executable_bytes(path, &native_executable_fixture());
}

#[test]
fn ffmpeg_locator_canonicalizes_explicit_identity_without_consulting_path() {
    let root = locator_scratch("explicit").expect("claim explicit locator scratch");
    let tool = root.join("custom-tool-name");
    write_executable(&tool);
    let locator = StdFfmpegLocator::from_search_path(Some(OsString::from(":relative")));

    let resolved = locator
        .locate_ffmpeg(&tool)
        .expect("absolute configuration bypasses PATH");
    assert_eq!(
        resolved.canonical_path(),
        std::fs::canonicalize(&tool)
            .expect("canonical fixture")
            .as_path()
    );
    let (mut opened, attestation) = resolved
        .open_current()
        .expect("issued identity revalidates through its opened handle");
    assert_eq!(
        attestation.architecture,
        if cfg!(target_arch = "x86_64") {
            NativeExecutableArchitecture::X86_64
        } else {
            NativeExecutableArchitecture::Aarch64
        }
    );
    #[cfg(any(target_os = "linux", target_os = "android"))]
    assert_eq!(attestation.format, NativeExecutableFormat::Elf64);
    #[cfg(target_os = "macos")]
    assert_eq!(attestation.format, NativeExecutableFormat::MachO64);
    #[cfg(windows)]
    assert_eq!(attestation.format, NativeExecutableFormat::Pe32Plus);
    assert_eq!(
        attestation.file_bytes,
        native_executable_fixture().len() as u64
    );
    assert_eq!(attestation.policy_version, 2);
    assert_eq!(
        resolved
            .attest_private_copy(&mut opened, &tool)
            .expect("same handle remains attestable"),
        attestation
    );

    #[cfg(unix)]
    {
        let not_executable = root.join("not-executable");
        std::fs::write(&not_executable, b"plain file").expect("write plain file");
        assert!(matches!(
            locator.locate_ffmpeg(&not_executable),
            Err(FfmpegLocatorError::NotExecutable { .. })
        ));
    }
}

#[test]
fn ffmpeg_locator_token_revalidates_replacement_and_bounds_source_size() {
    let root = locator_scratch("token_revalidation").expect("claim token scratch");
    let tool = root.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    });
    write_executable(&tool);
    let resolved = StdFfmpegLocator::default()
        .locate_ffmpeg(&tool)
        .expect("issue locator token");

    write_executable_bytes(&tool, b"#!/bin/sh\nexit 0\n");
    assert!(matches!(
        resolved.open_current(),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let oversized = root.join(if cfg!(windows) {
        "oversized.exe"
    } else {
        "oversized"
    });
    let file = std::fs::File::create(&oversized).expect("create sparse oversized candidate");
    file.set_len(MAX_FFMPEG_EXECUTABLE_BYTES + 1)
        .expect("size sparse candidate");
    drop(file);
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&oversized, std::fs::Permissions::from_mode(0o755))
            .expect("mark oversized candidate executable");
    }
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&oversized),
        Err(FfmpegLocatorError::ExecutableSizeLimit {
            bytes,
            max: MAX_FFMPEG_EXECUTABLE_BYTES,
            ..
        }) if bytes == MAX_FFMPEG_EXECUTABLE_BYTES + 1
    ));
}

#[test]
fn ffmpeg_locator_rejects_wrong_architecture_and_malformed_native_headers() {
    let root = locator_scratch("malformed_native").expect("claim malformed-image scratch");

    let mut wrong_arch = native_executable_fixture();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    wrong_arch[18..20].copy_from_slice(
        &(if cfg!(target_arch = "x86_64") {
            183_u16
        } else {
            62_u16
        })
        .to_le_bytes(),
    );
    #[cfg(target_os = "macos")]
    wrong_arch[4..8].copy_from_slice(
        &(if cfg!(target_arch = "x86_64") {
            0x0100_000c_u32
        } else {
            0x0100_0007_u32
        })
        .to_le_bytes(),
    );
    #[cfg(windows)]
    wrong_arch[0x84..0x86].copy_from_slice(
        &(if cfg!(target_arch = "x86_64") {
            0xaa64_u16
        } else {
            0x8664_u16
        })
        .to_le_bytes(),
    );
    let wrong_arch_path = root.join("wrong-architecture");
    write_executable_bytes(&wrong_arch_path, &wrong_arch);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&wrong_arch_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let mut malformed = native_executable_fixture();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    malformed[32..40].copy_from_slice(&u64::MAX.to_le_bytes());
    #[cfg(target_os = "macos")]
    malformed[20..24].copy_from_slice(&u32::MAX.to_le_bytes());
    #[cfg(windows)]
    malformed[0x94..0x96].copy_from_slice(&u16::MAX.to_le_bytes());
    let malformed_path = root.join("malformed-native");
    write_executable_bytes(&malformed_path, &malformed);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&malformed_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));
}

#[test]
fn ffmpeg_locator_requires_a_file_backed_executable_entry_point() {
    let root = locator_scratch("native_entry").expect("claim native-entry scratch");

    let mut non_executable_entry = native_executable_fixture();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    non_executable_entry[64 + 4..64 + 8].copy_from_slice(&4_u32.to_le_bytes());
    #[cfg(target_os = "macos")]
    non_executable_entry[32 + 72 + 60..32 + 72 + 64].copy_from_slice(&1_u32.to_le_bytes());
    #[cfg(windows)]
    non_executable_entry[0x188 + 36..0x188 + 40].copy_from_slice(&0x4000_0020_u32.to_le_bytes());
    let non_executable_path = root.join("non-executable-entry");
    write_executable_bytes(&non_executable_path, &non_executable_entry);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&non_executable_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let mut unmapped_entry = native_executable_fixture();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    unmapped_entry[24..32].copy_from_slice(&u64::MAX.to_le_bytes());
    #[cfg(target_os = "macos")]
    unmapped_entry[32 + 2 * 72 + 8..32 + 2 * 72 + 16].copy_from_slice(&u64::MAX.to_le_bytes());
    #[cfg(windows)]
    unmapped_entry[0x98 + 16..0x98 + 20].copy_from_slice(&0x3000_u32.to_le_bytes());
    let unmapped_path = root.join("unmapped-entry");
    write_executable_bytes(&unmapped_path, &unmapped_entry);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&unmapped_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let mut invalid_mapping = native_executable_fixture();
    #[cfg(any(target_os = "linux", target_os = "android"))]
    invalid_mapping[64 + 16..64 + 24].copy_from_slice(&0x1001_u64.to_le_bytes());
    #[cfg(target_os = "macos")]
    invalid_mapping[32 + 72 + 32..32 + 72 + 40].copy_from_slice(&0_u64.to_le_bytes());
    #[cfg(windows)]
    invalid_mapping[0x98 + 36..0x98 + 40].copy_from_slice(&0x201_u32.to_le_bytes());
    let invalid_mapping_path = root.join("invalid-mapping");
    write_executable_bytes(&invalid_mapping_path, &invalid_mapping);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&invalid_mapping_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    #[cfg(target_os = "macos")]
    {
        let mut missing_pagezero = native_executable_fixture();
        missing_pagezero[32 + 8..32 + 18].fill(0);
        let missing_pagezero_path = root.join("missing-hard-pagezero");
        write_executable_bytes(&missing_pagezero_path, &missing_pagezero);
        assert!(matches!(
            StdFfmpegLocator::default().locate_ffmpeg(&missing_pagezero_path),
            Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
        ));
    }
}

#[cfg(any(target_os = "linux", target_os = "android"))]
#[test]
fn ffmpeg_locator_rejects_malformed_gnu_property_notes() {
    const ELF_HEADER_BYTES: usize = 64;
    const PROGRAM_HEADER_BYTES: usize = 56;
    const PROPERTY_OFFSET: usize = ELF_HEADER_BYTES + 2 * PROGRAM_HEADER_BYTES;

    let root = locator_scratch("gnu_property").expect("claim GNU-property scratch");
    let mut malformed = native_executable_fixture();
    malformed.resize(PROPERTY_OFFSET + 16, 0);
    malformed[56..58].copy_from_slice(&2_u16.to_le_bytes());
    let property_header = &mut malformed[ELF_HEADER_BYTES + PROGRAM_HEADER_BYTES..PROPERTY_OFFSET];
    property_header[..4].copy_from_slice(&0x6474_e553_u32.to_le_bytes());
    property_header[8..16].copy_from_slice(&(PROPERTY_OFFSET as u64).to_le_bytes());
    property_header[32..40].copy_from_slice(&16_u64.to_le_bytes());
    property_header[40..48].copy_from_slice(&16_u64.to_le_bytes());
    malformed[PROPERTY_OFFSET..PROPERTY_OFFSET + 4].copy_from_slice(&4_u32.to_le_bytes());
    malformed[PROPERTY_OFFSET + 8..PROPERTY_OFFSET + 12].copy_from_slice(&5_u32.to_le_bytes());
    malformed[PROPERTY_OFFSET + 12..PROPERTY_OFFSET + 16].copy_from_slice(b"BAD\0");

    let malformed_path = root.join("malformed-gnu-property");
    write_executable_bytes(&malformed_path, &malformed);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&malformed_path),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));
}

#[test]
fn ffmpeg_locator_validates_the_complete_search_policy_before_lookup() {
    let root = locator_scratch("policy").expect("claim policy locator scratch");
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("create search directory");
    write_executable(&bin.join(if cfg!(windows) {
        "ffmpeg.exe"
    } else {
        "ffmpeg"
    }));

    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::SearchPathUnavailable)
    ));

    let empty_cases = [
        (
            std::env::join_paths([Path::new(""), bin.as_path()]).expect("leading-empty PATH"),
            0,
        ),
        (
            std::env::join_paths([bin.as_path(), Path::new("")]).expect("trailing-empty PATH"),
            1,
        ),
        (
            std::env::join_paths([bin.as_path(), Path::new(""), bin.as_path()])
                .expect("middle-empty PATH"),
            1,
        ),
    ];
    for (search_path, expected_index) in empty_cases {
        assert!(matches!(
            StdFfmpegLocator::from_search_path(Some(search_path))
                .locate_ffmpeg(Path::new("ffmpeg")),
            Err(FfmpegLocatorError::EmptySearchEntry { index })
                if index == expected_index
        ));
    }

    let relative =
        std::env::join_paths([bin.as_path(), Path::new("relative")]).expect("relative-entry PATH");
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(relative)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::RelativeSearchEntry { index: 1, .. })
    ));

    let parent_traversal = root.join("elsewhere").join("..");
    let parent = std::env::join_paths([bin.as_path(), parent_traversal.as_path()])
        .expect("parent-entry PATH");
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(parent)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::ParentTraversalSearchEntry { index: 1, .. })
    ));
}

#[cfg(unix)]
#[test]
fn ffmpeg_locator_skips_present_non_executables_in_search_order() {
    use std::os::unix::fs::symlink;

    let root = short_locator_scratch("nonexec").expect("claim short nonexec locator scratch");
    let broken = root.join("broken");
    let directory = root.join("directory");
    let special = root.join("special");
    let first = root.join("first");
    let second = root.join("second");
    std::fs::create_dir(&broken).expect("broken-link search directory");
    std::fs::create_dir(&directory).expect("directory-target search directory");
    std::fs::create_dir(&special).expect("special-file search directory");
    std::fs::create_dir(&first).expect("first search directory");
    std::fs::create_dir(&second).expect("second search directory");
    symlink(root.join("missing-target"), broken.join("ffmpeg")).expect("broken candidate link");
    symlink(&root, directory.join("ffmpeg")).expect("directory candidate link");
    let _socket = std::os::unix::net::UnixListener::bind(special.join("ffmpeg"))
        .expect("special-file candidate socket");
    std::fs::write(first.join("ffmpeg"), native_executable_fixture())
        .expect("non-executable native candidate");
    write_executable(&second.join("ffmpeg"));
    let search_path = std::env::join_paths([
        broken.as_path(),
        directory.as_path(),
        special.as_path(),
        first.as_path(),
        second.as_path(),
    ])
    .expect("ordered PATH");
    let locator = StdFfmpegLocator::from_search_path(Some(search_path));

    let resolved = locator
        .locate_ffmpeg(Path::new("ffmpeg"))
        .expect("later executable candidate");
    assert_eq!(
        resolved.canonical_path(),
        std::fs::canonicalize(second.join("ffmpeg"))
            .expect("canonical second candidate")
            .as_path()
    );

    use std::os::unix::fs::PermissionsExt as _;
    std::fs::set_permissions(first.join("ffmpeg"), std::fs::Permissions::from_mode(0o755))
        .expect("promote first candidate");
    assert_eq!(
        locator
            .locate_ffmpeg(Path::new("ffmpeg"))
            .expect("source-order candidate")
            .canonical_path(),
        std::fs::canonicalize(first.join("ffmpeg"))
            .expect("canonical first candidate")
            .as_path(),
        "the first valid candidate must win"
    );
    assert!(matches!(
        locator.locate_ffmpeg(Path::new("ffprobe")),
        Err(FfmpegLocatorError::UnsupportedConfiguredName { .. })
    ));
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[test]
fn ffmpeg_locator_refuses_interpreter_wrappers_and_skips_them_during_search() {
    let root = locator_scratch("wrapper").expect("claim wrapper locator scratch");
    let wrapper_bin = root.join("wrapper-bin");
    let native_bin = root.join("native-bin");
    std::fs::create_dir(&wrapper_bin).expect("create wrapper search directory");
    std::fs::create_dir(&native_bin).expect("create native search directory");
    let wrapper = wrapper_bin.join("ffmpeg");
    write_executable_bytes(&wrapper, b"#!/bin/sh\nprintf 'ffmpeg version wrapper\\n'\n");
    let native = native_bin.join("ffmpeg");
    write_executable(&native);

    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&wrapper),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let search_path = std::env::join_paths([wrapper_bin.as_path(), native_bin.as_path()])
        .expect("ordered wrapper/native PATH");
    let resolved = StdFfmpegLocator::from_search_path(Some(search_path))
        .locate_ffmpeg(Path::new("ffmpeg"))
        .expect("search skips an interpreter wrapper");
    assert_eq!(
        resolved.canonical_path(),
        std::fs::canonicalize(native)
            .expect("canonical native fixture")
            .as_path()
    );
}

#[cfg(unix)]
#[test]
fn ffmpeg_locator_canonicalizes_symlinks_and_resists_later_retargeting() {
    use std::os::unix::fs::symlink;

    let root = locator_scratch("symlink").expect("claim symlink locator scratch");
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("search directory");
    let first = root.join("ffmpeg-first");
    let second = root.join("ffmpeg-second");
    write_executable(&first);
    write_executable(&second);
    let searched = bin.join("ffmpeg");
    symlink(&first, &searched).expect("first ffmpeg symlink");
    let search_path = std::env::join_paths([bin.as_path()]).expect("search PATH");
    let locator = StdFfmpegLocator::from_search_path(Some(search_path));

    let first_resolution = locator
        .locate_ffmpeg(Path::new("ffmpeg"))
        .expect("resolve first target");
    let displaced = bin.join("ffmpeg-original-link");
    std::fs::rename(&searched, &displaced).expect("displace original symlink");
    symlink(&second, &searched).expect("retarget ffmpeg symlink");

    assert_eq!(
        first_resolution.canonical_path(),
        std::fs::canonicalize(&first)
            .expect("canonical first target")
            .as_path(),
        "retargeting the search symlink must not change an issued identity"
    );
    assert_eq!(
        locator
            .locate_ffmpeg(Path::new("ffmpeg"))
            .expect("resolve replacement target")
            .canonical_path(),
        std::fs::canonicalize(&second)
            .expect("canonical second target")
            .as_path()
    );
}

#[cfg(unix)]
#[test]
fn ffmpeg_locator_rejects_hostile_bytes_and_never_interprets_shell_text() {
    use std::os::unix::ffi::OsStringExt as _;

    let with_nul = OsString::from_vec(b"/tmp/fmn\0hostile".to_vec());
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(with_nul)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::MalformedSearchPath { .. })
    ));

    let non_utf8 = OsString::from_vec(b"/tmp/fmn-\xff".to_vec());
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(non_utf8)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::NonUtf8SearchEntry { index: 0 })
    ));

    let with_control = OsString::from("/tmp/fmn-\n-hostile");
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(with_control)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::ControlSearchEntry { index: 0 })
    ));

    let oversized = OsString::from(format!("/tmp/{}", "x".repeat(70 * 1024)));
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(oversized)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::SearchPathLimit {
            resource: "size",
            ..
        })
    ));

    let root = locator_scratch("shell_text").expect("claim hostile locator scratch");
    let marker = root.join("shell-was-interpreted");
    let hostile = OsString::from(format!("{};touch {}", root.display(), marker.display()));
    assert!(matches!(
        StdFfmpegLocator::from_search_path(Some(hostile)).locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::NotFound)
    ));
    assert!(
        !marker.exists(),
        "PATH bytes must be filesystem text, never a shell program"
    );

    let control_target = root.join("ffmpeg\ncontrol");
    write_executable(&control_target);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&control_target),
        Err(FfmpegLocatorError::InvalidCanonicalIdentity { .. })
    ));
}

#[cfg(windows)]
#[test]
fn windows_ffmpeg_search_uses_only_the_fixed_exe_leaf() {
    let root = locator_scratch("windows_name").expect("claim Windows locator scratch");
    let bin = root.join("bin");
    std::fs::create_dir(&bin).expect("search directory");
    let tool = bin.join("ffmpeg.exe");
    write_executable(&tool);
    let search_path = std::env::join_paths([bin.as_path()]).expect("search PATH");
    let locator = StdFfmpegLocator::from_search_path(Some(search_path));

    for configured in ["ffmpeg", "FFMPEG", "ffmpeg.exe", "FFMPEG.EXE"] {
        assert_eq!(
            locator
                .locate_ffmpeg(Path::new(configured))
                .expect("fixed Windows name")
                .canonical_path(),
            std::fs::canonicalize(&tool)
                .expect("canonical ffmpeg.exe")
                .as_path()
        );
    }
    assert!(matches!(
        locator.locate_ffmpeg(Path::new("ffmpeg.com")),
        Err(FfmpegLocatorError::UnsupportedConfiguredName { .. })
    ));

    let malformed = StdFfmpegLocator::from_search_path(Some(OsString::from("\"C:\\bin")));
    assert!(matches!(
        malformed.locate_ffmpeg(Path::new("ffmpeg")),
        Err(FfmpegLocatorError::MalformedSearchPath { .. })
    ));

    for extension in ["bat", "cmd"] {
        let wrapper = root.join(format!("custom-wrapper.{extension}"));
        write_executable_bytes(&wrapper, b"@echo off\r\necho ffmpeg version wrapper\r\n");
        assert!(matches!(
            locator.locate_ffmpeg(&wrapper),
            Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
        ));
    }

    let truncated = root.join("truncated-mz.exe");
    write_executable_bytes(&truncated, b"MZ");
    assert!(matches!(
        locator.locate_ffmpeg(&truncated),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let out_of_file = root.join("out-of-file-pe.exe");
    let mut out_of_file_bytes = vec![0_u8; 64];
    out_of_file_bytes[..2].copy_from_slice(b"MZ");
    out_of_file_bytes[0x3c..0x40].copy_from_slice(&u32::MAX.to_le_bytes());
    write_executable_bytes(&out_of_file, &out_of_file_bytes);
    assert!(matches!(
        locator.locate_ffmpeg(&out_of_file),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let bad_signature = root.join("bad-pe-signature.exe");
    let mut bad_signature_bytes = native_executable_fixture();
    bad_signature_bytes[0x80..0x84].copy_from_slice(b"PX\0\0");
    write_executable_bytes(&bad_signature, &bad_signature_bytes);
    assert!(matches!(
        locator.locate_ffmpeg(&bad_signature),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));
}

#[cfg(target_os = "macos")]
#[test]
fn macos_ffmpeg_locator_requires_a_complete_host_slice_in_universal_images() {
    let root = locator_scratch("mach_o_universal").expect("claim Mach-O locator scratch");
    let thin = native_executable_fixture();
    let slice_offset = 0x100_usize;
    let mut universal = vec![0_u8; slice_offset + thin.len()];
    universal[..4].copy_from_slice(&[0xca, 0xfe, 0xba, 0xbe]);
    universal[4..8].copy_from_slice(&1_u32.to_be_bytes());
    let cpu = if cfg!(target_arch = "x86_64") {
        0x0100_0007_u32
    } else {
        0x0100_000c_u32
    };
    universal[8..12].copy_from_slice(&cpu.to_be_bytes());
    let subtype = if cfg!(target_arch = "x86_64") {
        3_u32
    } else {
        0_u32
    };
    universal[12..16].copy_from_slice(&subtype.to_be_bytes());
    universal[16..20].copy_from_slice(&(slice_offset as u32).to_be_bytes());
    universal[20..24].copy_from_slice(&(thin.len() as u32).to_be_bytes());
    universal[slice_offset..].copy_from_slice(&thin);

    let candidate = root.join("ffmpeg-universal");
    write_executable_bytes(&candidate, &universal);
    let resolved = StdFfmpegLocator::default()
        .locate_ffmpeg(&candidate)
        .expect("complete universal host slice");
    let (_file, attestation) = resolved.open_current().expect("attest universal image");
    assert_eq!(attestation.format, NativeExecutableFormat::MachOUniversal);

    let mut host_slice_free = universal;
    host_slice_free[8..12].copy_from_slice(
        &(if cfg!(target_arch = "x86_64") {
            0x0100_000c_u32
        } else {
            0x0100_0007_u32
        })
        .to_be_bytes(),
    );
    let missing = root.join("ffmpeg-no-host-slice");
    write_executable_bytes(&missing, &host_slice_free);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&missing),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));

    let legacy = root.join("ffmpeg-thin-32");
    write_executable_bytes(&legacy, &[0xce, 0xfa, 0xed, 0xfe]);
    assert!(matches!(
        StdFfmpegLocator::default().locate_ffmpeg(&legacy),
        Err(FfmpegLocatorError::UnsupportedExecutableFormat { .. })
    ));
}

#[test]
fn std_fs_atomic_write_round_trips_and_lists_sorted() {
    let dir = scratch("stdfs");
    let fs = StdFs;
    let deep = dir.join("a/b/file.bin");
    fs.write_atomic(&deep, b"payload")
        .expect("atomic write creates parents");
    assert_eq!(fs.read(&deep).expect("read"), b"payload");
    fs.write_atomic(&deep, b"replaced").expect("atomic replace");
    assert_eq!(fs.read_to_string(&deep).expect("read"), "replaced");
    assert_eq!(
        fs.node_kind_no_follow(&deep).expect("classify"),
        Some(FsNodeKind::RegularFile)
    );
    fs.write_atomic(&dir.join("a/z.txt"), b"z").expect("write");
    fs.write_atomic(&dir.join("a/a.txt"), b"a").expect("write");
    let listed = fs.list_dir(&dir.join("a")).expect("list");
    let names: Vec<String> = listed
        .iter()
        .filter_map(|p| p.file_name().map(|n| n.to_string_lossy().into_owned()))
        .collect();
    assert_eq!(names, vec!["a.txt", "b", "z.txt"], "sorted listing");
    assert!(fs.exists(&deep));
    assert!(!fs.exists(&dir.join("missing")));
    assert_eq!(
        fs.node_kind_no_follow(&dir.join("missing"))
            .expect("classify absence"),
        None
    );
}

#[test]
fn std_fs_create_new_and_removal_lifecycle() {
    let dir = scratch("stdfs_lifecycle");
    let fs = StdFs;
    // CARGO_TARGET_TMPDIR persists across runs; start from a clean slate so
    // create_new's create-if-absent assertions are re-runnable.
    let _ = fs.remove_dir_all(&dir);
    std::fs::create_dir(&dir).expect("create lifecycle root");

    let empty = dir.join("empty");
    assert!(fs.create_dir(&empty).expect("create exact directory"));
    assert!(
        !fs.create_dir(&empty)
            .expect("existing exact directory is stable")
    );
    assert_eq!(
        fs.node_kind_no_follow(&empty).expect("classify directory"),
        Some(FsNodeKind::Directory)
    );

    // create_new: create-if-absent with full contents, atomically visible.
    let lock = dir.join("locks/maintenance.lock");
    assert!(fs.create_new(&lock, b"holder-1").expect("create"));
    assert!(
        !fs.create_new(&lock, b"holder-2")
            .expect("second create loses")
    );
    assert_eq!(fs.read(&lock).expect("read"), b"holder-1");
    // No temp residue from either attempt.
    let residue: Vec<_> = fs
        .list_dir(&dir.join("locks"))
        .expect("list")
        .into_iter()
        .filter(|p| {
            p.file_name()
                .is_some_and(|n| n.to_string_lossy().starts_with(".fmn-new."))
        })
        .collect();
    assert!(residue.is_empty(), "temp residue: {residue:?}");

    // remove_file: gone, and a second removal is a precise NotFound.
    fs.remove_file(&lock).expect("remove");
    assert!(!fs.exists(&lock));
    assert!(matches!(
        fs.remove_file(&lock),
        Err(fmn_platform::fs::FsError::NotFound { .. })
    ));

    // remove_dir_all: subtree gone, siblings untouched.
    fs.write_atomic(&dir.join("ns/a/objects/x"), b"x").unwrap();
    fs.write_atomic(&dir.join("ns/b/objects/y"), b"y").unwrap();
    fs.remove_dir_all(&dir.join("ns/a")).expect("purge");
    assert!(!fs.exists(&dir.join("ns/a")));
    assert_eq!(fs.read(&dir.join("ns/b/objects/y")).unwrap(), b"y");
}

#[test]
fn streaming_atomic_capabilities_separate_prepare_from_publication() {
    let fs = Arc::new(VirtualFs::new());
    let destination = PathBuf::from("/artifacts/frame.png");
    fs.insert(&destination, b"old".to_vec());

    let mut writer = fs
        .clone()
        .begin_atomic_file(&destination)
        .expect("begin private file");
    writer.write(b"new-").expect("first file chunk");
    writer.write(b"bytes").expect("second file chunk");
    let prepared = writer.prepare().expect("prepare private file");
    assert_eq!(fs.read(&destination).expect("old destination"), b"old");
    drop(prepared);
    assert_eq!(
        fs.read(&destination).expect("drop preserves destination"),
        b"old"
    );

    let mut writer = fs
        .clone()
        .begin_atomic_file(&destination)
        .expect("begin replacement file");
    writer.write(b"replacement").expect("replacement bytes");
    let prepared = writer.prepare().expect("prepare replacement");
    assert_eq!(fs.read(&destination).expect("still old"), b"old");
    prepared.commit().expect("commit replacement");
    assert_eq!(
        fs.read(&destination).expect("published replacement"),
        b"replacement"
    );
}

#[test]
fn std_streaming_atomic_file_and_directory_publish_only_at_commit() {
    let root = scratch("stdfs_streaming_atomic");
    let _ = StdFs.remove_dir_all(&root);
    std::fs::create_dir(&root).expect("fresh streaming root");
    let fs = Arc::new(StdFs);

    let file = root.join("artifact.bin");
    std::fs::write(&file, b"old").expect("old artifact");
    let mut writer = fs
        .clone()
        .begin_atomic_file(&file)
        .expect("begin host private file");
    writer.write(b"new").expect("host private bytes");
    let prepared = writer.prepare().expect("prepare host private file");
    assert_eq!(std::fs::read(&file).expect("still old"), b"old");
    prepared.commit().expect("publish host file");
    assert_eq!(std::fs::read(&file).expect("host replacement"), b"new");

    let generation = root.join("frames");
    let mut writer = fs
        .clone()
        .begin_atomic_directory(&generation)
        .expect("begin host generation");
    writer
        .write_file(Path::new("frame_0001.png"), b"one")
        .expect("first host child");
    writer
        .write_file(Path::new("frame_0002.png"), b"two")
        .expect("second host child");
    let prepared = writer.prepare().expect("prepare host generation");
    assert!(!generation.exists());
    prepared.commit().expect("publish host generation");
    assert_eq!(
        std::fs::read(generation.join(ATOMIC_DIRECTORY_COMPLETE_LEAF)).expect("completion marker"),
        b"fmn-atomic-directory-v1\n"
    );
    assert_eq!(
        std::fs::read(generation.join("frame_0001.png")).expect("first host child"),
        b"one"
    );
    assert_eq!(
        std::fs::read(generation.join("frame_0002.png")).expect("second host child"),
        b"two"
    );

    let mut contender = fs
        .clone()
        .begin_atomic_directory(&generation)
        .expect("begin host contender");
    contender
        .write_file(Path::new("frame_0001.png"), b"replacement")
        .expect("private host contender");
    assert!(
        contender
            .prepare()
            .expect("prepare host contender")
            .commit()
            .is_err(),
        "host generation must be no-clobber"
    );
    assert_eq!(
        std::fs::read(generation.join("frame_0001.png")).expect("stable host child"),
        b"one"
    );
}

#[test]
fn std_atomic_directory_concurrent_commit_is_no_clobber() {
    let root = scratch("stdfs_atomic_directory_race");
    let _ = StdFs.remove_dir_all(&root);
    std::fs::create_dir(&root).expect("fresh race root");
    let fs = Arc::new(StdFs);
    let destination = root.join("frames");

    let prepare = |bytes: &'static [u8]| {
        let mut writer = fs
            .clone()
            .begin_atomic_directory(&destination)
            .expect("begin contender");
        writer
            .write_file(Path::new("frame.png"), bytes)
            .expect("stage contender");
        writer.prepare().expect("prepare contender")
    };
    let first = prepare(b"first");
    let second = prepare(b"second");
    let barrier = Arc::new(std::sync::Barrier::new(3));
    let spawn = |prepared: Box<dyn fmn_platform::fs::PreparedAtomicDirectory>| {
        let barrier = barrier.clone();
        std::thread::spawn(move || {
            barrier.wait();
            prepared.commit()
        })
    };
    let first = spawn(first);
    let second = spawn(second);
    barrier.wait();
    let first = first.join().expect("first contender thread");
    let second = second.join().expect("second contender thread");

    assert_ne!(
        first.is_ok(),
        second.is_ok(),
        "exactly one no-clobber commit must win"
    );
    assert!(destination.join(ATOMIC_DIRECTORY_COMPLETE_LEAF).exists());
    let frame = std::fs::read(destination.join("frame.png")).expect("winner frame");
    assert!(
        frame == b"first" || frame == b"second",
        "generation mixed unexpected bytes: {frame:?}"
    );
}

#[test]
fn atomic_directory_generation_is_complete_no_clobber_and_leaf_safe() {
    let fs = Arc::new(VirtualFs::new());
    let destination = PathBuf::from("/frames");
    let mut writer = fs
        .clone()
        .begin_atomic_directory(&destination)
        .expect("begin generation");
    for leaf in [
        "../escape",
        "nested/frame.png",
        "bad\\name",
        "C:",
        "bad:name",
        "_leading",
        "trailing.",
        "trailing ",
        "CON",
        "con.txt",
        "COM1.png",
        "LPT9.gif",
        ATOMIC_DIRECTORY_COMPLETE_LEAF,
    ] {
        assert!(
            writer.write_file(Path::new(leaf), b"x").is_err(),
            "unsafe cross-platform leaf was accepted: {leaf:?}"
        );
    }
    writer
        .write_file(Path::new("frame_0001.png"), b"one")
        .expect("first child");
    assert!(
        writer
            .write_file(Path::new("frame_0001.png"), b"duplicate")
            .is_err()
    );
    writer
        .write_file(Path::new("frame_0002.png"), b"two")
        .expect("second child");
    let prepared = writer.prepare().expect("prepare generation");
    assert!(!fs.exists(&destination));
    prepared.commit().expect("publish complete generation");
    assert_eq!(
        fs.list_dir(&destination).expect("generation listing"),
        vec![
            destination.join(ATOMIC_DIRECTORY_COMPLETE_LEAF),
            destination.join("frame_0001.png"),
            destination.join("frame_0002.png"),
        ]
    );

    let mut contender = fs
        .clone()
        .begin_atomic_directory(&destination)
        .expect("begin competing generation");
    contender
        .write_file(Path::new("frame_0001.png"), b"new")
        .expect("private competing child");
    let contender = contender.prepare().expect("prepare competitor");
    assert!(contender.commit().is_err(), "existing generation wins");
    assert_eq!(
        fs.read(&destination.join("frame_0001.png"))
            .expect("original child"),
        b"one"
    );
    assert_eq!(
        fs.read(&destination.join("frame_0002.png"))
            .expect("original tail"),
        b"two"
    );
}

#[cfg(unix)]
#[test]
fn std_fs_classifies_links_without_following_them() {
    let dir = scratch("stdfs_no_follow_kind");
    let _ = StdFs.remove_dir_all(&dir);
    std::fs::create_dir(&dir).expect("create root");
    let target = dir.join("target");
    std::fs::write(&target, b"target").expect("write target");
    let link = dir.join("link");
    std::os::unix::fs::symlink(&target, &link).expect("create link");

    assert_eq!(
        StdFs.node_kind_no_follow(&link).expect("classify link"),
        Some(FsNodeKind::Link)
    );
    assert!(
        StdFs.create_dir(&link).is_err(),
        "exact directory creation must not accept a link winner"
    );
    assert_eq!(
        StdFs.node_kind_no_follow(&target).expect("classify target"),
        Some(FsNodeKind::RegularFile)
    );
}

fn spec(program: &str, argv: &[&str]) -> ProcessSpec {
    ProcessSpec {
        program: PathBuf::from(program),
        argv: argv.iter().map(ToString::to_string).collect(),
        env: Vec::new(),
        cwd: None,
        stdin: None,
        timeout: Duration::from_secs(10),
        max_output_bytes: 1 << 20,
    }
}

/// Deterministic absolute path of a host utility used by the spawn tests.
///
/// Certified Linux uses the usr-merged `/usr/bin` paths. Certified macOS
/// keeps `echo`, `cat`, and `sleep` in `/bin`, while the remaining fixtures
/// live in `/usr/bin`. The exact-image runner still receives only an absolute
/// native-image path and an argument vector; no shell or `PATH` lookup occurs.
#[cfg(all(feature = "exact-process", target_os = "linux"))]
fn host_bin(name: &str) -> String {
    format!("/usr/bin/{name}")
}

#[cfg(all(feature = "exact-process", target_os = "macos"))]
fn host_bin(name: &str) -> String {
    let directory = match name {
        "cat" | "echo" | "sleep" => "/bin",
        _ => "/usr/bin",
    };
    format!("{directory}/{name}")
}

#[cfg(all(
    feature = "exact-process",
    unix,
    not(any(target_os = "linux", target_os = "macos"))
))]
fn host_bin(name: &str) -> String {
    let candidates = [format!("/usr/bin/{name}"), format!("/bin/{name}")];
    candidates
        .into_iter()
        .find(|candidate| PathBuf::from(candidate).exists())
        .unwrap_or_else(|| format!("/nonexistent/fmn-platform-{name}"))
}

#[cfg(all(feature = "exact-process", unix))]
mod std_runner {
    use super::*;
    use fmn_platform::process::StdProcessRunner;
    use std::sync::mpsc;
    use std::thread;

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    #[test]
    fn exact_image_mechanism_identity_and_policy_are_stable() {
        assert_eq!(
            StdProcessRunner.mechanism(),
            ProcessMechanism::PosixSpawnAbsoluteProcessGroup
        );
        assert_eq!(
            StdProcessRunner.mechanism().identity(),
            "posix_spawn.absolute_path.new_process_group"
        );
        assert_eq!(StdProcessRunner.mechanism().policy_version(), 1);
    }

    #[test]
    fn argv_only_echo_succeeds() {
        let out = StdProcessRunner
            .run(&spec(&host_bin("echo"), &["hello", "argv world"]))
            .expect("run");
        assert!(out.success());
        // The two argv entries arrive as two arguments — no shell splitting.
        assert_eq!(out.stdout, b"hello argv world\n");
        assert!(matches!(out.termination, ProcessTermination::Exited(_)));
    }

    #[test]
    fn nonzero_exit_is_an_outcome_not_an_error() {
        let out = StdProcessRunner
            .run(&spec(&host_bin("false"), &[]))
            .expect("run");
        assert_eq!(out.termination, ProcessTermination::Exited(Some(1)));
        assert!(!out.success());
    }

    #[test]
    fn environment_is_cleared_then_allowlisted() {
        let mut s = spec(&host_bin("env"), &[]);
        s.env = vec![("FMN_ALLOWED".to_string(), "yes".to_string())];
        let out = StdProcessRunner.run(&s).expect("run");
        let text = String::from_utf8_lossy(&out.stdout);
        assert!(text.contains("FMN_ALLOWED=yes"), "{text}");
        // Nothing ambient leaks: PATH/HOME are gone.
        assert!(!text.contains("PATH="), "ambient PATH leaked: {text}");
        assert!(!text.contains("HOME="), "ambient HOME leaked: {text}");
    }

    #[test]
    fn stdin_bytes_flow_through() {
        let mut s = spec(&host_bin("cat"), &[]);
        s.stdin = Some(b"through the capability".to_vec());
        let out = StdProcessRunner.run(&s).expect("run");
        assert!(out.success());
        assert_eq!(out.stdout, b"through the capability");
    }

    #[test]
    fn timeout_kills_the_complete_process_group() {
        let script = scratch("process_tree").join("ffmpeg-with-descendant.sh");
        std::fs::write(&script, "sleep 5\n").expect("write descendant fixture");
        let script = script.to_str().expect("UTF-8 fixture path");
        let mut s = spec("/bin/sh", &[script]);
        s.timeout = Duration::from_millis(200);
        let started = std::time::Instant::now();
        let out = StdProcessRunner.run(&s).expect("run");
        assert_eq!(out.termination, ProcessTermination::TimedOut);
        assert!(!out.success());
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "process-tree kill was not prompt: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn exited_group_leader_cannot_disarm_descendant_timeout() {
        let script = scratch("process_tree").join("ffmpeg-with-background-descendant.sh");
        std::fs::write(&script, "sleep 5 &\nexit 0\n")
            .expect("write background-descendant fixture");
        let script = script.to_str().expect("UTF-8 fixture path");
        let mut s = spec("/bin/sh", &[script]);
        s.timeout = Duration::from_millis(200);
        let started = std::time::Instant::now();
        let out = StdProcessRunner.run(&s).expect("run");
        assert_eq!(out.termination, ProcessTermination::TimedOut);
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "exited leader disarmed process-tree kill: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn successful_exit_kills_redirected_descendants_before_return() {
        let dir = scratch("process_tree");
        let script = dir.join("ffmpeg-with-redirected-descendant.sh");
        let marker = dir.join(format!(
            "redirected-descendant-leak-{}.txt",
            std::process::id()
        ));
        assert!(!marker.exists(), "test marker must start absent");
        std::fs::write(
            &script,
            "(sleep 1; printf leaked > \"$1\") >/dev/null 2>&1 &\nexit 0\n",
        )
        .expect("write redirected-descendant fixture");
        let script = script.to_str().expect("UTF-8 fixture path");
        let marker_arg = marker.to_str().expect("UTF-8 marker path");
        let started = std::time::Instant::now();
        let out = StdProcessRunner
            .run(&spec("/bin/sh", &[script, marker_arg]))
            .expect("run");
        assert!(out.success(), "group leader should retain its zero exit");
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "successful process-tree cleanup was not prompt: {:?}",
            started.elapsed()
        );
        // A leaked descendant would create the marker after its delayed write.
        std::thread::sleep(Duration::from_millis(1_200)); // ubs:ignore bounded process-lifecycle probe
        assert!(
            !marker.exists(),
            "redirected descendant outlived successful group leader"
        );
    }

    #[test]
    fn output_cap_kills_and_truncates() {
        // `yes` produces unbounded output; the cap must stop it long before
        // the timeout.
        let mut s = spec(&host_bin("yes"), &[]);
        s.timeout = Duration::from_secs(30);
        s.max_output_bytes = 64 * 1024;
        let started = std::time::Instant::now();
        let out = StdProcessRunner.run(&s).expect("run");
        assert_eq!(out.termination, ProcessTermination::OutputLimitExceeded);
        assert!(out.stdout.len() as u64 <= s.max_output_bytes);
        assert!(
            started.elapsed() < Duration::from_secs(10),
            "cap kill was not prompt: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn spawn_failure_is_a_named_mechanism_error() {
        let err = StdProcessRunner
            .run(&spec("/nonexistent/fmn-no-such-program", &[]))
            .unwrap_err();
        assert!(err.to_string().contains("fmn-no-such-program"));
    }

    #[test]
    fn executable_text_without_shebang_never_issues_an_interpreter() {
        use std::os::unix::fs::PermissionsExt as _;

        let path = scratch("exact_image_no_shebang").join("ffmpeg-text-fixture");
        std::fs::write(&path, "printf 'FMN_INTERPRETER_FALLBACK_RAN\\n'\nexit 91\n")
            .expect("write executable-text fixture");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755))
            .expect("mark executable-text fixture executable");

        let error = StdProcessRunner
            .run(&spec(path.to_str().expect("UTF-8 fixture path"), &[]))
            .expect_err("ENOEXEC must remain a spawn error");
        assert!(
            matches!(error, ProcessError::Spawn { .. }),
            "unexpected no-shebang refusal: {error}"
        );
    }

    #[test]
    fn working_directory_requests_are_refused_before_spawn() {
        let mut s = spec(&host_bin("true"), &[]);
        s.cwd = Some(scratch("exact_image_cwd_refusal"));
        let error = StdProcessRunner.run(&s).expect_err("cwd must be refused");
        assert!(
            matches!(error, ProcessError::WorkingDirectoryUnsupported { .. }),
            "unexpected cwd refusal: {error}"
        );
    }

    #[test]
    fn external_cancellation_unblocks_a_full_stdin_pipe_and_reaps_the_child() {
        use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
        use std::time::Instant;

        // The child never reads its stdin, a Unix socket. How much the kernel
        // accepts before a write blocks is host-tuned: 8 KiB on macOS, about
        // 200 KiB on a stock Linux, 16 MiB where net.core.wmem_default is
        // raised. No fixed payload is known to fill it (fm-3u4o: an 8 MiB
        // write completed before cancellation on such a host). So the writer
        // streams chunks until one blocks, and cancellation is sent only once
        // progress has stalled with that write in flight.
        const CHUNK: usize = 1 << 20;
        const MAX_TOTAL: u64 = 1 << 30;
        const STALL: Duration = Duration::from_millis(500);
        const FILL_DEADLINE: Duration = Duration::from_secs(20);
        // The child outlives the fill deadline (it sleeps 30 s), and so must
        // its supervisor timeout. At `spec`'s 10 s, a fill that took longer
        // would end with the supervisor killing the child, and the write
        // failing on the closed pipe before cancellation was ever sent.
        let mut sleeper = spec(&host_bin("sleep"), &["30"]);
        sleeper.timeout = Duration::from_secs(60);
        let cancellation = ProcessCancellation::new();
        let process = StdProcessRunner
            .start(
                &sleeper,
                cancellation.clone(),
                ProcessStdinLimits::new(CHUNK as u64, MAX_TOTAL),
            )
            .expect("start sleeping child");
        let delivered = Arc::new(AtomicU64::new(0));
        let writing = Arc::new(AtomicBool::new(false));
        // The write's outcome is reported the moment it returns, before
        // `finish` waits on the child, so a write that ends without being
        // unblocked is seen as returning early rather than after the cancel.
        let (write_tx, write_rx) = mpsc::sync_channel(1);
        let (finish_tx, finish_rx) = mpsc::sync_channel(1);
        {
            let delivered = Arc::clone(&delivered);
            let writing = Arc::clone(&writing);
            let observer = cancellation.clone();
            thread::spawn(move || {
                let mut process = process;
                let chunk = vec![0u8; CHUNK];
                writing.store(true, Ordering::Release);
                let write = loop {
                    if let Err(error) = process.write_stdin(&chunk) {
                        break error;
                    }
                    delivered.fetch_add(CHUNK as u64, Ordering::AcqRel);
                };
                // A write that cancellation unblocked returns after the
                // request; one that fails on its own (a non-blocking pipe, a
                // write timeout) returns before it.
                let after_cancel = observer.is_cancelled();
                let _ = write_tx.send((write, after_cancel));
                let _ = finish_tx.send(process.finish());
            });
        }

        // A full pipe: the writer is running, has not returned, and has made
        // no progress for STALL.
        let deadline = Instant::now() + FILL_DEADLINE;
        let mut seen = delivered.load(Ordering::Acquire);
        let mut unchanged_since = Instant::now();
        loop {
            assert!(
                Instant::now() < deadline,
                "the stdin pipe never filled ({seen} bytes accepted)"
            );
            let early = write_rx.try_recv().ok().map(|(write, _)| write.to_string());
            assert_eq!(
                early, None,
                "the writer returned before cancellation after {seen} bytes"
            );
            thread::sleep(Duration::from_millis(10));
            let now = delivered.load(Ordering::Acquire);
            if now != seen || !writing.load(Ordering::Acquire) {
                seen = now;
                unchanged_since = Instant::now();
            } else if unchanged_since.elapsed() >= STALL {
                break;
            }
        }

        // ubs:ignore - elapsed-time assertion, not security-token generation.
        let started = Instant::now();
        cancellation.cancel();
        let (write, after_cancel) = write_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("cancellation must unblock a writer blocked on a full stdin pipe");
        assert!(
            after_cancel,
            "the write failed before cancellation was requested: {write}"
        );
        // The blocked write returns only because the killed child's end of
        // the pipe closed, never through the cancelled-before-write check.
        let pipe_closed = matches!(
            &write,
            ProcessError::Plumbing { detail, .. } if detail.starts_with("stdin write failed")
        );
        assert!(
            pipe_closed,
            "the blocked write must fail on the closed pipe: {write}"
        );
        assert!(delivered.load(Ordering::Acquire) < MAX_TOTAL);
        let finish = finish_rx
            .recv_timeout(Duration::from_secs(10))
            .expect("the cancelled session finishes");
        assert_eq!(
            finish.expect("supervisor outcome").termination,
            ProcessTermination::Cancelled
        );
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "cancellation did not promptly unblock and reap"
        );
    }

    #[test]
    fn dropping_a_live_session_cancels_and_reaps_before_returning() {
        // ubs:ignore - elapsed-time assertion, not security-token generation.
        let started = std::time::Instant::now();
        let process = StdProcessRunner
            .start(
                &spec(&host_bin("sleep"), &["30"]),
                ProcessCancellation::new(),
                ProcessStdinLimits::new(1, 1),
            )
            .expect("start sleeping child");
        drop(process);
        assert!(
            started.elapsed() < Duration::from_secs(3),
            "drop did not promptly cancel and reap"
        );
    }
}

#[cfg(all(feature = "exact-process", windows))]
mod windows_std_runner {
    use super::*;
    use fmn_platform::process::StdProcessRunner;
    use std::io::Read as _;

    const DESCENDANT_MARKER: &str = "FMN_EXACT_IMAGE_DESCENDANT_MARKER";
    const DESCENDANT_READY: &str = "FMN_EXACT_IMAGE_DESCENDANT_READY";
    const FIXTURE_MODE: &str = "FMN_EXACT_IMAGE_FIXTURE_MODE";
    const FIXTURE_TEST: &str = "windows_std_runner::exact_image_fixture_child";

    fn fixture_spec(mode: &str) -> ProcessSpec {
        ProcessSpec {
            program: std::env::current_exe().expect("absolute native test executable"),
            argv: vec![
                "--exact".to_owned(),
                FIXTURE_TEST.to_owned(),
                "--nocapture".to_owned(),
            ],
            env: vec![
                (FIXTURE_MODE.to_owned(), mode.to_owned()),
                ("FMN_ALLOWED".to_owned(), "yes".to_owned()),
            ],
            cwd: None,
            stdin: None,
            timeout: Duration::from_secs(10),
            max_output_bytes: 1 << 20,
        }
    }

    #[test]
    fn exact_image_fixture_child() {
        let Ok(mode) = std::env::var(FIXTURE_MODE) else {
            // The ordinary parent test pass discovers this fixture too.
            return;
        };
        match mode.as_str() {
            "stdio" => {
                let mut stdin = Vec::new();
                std::io::stdin()
                    .read_to_end(&mut stdin)
                    .expect("read exact-image stdin");
                assert_eq!(
                    std::env::var("FMN_ALLOWED").expect("allowlisted environment entry"),
                    "yes"
                );
                assert!(
                    std::env::var_os("PATH").is_none(),
                    "the child inherited ambient PATH"
                );
                println!("FMN_STDIN={}", String::from_utf8_lossy(&stdin));
                eprintln!("FMN_STDERR=owned");
            }
            "job-parent" => {
                let marker = std::env::var_os(DESCENDANT_MARKER).expect("descendant marker path");
                let ready = std::env::var_os(DESCENDANT_READY).expect("descendant ready path");
                // Deliberately never waited on: the fixture exists to prove
                // the runner's tree kill reaps this descendant, not us.
                #[allow(clippy::zombie_processes)]
                let descendant = std::process::Command::new(
                    std::env::current_exe().expect("native descendant executable"),
                )
                .args(["--exact", FIXTURE_TEST, "--nocapture"])
                .env_clear()
                .env(FIXTURE_MODE, "descendant")
                .env(DESCENDANT_MARKER, marker)
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn()
                .expect("spawn native job descendant");
                std::fs::write(ready, descendant.id().to_string())
                    .expect("record descendant readiness");
                std::thread::sleep(Duration::from_secs(30));
            }
            "descendant" => {
                let marker = std::env::var_os(DESCENDANT_MARKER).expect("descendant marker path");
                std::thread::sleep(Duration::from_secs(5));
                std::fs::write(marker, b"leaked").expect("write descendant leak marker");
            }
            "sleep" => std::thread::sleep(Duration::from_secs(30)),
            other => {
                eprintln!("unknown exact-image fixture mode {other}");
                std::process::exit(2);
            }
        }
    }

    #[test]
    fn exact_image_mechanism_environment_and_stdio_are_functional() {
        assert_eq!(
            StdProcessRunner.mechanism(),
            ProcessMechanism::WindowsCreateProcessJobList
        );
        assert_eq!(
            StdProcessRunner.mechanism().identity(),
            "create_process_w.explicit_application.atomic_job_list"
        );
        assert_eq!(StdProcessRunner.mechanism().policy_version(), 1);

        let mut child = fixture_spec("stdio");
        child.stdin = Some(b"shell text stays data: %PATH% & whoami".to_vec());
        let outcome = StdProcessRunner.run(&child).expect("run native image");
        assert!(outcome.success(), "{outcome:?}");
        let stdout = String::from_utf8_lossy(&outcome.stdout);
        let stderr = String::from_utf8_lossy(&outcome.stderr);
        assert!(
            stdout.contains("FMN_STDIN=shell text stays data: %PATH% & whoami"),
            "{stdout}"
        );
        assert!(stderr.contains("FMN_STDERR=owned"), "{stderr}");
    }

    #[test]
    fn executable_text_with_exe_suffix_never_issues_an_interpreter() {
        let root = locator_scratch("windows_exact_image_text").expect("claim exact-image scratch");
        let marker = root.join("interpreter-ran");
        let fixture = root.join("ffmpeg-text-fixture.exe");
        write_executable_bytes(
            &fixture,
            format!("@echo off\r\necho interpreted>\"{}\"\r\n", marker.display()).as_bytes(),
        );
        let error = StdProcessRunner
            .run(&ProcessSpec {
                program: fixture,
                argv: Vec::new(),
                env: Vec::new(),
                cwd: None,
                stdin: None,
                timeout: Duration::from_secs(10),
                max_output_bytes: 1 << 20,
            })
            .expect_err("non-PE executable text must remain a spawn error");
        assert!(
            matches!(error, ProcessError::Spawn { .. }),
            "unexpected executable-text refusal: {error}"
        );
        assert!(
            !marker.exists(),
            "an interpreter executed rejected executable text"
        );
    }

    #[test]
    fn timeout_terminates_and_reaps_the_job_child() {
        let mut child = fixture_spec("sleep");
        child.timeout = Duration::from_millis(200);
        let started = std::time::Instant::now();
        let outcome = StdProcessRunner
            .run(&child)
            .expect("supervise sleeping native child");
        assert_eq!(outcome.termination, ProcessTermination::TimedOut);
        assert!(
            started.elapsed() < Duration::from_secs(5),
            "job termination was not prompt: {:?}",
            started.elapsed()
        );
    }

    #[test]
    fn timeout_terminates_descendants_in_the_atomic_job() {
        let root =
            locator_scratch("windows_exact_image_descendant").expect("claim descendant scratch");
        let ready = root.join("descendant-ready");
        let marker = root.join("descendant-leaked");
        let mut child = fixture_spec("job-parent");
        child.env.extend([
            (
                DESCENDANT_READY.to_owned(),
                ready.to_string_lossy().into_owned(),
            ),
            (
                DESCENDANT_MARKER.to_owned(),
                marker.to_string_lossy().into_owned(),
            ),
        ]);
        child.timeout = Duration::from_secs(3);
        let outcome = StdProcessRunner
            .run(&child)
            .expect("supervise native process tree");
        assert_eq!(outcome.termination, ProcessTermination::TimedOut);
        assert!(ready.exists(), "the descendant probe never started");

        std::thread::sleep(Duration::from_millis(5_500));
        assert!(
            !marker.exists(),
            "a descendant escaped exact-image Job Object termination"
        );
    }

    #[test]
    fn working_directory_requests_are_refused_before_spawn() {
        let mut child = fixture_spec("stdio");
        child.cwd = Some(scratch("windows_exact_image_cwd_refusal"));
        let error = StdProcessRunner
            .run(&child)
            .expect_err("working directory must be refused");
        assert!(
            matches!(error, ProcessError::WorkingDirectoryUnsupported { .. }),
            "unexpected working-directory refusal: {error}"
        );
    }
}

#[test]
fn relative_program_paths_are_refused_by_contract() {
    // Both runners enforce it — PATH resolution is unreachable through the
    // capability (D2: the boundary resolves and fingerprints its one tool).
    let s = spec("echo", &["hi"]);
    let err = ScriptedRunner::new().run(&s).unwrap_err();
    assert!(err.to_string().contains("not absolute"), "{err}");
    #[cfg(all(feature = "exact-process", any(unix, windows)))]
    {
        use fmn_platform::process::StdProcessRunner;
        assert!(StdProcessRunner.run(&s).is_err());
    }
}

#[cfg(not(feature = "exact-process"))]
#[test]
fn absent_exact_process_feature_fails_closed_without_a_policy_identity() {
    use fmn_platform::process::{NoProcessRunner, ProcessError, ProcessMechanism};

    let requested = spec(fake_ffmpeg(), &[]);
    let error = NoProcessRunner.run(&requested).unwrap_err();
    assert!(matches!(error, ProcessError::CapabilityAbsent { .. }));
    assert_eq!(
        NoProcessRunner.mechanism(),
        ProcessMechanism::ExactImageUnavailable
    );
    assert_eq!(NoProcessRunner.mechanism().policy_version(), 0);
}

/// An absolute path for a fixture program the scripted runner will never
/// touch on disk. Absoluteness is platform-defined: `/fake/ffmpeg` has no
/// root on Windows, where an absolute path needs a drive prefix, and the
/// runner's ProgramNotAbsolute refusal fires before any scripting.
fn fake_ffmpeg() -> &'static str {
    if cfg!(windows) {
        r"C:\fake\ffmpeg.exe"
    } else {
        "/fake/ffmpeg"
    }
}

#[test]
fn scripted_runner_replays_and_logs() {
    let mut r = ScriptedRunner::new();
    r.script(
        fake_ffmpeg(),
        ProcessOutcome {
            termination: ProcessTermination::Exited(Some(0)),
            stdout: b"frame=  1".to_vec(),
            stderr: Vec::new(),
        },
    );
    let s = spec(fake_ffmpeg(), &["-i", "-", "out.mp4"]);
    let out = r.run(&s).expect("scripted");
    assert!(out.success());
    assert_eq!(out.stdout, b"frame=  1");
    let fake_other = if cfg!(windows) {
        r"C:\fake\other.exe"
    } else {
        "/fake/other"
    };
    assert!(r.run(&spec(fake_other, &[])).is_err());
    let runs = r.runs();
    assert_eq!(runs.len(), 1);
    assert_eq!(runs[0].argv, vec!["-i", "-", "out.mp4"]);
    assert!(runs[0].stdin.is_none());
}

#[test]
fn scripted_stream_preserves_chunk_order_and_enforces_both_input_bounds() {
    let mut runner = ScriptedRunner::new();
    runner.script(
        fake_ffmpeg(),
        ProcessOutcome {
            termination: ProcessTermination::Exited(Some(0)),
            stdout: Vec::new(),
            stderr: Vec::new(),
        },
    );
    let runner = Arc::new(runner);
    let stream_spec = spec(fake_ffmpeg(), &["-i", "-", "out.mp4"]);
    let mut process = runner
        .start(
            &stream_spec,
            ProcessCancellation::new(),
            ProcessStdinLimits::new(4, 6),
        )
        .expect("start scripted stream");
    process.write_stdin(b"ab").expect("first chunk");
    process.write_stdin(b"cdef").expect("second chunk");
    assert!(process.finish().expect("finish").success());
    assert_eq!(
        runner.runs()[0].stdin.as_deref(),
        Some(b"abcdef".as_slice())
    );

    let mut process = runner
        .start(
            &stream_spec,
            ProcessCancellation::new(),
            ProcessStdinLimits::new(3, 10),
        )
        .expect("start chunk-bound stream");
    assert!(matches!(
        process.write_stdin(b"four"),
        Err(fmn_platform::process::ProcessError::StdinChunkLimit {
            attempted: 4,
            max: 3,
            ..
        })
    ));
    process.cancel().expect("cancel chunk-bound stream");

    let mut process = runner
        .start(
            &stream_spec,
            ProcessCancellation::new(),
            ProcessStdinLimits::new(4, 5),
        )
        .expect("start total-bound stream");
    process.write_stdin(b"abc").expect("first bounded chunk");
    assert!(matches!(
        process.write_stdin(b"def"),
        Err(fmn_platform::process::ProcessError::StdinTotalLimit {
            attempted: 6,
            max: 5,
            ..
        })
    ));
    process.cancel().expect("cancel total-bound stream");
}
