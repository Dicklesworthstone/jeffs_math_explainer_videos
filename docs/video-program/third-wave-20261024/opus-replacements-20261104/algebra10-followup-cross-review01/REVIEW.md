# Algebra chapter 10 follow-up patch: cross-review

**Reviewer:** RubyRaven, for BlueSalmon's Mail 2018, bead `mx-video-program-program-87do.64`.
**Reviewed:** WhiteSwan's `algebra10-repair03-picture-review01/patches/cyclic-picture-followup.patch` (`dbb3a64a…`) against canonical `cyclic.rs` `bee439fd…`, with their `REVIEW.md`, the source, the script, the map and the repair03 frames.
Narration stays ElevenLabs Oliver. No audio was generated or listened to.

**This is a source and baseline-picture review. Nothing has drawn the patch, so no frame shows its result. Nothing was compiled. It is not listening, fine-sync or final AV acceptance.**

## Verdict

**Take the patch with two small corrections, and keep F.** All five fixes point the right way and the maths holds. One fix waits one sentence too early, and one new label is ambiguous.

| Item | Verdict | Evidence |
|---|---|---|
| A, 10_2 second meaning of order | **Correct** | Source; replay; empty slot measured on frame 100; sheet 025 |
| B, 10_10 progressive reveal | **Correct, one label corrected** (C2 below) | Source; replay; slots measured on frame 560; sheets 126, 140, 080 |
| C, 10_4 `k_split` before the hop | **Correct** | Same lines in a new order; replay; sheets 049, 050 |
| D, 10_1 question held back | **Right idea, wrong sentence: corrected** (C1 below) | ASR word times; sheet 003 |
| E, 10_6 rows after the question | **Correct** | Source; replay; sheet 071 |
| Tests | **Sound by reading; +1 test, so 111. Not run** | Text checks; Python mirror with mutations |
| Facts | **Still 240 by reading.** `main.rs` and `oracle.py` are untouched; the oracle prints 240; the Rust side was not run | `checks/verify.py` |
| Script and voice | **Unchanged.** One file touched, same cue order, nothing to synthesize | Hashes; `k.say` order |
| Hold binding | **448 becomes 453, still correct with both corrections.** Also applies to root's stage02 map | Marker located in each result |
| F, the unvoiced star | **Keep, unchanged.** WhiteSwan's removal cost is understated | Sheets 046 to 054; source; script |

## Disclosure

- **This is not independent of lineage, and I do not claim it is.** As RubyRaven I reviewed repair02 (assigned in Mail 1696, delivered in Mail 1737) and proposed the patch root adopted as repair03. This patch sits on that base, and B builds on my F2 sentence waits in 10_10.
- **Who wrote the first native version is unsettled.** WhiteSwan attributes the rebuild (Mail 1021) to LavenderPike. A shared memory store also lists it under an algebra lane note. A Mail search did not settle it, so treat that code as possibly my own lineage too.
- **WhiteSwan and I are the same kind of agent and share a memory store,** so we can share blind spots.
- **My replay is not blind.** I wrote my own logic and imported nothing, but I had read WhiteSwan's REVIEW, the top of their timing script and its output first.
- **Past error.** In an earlier assignment (Mail 1992) I logged sheets I had not seen. Here each coverage note was written after its image was on screen, and `checks/seal.py` refuses a header time that differs from `sampling.json`.

## Evidence and limits

| Source | Supports | Cannot show |
|---|---|---|
| 15 sheets viewed: 003, 025, 046 to 054, 071, 080, 126, 140 (`coverage/sheets-viewed.tsv`) | Today's picture at sampled instants | The patched picture. The other 142 sheets are WhiteSwan's view, not mine |
| 61 frames decoded and measured, not viewed (`checks/verify.out` lists them) | Empty slots, ink boxes, still stretches, replay controls | The patched picture |
| Source, kit and engine, read | Order, run times, types by eye | Compile, types, borrows: **nothing was compiled** |
| `checks/verify.py`, 225 checks passing | Patch round trip, subsets, rustfmt, indices, hold, replay, mirror, corrections | Anything that needs a build or a render |
| Root's large-model ASR for take01 and take02 | Provisional word times. No take is selected | Listening, fine sync. The kit reads real sentence onsets from the narration's `report.json`, which may differ from the ASR. Phrase times use the kit's fallback |

