#!/usr/bin/env bash
# Build and consume the actual fmn-wasm npm artifact. This is the W11 release
# gate: wasm-pack -> npm tarball/dry-run -> fresh consumer -> webpack -> Chrome.
set -euo pipefail

repo_root=$(cd "$(dirname "$0")/.." && pwd)
cd "$repo_root"

wasm_pack_bin=${FMN_WASM_PACK:-wasm-pack}
wasm_bindgen_bin=${FMN_WASM_BINDGEN:-wasm-bindgen}
wasm_opt_bin=${FMN_WASM_OPT:-wasm-opt}
webpack_bin=${FMN_WEBPACK:-webpack}
chrome_bin=${FMN_CHROME:-google-chrome}
http_user_agent=${FMN_HTTP_USER_AGENT:-fmn-wasm-package-gate}

for command_name in "$wasm_pack_bin" "$wasm_bindgen_bin" "$wasm_opt_bin" \
    "$webpack_bin" "$chrome_bin" npm python3 rustup git gzip; do
    if ! command -v "$command_name" >/dev/null 2>&1; then
        echo "ERROR: required WASM package-gate tool is unavailable: $command_name" >&2
        exit 1
    fi
done

if [[ -n "${FMN_NODE:-}" ]]; then
    node_bin=$FMN_NODE
elif node --version >/dev/null 2>&1; then
    node_bin=$(command -v node)
elif [[ -x /usr/bin/node ]] && /usr/bin/node --version >/dev/null 2>&1; then
    # This host intentionally puts Bun's limited `node` compatibility shim
    # first. npm itself needs a real Node runtime.
    node_bin=/usr/bin/node
else
    echo "ERROR: npm/webpack require a real Node.js runtime (set FMN_NODE)" >&2
    exit 1
fi
node_bin=$(command -v "$node_bin")
js_path="$(dirname "$node_bin"):$PATH"
node_path=${FMN_NODE_PATH:-${NODE_PATH:-}}
npm_bin=$(command -v npm)
webpack_bin=$(command -v "$webpack_bin")
node_version=$("$node_bin" --version)
npm_version=$(PATH="$js_path" "$npm_bin" --version)
chrome_version=$($chrome_bin --version)

wasm_pack_version=$($wasm_pack_bin --version)
if [[ "$wasm_pack_version" != "wasm-pack 0.15.0" ]]; then
    echo "ERROR: fmn-wasm release packaging requires wasm-pack 0.15.0; found: $wasm_pack_version" >&2
    exit 1
fi
wasm_bindgen_version=$($wasm_bindgen_bin --version)
if [[ "$wasm_bindgen_version" != "wasm-bindgen 0.2.127" ]]; then
    echo "ERROR: fmn-wasm threaded packaging requires wasm-bindgen 0.2.127; found: $wasm_bindgen_version" >&2
    exit 1
fi
wasm_opt_version=$($wasm_opt_bin --version)
if [[ "$wasm_opt_version" != "wasm-opt version 117 (version_117)" ]]; then
    echo "ERROR: fmn-wasm threaded packaging requires wasm-opt 117; found: $wasm_opt_version" >&2
    exit 1
fi
webpack_version=$(PATH="$js_path" NODE_PATH="$node_path" "$node_bin" -p 'require("webpack/package.json").version')
webpack_cli_version=$(PATH="$js_path" NODE_PATH="$node_path" "$node_bin" -p 'require("webpack-cli/package.json").version')
if [[ "$webpack_version" != "5.109.2" || "$webpack_cli_version" != "7.2.2" ]]; then
    echo "ERROR: browser consumer requires webpack 5.109.2 + webpack-cli 7.2.2; found $webpack_version + $webpack_cli_version" >&2
    exit 1
fi

