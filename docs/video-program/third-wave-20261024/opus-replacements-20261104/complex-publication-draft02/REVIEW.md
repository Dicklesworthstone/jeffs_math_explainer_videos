# Complex publication copy, draft 02

WhiteSwan, for root (Mail 5543). This is public copy only, and nothing was uploaded. `publication-stage06-draft01` is unchanged.

The copy is bound to the final `script.tsv`, sha256 `0bae7acb15f2f0453c8a25ba7110fedd65cf9156eba932143e08db7fdc8bfeee`, 105 cues, 15 chapters.

I checked words against the script and sources. I saw no pictures, heard no audio and had no timeline, so nothing here accepts the film. I wrote this copy, so this isn't an independent review.

## Files

| File | Use |
|---|---|
| `title.txt` | Full title, 80 characters |
| `description.txt` | Full description, with 16 tokens to fill |
| `tags.txt` | 22 tags |
| `chapter-times-pending.json` | The 15 keys, labels and tokens, in the shape of the logarithm `chapter-times02.json`. Every time is null. |

## Decisions for root

1. **Title.** The film's end card reads "The Hidden Geometry of Complex Functions" (`TITLE_CARD`, `src/chapters/closing.rs:39`), and the binary is frozen. I kept that phrase exactly and added a subtitle for the film's range. Dropping the subtitle restores the draft 01 title.
2. **Thumbnail.** `src/chapters/thumbnail.rs` reads "THE BEAUTY OF COMPLEX ANALYSIS" and "A PLANE IN COLOR". Neither phrase matches the title, and "COLOR" differs from the film's on-screen "domain colouring" (`src/chapters/reading_color.rs:429`). I changed nothing. It's your call whether to accept this.
3. **Logarithm link.** Cue 14_6 says "That story has its own video", and draft 01 had no link. If the logarithm film isn't public at upload, replace that line with: "The companion film on logarithms is in production; its link will be added here when published."
4. **Script wording, cue 09_5.** It says the formula "only holds when the function is holomorphic out to the circle and a little beyond". That is stronger than necessary: DLMF §1.9 needs analytic within and continuous on the contour. The cue's point is correct, since poles inside break the formula. The takes are final, so I left it, and the description says "is used for" instead of "only".

## What changed from draft 01

- **Corrected:** "a pair of complex poles can solve a real integral". The bump's loop encloses one pole, at i (13_2). Two poles appear only for 1/(1+x⁴) (13_6).
- **Added depth the film has:** Cauchy's theorem, Liouville's theorem, the fundamental theorem of algebra, conformal maps, the certain root count for programmers, the Riemann hypothesis as unsolved, and π/√2.
- **Added conditions** for the integral formula, identity theorem, Liouville's theorem and argument principle, plus what is proved and what is only stated.
- **Removed "natural boundary".** The script never says it, so the copy states the claim in the script's terms.
- **Credits** now say synthetic voice, native FrankenManim scenes and procedural score. Oliver appears nowhere.
- **Sources** now point at the sections that hold each theorem. Draft 01's §1.10 link alone doesn't contain Cauchy's formula or Liouville's theorem.

## Source support

| Claim | Cue | Source | Result |
|---|---|---|---|
| Cauchy–Riemann equations | 06_5, 06_6 | DLMF §1.9(ii) | Matches |
| Conformal where the derivative is nonzero | 07_2 | §1.9(iv) | Matches |
| Cauchy's theorem | 08_5 | §1.9(iii) | Script's condition is stricter than DLMF's, so it is safe |
| Integral formula and derivatives | 09_1, 09_2, 09_6 | §1.9(iii) | Holds; see decision 4 |
| Series converges on the largest holomorphic disk | 09_7 | §1.10(i) | Matches. DLMF gives radius at least R; the script adds that the pole at i stops it. |
| Identity theorem | 10_1, 10_5 | §1.10(ii) | Matches: an arc, or infinitely many points with a limit point in the domain. DLMF files it under analytic continuation, unnamed. |
| Liouville's theorem | 10_6 | §1.9(iii) | Matches |
| Argument principle | 11_2 to 11_4 | §1.10(iv); Howell and Mathews, Theorem 8.7.2 | Matches: positively oriented simple closed contour, no zero or pole on it |
| One root of z⁵ − 3z + 1 | 11_6 | Rouché, §1.10(iv) | Hypothesis holds, 3 against at most 2 on the circle |
| Fundamental theorem of algebra | 11_7 | Rouché, §1.10(iv) | Argument is sound |
| Residue theorem | 12_2 | §1.10(iv) | Matches. 12_2 doesn't restate the orientation, which 11_2 set just before. |
| At most one extension | 14_3 | §1.10(ii) | Matches |
| No continuation across the unit circle | 14_7 | Wikipedia, Lacunary function | Supported: dense singularities on the circle. DLMF has no statement of it, so this is the weakest source here. The page credits Whittaker and Watson, which I didn't open. |
| Gamma: integral, poles, no zeros, recurrence | 05_6, 14_8 | §5.2, Eq. 5.5.1 | Matches |
| Zeta: series, single pole | 05_7 | §25.2 | Matches |
| ζ(−1) = −1/12 | 14_5 | Eq. 25.6.3 with B₂ = 1/6 | Follows. DLMF doesn't write out the number. |
| Riemann hypothesis, unsolved | 05_8 | §25.10(i); Clay Mathematics Institute | DLMF states it without calling it open. Clay's page labels it "Unsolved". Clay isn't linked in the description, to stay within the limit. |

All 11 source pages were opened on 2026-11-05 through a page summariser. I also read the raw text of DLMF §1.9, §1.10 and §25.10 for the Liouville, identity theorem, argument principle, Rouché and Riemann hypothesis statements. I checked the other pages only through the summary.

## Numbers recomputed

These came from a short pure-Python check, separate from the crate:

| Quantity | Value |
|---|---|
| Series to x⁴⁰ at 1.1 | 25.232 |
| Bump at 1.1 | 0.4525 |
| Distance from 1 to i | 1.414214 |
| Bump's average on radius 2 | 7.6e-18 |
| Turns of z⁵ − 3z + 1 on the unit circle | 1.000000 |
| Area under the bump | 3.141593 |
| Area under 1/(1+x⁴) | 2.221441 |

## Limits checked

YouTube's API page gives title 100 characters, description 5,000 bytes, tags 500 characters with multi-word tags counted in quotes, and no `<` or `>`.

| Item | Measured |
|---|---|
| Title | 80 characters |
| Description as written | 4,761 bytes |
| Description with every time as 9:59:59 and a full YouTube link | 4,818 bytes |
| Tags, quoted | 458 characters |

There is no prose em dash, and no duration or clock time appears. The title has no em dash.

## Still pending

- Fill `[T01]` to `[T15]` from the successful timeline and final master, then seek each one. The usual rule is that the first time is 0:00 and times ascend. YouTube's chapter help page didn't load, so I didn't confirm its other rules.
- Fill `[LOGARITHM_LINK]`, or use the fallback in decision 3.
- Decide on decisions 1 and 2.
- Confirm against final pictures that the copy's claims hold, including "The film stops for questions", which I took from the 13 prediction windows in source.
- Everything else on draft 01's list stands: thumbnail pixels, final picture and technical checks, independent source and AV review, continuous listening, fine sync, upload gates, and public 4K60 and English checks before the original is unlisted.
