# Cache and Config Directory Conventions (W11)

Where FrankenManim keeps persistent state on each platform, how `--clear-cache`
and `fmn doctor` treat those locations, and the upgrade policy. This document
is locked to the code by a drift test (`fmn-cache/tests/conventions.rs`): if
the resolution behavior changes, the test fails until this document catches up.

These conventions **replace the Reference's `diskcache` and `appdirs`
dependencies** (plan dependency table): `diskcache`'s role is fmn-cache's
content-addressed store; `appdirs`' role is the per-platform resolution
below, implemented in `fmn_cache::resolve_host_cache_root` with no dependency
at all.

## Cache root resolution

The effective cache root comes from the `directories.cache` config key
(default: empty), resolved through the Reference's precedence —
**built-in defaults → user config file(s) → CLI overlay** — with `--cache-dir`
as the CLI spelling of the same key. Resolution rules:

1. **A non-empty configured value wins.** Absolute paths are used verbatim;
   relative paths are anchored to the current working directory. A configured
   value containing `..` is refused (`CacheRootError::InvalidConfigured`).
2. **An empty value selects the platform convention**, then appends the
   dedicated leaf `franken-manim`:

   | Platform | Resolved cache root |
   |---|---|
   | Linux and other Unix | `$XDG_CACHE_HOME/franken-manim` when `XDG_CACHE_HOME` is set and absolute; otherwise `$HOME/.cache/franken-manim` |
   | macOS | `$HOME/Library/Caches/franken-manim` |
   | Windows | `%LOCALAPPDATA%\franken-manim`; otherwise `%USERPROFILE%\AppData\Local\franken-manim` |

   A *relative* `XDG_CACHE_HOME` is ignored, per the XDG Base Directory
   specification. Environment paths are used as native bytes — never lossy
   Unicode-converted.
3. **No guessing.** If no trustworthy absolute base exists, resolution fails
   with `CacheRootError::PlatformDefaultUnavailable` — the store never falls
   back to the current directory or a temp directory.

`Store::open_host`, `fmn doctor`, and `--clear-cache` all resolve through
this one function, so the three can never disagree about where the cache is.

## Store layout and the migration-note policy

Inside the root:

```
<root>/
  STORE_OWNER        path-bound ownership marker (clear authorization)
  STORE_FORMAT       store-format stamp, currently "fmn-cache 1"
  ns/<name>/v<version>/…   one directory per versioned namespace
```

The migration policy is **versioned namespaces**, not in-place migration: a
namespace is `(name, schema_version)`, and its directory is
`ns/<name>/v<version>`. When a consumer's on-disk format changes, it bumps
its schema version — the typeset cache's `TYPESET_FORMAT_VERSION` in fmn-tex
is the standing example — and the new version opens cold. Nothing is
rewritten, nothing is half-migrated, and unrelated namespaces are untouched.
Versions coexist until an ownership-authorized `--clear-cache`: an opener
cannot prove that another process has stopped using an older sibling version,
so it never removes one. The earlier `Namespace::purge_stale_versions` design
is deliberately retired; automatic reclamation could make two live builds
delete and recreate each other's caches. A whole-store format break is the
`STORE_FORMAT` stamp instead: an unrecognized stamp is
`CacheError::FormatUnsupported`, never a misread.

The user-visible migration note for any release is therefore one of two
sentences: *"cache namespace X moved to vN; older versions remain inert until
the next explicit `--clear-cache`"* or *"the store format stamp moved; clear
the old cache with `--clear-cache`"*. Silent format reinterpretation and
automatic sibling-version deletion are not policy options.

Corruption posture: every entry is checksum-verified on read; a corrupt entry
is evicted and reported as a miss, never trusted, never fatal. The cache is an
optimization, never an oracle — `--clear-cache` can never change a render.

## `--clear-cache` and `fmn doctor`

- **`--clear-cache`** resolves the root as above, then requires an
  ownership proof: `CacheClearAuthorization::authorize` validates the
  path-bound `STORE_OWNER` marker before touching anything, and the clear
  itself atomically quarantines **only the managed `ns` tree** — a foreign
  directory is refused without mutation, and files outside `ns` are never
  removed. Concurrent readers see misses; concurrent writers recreate what
  they need. `--robot` emits one NDJSON `cache_clear` record naming the
  resolved root and the outcome (`cleared` / `already_absent`).
- **`fmn doctor`** reports the resolved root, whether it exists, and a direct
  entry count (or a precise warning) in both human and `--robot` output. An
  unavailable platform default degrades to a named report line; an invalid
  *configured* root is a config error. Doctor's cache report and the store
  contract are pinned to each other by test
  (`doctor_uses_the_same_platform_default_cache_root_as_the_store_contract`).

## Renders use the same root

Every front door that typesets opens the store at this same resolved root,
lazily, when a scene first needs a formula: `fmn` renders (`fmn batch`
included), `fmn::render`, and `fmn-python` scenes. The typeset cache lives in
the `ns/typeset/v<TYPESET_FORMAT_VERSION>` namespace. The portal reads
`directories.cache` from `custom_config.yml` before falling back to the
platform convention. An unavailable or refused root (the portal's config value
included) never fails a render: the scene typesets in memory and the render
record reports why (`cache_error`). A root that opens but then cannot read or
write (made read-only, replaced) is not silent either: each failed read or
write is counted (`store_errors`) and degrades to memory and a fresh layout.
Each `fmn` render record carries a `typesetting` object with that run's hits,
misses, bytes read and written, corrupt entries recomputed, store errors, and
the preflight's request count, workers and wall time. These are diagnostics,
never manifest inputs: a warm cache renders the same certified bits as a cold
one.

## Config file locations

The CLI first looks for one optional per-user config, using the native platform
convention:

| Platform | User config path |
|---|---|
| Linux and other Unix | `$XDG_CONFIG_HOME/franken-manim/config.yml` when `XDG_CONFIG_HOME` is set and absolute; otherwise `$HOME/.config/franken-manim/config.yml` |
| macOS | `$HOME/Library/Application Support/franken-manim/config.yml` |
| Windows | `%APPDATA%\franken-manim\config.yml` |

Environment paths remain native platform bytes rather than being lossy
Unicode-converted. A relative environment base is not trusted. When no
trustworthy absolute base exists, the user layer is simply absent: config
discovery never guesses a current-directory or temporary fallback.

The CLI then reads `custom_config.yml` from the current working directory
(Reference-exact behavior) and any file named by `--config_file`. All three
files are optional. A missing file is an empty layer; a file that exists but
cannot be read or parsed is a config error naming that exact layer.

## Precedence, end to end

```
built-in defaults
  → per-user platform config
  → ./custom_config.yml
  → --config_file <path>
  → CLI overlay (--cache-dir, --threads, --reproducible, …)
```

Later layers win key-by-key with the Reference's recursive merge; the
*resolved bytes* — not the file paths — are what enter the certified input
closure (C4).