Oliver times below are seconds into the cue, take01/take02. The model replays today's source on the scratch voice and lands within one sample of all 12 measured frame anchors. That is why I trust it on the takes, not proof of them. It rounds each short hop step up to a whole 30 fps frame, which is my assumption. WhiteSwan's `k_split` time today (14.60/14.53) omits that and mine reads 14.76/14.69. No conclusion depends on the difference.

My first run failed three checks. All three were errors in my checks: reverse-walk line arithmetic, assuming the map had one hold row rather than 16, and an over-strict diff count. None was a patch defect.

## What reproduced

- **Patch:** 12 hunks, +92 −17, reaching `bee0eb2c…` compared in full, by my own walker and by `patch --fuzz=0`, forward and reverse.
- **Subsets:** each of A to E applies alone, reaches WhiteSwan's prefix and passes `rustfmt --check`, as does the whole result. A misformatted copy is refused, so the check can fail.
- **Waits:** `k.say` order is unchanged. Every `until_sentence` and `until_phrase` is in its own cue, in range and in order. A planted out-of-range index and a planted misplaced phrase are both caught.
- **Pictures:** each new mobject is made once, shown once and hidden. Every picture beat in the five cues ends inside the voice on all three voices, so no cue should start later.

## The five fixes

### A. 10_2: correct

- `Whole clock group: 12 elements` arrives 0.36/0.16 s before the voice says those words, then both meanings pulse at "These are different meanings of order". That ends at 27.52/27.20, inside speech ending 29.44/28.80.
- The unchanged stretch today is 9.80/9.16 s, which matches WhiteSwan's figures.
- Width by the law is 3.88, spanning x 0.36 to 4.24. I checked the 1.25× pulse too, which WhiteSwan's REVIEW does not report: it reaches x −0.13, clear of the ring at −1.08, and stays 41 px from `ord(8)=3` with both pulsing.
- 10_3 draws `divides` in this same slot, but only after the label has left in its first line.

### B. 10_10: correct, with one label corrected

- The longest unchanged gap falls from 17.60/17.40 s to 7.46/7.26 s. `squares` and `no` keep their times.
- Slots on frame 560 are empty. New rows clear the table: its ink ends at x −1.59 and the leftmost new row starts at 0.39. All five right-column rows leave at 10_11.
- The tint rule takes exactly 5×5, 7×7 and 11×11, and without `value != 1` it would also take 1×1. On frame 560 the hero and ink colours are clearly different (254,215,113 against 234,238,242).
- Brute force confirms the order-two claim, that no element generates all four, and that the square's rotations do have an order-four element.
- The rotations label lands 0.08 s before "so this group is not", which is well placed. It leads "four rotations" by about 3 s but belongs to that clause.

**Defect, C2 below.** The new `ord(5)=ord(7)=ord(11)=2` is true in `U(12)`, but it uses the chapter's own symbol against its own table.

### C. 10_4: correct

The same lines in a new order. `k_split` now starts at 12.56/12.49, which is 0.78/0.81 s after "and k as" and 1.60/1.45 s before "k prime". `coprime` keeps its time on all three voices.

### D. 10_1: right idea, wrong sentence

**WhiteSwan's "add eight at 13.36/13.28" is the start of sentence 3, whose first word is "On". The ASR hears "add eight" at 16.00/15.90.** See C1 below.

### E. 10_6: correct

Only the `k` row stands during the question. The gcd and order rows, then the rule's pulse, finish at 10.72/10.70. The boxes (14.03/14.17) and every later beat keep their times. The rule stays on screen during the question, as it does today: that is the formula, not the answer.

## Corrections staged

Both apply on top of WhiteSwan's patch, in either order, change no line count and need no new test.

