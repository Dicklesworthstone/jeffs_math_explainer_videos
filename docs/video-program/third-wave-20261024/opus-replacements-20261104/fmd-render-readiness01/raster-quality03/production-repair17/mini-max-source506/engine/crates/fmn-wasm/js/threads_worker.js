let bindings;
let scene;

self.addEventListener("message", (event) => {
  void handleMessage(event.data);
});

async function handleMessage(message) {
  try {
    if (message.type === "init") {
      if (scene !== undefined) throw new Error("threaded fmn-wasm worker was initialized twice");
      bindings = await import("./fmn_wasm_threads.js");
      bindings.initSync({
        module: message.module,
        memory: message.memory,
        thread_stack_size: message.threadStackSize,
      });
      scene = new bindings.FmnScene(message.kind, message.width, message.height);
      self.postMessage({
        type: "ready",
        taskId: message.taskId,
        width: scene.width,
        height: scene.height,
        frameCount: scene.frame_count,
        engineVersion: bindings.engine_version(),
      });
      return;
    }
    if (message.type === "render") {
      if (scene === undefined) throw new Error("threaded fmn-wasm worker is not initialized");
      const pixels = new Uint8Array(scene.width * scene.height * 4);
      scene.render_into(message.frameIndex, pixels);
      self.postMessage(
        { type: "rendered", taskId: message.taskId, frameIndex: message.frameIndex, pixels },
        [pixels.buffer],
      );
      return;
    }
    if (message.type === "close") {
      if (scene !== undefined) scene.free();
      scene = undefined;
      self.postMessage({ type: "closed", taskId: message.taskId });
      self.close();
      return;
    }
    throw new Error(`unknown threaded fmn-wasm worker message: ${message.type}`);
  } catch (error) {
    self.postMessage({
      type: "error",
      taskId: message?.taskId,
      message: String(error?.stack ?? error),
    });
  }
}
