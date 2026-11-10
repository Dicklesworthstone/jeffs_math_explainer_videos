"""Clean-wheel acceptance: actual CPython worker, native IPC/HTTP/UI and PNGs.

Run with the installed wheel's interpreter and -I; no PYTHONPATH, ffmpeg,
source-tree imports, mocked renderers or alternate HTTP server are required.
"""
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import signal
import struct
import subprocess
import sys
import tempfile
import time
import unittest
import urllib.error
import urllib.parse
import urllib.request
import zlib

from fmn_python.studio import Studio


def rgba_png(data):
    """Independently decode the native RGBA8 PNG wire with standard zlib."""
    assert data[:8] == b"\x89PNG\r\n\x1a\n"
    offset, compressed = 8, bytearray()
    while offset < len(data):
        size = struct.unpack_from(">I", data, offset)[0]
        kind = data[offset + 4:offset + 8]
        payload = data[offset + 8:offset + 8 + size]
        crc = struct.unpack_from(">I", data, offset + 8 + size)[0]
        assert zlib.crc32(kind + payload) & 0xffffffff == crc
        if kind == b"IHDR":
            width, height, depth, color, _, _, interlace = struct.unpack(">IIBBBBB", payload)
            assert (depth, color, interlace) == (8, 6, 0)
        elif kind == b"IDAT":
            compressed.extend(payload)
        offset += size + 12
    packed = zlib.decompress(compressed)
    stride, rows, previous = width * 4, [], bytearray(width * 4)
    assert len(packed) == (stride + 1) * height
    for y in range(height):
        filter = packed[y * (stride + 1)]
        row = bytearray(packed[y * (stride + 1) + 1:(y + 1) * (stride + 1)])
        for x in range(stride):
            a, b, c = row[x - 4] if x >= 4 else 0, previous[x], previous[x - 4] if x >= 4 else 0
            if filter == 0: predictor = 0
            elif filter == 1: predictor = a
            elif filter == 2: predictor = b
            elif filter == 3: predictor = (a + b) // 2
            elif filter == 4:
                p = a + b - c
                distances = (abs(p - a), abs(p - b), abs(p - c))
                predictor = (a, b, c)[distances.index(min(distances))]
            else: raise AssertionError("unknown PNG filter")
            row[x] = (row[x] + predictor) & 255
        rows.append(row)
        previous = row
    return width, height, b"".join(rows)


class Preview:
    def __init__(self, host):
        url = urllib.parse.urlsplit(host.url)
        self.base = f"{url.scheme}://{url.netloc}"
        self.cap = urllib.parse.parse_qs(url.query)["cap"][0]
        self.opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
        self.expected_digest = None

    def request(self, path, fields=None, *, cap=True, origin=None):
        url = self.base + path + ("?cap=" + self.cap if cap else "")
        headers = {}
        data = None
        if fields is not None:
            data = urllib.parse.urlencode(fields).encode()
            headers["Content-Type"] = "application/x-www-form-urlencoded"
            headers["Origin"] = self.base if origin is None else origin
        return self.opener.open(urllib.request.Request(url, data=data, headers=headers), timeout=20)

    def json(self, path, fields=None):
        with self.request(path, fields) as response:
            result = json.load(response)
        if path in {"/api/scrub", "/api/restart"}:
            self.expected_digest = result["sha256"]
        return result

    def frame(self):
        with self.request("/stream") as response:
            assert response.headers["Content-Type"].startswith("multipart/x-mixed-replace")
            # A new client receives retained history, oldest first. Drain it
            # just as the persistent browser stream does, selecting by the
            # operation's content identity, NOT a reused frame-zero index.
            for _ in range(8):
                boundary = response.readline(1024).strip()
                if not boundary:
                    boundary = response.readline(1024).strip()
                assert boundary == b"--fmn-frame", boundary
                headers = {}
                for _ in range(16):
                    line = response.readline(4096).strip()
                    if not line:
                        break
                    name, value = line.decode().split(":", 1)
                    headers[name.lower()] = value.strip()
                else:
                    raise AssertionError("multipart header budget exceeded")
                size = int(headers["content-length"])
                assert 0 < size < 1024 * 1024
                data = response.read(size)
                assert len(data) == size
                digest = hashlib.sha256(data).hexdigest()
                if self.expected_digest is None or digest == self.expected_digest:
                    return data
            raise AssertionError("operation's PNG was absent from retained stream history")


