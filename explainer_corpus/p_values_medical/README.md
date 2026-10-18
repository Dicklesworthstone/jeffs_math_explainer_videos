# Why P-Values Confuse Even Doctors

A survey published in 2007 (run in 2006) found 88% of 277 internal-medicine
residents in Connecticut at least fairly confident about p-values; about 59% picked the answer
marked correct, and a later paper argued that answer is wrong too. This
video makes the trap visible by counting: a positive result from a test
that falsely alarms five percent of people without the disease, counted out
of ten thousand people, means the disease about fifteen percent of the
time; then a treatment study's imaginary world, every one of its 2^20 label
assignments listed, whose whole tail is the p-value.

**Status (2026-10-18):** chapters 01 (`01_five_percent_of_what`), 02
(`02_the_same_test`) and 03 (`03_the_imaginary_world`) are code the
orchestrator accepted as silent drafts (e6a446e, a201b4a and b41aa80: tests,
strict clippy and fmt, facts parity, glyph, cue, layout and frame review;
82.2 s and 74.4 s for chapters 02 and 03). Chapter 04
(`04_count_the_whole_tail`: the tail counted in assignments to 21,700 of
1,048,576, then named the one-sided p-value; the exact and strict tails as
other subsets) is written as code, about 80 s at the kit's scratch rate, and
awaits the same central checks. Nothing has been narrated; no audio is
accepted. The rest (chapters 05 and 06) is written out in
[`docs/storyboard.md`](docs/storyboard.md) §2, and its numbers are already
computed and oracle-checked.

| File | What it holds |
|---|---|
| `src/model.rs` | the mathematics: exact fractions, clinics counted person by person, the study's null world listed in full, the tails |
| `src/facts.rs` | every number the video shows or says, pinned to its source |
| `oracle.py` | the same numbers computed independently (standard library, and NumPy for the seeded re-tosses) |
| `src/chapters/five_percent.rs` | chapter 01's scene |
| `src/chapters/same_test.rs` | chapter 02's scene |
| `src/chapters/imaginary_world.rs` | chapter 03's scene, and its last picture rebuilt for chapter 04 |
| `src/chapters/count_tail.rs` | chapter 04's scene |
| `docs/storyboard.md` | the words and pictures (the source of `script.tsv`) |
| `docs/fact-sheet.md` | every number's source, what was read, competing explanations |
| `docs/brief.md` | the brief |

```bash
uv run --no-project --with numpy python3 -I -B oracle.py   # every number, and 53 cross-checks on stderr
python3 -I ../../tools/storyboard.py --check p_values_medical
./x p_values_medical facts-check              # the Rust facts against the oracle (builds)
```
