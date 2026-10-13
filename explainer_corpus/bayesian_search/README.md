# How to Find a Lost Submarine

Video H explains Bayesian search: how to look for one lost thing in a sea
too big to sweep. The owner's question opens it: "A nuclear sub has gone
missing, men are trapped, and time is running out. How can we possibly find
them in time?" The video then tells the truth about the USS *Scorpion* (all
99 men were lost in May 1968, and nothing here suggests they could have been
saved) and follows the method that her search made famous to the places
where hours do matter.

- **A map of chances.** The sea is cut into cells; stories of what might
  have happened, each with a weight, scatter dots over the cells; a cell's
  chance is its share of the dots. The 1968 map was exactly that: 10,000
  simulated submarines, counted by cell.
- **A search can miss.** The chance of a find is the chance it is there
  times the chance of seeing it, so the likeliest cell is not always the
  one to search.
- **Bayes' rule after an empty search.** The dots the search would have
  found go out; the searched cell's chance falls and every other cell's
  rises. Worked by hand on a sea of four cells and a hundred dots.
- **Where to look next.** Where the next search is most likely to find it;
  no order of searches does better. Why a second pass over the same water
  buys less than the first (Koopman's random search formula).
- **A target that drifts.** The map moves and spreads, and the same flight
  buys less every hour.
- **Four stories, each against its map:** the H-bomb off Palomares (1966),
  the *Scorpion* (1968), Air France 447 (found in 2011), and the US Coast
  Guard's searches for people in the water.

The viewer gets six "your turn" quizzes, each with a visible countdown.

**Status: in production.** The brief, the facts and their oracle are done
(`./x bayesian_search facts-check` passes); the storyboard, script, scenes
and drafts follow. Nothing has been narrated, scored or rendered at 4K yet.

```bash
./x bayesian_search test          # tests, clippy -D warnings, fmt --check
./x bayesian_search facts-check   # every number against oracle.py
./x bayesian_search storyboard    # the G2 checks; writes script.tsv
```

- `docs/brief.md`: the owner's request in his words, and every default chosen.
- `docs/fact-sheet.md`: every number and every quiz, every source with what
  was and was not seen, what the popular accounts get wrong, and what the
  video therefore does not say.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model.rs`: the four-cell sea, the 1968 map as Richardson and Stone
  printed it, Koopman's formula with seeded lanes, and a seeded drifting
  cloud; `src/facts.rs`: every number the video shows or says; `oracle.py`:
  the same numbers by counting dots, a second transcription of the map, and
  NumPy on the same seeds.

The sources are cited in the fact sheet and are not in this repository.
