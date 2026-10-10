#!/usr/bin/env bash
# E1 + E6 of the engine-feasibility review (plan review round 1, 2026-10-09).
#
# Builds two tiny Cargo workspaces in a fresh temporary directory:
#   upstream/            a local git repo shaped like franken_manim (workspace.package
#                        inheritance, path-based workspace.dependencies, a feature
#                        forward, a member with an `artifact = "bin:..."` dev-dep and a
#                        malformed nested Cargo.toml such as asupersync's test fixture)
#   franken_manim_wt/    a clone of upstream with uncommitted edits (the "sibling checkout")
#   downstream/          our workspace: fmn = { git = "file://.../upstream", rev = ... }
# and checks how the pinned cargo resolves the git dependency and a `--config`
# [patch] override of it (tests T1-T8, N1) and whether shallow git fetch works (E6).
#
# It writes only inside the temporary directory it prints (including CARGO_HOME and
# CARGO_TARGET_DIR) and never deletes anything; remove that directory yourself.
#
# Usage:  CARGO=<path to the real cargo binary> ./run.sh
# Needs:  git, the nightly toolchain named by TOOLCHAIN (default nightly-2026-08-31).
# If `cargo` on PATH is a wrapper (for example the RCH shim, which refuses builds
# outside its canonical project root), point CARGO at the toolchain's real binary.
set -u

CARGO="${CARGO:-cargo}"
TOOLCHAIN="${TOOLCHAIN:-nightly-2026-08-31}"
export RUSTUP_TOOLCHAIN="$TOOLCHAIN"

W="$(mktemp -d "${TMPDIR:-/tmp}/fmn-e1.XXXXXX")"
W="$(cd "$W" && pwd -P)"
export CARGO_HOME="$W/cargo_home"
export CARGO_TARGET_DIR="$W/target"
mkdir -p "$CARGO_HOME"
echo "work dir: $W (left in place; delete it yourself when done)"
"$CARGO" --version

pass=0
fail=0
# check NAME EXPECTED-SUBSTRING OUTPUT
check() {
  if printf '%s' "$3" | grep -qF -- "$2"; then
    echo "PASS $1"
    pass=$((pass + 1))
  else
    echo "FAIL $1 (expected output containing: $2)"
    printf '%s\n' "$3" | tail -6 | sed 's/^/    /'
    fail=$((fail + 1))
  fi
}
# Hide the malformed-fixture noise that cargo prints (and then ignores).
quiet() { grep -v -e '^ *|' -e '^ *-->' -e '^1 |' -e 'unclosed table' || true; }

# ---------------------------------------------------------------- upstream
U="$W/upstream"
mkdir -p "$U"/crates/{fmn-core,fmn,fmn-cli,fmn-conformance}/src "$U/fixtures/malformed" "$U/.cargo"
cat > "$U/Cargo.toml" <<'EOF'
[workspace]
resolver = "3"
members = ["crates/fmn-core", "crates/fmn", "crates/fmn-cli", "crates/fmn-conformance"]

[workspace.package]
version = "0.5.0"
edition = "2024"

[workspace.dependencies]
fmn-core = { path = "crates/fmn-core", default-features = false }
fmn = { path = "crates/fmn" }
fmn-cli = { path = "crates/fmn-cli" }

[profile.release]
lto = "thin"
EOF
cat > "$U/.cargo/config.toml" <<'EOF'
# Mirrors franken_manim: never read when the workspace is consumed as a git dependency.
[unstable]
bindeps = true
EOF
cat > "$U/crates/fmn-core/Cargo.toml" <<'EOF'
[package]
name = "fmn-core"
version.workspace = true
edition.workspace = true

[features]
default = ["exact-process"]
exact-process = []
EOF
cat > "$U/crates/fmn-core/src/lib.rs" <<'EOF'
pub const WHO: &str = "core@git";
pub fn process() -> bool { cfg!(feature = "exact-process") }
EOF
cat > "$U/crates/fmn/Cargo.toml" <<'EOF'
[package]
name = "fmn"
version.workspace = true
edition.workspace = true

[dependencies]
fmn-core.workspace = true

[features]
ffmpeg = ["fmn-core/exact-process"]
EOF
cat > "$U/crates/fmn/src/lib.rs" <<'EOF'
pub fn who() -> String { format!("fmn@git + {} (process={})", fmn_core::WHO, fmn_core::process()) }
EOF
cat > "$U/crates/fmn-cli/Cargo.toml" <<'EOF'
[package]
name = "fmn-cli"
version.workspace = true
edition.workspace = true

