//! Lossless storage and placement of music longer than one native sound cue.
//!
//! Short stems keep the original WAV contract. Long stems use consecutive
//! float32 pieces, each below the unchanged native 64 MiB cap. The manifest
//! binds their exact order, sample offsets, lengths and bytes. Splitting adds
//! no fades, padding or resampling: concatenation is the original F32 stream.

use crate::narration::{CueFile, inspect_cue};
use explainer_score::{SAMPLE_RATE, Stereo, wav};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Part {
    pub start: usize,
    pub file: CueFile,
}

fn invalid(message: impl Into<String>) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message.into())
}

pub fn manifest_path(stem: &Path) -> PathBuf {
    stem.with_extension("parts.tsv")
}

/// Load and validate either the legacy whole WAV or the complete parts list.
pub fn read(stem: &Path) -> io::Result<Vec<Part>> {
    let manifest = manifest_path(stem);
    if !manifest.exists() {
        return Ok(vec![Part {
            start: 0,
            file: inspect_cue(stem)?,
        }]);
    }
    let text = std::fs::read_to_string(&manifest)?;
    let mut lines = text.lines();
    if lines.next() != Some("score-parts\t1") {
        return Err(invalid("unknown score-parts header"));
    }
    let mut end = 0usize;
    let mut parts = Vec::new();
    for line in lines {
        let fields: Vec<_> = line.split('\t').collect();
        if fields.len() != 5 {
            return Err(invalid("score part needs start, frames, rate, sha256 and basename"));
        }
        let parse = |s: &str| s.parse::<usize>().map_err(|_| invalid("invalid score part integer"));
        let (start, frames, rate) = (parse(fields[0])?, parse(fields[1])?, parse(fields[2])?);
        let name = fields[4];
        if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
            return Err(invalid("score part must be a local basename"));
        }
        let file = inspect_cue(&stem.with_file_name(name))?;
        if start != end
            || frames == 0
            || frames != file.frames
            || rate != SAMPLE_RATE as usize
            || file.rate != SAMPLE_RATE
            || file.channels != 2
            || fields[3] != file.sha256
        {
            return Err(invalid("score parts differ in timing, format or hash"));
        }
        end = end.checked_add(frames).ok_or_else(|| invalid("score parts overflow"))?;
        parts.push(Part { start, file });
    }
    if parts.len() < 2 {
        return Err(invalid("segmented score needs at least two parts"));
    }
    Ok(parts)
}

/// Preserve the legacy path for short chapters, segment only over-cap stems.
pub fn write(stem: &Path, music: &Stereo) -> io::Result<()> {
    crate::cli::set_aside(&manifest_path(stem))?;
    crate::cli::set_aside(stem)?;
    if wav::encoded_size(music.len() as u64, wav::Format::F32) <= wav::MAX_CUE_BYTES {
        return wav::write(stem, music, wav::Format::F32);
    }
    write_parts(stem, music, 120 * SAMPLE_RATE as usize)
}

fn write_parts(stem: &Path, music: &Stereo, frames_per_part: usize) -> io::Result<()> {
    if frames_per_part == 0 || music.l.len() != music.r.len() || !music.is_finite() {
        return Err(invalid("invalid music buffer or part size"));
    }
    let basename = stem
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or_else(|| invalid("UTF-8 stem name required"))?;
    let mut text = String::from("score-parts\t1\n");
    for (k, start) in (0..music.len()).step_by(frames_per_part).enumerate() {
        let end = (start + frames_per_part).min(music.len());
        let piece = Stereo {
            l: music.l[start..end].to_vec(),
            r: music.r[start..end].to_vec(),
        };
        let path = stem.with_file_name(format!("{basename}.part{k:04}.wav"));
        crate::cli::set_aside(&path)?;
        wav::write(&path, &piece, wav::Format::F32)?;
        let file = inspect_cue(&path)?;
        text.push_str(&format!(
            "{start}\t{}\t{SAMPLE_RATE}\t{}\t{}\n",
            file.frames,
            file.sha256,
            path.file_name().unwrap().to_string_lossy()
        ));
    }
    // Publish the manifest only after every piece has been written and checked.
    let mut output = std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(manifest_path(stem))?;
    output.write_all(text.as_bytes())?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch() -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "score-parts-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        path.join("chapter.music.f32.wav")
    }

    #[test]
    fn pieces_reassemble_the_same_f32_frames_and_detect_tampering() {
        let stem = scratch();
        let music = Stereo {
            l: vec![0.0, 0.1, -0.25, 0.3, -0.0, 0.001, 0.9],
            r: vec![0.2, -0.7, 0.0, 0.4, 0.5, 0.8, -0.9],
        };
        write_parts(&stem, &music, 3).unwrap();
        let parts = read(&stem).unwrap();
        assert_eq!(parts.iter().map(|p| p.start).collect::<Vec<_>>(), vec![0, 3, 6]);
        let mut left = Vec::new();
        let mut right = Vec::new();
        for part in &parts {
            assert!(part.file.bytes <= wav::MAX_CUE_BYTES);
            let decoded = wav::read(&part.file.path).unwrap();
            left.extend(decoded.samples[0].iter().map(|&x| (x as f32).to_bits()));
            right.extend(decoded.samples[1].iter().map(|&x| (x as f32).to_bits()));
        }
        assert_eq!(left, music.l.iter().map(|&x| (x as f32).to_bits()).collect::<Vec<_>>());
        assert_eq!(right, music.r.iter().map(|&x| (x as f32).to_bits()).collect::<Vec<_>>());
        let manifest = manifest_path(&stem);
        let original = std::fs::read_to_string(&manifest).unwrap();
        for changed in [
            original.replacen("3\t3\t", "4\t3\t", 1),
            original.replacen("48000", "24000", 1),
            original.replacen("chapter.music.f32.wav.part0000.wav", "../part.wav", 1),
            original.lines().take(1).collect::<Vec<_>>().join("\n"),
        ] {
            std::fs::write(&manifest, changed).unwrap();
            assert!(read(&stem).is_err());
        }
        std::fs::write(&manifest, original).unwrap();
        let p = &parts[1].file.path;
        let mut bytes = std::fs::read(p).unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 1;
        std::fs::write(p, bytes).unwrap();
        assert!(read(&stem).is_err());
    }

    #[test]
    fn ordinary_stems_keep_the_legacy_file_and_part_size_is_bounded() {
        let stem = scratch();
        write(&stem, &Stereo::silence(5)).unwrap();
        assert!(stem.exists());
        assert!(!manifest_path(&stem).exists());
        assert_eq!(read(&stem).unwrap()[0].file.frames, 5);
        assert!(wav::encoded_size(120 * u64::from(SAMPLE_RATE), wav::Format::F32) <= wav::MAX_CUE_BYTES);
    }
}
