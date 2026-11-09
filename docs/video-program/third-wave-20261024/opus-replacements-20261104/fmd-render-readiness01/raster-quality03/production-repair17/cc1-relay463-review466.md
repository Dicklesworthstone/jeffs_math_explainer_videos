# Review 466: relay 463 (CC1)

CC1 (BronzeJaguar), 2026-11-09 UTC. An independent, targeted review of CC3's relay 463. It covers three things and is not a repeat of the generic audit:
- the fix of 457 M1 and L2;
- the 448 SSD routing;
- root's question about condition g.

All evidence here comes from source and fixtures. No host was contacted and nothing was imported. CSS442 is still live, and no import has run. All final AV acceptance stays open, and Frederick remains the narration.

## Files

| File | sha256 |
|---|---|
| `cc3-css-gallery-relay463.py` (reviewed, frozen) | `d1822a5157895ae19c7128a1de374dbafa7de204aeba8c99b3895fabe4d5b892` |
| `cc3-css-gallery-relay463-tests.py` (CC3, 96/96) | `9ba910a37f3dbfa48ef5e912610e259cd8214cb3dd98c329930fab5b7a6fd2ac` |
| `cc3-css-gallery-relay463-receipt.md` | `e00f8ae50739a65f88969b06fd97934c8b02a00c08be1b363635acbb7c49efdb` |
| `csd-gallery-import448.py` (pinned) | `b6653757c5c114029a94832a0dd4cb9597c42808614fbb959ef6448a5d994f6e` |
| `cc1-relay463-review466-fixtures/MANIFEST466.json` | `088bae1f10962bbc3b3918a3999c4f8077f51a0333ba01b4a05504ca23008e98` |

## Fixtures and what they prove

- **463 runs unchanged, in-process.** Only `HERE`, `load_helper` and `subprocess` are rebound.
- **448 runs as a twin:** its pinned bytes with only the four root literals rewritten.
- **The CSD receive end is scripted**, except in F1/F1r and F5r, where 448's real `receive` mode runs.

The fixture README states every limit. A pass here shows only how the code behaves in that world. It is not evidence about an actual import.

## Verdict

**No High or Medium findings.** The 457 M1 fix is in place: post-import a–h runs in `run` and in `verify`. Each condition fails on its own targeted break (F3). The 448 SSD routing is correct.

The lane, admission and attempt guards hold through every interruption tested. No path re-admits, retries, or sends before admission.

Two Lows have small, concrete changes. Neither blocks production use of 463 once the gallery and the lane are ready. L2's change matters to whoever recovers after a stream loss.

## L1 (Low): g is count-only, and c/d are not bound to the pinned receive

`expected_inputs` (463:276) returns a cardinality. g (463:300) checks only:
- `rows == distinct == want`;
- `changed_count == 0`.

**What a fabricated completed state passes** (F1). On a state with a terminal receipt, an import-result and the files placed, `validate` passes 8/8 when one row is swapped for another valid file at the same count. I tested a shared row and the Q permit copy. A dropped row or an extra row fails g.

**Unreachable through a real 448 receive** (F1/F1r). 448 receive checks the inputs rows before it reads the stream header (448:836, 843, 845), and it checks the receipt command (448:828).

| Case run against 448 receive | Result |
|---|---|
| Shared row substituted, same count | refused |
| System row substituted, same count | refused |
| Q permit copy substituted, same count | refused |
| Helper substituted, same count | refused |
| Guard substituted, same count | refused |
| Manifest substituted, same count | refused |
| Exact rows | binding passes |

None of the refused cases read or staged a byte, and none wrote an import-result. A completed job made by 448 receive therefore holds the exact set. f then ties the inputs file to the one admitted.

**The residual gap: `validate` does not prove that 448 receive made the job.**
- c accepts any terminal receipt. It never compares the receipt's `command` with `receive_command(Q/448, chapter, attempt, manifest)`.
- The `helper_sha256` that 448 check reports is ignored.

F5 shows the effect. With the receipt's command rewritten to name another helper path, or attempt 7, `verify` still exits 0 with 8/8. F5r shows the other side: 448 receive refuses such a job ("job admitted for another command"). So g's word "exactly" holds only in combination with conditions that `validate` does not itself check.

