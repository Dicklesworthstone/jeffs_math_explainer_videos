export interface ThreadedSceneOptions {
  /** Worker count. Defaults to hardware concurrency minus one, clamped to 2..32. */
  threads?: number;
  /** Per-worker operation timeout in milliseconds. Defaults to 30000. */
  timeoutMs?: number;
}

export class ThreadedWasmUnavailableError extends Error {
  readonly code: string;
  constructor(code: string, message: string);
}

export function assertThreadedWasmAvailable(): void;

export class ThreadedFmnScene {
  static create(
    kind: string,
    width: number,
    height: number,
    options?: ThreadedSceneOptions,
  ): Promise<ThreadedFmnScene>;

  readonly memory: WebAssembly.Memory;
  readonly threadCount: number;
  readonly width: number;
  readonly height: number;
  readonly frameCount: number;
  readonly engineVersion: string;

  renderFrame(frameIndex: number): Promise<Uint8Array>;
  renderFrames(frameIndices: number[]): Promise<Uint8Array[]>;
  close(): Promise<void>;
}

export function createThreadedScene(
  kind: string,
  width: number,
  height: number,
  options?: ThreadedSceneOptions,
): Promise<ThreadedFmnScene>;
