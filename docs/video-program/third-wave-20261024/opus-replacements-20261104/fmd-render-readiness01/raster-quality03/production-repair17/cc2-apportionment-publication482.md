# Apportionment publication copy 482 (CC2)

CC2, 2026-11-09, about 05:30 UTC, for root. Q is this folder. This is public copy only: a title, a description, tags and provisional chapter times.

**Nothing was uploaded or changed elsewhere.**
- No browser was opened.
- No catalog, Beads, TODO, storyboard, description.md or other shared file was edited.
- Nothing was launched.

**What this is not.**
- I checked words against the script and docs. I saw no picture and played no audio, and no master exists. Nothing here accepts the film or authorizes publication.
- I wrote this copy, so it is not an independent review.

## Status

| Item | State |
|---|---|
| Chapter times | **PROVISIONAL.** They are computed from timeline444's 15 chapter frame counts: 100,886 frames, 28:01.433 at 60 fps. Before upload, root re-derives every start from the final master and keeps these times only if all 15 starts and counts match exactly. |
| Narration | The 93 retained ElevenLabs Frederick Surrey takes. `decode-receipt433.json` has 93 records, `source` "ElevenLabs Frederick Surrey retained UI WAVs". `narration437/report.json` has 93 lines bound to script `b9344e11…`. No take was replaced or reworded. |
| Listening review | Open. `narration437-ready438.json` has `listening_accepted: false`. |
| Fine sync | Open. `fine_sync_accepted: false`. |
| Pictures, full independent AV review, caption playback | Open |
| Links | Open. The three URLs were not opened in 482; see decision 3. |

## Files

| File | sha256 | Use |
|---|---|---|
| `cc2-apportionment-publication482-title.txt` | `9d477f8941990c4c13f9f39010ae4245af4949896e52cac30ddcd510b659f3af` | Title, 87 characters. No em dash, no en dash, no `<` or `>`. |
| `cc2-apportionment-publication482-description.txt` | `32549909166bd27c0e7bd8012eab6908f1a1f986a08eb8f6621b385e178545bf` | Description: 4,882 characters, 4,891 UTF-8 bytes; the prose is 3,503 characters in 9 paragraphs. No em dash, no `<` or `>`. Its only non-ASCII characters are ö, the en dash in "Huntington–Hill" and "4602–4606", and √. |
| `cc2-apportionment-publication482-tags.txt` | `59356d6510cbed091ba9198cde3af5fe77135bd5542efdb884f10ae01c25cd07` | 23 tags. Joined with commas they are 422 characters; by YouTube's count (commas, plus quotes around tags with spaces) 462 of 500. The longest is "congressional apportionment", at 27. |
| `cc2-apportionment-publication482-chapters.json` | `08f6bd51ebd497d3fc326d063a1bd1e7ac6e7a0f8e6dbb0494b4d55ba097ca1a` | 15 keys, titles, frame starts, counts, seconds and timestamps. `status` is `PROVISIONAL`. It uses the shape of the logarithm `chapter-times03.json`. |

## Inputs

| Input | sha256 |
|---|---|
| `explainer_corpus/apportionment/script.tsv` (93 cues) | `b9344e1123170e7e3898d8aea2123e0b659bcd571085008fc97894e7d69da624` |
| `explainer_corpus/apportionment/docs/description.md` | `16c44c2c890d019a5b53d574970ede3dceaf87a8ea4e94e13a789cb5660056a9` |
| `explainer_corpus/apportionment/docs/storyboard.md` | `564e86db6c12c75a651bfcb6912619e2b3219337442095b64b78f2b49e930176` |
| `src/chapters/one_more_seat.rs` (on-screen title, line 122) | `d7a8b5e66053ed0ac4e9fc2a3fcd8684a862a22016cdffe3206e873bd5bacc5a` |
| WORK `apportionment-frederick-timeline444/timeline444-summary.json` | `686bf77dd9e392de1ee563e7bd691ad9cb71e783c05b7da2346d1f366f726277` |
| WORK `apportionment-frederick-timeline444/resource-receipt.json` (exit 0, finished 04:55:22Z) | `567a9b6271cb55fb12940606939c6f15bab78856f40b07a84bc248b8e822d864` |
| generation89 `decode-receipt433.json` | `7883d88bad152ba3577d8607511acbf2bdfb4a9200e75e23764f1c8a8b0a0e94` |
| generation89 `narration437/report.json` | `ee3c98182bcc9dbafa8d16c08778ed9d54b0714dcdf9c5505b65c50321314bc7` |
| generation89 `narration437-ready438.json` | `cfabaab20295a9f0929925155b3f5616788dc2cb755fa45bf0894f2cd6c5c958` |

## Provisional chapter times

