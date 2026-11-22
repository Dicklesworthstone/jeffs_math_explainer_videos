//! Native chapters with the shared narration and render interface.

mod benchmark01;
mod boundary17_01;
mod car18_01;
mod chapter01;
mod chapter02;
mod chapter03;
mod chapter04;
mod chapter05;
mod chapter06;
mod chapter07;
mod chapter08;
mod chapter09;
mod chapter10;
mod chapter11;
mod chapter12;
mod chapter13;
mod chapter14;
mod chapter15;
mod chapter16;
mod chapter20;
mod closing_wake;
mod lattice19_01;
mod ledger13;
mod rotating_wind01;
mod shapes18;
mod shapes19;
mod score;
mod shear_display;
mod staircase01;
mod tail17_01;
mod timestep;

use std::path::Path;
use std::sync::OnceLock;

use explainer_kit::kit::CameraShot;
use explainer_kit::{Chapter, Explainer, Facts, Kit, Palette};
use fmn::prelude::{Line, Mob, SceneConstruct, Stage, Style, Vec3};

use crate::field::{Frame, TunnelGeometry};
use crate::transport::{Pulse, Rule};
use crate::wake_ledger::Ledger as WakeLedger;

const FULL_SCRIPT: &str = include_str!("../script.tsv");
const OPENING: &str = "01_a_tunnel_inside_a_computer";
const WIND: &str = "02_what_a_flow_picture_shows";
const PARCEL: &str = "03_follow_a_parcel";
const FORCES: &str = "04_what_pushes_a_parcel";
const CONTINUITY: &str = "05_nothing_piles_up";
const LAW: &str = "06_the_law_written_down";
const LEDGER: &str = "07_a_ledger_of_cells";
const STEP: &str = "08_one_step_forward_in_time";
const PRESSURE: &str = "09_pressure_a_puzzle_for_the_whole_box";
const MULTIGRID: &str = "10_small_fixes_and_coarse_grids";
const DIFFUSION: &str = "11_a_stable_lie";
const TIMESTEP: &str = "12_time_is_also_a_mesh";
const ERRORS: &str = "13_which_mistake_are_we_measuring";
const ADAPTIVE: &str = "14_spend_cells_where_they_matter";
const SEAM: &str = "15_the_seam_between_coarse_and_fine";
const SAVING: &str = "16_was_the_saving_real";
const BOUNDARY: &str = "17_are_we_solving_the_right_problem";
const CAR: &str = "18_why_a_car_is_harder";
const LATTICE: &str = "19_two_numerical_languages";
const WAKE: &str = "20_back_to_the_wake";
static RETAINED_STARTUP: OnceLock<Vec<Frame>> = OnceLock::new();
static RETAINED_OPENING_WAKE: OnceLock<chapter01::Wake> = OnceLock::new();
static RETAINED_WAKE_LEDGER: OnceLock<WakeLedger> = OnceLock::new();
const CHAPTER_KEYS: [&str; 20] = [
    OPENING,
    WIND,
    PARCEL,
    FORCES,
    CONTINUITY,
    LAW,
    LEDGER,
    STEP,
    PRESSURE,
    MULTIGRID,
    DIFFUSION,
    TIMESTEP,
    ERRORS,
    ADAPTIVE,
    SEAM,
    SAVING,
    BOUNDARY,
    CAR,
    LATTICE,
    WAKE,
];

struct ParcelChapter {
    kit: Kit,
}

struct ParcelGap(TunnelGeometry);

fn retained_frame() -> Option<&'static Frame> {
    RETAINED_STARTUP.get().and_then(|frames| frames.last())
}

fn retained_geometry() -> Option<TunnelGeometry> {
    retained_frame().and_then(Frame::tunnel_geometry)
}

impl chapter03::TunnelGap for ParcelGap {
    fn gap_beside_cylinder(&mut self, kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let geometry = self.0;
        let width = 5.6;
        let center = [3.7, -2.55];
        let mut drawing = chapter09::tunnel(stage, geometry, width, center)?;
        let scale = width / geometry.length;
        let [cx, cy, radius] = geometry.cylinder;
        let x = center[0] + scale * (cx - geometry.length / 2.0);
        let at = |y| -> Vec3 { [x, center[1] + scale * (y - geometry.height / 2.0), 0.0] };
        for (bottom, top) in [(0.0, cy - radius), (cy + radius, geometry.height)] {
            drawing.push(
                stage.arena_mut().add(
                    Line::new(at(bottom), at(top))
                        .style(Style::default().stroke(Palette::ink(), 2.4, 1.0))
                        .build()?,
                ),
            );
        }
        chapter03::fade_in_all(stage, &drawing, 0.35)?;
        kit.hold(stage)?;
        chapter03::fade_out_all(stage, &drawing, 0.35)
    }
}