workspace_version=$(python3 - <<'PY'
import tomllib
with open("Cargo.toml", "rb") as source:
    print(tomllib.load(source)["workspace"]["package"]["version"])
PY
)
source_commit=$(git rev-parse HEAD)
source_dirty=false
# An untracked Rust module can affect the artifact just as much as a tracked
# edit. Git-ignored compiler/package outputs remain outside this source check.
source_changes=$(git status --porcelain=v1 --untracked-files=all)
if [[ -n "$source_changes" ]]; then
    source_dirty=true
    if [[ "${FMN_WASM_PACKAGE_ALLOW_DIRTY:-0}" != "1" ]]; then
        echo "ERROR: source tree is dirty; commit all inputs or use FMN_WASM_PACKAGE_ALLOW_DIRTY=1 for a non-release diagnostic" >&2
        exit 1
    fi
fi

if [[ -n "${FMN_WASM_PACKAGE_ROOT:-}" ]]; then
    task_root=$FMN_WASM_PACKAGE_ROOT
    if [[ -e "$task_root" ]]; then
        echo "ERROR: FMN_WASM_PACKAGE_ROOT must name a new path: $task_root" >&2
        exit 1
    fi
    mkdir -p "$task_root"
else
    task_root=$(mktemp -d "${TMPDIR:-/tmp}/fmn-wasm-package.XXXXXX")
fi
package_dir=$task_root/package
consumer_dir=$task_root/consumer
threads_raw_dir=$task_root/threads-raw
threads_target_dir=$task_root/threads-target
mkdir -p "$consumer_dir"

echo "==> fmn-wasm package evidence root: $task_root"
echo "==> wasm-pack bundler artifact ($wasm_pack_version)"
# The RCH fleet does not carry wasm32-unknown-unknown. Resolve the pinned local
# toolchain directly so this required host artifact cannot silently fall back
# from, or be misreported as, a remote build.
real_cargo=$(rustup which cargo)
PATH="$(dirname "$real_cargo"):$PATH" \
    "$wasm_pack_bin" build --target bundler --release --out-dir "$package_dir" crates/fmn-wasm

echo "==> fmn-wasm shared-memory frame-pool artifact ($wasm_bindgen_version, $wasm_opt_version)"
threads_rustflags='-C target-feature=+atomics,+bulk-memory,+mutable-globals,+nontrapping-fptoint,+sign-ext,+reference-types,+multivalue -C link-arg=--shared-memory -C link-arg=--import-memory -C link-arg=--max-memory=4294967296 -C link-arg=--export=__heap_base -C link-arg=--export=__tls_base -C link-arg=--export=__tls_size -C link-arg=--export=__tls_align -C link-arg=--export=__wasm_init_tls'
CARGO_TARGET_DIR="$threads_target_dir" \
CARGO_TARGET_WASM32_UNKNOWN_UNKNOWN_RUSTFLAGS="$threads_rustflags" \
PATH="$(dirname "$real_cargo"):$PATH" \
    "$real_cargo" build --locked --target wasm32-unknown-unknown --release \
    -p fmn-wasm -Z build-std=std,panic_abort
mkdir -p "$threads_raw_dir" "$package_dir/threads"
"$wasm_bindgen_bin" --target web --out-dir "$threads_raw_dir" \
    --out-name fmn_wasm_threads \
    "$threads_target_dir/wasm32-unknown-unknown/release/fmn_wasm.wasm"
"$wasm_opt_bin" -Oz --enable-threads --enable-bulk-memory \
    --enable-mutable-globals --enable-nontrapping-float-to-int \
    --enable-sign-ext --enable-reference-types --enable-multivalue \
    "$threads_raw_dir/fmn_wasm_threads_bg.wasm" \
    -o "$package_dir/threads/fmn_wasm_threads_bg.wasm"
cp "$threads_raw_dir/fmn_wasm_threads.js" "$package_dir/threads/fmn_wasm_threads.js"
cp "$threads_raw_dir/fmn_wasm_threads.d.ts" "$package_dir/threads/fmn_wasm_threads.d.ts"
cp "$threads_raw_dir/fmn_wasm_threads_bg.wasm.d.ts" \
    "$package_dir/threads/fmn_wasm_threads_bg.wasm.d.ts"
