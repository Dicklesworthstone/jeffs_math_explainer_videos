// Exercise the shipped controller, not a parallel event-queue implementation.
// DOM/HTTP fixtures isolate admission, ordering and ownership. browser.mjs is the
// independent real Chrome + native subprocess/pixel acceptance suite.
import assert from 'node:assert/strict';
import {test} from 'node:test';
import {readFileSync} from 'node:fs';
import vm from 'node:vm';

const source = readFileSync(new URL('../src/studio.js', import.meta.url), 'utf8');
const start = source.indexOf('  async function run(');
const end = source.indexOf('  // FrameHub parts', start);
assert.ok(start >= 0 && end > start, 'production input controller boundaries');
const controller = source.slice(start, end);
const overlayStart = source.indexOf('  function drawOverlay() {');
const viewStart = source.indexOf('  function viewOf(snapshot) {');
const viewEnd = source.indexOf('  function matchesFrame() {', viewStart);
assert.ok(overlayStart >= 0 && overlayStart < start && viewStart >= 0 && viewEnd > viewStart,
  'production inspector and overlay boundaries');

class Element {
  constructor() { this.events = new Map(); this.style = {}; this.checked = true; this.capture = new Set(); }
  addEventListener(name, callback) {
    if (!this.events.has(name)) this.events.set(name, []);
    this.events.get(name).push(callback);
  }
  emit(name, values = {}) {
    const event = {key:'', button:0, buttons:0, pointerId:1, clientX:48, clientY:27,
      ctrlKey:false, shiftKey:false, altKey:false, metaKey:false, preventDefault() {}, ...values};
    for (const callback of this.events.get(name) || []) callback(event);
  }
  getBoundingClientRect() { return this.rect || {left:0, top:0, width:96, height:54}; }
  focus() {}
  setPointerCapture(id) { this.capture.add(id); }
  hasPointerCapture(id) { return this.capture.has(id); }
  releasePointerCapture(id) { this.capture.delete(id); this.emit('lostpointercapture', {pointerId:id}); }
}
function fixture({yUp = true} = {}) {
  const elements = new Map(), $ = id => {
    if (!elements.has(id)) elements.set(id, new Element());
    return elements.get(id);
  };
  const window = new Element(), document = new Element();
  let frame = 5, revision = 0, generation = 3, release, first = true, animationFrame;
  const gate = new Promise(resolve => { release = resolve; });
  const calls = [], errors = [], control = {fail:false, conflict:false};
  const state = {snapshot:null, shown:null, expected:null, connected:true, generation,
    inputs:[], restartPending:false, busy:false, pending:null, playing:false,
    collapsed:new Set(), refreshNeeded:false, canSave:true, savePending:false, liveRunning:false};
  const refresh = async () => {
    state.generation = generation;
    state.snapshot = {view:{frame_index:frame, frame_count:control.live ? 6 : 61, width:96, height:54,
      scale:6, origin:[48,27], y_up:yUp, fps:30, input_events:true, input_revision:revision, live_advance:Boolean(control.live)}};
  };
  void refresh();
  const api = async (path, fields) => {
    calls.push({path, ...fields});
    if (first) { first = false; await gate; }
    if (path === '/api/event' || path === '/api/advance') {
      if (control.fail) throw new Error('native refusal');
      assert.equal(Number(fields.worker_generation), generation, 'original worker owner');
      assert.equal(Number(fields.revision), revision, 'optimistic revision');
      revision += control.conflict ? 2 : 1;
    } else if (path === '/api/restart') generation++;
    else if (path === '/api/scrub') { frame = Number(fields.frame); if (fields.commit === 'true') revision++; }
    return {frame_index:frame, sha256:'f'.repeat(64), worker_generation:generation,
      reused_entries:revision, replayed_entries:0, reexecuted_entries:0};
  };
  const context = vm.createContext({$,window,document,state,api,refresh,
    async downloadSession() { calls.push({path:'/api/session'}); if (control.failDownload) throw new Error('download refused'); },
    clearError() {}, report(error) { errors.push(error.message); }, displayState() {},
    matchesFrame() { return control.matched !== false; }, requestAnimationFrame(callback) { animationFrame = callback; }});
  vm.runInContext(controller + '\n globalThis.queue = {enqueueInput,seek,heldKeys};', context);
  return {$,window,document,state,calls,errors,control,release,queue:context.queue, refresh,
    tick(now) { animationFrame(now); },
    async settled() {
      release();
      for (let i=0; i<1000 && (state.busy || state.inputs.length || state.restartPending || state.pending || state.savePending); i++) {
        await new Promise(resolve => setImmediate(resolve));
      }
      assert.equal(state.busy, false, 'controller must settle');
    }};
}
const eventCalls = f => f.calls.filter(call => call.path === '/api/event');
test('saving drains accepted gestures and releases before exactly one export', async () => {
  const f = fixture(), preview = f.$('preview');
  preview.emit('keydown', {key:'g'}); preview.emit('pointerdown', {buttons:1});
  preview.emit('pointermove', {buttons:1, clientX:60});
  f.$('save-session').emit('click'); f.$('save-session').emit('click');
  preview.emit('keydown', {key:'x'}); f.queue.seek(15); f.$('restart').emit('click');
  await f.settled();
  assert.deepEqual(f.calls.map(x => x.type || x.path), ['key_press','mouse_press','mouse_drag','mouse_release','key_release','/api/session']);
  assert.match(f.$('session-save').textContent, /download started/);
  assert.equal(f.state.playing, false);
});
test('saving waits for an admitted seek without promoting a preview to a commit', async () => {
  const f = fixture(); f.queue.seek(12, false); f.$('save-session').emit('click');
  await f.settled();
  assert.deepEqual(f.calls.map(x=>x.path), ['/api/scrub','/api/session']);
  assert.equal(f.calls[0].commit, 'false');
});
test('failed edits cancel queued save and failed downloads never report success', async () => {
  const f = fixture(); f.control.fail = true;
  f.$('preview').emit('keydown', {key:'g'}); f.$('save-session').emit('click');
  await f.settled();
  assert.ok(!f.calls.some(x=>x.path === '/api/session'));
  const download = fixture(); download.control.failDownload = true;
  download.$('save-session').emit('click'); await download.settled();
  assert.equal(download.$('session-save').textContent, 'Session was not downloaded.');
  assert.deepEqual(download.errors, ['download refused']);
});
function gesture(f) {
  const preview = f.$('preview');
  preview.emit('keydown', {key:'g'});
  preview.emit('pointerdown', {buttons:1});
  preview.emit('pointermove', {buttons:1, clientX:50});
  preview.emit('pointermove', {buttons:1, clientX:53});
  preview.emit('pointerup', {clientX:55});
  f.window.emit('keyup', {key:'g'});
}

