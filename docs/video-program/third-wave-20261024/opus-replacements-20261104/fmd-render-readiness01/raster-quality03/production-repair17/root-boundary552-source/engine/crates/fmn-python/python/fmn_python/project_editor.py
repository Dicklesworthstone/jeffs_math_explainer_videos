"""Full scene reconstruction inside the existing host IPython editor.

A fresh Scene is prepared by SceneProject before replacing editor bindings.
Source-only autoreload remains separate. No shell is started during a project
build: an authored embed marks the end of construction and contributes locals.
"""
from __future__ import annotations

import importlib
import sys
from types import BuiltinMethodType, MethodType

from .embedded_shell import _classes, _caller_namespace, _method, _SESSION
from .scene_console import SceneConsole, _OWNER, _STUDIO_WORKER, _note
from .scene_project import SceneProject, _BUILDING, _PROJECT, _idle
from .source_autoreload import _SHELL_NAMES, _STATE as _AUTO_STATE, _exports

_MISSING = object()
_FIELDS = ("scene", "checkpoint_manager", "_fmn_console", "_fmn_namespace")
_HELP = """Rebuildable host scene editor:
  fmn-python edit [--watch [--kitty | --sixel]] SOURCE.py SCENE

Run the selected scene and enter its host IPython editor. An authored embed()
ends construction and exposes its local names; no nested terminal is opened.
At the prompt: reload() reconstructs the scene from current files; preview()
returns a native image; reload_source()/auto_reload() refresh definitions only.
--watch enables full-scene rebuilding from the first prompt. At the prompt,
auto_rebuild() reconstructs saved changes while the terminal prompt is idle,
or before the next cell in simple-prompt/GUI fallback mode. stop_auto_rebuild()
disables it. Asset paths, debounce, idle and poll_interval options are available.
--kitty/--sixel require --watch and a compatible terminal; each successful build
shows its native snapshot immediately. preview(protocol="kitty") displays the
current frame explicitly. No terminal protocol is guessed or silently changed.
The two automatic reload modes are mutually exclusive; no background thread
executes a scene. Unchanged sources preserve interactive edits and checkpoints.
A successful rebuild resets scene state and checkpoints. Failed imports,
construction or preview retain the last working generation. Existing aliases
to old objects remain old aliases. No output file is opened by the editor.
Requires host IPython and an interactive terminal; not available in robot mode.
"""


def _same(first, second):
    if first is second:
        return True
    if type(first) is MethodType and type(second) is MethodType:
        return first.__self__ is second.__self__ and first.__func__ is second.__func__
    if type(first) is BuiltinMethodType and type(second) is BuiltinMethodType:
        return first.__self__ is second.__self__ and first.__name__ == second.__name__
    return False


def _replacement(namespace, previous, values, project, scene):
    result = dict(namespace)
    owned = set().union(*(table.keys() for table in previous))
    for name in owned | values.keys():
        if name.startswith("__") or name in _SHELL_NAMES:
            continue
        current = namespace.get(name, _MISSING)
        if current is _MISSING or any(_same(current, table.get(name, _MISSING)) for table in previous):
            if name in values:
                result[name] = values[name]
            else:
                result.pop(name, None)
    # These are the editor's current generation, never a stale user alias.
    result.update(self=scene, scene=scene, project=project)
    return result


def _check_editor(embedded, project):
    project._check()
    if project._editor is not embedded or not embedded._fmn_launching:
        raise RuntimeError("scene reconstruction requires its active project editor")
    if _STUDIO_WORKER.get():
        raise RuntimeError("Studio worker scenes rebuild through their supervisor")
    if embedded.scene is not project.scene:
        raise RuntimeError("editor and project no longer share a scene generation")
    console = embedded._fmn_console
    if not isinstance(console, SceneConsole) or console._closed or console.scene is not project.scene:
        raise RuntimeError("scene reconstruction requires the current owned scene console")
    _idle(project.scene, console=console)
    shell = embedded.shell
    if shell is None or console.shell is not shell or type(shell.user_ns) is not dict:
        raise RuntimeError("scene reconstruction requires its active shell namespace")
    identity = vars(embedded).get("_fmn_project_shell")
    if identity is None or shell is not identity[0]:
        raise RuntimeError("scene reconstruction cannot replace its owning shell module")
    user_module = shell.user_module
    module_dict = getattr(user_module, "__dict__", None)
    underlying = getattr(module_dict, "_module_dict", None)
    if user_module is not identity[1] and underlying is not getattr(identity[1], "__dict__", None):
        raise RuntimeError("scene reconstruction cannot replace its owning shell module")
    return console


