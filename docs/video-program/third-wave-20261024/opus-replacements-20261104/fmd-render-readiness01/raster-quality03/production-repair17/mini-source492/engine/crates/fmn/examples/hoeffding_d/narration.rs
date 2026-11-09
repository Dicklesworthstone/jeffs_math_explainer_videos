//! Voice-over: the narration script, and the timing layer that makes the
//! animation wait for the narrator (3Blue1Brown pacing: picture follows
//! voice). Lines are spoken by FrankenTTS (`ftts`, voice "robert") into
//! `narration/<id>.wav`; franken_manim mixes them natively via `add_sound`.

use std::cell::Cell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use fmn::prelude::*;

/// Every spoken line, in order, keyed by the beat that triggers it.
pub const SCRIPT: &[(&str, &str)] = &[
    // 1 — the hook
    (
        "h1",
        "Here are a hundred and fifty measurements of two quantities, X and Y.",
    ),
    ("h2", "Are they related?"),
    (
        "h3",
        "Let's ask the usual suspects. Pearson's correlation, Spearman's rank correlation, and Kendall's tau.",
    ),
    (
        "h4",
        "All three come back at essentially zero. As far as they can tell, there's no relationship here at all.",
    ),
    (
        "h5",
        "But your eye says otherwise. These points lie on a circle. Knowing X narrows Y down to just two possibilities.",
    ),
    (
        "h6",
        "What we need is a measure of dependence, not a measure of slope.",
    ),
    (
        "h7",
        "One of the most elegant is called Hoeffding's D. Wassily Hoeffding introduced it in nineteen forty-eight.",
    ),
    // 2 — the gallery
    (
        "g1",
        "Let's watch four measures side by side, as one cloud of points changes shape.",
    ),
    (
        "g2",
        "Each bar shows how far a measure rises above pure chance. To estimate chance, we shuffle Y three hundred times, and recompute. The dashed line marks that noise ceiling.",
    ),
    (
        "g3",
        "For a straight line, everyone agrees. All four measures are far above the noise.",
    ),
    (
        "g4",
        "Now, bend it into a parabola. The three correlations collapse. The falling left half cancels out the rising right half. But Hoeffding's D barely flinches.",
    ),
    (
        "g5",
        "Next, a ring. The correlations sit right inside the noise.",
    ),
    (
        "g6",
        "But D sits well above it. It has found the dependence.",
    ),
    (
        "g7",
        "Now an X shape, two lines with opposite slopes. Once again, the correlations see nothing. D sees it clearly.",
    ),
    ("g8", "A wave, rising and falling twice. Same story."),
    (
        "g9",
        "And finally, genuinely independent noise. This time, every measure stays under its ceiling, D included. So D isn't just eager to find patterns. It finds them only when they're really there.",
    ),
    (
        "g10",
        "Correlation asks whether X and Y move together. Hoeffding's D asks a deeper question. Could X and Y be independent at all?",
    ),
    // 3 — why quadruples
    (
        "q1",
        "So why can D see shapes that correlations miss? It comes down to how many points you look at, at once.",
    ),
    (
        "q2",
        "Two points define a direction. Pearson's correlation is built from pairs of points, so straight lines are what it sees.",
    ),
    (
        "q3",
        "Three points can reveal a bend. Rank correlations, like Spearman's and Kendall's, capture any curve that keeps heading the same way.",
    ),
    (
        "q4",
        "But to see a turn, where a curve rises and then falls back down, you need at least four points.",
    ),
    (
        "q5",
        "Rings, crosses, and waves all turn. So detecting them means reasoning about groups of four.",
    ),
    (
        "q6",
        "The trouble is, the number of groups of four grows ferociously fast.",
    ),
    (
        "q7",
        "With five thousand points, there are more than twenty-six trillion of them.",
    ),
    (
        "q8",
        "Hoeffding's trick is to never list them at all. By working with ranks, he folds every quadruple into a single count for each point. We'll call that count Q.",
    ),
    // 4 — ranks
    (
        "r1",
        "Let's work through a small example, using the heights and weights of ten people.",
    ),
    ("r2", "Here they are as a scatter plot."),
    (
        "r3",
        "Step one. Forget the raw values, and keep only their order. The shortest person gets rank one. The next shortest gets rank two, and so on. Then we do the same for weight.",
    ),
    (
        "r4",
        "Three people are tied at seventy-eight. They share places eight, nine, and ten, so each of them gets the average. That's nine.",
    ),
    (
        "r5",
        "Plotting rank against rank gives us the same people, in new coordinates.",
    ),
    (
        "r6",
        "Ranks make the method robust. Push one person's weight way up, and their raw point jumps. But their rank stays at seven.",
    ),
    (
        "r7",
        "Only the order matters, so no single extreme value can dominate.",
    ),
    // 5 — counting Q
    (
        "c1",
        "Step two is the heart of the method. For each point, count how many other points sit strictly below it, and to its left.",
    ),
    (
        "c2",
        "That count, plus one, is called Q. A point that ties in one coordinate earns half credit. An exact twin earns a quarter.",
    ),
    (
        "c3",
        "Take the person ranked seventh in both height and weight. Six people sit below and to the left, so Q is seven.",
    ),
    (
        "c4",
        "If height and weight were unrelated, we'd expect only about five. Seeing seven tells us these points cluster together, more than chance allows.",
    ),
    (
        "c5",
        "For the person ranked sixth in height and fourth in weight, three points qualify. So Q is four.",
    ),
    (
        "c6",
        "Now one of our tied people, at rank nine, nine. Seven points lie strictly below and to the left. The two twins add a quarter each, for a total of eight and a half.",
    ),
    (
        "c7",
        "Repeat this for everyone, and we get the full list of Q values.",
    ),
    (
        "c8",
        "Hoeffding's D adds up how far these observed counts stray from what independence would predict.",
    ),
    // 6 — the formula
    (
        "f1",
        "Step three folds everything into three sums. The first one, D one, runs over the Q values. It measures how tightly the points pile up, below and to the left of one another.",
    ),
    (
        "f2",
        "D two uses only the ranks. These are the marginals, describing each variable on its own.",
    ),
    (
        "f3",
        "And D three is the cross term that ties the joint counts to the marginals.",
    ),
    (
        "f4",
        "Hoeffding's formula weighs these three against each other, and normalizes by the sample size.",
    ),
    ("f5", "Now let's plug in our ten people."),
    (
        "f6",
        "The numerator works out to four hundred and fourteen. And D comes out to about zero point four one.",
    ),
    (
        "f7",
        "For comparison, Pearson's correlation here is zero point nine three, and Kendall's tau is zero point eight six.",
    ),
    (
        "f8",
        "Keep in mind that D lives on its own scale. It's about zero under independence, and one for a perfectly monotone relationship. And it never drops below minus one half.",
    ),
    // 7 — the shuffle
    (
        "s1",
        "So what is D really measuring? Here are forty-eight points along a parabola, in rank coordinates.",
    ),
    (
        "s2",
        "The tick marks along the edges are the marginals. They show where the X values fall, and where the Y values fall, each on its own.",
    ),
    (
        "s3",
        "On the right, we count the points in each cell of a four by four grid. If X and Y were independent, every cell would hold about three.",
    ),
    (
        "s4",
        "And this is Hoeffding's D, recomputed live from the dots on every single frame.",
    ),
    (
        "s5",
        "Now watch what happens when we shuffle the Y values among the points.",
    ),
    (
        "s6",
        "Look at the tick marks. They haven't changed at all. Both marginals are exactly the same. Only the pairing is gone. The grid flattens out, and D drops to essentially zero.",
    ),
    ("s7", "Put them back, and D comes right back too."),
    (
        "s8",
        "That gives us a test. Shuffle two thousand times, and record D each time. Chance alone produces this narrow pile near zero.",
    ),
    (
        "s9",
        "Our observed value is far beyond anything shuffling ever produced. So X and Y are dependent. In the end, D measures exactly what shuffling destroys. It's the gap between the real pattern, and independence.",
    ),
    // 8 — the outro
    (
        "o1",
        "So, to recap. Pearson's correlation sees straight lines. Spearman and Kendall see any monotone curve.",
    ),
    (
        "o2",
        "And Hoeffding's D sees dependence of any shape. Rings, crosses, and waves.",
    ),
    (
        "o3",
        "It's symmetric. It's built on ranks, so it shrugs off outliers. Its price is that it reasons about groups of points, rather than pairs. And that's a price well worth paying, whenever the shape of a relationship matters.",
    ),
    (
        "o4",
        "Every frame, every formula, and every number in this video was rendered by franken manim, a pure Rust rebuild of manim. And this narration was spoken by franken T T S.",
    ),
];