[[bin]]
name = "fmn"
path = "src/main.rs"
EOF
echo 'fn main() {}' > "$U/crates/fmn-cli/src/main.rs"
echo 'fn main() { println!("cargo:rerun-if-changed=build.rs"); }' > "$U/crates/fmn-cli/build.rs"
cat > "$U/crates/fmn-conformance/Cargo.toml" <<'EOF'
[package]
name = "fmn-conformance"
version.workspace = true
edition.workspace = true

[dev-dependencies]
fmn-cli = { path = "../fmn-cli", artifact = "bin:fmn", lib = false }
EOF
: > "$U/crates/fmn-conformance/src/lib.rs"
printf '[package\nname = "broken"\n' > "$U/fixtures/malformed/Cargo.toml"
git -C "$U" init -q
git -C "$U" add -A
git -C "$U" -c user.email=e1@example.invalid -c user.name=e1 commit -qm init
REV="$(git -C "$U" rev-parse HEAD)"
echo "upstream rev: $REV"

# --------------------------------------------- the dirty "sibling checkout"
git clone -q "$U" "$W/franken_manim_wt"
echo 'pub const WHO: &str = "core@LOCAL-DIRTY";
pub fn process() -> bool { cfg!(feature = "exact-process") }' > "$W/franken_manim_wt/crates/fmn-core/src/lib.rs"
echo 'pub fn who() -> String { format!("fmn@LOCAL-DIRTY + {} (process={})", fmn_core::WHO, fmn_core::process()) }' \
  > "$W/franken_manim_wt/crates/fmn/src/lib.rs"

# -------------------------------------------------------------- downstream
D="$W/downstream"
mkdir -p "$D/crates/explainer_kit/src" "$D/explainer_corpus/demo/src"
cat > "$D/Cargo.toml" <<EOF
[workspace]
resolver = "3"
members = ["crates/*", "explainer_corpus/*"]

[workspace.dependencies]
fmn = { git = "file://$U", rev = "$REV", features = ["ffmpeg"] }
explainer_kit = { path = "crates/explainer_kit" }
EOF
cat > "$D/crates/explainer_kit/Cargo.toml" <<'EOF'
[package]
name = "explainer_kit"
version = "0.1.0"
edition = "2024"

[dependencies]
fmn.workspace = true
EOF
echo 'pub fn who() -> String { fmn::who() }' > "$D/crates/explainer_kit/src/lib.rs"
cat > "$D/explainer_corpus/demo/Cargo.toml" <<'EOF'
[package]
name = "demo"
version = "0.1.0"
edition = "2024"

[dependencies]
fmn.workspace = true
explainer_kit.workspace = true
EOF
echo 'fn main() { println!("{}", explainer_kit::who()); }' > "$D/explainer_corpus/demo/src/main.rs"

PATCH_REL="patch.\"file://$U\".fmn.path=\"../franken_manim_wt/crates/fmn\""
PATCH_ABS="patch.\"file://$U\".fmn.path=\"$W/franken_manim_wt/crates/fmn\""
LOCAL_LOCK="resolver.lockfile-path=\"$D/.x/fmn-local/Cargo.lock\""
mkdir -p "$D/.x/fmn-local"

# T1: plain git dependency (workspace inheritance, feature forward, skipped manifests)
out="$(cd "$D" && "$CARGO" run -q -p demo 2>&1)"
check "T1 git dep builds and runs" "fmn@git + core@git (process=true)" "$out"
check "T1 malformed nested manifest is reported, then skipped" "unclosed table" "$out"
cp "$D/Cargo.lock" "$W/lock.git"

# T2: --config [patch] with a relative path, from the workspace root
out="$(cd "$D" && "$CARGO" --config "$PATCH_REL" run -q -p demo 2>&1 | quiet)"
check "T2 patch overrides fmn and its internal path deps" "fmn@LOCAL-DIRTY + core@LOCAL-DIRTY" "$out"
if cmp -s "$W/lock.git" "$D/Cargo.lock"; then
  echo "FAIL T2 expected the patch to rewrite Cargo.lock"; fail=$((fail + 1))
else
  echo "PASS T2 the patch rewrites the committed Cargo.lock"; pass=$((pass + 1))
fi
cp "$W/lock.git" "$D/Cargo.lock"

