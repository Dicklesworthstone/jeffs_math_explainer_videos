# Review: shamir_secret_sharing ("Three Keys Open the Secret. Two Reveal Nothing.")

Reviewer: GentleLantern. Bead `mx-video-program-program-87do.46.2`. Date: 2026-10-24.
Subject: the intake snapshot
`narration_lab/third-wave-20261024/root/tend-0610/intake/shamir_secret_sharing/`
(called SNAP below), writer CrimsonGrove. I re-checked it against
`tend-0610/intake.json` (captured 06:19:25Z) at review time: 18 files, 0 hash
mismatches, no unlisted files.

## Verdict

The mathematics is right. I recomputed every number the draft says or shows
with my own methods, and all of them hold. The privacy argument works as
intended:

- it counts every spin pair;
- equal counts are shown to mean "nothing learned";
- four broken assumptions each make my checks fail.

The film keeps guessing, information and working the secret out apart
throughout. The vault, the trustee rail and the strip carry the argument
through chapter 08, before the dealer's recipe is written as a formula (09_2).

There are three teaching-logic problems and one picture contradiction to fix
before chapters 14–18 and the 01_3 beat are built:

1. 18_5's words contradict its own picture and the film's premise.
2. Chapters 14–15 lean on chapters 16 and 17 before those exist.
3. Chapter 16's example never actually needs division on the dial.
4. The native 01_3 shows a bribe recovering a note that was just lost.

The rest are smaller: wording, layout and provenance.

## Findings, most important first

### 1. 18_5 says "nothing looks wrong", but its picture sets off the alarm (fix)

The words are: "If trustee three hands in two instead of one, they compute six,
and nothing looks wrong." The picture cell for 18_5 is: "the dial turns to 6
and the alarm lamp flashes".

The film's premise (01_1) is that a wrong mark sets off the alarm. So in this
story the lie is noticed: what goes undetected is who lied. The arithmetic
itself is right: 8·3 − 3·3 + 2·1 = 17, which lands on 6.

Suggested words: "…they compute six. Nothing in the arithmetic shows the lie.
The alarm does, but it can't say who lied." This also sets up 18_6, where a
fourth share finds the liar's inconsistency without the vault.

### 2. Chapters 14–15 rely on chapters 16 and 17 (fix the argument, not the numbers)

The problem has three parts:

- **14_5 halves before halving exists.** It says "halve it" for the helper
  (x − 2)(x − 3), and 14_6 calls the halved product "a curve of the same kind
  as the dealer's".
- **That curve lives on paper.** Its spins are ½ and −5/2. On the dial, ½
  means "times 6", which chapter 16 introduces only later.
- **15_5 is false on paper.** It says "only one curve of this kind passes
  through three points. So the sum really is the dealer's curve". On paper,
  the weighted sum through the wrapped shares (1, 8), (2, 3), (3, 1) is not the
  dealer's curve. The dealer's paper curve passes through 19, 47 and 89, and
  the weighted sum's value at zero is 16, not 5. Uniqueness on the dial
  (chapter 17) applies only once the helpers are dial curves, which needs the
  division from chapter 16.

The fix needs no new chapter, because the weights 3, −3 and 1 are whole
numbers. Wrapping before the sum or after it lands on the same mark. On paper,
the same weights applied to the true heights give exactly the secret:

    3·19 − 3·47 + 1·89 = 5        (checked: side check "paper sum of true heights")

Chapter 09_4 already told the viewer that three points pin a paper parabola
down. So 15_5 can say: "On paper, the same weights on the unwrapped heights
give exactly five. Wrapping first changes nothing, because the weights are
whole numbers." Chapter 17 then proves uniqueness on the dial, where chapter
16 needs it. The second trio checks the same way on paper: (10/3)·47 − 5·145
+ (8/3)·215 = 5.

### 3. Chapter 16's thirds cancel in this example, so the question "what could dividing by three mean?" has an easy wrong-but-working answer (fix with one sentence)

16_3 asks: "A third isn't a mark on the dial. So what could dividing by three
mean here?"

A viewer who simply keeps the fractions gets the right answer:

- (10/3)·3 = 10, −5·2 = −10, (8/3)·6 = 16;
- the total is 16, which lands on 5.

