// Own one validated, renderable FMTL player. Replacement is transactional:
// failed/stale loads cannot discard the current movie or leak Wasm players.
export const MAX_BUNDLE_BYTES = 256 * 1024 * 1024;
const MAX_U32 = 0xffff_ffff;

function positiveInteger(value, name, max) {
  if (!Number.isInteger(value) || value < 1 || value > max) {
    throw new RangeError(`${name} must be an integer in 1..=${max}`);
  }
}

export async function readLocalBundle(file, maxBytes = MAX_BUNDLE_BYTES) {
  positiveInteger(maxBytes, "bundle byte budget", Number.MAX_SAFE_INTEGER);
  if (!file || !Number.isSafeInteger(file.size) || file.size < 0) {
    throw new TypeError("select a local FMTL bundle file");
  }
  if (file.size > maxBytes) {
    throw new RangeError(`bundle exceeds the ${maxBytes}-byte browser input budget`);
  }
  const bytes = new Uint8Array(await file.arrayBuffer());
  if (bytes.byteLength > maxBytes) {
    throw new RangeError(`bundle exceeds the ${maxBytes}-byte browser input budget`);
  }
  return bytes;
}

export class TimelineSession {
  #createPlayer;
  #width;
  #height;
  #maxBytes;
  #generation = 0;
  #disposed = false;
  #current = null;

  constructor(createPlayer, width, height, maxBytes = MAX_BUNDLE_BYTES) {
    if (typeof createPlayer !== "function") throw new TypeError("player factory required");
    positiveInteger(width, "viewport width", 4096);
    positiveInteger(height, "viewport height", 4096);
    positiveInteger(maxBytes, "bundle byte budget", Number.MAX_SAFE_INTEGER);
    this.#createPlayer = createPlayer;
    this.#width = width;
    this.#height = height;
    this.#maxBytes = maxBytes;
  }

  get current() { return this.#current; }

  async load(readBytes) {
    if (this.#disposed) throw new Error("timeline session is disposed");
    const generation = ++this.#generation;
    let candidate = null;
    try {
      const bytes = await readBytes();
      if (this.#disposed || generation !== this.#generation) return false;
      if (!(bytes instanceof Uint8Array)) throw new TypeError("bundle bytes must be Uint8Array");
      if (bytes.byteLength > this.#maxBytes) {
        throw new RangeError(`bundle exceeds the ${this.#maxBytes}-byte browser input budget`);
      }
      candidate = this.#createPlayer(bytes);
      candidate.set_viewport(this.#width, this.#height);
      positiveInteger(candidate.frame_count, "displayable frame count", MAX_U32);
      positiveInteger(candidate.fps, "bundle fps", MAX_U32);
      const scratch = new Uint8Array(this.#width * this.#height * 4);
      // A valid container can still contain content the renderer refuses.
      // Prove the first frame renderable before releasing the previous player.
      candidate.render_into(0, scratch);
      candidate.seek_frame(0);
      const metadata = Object.freeze({
        frameCount: candidate.frame_count,
        fps: candidate.fps,
        durationSeconds: candidate.duration_seconds,
        engineVersion: candidate.engine_version,
        hasCameraTrack: Boolean(candidate.has_camera_track),
        labels: Object.freeze(candidate.labels().map((name) => {
          const index = candidate.frame_of_label(name);
          return Object.freeze({
            name,
            index: Number.isInteger(index) && index >= 0 && index < candidate.frame_count
              ? index : null,
          });
        })),
      });
      if (this.#disposed || generation !== this.#generation) return false;
      const previous = this.#current;
      this.#current = Object.freeze({ player: candidate, scratch, metadata });
      candidate = null; // ownership has moved to the session
      previous?.player.free();
      return true;
    } catch (error) {
      if (this.#disposed || generation !== this.#generation) return false;
      throw error;
    } finally {
      candidate?.free();
    }
  }

  dispose() {
    this.#disposed = true;
    ++this.#generation;
    const previous = this.#current;
    this.#current = null;
    previous?.player.free();
  }
}