def _reload_editor(embedded, project, *, if_changed=False):
    old_console = _check_editor(embedded, project)
    shell, old_scene = embedded.shell, project.scene
    old_shortcuts = dict(embedded.get_shortcuts())
    # Include definition-only reloads published since this scene was built.
    previous = (project.namespace, dict(vars(project._source.module)), old_shortcuts)
    old_fields = {name: vars(embedded).get(name, _MISSING) for name in _FIELDS}
    old_session = vars(old_scene).get(_SESSION, _MISSING)
    old_namespaces, new_console = [], None
    auto = vars(embedded).get(_AUTO_STATE)
    auto_snapshots = []
    if auto is not None:
        auto.check()
        if auto.source is not None and auto.source is not project._source:
            raise RuntimeError("autoreload belongs to another source context")
        auto_snapshots.append((auto, "previous", auto.previous))
        for bindings in (auto.bindings, auto.global_bindings):
            if bindings is not None:
                auto_snapshots.extend(((bindings, "previous", bindings.previous),
                                       (bindings, "module", bindings.module)))

    def restore_editor():
        for owner, name, value in auto_snapshots:
            setattr(owner, name, value)
        for namespace, snapshot in old_namespaces:
            namespace.clear()
            namespace.update(snapshot)
        for name, value in old_fields.items():
            if value is _MISSING:
                vars(embedded).pop(name, None)
            else:
                vars(embedded)[name] = value
        if old_session is not _MISSING:
            vars(old_scene)[_SESSION] = old_session
        else:
            vars(old_scene).pop(_SESSION, None)

    def prepare(candidate):
        nonlocal new_console
        new_console = SceneConsole(candidate.scene, clipboard=embedded.clipboard, shell=shell,
                                   capture=old_console.capture,
                                   max_checkpoints=old_console.max_checkpoints,
                                   max_source_bytes=old_console.max_source_bytes,
                                   _native=project._native)
        new_console.__enter__()
        # Run authored shortcut getters before committing editor state. Native
        # imports/lifecycle are still inside the source rollback transaction.
        try:
            vars(embedded).update(scene=candidate.scene, checkpoint_manager=new_console.checkpoint_manager,
                                  _fmn_console=new_console)
            shortcuts = dict(embedded.get_shortcuts())
        finally:
            for name, value in old_fields.items():
                if value is _MISSING:
                    vars(embedded).pop(name, None)
                else:
                    vars(embedded)[name] = value
        values = dict(candidate.namespace)
        values.update(shortcuts)
        namespaces = [shell.user_ns]
        if vars(shell.user_module) is not shell.user_ns:
            namespaces.append(vars(shell.user_module))
        prepared = [(namespace, _replacement(namespace, previous, values, project, candidate.scene))
                    for namespace in namespaces]
        old_namespaces.extend((namespace, dict(namespace)) for namespace in namespaces)
        # No authored hook is called beyond this point. Keep the same module
        # and dict objects: cell-defined functions hold the module globals.
        for namespace, replacement in prepared:
            namespace.clear()
            namespace.update(replacement)
        new_console.namespace = shell.user_ns
        new_console._bound_shell = shell
        vars(embedded).update(scene=candidate.scene, checkpoint_manager=new_console.checkpoint_manager,
                              _fmn_console=new_console, _fmn_namespace=dict(candidate.namespace))
        if vars(old_scene).get(_SESSION) is embedded:
            vars(old_scene).pop(_SESSION)
        vars(candidate.scene)[_SESSION] = embedded
        # The next pre-cell autoreload must compare against the just-published
        # source names, not mistake them for interactive overrides of the old
        # generation. Its shell/module/source identities remain unchanged.
        if auto is not None:
            auto.previous = _exports(candidate.module)
            for bindings in (auto.bindings, auto.global_bindings):
                if bindings is not None:
                    bindings.previous = dict(auto.previous)
                    bindings.module = candidate.module

    try:
        project._rebuild(if_changed=if_changed, prepare=prepare)
    except BaseException as error:
        restore_editor()
        if new_console is not None:
            candidate_scene = new_console.scene
            if candidate_scene is not None and vars(candidate_scene).get(_SESSION) is embedded:
                vars(candidate_scene).pop(_SESSION)
            try:
                new_console.close()
            except BaseException as cleanup_error:
                _note(error, "candidate console cleanup also failed: " + type(cleanup_error).__name__)
        raise
    if new_console is None:
        # A conditional no-op must retain the live console and checkpoints.
        return project.scene
    # Old checkpoints belong to the old arena. They are never applied to the
    # new scene. If cleanup fails after commit, report that fact without
    # rolling back the now-active scene/module/preview into a mixed generation.
    try:
        old_console.close()
    except BaseException as error:
        _note(error, "new scene generation is active; previous console cleanup failed after rebuild")
        raise
    return project.scene


