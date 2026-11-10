# Project-local Python environments

Scenes may live beside the environment that runs them, including the common
`project/.venv` layout. `SceneSource` refreshes authored source, not installed
runtime packages. A project path prefix is necessary but no longer sufficient
to own an import.

Runtime exclusions come from the shared `runtime_paths.library_roots` helper
(sysconfig stdlib/platstdlib/purelib/platlib, site installation/user directories,
and the portal package root). Import ownership additionally excludes nested
interpreter prefixes and paths with a complete `site-packages` or `dist-packages`
component. Symlinked installation/search roots and real targets are compared.
Names merely containing those strings, such as `site-packages-examples`, are
still ordinary project paths. No package-name-specific NumPy workaround is used.

These exclusions precede Studio's declared source table. Cached runtime modules
are not evicted; newly imported library modules keep their ordinary loaders and
remain loaded when the source context exits. Entirely installed namespaces stay
host-owned. Mixed project/foreign namespaces retain the existing explicit
refusal rather than partially reloading a shared namespace. A selected scene
file inside a runtime installation is refused; keep scenes outside the runtime.

Local relative, absolute sibling and lazy helpers still compile from their
current source bytes. Same-size edits with restored timestamps are observed.
Definition reload, embedded autoreload publication and failed-import restoration
reuse the same SceneSource boundary; they do not independently reload libraries.
Executed-source receipts include authored helpers, not virtualenv dependencies.
This is source freshness, not a sandbox or complete certified input closure.

The synchronous SceneProject watcher prunes runtime directories before charging
its file/directory/byte budgets, also pruning inactive environments identified by
`pyvenv.cfg`. Explicit runtime paths and file symlinks into installed code do not
trigger rebuilds. Explicit hidden *project assets* remain watched. The native
Studio SourceWatch traversal is not replaced by this Python watcher.

Effect auditing shares root discovery but deliberately does not inherit the
broader nested-prefix import exclusion: an entire project or `/usr` is not
exempted from recording reads merely because an interpreter is there. Ordinary
project asset reads remain content-hashed and changed inputs still refuse
certification.

## Acceptance

`test_runtime_source_ownership.py` executes filesystem/import-cache tests, with
small Scene and cached-extension metadata fixtures but no native-render claims.
`test_runtime_project_watch.py` exercises real scans, budgets and effect reads.
`project_virtualenv.py` copies the executing installed portal and NumPy payloads
into a stdlib-created, project-local venv without pip or network access. A child
runs the real console, renders Text/Square animation, compares one/four-thread
frames, observes same-size helper edits, and exercises SourceNamespace reload
and failure recovery while preserving NumPy's native-module identities.

The installed test prints bounded JSON with environment/interpreter paths,
ownership decisions, exit status and frame hashes. A retained-wheel run with
new Python source is evidence for that tested combination, not a fresh-tree
wheel or a full-workspace/cross-platform certification result. Some NumPy
versions merely warn when reexecuted; the explicit engine/scene module-identity
check detects the defect even on those versions.
