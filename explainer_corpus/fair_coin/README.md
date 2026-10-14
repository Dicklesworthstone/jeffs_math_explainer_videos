# How to Make a Fair Coin Out of an Unfair One

Video E explains von Neumann's two-toss trick: toss an unfair coin twice,
throw the pair away if it matches, and let heads-then-tails against
tails-then-heads settle the bet. The two are equally likely whatever the
coin's chance of heads, because the same results in a different order always
have the same chance. The video then asks what the trick costs, where it
breaks (two different coins, a coin with memory), how the same idea makes a
fair die from an unfair coin or a loaded die, and how much fairness an unfair
coin holds in the first place.

**Status: scored draft ready for review.** The brief, the facts and their
oracle, the storyboard, the script, all eleven scenes, the narration (`michael`),
and the scored 540p draft are done and verified (`./x fair_coin facts-check`
and `./x fair_coin test` pass, score QA passes on all 11 chapters, and the
cue and join checks pass). Awaiting owner review before the 4K60 master.

```bash
./x fair_coin test          # tests, clippy -D warnings, fmt --check
./x fair_coin facts-check   # every number against oracle.py
```

- `docs/brief.md`: the owner's idea in his words, and every default chosen.
- `docs/fact-sheet.md`: every number, every source with its quoted passages,
  what was and was not verified, and the edge test behind each always/never
  sentence.
- `docs/storyboard.md`: the words and pictures, one row per narration line.
- `src/model.rs`: the mathematics, exact; `src/facts.rs`: every number the
  video shows or says; `oracle.py`: the same numbers, computed independently.
