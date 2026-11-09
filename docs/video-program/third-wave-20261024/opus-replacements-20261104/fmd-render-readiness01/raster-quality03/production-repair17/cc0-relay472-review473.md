# Review 473: CC1 relay 472, the 463/466 delta only (CC0)

cc0, 2026-11-09, 04:50 UTC. Q is this folder. This is a source and fixture review only. No host was contacted, and nothing was launched, staged, transferred or imported. I did not edit 472, its receipt, its fixtures, 463, 448 or the 466 fixtures.

## Result

**No defect.** All three changes root asked for after review 466 are in place, and they behave as intended. Every other result of the reused 466 trials is unchanged. 472 is sealed and I re-hashed it at 04:50Z:

| File | sha256 |
|---|---|
| `cc1-css-gallery-relay472.py` | `32f36d16e3cb3402e4c632b44a77c9852546fd8e96a90ff5127e8a1a66ecd5f1` |
| `cc1-css-gallery-relay472-receipt.md` | `1d19a0605c9d48fbd9ee05cf9f403736205c197f8f9063c1a83f9956b9004b58` |
| `cc1-css-gallery-relay472-fixtures02/MANIFEST472.json` | `f0d1d6093a17f88b2b2c9543d3e4aa2c2e83e4fb26efcbd33c6d8db4c9936db4` |
| base `cc3-css-gallery-relay463.py` / helper `csd-gallery-import448.py` | `d1822a51…` / `b6653757…` |

## The delta, read against the source

The full 463→472 diff touches only the following. Everything else is the same bytes.

**g: the exact input map.**
- g now adds `is_frozen_set(listed)` to 463's existing count, distinct and re-hash tests.
- Every row except the guard's must equal `frozen_rows()`:
  - each shared and system row at its manifest hash;
  - `Q/448` at `HELPER_SHA256`;
  - the manifest at `--manifest-sha`;
  - the Q permit copy at `permit_sha256`.
- The guard row must be present.
- `POST_PROBE` now prints `listed_rows` as [path, sha256] pairs read from the admitted inputs file.

**c: the receive command and the helper hash.**
- The receipt's `command`, as `str` items, must equal `pinned_receive()`, which is `receive_command(Q/448, chapter, attempt, Q/manifest)`. 448's own `receive` makes exactly this test (`csd-gallery-import448.py:828`), so verify now holds a job to the binding receive already enforces.
- 448 check's `helper_sha256` must equal the pin. That field is `sha(csd_side())`, the hash of the 448 file that actually ran (`:888`).

**The stream-loss stop** now reads "run verify --attempt N when both hosts answer". No "run status" text is left in 472.

**Refactors, equivalent by reading:**
- `frozen_rows()` is 463's `check_prepared` map expression, copied unchanged.
- `is_frozen_set()` is its "guard present, the rest equal" test. `check_prepared` keeps its refusal text, and it returns `rows[GUARD]`, which is the same value 463 returned.
- `pinned_receive()` is `start()`'s former inline admission list.

## Fixtures (reused; scratch, outside the repo)

**CC1's 466 trials, rerun on 472.** I copied CC1's six review-466 fixture files into a new scratch folder. Only three literals were swapped: the relay file name and its sha in `sim466.py`, and the record-folder glob in `test466.py`. I then ran trials a, b and c against Q. All three exit 0.

I compared the outputs leaf by leaf with CC1's 463 outputs. 33 leaves differ:
- **14 intended changes in behaviour**, the same 14 that CC1's receipt lists:
  - In a fabricated completion with a shared row or the Q permit copy substituted at the same count, validate drops from 8/8 to 7/8 and fails g.
  - In S02 and S03, the stop and the record message say `run verify --attempt 1`.
  - In F5, a receipt command naming another helper path or attempt 7 makes verify exit 3 instead of 0, failing c with 7/8. The F5 control is unchanged.
- **9 label-only changes:** the new names of c and g.
- **10 noise leaves:**
  - the manifest-sha prefixes in d's labels, which differ in every fresh world;
  - two S07 timestamps;
  - three F1 receive refusals, which differ only through CC1's documented cleaning of the 463 output.

Trial b (F1r, the F3 a–h breaks and the F4 routing) is byte-identical to CC1's 463 output (`666053991b…`).

**One targeted check of my own** (`test473.py`, 1 world per case). No reused trial flips c's helper clause on its own, so I tested it here. After one completed 472 run, verify runs with 448 check's reported `helper_sha256` replaced:

| Case | Result |
|---|---|
| Control, unchanged | exit 0, 8/8 |
| Replaced with `0000…` | exit 3, fails only c |
| Replaced with an upper-case near-copy of the pin | exit 3, fails only c |

This agrees with CC1's X2 case.

**Scratch hashes:**

| File | sha256 |
|---|---|
| retargeting script | `ec753df3…` |
| runner | `4895c897…` |
| compare script | `f8ec15a6…` |
| compare output | `982a18bc…` |
| `test473.py` | `c091c1c5…` |
| trial a output | `3b44387f…` |
| trial b output | `66605399…` |
| trial c output | `9b807bd3…` |
| `test473` output | `4a35ceee…` |

## Note

g and `--prepared` accept the guard row at whatever hash prepare recorded. That hash is not pinned, but f ties the inputs to the admitted file, and g's re-hash requires the guard to match it now. 448 receive binds the guard in the same way (`bound[GUARD] == sha(GUARD)`). This is unchanged from 463 and is not a 472 defect.

## Not tested

- any CSD or CSS run;
- the real guard;
- a real gallery export or import.

The CSD receive end is scripted, except where the review 466 twin runs 448's own `prepare`, `check` and `receive`.

## Not claimed

This review accepts no import or master. It accepts no picture, motion, sound, sync or AV result, and it authorises no launch or staging. No audio heard or checked.
