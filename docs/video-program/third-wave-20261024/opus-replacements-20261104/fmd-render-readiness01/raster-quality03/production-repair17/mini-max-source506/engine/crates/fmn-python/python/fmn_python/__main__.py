"""Console composition for ``fmn-python`` and ``python -m fmn_python``."""

import json
import sys

from . import _ensure_exclusive_manimlib_namespace, _ManimlibNamespaceCollision


def _emit_namespace_collision(error):
    message = str(error)[:1024]
    if "--robot" in sys.argv[1:]:
        payload = {
            "schema": "fmn-python.cli",
            "version": 1,
            "kind": "namespace-collision",
            "status": "error",
            "exit": {"code": 4, "identity": "capability"},
            "message": message,
            "providers": list(error.providers),
        }
        print(json.dumps(payload, sort_keys=True, separators=(",", ":")))
    else:
        print(
            f"fmn-python: capability/namespace-collision: {message}",
            file=sys.stderr,
        )
    return 4


def _emit_parity_audit(native):
    from .parity_audit import ParityAuditError, audit_embedded_overlay, render_json

    extra = [arg for arg in sys.argv[1:] if arg not in {"--audit-parity", "--robot"}]
    if extra:
        print(
            "fmn-python: --audit-parity accepts only the optional --robot flag",
            file=sys.stderr,
        )
        return 2
    try:
        report = audit_embedded_overlay(native)
    except ParityAuditError as error:
        if "--robot" in sys.argv[1:]:
            payload = {
                "schema": "fmn.portal.runtime-audit",
                "version": 1,
                "ok": False,
                "error": str(error)[:1024],
            }
            print(json.dumps(payload, sort_keys=True, separators=(",", ":")))
        else:
            print(f"fmn-python: parity audit invalid: {error}", file=sys.stderr)
        return 2
    if "--robot" in sys.argv[1:]:
        sys.stdout.write(render_json(report))
    else:
        counts = report["counts"]
        verdict = "PASS" if report["ok"] else "FAIL"
        print(
            f"portal parity audit: {verdict}; "
            f"{counts['reviewed_implemented']} SAME/IMPROVED rows, "
            f"{counts['runtime_placeholders']} placeholders, "
            f"{counts['missing_reviewed']} missing"
        )
        for item in report["contradictions"]:
            print(
                f"- {item['symbol']}: {item['code']}: {item['detail']}",
                file=sys.stderr,
            )
    return 0 if report["ok"] else 1


def main():
    try:
        _ensure_exclusive_manimlib_namespace()
    except _ManimlibNamespaceCollision as error:
        return _emit_namespace_collision(error)

    from manimlib import _native
    from .console_rendering import _tokens, try_render_cli
    from .scene_controls import try_scene_cli
    from .studio import try_studio_cli
    from .project_editor import try_edit_cli
    from .bundle_cli import try_bundle_cli

    arguments = list(sys.argv[1:])
    result = try_edit_cli(_native, arguments)
    if result is not None:
        return result
    if "--audit-parity" in _tokens(arguments)[2]:
        return _emit_parity_audit(_native)
    result = try_studio_cli(_native, arguments)
    if result is not None:
        return result
    result = try_bundle_cli(_native, arguments)
    if result is not None:
        return result
    result = try_render_cli(_native, arguments)
    if result is None:
        result = try_scene_cli(_native, arguments)
    return _native._console_main() if result is None else result


if __name__ == "__main__":
    raise SystemExit(main())
