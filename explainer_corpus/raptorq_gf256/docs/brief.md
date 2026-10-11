# Brief — The Arithmetic Where 3 × 7 = 9

Video B of the corpus (bead mx-e-b-qyy.1, gate G0). This file moves to
`explainer_corpus/raptorq_gf256/docs/brief.md` when the crate is scaffolded.
Values I chose myself are marked `(default)`. The owner reviews this file,
not a list of questions.

| Field | Value |
|---|---|
| Topic (owner's words) | "The finite field math that underlies RaptorQ fountain codes" |
| Working title | **The Arithmetic Where 3 × 7 = 9** (alternative: *Liquid Data*). 3·7 = 9 holds under every one of the 30 degree-8 fold polynomials, because the product has degree 3 and no fold happens (R1 RB-31; other labellings of GF(256), such as a normal basis, need not agree). |
| One-sentence takeaway | RaptorQ sends plain XOR packets; a random set of exactly enough XOR equations fails to pin down the file 71% of the time, but almost all of that risk lives in the last few equations, so RaptorQ closes those few directions with 10–16 check equations in a 256-element byte field, where a random equation misses any given hole only once in 256; and we can build that field because we multiply bytes as polynomials modulo one that cannot be factored. |
| The hook (where the obvious tool fails) | A million phones, each losing a different 1 in 100 packets, none able to ask twice. A carousel (send the file on a loop) makes most phones wait two or three loops and the unluckiest five or six. |
| Cold open (≤ 20 s, plan rule 20) | A million phones, each randomly losing about one packet in a hundred; almost every one rebuilds the picture from two packets more than it needs; the secret, "3 × 7 = 9" in chalk; then the title (storyboard 01_1–01_3, R1 B-14). |
| The pictures | (1) **The bucket's danger meter:** equations filling a space; the lit (already-known) fraction stays near zero for almost the whole fill and spikes only at the end. (2) **The byte clock:** a 255-hour clock where multiplying is adding clicks, set against the wrong 256-hour clock that adds. |
| Worked examples | A=5, B=3, C=7, D=2 with A⊕C=2, B⊕C⊕D=6, A⊕B=6, A⊕B⊕C⊕D=3 (rank 4) and the redundant B⊕C=4; GF(2)³ (8 patterns); 3·7 = 9, 2·128 = 29, 128·4 = 58, 150·3 = 167, 2·142 = 1, 0x57·0x83 = 0x31 (0xC1 under AES's polynomial); the 5-unknown peeling toy (C, A, B, D, E; c = 7); the pentagon (3A = 15 ⇒ A = 5). |
| Spoken vocabulary (plan rule 21) | **piece** (source symbol: a whole chunk of about 1 KB, not one byte), **packet** (encoding symbol), **card** for one equation on screen, **useful** (= independent), **wrap around** (said "mod" once), **byte field** (named GF(256) once). |
| Notation (screen) | From the glyph probe (spike S-7): the generator as `\text{α}` (upright and unmistakable), never `\alpha` in maths, which fmn draws as an italic Latin *a*; polynomials and powers in TeX (x⁸ = x⁴ + x³ + x² + 1), never Unicode superscripts in Text (they fall back to a bold sans); `\oplus` (heavier than Computer Modern, acceptable); hex (0x1D) and the Rust `xtime` / `mul` code in CM Typewriter (`Kit::mono`); "Reed–Solomon" with a literal en dash. |
| Colour roles | pieces A–E a fixed five-colour ramp (blue → teal → green → gold → orange); packets as warm white cards with coloured chips; XOR / GF(2) cool blue `#58C4DD`; the byte field gold `#FFD866`; lost packets red `#FC6255`, dimmed; the lit span translucent teal; the fold polynomial 0x11D orange `#F0AC5F`; wrap-around (Z/256) desaturated purple `#9A72AC` (plan §10.3). |
| Audience | The corpus's three viewers: Maya, 14; Dev, 28, who knows XOR and hashing; Ruth Adler, 45 (plan §1.2). Numbers before letters (plan rule 22). |
| Length target | ~13.5 min, 11 chapters ≤ 85 s each (plan D-05; chapter 03 was split at the G2 check; the owner may veto). |
| Format | 3840×2160 at 60 fps, 8-bit H.264 High at level 5.2 (x264 CRF 16, preset slow), exact BT.709 colour, AAC 256k, −16 LUFS by one fixed gain on the PCM master under −1.5 dBTP, SRT/WebVTT captions (default). Chapter 07's 256×256 table follows spike S-3's outcome. |
| Sound | The score engine with Video B's identity (plan §7.5): E major with a Lydian A♯, an E pedal under algebra, 96 BPM with sixteenth arpeggios in gaps only; Emaj9 → C♯m9 → Amaj9(♯11) → B6sus4; a warm-glass FM bell, Juno PWM strings, Karplus–Strong pluck, sub bass, tape wow, plate; the motif from 0x1D's set bits {0, 2, 3, 4}: E–G♯–B–C♯, sounding at every fold. Signature sonification in listen windows: a byte is an 8-note chord (bits → E4 F♯4 G♯4 B4 C♯5 E5 F♯5 G♯5), XOR toggles notes, ×α steps every note up a slot and an overflow flicks in the 0x1D chord; packets arrive as glass droplets, losses as dull ticks; the rank chord grows one tone per useful card. Narration at 0 dB. |
| Voice | FrankenTTS `robert`, voice-actor cadence, never time-stretched (default) |
| Look | 3Blue1Brown: dark background `#0E1116`, role colours, continuous morphs (default) |
| References | `docs/research/raptorq_gf256/` (the memo and its scripts, including an RFC 6330 implementation byte-identical to an independent one); plan §10 (arc, beats, facts ledger, engineering notes); `docs/reviews/plan_r1/`. RFC 6330 is cited, not vendored. |
| Owner script supplied? | no (default): written in P3 from the plan's draft words |
| Oracle | `oracle.py` imports `docs/research/raptorq_gf256/scripts/`; measured RaptorQ counts are recorded outputs pinned as constants, not recomputed per build (plan §10.1). |
| Pronunciations | RaptorQ ("raptor queue"), Galois, Reed–Solomon, Shokrollahi, Luby, Erdős, Borwein, ex-or, ATSC, RAID, "hex one D", polynomial, logarithm, Gaussian: each verified by the narration skill's probe before narration; the owner's pronunciation overrides. |
| Render host | this Mac with x264 until the GPU host's NVENC is restored (default) |
| franken_manim | git revision `e2eec6326cee490462e497509fce9289c8157bea` through the workspace pin; chapter 03's cube of GF(2)³ patterns runs on the camera route (spike S-5) |
| Out of scope | the RFC's tuple generator, systematic index tables, the robust-soliton derivation, LDPC row construction, Reed–Solomon decoding algorithms |
| Assumptions | The rows marked `(default)`; the arc of plan §10.2 ("each fix exposes the next problem"). |