def edit_project(project: SceneProject, *, clipboard=None, auto_rebuild=False, debounce=0.0, paths=(),
                 idle=None, poll_interval=0.25, preview_protocol=None):
    project._check()
    if type(auto_rebuild) is not bool:
        raise TypeError("auto_rebuild must be bool")
    from .project_autorebuild import _watch_options
    options = _watch_options(debounce=debounce, paths=paths, idle=idle,
                             poll_interval=poll_interval, preview_protocol=preview_protocol)
    if not auto_rebuild and (options["debounce"] != 0 or options["paths"] or idle is not None
                             or poll_interval != 0.25 or preview_protocol is not None):
        raise ValueError("watch options require auto_rebuild=True")
    if preview_protocol is not None:
        from .terminal_preview import _validate_project_preview
        _validate_project_preview(project, preview_protocol)
    if project._editor is not None:
        raise RuntimeError("this scene project already has an editor")
    if clipboard is not None and not callable(clipboard):
        raise TypeError("project editor clipboard must be callable or None")
    _idle(project.scene)
    _, Embedded = _classes(project._native)
    # Installed manimlib forwards private runtime attributes to its extension
    # via __getattr__; vars(package) is not the native initialization table.
    if getattr(project._native, "_FMN_SCENE_PROJECT_EDITOR_INSTALLED", False) is not True:
        raise ImportError("scene project editor is not installed in the native runtime")
    embedded = Embedded(project.scene)
    embedded._fmn_namespace = project.namespace
    embedded.clipboard = clipboard
    vars(embedded)[_PROJECT] = project
    if auto_rebuild:
        vars(embedded)["_fmn_project_autorebuild_options"] = options
    project._editor = embedded
    try:
        return embedded.launch()
    finally:
        project._editor = None
        vars(embedded).pop(_PROJECT, None)


def install_scene_project_editor(native):
    """Complete the existing Embedded class after definition-autoreload hooks."""
    if getattr(native, "_FMN_SCENE_PROJECT_EDITOR_INSTALLED", False) is True:
        return
    _, Embedded = _classes(native)
    original_launch, original_reload = Embedded.launch, Embedded.reload_scene
    original_shortcuts = Embedded.get_shortcuts
    original_update = Embedded.ensure_frame_update_post_cell
    original_scene_embed = getattr(native.Scene, "embed", None)

    def launch(self):
        building = _BUILDING.get()
        if building is not None:
            building._check_thread()
            if self.scene is not building._candidate_scene:
                raise RuntimeError("embedding is only allowed for the scene currently under construction")
            building._candidate_locals = dict(
                _caller_namespace() if self._fmn_namespace is None else self._fmn_namespace
            )
            # Native Scene.run already treats EndScene as normal completion
            # and owns teardown. Never run an IPython terminal inside a build.
            raise native.EndScene()
        return original_launch(self)

    def scene_embed(self, close_scene_on_exit=True, show_animation_progress=False):
        # Reference Scene.embed is a headless no-op outside an owned authoring
        # project. Inside one, the ordinary spelling must mark the construction
        # checkpoint and expose the caller's locals just like InteractiveScene.
        # Reuse Embedded.launch's build guard: never launch a terminal in run().
        if _BUILDING.get() is None:
            return original_scene_embed(self, close_scene_on_exit, show_animation_progress)
        embedded = Embedded(self)
        embedded._fmn_namespace = _caller_namespace()
        return embedded.launch()

    def ensure_update(self):
        result = original_update(self)
        if vars(self).get(_PROJECT) is not None:
            vars(self)["_fmn_project_shell"] = (self.shell, self.shell.user_module)
        return result

    def reload_scene(self, embed_line=None, *, if_changed=False):
        if type(if_changed) is not bool:
            raise TypeError("if_changed must be bool")
        project = vars(self).get(_PROJECT)
        if project is None:
            if if_changed:
                raise native._CapabilityError("conditional reload requires a SceneProject editor")
            return original_reload(self, embed_line)
        project._check_thread()
        if embed_line is not None:
            raise native._CapabilityError("project reload does not insert source lines; place embed() in the scene")
        if vars(self).get("_fmn_project_reloading", False):
            raise RuntimeError("project editor reconstruction is already in progress")
        vars(self)["_fmn_project_reloading"] = True
        try:
            return _reload_editor(self, project, if_changed=if_changed)
        finally:
            vars(self).pop("_fmn_project_reloading", None)

    def preview(self, *, protocol=None):
        project = vars(self).get(_PROJECT)
        if project is None:
            raise native._CapabilityError("project preview requires an active SceneProject editor")
        console = _check_editor(self, project)
        if protocol is not None:
            from .terminal_preview import _protocol, _snapshot, _stream, _show_snapshot
            _protocol(protocol)
            _stream(None)
            _snapshot(project._native, project.preview)
        snapshot = console.preview()
        if protocol is not None:
            _show_snapshot(snapshot, protocol, None, 16_777_216, project._native)
        return snapshot

    def shortcuts(self):
        result = dict(original_shortcuts(self))
        project = vars(self).get(_PROJECT)
        if project is not None:
            result.update(project=project, reload=self.reload_scene, preview=self._project_preview)
        return result

    for name, function in (("launch", launch), ("reload_scene", reload_scene),
                           ("ensure_frame_update_post_cell", ensure_update),
                           ("get_shortcuts", shortcuts), ("_project_preview", preview)):
        _method(Embedded, name, function)
    if original_scene_embed is not None:
        _method(native.Scene, "embed", scene_embed)
    from .project_autorebuild import install_project_autorebuild
    install_project_autorebuild(native)
    native._FMN_SCENE_PROJECT_EDITOR_INSTALLED = True


