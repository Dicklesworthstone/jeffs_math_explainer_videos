"""Install the pinned shared animation protocol onto the native module."""


def install(native, subsystems=True):
    """The shared protocol: the Animation/Transform core, then (unless
    subsystems is False) every subsystem protocol layered on it. The core
    alone needs only those classes; scripts/test_python_transform_paths.py
    exercises it over a minimal namespace. Idempotent."""
    g = vars(native)
    if g.get("_FMN_ANIMATION_SEMANTICS_INSTALLED", False):
        return
    np = g["_np"]
    interpolate_value = g["_interpolate"]
    copy_module = g["_copy"]
    smooth_rate = g["_smooth_rate"]
    refuse_unrouted = g["_refuse_unrouted"]
    Mobject = g["Mobject"]
    Animation = g["Animation"]
    NativeAnimation = g["_NativeAnimation"]
    Transform = g["Transform"]
    ReplacementTransform = g["ReplacementTransform"]
    TransformFromCopy = g["TransformFromCopy"]
    DrawBorderThenFill = g["DrawBorderThenFill"]
    FadeTransform = g["FadeTransform"]
    FadeTransformPieces = g["FadeTransformPieces"]
    original_transform_init = Transform.__init__
    original_requires_python = g["_requires_python_animation"]
    # Captured before any authored code runs: a later reassignment of
    # g["straight_path"] is authored and keeps the per-call view re-read.
    shipped_straight_path = g.get("straight_path")

    def uses_python_path(animation):
        path = getattr(animation, "path_func", None)
        return path is not None and getattr(path, "_fmn_path_arc", None) is None

    def transform_init(
        self,
        mobject,
        target_mobject=None,
        path_arc=0.0,
        path_arc_axis=g["_OUT"],
        path_func=None,
        **kwargs,
    ):
        if path_func is not None and not callable(path_func):
            raise TypeError("Transform path_func must be callable")
        python_path = (
            path_func is not None
            and getattr(path_func, "_fmn_path_arc", None) is None
        )
        # The callback segment now releases the Stage around every Python
        # hook. Keep scalar arc factories on Choreo's native path, but do not
        # reject an authored point-array map that this boundary can execute.
        # Target-less Swap/CyclicReplace and Restore still require their native lowering.
        Restore = g.get("Restore")
        if python_path and (self._target_attr is None or (Restore is not None and isinstance(self, Restore))):
            refuse_unrouted(type(self).__name__ + "()", [("path_func", True)])
        original_transform_init(
            self,
            mobject,
            target_mobject,
            path_arc=path_arc,
            path_arc_axis=path_arc_axis,
            path_func=None if python_path else path_func,
            **kwargs,
        )
        self.path_func = path_func

    def requires_python_animation(animation):
        if not getattr(animation, "_native_kind", None):
            return True
        # Choreo's tracker lane is scalar; a vector ValueTracker keeps its
        # value in the "value" uniform, which only the Python path blends.
        # Reduced installer tables carry no ValueTracker at all.
        tracker = g.get("ValueTracker")
        if tracker is not None and isinstance(animation, Transform) and any(
            isinstance(member, tracker) and member._vector_value() is not None
            for member in animation.mobject.get_family()
        ):
            return True
        # Consult the live path, not a constructor-time flag: scene authors
        # may replace it between plays or install it on an animation instance.
        if (
            isinstance(animation, Transform)
            and animation._target_attr is not None
        ):
            if uses_python_path(animation):
                return True
            # Compare against the nearest shipped class, not Transform alone:
            # Grow/Indicate/etc. already have native implementations of their
            # own hooks. Only authored changes require callback dispatch.
            for cls in type(animation).__mro__:
                baseline = transform_protocols.get(cls)
                if baseline is not None:
                    for name, expected in baseline.items():
                        method = getattr(animation, name)
                        if getattr(method, "__func__", method) is not expected:
                            return True
                    return False
        return original_requires_python(animation)

    def mobject_str(self):
        return type(self).__name__

    def interpolate_uniform(start, end, alpha):
        if isinstance(start, np.ndarray) or isinstance(end, np.ndarray):
            return interpolate_value(np.asarray(start), np.asarray(end), alpha)
        if isinstance(start, tuple) or isinstance(end, tuple):
            value = interpolate_value(
                np.asarray(start, dtype=float),
                np.asarray(end, dtype=float),
                alpha,
            )
            return tuple(value.tolist())
        if isinstance(start, list) or isinstance(end, list):
            value = interpolate_value(
                np.asarray(start, dtype=float),
                np.asarray(end, dtype=float),
                alpha,
            )
            return value.tolist()
        return interpolate_value(start, end, alpha)

    def mobject_interpolate(self, mobject1, mobject2, alpha, path_func=None):
        if path_func is None:
            path_func = g["straight_path"]
        alpha = float(alpha)
        locked_data = getattr(self, "locked_data_keys", ())
        constant_data = getattr(self, "const_data_keys", ())
        # Each `.data` read builds a fresh live view of the native records
        # (the Reference's is a plain attribute), so read each once per call
        # (fm-5wq.31: ~23 view constructions per call was 30% of a large
        # Transform). Only an authored path_func could replace the records
        # mid-loop; the views are re-read after each call to one, but not
        # after the shipped straight_path, which is NumPy arithmetic.
        data = self.data
        start_data, end_data = mobject1.data, mobject2.data
        data_keys = [
            key for key in data.dtype.names if key not in locked_data
        ]
        # Point-free family roots can still carry animated uniforms. They
        # have no first record from which to broadcast a constant data field.
        if len(data) == 0:
            data_keys = []
        if data_keys:
            self.note_changed_data()
        pointlike = self.pointlike_data_keys
        for key in data_keys:
            start = start_data[key]
            end = end_data[key]
            if key in constant_data:
                start = start[0]
                end = end[0]
            if key in pointlike:
                value = path_func(start, end, alpha)
                if path_func is not shipped_straight_path:
                    data, start_data, end_data = self.data, mobject1.data, mobject2.data
            else:
                value = interpolate_value(start, end, alpha)
            data[key][:] = value
        locked_uniforms = getattr(self, "locked_uniform_keys", ())
        for key in tuple(self.uniforms):
            if key in locked_uniforms:
                continue
            # Match Choreo's interpolate_fields: typed flags and joint style
            # stay the live object's own; only numeric uniforms interpolate.
            if key == "joint_type" or isinstance(self.uniforms[key], (bool, np.bool_)):
                continue
            if key not in mobject1.uniforms or key not in mobject2.uniforms:
                continue
            self.uniforms[key] = interpolate_uniform(
                mobject1.uniforms[key],
                mobject2.uniforms[key],
                alpha,
            )
        return self

    def validate_input_type(self, mobject):
        del self
        if not isinstance(mobject, Mobject):
            raise TypeError("Animation only works for Mobjects.")

    def animation_init(
        self,
        mobject,
        run_time=1.0,
        time_span=None,
        lag_ratio=0.0,
        rate_func=None,
        name="",
        remover=False,
        final_alpha_value=1.0,
        suspend_mobject_updating=False,
        **kwargs,
    ):
        self._validate_input_type(mobject)
        self.mobject = mobject
        self.run_time = float(run_time)
        self.time_span = time_span
        self.lag_ratio = float(lag_ratio)
        self.rate_func = (
            rate_func
            if rate_func is not None
            else g.get("smooth", smooth_rate)
        )
        self.name = name or type(self).__name__ + str(mobject)
        self.remover = bool(remover)
        self.final_alpha_value = float(final_alpha_value)
        self.suspend_mobject_updating = bool(suspend_mobject_updating)
        self.__dict__.update(kwargs)

    def ensure_runtime_defaults(self):
        if self.run_time is None:
            self.run_time = 1.0
        if self.rate_func is None:
            self.rate_func = g.get("smooth", smooth_rate)
        if self.lag_ratio is None:
            self.lag_ratio = 0.0

    def animation_str(self):
        return getattr(self, "name", type(self).__name__)

    def create_starting_mobject(self):
        starting = self.mobject.copy()
        CameraFrame = g.get("CameraFrame")
        if CameraFrame is not None and isinstance(self.mobject, CameraFrame):
            if hasattr(starting, "clear_updaters"):
                starting.clear_updaters()
            elif hasattr(starting, "updaters"):
                starting.updaters.clear()
        return starting

    def get_all_mobjects(self):
        return self.mobject, self.starting_mobject

    def get_all_families_zipped(self):
        return zip(*[
            mobject.get_family()
            for mobject in self.get_all_mobjects()
        ])

    def get_all_mobjects_to_update(self):
        result = []
        seen = set()
        CameraFrame = g.get("CameraFrame")
        for mobject in self.get_all_mobjects():
            identity = id(mobject)
            if mobject is self.mobject or identity in seen:
                continue
            if CameraFrame is not None and isinstance(mobject, CameraFrame):
                continue
            seen.add(identity)
            result.append(mobject)
        return result

    def update_mobjects(self, dt):
        for mobject in self.get_all_mobjects_to_update():
            mobject.update(dt)

    def animation_begin(self):
        self._ensure_runtime_defaults()
        if self.time_span is not None:
            self.run_time = max(float(self.time_span[1]), self.run_time)
        self.mobject.set_animating_status(True)
        self.starting_mobject = self.create_starting_mobject()
        # Duck-typed animations driven through this begin need not carry
        # the Transform re-alignment hook.
        align_starting_mobject = getattr(self, "_align_starting_mobject", None)
        if align_starting_mobject is not None:
            align_starting_mobject()
        self.mobject_was_updating = False
        if self.suspend_mobject_updating:
            self.mobject_was_updating = not self.mobject._is_updating_suspended()
            self.mobject.suspend_updating()
        self.families = list(self.get_all_families_zipped())
        self.interpolate(0.0)

    def animation_finish(self):
        self.interpolate(self.final_alpha_value)
        self.mobject.set_animating_status(False)
        if self.suspend_mobject_updating and self.mobject_was_updating:
            self.mobject.resume_updating()

    def animation_copy(self):
        return copy_module.deepcopy(self)

    def update_rate_info(self, run_time=None, rate_func=None, lag_ratio=None):
        self.run_time = run_time or self.run_time
        self.rate_func = rate_func or self.rate_func
        self.lag_ratio = lag_ratio or self.lag_ratio
        return self

    def animation_interpolate(self, alpha):
        self.interpolate_mobject(float(alpha))

    def animation_update(self, alpha):
        self.interpolate(alpha)

    def time_spanned_alpha(self, alpha):
        if self.time_span is None:
            return float(alpha)
        start, end = self.time_span
        return np.clip(
            float(alpha) * self.run_time - start,
            0.0,
            end - start,
        ) / (end - start)

    def interpolate_mobject(self, alpha):
        spanned_alpha = self.time_spanned_alpha(alpha)
        for index, mobjects in enumerate(self.families):
            sub_alpha = self.get_sub_alpha(
                spanned_alpha,
                index,
                len(self.families),
            )
            self.interpolate_submobject(*mobjects, sub_alpha)

    def interpolate_submobject(self, submobject, starting_submobject, alpha):
        del self, submobject, starting_submobject, alpha

    def get_sub_alpha(self, alpha, index, num_submobjects):
        full_length = (num_submobjects - 1) * self.lag_ratio + 1
        value = float(alpha) * full_length
        lower = index * self.lag_ratio
        return self.rate_func(min(max(value - lower, 0.0), 1.0))

    def set_run_time(self, run_time):
        self.run_time = float(run_time)
        return self

    def get_run_time(self):
        if self.time_span:
            return max(self.run_time, float(self.time_span[1]))
        return self.run_time

    def set_rate_func(self, rate_func):
        self.rate_func = rate_func
        return self

    def get_rate_func(self):
        return self.rate_func

    def set_name(self, name):
        self.name = name
        return self

    def is_remover(self):
        return bool(getattr(self, "remover", False))

    def clean_up_from_scene(self, scene):
        if self.is_remover():
            scene.remove(self.mobject)

    def native_animation_init(
        self,
        mobject,
        run_time=None,
        rate_func=None,
        lag_ratio=None,
        time_span=None,
        final_alpha_value=1.0,
        suspend_mobject_updating=False,
        name="",
        remover=False,
        **kwargs,
    ):
        # Native compositions derive their root from their members in Choreo.
        # None is an internal sentinel for that class family, not a generally
        # valid Animation target. Do not trust a spoofable _native_kind string.
        if mobject is not None or not isinstance(self, g["AnimationGroup"]):
            self._validate_input_type(mobject)
        refuse_unrouted(
            type(self).__name__ + "()",
            [(key, True) for key in sorted(kwargs)],
        )
        self.mobject = mobject
        self.run_time = run_time
        self.rate_func = rate_func
        self.lag_ratio = lag_ratio
        self.time_span = (
            None
            if time_span is None
            else (float(time_span[0]), float(time_span[1]))
        )
        self.final_alpha_value = float(final_alpha_value)
        self.suspend_mobject_updating = bool(suspend_mobject_updating)
        suffix = "" if mobject is None else str(mobject)
        self.name = name or type(self).__name__ + suffix
        self.remover = bool(remover)

    def init_path_func(self):
        if getattr(self, "path_func", None) is not None:
            return
        if float(self.path_arc) == 0.0:
            self.path_func = g["straight_path"]
        else:
            self.path_func = g["path_along_arc"](
                float(self.path_arc),
                self.path_arc_axis,
            )

    def create_target(self):
        target = getattr(self, "target_mobject", None)
        return self.mobject.copy() if target is None else target

    def check_target_mobject_validity(self):
        if not isinstance(self.target_mobject, Mobject):
            raise TypeError("Transform target must be a Mobject")

    def native_target(self):
        # Scene.play handles camera transforms before callback dispatch. Do
        # not silently lower a later-assigned path or authored hook to its
        # native endpoint-only camera track.
        if (
            isinstance(self.mobject, g["CameraFrame"])
            and requires_python_animation(self)
        ):
            raise NotImplementedError(
                "Python-callback animations of the camera frame await the "
                "camera track's per-frame callback seam"
            )
        # CyclicReplace/Swap carry multiple source mobjects rather than a
        # Transform target. Their native constructor owns those destinations.
        if self._target_attr is None:
            return None
        self.target_mobject = self.create_target()
        self.check_target_mobject_validity()
        return self.target_mobject

    def transform_begin(self):
        self._ensure_runtime_defaults()
        self.init_path_func()
        self.target_mobject = self.create_target()
        self.check_target_mobject_validity()
        if self.mobject.is_aligned_with(self.target_mobject):
            self.target_copy = self.target_mobject
        else:
            self.target_copy = self.target_mobject.copy()
        self.mobject.align_data_and_family(self.target_copy)
        Animation.begin(self)
        # Equal endpoint records do not imply a constant authored path: a
        # closed excursion may leave and return to the very same points.
        # Explicit user locks remain respected by Mobject.interpolate.
        # Camera pose is not in the RecordBuffer: equal empty endpoint
        # arrays must not lock the animated center and dimensions.
        if (not self.mobject.has_updaters() and not uses_python_path(self)
                and not isinstance(self.mobject, g["CameraFrame"])):
            self.mobject.lock_matching_data(
                self.starting_mobject,
                self.target_copy,
            )

    def animation_align_starting_mobject(self):
        # A plain Animation interpolates nothing pointwise from its start.
        return None

    def transform_align_starting_mobject(self):
        # BN-09: a rebuilt shape follows the one arc-density rule, so a
        # starting mobject derived from the aligned mobject can change its
        # point count. GrowArrow's scale(0) re-tessellates an arced Arrow at
        # a new stem angle. The Reference's Arrow keeps a flat 8-component
        # arc and never meets this. Re-align start, mobject and target copy
        # until all three agree. Alignment only inserts points, so this
        # settles within a few rounds, and an aligned start is untouched.
        start = self.starting_mobject
        if start.is_aligned_with(self.mobject):
            return
        if self.target_copy is self.target_mobject:
            self.target_copy = self.target_mobject.copy()
        for _ in range(3):
            start.align_data_and_family(self.mobject)
            self.mobject.align_data_and_family(self.target_copy)
            if start.is_aligned_with(self.mobject) and self.mobject.is_aligned_with(
                    self.target_copy):
                return

    def transform_finish(self):
        Animation.finish(self)
        self.mobject.unlock_data()

    def transform_cleanup(self, scene):
        Animation.clean_up_from_scene(self, scene)
        if self.replace_mobject_with_target_in_scene:
            scene.remove(self.mobject)
            scene.add(self.target_mobject)

    def transform_all_mobjects(self):
        return (
            self.mobject,
            self.starting_mobject,
            self.target_mobject,
            self.target_copy,
        )

    def transform_families(self):
        return zip(*[
            mobject.get_family()
            for mobject in (
                self.mobject,
                self.starting_mobject,
                self.target_copy,
            )
        ])

    def transform_submobject(
        self,
        submobject,
        starting_submobject,
        target_copy,
        alpha,
    ):
        submobject.interpolate(
            starting_submobject,
            target_copy,
            alpha,
            self.path_func,
        )
        return self

    def transform_interpolate_mobject(self, alpha):
        # Replace Transform's bootstrap-local straight-only implementation;
        # otherwise it shadows Animation's family/lag dispatch and none of
        # the installed path, style, uniform, or subclass hooks can run.
        Animation.interpolate_mobject(self, alpha)

    def transform_from_copy_init(self, mobject, target_mobject, **kwargs):
        Transform.__init__(self, mobject.copy(), target_mobject, **kwargs)
        # The public constructor already froze the source copy. The native
        # replacement transform must consume that exact object; invoking the
        # native copy constructor again would leave this first copy rooted.
        self._native_kind = "replacement_transform"

    Mobject.__str__ = mobject_str
    Mobject.interpolate = mobject_interpolate
    Animation.__init__ = animation_init
    Animation._validate_input_type = validate_input_type
    Animation._ensure_runtime_defaults = ensure_runtime_defaults
    Animation.__str__ = animation_str
    Animation.begin = animation_begin
    Animation._align_starting_mobject = animation_align_starting_mobject
    Animation.finish = animation_finish
    Animation.create_starting_mobject = create_starting_mobject
    Animation.get_all_mobjects = get_all_mobjects
    Animation.get_all_families_zipped = get_all_families_zipped
    Animation.get_all_mobjects_to_update = get_all_mobjects_to_update
    Animation.update_mobjects = update_mobjects
    Animation.copy = animation_copy
    Animation.update_rate_info = update_rate_info
    Animation.interpolate = animation_interpolate
    Animation.update = animation_update
    Animation.time_spanned_alpha = time_spanned_alpha
    Animation.interpolate_mobject = interpolate_mobject
    Animation.interpolate_submobject = interpolate_submobject
    Animation.get_sub_alpha = get_sub_alpha
    Animation.set_run_time = set_run_time
    Animation.get_run_time = get_run_time
    Animation.set_rate_func = set_rate_func
    Animation.get_rate_func = get_rate_func
    Animation.set_name = set_name
    Animation.is_remover = is_remover
    Animation.clean_up_from_scene = clean_up_from_scene
    NativeAnimation.__init__ = native_animation_init
    Transform.__init__ = transform_init
    Transform.replace_mobject_with_target_in_scene = False
    Transform.init_path_func = init_path_func
    Transform.create_target = create_target
    Transform.check_target_mobject_validity = check_target_mobject_validity
    Transform._native_target = native_target
    Transform.begin = transform_begin
    Transform._align_starting_mobject = transform_align_starting_mobject
    Transform.finish = transform_finish
    Transform.clean_up_from_scene = transform_cleanup
    Transform.get_all_mobjects = transform_all_mobjects
    Transform.get_all_families_zipped = transform_families
    Transform.interpolate_mobject = transform_interpolate_mobject
    Transform.interpolate_submobject = transform_submobject
    ReplacementTransform.replace_mobject_with_target_in_scene = True
    TransformFromCopy.replace_mobject_with_target_in_scene = True
    TransformFromCopy.__init__ = transform_from_copy_init

    def draw_border_get_outline(self):
        """Expose a native-backed outline copy without changing the source."""
        # The animation accepts one RGB color, not a per-vertex color list.
        # Normalize it once so array/tuple RGB values do not enter the style
        # setter's gradient-list interpretation or an ambiguous truth test.
        color = (None if self.stroke_color is None else
                 (g["_ColorValue"](g["_color_to_rgb"](self.stroke_color))
                  if "_ColorValue" in g and "_color_to_rgb" in g
                  else self.stroke_color))
        outline = self.mobject.copy()
        outline.set_fill(opacity=0)
        behind = bool(
            getattr(self.mobject, "stroke_behind", False)
            or (hasattr(self.mobject, "uniforms") and self.mobject.uniforms.get("stroke_behind", False))
        )
        for member in outline.family_members_with_points():
            member_behind = behind or bool(
                getattr(member, "stroke_behind", False)
                or (hasattr(member, "uniforms") and member.uniforms.get("stroke_behind", False))
            )
            member.set_stroke(
                color=color if color is not None else member.get_stroke_color(),
                width=self.stroke_width,
                behind=member_behind,
            )
        return outline

    def fade_transform_ghost_to(self, source, target):
        source.replace(target, stretch=self.stretch, dim_to_match=self.dim_to_match)
        source.set_uniform(**target.get_uniforms())
        source.set_opacity(0)

    def fade_transform_pieces_ghost_to(self, source, target):
        for source_member, target_member in zip(source.get_family(), target.get_family()):
            FadeTransform.ghost_to(self, source_member, target_member)

    DrawBorderThenFill.get_outline = draw_border_get_outline
    FadeTransform.ghost_to = fade_transform_ghost_to
    FadeTransformPieces.ghost_to = fade_transform_pieces_ghost_to

    # Both the embedded extension and installed wheel expose these same class
    # objects. Give their methods the public identities before either route
    # applies schema provenance or resolves qualified compatibility imports.
    semantic_methods = {
        Mobject: ("__str__", "interpolate"),
        Animation: (
            "__init__", "_validate_input_type", "_ensure_runtime_defaults",
            "__str__", "begin", "finish", "create_starting_mobject",
            "get_all_mobjects", "get_all_families_zipped",
            "get_all_mobjects_to_update", "update_mobjects", "copy",
            "update_rate_info", "interpolate", "update", "time_spanned_alpha",
            "interpolate_mobject", "interpolate_submobject", "get_sub_alpha",
            "set_run_time", "get_run_time", "set_rate_func", "get_rate_func",
            "set_name", "is_remover", "clean_up_from_scene",
        ),
        NativeAnimation: ("__init__",),
        Transform: (
            "__init__", "init_path_func", "create_target", "check_target_mobject_validity",
            "_native_target", "begin", "finish", "clean_up_from_scene",
            "get_all_mobjects", "get_all_families_zipped",
            "interpolate_mobject", "interpolate_submobject",
        ),
        TransformFromCopy: ("__init__",),
        DrawBorderThenFill: ("get_outline",),
        FadeTransform: ("ghost_to",),
        FadeTransformPieces: ("ghost_to",),
    }
    for cls, names in semantic_methods.items():
        for name in names:
            function = vars(cls)[name]
            function.__name__ = name
            function.__qualname__ = f"{cls.__qualname__}.{name}"
            function.__module__ = cls.__module__
    # Capture after shared installation so inherited methods and qualified
    # exports have their final identities. Retain function objects, not just
    # class names, so later class/instance monkeypatches are visible too.
    transform_hooks = (
        "create_target", "create_starting_mobject", "init_path_func",
        "check_target_mobject_validity", "get_all_mobjects",
        "get_all_families_zipped", "get_all_mobjects_to_update",
        "get_sub_alpha", "time_spanned_alpha",
    )
    transform_protocols = {
        cls: {name: getattr(cls, name) for name in transform_hooks}
        for cls in tuple(g.values())
        if isinstance(cls, type) and issubclass(cls, Transform)
        and getattr(cls, "_target_attr", None) is not None
    }
    g["_requires_python_animation"] = requires_python_animation
    if subsystems:
        _install_matching_parts(g)
        _install_matching_strings(g)
        _install_composition_lifecycle(g)
        _install_camera_pose(g)
        _install_camera_motion(g)
        _install_camera_choreography(g)
        if "ShowPartial" in g:
            _install_partial_reveals(g)
        if "DrawBorderThenFill" in g and "Write" in g:
            _install_border_write(g)
    g["_FMN_ANIMATION_SEMANTICS_INSTALLED"] = True


