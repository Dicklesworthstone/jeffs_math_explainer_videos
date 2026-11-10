# Handoff 727: apportionment caption 168 display repair

Author: CC3 (WhiteSwan). Status: staged for independent review. After that review, root applies it to the future master that follows the chapter 06 repair. Nothing has been applied. No SSH, build, render, synthesis, Beads edit or commit was done. Verifier 701 and master 700 are unchanged, and so is the kit's caption writer.

## The change

Root reported that verifier 701 refused caption 168. That cue belongs to 07_4 (`script.tsv` line 40). There are no master 700 captions on this machine, so I have not read the failing file myself. The values below are root's report, written in the kit's caption format.

Text, two lines of 41 and 42 characters, 84 characters joined:

```
Choose the number of people per seat that
makes the whole parts add up to the house.
```

| | Before | After |
|---|---|---|
| Start | 612110 ms, `00:10:12,110` | unchanged |
| End | 616290 ms, `00:10:16,290` | 616310 ms, `00:10:16,310` |
| Shown for | 4180 ms | 4200 ms |
| Characters per second | 20.0957 | 20 exactly |
| Gap to cue 169 at 616900 ms | 610 ms | 590 ms |

Verifier 701 flags a cue when `len(text.replace('\n', ' '))*1000/(end_ms-start_ms) > 20`, using integer milliseconds. 84000/4200 is exactly 20.0, so the cue passes. 616310 is the smallest end that passes, since 616309 gives 20.0048. The threshold is not changed, and the cue sits exactly on it, with no margin.

In each file the only change is two digits of that one end stamp (`290` becomes `310`). The words, the line breaks, every start and every other end keep their exact bytes. The VTT gets the same change with `.` in place of `,`.

**This is a display dwell adjustment, not proof of speech timing.** The kit sets each caption's span from the narration report and the audio (`video_captions`, `crates/explainer_kit/src/captions.rs:541`). The repair keeps the caption on screen 20 ms past the generated end. It does not measure when the words are spoken, and it does not show that the end of the speech falls inside the new span.

## Files

| File | sha256 |
|---|---|
| `repair_caption168.py` | `d94aab217a6ee6639328414b774a7a374fb610c0716f3213222ca421a3762b9f` |
| `test_repair_caption168.py` | `500b9f97c4bbde657054a831304a8ae4174867611c9df2e3607f893bd474d20c` |

Read but not changed: `apportionment-csd-verify701.py` `02ef7a5f4aae336b4a3795b066352a85c9a536d953c8920fab4755a2f174417f` and `apportionment-csd-master700.py` `e589f7f3302f9d97a9ad5b6b6f244ea3fccac71ffb91f5364f83fe555575743d`.

## Running it

```
python3 -I -B repair_caption168.py \
  --srt MASTER_DIR/apportionment.en.srt --vtt MASTER_DIR/apportionment.en.vtt \
  --srt-sha256 SRT_SHA256 --vtt-sha256 VTT_SHA256 --out NEW_DIR
```

The two hashes are the `srt_sha256` and `vtt_sha256` the future master records. `NEW_DIR` must not exist. The tool writes `apportionment.en.srt` and `apportionment.en.vtt` there with exclusive create and prints one JSON line with the input and output hashes, the before and after timings and rates, and the scope sentence above. The inputs are only read. Exit 1 means refused, with nothing written. Exit 2 means a missing argument. If a write fails after `NEW_DIR` is made, that directory stays; use a new name.

It refuses before writing anything when:
- either whole-file hash differs;
- cue 168 differs from the expected cue in any way (a word, the line break, the start, the end, including an already repaired end), or sits at another position;
- cue 169 does not start at 616900, so a timing shift anywhere before it is refused;
- the input has any fast cue other than 168;
- the SRT and the VTT do not parse to the same cues;
- the parse rules of verifier 701 fail anywhere in either file (index, stamp, 42 characters, two lines, overlap, past the film end of 100886 frames);
- the file has a byte order mark, a carriage return or invalid UTF-8, or cue 168 is not in the exact bytes the kit writes (for example `,` in the VTT, a VTT index line, a missing SRT index);
- the edit would change any byte other than those two digits, or the output still has a fast cue or differs from the input beyond that end;
- the output path exists or its parent does not.

## Where it fits

Master 700 writes both files with `captions all` (line 429) and records their hashes in its summary (line 440). It does not mux the captions into the MP4, so the movie bytes are unaffected. Verifier 701 checks the files in the master directory against those hashes (line 203) and then parses the same files (lines 229-233). The repaired pair therefore has to be wired in by a successor step that root chooses. Either the successor master runs the tool after `captions all` and records the repaired hashes, or a successor verifier reads `NEW_DIR` and its hashes. Either way, the `> 20` limit and the parse rules stay as they are, and both the input and output hashes get recorded.

Regenerating the captions writes 616290 again, so every new caption output needs the tool run again with that output's hashes. If the chapter 06 repair moves cue 168 or cue 169 by even 1 ms, the tool refuses. That needs new values chosen from the new failure, not a bypass. Any later caption derivative, such as a positioned VTT, should start from the repaired pair.

## Checks run

- `python3 -I -B test_repair_caption168.py`: 62 tests, OK. The fixture is synthetic: 180 cues in the byte format of `to_srt`/`to_vtt` (`captions.rs:630-658`), with cue 168 and the start of cue 169 at the reported values.
- The tests load the real verifier 701 by path, with its sha256 pinned. The real `captions()` reads both fixture inputs and both outputs and returns the same cues as the tool's own parser. The verifier's own `fast = [...]` line is taken from its source and executed: on the input it returns cue 168 alone, with these exact start, end and text, and on the output it returns nothing. Both parsers reject the same nine malformed SRT files and a VTT with no header.
- The tests cover exact matching, no overlaps, no shortening (only cue 168 grows, by 20 ms), the limit (616309 still fails), each refusal above, and checks that stay live when the edit itself is faulty.
- 26 planted faults in scratch copies (for example 616309, 616311, limit 21, `>=`, each dropped check, overwrite in place, a changed verifier limit): all 26 were caught by the tests.

## Not established

The tool has never read the actual master 700 captions or the future master's captions, because neither is on this machine. Nothing here was rendered, listened to or watched. Nothing here judges whether 20 characters per second reads comfortably; it only meets the verifier's rule. Independent review is required before this tool is run.
