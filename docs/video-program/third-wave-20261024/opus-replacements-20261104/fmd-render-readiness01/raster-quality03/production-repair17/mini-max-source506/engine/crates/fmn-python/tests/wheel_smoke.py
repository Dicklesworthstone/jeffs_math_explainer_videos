"""Installed-artifact acceptance for a native ``franken-manim`` wheel.

Run this file with the Python interpreter from a disposable virtual
environment containing exactly the wheel under test and its pinned NumPy.
The optional collision probe writes one inert ``.dist-info`` fixture into that
environment and therefore deliberately runs last.
"""

import argparse
import importlib.metadata
import json
import pathlib
import subprocess
import sys
import tempfile
import zipfile

SUBPROCESS_TIMEOUT_SECONDS = 60

REFERENCE_CONSTRUCTOR_ALIASES = {
    "group": ("manimlib.mobject.mobject", "Group"),
    "v_group": ("manimlib.mobject.types.vectorized_mobject", "VGroup"),
    "vectorized_point": (
        "manimlib.mobject.types.vectorized_mobject",
        "VectorizedPoint",
    ),
    "small_dot": ("manimlib.mobject.geometry", "SmallDot"),
    "sector": ("manimlib.mobject.geometry", "Sector"),
    "vector": ("manimlib.mobject.geometry", "Vector"),
    "polyline": ("manimlib.mobject.geometry", "Polyline"),
    "triangle": ("manimlib.mobject.geometry", "Triangle"),
    "rounded_rectangle": ("manimlib.mobject.geometry", "RoundedRectangle"),
    "svg_mobject": ("manimlib.mobject.svg.svg_mobject", "SVGMobject"),
    "tangent_line": ("manimlib.mobject.geometry", "TangentLine"),
    "curved_arrow": ("manimlib.mobject.geometry", "CurvedArrow"),
    "curved_double_arrow": ("manimlib.mobject.geometry", "CurvedDoubleArrow"),
    "curves_as_submobjects": (
        "manimlib.mobject.types.vectorized_mobject",
        "CurvesAsSubmobjects",
    ),
    "dashed_vmobject": (
        "manimlib.mobject.types.vectorized_mobject",
        "DashedVMobject",
    ),
    "v_highlight": ("manimlib.mobject.types.vectorized_mobject", "VHighlight"),
}


def require(condition, message):
    if not condition:
        raise AssertionError(message)


def exported_schema_names(path):
    names = set()
    in_symbols = False
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if line == "[symbols]":
            in_symbols = True
            continue
        if in_symbols and line.startswith("["):
            break
        if not in_symbols or not line or line.startswith("#"):
            continue
        fields = line.split("\t")
        require(len(fields) == 6, f"malformed API schema row: {raw_line}")
        if fields[4] == "1":
            names.add(fields[1])
    return names


def console_path():
    suffix = ".exe" if sys.platform == "win32" else ""
    path = pathlib.Path(sys.executable).with_name(f"fmn-python{suffix}")
    require(path.is_file(), f"installed console entry point is missing: {path}")
    return path


def run_console(console, *arguments, expected=0):
    result = subprocess.run(
        [str(console), *map(str, arguments)],
        check=False,
        capture_output=True,
        text=True,
        timeout=SUBPROCESS_TIMEOUT_SECONDS,
    )
    require(
        result.returncode == expected,
        f"console exit {result.returncode}, expected {expected}: {result.stderr}",
    )
    return result


