"""Atomic host invocation ownership; no scene mutation is performed by threads."""
import gc
from threading import Event, Thread
import unittest
import weakref

from fmn_python.invocation import InvocationGuard


class Owner:
    pass


class InvocationTests(unittest.TestCase):
    def test_overlapping_claim_is_atomic_and_does_not_release_an_existing_claim(self):
        a, b, c = Owner(), Owner(), Owner()
        guard = InvocationGuard()
        with guard.hold(a, b, a, message="active"):
            with self.assertRaisesRegex(RuntimeError, "overlap"):
                with guard.hold(b, c, message="overlap"):
                    self.fail("overlapping callback entered")
            self.assertTrue(guard.busy(a))
            self.assertTrue(guard.busy(b))
            self.assertFalse(guard.busy(c))
            with guard.hold(c, message="distinct"):
                self.assertTrue(guard.busy(c))
        self.assertFalse(any(guard.busy(obj) for obj in (a, b, c)))

    def test_cross_thread_overlap_refuses_instead_of_waiting_on_authored_code(self):
        a, b, c = Owner(), Owner(), Owner()
        guard, ready, release = InvocationGuard(), Event(), Event()
        failures = []
        def callback():
            try:
                with guard.hold(a, b, message="worker"):
                    ready.set()
                    if not release.wait(5):
                        raise TimeoutError("test did not release callback")
            except BaseException as error:
                failures.append(error)
        thread = Thread(target=callback)
        thread.start()
        try:
            self.assertTrue(ready.wait(5))
            with self.assertRaisesRegex(RuntimeError, "overlap"):
                with guard.hold(b, c, message="overlap"):
                    self.fail("concurrent overlap entered")
            self.assertTrue(guard.busy(a))
            self.assertFalse(guard.busy(c))
        finally:
            release.set()
            thread.join(5)
        self.assertFalse(thread.is_alive())
        self.assertEqual(failures, [])
        self.assertFalse(guard.busy(a))

    def test_owner_is_retained_only_until_invocation_unwinds(self):
        guard, obj = InvocationGuard(), Owner()
        reference = weakref.ref(obj)
        with self.assertRaises(KeyboardInterrupt):
            with guard.hold(obj, message="owned"):
                del obj
                gc.collect()
                self.assertIsNotNone(reference())
                raise KeyboardInterrupt()
        gc.collect()
        self.assertIsNone(reference())


if __name__ == "__main__":
    unittest.main(verbosity=2)
