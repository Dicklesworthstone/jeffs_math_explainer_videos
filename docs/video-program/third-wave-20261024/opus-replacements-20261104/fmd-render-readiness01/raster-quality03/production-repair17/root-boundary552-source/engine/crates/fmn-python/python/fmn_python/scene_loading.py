"""Scoped, source-fresh loading of Python scene projects.

A source is a real module for its entire render, not a transient ``runpy``
namespace. Package-relative imports, dataclasses, lazy imports and imports of
one's own scene module consequently see the same objects. Only project-local
modules are refreshed; interpreter/portal modules are never reloaded. This is
an authoring boundary, not a sandbox or a certified input-closure claim.
Namespace packages are scoped too; namespaces spanning the project and foreign
roots are refused rather than silently reusing another checkout's children.
"""
from __future__ import annotations

from collections.abc import Mapping
import hashlib
import importlib
import importlib.abc
import importlib.machinery
import importlib.util
import os
import sys
import threading
from pathlib import Path
from types import ModuleType
from typing import Any

from .runtime_paths import RuntimePaths

# sys.path, sys.meta_path and sys.modules are interpreter-wide. Reject another
# owner instead of blocking: authored code may join a thread trying to render.
_SOURCE_OWNER = threading.Lock()
_MISSING = object()
# Retaining source bytes must not make an ordinary render an unbounded archive.
# Exceeding either budget disables provenance capture, not Python execution.
_MAX_CAPTURED_SOURCES = 4096
_MAX_CAPTURED_SOURCE_BYTES = 64 * 1024 * 1024


def _origin(module: Any) -> Path | None:
    if not isinstance(module, ModuleType):
        return None
    filename = vars(module).get("__file__")
    if not isinstance(filename, str):
        return None
    try:
        return Path(filename).resolve()
    except (OSError, ValueError, RuntimeError):
        return None


def _protected(name: str) -> bool:
    return name in ("__main__", "manimlib", "fmn_python") or name.startswith(("manimlib.", "fmn_python."))


def _namespace_paths(module: Any) -> tuple[Path, ...]:
    if not isinstance(module, ModuleType):
        return ()
    spec = vars(module).get("__spec__")
    if spec is None or spec.origin is not None or spec.submodule_search_locations is None:
        return ()
    return tuple(Path(path).resolve() for path in vars(module).get("__path__", ()))


def _local_namespace(name: str, paths: tuple[Path, ...], root: Path) -> bool:
    runtime = RuntimePaths(root)
    local = [path.is_relative_to(root) and not runtime.contains(path) for path in paths]
    if any(local) and not all(local):
        raise ImportError(f"scene namespace {name!r} has locations outside the scene project")
    return bool(local) and all(local)


class _FreshNamespace(importlib.machinery.NamespaceLoader):
    def __init__(self, name: str, path: Any, owner: SceneSource) -> None:
        # Keep CPython's namespace search and resource-reader behavior. A bare
        # custom Loader would break importlib.resources for authored assets.
        super().__init__(name, path, importlib.machinery.PathFinder.find_spec)
        self.owner = owner

    def exec_module(self, module: ModuleType) -> None:
        module.__file__ = None
        if self.owner._reload_inputs is not None:
            self.owner._reload_inputs.remember_parent(module.__name__)
        self.owner._loaded[module.__name__] = module
        super().exec_module(module)


class _FreshSource(importlib.machinery.SourceFileLoader):
    def __init__(self, name: str, path: str, owner: SceneSource) -> None:
        super().__init__(name, path)
        self.owner = owner

    def get_code(self, fullname: str):
        # Timestamp/size-based pyc validation can miss an equal-length edit in
        # the same timestamp tick. Compile exactly the bytes observed here.
        path = Path(self.path).resolve()
        inputs = self.owner._reload_inputs
        source = self.get_data(self.path) if inputs is None else inputs.read(path)
        digest = hashlib.sha256(source).hexdigest()
        # Check the exact bytes about to execute, not just a scan after import.
        # Failed code must not get to run top-level authored effects first.
        self.owner._check_source(path, digest)
        code = self.source_to_code(source, self.path)
        self.owner._record_source(path, source, digest)
        return code

    def exec_module(self, module: ModuleType) -> None:
        if self.owner._reload_inputs is not None:
            self.owner._reload_inputs.remember_parent(module.__name__)
        self.owner._loaded[module.__name__] = module
        super().exec_module(module)


