# Storyboard — __TITLE__

## Arc

hook → why the obvious tool fails → the idea (shown, then named) → mechanism on
a tiny worked example → formula (symbols → numbers) → payoff → limits → credits

## Vocabulary (one word per idea)

| Idea | One word | Avoid |
|---|---|---|
| 4-point subset | quadruple | 4-tuple, tetrad, combination |
| total size | points | vertices, nodes, samples |

## Color roles

| Idea | Role color | Used in |
|---|---|---|
| the hero count | `palette::hero()` gold | live counter, surrounding box |
| the sample size $N$ | `palette::question()` blue | formula spans (`N`, `5000`) |

## Chapters

| Key | Purpose | Est. seconds (≤ 85) |
|---|---|---|
| `01_intro` | Pose the counting question and morph the symbolic formula to numbers | ~15 s |
| `02_outro` | State the oracle-pinned doctrine and render credits | ~12 s |

## 01_intro: The question

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 01_1 | How many ways can you choose four points out of five thousand? ⟨predict⟩ | Title question at top (`y = 2.6`) over the symbolic falling-product formula $C(N, 4) = \frac{N(N-1)(N-2)(N-3)}{24}$ with $N$ highlighted in blue | TeX derivation (`write`, `TransformMatchingTex`: symbols → numbers → answer), highlight (`indicate`) | hold |
| 01_2 | Let us count them. There are more than twenty-six trillion ways. | Formula morphs $N \to 5000$ while a live counter ticks up to $26{,}010{,}428{,}123{,}750$ inside a gold box | live counter (`ValueTracker`), surrounding box, listen window chime | listen 1.5 s |

## 02_outro: How it stays honest

| id | words | picture | features | hold/listen |
|---|---|---|---|---|
| 02_1 | Every number in this video was computed from the model, never typed. What keeps the picture and the voice in step? ⟨predict⟩ | Takeaway banner across center stage | text write | hold |
| 02_2 | It was rendered by franken manim, and narrated by franken T T S. | Credit line fades in below the takeaway | lagged fade-in, hold | hold |

## Showcase checklist (each with the idea it carries)

- [x] TeX derivations (`write`, `TransformMatchingTex`: symbols → numbers → answer): `01_1`, `01_2`
- [x] value trackers driving live numbers and bars: `01_2`
- [ ] morphs that keep identity (clouds, swarms, tables):
- [ ] data windows (plots, histograms, number lines):
- [x] lagged group reveals: `02_2`
- [x] highlights (`indicate`, surrounding boxes, flashes, glow): `01_1`, `01_2`
- [x] sound that teaches (listen windows, the score engine): `01_2`
- [ ] tables and grids built from primitives:
- [ ] the camera route (3D, orbits):
