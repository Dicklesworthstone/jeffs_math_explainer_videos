"""Content identities for the optional portal's installed runtime.

Hash actual installed payloads, not version labels or RECORD's claimed hashes.
The wheel identity is explicitly an *installed payload* digest: an installation
cannot recover the original wheel archive's byte hash. This supplies the runtime
part of the input closure, not a sandbox or a complete host-effect certificate.
No native engine starts a subprocess, imports CPython, or gains a dependency.
"""
from __future__ import annotations

import csv
import hashlib
import io
from itertools import islice
import importlib.metadata as metadata
import importlib.machinery
import json
import os
from pathlib import Path, PurePosixPath
import stat
import sys
import sysconfig

_MAX_FILES = 32768
_MAX_RECORD_BYTES = 8 * 1024 * 1024
_MAX_FILE_BYTES = 512 * 1024 * 1024
_MAX_TOTAL_BYTES = 2 * 1024 * 1024 * 1024
_SCHEMA = "fmn-python.runtime-inputs.v1"


class RuntimeIdentityError(RuntimeError):
    """A required runtime input is unavailable, unbounded, or has changed."""


def _canonical(value):
    return json.dumps(value, sort_keys=True, separators=(",", ":"),
                      ensure_ascii=True, allow_nan=False).encode("ascii")


def _digest(value):
    return hashlib.sha256(_canonical(value)).hexdigest()


def _label(value, name):
    if not isinstance(value, str) or not value or len(value) > 4096:
        raise RuntimeIdentityError(name + " requires a bounded nonempty identity")
    if value.lower() == "unknown" or any(ord(char) < 32 or ord(char) == 127 for char in value):
        raise RuntimeIdentityError(name + " is not an established runtime identity")
    return value


def _stamp(info):
    return (info.st_dev, info.st_ino, info.st_size, info.st_mtime_ns, info.st_ctime_ns)


def _hash_file(path, remaining):
    """Bound reads on one regular-file descriptor and check both path and fd.

    O_NONBLOCK prevents a raced FIFO from hanging the inspector. Replacement,
    symlink retargeting, truncation and growth are refused, never cached by mtime.
    Metadata is a race check, not part of the portable content identity.
    """
    path = Path(path)
    descriptor = None
    try:
        before_path = path.stat()
        if not stat.S_ISREG(before_path.st_mode):
            raise RuntimeIdentityError("runtime input must be a regular file: " + str(path))
        descriptor = os.open(path, os.O_RDONLY | getattr(os, "O_NONBLOCK", 0))
        with os.fdopen(descriptor, "rb") as stream:
            descriptor = None
            before = os.fstat(stream.fileno())
            if not stat.S_ISREG(before.st_mode) or _stamp(before_path) != _stamp(before):
                raise RuntimeIdentityError("runtime input changed while opening: " + str(path))
            limit = min(_MAX_FILE_BYTES, remaining)
            if before.st_size > limit:
                raise RuntimeIdentityError("runtime input exceeds the byte budget: " + str(path))
            digest, size = hashlib.sha256(), 0
            while chunk := stream.read(min(1024 * 1024, limit - size + 1)):
                size += len(chunk)
                if size > limit:
                    raise RuntimeIdentityError("runtime input grew past the byte budget: " + str(path))
                digest.update(chunk)
            after = os.fstat(stream.fileno())
        if _stamp(before) != _stamp(after) or _stamp(after) != _stamp(path.stat()) or size != after.st_size:
            raise RuntimeIdentityError("runtime input changed while hashing: " + str(path))
        return size, digest.hexdigest()
    except OSError as error:
        raise RuntimeIdentityError("cannot read runtime input: " + str(path)) from error
    finally:
        if descriptor is not None:
            os.close(descriptor)


