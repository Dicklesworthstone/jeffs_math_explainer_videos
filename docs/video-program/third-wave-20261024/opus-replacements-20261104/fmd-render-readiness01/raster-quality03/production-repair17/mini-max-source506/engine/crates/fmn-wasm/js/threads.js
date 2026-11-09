import initThreadRuntime from "./fmn_wasm_threads.js";

const DEFAULT_THREAD_STACK_SIZE = 1_048_576;
const DEFAULT_TIMEOUT_MS = 30_000;
const MAX_WORKERS = 32;

let runtimePromise;

export class ThreadedWasmUnavailableError extends Error {
  constructor(code, message) {
    super(message);
    this.name = "ThreadedWasmUnavailableError";
    this.code = code;
  }
}

export function assertThreadedWasmAvailable() {
  if (globalThis.crossOriginIsolated !== true) {
    throw new ThreadedWasmUnavailableError(
      "FMN_WASM_CROSS_ORIGIN_ISOLATION_REQUIRED",
      "fmn-wasm/threads requires cross-origin isolation (COOP same-origin and COEP require-corp)",
    );
  }
  if (typeof SharedArrayBuffer !== "function") {
    throw new ThreadedWasmUnavailableError(
      "FMN_WASM_SHARED_ARRAY_BUFFER_UNAVAILABLE",
      "fmn-wasm/threads requires SharedArrayBuffer",
    );
  }
  if (typeof Worker !== "function") {
    throw new ThreadedWasmUnavailableError(
      "FMN_WASM_WORKER_UNAVAILABLE",
      "fmn-wasm/threads requires module Worker support",
    );
  }
}

function positiveInteger(value, name, maximum) {
  if (!Number.isInteger(value) || value < 1 || value > maximum) {
    throw new RangeError(`${name} must be an integer in 1..=${maximum}; got ${value}`);
  }
  return value;
}

function workerCount(requested) {
  if (requested !== undefined) {
    return positiveInteger(requested, "threads", MAX_WORKERS);
  }
  const available = Number.isInteger(globalThis.navigator?.hardwareConcurrency)
    ? globalThis.navigator.hardwareConcurrency
    : 2;
  return Math.min(MAX_WORKERS, Math.max(2, available - 1));
}

async function loadRuntime() {
  if (runtimePromise === undefined) {
    runtimePromise = (async () => {
      const response = await fetch(new URL("./fmn_wasm_threads_bg.wasm", import.meta.url), {
        signal: AbortSignal.timeout(DEFAULT_TIMEOUT_MS),
      });
      if (!response.ok) {
        throw new Error(`threaded WebAssembly fetch failed: HTTP ${response.status}`);
      }
      const module = await WebAssembly.compileStreaming(response);
      const exports = await initThreadRuntime({
        module_or_path: module,
        thread_stack_size: DEFAULT_THREAD_STACK_SIZE,
      });
      const memory = exports.memory;
      if (!(memory instanceof WebAssembly.Memory)) {
        throw new ThreadedWasmUnavailableError(
          "FMN_WASM_SHARED_MEMORY_MISSING",
          "threaded fmn-wasm artifact did not export its instantiated memory",
        );
      }
      if (!(memory.buffer instanceof SharedArrayBuffer)) {
        throw new ThreadedWasmUnavailableError(
          "FMN_WASM_MEMORY_NOT_SHARED",
          "threaded fmn-wasm instantiated non-shared memory and refuses to arm",
        );
      }
      return { module, memory };
    })().catch((error) => {
      runtimePromise = undefined;
      throw error;
    });
  }
  return runtimePromise;
}

class WorkerEndpoint {
  #closed = false;
  #closePromise;
  #nextTaskId = 0;
  #pending = new Map();
  #ready;
  #worker;

