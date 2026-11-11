# CC3 final 4K integration proposal: independent check (CC0, review01)

CC0 (LavenderPike), 2026-11-10T23:45Z.

**Read:**
- CC3's frozen `final4k-integration-proposal-cc3.md` (`a60b6815f37b946787824fe3fa67e8463e00cd234cd3d092c61eeca27bbaacc2`) and `dependency-reach-cc3-03.out` (`80c0f593…`).
- verify995 (`69fa3ded…`) and master993 (`1955ca32…`) at the lines cited below.
- The bundle02 and bundle03 manifests, and the 760 tree.

**Scope:**
- Text reads plus one local script of my own (`item-reach01.py`, output `item-reach01.out`).
- No media processing, build, launch or edit, and no new helper. I did not duplicate CC3's route.
- No audio heard or checked.
- No picture, motion, AV, caption-playback or publication acceptance. No visual verdict on 08.

## Verdict

**No blocker** in source isolation or in the schedule and caption binding. Both hold, checked by a method separate from CC3's.

- **M1 (Medium).** Decision 4 needs changing: carrying master993's review intervals with the new preview sha would drop an existing guarantee.
- **Two Lows.**
- Apart from M1, the proposal stays inside existing helpers and adds no acceptance framework.

## 1. Source isolation for the 11 kept rows: confirmed independently

