// Real Chrome + shipped fmn Studio acceptance. No browser framework or mock
// server. Build fmn through RCH first, then pass its exact executable path.
// Usage: /usr/bin/node browser.mjs /absolute/fmn /absolute/evidence-directory
import {spawn, spawnSync, execFileSync} from "node:child_process";
import {mkdir, mkdtemp, readFile, writeFile, open, symlink} from "node:fs/promises";
import {createHash} from "node:crypto";
import {tmpdir} from "node:os";
import {join, resolve} from "node:path";
import {once} from "node:events";
import assert from "node:assert/strict";

const binary = resolve(process.argv[2]), output = resolve(process.argv[3]);
const chrome = process.env.FMN_CHROME || "/usr/bin/google-chrome";
const userAgent = "OpenAI File Downloader, XaiImageApiFetch/1.0";
await mkdir(output, {recursive:true});
const profile = await mkdtemp(join(tmpdir(), "fmn-studio-browser-"));
const secrets = [];
const redact = text => secrets.reduce((s, secret) => s.split(secret).join("[session]"), String(text));
const sleep = ms => new Promise(r => setTimeout(r, ms));
async function until(fn, label, milliseconds = 30000) {
  const end = Date.now() + milliseconds;
  while (Date.now() < end) { const value = await fn(); if (value) return value; await sleep(50); }
  throw new Error(`Timed out: ${label}`);
}
async function stop(child) {
  if (child.exitCode !== null || child.signalCode !== null) return;
  child.stdin?.end();
  const stopped = once(child,"exit");
  const timer = setTimeout(() => child.kill("SIGTERM"), 5000);
  const killTimer = setTimeout(() => child.kill("SIGKILL"), 10000);
  try { await stopped; } finally { clearTimeout(timer); clearTimeout(killTimer); }
}
async function startStudio(scene, extra = [], threads = "1") {
  const child = spawn(binary, ["studio","--robot","--no-browser","--resolution","384x216","--fps","8","--threads",threads,...extra,"@builtin",scene],
    {stdio:["pipe","pipe","pipe"], env:{...process.env, PYTHONHOME:"", PYTHONPATH:""}});
  let stdout = "";
  let stderr = "";
  child.stdout.on("data", chunk => { stdout += chunk; if (stdout.length > 65536) child.kill("SIGTERM"); });
  child.stderr.on("data", chunk => { stderr = (stderr + chunk).slice(-65536); });
  try {
    const ready = await until(() => {
      if (child.exitCode !== null) throw new Error(`Studio exited ${child.exitCode}: ${redact(stderr)}`);
      for (const line of stdout.split("\n")) { try { const data = JSON.parse(line); if (data.kind === "studio_ready") return data; } catch {} }
    }, "native Studio ready");
    const url = new URL(ready.url), cap = url.searchParams.get("cap");
    assert.equal(url.protocol,"http:"); assert.ok(["127.0.0.1","[::1]"].includes(url.hostname));
    assert.ok(cap); secrets.push(cap);
    return {child, url, cap, stderr:() => stderr};
  } catch (error) { await stop(child); throw error; }
}

const browser = spawn(chrome, ["--headless=new","--disable-gpu","--disable-dev-shm-usage","--no-first-run","--no-default-browser-check",
  "--remote-debugging-port=0",`--user-data-dir=${profile}`,`--user-agent=${userAgent}`,"about:blank"], {stdio:["ignore","ignore","pipe"]});
browser.stderr.resume();
let ws;
let studio;
let captureFailure;
const receipt = {schema:"fmn.studio.browser.v1", source_commit:execFileSync("git",["rev-parse","HEAD"],{encoding:"utf8"}).trim(),
  source_status:execFileSync("git",["status","--porcelain","--untracked-files=no"],{encoding:"utf8"}).trim(),
  binary_sha256:createHash("sha256").update(await readFile(binary)).digest("hex"), scenarios:[], browser_errors:[], input_transport:[]};