def _runtime_record(name):
    # Generated caches and installer bookkeeping are not imported runtime
    # payloads. They vary by installation prefix and cache warmness. Everything
    # else under site-packages, including .libs and package data, is hashed.
    parts = PurePosixPath(name).parts
    if "__pycache__" in parts or name.endswith((".pyc", ".pyo")):
        return False
    if parts and parts[0].endswith(".dist-info"):
        return len(parts) == 2 and parts[1] in {"METADATA", "WHEEL", "entry_points.txt"}
    if ".." in parts:
        # Wheel installation records may name generated console launchers.
        # Exclude only this known, non-imported surface; arbitrary traversal
        # entries must not turn the inventory into an ambient filesystem scan.
        leaf = parts[-1] if parts else ""
        if (set(parts[:-2]) <= {".."} and len(parts) >= 3
                and parts[-2] in {"bin", "Scripts"}
                and leaf in {"fmn-python", "fmn-python.exe", "f2py", "f2py.exe", "numpy-config", "numpy-config.exe"}):
            return False
        raise RuntimeIdentityError("runtime RECORD path escapes its installation: " + name)
    if (not name or name.startswith("/") or "\\" in name or ":" in name
            or any(part in {"", ".", ".."} for part in name.split("/"))
            or any(ord(char) < 32 or ord(char) == 127 for char in name)):
        raise RuntimeIdentityError("invalid runtime RECORD path: " + name)
    return True


def _distribution(name, required_paths):
    try:
        distribution = metadata.distribution(name)
        record = distribution.read_text("RECORD")
    except (metadata.PackageNotFoundError, OSError, ValueError) as error:
        raise RuntimeIdentityError("runtime identity requires an installed " + name + " distribution") from error
    if record is None:
        raise RuntimeIdentityError(name + " has no installed file inventory; use a built wheel")
    if len(record) > _MAX_RECORD_BYTES or len(record.encode("utf-8")) > _MAX_RECORD_BYTES:
        raise RuntimeIdentityError(name + " RECORD exceeds its byte budget")
    # Distribution.files filters out missing entries on some CPython builds.
    # Parse RECORD itself so missing payloads and traversal are refusals, not
    # quietly omitted members of a success-shaped runtime identity.
    try:
        files = list(islice(csv.reader(io.StringIO(record), strict=True), _MAX_FILES + 1))
    except csv.Error as error:
        raise RuntimeIdentityError(name + " has malformed installed RECORD metadata") from error
    if len(files) > _MAX_FILES:
        raise RuntimeIdentityError(name + " exceeds the runtime file budget")
    try:
        root = Path(distribution.locate_file("")).resolve(strict=True)
    except OSError as error:
        raise RuntimeIdentityError(name + " installation directory is unavailable") from error
    records, seen, covered = [], set(), set()
    for index, row in enumerate(files):
        if len(row) != 3:
            raise RuntimeIdentityError(name + " has malformed installed RECORD metadata")
        entry = row[0]
        if index >= _MAX_FILES:
            raise RuntimeIdentityError(name + " exceeds the runtime file budget")
        logical = str(entry)
        if not _runtime_record(logical):
            continue
        if logical in seen:
            raise RuntimeIdentityError("duplicate runtime RECORD path: " + logical)
        seen.add(logical)
        location = Path(distribution.locate_file(entry)).absolute()
        # Standard wheel payloads stay inside the installation directory even
        # when the interpreter itself was selected through a virtualenv symlink.
        try:
            actual = location.resolve(strict=True)
            actual.relative_to(root)
        except (OSError, ValueError) as error:
            raise RuntimeIdentityError("runtime payload is missing or escapes its installation: " + logical) from error
        records.append((logical, location))
        covered.add(actual)
    if not records:
        raise RuntimeIdentityError(name + " has an empty runtime inventory")
    for path in required_paths:
        try:
            actual = Path(path).resolve(strict=True)
        except (OSError, TypeError) as error:
            raise RuntimeIdentityError(name + " has no file-backed loaded runtime") from error
        if actual not in covered:
            raise RuntimeIdentityError(name + " loaded a module outside its installed payload: " + str(path))
    return _label(distribution.version, name + " version"), sorted(records)


def _module_paths(prefixes):
    # Pure schema-generated manimlib namespaces have no source file. Real
    # loaded Python/extension modules must be covered by the installed payload.
    result = []
    for name, module in tuple(sys.modules.items()):
        if any(name == prefix or name.startswith(prefix + ".") for prefix in prefixes):
            location = getattr(module, "__file__", None)
            if location:
                if str(location).endswith((".pyc", ".pyo")):
                    raise RuntimeIdentityError("sourceless runtime bytecode is not a certified input: " + name)
                result.append(location)
    return result