def _install_matching_parts(g):
    """Plan with public hooks; execute leaves on the existing Choreo boundary."""
    Parts = g["TransformMatchingParts"]
    Shapes = g["TransformMatchingShapes"]
    Animation = g["Animation"]
    AnimationGroup = g["AnimationGroup"]
    Mobject = g["Mobject"]
    Transform = g["Transform"]
    CallbackDriver = g["_CompositionCallbackDriver"]
    NativeLeaf = g["_NativeCompositionLeaf"]

    def unique(objects):
        result, seen = [], set()
        for obj in objects:
            if id(obj) not in seen:
                seen.add(id(obj))
                result.append(obj)
        return result

    def animated_mobjects(animation):
        # CameraFrame is a native pose, never a drawable Stage family member.
        if isinstance(animation.mobject, g["CameraFrame"]):
            return []
        if isinstance(animation.mobject, Mobject):
            return unique([animation.mobject, *getattr(animation, "_native_extra_mobjects", ())])
        if isinstance(animation, AnimationGroup):
            return unique(
                obj for member in animation.animations
                for obj in animated_mobjects(member)
            )
        raise TypeError("Matching animations must animate Mobjects")

    def matching_init(
        self, source, target, matched_pairs=(), match_animation=Transform,
        mismatch_animation=Transform, run_time=2, lag_ratio=0, **kwargs,
    ):
        if not isinstance(source, Mobject) or not isinstance(target, Mobject):
            raise TypeError(type(self).__name__ + " expects two Mobject families")
        if not callable(match_animation) or not callable(mismatch_animation):
            raise TypeError("match_animation and mismatch_animation must be callable")
        pairs = [tuple(pair) for pair in matched_pairs]
        if not all(len(pair) == 2 and all(isinstance(obj, Mobject) for obj in pair)
                   for pair in pairs):
            raise TypeError(type(self).__name__ + " matched_pairs must pair Mobjects")
        self.source, self.target = source, target
        self.target_mobject = target
        self.matched_pairs = pairs
        self.match_animation, self.mismatch_animation = match_animation, mismatch_animation
        self.anim_config = dict(kwargs)
        self.source_pieces = unique(source.family_members_with_points())
        self.target_pieces = unique(target.family_members_with_points())
        self.anims = []
        for pair in pairs:
            self.add_transform(*pair)
        # Snapshot the candidates before add_transform consumes their families.
        for pair in list(self.find_pairs_with_matching_shapes(
            self.source_pieces, self.target_pieces,
        )):
            self.add_transform(*pair)
        claimed = {
            id(member) for animation in self.anims
            for obj in animated_mobjects(animation) for member in obj.get_family()
        }
        for piece in self.source_pieces:
            if id(piece) not in claimed:
                self.anims.append(g["FadeOutToPoint"](
                    piece, target.get_center(), **self.anim_config,
                ))
        for piece in self.target_pieces:
            if id(piece) not in claimed:
                self.anims.append(g["FadeInFromPoint"](
                    piece, source.get_center(), **self.anim_config,
                ))
        AnimationGroup.__init__(self, *self.anims, run_time=run_time, lag_ratio=lag_ratio)
        # BN-11: members own easing; an omitted group curve is linear, not a
        # second smooth curve installed by the Python callback normalization.
        self.rate_func = g["_linear_rate"]
        objects = unique(obj for anim in self.animations for obj in animated_mobjects(anim))
        group_type = g["VGroup"] if all(isinstance(obj, g["VMobject"]) for obj in objects) else g["Group"]
        self.mobject = group_type(*objects)
        self.remover = True
        self._matching_driver = None
        self._matching_scene = None

    def add_transform(self, source, target):
        if not isinstance(source, Mobject) or not isinstance(target, Mobject):
            raise TypeError("add_transform expects two Mobjects")
        source_members = unique(source.family_members_with_points())
        target_members = unique(target.family_members_with_points())
        if not source_members or not target_members:
            return
        available_source = {id(obj) for obj in self.source_pieces}
        available_target = {id(obj) for obj in self.target_pieces}
        if (any(id(obj) not in available_source for obj in source_members)
                or any(id(obj) not in available_target for obj in target_members)):
            return
        factory = self.match_animation if source.has_same_shape_as(target) else self.mismatch_animation
        animation = factory(source, target, **self.anim_config)
        if not isinstance(animation, Animation):
            raise TypeError("A matching animation factory must return an Animation")
        # A failed factory must not consume either side of the match.
        self.anims.append(animation)
        source_ids, target_ids = {id(obj) for obj in source_members}, {id(obj) for obj in target_members}
        self.source_pieces[:] = [obj for obj in self.source_pieces if id(obj) not in source_ids]
        self.target_pieces[:] = [obj for obj in self.target_pieces if id(obj) not in target_ids]

    def find_pairs_with_matching_shapes(self, chars1, chars2):
        return [(source, target) for source in chars1 for target in chars2
                if source.has_same_shape_as(target)]

    def native_rate(animation):
        rate = animation.rate_func
        if rate is None or isinstance(rate, str):
            return rate
        name = g["_RATE_FUNC_NAMES"].get(rate)
        if name is not None:
            return name
        if not callable(rate):
            raise TypeError("rate_func must be a callable or a catalog name")
        frames = max(2, int(round(g["_composition_member_run_time"](animation) * 30.0)))
        return [float(rate(index / frames)) for index in range(frames + 1)]

    class PythonLeaf:
        def __init__(self, animation):
            self.animation = animation
            self.begun = False
            self.finished = False

        def get_run_time(self):
            return self.animation.get_run_time()

        def begin(self):
            self.begun, self.finished = True, False
            self.animation.begin()

        def update_mobjects(self, dt):
            self.animation.update_mobjects(dt)

        def interpolate(self, alpha):
            self.animation.interpolate(alpha)

        def finish(self):
            self.animation.finish()
            self.finished = True

        def clean_up_from_scene(self, scene):
            self.animation.clean_up_from_scene(scene)

        def abort(self):
            if not self.begun or self.finished:
                return
            # Several nested owners may unwind the same leaf. An authored
            # abort belongs to this execution once, even when it raises.
            self.finished = True
            animation = self.animation
            abort_animation = getattr(animation, "abort", None)
            if callable(abort_animation):
                abort_animation()
            else:
                animation.mobject.set_animating_status(False)
                if isinstance(animation, Transform):
                    animation.mobject.unlock_data()
                if (animation.suspend_mobject_updating
                        and getattr(animation, "mobject_was_updating", False)):
                    try:
                        animation.mobject.resume_updating(call_updater=False)
                    except TypeError:
                        animation.mobject.resume_updating()

    def make_driver(scene, animation):
        if isinstance(animation.mobject, g["CameraFrame"]):
            factory = g.get("_fmn_make_camera_driver")
            if factory is None:
                raise NotImplementedError("Matching animations cannot contain camera-frame tracks before camera initialization")
            return factory(scene, animation)
        if (g["_requires_python_animation"](animation)
                or (isinstance(animation, AnimationGroup)
                    and getattr(scene, "_fmn_camera_play_active", False)
                    and getattr(animation.begin, "__func__", animation.begin) is AnimationGroup.begin)):
            animation._ensure_runtime_defaults()
            if not isinstance(animation.mobject, Mobject):
                raise TypeError("A Python matching animation must animate a Mobject")
            if not animation.mobject._is_bound():
                scene._adopt(animation.mobject)
            return PythonLeaf(animation)
        if isinstance(animation, AnimationGroup):
            return CallbackDriver(animation, [make_driver(scene, member) for member in animation.animations])
        # This is the same narrow spec consumed by Scene.play. Only lowering
        # lives here: geometry, record alignment, leaf snapshots and frame
        # timing remain in the existing native drivers/interval builder.
        mobject = animation.mobject
        if isinstance(mobject, g["CameraFrame"]):
            raise NotImplementedError("Matching animations cannot contain camera-frame tracks")
        if not mobject._is_bound():
            scene._adopt(mobject)
        if animation._native_kind == "restore":
            saved = getattr(mobject, "saved_state", None)
            if saved is not None:
                if not saved._is_bound():
                    scene._adopt(saved)
                mobject._link_saved_state(saved)
        target = animation._native_target()
        if target is not None and not target._is_bound():
            scene._adopt(target)
        for extra in getattr(animation, "_native_extra_mobjects", ()):
            if not extra._is_bound():
                scene._adopt(extra)
        params = dict(animation._native_params())
        params["suspend_mobject_updating"] = bool(animation.suspend_mobject_updating)
        if animation.time_span is not None:
            params["time_span"] = animation.time_span
        spec = (
            animation._native_kind, mobject, target, animation.run_time,
            native_rate(animation), animation.lag_ratio, params,
        )
        return NativeLeaf(scene, scene._native_animation_driver(spec))

    # Shared lowering for authored compositions as well as matching plans.
    g["_fmn_make_animation_driver"] = make_driver
    g["_fmn_animated_mobjects"] = animated_mobjects

    def abort(self):
        driver = self._matching_driver
        if driver is None:
            return
        def unwind(child):
            if isinstance(child, CallbackDriver):
                errors = []
                for member in child.children:
                    try:
                        unwind(member)
                    except BaseException as error:
                        errors.append(error)
                if errors:
                    raise errors[0]
            else:
                child.abort()
        try:
            unwind(driver)
        finally:
            self._matching_driver = None
            self.mobject.set_animating_status(False)

    def drive(self, method, *args):
        if self._matching_driver is None:
            raise RuntimeError("Matching animation must begin before " + method)
        try:
            return getattr(self._matching_driver, method)(*args)
        except BaseException:
            # Abort does not finish an animation or publish the target. Keep
            # the original callback failure if a secondary cleanup also fails.
            try:
                self.abort()
            except BaseException:
                pass
            raise

    def begin(self):
        if not self.mobject._is_bound():
            raise RuntimeError("Matching animation begin requires a scene-bound mobject")
        if self._matching_driver is not None:
            self.abort()
        scene = self._matching_scene if self._matching_scene is not None else self.mobject._scene
        self._matching_driver = CallbackDriver(
            self, [make_driver(scene, member) for member in self.animations],
        )
        self.mobject.set_animating_status(True)
        drive(self, "begin")
        self.interpolate(0.0)

    def update_mobjects(self, dt):
        drive(self, "update_mobjects", dt)

    def interpolate(self, alpha):
        drive(self, "interpolate", float(alpha))

    def finish(self):
        drive(self, "finish")
        self.mobject.set_animating_status(False)

    def clean_up_from_scene(self, scene):
        drive(self, "clean_up_from_scene", scene)
        for mobject in (self.mobject, self.source):
            if getattr(mobject, "_is_bound", lambda: False)():
                try:
                    scene.remove(mobject)
                except Exception:
                    pass
        scene.add(self.target)
        self._matching_driver = None

    # Retain every already-published class object, including qualified
    # imports. Both front doors install this after bootstrap construction.
    if g.get("__engine__") == "FrankenManim":
        Parts._native_kind = "transform_matching_parts"
        Shapes._native_kind = "transform_matching_shapes"
        return
    Parts.__bases__ = (AnimationGroup,)
    Parts.__doc__ = "Match through public planning hooks, with native or authored leaves on Choreo's shared timeline."
    Parts._native_kind = None
    Shapes._native_kind = None
    methods = {
        "__init__": matching_init,
        "add_transform": add_transform,
        "find_pairs_with_matching_shapes": find_pairs_with_matching_shapes,
        "begin": begin, "update_mobjects": update_mobjects,
        "interpolate": interpolate, "finish": finish,
        "clean_up_from_scene": clean_up_from_scene, "abort": abort,
    }
    for name, function in methods.items():
        function.__name__ = name
        function.__qualname__ = f"{Parts.__qualname__}.{name}"
        function.__module__ = Parts.__module__
        setattr(Parts, name, function)

    original_play = g["Scene"].play

    def scene_play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        matches, seen = [], set()
        def visit(animation):
            if id(animation) in seen:
                return
            seen.add(id(animation))
            if isinstance(animation, Parts):
                matches.append(animation)
                animation._matching_scene = self
            if isinstance(animation, AnimationGroup):
                for member in animation.animations:
                    visit(member)
        for animation in proto_animations:
            visit(animation)
        try:
            return original_play(self, *proto_animations, run_time=run_time,
                                 rate_func=rate_func, lag_ratio=lag_ratio)
        except BaseException:
            # Errors in scene updaters or sibling animations must also unwind
            # the native leaves owned by a matching callback, even when the
            # failure did not originate inside that callback.
            for animation in reversed(matches):
                try:
                    animation.abort()
                except BaseException:
                    pass
            raise
        finally:
            for animation in matches:
                animation._matching_scene = None

    scene_play.__name__ = "play"
    scene_play.__qualname__ = g["Scene"].__qualname__ + ".play"
    scene_play.__module__ = g["Scene"].__module__
    g["Scene"].play = scene_play


