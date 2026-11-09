#!/usr/bin/env bash
# Hermetic integration checks for scripts/install.sh. No network is used.

set -euo pipefail
umask 022
cd "$(dirname "$0")/.." || exit 1

TMP_BASE="${TMPDIR:-/tmp}"
TMP_BASE="${TMP_BASE%/}"
TEST_ROOT=$(mktemp -d "$TMP_BASE/fmn-install-test.XXXXXXXX")
KEEP_TEST_ROOT="${FMN_INSTALL_KEEP_TEST_ROOT:-0}"

cleanup() {
    local status=$?
    trap - EXIT
    if [[ "$KEEP_TEST_ROOT" != "1" ]]; then
        case "$TEST_ROOT" in
            "$TMP_BASE"/fmn-install-test.*) rm -rf -- "$TEST_ROOT" ;;
        esac
    else
        printf 'installer test artifacts preserved at %s\n' "$TEST_ROOT"
    fi
    exit "$status"
}
trap cleanup EXIT

fail() {
    printf 'installer test failed: %s\n' "$*" >&2
    exit 1
}

sha256_file() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

expect_failure() {
    local label=$1
    shift
    if "$@" >"$TEST_ROOT/$label.stdout" 2>"$TEST_ROOT/$label.stderr"; then
        fail "$label unexpectedly succeeded"
    fi
}

bash -n scripts/install.sh
bash scripts/install.sh --help >"$TEST_ROOT/help.txt"
grep -q -- '--tier TIER' "$TEST_ROOT/help.txt" || fail "help omits tier mode"
grep -q -- '--offline ARCHIVE' "$TEST_ROOT/help.txt" || fail "help omits offline mode"
grep -q -- '--checksum HASH|FILE' "$TEST_ROOT/help.txt" || fail "help omits checksum mode"
grep -q -- '--features cli,batch' "$TEST_ROOT/help.txt" \
    || fail "source-build guidance can emit a batch-disabled fmn"

fixture="$TEST_ROOT/fixture"
mkdir -p "$fixture"
printf '%s\n' '#!/usr/bin/env bash' 'printf "fmn 9.8.7\\n"' >"$fixture/fmn"
chmod 0755 "$fixture/fmn"
archive="$TEST_ROOT/fmn-x86_64-unknown-linux-gnu.tar.xz"
tar -cJf "$archive" -C "$fixture" fmn
checksum=$(sha256_file "$archive")

success_dir="$TEST_ROOT/success/bin"
FMN_INSTALL_KEEP_STATE=1 bash scripts/install.sh --quiet --no-gum \
    --version 9.8.7 --install-dir "$success_dir" \
    --offline "$archive" --checksum "$checksum"
[[ -x "$success_dir/fmn" ]] || fail "offline install did not publish fmn"
[[ "$("$success_dir"/fmn --version)" == "fmn 9.8.7" ]] \
    || fail "installed binary reports the wrong version"

tier_auto_dir="$TEST_ROOT/tier-auto/bin"
FMN_INSTALL_KEEP_STATE=1 bash scripts/install.sh --quiet --no-gum \
    --tier auto --version 9.8.7 --install-dir "$tier_auto_dir" \
    --offline "$archive" --checksum "$checksum"
[[ -x "$tier_auto_dir/fmn" ]] || fail "--tier auto install did not publish fmn"

tier_portable_dir="$TEST_ROOT/tier-portable/bin"
FMN_INSTALL_KEEP_STATE=1 bash scripts/install.sh --quiet --no-gum \
    --tier portable --version 9.8.7 --install-dir "$tier_portable_dir" \
    --offline "$archive" --checksum "$checksum"
[[ -x "$tier_portable_dir/fmn" ]] || fail "--tier portable install did not publish fmn"

tier_v3_dir="$TEST_ROOT/tier-v3/bin"
FMN_INSTALL_KEEP_STATE=1 bash scripts/install.sh --quiet --no-gum \
    --tier x86-64-v3 --version 9.8.7 --install-dir "$tier_v3_dir" \
    --offline "$archive" --checksum "$checksum"
