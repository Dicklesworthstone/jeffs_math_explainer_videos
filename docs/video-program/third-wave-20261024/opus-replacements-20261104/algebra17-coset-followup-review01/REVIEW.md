# Algebra 17 coset bridge: follow-up review

RubyRaven, for BlueSalmon, Mail 2049, Bead mx-video-program-program-87do.64 (root alone edits Beads).

Reviewed: root's `applications-repair-stage01/candidate02` (source `93dd73b8`, script `c065e38e`), under `elevenlabs-expanded-algebra-20261103`.

**Verdict: the maths is right and the sentence index is right, and the timing model shows no problem. I recommend four small changes before the build.** The spoken question leaves out which kind of coset it means, and it is false for the other kind. The picture also removes `hH=H` at the moment the answer uses it. Three fixes are staged as source patches and one as a script patch. No Oliver take of chapter 17 exists, so a wording change loses no recording.

## Disclosure

- **I am not independent of this chapter.** I wrote the original 17 patch that root's bridge sits on. My picture review also raised two of the points this bridge answers: N5 (`hH=H` stops short of `(gh)H=gH`) and N7 (prompt 3's "keep it" is vague). I did not write root's bridge or its check. My maths code imports nothing from root's, but I had read root's check output before I wrote mine, so the agreement between them is not blind.
- **Correction to my earlier numbers.** My picture review's spoken times, and BronzeJaguar's scale table in the C1C2 review, put the 0.4 s pad inside the sentence share. The kit leaves it out (`estimated_starts(text, speech(text))`). A decoded preview settles it: the 17_8 label's first rise from background is at 204.033 and it clears my ink threshold at 204.067, against 204.002 by the kit's rule and 204.328 with the pad. So those times run about 2% late: 17_12's third prompt is 372.313, not 372.528, and 17_13's sentence 1 lateness on the C1C2 text is 1.06 s, not 1.02 s, at scale 1. Neither review's conclusion changes.
- An earlier integrity error of mine, disclosed in Mail 1992, is why I log a sheet only once the image is on screen and check its header times against `sampling.json`.

## The five questions

| Mail 2049 asks | Finding | Evidence |
|---|---|---|
| Precise orientation | **Equation right. Spoken question imprecise: F1.** | My permutations, 16 and 12 cases; film's own `Hr` counterexample |
| Notation introduced | **No new letters, but the only binding of `h` leaves: F2.** `g` is never bound at question time: F3 | Source; 14_4, 14_15, `cosets.rs:495` |
| Question and answer teaching | **The answer answers the question.** One law goes unspoken: W2 (optional) | Script, sentence by sentence |
| Sentence index | **Correct, but nothing guards it: F4** | My split; mutations; kit returns Ok when out of range |
| Reading time | **Holds at every scale modelled** | Replay, controlled against cue sheet and decoded frames; no Oliver timing |

## Findings

### F1. The spoken question does not say left coset, and it is false for a right coset. Recommended.

17_12 sentence 3 says "multiplying a coset representative on the right by a subgroup element does not change that coset." The film named both kinds in 14_14: "left and right cosets need not agree." For a right coset the claim fails.

- In the film's square, with H = {e, s}: Hr holds r and s r, as 14_14 says. Multiply r on the right by s, and H(rs) is a different set. (rs)H = rH holds on the left, with the same two letters.
- `H(gh) = Hg` fails in 4 of 16 cases in the square and 4 of 12 in S3. `(gh)H = gH` holds in all of them.
- The viewer gets the side only in 17_13's equation, after the think time. A careful viewer who tries it on 14_14's `Hr` finds a counterexample during the pause.

The old wording ("adding") could not hit this, because addition has no side. The side matters only since root's narrowing to multiplication, which I think was the right change. Root's own source comment already says "left coset".

**Staged wording (reporting only; root decides and binds), `patches/left-coset-wording.script.patch`:**

> Finally, pick a left coset and explain why multiplying its representative on the right by a subgroup element does not change that coset.

That is 4 more words and still five sentences. It keeps "left" and "right" apart by clause. The shorter fallback is "multiplying a left coset's representative on the right…" (1 more word), but no line of the film says "coset's", so its possessive is untried in Oliver and in ASR. I recommend the longer form. I have not heard either.

### F2. `hH=H` leaves just as the answer uses it. Recommended.

At sentence 3, line 683 fades out `c`, then `(gh)H=g(hH)=gH` takes its place. Two things go with it:

- the reason for the second equals sign;
- `(h∈H)`, the only on-screen binding of `h`. Without it the equation reads as a claim about any `h`, and that is false: with the non-subgroup {e, r}, `(gh)H = gH` fails.

The voice has just spent its longest sentence on this step, and the picture then takes it away. There is room to keep both.

**Staged, `patches/keep-the-step-in-view.patch`:** drop the fade-out and put the equation at `v(2., -1.75)`.

