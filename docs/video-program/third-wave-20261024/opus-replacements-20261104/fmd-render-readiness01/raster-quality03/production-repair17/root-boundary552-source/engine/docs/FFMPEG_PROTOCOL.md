# FFMPEG_PROTOCOL.md — v2

The contract of FrankenManim's **one external tool** (§3 D2, §14.3,
D-23). It requires ffmpeg to be the only external subprocess the engine
invokes — encode, mux, transcode — and this document defines the protocol every
conforming invocation obeys. Implementation: `fmn-output::negotiate` (pure
argv construction) and `fmn-output::ffmpeg` (sandboxed execution) over
`fmn-platform::process` (the argv-only interface). Section 2.1 records the
supported exact-image mechanisms, the separate filesystem capability boundary,
and the residual threat model that the implemented protocol does not claim to
close.

## 1. Negotiation, not a fixed pipe

v1-era manim pipes `rawvideo/rgba`, then repairs orientation with
`vflip` and color with `eq`. v2 deletes both repairs structurally:

- **Orientation.** fmn-frame renders in output orientation (row 0 is
  the top row, D-23). No argv builder can emit `vflip`; the contract
  suite asserts no filter argument exists in any invocation.
- **Transfer.** fmn-frame applies the transfer function once,
  natively. Color metadata (`-color_primaries`, `-color_trc`,
  `-colorspace`, `-color_range`) *describes* the bytes; no `eq`
  correction exists.

Negotiated per job:

| Dimension | Values | Wire meaning |
|---|---|---|
| Pixel format | `rgba`, `bgra` (alpha/compat), `nv12` (8-bit video), `p010le` (10-bit) | tightly-packed frames on stdin, frame-index order |
| Frame rate | exact rational `num/den` | from the RationalFrameClock, no float drift |
| Color | primaries BT.709; transfer `iec61966-2-1` or `bt709`; range `tv`/`pc` | metadata only, never a correction |
| Container | MP4 (`+faststart`), MOV, transparent MOV (`qtrle`/argb, requires an alpha wire), GIF mode | |
| Encoder | `Auto` → software default (`libx264`; `qtrle` for transparent MOV); hardware by explicit name only | identity → provenance |

The arithmetic that motivates NV12: 3840×2160 RGBA8 is 33,177,600
bytes/frame against NV12's 12,441,600 — 2.67× less pipe payload
(~1.99 GB/s vs ~746 MB/s at 60 fps) before counting copies.

Refused negotiations are typed and named: transparent MOV on an
opaque wire, CRF on a non-CRF encoder, an encoder the installed
ffmpeg does not offer, a payload that is not a whole number of
frames, zero dimensions or frame rate.

## 2. The D2 invocation protocol

Offline native video requests use the finite render policy described in
[native video limits](NATIVE_VIDEO_LIMITS.md): a 24-hour default watchdog,
explicit timeout/log/artifact/input budgets, and the same admission function
for the Rust API and standalone CLI. The streaming encoder's wall clock
includes scene construction and frame rendering. CLI overrides are
`--ffmpeg-timeout`, `--ffmpeg-max-log-bytes`, `--ffmpeg-max-artifact-bytes`, and
`--ffmpeg-max-input-bytes`. The resolved limits enter the video receipt and CLI
provenance; they do not alter tool-probe deadlines or the low-level boundary's
short-job defaults. [Video quality](VIDEO_ENCODING_QUALITY.md) documents the
separate negotiated CRF/preset/tune/bitrate controls.

### 2.0 Audited executable discovery

Executable discovery is an fmn-platform capability, not process-runner
behavior. The host snapshots one explicit native `PATH` value when it
constructs `FfmpegLocator`; neither the locator nor any consumer rereads the
environment. An absolute configured path bypasses `PATH` and is canonicalized
exactly as supplied. A relative configuration may be only the fixed bare
`ffmpeg` name (`ffmpeg.exe` is also accepted on Windows).

