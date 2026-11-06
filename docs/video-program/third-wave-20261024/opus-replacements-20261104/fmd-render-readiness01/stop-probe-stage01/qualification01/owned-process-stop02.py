"""Linux dedicated-guard descendant custody, including detached FFmpeg groups.
Install before the guard starts its job. Only this supervisor's descendants are
signalled; pidfds prevent PID reuse from redirecting a signal. Original resource
policy, input verification and exclusive lane are not changed.
"""
import ctypes
import os
from pathlib import Path
import select
import signal
import time

OWNER = None
LAST_STOP = None


def install(guard):
    global OWNER
    assert hasattr(os, 'pidfd_open') and hasattr(signal, 'pidfd_send_signal')
    libc = ctypes.CDLL(None, use_errno=True)
    assert libc.prctl(36, 1, 0, 0, 0) == 0, 'Cannot acquire subreaper custody'
    flag = ctypes.c_int()
    assert libc.prctl(37, ctypes.byref(flag), 0, 0, 0) == 0 and flag.value == 1
    OWNER = os.getpid()
    guard.stop_owned = stop_owned


def rows():
    result = {}
    for p in Path('/proc').glob('[0-9]*/stat'):
        try:
            raw = p.read_text(); fields = raw[raw.rindex(')')+2:].split()
            result[int(p.parent.name)] = (int(fields[1]), fields[19], fields[0])
        except (FileNotFoundError, ProcessLookupError, PermissionError):
            continue
    return result


def descendants(table):
    owned = {OWNER}
    while True:
        more = {pid for pid, row in table.items() if row[0] in owned} - owned
        if not more:
            return {pid: table[pid] for pid in owned if pid != OWNER and pid in table}
        owned |= more


def stop_owned(child):
    global LAST_STOP
    assert OWNER == os.getpid(), 'Stop adapter must run in its installing supervisor'
    # This module is for a dedicated serial guard, never an interactive process
    # that also owns unrelated jobs. The actual wrapper has exactly that role.
    handles = {}
    started = time.monotonic()
    try:
        while True:
            for pid, row in descendants(rows()).items():
                if row[2] == 'Z' or pid in handles:
                    continue
                try:
                    fd = os.pidfd_open(pid)
                except ProcessLookupError:
                    continue
                current = rows().get(pid)
                if current is None or current[1] != row[1]:
                    os.close(fd); continue
                handles[pid] = (fd, row[1])
                try:
                    signal.pidfd_send_signal(fd, signal.SIGTERM)
                except ProcessLookupError:
                    pass
            live = {pid: item for pid, item in handles.items()
                    if not select.select([item[0]], [], [], 0)[0]}
            current_live = {pid: row for pid, row in descendants(rows()).items() if row[2] != 'Z'}
            if not live and not current_live:
                break
            elapsed = time.monotonic()-started
            if elapsed >= 20:
                for fd, _ in live.values():
                    try:
                        signal.pidfd_send_signal(fd, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
            if elapsed >= 30:
                raise RuntimeError('Owned processes survived bounded TERM/KILL: '+repr(sorted(current_live)))
            time.sleep(.1)
        child.wait(timeout=1)
        # Reap adopted encoders only after Popen has recorded its own child.
        while True:
            try:
                pid, _ = os.waitpid(-1, os.WNOHANG)
                if not pid: break
            except ChildProcessError:
                break
        LAST_STOP = {'owner':OWNER,'identities':{str(p):start for p,(_,start) in handles.items()},
                     'elapsed':time.monotonic()-started,'child_exit':child.returncode,'live_descendants':[]}
    finally:
        for fd, _ in handles.values(): os.close(fd)
