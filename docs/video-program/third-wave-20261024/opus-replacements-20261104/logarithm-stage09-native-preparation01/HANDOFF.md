# Logarithm stage09 native preparation (root's Mail 3882, bead 87do.63)

RubyRaven, 2026-11-05. Root reviews and runs every step. This bundle holds a helper and its checks.
I did not compile, render, run cargo or write into W, EL, canonical source, Beads or the TODO.

## What it prepares

`prepare_stage09_native.py` installs root's integrated stage09 candidate as a crate beside root's W
engine. It then writes one resource-guarded job: the native tests, the binary and strict Clippy. The
shared binary is preserved before the job and the new one is copied out after it. It has four
subcommands, each run from the repository root:

| Step | Writes | Refuses when |
|---|---|---|
| `check --w W` | nothing | any pin below fails |
| `install --w W --into D` | `D/logarithm-stage09/` (48 files), `D/logarithm-stage09-original-Cargo.{toml,lock}`, `D/logarithm-stage09-source-binding01.json` | any output exists; D is inside the repository, W/engine, W/repo-view, W/logarithm-stage08 or EL |
| `script --w W --into D --attempt NN` | `D/logarithm-stage09-buildNN.sh`, `D/logarithm-stage09-buildNN-inputs.json` | the installed stage moved; either file, a preserved copy or the guard's receipt folder for that attempt exists |
| `verify --w W --into D --attempt NN` | nothing | the guard's receipt or log falls short (see below) |

## Which engine, and why

The helper uses **root's W stage**, through `W/engine-budget.toml` (`cdd28268`). This is the same
engine root's `logarithm-stage08-build01` used (exit 0, 02:39:03–02:39:44Z): canonical franken_manim
`a399d626` plus root's three geometry patches (`W/source-stage01.json`).

- **The wording change is the only variable.** `check` compares W/engine (4,057 files) and
  W/repo-view (68 files) file by file against the guard rows of the stage08 build
  (`W/logarithm-stage08-build01-inputs.json`, `f730d69d`). They are unchanged today. The budget must
  send all 19 franken_manim crates to `W/engine/crates/<name>`, and to nothing else.
- **No mixed stacks, no promotion.** The six-entry secondary-validation stack passed every guarded job,
  but root has not promoted it. Of the 12 files that stack touches, `check` requires the unpatched
  bytes of the 10 it modifies and the absence of the 2 it adds. The exact stage08 walk also refuses
  any other added file. A build on that stack would be a separate job after root promotes it. The
  canonical pin is untouched.

## Bound inputs

| Input | sha256 | Bound by |
|---|---|---|
| White's sealed bundle `logarithm-clarity-integration01/SHA256SUMS` | `5fcb5ea6…` | its 58 `stage09/` rows equal EL `combined-repair-stage09/candidate` (48) and `bound-plan` (10) byte for byte, with no file extra or missing |
| Root's `root-integration-receipt01.json` | `e16430ec…` | seal `5fcb5ea6`, script `559afa36`, changed cues exactly 04_4 and 08_5 (78 unchanged), 58 installed, replay and install-verify exit 0, voice ElevenLabs Oliver |
| `script.tsv` | `559afa3698222952036ece90a25eab926d45e201fc916bf32412e9485a52170a` | 80 cues; 04_4 text `7e4e9612…` with "which is why base e is called natural."; 08_5 text `8e26bcb0…` with "Taking half of the lost lap"; neither stage08 phrase anywhere |
| `narration_units.json` | `1132a5a4…` | script `559afa36`; direction map `docs/narration-direction-map-oliver16.json` `14e87893…`; 80 units; the 04_4 and 08_5 units carry the script's stage09 text; no mention of oliver15 |
| `src/chapters/product_rule.rs` | `bc9fbe1d…` | its test searches for "Taking half of the lost lap", so the native tests bind the new 08_5 words |
| candidate `Cargo.toml` / `Cargo.lock` | `a8c95209…` / `50cd8811…` | equal to root's stage08 originals |

**Refused stage08 bytes.** Script `b89eab60`, units `0e27709b` and `product_rule.rs` `f2c8684a` are
refused. The oliver15 map (`c94f0077`) stays in the crate, because it is part of the sealed
candidate, but nothing binds it. Against the 47 source hashes in root's stage08 binding (`eb71f333`),
stage09 changes exactly three files (`script.tsv`, `narration_units.json`, `product_rule.rs`) and
adds one (the oliver16 map). Any other difference is refused.

**Inherited stray file.** `src/chapters/doing_nothing.rs.orig` (`c526e62d`) comes from stage08 and
is copied as sealed. No module names it.

**Cargo files.** The only two changes are root's stage08 rule:
- `explainer_kit` and `explainer_score` point at `W/repo-view/crates/…`;
- the lock's 19 franken_manim packages lose their git `source` line.

Applied to the stage09 originals, this rule gives `dc41f239…` and `e0b265e8…`. Those are root's
`W/logarithm-stage08` bytes exactly, which `check` and `install` assert. Root's
`verify_patched_lock.py` passes the installed lock.

