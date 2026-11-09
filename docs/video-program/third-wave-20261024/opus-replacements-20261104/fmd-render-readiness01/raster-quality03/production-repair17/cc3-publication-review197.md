# cc3 publication review 197: complex analysis copy

Dispatch-3 (cc3), 2026-11-08 UTC. **Suggestions only.** Nothing was uploaded or published, and no packaging file was edited. `publication19-draft01/` and `publication19-draft02/` are unchanged. This file is the only file written.

The start of the request was cut off, and Agent Mail has nothing about it. I took it to mean: review the complex film's current publication copy (`publication19-draft02`: title, description, chapters, tags) and give public-ready replacement text. If a different film or file was meant, nothing here applies to it.

## What I checked, and what I did not

Checked:

- **Script.** `script.tsv` sha256 `234f7257…3d3c` equals the `source_script_sha256` in draft01's review. Every claim in the suggested copy below was checked against cue text in that script (cue ids are given beside each claim).
- **Timestamps.** I recomputed all 15 chapter starts from `publication19-draft02/review.json`:
  - The start frames are contiguous.
  - At 60 fps, every start rounds down to the timestamp printed in the draft.
  - The total is 133,662 frames, 37:07.7.
  - The repaired 08/09 timelines keep 14,959 and 16,504 frames (root's timeline165 receipt, quoted in `bead-recovery-final04-198.json`), so the repairs move no timestamp.
- **Limits.** The draft description is 1,595 bytes against YouTube's 5,000. The draft tags are 230 characters against the 500 limit.
- **DLMF.** On 2026-11-08 I fetched the section headings of DLMF §1.9 and §1.10 (results under "Sources" below).
- **Logarithm link.** YouTube oEmbed returns `lgpTDROHgQY` as public, titled "Logarithms Made Visual: From Slide Rules to Matrices", on the same channel.
- **Music.** The film's own binary writes the score (`score all` in `score22.py`), so the logarithm upload's credit wording ("procedural score generated in code") also applies here.

Not checked:

- The final master. It does not exist yet, and the timestamps must be re-read from it once it is joined.
- The thumbnail, the captions, and the pictures in the film.
- Listening, of any kind.

## Findings on draft02

1. **High: chapter 08's label contradicts the chapter.**
   - The label "Why closed-loop integrals cancel" is wrong for the chapter's most important loop integral: 08_3 shows that the loop integral of 1/z is 2πi, which does not cancel.
   - 08_4 then says that this one fact "powers most of what follows".
   - The rest of the chapter is the proof of Cauchy's theorem for squares and triangles (08_5).
   - Suggested label: "Loop integrals and Cauchy's theorem".
2. **High: the description says "We build the proofs as pictures", which overclaims.**
   - Only two arguments are carried out step by step:
     - Cauchy's theorem for squares and triangles, by nested squares (08_5).
     - Cauchy's integral formula for a disk (09_1).
   - Several other results are stated and illustrated, not proved:
     - the identity theorem (10_1);
     - Liouville's theorem (10_6);
     - the argument principle (11_2);
     - the expansion near a pole (12_1);
     - the convergence of the power series (09_7).
   - The public logarithm description already says which results are proved and which are only stated; this one should do the same.
3. **Medium: the description does not link the logarithm film that the narration points to.**
   - 14_6 ends "That story has its own video."
   - That film is public, so the description should link https://www.youtube.com/watch?v=lgpTDROHgQY.
4. **Medium: the narration credit does not say the voice is synthetic.** The public logarithm description says "Narration: ElevenLabs Oliver, a synthetic voice." For consistency and honesty, this one should read "Narration: ElevenLabs Frederick Surrey, a synthetic voice." It should also add the music line used for the logarithm film.
5. **Medium: chapter 07's label covers only its first two cues.**
   - "What happens when the derivative is zero" is 07_1–07_2.
   - The rest of the chapter, 07_3–07_6, is a function with a derivative at exactly one point and the definition of holomorphic, which every later claim depends on.
   - Suggested label: "Critical points and the meaning of holomorphic".
6. **Medium: one source is cited, and it is not the one for chapters 06 to 10.**
   - DLMF §1.10 covers Taylor series, analytic continuation, the residue theorem and the argument principle.
   - The Cauchy–Riemann equations, Cauchy's theorem, the integral formula, Liouville's theorem and winding numbers are in §1.9.
   - Cite both sections.
7. **Low: generic or vague lines.**
   - The title's "Changes Everything" says nothing specific.
   - "A smooth curve can hide a surprise" and "no visible trouble" leave out the film's own concrete hook. In 01_3, the series up to x⁴⁰ gives about 25 at x = 1.1, where the curve is below 1/2.
   - "We also examine where these tools stop working" can name the three limits that chapter 14 actually shows (14_5, 14_6, 14_7).
   - The description also leaves out the gamma and zeta functions and the Riemann hypothesis (05_6–05_8), and the root-counting proof of the fundamental theorem of algebra (11_7). Viewers search for all of these.
8. **Low: chapter 13 computes two areas.** It finds π (13_2) and π/√2 (13_9), so "A real integral" undercounts. Suggested label: "Two real integrals through the complex plane".
9. **Low: tag names.**
   - "Goursat theorem" is usually searched as "Cauchy-Goursat theorem". The narration never says "Goursat", so the description introduces the name as "Goursat's argument".
   - Missing tags: Cauchy-Riemann equations, contour integral, Riemann zeta function, Riemann hypothesis, gamma function, fundamental theorem of algebra, Liouville's theorem, conformal map, radius of convergence.

The narration uses British spelling ("colour", "centre"). The draft description uses US spelling, as the logarithm description does; I kept US spelling.

## Suggested public copy

The timestamps are draft02's, unchanged. They are correct for the current timelines and must be re-read from the final master.

### Title

Recommended. It uses the logarithm film's naming pattern and states the film's hook (68 characters):

```
Complex Analysis Made Visual: Why a Smooth Curve's Series Stops at 1
```

Alternative, if the series pattern matters more than the hook (62 characters):

```
Complex Analysis Made Visual: From Domain Coloring to Residues
```

### Description

```
Why does the Taylor series of 1/(1 + x²) stop working just past x = 1? The curve has no corners or gaps, and it never blows up. Yet at x = 1.1, the series up to x⁴⁰ gives about 25, while the curve itself is below 1/2. The reason is not on the real line at all.

We picture complex functions in two ways: as maps from one plane to another, and as colored plots of the input plane. We use them on polynomials, 1/z, the exponential, sine, tangent, the gamma function and the Riemann zeta function. Then we ask what a complex derivative demands: the same answer from every direction. That demand sends tiny squares to tiny squares and gives the Cauchy-Riemann equations. A derivative at a single point buys almost nothing. A derivative throughout an open region, which makes the function holomorphic, buys a great deal.

Two proofs are drawn step by step. For Cauchy's theorem on squares and triangles (Goursat's argument), shared edges cancel and nested squares shrink to a point. For Cauchy's integral formula on a disk, a ring of triangles carries the integral from a large circle to a small one. The formula answers the opening puzzle.

From there, the argument principle counts zeros and poles without finding them, which gives a proof that every nonconstant polynomial has a root. The residue theorem gives two real areas, π and π/√2. Analytic continuation carries a function past the disk where its series works, and chapter 14 shows where it stops: why 1 + 2 + 3 + ... does not equal -1/12, why the square root comes back as -1 after one trip around zero, and a series that cannot be continued past its circle at all.

The film assumes first-year calculus: derivatives, integrals and power series. It introduces complex numbers from the start, with questions to work through along the way. Cauchy's theorem and Cauchy's integral formula are proved step by step. The identity theorem, Liouville's theorem and the argument principle are stated and illustrated rather than proved.

The complex logarithm, which comes up in chapter 14, has its own film, Logarithms Made Visual: From Slide Rules to Matrices:
https://www.youtube.com/watch?v=lgpTDROHgQY

Chapters:
0:00 A smooth curve whose series stops at 1
1:45 Multiplication as turning and stretching
3:14 Watching a function move the plane
5:15 How to read the colors
7:08 A gallery of complex functions
9:53 A derivative from every direction
12:40 Critical points and the meaning of holomorphic
14:28 Loop integrals and Cauchy's theorem
18:37 How the boundary determines the inside
23:12 The rigidity of holomorphic functions
25:16 Counting zeros and poles
28:35 What a residue measures
29:41 Two real integrals through the complex plane
32:38 Continuation and its limits
35:54 Returning to the real line

Sources and further reading:
NIST Digital Library of Mathematical Functions, §1.9 Calculus of a Complex Variable (Cauchy-Riemann equations, Cauchy's theorem and integral formula, Liouville's theorem, winding number):
https://dlmf.nist.gov/1.9
NIST Digital Library of Mathematical Functions, §1.10 Functions of a Complex Variable (Taylor series, analytic continuation, residue theorem, argument principle):
https://dlmf.nist.gov/1.10

Visuals: FrankenManim.
Narration: ElevenLabs Frederick Surrey, a synthetic voice.
Music: procedural score generated in code.
```

Where each claim in the description comes from:

| Claim | Cue |
|---|---|
| The series fails just past 1; about 25 at x = 1.1, curve below 1/2 | 01_3 |
| "The reason is not on the real line at all" | 01_5 |
| Two planes; colored input plane | 03_1, 04_1 |
| The gallery of functions | 03_5–03_6, 05_1–05_7 |
| Same answer from every direction; squares to squares; Cauchy-Riemann | 06_2, 06_3, 06_6 |
| One point buys almost nothing; holomorphic on an open region | 07_5, 07_6 |
| Shared edges cancel; nested squares; triangles | 08_5 |
| Ring of triangles; large circle equals small circle | 09_1 |
| The formula answers the opening puzzle | 09_7 |
| Counting without finding; every polynomial has a root | 11_7, 11_8 |
| π and π/√2 | 13_2, 13_9 |
| -1/12 is ζ(-1); the sum diverges | 14_5 |
| The square root returns as -1 | 14_6 |
| The series that cannot cross its circle | 14_7 |
| Questions along the way | 02_5, 03_3, 04_6, 06_8, 07_3, 09_4, 09_8, 10_4, 11_5, 13_5, 14_4, 15_3 |

The two areas were also recomputed for this review:

- The degree-40 partial sum at 1.1 is (1 + 1.21²¹)/2.21 ≈ 25.2, and the curve is 1/2.21 ≈ 0.452 there.
- π/√2 ≈ 2.221, which matches 13_9's "about two point two two".

### Tags

```
complex analysis, complex numbers, complex derivative, holomorphic functions, Cauchy-Riemann equations, conformal map, domain coloring, contour integral, Cauchy-Goursat theorem, Cauchy integral formula, Liouville's theorem, radius of convergence, Taylor series, argument principle, winding number, fundamental theorem of algebra, residue theorem, analytic continuation, gamma function, Riemann zeta function, Riemann hypothesis, visual mathematics, math explainer
```

## Before root uses this

- Re-read the 15 chapter starts from the joined master. YouTube needs 0:00 first, at least three chapters, and each at least 10 s; the shortest here is chapter 12, at 65.7 s.
- Check the lengths again after any edit to the copy (the counts are in the closing section below).
- Choose the title. The thumbnail has not been reviewed, and the recommended title assumes the thumbnail shows the bump or its series.
- Root owns the upload and every packaging file. This review changes neither.

## Measured lengths of the suggested copy

Measured from the code blocks above with `python3 -I -B`:

| Item | Measurement | Limit |
|---|---|---|
| Title A | 68 characters | 100 |
| Title B | 62 characters | 100 |
| Description | 3,318 characters (3,329 bytes) | 5,000 |
| Tags | 23 tags, 463 characters as written | 500 |
| Tags, counted YouTube's way (quotes around each tag that has a space, plus commas) | 487 | 500 |

- **Tags are near the limit.** Only 13 characters remain, so any tag added must replace one. "math explainer" or "conformal map" are the cheapest to drop.
- **Timestamps.** The 15 timestamps are identical to draft02's.
- **Characters.** The suggested title, description and tags contain no em dash and no angle bracket. YouTube refuses angle brackets in descriptions.
- **Drafts untouched.** draft01 and draft02 still carry their Nov 7 11:48:49 and 12:13:39 local times. draft02 sha256 values:
  - description `2570d86c…574e`
  - review `ab320ba3…8019`
  - tags `17a7dea0…d51d`
  - title `32d01232…2570`