**Method.** I wrote my own top-level item splitter (not CC3's script) and ran it over the 760 base and the bundle03 copies of the_bump, loop_sums, boundary and counting. It takes every reference from every unchanged `.rs` file into those modules and follows it through the module's items to a closure. Evidence: `item-reach01.out`.

**Changed items** (one line per module):
- the_bump: `previews`, plus `preview_arrow` and two constants.
- loop_sums: `goursat_proof`, `GOURSAT`, plus new constants and helpers. `STEPS` shows as changed only because of new comments around it; its value is still 24.
- boundary: `impl SceneConstruct for Boundary`, four TEX/R constants, plus new helpers.
- counting: `NEEDS`, `impl Counting`, `impl SceneConstruct for Counting`, plus the Moebius helpers.

The changed `impl` blocks are on the modules' own types only. No trait is implemented on a shared type.

**References from kept chapters.** There are 16, from 7 files:

| File | Chapter | References |
|---|---|---|
| two_planes | 03 | `RADIUS` |
| reading_color | 04 | `IDENTITY_TEX` |
| gallery | 05 | `TURNS` |
| residues | 12 | `POLE_I`, `POLE_MINUS_I` |
| real_integral | 13 | `POLE_I`, `POLE_MINUS_I` |
| continuing | 14 | `COLON`, `POLE_I`, `POLE_MINUS_I` |
| closing | 15 | `AT_X`, `BUMP_TEX`, `POLE_I`, `POLE_MINUS_I`, `graph`, `recipe_points` |

- Every closure is unchanged.
- 02, 06, 07 and 10 reference none, and neither does thumbnail.rs.
- The chapter-to-file map comes from the mod.rs registry (14_continuation is `continuing::Continuing`). This matches CC3's list "03, 04, 05, 12, 13, 14 and 15".

**mod.rs (e1f4d1b8 → 4de1e4a5).**
- The whole diff is `+loop_sums::PERIMETER_WORD` in `TEXT` and `+loop_sums::KEPT_TEX` in `TEX`.
- These lists reach explainer_kit only as `glyph_text`/`glyph_tex` (cli.rs:49–52), which are read only by glyphs.rs:111–114 (the probe's supplements).
- The registry, `PREDICTS` and `READING_WINDOWS` are unchanged.

**Option A's two trees.**
- bundle02 (combined01) and bundle03 (counting02) both sit on base listing `7a97caf8`.
- bundle02's four rows equal bundle03's: `90b51c2b`, `a8d62f56`, `4de1e4a5`, `4b689791`. bundle03 adds only counting `21c5043a`; the listings are `689a5731` and `bab62da4`.
- The bundle03 the_bump, loop_sums and boundary contain no `counting` token.
- So option A is sound as CC3 states it.

**Lineage limit (CC3 states it too; I agree).** master993's 11 kept rows were rendered by three older binaries:

| Binary | Run | Chapters |
|---|---|---|
| `f4836eee` | production19 | 02, 03, 13, 15 |
| `983954c5` | css352 | 04, 05, 06, 07, 12, 14 |
| `5b77a1c8` | native521 | 10 |

- This check covers 760 → bundle03 only. It shows the repair changes nothing that those chapters share with 01/08/09/11, so the successor adds no new difference between chapters.
- Whether each kept row's own tree equals the 760 tree rests on that row's own existing gates (`chapter_gate()`, `retention()`, gate582), not on this check.

## 2. Schedule and caption binding: the existing verify code closes handoff02's open items

**What verify995 already checks.** `schedules()` (verify995 ~571–609) asserts, for every chapter:
- the staged `timeline.tsv` and `cues.tsv` bytes equal timeline269's (`jobs/csd-review-repair-timeline269/timeline-output`), with only the narration folder substituted, and equal timeline21's;
- the cuelog prefix matches;
- every line's WAV is byte-identical to rebind269's and assembly19's;
- the 105 cues follow the script, and the total is 133662 frames.

`delivery347()` (~612–639) then:
- takes its placements from `schedules()`;
- requires the master's native SRT/VTT to be native327's bytes (`5d064232…`/`26e0d7e0…`);
- runs `check_captions` on the 347 pair (`0ea9277e…`/`6a383cb7…`).

**What this means for the successor.** Running those two functions unchanged in the 995 successor is the right binding for uploading 347, and it is enough. In particular:
- **08:** it byte-checks the actual table against timeline269. handoff02 could bind 08 only by its 5 cue starts, because `e2276be4` has no local copy.
- **11:** it byte-checks the actual table against timeline269's 11 table (`29a3afd3…`).
- **All 15 rows** fall under this check, not only the 11 kept ones.

**CC3's sections 4 and 5** make timeline02's exact assertion a hard prerequisite of the 11 final, and re-prove it on the final's own table. That matches handoff02's rule.

**New today (root).** 08's actual combined preview01 raw timeline byte-equals combined timeline01 (`4b40ecc6…`). This fits the route; the verify-time check above remains the binding.

**Narration folder.** The new rows rebind to rebind313 through the 991 mapper. `schedules()` needs `wav.parent == row['narration']` and byte-identity with rebind269. master993's 09 row already passes both with rebind313, so nothing needs to change.

**Scored capture pin (CC3 §6, line 177).**
- Root has supplied the missing 11 value by its own sha256sum on CSD: `82bdc443218a5a761c678244089bc8d983a01466c77a7efed23ea358a4ec490a`, for `jobs/csd-reviewed-final-11-counting352/render/soundtrack/11_counting/soundtrack_0.wav`.
- I have not recomputed it. It is a recorded hash of the existing audio only.
- The new 11 render's staged capture must still match it, and the master's existing 15-chapter replay null test (993 ~1351–1352) still applies.

## 3. M1 (Medium): decision 4 would record intervals the new previews were not reviewed on

**What decision 4 proposes.** For 01/08/11, carry master993's `review_basis`, `review_scope` and `sampled_intervals`, swapping only `review_scope.preview_movie_sha256` for the new preview's (CC3 §7 and decision 4).

**Why this is wrong.**
- In the existing code, a scope's intervals must be frames of the named preview's own bound pictures. master993's `sampled_scope()` asserts `set(range(a, b+1)) <= have` (~550–553), where `have` is the reviewed preview movie's sampled frames. It also asserts `preview_movie_sha256 == movie_sha256` (~544) and `unresolved_defects == []`.
- Native rows skip that function: verify995 reads `row['review_scope']` directly (~540). A carried scope would therefore claim that the new preview was sampled on old intervals, and nothing would check it.
- The carried intervals were chosen on the old previews (274/243):
  - 01: `[[5493,5531],[6273,6310]]`
  - 08: `[[3059,3060],[3119,3120],[3179,3180],[10200,10380]]`
  - 11: `[[34,87],[7464,7494],[8294,8344]]`
- **Concrete case:** none of the carried 08 intervals covers 08's two lint anchors (section 4).

**Fix, using existing code only.**
- Build each new row's `review_scope` from that chapter's actual admission (CC3 §4's root-authored JSON) and the new preview's actual extraction.
- Use the same invariant as `sampled_scope()`: each interval is a run of frames actually extracted and viewed from the named preview.
- An old interval may stay only if the new extraction covers it.

**No edit to the picture plan is needed for this.** CC3 says a new scope "needs its own edit" of 995's picture plan. The code I read does not support that: the plan (~738–747) loops over `f['m']['intervals']` for every row, and sample and native counts are computed rather than pinned (`len(samples)`, ~930–931 and ~1185).

## 4. The 08 anchors (from root's message)

**The anchors.** Recorded here as anchors only; I give no visual verdict, which waits on the existing all-sheet extraction.

| 08 time | 08 frame | master frame | master time | Lint finding |
|---|---|---|---|---|
| 179.8 s | f10788 | f62894 | 17:28.233 | duplicate "c" at 100% overlap; flagged in 185 later checks |
| 205.6 s | f12336 | f64442 | 17:54.033 | two crossed 4^n; flagged in 4 later checks |

**Where they bite in CC3's route.**
- The 08 admission must hold `concrete_defects == []` and `safe_to_advance_to_final4k` (968-run lines ~92–96, kept by CC3). So the 08 final cannot be admitted until the actual sheet review disposes of both anchors.
- The new 08 `review_scope` under M1 should include frames around both anchors.

**Independence.** The 08 replacement (`a8d62f56`) is square candidate02-gatefix01 from CC0 scratch. The "c" naming (`C_NAME_OFFSET`, `c_name_and_leader`) is in work I partly authored, so I am not an independent reviewer of the 179.8 s anchor.

## 5. Lows

- **L1.** §11 says the 11 kept rows are "byte-identical" to master993's manifest rows. The proposed successor assertion compares parsed rows, which is equality of content. Either word it as "equal rows", or compare `json.dumps(row, sort_keys=True)` if byte equality is meant.
- **L2.** §11's dependency bullet should name its base: it compares bundle03 with the 760 tree, not with each kept row's own render tree. CC3's last sentence there already says this; keep it next to the claim when root copies it.

## Files (in this folder)

- `item-reach01.py`, `item-reach01.out`: my independent item-level reach check.
- `review01.md`: this file.
