//! The one RNG (§6.5, D-06, BN-01): PCG64DXSM with named substreams and
//! keyed per-frame forks.
//!
//! FrankenManim standardizes on NumPy-compatible PCG64DXSM seeded through
//! `SeedSequence`. The primitive state machine is owned by FrankenNumPy's
//! dependency-free `fnp-random-core`; this module owns only the scene-level
//! stream layout and the compatibility wrapper used by journals.

/// Render-affecting map iteration must be ordered (§6.5).
pub type OrderedMap<K, V> = std::collections::BTreeMap<K, V>;

/// Version of the root-seed, named-substream, and frame-fork derivation layout.
pub const RNG_LAYOUT_VERSION: u32 = 1;

/// The governed upstream SeedSequence authority.
pub use fnp_random_core::SeedSequence;

/// Compatibility wrapper around the governed upstream PCG64DXSM authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Pcg64Dxsm {
    inner: fnp_random_core::Pcg64Dxsm,
}

impl Pcg64Dxsm {
    /// Seed from a NumPy-compatible SeedSequence.
    #[must_use]
    pub fn from_seed_sequence(sequence: &SeedSequence) -> Self {
        Self {
            inner: fnp_random_core::Pcg64Dxsm::from_seed_sequence(sequence),
        }
    }

    /// Construct exactly as `numpy.random.PCG64DXSM(seed)`.
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self {
            inner: fnp_random_core::Pcg64Dxsm::from_seed(seed),
        }
    }

    /// Generate the next raw 64-bit word.
    pub fn next_u64(&mut self) -> u64 {
        self.inner.next_u64()
    }

    /// Generate NumPy's 53-bit floating value in `[0, 1)`.
    pub fn next_f64(&mut self) -> f64 {
        self.inner.next_f64()
    }

    /// Full split state for SceneState and the replay journal.
    #[must_use]
    pub fn state(&self) -> ([u64; 2], [u64; 2]) {
        self.inner.split_state()
    }

    /// Restore a state returned by [`Self::state`].
    #[must_use]
    pub fn restore(state: [u64; 2], increment: [u64; 2]) -> Self {
        Self {
            inner: fnp_random_core::Pcg64Dxsm::from_split_state(state, increment),
        }
    }
}

fn name_words(name: &str) -> Vec<u32> {
    let bytes = name.as_bytes();
    let byte_len = u32::try_from(bytes.len()).expect("substream name exceeds u32 length");
    let mut words = Vec::with_capacity(1 + bytes.len().div_ceil(4));
    words.push(byte_len);
    for chunk in bytes.chunks(4) {
        let mut padded = [0_u8; 4];
        padded[..chunk.len()].copy_from_slice(chunk);
        words.push(u32::from_le_bytes(padded));
    }
    words
}

/// A scene root: one explicit seed and any number of independent substreams.
#[derive(Debug, Clone)]
pub struct RngRoot {
    seed: u64,
}

impl RngRoot {
    /// Construct the deterministic root. Host entropy is supplied by
    /// fmn-platform before this semantic layer is entered.
    #[must_use]
    pub const fn from_seed(seed: u64) -> Self {
        Self { seed }
    }

    /// Return the explicit root seed.
    #[must_use]
    pub const fn seed(&self) -> u64 {
        self.seed
    }

    /// Derive the stable named substream handle for a subsystem.
    #[must_use]
    pub fn substream(&self, name: &str) -> Substream {
        Substream {
            seed: self.seed,
            key: name_words(name),
        }
    }
}

/// A named substream with one sequential stream and pure frame-keyed forks.
#[derive(Debug, Clone)]
pub struct Substream {
    seed: u64,
    key: Vec<u32>,
}

impl Substream {
    /// Construct the substream's sequential generator.
    #[must_use]
    pub fn sequential(&self) -> Pcg64Dxsm {
        let sequence = SeedSequence::with_spawn_key(self.seed, &self.key);
        Pcg64Dxsm::from_seed_sequence(&sequence)
    }

