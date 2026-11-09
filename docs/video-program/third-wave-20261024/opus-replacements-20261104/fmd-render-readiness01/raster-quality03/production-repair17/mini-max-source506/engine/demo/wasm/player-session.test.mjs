import assert from "node:assert/strict";
import test from "node:test";
import { TimelineSession, readLocalBundle } from "./player-session.mjs";

const bytes = () => new Uint8Array([70, 77, 84, 76]);
function deferred() {
  let resolve, reject;
  const promise = new Promise((yes, no) => { resolve = yes; reject = no; });
  return { promise, resolve, reject };
}
function fakePlayer(options = {}) {
  return {
    frame_count: 45, fps: 30, duration_seconds: 1.5,
    engine_version: "test-engine", has_camera_track: true,
    freed: 0, rendered: [], cursor: null,
    set_viewport(width, height) { this.width = width; this.height = height; },
    render_into(index, dst) { this.rendered.push(index); dst.fill(123); },
    seek_frame(index) { this.cursor = index; },
    labels() { return ["start", "end"]; },
    frame_of_label(name) { return name === "start" ? 0 : undefined; },
    free() { this.freed++; },
    ...options,
  };
}

test("loads the first frame and camera/label metadata before publishing", async () => {
  const player = fakePlayer();
  const session = new TimelineSession(() => player, 4, 3);
  assert.equal(await session.load(bytes), true);
  assert.deepEqual(player.rendered, [0]);
  assert.equal(player.cursor, 0);
  assert.equal(session.current.scratch.length, 48);
  assert.ok(session.current.scratch.every((byte) => byte === 123));
  assert.equal(session.current.metadata.hasCameraTrack, true);
  assert.deepEqual(session.current.metadata.labels, [
    { name: "start", index: 0 }, { name: "end", index: null },
  ]);
  assert.equal(player.freed, 0);
});

test("successful replacement releases the old Wasm player exactly once", async () => {
  const first = fakePlayer(), second = fakePlayer();
  const players = [first, second];
  const session = new TimelineSession(() => players.shift(), 4, 3);
  await session.load(bytes);
  await session.load(bytes);
  assert.equal(session.current.player, second);
  assert.equal(first.freed, 1);
  assert.equal(second.freed, 0);
  session.dispose();
  session.dispose();
  assert.equal(second.freed, 1);
  assert.equal(session.current, null);
});

test("decoder errors preserve the existing movie and its pixels", async () => {
  const original = fakePlayer();
  let fail = false;
  const session = new TimelineSession(() => {
    if (fail) throw new Error("EngineMismatch");
    return original;
  }, 4, 3);
  await session.load(bytes);
  const before = session.current;
  fail = true;
  await assert.rejects(session.load(bytes), /EngineMismatch/);
  assert.equal(session.current, before);
  assert.equal(original.freed, 0);
});

test("render and metadata failures release candidates without replacing the movie", async () => {
  for (const failing of [
    fakePlayer({ render_into() { throw new Error("render refused"); } }),
    fakePlayer({ labels() { throw new Error("labels refused"); } }),
    fakePlayer({ set_viewport() { throw new Error("viewport refused"); } }),
    fakePlayer({ frame_count: 0 }),
    fakePlayer({ fps: 0 }),
  ]) {
    const original = fakePlayer();
    const players = [original, failing];
    const session = new TimelineSession(() => players.shift(), 4, 3);
    await session.load(bytes);
    const before = session.current;
    await assert.rejects(session.load(bytes));
    assert.equal(session.current, before);
    assert.equal(original.freed, 0);
    assert.equal(failing.freed, 1);
  }
});

test("a stale successful read cannot replace a newer movie or allocate a player", async () => {
  const slow = deferred();
  const player = fakePlayer();
  let created = 0;
  const session = new TimelineSession(() => { created++; return player; }, 4, 3);
  const older = session.load(() => slow.promise);
  assert.equal(await session.load(bytes), true);
  slow.resolve(bytes());
  assert.equal(await older, false);
  assert.equal(created, 1);
  assert.equal(session.current.player, player);
});

test("a stale failed read does not surface an error over the newer movie", async () => {
  const slow = deferred();
  const session = new TimelineSession(() => fakePlayer(), 4, 3);
  const older = session.load(() => slow.promise);
  await session.load(bytes);
  slow.reject(new Error("old read failed"));
  assert.equal(await older, false);
  assert.ok(session.current);
});

test("a newer failure still invalidates an older outstanding load", async () => {
  const slow = deferred();
  const original = fakePlayer();
  const session = new TimelineSession(() => original, 4, 3);
  await session.load(bytes);
  const older = session.load(() => slow.promise);
  await assert.rejects(session.load(() => { throw new Error("new read failed"); }));
  slow.resolve(bytes());
  assert.equal(await older, false);
  assert.equal(session.current.player, original);
});

test("disposal invalidates reads and prevents future player construction", async () => {
  const slow = deferred();
  let created = 0;
  const session = new TimelineSession(() => { created++; return fakePlayer(); }, 4, 3);
  const pending = session.load(() => slow.promise);
  session.dispose();
  slow.resolve(bytes());
  assert.equal(await pending, false);
  assert.equal(created, 0);
  await assert.rejects(session.load(bytes), /disposed/);
});

test("session byte admission precedes decoder invocation", async () => {
  let created = 0;
  const session = new TimelineSession(() => { created++; return fakePlayer(); }, 4, 3, 3);
  await assert.rejects(session.load(bytes), /input budget/);
  await assert.rejects(session.load(() => "not bytes"), TypeError);
  assert.equal(created, 0);
});

test("oversized local files refuse before reading their contents", async () => {
  let reads = 0;
  await assert.rejects(readLocalBundle({
    size: 5,
    arrayBuffer() { reads++; return new ArrayBuffer(5); },
  }, 4), /input budget/);
  assert.equal(reads, 0);
  assert.deepEqual(await readLocalBundle({
    size: 4, arrayBuffer() { return bytes().buffer; },
  }, 4), bytes());
});

test("local file reads validate the returned size as well as metadata", async () => {
  await assert.rejects(readLocalBundle({
    size: 1, arrayBuffer() { return new ArrayBuffer(5); },
  }, 4), /input budget/);
  await assert.rejects(readLocalBundle({ size: NaN }), TypeError);
});

test("viewport and byte-budget validation happens before any loading", () => {
  for (const value of [0, -1, 4097, 0.5, NaN, Infinity]) {
    assert.throws(() => new TimelineSession(fakePlayer, value, 3), RangeError);
    assert.throws(() => new TimelineSession(fakePlayer, 4, value), RangeError);
  }
  assert.throws(() => new TimelineSession(null, 4, 3), TypeError);
  assert.throws(() => new TimelineSession(fakePlayer, 4, 3, 0), RangeError);
});