def _python_library():
    shared = sysconfig.get_config_var("Py_ENABLE_SHARED")
    framework = sysconfig.get_config_var("PYTHONFRAMEWORK")
    if shared == 0 and not framework:
        return []
    if shared not in (0, 1) and not framework:
        raise RuntimeIdentityError("the CPython shared-library configuration is unavailable")
    candidates = []
    filename = sysconfig.get_config_var("LDLIBRARY")
    if filename:
        for directory in (sysconfig.get_config_var("LIBDIR"), sysconfig.get_config_var("LIBPL")):
            if directory:
                candidates.append(Path(directory) / filename)
    if framework:
        prefix = sysconfig.get_config_var("PYTHONFRAMEWORKPREFIX")
        version = sysconfig.get_config_var("VERSION")
        if prefix and version:
            candidates.append(Path(prefix) / (framework + ".framework") / "Versions" / version / framework)
    for candidate in candidates:
        if candidate.is_file():
            return [("shared-library", candidate.absolute())]
    raise RuntimeIdentityError("the shared CPython runtime library cannot be identified")


def _layout(native):
    if sys.implementation.name != "cpython":
        raise RuntimeIdentityError("certified portal runtime requires CPython")
    native = getattr(native, "_native", native)
    extension = getattr(native, "__file__", None)
    if not extension or not any(str(extension).endswith(suffix) for suffix in importlib.machinery.EXTENSION_SUFFIXES):
        raise RuntimeIdentityError("portal runtime requires a file-backed native extension")
    numpy = getattr(native, "_np", None) or sys.modules.get("numpy")
    numpy_file = getattr(numpy, "__file__", None)
    if not numpy_file:
        raise RuntimeIdentityError("portal runtime requires an imported, file-backed NumPy")
    portal_version, portal_files = _distribution(
        "franken-manim", [extension, __file__, *_module_paths(("fmn_python", "manimlib"))],
    )
    numpy_version, numpy_files = _distribution("numpy", [numpy_file, *_module_paths(("numpy",))])
    if portal_version != getattr(native, "__version__", None) or numpy_version != getattr(numpy, "__version__", None):
        raise RuntimeIdentityError("loaded runtime versions disagree with installed metadata")
    abi = {
        "implementation": sys.implementation.name, "version": sys.version,
        "cache_tag": _label(sys.implementation.cache_tag, "CPython cache tag"),
        "soabi": _label(sysconfig.get_config_var("SOABI"), "CPython SOABI"),
        "abiflags": getattr(sys, "abiflags", ""),
        "portal_policy": _label(getattr(native, "__abi_policy__", None), "portal ABI policy"),
        "byteorder": sys.byteorder, "pointer_bits": 64 if sys.maxsize > 2**32 else 32,
    }
    python_files = [("executable", Path(os.path.abspath(sys.executable))), *_python_library()]
    return {"cpython": ("CPython " + sys.version.split()[0], python_files),
            "wheel": ("franken-manim " + portal_version, portal_files),
            "numpy": ("NumPy " + numpy_version, numpy_files)}, abi


def _capture(layout):
    groups, abi = layout()
    remaining, count, identities = _MAX_TOTAL_BYTES, 0, {}
    for role in ("cpython", "wheel", "numpy"):
        label, files = groups[role]
        rows, seen = [], set()
        for name, location in sorted(files):
            count += 1
            if count > _MAX_FILES:
                raise RuntimeIdentityError("runtime payload exceeds the total file budget")
            if name in seen:
                raise RuntimeIdentityError("duplicate runtime input: " + name)
            seen.add(name)
            size, digest = _hash_file(location, remaining)
            remaining -= size
            rows.append((name, size, digest))
        if not rows:
            raise RuntimeIdentityError("missing runtime payload: " + role)
        identities[role] = _label(label, role) + ";installed-payload-sha256=" + _digest((_SCHEMA, role, rows))
    identities["abi"] = "cpython-abi-sha256=" + _digest((_SCHEMA, abi))
    return identities


class RuntimeSnapshot:
    """Frozen identities, with fresh byte verification at publication barriers."""
    def __init__(self, layout):
        self._layout = layout
        self._identities = _capture(layout)

    @property
    def identities(self):
        return dict(self._identities)

    def verify(self):
        observed = _capture(self._layout)
        if observed != self._identities:
            changed = ", ".join(role for role in sorted(observed) if observed[role] != self._identities[role])
            raise RuntimeIdentityError("runtime inputs changed during rendering: " + changed)


def capture_runtime(native):
    """Capture the host's installed runtime, refusing unknown/mismatched inputs.

    Native callers without a file-backed installed portal are not silently
    certified with synthetic labels. Ordinary non-certified rendering is not
    subject to this installation requirement or the hashing cost.
    """
    return RuntimeSnapshot(lambda: _layout(native))
