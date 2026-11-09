//! Native decoder/mixer oracle for lossless music cue segmentation.
use explainer_kit::{
    master::{self, LoggedCue},
    narration::inspect_cue,
    score_stem,
};
use explainer_score::{Stereo, wav};

#[test]
fn whole_and_split_music_mix_identically_at_fractional_frame_boundaries() {
    let dir = std::env::temp_dir().join(format!(
        "native-score-parts-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let music = Stereo {
        l: (0..144_000).map(|i| 0.2 * (i as f64 * 0.037).sin()).collect(),
        r: (0..144_000).map(|i| 0.1 * (i as f64 * 0.051).cos()).collect(),
    };
    let whole = dir.join("whole.wav");
    wav::write(&whole, &music, wav::Format::F32).unwrap();
    let stem = dir.join("chapter.music.f32.wav");
    let mut text = String::from("score-parts\t1\n");
    for (k, (start, end)) in [(0, 60_001), (60_001, 144_000)].into_iter().enumerate() {
        let part = dir.join(format!("part{k}.wav"));
        wav::write(
            &part,
            &Stereo {
                l: music.l[start..end].to_vec(),
                r: music.r[start..end].to_vec(),
            },
            wav::Format::F32,
        )
        .unwrap();
        let file = inspect_cue(&part).unwrap();
        text.push_str(&format!(
            "{start}\t{}\t48000\t{}\tpart{k}.wav\n",
            file.frames, file.sha256
        ));
    }
    std::fs::write(score_stem::manifest_path(&stem), text).unwrap();
    for fps in [15, 60] {
        let frames = 3 * fps;
        let one = vec![LoggedCue {
            path: whole.clone(),
            sha256: inspect_cue(&whole).unwrap().sha256,
            frame: frames as i64,
            fps,
            offset: -3.0,
            gain_db: Some(0.0),
        }];
        let mut pieces: Vec<_> = score_stem::read(&stem)
            .unwrap()
            .into_iter()
            .map(|p| LoggedCue {
                path: p.file.path,
                sha256: p.file.sha256,
                frame: frames as i64,
                fps,
                offset: p.start as f64 / 48_000.0 - 3.0,
                gain_db: Some(0.0),
            })
            .collect();
        let expected = master::replay(&one, frames as u64, fps).unwrap().audio.samples;
        let actual = master::replay(&pieces, frames as u64, fps).unwrap().audio.samples;
        assert_eq!(actual, expected, "native F32 mix differs at {fps} fps");
        pieces[1].offset += 1.0 / 48_000.0;
        assert_ne!(
            master::replay(&pieces, frames as u64, fps).unwrap().audio.samples,
            expected,
            "one-sample shift must be detected"
        );
    }
}
