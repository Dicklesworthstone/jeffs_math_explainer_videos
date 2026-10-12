# How to Count Fish You've Never Seen: YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (whose
numbers are oracle keys and fact-sheet rows) and links back to it for the full notes; the
chapter times come from the final cut (`./x capture_recapture chapters`).

- Title: How to Count Fish You've Never Seen
- Visibility: unlisted until the owner has listened to it, then public
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: No (a generic synthetic narrator; no real person, place or event is depicted). The description says the narration is synthetic.
- Thumbnail: `renders/capture_recapture_thumbnail.png` (`./x capture_recapture thumbnail`)
- Captions: English, `renders/final/capture_recapture.en.srt` (`./x capture_recapture captions`)
- Tags: capture-recapture, mark and recapture, statistics, probability, estimation, maximum likelihood, hypergeometric distribution, confidence interval, Lincoln-Petersen estimator, William Feller, Laplace, multiple systems estimation, math explainer

## Description

```text
Mark a thousand fish, let them go, and catch a thousand more: the hundred caught twice tell you roughly how many fish no net ever touched. Starting from William Feller's 1950 textbook puzzle, this video builds the four-cell table, the exact best guess, a 95% range that keeps its promise, the ways the method fails, and its uses from Laplace's France to software bugs and the war dead of Kosovo.

Chapters
0:00 The fish nobody caught
0:58 The marks measure the net
2:14 Why the marks spread
3:38 Every possible netful
5:00 Turning it around
6:24 Same mountains, two questions
7:00 Is the score still rising?
8:20 The table balances
9:22 Ten thousand lakes
10:31 Dividing by zero
11:31 Turning it around again
12:27 A range that keeps its promise
13:41 One rule for every failure
15:04 France and the bugs
16:28 Kosovo, and the coda

Names to search: capture-recapture (mark-recapture), closed population, the hypergeometric distribution, maximum likelihood, the Lincoln-Petersen estimator, Chapman's estimator, confidence intervals by test inversion, multiple systems estimation.

The exact version of every simplification, every source, and the code that computes each number and checks it against an independent oracle:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/capture_recapture/docs/description.md

Sources
W. Feller, An Introduction to Probability Theory and Its Applications, Vol. I, 1st ed. (1950), ch. 2 §5, Example (e), pp. 37–38
Schnabel, "The Estimation of Total Fish Population of a Lake", The American Mathematical Monthly 45(6), 1938
Goudie and Goudie, J. R. Statist. Soc. A 170(3):825–839, 2007, on how the estimator grew from the "fishing coefficient"
Laplace, Essai philosophique sur les probabilités (1840 ed.), pp. 82–83: https://www.gutenberg.org/ebooks/58881
H. D. Mills, "On the Statistical Validation of Computer Programs" (1970)
Ball et al., Killings and Refugee Flow in Kosovo, March–June 1999, a report to the ICTY (2002): https://www.icty.org/x/file/About/OTP/War_Demographics/en/s_milosevic_kosovo_020103.pdf
Spagat, A Triumph of Remembering: Kosovo Memory Book (2014)

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```

## Chapter titles (in order; the times come from the final cut)

The fish nobody caught | The marks measure the net | Why the marks spread | Every possible netful |
Turning it around | Same mountains, two questions | Is the score still rising? | The table balances |
Ten thousand lakes | Dividing by zero | Turning it around again | A range that keeps its promise |
One rule for every failure | France and the bugs | Kosovo, and the coda