Before searching, the locator validates the complete path list. Empty entries
(which conventional lookup can reinterpret as the current directory),
relative entries, parent traversal, non-UTF-8 or control-bearing entries,
embedded NUL, malformed Windows quoting, and bounded-input overruns reject the
whole policy before any candidate is inspected. Search order is otherwise
preserved. A present directory, broken link, non-executable file, or
interpreter script is skipped; the first regular host-native executable image
wins. Structural policy v2 is deeper than a magic-byte check: Linux-family
hosts require a bounded ELF64 executable/PIE for the host machine with a valid
program-header table, file-backed executable entry point, interpreter shape,
and bounded GNU-property notes; macOS requires a bounded executable Mach-O
64-bit image or universal host slice with complete load-command/segment ranges,
a hard `__PAGEZERO`, and a file-backed `LC_MAIN`; Windows requires a
host-machine PE32+ executable with aligned, bounded optional/section tables and
a file-backed executable code entry, and refuses DLLs. Images above the 1 GiB
executable bound are refused before hashing or copying. This is a versioned
container attestation, not a claim of complete OS-loader equivalence. Other
hosts refuse discovery until their native image format is explicitly governed.
Windows search tests exactly `ffmpeg.exe` in each validated directory and
never consults `PATHEXT`, the application/current/system directories, or
command-interpreter formats.

Symlinks are supported for ordinary package-manager/version-manager layouts,
but the issued `FfmpegExecutable` contains their canonical target and is the
only input accepted by fmn-output resolution. Retargeting the searched symlink
afterward therefore cannot redirect the issued path, and a raw path cannot
bypass the locator. The token reopens and structurally revalidates the source
through each exact handle used for hashing/copying. This is not proof of
authorship: fmn-output owns the hash/private-copy binding below and revalidates
the exact private copy in the byte-binding portion of the D2 boundary.
Its resolution probe requires a strict UTF-8 first line that begins exactly
`ffmpeg version ` and contains no control characters. Native-image shape plus
that protocol response proves only that the selected bytes speak the governed
surface; it is not cryptographic authentication of the ffmpeg project.

Every invocation:

1. **argv-only, private-copy binding.** The configured tool is
   selected by a typed locator token and SHA-256 fingerprinted through a
   validated, bounded source handle. Resolution copies and hashes those
   bytes in one bounded pass through `create_new` into a private probe
   directory. The exact create-new handle is flushed, synced,
   permissioned, native-image-attested, and rehashed before it is closed;
   it is then reopened read-only, re-attested, rehashed, retained across
   spawn, and checked together with the current private pathname
   immediately around execution. (Linux refuses execution while a
   write-capable handle remains open.) Every later capability probe and
   job repeats this sequence and passes only the fixed absolute private
   leaf (`fmn-bound-ffmpeg`, or `.exe` on Windows) to `ProcessSpec`.
   The configured pathname never selects what the process mechanism
   spawns, and ambient `PATH` never selects it.
2. **Owned private hierarchy.** Resolution canonicalizes the caller's
   workdir parent and atomically claims one session root. Each probe and
   job then claims an exclusive child (`0700` on Unix). A collision is
   never opened, cleaned, or reused. The recorded filesystem identity of
   the session and each child must still match before later path-based
   work or cleanup; mismatch retains the path untouched. The child's `TMPDIR`,
   artifact, and bound executable all live there. Every governed path passed to
   ffmpeg is absolute: external audio and concat inputs are canonicalized
   before the job directory is created, while private inputs and outputs are
   constructed beneath that absolute directory. The child can therefore
   inherit the engine's working directory without interpreting anything
   relative to it; this avoids making a requested `cwd` another precondition
   of the pinned standard library's `posix_spawn` path.
3. **Environment allowlist + locale pinning.** The child environment
   is cleared and rebuilt as exactly `LANG=C`, `LC_ALL=C`,
   `TMPDIR=<job dir>`.