impl SceneConstruct for ParcelChapter {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let geometry = retained_geometry()
            .ok_or_else(|| crate::error("Chapter 03 requires --tunnel and the retained solver geometry".into()))?;
        chapter03::chapter03(&self.kit, stage, &mut ParcelGap(geometry), true)?;
        self.kit.finish(stage)
    }
}

struct ContinuityChapter {
    kit: Kit,
}

impl SceneConstruct for ContinuityChapter {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let geometry = retained_geometry()
            .ok_or_else(|| crate::error("Chapter 05 requires --tunnel and the retained solver geometry".into()))?;
        let shape = chapter05::TunnelShape {
            length: geometry.length,
            height: geometry.height,
            cylinder: geometry.cylinder,
        };
        chapter05::chapter05(&self.kit, stage, shape, &mut chapter05::NoThumbnail)?;
        self.kit.finish(stage)
    }
}

struct LawChapter {
    kit: Kit,
}

struct LawThumbnail {
    tunnel: TunnelGeometry,
    layers: Vec<Mob>,
}

impl chapter05::TunnelHandoff for LawThumbnail {
    fn return_thumbnail(&mut self, _kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        self.layers = chapter09::tunnel(stage, self.tunnel, 3.0, [5.4, 3.25])?;
        chapter03::fade_in_all(stage, &self.layers, 0.35)
    }

    fn leave_thumbnail(&mut self, _kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
        chapter03::fade_out_all(stage, &self.layers, 0.35)?;
        self.layers.clear();
        Ok(())
    }
}

impl SceneConstruct for LawChapter {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let tunnel = retained_geometry()
            .ok_or_else(|| crate::error("Chapter 06 requires --tunnel and the retained solver geometry".into()))?;
        let shape = chapter05::TunnelShape {
            length: tunnel.length,
            height: tunnel.height,
            cylinder: tunnel.cylinder,
        };
        let mut thumbnail = LawThumbnail {
            tunnel,
            layers: Vec::new(),
        };
        chapter06::chapter06(&self.kit, stage, shape, &mut thumbnail)?;
        self.kit.finish(stage)
    }
}

fn chapters() -> Vec<Chapter> {
    CHAPTER_KEYS
        .into_iter()
        .map(|key| match key {
            OPENING => Chapter {
                key,
                make: |kit| Box::new(chapter01::Chapter01 {
                    kit,
                    startup: RETAINED_STARTUP.get().map(Vec::as_slice),
                    wake: RETAINED_OPENING_WAKE.get(),
                }),
                intensity: 0.4,
            },
            WIND => Chapter {
                key,
                make: |kit| Box::new(chapter02::Chapter02 {
                    kit,
                    startup: RETAINED_STARTUP.get().map(Vec::as_slice),
                }),
                intensity: 0.45,
            },
            PARCEL => Chapter {
                key,
                make: |kit| Box::new(ParcelChapter { kit }),
                intensity: 0.4,
            },
            FORCES => Chapter {
                key,
                make: |kit| {
                    kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                    Box::new(chapter04::ParcelForcesScene { kit })
                },
                intensity: 0.45,
            },
            CONTINUITY => Chapter {
                key,
                make: |kit| {
                    kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                    Box::new(ContinuityChapter { kit })
                },
                intensity: 0.4,
            },
            LAW => Chapter {
                key,
                make: |kit| {
                    kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                    Box::new(LawChapter { kit })
                },
                intensity: 0.45,
            },
            LEDGER => Chapter {
                key,
                make: |kit| Box::new(chapter07::Chapter07 { kit }),
                intensity: 0.4,
            },
            STEP => Chapter {
                key,
                make: |kit| Box::new(chapter08::Chapter08 { kit }),
                intensity: 0.4,
            },
            PRESSURE => Chapter {
                key,
                make: |kit| {
                    Box::new(chapter09::Chapter09 {
                        kit,
                        tunnel: retained_geometry(),
                    })
                },
                intensity: 0.4,
            },
            MULTIGRID => Chapter {
                key,
                make: |kit| {
                    Box::new(chapter10::Chapter10 {
                        kit,
                        tunnel: retained_geometry(),
                    })
                },
                intensity: 0.4,
            },
            DIFFUSION => Chapter {
                key,
                make: |kit| Box::new(chapter11::Chapter11 { kit }),
                intensity: 0.45,
            },
            TIMESTEP => Chapter {
                key,
                make: |kit| Box::new(chapter12::Chapter12 { kit }),
                intensity: 0.45,
            },
            ERRORS => Chapter {
                key,
                make: |kit| Box::new(chapter13::Chapter13 { kit }),
                intensity: 0.4,
            },
            ADAPTIVE => Chapter {
                key,
                make: |kit| {
                    Box::new(chapter14::Chapter14 {
                        kit,
                        tunnel: retained_geometry().map(|tunnel| chapter14::TunnelShape {
                            length: tunnel.length,
                            height: tunnel.height,
                            cylinder: tunnel.cylinder,
                        }),
                    })
                },
                intensity: 0.45,
            },
            SEAM => Chapter {
                key,
                make: |kit| Box::new(chapter15::Chapter15 { kit }),
                intensity: 0.4,
            },
            SAVING => Chapter {
                key,
                make: |kit| {
                    kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                    Box::new(chapter16::Chapter16 { kit })
                },
                intensity: 0.4,
            },
            BOUNDARY => Chapter {
                key,
                make: |kit| Box::new(tail17_01::TailScene { kit }),
                intensity: 0.4,
            },
            CAR => Chapter {
                key,
                make: |kit| Box::new(car18_01::Chapter18 { kit }),
                intensity: 0.4,
            },
            LATTICE => Chapter {
                key,
                make: |kit| Box::new(lattice19_01::Chapter19 { kit }),
                intensity: 0.45,
            },
            WAKE => Chapter {
                key,
                make: |kit| {
                    if closing_wake::retained().is_some() {
                        kit.request_camera(CameraShot { theta: 0.0, phi: 0.0 });
                    }
                    let ledger = RETAINED_WAKE_LEDGER
                        .get()
                        .cloned()
                        .unwrap_or_else(|| WakeLedger::pending("No qualified measurements supplied"));
                    Box::new(chapter20::Chapter20 {
                        kit,
                        tunnel: retained_geometry().map(|tunnel| chapter20::TunnelShape {
                            length: tunnel.length,
                            height: tunnel.height,
                            cylinder: tunnel.cylinder,
                        }),
                        ledger,
                    })
                },
                intensity: 0.45,
            },
            _ => unreachable!("chapter key is missing its scene: {key}"),
        })
        .collect()
}

