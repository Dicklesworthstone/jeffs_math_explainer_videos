# Embedded Studio UI Assets (§13.5)

The Studio's browser UI is **compiled into the binary and versioned with it**.
There is no runtime file serving: no route reads the filesystem, no asset
directory exists at install time, and no request path can name a file.

## The mechanism

The asset set lives in `fmn_studio::ui` as compile-time constants:

- `ui_assets()` — the static asset table (`UiAsset { route, content_type,
  bytes }`). `/studio.js` and `/studio.css` are embedded verbatim with
  `include_str!`. Adding an asset is adding a table entry — an
  asset change is a source change, reviewed and versioned like any other code.
- `studio_index_html(capability_hex)` — the index shell. It is *not* a static
  asset because it carries the per-session capability token; the version meta
  tag in it is baked at compile time via `concat!` + `env!`.

`include_bytes!` is the mechanism of record should a binary asset (a font, an
icon) ever join the table: same compiled-in guarantee, same versioning. The
current assets are text and need no build script or asset pipeline.

## Version coupling

`fmn_studio::STUDIO_UI_VERSION` is `env!("CARGO_PKG_VERSION")` — the
fmn-studio crate version, a compile-time constant. Because the assets are
compiled in, the UI asset set shares the binary's crate version. The coupling
is observable:

- every UI response carries the `X-FMN-Studio-Version` header
  (`STUDIO_UI_VERSION_HEADER`), so a connected browser can detect a stale page
  against a restarted Studio;
- the index shell carries
  `<meta name="fmn-studio-ui-version" content="<version>">`.

## What the host serves

The host (`fmn_studio::host`) is a small loopback-only HTTP/1.1 server with an
exact-route table — not a general web framework. The UI routes:

| Route | Source | Stamping |
|---|---|---|
| `GET /` | `studio_index_html(token)` (per-session) | version header + meta tag |
| `GET /studio.js` | `ui_asset("/studio.js")` table bytes | version header |
| `GET /studio.css` | `ui_asset("/studio.css")` table bytes | version header |

Everything else under the UI surface is refused: unknown clean paths get 404
from the exact-route table; ambiguous paths (`..` components, percent escapes,
backslashes) get 400 at the request parser, before any routing. Dynamic
routes (`/stream`, `/api/scrub`, `/api/restart`, `/api/event`, `/api/inspect`,
`/api/overlays`, `/api/session`) serve protocol data, never arbitrary files.

