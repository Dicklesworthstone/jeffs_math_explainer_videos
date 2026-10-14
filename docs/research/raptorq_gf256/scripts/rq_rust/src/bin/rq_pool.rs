//! Rust mirror of docs/research/raptorq_gf256/scripts/rq_mc.py (crate raptorq =2.0.0).
//!
//! Usage: rq_pool K trials R [threads=4] [seed=2026] [hmax=2]
//!
//! T = 8. pool = Encoder::with_defaults(data, 8).get_encoded_packets(R) (K source + R repair).
//! For each h in 0..=hmax independently (like rq_mc.py): each trial draws K+h distinct packets
//! uniformly at random from the pool, in random order (rand::seq::index::sample is fully
//! shuffled, like Python's random.sample), feeds them one at a time to
//! Decoder::new(config).decode, stops at the first Some, and fails if none or result != data.
//! Trial (h, i) uses ChaCha12Rng::seed_from_u64(seed) with set_stream((h << 40) | i).
use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use raptorq::{Decoder, Encoder};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k: usize = args[1].parse().unwrap();
    let trials: u64 = args[2].parse().unwrap();
    let r: u32 = args[3].parse().unwrap();
    let threads: usize = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(4);
    let seed: u64 = args.get(5).map(|s| s.parse().unwrap()).unwrap_or(2026);
    let hmax: u64 = args.get(6).map(|s| s.parse().unwrap()).unwrap_or(2);
    let t: u16 = 8;

    let mut drng = ChaCha12Rng::seed_from_u64(seed ^ 0xda7a);
    let mut data = vec![0u8; k * t as usize];
    drng.fill(&mut data[..]);
    let encoder = Encoder::with_defaults(&data, t);
    let config = encoder.get_config();
    assert_eq!(config.source_blocks(), 1);
    let pool = encoder.get_encoded_packets(r);
    assert_eq!(pool.len(), k + r as usize);
    println!(
        "rq_pool K={} T={} R={} trials={} threads={} seed={} sub_blocks={}",
        k, t, r, trials, threads, seed, config.sub_blocks()
    );
    let t0 = Instant::now();
    for h in 0..=hmax {
        let next = AtomicU64::new(0);
        let fails = AtomicU64::new(0);
        let wrong = AtomicU64::new(0);
        let n = k + h as usize;
        std::thread::scope(|s| {
            for _ in 0..threads {
                s.spawn(|| loop {
                    let a = next.fetch_add(16, Ordering::Relaxed);
                    if a >= trials {
                        break;
                    }
                    for i in a..(a + 16).min(trials) {
                        let mut rng = ChaCha12Rng::seed_from_u64(seed);
                        rng.set_stream((h << 40) | i);
                        let idx = sample(&mut rng, pool.len(), n);
                        let mut dec = Decoder::new(config);
                        let mut out = None;
                        for j in idx.iter() {
                            if let Some(res) = dec.decode(pool[j].clone()) {
                                out = Some(res);
                                break;
                            }
                        }
                        match out {
                            Some(res) if res == data => {}
                            Some(_) => {
                                wrong.fetch_add(1, Ordering::Relaxed);
                                fails.fetch_add(1, Ordering::Relaxed);
                            }
                            None => {
                                fails.fetch_add(1, Ordering::Relaxed);
                            }
                        }
                    }
                });
            }
        });
        let f = fails.load(Ordering::Relaxed);
        println!(
            "K={} recv=K+{} trials={}: failures={} (Some-but-wrong-data={}) rate={:.3e}",
            k,
            h,
            trials,
            f,
            wrong.load(Ordering::Relaxed),
            f as f64 / trials as f64
        );
    }
    println!("time {:.1}s", t0.elapsed().as_secs_f64());
}