fn facts() -> Result<Facts, String> {
    let mut facts = chapter07::facts()?;
    chapter01::register(&mut facts)?;
    chapter08::register(&mut facts)?;
    chapter09::register(&mut facts)?;
    facts.add("ns.unknowns.2d", chapter06::unknowns(2).to_string())?;
    facts.add("ns.equations.2d", chapter06::equations(2).to_string())?;
    chapter10::register(&mut facts)?;
    chapter14::register(&mut facts)?;
    chapter15::register(&mut facts)?;
    chapter16::register(&mut facts)?;
    chapter20::register(&mut facts)?;
    for (key, rule) in [
        ("blend", Rule::Blend),
        ("curve", Rule::Curve),
        ("clamp", Rule::ClampedCurve),
        ("faces", Rule::LimitedFlux),
    ] {
        let mut pulse = Pulse::new(0);
        for _ in 0..pulse.step_count() {
            pulse.advance(rule, false);
        }
        let result = pulse.measure();
        for (name, number) in [
            ("peak", result.peak),
            ("min", result.minimum),
            ("total", result.total),
            ("error", result.error),
        ] {
            facts.add(&format!("{key}.end.{name}"), format!("{number:.2}"))?;
        }
    }
    Ok(facts)
}

/// Keep the text labels' powers but lay out their exponents with native TeX.
fn exponent_tex(label: &str) -> Option<String> {
    const SUPERSCRIPTS: &str = "⁰¹²³⁴⁵⁶⁷⁸⁹";
    let (lead, rest) = label.split_once('⁻')?;
    let mut tex = format!(r"\text{{{lead}}}");
    for part in rest.split('⁻') {
        let end = part.find(|c: char| !SUPERSCRIPTS.contains(c)).unwrap_or(part.len());
        let (power, tail) = part.split_at(end);
        if power.is_empty() {
            return None;
        }
        let digits: String = power
            .chars()
            .filter_map(|digit| SUPERSCRIPTS.chars().position(|c| c == digit))
            .map(|digit| char::from(b'0' + digit as u8))
            .collect();
        tex.push_str(&format!("^{{-{digits}}}"));
        if !tail.is_empty() {
            tex.push_str(&format!(r"\text{{{tail}}}"));
        }
    }
    Some(tex)
}

