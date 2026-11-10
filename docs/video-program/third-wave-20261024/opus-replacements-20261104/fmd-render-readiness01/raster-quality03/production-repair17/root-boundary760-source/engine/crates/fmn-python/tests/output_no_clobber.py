"""Installed-wheel proof of final publication ownership, not an exists precheck.

Every winning artifact is rendered through Lumen/Reel. Competing generations
are opened before either publishes; late external writers and link nodes must
survive without a success receipt. ffmpeg also crosses a filesystem boundary.
"""
from pathlib import Path
import hashlib
import os
import shutil
import struct
import tempfile
import wave

import manimlib as m
from fmn_python import record_scene, render_session


def inventory(path):
    files = sorted(path.rglob("*")) if path.is_dir() else [path]
    return tuple((str(p.relative_to(path)) if p != path else "", p.stat().st_ino,
                  p.stat().st_mtime_ns, hashlib.sha256(p.read_bytes()).hexdigest())
                 for p in files if p.is_file())


def new_session(path, format):
    scene = m.Scene()
    if format == "mov":
        scene.camera.background_rgba[3] = 0
    options = dict(format=format, resolution=(96, 54), threads=1)
    if format in {"png", "svg"}:
        session = render_session(scene, path, **options)
    else:
        # A real populated, already-advanced arena must survive each recording.
        scene.add(m.Square(side_length=1, fill_opacity=1))
        scene.wait(0.125)
        session = record_scene(scene, path, **options)
    return scene, session


def advance(scene, format, sound):
    if format in {"png", "svg"}:
        scene.add(m.Square(side_length=1, fill_opacity=1))
    if format == "wav":
        scene.add_sound(str(sound))
    scene.wait(0.125)


def refused_finish(session):
    try:
        session.finish()
    except Exception as error:
        message = str(error).lower()
        assert any(word in message for word in ("exist", "publish", "publication")), message
    else:
        raise AssertionError("a competing publisher replaced an owned artifact")
    assert session.result is None
    assert vars(session.scene).get("_fmn_owned_render_session") is None


def competing_generations(root, format, sound):
    path = root / ("winner." + format)
    first_scene, first = new_session(path, format)
    second_scene, second = new_session(path, format)
    try:
        first.__enter__()
        second.__enter__()
        advance(first_scene, format, sound)
        advance(second_scene, format, sound)
        assert not path.exists(), "an unfinished generation published output"
        result = first.finish()
        before = inventory(path)
        assert before and result.bytes > 0 and not result.certified
        if format != "png_sequence":
            assert result.digest == hashlib.sha256(path.read_bytes()).hexdigest()
        refused_finish(second)
        assert inventory(path) == before, "loser changed the winner's bytes or identity"
        if format not in {"png", "svg"}:
            # Publication failure must release ownership without erasing scene effects.
            mob = second_scene.mobjects[0]
            clock = m._portal_scene_clock(second_scene)
            with record_scene(second_scene, root / ("retry." + format), format=format,
                              resolution=(96, 54), threads=1) as retry:
                advance(second_scene, format, sound)
            assert retry.result.bytes > 0
            assert second_scene.mobjects[0] is mob
            assert retry.start_frame == clock[1] and retry.end_frame > clock[1]
    finally:
        first.abort()
        second.abort()


def late_writer(root, format, sound):
    path = root / ("late." + format)
    scene, session = new_session(path, format)
    try:
        session.__enter__()
        advance(scene, format, sound)
        path.write_bytes(b"external producer owns this destination")
        before = inventory(path)
        refused_finish(session)
        assert inventory(path) == before
    finally:
        session.abort()


def link_nodes(root, format, sound):
    if os.name != "posix":
        return
    for dangling in (False, True):
        target = root / ("missing" if dangling else "target")
        if not dangling:
            target.write_bytes(b"do not modify this target")
        path = root / ("link-" + str(dangling) + "." + format)
        path.symlink_to(target)
        scene, session = new_session(path, format)
        try:
            try:
                session.__enter__()
            except Exception as error:
                assert any(word in str(error).lower() for word in ("exist", "publish", "link", "directory"))
            else:
                advance(scene, format, sound)
                refused_finish(session)
            assert path.is_symlink() and path.readlink() == target
            if dangling:
                assert not target.exists()
            else:
                assert target.read_bytes() == b"do not modify this target"
        finally:
            session.abort()


def cross_filesystem_video(root, sound):
    shared = Path("/dev/shm")
    if not shared.is_dir() or not os.access(shared, os.W_OK) or shared.stat().st_dev == root.stat().st_dev:
        print("cross-filesystem video skipped: no second writable filesystem")
        return
    with tempfile.TemporaryDirectory(prefix="fmn-video-publish-", dir=shared) as temp:
        for format in ("mp4", "mov"):
            path = Path(temp) / ("cross." + format)
            scene, session = new_session(path, format)
            with session:
                advance(scene, format, sound)
            assert session.result.digest == hashlib.sha256(path.read_bytes()).hexdigest()
            assert session.result.ffmpeg_invocations and session.result.frame_count == 4
    print("native MP4 and MOV published across filesystems")


def main():
    formats = ["png", "png_sequence", "svg", "gif", "y4m", "wav"]
    ffmpeg = shutil.which("ffmpeg")
    if ffmpeg:
        formats += ["mp4", "mov"]
    else:
        assert os.environ.get("FMN_REQUIRE_FFMPEG") != "1", "ffmpeg required for video ownership acceptance"
    with tempfile.TemporaryDirectory(prefix="fmn-publication-owner-") as temp:
        root = Path(temp)
        sound = root / "cue.wav"
        with wave.open(str(sound), "wb") as stream:
            stream.setnchannels(1)
            stream.setsampwidth(2)
            stream.setframerate(48000)
            stream.writeframes(struct.pack("<hh", 4000, -4000) * 800)
        for format in formats:
            directory = root / format
            directory.mkdir()
            competing_generations(directory, format, sound)
            late_writer(directory, format, sound)
            link_nodes(directory, format, sound)
            assert not list(directory.glob(".fmn-*")), "unpublished staging leaked"
            print("native publication ownership passed:", format)
        if ffmpeg:
            cross_filesystem_video(root, sound)
    print("native output ownership acceptance passed")


main()