/// Silence between consecutive lines (a paragraph pause), seconds.
const BREATH: f64 = 0.6;
/// Narration level in dB (the score sits well below it).
const GAIN_DB: f64 = 0.0;

/// Duration of a PCM WAV from its header.
pub fn wav_seconds(path: &Path) -> std::io::Result<f64> {
    let bytes = std::fs::read(path)?;
    let bad = || {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("{}: not a PCM WAV", path.display()),
        )
    };
    if bytes.len() < 12 || &bytes[0..4] != b"RIFF" || &bytes[8..12] != b"WAVE" {
        return Err(bad());
    }
    let (mut rate, mut channels, mut bits) = (0u32, 0u16, 0u16);
    let mut at = 12;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let len = u32::from_le_bytes(bytes[at + 4..at + 8].try_into().map_err(|_| bad())?) as usize;
        let body = at + 8;
        if id == b"fmt " && body + 16 <= bytes.len() {
            channels = u16::from_le_bytes([bytes[body + 2], bytes[body + 3]]);
            rate = u32::from_le_bytes(bytes[body + 4..body + 8].try_into().map_err(|_| bad())?);
            bits = u16::from_le_bytes([bytes[body + 14], bytes[body + 15]]);
        } else if id == b"data" {
            if rate == 0 || channels == 0 || bits == 0 {
                return Err(bad());
            }
            let frame = f64::from(channels) * f64::from(bits) / 8.0;
            let available = len.min(bytes.len() - body);
            return Ok(available as f64 / frame / f64::from(rate));
        }
        at = body + len + (len & 1);
    }
    Err(bad())
}

