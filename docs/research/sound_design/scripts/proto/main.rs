// Throwaway TIMING prototype (not the production engine): std-only warm pad
// (2 PolyBLEP saws + sub, OU drift, ZDF ladder w/ saturation) + Juno-ish chorus
// + 8-line Householder FDN, stereo 48 kHz, written as 24-bit WAV with TPDF dither.
// Usage: proto [OUT.wav] [SECONDS]   (defaults: pad10min.wav 600)
// The memo's timing figure (6.27 s user CPU, M4 Pro, one core) is for SECONDS=600.
use std::f64::consts::{PI, TAU};
use std::io::Write;
const SR: f64 = 48_000.0;

#[derive(Clone)] struct Rng(u64);
impl Rng {
    fn u64(&mut self) -> u64 { self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0; z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB); z ^ (z >> 31) }
    fn uni(&mut self) -> f64 { (self.u64() >> 11) as f64 / (1u64 << 53) as f64 }
    fn bi(&mut self) -> f64 { 2.0 * self.uni() - 1.0 }
}
#[inline] fn poly_blep(t: f64, dt: f64) -> f64 {
    if t < dt { let t = t / dt; t + t - t * t - 1.0 }
    else if t > 1.0 - dt { let t = (t - 1.0) / dt; t * t + t + t + 1.0 } else { 0.0 }
}
#[inline] fn tanh_fast(x: f64) -> f64 { // Pade-ish, clamped; error < 2e-3 within |x|<3
    let x = x.clamp(-3.0, 3.0); let x2 = x * x; x * (27.0 + x2) / (27.0 + 9.0 * x2) }

struct Drift { x: f64, rng: Rng } // Ornstein-Uhlenbeck pitch wander in cents, control rate
impl Drift { fn step(&mut self, dt: f64, theta: f64, sigma: f64) -> f64 {
    self.x += -theta * self.x * dt + sigma * dt.sqrt() * self.rng.bi() * 3f64.sqrt(); self.x } }

