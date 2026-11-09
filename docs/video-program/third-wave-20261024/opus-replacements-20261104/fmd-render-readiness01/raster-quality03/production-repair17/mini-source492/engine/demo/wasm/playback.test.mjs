import assert from "node:assert/strict";
import test from "node:test";
import { FramePlaybackClock } from "./playback.mjs";

test("30 fps playback is independent of 60/120/144 Hz display refresh", () => {
  for (const refreshRate of [60, 120, 144]) {
    const clock = new FramePlaybackClock(30, 90);
    clock.play(0);
    assert.equal(clock.frameAt(0), 0);
    for (let tick = 1; tick < refreshRate; tick++) {
      clock.frameAt(tick * 1000 / refreshRate);
    }
    assert.equal(clock.frameAt(1000), 30);
    assert.equal(clock.frameAt(2000), 60);
    assert.equal(clock.frameAt(3000), 0);
  }
});

test("play does not advance until the first authored frame boundary", () => {
  const clock = new FramePlaybackClock(25, 50);
  clock.play(100);
  assert.equal(clock.frameAt(139.999), 0);
  assert.equal(clock.frameAt(140), 1);
  assert.equal(clock.frameAt(180), 2);
});

test("late presentation catches up without accumulated clock drift", () => {
  const clock = new FramePlaybackClock(30, 45);
  clock.play(123);
  assert.equal(clock.frameAt(1123), 30);
  assert.equal(clock.frameAt(6123), 0);
  assert.equal(clock.frameAt(3_600_123), 0);
  assert.equal(clock.frameAt(3_601_123), 30);
});

test("pause freezes the displayed frame and resume excludes paused time", () => {
  const clock = new FramePlaybackClock(10, 100);
  clock.play(0);
  assert.equal(clock.frameAt(350), 3);
  clock.pause();
  assert.equal(clock.frameAt(50_000), 3);
  clock.play(50_000);
  assert.equal(clock.frameAt(50_099), 3);
  assert.equal(clock.frameAt(50_100), 4);
});

test("seek stops playback and loop wrap preserves the selected anchor", () => {
  const clock = new FramePlaybackClock(30, 45);
  clock.play(0);
  clock.seek(44);
  assert.equal(clock.playing, false);
  assert.equal(clock.frameAt(1000), 44);
  clock.play(1000);
  assert.equal(clock.frameAt(1100), 2);
});

test("duplicate play calls do not reset elapsed playback", () => {
  const clock = new FramePlaybackClock(10, 100);
  clock.play(0);
  clock.play(900);
  assert.equal(clock.frameAt(1000), 10);
});

test("an old callback cannot rewind the movie", () => {
  const clock = new FramePlaybackClock(10, 100);
  clock.play(0);
  assert.equal(clock.frameAt(1000), 10);
  assert.equal(clock.frameAt(500), 10);
  assert.equal(clock.frameAt(1100), 11);
});

test("single-frame bundles remain stable while playing", () => {
  const clock = new FramePlaybackClock(60, 1);
  clock.play(0);
  assert.equal(clock.frameAt(123456), 0);
  assert.equal(clock.playing, true);
});

test("wide frame clocks use exact integer arithmetic for whole seconds", () => {
  const clock = new FramePlaybackClock(0xffff_ffff, 0xffff_fffe);
  clock.play(0);
  assert.equal(clock.frameAt(4_294_967_292_500), 2_147_483_645);
});

test("invalid clock inputs refuse without changing playback state", () => {
  for (const value of [0, -1, 0.5, NaN, Infinity, 0x1_0000_0000]) {
    assert.throws(() => new FramePlaybackClock(value, 10), RangeError);
    assert.throws(() => new FramePlaybackClock(30, value), RangeError);
  }
  const clock = new FramePlaybackClock(30, 45);
  clock.seek(7);
  clock.play(100);
  for (const value of [-1, NaN, Infinity, Number.MAX_SAFE_INTEGER + 1]) {
    assert.throws(() => clock.frameAt(value), RangeError);
    assert.throws(() => clock.play(value), RangeError);
  }
  for (const value of [-1, 45, 0.5, NaN, Infinity]) {
    assert.throws(() => clock.seek(value), RangeError);
  }
  assert.equal(clock.frame, 7);
  assert.equal(clock.playing, true);
});
