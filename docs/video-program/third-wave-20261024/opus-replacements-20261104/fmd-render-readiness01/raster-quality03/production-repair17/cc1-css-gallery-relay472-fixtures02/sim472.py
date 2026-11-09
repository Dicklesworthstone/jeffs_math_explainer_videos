"""relay472 fixtures (CC1): load relay 472 into a review-466 sim466 world, exactly as sim466.load463 loads 463.
Only HERE, load_helper and subprocess are rebound. Fixture only; never an actual import."""
import hashlib
import types

import sim466 as S


def load472(world, Q0, h472, faults=None, outcome=('complete', 0)):
    src = (Q0/'cc1-css-gallery-relay472.py').read_bytes()
    assert hashlib.sha256(src).hexdigest() == h472
    m = types.ModuleType('relay472_fixture')
    m.__file__ = str(world.ctl/'cc1-css-gallery-relay472.py')
    exec(compile(src, m.__file__, 'exec'), m.__dict__)
    assert m.HERE == world.ctl and m.VERSION == '472'
    m.subprocess = S.Shim(world, m, faults, outcome)
    m.load_helper = lambda: world.h
    return m
