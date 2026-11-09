# Apportionment narration review 197 (cc1): cues 10_1 to 15_5

Written 2026-11-08 (UTC) by the cc1 pane. It is a text-only review, written before root generates the remaining Frederick cues. I edited no script, storyboard, source, request, recording, Bead or TODO.

## What this review is and is not

- **What was checked:** the exact cue texts and the request inventory; an independent recomputation of every number, in exact rationals; reading of the cited sources.
- **Not done:** no takes heard, no pictures or movies viewed, no build, render, synthesis or listening. Nothing here is fine-sync or independent final AV acceptance.
- **Independence limit:** this lane wrote earlier apportionment handoffs, including the chapter 12 tile proof (bead .30 handoff 2) and a chapter 12 picture review. So the check of 12_5 to 12_9 is a re-derivation by the same author, not an independent second opinion. Chapters 10, 11, 13, 14 and 15 were checked against my own arithmetic and the primary text of the Gölz, Peters and Procaccia paper.
- **10_1:** submitted once at 2026-11-08T03:04:32Z (`frederick-generation89/10_1-submitted.json`). I found no defect in its text and propose no regeneration.

## Inputs (sha256)

| input | sha256 |
|---|---|
| `explainer_corpus/apportionment/script.tsv` | `29aa2993426bb323e5723162e446f1cf70d00836b8bc91ebb6503ad3ca48cf07` (equals requests04 `script_sha256`) |
| `narration_lab/…/elevenlabs-apportionment-rebuild-20261104/narration-frederick-requests04/requests04.json` | `1dd6c3e0ed8ce275acdcbe7f8da24f3cf5103adeb2ca86a6ab893262f2be9bbd` |
| `explainer_corpus/apportionment/narration_units.json` | `b1f8129c…` (equals requests04 `units_sha256`) |
| `explainer_corpus/apportionment/docs/storyboard.md` | `6324895bc5e9a821ee9efd4fa33945ab58729cd02bf9b2d2d1eca7b74f09b566` |
| `explainer_corpus/apportionment/docs/description.md` | `0c87d6bcdabea1120bdd0b89cc45ffe370c6f1c5de363446aa6959a5eb84cf75` |
| `explainer_corpus/apportionment/src/chapters/rounding_rules.rs` | `d5baaecdde2dd7d50b1cc8cbef60b177f6bf2e647709c181c3b9022aa04163f5` |
| `explainer_corpus/apportionment/src/chapters/no_rule.rs` | `aceba2952c63fd44692d5015958eb76e1ce96c87f63ac17cf7212c62815cbd5e` |
| Gölz, Peters, Procaccia, arXiv:2202.11061v2 PDF (fetched 2026-11-08, read as `pdftotext -layout` output) | `3fc44d995da7ab4fe6de3a7f16dc5e65cd13e19161617875d7bb9227d6124f5e` |

**Request texts:** requests04 holds 93 requests. Each `text/<cue>.txt` equals its script row byte for byte, and each word count matches. 11_2 is at the 100-word `max_words` cap, so any change to 11_2 must not add words. None is proposed.

## Findings, most severe first

### F1 (Medium): 14_8 calls the lottery "a way out" just after the impossibility, but the same paper proves a lottery does not escape it

**Current 14_8 (33 words):** "The same paper shows a way out. If the rule may draw lots, it can keep the quota promise, never take a seat back, and give every town exactly its share on average."

**What the paper says.** 14_7 has just proved that no rule keeps the quota promise and the population promise together. "A way out" then tells the viewer that drawing lots escapes that theorem. The paper says the opposite:

- Introduction: "In Section 3, we first show that no such randomized methods exist for population monotonicity. This impossibility is not due to randomization or ex ante proportionality, but due to the fact that population monotonicity and quota are outright incompatible."
- §3.1: "Since a method satisfying quota and population monotonicity would be a random choice over such solutions, no such method exists either."

The lottery's first two properties (quota in every draw, no take backs) are already kept by the deterministic quota method of chapter 12. What drawing lots adds is the third property: every town gets its exact share on average. The picture card ("a lottery rule (same paper)", three checks, Ash 54% / 46%) is accurate. The defect is the spoken framing. 15_5's "now you know exactly which promises can be kept together and which cannot" would also mislead while F1 stands.

**Smallest proposed correction (49 words):**

> The same paper shows what a lottery can add. If the rule may draw lots, it can keep the quota promise, never take a seat back, and give every town exactly its share on average. But even a lottery cannot keep the quota promise and the population promise together.