cp crates/fmn-wasm/js/threads.js "$package_dir/threads/threads.js"
cp crates/fmn-wasm/js/threads.d.ts "$package_dir/threads/threads.d.ts"
cp crates/fmn-wasm/js/threads_worker.js "$package_dir/threads/threads_worker.js"

for required in fmn_wasm.js fmn_wasm_bg.js fmn_wasm_bg.wasm \
    fmn_wasm.d.ts fmn_wasm_bg.wasm.d.ts package.json README.md \
    threads/fmn_wasm_threads.js threads/fmn_wasm_threads_bg.wasm \
    threads/fmn_wasm_threads.d.ts threads/fmn_wasm_threads_bg.wasm.d.ts \
    threads/threads.js threads/threads.d.ts threads/threads_worker.js; do
    if [[ ! -f "$package_dir/$required" ]]; then
        echo "ERROR: wasm-pack omitted required package member: $required" >&2
        exit 1
    fi
done

if [[ -e "$package_dir/LICENSE" || -e "$package_dir/FONT_BUNDLE.json" || -e "$package_dir/licenses" ]]; then
    echo "ERROR: wasm-pack unexpectedly populated a governed license path" >&2
    exit 1
fi
cp LICENSE "$package_dir/LICENSE"
cp dist/FONT_BUNDLE.json "$package_dir/FONT_BUNDLE.json"
cp -R dist/licenses "$package_dir/licenses"

python3 - "$package_dir/package.json" "$workspace_version" "$source_commit" "$source_dirty" <<'PY'
import json
import sys

path, version, commit, dirty = sys.argv[1:]
with open(path, encoding="utf-8") as source:
    package = json.load(source)
if package.get("version") != version:
    raise SystemExit(f"generated npm version {package.get('version')!r} != workspace {version!r}")
package["files"] = [
    "fmn_wasm_bg.wasm",
    "fmn_wasm.js",
    "fmn_wasm_bg.js",
    "fmn_wasm.d.ts",
    "fmn_wasm_bg.wasm.d.ts",
    "README.md",
    "LICENSE",
    "FONT_BUNDLE.json",
    "licenses/",
    "threads/",
]
package["exports"] = {
    ".": {"types": "./fmn_wasm.d.ts", "import": "./fmn_wasm.js"},
    "./threads": {"types": "./threads/threads.d.ts", "import": "./threads/threads.js"},
}
package["publishConfig"] = {"access": "public"}
package["frankenManim"] = {
    "engineVersion": version,
    "sourceCommit": commit,
    "sourceDirty": dirty == "true",
    "timelineSchema": "FMTL/1",
    "threading": {
        "default": "single",
        "variants": ["single", "shared-memory-frame-pool"],
        "requiresCrossOriginIsolation": True,
    },
    "certified": False,
}
with open(path, "w", encoding="utf-8", newline="\n") as output:
    json.dump(package, output, indent=2, ensure_ascii=False)
    output.write("\n")
PY

if ! grep -q 'engine_version' "$package_dir/fmn_wasm.d.ts"; then
    echo "ERROR: generated TypeScript surface omitted engine_version()" >&2
    exit 1
fi
if ! grep -q 'class ThreadedFmnScene' "$package_dir/threads/threads.d.ts"; then
    echo "ERROR: handwritten threads TypeScript surface omitted ThreadedFmnScene" >&2
    exit 1
fi

PATH="$js_path" "$node_bin" - \
    "$package_dir/threads/fmn_wasm_threads_bg.wasm" \
    "$package_dir/threads/fmn_wasm_threads.js" <<'JS'
const fs = require("node:fs");
const [wasmPath, gluePath] = process.argv.slice(2);
const module = new WebAssembly.Module(fs.readFileSync(wasmPath));
const memoryImports = WebAssembly.Module.imports(module).filter((entry) => entry.kind === "memory");
if (memoryImports.length !== 1) {
  throw new Error(`threaded artifact must import exactly one memory; got ${memoryImports.length}`);
}
const exports = new Set(WebAssembly.Module.exports(module).map((entry) => entry.name));
for (const required of ["memory", "__tls_base", "__wbindgen_thread_destroy", "__wbindgen_start"]) {
  if (!exports.has(required)) throw new Error(`threaded artifact omitted required export ${required}`);
}
const glue = fs.readFileSync(gluePath, "utf8");
if (!glue.includes("shared:true")) {
  throw new Error("threaded wasm-bindgen glue does not construct shared memory");
}
JS