def try_edit_cli(native, arguments):
    """Own only the explicit edit command; ordinary rendering stays unchanged."""
    tokens = [arg for arg in arguments if arg != "--robot"]
    if not tokens or tokens[0] != "edit":
        return None
    robot = "--robot" in arguments
    preview_flags = [arg for arg in tokens[1:] if arg in ("--kitty", "--sixel")]
    if len(preview_flags) > 1:
        return native._portal_cli_emit(2, "usage", "usage-error", "choose at most one of --kitty or --sixel", robot)
    preview_protocol = preview_flags[0][2:] if preview_flags else None
    tokens = [tokens[0], *(arg for arg in tokens[1:] if arg not in ("--kitty", "--sixel"))]
    watch = "--watch" in tokens[1:]
    if tokens[1:].count("--watch") > 1:
        return native._portal_cli_emit(2, "usage", "usage-error", "--watch may be supplied only once", robot)
    tokens = [tokens[0], *(arg for arg in tokens[1:] if arg != "--watch")]
    if tokens[1:] in (["--help"], ["-h"]):
        if robot:
            return native._portal_cli_emit(0, "success", "help", "fmn-python edit usage", True, help=_HELP)
        print(_HELP)
        return 0
    if preview_protocol is not None and not watch:
        return native._portal_cli_emit(2, "usage", "usage-error", "--kitty/--sixel require --watch", robot)
    if len(tokens) != 3 or any(arg.startswith("-") for arg in tokens[1:]):
        return native._portal_cli_emit(2, "usage", "usage-error", "expected: fmn-python edit [--watch] SOURCE.py SCENE", robot)
    if robot or not getattr(sys.stdin, "isatty", lambda: False)():
        return native._portal_cli_emit(4, "capability", "edit-capability-unavailable",
                                       "edit requires an interactive terminal; use SceneProject for host-controlled builds", robot)
    if preview_protocol is not None:
        from .terminal_preview import _encoding_type, _stream
        try:
            _encoding_type(native)
            _stream(None)
        except Exception as error:
            return native._portal_cli_emit(4, "capability", "edit-capability-unavailable", str(error), False)
    try:
        importlib.import_module("IPython.terminal.embed")
    except ImportError:
        return native._portal_cli_emit(4, "capability", "edit-capability-unavailable",
                                       "edit requires IPython in its host interpreter", False)
    try:
        with SceneProject(tokens[1], tokens[2], _native=native) as project:
            if watch:
                if preview_protocol is None:
                    project.edit(auto_rebuild=True)
                else:
                    project.edit(auto_rebuild=True, preview_protocol=preview_protocol)
            else:
                project.edit()
            generations = project.generation
    except (KeyboardInterrupt, SystemExit) as error:
        code = 130 if isinstance(error, KeyboardInterrupt) else 5
        return native._portal_cli_emit(code, "interrupted", "edit-interrupted", "scene editing interrupted", False)
    except Exception as error:
        try:
            message = str(error)[:2048]
        except Exception:
            message = type(error).__name__
        return native._portal_cli_emit(5, "scene", "scene-edit-failed", message, False)
    return native._portal_cli_emit(0, "success", "edit", "scene editor closed", False,
                                   source=tokens[1], scene=tokens[2], generations=generations)
