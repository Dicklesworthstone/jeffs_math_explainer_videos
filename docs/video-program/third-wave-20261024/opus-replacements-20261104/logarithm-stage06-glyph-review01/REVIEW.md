# Logarithm stage06 glyph review

LavenderPike, for BlueSalmon's Mail 2683. Reads guarded job `logarithm-stage06-glyph01`; I did not start or touch it. Narration is ElevenLabs Oliver, and nothing here changes a cue.

## Verdict

**No glyph failure.** I viewed all 51 pages against the rows each should hold. All 603 ink rows match their registry strings, with nothing missing or clipped. All 7 literals in Ruby's `glyph-delta.tsv` draw correctly, and the tight pairs I tested on them do not touch.

**Source coverage holds.** Checked literal by literal, the registry has every string the chapters draw. The one gap is the thumbnail, which is not a chapter (G1).

**Nothing blocks.** F1 is worth a one-line fix. F2 to F5 are low, and all of them predate stage06 except that `(ℓ)` reuses a glyph the film already had.

This is a result about probe sheets, not film frames. See "What the pages do not prove".

## Job receipt

| Item | Evidence |
|---|---|
| `resource-receipt.json` (`6f20b10065cb15dd`) | Phase preview, admitted, exit 0, 21:40:07Z to 21:42:48Z, no stop field |
| Inputs | `inputs_sha256` = `final_inputs_sha256` = `a346e3750520ca559d5e7332fdc2db23c73195b74ce33677184d2deb92a7dc31`, 226 files |
| `command.log` (`cf7b7d82c79e56be`) | `glyph probe: 588 sources, 0 failures`; all 13 chapters listed |
| `failures.txt` | 0 bytes |
| `registry.tsv` (`2523af1aec85f53f`) | 588 rows: 496 Text, 92 Tex. 587 captured from chapters, 1 supplement (row 66) |
| Pages | 51 plus `FMN_COMPLETE`. sha256 of the sorted hash list: `ea2c1a0f4d724011ffd62afb371ce7bd5c626b6c34de3b1a82bfe967923392a9` |

"0 failures" only means no unmapped character or TeX group error. The engine falls back per character, so it does not say which face drew what. That is why I read the pages and the font tables.

## Source coverage

`checks/registry_coverage.py` reads every string literal outside test code in the 15 source files and classes it against the registry. The chapter sources, `script.tsv`, `main.rs`, `kit.rs`, `glyphs.rs` and the binary (`a3a6110347784247`) all match the frozen inputs. Controls: a planted literal comes out unmatched, and the retired `singular A` is not on the registry.

| Class | Literals |
|---|---|
| Exact registry row | 402 |
| Template with holes, matching a computed row | 124 |
| Fragment of a longer row | 39 |
| Not display text (fact keys, scene names, ids, phrase hooks) | 618 |
| Unmatched, each read in place | 26 |

The 26 unmatched:
- **19** are error messages passed to failure paths, never drawn.
- **1** is a parse prefix, `undoing_growth.rs:114`, never drawn.
- **1** is `matrix_logs.rs:374` `"-b"`, which goes through `shown()` and is registered as `−b` (row 491).
- **5** are the thumbnail: its scene name and 4 strings (G1).

So no chapter string behind a branch is missing. Three limits on that:
- **6 literals** sit on the registry under another chapter than their own file, for example `unreachable.rs:183` `½`. Each string is on the pages through that other chapter. The probe did not capture them from their own file.
- **215 rows are computed**, not one literal. I read them on the pages.
- **Live numbers are samples.** `kit.rs:597,662,698` record the value at 1 fps under scratch timing. Against the previous job (583 rows), `0.27`, `0.40` and `0.49` appeared and `0.46` left. These are sampled values, not literals, and they will differ under Oliver. The characters they use are all covered.

`checks/registry_delta.out` lists everything added and removed: those numbers plus the 6 new strings on the registry, and the 3 old strings they replaced.

**Pages against rows:** `checks/page_map.py` found 603 ink bands for 603 expected rows, page by page. No row comes within 60 px of the right edge or 40 px of the left or bottom.

**Viewing:** `checks/viewing_proof.py` reads this session's transcript. All 51 pages came back to me as images, byte-identical to the files on disk. That proves delivery only. What I noticed is my claim, in the page table below.

## The new strings

