# combined01 native build: executable helpers

CC2 (RubyRaven), 2026-11-10. These files are prepared only.
- Nothing was staged on CSD, launched, built, tested or rendered.
- No production file, TODO or Bead was touched.
- The CSD and CSS reads were read-only, over ssh.
- Frederick narration is unchanged and was not assessed.
- This is not native or picture acceptance.

## M1, resolved from CSD

The successful build844 ran `csd-native844-build.py` `7932fe78…`. Three CSD records show it:
- **The receipt:** jobs/csd-native-build844/resource-receipt.json has exit 0, and its `command` is that helper with `--job csd-native-build844`.
- **The inputs:** `inputs_sha256` and `final_inputs_sha256` are both `3b54974b…`. That is D/Q `csd-native844-inputs.json`, and its row binds `csd-native844-build.py` at `7932fe78`.
- **The superseded version:** `20953c4d` is preserved on CSD as `csd-native844-before-config-fix.py`.

The command.log on CSD is empty (0 bytes).

## Frozen files

Folder `csd/`, SHA256SUMS `48a77b17…`. Copy each file to D/Q (D/qualification01/production-repair17) under the same name:

| File | sha256 |
|---|---|
| csd-combined01-build.py | `bfb59efdcd2c55d07d6ea5c602265162e4858ebc77ce842c78c7af04e3e8d156` |
| csd-combined01-launch.py (pins the helper) | `ff9083eb230661b1744c8b1fea9a1887999baad508e782e2bd111d9cba209630` |
| csd-combined01-bundle02-manifest.json (= CC0 combined-repair-bundle02/manifest.json) | `9c29672e24e7a64eca02ebede268a4b93a757f30fc42d0c8dc476c74817ad4b9` |
| csd-combined01-the_bump.rs (arrow candidate03) | `90b51c2b…` |
| csd-combined01-loop_sums.rs (square candidate02-gatefix01) | `a8d62f56…` |
| csd-combined01-mod.rs | `4de1e4a5…` |
| csd-combined01-boundary.rs (boundary candidate03) | `4b689791…` |

The build reuses six inputs already on D/Q from build844, each pinned by the launcher:
- csd-native844-base761.py
- csd-native844-source760.json
- csd-native844-narration_units.json
- csd-native844-Cargo.toml
- csd-native844-config.toml
- csd-native844-engine-budget.toml

## Commands (root, on CSD)

```
/usr/bin/python3 -I -B D/Q/csd-combined01-launch.py --check    # verifies, writes nothing, starts nothing
/usr/bin/python3 -I -B D/Q/csd-combined01-launch.py --launch   # writes csd-combined01-inputs.json (x), runs the guard once
```

`--launch` runs exactly:

```
/usr/bin/python3 -I -B D/guard_csd02.py --phase native --job csd-native-combined-build01 \
  --inputs D/Q/csd-combined01-inputs.json -- /usr/bin/python3 -I -B D/Q/csd-combined01-build.py --job csd-native-combined-build01
```

These names were checked free on CSD today:
- job: jobs/csd-native-combined-build01
- source: work/native-combined01
- inputs: csd-combined01-inputs.json
- binary: complex_analysis_visual-combined01

The build's metadata tag is `jmev_combined_repair01`.

## What the helper keeps and changes

**The helper is 844's own code.** It keeps:
- the base761 group, the stop signals and POLICY;
- the receipt checks, the 760 manifest and its listing;
- dependency_check, OLD's two allowed extras, and the OLD row hashes;
- the Cargo.toml and config path mapping, and the home-config and ancestor-config checks;
- the toolchain version checks, the 4-CPU affinity and the environment;
- the plate_fade_probe run, Clippy `-D warnings`, `cargo rustc`, facts `7d528509` and the 15 chapters;
- the unchanged listing, dependency and config checks at the end.

**The changes:**
- **Four substitutions from the pinned bundle manifest.** For each row:
  - the 760 base row must equal the bundle's base;
  - the OLD file must equal the 760 base row;
  - the staged file must equal the replacement;
  - the copied file is re-hashed.

  The 760 rows with the four replacements must give the bundle's combined listing, `689a5731…`.
- **Formatting:** `rustfmt --check` runs on all four files.
- **Tests:** exactly `247 passed; 0 failed; 1 ignored`. Each of the 12 new tests must be reported `ok` once:
  - the_bump: 1;
  - loop_sums: 4;
  - boundary: 7.
- **Toolchain:** cargo-rch-real is required, never the RCH script. The five binaries must equal build844's preflight844.json (`37a28c9c…`):
  - cargo-rch-real `19008157…`
  - rustc `49f9fa06…`
  - cargo-clippy `92708a26…`
  - clippy-driver `2cd2fc26…`
  - rustfmt `016e3712…`

  Versions checked: `cargo 1.100.0-nightly (e8cb624d5 2026-08-22)`, and rustc commit `90850177…`.

**The launcher's inputs manifest** mirrors build844's 4216 rows:
- the guard files;
- the Q inputs;
- both Cargo configs;
- the six toolchain binaries;
- all 72 OLD files;
- all 4125 work/tree rows;
- the bundle manifest and the four staged files.

## CSD state read today (read-only)

**Base:**
- All 4125 engine and repo-view rows still equal the 760 manifest.
- OLD's 70 rows equal it too, apart from the 3 rows 844 exempts.
- OLD's extras are exactly the two review313 files.

**Disk and load:**
- `/` has **19 GiB free** (98% used); the guard admits at 10 GiB and stops at 8 GiB.
- build844 grew D/work by about 1.1 GiB.
- Load is about 14, from other tenants.

**Expected noise:** cargo prints `error: unclosed table` for a malformed test-fixture Cargo.toml in an asupersync git checkout. It appeared in build844's build and Clippy logs and is not fatal.

## Local self-test (helpers not run)

- Both helpers parse.
- `bundle_check` passes on the local 760 manifest and bundle02, and refuses four planted faults.
- The 12 new test names are found, and one missing name is caught.

## Narration (render-time; not a build prerequisite)

**Proof:** `narration/narration_compare02.py` (`fb45d672…`) wrote `narration-01-08-proof02.json` (`a31b9bbe…`).
- It covers the 11 cues 01_1 to 01_6 and 08_1 to 08_5.
- It compares WAVs, report rows, assembly outputs and units, plan units, and the nested `pt["cues"]` phrase timings: 498 marks.
- Cues 01_4, 01_6, 08_1 and 08_4 have no marks in any folder.
- All of these agree across CSS css309 and CSD 849, 269, 313 and 19.
- phrase-timings.json is byte-identical in all five folders.
- The 12 stem files for 01/08/09 are equal across the two hosts.
- Six mutated controls are all refused.

**The superseded comparator** is `narration_compare01.py` (`7b42be8f…`), kept with its output (`df007b25…`). Its phrase check was vacuous, as root found.

**Before render:** a delivery rebind like 849's needs seven new map rows (`narration/map-holds-combined01.json`). No WAV or timing change is involved:

| Row | 760 file and line | Combined file and line |
|---|---|---|
| 01_4 | the_bump 245 | `90b51c2b` 245 |
| 08_5 | loop_sums 1186 | `a8d62f56` 1284 |
| 09_4 | boundary 985 | `4b689791` 1048 |
| 09_5 | boundary 1050 | `4b689791` 1113 |
| 09_8 | boundary 1329 | `4b689791` 1394 |
| mod.rs helper | line 31 | `4de1e4a5` 31 |
| mod.rs helper | line 58 | `4de1e4a5` 58 |