test('rapid held-key drag preserves every admitted transition while rendering is in flight', async () => {
  const f = fixture(); gesture(f);
  assert.equal(f.calls.length, 1);
  await f.settled();
  const calls = eventCalls(f);
  assert.deepEqual(calls.map(x => x.type), ['key_press','mouse_press','mouse_drag','mouse_drag','mouse_release','key_release']);
  assert.deepEqual(calls.map(x => x.revision), ['0','1','2','3','4','5']);
  assert.ok(calls.every(x => x.frame === '5' && x.worker_generation === '3'));
  assert.deepEqual(calls.filter(x => x.type === 'mouse_drag').map(x => x.dx), [1/3, 1/2]);
  assert.equal(f.errors.length, 0);
});

for (const [yUp, pressY, dragY, releaseY, wheelY] of [
  [true, 72, 81, 78, 63],
  [false, 78, 69, 72, 87],
]) {
  test(`pointer positions and deltas follow the renderer's ${yUp ? 'up' : 'down'} Y map through CSS scaling`, async () => {
    const f = fixture({yUp}), preview = f.$('preview');
    // The image is 96 x 54; CSS doubles it and moves its top-left to (13,21).
    // These independently chosen pixel positions represent the same world
    // gesture in the two native ScreenMap orientations.
    preview.rect = {left:13, top:21, width:192, height:108};
    preview.emit('pointerdown', {buttons:1, clientX:91, clientY:pressY});
    preview.emit('pointermove', {buttons:1, clientX:100, clientY:dragY});
    preview.emit('pointerup', {clientX:103, clientY:releaseY});
    preview.emit('wheel', {clientX:121, clientY:wheelY, deltaX:2, deltaY:-3});
    await f.settled();
    const calls = eventCalls(f);
    assert.deepEqual(calls.map(({type,x,y}) => [type,x,y]), [
      ['mouse_press',-1.5,0.25], ['mouse_drag',-0.75,-0.5],
      ['mouse_release',-0.5,-0.25], ['mouse_scroll',1,1],
    ]);
    assert.deepEqual([calls[1].dx,calls[1].dy], [0.75,-0.75]);
    assert.deepEqual([calls[3].offset_x,calls[3].offset_y], [2,-3]);
    assert.equal(f.errors.length, 0);
  });
}

