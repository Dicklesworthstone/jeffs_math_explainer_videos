# Algebra rebuild candidate: chapters 03, 08, 09, 14, 16, 17

Bead `mx-video-program-program-87do.64` · staging lane LavenderPike · 2026-11-04

Root assigned this work in `ORDERS.md`. Narration is **ElevenLabs Oliver only**,
by the owner's explicit requirement (2026-11-04), and that requirement replaces
any Michael or FrankenTTS default. I ran no synthesis.

The rebuild is native Rust FrankenManim scene code for the expanded abstract
algebra film. It covers the priority chapters 08 (subgroups), 09 (centralizer),
03 (equivalence classes, completing 02/03), 14 (cosets), 16 (quotients) and 17
(applications). Chapter 06 followed as a second change set (see the status
section below). I edited no canonical file.

**Status: uncompiled by me, unrendered, unheard.** Treat everything below as
a candidate until root's cross-review, Cargo gates, renders and listening.

## Status since integration (2026-11-04, updated 07:00Z)

- **Core integrated by root** at 04:46Z (Mail 760). It compiled: 55 tests,
  strict Clippy, and 92 facts matching the oracle (Mails 767, 780). Root's
  Clippy, 08-premise and 09-layering fixes live in canonical. The original
  core patch below is history now; **do not reapply it.**
- **Staging now equals canonical** for every file root edited (synced
  05:47Z). Every later change set is staged against current canonical and
  listed with base and staged hashes.
- **06 axioms native rebuild** (Mail 868). It changes `src/axioms.rs`, a
  `// 06:` block of 33 `axioms.*` facts in `main.rs`, and an independent
  `oracle.py` section. No 06 text changed. Root has already copied these
  files into canonical, with one `needless_range_loop` fix.
- **17 follow-up for Mail 736** (Mail 880), patch
  `patches/abstract_algebra_expanded-17-followup.patch`, against current
  canonical:
  - 17_16 gains a place-value sentence and reading (4, 2, 1; 101 = 4 + 0 + 1)
    before the label is read as five.
  - 17_18 uses root's softened wording.
  - The pictures add a place-value key, reused in 17_17 for 111 = 7, and the
    extended-code rules. Five facts back them, including 128/128 single flips
    corrected and 448/448 double flips reported.
  - **17_16 and 17_18 need Oliver synthesis.** Neither has a take.
- **Re-synced 06:20Z.** Root changed canonical `axioms.rs` (06 picture
  repair) and `subgroups.rs` (08 premise timing) after the 05:47Z sync.
  Staging took both, so neither change is lost. The old staged copies are
  kept outside the repository.
- **13 arithmetic picture repairs**, patch
  `patches/abstract_algebra_expanded-13-followup.patch`. Apply it after the
  17 follow-up: its `main.rs` and `oracle.py` hunks sit on the 17-applied
  files, while `arithmetic.rs` and `axioms.rs` sit on current canonical. No
  13 wording changed, so the existing Oliver takes 13_9 to 13_12 still bind.
  - 13_9: the package labels moved to y −1.95, below the turning corners'
    reach of −1.45 (root's corner-4 clearance finding at 189.5 s and
    201.5 s). Each tile now performs its package as one continuous motion:
    (sr)s runs the lone s, then sr; s(rs) runs rs, then the lone s, timed to
    "grouping the first two". The sentence on swapping two moves gets
    `sr ≠ rs`; the closing sentence shows the law beside `sr ≠ rs`.
  - 13_10: a number now travels *inside* three machines, h(x)=x+2,
    g(x)=3x and f(x)=x−1. Each machine changes it in place: 1, 3, 9, 8.
    Two trails record the values under f∘(g∘h) and (f∘g)∘h, each with its
    package box and bracket. The columns pulse together, then the
    associative law appears.
  - 13_11: four property cards land as each property is named, and a gold
    frame encloses them as "a group". The rack of eight appears as the
    square example. On a number line, 2 + 0 = 2, then 2 + 3 = 5 and
    5 + (−3) = 2. For the positive integers, zero and the negatives go dark,
    the Identity and Inverse cards are crossed, and undoing +3 is refused
    because −3 is missing.
  - 13_12: each sentence gets a rack calculation under its lit card:
    - closure: r s = rs;
    - associativity: s r s r² s = (s r s) r² s = r⁻¹ r² s = (r⁻¹ r²) s = rs;
    - identity: e r = r = r e;
    - inverses: r x = s, so x = r⁻¹ s = r³s, checked by r (r³s) = s.
    The four cards then return over three unknown "?" objects.
  - 21 `arithmetic.*` facts, which the independent oracle reproduces
    (151 lines, exit 0), and 6 new tests. The only `axioms.rs` change makes
    6 helpers `pub(crate)`.
