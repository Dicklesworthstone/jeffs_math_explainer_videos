# The One Idea Behind Bridges, Concert Halls and Economies

Video K explains what an eigenvector is and why engineers reach for it
everywhere. A system is a rule that turns one list of numbers into another
(how far each part moves into how hard each part pushes back); a few special
patterns come back from the rule unchanged except for size, and described in
those patterns a tangle of equations falls apart into separate one-line
problems.

- **Two carts and three springs**: the idea found by hand, with "your turn"
  quizzes: both carts together, the carts against each other, and the split
  that turns two tangled equations into two divisions.
- **A steel truss**: 21 members written into one stiffness table, one load
  moving every joint, the table's own patterns from soft to stiff (one of
  them carries 99.6% of the sag), and with mass, each pattern a note. Then
  Tacoma Narrows and London's Millennium Bridge, told from their sources.
- **A room and a concert hall**: air as carts and springs, a tone you can
  test in a bathroom, a hall with two million modes below 1 kHz that blur
  together above 20 Hz, and where single modes still rule.
- **An economy**: Leontief's input-output table, whose largest eigenvalue
  says whether the economy can feed itself; and a factory's linear program,
  shown as the different tool it is.
- **Where the idea stops**: tables with too few own patterns, systems with
  too many modes, and rules that are not tables.

**Status: in production.** The brief, the facts and their oracle, the
storyboard and its first review (R1, every finding disposed in
`docs/reviews/eigen_engineering/R1.md`) are done; `./x eigen_engineering
facts-check` passes. The script is done as far as P3 goes: a claim walk over
every spoken number and always or never sentence (`docs/fact-sheet.md`), and a
cut from 17.6 to 14.98 minutes by the G2 estimate, every cut a row of
`docs/decisions.md`. Review R2, the scenes and the drafts follow. Nothing has
been narrated, scored or rendered at 4K yet.

- `docs/protected.tsv`: the words every revision keeps, checked with the
  skill's `protected_check.py` after every script change.
- `docs/decisions.md`: each choice made so far, why, and where it lives.

```bash
./x eigen_engineering test          # tests, clippy -D warnings, fmt --check
./x eigen_engineering facts-check   # every number against oracle.py
./x eigen_engineering storyboard    # the G2 checks; writes script.tsv
```

- `docs/brief.md`: the owner's idea in his words, and every default chosen.
- `docs/fact-sheet.md`: every number and every quiz, every source with what
  was and was not seen, the edge test behind each always sentence, and what
  the owner's FrankenSim does and does not hold on these subjects.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model/`: the mathematics (a small dense linear algebra with cyclic
  Jacobi eigenpairs, the carts, the truss by the stiffness method, room
  modes counted exactly, the economy in exact fractions, the factory);
  `src/facts.rs`: every number the video shows or says; `oracle.py`: the same
  numbers by other routes (LAPACK, joint equilibrium, a second enumeration).
