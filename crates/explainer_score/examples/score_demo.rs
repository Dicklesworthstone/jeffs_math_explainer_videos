//! Score the Hoeffding's D explainer with the new engine and compare it with
//! the v5 score (pad + chimes), on the same narration, by the same QA.
//!
//! Input is the output directory of the explainer's timeline pass, run with
//! the v5 narration so the v5 pad and chimes play:
//! `hoeffding render --timeline-only --narration NARRATION_DIR --out RENDER_DIR`.
//! Each chapter's `<chapter>.timeline.tsv` gives the narration placements and
//! the scene length; its exact `<chapter>.cuelog.tsv` gives every other cue
//! (the v5 pad and chimes) at the sample franken_manim's mixer placed it.
//!
//! ```text
//! cargo run --release -p explainer_score --example score_demo -- RENDER_DIR OUT_DIR
//! ```
//!
//! Writes to OUT_DIR: `stems/<chapter>.music.f32.wav`
//! (the new score, each exactly its scene's length); `qa/{new,old}_<chapter>.json`;
//! `hoeffding_{old,new}_ab.wav` (voice + music for the whole video, both by
//! one fixed gain to the same loudness, the loudest both reach under
//! −1.5 dBTP: a fair A/B); `hoeffding_new_master.wav` (the new mix alone at
//! the delivery spec, −16 LUFS); `ab/<chapter>_{old,new}.wav` for 02_gallery
//! and 07_shuffle, matched the same way; and
//! `summary.tsv` with every metric side by side.

use std::collections::{BTreeMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;

use explainer_score::qa::{self, ScoreFacts};
use explainer_score::score::{self, ChapterPlan, Identity, Section};
use explainer_score::timeline::ChapterTimeline;
use explainer_score::units::db_to_gain;
use explainer_score::{Stereo, meter, wav};

struct Chapter {
    timeline: ChapterTimeline,
    /// (wav, start sample, gain dB) of the v5 pad and chimes.
    music: Vec<(PathBuf, usize, f64)>,
}

/// Every chapter of a timeline pass, in order: its timeline, and from its
/// exact cue log every cue that is not a narration line.
fn read_render_dir(dir: &Path) -> Result<Vec<Chapter>, String> {
    let mut paths: Vec<PathBuf> = fs::read_dir(dir)
        .map_err(|e| format!("{}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.to_string_lossy().ends_with(".timeline.tsv"))
        .collect();
    paths.sort();
    let mut out = Vec::new();
    for p in paths {
        let timeline = ChapterTimeline::read(&p).map_err(|e| format!("{}: {e}", p.display()))?;
        let log_path = dir.join(format!("{}.cuelog.tsv", timeline.chapter));
        let log = fs::read_to_string(&log_path).map_err(|e| format!("{}: {e}", log_path.display()))?;
        let voice: HashSet<&Path> = timeline.lines.iter().map(|l| l.wav.as_path()).collect();
        let mut music = Vec::new();
        for (n, raw) in log.lines().enumerate() {
            if raw.starts_with('#') || raw.trim().is_empty() {
                continue;
            }
            let bad = |m: &str| format!("{}:{}: {m}", log_path.display(), n + 1);
            let f: Vec<&str> = raw.split('\t').collect();
            if f.len() != 13 {
                return Err(bad("want 13 tab-separated fields"));
            }
            let path = PathBuf::from(f[0]);
            if voice.contains(path.as_path()) {
                continue;
            }
            let start: usize = f[12].parse().map_err(|_| bad("bad start sample"))?;
            let gain = match f[8] {
                "-" => 0.0,
                bits => f64::from_bits(
                    u64::from_str_radix(bits.trim_start_matches("0x"), 16).map_err(|_| bad("bad gain bits"))?,
                ),
            };
            music.push((path, start, gain));
        }
        out.push(Chapter { timeline, music });
    }
    if out.is_empty() {
        return Err(format!(
            "{}: no <chapter>.timeline.tsv (run the timeline pass first)",
            dir.display()
        ));
    }
    Ok(out)
}

/// The v5 music of one chapter, placed exactly as franken_manim placed it.
fn old_music(ch: &Chapter) -> Result<Stereo, String> {
    let mut bus = Stereo::silence(ch.timeline.samples());
    for (path, start, gain) in &ch.music {
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
        bus.mix_in(&s, *start, db_to_gain(*gain));
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
    let [render_dir, out] = args.as_slice() else {
        return Err("usage: score_demo RENDER_DIR OUT_DIR".into());
    };
    let out = PathBuf::from(out);
    let chapters = read_render_dir(Path::new(render_dir))?;
    for d in ["stems", "qa", "ab"] {
        fs::create_dir_all(out.join(d)).map_err(|e| e.to_string())?;
    }
    let total_frames: u64 = chapters.iter().map(|c| c.timeline.frames).sum();
    println!(
        "{} chapters, {} narration lines, {} v5 music cues, {total_frames} frames",
        chapters.len(),
        chapters.iter().map(|c| c.timeline.lines.len()).sum::<usize>(),
        chapters.iter().map(|c| c.music.len()).sum::<usize>()
    );

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