- By the law, the equation is 2.98 wide: x 0.51 to 3.49, y −1.91 to −1.58.
- That slot is empty on baseline frame 920, and the same box raised to `c`'s row does find ink.
- It clears `c` by 39 px, `d` by 44 px and the clock by 175 px.
- `c`, `d`, `a` and `b` share these coordinates in baseline and candidate02, by string match.
- The width law is measured on `c` itself: `hH=H` 1.14 by law, 1.14 by ink; `(h∈H)` 1.09 by law, 1.04 by ink.

### F3. The third prompt names neither the side nor the product. Recommended.

"Pause and work them out" sends the viewer to the screen, and the screen still says "3. Why does changing representative within a coset keep it?" That sentence is true of either side, so it is not wrong, but it drops the orientation root added to the voice. It also leaves `g` and `h` unbound until the answer.

**Staged, `patches/prompt-names-the-side.patch`:**

> 3. Why does gh name the same left coset as g, when h is in H?

- It restates the claim and leaks nothing: `hH=H` does not appear.
- It uses 14_5's verb ("name the same coset").
- It binds the answer's letters at question time, with `h` visibly on the right.
- It is 7.44 wide at 27 pt, ending at x 3.84, against 7.28 for prompt 2.
- The letters are plain roman, as `H` already is in the film's text labels such as "Same coset: the correction lies in H". Whether roman `gh` reads well needs root's frame check.

### F4. Nothing would catch a wrong sentence index. Recommended.

The index is right today (below). But `until_sentence` returns Ok when out of range, and no test names 17_12 or 17_13. On the baseline text, index 3 would land on "These are small examples" without any error. A script and source that drift apart would therefore fail silently.

**Staged, `patches/waits-are-pinned.test.patch`:** one test, written like the test beside it, pinning the six waits by each sentence's opening words. It touches no scene line and adds 1 to the test count. Its Python mirror passes on candidate02, W1 and W2. It fails on the baseline script and when a colon becomes a full stop. The Rust itself has not been compiled.

### W2. The first equals sign is never spoken. Optional; I lean yes.

`(gh)H = g(hH)` is associativity, and root's check names it ("associativity plus hH=H"). The voice goes straight from permuting H to applying `g`, so it leaves out why the new representative's coset is that set at all. The picture carries the step. 06_6 has already taught it: "Associativity changes the parentheses while keeping the moves in the same order."

> Now apply the original coset representative on the left of every result: by associativity, the same elements form the same coset.

That adds 2 words and stays five sentences. If root declines, the equation still shows the step. W2 is not a patch: replace the sentence by hand.

## What holds

**Maths (my own permutations, rightmost move first).** `(gh)H = gH` holds in all 12 S3 cases and all 16 in the square. `hH = H` and `(gh)H = g(hH)` hold element by element. Neither subgroup is normal. The wrong side, `(hg)H = gH`, fails in 4 of 12 and 4 of 16, and my S3 list equals root's four counterexamples. The inverse clause in sentence 2 earns its place: a closed set of maps without inverses keeps results inside and still loses elements. The other reading convention leaves the identity intact. As root's check says, enumeration only illustrates; the general proof is associativity plus `hH=H`, and normality is not needed.

**Teaching.** 17_13's sentences 2 and 3 answer the question, in order. 14_15 taught the same argument with the same letters (`k` = g times correction h, `kH=g(hH)=gH`), so this is a real retrieval exercise. `(gh)H` adds no new letter; it writes out `k`. "The same elements form the same coset" is loose, but "permutes" just before makes it fair. Nothing shows an answer before the think time ends, by source; candidate pictures do not exist yet.

**Index.** Both cues have five sentences by the kit's split, which I wrote again and ran with mutations. `until_sentence("17_13", 3)` lands on "Now apply". The colon in "every result:" does not split. 17_12's three waits land on Find, Decide and Finally.

**Bindings.**
- `candidate` is byte for byte the C1C2 stack I reviewed.
- Root's patch round-trips at fuzz 0, by my walker and by patch(1).
- Only 17_12 and 17_13 change, only as Mail 2049 says.
- Units equal the script, and the map hash is what the units contract names.
- The hold row binds line 606, marker `k.hold(stage)?;`, followed by the authored 5 s.
- Stage02 carries the same source, script and hold row.
- Canonical is still the baseline.

## Reading time

This is scratch-rule arithmetic at voice scales, not Oliver timing. Measured Oliver/scratch length on 23 cues of this preview's other chapters: min 0.866, median 1.074, max 1.247. Chapter 17 itself is unmeasured.

| Scale | 17_12 s | 17_13 s | Think s | s1 late | s2, s3 late | Equation stays s | "the same elements" after equation s |
|---|---|---|---|---|---|---|---|
| 0.75 | 15.69 | 24.92 | 13.11 | 1.73 | 0 | 10.11 | 2.81 |
| 0.85 | 17.73 | 28.19 | 14.10 | 1.46 | 0 | 11.42 | 3.28 |
| 1.00 | 20.78 | 33.09 | 15.60 | 1.04 | 0 | 13.38 | 3.98 |
| 1.07 | 22.21 | 35.38 | 16.29 | 0.85 | 0 | 14.29 | 4.31 |
| 1.25 | 25.88 | 41.27 | 18.08 | 0.35 | 0 | 16.65 | 5.15 |