| Chapter | Start frame | Frames | Start (s) | Timestamp |
|---|---|---|---|---|
| 01_one_more_seat | 0 | 4806 | 0.000 | 0:00 |
| 02_shares | 4806 | 6647 | 80.100 | 1:20 |
| 03_largest_tails | 11453 | 5527 | 190.883 | 3:10 |
| 04_sixteenth | 16980 | 4238 | 283.000 | 4:43 |
| 05_dials | 21218 | 6316 | 353.633 | 5:53 |
| 06_not_every_time | 27534 | 4947 | 458.900 | 7:38 |
| 07_yardstick | 32481 | 5049 | 541.350 | 9:01 |
| 08_signposts | 37530 | 6448 | 625.500 | 10:25 |
| 09_cost | 43978 | 4907 | 732.967 | 12:12 |
| 10_rounding_rules | 48885 | 8471 | 814.750 | 13:34 |
| 11_webster_quota | 57356 | 6480 | 955.933 | 15:55 |
| 12_both_promises | 63836 | 13460 | 1063.933 | 17:43 |
| 13_population | 77296 | 7459 | 1288.267 | 21:28 |
| 14_no_rule | 84755 | 10390 | 1412.583 | 23:32 |
| 15_recap | 95145 | 5741 | 1585.750 | 26:25 |
| End | 100886 | | 1681.433 | 28:01.433 |

**How the times were computed.**
- Each start is the sum of the earlier chapters' frame counts, divided by 60.
- The timestamp rounds that start down to the whole second, as the logarithm draft does (for example 352.95 s became 5:52). Rounding to the nearest would move 10 of these 15 by one second.
- This assumes the master is the 15 chapters in order, with nothing before, between or after them.

**What was checked.**
- Each count equals timeline444's summary and the `frames` line of that chapter's `timeline.tsv`.
- The titles equal the description.md chapter list, and the storyboard headings apart from case.
- They meet YouTube's chapter rules: the first is 0:00, they ascend, and the shortest chapter (04) is 70.6 s.
- Each chapter's `timeline.tsv` has exactly one predict mark. That is the basis for the line "Every chapter pauses on a question before answering it."

## The impossibility theorem, as the description states it

| The description says | Basis |
|---|---|
| Balinski and Young proved in 1982 that no rule keeps both promises on every map, with the extra assumption that a town with strictly more people than another never gets fewer seats | Cue 14_1 (the assumption in its words). description.md "The impossibility (chapter 14)", with the GPP §3.1 quote on the order-preserving property. The 1982 book is cited through GPP and was not read. |
| Gölz, Peters and Procaccia proved the same without that assumption, for any rule that does not draw lots | 14_1 ("any rule that always gives the same answer to the same question"). description.md: GPP Theorem 1, any deterministic method. |
| Their proof uses a city of 824 and four villages of 44, at 10 seats; the film follows it | 14_2 to 14_6. description.md: Table 1, maps A to C. |
| With three towns Webster keeps both promises | 14_7, 11_2 and 13_5. description.md: Balinski and Young 1982 Proposition 6.2, as cited by GPP. |
| With four, without the assumption, the question was open as of the paper | 14_7 and description.md |
| A lottery can keep quota in every draw, never take a seat back and give each town exactly its share on average. No lottery keeps the quota promise and the population promise together. | 14_8 and description.md |

**Left out on purpose.**
- How many towns the 1982 version needs. Neither the script nor the docs say.
- Cue 14_7's "the same trick works with more". Only the script says it; description.md does not. The description's "on every map" needs only the five-town map.
- Any uniqueness claim for the quota method. The film makes none, and description.md says the film does not prove it.

**Not reopened in 482.** The exact definitions GPP use for population monotonicity of a lottery. The lottery line is carried from 14_8 and description.md.

## Where each paragraph comes from

| Paragraph | Cues | description.md section |
|---|---|---|
| 1. Fifteen seats give 2, 3, 10; sixteen give 1, 4, 11; Alabama paradox | 01_1 to 01_5, 02_2, 03_4, 04_1, 04_4 | Shares; Hamilton's rule |
| 2. Seaton in 1880; the quote; alternative calculations | 06_4 to 06_6 | Seaton's letter |
| 3. Tails move by 4/39, 9/39, 26/39; 5 of 99 additions, all Ash | 05_2, 05_5, 06_3 | Hamilton's rule |
| 4. Divisor rules; Hill's thresholds; one ranked list; ties; quota breaks; New York 38.593 for 40; Hill 1, 1, 1 | 07_2 to 07_4, 08_2, 08_5, 08_6, 09_2, 09_4, 10_2 to 10_7 | Divisor rules; Hill's thresholds; Sources (Young, Table 4) |
| 5. Webster keeps quota with three towns; four towns get 1, 2, 3, 8 | 11_2 to 11_4 | Webster keeps quota with three towns |
| 6. The quota method; skips past the share rounded up; the tile proof | 12_2 to 12_9 | The quota method; Why the quota method keeps quota |
| 7. The population promise; 3,900, 9,500, 26,100; Hamilton moves a seat from Cedar to Ash; divisor rules don't | 13_1 to 13_5 | Divisor rules and growth |
| 8. The theorem, with its qualifications | 14_1 to 14_8 | The impossibility (chapter 14) |
| 9. Every chapter pauses on a question | One predict mark per chapter | (timeline444) |