for (const [yUp, pointY, boundsY, boundsHeight] of [
  [true,24,28.5,-4.5], [false,30,25.5,4.5],
]) {
  test(`control points and bounds use the same ${yUp ? 'up' : 'down'} Y map as the preview`, () => {
    const marks = [], status = {dataset:{}};
    const canvas = {width:96, height:54, getContext:() => ({
      clearRect() {}, beginPath() {}, fill() {},
      arc(x,y) { marks.push(['point',x,y]); },
      strokeRect(x,y,width,height) { marks.push(['bounds',x,y,width,height]); },
    })};
    const state = {
      snapshot:{view:{frame_index:0, width:96, height:54, scale:6, origin:[48,27], y_up:yUp}, nodes:[]},
      overlay:{layers:6, tiles:[], nodes:[{id:0, control_points:[[1,0.5,0]],
        bounds:[[-0.5,-0.25,0],[0.25,0.125,0],[1,0.5,0]]}]},
    };
    vm.runInNewContext(source.slice(overlayStart, start) + '\n drawOverlay();', {
      state, $:id => id === 'overlay' ? canvas : status,
      layers:() => 6, matchesFrame:() => true,
    });
    assert.deepEqual(marks, [['point',54,pointY], ['bounds',45,boundsY,9,boundsHeight]]);
    assert.equal(status.dataset.marks, '2');
  });
}

test('inspector views require an explicit boolean renderer orientation', () => {
  const snapshot = {version:1, scene_time:0, nodes:[], view:{frame_index:0, frame_count:1,
    fps:30, width:96, height:54, scale:6, origin:[48,27], input_events:false}};
  const readView = () => vm.runInNewContext(source.slice(viewStart, viewEnd) + '\n viewOf(snapshot);', {snapshot});
  for (const yUp of [undefined,null,0,1,'true']) {
    snapshot.view.y_up = yUp;
    assert.throws(readView, /invalid inspector view metadata/);
  }
  for (const yUp of [true,false]) {
    snapshot.view.y_up = yUp;
    assert.equal(readView().y_up, yUp);
  }
});

test('restart waits for accepted gesture transitions and synthetic releases', async () => {
  const f = fixture(), preview = f.$('preview');
  preview.emit('keydown', {key:'g'});
  preview.emit('pointerdown', {buttons:1});
  preview.emit('pointermove', {buttons:1, clientX:60});
  f.$('restart').emit('click');
  await f.settled();
  assert.deepEqual(f.calls.map(x => x.type || x.path), ['key_press','mouse_press','mouse_drag','mouse_release','key_release','/api/restart']);
  assert.equal(f.state.generation, 4);
  assert.equal(f.errors.length, 0);
});

test('scrub releases held native edit modes before changing frame ownership', async () => {
  const f = fixture(); f.$('preview').emit('keydown', {key:'t'});
  f.queue.seek(20);
  await f.settled();
  assert.deepEqual(f.calls.map(x => x.type || x.path), ['key_press','key_release','/api/scrub']);
  assert.equal(f.state.snapshot.view.frame_index, 20);
});

