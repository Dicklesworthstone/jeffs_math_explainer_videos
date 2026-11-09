# Review 378: verifier 377 (CC0)

cc0, 2026-11-08, 22:35 UTC. This is an independent review of CC2's frozen `csd-final-verify377.py` and its notes, against root's disposition of review 367. Q is this folder locally, or `D/qualification01/production-repair17` on CSD.

Nothing ran on any host. The only host commands were read-only `sha256sum` and `ls` on CSD (section 5). Every replay and mutation below ran locally, in memory, on read-only stand-ins. None of them is the CSD `check`. No audio heard or checked.

## Verdict

- No High and no Medium finding.
- One Low, L1. Three comments name 07's rule, but 07's instants get no native frame. The rule as coded for 09 is the one root asked for, so only the wording is wrong.
- Root's disposition 377 is met on every point (section 2). Nothing 363 planned is lost.
- 377 can be used as frozen. L1 needs no code change unless root also wants natives at 07's instants, which disposition 377 does not ask for.

## 1. Files

| File | sha256 | State |
|---|---|---|
| `csd-final-verify377.py` | `827be351861d0fd4ebc083aa24d594111b059dd9f78c7ca43072b0b6f12f22fe` | reviewed; 1219 lines |
| `cc2-final-verify377-notes.md` | `d176024bacf77fc3e853b99f928e5c8b564fad5e24c63218145ca770ae6c78d7` | reviewed; 124 lines |
| `csd-final-verify363.py` | `b4f687a58bc7233341e2b19fcfcf9b886faf31217d255d68073e62c9ecf09c1a` | unchanged |
| `cc2-final-verify363-notes.md` | `90f08901434408d4aa49bd4b8ead8d9aeffd276a973cbccb6b53e1a7e0186272` | unchanged |
| `cc0-verifier363-review367.md` | `f577a1b36768ac3b59caaec6f96b0790398a683a2525c6d70a72427bc2b62314` | unchanged |
| `root-verifier367-disposition377.json` | `ecc64557495d90d7762291df7e22284a22f8b98969eb58ad5175a95bc48537ff` | root's; the basis |
| `derivative-sheets359-review.json` | `40e2fca805e0dfb7b138b2ef68c2770b947bd7ae5e435e3d712bf2ea476d34de` | root's |
| `final-boundary-permit351.json` | `873c33bde99650c7f471c690852dff8860c03ef44b9ff8c695ce57788ba7b419` | read only |
| permits 364, 366, 370 and 375 (ten rows) | as in Q | read only |

## 2. Root's requirements, one by one

| Requirement (disposition 377, review 367) | 377 | Evidence |
|---|---|---|
| 09 playback centres, round(s*60) ±2 | `master350()` returns each sampled_preview row's `playback_seconds` (:466–475). `picture_plan()` samples each centre and 2 frames either side (:661–669). | Replay: permit 351's full-precision instants 27.68726914660832, 31.90054923413567, 35.51193216630197 and 39.12331509846828 s give centres 1661, 1914, 2131 and 2347. All 20 frames are samples. Each sheet label names its instant. |
| A native at each 09 centre | `add(natives, key, centre, ...)` (:669) | Replay: all four centres are natives. |
| 359 bound | `FINAL_FRAME_SOURCES` at `40e2fca8`. `plan_for()` asserts chapter 06, movie `5e45d72d` and exactly the two `viewed_intervals` (:1006–1009). | Mutation: other windows, another movie or another chapter are each refused. |
| 06 windows 9630–9730 and 9970–10029 | `REQUIRED_WINDOWS` (:194). Each window must lie inside one of the final's sampled intervals (:1010–1012). Every frame is sampled, with a native inside (:670–677). | Replay: every frame is sampled; natives at 9680 and 9999. Mutation: 06 with 9970–10020 is refused. |
| 08 run 10200–10380 | `REQUIRED_WINDOWS`, citing 377, with the same containment and planning | Replay: every frame is sampled; native at 10290. Mutation: 08 with 10200–10379 is refused. |
| Natives 11 2010, 2280, 4050 and 14 1140 | `FINAL_FRAME_CHECKS` (:180–197). The per-chapter union must equal `required_native_frames` (:996–1002). | Replay: all 17 frames in disposition 377, plus 06's seven, are both samples and natives. Mutations refused: 377 without 1140, 377 without 2280, 377 with an extra 12 frame, code without 2010. |
| Endpoints 11 1860, 2160, 3840, 4260 and 14 1080, 1200 | `FINAL_ENDPOINT_CHECKS` (:189), samples only. Must equal `additional_overview_endpoints` (:1003). | Replay: all six are samples and none is native, as the record intends. Mutations refused: 377 without 4260, code without 1200. |
| Disposition 377 pinned | `FINAL_FRAME_SOURCES` hash. `d377['review'] == REVIEW367` (:995). Flags must be exactly `False` (:1014). | Mutations refused: another review hash, `picture_acceptance` true, an acceptance flag set to `0`, a changed pin. |
| Earlier anchors kept | All but 12 of the top-level items are AST-identical to 363: 95 of 107. They include `stable`, `success`, `film`, `verify`, `verified`, `delivery347`, `schedules`, `frame_hashes` and `launch`, plus every window, instant, layout, flag and budget constant. | Replay: 0 of 363's 8,077 samples and 0 of its 119 natives are lost. Every 363 final-frame check is still a sample and a native. Every permit interval has every frame sampled and a native inside. |
| 363 and 367 preserved | 377 is a separate file. | Hashes unchanged (section 1). |