**Mechanical checks.**
- Every number in the prose occurs in description.md.
- Every person and place named in it occurs in the script or description.md.
- The Seaton quote is a verbatim substring of description.md and cue 06_5.

## Decisions for root

1. **Title.**
   - The on-screen title "Adding a Seat" (`one_more_seat.rs:122`, description.md heading, storyboard.md line 1) is kept exactly.
   - The subtitle comes from 01_5 (the Alabama paradox) and 01_4 ("The council got bigger, and the smallest town came out with less").
   - At 87 characters, search results may cut it after about 70; the part before the cut is "Adding a Seat: The Alabama Paradox and Why a Bigger".
   - Alternatives:
     - "Adding a Seat: How One More Seat Can Cost a Town a Seat", the wording of `src/main.rs:1`;
     - the catalog working title "How adding a seat can make a state lose one" (catalog.json, unchanged).
2. **Length.**
   - YouTube's API counts the 5,000 limit in bytes. The description is at 4,891, so about 100 bytes are left.
   - Draft 1, at 5,897 characters, was cut to fit. The cuts were:
     - the "method of largest remainders" aside (the tags keep the name);
     - Seaton's title, chief clerk;
     - a lead-in sentence for the mechanism;
     - Adams's and Webster's first names;
     - Hill's reason for 1, 1, 1;
     - the quota method's 1, 4, 10 at fifteen seats;
     - the "because" clause on divisor rules and growth;
     - an opening line and the definition of "deterministic" in the theorem paragraph;
     - the line on which examples use three towns;
     - the puzzle mention;
     - PNAS's full name;
     - the arXiv identifier as text (the URL keeps it).
   - None of these changes what the description claims.
   - "Signpost" is the film's own word and is never defined in the description, so it was replaced by "one ranked list".
3. **Links.**
   - `https://doi.org/10.1073/pnas.71.11.4602` and `https://arxiv.org/abs/2202.11061v2` are the standard resolver forms of the identifiers in description.md.
   - The Census Bureau URL is copied from description.md.
   - I opened none of the three in 482. Root or the reviewer should open each before upload.
4. **Credits that nobody read.**
   - Fair Representation (1982) is labelled in the description as cited through Gölz, Peters and Procaccia.
   - The 1975 Monthly paper is kept as further reading because description.md lists it. description.md also says that paper was not read, so root may drop it.
5. **Tags.**
   - Three tags use names that the script, description.md and the storyboard never use:
     - "population monotonicity", for what the film calls the population promise;
     - "population paradox", which only `src/main.rs` uses;
     - "Balinski Young theorem", a search phrase.
   - The other tags are among:
     - the film's own terms ("house monotonicity" is the storyboard's, lines 38–40);
     - the titles of its sources;
     - the two generic tags of the earlier drafts, "visual mathematics" and "math explainer".
   - The other names of Jefferson's and Webster's rules (D'Hondt, Sainte-Laguë) are left out because neither the script nor the docs use them. Both would fit within 500.
6. **Stale narration note, not mine to change.** storyboard.md line 18 still says the narration is ElevenLabs Oliver. The unit ids in `narration437/report.json` keep the legacy `oliver_` prefix. The takes' own receipt says Frederick Surrey, and the description credits Frederick Surrey.

## Checks run

- **The check.** A short pure-Python check, kept in scratch outside the repo, was run with `python3 -I -B`.
  - Runs 02 and later passed every case: 36 of 36, then 37 of 37 on the final copy.
  - The added case confirms that every capitalised word in the prose is a listed name, and that each name occurs in the script or description.md.
- **What it covered:**
  - the source pins;
  - title length and characters;
  - description characters and bytes, em dash, `<` and `>`, and NFC;
  - the JSON against the summary, each `timeline.tsv`, description.md and the storyboard;
  - the chapter lines against the JSON, and YouTube's chapter rules;
  - prose numbers, names and the quote against the sources;
  - every description.md source credit kept, and the credit line;
  - tag count, duplicates and YouTube length;
  - a list of stock AI words;
  - no listening or picture claim words;
  - repo hygiene on every 482 file.
- **Run 01 failures.** Run 01 failed 4 checks:
  - Two were length: 5,897 characters and 5,906 bytes.
  - Two were faults in the check: a number pattern that caught "8," with its comma, and the quote test missing the quote because description.md wraps it across lines.
  - Both faults were fixed in the check and draft 1 was cut. Run 01's output and draft 1 are kept in scratch.
- **Manual read.** The style pass was manual and line by line, following the de-slopify rules. The one "X, not Y" sentence is kept on purpose, because it corrects the usual misreading of the 1880 story.

## Not done

- No upload, browser, catalog, Beads, TODO or memory edit.
- No source page or link opened.
- No picture or audio examined.
- No timestamp sought in any movie.
