use explainer_kit::{Chapter, Explainer, Kit};
use fmn::prelude::*;

mod algebra_at_work;
mod arm_torus;
mod body_trees;
mod contact_cycles;
mod drive_actions;
pub mod facts;
mod floating_tree;
mod group_foundations;
mod leg_actions;
pub mod model;
pub mod palette;
mod rover_bracket;
mod score;
mod snake_holonomy;
mod symmetry_modes;
mod wheel_maps;

struct BodyScene {
    kit: Kit,
    draw: fn(&Kit, &mut Stage<'_>) -> fmn::Result<()>,
}

impl SceneConstruct for BodyScene {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        (self.draw)(&self.kit, stage)
    }
}

fn snake_demo(kit: &Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    snake_holonomy::construct_loop(kit, stage)?;
    contact_cycles::construct(kit, stage)
}

fn chapters() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_three_ways_to_multiply",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: group_foundations::opening,
                })
            },
            intensity: 0.46,
        },
        Chapter {
            key: "02_arm_on_a_torus",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: arm_torus::construct,
                })
            },
            intensity: 0.40,
        },
        Chapter {
            key: "03_ordered_rigid_poses",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: group_foundations::rigid_poses,
                })
            },
            intensity: 0.40,
        },
        Chapter {
            key: "04_same_group_different_motion",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: drive_actions::construct,
                })
            },
            intensity: 0.45,
        },
        Chapter {
            key: "05_the_rover_bracket",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: rover_bracket::construct,
                })
            },
            intensity: 0.44,
        },
        Chapter {
            key: "06_floating_base_tree",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: floating_tree::construct,
                })
            },
            intensity: 0.44,
        },
        Chapter {
            key: "07_symmetries_as_permutations",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: leg_actions::construct,
                })
            },
            intensity: 0.45,
        },
        Chapter {
            key: "08_four_modes",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: symmetry_modes::construct,
                })
            },
            intensity: 0.43,
        },
        Chapter {
            key: "09_shape_cycle",
            make: |kit| Box::new(BodyScene { kit, draw: snake_demo }),
            intensity: 0.45,
        },
        Chapter {
            key: "10_wheel_maps",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: wheel_maps::construct,
                })
            },
            intensity: 0.42,
        },
        Chapter {
            key: "11_body_trees",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: body_trees::construct,
                })
            },
            intensity: 0.44,
        },
        Chapter {
            key: "12_algebra_at_work",
            make: |kit| {
                Box::new(BodyScene {
                    kit,
                    draw: algebra_at_work::construct,
                })
            },
            intensity: 0.46,
        },
    ]
}

fn main() {
    let spec = Explainer {
        crate_name: "frankenrobotics_native_representation",
        script: include_str!("../script.tsv"),
        chapters,
        facts: wheel_maps::facts,
        glyph_text: &[],
        glyph_tex: &[],
        glyph_mono: &[],
        identity: score::identity,
        narration_gain_db: -3.0,
        line_gaps: explainer_kit::narration::LineGaps::Constant,
        line_owner: None,
        thumbnail: None,
    };
    std::process::exit(explainer_kit::cli::run(&spec, std::env::args().skip(1)));
}
