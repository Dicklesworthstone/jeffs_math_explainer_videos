# Mental Math Tricks That Actually Work (and Why)

Video F teaches a handful of mental arithmetic tricks that come up in real
life, and shows why each one works as a picture: a product is the area of a
rectangle, and every trick is one way to cut that rectangle into easy pieces
or trade it for an equal one.

- **Squares of numbers ending in 5** (the centrepiece): the number in front,
  times the next number, then 25. Slice a strip off the top of the square
  and stand it against the side, and the square of 35 is a rectangle 30 by
  40 with a 5 by 5 square sticking up.
- **Any two-digit product by parts**, kept as a running total so only one
  number is held between steps.
- **Everyday tricks**: percents from a tenth, x% of y = y% of x, halving and
  doubling, and round then fix for prices that end in 99.
- **Two numbers the same distance from a middle**: 47 × 53 is 50² − 3², the
  same move run backwards.
- **Squares near 50**, with the story Richard Feynman tells of Hans Bethe
  squaring 48 at Los Alamos: the same move again.

The viewer gets 14 "your turn" quizzes, each with a visible countdown and
the answer worked through. In the last four the first job is to pick the
trick: one is a product that no trick helps with, where paper is the right
tool, and the last is a square that two tricks agree on.

**Status: scored draft ready for review.** The brief, the facts and their
oracle, the storyboard, the script, all fourteen scenes, the narration
(`michael`), and the scored 540p draft (`936 s`) are done and verified
(`./x mental_math facts-check` and `./x mental_math test` pass, score QA passes
on all 14 chapters, and the quiz-timing and join checks pass). Awaiting owner
review before the 4K60 master.

```bash
./x mental_math test          # tests, clippy -D warnings, fmt --check
./x mental_math facts-check   # every number against oracle.py
./x mental_math storyboard    # the G2 checks; writes script.tsv
./x mental_math draft         # 540p, scratch voice, layout lint, into renders/draft/
./x mental_math sheets        # contact sheets of those drafts, to read
```

- `docs/brief.md`: the owner's idea in his words, and every default chosen.
- `docs/fact-sheet.md`: every number and every quiz, every source with what
  was and was not seen, and the edge test behind each always sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `docs/claims.md`: the claim walk of the script and what it changed.
- `docs/description.md`: the exact form of every trick, for the video's
  description.
- `lexicon.json`: the spoken names, probed in the video's voice.
- `src/model.rs`: each trick as the video works it; `src/facts.rs`: every
  number the video shows or says; `oracle.py`: the same numbers by plain
  arithmetic and by counting unit squares.
- `src/chapters/`: one scene per chapter on the shared drawing kit
  `draw.rs` (the strip that turns, the cut rectangle, the quiz card and its
  countdown, the six small pictures) and the teaching sounds of `sound.rs`.