budget_for() {
    awk -F '\t' -v artifact="$1" '$1 == artifact { print $3; found = 1 } END { if (!found) exit 1 }' \
        crates/fmn-wasm/SIZE_BUDGET.tsv
}

wasm_path=$package_dir/fmn_wasm_bg.wasm
raw_bytes=$(stat -c '%s' "$wasm_path")
raw_budget=$(budget_for wasm-bindgen-bundler-pkg)
if (( raw_bytes > raw_budget )); then
    echo "ERROR: bundler wasm is $raw_bytes bytes, over the $raw_budget-byte budget" >&2
    exit 1
fi
gzip_path=$task_root/fmn_wasm_bg.wasm.gz
gzip -n -9 -c "$wasm_path" > "$gzip_path"
gzip_bytes=$(stat -c '%s' "$gzip_path")
gzip_budget=$(budget_for wasm-bindgen-bundler-pkg-gzip)
if (( gzip_bytes > gzip_budget )); then
    echo "ERROR: gzip bundler wasm is $gzip_bytes bytes, over the $gzip_budget-byte budget" >&2
    exit 1
fi

threads_wasm_path=$package_dir/threads/fmn_wasm_threads_bg.wasm
threads_raw_bytes=$(stat -c '%s' "$threads_wasm_path")
threads_raw_budget=$(budget_for wasm-bindgen-threads-pkg)
if (( threads_raw_bytes > threads_raw_budget )); then
    echo "ERROR: threaded wasm is $threads_raw_bytes bytes, over the $threads_raw_budget-byte budget" >&2
    exit 1
fi
threads_gzip_path=$task_root/fmn_wasm_threads_bg.wasm.gz
gzip -n -9 -c "$threads_wasm_path" > "$threads_gzip_path"
threads_gzip_bytes=$(stat -c '%s' "$threads_gzip_path")
threads_gzip_budget=$(budget_for wasm-bindgen-threads-pkg-gzip)
if (( threads_gzip_bytes > threads_gzip_budget )); then
    echo "ERROR: gzip threaded wasm is $threads_gzip_bytes bytes, over the $threads_gzip_budget-byte budget" >&2
    exit 1
fi

echo "==> npm pack + publish dry-run"
PATH="$js_path" "$npm_bin" pack "$package_dir" --ignore-scripts --json --pack-destination "$task_root" \
    > "$task_root/npm-pack.json"
(cd "$package_dir" && PATH="$js_path" "$npm_bin" publish --dry-run --ignore-scripts --json) \
    > "$task_root/npm-publish-dry-run.json"

tarball_name=$(python3 - "$task_root/npm-pack.json" "$task_root/npm-publish-dry-run.json" <<'PY'
import json
import os
import sys

pack_path, publish_path = sys.argv[1:]
with open(pack_path, encoding="utf-8") as source:
    pack = json.load(source)
with open(publish_path, encoding="utf-8") as source:
    publish = json.load(source)
pack = pack[0] if isinstance(pack, list) else pack
# npm >= 10 emits a bare receipt object; npm 9 wraps it in a
# {package-name: receipt} mapping. Accept both so the gate records the
# same inventory regardless of the host npm major.
if isinstance(publish, dict) and set(publish) != {"files"} and all(
    isinstance(value, dict) for value in publish.values()
):
    publish = next(iter(publish.values()))
publish = publish[0] if isinstance(publish, list) else publish

expected = {
    "package.json",
    "README.md",
    "LICENSE",
    "FONT_BUNDLE.json",
    "fmn_wasm.js",
    "fmn_wasm_bg.js",
    "fmn_wasm_bg.wasm",
    "fmn_wasm.d.ts",
    "fmn_wasm_bg.wasm.d.ts",
    "threads/fmn_wasm_threads.js",
    "threads/fmn_wasm_threads_bg.wasm",
    "threads/fmn_wasm_threads.d.ts",
    "threads/fmn_wasm_threads_bg.wasm.d.ts",
    "threads/threads.js",
    "threads/threads.d.ts",
    "threads/threads_worker.js",
}
for root, _, files in os.walk("dist/licenses"):
    for filename in files:
        relative = os.path.relpath(os.path.join(root, filename), "dist")
        expected.add(relative.replace(os.sep, "/"))

def inventory(document, label):
    actual = {member["path"] for member in document.get("files", [])}
    if actual != expected:
        missing = sorted(expected - actual)
        extra = sorted(actual - expected)
        raise SystemExit(f"{label} inventory mismatch; missing={missing}, extra={extra}")

inventory(pack, "npm pack")
inventory(publish, "npm publish --dry-run")
print(pack["filename"])
PY
)
tarball=$task_root/$tarball_name
if [[ ! -f "$tarball" ]]; then
    echo "ERROR: npm pack receipt names a missing tarball: $tarball" >&2
    exit 1
