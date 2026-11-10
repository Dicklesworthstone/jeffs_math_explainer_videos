"""Scene-scoped input on the existing event, control and native geometry APIs.

This is the host input adapter, not a window or a Studio transport. Direct
EventDispatcher calls remain standalone; Scene callbacks select a live scene
family and keep pointer/key/capture state isolated from other scenes.
"""
from __future__ import annotations

from contextvars import ContextVar
from functools import wraps
import inspect
import sys
from types import SimpleNamespace
from typing import Any
import weakref


_INPUT = ContextVar("fmn_scene_input", default=None)
_LIVE_EVENT = ContextVar("fmn_live_event", default=None)


def dispatch_live_input(scene, method, arguments, modifiers):
    """Deliver one admitted native event without changing Reference signatures.

    Motion and scroll methods do not take modifier bits in manimlib. Keep the
    wire's authoritative snapshot in a scoped context rather than guessing from
    modifier-key presses (browsers do not always deliver those). Nested delivery
    and exceptions restore the previous context; no state crosses scenes.
    """
    if method not in {"on_mouse_motion", "on_mouse_drag", "on_mouse_press",
                      "on_mouse_release", "on_mouse_scroll", "on_key_press",
                      "on_key_release"}:
        raise ValueError("unsupported live Scene input method")
    if type(modifiers) is not int or modifiers < 0 or modifiers & ~0x47:
        raise ValueError("invalid live input modifier bits")
    token = _LIVE_EVENT.set((scene, modifiers))
    try:
        return getattr(scene, method)(*arguments)
    finally:
        _LIVE_EVENT.reset(token)


def input_modifiers(scene):
    """Current scene modifier snapshot, with a real host window taking precedence."""
    window = scene.get_window()
    if window is not None:
        return sum(bit for bit, keys in ((1, (0xffe1, 0xffe2)),
                   (2, (0xffe3, 0xffe4)), (4, (0xffe9, 0xffea)),
                   (64, (0xffeb, 0xffec)))
                   if any(window.is_key_pressed(key) for key in keys))
    live = _LIVE_EVENT.get()
    if live is not None and live[0] is scene:
        return live[1]
    entry = scene.__dict__.get("_fmn_input_state")
    return getattr(entry[1], "modifiers", 0) if entry is not None else 0


def input_key_pressed(scene, symbol):
    """Use existing window or scene-scoped dispatcher state, never global keys."""
    window = scene.get_window()
    if window is not None:
        return bool(window.is_key_pressed(symbol))
    entry = scene.__dict__.get("_fmn_input_state")
    return entry is not None and symbol in entry[1].pressed_keys


def _method(cls, name, function):
    function.__name__ = name
    function.__qualname__ = cls.__qualname__ + "." + name
    function.__module__ = cls.__module__
    setattr(cls, name, function)


def _state(dispatcher):
    context = _INPUT.get()
    if context is None:
        return dispatcher
    scene = context.scene
    entry = scene.__dict__.get("_fmn_input_state")
    if entry is None or entry[0]() is not dispatcher:
        state = SimpleNamespace(
            mouse_point=dispatcher.mouse_point.copy(),
            mouse_drag_point=dispatcher.mouse_drag_point.copy(),
            pressed_keys=set(), draggable_object_listners=[], modifiers=0,
        )
        scene.__dict__["_fmn_input_state"] = (weakref.ref(dispatcher), state)
        return state
    return entry[1]


def _in_scene(listener):
    context = _INPUT.get()
    if context is None:
        return True
    target = listener.mobject
    # A detached listener explicitly added to the global dispatcher (rather
    # than through Mobject's local registration API) is a compatibility-level
    # global interceptor. Bound objects and locally registered controls are
    # always subject to current scene membership.
    if (getattr(target, "_scene", None) is None
            and not any(listener is item for item in target.event_listners)):
        return True
    # Use current drawable membership, not the persistent arena-owner field:
    # removing a control from a scene does not unbind its native allocation.
    return any(target is member for root in context.scene.mobjects
               for member in root.get_family())


def _registered(dispatcher, listener, event_type):
    return any(listener is item for item in dispatcher.event_listners[event_type])