def verify_wheel_archive(path):
    require(path.is_file(), f"wheel does not exist: {path}")
    require(path.name.endswith(".whl"), f"not a wheel filename: {path.name}")
    filename_parts = path.name[:-4].split("-")
    require(len(filename_parts) >= 5, f"malformed wheel filename: {path.name}")
    python_tag, abi_tag, platform_tag = filename_parts[-3:]
    require(python_tag == "cp313", f"unexpected Python tag: {python_tag}")
    require(abi_tag == "cp313", f"unexpected ABI tag: {abi_tag}")
    require(platform_tag != "any", "native portal wheel cannot be platform-neutral")

    with zipfile.ZipFile(path) as archive:
        members = set(archive.namelist())
    native = [
        member
        for member in members
        if member.startswith("manimlib/manimlib.")
        and member.endswith((".so", ".pyd", ".dll"))
    ]
    require(len(native) == 1, f"expected one native extension, found {native}")
    require("manimlib/__init__.py" in members, "manimlib package wrapper is absent")
    require("fmn_python/__main__.py" in members, "console package is absent")
    require("LICENSE" in members, "root engine license is absent")
    for face in ("computer-modern", "ibm-plex-sans", "noto-sans-math"):
        require(
            f"dist/licenses/fonts/{face}-OFL.txt" in members,
            f"bundled {face} OFL text is absent",
        )
    require(
        any(".dist-info/sboms/" in member for member in members),
        "wheel CycloneDX SBOM is absent",
    )
    return python_tag, abi_tag, platform_tag, native[0]


def verify_reference_constructor_aliases(manimlib):
    """Exercise the Reference classes without exporting Rust-only aliases."""

    leaked = sorted(
        alias for alias in REFERENCE_CONSTRUCTOR_ALIASES if hasattr(manimlib, alias)
    )
    require(not leaked, f"Rust-only constructor helpers leaked into manimlib: {leaked}")

    with tempfile.TemporaryDirectory(prefix="fmn-alias-smoke-") as directory:
        svg_path = pathlib.Path(directory) / "minimal.svg"
        svg_path.write_text(
            '<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 2 2">'
            '<path d="M 0 0 L 2 0 L 2 2 Z"/>'
            "</svg>",
            encoding="utf-8",
        )

        builders = {
            "Group": lambda: manimlib.Group(manimlib.Mobject(), manimlib.Mobject()),
            "VGroup": lambda: manimlib.VGroup(
                manimlib.Circle(radius=0.2),
                manimlib.Square(side_length=0.2),
            ),
            "VectorizedPoint": lambda: manimlib.VectorizedPoint((0.1, 0.2, 0.0)),
            "SmallDot": lambda: manimlib.SmallDot((0.0, 0.0, 0.0)),
            "Sector": lambda: manimlib.Sector(),
            "Vector": lambda: manimlib.Vector((0.5, 0.25, 0.0)),
            "Polyline": lambda: manimlib.Polyline(
                (0.0, 0.0, 0.0),
                (0.5, 0.0, 0.0),
                (0.5, 0.5, 0.0),
            ),
            "Triangle": lambda: manimlib.Triangle(),
            "RoundedRectangle": lambda: manimlib.RoundedRectangle(
                width=0.5,
                height=0.25,
                corner_radius=0.05,
            ),
            "SVGMobject": lambda: manimlib.SVGMobject(str(svg_path)),
            "TangentLine": lambda: manimlib.TangentLine(
                manimlib.Circle(radius=0.5),
                0.25,
                length=0.5,
            ),
            "CurvedArrow": lambda: manimlib.CurvedArrow(
                (0.0, 0.0, 0.0),
                (0.75, 0.25, 0.0),
            ),
            "CurvedDoubleArrow": lambda: manimlib.CurvedDoubleArrow(
                (0.0, 0.0, 0.0),
                (0.75, 0.25, 0.0),
            ),
            "CurvesAsSubmobjects": lambda: manimlib.CurvesAsSubmobjects(
                manimlib.Circle(radius=0.4)
            ),
            "DashedVMobject": lambda: manimlib.DashedVMobject(
                manimlib.Circle(radius=0.4),
                num_dashes=4,
            ),
            "VHighlight": lambda: manimlib.VHighlight(
                manimlib.Circle(radius=0.4),
                n_layers=2,
            ),
        }

        require(
            set(builders)
            == {
                class_name
                for _module, class_name in REFERENCE_CONSTRUCTOR_ALIASES.values()
            },
            "constructor smoke table does not match the alias policy",
        )
        for alias, (module_name, class_name) in REFERENCE_CONSTRUCTOR_ALIASES.items():
            constructor = getattr(manimlib, class_name)
            require(
                module_name in sys.modules,
                f"schema module {module_name} was not assembled",
            )
            require(
                getattr(sys.modules[module_name], class_name, None) is constructor,
                f"{module_name}.{class_name} is not the root constructor identity",
            )
            instance = builders[class_name]()
            require(
                isinstance(instance, constructor),
                f"{class_name} returned the wrong concrete class",
            )
            require(
                isinstance(instance, manimlib.Mobject),
                f"{class_name} did not construct a Mobject",
            )
            family = instance.get_family()
            require(
                family and family[0] is instance,
                f"{class_name} has an invalid family root",
            )
            require(
                not hasattr(manimlib, alias),
                f"constructing {class_name} published Rust helper {alias!r}",
            )


