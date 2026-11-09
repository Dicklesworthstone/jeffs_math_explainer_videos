//! The corpus scene-sweep ratchet (fm-5wq.14).
//!
//! `scripts/corpus_scene_sweep.py` renders every Scene class of the pinned
//! 3b1b corpus through fmn-python and publishes
//! `docs/ratchet/scene_outcomes.tsv` plus `scene_dashboard.md`. The sweep
//! needs a portal build and the corpus, so CI cannot re-run it; this
//! always-on test guards what is committed instead:
//! - the TSV is well-formed: exact header, one sorted row per scene, the
//!   closed outcome vocabulary, and exception type names only (the corpus is
//!   CC BY-NC-SA, so no scene text may reach the file);
//! - it covers exactly the baseline's denominator;
//! - it reports at least the baseline's `ok_floor`;
//! - the dashboard headline matches the TSV.

use std::collections::BTreeSet;
use std::path::PathBuf;

const OUTCOMES: [&str; 11] = [
    "ok",
    "capability_refusal",
    "portal_unbound",
    "tex_unsupported",
    "missing_dependency",
    "missing_asset",
    "scene_code_error",
    "portal_exception",
    "runtime_error",
    "timeout",
    "crash",
];
const HEADER: &str = "module\tscene\toutcome\texception";

fn repo_file(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(name);
    std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("read {name}: {error}"))
}

#[derive(Debug, PartialEq, Eq)]
struct Baseline {
    denominator: usize,
    ok_floor: usize,
}

fn parse_baseline(text: &str) -> Result<Baseline, String> {
    let (mut denominator, mut ok_floor) = (None, None);
    for line in text
        .lines()
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
    {
        let (key, value) = line
            .split_once('\t')
            .ok_or_else(|| format!("baseline row is not key<TAB>value: {line:?}"))?;
        let value: usize = value
            .parse()
            .map_err(|error| format!("baseline {key}: {error}"))?;
        match key {
            "denominator" if denominator.is_none() => denominator = Some(value),
            "ok_floor" if ok_floor.is_none() => ok_floor = Some(value),
            _ => return Err(format!("unknown or repeated baseline key {key:?}")),
        }
    }
    Ok(Baseline {
        denominator: denominator.ok_or("baseline lacks denominator")?,
        ok_floor: ok_floor.ok_or("baseline lacks ok_floor")?,
    })
}

/// Every violation of the ratchet contract; empty when the TSV holds.
fn violations(tsv: &str, baseline: &Baseline) -> (Vec<String>, usize) {
    let mut problems = Vec::new();
    let mut lines = tsv.lines();
    if lines.next() != Some(HEADER) {
        problems.push(format!("the TSV header must be {HEADER:?}"));
    }
    let mut keys = Vec::new();
    let mut ok = 0;
    for line in lines {
        let fields: Vec<&str> = line.split('\t').collect();
        let [module, scene, outcome, exception] = fields[..] else {
            problems.push(format!("row does not have 4 fields: {line:?}"));
            continue;
        };
        if !OUTCOMES.contains(&outcome) {
            problems.push(format!(
                "outcome outside the closed vocabulary: {outcome:?}"
            ));
        }
        if !exception
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
        {
            problems.push(format!("exception is not a bare type name: {exception:?}"));
        }
        if outcome == "ok" {
            ok += 1;
        }
        keys.push((module.to_owned(), scene.to_owned()));
    }
    if !keys.windows(2).all(|pair| pair[0] < pair[1]) {
        problems.push("rows must be sorted by (module, scene) with no duplicates".to_owned());
    }
    if keys.iter().collect::<BTreeSet<_>>().len() != baseline.denominator
        || keys.len() != baseline.denominator
    {
        problems.push(format!(
            "the sweep covers {} scenes; the baseline denominator is {}",
            keys.len(),
            baseline.denominator
        ));
    }
    if ok < baseline.ok_floor {
        problems.push(format!(
            "ok dropped to {ok}, below the ratchet floor {}; lower ok_floor only with a written reason",
            baseline.ok_floor
        ));
    }
    (problems, ok)
}

fn headline(total: usize, ok: usize) -> String {
    // One decimal place, as corpus_scene_sweep.py writes it.
    #[allow(clippy::cast_precision_loss)]
    let share = 100.0 * ok as f64 / total as f64;
    format!("{total} scene classes; **{ok} ok ({share:.1}%)**.")
}

#[test]
fn the_committed_scene_sweep_holds_the_ratchet() {
    let baseline = parse_baseline(&repo_file("docs/ratchet/scene_baseline.tsv"))
        .unwrap_or_else(|error| panic!("scene_baseline.tsv: {error}"));
    let (problems, ok) = violations(&repo_file("docs/ratchet/scene_outcomes.tsv"), &baseline);
    assert!(
        problems.is_empty(),
        "scene ratchet violations:\n{}",
        problems.join("\n")
    );
    let dashboard = repo_file("docs/ratchet/scene_dashboard.md");
    let expected = headline(baseline.denominator, ok);
    assert!(
        dashboard.lines().any(|line| line.starts_with(&expected)),
        "scene_dashboard.md does not carry the TSV's headline {expected:?}; regenerate it with \
         corpus_scene_sweep.py --report"
    );
}

#[test]
fn planted_violations_are_each_caught() {
    let baseline = Baseline {
        denominator: 2,
        ok_floor: 1,
    };
    let good = format!("{HEADER}\na.py\tA\tok\t\nb.py\tB\tscene_code_error\tTypeError\n");
    assert_eq!(violations(&good, &baseline), (Vec::new(), 1));
    let planted = [
        ("a dropped ok", good.replace("\tok\t", "\ttimeout\t")),
        ("an unknown outcome", good.replace("\tok\t", "\tfine\t")),
        (
            "leaked scene text",
            good.replace("TypeError", "TypeError: 'x^2'"),
        ),
        ("a missing row", format!("{HEADER}\na.py\tA\tok\t\n")),
        (
            "unsorted rows",
            format!("{HEADER}\nb.py\tB\tok\t\na.py\tA\tok\t\n"),
        ),
        (
            "a duplicate row",
            format!("{HEADER}\na.py\tA\tok\t\na.py\tA\tok\t\n"),
        ),
        ("a wrong header", good.replace("exception", "error")),
        (
            "a short row",
            format!("{HEADER}\na.py\tA\tok\nb.py\tB\tok\t\n"),
        ),
    ];
    for (label, tsv) in planted {
        assert!(
            !violations(&tsv, &baseline).0.is_empty(),
            "{label} was not caught"
        );
    }
    assert_eq!(
        headline(2814, 936),
        "2814 scene classes; **936 ok (33.3%)**."
    );
    assert!(parse_baseline("denominator\t2\n").is_err());
    assert!(parse_baseline("denominator\t2\nok_floor\t1\nok_floor\t1\n").is_err());
}
