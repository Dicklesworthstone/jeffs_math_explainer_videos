"""Scene inspection and construct-only execution using the render source owner."""
from __future__ import annotations

import contextlib
import sys
from typing import Any

from .batch_rendering import _error_fields
from .console_rendering import _tokens
from .scene_loading import SceneSource

_CONTROLS = frozenset({"--list-scenes", "--construct-only"})


def try_scene_cli(native: Any, arguments: list[str]) -> int | None:
    """Handle source controls; leave help, version and Studio to their owner.

    No output generation is opened here. Robot stdout is reserved for exactly one
    terminal receipt, including failures in module code and interrupted scenes.
    """
    options, positionals, switches = _tokens(arguments)
    controls = _CONTROLS.intersection(switches)
    if not controls or (positionals and positionals[0] == "studio"):
        return None
    robot = "--robot" in switches
    if (len(controls) != 1 or switches.count("--robot") > 1
            or any(switches.count(control) > 1 for control in controls)):
        return native._portal_cli_emit(
            2, "usage", "usage-error", "select one scene control without repeated switches", robot,
        )
    control = next(iter(controls))
    if any(option not in {control, "--robot"} for option in options):
        return native._portal_cli_emit(
            2, "usage", "usage-error",
            f"{control} accepts only SOURCE.py and an optional scene name", robot,
        )
    constructing = control == "--construct-only"
    if len(positionals) not in ((1, 2) if constructing else (1,)):
        detail = ("requires SOURCE.py and accepts one optional SCENE" if constructing
                  else "requires exactly one SOURCE.py")
        return native._portal_cli_emit(2, "usage", "usage-error", f"{control} {detail}", robot)

    source = positionals[0]
    selected = positionals[1] if len(positionals) == 2 else None
    phase = "load"
    redirect = contextlib.redirect_stdout(sys.stderr) if robot else contextlib.nullcontext()
    try:
        with redirect, SceneSource(source, native.Scene) as loaded:
            scenes, names = loaded.scenes, sorted(loaded.scenes)
            if constructing:
                phase = "select"
                if selected is None:
                    if len(names) != 1:
                        raise ValueError("select one scene explicitly; discovered: "
                                         + (", ".join(names) or "none"))
                    selected = names[0]
                if selected not in scenes:
                    raise ValueError(f"scene {selected!r} was not declared by {source}; discovered: "
                                     + (", ".join(names) or "none"))
                phase = "construct"
                scene = scenes[selected]()
                phase = "execute"
                try:
                    scene.run()
                except native.EndScene:
                    pass
                phase = "inspect"
                roots, family, low, high = scene._engine_facts()
                details = {
                    "scene_time": float(scene.time),
                    "root_count": int(roots), "family_count": int(family),
                    "bounds_low": [float(value) for value in low],
                    "bounds_high": [float(value) for value in high],
                }
    except (KeyboardInterrupt, SystemExit) as error:
        code = 130 if isinstance(error, KeyboardInterrupt) else 5
        name, message = _error_fields(error)
        return native._portal_cli_emit(
            code, "interrupted", "construct-interrupted" if constructing else "scene-list-interrupted",
            f"{name}: {message}", robot, source=source, scene=selected, phase=phase, rendered=False,
        )
    except Exception as error:
        name, message = _error_fields(error)
        return native._portal_cli_emit(
            5, "scene", "construct-failed", f"{name}: {message}", robot,
            source=source, phase=phase, rendered=False,
        )

    # Emit only after authored stdout redirection and project imports unwind.
    if not constructing:
        return native._portal_cli_emit(
            0, "success", "scene-list", "\n".join(names) or "no Scene subclasses found",
            robot, source=source, scenes=names,
        )
    return native._portal_cli_emit(
        0, "success", "construct-only", f"constructed {selected} without rendering pixels",
        robot, source=source, scene=selected, rendered=False, **details,
    )