# T3: the same relative path from a subdirectory resolves against cwd
out="$(cd "$D/explainer_corpus/demo" && "$CARGO" --config "$PATCH_REL" run -q -p demo 2>&1 | quiet)"
check "T3 relative --config patch path is resolved from cwd" "does not contain packages matching" "$out"

# T4: --locked refuses the patch
out="$(cd "$D" && "$CARGO" --config "$PATCH_ABS" build -q --locked -p demo 2>&1 | quiet)"
check "T4 --locked plus patch is refused" "cannot update the lock file" "$out"

# T5: a separate lockfile keeps the committed one untouched
out="$(cd "$D" && "$CARGO" --config "$PATCH_ABS" --config "$LOCAL_LOCK" run -q -p demo 2>&1 | quiet)"
check "T5 resolver.lockfile-path build uses the local tree" "fmn@LOCAL-DIRTY" "$out"
if cmp -s "$W/lock.git" "$D/Cargo.lock"; then
  echo "PASS T5 committed Cargo.lock untouched"; pass=$((pass + 1))
else
  echo "FAIL T5 committed Cargo.lock changed"; fail=$((fail + 1))
fi
out="$(cd "$D" && "$CARGO" --config "$PATCH_ABS" --config "resolver.lockfile-path=\"$D/.x/local.lock\"" metadata --format-version 1 2>&1 | quiet)"
check "T5 lockfile-path must name a Cargo.lock file" "must be a path to a Cargo.lock file" "$out"

# T6: back to the git pin with --locked
out="$(cd "$D" && "$CARGO" run -q --locked -p demo 2>&1 | quiet)"
check "T6 --locked git build after T5" "fmn@git + core@git" "$out"

# T8: a relative [patch] in <root>/.cargo/config.toml resolves against the root
cp -R "$D" "$W/downstream_cfg"
mkdir -p "$W/downstream_cfg/.cargo"
printf '[patch."file://%s"]\nfmn = { path = "../franken_manim_wt/crates/fmn" }\n' "$U" > "$W/downstream_cfg/.cargo/config.toml"
out="$(cd "$W/downstream_cfg/explainer_corpus/demo" && "$CARGO" run -q -p demo \
  --config "resolver.lockfile-path=\"$W/downstream_cfg/.x/fmn-local/Cargo.lock\"" 2>&1 | quiet)"
check "T8 config-file patch path is root-relative from any cwd" "fmn@LOCAL-DIRTY" "$out"

# N1: a member glob covering a crate with its own [workspace]
N="$W/downstream_nested"
cp -R "$D" "$N"
mkdir -p "$N/explainer_corpus/legacy/src"
printf '[package]\nname = "legacy"\nversion = "0.1.0"\nedition = "2024"\n\n[workspace]\n' > "$N/explainer_corpus/legacy/Cargo.toml"
echo 'fn main() {}' > "$N/explainer_corpus/legacy/src/main.rs"
out="$(cd "$N" && "$CARGO" metadata --format-version 1 --no-deps 2>&1 | quiet)"
check "N1 nested workspace root inside the glob is an error" "multiple workspace roots found" "$out"
cat > "$N/Cargo.toml" <<EOF
[workspace]
resolver = "3"
members = ["crates/*", "explainer_corpus/*"]
exclude = ["explainer_corpus/legacy"]

[workspace.dependencies]
fmn = { git = "file://$U", rev = "$REV", features = ["ffmpeg"] }
explainer_kit = { path = "crates/explainer_kit" }
EOF
out="$(cd "$N" && "$CARGO" metadata --format-version 1 --no-deps 2>&1 | quiet)"
check "N1 exclude fixes it" '"name":"explainer_kit"' "$out"

# E6: shallow git dependency fetch (local transport; https was not tested)
for mode in "-Zgit=shallow-deps" "-Zgit=shallow-deps -Zgitoxide=fetch"; do
  home="$W/cargo_home_shallow_$(printf '%s' "$mode" | tr -c 'a-z' '_')"
  mkdir -p "$home"
  # shellcheck disable=SC2086 # $mode is deliberately word-split into flags
  (cd "$D" && CARGO_HOME="$home" "$CARGO" $mode metadata --format-version 1 2>&1 >/dev/null | quiet | grep -i 'shallow' | head -2)
  for db in "$home"/git/db/*; do
    [ -d "$db" ] && echo "E6 [$mode] $(basename "$db"): is-shallow=$(git -C "$db" rev-parse --is-shallow-repository)"
  done
done

echo
echo "passed: $pass  failed: $fail  (work dir: $W)"
[ "$fail" -eq 0 ]