def verify_installed_distribution(schema_path, scene_path):
    require(sys.implementation.name == "cpython", "portal requires CPython")
    require(sys.version_info[:2] == (3, 13), "portal requires CPython 3.13")

    import manimlib

    require(manimlib.__distribution__ == "franken-manim", "wrong provider")
    require(manimlib.__franken_manim__, "provider sentinel is absent")
    require(
        manimlib.__abi_policy__ == "cpython-3.13-full-abi",
        "wrong portal ABI policy",
    )
    require(not hasattr(manimlib, "__all__"), "Reference root has no __all__")

    observed = {name for name in vars(manimlib) if not name.startswith("_")}
    expected = exported_schema_names(schema_path)
    require(
        observed == expected,
        f"wildcard surface drift: missing={sorted(expected - observed)}, "
        f"extra={sorted(observed - expected)}",
    )
    require(len(observed) == 663, f"unexpected wildcard size: {len(observed)}")
    verify_reference_constructor_aliases(manimlib)

    distribution = importlib.metadata.distribution("franken-manim")
    requires = set(distribution.requires or ())
    require(requires == {"numpy==2.5.2"}, f"unexpected requirements: {requires}")

    console = console_path()
    version = run_console(console, "--robot", "--version")
    version_payload = json.loads(version.stdout)
    require(version_payload["kind"] == "version", version.stdout)
    require(version_payload["exit"] == {"code": 0, "identity": "success"}, version.stdout)

    scenes = run_console(console, "--robot", "--list-scenes", scene_path)
    scenes_payload = json.loads(scenes.stdout)
    require(scenes_payload["scenes"] == ["Hello"], scenes.stdout)

    constructed = run_console(
        console,
        "--robot",
        "--construct-only",
        scene_path,
        "Hello",
    )
    constructed_payload = json.loads(constructed.stdout)
    rendered = constructed_payload["rendered"]
    require(isinstance(rendered, bool) and not rendered, constructed.stdout)
    require(constructed_payload["scene_time"] == 3.0, constructed.stdout)

    output_root = pathlib.Path(tempfile.mkdtemp(prefix="fmn-wheel-render-"))
    destination = output_root / "frames"
    rendered = run_console(
        console,
        "--robot",
        scene_path,
        "Hello",
        "--format",
        "png_sequence",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        destination,
    )
    render_payload = json.loads(rendered.stdout)
    require(render_payload["kind"] == "render", rendered.stdout)
    require(render_payload["rendered"] is True, rendered.stdout)
    require(render_payload["frame_count"] == 90, rendered.stdout)
    require(render_payload["bytes"] > 0, rendered.stdout)
    require(len(render_payload["digest"]) == 64, rendered.stdout)
    frames = sorted(destination.glob("frame_*.png"))
    require(len(frames) == render_payload["frame_count"], rendered.stdout)
    require(
        all(frame.read_bytes().startswith(b"\x89PNG\r\n\x1a\n") for frame in frames),
        "render output contains a non-PNG frame",
    )

    still_destination = output_root / "final.png"
    still = run_console(
        console,
        "--robot",
        scene_path,
        "Hello",
        "--format",
        "png",
        "--resolution",
        "96x54",
        "--fps",
        "30",
        "--threads",
        "1",
        "--video_dir",
        still_destination,
    )
    still_payload = json.loads(still.stdout)
    require(still_payload["kind"] == "render", still.stdout)
    require(still_payload["format"] == "png", still.stdout)
    require(still_payload["rendered"] is True, still.stdout)
    require(still_payload["frame_count"] == 1, still.stdout)
    require(still_payload["bytes"] == still_destination.stat().st_size, still.stdout)
    require(len(still_payload["digest"]) == 64, still.stdout)
    require(
        still_destination.read_bytes().startswith(b"\x89PNG\r\n\x1a\n"),
        "final-state output is not a PNG",
    )

    refused = run_console(
        console,
        "--robot",
        "--reproducible",
        "missing-source.py",
        "MissingScene",
        expected=4,
    )
    refusal_payload = json.loads(refused.stdout)
    require(refusal_payload["kind"] == "render-capability-unavailable", refused.stdout)
    return console, pathlib.Path(manimlib.__file__).resolve().parent.parent