fi
npm_tarball_bytes=$(stat -c '%s' "$tarball")
npm_tarball_budget=$(budget_for npm-package-tarball-threaded)
if (( npm_tarball_bytes > npm_tarball_budget )); then
    echo "ERROR: npm tarball is $npm_tarball_bytes bytes, over the $npm_tarball_budget-byte budget" >&2
    exit 1
fi

echo "==> fresh npm consumer + webpack bundle"
(cd "$consumer_dir" && PATH="$js_path" "$npm_bin" init --yes >/dev/null)
(cd "$consumer_dir" && PATH="$js_path" "$npm_bin" install --ignore-scripts --no-audit --no-fund --no-package-lock "$tarball")
cp demo/wasm/smoke.mjs "$consumer_dir/smoke.mjs"
cp demo/wasm/webpack.config.cjs "$consumer_dir/webpack.config.cjs"
mkdir -p "$consumer_dir/dist"
(cd "$consumer_dir" && PATH="$js_path" NODE_PATH="$node_path" "$webpack_bin" --config webpack.config.cjs)
cp demo/wasm/smoke.html "$consumer_dir/dist/smoke.html"
cp demo/wasm/bundle.fmtl "$consumer_dir/dist/bundle.fmtl"

echo "==> real Chromium package smoke"
server_log=$task_root/http-server.log
python3 -u - "$consumer_dir/dist" > "$server_log" 2>&1 <<'PY' &
import http.server
import os
import sys
import urllib.parse

root = sys.argv[1]

class Handler(http.server.SimpleHTTPRequestHandler):
    def __init__(self, *args, **kwargs):
        super().__init__(*args, directory=root, **kwargs)

    def end_headers(self):
        parsed = urllib.parse.urlsplit(self.path)
        query = urllib.parse.parse_qs(parsed.query)
        negative_control = parsed.path == "/smoke.html" and query.get("nonisolated") == ["1"]
        if not negative_control:
            self.send_header("Cross-Origin-Opener-Policy", "same-origin")
            self.send_header("Cross-Origin-Embedder-Policy", "require-corp")
        super().end_headers()

server = http.server.ThreadingHTTPServer(("127.0.0.1", 0), Handler)
print(f"Serving fmn-wasm on port {server.server_port}", flush=True)
server.serve_forever()
PY
server_pid=$!
chrome_pid=
stop_server() {
    if kill -0 "$server_pid" 2>/dev/null; then
        kill "$server_pid"
        wait "$server_pid" 2>/dev/null || true
    fi
}
stop_chrome() {
    if [[ -n "$chrome_pid" ]] && kill -0 "$chrome_pid" 2>/dev/null; then
        kill "$chrome_pid"
        wait "$chrome_pid" 2>/dev/null || true
    fi
    chrome_pid=
}
stop_browser_services() {
    stop_chrome
    stop_server
}
trap stop_browser_services EXIT

