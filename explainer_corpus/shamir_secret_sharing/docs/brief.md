# Brief: Three Keys Open the Secret. Two Reveal Nothing.

What the video teaches, where its claims come from, what the competition
check did and did not cover, and the words that need a pronunciation check.
Every number on screen or in the narration is a row of `vault facts`
(src/facts.rs), and `oracle.py` checks those rows independently.

## The primary source

Adi Shamir, "How to Share a Secret", *Communications of the ACM* 22(11),
612–613, November 1979. It was read from a scan of the two printed pages
(a public copy at web.mit.edu/6.857/OldStuff/Fall03/ref/Shamir-HowToShareASecret.pdf).
No copy is kept in this repository. The quotes below were transcribed from
the page images, so a transcription slip is possible. Check one against the
printed page before it goes on a quote card.

**The group problem: one keeper, then copies (page 612).** Chapter 01 is
built on this passage:

> "The most secure key management scheme keeps the key in a single,
> well-guarded location (a computer, a human brain, or a safe). This scheme
> is highly unreliable since a single misfortune (a computer breakdown,
> sudden death, or sabotage) can make the information inaccessible. An
> obvious solution is to store multiple copies of the key at different
> locations, but this increases the danger of security breaches (computer
> penetration, betrayal, or human errors)."

**The goal (page 612).** Divide D into pieces D₁, …, Dₙ so that:

> "(1) knowledge of any k or more Dᵢ pieces makes D easily computable;
> (2) knowledge of any k − 1 or fewer Dᵢ pieces leaves D completely
> undetermined (in the sense that all its possible values are equally
> likely)."

Claim (2) is the definition the film demonstrates by counting. With fewer
than three shares, every candidate secret fits, and fits in the same number
of ways (S-F7 in the oracle; S-F8 is the ten-mark dial, where it fails).

**The abstract (page 612)**, quoted in draft line 23_5:

> "even complete knowledge of k − 1 pieces reveals absolutely no
> information about D."

**Liu's cabinet (page 612).** Shamir introduces the problem with "In [4],
Liu considers the following problem:", where [4] is C. L. Liu,
*Introduction to Combinatorial Mathematics* (McGraw-Hill, 1968). The problem
is set as an indented block in smaller type:

> "Eleven scientists are working on a secret project. They wish to lock up
> the documents in a cabinet so that the cabinet can be opened if and only
> if six or more of the scientists are present. What is the smallest number
> of locks needed? What is the smallest number of keys to the locks each
> scientist must carry?"

The counts are Shamir's own next sentence, not part of the problem:

> "It is not hard to show that the minimal solution uses 462 locks and 252
> keys per scientist."

Draft chapter 06 uses both. Its quote card credits the problem to Liu
through Shamir, and its 06_7 card gives the counts as Shamir's words. Liu's
book was not read, so whether Shamir's block is Liu's exact wording is
unknown. The oracle computes 462 = C(11, 5) and 252 = C(10, 5) from one lock
per group of five, then checks the scheme by trying every group against
every lock. The values are pinned to Shamir's sentence as S-X1.shamir.locks
and S-X1.shamir.keys. (Revision 4 corrects the earlier wording here, which
read the counts as part of the quotation. GentleLantern's review found the
slip, and I re-read the page image on 2026-10-24.)

**The construction (page 613).** Shamir picks a random polynomial with
a₀ = D and hands out Dᵢ = q(i):

> "To make this claim more precise, we use modular arithmetic instead of
> real arithmetic." … "The coefficients a₁, …, aₖ₋₁ in q(x) are randomly
> chosen from a uniform distribution over the integers in [0, p), and the
> values D₁, …, Dₙ are computed modulo p."

The secrecy argument, also on page 613. The misspelling "abolutely" is in
the original.

> "For each candidate value D′ in [0, p) he can construct one and only one
> polynomial q′(x) of degree k − 1 such that q′(0) = D′ and q′(i) = Dᵢ for
> the k − 1 given arguments. By construction, these p possible polynomials
> are equally likely, and thus there is abolutely nothing the opponent can
> deduce about the real value of D."

Also on page 613: "the smallest usable value of p is n + 1 (there must be at
least n + 1 distinct arguments in [0, p) to evaluate q(x) at)". Earlier on
the same page, the construction asks for "a prime p which is bigger than
both D and n". Draft line 13_9 rests on both. It says the prime must exceed
the number of trustees and that the secret must be one of the dial's marks.

**The size of each piece (page 613)** is property (1) in the list "Some of
the useful properties of this (k, n) threshold scheme (when compared to the
mechanical locks and keys solutions) are:". Draft line 06_8 shows it as a
small quote:

