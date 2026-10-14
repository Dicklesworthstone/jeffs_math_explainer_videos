# __TITLE__

A 3Blue1Brown-style explainer in the `jeffs_math_explainer_videos` corpus, rendered by **franken_manim** (`fmn`, native Rust and native TeX) and narrated by **FrankenTTS** (`ftts`).

## What this video explains

Starting from a concrete counting question — how many ways to choose 4 points out of 5,000 — this explainer traces the falling product over $4!$, morphs symbols into concrete numbers, and verifies every spoken and displayed value against an independent oracle.

## Chapters

| Key | Title | Purpose |
|---|---|---|
| `01_intro` | The question | Count quadruples among 5,000 points and derive $C(N, 4)$ |
| `02_outro` | How it stays honest | Oracle-checked facts and native rendering |

## Reproduce & verify

```bash
./x __SLUG__ facts-check
./x __SLUG__ storyboard
./x __SLUG__ test
./x __SLUG__ lint
./x __SLUG__ probe
./x __SLUG__ draft
```

## How it was made

| Stage | Detail |
|---|---|
| Scene engine | `franken_manim` (`fmn`, native TeX, retained route) |
| Shared kit | `explainer_kit` + `explainer_score` |
| Narration | FrankenTTS (`ftts`, voice `michael`), verified with FrankenWhisper (`fw`) |
| Oracle | `oracle.py` (`./x __SLUG__ facts-check`) |
| Final target | 3840×2160 @ 60 fps, BT.709, −16 LUFS ±1, ≤ −1.0 dBTP |
