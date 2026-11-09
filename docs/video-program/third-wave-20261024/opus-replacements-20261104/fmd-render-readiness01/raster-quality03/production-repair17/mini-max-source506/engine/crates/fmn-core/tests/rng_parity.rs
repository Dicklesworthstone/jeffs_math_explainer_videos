//! NumPy seed-parity for the one RNG (fm-m1u): SeedSequence words, raw
//! draws, doubles, named substreams, and keyed per-frame forks must be
//! bit-identical to `fixtures/rng_vectors.txt`, generated from NumPy
//! itself by `scripts/gen_rng_vectors.py`.

use fmn_core::rng::{Pcg64Dxsm, RngRoot, SeedSequence};

fn rows(kind: &str) -> Vec<Vec<String>> {
    let path = format!("{}/fixtures/rng_vectors.txt", env!("CARGO_MANIFEST_DIR"));
    let content = std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("reading {path}: {e}"));
    content
        .lines()
        .filter(|l| l.starts_with(kind))
        .map(|l| l.split('\t').skip(1).map(str::to_string).collect())
        .collect()
}

#[test]
fn seed_sequence_words_match_numpy() {
    let cases = rows("seedseq");
    assert!(!cases.is_empty());
    for row in cases {
        let seed: u64 = row[0].parse().unwrap();
        let expected: Vec<u32> = row[1..]
            .iter()
            .map(|w| u32::from_str_radix(w, 16).unwrap())
            .collect();
        let words = SeedSequence::from_seed(seed).generate_state(expected.len());
        assert_eq!(words, expected, "seed {seed}");
    }
}

#[test]
fn raw_draws_match_numpy() {
    for row in rows("draws\t") {
        let seed: u64 = row[0].parse().unwrap();
        let mut generator = Pcg64Dxsm::from_seed(seed);
        for (i, hex) in row[1..].iter().enumerate() {
            let expected = u64::from_str_radix(hex, 16).unwrap();
            assert_eq!(generator.next_u64(), expected, "seed {seed} draw {i}");
        }
    }
}

#[test]
fn doubles_match_numpy() {
    for row in rows("doubles") {
        let seed: u64 = row[0].parse().unwrap();
        let mut generator = Pcg64Dxsm::from_seed(seed);
        for (i, hex) in row[1..].iter().enumerate() {
            let expected = f64::from_bits(u64::from_str_radix(hex, 16).unwrap());
            assert_eq!(
                generator.next_f64().to_bits(),
                expected.to_bits(),
                "seed {seed} double {i}"
            );
        }
    }
}

#[test]
fn substreams_match_numpy_spawn_keys() {
    for row in rows("substream") {
        let seed: u64 = row[0].parse().unwrap();
        let name = &row[1];
        let mut generator = RngRoot::from_seed(seed).substream(name).sequential();
        for (i, hex) in row[2..].iter().enumerate() {
            let expected = u64::from_str_radix(hex, 16).unwrap();
            assert_eq!(
                generator.next_u64(),
                expected,
                "seed {seed} substream {name} draw {i}"
            );
        }
    }
}

#[test]
fn frame_forks_match_numpy_spawn_keys() {
    for row in rows("fork") {
        let seed: u64 = row[0].parse().unwrap();
        let name = &row[1];
        let frame: u64 = row[2].parse().unwrap();
        let mut generator = RngRoot::from_seed(seed).substream(name).fork_frame(frame);
        for (i, hex) in row[3..].iter().enumerate() {
            let expected = u64::from_str_radix(hex, 16).unwrap();
            assert_eq!(
                generator.next_u64(),
                expected,
                "seed {seed} fork {name}/{frame} draw {i}"
            );
        }
    }
}
