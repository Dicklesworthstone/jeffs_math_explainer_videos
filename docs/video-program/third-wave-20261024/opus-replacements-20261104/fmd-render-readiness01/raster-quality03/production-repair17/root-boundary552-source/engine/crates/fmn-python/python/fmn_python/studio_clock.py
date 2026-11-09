"""Explicit live stepping through the existing authenticated Studio route.

No timers, renderer, updater executor or retry loop belongs in this adapter.
Inspection is an optimistic admission read; the native generation/revision
checks decide whether the subsequent command may execute authored effects.
"""
from __future__ import annotations

import json
import urllib.error
import urllib.parse
import urllib.request
from typing import Any


def advance(url: str, timeout: int, frames: int) -> dict[str, Any]:
    if isinstance(frames, bool) or not isinstance(frames, int) or not 1 <= frames <= 240:
        raise ValueError("frames must be an integer in 1..240")
    parsed = urllib.parse.urlsplit(url)
    if parsed.scheme != "http" or parsed.hostname != "127.0.0.1" or parsed.username or parsed.password:
        raise RuntimeError("Studio advance requires its native loopback host")
    authority = "http://" + parsed.netloc

    class NoRedirect(urllib.request.HTTPRedirectHandler):
        def redirect_request(self, *_args, **_kwargs):
            return None

    opener = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect())

    def request(route, form=None):
        data = None if form is None else urllib.parse.urlencode(form).encode("ascii")
        req = urllib.request.Request(authority + route + "?" + parsed.query, data=data,
            headers={"Origin": authority, "Content-Type": "application/x-www-form-urlencoded"})
        limit = 8 * 1024 * 1024 if form is None else 65536
        with opener.open(req, timeout=timeout) as response:
            payload = response.read(limit + 1)
            generation = response.headers.get("X-FMN-Worker-Generation")
        if len(payload) > limit:
            raise RuntimeError("Studio advance response exceeds its byte budget")
        result = json.loads(payload)
        if not isinstance(result, dict):
            raise RuntimeError("Studio advance returned invalid metadata")
        return result, generation

    try:
        snapshot, generation = request("/api/inspect")
        view = snapshot.get("view")
        if not isinstance(view, dict) or view.get("live_advance") is not True or view.get("input_events") is not True:
            raise RuntimeError("Live stepping requires the selected final frame of an interactive Studio worker")
        for key, minimum in (("input_revision", 0), ("frame_index", 0), ("frame_count", 1), ("fps", 1)):
            value = view.get(key)
            if isinstance(value, bool) or not isinstance(value, int) or value < minimum:
                raise RuntimeError("Studio advance returned invalid view counters")
        if view["frame_index"] != view["frame_count"] - 1:
            raise RuntimeError("Historical captures are read-only")
        if frames > view["fps"]:
            raise ValueError("Each live advance is limited to one nominal second")
        if generation is None or not generation.isascii() or not generation.isdecimal() or int(generation) < 1:
            raise RuntimeError("Studio advance returned no valid worker generation")
        result, _ = request("/api/advance", dict(worker_generation=generation,
            frame=view["frame_index"], revision=view["input_revision"], frames=frames))
        digest = result.get("sha256")
        if not isinstance(digest, str) or len(digest) != 64 or any(c not in "0123456789abcdef" for c in digest):
            raise RuntimeError("Studio advance returned no native frame receipt")
        return result
    except urllib.error.HTTPError as error:
        with error:
            detail = error.read(4096).decode("utf-8", "replace")
        raise RuntimeError(f"Studio advance refused ({error.code}); not retried: {detail}") from None
    except (urllib.error.URLError, TimeoutError, OSError) as error:
        # A timed-out request may have executed effects. Never retry it.
        raise RuntimeError("Studio advance connection failed; effects may have occurred and were not retried") from error