for (const trigger of ['window-blur','canvas-blur','hidden','disable','cancel','lost']) {
  test(`${trigger} releases captured input exactly once`, async () => {
    const f = fixture(), preview = f.$('preview');
    preview.emit('keydown', {key:'g'}); preview.emit('pointerdown', {buttons:1});
    if (trigger === 'window-blur') f.window.emit('blur');
    if (trigger === 'canvas-blur') preview.emit('blur');
    if (trigger === 'hidden') { f.document.hidden = true; f.document.emit('visibilitychange'); }
    if (trigger === 'disable') { f.$('input-events').checked = false; f.$('input-events').emit('change'); }
    if (trigger === 'cancel') preview.emit('pointercancel');
    if (trigger === 'lost') preview.emit('lostpointercapture');
    f.window.emit('keyup', {key:'g'}); preview.emit('pointerup');
    await f.settled();
    assert.deepEqual(eventCalls(f).map(x => x.type), ['key_press','mouse_press','mouse_release','key_release']);
    assert.equal(f.errors.length, 0);
  });
}

test('bounded queue preserves release capacity without dropping earlier accepted input', async () => {
  const f = fixture(), preview = f.$('preview');
  preview.emit('keydown', {key:'g'}); preview.emit('pointerdown', {buttons:1});
  for (let i=0; i<400; i++) preview.emit('pointermove', {buttons:1, clientX:50});
  assert.equal(f.state.inputs.length, 256);
  preview.emit('pointerup'); f.window.emit('keyup', {key:'g'});
  assert.equal(f.state.inputs.length, 258, 'only owned release transitions may use reserved capacity');
  await f.settled();
  assert.equal(eventCalls(f).length, 259);
  assert.deepEqual(eventCalls(f).slice(-2).map(x => x.type), ['mouse_release','key_release']);
});

for (const mode of ['fail','conflict']) {
  test(`${mode} discards later actions instead of retrying against another scene revision`, async () => {
    const f = fixture(); gesture(f); f.control[mode] = true;
    await f.settled();
    assert.equal(eventCalls(f).length, 1);
    assert.equal(f.state.inputs.length, 0);
    assert.equal(f.$('input-events').checked, false);
    assert.equal(f.errors.length, 1);
  });
}

test('stale admitted owner is rejected locally without sending it to a replacement worker', async () => {
  const f = fixture();
  f.queue.enqueueInput({type:'key_press', key:'g'}, {generation:2,frame:5});
  await f.settled();
  assert.equal(f.calls.length, 0);
  assert.equal(f.errors.length, 1);
});


test('modifier changes release the physical key originally admitted by native input', async () => {
  const f = fixture();
  f.$('preview').emit('keydown', {key:'g', code:'KeyG'});
  f.window.emit('keyup', {key:'G', code:'KeyG', shiftKey:true});
  await f.settled();
  assert.deepEqual(eventCalls(f).map(x => [x.type,x.key,x.modifiers]), [
    ['key_press','g','0'], ['key_release','g','1'],
  ]);
  assert.equal(f.queue.heldKeys.size, 0);
  assert.equal(f.errors.length, 0);
});

test('layout changes during a held repeat do not strand a second native key', async () => {
  const f = fixture();
  f.$('preview').emit('keydown', {key:'z', code:'KeyY'});
  f.$('preview').emit('keydown', {key:'y', code:'KeyY', repeat:true});
  f.window.emit('keyup', {key:'y', code:'KeyY'});
  await f.settled();
  assert.deepEqual(eventCalls(f).map(x => [x.type,x.key]), [
    ['key_press','z'], ['key_press','z'], ['key_release','z'],
  ]);
  assert.equal(f.queue.heldKeys.size, 0);
  assert.equal(f.errors.length, 0);
});