| Literal | Source | Row | Page | Result |
|---|---|---|---|---|
| `y = x` | `undoing_growth.rs:479` | 584 | 49 | Correct |
| `the line goes on` | `complex_turns.rs:67` | 420 | 35 | Correct |
| `e^{L(z)} = z = e^{\ell} \;\Rightarrow\; L(z) - \ell = 2\pi i\,k,\ \ k \text{ whole}` | `lifting_loops.rs:79` | 565 | 47 | Correct. `e`, superscript `ℓ` and the pairs around `−` stay apart at every ink level |
| `\ell` colour key | `lifting_loops.rs:219` | inside 497, 565 | 41, 47 | No row of its own |
| `(\ell)` | `lifting_loops.rs:634` | 497 | 41 | Correct. Three separate components |
| `e^{tp} > 0 \text{ for real } t \text{ and } p\text{: } e^{tX} \text{ reverses no axis}` | `unreachable.rs:1309` | 575 | 48 | Correct. `e`, `t` and `p`, and `e`, `t` and `X`, stay apart |
| `\text{singular } S\text{: } \operatorname{det} S = 0 \quad\text{but}\quad \operatorname{det} e^{X} = e^{\operatorname{tr} X} \neq 0` | `unreachable.rs:1662` | 559 | 47 | Correct. `det` upright, `e` and `X` apart, colon tight |

Enlargements are in `evidence/delta-*.png`.

