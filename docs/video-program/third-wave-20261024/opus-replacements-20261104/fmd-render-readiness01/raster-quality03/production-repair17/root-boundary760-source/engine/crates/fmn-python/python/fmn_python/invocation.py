"""Invocation-local exclusion, never copied or serialized as mobject state.

Callbacks may copy, save or checkpoint their own object. A busy flag in its
__dict__ becomes a permanent lock in those copies. Keep active identities in
the adapter instead, retaining each owner only for the duration of the call.
"""
from __future__ import annotations

from contextlib import contextmanager
from threading import Lock


class InvocationGuard:
    """Refuse overlapping invocations without locking across authored code.

    Multi-owner claims are atomic, including when callbacks release the GIL.
    A failed claim acquires nothing. Distinct copies remain independently
    usable, and BaseException unwinding releases every identity in the claim.
    This is exclusion, not a general promise of thread-safe scene mutation.
    """

    def __init__(self):
        self._active = set()
        self._lock = Lock()

    def busy(self, owner):
        with self._lock:
            return id(owner) in self._active

    @contextmanager
    def hold(self, *owners, message):
        # owners holds strong references until finally: an active id cannot be
        # recycled for an unrelated object even during collection in a callback.
        identities = {id(owner) for owner in owners}
        with self._lock:
            if not self._active.isdisjoint(identities):
                raise RuntimeError(message)
            self._active.update(identities)
        try:
            yield
        finally:
            with self._lock:
                self._active.difference_update(identities)