def _install_matching_strings(g):
    """Dispatch authored block matching over Scribe's native byte-span parts."""
    Parts = g["TransformMatchingParts"]
    Strings = g["TransformMatchingStrings"]
    Tex = g["TransformMatchingTex"]
    Mobject = g["Mobject"]
    StringMobject = g["StringMobject"]

    def point_ids(mobject):
        return {id(part) for part in mobject.family_members_with_points()}

    def claimed_parts(self, source_keys, target_keys):
        available = [set().union(*(point_ids(part) for part, _ in keys))
                     for keys in (source_keys, target_keys)]
        claimed = [set(), set()]
        for pair in self.matched_pairs:
            for side, member in enumerate(pair):
                ids = point_ids(member)
                if not ids or not ids <= available[side]:
                    raise ValueError(type(self).__name__ + " matched_pairs member is not a live span-map part of the "
                                     + ("source" if side == 0 else "target") + " family")
                if ids & claimed[side]:
                    raise ValueError(type(self).__name__ + " matched_pairs claims the same part twice")
                claimed[side].update(ids)
        return claimed

    def strings_init(
        self, source, target, matched_keys=(), key_map=None, matched_pairs=(),
        run_time=2, lag_ratio=0, **kwargs,
    ):
        if not isinstance(source, StringMobject) or not isinstance(target, StringMobject):
            raise TypeError(type(self).__name__ + " expects two StringMobject instances")
        if not source._string_sub_spans or not target._string_sub_spans:
            raise g["_TexError"](type(self).__name__ + " requires non-empty native span maps")
        explicit = [tuple(pair) for pair in matched_pairs]
        if not all(len(pair) == 2 and all(isinstance(obj, Mobject) for obj in pair) for pair in explicit):
            raise TypeError(type(self).__name__ + " matched_pairs must pair Mobjects")
        self.matched_pairs = explicit
        self.matched_keys, self.key_map = tuple(matched_keys), dict(key_map or {})
        if not all(isinstance(key, str) for key in (*self.matched_keys, *self.key_map, *self.key_map.values())):
            raise TypeError("Matching string keys must be strings")
        # Validate even an authored matcher: no override bypasses the native
        # UTF-8 provenance or explicit-family ownership checks.
        claimed_parts(self, self._native_span_keys(source), self._native_span_keys(target))
        blocks = list(self.matching_blocks(source, target, self.matched_keys, self.key_map))
        Parts.__init__(self, source, target, matched_pairs=explicit + blocks,
                       run_time=run_time, lag_ratio=lag_ratio, **kwargs)
        self.matched_pairs = explicit

    def matching_blocks(self, source, target, matched_keys=(), key_map=None):
        keys = [self._native_span_keys(source), self._native_span_keys(target)]
        claimed = claimed_parts(self, *keys)
        sequences = [[key for _, key in side] for side in keys]
        masks = [object(), object()]
        used = [[bool(point_ids(part) & claimed[side]) for part, _ in entries]
                for side, entries in enumerate(keys)]
        for side in range(2):
            for index, taken in enumerate(used[side]):
                if taken:
                    sequences[side][index] = masks[side]
        pairs = []

        def group(side, indices):
            parts, seen = [], set()
            for index in indices:
                part = keys[side][index][0]
                if id(part) not in seen:
                    seen.add(id(part))
                    parts.append(part)
            if len(parts) == 1:
                return parts[0]
            cls = g["VGroup"] if all(isinstance(part, g["VMobject"]) for part in parts) else g["Group"]
            return cls(*parts)

        def claim(source_indices, target_indices):
            if any(used[side][index] for side, indices in enumerate((source_indices, target_indices)) for index in indices):
                return
            pairs.append((group(0, source_indices), group(1, target_indices)))
            for side, indices in enumerate((source_indices, target_indices)):
                consumed = set().union(*(point_ids(keys[side][index][0]) for index in indices))
                direct = set(indices)
                for index, (part, _) in enumerate(keys[side]):
                    if index in direct or point_ids(part) & consumed:
                        used[side][index] = True
                        sequences[side][index] = masks[side]

        def occurrences(side, key):
            if not key:
                return []
            mobject = (source, target)[side]
            encoded, needle = mobject.get_string().encode("utf-8"), key.encode("utf-8")
            result, position = [], 0
            while True:
                start = encoded.find(needle, position)
                if start < 0:
                    return result
                end = start + len(needle)
                position = end
                indices = [index for index, (a, b) in enumerate(mobject._string_sub_spans)
                           if a < end and b > start]
                if not indices or any(used[side][index] for index in indices):
                    continue
                if any(not start <= mobject._string_sub_spans[index][0]
                           < mobject._string_sub_spans[index][1] <= end for index in indices):
                    raise g["_TexError"]("Matching key " + repr(key) + " splits a native source-span part")
                result.append(indices)

        # Explicit pairs were masked above. Authored renames claim next,
        # followed by pinned keys. Never close gaps by deleting claimed slots:
        # that would invent adjacency across a moved substring.
        mapping = dict(key_map or {})
        for source_key, target_key in mapping.items():
            for source_indices, target_indices in zip(occurrences(0, source_key), occurrences(1, target_key)):
                claim(source_indices, target_indices)
        for key in matched_keys:
            if key in mapping:
                continue
            for source_indices, target_indices in zip(occurrences(0, key), occurrences(1, key)):
                claim(source_indices, target_indices)

        if not self._match_by_blocks:
            # D-09: Tex matches semantic native keys, never geometry. Retain
            # the existing Tex matched_keys admission filter for leftovers.
            admitted = set(matched_keys)
            for source_index, (_, source_key) in enumerate(keys[0]):
                if used[0][source_index] or (admitted and source_key not in admitted):
                    continue
                for target_index, (_, target_key) in enumerate(keys[1]):
                    if not used[1][target_index] and source_key == target_key:
                        claim([source_index], [target_index])
                        break
            return pairs

        # Repeated longest-block matching also handles reordered runs. The
        # two side-specific non-string sentinels cannot collide with authored
        # text (including the Reference's literal "Null1"/"Null2" strings).
        while True:
            matcher = g["_difflib"].SequenceMatcher(None, *sequences, autojunk=False)
            block = matcher.find_longest_match()
            if block.size == 0:
                break
            claim(list(range(block.a, block.a + block.size)),
                  list(range(block.b, block.b + block.size)))
        return pairs

    def no_shape_fallback(self, sources, targets):
        # Matching text by outline would discard the native semantic identity
        # when two unrelated glyphs happen to have the same shape.
        return []

    if g.get("__engine__") == "FrankenManim":
        Strings._native_kind = "transform_matching_strings"
        Tex._native_kind = "transform_matching_tex"
        Strings.matching_blocks = matching_blocks
        Strings.find_pairs_with_matching_shapes = no_shape_fallback
        return
    Strings.__bases__ = (Parts,)
    Strings._native_kind = None
    Tex._native_kind = None
    methods = {
        "__init__": strings_init, "matching_blocks": matching_blocks,
        "find_pairs_with_matching_shapes": no_shape_fallback,
    }
    for name, function in methods.items():
        function.__name__ = name
        function.__qualname__ = f"{Strings.__qualname__}.{name}"
        function.__module__ = Strings.__module__
        setattr(Strings, name, function)


