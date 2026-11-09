"""InteractiveScene gestures over the existing native selection and SceneState.

No window, renderer, geometry store or history serializer lives here. The
worker's admitted events drive the Reference-named methods on the original
class. Transformations still use native Mobject/Group operations, and history
still uses the normal SceneState scope (not arbitrary Python-effect rollback).
"""
from __future__ import annotations

from itertools import product
from types import SimpleNamespace
from typing import Any

from .interaction import _method, input_key_pressed, input_modifiers


def install_interactive_editing(native: Any) -> None:
    g = vars(native)
    if g.get("_FMN_INTERACTIVE_EDITING_INSTALLED", False):
        return
    Scene, Interactive, np = g["Scene"], g["InteractiveScene"], g["_np"]
    primary = g["_PYGLET_MOD_CTRL"] | g["_PYGLET_MOD_COMMAND"]
    shift, control = g["_PYGLET_MOD_SHIFT"], g["_PYGLET_MOD_CTRL"]

    def keys():
        return g["_pinned_manim_config"]().key_bindings

    def selected(scene):
        return tuple(id(mob) for mob in scene.selection)

    def fixed_selection(scene):
        spaces = {mob.is_fixed_in_frame() for mob in scene.selection.family_members_with_points()}
        if len(spaces) > 1:
            raise ValueError("select world-space and fixed-frame objects separately before transforming")
        return spaces == {True}

    def pointer(scene, point, fixed):
        point = np.asarray(g["_vec3"](point))
        return scene.frame.to_fixed_frame_point(point) if fixed else point

    def remember(scene, gesture=None):
        if gesture is None or not gesture.saved:
            scene.save_state()
            # save_state deduplicates identical snapshots. A real new edit
            # still invalidates redo even when its pre-state was deduplicated.
            scene.redo_stack.clear()
            if gesture is not None:
                gesture.saved = True

    def cancel(scene):
        scene.__dict__.pop("_fmn_edit_gesture", None)
        scene.is_grabbing = False

    def current(scene, kind):
        gesture = scene.__dict__.get("_fmn_edit_gesture")
        if gesture is not None:
            visible = {id(member) for root in scene.mobjects for member in root.get_family()}
            if (gesture.members != selected(scene)
                    or not set(gesture.members).issubset(visible)
                    or gesture.fixed != fixed_selection(scene)):
                cancel(scene)
                return None
        return gesture if gesture is not None and gesture.kind == kind else None

    def prepare_grab(self):
        old = self.__dict__.get("_fmn_edit_gesture")
        fixed = fixed_selection(self)
        selection_center = (
            self.selection.get_center()
            if len(self.selection) and len(self.selection.get_all_points()) > 0
            else np.zeros(3)
        )
        self.mouse_to_selection = pointer(self, self.mouse_point.get_center(), fixed) - selection_center
        self._fmn_edit_gesture = SimpleNamespace(
            kind="grab", members=selected(self), saved=bool(old and old.saved),
            key=None, axis=None, fixed=fixed,
        )
        self.is_grabbing = True

    def handle_grabbing(self, point):
        gesture = current(self, "grab")
        if gesture is None or not len(self.selection):
            return
        desired = pointer(self, point, gesture.fixed) - self.mouse_to_selection
        delta = desired - self.selection.get_center()
        k = keys()
        # The key that opened a gesture is its owner; unrelated held keys and
        # auto-repeat cannot redirect it. Direct calls may use host key state.
        axis = gesture.axis
        if gesture.key is None:
            for index, key in enumerate((k.x_grab, k.y_grab, k.z_grab)):
                if input_key_pressed(self, ord(key)):
                    axis = index
                    break
        if axis is not None:
            delta = np.array([value if index == axis else 0.0 for index, value in enumerate(delta)])
        if not np.isfinite(delta).all():
            raise ValueError("grab must produce finite coordinates")
        if np.any(delta != 0):
            remember(self, gesture)
            self.selection.shift(delta)

    def prepare_resizing(self, about_corner=False):
        if not len(self.selection):
            cancel(self)
            return
        old = self.__dict__.get("_fmn_edit_gesture")
        fixed = fixed_selection(self)
        center, mouse = self.selection.get_center(), pointer(self, self.mouse_point.get_center(), fixed)
        self.scale_about_point = (self.selection.get_corner(center - mouse)
                                  if about_corner else center.copy())
        self.scale_ref_vect = mouse - self.scale_about_point
        self.scale_ref_width, self.scale_ref_height = self.selection.get_width(), self.selection.get_height()
        self._fmn_edit_gesture = SimpleNamespace(
            kind="resize", members=selected(self), saved=bool(old and old.saved),
            key=keys().resize, scales=np.ones(3), fixed=fixed,
        )
        self.is_grabbing = False

    def handle_resizing(self, point):
        gesture = current(self, "resize")
        if gesture is None:
            return
        vector = pointer(self, point, gesture.fixed) - self.scale_about_point
        reference = self.scale_ref_vect
        if input_modifiers(self) & control:
            target = np.ones(3)
            for dim in (0, 1):
                if abs(reference[dim]) > np.finfo(float).eps:
                    ratio = vector[dim] / reference[dim]
                    # Keep a reversible, noncollapsed geometry at the pivot;
                    # crossing it can reflect an axis, never divide by zero.
                    target[dim] = np.copysign(max(abs(ratio), 1e-6), ratio)
        else:
            length = float(np.linalg.norm(reference))
            if length <= np.finfo(float).eps:
                return
            target = np.full(3, max(float(np.linalg.norm(vector)) / length, 1e-6))
        if not np.isfinite(target).all() or np.any(np.abs(target) > 1e6):
            raise ValueError("resize scale must be finite and no larger than 1000000")
        ratios = target / gesture.scales
        if np.any(ratios != 1):
            remember(self, gesture)
            if np.all(ratios == ratios[0]) and ratios[0] > 0:
                self.selection.scale(float(ratios[0]), about_point=self.scale_about_point)
            else:
                for dim, ratio in enumerate(ratios):
                    if ratio != 1:
                        self.selection.stretch(float(ratio), dim, about_point=self.scale_about_point)
            gesture.scales = target

    def edit_motion(self, point):
        crosshair = getattr(self, "crosshair", None)
        if crosshair is not None:
            crosshair.move_to(self.frame.to_fixed_frame_point(point))
        gesture = self.__dict__.get("_fmn_edit_gesture")
        if gesture is not None:
            if gesture.kind == "grab":
                self.handle_grabbing(point)
            else:
                self.handle_resizing(point)
        elif getattr(self, "is_selecting", False):
            if input_modifiers(self) & shift:
                self._fmn_selection_swept = True
                self.handle_sweeping_selection(point)
            else:
                self.update_selection_rectangle(self.selection_rectangle)

    def on_mouse_motion(self, point, d_point):
        Scene.on_mouse_motion(self, point, d_point)
        edit_motion(self, point)

    def on_mouse_drag(self, point, d_point, buttons, modifiers):
        # Preserve ordinary camera drag when no editing gesture owns it.
        # Listeners already had their chance to consume the event in the
        # scene input gateway; editing never dispatches them a second time.
        if self.__dict__.get("_fmn_edit_gesture") is not None or getattr(self, "is_selecting", False):
            edit_motion(self, point)
        else:
            Scene.on_mouse_drag(self, point, d_point, buttons, modifiers)
            crosshair = getattr(self, "crosshair", None)
            if crosshair is not None:
                crosshair.move_to(self.frame.to_fixed_frame_point(point))

    def projected_bounds(scene, mob):
        # Transform ALL eight native bounding-box corners. Transforming only
        # min and max reverses/loses extents under a rotated camera. Fixed
        # objects already occupy the advertised frame plane; do not map twice.
        bounds = mob.get_bounding_box()
        corners = np.array(list(product(*zip(bounds[0], bounds[2]))))
        if not mob.is_fixed_in_frame():
            corners = np.array([scene.frame.to_fixed_frame_point(point) for point in corners])
        if not np.isfinite(corners).all():
            raise ValueError("selection requires finite projected bounds")
        return corners[:, :2].min(axis=0), corners[:, :2].max(axis=0)

    def regenerate_selection_search_set(self):
        excluded = {id(mob) for mob in getattr(self, "unselectables", ())}
        candidates, seen = [], set()
        for root in self.mobjects:
            if id(root) in excluded:
                continue
            members = [root] if self.select_top_level_mobs else root.family_members_with_points()
            for mob in members:
                if id(mob) not in excluded and id(mob) not in seen:
                    seen.add(id(mob))
                    candidates.append(mob)
        self.selection_search_set = candidates

    def gather_new_selection(self):
        self.is_selecting = False
        rectangle = self.selection_rectangle
        if rectangle not in self.mobjects:
            return
        self.remove(rectangle)
        bounds = rectangle.get_bounding_box()
        low, high = bounds[0, :2], bounds[2, :2]
        tap = bool(np.max(high - low) < 1e-2)
        additions = []
        for mob in reversed(self.get_selection_search_set()):
            start, end = projected_bounds(self, mob)
            if np.all(end >= low - 1e-2) and np.all(start <= high + 1e-2):
                additions.append(mob)
                if tap:
                    break
        self.toggle_from_selection(*additions)

    def handle_sweeping_selection(self, point):
        fixed = self.frame.to_fixed_frame_point(point)[:2]
        for mob in reversed(self.get_selection_search_set()):
            low, high = projected_bounds(self, mob)
            if np.all(fixed >= low - g["_SMALL_BUFF"]) and np.all(fixed <= high + g["_SMALL_BUFF"]):
                self.add_to_selection(mob)
                break

    def choose_color(self, point):
        fixed = self.frame.to_fixed_frame_point(point)[:2]
        # The palette is deliberately unselectable but must remain pickable.
        # Include its actual native swatches, not a duplicate color lookup.
        candidates = [member for root in self.mobjects
                      if root is self.color_palette or root not in self.unselectables
                      for member in root.family_members_with_points()]
        color = None
        for mob in reversed(candidates):
            low, high = projected_bounds(self, mob)
            if np.all(fixed >= low) and np.all(fixed <= high):
                color = mob.get_color()
                break
        # The palette is transient UI, not part of the authored edit to undo.
        self.remove(self.color_palette)
        if color is not None and len(self.selection):
            remember(self)
            self.selection.set_color(color)

    def toggle_color_palette(self):
        if not len(self.selection):
            return
        if self.color_palette in self.mobjects:
            self.remove(self.color_palette)
        else:
            self.save_state()
            self.add(self.color_palette)

    def on_key_press(self, symbol, modifiers):
        k = keys()
        try:
            char = chr(int(symbol))
        except (OverflowError, TypeError, ValueError):
            return
        modifiers = int(modifiers)
        ctrl = bool(modifiers & primary)
        grabs = (k.grab, k.x_grab, k.y_grab, k.z_grab)
        # Scene owns undo/redo, camera reset and presenter keys. Return after
        # undo/redo: ctrl-z must not create a new grab snapshot of its result.
        Scene.on_key_press(self, symbol, modifiers)
        if ctrl and char == "z":
            return
        if char in grabs and modifiers == 0:
            gesture = self.__dict__.get("_fmn_edit_gesture")
            if gesture is None or gesture.kind != "grab" or gesture.key != char:
                self.prepare_grab()
                gesture = current(self, "grab")
                if gesture is not None:
                    gesture.key = char
                    gesture.axis = None if char == k.grab else grabs.index(char) - 1
        elif char == k.resize and not ctrl:
            gesture = current(self, "resize")
            if gesture is None and (modifiers & shift):
                self.prepare_resizing(about_corner=True)
        elif char == k.select and not ctrl:
            if not self.is_selecting:
                cancel(self)
                self._fmn_selection_swept = False
                self.enable_selection()
                self.update_selection_rectangle(self.selection_rectangle)
            self.add(self.crosshair)
        elif char == k.unselect:
            cancel(self)
            self.clear_selection()
        elif char == k.color and modifiers == 0:
            self.toggle_color_palette()
        elif char == k.information and modifiers == 0:
            self.display_information()
        elif ctrl:
            cancel(self)
            if char == "a":
                self.clear_selection()
                self.add_to_selection(*self.get_selection_search_set())
            elif char == "g" and len(self.selection):
                remember(self)
                self.ungroup_selection() if modifiers & shift else self.group_selection()
            elif char == "t":
                self.toggle_selection_mode()
            elif char == "c":
                self.copy_selection()
            elif char == "v":
                self.paste_selection()
            elif char == "x" and len(self.selection):
                self.copy_selection()
                remember(self)
                self.delete_selection()
        elif symbol == g["_PYGLET_BACKSPACE"] and len(self.selection):
            cancel(self)
            remember(self)
            self.delete_selection()
        elif symbol in g["_PYGLET_ARROW_SYMBOLS"] and len(self.selection):
            cancel(self)
            fixed_selection(self)
            remember(self)
            vectors = (g["_LEFT"], g["_UP"], g["_RIGHT"], g["_DOWN"])
            self.nudge_selection(vectors[g["_PYGLET_ARROW_SYMBOLS"].index(symbol)], large=bool(modifiers & shift))
        elif char == "d" and modifiers & shift:
            self.copy_frame_positioning()
        elif char == "c" and modifiers & shift:
            self.copy_cursor_position()
        if char == k.cursor and not ctrl:
            if self.crosshair in self.mobjects:
                self.remove(self.crosshair)
            else:
                self.add(self.crosshair)
        if (char in grabs or char == k.resize) and not ctrl:
            self.save_state()

    def on_key_release(self, symbol, modifiers):
        Scene.on_key_release(self, symbol, modifiers)
        try:
            char = chr(int(symbol))
        except (OverflowError, TypeError, ValueError):
            return
        gesture = self.__dict__.get("_fmn_edit_gesture")
        if gesture is not None and gesture.key == char:
            cancel(self)
        k = keys()
        if char == k.select:
            # Sweeping is additive. Reapplying the marquee toggle would
            # unselect the very objects collected during this gesture.
            if self.__dict__.pop("_fmn_selection_swept", False):
                self.is_selecting = False
                self.remove(self.selection_rectangle)
            elif self.is_selecting:
                self.gather_new_selection()
        elif char == k.information:
            self.display_information(False)

    def get_state(self):
        # Host-owned notebooks and consoles can edit a Scene before run/setup.
        # Snapshot only the overlays which actually exist; taking a checkpoint
        # must not initialize UI, run authored setup or change drawable roots.
        overlays = [getattr(self, name, None) for name in (
            "selection_highlight", "selection_rectangle", "crosshair",
        )]
        return g["SceneState"](self, ignore=[mob for mob in overlays if mob is not None])

    def restore_state(self, scene_state):
        # Use the existing Scene/SceneState authority even before interactive
        # widgets have been initialized. Do not call setup as a repair: authored
        # setup may allocate content, consume RNG, or overwrite user objects.
        Scene.restore_state(self, scene_state)
        highlight = getattr(self, "selection_highlight", None)
        if highlight is not None:
            self.add(highlight)
            self.bring_to_back(highlight)
        cancel(self)
        self.is_selecting = False
        self.__dict__.pop("_fmn_selection_swept", None)
        self.clear_selection()
        self.regenerate_selection_search_set()

    for name, method in {
        "prepare_grab": prepare_grab, "handle_grabbing": handle_grabbing,
        "prepare_resizing": prepare_resizing, "handle_resizing": handle_resizing,
        "on_mouse_motion": on_mouse_motion, "on_mouse_drag": on_mouse_drag,
        "on_key_press": on_key_press, "on_key_release": on_key_release,
        "get_state": get_state, "restore_state": restore_state,
        "regenerate_selection_search_set": regenerate_selection_search_set,
        "gather_new_selection": gather_new_selection,
        "handle_sweeping_selection": handle_sweeping_selection,
        "choose_color": choose_color, "toggle_color_palette": toggle_color_palette,
    }.items():
        _method(Interactive, name, method)
    g["_FMN_INTERACTIVE_EDITING_INSTALLED"] = True
