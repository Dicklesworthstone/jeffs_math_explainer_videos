"""Transform's native image axis alongside the existing record/path interpolation.

No alternate geometry, sampler, clock, frame loop or public class is introduced.
Only changed materials acquire decoded plans; ordinary placement remains native.
"""
from __future__ import annotations

import copy
from contextvars import ContextVar
from functools import wraps

from .image_authoring import _method
from .raster_animation import _METADATA
from .scene_execution import _TransientState, _MISSING as _TRANSIENT_MISSING, _note

_CONTEXT = ContextVar('fmn_transform_material_plans', default=None)
_PREPARING = ContextVar('fmn_transform_material_preparing', default=None)
_FRAME_BUSY = '_fmn_transform_material_interpolating'
_CACHE = '_fmn_transform_material_plans'
_TRANSIENTS = '_fmn_transform_material_transients'
_ACTIVE = '_fmn_transform_material_active'
_MAX_TEXELS = 16_777_216  # 256 MiB, summed before decoding a whole family.
_MISSING = object()


class _TransformTransient(_TransientState):
    def __deepcopy__(self, memo):
        # An active Animation.copy clones its endpoints through the same memo.
        # Missing flag values are ownership sentinels, not authored objects:
        # copying them into new object() values would restore truthy garbage.
        result = type(self).__new__(type(self))
        memo[id(self)] = result
        for key, value in vars(self).items():
            setattr(result, key, value if value is _TRANSIENT_MISSING else copy.deepcopy(value, memo))
        return result