def _install_composition_lifecycle(g):
    """Execute authored group protocols without replacing native leaf kernels."""
    AnimationGroup = g["AnimationGroup"]
    Mobject = g["Mobject"]
    CallbackDriver = g["_CompositionCallbackDriver"]
    make_driver = g["_fmn_make_animation_driver"]
    animated_mobjects = g["_fmn_animated_mobjects"]
    original_requires = g["_requires_python_animation"]
    original_play = g["Scene"].play
    original_init = AnimationGroup.__init__
    original_driver_init = CallbackDriver.__init__
    original_driver_interpolate = CallbackDriver.interpolate

    # Some shipped group subclasses (notably following Flash) deliberately
    # implement a leaf-style begin via Animation.begin, not a child driver.
    # Freeze their previously inherited lifecycle before installing the group
    # protocol; otherwise that begin would call our driver-only interpolate.
    legacy_methods = ("_ensure_runtime_defaults", "get_all_mobjects", "begin",
                      "update_mobjects", "interpolate", "finish", "clean_up_from_scene")
    legacy_classes = [
        cls for cls in set(value for value in g.values() if isinstance(value, type))
        if cls is not AnimationGroup and cls is not g["TransformMatchingParts"]
        and issubclass(cls, AnimationGroup) and "begin" in vars(cls)
    ]
    for cls in legacy_classes:
        inherited = {name: getattr(cls, name) for name in legacy_methods if name not in vars(cls)}
        for name, function in inherited.items():
            setattr(cls, name, function)

    def legacy_abort(self):
        self.mobject.set_animating_status(False)
        if self.suspend_mobject_updating and getattr(self, "mobject_was_updating", False):
            self.mobject.resume_updating()

    for cls in legacy_classes:
        if not hasattr(cls, "abort"):
            cls.abort = legacy_abort

    def timing_profile(animation):
        return tuple((id(member), start, end) for member, start, end in animation.anims_with_timings)

    def group_init(self, *animations, run_time=-1, lag_ratio=None, group=None, group_type=None, **kwargs):
        if group is not None and not isinstance(group, Mobject):
            raise TypeError("AnimationGroup group must be a Mobject")
        if group is None and group_type is not None and not callable(group_type):
            raise TypeError("AnimationGroup group_type must be callable")
        original_init(self, *animations, run_time=run_time, lag_ratio=lag_ratio, **kwargs)
        self._composition_authored_root = group is not None or group_type is not None
        self._composition_initial_timings = timing_profile(self)
        if group is not None:
            self.mobject = self.group = group
        elif group_type is not None:
            root = group_type(*member_objects(self))
            if not isinstance(root, Mobject):
                raise TypeError("AnimationGroup group_type must return a Mobject")
            self.mobject = self.group = root

    def member_objects(animation, visiting=None):
        visiting = set() if visiting is None else visiting
        if id(animation) in visiting:
            raise ValueError("Animation composition contains a cycle")
        visiting.add(id(animation))
        objects, seen = [], set()
        try:
            for child in animation.animations:
                if isinstance(child, AnimationGroup) and child.mobject is None:
                    ensure_root(child, visiting)
                for obj in animated_mobjects(child):
                    if id(obj) not in seen:
                        seen.add(id(obj))
                        objects.append(obj)
        finally:
            visiting.remove(id(animation))
        return objects

    def ensure_root(animation, visiting=None):
        if animation.mobject is None:
            existing = getattr(animation, "group", None)
            if isinstance(existing, Mobject):
                animation.mobject = existing
            else:
                objects = member_objects(animation, visiting)
                group_type = g["VGroup"] if all(isinstance(obj, g["VMobject"]) for obj in objects) else g["Group"]
                animation.mobject = group_type(*objects)
        if not isinstance(animation.mobject, Mobject):
            raise TypeError("AnimationGroup must animate a Mobject group")
        animation.group = animation.mobject
        return animation.mobject

    def ensure_runtime_defaults(self):
        if self.run_time is None or self.run_time < 0:
            self.calculate_max_end_time()
        if self.rate_func is None:
            self.rate_func = g["_linear_rate"]
        if self.lag_ratio is None:
            self.lag_ratio = type(self)._default_lag_ratio

    def abort_children(driver):
        if isinstance(driver, CallbackDriver):
            first_error = None
            for child in driver.children:
                if child is None:
                    continue
                try:
                    abort_children(child)
                except BaseException as error:
                    if first_error is None:
                        first_error = error
            if first_error is not None:
                raise first_error
        else:
            driver.abort()

    def release(self, *, call_updater=True):
        cameras = getattr(self, "_composition_camera_resumes", ())
        self._composition_camera_resumes = []
        try:
            self.mobject.set_animating_status(False)
            if getattr(self, "_composition_resumes_updating", False):
                self._composition_resumes_updating = False
                try:
                    self.mobject.resume_updating(call_updater=call_updater)
                except TypeError:
                    self.mobject.resume_updating()
        finally:
            for camera in cameras:
                try:
                    camera.resume_updating(call_updater=call_updater)
                except TypeError:
                    camera.resume_updating()

    def abort(self):
        driver = getattr(self, "_composition_driver", None)
        if driver is None:
            return
        try:
            abort_children(driver)
        finally:
            self._composition_driver = None
            release(self, call_updater=False)

    def drive(self, method, *args):
        driver = getattr(self, "_composition_driver", None)
        if driver is None:
            raise RuntimeError("AnimationGroup must begin before " + method)
        try:
            return getattr(driver, method)(*args)
        except BaseException:
            try:
                abort(self)
            except BaseException:
                pass
            raise

    def begin(self):
        if getattr(self, "_composition_driver", None) is not None:
            abort(self)
        root = ensure_root(self)
        scene = getattr(self, "_composition_scene", None)
        if scene is None:
            scene = next((obj._scene for obj in root.get_family() if obj._is_bound()), None)
        if scene is None:
            raise RuntimeError("AnimationGroup.begin requires a scene-bound group or member; use Scene.play(group)")
        if not root._is_bound():
            scene._adopt(root)
        self._ensure_runtime_defaults()
        # The existing driver owns the interval algebra, eager simultaneous
        # begin, and just-in-time Succession transitions. No second clock.
        self._composition_driver = CallbackDriver(
            self, [make_driver(scene, child) for child in self.animations],
        )
        self._composition_resumes_updating = False
        self._composition_camera_resumes = []
        try:
            root.set_animating_status(True)
            if self.suspend_mobject_updating and not root._is_updating_suspended():
                self._composition_resumes_updating = True
                root.suspend_updating()
            if self.suspend_mobject_updating and "_fmn_group_camera_frames" in g:
                for camera in g["_fmn_group_camera_frames"](self):
                    if not camera._is_updating_suspended():
                        self._composition_camera_resumes.append(camera)
                        camera.suspend_updating()
            drive(self, "begin")
            self.interpolate(0.0)
        except BaseException:
            try:
                abort(self)
            except BaseException:
                pass
            raise

    def update_mobjects(self, dt):
        drive(self, "update_mobjects", dt)

    def interpolate(self, alpha):
        drive(self, "interpolate", float(alpha))

    def finish(self):
        drive(self, "finish")
        release(self)

    def clean_up_from_scene(self, scene):
        drive(self, "clean_up_from_scene", scene)
        if self.remover:
            scene.remove(self.mobject)
        self._composition_driver = None

    def get_all_mobjects(self):
        return ensure_root(self)

    methods = {
        "__init__": group_init,
        "_ensure_runtime_defaults": ensure_runtime_defaults,
        "begin": begin, "update_mobjects": update_mobjects,
        "interpolate": interpolate, "finish": finish,
        "clean_up_from_scene": clean_up_from_scene, "abort": abort,
        "get_all_mobjects": get_all_mobjects,
    }
    for name, function in methods.items():
        function.__name__ = name
        function.__qualname__ = AnimationGroup.__qualname__ + "." + name
        function.__module__ = AnimationGroup.__module__
        setattr(AnimationGroup, name, function)

    # Capture shipped subclasses too: numeric Flash, Broadcast, and other
    # native specializations must not become callbacks merely for having
    # their own built-in methods. Compare identities, including monkeypatches.
    hooks = ("begin", "finish", "interpolate", "update_mobjects",
             "clean_up_from_scene", "_ensure_runtime_defaults",
             "build_animations_with_timings", "calculate_max_end_time")
    protocols = {
        cls: {name: getattr(cls, name) for name in hooks}
        for cls in tuple(g.values())
        if isinstance(cls, type) and issubclass(cls, AnimationGroup)
    }

    def custom_timings(animation):
        initial = getattr(animation, "_composition_initial_timings", None)
        if initial is not None and timing_profile(animation) != initial:
            return True
        for cls in type(animation).__mro__:
            baseline = protocols.get(cls)
            if baseline is not None:
                return any(getattr(getattr(animation, name), "__func__", getattr(animation, name)) is not baseline[name]
                           for name in ("build_animations_with_timings", "calculate_max_end_time"))
        return False

    def authored_timings(group, children):
        import math
        rows = list(group.anims_with_timings)
        if len(rows) != len(group.animations) or len(children) != len(group.animations):
            raise ValueError("AnimationGroup timings need one row per animation")
        available = {}
        for index, member in enumerate(group.animations):
            available.setdefault(id(member), []).append(index)
        timings = []
        for member, start, end in rows:
            candidates = available.get(id(member), [])
            if not candidates:
                raise ValueError("AnimationGroup timing row contains a foreign or duplicate animation")
            index = candidates.pop(0)
            start, end = float(start), float(end)
            if not math.isfinite(start) or not math.isfinite(end) or end < start:
                raise ValueError("AnimationGroup timing bounds must be finite and ordered")
            timings.append((children[index], start, end))
        max_end = float(group.max_end_time)
        if not math.isfinite(max_end) or max_end < 0:
            raise ValueError("AnimationGroup max_end_time must be finite and nonnegative")
        return timings, max_end

    def driver_init(self, group, children, run_time=None, rate_func=None):
        original_driver_init(self, group, children, run_time=run_time, rate_func=rate_func)
        self._authored_timing_rows = custom_timings(group)
        if self._authored_timing_rows:
            self.timings, self.max_end = authored_timings(group, children)
            if self.successive:
                if any(a[1] > b[1] for a, b in zip(self.timings, self.timings[1:])):
                    raise ValueError("Succession timing rows must be ordered by start time")
                self.children = [child for child, _, _ in self.timings]

    def driver_interpolate(self, alpha):
        if not self._authored_timing_rows or self.successive:
            return original_driver_interpolate(self, alpha)
        time = g["_composition_timeline_position"](
            self.group, float(alpha), self.max_end, self.get_run_time(), self.rate_func,
        )
        # Authored row order controls interpolation order; eager begin,
        # helper updates and finish retain the original argument order.
        for child, start, end in self.timings:
            if child is not None:
                sub = 0.0 if end == start else min(max((time - start) / (end - start), 0.0), 1.0)
                child.interpolate(sub)

    CallbackDriver.__init__ = driver_init
    CallbackDriver.interpolate = driver_interpolate

    # Camera-containing plays use these same roots, ownership checks, timing
    # rows and abort traversal; they do not implement a second composition.
    g["_fmn_ensure_composition_root"] = ensure_root
    g["_fmn_validate_composition_timings"] = authored_timings
    g["_fmn_abort_animation_driver"] = abort_children

    def requires_python_animation(animation):
        if isinstance(animation, AnimationGroup):
            if getattr(animation, "_composition_authored_root", False) or custom_timings(animation):
                return True
            for cls in type(animation).__mro__:
                baseline = protocols.get(cls)
                if baseline is not None:
                    if any(getattr(getattr(animation, name), "__func__", getattr(animation, name)) is not expected
                           for name, expected in baseline.items()):
                        return True
                    break
        return original_requires(animation)

    def scene_play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        groups, seen, visiting = [], set(), set()

        def visit(animation):
            if not isinstance(animation, AnimationGroup):
                return
            if id(animation) in visiting:
                raise ValueError("Animation composition contains a cycle")
            if id(animation) in seen:
                return
            visiting.add(id(animation))
            for child in animation.animations:
                visit(child)
            visiting.remove(id(animation))
            seen.add(id(animation))
            if requires_python_animation(animation):
                if custom_timings(animation):
                    authored_timings(animation, animation.animations)
                ensure_root(animation)
                groups.append((animation, getattr(animation, "_composition_scene", None)))
                animation._composition_scene = self

        try:
            for animation in proto_animations:
                visit(animation)
            return original_play(self, *proto_animations, run_time=run_time,
                                 rate_func=rate_func, lag_ratio=lag_ratio)
        except BaseException:
            # An authored override may raise outside super(), or a sibling /
            # scene updater may fail after these native children have begun.
            for animation, _ in reversed(groups):
                try:
                    animation.abort()
                except BaseException:
                    pass
            raise
        finally:
            for animation, previous_scene in groups:
                animation._composition_scene = previous_scene

    scene_play.__name__ = "play"
    scene_play.__qualname__ = g["Scene"].__qualname__ + ".play"
    scene_play.__module__ = g["Scene"].__module__
    g["_requires_python_animation"] = requires_python_animation
    g["Scene"].play = scene_play


