//! One scene per chapter, in storyboard order, with its score intensity
//! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
//! the chapter with the same `NN`.

pub mod draw;
pub mod glyphs;
pub mod same_shadows;
pub mod stops;
pub mod water;

use explainer_kit::Chapter;

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_same_shadows",
            make: |kit| Box::new(same_shadows::SameShadows { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "02_water_through_the_grid",
            make: |kit| Box::new(water::WaterThroughTheGrid { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "03_where_the_water_stops",
            make: |kit| Box::new(stops::WhereTheWaterStops { kit }),
            intensity: 0.5,
        },
    ]
}
