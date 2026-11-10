// Presentation clock for already-recorded frames. The browser may skip display
// frames when late; it never retimes or resamples the bundle's scene clock.
const MAX_U32 = 0xffff_ffff;

function timestamp(value) {
  if (!Number.isFinite(value) || value < 0 || value > Number.MAX_SAFE_INTEGER) {
    throw new RangeError("playback timestamp must be finite and in 0..=Number.MAX_SAFE_INTEGER");
  }
  return value;
}

export class FramePlaybackClock {
  #fps;
  #count;
  #frame = 0;
  #playing = false;
  #anchorFrame = 0;
  #anchorTime = 0;
  #lastTime = 0;

  constructor(fps, frameCount) {
    for (const [name, value] of [["fps", fps], ["frame count", frameCount]]) {
      if (!Number.isInteger(value) || value < 1 || value > MAX_U32) {
        throw new RangeError(`${name} must be an integer in 1..=4294967295`);
      }
    }
    this.#fps = fps;
    this.#count = frameCount;
  }

  get playing() { return this.#playing; }
  get frame() { return this.#frame; }

  play(now) {
    timestamp(now);
    if (this.#playing) return;
    this.#anchorFrame = this.#frame;
    this.#anchorTime = now;
    this.#lastTime = now;
    this.#playing = true;
  }

  // Freeze the last displayed frame, not an undisplayed wall-clock position.
  pause() { this.#playing = false; }

  seek(index) {
    if (!Number.isInteger(index) || index < 0 || index >= this.#count) {
      throw new RangeError(`frame index must be in 0..${this.#count - 1}`);
    }
    this.#frame = index;
    this.#playing = false;
  }

  frameAt(now) {
    timestamp(now);
    if (!this.#playing) return this.#frame;
    // Defensive monotonicity: an old callback cannot rewind the movie.
    this.#lastTime = Math.max(now, this.#lastTime);
    const elapsed = this.#lastTime - this.#anchorTime;
    // Multiply whole elapsed seconds exactly, even for the full u32 fps/count
    // surface. Fractional milliseconds stay within one second before scaling.
    const seconds = Math.floor(elapsed / 1000);
    const remainderMs = elapsed % 1000;
    const whole = Number((BigInt(seconds) * BigInt(this.#fps)) % BigInt(this.#count));
    const fraction = Math.floor(remainderMs * this.#fps / 1000);
    this.#frame = (this.#anchorFrame + whole + fraction) % this.#count;
    return this.#frame;
  }
}