**Change.** Either of these closes the gap; the first is preferred.
1. Make g compare the exact {path: sha} map, the way `check_prepared` does (463:327):
   - the shared and system rows at their manifest hashes;
   - `Q/448` at `HELPER_SHA256`;
   - the manifest at `manifest_sha`;
   - the Q permit copy at `permit_sha256`;
   - `GUARD` present (at any hash, or at the admitted one).

   Add to c two checks:
   - the receipt's `command` equals `receive_command(...)`;
   - 448 check's `helper_sha256 == HELPER_SHA256`.
2. Or reword g to "count, distinct and re-hash", and say that exactness rests on 448 receive's binding.

`--prepared` already does the exact compare: substituted prepared inputs are refused (F1, S11).

## L2 (Low, inherited from 394): after a stream loss, the stop says `status`, but recovery is `verify`

463:447 says "run status when both hosts answer". In S02, the stream is lost (receive 255) after the remote end completed. There:
- `status` exits 1, reporting "already imported by …-a1; nothing to do" and "attempt 1 is not above every existing attempt", and runs no a–h;
- `verify --attempt 1` exits 0, passing 8/8.

When the remote end failed instead (S03), `verify` exits 3 and names c, d and h.

**Change.** Make the text say "run verify --attempt N when both hosts answer". The validation-interrupted stop at 463:455 already names `verify`.

## Cosmetic

A stop message can carry a newline from ssh stderr. S05's record holds `…Operation timed out\n; nothing is used until …`. Joining the lines, or stripping the ssh text, would keep each stop on one line.

## Checked and holding

**a–h (F3).** Each break below was made on its own, on top of an import completed by a 463 run. `verify` names exactly the expected conditions.

| Break | Conditions that fail |
|---|---|
| Placed SSD file changed | a, e |
| RECOVERY alias removed | a, e |
| RECOVERY alias pointing elsewhere | a, e |
| import-result names another manifest | a, d, e |
| Inputs rewritten after admission | f |
| Shared row changed | a, e, g |
| Receipt exit 1 | c, h |
| rows350 names attempt 2 | h |
| `--attempt 2` (no such job) | all eight |

**Recovery after a transport interruption (F2).** After every case below, CSD jobs and Q stayed unchanged.

| Scenario | Exit | Then |
|---|---|---|
| S02: stream lost, remote completed | 2 | `verify` 0. A rerun at the same attempt and attempt 2 both exit 1: already imported, no second import. |
| S03: stream lost, remote failed | 2 | `verify` 3. A rerun exits 1, and `--prepared` exits 1. |
| S04: POST_PROBE timeout | 2 | `verify` 0. |
| S05: second check 255 | 2 | `verify` 0. |
| S06: admission poll timeout | 2 | No byte sent. `verify` 3; a rerun exits 1. |
| S07: guard refused admission | 3 | No byte sent. `verify` 3; a rerun exits 1. |

**Guards.**
- S08: lane busy exits 4, and nothing starts.
- S09: a stopped prepare without `--prepared` exits 1.
- S10: with `--prepared`, it exits 0.
- S11: substituted inputs with `--prepared` exit 1.
- S12: a missing 371 import of chapter 14 exits 1 (serial order).
- S13: a CSS probe 255 exits 2, with nothing on CSD.
- S01 happy path order: the lane is checked twice before receive, and the send starts only after the receipt shows admission.

**448 SSD routing (F4).** Each case below exits 1 with a specific refusal, and nothing is written on CSD:
- the CSS RECOVERY copy of 448 differs;
- the CSS RECOVERY manifest copy is missing;
- the 442 receipt is not terminal;
- the 442 job exists only under the CSS root;
- the CSD Q copy of 448 differs;
- the CSD RECOVERY alias is missing (448 check refuses before prepare);
- chapter 14 was imported only under the 448 prefix (the serial gate uses the 371 prefix).

**457 L1/L2.** These arrive through 394. CC3's section 0 checks that 463 is 394 plus the listed delta. I did not re-test the `--prepared` TypeError separately.

## Limits

- These are fixtures, not an actual import. The receive end is scripted outside F1/F1r and F5r.
- The twin rewrites only the four roots.
- Runs were on macOS with Python 3.15.0. Nothing was run on CSD or CSS, and no remote call was made.
- 463 and every root file are unchanged.
- This review is not acceptance of the gallery import or of any master, and it makes no audio-review or picture claim.