port=
for _ in $(seq 1 100); do
    port=$(sed -n 's/.* port \([0-9][0-9]*\)$/\1/p' "$server_log" | head -n 1)
    if [[ -n "$port" ]]; then
        break
    fi
    if ! kill -0 "$server_pid" 2>/dev/null; then
        echo "ERROR: HTTP server exited before reporting its port" >&2
        cat "$server_log" >&2
        exit 1
    fi
    sleep 0.1
done
if [[ -z "$port" ]]; then
    echo "ERROR: HTTP server did not report a port" >&2
    exit 1
fi

capture_browser_dom() {
    local url=$1
    local profile=$2
    local dom=$3
    local log=$4
    mkdir -p "$profile"
    "$chrome_bin" --headless=new --disable-gpu --disable-dev-shm-usage \
        --user-agent="$http_user_agent" \
        --no-first-run --no-default-browser-check --remote-debugging-port=0 \
        --user-data-dir="$profile" "$url" > /dev/null 2> "$log" &
    chrome_pid=$!

    local devtools_port=
    for _ in $(seq 1 100); do
        if [[ -f "$profile/DevToolsActivePort" ]]; then
            devtools_port=$(head -n 1 "$profile/DevToolsActivePort")
            break
        fi
        if ! kill -0 "$chrome_pid" 2>/dev/null; then
            echo "ERROR: Chrome exited before exposing its DevTools port" >&2
            cat "$log" >&2
            return 1
        fi
        sleep 0.1
    done
    if [[ -z "$devtools_port" ]]; then
        echo "ERROR: Chrome did not expose its DevTools port" >&2
        return 1
    fi

    PATH="$js_path" "$node_bin" --experimental-websocket --input-type=module - \
        "$devtools_port" "$url" "$http_user_agent" > "$dom" <<'JS'
const [port, expectedUrl, userAgent] = process.argv.slice(2);
const deadline = Date.now() + 30_000;
let target;
while (Date.now() < deadline) {
  const targets = await fetch(`http://127.0.0.1:${port}/json/list`, {
    headers: { "User-Agent": userAgent },
  }).then((response) => response.json());
  target = targets.find((entry) => entry.type === "page" && entry.url === expectedUrl);
  if (target !== undefined) break;
  await new Promise((resolve) => setTimeout(resolve, 100));
}
if (target === undefined) throw new Error(`Chrome did not expose page target ${expectedUrl}`);

const socket = new WebSocket(target.webSocketDebuggerUrl);
await new Promise((resolve, reject) => {
  socket.addEventListener("open", resolve, { once: true });
  socket.addEventListener("error", () => reject(new Error("DevTools WebSocket failed")), {
    once: true,
  });
});

let nextId = 1;
const pending = new Map();
socket.addEventListener("message", (event) => {
  const message = JSON.parse(event.data);
  const waiter = pending.get(message.id);
  if (waiter === undefined) return;
  pending.delete(message.id);
  if (message.error !== undefined) waiter.reject(new Error(JSON.stringify(message.error)));
  else waiter.resolve(message.result);
});

function command(method, params = {}) {
  const id = nextId;
  nextId += 1;
  return new Promise((resolve, reject) => {
    pending.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params }));
  });
}

let state;
while (Date.now() < deadline) {
  const evaluation = await command("Runtime.evaluate", {
    expression: `JSON.stringify((() => {
      const result = document.getElementById("result");
      return result === null ? null : { status: result.dataset.status, text: result.textContent };
    })())`,
    returnByValue: true,
  });
  state = JSON.parse(evaluation.result.value);
  if (state?.status === "success" || state?.status === "failure") break;
  await new Promise((resolve) => setTimeout(resolve, 100));
}
const documentResult = await command("Runtime.evaluate", {
  expression: "document.documentElement.outerHTML",
  returnByValue: true,
});
console.log(documentResult.result.value);
socket.close();
if (state?.status !== "success") {
  throw new Error(`browser smoke ended in ${state?.status ?? "timeout"}: ${state?.text ?? ""}`);
}
JS
    stop_chrome
}