/// The narrator: knows every line's file and length, and when the current
/// line ends. Shared by clones of the `Kit`.
pub struct Narrator {
    dir: PathBuf,
    lengths: HashMap<&'static str, f64>,
    busy_until: Cell<f64>,
}

impl Narrator {
    /// Load every scripted line's duration; a missing file is an error so a
    /// render never silently drops a line.
    pub fn load(dir: &Path) -> std::io::Result<Rc<Self>> {
        let mut lengths = HashMap::new();
        for &(id, _) in SCRIPT {
            lengths.insert(id, wav_seconds(&dir.join(format!("{id}.wav")))?);
        }
        Ok(Rc::new(Self {
            dir: dir.to_path_buf(),
            lengths,
            busy_until: Cell::new(0.0),
        }))
    }

    /// Every chapter is its own scene whose clock starts at zero.
    pub fn reset(&self) {
        self.busy_until.set(0.0);
    }

    pub fn total_seconds(&self) -> f64 {
        self.lengths.values().sum()
    }

    /// Wait until the current line (plus a breath) has finished.
    pub fn hold(&self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let now = stage.scene().time().to_f64();
        let until = self.busy_until.get() + BREATH;
        if until - now > 1e-3 {
            stage.wait(until - now)?;
        }
        Ok(())
    }

    /// Speak line `id` now (after the previous one has finished).
    pub fn say(&self, stage: &mut Stage<'_>, id: &str) -> fmn::Result<()> {
        let (&key, &len) =
            self.lengths
                .get_key_value(id)
                .ok_or(fmn::Error::Scene(SceneError::InvalidConfig(
                    "a narration cue names a line that is not in the script",
                )))?;
        self.hold(stage)?;
        let now = stage.scene().time().to_f64();
        stage.scene_mut().add_sound(
            self.dir.join(format!("{key}.wav")),
            0.0,
            Some(GAIN_DB),
            None,
        )?;
        if std::env::var_os("HOEFFDING_CUE_LOG").is_some() {
            eprintln!("cue\t{key}\t{now:.3}\t{len:.3}");
        }
        self.busy_until.set(now + len);
        Ok(())
    }
}
