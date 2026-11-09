"""Vector groups use the shared Group ingestion and native VMobject lifecycle.

This establishes the public class lineage once; it does not wrap constructors
or introduce another family, copy, style or rendering implementation.
"""
from __future__ import annotations


def install_grouping(native):
    g = vars(native)
    if g.get("_FMN_GROUPING_INSTALLED", False):
        return
    Group, VGroup, VMobject, Mobject = (g[name] for name in (
        "Group", "VGroup", "VMobject", "Mobject",
    ))

    def group_init(self, *vmobjects, **kwargs):
        # Group precedes VMobject in the Reference MRO. Its cooperative
        # constructor initializes the vector record schema exactly once;
        # its public ingestion uses add(), including authored overrides.
        super(VGroup, self).__init__(**kwargs)
        if any(isinstance(mob, Mobject) and not isinstance(mob, VMobject)
               for mob in vmobjects):
            raise Exception("Only VMobjects can be passed into VGroup")
        self._ingest_args(*vmobjects)
        if self.submobjects:
            # Copy the values, never the live mapping's owner. This is the
            # Reference's post-ingestion uniform precedence, not recursive
            # restyling of the children or a constructor-default override.
            self.uniforms.update(self.submobjects[0].uniforms)

    def add(self, other):
        # Reference VGroup addition is mutating, not group concatenation into
        # a replacement object. Native add owns duplicate/scene/cycle checks.
        assert isinstance(other, VMobject)
        return self.add(other)

    # Preserve qualified aliases and already-defined library subclasses.
    # This runs before installers take method-identity dispatch snapshots.
    VGroup.__bases__ = (Group, VMobject)
    group_init.__annotations__ = dict(VGroup.__init__.__annotations__)
    for name, method in (("__init__", group_init), ("__add__", add)):
        method.__name__ = name
        method.__qualname__ = VGroup.__qualname__ + "." + name
        method.__module__ = VGroup.__module__
        setattr(VGroup, name, method)
    g["_FMN_GROUPING_INSTALLED"] = True
