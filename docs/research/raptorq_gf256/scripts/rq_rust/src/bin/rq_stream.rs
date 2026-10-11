//! Measurement 2: the broadcast "hook stream" with the real decoder (crate raptorq =2.0.0).
//!
//! Usage: rq_stream K R trials [threads=4] [seed=20261011] [hmax=2]
//!
//! T = 8. One Encoder::with_defaults(data, 8) per K; pool = get_encoded_packets(R) =
//! K source packets (ESI 0..K-1) followed by R repair packets, in that send order.
//! Trial i (phone i): ChaCha12Rng::seed_from_u64(seed) with set_stream(i); each sent packet is
//! lost iff gen_range(0..100u32) == 0 (i.i.d., probability exactly 1/100). The phone keeps the
//! first K+hmax packets it receives, in send order, and feeds them one at a time to
//! Decoder::new(config).decode(packet), stopping at the first Some (rq_mc.py's usage).
//! A phone that keeps only the first K+h packets sees exactly the first K+h of these calls, so for
//! every h <= hmax:  success(h) = first Some came at packet n <= K+h AND it equals the data.
//! (All h are scored on the same phones/loss patterns; results are exact, not extrapolated.)
//! A phone that receives fewer than K+hmax packets out of K+R is counted as ran_out (asserted 0).
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use raptorq::{extended_source_block_symbols, Decoder, Encoder, EncodingPacket, ObjectTransmissionInformation};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Default, Clone)]
struct Stats {
    trials: u64,
    fail: Vec<u64>,
    wrong_data: Vec<u64>,
    ran_out: u64,
    no_source_lost: u64,
    lost_src_sum: u64,
    lost_src_max: u64,
    repair_used_max: u64, // largest repair index (0-based, in send order) any phone needed + 1
    fail_trials_h0: Vec<u64>,
    fail_trials_ge1: Vec<(usize, u64)>, // (h, trial) for every failure with h >= 1
}

impl Stats {
    fn new(hmax: usize) -> Self {
        Stats { fail: vec![0; hmax + 1], wrong_data: vec![0; hmax + 1], ..Default::default() }
    }
    fn merge(&mut self, o: &Stats) {
        self.trials += o.trials;
        for h in 0..self.fail.len() {
            self.fail[h] += o.fail[h];
            self.wrong_data[h] += o.wrong_data[h];
        }
        self.ran_out += o.ran_out;
        self.no_source_lost += o.no_source_lost;
        self.lost_src_sum += o.lost_src_sum;
        self.lost_src_max = self.lost_src_max.max(o.lost_src_max);
        self.repair_used_max = self.repair_used_max.max(o.repair_used_max);
        self.fail_trials_h0.extend_from_slice(&o.fail_trials_h0);
        self.fail_trials_ge1.extend_from_slice(&o.fail_trials_ge1);
    }
}