The thirds cancel because trustees 2 and 5 hold 3 and 6. In fact, for this
dealing, all ten trios' fraction sums are whole numbers, and each lands on 5.

This is not true in general. For the secret 5, 80 of the 121 spin pairs would
leave trio (2, 4, 5) with a fractional sum. One example:

- spins 0 and 1 deal 9, 10 and 8;
- the sum is 4/3, which is no mark;
- on the dial, 4 × 4 = 16, which lands on 5.

Suggested fix: one sentence in 16_3, or a "precisely:" card. For example:
"Here the thirds happen to cancel, because three and six are multiples of
three. With other spins they wouldn't, and the dial still needs an answer."
The 0-and-1 dealing makes a good inset.

### 4. Native 01_3: the bribe appears to recover the lost note (fix in source and storyboard row together)

The lost note and the bribe are alternative failures ("one lost note can shut
the vault … and one bribe can open it"), but the scene plays them one after
the other on the same frame. At `SNAP/src/chapters/the_vault.rs:121-130`:

- the "lost" tag replaces the gold 5 card;
- then `swap(lost → bribed)` puts a "bribed" tag in the same empty frame;
- the strip then shows 5 alone, "for: whoever bribes trustee 1".

The keeper visibly has no card, yet the briber learns the secret.

Suggested fix: reset between the two failures. Bring the gold 5 card back,
with the "bribed" tag beside it or replacing the "lost" tag, then show the
briber's strip. Alternatively, show the bribe on its own frame. The storyboard
row 01_3 and the draft's identical row must change together, because the test
`the_production_chapters_are_the_drafts` compares them.

### 5. 06_6's narration drops "only if" (fix)

The voice says: "Eleven scientists, and any six of them must be able to open a
cabinet." Without "and no five can", a single lock with a key for everyone
would solve the problem. Then 06_7's "four hundred and sixty-two locks" does
not follow.

The quote card in the same beat does say "if and only if", but the narration
should carry the condition too. Suggested: "…any six of them must be able to
open a cabinet, and no five can."

### 6. 17_6's ten-mark failure never touches the secret (consider)

On ten marks, 5x + 5x² is zero at every mark, so adding it changes the spins
without changing any share. Its constant is 0, though, so the secret does not
change either. A viewer can fairly ask "so what?", or wrongly conclude that
three shares cannot open a ten-mark vault.

My count: on ten marks, 9 of the 10 trios still determine the secret. The
opening trio's shares (9, 7, 9) leave only 5.

The one true failure is a stronger example. On ten marks, 5 + 5x² is zero at
positions 1, 3 and 5:

- trustees 1, 3 and 5 hold 9, 9 and 5;
- those shares fit both 0 and 5;
- three trustees therefore cannot tell 5 from 0.

Showing this case makes "on ten marks even this fails" bite on the vault
itself.

### 7. The door's pulse covers the card frames (consider; cheap)

`flash_all(&[vault.door])` (`the_vault.rs:125` and `:181`) uses fmn's
`indicate` at scale 1.25, which grows the door about its centre:

- the bottom edge moves from −0.90 to 0.85 − 1.25·1.75 = −1.34;
- the card frames' top edge is at −1.23;
- so during the 01_3 pulse the door covers the top of trustee 1's frame, which
  holds the "lost" tag, and it does the same during 01_5's pulse.

A scale of 1.08 keeps the bottom at −1.04. Pulsing the lamp or the dial rim
instead would also avoid the overlap. This was computed from the layout
constants and has not been seen rendered.

### 8. 01_4: "One absence" will likely draw late (consider; recheck on narration)

At "a copy", the scene plays three animations one after another:

- door recolour, 0.3 s;
- cards swap in, 0.5 s;
- strip update, 0.6 s.

That is 1.4 s, but "a copy" ends a 24-character sentence. My no-audio
estimate puts the picture 0.7–0.8 s behind "One absence" at 2.4, 2.68 and 3.0
words per second (`run-output.txt`, timeline section). The estimate uses even
character shares and no sentence pause.

Playing the three animations together, in at most 0.6 s, removes most of the
lag. The draft's "thought after 'a copy.'" pause may hide the rest. This is an
estimate, not a measurement.

### 9. Primary-source provenance (low)

I read the two page images of the scan the brief cites (pp. 612–613). The
brief's quotes match the printed text, including "abolutely". There are two
small gaps:

- **06_8's quote card is not recorded in the brief.** It reads "The size of
  each piece does not exceed the size of the original data." It is verbatim:
  page 613, property (1), in the list "Some of the useful properties of this
  (k, n) threshold scheme". Add it to the brief with its page.
- **The 462 and 252 are Shamir's own sentence.** He writes: "It is not hard to
  show that the minimal solution uses 462 locks and 252 keys per scientist."
  Only the problem is quoted from Liu ("In [4], Liu considers the following
  problem"). The brief's quote block and the oracle/facts comments
  (`SHAMIR_LOCKS … quoting Liu 1968`) attribute the numbers to the quotation.
  06_6's spoken "taken from an older book" refers to the problem, so it is
  fine.

### 10. Smaller wording points (low)

- **13_7, the size of the dial.** "Any prime number of marks works, if it's
  more than the number of trustees": Shamir's condition is "a prime p which is
  bigger than both D and n". In the film, the secret is a mark of the dial, so
  this holds implicitly. Still, "For five trustees, seven marks would already
  do" only covers a secret from 0 to 6. Suggested: "…and the secret is one of
  its marks."
- **12_1, "Most dials have ten."** Combination dials often have 40 or 100
  marks. "We count in tens" makes the same point safely.
- **07_3, the trustee/position identity.** The identity is only pictured
  (discs slide onto positions 1 to 5). One spoken clause, such as "trustee
  three sits at position three", would make it hold for a listener too.

### 11. Consistency nits (low)

- **01_6 strip caption.** During 01_6 the strip still reads "wanted, for
  trustees 4 and 5" with all 11 marks lit, while trustees 4 and 5 are tagged
  "away" and the trio opens the door. Switching to "wanted, for trustees 1, 2
  and 3" with 5 lit, or fading the caption, would match the beat.
- **Strip position.** The draft's Vocabulary says the strip sits "under the
  vault", but the Devices sections and the code put it at the right.
- **Stale chapter numbers in comments.** The oracle's comments and fact
  prefixes use an older chapter numbering: "# 12: what it promises" is draft
  chapter 18, and "# 13: the rule again" is 19. `facts.rs`'s header maps them
  correctly.

## What the review confirms

### Mathematics: 40 draft claims recomputed, all equal

| What | Claims | Result |
|---|---|---|
| The five shares and ordinary heights (19, 47, 89, 145, 215 give 8, 3, 1, 2, 6) | 47 = 4·11 + 3 | ✓ |
| Two-of-two (chapters 02, 03) | | ✓ |
| The line, on paper and on the dial | {0, 5, 10}; trustee 2's 11 candidates; the doubles | ✓ |
| Every pair's conditional counts | all 10 pairs × all 121 share views × every secret: one spin pair each | ✓ |
| The pair's (2, 6) under each secret | spin pairs 1:4, 0:9, …; 7:7 for the secret 5 | ✓ |
| Posterior | under a lopsided prior (∝ secret + 1), the pair's posterior equals the prior exactly (05_4's "as likely as before") | ✓ |
| Joiner | trustee 1's values by secret: 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0 | ✓ |
| Paper spins for secret 0 | 37/4 and 27/4; only 5 has whole spins | ✓ |
| Lagrange weights | (3, −3, 1) → (3, 8, 1); 16 → 5. (10/3, −5, 8/3) → (7, 6, 10); 93 → 5 | ✓ |
| Gaussian elimination mod 11 | recovers (5, 7, 7) | ✓ |
| Threshold | all ten trios rebuild 5; a nonzero recipe has at most 2 zeros | ✓ |
| Ten-mark failure | shares 9, 7, 9, 5, 5; pair (4, 5) sees (5, 5) and only 5 fits; 5y₄ − 4y₅ ≡ s for all 1,000 recipes; on eleven marks it is s + 2b, and −14 lands on 8 | ✓ |
| Leaking pairs on ten marks | 9 of 10; the one safe pair is (1, 3), found by brute force and by the criterion "(1, 1) is a value of (a·x₁ + b·x₁², a·x₂ + b·x₂²)" | ✓ |
| Primes | ×5 rows; units 1, 3, 7, 9; smallest prime above five is 7, and on seven marks every pair learns nothing | ✓ |
| Padlocks | any lock every trio can open stops at most one pair (all 32 key sets tried), so 10 locks are needed; 6 keys each; 462 and 252 | ✓ |
| Liar | 17 → 6, predicts 5 at trustee 4, who holds 2 | ✓ |
| Four of seven | det[x, x², x³] ≠ 0 mod 11 for all 35 triples of positions 1–7; 11 curves | ✓ |

### Random-coefficient assumptions

Fair, independent spins over 0–10, with zero allowed: every pair learns
nothing. Each of these broken assumptions leaks for the pair (4, 5):

- refusing b = 0 (the draft's 18_3 case: 8 is crossed off, 10 secrets are
  left, each fitting one of the 110 spin pairs);
- refusing a = 0;
- spinning a ten-mark spare dial (0–9);
- tying the first spin to the secret.

The draft's "free to land anywhere, zero included" (18_2) covers the first
three cases.

### Guessing, information and reconstruction stay distinct

| Concept | Where the draft uses it |
|---|---|
| Opening = working the secret out | defined once (01_6) |
| Guessing (1 in 11) | 04_3, 05_4, 18_1, 18_7 |
| Information ("not one mark ruled out") | 04_4, 05_4, 10_6 |

The film never claims that two trustees "can never open it". 18_4 is an
example of information without reconstruction: one secret is crossed off, and
none is determined. 12_2 and 13_2 separate "work out alone" from "two possible
secrets" correctly.

### Picture before symbols

The privacy result is shown by counting on the strip and the spin grid in
chapter 05. That is four chapters before the dealer's recipe becomes a
formula (09_2), and nine before the first helper product (14_5).

Before 09_2 the only symbols are:

- dial arithmetic, such as 03_2's "9 + 7 = 16 → 5";
- 08_5's word formula "share = secret + 2 × spin";
- two "precisely:" cards with a conditional probability (03_6, 08_8). Both
  repeat what the strip has already shown.

The identities hold throughout:

- A and B (chapters 02–03) get their own small door, separate from trustees
  1–5;
- the five trustees keep their numbers and become points on the boards;
- the strip's caption always names whose knowledge it shows.

### Native source and API (chapter 01)

Read against the current `crates/explainer_kit` (working tree) and the
committed crates that use the same calls. These helpers are byte-identical to
randomized_response's committed `draw.rs`:

- `think` and `Spoken`;
- `fade_in_all`, `fade_in_lagged`, `fade_out_all`, `flash_all`, `swap`;
- `pause`, `wait_until`, `read_by`;
- `flat`, `line_style`, `fail`.

Other calls match committed patterns:

- `recolor` matches resistor_random_walk's committed helper;
- `bundle` matches the kit's `group_of`;
- `Kit::live_number` adds its readout to the scene itself, so dropping its
  return value is fine (the count shows 0 until its tween);
- `precisely` returns a detached group, which the scene fades in;
- `Timed` is used through `.rt()`.

The registry holds only `01_the_vault`, as Mail 6790 directs.

### Picture/model binding

- The strip's ticks and count come from `View::possible`.
- The two "wanted" views are labelled as goals and use the trivial `Copy` and
  `Outsider` views, which is honest for chapter 01.
- Every number the scene draws comes from the model's constants. The facts
  test ties those constants to `facts.rs`, which the oracle mirrors (21 keys
  compared with mine, 0 differ).

### Completed timeline and protected question hold

- All 16 cue phrases are found in their lines.
- The named hold after 01_2 (`wait_end` + 1.0 s) is in place.
- The ⟨predict⟩ at 01_6 waits for the line to end (`kit.hold`). The ring then
  drains for 4.0 s, with 0.2 s fades either side, followed by 0.5 s and a
  0.6 s fade before the chapter ends. Nothing cuts the hold.
- `tools/storyboard.py` budgets 2.0 + 2.5 = 4.5 s for a ⟨predict⟩ "question
  hold" row, which matches `think`'s 4.4 s.
- In the full draft, 19 of 20 questions are ⟨predict⟩ rows with think time.
  The one without (17_2) is marked rhetorical.
- My chapter-01 estimate (84.2 s at 2.68 words/s) agrees with the author's
  84.5 s.

### Production matches the draft

- `storyboard.md`'s six 01 rows equal the draft's.
- `script.tsv` equals the draft script's 01 lines.
- The uncompiled `SPOKEN` test, replayed in Python against the oracle's facts,
  passes: 14 rows, every number word covered.

## Exact check scope

All runs were from the repository root, Python 3 standard library only,
`-I -B`, stderr not redirected away.

1. `python3 -I -B docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/check_shamir.py --oracle narration_lab/third-wave-20261024/root/tend-0610/intake/shamir_secret_sharing/oracle.py`
   - **Result:** exit 0, "ALL HOLD"; full output in
     `reviews/shamir-GentleLantern/run-output.txt` (154 lines).
   - **Claims:** 40 draft claims, all equal.
   - **Negative controls:** each must fail at named claims, and does:

     | Control | Claims failed | Fails at the named claims |
     |---|---|---|
     | Ten-mark dial | 31 | 05_3, 05_4, 11_6 |
     | Spins 7 and 6 | 12 | 04_1/09_6, 10_1 |
     | Refuse b = 0 | 3 | 05_3, 05_4, 11_6 |
     | Ten-mark spare dial | 4 | 05_3, 05_4, 11_6 |

   - **Side checks:** 30 asserted, all hold; 7 informational rows.
   - **Timeline:** the chapter-01 estimate.
   - **Author's oracle, run unchanged:** exit 0; its stderr reads "self-test:
     ten-mark dial pairs that leak: 9 of 10" and "152 facts, 0 failed
     cross-checks". 21 of its facts were compared with mine, and 0 differ.
   - **Source consistency:** three checks, all ok.
2. `python3 -I -B docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/control_spoken.py`
   - **Result:** exit 0; 3 of 3 deliberate in-memory edits caught (an
     uncovered number word, a missing SPOKEN phrase, production drifting from
     the draft); output in `control-output.txt`.
3. **Snapshot hashes** against `intake.json`: 18 of 18 match, no unlisted
   files.

### How my methods differ from the author's

| Step | Author's oracle | Author's model | This review |
|---|---|---|---|
| Division on the dial | a search | extended Euclid | Fermat's little theorem |
| Two shares | search over spins | | closed-form Cramer's rule |
| Counting | loops over share values, then spins | | tallies all p³ recipes by the shares they deal |
| Reconstruction | | | Lagrange's product formula plus Gaussian elimination mod p |
| Lock bound | | | an argument tried on every key set |

## Not verified

- **Compilation.** No Rust was compiled. Per the assignment, I did not run
  Cargo, rustc or clippy-driver. `main.rs`, `the_vault.rs`, `draw.rs`,
  `palette.rs` and `score.rs` remain uncompiled, and their tests have never
  run. Matching call shapes against committed crates does not replace
  compiling.
- **Rendering and audio.** Nothing was rendered, synthesized, watched or
  listened to. Layout findings (7) come from constants, and timing findings
  (8) are estimates. Nothing here shows rendered clarity, natural pacing or
  accent acceptance.
- **Primary sources.**
  - Shamir 1979, pp. 612–613: read as page images from a one-off download of
    the scan the brief cites. The scan has no text layer, so I compared it by
    eye. No copy is in the repository.
  - Liu (1968) and Blakley (1979): not read; known only through Shamir's text
    and references.
- **Pronunciation.** The brief's risks were not auditioned: "Shamir", "Adi",
  the letters "D" and "k", "parabola", "polynomial", "prime" and "nineteen
  seventy-nine".
- **Competition.** No competing video was watched. The brief's competition
  section is search results only, and says so.
- **Peer reviews.** Other writers' reviews were not read.

## Files

- `docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern.md`
  (this review)
- `docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/check_shamir.py`
- `docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/control_spoken.py`
- `docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/run-output.txt`
- `docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern/control-output.txt`

No Shamir or shared file was modified.
