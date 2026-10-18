# Brief — Two Bad Echoes Can Make One Perfect Measurement

Values I chose without the owner are marked `(default)`.

| Field | Value |
|---|---|
| Topic (owner's words) | "Two bad echoes can make one perfect measurement" (the first-wave Golay selection, `docs/video-program/research-20261016/SLATE.md`; catalog `golay_echoes`) |
| One-sentence takeaway | Any single pattern of up and down pulses leaves ghost echoes, but two patterns chosen as a pair have ghosts of equal size and opposite sign, so adding their two measurements of the same still scene leaves only the real echoes. |
| The promise | In the first 20 seconds: a measurement that is wrong in a precise way, and the surprise that a second wrong measurement fixes it exactly. It matters to anyone who measures by echoes (radar, sonar, a room's sound): the fix is free arithmetic, as long as the scene holds still. |
| Vocabulary (one word per idea) | pulse, pattern, send, echo, copy, line up, agree/disagree, block, score, shift/step, ghost, row (storyboard, "Vocabulary") |
| Protected facts | `docs/protected.tsv` |
| The hook (where the obvious tool fails) | The obvious receiver, sliding a copy and adding products (a matched filter), reports a score where nothing is: a ghost a quarter the size of the echo. No single pattern of more than one pulse avoids this (`G-X1`). |
| The one picture | The score row as columns of blocks: an agreement is a green block above the line, a disagreement a red block that cancels a green one or hangs below. Adding two rows is dropping one onto the other. |
| Worked example | Pattern A = up, up, up, down; pattern B = up, up, down, up (the length-four pair in Wikipedia's *Complementary sequences* and the slate's toy check). A scores −1, 0, 1, 4, 1, 0, −1; B scores 1, 0, −1, 4, −1, 0, 1; added, 0, 0, 0, 8, 0, 0, 0. |
| Audience | a curious 14-year-old with school algebra; a programmer who could implement it afterwards; a mathematician who finds nothing wrong (default, plan §1.2) |
| Length target | chapter 01 under 85 s: 83.9 s estimated with thought gaps, the four-second countdown, the 2 s hold and the ending (words at 2.68 a second, a rate that already includes pauses, so likely long); the narrated draft measures it. Later chapters are planned in the storyboard's arc, not yet written |
| Format | 3840×2160 @ 60 fps, H.264, AAC 256k, −16 LUFS target with a −1.0 dBTP ceiling; quieter when the ceiling binds, never loudnormed (default) |
| Voice | FrankenTTS `michael` with the owner's English accent. The latest Michael sample (a 25 s Golay opening in `narration_lab/michael-coherent-20261017`) is pending owner listening; no voice approval is inferred here. |
| Look | 3Blue1Brown: dark background, role colors, continuous motion (default) |
| References | Chi, Pezeshki and Calderbank, *Complementary Waveforms for Sidelobe Suppression and Radar Polarimetry* (book chapter, PDF at users.ece.cmu.edu/~yuejiec/papers/Golaybkch_final.pdf, read 2026-10-17: definition 1, eqs. 1, 2, 7–9, 33, 40; its Doppler discussion). Wikipedia, *Complementary sequences* (read through a fetch summary 2026-10-17: the length-four example and its sum). Purdue lecture notes, M. Bell, Session 37 (engineering.purdue.edu/~mrb/resources/LecturesF/Session_37.pdf, text extracted 2026-10-17: matched-filter outputs added with delay; "used ... where there is not significant motion between measurements"). Golay's 1949/1951/1961 papers: **not read** (cited by the chapter only). |
| Competing explanations | Two web searches on 2026-10-17 ("Golay complementary sequences explained video sidelobe cancellation"; "youtube Golay complementary pair autocorrelation animation") returned papers, lecture notes, Wikipedia and Jedwab's research page, and **no video**. YouTube itself was not searched directly; no video was watched. Whether a close animated explanation exists is **unknown**. The written explanations found state the definition and the radar use; none shown the per-product block picture this chapter uses (seen: Wikipedia summary, Purdue notes, the chapter). |
| Owner script supplied? | no |
| Oracle(s) | `oracle.py`: every score row by polynomial products (each pair of pulses adds to shift i − j), complementarity also by the unit-circle identity, exhaustive pair counts and the cancellation on every small scene, numpy.correlate parity when numpy is present (`./x golay_echoes facts-check`). |
| Pronunciations | chapter 01 says no name and no term of art (`lexicon.json` is empty on purpose); "Golay" will need lookup and probing before any chapter says it |
| Render host | the build host (`project.env`); SageBasin runs the combined build, tests and bounded draft for the first wave |
| franken_manim | the workspace pin (`Cargo.toml`, `FRANKEN_MANIM.pin`); not compiled by this lane |
| Out of scope | Doppler-resilient pulse trains (Prouhet–Thue–Morse ordering), polarimetry, polyphase codes, and any claim about where Golay pairs are deployed today: none was verified from a primary source for this chapter |
| Assumptions | (1) "Perfect" means only the ideal noise-free measurement of one still scene, sampled one value per pulse; the 01_10 lower-third says so. (2) The scene moving one step between the sends is the chapter's counterexample; the general statement kept in the model is that what remains is the change, scored by pattern A. (3) Pulses are drawn as up/down bars; in radar they are phase flips (default). |