def _install_camera_pose(g):
    """Interpolate CameraFrame's real native pose, not its empty record buffer."""
    CameraFrame = g["CameraFrame"]
    np = g["_np"]
    copy_core = g["_copy"].copy
    lerp = g["_interpolate"]

    def control_points(frame):
        center = np.asarray(frame._core.center(), dtype=float)
        width, height = frame._core.shape()
        # The pinned CameraFrame uses center, left, right, bottom, top. A
        # path_func must see that same point array, including during zooms.
        return center + np.array([
            [0., 0., 0.], [-width / 2., 0., 0.], [width / 2., 0., 0.],
            [0., -height / 2., 0.], [0., height / 2., 0.],
        ])

    def interpolate(self, mobject1, mobject2, alpha, path_func=None):
        if not isinstance(mobject1, CameraFrame) or not isinstance(mobject2, CameraFrame):
            raise TypeError("CameraFrame interpolation requires two CameraFrame endpoints")
        alpha = float(alpha)
        if not np.isfinite(alpha):
            raise ValueError("CameraFrame interpolation alpha must be finite")
        if path_func is None:
            path_func = g["straight_path"]
        if not callable(path_func):
            raise TypeError("CameraFrame path_func must be callable")
        # Validate on a private native value before changing the live core.
        # Lumen owns dimension/FOV/quaternion validity and normalization; no
        # callback failure can leave a half-updated live camera pose.
        changes = []
        if "point" not in getattr(self, "locked_data_keys", ()):
            points = np.asarray(path_func(control_points(mobject1), control_points(mobject2), alpha), dtype=float)
            if points.shape != (5, 3) or not np.isfinite(points).all():
                raise ValueError("CameraFrame path_func must return a finite (5, 3) point array")
            changes.append(("set_center", tuple(points[0])))
            changes.append(("set_shape", (float(points[2, 0] - points[1, 0]),
                                          float(points[4, 1] - points[3, 1]))))
        locked = getattr(self, "locked_uniform_keys", ())
        if "fovy" not in locked:
            changes.append(("set_field_of_view", float(lerp(mobject1._core.field_of_view(),
                                                            mobject2._core.field_of_view(), alpha))))
        if "orientation" not in locked:
            # Same componentwise quaternion interpolation as CameraLerp in
            # the native bridge, normalized by Lumen on write.
            orientation = lerp(np.asarray(mobject1._core.orientation(), dtype=float),
                               np.asarray(mobject2._core.orientation(), dtype=float), alpha)
            changes.append(("set_orientation", tuple(orientation)))
        candidate = copy_core(self._core)
        for method, value in changes:
            getattr(candidate, method)(value)
        # Preserve the core identity held by the renderer and native tracks.
        # Publish the raw inputs, not an already-normalized quaternion, so
        # normalization occurs exactly once on the live value as in Lumen.
        for method, value in changes:
            getattr(self._core, method)(value)
        if changes:
            self.note_changed_data()
        return self

    interpolate.__name__ = "interpolate"
    interpolate.__qualname__ = CameraFrame.__qualname__ + ".interpolate"
    interpolate.__module__ = CameraFrame.__module__
    CameraFrame.interpolate = interpolate


