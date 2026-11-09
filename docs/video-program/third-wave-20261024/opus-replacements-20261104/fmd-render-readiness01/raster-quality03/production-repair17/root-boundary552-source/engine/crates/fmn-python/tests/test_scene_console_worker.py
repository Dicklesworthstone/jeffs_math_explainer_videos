"""Worker launch orchestration retains its protocol while authored code runs."""
import json
from pathlib import Path
import sys
from types import SimpleNamespace
import unittest
from unittest.mock import patch

from fmn_python import studio
from fmn_python.scene_console import _STUDIO_WORKER
from test_embedded_shell_protocol import native_classes, Shell, CapabilityError


class SceneConsoleWorkerTests(unittest.TestCase):
    def request(self):
        native = SimpleNamespace(__file__=studio.__file__)
        runtime = {
            "python": studio._file_digest(Path(sys.executable)),
            "native": studio._file_digest(Path(native.__file__).resolve()),
            "portal": studio._file_digest(Path(studio.__file__).resolve()),
            "inputs": studio._file_digest(Path(sys.modules[studio.StudioInputs.__module__].__file__).resolve()),
            "loader": studio._file_digest(Path(studio.__file__).with_name("scene_loading.py")),
            "abi": sys.implementation.cache_tag,
        }
        request = {"schema": studio._SCHEMA, "version": 2, "runtime": runtime}
        request["build_id"] = studio._build_id(request)
        return native, json.dumps(request)
    def test_import_construct_and_live_callbacks_cannot_consume_worker_stdio(self):
        native, request = self.request()
        facts = []
        def capture(request, received):
            self.assertIs(received, native)
            self.assertTrue(_STUDIO_WORKER.get())
            self.assertIs(sys.stdout, sys.stderr)
            classes = native_classes()
            embedded = classes.InteractiveSceneEmbed(classes.Scene())
            embedded.shell = Shell(lambda shell: self.fail("worker entered terminal"))
            with self.assertRaisesRegex(CapabilityError, "native protocol"):
                embedded.launch()
            facts.append("captured")
            return SimpleNamespace(serve=serve)
        def serve():
            self.assertFalse(_STUDIO_WORKER.get())
            self.assertIsNot(sys.stdout, sys.stderr)
            facts.append("read_only_serve")
        with patch.dict(sys.modules, {"manimlib": SimpleNamespace(_native=native)}), \
                patch.object(studio, "_capture", capture):
            self.assertEqual(studio._worker(request), 0)
        self.assertEqual(facts, ["captured", "read_only_serve"])
        self.assertFalse(_STUDIO_WORKER.get())
    def test_guard_and_stdout_are_restored_when_authored_worker_code_fails(self):
        native, request = self.request()
        primary = KeyboardInterrupt("authored capture stopped")
        stdout = sys.stdout
        def capture(*_):
            self.assertTrue(_STUDIO_WORKER.get())
            raise primary
        with patch.dict(sys.modules, {"manimlib": SimpleNamespace(_native=native)}), \
                patch.object(studio, "_capture", capture):
            with self.assertRaises(KeyboardInterrupt) as caught:
                studio._worker(request)
        self.assertIs(caught.exception, primary)
        self.assertFalse(_STUDIO_WORKER.get())
        self.assertIs(sys.stdout, stdout)
    def test_generation_mismatch_executes_no_authored_code(self):
        native, encoded = self.request()
        request = json.loads(encoded)
        request["runtime"]["portal"] = "0" * 64
        request["build_id"] = studio._build_id(request)
        with patch.dict(sys.modules, {"manimlib": SimpleNamespace(_native=native)}), \
                patch.object(studio, "_capture", side_effect=AssertionError("authored code executed")):
            with self.assertRaisesRegex(RuntimeError, "differs"):
                studio._worker(json.dumps(request))
        self.assertFalse(_STUDIO_WORKER.get())


if __name__ == "__main__":
    unittest.main()