def verify_detectable_collision(console, site_packages):
    fixture = pathlib.Path(
        tempfile.mkdtemp(prefix="fmn_namespace_probe-", suffix=".dist-info", dir=site_packages)
    )
    provider = "fmn-namespace-collision-probe"
    (fixture / "METADATA").write_text(
        f"Metadata-Version: 2.4\nName: {provider}\nVersion: 0\n",
        encoding="utf-8",
    )
    # importlib.metadata exposes only RECORD paths which physically exist. The
    # fixture claims the already-installed wrapper without touching it, exactly
    # modelling the stale dual-ownership metadata left by an overlapping wheel.
    (fixture / "RECORD").write_text("manimlib/__init__.py,,\n", encoding="utf-8")

    refused = run_console(console, "--robot", "--version", expected=4)
    payload = json.loads(refused.stdout)
    require(payload["kind"] == "namespace-collision", refused.stdout)
    require(payload["providers"] == [provider], refused.stdout)

    direct = subprocess.run(
        [sys.executable, "-c", "import manimlib"],
        check=False,
        capture_output=True,
        text=True,
        timeout=SUBPROCESS_TIMEOUT_SECONDS,
    )
    require(direct.returncode != 0, "direct import accepted a namespace collision")
    require("separate virtual environment" in direct.stderr, direct.stderr)
    return fixture


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--wheel", type=pathlib.Path, required=True)
    parser.add_argument("--schema", type=pathlib.Path, required=True)
    parser.add_argument("--scene", type=pathlib.Path, required=True)
    parser.add_argument("--probe-collision", action="store_true")
    arguments = parser.parse_args()

    archive = verify_wheel_archive(arguments.wheel.resolve())
    console, site_packages = verify_installed_distribution(
        arguments.schema.resolve(), arguments.scene.resolve()
    )
    collision_fixture = None
    if arguments.probe_collision:
        collision_fixture = verify_detectable_collision(console, site_packages)

    print(
        json.dumps(
            {
                "abi_tag": archive[1],
                "collision_probe": str(collision_fixture) if collision_fixture else None,
                "distribution": "franken-manim",
                "kind": "wheel-smoke",
                "native_member": archive[3],
                "platform_tag": archive[2],
                "python_tag": archive[0],
                "status": "success",
                "wildcard_names": 663,
            },
            sort_keys=True,
            separators=(",", ":"),
        )
    )


if __name__ == "__main__":
    main()
