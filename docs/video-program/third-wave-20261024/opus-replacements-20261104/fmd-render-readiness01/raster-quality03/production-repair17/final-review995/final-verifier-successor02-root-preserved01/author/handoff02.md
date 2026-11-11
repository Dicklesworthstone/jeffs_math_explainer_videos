# 995 successor02 (final-verify-cc3-02): CC3 scratch draft, CC0 agreed

CC3 (WhiteSwan), 2026-11-11. This folder is new. The sealed first successor in final-verify-cc3 (`5c30a62e`,
handoff `05e68894`) stays as it is, and so do every earlier attempt by CC0 and by me. No job was launched, the
preview was not touched, and no production file was edited. No audio was heard or checked. Nothing here grants
picture, motion, fine-sync, caption, listening or AV acceptance.

## Files

| File | sha256 | What |
|---|---|---|
| csd-final-verify-repaired02.py | `1220fdb1ffe84d06e24c3c4e836ef00d97ed0b8e65e9151b5754b1ee23de6901` | successor02 |
| verify02-vs-01.diff | `3414342e5c01ecd01693a3a11a3d30b1fa9dc0b3a05912a77ee976f47258eee6` | +35 -2 against `5c30a62e` |
| verify02-vs-995.diff | `c425d1a87f5f93a58180aedf9a54bc98692e760c10ce077a5e638730bfd59a04` | +118 -32 against `69fa3ded` |
| check-final-verify-cc3-02-r2.py | `1c8d7b6d381126f1e8f869a6e6501c942abaa7601428629216b40e2e91e97c6e` | local checks |
| check-final-verify-cc3-02-r2.out | `a7f1c40ff39cd785dc920f96b9b355595a6d5716b3e8e486c06f852c7c685bfb` | 66 passed, 0 failed |
| check-final-verify-cc3-02.py / .out | `336b988b…` / `fe2fce92…` | first attempt; a syntax error in my rewrite, never ran; kept |

The after-master launcher is unchanged: `final-verify-cc3/csd-final-after-master-repaired01.py` (`15094001`). Its
`HELPER_NAME`/`HELPER_SHA` pins name the adopted verifier, which is now successor02.

## What changes from successor01 (CC0 review01 F1, with CC3's R1 placement)

Under M1, a repaired row's scope can hold only frames that the new preview extracted and root viewed. So the
admitted scopes need not contain master993's old sampled intervals of 01, 08, 09 and 11, which 995 planned
every frame of. Successor02 keeps those intervals in the plan as stills of the new final:
- **`PRIOR993_INTERVALS`** holds the `sampled_intervals` of master993's rows 01, 08, 09 and 11. `MASTER993_SUMMARY`
  is master993's own job summary at `38b52105` (I read the CSD copy's hash read-only). `plan_for` requires the
  summary at that hash and the constant to equal its rows. It binds the summary in `used`, and the pictures
  record lists the constant.
- **`picture_plan`** adds every frame of those intervals, labelled "master993 sampled interval a-b, every frame
  (old render review scope); final still only", plus a native at each interval's middle, under 995's own rule.
  The rule is evaluated on `natives995`, the natives that existed before any sampled interval was planned, which
  is the set 995's rule saw.
- **Placement.** The block sits after the 377 required windows. So it never displaces an admitted-scope native or
  a required-window native, or their labels. That is R1: CC0's fix02/fix03 placement lost them, including whole
  admitted natives when an admitted interval encloses an old one. CC0 withdrew fix03.
- The labels claim no viewing, playback or listening, and the old intervals do not enter any row's
  `review_scope`.

Everything else in successor01 is unchanged:
- the pending pins and their refusal;
- the four repaired kinds;
- the final extraction carry-over;
- root 343's four 09 instants as final stills;
- 995:1102-1104;
- every technical, caption, schedule and frame-identity check.

## Evidence

**CC3 review of fix02** (folder fix02-review-cc3, sealed): `review-fix02.md` `520cafca`, with the ALT test copy
`0f31010e`. Successor02's code is AST-equal to the ALT test copy; only the module docstring differs. On four scope
cases (master993's intervals, smallest, wide, disjoint), ALT:
- removes no frame/reason entry of `5c30a62e`;
- keeps every one of 995's 8351 samples and 123 natives;
- adds only the two master993 labels.

**CC0 validation** (Mails 25195 and 25213). CC0 ran seven cases (my four plus wide2, partial overlaps and
disjoint2) on ALT and then on `1220fdb1` itself, and found the same in every case. With master993's intervals as
the scopes, the natives equal 995's. CC0 agreed to seal as is.

**CC3 checks** (66 passed): the 57 checks of successor01, adapted. They add:
- the AST equality with ALT;
- that against successor01 only `picture_plan`, `plan_for` and `pictures` change, and only the two names are added;
- the summary path;
- that `plan_for` binds the summary and refuses a changed constant or another hash.

**Preservation of 995's frames (CC0 L2).** 995's samples and natives are planned in every tested case. This also
holds by construction: 995's rule is applied to the same intervals against the same pre-interval natives. This
draft makes no claim beyond those two grounds.

CC0's L1 (an extra prior775 assert) is not taken. The run proofs and the extraction helper already require prior775,
and root asked for no new gates.

## Pending pins (unchanged; `None`, refusing)

**Verifier:**
- `VERSION` and `MASTER_VERSION`
- `MASTER_HELPER_NAME` and `MASTER_HELPER_SHA`
- `STAGE_VERSION`
- `WORKER_NAME` and `WORKER_SHA` (CC0's impl02 materializer `e5abe22e`, by its Q copy)
- both `REPAIRED_BINARY` values. For combined01 this is the value read on CSD, `aed99576b7a9cc42353c43963e1828acf3465426e028fc424eaf6712133725ce`; for counting02 it is pending its build.

**After-master:** `HELPER_NAME` and `HELPER_SHA`, which name successor02's Q copy.
