"""Runtime content verification at the existing native publication boundary.

The one RenderSession remains the generation owner. This adapter snapshots
installed runtime inputs before native acquisition and verifies them after
source-provider callbacks and before publishing the provenance sidecar.
It also records the scene's effects (effect_audit) from acquisition to
publication. There is no second renderer, sink, clock, or configuration
parser.
"""
from __future__ import annotations

from functools import wraps
import weakref

from . import effect_audit
from .runtime_identity import RuntimeIdentityError, capture_runtime


def _install_session_guard(rendering):
    Session = rendering.RenderSession
    if vars(Session).get("_fmn_runtime_provenance_installed", False):
        return
    original_enter, original_finish, original_abort = Session.__enter__, Session.finish, Session.abort
    legacy_labels = rendering._runtime_identities

    def capability(native):
        return getattr(native, "_CapabilityError", RuntimeError)

    def stop_effects(session):
        effects = vars(session).pop("_fmn_scene_effects", None)
        if effects is not None:
            effects.stop()

    @wraps(original_enter)
    def enter(self):
        self._check_owner()
        if (not self.reproducible or self._state != "new"
                or getattr(self.scene, "_fmn_owned_render_session", None) is not None):
            return original_enter(self)
        try:
            # Hashing the runtime is not a scene effect, even while a
            # certified CLI invocation is recording its scene module.
            with effect_audit.paused():
                snapshot = capture_runtime(self._native)
            measured = snapshot.identities
            supplied = self.runtime_identities
            if supplied is not None and supplied != measured and supplied != legacy_labels(self._native):
                raise RuntimeIdentityError("supplied runtime identities disagree with the installed runtime")
        except RuntimeIdentityError as error:
            raise capability(self._native)(
                "CAPABILITY: portal runtime input closure unavailable: " + str(error)
            ) from error
        # The old console supplies version labels. Accept that spelling for
        # compatibility, but ALWAYS replace it with measured payload digests.
        # Arbitrary caller-supplied identifiers can no longer certify a build.
        self.runtime_identities = measured
        effects = effect_audit.begin()
        # An abandoned session must not record for the rest of the process.
        weakref.finalize(self, effects.stop)
        self._fmn_scene_effects = effects
        try:
            # Refuse an invocation whose scene module already did something
            # uncapturable before rendering anything.
            effects.check(capability(self._native))
            result = original_enter(self)
        except BaseException:
            stop_effects(self)
            raise
        self._fmn_runtime_snapshot = snapshot
        return result

    @wraps(original_abort)
    def abort(self):
        try:
            return original_abort(self)
        finally:
            if self._state != "active":
                stop_effects(self)

    @wraps(original_finish)
    def finish(self):
        self._check_owner()
        if not self.reproducible or self._state != "active" or self._finishing:
            return original_finish(self)
        snapshot = getattr(self, "_fmn_runtime_snapshot", None)
        if snapshot is None:
            error = RuntimeIdentityError("certified generation has no acquired runtime input snapshot")
            self._cancel_preserving(error)
            raise error
        provider, publisher = self.sources, self._publish_manifest
        effects = vars(self).get("_fmn_scene_effects")
        refused = capability(self._native)

        def checked_sources():
            # Declaring sources is not a scene effect.
            with effect_audit.paused():
                values = provider() if callable(provider) else provider
                # Freeze the returned mapping too: custom Mapping iteration can
                # itself run Python and change runtime files. Check after it ends.
                values = rendering._source_snapshot(values)
                snapshot.verify()
            self.runtime_identities = snapshot.identities
            # Refuse uncapturable effects, and inputs changed since the scene
            # read them, while the generation can still be cancelled.
            if effects is not None:
                effects.inputs(refused)
            return values

        def checked_manifest(destination, format, resolution, fps, threads, seed,
                             artifact_report, sources, runtime_identities, cue_assets=None):
            # Native finish may capture a static final frame and invoke camera
            # hooks AFTER checked_sources. Detect edits there before sidecar
            # publication. RenderSession reports artifact_published=True on a
            # failure here; do not pretend the native artifact was rolled back.
            with effect_audit.paused():
                snapshot.verify()
            if runtime_identities != snapshot.identities:
                raise RuntimeIdentityError("runtime provenance was changed during native finalization")
            if effects is None:
                raise RuntimeIdentityError("certified generation has no scene effect record")
            effects.stop()
            scene_reads = effects.inputs(refused)
            return publisher(destination, format, resolution, fps, threads, seed,
                             artifact_report, sources, snapshot.identities, cue_assets,
                             **({"scene_reads": scene_reads} if scene_reads else {}))

        self.sources, self._publish_manifest = checked_sources, checked_manifest
        try:
            return original_finish(self)
        finally:
            self.sources, self._publish_manifest = provider, publisher
            stop_effects(self)

    Session.__enter__, Session.finish, Session.abort = enter, finish, abort
    Session._fmn_runtime_provenance_installed = True


def install_runtime_provenance(native):
    """Enable the same guarded session from the free functions, CLI and Scene."""
    if vars(native).get("_FMN_RUNTIME_PROVENANCE_INSTALLED", False):
        return
    from . import rendering

    _install_session_guard(rendering)
    Scene = native.Scene

    def scene_render_session(self, destination=None, *, format=None, resolution=None,
                             fps=None, threads=None, animation_range=None,
                             reproducible=False, sources=None, runtime_identities=None):
        return rendering._configured_scene_session(
            self, destination, format, resolution, fps, threads, animation_range, native,
            reproducible=reproducible, sources=sources, runtime_identities=runtime_identities,
        )

    def render(self, destination=None, *, format=None, resolution=None, fps=None,
               threads=None, animation_range=None, reproducible=False,
               sources=None, runtime_identities=None):
        session = scene_render_session(
            self, destination, format=format, resolution=resolution, fps=fps,
            threads=threads, animation_range=animation_range, reproducible=reproducible,
            sources=sources, runtime_identities=runtime_identities,
        )
        return rendering._run_owned_scene_render(self, session, native)

    for name, function in (("render", render), ("render_session", scene_render_session)):
        function.__name__ = name
        function.__qualname__ = Scene.__qualname__ + "." + name
        function.__module__ = Scene.__module__
        setattr(Scene, name, function)
    native._FMN_RUNTIME_PROVENANCE_INSTALLED = True
