//! Measurement 1: wall time of ONE RaptorQ decode with the real decoder (crate raptorq =2.0.0).
//!
//! Usage: rq_cost K T [runs=3] [seed=2026]
//!
//! Encoder::with_defaults(data, T) on K*T random bytes (one source block is asserted).
//! Two packet sets of exactly K+2 packets each:
//!   "realistic":   all K source packets except a random 1% (floor(K/100), new random set per run)
//!                  plus the first floor(K/100)+2 repair packets (ESI order);
//!   "repair_only": K+2 repair packets (a disjoint ESI range per run).
//! Two ways to decode each set (single-threaded; the crate has no internal threads):
//!   "stream": Decoder::new(config) and decode(packet) one packet at a time until it returns Some
//!             (exactly rq_mc.py's API usage; the decoder first attempts a solve at the K-th packet
//!             and re-solves from scratch at every later packet while it keeps failing);
//!   "batch":  SourceBlockDecoder::new(0, config, K*T).decode(all K+2 packets): one solve, K+2 rows.
//! The timed region is from decoder construction to the returned result (packet cloning excluded).
//! Every result is compared with the original data.
use rand::seq::index::sample;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha12Rng;
use raptorq::{extended_source_block_symbols, Decoder, Encoder, EncodingPacket, SourceBlockDecoder};
use std::time::{Duration, Instant};

fn ms(d: Duration) -> f64 {
    d.as_secs_f64() * 1e3
}

fn median(v: &mut [f64]) -> f64 {
    v.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = v.len();
    if n % 2 == 1 {
        v[n / 2]
    } else {
        0.5 * (v[n / 2 - 1] + v[n / 2])
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let k: usize = args[1].parse().unwrap();
    let t: u16 = args[2].parse().unwrap();
    let runs: usize = args.get(3).map(|s| s.parse().unwrap()).unwrap_or(3);
    let seed: u64 = args.get(4).map(|s| s.parse().unwrap()).unwrap_or(2026);

    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let mut data = vec![0u8; k * t as usize];
    rng.fill(&mut data[..]);

    // Encoder setup (Encoder::with_defaults = OTI + encoding plan + intermediate-symbol solve),
    // timed `runs` times; the last encoder is kept.
    let mut enc_times = vec![];
    let mut encoder = None;
    for _ in 0..runs {
        let t0 = Instant::now();
        let e = Encoder::with_defaults(&data, t);
        enc_times.push(ms(t0.elapsed()));
        encoder = Some(e);
    }
    let encoder = encoder.unwrap();
    let enc_all = enc_times.iter().map(|x| format!("{:.3}", x)).collect::<Vec<_>>().join(",");
    let enc_median = median(&mut enc_times.clone());
    let config = encoder.get_config();
    assert_eq!(config.source_blocks(), 1, "expected one source block");
    let kprime = extended_source_block_symbols(k as u32);
    println!(
        "K={} T={} K'={} padding={} transfer_length={} source_blocks={} sub_blocks={} alignment={} seed={} runs={}",
        k,
        t,
        kprime,
        kprime as usize - k,
        config.transfer_length(),
        config.source_blocks(),
        config.sub_blocks(),
        config.symbol_alignment(),
        seed,
        runs
    );

    let drop = k / 100;
    let n_repair = (runs * (k + 2)).max(drop + 2);
    let t1 = Instant::now();
    let pool: Vec<EncodingPacket> = encoder.get_encoded_packets(n_repair as u32);
    let gen_time = t1.elapsed();
    assert_eq!(pool.len(), k + n_repair);
    for (i, p) in pool.iter().enumerate() {
        let esi = p.payload_id().encoding_symbol_id() as usize;
        // RFC 6330 5.3.1: source ESI 0..K-1, repair ESI K, K+1, ... (crate maps ISI = ESI + K' - K)
        let want = i;
        assert_eq!(esi, want, "unexpected ESI order");
    }
    println!(
        "encoder_setup Encoder::with_defaults: median {:.3} ms runs_ms=[{}]",
        enc_median, enc_all
    );
    println!(
        "get_encoded_packets({}) [K source + {} repair]: {:.3} ms",
        n_repair,
        n_repair,
        ms(gen_time)
    );

    let mut res: Vec<(String, String, Vec<f64>)> = vec![];
    for case in ["realistic", "repair_only"] {
        for mode in ["stream", "batch"] {
            let mut times = vec![];
            for run in 0..runs {
                let pk: Vec<EncodingPacket> = if case == "realistic" {
                    let mut r2 = ChaCha12Rng::seed_from_u64(seed ^ 0x5eed);
                    r2.set_stream(run as u64);
                    let dropped = sample(&mut r2, k, drop);
                    let mut keep = vec![true; k];
                    for d in dropped.iter() {
                        keep[d] = false;
                    }
                    let mut v: Vec<EncodingPacket> =
                        (0..k).filter(|&i| keep[i]).map(|i| pool[i].clone()).collect();
                    v.extend(pool[k..k + drop + 2].iter().cloned());
                    v
                } else {
                    pool[k + run * (k + 2)..k + (run + 1) * (k + 2)].to_vec()
                };
                assert_eq!(pk.len(), k + 2);
                let n_src = pk
                    .iter()
                    .filter(|p| (p.payload_id().encoding_symbol_id() as usize) < k)
                    .count();
                let (el, out, at) = if mode == "stream" {
                    let start = Instant::now();
                    let mut dec = Decoder::new(config);
                    let mut out = None;
                    let mut at = 0;
                    for (i, p) in pk.into_iter().enumerate() {
                        if let Some(r) = dec.decode(p) {
                            out = Some(r);
                            at = i + 1;
                            break;
                        }
                    }
                    (start.elapsed(), out, at)
                } else {
                    let start = Instant::now();
                    let mut sbd = SourceBlockDecoder::new(0, &config, (k * t as usize) as u64);
                    let out = sbd.decode(pk);
                    (start.elapsed(), out, k + 2)
                };
                let ok = out.as_ref().map(|r| *r == data).unwrap_or(false);
                println!(
                    "case={} mode={} run={} packets=K+2={} source_pkts={} repair_pkts={} returned_some_at_packet={} correct={} time_ms={:.3}",
                    case,
                    mode,
                    run,
                    k + 2,
                    n_src,
                    k + 2 - n_src,
                    if out.is_some() { at.to_string() } else { "never".into() },
                    ok,
                    ms(el)
                );
                times.push(ms(el));
            }
            res.push((case.into(), mode.into(), times));
        }
    }
    println!("--- summary K={} T={} (median of {} runs, ms) ---", k, t, runs);
    println!("encoder_setup median_ms={:.3} runs_ms=[{}]", enc_median, enc_all);
    println!("get_encoded_packets({}) ms={:.3} (single call)", n_repair, ms(gen_time));
    for (c, m, mut v) in res {
        let all = v.iter().map(|x| format!("{:.3}", x)).collect::<Vec<_>>().join(",");
        println!("{} {} median_ms={:.3} runs_ms=[{}]", c, m, median(&mut v), all);
    }
}