def _install_dispatcher(g):
    # The pinned schema exposes EventDispatcher in its owning module, not at
    # the manimlib root. Patch that exact class (also used by EVENT_DISPATCHER)
    # without broadening the root's public exports. Storage doubles may supply
    # the class directly in their isolated namespace.
    Dispatcher = g.get("EventDispatcher")
    if Dispatcher is None:
        module = sys.modules.get("manimlib.event_handler.event_dispatcher")
        Dispatcher = getattr(module, "EventDispatcher", None)
    if not isinstance(Dispatcher, type):
        raise ImportError("native EventDispatcher is missing from its canonical module")
    Event = g["EventType"]
    np = g["_np"]
    standalone_dispatch = Dispatcher.dispatch

    def dispatch(self, event_type, **event_data):
        if _INPUT.get() is None:
            # Direct dispatcher calls preserve the pinned hover/capture and
            # pointer-identity contract. Only a Scene gateway selects the new
            # current-event coordinates and scene membership policy.
            return standalone_dispatch(self, event_type, **event_data)
        if not isinstance(event_type, Event):
            raise TypeError("dispatch requires an EventType")
        state = _state(self)
        live = _LIVE_EVENT.get()
        owner = _INPUT.get()
        if live is not None and owner is not None and live[0] is owner.scene:
            state.modifiers = live[1]
        elif "modifiers" in event_data or "mods" in event_data:
            state.modifiers = int(event_data.get("modifiers", event_data.get("mods", 0)))
        mouse = event_type.value.startswith("mouse")
        if mouse:
            point = np.asarray(event_data["point"], dtype=float)
            if point.shape != (3,) or not np.isfinite(point).all():
                raise ValueError("mouse input requires a finite 3D point")
            # Press and scroll carry their own authoritative point; drag and release
            # preserve the existing motion/press cursor position.
            if event_type not in (Event.MouseDragEvent, Event.MouseReleaseEvent):
                state.mouse_point = point.copy()
            if event_type in (Event.MouseDragEvent, Event.MousePressEvent):
                state.mouse_drag_point = point.copy()
        elif event_type == Event.KeyPressEvent:
            state.pressed_keys.add(event_data["symbol"])
        elif event_type == Event.KeyReleaseEvent:
            state.pressed_keys.discard(event_data["symbol"])

        def active(listener, kind):
            return _registered(self, listener, kind) and _in_scene(listener)

        def payload(listener):
            context = _INPUT.get()
            if mouse and context is not None and listener.mobject.is_fixed_in_frame():
                # World input is projected by the existing native CameraFrame
                # API for fixed-frame controls. No second coordinate model.
                data = dict(event_data)
                frame = context.scene.frame
                data["point"] = frame.to_fixed_frame_point(event_data["point"])
                if "d_point" in data:
                    data["d_point"] = frame.to_fixed_frame_point(data["d_point"], relative=True)
                return data
            return event_data

        def hit(listener, data):
            return listener.mobject.is_point_touching(data["point"])

        if event_type == Event.MousePressEvent:
            state.draggable_object_listners = []
            state.draggable_object_listners = [
                listener for listener in tuple(self.event_listners[Event.MouseDragEvent])
                if active(listener, Event.MouseDragEvent) and hit(listener, payload(listener))
            ]
        elif event_type == Event.MouseReleaseEvent:
            state.draggable_object_listners = []
        captured = event_type == Event.MouseDragEvent
        listeners = tuple(state.draggable_object_listners if captured
                          else self.event_listners[event_type])
        result = None
        try:
            for listener in listeners:
                # Snapshot the delivery order, but recheck removals and scene
                # membership at each turn. Added listeners wait for next input.
                if not active(listener, event_type):
                    continue
                data = payload(listener)
                if mouse and not captured and not hit(listener, data):
                    continue
                result = listener.callback(listener.mobject, data)
                if result is False:
                    return False
            return result
        except BaseException:
            if mouse:
                state.draggable_object_listners = []
            raise
        finally:
            # A removed listener must not remain a strong capture owner.
            state.draggable_object_listners[:] = [
                listener for listener in state.draggable_object_listners
                if _registered(self, listener, Event.MouseDragEvent)
            ]

    def mouse_point(self):
        return _state(self).mouse_point

    def drag_point(self):
        return _state(self).mouse_drag_point

    def pressed(self, symbol):
        return symbol in _state(self).pressed_keys

    _method(Dispatcher, "dispatch", dispatch)
    Dispatcher.__call__ = dispatch
    _method(Dispatcher, "get_mouse_point", mouse_point)
    _method(Dispatcher, "get_mouse_drag_point", drag_point)
    _method(Dispatcher, "is_key_pressed", pressed)