#[allow(clippy::too_many_arguments)]
fn run_trial(
    idx: u64,
    seed: u64,
    k: usize,
    hmax: usize,
    pool: &[EncodingPacket],
    data: &[u8],
    config: ObjectTransmissionInformation,
    st: &mut Stats,
) {
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    rng.set_stream(idx);
    let need = k + hmax;
    let mut kept: Vec<usize> = Vec::with_capacity(need);
    let mut lost_src = 0u64;
    for i in 0..pool.len() {
        if rng.gen_range(0..100u32) == 0 {
            if i < k {
                lost_src += 1;
            }
            continue;
        }
        kept.push(i);
        if kept.len() == need {
            break;
        }
    }
    st.trials += 1;
    st.lost_src_sum += lost_src;
    st.lost_src_max = st.lost_src_max.max(lost_src);
    if lost_src == 0 {
        st.no_source_lost += 1;
    }
    if kept.len() < need {
        st.ran_out += 1;
        for h in 0..=hmax {
            st.fail[h] += 1;
        }
        return;
    }
    let last = *kept.last().unwrap();
    if last >= k {
        st.repair_used_max = st.repair_used_max.max((last - k + 1) as u64);
    }
    let mut dec = Decoder::new(config);
    let mut first: Option<(usize, bool)> = None;
    for (n, &i) in kept.iter().enumerate() {
        if let Some(r) = dec.decode(pool[i].clone()) {
            first = Some((n + 1, r.as_slice() == data));
            break;
        }
    }
    for h in 0..=hmax {
        let ok = match first {
            Some((n, correct)) if n <= k + h => {
                if !correct {
                    st.wrong_data[h] += 1;
                }
                correct
            }
            _ => false,
        };
        if !ok {
            st.fail[h] += 1;
            if h == 0 && st.fail_trials_h0.len() < 1000 {
                st.fail_trials_h0.push(idx);
            }
            if h >= 1 {
                st.fail_trials_ge1.push((h, idx));
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k: usize = args[1].parse().unwrap();
    let r: u32 = args[2].parse().unwrap();
    let trials: u64 = args[3].parse().unwrap();
    let threads: usize = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(4);
    let seed: u64 = args.get(5).map(|s| s.parse().unwrap()).unwrap_or(20261011);
    let hmax: usize = args.get(6).map(|s| s.parse().unwrap()).unwrap_or(2);
    let t: u16 = 8;

    let mut drng = ChaCha12Rng::seed_from_u64(seed ^ 0xda7a);
    let mut data = vec![0u8; k * t as usize];
    drng.fill(&mut data[..]);
    let t0 = Instant::now();
    let encoder = Encoder::with_defaults(&data, t);
    let config = encoder.get_config();
    assert_eq!(config.source_blocks(), 1);
    let pool = encoder.get_encoded_packets(r);
    let kprime = extended_source_block_symbols(k as u32) as usize;
    assert_eq!(pool.len(), k + r as usize);
    for (i, p) in pool.iter().enumerate() {
        let esi = p.payload_id().encoding_symbol_id() as usize;
        // RFC 6330 5.3.1: source ESI 0..K-1, repair ESI K, K+1, ... (crate maps ISI = ESI + K' - K)
        assert_eq!(esi, i, "unexpected ESI order");
    }
    println!(
        "rq_stream K={} K'={} T={} R={} trials={} threads={} seed={} hmax={} loss=1/100 iid; sub_blocks={} encode_ms={:.1}",
        k,
        kprime,
        t,
        r,
        trials,
        threads,
        seed,
        hmax,
        config.sub_blocks(),
        t0.elapsed().as_secs_f64() * 1e3
    );

    let next = AtomicU64::new(0);
    let done = AtomicU64::new(0);
    let total = Mutex::new(Stats::new(hmax));
    let chunk = 16u64;
    let start = Instant::now();
    std::thread::scope(|s| {
        for _ in 0..threads {
            s.spawn(|| {
                let mut st = Stats::new(hmax);
                loop {
                    let a = next.fetch_add(chunk, Ordering::Relaxed);
                    if a >= trials {
                        break;
                    }
                    let b = (a + chunk).min(trials);
                    for idx in a..b {
                        run_trial(idx, seed, k, hmax, &pool, &data, config, &mut st);
                    }
                    done.fetch_add(b - a, Ordering::Relaxed);
                }
                total.lock().unwrap().merge(&st);
            });
        }
        let mut last = Instant::now();
        while done.load(Ordering::Relaxed) < trials {
            std::thread::sleep(Duration::from_millis(200));
            if last.elapsed() >= Duration::from_secs(60) {
                last = Instant::now();
                eprintln!(
                    "progress {}/{} elapsed {:.0}s",
                    done.load(Ordering::Relaxed),
                    trials,
                    start.elapsed().as_secs_f64()
                );
            }
        }
    });
    let el = start.elapsed().as_secs_f64();
    let mut st = total.into_inner().unwrap();
    st.fail_trials_h0.sort();
    st.fail_trials_ge1.sort();
    assert_eq!(st.trials, trials);
    for h in 0..=hmax {
        println!(
            "K={} recv=K+{} trials={}: failures={} (of which Some-but-wrong-data={}) rate={:.3e}",
            k,
            h,
            st.trials,
            st.fail[h],
            st.wrong_data[h],
            st.fail[h] as f64 / st.trials as f64
        );
    }
    println!(
        "ran_out={} no_source_lost={} mean_source_lost={:.4} max_source_lost={} max_repair_packets_needed={} (of R={})",
        st.ran_out,
        st.no_source_lost,
        st.lost_src_sum as f64 / st.trials as f64,
        st.lost_src_max,
        st.repair_used_max,
        r
    );
    println!("sample of failing trial indices at h=0 (each thread keeps its first <=1000; union, sorted): {:?}", st.fail_trials_h0);
    println!("failing (h, trial) pairs for h>=1: {:?}", st.fail_trials_ge1);
    println!("wall_s={:.1} trials_per_s={:.1}", el, st.trials as f64 / el);
    assert_eq!(st.ran_out, 0, "some phone ran out of packets");
}