- **Triggers:** `no_rule.rs` waits on "it can keep the quota promise" and "exactly its share on average". Both are unchanged, and both are still the first occurrence in the line.
- **Rules:** the line follows the spoken-text rules (no digits, colons, dashes or parentheses).
- **Optional, root's choice (picture):** a fourth row with a red cross, such as "quota + population promise: still impossible, even by lot", shown at "But even a lottery". This would need a new phrase wait. Without it, the spoken correction alone is enough.
- **Comment and description:** the code comment `// 14_8 the way out: draw lots` and the lottery sentence in `description.md` could get the same caveat. Both are root's files.

### F2 (Medium-low): 10_2 says "any piece of a share", but Adams rounds quotients, and the cue's own result contradicts the sentence

**Current 10_2:** "John Quincy Adams proposed rounding up instead. Now any piece of a share earns a whole seat, so the lean flips toward small towns. At fifteen seats Adams gives two, four and nine. Cedar falls below its share of exactly ten."

**Why it is wrong.** In the film's vocabulary (02_2), "share" is the exact quota. Cedar's share at fifteen seats is exactly 10, with no piece, so read literally the sentence gives Cedar 10. Rounding up each share would give 2 + 4 + 10 = 16 seats. Adams rounds the **quotients** at a raised yardstick. In my check, any yardstick from 26,000/9 ≈ 2,888.9 up to but not including 3,000 gives 2, 4, 9, which is how Cedar ends below its share. 10_1 ("Jefferson rounds every quotient down") and 10_5 ("once a quotient passes") already use "quotient".

**Smallest proposed correction (one word, still 41 words):** replace "Now any piece of a share earns a whole seat" with **"Now any piece of a quotient earns a whole seat"**.

- **Triggers:** "proposed rounding up", "lean flips toward small towns", "At fifteen seats Adams gives" and "Cedar falls below" are all unchanged.
- **Stale comment:** the code comment `// 10_2 Adams rounds up: any piece of a share earns a seat` would then be stale (comment only).

### F3 (Low, optional): 14_7's "With four, as of that paper, nobody knew" needs "without the extra assumption" to be exact

**Current:** "…With three towns, Webster's rule keeps both. With four, as of that paper, nobody knew."

**What the paper says.** "Whether impossibility holds for n = 4 remains an open question. The original impossibility result of Balinski and Young (1982) (which, in addition, assumes the order-preserving property) uses only n = 4 states." In context (14_7 is about the case without the assumption) the line is defensible. But a viewer who has just heard 14_1 can take "nobody knew" to cover the version with the assumption, which Balinski and Young settled with four towns.

**Proposed correction (+4 words, 48):** "With four, without the extra assumption, nobody knew as of that paper." The trigger "With four" is unchanged. If root adopts it, the card "4 towns: open, as of that paper" could add "(without the extra assumption)".

### O1 (Low, optional teaching hazard, not an error): two spoken "three point four six" values next to "Hill agrees with Webster at fifteen"

02_2 gives Birch's share at 15 as "three point four six". 10_6 gives Hill's 3-to-4 threshold, √12, as "about three point four six". 10_5 says Hill agrees with Webster at 15 seats, which needs Birch to round **down**. That is correct: 9,000/2,600 = 3.4615 < √12 = 3.4641. But a careful viewer hears two equal numbers and may think it is a tie or an error. The smallest fix is on the picture only: show "Birch 3.4615 < √12 ≈ 3.4641" at "Between three and four". As a spoken alternative, 10_6 could say "the square root of twelve, about three point four six four, just above Birch's share at fifteen seats" (+9 words; the trigger "Between three and four" is kept).

## Checked, no change proposed

All numbers below were recomputed independently in exact rationals (`Fraction`), not taken from `oracle.py` or `facts.rs`. Divisor rules were computed as signpost lists, and ties at the h-th signpost were checked (none in any case the film narrates).

