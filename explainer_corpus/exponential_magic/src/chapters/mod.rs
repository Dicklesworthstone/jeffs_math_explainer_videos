//! Chapters 01 to 15 of the development registry. The full film's opening is still the root's open item.
mod calculating_ruler;
mod changing_rates;
pub mod circuit_scene;
pub mod decay_scene;
mod ending_draw;
mod euler_identity;
mod factorials;
mod log_bridge;
mod money_limits;
mod only_solution;
mod product_grid;
mod rate_law;
pub mod thumbnail;
mod tiny_multipliers;
mod turning_rate;
use explainer_kit::Chapter;
pub const TEXT: &[&str] = &[];
pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_rate_law",
            make: |kit| Box::new(rate_law::RateLaw { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_factorials",
            make: |kit| Box::new(factorials::Factorials { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "03_product_grid",
            make: |kit| Box::new(product_grid::ProductGrid { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "04_tiny_multipliers",
            make: |kit| Box::new(tiny_multipliers::TinyMultipliers { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_whole_sum",
            make: |kit| Box::new(tiny_multipliers::WholeSum { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "06_only_solution",
            make: |kit| Box::new(only_solution::OnlySolution { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "07_calculating_ruler",
            make: |kit| Box::new(calculating_ruler::CalculatingRuler { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_decay_law",
            make: |kit| Box::new(decay_scene::DecayLaw { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "09_charging_gap",
            make: |kit| Box::new(circuit_scene::ChargingGap { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "10_changing_rates",
            make: |kit| Box::new(changing_rates::ChangingRates { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "11_money_limits",
            make: |kit| Box::new(money_limits::MoneyLimits { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "12_chance_and_limits",
            make: |kit| Box::new(money_limits::ChanceAndLimits { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "13_turning_rate",
            make: |kit| Box::new(turning_rate::TurningRate { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "14_euler_identity",
            make: |kit| Box::new(euler_identity::EulerIdentity { kit }),
            intensity: 0.55,
        },
        Chapter {
            key: "15_logarithm_bridge",
            make: |kit| Box::new(log_bridge::LogBridge { kit }),
            intensity: 0.4,
        },
    ]
}
