# What Does It Mean to Raise e to a Matrix?

Video J starts from a question that sounds like a typo, e to the power of a
grid of numbers, and answers it with one picture: the exponential turns a
rule for tiny steps into where you end up.

- **One function, three faces.** "Its rate equals its size", the limit of
  compound interest and the power series are the same function; the series
  is earned term by term.
- **Put a matrix in.** The series only needs adding, multiplying and
  dividing by whole numbers, and square matrices can do all three. For the
  quarter turn the series becomes cosine and sine: e to that matrix is a
  rotation.
- **Why it always settles.** The size of the k-th power is at most the size
  to the k, so the factorial wins for every square matrix.
- **What it means.** e^{tA} is where a linked system is after time t: salt
  between two tanks, walks on a network added up, heat and chance spreading,
  and, with one letter i, a quantum system that never settles.
- **How it is computed.** The series is correct mathematics and a poor
  algorithm; real libraries halve the matrix, use a ratio of polynomials,
  and square back up.
- **Where it leads.** e^{A+B} = e^A e^B needs A and B to commute; what is
  left over when they don't is the commutator, and that is the doorway to
  Lie groups and Lie algebras: rotations, the Standard Model's twelve force
  carriers, and (in the companion video) robot arms.

Quizzes with a visible countdown run through the whole video.

**Status: in production.** The brief and the facts with their oracle are
done (`./x matrix_exponential facts-check` passes); the storyboard, script,
scenes and drafts follow. Nothing has been narrated, scored or rendered at
4K yet.

```bash
./x matrix_exponential test          # tests, clippy -D warnings, fmt --check
./x matrix_exponential facts-check   # every number against oracle.py (NumPy and SciPy through uv)
./x matrix_exponential storyboard    # the G2 checks; writes script.tsv
```

- `docs/brief.md`: the owner's request in his words, and every default chosen.
- `docs/fact-sheet.md`: every number, every source with what was and was not
  seen, the simplifications with their exact versions, and the edge test
  behind each always sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model.rs`: the mathematics as the video works it; `src/facts.rs`:
  every number the video shows or says; `oracle.py`: the same numbers by
  exact rational series, closed forms and SciPy's `expm`.