/// Bind saved startup frames and optional qualified wake evidence.
fn bind_story_inputs(args: Vec<String>) -> Result<Vec<String>, String> {
    let mut rest = Vec::with_capacity(args.len());
    let mut items = args.into_iter();
    while let Some(arg) = items.next() {
        if arg == "--wake-ledger" {
            let path = items.next().ok_or("--wake-ledger needs a validated ledger file")?;
            let content = std::fs::read_to_string(&path).map_err(|error| format!("{path}: {error}"))?;
            let ledger = WakeLedger::parse(&content).map_err(|error| format!("{path}: {error}"))?;
            if RETAINED_WAKE_LEDGER.set(ledger).is_err() {
                return Err("--wake-ledger is given once".into());
            }
            continue;
        }
        if arg == "--opening-wake" {
            let path = items.next().ok_or("--opening-wake needs an opening-wake01 file")?;
            let wake = chapter01::Wake::load(Path::new(&path))?;
            RETAINED_OPENING_WAKE.set(wake).map_err(|_| "--opening-wake is given once")?;
            continue;
        }
        if arg != "--tunnel" {
            rest.push(arg);
            continue;
        }
        let directory = items.next().ok_or("--tunnel needs a saved snapshot directory")?;
        let frames = crate::load_frames(Path::new(&directory))
            .map_err(|error| format!("{directory}: {error}"))?;
        if frames.last().and_then(Frame::tunnel_geometry).is_none() {
            return Err(format!("{directory}: the last saved frame is missing tunnel geometry"));
        }
        if RETAINED_STARTUP.set(frames).is_err() {
            return Err("--tunnel is given once".into());
        }
    }
    Ok(rest)
}

pub fn run(args: Vec<String>) -> i32 {
    let mut args = match bind_story_inputs(args)
        .and_then(shear_display::bind)
        .and_then(closing_wake::bind)
    {
        Ok(args) => args,
        Err(error) => {
            eprintln!("{error}");
            return 2;
        }
    };
    match args.first().map(String::as_str) {
        Some("render" | "captions") => {
            if let Ok(parsed) = explainer_kit::cli::parse_args(args.iter().cloned())
                && !parsed.help
                && (parsed.targets.is_empty()
                    || parsed.targets.iter().any(|key| key != "all" && !CHAPTER_KEYS.contains(&key.as_str())))
            {
                eprintln!("Select a wired chapter. {}", CHAPTER_KEYS.join(", "));
                return 2;
            }
        }
        Some("script" | "facts" | "glyphs" | "score" | "master") => {}
        Some("chapters") => {
            println!("{}", CHAPTER_KEYS.join("\n"));
            return 0;
        }
        None | Some("help" | "--help" | "-h") => {
            println!("Navier–Stokes native chapters: --story render|score|master|captions|script|chapters|facts|glyphs");
            return 0;
        }
        Some(other) => {
            eprintln!("Story command {other:?} is unavailable until the full film is assembled");
            return 2;
        }
    }
    if matches!(args.first().map(String::as_str), Some("render" | "glyphs"))
        && !args.iter().any(|arg| arg == "--frame-height")
    {
        args.extend(["--frame-height".to_owned(), crate::FRAME_HEIGHT.to_string()]);
    }
    let film = Explainer {
        crate_name: "navier_stokes_simulation",
        script: FULL_SCRIPT,
        chapters,
        facts,
        glyph_text: &[],
        glyph_tex: &[],
        glyph_mono: &[],
        identity: score::identity,
        narration_gain_db: -3.0,
        line_gaps: explainer_kit::narration::LineGaps::Constant,
        line_owner: None,
        thumbnail: None,
    };
    explainer_kit::cli::run_with_contract(&film, args, Path::new(env!("CARGO_MANIFEST_DIR")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_exponents_keep_the_math_and_the_surrounding_text() {
        assert_eq!(
            exponent_tex("error 2.1 × 10⁻⁷ → 10⁻¹²").as_deref(),
            Some(r"\text{error 2.1 × 10}^{-7}\text{ → 10}^{-12}")
        );
        assert_eq!(exponent_tex("eight fluid layers"), None);
    }

    #[test]
    fn canonical_speech_has_one_chapter_per_cue() {
        let script = explainer_kit::Script::parse(FULL_SCRIPT).unwrap();
        explainer_kit::cli::check_script_owners(&script, &CHAPTER_KEYS, None).unwrap();
        assert_eq!(script.lines().len(), 278);
    }

    #[test]
    fn parcel_handoffs_use_the_canonical_spoken_text() {
        for (prefix, handoff) in [
            ("03_", chapter03::SCRIPT03),
            ("04_", chapter04::SCRIPT04),
            ("05_", chapter05::SCRIPT05),
            ("06_", chapter06::SCRIPT06),
        ] {
            let canonical = FULL_SCRIPT.lines().filter(|line| line.starts_with(prefix));
            assert!(canonical.eq(handoff.lines()), "Changed dialogue in {prefix}");
        }
    }
}
