# How to Find a Lost Submarine: YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (every number
there is an oracle key or a row of `docs/description_numbers.tsv`, which
`tools/description_numbers.py` checks) and links back to it for the full notes; the chapter times
come from the final cut (`./x bayesian_search chapters`; `tools/youtube_package.py fill
bayesian_search --finals renders/final` writes them over the `?:??` and checks them).

- Title: How to Find a Lost Submarine (exactly this: no voice or review suffix)
- Voice: michael. The upload text is `publish/michael/{title,description,tags}.txt`; the description below equals `publish/michael/description.txt` byte for byte.
- Visibility: Private, for the owner's review. Only the upload lane uploads it, and it is made public only on the owner's word.
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: Yes (disclosed: the narration is a synthetic voice and imitates no one; the real losses and searches it tells of, the Scorpion, Air France 447, Palomares and John Aldridge's rescue, are told from published accounts and drawn as maps and diagrams, not depicted). The description says so too.
- Thumbnail: `renders/bayesian_search_thumbnail.png` (`./x bayesian_search thumbnail`)
- Captions: English, `renders/final/bayesian_search.en.srt` (`./x bayesian_search captions`)
- Tags: Bayesian search theory, Bayes' rule, probability, search and rescue, USS Scorpion, Air France 447, Palomares, Koopman, search theory, sweep width, SAROPS, Coast Guard, conditional probability, math explainer

## Description

```text
A nuclear submarine has gone missing and time is running out. How do you look for something you cannot see, in an ocean you cannot search all of? The answer is a map of where it could be, a rule for where to look next, and one move that turns "we found nothing" into new knowledge. The video builds that move by counting dots, then shows it at work in four real searches: the hydrogen bomb lost at Palomares, the USS Scorpion, Air France 447 and the Coast Guard's search for a lobsterman who fell overboard.

Five "your turn" quizzes with a countdown. Pause whenever you want more time.

The Scorpion was lost on 22 May 1968 with all 99 men aboard. The Navy searched from 27 May to 5 June as if they might still be saved; they could not be. Nothing in the video says otherwise.

Chapters
?:?? The question
?:?? Ninety-nine
?:?? A map of chances
?:?? Ten thousand dots
?:?? A search can miss
?:?? Nothing found
?:?? Bayes' rule
?:?? Where next
?:?? The second pass
?:?? Adrift
?:?? Twelve hours
?:?? The silence
?:?? How well they looked
?:?? Four maps

The method, exactly
One lost thing, in one of N cells, that stays where it is. Cell i holds it with chance pᵢ, and a search of cell i, if the thing is there, finds it with chance dᵢ, the "chance of seeing it".
The chance of a find from one search of cell i is pᵢdᵢ: the chance it is there times the chance of seeing it.
After an empty search of cell i: pᵢ′ = pᵢ(1 − dᵢ) ÷ (1 − pᵢdᵢ), and pⱼ′ = pⱼ ÷ (1 − pᵢdᵢ) for every other cell j. This is Bayes' rule, in the form Richardson and Stone printed in 1971.
The rule: search the cell with the greatest pᵢdᵢ, after each update. For one thing that does not move, searches that are independent tries with a known, fixed dᵢ, and equal cost per search, no plan has a better chance of having found it after n searches, for every n (Chew 1967).
The update never adds: a cell that holds no dots keeps none, whatever is searched. So the first map must leave room for the truth.

What the video does not say: that Bayes' rule found the Scorpion, or the H-bomb; that mathematicians proved the beacons of Air France 447 were broken; that the software found John Aldridge.

Every number in this video is in the code, and each was checked a second way. The exact version of every simplification, the four searches in their sources' own words, the paper's own numbers for chapter 13, and code you can run:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/bayesian_search/docs/description.md

Names to search: Bayesian search theory; Koopman, Search and Screening; Richardson and Stone 1971; Stone, Theory of Optimal Search; SAROPS.

Sources
H. R. Richardson and L. D. Stone, "Operations Analysis During the Underwater Search for Scorpion", Naval Research Logistics Quarterly 18(2), 1971, pp. 141-157
B. O. Koopman, Search and Screening, OEG Report 56, 1946
L. D. Stone, C. M. Keller, T. M. Kratzke and J. P. Strumpfer, "Search for the Wreckage of Air France Flight AF 447", Statistical Science 29(1), 2014, pp. 69-80
J. R. Frost and L. D. Stone, Review of Search Theory, USCG report CG-D-15-01, 2001
Defense Nuclear Agency, Palomares Summary Report, 1975
Paul Tough, "A Speck in the Sea", The New York Times Magazine, 2 January 2014

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```

## Chapter titles (in order; the times come from the final cut)

The question | Ninety-nine | A map of chances | Ten thousand dots | A search can miss | Nothing found | Bayes' rule | Where next | The second pass | Adrift | Twelve hours | The silence | How well they looked | Four maps