def _install_camera_motion(g):
    """Give spatial camera animations a real pose protocol, not empty records.

    These are the same public classes used for drawables. Choreo continues to
    own sampling and composition; the CameraFrame core owns rotations and the
    VMobject path owns true-arclength sampling. No alternate camera clock or
    substitute point buffer is introduced.
    """
    if g.get("_FMN_CAMERA_MOTION_INSTALLED", False):
        return
    CameraFrame = g["CameraFrame"]
    supported = []
    Rotating = g.get("Rotating")
    if Rotating is not None:
        original_rotation_interpolate = Rotating.interpolate_mobject

        def rotating_interpolate(self, alpha):
            frame = self.mobject
            if not isinstance(frame, CameraFrame):
                return original_rotation_interpolate(self, alpha)
            starting = self.starting_mobject
            # A CameraFrame has no drawable records. Rotating's ordinary
            # family_members_with_points()/match_points() loop therefore
            # cannot restore its pose. Reset the spatial components before
            # each absolute-angle rotation, including begin(0) and finish,
            # so orientation does not accumulate with the frame count.
            # FOV is an optical uniform, not part of that spatial reset.
            frame._core.set_center(starting._core.center())
            frame._core.set_shape(starting._core.shape())
            frame._core.set_orientation(starting._core.orientation())
            # As for drawable Rotating, restoration precedes rate evaluation:
            # an authored easing function may inspect or modify the live pose.
            angle = self.rate_func(self.time_spanned_alpha(float(alpha))) * self.angle
            # Keep authored CameraFrame.rotate overrides on their receiver.
            # The dispatched method owns validation; pre-calling the stock
            # core would wrongly reject inputs an override knows how to use.
            # Like CameraFrame.rotate itself, about_point/about_edge do not
            # translate the frame; the native core rotates its orientation.
            frame.rotate(angle, axis=self.axis, about_point=self.about_point,
                         about_edge=self.about_edge)

        rotating_interpolate.__name__ = "interpolate_mobject"
        rotating_interpolate.__qualname__ = Rotating.__qualname__ + ".interpolate_mobject"
        rotating_interpolate.__module__ = Rotating.__module__
        Rotating.interpolate_mobject = rotating_interpolate
        supported.append(Rotating)

    PathMotion = g.get("MoveAlongPath")
    if PathMotion is not None:
        # The existing Python lifecycle calls path.point_from_proportion and
        # frame.move_to. Both already dispatch to the correct native core.
        supported.append(PathMotion)

    Maintain = g.get("MaintainPositionRelativeTo")
    if Maintain is not None:
        original_init = Maintain.__init__
        original_maintain_interpolate = Maintain.interpolate_mobject

        def maintain_init(self, mobject, tracked_mobject=None, **kwargs):
            original_init(self, mobject, tracked_mobject, **kwargs)
            if isinstance(mobject, CameraFrame):
                # Reference update.py captures the offset at construction,
                # not when a later Succession interval begins.
                self.diff = mobject.get_center() - tracked_mobject.get_center()

        def maintain_interpolate(self, alpha):
            if not isinstance(self.mobject, CameraFrame):
                return original_maintain_interpolate(self, alpha)
            self.mobject.shift(self.tracked_mobject.get_center()
                               - self.mobject.get_center() + self.diff)

        # The separately installed wheel's all-mobject tracking adapter
        # must not sample authored center getters a second time.
        maintain_init._fmn_captures_camera_offset = True
        for name, function in (("__init__", maintain_init),
                               ("interpolate_mobject", maintain_interpolate)):
            function.__name__ = name
            function.__qualname__ = Maintain.__qualname__ + "." + name
            function.__module__ = Maintain.__module__
            setattr(Maintain, name, function)
        supported.append(Maintain)
    g["_fmn_camera_motion_types"] = tuple(supported)
    g["_FMN_CAMERA_MOTION_INSTALLED"] = True


