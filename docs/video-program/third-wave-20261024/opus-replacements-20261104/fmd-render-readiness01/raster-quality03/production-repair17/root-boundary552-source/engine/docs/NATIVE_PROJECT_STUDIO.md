# Native project Studio: edit, compile, replace, replay

Studio now has a source-project host, rather than requiring every application
host to relaunch the same already-built executable. `CargoRebuildDriver`
implements the existing `RebuildDriver` interface with actual incremental
Cargo builds. `native_project` connects it to source watching, the isolated
worker, the authenticated browser host and the existing replay machinery.

## Run against this checkout's native example

From the repository root, with its pinned nightly toolchain installed and
locked dependencies available locally:

```sh
cargo run --locked -p fmn-studio --features native-build --example native_project -- \
  "$(rustup which cargo)" "$PWD/Cargo.toml" fmn-studio \
  example:native_live NativeLive --native-live-worker \
  "$PWD/crates/fmn-studio/examples/native_live.rs"
```

Open the private loopback URL it prints. Move the timeline to a committed
frame, then edit the watched example's geometry or color. After the edit
settles, the host compiles a new child, performs the normal version/build
handshake, re-executes the invalidated commands and publishes the restored
committed frame. Browser **Restart** invokes the same compiler-backed path.
A syntax error returns bounded compiler diagnostics while retaining the old
worker and last published image. Fixing the source triggers another build.
Press Enter in the launching terminal to stop the host.

The camera application can be selected with `example:native_camera`, scene
`NativeCamera`, worker argument `--native-camera-worker`, and that example's
source path. Its existing camera rendering, animation and capability boundaries
remain unchanged.

For another project, the positional arguments are:

```text
native_project CARGO MANIFEST PACKAGE bin:TARGET|example:TARGET SCENE WORKER_ARG [WATCH_PATH ...]
```

`CARGO` must be an absolute path to the selected nightly Cargo executable;
`rustup which cargo` supplies the actual selected toolchain binary. The host
uses its bounded verbose version output to obtain the explicit native target
triple, and uses the sibling rustc from that toolchain. The source manifest and
additional watch paths are resolved before starting. `bin:TARGET` selects one
package binary, while `example:TARGET` selects one executable example. Neither
package nor target supports wildcards, shell syntax or arbitrary Cargo flags.

The selected child must implement the existing versioned Studio protocol,
including the scene name and canonical native seek operations. A typical main
constructs `NativeSceneWorker`, hashes its actual executable for `build_id`,
and calls `serve_worker` on stdin/stdout. See `native_live.rs` and
`native_camera.rs` for complete native worker implementations. The parent
never constructs the scene. This host does not turn an arbitrary Rust `main`
into a Studio worker or discover source-level scene functions automatically.

## Build authority and project configuration

Builds select an explicit manifest, package, executable target, target triple
and output directory. They are `--locked` and offline by default. Prepare or
update a project's Cargo.lock and fetch dependencies deliberately before
launching an offline session. Library callers can explicitly disable offline
mode; the command-line example does not silently enable network downloads.

The selected project must use a nightly Cargo supporting `-Z unstable-options
-C`. The driver passes the manifest directory through Cargo's own `-C`, so its
`.cargo/config.toml`, workspace configuration and relative build settings are
read in the intended project context. `--manifest-path` alone does not change
Cargo's configuration search directory. The supervisor's working directory is
never changed, and the exact-image process primitive still receives `cwd: None`.
Explicit CLI target/output flags take precedence over a project's default cross
build target or output directory.

The compiler environment is a supplied allowlist, not an inherited environment.
The example snapshots only its documented toolchain/home/path/temp/platform
variables and supplies an explicit rustc. Worker arguments, environment and
working-directory authority are separate; compiler credentials are not copied
into the child environment. Advanced integrations can configure selected Cargo
features, release mode, worker environment and resource limits through
`CargoBuildConfig` without exposing an arbitrary shell-command option.

The `native-build` feature enables the existing fmn-platform exact-image process
substrate for this development-host operation. It introduces no new third-party
dependency, scene subprocess hook, rendering backend or second scheduler.
Compiler output is capped per stream, compiler runtime has a process-tree
deadline, and failures remain errors rather than selecting a stale output file.

## Immutable worker generations