> "The size of each piece does not exceed the size of the original data."


Shamir also names G. R. Blakley's scheme (AFIPS 1979). The film cites
Blakley and Liu only through Shamir's references, and neither was read
first-hand.

## What the film assumes (also in oracle.py's docstring)

- The secret is one mark of an eleven-mark dial (p = 11, marks 0 to 10).
- Trustees sit at positions 1 to 5. This is Shamir's Dᵢ = q(i) with n = 5
  and k = 3.
- The two spins are fair and independent. Every mark is equally likely,
  zero included, which matches Shamir's uniform draw over [0, p). The worked
  example's spins, 7 and 7, are chosen for the picture. Every privacy claim
  is checked for all 121 spin pairs, not only that one.
- "Opening the vault" means working out the secret from what the trustees
  were handed. A guess still succeeds one time in eleven, and the alarm does
  not make guessing impossible.

  So the film says "two shares cannot determine the secret". It never says
  "two people can never open it" or "it always takes three people agreeing".
  Both of those are stronger than the mathematics.

## What the film does not claim

The film makes no claims about production cryptography. An eleven-mark dial
is a teaching scale. The film says nothing about any library or deployment,
or about practical concerns such as dealer trust, randomness sources, share
authentication or side channels.

Draft chapter 22 follows one false share through four stages:

1. The arithmetic of three shares cannot show a lie, because any three
   numbers fit some curve.
2. The vault's alarm shows that the result is wrong, but not who lied.
3. A fourth share shows a disagreement, but any of the four could be the
   liar.
4. All five shares single out the liar, because only the dealer's curve
   passes through four of them.

That shows a property of the arithmetic for one liar among five, not an
error-correcting protocol. The film claims nothing about more liars.

## Competition check (2026-10-24): search results only, nothing watched

The searches ran on 2026-10-24. No video was opened, watched or listened to,
so nothing below describes any video's content, pacing or quality. Each item
is a search result title as returned.

- **"Secret Sharing Explained Visually"** (IEEE Information Theory Society,
  Great Papers series). The search tool's summary placed it at
  youtube.com/watch?v=iFY5SyY3IMQ. Fetching the ITSoc page
  (itsoc.org/node/34597) returned HTTP 403, so the link and the series
  claim are unverified.
- **Unverified video titles.** "Protecting Your Secrets With Polynomials -
  Shamir's Secret Sharing", "Shamir's Secret Sharing Explained" and "Simple
  introduction to Shamir's Secret Sharing and Lagrange interpolation"
  appeared only as titles on an aggregator page, with no channel or link.
- **Written explainers in the results:**
  - Evervault blog (evervault.com/blog/shamir-secret-sharing)
  - QuillAudits, "Shamir's Secret Sharing Demystified: A Visual Guide"
  - dev.to, "How to share a secret"
  - a Fourmilab Scanalyst forum thread
  - Ledger Academy and Blockstream glossary entries
  - course slides and patents
- **Channel-name search.** A search naming Computerphile, Art of the
  Problem, 3Blue1Brown and Numberphile surfaced no video from them. That
  means only that this one search did not find one.

According to the search summary, at least one written explainer already
argues that integer polynomials leak and that a modulus fixes this, so the
leak alone is not new. The editorial hypothesis, which is not a promise of
views, is that the combination is less common:

- the group problem comes first;
- the dial is taught before any polynomial;
- equal likelihood is shown by counting every candidate secret;
- the attractive wrong explanations are shown failing as controlled
  counterexamples:
  - ordinary cut-up numbers leak;
  - a curve on paper leaks;
  - a ten-mark dial leaks for 9 of its 10 pairs, and one of its ten trios
    can't tell the secret 5 from 0.

Watching the top results is still owed before the first cut.

## Pronunciation risks (unverified; for later qualification)

Chapter 01 says no proper name and no term of art. The full draft uses:

- **Names:** "Shamir" (06_6, 06_8, 23_5) and "Adi" (06_6). Look each one up
  in a source with audio, then audition it against a control. The owner's
  pronunciation overrides every source.
- **Single letters:** "D" and "k minus one" (23_5). TTS may read a lone
  letter as a word.
- **Terms of art:** "parabola" (09_4, 14_2, 14_8), "polynomial" (09_7) and "prime"
  (13_6, 13_7, 13_9). Check the stress.
- **The year:** "nineteen seventy-nine" (06_6) is written out, so it should
  not be read as a cardinal.

No pronunciation has been listened to, and nothing here claims accent
consistency.
