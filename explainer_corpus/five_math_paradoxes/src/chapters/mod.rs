pub mod thumbnail;
mod bookends;
mod birthday;
mod monty;
mod simpson;
mod zeno;
mod hotel;

use explainer_kit::Chapter;

pub const TEXT: &[&str] = &[
    "Five paradoxes, made clear What does the picture leave out? Host rules Matching pairs Weighted averages Infinite stages Matching rooms One useful question can change the picture",
    "Switch doors? A leads in each group B leads overall How can both be true? Infinitely many stages A finite catch time How can that add up? Five surprises, five explanations",
    "Should you switch?",
    "You choose the first door",
    "Example: host reveals a goat",
    "GOAT",
    "Car behind",
    "Host opens",
    "Switch to",
    "Result",
    "WIN",
    "LOSE",
    "The host knows, reveals a goat, and always offers a switch.",
    "0123456789 /",
    "When does a shared birthday become more likely than not?",
    "people: any matching pair",
    "Independent, uniform birthdays; days",
    "no match:",
    "at least one match:",
    "pairs; their match events are not mutually independent",
    "Inspection pass rates Machine A Machine B Easy parts Hard parts All parts Which machine leads overall? A: 90 of 100 hard B: 90 of 100 easy Different group weights; no causal conclusion by itself",
    "Can Achilles catch the tortoise? head start: metres Achilles: m/s tortoise: Tortoise Two lanes show position; time accelerated Same position at the catch Each stage takes a tenth as long sum = ≈ relative speed:",
    "Every room has a guest A new guest arrives. Can everyone have a room? room guest … Give each original guest the next room A window on the assignment, not a physical schedule new → in general: guest n → room n + 1 new guest A mathematical assignment, not a physical moving plan",
];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter { key: "00_opening", make: |kit| Box::new(bookends::Opening { kit }), intensity: 0.4 },
        Chapter {
            key: "01_monty_hall",
            make: |kit| Box::new(monty::MontyHall { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "02_birthday",
            make: |kit| Box::new(birthday::Birthday { kit }),
            intensity: 0.4,
        },
        Chapter { key: "03_simpson", make: |kit| Box::new(simpson::Simpson { kit }), intensity: 0.4 },
        Chapter { key: "04_zeno", make: |kit| Box::new(zeno::Zeno { kit }), intensity: 0.4 },
        Chapter { key: "05_hilbert_hotel", make: |kit| Box::new(hotel::Hotel { kit }), intensity: 0.4 },
        Chapter { key: "06_closing", make: |kit| Box::new(bookends::Closing { kit }), intensity: 0.3 },
    ]
}
