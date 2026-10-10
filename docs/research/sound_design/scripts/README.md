# Sound-design measurement scripts

These scripts back the computed numbers in `../MEMO.md` (2026-10-09). They are research tools, not production code.

**Running them:** `python3 -I -B <path>/scripts/<script>.py ARGS` from any working directory.
- Scripts that share helpers (`lufs.py`, `current.py`, `sii.py`) add their own folder to `sys.path`, so `-I` is fine.
- `-B` stops `__pycache__` from being written here.

**Dependencies:** Python 3 with **numpy** and **scipy** (tested with numpy 2.5.3, scipy 1.18.1). For example: `uv run --with numpy --with scipy python3 -I -B voice.py DIR`.

**Narration input:** the narration WAVs are not committed. Pass a directory of per-sentence WAVs (24 kHz mono s16 from FrankenTTS "robert"), such as a local copy of `explainer_corpus/hoeffding_d/narration_v5/`.

**Saved runs:** `outputs/<script>_v5_out.txt` holds the smoke runs against narration_v5 made from this folder on 2026-10-09. Wall times were inflated by a heavily loaded machine.

## Scripts

- `lufs.py` — library, not run directly. BS.1770 K-weighting, gated integrated loudness, loudest-400 ms, 3 s short-term, 4× true peak, and a WAV reader that resamples to 48 kHz mono.
- `nar.py DIR [DIR…]` — per-sentence narration loudness and true peak (~30 s). Reproduces −23.3 LUFS median and −7.5 dBTP maximum (memo §1, §2.2).
- `voice.py DIR` — formats, long-term average speech spectrum in third-octaves, and leading/trailing silence. Its crude f0 has octave errors; use `yin2.py` for f0. Reproduces "the voice's strongest band is 250–630 Hz" (§1). About 2 min under load.
- `yin2.py DIR` — narrator f0 (YIN on 8 kHz audio) and pauses inside sentences (~30 s). Reproduces the f0 median 136 Hz / interquartile range 121–154 / 5–95% 107–200 Hz and the pause statistics 4.1 per sentence, median 0.20 s, p90 0.52 s, max 0.91 s (§1, §2.3). **See the provenance note below.**
- `current.py` (no arguments) — replica of the current pad and chime from the Hoeffding `sound.rs`, plus their measurements (~25 s). Reproduces: pad −38.3 LUFS; chime loudest-400 ms −25.6 to −26.2 LUFS; partials up to 4.75 kHz; 5.19-cent detune; beats 0.29–1.32 Hz; 12 dB modulation depth; 4.4 dB / 14.3 s breathe swing; centroid 257 Hz; −145 dB above 2 kHz (§0, §1).
- `sii.py DIR` — "SII-lite" energetic-masking estimate using ANSI S3.5 third-octave band importances. It is a QA heuristic, not ANSI-compliant (~1 min). Reproduces:
  - the pad's SII loss of 0.013, and the pad sitting only about 5 dB below the voice in the 160/200 Hz bands
  - a chime's first 400 ms costing 0.107–0.155 alone, or 0.119–0.165 together with the pad. The memo's "0.11–0.165" spans both. (§0, §1)
- `cue.py DIR [--first N]` — speech loudest-400 ms (median over the first N = 20 files) and the SII loss of a soft modal marimba cue at −14 / −18 / −20 / −22 LU below it (~1 min). Reproduces −20.1 LUFS, 0.03–0.05 at −14 LU and 0.010–0.016 at −20 LU (§2.2).
- `duck.py DIR [--prefix c] [--breath 0.6]` — places the lines with a given prefix back to back with a fixed breath and reports the silent gaps and merged speech regions (~5 s). Reproduces "longest gap between sentences 0.75 s; merging gaps under 1.2 s leaves one region per chapter" (§2.3).
- `newpad.py PAD_WAV DIR` — measures a prototype pad render against the narration, plus a PolyBLEP-vs-naive aliasing check (~1 min). Reproduces:
  - SII loss ≈0.0085 at 15 LU below the voice and 0.0016 at ≈18 LU. A 64 s render gives 0.0088; the memo used the first 60 s of the 600 s render.
  - centroid ≈400 Hz, and energy above 2 kHz −35 dB
  - aliasing −20.3 dB naive vs −55.5 dB with PolyBLEP (§3.1)
- `proto/` (`main.rs`, `Cargo.toml`) — **a throwaway timing experiment, not the production engine.** It is a std-only Rust pad (2 PolyBLEP saws + sub, random-walk drift, ZDF ladder with saturation), Juno-style chorus and 8-line Householder FDN, written as a 24-bit WAV.
  - Run with `proto [OUT.wav] [SECONDS]` (defaults `pad10min.wav 600`).
  - Build with `cargo build --release --manifest-path proto/Cargo.toml --target-dir <somewhere outside the repo>`, or `rustc -C opt-level=3 -C target-cpu=native -o <out>/proto proto/main.rs`.
  - Reproduces the memo §6 figure: 600 s rendered in 6.27 s user CPU on one M4 Pro core. A 64 s render took 0.75 s user CPU (`outputs/proto_64s_out.txt`).
  - Use a 64 s render as `newpad.py` input.

## Provenance note: f0 and pauses (memo §1 and §2.3)

The memo's f0, pause and long-term-spectrum figures were measured on the **earlier take** of the same robert narration (the Hoeffding explainer's `narration/`, which is not in the repo). `outputs/yin2_pre_v5_out.txt` shows that `yin2.py` reproduces the f0 and pause figures exactly from this folder. The spectrum peak (250–630 Hz) is unchanged on v5 (`outputs/voice_v5_out.txt`).

On `narration_v5`, the same script gives (`outputs/yin2_v5_out.txt`):
- f0 median **140 Hz**, interquartile range **125–160 Hz**, 5–95% **105–216 Hz**
- pauses inside sentences: **2.5 per sentence**, median **0.38 s**, p90 **0.62 s**, max **0.85 s**

The memo's conclusions still hold on v5:
- merging gaps under 1.2 s still leaves one speech region per chapter
- the voice's pitch band still covers the B2–F#3 pad tones (5–95% range)

The v5 95th-percentile f0 (216 Hz) does reach almost to the memo's "upper voices from A3 (220 Hz)" floor. All other numbers in the memo were measured on narration_v5 and reproduce from the v5 outputs here.
