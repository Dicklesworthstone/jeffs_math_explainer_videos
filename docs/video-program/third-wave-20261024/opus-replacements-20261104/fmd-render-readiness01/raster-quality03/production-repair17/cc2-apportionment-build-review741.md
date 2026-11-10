# CC2 review 741: guarded CSD build 731 for apportionment chapter 06

CC2 (RubyRaven), 2026-11-10. Independent source review of CC0's sealed helper and handoff. I read both only after CC0
confirmed the seal at 01:21 UTC. No SSH, Cargo, build, render, synthesis, Beads change, commit or canonical edit. Every
local check ran on scratch copies or read-only.

| File | sha256 | Read |
|---|---|---|
| `apportionment-ch06-build731.py` | `8963f280589090c524137465694fc82aabe8d6608f382c6d6e4155bb412d4b00` | in full |
| `apportionment-ch06-handoff732.md` | `c4624178996e5a923428b8043299e7b91d7a36e01b523cd011f0f0735666c605` | in full |
| `exponential-build716.py` | `f9c25c382f8859efe1bf338aad57a12686c1bbb7922dbf5716ed428a797d3bfd` | shared functions compared |
| `apportionment-build656.py` | `b5990c28d25b4daeb7e1247151acc390ccdaae81ee31f74ac911f0ee9670f040` | in full |
| `apportionment-ch06-repair722/handoff722.md` | `0516ec6c1b32b25dc219c027d1b8246c76b46092bccfafc884cdacfdd377fa1e` | in full |
| `cc0-apportionment-repair-review725.md` | `20991b7ad7b88a61c1d4c0ac6d951127b8d9af401c2ba2b19a2f4f3c464795d9` | in full |

## Result

**No blocker. Nothing needs changing before the one guarded launch.** Every pin, test name, parser and write location
I could check without Cargo holds. Four Low or information items follow. None needs a new helper.

## Evidence, by kind