class StudioWheelTests(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix="fmn studio scene ")
        self.root = Path(self.temp.name)
        self.source = self.root / "scene.py"
        self.helper = self.root / "helper.py"
        self.marker = self.root / "runs.txt"
        self.helper.write_text("OFFSET = 1\n")
        self.original = (
            "from manimlib import *\nimport os\nfrom pathlib import Path\nimport helper\n"
            "class Moving(Scene):\n"
            "    def __init__(self):\n        super().__init__()\n"
            "    def construct(self):\n"
            "        print('authored output must not corrupt the native protocol')\n"
            "        os.environ['FMN_STUDIO_TEST_CHILD'] = 'isolated'\n"
            f"        with open({str(self.marker)!r}, 'a') as f: f.write(str(os.getpid()) + '\\n')\n"
            "        box = Square(fill_color=RED, fill_opacity=1, stroke_width=0).shift(LEFT)\n"
            "        self.add(box)\n"
            "        self.play(box.animate.shift(helper.OFFSET * RIGHT), run_time=0.5)\n"
            "        self.wait(0.25)\n"
        )
        self.source.write_text(self.original)

    def tearDown(self):
        self.temp.cleanup()

    def host(self, **options):
        return Studio(self.source, "Moving", resolution=(96, 54), fps=8, threads=1,
                      max_bytes=2 * 1024 * 1024, **options)

    def test_captured_frames_native_inspection_isolation_auth_and_fresh_reload(self):
        old_environment = os.environ.get("FMN_STUDIO_TEST_CHILD")
        with self.host() as host:
            api = Preview(host)
            with self.assertRaises(urllib.error.HTTPError) as failure:
                api.request("/api/inspect", cap=False)
            self.assertEqual(failure.exception.code, 403)
            with self.assertRaises(urllib.error.HTTPError) as failure:
                api.request("/api/scrub", {"frame": 0}, origin="https://untrusted.invalid")
            self.assertEqual(failure.exception.code, 403)
            with api.request("/") as page:
                self.assertIn(b"studio.js", page.read())
            with api.request("/studio.js") as script:
                self.assertIn(b"/api/scrub", script.read())
            first = api.json("/api/inspect")
            self.assertEqual(first["view"]["frame_count"], 6)
            self.assertFalse(first["view"]["input_events"])
            self.assertTrue(first["nodes"])
            first_png = api.frame()
            last_receipt = api.json("/api/scrub", {"frame": 5})
            last_png = api.frame()
            last = api.json("/api/inspect")
            self.assertEqual(last_receipt["sha256"], hashlib.sha256(last_png).hexdigest())
            self.assertEqual(last["view"]["frame_index"], 5)
            self.assertLess(first["scene_time"], last["scene_time"])
            self.assertNotEqual(first["nodes"], last["nodes"])
            centers = []
            for image in (first_png, last_png):
                width, height, rgba = rgba_png(image)
                self.assertEqual((width, height), (96, 54))
                xs = [p % width for p in range(width * height)
                      if rgba[4*p] > 100 and rgba[4*p] > rgba[4*p+1] * 1.5]
                self.assertGreater(len(xs), 30)
                centers.append(sum(xs) / len(xs))
            self.assertGreater(centers[1], centers[0] + 2)
            api.json("/api/scrub", {"frame": 0})
            self.assertEqual(api.frame(), first_png)
            self.assertEqual(api.json("/api/inspect"), first)
            runs = self.marker.read_text().splitlines()
            self.assertEqual(len(runs), 1, "scrub must not reexecute source")
            self.assertNotEqual(int(runs[0]), os.getpid())
            self.assertEqual(os.environ.get("FMN_STUDIO_TEST_CHILD"), old_environment)
            self.assertNotIn("helper", sys.modules, "the parent must not import authored code")

            # Same length and timestamp: only a fresh source loader catches it.
            stamp = self.helper.stat()
            self.helper.write_text("OFFSET = 3\n")
            os.utime(self.helper, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
            api.json("/api/scrub", {"frame": 5, "commit": "true"})
            reload = api.json("/api/restart", {})
            self.assertEqual(reload["frame_index"], 0)
            self.assertEqual(reload["reused_entries"], 0)
            self.assertEqual(reload["replayed_entries"], 0)
            self.assertEqual(reload["reexecuted_entries"], 0)
            api.json("/api/scrub", {"frame": 5})
            changed = api.frame()
            self.assertNotEqual(changed, last_png)
            runs = self.marker.read_text().splitlines()
            self.assertEqual(len(runs), 2)
            self.assertNotEqual(runs[0], runs[1])

            # Syntax failures are caught before replacing the usable worker.
            self.source.write_text("class Broken(\n")
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/restart", {})
            self.assertEqual(api.json("/api/inspect")["view"]["frame_index"], 5)
            self.assertEqual(self.marker.read_text().splitlines(), runs)

            # A valid but failed execution is contained; the stable host and
            # previous PNG remain, and fixing source permits another reload.
            self.source.write_text(self.original.replace("        print(", "        raise RuntimeError('authored failure')\n        print("))
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/restart", {})
            self.assertTrue(host.alive)
            self.assertEqual(api.frame(), changed)
            # Source now has only one static capture. Old committed frame five
            # cannot prevent a shorter scene from being loaded and inspected.
            self.source.write_text("from manimlib import *\nclass Moving(Scene):\n    def construct(self):\n        self.add(Circle())\n")
            reload = api.json("/api/restart", {})
            self.assertEqual(reload["frame_index"], 0)
            self.assertEqual(api.json("/api/inspect")["view"]["frame_count"], 1)
            with self.assertRaises(urllib.error.HTTPError):
                api.json("/api/scrub", {"frame": 1})
        self.assertFalse(host.alive)
        host.close()  # idempotent, and its disposable process has been reaped

    def test_capture_budget_and_worker_timeout_refuse_instead_of_hanging(self):
        with self.assertRaises(RuntimeError):
            self.host(max_frames=1)
        # Catching an authored capture error must not publish a partial timeline.
        self.source.write_text("from manimlib import *\nclass Moving(Scene):\n    def construct(self):\n        try: self.wait(1)\n        except RuntimeError: pass\n")
        with self.assertRaises(RuntimeError):
            self.host(max_frames=1)
        self.source.write_text("from manimlib import *\nclass Moving(Scene):\n    def construct(self):\n        while True: pass\n")
        started = time.monotonic()
        with self.assertRaises(RuntimeError):
            self.host(timeout=1)
        self.assertLess(time.monotonic() - started, 12)

    def test_autoreload_uses_native_content_watch_and_recovers_failed_edits(self):
        asset = self.root / "values.csv"
        asset.write_text("1")
        self.source.write_text(self.original.replace("helper.OFFSET * RIGHT",
            f"helper.OFFSET * float(Path({str(asset)!r}).read_text()) * RIGHT"))

        def wait_for(host, predicate):
            deadline = time.monotonic() + 20
            while time.monotonic() < deadline:
                status = host.reload_status
                if predicate(status):
                    return status
                self.assertTrue(host.alive, status)
                time.sleep(0.1)
            self.fail("autoreload did not settle: " + repr(host.reload_status))

        with self.host(autoreload=True, watch_paths=[asset], debounce_ms=50) as host:
            api = Preview(host)
            api.json("/api/scrub", {"frame": 5})
            initial = api.frame()
            # A same-size edit with unchanged mtime must still restart.
            stamp = self.helper.stat()
            self.helper.write_text("OFFSET = 3\n")
            os.utime(self.helper, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
            first = wait_for(host, lambda status: status["completed"] == 1)
            self.assertIsNone(first["error"])
            self.assertEqual(first["result"]["frame_index"], 0)
            api.json("/api/scrub", {"frame": 5})
            changed = api.frame()
            self.assertNotEqual(initial, changed)
            # Generated files, empty output directories and identical saves
            # cannot continually re-execute the scene's authored side effects.
            (self.root / "media").mkdir()
            (self.root / "media/preview.png").write_bytes(changed)
            self.helper.write_text("OFFSET = 3\n")
            time.sleep(0.7)
            self.assertEqual(host.reload_status["completed"], 1)
            self.assertEqual(len(self.marker.read_text().splitlines()), 2)

            asset.write_text("2")
            wait_for(host, lambda status: status["completed"] == 2)
            api.json("/api/scrub", {"frame": 5})
            asset_changed = api.frame()
            self.assertNotEqual(changed, asset_changed)

            # A bad helper is discovered inside the candidate worker, not by
            # executing Python in the stable host. The old worker stays usable.
            self.helper.write_text("OFFSET = (\n")
            failed = wait_for(host, lambda status: status["error"] is not None)
            self.assertEqual(failed["completed"], 2)
            self.assertEqual(api.json("/api/inspect")["view"]["frame_index"], 5)
            self.assertEqual(api.frame(), asset_changed)
            time.sleep(0.5)
            self.assertEqual(host.reload_status, failed, "failed source must not execute on every poll")
            self.assertEqual(len(self.marker.read_text().splitlines()), 3)

            self.helper.write_text("OFFSET = 1\n")
            self.source.write_text("from manimlib import *\nclass Moving(Scene):\n    def construct(self):\n        self.add(Circle())\n")
            recovered = wait_for(host, lambda status: status["completed"] == 3)
            self.assertIsNone(recovered["error"])
            self.assertEqual(api.json("/api/inspect")["view"]["frame_count"], 1)
            self.assertEqual(recovered["result"]["replayed_entries"], 0)
            watch_thread = host._watch_thread
        self.assertFalse(watch_thread.is_alive())
        with self.assertRaisesRegex(RuntimeError, "closed"):
            host.reload()

    def test_cli_selects_native_studio_and_stops_on_interrupt(self):
        command = [sys.executable, "-I", "-m", "fmn_python", "--robot", "studio", str(self.source), "Moving",
                   "--resolution", "96x54", "--fps", "8", "--timeout", "15"]
        with tempfile.TemporaryFile() as stderr:
            process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=stderr, text=True)
            try:
                # Deadline on stdout discovery rather than an unbounded readline.
                import selectors
                with selectors.DefaultSelector() as selector:
                    selector.register(process.stdout, selectors.EVENT_READ)
                    self.assertTrue(selector.select(20), "CLI did not announce Studio")
                line = process.stdout.readline()
                if not line:
                    stderr.seek(0)
                    self.fail("CLI exited without a receipt: " + stderr.read().decode(errors="replace"))
                report = json.loads(line)
                self.assertEqual(report["exit"]["code"], 0, report)
                self.assertEqual(report["preview_mode"], "captured-read-only")
                self.assertFalse(report["certified"])
                self.assertIn("?cap=", report["url"])
                process.send_signal(signal.SIGINT)
                process.wait(timeout=15)
                self.assertEqual(process.returncode, 130)
            finally:
                if process.poll() is None:
                    process.kill()
                    process.wait(timeout=15)
                process.stdout.close()
        help = subprocess.run([sys.executable, "-I", "-m", "fmn_python", "--robot", "studio", "--help"],
                              capture_output=True, text=True, timeout=15, check=True)
        self.assertIn("read-only", json.loads(help.stdout)["help"].lower())


if __name__ == "__main__":
    unittest.main(verbosity=2)