## Root's run

```bash
H=docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-native-preparation01
W=…   # root's geometry stage: the folder holding engine-budget.toml cdd28268 and logarithm-stage08/
python3 -I -B $H/prepare_stage09_native.py check   --w "$W"
python3 -I -B $H/prepare_stage09_native.py install --w "$W" --into "$W"     # W/logarithm-stage09, beside stage08
python3 -I -B $H/prepare_stage09_native.py script  --w "$W" --into "$W" --attempt 01
# then the printed "launch_from_repo_root" line:
python3 -I -B docs/video-program/third-wave-20261024/opus-replacements-20261104/engine-secondary-validation01/guard_owner56g_target20g.py \
  --phase native --job logarithm-stage09-build01 --inputs "$W/logarithm-stage09-build01-inputs.json" -- sh "$W/logarithm-stage09-build01.sh"
python3 -I -B $H/prepare_stage09_native.py verify  --w "$W" --into "$W" --attempt 01
```

`--into "$W"` mirrors stage08's layout. Any other folder outside the repository works the same way.
The install and the job write absolute scratch paths, which is why D may not be inside the repository.

### The job (attempt 01, as generated)

The three `cargo` lines are root's stage08 lines verbatim; `$M` is now `$D/logarithm-stage09/Cargo.toml`.

```sh
set -eu
refuse() { echo "REFUSED: $1" >&2; exit 1; }
sha() { shasum -a 256 "$1" | cut -d ' ' -f 1; }
W=…  D=…  R=…  T=…/target
[ "${CARGO_TARGET_DIR:-}" = "$T" ] || refuse "CARGO_TARGET_DIR is not the guard's target"
export CARGO_INCREMENTAL=0
export RCH_SHIM_LOCAL_IDE=1
export CARGO_TARGET_DIR="$CARGO_TARGET_DIR/geometry-kit01"
cd "$R"
M="$D/logarithm-stage09/Cargo.toml"
B="$CARGO_TARGET_DIR/debug/logarithm_mysteries"
BEFORE="$D/before-logarithm-stage09-build01-binary"; AFTER="$D/logarithm-stage09-build01-binary"
SUMS="$D/logarithm-stage09-build01-binaries.sha256"
for f in "$BEFORE" "$AFTER" "$SUMS"; do [ ! -e "$f" ] || refuse "$f exists"; done
[ -f "$B" ] && [ "$(sha "$B")" = e9233401… ] || refuse "the shared binary is not the one this script preserves"
cp -p "$B" "$BEFORE"
[ "$(sha "$BEFORE")" = e9233401… ] || refuse "the preserved copy differs"
python3 -I -B "$R/…/verify_patched_lock.py" "$D/logarithm-stage09-original-Cargo.lock" "$D/logarithm-stage09/Cargo.lock"
cargo test --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets
cargo build --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --bin logarithm_mysteries
cp -p "$B" "$AFTER"
cargo clippy --locked --offline --config "$W/engine-budget.toml" --manifest-path "$M" --all-targets --no-deps -- -D warnings
cmp "$B" "$AFTER" || refuse "Clippy changed the shared binary"
(cd "$D" && shasum -a 256 before-logarithm-stage09-build01-binary logarithm-stage09-build01-binary) > "$SUMS"
"$AFTER" chapters
```

### Preservation

The shared path `target/geometry-kit01/debug/logarithm_mysteries` holds one binary, and each binary
is tied to the crate copy it was built from: the kit compares `topic/script.tsv` with the compiled
script. Today that path holds `e9233401…`, which is stage08's binary (linked 02:39:28Z, inside root's
stage08 build). The script records whichever hash it sees when written. It then:

- copies that binary to `before-…-binary` and refuses before any cargo runs if the binary has changed
  since;
- copies the new binary to `logarithm-stage09-build01-binary` right after the build, so a Clippy
  failure keeps it;
- checks that Clippy left it unchanged;
- binds both copies in the sums file.

After the job, the shared path holds the stage09 binary, as it held stage08's after root's build.
`W/logarithm-stage08`, its binding and its original Cargo files are guard inputs, so the job cannot
change them.

### Guard inputs (4,298 rows at attempt 01)

The job script (first row), and the installed stage with its originals and binding (52). W/engine
(4,057), W/repo-view (68), W/logarithm-stage08 (47) and W's eight pinned files. The EL stage09
candidate (48), bound plan (10) and root's receipt. White's seal, this helper, root's
`verify_patched_lock.py`, the 56g/20g guard wrapper and its source. Two film binaries,
`complex_analysis_visual` and `examples/plate_lab`, are pinned only to show that this job relinks
neither. If root relinks one between `script` and launch, the guard refuses; write a new attempt.
The shared logarithm binary is not a row, because the job replaces it; the script guards it instead.

### What `verify` requires

- The guard receipt shows phase native, admitted, and this script as its command. It shows exit 0,
  no resource stop and no error. Its start and final input hashes equal this attempt's inputs file.
