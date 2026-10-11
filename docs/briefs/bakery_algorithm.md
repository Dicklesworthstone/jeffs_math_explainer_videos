# Brief — The Algorithm Lamport Is Proudest Of

Video C of the corpus (bead mx-e-c-7v0.1, gate G0). This file moves to
`explainer_corpus/bakery_algorithm/docs/brief.md` when the crate is
scaffolded. Values I chose myself are marked `(default)`. The owner reviews
this file, not a list of questions.

| Field | Value |
|---|---|
| Topic (owner's words) | "The baker algorithm (see this explainer by me)" |
| Working title | **The Algorithm Lamport Is Proudest Of** (alternative: *Taking Turns When Memory Lies*); "proudest" is supported by the Computer History Museum oral history ("perhaps that's why I'm proudest of it"). |
| One-sentence takeaway | Customers can take turns safely using only slates each of them writes alone: take a ticket bigger than every ticket you see, break ties by name, and raise a "choosing" flag so nobody is judged before their ticket is written; and the proof never trusts a read that overlaps a write, so it survives memory that returns garbage. |
| The hook (where intuition fails) | A slate reads 9; someone is rewriting it as 10; glance mid-stroke and you might read 9, or 10, or, Lamport says, 2496. Could computers take turns safely with memory like that? |
| Cold open (≤ 20 s, plan rule 20) | The torn read on a slate (9 → 10 read as 2496), then the title; then the obvious check-then-set failing, "one plus one is one" (plan §11.4, chapter 01). |
| The pictures | (1) **Space-time lanes**, one per customer, time running left to right: reads are arrows in, writes arrows out, and the lanes double as a musical score. (2) **The progress square:** every order of two programs is a staircase, and the bad orders pass through a red square. The lexicographic grid supports chapter 04. |
| Worked examples | the 6 orders of two increments (4 lose a point); the 14-order sign (8 bad); the 8-step unwritten-ticket collision and the same order with the flag; "both see 5, both take 6"; slates 0, 3, 0, 0, 2 → customer 4 takes 4; a 4-customer run; the garbage-read traces e1 and e3; the 12-step store-buffer failure. |
| Spoken vocabulary (plan rule 21) | **customer** (lower-third "process"), **the room** (named "critical section" once), **order** (lower-third: counting each read, write and successful wait as one step), **slate** (register), **flag** (choosing), **ticket**. |
| Notation (screen) | From the glyph probe (spike S-7): the order as (n_j, j) < (n_i, i) with the words "comes before" on its first appearance (`\prec` renders as a heavy "<" anyway, and Lamport writes <); n_i = 1 + max(n_1, …, n_N) in TeX; the code (`volatile int[]`, `AtomicIntegerArray`, the `choosing` / `number` loops) in CM Typewriter (`Kit::mono`); the "3,027,633 states, no violation" card typesets as planned. |
| What the script must get right | Plan §11.1's ten points, among them: "the first mutual-exclusion algorithm that assumes no lower-level mutual exclusion, not even atomic reads and writes", never "the first correct solution"; Lamport noticed garbage tolerance after writing the proof; equal tickets are normal and (ticket, id) is unique; `choosing` is not a lock; tickets are unbounded; Acquire/Release is not enough on real CPUs; the max reads each slate once; quotes only verbatim. |
| Colour roles | customers 1–5 five fixed hues (blue, green, orange, purple, pink) for the whole video; the room warm red; the flag raised amber `#F0AC5F`; tickets and slates chalk white; the order a gold `#FFD866` scanline; garbage reads flickering magenta; store buffers ("outboxes") cool grey (plan §11.3). |
| Audience | The corpus's three viewers: Maya, 14; Dev, 28, a strong software developer; Ruth Adler, 45 (plan §1.2). Numbers before letters (plan rule 22). |
| Length target | ~13 min, 11 chapters ≤ 85 s each (plan D-05; chapter 10 was split at the G2 check; the owner may veto). |
| Format | 3840×2160 at 60 fps, 8-bit H.264 High at level 5.2 (x264 CRF 16, preset slow), exact BT.709 colour, AAC 256k, −16 LUFS by one fixed gain on the PCM master under −1.5 dBTP, SRT/WebVTT captions (default) |
| Sound | The score engine with Video C's identity (plan §7.5): B♭ major, whole-tone and augmented tension for races, resolved at `choosing`; 104 BPM with a light swing (56%); B♭maj7 → Gm9 → E♭maj9(♯11) → F7sus4; one timbre per process (felt piano, marimba, pizzicato, FM electric piano), a small-room reverb (T60 0.9 s); motifs "take a number" 1̂–2̂–3̂ and "served" 5̂–4̂–2̂–1̂. Signature sonification: the ticket number is a scale degree, waiting is a soft repeated note, and the customer in the room plays the motif, so only one motif ever sounds at once: mutual exclusion is audible, and a violation sounds like a bug. Narration at 0 dB. |
| Voice | FrankenTTS `robert`, voice-actor cadence, never time-stretched (default) |
| Look | 3Blue1Brown: dark background `#0E1116`, role colours, continuous morphs (default) |
| References | `docs/research/bakery_algorithm/` (the memo and its scripts, including an exhaustive model checker for the bakery algorithm); the owner's own TypeScript explainer (its `(t[j], j) < (t[i], i)` is JavaScript's comma operator, plan §11.1 point 9); plan §11 (arc, beats, facts ledger, engineering notes); `docs/reviews/plan_r1/`. Lamport's papers are cited, not vendored. |
| Owner script supplied? | no (default): written in P3 from the plan's draft words |
| Oracle | `oracle.py` imports `docs/research/bakery_algorithm/scripts/`; long sweeps (the model checker's state counts) are recorded outputs (plan §11.1). |
| Pronunciations | Lamport, Dijkstra (probe hard: only partially verified so far), Dekker, Knuth ("kuh-NOOTH", the K sounded), Peterson, Herlihy, Taubenfeld, Paxos, Anatol Holt, ARM, x86 ("ex eighty-six"), mutex: each verified by the narration skill's probe before narration; the owner's pronunciation overrides. |
| Render host | this Mac with x264 until the GPU host's NVENC is restored (default) |
| franken_manim | git revision `e2eec6326cee490462e497509fce9289c8157bea` through the workspace pin |
| Out of scope | Paxos itself, bounded-bakery constructions, TLA+ |
| Assumptions | The rows marked `(default)`; the arc of plan §11.2. |