4. **Timeout + cancellation.** A wall-clock bound or cooperative
   cancellation kills the complete isolated process group. Targets
   without a safe process-tree mechanism are refused before spawn.
5. **Output-size limits.** Captured logs are capped per stream
   (overflow kills the child); the artifact is size-checked against a
   declared budget before publication.
6. **Atomic publication.** The artifact reaches its destination only
   through `rename` after verification. A failed, timed-out, or
   oversized job leaves the destination untouched.
7. **Provenance.** Every invocation records the canonical configured
   path, the private executable path actually spawned, their enforced
   shared SHA-256, the versioned native-image format/architecture
   attestation, the `-version` line, resolved encoder, and full argv.
   The retained private handle and current private path are both
   re-attested and rehashed after the process exits and before its
   artifact can publish.

### 2.1 Supported executable and filesystem capability

The same private-copy form used by jobs is what earns the resolution-time
version. This deliberately supports self-contained or relocation-safe
ffmpeg builds. On the currently supported Unix boundary, an installation
whose loader requires libraries beside the configured executable—for example
through ELF `$ORIGIN` or Mach-O `@executable_path`—is rejected as
`UnsupportedRelocatedExecutable` if the private-copy probe cannot run. There
is no fallback to probing or executing the mutable configured path.

On Unix, missing workdir-parent components and every claimed directory are
created as mode `0700`. A canonical ancestor writable by group/other is
accepted only when it has the sticky bit (as `/tmp` normally does).
Parent-directory (`..`) components are refused before creation. Windows and
every other non-Unix target fail earlier with `Workdir`, before any executable
probe (including the default `fmn doctor` probe), because safe `std` cannot
prove a private directory ACL there and because the exact-image mechanism
starts the child with a NULL `CreateProcessW` current directory: the child
would inherit the caller's working directory, which the default Windows DLL
search order consults for non-KnownDLL imports. Relocating and hashing the EXE
does not bind its DLL closure, so the boundary stays fail-closed on Windows
until a controlled child cwd and a verified DLL closure are proved natively
(issue #3). This is separate
from the process mechanism's own fail-closed requirement for complete process-tree
cancellation.

Safe `std` does not expose a portable execute-by-handle primitive or
directory-handle-relative recursive deletion. The caller must supply trusted
workdir ancestry; within it, the private hierarchy, content hashes, and
filesystem-identity checks prevent ambient-path substitution and make
persistent replacement fail closed. A hostile owner of that ancestry or an
actor already running under the same OS identity can still race a pathname
between the last check and the OS spawn/removal operation. Supporting that
stronger threat model requires an opaque host executable/filesystem capability;
it is not papered over by a weaker or unsafe platform path.

The process mechanism no longer rides `std::process::Command`'s potential
`fork`/`execvp` fallback. `StdProcessRunner` delegates to the audited
asupersync exact-image capability and requires an absolute `ProcessSpec`
program. Linux and macOS use `posix_spawn` with that absolute image into a new
process group—never `posix_spawnp`, `execvp`, or an `ENOEXEC` shell fallback.
Windows uses an explicit `CreateProcessW` application and assigns it atomically
to a kill-on-close Job Object. Other targets select `exact_image.unavailable`
and refuse before issuing a process. The stable mechanism identity
(`posix_spawn.absolute_path.new_process_group` or
`create_process_w.explicit_application.atomic_job_list`) and its policy version
enter C9 provenance for every successful invocation.

This exact-image capability makes a loader rejection a typed spawn failure,
not an opportunity to try an interpreter. It does not recast the versioned
native-container parser as proof of complete loader equivalence, and it does
not widen the trusted-ancestry filesystem bound above. The paired tranches
`fm-x4pp`, `fm-2sxz`, and `fm-8foa` complete provenance: their native functional suite
proved the no-interpreter mechanism and private-workdir capability on linux-x86-64 glibc,
linux-aarch64 musl, macos-aarch64, and windows-x86-64 MSVC.

## 3. Optionality

ffmpeg's absence is a **capability error naming the alternative**:
y4m, PNG sequences, and GIF are native outputs needing no ffmpeg;
only encoded video (mp4/mov), the audio mux, and media transcode
require the tool. There is no silent format substitution, ever.

## 4. Retained modes

- **GIF mode** — `-f gif`, muxer-level, no `-c:v` (parity with the
  Reference; the native GIF codec is the default path).
- **Two-stage audio mux** — stage 1 encodes video; stage 2 runs
  `-i video -i audio -c:v copy -c:a aac -map 0:v:0 -map 1:a:0`.
  Stage 2 **must not re-encode video**: `-c:v copy` is contract,
  asserted by the fake-ffmpeg suite.
- **Insert files / partial movies** — concat demuxer with stream
  copy (`-f concat -safe 0 -i list -c copy`); every input is resolved
  to an absolute UTF-8 path before list creation, and paths containing
  quotes or line breaks are refused rather than escaped.
- **`--subdivide` outputs** — one boundary job per subdivision; the
  protocol is per-invocation and needs nothing special.
- **`--prerun` counting** — a counting pass invokes the boundary
  zero times; the contract suite asserts no spawn occurs.
- **Media transcode as a capability** — audio decode beyond WAV
  (`-vn -acodec pcm_s16le -f wav`) and exotic image formats
  (`-frames:v 1 -c:v png`) ride the same sandbox and the same
  fingerprinting; absence yields the same named capability error.

## 5. Hardware encoders

Hardware encoders enter **here and only here**: ffmpeg products are
excluded from certification by construction, so hardware encode
changes nothing about the determinism story. Recognized names:
`h264_videotoolbox`, `hevc_videotoolbox`, `prores_videotoolbox`,
`h264_nvenc`, `hevc_nvenc`, `av1_nvenc`. Policy: `Auto` always
resolves to the software default; hardware is selected only by
explicit name, validated against the probed `-encoders` inventory,
and recorded in provenance. `fmn doctor` reports the installed
ffmpeg's fingerprint and its recognized hardware encoders.

## 6. CI: the fake ffmpeg

The protocol is CI-verified without real encoders at three complementary
layers:

- **Contract suite** (`ScriptedRunner`): every negotiation dimension,
  the env allowlist, both mux stages, and the failure modes (timeout,
  log overflow, nonzero exit, missing/oversized artifact), executable
  substitution, relocation refusal, exclusive-directory collision
  handling, and fail-closed cleanup after directory replacement are
  asserted against recorded `ProcessSpec`s — no process spawns.
- **Sandbox suite** (Cargo-built host-native fake ffmpeg + the real
  `StdProcessRunner`, under the `ffmpeg-test-fixture` feature): a
  std-only stand-in binary asserts the spawn-level behavior —
  artifact publication, stdin consumption, timeout kill, oversized-
  artifact refusal, failed-job destination integrity, and that source
  replacement after binding cannot change the executable that runs.
  Interpreter-script bytes remain only as hostile replacements whose
  rejection is asserted; no positive boundary fixture executes a shell.
- **Native exact-image capability suite** (`fmn-platform`): executable text
  remains a spawn error without an interpreter, argv/environment/stdio are
  preserved, mechanism identity is locked, and complete-tree timeout and
  cancellation are exercised natively on linux-x86-64 glibc,
  linux-aarch64 musl, macos-aarch64, and windows-x86-64 MSVC. The Windows leg
  is process-substrate evidence, not a claim that the separate non-Unix
  private-workdir capability has landed.
- **Real-ffmpeg smoke test**, behind `FMN_REAL_FFMPEG=1`, encodes a
  short NV12 clip through the installed tool.
