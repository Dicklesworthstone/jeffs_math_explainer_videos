# Owned array state in mobject copies

Ordinary `Mobject.copy()` and `copy.copy(mobject)` recursively copy the native
family and direct NumPy-array attributes on every member. Editing a generated
`.animate` target must not change the original object's arrays or arrays on an
existing saved state before playback begins (plan §8.3).

The production initializer installs `fmn_python.copying` over the common native
family-copy helper. Native records, arena ownership, shared-child identities,
root membership, subclass allocation, updater lists and selected relationship
links remain the existing copier's responsibility. Public classes and method
signatures are unchanged; specialized callers of that helper also participate.

One memo covers arrays across the family. Repeated references to one array
remain aliases inside the copy but no longer alias the source. Object-array
slots referencing copied family members point at their corresponding copies;
external mobjects and other Python payloads retain identity. This includes
structured object fields, nested arrays, self-referential arrays, and deep
array chains. Shapes, dtypes and numeric bits are retained. Like NumPy's normal
array-copy operation, the materialized arrays are writable. Distinct overlapping
views become independent arrays; their shared-memory relationship is not rebuilt.

Ordinary Python lists/dictionaries remain shallow references, as before. This
change does not turn `copy()` into `deepcopy()`, copy callbacks or external
resources, or redefine extension-uniform copying. The existing deep-copy path
continues to use its own global memo and authored `__deepcopy__` behavior.
The stronger owned-container policy for SceneState remains a separate contract.

`tests/copying_semantics.py` exercises actual native objects and both bound and
detached copying, target isolation, aliasing, family maps, live native views,
existing memo entries and deep-copy compatibility. `tests/copying_render.py`
constructs a scene with an array-driven interpolator, renders four native Y4M
frames, independently decodes their luma planes and checks the expected curved
trajectory. The same frames must be identical at one and four renderer threads.
Both run in the existing installed-wheel acceptance gate. They provide bounded
semantic/render evidence, not complete corpus or cross-platform certification.
