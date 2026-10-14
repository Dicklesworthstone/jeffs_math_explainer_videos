# Mental Math Tricks That Actually Work (and Why): YouTube package

What goes on the upload page. The description is cut from `docs/description.md` (whose statements
rest on fact-sheet rows and oracle keys) and links back to it for the full notes; the chapter times
come from the final cut (`./x mental_math chapters`; `tools/youtube_package.py check mental_math
--finals renders/final` holds them to it).

- Title: Mental Math Tricks That Actually Work (and Why) (exactly this: no voice or review suffix)
- Voice: michael. The upload text is `publish/michael/{title,description,tags}.txt`; the description below equals `publish/michael/description.txt` byte for byte.
- Visibility: Private, for the owner's review. Only the upload lane uploads it, and it is made public only on the owner's word.
- Category: Education. Audience: not made for kids.
- Altered or synthetic content: Yes (disclosed: the narration is a synthetic voice and imitates no one; the real people the video names, Feynman and Bethe, are quoted from print, not depicted). The description says so too.
- Thumbnail: `renders/mental_math_thumbnail.png` (`./x mental_math thumbnail`)
- Captions: English, `renders/final/mental_math.en.srt` (`./x mental_math captions`)
- Tags: mental math, mental arithmetic, math tricks, multiplication tricks, squaring numbers, difference of squares, distributive law, area model, Richard Feynman, Hans Bethe, Karatsuba multiplication, math explainer

## Description

```text
A handful of mental arithmetic tricks that come up in real life, each with the picture that makes it true, and thirteen "your turn" quizzes with a countdown. Pause whenever you want more time. A product is the area of a rectangle: every trick here is one way to cut that rectangle into easy pieces, or to trade it for an equal one.

Chapters
0:00 Eighty-five squared
0:58 Squares that end in five
2:05 Your turn
2:54 Any product, by parts
4:14 A running total
5:38 Percents: a tenth, and the swap
6:54 Round numbers: half and double; round, then fix
8:21 The middle
9:38 Feynman, Bethe and forty-eight squared
10:32 Squares near fifty
11:45 Your turn
12:36 Which trick?
13:16 Nothing to grab
14:12 Every number is near something you know

The tricks, exactly
Squares ending in 5: (10n + 5)² = 100 × n × (n + 1) + 25 for every whole number n from 0 up. It is only as easy as that one product.
The picture: slice a strip off the top of a square and stand it against the side. a² = (a − b)(a + b) + b², for every a and every b up to a.
By parts: (a + b)(c + d) = ac + ad + bc + bd, the distributive law. Added in a fixed order, every total on the way is a floor.
Percents: x% of y = x × y ÷ 100 = y% of x, always. It helps only when one of the two is a percent you know as a fraction.
Half and double: (a ÷ 2) × (2b) = a × b; it pays when doubling gives a round number. Round, then fix: n × (r − e) = n × r − n × e; it pays when the fix is small.
The middle: (a + b)(a − b) = a² − b², for any two numbers.
Near fifty: (50 ± b)² = 2500 ± 100b + b². The two-digit reading works for 41 to 59 and for no other number from 0 to 100.

The story of Hans Bethe at Los Alamos is told by Richard Feynman in "Surely You're Joking, Mr. Feynman!": Adventures of a Curious Character (W. W. Norton, 1985), in the chapter "Lucky Numbers", pages 193 and 194. The video quotes two of its sentences.

For programmers: the four pieces of a product by parts are schoolbook multiplication. Karatsuba's algorithm gets both strips from one more multiplication of the digit sums: three multiplications instead of four, a good trade for a computer multiplying very long numbers, a bad one for a head.

Not new: three of these tricks are on one page of an American school book of 1844, Pliny E. Chase, The Elements of Arithmetic, Part Second, page 60.

Every calculation in the video was checked by two independent programs. The exact version of every simplification, every source, and the code that computes each number:
https://github.com/Dicklesworthstone/jeffs_math_explainer_videos/blob/main/explainer_corpus/mental_math/docs/description.md

Names to search: the distributive law; difference of squares; the area model; Karatsuba multiplication.

Narration: synthetic speech (FrankenTTS). Animation: franken_manim, a Rust rewrite of manim.
```

## Chapter titles (in order; the times come from the final cut)

Eighty-five squared | Squares that end in five | Your turn | Any product, by parts | A running total | Percents: a tenth, and the swap | Round numbers: half and double; round, then fix | The middle | Feynman, Bethe and forty-eight squared | Squares near fifty | Your turn | Which trick? | Nothing to grab | Every number is near something you know
