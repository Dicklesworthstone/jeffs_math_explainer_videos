//! The facts harness: every number a video shows or says is computed, never
//! typed, and pinned against a value verified independently (an oracle
//! script, a paper, a reference implementation) before any frame renders.
//!
//! A video builds one [`Facts`] registry at startup: `add` for computed
//! values the oracle recomputes, `pin` for values also checked against a
//! literal from an independent source. [`Facts::self_check`] fails on any
//! mismatch, [`Facts::print`] emits the `key<TAB>value` lines `./x
//! facts-check` compares with the oracle's output ([`oracle_diff`]).
//!
//! Keep the three test patterns in each video's own facts tests; they are
//! what caught real errors before:
//! 1. narration truth: assert what the voice says, on the exact rendered data;
//! 2. brute force against the closed form, on sizes small enough to enumerate;
//! 3. the edges of every always/never sentence (Hoeffding's "D never drops
//!    below minus one half" reached −1.84 with ties).

use std::collections::HashMap;
use std::fmt::Display;

/// n choose k, exactly (0 when k > n).
///
/// # Panics
/// If the result overflows u128 (release builds would wrap silently).
pub fn choose(n: u64, k: u64) -> u128 {
    if k > n {
        return 0;
    }
    let k = k.min(n - k);
    // exact at every step: a product of i + 1 consecutive integers is
    // divisible by (i + 1)!
    (0..k).fold(1u128, |acc, i| {
        acc.checked_mul(u128::from(n - i)).expect("choose overflows u128") / u128::from(i + 1)
    })
}

/// One number of the video.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Fact {
    pub key: String,
    /// The value exactly as printed for the oracle.
    pub value: String,
    /// An independently verified literal, and where it comes from.
    pub pinned: Option<(String, String)>,
}

/// The registry of a video's numbers, in insertion order.
#[derive(Clone, Debug, Default)]
pub struct Facts {
    facts: Vec<Fact>,
    index: HashMap<String, usize>,
}

impl Facts {
    pub fn new() -> Self {
        Self::default()
    }

    fn insert(&mut self, key: &str, value: String, pinned: Option<(String, String)>) -> Result<&mut Self, String> {
        if key.is_empty() || key.chars().any(char::is_whitespace) {
            return Err(format!("fact key {key:?}: keys are non-empty and have no whitespace"));
        }
        if value.contains(['\t', '\n']) {
            return Err(format!("fact {key}: a value cannot hold a tab or a newline"));
        }
        if self.index.contains_key(key) {
            return Err(format!("fact {key} registered twice"));
        }
        self.index.insert(key.to_owned(), self.facts.len());
        self.facts.push(Fact {
            key: key.to_owned(),
            value,
            pinned,
        });
        Ok(self)
    }

    /// Register a computed value (the oracle recomputes it).
    pub fn add(&mut self, key: &str, value: impl Display) -> Result<&mut Self, String> {
        self.insert(key, value.to_string(), None)
    }

    /// Register a computed value and the independently verified literal it
    /// must equal, with the literal's source.
    pub fn pin(&mut self, key: &str, value: impl Display, expected: &str, source: &str) -> Result<&mut Self, String> {
        self.insert(key, value.to_string(), Some((expected.to_owned(), source.to_owned())))
    }

    pub fn get(&self, key: &str) -> Option<&str> {
        self.index.get(key).map(|&i| self.facts[i].value.as_str())
    }

    pub fn len(&self) -> usize {
        self.facts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.facts.is_empty()
    }

    /// Every pinned value against its literal; all mismatches, not just the first.
    pub fn self_check(&self) -> Result<(), String> {
        let bad: Vec<String> = self
            .facts
            .iter()
            .filter_map(|f| {
                let (want, source) = f.pinned.as_ref()?;
                (f.value != *want).then(|| format!("{}: computed {}, pinned {} ({source})", f.key, f.value, want))
            })
            .collect();
        if bad.is_empty() { Ok(()) } else { Err(bad.join("\n")) }
    }