class _ProjectFinder(importlib.abc.MetaPathFinder):
    def __init__(self, owner: SceneSource) -> None:
        self.owner = owner

    def find_spec(self, fullname, path=None, target=None):
        if _protected(fullname):
            return None
        spec = importlib.machinery.PathFinder.find_spec(fullname, path)
        if spec is not None and spec.loader is None and spec.submodule_search_locations is not None:
            locations = tuple(Path(item).resolve() for item in spec.submodule_search_locations)
            if not self.owner._owns_namespace(fullname, locations):
                return None
            spec.loader = _FreshNamespace(fullname, spec.submodule_search_locations, self.owner)
            return spec
        if (spec is None or not isinstance(spec.loader, importlib.machinery.SourceFileLoader)
                or not isinstance(spec.origin, str)):
            return None
        origin = Path(spec.origin).resolve()
        if not self.owner._owns_source(origin):
            self.owner._check_package_scope(fullname, origin)
            return None
        spec.loader = _FreshSource(fullname, str(origin), self.owner)
        return spec


class SceneSource:
    """Own one source module and its local imports through scene execution.

    ``scenes`` contains only Scene subclasses declared by the selected file.
    Imports stay live until exit, including imports inside ``construct`` and
    ``tear_down``. A later invocation observes current source bytes rather than
    stale modules or pyc files. Previously loaded project modules are restored
    afterward; foreign modules and authored sys.path changes are not discarded.
    Returned classes are intended for use inside the context.

    ``source_inputs`` optionally freezes a declared path-to-SHA256 table from
    the native Studio scan. Declared shared helpers outside the scene directory
    then use the same fresh loader, and changed/undeclared project source is
    refused before execution. This does not change import search precedence or
    certify third-party packages, assets, or arbitrary Python host effects.
    """

    def __init__(
        self, source: str | Path, scene_type: type, *,
        source_inputs: Mapping[str | Path, str] | None = None,
    ) -> None:
        self.path = Path(source).resolve()
        if self.path.suffix.lower() not in (".py", ".pyw"):
            raise ValueError("the Python portal accepts only .py or .pyw scene sources")
        if not self.path.is_file():
            raise FileNotFoundError(f"scene source does not exist: {source}")
        if not isinstance(scene_type, type):
            raise TypeError("scene_type must be a Scene class")
        self.scene_type = scene_type
        parts = []
        root = self.path.parent
        while (root / "__init__.py").is_file():
            if not root.name.isidentifier():
                raise ImportError(f"scene package directory is not a Python identifier: {root.name!r}")
            parts.insert(0, root.name)
            if root.parent == root:
                raise ImportError("the filesystem root cannot be a scene package")
            root = root.parent
        self.root = root
        self._runtime_paths = RuntimePaths(root)
        if self._runtime_paths.contains(self.path):
            raise ImportError("scene source is inside a runtime installation; move it into the project")
        self._source_inputs = self._freeze_inputs(source_inputs)
        self._source_directories = frozenset(
            path.parent for path in (self._source_inputs or {})
            if path.suffix.lower() in (".py", ".pyw")
            and not self._runtime_paths.contains(path)
        )
        self.package = ".".join(parts)
        if self.package and self.path.name == "__init__.py":
            self.name = self.package
        elif self.package and self.path.stem.isidentifier() and self.path.suffix == ".py":
            self.name = self.package + "." + self.path.stem
        else:
            digest = hashlib.sha256(str(self.path).encode("utf-8", "surrogatepass")).hexdigest()[:24]
            self.name = (self.package + "." if self.package else "") + "__fmn_scene_" + digest
        self.module: ModuleType | None = None
        self.scenes: dict[str, type] = {}
        # Source-only observations, including lazy imports. This deliberately
        # does not purport to capture C2-C10 or arbitrary Python host effects.
        self.source_digests: dict[Path, str] = {}
        self._source_bytes: dict[Path, bytes] = {}
        self._source_capture_bytes = 0
        self._source_capture_error: str | None = None
        self._saved: dict[str, Any] = {}
        self._loaded: dict[str, ModuleType] = {}
        self._paths: list[str] = []
        self._finder = _ProjectFinder(self)
        self._entered = False
        self._active = False
        self._owner_thread = None
        self._reload_inputs = None

    @staticmethod
    def _freeze_inputs(values):
        if values is None:
            return None
        if not isinstance(values, Mapping) or not 1 <= len(values) <= 4096:
            raise ValueError("declared scene sources require a mapping of 1..4096 inputs")
        result = {}
        for name, digest in values.items():
            path = Path(name)
            if not path.is_absolute() or "\0" in str(path) or ".." in path.parts:
                raise ValueError("declared scene source paths must be absolute without NUL or parent traversal")
            if (not isinstance(digest, str) or len(digest) != 64
                    or any(char not in "0123456789abcdef" for char in digest)):
                raise ValueError("declared scene sources require lowercase SHA-256 digests")
            if path in result:
                raise ValueError("declared scene source paths must be unique")
            result[path] = digest
        return result

    def _owns_source(self, path: Path) -> bool:
        # Runtime exclusions precede even an explicit Studio watch declaration.
        return not self._runtime_paths.contains(path) and (
            path.is_relative_to(self.root) or (
                self._source_inputs is not None and path in self._source_inputs
            )
        )

    def _owns_namespace(self, name: str, paths: tuple[Path, ...]) -> bool:
        # Namespace containers have no source file. Refresh them when they
        # contain declared helper sources, or cached child attributes bypass
        # the loader even after the children's sys.modules entries are evicted.
        local = [not self._runtime_paths.contains(path) and (
            path.is_relative_to(self.root) or any(
                directory.is_relative_to(path) for directory in self._source_directories
            )
        ) for path in paths]
        if any(local) and not all(local):
            raise ImportError(f"scene namespace {name!r} has locations outside the scene project")
        return bool(local) and all(local)

    def _check_package_scope(self, name: str, origin: Path | None) -> None:
        if (origin is not None and not self._runtime_paths.contains(origin)
                and origin.name == "__init__.py"
                and not self._owns_source(origin)
                and any(directory.is_relative_to(origin.parent)
                        for directory in self._source_directories)):
            # An exact helper-file declaration does not authorize stale cached
            # package initializers. Declare the whole containing package.
            raise ImportError(
                f"declared helper package {name!r} has an undeclared initializer; "
                "include the containing package directory in watch_paths"
            )

    def _check_source(self, path: Path, digest: str) -> None:
        if self._source_inputs is None:
            return
        expected = self._source_inputs.get(path)
        if expected is None:
            raise RuntimeError(
                f"Studio imported undeclared project source {path}; "
                "include it in watch_paths"
            )
        if expected != digest:
            raise RuntimeError(f"Studio executed changed project source {path}; reload")

    def _record_source(self, path: Path, source: bytes, digest: str) -> None:
        """Retain the exact compilation input, never a later filesystem read.

        Ordinary Python may reload edited code. A one-version-per-path closure
        cannot represent that execution, so refuse its snapshot rather than
        changing ordinary render semantics or silently blessing the last edit.
        """
        self.source_digests[path] = digest
        if self._source_capture_error is not None:
            return
        previous = self._source_bytes.get(path)
        if previous is not None:
            if previous != source:
                self._source_capture_error = f"scene source changed during execution: {path}"
            return
        if (len(self._source_bytes) >= _MAX_CAPTURED_SOURCES
                or len(source) > _MAX_CAPTURED_SOURCE_BYTES - self._source_capture_bytes):
            self._source_capture_error = "executed scene source capture exceeds its count or byte budget"
            return
        self._source_bytes[path] = source
        self._source_capture_bytes += len(source)

    def _check_collisions(self) -> None:
        # A pre-existing foreign package must not redirect relative imports to
        # another checkout. Refuse before executing either project's code.
        if self.package:
            top = self.package.split(".", 1)[0]
            prior = sys.modules.get(top, _MISSING)
            if prior is not _MISSING:
                origin = _origin(prior)
                if origin is None or origin != self.root / top / "__init__.py":
                    raise ImportError(f"scene package {top!r} is already loaded from another location")
        # Cached absolute sibling imports bypass finders. Never silently reuse
        # a different project's helper just because it has the same filename.
        for name, module in tuple(sys.modules.items()):
            if "." in name or not name.isidentifier():
                continue
            for directory in (self.root, self.path.parent):
                namespace = _namespace_paths(module)
                if namespace and (directory / name).is_dir() and not self._owns_namespace(name, namespace):
                    raise ImportError(f"scene namespace {name!r} has locations outside the scene project")
                candidates = (directory / (name + ".py"), directory / name / "__init__.py")
                for candidate in candidates:
                    if (candidate != self.path and candidate.is_file()
                            and _origin(module) != candidate.resolve()):
                        raise ImportError(f"scene project module {name!r} is already loaded from another location")
    def _prepare(self) -> None:
        self._check_collisions()
        for name, module in tuple(sys.modules.items()):
            # Never evict the active engine even if a scene was placed in its
            # installation tree. That would fork native class identity.
            if _protected(name):
                continue
            origin = _origin(module)
            self._check_package_scope(name, origin)
            local_namespace = self._owns_namespace(name, _namespace_paths(module))
            if (origin is not None and self._owns_source(origin)) or local_namespace:
                # A namespace has no __file__, but retains child attributes.
                # Refresh its container as well as its source children, or
                # `from namespace import helper` can bypass sys.modules and
                # keep using a previous generation's helper indefinitely.
                self._saved[name] = module
        for name in self._saved:
            sys.modules.pop(name, None)
        # Keep the historical absolute sibling-import behavior as well as the
        # package root needed for normal qualified/relative imports.
        for path in dict.fromkeys((str(self.root), str(self.path.parent))):
            sys.path.insert(0, path)
            self._paths.append(path)
        index = next((i for i, finder in enumerate(sys.meta_path)
                      if finder is importlib.machinery.PathFinder), len(sys.meta_path))
        sys.meta_path.insert(index, self._finder)
        importlib.invalidate_caches()

    def _load(self) -> ModuleType:
        if self.package:
            parent = self.name.rpartition(".")[0]
            if parent:
                importlib.import_module(parent)
        existing = sys.modules.get(self.name)
        if existing is not None:
            if _origin(existing) != self.path:
                raise ImportError(f"scene module {self.name!r} resolves to another source")
            # A package initializer may already have imported this very file.
            return existing
        loader = _FreshSource(self.name, str(self.path), self)
        locations = [str(self.path.parent)] if self.path.name == "__init__.py" else None
        spec = importlib.util.spec_from_file_location(
            self.name, self.path, loader=loader, submodule_search_locations=locations,
        )
        if spec is None:
            raise ImportError(f"cannot create a scene module for {self.path}")
        module = importlib.util.module_from_spec(spec)
        sys.modules[self.name] = module
        self._loaded[self.name] = module
        # A standalone module can import itself by its normal filename. Do not
        # overwrite a stdlib/third-party module with a coincidentally equal name.
        if not self.package and self.path.stem.isidentifier():
            alias = self.path.stem
            if alias not in sys.modules:
                sys.modules[alias] = module
                self._loaded[alias] = module
        loader.exec_module(module)
        if self.package:
            parent, _, leaf = self.name.rpartition(".")
            if parent:
                setattr(sys.modules[parent], leaf, module)
        return module

    def __enter__(self) -> SceneSource:
        if self._entered:
            raise RuntimeError("a SceneSource can be entered only once")
        if not _SOURCE_OWNER.acquire(blocking=False):
            raise RuntimeError("another scene source owns this interpreter's import context")
        self._entered = self._active = True
        self._owner_thread = threading.get_ident()
        try:
            self._prepare()
            self.module = self._load()
            self.scenes = {
                name: value for name, value in vars(self.module).items()
                if isinstance(value, type) and value is not self.scene_type
                and issubclass(value, self.scene_type) and value.__module__ == self.name
            }
            return self
        except BaseException:
            self.__exit__(*sys.exc_info())
            raise

    def reload(self, *, if_changed: bool = False) -> ModuleType:
        """Refresh project definitions without replacing an existing live Scene.

        Imports are rolled back on failure, not arbitrary authored host effects.
        A changed reload invalidates a one-version-per-path source manifest.
        """
        from .source_reload import reload_source
        return reload_source(self, if_changed=if_changed)

    def __exit__(self, exc_type, exc_value, traceback) -> bool:
        if self._reload_inputs is not None:
            raise RuntimeError("cannot close a scene source during reload")
        if not self._active:
            return False
        try:
            for name, module in self._loaded.items():
                if sys.modules.get(name) is module:
                    sys.modules.pop(name, None)
            for name, module in self._saved.items():
                # Preserve an explicit authored replacement instead of silently
                # destroying it. Normal imports above have just been removed.
                if name not in sys.modules:
                    sys.modules[name] = module
            for index in range(len(sys.meta_path) - 1, -1, -1):
                if sys.meta_path[index] is self._finder:
                    del sys.meta_path[index]
            for entry in self._paths:
                for index, value in enumerate(sys.path):
                    if value is entry:
                        del sys.path[index]
                        break
        finally:
            self._active = False
            _SOURCE_OWNER.release()
        return False

    @property
    def sources(self) -> dict[str, bytes]:
        """Snapshot exact compilation bytes, including imports observed so far.

        Snapshots do not reread files and remain valid after context exit,
        edits, or deletion. Local-only projects keep their root-relative names.
        Declared external helpers expand the common root instead of collapsing
        to basenames and overwriting other inputs. These are source-only
        observations, not a claim to capture assets or arbitrary host effects.
        """
        if self._source_capture_error is not None:
            raise RuntimeError(self._source_capture_error)
        if not self._source_bytes:
            return {}
        try:
            root = Path(os.path.commonpath((self.root, *self._source_bytes)))
        except ValueError as error:
            raise RuntimeError("scene source capture cannot span filesystem volumes") from error
        return {
            path.relative_to(root).as_posix(): source
            for path, source in sorted(self._source_bytes.items())
        }
