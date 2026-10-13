# Mental Math Tricks That Actually Work (and Why)

Video F teaches a handful of mental arithmetic tricks that come up in real
life, and shows why each one works as a picture: a product is the area of a
rectangle, and every trick is one way to cut that rectangle into easy pieces
or trade it for an equal one.

- **Squares of numbers ending in 5** (the centrepiece): the number in front,
  times the next number, then 25. A square whose two thin strips slide
  together shows where "the next number" comes from.
- **Any two-digit product by parts**, kept as a running total so only one
  number is ever held.
- **Two numbers the same distance from a middle**: 47 × 53 is 50² − 3².
- **Squares near 50**, with the story Richard Feynman tells of Hans Bethe
  squaring 48 at Los Alamos.
- **Everyday tricks**: percents from the ten-percent block, x% of y = y% of
  x, halving and doubling, and round then fix for prices that end in 99.

The viewer gets 22 "your turn" quizzes, each with a visible countdown and
the answer worked through, a mixed round where the first job is to pick the
trick, and one product that no trick helps with, where paper is the right
tool.

**Status: in production.** The brief, the facts and their oracle and the
storyboard are done (`./x mental_math facts-check` passes); the script,
scenes and drafts follow. Nothing has been narrated, scored or rendered at
4K yet.

```bash
./x mental_math test          # tests, clippy -D warnings, fmt --check
./x mental_math facts-check   # every number against oracle.py
./x mental_math storyboard    # the G2 checks; writes script.tsv
```

- `docs/brief.md`: the owner's idea in his words, and every default chosen.
- `docs/fact-sheet.md`: every number and every quiz, every source with what
  was and was not seen, and the edge test behind each always sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model.rs`: each trick as the video works it; `src/facts.rs`: every
  number the video shows or says; `oracle.py`: the same numbers by plain
  arithmetic and by counting unit squares.