| Result | sha256 |
|---|---|
| Patch alone | `bee0eb2c55f2b9ad43fcd10b4f0993947b7fd104cda963e63ed27ccfefa4f17f` |
| Patch + C1 | `5005476e2517668edcd66587d0eb8e108b1da45cf0898fdab174aa34d8e1851e` |
| Patch + C2 | `b1e1e7ca8c3206e19d9fe328551f50bab7473935f954476f05cf93a082358090` |
| Patch + both | `e948e2b909c7a883a2dbb2d7ed4e96c403897ecc1ffdad6e9b29203f03393202` |

### C1, low-medium: `patches/question-at-predict.patch`

The patch's comment says "The question waits for the move it asks about". At index 3 it doesn't: `Add 8: how many positions?` arrives 2.64/2.62 s before the voice says "add eight", and 5.74/5.56 s before "predict how many". This is smaller than the 13 s lead it replaces, but it is the same fault.

**Change:** `until_sentence(stage, "10_1", 3)` becomes `4`.

| | Question starts | "add eight" | "predict" |
|---|---|---|---|
| Patch | 13.36/13.28 | 16.00/15.90 | 19.10/18.84 |
| Patch + C1 | 17.64/17.52 | same | same |

- The move has just been said, and the question leads "predict" by 1.46/1.32 s.
- It stays at least 4.36/4.24 s before the voice ends, then through the breath and the 3 s hold. Its reading floor is 1.73 s.
- If real onsets follow character shares instead of the ASR, sentence 3 lands at 13.00/12.86 and sentence 4 at 16.89/16.71. Index 4 is right either way.
- A phrase wait would land at about 16.86/16.75 by the fallback, but it needs a new constant and a table row to gain 0.8 s.

**Trade-off:** the clock stands without any change for about 4 s longer, roughly 17 s rather than 13 s on Oliver, while the voice defines "generated". Today's picture is still for about 25 s, so this remains an improvement. WhiteSwan's point stands that adding motion there is a separate design choice for root.

### C2, low-medium: `patches/unit-orders-names-group.patch`

`ord(5)=ord(7)=ord(11)=2` is true under multiplication. But 10_6 has just boxed `ord(k)` = 12 under 5, 7 and 11, beneath `ord(k)=12/gcd(k,12)` (sheet 080). About 100 s later, the same symbol with the same arguments gives a different number, and nothing on the label says the group has changed. A viewer who remembers the boxes sees a contradiction. The group's name is only further up the page.

**Change:** a text row, `U(12): 5, 7 and 11 each have order 2`, matching chapter 12's existing `U(12): no element has order 4` (`isomorphism.rs:165`).

- Width 4.49, spanning x 0.26 to 4.74. The slot is empty on frame 560. It clears `squares` by 31 px and `no` by 34 px.
- The glyph list already covers every character. On Oliver it has at least 11 s against a 2.40 s reading floor.

## Tests

- **New test, by reading:** types look right. `Vec == array` is already used, `pow` infers `u32` and 11³ fits in `i32`. A Python mirror passes on the film's group and fails on a wrong list, a short list, `U(5)` and `U(10)`. `U(8)` rightly passes, because it also squares to one.
- **List changes:** `PHRASES` declares and lists 13. Every phrase the scene waits for is in that list. The new `("10_1", 5)` row matters, because an out-of-range index would silently return Ok. It also makes C1's index 4 safe.
- **Low, no change needed:** the test overlaps `clock_orders_and_noncyclic_units_match_examples` (`main.rs:706`), which already pins squares and closure. It adds the tie to `generators(N)`, the gcds and the powers. It does not pin the scene's literal `labels` or the tint rule, the same exposure `U(12)` has today.

## Hold binding