Every request is authenticated with the per-session 256-bit capability
(header or `cap` query, never both), Host/Origin-validated, rate-limited, and
bounded (§13.5's security model); the socket is loopback-only and the session
expires.

## Browser controls

The timeline supports frame stepping, keyboard navigation, preview playback,
and scrubbing. Releasing a scrub commits the position used by worker replay;
temporary previews do not replace that committed position. Restart reports
the restored frame and replay outcome. Frame count and effective FPS come
from the worker, and the displayed time comes from the captured scene clock.

The keyboard-accessible family tree opens live records, uniforms, placement,
bounds, and UTF-8 source spans. Object IDs belong to one capture, so a frame
change clears the inspector's selection. Debug layers draw tiles, control points,
bounds, winding, and depth over the preview using the renderer's coordinate map.
The browser verifies multipart framing, publication order, PNG hashes and
dimensions before marking the inspector synchronized. Large trees and
overlays have explicit display limits.

Scene input is opt-in and enabled only when the worker advertises a live
adapter and revision. The shipped `@builtin interactive.v1` route now supplies
one over the existing native `InteractiveScene` and replay engine. Immutable
FMTL and other captured preview artifacts still report no live adapter and
return a typed refusal without disabling timeline controls. The capability is
removed from browser history after asset startup and redacted from errors.

## Edit the shipped native canvas

Build the current shipping executable with the pinned toolchain, or use a
binary built from these sources:

```bash
cargo run --locked -p fmn-cli --features cli,batch --bin fmn -- \
  studio --resolution 384x216 --threads 1 @builtin interactive.v1

# With the current executable installed:
fmn studio @builtin interactive.v1
```

Open the private loopback launch URL, enable **Send preview input to the scene**, and focus
the preview canvas. This route has a blue circle nested in a group and a red
square to use as a color source. It needs neither CPython nor ffmpeg. Its native
windowed clock is 30 fps: frame zero is the constructed state, followed by 60
actual native wait frames. Offline skip/range/presenter flags are not applicable
to the live timeline and are rejected rather than reinterpreted.

| Control in the focused preview | Native editor operation |
|---|---|
| Ctrl/Command+T, then Ctrl/Command+click the circle | Switch selection scope to nested objects, then select the child |
| Hold G and drag; release G | Move the selection |
| Move the pointer away from the selection center, hold T, move radially, release T | Resize relative to the initial pointer distance |
| C, then click the red square | Pick its color for the selection |
| Ctrl/Command+C and Ctrl/Command+V | Copy and paste through the native clipboard |
| Ctrl/Command+Z; Ctrl/Command+Shift+Z | Undo; redo |
| Arrow keys | Nudge the native selection |

The family tree selects an **inspection entry**, not an editor target; use
native canvas selection for edits. Accepted input is committed at its original
frame in deterministic order. Scrubbing forward retains earlier edits;
returning before an edit shows the prior state. Editing the past replaces only
the later input branch. **Restart** recreates the native worker and verifies
re-executed commands against the recorded input/source and state hashes,
including the editor's selection and undo history.

The browser queues admitted key/pointer transitions rather than dropping
releases during a render. Pointer capture and focus-loss handling release held
edit modes. Before a seek or restart, previously admitted transitions and their
releases drain first. Queue limits refuse additional input explicitly, retaining
capacity for owned release events. A transport or revision error disables input
and discards unsent actions; it does not roll back actions already committed.

Every shipped HTTP event requires `worker_generation`, `frame`, and `revision`.
The inspector supplies the generation through `X-FMN-Worker-Generation` and the
revision through `view.input_revision`. Host generation checks and dispatch are
one serialized operation. Stale revisions, unsupported explicit targets,
malformed input, and coordinate/delta budget overflow cannot silently edit a
different worker. Capture-local inspector IDs are not durable target handles;
explicit HTTP `target` values are currently refused. Normal canvas input uses
the native hit-testing and selection owner instead.

This is a registered affine native editing surface, not arbitrary Rust-source
compilation, perspective picking, a Python Studio worker, or graphical edits
written back into source files. Worker restart retains the session in memory;
use **Save session** before closing the entire host to keep committed edits
for a later invocation. The reusable native worker and explicit
source-project host are described in `docs/NATIVE_STUDIO_WORKER.md` and
`docs/NATIVE_PROJECT_STUDIO.md`; their own capability boundaries still apply.
No warm-restart latency or full G3/certification claim follows from this route.

## Save and reopen native editing sessions

On the registered live native canvas, **Save session** downloads
`studio-session.fmns`. It first drains accepted input and releases held edit
modes. The archive preserves committed commands, checkpoints, native editing
history, and the last committed timeline position. Merely playing or previewing
a different frame does not change that saved position. Saving is explicit, not
automatic; wait for the browser download to complete before closing Studio.

Reopen with the same executable, scene and resolved render settings:

```bash
fmn studio --resolution 384x216 --threads 1 \
  --restore-session /path/to/studio-session.fmns @builtin interactive.v1
```

Use the resolution and other settings from the original invocation, not
necessarily the example above. Thread count may change: it is deliberately
excluded by the engine's canonical configuration identity. Session ports and
capability tokens are newly created on each invocation. The old capability
does not authenticate the new host. Resume never changes the input archive;
continue editing and download a new session to retain further work.

The versioned `FMNS` archive carries the canonical journal and checkpoints,
so no old cache directory is required. It is bounded to 64 MiB overall and to
the receiving worker's tighter journal/checkpoint budgets. Checksums detect
corruption, not authorship. Archives contain no launch command, executable
path, environment settings, capability token, or arbitrary-code deserialization.
The selected host validates the scene, build, resolved configuration, and
independently resolved native input identities before restoring through the
existing worker protocol. Corruption, unsupported versions, changed inputs,
opaque effects and replay divergence refuse rather than silently starting an
empty or partially restored project.

Typed native editing commands use the journal's `Input` vocabulary, retaining
their stateful effect classification and input identities. They are not
arbitrary `Custom` callbacks; those remain opaque replay barriers. The journal
minor version and native edited-checkpoint schema distinguish the two, and
the live worker handshake refuses incompatible versions.

`--restore-session` currently requires the registered `@builtin interactive.v1`
adapter and an existing regular file. It does not import arbitrary Rust/Python
scenes, migrate across builds, or write graphical changes back into source.
The exact authenticated `GET /api/session` endpoint exports in-memory data;
there is no HTTP upload, server-side save-path parameter, or filesystem browser.

## The acceptance tests

`crates/fmn-studio/tests/ui.rs` drives the real socket:

- each served script and stylesheet is byte-identical to its embedded entry,
  with matching SHA-256 content hashes;
- all UI routes carry `X-FMN-Studio-Version: <crate version>` and the shell
  carries the matching meta tag, proving the version coupling end to end;
- unknown and traversal-shaped paths have no route (404/400), proving there is
  nothing to reach a filesystem with.

`fmn_studio::ui`'s unit tests pin the table itself: routes exact and unique,
assets non-empty, the version equal to the crate version.

The Gauntlet fast tier registers `lifecycle.studio_native_worker.v1` and the
positive `lifecycle.studio_native_interactive.v1`, using Cargo's exact shipping
CLI artifact to exercise the real supervisor, isolated worker, authenticated
sockets, decoded PNG pixels, inspector, guarded edits and checkpoint replay.

`node --test crates/fmn-studio/tests/input_queue.mjs` exercises the actual
shipped controller's queue with isolated DOM/transport fixtures: rapid gestures,
key release ordering, focus/capture loss, modifier/layout changes, bounded
admission, restart/seek ordering, and generation/revision refusal. This layer
does not substitute for native rendering or a real browser.

After building `fmn`, run the real Chrome acceptance driver:

```bash
node crates/fmn-studio/tests/browser.mjs /absolute/path/to/fmn /absolute/evidence-directory
```

Use Node with built-in WebSocket support. `FMN_CHROME` can select the installed
Chrome executable. The driver checks animated fields, nested TeX spans,
scrubbing, overlays, restart, keyboard operation, a 390-pixel viewport, and
request refusals. Its live-canvas scenario selects a nested child, drags,
resizes, recolors, copies/pastes, undoes, resumes playback, scrubs and restarts;
it compares inspected state and decoded rendered pixels. Existing immutable
input-refusal scenarios remain. Screenshots and a capability-redacted receipt
record the source state, executable hash and actual results.

The saved-session scenario additionally downloads through the real browser,
closes the entire host, reopens with a different render-thread count, checks
native state and decoded pixels, performs undo and further edits, saves again,
and repeats the whole-host restart. It also checks export authentication and
refuses damaged, truncated, over-budget, linked and mismatched-input archives.