struct Voice { f0: f64, ph: [f64; 3], drift: [Drift; 2], s: [f64; 4], t_on: f64, t_off: f64, amp: f64, pan: f64, cents: f64, g: f64, d0: f64, d1: f64 }
impl Voice {
    fn new(f0: f64, t_on: f64, t_off: f64, rng: &mut Rng, pan: f64) -> Voice {
        Voice { f0, ph: [rng.uni(), rng.uni(), rng.uni()],
            drift: [Drift { x: 0.0, rng: Rng(rng.u64()) }, Drift { x: 0.0, rng: Rng(rng.u64()) }],
            s: [0.0; 4], t_on, t_off, amp: 0.0, pan, cents: 7.0 + 3.0 * rng.uni(), g: 0.0, d0: 1.0, d1: 1.0 }
    }
    fn env(&self, t: f64) -> f64 { // slow attack, long release, linear-in-dB-ish via smoothstep
        let a = ((t - self.t_on) / 1.5).clamp(0.0, 1.0); let r = 1.0 - ((t - self.t_off) / 3.0).clamp(0.0, 1.0);
        let e = a.min(r); e * e * (3.0 - 2.0 * e) }
    fn done(&self, t: f64) -> bool { t > self.t_off + 3.0 }
}
fn main() {
    let args: Vec<String> = std::env::args().collect();
    let out_path = args.get(1).cloned().unwrap_or_else(|| "pad10min.wav".to_string());
    let secs: f64 = args.get(2).map(|a| a.parse().expect("SECONDS must be a number")).unwrap_or(600.0);
    let n = (secs * SR) as usize;
    let mut out_l = vec![0.0f64; n]; let mut out_r = vec![0.0f64; n];
    let mut rng = Rng(0x5EED);
    // Voice-led progression in D: Dmaj9 -> Bm11 -> Gmaj7#11 -> A7sus4 (MIDI), 8 s per chord
    let prog: [[i32; 5]; 4] = [[50, 57, 64, 66, 69], [47, 57, 62, 64, 69], [43, 54, 61, 62, 66], [45, 55, 62, 64, 67]];
    let midi = |m: i32| 440.0 * 2f64.powf((m as f64 - 69.0) / 12.0);
    let mut voices: Vec<Voice> = Vec::new();
    let chord_len = 8.0; let nchords = (secs / chord_len) as usize;
    for c in 0..nchords { for (k, &m) in prog[c % 4].iter().enumerate() {
        let t0 = c as f64 * chord_len; voices.push(Voice::new(midi(m), t0, t0 + chord_len, &mut rng, if k % 2 == 0 { -0.4 } else { 0.4 })); } }
    let t_start = std::time::Instant::now();
    // ---- voices: 2 polyBLEP saws (detuned, drifting) + sub triangle -> ZDF ladder (Zavalishin TPT) ----
    let block = 32;
    for v in voices.iter_mut() {
        let i0 = (v.t_on * SR) as usize; let i1 = (((v.t_off + 3.0) * SR) as usize).min(n);
        let cutoff = (v.f0 * 3.0).min(1800.0) + 250.0; // key-tracked, dark
        let k = 1.2; // ladder resonance (0..4)
        let mut i = i0;
        while i < i1 {
            let t = i as f64 / SR;
            v.d0 = 2f64.powf((v.cents + v.drift[0].step(block as f64 / SR, 0.7, 4.0)) / 1200.0);
            v.d1 = 2f64.powf((-v.cents + v.drift[1].step(block as f64 / SR, 0.7, 4.0)) / 1200.0);
            let env_a = v.env(t); let env_b = v.env(t + block as f64 / SR);
            let fc = cutoff * (0.6 + 0.4 * env_a);
            v.g = (PI * fc / SR).tan(); let gg = v.g / (1.0 + v.g); let beta = 1.0 / (1.0 + v.g);
            let gl = (v.pan * 0.5 + 0.5) * PI / 2.0; let (pl, pr) = (gl.cos(), gl.sin());
            for j in 0..block.min(i1 - i) {
                let e = env_a + (env_b - env_a) * j as f64 / block as f64;
                let mut x = 0.0;
                for (o, d) in [(0usize, v.d0), (1, v.d1)] { let dt = v.f0 * d / SR; let p = v.ph[o];
                    x += 2.0 * p - 1.0 - poly_blep(p, dt); v.ph[o] = if p + dt >= 1.0 { p + dt - 1.0 } else { p + dt }; }
                let dt = 0.5 * v.f0 / SR; let p = v.ph[2]; x += 0.5 * (4.0 * (p - 0.5).abs() - 1.0); v.ph[2] = (p + dt).fract();
                // linear ZDF ladder solve, then saturate the ladder input
                let s = &mut v.s; let g2 = gg * gg; let g4 = g2 * g2;
                let sum = beta * (gg * g2 * s[0] + g2 * s[1] + gg * s[2] + s[3]);
                let y4 = (g4 * x * 0.35 + sum) / (1.0 + k * g4);
                let mut u = tanh_fast((x * 0.35 - k * y4) * 1.5) / 1.5;
                for st in s.iter_mut() { let vv = (u - *st) * gg; let y = vv + *st; *st = y + vv; u = y; }
                let y = u * (1.0 + 0.5 * k) * e * 0.12;
                out_l[i + j] += y * pl; out_r[i + j] += y * pr;
            }
            i += block;
        }
    }
    let t_vo = t_start.elapsed().as_secs_f64();
    // ---- Juno-style chorus: triangle LFO 0.5 Hz, 3.5 ms +/- 1.8 ms, L/R opposite phase ----
    let dl = 4096; let mut buf = vec![0.0f64; dl]; let mut w = 0usize;
    for i in 0..n {
        let m = 0.5 * (out_l[i] + out_r[i]); buf[w] = m;
        let lfo = 4.0 * ((0.5 * i as f64 / SR).fract() - 0.5).abs() - 1.0;
        let tap = |d_ms: f64, buf: &Vec<f64>| { let d = d_ms * 1e-3 * SR; let pos = w as f64 - d + dl as f64;
            let i0 = pos.floor() as usize; let fr = pos - pos.floor(); let a = buf[i0 % dl]; let b = buf[(i0 + 1) % dl]; a + (b - a) * fr };
        let wl = tap(3.5 + 1.8 * lfo, &buf); let wr = tap(3.5 - 1.8 * lfo, &buf);
        out_l[i] = 0.7 * out_l[i] + 0.5 * wl; out_r[i] = 0.7 * out_r[i] + 0.5 * wr; w = (w + 1) % dl;
    }
    // ---- 8-line FDN, Householder feedback, per-line one-pole damping, T60 = 2.8 s ----
    let lens = [1427usize, 1777, 2011, 2203, 2539, 2861, 3089, 3373];
    let t60 = 2.8; let mut lines: Vec<Vec<f64>> = lens.iter().map(|&l| vec![0.0; l]).collect();
    let mut idx = [0usize; 8]; let mut lp = [0.0f64; 8];
    let gains: Vec<f64> = lens.iter().map(|&l| 10f64.powf(-3.0 * l as f64 / (t60 * SR))).collect();
    let damp = 0.35; let pre = (0.025 * SR) as usize; let mut pbuf = vec![0.0f64; pre + 1]; let mut pw = 0;
    for i in 0..n {
        let x_in = 0.5 * (out_l[i] + out_r[i]); let x = pbuf[pw]; pbuf[pw] = x_in; pw = (pw + 1) % pbuf.len();
        let mut o = [0.0f64; 8]; for k in 0..8 { o[k] = lines[k][idx[k]]; }
        let mut s = 0.0; for k in 0..8 { lp[k] = o[k] * gains[k] * (1.0 - damp) + lp[k] * damp; s += lp[k]; }
        let hh = 2.0 / 8.0 * s;
        for k in 0..8 { let sign = if k % 2 == 0 { 1.0 } else { -1.0 };
            lines[k][idx[k]] = lp[k] - hh + sign * x * 0.35; idx[k] = (idx[k] + 1) % lens[k]; }
        let rl = o[0] - o[2] + o[4] - o[6]; let rr = o[1] - o[3] + o[5] - o[7];
        out_l[i] += 0.22 * rl; out_r[i] += 0.22 * rr;
    }
    let t_all = t_start.elapsed().as_secs_f64();
    // ---- write 24-bit WAV with TPDF dither ----
    let mut peak = 0.0f64; for i in 0..n { peak = peak.max(out_l[i].abs()).max(out_r[i].abs()); }
    let norm = 0.5 / peak; let mut d = Rng(7);
    let mut bytes = Vec::with_capacity(44 + n * 6);
    let data = (n * 6) as u32; bytes.extend_from_slice(b"RIFF"); bytes.extend_from_slice(&(36 + data).to_le_bytes());
    bytes.extend_from_slice(b"WAVEfmt "); bytes.extend_from_slice(&16u32.to_le_bytes()); bytes.extend_from_slice(&1u16.to_le_bytes());
    bytes.extend_from_slice(&2u16.to_le_bytes()); bytes.extend_from_slice(&48000u32.to_le_bytes()); bytes.extend_from_slice(&(48000u32 * 6).to_le_bytes());
    bytes.extend_from_slice(&6u16.to_le_bytes()); bytes.extend_from_slice(&24u16.to_le_bytes()); bytes.extend_from_slice(b"data"); bytes.extend_from_slice(&data.to_le_bytes());
    for i in 0..n { for s in [out_l[i], out_r[i]] {
        let q = (s * norm * 8_388_607.0 + d.uni() - d.uni()).round().clamp(-8_388_608.0, 8_388_607.0) as i32;
        bytes.extend_from_slice(&q.to_le_bytes()[..3]); } }
    std::fs::File::create(&out_path).expect("cannot create output WAV").write_all(&bytes).unwrap();
    eprintln!("voices {} | voice render {:.2}s | +chorus+fdn {:.2}s | total incl. write {:.2}s | peak {:.3}", voices.len(), t_vo, t_all, t_start.elapsed().as_secs_f64(), peak);
    let _ = TAU;
}
