"""Admission and no-retry semantics for the programmatic live clock adapter."""
import io
import json
import unittest
import urllib.error
import urllib.parse
from unittest.mock import patch

from fmn_python.studio_clock import advance


class Response(io.BytesIO):
    def __init__(self, document, generation="3"):
        super().__init__(json.dumps(document).encode())
        self.headers = {"X-FMN-Worker-Generation": generation}


class ClockProtocolTests(unittest.TestCase):
    def setUp(self):
        self.view = dict(live_advance=True, input_events=True, frame_index=5,
                         frame_count=6, fps=8, input_revision=12)
        self.calls = []
        self.error = None

    def open(self, request, timeout):
        self.calls.append((request, timeout))
        if request.get_method() == "GET":
            return Response(dict(view=self.view))
        if self.error:
            raise self.error
        return Response(dict(frame_index=5, sha256="a" * 64))

    def invoke(self, frames=1, url="http://127.0.0.1:12345/?cap=fixture-cap"):
        with patch("urllib.request.build_opener") as build:
            build.return_value.open.side_effect = self.open
            result = advance(url, 17, frames)
            self.assertEqual(build.call_count, 1)
            proxy, redirects = build.call_args.args
            self.assertEqual(proxy.proxies, {}, "ambient proxy settings must not receive the capability")
            self.assertIsNone(redirects.redirect_request(None))
            return result

    def test_clock_uses_exact_generation_frame_revision_and_nominal_count(self):
        self.assertEqual(self.invoke(3)["sha256"], "a" * 64)
        self.assertEqual(len(self.calls), 2)
        inspection, command = [item[0] for item in self.calls]
        self.assertEqual(inspection.full_url, "http://127.0.0.1:12345/api/inspect?cap=fixture-cap")
        self.assertEqual(command.full_url, "http://127.0.0.1:12345/api/advance?cap=fixture-cap")
        self.assertEqual(command.get_header("Origin"), "http://127.0.0.1:12345")
        self.assertEqual(urllib.parse.parse_qs(command.data.decode()),
                         dict(worker_generation=["3"], frame=["5"], revision=["12"], frames=["3"]))
        self.assertEqual([item[1] for item in self.calls], [17, 17])

    def test_invalid_step_counts_and_nonlocal_hosts_do_not_open_a_connection(self):
        for frames in (0, -1, 241, True, 1.0, "1"):
            with self.subTest(frames=frames), self.assertRaises(ValueError):
                self.invoke(frames)
        for url in ("https://127.0.0.1:10", "http://example.org", "http://user@127.0.0.1"):
            with self.subTest(url=url), self.assertRaises(RuntimeError):
                self.invoke(url=url)
        self.assertEqual(self.calls, [])

    def test_historical_readonly_or_oversized_requests_cannot_send_a_command(self):
        for changed in ({"live_advance": False}, {"input_events": False}, {"frame_index": 0},
                        {"input_revision": True}, {"input_revision": -1}, {"fps": 0}):
            with self.subTest(changed=changed):
                saved = self.view.copy()
                self.view.update(changed)
                with self.assertRaises((RuntimeError, ValueError)):
                    self.invoke()
                self.view = saved
        with self.assertRaises(ValueError):
            self.invoke(9)
        self.assertTrue(all(req.get_method() == "GET" for req, _ in self.calls))

    def test_timeout_or_conflict_after_admission_is_never_retried(self):
        for error in (TimeoutError("late"), urllib.error.HTTPError("", 409, "conflict", {}, io.BytesIO(b"stale revision"))):
            self.calls.clear()
            self.error = error
            with self.subTest(error=error), self.assertRaisesRegex(RuntimeError, "not retried"):
                self.invoke()
            self.assertEqual(len(self.calls), 2, "one inspection and at most one effectful request")


if __name__ == "__main__":
    unittest.main()
