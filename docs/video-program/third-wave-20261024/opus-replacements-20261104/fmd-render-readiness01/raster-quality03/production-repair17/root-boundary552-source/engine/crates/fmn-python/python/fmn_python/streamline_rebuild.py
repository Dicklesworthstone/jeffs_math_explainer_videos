"""Rebuild live flow families through the existing detached native constructor."""
from __future__ import annotations

import inspect

from .streamline_authoring import _editing, _MAX_POINTS


def install_streamline_rebuild(native):
    g = vars(native)
    if g.get("_FMN_STREAMLINE_REBUILD_INSTALLED", False):
        return
    Lines, np = g["StreamLines"], g["_np"]
    initialize = Lines.__init__
    signature = inspect.signature(initialize)

    def draw_lines(self):
        """Reintegrate current controls, publishing only a complete native family.

        The StreamLines container, its scene membership and its updaters stay
        live. Individual curves are replaced, as in the Reference. A running
        animation owns its old curves: finish/clear it before rebuilding.
        """
        with _editing(self):
            def idle():
                if any(member._is_updating_suspended() or vars(member).get("_is_animating", False)
                       for member in self.get_family()):
                    raise RuntimeError("StreamLines.draw_lines requires released animations and updater suspension")
            idle()
            members = tuple(self.submobjects)
            if sum(member.get_num_points() for member in members) > _MAX_POINTS:
                raise ValueError("StreamLines rebuild exceeds the 65536-point resource budget")
            records = [member.data.copy() for member in members]
            # Read every control before invoking the field. Mutable authored
            # range lists must not change the candidate halfway through a build.
            controls = {name: getattr(self, name) for name in signature.parameters
                        if name not in ("self", "kwargs")}
            controls["magnitude_range"] = tuple(controls["magnitude_range"])
            for name in ("density", "solution_time", "dt", "arc_len", "cutoff_norm",
                         "stroke_width", "stroke_opacity"):
                controls[name] = float(controls[name])
            if controls["noise_factor"] is not None:
                controls["noise_factor"] = float(controls["noise_factor"])
            for name in ("color_by_magnitude", "taper_stroke_width"):
                controls[name] = bool(controls[name])
            # Never reconstruct self: native builders are detached-only and
            # reset live state. The seed adapter, when installed, supplies the
            # candidate through the same native solver and live style planner.
            builder = g.get("_fmn_streamline_candidate")
            if builder is None:
                candidate = Lines.__new__(Lines)
                initialize(candidate, **controls)
            else:
                candidate = builder(self, controls)
            idle()
            if tuple(self.submobjects) != members or any(
                not np.array_equal(member.data, record)
                for member, record in zip(members, records)
            ):
                raise RuntimeError("StreamLines family changed during integration")
            children = list(candidate.submobjects)
            times = list(candidate._stream_virtual_times)
            draws = candidate._stream_rng_draws
            # Remove scratch parent back-edges before adoption. Slice assignment
            # uses Marionette's prevalidated family replacement; clear/add would
            # expose an empty live family if adoption were refused.
            candidate.submobjects.clear()
            self.submobjects[:] = children
            self._stream_virtual_times, self._stream_rng_draws = times, draws

    draw_lines.__module__ = Lines.__module__
    draw_lines.__qualname__ = Lines.__qualname__ + ".draw_lines"
    Lines.draw_lines = draw_lines
    g["_FMN_STREAMLINE_REBUILD_INSTALLED"] = True