chrome_profile=$task_root/chrome-isolated-profile
chrome_dom=$task_root/chrome-isolated-dom.html
chrome_log=$task_root/chrome-isolated.log
capture_browser_dom \
    "http://127.0.0.1:$port/smoke.html?version=$workspace_version" \
    "$chrome_profile" "$chrome_dom" "$chrome_log"

if ! grep -q 'data-status="success"' "$chrome_dom"; then
    echo "ERROR: isolated packaged browser smoke did not report success" >&2
    cat "$chrome_dom" >&2
    cat "$chrome_log" >&2
    exit 1
fi

chrome_nonisolated_profile=$task_root/chrome-nonisolated-profile
chrome_nonisolated_dom=$task_root/chrome-nonisolated-dom.html
chrome_nonisolated_log=$task_root/chrome-nonisolated.log
capture_browser_dom \
    "http://127.0.0.1:$port/smoke.html?version=$workspace_version&nonisolated=1" \
    "$chrome_nonisolated_profile" "$chrome_nonisolated_dom" "$chrome_nonisolated_log"
stop_server
trap - EXIT

if ! grep -q 'data-status="success"' "$chrome_nonisolated_dom"; then
    echo "ERROR: non-isolated packaged browser refusal smoke did not report success" >&2
    cat "$chrome_nonisolated_dom" >&2
    cat "$chrome_nonisolated_log" >&2
    exit 1
fi

python3 - "$task_root" "$workspace_version" "$source_commit" "$source_dirty" \
    "$raw_bytes" "$gzip_bytes" "$threads_raw_bytes" "$threads_gzip_bytes" \
    "$wasm_pack_version" "$wasm_bindgen_version" "$wasm_opt_version" "$webpack_version" \
    "$webpack_cli_version" "$node_version" "$npm_version" "$chrome_version" \
    "$tarball" "$chrome_dom" "$chrome_nonisolated_dom" <<'PY'
import hashlib
import html
import json
import re
import sys

(root, version, commit, dirty, raw_bytes, gzip_bytes, threads_raw_bytes,
 threads_gzip_bytes, wasm_pack, wasm_bindgen, wasm_opt, webpack, webpack_cli,
 node, npm, chrome, tarball, dom_path, nonisolated_dom_path) = sys.argv[1:]
with open(tarball, "rb") as source:
    tarball_bytes = source.read()
def browser_result(path):
    with open(path, encoding="utf-8") as source:
        dom = source.read()
    match = re.search(r'<pre id="result" data-status="success">(.*?)</pre>', dom, re.S)
    if match is None:
        match = re.search(r'<pre data-status="success" id="result">(.*?)</pre>', dom, re.S)
    if match is None:
        raise SystemExit(f"success marker exists but result payload is missing in {path}")
    return json.loads(html.unescape(match.group(1)))

browser = browser_result(dom_path)
nonisolated_browser = browser_result(nonisolated_dom_path)
receipt = {
    "schema": "fmn-wasm-package-receipt/2",
    "version": version,
    "source_commit": commit,
    "source_dirty": dirty == "true",
    "wasm_pack": wasm_pack,
    "wasm_bindgen": wasm_bindgen,
    "wasm_opt": wasm_opt,
    "webpack": webpack,
    "webpack_cli": webpack_cli,
    "node": node,
    "npm": npm,
    "chrome": chrome,
    "wasm_raw_bytes": int(raw_bytes),
    "wasm_gzip_bytes": int(gzip_bytes),
    "threads_wasm_raw_bytes": int(threads_raw_bytes),
    "threads_wasm_gzip_bytes": int(threads_gzip_bytes),
    "npm_tarball": {
        "path": tarball,
        "bytes": len(tarball_bytes),
        "sha256": hashlib.sha256(tarball_bytes).hexdigest(),
    },
    "browser": browser,
    "nonisolated_browser": nonisolated_browser,
}
with open(f"{root}/receipt.json", "w", encoding="utf-8", newline="\n") as output:
    json.dump(receipt, output, indent=2, sort_keys=True)
    output.write("\n")
print(json.dumps(receipt, sort_keys=True))
PY

echo "OK: fmn-wasm npm package gate green; evidence preserved at $task_root"
