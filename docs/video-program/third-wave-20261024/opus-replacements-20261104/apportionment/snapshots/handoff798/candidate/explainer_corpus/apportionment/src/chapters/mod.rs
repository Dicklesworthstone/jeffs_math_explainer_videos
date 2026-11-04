//! The fifteen chapters of Adding a Seat, in storyboard order, and the
//! glyphs their labels need. Candidate: uncompiled and unrendered.

mod both_promises;
mod cost;
mod dials;
pub mod draw;
mod largest_tails;
mod no_rule;
mod not_every_time;
mod one_more_seat;
mod population;
mod recap;
mod rounding_rules;
mod shares;
mod signposts;
mod sixteenth;
pub mod thumbnail;
mod webster_quota;
mod yardstick;

use explainer_kit::Chapter;

/// Every character a label can contain (labels are built at run time from
/// `crate::model`, so the whole printable set is declared, plus the few
/// symbols the cards use).
pub const TEXT: &[&str] = &[
    "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789",
    " !\"#%&'()*+,-./:;<=>?@[]_",
    "−×→–ö√≈÷",
    "Adding a Seat 15 seats 16 seats 2 → 1 ? + ? + ? = 15 1 left over 2 left over",
    "compare what is left over change the people per seat people per seat",
    "quota promise no take backs population promise Hamilton Jefferson Adams Webster Hill quota rule",
    "city village Gölz, Peters and Procaccia Balinski and Young pause for more time",
];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "01_one_more_seat",
            make: |kit| Box::new(one_more_seat::OneMoreSeat { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "02_shares",
            make: |kit| Box::new(shares::Shares { kit }),
            intensity: 0.35,
        },
        Chapter {
            key: "03_largest_tails",
            make: |kit| Box::new(largest_tails::LargestTails { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "04_sixteenth",
            make: |kit| Box::new(sixteenth::Sixteenth { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "05_dials",
            make: |kit| Box::new(dials::Dials { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "06_not_every_time",
            make: |kit| Box::new(not_every_time::NotEveryTime { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "07_yardstick",
            make: |kit| Box::new(yardstick::Yardstick { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "08_signposts",
            make: |kit| Box::new(signposts::Signposts { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "09_cost",
            make: |kit| Box::new(cost::Cost { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "10_rounding_rules",
            make: |kit| Box::new(rounding_rules::RoundingRules { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "11_webster_quota",
            make: |kit| Box::new(webster_quota::WebsterQuota { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "12_both_promises",
            make: |kit| Box::new(both_promises::BothPromises { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "13_population",
            make: |kit| Box::new(population::Population { kit }),
            intensity: 0.45,
        },
        Chapter {
            key: "14_no_rule",
            make: |kit| Box::new(no_rule::NoRule { kit }),
            intensity: 0.5,
        },
        Chapter {
            key: "15_recap",
            make: |kit| Box::new(recap::Recap { kit }),
            intensity: 0.35,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Each chapter's source, by registry key.
    const SOURCES: [(&str, &str); 15] = [
        ("01_one_more_seat", include_str!("one_more_seat.rs")),
        ("02_shares", include_str!("shares.rs")),
        ("03_largest_tails", include_str!("largest_tails.rs")),
        ("04_sixteenth", include_str!("sixteenth.rs")),
        ("05_dials", include_str!("dials.rs")),
        ("06_not_every_time", include_str!("not_every_time.rs")),
        ("07_yardstick", include_str!("yardstick.rs")),
        ("08_signposts", include_str!("signposts.rs")),
        ("09_cost", include_str!("cost.rs")),
        ("10_rounding_rules", include_str!("rounding_rules.rs")),
        ("11_webster_quota", include_str!("webster_quota.rs")),
        ("12_both_promises", include_str!("both_promises.rs")),
        ("13_population", include_str!("population.rs")),
        ("14_no_rule", include_str!("no_rule.rs")),
        ("15_recap", include_str!("recap.rs")),
    ];

    #[test]
    fn script_and_registry_follow_the_storyboard() {
        let keys: Vec<&str> = registry().iter().map(|c| c.key).collect();
        explainer_kit::storyboard::check_video(crate::STORYBOARD, crate::SCRIPT, &keys).expect("storyboard binding");
    }

    /// Every script line is spoken by its own chapter, exactly once, and in
    /// script order (`say(&kit, stage, "NN_k")` in that chapter's source).
    #[test]
    fn every_line_is_said_once_in_order() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).expect("script parses");
        for (key, source) in SOURCES {
            let prefix = &key[..3];
            let want: Vec<&str> = script
                .lines()
                .iter()
                .map(|(id, _)| id.as_str())
                .filter(|id| id.starts_with(prefix))
                .collect();
            let mut said = Vec::new();
            for piece in source.split("say(&kit, stage, \"").skip(1) {
                said.push(piece.split('"').next().unwrap_or(""));
            }
            assert_eq!(said, want, "{key}");
        }
    }

    /// The words follow the voice rules: no digits, colons, semicolons,
    /// dashes or parentheses.
    #[test]
    fn spoken_words_follow_the_voice_rules() {
        let script = explainer_kit::Script::parse(crate::SCRIPT).expect("script parses");
        for (id, text) in script.lines() {
            assert!(
                !text.chars().any(|c| c.is_ascii_digit() || ":;()–—-".contains(c)),
                "{id}: {text}"
            );
        }
    }
}