def _install_scene_input(g):
    Scene, Event = g["Scene"], g["EventType"]
    previous_stopped = g["_scene_event_stopped"]

    def stopped(event_type, **event_data):
        context = _INPUT.get()
        if (context is not None and context.phase == "defaults"
                and context.event_type == event_type):
            # The outer scene gateway already dispatched this event. The
            # original Scene/InteractiveScene super chain runs defaults only.
            return False
        return previous_stopped(event_type, **event_data)

    g["_scene_event_stopped"] = stopped
    specifications = {
        "on_mouse_motion": (Event.MouseMotionEvent, ("point", "d_point")),
        "on_mouse_drag": (Event.MouseDragEvent, ("point", "d_point", "buttons", "modifiers")),
        "on_mouse_press": (Event.MousePressEvent, ("point", "button", "mods")),
        "on_mouse_release": (Event.MouseReleaseEvent, ("point", "button", "mods")),
        "on_mouse_scroll": (Event.MouseScrollEvent, ("point", "offset")),
        "on_key_press": (Event.KeyPressEvent, ("symbol", "modifiers")),
        "on_key_release": (Event.KeyReleaseEvent, ("symbol", "modifiers")),
    }

    def wrap(cls, original, event_type, fields):
        signature = inspect.signature(original)

        @wraps(original)
        def deliver(self, *args, **kwargs):
            enclosing = _INPUT.get()
            if (enclosing is not None and enclosing.scene is self
                    and enclosing.event_type == event_type
                    and enclosing.phase == "defaults"
                    and enclosing.owner is not cls and issubclass(enclosing.owner, cls)):
                return original(self, *args, **kwargs)
            bound = signature.bind(self, *args, **kwargs)
            bound.apply_defaults()
            data = {name: bound.arguments[name] for name in fields}
            if "point" in data:
                point = g["_np"].asarray(data["point"], dtype=float)
                if point.shape != (3,) or not g["_np"].isfinite(point).all():
                    raise ValueError("mouse input requires a finite 3D point")
                # A press/scroll may be the first event after focus. Both the
                # Scene cursor and dispatcher must use that event's point.
                if event_type not in (Event.MouseDragEvent, Event.MouseReleaseEvent):
                    self.mouse_point.move_to(point)
                if event_type in (Event.MouseDragEvent, Event.MousePressEvent):
                    self.mouse_drag_point.move_to(point)
            context = SimpleNamespace(scene=self, event_type=event_type,
                                      phase="dispatch", owner=cls)
            token = _INPUT.set(context)
            try:
                if g["_event_dispatcher"]().dispatch(event_type, **data) is False:
                    return False
                context.phase = "defaults"
                return original(self, *args, **kwargs)
            finally:
                _INPUT.reset(token)
        return deliver

    # Wrap only the shipped owning definitions, not aliases or inherited
    # methods. User subclasses keep their normal MRO and super() behavior.
    classes = {cls for cls in tuple(g.values())
               if isinstance(cls, type) and issubclass(cls, Scene)}
    for cls in classes:
        for name, (event_type, fields) in specifications.items():
            original = vars(cls).get(name)
            if original is not None:
                setattr(cls, name, wrap(cls, original, event_type, fields))


def install_interaction(native: Any) -> None:
    """Bind scene input without changing published class or dispatcher identity."""
    g = vars(native)
    if g.get("_FMN_INTERACTION_INSTALLED", False):
        return
    _install_dispatcher(g)
    _install_scene_input(g)
    g["_FMN_INTERACTION_INSTALLED"] = True