def install_raster_transform(native):
    g = vars(native)
    if g.get('_FMN_RASTER_TRANSFORM_INSTALLED', False):
        return
    Plan = g['_RasterTransition']
    if not callable(getattr(Plan, 'between', None)):
        raise ImportError('native Transform material planning is missing')
    Mobject, Transform, CameraFrame = g['Mobject'], g['Transform'], g['CameraFrame']
    previous_interpolate = Mobject.interpolate
    previous_align = Mobject.align_data_and_family
    previous_begin, previous_finish = Transform.begin, Transform.finish
    previous_frame = Transform.interpolate_mobject
    previous_abort = getattr(Transform, 'abort', None)
    previous_requires = g['_requires_python_animation']

    def clear(self):
        vars(self).pop(_CACHE, None)
        vars(self).pop(_TRANSIENTS, None)
        vars(self).pop(_ACTIVE, None)

    def abort(self):
        if not vars(self).get(_ACTIVE, False):
            return
        snapshots = vars(self).pop(_TRANSIENTS, ())
        clear(self)  # Release decoded resources even when inherited cleanup fails.
        primary = None
        try:
            if callable(previous_abort) and not getattr(previous_abort, '_fmn_schema_placeholder', False):
                previous_abort(self)
        except BaseException as error:
            primary = error
        for snapshot in reversed(snapshots):
            error = snapshot.restore(primary)
            if primary is None:
                primary = error
        if primary is not None:
            raise primary

    def failed(self, error):
        try:
            abort(self)
        except BaseException as cleanup:
            _note(error, 'Transform material cleanup also failed: ' + type(cleanup).__name__)

    def capture_transients(animation):
        snapshots = vars(animation).setdefault(_TRANSIENTS, [])
        seen = {id(snapshot.mob) for snapshot in snapshots}
        family = list(animation.mobject.get_family())
        index = 0
        while index < len(family):
            member = family[index]
            index += 1
            if id(member) in seen:
                continue
            seen.add(id(member))
            snapshots.append(_TransformTransient(member))
            # Animating status also propagates upward. Capture those owners,
            # not their unrelated siblings, before acquiring descendant flags.
            family.extend(tuple(getattr(member, 'parents', ())))

    @wraps(previous_align)
    def align(self, mobject):
        result = previous_align(self, mobject)
        animation = _PREPARING.get()
        if animation is not None and self is animation.mobject:
            # Alignment can create copies/null members. Capture their baseline
            # after creation but BEFORE Animation.begin acquires their flags.
            capture_transients(animation)
        return result

    @wraps(previous_begin)
    def begin(self):
        if vars(self).get(_ACTIVE, False):
            abort(self)
        clear(self)
        vars(self).update({_CACHE: {}, _TRANSIENTS: [], _ACTIVE: True})
        token = _PREPARING.set(self)
        try:
            capture_transients(self)
            return previous_begin(self)
        except BaseException as error:
            failed(self, error)
            raise
        finally:
            _PREPARING.reset(token)

    @wraps(previous_finish)
    def finish(self):
        try:
            return previous_finish(self)
        except BaseException as error:
            failed(self, error)
            raise
        finally:
            clear(self)

    def plans(self):
        cache = vars(self).setdefault(_CACHE, {})
        pending, retained, total = {}, {}, 0
        for family in self.families:
            if len(family) != 3 or any(isinstance(obj, CameraFrame) for obj in family):
                continue
            live, start, end = family
            key = (id(live), id(start), id(end))
            if key in retained or key in pending:
                continue
            old = cache.get(key)
            cost = Plan.required_texels(start, end)
            total += cost
            if total > _MAX_TEXELS:
                raise ValueError('Transform material family exceeds its 256 MiB decoded endpoint budget')
            if old is not None and old.matches(start, end):
                retained[key] = old
            elif cost or old is not None:
                pending[key] = (start, end)
            else:
                # An unchanged material is not this animation's write axis.
                # It must not undo a simultaneous RasterTransition or updater.
                retained[key] = None
        # All admission checks precede decoding or record/pixel interpolation.
        # Discard stale plans first, avoiding a second full family of decodes.
        old = None  # Do not retain the final stale decode through the loop local.
        cache.clear()
        cache.update(retained)
        for key, (start, end) in pending.items():
            cache[key] = Plan.between(start, end)
        return cache

    @wraps(previous_frame)
    def frame(self, alpha):
        if vars(self).get(_FRAME_BUSY, False):
            raise RuntimeError('Transform material interpolation cannot reenter the same animation')
        vars(self)[_FRAME_BUSY] = True
        owned = vars(self).get(_ACTIVE, False)
        token = None
        try:
            token = _CONTEXT.set(plans(self))
            return previous_frame(self, alpha)
        except BaseException as error:
            failed(self, error)
            raise
        finally:
            if token is not None:
                _CONTEXT.reset(token)
            if not owned:
                vars(self).pop(_CACHE, None)
            vars(self).pop(_FRAME_BUSY, None)

    @wraps(previous_interpolate)
    def interpolate(self, mobject1, mobject2, alpha, path_func=None):
        start, end = mobject1, mobject2
        if any(isinstance(obj, CameraFrame) for obj in (self, start, end)):
            return previous_interpolate(self, start, end, alpha, path_func)
        context = _CONTEXT.get()
        plan = _MISSING if context is None else context.get((id(self), id(start), id(end)), _MISSING)
        if plan is _MISSING:
            plan = Plan.between(start, end)
        value = float(alpha)
        candidate = None if plan is None else plan.sample(value)
        result = previous_interpolate(self, start, end, value, path_func)
        if candidate is not None:
            g['_replace_raster_image'](self, candidate)
            attrs = vars(self)
            if value <= 0 or value >= 1:
                endpoint = vars(start if value <= 0 else end)
                for name in _METADATA:
                    if name in endpoint:
                        attrs[name] = endpoint[name]
                    else:
                        attrs.pop(name, None)
                has_dark = plan.start_has_dark if value <= 0 else plan.end_has_dark
            else:
                for name in _METADATA:
                    if name in attrs and name != 'num_textures':
                        attrs[name] = None
                has_dark = plan.has_dark
            if isinstance(self, g['TexturedSurface']) or 'num_textures' in attrs:
                attrs['num_textures'] = 2 if has_dark else 1
        return result

    def requires(animation):
        if previous_requires(animation):
            return True
        if (not isinstance(animation, Transform) or getattr(animation, '_native_kind', None)
                not in ('transform', 'replacement_transform', 'transform_from_copy')):
            return False
        attrs = vars(animation)
        roots = attrs.get('mobject'), attrs.get('target_mobject')
        if not all(isinstance(root, Mobject) and not isinstance(root, CameraFrame) for root in roots):
            return False  # Existing deferred-target admission owns these families.
        a, b = (root.get_family() for root in roots)
        if len(a) != len(b):
            return any(Plan.between(obj, obj) is not None for obj in (*a, *b))
        for start, end in zip(a, b):
            if not g['_raster_images_equal'](start, end):
                return True
            if (start.has_updaters() or end.has_updaters()) and Plan.between(start, start) is not None:
                return True
        return False

    # Install before playback freezes the shipped Mobject/Transform protocols.
    # The later authored-dispatch guard runs before this resource admission.
    for cls, name, method in ((Mobject, 'interpolate', interpolate),
                              (Mobject, 'align_data_and_family', align),
                              (Transform, 'begin', begin), (Transform, 'finish', finish),
                              (Transform, 'interpolate_mobject', frame), (Transform, 'abort', abort)):
        _method(cls, name, method)
    g['_requires_python_animation'] = requires
    g['_FMN_RASTER_TRANSFORM_INSTALLED'] = True
