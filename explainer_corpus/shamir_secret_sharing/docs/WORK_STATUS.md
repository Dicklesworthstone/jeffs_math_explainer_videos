# shamir_secret_sharing: work status

Owner: CrimsonGrove (NTM pane %14, session `jeffs_math_explainer_videos--third-wave-20261024`).
Bead: `mx-video-program-program-87do.46.1` (root BlueSalmon owns Beads; this lane
does not write Beads and does not close its own work).
Agent Mail reservation 7536: `explainer_corpus/shamir_secret_sharing/**`, exclusive.

## Acknowledgment (2026-10-24, intake)

Assignment received and read: AGENTS.md including the 2026-10-24 amendment,
docs/video-program/README.md, plan §0, §1, §3, §4 and §20, the
third-wave README, MOR_CASE.md, SELECTION.md and TODO.md, the catalog entry
(`shamir_secret_sharing`, inbox, no revisions), and the four named skills
(end-to-end explainer, franken-manim mega skill, natural-sounding narration,
de-slopify), with modes-of-reasoning and idea-wizard used only as bounded
reasoning aids. The randomized_response crate was read as the code pattern.
No other writer's proposal was read.

Scope kept: only this directory. No root files, other topics, Beads, Cargo
builds, synthesis, renders, commits, pushes, account changes, extra agents or
pane operations.

## Workspace note for root

The root manifest's `members = ["crates/*", "explainer_corpus/*"]` makes any
directory under `explainer_corpus/` a member, and cargo fails for the whole
workspace when such a directory has no `Cargo.toml` (checked with
`cargo metadata --no-deps --offline` on a toy workspace in my scratch
directory, not on this repository). The directory was therefore created in
one step together with its `Cargo.toml` (same shape as freivalds and
randomized_response: workspace dependencies only, bin `vault`). The root
`Cargo.toml` and `Cargo.lock` were not touched; the next cargo run will add
this package to the lock. The Rust here is uncompiled until root's serialized
pass. If root prefers no uncompiled member in the glob meanwhile, an
`exclude` entry in the root manifest (root's file) would do it.

Root has since added this directory to the root `[workspace].exclude`
(Mail 6769), so no Cargo command builds it until root removes that entry at
integration.

## Handoff (2026-10-24, batch_pending, not self-closed)

This first handoff is superseded by "Review repair (revision 4)" at the end of
this file. Its file list and check results describe the source as it stood
before the repair, which is kept at
`docs/previous/source-before-review-repair-20261024/`.

Direction consumed: Mail 6778 (more setup and visual intuition) and Mail
6790 (production registry holds implemented chapters only), plus root's TODO
line for this lane. Root verifies and decides the bead's state.

### Files (all under explainer_corpus/shamir_secret_sharing/)

- `docs/brief.md`: the primary source with quotes transcribed from the
  printed pages of Shamir 1979, the assumptions, the scope of claims, the
  competition check (search results only, nothing watched) and the
  pronunciation risks.
- `docs/storyboard-draft.md` and `docs/script-draft.tsv`: the full draft, 19
  chapters and 121 lines. Each row has words, picture, fact tags and its
  pause intention.
- `docs/storyboard.md` and `script.tsv`: production, chapter 01 only (6 lines).
  Its rows are identical to the draft's chapter 01.
- `oracle.py`: the independent facts oracle (standard library only). It
  prints 152 facts and has a `--self-test` mode with edge tests and negative
  controls.
- `src/model.rs` and `src/facts.rs`: the native model and the 152 facts.
- `src/chapters/the_vault.rs`, `src/chapters/draw.rs`,
  `src/chapters/glyphs.rs` and `src/chapters/mod.rs`: chapter 01 and its
  devices (vault, trustee rail, possible-secrets strip). The registry holds
  only `01_the_vault`.
- `src/main.rs`, `src/palette.rs` and `src/score.rs`: the entry point, the
  colour roles and the music identity.
- `lexicon.json`: empty. Chapter 01 says no proper name.

Older versions are kept in `docs/previous/`. That directory is gitignored,
so these copies exist on this machine only.

### Checks actually run, and their results

- **Oracle.** `python3 -I -B oracle.py --self-test`: 152 facts, 0 failed
  cross-checks, and ten-mark dial pairs that leak: 9 of 10. The script was
  last changed after that run, but only in its docstring, and it still
  parses.
- **Rust model and facts.** `model.rs` and `facts.rs`, plus the kit's real
  `facts.rs`, were compiled with plain `rustc --test` using the pinned
  nightly, outside Cargo:
  - 19 passed, 0 failed.
  - `clippy-driver -D warnings` in test mode: rc 0.
  - The facts binary's output is byte-identical to the oracle's stdout
    (`cmp` rc 0, 152 lines).
  - A scratch mutation (spins 7,6) makes the facts test fail.
- **Formatting.** `rustfmt --edition 2024 --check src/main.rs`, which covers
  the whole crate: rc 0.
- **Storyboard and script.**
  - `python3 -I tools/storyboard.py shamir_secret_sharing --check`: rc 0.
    Chapter 01 estimates 84.5 s over 6 beats.
  - `tools/lint_script.py script.tsv`: 6 lines, 0 errors, 0 warnings.
  - The full draft passes the same estimator: all 19 chapters are at most
    85 s, 23.1 min in total.
- **Spoken numbers.** A Python check of main.rs's SPOKEN table against
  script.tsv and the oracle's facts: 14 rows, every number word covered, no
  problems.
- **Cue phrases.** A Python check of the 16 phrases chapter 01 waits for:
  each is present once in its line, in order.

### Not verified

- **Uncompiled files.** `main.rs`, `the_vault.rs`, `draw.rs`, `palette.rs`
  and `score.rs` need fmn, so they have never been compiled. Their calls were
  checked by hand against the fmn pin and the kit's signatures. Their tests,
  including the production-equals-draft test, have never run.
- **No output.** Nothing has been rendered, synthesized or listened to.
  Written plans and passing arithmetic do not show rendered clarity, natural
  pacing or English-accent acceptance.
- **Pronunciation.** Pronunciations are unverified (docs/brief.md).
- **Competition.** No competing video was watched.

### Incident: a write outside this lane

At about 06:11Z I ran `tools/storyboard.py --help` to see its options. The
script has no help flag: without `--check` it regenerates `script.tsv` for
every topic with a `docs/storyboard.md`, and it did so for all 23.

- **Tracked topics:** `git diff` and `git status` show no change, so those
  files are byte-identical to the index.
- **Untracked peer topics:** the files rewritten were
  `explainer_corpus/{crofton,german_tank,inspection_paradox,parrondo}/script.tsv`,
  each now regenerated from its own current `docs/storyboard.md`. There is
  no copy of their earlier content, and no local snapshot exists, so I
  cannot show they are unchanged. Each owner should run
  `tools/storyboard.py <slug>` or `--check` and compare with what they
  intended.

Root and the four owners have been told by Agent Mail.

## Review repair (revision 4, 2026-10-24, batch_pending, not self-closed)

Direction: root's Mail 6834, on GentleLantern's review
(`docs/video-program/third-wave-20261024/reviews/shamir-GentleLantern.md`,
Mail 6833). The source as it stood before this repair is copied, with
timestamps, to `docs/previous/source-before-review-repair-20261024/`. That
directory is gitignored, so the copy exists on this machine only. The source
is now frozen for root's native qualification.

### What changed, by review finding

1. **18_5's alarm contradiction (fix).** The new chapter 22, "a liar", has
   seven rows:
   - 22_1: the lie gives 17, which lands on 6.
   - 22_2: nothing in the arithmetic shows the lie, because any three
     numbers fit some curve. The alarm does show it, but cannot say who lied.
   - 22_3: a fourth share disagrees (it should be 5, and is 2); no curve
     fits all four.
   - 22_4: any of the four could be the liar (secrets 8, 3, 5, 6).
   - 22_5: a ⟨predict⟩.
   - 22_6: with five shares, exactly one curve meets four of them. It is the
     dealer's (secret 5), and trustee 3 is off it.
2. **Chapters 14–15 depended on 16–17 (fix).**
   - The helpers are now built on paper, from the heights 19, 47, 89. Halving
     is fine on paper, and 09_4's "three points pin a parabola" makes the
     weighted sum the dealer's paper curve.
   - 15_2: 3·19 − 3·47 + 89 = 57 − 141 + 89 = 5.
   - 15_3: a ⟨predict⟩ on the wrapped shares.
   - 15_4: 24 − 9 + 1 = 16 → 5.
   - 15_5 and 15_6: 19 = 8 + 1·11, 47 = 3 + 4·11, 89 = 1 + 8·11. Whole
     weights keep the two sums whole elevens apart (16 − 5 = 11), so
     wrapping first changes nothing.
   - Division and dial uniqueness come later (17 to 19).
3. **The thirds cancelled (fix).**
   - 16_4 says the cancellation is luck, since 3 and 6 are multiples of 3.
   - 17_1: the same secret with spins 0 and 1 gives heights 9, 21, 30 and
     shares 9, 10, 8, and the fraction sum is 4/3.
   - 17_2: 80 of the 121 spin pairs give a fraction.
   - 17_3: the whole weights 10, −15, 8 give 90 − 150 + 64 = 4.
   - 17_4: a ⟨predict⟩.
   - 17_5: 3·5 = 15 → 4, and 5 is the only such mark (row × 3).
   - 18_1: the real shares give 48 → 4, so 5.
   - 18_2 and 18_3: partners, ÷ 3 = × 4, only after that.
4. **01_3: the bribe recovered the lost note (fix).** At "one bribe" the
   keeper's card "5" returns, red, in place of the "lost" tag. The door and
   the strip change in the same 0.6 s move. The "bribed" tag is gone; the
   caption still names the briber.
5. **06_6 (fix).** It now says "Any six of eleven scientists must be able to
   open a cabinet, and no five can."
6. **A ten-mark failure that moves the secret (consider; done).** Chapter
   20:
   - The line 5 + 5x is 10, 20, 30 at trustees 1, 3, 5, so it is zero there.
     It is zero at every odd mark.
   - Added to the dealer's recipe, it gives 0 + 2x + 7x². That deals the
     same 9, 9, 5, so the trio cannot tell 5 from 0.
   - 9 of the 10 trios still pin the secret.

   The review suggested 5 + 5x². Both work. I used the line because the
   model's and oracle's searches return the lowest-degree recipe, and it ties
   to 19_4's "a line has at most one zero" (on ten marks it has five).
7. **The door pulse covered the frames (consider; done).**
   - `Vault::pulse` uses `DOOR_PULSE = 1.08`. The door's bottom moves to
     −1.04, against the frames' top at −1.23, and its top to 2.74, against
     the lamp's bottom at 2.84.
   - Two compile-time asserts in `the_layout_keeps_its_pieces_apart` keep
     those margins.
   - `flash_all` keeps 1.25 for everything else.
8. **01_4's late "One absence" (consider; done).** The door recolour, the
   card swap and the strip update are one `Strip::show_with` move of 0.6 s,
   not 1.4 s in sequence. This is an estimate; nothing was measured.
9. **Provenance (low).** In `docs/brief.md`:
   - The counts 462 and 252 are now attributed to Shamir's own sentence
     ("It is not hard to show that the minimal solution uses 462 locks and
     252 keys per scientist").
   - The problem is credited to Liu through Shamir.
   - 06_8's quote is recorded (p. 613, property (1)).
   - The condition "a prime p which is bigger than both D and n" is added.

   I re-read all four from the page images of the scan on 2026-10-24. The
   oracle, facts and draft comments match.
10. **Smaller wording (low).**
    - 13_7 now says "The secret also has to be one of its marks."
    - 12_1 now says "We count in tens."
    - 07_3 now says "Trustee one sits at position one, trustee two at
      position two, and so on."
11. **Consistency (low).**
    - 01_6's strip shows "wanted, for trustees 1, 2 and 3" with 5 lit while
      4 and 5 are away, then returns to "for: anyone outside".
    - The vocabulary puts the strip at the right.
    - Every chapter-number comment in `oracle.py`, `model.rs` and `facts.rs`
      uses the draft's 23-chapter numbering.

**Length.** No intuition was cut. The 21-chapter version had four chapters
over 85 s, and three of those were split where the idea changes:

- fractions (16);
- thirds that don't cancel (17);
- partners (18);
- any three (19);
- the ten-mark line (20).

Chapter 06 lost two words instead ("Eleven scientists, and any six of them"
became "Any six of eleven scientists"). The full draft is now 23 chapters,
27.6 min.

### Files changed (all under explainer_corpus/shamir_secret_sharing/)

- `oracle.py`: 38 new facts and 2 removed (`S-F11.ten.always_zero` and
  `.zeros`), so 188 in all. All other values are unchanged. It has new
  self-tests and renumbered comments.
- `src/model.rs`:
  - new: `OTHER_SPINS`, `Ratio::{minus, whole}`,
    `Recipe::{coefficients, plus}`, `most_zeros`, `ghost`, `paper_terms`,
    `paper_sum`, `cleared_weights`, `undo` and `curves_through`;
  - `ordinary_sum` now delegates to `paper_sum`;
  - `vanishing_spins` removed, since nothing uses it;
  - three new tests: `whole_weights_commute_with_wrapping_and_thirds_do_not`,
    `ghosts_exist_only_on_ten` and `one_liar_among_five_is_found`;
  - `at_most_two_zeros_on_eleven` extended.
- `src/facts.rs`: the new facts, in the oracle's order. The embedded
  `ORACLE` is replaced by the oracle's current 188-line stdout, and the test
  expects `f.len() == 188`. The header's chapter mapping and the Shamir
  attribution are fixed.
- `src/chapters/draw.rs`: `DOOR_PULSE`, `Vault::pulse`, `pulse_all`,
  `recolorings`, `swaps` and `Strip::show_with`. `flash_all`, `recolor_all`,
  `swap` and `Strip::show` keep their behaviour and now delegate to these.
- `src/chapters/the_vault.rs`: findings 4, 7, 8 and 11, `wanted_trio`, and
  the layout tests.
- `docs/storyboard-draft.md`: revision 4, 23 chapters.
- `docs/script-draft.tsv`: regenerated, 137 lines.
- `docs/storyboard.md`: the picture cells of 01_3, 01_4 and 01_6, identical
  to the draft's, and the chapter count.
- `docs/brief.md`: provenance, the liar's scope, line references and
  pronunciation rows.
- `docs/WORK_STATUS.md`: this section.

Unchanged: `script.tsv` (chapter 01's words did not change), `src/main.rs`,
`lexicon.json`, `src/palette.rs`, `src/score.rs`, `src/chapters/glyphs.rs`
and `src/chapters/mod.rs`.

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

- **Oracle.**
  - `oracle.py --self-test`: rc 0. It prints "# self-test: ten-mark dial
    pairs that leak: 9 of 10" and "# 188 facts, 0 failed cross-checks".
  - The plain run gives rc 0 and 188 stdout lines, byte-identical to
    `facts.rs`'s embedded `ORACLE`.
  - The new cross-checks:
    - for all 1,331 recipes, the paper weights on the heights give the
      secret, and the whole weights on the shares wrap to it;
    - negative control: the fractional weights on the shares are not always
      whole;
    - for every recipe, liar and lie among five shares, exactly the dealer's
      curve fits four;
    - four shares with a lie fit no curve;
    - a nonzero line has 5 zeros on ten marks and 1 on eleven.
- **Storyboard.**
  - Production: `tools/storyboard.py shamir_secret_sharing --check`, rc 0,
    01 at 84.5 s. Its words are unchanged, so `script.tsv` is still current.
  - The draft, through a copy of the tool in my scratch directory: rc 0, 23
    chapters, each at most 85 s, each with a ⟨predict⟩. `docs/script-draft.tsv`
    equals that run's output.
- **Lint.** `tools/lint_script.py`:
  - production: 6 lines, 0 errors, 0 warnings;
  - draft: 137 lines, 0 errors, 0 warnings, after recasting the two colons
    and the 32-word sentence it flagged.
- **Production and draft agree.** The draft's 01 section equals
  production's, compared byte for byte in Python. All 158 bracketed fact
  keys in the two storyboards exist in the oracle's output.
- **Spoken numbers and cues.**
  - The SPOKEN replay: 14 rows, 1 idiom, 40 number words, no problems.
  - The cue phrases: the scene waits for 16, the same list as its test, each
    once and in order in its line.
- **Formatting.** `rustfmt --edition 2024 --check` over the crate (from
  `src/main.rs`): rc 0, after `rustfmt` rewrote layout in `model.rs`,
  `facts.rs` and `the_vault.rs`. rustfmt parses every module, so there are
  no syntax errors. It does not type-check.

### Not verified

- **Nothing compiled.** No Cargo, rustc or clippy-driver ran, per Mail
  6834. The new model functions, facts, tests and scene code have never been
  compiled or run, so type or borrow errors are possible. The rustc results
  in the first handoff apply to the pre-repair model only.
- **Test runtime.** `one_liar_among_five_is_found` tries 267 recipes × 5
  liars × 10 lies. Its debug-mode runtime is unmeasured; my estimate is a
  few seconds.
- **Pictures.** The door-pulse clearance is computed from the layout
  constants. It assumes fmn's `indicate` scales about the centre, and nothing
  was rendered. The 01_4 timing gain is an estimate.
- **No output.** Nothing has been synthesized, rendered, watched or listened
  to. Pronunciations are unverified, and nothing here claims English-accent
  consistency.
- **A stray hook file.**
  `explainer_corpus/shamir_secret_sharing/.ee/hook-state/session_start_orient.last.json`
  (dated 01:06) was written into this directory by a session hook. It is not
  a lane file. I did not delete it, and root may want it gone before any
  commit.


## Chapters 02 and 03, unregistered (2026-10-24, batch_pending, not self-closed)

Root's direction after Mail 6852: implement `02_cutting_it_up` and
`03_the_dial` from the revision-4 draft as new, unregistered scene files,
with a new helper if needed, and touch nothing else but this file. The words
are the draft's, unchanged.

Everything registered stays as handed off in 6852, frozen: the opening,
model, facts, oracle, `draw.rs`, `mod.rs`, the storyboards, both scripts,
Cargo. My reservation now covers only the three new files and this one
(7852–7855). Topic-wide 7536 is released, so root can integrate the opening.
At 07:48Z, `the_vault.rs` changed outside my edits: its last layout check is
now a `const` assert. I left it alone.

### Files (all new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

- **`two_keys.rs`** (helper):
  - a dial `Face` drawable at any centre and radius, whose needle tracker is
    unwrapped, so 16 points where 5 does;
  - `Face::click`, which turns one mark per click, flashes each mark landed
    on and can drive a count;
  - `Face::arc`, a labelled arc outside the rim;
  - `Face::zoom`, a cross-zoom between two faces (the needle gets a
    corrected shift);
  - `TwoKeys`: the small door, with A and B and their card frames;
  - `holder_card` and `card_back`;
  - the searches: `ordinary_secrets`, `if_a_held` and `dial_secrets`;
  - the strip views `ordinary_view` and `dial_view`, and `check_strip`;
  - `passed` and `next_top`;
  - the spin role `spin()` (yellow);
  - `moving`, `move_all`, `leaving` and `leaving_all_but`;
  - tests.
- **`cutting_it_up.rs`** (`CuttingItUp`, key `02_cutting_it_up`), with
  tests.
- **`the_dial.rs`** (`TheDial`, key `03_the_dial`), with tests.

### How the brief's requirements are met

- **The strip and the cards come from one variable.** For B, the strip's
  view is built from the same `b` that B's card prints. Each time the strip
  shows it, the scene checks `strip.lit()` against a separate search over
  what A could hold, run on that `b`, and fails if they differ.
  - **Cut pieces:** the view is `View::OrdinaryPiece(b)`. The search tries
    every whole-number piece for A, 0 or more, whose sum with `b` lands on a
    mark. 02_4 strikes exactly the marks the search excludes, 0 to b − 1.
    02_5 re-checks at every step, b = 0 to 5, and the counts run 11, 10, 9,
    8, 7, 6.
  - **On the dial:** the view is `dial_view(b)`. The model has no view for a
    two-of-two dial share, so this one stands in: `View::Shares { points:
    [(1, b)], spins: 1 }`. That is the same mathematics, because the secret
    take away a fair spin is the secret plus a fair spin. The search turns
    each of A's 11 marks on by `b`. It keeps all eleven secrets, each met
    exactly once, and the scene fails if any count is not 1.
  - **Tests:** both rules are tested for every b from 0 to 10.
- **Real turning.** Every dial sum is shown as clicks, one mark at a time.
  Each mark landed on flashes, and a live count reads the clicks.
  - 03_1: 9 → 10 → 0 → … → 5.
  - 03_2: back from 5 → … → 0 → 10 → 9.
  - 03_4: 9 → 5, following the "+7" arc.
  - The equations appear only after the clicks: "9 + 7 = 16 → 5" with grey
    "16 − 11 = 5", then a "+1" arc from 10 to 0 with "10 + 1 → 0", then
    "5 − 7 → 9".
  - The scene checks the marks the clicks land on, and the click timing is
    fitted to the words, at 0.3 to 0.6 s a click.
- **A fair spin.** The spare dial is titled "dealer's fair spin". It makes
  two whole turns, slows and stops at 9, and A's card comes from beside its
  9. In 03_6, A's face-down card flashes at "A's mark came from a fair
  spin".
- **A continuing vault, cards and dial.**
  - 02 opens on 01's last frame: the vault, the rail of empty frames, and
    the strip for anyone outside.
  - The vault and rail slide aside for the small door, whose dial is the
    vault's dial. The whole secret, the gold 5, splits into the cards 2 and
    3, which fly to A and B.
  - 02 ends with the small dial zooming into the enlarged one, and 03 opens
    on it.
  - 03_3 zooms back to the small door, where A and B return with their old
    cards 2 and 3, which then fade.
  - 03 ends with the big vault, the rail and "threshold 3".
- **Prediction holds.** `think()` runs after 02_3 and 03_4, once each
  question's line has ended. Each "?" is cleared when the answer starts.
- **Leaking and needing both are kept apart.**
  - Leaking is about what B can work out, so it lives on the strip: 02_4 and
    02_5. Its tag, "each piece leaks", sits under the strip's caption.
  - Needing both is about the door: in 02_6, A greys out and the door pulses
    shut. Its tag, "needs A and B both", sits beside the door.
  - In 03_7, the green "B's share leaks nothing" sits under the strip, and
    the red "still needs A and B both" sits by the door, which pulses.

### Picture cells that differ from the draft (words unchanged)

The production storyboard's rows must equal the draft's, so these cells
need root's agreement when the chapters are integrated.

1. **02_3 and 03_4.** The draft has the caption change to "for: B" during
   the question. The scenes keep "for: anyone outside" until the answer, and
   then caption, ticks and count change in one move: at "Zero, one and two"
   in 02_4, and at "once each" in 03_5. Under B's caption, the strip must
   show the set computed from B's share. Showing it during the question
   would give the answer away, and showing all eleven would be false for a
   cut piece.
2. **03_5.** The draft has the strip "relight one tick per row". In the
   scene, the strip stays lit, and each table row puts one blue dot over
   the tick of the secret it meets: eleven dots, one per tick, which is
   "once each". At "once each" the caption becomes "for: B", recomputed from
   B's 7, and the dots flash. A partly lit strip is no one's set of possible
   secrets, and `Strip` cannot show one.
3. **02_4.** Two notes earn "at least three": "A's piece ≥ 0" and "secret =
   A's piece + 3 ≥ 3".
4. **02_7 and 03_7.** The two tags sit by what they are about, as above,
   not both under the small door.
5. **03_2.** The "+1" arc from 10 to 0 and the "5 − 7 → 9" equation are
   added.
6. **03_3.** The dealer's "secret 5" sits under the spare dial.
7. **03_6.**
   - The strip's caption reads "for: B, holding 7 on the dial".
   - The cut-piece line runs under the strip, column for column, with a red
     "floor" bar between marks 2 and 3. It is captioned "for: B, holding a
     cut piece 3: 8 possible". Both the line and the strip are computed and
     checked.

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

- **Formatting.** `rustfmt --edition 2024 --config-path rustfmt.toml
  --check` on each new file gives rc 0 after layout rewrites. Each file is
  parsed on its own, since none is reachable from `main.rs`, so there are no
  syntax errors. Nothing was type-checked.
- **Cue phrases** (scratch `ch0203/check_cues.py`). The script reads the
  phrases out of the scene code itself: each `say` line's `wait_for` and
  `at`. There are 47 uses, 18 in 02 and 29 in 03. Every one is in its line
  of `docs/script-draft.tsv`, and they come in spoken order. Each scene says
  its seven lines in draft order, and every phrase is in the scene's test
  list. 0 failures.
- **A Python replica of the tests' numbers** (scratch `ch0203/replica.py`).
  The searches are written out again, and the `Shares` view is
  brute-forced. 0 failures:
  - For every b: the cut-piece rule, the dial rule and once-each.
  - Clicks, `next_top`, `click_time` and the zoom identity.
  - 17 S-F0 and S-F2 values, checked against the oracle's output.
  - The pinned strings. The 03_6 card is 95 characters: two lines at every
    wrap from 92 to 72, and 6.3 s to read at 15 characters a second.
  - 30 layout bounds.
- **The oracle**, re-run: rc 0, "188 facts, 0 failed cross-checks", and its
  output is byte-equal to the repair's.
- **Proposed `SPOKEN` coverage.** The 29 rows and 4 idioms listed below
  cover every number word of the 02 and 03 draft lines. Each phrase is in
  its line, and each value equals the oracle's, except for the four facts
  proposed below. 0 problems.
- **A timing walk** using the scratch voice's 2.6 words a second, which is
  only an estimate. It found four pictures landing 0.2–0.6 s after their
  words: 02_3's question, 03_1's first turn, 03_3's "B gets the turn" and
  03_4's question. Each was fixed by merging sequential moves into one move.
  The needle also returns to the top before each question, so the secret
  isn't left showing on the dial.

### Not verified

- **Nothing compiled.** No Cargo, rustc or clippy-driver ran. All three
  files are unregistered, and they have never been compiled, run or tested.
  Expect type and borrow fixes.
  - I removed one nested `stage` borrow and one runtime assert on
    constants, after root's change to `the_vault.rs` suggested a Clippy
    lint. Others may remain.
  - The tests read `docs/script-draft.tsv` through `include_str!`, because
    `script.tsv` has chapter 01 only. Once integrated, the scenes' `say` will
    need the 02 and 03 lines in `script.tsv`.
- **Engine behaviour I relied on without running it:**
  - `fade_in` and `fade_out` with a scale and a shift, used as a zoom. The
    fmn source says the start copy is scaled by 1/k about its centre and then
    shifted by −d, and the end copy is shifted by d and then scaled by k.
  - The scene's top-level list, snapshotted after the vault and rail come
    in, used to send them aside in 02_2.
  - `indicate` on a label inside a bundled face.
- **Pictures and pacing.** Nothing was rendered, synthesized, watched or
  listened to.
  - All timings are estimates.
  - The text widths in the layout tests are generous per-character guesses.
  - "≥" and "→" are in other topics' rendered text, but the glyph probe
    has not run on these strings.
  - Nothing here claims English-accent consistency.

### For root's integration (proposals; I changed none of these files)

- **`mod.rs`.** Add `pub mod two_keys; pub mod cutting_it_up; pub mod
  the_dial;` and two `Chapter`s, keys `02_cutting_it_up` and `03_the_dial`
  (intensity proposed: 0.5, as 01).
- **`docs/storyboard.md`.**
  - Copy the 02 and 03 rows from the draft, with the picture cells above.
  - Fix the stale "21 chapters" line; the draft has 23.
  - Regenerate `script.tsv`.
  - The scenes' cue tests could then read `crate::SCRIPT` instead of the
    draft.
- **`main.rs`.**
  - New `SPOKEN` rows using existing facts:
    - 02_2: "both of two trustees" and "into two ordinary numbers" →
      `S-F0.pieces.both_needed` 2.
    - 02_3: "add two and three, get five" → `S-F0.pieces.sum` 5.
    - 02_4: "Zero, one and two" → `S-F0.pieces.b_alone_ruled_out` "0,1,2";
      "only eight possible" → `b_alone_left` 8.
    - 02_6: "eight possible" → `b_alone_left` 8.
    - 03_1: "From nine" → `S-F2.a_share`; "turn seven marks" →
      `b_share`; "past ten" → `top`; "round through zero" →
      `ten_plus_one`; "stop at five" → `sum`.
    - 03_2: "nine plus seven is five" → `sum`; "Ten plus one is zero" →
      `ten_plus_one`; "five take away seven is nine" → `turn_back`.
    - 03_3: "stops, nine" → `a_share`; "from nine to the secret, seven" →
      `b_share`.
    - 03_4: "turn nine on by seven, reach five" → `sum`; "B's seven" →
      `b_share`.
    - 03_5: "all eleven marks" → `S-F1.marks`; "all eleven secrets" →
      `distinct_secrets`; "once each" → `ways_each`.
    - 03_6: "B's seven" → `b_share`.
  - Four phrases need single-valued facts that don't exist yet, which is an
    oracle and `facts.rs` change:
    - "two for A" → `S-F0.pieces.a` = 2;
    - "three for B", "B's three" and "at least three" → `S-F0.pieces.b`
      = 3;
    - "If A had zero, the secret would be seven" →
      `S-F2.if_a_held.0` = 7;
    - "If A had one, eight" → `S-F2.if_a_held.1` = 8.

    Alternatively, map them to the list keys.
  - `NOT_A_COUNT`: 02_1 "The first idea", 02_4 "less than zero" (the
    floor of whole numbers), 02_6 "a second way", 02_7 "fixed first".
- **Changes to existing helpers** (copied into `two_keys.rs` for now):
  - **`model.rs`:** `View::DialPiece(b)`, to replace `dial_view`'s
    stand-in.
  - **`draw.rs`:**
    - Make `DOOR_O`, `CARD_SIZE`, `FLASH` and the dial proportions `pub`.
    - Add `parts()` accessors on `Vault`, `Rail` and `Strip`, so a scene can
      move or fade a device by name instead of by scene snapshot.
    - Consider building `Vault` on `Face`.
  - **`palette.rs`:** `spin()`.
- **Glyphs.** The probe captures run-time strings, so no list change should
  be needed.

## Chapters 02 and 03: text-emphasis repair (2026-10-24, batch_pending, not self-closed)

Root's Mail 6866, after 6865. Root's native previews of chapter 01 show
text emphasis doubling the displayed 5, and a recolor leaving it doubled.
The ask:
- repair only the three unregistered 02/03 files;
- no indicate or style transform on typeset text: use a separate outline
  or halo, or replacement glyphs;
- preserve the prior bytes;
- prepare a precise patch for the missing helpers, without editing
  registered files;
- check whether `View::DialPiece` is needed;
- make sure no prediction answer shows early.

The registered source is untouched: `the_vault.rs`, `draw.rs`, `model.rs`,
`facts.rs`, `palette.rs`, `mod.rs`, `main.rs`, the scripts, the storyboards
and Cargo. Reservations: 7852–7855 (the three files and this one) and 7898
(the snapshot folder).

### Prior bytes

`docs/previous/ch0203-before-text-emphasis-repair-20261024/` is gitignored.
It holds 6865's bytes of the three files and of this file, copied before any
change:
- `two_keys.rs` 346bbdb6…
- `cutting_it_up.rs` 691846fb…
- `the_dial.rs` 079e1ef8…
- `WORK_STATUS.md` c888e34e…

After the repair (sha256):
- `two_keys.rs` 8f0b58e4…
- `cutting_it_up.rs` bd300635…
- `the_dial.rs` 35e89870…

### What changed

**The rule.** Typeset text is never indicated, recolored or scaled for
emphasis. A mark, a card or a tag is pointed at by a ring or an outline of
its own. The ring fades in for half the old flash time and out for the
other half, so every cue keeps its timing. Only shapes swell, tinted in
their own role.

**New in `two_keys.rs`** (its emphasis section):
- `Face::halo(stage, k, color)`: a ring round mark k's label, radius 0.135
  of the face's, line width 2.
- `slot_halo(stage, i)`: a share-blue outline 0.07 outside holder i's card
  frame, line width 3. It works for a card face up or face down.
- `text_halo(stage, m, color)`: an outline 0.12 outside the words' measured
  bounds.
- `glowing` / `glow`: each halo is a `Succession` of `fade_in` then
  `fade_out`. The fade-out is a remover, so no halo stays in the scene.
- `swell_all(stage, shapes, color, rt)`: indicate, on shapes only, tinted
  `color`. It never uses the default yellow, which is the spins' role here.
- `TwoKeys::pulse` now tints the small door ghost, as root's `Vault::pulse`
  does ("the indication default yellow … looked like an opening/failure
  colour in the preview"). In 6865 it was yellow.
- `Face::label` and the `labels` field are removed. With no caller left,
  nothing can hand a label to an emphasis.

**Every emphasis, before and after.** Each cue and its time are unchanged.

| Line | Cue | 6865 | Now |
|---|---|---|---|
| 02_3 | after the sum | flash the cards 2 and 3 (text) | halos round A's and B's frames, 0.4 s |
| 02_3 | "what does B's three say" | indicate the card 3 (text), with the card swap | B's frame halo, 0.5 s, in the same move |
| 02_4 | "A's piece can't be less than zero" | flash A's face-down card (a panel, yellow) | A's frame halo, 0.4 s |
| 02_6 | "B's three still leaves" | flash the card 3 (text) | B's frame halo, 0.4 s |
| 02_6, 02_7 | "stays shut", "needs every piece" | door pulse, yellow | door pulse, ghost tint |
| 02_7 | "The leak gets fixed first" | flash the tag "each piece leaks" (text) | red outline round the tag, 0.5 s |
| 03_1 | "From nine" | flash the label 9 | hero ring round 9, 0.3 s |
| 03_1, 03_2, 03_4 | every click | indicate the label landed on | hero ring round it, over the click |
| 03_1 | "stop at five" | flash 5 | ring round 5, 0.5 s |
| 03_2 | "Ten plus one is zero" | flash 10, then 0 | ring round 10, then round 0, 0.3 s each |
| 03_2 | "is nine" | flash 9 | ring round 9, 0.4 s |
| 03_3 | "stops, nine" | indicate the spare dial's 9, with A's card | spin-yellow ring round the spare's 9, in the same move |
| 03_3 | "B gets the turn" | flash the small dial's 5 | ring round 5, 0.4 s |
| 03_4 | "turn nine on by seven" | flash the card 9 (text) | A's frame halo, 0.3 s |
| 03_4 | "What does B's seven say" | flash the card 7 (text) | B's frame halo, 0.5 s |
| 03_5 | "once each" | flash the eleven dots (yellow) | the dots swell, share tint |
| 03_6 | "A's mark came from a fair spin" | flash A's face-down card (yellow) | A's frame halo, 0.4 s |
| 03_6 | "above or below anything" | flash the floor bar (yellow) | the bar swells, leak tint |
| 03_7 | "still needs both trustees" | door pulse, yellow | door pulse, ghost tint |

**What still touches text.** None of these is a style transform:
- Cards move to their frames (`moving`).
- A changed card or caption is new text crossfaded in (`swap`,
  `Strip::show`).
- The zooms at the end of 02_7 and in 03_3 fade the face group with a
  scale. Whether a scaled fade of text doubles glyphs natively is untested.
  The fallback is `Face::zoom` with `d = ORIGIN` and `k = 1.0`, a plain
  crossfade at the same timing.

**The recolors left.** The small door (a panel), A's disc and A's face-down
card (a panel). `Strip::show` (`draw.rs`) recolors only the tick panels.

### Prediction holds

I checked both holds again with the halos in:
- **02_3.** Before the "?", the strip still says "for: anyone outside", with
  all eleven ticks lit. The sum and A's card have gone, the needle is back
  at the top, and only B's frame is ringed. The strikes, the notes and B's
  caption arrive in 02_4.
- **03_4.** Before the question, the "+7" arc, A's card and the open door
  have gone, and the needle is at the top. The ring on B's frame is the only
  emphasis. Nothing for B is shown until "once each" in 03_5.

Every halo has faded out by the end of its glow, so none carries into a
hold.

### `View::DialPiece` is not needed: withdrawn

`dial_view(b)` is `View::Shares { points: [(1, b)], spins: 1 }`. B's dial
share is the secret minus A's fair spin, which is the secret plus a fair
spin: the one-spin share at 1. `View::possible` then keeps all eleven
secrets for every b.

The test `a_dial_share_keeps_every_secret_once` now also asserts
`learns_nothing(dial, &[1], 1, any_spins)`, the model's own equal-count
check. No model change is proposed.

### Integration patch (a proposal; nothing registered was edited)

**Withdrawn from 6865's list:**
- `View::DialPiece` (see above).
- `parts()` on `Vault`, `Rail` and `Strip`. 02_2's scene snapshot is taken
  before the strip comes in, so it already names exactly the vault and the
  rail.

**What the patch below does:**
- **`draw.rs`.**
  - `pub` on `DOOR_O`, `TICK_IN`, `NEEDLE_R`, `HUB_R`, `CARD_SIZE` and
    `FLASH`.
  - New `pub` constants `MARK_LABEL_SIZE` 18, `DIAL_LINE` 3 and `HAND_LINE`
    6, used in `Vault::build` in place of its literals. No value changes.
- **`palette.rs`.** `spin()` is yellow and joins the roles-distinct test.
  The module note is updated.
- **`mod.rs`.**
  - `pub mod` for the three files.
  - Chapters 02 and 03 registered, at intensity 0.5.
  - The doc's count fixed: chapters 01 to 03, and twenty more.
- **`two_keys.rs` and `the_dial.rs`.** They use these instead of their own
  copies: two_keys's `spin`, `DOOR_O`, `CARD_SIZE` and `SWELL`. The face
  proportions become draw.rs's divided by `DIAL_R`.

**Apply it as one unit**, together with:
- the 02 and 03 rows in `docs/storyboard.md`;
- a regenerated `script.tsv`;
- the `SPOKEN` rows in the section above.

The reasons: `main.rs`'s test `check_video` compares the registry with the
storyboard and the script, and `palette::spin` is dead code until the_dial
is registered. The `draw.rs` hunks alone change no value, so they can land
earlier.

**Checked.**
- `git apply --check` from the repository root gives rc 0. Nothing was
  applied.
- On the patched tree, `rustfmt --check` on `main.rs`, which reaches every
  module, gives rc 0.
- Nothing was compiled.

To extract the patch and apply it:

~~~sh
awk '/^<!-- shamir-0203 patch end -->$/{p=0} p; /^<!-- shamir-0203 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-0203.patch
git apply --check shamir-0203.patch && git apply shamir-0203.patch
~~~

<!-- shamir-0203 patch begin -->
```diff
diff -ruN a/explainer_corpus/shamir_secret_sharing/src/chapters/draw.rs b/explainer_corpus/shamir_secret_sharing/src/chapters/draw.rs
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/draw.rs	2026-10-24 03:55:38.358309689 -0400
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/draw.rs	2026-10-24 04:20:48.610570245 -0400
@@ -95,15 +95,19 @@
 pub const DOOR_W: f64 = 3.0;
 pub const DOOR_H: f64 = 3.5;
 /// The door's fill opacity, whatever its color.
-const DOOR_O: f64 = 0.55;
+pub const DOOR_O: f64 = 0.55;
 pub const DIAL_Y: f64 = 1.05;
 pub const DIAL_R: f64 = 1.0;
 /// Where a mark's tick starts (it runs out to the rim), its label sits, and
 /// the needle's tip reaches, from the dial's centre.
-const TICK_IN: f64 = 0.86;
+pub const TICK_IN: f64 = 0.86;
 pub const MARK_LABEL_R: f64 = 0.68;
-const NEEDLE_R: f64 = 0.48;
-const HUB_R: f64 = 0.08;
+pub const NEEDLE_R: f64 = 0.48;
+pub const HUB_R: f64 = 0.08;
+/// A mark's label size, the rim's and ticks' line width, the needle's.
+pub const MARK_LABEL_SIZE: f64 = 18.0;
+pub const DIAL_LINE: f64 = 3.0;
+pub const HAND_LINE: f64 = 6.0;
 /// The alarm lamp over the door.
 pub const LAMP_Y: f64 = 3.0;
 pub const LAMP_R: f64 = 0.16;
@@ -140,7 +144,7 @@
         let door = panel(stage, palette::ghost(), DOOR_O, v(DOOR_X, DOOR_Y), DOOR_W, DOOR_H)?;
         let rim = Circle::new()
             .radius(DIAL_R)
-            .style(line_style(Palette::dim(), 3.0))
+            .style(line_style(Palette::dim(), DIAL_LINE))
             .build()
             .moved_to(v(DOOR_X, DIAL_Y));
         let rim = stage.arena_mut().add(rim);
@@ -152,12 +156,12 @@
                 stage,
                 vec![on_dial(at, marks, TICK_IN), on_dial(at, marks, DIAL_R)],
                 Palette::dim(),
-                3.0,
+                DIAL_LINE,
             ));
             parts.push(kit.text(
                 stage,
                 &k.to_string(),
-                18.0,
+                MARK_LABEL_SIZE,
                 Palette::ink(),
                 on_dial(at, marks, MARK_LABEL_R),
             )?);
@@ -169,7 +173,7 @@
         let needle_mob = stage.always_redraw(move |s| {
             let k = s.tracker_value(needle).unwrap_or(0.0);
             let hand = polyline(vec![v(DOOR_X, DIAL_Y), on_dial(k, marks, NEEDLE_R)])
-                .style(line_style(palette::secret(), 6.0))
+                .style(line_style(palette::secret(), HAND_LINE))
                 .build();
             s.add(hand)
         });
@@ -225,7 +229,7 @@
 pub const FRAME_W: f64 = 0.62;
 pub const FRAME_H: f64 = 0.44;
 /// A card's digits, and a tag's words, in a frame.
-const CARD_SIZE: f64 = 26.0;
+pub const CARD_SIZE: f64 = 26.0;
 const TAG_SIZE: f64 = 16.0;
 
 /// Trustee `t`'s disc centre (trustees count from 1).
@@ -536,7 +540,7 @@
 }
 
 /// How far a flash grows what it highlights.
-const FLASH: f64 = 1.25;
+pub const FLASH: f64 = 1.25;
 
 /// Flash several mobjects (the highlight role), together.
 pub fn flash_all(stage: &mut Stage<'_>, mobs: &[Mob], rt: f64) -> fmn::Result<()> {
diff -ruN a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs	2026-10-24 01:54:53.383078206 -0400
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs	2026-10-24 04:20:48.610937789 -0400
@@ -1,20 +1,35 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapter 01 so far. The other eighteen
-//! are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 03 so far. The other
+//! twenty are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
+pub mod cutting_it_up;
 pub mod draw;
 pub mod glyphs;
+pub mod the_dial;
 pub mod the_vault;
+pub mod two_keys;
 
 use explainer_kit::Chapter;
 
 pub fn registry() -> Vec<Chapter> {
-    vec![Chapter {
-        key: "01_the_vault",
-        make: |kit| Box::new(the_vault::TheVault { kit }),
-        intensity: 0.5,
-    }]
+    vec![
+        Chapter {
+            key: "01_the_vault",
+            make: |kit| Box::new(the_vault::TheVault { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "02_cutting_it_up",
+            make: |kit| Box::new(cutting_it_up::CuttingItUp { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "03_the_dial",
+            make: |kit| Box::new(the_dial::TheDial { kit }),
+            intensity: 0.5,
+        },
+    ]
 }
diff -ruN a/explainer_corpus/shamir_secret_sharing/src/chapters/the_dial.rs b/explainer_corpus/shamir_secret_sharing/src/chapters/the_dial.rs
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/the_dial.rs	2026-10-24 04:15:35.457067721 -0400
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/the_dial.rs	2026-10-24 04:21:37.991719507 -0400
@@ -26,7 +26,7 @@
 use super::two_keys::{
     ARC_LABEL_IN, BIG_R, BIG_X, BIG_Y, Face, MID_X, SPARE_R, SPARE_Y, TwoKeys, big_centre, card_back, check_strip,
     dial_secrets, dial_view, glow, glowing, holder_card, if_a_held, leaving, move_all, next_top, ordinary_secrets,
-    ordinary_view, slot_at, slot_halo, spin, swell_all,
+    ordinary_view, slot_at, slot_halo, swell_all,
 };
 use crate::model::{A_SHARE, Dial, ORDINARY_PIECES, Ratio, SECRET, THRESHOLD, TRUSTEES, View};
 use crate::palette;
@@ -273,8 +273,14 @@
         strip.reveal(&kit, stage, 0.6)?;
         strip.show(&kit, stage, &View::Outsider, FOR_OUTSIDE, 0.3)?;
         line.wait_for(stage, "The dealer")?;
-        let spare = Face::build(&kit, stage, dial, v(MID_X, SPARE_Y), SPARE_R, spin(), 0)?;
-        let title = kit.text(stage, SPARE_TITLE, 22.0, spin(), v(MID_X, SPARE_Y + SPARE_TITLE_DY))?;
+        let spare = Face::build(&kit, stage, dial, v(MID_X, SPARE_Y), SPARE_R, palette::spin(), 0)?;
+        let title = kit.text(
+            stage,
+            SPARE_TITLE,
+            22.0,
+            palette::spin(),
+            v(MID_X, SPARE_Y + SPARE_TITLE_DY),
+        )?;
         let dealer_secret = kit.text(
             stage,
             &format!("secret {SECRET}"),
@@ -302,7 +308,7 @@
         let a_card = holder_card(&kit, stage, A, &A_SHARE.to_string(), palette::share())?;
         stage.move_to(a_card, spare.point(f64::from(A_SHARE), SPARE_R * ARC_LABEL_IN), ORIGIN);
         let mut anims = swaps(stage, &[], &[a_card], 0.3)?;
-        let spun = spare.halo(stage, A_SHARE, spin());
+        let spun = spare.halo(stage, A_SHARE, palette::spin());
         anims.extend(glowing(stage, &[spun], 0.3)?);
         stage.play_prepared(anims)?;
         move_all(stage, &[(a_card, slot_at(A))], 0.5)?;
diff -ruN a/explainer_corpus/shamir_secret_sharing/src/chapters/two_keys.rs b/explainer_corpus/shamir_secret_sharing/src/chapters/two_keys.rs
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/two_keys.rs	2026-10-24 04:18:49.404793470 -0400
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/two_keys.rs	2026-10-24 04:21:24.224568768 -0400
@@ -24,19 +24,12 @@
 use fmn::prelude::*;
 
 use super::draw::{
-    DOOR_PULSE, FRAME_H, FRAME_W, FRAME_Y, RAIL_Y, Strip, TRUSTEE_R, bundle, disc, fail, frame, line_style, panel,
-    stroke,
+    CARD_SIZE, DIAL_LINE, DIAL_R, DOOR_O, DOOR_PULSE, FLASH, FRAME_H, FRAME_W, FRAME_Y, HAND_LINE, HUB_R, MARK_LABEL_R,
+    MARK_LABEL_SIZE, NEEDLE_R, RAIL_Y, Strip, TRUSTEE_R, bundle, disc, fail, frame, line_style, panel, stroke,
 };
 use crate::model::{Dial, View};
 use crate::palette;
 
-/// The dealer's spins (storyboard-draft, "Color roles": yellow). Chapter
-/// 03's spare dial is the first spin drawn; palette.rs takes this role when
-/// the chapter is registered.
-pub fn spin() -> Srgb {
-    Palette::yellow()
-}
-
 // ------------------------------------------------------------- B's share
 
 /// The secrets B's ordinary piece `b` leaves: A's piece is a whole number,
@@ -110,16 +103,13 @@
 // ------------------------------------------------------------ a dial face
 
 /// Where a mark's tick starts, its label sits and the needle's tip reaches,
-/// as fractions of the face's radius (the vault's proportions, draw.rs).
-const TICK_IN: f64 = 0.86;
-const LABEL_IN: f64 = 0.68;
-const NEEDLE_IN: f64 = 0.48;
-const HUB_IN: f64 = 0.08;
-/// A mark's label size per unit of radius (the vault's: 18 at radius 1).
-const LABEL_PER_R: f64 = 18.0;
-/// The ticks' and the needle's line widths (the vault's).
-const TICK_LINE: f64 = 3.0;
-const HAND_LINE: f64 = 6.0;
+/// as fractions of the face's radius: the vault's proportions (draw.rs).
+const TICK_IN: f64 = super::draw::TICK_IN / DIAL_R;
+const LABEL_IN: f64 = MARK_LABEL_R / DIAL_R;
+const NEEDLE_IN: f64 = NEEDLE_R / DIAL_R;
+const HUB_IN: f64 = HUB_R / DIAL_R;
+/// A mark's label size per unit of radius (the vault's).
+const LABEL_PER_R: f64 = MARK_LABEL_SIZE / DIAL_R;
 /// A mark's halo's radius, as a fraction of the face's, and its line width:
 /// round the label, inside the ticks, clear of the needle's tip and of the
 /// neighbouring labels, on the smallest face too.
@@ -159,7 +149,7 @@
         let at = move |k: f64, d: f64| -> Vec3 { point_on(centre, marks, k, d) };
         let ring = Circle::new()
             .radius(r)
-            .style(line_style(rim, 3.0))
+            .style(line_style(rim, DIAL_LINE))
             .build()
             .moved_to(centre);
         let ring = stage.arena_mut().add(ring);
@@ -170,7 +160,7 @@
                 stage,
                 vec![at(kf, TICK_IN * r), at(kf, r)],
                 Palette::dim(),
-                TICK_LINE,
+                DIAL_LINE,
             ));
             members.push(kit.text(
                 stage,
@@ -340,14 +330,10 @@
 pub const SMALL_H: f64 = 2.7;
 pub const SMALL_DIAL_Y: f64 = 1.1;
 pub const SMALL_DIAL_R: f64 = 0.8;
-/// The door's fill opacity, as the big door's (draw.rs).
-const DOOR_O: f64 = 0.55;
 /// A and B, under the small door, each with a card frame over them, at the
 /// rail's heights (draw.rs).
 pub const HOLDERS: [&str; 2] = ["A", "B"];
 pub const HOLDER_X: [f64; 2] = [-5.75, -4.65];
-/// A card's digits, as the rail's cards (draw.rs).
-const CARD_SIZE: f64 = 26.0;
 /// The column between the small door and the strip: equations, notes, the
 /// dealer's spare dial, chapter 03's table.
 pub const MID_X: f64 = -1.6;
@@ -467,8 +453,6 @@
 /// words.
 const SLOT_PAD: f64 = 0.07;
 pub const TEXT_PAD: f64 = 0.12;
-/// How far a swell grows a shape (as draw.rs's flash).
-const SWELL: f64 = 1.25;
 
 /// A ring of radius `r` round `at`, in `color`. Detached.
 fn ring(stage: &mut Stage<'_>, at: Vec3, r: f64, color: Srgb) -> Mob {
@@ -557,7 +541,7 @@
     }
     let mut anims: Vec<Box<dyn Animation>> = Vec::with_capacity(shapes.len());
     for &m in shapes {
-        anims.push(swelling(stage, m, SWELL, color, rt)?);
+        anims.push(swelling(stage, m, FLASH, color, rt)?);
     }
     stage.play_prepared(anims)?;
     Ok(())
@@ -748,7 +732,7 @@
             let reach = (0.0055 * LABEL_PER_R * r).hypot(0.0065 * LABEL_PER_R * r / 2.0);
             assert!(ring - half(RING_LINE) > reach, "the ring clears the label, radius {r}");
             assert!(
-                centre + ring + half(RING_LINE) < TICK_IN * r - half(TICK_LINE) - 0.003,
+                centre + ring + half(RING_LINE) < TICK_IN * r - half(DIAL_LINE) - 0.003,
                 "the ring inside the ticks, radius {r}"
             );
             assert!(
diff -ruN a/explainer_corpus/shamir_secret_sharing/src/palette.rs b/explainer_corpus/shamir_secret_sharing/src/palette.rs
--- a/explainer_corpus/shamir_secret_sharing/src/palette.rs	2026-10-24 02:05:30.498755969 -0400
+++ b/explainer_corpus/shamir_secret_sharing/src/palette.rs	2026-10-24 04:20:48.610243577 -0400
@@ -1,7 +1,6 @@
 //! The video's color roles (docs/storyboard-draft.md, "Color roles"): one
 //! color per idea for the whole video, on top of the corpus's shared roles.
-//! Only the roles an implemented chapter draws are here; the dealer's spins
-//! (yellow, chapter 05 on) join with their scene.
+//! Only the roles an implemented chapter draws are here.
 
 use explainer_kit::Palette;
 use fmn::prelude::Srgb;
@@ -41,13 +40,19 @@
     Palette::green()
 }
 
+/// The dealer's fair spins: the spare dial, its title and the mark it stops
+/// on (chapter 03 on).
+pub fn spin() -> Srgb {
+    Palette::yellow()
+}
+
 #[cfg(test)]
 mod tests {
     use super::*;
 
     #[test]
     fn every_role_is_its_own_color() {
-        let roles = [secret(), share(), trio(), pair(), ghost(), leak(), open()];
+        let roles = [secret(), share(), trio(), pair(), ghost(), leak(), open(), spin()];
         for (i, a) in roles.iter().enumerate() {
             for b in &roles[i + 1..] {
                 assert_ne!(a, b);
```
<!-- shamir-0203 patch end -->

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

- **Formatting.** `rustfmt --check --edition 2024` gives rc 0 on each of the
  three files. This is a parse check only.
- **Cue phrases** (scratch `ch0203/check_cues.py`). 18 phrases in 02 and 29
  in 03; each scene says its seven lines in order. 0 failures. The phrases
  are unchanged.
- **The replica** (scratch `ch0203/replica.py`), re-run: 17 facts and 30
  layout bounds, 0 failures. No number moved.
- **Halo bounds** (new scratch `ch0203/halo_bounds.py`). It recomputes the
  new asserts from constants copied by hand: 0 failures. A line-width unit
  is 0.01 scene units (fmn's `STROKE_WIDTH_CONVERSION`), and line ends are
  round-capped. The clearances, in scene units:

  | Face | Ring radius | Clear of "10" (estimate / at 0.0045 a point) | Clear of its tick | Clear of the needle's tip |
  |---|---|---|---|---|
  | spare, r 0.75 | 0.101 | 0.005 / 0.020 | 0.009 | 0.009 |
  | small door, r 0.8 | 0.108 | 0.006 / 0.022 | 0.011 | 0.012 |
  | enlarged, r 2.2 | 0.297 | 0.034 / 0.079 | 0.074 | 0.103 |
- **The patch.**
  - `git apply --check` gives rc 0.
  - On the patched tree, `rustfmt --check` via `main.rs` gives rc 0.
  - The block above, extracted with the awk line, is byte-equal to the
    generated patch, and the extract passes `git apply --check`.
- **Static audit of the three files.**
  - `indicate` appears only in `swelling`, which is given shapes: the door,
    the dots and the floor bar.
  - The recolors target the door, A's disc and A's face-down card.
  - No emphasis call takes a text mobject.

### Not verified

- **Nothing compiled or ran.** No Cargo, rustc or clippy-driver.
  - The halos rely on a `Succession` of a `fade_in` and a `fade_out` of one
    mobject. fmn's own tests cover fade successions (`deferred_fading.rs`),
    but no scene in this repository uses `Succession` yet.
  - They also rely on fmn's `remove_from_scene` restructuring a halo out of
    the succession's container.
- **Nothing rendered or watched.**
  - On the small faces, the ring comes within about 0.01 scene units of its
    tick's end and of a needle pointing at it. That is about 2.7 px at 4K,
    so it may read as touching. If it crowds, use a `RING_LINE` of 1.5, or
    ring marks only on the enlarged dial.
  - The text sizes are estimates.
- **The doubling's cause is root's finding.** I have not reproduced it. The
  repair avoids the two operations named, indicate and recolor on text, but
  moves, crossfades and the zoom's scaled fade still touch text.
- **No listening.** Nothing here claims accent consistency.

### For chapter 01 (an offer only)

If root wants the same device in `the_vault.rs`, two_keys's emphasis
section can move to `draw.rs` unchanged: `ring`, `outline`, `text_halo`,
`glowing`, `glow`, `tint`, `swelling` and `swell_all`. A `frame_halo(t)` on
`frame_at(t)` would take the place of `slot_halo(i)`.

## Chapters 04 and 05, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment after Mail 6872 (root's `tend-0818/shamir-next04-05.txt`)
and Mail 6873. The ask:
- write new native chapters `04_the_goal` and `05_an_experiment` from the
  accepted draft storyboard, with the same vault, trustees and shares;
- keep the dealer's method sealed, labelled as not yet explained;
- in 05, compute the 11 × 11 fair-spin grid and each secret's fitting cell
  from the model;
- keep the all-11-secret strip and the conditional-probability statement
  true for an arbitrary prior;
- hold back the fitting tally until after 05_2's prediction;
- keep readable numbers apart from emphasis shapes, with no overlapping
  different-digit crossfades;
- give the explanation and inspection generous time, with no runtime cap;
- write only new unregistered files and this file, and propose everything
  shared in an exact integration note.

**Status.**
- I wrote three new files.
- Root has integrated 02/03 meanwhile. I edited no registered or delivered
  file: `the_vault.rs`, `two_keys.rs`, `cutting_it_up.rs`, `the_dial.rs`,
  `draw.rs`, `palette.rs`, `glyphs.rs`, `mod.rs`, `model.rs`, `facts.rs`,
  `main.rs`, `oracle.py`, the scripts, the storyboards and Cargo.
- Nothing was compiled, rendered, synthesized or listened to.

**Reservations.**
- Held: 7939 (`the_goal.rs`), 7940 (`an_experiment.rs`), 7941 (this file),
  7942 (`docs/previous/ch0405-*/**`) and 7970 (`envelope.rs`).
- Released at 08:30:11Z: the 02/03 leases 7852–7855 and 7898.

**Prior bytes.** `docs/previous/ch0405-before-04-05-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section, sha256 6ee2b369….

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `envelope.rs` | 441 | d724380a5b6b1abe014acf7835ae9b5ae904940499f1a3650fddab00c6c22901 |
| `the_goal.rs` | 546 | 26e6d843fffadf0428a0a0b0e95288cf803fc5ace096a60bb1d1b585a95819c9 |
| `an_experiment.rs` | 799 | 5de3cd868b70d7aacf7a0b675ade9b56678e00126aa3c005384a2913931817db |

- **`envelope.rs`**: the devices 04 and 05 share.
  - The worked example's dealt shares and views (`dealt`, `holding`).
  - The captions `wanted_pair` and `for_pair`.
  - The sealed `Envelope`: a body, a V flap and a seal, with "the dealer's
    method" over "sealed until chapters 07 to 09". Its `halo` is an outline
    of its own.
  - `going_in`, and the unnumbered yellow `Spinner` dials.
  - Emphasis shapes: `outline`, `frame_halos`, `swelling` on shapes only,
    `door_glow`, the lamp's `alarm`, and `fills`.
  - 4 tests.
- **`the_goal.rs`**: `TheGoal`, key `04_the_goal`, 4 tests.
- **`an_experiment.rs`**: `AnExperiment`, key `05_an_experiment`, 7 tests.
  - These include a planted negative (a secret not fitting exactly once is
    an error).
  - They also check that the chance moves no belief under four integer
    priors, by exact cross-multiplication.

Both scenes' phrase tests read `crate::SCRIPT`, as root made the_dial's.
Registering the chapters brings their lines into `script.tsv`, so my files
need no edit at integration.

### How the brief is met

- **The same example.** The shares come from `Recipe::example` through the
  model: 8, 3, 1, 2, 6. 04 checks that the trio's view leaves 5 alone
  before it draws anything; 05 checks it before its closing "3 shares →
  the secret". Both stop with an error otherwise. The pair's 2 and 6 are
  dealt, never typed.
- **The method stays sealed.** The envelope never opens.
  - In 04_1 the gold 5 and two spin dials go into it. The dials stop on the
    example's 7 and 7, but carry no numbers.
  - Its outline glows at "hands out five shares" and again at "How they do
    it comes later".
  - In 05 it is fed every secret and glows at "Feed the dealer's method".
    At "The rest of the story" it glows and stays shut.
- **05 computes the experiment before drawing it.**
  - The grid is `model::tuples(dial, 2)`: 121 cells. Each secret's cells are
    `model::fitting_spins` for the pair's dealt shares.
  - `one_cell_each` errors unless every secret fits exactly one cell and no
    two secrets share one.
  - The scene also errors unless the chance, ways over 121, is the same for
    every secret.
  - After the count, the strip shows `holding(PAIR).possible`, and
    `check_strip` compares it with the grid's result.
- **True for any prior.**
  - 05_4's card is the likelihood: "with fair spins, P(shares 2 and 6 | s)
    = 1/121 for every secret s, so their belief about s is unchanged".
  - The strip's 11 lit ticks mean "still possible", not "equally likely".
    Nothing in 05 says "1 in 11 each".
  - 04's "guess: 1 in 11" and the demand card's "guessing stays 1 in 11"
    are the draft's words. They are exact for a uniformly random guess
    under any prior. With an uneven prior, the best guess's chance is the
    prior's largest value, and the shares leave that unchanged too. The
    evidence below checks both.
  - If root wants the card to say which guess it means, "a blind guess:
    1 in 11" is a one-string change (`guess_card`).
- **The tally waits.**
  - At "count the ways of spinning", empty slots appear under the ticks,
    titled "ways that deal 2 and 6". A yellow bar sweeps the columns and
    lights nothing.
  - At "a two and a six", slot 0 shows "?" and the think ring comes.
  - In 05_3, the ring and the "?" leave before the first count comes in.
  - The strip keeps chapter 04's "wanted" caption until "So the pair's two
    and six".
- **A deviation from the draft, in 04_3.**
  - The draft gives the pair's strip the caption "for: trustees 4 and 5"
    over a full strip. That shows 05's answer, "two shares leave all
    eleven", before 05_2 asks the question.
  - The scene uses 01_5's goal caption, "wanted, for trustees 4 and 5"
    (`wanted_pair`).
  - To revert it, call `for_pair` at `the_goal.rs:274`.
  - The proposed storyboard rows below describe the scene as written.
- **Numbers kept apart from emphasis.**
  - No `indicate`, recolor, style change or scale ever touches typeset
    text.
  - Text is pointed at by outlines of its own that fade in and out: the
    card frames' halos, the strip's outline, the tally row, the axis
    titles, the bubbles and the envelope.
  - Only shapes swell: the door, the lamp, the trio's discs and the
    counter's outline. Only discs, the door and the lamp change color.
  - The alarm's swell is tinted with the leak red, not the default yellow,
    because yellow is the spin role here.
- **No different digits cross at one place.**
  - The secret counter is an outline that moves along the strip's
    unchanging labels.
  - Each lit cell's number and each tally "1" comes in where nothing was.
  - Slot 0's "?" is gone, in a separate play, before its "1" comes in.
  - The spin dials carry no numbers.
  - What still changes in place is root's `Strip` machinery:
    - the live count tweens 11 → 1 → 11 in 04_2 (one number, re-set each
      frame);
    - its captions crossfade in 04_1, twice in 04_2, and in 04_3, 05_1 and
      05_3.
  - The 05_3 caption change crossfades "wanted, for trustees 4 and 5" into
    "for: trustees 4 and 5". The 4 and the 5 sit at different x in the two
    captions. Not checked by eye.
- **Generous time, no cap.**
  - 1.0 s holds on the open door (04_2) and on the finished grid (05_3).
  - `read_by` waits for the demand and chance cards.
  - Both questions get the think ring.
  - In 05_3 each secret's step is 0.4 to 0.8 s, timed to finish when "So
    the pair's two and six" begins.
  - On the draft's words alone, that sentence gives about 4.5 s, so steps
    of about 0.41 s, near the floor. Hence the proposed hold after "a
    different way each time." below: with 3 to 4 s of hold, the steps
    reach about 0.7 to 0.8 s.
  - This assumes the narrator's sentence onsets include inserted holds.
    `Spoken::at` reads them.

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

The scratch scripts named below live in this session's scratchpad, not in
the repository. Their full outputs are quoted in my handoff mail.

- **Formatting.** `rustfmt --check --edition 2024 --config-path rustfmt.toml`
  gives rc 0 on each of the three files, and on the live `main.rs`. This is
  a parse check only.
- **The module tree with 04/05 registered.** On a scratch copy of the topic
  with the patch below applied, `rustfmt --check` on `main.rs` gives rc 0.
  That run parses every module, including the patched `facts.rs`.
- **Cues** (`check_cues.py TOPIC 04:the_goal 05:an_experiment`).
  - Each scene says its five lines in order.
  - 18 cue phrases in 04 (17 distinct) and 16 in 05 (15 distinct) occur in
    their lines, in spoken order.
  - Each test list equals the code's.
  - 0 failures, rc 0.
- **Independent arithmetic** (`evidence.py`). This is brute force written
  from the storyboard's statements, not from `oracle.py` or the Rust model.
  It has 34 checks, 0 failures, rc 0:
  - the shares of 5 + 7x + 7x² mod 11 at 1..5 are 8, 3, 1, 2, 6;
  - of all 1331 recipes, only (5, 7, 7) deals the trio's 8, 3, 1;
  - the pair's 2 and 6: for each secret exactly one of the 121 spin pairs
    fits. The 11 cells are all different, with one in every row and every
    column. The true secret's cell is (7, 7), and there are 11 in all;
  - cells by secret 0..10: 1:4, 0:9, 10:3, 9:8, 8:2, 7:7, 6:1, 5:6, 4:0,
    3:5, 2:10;
  - P(2, 6 | s) = 1/121 for every s;
  - the posterior equals the prior exactly (`Fraction`) for 204 priors:
    uniform, all mass on 0, no mass on 5, geometric, and 200 seeded random;
  - a uniformly random guess is right 1/11 of the time under every prior,
    and the best guess's chance is unchanged by the shares;
  - generalised: every pair of trustees and every one of the 121 share
    values they could hold deal each secret the same number of ways;
  - **negative controls.**
    - If the second spin is never 0, secret 8 is ruled out, and the uniform
      prior moves to 1/10.
    - On a ten-mark dial the pair's shares 5, 5 fit only secret 5.
  - 12 values equal the fresh oracle's printing. The four proposed facts
    are absent from it.
- **The oracle, fresh.** On the live tree it prints 192 facts, with stderr
  `# 192 facts, 0 failed cross-checks`, rc 0. Its output's sha256 is
  6ad37cfff23c30560d73b7491a055658de654994af17ff40ee91d5dcf350a4cf. On the
  patched scratch tree it prints `# 196 facts, 0 failed cross-checks`,
  rc 0.
- **The golden table** (`check_golden.py`, mirroring `oracle_diff` and
  `f.len()`).
  - Live: 192 rows, no diff.
  - Patched: 196 rows, distinct 196, against the patched oracle's 196, with
    `f.len()` asserted 196. No diff.
- **Spoken numbers.**
  - `check_spoken.py` (04/05 rows only) has 26 rows, 1 idiom and 45 number
    words in 04/05. Problems: none, rc 0.
  - Planted negatives each give rc 1 and name the uncovered word or the
    wrong value: dropping row 11 ("ten", "times"), dropping row 14 ("one"),
    and corrupting row 18's value.
  - The whole-table checker on the patched tree has 69 rows and 6 idioms
    against its `script.tsv`. Problems: none, rc 0.
- **Storyboards** (copies of `tools/storyboard.py` in scratch trees, which
  write only there).
  - Regeneration: 04 is estimated at 60.7 s and 05 at 58.1 s, 6.0 min
    over 5 chapters. rc 0.
  - `--check` on the patched tree gives rc 0 and 0 PROBLEM lines.
  - `check_sb.py` gives 0 failures:
    - production chapters equal the draft's, row for row;
    - only 04/05 changed in the draft, and only in their picture, features
      and hold cells;
    - the draft's words, and so `docs/script-draft.tsv`, are unchanged;
    - the regenerated `script.tsv` is the live 20 lines plus exactly the
      draft's ten 04/05 lines.
- **The patch.** `git apply --check` from the repository root gives rc 0.
  Nothing was applied.
- **Static audit.**
  - In the three files, `indicate` appears only inside `swelling`, which
    is given the door, the lamp, the trio's discs and the counter's outline.
  - Every recolor targets discs, the door or the lamp, because
    `Rail::tint` returns discs.

### Not verified

- **Nothing compiled.** No Cargo, rustc or clippy-driver; all 15 new tests
  are unrun.
  - Clippy's view of the casts and the const blocks is unknown.
  - Borrow and type errors are possible.
- **Nothing rendered or watched.** The engine behaviours I rely on, read
  from the pinned source but not run:
  - `set_fill` opacity on the trio's discs (`fills`);
  - outline glows as a `Succession` of `fade_in` and `fade_out`;
  - `fade_in` with a shift (the rising bubbles);
  - `fade_out` toward the envelope's mouth after `clear_updaters` (the
    spinners' needles);
  - `rush_from` on the spins;
  - a linear `move_to` for the sweep.
- **Layout is estimated.** The layout tests use estimated text sizes:
  0.0055 units a point per character wide and 0.0065 a point tall.
- **Glyphs are unprobed.** The cards use "×", "→", "|", "(" and ")". The
  kit's `glyphs` probe captures the actual scene strings, but it renders,
  so it is root's to run.
- **No listening.** Nothing here claims accent consistency. No timing is
  measured; every duration above is an estimate.

### For root's integration (a proposal; I edited none of these files)

Apply as one unit, together with `script.tsv` regenerated by
`tools/storyboard.py shamir_secret_sharing` (the patch includes its result).
The reason: `main.rs`'s `check_video` compares the registry with the
storyboard and the script.

- **`src/chapters/mod.rs`.**
  - `pub mod an_experiment;`, `pub mod envelope;` and `pub mod the_goal;`.
  - `04_the_goal` → `the_goal::TheGoal` and `05_an_experiment` →
    `an_experiment::AnExperiment`, each at intensity 0.5.
  - The doc reads "chapters 01 to 05 so far. The other eighteen …".
- **`docs/storyboard.md` and `docs/storyboard-draft.md`.**
  - Identical 04/05 rows describing the scenes as written. The words are
    unchanged. Two hold placements change (below), and the 04_3 caption is
    the "wanted" one.
  - Production also gets "chapters 01 to 05" in its intro and a short
    "Devices in chapters 04 and 05" section.
  - In the draft, nothing outside 04/05 changes.
- **`src/main.rs`.** 26 `SPOKEN` rows and one `NOT_A_COUNT` row, ("05_1",
  "every one of the").
- **`src/facts.rs` and `oracle.py`.** Four new facts, each computed on both
  sides independently:

  | Key | Value | Rust (model) | Python (oracle) |
  |---|---|---|---|
  | `S-F6.spin_count` | 2 | `SPINS.len()` | `len(SPINS)` |
  | `S-F7.secrets` | 0,1,…,10 | `dial.all()` | `range(P)` |
  | `S-F7.ruled_out` | 0 | marks − the pair's `View::Shares::possible` | secrets whose `fitting_spins` list is empty |
  | `S-F12.guess_misses` | 10 | marks ≠ `SECRET` | marks ≠ `SECRET` |

  Also four golden rows, `f.len()` 192 → 196, and the module doc's "Only
  chapter 01 is implemented" becomes "Chapters 01 to 05".
- **Pause intentions** (hold/listen cells; the pause map is root's):
  - 04_1: thought after "where we're headed."
  - 04_2: clause after "eight, three and one,"; hold on the open door. The
    scene adds 1.0 s after the line.
  - 04_3: thought after "they can guess."
  - 04_4: thought after "can never get lucky."
  - 04_5 and 05_2: question holds. The scene's think ring stays the
    kit's `THINK_S` 4 s.
  - 05_1: thought after "Here's an experiment."
  - 05_3: thought after "No."; **hold on the grid after "a different way
    each time."** This is moved from the line's end so the count gets room
    (see above). The scene still adds 1.0 s after the line.
  - 05_4: thought after "what to guess."
  - 05_5: chapter_transition.
- **Dependencies already in the live tree:** `palette::spin()`,
  `draw::FLASH` and the `pub` draw constants from 02/03.
- **Offers only:**
  - `Vault::alarm` could take the same leak tint as `envelope::alarm`.
  - `two_keys`'s emphasis helpers and `envelope.rs`'s `outline`/`swelling`
    could move to `draw.rs` together.

To extract the patch and apply it:

~~~sh
awk '/^<!-- shamir-0405 patch end -->$/{p=0} p; /^<!-- shamir-0405 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-0405.patch
git apply --check shamir-0405.patch && git apply shamir-0405.patch
~~~

<!-- shamir-0405 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,14 +1,17 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 03 so far. The other
-//! twenty are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 05 so far. The other
+//! eighteen are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
+pub mod an_experiment;
 pub mod cutting_it_up;
 pub mod draw;
+pub mod envelope;
 pub mod glyphs;
 pub mod the_dial;
+pub mod the_goal;
 pub mod the_vault;
 pub mod two_keys;
 
@@ -31,5 +34,15 @@
             make: |kit| Box::new(the_dial::TheDial { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "04_the_goal",
+            make: |kit| Box::new(the_goal::TheGoal { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "05_an_experiment",
+            make: |kit| Box::new(an_experiment::AnExperiment { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -114,6 +114,34 @@
         ("03_5", "all eleven secrets", "S-F2.distinct_secrets", "11"),
         ("03_5", "once each", "S-F2.ways_each", "1"),
         ("03_6", "B's seven", "S-F2.b_share", "7"),
+        ("04_1", "Back to five trustees", "S-F1.trustees", "5"),
+        ("04_1", "a threshold of three", "S-F1.threshold", "3"),
+        ("04_1", "two fair spins", "S-F6.spin_count", "2"),
+        // a share for each trustee
+        ("04_1", "hands out five shares", "S-F1.trustees", "5"),
+        ("04_1", "eight, three, one, two and six", "S-F6.shares", "8,3,1,2,6"),
+        ("04_2", "Trustees one, two and three", "S-F1.opening_trio", "1,2,3"),
+        ("04_2", "their eight, three and one", "S-F1.opening_shares", "8,3,1"),
+        ("04_2", "the secret, five", "S-F1.secret", "5"),
+        ("04_3", "Trustees four and five", "S-F1.pair", "4,5"),
+        ("04_3", "hold two and six", "S-F1.pair_shares", "2,6"),
+        ("04_3", "one time in eleven", "S-F12.guess_chance", "1/11"),
+        ("04_3", "the other ten times", "S-F12.guess_misses", "10"),
+        ("04_4", "two trustees can never", "S-F1.too_few", "2"),
+        ("04_4", "their two shares", "S-F1.too_few", "2"),
+        // "not one" mark: none
+        ("04_4", "not one mark ruled out", "S-F7.ruled_out", "0"),
+        ("04_5", "Could two shares", "S-F1.too_few", "2"),
+        ("04_5", "when three open", "S-F1.threshold", "3"),
+        ("05_1", "from zero to ten", "S-F7.secrets", "0,1,2,3,4,5,6,7,8,9,10"),
+        ("05_1", "hundred and twenty-one ways", "S-F6.spin_pairs", "121"),
+        ("05_1", "its two spins", "S-F6.spin_count", "2"),
+        ("05_2", "trustees four and five", "S-F1.pair", "4,5"),
+        ("05_2", "a two and a six", "S-F1.pair_shares", "2,6"),
+        ("05_3", "exactly one way", "S-F7.ways_each", "1"),
+        ("05_3", "two and six fit", "S-F1.pair_shares", "2,6"),
+        ("05_5", "Two shares can't", "S-F1.too_few", "2"),
+        ("05_5", "and three can", "S-F1.threshold", "3"),
     ];
 
     /// Number words said as an article, not a count, each with its line.
@@ -123,6 +151,7 @@
         ("02_4", "less than zero"), // the definition of a nonnegative piece
         ("02_6", "a second way"),
         ("02_7", "fixed first"),
+        ("05_1", "every one of the"),
     ];
 
     const NUMBER_WORDS: &[&str] = &[
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -6,7 +6,7 @@
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
 //! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper and S-F7.join
 //! 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11 19 and 20,
-//! S-F12 21 and 22, and S-F13 23. Only chapter 01 is implemented so far; the
+//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 05 are implemented so far; the
 //! rest are computed and checked now so the draft's numbers are settled
 //! before its scenes exist.
 
@@ -196,6 +196,7 @@
     f.add("S-F6.heights", csv(&heights))?;
     f.add("S-F6.shares", csv(&shares))?;
     f.add("S-F6.spin_pairs", model::tuples(dial, SPINS.len()).count())?;
+    f.add("S-F6.spin_count", SPINS.len())?;
 
     // 05, 10: two shares, one spin pair per secret
     let pair_points = dealt(dial, &recipe, &PAIR);
@@ -215,6 +216,15 @@
         .find(|&s| per_secret[s as usize][1] == 0)
         .ok_or("no secret's curve through the pair is a line")?;
     f.add("S-F7.straight_one", straight)?;
+    // 04 and 05: every secret the experiment feeds in, and the marks the
+    // pair's two shares rule out
+    f.add("S-F7.secrets", csv(&dial.all().collect::<Vec<_>>()))?;
+    let pair_left = View::Shares {
+        points: pair_points.clone(),
+        spins: SPINS.len(),
+    }
+    .possible(dial);
+    f.add("S-F7.ruled_out", dial.marks() as usize - pair_left.len())?;
     // 11: the same pair on ordinary paper
     let paper = [PAIR[0], PAIR[1]].map(|x| (x, recipe.height(x)));
     f.add("S-F7.paper.heights", csv(&paper.map(|p| p.1)))?;
@@ -473,6 +483,7 @@
 
     // 04 (the guess), 21: what it promises, 22: a liar
     f.add("S-F12.guess_chance", Ratio::new(1, i64::from(MARKS)))?;
+    f.add("S-F12.guess_misses", dial.all().filter(|&s| s != SECRET).count())?;
     let excluded: Vec<u32> = dial.all().filter(|&s| per_secret[s as usize][1] == 0).collect();
     f.add("S-F12.nonzero_last.excluded", csv(&excluded))?;
     f.add("S-F12.nonzero_last.left", dial.marks() as usize - excluded.len())?;
@@ -627,12 +638,15 @@
 S-F6.heights	19,47,89,145,215
 S-F6.shares	8,3,1,2,6
 S-F6.spin_pairs	121
+S-F6.spin_count	2
 S-F7.fitting	11
 S-F7.ways_each	1
 S-F7.spins_by_secret	1:4,0:9,10:3,9:8,8:2,7:7,6:1,5:6,4:0,3:5,2:10
 S-F7.chance_each	1/121
 S-F7.true_spins	7:7
 S-F7.straight_one	8
+S-F7.secrets	0,1,2,3,4,5,6,7,8,9,10
+S-F7.ruled_out	0
 S-F7.paper.heights	145,215
 S-F7.paper.curves	11
 S-F7.paper.whole_spin_secrets	5
@@ -723,6 +737,7 @@
 S-F11.ten.ghost.zeros	1,3,5,7,9
 S-F11.ten.moved_recipe	0,2,7
 S-F12.guess_chance	1/11
+S-F12.guess_misses	10
 S-F12.nonzero_last.excluded	8
 S-F12.nonzero_last.left	10
 S-F12.liar	3
@@ -755,7 +770,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 192);
+        assert_eq!(f.len(), 196);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -311,6 +311,7 @@
     put("S-F6.heights", csv(SECRET + SPINS[0] * x + SPINS[1] * x * x for x in TRUSTEES))
     put("S-F6.shares", csv(shares))
     put("S-F6.spin_pairs", P * P)
+    put("S-F6.spin_count", len(SPINS))
 
     # 05, 10: two shares
     pv = [shares[x - 1] for x in PAIR]
@@ -322,6 +323,9 @@
     put("S-F7.true_spins", ":".join(str(v) for v in per_secret[SECRET][0]))
     line_secret = next(s for s in range(P) if per_secret[s][0][1] == 0)
     put("S-F7.straight_one", line_secret)
+    # 04, 05: the secrets fed in, and those no spin pair deals the pair's shares with
+    put("S-F7.secrets", csv(range(P)))
+    put("S-F7.ruled_out", sum(1 for ways in per_secret if not ways))
     # the same pair on ordinary paper: a curve through every secret, but whole spins 0 to 10 for one
     tall_pair = [SECRET + SPINS[0] * x + SPINS[1] * x * x for x in PAIR]
     paper_curves = []
@@ -511,6 +515,7 @@
 
     # 04 (the guess), 21: what it promises, 22: a liar
     put("S-F12.guess_chance", frac(Fraction(1, P)))
+    put("S-F12.guess_misses", sum(1 for g in range(P) if g != SECRET))
     allowed = [s for s in range(P) if any(b != 0 for _, b in per_secret[s])]
     put("S-F12.nonzero_last.excluded", csv(sorted(set(range(P)) - set(allowed))))
     put("S-F12.nonzero_last.left", len(allowed))
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 03, per root's direction in Mail
+chapters 01 to 05, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -67,3 +67,36 @@
 | 03_6 | A's mark came from a fair spin, so B's seven fits every secret equally well. A circle has no floor and no ceiling, so a share can't push the secret above or below anything. | Split view, same B: the ordinary number line with B's piece 3 acting as a floor (strip 8), beside the dial with B's 7 (strip 11). precisely: "for every secret s, exactly one of A's 11 equally likely marks gives B a 7: P(B = 7 \| s) = 1/11" [S-F2.chance_each]. | split view, precisely card | thought after "equally well." |
 | 03_7 | That fixes the leak, but this vault still needs both trustees. | Green "B's share leaks nothing" under the strip, red "still needs A and B both" beside the closed small door. A and B fade; the big vault and five-trustee rail return, with "threshold 3". | morph | chapter_transition |
 
+## Devices in chapters 04 and 05
+
+- **The envelope**, upper right: the dealer's method, sealed until chapters
+  07 to 09. The secret and two spins go in, and shares come out; it never
+  opens here. Its outline glows when the words point at the method.
+- **Spin dials**, two small unnumbered yellow dials (the spin role) that
+  spin and stop on the example's spins before going into the envelope.
+- **The spin grid**, upper left in the vault's place: 11 × 11 faint yellow
+  cells, first spin across and second up, from `model::tuples`.
+- **The counter and tally**: a gold outline marked "trying" that moves
+  along the strip's own labels, and an empty slot under each tick that gets
+  its count only after 05_2's question. No number changes in place.
+
+## 04_the_goal: what two shares must not do
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 04_1 | Back to five trustees and a threshold of three. Here is where we're headed. Using the secret and two fair spins, the dealer hands out five shares, eight, three, one, two and six. | Big vault, trustee rail and strip as in chapter 01, the strip captioned "for: anyone outside". At "a threshold of three" the card "threshold 3" [S-F1.threshold]. At "Using the secret" a sealed envelope at the upper right, "the dealer's method" over "sealed until chapters 07 to 09", with the gold 5 and two small unnumbered yellow spin dials beside it. At "two fair spins" the dials spin and stop on the example's spins [S-F6.spins, S-F6.spin_count]; the 5 and both dials slide into the envelope before "hands out five shares", where the envelope's outline glows. At "eight, three, one, two and six" the cards fill 1 → 8, 2 → 3, 3 → 1, 4 → 2, 5 → 6, one after another [S-F6.shares]. | lagged cards, spin, outline glow | thought after "where we're headed." |
+| 04_2 | Trustees one, two and three bring their eight, three and one, work out the secret, five, and the door opens. How they do it comes later. | Trustees 1, 2 and 3 teal. At "bring their eight, three and one" their cards 8, 3, 1 [S-F1.opening_shares] slide into an outlined box beside the dial, a "?" over it. At "work out the secret, five" the needle turns 0 → 5 [S-F1.secret] and, in the same move, the strip shows what their three shares leave, 5 alone, captioned "for: trustees 1, 2 and 3". At "the door opens" the door turns green and swells. At "How they do it comes later" the envelope's outline glows. After the hold, in one move, the cards go home, the trio turns blue, the door shuts, the needle returns to 0 and the strip relights all eleven "for: anyone outside". | morph, swell, outline glow | clause after "eight, three and one,"; hold on the open door |
+| 04_3 | Trustees four and five hold two and six. Like anyone, they can guess. They'd be right one time in eleven, and the other ten times the alarm goes off. | Trustees 4 and 5 purple with 2 and 6 [S-F1.pair_shares]. The strip stays full with their goal caption, "wanted, for trustees 4 and 5", as in 01_5: what two shares actually leave is chapter 05's result. At "hold two and six" outlines glow round their two cards. At "They'd be right one time in eleven" a gamble card "guess: 1 in 11" [S-F12.guess_chance]. At "the other ten times" [S-F12.guess_misses] the needle turns to 3, the alarm lamp flashes red, and the needle returns to 0. | highlight, flash | thought after "they can guess." |
+| 04_4 | So the demand isn't that two trustees can never get lucky. It's that their two shares tell them nothing, not one mark ruled out, not even a hint that the secret is odd. | The gamble card goes. At "It's that their two shares tell them nothing" a "the demand:" lower third, "two shares cannot determine the secret, or narrow it; guessing stays 1 in 11", while the outlines round the pair's cards glow. At "not one mark ruled out" [S-F7.ruled_out] an outline glows round the full strip and a bubble "not 3?" rises over the pair; at "not even a hint that the secret is odd" a bubble "odd?" rises. The bubbles drift up and fade, unanswered; the strip stays full. | card, highlight | thought after "can never get lucky." |
+| 04_5 | ⟨predict⟩ Is that even possible? Could two shares be completely useless, when three open the vault? | The lower third goes; trustees 1, 2 and 3 faint teal, 4 and 5 purple. At "Could two shares be completely useless" the outlines round the pair's cards glow; at "when three open the vault" the trio's discs swell. The strip stays full. The think ring with its "?" at the lower right. | predict | question hold (think time) |
+
+## 05_an_experiment: every secret still fits
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 05_1 | Here's an experiment. Feed the dealer's method every secret from zero to ten, with every one of the hundred and twenty-one ways its two spins could land. | The rail with trustees 4 and 5 purple holding 2 and 6 [S-F1.pair_shares], the strip still captioned "wanted, for trustees 4 and 5", and the sealed envelope, whose outline glows at "Feed the dealer's method". At "every secret from zero to ten" [S-F7.secrets] a gold counter, an outline marked "trying", appears round the strip's 0 and glides to 10 and back; the strip's own labels never change. At "hundred and twenty-one ways" the 11 × 11 spin grid comes in column by column at the upper left, in the vault's place, faint yellow, first spin across and second up, numbered 0 to 10 on both axes, "11 × 11 = 121 spin pairs" over it [S-F6.spin_pairs]. At "its two spins could land" outlines glow round the two axis titles. | grid, lagged, counter | thought after "Here's an experiment." |
+| 05_2 | For each secret, count the ways of spinning that hand trustees four and five exactly what they hold, a two and a six. ⟨predict⟩ Will some secrets fit more ways than others? | At "For each secret" the counter, at 0, swells. At "count the ways of spinning" an empty tally slot appears under each tick, titled "ways that deal 2 and 6", and a yellow bar sweeps the grid's columns once, lighting nothing. At "a two and a six" the outlines round the pair's cards glow and slot 0 shows "?". No cell or count is shown before the question; the think ring with its "?" at the lower right. | predict | question hold (think time) |
+| 05_3 | No. Every secret fits exactly one way, and a different way each time. So the pair's two and six fit every secret equally well. | The think ring and the waiting "?" go before any count. From "Every secret fits exactly one way" the counter steps 0 → 10; for each secret one cell lights gold with that secret's number in it, a different cell each time, one in every row and every column [S-F7.spins_by_secret], and its tally slot gets a 1 [S-F7.ways_each]: 11 cells in all [S-F7.fitting]. At "So the pair's two and six" the counter goes, the outlines round the pair's cards glow, and the strip, now computed from their two shares, shows all eleven, captioned "for: trustees 4 and 5". | grid highlight, live counter | thought after "No."; hold on the grid after "a different way each time." |
+| 05_4 | They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met. | precisely: "with fair spins, P(shares 2 and 6 \| s) = 1/121 for every secret s, so their belief about s is unchanged" [S-F7.chance_each]. At "every secret is exactly as likely" an outline glows round the tally row of 1s. | precisely card | thought after "what to guess." |
+| 05_5 | Two shares can't determine the secret, and three can. The rest of the story is why both are true. | The card goes. At "Two shares can't determine the secret" the line "2 shares → all 11 secrets still fit"; at "and three can" the line "3 shares → the secret" above it. The envelope stays sealed; its outline glows at "The rest of the story". | card | chapter_transition |
+
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -207,21 +207,21 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 04_1 | Back to five trustees and a threshold of three. Here is where we're headed. Using the secret and two fair spins, the dealer hands out five shares, eight, three, one, two and six. | Big vault, trustee rail. A sealed envelope marked "the dealer's method (chapters 07 to 09)" with the gold 5 and two yellow spin dials going in; cards fill 1 → 8, 2 → 3, 3 → 1, 4 → 2, 5 → 6 [S-F6.shares]. | lagged cards | thought after "where we're headed." |
-| 04_2 | Trustees one, two and three bring their eight, three and one, work out the secret, five, and the door opens. How they do it comes later. | Trustees 1, 2, 3 teal; their cards 8, 3, 1 [S-F1.opening_shares] slide to a box "?" beside the dial; the needle sweeps 0 → 5 [S-F1.secret]; the door glows green and swings open. | morph, flash | clause after "eight, three and one,"; hold on the open door |
-| 04_3 | Trustees four and five hold two and six. Like anyone, they can guess. They'd be right one time in eleven, and the other ten times the alarm goes off. | Door shut; trustees 4 and 5 purple with 2 and 6 [S-F1.pair_shares]; the strip's caption reads "for: trustees 4 and 5", strip full. A gamble card "guess: 1 in 11" [S-F12.guess_chance]; the alarm lamp flashes red. | highlight | thought after "they can guess." |
-| 04_4 | So the demand isn't that two trustees can never get lucky. It's that their two shares tell them nothing, not one mark ruled out, not even a hint that the secret is odd. | Thought bubbles "odd?", "not 3?" rise over the pair, then fade; the strip stays full. Card: "two shares cannot determine the secret, or narrow it; guessing stays 1 in 11". | card | thought after "can never get lucky." |
-| 04_5 | ⟨predict⟩ Is that even possible? Could two shares be completely useless, when three open the vault? | The strip with a "?" over it; trustees 1, 2, 3 faint teal on one side, 4 and 5 purple on the other. | predict | question hold (think time) |
+| 04_1 | Back to five trustees and a threshold of three. Here is where we're headed. Using the secret and two fair spins, the dealer hands out five shares, eight, three, one, two and six. | Big vault, trustee rail and strip as in chapter 01, the strip captioned "for: anyone outside". At "a threshold of three" the card "threshold 3" [S-F1.threshold]. At "Using the secret" a sealed envelope at the upper right, "the dealer's method" over "sealed until chapters 07 to 09", with the gold 5 and two small unnumbered yellow spin dials beside it. At "two fair spins" the dials spin and stop on the example's spins [S-F6.spins, S-F6.spin_count]; the 5 and both dials slide into the envelope before "hands out five shares", where the envelope's outline glows. At "eight, three, one, two and six" the cards fill 1 → 8, 2 → 3, 3 → 1, 4 → 2, 5 → 6, one after another [S-F6.shares]. | lagged cards, spin, outline glow | thought after "where we're headed." |
+| 04_2 | Trustees one, two and three bring their eight, three and one, work out the secret, five, and the door opens. How they do it comes later. | Trustees 1, 2 and 3 teal. At "bring their eight, three and one" their cards 8, 3, 1 [S-F1.opening_shares] slide into an outlined box beside the dial, a "?" over it. At "work out the secret, five" the needle turns 0 → 5 [S-F1.secret] and, in the same move, the strip shows what their three shares leave, 5 alone, captioned "for: trustees 1, 2 and 3". At "the door opens" the door turns green and swells. At "How they do it comes later" the envelope's outline glows. After the hold, in one move, the cards go home, the trio turns blue, the door shuts, the needle returns to 0 and the strip relights all eleven "for: anyone outside". | morph, swell, outline glow | clause after "eight, three and one,"; hold on the open door |
+| 04_3 | Trustees four and five hold two and six. Like anyone, they can guess. They'd be right one time in eleven, and the other ten times the alarm goes off. | Trustees 4 and 5 purple with 2 and 6 [S-F1.pair_shares]. The strip stays full with their goal caption, "wanted, for trustees 4 and 5", as in 01_5: what two shares actually leave is chapter 05's result. At "hold two and six" outlines glow round their two cards. At "They'd be right one time in eleven" a gamble card "guess: 1 in 11" [S-F12.guess_chance]. At "the other ten times" [S-F12.guess_misses] the needle turns to 3, the alarm lamp flashes red, and the needle returns to 0. | highlight, flash | thought after "they can guess." |
+| 04_4 | So the demand isn't that two trustees can never get lucky. It's that their two shares tell them nothing, not one mark ruled out, not even a hint that the secret is odd. | The gamble card goes. At "It's that their two shares tell them nothing" a "the demand:" lower third, "two shares cannot determine the secret, or narrow it; guessing stays 1 in 11", while the outlines round the pair's cards glow. At "not one mark ruled out" [S-F7.ruled_out] an outline glows round the full strip and a bubble "not 3?" rises over the pair; at "not even a hint that the secret is odd" a bubble "odd?" rises. The bubbles drift up and fade, unanswered; the strip stays full. | card, highlight | thought after "can never get lucky." |
+| 04_5 | ⟨predict⟩ Is that even possible? Could two shares be completely useless, when three open the vault? | The lower third goes; trustees 1, 2 and 3 faint teal, 4 and 5 purple. At "Could two shares be completely useless" the outlines round the pair's cards glow; at "when three open the vault" the trio's discs swell. The strip stays full. The think ring with its "?" at the lower right. | predict | question hold (think time) |
 
 ## 05_an_experiment: every secret still fits
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 05_1 | Here's an experiment. Feed the dealer's method every secret from zero to ten, with every one of the hundred and twenty-one ways its two spins could land. | Beside the pair's strip, the 11 × 11 spin grid in yellow [S-F6.spin_pairs], first spin across, second up; a secret counter at 0. | grid | thought after "Here's an experiment." |
-| 05_2 | For each secret, count the ways of spinning that hand trustees four and five exactly what they hold, a two and a six. ⟨predict⟩ Will some secrets fit more ways than others? | The counter at 0; the method sweeps all 121 cells and a tally waits beside "secret 0". | predict | question hold (think time) |
-| 05_3 | No. Every secret fits exactly one way, and a different way each time. So the pair's two and six fit every secret equally well. | The counter steps 0 → 10; for each secret one cell lights, a different one each time [S-F7.ways_each, S-F7.fitting]; that secret's tick on the strip pulses, until all 11 have pulsed once; the tally column reads 1 for every secret. | grid highlight, live counter | thought after "No."; hold on the grid |
-| 05_4 | They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met. | precisely: "with fair spins, P(shares 2 and 6 \| s) = 1/121 for every secret s, so their belief about s is unchanged" [S-F7.chance_each]. | precisely card | thought after "what to guess." |
-| 05_5 | Two shares can't determine the secret, and three can. The rest of the story is why both are true. | Two-line card: "3 shares → the secret" / "2 shares → all 11 secrets still fit". The envelope stays sealed. | card | chapter_transition |
+| 05_1 | Here's an experiment. Feed the dealer's method every secret from zero to ten, with every one of the hundred and twenty-one ways its two spins could land. | The rail with trustees 4 and 5 purple holding 2 and 6 [S-F1.pair_shares], the strip still captioned "wanted, for trustees 4 and 5", and the sealed envelope, whose outline glows at "Feed the dealer's method". At "every secret from zero to ten" [S-F7.secrets] a gold counter, an outline marked "trying", appears round the strip's 0 and glides to 10 and back; the strip's own labels never change. At "hundred and twenty-one ways" the 11 × 11 spin grid comes in column by column at the upper left, in the vault's place, faint yellow, first spin across and second up, numbered 0 to 10 on both axes, "11 × 11 = 121 spin pairs" over it [S-F6.spin_pairs]. At "its two spins could land" outlines glow round the two axis titles. | grid, lagged, counter | thought after "Here's an experiment." |
+| 05_2 | For each secret, count the ways of spinning that hand trustees four and five exactly what they hold, a two and a six. ⟨predict⟩ Will some secrets fit more ways than others? | At "For each secret" the counter, at 0, swells. At "count the ways of spinning" an empty tally slot appears under each tick, titled "ways that deal 2 and 6", and a yellow bar sweeps the grid's columns once, lighting nothing. At "a two and a six" the outlines round the pair's cards glow and slot 0 shows "?". No cell or count is shown before the question; the think ring with its "?" at the lower right. | predict | question hold (think time) |
+| 05_3 | No. Every secret fits exactly one way, and a different way each time. So the pair's two and six fit every secret equally well. | The think ring and the waiting "?" go before any count. From "Every secret fits exactly one way" the counter steps 0 → 10; for each secret one cell lights gold with that secret's number in it, a different cell each time, one in every row and every column [S-F7.spins_by_secret], and its tally slot gets a 1 [S-F7.ways_each]: 11 cells in all [S-F7.fitting]. At "So the pair's two and six" the counter goes, the outlines round the pair's cards glow, and the strip, now computed from their two shares, shows all eleven, captioned "for: trustees 4 and 5". | grid highlight, live counter | thought after "No."; hold on the grid after "a different way each time." |
+| 05_4 | They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met. | precisely: "with fair spins, P(shares 2 and 6 \| s) = 1/121 for every secret s, so their belief about s is unchanged" [S-F7.chance_each]. At "every secret is exactly as likely" an outline glows round the tally row of 1s. | precisely card | thought after "what to guess." |
+| 05_5 | Two shares can't determine the secret, and three can. The rest of the story is why both are true. | The card goes. At "Two shares can't determine the secret" the line "2 shares → all 11 secrets still fit"; at "and three can" the line "3 shares → the secret" above it. The envelope stays sealed; its outline glows at "The rest of the story". | card | chapter_transition |
 
 ## 06_padlocks: a way that works, and its price
 
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -19,3 +19,13 @@
 03_5	Nothing at all. If A had zero, the secret would be seven. If A had one, eight. Go through all eleven marks A could hold, and you meet all eleven secrets, once each.
 03_6	A's mark came from a fair spin, so B's seven fits every secret equally well. A circle has no floor and no ceiling, so a share can't push the secret above or below anything.
 03_7	That fixes the leak, but this vault still needs both trustees.
+04_1	Back to five trustees and a threshold of three. Here is where we're headed. Using the secret and two fair spins, the dealer hands out five shares, eight, three, one, two and six.
+04_2	Trustees one, two and three bring their eight, three and one, work out the secret, five, and the door opens. How they do it comes later.
+04_3	Trustees four and five hold two and six. Like anyone, they can guess. They'd be right one time in eleven, and the other ten times the alarm goes off.
+04_4	So the demand isn't that two trustees can never get lucky. It's that their two shares tell them nothing, not one mark ruled out, not even a hint that the secret is odd.
+04_5	Is that even possible? Could two shares be completely useless, when three open the vault?
+05_1	Here's an experiment. Feed the dealer's method every secret from zero to ten, with every one of the hundred and twenty-one ways its two spins could land.
+05_2	For each secret, count the ways of spinning that hand trustees four and five exactly what they hold, a two and a six. Will some secrets fit more ways than others?
+05_3	No. Every secret fits exactly one way, and a different way each time. So the pair's two and six fit every secret equally well.
+05_4	They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met.
+05_5	Two shares can't determine the secret, and three can. The rest of the story is why both are true.
```
<!-- shamir-0405 patch end -->

The patch's sha256 is
ca8d7daf4c50185b7142edbeb9b446ed1ac0ddc26efb05afb0fac1ec1452807a (285
lines). It was made against the live tree as of 09:10Z: `mod.rs`
9435dcc5…, `main.rs` a7f52a59…, `facts.rs` 260afb41…, `oracle.py`
2e9dffc6…, `storyboard.md` 153baf48…, `storyboard-draft.md` f3cf3d6b… and
`script.tsv` e52090ac…. If root has changed any of these since, the hunks
may need rebasing; the content is the list above.

## Chapters 06 and 07, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment after Mail 6885 (root's `tend-0913/shamir-next06-07.txt`
and Mail 6887). The ask:
- write new native chapters `06_padlocks` and `07_a_line_through_the_secret`
  from the full draft, keeping the vault and trustee identities;
- in 06, build the ten-lock construction in legible stages, show why the
  missing pair cannot open its own lock, and earn minimality with a concrete
  merged-lock failure before the 462/252 comparison, with an adequate
  reading hold for the quote and cards;
- in 07, announce the simpler two-of-five example, carry the trustee
  positions into the line plot, show the slope and the share heights from
  the model, withhold the one-share candidate count through the prediction,
  and make the three compatible secrets against eleven visible before
  chapter 08 wraps;
- no text recolor, scale or flash that overlaps digits: independent outline
  and shape emphasis only;
- take the time the explanation needs (the 85-second estimate is advisory),
  with no arbitrary word cuts or forced splitting;
- write only newly reserved files and this file, reuse the delivered helpers
  without editing them, and propose root's changes as a patch against fresh
  shared hashes.

**Status.**
- I wrote three new files. I edited no registered or delivered file:
  `the_vault.rs`, `two_keys.rs`, `cutting_it_up.rs`, `the_dial.rs`,
  `envelope.rs`, `the_goal.rs`, `an_experiment.rs`, `draw.rs`, `palette.rs`,
  `glyphs.rs`, `mod.rs`, `model.rs`, `facts.rs`, `main.rs`, `oracle.py`, the
  scripts, the storyboards and Cargo. Root's frozen 04/05 intake was not
  touched.
- Nothing was compiled, rendered, synthesized or listened to.
- Bead mx-video-program-program-87do.46.3 stays in progress; I close nothing.

**Reservations.**
- Held: 8034–8036 (the three new files), 8037
  (`docs/previous/ch0607-*/**`) and 7941 (this file).
- Released at 09:16:53Z: the 04/05 leases 7939, 7940, 7942 and 7970.

**Prior bytes.** `docs/previous/ch0607-before-06-07-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section, sha256
46a2f27131fefcf89819b0197538383747e4845381e11df87fa214d43549bdbd.

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `plot.rs` | 353 | 5230d531c30d6e3c655f8aee5d15e78dd28bf1b6cd00414a0a7a3288893a5563 |
| `padlocks.rs` | 1308 | 3187d465c2e4ed64ea07374f61c0341dc13197973f044893680d819b1a127d62 |
| `a_line_through_the_secret.rs` | 778 | 1d8174b26c13433f6854372e48d7840f6e22a47697d6a325b1cbc3b531fc258d |

- **`plot.rs`**: the paper plot.
  - Positions 0 to 5 along the bottom and heights 0 to 25 up the side,
    labelled every 5.
  - `Marker` (position discs), `secret_name`, `point`, `point_label` and
    `segment`.
  - `band`, an outline round a span of heights up the side.
  - `through`, a line's two ends clipped to the plot, and `at_zero`, a
    line's integer height at position 0.
  - 4 tests.
- **`padlocks.rs`**: `ThePadlocks`, key `06_padlocks`. It holds the padlock
  and key shapes, the merged-lock rows, the quote cards (wrapped with the
  kit's `animatic::wrap`) and the table. 7 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `the_keys_fill_every_slot_once`;
  - `a_merged_lock_shuts_out_a_group_that_must_open`, with a planted
    negative: a lock for one pair shuts out no trio;
  - `the_quote_cards_are_whole`;
  - `the_envelope_parts_are_in_the_order_the_scene_opens`;
  - `the_layout_keeps_its_pieces_apart`.
- **`a_line_through_the_secret.rs`**: `ALineThroughTheSecret`, key
  `07_a_line_through_the_secret`. 6 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `the_fan_meets_three_secrets`;
  - `any_two_points_give_back_the_secret`;
  - `the_card_is_two_lines`;
  - `the_layout_keeps_its_pieces_apart`.

**Dependencies.**
- The three files import only `pub` items of the delivered helpers:
  - `draw.rs`: `Rail`, `Strip`, `read_by`, `READ_CPS`, `think`, `say` and
    others;
  - `envelope.rs`: `Envelope`, `dealt`, `outline`, `frame_halos` and
    `swelling`;
  - `two_keys.rs`: `Face`, `glow`, `glowing`, `text_halo`, `leaving`,
    `moving`, `check_strip` and the ordinary view.
- A scan found all 94 imported names to be top-level `pub` items.
- They need the patch's `model.rs` to compile: `padlocks.rs` calls
  `model::fewest_locks`, and 07 builds `View::Paper`.
- Their phrase tests read `crate::SCRIPT`, so they also need the patch's
  `script.tsv` lines.
- My files need no edit at integration.

### How the brief is met

**06, checked before anything is drawn.** `construct` builds
`Padlocks::new(&TRUSTEES, THRESHOLD)`. It stops with an error unless:
- every trustee carries the same number of keys;
- trio 1, 2, 3 opens every lock;
- pair 4, 5 is stopped by exactly one lock;
- `fewest_locks` equals both the plan's lock count and the number of
  pairs, 10;
- each merged lock shuts out at least three trustees.

**The ten locks in stages.**
- **06_1.** The vault's door and lamp, without the dial; the rail; the
  rule card "any 3 of 5".
  - At "Put ten padlocks on the door" the ten locks come in one after
    another, two rows of five, each labelled by its pair (1·2 … 4·5).
  - At "one for each pair of trustees" outlines glow round lock 1·2 and
    round trustees 1 and 2's frames.
- **06_2: one lock first.** Lock 4·5 gets a purple outline and trustees 4
  and 5 turn purple. Three keys fly from it into the frames of 1, 2 and 3.
- **06_3.**
  - The other 27 keys fly to the trustees outside each lock's pair, six
    slots to a frame. The card reads "6 keys each, 30 keys in all".
  - Trio 1, 2, 3 turns teal and all ten locks spring open, one after
    another.
  - At "because no pair contains all three of them", two examples: lock 1·2
    with trustee 3's frame, then lock 4·5 with trustee 1's.

**Why the pair cannot open its own lock.**
- **06_4.** The locks shut, 4 and 5 turn purple, and the think ring comes.
- **06_5.**
  - As the "?" goes, in one move, the nine locks that 4 and 5 hold keys to
    spring open, and lock 4·5 stays shut with a purple outline.
  - At "their own", outlines glow round its key holders, 1, 2 and 3. Those
    keys have sat in their frames since 06_2.

**Minimality, earned by a concrete failure before 462/252.**
- At "And no lock can do that job" two merged locks come in together, right
  of the door. The pair labels move off the grid onto them: 1·2 and 2·3 on
  the first, 1·2 and 3·4 on the second.
- At "or it would also stop the three": "keys only with 4 and 5" over "1, 2,
  3 together: shut out" in red. Trustees 1, 2 and 3 turn teal, red outlines
  glow round their frames, and the lock's body swells red.
- At "or four trustees": "keys only with 5" over "1, 2, 3, 4 together: shut
  out".
- The holders and the shut-out group come from `merged`: a merged lock's
  keys go only to the trustees outside both pairs.
- At "So ten locks is the fewest" the card "every pair needs a lock of its
  own," over "so 10 pairs need at least 10 locks", and an outline round the
  grid.
- The argument on screen is the counting one: no lock serves two pairs, so
  each pair needs its own. The exhaustive `fewest_locks` search (10, over
  16 usable holder sets) backs `S-F3.fewest_locks` but is not drawn.

**Reading holds.**
- **06_5.** The two merged rows and the fewest card are read one after
  another. `read_on` starts each text's reading when the previous one's
  ends, at the house 15 characters a second. The scene waits until all
  three are read.
- **06_6 and 06_7.**
  - The quote card comes in as 06_6 begins.
  - The table and Shamir's next sentence, a full verbatim card, come in as
    06_7 begins.
  - After 06_7 the scene holds until both cards are read, on their own
    parallel `read_by` clocks (see the timing risks).
- **06_8.** The property card comes in at "from the same dial as the
  secret", and the scene holds until it is read.

**The table.**
- The "any 3 of 5 trustees" row shows 10 and 6, from the plan.
- The "any 6 of 11 scientists" row counts up 0 → 462 and 0 → 252 at their
  phrases, from `Padlocks::new(1..=11, 6)`.
- Then "= C(11, 5)" and "= C(10, 5)" appear under the counts.

**06_8: no locks at all.**
- The cards and the table go, and the envelope from 04 returns, 0.8 lower
  (`ENV_DROP`).
- At "with no locks at all", the keys and the counts go, the locks fade one
  after another, and the vault's dial returns to the door as a `Face`.
- At "Each trustee carries one number":
  - the envelope opens: the flap, the seal and the "sealed" note go, and the
    flap stands open;
  - the rail's cards 8, 3, 1, 2, 6 come in one after another, from
    `envelope::dealt`.
- At "from the same dial as the secret", the property card, outlines round
  all five frames, and a gold ring round the dial.

**07, checked before anything is drawn.**
- The line is `Recipe::new(SECRET, &[LINE_SLOPE])`. The scene errors unless
  every height stays on the plot and the height at 0 is the secret.
- Trustee 5's paper view is `View::Paper { points: [(5, 25)], spins: 1 }`.
  The scene errors unless the fan's lines that meet position 0 on the dial
  are exactly that view's secrets, with one line per mark.

**The simpler example, announced.** At "start with a simpler rule" the card
"a simpler rule: any 2 of 5" comes in at the upper right.

**Trustee positions carried into the plot.**
- At "Mark the secret", copies of the rail's five discs lift onto positions
  1 to 5, and a gold disc 0, "secret", sits under position 0.
- In 07_3:
  - at "Trustee one sits at position one" and "trustee two at position
    two", rings glow round those position discs, with outlines round the
    same trustees' frames;
  - at "and so on", the same for 3 to 5.

**The slope and the heights, from the model.**
- The yellow spare dial, titled "dealer's fair spin", turns twice and stops
  on `LINE_SLOPE`, 4. A "+4" appears by it.
- The line grows from (0, 5) to (5, 25). Each step's run (grey) and rise
  (yellow) is drawn in turn, with a "+4" by every rise.
- Guides rise to blue points at `line.height(t)`. Each height's label comes
  in as it is said, and the same number drops into that trustee's card.

**Any two redraw the line (07_4).**
- Trustees 2 and 3 stay. The redraw is checked in integers first: a rise of
  4 over a run of 1, meeting 0 at 5.
- It is drawn from (3, 17) back through (2, 13) to position 0.
- Then the gold point at (0, 5) swells and the side's "5" gets an outline.

**The count, withheld through the prediction (07_5).** On screen:
- only (5, 25);
- a grey line pivoting through it, at "not enough to redraw it";
- the strip with all eleven lit, "for: anyone outside";
- the 25 in trustee 5's card.

There is no fan and no count before the think ring.

**3 against 11, visible before 08 (07_6).**
- The "?" goes. A gold band outline marks heights 0 to 10 up the side.
- The fan comes in: one faint line per spin 0 to 10, from (5, 25) toward
  position 0, clipped to the plot.
- At "zero, five or ten":
  - the three lines meeting position 0 inside the band, spins 5, 4 and 3,
    turn gold, with gold points at 0, 5 and 10;
  - the strip drops from 11 to 3, "for: trustee 5, on paper".
- At "No other secret" the other eight lines fade to ghost and the
  precisely card comes in.

**07_7 hands over.**
- Red outlines go along the side from 10 to 25, and round the 25 and trustee
  5's frame.
- The strip shows chapter 02's B holding 3: 8 lit.
- A small dial comes in between the plot and the strip, and the band glows.
  Chapter 08 does the wrapping.

**Numbers kept apart from emphasis, in both chapters.**
- No `indicate`, recolor, style change or scale ever touches typeset text.
- Recolors go to lock bodies, rail discs (`Rail::tint`) and fan lines.
  `swelling` goes only to a lock body and to plot points.
- Text is pointed at by outlines of its own: card-frame halos, `text_halo`s,
  the side's band, and rings round position discs.
- In my code, no different digits ever crossfade at one place:
  - labels and card numbers come in where nothing was;
  - they leave before anything else comes there.
- What still changes in place:
  - the 06_7 count-ups (`live_number`, one number re-set each frame);
  - root's `Strip` machinery in 07: the live count tweens 11 → 3 → 8, and
    the captions crossfade from "for: anyone outside" to "for: trustee 5,
    on paper" to B's caption.

  Not checked by eye.

### Spoken and visual deltas from the draft

**Spoken: none.**
- The draft's words for 06 and 07 are unchanged, and so is
  `docs/script-draft.tsv`.
- No word was cut and no line split.

**Visual deltas.** The patch rewrites only the picture, features and
hold/listen cells of the draft's 06/07 rows, to describe the scenes as
written. Production gets identical rows.
- **06_1.** Adds the rule card and the lock 1·2 outline at "one for each
  pair".
- **06_2.** An outline instead of a purple highlight on the lock. The keys
  fly into card frames.
- **06_3.**
  - Keys fly into six slots a frame, instead of a key-ring count-up.
  - Adds the card "6 keys each, 30 keys in all".
  - Adds two worked examples at "because no pair contains all three of
    them".
- **06_5.** The draft's flash and single crowded lock become:
  - the pair's own lock, outlined;
  - two merged locks with their failure lines;
  - red outlines and a body swell instead of a flash;
  - the fewest card.
- **06_6.** Adds an outline round the quote at "Any six of eleven
  scientists".
- **06_7.**
  - Adds the table (10 and 6 against 462 and 252).
  - Shamir's sentence becomes a full verbatim card.
  - "= C(11, 5)" and "= C(10, 5)" become two labels under the counts.
- **06_8.**
  - The envelope is lower than in 04, and the dial comes back as a `Face`.
  - Adds outlines round the five frames and a gold ring round the dial.
- **07_1.** Adds the rule card, the position-0 ring and the outline round
  the side's "5".
- **07_2.** Adds the spare dial's title and the "+4" ring. Runs and rises
  are drawn step by step.
- **07_3.** Adds rings, frame outlines and guides.
- **07_4.** The draft's "flashes gold" becomes a swell plus an outline round
  the side's "5".
- **07_5.** The fan moves to 07_6, so that the count is withheld through the
  prediction. 07_5 gets instead the pivoting line, the strip at 11 and the
  25 into trustee 5's card.
- **07_6.**
  - The misses fade to ghost, not red: red is the leak role, which 07_7
    uses.
  - Adds the precisely card and the gold band.
- **07_7.** The draft's axis fold becomes a gold band outline and a small
  dial; the fold itself is chapter 08's. Adds red outlines on the leak.

**A corrected draft statement.**
- 07_5's draft says "(most far below 0 or above 25)". That is wrong: no spin
  meets position 0 above 25.
- Spins 0 to 2 meet it above 10, at 25, 20 and 15.
- Spins 6 to 10 reach the bottom axis first, at x = 5/6, 10/7, 15/8, 20/9
  and 5/2.
- The proposed 07_6 row says this.

**Semantic pauses and visual holds** (hold/listen cells; the pause map is
root's). New or changed entries are in bold.
- 06_1: thought after "with padlocks."
- 06_2: thought after "outside its pair."
- 06_3: thought after "six keys."
- 06_4 and 07_5: question holds. The think ring is the kit's `THINK_S`,
  4 s.
- 06_5: thought after "their own."; thought before "So ten"; **visual hold
  after "works." until both merged rows and the card are read one after
  another**.
- 06_6: thought after "an older book."
- 06_7: **hold on the two numbers until both cards are read**.
- 06_8: chapter_transition; **visual hold until the property card is
  read**.
- 07_1: thought after "any two of five."
- 07_2: thought after "gives four."
- 07_3: connected list (no pause between heights).
- 07_4: thought after "redraw the line,"
- 07_6: thought after "zero, five or ten."; **visual hold until the card is
  read**.
- 07_7: chapter_transition.

### Timing and layout risks (estimates; nothing is measured)

Speech is estimated at words / 2.6 + 0.4 s, and reading at 15 characters a
second (`READ_CPS`).
- **06_5** (≈ 14.6 s spoken). The rows come in at their cause phrases. The
  one-after-another reading clock leaves ≈ 3 to 4 s of hold after "works."
- **06_6 and 06_7** (≈ 21.6 s spoken together).
  - The quote with its source is 297 characters: ≈ 19.8 s of reading from
    its arrival as 06_6 starts.
  - The answer with its source is 106 characters: ≈ 7.1 s from 06_7's
    start.
  - On the scene's parallel clocks, the hold after 06_7 comes to ≈ 0 to
    1 s.
  - **Root's decision:** if a viewer should read the quote and then the
    answer, one after the other as in 06_5, the hold becomes ≈ 6 to 7 s
    longer. That is a one-line change at `padlocks.rs:804`:
    `read = read_on(stage, read, &format!("{ANSWER} {ANSWER_SOURCE}"));`.
  - I left it parallel because the quote is mostly read before 06_7
    begins, and the table states the answer's claim. I have not watched it.
- **06_8.** The property card (103 characters) comes in at the line's last
  phrase, so the hold is ≈ 5 to 6 s.
- **07_6** (≈ 6.9 s spoken). The precisely card (125 characters, 2 lines)
  needs ≈ 8.3 s from "No other secret", so the hold is ≈ 4 to 5 s.
- **Cue-bound moves can land late** if the narrator is faster than the
  estimate:
  - the 07_2 spin (two turns) before "say it gives four";
  - the 06_3 flight of 27 lagged keys;
  - the 06_5 opening move (1.2 s) before "their own".
- **Chapter lengths.** The storyboard estimates 06 at 89.6 s, flagged
  ADVISORY ("longer chapter"), and 07 at 74.6 s. Nothing was cut for
  length.
- **Layout.** Text sizes are estimated at 0.0055 units a point per
  character wide and 0.0065 a point tall. The quote card wraps to 5 + 2
  lines; the answer and property cards to 2 + 1. Real widths may differ.
- **Glyphs are unprobed.** "·" (the lock labels), "−" and "×" (07_6's
  card), and "(", ")" and "=" (the C(n, k) labels).

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

The scratch scripts named below live in this session's scratchpad, not in
the repository; my handoff mail quotes their results. These are the runs of
10:26–10:40Z.

- **Formatting** (`rustfmt --check --edition 2024 --config-path
  rustfmt.toml`, a parse check only).
  - rc 0 on each of the three files.
  - rc 0 on the patched scratch tree's `main.rs`. That run parses every
    module, including the patched `model.rs` and `facts.rs`.
- **Cues** (`check_cues.py TOPIC 06:padlocks 07:a_line_through_the_secret`).
  - Each scene says its lines in order: 06_1–06_8 and 07_1–07_7.
  - All 17 cue phrases of 06 and all 22 of 07 occur in their lines.
  - 7 test-listed phrases reach `wait_for` through arrays (`CAUSES`, the
    06_7 table rows and the 07_3 groups). The checker reports them as "not
    waited on" and still finds each in its line.
  - 0 failures, rc 0.
- **Independent arithmetic** (`evidence.py`). Brute force written from the
  storyboard's statements, not from `oracle.py` or the Rust model. 43
  checks, 0 failures, rc 0:
  - **the locks.**
    - 10 locks, one per pair; 6 keys each, 30 in all.
    - Lock 4·5's keys go to 1, 2 and 3.
    - Every trio opens all ten, and every pair is stopped by exactly its
      own lock.
  - **merged locks.**
    - 45 pairs of different pairs, with unions of 3 or 4 trustees (30
      overlapping, 15 separate).
    - Every merged lock shuts out a whole trio.
    - 1·2 + 2·3 has keys only with 4 and 5 and shuts out 1, 2, 3; 1·2 + 3·4
      has keys only with 5 and shuts out 1, 2, 3, 4.
    - **Negative:** a lock for one pair shuts out no trio.
  - **the fewest locks.**
    - An exhaustive search gives 10, over 16 usable holder sets.
    - **Control:** any 2 of 5 needs 5, so the search can answer other than
      10.
  - **Liu's numbers.**
    - 462 = C(11, 5) and 252 = C(10, 5), by enumeration.
    - Every usable lock of the 2048 stops at most one five-group, so 462
      is the fewest.
    - 462 × 6 / 11 = 252.
  - **the line.**
    - Heights 9, 13, 17, 21, 25, every step climbing 4.
    - All 10 pairs rebuild slope 4 and meet position 0 at 5.
    - **Negative:** a share off by one rebuilds secret 3.
  - **one share.**
    - 25 at position 5 leaves 0, 5 and 10: 3 of 11.
    - Spins 0–2 meet position 0 above 10, spins 6–10 below 0, and none
      above 25.
    - The fan's ends are at the bottom-axis x values above.
    - **Negative:** taking secrets as 0–25 would leave 6.
  - **B holding 3** leaves 8.
  - **the quotes.**
    - QUOTE (190 characters), ANSWER (91) and PIECE (69) are verbatim in
      `docs/brief.md`.
    - The quote stops before Liu's questions.
    - **Negative:** a changed quote is not found.
  - **the cards.** Each card wraps whole. The 07_6 card, recomputed, equals
    the test's string (125 characters) and is 2 lines at every wrap from
    64 to 92.
- **A Python mirror of the proposed `fewest_locks` and `View::Paper`**
  (`mirror_fewest.py`), rc 0.
  - needed 1 → 1 lock over 1 usable set; 2 → (5, 6); 3 → (10, 16).
  - needed 0 → None, and 4, 5 and 6 → None. For 4 and 5 the None is the
    documented `PLANS_TRIED` limit (26 and 31 usable sets, over 20), not "no
    plan exists".
  - Paper views:
    - (5, 25) with 1 spin → 0, 5, 10;
    - (4, 145) and (5, 215) with 2 spins → 5;
    - (2, 13) and (3, 17) → 5;
    - (2, 13) and (3, 18) → 3;
    - (2, 13) and (4, 22) → none.

    These are the proposed model tests' expectations. This mirror caught a
    wrong expectation in my first draft of that test, now fixed.
- **Layout mirror** (`layout_mirror.py`). The constants are read out of the
  Rust sources, and the layout and arithmetic assertions of `plot.rs`,
  `padlocks.rs` and 07 are transcribed one for one.
  - 85 assertions, 0 failures, rc 0.
  - A planted control (a 0.9 wrap dial touching the strip) fails as
    planted.
- **The oracle on the patched scratch tree.**
  - `oracle.py --self-test` prints 210 facts. stderr: `# self-test:
    ten-mark dial pairs that leak: 9 of 10` and `# 210 facts, 0 failed
    cross-checks`. rc 0.
  - The output's sha256 is
    af83e66c9c3c5b6e40c8303dee7b5521f8284ab61ea75b55024e6d3684fa8b49.
- **The golden table** (`check_golden.py`, mirroring `oracle_diff` and
  `f.len()`). The patched `facts.rs` has 210 golden rows (210 distinct),
  against the patched oracle's 210, with `f.len()` asserted 210. Diff:
  none. rc 0.
- **Spoken numbers** (`check_spoken.py`, mirroring
  `every_spoken_number_is_the_fact_it_speaks_for`). Patched `main.rs`: 110
  `SPOKEN` rows, 6 `NOT_A_COUNT` and 40 number words.
  - All 45 lines of the patched `script.tsv`: 178 number words, 0
    failures, rc 0.
  - 06/07 only: 15 lines, 67 number words, 0 failures, rc 0.
  - In both runs the planted control was caught.
- **Storyboards** (a copy of `tools/storyboard.py` in a scratch tree, which
  writes only there).
  - `--check shamir_secret_sharing` gives rc 0, with the estimates above
    and a total of 8.8 min over 7 chapters.
  - `check_sb.py` gives 0 failures, rc 0:
    - production chapters 01–07 equal the draft's, row for row;
    - in the draft only the 06/07 rows changed, and only outside their
      words and titles;
    - the draft's generated lines equal `docs/script-draft.tsv`;
    - `script.tsv` is the 30 lines of the 0405-patched tree plus exactly
      the draft's 15 lines for 06 and 07: 45.
- **The patch** (all on scratch copies of the topic; nothing was applied
  to the repository).
  - On a fresh copy of the live topic (10:36Z), 0405 applied as is (rc 0,
    with its own known "new blank line at EOF" warning), then 0607:
    `--check` rc 0 and apply rc 0.
    - A recursive diff against the scratch tree the checks above ran on
      differs only in `docs/WORK_STATUS.md` and `docs/previous/`, which the
      patch does not touch.
    - All 8 patched files and the three new files are byte-identical.
  - With 0405 applied `--whitespace=fix` (which drops `storyboard.md`'s
    trailing blank line), the storyboard hunk fails, rc 1.
    - `-C1` then applies with rc 0, but **misplaces the 06/07 sections**
      at line 94, inside chapter 05. Do not use it.
    - Appending one empty line to `docs/storyboard.md` and then applying
      normally gives rc 0, with all 8 files byte-identical.
  - On the live tree without 0405, 0607's `--check` fails (rc 1), as
    expected: it is made on top of 0405.
- **Static audit.**
  - In the three files, recolors target only lock bodies, rail discs and
    fan lines, and `swelling` only a lock body and plot points.
  - No `indicate` or flash is called directly. `indicate` is reached only
    through `envelope::swelling`.
  - The numbers on cards and labels are formatted from the model, the plan
    or named constants. `the_pictures_are_the_facts` (unrun) compares them
    with the facts.

### Not verified

- **Nothing compiled.** No Cargo, rustc or clippy-driver. All 19 new tests
  are unrun: padlocks 7, plot 4, 07 6 and model 2. So are the changed facts
  and `main.rs` tests.
  - Borrow and type errors are possible. I split two nested `stage`
    borrows by hand and found no others by search.
  - Clippy's view of the casts and the bit-mask search is unknown.
- **Nothing rendered or watched.** The engine behaviours I rely on, read
  from the pinned source but not run:
  - `AnimationGroup::with_lag_ratio`;
  - `create_lagged` and `show_creation`;
  - `tween` on a value tracker with `live_number`;
  - the `Face` needle spun with `rush_from`;
  - `recolor` of stroked lines keeping the fill invisible;
  - labels moved with `moving`;
  - `precisely`'s wrap.
- **Layout is estimated**, with the text sizes above.
- **Glyphs are unprobed.** The kit's `glyphs` probe renders, so it is root's
  to run.
- **No listening.** Nothing here claims accent consistency. Every duration
  above is an estimate.
- **The kit is changing.** The working tree has an uncommitted
  `crates/explainer_kit/src/narration.rs` change that is not mine; I
  checked the kit items I import against the current working tree.

### For root's integration (a proposal; I edited none of these files)

Apply it **after shamir-0405** (above, sha256 ca8d7daf…), as one unit. The
reason: `main.rs`'s `check_video` compares the registry with the storyboard
and the script. The patch includes `script.tsv` as regenerated by
`tools/storyboard.py shamir_secret_sharing`.

- **`src/chapters/mod.rs`.**
  - `pub mod a_line_through_the_secret;`, `pub mod padlocks;` and
    `pub mod plot;`.
  - `06_padlocks` → `padlocks::ThePadlocks` and
    `07_a_line_through_the_secret` →
    `a_line_through_the_secret::ALineThroughTheSecret`, each at intensity
    0.5.
  - The doc reads "chapters 01 to 07 so far. The other sixteen …".
- **`src/model.rs`** (additions only).
  - `View::Paper { points: Vec<(u32, i64)>, spins: usize }`. Its
    `possible` is the secrets `s` for which some `tuples(dial, spins)` `t`
    has `Recipe::new(s, &t).height(x) == h` at every point.
  - `pub fn fewest_locks(trustees: &[u32], needed: usize) -> Option<usize>`:
    - every family of usable holder sets is searched by bit mask;
    - `None` if needed is 0 or over n, if n ≥ 32, or if there are more than
      `pub const PLANS_TRIED: usize = 20` usable sets.
  - Two tests:
    - `the_paper_view_agrees_with_the_paper_counts`, which includes the
      other-secret ([3]) and empty cases;
    - `the_padlock_answer_is_the_fewest`: for needed 1 to 3 it equals the
      `Padlocks` plan's lock count, 3 gives 10, and 0 and 6 give `None`.
- **`src/facts.rs` and `oracle.py`.** 14 new facts. The golden table grows
  from 196 to 210 rows, `f.len()` from 196 to 210, and the module doc says
  "Chapters 01 to 07":

  | Key | Value | Rust (model) | Python (oracle) |
  |---|---|---|---|
  | `S-F3.shamir.too_few` | 5 | `SHAMIR_NEEDED - 1` | the largest k below 6 |
  | `S-F3.locks_per_pair` | 1 | each pair's count among the plan's locks | the same, over its own lock list |
  | `S-F3.pair_lock.holders` | 1,2,3 | trustees outside lock 4·5 | trustees whose key lists include 4·5 |
  | `S-F3.merged.pairs` | 2 | pairs picked | `len` of a pair of pairs |
  | `S-F3.merged.unions` | 3,4 | union sizes over all 45 picks | the same, with sets |
  | `S-F3.fewest_locks` | 10 | `model::fewest_locks` (bit masks) | its own `fewest_locks`, plans tried smallest first |
  | `S-F3.dealer.numbers_each` | 1 | numbers per trustee in `dealt` | shares ÷ trustees |
  | `S-F4.paper.count` | 3 | `paper_line_secrets(…).len()` | a count by enumeration |
  | `S-F4.needed` | 2 | `line.coefficients().len()` | the fewest trustees whose shares always leave one secret, by search |
  | `S-F4.too_few` | 1 | the above − 1 | the above − 1 |
  | `S-F4.spin_count` | 1 | `line.spins.len()` | `len((LINE_SLOPE,))` |
  | `S-F4.secret_position` | 0 | where the line's height is the secret | the same, by search |
  | `S-F4.position_of.1` | 1 | the trustee's own number | where the line's height equals trustee 1's share, by search |
  | `S-F4.position_of.2` | 2 | as above | as above, for trustee 2 |

  - **Honest note.** Several facts are bookkeeping for the spoken words,
    not independent arithmetic: `S-F3.shamir.too_few`, `S-F3.merged.pairs`,
    `S-F4.too_few` and `S-F4.spin_count` are computed the same simple way on
    both sides, and the Rust `S-F4.position_of.*` simply restates a
    trustee's own number (only the oracle searches for it).
  - New oracle cross-checks: every lock serving two pairs shuts out a trio,
    and any two of five needs a lock per trustee.
- **`src/main.rs`.** 41 `SPOKEN` rows for 06_1–07_6, among them "Put ten
  padlocks" (`S-F3.locks`, 10) and "trustees four and five" for 06_4. No
  new `NOT_A_COUNT` rows.
- **`docs/storyboard-draft.md`.** The 06/07 rows as quoted in the deltas
  above. Their words are unchanged, and nothing outside 06/07 changes.
- **`docs/storyboard.md`.** "chapters 01 to 07" in its intro, and the
  draft's 06 and 07 sections appended verbatim, after one blank line.
- **`script.tsv`.** Regenerated: 45 lines.
- **No change to a delivered helper is needed.**

To extract the patch and apply it (after 0405):

~~~sh
awk '/^<!-- shamir-0607 patch end -->$/{p=0} p; /^<!-- shamir-0607 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-0607.patch
git apply --check shamir-0607.patch && git apply shamir-0607.patch
~~~

If root applied 0405 with `--whitespace=fix`, `docs/storyboard.md` has lost
its trailing blank line, and the storyboard hunk fails. Append one empty
line to that file first, then apply as above. **Do not use `-C1`**: tested,
it puts the 06/07 sections inside chapter 05.

<!-- shamir-0607 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,15 +1,18 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 05 so far. The other
-//! eighteen are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 07 so far. The other
+//! sixteen are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
+pub mod a_line_through_the_secret;
 pub mod an_experiment;
 pub mod cutting_it_up;
 pub mod draw;
 pub mod envelope;
 pub mod glyphs;
+pub mod padlocks;
+pub mod plot;
 pub mod the_dial;
 pub mod the_goal;
 pub mod the_vault;
@@ -44,5 +47,15 @@
             make: |kit| Box::new(an_experiment::AnExperiment { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "06_padlocks",
+            make: |kit| Box::new(padlocks::ThePadlocks { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "07_a_line_through_the_secret",
+            make: |kit| Box::new(a_line_through_the_secret::ALineThroughTheSecret { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -142,6 +142,54 @@
         ("05_3", "two and six fit", "S-F1.pair_shares", "2,6"),
         ("05_5", "Two shares can't", "S-F1.too_few", "2"),
         ("05_5", "and three can", "S-F1.threshold", "3"),
+        ("06_1", "any three", "S-F1.threshold", "3"),
+        ("06_1", "of five", "S-F1.trustees", "5"),
+        ("06_1", "Put ten padlocks", "S-F3.locks", "10"),
+        ("06_1", "one for each pair", "S-F3.locks_per_pair", "1"),
+        ("06_2", "four and five", "S-F1.pair", "4,5"),
+        ("06_2", "with one, two and three", "S-F3.pair_lock.holders", "1,2,3"),
+        ("06_3", "six keys", "S-F3.keys_each", "6"),
+        ("06_3", "Any three trustees", "S-F1.threshold", "3"),
+        ("06_3", "all three of them", "S-F1.threshold", "3"),
+        ("06_4", "trustees four and five", "S-F1.pair", "4,5"),
+        // how many locks stop the pair: just their own
+        ("06_5", "Just one", "S-F3.locks_a_pair_cannot_open", "1"),
+        ("06_5", "two different pairs", "S-F3.merged.pairs", "2"),
+        ("06_5", "the three or four trustees", "S-F3.merged.unions", "3,4"),
+        ("06_5", "So ten locks", "S-F3.fewest_locks", "10"),
+        ("06_6", "nineteen seventy-nine", "S-X1.shamir.year", "1979"),
+        ("06_6", "Any six", "S-F3.shamir.needed", "6"),
+        ("06_6", "of eleven scientists", "S-F3.shamir.scientists", "11"),
+        ("06_6", "no five can", "S-F3.shamir.too_few", "5"),
+        ("06_7", "four hundred and sixty-two locks", "S-F3.shamir.locks", "462"),
+        ("06_7", "two hundred and fifty-two keys", "S-F3.shamir.keys_each", "252"),
+        ("06_8", "one number", "S-F3.dealer.numbers_each", "1"),
+        ("07_1", "any two", "S-F4.needed", "2"),
+        ("07_1", "of five", "S-F1.trustees", "5"),
+        ("07_1", "position zero", "S-F4.secret_position", "0"),
+        ("07_1", "height five", "S-F1.secret", "5"),
+        ("07_2", "spins once", "S-F4.spin_count", "1"),
+        ("07_2", "gives four", "S-F4.slope", "4"),
+        ("07_2", "climbs four", "S-F4.slope", "4"),
+        ("07_3", "Trustee one sits at position one", "S-F4.position_of.1", "1"),
+        ("07_3", "trustee two at position two", "S-F4.position_of.2", "2"),
+        (
+            "07_3",
+            "Nine, thirteen, seventeen, twenty-one and twenty-five",
+            "S-F4.heights",
+            "9,13,17,21,25",
+        ),
+        ("07_4", "Any two trustees", "S-F4.needed", "2"),
+        ("07_4", "their two points", "S-F4.needed", "2"),
+        ("07_4", "the height at zero", "S-F4.secret_position", "0"),
+        ("07_5", "One point is not enough", "S-F4.too_few", "1"),
+        ("07_5", "Trustee five", "S-F4.paper.holder", "5"),
+        ("07_5", "holds twenty-five", "S-F4.paper.share", "25"),
+        ("07_6", "Just three", "S-F4.paper.count", "3"),
+        ("07_6", "zero, five or ten", "S-F4.paper.candidates", "0,5,10"),
+        ("07_6", "is twenty-five", "S-F4.paper.share", "25"),
+        // the line through (5, 25) climbing a whole spin a step
+        ("07_6", "five times a spin", "S-F4.paper.holder", "5"),
     ];
 
     /// Number words said as an article, not a count, each with its line.
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -6,7 +6,7 @@
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
 //! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper and S-F7.join
 //! 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11 19 and 20,
-//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 05 are implemented so far; the
+//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 07 are implemented so far; the
 //! rest are computed and checked now so the draft's numbers are settled
 //! before its scenes exist.
 
@@ -156,6 +156,49 @@
     f.add("S-F3.shamir.needed", SHAMIR_NEEDED)?;
     f.pin("S-F3.shamir.locks", liu.locks().len(), "462", SHAMIR)?;
     f.pin("S-F3.shamir.keys_each", liu.keys(1), "252", SHAMIR)?;
+    f.add("S-F3.shamir.too_few", SHAMIR_NEEDED - 1)?;
+    // one lock for each pair; the lock for 4·5 has its keys with the three
+    // outside it
+    f.add(
+        "S-F3.locks_per_pair",
+        set_csv(pairs.iter().map(|g| locks.locks().iter().filter(|l| *l == g).count())),
+    )?;
+    let pair_lock = locks
+        .locks()
+        .iter()
+        .find(|g| g.as_slice() == PAIR)
+        .ok_or_else(|| "no lock for the pair".to_owned())?;
+    let pair_lock_holders: Vec<u32> = TRUSTEES.iter().copied().filter(|t| !pair_lock.contains(t)).collect();
+    f.add("S-F3.pair_lock.holders", csv(&pair_lock_holders))?;
+    // one lock serving two different pairs has its keys only outside both,
+    // so the three or four trustees in those pairs together hold none
+    let two = 2;
+    let ids: Vec<u32> = (0..pairs.len() as u32).collect();
+    let unions: BTreeSet<usize> = combinations(&ids, two)
+        .iter()
+        .map(|pick| {
+            TRUSTEES
+                .iter()
+                .filter(|t| pick.iter().any(|&i| pairs[i as usize].contains(t)))
+                .count()
+        })
+        .collect();
+    f.add("S-F3.merged.pairs", two)?;
+    f.add("S-F3.merged.unions", csv(&unions.into_iter().collect::<Vec<_>>()))?;
+    f.add(
+        "S-F3.fewest_locks",
+        model::fewest_locks(&TRUSTEES, THRESHOLD).ok_or_else(|| "no plan of locks works".to_owned())?,
+    )?;
+    // the dealer's method instead: one number each
+    let handed = dealt(dial, &Recipe::example(), &TRUSTEES);
+    f.add(
+        "S-F3.dealer.numbers_each",
+        set_csv(
+            TRUSTEES
+                .iter()
+                .map(|&t| handed.iter().filter(|&&(x, _)| x == t).count()),
+        ),
+    )?;
 
     // 07, 08: a line through the secret (any two of five)
     let line = Recipe::new(SECRET, &[LINE_SLOPE]);
@@ -167,10 +210,21 @@
     let tall = line.height(TEN_HOLDER);
     f.add("S-F4.paper.holder", TEN_HOLDER)?;
     f.add("S-F4.paper.share", tall)?;
-    f.add(
-        "S-F4.paper.candidates",
-        csv(&model::paper_line_secrets(MARKS, TEN_HOLDER, tall)),
-    )?;
+    let paper = model::paper_line_secrets(MARKS, TEN_HOLDER, tall);
+    f.add("S-F4.paper.candidates", csv(&paper))?;
+    f.add("S-F4.paper.count", paper.len())?;
+    // 07: two points fix a line, one doesn't; the secret is its height at
+    // position 0, and each trustee sits at the position of its number
+    f.add("S-F4.needed", line.coefficients().len())?;
+    f.add("S-F4.too_few", line.coefficients().len() - 1)?;
+    f.add("S-F4.spin_count", line.spins.len())?;
+    let secret_at = (0..=TEN_HOLDER)
+        .find(|&x| line.height(x) == i64::from(SECRET))
+        .ok_or_else(|| "the line never has the secret's height".to_owned())?;
+    f.add("S-F4.secret_position", secret_at)?;
+    for &t in &TRUSTEES[..2] {
+        f.add(&format!("S-F4.position_of.{t}"), t)?;
+    }
     // neighbours: one position apart, so the climb between them is the spin
     let [(x0, u), (_, w)] = [ADJACENT[0], ADJACENT[1]].map(|x| (x, line.share(dial, x)));
     let step = dial.sub(w, u);
@@ -617,6 +671,13 @@
 S-F3.shamir.needed	6
 S-F3.shamir.locks	462
 S-F3.shamir.keys_each	252
+S-F3.shamir.too_few	5
+S-F3.locks_per_pair	1
+S-F3.pair_lock.holders	1,2,3
+S-F3.merged.pairs	2
+S-F3.merged.unions	3,4
+S-F3.fewest_locks	10
+S-F3.dealer.numbers_each	1
 S-F4.slope	4
 S-F4.heights	9,13,17,21,25
 S-F4.shares	9,2,6,10,3
@@ -624,6 +685,13 @@
 S-F4.paper.holder	5
 S-F4.paper.share	25
 S-F4.paper.candidates	0,5,10
+S-F4.paper.count	3
+S-F4.needed	2
+S-F4.too_few	1
+S-F4.spin_count	1
+S-F4.secret_position	0
+S-F4.position_of.1	1
+S-F4.position_of.2	2
 S-F4.adjacent	2,3
 S-F4.adjacent.shares	2,6
 S-F4.adjacent.step	4
@@ -770,7 +838,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 196);
+        assert_eq!(f.len(), 210);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/src/model.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/model.rs
@@ -512,6 +512,9 @@
     OrdinaryPiece(u32),
     /// Trustees holding these shares of a recipe with `spins` fair spins.
     Shares { points: Vec<(u32, u32)>, spins: usize },
+    /// Trustees holding these heights on ordinary paper, nothing wrapped,
+    /// of a recipe with `spins` whole spins 0 to marks − 1 (chapter 07).
+    Paper { points: Vec<(u32, i64)>, spins: usize },
 }
 
 impl View {
@@ -525,6 +528,15 @@
                 .all()
                 .filter(|&s| !fitting_spins(dial, points, s, *spins).is_empty())
                 .collect(),
+            Self::Paper { points, spins } => dial
+                .all()
+                .filter(|&s| {
+                    tuples(dial, *spins).any(|t| {
+                        let r = Recipe::new(s, &t);
+                        points.iter().all(|&(x, h)| r.height(x) == h)
+                    })
+                })
+                .collect(),
         }
     }
 }
@@ -693,6 +705,46 @@
     }
 }
 
+/// The fewest locks of any plan, not only the padlock answer's, for which
+/// every group of `needed` of `trustees` opens every lock and every group
+/// of needed − 1 is stopped by one, by trying every plan. A lock is the set
+/// of trustees given its key; it is usable only if every `needed`-group
+/// holds a key to it, and it stops a group holding none. `None` if no plan
+/// works, or if there are more than [`PLANS_TRIED`] usable locks to try.
+pub fn fewest_locks(trustees: &[u32], needed: usize) -> Option<usize> {
+    let n = trustees.len();
+    if needed == 0 || needed > n || n >= 32 {
+        return None;
+    }
+    let mask = |g: &[u32]| -> u32 {
+        g.iter()
+            .filter_map(|t| trustees.iter().position(|x| x == t))
+            .fold(0, |m, i| m | (1 << i))
+    };
+    let must_open: Vec<u32> = combinations(trustees, needed).iter().map(|g| mask(g)).collect();
+    let must_stop: Vec<u32> = combinations(trustees, needed - 1).iter().map(|g| mask(g)).collect();
+    let usable: Vec<u32> = (0..1u32 << n)
+        .filter(|&h| must_open.iter().all(|&g| g & h != 0))
+        .collect();
+    if usable.len() > PLANS_TRIED {
+        return None;
+    }
+    (0..1u64 << usable.len())
+        .filter(|&plan| {
+            must_stop.iter().all(|&g| {
+                usable
+                    .iter()
+                    .enumerate()
+                    .any(|(i, &h)| (plan >> i) & 1 == 1 && g & h == 0)
+            })
+        })
+        .map(|plan| plan.count_ones() as usize)
+        .min()
+}
+
+/// The most usable locks [`fewest_locks`] tries every family of.
+pub const PLANS_TRIED: usize = 20;
+
 #[cfg(test)]
 mod tests {
     use super::*;
@@ -909,6 +961,57 @@
         assert_eq!(ten_pair.possible(TEN_DIAL), vec![SECRET]);
     }
 
+    /// On paper one height leaves the secrets a whole spin reaches it from,
+    /// as `paper_line_secrets` counts them; a curve's two heights leave
+    /// what `paper_whole_spin_secrets` does; two points of chapter 07's
+    /// line leave only the secret. A height one too high leaves another
+    /// line's secret (13 and 18 climb 5 a step from 3), and one that makes
+    /// the climb a fraction (13 and 22, 9/2 a step) leaves none.
+    #[test]
+    fn the_paper_view_agrees_with_the_paper_counts() {
+        let one = View::Paper {
+            points: vec![(5, 25)],
+            spins: 1,
+        };
+        assert_eq!(one.possible(DIAL), paper_line_secrets(MARKS, 5, 25));
+        assert_eq!(one.possible(DIAL), vec![0, 5, 10]);
+        let pair = [(4, 145), (5, 215)];
+        let two = View::Paper {
+            points: pair.to_vec(),
+            spins: 2,
+        };
+        assert_eq!(two.possible(DIAL), paper_whole_spin_secrets(MARKS, pair));
+        let line = Recipe::new(SECRET, &[LINE_SLOPE]);
+        let both = View::Paper {
+            points: vec![(2, line.height(2)), (3, line.height(3))],
+            spins: 1,
+        };
+        assert_eq!(both.possible(DIAL), vec![SECRET]);
+        let other = View::Paper {
+            points: vec![(2, line.height(2)), (3, line.height(3) + 1)],
+            spins: 1,
+        };
+        assert_eq!(other.possible(DIAL), vec![3]);
+        let none = View::Paper {
+            points: vec![(2, line.height(2)), (4, line.height(4) + 1)],
+            spins: 1,
+        };
+        assert!(none.possible(DIAL).is_empty());
+    }
+
+    /// No plan at all does with fewer locks than the padlock answer, for any
+    /// one, two or three of five; nothing works for none or six of five.
+    #[test]
+    fn the_padlock_answer_is_the_fewest() {
+        for needed in 1..=THRESHOLD {
+            let plan = Padlocks::new(&TRUSTEES, needed);
+            assert_eq!(fewest_locks(&TRUSTEES, needed), Some(plan.locks().len()), "{needed}");
+        }
+        assert_eq!(fewest_locks(&TRUSTEES, THRESHOLD), Some(10));
+        assert_eq!(fewest_locks(&TRUSTEES, 0), None);
+        assert_eq!(fewest_locks(&TRUSTEES, TRUSTEES.len() + 1), None);
+    }
+
     #[test]
     fn paper_leaks_where_the_dial_does_not() {
         assert_eq!(paper_line_secrets(MARKS, 5, 25), vec![0, 5, 10]);
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -189,6 +189,19 @@
                 for bribed in itertools.combinations(TRUSTEES, k) if set(holders) & set(bribed)), default=None)
 
 
+def fewest_locks(people, k):
+    """The fewest locks of any plan (a lock is the set of people holding its key) with
+    which every k of `people` open them all and every k - 1 are stopped by one: plans
+    are tried smallest first. None if none works."""
+    holder_sets = [set(s) for r in range(len(people) + 1) for s in itertools.combinations(people, r)]
+    usable = [h for h in holder_sets if all(h & set(g) for g in itertools.combinations(people, k))]
+    stop = [set(g) for g in itertools.combinations(people, k - 1)]
+    for n in range(1, len(usable) + 1):
+        if any(all(any(not (h & g) for h in plan) for g in stop) for plan in itertools.combinations(usable, n)):
+            return n
+    return None
+
+
 def facts():
     f = {}
 
@@ -277,6 +290,21 @@
     put("S-F3.shamir.locks", math.comb(SHAMIR_SCIENTISTS, SHAMIR_NEEDED - 1))
     put("S-F3.shamir.keys_each", math.comb(SHAMIR_SCIENTISTS - 1, SHAMIR_NEEDED - 1))
     check(math.comb(11, 5) == SHAMIR_LOCKS and math.comb(10, 5) == SHAMIR_KEYS, "Shamir's 462 and 252")
+    # "no five can": the most scientists who must stay locked out
+    put("S-F3.shamir.too_few", max(k for k in range(SHAMIR_SCIENTISTS + 1) if k < SHAMIR_NEEDED))
+    # one lock for each pair, and the lock for the pair has its keys with the three outside it
+    put("S-F3.locks_per_pair", csv(sorted({sum(1 for lk in pairs if set(lk) == set(g)) for g in pairs})))
+    put("S-F3.pair_lock.holders", csv(t for t in TRUSTEES if tuple(PAIR) in keys[t]))
+    # a lock stopping two different pairs keeps its key from everyone in either
+    two_pairs = list(itertools.combinations(pairs, 2))
+    put("S-F3.merged.pairs", len(two_pairs[0]))
+    put("S-F3.merged.unions", csv(sorted({len(set(p) | set(q)) for p, q in two_pairs})))
+    check(all(any(set(tr) <= set(p) | set(q) for tr in trios) for p, q in two_pairs),
+          "every lock serving two pairs shuts out a trio")
+    put("S-F3.fewest_locks", fewest_locks(TRUSTEES, THRESHOLD))
+    check(fewest_locks(TRUSTEES, 2) == len(TRUSTEES), "any two of five needs a lock per trustee")
+    # the dealer's method: one number for each trustee
+    put("S-F3.dealer.numbers_each", len(shares) // len(TRUSTEES))
 
     # 07, 08: a line through the secret (any two of five)
     put("S-F4.slope", LINE_SLOPE)
@@ -289,6 +317,21 @@
     put("S-F4.paper.holder", TEN_HOLDER)
     put("S-F4.paper.share", tall)
     put("S-F4.paper.candidates", csv(sorted({s for s in range(P) for a in range(P) if s + a * TEN_HOLDER == tall})))
+    put("S-F4.paper.count", len({s for s in range(P) for a in range(P) if s + a * TEN_HOLDER == tall}))
+
+    # 07: the fewest trustees whose shares always leave one secret of the line, by search
+    def line_left(xs):
+        got = [line_share(SECRET, LINE_SLOPE, x) for x in xs]
+        return {s for s in range(P) for a in range(P) if all(line_share(s, a, x) == y for x, y in zip(xs, got))}
+    need = min(k for k in range(1, len(TRUSTEES) + 1)
+               if all(len(line_left(xs)) == 1 for xs in itertools.combinations(TRUSTEES, k)))
+    put("S-F4.needed", need)
+    put("S-F4.too_few", need - 1)
+    put("S-F4.spin_count", len((LINE_SLOPE,)))
+    put("S-F4.secret_position", next(x for x in range(P) if SECRET + LINE_SLOPE * x == SECRET))
+    # each trustee sits at the position of its number: its share is the height there
+    for t in TRUSTEES[:2]:
+        put(f"S-F4.position_of.{t}", next(x for x in range(P) if line[t - 1] == line_share(SECRET, LINE_SLOPE, x)))
     u, w = (line[x - 1] for x in ADJACENT)
     step = (w - u) % P
     put("S-F4.adjacent", csv(ADJACENT))
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 05, per root's direction in Mail
+chapters 01 to 07, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -100,3 +100,27 @@
 | 05_4 | They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met. | precisely: "with fair spins, P(shares 2 and 6 \| s) = 1/121 for every secret s, so their belief about s is unchanged" [S-F7.chance_each]. At "every secret is exactly as likely" an outline glows round the tally row of 1s. | precisely card | thought after "what to guess." |
 | 05_5 | Two shares can't determine the secret, and three can. The rest of the story is why both are true. | The card goes. At "Two shares can't determine the secret" the line "2 shares → all 11 secrets still fit"; at "and three can" the line "3 shares → the secret" above it. The envelope stays sealed; its outline glows at "The rest of the story". | card | chapter_transition |
 
+## 06_padlocks: a way that works, and its price
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 06_1 | There is an older way to get any three of five, with padlocks. Put ten padlocks on the door, one for each pair of trustees. | The vault's door and lamp without the dial, and the rail. At "any three of five" the card "any 3 of 5" at the upper right [S-F1.threshold, S-F1.trustees]. At "Put ten padlocks on the door" ten padlocks come in one after another, two rows of five on the door, labelled 1·2, 1·3, 1·4, 1·5, 2·3 over 2·4, 2·5, 3·4, 3·5, 4·5 [S-F3.locks]. At "one for each pair of trustees" outlines glow round lock 1·2 and round trustees 1 and 2's card frames [S-F3.locks_per_pair]. | lagged, outline glow | thought after "with padlocks." |
+| 06_2 | Each padlock's key goes to everyone outside its pair. The lock for four and five has its keys with one, two and three. | At "The lock for four and five" a purple outline round lock 4·5, and trustees 4 and 5 turn purple. At "has its keys with one, two and three" three keys fly from lock 4·5 into the card frames of trustees 1, 2 and 3 [S-F3.pair_lock.holders], outlines glowing round those frames. The outline goes and 4 and 5 turn blue again. | outline glow, flight | thought after "outside its pair." |
+| 06_3 | Every trustee then carries six keys. Any three trustees together hold a key to every lock, because no pair contains all three of them. | The other 27 keys fly from their locks to the trustees outside each lock's pair, six slots to a frame. At "six keys" the card "6 keys each, 30 keys in all" under the rail [S-F3.keys_each, S-F3.keys_total]. At "Any three trustees together" trustees 1, 2 and 3 turn teal, outlines glowing round their frames; at "hold a key to every lock" all ten locks spring open, one after another [S-F3.trios_that_open]. At "because no pair contains all three of them" outlines glow round lock 1·2 with trustee 3's frame, then lock 4·5 with trustee 1's. | flight, lagged, outline glow | thought after "six keys." |
+| 06_4 | ⟨predict⟩ How many locks stop trustees four and five? | The locks spring shut; trustees 1, 2 and 3 blue again, 4 and 5 purple, outlines glowing round their frames. The think ring with its "?" at the right, under the rule card. | predict | question hold (think time) |
+| 06_5 | Just one, their own. And no lock can do that job for two different pairs, or it would also stop the three or four trustees in those pairs together. So ten locks is the fewest that works. | As the "?" goes, the nine locks 4 and 5 hold a key to spring open, one after another, and lock 4·5, still shut, gets a purple outline [S-F3.locks_a_pair_cannot_open]; at "their own" outlines glow round its key holders' frames, 1, 2 and 3. At "And no lock can do that job" the nine shut, the outline goes and 4 and 5 turn blue. Two merged locks come in right of the door, the labels 1·2 and 2·3 moving off the grid onto the first, 1·2 and 3·4 onto the second [S-F3.merged.pairs]. At "or it would also stop the three" the first's words come in, "keys only with 4 and 5" over "1, 2, 3 together: shut out" in red, as 1, 2 and 3 turn teal, red outlines glow round their frames and the lock's body swells red; at "or four trustees" the second's, "keys only with 5" over "1, 2, 3, 4 together: shut out" [S-F3.merged.unions]. At "So ten locks is the fewest" the card "every pair needs a lock of its own," over "so 10 pairs need at least 10 locks" [S-F3.fewest_locks], as an outline glows round the grid and the teal trustees turn blue. | outline glow, morph, swell | thought after "their own."; thought before "So ten"; visual hold after "works." until both merged rows and the card are read one after another |
+| 06_6 | Adi Shamir's nineteen seventy-nine paper, How to Share a Secret, opens with a bigger version, taken from an older book. Any six of eleven scientists must be able to open a cabinet, and no five can. | The merged locks, the card and the rule card go. As the line begins, the quote card at the upper right, verbatim: "Eleven scientists are working on a secret project. They wish to lock up the documents in a cabinet so that the cabinet can be opened if and only if six or more of the scientists are present." Source: Shamir, How to Share a Secret, Comm. ACM 22(11), 1979, p. 612, introducing Liu's problem (C. L. Liu, 1968) [S-X1.shamir.year]. At "Any six of eleven scientists" an outline glows round the card [S-F3.shamir.needed, S-F3.shamir.scientists]. | quote card, outline glow | thought after "an older book." |
+| 06_7 | The smallest answer is four hundred and sixty-two locks, with two hundred and fifty-two keys for each scientist. | As the line begins, under the quote, a table headed "locks" and "keys each", its rows "any 3 of 5 trustees" with 10 and 6 and "any 6 of 11 scientists", and under it Shamir's next sentence, verbatim: "It is not hard to show that the minimal solution uses 462 locks and 252 keys per scientist." (Shamir, p. 612). At "four hundred and sixty-two locks" the scientists' locks count up 0 → 462 [S-F3.shamir.locks]; at "two hundred and fifty-two keys" their keys 0 → 252 [S-F3.shamir.keys_each]; then small grey "= C(11, 5)" and "= C(10, 5)" under the counts. | table, count-up, quote card | hold on the two numbers until both cards are read |
+| 06_8 | Shamir's own answer is the dealer's method, with no locks at all. Each trustee carries one number from the same dial as the secret. | The cards and the table go. At "the dealer's method" the sealed envelope from 04_1 returns, lower than in 04. At "with no locks at all" the keys and the count go, the locks fade one after another, and the vault's dial comes back to the door. At "Each trustee carries one number" the envelope opens (flap, seal and "sealed" note go; the flap stands open) and the rail's cards 8, 3, 1, 2, 6 come in one after another [S-F6.shares, S-F3.dealer.numbers_each]. At "from the same dial as the secret" the property card under the envelope, verbatim: "The size of each piece does not exceed the size of the original data." (Shamir 1979, p. 613, property (1)), as outlines glow round all five frames and a gold ring round the dial. | morph, quote card, outline glow | chapter_transition; visual hold until the property card is read |
+
+## 07_a_line_through_the_secret: two of five on paper
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 07_1 | To see the dealer's method, start with a simpler rule, any two of five. Mark the secret as a point at position zero, height five. | The rail. At "start with a simpler rule" the card "a simpler rule: any 2 of 5" at the upper right [S-F4.needed, S-F1.trustees]. At "Mark the secret" a paper plot at the left, positions 0 to 5 along the bottom and heights 0 to 25 up the side, labelled every 5; copies of the rail's five discs lift onto positions 1 to 5, and a gold disc 0, "secret", sits under position 0. At "position zero" a gold ring glows round it [S-F4.secret_position]; at "height five" a gold point at (0, 5) swells and an outline glows round the side's "5" [S-F1.secret]. | axes, identity kept, swell, outline glow | thought after "any two of five." |
+| 07_2 | The dealer spins once, and say it gives four. Draw the straight line that starts at the secret and climbs four for every step to the right. | The yellow spare dial, "dealer's fair spin", right of the plot. At "spins once" it turns twice round and stops on 4 [S-F4.spin_count]; at "say it gives four" a "+4" by the mark, a ring glowing round it [S-F4.slope]. At "Draw the straight line" the line grows from (0, 5) to (5, 25). At "climbs four for every step" the "+4" moves onto the first step, and each step's run (grey) and rise (yellow) draw in turn, a "+4" by every rise. | spin, line, lagged | thought after "gives four." |
+| 07_3 | Trustee one sits at position one, trustee two at position two, and so on. Each trustee's share is the line's height at their own position. Nine, thirteen, seventeen, twenty-one and twenty-five. | The spare dial goes. At "Trustee one sits at position one", "trustee two at position two" and "and so on" rings glow round those position discs and outlines round the same trustees' card frames [S-F4.position_of.1, S-F4.position_of.2]. At "Each trustee's share is the line's height" grey guides rise from positions 1 to 5 to the line, with blue points where they meet. At "Nine, thirteen, seventeen, twenty-one and twenty-five" each height's label comes in by its point as it is said, and the same number drops into that trustee's card [S-F4.heights]. | lagged labels | connected list (no pause between heights) |
+| 07_4 | Any two trustees can lay down their two points and redraw the line, and the line tells them the height at zero. | The line, its steps and guides, the secret's point and trustees 1, 4 and 5's points, labels and cards go, as rings and outlines glow round trustees 2 and 3 [S-F4.needed]. At "lay down their two points" their points swell. At "redraw the line" a line is drawn from (3, 17) back through (2, 13) to position 0; at "the height at zero" a gold point at (0, 5) swells and an outline glows round the side's "5" [S-F4.secret_position]. | line, swell, outline glow | thought after "redraw the line," |
+| 07_5 | One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. ⟨predict⟩ Which secrets could go with that? | Trustees 2 and 3's line, points and cards go; only trustee 5's point (5, 25) and its label [S-F4.paper.share]. At "not enough to redraw it" a grey line pivots through (5, 25), its climb sweeping down and up past 4 [S-F4.too_few]. At "on ordinary paper" the strip comes in, all eleven lit, "for: anyone outside". At "Trustee five holds twenty-five" the 25 drops into trustee 5's card, a ring and an outline glowing round its disc and frame [S-F4.paper.holder]. No fan and no count yet: the think ring with its "?" at the right, the strip still at 11. | pivot, strip, predict | question hold (think time) |
+| 07_6 | Just three, zero, five or ten. No other secret is twenty-five take away five times a spin. | The "?" goes. A gold outline marks heights 0 to 10 up the side, and the fan comes in from (5, 25), one faint line per whole spin 0 to 10, each meeting position 0 at 25 − 5 × spin: spins 0 to 2 meet it above 10 (at 25, 20 and 15), spins 6 to 10 reach the bottom axis first (below 0 at position 0). At "zero, five or ten" the three lines meeting position 0 inside 0 to 10, spins 5, 4 and 3, turn gold with gold points at 0, 5 and 10, as the strip drops from 11 to 3, "for: trustee 5, on paper" [S-F4.paper.candidates, S-F4.paper.count]. At "No other secret" the other eight fade to ghost and a precisely card: "secret = 25 − 5 × spin; spins 0 to 10 give 25, 20, 15, 10, 5, 0, −5, −10, −15, −20, −25; only 0, 5 and 10 are secrets 0 to 10". | fan, highlight, live count, precisely card | thought after "zero, five or ten."; visual hold until the card is read |
+| 07_7 | Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead. | The card goes. At "Ordinary numbers keep growing" a red outline glows along the side from 10 to 25; at "their size leaks" red outlines round trustee 5's "25" and card frame. At "just as B's piece did" the strip shows chapter 02's B holding 3, 8 lit, "for: B, holding 3 (chapter 02)" [S-F0.pieces.b, S-F0.pieces.b_alone_left]. At "So wrap the heights" a small dial comes in between the plot and the strip, and the gold outline round heights 0 to 10 glows: chapter 08 wraps onto it. | callback, outline glow | chapter_transition |
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -227,26 +227,26 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 06_1 | There is an older way to get any three of five, with padlocks. Put ten padlocks on the door, one for each pair of trustees. | Ten padlocks in a 2 × 5 grid on the door [S-F3.locks], labelled 1·2, 1·3, 1·4, 1·5, 2·3, 2·4, 2·5, 3·4, 3·5, 4·5. | grid, lagged | thought after "with padlocks." |
-| 06_2 | Each padlock's key goes to everyone outside its pair. The lock for four and five has its keys with one, two and three. | Lock 4·5 highlighted purple; three keys fly to trustees 1, 2, 3 on the rail. | highlight | thought after "outside its pair." |
-| 06_3 | Every trustee then carries six keys. Any three trustees together hold a key to every lock, because no pair contains all three of them. | Each trustee's key ring fills to 6 [S-F3.keys_each] (30 keys in all [S-F3.keys_total]). Trustees 1, 2, 3 teal: all ten locks spring open, lagged [S-F3.trios_that_open]. | count-up, lagged | thought after "six keys." |
-| 06_4 | ⟨predict⟩ How many locks stop trustees four and five? | Locks re-shut; trustees 4 and 5 purple with their key rings. | predict | question hold (think time) |
-| 06_5 | Just one, their own. And no lock can do that job for two different pairs, or it would also stop the three or four trustees in those pairs together. So ten locks is the fewest that works. | Nine locks open for 4 and 5; lock 4·5 stays shut [S-F3.locks_a_pair_cannot_open]. Then the labels 1·2 and 2·3 crowd onto one lock, which shuts out "1, 2, 3" in red. | flash, morph | thought after "their own."; thought before "So ten" |
-| 06_6 | Adi Shamir's nineteen seventy-nine paper, How to Share a Secret, opens with a bigger version, taken from an older book. Any six of eleven scientists must be able to open a cabinet, and no five can. | Quote card, verbatim: "Eleven scientists are working on a secret project. They wish to lock up the documents in a cabinet so that the cabinet can be opened if and only if six or more of the scientists are present." Source: Shamir, How to Share a Secret, Comm. ACM 22(11), 1979, p. 612, section 1, where Shamir introduces the problem as Liu's (C. L. Liu, 1968) [S-X1.shamir.year]. | quote card | thought after "an older book." |
-| 06_7 | The smallest answer is four hundred and sixty-two locks, with two hundred and fifty-two keys for each scientist. | Under the quote, Shamir's words: "462 locks and 252 keys per scientist" [S-F3.shamir.locks, S-F3.shamir.keys_each]; small grey "= C(11, 5) and C(10, 5)". | count-up | hold on the two numbers |
-| 06_8 | Shamir's own answer is the dealer's method, with no locks at all. Each trustee carries one number from the same dial as the secret. | Locks dissolve; the sealed envelope from 04_1 returns and opens. The rail's cards 8, 3, 1, 2, 6. Quote, small: "The size of each piece does not exceed the size of the original data." (Shamir 1979, p. 613, property (1)). | morph, quote | chapter_transition |
+| 06_1 | There is an older way to get any three of five, with padlocks. Put ten padlocks on the door, one for each pair of trustees. | The vault's door and lamp without the dial, and the rail. At "any three of five" the card "any 3 of 5" at the upper right [S-F1.threshold, S-F1.trustees]. At "Put ten padlocks on the door" ten padlocks come in one after another, two rows of five on the door, labelled 1·2, 1·3, 1·4, 1·5, 2·3 over 2·4, 2·5, 3·4, 3·5, 4·5 [S-F3.locks]. At "one for each pair of trustees" outlines glow round lock 1·2 and round trustees 1 and 2's card frames [S-F3.locks_per_pair]. | lagged, outline glow | thought after "with padlocks." |
+| 06_2 | Each padlock's key goes to everyone outside its pair. The lock for four and five has its keys with one, two and three. | At "The lock for four and five" a purple outline round lock 4·5, and trustees 4 and 5 turn purple. At "has its keys with one, two and three" three keys fly from lock 4·5 into the card frames of trustees 1, 2 and 3 [S-F3.pair_lock.holders], outlines glowing round those frames. The outline goes and 4 and 5 turn blue again. | outline glow, flight | thought after "outside its pair." |
+| 06_3 | Every trustee then carries six keys. Any three trustees together hold a key to every lock, because no pair contains all three of them. | The other 27 keys fly from their locks to the trustees outside each lock's pair, six slots to a frame. At "six keys" the card "6 keys each, 30 keys in all" under the rail [S-F3.keys_each, S-F3.keys_total]. At "Any three trustees together" trustees 1, 2 and 3 turn teal, outlines glowing round their frames; at "hold a key to every lock" all ten locks spring open, one after another [S-F3.trios_that_open]. At "because no pair contains all three of them" outlines glow round lock 1·2 with trustee 3's frame, then lock 4·5 with trustee 1's. | flight, lagged, outline glow | thought after "six keys." |
+| 06_4 | ⟨predict⟩ How many locks stop trustees four and five? | The locks spring shut; trustees 1, 2 and 3 blue again, 4 and 5 purple, outlines glowing round their frames. The think ring with its "?" at the right, under the rule card. | predict | question hold (think time) |
+| 06_5 | Just one, their own. And no lock can do that job for two different pairs, or it would also stop the three or four trustees in those pairs together. So ten locks is the fewest that works. | As the "?" goes, the nine locks 4 and 5 hold a key to spring open, one after another, and lock 4·5, still shut, gets a purple outline [S-F3.locks_a_pair_cannot_open]; at "their own" outlines glow round its key holders' frames, 1, 2 and 3. At "And no lock can do that job" the nine shut, the outline goes and 4 and 5 turn blue. Two merged locks come in right of the door, the labels 1·2 and 2·3 moving off the grid onto the first, 1·2 and 3·4 onto the second [S-F3.merged.pairs]. At "or it would also stop the three" the first's words come in, "keys only with 4 and 5" over "1, 2, 3 together: shut out" in red, as 1, 2 and 3 turn teal, red outlines glow round their frames and the lock's body swells red; at "or four trustees" the second's, "keys only with 5" over "1, 2, 3, 4 together: shut out" [S-F3.merged.unions]. At "So ten locks is the fewest" the card "every pair needs a lock of its own," over "so 10 pairs need at least 10 locks" [S-F3.fewest_locks], as an outline glows round the grid and the teal trustees turn blue. | outline glow, morph, swell | thought after "their own."; thought before "So ten"; visual hold after "works." until both merged rows and the card are read one after another |
+| 06_6 | Adi Shamir's nineteen seventy-nine paper, How to Share a Secret, opens with a bigger version, taken from an older book. Any six of eleven scientists must be able to open a cabinet, and no five can. | The merged locks, the card and the rule card go. As the line begins, the quote card at the upper right, verbatim: "Eleven scientists are working on a secret project. They wish to lock up the documents in a cabinet so that the cabinet can be opened if and only if six or more of the scientists are present." Source: Shamir, How to Share a Secret, Comm. ACM 22(11), 1979, p. 612, introducing Liu's problem (C. L. Liu, 1968) [S-X1.shamir.year]. At "Any six of eleven scientists" an outline glows round the card [S-F3.shamir.needed, S-F3.shamir.scientists]. | quote card, outline glow | thought after "an older book." |
+| 06_7 | The smallest answer is four hundred and sixty-two locks, with two hundred and fifty-two keys for each scientist. | As the line begins, under the quote, a table headed "locks" and "keys each", its rows "any 3 of 5 trustees" with 10 and 6 and "any 6 of 11 scientists", and under it Shamir's next sentence, verbatim: "It is not hard to show that the minimal solution uses 462 locks and 252 keys per scientist." (Shamir, p. 612). At "four hundred and sixty-two locks" the scientists' locks count up 0 → 462 [S-F3.shamir.locks]; at "two hundred and fifty-two keys" their keys 0 → 252 [S-F3.shamir.keys_each]; then small grey "= C(11, 5)" and "= C(10, 5)" under the counts. | table, count-up, quote card | hold on the two numbers until both cards are read |
+| 06_8 | Shamir's own answer is the dealer's method, with no locks at all. Each trustee carries one number from the same dial as the secret. | The cards and the table go. At "the dealer's method" the sealed envelope from 04_1 returns, lower than in 04. At "with no locks at all" the keys and the count go, the locks fade one after another, and the vault's dial comes back to the door. At "Each trustee carries one number" the envelope opens (flap, seal and "sealed" note go; the flap stands open) and the rail's cards 8, 3, 1, 2, 6 come in one after another [S-F6.shares, S-F3.dealer.numbers_each]. At "from the same dial as the secret" the property card under the envelope, verbatim: "The size of each piece does not exceed the size of the original data." (Shamir 1979, p. 613, property (1)), as outlines glow round all five frames and a gold ring round the dial. | morph, quote card, outline glow | chapter_transition; visual hold until the property card is read |
 
 ## 07_a_line_through_the_secret: two of five on paper
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 07_1 | To see the dealer's method, start with a simpler rule, any two of five. Mark the secret as a point at position zero, height five. | Paper plot: the rail's discs slide up onto positions 1 to 5 along the bottom; position 0 gold, "secret"; heights 0 to 25 up the side. Gold point at (0, 5) [S-F1.secret]. | axes, point, identity kept | thought after "any two of five." |
-| 07_2 | The dealer spins once, and say it gives four. Draw the straight line that starts at the secret and climbs four for every step to the right. | The yellow spare dial spins to 4 [S-F4.slope]. A line grows from (0, 5) rising 4 per step, with "+4" step marks. | spin, line | thought after "gives four." |
-| 07_3 | Trustee one sits at position one, trustee two at position two, and so on. Each trustee's share is the line's height at their own position. Nine, thirteen, seventeen, twenty-one and twenty-five. | Blue points and labels at positions 1 to 5: 9, 13, 17, 21, 25 [S-F4.heights]; each value also drops into that trustee's card on the rail. | lagged labels | connected list (no pause between heights) |
-| 07_4 | Any two trustees can lay down their two points and redraw the line, and the line tells them the height at zero. | Only trustees 2 and 3's points stay; the line is redrawn through them and extended left to (0, 5), which flashes gold. | flash | thought after "redraw the line," |
-| 07_5 | One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. ⟨predict⟩ Which secrets could go with that? | Only trustee 5's point (5, 25) [S-F4.paper.share]; a fan of faint lines through it, one per spin 0 to 10, each meeting position 0 at 25 − 5 × spin (most far below 0 or above 25). | fan, predict | question hold (think time) |
-| 07_6 | Just three, zero, five or ten. No other secret is twenty-five take away five times a spin. | Only three fan lines reach position 0 between 0 and 10, spins 5, 4, 3 at 0, 5, 10 [S-F4.paper.candidates]; the rest fade red; the strip shows 3 lit. | highlight, live count | thought after "zero, five or ten." |
-| 07_7 | Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead. | Callback inset: B's piece 3 with its strip of 8. The plot's tall axis folds down onto 0 to 10, ready for chapter 08. | callback, morph | chapter_transition |
+| 07_1 | To see the dealer's method, start with a simpler rule, any two of five. Mark the secret as a point at position zero, height five. | The rail. At "start with a simpler rule" the card "a simpler rule: any 2 of 5" at the upper right [S-F4.needed, S-F1.trustees]. At "Mark the secret" a paper plot at the left, positions 0 to 5 along the bottom and heights 0 to 25 up the side, labelled every 5; copies of the rail's five discs lift onto positions 1 to 5, and a gold disc 0, "secret", sits under position 0. At "position zero" a gold ring glows round it [S-F4.secret_position]; at "height five" a gold point at (0, 5) swells and an outline glows round the side's "5" [S-F1.secret]. | axes, identity kept, swell, outline glow | thought after "any two of five." |
+| 07_2 | The dealer spins once, and say it gives four. Draw the straight line that starts at the secret and climbs four for every step to the right. | The yellow spare dial, "dealer's fair spin", right of the plot. At "spins once" it turns twice round and stops on 4 [S-F4.spin_count]; at "say it gives four" a "+4" by the mark, a ring glowing round it [S-F4.slope]. At "Draw the straight line" the line grows from (0, 5) to (5, 25). At "climbs four for every step" the "+4" moves onto the first step, and each step's run (grey) and rise (yellow) draw in turn, a "+4" by every rise. | spin, line, lagged | thought after "gives four." |
+| 07_3 | Trustee one sits at position one, trustee two at position two, and so on. Each trustee's share is the line's height at their own position. Nine, thirteen, seventeen, twenty-one and twenty-five. | The spare dial goes. At "Trustee one sits at position one", "trustee two at position two" and "and so on" rings glow round those position discs and outlines round the same trustees' card frames [S-F4.position_of.1, S-F4.position_of.2]. At "Each trustee's share is the line's height" grey guides rise from positions 1 to 5 to the line, with blue points where they meet. At "Nine, thirteen, seventeen, twenty-one and twenty-five" each height's label comes in by its point as it is said, and the same number drops into that trustee's card [S-F4.heights]. | lagged labels | connected list (no pause between heights) |
+| 07_4 | Any two trustees can lay down their two points and redraw the line, and the line tells them the height at zero. | The line, its steps and guides, the secret's point and trustees 1, 4 and 5's points, labels and cards go, as rings and outlines glow round trustees 2 and 3 [S-F4.needed]. At "lay down their two points" their points swell. At "redraw the line" a line is drawn from (3, 17) back through (2, 13) to position 0; at "the height at zero" a gold point at (0, 5) swells and an outline glows round the side's "5" [S-F4.secret_position]. | line, swell, outline glow | thought after "redraw the line," |
+| 07_5 | One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. ⟨predict⟩ Which secrets could go with that? | Trustees 2 and 3's line, points and cards go; only trustee 5's point (5, 25) and its label [S-F4.paper.share]. At "not enough to redraw it" a grey line pivots through (5, 25), its climb sweeping down and up past 4 [S-F4.too_few]. At "on ordinary paper" the strip comes in, all eleven lit, "for: anyone outside". At "Trustee five holds twenty-five" the 25 drops into trustee 5's card, a ring and an outline glowing round its disc and frame [S-F4.paper.holder]. No fan and no count yet: the think ring with its "?" at the right, the strip still at 11. | pivot, strip, predict | question hold (think time) |
+| 07_6 | Just three, zero, five or ten. No other secret is twenty-five take away five times a spin. | The "?" goes. A gold outline marks heights 0 to 10 up the side, and the fan comes in from (5, 25), one faint line per whole spin 0 to 10, each meeting position 0 at 25 − 5 × spin: spins 0 to 2 meet it above 10 (at 25, 20 and 15), spins 6 to 10 reach the bottom axis first (below 0 at position 0). At "zero, five or ten" the three lines meeting position 0 inside 0 to 10, spins 5, 4 and 3, turn gold with gold points at 0, 5 and 10, as the strip drops from 11 to 3, "for: trustee 5, on paper" [S-F4.paper.candidates, S-F4.paper.count]. At "No other secret" the other eight fade to ghost and a precisely card: "secret = 25 − 5 × spin; spins 0 to 10 give 25, 20, 15, 10, 5, 0, −5, −10, −15, −20, −25; only 0, 5 and 10 are secrets 0 to 10". | fan, highlight, live count, precisely card | thought after "zero, five or ten."; visual hold until the card is read |
+| 07_7 | Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead. | The card goes. At "Ordinary numbers keep growing" a red outline glows along the side from 10 to 25; at "their size leaks" red outlines round trustee 5's "25" and card frame. At "just as B's piece did" the strip shows chapter 02's B holding 3, 8 lit, "for: B, holding 3 (chapter 02)" [S-F0.pieces.b, S-F0.pieces.b_alone_left]. At "So wrap the heights" a small dial comes in between the plot and the strip, and the gold outline round heights 0 to 10 glows: chapter 08 wraps onto it. | callback, outline glow | chapter_transition |
 
 ## 08_the_line_on_the_dial: one share fits every secret
 
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -29,3 +29,18 @@
 05_3	No. Every secret fits exactly one way, and a different way each time. So the pair's two and six fit every secret equally well.
 05_4	They can still guess, but their shares can't tell them what to guess. With fair spins, every secret is exactly as likely as it was before they met.
 05_5	Two shares can't determine the secret, and three can. The rest of the story is why both are true.
+06_1	There is an older way to get any three of five, with padlocks. Put ten padlocks on the door, one for each pair of trustees.
+06_2	Each padlock's key goes to everyone outside its pair. The lock for four and five has its keys with one, two and three.
+06_3	Every trustee then carries six keys. Any three trustees together hold a key to every lock, because no pair contains all three of them.
+06_4	How many locks stop trustees four and five?
+06_5	Just one, their own. And no lock can do that job for two different pairs, or it would also stop the three or four trustees in those pairs together. So ten locks is the fewest that works.
+06_6	Adi Shamir's nineteen seventy-nine paper, How to Share a Secret, opens with a bigger version, taken from an older book. Any six of eleven scientists must be able to open a cabinet, and no five can.
+06_7	The smallest answer is four hundred and sixty-two locks, with two hundred and fifty-two keys for each scientist.
+06_8	Shamir's own answer is the dealer's method, with no locks at all. Each trustee carries one number from the same dial as the secret.
+07_1	To see the dealer's method, start with a simpler rule, any two of five. Mark the secret as a point at position zero, height five.
+07_2	The dealer spins once, and say it gives four. Draw the straight line that starts at the secret and climbs four for every step to the right.
+07_3	Trustee one sits at position one, trustee two at position two, and so on. Each trustee's share is the line's height at their own position. Nine, thirteen, seventeen, twenty-one and twenty-five.
+07_4	Any two trustees can lay down their two points and redraw the line, and the line tells them the height at zero.
+07_5	One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. Which secrets could go with that?
+07_6	Just three, zero, five or ten. No other secret is twenty-five take away five times a spin.
+07_7	Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead.
```
<!-- shamir-0607 patch end -->

The patch's sha256 is
bdaa40d530be3f213a5252cfa8273ee3c1d8c6888ad48bb04fb851ecaa9f4c72 (523
lines).

It was made against the live tree **with shamir-0405 applied**. The live
shared files were unchanged from 09:17Z to 10:27Z: `mod.rs` 9435dcc5…,
`main.rs` a7f52a59…, `facts.rs` 260afb41…, `model.rs` 667f4b07…,
`oracle.py` 2e9dffc6…, `storyboard.md` 153baf48…, `storyboard-draft.md`
f3cf3d6b… and `script.tsv` e52090ac….

The patch's base, and the result of applying it:

| File | after 0405 (base) | after 0607 |
|---|---|---|
| `src/chapters/mod.rs` | 232f13fd… | a587eeab… |
| `src/main.rs` | 47fde7e9… | a4186f59… |
| `src/facts.rs` | e41193ad… | 083281f8… |
| `src/model.rs` | 667f4b07… | 16e4bbce… |
| `oracle.py` | 0ddfefe3… | 332665c6… |
| `docs/storyboard.md` | 50fbff23… | f1a3f1fc… |
| `docs/storyboard-draft.md` | df98b004… | b2abe227… |
| `script.tsv` | 196b26d8… | eb1f06de… |

If root has changed any of these since, the hunks may need rebasing; the
content is the list above.

## Chapters 08 and 09, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment in Mail 6905, after it froze Mail 6903, and Mail 6906. The
ask:
- write new native chapters `08_the_line_on_the_dial` and
  `09_a_curve_for_three` in new `the_line_on_the_dial.rs` and
  `a_curve_for_three.rs`, any new helper in a newly reserved file, without
  changing the delivered plot, draw, envelope or two_keys helpers or the
  shared model, main, registry or scripts;
- teach real wraparound before relying on it: heights 9, 13, 17, 21, 25
  become 9, 2, 6, 10, 3 with visible full turns, then neighbours 2 and 3
  recover 5 by two explicit steps back;
- keep the one-share prediction unanswered until the fan and count;
- in 08, show the complete doubling permutation, each mark visited once,
  and tell equal likelihoods P(share | secret) apart from assuming a uniform
  prior;
- in 09, keep the vault's original five cards, define the second spin and
  the squared position concretely before the polynomial label, compute 19
  and 47 visibly, wrap 47 only after the prediction, and keep the smooth
  paper parabola separate from the discrete dial picture;
- emphasis by separate hollow outlines; extra setup and reading time are
  welcome, with no runtime target;
- real code and meaningful tests, cue phrases, numeric bindings, semantic
  holds, an integration proposal and exact hashes; cheap syntax and
  arithmetic checks only.

Mail 6906 adds root's 04_3 wording ("Like anyone, they can pick a mark at
random.") and the 04 demand card. Both are in this patch's base, unchanged.

**Status.**
- I wrote three new files and edited no registered or delivered file:
  `the_vault.rs`, `two_keys.rs`, `cutting_it_up.rs`, `the_dial.rs`,
  `envelope.rs`, `the_goal.rs`, `an_experiment.rs`, `padlocks.rs`,
  `plot.rs`, `a_line_through_the_secret.rs`, `draw.rs`, `glyphs.rs`,
  `mod.rs`, `model.rs`, `facts.rs`, `main.rs`, `oracle.py`, the scripts,
  the storyboards and Cargo. Root's frozen intakes were not touched.
- Nothing was compiled, rendered, synthesized or listened to.
- Bead mx-video-program-program-87do.46.3 stays in progress; I close
  nothing.

**Reservations.**
- Held: 8117 (`the_line_on_the_dial.rs`), 8118 (`a_curve_for_three.rs`),
  8119 (`docs/previous/ch0809-*/**`), 8136 (`board.rs`) and 7941 (this
  file).
- Released at 10:40:30Z, as Mail 6905 asked: the 06/07 leases 8034–8037.

**Prior bytes.** `docs/previous/ch0809-before-08-09-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section, sha256
d0a36b2f46405687c158c7d2754a449dc29aa74c18df5e29294900e7c654ae98.

`docs/previous/ch0809-superseded-section-20261024/WORK_STATUS.md` keeps
the first version of this section (the `MarkerWithDisc` one, never handed
off), sha256 c649db5383037ef346249edd59cb9821057fb4dcbe93a760116b51f67ea19a7e.
This section replaced it in place.

**Root's 06/07 integration (11:49–11:50Z), and how 08/09 follow it.** The
live shared files now equal the result of my 0607 patch exactly. Root then
changed three 06/07 files:
- `plot.rs` (5230d531… → 4c5474ac…): `Marker` lost its unread `disc`
  field, and `Axes::show` uses `parts()`;
- `padlocks.rs` (3187d465… → aa56b8cc…): the answer card's reading time is
  sequential after the quote (`read_on`), and three assertions on constants
  are now `const { assert!(..) }`;
- `a_line_through_the_secret.rs` (1d8174b2… → 5c379c3b…): `type FanLine`
  for `fan`'s return type, and one `const { assert!(..) }`;
- and at 11:58Z `plot.rs` again (→ e572fc8a…): the "height" title sits
  lower (`HEIGHT_TITLE_GAP` 0.14) to clear the top title-safe edge, and its
  test keeps 0.3 from the top. My files use neither plot's title gap nor
  its height title. My checks below ran with this `plot.rs`.

My 08 scene greys the other trustees' position discs in 08_5, so it reads
`Marker.disc`. As Mails 6918 and 6919 ask, the patch restores that field
and its build member in `plot.rs`. The hunk is made against root's current
`plot.rs` (e572fc8a…) and keeps `HEIGHT_TITLE_GAP` and `parts()`. Its only
consumer is 08. I did not edit the live `plot.rs`.

For about a quarter of an hour (12:00–12:14Z), before I had read those
mails, `board.rs` carried a copy of the marker (`MarkerWithDisc`). I
withdrew it, and `board.rs` is back to its earlier bytes (5ad93119…). The
superseded section is kept (see Prior bytes).

I also wrapped my seven constant-only assertions in `const { }`:
- 08: `READY_Y - READY_R > PLOT_Y0 + 0.3`, `THINK_Y + THINK_R <
  STRIP_CAPTION_Y - 0.2`, `SPIN_ROW_Y - ARROW_GAP > DOUBLE_ROW_Y + ARROW_GAP
  + 0.5` and `FRAME_Y + FRAME_H / 2.0 < SPIN_ROW_Y`;
- 09: three in one block (`SKETCH_FROM.1 > FRAME_Y + 0.6`, `SKETCH_TO.0 <
  SPIN_X[0] - SPARE_R * ARC_LABEL_IN - 0.3` and `SPIN_X[0] + SPARE_R *
  ARC_LABEL_IN < SPIN_X[1] - SPARE_R * ARC_LABEL_IN`).

The integration patch's earlier hunks needed no change. It now applies
straight to the live tree, with no 0607 step first, and it gains the
`plot.rs` hunk.

Root's other repairs are untouched by it: `padlocks.rs`'s sequential
quotation reading, `FanLine`, the `const` assertions, and the height-title
gap.

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `board.rs` | 499 | 5ad9311951cb6b237fc8fe0146b3fc9a569e2b84097464090f8da719cd46f305 |
| `the_line_on_the_dial.rs` | 1194 | 9d8132765e093cc7625489c7f02502fe90a3422cfbc290d4e216537d8f9ab381 |
| `a_curve_for_three.rs` | 1211 | d0f421dc9bfba2c7728967066637fe8cf53fc07653572cb745f2582ebcdf960b |

- **`board.rs`**: the dial board and its devices. Every shape is computed
  from the numbers passed in, and the tests check it against `Dial::wrap`.
  - `Grid` and `Board`: positions across, marks 0 to 10 up, a faint dot at
    every (position, mark), row and position labels, two edges and a title.
  - `wrapped`: the pieces a straight rise or fall breaks into at the edges.
  - `turns_and_mark`, `taken_off` ("25 − 11 − 11 = 3") and `turns_and_more`
    ("47 = 4 × 11 + 3").
  - `nearest_top` (the needle back to 0 the short way), and `coil_radius`
    and `coil` (one loop per whole turn, each loop further out).
  - `ring`, `create_lagged` and `now`.
  - 6 tests:
    - `the_wrapped_line_meets_every_share`;
    - `wrapped_lines_meet_at_the_edges`;
    - `turns_land_where_the_dial_wraps`;
    - `the_nearest_top_is_the_short_way_back`;
    - `a_coils_turns_stand_apart`;
    - `a_board_is_where_it_says`.
- **`the_line_on_the_dial.rs`**: `TheLineOnTheDial`, key
  `08_the_line_on_the_dial`. 7 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `two_steps_back_find_the_secret`, with planted negatives: one step too
    few or too many, or the wrong step, miss the secret;
  - `one_share_fits_every_secret_once`, with a planted negative: on ten
    marks the same share fits only five secrets;
  - `doubling_visits_every_mark_once`, with a negative control: on ten
    marks doubling hits only the even marks;
  - `the_card_is_two_lines`;
  - `the_layout_keeps_its_pieces_apart`.
- **`a_curve_for_three.rs`**: `ACurveForThree`, key `09_a_curve_for_three`.
  6 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `the_cards_are_the_vaults`, with a planted negative: chapter 05's other
    spins deal different cards from the same secret;
  - `any_three_points_pin_the_paper_curve`, which also shows that two points
    leave a fractional-spin curve for every secret;
  - `the_parabola_meets_every_point`;
  - `the_layout_keeps_its_pieces_apart`.

**Dependencies.**
- The three files import only `pub` items. A scan of their `use super::…`,
  `use crate::model::…` and inline paths, in the code and the tests, found
  all 150 names to be top-level `pub` items of the modules they name, as
  they are after root's 11:58Z changes.
- 08 also reads the field `plot::Marker::disc`, which the patch restores.
- They need no `model.rs` change.
- Their phrase tests read `crate::SCRIPT`, so they need the patch's
  `script.tsv` lines. `the_pictures_are_the_facts` needs the patch's
  `facts.rs`.
- My files need no edit at integration.

### How the brief is met

**Wraparound taught before it is used (08_1).**
- The plot from 07 stays: the line, the gold band of heights 0 to 10, the
  blue points 9 to 25 and the same numbers on the rail's cards.
- 07's small dial grows into a wheel right of the plot, and the line turns
  ghost.
- At each height as it is said, the needle goes back to 0 the short way.
  A blue coil winds that height round the wheel, one loop per whole turn,
  each loop a little further out so the turns can be counted. The needle
  follows and lands, and a ring glows at its mark: 9 stays 9 (no loop); 13
  goes once round to 2; 17 to 6; 21 to 10; 25 to 3 with two loops.
- Each landing's working appears under the wheel: "9 stays 9", "13 − 11 =
  2", "17 − 11 = 6", "21 − 11 = 10", "25 − 11 − 11 = 3".
- Each point then drops down its column onto its mark. Its card is emptied
  before the share drops in, so no number morphs into another in place.

**The same rule, wrapped (08_2).** The points slide onto the dial board in
the same columns: rows 0 to 10, dots at every mark, "on the dial: points
only".
- The same straight line, wrapped, is three grey pieces through every
  point. Each piece leaves at the top edge and comes back in at the bottom.
- "Start at the secret": the gold point swells and the needle turns to 5.
- "Turn on four for each step": five yellow steps. Each step that runs off
  the top comes back in at the bottom, the needle turns on 4 each time,
  "+4" sits between position discs, and rings mark each edge crossed.

**Neighbours, then two explicit steps back (08_3, 08_4).**
- 08_3: trustees 2 and 3 are ringed (points, discs, frames), and their row
  labels 2 and 6 are outlined.
  - A grey run, one position, shows "one step apart". A yellow rise from 2
    to 6 carries "6 − 2 = 4", and the wheel clicks 2 → 6 with a "+4" arc.
- 08_4: rings round position discs 1, then 0.
  - "Two take away four is nine": the needle clicks back four to 9 ("−4"
    arc). A yellow step from (2, 2) to (1, 9) runs off the bottom edge and
    back in at the top, with rings at both edges and "2 − 4 → 9".
  - "Nine take away four is five": the same to (0, 5), "9 − 4 → 5".
  - "The secret": a gold point at (0, 5) swells, with outlines round row
    label 5, the wheel's 5 and position disc 0. A 1 s hold follows.

**The one-share prediction stays open (08_5 → 08_6).**
- Trustee 2 alone. Everything else goes, and the other discs and frames
  turn ghost (the discs through `Marker.disc`).
- The rule "share = secret + 2 × spin" and, in blue, "trustee 2: 2 = secret
  + 2 × spin".
- No strip, no lines and no count: the think ring with its "?" (4 s, "pause
  for more time").
- The strip ("for: trustee 2", all eleven lit) comes only at 08_6's "All
  eleven". Then:
  - spin 0's line, with "spin 0: secret 2";
  - spin 1's, with "spin 1: secret 2 − 2 × 1 = 0";
  - the other nine wrapped lines one after another, each meeting position
    0 on its own mark (9, 7, 5, 3, 1, 10, 8, 6, 4), a gold point and a dot
    on the strip for each;
  - "spins 2 to 10: secrets 9, 7, 5, 3, 1, 10, 8, 6, 4" and an outline
    round position 0's column, every mark met once.

**The complete doubling permutation (08_7).**
- Two rows under the strip's ticks: "spin" 0 to 10 over "2 × spin".
- One yellow arrow per spin, down to its double, one after another as the
  doubles are said, with a ring on each double: 0 → 0 up to 5 → 10. Then
  the note "past 10, take away 11", and 6 → 1 up to 10 → 9.
- "Every mark, once": an outline round the doubles' row, every mark ringed
  exactly once. The test checks it is a permutation, and that the first
  double past the top is 6's.

**Equal likelihoods versus a uniform prior (08_8).** The precisely card
reads: "chance of share 2, given secret s: 1/11 for every s (one spin each).
Equal likelihoods, not a uniform prior: any belief about s survives".
- It is 136 characters, two lines at every wrap from 72 to 92.
- It uses no "|" glyph.
- It matches Mail 6906's point: the security claim leaves any prior as it
  was, rather than forcing 1/11 on a skewed guesser.
- "Just like B's seven" then shows chapter 03's B on the strip ("for: B,
  holding 7 (chapter 03)", all eleven lit; `check_strip` confirms).

**The vault's own cards (09).**
- The five cards chapter 06 handed out from the sealed envelope (8, 3, 1,
  2, 6) lie face down in the rail's frames from 09_1, so nothing answers
  09_5's question early.
- They turn over only in 09_6, one at each share as it is said: the back
  lifts away, then the number drops in.
- `the_cards_are_the_vaults` checks each wrapped height against the
  recipe's share (Horner's rule on the dial).

**The second spin and the square before the name (09_1–09_3).**
- 09_1: "the vault's rule: any 3 of 5"; a line sketched, then a curve over
  it.
  - Two yellow dials, "1st spin" and "2nd spin", each turning twice round
    to 7.
  - "any of the 121 pairs can come up, repeats and zeros included".
- 09_2: the formula, one term per clause as it is said, each tagged: "height
  = 5" (the secret), "+ 7 × position" (1st spin), "+ 7 × position ×
  position" (2nd spin).
  - Then the rows "position" 1 to 5 and "squared: position × position" 1,
    4, 9, 16, 25.
  - The rule is not named until 09_7.
- 09_3: "trustee 1: 5 + 7 × 1 + 7 × 1 × 1", "= 5 + 7 + 7", "= 19", and
  "trustee 2: 5 + 7 × 2 + 7 × 2 × 2", "= 5 + 14 + 28", "= 47". Cards stay
  face down.

**The paper parabola, apart from the dial (09_4).** A paper plot at the
left, "on paper: ordinary numbers", heights 0 to 220.
- 89, 145 and 215 come in as they are said.
- The parabola 5 + 7x + 7x² is drawn through all six points.
- A teal copy redraws through trustees 1, 3, 5 and then through 2, 3, 4
  ("any three of them pin it down").
- The smooth curve exists only on this plot. 09_5's board is titled "on the
  dial: points only, no curve" and never shows a curve.

**47 wrapped only after the prediction (09_5, 09_6).**
- 09_5: the empty board at the right, with outlines round the plot's 47 and
  trustee 2's face-down card. Nothing wraps: the think ring (4 s).
- 09_6: a small dial between plot and board. A blue coil winds 47, four
  loops and on, timed to land at "so it lands on three", with "47 = 4 × 11
  + 3" under it. Then:
  - (2, 3) on the board;
  - the other rows "19 = 1 × 11 + 8", "89 = 8 × 11 + 1", "145 = 13 × 11 +
    2" and "215 = 19 × 11 + 6";
  - each card turning over as its share is said;
  - "the vault's shares: 8, 3, 1, 2, 6", with outlines round all five
    frames and a 1.5 s hold.

**The name last (09_7).**
- "secret + spin × position + spin × position × position + …", then "a
  polynomial:" with an outline.
- A second board at the left with 08's points and wrapped line: "1 spin: a
  line, any 2 of 5", with rings on trustees 2 and 3.
- The right board gets "2 spins: a curve, any 3 of 5". The secret's gold
  point joins its five shares, and teal rings and outlines mark trustees 1,
  2 and 3.

**Emphasis.**
- Only rings, outlines (`envelope::outline`, `frame_halos`, `text_halo`)
  and shapes (points swell; discs, frames and lines change color) are
  emphasised.
- No typeset text is recolored, scaled or flashed. The draft's 08_4
  "flash" is gone.
- No number turns into another in place. A card is emptied before its new
  number drops in. In 08_1 a point's old label fades out where the point
  was while its new label fades in beside the point at its new mark (0.5 s
  together, at different heights).

**Numbers.** Every number on a card, label or row is formatted from
`model.rs` or named constants. `the_pictures_are_the_facts` (unrun) compares
them with `facts::facts()`.

### Spoken and visual deltas from the draft

- **Spoken: none.** The 15 lines' words, ids and titles are unchanged, so
  `docs/script-draft.tsv` is unchanged (0aee3711…).
- **Visual.** All 15 picture cells of 08_1–09_7 in
  `docs/storyboard-draft.md` are rewritten to describe the scenes as built.
  The rows above summarise them; the patch carries them in full.
- **Features cells.** 12 changed, for example 08_1 "morph, lagged" → "coil,
  morph, card swap" and 08_4 "arrows, flash" → "click, wrapped steps,
  swell, outline glow".
- **Hold cells.** One changed: 08_8 "chapter_transition" →
  "chapter_transition; visual hold until the card is read".
- `docs/storyboard.md` gets the same 08/09 sections and a short "Devices in
  chapters 08 and 09" note.

### Semantic holds

**Narration-side** (from the hold column, for the pause director):
- 08_1 and 08_7: connected lists, with "thought before 'Every mark, once.'"
  in 08_7;
- thoughts after:
  - "the rule is the same." (08_2);
  - "two and six." (08_3);
  - "is nine," (08_4);
  - "one for each spin." (08_6);
  - "a curve." (09_1);
  - "two hundred and fifteen." (09_4);
  - "lands on three." (09_6);
- a clause pause after "the secret," (09_2);
- question holds (think time) after 08_5 and 09_5;
- chapter transitions after 08_8 and 09_7.

**Scene-side** (in code):
- 08_1: 0.6 s after the list;
- 08_4: 1.0 s on the gold point (`HOLD_ON_SECRET`);
- 08_5 and 09_5: 4 s think rings (`draw::THINK_S`);
- 08_8: the card held until read at 15 characters a second from fully in
  (`read_by`, about 9.1 s; about 7 s past the line's end), then 0.5 s;
- 09_6: 1.5 s on the matched cards (`HOLD_MATCHED`);
- 09_7: 0.8 s.

### Timing and layout risks (estimates; nothing is measured)

- **Spoken length.** The storyboard estimates (words only) are 08 86.7 s,
  flagged ADVISORY ("longer chapter"), and 09 73.5 s.
- **Scene length.** The holds above, two think rings, the coils, the lagged
  reveals and the fades add roughly 15 s to 08 and 9 s to 09. That gives
  about 100 s and 82 s, as estimates. Nothing was cut for length.
- **Cue-bound moves can land late** if the narrator is faster than the
  estimate:
  - 13's coil is timed to "lands on two" (0.8–2.0 s);
  - 17, 21 and 25 get 0.8 s each, so 25's two loops are quick;
  - 47's coil is timed to "so it lands on three" (1.6–3.2 s);
  - 09_1's first spin (1.4 s, after 0.6 s of fades) runs from "The dealer
    spins twice" towards "say the dial gives seven";
  - 08_7's 11 lagged arrows run against the spoken doubles.
- **Layout.** Text sizes are estimated at 0.0055 units a point per
  character wide and 0.0065 a point tall. The 08_8 card wraps to 2 lines.
  Real widths may differ. The 08_6 wrapped lines and the 09_6 card turns
  are the busiest frames.
  - Root's height-title fix shows rendered glyphs can stand taller than
    this estimate. I therefore reran the layout mirror with root's 0.3 (in
    place of the kit's 0.25 `TEXT_MARGIN`) on every top, left and right
    bound: all 120 assertions still pass.
  - By estimate, the closest texts to the top are 09's polynomial line
    (0.37 from the edge) and 08's tallest point label (0.47).
- **Glyphs.** "−", "×" and "→" are in `glyphs.rs`'s probed DIGITS. "…" (09_7)
  is the only other non-ASCII character. No registered Shamir chapter
  typesets it yet, but chapters of bakery_algorithm, bayesian_search and
  capture_recapture do. I did not run the glyph probe.
- **Source style.** In string literals my files write these symbols as
  `\u{2212}`, `\u{d7}`, `\u{2192}` and `\u{2026}`, as
  `a_line_through_the_secret.rs` does. The probe sees the characters.

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

The scratch scripts named below live in this session's scratchpad, not in
the repository; my handoff mail quotes their results.

The results below are from the final runs (12:14–12:17Z), on the patch
below. The tree was built at 12:14Z: a fresh copy of the live topic (with
`tools/storyboard.py` and `rustfmt.toml`), with the patch below applied.
- On a second fresh copy of the live topic, `git apply --check` and
  `git apply` gave rc 0 and rc 0.
- A recursive diff of the two trees showed no difference.
- The three new files equal the live ones.
- The same suite on the first version (11:51–12:10Z, without the `plot.rs`
  hunk) also passed.

- **Formatting** (`rustfmt --check --edition 2024 --config-path
  rustfmt.toml`, a parse check only).
  - rc 0 on each of the three files, after formatting them in place with
    the same rustfmt (my own files only).
  - rc 0 on the applied tree's `main.rs`. That run parses every module.
- **The oracle on the applied tree.** `oracle.py --self-test` prints 234
  facts. stderr: `# self-test: ten-mark dial pairs that leak: 9 of 10` and
  `# 234 facts, 0 failed cross-checks`. rc 0.
  - The output's sha256 is
    06c229123a827fa3395f32758f9aea63a798b12b9383ba1c1af6002d1368cee6.
  - The live oracle prints 210 facts (stderr `# 210 facts, 0 failed
    cross-checks`, rc 0). The patched one adds exactly the 24 rows below
    and removes none.
- **The golden table** (`check_golden.py`, mirroring `oracle_diff` and
  `f.len()`). 234 golden rows (234 distinct), against the oracle's 234,
  with `f.len()` asserted 234. Diff: none. rc 0.
- **Spoken numbers** (`check_spoken.py`, mirroring
  `every_spoken_number_is_the_fact_it_speaks_for`). Patched `main.rs`: 158
  `SPOKEN` rows, 7 `NOT_A_COUNT` and 40 number words. All 60 lines of the
  patched `script.tsv`: 296 number words, 0 failures. The planted control
  was caught. rc 0.
- **Cues** (`check_cues.py`).
  - Against the applied `script.tsv` and against `docs/script-draft.tsv`:
    - 08 says 08_1–08_8 in order: 34 cue uses (32 distinct phrases, 31
      waits);
    - 09 says 09_1–09_7 in order: 37 cue uses (36 distinct, 35 waits);
    - every phrase occurs exactly once in its line and in spoken order;
    - 0 failures, rc 0.
  - A planted copy of the script with altered lines gives 5 failures, among
    them `OUT OF ORDER in 09_6`, rc 1.
- **Storyboards** (a copy of `tools/storyboard.py` in the scratch tree,
  which writes only there).
  - `--check shamir_secret_sharing`: rc 0, with the estimates above and a
    total of 11.5 min over 9 chapters.
  - `check_sb.py`: 0 failures, rc 0:
    - production chapters 01–09 equal the draft's, row for row;
    - in the draft only the 08/09 rows changed, and only outside their
      words, titles and ids;
    - the draft's generated lines are unchanged by this patch;
    - `script.tsv` is the live 45 lines plus exactly the draft's 15 lines
      for 08 and 09: 60.
  - It also reports a staleness already in the live tree (below).
- **Independent arithmetic** (`evidence.py`). Recomputed by enumeration or
  repeated subtraction, not from `oracle.py` or the Rust model, and compared
  with the strings the unrun tests expect. 38 checks, 0 failures, rc 0:
  - **winding.** 9, 13, 17, 21, 25 → 9, 2, 6, 10, 3 with 0, 1, 1, 1, 2
    turns; the five working texts.
  - **the wrapped line.** Sampled finely, it is 3 pieces, all between the
    edges, meeting the secret and every share.
  - **neighbours and back.**
    - 2 and 6 differ by 4.
    - Back 2 → 9 → 5, the first step running off the bottom.
    - **Control:** one or three steps back miss 5.
  - **one share.**
    - Trustee 2's 2 gives secrets 2, 0, 9, 7, 5, 3, 1, 10, 8, 6, 4 for
      spins 0–10: a permutation, one secret per spin.
    - **Control:** ten marks leave {0, 2, 4, 6, 8}.
  - **doubling.**
    - 0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9: every mark once, the first past
      the top at 6.
    - **Control:** ten marks hit 5.
  - **08_8.**
    - B's 7.
    - The card (136 characters) is 2 lines at every wrap from 72 to 92,
      has no "|", and names 1/11 and the uniform prior.
    - **Control:** a long text wraps past two lines.
  - **09.**
    - Heights 19, 47, 89, 145, 215; terms 5 + 7 + 7 and 5 + 14 + 28.
    - Shares 8, 3, 1, 2, 6 with turns 1, 4, 8, 13, 19; the turn texts.
    - All 10 trios give 5 + 7x + 7x² by Fraction elimination.
    - Two points leave a curve for all 11 secrets.
    - The parabola stays under 220.
    - **Controls:**
      - the nudged trio (1, 19), (3, 89), (5, 216) gives the paper secret
        43/8;
      - a nudged height moves the curve;
      - chapter 05's other spins deal 6, 9, 3, 10, 8.
- **Layout mirror** (`layout_mirror.py`). The constants and `Grid`s are read
  out of the Rust sources, and the layout assertions of the three files are
  transcribed.
  - 120 assertions, 0 failures, rc 0.
  - A planted control (the wheel moved off the frame) fails as planted.
  - A copy with every frame-edge margin raised to root's 0.3
    (`strict_mirror_030_r2.py`, 9 bounds changed) also gives 120
    assertions, 0 failures, rc 0.
- **Imports** (`pubscan.py`). 150 names, 0 missing, rc 0.
  - A planted file is caught: one real item, one missing name and one
    private item (plot's `MARKER_NUMBER_SIZE`). It reports the 2 bad
    names, rc 1.
- **Lint mirrors** (textual; not Clippy). Each is calibrated on root's own
  11:50Z fixes.
  - **Constant-only `assert!`s** outside `const { }`: 0 in my files.
    - On the delivered 06/07 bytes it finds exactly root's four fixes
      (`padlocks.rs` 1248, 1291, 1292 and `a_line_through_the_secret.rs`
      769).
    - On every registered chapter as it is now, it finds 0.
  - **Dead code** (items or fields that no non-test code reads), over the
    applied tree for my three files and the patched `plot.rs`: only each
    scene's `construct`, a trait impl, as in every registered chapter.
    - The restored `Marker::disc` is read by 08_5.
    - On the delivered `plot.rs` without 08, the scan flags `Marker::disc`,
      the field root removed.
  - **Default-on patterns:** 0 leads in my three files and the patched
    `plot.rs`. The patterns are type_complexity,
    too_many_arguments, map_clone, assign_op_pattern, redundant_closure,
    vec_init_then_push, manual_range_contains, items_after_test_module,
    empty_line_after_doc_comments, doc_lazy_continuation,
    uninlined_format_args and literal casts.
    - On the delivered `a_line_through_the_secret.rs` it flags `fan`'s
      `Vec<([(f64, f64); 2], i64, bool)>` (score 260 over 250), which root
      aliased as `FanLine`.
  - **Unused imports:** `Timed` is reported in both scenes. It is a trait,
    and its methods are called there (`.rt(` and `.lag(`: 6 calls in 08's
    and 7 in 09's non-test code). The registered chapters import it the
    same way. A planted unused import is caught.
- **By reading, against live `model.rs` and `draw.rs`.** The new `facts.rs`
  code's calls and types match:
  - `Dial::sub`, `add`, `mul`, `wrap`, `all` and `times_row`;
  - `Ratio::int`, `plus`, `times` and `whole` (Ratio is Copy, PartialEq
    and Debug);
  - `paper_sum`, `paper_curve_spins` and `combinations`;
  - the u32/i64 bindings `u`, `w`, `step`, `x0`, `held`, `by_slope`,
    `line_heights` and `heights`.

  So do the scenes' `Spoken::at` / `wait_for` / `wait_end`. The restored
  `disc: d` is the same handle `build` already bundles (`Mob` is Copy, as
  the delivered field relied on).
- **One slip.** At 11:58Z I ran `cargo clippy --version` while looking up
  the toolchain. It is a version query, not a build. The rch hook printed a
  "falling back local" warning, and nothing else ran. The toolchain is
  `rustc 1.100.0-nightly (908501772 2026-08-30)`.

### Not verified

- **Nothing compiled.** No Cargo build, test or Clippy, beyond the version
  query above.
- **All 19 new tests are unrun:** `board.rs` 6, 08 7 and 09 6. So are the
  changed golden, `f.len()` and `SPOKEN` tests.
  - Borrow and type errors are possible. The types above were checked by
    reading only.
  - The lint mirrors are patterns, not Clippy. A nightly Clippy can have
    default lints they do not know.
- **Nothing rendered or watched.** The engine behaviours I rely on, read
  from the pinned source and the topic's helpers but not run:
  - a coil (`stroke` + `show_creation`) growing from mark 0 while `tween`
    drives the needle;
  - recoloring a disc that sits inside a `bundle` (`Marker.disc`, 08_5);
  - `AnimationGroup::with_lag_ratio` (`create_lagged`);
  - `Succession` for a card's empty-then-drop;
  - `Arrow` for the doubling rows;
  - `precisely`'s wrap.
- **Layout is estimated**, with the text sizes above.
- **Glyphs are unprobed.** The kit's `glyphs` probe renders, so it is root's
  to run.
- **No listening.** Nothing here claims accent consistency, and every
  duration above is an estimate.

### Already in the live tree, not from this patch

- **`docs/script-draft.tsv` 04_3** still reads "Like anyone, they can
  guess." `storyboard-draft.md` and `script.tsv` carry root's Mail 6906
  wording, "Like anyone, they can pick a mark at random." The patch leaves
  `script-draft.tsv` untouched (0aee3711…). Regenerating it from the draft
  storyboard (root's call; `tools/storyboard.py` writes only `script.tsv`)
  would bring the two together.
- **A stray hook file**, `explainer_corpus/shamir_secret_sharing/.ee/hook-state/session_start_orient.last.json`.
  A session-start hook wrote it, not I. I left it in place (no deletions).

### For root's integration (a proposal; I edited none of these files)

Apply it to the **live tree as it stood at 11:58Z** (after root's
height-title repair), as one unit, with nothing before it. The reason: `main.rs`'s `check_video` compares the
registry with the storyboard and the script. The patch includes
`script.tsv` as regenerated by `tools/storyboard.py shamir_secret_sharing`.

- **`src/chapters/mod.rs`.**
  - `pub mod a_curve_for_three;`, `pub mod board;` and
    `pub mod the_line_on_the_dial;`.
  - `08_the_line_on_the_dial` → `the_line_on_the_dial::TheLineOnTheDial`
    and `09_a_curve_for_three` → `a_curve_for_three::ACurveForThree`, each
    at intensity 0.5.
  - The doc reads "chapters 01 to 09 so far. The other fourteen …".
- **`src/model.rs`: no change.**
- **`src/facts.rs` and `oracle.py`.** 24 new facts. The golden table grows
  from 210 to 234 rows, `f.len()` from 210 to 234, and the module doc says
  "Chapters 01 to 09". The new facts:

  | Key | Value | Rust (model) | Python (oracle) |
  |---|---|---|---|
  | `S-F4.wrap.1`…`.5` | 9,0,9 · 13,1,2 · 17,1,6 · 21,1,10 · 25,2,3 | height, `div_euclid(11)`, `Dial::wrap` | `wind`: 11 taken away until under 11, checked against the line's shares |
  | `S-F4.adjacent.apart` | 1 | `ADJACENT[1] − ADJACENT[0]` | the same |
  | `S-F4.back.steps` | 2 | position of trustee 2 (steps to 0) | steps walked |
  | `S-F4.back.from` | 2 | `ADJACENT[0]` | `ADJACENT[0]` |
  | `S-F4.back.1`, `.2` | 2,4,9 · 9,4,5 | `Dial::sub` twice, an error unless it reaches `SECRET` | each step: the mark that turning on 4 carries here, by trying all 11 |
  | `S-F4.times2.of` | 0,…,10 | `dial.all()` | `range(P)` |
  | `S-F4.times2.hits_each` | 1 | how often each mark appears in `times_row(2)` | the same, by counting |
  | `S-F5.per_spin` | 1 | secrets fitting trustee 2's 2 for each spin, by search | the same |
  | `S-F5.spin.0`, `.1` | 2 · 0 | the existing `by_slope` | the existing `by_slope` |
  | `S-F5.spin.1.working` | 2,2,0 | share, 2 × 1, result | checked `(2 − 2) % 11` |
  | `S-F6.terms.1`, `.2` | 1,5,7,7,19 · 2,5,14,28,47 | terms from `SECRET`, `SPINS`, an error unless they add to `recipe.height` | the same terms, summed |
  | `S-F6.rest`, `.rest.heights` | 3,4,5 · 89,145,215 | `TRUSTEES[2..]`, `heights[2..]` | from its own curve heights |
  | `S-F6.paper.curves` | 1 | every trio's `paper_sum` and `paper_curve_spins`, each checked against all five points, deduplicated | Gaussian elimination over fractions per trio, checked against all five; must equal (5, 7, 7) |
  | `S-F6.asked` | 2,47 | trustee 2 and its height | the same |
  | `S-F6.wrap.2` | 47,4,3 | as `S-F4.wrap` | `wind`, checked against the vault's share |
  | `S-F6.turns` | 1,4,8,13,19 | `div_euclid(11)` of each height | `wind`'s turns |

  - **Honest note.** Several facts are bookkeeping for the spoken words or
    pictures, not independent arithmetic: `S-F4.adjacent.apart`,
    `S-F4.back.steps` and `.from` (both the position 2), `S-F4.times2.of`,
    `S-F6.rest` and `S-F6.asked`.
  - **New oracle cross-checks:**
    - winding equals divmod for every height below 250;
    - one share leaves every secret, one spin each, for every holder 1–10
      and share on eleven marks;
    - ten marks with holder 2 leave five (the control);
    - every recipe with spins 0–10 comes back from every trio on paper;
    - a nudged height moves the paper curve (the control).
- **`src/main.rs`.**
  - 48 `SPOKEN` rows for 08_1–09_7, for example:
    - ("08_1", "Seventeen lands on six", "S-F4.wrap.3", "17,1,6");
    - ("08_4", "nine take away four is five", "S-F4.back.2", "9,4,5");
    - ("08_7", "Every mark, once", "S-F4.times2.hits_each", "1");
    - ("09_5", "trustee two's forty-seven", "S-F6.asked", "2,47").
  - One new `NOT_A_COUNT`: ("09_7", "the secret plus spins times powers").
- **`docs/storyboard-draft.md`.** The 08/09 rows as described in the deltas
  above. Their words are unchanged, and nothing outside 08/09 changes.
- **`docs/storyboard.md`.**
  - "chapters 01 to 09" in its intro;
  - a "Devices in chapters 08 and 09" note;
  - the draft's 08 and 09 sections, appended verbatim.
- **`script.tsv`.** Regenerated: 60 lines.
- **`src/chapters/plot.rs`** (Mails 6918 and 6919). `Marker` gets back
  `pub disc: Mob` and `build` sets `disc: d`. The doc names its consumer:
  "08_5 greys the other trustees' discs while their numbers stay on them".
  - The hunk is made against root's e572fc8a… and touches nothing else.
  - `HEIGHT_TITLE_GAP`, `parts()` and the 0.3 title bound stay.
  - Apply it with 08's registration, which is what reads the field.
- **No other delivered helper changes.**

To extract the patch and apply it:

~~~sh
awk '/^<!-- shamir-0809 patch end -->$/{p=0} p; /^<!-- shamir-0809 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-0809.patch
git apply --check shamir-0809.patch && git apply shamir-0809.patch
~~~

<!-- shamir-0809 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,12 +1,14 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 07 so far. The other
-//! sixteen are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 09 so far. The other
+//! fourteen are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
+pub mod a_curve_for_three;
 pub mod a_line_through_the_secret;
 pub mod an_experiment;
+pub mod board;
 pub mod cutting_it_up;
 pub mod draw;
 pub mod envelope;
@@ -15,6 +17,7 @@
 pub mod plot;
 pub mod the_dial;
 pub mod the_goal;
+pub mod the_line_on_the_dial;
 pub mod the_vault;
 pub mod two_keys;
 
@@ -57,5 +60,15 @@
             make: |kit| Box::new(a_line_through_the_secret::ALineThroughTheSecret { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "08_the_line_on_the_dial",
+            make: |kit| Box::new(the_line_on_the_dial::TheLineOnTheDial { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "09_a_curve_for_three",
+            make: |kit| Box::new(a_curve_for_three::ACurveForThree { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -190,6 +190,98 @@
         ("07_6", "is twenty-five", "S-F4.paper.share", "25"),
         // the line through (5, 25) climbing a whole spin a step
         ("07_6", "five times a spin", "S-F4.paper.holder", "5"),
+        // each height wound round the dial: height, whole turns, mark
+        ("08_1", "Nine stays nine", "S-F4.wrap.1", "9,0,9"),
+        (
+            "08_1",
+            "Thirteen goes once round the dial and lands on two",
+            "S-F4.wrap.2",
+            "13,1,2",
+        ),
+        ("08_1", "Seventeen lands on six", "S-F4.wrap.3", "17,1,6"),
+        ("08_1", "twenty-one on ten", "S-F4.wrap.4", "21,1,10"),
+        ("08_1", "twenty-five on three", "S-F4.wrap.5", "25,2,3"),
+        ("08_2", "turn on four", "S-F4.slope", "4"),
+        ("08_3", "trustees two and three", "S-F4.adjacent", "2,3"),
+        ("08_3", "holding two and six", "S-F4.adjacent.shares", "2,6"),
+        ("08_3", "one step apart", "S-F4.adjacent.apart", "1"),
+        ("08_3", "differ by four", "S-F4.adjacent.step", "4"),
+        ("08_3", "each step adds four", "S-F4.adjacent.step", "4"),
+        ("08_4", "Step back twice", "S-F4.back.steps", "2"),
+        ("08_4", "from trustee two", "S-F4.back.from", "2"),
+        ("08_4", "Two take away four is nine", "S-F4.back.1", "2,4,9"),
+        ("08_4", "nine take away four is five", "S-F4.back.2", "9,4,5"),
+        ("08_5", "trustee two is alone", "S-F5.holder", "2"),
+        ("08_5", "holding two", "S-F5.share", "2"),
+        // twice: trustee 2's position times the spin
+        ("08_5", "the secret plus twice the spin", "S-F5.holder", "2"),
+        ("08_5", "the eleven marks", "S-F1.marks", "11"),
+        ("08_6", "All eleven", "S-F5.distinct", "11"),
+        ("08_6", "one for each spin", "S-F5.per_spin", "1"),
+        ("08_6", "Spin zero means the secret is two", "S-F5.spin.0", "2"),
+        // spin 1's secret, then its working: the share take away twice 1
+        ("08_6", "Spin one means", "S-F5.spin.1", "0"),
+        ("08_6", "two take away two, zero", "S-F5.spin.1.working", "2,2,0"),
+        (
+            "08_7",
+            "the doubles of zero to ten",
+            "S-F4.times2.of",
+            "0,1,2,3,4,5,6,7,8,9,10",
+        ),
+        (
+            "08_7",
+            "zero, two, four, six, eight, ten, then one, three, five, seven and nine",
+            "S-F4.times2",
+            "0,2,4,6,8,10,1,3,5,7,9",
+        ),
+        ("08_7", "Every mark, once", "S-F4.times2.hits_each", "1"),
+        ("08_8", "trustee two's share", "S-F5.holder", "2"),
+        ("08_8", "B's seven", "S-F2.b_share", "7"),
+        ("09_1", "any three", "S-F1.threshold", "3"),
+        ("09_1", "of five", "S-F1.trustees", "5"),
+        ("09_1", "spins twice", "S-F6.spin_count", "2"),
+        ("09_1", "gives seven, then seven again", "S-F6.spins", "7,7"),
+        (
+            "09_2",
+            "plus seven times their position, plus seven times their position squared",
+            "S-F6.spins",
+            "7,7",
+        ),
+        // position, the secret, the two spins' terms, the height
+        (
+            "09_3",
+            "Trustee one gets five plus seven plus seven, nineteen",
+            "S-F6.terms.1",
+            "1,5,7,7,19",
+        ),
+        (
+            "09_3",
+            "Trustee two gets five plus fourteen plus twenty-eight, forty-seven",
+            "S-F6.terms.2",
+            "2,5,14,28,47",
+        ),
+        ("09_4", "Three, four and five get", "S-F6.rest", "3,4,5"),
+        (
+            "09_4",
+            "eighty-nine, a hundred and forty-five, and two hundred and fifteen",
+            "S-F6.rest.heights",
+            "89,145,215",
+        ),
+        ("09_4", "one smooth curve", "S-F6.paper.curves", "1"),
+        ("09_4", "any three of them pin it down", "S-F1.threshold", "3"),
+        ("09_5", "trustee two's forty-seven", "S-F6.asked", "2,47"),
+        (
+            "09_6",
+            "Forty-seven is four elevens and three more, so it lands on three",
+            "S-F6.wrap.2",
+            "47,4,3",
+        ),
+        ("09_6", "All five shares", "S-F1.trustees", "5"),
+        ("09_6", "eight, three, one, two and six", "S-F6.shares", "8,3,1,2,6"),
+        ("09_7", "One spin gives a line", "S-F4.spin_count", "1"),
+        ("09_7", "for any two", "S-F4.needed", "2"),
+        ("09_7", "Two spins give a curve", "S-F6.spin_count", "2"),
+        ("09_7", "for any three", "S-F1.threshold", "3"),
     ];
 
     /// Number words said as an article, not a count, each with its line.
@@ -200,6 +292,7 @@
         ("02_6", "a second way"),
         ("02_7", "fixed first"),
         ("05_1", "every one of the"),
+        ("09_7", "the secret plus spins times powers"), // the rule's shape, no count
     ];
 
     const NUMBER_WORDS: &[&str] = &[
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -6,7 +6,7 @@
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
 //! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper and S-F7.join
 //! 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11 19 and 20,
-//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 07 are implemented so far; the
+//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 09 are implemented so far; the
 //! rest are computed and checked now so the draft's numbers are settled
 //! before its scenes exist.
 
@@ -234,6 +234,33 @@
     f.add("S-F4.adjacent.step", step)?;
     f.add("S-F4.adjacent.secret_unwrapped", back)?;
     f.add("S-F4.adjacent.secret", dial.wrap(back))?;
+    // 08: each height wrapped: the height, its whole turns, its mark
+    let m = i64::from(MARKS);
+    for (&t, &h) in TRUSTEES.iter().zip(&line_heights) {
+        f.add(
+            &format!("S-F4.wrap.{t}"),
+            csv(&[h, h.div_euclid(m), i64::from(dial.wrap(h))]),
+        )?;
+    }
+    f.add("S-F4.adjacent.apart", ADJACENT[1] - ADJACENT[0])?;
+    // 08: back from trustee 2 one step a position, by the neighbours' step,
+    // to the secret at position 0
+    let one_back = dial.sub(u, step);
+    let two_back = dial.sub(one_back, step);
+    if two_back != SECRET {
+        return Err(format!("back from {u} by {step} twice is {two_back}"));
+    }
+    f.add("S-F4.back.steps", x0)?;
+    f.add("S-F4.back.from", x0)?;
+    f.add("S-F4.back.1", csv(&[u, step, one_back]))?;
+    f.add("S-F4.back.2", csv(&[one_back, step, two_back]))?;
+    // 08: the doubles of every mark hit every mark once
+    let doubles = dial.times_row(2);
+    f.add("S-F4.times2.of", csv(&dial.all().collect::<Vec<_>>()))?;
+    f.add(
+        "S-F4.times2.hits_each",
+        set_csv(dial.all().map(|k| doubles.iter().filter(|&&d| d == k).count())),
+    )?;
 
     // 08: one share on the dial
     let held = line.share(dial, LINE_HOLDER);
@@ -243,6 +270,18 @@
     f.add("S-F5.secret_by_slope", csv(&by_slope))?;
     f.add("S-F5.distinct", distinct(by_slope.iter().copied()))?;
     f.add("S-F5.true_slope_secret", by_slope[LINE_SLOPE as usize])?;
+    // 08: the secrets each spin allows (one), spin 0's and spin 1's
+    f.add(
+        "S-F5.per_spin",
+        set_csv(dial.all().map(|a| {
+            dial.all()
+                .filter(|&s| dial.add(s, dial.mul(a, LINE_HOLDER)) == held)
+                .count()
+        })),
+    )?;
+    f.add("S-F5.spin.0", by_slope[0])?;
+    f.add("S-F5.spin.1", by_slope[1])?;
+    f.add("S-F5.spin.1.working", csv(&[held, LINE_HOLDER, by_slope[1]]))?;
 
     // 09: a curve for three
     f.add("S-F6.spins", csv(&SPINS))?;
@@ -251,6 +290,62 @@
     f.add("S-F6.shares", csv(&shares))?;
     f.add("S-F6.spin_pairs", model::tuples(dial, SPINS.len()).count())?;
     f.add("S-F6.spin_count", SPINS.len())?;
+    // 09: trustees 1 and 2's heights term by term (position, the secret,
+    // the first spin's term, the second's, the height), then the others'
+    for &x in &TRUSTEES[..2] {
+        let xi = i64::from(x);
+        let terms = [
+            i64::from(SECRET),
+            i64::from(SPINS[0]) * xi,
+            i64::from(SPINS[1]) * xi * xi,
+        ];
+        let total: i64 = terms.iter().sum();
+        if total != recipe.height(x) {
+            return Err(format!("trustee {x}'s terms add to {total}"));
+        }
+        f.add(
+            &format!("S-F6.terms.{x}"),
+            csv(&[xi, terms[0], terms[1], terms[2], total]),
+        )?;
+    }
+    f.add("S-F6.rest", csv(&TRUSTEES[2..]))?;
+    f.add("S-F6.rest.heights", csv(&heights[2..]))?;
+    // 09: on paper, every three of the five give back one curve, through
+    // all five points
+    let paper_points: Vec<(u32, i64)> = TRUSTEES.iter().copied().zip(heights.iter().copied()).collect();
+    let mut curves: Vec<(Ratio, Ratio, Ratio)> = Vec::new();
+    for trio in combinations(&TRUSTEES, THRESHOLD) {
+        let points: Vec<(u32, i64)> = paper_points.iter().copied().filter(|p| trio.contains(&p.0)).collect();
+        let s = model::paper_sum(&points);
+        let secret = s
+            .whole()
+            .and_then(|w| u32::try_from(w).ok())
+            .ok_or_else(|| format!("trustees {trio:?} give the paper secret {s:?}"))?;
+        let (a, b) = model::paper_curve_spins([points[0], points[1]], secret)
+            .ok_or_else(|| format!("trustees {trio:?} give no paper curve"))?;
+        let misses = paper_points.iter().any(|&(x, h)| {
+            let x = i64::from(x);
+            Ratio::int(i64::from(secret)).plus(a.times(x)).plus(b.times(x * x)) != Ratio::int(h)
+        });
+        if misses {
+            return Err(format!("trustees {trio:?}'s paper curve misses a point"));
+        }
+        if !curves.contains(&(s, a, b)) {
+            curves.push((s, a, b));
+        }
+    }
+    f.add("S-F6.paper.curves", curves.len())?;
+    // 09: trustee 2's 47 asked about, wrapped: four whole turns and 3; every
+    // height's whole turns
+    let asked = TRUSTEES[1];
+    let h = recipe.height(asked);
+    f.add("S-F6.asked", csv(&[i64::from(asked), h]))?;
+    f.add(
+        &format!("S-F6.wrap.{asked}"),
+        csv(&[h, h.div_euclid(m), i64::from(dial.wrap(h))]),
+    )?;
+    let turns: Vec<i64> = heights.iter().map(|h| h.div_euclid(m)).collect();
+    f.add("S-F6.turns", csv(&turns))?;
 
     // 05, 10: two shares, one spin pair per secret
     let pair_points = dealt(dial, &recipe, &PAIR);
@@ -697,16 +792,40 @@
 S-F4.adjacent.step	4
 S-F4.adjacent.secret_unwrapped	-6
 S-F4.adjacent.secret	5
+S-F4.wrap.1	9,0,9
+S-F4.wrap.2	13,1,2
+S-F4.wrap.3	17,1,6
+S-F4.wrap.4	21,1,10
+S-F4.wrap.5	25,2,3
+S-F4.adjacent.apart	1
+S-F4.back.steps	2
+S-F4.back.from	2
+S-F4.back.1	2,4,9
+S-F4.back.2	9,4,5
+S-F4.times2.of	0,1,2,3,4,5,6,7,8,9,10
+S-F4.times2.hits_each	1
 S-F5.holder	2
 S-F5.share	2
 S-F5.secret_by_slope	2,0,9,7,5,3,1,10,8,6,4
 S-F5.distinct	11
 S-F5.true_slope_secret	5
+S-F5.per_spin	1
+S-F5.spin.0	2
+S-F5.spin.1	0
+S-F5.spin.1.working	2,2,0
 S-F6.spins	7,7
 S-F6.heights	19,47,89,145,215
 S-F6.shares	8,3,1,2,6
 S-F6.spin_pairs	121
 S-F6.spin_count	2
+S-F6.terms.1	1,5,7,7,19
+S-F6.terms.2	2,5,14,28,47
+S-F6.rest	3,4,5
+S-F6.rest.heights	89,145,215
+S-F6.paper.curves	1
+S-F6.asked	2,47
+S-F6.wrap.2	47,4,3
+S-F6.turns	1,4,8,13,19
 S-F7.fitting	11
 S-F7.ways_each	1
 S-F7.spins_by_secret	1:4,0:9,10:3,9:8,8:2,7:7,6:1,5:6,4:0,3:5,2:10
@@ -838,7 +957,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 210);
+        assert_eq!(f.len(), 234);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -155,6 +155,32 @@
     return str(q.numerator) if q.denominator == 1 else f"{q.numerator}/{q.denominator}"
 
 
+def wind(h, p=P):
+    """The whole turns a height makes round a p-mark dial and the mark it lands on,
+    by taking p away until what is left is under p (chapters 08 and 09)."""
+    check(h >= 0, f"a height {h} below zero")
+    turns = 0
+    while h >= p:
+        h -= p
+        turns += 1
+    return turns, h
+
+
+def paper_curve(points):
+    """The ordinary curve s + a*x + b*x*x through three (x, height) points, solved by
+    Gaussian elimination over fractions: (s, a, b)."""
+    rows = [[Fraction(1), Fraction(x), Fraction(x * x), Fraction(h)] for x, h in points]
+    n = 3
+    for c in range(n):
+        pivot = next(r for r in range(c, n) if rows[r][c] != 0)
+        rows[c], rows[pivot] = rows[pivot], rows[c]
+        for r in range(n):
+            if r != c and rows[r][c] != 0:
+                f = rows[r][c] / rows[c][c]
+                rows[r] = [u - f * w for u, w in zip(rows[r], rows[c])]
+    return tuple(rows[i][3] / rows[i][i] for i in range(n))
+
+
 def fitting_spins(marks, values, s, p=P):
     """The spin pairs (a, b) that, with secret s, deal `values` at `marks`."""
     return [(a, b) for a in range(p) for b in range(p) if all(share((s, a, b), x, p) == y for x, y in zip(marks, values))]
@@ -339,6 +365,27 @@
     put("S-F4.adjacent.step", step)
     put("S-F4.adjacent.secret_unwrapped", u - ADJACENT[0] * step)
     put("S-F4.adjacent.secret", (u - ADJACENT[0] * step) % P)
+    # 08: each height wound round the dial, eleven taken away at a time
+    for t in TRUSTEES:
+        h = SECRET + LINE_SLOPE * t
+        turns, mark = wind(h)
+        check(mark == line[t - 1], f"height {h} winds onto {mark}, not trustee {t}'s share")
+        put(f"S-F4.wrap.{t}", csv((h, turns, mark)))
+    put("S-F4.adjacent.apart", ADJACENT[1] - ADJACENT[0])
+    # 08: step back from trustee 2, one position at a time: the mark that a turn on by
+    # the step would carry onto where we are, by trying every mark
+    here, walked = u, [u]
+    for _ in range(ADJACENT[0], 0, -1):
+        here = next(k for k in range(P) if (k + step) % P == here)
+        walked.append(here)
+    check(walked[-1] == SECRET, f"two steps back from {u} reach {walked[-1]}")
+    put("S-F4.back.steps", len(walked) - 1)
+    put("S-F4.back.from", ADJACENT[0])
+    put("S-F4.back.1", csv((walked[0], step, walked[1])))
+    put("S-F4.back.2", csv((walked[1], step, walked[2])))
+    # 08: doubling every mark, and how often each mark is hit
+    put("S-F4.times2.of", csv(range(P)))
+    put("S-F4.times2.hits_each", csv(sorted({sum(1 for x in range(P) if (2 * x) % P == k) for k in range(P)})))
 
     # 08: one share on the dial
     held = line[LINE_HOLDER - 1]
@@ -348,6 +395,13 @@
     put("S-F5.secret_by_slope", csv(by_slope))
     put("S-F5.distinct", len(set(by_slope)))
     put("S-F5.true_slope_secret", by_slope[LINE_SLOPE])
+    # 08: how many secrets each spin allows, by search, and spins 0 and 1's
+    per_spin = {len([s for s in range(P) if line_share(s, a, LINE_HOLDER) == held]) for a in range(P)}
+    put("S-F5.per_spin", csv(sorted(per_spin)))
+    put("S-F5.spin.0", by_slope[0])
+    put("S-F5.spin.1", by_slope[1])
+    check((held - LINE_HOLDER * 1) % P == by_slope[1], "spin 1: the share take away twice 1")
+    put("S-F5.spin.1.working", csv((held, LINE_HOLDER * 1, by_slope[1])))
 
     # 09: a curve for three
     put("S-F6.spins", csv(SPINS))
@@ -355,6 +409,30 @@
     put("S-F6.shares", csv(shares))
     put("S-F6.spin_pairs", P * P)
     put("S-F6.spin_count", len(SPINS))
+    # 09: trustees 1 and 2 term by term; the others' heights
+    curve_heights = {x: SECRET + SPINS[0] * x + SPINS[1] * x * x for x in TRUSTEES}
+    for x in TRUSTEES[:2]:
+        terms = (SECRET, SPINS[0] * x, SPINS[1] * x * x)
+        put(f"S-F6.terms.{x}", csv((x, *terms, sum(terms))))
+    put("S-F6.rest", csv(TRUSTEES[2:]))
+    put("S-F6.rest.heights", csv(curve_heights[x] for x in TRUSTEES[2:]))
+    # 09: on paper, the curves through every three of the five points (Gaussian
+    # elimination over fractions), each checked against all five
+    paper_curves = set()
+    for trio in itertools.combinations(TRUSTEES, THRESHOLD):
+        s, a, b = paper_curve([(x, curve_heights[x]) for x in trio])
+        check(all(s + a * x + b * x * x == h for x, h in curve_heights.items()),
+              f"trustees {trio}' paper curve misses a point")
+        paper_curves.add((s, a, b))
+    check(paper_curves == {(SECRET, *SPINS)}, f"paper curves {paper_curves}")
+    put("S-F6.paper.curves", len(paper_curves))
+    # 09: trustee 2's 47 asked about, then wound round; every height's turns
+    asked = TRUSTEES[1]
+    put("S-F6.asked", csv((asked, curve_heights[asked])))
+    turns, mark = wind(curve_heights[asked])
+    check(mark == shares[asked - 1], f"{curve_heights[asked]} winds onto {mark}")
+    put(f"S-F6.wrap.{asked}", csv((curve_heights[asked], turns, mark)))
+    put("S-F6.turns", csv(wind(curve_heights[x])[0] for x in TRUSTEES))
 
     # 05, 10: two shares
     pv = [shares[x - 1] for x in PAIR]
@@ -722,6 +800,23 @@
                     check(share(c, liar) != handed[liar - 1], f"four shares fit no curve, liar {liar}, recipe {r}")
     # negative control 5: on ten marks a nonzero line can have five zeros; on eleven, one
     check(f["S-F11.ten.line_most_zeros"] == "5" and f["S-F11.line_most_zeros"] == "1", "line zeros on ten and eleven")
+    # chapters 08 and 09: winding takes off whole turns and lands where the remainder
+    # does, for every height up to 250
+    check(all(wind(h)[0] * P + wind(h)[1] == h and wind(h)[1] == h % P for h in range(250)), "winding")
+    # chapter 08: one share leaves every secret, one spin each, for every holder and
+    # share on eleven marks; on ten marks holder 2 leaves only five (the control)
+    for x in range(1, P):
+        for y in range(P):
+            check(sorted((y - x * a) % P for a in range(P)) == list(range(P)), f"one share {x}={y}")
+    check(len({(2 - 2 * a) % TEN for a in range(TEN)}) == 5, "negative control: ten marks, holder 2 leaves five")
+    # chapter 09: on paper, any three of the five heights give back the recipe, for every
+    # recipe with spins 0 to 10; a height off by one gives another curve (the control)
+    for r in recipes():
+        tall = {x: r[0] + r[1] * x + r[2] * x * x for x in TRUSTEES}
+        for trio in itertools.combinations(TRUSTEES, THRESHOLD):
+            check(paper_curve([(x, tall[x]) for x in trio]) == r, f"paper curve {r} from {trio}")
+    nudged = [(1, 19), (2, 47), (3, 90)]
+    check(paper_curve(nudged) != (SECRET, *SPINS), "negative control: a nudged height moves the paper curve")
     return len(leaks)
 
 
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 07, per root's direction in Mail
+chapters 01 to 09, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -124,3 +124,54 @@
 | 07_5 | One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. ⟨predict⟩ Which secrets could go with that? | Trustees 2 and 3's line, points and cards go; only trustee 5's point (5, 25) and its label [S-F4.paper.share]. At "not enough to redraw it" a grey line pivots through (5, 25), its climb sweeping down and up past 4 [S-F4.too_few]. At "on ordinary paper" the strip comes in, all eleven lit, "for: anyone outside". At "Trustee five holds twenty-five" the 25 drops into trustee 5's card, a ring and an outline glowing round its disc and frame [S-F4.paper.holder]. No fan and no count yet: the think ring with its "?" at the right, the strip still at 11. | pivot, strip, predict | question hold (think time) |
 | 07_6 | Just three, zero, five or ten. No other secret is twenty-five take away five times a spin. | The "?" goes. A gold outline marks heights 0 to 10 up the side, and the fan comes in from (5, 25), one faint line per whole spin 0 to 10, each meeting position 0 at 25 − 5 × spin: spins 0 to 2 meet it above 10 (at 25, 20 and 15), spins 6 to 10 reach the bottom axis first (below 0 at position 0). At "zero, five or ten" the three lines meeting position 0 inside 0 to 10, spins 5, 4 and 3, turn gold with gold points at 0, 5 and 10, as the strip drops from 11 to 3, "for: trustee 5, on paper" [S-F4.paper.candidates, S-F4.paper.count]. At "No other secret" the other eight fade to ghost and a precisely card: "secret = 25 − 5 × spin; spins 0 to 10 give 25, 20, 15, 10, 5, 0, −5, −10, −15, −20, −25; only 0, 5 and 10 are secrets 0 to 10". | fan, highlight, live count, precisely card | thought after "zero, five or ten."; visual hold until the card is read |
 | 07_7 | Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead. | The card goes. At "Ordinary numbers keep growing" a red outline glows along the side from 10 to 25; at "their size leaks" red outlines round trustee 5's "25" and card frame. At "just as B's piece did" the strip shows chapter 02's B holding 3, 8 lit, "for: B, holding 3 (chapter 02)" [S-F0.pieces.b, S-F0.pieces.b_alone_left]. At "So wrap the heights" a small dial comes in between the plot and the strip, and the gold outline round heights 0 to 10 glows: chapter 08 wraps onto it. | callback, outline glow | chapter_transition |
+
+## Devices in chapters 08 and 09
+
+- **The dial board**: positions across, in chapter 07's columns over the
+  same position discs, and the dial's marks 0 to 10 up: a faint dot at every
+  position and mark, the row labels up the left, two edges and a title ("on
+  the dial: points only"). Every point sits on a mark and nothing lies
+  between marks. A straight rise that runs off the top edge leaves there and
+  comes back in at the bottom edge at the same position, so a wrapped line
+  is parallel pieces (`board.rs`, checked against `Dial::wrap`).
+- **The wheel and its coils**: chapter 07's small dial grows into a wheel
+  right of the plot (chapter 08), and a small dial sits between the plot and
+  the board (chapter 09). A height winds round it as a blue coil, one loop
+  for every whole turn and each loop a little further out so the turns can
+  be counted, before the needle lands on its mark; the working ("13 − 11 =
+  2", "47 = 4 × 11 + 3") sits under it.
+- **The doubling rows** (08_7): spins 0 to 10 over their doubles, one arrow
+  a spin, the doubles past 10 after the note "past 10, take away 11".
+- **The paper plot** (09_4): ordinary heights 0 to 220 at the left, apart
+  from the board. The smooth parabola is drawn only here; the board shows
+  points only ("no curve").
+- **Face-down cards** (09): the vault's five cards from chapter 06 lie face
+  down until 09_6, so no card answers 09_5's question; each turns over, its
+  back lifting away before its number drops in, as its share is said.
+- Emphasis is by rings and outlines of their own; no typeset text changes
+  color or size, and no number turns into another in place.
+
+## 08_the_line_on_the_dial: one share fits every secret
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 08_1 | Nine stays nine. Thirteen goes once round the dial and lands on two. Seventeen lands on six, twenty-one on ten, and twenty-five on three. | Chapter 07's plot as it ended: the line from (0, 5) to (5, 25), the gold outline round heights 0 to 10, the blue points labelled 9, 13, 17, 21 and 25 and the same numbers in the rail's cards [S-F4.heights]; chapter 07's small dial grows into a wheel right of the plot, and the line turns ghost. At each height as it is said, the needle goes back to 0 the short way and a blue coil winds that height round the wheel, one loop for every whole turn and each loop a little further out, while the needle follows and lands, a ring glowing at its mark: 9 stays 9 with no loop, 13 goes once round and lands on 2 as "lands on two" is said, then 17 on 6, 21 on 10 and 25 on 3, two loops [S-F4.wrap.1, S-F4.wrap.2, S-F4.wrap.3, S-F4.wrap.4, S-F4.wrap.5]. Each height's working comes in under the wheel, one row each: "9 stays 9", "13 − 11 = 2", "17 − 11 = 6", "21 − 11 = 10", "25 − 11 − 11 = 3". For 9 outlines glow round its label and card and nothing moves; every other point drops down its column onto its mark, its label is replaced, and its card is emptied before the share drops in [S-F4.shares]. | coil, morph, card swap | connected list |
+| 08_2 | On the dial the points look scattered, but the rule is the same. Start at the secret and turn on four for each step. | The plot's axes, line, outline and labels go; the points slide onto the dial board in the same columns over the same position discs: rows 0 to 10, a faint dot at every mark, the row labels up the left, two edges and "on the dial: points only", the gold point at (0, 5). At "but the rule is the same" the same straight line, wrapped: three grey pieces through every point, each leaving at the top edge and coming back in at the bottom at the same position; rings glow round the six points. At "Start at the secret" the gold point swells and the needle turns to 5. At "turn on four for each step" five yellow steps, one position each, every step that runs off the top coming back in at the bottom, the needle turning on 4 with each [S-F4.slope], a "+4" between each pair of position discs and rings at each edge a step crosses. | wrapped line, steps, swell | thought after "the rule is the same." |
+| 08_3 | Take trustees two and three, holding two and six. They are one step apart and differ by four, so each step adds four. | The steps, the wrapped line and the working go; the other points turn ghost. At "Take trustees two and three" rings round (2, 2) and (3, 6), round their position discs and outlines round their card frames [S-F4.adjacent]; at "holding two and six" outlines round the row labels 2 and 6 [S-F4.adjacent.shares]. At "one step apart" a grey run from (2, 2) to (3, 2) [S-F4.adjacent.apart]; at "differ by four" a yellow rise from (3, 2) up to (3, 6) with "6 − 2 = 4" beside it [S-F4.adjacent.step]. At "each step adds four" the wheel's needle goes to 2 and clicks on four marks to 6, a "+4" arc by the rim. | highlight, outline glow, run and rise, click | thought after "two and six." |
+| 08_4 | Step back twice from trustee two. Two take away four is nine, and nine take away four is five, the secret. | The run, rise and arc go and the needle goes back to 2. At "Step back twice" rings glow round position discs 1 and then 0 [S-F4.back.steps, S-F4.back.from]. At "Two take away four is nine" the needle clicks back four marks to 9 with a "−4" arc, and a yellow step from (2, 2) to (1, 9) runs off the bottom edge and back in at the top, rings at both edges, "2 − 4 → 9" under the wheel [S-F4.back.1]; at "nine take away four is five" the same from (1, 9) to (0, 5), "9 − 4 → 5" [S-F4.back.2]. At "the secret" a gold point at (0, 5) swells, with outlines round the row label 5, the wheel's 5 and position disc 0 [S-F4.adjacent.secret]. | click, wrapped steps, swell, outline glow | thought after "is nine,"; hold on the gold point |
+| 08_5 | Now trustee two is alone, holding two, which is the secret plus twice the spin. ⟨predict⟩ If the spin could be any of the eleven marks, how many secrets fit that share? | Everything but trustee 2's point goes: the steps, the wheel, the secret's points and the other points and cards; the other discs and frames turn ghost. At "trustee two is alone" rings round (2, 2) and its disc and an outline round its frame [S-F5.holder]; at "holding two" an outline round the row label 2 [S-F5.share]. At "the secret plus twice the spin" the rule "share = secret + 2 × spin" at the upper right and, under it in blue, "trustee 2: 2 = secret + 2 × spin". No strip, no lines and no count yet: the think ring with its "?" at the right. | predict | question hold (think time) |
+| 08_6 | All eleven, one for each spin. Spin zero means the secret is two. Spin one means two take away two, zero. | The "?" goes. At "All eleven" the strip comes in under the rule, all eleven lit, "for: trustee 2" [S-F5.distinct, S-F1.marks]. At "Spin zero means the secret is two" a grey line from (2, 2) to position 0, flat, a gold point at (0, 2), a blue dot over the strip's tick 2 and "spin 0: secret 2" under the strip [S-F5.spin.0]; at "Spin one means" the line from (2, 2) falling 2 a step to (0, 0), its gold point and dot, and "spin 1: secret 2 − 2 × 1 = 0" [S-F5.spin.1, S-F5.spin.1.working]. After the line, the other nine spins' wrapped lines one after another, each meeting position 0 at its own mark, 9, 7, 5, 3, 1, 10, 8, 6 and 4, each with its gold point and its dot [S-F5.secret_by_slope]; then "spins 2 to 10: secrets 9, 7, 5, 3, 1, 10, 8, 6, 4" and an outline round position 0's column, every mark met once [S-F5.per_spin]. | wrapped lines, lagged, strip | thought after "one for each spin." |
+| 08_7 | Doubling is the reason. On this dial, the doubles of zero to ten are zero, two, four, six, eight, ten, then one, three, five, seven and nine. Every mark, once. | The three rows of spins and secrets go. At "Doubling is the reason" an outline round trustee 2's rule. At "the doubles of zero to ten" two rows under the strip's ticks, yellow "spin" 0 to 10 over "2 × spin" 0 to 10 [S-F4.times2.of]. From "are zero, two, four" a yellow arrow from each spin down to its double, one after another as the doubles are said, a ring on each double: 0 to 0 up to 5 to 10; at "then one" the grey note "past 10, take away 11" under the rows, and 6 to 1 up to 10 to 9 [S-F4.times2]. At "Every mark, once" an outline round the doubles' row, every mark ringed once [S-F4.times2.hits_each]. | arrows, lagged, outline glow | connected list; thought before "Every mark, once." |
+| 08_8 | So each spin gives its own secret, and trustee two's share fits every secret equally well, just like B's seven. | At "each spin gives its own secret" the outline round position 0's column glows again. At "fits every secret equally well" a precisely card: "chance of share 2, given secret s: 1/11 for every s (one spin each). Equal likelihoods, not a uniform prior: any belief about s survives" [S-F5.share, S-F1.marks]. At "just like B's seven" the strip shows chapter 03's B, all eleven lit, "for: B, holding 7 (chapter 03)" [S-F2.b_share]. | precisely card, callback | chapter_transition; visual hold until the card is read |
+
+## 09_a_curve_for_three: dealing the five shares
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 09_1 | For any three of five, replace the line with a curve. The dealer spins twice, and say the dial gives seven, then seven again. | The rail with the vault's five cards from chapter 06 lying face down in their frames, so no card answers 09_5's question early. At "For any three of five" the card "the vault's rule: any 3 of 5" at the upper left [S-F1.threshold, S-F1.trustees], outlines glowing round all five frames. At "replace the line" a grey line sketched at the left; at "with a curve" a curve between the same two ends draws over it and the line turns ghost. At "The dealer spins twice" two yellow dials at the right, "1st spin" and "2nd spin" [S-F6.spin_count]: the first turns twice round and stops on 7, and at "say the dial gives seven" a "7" by its mark with a ring; at "then seven again" the second does the same [S-F6.spins]. Small grey under them: "any of the 121 pairs can come up, repeats and zeros included" [S-F6.spin_pairs]. | spin, sketch, outline glow | thought after "a curve." |
+| 09_2 | Every trustee's height is the secret, plus seven times their position, plus seven times their position squared. | The sketch, the dials and the note go. Each term comes in as it is said, one line each at the upper right, a tag beside it: at "the secret," "height = 5", the 5 gold, "the secret"; at "plus seven times their position," "+ 7 × position", the 7 yellow, "1st spin"; at "plus seven times their position squared" "+ 7 × position × position", the 7 yellow, "2nd spin" [S-F6.spins]. Then two rows under the formula: "position" with 1 to 5 in blue, and "squared: position × position" with 1, 4, 9, 16 and 25. No name for the rule yet. | formula, table | clause after "the secret," |
+| 09_3 | Trustee one gets five plus seven plus seven, nineteen. Trustee two gets five plus fourteen plus twenty-eight, forty-seven. | At "Trustee one gets" "trustee 1: 5 + 7 × 1 + 7 × 1 × 1" under the squares, rings round trustee 1's disc and an outline round its frame; at "five plus seven plus seven" "= 5 + 7 + 7" under it; at "nineteen" "= 19" in blue after it, an outline glowing round it [S-F6.terms.1]. The same for trustee 2: "trustee 2: 5 + 7 × 2 + 7 × 2 × 2", "= 5 + 14 + 28", "= 47" [S-F6.terms.2]. The cards stay face down. | derivation, outline glow | thought after "nineteen." |
+| 09_4 | Three, four and five get eighty-nine, a hundred and forty-five, and two hundred and fifteen. On paper, these points lie on one smooth curve, a parabola, and any three of them pin it down. | The rule card goes. A paper plot at the left, "on paper: ordinary numbers", positions 0 to 5 and heights 0 to 220 labelled every 50, with the gold point (0, 5) and blue points labelled 19 and 47, as outlines glow round the "= 19" and "= 47". At "Three, four and five get" rings round discs 3, 4 and 5 [S-F6.rest]; from "eighty-nine" their points and labels 89, 145 and 215 come in one after another as they are said [S-F6.rest.heights, S-F6.heights]. At "one smooth curve" the parabola 5 + 7 × position + 7 × position × position is drawn through all six points [S-F6.paper.curves]. At "any three of them pin it down" teal rings round trustees 1, 3 and 5's points as a teal copy of the curve redraws through them, then the same for 2, 3 and 4 [S-F1.threshold]. The smooth curve lives only on this paper plot. | plot, curve, lagged | connected list; thought after "two hundred and fifteen." |
+| 09_5 | Now wrap them. ⟨predict⟩ Where does trustee two's forty-seven land on the dial? | The formula, squares and working go; the plot stays at the left. At "Now wrap them" the dial board at the right, positions 0 to 5 numbered under it, rows 0 to 10, "on the dial: points only, no curve", empty. At "trustee two's forty-seven" outlines round the plot's 47 and trustee 2's face-down card [S-F6.asked]. Nothing wraps yet: the think ring with its "?" under the board. | predict | question hold (think time) |
+| 09_6 | Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault. | The "?" goes. A small dial between the plot and the board; a blue coil winds 47 round it, four loops and on, as the needle follows and lands on 3 at "so it lands on three", a ring there and "47 = 4 × 11 + 3" under the dial [S-F6.wrap.2]; trustee 2's point (2, 3) comes onto the board and swells. At "All five shares" the other heights' rows: "19 = 1 × 11 + 8", "89 = 8 × 11 + 1", "145 = 13 × 11 + 2", "215 = 19 × 11 + 6" [S-F6.turns]. At each share as it is said, that trustee's card turns over (its back lifts away, then its number drops in), its point comes onto the board and a ring glows: 8, 3, 1, 2, 6 [S-F6.shares]. At "the very shares from the vault" the caption "the vault's shares: 8, 3, 1, 2, 6" under the board, outlines glowing round all five frames. | coil, card turn, callback | thought after "lands on three."; hold on the matched cards |
+| 09_7 | A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three. | The plot, the small dial, the rows and the caption go. At "the secret plus spins times powers" "secret + spin × position + spin × position × position + …" across the top; at "is called a polynomial" "a polynomial:" before it, with an outline. At "One spin gives a line" a second board at the left with chapter 08's points, its wrapped line and "1 spin: a line, any 2 of 5" under it [S-F4.spin_count, S-F4.needed]; at "for any two" rings round trustees 2 and 3's points. At "Two spins give a curve" "2 spins: a curve, any 3 of 5" under the right board as the secret's gold point joins its five shares [S-F6.spin_count]; at "for any three" teal rings round trustees 1, 2 and 3's points and outlines round their cards [S-F1.threshold]. | card, boards, outline glow | chapter_transition |
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -252,26 +252,26 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 08_1 | Nine stays nine. Thirteen goes once round the dial and lands on two. Seventeen lands on six, twenty-one on ten, and twenty-five on three. | The dial board (positions 0 to 5, marks 0 to 10), rail below. Each height drops onto the board: 9 → 9, 13 → 2, 17 → 6, 21 → 10, 25 → 3 [S-F4.heights, S-F4.shares], each with a small "− 11" or "− 22"; the rail's cards update. | morph, lagged | connected list |
-| 08_2 | On the dial the points look scattered, but the rule is the same. Start at the secret and turn on four for each step. | Arrows from (0, 5) step +4 to each point, wrapping off the top and back in at the bottom. | arrows | thought after "the rule is the same." |
-| 08_3 | Take trustees two and three, holding two and six. They are one step apart and differ by four, so each step adds four. | Points (2, 2) and (3, 6) [S-F4.adjacent.shares] highlighted; a bracket "6 − 2 = 4" [S-F4.adjacent.step]. | highlight | thought after "two and six." |
-| 08_4 | Step back twice from trustee two. Two take away four is nine, and nine take away four is five, the secret. | Arrows back from (2, 2) to (1, 9) and to (0, 5) [S-F4.adjacent.secret], which flashes gold. | arrows, flash | thought after "is nine,"; hold on the gold point |
-| 08_5 | Now trustee two is alone, holding two, which is the secret plus twice the spin. ⟨predict⟩ If the spin could be any of the eleven marks, how many secrets fit that share? | Only (2, 2) [S-F5.share] remains; "share = secret + 2 × spin"; the strip's caption reads "for: trustee 2". | predict | question hold (think time) |
-| 08_6 | All eleven, one for each spin. Spin zero means the secret is two. Spin one means two take away two, zero. | Faint wrapped lines through (2, 2), one per spin, each meeting position 0 at its own mark: spin 0 → 2, spin 1 → 0, then lagged 9, 7, 5, 3, 1, 10, 8, 6, 4 [S-F5.secret_by_slope]; the strip fills, each tick once [S-F5.distinct]. | lagged, live count | thought after "one for each spin." |
-| 08_7 | Doubling is the reason. On this dial, the doubles of zero to ten are zero, two, four, six, eight, ten, then one, three, five, seven and nine. Every mark, once. | Times row "× 2": cells 0 … 10 → 0, 2, 4, 6, 8, 10, 1, 3, 5, 7, 9 [S-F4.times2], each result cell lighting once. | table | connected list; thought before "Every mark, once." |
-| 08_8 | So each spin gives its own secret, and trustee two's share fits every secret equally well, just like B's seven. | precisely: "for every secret s, exactly one of the 11 spins gives trustee 2 the share 2: P(share = 2 \| s) = 1/11". B's 7 card from chapter 03 appears small beside it. | precisely card, callback | chapter_transition |
+| 08_1 | Nine stays nine. Thirteen goes once round the dial and lands on two. Seventeen lands on six, twenty-one on ten, and twenty-five on three. | Chapter 07's plot as it ended: the line from (0, 5) to (5, 25), the gold outline round heights 0 to 10, the blue points labelled 9, 13, 17, 21 and 25 and the same numbers in the rail's cards [S-F4.heights]; chapter 07's small dial grows into a wheel right of the plot, and the line turns ghost. At each height as it is said, the needle goes back to 0 the short way and a blue coil winds that height round the wheel, one loop for every whole turn and each loop a little further out, while the needle follows and lands, a ring glowing at its mark: 9 stays 9 with no loop, 13 goes once round and lands on 2 as "lands on two" is said, then 17 on 6, 21 on 10 and 25 on 3, two loops [S-F4.wrap.1, S-F4.wrap.2, S-F4.wrap.3, S-F4.wrap.4, S-F4.wrap.5]. Each height's working comes in under the wheel, one row each: "9 stays 9", "13 − 11 = 2", "17 − 11 = 6", "21 − 11 = 10", "25 − 11 − 11 = 3". For 9 outlines glow round its label and card and nothing moves; every other point drops down its column onto its mark, its label is replaced, and its card is emptied before the share drops in [S-F4.shares]. | coil, morph, card swap | connected list |
+| 08_2 | On the dial the points look scattered, but the rule is the same. Start at the secret and turn on four for each step. | The plot's axes, line, outline and labels go; the points slide onto the dial board in the same columns over the same position discs: rows 0 to 10, a faint dot at every mark, the row labels up the left, two edges and "on the dial: points only", the gold point at (0, 5). At "but the rule is the same" the same straight line, wrapped: three grey pieces through every point, each leaving at the top edge and coming back in at the bottom at the same position; rings glow round the six points. At "Start at the secret" the gold point swells and the needle turns to 5. At "turn on four for each step" five yellow steps, one position each, every step that runs off the top coming back in at the bottom, the needle turning on 4 with each [S-F4.slope], a "+4" between each pair of position discs and rings at each edge a step crosses. | wrapped line, steps, swell | thought after "the rule is the same." |
+| 08_3 | Take trustees two and three, holding two and six. They are one step apart and differ by four, so each step adds four. | The steps, the wrapped line and the working go; the other points turn ghost. At "Take trustees two and three" rings round (2, 2) and (3, 6), round their position discs and outlines round their card frames [S-F4.adjacent]; at "holding two and six" outlines round the row labels 2 and 6 [S-F4.adjacent.shares]. At "one step apart" a grey run from (2, 2) to (3, 2) [S-F4.adjacent.apart]; at "differ by four" a yellow rise from (3, 2) up to (3, 6) with "6 − 2 = 4" beside it [S-F4.adjacent.step]. At "each step adds four" the wheel's needle goes to 2 and clicks on four marks to 6, a "+4" arc by the rim. | highlight, outline glow, run and rise, click | thought after "two and six." |
+| 08_4 | Step back twice from trustee two. Two take away four is nine, and nine take away four is five, the secret. | The run, rise and arc go and the needle goes back to 2. At "Step back twice" rings glow round position discs 1 and then 0 [S-F4.back.steps, S-F4.back.from]. At "Two take away four is nine" the needle clicks back four marks to 9 with a "−4" arc, and a yellow step from (2, 2) to (1, 9) runs off the bottom edge and back in at the top, rings at both edges, "2 − 4 → 9" under the wheel [S-F4.back.1]; at "nine take away four is five" the same from (1, 9) to (0, 5), "9 − 4 → 5" [S-F4.back.2]. At "the secret" a gold point at (0, 5) swells, with outlines round the row label 5, the wheel's 5 and position disc 0 [S-F4.adjacent.secret]. | click, wrapped steps, swell, outline glow | thought after "is nine,"; hold on the gold point |
+| 08_5 | Now trustee two is alone, holding two, which is the secret plus twice the spin. ⟨predict⟩ If the spin could be any of the eleven marks, how many secrets fit that share? | Everything but trustee 2's point goes: the steps, the wheel, the secret's points and the other points and cards; the other discs and frames turn ghost. At "trustee two is alone" rings round (2, 2) and its disc and an outline round its frame [S-F5.holder]; at "holding two" an outline round the row label 2 [S-F5.share]. At "the secret plus twice the spin" the rule "share = secret + 2 × spin" at the upper right and, under it in blue, "trustee 2: 2 = secret + 2 × spin". No strip, no lines and no count yet: the think ring with its "?" at the right. | predict | question hold (think time) |
+| 08_6 | All eleven, one for each spin. Spin zero means the secret is two. Spin one means two take away two, zero. | The "?" goes. At "All eleven" the strip comes in under the rule, all eleven lit, "for: trustee 2" [S-F5.distinct, S-F1.marks]. At "Spin zero means the secret is two" a grey line from (2, 2) to position 0, flat, a gold point at (0, 2), a blue dot over the strip's tick 2 and "spin 0: secret 2" under the strip [S-F5.spin.0]; at "Spin one means" the line from (2, 2) falling 2 a step to (0, 0), its gold point and dot, and "spin 1: secret 2 − 2 × 1 = 0" [S-F5.spin.1, S-F5.spin.1.working]. After the line, the other nine spins' wrapped lines one after another, each meeting position 0 at its own mark, 9, 7, 5, 3, 1, 10, 8, 6 and 4, each with its gold point and its dot [S-F5.secret_by_slope]; then "spins 2 to 10: secrets 9, 7, 5, 3, 1, 10, 8, 6, 4" and an outline round position 0's column, every mark met once [S-F5.per_spin]. | wrapped lines, lagged, strip | thought after "one for each spin." |
+| 08_7 | Doubling is the reason. On this dial, the doubles of zero to ten are zero, two, four, six, eight, ten, then one, three, five, seven and nine. Every mark, once. | The three rows of spins and secrets go. At "Doubling is the reason" an outline round trustee 2's rule. At "the doubles of zero to ten" two rows under the strip's ticks, yellow "spin" 0 to 10 over "2 × spin" 0 to 10 [S-F4.times2.of]. From "are zero, two, four" a yellow arrow from each spin down to its double, one after another as the doubles are said, a ring on each double: 0 to 0 up to 5 to 10; at "then one" the grey note "past 10, take away 11" under the rows, and 6 to 1 up to 10 to 9 [S-F4.times2]. At "Every mark, once" an outline round the doubles' row, every mark ringed once [S-F4.times2.hits_each]. | arrows, lagged, outline glow | connected list; thought before "Every mark, once." |
+| 08_8 | So each spin gives its own secret, and trustee two's share fits every secret equally well, just like B's seven. | At "each spin gives its own secret" the outline round position 0's column glows again. At "fits every secret equally well" a precisely card: "chance of share 2, given secret s: 1/11 for every s (one spin each). Equal likelihoods, not a uniform prior: any belief about s survives" [S-F5.share, S-F1.marks]. At "just like B's seven" the strip shows chapter 03's B, all eleven lit, "for: B, holding 7 (chapter 03)" [S-F2.b_share]. | precisely card, callback | chapter_transition; visual hold until the card is read |
 
 ## 09_a_curve_for_three: dealing the five shares
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 09_1 | For any three of five, replace the line with a curve. The dealer spins twice, and say the dial gives seven, then seven again. | The yellow spare dial spins twice: 7, then 7 [S-F6.spins]. Small grey "any of the 121 pairs can come up, repeats and zeros included" [S-F6.spin_pairs]. | spin | thought after "a curve." |
-| 09_2 | Every trustee's height is the secret, plus seven times their position, plus seven times their position squared. | TeX: height = 5 + 7·x + 7·x² (x = position); the 5 gold, both 7s yellow. | formula | clause after "the secret," |
-| 09_3 | Trustee one gets five plus seven plus seven, nineteen. Trustee two gets five plus fourteen plus twenty-eight, forty-seven. | TeX rows: 5 + 7 + 7 = 19; 5 + 14 + 28 = 47 [S-F6.heights]; the values drop into trustees 1 and 2's cards. | derivation | thought after "nineteen." |
-| 09_4 | Three, four and five get eighty-nine, a hundred and forty-five, and two hundred and fifteen. On paper, these points lie on one smooth curve, a parabola, and any three of them pin it down. | Paper plot (heights 0 to 220): points 19, 47, 89, 145, 215 at positions 1 to 5 and the gold 5 at 0, on the drawn parabola [S-F6.heights]. | plot, curve | connected list; thought after "two hundred and fifteen." |
-| 09_5 | Now wrap them. ⟨predict⟩ Where does trustee two's forty-seven land on the dial? | Trustee 2's 47 highlighted; the dial board below. | predict | question hold (think time) |
-| 09_6 | Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault. | "47 = 4 × 11 + 3"; heights drop onto the board: 19 → 8, 47 → 3, 89 → 1, 145 → 2, 215 → 6 [S-F6.shares]; the rail's cards from chapter 04 light and match one by one. | morph, callback | thought after "lands on three."; hold on the matched cards |
-| 09_7 | A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three. | Card: "polynomial: s + a·x + b·x² + …"; two small boards side by side: the line (1 spin, any 2) and the curve (2 spins, any 3). | card | chapter_transition |
+| 09_1 | For any three of five, replace the line with a curve. The dealer spins twice, and say the dial gives seven, then seven again. | The rail with the vault's five cards from chapter 06 lying face down in their frames, so no card answers 09_5's question early. At "For any three of five" the card "the vault's rule: any 3 of 5" at the upper left [S-F1.threshold, S-F1.trustees], outlines glowing round all five frames. At "replace the line" a grey line sketched at the left; at "with a curve" a curve between the same two ends draws over it and the line turns ghost. At "The dealer spins twice" two yellow dials at the right, "1st spin" and "2nd spin" [S-F6.spin_count]: the first turns twice round and stops on 7, and at "say the dial gives seven" a "7" by its mark with a ring; at "then seven again" the second does the same [S-F6.spins]. Small grey under them: "any of the 121 pairs can come up, repeats and zeros included" [S-F6.spin_pairs]. | spin, sketch, outline glow | thought after "a curve." |
+| 09_2 | Every trustee's height is the secret, plus seven times their position, plus seven times their position squared. | The sketch, the dials and the note go. Each term comes in as it is said, one line each at the upper right, a tag beside it: at "the secret," "height = 5", the 5 gold, "the secret"; at "plus seven times their position," "+ 7 × position", the 7 yellow, "1st spin"; at "plus seven times their position squared" "+ 7 × position × position", the 7 yellow, "2nd spin" [S-F6.spins]. Then two rows under the formula: "position" with 1 to 5 in blue, and "squared: position × position" with 1, 4, 9, 16 and 25. No name for the rule yet. | formula, table | clause after "the secret," |
+| 09_3 | Trustee one gets five plus seven plus seven, nineteen. Trustee two gets five plus fourteen plus twenty-eight, forty-seven. | At "Trustee one gets" "trustee 1: 5 + 7 × 1 + 7 × 1 × 1" under the squares, rings round trustee 1's disc and an outline round its frame; at "five plus seven plus seven" "= 5 + 7 + 7" under it; at "nineteen" "= 19" in blue after it, an outline glowing round it [S-F6.terms.1]. The same for trustee 2: "trustee 2: 5 + 7 × 2 + 7 × 2 × 2", "= 5 + 14 + 28", "= 47" [S-F6.terms.2]. The cards stay face down. | derivation, outline glow | thought after "nineteen." |
+| 09_4 | Three, four and five get eighty-nine, a hundred and forty-five, and two hundred and fifteen. On paper, these points lie on one smooth curve, a parabola, and any three of them pin it down. | The rule card goes. A paper plot at the left, "on paper: ordinary numbers", positions 0 to 5 and heights 0 to 220 labelled every 50, with the gold point (0, 5) and blue points labelled 19 and 47, as outlines glow round the "= 19" and "= 47". At "Three, four and five get" rings round discs 3, 4 and 5 [S-F6.rest]; from "eighty-nine" their points and labels 89, 145 and 215 come in one after another as they are said [S-F6.rest.heights, S-F6.heights]. At "one smooth curve" the parabola 5 + 7 × position + 7 × position × position is drawn through all six points [S-F6.paper.curves]. At "any three of them pin it down" teal rings round trustees 1, 3 and 5's points as a teal copy of the curve redraws through them, then the same for 2, 3 and 4 [S-F1.threshold]. The smooth curve lives only on this paper plot. | plot, curve, lagged | connected list; thought after "two hundred and fifteen." |
+| 09_5 | Now wrap them. ⟨predict⟩ Where does trustee two's forty-seven land on the dial? | The formula, squares and working go; the plot stays at the left. At "Now wrap them" the dial board at the right, positions 0 to 5 numbered under it, rows 0 to 10, "on the dial: points only, no curve", empty. At "trustee two's forty-seven" outlines round the plot's 47 and trustee 2's face-down card [S-F6.asked]. Nothing wraps yet: the think ring with its "?" under the board. | predict | question hold (think time) |
+| 09_6 | Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault. | The "?" goes. A small dial between the plot and the board; a blue coil winds 47 round it, four loops and on, as the needle follows and lands on 3 at "so it lands on three", a ring there and "47 = 4 × 11 + 3" under the dial [S-F6.wrap.2]; trustee 2's point (2, 3) comes onto the board and swells. At "All five shares" the other heights' rows: "19 = 1 × 11 + 8", "89 = 8 × 11 + 1", "145 = 13 × 11 + 2", "215 = 19 × 11 + 6" [S-F6.turns]. At each share as it is said, that trustee's card turns over (its back lifts away, then its number drops in), its point comes onto the board and a ring glows: 8, 3, 1, 2, 6 [S-F6.shares]. At "the very shares from the vault" the caption "the vault's shares: 8, 3, 1, 2, 6" under the board, outlines glowing round all five frames. | coil, card turn, callback | thought after "lands on three."; hold on the matched cards |
+| 09_7 | A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three. | The plot, the small dial, the rows and the caption go. At "the secret plus spins times powers" "secret + spin × position + spin × position × position + …" across the top; at "is called a polynomial" "a polynomial:" before it, with an outline. At "One spin gives a line" a second board at the left with chapter 08's points, its wrapped line and "1 spin: a line, any 2 of 5" under it [S-F4.spin_count, S-F4.needed]; at "for any two" rings round trustees 2 and 3's points. At "Two spins give a curve" "2 spins: a curve, any 3 of 5" under the right board as the secret's gold point joins its five shares [S-F6.spin_count]; at "for any three" teal rings round trustees 1, 2 and 3's points and outlines round their cards [S-F1.threshold]. | card, boards, outline glow | chapter_transition |
 
 ## 10_two_shares: eleven curves, one for each secret
 
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -44,3 +44,18 @@
 07_5	One point is not enough to redraw it, but on ordinary paper it still gives something away. Trustee five holds twenty-five. Which secrets could go with that?
 07_6	Just three, zero, five or ten. No other secret is twenty-five take away five times a spin.
 07_7	Ordinary numbers keep growing, and their size leaks, just as B's piece did. So wrap the heights around the dial instead.
+08_1	Nine stays nine. Thirteen goes once round the dial and lands on two. Seventeen lands on six, twenty-one on ten, and twenty-five on three.
+08_2	On the dial the points look scattered, but the rule is the same. Start at the secret and turn on four for each step.
+08_3	Take trustees two and three, holding two and six. They are one step apart and differ by four, so each step adds four.
+08_4	Step back twice from trustee two. Two take away four is nine, and nine take away four is five, the secret.
+08_5	Now trustee two is alone, holding two, which is the secret plus twice the spin. If the spin could be any of the eleven marks, how many secrets fit that share?
+08_6	All eleven, one for each spin. Spin zero means the secret is two. Spin one means two take away two, zero.
+08_7	Doubling is the reason. On this dial, the doubles of zero to ten are zero, two, four, six, eight, ten, then one, three, five, seven and nine. Every mark, once.
+08_8	So each spin gives its own secret, and trustee two's share fits every secret equally well, just like B's seven.
+09_1	For any three of five, replace the line with a curve. The dealer spins twice, and say the dial gives seven, then seven again.
+09_2	Every trustee's height is the secret, plus seven times their position, plus seven times their position squared.
+09_3	Trustee one gets five plus seven plus seven, nineteen. Trustee two gets five plus fourteen plus twenty-eight, forty-seven.
+09_4	Three, four and five get eighty-nine, a hundred and forty-five, and two hundred and fifteen. On paper, these points lie on one smooth curve, a parabola, and any three of them pin it down.
+09_5	Now wrap them. Where does trustee two's forty-seven land on the dial?
+09_6	Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault.
+09_7	A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three.
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/plot.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/plot.rs
@@ -189,6 +189,9 @@
 pub struct Marker {
     /// Disc and number together, to move as one.
     pub whole: Mob,
+    /// The disc alone, the shape that changes color: 08_5 greys the other
+    /// trustees' discs while their numbers stay on them.
+    pub disc: Mob,
 }
 
 impl Marker {
@@ -197,6 +200,7 @@
         let n = kit.text(stage, &x.to_string(), MARKER_NUMBER_SIZE, hex(BACKGROUND), at)?;
         Ok(Self {
             whole: bundle(stage, &[d, n])?,
+            disc: d,
         })
     }
 }
```
<!-- shamir-0809 patch end -->

The patch's sha256 is
5b9dda493b79d60af9a4e932bcff4af7e181c4f9f4685b90a2047639eda55197 (612
lines). It is the first version's 592 lines, unchanged, followed by the
20-line `plot.rs` hunk.

It was made against the live tree as root left it after integrating 06/07
(11:49–11:58Z). The files below are unchanged from 11:58Z to this handoff
(rechecked before sending).

The patch's base, and the result of applying it:

| File | live (base) | after 0809 |
|---|---|---|
| `src/chapters/mod.rs` | a587eeab… | a677310e… |
| `src/chapters/plot.rs` | e572fc8a… | 8f479579… |
| `src/main.rs` | a4186f59… | eab7d9e0… |
| `src/facts.rs` | 083281f8… | 74685abe… |
| `src/model.rs` | 16e4bbce… | 16e4bbce… (unchanged) |
| `oracle.py` | 332665c6… | 29af2801… |
| `docs/storyboard.md` | abe7dafe… | b8aaa4be… |
| `docs/storyboard-draft.md` | 3006fc27… | e6898a79… |
| `docs/script-draft.tsv` | 0aee3711… | 0aee3711… (unchanged) |
| `script.tsv` | 1dd6689f… | 7a8f5431… |

Files the patch does not touch:
- root's 06/07 chapters, live: `padlocks.rs` aa56b8cc… and
  `a_line_through_the_secret.rs` 5c379c3b…;
- my new files, as in the table above: `board.rs` 5ad93119…,
  `the_line_on_the_dial.rs` 9d813276… and `a_curve_for_three.rs`
  d0f421dc….

If root has changed any of these since, the hunks may need rebasing; the
content is the list above.

## Chapters 10 and 11, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment in Mail 6921 (12:22Z), after it froze the 08/09 handoff
(Mail 6920) by hash in
`narration_lab/third-wave-20261024/root/tend-1205/shamir-08-09-intake/`.
The ask:
- write new native chapters `10_two_shares` and `11_one_more_share`, keeping
  the vault's original cards and the same trustees 4 and 5;
- build the intuition that every secret has exactly one equally likely spin
  pair, then show why an unwrapped paper-curve picture alone does not prove
  secrecy, and let trustee 1's 8 select the unique secret;
- give viewers time to predict and inspect before the answers;
- on the dial, show the computed discrete points and label any joining
  guides; keep smooth curves for the clearly marked paper comparison;
- keep likelihood independent of an assumed uniform prior;
- independently enumerate all pairs, secret candidates and third-share
  recovery, with wrong-modulus or wrong-card controls;
- write only new unregistered modules and WORK_STATUS / owned previous
  evidence, preserve `board.rs` and the delivered modules exactly, and
  propose shared edits as an integration patch against current source,
  keeping root's fixes and the pending `Marker.disc` restoration;
- include authored semantic pause intentions and explicit prediction and
  reading holds; Python and rustfmt checks only.

**Status.**
- I wrote two new files, `two_shares.rs` and `one_more_share.rs`. I edited
  no registered, delivered or shared file. `board.rs`,
  `the_line_on_the_dial.rs` and `a_curve_for_three.rs` are still
  5ad93119…, 9d813276… and d0f421dc…, as root froze them. Root's 08/09
  intake was not touched.
- Nothing was compiled, rendered, synthesized or listened to.
- Bead mx-video-program-program-87do.46.3 stays in progress; I close
  nothing.

**Reservations.**
- Released at 12:25:34Z, as Mail 6921 asked: 8117
  (`the_line_on_the_dial.rs`), 8118 (`a_curve_for_three.rs`) and 8136
  (`board.rs`).
- Held: 8251 (`two_shares.rs`), 8252 (`one_more_share.rs`), 8254
  (`docs/previous/ch1011-*/**`), 8119 (`docs/previous/ch0809-*/**`) and 7941
  (this file). 8253 (`candidates.rs`) was reserved but never used: no such
  file exists, and I release it with this handoff.

**Prior bytes.** `docs/previous/ch1011-before-10-11-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section, sha256
75ffdfb8bda6293ba534a03889699c0911cb327206e7b78e1efe31e9806f9062, the
bytes root froze.

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `two_shares.rs` | 1151 | 7ffd84444ab550a6ae470e3e476e0b0da54a23914ca6422957f85f3cacdf1f42 |
| `one_more_share.rs` | 1356 | 30f4d0daaff3c4d903892b7a794626dec35f073552749133bf8ca5fa4708b9fd |

- **`two_shares.rs`**: `TwoShares`, key `10_two_shares`. It also holds what
  both chapters draw: `Candidate` and `candidates` (each secret's one curve
  through the pair's shares, from the model's brute force, an error unless
  every secret fits exactly one spin pair), `Drawn` (a curve's points,
  guides and spins' tag), `pair_points`, `Opening` (the rail with the
  vault's cards and the empty board), `card_back`, `disc_ring`,
  `column_outline`, `ticks_outline` and `text_right`. 8 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `every_secret_fits_one_spin_pair_for_every_pair`: all 10 pairs, all 121
    pairs of shares, every secret, with the share written out apart from
    the model's Horner rule. **Control (wrong modulus):** on ten marks the
    vault's pair fits secrets unevenly and `candidates` refuses it;
  - `the_curves_fill_every_column_away_from_the_pair`. **Planted
    negatives:** one share, or one trustee holding two cards, gives no
    curve per secret;
  - `guides_only_join_a_curves_points`, with a control (held positions that
    are not neighbours share no guide);
  - `the_cards_are_the_vaults`, with a planted negative (chapter 17's other
    spins deal the pair other cards);
  - `the_card_is_two_lines` (the precisely card at every wrap 72 to 92);
  - `the_layout_keeps_its_pieces_apart`.
- **`one_more_share.rs`**: `OneMoreShare`, key `11_one_more_share`. Its data
  functions are `paper_pair`, `paper_spins` (the model's Cramer's rule,
  exact fractions), `whole_spin_secrets`, `spin_text` and `paper_tag`
  ("secret 0: spins 9 + 1/4 and 6 + 3/4"), `paper_curve`, `marks_at`,
  `secrets_through`, and `Tally` / `every_pair` (the 11_6 table, counted by
  the scene). 9 tests:
  - `every_phrase_the_scene_waits_for_is_in_its_line`;
  - `the_pictures_are_the_facts`;
  - `a_third_share_leaves_one_secret_for_every_pair`: all 1331 recipes
    dealt by the definition, for every pair and every third trustee; each
    three shares come from one recipe, whose secret `model::rebuild` finds;
    the scene's `Tally` agrees;
  - `a_wrong_card_picks_another_secret` (**control, wrong card**): every
    card but 8 leaves exactly one other secret, each card a different one,
    and the definition agrees;
  - `ten_marks_break_the_count` (**control, wrong modulus**): ways 0 and
    more than 1 appear, and `candidates` refuses the ten-mark pair;
  - `on_paper_the_heights_give_the_secret_away`: exact fractions, a search
    over every recipe, `View::Paper` and the dial's control all agree.
    **Control:** a nudged height (4, 146) gives the secret 10 away instead;
  - `spins_are_written_as_mixed_numbers`;
  - `the_window_shows_the_curves_apart`;
  - `the_layout_keeps_its_pieces_apart`.

**Dependencies.**
- Both files import only `pub` items. A scan of their `use` lines and
  inline paths (and the patched `facts.rs`'s) found all 197 names.
- They need two `model.rs` additions from the patch: `ASKED_SECRETS` and
  `Ratio::mixed` / `Ratio::to_f64`.
- Their phrase tests read `crate::SCRIPT`, so they need the patch's
  `script.tsv`. Their fact tests need the patch's `facts.rs`.
- They use `board.rs` (`Board`, `Grid`, `ring`, `now`, `turns_and_mark`,
  the label sizes) as delivered, and `envelope.rs`, `two_keys.rs` and
  `draw.rs` as registered. No helper changes.

### How the brief is met

**The vault's cards and trustees 4 and 5.** Both chapters open on the rail
with the vault's five cards in their frames: 4 and 5 face up (2 and 6),
their discs purple, and 1, 2 and 3 face down, their discs faint. 10_1's
dealer's curve passes 8, 3 and 1 at positions 1 to 3, the face-down cards
(`the_cards_are_the_vaults`). In 11_4 trustee 1's own card turns over to 8.

**One equally likely spin pair for every secret (chapter 10).**
- 10_1 recalls chapter 05's experiment (the strip for the pair, all 11
  lit) and draws the dealer's curve: the gold (0, 5), blue points at
  positions 1 to 3, the purple pair, and its tag "7 and 7".
- 10_2 asks how many curves of this kind run through the pair and the
  secret 0, with a gold point at (0, 0) and the think ring. Nothing
  answers it.
- 10_3 answers ("Exactly one, with spins one and four") and then checks
  it, out loud and on screen: "trustee 4: 0 + 1 × 4 + 4 × 4 × 4 = 68 → 2"
  and "trustee 5: 0 + 1 × 5 + 4 × 5 × 5 = 105 → 6". Then the secret 1's
  curve, "0 and 9".
- 10_4 brings in the other eight. The gold column holds every mark,
  positions 1 to 3 each hold all eleven marks, and the eleven tags are
  eleven different spin pairs. The board then holds for 2 s.
- 10_5 brings back the spin grid (11 × 11 = 121, "each as likely as any
  other"). Secret by secret, its point is ringed, its tag glows, and its
  one cell lights with the secret written in it: eleven different cells.
- 10_6's precisely card gives the likelihood, not a prior: "chance of
  shares 2 and 6, given secret s: 1/121 for every s (one spin pair each).
  Equal likelihoods, not a uniform prior: any belief survives" (141
  characters, two lines at every wrap the lower third can choose; the
  test checks it). The narration says the same ("just as likely under
  every secret").

**Why the picture alone is not the proof (11_1 to 11_3).**
- 11_1 points at what the argument used (a yellow outline round the spin
  tags) and brings in the paper comparison at the lower right. A plot
  titled "on paper: ordinary numbers" shows heights 0 to 220 and the
  pair's 145 and 215. Beside it, a window titled "near position 0,
  enlarged" covers positions 0 to 1 and heights 0 to 25, its region
  outlined on the plot. A smooth curve grows through every secret on
  both. On the plot they almost coincide: neighbouring curves start
  0.0095 apart, under the 0.015 stroke. In the window they stand apart,
  with a gold dot at each secret. Outlines round the board and the paper
  then show the same picture twice.
- 11_2: on paper only the secret 5 gets whole spins. The secret 0 needs
  "9 + 1/4 and 6 + 3/4" (red outline, "a spin is a whole mark, 0 to 10"),
  and only "secret 5: spins 7 and 7" stays blue. The narration adds "so on
  paper their heights give the secret away", and the strip shows it:
  "for: trustees 4 and 5, on paper", only 5 lit.
- 11_3: on the dial every curve has real spins, one pair each. The strip
  is back to 11 lit, every tag glows, and "each secret: exactly 1 of the
  121 spin pairs" appears. The paper leaves at "That count is the proof",
  and the board is outlined at "the pictures only help us see it".

**Trustee 1's 8 picks the secret (11_4 and 11_5).**
- 11_4: trustee 1's disc turns teal, its face-down card turns over to 8
  (chapter 09's turn), and a teal point lands at (1, 8) with its column
  outlined. Then the question "How many of the eleven curves pass through
  eight at position one?", answered by nothing but the think ring.
- 11_5: "Just one." Secret by secret, each curve's point at position 1 is
  ringed and its mark written under the secret's tick: 5, 10, 4, 9, 3, 8,
  2, 7, 1, 6, 0, all different. Only the curve for 5 passes 8. It turns
  blue; the others and their gold points go faint and their tags leave.
  Then the rail lights 1, 4 and 5 and the strip drops to "for: trustees
  1, 4 and 5", only 5 lit. A 1.5 s hold on the single curve follows.
- 11_6 generalises it with a table the scene counts itself (`every_pair`,
  checked at construct time): "every pair of trustees (10 pairs):", the
  ten pairs, "each holding any of the 121 pairs of shares:", "2 shares:
  every secret fits exactly 1 spin pair" (purple) and "+ any third share:
  exactly 1 secret left" (teal). It is held until it can be read.

**Discrete dial, labelled guides, smooth curves only on paper.**
- The board is titled "on the dial: points only". Every curve on it is
  its computed points (`Recipe::share` at positions 0 to 5). Straight
  guides join each point to the next, and the legend under the board says
  "lines are guides joining one curve's points".
- No guide is drawn between the two held positions twice: the pair's one
  purple guide stands for all eleven (`guides_only_join_a_curves_points`).
- Smooth strokes are drawn only in `one_more_share.rs`'s `Paper`, under
  the two "on paper" / "enlarged" titles and inside the outlined paper
  area.

**Independent enumeration and controls.** There are three separate
counts. None runs the others' code:
- the Rust tests, with the share written as s + a·x + b·x² apart from the
  model's Horner rule (unrun);
- `oracle.py`, which deals all 1331 recipes and groups them by the shares
  they deal, for every pair and every third trustee (run: 0 failed
  cross-checks);
- my scratch `evidence.py`, plain Python (run, below).

The controls:
- **wrong modulus:** ten marks in both chapters' tests, in the oracle's
  self-test and in `evidence.py`. Ten-mark ways over every pair are {0, 2,
  4, 10}. For the vault's pair on ten marks only the secret 5 fits (10
  ways), so that pair would see the secret outright;
- **wrong card:** `a_wrong_card_picks_another_secret`, the oracle's new
  self-test row and `evidence.py`. Every card trustee 1 could hold leaves
  exactly one secret, each card its own;
- **nudged height:** on paper, (4, 146) gives away 10 instead of 5.

### Spoken deltas from the draft

Words change in three rows only. Titles and ids are unchanged, and nothing
outside 10 and 11 changes.
- **10_3** adds the check: "Check it: trustee four's height would be
  sixty-eight, which lands on two, and trustee five's a hundred and five,
  which lands on six." It is the working on screen, and it shows the answer
  is a real dealing.
- **11_2** ends "…Only the secret five gets whole spins, so on paper their
  heights give the secret away." It says the consequence the strip then
  shows.
- **11_3** "and the curves just help us see it" → "and the pictures only
  help us see it". On the dial there are no curves, only points and
  guides, and the sentence covers both pictures.

All 12 rows' picture, feature and hold cells are rewritten to describe the
scenes as written, with their fact keys. `docs/script-draft.tsv` changes in
exactly those three lines. The 04_3 difference already in the live tree
stays as it is.

### Semantic holds

**Narration-side.** These are director overrides proposed in the patch's
`docs/pause-overrides.json`, in the director's own format. Each resolves to
one boundary (checked below).

| Cue | After | Role | Range (s) | Why |
|---|---|---|---|---|
| 10_1 | "Now we can see what it found" | thought | 0.5–0.9 | the recalled strip lands before the curve that explains it |
| 10_3 | "with spins one and four" | thought | 0.5–0.9 | the answer, then a pause before checking it |
| 10_3 | "which lands on six" | thought | 0.4–0.8 | closes the two-line check |
| 10_5 | "are all equally likely" | thought | 0.5–0.9 | fairness lands before the cells light |
| 11_1 | "Notice what that argument used" | thought | 0.5–0.9 | the turn to what the argument used |
| 11_1 | "can't be the reason" | cue_transition | 0.8–1.3 | the claim stands before the numbers test it |
| 11_2 | "two hundred and fifteen" | thought | 0.5–0.9 | the heights, then the spins they force |
| 11_2 | "no spin can be that" | thought | 0.6–1.0 | the impossibility registers |
| 11_3 | "all equally likely" | thought | 0.5–0.9 | the counting claim before "the proof" |
| 11_5 | "Just one" | thought | 0.5–0.9 | the answer, then why |

The defaults stand elsewhere: question holds at the ends of 10_2 and 11_4,
and chapter transitions after 10_6 and 11_6. The overrides' `_comment` now
names 10/11 and the scene holds below. These ranges are proposals; nothing
is measured or heard.

**Scene-side** (in code, protected visual holds):
- 10_2 and 11_4: 4 s think rings (`draw::think`, after the line ends);
- 10_4: 2.0 s on the eleven curves (`HOLD_ON_ELEVEN`);
- 10_6: the card held until read at 15 characters a second from fully in
  (`read_by`, 141 characters, about 9.4 s). By the word-rate estimate
  that is about 4 s past the line's end. Then 0.6 s;
- 11_5: 1.5 s on the single curve (`HOLD_ON_ONE`);
- 11_6: the table held until every line can be read (`read_by` per line,
  from its fade-in). The last line (40 characters, about 2.7 s) appears at
  "any third share leaves exactly one", so the hold runs about 1–2 s past
  the line's end by estimate. Then 0.8 s.

### Timing and layout risks (estimates; nothing is measured)

- **Spoken length.** `tools/storyboard.py` estimates 10 at 78.0 s and 11 at
  84.7 s (words at 2.68 a second, breaths, holds, predictions). Neither is
  flagged. Total: 14.2 min over 11 chapters.
- **Scene length.** Think rings, the holds above and the fades add roughly
  10–12 s to each chapter: about 90 s and 97 s, as estimates. Nothing was
  cut for length.
- **Cue-bound moves can land late** if the narrator is faster than the
  estimate. `Spoken::at` and `wait_for` both use a phrase's start.
  - 10_4's eight arriving curves take 2.0–3.2 s, from the start of "Here
    are all eleven." (4 words and a thought pause, about 1.9–2.6 s) to
    "Each one is a dealing". They may run about half a second into it.
  - 11_5's eleven values take at least 1.6 s, after a 0.4 s title fade,
    from the start of "Each curve passes position one" (5 words, about
    1.9 s) to "at a different mark". They finish about when it begins.
  - 11_1's paper frame (0.5 s) and curves (1.0–2.2 s) have about 3.7 s of
    speech before "the same two points", so they are not at risk.
- **Layout.** Text sizes are estimated as before (0.0055 units a point per
  character wide, 0.0065 tall). The busiest frames are 11_1 to 11_3 (board,
  tags, strip, paper plot and window, upper-right tags) and 11_6 (table
  under the strip).
  - By estimate the paper outline's top edge is about 0.47 under the
    strip's caption (the test asks 0.2), and its right edge is 0.28 inside
    the 0.3 frame margin.
  - The lower-third card is two lines, so its band's top (about y −2.78)
    stays under the rail's discs (bottom −2.46). A three-line card would
    reach −2.42, which is why 10_6's text was shortened to 141 characters.
- **Test time.** `facts()` now counts every pair (13,310 `fitting_spins`
  calls over 121 spin pairs, plus 439,230 third-share checks), and so does
  11's `every_pair` at construct time and in two tests. That is cheap in
  release builds. In debug test builds it may add on the order of a
  second per `facts()` call, by estimate only. If root prefers, the
  `S-F7.all_pairs` block could move into a test.
- **Glyphs.** On screen only "×" (`\u{d7}`) and "→" (`\u{2192}`) are
  non-ASCII, both in the probed DIGITS. The other non-ASCII characters
  ("−", "·", "²", "…") are in doc comments only.

### Checks actually run (repository root, `python3 -I -B`, stderr shown)

The scripts named below are copied, with the patch, into
`docs/previous/ch1011-evidence-20261024/` (gitignored): `check_cues.py`,
`check_golden.py`, `check_overrides.py`, `check_sb.py`, `check_spoken.py`,
`evidence.py`, `layout_mirror.py`, `make_patch.py` and
`integration-10-11.patch`. `integration-10-11.superseded-r1.patch` there
is a first cut, 2ae1b625…, never handed off. It predates the shorter 10_6
card and the two-line test. The lint mirrors (`pubscan.py`,
`const_asserts.py`, `dead_code.py`, `lint_sweep.py` and
`unused_imports.py`) are the 08/09 ones, unchanged, in this session's
scratchpad.

The final runs (13:31–13:40Z) were on the patch below, applied to a fresh
copy of the live topic (with `tools/storyboard.py` and `rustfmt.toml`):
- `git apply` of 5b9dda49 first: `--check` rc 0, apply rc 0;
- then this patch: `--check` rc 0, apply rc 0;
- a recursive diff of that tree against my scratch work tree: identical
  except my two new gitignored `docs/previous/ch1011-*` folders, which
  exist only in the live copy;
- this patch alone on the live tree (check only): refused at `mod.rs:1`
  and `main.rs:282`, so the order matters.

The checks:
- **Formatting** (`rustfmt --check --edition 2024 --config-path
  rustfmt.toml`, a parse check only). rc 0 on both new files. rc 0 on the
  applied tree's `main.rs`, which parses every module, including the
  patched `facts.rs` and `main.rs`. The four layout hunks rustfmt wanted
  in my scratch `facts.rs` and `main.rs` were made by hand before the
  patch was cut.
- **The oracle on the applied tree.** `oracle.py --self-test` prints 262
  facts, rc 0. stderr: `# self-test: ten-mark dial pairs that leak: 9 of 10`
  and `# 262 facts, 0 failed cross-checks`. The output's sha256 is
  b205a0e8da5bbfa0a286bed541be17bfd05e142e09705a0b0576caf743e2bced. It
  adds exactly the 28 new rows to the base's 234 and changes none.
- **The golden table** (`check_golden.py`). 262 golden rows (262 distinct)
  against the oracle's 262, with `f.len()` asserted 262. Diff: none. rc 0.
- **Spoken numbers** (`check_spoken.py`). 195 `SPOKEN` rows, 9
  `NOT_A_COUNT` and 40 number words. All 72 lines: 361 number words, 0
  failures. The planted control was caught. rc 0.
- **Cues** (`check_cues.py`).
  - 10 says 10_1–10_6 in order: 30 cue uses, 27 distinct, 27 waits.
  - 11 says 11_1–11_6 in order: 30 uses, 28 distinct, 28 waits.
  - Every phrase is once in its line, in spoken order, and equals the
    tests' lists. 0 failures, rc 0, against both `script.tsv` and
    `docs/script-draft.tsv`.
  - A planted wrong phrase ("Just two") gives 3 failures, rc 1.
  - 10_3's loop now walks `[cue, landing]` pairs, so the checker sees all
    four of its phrases. An indexed array literal hid two of them from it
    before.
- **Storyboards** (the scratch copy of `tools/storyboard.py`, writing only
  there).
  - `--check shamir_secret_sharing`: rc 0, with the estimates above.
  - `check_sb.py`: 0 failures, rc 0:
    - production chapters 01–11 equal the draft's, row for row;
    - in the draft only 10/11 changed, with titles and ids kept and words
      changed only in 10_3, 11_2 and 11_3;
    - `script-draft.tsv` is what the draft generates apart from the
      pre-existing 04_3, and its edits are exactly those three lines;
    - `script.tsv` is the base's 60 lines plus exactly the 12 new ones: 72.
- **Director overrides** (`check_overrides.py`, the director's `tokens` and
  override rules copied). All 18 overrides (8 existing, 10 new) resolve to
  one boundary each, with no duplicates and valid ranges. A planted
  ambiguous phrase is caught. rc 0.
- **Independent arithmetic** (`evidence.py`). Plain Python enumeration and
  exact fractions, compared with the literals the unrun tests expect,
  read out of the test sources. 165 checks, 0 failures, 4 of 4 planted
  controls caught, rc 0:
  - the vault's pair (4, 2), (5, 6) and cards 8, 3, 1, 2, 6;
  - one spin pair per secret, 1:4, 0:9, 10:3, … 2:10, all different, and
    the dealer's 7:7;
  - marks per position 11, 11, 11, 11, 1, 1, positions 0–3 each a full
    set;
  - the two check rows, with 68 → 2 and 105 → 6;
  - guides 4 per curve, ending on (4, 2), with the control at 5;
  - the chance 1/121 for every secret, by counting recipes, and the card's
    opening words;
  - every pair × every pair of shares × every secret: one spin pair each;
  - ten-mark counts for the vault's pair, [0, 0, 0, 0, 0, 10, 0, 0, 0, 0];
  - paper (4, 145), (5, 215): every secret's fractions meet both heights,
    whole spins only for 5 (also by searching every recipe), the nudged
    (4, 146) gives 10, the tags and mixed numbers;
  - position 1's marks 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0, all different;
    8 leaves 5; every card picks its own secret;
  - all 1331 recipes: two shares leave {11} secrets, three leave {1}; the
    table's four texts;
  - ten-mark ways over every pair {0, 2, 4, 10}.
  - **Controls:** a reversed marks row, a wrong check row, ten marks at
    the odd positions 1, 3 and 5, and the nudged height are each caught.
    Ten marks at positions 1, 4 and 5 are *not* a failing control: mod 2
    the even position 4 pins the secret's parity, so those three still
    determine it. I first planted that one, saw it pass, and replaced it.
- **The card** (one-off command). The text built from `card_text`'s
  format string in the source wraps to 2 lines at 92, 88, 84, 80, 76 and
  72 (the kit's greedy `wrap`, mirrored). The longer first version was 3
  lines at 80 and below.
- **Layout mirror** (`layout_mirror.py`). Constants, text constants and
  the `Grid` are read out of the Rust sources. Both layout tests and the
  window test are transcribed.
  - 106 assertions, 0 failures, rc 0.
  - A planted copy with an overlapping table row fails exactly "11 table
    lines apart", rc 1.
- **Imports** (`pubscan.py`). 197 names, 0 missing, rc 0.
- **Lint mirrors** (textual; not Clippy), calibrated as for 08/09:
  - **constant-only `assert!`s outside `const { }`:** 0;
  - **dead code:** only each scene's `construct`, a trait impl, as in
    every registered chapter. A planted unused `pub fn` is caught;
  - **unused imports:** only `main.rs`'s `Explainer`, which is used as a
    const's type. The tool reports the same on the root-clean base
    `main.rs`;
  - **default-on patterns:** one real lead, fixed. A doc line in 11's
    tests began "5. The dial's control…", which markdown reads as a list
    item (`doc_lazy_continuation`); it is reworded. A remaining
    `redundant_closure` pattern, `|p| pair_text(p)`, needs a `&Vec<u32>`
    → `&[u32]` coercion, the same form root's Clippy accepted in a clean
    file. The `model.rs` hits (lines 565, 680, 741, 742) are in root's
    existing code, not in the patch's two hunks.
  - I also removed a no-op `ys.dedup()` from 11's layout test.

### Not verified

- **Nothing compiled.** No Cargo build, test or Clippy. All 17 new tests
  are unrun (10: 8, 11: 9), as are the changed golden, `f.len()`, `SPOKEN`,
  storyboard and registry tests.
  - Types and borrows were checked by reading only, among them:
    - `ASKED_SECRETS` shadowing the earlier `asked` in `facts()` (its
      later uses are all before the shadow);
    - the `(0..).zip(&fits)` closure's `*s`;
    - `[cue, lands]` destructuring in 10_3;
    - `Ratio`'s `PartialEq` in `spin_text`.
  - The lint mirrors are patterns, not Clippy.
- **Nothing rendered or watched.** I relied on engine behaviours that I
  read in the pinned source and the topic's helpers but did not run:
  - `show_creation` on 61-sample paper strokes;
  - recoloring strokes and discs (`recolorings`) on the board and paper;
  - lagged `AnimationGroup`s of point-and-guide steps;
  - 121 `panel` cells in 11 bundles;
  - the `Succession` card turn;
  - `Strip::show_with`'s count tween;
  - the lower third's narrowing loop.
- **Layout is estimated**, with the text sizes above. Real glyph widths
  decide where the lower third wraps.
- **Glyphs are unprobed.** The probe renders, so it is root's to run.
- **No listening.** Nothing here claims accent consistency, and every
  duration above is an estimate.

### For root's integration (a proposal; I edited none of these files)

**Order.**
1. Root's frozen 08/09 patch, `integration-08-09.patch` (sha256 5b9dda49…,
   612 lines). It carries the `Marker.disc` restoration and keeps
   `HEIGHT_TITLE_GAP`, `parts()`, `FanLine`, the `const` assertions and
   the sequential quote reading.
2. Then this patch: sha256
   f865d6bff627bda6fc4178fb8aebc3140d4c4726c367edeef7b8a149b012efa5, 631
   lines, 10 files.

It touches no `plot.rs`, `draw.rs`, `envelope.rs`, `two_keys.rs`,
`board.rs`, `glyphs.rs` or registered or delivered chapter. `two_shares.rs`
and `one_more_share.rs` are already in place, so the patch needs no new
files. The two chapters register with the patch's `mod.rs`, and must not be
registered without its `script.tsv` (`check_video` compares the registry,
the storyboard and the script).

| File | live | after 5b9dda49 (base) | after this patch |
|---|---|---|---|
| `src/chapters/mod.rs` | a587eeab | a677310e | d1e3e70c |
| `src/model.rs` | 16e4bbce | 16e4bbce | 49c4b87a |
| `src/main.rs` | a4186f59 | eab7d9e0 | 2a2a222a |
| `src/facts.rs` | 083281f8 | 74685abe | c2837471 |
| `oracle.py` | 332665c6 | 29af2801 | 005d2491 |
| `docs/storyboard.md` | abe7dafe | b8aaa4be | 3192f4b0 |
| `docs/storyboard-draft.md` | 3006fc27 | e6898a79 | b2b63c67 |
| `docs/script-draft.tsv` | 0aee3711 | 0aee3711 | a8a917b9 |
| `script.tsv` | 1dd6689f | 7a8f5431 | be6a1554 |
| `docs/pause-overrides.json` | d23a2573 | d23a2573 | 1772d081 |

The hunks, file by file:
- **`src/chapters/mod.rs`.**
  - `pub mod one_more_share;` and `pub mod two_shares;`.
  - Chapters `10_two_shares` → `two_shares::TwoShares` and
    `11_one_more_share` → `one_more_share::OneMoreShare`, each at
    intensity 0.5.
  - The doc reads "chapters 01 to 11 so far. The other twelve …".
- **`src/model.rs`.**
  - `pub const ASKED_SECRETS: [u32; 2] = [0, 1]` (the secrets asked about
    after the dealer's).
  - `Ratio::mixed` (whole part and remainder, by `div_euclid` /
    `rem_euclid`) and `Ratio::to_f64` (for drawing).
- **`src/facts.rs` and `oracle.py`.** 28 new facts. The golden table grows
  from 234 to 262 rows, `f.len()` from 234 to 262, and the module doc says
  "Chapters 01 to 11" and where S-F7.paper/join/all_pairs serve. The new
  facts:

  | Key | Value | Rust (model) | Python (oracle) |
  |---|---|---|---|
  | `S-F7.pair.count` | 2 | the pair's points | `len(PAIR)` |
  | `S-F7.asked.first`, `.second` | 0 · 1 | `ASKED_SECRETS` | `ASKED` |
  | `S-F7.spins.0`, `.1` | 1,4 · 0,9 | `one_way_each`'s spins | `per_secret` by search |
  | `S-F7.check.4`, `.5` | 4,68,2 · 5,105,6 | `Recipe::height`, an error unless `Dial::wrap` gives the share | heights written out, `wind` checked against the share |
  | `S-F7.board.0`…`.10` | 0,5,7,6,2,6 … 10,0,10,7,2,6 | `Recipe::share` at positions 0–5 | `share` per secret's spins |
  | `S-F7.board.columns` | 11,11,11,11,1,1 | distinct marks per position | the same, by set |
  | `S-F7.paper.secret5_spins` | 7,7 | `paper_curve_spins` | the solved fractions |
  | `S-F7.join.point` | 1,8 | `JOINER`, its share | the same |
  | `S-F7.join.count` | 1 | secrets left | the same |
  | `S-F7.join.drop` | 11,1 | the pair's `View::Shares`, then the secrets the joiner's share leaves | fitting secrets, then joined |
  | `S-F7.all_pairs.pairs`, `.share_pairs` | 10 · 121 | `combinations`, `tuples` | `itertools` |
  | `S-F7.all_pairs.ways_each` | 1 | `fitting_spins` for every pair, shares and secret | all 1331 recipes grouped by the two shares |
  | `S-F7.all_pairs.two_leave` | 2,11 | secrets with a fit | distinct secrets per group |
  | `S-F7.all_pairs.third_leaves` | 3,1 | secrets a third share keeps | recipes grouped by three shares |

  - **Honest note.** Several are bookkeeping for the spoken words, not
    independent arithmetic: `pair.count`, `asked.*`, `join.point`,
    `all_pairs.pairs` and `.share_pairs`.
  - **New oracle self-test row (wrong card):** every card the joiner could
    hold leaves exactly one secret, 5 only for the vault's 8, and the
    eleven cards pick eleven different secrets. The docstring names the
    p**3 table.
- **`src/main.rs`.**
  - 37 `SPOKEN` rows for 10_1–11_6, for example:
    - ("10_3", "trustee four's height would be sixty-eight, which lands on
      two", "S-F7.check.4", "4,68,2");
    - ("11_2", "nine and a quarter and six and three quarters",
      "S-F7.paper.secret0_spins", "37/4,27/4");
    - ("11_5", "from eleven secrets to one", "S-F7.join.drop", "11,1");
    - ("11_6", "any third share leaves exactly one",
      "S-F7.all_pairs.third_leaves", "3,1").
  - 2 `NOT_A_COUNT` rows: ("10_4", "Each one is a dealing") and ("11_3",
    "every one of the").
- **`docs/storyboard-draft.md`.** The 10/11 rows as described above. No
  other chapter changes.
- **`docs/storyboard.md`.**
  - "chapters 01 to 11" in its intro;
  - a "Devices in chapters 10 and 11" note (the pair's board and guides
    legend, spins' tags, the spin grid, the paper comparison and window,
    face-down cards, the table, emphasis by outlines);
  - the draft's 10 and 11 sections, appended verbatim.
- **`docs/script-draft.tsv`.** Only the 10_3, 11_2 and 11_3 lines. Its
  header and 04_3 stay.
- **`script.tsv`.** Regenerated by `tools/storyboard.py
  shamir_secret_sharing`: 72 lines.
- **`docs/pause-overrides.json`.** The 10 overrides above and the updated
  `_comment`. The 8 existing overrides are unchanged.

To extract the patch and apply it, after 5b9dda49:

~~~sh
awk '/^<!-- shamir-1011 patch end -->$/{p=0} p; /^<!-- shamir-1011 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-1011.patch
git apply --check shamir-1011.patch && git apply shamir-1011.patch
~~~

The same bytes are in
`docs/previous/ch1011-evidence-20261024/integration-10-11.patch`.

<!-- shamir-1011 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,8 +1,8 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 09 so far. The other
-//! fourteen are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 11 so far. The other
+//! twelve are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
 pub mod a_curve_for_three;
@@ -13,6 +13,7 @@
 pub mod draw;
 pub mod envelope;
 pub mod glyphs;
+pub mod one_more_share;
 pub mod padlocks;
 pub mod plot;
 pub mod the_dial;
@@ -20,6 +21,7 @@
 pub mod the_line_on_the_dial;
 pub mod the_vault;
 pub mod two_keys;
+pub mod two_shares;
 
 use explainer_kit::Chapter;
 
@@ -70,5 +72,15 @@
             make: |kit| Box::new(a_curve_for_three::ACurveForThree { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "10_two_shares",
+            make: |kit| Box::new(two_shares::TwoShares { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "11_one_more_share",
+            make: |kit| Box::new(one_more_share::OneMoreShare { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/model.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/model.rs
@@ -30,6 +30,9 @@
 pub const PAIR: [u32; 2] = [4, 5];
 pub const JOINER: u32 = 1;
 pub const SECOND_TRIO: [u32; 3] = [2, 4, 5];
+/// Chapters 10 and 11: the secrets asked about after the dealer's, each
+/// with its own curve through the pair's two shares.
+pub const ASKED_SECRETS: [u32; 2] = [0, 1];
 /// Chapter 17: another dealing of the same secret, whose thirds do not
 /// cancel on the second trio's shares.
 pub const OTHER_SPINS: [u32; 2] = [0, 1];
@@ -196,6 +199,20 @@
         (self.den == 1).then_some(self.num)
     }
 
+    /// Its whole part, rounded down, and what is left over, 0 or more and
+    /// under 1: 37/4 is 9 and 1/4 (chapter 11's paper spins).
+    pub fn mixed(self) -> (i64, Self) {
+        (
+            self.num.div_euclid(self.den),
+            Self::new(self.num.rem_euclid(self.den), self.den),
+        )
+    }
+
+    /// The nearest float, for drawing.
+    pub fn to_f64(self) -> f64 {
+        self.num as f64 / self.den as f64
+    }
+
     /// The fraction on the dial: its top times the partner of its bottom
     /// (dividing by 3 on eleven marks is multiplying by 4); `None` when the
     /// bottom has no partner.
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -282,6 +282,80 @@
         ("09_7", "for any two", "S-F4.needed", "2"),
         ("09_7", "Two spins give a curve", "S-F6.spin_count", "2"),
         ("09_7", "for any three", "S-F1.threshold", "3"),
+        ("10_1", "Back to trustees four and five", "S-F1.pair", "4,5"),
+        ("10_1", "with their two and six", "S-F1.pair_shares", "2,6"),
+        ("10_1", "Through their two points", "S-F7.pair.count", "2"),
+        ("10_1", "the secret five", "S-F1.secret", "5"),
+        ("10_1", "with spins seven and seven", "S-F7.true_spins", "7:7"),
+        ("10_2", "their two points", "S-F7.pair.count", "2"),
+        ("10_2", "the secret zero instead", "S-F7.asked.first", "0"),
+        ("10_3", "Exactly one", "S-F7.ways_each", "1"),
+        ("10_3", "with spins one and four", "S-F7.spins.0", "1,4"),
+        // the secret zero's curve at each of the pair: position, height, mark
+        (
+            "10_3",
+            "trustee four's height would be sixty-eight, which lands on two",
+            "S-F7.check.4",
+            "4,68,2",
+        ),
+        (
+            "10_3",
+            "trustee five's a hundred and five, which lands on six",
+            "S-F7.check.5",
+            "5,105,6",
+        ),
+        ("10_3", "For the secret one", "S-F7.asked.second", "1"),
+        ("10_3", "the spins would be zero and nine", "S-F7.spins.1", "0,9"),
+        ("10_4", "all eleven", "S-F7.fitting", "11"),
+        (
+            "10_5",
+            "the hundred and twenty-one spin pairs",
+            "S-F6.spin_pairs",
+            "121",
+        ),
+        ("10_5", "exactly one of them", "S-F7.ways_each", "1"),
+        ("10_5", "trustees four and five", "S-F1.pair", "4,5"),
+        ("10_5", "a two and a six", "S-F1.pair_shares", "2,6"),
+        ("11_1", "the same two points", "S-F7.pair.count", "2"),
+        (
+            "11_2",
+            "a hundred and forty-five and two hundred and fifteen",
+            "S-F7.paper.heights",
+            "145,215",
+        ),
+        ("11_2", "For the secret zero", "S-F7.asked.first", "0"),
+        (
+            "11_2",
+            "nine and a quarter and six and three quarters",
+            "S-F7.paper.secret0_spins",
+            "37/4,27/4",
+        ),
+        ("11_2", "Only the secret five", "S-F7.paper.whole_spin_secrets", "5"),
+        ("11_3", "the eleven curves", "S-F7.fitting", "11"),
+        ("11_3", "one pair each", "S-F7.ways_each", "1"),
+        ("11_4", "let trustee one join them", "S-F7.join.trustee", "1"),
+        ("11_4", "holding eight", "S-F7.join.share", "8"),
+        ("11_4", "the eleven curves", "S-F7.fitting", "11"),
+        ("11_4", "through eight at position one", "S-F7.join.point", "1,8"),
+        ("11_5", "Just one", "S-F7.join.count", "1"),
+        ("11_5", "passes position one", "S-F7.join.trustee", "1"),
+        ("11_5", "the curve for five", "S-F7.join.secrets_left", "5"),
+        ("11_5", "passes at eight", "S-F7.join.share", "8"),
+        ("11_5", "from eleven secrets to one", "S-F7.join.drop", "11,1"),
+        ("11_6", "every two shares", "S-F7.pair.count", "2"),
+        // how many shares, then how many secrets they leave, for every pair
+        (
+            "11_6",
+            "Two shares leave all eleven secrets",
+            "S-F7.all_pairs.two_leave",
+            "2,11",
+        ),
+        (
+            "11_6",
+            "any third share leaves exactly one",
+            "S-F7.all_pairs.third_leaves",
+            "3,1",
+        ),
     ];
 
     /// Number words said as an article, not a count, each with its line.
@@ -293,6 +367,8 @@
         ("02_7", "fixed first"),
         ("05_1", "every one of the"),
         ("09_7", "the secret plus spins times powers"), // the rule's shape, no count
+        ("10_4", "Each one is a dealing"),
+        ("11_3", "every one of the"),
     ];
 
     const NUMBER_WORDS: &[&str] = &[
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -4,20 +4,20 @@
 //! S-X rows are the primary source's own numbers. By the full draft's
 //! chapters (docs/storyboard-draft.md): S-F0 and S-F1 serve 01 and 02, S-F2
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
-//! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper and S-F7.join
-//! 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11 19 and 20,
-//! S-F12 21 and 22, and S-F13 23. Chapters 01 to 09 are implemented so far; the
-//! rest are computed and checked now so the draft's numbers are settled
-//! before its scenes exist.
+//! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper, S-F7.join and
+//! S-F7.all_pairs 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11
+//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 11 are
+//! implemented so far; the rest are computed and checked now so the draft's
+//! numbers are settled before its scenes exist.
 
 use std::collections::BTreeSet;
 
 use explainer_kit::Facts;
 
 use crate::model::{
-    self, A_SHARE, ADJACENT, Dial, FOUR_OF_SEVEN, FOUR_OF_SEVEN_SEEN, FOURTH, JOINER, LIAR, LINE_HOLDER, LINE_SLOPE,
-    MARKS, OPENING_TRIO, ORDINARY_PIECES, OTHER_SPINS, PAIR, Padlocks, Ratio, Recipe, SECOND_TRIO, SECRET, SPINS, TEN,
-    TEN_HOLDER, TEN_PARITY_HOLDER, THRESHOLD, TRUSTEES, View, combinations, csv,
+    self, A_SHARE, ADJACENT, ASKED_SECRETS, Dial, FOUR_OF_SEVEN, FOUR_OF_SEVEN_SEEN, FOURTH, JOINER, LIAR, LINE_HOLDER,
+    LINE_SLOPE, MARKS, OPENING_TRIO, ORDINARY_PIECES, OTHER_SPINS, PAIR, Padlocks, Ratio, Recipe, SECOND_TRIO, SECRET,
+    SPINS, TEN, TEN_HOLDER, TEN_PARITY_HOLDER, THRESHOLD, TRUSTEES, View, combinations, csv,
 };
 
 /// Shamir's opening example (section 1, a problem he introduces as Liu's,
@@ -374,6 +374,41 @@
     }
     .possible(dial);
     f.add("S-F7.ruled_out", dial.marks() as usize - pair_left.len())?;
+    // 10: the pair's two points; the secrets asked about after the
+    // dealer's, their spins, and the first one's heights at the pair worked
+    // out and wrapped; the board, each secret's curve at positions 0 to 5,
+    // and how many different marks each position holds
+    f.add("S-F7.pair.count", pair_points.len())?;
+    let [asked, next] = ASKED_SECRETS;
+    f.add("S-F7.asked.first", asked)?;
+    f.add("S-F7.asked.second", next)?;
+    f.add("S-F7.spins.0", csv(&per_secret[asked as usize]))?;
+    f.add("S-F7.spins.1", csv(&per_secret[next as usize]))?;
+    let asked_recipe = Recipe::new(asked, &per_secret[asked as usize]);
+    for &(x, y) in &pair_points {
+        let h = asked_recipe.height(x);
+        if dial.wrap(h) != y {
+            return Err(format!(
+                "the secret {asked}'s curve reaches {h} at {x}, which is not {y}"
+            ));
+        }
+        f.add(&format!("S-F7.check.{x}"), csv(&[i64::from(x), h, i64::from(y)]))?;
+    }
+    let last = TRUSTEES.len() as u32;
+    let board: Vec<Vec<u32>> = dial
+        .all()
+        .map(|s| {
+            let r = Recipe::new(s, &per_secret[s as usize]);
+            (0..=last).map(|x| r.share(dial, x)).collect()
+        })
+        .collect();
+    for (s, along) in board.iter().enumerate() {
+        f.add(&format!("S-F7.board.{s}"), csv(along))?;
+    }
+    let columns: Vec<usize> = (0..=last as usize)
+        .map(|x| distinct(board.iter().map(|along| along[x])))
+        .collect();
+    f.add("S-F7.board.columns", csv(&columns))?;
     // 11: the same pair on ordinary paper
     let paper = [PAIR[0], PAIR[1]].map(|x| (x, recipe.height(x)));
     f.add("S-F7.paper.heights", csv(&paper.map(|p| p.1)))?;
@@ -388,6 +423,8 @@
     )?;
     let (a0, b0) = model::paper_curve_spins(paper, 0).ok_or("no paper curve for secret 0")?;
     f.add("S-F7.paper.secret0_spins", csv(&[a0, b0]))?;
+    let (a5, b5) = model::paper_curve_spins(paper, SECRET).ok_or("no paper curve for the secret")?;
+    f.add("S-F7.paper.secret5_spins", csv(&[a5, b5]))?;
     // 11: a third share joins
     let at_joiner: Vec<u32> = dial
         .all()
@@ -402,6 +439,42 @@
         .filter(|&s| at_joiner[s as usize] == share_at(JOINER))
         .collect();
     f.add("S-F7.join.secrets_left", csv(&left))?;
+    f.add("S-F7.join.point", csv(&[JOINER, share_at(JOINER)]))?;
+    f.add("S-F7.join.count", left.len())?;
+    f.add("S-F7.join.drop", csv(&[pair_left.len(), left.len()]))?;
+    // 11: every pair of trustees, every two shares they could hold and
+    // every third trustee's every share: how many spin pairs each secret
+    // fits, how many secrets two shares leave, and how many a third leaves
+    let pairs = combinations(&TRUSTEES, 2);
+    let (mut ways, mut two_leave, mut third_leaves) = (BTreeSet::new(), BTreeSet::new(), BTreeSet::new());
+    for pair in &pairs {
+        for values in model::tuples(dial, 2) {
+            let points: Vec<(u32, u32)> = pair.iter().copied().zip(values).collect();
+            let fits: Vec<Vec<Vec<u32>>> = dial.all().map(|s| model::fitting_spins(dial, &points, s, 2)).collect();
+            ways.extend(fits.iter().map(Vec::len));
+            two_leave.insert(fits.iter().filter(|w| !w.is_empty()).count());
+            for &j in TRUSTEES.iter().filter(|j| !pair.contains(j)) {
+                for y in dial.all() {
+                    let count = (0..)
+                        .zip(&fits)
+                        .filter(|(s, w)| w.iter().any(|t| Recipe::new(*s, t).share(dial, j) == y))
+                        .count();
+                    third_leaves.insert(count);
+                }
+            }
+        }
+    }
+    f.add("S-F7.all_pairs.pairs", pairs.len())?;
+    f.add("S-F7.all_pairs.share_pairs", model::tuples(dial, 2).count())?;
+    f.add("S-F7.all_pairs.ways_each", set_csv(ways))?;
+    f.add(
+        "S-F7.all_pairs.two_leave",
+        format!("{},{}", PAIR.len(), set_csv(two_leave)),
+    )?;
+    f.add(
+        "S-F7.all_pairs.third_leaves",
+        format!("{THRESHOLD},{}", set_csv(third_leaves)),
+    )?;
 
     // 12, 13: the same recipe, and the line, on ten marks
     let ten_shares = recipe.deal(ten, &TRUSTEES);
@@ -834,15 +907,43 @@
 S-F7.straight_one	8
 S-F7.secrets	0,1,2,3,4,5,6,7,8,9,10
 S-F7.ruled_out	0
+S-F7.pair.count	2
+S-F7.asked.first	0
+S-F7.asked.second	1
+S-F7.spins.0	1,4
+S-F7.spins.1	0,9
+S-F7.check.4	4,68,2
+S-F7.check.5	5,105,6
+S-F7.board.0	0,5,7,6,2,6
+S-F7.board.1	1,10,4,5,2,6
+S-F7.board.2	2,4,1,4,2,6
+S-F7.board.3	3,9,9,3,2,6
+S-F7.board.4	4,3,6,2,2,6
+S-F7.board.5	5,8,3,1,2,6
+S-F7.board.6	6,2,0,0,2,6
+S-F7.board.7	7,7,8,10,2,6
+S-F7.board.8	8,1,5,9,2,6
+S-F7.board.9	9,6,2,8,2,6
+S-F7.board.10	10,0,10,7,2,6
+S-F7.board.columns	11,11,11,11,1,1
 S-F7.paper.heights	145,215
 S-F7.paper.curves	11
 S-F7.paper.whole_spin_secrets	5
 S-F7.paper.secret0_spins	37/4,27/4
+S-F7.paper.secret5_spins	7,7
 S-F7.join.trustee	1
 S-F7.join.share	8
 S-F7.join.values_by_secret	5,10,4,9,3,8,2,7,1,6,0
 S-F7.join.distinct	11
 S-F7.join.secrets_left	5
+S-F7.join.point	1,8
+S-F7.join.count	1
+S-F7.join.drop	11,1
+S-F7.all_pairs.pairs	10
+S-F7.all_pairs.share_pairs	121
+S-F7.all_pairs.ways_each	1
+S-F7.all_pairs.two_leave	2,11
+S-F7.all_pairs.third_leaves	3,1
 S-F8.ten.marks	10
 S-F8.ten.shares	9,7,9,5,5
 S-F8.ten.pair_shares	5,5
@@ -957,7 +1058,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 234);
+        assert_eq!(f.len(), 262);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -31,7 +31,9 @@
   * "three times the secret lands on 4" (chapters 17 and 18) is undone by
     trying every mark;
   * "two shares reveal nothing" is counted: for each secret, the spin pairs
-    out of all p**2 that deal the two shares seen;
+    out of all p**2 that deal the two shares seen; chapter 11's table over
+    every pair, every two shares and every third share deals all p**3
+    recipes and groups them by the shares they deal;
   * the padlock scheme is built as one lock per pair and every group of
     trustees is tried against every lock; Shamir's 462 and 252 are
     math.comb(11, 5) and math.comb(10, 5), pinned to the paper's text;
@@ -77,6 +79,7 @@
 FOURTH = 4                  # chapter 22: the fourth trustee whose share shows a disagreement
 ORDINARY_PIECES = (2, 3)    # chapter 02: A's and B's ordinary pieces of the secret
 JOINER = 1                  # chapter 11: the third trustee who joins the pair
+ASKED = (0, 1)              # chapters 10 and 11: the secrets asked about after the dealer's
 
 # Shamir 1979, section 1: the problem is quoted from Liu (1968); the counts
 # are Shamir's own sentence ("the minimal solution uses 462 locks and 252
@@ -447,6 +450,25 @@
     # 04, 05: the secrets fed in, and those no spin pair deals the pair's shares with
     put("S-F7.secrets", csv(range(P)))
     put("S-F7.ruled_out", sum(1 for ways in per_secret if not ways))
+    # 10: the pair's two points; the secrets asked about after the dealer's, their spins,
+    # and the first one's heights at the pair written out, then wound round the dial; the
+    # board: each secret's curve at positions 0 to 5, and the marks each position holds
+    put("S-F7.pair.count", len(PAIR))
+    put("S-F7.asked.first", ASKED[0])
+    put("S-F7.asked.second", ASKED[1])
+    for i, s in enumerate(ASKED):
+        (a, b), = per_secret[s]
+        put(f"S-F7.spins.{i}", csv((a, b)))
+    (a, b), = per_secret[ASKED[0]]
+    for x, y in zip(PAIR, pv):
+        h = ASKED[0] + a * x + b * x * x
+        check(wind(h)[1] == y, f"the secret {ASKED[0]}'s height {h} at {x} winds onto {y}")
+        put(f"S-F7.check.{x}", csv((x, h, y)))
+    positions = range(len(TRUSTEES) + 1)
+    board = [[share((s, *per_secret[s][0]), x) for x in positions] for s in range(P)]
+    for s, row in enumerate(board):
+        put(f"S-F7.board.{s}", csv(row))
+    put("S-F7.board.columns", csv(len({row[x] for row in board}) for x in positions))
     # the same pair on ordinary paper: a curve through every secret, but whole spins 0 to 10 for one
     tall_pair = [SECRET + SPINS[0] * x + SPINS[1] * x * x for x in PAIR]
     paper_curves = []
@@ -464,6 +486,7 @@
     check(whole == [s for s, a, b in paper_curves if a.denominator == b.denominator == 1 and 0 <= a < P and 0 <= b < P],
           "paper pair: search agrees with the solved curves")
     put("S-F7.paper.secret0_spins", csv(frac(q) for q in paper_curves[0][1:]))
+    put("S-F7.paper.secret5_spins", csv(frac(q) for q in next(c for c in paper_curves if c[0] == SECRET)[1:]))
     # a third share joins: each of the eleven curves meets the joiner's position at its own mark
     at_joiner = [share((s, *per_secret[s][0]), JOINER) for s in range(P)]
     put("S-F7.join.trustee", JOINER)
@@ -471,6 +494,38 @@
     put("S-F7.join.values_by_secret", csv(at_joiner))
     put("S-F7.join.distinct", len(set(at_joiner)))
     put("S-F7.join.secrets_left", csv(s for s in range(P) if at_joiner[s] == shares[JOINER - 1]))
+    joined = [s for s in range(P) if at_joiner[s] == shares[JOINER - 1]]
+    put("S-F7.join.point", csv((JOINER, shares[JOINER - 1])))
+    put("S-F7.join.count", len(joined))
+    put("S-F7.join.drop", csv((sum(1 for ways in per_secret if ways), len(joined))))
+    # 11: every pair of trustees, all 121 pairs of shares and every third trustee's every
+    # share, by dealing all 1331 recipes: how many spin pairs each secret fits two shares
+    # with, how many secrets two shares leave, and how many a third share leaves
+    all_recipes = list(recipes())
+    pairs = list(itertools.combinations(TRUSTEES, 2))
+    share_pairs = list(itertools.product(range(P), repeat=2))
+    ways, two_leave, third_leaves = set(), set(), set()
+    for pair in pairs:
+        by_two = {}
+        for r in all_recipes:
+            by_two.setdefault(tuple(share(r, x) for x in pair), []).append(r[0])
+        for values in share_pairs:
+            got = by_two.get(values, [])
+            ways.update(got.count(s) for s in range(P))
+            two_leave.add(len(set(got)))
+        for j in TRUSTEES:
+            if j in pair:
+                continue
+            by_three = {}
+            for r in all_recipes:
+                by_three.setdefault(tuple(share(r, x) for x in (*pair, j)), set()).add(r[0])
+            for values in itertools.product(range(P), repeat=3):
+                third_leaves.add(len(by_three.get(values, ())))
+    put("S-F7.all_pairs.pairs", len(pairs))
+    put("S-F7.all_pairs.share_pairs", len(share_pairs))
+    put("S-F7.all_pairs.ways_each", csv(sorted(ways)))
+    put("S-F7.all_pairs.two_leave", csv((len(PAIR), *sorted(two_leave))))
+    put("S-F7.all_pairs.third_leaves", csv((THRESHOLD, *sorted(third_leaves))))
 
     # 12, 13: why eleven: the same recipe, and the line, on a ten-mark dial
     ten_curve = [share(recipe, x, TEN) for x in TRUSTEES]
@@ -749,6 +804,17 @@
             for values in itertools.product(range(P), repeat=2):
                 met = [share((s, *fitting_spins(marks, values, s)[0]), j) for s in range(P)]
                 check(sorted(met) == list(range(P)), f"joiner {j} after {marks}={values}")
+    # chapter 11: the joiner's own card leaves the secret alone; every other card it could
+    # hold leaves exactly one other secret, a different one for each card (the control)
+    dealt_ = [share((SECRET, *SPINS), x) for x in TRUSTEES]
+    pair_values = [dealt_[x - 1] for x in PAIR]
+    picked = []
+    for y in range(P):
+        left = sorted({s for s in range(P) for a, b in fitting_spins(PAIR, pair_values, s)
+                       if share((s, a, b), JOINER) == y})
+        check(len(left) == 1 and (left == [SECRET]) == (y == dealt_[JOINER - 1]), f"joiner holding {y}: {left}")
+        picked.extend(left)
+    check(sorted(picked) == list(range(P)), "every card the joiner could hold picks its own secret")
     # negative control 0: ordinary pieces that add up leak; dial pieces never do
     for s in range(P):
         for a in range(s + 1):
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 09, per root's direction in Mail
+chapters 01 to 11, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -175,3 +175,57 @@
 | 09_5 | Now wrap them. ⟨predict⟩ Where does trustee two's forty-seven land on the dial? | The formula, squares and working go; the plot stays at the left. At "Now wrap them" the dial board at the right, positions 0 to 5 numbered under it, rows 0 to 10, "on the dial: points only, no curve", empty. At "trustee two's forty-seven" outlines round the plot's 47 and trustee 2's face-down card [S-F6.asked]. Nothing wraps yet: the think ring with its "?" under the board. | predict | question hold (think time) |
 | 09_6 | Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault. | The "?" goes. A small dial between the plot and the board; a blue coil winds 47 round it, four loops and on, as the needle follows and lands on 3 at "so it lands on three", a ring there and "47 = 4 × 11 + 3" under the dial [S-F6.wrap.2]; trustee 2's point (2, 3) comes onto the board and swells. At "All five shares" the other heights' rows: "19 = 1 × 11 + 8", "89 = 8 × 11 + 1", "145 = 13 × 11 + 2", "215 = 19 × 11 + 6" [S-F6.turns]. At each share as it is said, that trustee's card turns over (its back lifts away, then its number drops in), its point comes onto the board and a ring glows: 8, 3, 1, 2, 6 [S-F6.shares]. At "the very shares from the vault" the caption "the vault's shares: 8, 3, 1, 2, 6" under the board, outlines glowing round all five frames. | coil, card turn, callback | thought after "lands on three."; hold on the matched cards |
 | 09_7 | A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three. | The plot, the small dial, the rows and the caption go. At "the secret plus spins times powers" "secret + spin × position + spin × position × position + …" across the top; at "is called a polynomial" "a polynomial:" before it, with an outline. At "One spin gives a line" a second board at the left with chapter 08's points, its wrapped line and "1 spin: a line, any 2 of 5" under it [S-F4.spin_count, S-F4.needed]; at "for any two" rings round trustees 2 and 3's points. At "Two spins give a curve" "2 spins: a curve, any 3 of 5" under the right board as the secret's gold point joins its five shares [S-F6.spin_count]; at "for any three" teal rings round trustees 1, 2 and 3's points and outlines round their cards [S-F1.threshold]. | card, boards, outline glow | chapter_transition |
+
+## Devices in chapters 10 and 11
+
+- **The pair's board**: chapter 08's dial board, a little narrower and
+  further right, "on the dial: points only", with positions 0 to 5 and rows
+  0 to 10. Every curve is its computed points only: its gold secret at
+  position 0, a point at each of positions 1 to 3, and the pair's purple
+  points at 4 and 5. Straight guides join each point to the next, and the
+  legend under the board says what they are ("lines are guides joining one
+  curve's points"); nothing on the board is a smooth curve and nothing lies
+  between marks (`two_shares.rs`, `Drawn`, checked against the model's
+  brute force).
+- **Spins' tags**: under "spins", left of the row labels, each curve's two
+  spins ("1 and 4") stand beside its secret's row, so the column of tags is
+  eleven different spin pairs.
+- **The spin grid** (10_5): chapter 05's 11 × 11 grid, smaller, at the lower
+  right; each secret's curve lights its one cell, with the secret written in
+  it.
+- **The paper comparison** (11_1 to 11_3): a plot of ordinary heights 0 to
+  220, apart from the board at the lower right, and a window beside it on
+  positions 0 to 1 and heights 0 to 25, enlarged so the eleven smooth curves,
+  which almost coincide on the plot, stand apart at their secrets. Smooth
+  curves are drawn only here, under "on paper: ordinary numbers" (the tests
+  check that they are apart in the window).
+- **Face-down cards**: trustees 1, 2 and 3's cards stay face down while the
+  pair meets; trustee 1's turns over (chapter 09's turn) when it joins.
+- **The table** (11_6): the ten pairs of trustees and three lines, counted by
+  the scene over every pair, both shares and every third share, held until
+  they can be read.
+- Emphasis is by rings and outlines of their own; no typeset text changes
+  color or size, and the only number that changes in place is the strip's
+  live count.
+
+## 10_two_shares: eleven curves, one for each secret
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 10_1 | Back to trustees four and five, with their two and six, and the experiment from before. Now we can see what it found. Through their two points and the secret five runs the dealer's curve, with spins seven and seven. | The rail with the vault's five cards in their frames: 4 and 5 face up, 2 and 6, their discs purple; 1, 2 and 3 face down, their discs faint. The dial board at the left, "on the dial: points only", positions 0 to 5 numbered under it and rows 0 to 10, empty. At "Back to trustees four and five" purple rings round discs 4 and 5 and outlines round their frames [S-F1.pair]; at "with their two and six" their purple points (4, 2) and (5, 6) come onto the board and the row labels 2 and 6 glow [S-F1.pair_shares]. At "the experiment from before" the strip, "for: trustees 4 and 5", all 11 lit, as chapter 05 found [S-F7.ruled_out]; at "Now we can see what it found" an outline glows round its ticks. At "Through their two points" purple rings round them [S-F7.pair.count]; at "and the secret five" the gold point (0, 5) with a ring [S-F1.secret]. At "runs the dealer's curve" its blue points (1, 8), (2, 3) and (3, 1) come in one after another, each joined to the last by a straight blue guide, the last guide reaching the pair and the purple guide joining the pair's two points [S-F6.shares]; under the board the legend "lines are guides joining one curve's points". At "with spins seven and seven" "spins" heads a column left of the row labels and "7 and 7" stands left of row 5, glowing yellow [S-F7.true_spins]. | callback, board, guides, strip | thought after "Now we can see what it found." |
+| 10_2 | ⟨predict⟩ How many curves of this kind run through their two points and the secret zero instead? | The dealer's points go grey and its guides faint; its gold point stays. At "their two points" purple rings round (4, 2) and (5, 6) again [S-F7.pair.count]. At "the secret zero instead" a gold point at (0, 0) with a ring [S-F7.asked.first]. Nothing answers yet: the think ring with its "?" at the lower right. | predict | question hold (think time) |
+| 10_3 | Exactly one, with spins one and four. Check it: trustee four's height would be sixty-eight, which lands on two, and trustee five's a hundred and five, which lands on six. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins. | The "?" goes. At "Exactly one" a white curve grows from (0, 0) through (1, 5), (2, 7) and (3, 6) into the pair's points, point by point along its guides [S-F7.ways_each, S-F7.board.0]; at "with spins one and four" its tag "1 and 4" left of row 0, glowing [S-F7.spins.0]. At each of the pair as it is said, its working at the upper right and an outline round its frame, and at "which lands on" a ring round its point as it swells: "trustee 4: 0 + 1 × 4 + 4 × 4 × 4 = 68 → 2" [S-F7.check.4], then "trustee 5: 0 + 1 × 5 + 4 × 5 × 5 = 105 → 6" [S-F7.check.5]. At "For the secret one" the working goes, the secret 0's curve goes grey, and the secret 1's grows from (0, 1) through (1, 10), (2, 4) and (3, 5), tagged "0 and 9" [S-F7.asked.second, S-F7.board.1]; at "the spins would be zero and nine" the tag glows [S-F7.spins.1]. At "Every secret gets its own curve" it goes grey too and both tags glow. | curve, derivation, outline glow | thought after "with spins one and four."; thought after "which lands on six." |
+| 10_4 | Here are all eleven. Each one is a dealing the dealer could have made, if the secret had been different. | At "Here are all eleven" the other eight curves come in one after another, each from its gold point at position 0 along faint guides through grey points into the pair's, each tagged with its spins left of its secret's row: the gold column holds every mark 0 to 10 and no two tags are the same [S-F7.fitting, S-F7.spins_by_secret, S-F7.board.0, S-F7.board.1, S-F7.board.2, S-F7.board.3, S-F7.board.4, S-F7.board.5, S-F7.board.6, S-F7.board.7, S-F7.board.8, S-F7.board.9, S-F7.board.10]. Positions 1, 2 and 3 each hold all eleven marks, positions 4 and 5 only the pair's [S-F7.board.columns]. At "Each one is a dealing" a gold outline round the column at position 0; at "if the secret had been different" every gold point swells. | lagged, board, outline glow | hold on the eleven curves (2 s) |
+| 10_5 | And the hundred and twenty-one spin pairs are all equally likely. Whatever the secret, exactly one of them hands trustees four and five a two and a six. | At "the hundred and twenty-one spin pairs" chapter 05's spin grid, smaller, at the lower right: 11 × 11 yellow cells, "1st spin" across and "2nd spin" up, 0 to 10 along each, and "11 × 11 = 121 spin pairs" beside it [S-F6.spin_pairs]; its columns come in one after another. At "are all equally likely" "each as likely as any other" under the count, which glows. From "Whatever the secret", secret by secret, its gold point is ringed, its tag glows and its one cell lights gold with the secret written in it: secret 0 at spins (1, 4), 1 at (0, 9), and so on to 10 at (2, 10), eleven different cells [S-F7.spins_by_secret, S-F7.ways_each]. At "hands trustees four and five" rings round the pair's points and outlines round their frames [S-F1.pair, S-F1.pair_shares]. | grid, linked views | thought after "are all equally likely." |
+| 10_6 | So what they hold is just as likely under every secret, and meeting has taught them nothing. | The grid, its numbers, count and note go. A precisely card: "chance of shares 2 and 6, given secret s: 1/121 for every s (one spin pair each). Equal likelihoods, not a uniform prior: any belief survives", two lines [S-F7.chance_each]. At "just as likely under every secret" an outline round the strip's ticks, all 11 still lit for the pair. The card stays until it can be read. | precisely card | chapter_transition; hold until the card is read |
+
+## 11_one_more_share: the picture, the proof, and a third share
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 11_1 | Notice what that argument used. On paper, a curve also runs through any secret and the same two points, so the picture alone can't be the reason. | Chapter 10's end: the board with its eleven curves (grey points, faint guides, gold secrets), their spins' tags and the legend; the pair's cards face up; the strip for the pair, 11 lit. At "Notice what that argument used" a yellow outline round the tags. At "On paper, a curve also runs" the paper comparison at the lower right: a plot, "on paper: ordinary numbers", positions 0 to 5 and heights 0 to 220 labelled every 50, with the pair's purple points labelled 145 and 215 [S-F7.paper.heights]; beside it a window, "near position 0, enlarged", positions 0 to 1 and heights 0 to 25, its region outlined on the plot. For each secret a smooth curve grows on both, eleven in all [S-F7.paper.curves], with a small gold dot at its secret in the window: on the plot the curves almost coincide; in the window they stand apart. At "the same two points" purple rings round the paper points and the board's [S-F7.pair.count]. At "so the picture alone" outlines round the board and round the paper: both show a curve through every secret. | coordinated views, plot, window | thought after "Notice what that argument used."; cue_transition after "can't be the reason." |
+| 11_2 | On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins, so on paper their heights give the secret away. | At "their heights are" the labels 145 and 215 glow [S-F7.paper.heights]. At "For the secret zero" its curve turns white on the plot and in the window, a ring at its gold dot [S-F7.asked.first]; at "needs spins of nine and a quarter" "secret 0: spins 9 + 1/4 and 6 + 3/4" at the upper right [S-F7.paper.secret0_spins]; at "and no spin can be that" "a spin is a whole mark, 0 to 10" lower down, the curve goes faint and a red outline glows round its spins. At "Only the secret five gets whole spins" "secret 5: spins 7 and 7" in blue between them [S-F7.paper.secret5_spins]; the secret 5's curve turns blue and all the others faint, a ring at its dot [S-F7.paper.whole_spin_secrets]. At "give the secret away" the strip shows "for: trustees 4 and 5, on paper", only 5 lit, and a red outline glows round its ticks. | highlight, live count | thought after "two hundred and fifteen."; thought after "no spin can be that." |
+| 11_3 | On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the pictures only help us see it. | At "On the dial, every one" the paper's spins and note go and a yellow outline glows round the board's tags. At "comes from real spins" the strip is back to "for: trustees 4 and 5", 11 lit. At "one pair each" every tag glows [S-F7.fitting, S-F7.ways_each]. At "all equally likely" "each secret: exactly 1 of the 121 spin pairs" at the upper right in yellow [S-F6.spin_pairs]. At "That count is the proof" the paper plot and window go and the count glows. At "the pictures only help us see it" an outline round the board. | linked views, outline glow | thought after "all equally likely." |
+| 11_4 | Now let trustee one join them, holding eight. ⟨predict⟩ How many of the eleven curves pass through eight at position one? | At "Now let trustee one join them" the count goes; trustee 1's disc turns teal with a ring [S-F7.join.trustee]. At "holding eight" its card turns over, the back lifting away before 8 drops in, an outline round its frame [S-F7.join.share]; a teal point at (1, 8) on the board with a ring, and the row label 8 glows [S-F7.join.point]. At "pass through eight at position one" a teal outline round the board's column at position 1. Nothing answers yet: the think ring with its "?" at the lower right. | card turn, predict | question hold (think time) |
+| 11_5 | Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one. | The "?" goes. At "Just one" the teal point swells and a teal ring round the secret 5's point there [S-F7.join.count]. At "Each curve passes position one" "each secret's curve at position 1:" under the strip, then secret by secret a white ring round its curve's point at position 1 and that mark under the secret's tick: 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0 [S-F7.join.values_by_secret]. At "at a different mark" outlines round the column at position 1 and round the row: eleven different marks [S-F7.join.distinct]. At "only the curve for five passes at eight" its points and guides turn blue and are ringed, its tag glows and the 8 under tick 5 glows teal; every other curve and its gold point go faint and its tag leaves [S-F7.join.secrets_left]. At "The strip drops from eleven secrets to one" discs 1, 4 and 5 turn teal and the strip shows "for: trustees 1, 4 and 5", only 5 lit [S-F7.join.drop]. | live count, highlight | thought after "Just one."; hold on the single curve (1.5 s) |
+| 11_6 | The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one. | The row and its title go. At "The same count holds for every pair of trustees" "every pair of trustees (10 pairs):" under the strip, then the ten pairs, "1 and 2" to "4 and 5", in two rows of five [S-F7.all_pairs.pairs]. At "every two shares they could hold" "each holding any of the 121 pairs of shares:" [S-F7.all_pairs.share_pairs]. At "Two shares leave all eleven secrets" "2 shares: every secret fits exactly 1 spin pair" in purple [S-F7.all_pairs.ways_each, S-F7.all_pairs.two_leave]. At "any third share leaves exactly one" "+ any third share: exactly 1 secret left" in teal, glowing [S-F7.all_pairs.third_leaves]. The scene counts the table itself over every pair, both shares and every third share; oracle.py recounts it by dealing every recipe. The lines stay until they can be read. | table | chapter_transition; hold until the table is read |
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -277,23 +277,23 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 10_1 | Back to trustees four and five, with their two and six, and the experiment from before. Now we can see what it found. Through their two points and the secret five runs the dealer's curve, with spins seven and seven. | Dial board with only (4, 2) and (5, 6) in purple [S-F1.pair_shares], the rail below with 4 and 5 lit. The true curve in blue through (0, 5), labelled "7 and 7" [S-F7.true_spins]. | callback, curve | thought after "now we can see what it found." |
-| 10_2 | ⟨predict⟩ How many curves of this kind run through their two points and the secret zero instead? | A gold point at (0, 0); the blue curve dims. | predict | question hold (think time) |
-| 10_3 | Exactly one, with spins one and four. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins. | One faint curve threads (0, 0), (4, 2) and (5, 6), labelled "1 and 4"; then (0, 1) with "0 and 9" [S-F7.spins_by_secret]. | curve | thought after "one and four." |
-| 10_4 | Here are all eleven. Each one is a dealing the dealer could have made, if the secret had been different. | All eleven curves ghosted, the gold column filled 0 to 10, each curve tagged with its spins [S-F7.spins_by_secret]. | lagged | hold on the eleven curves |
-| 10_5 | And the hundred and twenty-one spin pairs are all equally likely. Whatever the secret, exactly one of them hands trustees four and five a two and a six. | The spin grid from chapter 05 returns beside the board [S-F6.spin_pairs]; selecting a curve lights its one cell [S-F7.fitting, S-F7.ways_each]. | grid, linked views | thought after "equally likely." |
-| 10_6 | So what they hold is just as likely under every secret, and meeting has taught them nothing. | precisely: "for every secret s, P(shares 2 and 6 \| s) = 1/121: the two shares do not depend on the secret, so any belief about it is unchanged" [S-F7.chance_each]. The pair's strip: full, count 11. | precisely card | chapter_transition |
+| 10_1 | Back to trustees four and five, with their two and six, and the experiment from before. Now we can see what it found. Through their two points and the secret five runs the dealer's curve, with spins seven and seven. | The rail with the vault's five cards in their frames: 4 and 5 face up, 2 and 6, their discs purple; 1, 2 and 3 face down, their discs faint. The dial board at the left, "on the dial: points only", positions 0 to 5 numbered under it and rows 0 to 10, empty. At "Back to trustees four and five" purple rings round discs 4 and 5 and outlines round their frames [S-F1.pair]; at "with their two and six" their purple points (4, 2) and (5, 6) come onto the board and the row labels 2 and 6 glow [S-F1.pair_shares]. At "the experiment from before" the strip, "for: trustees 4 and 5", all 11 lit, as chapter 05 found [S-F7.ruled_out]; at "Now we can see what it found" an outline glows round its ticks. At "Through their two points" purple rings round them [S-F7.pair.count]; at "and the secret five" the gold point (0, 5) with a ring [S-F1.secret]. At "runs the dealer's curve" its blue points (1, 8), (2, 3) and (3, 1) come in one after another, each joined to the last by a straight blue guide, the last guide reaching the pair and the purple guide joining the pair's two points [S-F6.shares]; under the board the legend "lines are guides joining one curve's points". At "with spins seven and seven" "spins" heads a column left of the row labels and "7 and 7" stands left of row 5, glowing yellow [S-F7.true_spins]. | callback, board, guides, strip | thought after "Now we can see what it found." |
+| 10_2 | ⟨predict⟩ How many curves of this kind run through their two points and the secret zero instead? | The dealer's points go grey and its guides faint; its gold point stays. At "their two points" purple rings round (4, 2) and (5, 6) again [S-F7.pair.count]. At "the secret zero instead" a gold point at (0, 0) with a ring [S-F7.asked.first]. Nothing answers yet: the think ring with its "?" at the lower right. | predict | question hold (think time) |
+| 10_3 | Exactly one, with spins one and four. Check it: trustee four's height would be sixty-eight, which lands on two, and trustee five's a hundred and five, which lands on six. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins. | The "?" goes. At "Exactly one" a white curve grows from (0, 0) through (1, 5), (2, 7) and (3, 6) into the pair's points, point by point along its guides [S-F7.ways_each, S-F7.board.0]; at "with spins one and four" its tag "1 and 4" left of row 0, glowing [S-F7.spins.0]. At each of the pair as it is said, its working at the upper right and an outline round its frame, and at "which lands on" a ring round its point as it swells: "trustee 4: 0 + 1 × 4 + 4 × 4 × 4 = 68 → 2" [S-F7.check.4], then "trustee 5: 0 + 1 × 5 + 4 × 5 × 5 = 105 → 6" [S-F7.check.5]. At "For the secret one" the working goes, the secret 0's curve goes grey, and the secret 1's grows from (0, 1) through (1, 10), (2, 4) and (3, 5), tagged "0 and 9" [S-F7.asked.second, S-F7.board.1]; at "the spins would be zero and nine" the tag glows [S-F7.spins.1]. At "Every secret gets its own curve" it goes grey too and both tags glow. | curve, derivation, outline glow | thought after "with spins one and four."; thought after "which lands on six." |
+| 10_4 | Here are all eleven. Each one is a dealing the dealer could have made, if the secret had been different. | At "Here are all eleven" the other eight curves come in one after another, each from its gold point at position 0 along faint guides through grey points into the pair's, each tagged with its spins left of its secret's row: the gold column holds every mark 0 to 10 and no two tags are the same [S-F7.fitting, S-F7.spins_by_secret, S-F7.board.0, S-F7.board.1, S-F7.board.2, S-F7.board.3, S-F7.board.4, S-F7.board.5, S-F7.board.6, S-F7.board.7, S-F7.board.8, S-F7.board.9, S-F7.board.10]. Positions 1, 2 and 3 each hold all eleven marks, positions 4 and 5 only the pair's [S-F7.board.columns]. At "Each one is a dealing" a gold outline round the column at position 0; at "if the secret had been different" every gold point swells. | lagged, board, outline glow | hold on the eleven curves (2 s) |
+| 10_5 | And the hundred and twenty-one spin pairs are all equally likely. Whatever the secret, exactly one of them hands trustees four and five a two and a six. | At "the hundred and twenty-one spin pairs" chapter 05's spin grid, smaller, at the lower right: 11 × 11 yellow cells, "1st spin" across and "2nd spin" up, 0 to 10 along each, and "11 × 11 = 121 spin pairs" beside it [S-F6.spin_pairs]; its columns come in one after another. At "are all equally likely" "each as likely as any other" under the count, which glows. From "Whatever the secret", secret by secret, its gold point is ringed, its tag glows and its one cell lights gold with the secret written in it: secret 0 at spins (1, 4), 1 at (0, 9), and so on to 10 at (2, 10), eleven different cells [S-F7.spins_by_secret, S-F7.ways_each]. At "hands trustees four and five" rings round the pair's points and outlines round their frames [S-F1.pair, S-F1.pair_shares]. | grid, linked views | thought after "are all equally likely." |
+| 10_6 | So what they hold is just as likely under every secret, and meeting has taught them nothing. | The grid, its numbers, count and note go. A precisely card: "chance of shares 2 and 6, given secret s: 1/121 for every s (one spin pair each). Equal likelihoods, not a uniform prior: any belief survives", two lines [S-F7.chance_each]. At "just as likely under every secret" an outline round the strip's ticks, all 11 still lit for the pair. The card stays until it can be read. | precisely card | chapter_transition; hold until the card is read |
 
 ## 11_one_more_share: the picture, the proof, and a third share
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 11_1 | Notice what that argument used. On paper, a curve also runs through any secret and the same two points, so the picture alone can't be the reason. | The paper plot beside the dial board: the pair's ordinary heights 145 and 215 [S-F7.paper.heights], with a curve through (0, s) for each s, eleven in all [S-F7.paper.curves]. | coordinated views | thought after "that argument used." |
-| 11_2 | On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins. | The secret-0 paper curve tagged "9¼ and 6¾" [S-F7.paper.secret0_spins] in red; the other non-whole curves red; only the secret-5 curve, "7 and 7", stays blue [S-F7.paper.whole_spin_secrets]. The paper strip: 1 lit. | highlight, live count | thought after "two hundred and fifteen."; thought after "no spin can be that." |
-| 11_3 | On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the curves just help us see it. | Dial side: all eleven curves blue, each with its spin pair and one grid cell; the dial strip: 11 lit. | linked views | thought after "all equally likely." |
-| 11_4 | Now let trustee one join them, holding eight. ⟨predict⟩ How many of the eleven curves pass through eight at position one? | Trustee 1 lights on the rail; its disc slides up to position 1 with its card 8 [S-F7.join.trustee, S-F7.join.share]; the eleven curves still ghosted. | predict | question hold (think time) |
-| 11_5 | Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one. | Each curve's height at position 1 marked, by secret 0 … 10: 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0 [S-F7.join.values_by_secret, S-F7.join.distinct]; all but the secret-5 curve fade; strip count 11 → 1, tick 5 [S-F7.join.secrets_left]. | live count, highlight | thought after "Just one."; hold on the single curve |
-| 11_6 | The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one. | A small table of the ten pairs 1·2 … 4·5, each ticked "every secret: 1 curve; any third share: 1 secret" (oracle self-test). | table | chapter_transition |
+| 11_1 | Notice what that argument used. On paper, a curve also runs through any secret and the same two points, so the picture alone can't be the reason. | Chapter 10's end: the board with its eleven curves (grey points, faint guides, gold secrets), their spins' tags and the legend; the pair's cards face up; the strip for the pair, 11 lit. At "Notice what that argument used" a yellow outline round the tags. At "On paper, a curve also runs" the paper comparison at the lower right: a plot, "on paper: ordinary numbers", positions 0 to 5 and heights 0 to 220 labelled every 50, with the pair's purple points labelled 145 and 215 [S-F7.paper.heights]; beside it a window, "near position 0, enlarged", positions 0 to 1 and heights 0 to 25, its region outlined on the plot. For each secret a smooth curve grows on both, eleven in all [S-F7.paper.curves], with a small gold dot at its secret in the window: on the plot the curves almost coincide; in the window they stand apart. At "the same two points" purple rings round the paper points and the board's [S-F7.pair.count]. At "so the picture alone" outlines round the board and round the paper: both show a curve through every secret. | coordinated views, plot, window | thought after "Notice what that argument used."; cue_transition after "can't be the reason." |
+| 11_2 | On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins, so on paper their heights give the secret away. | At "their heights are" the labels 145 and 215 glow [S-F7.paper.heights]. At "For the secret zero" its curve turns white on the plot and in the window, a ring at its gold dot [S-F7.asked.first]; at "needs spins of nine and a quarter" "secret 0: spins 9 + 1/4 and 6 + 3/4" at the upper right [S-F7.paper.secret0_spins]; at "and no spin can be that" "a spin is a whole mark, 0 to 10" lower down, the curve goes faint and a red outline glows round its spins. At "Only the secret five gets whole spins" "secret 5: spins 7 and 7" in blue between them [S-F7.paper.secret5_spins]; the secret 5's curve turns blue and all the others faint, a ring at its dot [S-F7.paper.whole_spin_secrets]. At "give the secret away" the strip shows "for: trustees 4 and 5, on paper", only 5 lit, and a red outline glows round its ticks. | highlight, live count | thought after "two hundred and fifteen."; thought after "no spin can be that." |
+| 11_3 | On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the pictures only help us see it. | At "On the dial, every one" the paper's spins and note go and a yellow outline glows round the board's tags. At "comes from real spins" the strip is back to "for: trustees 4 and 5", 11 lit. At "one pair each" every tag glows [S-F7.fitting, S-F7.ways_each]. At "all equally likely" "each secret: exactly 1 of the 121 spin pairs" at the upper right in yellow [S-F6.spin_pairs]. At "That count is the proof" the paper plot and window go and the count glows. At "the pictures only help us see it" an outline round the board. | linked views, outline glow | thought after "all equally likely." |
+| 11_4 | Now let trustee one join them, holding eight. ⟨predict⟩ How many of the eleven curves pass through eight at position one? | At "Now let trustee one join them" the count goes; trustee 1's disc turns teal with a ring [S-F7.join.trustee]. At "holding eight" its card turns over, the back lifting away before 8 drops in, an outline round its frame [S-F7.join.share]; a teal point at (1, 8) on the board with a ring, and the row label 8 glows [S-F7.join.point]. At "pass through eight at position one" a teal outline round the board's column at position 1. Nothing answers yet: the think ring with its "?" at the lower right. | card turn, predict | question hold (think time) |
+| 11_5 | Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one. | The "?" goes. At "Just one" the teal point swells and a teal ring round the secret 5's point there [S-F7.join.count]. At "Each curve passes position one" "each secret's curve at position 1:" under the strip, then secret by secret a white ring round its curve's point at position 1 and that mark under the secret's tick: 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0 [S-F7.join.values_by_secret]. At "at a different mark" outlines round the column at position 1 and round the row: eleven different marks [S-F7.join.distinct]. At "only the curve for five passes at eight" its points and guides turn blue and are ringed, its tag glows and the 8 under tick 5 glows teal; every other curve and its gold point go faint and its tag leaves [S-F7.join.secrets_left]. At "The strip drops from eleven secrets to one" discs 1, 4 and 5 turn teal and the strip shows "for: trustees 1, 4 and 5", only 5 lit [S-F7.join.drop]. | live count, highlight | thought after "Just one."; hold on the single curve (1.5 s) |
+| 11_6 | The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one. | The row and its title go. At "The same count holds for every pair of trustees" "every pair of trustees (10 pairs):" under the strip, then the ten pairs, "1 and 2" to "4 and 5", in two rows of five [S-F7.all_pairs.pairs]. At "every two shares they could hold" "each holding any of the 121 pairs of shares:" [S-F7.all_pairs.share_pairs]. At "Two shares leave all eleven secrets" "2 shares: every secret fits exactly 1 spin pair" in purple [S-F7.all_pairs.ways_each, S-F7.all_pairs.two_leave]. At "any third share leaves exactly one" "+ any third share: exactly 1 secret left" in teal, glowing [S-F7.all_pairs.third_leaves]. The scene counts the table itself over every pair, both shares and every third share; oracle.py recounts it by dealing every recipe. The lines stay until they can be read. | table | chapter_transition; hold until the table is read |
 
 ## 12_a_ten_mark_dial: why not ten?
 
--- a/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
+++ b/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
@@ -61,13 +61,13 @@
 09_7	A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three.
 10_1	Back to trustees four and five, with their two and six, and the experiment from before. Now we can see what it found. Through their two points and the secret five runs the dealer's curve, with spins seven and seven.
 10_2	How many curves of this kind run through their two points and the secret zero instead?
-10_3	Exactly one, with spins one and four. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins.
+10_3	Exactly one, with spins one and four. Check it: trustee four's height would be sixty-eight, which lands on two, and trustee five's a hundred and five, which lands on six. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins.
 10_4	Here are all eleven. Each one is a dealing the dealer could have made, if the secret had been different.
 10_5	And the hundred and twenty-one spin pairs are all equally likely. Whatever the secret, exactly one of them hands trustees four and five a two and a six.
 10_6	So what they hold is just as likely under every secret, and meeting has taught them nothing.
 11_1	Notice what that argument used. On paper, a curve also runs through any secret and the same two points, so the picture alone can't be the reason.
-11_2	On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins.
-11_3	On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the curves just help us see it.
+11_2	On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins, so on paper their heights give the secret away.
+11_3	On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the pictures only help us see it.
 11_4	Now let trustee one join them, holding eight. How many of the eleven curves pass through eight at position one?
 11_5	Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one.
 11_6	The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one.
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -59,3 +59,15 @@
 09_5	Now wrap them. Where does trustee two's forty-seven land on the dial?
 09_6	Forty-seven is four elevens and three more, so it lands on three. All five shares come out eight, three, one, two and six, the very shares from the vault.
 09_7	A rule like this, the secret plus spins times powers of the position, is called a polynomial. One spin gives a line, for any two. Two spins give a curve, for any three.
+10_1	Back to trustees four and five, with their two and six, and the experiment from before. Now we can see what it found. Through their two points and the secret five runs the dealer's curve, with spins seven and seven.
+10_2	How many curves of this kind run through their two points and the secret zero instead?
+10_3	Exactly one, with spins one and four. Check it: trustee four's height would be sixty-eight, which lands on two, and trustee five's a hundred and five, which lands on six. For the secret one, the spins would be zero and nine. Every secret gets its own curve and its own pair of spins.
+10_4	Here are all eleven. Each one is a dealing the dealer could have made, if the secret had been different.
+10_5	And the hundred and twenty-one spin pairs are all equally likely. Whatever the secret, exactly one of them hands trustees four and five a two and a six.
+10_6	So what they hold is just as likely under every secret, and meeting has taught them nothing.
+11_1	Notice what that argument used. On paper, a curve also runs through any secret and the same two points, so the picture alone can't be the reason.
+11_2	On paper, their heights are a hundred and forty-five and two hundred and fifteen. For the secret zero, the curve through them needs spins of nine and a quarter and six and three quarters, and no spin can be that. Only the secret five gets whole spins, so on paper their heights give the secret away.
+11_3	On the dial, every one of the eleven curves comes from real spins, one pair each, all equally likely. That count is the proof, and the pictures only help us see it.
+11_4	Now let trustee one join them, holding eight. How many of the eleven curves pass through eight at position one?
+11_5	Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one.
+11_6	The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one.
--- a/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
+++ b/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored intentions for chapters06/07 before synthesis. Preserve scene prediction and sequential quotation/card reading holds separately. Earlier chapters retain storyboard directions. Durations are proposals, not measured or heard acceptance.",
+  "_comment": "Authored intentions for chapters06/07 and 10/11 before synthesis. Preserve scene prediction and sequential quotation/card reading holds separately (10_2 and 11_4 think rings, 10_4 and 11_5 looking holds, 10_6 card and 11_6 table reading holds). Earlier chapters retain storyboard directions. Durations are proposals, not measured or heard acceptance.",
   "overrides": [
     {"video":"shamir_secret_sharing","cue":"06_1","after":"any three of five","role":"thought","range_s":[0.5,0.9],"reason":"Recall the opening rule before constructing locks."},
     {"video":"shamir_secret_sharing","cue":"06_5","after":"their own","role":"thought","range_s":[0.5,0.9],"reason":"Let the single closed lock land before proving minimality."},
@@ -8,6 +8,16 @@
     {"video":"shamir_secret_sharing","cue":"07_1","after":"any two of five","role":"thought","range_s":[0.6,1.0],"reason":"Make the change of threshold explicit before axes."},
     {"video":"shamir_secret_sharing","cue":"07_2","after":"say it gives four","role":"thought","range_s":[0.5,0.9],"reason":"Connect the random spin to the slope before drawing it."},
     {"video":"shamir_secret_sharing","cue":"07_3","after":"at their own position","role":"thought","range_s":[0.5,0.9],"reason":"Establish position versus height before the sequence of shares."},
-    {"video":"shamir_secret_sharing","cue":"07_6","after":"zero, five or ten","role":"thought","range_s":[0.7,1.1],"reason":"Let the three possibilities appear before the exhaustive argument."}
+    {"video":"shamir_secret_sharing","cue":"07_6","after":"zero, five or ten","role":"thought","range_s":[0.7,1.1],"reason":"Let the three possibilities appear before the exhaustive argument."},
+    {"video":"shamir_secret_sharing","cue":"10_1","after":"Now we can see what it found","role":"thought","range_s":[0.5,0.9],"reason":"Let the recalled full strip register before the curve that explains it."},
+    {"video":"shamir_secret_sharing","cue":"10_3","after":"with spins one and four","role":"thought","range_s":[0.5,0.9],"reason":"Answer the prediction, then pause before checking it."},
+    {"video":"shamir_secret_sharing","cue":"10_3","after":"which lands on six","role":"thought","range_s":[0.4,0.8],"reason":"Close the two-line check before the next secret."},
+    {"video":"shamir_secret_sharing","cue":"10_5","after":"are all equally likely","role":"thought","range_s":[0.5,0.9],"reason":"Let fairness of the spins land before each secret lights its one cell."},
+    {"video":"shamir_secret_sharing","cue":"11_1","after":"Notice what that argument used","role":"thought","range_s":[0.5,0.9],"reason":"Mark the turn from the result to what the argument depended on."},
+    {"video":"shamir_secret_sharing","cue":"11_1","after":"can't be the reason","role":"cue_transition","range_s":[0.8,1.3],"reason":"Let the claim stand before the paper numbers test it."},
+    {"video":"shamir_secret_sharing","cue":"11_2","after":"two hundred and fifteen","role":"thought","range_s":[0.5,0.9],"reason":"Separate the two heights from the fractional spins they force."},
+    {"video":"shamir_secret_sharing","cue":"11_2","after":"no spin can be that","role":"thought","range_s":[0.6,1.0],"reason":"Let the impossibility register before the one whole-spin secret."},
+    {"video":"shamir_secret_sharing","cue":"11_3","after":"all equally likely","role":"thought","range_s":[0.5,0.9],"reason":"Let the counting claim stand before naming it the proof."},
+    {"video":"shamir_secret_sharing","cue":"11_5","after":"Just one","role":"thought","range_s":[0.5,0.9],"reason":"Answer the prediction, then show why."}
   ]
 }
```
<!-- shamir-1011 patch end -->

## Chapters 12 and 13, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment in Mail 6934 (13:45Z), after it froze the 10/11 handoff
(Mail 6933) by hash in
`narration_lab/third-wave-20261024/root/tend-1336/shamir-10-11-intake/`.
The ask:
- write native chapters `12_a_ten_mark_dial` and `13_why_prime` on the
  vault, trustees 4 and 5, the secret 5 and spins 7 and 7;
- let the viewer see the ten-mark dial lose secrecy before any word about
  zero divisors or primes;
- put the prediction markers at the end of the full questions;
- derive the pair's weighted combination with visible cancellation, not an
  abrupt formula;
- show the ten- and eleven-mark results side by side;
- contrast ×5's collisions on ten marks with its eleven distinct results on
  eleven marks, giving the permutation time and not a fast spoken list;
- name "prime" only after the failure and the success, prove the nonzero
  product property accessibly, and never imply that composite dials always
  reveal everything;
- keep equal likelihood apart from a uniform prior, dial points apart from
  paper curves, and claim no security certification;
- enumerate every stated fact independently, with wrong-modulus and
  wrong-card controls, phrase-cue bindings, holds and authored pause
  intentions;
- deliver a separate integration patch after the 10/11 patch, preserving
  root's HEIGHT_TITLE_GAP, sequential reading deadlines and the
  `Marker.disc` restoration.

**Status.**
- I wrote two new unregistered modules, `a_ten_mark_dial.rs` and
  `why_prime.rs`. I edited no registered, delivered or shared file;
  `two_shares.rs` and `one_more_share.rs` are still 7ffd8444… and
  30f4d0da…, as root froze them.
- Nothing was compiled, rendered, synthesized or listened to. No Cargo
  command was run.
- Bead mx-video-program-program-87do.46.3 stays in progress; I close
  nothing.

**The base moved under this work.** Root integrated 08/09 into live at
about 14:30Z (Mail 6979). It also repaired:
- 09_7: the card rail is retired before the one-spin board appears;
- `Board.edges` and the unused `holder_point` binding are removed;
- the `nearest_top(28, 11)` test expectation is now 33;
- the storyboards and `pause-overrides.json`, the latter re-written as
  indented JSON.

The frozen 10/11 patch (f865d6bf…) **no longer applies to live**:
`docs/storyboard.md` hunk 2 and both `docs/pause-overrides.json` hunks
fail. Every other hunk applies with no offset or fuzz.

So I rebased it by hand onto live, as
`docs/previous/ch1213-evidence-20261024/rebased-integration-10-11.patch`
(503c9a52…):
- `compare_1011.py` checks that it adds and removes exactly the same lines
  as the frozen patch in nine of the ten files;
- in the tenth, `pause-overrides.json`, it adds the same ten 10/11 entries,
  equal as JSON data, in root's indented format, with root's comment
  changed only from "chapters06–09" to "chapters06–11".

The 12/13 patch applies after that rebased 10/11 patch. Nothing in 12/13
uses `Board.edges`, `holder_point` or the old 09_7 end state. Root's
HEIGHT_TITLE_GAP (`plot.rs`), `Marker.disc` and its 08_5 consumer are
untouched; no patch here touches `plot.rs`, `draw.rs`, `board.rs` or a
registered chapter.

**Defect found in the frozen 10/11 bytes (not fixed by me).**
`one_more_share.rs:397` and `:399` call `a.rt(..)` on an fmn `Transform`.
`rt` exists only as a method of `explainer_kit::Timed`, and
`one_more_share.rs:34` does not import `Timed`, so the module will fail
with E0599. I checked this against the pinned fmn a399d62: `Transform` has
no inherent `rt`, and every root-built chapter that calls `.rt` imports
`Timed`. The one-line fix is in
`docs/previous/ch1213-evidence-20261024/proposed-one_more_share-timed.patch`
(8398d680…): `use explainer_kit::{Kit, Palette, Timed, fade_all, v};`.
rustfmt passes on the fixed copy. I did not touch the frozen file.

The same mistake was in my own `a_ten_mark_dial.rs` (line 479, the bars'
`grow_from_edge(...).rt(rt)`), and I fixed it there by importing `Timed`.
`lint_mirror.py` now checks for this. It flags the frozen
`one_more_share.rs` and my pre-fix `a_ten_mark_dial.rs`, and nothing in
the 16 root-built chapter modules.

**Reservations.**
- Released after Mail 6934, as it asked: 8251 (`two_shares.rs`) and 8252
  (`one_more_share.rs`).
- Held: 8376 (`a_ten_mark_dial.rs`), 8377 (`why_prime.rs`) and 8378
  (`docs/previous/ch1213-*/**`), all until 19:03Z; 7941 (this file); 8254
  (`docs/previous/ch1011-*/**`); and 8119 (`docs/previous/ch0809-*/**`).
  I wrote nothing under the last two in this round.

**Prior bytes.** `docs/previous/ch1213-before-12-13-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section: sha256
db410efd088e731c830d1849d2018fd6aaa053793be34e2261faca1b9964911e, the
bytes root froze.

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `a_ten_mark_dial.rs` | 1412 | 892cd8f7c955cc4c2bce71f7c5f6b48689816fbc37fd03b500b784e9ff2ca98f |
| `why_prime.rs` | 1432 | c24ab6231115575681cba2f400a78cbd5bd69cd2d893a6bfe7f3db1367e6eea4 |

- **`a_ten_mark_dial.rs`**: `ATenMarkDial`, key `12_a_ten_mark_dial`.
  - Data: `tally`, `tally_title`, `fits_line`, `amounts`, `scaled`,
    `copies`, `difference`, `term`, `terms`, `landed`, `sum_pieces`,
    `table`, `table_row` and `eleven_row`.
  - Also `read_on` and `text_after`, which `why_prime.rs` uses.
  - Five tests:
    - `every_phrase_the_scene_waits_for_is_in_its_line` (45 phrases, in
      order);
    - `the_pictures_are_the_facts`;
    - `the_tallies_and_the_shortcut_hold_for_every_card`, with the controls
      (wrong card 6 and 5 → secret 0; 12 marks leave three secrets, refused;
      9 marks give a level tally; on 12 marks −20 lands on 4b);
    - `the_table_is_counted`, with a 12-mark control and trustees 1 and 3
      recounted by hand;
    - `terms_and_sums_read_as_written`;
    - `the_layout_keeps_its_pieces_apart`.
- **`why_prime.rs`**: `WhyPrime`, key `13_why_prime`.
  - Data: `arrows`, `hits`, `reached`, `reach_line`, `times_title`,
    `wrap_note`, `fitting`, `fits_title`, `line_name`, `holder_rule`,
    `product_pieces`, `split_line`, `lands_line`, `prime_line` (an error for
    a composite), `proof_lines` (an error unless prime), `same_mark_lines`,
    `reach_rows`, `grid_title` and `card_text`.
  - Six tests:
    - `every_phrase_the_scene_waits_for_is_in_its_line` (40 phrases, and
      13_4 ends with the whole question);
    - `the_pictures_are_the_facts`;
    - `the_rows_are_counted_by_hand`, with the controls (wrong card 6 → 1
      and 6; ×5 shuffles 12 marks, and 2, 3, 4, 6, 8, 9 and 10 don't; on 9
      marks ×3 reaches 0, 3 and 6; on 49 marks ×1 to ×6 shuffle and 49 is
      not prime);
    - `the_proof_holds_for_every_product`;
    - `the_card_is_two_lines` (wraps 72 to 92; the card never says needs,
      only, secure or safe);
    - `the_layout_keeps_its_pieces_apart` (pure-constant asserts in
      `const { }` blocks).

### The chapters against the brief

Line ids are 12_1 to 12_8 and 13_1 to 13_9. **The brief's "13_3" is my
13_4**: 13_1 to 13_3 show the failure before the eleven-mark question. Both
prediction markers end their lines:
- 12_1 ends "Would trustees four and five still learn nothing? ⟨predict⟩";
- 13_4 ends "How many different marks come out? ⟨predict⟩".

Each has a think ring and its "question hold (think time)".

- **Seen before it is explained (12_1 to 12_3).** The rail holds the
  vault's cards 8, 3, 1, 2, 6, with each trustee's height on paper over its
  frame. On ten marks each height lands on its last digit, and red frames
  show 9, 7, 9, 5, 5: the pair holds 5 and 5, not 2 and 6. Chapter 05's
  tally then appears twice, on the strip's columns:
  - above, "11 marks: trustees 4 and 5 hold 2 and 6", with every secret
    fitting one way;
  - under it, "10 marks: … hold 5 and 5", with only secret 5 fitting, ten
    ways.

  The card values and the scope differ in each title. A 1.5 s hold
  (`HOLD_ON_TALLIES`) follows.
- **Visible cancellation (12_4 to 12_5).** The heights are written in three
  columns: s, a and b.
  - Trustee 4: s + 4a + 16b. Trustee 5: s + 5a + 25b.
  - Five copies of the first and four of the second both hold 20a. Those
    two 20a terms are outlined before the subtraction, struck out after
    it, and the column then reads "0".
  - The result row is s − 20b. The copy counts come from
    `model::pair_shortcut`, and `copies` refuses any that do not hold the
    same first spin.
- **Side by side (12_6 to 12_7).** The lines go under each tally:
  - ten marks: "− 20b lands on 0: s", then 5 × 5 − 4 × 5 = 25 − 20 = 5;
  - eleven marks: "− 20b lands on 2b: s + 2b", then 5 × 2 − 4 × 6 = 10 −
    24 = −14 → 8.

  Every secret makes 8 with exactly one second spin.
- **What the leak means (12_8).** The table counts, on ten marks, what each
  pair's two shares always leave, whatever shares they hold:
  - 1 secret: pairs 2-5 and 4-5;
  - 2 secrets: pairs 1-5 and 3-5;
  - 5 secrets: pairs 1-2, 1-4, 2-3, 2-4 and 3-4;
  - all 10: pair 1-3.

  So nine pairs "can always cross off some secrets", two "always find it",
  and only 1 and 3 learn nothing. The note says "what is left fits equally
  well" (a likelihood, never "equally likely"). The eleven-mark line under
  it reads "every pair: all 11". `wait_until(read)` and a 0.8 s hold
  follow.
- **Collisions versus a permutation (13_1 to 13_5).**
  - Ten-mark side: the red face's needle turns 5 marks and 5 more and lands
    on 0 (2 × 5 = 10 → 0). Arrows from "mark" to "5 × mark" land only on 0
    and 5; the counts read 5 under 0 and 5 under 5, then "reaches 2 of 10
    marks", then a 1.5 s hold (`HOLD_ON_TEN`).
  - Consequence for one card: chapter 08's straight line on ten marks
    deals 9, 3, 7, 1, 5. Trustee 5's card fits only secrets 0 and 5.
  - Eleven-mark side: the same rows sit lower right, every mark under the
    same mark, and the question comes with no arrow drawn. Arrows 1 → 5,
    2 → 10 and 3 → 15 → 4 (with "15 → 4") come on their phrases. The rest
    (4 to 10, then 0) grow together while the narration says "Each step
    lands five further round the dial", up to "every mark is reached
    exactly once". Their run time is clamped to 2.4–4.0 s, and the eleven
    numbers are never listed aloud. The counts read 1 under every mark,
    then "reaches 11 of 11 marks". Trustee 5's card fits all 11, then a
    2.0 s hold (`HOLD_ON_BOTH`).
- **Prime only after both (13_6 to 13_7).** The cards read "10 = 2 × 5",
  "on 10 marks, 2 × 5 = 10 lands on 0", "11 = 1 × 11: no two numbers from 2
  to 10 multiply to 11" and "on 11 marks, 2 × 5 = 10 lands on 10". "It's
  prime" is outlined. The five-line proof then follows: unique factorization
  means 11 divides a product of marks only if it divides a factor, and no
  mark from 1 to 10 has it. Its lines appear on their phrases with
  sequential reading deadlines (`read_on`, 15 chars/s), then
  `wait_until(read)`.
- **Every multiplier (13_8).**
  - The lemma moves up and the same-mark argument follows: "5 × u and 5 ×
    w land on the same mark", "so 5 × (u − w) lands on 0", "5 isn't 0, so
    u − w lands on 0:", "u and w are the same mark".
  - The eleven-mark reach grid for ×1 to ×10 comes in, every row full.
  - Then the ten-mark grid, full only at ×1, ×3, ×7 and ×9, then a 1.5 s
    hold (`HOLD_ON_GRIDS`).
- **Card (13_9).** It reads: "the counting holds on any prime number of
  marks above both the number of trustees and the secret (here: 7, 11, 13
  and on; Shamir 1979)". It says "holds", never "needs" or "only", and
  nothing about security. A composite dial is never said to reveal
  everything; the 12- and 49-mark controls show why. `wait_until(read)`
  then a 0.6 s hold.

**Authored pause intentions.** There are 20 new overrides in
`docs/pause-overrides.json` (12_1 ×1, 12_3 ×2, 12_4 ×2, 12_5 ×2, 12_6, 12_7,
12_8, 13_1, 13_2, 13_3, 13_5 ×2, 13_6 ×2, 13_7 ×2, 13_8), each with a
reason. They are proposals, not measured or heard. The two think holds
come from the storyboard's predict rows.

### The integration patch

`docs/previous/ch1213-evidence-20261024/integration-12-13.patch`,
db5574e3f4eba7548c36837a01ae111d4a4ca5a90bc1572bdbd974d2205a6c8a, 1097
lines, 11 files. Apply it in this order, after root's 08/09 integration (=
live):
1. `rebased-integration-10-11.patch` (503c9a52…), or root's own rebase of
   f865d6bf;
2. this patch.

Tested on a fresh copy of live:
- the rebased 10/11 patch applies cleanly and gives exactly my base tree;
- this patch then applies cleanly and gives exactly my work tree (both by
  `diff -rq`, apart from the new evidence folder and the unpatched
  modules);
- applied alone to live, this patch is refused: 17 hunks fail.

If root rebases 10/11 differently, the hunks most likely to need a hand
merge are `pause-overrides.json`'s comment line and its tail (the 20
entries are in `add_overrides_1213.py`'s input form in the work tree, and
the same objects are in the patch).

| File | live | after rebased 10/11 (base) | after this patch |
|---|---|---|---|
| `src/chapters/mod.rs` | a677310e | d1e3e70c | 64f0b437 |
| `src/model.rs` | 16e4bbce | 49c4b87a | 67886a74 |
| `src/main.rs` | eab7d9e0 | 2a2a222a | 63059eea |
| `src/facts.rs` | 74685abe | c2837471 | 52b4d1e3 |
| `oracle.py` | 29af2801 | 005d2491 | d1c1ce7e |
| `docs/storyboard.md` | 84b6ef3a | 8426f4e9 | 98ccbc98 |
| `docs/storyboard-draft.md` | 5949fce2 | 123179f9 | 5f0c8d22 |
| `docs/script-draft.tsv` | 0aee3711 | a8a917b9 | b2a1a17c |
| `script.tsv` | 7a8f5431 | be6a1554 | 2918d4f1 |
| `docs/pause-overrides.json` | 7da9744b | f13918c5 | 7e680c51 |
| `docs/brief.md` | 1b75b7a8 | 1b75b7a8 | 91734f69 |

The hunks, file by file:
- **`src/chapters/mod.rs`.** `pub mod a_ten_mark_dial;` and `pub mod
  why_prime;`, the two `Chapter`s at intensity 0.5, and the doc says
  "chapters 01 to 13 so far. The other ten …".
- **`src/model.rs`.** `what_shares_leave(dial, xs, spins)`. It deals every
  recipe once and, for each tuple of shares, gives how many secrets fit
  and whether they fit equally often. The patch also adds the
  `std::collections::{BTreeMap, BTreeSet}` import. Everything else 12/13
  uses (`pair_shortcut`, `fitting_spins`, `zero_product`, `times_row`,
  `shuffles`, `is_prime`, `TEN_HOLDER`, `LINE_SLOPE`) is already in the
  base.
- **`src/facts.rs` and `oracle.py`.** 46 new S-F8 facts. The golden table
  and `f.len()` go from 262 to 308, and `facts.rs` imports `BTreeMap` too.
  Examples:
  - pair ways, 0,…,10,… and 1 × 11;
  - copies 5,4 and 4,5;
  - scaled 5,20,80 and 4,20,100;
  - spin terms −20,0,1;
  - −20 lands on 0 on ten marks and on 2 on eleven;
  - formula terms, products and values 5 and −14 → 8;
  - `second_spin_each` 1;
  - `pairs_left.1/.2/.5/.10`, find 2, learns nothing 1,3, even-left 10;
  - the zero product 2,5 → 0;
  - ×5 rows, hits and reached;
  - steps 1,5 · 2,10 · 3,15,4;
  - line candidates 0,5 and 11;
  - split 10,2,5 and factors 1,11;
  - reach 10,5,10,5,2,5,10,5,10 and 11 × 10;
  - shuffling rows 1,3,7,9 and 10;
  - primes 7,11,13.

  `oracle.py` adds a 12/13 self-test block:
  - the ten-mark shortcut for every card pair;
  - −20 vanishes only on ten marks of 7 to 13;
  - pairs learn nothing on 7 and 13 marks but leak on 9 and 12;
  - zero products on 7, 11 and 13 versus 10 and 12;
  - ×5 shuffles 12 marks.
- **`src/main.rs`.** 89 `SPOKEN` rows and 13 `NOT_A_COUNT` rows for
  12_1–13_9 (195 → 284 and 9 → 22).
- **`docs/storyboard-draft.md`.**
  - The 12 and 13 sections, from `sections-12-13.md` (d7025094…); 12 goes
    from 5 to 8 rows and 13 from 7 to 9.
  - The vocabulary's "prime" note moves to 13_6.
  - 17_5's picture reference changes from "13_4" to "13_5".
- **`docs/storyboard.md`.** "chapters 01 to 13", the "Devices in chapters 12
  and 13" note (`devices-12-13.md`) and the same two sections, appended
  verbatim.
- **`docs/script-draft.tsv`.** The 12_/13_ lines (12 → 17) as the new draft
  generates them; nothing else.
- **`script.tsv`.** Regenerated by `tools/storyboard.py
  shamir_secret_sharing`: 72 + 17 = 89 lines.
- **`docs/pause-overrides.json`.** The 20 overrides, in root's indented
  format, and the comment "chapters06–13".
- **`docs/brief.md`.** Two line references: "Draft line 13_9 rests on both"
  and "prime (13_6, 13_7, 13_9)".

### Checks that actually ran (Python 3.15 `-I -B`, rustfmt; no Cargo)

All ran on the scratch work tree (live + rebased 10/11 + 12/13, both
modules in place), with the tools in
`docs/previous/ch1213-evidence-20261024/`:
- **`oracle.py --self-test`**: rc 0; stderr "# self-test: ten-mark dial
  pairs that leak: 9 of 10" and "# 308 facts, 0 failed cross-checks". The
  base gives 262 facts, 0 failed.
- **`check_golden.py`**: 308 golden rows (308 distinct), 308 oracle rows,
  `f.len()` asserted 308; diff none.
- **`check_spoken.py`**: 284 SPOKEN, 22 NOT_A_COUNT. Every number word in
  12_/13_ (17 lines, 188 words) and in all 89 lines (549 words) is covered.
  The planted control was caught. 0 failures.
- **`check_cues.py 12:a_ten_mark_dial 13:why_prime`**: says in order; 47
  and 45 cue uses (two and four cue loops) all occur once, in order, and
  equal the tests' lists. The planted swapped loop and repeated phrase
  were caught. 0 failures.
- **`check_overrides.py`**: 49 overrides resolved, 0 errors; the planted
  ambiguous phrases were caught.
- **`check_sb.py`**:
  - production = base's 11 chapters + 12 and 13, each equal to the draft;
  - only 12/13 changed in the draft, plus 17_5's reference;
  - the script lines follow the draft (only 04_3 differs, and it differs
    in the base too);
  - `script.tsv` is the base plus 17 lines;
  - `brief.md` changed in its two references only.
- **`tools/storyboard.py --check shamir_secret_sharing`** (the copied
  tool): rc 0. Advisory estimates are 12: 143.0 s (8 beats) and 13:
  156.8 s (9 beats), 19.2 min over 13 chapters. They are not measured and
  are not a cap.
- **`layout_mirror.py`**: 68 + 90 layout assertions mirrored from the two
  tests, constants parsed from the Rust, 0 failed. The card wraps to two
  lines at 92 down to 72. The planted control (the grid a row lower) was
  caught.
- **`evidence.py`**: 5091 checks, 0 failed. It independently enumerates
  every S-F8 fact and every test-expected literal (read from the test
  sources), plus "seven marks would already do" (every pair leaves all 7,
  any trio leaves 1). Its planted controls all fire: six marks leak,
  twelve keeps 4b, and a wrong fact, a wrong string and a missing
  expectation are each caught.
- **`lint_mirror.py`** (a text heuristic, not rustc/Clippy): 0 findings for
  both modules and for the 16 root-built modules. It reports one finding
  for frozen `one_more_share.rs` (Timed, above); its planted controls are
  caught.
- **`compare_1011.py`**: the rebased 10/11 patch equals the frozen one as
  described above.
- **`rustfmt --check --config-path rustfmt.toml src/main.rs`** on the work
  tree (every registered module, both new ones included): rc 0.
- **Apply tests** as above. A public-hygiene scan found no home paths,
  user names or scratch paths in the new files.

### Not verified and open risks

- Nothing was compiled, so type errors, borrow errors and rustc or Clippy
  warnings beyond the text heuristics are possible.
  - The `Timed` miss shows that a careful reading misses things.
  - `fmn::animation::grow_arrow` and `grow_from_edge` are used as in other
    corpus chapters but have never run in this crate.
  - 12/13 depends on the 10/11 modules compiling: see the
    `one_more_share.rs` defect above.
- No frame was drawn or inspected. The layout is checked only by the
  mirrored generous text-width estimates. The glyphs ×, −, → and "'"
  appear in new contexts and were not probed.
- Timing:
  - Chapter estimates are advisory.
  - The 13_5 arrows' run time is clamped to 2.4–4.0 s against an unmeasured
    narration time.
  - The think and reading holds rest on the scratch voice's timing until a
    narrated draft exists.
  - 11_6's non-sequential holds (frozen 10/11) are unchanged.
- No listening of any kind, and no claim about accent or pacing as heard.

### Evidence files (docs/previous/ch1213-evidence-20261024/, gitignored)

| File | sha256 (first 8) |
|---|---|
| `integration-12-13.patch` | db5574e3 |
| `rebased-integration-10-11.patch` | 503c9a52 |
| `proposed-one_more_share-timed.patch` | 8398d680 |
| `check_cues.py` | 13c06386 |
| `check_golden.py` | 3eecb0d5 |
| `check_spoken.py` | 1f428dcf |
| `check_overrides.py` | 5c39d678 |
| `check_sb.py` | 23c72fc5 |
| `layout_mirror.py` | 805faefc |
| `evidence.py` | 0e00b376 |
| `lint_mirror.py` | c9569b0e |
| `compare_1011.py` | 8b974602 |
| `make_patch.py` | d8d8c221 |
| `apply_sb.py` | fb1e20a5 |
| `rebase_1011.py` | 0bb2b897 |
| `add_overrides_1213.py` | c55a302c |
| `sections-12-13.md` | d7025094 |
| `devices-12-13.md` | 12454448 |

To extract the patch and apply it, after the 10/11 patch:

~~~sh
awk '/^<!-- shamir-1213 patch end -->$/{p=0} p; /^<!-- shamir-1213 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-1213.patch
git apply --check shamir-1213.patch && git apply shamir-1213.patch
~~~

The same bytes are in
`docs/previous/ch1213-evidence-20261024/integration-12-13.patch`.

<!-- shamir-1213 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,12 +1,13 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 11 so far. The other
-//! twelve are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 13 so far. The other
+//! ten are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
 pub mod a_curve_for_three;
 pub mod a_line_through_the_secret;
+pub mod a_ten_mark_dial;
 pub mod an_experiment;
 pub mod board;
 pub mod cutting_it_up;
@@ -22,6 +23,7 @@
 pub mod the_vault;
 pub mod two_keys;
 pub mod two_shares;
+pub mod why_prime;
 
 use explainer_kit::Chapter;
 
@@ -82,5 +84,15 @@
             make: |kit| Box::new(one_more_share::OneMoreShare { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "12_a_ten_mark_dial",
+            make: |kit| Box::new(a_ten_mark_dial::ATenMarkDial { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "13_why_prime",
+            make: |kit| Box::new(why_prime::WhyPrime { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/model.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/model.rs
@@ -7,6 +7,7 @@
 //! turns it into the video's numbers; `oracle.py` recomputes them another
 //! way (the definitions written out, inverses and rebuilding by search).
 
+use std::collections::{BTreeMap, BTreeSet};
 use std::fmt;
 
 /// The vault's dial: eleven marks, 0 to 10.
@@ -578,6 +579,24 @@
     true
 }
 
+/// For every tuple of shares some recipe with `spins` fair spins deals the
+/// trustees at `xs`: how many secrets still fit it, and whether those
+/// secrets all fit it the same number of ways (chapter 12's table). Every
+/// recipe is dealt once and counted under what it hands them.
+pub fn what_shares_leave(dial: Dial, xs: &[u32], spins: usize) -> BTreeSet<(usize, bool)> {
+    let mut ways: BTreeMap<Vec<u32>, Vec<usize>> = BTreeMap::new();
+    for t in tuples(dial, spins + 1) {
+        let dealt = Recipe::new(t[0], &t[1..]).deal(dial, xs);
+        ways.entry(dealt).or_insert_with(|| vec![0; dial.marks() as usize])[t[0] as usize] += 1;
+    }
+    ways.values()
+        .map(|per_secret| {
+            let fits: Vec<usize> = per_secret.iter().copied().filter(|&w| w > 0).collect();
+            (fits.len(), fits.windows(2).all(|w| w[0] == w[1]))
+        })
+        .collect()
+}
+
 /// A shortcut for two trustees: whole-number weights u and w, each between
 /// −(marks − 1) and marks − 1, with u times the first share plus w times the
 /// second landing on the secret for every recipe of two spins. The smallest
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -356,6 +356,202 @@
             "S-F7.all_pairs.third_leaves",
             "3,1",
         ),
+        ("12_1", "have eleven marks", "S-F1.marks", "11"),
+        ("12_1", "ten would seem", "S-F8.ten.marks", "10"),
+        ("12_1", "the same two spins", "S-F6.spin_count", "2"),
+        ("12_1", "on a ten-mark dial", "S-F8.ten.marks", "10"),
+        ("12_1", "trustees four and five", "S-F1.pair", "4,5"),
+        ("12_2", "On ten marks", "S-F8.ten.marks", "10"),
+        (
+            "12_2",
+            "nine, seven, nine, five and five",
+            "S-F8.ten.shares",
+            "9,7,9,5,5",
+        ),
+        ("12_2", "trustees four and five", "S-F1.pair", "4,5"),
+        ("12_2", "hold five and five", "S-F8.ten.pair_shares", "5,5"),
+        ("12_2", "not two and six", "S-F1.pair_shares", "2,6"),
+        ("12_3", "those two cards", "S-F7.pair.count", "2"),
+        ("12_3", "On eleven marks", "S-F1.marks", "11"),
+        ("12_3", "two and six fitted", "S-F1.pair_shares", "2,6"),
+        ("12_3", "exactly one way", "S-F7.ways_each", "1"),
+        ("12_3", "On ten marks", "S-F8.ten.marks", "10"),
+        ("12_3", "five and five come", "S-F8.ten.pair_shares", "5,5"),
+        ("12_3", "from ten spin pairs", "S-F8.ten.pair_spin_pairs", "10"),
+        ("12_3", "has the secret five", "S-F8.ten.pair_secrets", "5"),
+        ("12_3", "These two", "S-F7.pair.count", "2"),
+        // the secret's, the first spin's and the second spin's amounts
+        (
+            "12_4",
+            "Trustee four's is the secret, plus four of the first spin, plus sixteen of the second",
+            "S-F8.pair_sum.height.4",
+            "1,4,16",
+        ),
+        (
+            "12_4",
+            "Trustee five's is the secret, plus five of the first, plus twenty-five of the second",
+            "S-F8.pair_sum.height.5",
+            "1,5,25",
+        ),
+        (
+            "12_5",
+            "Five copies of trustee four's height",
+            "S-F8.pair_sum.copies.4",
+            "5,4",
+        ),
+        ("12_5", "hold twenty", "S-F8.pair_sum.first_spin", "20"),
+        ("12_5", "four copies of trustee five's", "S-F8.pair_sum.copies.5", "4,5"),
+        ("12_5", "the first spin cancels", "S-F8.pair_sum.first_spin_left", "0"),
+        // the second spin's, the first spin's and the secret's amounts left
+        (
+            "12_5",
+            "the secret, minus twenty of the second spin",
+            "S-F8.pair_sum.spin_terms",
+            "-20,0,1",
+        ),
+        ("12_6", "On ten marks", "S-F8.ten.marks", "10"),
+        (
+            "12_6",
+            "twenty of anything lands on zero",
+            "S-F8.ten.twenty_lands",
+            "20,0",
+        ),
+        ("12_6", "the second spin drops out too", "S-F8.ten.leftover", "0"),
+        (
+            "12_6",
+            "five times five, minus four times five",
+            "S-F8.ten.pair_formula_terms",
+            "5,5,-4,5",
+        ),
+        (
+            "12_6",
+            "is twenty-five minus twenty",
+            "S-F8.ten.pair_formula_products",
+            "25,-20",
+        ),
+        ("12_6", "which is five", "S-F8.ten.pair_formula_value", "5"),
+        ("12_7", "On eleven marks", "S-F1.marks", "11"),
+        (
+            "12_7",
+            "minus twenty lands on two",
+            "S-F8.eleven.minus_twenty_lands",
+            "-20,2",
+        ),
+        (
+            "12_7",
+            "the secret plus twice the second spin",
+            "S-F8.eleven.leftover",
+            "2",
+        ),
+        ("12_7", "their two and six", "S-F1.pair_shares", "2,6"),
+        (
+            "12_7",
+            "five times two, minus four times six",
+            "S-F8.eleven.pair_formula_terms",
+            "5,2,-4,6",
+        ),
+        ("12_7", "is minus fourteen", "S-F8.eleven.pair_formula_unwrapped", "-14"),
+        ("12_7", "which lands on eight", "S-F8.eleven.pair_formula_value", "8"),
+        ("12_7", "makes eight", "S-F8.eleven.pair_formula_value", "8"),
+        ("12_7", "exactly one second spin", "S-F8.eleven.second_spin_each", "1"),
+        ("12_7", "so eight fits them all", "S-F8.eleven.pair_formula_value", "8"),
+        ("12_8", "On ten marks", "S-F8.ten.marks", "10"),
+        ("12_8", "nine of the", "S-F8.ten.pairs_that_leak", "9"),
+        ("12_8", "ten pairs of trustees", "S-F7.all_pairs.pairs", "10"),
+        ("12_8", "two pairs always find it", "S-F8.ten.pairs_that_find", "2"),
+        (
+            "12_8",
+            "trustees one and three",
+            "S-F8.ten.pair_that_learns_nothing",
+            "1,3",
+        ),
+        ("13_1", "On ten marks", "S-F8.ten.marks", "10"),
+        ("13_1", "Two times five is ten", "S-F8.ten.zero_product_sum", "2,5,10"),
+        ("13_1", "which lands on zero", "S-F8.ten.zero_product_mark", "0"),
+        ("13_2", "multiplying by five", "S-F8.multiplier", "5"),
+        ("13_2", "On ten marks", "S-F8.ten.marks", "10"),
+        ("13_2", "five times any mark", "S-F8.multiplier", "5"),
+        ("13_2", "lands on zero or five", "S-F8.ten.times5_values", "0,5"),
+        ("13_2", "the ten marks crowd onto two", "S-F8.ten.times5_reached", "2"),
+        ("13_3", "On ten marks", "S-F8.ten.marks", "10"),
+        (
+            "13_3",
+            "nine, three, seven, one and five",
+            "S-F8.line.shares",
+            "9,3,7,1,5",
+        ),
+        ("13_3", "Trustee five's share", "S-F8.line.holder", "5"),
+        ("13_3", "the secret plus five times the spin", "S-F8.line.holder", "5"),
+        (
+            "13_3",
+            "five times the spin is zero or five",
+            "S-F8.ten.times5_values",
+            "0,5",
+        ),
+        ("13_3", "leaves two possible secrets", "S-F8.line.candidate_count", "2"),
+        ("13_3", "here zero and five", "S-F8.line.candidates", "0,5"),
+        ("13_4", "on eleven marks", "S-F1.marks", "11"),
+        ("13_4", "every mark by five", "S-F8.multiplier", "5"),
+        ("13_5", "All eleven", "S-F8.eleven.times5_reached", "11"),
+        ("13_5", "One goes to five", "S-F8.eleven.times5_step.1", "1,5"),
+        ("13_5", "two to ten", "S-F8.eleven.times5_step.2", "2,10"),
+        (
+            "13_5",
+            "three to fifteen, which wraps round to four",
+            "S-F8.eleven.times5_step.3",
+            "3,15,4",
+        ),
+        ("13_5", "lands five further", "S-F8.multiplier", "5"),
+        ("13_5", "reached exactly once", "S-F8.eleven.times5_each", "1"),
+        ("13_5", "on eleven marks", "S-F1.marks", "11"),
+        ("13_5", "trustee five's share", "S-F8.line.holder", "5"),
+        ("13_6", "Ten is two times five", "S-F8.ten.split", "10,2,5"),
+        (
+            "13_6",
+            "two and five can each bring part of the ten",
+            "S-F8.ten.split",
+            "10,2,5",
+        ),
+        ("13_6", "their product lands on zero", "S-F8.ten.zero_product_mark", "0"),
+        ("13_6", "Eleven doesn't split", "S-F1.marks", "11"),
+        (
+            "13_6",
+            "its only factors are one and itself",
+            "S-F8.eleven.factors",
+            "1,11",
+        ),
+        ("13_7", "a multiple of eleven", "S-F1.marks", "11"),
+        ("13_7", "has eleven among its primes", "S-F1.marks", "11"),
+        ("13_7", "from one to ten", "S-F8.eleven.nonzero_range", "1,10"),
+        ("13_7", "on eleven marks", "S-F1.marks", "11"),
+        (
+            "13_8",
+            "multiplying by five, five times their difference",
+            "S-F8.multiplier",
+            "5",
+        ),
+        (
+            "13_8",
+            "every multiplier from one to ten",
+            "S-F8.eleven.nonzero_range",
+            "1,10",
+        ),
+        ("13_8", "the eleven marks", "S-F1.marks", "11"),
+        ("13_8", "On ten marks", "S-F8.ten.marks", "10"),
+        (
+            "13_8",
+            "only one, three, seven and nine",
+            "S-F8.ten.rows_that_shuffle",
+            "1,3,7,9",
+        ),
+        ("13_9", "a position besides zero", "S-F4.secret_position", "0"),
+        ("13_9", "For five trustees", "S-F1.trustees", "5"),
+        (
+            "13_9",
+            "seven marks would already do",
+            "S-F8.smallest_prime_for_five",
+            "7",
+        ),
     ];
 
     /// Number words said as an article, not a count, each with its line.
@@ -369,6 +565,19 @@
         ("09_7", "the secret plus spins times powers"), // the rule's shape, no count
         ("10_4", "Each one is a dealing"),
         ("11_3", "every one of the"),
+        ("12_3", "every one of them"),
+        ("12_5", "rid of the first spin"), // the spin's name, as in "the first spin cancels"
+        ("12_5", "Take one from the other"),
+        ("13_1", "that aren't zero can multiply to zero"), // the question, for any marks
+        ("13_7", "Landing on zero means"),                 // the definition
+        ("13_7", "in only one way"),                       // the theorem, for every number
+        ("13_7", "only if one of the marks does"),
+        ("13_7", "lands on zero only when one of its factors is zero"),
+        ("13_8", "if two marks gave the same answer"), // any two marks
+        ("13_8", "would land on zero"),
+        ("13_8", "each one only shuffles"),
+        ("13_9", "each one has"),
+        ("13_9", "the secret is one of its marks"),
     ];
 
     const NUMBER_WORDS: &[&str] = &[
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -6,11 +6,11 @@
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
 //! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper, S-F7.join and
 //! S-F7.all_pairs 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11
-//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 11 are
+//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 13 are
 //! implemented so far; the rest are computed and checked now so the draft's
 //! numbers are settled before its scenes exist.
 
-use std::collections::BTreeSet;
+use std::collections::{BTreeMap, BTreeSet};
 
 use explainer_kit::Facts;
 
@@ -537,6 +537,139 @@
     f.add("S-F8.line.parity_candidates", csv(&candidates(TEN_PARITY_HOLDER)))?;
     let leaky_marks: Vec<u32> = (1..TEN).filter(|&x| !ten.shuffles(x)).collect();
     f.add("S-F8.line.leaky_marks", csv(&leaky_marks))?;
+    // 12 (native): the pair's tally on each dial
+    let tally = |d: Dial, points: &[(u32, u32)]| -> Vec<usize> {
+        d.all().map(|s| model::fitting_spins(d, points, s, 2).len()).collect()
+    };
+    let ten_ways = tally(ten, &ten_pair);
+    f.add("S-F8.ten.pair_ways", csv(&ten_ways))?;
+    f.add("S-F8.eleven.pair_ways", csv(&tally(dial, &pair_points)))?;
+    f.add("S-F8.ten.pair_spin_pairs", ten_ways.iter().sum::<usize>())?;
+    // 12_4, 12_5: each height's amounts of the secret and the two spins, and
+    // the shortcut's weights as copies, which hold the same amount of the first
+    let amounts = PAIR.map(|x| [1, i64::from(x), i64::from(x * x)]);
+    let copies = [su, -sw];
+    for (x, a) in PAIR.iter().zip(&amounts) {
+        f.add(&format!("S-F8.pair_sum.height.{x}"), csv(a))?;
+    }
+    for (&x, c) in PAIR.iter().zip(copies) {
+        f.add(&format!("S-F8.pair_sum.copies.{x}"), csv(&[c, i64::from(x)]))?;
+    }
+    let scaled = [0, 1].map(|i| amounts[i].map(|t| copies[i] * t));
+    f.add("S-F8.pair_sum.first_spin", scaled[0][1])?;
+    for (x, s) in PAIR.iter().zip(&scaled) {
+        f.add(&format!("S-F8.pair_sum.scaled.{x}"), csv(s))?;
+    }
+    f.add("S-F8.pair_sum.first_spin_left", scaled[0][1] - scaled[1][1])?;
+    // 12_6, 12_7: the −20 on each dial, and the pair's sum term by term
+    let twenty = terms[0];
+    f.add("S-F8.ten.twenty_lands", csv(&[-twenty, i64::from(ten.wrap(-twenty))]))?;
+    let sum_terms = |y: [u32; 2]| [su, i64::from(y[0]), sw, i64::from(y[1])];
+    let products = |y: [u32; 2]| [su * i64::from(y[0]), sw * i64::from(y[1])];
+    f.add("S-F8.ten.pair_formula_terms", csv(&sum_terms(ten_pair_values)))?;
+    f.add("S-F8.ten.pair_formula_products", csv(&products(ten_pair_values)))?;
+    f.add(
+        "S-F8.eleven.minus_twenty_lands",
+        csv(&[twenty, i64::from(dial.wrap(twenty))]),
+    )?;
+    f.add("S-F8.eleven.pair_formula_terms", csv(&sum_terms(pair_values)))?;
+    f.add("S-F8.eleven.pair_formula_products", csv(&products(pair_values)))?;
+    let eight = dial.wrap(applied(pair_values));
+    let second_spins = dial.all().map(|s| {
+        dial.all()
+            .filter(|&b| dial.add(s, dial.mul(dial.wrap(twenty), b)) == eight)
+            .count()
+    });
+    f.add("S-F8.eleven.second_spin_each", set_csv(second_spins))?;
+    // 12_8: what each pair's two shares leave on ten marks, by dealing every recipe
+    let ten_pairs = combinations(&TRUSTEES, 2);
+    let mut by_left: BTreeMap<usize, Vec<String>> = BTreeMap::new();
+    let mut even_left = 0;
+    for pair in &ten_pairs {
+        let leaves = model::what_shares_leave(ten, pair, 2);
+        let counts: Vec<usize> = leaves
+            .iter()
+            .map(|&(n, _)| n)
+            .collect::<BTreeSet<_>>()
+            .into_iter()
+            .collect();
+        let [n] = counts[..] else {
+            return Err(format!("ten marks: {pair:?}'s shares leave {counts:?} secrets"));
+        };
+        by_left.entry(n).or_default().push(format!("{}-{}", pair[0], pair[1]));
+        even_left += usize::from(leaves.iter().all(|&(_, even)| even));
+    }
+    for (n, names) in &by_left {
+        f.add(&format!("S-F8.ten.pairs_left.{n}"), names.join(","))?;
+    }
+    f.add("S-F8.ten.pairs_that_find", by_left.get(&1).map_or(0, Vec::len))?;
+    let silent: Vec<&Vec<u32>> = ten_pairs
+        .iter()
+        .filter(|p| model::learns_nothing(ten, p, 2, model::any_spins))
+        .collect();
+    let [quiet] = silent[..] else {
+        return Err(format!("ten marks: {silent:?} learn nothing, not one pair"));
+    };
+    f.add("S-F8.ten.pair_that_learns_nothing", csv(quiet))?;
+    f.add("S-F8.ten.pairs_even_left", even_left)?;
+
+    // 13 (native): multiplying by five on both dials
+    f.add("S-F8.ten.zero_product_sum", csv(&[z1, z2, z1 * z2]))?;
+    f.add("S-F8.ten.zero_product_mark", ten.mul(z1, z2))?;
+    f.add("S-F8.multiplier", TEN_HOLDER)?;
+    let hits = |d: Dial| -> Vec<usize> {
+        let row = d.times_row(TEN_HOLDER);
+        d.all().map(|m| row.iter().filter(|&&y| y == m).count()).collect()
+    };
+    let (ten_hits, eleven_hits) = (hits(ten), hits(dial));
+    let reached: Vec<u32> = ten.all().filter(|&m| ten_hits[m as usize] > 0).collect();
+    f.add("S-F8.ten.times5_values", csv(&reached))?;
+    f.add("S-F8.ten.times5_reached", reached.len())?;
+    f.add("S-F8.ten.times5_hits", csv(&ten_hits))?;
+    f.add("S-F8.line.candidate_count", candidates(TEN_HOLDER).len())?;
+    f.add(
+        "S-F8.eleven.times5_reached",
+        eleven_hits.iter().filter(|&&h| h > 0).count(),
+    )?;
+    f.add("S-F8.eleven.times5_hits", csv(&eleven_hits))?;
+    f.add("S-F8.eleven.times5_each", set_csv(eleven_hits.iter().copied()))?;
+    for x in 1..=3 {
+        let k = TEN_HOLDER * x;
+        let step = if k < MARKS {
+            vec![x, k]
+        } else {
+            vec![x, k, dial.mul(TEN_HOLDER, x)]
+        };
+        f.add(&format!("S-F8.eleven.times5_step.{x}"), csv(&step))?;
+    }
+    let one_share = View::Shares {
+        points: vec![(TEN_HOLDER, line.share(dial, TEN_HOLDER))],
+        spins: 1,
+    };
+    f.add("S-F8.eleven.line_candidates", one_share.possible(dial).len())?;
+    let splits: Vec<[u32; 3]> = (2..TEN)
+        .flat_map(|a| (a..TEN).map(move |b| [TEN, a, b]))
+        .filter(|&[n, a, b]| a * b == n)
+        .collect();
+    let [split] = splits[..] else {
+        return Err(format!("ten splits as {splits:?}, not one way"));
+    };
+    f.add("S-F8.ten.split", csv(&split))?;
+    let factors: Vec<u32> = (1..=MARKS).filter(|&d| MARKS.is_multiple_of(d)).collect();
+    f.add("S-F8.eleven.factors", csv(&factors))?;
+    f.add("S-F8.eleven.zero_product_check", csv(&[z1, z2, dial.mul(z1, z2)]))?;
+    if let Some(pair) = model::zero_product(dial) {
+        return Err(format!("eleven marks: {pair:?} multiply to zero"));
+    }
+    f.add("S-F8.eleven.nonzero_range", csv(&[1, MARKS - 1]))?;
+    let reach = |d: Dial| -> Vec<usize> { (1..d.marks()).map(|k| distinct(d.times_row(k))).collect() };
+    f.add("S-F8.ten.reach", csv(&reach(ten)))?;
+    f.add("S-F8.eleven.reach", csv(&reach(dial)))?;
+    let primes: Vec<u32> = (TRUSTEES.len() as u32 + 1..)
+        .filter(|&q| Dial::new(q).is_prime())
+        .take(3)
+        .collect();
+    f.add("S-F8.primes_for_five", csv(&primes))?;
 
     // 14, 15: helper curves and three shares (the opening trio)
     let trio_points = dealt(dial, &recipe, &OPENING_TRIO);
@@ -970,6 +1103,52 @@
 S-F8.line.parity_share	3
 S-F8.line.parity_candidates	1,3,5,7,9
 S-F8.line.leaky_marks	2,4,5,6,8
+S-F8.ten.pair_ways	0,0,0,0,0,10,0,0,0,0
+S-F8.eleven.pair_ways	1,1,1,1,1,1,1,1,1,1,1
+S-F8.ten.pair_spin_pairs	10
+S-F8.pair_sum.height.4	1,4,16
+S-F8.pair_sum.height.5	1,5,25
+S-F8.pair_sum.copies.4	5,4
+S-F8.pair_sum.copies.5	4,5
+S-F8.pair_sum.first_spin	20
+S-F8.pair_sum.scaled.4	5,20,80
+S-F8.pair_sum.scaled.5	4,20,100
+S-F8.pair_sum.first_spin_left	0
+S-F8.ten.twenty_lands	20,0
+S-F8.ten.pair_formula_terms	5,5,-4,5
+S-F8.ten.pair_formula_products	25,-20
+S-F8.eleven.minus_twenty_lands	-20,2
+S-F8.eleven.pair_formula_terms	5,2,-4,6
+S-F8.eleven.pair_formula_products	10,-24
+S-F8.eleven.second_spin_each	1
+S-F8.ten.pairs_left.1	2-5,4-5
+S-F8.ten.pairs_left.2	1-5,3-5
+S-F8.ten.pairs_left.5	1-2,1-4,2-3,2-4,3-4
+S-F8.ten.pairs_left.10	1-3
+S-F8.ten.pairs_that_find	2
+S-F8.ten.pair_that_learns_nothing	1,3
+S-F8.ten.pairs_even_left	10
+S-F8.ten.zero_product_sum	2,5,10
+S-F8.ten.zero_product_mark	0
+S-F8.multiplier	5
+S-F8.ten.times5_values	0,5
+S-F8.ten.times5_reached	2
+S-F8.ten.times5_hits	5,0,0,0,0,5,0,0,0,0
+S-F8.line.candidate_count	2
+S-F8.eleven.times5_reached	11
+S-F8.eleven.times5_hits	1,1,1,1,1,1,1,1,1,1,1
+S-F8.eleven.times5_each	1
+S-F8.eleven.times5_step.1	1,5
+S-F8.eleven.times5_step.2	2,10
+S-F8.eleven.times5_step.3	3,15,4
+S-F8.eleven.line_candidates	11
+S-F8.ten.split	10,2,5
+S-F8.eleven.factors	1,11
+S-F8.eleven.zero_product_check	2,5,10
+S-F8.eleven.nonzero_range	1,10
+S-F8.ten.reach	10,5,10,5,2,5,10,5,10
+S-F8.eleven.reach	11,11,11,11,11,11,11,11,11,11
+S-F8.primes_for_five	7,11,13
 S-F9.helper_at_own	2,-1,2
 S-F9.helper_at_zero	6,3,2
 S-F9.weights	3,-3,1
@@ -1058,7 +1237,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 262);
+        assert_eq!(f.len(), 308);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -574,6 +574,87 @@
     leaky = [x for x in range(1, TEN) if len({(a * x) % TEN for a in range(TEN)}) < TEN]
     put("S-F8.line.leaky_marks", csv(leaky))
 
+    # 12 (native): the pair's two tallies, by trying every spin pair for every secret
+    ten_ways = [len(fitting_spins(PAIR, tpv, s, TEN)) for s in range(TEN)]
+    eleven_ways = [len(fitting_spins(PAIR, pv, s)) for s in range(P)]
+    put("S-F8.ten.pair_ways", csv(ten_ways))
+    put("S-F8.eleven.pair_ways", csv(eleven_ways))
+    put("S-F8.ten.pair_spin_pairs", sum(ten_ways))
+    # 12_4, 12_5: each height's amounts of the secret, the first spin and the second; the
+    # fewest copies of each that hold the same amount of the first spin, found by search
+    amounts = {x: (1, x, x * x) for x in PAIR}
+    for x in PAIR:
+        put(f"S-F8.pair_sum.height.{x}", csv(amounts[x]))
+    copies = next((c4, c5) for n in range(2, 99) for c4 in range(1, n) for c5 in [n - c4]
+                  if c4 * amounts[PAIR[0]][1] == c5 * amounts[PAIR[1]][1])
+    put(f"S-F8.pair_sum.copies.{PAIR[0]}", csv((copies[0], PAIR[0])))
+    put(f"S-F8.pair_sum.copies.{PAIR[1]}", csv((copies[1], PAIR[1])))
+    scaled = {x: [c * t for t in amounts[x]] for c, x in zip(copies, PAIR)}
+    put("S-F8.pair_sum.first_spin", scaled[PAIR[0]][1])
+    for x in PAIR:
+        put(f"S-F8.pair_sum.scaled.{x}", csv(scaled[x]))
+    left = [u - w for u, w in zip(scaled[PAIR[0]], scaled[PAIR[1]])]
+    check(left == [1, 0, -20], f"the difference of the copies is s - 20b: {left}")
+    put("S-F8.pair_sum.first_spin_left", left[1])
+    # 12_6, 12_7: the -20 wound onto each dial, and the sum on the pair's cards term by term
+    put("S-F8.ten.twenty_lands", csv((-left[2], -left[2] % TEN)))
+    put("S-F8.ten.pair_formula_terms", csv((copies[0], tpv[0], -copies[1], tpv[1])))
+    put("S-F8.ten.pair_formula_products", csv((copies[0] * tpv[0], -copies[1] * tpv[1])))
+    put("S-F8.eleven.minus_twenty_lands", csv((left[2], left[2] % P)))
+    put("S-F8.eleven.pair_formula_terms", csv((copies[0], pv[0], -copies[1], pv[1])))
+    put("S-F8.eleven.pair_formula_products", csv((copies[0] * pv[0], -copies[1] * pv[1])))
+    eight = (copies[0] * pv[0] - copies[1] * pv[1]) % P
+    put("S-F8.eleven.second_spin_each",
+        csv(sorted({sum(1 for b in range(P) if (s + (left[2] % P) * b) % P == eight) for s in range(P)})))
+    # 12_8: every pair of trustees on ten marks, every pair of shares some recipe deals them
+    # (all 1000 recipes): how many secrets are left, and whether those all fit equally often
+    by_left, even = {}, 0
+    for pair in itertools.combinations(TRUSTEES, 2):
+        dealt_by = {}
+        for r in recipes(TEN):
+            dealt_by.setdefault(tuple(share(r, x, TEN) for x in pair), []).append(r[0])
+        lefts = {len(set(v)) for v in dealt_by.values()}
+        check(len(lefts) == 1, f"ten marks {pair}: what is left depends on the shares: {lefts}")
+        by_left.setdefault(lefts.pop(), []).append(pair)
+        even += all(len({v.count(s) for s in set(v)}) == 1 for v in dealt_by.values())
+    for n, group in sorted(by_left.items()):
+        put(f"S-F8.ten.pairs_left.{n}", csv(f"{a}-{b}" for a, b in group))
+    put("S-F8.ten.pairs_that_find", len(by_left.get(1, [])))
+    nothing = by_left.get(TEN, [])
+    check(len(nothing) == 1, f"one pair learns nothing on ten marks: {nothing}")
+    put("S-F8.ten.pair_that_learns_nothing", csv(nothing[0]))
+    put("S-F8.ten.pairs_even_left", even)
+
+    # 13 (native): multiplying on ten and eleven marks, every product tried
+    zp = [(a, b) for a in range(2, TEN) for b in range(a, TEN) if a * b == TEN]
+    check(zp == [(2, 5)], f"the factor pairs of ten: {zp}")
+    put("S-F8.ten.zero_product_sum", csv((*zp[0], zp[0][0] * zp[0][1])))
+    put("S-F8.ten.zero_product_mark", (zp[0][0] * zp[0][1]) % TEN)
+    put("S-F8.multiplier", TEN_HOLDER)
+    k = TEN_HOLDER
+    ten_hits = [sum(1 for x in range(TEN) if (k * x) % TEN == m) for m in range(TEN)]
+    eleven_hits = [sum(1 for x in range(P) if (k * x) % P == m) for m in range(P)]
+    put("S-F8.ten.times5_values", csv(m for m in range(TEN) if ten_hits[m]))
+    put("S-F8.ten.times5_reached", sum(1 for h in ten_hits if h))
+    put("S-F8.ten.times5_hits", csv(ten_hits))
+    put("S-F8.line.candidate_count", len(ten_candidates(TEN_HOLDER)))
+    put("S-F8.eleven.times5_reached", sum(1 for h in eleven_hits if h))
+    put("S-F8.eleven.times5_hits", csv(eleven_hits))
+    put("S-F8.eleven.times5_each", csv(sorted(set(eleven_hits))))
+    for x in (1, 2, 3):
+        put(f"S-F8.eleven.times5_step.{x}", csv((x, k * x) if k * x < P else (x, k * x, (k * x) % P)))
+    eleven_line = line_share(SECRET, LINE_SLOPE, TEN_HOLDER)
+    put("S-F8.eleven.line_candidates",
+        len({s for s in range(P) for a in range(P) if line_share(s, a, TEN_HOLDER) == eleven_line}))
+    put("S-F8.ten.split", csv((TEN, *zp[0])))
+    put("S-F8.eleven.factors", csv(d for d in range(1, P + 1) if P % d == 0))
+    put("S-F8.eleven.zero_product_check", csv((*zp[0], (zp[0][0] * zp[0][1]) % P)))
+    check(all((a * b) % P for a in range(1, P) for b in range(1, P)), "no two nonzero marks of eleven multiply to 0")
+    put("S-F8.eleven.nonzero_range", csv((1, P - 1)))
+    put("S-F8.ten.reach", csv(len({(m * x) % TEN for x in range(TEN)}) for m in range(1, TEN)))
+    put("S-F8.eleven.reach", csv(len({(m * x) % P for x in range(P)}) for m in range(1, P)))
+    put("S-F8.primes_for_five", csv([q for q in primes if q > len(TRUSTEES)][:3]))
+
     # 14, 15: three shares rebuild it (the opening trio)
     tv = [shares[x - 1] for x in OPENING_TRIO]
     put("S-F9.helper_at_own", csv(math.prod(x - y for y in OPENING_TRIO if y != x) for x in OPENING_TRIO))
@@ -883,6 +964,35 @@
             check(paper_curve([(x, tall[x]) for x in trio]) == r, f"paper curve {r} from {trio}")
     nudged = [(1, 19), (2, 47), (3, 90)]
     check(paper_curve(nudged) != (SECRET, *SPINS), "negative control: a nudged height moves the paper curve")
+    # chapter 12: on ten marks, whatever two cards trustees 4 and 5 hold, the only secret
+    # that fits is 5*y4 - 4*y5, ten ways; so a wrong card picks another secret (the control)
+    for y4 in range(TEN):
+        for y5 in range(TEN):
+            ways = [len(fitting_spins(PAIR, (y4, y5), s, TEN)) for s in range(TEN)]
+            want = (5 * y4 - 4 * y5) % TEN
+            check(ways == [TEN if s == want else 0 for s in range(TEN)], f"ten marks, the pair holding {y4},{y5}")
+    check(f["S-F8.ten.pair_secrets"] == "5" and [s for s in range(TEN) if fitting_spins(PAIR, (6, 5), s, TEN)] == [0],
+          "negative control: on ten marks the pair holding 6 and 5 finds 0, not 5")
+    # wrong modulus: the -20 vanishes only on a dial whose marks divide 20; of 7 to 13 marks,
+    # only ten; on twelve it leaves 4b and on eleven 2b
+    check([p for p in range(7, 14) if 20 % p == 0] == [TEN] and -20 % 12 == 4 and -20 % P == 2,
+          "the -20 vanishes on ten marks alone")
+    # chapter 13: on a prime dial with more marks than trustees, every pair learns nothing
+    # (7 and 13 marks); on nine and twelve marks some pairs leak (the controls)
+    for p, leaking in ((7, 0), (13, 0), (9, 4), (12, 9)):
+        got = sum(1 for m in itertools.combinations(TRUSTEES, 2) if not pair_learns_nothing(p, m)[0])
+        check(got == leaking, f"{p} marks: {got} leaking pairs, expected {leaking}")
+    # edge: with only five marks trustee 5 sits at 0 and holds the secret itself
+    check(all(share(r, 5, 5) == r[0] for r in recipes(5)), "on five marks trustee 5's share is the secret")
+    # a product lands on 0 only when a factor is 0, on 7, 11 and 13 marks; on ten and
+    # twelve marks, some nonzero marks multiply to 0 (the control)
+    for p in (7, P, 13):
+        check(all((a * b) % p != 0 for a in range(1, p) for b in range(1, p)), f"{p} marks: no zero products")
+        check(all(len({(m * x) % p for x in range(p)}) == p for m in range(1, p)), f"{p} marks: every row shuffles")
+    for p in (TEN, 12):
+        check(any((a * b) % p == 0 for a in range(1, p) for b in range(1, p)), f"negative control: {p} marks")
+    # a composite dial's rows do not all collapse: on twelve marks, times 5 shuffles too
+    check(len({(5 * x) % 12 for x in range(12)}) == 12, "on twelve marks, times 5 shuffles")
     return len(leaks)
 
 
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 11, per root's direction in Mail
+chapters 01 to 13, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -229,3 +229,60 @@
 | 11_4 | Now let trustee one join them, holding eight. ⟨predict⟩ How many of the eleven curves pass through eight at position one? | At "Now let trustee one join them" the count goes; trustee 1's disc turns teal with a ring [S-F7.join.trustee]. At "holding eight" its card turns over, the back lifting away before 8 drops in, an outline round its frame [S-F7.join.share]; a teal point at (1, 8) on the board with a ring, and the row label 8 glows [S-F7.join.point]. At "pass through eight at position one" a teal outline round the board's column at position 1. Nothing answers yet: the think ring with its "?" at the lower right. | card turn, predict | question hold (think time) |
 | 11_5 | Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one. | The "?" goes. At "Just one" the teal point swells and a teal ring round the secret 5's point there [S-F7.join.count]. At "Each curve passes position one" "each secret's curve at position 1:" under the strip, then secret by secret a white ring round its curve's point at position 1 and that mark under the secret's tick: 5, 10, 4, 9, 3, 8, 2, 7, 1, 6, 0 [S-F7.join.values_by_secret]. At "at a different mark" outlines round the column at position 1 and round the row: eleven different marks [S-F7.join.distinct]. At "only the curve for five passes at eight" its points and guides turn blue and are ringed, its tag glows and the 8 under tick 5 glows teal; every other curve and its gold point go faint and its tag leaves [S-F7.join.secrets_left]. At "The strip drops from eleven secrets to one" discs 1, 4 and 5 turn teal and the strip shows "for: trustees 1, 4 and 5", only 5 lit [S-F7.join.drop]. | live count, highlight | thought after "Just one."; hold on the single curve (1.5 s) |
 | 11_6 | The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one. | The row and its title go. At "The same count holds for every pair of trustees" "every pair of trustees (10 pairs):" under the strip, then the ten pairs, "1 and 2" to "4 and 5", in two rows of five [S-F7.all_pairs.pairs]. At "every two shares they could hold" "each holding any of the 121 pairs of shares:" [S-F7.all_pairs.share_pairs]. At "Two shares leave all eleven secrets" "2 shares: every secret fits exactly 1 spin pair" in purple [S-F7.all_pairs.ways_each, S-F7.all_pairs.two_leave]. At "any third share leaves exactly one" "+ any third share: exactly 1 secret left" in teal, glowing [S-F7.all_pairs.third_leaves]. The scene counts the table itself over every pair, both shares and every third share; oracle.py recounts it by dealing every recipe. The lines stay until they can be read. | table | chapter_transition; hold until the table is read |
+
+## Devices in chapters 12 and 13
+
+- **Two dials side by side** (12_1): the vault's 11-mark face and a red
+  10-mark face, the same secret and spins dealt on both. Over the rail each
+  trustee's height on paper stands above two cards: the red one it lands on
+  with ten marks (its last digit) and the vault's own in the rail's frame.
+- **The tallies** (12_3 to 12_7): chapter 05's tally, once for each dial, one
+  over the other on the strip's columns, so a mark sits in the same place in
+  both. Over each secret's tick a yellow bar counts the spin pairs that deal
+  trustees 4 and 5 their cards with that secret; ticks with no bar go faint.
+  The lines under each tally say what is left and, later, what the pair's
+  sum lands on there.
+- **The derivation** (12_4 and 12_5): the two heights in letters, s for the
+  secret and a and b for the spins, in three columns so that like terms
+  stand under like terms. Five copies and four copies are written out, the
+  two 20a terms are struck through, and the result is "s − 20b".
+- **The ten-mark table** (12_8): what each of the ten pairs' shares leave on
+  ten marks, counted by the scene for every pair and every possible pair of
+  shares, with the eleven-mark line under it.
+- **Times rows** (13_2 to 13_5): chapter 08's doubling rows with five in
+  place of two. Each mark has an arrow down to five times it, and the lower
+  row's marks never move, so the ten-mark rows show arrows crowding onto 0
+  and 5 while the eleven-mark rows show every mark reached once. A count
+  under each mark says how many arrows it gets.
+- **Reach rows** (13_8): on the same columns, one row of cells for each
+  multiplier, lit at every mark it reaches. Every row is full on eleven
+  marks; on ten marks only 1, 3, 7 and 9 are.
+- Emphasis is by rings, outlines and strokes of their own. No typeset text
+  changes color or size.
+
+## 12_a_ten_mark_dial: why not ten?
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 12_1 | Why does the dial have eleven marks? We count in tens, so ten would seem the natural choice. Deal the same secret with the same two spins, but on a ten-mark dial. Would trustees four and five still learn nothing? ⟨predict⟩ | The rail with the vault's five cards face up in their frames, 8, 3, 1, 2 and 6 [S-F6.shares], trustees 4 and 5's discs purple; over the frames each trustee's height on paper, 19, 47, 89, 145 and 215 [S-F6.heights], and right of the rows their names, "height on paper" and "on 11 marks". At "Why does the dial have eleven marks?" the vault's dial face at the upper left, "the vault: 11 marks", its needle on 5 [S-F1.marks, S-F1.secret]; at "ten would seem the natural choice" a red face beside it, "10 marks", 0 to 9 [S-F8.ten.marks]. At "Deal the same secret with the same two spins" an outline round the heights [S-F6.spin_count]; at "on a ten-mark dial" a row of empty red frames between the heights and the rail, named "on 10 marks". At "Would trustees four and five" purple rings round discs 4 and 5 and outlines round both their frames [S-F1.pair]. Nothing answers yet: the think ring with its "?" at the right. | dial, predict | thought after "the natural choice."; question hold (think time) |
+| 12_2 | On ten marks, each height simply lands on its last digit: nine, seven, nine, five and five. So trustees four and five would hold five and five, not two and six. | The "?" goes. At "each height simply lands on its last digit" an outline glows round the heights. From "nine, seven, nine, five and five" each trustee's red card drops into its red frame as it is said, under its height [S-F8.ten.shares]. At "would hold five and five" purple rings round discs 4 and 5 and red outlines round their red frames [S-F8.ten.pair_shares]; at "not two and six" purple outlines round their frames on the rail, still holding the vault's 2 and 6 [S-F1.pair_shares]. | lagged cards, outline glow | connected list (the five shares); clause after "five and five," |
+| 12_3 | Now run the experiment again: for each secret, count the spin pairs that would hand them those two cards. On eleven marks, two and six fitted every secret exactly one way. On ten marks, five and five come from ten spin pairs, and every one of them has the secret five. These two could work out the secret alone. | The faces go. At "run the experiment again" chapter 05's tally at the right, twice, one over the other on the strip's columns: "11 marks: trustees 4 and 5 hold 2 and 6" over ticks 0 to 10, and "10 marks: trustees 4 and 5 hold 5 and 5" over ticks 0 to 9, each mark's tick in the same place in both, "ways" beside each, and over each tick a yellow bar for the spin pairs that deal the pair those cards with that secret, all still empty [S-F7.pair.count]. At "On eleven marks" the upper bars rise together, each one spin pair high with "1" over it, every tick lit, and under them "every secret fits, 1 way each" [S-F7.ways_each, S-F7.fitting, S-F8.eleven.pair_ways]. At "On ten marks" the lower bars come secret by secret, a yellow ring glowing round each tick as it is tried: none at 0 to 4, one ten times as high at 5 with "10" over it, none at 6 to 9; the ticks with no bar go faint, 5 alone lit, and under them "only secret 5 fits: 10 ways" [S-F8.ten.pair_ways, S-F8.ten.pair_spin_pairs, S-F8.ten.pair_secrets]; at "has the secret five" the bar at 5 swells. At "These two could work out the secret alone" purple rings round discs 4 and 5 and a red outline round the lower tick 5. Both tallies are held for comparing. | tally, bars, linked views | thought after "exactly one way."; thought after "the secret five."; hold on the two tallies (1.5 s) |
+| 12_4 | How? Write their heights out. Trustee four's is the secret, plus four of the first spin, plus sixteen of the second. Trustee five's is the secret, plus five of the first, plus twenty-five of the second. | The rail's heights and cards stay. At "Write their heights out" at the upper left, dim, "s: the secret   a: the 1st spin   b: the 2nd spin". At "Trustee four's is" a purple outline glows round trustee 4's height, 145, and "trustee 4:" with "s", "+ 4a" and "+ 16b" coming in as said [S-F8.pair_sum.height.4]; at "Trustee five's is" a purple outline glows round 215 and "trustee 5:" with "s", "+ 5a" and "+ 25b" under it, each term under trustee 4's like term [S-F8.pair_sum.height.5]: three columns, the secret's, the first spin's and the second spin's. | derivation, columns | thought after "Write their heights out."; thought after "sixteen of the second." |
+| 12_5 | To get rid of the first spin, make both hold the same amount of it. Five copies of trustee four's height hold twenty, and so do four copies of trustee five's. Take one from the other, and the first spin cancels. What's left is the secret, minus twenty of the second spin. | At "Five copies of trustee four's height" "5 × trustee 4:" with "5s", "+ 20a" and "+ 80b" under the two rows, term under term [S-F8.pair_sum.copies.4, S-F8.pair_sum.scaled.4]; at "hold twenty" a yellow outline round its "+ 20a" [S-F8.pair_sum.first_spin]; at "four copies of trustee five's" "4 × trustee 5:" with "4s", "+ 20a" and "+ 100b" under it, and a yellow outline round its "+ 20a" [S-F8.pair_sum.copies.5, S-F8.pair_sum.scaled.5]. At "Take one from the other" a rule under them and a "−" left of the second. At "the first spin cancels" a red stroke through both "+ 20a" and a red "0" under the rule in the first spin's column [S-F8.pair_sum.first_spin_left]. At "What's left is the secret" "first − second:" with "s" under 5s − 4s, and at "minus twenty of the second spin" "− 20b" under 80b − 100b [S-F8.pair_sum.spin_terms]. | derivation, strike, outline glow | thought after "the same amount of it."; thought after "the first spin cancels." |
+| 12_6 | On ten marks, twenty of anything lands on zero, so the second spin drops out too and leaves the secret bare. With their cards, five times five, minus four times five, is twenty-five minus twenty, which is five. | At "twenty of anything lands on zero" a red outline round the result's "− 20b" and, under the lower tally, in red, "− 20b lands on 0:" [S-F8.ten.twenty_lands, S-F8.ten.leftover]; at "leaves the secret bare" a red stroke through the result's "− 20b" and a red "s" after "− 20b lands on 0:". At "five times five" on the next line "5 × 5 − 4 × 5" [S-F8.ten.pair_formula, S-F8.ten.pair_formula_terms]; at "is twenty-five minus twenty" "= 25 − 20" [S-F8.ten.pair_formula_products]; at "which is five" "= 5", the lower tally's bar at 5 swells and a red ring round its tick [S-F8.ten.pair_formula_value]. | derivation, linked views | thought after "the secret bare." |
+| 12_7 | On eleven marks, minus twenty lands on two instead, so the same sum is the secret plus twice the second spin. With their two and six, five times two, minus four times six, is minus fourteen, which lands on eight. Every secret makes eight with exactly one second spin, so eight fits them all. | At "minus twenty lands on two" a purple outline round the result's "− 20b" and, under the upper tally, "− 20b lands on 2b:" [S-F8.eleven.minus_twenty_lands]; at "the secret plus twice the second spin" "s + 2b" after it [S-F8.eleven.leftover]. At "five times two" on the next line "5 × 2 − 4 × 6" [S-F8.eleven.pair_formula_terms]; at "is minus fourteen" "= 10 − 24 = −14" [S-F8.eleven.pair_formula_products, S-F8.eleven.pair_formula_unwrapped]; at "which lands on eight" "→ 8" [S-F8.eleven.pair_formula_value]. At "Every secret makes eight" an outline round the upper bars, each still one way high [S-F8.eleven.second_spin_each, S-F7.ways_each]. | derivation, linked views, outline glow | thought after "which lands on eight." |
+| 12_8 | And it isn't only this pair. On ten marks, nine of the ten pairs of trustees can always cross off some secrets, and two pairs always find it. Only trustees one and three learn nothing. | The derivation goes. At "On ten marks" the table at the upper left, "on 10 marks, what each pair's two shares leave" [S-F7.all_pairs.pairs], row by row: "1 secret: 2 and 5, 4 and 5" [S-F8.ten.pairs_left.1], "2 secrets: 1 and 5, 3 and 5" [S-F8.ten.pairs_left.2], "5 secrets: 1 and 2, 1 and 4, 2 and 3, 2 and 4, 3 and 4" [S-F8.ten.pairs_left.5], "all 10: 1 and 3" [S-F8.ten.pairs_left.10], then the note "whatever shares they hold; what is left fits equally well" [S-F8.ten.pairs_even_left] and, under it, "on 11 marks, every pair: all 11" [S-F7.all_pairs.two_leave]. At "nine of the ten pairs" a red outline round the first three rows [S-F8.ten.pairs_that_leak]; at "two pairs always find it" round the first [S-F8.ten.pairs_that_find]; at "Only trustees one and three" a gold outline round the last [S-F8.ten.pair_that_learns_nothing]. The table stays until it can be read. | table, outline glow | thought after "always find it."; chapter_transition; hold until the table is read |
+
+## 13_why_prime: multiplying that never collapses
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 13_1 | What went wrong? On ten marks, marks that aren't zero can multiply to zero. Two times five is ten, which lands on zero. | The rail, its discs and empty frames. At "What went wrong?" the red ten-mark face at the upper left, "10 marks" over it, its needle on 0 [S-F8.ten.marks]. At "marks that aren't zero" purple rings glow round the face's 2 and 5 [S-F8.ten.zero_product]. At "Two times five is ten" "2 × 5 = 10" right of the face, and the needle turns five marks and five more, round past the top to 0 [S-F8.ten.zero_product_sum]; at "which lands on zero" a red "→ 0" after it and a red ring glows round the face's 0 [S-F8.ten.zero_product_mark]. | dial, equation | thought after "multiply to zero." |
+| 13_2 | Watch what that does to multiplying by five. On ten marks, five times any mark lands on zero or five and nothing else, so the ten marks crowd onto two. | At "multiplying by five" at the upper right, on the strip's columns as chapter 08's doubling rows, red "on 10 marks" over "mark" 0 to 9 and, under it, "5 × mark" 0 to 9, the rows' titles left of them [S-F8.multiplier]. Then a red arrow grows from each mark down to five times it, one after another, a red ring on the first mark each lands on: 0 to 0, 1 to 5, 2 to 0, and so on to 9 to 5 [S-F8.ten.times5]. At "lands on zero or five" red outlines glow round the lower row's 0 and 5 [S-F8.ten.times5_values], and under the lower row, titled "arrows", how many arrows each mark gets: 5 at 0 and at 5 in red, 0 at the rest [S-F8.ten.times5_hits]. At "crowd onto two" "reaches 2 of 10 marks" under the counts [S-F8.ten.times5_reached]. Held for looking. | arrows, lagged, count | thought after "multiplying by five."; hold on the arrows (1.5 s) |
+| 13_3 | That's how a single share leaks. On ten marks, the straight line from before deals nine, three, seven, one and five. Trustee five's share is the secret plus five times the spin, and five times the spin is zero or five. So that share alone leaves two possible secrets, here zero and five. | At "the straight line from before" over the rail, red, "the line 5 + 4 × position, on 10 marks"; at "deals nine, three, seven, one and five" its heights' last digits drop into the rail's frames in red, one after another: 9, 3, 7, 1, 5 [S-F4.heights, S-F8.line.shares]. At "Trustee five's share" a red outline glows round trustee 5's frame and, over the line's name, "trustee 5: secret + 5 × spin" [S-F8.line.holder]; at "five times the spin is zero or five" the red outlines round the lower row's 0 and 5 glow again [S-F8.ten.times5_values]. At "leaves two possible secrets" faint ticks 0 to 9 at the left under the face, marks under them, titled "secrets that fit trustee 5's 25 → 5, on 10 marks" [S-F8.line.holder_share]; at "here zero and five" ticks 0 and 5 turn gold and gold rings glow round them [S-F8.line.candidates, S-F8.line.candidate_count]. | callback, ticks | connected list (the five shares); thought after "zero or five." |
+| 13_4 | Now do the same on eleven marks. Multiply every mark by five. How many different marks come out? ⟨predict⟩ | At "on eleven marks" the same rows lower right, "on 11 marks" over "mark" 0 to 10 and "5 × mark" 0 to 10, every mark under the same mark of the rows above [S-F1.marks]. At "Multiply every mark by five" an outline glows round "5 × mark". No arrow yet: the think ring with its "?" left of the lower rows. | rows, predict | question hold (think time) |
+| 13_5 | All eleven. One goes to five, two to ten, and three to fifteen, which wraps round to four. Each step lands five further round the dial, and every mark is reached exactly once. So on eleven marks, trustee five's share would fit every secret. | The "?" goes. At "All eleven" "reaches 11 of 11 marks" under where the lower counts will be [S-F8.eleven.times5_reached]. At "One goes to five" the arrow from 1 to 5 grows, a ring on 5 [S-F8.eleven.times5_step.1]; at "two to ten" from 2 to 10 [S-F8.eleven.times5_step.2]; at "wraps round to four" the grey note "15 → 4" at the right of the reach line and the arrow from 3 to 4 [S-F8.eleven.times5_step.3]. From "Each step lands five further" the other arrows, one after another, given time to form: 4 to 9, 5 to 3, and so on to 10 to 6, then 0 to 0 [S-F8.eleven.times5]; at "every mark is reached exactly once" each mark's count under the lower row, "arrows", 1 at every mark, and an outline glows round the row [S-F8.eleven.times5_hits, S-F8.eleven.times5_each]. At "trustee five's share would fit every secret" the left's ticks become 0 to 10, "secrets that fit trustee 5's 25 → 3, on 11 marks", and all turn gold [S-F4.wrap.5, S-F8.eleven.line_candidates]. Both rows held for comparing. | arrows, lagged, count, callback | thought after "All eleven."; thought after "reached exactly once."; hold on both rows (2 s) |
+| 13_6 | What's the difference? Ten is two times five, so two and five can each bring part of the ten, and their product lands on zero. Eleven doesn't split like that. It's prime: its only factors are one and itself. | The face, its product, the line's name, trustee 5's rule and the left's ticks go; the red cards stay. At "Ten is two times five" at the upper left, red, "10 = 2 × 5" [S-F8.ten.split]; at "their product lands on zero" red "on 10 marks, 2 × 5 = 10 lands on 0" [S-F8.ten.zero_product_mark]. At "Eleven doesn't split like that" in gold "11 = 1 × 11: no two numbers from 2 to 10 multiply to 11" [S-F8.eleven.factors], and under it "on 11 marks, 2 × 5 = 10 lands on 10" [S-F8.eleven.zero_product_check]; at "It's prime" a gold outline glows round the gold line. | card, controlled change | thought after "lands on zero."; thought after "split like that." |
+| 13_7 | Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero. | Under the cards, one line at a time at its phrase: at "Landing on zero means" "a × b lands on 0 exactly when a × b is a multiple of 11"; at "Every whole number splits into primes" "each number splits into primes one way only, so 11 is" and at "so a product of marks has eleven" "among a × b's primes only if it is among a's or b's"; at "No mark from one to ten does" "a and b from 1 to 10: no 11 among their primes" [S-F8.eleven.nonzero_range]; at "So on eleven marks, a product lands on zero" "so on 11 marks, a × b lands on 0 only if a or b is 0", and a gold outline glows round it. The cards and the proof stay until they can be read. | derivation | thought after "a multiple of eleven."; thought after "from one to ten does." |
+| 13_8 | So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do. | The cards and the proof's first lines go; its last line moves to the top. At "if two marks gave the same answer" under it "5 × u and 5 × w land on the same mark"; at "five times their difference would land on zero" "so 5 × (u − w) lands on 0"; at "unless they're the same mark" "5 isn't 0, so u − w lands on 0:" and "u and w are the same mark" [S-F8.multiplier]. At "every multiplier from one to ten" the rows, arrows, counts and notes go and, on the same columns, lower right, "on 11 marks, the marks reached by × 1 to × 10": for each multiplier "× 1" to "× 10" a row of cells, yellow at every mark it reaches, every row full [S-F8.eleven.reach, S-F8.eleven.rows_that_shuffle]; at "each one only shuffles the eleven marks" an outline glows round them all. At "On ten marks" upper right "on 10 marks, the marks reached by × 1 to × 9", the cells it misses red [S-F8.ten.reach]; at "only one, three, seven and nine do" outlines glow round those four full rows [S-F8.ten.rows_that_shuffle]. Held for looking. | derivation, grid | thought after "the same mark."; hold on the rows (1.5 s) |
+| 13_9 | That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do. | At "Any prime number of marks works" a precisely card: "the counting holds on any prime number of marks above both the number of trustees and the secret (here: 7, 11, 13 and on; Shamir 1979)" [S-F8.primes_for_five, S-F8.smallest_prime_for_five, S-X1.shamir.year]. At "each one has a position besides zero" rings glow round the rail's five discs [S-F1.trustees, S-F4.secret_position]. The card stays until it can be read. | precisely card | chapter_transition; hold until the card is read |
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -103,7 +103,8 @@
 - **line**, **curve**: what the dealer's recipe draws. "Curve" always means
   the two-spin kind. "Polynomial" is named once (09_7), after it has been shown.
 - **helper** (curve) and **weight**: chapters 14 and 15, built on paper.
-- **prime**: named at 13_5, after the ten-mark failure.
+- **prime**: named at 13_6, after the ten-mark failure and the eleven-mark
+  success.
 - **partner** (of a mark): the mark that, times it, lands on 1. Chapter 18,
   after dividing has been done the long way in chapter 17.
 
@@ -299,23 +300,28 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 12_1 | Why does the dial have eleven marks? We count in tens. ⟨predict⟩ With the same secret and the same spins on a ten-mark dial, would trustees four and five still learn nothing? | A red-tinted ten-mark dial (0 to 9) [S-F8.ten.marks] beside the eleven-mark one. | dial, predict | thought after "We count in tens."; question hold (think time) |
-| 12_2 | Their shares come out five and five. Try every secret, and only one fits those two shares. Five. On ten marks, these two can work out the secret alone. | Ten-mark board, rail cards 9, 7, 9, 5, 5 [S-F8.ten.shares]; trustees 4 and 5 hold 5 and 5 [S-F8.ten.pair_shares]. Secrets 0 … 9 tried in turn: only 5 keeps a curve [S-F8.ten.pair_secrets]; the pair's strip drops to 1; a red door opens. | sweep, live count | thought after "Five." |
-| 12_3 | Here's how. Take five times trustee four's height, minus four times trustee five's. The first spin cancels, and what's left is the secret minus twenty times the second spin. | TeX: 5·y₄ − 4·y₅ with y₄ = s + 4a + 16b, y₅ = s + 5a + 25b written small; the a terms (20a − 20a) cancel visibly; result s − 20·b [S-F8.ten.pair_formula, S-F8.pair_sum.spin_terms]. | derivation | thought after "Here's how."; clause after "minus four times trustee five's." |
-| 12_4 | On ten marks, twenty of anything lands on zero, so the second spin drops out too and the secret is left bare. With their shares, five fives minus four fives is five. | "20·b → 0 on ten marks" [S-F8.ten.leftover]; 5·5 − 4·5 = 25 − 20 = 5 [S-F8.ten.pair_formula_value]. | derivation | thought after "left bare." |
-| 12_5 | On eleven marks, minus twenty lands on two, so the same sum gives the secret plus twice the second spin, and that spin could be anything. For our pair it comes to eight, which says nothing. | Eleven-mark side: "− 20 → 2", 5·y₄ − 4·y₅ = s + 2·b [S-F8.eleven.leftover]; 5·2 − 4·6 = −14 → 8 [S-F8.eleven.pair_formula_unwrapped, S-F8.eleven.pair_formula_value]. precisely: "on ten marks, 9 of the 10 pairs of trustees learn something" [S-F8.ten.pairs_that_leak]. | derivation, precisely card | chapter_transition |
+| 12_1 | Why does the dial have eleven marks? We count in tens, so ten would seem the natural choice. Deal the same secret with the same two spins, but on a ten-mark dial. Would trustees four and five still learn nothing? ⟨predict⟩ | The rail with the vault's five cards face up in their frames, 8, 3, 1, 2 and 6 [S-F6.shares], trustees 4 and 5's discs purple; over the frames each trustee's height on paper, 19, 47, 89, 145 and 215 [S-F6.heights], and right of the rows their names, "height on paper" and "on 11 marks". At "Why does the dial have eleven marks?" the vault's dial face at the upper left, "the vault: 11 marks", its needle on 5 [S-F1.marks, S-F1.secret]; at "ten would seem the natural choice" a red face beside it, "10 marks", 0 to 9 [S-F8.ten.marks]. At "Deal the same secret with the same two spins" an outline round the heights [S-F6.spin_count]; at "on a ten-mark dial" a row of empty red frames between the heights and the rail, named "on 10 marks". At "Would trustees four and five" purple rings round discs 4 and 5 and outlines round both their frames [S-F1.pair]. Nothing answers yet: the think ring with its "?" at the right. | dial, predict | thought after "the natural choice."; question hold (think time) |
+| 12_2 | On ten marks, each height simply lands on its last digit: nine, seven, nine, five and five. So trustees four and five would hold five and five, not two and six. | The "?" goes. At "each height simply lands on its last digit" an outline glows round the heights. From "nine, seven, nine, five and five" each trustee's red card drops into its red frame as it is said, under its height [S-F8.ten.shares]. At "would hold five and five" purple rings round discs 4 and 5 and red outlines round their red frames [S-F8.ten.pair_shares]; at "not two and six" purple outlines round their frames on the rail, still holding the vault's 2 and 6 [S-F1.pair_shares]. | lagged cards, outline glow | connected list (the five shares); clause after "five and five," |
+| 12_3 | Now run the experiment again: for each secret, count the spin pairs that would hand them those two cards. On eleven marks, two and six fitted every secret exactly one way. On ten marks, five and five come from ten spin pairs, and every one of them has the secret five. These two could work out the secret alone. | The faces go. At "run the experiment again" chapter 05's tally at the right, twice, one over the other on the strip's columns: "11 marks: trustees 4 and 5 hold 2 and 6" over ticks 0 to 10, and "10 marks: trustees 4 and 5 hold 5 and 5" over ticks 0 to 9, each mark's tick in the same place in both, "ways" beside each, and over each tick a yellow bar for the spin pairs that deal the pair those cards with that secret, all still empty [S-F7.pair.count]. At "On eleven marks" the upper bars rise together, each one spin pair high with "1" over it, every tick lit, and under them "every secret fits, 1 way each" [S-F7.ways_each, S-F7.fitting, S-F8.eleven.pair_ways]. At "On ten marks" the lower bars come secret by secret, a yellow ring glowing round each tick as it is tried: none at 0 to 4, one ten times as high at 5 with "10" over it, none at 6 to 9; the ticks with no bar go faint, 5 alone lit, and under them "only secret 5 fits: 10 ways" [S-F8.ten.pair_ways, S-F8.ten.pair_spin_pairs, S-F8.ten.pair_secrets]; at "has the secret five" the bar at 5 swells. At "These two could work out the secret alone" purple rings round discs 4 and 5 and a red outline round the lower tick 5. Both tallies are held for comparing. | tally, bars, linked views | thought after "exactly one way."; thought after "the secret five."; hold on the two tallies (1.5 s) |
+| 12_4 | How? Write their heights out. Trustee four's is the secret, plus four of the first spin, plus sixteen of the second. Trustee five's is the secret, plus five of the first, plus twenty-five of the second. | The rail's heights and cards stay. At "Write their heights out" at the upper left, dim, "s: the secret   a: the 1st spin   b: the 2nd spin". At "Trustee four's is" a purple outline glows round trustee 4's height, 145, and "trustee 4:" with "s", "+ 4a" and "+ 16b" coming in as said [S-F8.pair_sum.height.4]; at "Trustee five's is" a purple outline glows round 215 and "trustee 5:" with "s", "+ 5a" and "+ 25b" under it, each term under trustee 4's like term [S-F8.pair_sum.height.5]: three columns, the secret's, the first spin's and the second spin's. | derivation, columns | thought after "Write their heights out."; thought after "sixteen of the second." |
+| 12_5 | To get rid of the first spin, make both hold the same amount of it. Five copies of trustee four's height hold twenty, and so do four copies of trustee five's. Take one from the other, and the first spin cancels. What's left is the secret, minus twenty of the second spin. | At "Five copies of trustee four's height" "5 × trustee 4:" with "5s", "+ 20a" and "+ 80b" under the two rows, term under term [S-F8.pair_sum.copies.4, S-F8.pair_sum.scaled.4]; at "hold twenty" a yellow outline round its "+ 20a" [S-F8.pair_sum.first_spin]; at "four copies of trustee five's" "4 × trustee 5:" with "4s", "+ 20a" and "+ 100b" under it, and a yellow outline round its "+ 20a" [S-F8.pair_sum.copies.5, S-F8.pair_sum.scaled.5]. At "Take one from the other" a rule under them and a "−" left of the second. At "the first spin cancels" a red stroke through both "+ 20a" and a red "0" under the rule in the first spin's column [S-F8.pair_sum.first_spin_left]. At "What's left is the secret" "first − second:" with "s" under 5s − 4s, and at "minus twenty of the second spin" "− 20b" under 80b − 100b [S-F8.pair_sum.spin_terms]. | derivation, strike, outline glow | thought after "the same amount of it."; thought after "the first spin cancels." |
+| 12_6 | On ten marks, twenty of anything lands on zero, so the second spin drops out too and leaves the secret bare. With their cards, five times five, minus four times five, is twenty-five minus twenty, which is five. | At "twenty of anything lands on zero" a red outline round the result's "− 20b" and, under the lower tally, in red, "− 20b lands on 0:" [S-F8.ten.twenty_lands, S-F8.ten.leftover]; at "leaves the secret bare" a red stroke through the result's "− 20b" and a red "s" after "− 20b lands on 0:". At "five times five" on the next line "5 × 5 − 4 × 5" [S-F8.ten.pair_formula, S-F8.ten.pair_formula_terms]; at "is twenty-five minus twenty" "= 25 − 20" [S-F8.ten.pair_formula_products]; at "which is five" "= 5", the lower tally's bar at 5 swells and a red ring round its tick [S-F8.ten.pair_formula_value]. | derivation, linked views | thought after "the secret bare." |
+| 12_7 | On eleven marks, minus twenty lands on two instead, so the same sum is the secret plus twice the second spin. With their two and six, five times two, minus four times six, is minus fourteen, which lands on eight. Every secret makes eight with exactly one second spin, so eight fits them all. | At "minus twenty lands on two" a purple outline round the result's "− 20b" and, under the upper tally, "− 20b lands on 2b:" [S-F8.eleven.minus_twenty_lands]; at "the secret plus twice the second spin" "s + 2b" after it [S-F8.eleven.leftover]. At "five times two" on the next line "5 × 2 − 4 × 6" [S-F8.eleven.pair_formula_terms]; at "is minus fourteen" "= 10 − 24 = −14" [S-F8.eleven.pair_formula_products, S-F8.eleven.pair_formula_unwrapped]; at "which lands on eight" "→ 8" [S-F8.eleven.pair_formula_value]. At "Every secret makes eight" an outline round the upper bars, each still one way high [S-F8.eleven.second_spin_each, S-F7.ways_each]. | derivation, linked views, outline glow | thought after "which lands on eight." |
+| 12_8 | And it isn't only this pair. On ten marks, nine of the ten pairs of trustees can always cross off some secrets, and two pairs always find it. Only trustees one and three learn nothing. | The derivation goes. At "On ten marks" the table at the upper left, "on 10 marks, what each pair's two shares leave" [S-F7.all_pairs.pairs], row by row: "1 secret: 2 and 5, 4 and 5" [S-F8.ten.pairs_left.1], "2 secrets: 1 and 5, 3 and 5" [S-F8.ten.pairs_left.2], "5 secrets: 1 and 2, 1 and 4, 2 and 3, 2 and 4, 3 and 4" [S-F8.ten.pairs_left.5], "all 10: 1 and 3" [S-F8.ten.pairs_left.10], then the note "whatever shares they hold; what is left fits equally well" [S-F8.ten.pairs_even_left] and, under it, "on 11 marks, every pair: all 11" [S-F7.all_pairs.two_leave]. At "nine of the ten pairs" a red outline round the first three rows [S-F8.ten.pairs_that_leak]; at "two pairs always find it" round the first [S-F8.ten.pairs_that_find]; at "Only trustees one and three" a gold outline round the last [S-F8.ten.pair_that_learns_nothing]. The table stays until it can be read. | table, outline glow | thought after "always find it."; chapter_transition; hold until the table is read |
 
 ## 13_why_prime: multiplying that never collapses
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 13_1 | The weak point on ten marks is that two numbers that aren't zero can multiply to zero. Two times five is ten, which lands on zero. | Ten-mark dial: 2 × 5 = 10 → 0 [S-F8.ten.zero_product], in red. | equation | thought after "multiply to zero." |
-| 13_2 | Multiply every mark by five, and only zero and five come out. So with a line on ten marks, trustee five's share alone would leave just two possible secrets. | Times row "× 5" on ten marks: 0, 5, 0, 5, 0, 5, 0, 5, 0, 5 [S-F8.ten.times5], the repeats red. Inset: the ten-mark line's shares 9, 3, 7, 1, 5 [S-F8.line.shares]; trustee 5 holds 5 [S-F8.line.holder_share]; strip 2 lit, 0 and 5 [S-F8.line.candidates]. | table, live count | thought after "come out." |
-| 13_3 | Now do the same on eleven marks. ⟨predict⟩ Multiply every mark by five. How many different marks come out? | An empty times row "× 5" on eleven marks. | predict | question hold (think time) |
-| 13_4 | All eleven. Zero, five, ten, four, nine, three, eight, two, seven, one and six. | The row fills: 0, 5, 10, 4, 9, 3, 8, 2, 7, 1, 6 [S-F8.eleven.times5]; every result once. | lagged | connected list |
-| 13_5 | That's because eleven is prime, with no factors except one and itself. So on eleven marks, a product lands on zero only when one of its factors is zero. | Card: "11 is prime: a × b lands on 0 only if a or b is 0". All ten rows × 1 … × 10 flash full [S-F8.eleven.rows_that_shuffle]; on ten marks only × 1, 3, 7, 9 [S-F8.ten.rows_that_shuffle]. | grid | thought after "one and itself." |
-| 13_6 | So if two marks gave the same answer after multiplying by five, their difference times five would land on zero. That can't happen unless they were the same mark. | TeX: 5·u = 5·w ⇒ 5·(u − w) = 0 ⇒ u = w. | derivation | thought after "would land on zero." |
-| 13_7 | Any prime number of marks works, if it's more than the number of trustees, so that each trustee has a position besides zero. The secret also has to be one of its marks. For five trustees, seven marks would already do. | precisely: "any prime p larger than the number of trustees works; the smallest for five is 7 [S-F8.smallest_prime_for_five]; Shamir asks for p bigger than both the secret and n". | precisely card | chapter_transition |
+| 13_1 | What went wrong? On ten marks, marks that aren't zero can multiply to zero. Two times five is ten, which lands on zero. | The rail, its discs and empty frames. At "What went wrong?" the red ten-mark face at the upper left, "10 marks" over it, its needle on 0 [S-F8.ten.marks]. At "marks that aren't zero" purple rings glow round the face's 2 and 5 [S-F8.ten.zero_product]. At "Two times five is ten" "2 × 5 = 10" right of the face, and the needle turns five marks and five more, round past the top to 0 [S-F8.ten.zero_product_sum]; at "which lands on zero" a red "→ 0" after it and a red ring glows round the face's 0 [S-F8.ten.zero_product_mark]. | dial, equation | thought after "multiply to zero." |
+| 13_2 | Watch what that does to multiplying by five. On ten marks, five times any mark lands on zero or five and nothing else, so the ten marks crowd onto two. | At "multiplying by five" at the upper right, on the strip's columns as chapter 08's doubling rows, red "on 10 marks" over "mark" 0 to 9 and, under it, "5 × mark" 0 to 9, the rows' titles left of them [S-F8.multiplier]. Then a red arrow grows from each mark down to five times it, one after another, a red ring on the first mark each lands on: 0 to 0, 1 to 5, 2 to 0, and so on to 9 to 5 [S-F8.ten.times5]. At "lands on zero or five" red outlines glow round the lower row's 0 and 5 [S-F8.ten.times5_values], and under the lower row, titled "arrows", how many arrows each mark gets: 5 at 0 and at 5 in red, 0 at the rest [S-F8.ten.times5_hits]. At "crowd onto two" "reaches 2 of 10 marks" under the counts [S-F8.ten.times5_reached]. Held for looking. | arrows, lagged, count | thought after "multiplying by five."; hold on the arrows (1.5 s) |
+| 13_3 | That's how a single share leaks. On ten marks, the straight line from before deals nine, three, seven, one and five. Trustee five's share is the secret plus five times the spin, and five times the spin is zero or five. So that share alone leaves two possible secrets, here zero and five. | At "the straight line from before" over the rail, red, "the line 5 + 4 × position, on 10 marks"; at "deals nine, three, seven, one and five" its heights' last digits drop into the rail's frames in red, one after another: 9, 3, 7, 1, 5 [S-F4.heights, S-F8.line.shares]. At "Trustee five's share" a red outline glows round trustee 5's frame and, over the line's name, "trustee 5: secret + 5 × spin" [S-F8.line.holder]; at "five times the spin is zero or five" the red outlines round the lower row's 0 and 5 glow again [S-F8.ten.times5_values]. At "leaves two possible secrets" faint ticks 0 to 9 at the left under the face, marks under them, titled "secrets that fit trustee 5's 25 → 5, on 10 marks" [S-F8.line.holder_share]; at "here zero and five" ticks 0 and 5 turn gold and gold rings glow round them [S-F8.line.candidates, S-F8.line.candidate_count]. | callback, ticks | connected list (the five shares); thought after "zero or five." |
+| 13_4 | Now do the same on eleven marks. Multiply every mark by five. How many different marks come out? ⟨predict⟩ | At "on eleven marks" the same rows lower right, "on 11 marks" over "mark" 0 to 10 and "5 × mark" 0 to 10, every mark under the same mark of the rows above [S-F1.marks]. At "Multiply every mark by five" an outline glows round "5 × mark". No arrow yet: the think ring with its "?" left of the lower rows. | rows, predict | question hold (think time) |
+| 13_5 | All eleven. One goes to five, two to ten, and three to fifteen, which wraps round to four. Each step lands five further round the dial, and every mark is reached exactly once. So on eleven marks, trustee five's share would fit every secret. | The "?" goes. At "All eleven" "reaches 11 of 11 marks" under where the lower counts will be [S-F8.eleven.times5_reached]. At "One goes to five" the arrow from 1 to 5 grows, a ring on 5 [S-F8.eleven.times5_step.1]; at "two to ten" from 2 to 10 [S-F8.eleven.times5_step.2]; at "wraps round to four" the grey note "15 → 4" at the right of the reach line and the arrow from 3 to 4 [S-F8.eleven.times5_step.3]. From "Each step lands five further" the other arrows, one after another, given time to form: 4 to 9, 5 to 3, and so on to 10 to 6, then 0 to 0 [S-F8.eleven.times5]; at "every mark is reached exactly once" each mark's count under the lower row, "arrows", 1 at every mark, and an outline glows round the row [S-F8.eleven.times5_hits, S-F8.eleven.times5_each]. At "trustee five's share would fit every secret" the left's ticks become 0 to 10, "secrets that fit trustee 5's 25 → 3, on 11 marks", and all turn gold [S-F4.wrap.5, S-F8.eleven.line_candidates]. Both rows held for comparing. | arrows, lagged, count, callback | thought after "All eleven."; thought after "reached exactly once."; hold on both rows (2 s) |
+| 13_6 | What's the difference? Ten is two times five, so two and five can each bring part of the ten, and their product lands on zero. Eleven doesn't split like that. It's prime: its only factors are one and itself. | The face, its product, the line's name, trustee 5's rule and the left's ticks go; the red cards stay. At "Ten is two times five" at the upper left, red, "10 = 2 × 5" [S-F8.ten.split]; at "their product lands on zero" red "on 10 marks, 2 × 5 = 10 lands on 0" [S-F8.ten.zero_product_mark]. At "Eleven doesn't split like that" in gold "11 = 1 × 11: no two numbers from 2 to 10 multiply to 11" [S-F8.eleven.factors], and under it "on 11 marks, 2 × 5 = 10 lands on 10" [S-F8.eleven.zero_product_check]; at "It's prime" a gold outline glows round the gold line. | card, controlled change | thought after "lands on zero."; thought after "split like that." |
+| 13_7 | Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero. | Under the cards, one line at a time at its phrase: at "Landing on zero means" "a × b lands on 0 exactly when a × b is a multiple of 11"; at "Every whole number splits into primes" "each number splits into primes one way only, so 11 is" and at "so a product of marks has eleven" "among a × b's primes only if it is among a's or b's"; at "No mark from one to ten does" "a and b from 1 to 10: no 11 among their primes" [S-F8.eleven.nonzero_range]; at "So on eleven marks, a product lands on zero" "so on 11 marks, a × b lands on 0 only if a or b is 0", and a gold outline glows round it. The cards and the proof stay until they can be read. | derivation | thought after "a multiple of eleven."; thought after "from one to ten does." |
+| 13_8 | So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do. | The cards and the proof's first lines go; its last line moves to the top. At "if two marks gave the same answer" under it "5 × u and 5 × w land on the same mark"; at "five times their difference would land on zero" "so 5 × (u − w) lands on 0"; at "unless they're the same mark" "5 isn't 0, so u − w lands on 0:" and "u and w are the same mark" [S-F8.multiplier]. At "every multiplier from one to ten" the rows, arrows, counts and notes go and, on the same columns, lower right, "on 11 marks, the marks reached by × 1 to × 10": for each multiplier "× 1" to "× 10" a row of cells, yellow at every mark it reaches, every row full [S-F8.eleven.reach, S-F8.eleven.rows_that_shuffle]; at "each one only shuffles the eleven marks" an outline glows round them all. At "On ten marks" upper right "on 10 marks, the marks reached by × 1 to × 9", the cells it misses red [S-F8.ten.reach]; at "only one, three, seven and nine do" outlines glow round those four full rows [S-F8.ten.rows_that_shuffle]. Held for looking. | derivation, grid | thought after "the same mark."; hold on the rows (1.5 s) |
+| 13_9 | That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do. | At "Any prime number of marks works" a precisely card: "the counting holds on any prime number of marks above both the number of trustees and the secret (here: 7, 11, 13 and on; Shamir 1979)" [S-F8.primes_for_five, S-F8.smallest_prime_for_five, S-X1.shamir.year]. At "each one has a position besides zero" rings glow round the rail's five discs [S-F1.trustees, S-F4.secret_position]. The card stays until it can be read. | precisely card | chapter_transition; hold until the card is read |
 
 ## 14_helper_curves: building a curve through three points, on paper
 
@@ -356,7 +362,7 @@
 | 17_2 | For this secret, eighty of the hundred and twenty-one spin pairs leave a fraction like that. A third of an eleven isn't a whole eleven, so the reason the whole weights worked breaks. | The spin grid, 121 cells [S-F6.spin_pairs], with the 80 whose sum is a fraction [S-F10.fractional_spin_pairs] tinted red. Card: "1/3 of 11 is not a whole number of elevens". | grid, card | thought after "a fraction like that." |
 | 17_3 | Clear the thirds first. Three times each weight is a whole number, ten, minus fifteen and eight. On paper they give three times the secret, and whole weights survive wrapping. On the other dealing's shares, nine, ten and eight, that's ninety, minus a hundred and fifty, plus sixty-four, which is four. | Weights × 3 = 10, −15, 8 [S-F10.cleared_weights]; TeX: 10·9 − 15·10 + 8·8 = 90 − 150 + 64 = 4 [S-F10.other.cleared_sum, S-F10.other.cleared_lands]; a tag "3 × secret lands on 4". | derivation | thought after "Clear the thirds first." |
 | 17_4 | So three times the secret lands on four. ⟨predict⟩ Which mark, times three, lands on four? | The times-three row on eleven marks, cells hidden; a gold "?" over it with "→ 4". | predict | question hold (think time) |
-| 17_5 | Five. Three times five is fifteen, which lands on four. And no other mark does, because the row of threes reaches every mark exactly once, as the row of fives did on eleven marks. | Row × 3: 0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8 [S-F10.times3]; the cell under 5 lights: 3 × 5 = 15 → 4 [S-F10.thrice, S-F10.thrice_unwrapped]; the row of fives from 13_4 beside it. | row, highlight | thought after "Five."; chapter_transition |
+| 17_5 | Five. Three times five is fifteen, which lands on four. And no other mark does, because the row of threes reaches every mark exactly once, as the row of fives did on eleven marks. | Row × 3: 0, 3, 6, 9, 1, 4, 7, 10, 2, 5, 8 [S-F10.times3]; the cell under 5 lights: 3 × 5 = 15 → 4 [S-F10.thrice, S-F10.thrice_unwrapped]; the row of fives from 13_5 beside it. | row, highlight | thought after "Five."; chapter_transition |
 
 ## 18_dividing_on_the_dial: partners
 
--- a/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
+++ b/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
@@ -71,18 +71,23 @@
 11_4	Now let trustee one join them, holding eight. How many of the eleven curves pass through eight at position one?
 11_5	Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one.
 11_6	The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one.
-12_1	Why does the dial have eleven marks? We count in tens. With the same secret and the same spins on a ten-mark dial, would trustees four and five still learn nothing?
-12_2	Their shares come out five and five. Try every secret, and only one fits those two shares. Five. On ten marks, these two can work out the secret alone.
-12_3	Here's how. Take five times trustee four's height, minus four times trustee five's. The first spin cancels, and what's left is the secret minus twenty times the second spin.
-12_4	On ten marks, twenty of anything lands on zero, so the second spin drops out too and the secret is left bare. With their shares, five fives minus four fives is five.
-12_5	On eleven marks, minus twenty lands on two, so the same sum gives the secret plus twice the second spin, and that spin could be anything. For our pair it comes to eight, which says nothing.
-13_1	The weak point on ten marks is that two numbers that aren't zero can multiply to zero. Two times five is ten, which lands on zero.
-13_2	Multiply every mark by five, and only zero and five come out. So with a line on ten marks, trustee five's share alone would leave just two possible secrets.
-13_3	Now do the same on eleven marks. Multiply every mark by five. How many different marks come out?
-13_4	All eleven. Zero, five, ten, four, nine, three, eight, two, seven, one and six.
-13_5	That's because eleven is prime, with no factors except one and itself. So on eleven marks, a product lands on zero only when one of its factors is zero.
-13_6	So if two marks gave the same answer after multiplying by five, their difference times five would land on zero. That can't happen unless they were the same mark.
-13_7	Any prime number of marks works, if it's more than the number of trustees, so that each trustee has a position besides zero. The secret also has to be one of its marks. For five trustees, seven marks would already do.
+12_1	Why does the dial have eleven marks? We count in tens, so ten would seem the natural choice. Deal the same secret with the same two spins, but on a ten-mark dial. Would trustees four and five still learn nothing?
+12_2	On ten marks, each height simply lands on its last digit: nine, seven, nine, five and five. So trustees four and five would hold five and five, not two and six.
+12_3	Now run the experiment again: for each secret, count the spin pairs that would hand them those two cards. On eleven marks, two and six fitted every secret exactly one way. On ten marks, five and five come from ten spin pairs, and every one of them has the secret five. These two could work out the secret alone.
+12_4	How? Write their heights out. Trustee four's is the secret, plus four of the first spin, plus sixteen of the second. Trustee five's is the secret, plus five of the first, plus twenty-five of the second.
+12_5	To get rid of the first spin, make both hold the same amount of it. Five copies of trustee four's height hold twenty, and so do four copies of trustee five's. Take one from the other, and the first spin cancels. What's left is the secret, minus twenty of the second spin.
+12_6	On ten marks, twenty of anything lands on zero, so the second spin drops out too and leaves the secret bare. With their cards, five times five, minus four times five, is twenty-five minus twenty, which is five.
+12_7	On eleven marks, minus twenty lands on two instead, so the same sum is the secret plus twice the second spin. With their two and six, five times two, minus four times six, is minus fourteen, which lands on eight. Every secret makes eight with exactly one second spin, so eight fits them all.
+12_8	And it isn't only this pair. On ten marks, nine of the ten pairs of trustees can always cross off some secrets, and two pairs always find it. Only trustees one and three learn nothing.
+13_1	What went wrong? On ten marks, marks that aren't zero can multiply to zero. Two times five is ten, which lands on zero.
+13_2	Watch what that does to multiplying by five. On ten marks, five times any mark lands on zero or five and nothing else, so the ten marks crowd onto two.
+13_3	That's how a single share leaks. On ten marks, the straight line from before deals nine, three, seven, one and five. Trustee five's share is the secret plus five times the spin, and five times the spin is zero or five. So that share alone leaves two possible secrets, here zero and five.
+13_4	Now do the same on eleven marks. Multiply every mark by five. How many different marks come out?
+13_5	All eleven. One goes to five, two to ten, and three to fifteen, which wraps round to four. Each step lands five further round the dial, and every mark is reached exactly once. So on eleven marks, trustee five's share would fit every secret.
+13_6	What's the difference? Ten is two times five, so two and five can each bring part of the ten, and their product lands on zero. Eleven doesn't split like that. It's prime: its only factors are one and itself.
+13_7	Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero.
+13_8	So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do.
+13_9	That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do.
 14_1	Now the door from the start. Trustees one, two and three hold eight, three and one, and those three numbers somehow give five. To see why, go back to paper first, where the dealer's curve had heights nineteen, forty-seven and eighty-nine.
 14_2	On paper, three points pin a parabola down, as we saw. So build that one parabola from three helper curves. Trustee one's helper is one at position one, and zero at positions two and three. The other two helpers do the same for their own trustees.
 14_3	Then nineteen times the first helper, plus forty-seven times the second, plus eighty-nine times the third, passes through all three points. At each trustee, only that trustee's helper is switched on.
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -71,3 +71,20 @@
 11_4	Now let trustee one join them, holding eight. How many of the eleven curves pass through eight at position one?
 11_5	Just one. Each curve passes position one at a different mark, and only the curve for five passes at eight. The strip drops from eleven secrets to one.
 11_6	The same count holds for every pair of trustees and every two shares they could hold. Two shares leave all eleven secrets, and any third share leaves exactly one.
+12_1	Why does the dial have eleven marks? We count in tens, so ten would seem the natural choice. Deal the same secret with the same two spins, but on a ten-mark dial. Would trustees four and five still learn nothing?
+12_2	On ten marks, each height simply lands on its last digit: nine, seven, nine, five and five. So trustees four and five would hold five and five, not two and six.
+12_3	Now run the experiment again: for each secret, count the spin pairs that would hand them those two cards. On eleven marks, two and six fitted every secret exactly one way. On ten marks, five and five come from ten spin pairs, and every one of them has the secret five. These two could work out the secret alone.
+12_4	How? Write their heights out. Trustee four's is the secret, plus four of the first spin, plus sixteen of the second. Trustee five's is the secret, plus five of the first, plus twenty-five of the second.
+12_5	To get rid of the first spin, make both hold the same amount of it. Five copies of trustee four's height hold twenty, and so do four copies of trustee five's. Take one from the other, and the first spin cancels. What's left is the secret, minus twenty of the second spin.
+12_6	On ten marks, twenty of anything lands on zero, so the second spin drops out too and leaves the secret bare. With their cards, five times five, minus four times five, is twenty-five minus twenty, which is five.
+12_7	On eleven marks, minus twenty lands on two instead, so the same sum is the secret plus twice the second spin. With their two and six, five times two, minus four times six, is minus fourteen, which lands on eight. Every secret makes eight with exactly one second spin, so eight fits them all.
+12_8	And it isn't only this pair. On ten marks, nine of the ten pairs of trustees can always cross off some secrets, and two pairs always find it. Only trustees one and three learn nothing.
+13_1	What went wrong? On ten marks, marks that aren't zero can multiply to zero. Two times five is ten, which lands on zero.
+13_2	Watch what that does to multiplying by five. On ten marks, five times any mark lands on zero or five and nothing else, so the ten marks crowd onto two.
+13_3	That's how a single share leaks. On ten marks, the straight line from before deals nine, three, seven, one and five. Trustee five's share is the secret plus five times the spin, and five times the spin is zero or five. So that share alone leaves two possible secrets, here zero and five.
+13_4	Now do the same on eleven marks. Multiply every mark by five. How many different marks come out?
+13_5	All eleven. One goes to five, two to ten, and three to fifteen, which wraps round to four. Each step lands five further round the dial, and every mark is reached exactly once. So on eleven marks, trustee five's share would fit every secret.
+13_6	What's the difference? Ten is two times five, so two and five can each bring part of the ten, and their product lands on zero. Eleven doesn't split like that. It's prime: its only factors are one and itself.
+13_7	Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero.
+13_8	So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do.
+13_9	That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do.
--- a/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
+++ b/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored semantic intentions for chapters06\u201311 before synthesis. Preserve predictions, sequential card reading and scene-work overruns separately. Durations are proposals, not measured speech or listening acceptance.",
+  "_comment": "Authored semantic intentions for chapters06\u201313 before synthesis. Preserve predictions, sequential card reading and scene-work overruns separately. Durations are proposals, not measured speech or listening acceptance.",
   "overrides": [
     {
       "video": "shamir_secret_sharing",
@@ -319,6 +319,226 @@
         0.9
       ],
       "reason": "Answer the prediction, then show why."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_1",
+      "after": "the natural choice",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the natural expectation stand before the dial is changed."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_3",
+      "after": "exactly one way",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the full eleven-mark tally register before the ten-mark one fills."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_3",
+      "after": "has the secret five",
+      "role": "thought",
+      "range_s": [
+        0.7,
+        1.1
+      ],
+      "reason": "Let the single lit tick contrast with the full row above it."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_4",
+      "after": "Write their heights out",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "Set up the letters before the first height is written."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_4",
+      "after": "sixteen of the second",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "Let the first row's three columns settle before the second row."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_5",
+      "after": "the same amount of it",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the plan stand before the copies are written out."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_5",
+      "after": "the first spin cancels",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the struck terms and their zero register."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_6",
+      "after": "the secret bare",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the bare secret land before the cards are put in."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_7",
+      "after": "which lands on eight",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Close the eleven-mark sum before saying why eight hides nothing."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "12_8",
+      "after": "always find it",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Separate the leaking pairs from the one pair that learns nothing."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_1",
+      "after": "multiply to zero",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the surprising claim stand before its example."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_2",
+      "after": "multiplying by five",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "Let the rows appear before the arrows start."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_3",
+      "after": "the spin is zero or five",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Connect the crowded row to the share before the two candidates."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_5",
+      "after": "All eleven",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Answer the prediction, then show it arrow by arrow."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_5",
+      "after": "reached exactly once",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the full row of ones register before the callback."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_6",
+      "after": "their product lands on zero",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Close the ten-mark split before the eleven-mark contrast."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_6",
+      "after": "split like that",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the contrast land before the word prime."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_7",
+      "after": "a multiple of eleven",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the definition settle before the factorization step."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_7",
+      "after": "one to ten does",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the premise land before the conclusion."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "13_8",
+      "after": "the same mark",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Close the proof before it is extended to every multiplier."
     }
   ]
 }
--- a/explainer_corpus/shamir_secret_sharing/docs/brief.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/brief.md
@@ -87,7 +87,7 @@
 Also on page 613: "the smallest usable value of p is n + 1 (there must be at
 least n + 1 distinct arguments in [0, p) to evaluate q(x) at)". Earlier on
 the same page, the construction asks for "a prime p which is bigger than
-both D and n". Draft line 13_7 rests on both. It says the prime must exceed
+both D and n". Draft line 13_9 rests on both. It says the prime must exceed
 the number of trustees and that the secret must be one of the dial's marks.
 
 **The size of each piece (page 613)** is property (1) in the list "Some of
@@ -192,7 +192,7 @@
 - **Single letters:** "D" and "k minus one" (23_5). TTS may read a lone
   letter as a word.
 - **Terms of art:** "parabola" (09_4, 14_2, 14_6), "polynomial" (09_7) and "prime"
-  (13_5, 13_7). Check the stress.
+  (13_6, 13_7, 13_9). Check the stress.
 - **The year:** "nineteen seventy-nine" (06_6) is written out, so it should
   not be read as a cardinal.
 
```
<!-- shamir-1213 patch end -->

## Chapters 14 and 15, unregistered (2026-10-24, batch_pending, not self-closed)

Root's assignment (after it froze my six held-out directions, Mail 7039):
- continue native chapters `14_helper_curves` and `15_three_shares` from
  the full storyboard draft, carrying the motivation and the same example
  (trustees 1, 2 and 3; cards 8, 3 and 1; heights 19, 47 and 89) before the
  technical steps;
- write only new unregistered source in this topic and topic-local
  integration notes and draft rows; preserve all handed-off 01–13 source;
  no registration or shared-file edit, no Cargo, render, TTS or ASR;
- include independent arithmetic checks, cue/script bindings, authored
  semantic pause intentions and native reading and prediction holds;
- make the pictures show the reasoning with persistent objects and
  controlled changes, not only equations and text;
- mail the exact frozen hashes and dependencies.

**Status.**
- I wrote two new unregistered modules, `helper_curves.rs` and
  `three_shares.rs`. I edited no registered, delivered or shared file; every
  change to a registered file is in the integration patch below.
- Nothing was compiled, rendered, synthesized or listened to. No Cargo
  command was run.
- Bead mx-video-program-program-87do.46.1's Shamir lane stays with root; I
  close nothing.

**The base moved under this work.** Root integrated 10/11 into live at
about 12:36 local time. For every registered file, live now equals the
live I started from plus `rebased-integration-10-11.patch` (503c9a52…). The
exception is `one_more_share.rs`, which is root's own version: the `Timed`
import and the sequential `read_on` card deadlines. 12/13 is not yet in
live. I tested `integration-12-13.patch` (db5574e3…) on a fresh copy of
current live: it applies cleanly and gives exactly my base, all 11
registered files equal by `cmp`. Nothing in 14/15 uses `one_more_share.rs`.

**Prior bytes.** `docs/previous/ch1415-before-14-15-20261024/WORK_STATUS.md`
(gitignored) is this file as it stood before this section: sha256
c96b633c390c57eed9ec567635546a221860e3755751879b6bad24ec15602655, the
bytes root froze with the 12/13 intake.

### Files (new, under explainer_corpus/shamir_secret_sharing/src/chapters/)

| File | Lines | sha256 |
|---|---|---|
| `helper_curves.rs` | 1445 | 7a4452cd80b1f90d54d5a5aadb300b0ff751e43cfffabee823527d36f511b553 |
| `three_shares.rs` | 1016 | 6cd22af4486b304076ec723e8a690e7b617a5a37561dde0835bf9f12cb362f74 |

- **`helper_curves.rs`**: `HelperCurves`, key `14_helper_curves`.
  - Data: `product_at`, `helper_value`, `weighted_sum_at`, `helper_exact`,
    `at_trio` (an error unless each helper is 1 at its own trustee and 0
    at the others), `sum_exact`, `zeros`, `factors_at`, `sampled`,
    `clipped`, `dealer_height`, `paper_trio` and `check_helpers` (the
    scene stops unless the drawn sum lies on the dealer's parabola).
  - Text: `signed`, `bracketed`, `helper_name`, `product_text`,
    `recipe_line`, `divide_text`, `own_working`, `own_line`, `sum_line`,
    `scale_tag` and `card_text`.
  - `Setting`, the plot and the three helper boards, which chapter 15
    reuses.
  - Six tests:
    - `every_phrase_the_scene_waits_for_is_in_its_line` (45 phrases, in
      order; 14_5 ends with the whole question);
    - `the_pictures_are_the_facts`;
    - `the_helpers_are_counted_by_hand`, with the controls: the product
      alone is 2 at trustee 1; a wrong height breaks `check_helpers`; trio
      2, 4, 5 has weight 10/3;
    - `the_clip_cuts_where_the_curve_leaves`;
    - `the_card_is_two_lines` (wraps 72 to 92);
    - `the_layout_keeps_its_pieces_apart`.
- **`three_shares.rs`**: `ThreeShares`, key `15_three_shares`.
  - `Working::new` is an error unless:
    - the weights are whole;
    - each height is its card plus whole elevens;
    - the paper sum is the secret;
    - the two sums are whole elevens apart;
    - the cards' sum lands on the secret;
    - the dial weights are the model's;
    - the model rebuilds the secret.
  - Text: `zero_working`, `weight_tag`, `signed_sum`, `weight_piece`,
    `term_text`, `paper_line`, `cards_line`, `split_row`, `elevens_line`,
    `turns_line` and `card_text`.
  - Seven tests:
    - `every_phrase_the_scene_waits_for_is_in_its_line` (31 phrases; 15_4
      ends with the whole question);
    - `the_pictures_are_the_facts`;
    - `the_door_is_counted_by_hand`, with the controls: weights 3, −3, 2
      give 94 on paper and 17 → 6 on the cards; the dial weights on the
      heights give 522, which lands on 5 but is not 5; trio 2, 4, 5 and a
      nudged card are refused;
    - `the_card_is_two_lines` (it never says always, any, secure or safe);
    - `the_layout_keeps_its_pieces_apart` (pure-constant asserts in
      `const { }` blocks);
    - `the_needle_runs_the_sum`.

### The chapters against the assignment

The rows are in `docs/previous/ch1415-evidence-20261024/sections-14-15.md`
and, after the patch, in both storyboards. Line ids are 14_1 to 14_8 and
15_1 to 15_7.

- **Motivation first, the same example.**
  - 14_1 returns to the door the trio opened. It opens again on 8, 3 and 1
    and shuts. The trio's three points sit on chapter 09's dial board as
    "on the dial: points only, no curve" over "spins: ? and ?".
  - The question is "So how can three numbers give five?". It is
    rhetorical, so it gets a cue transition, not think time.
- **The idea on paper, kept apart from the dial.**
  - 14_2 pretends the heights 19, 47 and 89 are known. The plot is titled
    "on paper: ordinary numbers"; the dealer's parabola is in ink, with a
    gold "?" where it meets position 0.
  - 14_6 adds "on paper, halving is fine". Fractions on the dial are left
    to chapter 16.
- **The goal before the construction.**
  - 14_3 shows each helper only as its three dots, 1 at its trustee and 0
    at the others.
  - 14_4 scales by the heights. One column outline moves from position 1
    to 2 to 3 while each sum is written, so the zero helpers are visible.
  - 14_5 then asks how such a helper can be built. Its ⟨predict⟩ ends the
    whole question, "What simple recipe is zero at both positions two and
    three?", with think time.
- **The construction as controlled changes.**
  - 14_6 builds trustee 1's helper. The product's curve is drawn in ink and
    clipped to the board; a ring marks its 2 at position 1. It then gives
    way to the halved helper in teal: the same curve, rescaled.
  - 14_7 repeats this for the other two, with "÷ (−1)".
  - 14_8 draws the weighted sum along the ink parabola, which then turns
    teal: one curve. Rings mark the three points and the "?". Its precisely
    card reads "three points pin one parabola (chapter 09), so 19 × helper 1
    + 47 × helper 2 + 89 × helper 3 is the dealer's curve; at 0 it is the
    secret".
- **The weights from the same boards.**
  - In 15_1 and 15_2, a gold column at position 0 reads each helper there:
    6 ÷ 2 = 3, 3 ÷ (−1) = −3, 2 ÷ 2 = 1. A gold dot sits on each board, and
    each weight goes under its trustee's disc.
  - 15_3 moves each weight from the board to stand before its "× 19" tag,
    then the term: 57 − 141 + 89 = 5.
- **The obstacle, then the payoff.**
  - 15_4's ⟨predict⟩ ends "Will the same weights still work on what they
    hold?", with think time where the plot was.
  - In 15_5 the cards give 24 − 9 + 1 = 16 → 5. The vault returns, its
    needle runs all sixteen marks, once round and on to 5, and the door
    opens, with a 0.8 s hold.
- **Why it works, with the same numbers.**
  - 15_6 splits each height into its card and whole elevens (19 = 8 +
    1 × 11 and so on), with a halo on each trustee's frame.
  - 15_7 adds:
    - the weighted elevens, = −1 × 11;
    - an outline round both sums;
    - "16 = 5 + 1 × 11".
  - "weight −3" then gives way to "weight 8" (the dial weights 3, 8 and 1).
  - Its precisely card names Lagrange interpolation and claims nothing
    about security.
- **Emphasis** is by rings, outlines, halos and swells of shapes only. No
  typeset text changes colour or size.
- **Holds.**
  - Each chapter ends on a single card, held until `read_by` and then
    0.6 s more.
  - Each scene waits on its phrases in spoken order.
  - The authored pause intentions are in each row's hold column and in
    `docs/pause-overrides.json`: 24 entries, all inside the director's
    policy ranges.

### The integration patch

`docs/previous/ch1415-evidence-20261024/integration-14-15.patch`,
2982efc44b385b214e49a84641506f89331df7b9bbdf174cf8047c7065dde23a, 1003
lines, 10 files, 21 hunks. Apply it in this order:
1. live as it is now (10/11 integrated by root);
2. `integration-12-13.patch` (db5574e3…);
3. this patch.

Tested on a fresh copy of live:
- the 12/13 patch applies cleanly and gives exactly my base;
- this patch then applies cleanly, with no fuzz and no `.rej`, and gives
  exactly my work tree (all 11 registered files by `cmp`; `model.rs` is
  unchanged);
- applied alone to live, this patch is refused: 19 hunks fail.

With the two new modules copied in, that tree passes `rustfmt --check` and
the lint mirror.

| File | live now | after 12/13 (base) | after this patch |
|---|---|---|---|
| `src/chapters/mod.rs` | d1e3e70c | 64f0b437 | 7d5aa429 |
| `src/main.rs` | 2a2a222a | 63059eea | a8ac43e6 |
| `src/facts.rs` | c2837471 | 52b4d1e3 | 2e34f8d6 |
| `oracle.py` | 005d2491 | d1c1ce7e | 934c318a |
| `docs/storyboard.md` | 8426f4e9 | 98ccbc98 | b9da708a |
| `docs/storyboard-draft.md` | 123179f9 | 5f0c8d22 | cfa2724b |
| `docs/script-draft.tsv` | a8a917b9 | b2a1a17c | de6123a4 |
| `script.tsv` | be6a1554 | 2918d4f1 | ff2bd25f |
| `docs/pause-overrides.json` | f13918c5 | 7e680c51 | 3c71e90f |
| `docs/brief.md` | 1b75b7a8 | 91734f69 | e3b078ec |

What the patch changes:
- **`src/chapters/mod.rs`.** Registers `helper_curves` and `three_shares`,
  intensity 0.5. The doc says 01 to 15, "the other eight".
- **`src/facts.rs`.** 43 new S-F9 facts (308 → 351), in a block whose
  error checks are:
  - each helper's factors multiply to its own value;
  - the scaled sum hits each height;
  - the helpers are alike on and off;
  - each value at 0 over its own value is the weight;
  - the weights are whole;
  - the cards' terms add to the unwrapped sum;
  - each height is its card and its elevens;
  - the paper sum is the cards' sum and the weighted elevens.

  The golden table has the 43 oracle lines, and the count is
  `assert_eq!(f.len(), 351)`. The doc says chapters 01 to 15.
- **`oracle.py`.** An independent 14/15 block:
  - each helper as the ordinary curve through 1 and 0, 0 by Gaussian
    elimination over fractions, not the product the scene writes;
  - zeros found by trying every whole position from −20 to 20;
  - each product of factors compared at 41 positions;
  - the heights split by winding.

  Self-test additions:
  - every trio of the five trustees: each helper is 1/0 and equals its
    product over its own value;
  - every recipe: the paper and card sums differ by eleven times the
    weighted elevens;
  - planted controls: weights 3, −3, 2 miss, and swapped heights miss 19.
- **`src/main.rs`.** 82 SPOKEN rows and 4 NOT_A_COUNT rows for 14_/15_
  (366 and 26 in all).
- **`docs/storyboard-draft.md`.** Chapters 14 and 15 rewritten: 6 → 8 and
  6 → 7 rows, titles kept. The header's stale "chapters 01 to 03" now says
  01 to 15.
- **`docs/storyboard.md`.** The header says 01 to 15, and the devices note
  and both sections are appended, equal to the draft's rows.
- **`docs/script-draft.tsv`.** The 14_/15_ lines are what the new draft
  generates (12 → 15); nothing else changed.
- **`script.tsv`.** Regenerated by `tools/storyboard.py
  shamir_secret_sharing`: 89 + 15 = 104 lines.
- **`docs/pause-overrides.json`.** The 24 overrides, in root's indented
  format, and the comment "chapters06–15".
- **`docs/brief.md`.** One line reference, "parabola" (09_4, 14_2, 14_8):
  14_8 now says it, and 14_6 does not.

### Checks that actually ran (Python 3.15 `-I -B`, rustfmt; no Cargo)

All ran on the scratch work tree (live + 12/13 + 14/15, both modules in
place) through `run_checks_1415.py`, which saved each command's output in
`docs/previous/ch1415-evidence-20261024/logs/`:
- **`oracle.py`** and **`oracle.py --self-test`**: rc 0, "# 351 facts, 0
  failed cross-checks". The self-test also says "ten-mark dial pairs that
  leak: 9 of 10". The base gives 308 facts, 0 failed.
- **`check_golden.py`** (12/13 tool): 351 golden rows (351 distinct), 351
  oracle rows, `f.len()` asserted 351; diff none.
- **`check_spoken.py`** (12/13 tool): 366 SPOKEN, 26 NOT_A_COUNT. Every
  number word is covered, in 14_/15_ (15 lines, 192 words) and in all 104
  lines (741 words). The planted control was caught. 0 failures.
- **`check_cues_1415.py`**: says in order. The call sites pass the indices
  each line needs (14_6 passes 0 and 14_7 walks 1 and 2; 15_1 and 15_2
  likewise). There are 47 waits (45 distinct) and 31 waits; each occurs
  once in its line, in spoken order, and both sets equal the tests' lists.
  The planted swapped pair and repeated phrase were caught. 0 failures.
- **`check_overrides.py`** (12/13 tool): 73 overrides resolved, 0 errors;
  the planted ambiguous phrases were caught.
- **`director_directions.py`**: the 24 overrides, resolved by the live
  `tools/narration_director.py` `directions` function itself (no plan, no
  audit). 73 authored boundaries, each 14/15 one inside its role's policy
  range. A planted ambiguous override was refused.
- **`check_sb_1415.py`**:
  - production = base's 13 chapters + 14 and 15, each equal to the draft;
  - in the draft only 14 and 15 changed, plus the header;
  - the script lines follow the draft (only 04_3 differs, and it differs in
    the base too);
  - `script.tsv` is the base plus 15 lines;
  - `brief.md` changed in its one reference;
  - "parabola" is said in exactly 09_4, 14_2 and 14_8.
- **`tools/storyboard.py --check shamir_secret_sharing`** (the copied
  tool): rc 0. The advisory estimates are 14: 151.7 s (8 beats) and 15:
  108.5 s (7 beats), 23.5 min over 15 chapters. They are not measured and
  are not a cap.
- **`layout_mirror_1415.py`**: 44 + 54 assertions mirrored from the layout,
  clip, card and needle tests, constants parsed from the Rust, 0 failed.
  Both cards wrap to two lines at 92 down to 72. The planted controls (a
  working line lowered into the band, a weight on its board's name) were
  caught.
- **`evidence_1415.py`**:
  - the new facts.rs block, transliterated statement for statement, gives
    the oracle's value for all 43 facts;
  - the chapters' functions, transliterated, give the facts for all 55
    picture keys;
  - all 1331 recipes: the whole weights give the secret on paper and from
    the cards;
  - controls: spins 7, 6 change 15 of the 43 facts; a nudged card and trio
    2, 4, 5 are refused; 3, −3, 2 give 17 → 6 and 94; the dial weights on
    the heights give 522.
- **`lint_mirror_1415.py`** (a text heuristic, not rustc/Clippy):
  - 0 findings for both modules;
  - it now also treats paths to consts (`super::super::draw::FRAME_Y`) as
    constant, which found one bare constant assert in `three_shares.rs`,
    since moved into `const { }`;
  - 0 findings with root's `one_more_share.rs` on the integrated tree.
- **`range_loops.py`** (heuristic for Clippy `needless_range_loop`): 0. It
  found 15_2's `for i in 1..xs.len()` over `zeros[i]`, now
  `zeros.iter().enumerate().skip(1)`; its planted loop was caught.
- **`rustfmt --check --config-path rustfmt.toml src/main.rs`** on the work
  tree and on the integrated copy: rc 0.
- **Apply tests** as above. A public-hygiene scan found no home paths,
  user names or scratch paths in the new files, the evidence or the logs.

### Not verified and open risks

- Nothing was compiled, so type, borrow and lifetime errors and rustc or
  Clippy warnings beyond the heuristics are possible. The APIs used were
  read from `draw.rs`, `two_keys.rs`, `two_shares.rs`, `envelope.rs`,
  `board.rs`, `a_ten_mark_dial.rs` and explainer_kit, but have not run here.
  The things most worth a look:
  - `tween(vault.needle, 16.0, ..)` reads the needle tracker as marks
    turned, in 15_5, and 14_1's `tween(vault.needle, 0.0, ..)` returns it
    to 0;
  - the closures that borrow `stage` in `.map(..)` to build rings;
  - `stage.prepare(recolor(..)?)` mixed with `play_prepared`.
- No frame was drawn or inspected. The layout is checked only by mirrored
  generous text-width estimates. The glyphs ÷, ×, −, → and "'" appear in
  new contexts and were not probed; ÷ is new to this video.
- Timing:
  - Chapter estimates are advisory.
  - The think, card and door holds rest on the scratch voice's timing
    until a narrated draft exists.
  - The pause ranges are proposals, not measured speech or listening
    acceptance.
- No listening of any kind, and no claim about accent or pacing as heard.

### Evidence files (docs/previous/ch1415-evidence-20261024/, gitignored)

| File | sha256 (first 8) |
|---|---|
| `integration-14-15.patch` | 2982efc4 |
| `sections-14-15.md` | 86a86758 |
| `devices-14-15.md` | ab211849 |
| `apply_sb_1415.py` | 50ccf97f |
| `golden_1415.py` | 3d217b9a |
| `add_overrides_1415.py` | 2f70171f |
| `check_cues_1415.py` | e9735486 |
| `check_sb_1415.py` | f999c90a |
| `layout_mirror_1415.py` | 1a0699a0 |
| `lint_mirror_1415.py` | 91fd1680 |
| `range_loops.py` | 371c896a |
| `evidence_1415.py` | 1277b813 |
| `director_directions.py` | 64fe31bf |
| `make_patch_1415.py` | 1da65c91 |
| `run_checks_1415.py` | 21ebd9fa |
| `logs/` | one file per check above, plus `oracle.out` |

To extract the patch and apply it, after the 12/13 patch:

~~~sh
awk '/^<!-- shamir-1415 patch end -->$/{p=0} p; /^<!-- shamir-1415 patch begin -->$/{p=1}' \
  explainer_corpus/shamir_secret_sharing/docs/WORK_STATUS.md | grep -v '^```' > shamir-1415.patch
git apply --check shamir-1415.patch && git apply shamir-1415.patch
~~~

The same bytes are in
`docs/previous/ch1415-evidence-20261024/integration-14-15.patch`.

<!-- shamir-1415 patch begin -->
```diff
--- a/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/chapters/mod.rs
@@ -1,8 +1,8 @@
 //! One scene per chapter, in storyboard order, with its score intensity
 //! (`0.0..=1.0`). Keys are `NN_name`; narration line ids `NN_k` belong to
 //! the chapter with the same `NN`. Only implemented chapters are registered
-//! (root's direction, Mail 6790): chapters 01 to 13 so far. The other
-//! ten are planned in docs/storyboard-draft.md and join this list, and
+//! (root's direction, Mail 6790): chapters 01 to 15 so far. The other
+//! eight are planned in docs/storyboard-draft.md and join this list, and
 //! docs/storyboard.md, when their scenes are written.
 
 pub mod a_curve_for_three;
@@ -14,6 +14,7 @@
 pub mod draw;
 pub mod envelope;
 pub mod glyphs;
+pub mod helper_curves;
 pub mod one_more_share;
 pub mod padlocks;
 pub mod plot;
@@ -21,6 +22,7 @@
 pub mod the_goal;
 pub mod the_line_on_the_dial;
 pub mod the_vault;
+pub mod three_shares;
 pub mod two_keys;
 pub mod two_shares;
 pub mod why_prime;
@@ -94,5 +96,15 @@
             make: |kit| Box::new(why_prime::WhyPrime { kit }),
             intensity: 0.5,
         },
+        Chapter {
+            key: "14_helper_curves",
+            make: |kit| Box::new(helper_curves::HelperCurves { kit }),
+            intensity: 0.5,
+        },
+        Chapter {
+            key: "15_three_shares",
+            make: |kit| Box::new(three_shares::ThreeShares { kit }),
+            intensity: 0.5,
+        },
     ]
 }
--- a/explainer_corpus/shamir_secret_sharing/src/main.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/main.rs
@@ -552,6 +552,168 @@
             "S-F8.smallest_prime_for_five",
             "7",
         ),
+        ("14_1", "trustees one, two and three", "S-F1.opening_trio", "1,2,3"),
+        (
+            "14_1",
+            "brought their eight, three and one",
+            "S-F1.opening_shares",
+            "8,3,1",
+        ),
+        ("14_1", "They know three points", "S-F1.threshold", "3"),
+        ("14_1", "how can three numbers", "S-F1.threshold", "3"),
+        ("14_1", "give five", "S-F1.secret", "5"),
+        (
+            "14_2",
+            "nineteen, forty-seven and eighty-nine",
+            "S-F9.paper.heights",
+            "19,47,89",
+        ),
+        ("14_2", "Three points pin a parabola down", "S-F1.threshold", "3"),
+        ("14_2", "its height at zero", "S-F4.secret_position", "0"),
+        ("14_3", "Trustee one's helper", "S-F9.helper.1.position", "1"),
+        ("14_3", "is one at", "S-F9.helper.on", "1"),
+        ("14_3", "at position one", "S-F9.helper.1.position", "1"),
+        ("14_3", "and zero at positions", "S-F9.helper.off", "0"),
+        ("14_3", "positions two and three", "S-F9.helper.1.zeros", "2,3"),
+        ("14_3", "Trustee two's", "S-F9.helper.2.position", "2"),
+        ("14_3", "at position two", "S-F9.helper.2.position", "2"),
+        ("14_3", "trustee three's", "S-F9.helper.3.position", "3"),
+        ("14_3", "at position three", "S-F9.helper.3.position", "3"),
+        ("14_3", "each zero", "S-F9.helper.off", "0"),
+        ("14_3", "the other two positions", "S-F9.helper.others", "2"),
+        ("14_4", "nineteen times the first", "S-F9.helper.1.scaled", "19,1"),
+        ("14_4", "forty-seven times the second", "S-F9.helper.2.scaled", "47,2"),
+        ("14_4", "eighty-nine times the third", "S-F9.helper.3.scaled", "89,3"),
+        ("14_4", "At position one", "S-F9.helper.1.position", "1"),
+        (
+            "14_4",
+            "the second and third helpers",
+            "S-F9.sum_at.1.zero_helpers",
+            "2,3",
+        ),
+        ("14_4", "helpers are zero", "S-F9.helper.off", "0"),
+        ("14_4", "the sum there is nineteen", "S-F9.sum_at.1", "19"),
+        ("14_4", "At position two", "S-F9.helper.2.position", "2"),
+        ("14_4", "it's forty-seven", "S-F9.sum_at.2", "47"),
+        ("14_4", "at three", "S-F9.helper.3.position", "3"),
+        ("14_4", "it's eighty-nine", "S-F9.sum_at.3", "89"),
+        ("14_4", "all three points", "S-F1.threshold", "3"),
+        ("14_5", "is zero at both", "S-F9.helper.off", "0"),
+        ("14_5", "positions two and three", "S-F9.helper.1.zeros", "2,3"),
+        (
+            "14_6",
+            "Position minus two, times position minus three",
+            "S-F9.helper.1.zeros",
+            "2,3",
+        ),
+        ("14_6", "the product is zero at both", "S-F9.helper.off", "0"),
+        ("14_6", "At position one", "S-F9.helper.1.position", "1"),
+        (
+            "14_6",
+            "minus one times minus two",
+            "S-F9.helper.1.factors_at_own",
+            "-1,-2",
+        ),
+        ("14_6", "which is two", "S-F9.helper.1.own", "2"),
+        ("14_6", "the helper is one there", "S-F9.helper.on", "1"),
+        ("14_7", "the other two", "S-F9.helper.others", "2"),
+        ("14_7", "Trustee two's", "S-F9.helper.2.position", "2"),
+        (
+            "14_7",
+            "position minus one, times position minus three",
+            "S-F9.helper.2.zeros",
+            "1,3",
+        ),
+        ("14_7", "which is minus one", "S-F9.helper.2.own", "-1"),
+        ("14_7", "at position two", "S-F9.helper.2.position", "2"),
+        ("14_7", "so divide by minus one", "S-F9.helper.2.own", "-1"),
+        ("14_7", "Trustee three's", "S-F9.helper.3.position", "3"),
+        (
+            "14_7",
+            "position minus one, times position minus two",
+            "S-F9.helper.3.zeros",
+            "1,2",
+        ),
+        ("14_7", "which is two", "S-F9.helper.3.own", "2"),
+        ("14_7", "at position three", "S-F9.helper.3.position", "3"),
+        ("14_8", "the dealer's three points", "S-F1.threshold", "3"),
+        ("14_8", "only one parabola does", "S-F6.paper.curves", "1"),
+        ("14_8", "Its height at zero", "S-F4.secret_position", "0"),
+        ("15_1", "at position zero", "S-F4.secret_position", "0"),
+        ("15_1", "Trustee one's", "S-F9.helper.1.position", "1"),
+        (
+            "15_1",
+            "minus two times minus three",
+            "S-F9.helper.1.factors_at_zero",
+            "-2,-3",
+        ),
+        ("15_1", "which is six", "S-F9.helper.1.at_zero", "6"),
+        ("15_1", "so three", "S-F9.helper.1.weight", "3"),
+        ("15_1", "trustee one's weight", "S-F9.helper.1.position", "1"),
+        ("15_1", "counts for at zero", "S-F4.secret_position", "0"),
+        ("15_2", "Trustee two's", "S-F9.helper.2.position", "2"),
+        (
+            "15_2",
+            "minus one times minus three",
+            "S-F9.helper.2.factors_at_zero",
+            "-1,-3",
+        ),
+        ("15_2", "which is three", "S-F9.helper.2.at_zero", "3"),
+        ("15_2", "divided by minus one", "S-F9.helper.2.own", "-1"),
+        ("15_2", "its weight is minus three", "S-F9.helper.2.weight", "-3"),
+        ("15_2", "Trustee three's", "S-F9.helper.3.position", "3"),
+        (
+            "15_2",
+            "minus one times minus two",
+            "S-F9.helper.3.factors_at_zero",
+            "-1,-2",
+        ),
+        ("15_2", "which is two", "S-F9.helper.3.at_zero", "2"),
+        ("15_2", "so one", "S-F9.helper.3.weight", "1"),
+        (
+            "15_2",
+            "the positions, one, two and three",
+            "S-F1.opening_trio",
+            "1,2,3",
+        ),
+        (
+            "15_3",
+            "three times nineteen, minus three times forty-seven, plus one times eighty-nine",
+            "S-F9.paper.weighted",
+            "3,19,-3,47,1,89",
+        ),
+        (
+            "15_3",
+            "fifty-seven, minus a hundred and forty-one, plus eighty-nine",
+            "S-F9.paper.terms",
+            "57,-141,89",
+        ),
+        ("15_3", "which is five", "S-F9.paper.sum", "5"),
+        ("15_4", "eight, three and one", "S-F1.opening_shares", "8,3,1"),
+        (
+            "15_5",
+            "Three times eight, minus three times three, plus one times one",
+            "S-F9.cards.weighted",
+            "3,8,-3,3,1,1",
+        ),
+        ("15_5", "is sixteen", "S-F9.sum_unwrapped", "16"),
+        ("15_5", "sixteen lands on five", "S-F9.sum_lands", "16,5"),
+        ("15_5", "The dial turns to five", "S-F9.secret", "5"),
+        ("15_6", "Nineteen is eight and one eleven", "S-F9.split.1", "19,8,1"),
+        (
+            "15_6",
+            "forty-seven is three and four elevens",
+            "S-F9.split.2",
+            "47,3,4",
+        ),
+        ("15_6", "eighty-nine is one and eight elevens", "S-F9.split.3", "89,1,8"),
+        ("15_7", "sixteen is five and one eleven", "S-F9.sum_split", "16,5,1"),
+        (
+            "15_7",
+            "On the dial, they're three, eight and one",
+            "S-F9.weights_on_dial",
+            "3,8,1",
+        ),
     ];
 
     /// Number words said as an article, not a count, each with its line.
@@ -578,6 +740,10 @@
         ("13_8", "each one only shuffles"),
         ("13_9", "each one has"),
         ("13_9", "the secret is one of its marks"),
+        ("14_2", "so the one through these"),    // the curve, not a count
+        ("14_4", "zero times anything is zero"), // the rule, for any height
+        ("14_6", "Each factor is zero at its own position"), // either factor
+        ("15_7", "wrapping first changes nothing"), // the order, not a count
     ];
 
     const NUMBER_WORDS: &[&str] = &[
--- a/explainer_corpus/shamir_secret_sharing/src/facts.rs
+++ b/explainer_corpus/shamir_secret_sharing/src/facts.rs
@@ -6,7 +6,7 @@
 //! 03, S-F1, S-F6 and S-F12.guess_chance 04, S-F6 and S-F7 05 and 10, S-F3
 //! and S-X1 06, S-F4 07 and 08, S-F5 08, S-F6 09, S-F7.paper, S-F7.join and
 //! S-F7.all_pairs 11, S-F8 12 and 13, S-F9 14 and 15, S-F10 16 to 18, S-F11
-//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 13 are
+//! 19 and 20, S-F12 21 and 22, and S-F13 23. Chapters 01 to 15 are
 //! implemented so far; the rest are computed and checked now so the draft's
 //! numbers are settled before its scenes exist.
 
@@ -706,6 +706,134 @@
         "S-F9.secret",
         model::rebuild(dial, &trio_points).ok_or("the opening trio rebuilds nothing")?,
     )?;
+    {
+        // 14 (native): each helper at the trio's positions, the positions where it is
+        // zero, its factors and its value at its own trustee; the scaled helpers summed
+        // at each trustee's position
+        let xs = OPENING_TRIO;
+        let helper = |i: usize, at: i64| Ratio::new(model::helper_at(&xs, i, at), own[i]);
+        let positions: Vec<i64> = xs.iter().map(|&x| i64::from(x)).collect();
+        let heights: Vec<i64> = tall_trio.iter().map(|p| p.1).collect();
+        for (i, &x) in xs.iter().enumerate() {
+            let n = i + 1;
+            f.add(&format!("S-F9.helper.{n}.position"), x)?;
+            let row: Vec<Ratio> = positions.iter().map(|&p| helper(i, p)).collect();
+            f.add(&format!("S-F9.helper.{n}.at_trio"), csv(&row))?;
+            let zeros: Vec<u32> = xs
+                .iter()
+                .zip(&row)
+                .filter(|&(_, v)| v.whole() == Some(0))
+                .map(|(&y, _)| y)
+                .collect();
+            f.add(&format!("S-F9.helper.{n}.zeros"), csv(&zeros))?;
+            let factors: Vec<i64> = xs
+                .iter()
+                .filter(|&&y| y != x)
+                .map(|&y| i64::from(x) - i64::from(y))
+                .collect();
+            if factors.iter().product::<i64>() != own[i] {
+                return Err(format!(
+                    "helper {n}'s factors {factors:?} do not multiply to {}",
+                    own[i]
+                ));
+            }
+            f.add(&format!("S-F9.helper.{n}.factors_at_own"), csv(&factors))?;
+            f.add(&format!("S-F9.helper.{n}.own"), own[i])?;
+            f.add(&format!("S-F9.helper.{n}.scaled"), csv(&[heights[i], n as i64]))?;
+        }
+        for (i, &p) in positions.iter().enumerate() {
+            let sum = heights
+                .iter()
+                .enumerate()
+                .fold(Ratio::int(0), |acc, (j, &h)| acc.plus(helper(j, p).times(h)));
+            if sum.whole() != Some(heights[i]) {
+                return Err(format!(
+                    "the scaled helpers give {sum} at position {p}, not {}",
+                    heights[i]
+                ));
+            }
+            f.add(&format!("S-F9.sum_at.{}", i + 1), sum)?;
+        }
+        let zero_at_first: Vec<usize> = (0..xs.len())
+            .filter(|&i| helper(i, positions[0]).whole() == Some(0))
+            .map(|i| i + 1)
+            .collect();
+        f.add("S-F9.sum_at.1.zero_helpers", csv(&zero_at_first))?;
+        let (mut on, mut off) = (Vec::new(), Vec::new());
+        for i in 0..xs.len() {
+            for (j, &p) in positions.iter().enumerate() {
+                if i == j {
+                    on.push(helper(i, p));
+                } else {
+                    off.push(helper(i, p));
+                }
+            }
+        }
+        let alike = |values: &[Ratio], what: &str| -> Result<Ratio, String> {
+            match values.first() {
+                Some(&first) if values.iter().all(|&v| v == first) => Ok(first),
+                _ => Err(format!("the helpers {what} are {}", csv(values))),
+            }
+        };
+        f.add("S-F9.helper.on", alike(&on, "at their own trustees")?)?;
+        f.add("S-F9.helper.off", alike(&off, "at the other trustees")?)?;
+        f.add("S-F9.helper.others", xs.len() - 1)?;
+
+        // 15 (native): each helper at position zero, worked as the scene works it; the
+        // weighted heights and cards, the cards' sum wrapped onto the dial, and each height
+        // split into its card and whole elevens
+        let weights = model::ordinary_weights(&xs);
+        for (i, &x) in xs.iter().enumerate() {
+            let n = i + 1;
+            let factors: Vec<i64> = xs.iter().filter(|&&y| y != x).map(|&y| -i64::from(y)).collect();
+            let value: i64 = factors.iter().product();
+            if value != at0[i] || Ratio::new(value, own[i]) != weights[i] {
+                return Err(format!(
+                    "helper {n} at zero is {value}, not weight {} times {}",
+                    weights[i], own[i]
+                ));
+            }
+            f.add(&format!("S-F9.helper.{n}.factors_at_zero"), csv(&factors))?;
+            f.add(&format!("S-F9.helper.{n}.at_zero"), value)?;
+            f.add(&format!("S-F9.helper.{n}.weight"), weights[i])?;
+        }
+        let whole: Vec<i64> = weights
+            .iter()
+            .map(|w| w.whole().ok_or("the opening trio's weights are not whole"))
+            .collect::<Result<_, _>>()?;
+        let cards: Vec<i64> = trio_points.iter().map(|p| i64::from(p.1)).collect();
+        let interleave =
+            |values: &[i64]| -> Vec<i64> { whole.iter().zip(values).flat_map(|(&w, &v)| [w, v]).collect() };
+        f.add("S-F9.paper.weighted", csv(&interleave(&heights)))?;
+        f.add("S-F9.cards.weighted", csv(&interleave(&cards)))?;
+        let share_terms: Vec<i64> = whole.iter().zip(&cards).map(|(w, c)| w * c).collect();
+        f.add("S-F9.share_terms", csv(&share_terms))?;
+        let total: i64 = share_terms.iter().sum();
+        if Ratio::int(total) != trio_total {
+            return Err(format!("the cards' terms add to {total}, not {trio_total}"));
+        }
+        f.add("S-F9.sum_lands", csv(&[total, i64::from(dial.wrap(total))]))?;
+        for (i, (&h, &c)) in heights.iter().zip(&cards).enumerate() {
+            if h != c + m * elevens[i] {
+                return Err(format!("height {h} is not card {c} and {} elevens", elevens[i]));
+            }
+            f.add(&format!("S-F9.split.{}", i + 1), csv(&[h, c, elevens[i]]))?;
+        }
+        let weighted_elevens: i64 = whole.iter().zip(&elevens).map(|(w, e)| w * e).sum();
+        f.add("S-F9.weighted_elevens", weighted_elevens)?;
+        if paper_total != Ratio::int(total + m * weighted_elevens) {
+            return Err(format!(
+                "paper sum {paper_total} is not {total} and {weighted_elevens} elevens"
+            ));
+        }
+        if total.rem_euclid(m) != i64::from(dial.wrap(total)) {
+            return Err(format!("{total} leaves {} over whole elevens", total.rem_euclid(m)));
+        }
+        f.add(
+            "S-F9.sum_split",
+            csv(&[total, total.rem_euclid(m), total.div_euclid(m)]),
+        )?;
+    }
 
     // 16 to 18: fractional weights, and dividing on the dial (trustees 2, 4 and 5)
     let second = dealt(dial, &recipe, &SECOND_TRIO);
@@ -1160,6 +1288,49 @@
 S-F9.sum_unwrapped	16
 S-F9.sum_minus_secret	11
 S-F9.secret	5
+S-F9.helper.1.position	1
+S-F9.helper.1.at_trio	1,0,0
+S-F9.helper.1.zeros	2,3
+S-F9.helper.1.factors_at_own	-1,-2
+S-F9.helper.1.own	2
+S-F9.helper.1.scaled	19,1
+S-F9.helper.2.position	2
+S-F9.helper.2.at_trio	0,1,0
+S-F9.helper.2.zeros	1,3
+S-F9.helper.2.factors_at_own	1,-1
+S-F9.helper.2.own	-1
+S-F9.helper.2.scaled	47,2
+S-F9.helper.3.position	3
+S-F9.helper.3.at_trio	0,0,1
+S-F9.helper.3.zeros	1,2
+S-F9.helper.3.factors_at_own	2,1
+S-F9.helper.3.own	2
+S-F9.helper.3.scaled	89,3
+S-F9.sum_at.1	19
+S-F9.sum_at.2	47
+S-F9.sum_at.3	89
+S-F9.sum_at.1.zero_helpers	2,3
+S-F9.helper.on	1
+S-F9.helper.off	0
+S-F9.helper.others	2
+S-F9.helper.1.factors_at_zero	-2,-3
+S-F9.helper.1.at_zero	6
+S-F9.helper.1.weight	3
+S-F9.helper.2.factors_at_zero	-1,-3
+S-F9.helper.2.at_zero	3
+S-F9.helper.2.weight	-3
+S-F9.helper.3.factors_at_zero	-1,-2
+S-F9.helper.3.at_zero	2
+S-F9.helper.3.weight	1
+S-F9.paper.weighted	3,19,-3,47,1,89
+S-F9.cards.weighted	3,8,-3,3,1,1
+S-F9.share_terms	24,-9,1
+S-F9.sum_lands	16,5
+S-F9.split.1	19,8,1
+S-F9.split.2	47,3,4
+S-F9.split.3	89,1,8
+S-F9.weighted_elevens	-1
+S-F9.sum_split	16,5,1
 S-F10.trio	2,4,5
 S-F10.trio_shares	3,2,6
 S-F10.helper_at_own	6,-2,3
@@ -1237,7 +1408,7 @@
         f.self_check().unwrap();
         let diff = explainer_kit::facts::oracle_diff(&f.print(), ORACLE);
         assert!(diff.is_empty(), "{}", diff.join("\n"));
-        assert_eq!(f.len(), 308);
+        assert_eq!(f.len(), 351);
     }
 
     /// Planted negatives: a different secret, spins or dial changes facts
--- a/explainer_corpus/shamir_secret_sharing/oracle.py
+++ b/explainer_corpus/shamir_secret_sharing/oracle.py
@@ -679,6 +679,67 @@
     put("S-F9.secret", csv(rebuild(OPENING_TRIO, tv)))
     check(all(q.denominator == 1 and int(q) % P == d for q, d in zip(rw, dw)), "15 weights wrap onto the dial weights")
 
+    # 14 (native): each helper is the ordinary curve through 1 at its own trustee and 0 at
+    # the other two (Gaussian elimination, not the product the scene writes); its zeros by
+    # trying every whole position from -20 to 20; the scene's product of factors compared
+    helpers = [paper_curve([(x, 1 if j == i else 0) for j, x in enumerate(OPENING_TRIO)]) for i in range(3)]
+
+    def at(curve, x):
+        return curve[0] + curve[1] * x + curve[2] * x * x
+
+    for i, (x, curve) in enumerate(zip(OPENING_TRIO, helpers)):
+        n = i + 1
+        put(f"S-F9.helper.{n}.position", x)
+        put(f"S-F9.helper.{n}.at_trio", csv(frac(at(curve, y)) for y in OPENING_TRIO))
+        put(f"S-F9.helper.{n}.zeros", csv(y for y in range(-20, 21) if at(curve, y) == 0))
+        factors = [x - y for y in OPENING_TRIO if y != x]
+        put(f"S-F9.helper.{n}.factors_at_own", csv(factors))
+        put(f"S-F9.helper.{n}.own", math.prod(factors))
+        check(all(at(curve, y) * math.prod(factors) == math.prod(y - z for z in OPENING_TRIO if z != x)
+                  for y in range(-20, 21)), f"14 helper {n} is its product of factors over {math.prod(factors)}")
+        put(f"S-F9.helper.{n}.scaled", csv((tall_trio[i], n)))
+    for i, x in enumerate(OPENING_TRIO):
+        value = sum(h * at(curve, x) for h, curve in zip(tall_trio, helpers))
+        put(f"S-F9.sum_at.{i + 1}", frac(value))
+        check(value == tall_trio[i], f"14 the scaled helpers' sum passes through trustee {x}'s point")
+    put("S-F9.sum_at.1.zero_helpers", csv(j + 1 for j, curve in enumerate(helpers) if at(curve, OPENING_TRIO[0]) == 0))
+    on = {at(curve, x) for x, curve in zip(OPENING_TRIO, helpers)}
+    off = {at(curve, y) for x, curve in zip(OPENING_TRIO, helpers) for y in OPENING_TRIO if y != x}
+    check(len(on) == 1 and len(off) == 1, "14 every helper is on at its own trustee and off at the others alike")
+    put("S-F9.helper.on", frac(min(on)))
+    put("S-F9.helper.off", frac(min(off)))
+    put("S-F9.helper.others", sum(1 for y in OPENING_TRIO if y != OPENING_TRIO[0]))
+    sum_curve = tuple(sum(h * c[k] for h, c in zip(tall_trio, helpers)) for k in range(3))
+    check(sum_curve == (SECRET, *SPINS) and at(sum_curve, 0) == SECRET, "14 the scaled helpers' sum is the dealer's curve")
+    check(paper_curve(list(zip(OPENING_TRIO, tall_trio))) == (SECRET, *SPINS), "14 one parabola through the three points")
+
+    # 15 (native): each helper read at position zero, as the scene works it, against the weight
+    # elimination found; the weighted heights and cards, the sum on the cards wound round the
+    # dial, and each height split into its card and whole elevens by winding
+    for i, x in enumerate(OPENING_TRIO):
+        n = i + 1
+        factors = [0 - y for y in OPENING_TRIO if y != x]
+        put(f"S-F9.helper.{n}.factors_at_zero", csv(factors))
+        put(f"S-F9.helper.{n}.at_zero", math.prod(factors))
+        put(f"S-F9.helper.{n}.weight", frac(rw[i]))
+        check(Fraction(math.prod(factors), math.prod(x - y for y in OPENING_TRIO if y != x)) == rw[i] == at(helpers[i], 0),
+              f"15 helper {n} at zero is its weight")
+    put("S-F9.paper.weighted", csv(frac(v) for q, h in zip(rw, tall_trio) for v in (q, h)))
+    put("S-F9.cards.weighted", csv(frac(v) for q, y in zip(rw, tv) for v in (q, y)))
+    put("S-F9.share_terms", csv(frac(q * y) for q, y in zip(rw, tv)))
+    turns, mark = wind(int(total))
+    check(total.denominator == 1 and mark == SECRET, "15 the sum on the cards lands on the secret")
+    put("S-F9.sum_lands", csv((frac(total), mark)))
+    splits = [wind(h) for h in tall_trio]
+    for i, (h, (t, m)) in enumerate(zip(tall_trio, splits)):
+        check(m == tv[i] and h == m + t * P, f"15 height {h} is card {tv[i]} and whole elevens")
+        put(f"S-F9.split.{i + 1}", csv((h, m, t)))
+    weighted_elevens = sum(q * t for q, (t, _) in zip(rw, splits))
+    put("S-F9.weighted_elevens", frac(weighted_elevens))
+    check(paper_total == total + P * weighted_elevens, "15 the paper sum is the cards' sum and the weighted elevens")
+    put("S-F9.sum_split", csv((frac(total), mark, turns)))
+    check(turns == -weighted_elevens, "15 the cards' sum is the secret and one eleven: the elevens cancel")
+
     # 16 to 18: fractional weights, and dividing on the dial (trustees 2, 4 and 5)
     sv = [shares[x - 1] for x in SECOND_TRIO]
     put("S-F10.trio", csv(SECOND_TRIO))
@@ -927,6 +988,28 @@
         check(int(sum(q * y for q, y in zip(w_open, dealt_))) % P == r[0], f"whole weights on shares {r}")
     check(any(sum(q * share(r, x) for q, x in zip(w_second, SECOND_TRIO)).denominator != 1 for r in recipes()),
           "negative control: fractional weights on shares are not always whole")
+    # chapter 14: for every three of the five trustees, each helper found by elimination is
+    # 1 at its own trustee, 0 at the other two, and its product of factors over its own value;
+    # chapter 15: the paper sum and the cards' sum differ by eleven times the weighted elevens
+    for trio in itertools.combinations(TRUSTEES, 3):
+        for i, x in enumerate(trio):
+            c = paper_curve([(y, 1 if j == i else 0) for j, y in enumerate(trio)])
+            own = math.prod(x - z for z in trio if z != x)
+            check([c[0] + c[1] * y + c[2] * y * y for y in trio] == [1 if y == x else 0 for y in trio]
+                  and all((c[0] + c[1] * y + c[2] * y * y) * own == math.prod(y - z for z in trio if z != x)
+                          for y in range(-10, 11)), f"helper {i + 1} of {trio}")
+    for r in recipes():
+        tall = [r[0] + r[1] * x + r[2] * x * x for x in OPENING_TRIO]
+        dealt_ = [share(r, x) for x in OPENING_TRIO]
+        gap = sum(q * h for q, h in zip(w_open, tall)) - sum(q * y for q, y in zip(w_open, dealt_))
+        check(gap == P * sum(q * ((h - y) // P) for q, h, y in zip(w_open, tall, dealt_)), f"whole elevens {r}")
+    # negative control 5: a nudged weight, or a helper scaled by the wrong height, misses
+    nudged_weights = (3, -3, 2)
+    check(sum(q * y for q, y in zip(nudged_weights, (8, 3, 1))) % P != SECRET,
+          "negative control: weights 3, -3, 2 on the trio's cards miss the secret")
+    swapped = [paper_curve([(x, 1 if j == i else 0) for j, x in enumerate(OPENING_TRIO)]) for i in range(3)]
+    wrong = sum(h * (c[0] + c[1] + c[2]) for h, c in zip((47, 19, 89), swapped))
+    check(wrong != 19, "negative control: heights 47 and 19 swapped, the sum at position 1 is not 19")
     # chapter 22: among five shares with one lie, exactly one curve fits four of them,
     # the dealer's; four shares with one lie fit no curve (every recipe, liar and lie)
     tables = {g: {tuple(share(r, x) for x in g): r for r in recipes()} for g in itertools.combinations(TRUSTEES, 3)}
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard.md
@@ -1,7 +1,7 @@
 # Storyboard — Three Keys Open the Secret. Two Reveal Nothing.
 
 The production storyboard holds only the chapters that exist as native scenes:
-chapters 01 to 13, per root's direction in Mail
+chapters 01 to 15, per root's direction in Mail
 6790. `python3 -I tools/storyboard.py shamir_secret_sharing` checks this file
 (G2) and writes `script.tsv` from its words column. The tests check that file
 and the chapter registry against it.
@@ -286,3 +286,62 @@
 | 13_7 | Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero. | Under the cards, one line at a time at its phrase: at "Landing on zero means" "a × b lands on 0 exactly when a × b is a multiple of 11"; at "Every whole number splits into primes" "each number splits into primes one way only, so 11 is" and at "so a product of marks has eleven" "among a × b's primes only if it is among a's or b's"; at "No mark from one to ten does" "a and b from 1 to 10: no 11 among their primes" [S-F8.eleven.nonzero_range]; at "So on eleven marks, a product lands on zero" "so on 11 marks, a × b lands on 0 only if a or b is 0", and a gold outline glows round it. The cards and the proof stay until they can be read. | derivation | thought after "a multiple of eleven."; thought after "from one to ten does." |
 | 13_8 | So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do. | The cards and the proof's first lines go; its last line moves to the top. At "if two marks gave the same answer" under it "5 × u and 5 × w land on the same mark"; at "five times their difference would land on zero" "so 5 × (u − w) lands on 0"; at "unless they're the same mark" "5 isn't 0, so u − w lands on 0:" and "u and w are the same mark" [S-F8.multiplier]. At "every multiplier from one to ten" the rows, arrows, counts and notes go and, on the same columns, lower right, "on 11 marks, the marks reached by × 1 to × 10": for each multiplier "× 1" to "× 10" a row of cells, yellow at every mark it reaches, every row full [S-F8.eleven.reach, S-F8.eleven.rows_that_shuffle]; at "each one only shuffles the eleven marks" an outline glows round them all. At "On ten marks" upper right "on 10 marks, the marks reached by × 1 to × 9", the cells it misses red [S-F8.ten.reach]; at "only one, three, seven and nine do" outlines glow round those four full rows [S-F8.ten.rows_that_shuffle]. Held for looking. | derivation, grid | thought after "the same mark."; hold on the rows (1.5 s) |
 | 13_9 | That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do. | At "Any prime number of marks works" a precisely card: "the counting holds on any prime number of marks above both the number of trustees and the secret (here: 7, 11, 13 and on; Shamir 1979)" [S-F8.primes_for_five, S-F8.smallest_prime_for_five, S-X1.shamir.year]. At "each one has a position besides zero" rings glow round the rail's five discs [S-F1.trustees, S-F4.secret_position]. The card stays until it can be read. | precisely card | chapter_transition; hold until the card is read |
+
+## Devices in chapters 14 and 15
+
+- **The opening callback** (14_1): chapter 04's door, opened by the trio's
+  cards 8, 3 and 1 and shut again, then the trio's three points on chapter
+  09's dial board with "spins: ? and ?" under it: three points, no curve.
+- **The paper plot** (14_2 to 15_3): chapter 09's ordinary-number plot cut
+  to positions 0 to 3 and heights 0 to 100, with the trio's points 19, 47
+  and 89, the dealer's parabola in ink and a gold "?" where it meets the
+  side at position 0.
+- **The helper boards** (14_3 to 15_5): three small boards at the right,
+  one per trustee of the trio, one over another, their columns under the
+  same position numbers 0 to 3 and their sides running from −3 to 3. Each
+  helper is first only its three dots, 1 at its own trustee and 0 at the
+  other two; its curve comes later, built from its product of factors. The
+  product is drawn in ink and gives way to the helper in teal when it is
+  halved or divided by −1, so the scaling is a visible change of one curve.
+- **The column outline** (14_4, 15_1): one outline down all three boards at
+  one position, moving from position 1 to 2 to 3 as each sum is written,
+  and in gold at position 0 while the weights are read.
+- **The scales and terms** (14_4, 15_3): "× 19", "× 47" and "× 89" right of
+  the boards; in 15_3 each weight comes over from the board's side to stand
+  in front of its scale, and the term follows: "3 × 19 = 57".
+- **The weights on the rail** (15_1 to 15_7): "weight 3", "weight −3" and
+  "weight 1" under the trio's discs; at the end "weight −3" gives way to
+  "weight 8", the same weight on the dial.
+- **The working** under the boards, left-aligned, one line at a time,
+  always above the lower third's band where the precisely cards come up;
+  14_8 clears it before its card, and 15_7 keeps both sums in view beside
+  its card.
+- **The needle's run** (15_5): the vault returns and its needle turns
+  through all sixteen marks of the cards' sum, once round and on to 5.
+- Emphasis is by rings, outlines and strokes of their own. No typeset text
+  changes color or size.
+
+## 14_helper_curves: building a curve through three points, on paper
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 14_1 | Back at the start, trustees one, two and three brought their eight, three and one, and the door opened. How they did it was left for later. They know three points of the dealer's curve, but not its spins. So how can three numbers give five? | The rail, trustees 1, 2 and 3 teal with their cards 8, 3 and 1 in their frames [S-F1.opening_trio, S-F1.opening_shares], 4 and 5 grey with empty frames; the vault at the upper left as in chapter 04, door shut, needle on 0. At "brought their eight, three and one" teal outlines glow round the three cards; at "and the door opened" the needle turns to 5 and the door turns green and swells [S-F1.secret]. At "How they did it was left for later" the door shuts and the needle goes back to 0. At "They know three points" the dial board at the right, "on the dial: points only, no curve", and the trio's points (1, 8), (2, 3) and (3, 1) come onto it in teal, a ring glowing round each [S-F1.threshold, S-F1.opening_shares]; at "but not its spins" yellow "spins: ? and ?" under the board [S-F6.spin_count]. At "So how can three numbers give five" a gold ring glows round the vault's dial [S-F1.secret]. | callback, board, outline glow | thought after "the door opened."; cue_transition after the question (rhetorical: a beat, no think time) |
+| 14_2 | To see the idea, go back to paper, where nothing wraps, and pretend for now that they knew their heights there: nineteen, forty-seven and eighty-nine. Three points pin a parabola down, so the one through these is the dealer's curve, and its height at zero is the secret. The trick is to build that curve out of simpler pieces. | The vault, the board and the spins' note go; the rail stays. At "go back to paper" the paper plot at the left, "on paper: ordinary numbers", positions 0 to 3 under it and heights 0 to 100 up the side, labelled every 20. At "nineteen", "forty-seven" and "eighty-nine" the trio's points come in one by one, teal, labelled 19, 47 and 89 [S-F9.paper.heights], a teal ring glowing round each trustee's disc on the rail as its point appears. At "Three points pin a parabola down" the dealer's parabola draws in ink through the three points, as in chapter 09, from position 0 to just past 3 [S-F1.threshold, S-F6.paper.curves]; at "its height at zero is the secret" a gold "?" just above where it meets the side [S-F4.secret_position]. | plot, lagged points, curve | clause after "where nothing wraps,"; thought after "is the secret." |
+| 14_3 | Give each trustee a helper curve. Trustee one's helper is one at position one, and zero at positions two and three. Trustee two's is one at position two, and trustee three's is one at position three, each zero at the other two positions. A helper answers to its own trustee only. | At "Give each trustee a helper curve" three small paper boards at the right, one over another, each named on its left, "trustee 1's helper", "trustee 2's helper" and "trustee 3's helper"; the positions 0 to 3 over the top board, every board's columns under the same numbers, each board's side running from −3 to 3 with its 0 line across and a tick at 1. At "Trustee one's helper is one at position one" a teal dot at height 1 over position 1 on the top board, labelled "1" [S-F9.helper.1.position, S-F9.helper.on]; at "and zero at positions two and three" grey dots on its 0 line over positions 2 and 3, labelled "0" [S-F9.helper.1.zeros, S-F9.helper.off, S-F9.helper.1.at_trio]. At "Trustee two's is one at position two" the middle board's dots, 0, 1 and 0 [S-F9.helper.2.at_trio]; at "trustee three's is one at position three" the bottom board's, 0, 0 and 1 [S-F9.helper.3.at_trio, S-F9.helper.others]. No helper is drawn as a curve yet, only what each has to do at the three positions. At "answers to its own trustee only" teal rings glow round the three 1 dots together, a diagonal down the boards. | small multiples, points, outline glow | thought after "helper curve."; thought after "two and three." |
+| 14_4 | Now scale each helper by its trustee's height: nineteen times the first, forty-seven times the second, and eighty-nine times the third, and add them. At position one, the second and third helpers are zero, and zero times anything is zero, so the sum there is nineteen. At position two it's forty-seven, and at three it's eighty-nine. The sum passes through all three points. | At "nineteen times the first" a teal "× 19" right of the top board, then "× 47" right of the middle one and "× 89" right of the bottom one, each as it is said [S-F9.helper.1.scaled, S-F9.helper.2.scaled, S-F9.helper.3.scaled]. At "At position one" a teal outline round position 1's column, down all three boards; at "zero times anything is zero" grey rings glow round its two 0 dots [S-F9.sum_at.1.zero_helpers]; at "so the sum there is nineteen" under the boards "at 1: 19 × 1 + 47 × 0 + 89 × 0 = 19", and a teal ring glows round the plot's point (1, 19) [S-F9.sum_at.1]. At "At position two" the outline moves to position 2's column and "at 2: 19 × 0 + 47 × 1 + 89 × 0 = 47" comes under the first line, a ring round (2, 47) [S-F9.sum_at.2]; at "and at three" it moves to position 3 with "at 3: 19 × 0 + 47 × 0 + 89 × 1 = 89", a ring round (3, 89) [S-F9.sum_at.3]. The three lines stay, their 1s on a diagonal. At "passes through all three points" the plot's three points swell together [S-F1.threshold]. | linked views, moving outline | thought after "and add them."; thought after "the sum there is nineteen." |
+| 14_5 | But can a helper like that be built? What simple recipe is zero at both positions two and three? ⟨predict⟩ | The column outline and the three lines go; the dots on the middle and bottom boards dim. At "What simple recipe" grey rings glow round the top board's two 0 dots [S-F9.helper.1.zeros, S-F9.helper.off]. No curve yet: the think ring with its "?" at the lower right. | predict | thought after "be built?"; question hold (think time) |
+| 14_6 | Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's minus one times minus two, which is two, so halve it, and the helper is one there. On paper, halving is no trouble. | The "?" goes. At "Position minus two, times position minus three" under the boards "trustee 1's helper: (position − 2) × (position − 3)" [S-F9.helper.1.zeros]; at "Each factor is zero at its own position" teal rings glow round the top board's 0 dots; at "so the product is zero at both" the product's curve draws in ink on the top board through both 0 dots, climbing toward position 0 until it leaves the board's top [S-F9.helper.off]. At "At position one it's minus one times minus two" under that line "at 1: (1 − 2) × (1 − 3) = (−1) × (−2)" [S-F9.helper.1.position, S-F9.helper.1.factors_at_own]; at "which is two" "= 2" after it and a ring where the curve passes height 2 over position 1 [S-F9.helper.1.own]. At "so halve it" "÷ 2" after the first line, and the curve gives way to one half as high everywhere, teal, through the 1 dot and both 0 dots, meeting the side at 3 [S-F9.helper.1.at_trio, S-F9.helper.on]. At "On paper, halving is no trouble" a small grey "on paper, halving is fine" under the lines. | derivation, controlled change | thought after "times position minus three."; thought after "zero at both." |
+| 14_7 | The same steps make the other two. Trustee two's is position minus one, times position minus three, which is minus one at position two, so divide by minus one. Trustee three's is position minus one, times position minus two, which is two at position three, so halve it. | Trustee 1's lines go and the dots on the middle and bottom boards brighten again [S-F9.helper.others]. At "Trustee two's is position minus one, times position minus three" under the boards "trustee 2's helper: (position − 1) × (position − 3)" [S-F9.helper.2.position, S-F9.helper.2.zeros] and its product's curve in ink on the middle board, through its 0 dots and down to −1 over position 2; at "which is minus one at position two" "at 2: (2 − 1) × (2 − 3) = −1" under it and a ring where the curve passes −1 [S-F9.helper.2.factors_at_own, S-F9.helper.2.own]; at "so divide by minus one" "÷ (−1)" after the first line, and the curve gives way to its mirror image, teal, up through the 1 dot [S-F9.helper.2.at_trio]. At "Trustee three's is position minus one, times position minus two" "trustee 3's helper: (position − 1) × (position − 2)" [S-F9.helper.3.position, S-F9.helper.3.zeros] and its product's curve on the bottom board, 2 over position 3; at "which is two at position three" "at 3: (3 − 1) × (3 − 2) = 2" and a ring where the curve passes 2 [S-F9.helper.3.factors_at_own, S-F9.helper.3.own]; at "so halve it" "÷ 2" and the curve gives way to one half as high, through the 1 dot [S-F9.helper.3.at_trio]. Each helper is now a curve, 1 at its own trustee and 0 at the other two. | derivation, controlled change | thought after "the other two."; thought after "divide by minus one." |
+| 14_8 | Multiplied out, each helper is a parabola, and so is their weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself. Its height at zero is the secret. | The lines under the boards go. At "so is their weighted sum" 19 times the top helper, plus 47 times the middle one, plus 89 times the bottom one draws in teal on the paper plot, from position 0 to just past 3, along the ink parabola [S-F9.helper.1.scaled, S-F9.helper.2.scaled, S-F9.helper.3.scaled]; at "passes through the dealer's three points" rings glow round the three points [S-F1.threshold, S-F9.sum_at.1, S-F9.sum_at.2, S-F9.sum_at.3]; at "only one parabola does" the ink parabola under it turns teal too, the two one curve all the way [S-F6.paper.curves]. At "Its height at zero is the secret" a gold ring glows where the curve meets the side and a gold outline round the "?" [S-F4.secret_position], and a precisely card: "three points pin one parabola (chapter 09), so 19 × helper 1 + 47 × helper 2 + 89 × helper 3 is the dealer's curve; at 0 it is the secret". The card stays until it can be read. | curve, linked views, precisely card | chapter_transition; hold until the card is read |
+
+## 15_three_shares: the door, explained
+
+| id | words | picture | features | hold/listen |
+|---|---|---|---|---|
+| 15_1 | So read each helper at position zero. Trustee one's is minus two times minus three, which is six, halved, so three. That's trustee one's weight: how much their height counts for at zero. | As chapter 14 ended: the paper plot with the trio's three points, the teal sum on the parabola and the gold "?" at position 0; the three helper boards with their curves and dots, and "× 19", "× 47" and "× 89"; the rail, 1, 2 and 3 teal with 8, 3 and 1. At "read each helper at position zero" a gold outline round position 0's column, down all three boards [S-F4.secret_position]. At "Trustee one's is minus two times minus three" under the boards "trustee 1 at 0: (0 − 2) × (0 − 3)" [S-F9.helper.1.position, S-F9.helper.1.factors_at_zero]; at "which is six, halved, so three" "= 6, ÷ 2: 3" after it [S-F9.helper.1.at_zero, S-F9.helper.1.own, S-F9.helper.1.weight], a gold dot where the top curve meets position 0, at height 3, and a teal "3" beside the board's side. At "That's trustee one's weight" a teal "weight 3" under trustee 1's disc, a ring glowing round the disc. | derivation, linked views | thought after "so three." |
+| 15_2 | Trustee two's is minus one times minus three, which is three, divided by minus one, so its weight is minus three. Trustee three's is minus one times minus two, which is two, halved, so one. The weights use only the positions, one, two and three, which everyone knows. | At "Trustee two's is minus one times minus three" "trustee 2 at 0: (0 − 1) × (0 − 3)" under the first line [S-F9.helper.2.position, S-F9.helper.2.factors_at_zero]; at "which is three, divided by minus one" "= 3, ÷ (−1): −3" after it [S-F9.helper.2.at_zero, S-F9.helper.2.own, S-F9.helper.2.weight], a gold dot at height −3 over position 0 on the middle board with a teal "−3" beside it, and "weight −3" under trustee 2's disc, a ring glowing round it. At "Trustee three's is minus one times minus two" "trustee 3 at 0: (0 − 1) × (0 − 2)" [S-F9.helper.3.position, S-F9.helper.3.factors_at_zero]; at "which is two, halved, so one" "= 2, ÷ 2: 1" [S-F9.helper.3.at_zero, S-F9.helper.3.own, S-F9.helper.3.weight], a gold dot at height 1 on the bottom board with "1", and "weight 1" under trustee 3's disc. At "use only the positions" teal outlines glow round the position numbers 1, 2 and 3 over the top board and round the three weights on the rail [S-F1.opening_trio, S-F9.weights]. | derivation, outline glow | thought after "so its weight is minus three."; thought after "so one." |
+| 15_3 | So on paper, the secret is three times nineteen, minus three times forty-seven, plus one times eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five. | The three lines under the boards and the gold column outline go. At "three times nineteen" the top board's weight comes over from beside its side to stand in front of "× 19", which makes room: "3 × 19"; at "minus three times forty-seven" the middle one's, "−3 × 47"; at "plus one times eighty-nine" the bottom one's, "1 × 89" [S-F9.paper.weighted]. At "That's fifty-seven" "= 57" after the first, at "minus a hundred and forty-one" "= −141" after the second, and at "plus eighty-nine" "= 89" after the third [S-F9.paper.terms], with "on paper: 57 − 141 + 89" under the boards. At "which is five" a gold "= 5" after it, and on the plot the "?" gives way to a gold point and a gold "5"; the point swells [S-F9.paper.sum, S-F1.secret]. | derivation, controlled change | clause after "minus three times forty-seven,"; cue_transition after "which is five." |
+| 15_4 | But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. Will the same weights still work on what they hold? ⟨predict⟩ | At "never see those heights" the paper plot and the terms beside the boards go; the line "on paper: 57 − 141 + 89 = 5" stays under the boards. At "eight, three and one" teal outlines glow round the rail's three cards [S-F1.opening_shares]; at "Will the same weights" teal outlines glow round the three weights under the discs [S-F9.weights]. The think ring with its "?" at the upper left, where the plot was. | predict | thought after "never see those heights."; question hold (think time) |
+| 15_5 | Yes. Three times eight, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before. | The "?" goes. At "Three times eight, minus three times three, plus one times one" under the paper line "on the cards: 3 × 8 − 3 × 3 + 1 × 1" [S-F9.cards.weighted]; at "is sixteen" "= 24 − 9 + 1 = 16" after it [S-F9.share_terms, S-F9.sum_unwrapped]; at "sixteen lands on five" a gold "→ 5" [S-F9.sum_lands]. At "The dial turns to five" the vault at the upper left as in chapter 04, and its needle runs sixteen marks, once round and on to 5 [S-F9.sum_split, S-F9.secret]; at "the door opens" the door turns green and swells. The open door holds. | derivation, callback | thought after "lands on five."; hold on the open door |
+| 15_6 | Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens. | The helper boards go; the vault stays open. At "Nineteen is eight and one eleven" at the upper right "19 = 8 + 1 × 11", and a teal outline glows round trustee 1's card [S-F9.split.1]; at "forty-seven is three and four elevens" "47 = 3 + 4 × 11" under it, an outline round trustee 2's card [S-F9.split.2]; at "eighty-nine is one and eight elevens" "89 = 1 + 8 × 11", an outline round trustee 3's [S-F9.split.3, S-F9.paper.elevens]. | derivation, outline glow | thought after "Here's why."; connected list (short clauses after "one eleven," and "four elevens,") |
+| 15_7 | Whole weights turn whole elevens into whole elevens. So the sum on paper and the sum on the cards can differ only by whole elevens, and sixteen is five and one eleven. That's why wrapping first changes nothing, as long as the weights are whole. On the dial, they're three, eight and one. | At "Whole weights turn whole elevens into whole elevens" under the three rows "3 × (1 × 11) − 3 × (4 × 11) + 1 × (8 × 11) = −1 × 11" [S-F9.weights, S-F9.paper.elevens, S-F9.weighted_elevens]. At "can differ only by whole elevens" teal outlines glow round the sum on paper and the sum on the cards; at "sixteen is five and one eleven" "16 = 5 + 1 × 11" under them [S-F9.sum_split, S-F9.sum_minus_secret]. At "On the dial, they're three, eight and one" trustee 2's "weight −3" gives way to "weight 8", −3 landing on 8, and outlines glow round the three weights [S-F9.weights_on_dial]; then a precisely card: "wrapped before the sum or after, whole weights land on the same mark; on the dial they are 3, 8 and 1 (Lagrange interpolation)". The card stays until it can be read. | derivation, precisely card | thought after "into whole elevens."; chapter_transition; hold until the card is read |
--- a/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/storyboard-draft.md
@@ -2,7 +2,7 @@
 
 The full planned film: every chapter's spoken words and pictures, 23
 chapters. `docs/storyboard.md` (with `script.tsv` and the chapter registry)
-holds only the chapters that exist as native scenes, chapters 01 to 03
+holds only the chapters that exist as native scenes, chapters 01 to 15
 (root's direction, Mail 6790); a chapter moves there when its scene is
 written. `docs/script-draft.tsv` is the script `tools/storyboard.py`
 generated from this full plan. Revision 2 before the wording fixes below is
@@ -327,23 +327,26 @@
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 14_1 | Now the door from the start. Trustees one, two and three hold eight, three and one, and those three numbers somehow give five. To see why, go back to paper first, where the dealer's curve had heights nineteen, forty-seven and eighty-nine. | The rail with 1, 2, 3 lit teal and their cards 8, 3, 1 [S-F1.opening_shares]. Each card rises into the paper plot of 09_4 as its unwrapped point (1, 19), (2, 47), (3, 89) [S-F9.paper.heights]; position 0 has an empty gold "?". | callback, plot | thought after "the door from the start." |
-| 14_2 | On paper, three points pin a parabola down, as we saw. So build that one parabola from three helper curves. Trustee one's helper is one at position one, and zero at positions two and three. The other two helpers do the same for their own trustees. | Three small paper boards, one helper each: a faint curve that is 1 at its own trustee and 0 at the other two. | curves | thought after "helper curves." |
-| 14_3 | Then nineteen times the first helper, plus forty-seven times the second, plus eighty-nine times the third, passes through all three points. At each trustee, only that trustee's helper is switched on. | The helpers scaled by 19, 47, 89 stack into one curve through the three teal points; at each point only one helper is nonzero (highlighted). | morph | thought after "all three points." |
-| 14_4 | ⟨predict⟩ What simple recipe is zero at both positions two and three? | Trustee one's board with "0" marked at positions 2 and 3 and no curve yet. | predict | question hold (think time) |
-| 14_5 | Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's two, so halve it, and the helper is one there. On paper, halving is no trouble. | TeX: (x − 2)(x − 3); each factor's zero marked; at x = 1: (−1)(−2) = 2 [S-F9.helper_at_own]; ÷ 2 → 1. The helper curve drawn on paper. | derivation | thought after "times position minus three." |
-| 14_6 | Multiplied out, each helper is a parabola too, and so is the weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself. | The weighted sum lands exactly on the dealer's parabola from 09_4, which runs on to the gold 5 at position 0 [S-F1.secret]. Card: "three points → one parabola (09_4)". | morph, card | chapter_transition |
+| 14_1 | Back at the start, trustees one, two and three brought their eight, three and one, and the door opened. How they did it was left for later. They know three points of the dealer's curve, but not its spins. So how can three numbers give five? | The rail, trustees 1, 2 and 3 teal with their cards 8, 3 and 1 in their frames [S-F1.opening_trio, S-F1.opening_shares], 4 and 5 grey with empty frames; the vault at the upper left as in chapter 04, door shut, needle on 0. At "brought their eight, three and one" teal outlines glow round the three cards; at "and the door opened" the needle turns to 5 and the door turns green and swells [S-F1.secret]. At "How they did it was left for later" the door shuts and the needle goes back to 0. At "They know three points" the dial board at the right, "on the dial: points only, no curve", and the trio's points (1, 8), (2, 3) and (3, 1) come onto it in teal, a ring glowing round each [S-F1.threshold, S-F1.opening_shares]; at "but not its spins" yellow "spins: ? and ?" under the board [S-F6.spin_count]. At "So how can three numbers give five" a gold ring glows round the vault's dial [S-F1.secret]. | callback, board, outline glow | thought after "the door opened."; cue_transition after the question (rhetorical: a beat, no think time) |
+| 14_2 | To see the idea, go back to paper, where nothing wraps, and pretend for now that they knew their heights there: nineteen, forty-seven and eighty-nine. Three points pin a parabola down, so the one through these is the dealer's curve, and its height at zero is the secret. The trick is to build that curve out of simpler pieces. | The vault, the board and the spins' note go; the rail stays. At "go back to paper" the paper plot at the left, "on paper: ordinary numbers", positions 0 to 3 under it and heights 0 to 100 up the side, labelled every 20. At "nineteen", "forty-seven" and "eighty-nine" the trio's points come in one by one, teal, labelled 19, 47 and 89 [S-F9.paper.heights], a teal ring glowing round each trustee's disc on the rail as its point appears. At "Three points pin a parabola down" the dealer's parabola draws in ink through the three points, as in chapter 09, from position 0 to just past 3 [S-F1.threshold, S-F6.paper.curves]; at "its height at zero is the secret" a gold "?" just above where it meets the side [S-F4.secret_position]. | plot, lagged points, curve | clause after "where nothing wraps,"; thought after "is the secret." |
+| 14_3 | Give each trustee a helper curve. Trustee one's helper is one at position one, and zero at positions two and three. Trustee two's is one at position two, and trustee three's is one at position three, each zero at the other two positions. A helper answers to its own trustee only. | At "Give each trustee a helper curve" three small paper boards at the right, one over another, each named on its left, "trustee 1's helper", "trustee 2's helper" and "trustee 3's helper"; the positions 0 to 3 over the top board, every board's columns under the same numbers, each board's side running from −3 to 3 with its 0 line across and a tick at 1. At "Trustee one's helper is one at position one" a teal dot at height 1 over position 1 on the top board, labelled "1" [S-F9.helper.1.position, S-F9.helper.on]; at "and zero at positions two and three" grey dots on its 0 line over positions 2 and 3, labelled "0" [S-F9.helper.1.zeros, S-F9.helper.off, S-F9.helper.1.at_trio]. At "Trustee two's is one at position two" the middle board's dots, 0, 1 and 0 [S-F9.helper.2.at_trio]; at "trustee three's is one at position three" the bottom board's, 0, 0 and 1 [S-F9.helper.3.at_trio, S-F9.helper.others]. No helper is drawn as a curve yet, only what each has to do at the three positions. At "answers to its own trustee only" teal rings glow round the three 1 dots together, a diagonal down the boards. | small multiples, points, outline glow | thought after "helper curve."; thought after "two and three." |
+| 14_4 | Now scale each helper by its trustee's height: nineteen times the first, forty-seven times the second, and eighty-nine times the third, and add them. At position one, the second and third helpers are zero, and zero times anything is zero, so the sum there is nineteen. At position two it's forty-seven, and at three it's eighty-nine. The sum passes through all three points. | At "nineteen times the first" a teal "× 19" right of the top board, then "× 47" right of the middle one and "× 89" right of the bottom one, each as it is said [S-F9.helper.1.scaled, S-F9.helper.2.scaled, S-F9.helper.3.scaled]. At "At position one" a teal outline round position 1's column, down all three boards; at "zero times anything is zero" grey rings glow round its two 0 dots [S-F9.sum_at.1.zero_helpers]; at "so the sum there is nineteen" under the boards "at 1: 19 × 1 + 47 × 0 + 89 × 0 = 19", and a teal ring glows round the plot's point (1, 19) [S-F9.sum_at.1]. At "At position two" the outline moves to position 2's column and "at 2: 19 × 0 + 47 × 1 + 89 × 0 = 47" comes under the first line, a ring round (2, 47) [S-F9.sum_at.2]; at "and at three" it moves to position 3 with "at 3: 19 × 0 + 47 × 0 + 89 × 1 = 89", a ring round (3, 89) [S-F9.sum_at.3]. The three lines stay, their 1s on a diagonal. At "passes through all three points" the plot's three points swell together [S-F1.threshold]. | linked views, moving outline | thought after "and add them."; thought after "the sum there is nineteen." |
+| 14_5 | But can a helper like that be built? What simple recipe is zero at both positions two and three? ⟨predict⟩ | The column outline and the three lines go; the dots on the middle and bottom boards dim. At "What simple recipe" grey rings glow round the top board's two 0 dots [S-F9.helper.1.zeros, S-F9.helper.off]. No curve yet: the think ring with its "?" at the lower right. | predict | thought after "be built?"; question hold (think time) |
+| 14_6 | Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's minus one times minus two, which is two, so halve it, and the helper is one there. On paper, halving is no trouble. | The "?" goes. At "Position minus two, times position minus three" under the boards "trustee 1's helper: (position − 2) × (position − 3)" [S-F9.helper.1.zeros]; at "Each factor is zero at its own position" teal rings glow round the top board's 0 dots; at "so the product is zero at both" the product's curve draws in ink on the top board through both 0 dots, climbing toward position 0 until it leaves the board's top [S-F9.helper.off]. At "At position one it's minus one times minus two" under that line "at 1: (1 − 2) × (1 − 3) = (−1) × (−2)" [S-F9.helper.1.position, S-F9.helper.1.factors_at_own]; at "which is two" "= 2" after it and a ring where the curve passes height 2 over position 1 [S-F9.helper.1.own]. At "so halve it" "÷ 2" after the first line, and the curve gives way to one half as high everywhere, teal, through the 1 dot and both 0 dots, meeting the side at 3 [S-F9.helper.1.at_trio, S-F9.helper.on]. At "On paper, halving is no trouble" a small grey "on paper, halving is fine" under the lines. | derivation, controlled change | thought after "times position minus three."; thought after "zero at both." |
+| 14_7 | The same steps make the other two. Trustee two's is position minus one, times position minus three, which is minus one at position two, so divide by minus one. Trustee three's is position minus one, times position minus two, which is two at position three, so halve it. | Trustee 1's lines go and the dots on the middle and bottom boards brighten again [S-F9.helper.others]. At "Trustee two's is position minus one, times position minus three" under the boards "trustee 2's helper: (position − 1) × (position − 3)" [S-F9.helper.2.position, S-F9.helper.2.zeros] and its product's curve in ink on the middle board, through its 0 dots and down to −1 over position 2; at "which is minus one at position two" "at 2: (2 − 1) × (2 − 3) = −1" under it and a ring where the curve passes −1 [S-F9.helper.2.factors_at_own, S-F9.helper.2.own]; at "so divide by minus one" "÷ (−1)" after the first line, and the curve gives way to its mirror image, teal, up through the 1 dot [S-F9.helper.2.at_trio]. At "Trustee three's is position minus one, times position minus two" "trustee 3's helper: (position − 1) × (position − 2)" [S-F9.helper.3.position, S-F9.helper.3.zeros] and its product's curve on the bottom board, 2 over position 3; at "which is two at position three" "at 3: (3 − 1) × (3 − 2) = 2" and a ring where the curve passes 2 [S-F9.helper.3.factors_at_own, S-F9.helper.3.own]; at "so halve it" "÷ 2" and the curve gives way to one half as high, through the 1 dot [S-F9.helper.3.at_trio]. Each helper is now a curve, 1 at its own trustee and 0 at the other two. | derivation, controlled change | thought after "the other two."; thought after "divide by minus one." |
+| 14_8 | Multiplied out, each helper is a parabola, and so is their weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself. Its height at zero is the secret. | The lines under the boards go. At "so is their weighted sum" 19 times the top helper, plus 47 times the middle one, plus 89 times the bottom one draws in teal on the paper plot, from position 0 to just past 3, along the ink parabola [S-F9.helper.1.scaled, S-F9.helper.2.scaled, S-F9.helper.3.scaled]; at "passes through the dealer's three points" rings glow round the three points [S-F1.threshold, S-F9.sum_at.1, S-F9.sum_at.2, S-F9.sum_at.3]; at "only one parabola does" the ink parabola under it turns teal too, the two one curve all the way [S-F6.paper.curves]. At "Its height at zero is the secret" a gold ring glows where the curve meets the side and a gold outline round the "?" [S-F4.secret_position], and a precisely card: "three points pin one parabola (chapter 09), so 19 × helper 1 + 47 × helper 2 + 89 × helper 3 is the dealer's curve; at 0 it is the secret". The card stays until it can be read. | curve, linked views, precisely card | chapter_transition; hold until the card is read |
 
 ## 15_three_shares: the door, explained
 
 | id | words | picture | features | hold/listen |
 |---|---|---|---|---|
-| 15_1 | At zero, trustee one's halved helper is six halves, three. That's trustee one's weight, how much their height counts for at zero. | TeX: at x = 0: (−2)(−3) = 6, 6 ÷ 2 = 3 [S-F9.helper_at_zero]; a weight tag "3" hangs under trustee 1 on the rail. | derivation | thought after "six halves, three." |
-| 15_2 | The same steps give trustee two a weight of minus three, and trustee three a weight of one. So on paper, the secret is three times nineteen, minus three times forty-seven, plus eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five. | Weights row 3, −3, 1 [S-F9.weights], each under its trustee. TeX: 3·19 − 3·47 + 1·89 = 57 − 141 + 89 = 5 [S-F9.paper.heights, S-F9.paper.terms, S-F9.paper.sum]; the gold 5 at position 0 on the paper plot. | lagged, derivation | clause after "minus three,"; thought after "which is five." |
-| 15_3 | But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. ⟨predict⟩ Will the same weights still work on what they hold? | The paper heights fade; the rail's cards 8, 3, 1 [S-F1.opening_shares] stay lit; the weights 3, −3, 1 wait beside them. | predict | question hold (think time) |
-| 15_4 | Yes. Eight times three, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before. | TeX: 8·3 − 3·3 + 1·1 = 24 − 9 + 1 = 16 → 5 [S-F9.sum_unwrapped, S-F9.secret]; the vault from chapter 04 returns, needle to 5, door green. | derivation, callback | hold on the open door |
-| 15_5 | Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens. | 19 = 8 + 1·11, 47 = 3 + 4·11, 89 = 1 + 8·11 [S-F9.paper.elevens]; the elevens drawn as whole turns of the dial. | derivation | connected list |
-| 15_6 | Whole weights times whole elevens are still whole elevens, so the two sums differ by whole elevens. Sixteen is five and one eleven. Wrapping first changes nothing, because these weights are whole numbers. On the dial they're three, eight and one. | 16 − 5 = 11 [S-F9.sum_minus_secret]. precisely: "with whole weights, wrapping before the sum or after it lands on the same mark; on the dial the weights are 3, 8, 1 [S-F9.weights_on_dial]; this is Lagrange interpolation". | precisely card | chapter_transition |
+| 15_1 | So read each helper at position zero. Trustee one's is minus two times minus three, which is six, halved, so three. That's trustee one's weight: how much their height counts for at zero. | As chapter 14 ended: the paper plot with the trio's three points, the teal sum on the parabola and the gold "?" at position 0; the three helper boards with their curves and dots, and "× 19", "× 47" and "× 89"; the rail, 1, 2 and 3 teal with 8, 3 and 1. At "read each helper at position zero" a gold outline round position 0's column, down all three boards [S-F4.secret_position]. At "Trustee one's is minus two times minus three" under the boards "trustee 1 at 0: (0 − 2) × (0 − 3)" [S-F9.helper.1.position, S-F9.helper.1.factors_at_zero]; at "which is six, halved, so three" "= 6, ÷ 2: 3" after it [S-F9.helper.1.at_zero, S-F9.helper.1.own, S-F9.helper.1.weight], a gold dot where the top curve meets position 0, at height 3, and a teal "3" beside the board's side. At "That's trustee one's weight" a teal "weight 3" under trustee 1's disc, a ring glowing round the disc. | derivation, linked views | thought after "so three." |
+| 15_2 | Trustee two's is minus one times minus three, which is three, divided by minus one, so its weight is minus three. Trustee three's is minus one times minus two, which is two, halved, so one. The weights use only the positions, one, two and three, which everyone knows. | At "Trustee two's is minus one times minus three" "trustee 2 at 0: (0 − 1) × (0 − 3)" under the first line [S-F9.helper.2.position, S-F9.helper.2.factors_at_zero]; at "which is three, divided by minus one" "= 3, ÷ (−1): −3" after it [S-F9.helper.2.at_zero, S-F9.helper.2.own, S-F9.helper.2.weight], a gold dot at height −3 over position 0 on the middle board with a teal "−3" beside it, and "weight −3" under trustee 2's disc, a ring glowing round it. At "Trustee three's is minus one times minus two" "trustee 3 at 0: (0 − 1) × (0 − 2)" [S-F9.helper.3.position, S-F9.helper.3.factors_at_zero]; at "which is two, halved, so one" "= 2, ÷ 2: 1" [S-F9.helper.3.at_zero, S-F9.helper.3.own, S-F9.helper.3.weight], a gold dot at height 1 on the bottom board with "1", and "weight 1" under trustee 3's disc. At "use only the positions" teal outlines glow round the position numbers 1, 2 and 3 over the top board and round the three weights on the rail [S-F1.opening_trio, S-F9.weights]. | derivation, outline glow | thought after "so its weight is minus three."; thought after "so one." |
+| 15_3 | So on paper, the secret is three times nineteen, minus three times forty-seven, plus one times eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five. | The three lines under the boards and the gold column outline go. At "three times nineteen" the top board's weight comes over from beside its side to stand in front of "× 19", which makes room: "3 × 19"; at "minus three times forty-seven" the middle one's, "−3 × 47"; at "plus one times eighty-nine" the bottom one's, "1 × 89" [S-F9.paper.weighted]. At "That's fifty-seven" "= 57" after the first, at "minus a hundred and forty-one" "= −141" after the second, and at "plus eighty-nine" "= 89" after the third [S-F9.paper.terms], with "on paper: 57 − 141 + 89" under the boards. At "which is five" a gold "= 5" after it, and on the plot the "?" gives way to a gold point and a gold "5"; the point swells [S-F9.paper.sum, S-F1.secret]. | derivation, controlled change | clause after "minus three times forty-seven,"; cue_transition after "which is five." |
+| 15_4 | But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. Will the same weights still work on what they hold? ⟨predict⟩ | At "never see those heights" the paper plot and the terms beside the boards go; the line "on paper: 57 − 141 + 89 = 5" stays under the boards. At "eight, three and one" teal outlines glow round the rail's three cards [S-F1.opening_shares]; at "Will the same weights" teal outlines glow round the three weights under the discs [S-F9.weights]. The think ring with its "?" at the upper left, where the plot was. | predict | thought after "never see those heights."; question hold (think time) |
+| 15_5 | Yes. Three times eight, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before. | The "?" goes. At "Three times eight, minus three times three, plus one times one" under the paper line "on the cards: 3 × 8 − 3 × 3 + 1 × 1" [S-F9.cards.weighted]; at "is sixteen" "= 24 − 9 + 1 = 16" after it [S-F9.share_terms, S-F9.sum_unwrapped]; at "sixteen lands on five" a gold "→ 5" [S-F9.sum_lands]. At "The dial turns to five" the vault at the upper left as in chapter 04, and its needle runs sixteen marks, once round and on to 5 [S-F9.sum_split, S-F9.secret]; at "the door opens" the door turns green and swells. The open door holds. | derivation, callback | thought after "lands on five."; hold on the open door |
+| 15_6 | Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens. | The helper boards go; the vault stays open. At "Nineteen is eight and one eleven" at the upper right "19 = 8 + 1 × 11", and a teal outline glows round trustee 1's card [S-F9.split.1]; at "forty-seven is three and four elevens" "47 = 3 + 4 × 11" under it, an outline round trustee 2's card [S-F9.split.2]; at "eighty-nine is one and eight elevens" "89 = 1 + 8 × 11", an outline round trustee 3's [S-F9.split.3, S-F9.paper.elevens]. | derivation, outline glow | thought after "Here's why."; connected list (short clauses after "one eleven," and "four elevens,") |
+| 15_7 | Whole weights turn whole elevens into whole elevens. So the sum on paper and the sum on the cards can differ only by whole elevens, and sixteen is five and one eleven. That's why wrapping first changes nothing, as long as the weights are whole. On the dial, they're three, eight and one. | At "Whole weights turn whole elevens into whole elevens" under the three rows "3 × (1 × 11) − 3 × (4 × 11) + 1 × (8 × 11) = −1 × 11" [S-F9.weights, S-F9.paper.elevens, S-F9.weighted_elevens]. At "can differ only by whole elevens" teal outlines glow round the sum on paper and the sum on the cards; at "sixteen is five and one eleven" "16 = 5 + 1 × 11" under them [S-F9.sum_split, S-F9.sum_minus_secret]. At "On the dial, they're three, eight and one" trustee 2's "weight −3" gives way to "weight 8", −3 landing on 8, and outlines glow round the three weights [S-F9.weights_on_dial]; then a precisely card: "wrapped before the sum or after, whole weights land on the same mark; on the dial they are 3, 8 and 1 (Lagrange interpolation)". The card stays until it can be read. | derivation, precisely card | thought after "into whole elevens."; chapter_transition; hold until the card is read |
 
 ## 16_fraction_weights: when the weights are fractions
 
--- a/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
+++ b/explainer_corpus/shamir_secret_sharing/docs/script-draft.tsv
@@ -88,18 +88,21 @@
 13_7	Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero.
 13_8	So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do.
 13_9	That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do.
-14_1	Now the door from the start. Trustees one, two and three hold eight, three and one, and those three numbers somehow give five. To see why, go back to paper first, where the dealer's curve had heights nineteen, forty-seven and eighty-nine.
-14_2	On paper, three points pin a parabola down, as we saw. So build that one parabola from three helper curves. Trustee one's helper is one at position one, and zero at positions two and three. The other two helpers do the same for their own trustees.
-14_3	Then nineteen times the first helper, plus forty-seven times the second, plus eighty-nine times the third, passes through all three points. At each trustee, only that trustee's helper is switched on.
-14_4	What simple recipe is zero at both positions two and three?
-14_5	Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's two, so halve it, and the helper is one there. On paper, halving is no trouble.
-14_6	Multiplied out, each helper is a parabola too, and so is the weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself.
-15_1	At zero, trustee one's halved helper is six halves, three. That's trustee one's weight, how much their height counts for at zero.
-15_2	The same steps give trustee two a weight of minus three, and trustee three a weight of one. So on paper, the secret is three times nineteen, minus three times forty-seven, plus eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five.
-15_3	But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. Will the same weights still work on what they hold?
-15_4	Yes. Eight times three, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before.
-15_5	Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens.
-15_6	Whole weights times whole elevens are still whole elevens, so the two sums differ by whole elevens. Sixteen is five and one eleven. Wrapping first changes nothing, because these weights are whole numbers. On the dial they're three, eight and one.
+14_1	Back at the start, trustees one, two and three brought their eight, three and one, and the door opened. How they did it was left for later. They know three points of the dealer's curve, but not its spins. So how can three numbers give five?
+14_2	To see the idea, go back to paper, where nothing wraps, and pretend for now that they knew their heights there: nineteen, forty-seven and eighty-nine. Three points pin a parabola down, so the one through these is the dealer's curve, and its height at zero is the secret. The trick is to build that curve out of simpler pieces.
+14_3	Give each trustee a helper curve. Trustee one's helper is one at position one, and zero at positions two and three. Trustee two's is one at position two, and trustee three's is one at position three, each zero at the other two positions. A helper answers to its own trustee only.
+14_4	Now scale each helper by its trustee's height: nineteen times the first, forty-seven times the second, and eighty-nine times the third, and add them. At position one, the second and third helpers are zero, and zero times anything is zero, so the sum there is nineteen. At position two it's forty-seven, and at three it's eighty-nine. The sum passes through all three points.
+14_5	But can a helper like that be built? What simple recipe is zero at both positions two and three?
+14_6	Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's minus one times minus two, which is two, so halve it, and the helper is one there. On paper, halving is no trouble.
+14_7	The same steps make the other two. Trustee two's is position minus one, times position minus three, which is minus one at position two, so divide by minus one. Trustee three's is position minus one, times position minus two, which is two at position three, so halve it.
+14_8	Multiplied out, each helper is a parabola, and so is their weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself. Its height at zero is the secret.
+15_1	So read each helper at position zero. Trustee one's is minus two times minus three, which is six, halved, so three. That's trustee one's weight: how much their height counts for at zero.
+15_2	Trustee two's is minus one times minus three, which is three, divided by minus one, so its weight is minus three. Trustee three's is minus one times minus two, which is two, halved, so one. The weights use only the positions, one, two and three, which everyone knows.
+15_3	So on paper, the secret is three times nineteen, minus three times forty-seven, plus one times eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five.
+15_4	But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. Will the same weights still work on what they hold?
+15_5	Yes. Three times eight, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before.
+15_6	Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens.
+15_7	Whole weights turn whole elevens into whole elevens. So the sum on paper and the sum on the cards can differ only by whole elevens, and sixteen is five and one eleven. That's why wrapping first changes nothing, as long as the weights are whole. On the dial, they're three, eight and one.
 16_1	Another day, trustees two, four and five come, with three, two and six. On paper their heights were forty-seven, a hundred and forty-five, and two hundred and fifteen. Trustee two's helper starts as position minus four, times position minus five.
 16_2	At trustee two that's six, and at zero it's twenty, so trustee two's weight is twenty divided by six, ten thirds. Trustee four's comes out as minus five, and trustee five's as eight thirds. On paper these weights still turn the three heights into exactly five.
 16_3	But will these fractions still work on the shares, three, two and six?
--- a/explainer_corpus/shamir_secret_sharing/script.tsv
+++ b/explainer_corpus/shamir_secret_sharing/script.tsv
@@ -88,3 +88,18 @@
 13_7	Landing on zero means being a multiple of eleven. Every whole number splits into primes in only one way, so a product of marks has eleven among its primes only if one of the marks does. No mark from one to ten does. So on eleven marks, a product lands on zero only when one of its factors is zero.
 13_8	So if two marks gave the same answer after multiplying by five, five times their difference would land on zero, and that can't happen unless they're the same mark. The same holds for every multiplier from one to ten: each one only shuffles the eleven marks. On ten marks, only one, three, seven and nine do.
 13_9	That's what the counting needs. Any prime number of marks works, if it's more than the number of trustees, so that each one has a position besides zero, and the secret is one of its marks. For five trustees, seven marks would already do.
+14_1	Back at the start, trustees one, two and three brought their eight, three and one, and the door opened. How they did it was left for later. They know three points of the dealer's curve, but not its spins. So how can three numbers give five?
+14_2	To see the idea, go back to paper, where nothing wraps, and pretend for now that they knew their heights there: nineteen, forty-seven and eighty-nine. Three points pin a parabola down, so the one through these is the dealer's curve, and its height at zero is the secret. The trick is to build that curve out of simpler pieces.
+14_3	Give each trustee a helper curve. Trustee one's helper is one at position one, and zero at positions two and three. Trustee two's is one at position two, and trustee three's is one at position three, each zero at the other two positions. A helper answers to its own trustee only.
+14_4	Now scale each helper by its trustee's height: nineteen times the first, forty-seven times the second, and eighty-nine times the third, and add them. At position one, the second and third helpers are zero, and zero times anything is zero, so the sum there is nineteen. At position two it's forty-seven, and at three it's eighty-nine. The sum passes through all three points.
+14_5	But can a helper like that be built? What simple recipe is zero at both positions two and three?
+14_6	Position minus two, times position minus three. Each factor is zero at its own position, so the product is zero at both. At position one it's minus one times minus two, which is two, so halve it, and the helper is one there. On paper, halving is no trouble.
+14_7	The same steps make the other two. Trustee two's is position minus one, times position minus three, which is minus one at position two, so divide by minus one. Trustee three's is position minus one, times position minus two, which is two at position three, so halve it.
+14_8	Multiplied out, each helper is a parabola, and so is their weighted sum. It passes through the dealer's three points, and only one parabola does, so the sum is the dealer's curve itself. Its height at zero is the secret.
+15_1	So read each helper at position zero. Trustee one's is minus two times minus three, which is six, halved, so three. That's trustee one's weight: how much their height counts for at zero.
+15_2	Trustee two's is minus one times minus three, which is three, divided by minus one, so its weight is minus three. Trustee three's is minus one times minus two, which is two, halved, so one. The weights use only the positions, one, two and three, which everyone knows.
+15_3	So on paper, the secret is three times nineteen, minus three times forty-seven, plus one times eighty-nine. That's fifty-seven, minus a hundred and forty-one, plus eighty-nine, which is five.
+15_4	But the trustees never see those heights. They hold only the wrapped numbers, eight, three and one. Will the same weights still work on what they hold?
+15_5	Yes. Three times eight, minus three times three, plus one times one is sixteen, and sixteen lands on five. The dial turns to five, and the door opens, just as it did before.
+15_6	Here's why. Each share is its height with whole elevens taken off. Nineteen is eight and one eleven, forty-seven is three and four elevens, and eighty-nine is one and eight elevens.
+15_7	Whole weights turn whole elevens into whole elevens. So the sum on paper and the sum on the cards can differ only by whole elevens, and sixteen is five and one eleven. That's why wrapping first changes nothing, as long as the weights are whole. On the dial, they're three, eight and one.
--- a/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
+++ b/explainer_corpus/shamir_secret_sharing/docs/pause-overrides.json
@@ -1,5 +1,5 @@
 {
-  "_comment": "Authored semantic intentions for chapters06\u201313 before synthesis. Preserve predictions, sequential card reading and scene-work overruns separately. Durations are proposals, not measured speech or listening acceptance.",
+  "_comment": "Authored semantic intentions for chapters06\u201315 before synthesis. Preserve predictions, sequential card reading and scene-work overruns separately. Durations are proposals, not measured speech or listening acceptance.",
   "overrides": [
     {
       "video": "shamir_secret_sharing",
@@ -539,6 +539,270 @@
         1.0
       ],
       "reason": "Close the proof before it is extended to every multiplier."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_1",
+      "after": "the door opened",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the remembered opening land before the open question."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_1",
+      "after": "give five",
+      "role": "cue_transition",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "A rhetorical question: a beat into the paper picture, not think time."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_2",
+      "after": "where nothing wraps",
+      "role": "clause",
+      "range_s": [
+        0.3,
+        0.6
+      ],
+      "reason": "Mark the switch to ordinary numbers before the pretence."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_2",
+      "after": "is the secret",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the goal, the height at zero, settle before the trick."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_3",
+      "after": "helper curve",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Name the new object before its rule."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_3",
+      "after": "two and three",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the first helper's three dots be read before the others."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_4",
+      "after": "and add them",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Finish the recipe before checking it at each position."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_4",
+      "after": "the sum there is nineteen",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the first check land before the other two positions."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_5",
+      "after": "be built",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Pose the doubt before the concrete question."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_6",
+      "after": "times position minus three",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the product be read before why it works."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_6",
+      "after": "zero at both",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Close the zeros before the value at position one."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_7",
+      "after": "the other two",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "Announce the repetition before the second helper."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "14_7",
+      "after": "divide by minus one",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Separate the second helper's working from the third's."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_1",
+      "after": "so three",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the first weight land before naming it."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_2",
+      "after": "so its weight is minus three",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Separate the second weight's working from the third's."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_2",
+      "after": "so one",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the last weight land before what the weights use."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_3",
+      "after": "minus three times forty-seven",
+      "role": "clause",
+      "range_s": [
+        0.25,
+        0.55
+      ],
+      "reason": "Keep the three weighted heights apart as separate terms."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_3",
+      "after": "which is five",
+      "role": "cue_transition",
+      "range_s": [
+        0.8,
+        1.3
+      ],
+      "reason": "Let the paper secret land before the trustees' problem."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_4",
+      "after": "never see those heights",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the obstacle register before what they hold."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_5",
+      "after": "lands on five",
+      "role": "thought",
+      "range_s": [
+        0.5,
+        0.9
+      ],
+      "reason": "Let the answer land before the door opens."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_6",
+      "after": "Here's why",
+      "role": "thought",
+      "range_s": [
+        0.4,
+        0.8
+      ],
+      "reason": "A beat before the explanation."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_6",
+      "after": "one eleven",
+      "role": "clause",
+      "range_s": [
+        0.15,
+        0.35
+      ],
+      "reason": "A short list beat: keep the three splits together."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_6",
+      "after": "four elevens",
+      "role": "clause",
+      "range_s": [
+        0.15,
+        0.35
+      ],
+      "reason": "A short list beat: keep the three splits together."
+    },
+    {
+      "video": "shamir_secret_sharing",
+      "cue": "15_7",
+      "after": "into whole elevens",
+      "role": "thought",
+      "range_s": [
+        0.6,
+        1.0
+      ],
+      "reason": "Let the principle land before it is applied to the sums."
     }
   ]
 }
--- a/explainer_corpus/shamir_secret_sharing/docs/brief.md
+++ b/explainer_corpus/shamir_secret_sharing/docs/brief.md
@@ -191,7 +191,7 @@
   pronunciation overrides every source.
 - **Single letters:** "D" and "k minus one" (23_5). TTS may read a lone
   letter as a word.
-- **Terms of art:** "parabola" (09_4, 14_2, 14_6), "polynomial" (09_7) and "prime"
+- **Terms of art:** "parabola" (09_4, 14_2, 14_8), "polynomial" (09_7) and "prime"
   (13_6, 13_7, 13_9). Check the stress.
 - **The year:** "nineteen seventy-nine" (06_6) is written out, so it should
   not be read as a cardinal.
```
<!-- shamir-1415 patch end -->