[[ -x "$tier_v3_dir/fmn" ]] || fail "--tier x86-64-v3 install did not publish fmn"

tier_invalid_dir="$TEST_ROOT/tier-invalid/bin"
expect_failure tier-invalid env FMN_INSTALL_KEEP_STATE=1 \
    bash scripts/install.sh --quiet --no-gum --tier invalid-tier \
    --version 9.8.7 --install-dir "$tier_invalid_dir" \
    --offline "$archive" --checksum "$checksum"
[[ ! -e "$tier_invalid_dir/fmn" ]] || fail "invalid tier published a binary"
grep -q 'unknown or unsupported SIMD tier' "$TEST_ROOT/tier-invalid.stderr" \
    || fail "invalid tier refusal was not precise"

tier_incompatible_dir="$TEST_ROOT/tier-incompatible/bin"
expect_failure tier-incompatible env FMN_INSTALL_KEEP_STATE=1 \
    bash scripts/install.sh --quiet --no-gum --tier aarch64-neon \
    --version 9.8.7 --install-dir "$tier_incompatible_dir" \
    --offline "$archive" --checksum "$checksum"
[[ ! -e "$tier_incompatible_dir/fmn" ]] || fail "incompatible tier published a binary"
grep -q 'is not supported for' "$TEST_ROOT/tier-incompatible.stderr" \
    || fail "incompatible tier refusal was not precise"

checksum_dir="$TEST_ROOT/checksum-failure/bin"
expect_failure checksum-mismatch env FMN_INSTALL_KEEP_STATE=1 \
    bash scripts/install.sh --quiet --no-gum --version 9.8.7 \
    --install-dir "$checksum_dir" --offline "$archive" \
    --checksum 0000000000000000000000000000000000000000000000000000000000000000
[[ ! -e "$checksum_dir/fmn" ]] || fail "checksum failure published a binary"
grep -q 'SHA-256 mismatch' "$TEST_ROOT/checksum-mismatch.stderr" \
    || fail "checksum failure was not precise"

bad_fixture="$TEST_ROOT/bad-fixture"
mkdir -p "$bad_fixture"
printf '%s\n' 'not a binary' >"$bad_fixture/not-fmn"
bad_archive="$TEST_ROOT/bad.tar.xz"
tar -cJf "$bad_archive" -C "$bad_fixture" not-fmn
bad_checksum=$(sha256_file "$bad_archive")
bad_dir="$TEST_ROOT/bad-archive/bin"
expect_failure bad-archive env FMN_INSTALL_KEEP_STATE=1 \
    bash scripts/install.sh --quiet --no-gum --version 9.8.7 \
    --install-dir "$bad_dir" --offline "$bad_archive" --checksum "$bad_checksum"
[[ ! -e "$bad_dir/fmn" ]] || fail "malformed archive published a binary"
grep -q 'exactly one top-level fmn binary' "$TEST_ROOT/bad-archive.stderr" \
    || fail "malformed archive refusal was not precise"

locked_dir="$TEST_ROOT/concurrent/bin"
mkdir -p "$locked_dir/.fmn-install.lock"
printf '%s\n' "$$" >"$locked_dir/.fmn-install.lock/pid"
expect_failure concurrent env FMN_INSTALL_KEEP_STATE=1 \
    bash scripts/install.sh --quiet --no-gum --version 9.8.7 \
    --install-dir "$locked_dir" --offline "$archive" --checksum "$checksum"
[[ ! -e "$locked_dir/fmn" ]] || fail "concurrent install published a binary"
grep -q 'another install may be active' "$TEST_ROOT/concurrent.stderr" \
    || fail "concurrent-install refusal was not precise"

# Online path, still hermetic: a stand-in curl on PATH serves fixtures by URL
# and logs every URL requested. Covers version discovery from compact and
# pretty-printed GitHub JSON, and the fallback when the API refuses (rate
# limiting): the fallback warning once leaked into `version=$(resolve_version)`
# and produced an unusable download URL.
online_cases=skipped
if [[ "$(uname -s)" == "Linux" && "$(uname -m)" == "x86_64" ]]; then
    fakebin="$TEST_ROOT/fakebin"
    mkdir -p "$fakebin"
    cat >"$fakebin/curl" <<'CURL'
