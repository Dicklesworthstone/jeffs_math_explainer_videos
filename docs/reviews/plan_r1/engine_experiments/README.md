# Engine-feasibility experiments (plan review round 1)

These are the sources of the experiments behind `../engine_feasibility.md` (findings
F-01 to F-05, F-11 and F-22), run on 2026-10-09 against franken_manim `6043e68c`
with cargo 1.100.0-nightly (`e8cb624d5`) from the pinned toolchain
`nightly-2026-08-31`. Nothing here is built by the workspace. Every script writes only
to a temporary directory or to stdout, and none of them deletes anything.

Placeholders used below:
- `<franken_manim>`: a checkout of github.com/Dicklesworthstone/franken_manim.
- `<franken_markdown>`: a checkout of github.com/Dicklesworthstone/franken_markdown at
  the rev franken_manim pins (`Cargo.toml`, `franken_markdown = { … rev = … }`).
- `<real-cargo>`: the toolchain's real cargo binary. If `cargo` on your PATH is a
  wrapper, for example the RCH shim, which refuses builds outside its canonical project
  root, use the binary inside `$(rustup toolchain list -v | grep nightly-2026-08-31)`'s
  `bin/` directory instead.

## E1 + E6: git dependency on a franken_manim-shaped workspace, and the `--config` patch override

`e1_cargo_git_patch/run.sh` generates two tiny workspaces in a fresh temporary
directory and then runs the checks. The workspaces are:
- an "upstream" local git repo shaped like franken_manim:
  - `[workspace.package]` inheritance and path-based `[workspace.dependencies]`;
  - `default-features = false` and an `ffmpeg` feature forward;
  - a member with an `artifact = "bin:fmn"` dev-dependency;
  - a malformed nested `Cargo.toml` like asupersync's test fixture;
- a dirty clone of upstream, standing in for the sibling checkout;
- a "downstream" workspace that depends on `fmn = { git = "file://…", rev = … }`.

```bash
CARGO=<real-cargo> ./e1_cargo_git_patch/run.sh        # TOOLCHAIN=… to override the nightly
```

It prints `PASS`/`FAIL` per check and exits non-zero if any check fails. Expected
results (13 PASS on 2026-10-09):

| test | what it shows |
|---|---|
| T1 | A git dependency with workspace inheritance builds, and the feature forwards. The malformed nested manifest prints `error: unclosed table` and is then skipped, which is not a failure. The artifact dev-dep member needs no `-Zbindeps` downstream. |
| T2 | `--config 'patch."<git-url>".fmn.path="<path>"'` overrides `fmn` **and** its internal path dependencies, but **rewrites the committed `Cargo.lock`**. |
| T3 | A relative patch path given with `--config` is resolved from the **current directory**, so it fails from a subdirectory. |
| T4 | `--locked` together with the patch is refused ("cannot update the lock file"). |
| T5 | `--config resolver.lockfile-path="<abs>/.x/fmn-local/Cargo.lock"` builds against the local tree and leaves the committed lock untouched. The file must be named `Cargo.lock`. |
| T6 | A `--locked` git-pin build still works afterwards. |
| T8 | A relative `[patch]` path inside `<root>/.cargo/config.toml` resolves against the workspace root from any directory. |
| N1 | A `members` glob that covers a crate with its own `[workspace]` fails with "multiple workspace roots found". `exclude` fixes it. |
| E6 | `-Zgit=shallow-deps` alone falls back to a full fetch on the local transport ("shallow fetch is not supported by the local transport"). With `-Zgitoxide=fetch` added, the git db is shallow. This was **not tested over https**. |

The script leaves its temporary directory in place, including its private `CARGO_HOME`
and `CARGO_TARGET_DIR`, and prints the path so you can inspect or delete it.

## E2: what a fresh clone of the dependency closure costs (commands only)

```bash
# GitHub's packed repository sizes (KB) for franken_manim and the FrankenSuite repos fmn pulls
for r in franken_manim asupersync franken_markdown franken_numpy frankenscipy franken_networkx frankenpandas; do
  printf '%-18s ' "$r"; gh api "repos/Dicklesworthstone/$r" --jq .size
done
# On a machine that has built fmn: local git dbs and checkouts
du -sh "${CARGO_HOME:-$HOME/.cargo}"/git/db/* "${CARGO_HOME:-$HOME/.cargo}"/git/checkouts/*/*
# The size of franken_manim's own tree at a rev
git -C <franken_manim> ls-tree -r -l <rev> | awk '{s+=$4} END {printf "%.1f MiB\n", s/1048576}'
```

On 2026-10-09 GitHub reported about 37.7 MB for franken_manim and about 1.7 GB for the
FrankenSuite repos in fmn's closure. The local checkouts took about 8 GB, including
4.5 GB of frankenpandas test fixtures. frankentorch is not in fmn's graph.

## E3: glyph coverage of the bundled faces

`e3_cmap_probe/cmap_probe.py` is a std-library TrueType cmap reader. It lists which
bundled faces map each code point the videos plan to show.

```bash
F=<franken_markdown>/fmd-font/fonts
python3 -I e3_cmap_probe/cmap_probe.py \
  "$F"/computer-modern/cmunrm.ttf "$F"/computer-modern/cmunti.ttf "$F"/computer-modern/cmuntt.ttf \
  "$F"/ibm-plex-sans/IBMPlexSans-Regular.ttf "$F"/noto-sans-math/NotoSansMathSymbols.ttf
```

Results on 2026-10-09:
- ⊕ ≺ ⌊ ⌋ ⌉ ≡ ⇔ ⋅ ∈ are mapped only by NotoSansMathSymbols, so they render in its
  heavier sans style.
- α, θ, ő and – are in CMU and IBM Plex.
- ✗ and ⪯ are in no bundled face.

This complements a glyph probe sheet; it does not replace one. A mapped code point
says nothing about shape (CMU's θ is outline-identical to ϑ), and Tex chooses faces
through fmd-math's chains.

## E4: crates.io version drift against franken_manim's lock

`e4_lock_drift/lockdiff.py` lists packages whose resolved versions differ between our
`Cargo.lock` and franken_manim's `Cargo.lock` at the pin. It exits 1 on any drift.

```bash
git -C <franken_manim> show <pin>:Cargo.lock > "${TMPDIR:-/tmp}/franken_manim.lock"
python3 -I e4_lock_drift/lockdiff.py explainer_corpus/hoeffding_d/Cargo.lock "${TMPDIR:-/tmp}/franken_manim.lock"
```

On 2026-10-09, comparing Hoeffding's lock with franken_manim's at `e2eec632` found
261 common packages and 0 version mismatches.

## E5: the level reference behind F-11 (no script needed)

`docs/research/sound_design/scripts/current.py` measures the old pad with its −9 dB mix
gain applied (`pad(...) * g(-9)`). `nar.py` measures the narration WAVs without the
−9.4 dB mix gain that the kit applies in `add_sound`. Read the two scripts side by side.
