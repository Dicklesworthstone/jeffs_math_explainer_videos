//! Read-only diagnosis of the native harmony candidates; no score or audio edits.
use explainer_score::{FS, Stereo, duck, rng::Rng, score, timeline::ChapterTimeline};
use std::{env, fs, path::PathBuf};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = PathBuf::from(env::args().nth(1).ok_or("timeline directory")?);
    let identity = env::args().nth(2).ok_or("identity name")?;
    let mut paths: Vec<_> = fs::read_dir(dir)?
        .map(|e| e.map(|e| e.path()))
        .collect::<Result<_, _>>()?;
    paths.retain(|p| p.to_string_lossy().ends_with(".timeline.tsv"));
    paths.sort();
    let mut voice = Stereo::default();
    let mut chapters = Vec::new();
    for path in paths {
        let t = ChapterTimeline::read(&path)?;
        let v = t.voice_bus(None)?;
        let start = voice.len();
        voice.l.extend(v.l);
        voice.r.extend(v.r);
        chapters.push((t.chapter, start, voice.len()));
    }
    let fine = duck::regions(&duck::vad(&voice), 0.2);
    drop(voice);
    for (name, a, b) in chapters {
        let (t0, t1) = (a as f64 / FS, b as f64 / FS);
        let local: Vec<_> = fine
            .iter()
            .filter(|r| r.1 > t0 && r.0 < t1)
            .map(|&(s, e)| (s.max(t0) - t0, e.min(t1) - t0))
            .collect();
        let candidates = score::change_candidates(&local, t1 - t0);
        let mut rng = Rng::new(explainer_score::rng::seed_for(&identity, &name, "harmony", 0));
        let changes = score::plan_changes(&candidates, t1 - t0, (6.0, 12.0), &mut rng);
        for (j, &start) in changes.iter().enumerate() {
            let end = changes.get(j + 1).copied().unwrap_or(t1 - t0);
            if end - start > 16.0 {
                println!("LONG\t{name}\t{start:.9}\t{end:.9}\t{:.9}", end - start);
                println!(
                    "CANDIDATES\t{:?}",
                    candidates
                        .iter()
                        .filter(|&&t| t >= start - 4.0 && t <= end + 4.0)
                        .collect::<Vec<_>>()
                );
                println!(
                    "SPEECH\t{:?}",
                    local.iter().filter(|r| r.1 > start && r.0 < end).collect::<Vec<_>>()
                );
            }
        }
    }
    Ok(())
}
