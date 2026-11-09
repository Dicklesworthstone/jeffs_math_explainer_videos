#!/usr/bin/env python3
"""Exercise the actual watched-project executable with private native sources.

Requires a built native_project host, its selected nightly Cargo, and the
checkout's already-fetched dependency closure. Uses only temporary projects.
"""
from __future__ import annotations

import argparse
from collections import deque
import hashlib
import ipaddress
import json
import os
from pathlib import Path
import queue
import subprocess
import sys
import tempfile
import threading
import time
import urllib.parse
import urllib.request
from typing import BinaryIO, Callable

MAX_PNG = 2 * 1024 * 1024


def check(condition: bool, message: str) -> None:
    if not condition:
        raise AssertionError(message)


def run(host: Path, cargo: Path, repo: Path) -> None:
    fixture = (repo / "crates/fmn-studio/tests/fixtures/project_worker.rs").read_text()
    check("const OFFSET: f64 = 0.0;" in fixture, "worker fixture changed its edit marker")
    with tempfile.TemporaryDirectory(prefix="fmn-watched-host-") as temporary:
        root = Path(temporary)
        (root / "src").mkdir()
        source = root / "src/main.rs"

        def write_source(offset: float) -> None:
            replacement = source.with_suffix(".tmp")
            replacement.write_text(fixture.replace(
                "const OFFSET: f64 = 0.0;", f"const OFFSET: f64 = {offset};"))
            replacement.replace(source)

        write_source(0.0)
        manifest = root / "Cargo.toml"
        text = "[package]\nname='project-worker'\nversion='0.1.0'\nedition='2024'\n[workspace]\n[dependencies]\n"
        for name in ("fmn-core", "fmn-mobject", "fmn-render", "fmn-scene", "fmn-studio"):
            text += f"{name}={{path={json.dumps(str(repo / 'crates' / name))}}}\n"
        manifest.write_text(text)
        lock = subprocess.run(
            [str(cargo), "-Z", "unstable-options", "-C", str(root),
             "generate-lockfile", "--offline", "--manifest-path", str(manifest)],
            stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=60, check=False,
        )
        check(lock.returncode == 0, "private lockfile bootstrap failed: " +
              lock.stderr[-16_384:].decode(errors="replace"))

        messages: queue.Queue[tuple[str, str]] = queue.Queue(maxsize=512)
        frames: queue.Queue[tuple[int, str, bytes] | Exception] = queue.Queue(maxsize=64)
        tail: deque[str] = deque(maxlen=128)
        process = subprocess.Popen(
            [str(host), str(cargo), str(manifest), "project-worker",
             "bin:project-worker", "ProjectScene", "--worker", str(source)],
            stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
        )

        def drain(stream: BinaryIO, label: str) -> None:
            for raw in iter(lambda: stream.readline(65_536), b""):
                line = raw.decode(errors="replace").rstrip()
                if label == "stderr":
                    tail.append(line)
                try:
                    messages.put_nowait((label, line))
                except queue.Full:
                    # Keep bounded diagnostics; don't block and deadlock child I/O.
                    pass

        assert process.stdout is not None and process.stderr is not None
        readers = [threading.Thread(target=drain, args=(process.stdout, "stdout"), daemon=True),
                   threading.Thread(target=drain, args=(process.stderr, "stderr"), daemon=True)]
        for reader in readers:
            reader.start()

        def message(predicate: Callable[[str, str], bool], timeout: float = 330) -> str:
            deadline = time.monotonic() + timeout
            while time.monotonic() < deadline:
                check(process.poll() is None, "project host exited early")
                try:
                    label, line = messages.get(timeout=min(1, max(0.01, deadline - time.monotonic())))
                except queue.Empty:
                    continue
                if predicate(label, line):
                    return line
            raise TimeoutError("project host did not reach the expected boundary")

        connection = None
        frame_reader = None
        try:
            launch = message(lambda label, line: label == "stdout" and line.startswith("http://"))
            parsed = urllib.parse.urlsplit(launch)
            check(parsed.scheme == "http" and parsed.hostname is not None and
                  ipaddress.ip_address(parsed.hostname).is_loopback, "host must publish a loopback URL")
            token = urllib.parse.parse_qs(parsed.query).get("cap", [])
            check(len(token) == 1, "host omitted its private capability")
            origin = f"http://{parsed.netloc}"
            # Disable proxy discovery for these loopback-only acceptance requests.
            opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))

            def request(route: str, body: str | None = None, timeout: float = 330):
                req = urllib.request.Request(origin + route,
                    data=None if body is None else body.encode(),
                    headers={"Origin": origin, "X-FMN-Capability": token[0],
                             "Content-Type": "application/x-www-form-urlencoded"})
                return opener.open(req, timeout=timeout)

            connection = request("/stream")

            def read_frames() -> None:
                try:
                    while True:
                        boundary = connection.readline(1024)
                        if not boundary:
                            return
                        if boundary == b"\r\n":
                            continue
                        check(boundary.startswith(b"--"), "invalid multipart boundary")
                        if boundary.rstrip().endswith(b"--"):
                            return
                        headers = {}
                        for _ in range(16):
                            line = connection.readline(4096)
                            if line == b"\r\n":
                                break
                            key, value = line.decode("ascii").split(":", 1)
                            headers[key.lower()] = value.strip()
                        else:
                            raise AssertionError("multipart headers exceeded bound")
                        size = int(headers["content-length"])
                        check(0 < size <= MAX_PNG, "published PNG exceeded test budget")
                        png = connection.read(size)
                        check(len(png) == size and png.startswith(b"\x89PNG\r\n\x1a\n"), "incomplete PNG")
                        digest = hashlib.sha256(png).hexdigest()
                        check(digest == headers["x-fmn-sha256"], "published PNG digest mismatch")
                        frames.put((int(headers["x-fmn-frame-index"]), digest, png), timeout=10)
                except Exception as error:
                    try:
                        frames.put_nowait(error)
                    except queue.Full:
                        pass

            frame_reader = threading.Thread(target=read_frames, daemon=True)
            frame_reader.start()

            def next_frame(index: int, different: str | None = None) -> tuple[int, str, bytes]:
                deadline = time.monotonic() + 330
                while time.monotonic() < deadline:
                    check(process.poll() is None, "project host exited while rebuilding")
                    try:
                        value = frames.get(timeout=min(1, max(0.01, deadline - time.monotonic())))
                    except queue.Empty:
                        continue
                    if isinstance(value, Exception):
                        raise value
                    if value[0] == index and (different is None or value[1] != different):
                        return value
                raise TimeoutError("source edit did not publish the committed frame")

            initial = next_frame(0)
            with request("/api/scrub", "frame=4&commit=true") as result:
                check(result.status == 200, "cannot commit the test frame")
            baseline = next_frame(4)
            check(baseline[2] == initial[2], "static fixture should not move during wait")

            # No Restart request here: only the production source watcher can cause this build.
            write_source(0.8)
            changed = next_frame(4, baseline[1])
            message(lambda label, line: label == "stderr" and "Native worker rebuilt;" in line)

            source.write_text("this is intentionally invalid Rust\n")
            message(lambda label, line: label == "stderr" and "source rebuild refused;" in line)
            with request("/api/scrub", "frame=4") as result:
                check(result.status == 200, "old worker must remain usable after a compiler error")
            retained = next_frame(4)
            check(retained[2] == changed[2], "failed compile replaced the healthy frame")

            # Repair through atomic rename: the same host/session must rebuild again.
            write_source(-0.8)
            repaired = next_frame(4, changed[1])
            check(repaired[2] != baseline[2], "corrected source was not compiled")
            with request("/api/inspect") as result:
                inspection = json.loads(result.read(2 * 1024 * 1024))
            check(inspection["view"]["frame_index"] == 4, "rebuild lost the committed timeline position")
            check(process.poll() is None, "watch host unexpectedly terminated")

            assert process.stdin is not None
            process.stdin.write(b"\n")
            process.stdin.flush()
            check(process.wait(timeout=30) == 0, "project host did not stop cleanly")
            images = root / "target/fmn-studio-workers"
            check(images.exists() and not any(images.iterdir()), "private worker images survived clean shutdown")
            check((root / "target/fmn-studio-build").exists(), "incremental Cargo cache was incorrectly deleted")
            print("OK: actual source watcher, automatic Cargo rebuild, compiler-error survival, atomic-save repair, committed PNGs and clean shutdown")
        except Exception:
            # Capability URLs are intentionally never echoed into failure logs.
            print("\n".join(tail), file=sys.stderr)
            raise
        finally:
            if process.poll() is None:
                try:
                    assert process.stdin is not None
                    process.stdin.write(b"\n")
                    process.stdin.flush()
                    process.wait(timeout=30)
                except (OSError, subprocess.TimeoutExpired):
                    process.kill()
                    process.wait(timeout=10)
            if connection is not None:
                connection.close()
            if frame_reader is not None:
                frame_reader.join(timeout=5)
            for reader in readers:
                reader.join(timeout=5)
            for pipe in (process.stdin, process.stdout, process.stderr):
                if pipe is not None:
                    pipe.close()


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--host", required=True, type=Path)
    parser.add_argument("--cargo", required=True, type=Path)
    parser.add_argument("--repo", type=Path, default=Path(__file__).resolve().parents[1])
    args = parser.parse_args()
    check(os.name == "posix", "real watched-project acceptance requires the Unix host capability")
    run(args.host.resolve(strict=True), args.cargo.resolve(strict=True), args.repo.resolve(strict=True))


if __name__ == "__main__":
    main()