- The new sentence is 19 words, 7.12 s at scale 1. candidate02 adds 8.46 s to the chapter; W1 and W2 add 2.31 s more.
- 17_13 is 85 words, 87 with W2, and no sentence passes 24 words. 17_12 is 53, 57 with W1; its longest sentence becomes 23.
- With W1, W2 and patch A, think time, prompt reading and the equation's stay all still pass at every scale I replayed (0.75, 1.0, 1.07, 1.25).

Notes, with no patch:

- **N1.** Sentence 1 is 0.85 s late at scale 1.07. The C1C2 review recorded this lateness before the bridge (0.83 s with the pad, 0.87 s by the kit's rule), so it does not come from this bridge, and it does not carry into sentences 2 or 3.
- **N2.** The equation's conclusion is on screen about 4.3 s before the voice reaches "the same elements". The voice is walking the same chain, so I leave it. If listening says the picture jumps ahead, `until_phrase` at "the same elements" is the lever.
- **N3.** 17_13's last sentence still has no picture of its own (my N8). That now gives the equation room, so I would leave it too.

## Staged, and how to bind it

Source patches, each against candidate02 `93dd73b8…`, `patch -p1`:

| Patch | sha256 | Result sha256 |
|---|---|---|
| `keep-the-step-in-view.patch` (F2) | `d78de031300069a1` | `8e4f8c4934ba77955d14246aba2d3816fbd503c4a34de81e25f0fe92baaaeded` |
| `prompt-names-the-side.patch` (F3) | `8d6d7e0bb0a76d7a` | `b8b43e16630dd25d5e5cf0f6b65cbf10a0146b479faf8bd9d498b97d8f29eb3f` |
| `waits-are-pinned.test.patch` (F4) | `246a0d4c70525d3f` | `54d80092a5edca19a9a1f13a9076abe02370ad940be461c09cf1a03aed3fd298` |
| All three (they commute) | | `24bcac595ae9476519ca69fbe6558e5db0c60547b677eacb0c76df3b697d43e5` |

Script, against `c065e38e…`:

| Change | Result sha256 |
|---|---|
| `left-coset-wording.script.patch` (F1, patch `7f000171ae4c96c8`) | `6d3e6f263295c4718651143c37b6b7d855c1c4384ae6a34f609e13f116166ede` |
| With W2 by hand | `9c983cae910562d1ca75e41c316cb88f25dbb8fbc062f730ea87f223bc250400` |

If adopted, in the applications stage and in stage02 (which carries candidate02 unchanged):

- The hold row's `sha256` takes the new source hash. **Line 606 and its marker do not move** under any of the patches.
- With a wording change: the script row, unit `oliver_17_12` (and `oliver_17_13` with W2), the map's `scripts` hash, the 17_12 override's `after`, and the units contract's map hash.
- No take is affected. By name search there is no chapter 17 WAV under root's algebra folder, and `current-retained-coverage40.json` lists both cues as missing.

## What this does not establish

| Claim | Status |
|---|---|
| Compiles, tests pass, Clippy | **Not run.** rustfmt only, and text checks cannot see types or borrows. Root owns the build. F4's test has not executed |
| Candidate pictures | **None exist.** All seven sheets and two measured frames are baseline, whose prompts are centred, all up at once, and still say "equal" |
| Patch A's layout | Modelled from baseline ink and the law, not seen |
| Equation's glyphs | Same characters as `kH=g(hH)=gH` and `h\longmapsto gh` in chapter 14's source. I did not view chapter 14's frames, and `glyph_tex` lists neither |
| Oliver timing of the new sentence | **Open.** The scales bracket other chapters' takes, not this cue |
| How W1 and W2 sound | **Not heard.** Word choice here is judged from the page |
| Real-voice sentence split | I proved the kit's text rule. A narration report that split these cues differently is outside this check |
| Listening, fine sync, independent final AV | **Not done and not claimed** |

## Evidence

- `checks/verify.py`, 193 ok, 0 FAIL; output in `checks/verify.out`.
- `checks/build_patches.py` writes the patches by exact replacement.
- `coverage/sheets-viewed.tsv`: seven sheets viewed, 205, 208, 211, 223, 225, 230 and 236, with header times typed from each image and checked against `sampling.json`.
- 33 frames of the baseline preview are decoded into a temporary folder by `verify.py` for the timing-rule control; none are kept.
- `base.sha256` and `bundle.sha256` seal the inputs and this folder.
- My first run failed 10 checks. All were my own expectations, not defects in root's work: nine controls compared against older numbers that carry the pad (see Disclosure), and one assumed the glyph list holds a full stop. The full stop is in the old prompt, which frame 836 shows.

Nothing here touched canonical, the stage, Beads or narration, and no job was launched beyond that small decode.
