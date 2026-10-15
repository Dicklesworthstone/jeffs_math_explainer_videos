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
0:00 Call it
0:57 Toss twice
2:18 Every coin
3:39 The price
4:33 Where it breaks
5:42 Same counts
7:04 A die from a coin
7:57 Loaded dice
8:44 Six orders
9:53 What is left
11:00 The fair coin inside

The exact versions
The 80% coin is a model. A coin that is flipped and caught cannot be made to favour a side by weighting it (Gelman and Nolan 2002); a coin that is spun, or allowed to bounce, can.
For two independent tosses with the same probability p of heads, P(heads then tails) = p(1 − p) = P(tails then heads), for every p in [0, 1]. A coin that cannot land both ways never produces a differing pair. Independence with one shared p is enough, not necessary: all the trick uses is that swapping the two tosses of a pair leaves the pair's probability unchanged.
A pair differs with probability 2p(1 − p), so the number of pairs is geometric and the expected number of tosses is 1/(p(1 − p)): 6.25 at p = 4/5, 4 at p = 1/2, about 101 at p = 0.99. No rule that always names a winner within a fixed number n of tosses is fair for every coin: at p near 1 the all-heads pattern has nearly all the probability, and it belongs to one player.
With chances p₁ then p₂, the two differing pairs have probabilities p₁(1 − p₂) and (1 − p₁)p₂, equal exactly when p₁ = p₂.
For a coin that remembers only its last toss, the tosses that come straight after a head are independent with one shared chance, so the trick applied to those alone is exactly fair when played to the end (Samuelson 1968, as described by Elias 1972, who credits the best form of it to John W. Pratt; Samuelson's own pages were not seen for this video).
For independent tosses of one coin, a pattern's probability depends only on how many heads it has: p^k (1 − p)^(n−k). So, given the counts, every arrangement is equally likely. For independent rolls of one die, the n! orders of n different values are equally likely, whatever the die's chances.
The trick's long-run yield is p(1 − p) fair tosses per toss: 0.16 at p = 4/5 and 0.25 for a fair coin, the "at most 25 percent" of the 1951 summary. The ceiling is the binary entropy h(p) = −p log₂ p − (1 − p) log₂(1 − p), 0.7219 at p = 4/5: no rule whose output is exactly fair can average more per toss over a long run (Elias 1972, Theorem 1). Elias's rule on blocks of N tosses and Peres's iterated procedure both approach the ceiling and never reach it.

For programmers
Fresh tosses for every pair. A sliding window that keeps the second toss of a thrown-away pair and compares it with the next one returns plain heads-or-tails: the unfair coin, unchanged.
It is rejection sampling: a retry loop with no upper bound. If you cap the loop, the cap must return "no result", never a default winner.
Coins with memory, using every toss (Blum). Treat "after a head" and "after a tail" as two coins and run the trick on each. A result must be announced only when the coin is next in the state that produced it; announcing it at once is wrong.

Every count and chance in the video is computed by the video's own code and checked against an independent script. The exact version of every simplification, the programmer notes in full, the seeds of every simulated run, and every source:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/fair_coin/docs/description.md

Names to search: von Neumann's trick (the von Neumann extractor), randomness extraction, rejection sampling, the binary entropy function, Elias's algorithm, Peres's iterated procedure, Blum's algorithm for Markov chains, composition classes, Pascal's triangle.

Sources
John von Neumann, "Various Techniques Used in Connection With Random Digits" (1951)
Wassily Hoeffding and Gordon Simons, "Unbiased Coin Tossing with a Biased Coin" (1970)
Peter Elias, "The Efficient Construction of an Unbiased Random Sequence" (1972)
Manuel Blum, "Independent Unbiased Coin Flips From a Correlated Biased Source" (1986)
Yuval Peres, "Iterating von Neumann's Procedure for Extracting Random Bits" (1992)
Andrew Gelman and Deborah Nolan, "You Can Load a Die, But You Can't Bias a Coin" (2002)
Persi Diaconis, Susan Holmes and Richard Montgomery, "Dynamical Bias in the Coin Toss" (2007)
František Bartoš and others, "Fair Coins Tend to Land on the Same Side They Started" (2025)
Benjamin Jun and Paul Kocher, "The Intel Random Number Generator" (1999)

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```