A successful build is not returned as a mutable `target/debug/...` pathname.
The driver reads the bounded regular executable, hashes its bytes, and creates
an exclusive private copy before returning `WorkerArtifact`. The ordinary
supervisor handshake checks that exact content identity. Later Cargo links
cannot overwrite the executable needed to restart the previous worker.

Identical output reuses its verified private image. Different output creates a
new generation. Defaults admit at most 32 distinct images of at most 512 MiB
each; configured limits have hard ceilings. Exceeding a limit refuses publication
without deleting earlier images. Start a new build session to release its old
generations. The incremental Cargo cache is separately caller-managed; this is
not a disk quota for all compiler intermediate files.

`CargoRebuildDriver` deliberately does not delete its images on Drop because
cloned `WorkerArtifact` values may outlive it in a supervisor. Call `cleanup`
only after stopping all users. The runnable host removes its private image
directory after shutting down its worker, but preserves the incremental cache.
The file checks and exclusive private-directory creation are not an atomic
sandbox against a malicious owner of the supplied filesystem paths.

## Source watching and replay

`SourceWatch` watches explicit files or directory trees by content digest,
including new/deleted files and atomic-save replacements. It ignores `.git`
and `target` directories during recursive traversal, plus explicit exclusions.
A file's mtime alone is not a change. The default scan bounds are 4,096 entries,
64 MiB of content and depth 64; an oversized/unreadable/symlinked input refuses
rather than claiming a complete scan. This is not Cargo dependency discovery:
add relevant library, asset or configuration paths to the watch list yourself.

The project host always watches its manifest and lockfile. Without extra paths,
it also watches the project's `src`; with explicit paths, it watches those
instead. Its own generated output is excluded. The example waits for 300 ms of
stable observed content and polls at 100 ms intervals. Those are debounce
settings, not a compile/rebuild performance guarantee.

A content change is marked attempted before its build starts. An unsuccessful
build therefore does not loop forever on identical bad source. Edits made while
compilation is running are picked up by a later poll. Reverting an unsettled edit
to the previously attempted content cancels the unnecessary pending build.
The watcher invokes the existing authenticated Restart endpoint; it does not
inject source code into a running process or bypass browser authorization.

A generic project host cannot attest all asset reads made by arbitrary project
code. It therefore supplies a conservative verifier and re-executes invalidated
commands instead of importing stale checkpoints from a changed build. It restores
the last **committed** timeline frame, not an uncommitted preview or transient
interactive edits. Applications with their own verified input closure can use
`CargoRebuildDriver` with a more selective existing supervisor verifier.

Compilation failures do not replace the healthy worker. The existing supervisor
also rejects a bad new handshake before replacing it. Failures in later scene
execution/replay are reported by the existing runtime; this implementation does
not promise transactional rollback of arbitrary scene side effects or every
post-handshake failure. Requests that need the supervisor serialize behind a
rebuild; the browser can retain the last published frame while it is compiling.

## Trust and remaining boundaries

Only compile **trusted projects**. Cargo build scripts, procedural macros and
native scene code execute with the selected user's capabilities. Process-tree
supervision, loopback tokens and byte limits are not a filesystem/network sandbox
for arbitrary project code. The example uses the existing Unix host-entropy
capability; its real compiler/socket acceptance is currently Unix-only.

The new host is an explicit source-project front door; the default `fmn-cli`
captured-artifact path is unchanged. It does not add persistent graphical edits,
perspective picking, audio export, arbitrary code discovery, or constant-time
warm rebuild recovery. Dynamic-library deployment beside copied executables may
require an application-specific worker environment/artifact policy. Compiler
incrementality, native rendering and source replay are connected, but no Studio
latency or release-certification performance claim is made.

## Acceptance

The standard Studio suite includes scripted compiler outcomes with real private
artifact publication and clock-injected watcher tests. With `native-build`, the
real Cargo acceptance builds private projects, executes both old/new native
images, introduces syntax errors, verifies the healthy worker survives, and
checks changed native PNGs and committed-frame replay through actual supervisor
and authenticated HTTP calls. Another test requires project-local `.cargo`
configuration and includes a failing --manifest-path-only negative control.

```sh
cargo test --locked -p fmn-studio
cargo test --locked -p fmn-studio --features native-build \
  --test project_cargo --test project_cargo_config -- --nocapture
cargo build --locked -p fmn-studio --features native-build --example native_project
```

These run alongside the existing native child/camera HTTP self-tests. They do
not replace or weaken the mandatory repository governance, format or lint gates.
