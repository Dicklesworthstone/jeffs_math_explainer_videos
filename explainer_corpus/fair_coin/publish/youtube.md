# How to Make a Fair Coin Out of an Unfair One: YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (whose statements
rest on fact-sheet rows and oracle keys) and links back to it for the full notes; the chapter times
come from the final cut (`./x fair_coin chapters`; `tools/youtube_package.py check fair_coin
--finals renders/final` holds them to it).

- Title: How to Make a Fair Coin Out of an Unfair One (exactly this: no voice or review suffix)
- Voice: michael. The upload text is `publish/michael/{title,description,tags}.txt`; the description below equals `publish/michael/description.txt` byte for byte.
- Visibility: Private, for the owner's review. Only the upload lane uploads it, and it is made public only on the owner's word.
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: Yes (disclosed: the narration is a synthetic voice and imitates no one; the real people named, John von Neumann and Persi Diaconis, are quoted from print, not depicted). The description says so too.
- Thumbnail: `renders/fair_coin_thumbnail.png` (`./x fair_coin thumbnail`)
- Captions: English, `renders/final/fair_coin.en.srt` (`./x fair_coin captions`)
- Tags: fair coin, unfair coin, biased coin, von Neumann coin, probability, coin flipping, Bernoulli trials, random bits, information theory, John von Neumann, loaded dice, math explainer

## Description

```text
Toss an unfair coin twice. If the two results match, throw them away and toss two fresh ones. If they differ, heads-then-tails wins for one player and tails-then-heads for the other. The game is exactly fair whatever the coin's chance of heads, and nobody has to know that chance. This video shows why, what it costs, how it breaks, and how the same idea makes a fair die out of a loaded one.

Chapters
?:?? Call it
?:?? Toss twice
?:?? Every coin
?:?? The price
?:?? Where it breaks
?:?? Same counts
?:?? A die from a coin
?:?? Loaded dice
?:?? Six orders
?:?? What is left
?:?? The fair coin inside

The trick, exactly
Von Neumann's trick: toss twice. HT and TH each have probability p(1 − p). Because multiplication is commutative, the two outcomes have identical probability for any bias p between 0 and 1.
The price: for an 80% coin, matches occur 68% of the time, so on average it takes 6.25 tosses (or 3.125 pairs) to get one decision. At p = 0.5, it takes 4 tosses.
Where it breaks: if the coin has memory (Markov dependence) or if two different coins are used, the probabilities diverge and the game is no longer fair.
Loaded dice: grouping outcomes by frequency counts allows extracting unbiased uniform choices from biased independent trials.

John von Neumann introduced the method in "Various techniques used in connection with random digits", Applied Mathematics Series, 1951.

For programmers: von Neumann's procedure extracts unbiased random bits from any independent identically distributed Bernoulli source. Advanced algorithms like Peres (1992) and Knuth-Yao achieve optimal entropy extraction rates.

Every calculation in the video was checked by two independent programs. The exact version of every simplification, every source, and the code that computes each number:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/fair_coin/docs/description.md

Names to search: von Neumann coin; Bernoulli process; randomness extractor; fair decision from biased coin.

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```