- `k.hold(stage)?;` is unique in `cyclic.rs`: line 448 today, **line 453** after the whole patch, with or without C1 and C2. `stage.wait(3.0)` and `k.say("10_2")` still follow it.
- If root takes a subset, the line is 450 with B alone, 451 with D alone and 448 with A, C or E alone. Recompute for any other mix.
- Only `opening` and `question` appear before the hold. Nothing in the window shows a dot, trail or answer. Sheet 003 and pixels show today's window is clean. The patched window is source only.
- **Root's stage02 needs it too.** `combined-repair-stage02` holds this same source and binds it at line 448 through `narration-direction-map-oliver26.json`, not canonical's `oliver24`. Rebind whichever map is live, and `narration_units.json` follows that map's new hash.
- **It may not fail loudly if missed.** No code in `tools/*.py` reads a hold's `marker` key, so a stale row may pass unnoticed. Root's logarithm lane has bind scripts that recompute these, which I did not run.

## F: keep the star

**Recommendation: keep it, unchanged, with no patch.**

**The teaching reason:** the star is the only picture in the chapter that makes `d` a count rather than a calculation.

- **It explains the factorisation that follows.** The voice says "Let d be the greatest common divisor", `d = gcd(12,8) = 4` appears, and four walks draw from it. Then `4 orbits, 3 positions each` stands directly above `n = d n' = 4·3` (sheets 049 to 053), so "Write n as d times n prime" arrives with its meaning already on screen. Retired, `d = 4` is bare arithmetic.
- **It answers the question 10_2 leaves.** +8 reaches 3 of 12 hours, and the star shows where the other nine went. That sets up 10_6's "A shared factor traps the walk in one smaller pattern".
- **It carries a word the film needs.** This label is the only place "orbit" is on screen in the whole film, and the first spoken use is in 10_11's recap: "repeating a move produces an orbit". Retired, a recap introduces the term.
- **It prepares chapter 14.** The four triangles are exactly the four shifted copies of {0,4,8}, and 14_8 says "The twelve-clock is four cosets of three".

**Why being unvoiced doesn't sink it:** the order is right, since the voice and `d` come first. On Oliver the star finishes inside the sentence, stands whole for 2.05 s, and the label is full for 1.75 s alongside it against a 1.73 s floor. The label then stays, so reading isn't cut off. Retiring it would leave about 4.2 s with no change after the `d` row.

**Downside, low:** the label outlives the three triangles by at least 10.64/10.30 s. For that time `4 orbits` stands beside one triangle and three arcs, and 12 = 4·3 reads two ways: 4 orbits of 3, and 3 blocks of 4 (sheet 052). Both are true. I considered removing the label with the star and rejected it, because it would give up the adjacency to `4·3` that is the main reason to keep F.

**WhiteSwan's cost of removal is wrong, if root still prefers to retire it.** "Used only by tests" fails for both helpers:

- `orbits` feeds the facts at `main.rs:279`, outside any test, and two tests. It stays, and `cyclic.orbits` would then be facts about nothing on screen.
- `orbit_color` has no test use. Only the scene calls it, so it must be deleted, or Clippy's dead-code warning blocks the build.
- `triangles[0]` must be rebuilt alone, because it stays after the star (sheet 048), 10_5 pulses it at the formula and 10_6 hides it.
- `colored`, `count` and the comments naming four orbit colours (`:1408`) and the header (`:2`) would all need changes. The 10_5 hide list would too.

## If root integrates

1. Apply `cyclic-picture-followup.patch`, then either or both corrections. Both apply at `--fuzz=0` on the full patch. On a subset, C1 needs D and C2 needs B.
2. Rebind the 10_1 hold to line 453 with the new source hash in the live map, and let `narration_units.json` follow.
3. Expect 111 tests and 240 facts. Nothing to synthesize, and all Oliver takes stand.

## What the first candidate frames must confirm

WhiteSwan's list stands, with these changes:

- **10_1:** no question before sentence 4. On the scratch voice that is 16.55 s on the film clock (cue at 0.60 plus 15.95), and the hold is unchanged.
- **10_10:** the fourth right-column row reads `U(12): 5, 7 and 11 each have order 2` and clears its neighbours.
- **The 10_2 and 10_10 pulses and tints:** half-second samples can miss them, so add frames around each.

## Still required

Native compile, tests and Clippy; a guarded preview; real Oliver timing; listening; independent final AV. Those are root's, and none is claimed here.