test('focus loss releases multiple physical keys using their admitted native values', async () => {
  const f = fixture();
  f.$('preview').emit('keydown', {key:'G', code:'KeyG', shiftKey:true});
  f.$('preview').emit('keydown', {key:'t', code:'KeyT'});
  f.window.emit('blur');
  f.window.emit('keyup', {key:'g', code:'KeyG'});
  await f.settled();
  assert.deepEqual(eventCalls(f).map(x => [x.type,x.key]), [
    ['key_press','G'], ['key_press','t'], ['key_release','G'], ['key_release','t'],
  ]);
  assert.equal(f.queue.heldKeys.size, 0);
  assert.equal(f.errors.length, 0);
});

async function liveFixture() {
  const f = fixture(); f.control.live = true; await f.refresh(); return f;
}

test('live clock is opt-in, single-flight, nominally paced, and does not catch up', async () => {
  const f = await liveFixture();
  f.tick(1000); assert.equal(f.calls.length, 0, 'paused must execute nothing');
  f.$('live-run').emit('click');
  f.tick(2000); f.tick(5000); f.tick(9000);
  assert.equal(f.calls.length, 1, 'slow worker must not accumulate clock requests');
  assert.equal(f.calls[0].path, '/api/advance');
  assert.equal(f.calls[0].frames, '1');
  assert.equal(f.calls[0].frame, '5');
  assert.equal(f.calls[0].worker_generation, '3');
  assert.equal(f.calls[0].revision, '0');
  await f.settled();
  f.tick(10000); await f.settled();
  assert.equal(f.calls.length, 2, 'only one step even after a long scheduling gap');
  assert.equal(f.calls[1].revision, '1');
  f.tick(10001); assert.equal(f.calls.length, 2, 'nominal pacing bound');
  f.$('live-run').emit('click'); f.tick(20000);
  assert.equal(f.calls.length, 2, 'Pause must not restart clock execution');
});

test('pause during an accepted live frame completes only that frame', async () => {
  const f = await liveFixture();
  f.$('live-run').emit('click'); f.tick(1000);
  f.$('live-run').emit('click');
  await f.settled(); f.tick(2000);
  assert.equal(f.calls.length, 1);
  assert.equal(f.state.liveRunning, false);
});

test('single live step cannot execute captures or queue a second step', async () => {
  const captured = fixture();
  captured.$('live-step').emit('click');
  assert.equal(captured.calls.length, 0, 'missing capability cannot run updaters');
  const f = await liveFixture();
  f.$('live-step').emit('click'); f.$('live-step').emit('click');
  await f.settled(); f.tick(1000);
  assert.equal(f.calls.length, 1);
  assert.equal(f.state.liveRunning, false, 'single step must not enable automatic execution');
});

test('clock conflicts and updater failures stop execution without retry', async () => {
  for (const error of ['conflict', 'fail']) {
    const f = await liveFixture(); f.control[error] = true;
    f.$('live-run').emit('click'); f.tick(1000); await f.settled(); f.tick(2000);
    assert.equal(f.calls.length, 1);
    assert.equal(f.state.liveRunning, false);
    assert.equal(f.errors.length, 1);
  }
});

test('input, history navigation, and reload stop the clock behind an accepted frame', async () => {
  for (const action of ['input', 'seek', 'restart']) {
    const f = await liveFixture();
    f.$('live-run').emit('click'); f.tick(1000);
    if (action === 'input') f.$('preview').emit('keydown', {key:'k'});
    else if (action === 'seek') f.queue.seek(0);
    else f.$('restart').emit('click');
    await f.settled(); f.tick(2000);
    assert.equal(f.calls[0].path, '/api/advance');
    assert.equal(f.calls.length, 2);
    assert.equal(f.state.liveRunning, false);
    if (action === 'input') assert.equal(f.calls[1].revision, '1');
  }
});

test('hidden documents and unsynchronized previews never execute live clock requests', async () => {
  const f = await liveFixture();
  f.$('live-run').emit('click'); f.document.hidden = true; f.tick(1000);
  assert.equal(f.calls.length, 0);
  f.document.emit('visibilitychange'); f.document.hidden = false; f.tick(2000);
  assert.equal(f.state.liveRunning, false, 'returning to a tab does not authorize execution');
  f.control.matched = false; f.$('live-run').emit('click'); f.tick(3000);
  assert.equal(f.calls.length, 0);
});
