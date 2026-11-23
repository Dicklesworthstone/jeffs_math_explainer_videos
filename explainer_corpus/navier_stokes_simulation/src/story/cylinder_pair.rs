//! Display values emitted from the guarded four-second Re20 pair by prepare.py.

const DATA: &str = include_str!("cylinder-pair01.tsv");
pub(super) const TITLE: &str = "Cylinder, Re 20: one fixed/adaptive pair";
pub(super) const OUTCOME: &str = "Drag agreement target missed";
pub(super) const COLUMNS: [&str; 2] = ["Fixed L10", "Adaptive up to L10"];

/// Rows are [measure, fixed, adaptive], with the drag gap and its limit last.
pub(super) fn rows() -> Result<[[&'static str; 3]; 7], String> {
    let mut lines = DATA.lines();
    if lines.next() != Some("Measure\tFixed L10\tAdaptive up to L10") {
        return Err("cylinder pair has unexpected headings".to_owned());
    }
    let mut rows = [[""; 3]; 7];
    for (index, cells) in rows.iter_mut().enumerate() {
        let row = lines.next().ok_or_else(|| format!("missing cylinder row {index}"))?;
        let mut values = row.split('\t');
        for cell in cells {
            *cell = values
                .next()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| format!("incomplete cylinder row {index}"))?;
        }
        if values.next().is_some() {
            return Err(format!("extra entry in cylinder row {index}"));
        }
    }
    if lines.next().is_some() {
        return Err("extra cylinder measurement".to_owned());
    }
    Ok(rows)
}