    /// Derive a generator from `(substream, frame_index)` without consuming
    /// sequential state. Calls are independent of order and thread placement.
    #[must_use]
    pub fn fork_frame(&self, frame_index: u64) -> Pcg64Dxsm {
        let mut key = Vec::with_capacity(self.key.len() + 2);
        key.extend_from_slice(&self.key);
        key.push(frame_index as u32);
        key.push((frame_index >> 32) as u32);
        let sequence = SeedSequence::with_spawn_key(self.seed, &key);
        Pcg64Dxsm::from_seed_sequence(&sequence)
    }
}

#[cfg(test)]
mod tests {
    use super::{Pcg64Dxsm, RNG_LAYOUT_VERSION, RngRoot, SeedSequence};

    #[test]
    fn upstream_seed_sequence_and_pcg_vectors_remain_exact() {
        assert_eq!(RNG_LAYOUT_VERSION, 1);
        assert_eq!(
            SeedSequence::from_seed(42).generate_state(8),
            [
                0xcd54_0ab7,
                0x9f1e_2e6d,
                0x79fb_94b6,
                0xd578_73dc,
                0x64d4_20b7,
                0x7d28_2a1b,
                0x4692_d5ff,
                0x3365_7971,
            ]
        );
        let mut generator = Pcg64Dxsm::from_seed(42);
        let words: Vec<u64> = (0..8).map(|_| generator.next_u64()).collect();
        assert_eq!(
            words,
            [
                0xab1c_5033_8e63_481d,
                0x01bd_f91d_548d_1872,
                0xa872_905d_0418_d0a1,
                0x5f0a_8427_0b80_eabc,
                0x34e8_2505_4db5_f685,
                0x319f_f93c_b20c_b433,
                0xc24f_b90e_b5d6_26af,
                0xf1c7_6bf8_e2e9_99a6,
            ]
        );
    }

    #[test]
    fn state_round_trip_preserves_the_exact_stream() {
        let mut generator = Pcg64Dxsm::from_seed(u64::MAX);
        for _ in 0..257 {
            let _ = generator.next_u64();
        }
        let (state, increment) = generator.state();
        let mut restored = Pcg64Dxsm::restore(state, increment);
        for _ in 0..128 {
            assert_eq!(generator.next_u64(), restored.next_u64());
        }
    }

    #[test]
    fn substream_independence_is_structural() {
        let root = RngRoot::from_seed(7);
        let before: Vec<u64> = {
            let mut generator = root.substream("alpha").sequential();
            (0..8).map(|_| generator.next_u64()).collect()
        };
        let mut other = root.substream("beta").sequential();
        for _ in 0..10_000 {
            let _ = other.next_u64();
        }
        let _ = root.substream("new-consumer").sequential();
        let after: Vec<u64> = {
            let mut generator = root.substream("alpha").sequential();
            (0..8).map(|_| generator.next_u64()).collect()
        };
        assert_eq!(before, after);
    }

    #[test]
    fn frame_forks_are_call_order_invariant() {
        let substream = RngRoot::from_seed(99).substream("frames");
        let forward: Vec<u64> = (0..64)
            .map(|frame| substream.fork_frame(frame).next_u64())
            .collect();
        let mut backward: Vec<u64> = (0..64)
            .rev()
            .map(|frame| substream.fork_frame(frame).next_u64())
            .collect();
        backward.reverse();
        assert_eq!(forward, backward);
        assert_eq!(
            substream.fork_frame(17).next_u64(),
            substream.fork_frame(17).next_u64()
        );
    }

    #[test]
    fn name_encoding_separates_utf8_length_and_padding() {
        let root = RngRoot::from_seed(123);
        assert_ne!(
            root.substream("a").sequential().next_u64(),
            root.substream("a\0").sequential().next_u64()
        );
        assert_ne!(
            root.substream("é").sequential().next_u64(),
            root.substream("e").sequential().next_u64()
        );
    }
}