#!/usr/bin/env bash
set -euo pipefail
url="" out="" prev=""
for arg in "$@"; do
    [[ "$prev" == "-o" ]] && out=$arg
    [[ "$arg" == https://* ]] && url=$arg
    prev=$arg
done
printf '%s\n' "$url" >>"$FAKE_CURL_LOG"
case "$url" in
    https://api.github.com/*)
        case "$FAKE_API_MODE" in
            fail) exit 22 ;;
            empty) printf '[]' ;;
            compact) printf '[{"url":"x","tag_name":"v%s","prerelease":true}]' "$FAKE_VERSION" ;;
            pretty) printf '[\n  {\n    "tag_name": "v%s",\n    "prerelease": true\n  }\n]\n' "$FAKE_VERSION" ;;
        esac
        ;;
    */SHA256SUMS) cp "$FAKE_RELEASE_DIR/SHA256SUMS" "$out" ;;
    */fmn-x86_64-unknown-linux-gnu.tar.xz) cp "$FAKE_RELEASE_DIR/fmn-x86_64-unknown-linux-gnu.tar.xz" "$out" ;;
    *) exit 22 ;;
esac
CURL
    chmod 0755 "$fakebin/curl"
    fallback_version=$(sed -n 's/^FALLBACK_VERSION="\(.*\)"$/\1/p' scripts/install.sh)
    [[ -n "$fallback_version" ]] || fail "installer fallback version not found"

    online_case() {
        local mode=$1 version=$2 case_root="$TEST_ROOT/online-$1" url
        mkdir -p "$case_root/release" "$case_root/fixture"
        printf '%s\n' '#!/usr/bin/env bash' "printf 'fmn $version\\n'" >"$case_root/fixture/fmn"
        chmod 0755 "$case_root/fixture/fmn"
        tar -cJf "$case_root/release/fmn-x86_64-unknown-linux-gnu.tar.xz" -C "$case_root/fixture" fmn
        printf '%s  %s\n' "$(sha256_file "$case_root/release/fmn-x86_64-unknown-linux-gnu.tar.xz")" \
            fmn-x86_64-unknown-linux-gnu.tar.xz >"$case_root/release/SHA256SUMS"
        : >"$case_root/urls.log"
        if ! env PATH="$fakebin:$PATH" FAKE_CURL_LOG="$case_root/urls.log" \
            FAKE_API_MODE="$mode" FAKE_VERSION="$version" FAKE_RELEASE_DIR="$case_root/release" \
            FMN_INSTALL_KEEP_STATE=1 bash scripts/install.sh --no-gum --tier portable \
            --install-dir "$case_root/bin" >"$case_root/stdout" 2>"$case_root/stderr"; then
            fail "online install ($mode) failed: $(tr '\n' ' ' <"$case_root/stderr")"
        fi
        [[ "$("$case_root/bin/fmn" --version)" == "fmn $version" ]] \
            || fail "online install ($mode) published the wrong version"
        while IFS= read -r url; do
            [[ "$url" =~ ^https://[^[:space:]]+$ ]] \
                || fail "online install ($mode) requested a malformed URL: $url"
        done <"$case_root/urls.log"
        grep -qx "https://github.com/Dicklesworthstone/franken_manim/releases/download/v$version/SHA256SUMS" \
            "$case_root/urls.log" || fail "online install ($mode) did not fetch v$version SHA256SUMS"
        if grep -q 'WARNING' "$case_root/stdout"; then
            fail "online install ($mode) wrote diagnostics to stdout"
        fi
    }

    online_case compact 9.8.7
    online_case pretty 9.8.7
    online_case fail "$fallback_version"
    online_case empty "$fallback_version"
    grep -q 'using installer fallback' "$TEST_ROOT/online-fail/stderr" \
        || fail "fallback warning was not reported on stderr"
    online_cases=passed
fi

printf 'installer smoke: success, tier selection, tier refusal, checksum refusal, archive refusal, lock refusal passed; online discovery/fallback %s\n' "$online_cases"