**Which face drew `\ell` (Ruby's question): Noto Sans Math.** From `checks/tex_faces.py`:
- U+2113 is in the cmap of Noto Sans Math and IBM Plex Sans only. No Computer Modern face has it.
- fmd-math's chain for it is Regular, then Symbols, then Italic (`faces.rs`, `default_math_chain`), which lands on Noto.
- The page agrees. Correlation of page ink with the glyph drawn from each face:

| Glyph on the page | Chain predicts | Noto Sans Math | IBM Plex Sans | CM Italic | CM Regular |
|---|---|---|---|---|---|
| `ℓ` in `L(z) − ℓ` | Noto | **0.922** | 0.469 | unmapped | unmapped |
| `ℓ` in `(ℓ)` | Noto | **0.934** | 0.458 | unmapped | unmapped |
| `ℓ` superscript | Noto | **0.853** | 0.331 | unmapped | unmapped |
| `L`, control | CM Italic | unmapped | −0.061 | **0.937** | 0.326 |
| `(`, control | CM Regular | unmapped | 0.827 | 0.340 | **0.924** |
| `z`, control | CM Italic | unmapped | 0.407 | **0.912** | 0.173 |

The other TeX characters from Noto are `× ↔ ⇒ − ∞ ≈ ≠ ≤ ⋅ ⟷ ⟺`. Row lists are in `checks/tex_faces.out`.

**Extents at film size,** scaled from each row's width on the sheet (`checks/gaps.out`). All are inside the film's stated safe area of ±6.90:

| Row | Size | x extent, frame units |
|---|---|---|
| 565 DIFFER | 28 | −1.50 to 3.90 |
| 559 singular S | 28 | −2.84 to 2.84 |
| 575 `e^{tp}` line | 22 | 2.20 to 6.41 |
| 497 `(ℓ)` | 22 | 3.32 to 3.57 |

## Findings

No edits made and no patch staged. The fixes below are proposals, unrun.

### F1, low: `so − 1 = 1 ?` sets its minus as a subtraction

`product_rule.rs:343`, `r"\text{so } -1 = 1\ ?"`, row 560, size 36, red, chapter 08. After `\text{}` the minus takes binary spacing.

| Row | Gap after the minus |
|---|---|
| 560 `so −1` | **17 px** |
| 498 `−1 = i · i`, a leading minus | 6 px |
| 565 `L(z) − ℓ`, a real subtraction | 16 px |

The card is the punchline of the false chain, and it reads as "so, minus, one". See `evidence/unary-minus-spacing.png`.

**Fix:** `r"\text{so } {-1} = 1\ ?"`. Rows 508 (`\sqrt{-1}`) and 578 (`e^{-x}`) show a minus opening a group sets tight, but this exact form has not been drawn. No test pins the string and no character changes. It moves the hash of `product_rule.rs`.

### F2, low and optional: math-mode colons float

TeX treats `:` as a relation. Six rows have it:

| Row | Source |
|---|---|
| 503 | `product_rule.rs:262` |
| 512 | `doing_nothing.rs:293` |
| 513 | `doing_nothing.rs:298` |
| 532 | `undoing_growth.rs:654` |
| 557 | `branch_cut.rs:309` |
| 586 | `product_rule.rs:129` |

Measured on 512, 513, 557 and 586, the colon sits 17–23 px from the glyph before it. The new labels use `\text{: }` and get 4 px (row 559). I judged 503 and 532 by eye.

**Fix, only if these lines are being touched anyway:** replace `:` with `\text{:}`, and double the braces in the two `format!` strings (503, 586).

### F3, low, please look: live negative numbers use a hairline sign

The engine's `DecimalNumber` replaces `-` with an en dash (`fmn-library/src/numbers.rs:60-61`), while fixed labels draw Noto's `−`. See `evidence/minus-en-dash-vs-noto.png`, rows 472 and 479. Thickness is 0.034 em against 0.071 em.

| Readout | Size | En dash, 1080p / 4K | A fixed minus would be |
|---|---|---|---|
| Chapter 07 angle, `branch_cut.rs:543` | 26 | 1.19 / 2.39 px | 2.49 / 4.98 px |
| Chapter 12 `laps so far`, `what_it_forgets.rs:1073` | 20 | 0.92 / 1.84 px | 1.92 / 3.83 px |

The sign carries meaning in both places: the angle flips sign in 07, and the spin runs the other way in 12.

**Action:** check that the sign reads at 1080p in your frames. If it does not, a larger readout is the only film-local option, since the glyph belongs to the engine.

### F4, low, recommend keeping: `ℓ` is heavier than the letters around it

It is the only letter in the film's TeX from the symbol face. Its median stroke is 5 px on the sheet, against 2–4 px for `z`, `(` and `L` in the same row. It is legible and clearly an ℓ.

**Recommendation:** keep it. CM Italic `l` would match the weight (`evidence/ell-faces.png`), but it has two costs:
- It touches 5 lines: `lifting_loops.rs:79`, `:219`, `:634`, `:886` and `:887`.
- A plain `l` colour key would also hit `whole`, because `find_occurrences` is a substring match.

The narration never says the letter, so no take depends on it either way.

### F5, low, no action: known fallback classes

Same classes as my complex review (F2, F4, F5). I found no ruling on them for this film.
- **IBM Plex Sans:** `½` on 19 rows, `²` on row 287, `₂` on supplement row 66 only.
- **Noto Sans Math:** operators heavier than CM, in Text and in TeX as listed above.
- Between them, these two faces supply at least one glyph on 99 chapter Text rows. `font_faces.out` lists them.
- **Open `θ`:** CM draws U+03B8 in the open form. Row 377 (chapter 07) is the only chapter use, and the film has no math-mode `θ`, so nothing here contradicts it.

### G1, coverage gap: the thumbnail is not probed

`thumbnail.rs:36-41` draws `WHERE DO`, `THE LAPS GO?` and `RULERS, TURNS, FLOWS` in bold through `text_styled`, plus `\log z`. It is not a chapter, so the probe never builds it. None of the four is on a page, and the bold face is on no sheet. No chapter uses `text_styled`, so the film itself has no bold or italic gap.

**Action:** read the real thumbnail at delivery.

## What the pages do not prove

- **Sheets are not film frames.** Each string is drawn alone, in near-white and the regular face, at size 36 (Tex) or 34 (Text). Text rows break at 64 characters, so mid-word breaks are the probe's.
- **Colour.** `\ell` and `L(z)` colouring (`lifting_loops.rs:219`) is not on a sheet. From source, each key is a plain substring with 2 hits. Not seen in pixels.
- **Neighbours.** Extents above are x ranges only, and I did not check labels against each other. One to look at: the tall axis's foot ends at y −2.85 (`lifting_loops.rs:242`) and is on screen again from 06_6 (`:524-527`). DIFFER is centred at y −3.0, which by my scaling puts the top of `L` and its `(` about 0.03–0.05 units below it, or 4–7 px at 1080p (`checks/gaps.out`). The test at `:822` checks the theorem against that axis, not DIFFER. Please look at chapter 06 after 06_20.
- **Fit of `(ℓ)` beside `ours`.** The offset is `ours_right + 0.12` at the same y as the word. I have no picture of it.
- **Reading time.** Not tested, including Ruby's 11_8 budget of 6.20 s for the two red lines.
- **Live numbers as drawn.** Sheet rows draw `num_string` through the Text path, not `DecimalNumber`'s own build.
- **Build claims.** Your 215 tests, strict Clippy and 102 oracle facts are your result. I ran no Cargo, tests or Clippy.
- **Audio and motion.** No listening, fine sync or continuous AV. No narrated picture was read.

## Authorship: not independent

- I reviewed and repaired logarithm chapters 01–04 (Mails 1448, 1502), so rows from those chapters include my own work.
- The `singular S` and `e^{tp}` labels came from the label patch Ruby attributes to BronzeJaguar under shared memory lineage. My memory carries that lineage, so treat those two rows as not independently reviewed.
- RubyRaven's chapter 04–06 repairs (Mail 1992) include DIFFER and `(ℓ)`, and name `y = x` and `the line goes on`.
- The checks are adapted from my complex glyph review, so they share its blind spots. Nobody has cross-checked my reading of these pages.

## Reproduce

From the repo root. These read the job and write only in this folder:

```
python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-glyph-review01/checks/run_checks.py
python3 -B docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage06-glyph-review01/checks/viewing_proof.py SESSION_ID
```

`viewing_proof.py` needs this session's transcript, so its saved `.out` is the record. The tools read `target/debug/logarithm_mysteries`, so a rebuilt binary fails the hash checks.

| Path | Holds |
|---|---|
| `coverage/page-evidence.tsv` | Full hash and note per page |
| `coverage/row-map.tsv` | Every ink row, its box and its registry row |
| `checks/*.out` | Saved outputs |
| `evidence/` | Enlargements, for looking only |
| `SHA256SUMS` | The bundle |

## Every page

Hashes here are the first 12 characters; full ones are in `coverage/page-evidence.tsv`. Ink rows exceed registry rows where a long Text row wraps. `F` numbers refer to the findings above. Unmarked pages match their rows with nothing to report.

| Page | sha256 | Rows | Ink rows | What I read |
|---|---|---|---|---|
| 0 | `97ffb986a0da` | 1–13 | 13 | Numbers and short labels. r4 `→` is the heavier Noto arrow. |
| 1 | `7ce91ca4412f` | 14–26 | 13 | Numbers and short labels. Clean. |
| 2 | `ea876db60c9e` | 27–39 | 13 | Numbers and short labels. Clean. |
| 3 | `e360862aa88a` | 40–52 | 13 | Numbers and short labels. Clean. |
| 4 | `09f9a306226e` | 53–65 | 13 | Numbers and short labels. Clean. |
| 5 | `22e4e2204acc` | 66–77 | 13 | r66 is the supplement, the only row not captured from a chapter: `θ` in its open form, Noto `− × ≈ ≠ ≤ → ↔ ∞`, Plex `½ ₂`. |
| 6 | `ac70b4253bf8` | 78–90 | 13 | Short labels. Clean. |
| 7 | `dc9e950568bf` | 91–103 | 13 | Short labels. Clean. |
| 8 | `ce162b1b0696` | 104–116 | 13 | Short labels. `×` heavier than its digits (F5). |
| 9 | `5f7eb82529c4` | 117–129 | 13 | `2×2`, `2π`, `2πi`: tight, legible. |
| 10 | `e0536cb23dcf` | 130–142 | 13 | Short labels. Clean. |
| 11 | `49dc8dccdff3` | 143–155 | 13 | Short labels. Clean. |
| 12 | `000d20140c39` | 156–167 | 12 | r160–r162 `(−I) × diag(2, ½)`, `diag(−2, −½)`: Noto `−`, Plex `½` (F5). |
| 13 | `17769435e307` | 168–179 | 12 | Short labels. Clean. |
| 14 | `d00f2acca33e` | 180–191 | 12 | `Log(−i) = −πi/2`. Clean. |
| 15 | `cfaafd5ac48a` | 192–203 | 12 | Short labels. Clean. |
| 16 | `29a6629cff8d` | 204–215 | 12 | Short labels. Clean. |
| 17 | `9c7ad897b733` | 216–227 | 12 | `−cI`, `near π; … −π`. Clean. |
| 18 | `a716df66c180` | 228–238 | 12 | r235 breaks mid-word: probe's 64-character chunking, not the film. |
| 19 | `a82591f2fa19` | 239–250 | 12 | `π i → −πi, gap 0 → −1`, `|z| = r`. Clean. |
| 20 | `cabfe5b2d5e8` | 251–262 | 12 | r254, r258, r259 Plex `½` (F5); r261 `e ≈ 2.718`; `det = (−2) × (−½) = +1`. |
| 21 | `82b8d0b5acce` | 263–274 | 12 | `(−1) − (−1) = 0`, `±1.5 ×`. Clean. |
| 22 | `6616b6c1b8af` | 275–286 | 12 | `Log(−1) = iπ`. Clean. |
| 23 | `008d3b020d84` | 287–298 | 12 | r287 `for z²:` Plex `²` heavier than the serif beside it (F5); `height −π`. |
| 24 | `47f6509b8bb9` | 299–310 | 12 | r302 `i × i = −1: two quarter turns`; r307 `it breaks at −1`. Clean. |
| 25 | `e35c374ca35c` | 311–322 | 12 | r313, r315 `→`; r317 `keep −π < turning < π`; r322 `½, 1½, 2½, …` (F5). |
| 26 | `797d738a5b44` | 323–334 | 12 | r323 `→`; r325 ends in `×`; r328 `ln 2 ≈ 0.693`; r329 `×e`. Clean. |
| 27 | `cf1a60c01fdf` | 335–346 | 12 | r335 `mirror: −1`; r346 `×10`. Clean. |
| 28 | `fa5ff77678dc` | 347–358 | 12 | r350, r353 `×10`; r357 `×4`; r355 `size-2`. Clean. |
| 29 | `d1cde5a6c337` | 359–367 | 12 | r359 `2πi`; r364–r367 are 64-character chunks; r365 en dash `1146–1151`; r367 `(−∞, 0]`. |
| 30 | `e8fb94599733` | 367–372 | 11 | r370 `∫` is the Noto integral; r372 `⇒`, `(−∞, 0]`; r367 `≤`. Chunk breaks are the probe's. |
| 31 | `010dec75c535` | 372–382 | 12 | r372 tail `)`; r373 `±½, ±1½, …`, `§4`; r377 `θ` open form, twice (F5); r382 `≈`, `÷`. |
| 32 | `b8ed44ec5b60` | 383–393 | 12 | r393 `−I`; r389 chunk break is the probe's. Clean. |
| 33 | `37ad0e9d0c0e` | 394–405 | 12 | r395–r399 `½`, `1½`, `2½` (F5); r400 `0.28π`. |
| 34 | `f1c34fb0b9be` | 406–417 | 12 | r410 `½(I + mirror)` (F5); r406, r407 live-style numbers. Clean. |
| 35 | `138fc6c8e7e1` | 418–429 | 12 | **r420 `the line goes on`: new, correct.** r428 `±π`. |
| 36 | `6ae37e14f942` | 430–440 | 12 | r436 `↔`; r438 chunk break is the probe's; r435 `1.380 laps`. Clean. |
| 37 | `dac44dfef993` | 441–453 | 13 | r443 `(−½) − (−2)`; r452 `→`. Clean. |
| 38 | `d33ab9898f6f` | 454–466 | 13 | r455 `½` alone, in Plex; r456 `× ?`; r466 `×½` (F5). |
| 39 | `6f461f5b9644` | 467–479 | 13 | r472–r476 live values with thin en dash, against r479 heavy `−` (F3); r467 `√`; r478 `⇒`; r477 `…` low. |
| 40 | `afc64d8226e3` | 480–492 | 13 | r480–r492 fixed-label `−`; r491 `−b`, from `"-b"` through `shown()`; r492 `−½`. |
| 41 | `1e2863218975` | 493–504 | 12 | **r497 `(ℓ)`: new, correct, Noto (F4).** r498 tight unary control; r503 floating colon (F2); r502 `P^{-1}` clears at every ink level. |
| 42 | `9d71e62b3a24` | 505–515 | 11 | r507–r509 root bars clear by eye; r511 `e^X` clears at every ink level, r510 by eye; r512, r513 floating colon (F2); r514 `⟷`, `\left(`. |
| 43 | `51004e3a889c` | 516–525 | 10 | r516 `⟺`; r517 `⇒`; r518, r519 `\frac`; r520 `↔`; r522–r524 `≈`. Clean. |
| 44 | `e1b3508b1453` | 526–534 | 9 | r527 `e^{\ln y}`; r528 `diag`; r530, r531, r533 subscripts on `log`; r532 floating colon (F2). |
| 45 | `7300091e3e5f` | 535–544 | 10 | r535 `\tfrac`; r538 `Arg(−1)`; r542 `≠`; r543 `⟺`, `≤`; r540, r544 clean. |
| 46 | `41035b34d08f` | 545–555 | 11 | r546–r549 root bars and exponents; r547, r549 `\frac` in an exponent; r550 `×`; r551, r555 `\tfrac`. |
| 47 | `f88a88557bea` | 556–565 | 10 | **r559 singular S and r565 DIFFER: new, correct.** r560 `so − 1` spacing (F1); r557 floating colon (F2); r558, r563 clean. |
| 48 | `ad9cc0bdd560` | 566–576 | 11 | **r575 `e^{tp}` line: new, correct.** r569 `e^{-\ln 2+i\pi}`; r570 `≠`; r574 `e^{tX}`. |
| 49 | `a7487832497d` | 577–586 | 10 | **r584 `y = x`: new, correct.** r578 `e^{-x}` clears; r579 `\frac`; r586 floating colon (F2). |
| 50 | `3e55c5881088` | 587–588 | 2 | r587, r588. Last page; the rest of the sheet is empty, as expected. |