- The command log shows root's lock check PASS and exactly one test result,
  `ok. 241 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`. 241 is the number of `#[test]`
  functions in the stage09 sources, and stage08's build ran 241. The log ends with the 13 chapter keys.
- The sums file matches both copies.
- The installed stage still matches its binding.

Clippy's `-D warnings` is enforced by the script's exit status. `verify` writes nothing; it prints
the hashes for root's record.

## Checks

```bash
python3 -I -B $H/checks/run_checks.py FRESH_SCRATCH --w "$W" > out.txt     # 58 PASS, 0 FAIL, about 90 s
```

`checks/run_checks.out` is the transcript. Paths are shortened to SCRATCH, W, REPO and ~.

- **R1–R4.** `check` passes on the real inputs; the derived Cargo files are root's stage08 bytes;
  stage09 differs from stage08 only in the reviewed files; the receipt's open gates are carried.
- **I1–I6.** Install into scratch: 48 files, the binding, root's lock check passes. A second install
  and an install inside the repository are refused and write nothing.
- **J1–J8.** The script is written on the real target. Its cargo lines are root's verbatim, and it
  preserves `e9233401` (stage08's binary). The inputs are absolute, unique and current, with the shared
  binary excluded. The launch line is right; a repeated or malformed attempt is refused.
- **H1–H9.** The generated script is run with a **stub cargo** first on PATH: it logs its argv and
  copies a stand-in binary. The real cargo is never reached, and nothing compiles.
  - The exact test, build and Clippy argv.
  - The copy made before cargo, the copy made after the build, the sums file, and chapters run from
    the copy.
  - Refusal without the guard's `CARGO_TARGET_DIR`, after a relink, and when a preserved copy exists.
  - A Clippy failure keeps both copies.
  - A used receipt number is refused.
- **V1–V6.** `verify` passes the stub run's receipt. It refuses 240 tests, exit 101, foreign input
  hashes, a short chapters listing and a failed lock check.
- **F0–F13.** Source forgeries in scratch copies. White's seal and root's receipt are rewritten to
  match each copy, and the pins are moved with it, so only the guard under test can refuse. F0, the
  unmutated forgery, passes. The forgeries:
  - the stage08 script, units or `product_rule.rs`;
  - 04_4 reverted;
  - the old 08_5 words appended as a comment;
  - units bound to oliver15, by path or by bytes;
  - a stray `.ee/hook-state`;
  - an unsealed bound-plan byte;
  - a changed lock;
  - a receipt with other changed cues;
  - a tampered seal;
  - a neighbour chapter changed under a valid seal, which is refused against stage08's sources.
- **E0–E9.** Copies of W, given their own pins. E0, the unmutated copy, passes. The rest:
  - the stack's `frame_table_limits.rs` added;
  - a patched `rendering/pipeline.rs`;
  - an engine file changed, or one added;
  - a budget that sends `fmn-render` elsewhere;
  - a changed `W/logarithm-stage08` or repo-view file;
  - an unpinned `Cargo.lock.pinned`;
  - `--into` inside W/engine.
- **U1.** The real EL stage09, White's seal, W's engine, repo-view, stage08 and pinned files, and the
  shared logarithm binary are byte-identical after the run, and no stage09 file appeared in W.

The first scratch run failed H1, H4 and V1. My stub's `cp` onto the planted file kept that file's
644 mode, so the copy could not run. Cargo links a new executable, so the helper was not at fault.
The stub now runs `chmod 755` on what it builds, and runs 02 and 03 passed 58/58.

## Not done, not claimed

- Nothing was compiled. The stub harness tests the script's shell logic only; it is not native
  evidence. `verify` has been exercised only on stub receipts.
- There is no mutation check of `run_checks.py`. Each negative control instead requires its own
  guard's message.
- I took no resource measurement for this job. Stage08's build took 41 s on warm dependencies.
  Stage09 sits at a new path, so the crate itself compiles again; the dependencies should be reused.

## Gates still open after this job (root's)

1. **Native build, tests and strict Clippy**: this job, then `verify`.
2. **Glyph probe**: optional. Stage08 ran `logarithm-stage08-glyphs01`, a preview run of the
   binary's `glyphs`. Stage09 changes no on-screen source, only a comment and a test string in
   `product_rule.rs`, so whether to rerun it is root's call.
3. **Oliver takes for 04_4 and 08_5**: root selects them, and WhiteSwan's
   `logarithm-stage09-audio-preparation01` decodes, prepares and assembles them and writes the
   timings (`phrase-timings.json` into the narration folder). Fresh timings for the changed takes
   include the two 08_5 phrase anchors. No stage08 selection, map or timing applies.
4. **Narrated render with the stage09 binary**, from `D/logarithm-stage09` (the binary's compiled
   project folder), followed by scene review. Chapters 04 and 08 carry the change; the whole film
   uses the new binary.
5. **Listening**: 04_4 and 08_5 and their joins, including how "e" is spoken in 04_4.
6. **Fine sync, then the final master's verify and independent final AV acceptance.** Logarithm stays
   Oliver throughout, with no mixed voice.
