# Independent read: extractor 60addbf3 in the staged final route (CC1 BronzeJaguar, 2026-11-11)

**Scope.** I looked for actual integration bugs only. I launched nothing and touched nothing on CSD. I read local root copies, CC0 scratch and the local reference tables. No picture viewing, listening or AV claim.

**Files read.** All are from root's local `final-review995/` copies, which equal the staged hashes:

| File | sha256 (prefix) |
|---|---|
| extractor | 60addbf3 |
| after-final02 | 332ee000 |
| runner | 7cafacfa |
| launcher | 3c292758 |
| serial controller | 15653bf0 |
| combined01 preview comparison | `combined-preview-terminal-root01/comparison-combined01.json` |
| 09 preview extraction record | `boundary09-reviews-root-preserved01/pictures.json` |
| 01/08 preview extraction records | `combined-pictures-root-review01/<ch>/pictures.json` |
| 969 extraction receipt | `completed-pictures969-resource-receipt.json` |
| 968 final receipt | `completed-native968-resource-receipt.json` |

## Result: no integration bug found. Nothing to fix before extraction.

What I checked:

1. **Call contract.** `retained()` calls `run.review_checks(chapter, review, root, independent, author, preview_record, sha(record))`. That is the same argument order and the same objects that the runner's own `proofs()` passes at line 263. `author` is None exactly when `REVIEW_ROUTES` has no entry, which matches the runner's assertion.

2. **Post-decode table asserts on actual bytes** (`extractor60-tables01.py` / `.log`). CC0's probe did not exercise these.
   - **Method.** I rebuilt each chapter's actual combined01 `timeline.tsv` from local reference tables, by moving the line-row narration folder to `csd-native-combined-delivery01/narration`. Each rebuild was then verified against the comparison's `actual_sha256` (01 129947a3, 08 4b40ecc6, 09 9bc2e615), using 2–3 independent references per chapter.
   - **`normalized()`** on those bytes gives the runner spec for every chapter: 01 150147fe, 08 43c8bab2, 09 5a91b8f9.
   - **The 09 replace** gives `EXPECTED09` timeline 5a91b8f9.
   - **09 `cues.tsv`.** The combined01 preview record's raw sha is already eed5a50a, which equals `EXPECTED09`, so the replace is a no-op. The final table must be byte-equal to the preview's.

3. **CAP 4000.** The preview records hold 120, 484 and 1506 frames for 01, 08 and 09, with 1, 3 and 2 lint anchors.
   - Each anchor adds at most 11 + ⌈remaining/300⌉ ≤ 66 frames, so 09 can reach at most about 1638.
   - The cap is exceeded only if the final layout has more than about 37 anchors.

4. **Growth.** 969 extracted 814 4K pictures plus 102 sheets for 0.27 GB of work growth. 09 at about 1638 pictures comes to roughly 0.55 GB, under the 4 GiB phase cap.

5. **The 30 h waits.** The 968 final of 09 took 4 h 17 min (16504 frames). 01 + 08 + 09 come to about 10 h at that rate, well inside after-final02's 108000 s window.

6. **Process detection.**
   - The serial controller's `alive()` matches only LAUNCH or RUNNER path arguments, or an argument starting with `csd-native-repair-final01-`.
   - The waiting after-final02 controllers and the extraction guard carry none of those: the job is `csd-native-repair-pictures01-*`, and `--inputs` is an absolute path.
   - The launcher's own alive check matches only the final binary at 3840x2160, or RUNNER.
   - So an extraction that overlaps a later final cannot make the chain refuse.

7. **Lineage.** `lineage()` requires the final receipt command `[sys.executable, -I, -B, RUNNER, --chapter, ch]`. The launcher passes exactly its own `sys.executable` and `str(RUNNER)`.
   - The serial controller requires `/usr/bin/python3`. after-final02 must also run as `/usr/bin/python3`, which root says it does.
   - 01's manual launch must also have used `/usr/bin/python3`. **Root, please confirm this from 01's receipt command once it exists.**

**Not checked:** the actual final layout anchors, the extraction itself, CSD free space and the work cap at extraction time.