  constructor(runtime, scene, timeoutMs) {
    this.#worker = new Worker(new URL("./threads_worker.js", import.meta.url), {
      name: "fmn-wasm-frame-worker",
      type: "module",
    });
    this.#worker.addEventListener("message", (event) => this.#onMessage(event.data));
    this.#worker.addEventListener("error", (event) => {
      this.#crash(new Error(`threaded fmn-wasm worker failed: ${event.message}`));
    });
    this.#worker.addEventListener("messageerror", () => {
      this.#crash(new Error("threaded fmn-wasm worker returned an unreadable message"));
    });
    this.#ready = this.#request(
      {
        type: "init",
        module: runtime.module,
        memory: runtime.memory,
        threadStackSize: scene.threadStackSize,
        kind: scene.kind,
        width: scene.width,
        height: scene.height,
      },
      "ready",
      timeoutMs,
    );
  }

  async ready() {
    return this.#ready;
  }

  async render(frameIndex, timeoutMs) {
    await this.#ready;
    const reply = await this.#request({ type: "render", frameIndex }, "rendered", timeoutMs);
    if (!(reply.pixels instanceof Uint8Array)) {
      throw new Error("threaded fmn-wasm worker returned pixels with the wrong type");
    }
    return reply.pixels;
  }

  close() {
    if (this.#closePromise === undefined) this.#closePromise = this.#close();
    return this.#closePromise;
  }

  #request(message, expectedType, timeoutMs) {
    if (this.#closed) {
      return Promise.reject(new Error("threaded fmn-wasm worker pool is closed"));
    }
    const taskId = this.#nextTaskId;
    this.#nextTaskId += 1;
    return new Promise((resolve, reject) => {
      const timeout = setTimeout(() => {
        this.#pending.delete(taskId);
        reject(new Error(`threaded fmn-wasm worker timed out after ${timeoutMs} ms`));
      }, timeoutMs);
      this.#pending.set(taskId, { expectedType, reject, resolve, timeout });
      try {
        this.#worker.postMessage({ ...message, taskId });
      } catch (error) {
        this.#pending.delete(taskId);
        clearTimeout(timeout);
        reject(error);
      }
    });
  }

  #onMessage(message) {
    const pending = this.#pending.get(message?.taskId);
    if (pending === undefined) return;
    this.#pending.delete(message.taskId);
    clearTimeout(pending.timeout);
    if (message.type === "error") {
      pending.reject(new Error(message.message));
    } else if (message.type !== pending.expectedType) {
      pending.reject(
        new Error(`threaded fmn-wasm worker returned ${message.type}; expected ${pending.expectedType}`),
      );
    } else {
      pending.resolve(message);
    }
  }

  #failAll(error) {
    for (const pending of this.#pending.values()) {
      clearTimeout(pending.timeout);
      pending.reject(error);
    }
    this.#pending.clear();
  }

  #crash(error) {
    if (this.#closed) return;
    this.#closed = true;
    this.#failAll(error);
    this.#worker.terminate();
  }

  async #close() {
    if (this.#closed) return;
    try {
      await this.#request({ type: "close" }, "closed", 1_000);
    } finally {
      this.#closed = true;
      this.#failAll(new Error("threaded fmn-wasm worker pool closed"));
      this.#worker.terminate();
    }
  }
}

export class ThreadedFmnScene {
  #closed = false;
  #nextWorker = 0;
  #timeoutMs;
  #workers;

  static async create(kind, width, height, options = {}) {
    assertThreadedWasmAvailable();
    const threads = workerCount(options.threads);
    const timeoutMs = positiveInteger(
      options.timeoutMs ?? DEFAULT_TIMEOUT_MS,
      "timeoutMs",
      300_000,
    );
    const runtime = await loadRuntime();
    const scene = { kind, width, height, threadStackSize: DEFAULT_THREAD_STACK_SIZE };
    const workers = Array.from(
      { length: threads },
      () => new WorkerEndpoint(runtime, scene, timeoutMs),
    );
    try {
      const descriptions = await Promise.all(workers.map((worker) => worker.ready()));
      const expected = descriptions[0];
      for (const description of descriptions.slice(1)) {
        if (
          description.width !== expected.width ||
          description.height !== expected.height ||
          description.frameCount !== expected.frameCount ||
          description.engineVersion !== expected.engineVersion
        ) {
          throw new Error("threaded fmn-wasm workers disagree about scene identity");
        }
      }
      return new ThreadedFmnScene(runtime.memory, workers, expected, timeoutMs);
    } catch (error) {
      await Promise.allSettled(workers.map((worker) => worker.close()));
      throw error;
    }
  }

  constructor(memory, workers, description, timeoutMs) {
    this.memory = memory;
    this.threadCount = workers.length;
    this.width = description.width;
    this.height = description.height;
    this.frameCount = description.frameCount;
    this.engineVersion = description.engineVersion;
    this.#workers = workers;
    this.#timeoutMs = timeoutMs;
  }

  renderFrame(frameIndex) {
    if (this.#closed) {
      return Promise.reject(new Error("threaded fmn-wasm scene is closed"));
    }
    const worker = this.#workers[this.#nextWorker];
    this.#nextWorker = (this.#nextWorker + 1) % this.#workers.length;
    return worker.render(frameIndex, this.#timeoutMs);
  }

  renderFrames(frameIndices) {
    if (!Array.isArray(frameIndices)) {
      return Promise.reject(new TypeError("frameIndices must be an array"));
    }
    return Promise.all(frameIndices.map((frameIndex) => this.renderFrame(frameIndex)));
  }

  close() {
    if (this.#closed) return Promise.resolve();
    this.#closed = true;
    return Promise.allSettled(this.#workers.map((worker) => worker.close())).then(() => undefined);
  }
}

export function createThreadedScene(kind, width, height, options) {
  return ThreadedFmnScene.create(kind, width, height, options);
}
