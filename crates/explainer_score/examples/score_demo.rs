//! Score the Hoeffding's D explainer with the new engine and compare it with
//! the v5 score (pad + chimes), on the same narration, by the same QA.
//!
//! Input is the cue log of a `HOEFFDING_CUE_LOG=1` render of
//! `explainer_corpus/hoeffding_d` (any resolution; only scene times matter):
//! `cue <id> <seconds> <length>` per narration line, `music <wav> <seconds>
//! <gain_db>` per pad/chime, and one summary line per chapter
//! (`<NN_name> <frames> frames ...`).
//!
//! ```text
//! cargo run --release -p explainer_score --example score_demo -- \
//!     CUE_LOG NARRATION_DIR OUT_DIR
//! ```
//!
//! Writes to OUT_DIR: `timelines/<chapter>.tl`; `stems/<chapter>.music.f32.wav`
//! (the new score, each exactly its scene's length); `qa/{new,old}_<chapter>.json`;
//! `hoeffding_{old,new}_ab.wav` (voice + music for the whole video, both by
//! one fixed gain to the same loudness, the loudest both reach under
//! −1.5 dBTP: a fair A/B); `hoeffding_new_master.wav` (the new mix alone at
//! the delivery spec, −16 LUFS); `ab/<chapter>_{old,new}.wav` for 02_gallery
//! and 07_shuffle, matched the same way; and
//! `summary.tsv` with every metric side by side.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use explainer_score::qa::{self, ScoreFacts};
use explainer_score::score::{self, ChapterPlan, Identity, Section};
use explainer_score::timeline::{ChapterTimeline, LinePlacement};
use explainer_score::units::db_to_gain;
use explainer_score::{Stereo, meter, wav};

/// The v5 narrator placed every line at this gain (hoeffding_d narration.rs).
const V5_VOICE_GAIN_DB: f64 = -9.4;
const FPS: u32 = 60;

struct Chapter {
    timeline: ChapterTimeline,
    /// (wav, seconds, gain dB) of the v5 pad and chimes.
    music: Vec<(PathBuf, f64, f64)>,
}

fn parse_cue_log(text: &str, narration: &Path) -> Result<Vec<Chapter>, String> {
    let mut out = Vec::new();
    let (mut lines, mut music) = (Vec::new(), Vec::new());
    for raw in text.lines() {
        let f: Vec<&str> = raw.split('\t').collect();
        match f.first().copied() {
            Some("cue") if f.len() == 4 => {
                let secs: f64 = f[2].parse().map_err(|_| format!("bad cue time: {raw}"))?;
                lines.push(LinePlacement {
                    id: f[1].to_string(),
                    start_frame: (secs * f64::from(FPS)).round() as u64,
                    gain_db: V5_VOICE_GAIN_DB,
                    wav: narration.join(format!("{}.wav", f[1])),
                });
            }
            Some("music") if f.len() == 4 => {
                let secs: f64 = f[2].parse().map_err(|_| format!("bad music time: {raw}"))?;
                let gain: f64 = f[3].parse().map_err(|_| format!("bad music gain: {raw}"))?;
                music.push((PathBuf::from(f[1]), secs, gain));
            }
            _ => {
                // "<NN_name>   <frames> frames ..." closes a chapter
                let w: Vec<&str> = raw.split_whitespace().collect();
                if w.len() >= 3 && w[2] == "frames" && w[0].as_bytes().first().is_some_and(u8::is_ascii_digit) {
                    let frames: u64 = w[1].parse().map_err(|_| format!("bad frame count: {raw}"))?;
                    out.push(Chapter {
                        timeline: ChapterTimeline {
                            chapter: w[0].to_string(),
                            fps: FPS,
                            frames,
                            lines: std::mem::take(&mut lines),
                            ..Default::default()
                        },
                        music: std::mem::take(&mut music),
                    });
                }
            }
        }
    }
    if out.is_empty() {
        return Err("no chapter summary lines in the cue log".into());
    }
    Ok(out)
}

/// The v5 music of one chapter, placed exactly as franken_manim placed it.
fn old_music(ch: &Chapter) -> Result<Stereo, String> {
    let mut bus = Stereo::silence(ch.timeline.samples());
    for (path, secs, gain) in &ch.music {
        let d = wav::read(path).map_err(|e| format!("{}: {e}", path.display()))?;
        if d.rate != explainer_score::SAMPLE_RATE {
            return Err(format!("{}: {} Hz (expected 48 kHz)", path.display(), d.rate));
        }
        let s = match d.samples.as_slice() {
            [l, r] => Stereo {
                l: l.clone(),
                r: r.clone(),
            },
            [m] => Stereo::from_mono(m),
            _ => return Err(format!("{}: {} channels", path.display(), d.channels)),
        };
        let frame = (secs * f64::from(FPS)).round() as u64;
        bus.mix_in(&s, score::frame_sample(frame, FPS), db_to_gain(*gain));
    }
    Ok(bus)
}