| cue | verdict | what was checked |
|---|---|---|
| 10_1 | OK (already generated) | "rounds every quotient down"; family framing consistent with 10_2–10_7 |
| 10_3 | OK | question only |
| 10_4 | OK | Webster 15/16/17 = 2,3,10 / 2,4,10 / 2,4,11; Webster within quota for these towns at every house size 1–200 and house monotone 3–100 |
| 10_5 | OK | 1941 law (also stated in GPP §1); √2 ≈ 1.41; Hill 15/16/17 = Webster's (see O1 for the 15-seat margin) |
| 10_6 | OK | √6 ≈ 2.4495, √12 ≈ 3.4641, √0 = 0; √(n(n+1)) < n + ½ with the gap shrinking as n grows |
| 10_7 | OK | Hill at 3 seats = 1,1,1; Cedar share 3 × 26/39 = 2 exactly; every divisor rule house monotone; each can break quota for some map |
| 11_1 | OK | question only |
| 11_2 | OK | argument checked step by step: a deficit of ≥ 1 forces yardstick > exact, every other town < share + ½, two others < 1, total < h (contradiction); mirror for excess; attribution matches GPP §3.1 citing Balinski and Young 1982, Prop. 6.2; 100 words, at the cap |
| 11_3 | OK | 2,000/5,000/8,000/27,000 at 14: shares 2/3, 5/3, 8/3, 9; nearest rounding gives 15 |
| 11_4 | OK | halfway yardsticks 4,000 / 3,333.3 / 3,200 / 3,176.5; big town first; Webster 14 = 1,2,3,8, the only allocation with 14 |
| 11_5 | OK | question only |
| 12_1–12_4 | OK | quota method (eligibility a+1 ≤ ⌈share at h⌉, Jefferson priority) gives 1,3,10 at 14, 1,4,10 at 15, 1,4,11 at 16, 1,4,12 at 17; differs from Jefferson only at 6 and 15 (within 1–20); within quota 1–200; B4 2,250 > A2 2,000; "quota method" and 1974 confirmed in the PNAS abstract (PMC433936) |
| 12_5–12_9 | OK (same-author re-derivation) | tile lengths 39/4, 13/3, 3/2; passed/entered = ⌊share⌋/⌈share⌉; Jefferson order = tile end order (end = 39,000 ÷ signpost); "soonest-ending entered unheld tile" equals the B–Y rule; the counting argument (h − s handed-out tiles plus the missing one, all inside [s, h]; at most (h − s)·pᵢ/P whole tiles per row) is sound, including s = 0 and s = h |
| 12_10 | OK | Hamilton 2,3,10 and quota method 1,4,10 both within quota at 15 |
| 13_1 | OK | matches the Robinson–Ullman pairwise form (GPP §2), house fixed |
| 13_2–13_4 | OK | Hamilton 16 before 1,4,11, after 2,4,10; shares 1.5797, 3.8481, 10.5722; Ash tail beats Cedar's |
| 13_5 | OK | signpost monotonicity argument |
| 13_6 | OK | B4 2,375 passes C11 2,363.6; C13 2,007.7 passes A2 2,000 (tie before); Ash 3,900 / 1,950: no rank changes; Jefferson 16 = 1,4,11 before and after |
| 13_7 | OK | — |
| 14_1 | OK | order-preserving wording and Theorem 1 ("No (deterministic) apportionment solution…") match GPP §3.1; v1 2022, v2 2024 (Operations Research) |
| 14_2 | OK | three towns: Webster keeps both (GPP, citing B–Y 1982 Prop. 6.2) |
| 14_3–14_6 | OK | GPP Table 1: totals 1,000 / 1,178 / 914; shares 8.24, 0.44; 6.9949 (≤ 7), 1.8846 (≤ 2), so ≥ 1 seat for villages 2–4; 9.0153 (≥ 9), at most one seat for villages 4–5 |
| 14_7 | OK except F3 | five towns; generalises to more (GPP) |
| 15_1, 15_2 | OK | consistent with chapters 4, 9, 10, 13 and 14; "the theorem" covers the quota method because it is deterministic |
| 15_3, 15_4 | OK | Webster with Cedar 26k → 30k → 300k: Ash never gains (2,3,10 → 1,3,11 → … → 0,0,15); 15-seat cutoff 2,666.7 → 2,857.1; Ash's next signpost 1,600 |
| 15_5 | OK once F1 is fixed | closing claim relies on 14_8 not implying an escape |

## Notes for root (all yours to decide)

1. Script rows are generated from `docs/storyboard.md` (per the script header), so any adopted wording means a storyboard edit, script regeneration and a new request manifest (requests05). That requires rebinding the units, map and holds, as in the 05_2/08_6 clarity repair.
2. **Order of generation if adopted:** F2 affects 10_2, the next cue to be generated. F1 and F3 affect 14_8 and 14_7, so there is time.
3. `unit_id` values in requests04 keep the legacy prefix `oliver_` while the voice is Frederick. It is harmless naming, noted only so a reader is not misled.
4. **Not covered here:** TTS pronunciation of Gölz, Procaccia and Balinski; any recorded take; cues before 10_1.

## Separate note (complex lane, not apportionment)

`production-repair17/boundary.rs` (sha256 `c9e6b5ba84c4cf5ae327a8268d67c4f134484be8538cb99bdf1828ebe051f630`) is my partial 09_1 edit from 2026-11-07, stopped when the quota ran out.

- **State:** new constants and helpers were added; the original 09_1 timeline is unchanged.
- **Not usable:** it was never compiled and would fail Clippy on unused code.
- **Root's guard:** root's `boundary-label-build52.py` and `two-label-build164.py` use this hash as the "before" guard and replace the file with `boundary-label-repair52.rs`.
- **Left as is:** I have not touched it since. The `09_1-script.txt` (a24b7127…) wording is unchanged.