try {
  const port = await until(async () => { try { return Number((await readFile(join(profile,"DevToolsActivePort"),"utf8")).split("\n")[0]); } catch { return null; } }, "Chrome DevTools port");
  const targets = await (await fetch(`http://127.0.0.1:${port}/json/list`, {headers:{"User-Agent":userAgent},signal:AbortSignal.timeout(10000)})).json();
  ws = new WebSocket(targets.find(t => t.type === "page").webSocketDebuggerUrl); await once(ws,"open");
  let next = 0; const pending = new Map();
  ws.addEventListener("message", event => {
    const m = JSON.parse(event.data);
    if (m.id && pending.has(m.id)) { const p = pending.get(m.id); pending.delete(m.id); clearTimeout(p.timer); if (m.error) p.reject(new Error(m.error.message)); else p.resolve(m.result); }
    if (m.method === "Network.requestWillBeSent" || m.method === "Network.responseReceived") {
      const packet = m.params.request || m.params.response;
      const path = new URL(packet.url).pathname;
      if (path.startsWith("/api/")) {
        if (receipt.input_transport.length >= 512) receipt.input_transport.shift();
        receipt.input_transport.push({event:m.method, id:m.params.requestId, path,
          ...(packet.postData ? {body:redact(packet.postData).slice(0,4096)} : {}),
          ...(packet.status ? {status:packet.status} : {})});
      }
    }
    // ubs:ignore — this compares a public CDP event name, not an authentication secret.
    if (m.method === "Runtime.exceptionThrown") receipt.browser_errors.push(redact(m.params.exceptionDetails.exception?.description || m.params.exceptionDetails.text));
  });
  function command(method, params = {}) {
    return new Promise((resolve, reject) => { const id = ++next;
      const timer = setTimeout(() => { pending.delete(id); reject(new Error(`CDP timeout: ${method}`)); }, 30000);
      pending.set(id,{resolve,reject,timer}); ws.send(JSON.stringify({id,method,params}));
    });
  }
  async function evaluate(expression) {
    const result = await command("Runtime.evaluate", {expression, awaitPromise:true, returnByValue:true});
    if (result.exceptionDetails) throw new Error(redact(result.exceptionDetails.exception?.description || result.exceptionDetails.text));
    return result.result.value;
  }
  captureFailure = () => evaluate(`({
    error:document.getElementById("error")?.textContent,
    error_hidden:document.getElementById("error")?.hidden,
    display:document.getElementById("display")?.dataset,
    replay:document.getElementById("replay")?.textContent,
    input_enabled:document.getElementById("input-events")?.checked,
    busy:document.getElementById("inspect")?.disabled,
    focused:document.activeElement?.id,
    input_trace:window.__fmnInputTrace || [],
  })`);
  async function click(selector) {
    const point = await evaluate(`(() => { const element=document.querySelector(${JSON.stringify(selector)}); element.scrollIntoView({block:"center"}); const r=element.getBoundingClientRect(); return {x:r.x+r.width/2,y:r.y+r.height/2}; })()`);
    await command("Input.dispatchMouseEvent", {type:"mousePressed",button:"left",clickCount:1,...point});
    await command("Input.dispatchMouseEvent", {type:"mouseReleased",button:"left",clickCount:1,...point});
  }
  async function key(key, code, virtualKey) {
    await command("Input.dispatchKeyEvent", {type:"keyDown",key,code,windowsVirtualKeyCode:virtualKey});
    await command("Input.dispatchKeyEvent", {type:"keyUp",key,code,windowsVirtualKeyCode:virtualKey});
  }
  async function synchronized(frame) {
    await until(() => evaluate(`document.getElementById("display")?.dataset.synchronized === "true" && document.getElementById("display")?.dataset.frame === ${JSON.stringify(String(frame))} && document.getElementById("inspect")?.disabled === false`), `frame ${frame} synchronized`);
  }
  async function inspect() {
    return evaluate(`fetch("/api/inspect",{headers:{"X-FMN-Capability":${JSON.stringify(studio.cap)}},signal:AbortSignal.timeout(10000)}).then(r=>r.json())`);
  }
  async function screenshot(name) { const result = await command("Page.captureScreenshot",{format:"png",captureBeyondViewport:true}); await writeFile(join(output,name),Buffer.from(result.data,"base64")); }
  async function editBuiltinScene(scene, initial, frame) {
    await command("Emulation.setDeviceMetricsOverride",{width:1280,height:1000,deviceScaleFactor:1,mobile:false});
    await click("#inspect"); await synchronized(frame);
    const before = await inspect(), beforePixels = await pixelHash();
    const nested = scene === "tex_span.v1";
    const subject = nested
      ? before.nodes.find(node => node.source_span?.kind === "math_glyph" && node.source_span.excerpt === "x")
      : before.nodes.find(node => node.root && node.records.count &&
          JSON.stringify(node.bounds) !== JSON.stringify(initial.nodes.find(old => old.id === node.id)?.bounds));
    assert.ok(subject?.bounds, `${scene} has its actual authored edit target`);
    if (nested) assert.ok(subject.parents.length > 0, "edit an actual nested Tex glyph");
    const siblings = before.nodes.filter(node => node.id !== subject.id && node.records.count);
    const candidates = before.nodes.filter(node => node.bounds && (nested ? node.records.count : node.root));
    let pick;
    for (const fy of [0.5,0.2,0.8]) for (const fx of [0.5,0.2,0.8]) {
      const point = [subject.bounds[0][0] + fx * (subject.bounds[2][0] - subject.bounds[0][0]),
        subject.bounds[0][1] + fy * (subject.bounds[2][1] - subject.bounds[0][1])];
      if (!pick && !candidates.some(node => node.id !== subject.id && point.every((value,axis) =>
        value >= node.bounds[0][axis] - 0.01 && value <= node.bounds[2][axis] + 0.01))) pick = point;
    }
    assert.ok(pick, `${scene} target has a visible, independently selectable point`);
    await click("#input-events");
    await evaluate("document.getElementById('preview').scrollIntoView({block:'center'}); document.getElementById('preview').focus()");
    if (nested) await nativeKey("t",2);
    await nativePointer(pick);
    await nativePointer(pick,"mousePressed",2,1);
    await nativePointer(pick,"mouseReleased",2);
    assert.equal((await inspect()).nodes.find(node => node.id === subject.id).animating,true,
      "browser selection reaches the original native object");
    await nativeKey("g",0,["keyDown"]);
    await nativePointer(pick,"mousePressed",0,1);
    const destination = [pick[0] + 0.5,pick[1] + 0.25];
    await nativePointer(destination,"mouseMoved",0,1);
    await nativePointer(destination,"mouseReleased");
    await nativeKey("g",0,["keyUp"]);
    let edited = await inspect(), target = edited.nodes.find(node => node.id === subject.id);
    for (const [axis,delta] of [[0,0.5],[1,0.25]]) assert.ok(
      Math.abs(target.bounds[1][axis] - subject.bounds[1][axis] - delta) < 1e-4,
      `${scene} drag preserves its pointer offset and moves the selected coordinate`);
    assert.notEqual(await pixelHash(),beforePixels,"editing the authored object changes real decoded pixels");
    const width = target.bounds[2][0] - target.bounds[0][0], center = target.bounds[1];
    assert.ok(width > 0);
    await nativePointer([center[0] + 0.4,center[1]]);
    await nativeKey("t",0,["keyDown"]);
    await nativePointer([center[0] + 0.6,center[1]]);
    await nativeKey("t",0,["keyUp"]);
    edited = await inspect(); target = edited.nodes.find(node => node.id === subject.id);
    assert.ok(Math.abs((target.bounds[2][0] - target.bounds[0][0]) / width - 1.5) < 1e-4,
      "native resize uses the authored object's current center");
    for (const sibling of siblings) assert.deepEqual(edited.nodes.find(node => node.id === sibling.id).bounds,
      sibling.bounds,"editing one authored object preserves other leaf geometry");
    if (nested) assert.deepEqual(target.source_span,subject.source_span,"live glyph edits retain their real source span");
    const resizedPixels = await pixelHash();
    const fill = node => node.records.fields.find(field => field.name === "fill_rgba")?.values.slice(0,4);
    const colorSource = edited.nodes.find(node => node.id !== subject.id && node.records.count && node.bounds && fill(node));
    assert.ok(colorSource,"the authored scene has another native color source");
    const contrasting = JSON.stringify(fill(colorSource).slice(0,3)) !== JSON.stringify(fill(target).slice(0,3));
    const opacity = fill(target)[3];
    await nativeKey("c");
    await nativePointer(colorSource.bounds[1],"mousePressed",0,1);
    await nativePointer(colorSource.bounds[1],"mouseReleased");
    edited = await inspect(); target = edited.nodes.find(node => node.id === subject.id);
    assert.deepEqual(fill(target).slice(0,3),fill(colorSource).slice(0,3),"native color picking reaches the authored target");
    assert.equal(fill(target)[3],opacity,"color picking preserves the selected object's opacity");
    const coloredPixels = await pixelHash();
    if (contrasting) assert.notEqual(coloredPixels,resizedPixels,"contrasting native color picking changes pixels");
    else assert.equal(coloredPixels,resizedPixels,"picking the same authored white has no invented color change");
    const beforePaste = edited.nodes.length;
    await nativeKey("c",2); await nativeKey("v",2);
    assert.equal((await inspect()).nodes.length,beforePaste + 1,"clipboard pastes exactly the selected primitive or glyph");
    await nativeKey("ArrowRight",8);
    const pasted = await inspect(), pastedPixels = await pixelHash();
    assert.notEqual(pastedPixels,coloredPixels,"the copied object is independently movable and visible");
    await nativeKey("z",2); await nativeKey("z",2);
    assert.equal((await inspect()).nodes.length,beforePaste,"undo removes the paste after undoing its move");
    assert.equal(await pixelHash(),coloredPixels,"undo restores actual edited pixels");
    await nativeKey("z",10); await nativeKey("z",10);
    assert.deepEqual((await inspect()).nodes,pasted.nodes,"redo restores the copied object's state and selection");
    assert.equal(await pixelHash(),pastedPixels,"redo restores the copied object's actual pixels");
    const final = await inspect();
    assert.equal(final.view.frame_index,frame,"editing never silently advances the authored frame");
    assert.equal(final.view.frame_count,initial.view.frame_count);
    assert.equal(final.scene_time,before.scene_time,"editing does not advance the scene clock");
    assert.ok(final.view.input_revision > before.view.input_revision);
    await evaluate("document.getElementById('timeline').value='0'; document.getElementById('timeline').dispatchEvent(new Event('input',{bubbles:true}))");
    await synchronized(0);
    assert.deepEqual((await inspect()).nodes,initial.nodes,"future edits do not leak into earlier authored frames");
    await click("#last"); await synchronized(frame);
    const committed = await inspect();
    assert.deepEqual(committed.nodes,final.nodes,"scrubbing replays committed edits at their own frame");
    assert.equal(await pixelHash(),pastedPixels);
    await click("#restart"); await synchronized(frame);
    await until(() => evaluate("/Restart restored/.test(document.getElementById('replay').textContent)"),"authored live scene edit replay");
    assert.deepEqual((await inspect()).nodes,committed.nodes,"worker restart restores authored geometry and edit history");
    assert.equal((await inspect()).view.input_revision,committed.view.input_revision);
    assert.equal(await pixelHash(),pastedPixels,"worker restart restores exact decoded authored-scene pixels");
    const saved = await download(scene + ".saved-session"), oldCapability = studio.cap;
    await command("Page.navigate",{url:"about:blank"}); await stop(studio.child);
    assert.equal(studio.child.exitCode,0); assert.equal(studio.stderr(),""); studio = null;
    studio = await startStudio(scene,["--restore-session",saved.path],"4");
    assert.notEqual(studio.cap,oldCapability);
    await command("Page.navigate",{url:studio.url.href}); await synchronized(frame);
    assert.deepEqual((await inspect()).nodes,committed.nodes,"fresh host restores the original native scene and all edits");
    assert.equal((await inspect()).view.input_revision,committed.view.input_revision);
    assert.equal(await pixelHash(),pastedPixels,"fresh host and four render threads reproduce exact edited pixels");
    await screenshot(scene + ".edited-restored.png");
    return {nested_selection:nested,drag:true,resize:true,color_picker:true,contrasting_recolor:contrasting,
      clipboard:true,undo:true,redo:true,frame_identity:true,restart_same_state:true,restart_same_pixels:true,
      whole_host_resume:true,cross_thread_resume:true,final_revision:committed.view.input_revision,final_pixels:pastedPixels};
  }
  await command("Page.enable"); await command("Runtime.enable"); await command("Network.enable");
  // Observe actual browser delivery without intercepting the installed handler.
  await command("Page.addScriptToEvaluateOnNewDocument", {source:`
    window.__fmnInputTrace = [];
    for (const name of ["keydown","keyup","pointerdown","pointermove","pointerup","pointercancel","gotpointercapture","lostpointercapture","blur"]) {
      window.addEventListener(name, event => {
        if (event.target?.id !== "preview" || window.__fmnInputTrace.length >= 256) return;
        window.__fmnInputTrace.push({type:event.type, key:event.key, code:event.code,
          id:event.pointerId, button:event.button, buttons:event.buttons,
          x:event.clientX, y:event.clientY});
      }, true);
    }
  `});
  await command("Emulation.setDeviceMetricsOverride",{width:1280,height:1000,deviceScaleFactor:1,mobile:false});
  for (const scene of ["circle_shift.v1","tex_span.v1"]) {
    studio = await startStudio(scene);
    await command("Page.navigate",{url:studio.url.href});
    await synchronized(0);
    const begin = await inspect(), last = begin.view.frame_count - 1;
    assert.ok(last >= 1);
    assert.equal(begin.view.frame_count,scene === "circle_shift.v1" ? 13 : 9,"live previews include the initial authored frame");
    assert.equal(begin.scene_time,0,"live frame zero precedes animation and updater sampling");
    assert.equal(begin.view.fps,30,"native windowed clock overrides requested FPS");
    assert.ok((await evaluate("document.getElementById('position').textContent")).includes(begin.scene_time.toFixed(3) + " s"));
    assert.equal(await evaluate("location.search"), "");
    assert.equal(begin.view.input_events,true,"ordinary native registrations expose actual live editing");
    assert.equal(begin.view.input_revision,0);
    assert.equal(await evaluate("document.getElementById('input-events').disabled"), false);
    const initialHash = await evaluate("document.getElementById('display').dataset.sha256");
    await click("#last"); await synchronized(last);
    const end = await inspect();
    if (scene === "circle_shift.v1") {
      assert.notDeepEqual(end.nodes, begin.nodes, "animated live records must change");
      assert.notEqual(await evaluate("document.getElementById('display').dataset.sha256"), initialHash, "rendered frame must change");
      // Affine animation updates the live placement; the shared local record
      // geometry correctly remains unchanged until a geometry edit occurs.
      const moving = end.nodes.find(node => JSON.stringify(node.placement) !== JSON.stringify(begin.nodes.find(n=>n.id===node.id)?.placement));
      assert.ok(moving);
      await click(`[data-node="${moving.id}"] > .tree-label`);
      assert.ok((await evaluate("document.getElementById('details').textContent")).includes(String(moving.records.fields.find(field=>field.name==="point").values[0])));
      await click("#details details:nth-last-child(2) > summary");
      assert.ok((await evaluate("document.querySelector('#details details:nth-last-child(2)').innerText")).includes(String(moving.placement.translation[0])));
      await screenshot(scene + ".inspection.png");
    } else {
      const glyph = end.nodes.find(n => n.source_span?.kind === "math_glyph" && n.source_span.excerpt === "x"); assert.ok(glyph);
      assert.ok(glyph.parents.length > 0, "select a nested glyph");
      await evaluate(`document.querySelector('[data-node="${glyph.id}"]').scrollIntoView({block:"center"})`);
      await click(`[data-node="${glyph.id}"] > .tree-label`);
      assert.match(await evaluate("document.getElementById('details').textContent"), /UTF-8 bytes 6\.\.7/);
      await screenshot(scene + ".inspection.png");
      assert.equal(await evaluate("document.activeElement.getAttribute('role')"), "treeitem");
      await key("Home","Home",36);
      const first = await evaluate("document.activeElement.dataset.node");
      await key("ArrowDown","ArrowDown",40);
      assert.notEqual(await evaluate("document.activeElement.dataset.node"), first, "keyboard traverses nested family");
    }
    await evaluate("document.getElementById('timeline').value='0'; document.getElementById('timeline').dispatchEvent(new Event('input',{bubbles:true}))");
    await synchronized(0);
    assert.match(await evaluate("document.getElementById('replay').textContent"), /Previewing frame 0/);
    const beforeRestart = await evaluate("Number(document.getElementById('display').dataset.publication)");
    await click("#restart"); await synchronized(last);
    await until(() => evaluate(`Number(document.getElementById('display').dataset.publication) > ${beforeRestart}`), "restarted worker publishes a new PNG part");
    assert.match(await evaluate("document.getElementById('worker').textContent"), /generation/);
    assert.match(await evaluate("document.getElementById('replay').textContent"), /Restart restored/);
    await evaluate(`(() => { const slider=document.getElementById('timeline'); for(const frame of [0,1,${last}]) { slider.value=String(frame); slider.dispatchEvent(new Event('input',{bubbles:true})); } slider.dispatchEvent(new Event('change',{bubbles:true})); })()`);
    await synchronized(last);
    assert.match(await evaluate("document.getElementById('replay').textContent"), new RegExp(`Committed frame ${last}`));
    const overlayNode = end.nodes.find(node => node.records.count > 0);
    await click(`[data-node="${overlayNode.id}"] > .tree-label`);
    for (const value of [1,2,4,8,16]) {
      await click(`#layers input[value="${value}"]`);
      await until(() => evaluate("!document.getElementById('inspect').disabled && Number(document.getElementById('overlay-state').dataset.marks)>0"), `overlay ${value}`);
      assert.ok(await evaluate("document.getElementById('overlay').getContext('2d').getImageData(0,0,384,216).data.some(value=>value!==0)"));
      await click(`#layers input[value="${value}"]`);
      await until(() => evaluate("!document.getElementById('inspect').disabled && document.getElementById('overlay-state').textContent==='Overlays off.'"), "overlay off");
    }
    await click('#layers input[value="4"]');
    await until(() => evaluate("!document.getElementById('inspect').disabled && Number(document.getElementById('overlay-state').dataset.marks)>0"), "bounds overlay");
    await screenshot(scene + ".desktop.png");
    await command("Emulation.setDeviceMetricsOverride",{width:390,height:844,deviceScaleFactor:1,mobile:false});
    assert.equal(await evaluate("document.documentElement.scrollWidth <= innerWidth"), true, "compact viewport has no horizontal overflow");
    await evaluate("document.getElementById('timeline').focus()"); await key("Home","Home",36); await synchronized(0);
    await key("End","End",35); await synchronized(last);
    await screenshot(scene + ".compact.png");
    const rejections = [];
    for (const [fields, expected] of [[{frame:"-1",commit:"true"},400],[{frame:"banana",commit:"true"},400],[{frame:String(begin.view.frame_count),commit:"false"},422]]) {
      const response = await fetch(new URL("/api/scrub", studio.url), {method:"POST", headers:{"User-Agent":userAgent,"X-FMN-Capability":studio.cap,"Origin":studio.url.origin,"Content-Type":"application/x-www-form-urlencoded"},body:new URLSearchParams(fields),signal:AbortSignal.timeout(10000)});
      const body = await response.text(); assert.equal(response.status,expected); assert.ok(body.length < 8192); assert.ok(!body.includes(studio.cap)); rejections.push(response.status);
    }
    const denied = await fetch(new URL("/api/inspect", studio.url),{headers:{"User-Agent":userAgent,"X-FMN-Capability":"0".repeat(64)},signal:AbortSignal.timeout(10000)});
    assert.equal(denied.status,403); assert.ok(!(await denied.text()).includes(studio.cap));
    const event = await fetch(new URL("/api/event", studio.url),{method:"POST",headers:{"User-Agent":userAgent,"X-FMN-Capability":studio.cap,"Origin":studio.url.origin,"Content-Type":"application/x-www-form-urlencoded"},body:"type=key_press&key=x",signal:AbortSignal.timeout(10000)});
    assert.equal(event.status,422); assert.match(await event.text(),/worker_generation, frame and revision are required/);
    // A real typed refusal must be useful in the page, too: inject an invalid
    // control value then submit through the actual installed event handler.
    await evaluate(`document.getElementById('frame').value=${JSON.stringify(String(begin.view.frame_count))}; document.getElementById('timeline-form').dispatchEvent(new Event('submit',{bubbles:true,cancelable:true}))`);
    assert.match(await evaluate("document.getElementById('error').textContent"), /Choose an integer frame/);
    assert.equal(await evaluate(`document.body.innerText.includes(${JSON.stringify(studio.cap)})`),false);
    const overlayMarks = await evaluate("Number(document.getElementById('overlay-state').dataset.marks)");
    const editing = await editBuiltinScene(scene,begin,last);
    receipt.scenarios.push({scene,frame_count:begin.view.frame_count,nested_nodes:begin.nodes.filter(n=>n.parents.length).length,
      committed_restart_frame:last,rejections,overlay_marks:overlayMarks,keyboard:true,compact:true,...editing});
    await command("Page.navigate",{url:"about:blank"}); await stop(studio.child);
    assert.equal(studio.child.exitCode,0); assert.equal(studio.stderr(),""); studio = null;
    await command("Emulation.setDeviceMetricsOverride",{width:1280,height:1000,deviceScaleFactor:1,mobile:false});
  }
  studio = await startStudio("interactive.v1");
  await command("Page.navigate", {url:studio.url.href});
  await synchronized(0);
  const interactiveInitial = await inspect();
  assert.equal(interactiveInitial.view.input_events, true);
  assert.equal(interactiveInitial.view.y_up, true, "native scenes render world +Y toward the top of the image");
  assert.equal(interactiveInitial.view.input_revision, 0);
  const nested = interactiveInitial.nodes.find(node => node.parents.length && node.records.count);
  const swatch = interactiveInitial.nodes.find(node => node.root && node.records.count);
  assert.ok(nested && swatch, "interactive canvas has a nested object and a color source");
  await click("#first"); await synchronized(0);
  await until(async () => (await inspect()).view.input_revision === 1, "initial input checkpoint");
  await click("#input-events");
  assert.equal(await evaluate("document.getElementById('input-events').checked"), true);
  await evaluate("document.getElementById('preview').focus()");
  async function inputReady(revision) {
    await until(async () => {
      const error = await evaluate("document.getElementById('error').hidden ? null : document.getElementById('error').textContent");
      if (error) throw new Error(`Native input refused: ${error}`);
      if (!await evaluate("!document.getElementById('inspect').disabled && document.getElementById('display').dataset.synchronized === 'true'")) return false;
      return (await inspect()).view.input_revision >= revision;
    }, `native input revision ${revision}`);
    assert.equal(await evaluate("document.getElementById('error').hidden"), true);
  }
  async function nativeKey(value, modifiers = 0, types = ["keyDown", "keyUp"]) {
    // CDP modifiers use Alt=1, Control=2, Meta=4, Shift=8; Studio maps DOM
    // modifier flags into its separate native bit layout after delivery.
    const view = (await inspect()).view, revision = view.input_revision;
    const code = value === "ArrowRight" ? "ArrowRight" : "Key" + value.toUpperCase();
    const windowsVirtualKeyCode = value === "ArrowRight" ? 39 : value.toUpperCase().charCodeAt(0);
    for (const type of types) await command("Input.dispatchKeyEvent", {type,key:value,code,windowsVirtualKeyCode,modifiers});
    await inputReady(revision + types.length);
    assert.equal((await inspect()).view.frame_index,view.frame_index,"keyboard input keeps its admitted frame owner");
  }
  async function nativePointer(point, type = "mouseMoved", modifiers = 0, buttons = 0) {
    const view = (await inspect()).view, revision = view.input_revision;
    const yScale = view.y_up ? -view.scale : view.scale;
    const position = await evaluate(`(() => { const r=document.getElementById('preview').getBoundingClientRect(); return {x:r.x + (${point[0]} * ${view.scale} + ${view.origin[0]}) * r.width / ${view.width}, y:r.y + (${point[1]} * ${yScale} + ${view.origin[1]}) * r.height / ${view.height}}; })()`);
    // CDP requires the held button on mouseMoved too. Omitting it can drop
    // pointer capture even when the buttons bitmask still says left is down.
    await command("Input.dispatchMouseEvent", {type,...position,modifiers,buttons,
      button: type === "mouseMoved" ? (buttons & 1 ? "left" : "none") : "left",
      ...(type === "mouseMoved" ? {} : {clickCount:1})});
    await inputReady(revision + 1);
    assert.equal((await inspect()).view.frame_index,view.frame_index,"pointer input keeps its admitted frame owner");
  }
  async function pixelHash() {
    return evaluate("(async () => { const c=document.getElementById('preview'); const bytes=c.getContext('2d').getImageData(0,0,c.width,c.height).data; return Array.from(new Uint8Array(await crypto.subtle.digest('SHA-256',bytes)),x=>x.toString(16).padStart(2,'0')).join(''); })()");
  }
  const initialPixels = await pixelHash();
  await nativeKey("t", 2);
  await nativePointer([-1.5,0]);
  await nativePointer([-1.5,0], "mousePressed", 2, 1);
  await nativePointer([-1.5,0], "mouseReleased", 2);
  let edited = await inspect();
  assert.equal(edited.nodes.find(node => node.id === nested.id).animating, true, "nested selection reaches native state");
  await nativeKey("g", 0, ["keyDown"]);
  await nativePointer([-1.5,0], "mousePressed", 0, 1);
  await nativePointer([-0.75,0.25], "mouseMoved", 0, 1);
  await nativePointer([-0.75,0.25], "mouseReleased");
  await nativeKey("g", 0, ["keyUp"]);
  edited = await inspect();
  let child = edited.nodes.find(node => node.id === nested.id);
  assert.ok(Math.abs(child.bounds[1][0] + 0.75) < 1e-5 && Math.abs(child.bounds[1][1] - 0.25) < 1e-5, "grab moves the nested object to the requested coordinates");
  assert.deepEqual(edited.nodes.find(node => node.id === swatch.id).bounds, swatch.bounds, "editing one child preserves its sibling");
  assert.notEqual(await pixelHash(), initialPixels, "native drag changes decoded rendered pixels");
  const width = child.bounds[2][0] - child.bounds[0][0];
  await nativePointer([-0.15,0.25]);
  await nativeKey("t", 0, ["keyDown"]);
  await nativePointer([0.15,0.25]);
  await nativeKey("t", 0, ["keyUp"]);
  child = (await inspect()).nodes.find(node => node.id === nested.id);
  assert.ok(Math.abs((child.bounds[2][0] - child.bounds[0][0]) / width - 1.5) < 1e-4, "native resize follows the pointer ratio");
  const resizedPixels = await pixelHash();
  await nativeKey("c");
  await nativePointer([1.5,0], "mousePressed", 0, 1);
  await nativePointer([1.5,0], "mouseReleased");
  edited = await inspect();
  child = edited.nodes.find(node => node.id === nested.id);
  const fill = node => node.records.fields.find(field => field.name === "fill_rgba").values.slice(0,4);
  assert.deepEqual(fill(child), fill(edited.nodes.find(node => node.id === swatch.id)), "native color picker uses the swatch color");
  assert.notEqual(await pixelHash(), resizedPixels, "native recoloring changes decoded pixels");
  const coloredPixels = await pixelHash(), beforePaste = edited.nodes.length;
  await nativeKey("c", 2);
  await nativeKey("v", 2);
  const pasted = await inspect();
  assert.equal(pasted.nodes.length, beforePaste + 1, "native clipboard creates one detached child copy");
  const pastedId = pasted.nodes.find(node => !edited.nodes.some(old => old.id === node.id)).id;
  await nativeKey("z", 2);
  assert.equal((await inspect()).nodes.length, beforePaste, "native undo reverses paste");
  assert.equal(await pixelHash(), coloredPixels, "native undo restores decoded pixels");
  await nativeKey("ArrowRight");
  const finalPixels = await pixelHash();
  assert.notEqual(finalPixels, coloredPixels, "selection survives clipboard undo");
  const beforePlayback = Number(await evaluate("document.getElementById('display').dataset.frame"));
  await click("#play");
  await until(() => evaluate(`Number(document.getElementById('display').dataset.frame) > ${beforePlayback}`), "native preview resumes");
  await click("#play");
  assert.equal(await evaluate("document.getElementById('play').textContent"), "Play", "native preview pauses");
  await click("#last"); await synchronized(interactiveInitial.view.frame_count - 1);
  const finalState = await inspect();
  assert.equal(await pixelHash(), finalPixels, "static native canvas edits survive timeline scrubbing");
  await screenshot("interactive.v1.edited.desktop.png");
  await click("#restart"); await synchronized(interactiveInitial.view.frame_count - 1);
  await until(() => evaluate("/Restart restored/.test(document.getElementById('replay').textContent)"), "native edit replay completes");
  const replayStatus = await evaluate("document.getElementById('replay').textContent");
  assert.ok(replayStatus.includes(`${finalState.view.input_revision} reused`), "typed input has no opaque replay barrier");
  assert.match(replayStatus, /0 re-executed/);
  const restoredState = await inspect();
  assert.deepEqual(restoredState.nodes, finalState.nodes, "checkpoint plus input reexecution restores native scene state");
  assert.equal(await pixelHash(), finalPixels, "checkpoint plus input reexecution restores decoded pixels");
  const generation = Number((await evaluate("document.getElementById('worker').textContent")).match(/generation (\d+)/)[1]);
  assert.equal(generation, 2);
  const revision = restoredState.view.input_revision, frame = restoredState.view.frame_index;
  const guarded = {worker_generation:String(generation),frame:String(frame),revision:String(revision),type:"key_press",key:"arrow_right"};
  const refusalCases = [
    [{...guarded,worker_generation:"1"},400,/stale.*generation/],
    [{...guarded,revision:"0"},422,/stale native input revision/],
    [{...guarded,target:String(pastedId)},422,/stale native input object/],
    [{...guarded,type:"mouse_motion",x:"NaN",y:"0",dx:"0",dy:"0"},400,/invalid event coordinates/],
    [{...guarded,type:"mouse_motion",x:"10000000",y:"0",dx:"0",dy:"0"},422,/budget/],
  ];
  for (const [fields,status,diagnostic] of refusalCases) {
    const response = await fetch(new URL("/api/event",studio.url), {method:"POST",headers:{"User-Agent":userAgent,"X-FMN-Capability":studio.cap,"Origin":studio.url.origin,"Content-Type":"application/x-www-form-urlencoded"},body:new URLSearchParams(fields),signal:AbortSignal.timeout(10000)});
    const text = await response.text(); assert.equal(response.status,status); assert.match(text,diagnostic);
    assert.ok(text.length < 8192 && !text.includes(studio.cap));
    assert.deepEqual((await inspect()).nodes, restoredState.nodes, "rejected input preserves native state");
    assert.equal((await inspect()).view.input_revision, revision);
  }
  const deniedInput = await fetch(new URL("/api/event",studio.url), {method:"POST",headers:{"User-Agent":userAgent,"X-FMN-Capability":"0".repeat(64),"Origin":studio.url.origin,"Content-Type":"application/x-www-form-urlencoded"},body:new URLSearchParams(guarded),signal:AbortSignal.timeout(10000)});
  assert.equal(deniedInput.status,403); assert.ok(!(await deniedInput.text()).includes(studio.cap));
  assert.deepEqual((await inspect()).nodes, restoredState.nodes);
  assert.equal(await pixelHash(), finalPixels);
  await command("Emulation.setDeviceMetricsOverride",{width:390,height:844,deviceScaleFactor:1,mobile:false});
  assert.equal(await evaluate("document.documentElement.scrollWidth <= innerWidth"), true);
  await screenshot("interactive.v1.edited.compact.png");
  await writeFile(join(output,"interactive.v1.initial.json"),JSON.stringify(interactiveInitial,null,2)+"\n");
  await writeFile(join(output,"interactive.v1.restored.json"),JSON.stringify(restoredState,null,2)+"\n");
  receipt.scenarios.push({scene:"interactive.v1",nested_selection:true,drag:true,resize:true,recolor:true,clipboard:true,undo:true,
    pause_resume:true,restart_same_pixels:true,restart_same_state:true,initial_pixels:initialPixels,final_pixels:finalPixels,
    refusals:refusalCases.map(([,status])=>status),unauthorized:deniedInput.status});

  // Download through the actual UI, terminate the entire host (not /restart),
  // then resume with an independent process, cache and fresh capability.
  async function download(directory) {
    const folder = join(output, directory); await mkdir(folder, {recursive:true});
    await command("Browser.setDownloadBehavior", {behavior:"allow",downloadPath:folder});
    assert.equal(await evaluate("document.getElementById('save-session').disabled"), false);
    await click("#save-session");
    const path = join(folder, "studio-session.fmns");
    const bytes = await until(async () => {
      try { const data = await readFile(path); return data.length > 4 ? data : null; } catch { return null; }
    }, "complete browser session download");
    assert.equal(bytes.subarray(0,4).toString(), "FMNS");
    assert.ok(secrets.every(secret => !bytes.includes(Buffer.from(secret))), "saved data carries no session capability");
    await until(() => evaluate("document.getElementById('session-save').textContent.includes('download started') && !document.getElementById('inspect').disabled"), "save status");
    return {path,bytes};
  }
  const saved = await download("saved-session");
  for (const headers of [{}, {"X-FMN-Capability":"0".repeat(64)}, {"X-FMN-Capability":studio.cap,"Origin":"https://untrusted.invalid"}]) {
    const response = await fetch(new URL("/api/session",studio.url), {headers:{"User-Agent":userAgent,...headers},signal:AbortSignal.timeout(10000)});
    assert.equal(response.status, 403, "session export retains authentication and origin checks");
    assert.ok(!(await response.text()).includes(studio.cap));
  }
  const oldCapability = studio.cap, committedFrame = restoredState.view.frame_index;
  // A transient scrub is deliberately excluded from the saved position.
  await evaluate("document.getElementById('timeline').value='0'; document.getElementById('timeline').dispatchEvent(new Event('input',{bubbles:true}));");
  await synchronized(0);
  const previewSaved = await download("saved-preview");
  assert.deepEqual(previewSaved.bytes, saved.bytes, "preview-only navigation leaves the committed archive unchanged");
  await command("Page.navigate",{url:"about:blank"}); await stop(studio.child);
  assert.equal(studio.child.exitCode,0); assert.equal(studio.stderr(),""); studio = null;
  studio = await startStudio("interactive.v1", ["--restore-session",saved.path], "4");
  assert.notEqual(studio.cap, oldCapability, "fresh host rotates the capability");
  await command("Emulation.setDeviceMetricsOverride",{width:1280,height:1000,deviceScaleFactor:1,mobile:false});
  await command("Page.navigate", {url:studio.url.href}); await synchronized(committedFrame);
  assert.deepEqual((await inspect()).nodes, restoredState.nodes, "entire-host restart restores exact edited nodes");
  assert.equal((await inspect()).view.input_revision, restoredState.view.input_revision);
  assert.equal(await pixelHash(), finalPixels, "one-to-four-thread fresh-host resume restores exact decoded pixels");
  const stale = await fetch(new URL("/api/session",studio.url), {headers:{"User-Agent":userAgent,"X-FMN-Capability":oldCapability},signal:AbortSignal.timeout(10000)});
  assert.equal(stale.status,403);
  await click("#input-events"); await evaluate("document.getElementById('preview').focus()");
  await nativeKey("z",2);
  assert.equal(await pixelHash(), coloredPixels, "native undo history survives closing the entire host");
  await nativeKey("ArrowRight"); await nativeKey("ArrowRight");
  const continuedState = await inspect(), continuedPixels = await pixelHash();
  assert.notEqual(continuedPixels, finalPixels, "resumed scene remains editable");
  await screenshot("interactive.v1.session-resumed.desktop.png");
  const continued = await download("saved-continued");
  await command("Page.navigate",{url:"about:blank"}); await stop(studio.child);
  assert.equal(studio.child.exitCode,0); assert.equal(studio.stderr(),""); studio = null;
  studio = await startStudio("interactive.v1", ["--restore-session",continued.path]);
  await command("Page.navigate", {url:studio.url.href}); await synchronized(committedFrame);
  assert.deepEqual((await inspect()).nodes, continuedState.nodes, "re-saved continuation survives a second whole-host restart");
  assert.equal(await pixelHash(), continuedPixels);
  await screenshot("interactive.v1.session-reopened.desktop.png");
  await command("Page.navigate",{url:"about:blank"}); await stop(studio.child);
  assert.equal(studio.child.exitCode,0); assert.equal(studio.stderr(),""); studio = null;

  // Import refusals must happen before publishing a ready listener, without
  // modifying the input archive or falling back to an empty scene.
  const badPath = join(output,"damaged-session.fmns"), shortPath = join(output,"truncated-session.fmns");
  const damaged = Buffer.from(saved.bytes); damaged[40] ^= 1; await writeFile(badPath,damaged);
  await writeFile(shortPath,saved.bytes.subarray(0,saved.bytes.length-1));
  const hugePath = join(output,"oversize-session.fmns");
  const huge = await open(hugePath,"wx"); await huge.truncate(64*1024*1024+1); await huge.close();
  const linkPath = join(output,"linked-session.fmns"); await symlink(saved.path,linkPath);
  const rejected = [];
  for (const [path,scene,resolution,diagnostic] of [
    [badPath,"interactive.v1","384x216",/decode saved session/],
    [shortPath,"interactive.v1","384x216",/decode saved session/],
    [hugePath,"interactive.v1","384x216",/read saved session/],
    [linkPath,"interactive.v1","384x216",/regular file/],
    [saved.path,"circle_shift.v1","384x216",/saved scene or render configuration does not match/],
    [saved.path,"interactive.v1","192x108",/configuration/],
  ]) {
    const result = spawnSync(binary,["studio","--robot","--no-browser","--resolution",resolution,"--fps","8","--threads","1","--restore-session",path,"@builtin",scene],
      {input:"",encoding:"utf8",timeout:15000,maxBuffer:65536});
    assert.equal(result.signal,null,"invalid session must not hang");
    assert.notEqual(result.status,0,"invalid session must refuse");
    assert.ok(!result.stdout.includes('"kind":"studio_ready"'), "invalid session must not publish a listener");
    assert.match(result.stdout + result.stderr,diagnostic);
    rejected.push(result.status);
  }
  assert.deepEqual(await readFile(saved.path),saved.bytes,"restore never overwrites the archive");
  receipt.scenarios.push({scene:"saved-session.v1",whole_host_restarts:2,preview_not_committed:true,
    cross_thread_resume:true,undo_history:true,continued_editing:true,rotated_capability:true,
    rejected_imports:rejected,session_sha256:createHash("sha256").update(saved.bytes).digest("hex"),
    saved_frame:committedFrame,saved_pixels:finalPixels,continued_pixels:continuedPixels});
  assert.deepEqual(receipt.browser_errors,[]);
  receipt.passed = true;
} catch (error) {
  receipt.passed = false; receipt.error = redact(error.stack || error); process.exitCode = 1;
  try { receipt.failure_state = await captureFailure?.(); }
  catch (diagnostic) { receipt.diagnostic_error = redact(diagnostic.message || diagnostic); }
} finally {
  if (studio) await stop(studio.child);
  ws?.close(); browser.kill("SIGTERM"); await stop(browser);
  await writeFile(join(output,"receipt.json"),JSON.stringify(receipt,null,2)+"\n");
  process.stdout.write(JSON.stringify(receipt)+"\n");
}
