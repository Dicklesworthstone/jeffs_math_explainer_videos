// Included by the crate root that checks/run_against_stub.py writes for each tree.
const SCRIPT: &str = include_str!(concat!(env!("TREE_ROOT"), "/script.tsv"));

fn run(prefix: &str, rate: f64, snapshots: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    use chapters::{building_ruler, folded_decade, slide_rule, undoing_growth};
    use fmn::prelude::*;
    fmn::stub::reset();
    let kit = explainer_kit::Kit::with_script(SCRIPT, rate, prefix);
    let mut stage = Stage::new();
    let name = match prefix {
        "01_" => {
            let mut scene = building_ruler::BuildingRuler { kit: kit.clone() };
            scene.construct(&mut stage)?;
            scene.name().to_owned()
        }
        "02_" => {
            let mut scene = slide_rule::SlideRule { kit: kit.clone() };
            scene.construct(&mut stage)?;
            scene.name().to_owned()
        }
        "03_" => {
            let mut scene = folded_decade::FoldedDecade { kit: kit.clone() };
            scene.construct(&mut stage)?;
            scene.name().to_owned()
        }
        _ => {
            let mut scene = undoing_growth::UndoingGrowth { kit: kit.clone() };
            scene.construct(&mut stage)?;
            scene.name().to_owned()
        }
    };
    println!("== {name} at {rate} words per second");
    println!("id\tstart\tvoice\tpicture_over\tlatest_beat\tin_sentence");
    let mut end: f64 = 0.0;
    for cue in kit.reports() {
        println!(
            "{}\t{:.1}\t{:.1}\t{:+.1}\t{:.1}\t{}",
            cue.id,
            cue.start,
            cue.length,
            cue.picture_end - cue.start - cue.length,
            cue.latest,
            cue.latest_sentence
        );
        end = end.max(cue.picture_end);
    }
    println!("chapter ends at {end:.1} s");
    let (problems, met, rows) = fmn::stub::with(|book| {
        // The last look's pairs last until the chapter ends.
        let last = book.last_lint;
        for pair in std::mem::take(&mut book.meeting) {
            *book.met.entry(pair).or_insert(0.0) += end - last;
        }
        (book.problems.clone(), book.met.clone(), std::mem::take(&mut book.snapshots))
    });
    println!("{} object-ledger problems", problems.len());
    for problem in problems {
        println!("  {problem}");
    }
    let held: Vec<_> = met.iter().filter(|(pair, _)| !pair.starts_with("(in motion)")).collect();
    let moving: Vec<_> = met.iter().filter(|(pair, _)| pair.starts_with("(in motion)")).collect();
    println!("{} estimated overlaps at rest (seconds on screen)", held.len());
    for (pair, seconds) in held {
        println!("  {seconds:5.1}  {pair}");
    }
    println!("{} estimated overlaps seen only while a tracker runs", moving.len());
    for (pair, _) in moving {
        println!("  {pair}");
    }
    for row in rows {
        snapshots.push(format!("{prefix}\t{row}"));
    }
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut arguments = std::env::args().skip(1);
    let rate: f64 = arguments.next().map_or(Ok(2.68), |rate| rate.parse())?;
    let out = arguments.next();
    let mut snapshots = Vec::new();
    for prefix in ["01_", "02_", "03_", "04_"] {
        run(prefix, rate, &mut snapshots)?;
    }
    if let Some(path) = out {
        std::fs::write(path, snapshots.join("\n") + "\n")?;
    }
    Ok(())
}