def _install_camera_choreography(g):
    """Run camera and drawable animations on the same Choreo release boundary."""
    import math
    CameraFrame = g["CameraFrame"]
    Animation = g["Animation"]
    AnimationGroup = g["AnimationGroup"]
    Transform = g["Transform"]
    Builder = g["_AnimationBuilder"]
    original_play = g["Scene"].play
    original_dispatch = CameraFrame._dispatch_updater
    make_driver = g["_fmn_make_animation_driver"]
    abort_driver = g["_fmn_abort_animation_driver"]

    def camera_dispatch(self, updater, dt):
        # The camera is intentionally outside the Stage's suspended-subtree
        # walk. Its dedicated first-in-scene updater pass needs the same gate.
        if not self._is_updating_suspended():
            return original_dispatch(self, updater, dt)

    def validate_camera(scene, animation):
        if getattr(animation, "_fmn_allow_camera_callback", False):
            return
        if animation.mobject is not scene.frame:
            raise ValueError("Camera animation must target this Scene.frame")
        if animation.remover or getattr(animation, "replace_mobject_with_target_in_scene", False):
            raise NotImplementedError("Camera animation cannot remove or replace the scene's camera identity")
        if not getattr(animation, "_native_kind", None):
            raise NotImplementedError(
                "Python-callback animations of the camera frame await the "
                "camera track's per-frame callback seam; use frame.animate or "
                "Transform onto a CameraFrame target"
            )
        if not isinstance(animation, (Transform, *g.get("_fmn_camera_motion_types", ()))):
            raise NotImplementedError(type(animation).__name__ + " has no camera-pose animation protocol; use Transform or frame.animate")
        # The live camera deliberately has no Stage owner. Check helper
        # ownership against the Scene explicitly instead of comparing with
        # camera._scene (which would silently allow foreign paths/targets).
        for class_name, attribute in (("MoveAlongPath", "path"),
                                      ("MaintainPositionRelativeTo", "tracked_mobject")):
            cls = g.get(class_name)
            if cls is not None and isinstance(animation, cls):
                helper = getattr(animation, attribute)
                for member in helper.get_family():
                    owner = getattr(member, "_scene", None)
                    if owner is not None and owner is not scene:
                        error = g.get("_ForeignStageError", ValueError)
                        raise error("Camera motion cannot reference a " + attribute
                                    + " from another Scene; copy it")
        if isinstance(animation, Transform) and animation._target_attr is None:
            raise NotImplementedError("A camera Transform requires a camera target")
        target = getattr(animation, "target_mobject", None)
        if target is not None and not isinstance(target, CameraFrame):
            raise TypeError("Camera Transform target must be a CameraFrame")

    def normalize_rate(animation):
        rate = animation.rate_func
        if isinstance(rate, str):
            function = next((function for function, name in g["_RATE_FUNC_NAMES"].items() if name == rate), None)
            if function is None:
                raise ValueError("unknown rate function: " + rate)
            animation.rate_func = function
        elif rate is not None and not callable(rate):
            raise TypeError("rate_func must be a callable or a catalog name")

    class CameraLeaf:
        def __init__(self, scene, animation):
            validate_camera(scene, animation)
            animation._ensure_runtime_defaults()
            normalize_rate(animation)
            self.animation, self.begun, self.finished = animation, False, False
            self.was_suspended = True

        def get_run_time(self):
            return self.animation.get_run_time()

        def begin(self):
            self.begun, self.finished = True, False
            self.was_suspended = self.animation.mobject._is_updating_suspended()
            self.animation.begin()

        def update_mobjects(self, dt):
            self.animation.update_mobjects(dt)

        def interpolate(self, alpha):
            self.animation.interpolate(alpha)

        def finish(self):
            if self.begun and not self.finished:
                self.animation.finish()
                self.finished = True

        def clean_up_from_scene(self, scene):
            self.animation.clean_up_from_scene(scene)

        def abort(self):
            if not self.begun or self.finished:
                return
            self.finished = True
            animation = self.animation
            custom = getattr(animation, "abort", None)
            if callable(custom):
                custom()
            else:
                animation.mobject.set_animating_status(False)
                if isinstance(animation, Transform):
                    animation.mobject.unlock_data()
                if (not self.was_suspended and animation.suspend_mobject_updating
                        and animation.mobject._is_updating_suspended()):
                    animation.mobject_was_updating = False
                    try:
                        animation.mobject.resume_updating(call_updater=False)
                    except TypeError:
                        animation.mobject.resume_updating()

    g["_fmn_make_camera_driver"] = CameraLeaf

    def group_camera_frames(group):
        result, seen, frames = [], set(), set()
        stack = list(group.animations)
        while stack:
            animation = stack.pop()
            if id(animation) in seen:
                continue
            seen.add(id(animation))
            if isinstance(animation.mobject, CameraFrame) and id(animation.mobject) not in frames:
                frames.add(id(animation.mobject))
                result.append(animation.mobject)
            if isinstance(animation, AnimationGroup):
                stack.extend(animation.animations)
        return result

    g["_fmn_group_camera_frames"] = group_camera_frames

    def contains_camera(animation, visiting):
        if id(animation) in visiting:
            raise ValueError("Animation composition contains a cycle")
        visiting.add(id(animation))
        try:
            if isinstance(getattr(animation, "mobject", None), CameraFrame):
                return True
            if any(isinstance(obj, CameraFrame) for obj in getattr(animation, "_native_extra_mobjects", ())):
                return True
            if isinstance(animation, Builder):
                overridden = getattr(animation, "overridden_animation", None)
                return overridden is not None and contains_camera(overridden, visiting)
            if isinstance(animation, AnimationGroup):
                # Do not short-circuit: a later member might contain a cycle.
                results = [contains_camera(member, visiting) for member in animation.animations]
                return any(results)
            return False
        finally:
            visiting.remove(id(animation))

    class ClockDriver:
        """One non-rendering timing slot; no independent sampling or clock."""
        def __init__(self, scene, slot, children, animations):
            self.scene, self.slot, self.children = scene, slot, children
            # Retain the actual public roots separately from the private
            # timing slot. The execution owner snapshots these BEFORE begin;
            # the point-free slot has none of their suspension/lock state.
            self.animations = tuple(animations)
            self.run_times = [float(child.get_run_time()) for child in children]
            if any(not math.isfinite(value) or value < 0 for value in self.run_times):
                raise ValueError("Camera play runtimes must be finite and nonnegative")
            self.run_time = max(self.run_times, default=0.)
            self.dt = 0.
            self.completed = 0

        def hide_slot(self):
            g["_SceneCore"].remove(self.scene, self.slot)

        def begin(self):
            self.hide_slot()
            for child in self.children:
                child.begin()

        def update_mobjects(self, dt):
            self.hide_slot()
            self.dt = dt

        def interpolate(self, alpha):
            time = float(alpha) * self.run_time
            # Match the engine's per-animation step-1/step-2 ordering, not
            # all helper updates followed by all interpolations. Later
            # siblings must observe earlier siblings' current-frame state.
            for child, duration in zip(self.children, self.run_times):
                child.update_mobjects(self.dt)
                child.interpolate(1. if duration == 0 else time / duration)

        def finish(self):
            self.hide_slot()
            # Top-level Choreo finish order is finish + cleanup per child.
            # An enclosing composition still owns its own member protocol.
            while self.completed < len(self.children):
                child = self.children[self.completed]
                child.finish()
                child.clean_up_from_scene(self.scene)
                self.completed += 1

        def clean_up_from_scene(self, scene):
            self.hide_slot()

        def abort(self):
            first = None
            for child in self.children:
                try:
                    abort_driver(child)
                except BaseException as error:
                    if first is None:
                        first = error
            if first is not None:
                raise first

    g["_fmn_camera_clock_driver_type"] = ClockDriver

    def scene_play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        if getattr(self, "_fmn_camera_play_active", False):
            raise RuntimeError("Reentrant Scene.play during camera choreography is not supported")
        has_camera = [contains_camera(animation, set()) for animation in proto_animations]
        if not any(has_camera):
            return original_play(self, *proto_animations, run_time=run_time,
                                 rate_func=rate_func, lag_ratio=lag_ratio)
        animations = [g["prepare_animation"](animation) for animation in proto_animations]
        nodes, seen, visiting = [], set(), set()

        def visit(animation):
            if not isinstance(animation, Animation):
                raise TypeError("Camera compositions accept Animation instances")
            if id(animation) in visiting:
                raise ValueError("Animation composition contains a cycle")
            if id(animation) in seen:
                return
            visiting.add(id(animation))
            if isinstance(animation, AnimationGroup):
                for child in animation.animations:
                    visit(child)
                g["_fmn_validate_composition_timings"](animation, animation.animations)
            elif isinstance(animation.mobject, CameraFrame):
                validate_camera(self, animation)
            elif any(isinstance(obj, CameraFrame) for obj in getattr(animation, "_native_extra_mobjects", ())):
                raise NotImplementedError("A camera animation requires its own Transform; it cannot be an extra drawable mobject")
            visiting.remove(id(animation))
            seen.add(id(animation))
            nodes.append(animation)

        for animation in animations:
            visit(animation)
        for animation in animations:
            if run_time is not None:
                value = float(run_time)
                if not math.isfinite(value) or value < 0:
                    raise ValueError("Camera play run_time must be finite and nonnegative")
                animation.run_time = value
            if rate_func is not None:
                animation.rate_func = rate_func
            if lag_ratio is not None:
                value = float(lag_ratio)
                if not math.isfinite(value) or value < 0:
                    raise ValueError("Camera play lag_ratio must be finite and nonnegative")
                animation.lag_ratio = value
        for animation in nodes:
            normalize_rate(animation)
        contexts, children = [], []
        drawable = {}
        for animation in nodes:
            drawable[id(animation)] = (
                getattr(animation, "_composition_authored_root", False)
                or any(drawable[id(child)] for child in animation.animations)
            ) if isinstance(animation, AnimationGroup) else not isinstance(animation.mobject, CameraFrame)
        slot = g["Mobject"]()
        failed = False
        self._fmn_camera_play_active = True
        try:
            for animation in nodes:
                if isinstance(animation, AnimationGroup):
                    root = g["_fmn_ensure_composition_root"](animation)
                    if any(isinstance(member, CameraFrame) for member in root.get_family()):
                        raise ValueError("A composition's drawable group cannot contain CameraFrame")
                    for name in ("_composition_scene", "_matching_scene"):
                        contexts.append((animation, name, getattr(animation, name, None)))
                        setattr(animation, name, self)
            # Keep the camera itself detached. Adopt ordinary animated roots
            # through the existing arena path; the private point-free slot
            # is removed before every user callback/updater and at teardown.
            # Like the Reference's begin_animations, add only roots not yet
            # in the scene: `add` re-appends, which would move a present
            # root to the top of the draw order (fm-kpvx).
            present = set(self.get_mobject_family_members())
            for animation in animations:
                if drawable[id(animation)]:
                    for root in g["_fmn_animated_mobjects"](animation):
                        if root not in present:
                            self.add(root)
                            present.update(root.get_family())
                children.append(make_driver(self, animation))
            self._adopt(slot)
            clock = ClockDriver(self, slot, children, animations)
            spec = ("python_callback", slot, None, clock.run_time, None, 0., {"remover": True})
            return self._play_animations([spec], [clock], None, None, None, None)
        except BaseException as primary:
            failed = True
            for child in children:
                try:
                    abort_driver(child)
                except BaseException as cleanup_error:
                    try:
                        BaseException.add_note(primary, "camera animation abort also failed: "
                                               + type(cleanup_error).__name__)
                    except BaseException:
                        pass
            raise
        finally:
            try:
                if slot._is_bound():
                    g["_SceneCore"].remove(self, slot)
            except BaseException:
                if not failed:
                    raise
            finally:
                self._fmn_camera_play_active = False
                for animation, name, previous in reversed(contexts):
                    setattr(animation, name, previous)

    camera_dispatch.__name__ = "_dispatch_updater"
    camera_dispatch.__qualname__ = CameraFrame.__qualname__ + "._dispatch_updater"
    camera_dispatch.__module__ = CameraFrame.__module__
    CameraFrame._dispatch_updater = camera_dispatch
    scene_play.__name__ = "play"
    scene_play.__qualname__ = g["Scene"].__qualname__ + ".play"
    scene_play.__module__ = g["Scene"].__module__
    g["Scene"].play = scene_play


def _install_partial_reveals(g):
    """Dispatch authored reveal windows to the existing native partial kernels."""
    import math
    Animation = g["Animation"]
    Partial = g["ShowPartial"]
    Creation = g["ShowCreation"]
    Uncreate = g["Uncreate"]
    Passing = g["ShowPassingFlash"]
    Scene = g["Scene"]
    original_requires = g["_requires_python_animation"]
    original_play = Scene.play
    creation_params = Creation._native_params
    uncreate_params = Uncreate._native_params

    def implementation(method):
        return getattr(method, "__func__", method)

    def normalize_rate(self):
        if isinstance(self.rate_func, str):
            name = self.rate_func
            function = next((fn for fn, label in g["_RATE_FUNC_NAMES"].items()
                             if label == name), None)
            if function is None:
                raise ValueError("unknown rate function: " + name)
            self.rate_func = function

    def reverse_smooth(alpha):
        return g.get("smooth", g["_smooth_rate"])(1.0 - alpha)

    def ensure_defaults(self):
        if self.rate_func is None and isinstance(self, Uncreate):
            self.rate_func = reverse_smooth
        Animation._ensure_runtime_defaults(self)
        normalize_rate(self)

    def abort(self):
        if not getattr(self, "_partial_active", False):
            return
        self._partial_active = False
        try:
            self.mobject.set_animating_status(False)
        finally:
            if (self.suspend_mobject_updating
                    and not self._partial_was_suspended
                    and self.mobject._is_updating_suspended()):
                self.mobject_was_updating = False
                # Failure unwinding must not execute another authored updater.
                try:
                    self.mobject.resume_updating(call_updater=False)
                except TypeError:
                    self.mobject.resume_updating()

    def abort_preserving_error(self):
        try:
            abort(self)
        except BaseException:
            pass

    def begin(self):
        abort(self)
        self._partial_was_suspended = self.mobject._is_updating_suspended()
        self._partial_active = True
        try:
            Animation.begin(self)
        except BaseException:
            abort_preserving_error(self)
            raise

    def interpolate(self, alpha):
        try:
            alpha = float(alpha)
            if not math.isfinite(alpha):
                raise ValueError("Partial reveal alpha must be finite")
            # Scene.play may install its rate override after normalization.
            normalize_rate(self)
            Animation.interpolate(self, alpha)
        except BaseException:
            abort_preserving_error(self)
            raise

    def update_mobjects(self, dt):
        try:
            Animation.update_mobjects(self, dt)
        except BaseException:
            abort_preserving_error(self)
            raise

    def interpolate_submobject(self, submob, start_submob, alpha):
        # Do not probe or approximate an authored function. A rule which
        # agrees with a stock rule at four samples need not agree elsewhere,
        # and closures may legitimately depend on live scene state.
        lower, upper = self.get_bounds(alpha)
        lower, upper = float(lower), float(upper)
        if not math.isfinite(lower) or not math.isfinite(upper):
            raise ValueError(type(self).__name__ + ".get_bounds must return two finite bounds")
        # VMobject and Surface own clipping/alignment in their native kernels.
        # In particular, never flatten a Surface into a VMobject record schema.
        submob.pointwise_become_partial(start_submob, lower, upper)

    def finish(self):
        try:
            Animation.finish(self)
            self._partial_active = False
        except BaseException:
            abort_preserving_error(self)
            raise

    def passing_finish(self):
        finish(self)
        for submob, start in self.get_all_families_zipped():
            submob.pointwise_become_partial(start, 0.0, 1.0)

    def uncreate_init(self, mobject, rate_func=None, remover=True,
                      should_match_start=True, **kwargs):
        Creation.__init__(self, mobject, rate_func=rate_func, remover=remover,
                          should_match_start=should_match_start, **kwargs)
        # ShowPartial leaves a point cloud or plain Group Python-driven.
        if self._native_kind is not None:
            self._native_kind = ("uncreate_surface" if isinstance(mobject, g["Surface"])
                                 and hasattr(mobject, "resolution") else "uncreate")

    def passing_init(self, mobject, time_width=0.1, remover=True, **kwargs):
        self.time_width = float(time_width)
        if not math.isfinite(self.time_width) or self.time_width < 0:
            raise ValueError("ShowPassingFlash time_width must be finite and nonnegative")
        Partial.__init__(self, mobject, remover=remover, **kwargs)

    def partial_params(self):
        return {"remover": self.remover, "final_alpha_value": self.final_alpha_value}

    def show_params(self):
        return {**creation_params(self), **partial_params(self)}

    def uncreate_native_params(self):
        return {**uncreate_params(self), **partial_params(self)}

    def passing_params(self):
        return {"time_width": self.time_width, **partial_params(self)}

    # Fix the hierarchy in place, preserving all previously published names.
    Passing.__bases__ = (Partial,)
    Partial.__doc__ = "Reveal native curves or surfaces through a live, overridable get_bounds rule."
    methods = {
        Partial: {"begin": begin, "finish": finish, "interpolate": interpolate,
                  "update_mobjects": update_mobjects, "abort": abort,
                  "interpolate_submobject": interpolate_submobject,
                  "_ensure_runtime_defaults": ensure_defaults},
        Creation: {"_native_params": show_params},
        Uncreate: {"__init__": uncreate_init, "_native_params": uncreate_native_params},
        Passing: {"__init__": passing_init, "finish": passing_finish, "_native_params": passing_params},
    }
    for cls, entries in methods.items():
        for name, function in entries.items():
            function.__name__ = name
            function.__qualname__ = cls.__qualname__ + "." + name
            function.__module__ = cls.__module__
            setattr(cls, name, function)

    hooks = ("get_bounds", "begin", "finish", "interpolate", "interpolate_mobject",
             "interpolate_submobject", "update_mobjects", "clean_up_from_scene",
             "create_starting_mobject", "get_all_mobjects", "get_all_families_zipped",
             "get_all_mobjects_to_update", "get_sub_alpha", "time_spanned_alpha",
             "_ensure_runtime_defaults")
    protocols = {
        cls: {name: implementation(getattr(cls, name)) for name in hooks}
        for cls in tuple(g.values()) if isinstance(cls, type) and issubclass(cls, Partial)
    }
    object_protocols = {
        cls: implementation(getattr(cls, "pointwise_become_partial"))
        for cls in tuple(g.values()) if isinstance(cls, type)
        and issubclass(cls, g["Mobject"]) and hasattr(cls, "pointwise_become_partial")
    }

    def requires_python(animation):
        if not isinstance(animation, Partial):
            return original_requires(animation)
        if not getattr(animation, "_native_kind", None):
            return True
        if isinstance(animation, (Uncreate, Passing)) and not animation.remover:
            return True
        if isinstance(animation, Passing) and isinstance(animation.mobject, g["Surface"]):
            return True
        if animation.final_alpha_value != 1.0:
            return True
        for cls in type(animation).__mro__:
            baseline = protocols.get(cls)
            if baseline is not None:
                if any(implementation(getattr(animation, name)) is not expected
                       for name, expected in baseline.items()):
                    return True
                break
        for member in animation.mobject.get_family():
            for cls in type(member).__mro__:
                expected = object_protocols.get(cls)
                if expected is not None:
                    if implementation(member.pointwise_become_partial) is not expected:
                        return True
                    break
        return False

    def scene_play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        # An override_animate builder already owns its resulting animation.
        # Resolve just that case once, leaving ordinary builder lowering alone.
        animations = tuple(g["prepare_animation"](anim)
                           if isinstance(anim, g["_AnimationBuilder"])
                           and getattr(anim, "overridden_animation", None) is not None
                           else anim for anim in proto_animations)
        reveals, seen = [], set()
        stack = list(animations)
        while stack:
            animation = stack.pop()
            if id(animation) in seen:
                continue
            seen.add(id(animation))
            if isinstance(animation, Partial):
                reveals.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend(animation.animations)
        try:
            return original_play(self, *animations, run_time=run_time,
                                 rate_func=rate_func, lag_ratio=lag_ratio)
        except BaseException:
            # Includes failures outside an overridden super() call, in a
            # sibling, or in a scene updater. Never finish/publish on failure.
            for animation in reveals:
                abort_preserving_error(animation)
            raise

    scene_play.__name__ = "play"
    scene_play.__qualname__ = Scene.__qualname__ + ".play"
    scene_play.__module__ = Scene.__module__
    g["_requires_python_animation"] = requires_python
    Scene.play = scene_play