fn concat(parts: &[Stereo]) -> Stereo {
    let mut out = Stereo::default();
    for p in parts {
        out.l.extend(&p.l);
        out.r.extend(&p.r);
    }
    out
}

fn add(a: &Stereo, b: &Stereo) -> Stereo {
    Stereo {
        l: a.l.iter().zip(&b.l).map(|(x, y)| x + y).collect(),
        r: a.r.iter().zip(&b.r).map(|(x, y)| x + y).collect(),
    }
}

/// One fixed gain: −16 LUFS integrated, unless the −1.5 dBTP ceiling binds.
fn master(mix: &Stereo) -> (Stereo, f64, f64, f64) {
    let (i, tp) = (meter::integrated(mix), meter::true_peak_db(mix));
    let g = (-16.0 - i).min(-1.5 - tp);
    let mut m = mix.clone();
    m.scale(db_to_gain(g));
    (m, g, i + g, tp + g)
}

/// Master an A/B pair fairly: each by one fixed gain, both to the same
/// integrated loudness, the loudest both can reach (−16 LUFS unless either
/// one's true peak would pass −1.5 dBTP first). A louder version tends to win
/// a listening test, so the two are never compared at different loudness.
/// Returns the masters and, per version, (gain, LUFS, dBTP).
fn master_pair(a: &Stereo, b: &Stereo) -> (Vec<Stereo>, Vec<(f64, f64, f64)>) {
    let m: Vec<(f64, f64)> = [a, b]
        .iter()
        .map(|x| (meter::integrated(x), meter::true_peak_db(x)))
        .collect();
    let target = m.iter().map(|&(i, tp)| i + (-1.5 - tp)).fold(-16.0_f64, f64::min);
    let mut out = Vec::new();
    let mut stats = Vec::new();
    for (x, &(i, tp)) in [a, b].iter().zip(&m) {
        let g = target - i;
        let mut y = (*x).clone();
        y.scale(db_to_gain(g));
        out.push(y);
        stats.push((g, i + g, tp + g));
    }
    (out, stats)
}

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let [cue_log, narration, out] = args.as_slice() else {
        return Err("usage: score_demo CUE_LOG NARRATION_DIR OUT_DIR".into());
    };
    let (narration, out) = (PathBuf::from(narration), PathBuf::from(out));
    let text = fs::read_to_string(cue_log).map_err(|e| format!("{cue_log}: {e}"))?;
    let chapters = parse_cue_log(&text, &narration)?;
    for d in ["timelines", "stems", "qa", "ab"] {
        fs::create_dir_all(out.join(d)).map_err(|e| e.to_string())?;
    }
    let total_frames: u64 = chapters.iter().map(|c| c.timeline.frames).sum();
    println!(
        "{} chapters, {} narration lines, {} v5 music cues, {total_frames} frames",
        chapters.len(),
        chapters.iter().map(|c| c.timeline.lines.len()).sum::<usize>(),
        chapters.iter().map(|c| c.music.len()).sum::<usize>()
    );
    for c in &chapters {
        fs::write(
            out.join(format!("timelines/{}.tl", c.timeline.chapter)),
            c.timeline.to_text(),
        )
        .map_err(|e| e.to_string())?;
    }

    // the new score: narration at 0 dB (plan §7.2), the music under it
    let last = chapters.len() - 1;
    let sections: Vec<Section> = chapters
        .iter()
        .enumerate()
        .map(|(k, c)| Section {
            timeline: c.timeline.clone(),
            plan: ChapterPlan {
                // a gentle arc: quiet hook, fuller middle, settled outro
                intensity: [0.3, 0.45, 0.5, 0.5, 0.55, 0.6, 0.65, 0.4]
                    .get(k)
                    .copied()
                    .unwrap_or(0.5),
                resolve: k == last,
            },
        })
        .collect();
    let t = Instant::now();
    let new = score::render(&Identity::hoeffding(), &sections, Some(0.0)).map_err(|e| e.to_string())?;
    let render_s = t.elapsed().as_secs_f64();
    println!(
        "new score: {:.1} s of music rendered in {render_s:.1} s; {} chords, {} notes, {} phrases; voice {:.1} LUFS; stem {:.1} dBTP (cap {:+.1} dB)",
        new.music.len() as f64 / 48_000.0,
        new.chords.len(),
        new.events.len(),
        new.phrases.len(),
        new.voice_lufs,
        new.true_peak_db,
        new.cap_db
    );

    // QA, both scores, chapter by chapter
    let mut rows: BTreeMap<String, Vec<(String, String)>> = BTreeMap::new();
    let mut old_voice_parts = Vec::new();
    let mut old_music_parts = Vec::new();
    let (mut new_pass, mut old_pass) = (0, 0);
    for (k, c) in chapters.iter().enumerate() {
        let name = &c.timeline.chapter;
        let len = c.timeline.samples();
        let (nv, nm) = (new.chapter_voice(k), new.chapter_music(k));
        wav::write(&out.join(format!("stems/{name}.music.f32.wav")), &nm, wav::Format::F32)
            .map_err(|e| e.to_string())?;
        let rn = qa::evaluate(name, &nv, &nm, len, Some(&ScoreFacts::chapter(&new, k)));
        let ov = c.timeline.voice_bus(None).map_err(|e| e.to_string())?;
        let om = old_music(c)?;
        let ro = qa::evaluate(name, &ov, &om, len, None);
        fs::write(out.join(format!("qa/new_{name}.json")), rn.to_json()).map_err(|e| e.to_string())?;
        fs::write(out.join(format!("qa/old_{name}.json")), ro.to_json()).map_err(|e| e.to_string())?;
        new_pass += usize::from(rn.passed());
        old_pass += usize::from(ro.passed());
        for m in &rn.metrics {
            let o = ro.get(&m.name);
            let fmt = |v: f64, p: Option<bool>| {
                format!(
                    "{v:.3}{}",
                    match p {
                        Some(true) => "",
                        Some(false) => " FAIL",
                        None => " (info)",
                    }
                )
            };
            rows.entry(m.name.clone()).or_default().push((
                name.clone(),
                format!(
                    "{}\t{}\t{}",
                    m.limit,
                    o.map_or("-".to_string(), |o| fmt(o.value, o.pass)),
                    fmt(m.value, m.pass)
                ),
            ));
        }
        println!(
            "{name:<14} new {}  old {}{}",
            if rn.passed() { "PASS" } else { "FAIL" },
            if ro.passed() { "PASS" } else { "FAIL" },
            if rn.passed() {
                String::new()
            } else {
                format!(
                    "  (new fails: {})",
                    rn.failures()
                        .iter()
                        .map(|m| format!("{} = {:.3}", m.name, m.value))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
        );
        if name == "02_gallery" || name == "07_shuffle" {
            let (pair, _) = master_pair(&add(&ov, &om), &add(&nv, &nm));
            for (tag, mixed) in ["old", "new"].iter().zip(&pair) {
                wav::write(&out.join(format!("ab/{name}_{tag}.wav")), mixed, wav::Format::S24)
                    .map_err(|e| e.to_string())?;
            }
        }
        old_voice_parts.push(ov);
        old_music_parts.push(om);
    }
    println!(
        "chapters passing QA: new {new_pass}/{}, old {old_pass}/{}",
        chapters.len(),
        chapters.len()
    );

    // whole-video masters
    let new_mix = add(&new.voice, &new.music);
    let old_mix = add(&concat(&old_voice_parts), &concat(&old_music_parts));
    // the A/B pair at one loudness, then the new mix alone at the delivery spec
    let (pair, stats) = master_pair(&old_mix, &new_mix);
    for ((tag, m), (g, i, tp)) in ["old", "new"].iter().zip(&pair).zip(stats) {
        wav::write_master(&out.join(format!("hoeffding_{tag}_ab.wav")), m, wav::Format::S24)
            .map_err(|e| e.to_string())?;
        println!(
            "{tag} (A/B): gain {g:+.2} dB -> {i:.2} LUFS, {tp:.2} dBTP, {:.3} s",
            m.len() as f64 / 48_000.0
        );
    }
    drop(pair);
    let (m, g, i, tp) = master(&new_mix);
    wav::write_master(&out.join("hoeffding_new_master.wav"), &m, wav::Format::S24).map_err(|e| e.to_string())?;
    println!("new master at spec: gain {g:+.2} dB -> {i:.2} LUFS, {tp:.2} dBTP");
    let mut tsv = String::from("metric\tchapter\tlimit\told (v5)\tnew\n");
    for (metric, per) in &rows {
        for (ch, cells) in per {
            tsv.push_str(&format!("{metric}\t{ch}\t{cells}\n"));
        }
    }
    fs::write(out.join("summary.tsv"), tsv).map_err(|e| e.to_string())?;
    println!("every metric, old beside new: {}", out.join("summary.tsv").display());
    Ok(())
}