The native caption check and the 347 delivery distinction are unchanged: `verify` and `delivery347` are AST-identical. The overview ledger comment and its procedure in `pictures()` are also unchanged. The diff there only adds three fields to `sampling.json`.

## 3. Findings

### L1 (Low). "07's rule" is said to include a native centre, but 07's instants get none.

- Where the claim is made:
  - the docstring at :8–9 says the instant's centre "is a native 4K frame: 07's rule";
  - the comment at :661 says "as 07's: round(s*60) +-2, native at the centre";
  - notes §2.1 says "This is 339's 07 rule, `ONE_POINT_INSTANTS`, applied".
- What 363 and 377 actually do: `picture_plan()` gives the five 07 instants (4858, 4872, 5099, 5355 and 5681) samples at ±2 only (:642–644). In the replay none of the five is a native.
- Effect: the 09 behaviour is correct, and it is what disposition 377 asks for ("centers ... plus/minus2 and native centers"). A reader of the comments would wrongly conclude that 07's instants have 4K natives.
- Repair:
  - wording only, in a successor or a notes addendum: "07's sampling rule, plus a native at each centre";
  - or, if root also wants 07's centres native, that is a root decision. It adds 5 natives, about 0.12 GiB at 3840×2160×3 bytes each. Headroom falls from 21 to 16, and the native part rises from 3.006 to about 3.12 of 3.5 GiB. Disposition 377 does not ask for this: its playback requirement names the permit's instants, and 07's come from root 305, not from a permit.

### Notes (no change needed)

- **N1.** The 08 check `'10200-10380' in ' '.join(d377['requirements'])` (:1004) matches text in root's prose. Disposition 377's bytes are already fixed by hash, so this only ties `REQUIRED_WINDOWS` to the record.
- **N2.** A NaN or infinite instant makes `round()` raise `ValueError` or `OverflowError`, not the named assertion (:472). Either way 377 refuses.
- **N3.** 359 has no `*_acceptance` keys, so the flag check at :1014 passes on it trivially. 359 claims no acceptance.
- **N4.** 377 never reads review 367's file. It compares disposition 377's review entry with `REVIEW367`, which is enough because disposition 377 is pinned by hash.
- **N5. Counts.** These are from my replay on the real permits, with plan367c's stand-in layouts and empty timeline cue starts:
  - 377: 8,097 samples and 127 natives;
  - 363: 8,077 samples and 119 natives;
  - parts: overview 1.531 GiB, native 3.006 GiB, each against a 3.5 GiB cap; 148 natives fit, so 21 spare.

  CC2's fixture gave 8,674 and 126 because its layouts differ. The real counts come only from `check` on CSD.
- **N6.** On the real permits, `REQUIRED_WINDOWS` adds reasons, not frames: none of the 20 new samples is in 06 or 08. Its native branch does work when a window sits inside a wider interval. With 06 given one interval, 9600–10029, the plan adds one native, for 128.

## 4. Methods

- **AST.** I compared the top level of 377 against 363: 95 items identical, 12 changed, 4 added, 0 removed, as CC2 lists. I read the 153-line text diff in full.
- **Replay** (`plan378.py`, sha `573825b5…`; `label378.py`, `0bba64de…`). This runs 363's and 377's own `picture_plan()` and `part_upper()` side by side. It uses plan182's chapters, the real permits 351, 364, 366, 370 and 375, local layout tables and preview313-review333. 377's playback dict is built from the permits' `review_scope` the way `master350()` builds it.
- **Mutations** (`mut378.py`, sha `8f31a6c8…`): 33 of 33 cases behaved as expected.
  - 22 cases run 377's `plan_for()` with module globals pointed at local copies. Each JSON record is altered only in memory, after its real bytes pass the hash check.
  - 11 cases evaluate the exact source of `master350()`'s playback predicate, taken from the AST. They accept `[]`, the real instants and the last legal centre. They refuse 0.01 s, `True`, `'27.687'`, a tuple, an instant past the chapter end, NaN, and a centre of 1.
- All three scripts are CC0 scratch files outside the repo. They wrote nothing.

## 5. CSD staging (read-only, 22:30–22:33 UTC)

- `csd-final-verify377.py` and its notes are in CSD Q at the hashes above.
- `sha256sum -c` passed for all 25 of 377's `TRANSITION_SOURCES` and `FINAL_FRAME_SOURCES` in CSD Q. These include `cc1-gallery-transition-review319.md` and `open-region-playback-second305.json` (staged by root in 383), disposition 377 and 359. So the plan_for refusal noted in CC2's notes §3 no longer applies.
- `csd-final-verify363.py` is not in CSD Q, and 377 does not need it.
- CSD has no `csd-final-verify*`, `csd-final-pictures*` or `csd-css352-import371*` job folder.

## 6. Not claimed

- This review accepts no picture, motion, sound, fine sync or independent final AV. It authorises no launch or publication.
- My replay and mutation counts come from local stand-ins, not from CSD.
- No audio heard or checked.