def _install_border_write(g):
    """Execute the outline/reveal/fill protocol over native mobject operations."""
    import math
    Animation = g["Animation"]
    Border = g["DrawBorderThenFill"]
    Write = g["Write"]
    VMobject = g["VMobject"]
    Scene = g["Scene"]
    original_requires = g["_requires_python_animation"]
    original_play = Scene.play
    original_write_init = Write.__init__
    original_border_params = Border._native_params

    def implementation(method):
        return getattr(method, "__func__", method)

    def normalize_rate(self):
        if isinstance(self.rate_func, str):
            name = self.rate_func
            function = next((fn for fn, label in g["_RATE_FUNC_NAMES"].items()
                             if label == name), None)
            if function is None:
                raise ValueError("unknown rate function: " + name)
            self.rate_func = function

    def ensure_defaults(self):
        if self.rate_func is None:
            self.rate_func = g["linear"] if isinstance(self, Write) else g["double_smooth"]
        Animation._ensure_runtime_defaults(self)
        normalize_rate(self)

    def abort(self):
        if not getattr(self, "_border_active", False):
            return
        self._border_active = False
        try:
            self.mobject.set_animating_status(False)
        finally:
            if (self.suspend_mobject_updating and not self._border_was_suspended
                    and self.mobject._is_updating_suspended()):
                self.mobject_was_updating = False
                # Failure unwinding must not execute another authored updater.
                try:
                    self.mobject.resume_updating(call_updater=False)
                except TypeError:
                    self.mobject.resume_updating()

    def abort_preserving_error(self):
        try:
            abort(self)
        except BaseException:
            pass

    def begin(self):
        abort(self)
        self._border_was_suspended = self.mobject._is_updating_suspended()
        self._border_active = True
        try:
            self._ensure_runtime_defaults()
            outline = self.get_outline()
            if not isinstance(outline, VMobject):
                raise TypeError(type(self).__name__ + ".get_outline must return a VMobject")
            live_ids = {id(member) for member in self.mobject.get_family()}
            if any(id(member) in live_ids for member in outline.get_family()):
                raise ValueError("A drawing outline must not alias the animated family; return a copy")
            self.outline = outline
            # Custom outlines may refine the path or family. Marionette owns
            # alignment and preserves geometry while equalizing record counts.
            self.mobject.align_data_and_family(self.outline)
            self.sm_to_index = {hash(member): 0 for member in self.mobject.get_family()}
            Animation.begin(self)
            # interpolate(0) already installs the appropriate outline or fill
            # state. A post-begin match_style would overwrite a nonzero rate(0).
        except BaseException:
            abort_preserving_error(self)
            raise

    def get_all_mobjects(self):
        return [*Animation.get_all_mobjects(self), self.outline]

    def interpolate_submobject(self, submob, start, outline, alpha):
        alpha = float(alpha)
        if not math.isfinite(alpha):
            raise ValueError("Drawing phase alpha must be finite")
        alpha = min(max(alpha, 0.0), 1.0)
        if alpha < 0.5:
            # Reestablish the full outline before clipping, including after
            # a backward seek or nonmonotonic easing crossed the fill phase.
            # These are native data/style operations, not a Python path kernel.
            submob.set_data(outline.data)
            submob.match_style(outline)
            submob.set_uniform(**outline.get_uniforms())
            submob.pointwise_become_partial(outline, 0.0, 2.0 * alpha)
            self.sm_to_index[hash(submob)] = 0
        else:
            if self.sm_to_index.get(hash(submob), 0) == 0:
                submob.set_data(outline.data)
            submob.interpolate(outline, start, 2.0 * alpha - 1.0)
            self.sm_to_index[hash(submob)] = 1

    def interpolate(self, alpha):
        try:
            alpha = float(alpha)
            if not math.isfinite(alpha):
                raise ValueError("Drawing animation alpha must be finite")
            normalize_rate(self)
            Animation.interpolate(self, alpha)
        except BaseException:
            abort_preserving_error(self)
            raise

    def update_mobjects(self, dt):
        try:
            Animation.update_mobjects(self, dt)
        except BaseException:
            abort_preserving_error(self)
            raise

    def finish(self):
        try:
            Animation.finish(self)
            self.mobject.refresh_joint_angles()
            self._border_active = False
        except BaseException:
            abort_preserving_error(self)
            raise

    def write_init(self, vmobject, run_time=-1, lag_ratio=-1, rate_func=None,
                   stroke_color=None, **kwargs):
        if not isinstance(vmobject, VMobject):
            raise TypeError("Write requires a VMobject")
        explicit_stroke_color = stroke_color
        if stroke_color is None:
            stroke_color = vmobject.get_color()
        original_write_init(self, vmobject, run_time=run_time, lag_ratio=lag_ratio,
                            rate_func=g["linear"] if rate_func is None else rate_func,
                            stroke_color=stroke_color, **kwargs)
        self._explicit_stroke_color = explicit_stroke_color

    def native_params(self):
        return {**original_border_params(self), "remover": self.remover,
                "final_alpha_value": self.final_alpha_value}

    def write_native_params(self):
        if getattr(self, "_explicit_stroke_color", None) is None:
            return {}
        if "_color_to_rgb" in g:
            return {"stroke_color": tuple(g["_color_to_rgb"](self.stroke_color))}
        return {"stroke_color": tuple(self.stroke_color)}

    methods = {
        Border: {"begin": begin, "finish": finish, "abort": abort,
                 "get_all_mobjects": get_all_mobjects, "interpolate": interpolate,
                 "interpolate_submobject": interpolate_submobject,
                 "update_mobjects": update_mobjects, "_ensure_runtime_defaults": ensure_defaults,
                 "_native_params": native_params},
        Write: {"__init__": write_init, "_native_params": write_native_params},
    }
    for cls, entries in methods.items():
        for name, function in entries.items():
            function.__name__ = name
            function.__qualname__ = cls.__qualname__ + "." + name
            function.__module__ = cls.__module__
            setattr(cls, name, function)

    hooks = ("get_outline", "begin", "finish", "interpolate", "interpolate_mobject",
             "interpolate_submobject", "update_mobjects", "clean_up_from_scene",
             "create_starting_mobject", "get_all_mobjects", "get_all_families_zipped",
             "get_all_mobjects_to_update", "get_sub_alpha", "time_spanned_alpha",
             "_ensure_runtime_defaults")
    protocols = {
        cls: {name: implementation(getattr(cls, name)) for name in hooks}
        for cls in tuple(g.values()) if isinstance(cls, type) and issubclass(cls, Border)
    }
    object_hooks = ("pointwise_become_partial", "interpolate")
    object_protocols = {
        cls: {name: implementation(getattr(cls, name)) for name in object_hooks}
        for cls in tuple(g.values()) if isinstance(cls, type) and issubclass(cls, VMobject)
    }

    def requires_python(animation):
        if not isinstance(animation, Border):
            return original_requires(animation)
        if not getattr(animation, "_native_kind", None):
            return True
        if animation.remover or animation.final_alpha_value != 1.0:
            return True
        # Choreo's Write constructor currently consumes stroke color but not
        # a nondefault border width. Execute the real protocol for that case.
        if isinstance(animation, Write) and animation.stroke_width != 2.0:
            return True
        for cls in type(animation).__mro__:
            baseline = protocols.get(cls)
            if baseline is not None:
                if any(implementation(getattr(animation, name)) is not expected
                       for name, expected in baseline.items()):
                    return True
                break
        for member in animation.mobject.get_family():
            for cls in type(member).__mro__:
                baseline = object_protocols.get(cls)
                if baseline is not None:
                    if any(implementation(getattr(member, name)) is not expected
                           for name, expected in baseline.items()):
                        return True
                    break
        return False

    def scene_play(self, *proto_animations, run_time=None, rate_func=None, lag_ratio=None):
        animations = tuple(g["prepare_animation"](anim)
                           if isinstance(anim, g["_AnimationBuilder"])
                           and getattr(anim, "overridden_animation", None) is not None
                           else anim for anim in proto_animations)
        drawings, seen = [], set()
        stack = list(animations)
        while stack:
            animation = stack.pop()
            if id(animation) in seen:
                continue
            seen.add(id(animation))
            if isinstance(animation, Border):
                drawings.append(animation)
            if isinstance(animation, g["AnimationGroup"]):
                stack.extend(animation.animations)
        try:
            return original_play(self, *animations, run_time=run_time,
                                 rate_func=rate_func, lag_ratio=lag_ratio)
        except BaseException:
            for animation in drawings:
                abort_preserving_error(animation)
            raise

    scene_play.__name__ = "play"
    scene_play.__qualname__ = Scene.__qualname__ + ".play"
    scene_play.__module__ = Scene.__module__
    g["_requires_python_animation"] = requires_python
    Scene.play = scene_play