    /// `key<TAB>value` lines, the format `oracle.py` prints.
    pub fn print(&self) -> String {
        self.facts.iter().map(|f| format!("{}\t{}\n", f.key, f.value)).collect()
    }
}

/// Compare our `print()` output with the oracle's: every key on one side only,
/// and every value that differs. Empty when they agree.
pub fn oracle_diff(ours: &str, oracle: &str) -> Vec<String> {
    let parse = |s: &str| -> Vec<(String, String)> {
        s.lines()
            .filter(|l| !l.trim().is_empty() && !l.starts_with('#'))
            .map(|l| match l.split_once('\t') {
                Some((k, v)) => (k.trim().to_owned(), v.trim().to_owned()),
                None => (l.trim().to_owned(), String::new()),
            })
            .collect()
    };
    let (a, b) = (parse(ours), parse(oracle));
    let (ma, mb): (HashMap<_, _>, HashMap<_, _>) = (a.iter().cloned().collect(), b.iter().cloned().collect());
    let mut out = Vec::new();
    for (k, v) in &a {
        match mb.get(k) {
            None => out.push(format!("{k}: only in the video ({v})")),
            Some(w) if w != v => out.push(format!("{k}: video {v}, oracle {w}")),
            _ => {}
        }
    }
    for (k, v) in &b {
        if !ma.contains_key(k) {
            out.push(format!("{k}: only in the oracle ({v})"));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn choose_is_exact_and_matches_enumeration() {
        assert_eq!(choose(10, 4), 210);
        assert_eq!(choose(3, 4), 0);
        assert_eq!(choose(5_000, 4), 26_010_428_123_750);
        for n in 0..=12u64 {
            let subsets = (0u32..(1 << n)).filter(|m| m.count_ones() == 4).count() as u128;
            assert_eq!(choose(n, 4), subsets, "n = {n}");
        }
    }

    #[test]
    #[should_panic(expected = "overflows")]
    fn choose_refuses_to_overflow() {
        let _ = choose(1_000, 500);
    }

    #[test]
    fn the_registry_refuses_duplicates_and_bad_keys() {
        let mut f = Facts::new();
        f.add("quadruples_5000", choose(5_000, 4)).unwrap();
        assert!(f.add("quadruples_5000", 1).unwrap_err().contains("twice"));
        assert!(f.add("has space", 1).is_err());
        assert!(f.add("", 1).is_err());
        assert!(f.add("tabbed", "a\tb").is_err());
        assert_eq!(f.get("quadruples_5000"), Some("26010428123750"));
        assert_eq!(f.print(), "quadruples_5000\t26010428123750\n");
    }

    #[test]
    fn self_check_reports_every_wrong_pin() {
        let mut f = Facts::new();
        f.pin("c_10_4", choose(10, 4), "210", "math.comb").unwrap();
        assert!(f.self_check().is_ok());
        f.pin("wrong_a", 7, "8", "a typo").unwrap();
        f.pin("wrong_b", 1, "2", "another").unwrap();
        let e = f.self_check().unwrap_err();
        assert!(
            e.contains("wrong_a: computed 7, pinned 8 (a typo)") && e.contains("wrong_b"),
            "{e}"
        );
    }

    #[test]
    fn the_oracle_diff_finds_missing_extra_and_different_values() {
        let ours = "a\t1\nb\t2\nc\t3\n";
        let oracle = "# from oracle.py\na\t1\nb\t20\nd\t4\n";
        let d = oracle_diff(ours, oracle);
        assert_eq!(
            d,
            vec![
                "b: video 2, oracle 20".to_string(),
                "c: only in the video (3)".to_string(),
                "d: only in the oracle (4)".to_string()
            ]
        );
        assert!(oracle_diff(ours, ours).is_empty());
    }
}