**Native (CSD, run by root; from root's TODO, not seen by me).**

- 731's `inputs` command passed against the retained 656 job, with 4,164 physical bindings and manifest
  `f36ae00a64c488f063f04f3b15121cfb55cca78b0053113975d827723fe49cc8` (TODO entry of 01:22 UTC). `inputs` runs every
  assertion in `evidence656()`, so 656's state is native evidence, not a prediction:
  - its receipt shows a clean exit 0 with stable inputs;
  - its recorded map equals `SOURCE656`, and each of the 4,152 files is re-hashed;
  - its `tests.stdout` has exactly the 30 `ORIGINAL_TESTS` names;
  - its `oracle.stdout` is at `2d622f71…`;
  - its facts equal its oracle at 443 keys;
  - its chapters equal `CHAPTERS`;
  - the five transferred files are at their hashes.
- Guarded 716 finished at 00:47:01 UTC with exit 0 (TODO entry of 00:48 UTC). It used the same kit bytes in place from
  656, the same toolchain pins, and the functions below that 731 shares unchanged. In it, strict Clippy, the cue probe
  and the glyph probe all passed on the CSD.

**Independent recomputation (mine, on the real repository files).**

| Check | Result |
|---|---|
| `SOURCE656` from exports 652/655, 656's copy rule and its two edits | 4,152 files (95 workspace, 4,057 engine), `77fd892a…`. Equal to the pin. The edited root `Cargo.toml` is `2fa23046…` and the crate `Cargo.lock` is `a4187255…`, both equal to 731's and 716's values. The 19 engine packages match. |
| `WORKSPACE731` and `CRATE731` with the four candidates swapped in | 95 files, `eb860846…`; 49 files, `6a63d40f…`. Both equal the pins. |
| `FROZEN` (script, `description_numbers.tsv`, `Cargo.toml`, `Cargo.lock`, `main.rs`, `chapters/mod.rs`, `draw.rs`) and `CARGO_CONFIG` | All equal the recomputed crate. The local canonical crate, kit and score are byte-equal to what 656 built, apart from the edited lock. |
| 722 files | All 11 files in `apportionment-ch06-repair722/` match handoff 722 and review 725. The four canonical files are at their "before" hashes. `git status` is clean for the crate and `tools/`. |
| `ORIGINAL_TESTS` | Equal to the 30 `#[test]` functions in the canonical sources, by module path. None is ignored or should-panic. |
| `NEW_TESTS` | The three names exist in the candidates. The `before-*` copies have only the two existing `facts` tests, so the total is 33. |
| Oracle, scratch copy of the candidate crate | Stdout is 443 lines at `2d622f71…`. `--check` gives byte-equal stdout, and stderr contains `ok census year 1880: the sourced row and the narration agree` as a whole line. The oracle writes no file. |
| `script_rows`, `check_script` and `static_cues` (731's own code, run on a scratch candidate crate) | 93 cues, 4,166 words, 17 modules, each chapter's cues in order, 06 says 06_1 to 06_6. |
| `NEW_STRINGS` | All five reach `kit.text`, which records them as `Text`: `not_every_time.rs:95` (labels), `:97` (question) and `:194-200` (the Seaton line, with the year from `crate::facts::seaton_census_year()`). The candidate's tests pin the labels and the question exactly. The kit writes registry rows as `{kind:?}\t{chapters}\t{source:?}` (`glyphs.rs:122-131`). For these strings, Rust's Debug form equals the helper's `json.dumps(..., ensure_ascii=False)`, because no string has a control character and "→" is printable. |
| Kit output formats against 731's parsers | `script` prints `{id}\t{text}` after the same skip, trim and `#` rules as `script_rows` (`cli.rs:361-364`, `narration.rs:358-386`). Scratch-voice `cues.tsv` rows are `{id}\t{at:.3}\t{len:.3}` with no header, sorted by time (`cli.rs:712-719`, `narration.rs:1045-1049`). The glyph probe prints `glyph registry K: N sources -> P` per chapter and one `glyph probe: N sources, F failures -> P` line, and writes `failures.txt` empty when F is 0 (`glyphs.rs:95-149`). |
| Where the binary writes during the probes | Only under `--out` and the glyph probe's target, both in the job folder (`cli.rs:577-600`, `glyphs.rs:70-72`). The engine's typeset cache uses `XDG_CACHE_HOME`, which is `work/cache`. Kit sounds and engine temp files use `TMPDIR`, which is `work/tmp`. Nothing writes into the job crate, so the final `WORKSPACE731` re-listing can pass. |
| Guard and stop code against 716 | `end_group`, `on_stop`, `run_step`, `said_cues`, `script_rows`, `passed_tests`, `fact_rows`, `check_files`, `cargo_home` and `env_problems` are byte-identical. `own_group` and `read_receipt` differ only in "716" becoming "731". `guard_binding`, `toolchain` and `config_problems` differ only where 716's host profile became 731's fixed CSD constants. `STOP_SIGNALS`, `TERM_GRACE`, the forbidden environment and config lists, the toolchain pins and `CARGO_VERSION` are equal. |
| 06_1 hold record (a later step) | `docs/narration-direction-map-frederick01.json` (`d6d21eb2…`) has 15 `visual_holds`. `[5]` names `not_every_time.rs` at `d3e88f7b…`, line 58, marker `let q = think_here(&kit, stage)?;`. The candidate has the marker once, at line 102. This matches handoff 732 and `qualification731.json`'s `required_before_reuse`. |
| Retained provenance | `retained` is `CHAPTERS` without 06, so 14 chapters. Export 682 holds only 01 and 02 (5 files each). Root's TODO records final672 03 to 05 as exit 0 from `apportionment-native656`. The record claims nothing for the 731 binary beyond chapter 06, and it says so. |

**Simulated (CC0's scratch stub, not repeated by me).** The stops, the refusals after launch and at `inputs`, and the
control are described in handoff 732. They used stand-ins for Cargo, the binary and the receipt, so they are not native
evidence.

**Not established.** Nothing has compiled. These have not run natively on 731's sources:
- the 33 tests and strict Clippy;
- 731's facts, chapters and script output;
- the cue and glyph probes;
- any stop while ffmpeg is running.

No glyph page has been read. Zero glyph failures will not accept the pages: root's review 729 of 716's pages found
duplicated `n` denominators on page 39, which the probe had passed (TODO entry of 00:57 UTC). The helper says the pages
still need reading (lines 49-50). The pages that hold 06's five new strings should be among the first read. No
narration, picture, motion, fine-sync, AV or release claim is made.

## Low and information items

**L1, Low: strict Clippy over the test code is new for this crate on the CSD.** 656 linted `--bin apportionment` only,
so the 30 old tests have never run under `--all-targets -D warnings` there. A finding in old test code would stop the job
after the tests, before build, facts and both probes, and would leave no native 06 evidence. Handoff 732 says this.

The 722 tests add no new kind of lint risk. Their one `assert_eq!(…, vec![…])` has the same form as 21 existing
assertions in `model.rs` and `draw.rs`. Their format strings are already inlined or take indexed expressions. No action
before launch. If Clippy stops in an old file, the fix is a canonical file outside 722's four, so it needs a new
candidate set and a successor helper, as handoff 732's table says.

**L2, Low: the ffmpeg stop path depends on `guard_csd02.py`, which neither CC0 nor I have read.** It is not in the
repository. The engine starts ffmpeg in its own process group, so the helper's `killpg` does not reach it, and handoff
732 relies on the guard's own stop. This is the same as in 716, whose cue probe finished without a stop. It matters
only if a stop arrives during the cue probe. When the binary dies, ffmpeg's input should close, but that has not been
tested. If root knows how `guard_csd02` stops a session, the question is closed.

**L3, information: `qualification731.json` lists three of handoff 732's five later steps.** Its `required_before_reuse`
names the hold rebind, the 93 PCM recordings and the 45 tables. It leaves out item 4: the four runtime tools under the
job's `workspace/tools/` before any `--narration` render. Root's TODO entry of 01:24 UTC describes CC0's successor to
664/666/667 (744/745) with the same three conditions and no runtime tools. That successor should take its preconditions
from handoff 732, not only from the JSON.

**L4, information: review 725 F1 stays open and cannot affect this build.** The glyph check is native proof that the
sourced 1880 row reaches the binary, and the cue probe proves 06_4 is said. Nothing ties the census phrase to 06_4
itself. But `script.tsv` is pinned by hash and compared byte for byte with the binary's script, so this script cannot
move the phrase.

## Launch conditions I agree with (from handoff 732)

- One launch, exactly as written: `inputs` has already run, then the guard command.
- No other Cargo job may use `work/target`. The helper does not lock it. Cargo's own build-directory lock serializes
  Cargo, but not a later copy of `target/release/apportionment`.
- A guard 75 or a resource stop means a rerun as `-a2` with its own `inputs`. A source stop means a successor with new
  names.
- 656's job folder and the candidate folder are read only. The helper copies, never links, and re-hashes all 4,164
  bound files at the end.