- **Re-synced 06:35Z.** Root changed canonical `axioms.rs` at 06:31Z (the
  06_7 check label and hide list) and `centralizer.rs` (09 proof lines).
  Staged `centralizer.rs` now equals canonical. Staged `axioms.rs` is
  canonical `d204a79b` plus only the six `pub(crate)` edits, giving
  `605d531b`. The 13 patch still applies to it with no offset (dry-run
  checked).
- **10 cyclic native rebuild (Mail 815)**, patch
  `patches/abstract_algebra_expanded-10-cyclic.patch` (sha256 prefix
  `f9c2b58a`, 12 hunks).
  - **Apply order: 17, then 13, then 10.** Its `cyclic.rs` hunks sit on
    canonical `21bdb31b`. Its `main.rs` and `oracle.py` hunks sit on the
    13-applied files (`c2b4d312`, `c7adfde6`).
  - **Round trip:** current canonical, then 17, 13 and 10, is byte-identical
    to staging for all seven touched files.
  - **Staged hashes:** `cyclic.rs` `946f1da4`, `main.rs` `0834b53e`,
    `oracle.py` `f98c3db4`.
  - **No cue text changed**, so all 11 of 10_1–10_11 stay as they are.
    10_1–10_7 keep their Oliver takes. 10_8–10_11 still have none, exactly as
    before this change.
  - **What each cue shows:**
    - 10_1 and 10_2: kept as they were. The hour labels and gold dots are
      now named, so later beats can reuse them.
    - 10_3: a table of m, 8m and the position (1, 2, 3 / 8, 16, 24 / 8, 4, 0)
      fills one column at a time, and each trip's gold dot pulses with its
      cell. `12 ∣ 8m` appears, then crosses, crosses, tick. A frame closes
      round the m = 3 column. At "and the distinct positions", 8, 4, 0 turn
      green with ⟨8⟩.
    - 10_4: d = gcd(12, 8) = 4. Four orbit triangles are drawn on the clock
      ({0,8,4} gold, {1,9,5}, {2,10,6}, {3,11,7}) and the hours take their
      orbit's colour: "4 orbits, 3 positions each". For n = d n' = 4·3, three
      teal block arcs of four hours, numbered 0, 1, 2, ring the clock. At
      "and k as d times", a hop disc runs 8 hours and the block numbers 1, 2
      pulse as it crosses them: k = d k' = 4·2. Then gcd(n′, k′) = gcd(3, 2) = 1.
    - 10_5: `n ∣ mk   12 ∣ 8m`. Then `d n′ ∣ m d k′   4·3 ∣ m·4·2`, with
      both d's and both 4s red, morphs into `n′ ∣ m k′   3 ∣ 2m`. The hours
      dim and the block numbers light: the scaled clock. For
      gcd(n′,k′) = 1 ⇒ n′ ∣ m, the marker walks three moves of 8 hours. A
      counter runs m = 0, 1, 2, 3 and each landing block (2, 1, 0) pulses;
      the walk is home at m = 3 (tick). Then m = n′ = 3, and
      ord(k) = n/gcd(n,k) = 12/4 = 3.
    - 10_6: the formula rises to ord(k) = 12/gcd(k,12). Rows of k = 1..11,
      gcd(k,12) and ord(k) follow, the ord row computed by walking. At "one,
      five, seven", green frames close round 1, 5, 7, 11 and the other
      columns dim. Four small clocks trace the 12-step tours of +1, +5, +7
      and +11 in visiting order. A fifth clock draws +8's red triangle,
      gcd(8,12) = 4 is framed red, and the 9 unvisited dots go dark ("3 of
      12 positions").
    - 10_7: the theorem becomes the title. An exponent line runs m = −8..12,
      with G = ⟨g⟩, H ≤ G and the example g = +1, H = ⟨8⟩ = {0,4,8} on a
      small clock. Trivial case: only g⁰ = e lights, with H = {e} ⇒ H = ⟨e⟩.
      "Otherwise": a walker on the line and a hand on the small clock step
      m = 1..12 together, and each member's dot (4, 8, 12) drops in green as
      it is visited. At "and choose the one", 1, 2, 3 get crosses and 4 a
      tick. At "call that exponent d", 4 is ringed and
      d = min{j > 0 : gʲ ∈ H} = 4 appears. Last sentence: the walk back to
      −8 lights −4 and −8; −8 is ringed, and an arc carries it to 8:
      (g⁻⁸)⁻¹ = g⁸ ∈ H.
    - 10_8: the member m = 8 is ringed: `g^m ∈ H: m = 8`. Then
      m = qd + r, 0 ≤ r ≤ d − 1 (the spoken bound). Two gold chunks of d run
      0→4→8: 8 = 2·4 + 0. At "Because g to the d is in H", (g^d)^q = g^{qd} ∈ H,
      green chunks run on to −8 and 12, and all six members pulse.
    - 10_9: two blue chunks run back from 8 to 4 to 0, and the landing is
      ringed: g^m g^{−qd} = g^r ∈ H. The contradiction: "if g¹⁰ ∈ H:
      10 = 2·4 + 2". Red chunks run back 10→6→2 and land on the 2 that 10_7
      crossed out, which pulses with `0 < 2 < d`. The supposition is
      dismissed: r = 0, g^m = (g^d)^q. At "and every element of H", chunks
      of d tile the whole line and every member pulses. Last sentence:
      H = ⟨g^d⟩ = ⟨g⁴⟩ as the small clock traces 0→4→8.
    - 10_10: it now clears the 10_7–10_9 objects; the U(12) table beats are
      unchanged.
  - **Facts and checks:**
    - 33 `cyclic.*` facts in a `// 10:` block of `rebuild_facts`. They are
      computed by walks and divisor search: ord comes from the first return,
      not from n/gcd, and H is the walk of 8.
    - The new independent `oracle.py` section uses n/gcd (Euclid), multiples
      8m, and H as the closure of {8} under addition. It prints 184 lines
      (was 151), exit 0.
    - A Python mirror of the Rust formulas agrees on all 33 keys.
  - **Tests and formatting:** five new tests (a Python mirror passes all
    five):
    - orbits, orders and gcds on every n ≤ 36;
    - the block reading and the table;
    - generators equal the coprime moves;
    - every drawn member cuts into whole chunks, and the ghost's remainder is
      not a member;
    - every waited phrase sits in its sentence, and sentence counts for 10_2
      to 10_10.

    `rustfmt --check` passes on the whole overlay crate.
  - **Timing:** the beats are budgeted from take01 sentence starts (10_3 to
    10_7) by character share. Every sentence's picture work ends before the
    next sentence starts. The tightest is 10_5 s2 (about 4.4 s of 5.8 s).
    10_8 and 10_9 have no takes, so their budget assumes about 2.2 words
    per second (about 19 s and 25 s).
- **Remaining:** root's compile, Clippy, tests and facts-check on 17, then 13,
  then 10. After that: renders, Oliver synthesis of 10_8–10_11 plus 17_16 and
  17_18, listening, fine-sync and independent AV review. Chapters 09 and 11
  are root's.

## Files

| Path (relative to this directory) | What it is |
|---|---|
| `candidate/explainer_corpus/abstract_algebra_expanded/script.tsv` | Full spoken text: 169 cues (was 145), 9,114 words (was 7,971) |
| `candidate/.../src/d4_group.rs` | **New module.** Exact D4 arithmetic (`Elem`, rightmost-first `mul`, closure rounds, subgroups, centralizer, center, cosets, mirror-line carrying) and shared pictures (rack thumbnails, slot frames, carried mirror lines, show/hide/glide) |
| `candidate/.../src/subgroups.rs` | 08 rebuilt |
| `candidate/.../src/centralizer.rs` | 09 rebuilt |
| `candidate/.../src/transitivity.rs` | 03 rebuilt and extended |
| `candidate/.../src/cosets.rs` | 14 rebuilt |
| `candidate/.../src/quotients.rs` | 16: canonical beats kept, four beats inserted |
| `candidate/.../src/applications.rs` | 17: canonical ISBN beats kept, Hamming(7,4) film added |
| `candidate/.../src/main.rs` | Canonical file plus `mod d4_group;` and `rebuild_facts()` (72 new facts). rustfmt reordered the `mod` lines alphabetically |
| `candidate/.../oracle.py` | Canonical oracle, unchanged, plus an independent section printing the same 72 keys |
| `patches/abstract_algebra_expanded-rebuild.patch` | All 10 files as one `git apply` patch (sha256 prefix `bb5bc43bfd0dd500`) |
| `checks/algebra_checks.py` | Independent first-principles checker (36 values) |
| `checks/build_candidate_script.py` | Builds `script.tsv` from the canonical script by cue id and asserts every retained cue is byte-identical. Usage: `python3 -I -B checks/build_candidate_script.py CANONICAL OUT` |

### Integration

From the repository root, run
`git apply docs/video-program/third-wave-20261024/opus-replacements-20261104/algebra/patches/abstract_algebra_expanded-rebuild.patch`.
Alternatively, copy the 10 candidate files over their mirrors.

The canonical crate is untracked, so the patch's base is recorded by hash
instead of by git revision. These are sha256 prefixes of the canonical files as
of 2026-11-04 04:4x UTC:

```
9e7195d6fd85b37c  script.tsv          25a06ab1529e5b17  src/centralizer.rs
b26fb175a754dc82  oracle.py           7495079bbe7ec13e  src/transitivity.rs
6d8d5f09fe4bdaf2  src/main.rs         6e6c45fa1f1f67fd  src/cosets.rs
409a32a36d8b898b  src/subgroups.rs    243aecec4a92ab35  src/quotients.rs
c12bde184cfe4d05  src/applications.rs
```

If any of these changed (for example main.rs, which every chapter shares),
merge instead of applying.

## Narration: what the ElevenLabs lane must synthesize

The film must stay in this order: each scene's `say` order equals the script
order, which I checked mechanically.

- **25 new cues:** 03_4 03_5 03_6 03_7 03_8 03_9 · 08_7 08_8 08_9 08_10 08_11 08_12 08_13 · 09_9 09_10 · 14_14 14_15 · 16_14 16_15 16_16 16_17 · 17_15 17_16 17_17 17_18
- **16 reworded cues** (any take of the old wording is stale; reuse takes only by exact text): 08_4 08_5 08_6 · 09_2 09_3 09_4 09_5 09_6 09_7 09_8 · 14_4 14_5 14_6 14_7 14_8 · 17_9
- **Removed:** 17_10. Its content moved into the Hamming beats. Preserve any existing take; nothing needs deleting.
- **Retained untouched:** all 57 cues with retained Oliver takes in root's `current-retained-coverage38.json` (04:36Z), checked byte-identical by the builder. In these chapters they are 03_1–03_3 and 08_1–08_3. Chapter 10 (all 11 cues, frozen) is unchanged, and none of the 57 retained cues is reworded or removed by this candidate.
- **One new seam to hear:** 03_3 used to end chapter 03 (14_1 followed it) and now leads into 03_4. Its single-cue UI take carries no context, so the text is reusable. Only listening can tell whether its ending sounds chapter-final.
- Unchanged cues that have no retained take (for example 09_1, 14_1) are unaffected by this candidate. They stay in the existing queue.

No cue contains digits or symbols: the builder asserts this, so every number is
spoken as a word. The longest new cue is 86 words.

## What each rebuilt chapter now shows

- **03.** Defines the class of an element, then shows why classes are disjoint or equal: the overlap c, with symmetry and transitivity joining the two links. Closeness overlaps without coinciding (near 1 is {0,1,2}, near 0 is {−1,0,1}). Twelve clock lanes partition the integers, and any partition gives back a relation. Fraction pairs sit on lines through the origin with the value as slope. A forward pointer to cosets closes the chapter.
- **08.** A missing product is repaired by a closure machine that runs pass by pass on screen: {s} gives 2 moves; {s, r²} gives +2, then nothing; {r, s} gives +4, then +2, the whole rack. A census then finds exactly ten subgroups, shown on shelves by size, with no subgroup of size 3, 5, 6 or 7. The one-step test is fed (h, h), (e, h) and (a, b⁻¹) on screen.
- **09.** Two tiles run "g then s" against "s then g" for every move, and a scoreboard collects the checks and crosses. The reason is shown, not asserted: the mirror line rides along with each move, the moves that earn checks are exactly the ones that return the line, and g s g⁻¹ is the flip across the carried line. The centralizer is matched with the subgroup grown in 08. The sliding proof moves s across a·b⁻¹. The center keeps all four lines.
- **14.** Left cosets of {e, s} are the rack columns, and right cosets cut differently (Hr = {r, r³s}). The clock test (1 and 9 share a coset, 1 and 7 do not) becomes the relation "g⁻¹k ∈ H". A triangle callback shows the bijection h ↦ gh. Lagrange's theorem explains the 08 census.
- **16.** rH·rH gives four products in two cosets. The 64-entry table, regrouped into rotations and flips, shows four solid blocks that shrink to the 2×2 quotient. Sorted by cosets of {e, s}, 8 of the 16 blocks are mixed, and the rH·rH block is highlighted.
- **17.** Hamming(7,4): 16 codewords form a subgroup of the 128 words, giving 8 cosets of 16 laid out as a grid with the syndrome as row label. 0110011 with bit 5 flipped reads label 101 and is repaired. Bits 2 and 5 flipped read label 111; the decoder flips bit 7 and lands on another codeword (0010110).

Every highlighted set and every narrated number is computed: the sets in
`d4_group.rs`, the numbers in `main.rs` facts. All 72 new facts come from the
same helpers the scenes draw with. For example, the 08 machine scenes and
`rebuild_facts` share the `FLIP_ALONE`, `FLIP_AND_HALF_TURN` and
`QUARTER_TURN_AND_FLIP` arrays.

## Checks run, with results

| Check | Result |
|---|---|
| `python3 -I -B oracle.py` (candidate crate dir) | exit 0; 92 lines (20 canonical + 72 new) |
| Python mirror of `rebuild_facts()` using the Rust formulas (r^t s^f arithmetic, `carry_line`, `mixed_blocks`, byte layout), diffed against the oracle | 72 keys, 0 mismatches. The mirror script was a scratch file and is not kept |
| `python3 -I -B checks/algebra_checks.py` | `ok 36 values; all assertions held` |
| `build_candidate_script.py` rerun to scratch, then `cmp` | byte-identical to the staged `script.tsv`; 57 retained cues untouched |
| skill `lint_script.py` on the candidate script | 0 errors, 160 warnings, PASS (canonical: 0 errors, 156 warnings) |
| Lint warnings on the 41 changed cues | 25, all "N sentences; split at a picture change". Disposition: each is paced in-scene with `kit.until_sentence` at sentence boundaries, and splitting would only add TTS units |
| `until_sentence` audit | every literal index is below its cue's sentence count (the kit's splitter) and non-decreasing within the cue. The three loop-driven sites (08 machine, 08_11, 09_9) were checked by hand |
| `say` order against script order, per chapter | equal for 03, 08, 09, 14, 16 and 17 |
| `rustfmt --edition 2024 --config max_width=120 --check` | all 8 candidate .rs files clean, in place and inside a full overlay of the canonical crate (which also parses every module) |
| Round trip: patch applied to a scratch copy of the canonical crate | applies cleanly; all 10 files byte-identical to the candidate; patched oracle runs |
| Dead-code audit (non-test items used at least once outside tests) | clean after `Elem::apply` went `#[cfg(test)]` and canonical `commutes_with_flip` became a cross-check in the 09 scoreboard |
| Glyph sweep | every non-ASCII character in candidate strings (² ³ · → −) already renders in canonical strings. The glyph probe auto-captures kit text, so main.rs glyph lists need no change |
| Absolute paths or usernames in staged files | none (only root's `ORDERS.md` has them) |

## Not done (root's gates)

- **Rust is uncompiled.** No `cargo build`, `clippy -D warnings`, `cargo test`
  (39 tests in the rebuilt modules, 33 of them new), `facts-check`, glyph probe
  or `ubs`. The seven files grow from 1,681 to 6,481 lines (d4_group.rs
  included), so I expect compile fixes. Check fmn API arity first:
  `indicate`, `fade_in` shift, `ArcBetweenPoints::angle`,
  `replacement_transform`, `always_redraw`, the AnimBuilder chains.
- **`cargo fmt --check` will still fail on this crate**, from root-owned
  canonical modules: arithmetic, axioms, composition, d4, eight_construction,
  products, tile_actor and tile_model. None of my files is involved.
- No render, contact sheet or frame inspection, so layout is computed but
  unseen.
- No TTS, ASR, listening, fine-sync or independent AV review. Sentence anchors
  will fall back to estimated starts until real takes exist.

## Strongest unresolved teaching defects

1. **Unseen density in 16_15–16_17 and the 17 grid.** An 8×8 table with
   headers at 0.66-unit cells is shown beside labels. The Hamming film puts a
   16×8 frame grid with row labels at x −6.35 and right labels at 4.95. The
   coordinates fit the frame on paper, but legibility at 4K and collisions
   with the side text are unverified.
2. **Algebra-heavy stretches.** 09_7 (sliding proof, 7 sentences), 14_6 and
   14_15 (correction g⁻¹k) carry most of their weight in TeX lines. They have
   moving pictures (sliding s, an s·H swap), but they are the likeliest places
   for a viewer to lose the thread. They need a first-viewer watch.
3. **17_18's real-world claim has no cited source** ("ECC memory commonly
   uses longer codes from this family, with one extra check bit"). It is
   SEC-DED in the Hamming/Hsiao family. Root should attach a citation or soften
   the wording before publication.
4. **Pacing is unmeasured.** 08_11 (10 sentences) and 14_8 (8 sentences) are
   dense. Breath and cut timing can only be judged after synthesis.
5. **Binary is used before it is taught.** 17_16 says the syndrome "spells
   the position of the flipped bit in binary", and the film never introduces
   place values. The syndrome stack pairs each position with its three-bit
   string (5 → 101) and XORs the strings, but a viewer without binary must
   take those pairings on trust. A one-beat
   place-value card (4 + 0 + 1) would close this gap, at the cost of one
   sentence and one more TTS unit.
