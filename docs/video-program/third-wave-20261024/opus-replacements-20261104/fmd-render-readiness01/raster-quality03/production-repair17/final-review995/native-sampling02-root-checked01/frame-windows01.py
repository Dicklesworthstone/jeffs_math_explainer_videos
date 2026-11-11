"""Exact frame windows for the combined repairs' findings (01_6 arrow, 08_5 square, 09_1/09_7/09_8/09_9 boundary),
computed from the selected timelines' line frames, phrase-timings b025eaa7 marks and the narration report's
sentence onsets, with the kit's own rules (read-only; writes one JSON next to this script).

Kit rules mirrored (explainer_kit 8072720d narration.rs / ef035faf kit.rs; draw.rs of root-boundary760-source):
- Spoken.start = line frame / 60; at(phrase) = start + mark (byte offset of text.find(phrase)), else character share
  between sentence starts (measured starts only when draw.rs's sentence split count equals the report's).
- wait_until(t): if t > now, wait frames_covering(t - now) = ceil(60 (t - now) - 1e-6) frames.
- a play of run time rt takes n = ceil(exact(rt) * 60) frames (fmn a399d626 RationalFrameClock, BN-02).
- an event starting at clock frame F with n frames shows in video frames F .. F+n-1 (sample k at video F+k-1);
  a linear tween's alpha there is min(k / (60 rt), 1).
- hold_breath / music_breath: wait until line start + len + BREATH (0.6), then window frames_covering(seconds).

usage: python3 -I -B frame-windows01.py OUT.json
"""
import hashlib
import json
import math
import sys
from fractions import Fraction

REPO = "/Users/jemanuel/projects/jeffs_math_explainer_videos/"
P17 = REPO + "docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/production-repair17/"
SCRIPT = P17 + "root-boundary760-source/workspace/explainer_corpus/complex_analysis_visual/script.tsv"
TIMINGS = P17 + "final-picture-source41/001-phrase-timings.json"
REPORT = REPO + "narration_lab/third-wave-20261024/root/five-film-release-20261027/complex-mini-transfer812/tree/inputs557/jobs/csd-review-repair-rebind313/narration/report.json"
TIMELINES = {
    "01_the_bump": (P17 + "preview274-review305/preview/01_the_bump.timeline.tsv",
                    "/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration",
                    "150147fe5bc7cdb0a111061c4530a16b104d196e004ae29bcf210d1c9c32ac81"),
    "08_loop_sums": (P17 + "preview243-review265/preview/08_loop_sums.timeline.tsv",
                     "/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration",
                     "43c8bab2f7ca61c8692e0f1686798d54e187dcfeeab0d03311c2d4c9a5f54812"),
    "09_boundary": (P17 + "root-boundary558-reference/09_boundary.timeline.tsv", None,
                    "5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e"),
}
CHAPTER_START = {"01_the_bump": 0, "08_loop_sums": 52106, "09_boundary": 67065}
CHAPTER_FRAMES = {"01_the_bump": 6311, "08_loop_sums": 14959, "09_boundary": 16504}
FPS = 60
BREATH = 0.6


def sha(path):
    return hashlib.sha256(open(path, "rb").read()).hexdigest()


def timeline(chapter):
    path, new_dir, want = TIMELINES[chapter]
    raw = open(path, "rb").read()
    text = raw.decode("utf-8")
    rows = [line.split("\t") for line in text.split("\n")]
    if new_dir is not None:
        out = []
        for cols in rows:
            cols = list(cols)
            if cols[0] == "line":
                cols[-1] = new_dir + "/" + cols[-1].rsplit("/", 1)[1]
            out.append("\t".join(cols))
        selected = hashlib.sha256("\n".join(out).encode("utf-8")).hexdigest()
    else:
        selected = hashlib.sha256(raw).hexdigest()
    lines = {c[1]: int(c[2]) for c in rows if c[0] == "line"}
    marks = [(c[1], int(c[2])) for c in rows if c[0] == "mark"]
    breaths = [(int(c[1]), c[2]) for c in rows if c[0] == "breath"]
    frames = int(next(c[1] for c in rows if c[0] == "frames"))
    return {"path": path, "local_sha256": hashlib.sha256(raw).hexdigest(), "selected_sha256": want,
            "rebuilt_equals_selected": selected == want, "lines": lines, "marks": marks, "breaths": breaths,
            "frames": frames}


def play_frames(rt):
    return math.ceil(Fraction(rt) * FPS)


def frames_covering(seconds):
    return max(0, math.ceil(seconds * float(FPS) - 1e-6))


def sentence_offsets(text):
    chars = list(text)
    out = [0]
    for i in range(max(len(chars) - 2, 0)):
        if chars[i] in ".?!" and chars[i + 1] == " ":
            out.append(i + 2)
    return out


class Spoken:
    def __init__(self, cue, frame, text, report, marks):
        self.cue, self.frame, self.text = cue, frame, text
        self.start = frame / FPS
        onsets, total = report[cue]
        self.len = total
        offs = sentence_offsets(text)
        self.measured = len(onsets) == len(offs)
        allc = max(len(text), 1)
        self.sentences = [(at, onsets[j] if self.measured else total * at / allc) for j, at in enumerate(offs)]
        self.marks = marks

    def sentence(self, j):
        return self.start + self.sentences[j][1]

    def at(self, phrase):
        b = self.text.encode("utf-8").find(phrase.encode("utf-8"))
        if b < 0:
            raise SystemExit(f"{self.cue}: no phrase {phrase!r}")
        if b in self.marks:
            return self.start + self.marks[b], "mark"
        i = len(self.text.encode("utf-8")[:b].decode("utf-8"))
        allc = max(len(self.text), 1)
        j = max(k for k, (a, _) in enumerate(self.sentences) if a <= i)
        frm, t0 = self.sentences[j]
        to, t1 = self.sentences[j + 1] if j + 1 < len(self.sentences) else (allc, self.len)
        share = (i - frm) / max(to - frm, 1)
        kind = "interpolated-last-sentence" if j + 1 >= len(self.sentences) else "interpolated"
        return self.start + t0 + (t1 - t0) * share, kind

    def end(self):
        return self.start + self.len


class Sim:
    def __init__(self, chapter, label):
        self.chapter, self.label = chapter, label
        self.clock = None
        self.events = []
        self.anchors = []
        self.line = None

    def g(self, f):
        return CHAPTER_START[self.chapter] + f

    def say(self, spoken):
        if self.clock is not None and self.clock > spoken.frame:
            raise SystemExit(f"{spoken.cue}: simulated clock {self.clock} passed its timeline frame {spoken.frame}")
        self.clock = spoken.frame
        self.line = spoken
        self.anchors.append({"kind": "say", "cue": spoken.cue, "frame": spoken.frame, "global": self.g(spoken.frame)})

    def start_at_phrase(self, spoken, phrase):
        """Start a modelled chain at a phrase, assuming the scene is idle there (validated by anchors)."""
        self.line = spoken
        t, kind = spoken.at(phrase)
        idle = spoken.frame + frames_covering(t - spoken.start)
        self.clock = idle
        self.anchors.append({"kind": "assumed-idle", "cue": spoken.cue, "phrase": phrase, "source": kind,
                             "phrase_seconds_in_line": round(t - spoken.start, 6), "frame": idle,
                             "global": self.g(idle)})

    def _wait_until(self, t, what, kind):
        now = self.clock / FPS
        idle = self.clock + frames_covering(t - now) if t > now else self.clock
        late = 0
        if t <= now:
            want = self.line.frame + frames_covering(t - self.line.start)
            late = self.clock - want
        self.anchors.append({"kind": what, "cue": self.line.cue, "source": kind,
                             "phrase_seconds_in_line": round(t - self.line.start, 6), "frame": idle,
                             "global": self.g(idle), "slack_frames": idle - self.clock, "late_frames": late})
        self.clock = idle
        return self.anchors[-1]

    def wait_for(self, phrase):
        t, kind = self.line.at(phrase)
        a = self._wait_until(t, "wait_for", kind)
        a["phrase"] = phrase

    def wait_sentence(self, j):
        a = self._wait_until(self.line.sentence(j), "wait_sentence", "report-onset")
        a["sentence"] = j

    def wait_end(self):
        self._wait_until(self.line.end(), "wait_until(line.end())", "report-total")

    def play(self, rt, what, **extra):
        n = play_frames(rt)
        f = self.clock
        ev = {"what": what, "rt": rt, "frames": n, "clock_start": f, "video_local": [f, f + n - 1],
              "video_global": [self.g(f), self.g(f + n - 1)]}
        ev.update(extra)
        self.events.append(ev)
        self.clock = f + n
        return ev

    def pause(self, seconds, what="pause"):
        n = frames_covering(seconds)
        self.events.append({"what": what, "seconds": seconds, "frames": n, "clock_start": self.clock})
        self.clock += n

    def until_play(self, phrase, floor, cap, what, **extra):
        now = self.clock / FPS
        t, kind = self.line.at(phrase)
        rt = max(t - now, floor)
        if cap is not None:
            rt = min(rt, cap)
        ev = self.play(rt, what, **extra)
        ev["rt_from"] = {"phrase": phrase, "source": kind, "floor": floor, "cap": cap}
        return ev

    def breath(self, seconds):
        self._wait_until(self.line.end() + BREATH, "hold_breath", "report-total+BREATH")
        self.anchors[-1]["breath_frame"] = self.clock
        self.pause(seconds, f"music_breath {seconds}")

    def hold(self):
        self._wait_until(self.line.end() + BREATH, "kit.hold (BREATH bound; Thought gaps may be shorter)",
                         "report-total+BREATH")


def tween_frame(ev, r0, r1, r):
    """First video frame (local) whose tween value has reached r (moving from r0 towards r1)."""
    n, rt, f = ev["frames"], ev["rt"], ev["clock_start"]
    for k in range(1, n + 1):
        alpha = min(k / (FPS * rt), 1.0)
        value = r0 + (r1 - r0) * alpha
        if (r1 >= r0 and value >= r - 1e-12) or (r1 < r0 and value <= r + 1e-12):
            return f + k - 1
    return None


def main():
    out_path = sys.argv[1]
    script_text = {}
    for line in open(SCRIPT, encoding="utf-8"):
        if line.startswith("#") or "\t" not in line:
            continue
        cue, text = line.rstrip("\n").split("\t", 1)
        script_text[cue] = text
    timings = json.load(open(TIMINGS, encoding="utf-8"))["cues"]
    rep = json.load(open(REPORT, encoding="utf-8"))
    rate = rep["meta"]["rate"]
    report = {}
    for cue, rows in rep["lines"].items():
        t, starts = 0, []
        for r in rows:
            starts.append(t / rate)
            t += r["out_samples"] + r["gap_after_samples"]
        report[cue] = (starts, t / rate)
    tl = {ch: timeline(ch) for ch in TIMELINES}
    checks = {"timelines": {ch: {k: v for k, v in d.items() if k not in ("lines", "marks", "breaths")} for ch, d in tl.items()}}
    spoken = {}
    text_checks = {}
    for cue, ch in [("01_6", "01_the_bump"), ("08_5", "08_loop_sums"), ("09_1", "09_boundary"),
                    ("09_7", "09_boundary"), ("09_8", "09_boundary"), ("09_9", "09_boundary")]:
        text = script_text[cue]
        entry = timings[cue]
        th = hashlib.sha256(text.encode("utf-8")).hexdigest()
        marks = {int(m["byte"]): float(m["seconds"]) for m in entry["marks"]}
        text_checks[cue] = {"text_sha256": th, "matches_phrase_timings": th == entry["text_sha256"],
                            "wav_sha256": entry["wav_sha256"], "marks": len(marks)}
        spoken[cue] = Spoken(cue, tl[ch]["lines"][cue], text, report, marks)
        text_checks[cue].update({"line_frame": spoken[cue].frame, "len_report": spoken[cue].len,
                                 "measured_sentence_starts": spoken[cue].measured,
                                 "sentences": len(spoken[cue].sentences)})
    checks["cues"] = text_checks
    sims = []

    # ---- 01_6: arrow candidate03 (the_bump.rs lines 324-349)
    s = Sim("01_the_bump", "01_6 previews and chapter close")
    s.say(spoken["01_6"])
    s.wait_sentence(2)
    s.play(0.5, "fade_in chunk 1 [COMING_UP caption, z^2 portrait, loop circle]")
    s.pause(0.6)
    s.play(0.5, "fade_in chunk 2 [ARROW (buff 0.0, preview_arrow), half-disk path, zero_mark 0]", finding="arrow")
    s.pause(0.6)
    s.play(0.5, "fade_in chunk 3 [zero_mark 1]")
    s.pause(0.6)
    s.wait_sentence(3)
    s.wait_end()
    s.breath(2.0)
    s.play(0.6, "fade_out all (axes, curve, vertical, ring, radius, title, formula, poles, thumbs incl. arrow)",
           finding="arrow")
    checks["01_end_clock_equals_frames"] = s.clock == CHAPTER_FRAMES["01_the_bump"]
    checks["01_breath_frame_equals_timeline"] = [s.anchors[-1].get("breath_frame"), tl["01_the_bump"]["breaths"]]
    sims.append(s)

    # ---- 08_5: square candidate02 (loop_sums.rs 518-859), chain started at "Keep that square"
    L = spoken["08_5"]
    s = Sim("08_loop_sums", "08_5 nest, c, delta circle, magnified square, walk, quarter")
    s.start_at_phrase(L, "Keep that square, and cut it into four again")
    s.play(0.3, "fade_out [selected, identity, triangle_ineq, assumption]")
    s.play(0.3, "fade_out subdivision")
    for it, idx in enumerate([0, 3, 1, 2]):
        s.play(0.3, f"nest cut {it + 1}: fade_in grid")
        s.play(0.35, f"nest cut {it + 1}: fade_in [fill, edge] (quarter {idx})")
        s.play(0.2, f"nest cut {it + 1}: fade_out grid")
        s.play(0.2, f"nest cut {it + 1}: fade_out fill")
    s.wait_for("These nested squares shrink toward a single point")
    s.play(0.35, "fade_in [point, point_leader, point_name] (L5 c, leader, junction)", finding="L5")
    s.play(0.3, "fade_in nested_name Q_0 > Q_1 > ...")
    s.wait_for("the derivative at that one point only")
    s.play(0.3, "fade_in one_point text")
    s.wait_for("Near c, the function is a constant")
    s.play(0.3, "fade_out [hypothesis, nested_name, selection, one_point, title]")
    s.play(0.4, "fade_in expansion")
    s.wait_for("The constant integrates to zero")
    s.play(0.25, "fade_out affine (constant term)")
    s.play(0.3, "fade_in [equation, caption] (constant)")
    s.anchors.append({"kind": "unmodelled", "what": "lay_down(constant arrows, 0.3 each): arrow count not computed"})
    s.clock = None
    sims.append(s)

    s = Sim("08_loop_sums", "08_5 linear term to delta circle (chain restarted at phrase)")
    s.start_at_phrase(L, "The linear term integrates to zero too")
    s.play(0.25, "fade_out affine (constant chain incl. caption)", finding="L7-anchor")
    s.play(0.3, "fade_in [equation, caption] (linear)", finding="L7-anchor")
    s.anchors.append({"kind": "unmodelled", "what": "lay_down(linear arrows, 0.3 each) + zero dot 0.25"})
    sims.append(s)

    s = Sim("08_loop_sums", "08_5 remainder, epsilon, delta circle, magnified square, walk, quarter, triangle")
    s.start_at_phrase(L, "So only the remainder contributes")
    s.play(0.3, "fade_out affine")
    s.play(0.35, "fade_in remainder")
    s.wait_for("Pick any tiny factor, and call it epsilon")
    s.play(0.3, "fade_in epsilon text")
    s.wait_for("Close enough to c")
    s.play(0.4, "fade_in [error, local, neighbourhood delta-circle] (delta-circle meets c/leader)", finding="L5")
    s.wait_for("After enough cuts, the whole nested square")
    s.play(0.4, "fade_in small (nested square fill)", finding="L5")
    s.play(0.25, "fade_out nested squares", finding="L5")
    s.play(0.25, "fade_out [point, point_leader, point_name, neighbourhood, small]", finding="L5")
    s.play(0.4, "fade_in [inset, centre_dot, centre_name, zoom_name] (magnified square)")
    s.wait_for("the distance to c is at most the square's diagonal")
    s.play(0.4, "fade_in [diagonal, diagonal_name D_n, distance |z-c|, walker] (WALK_START corner)", finding="L11")
    s.wait_for("the steps add up to the perimeter")
    walk = s.play(2.0, "walk tween progress 0->4 (2.0 s) with c_aside/d_aside fade_out 0.3", finding="L11")
    walk["aside_fade_video_local"] = [walk["clock_start"], walk["clock_start"] + play_frames(0.3) - 1]
    walk["corner_video_local"] = [tween_frame(walk, 0.0, 4.0, float(k)) for k in (1, 2, 3, 4)]
    s.play(0.3, "fade_in [perimeter_word, perimeter_name P_n, centre_name, diagonal_name] (F5 baseline)",
           finding="L11")
    s.wait_for("size at most epsilon, times its diagonal, times its perimeter")
    s.play(0.35, "fade_in bound |I_n| <= eps D_n P_n")
    s.wait_for("Each cut halves the diagonal and halves the perimeter")
    s.play(0.3, "fade_out [expansion, remainder, epsilon, error, local, distance, walker]")
    s.play(0.35, "fade_in [product_title, scale_d, scale_p]")
    s.play(0.5, "fade_in quarter_picture (cuts, kept fill/edge/diagonal, c dot + c redrawn, KEPT_TEX)",
           finding="L12")
    s.wait_for("this bound shrinks to a quarter every time")
    s.play(0.35, "fade_in product D_nP_n = D_0P_0/4^n")
    s.wait_for("But our rule kept at least a quarter")
    s.play(0.3, "fade_in lower |I_{n+1}| >= |I_n|/4")
    s.wait_for("So after n cuts")
    s.play(0.3, "fade_out [bound, scale_d, scale_p, product, lower]")
    s.play(0.4, "fade_in accumulated")
    s.wait_for("The small square's integral has size at most epsilon")
    s.play(0.4, "fade_in upper")
    s.wait_for("Multiplying by four to the n cancels the fours")
    s.play(0.3, "fade_out [accumulated, upper]")
    s.play(0.4, "fade_in [lhs, four_top, numerator, bar, four_bottom]")
    s.play(0.5, "fade_in [cancel_top, cancel_bottom]")
    s.wait_for("the original integral's size is at most epsilon times")
    s.play(0.3, "fade_out (cancellation group)")
    s.play(0.4, "fade_in [final_bound, arbitrary]")
    s.wait_for("Since epsilon can be as small as we like")
    s.play(0.6, "fade_in zero")
    s.wait_for("The same shrinking argument works for a triangle")
    s.play(0.4, "fade_out square_picture (incl. quarter_picture, redrawn c)", finding="L12")
    s.play(0.4, "fade_in [triangle_outline, triangle_title]")
    s.wait_for("Join the midpoints of its sides")
    sims.append(s)

    # ---- 09_1: boundary candidate03 (boundary.rs 792-931), chain started at "Now keep the small circle fixed"
    L = spoken["09_1"]
    s = Sim("09_boundary", "09_1 refinement, arc/sum bounds, circles equal, shrink, zero, rest")
    s.start_at_phrase(L, "Now keep the small circle fixed")
    s.play(0.3, "fade_in fixed")
    for what in ("outer_arrows", "inner_ccw", "polygon_names", "both"):
        s.play(0.2, f"fade_out {what}")
    s.wait_for("give the polygons more and more sides")
    for power in (4, 5, 6):
        s.play(0.15, f"fade_out polygons (sides 2^{power - 1})")
        s.play(0.25, f"fade_in polygons 2^{power}")
        s.pause(0.25)
    s.wait_for("Each short chord and its arc share their endpoints")
    s.play(0.3, "fade_in inset card")
    s.play(0.3, "fade_in piece")
    s.wait_for("so a constant has the same integral along both")
    s.play(0.2, "fade_out polygons_equal")
    s.play(0.3, "fade_in CONSTANT_TEX")
    s.wait_for("The difference comes only from how much the quotient wobbles")
    s.play(0.2, "fade_out CONSTANT_TEX")
    s.play(0.3, "fade_in ARC_TEX (M1 closing bar dz\\,\\Big|)", finding="M1")
    s.play(0.3, "fade_in WOBBLE lines")
    s.wait_for("Continuity makes every wobble small at once")
    s.play(0.2, "fade_out ARC_TEX", finding="M1")
    s.play(0.3, "fade_in SUM_TEX (M1 closing bar; L5 width cap)", finding="M1")
    s.wait_for("So in the limit, the big circle's integral equals the small circle's")
    s.play(0.3, "fade_out [SUM_TEX, outer_poly, inner_poly, piece]", finding="M1")
    s.play(0.3, "fade_out inset")
    s.play(0.3, "fade_in CIRCLES_TEX")
    s.play(0.3, "fade_in circle_arrows")
    s.play(0.3, "fade_in small_arrows")
    s.wait_for("That holds for every small radius")
    s.play(0.3, "fade_in EVERY_TEX")
    s.wait_for("Only now do we shrink the small circle")
    s.play(0.2, "fade_out [fixed, every]", finding="M2")
    s.play(0.2, "fade_out leaving [small_arrows, ca (a at A_AT), rho (at RHO_AT)]", finding="M2")
    s.play(0.3, "fade_in [bar, bar_name, ca_out, rho_out, leader] (riding names at SMALL_R, leader 0.6)",
           finding="M2/M3/leader")
    t1 = s.play(0.7, "tween radius SMALL_R 0.18 -> MID_R 0.12 (updater: circle, bar, a, rho, leader)",
                finding="M2/M3/leader")
    s.wait_for("The magnitude of its integral is at most")
    s.play(0.2, "fade_out CIRCLES_TEX")
    s.play(0.3, "fade_in SHRINK_TEX (M1 closing bar)", finding="M1")
    s.wait_for("and that goes to zero")
    t2 = s.play(1.0, "tween radius MID_R 0.12 -> SHRUNK_R 0.06 (M3 crossing in base; L14 rest gap)",
                finding="M3/L14/leader")
    s.wait_for("But the big circle's integral never changed")
    s.play(0.2, "fade_out SHRINK_TEX")
    s.play(0.3, "fade_in ZERO_TEX")
    s.wait_for("The second piece contributes nothing")
    s.play(0.4, "fade_out proof_gone (outer, small, hole, ca_out, rho_out, leader, ...)", finding="leader")
    s.play(0.4, "fade_in [rest, result]")
    s.wait_end()
    checks["09_1_clock_at_end_vs_09_2_frame"] = [s.clock, tl["09_boundary"]["lines"]["09_2"]]
    sims.append(s)
    radius_marks = {}
    for name, ev, r0, r1, rs in (("shrink1", t1, 0.18, 0.12, [0.17, 0.16, 0.15, 0.14, 0.13, 0.12]),
                                 ("shrink2", t2, 0.12, 0.06, [0.11, 0.10, 0.09, 0.08, 0.07, 0.06])):
        radius_marks[name] = {f"{r:.2f}": tween_frame(ev, r0, r1, r) for r in rs}
    checks["09_1_radius_frames_local"] = radius_marks

    # ---- 09_7 growth (boundary.rs 1239-1361)
    L = spoken["09_7"]
    s = Sim("09_boundary", "09_7 largest disk grows through a; poles; no further")
    s.say(L)
    s.play(0.4, "fade_out gone (09_6 picture)")
    s.play(0.5, "fade_in axes_g (graph)")
    s.play(0.5, "fade_in recipe0 + name")
    s.wait_for("the largest disk")
    s.play(0.1, "fade_in unit circle r=0.001 (0.1 s = 7 frames, BN-02)", finding="M4")
    g7 = s.until_play("That answers", 1.6, 3.0, "tween disk 0.001 -> 1.0 (a_name_shown: hidden r 0.11-0.43)",
                      finding="M4/L2")
    s.wait_sentence(2)
    s.play(0.4, "fade_in pole names")
    s.wait_sentence(3)
    s.play(0.5, "fade_in [reach0, radius_name, glow0]")
    s.wait_for("no further")
    s.play(0.3, "fade_in stops")
    s.hold()
    sims.append(s)
    checks["09_7_hold_vs_09_8_frame"] = [s.clock, tl["09_boundary"]["lines"]["09_8"]]
    a7 = {f"{r:.2f}": tween_frame(g7, 0.001, 1.0, r) for r in (0.01, 0.11, 0.125, 0.413, 0.43, 0.53, 1.0)}

    # ---- 09_8 slide and "1" card
    L = spoken["09_8"]
    s = Sim("09_boundary", "09_8 recentre slide, 1 card, predict")
    s.say(L)
    s.play(0.3, "fade_out gone (09_7 disk etc.)")
    s.play(1.8, "tween slide 0 -> 1 (centre and a's name move with it)", finding="09_8")
    s.play(0.2, "fade_in one_name '1' card", finding="09_9-card")
    s.wait_sentence(len(L.sentences) - 1)
    s.play(0.4, "predict: fade_in question card")
    s.hold()
    checks["09_8_predict_mark_vs_timeline"] = [s.clock, tl["09_boundary"]["marks"]]
    sims.append(s)

    # ---- 09_9 growth through a and over the "1" card
    L = spoken["09_9"]
    s = Sim("09_boundary", "09_9 disk grows to sqrt 2 through a and the re-rooted 1 card")
    s.say(L)
    s.play(0.3, "fade_out asked (predict card)")
    s.play(0.3, "fade_in [unit r=0.001, reach1, reach_name]; then one_name re-rooted above the disk", finding="L3")
    g9 = s.until_play("So the new recipe", 1.6, None, "tween grow 0.001 -> sqrt 2 (a hidden r 0.11-0.43; card)",
                      finding="M4/L3")
    s.play(0.6, "fade_in recipe1 + name")
    s.play(0.5, "fade_in [glow1, works]")
    s.wait_sentence(2)
    sims.append(s)
    a9 = {f"{r:.3f}": tween_frame(g9, 0.001, math.sqrt(2.0), r)
          for r in (0.01, 0.11, 0.125, 0.38, 0.413, 0.43, 0.53, 0.62, 0.91, math.sqrt(2.0))}
    checks["09_7_radius_frames_local"] = a7
    checks["09_9_radius_frames_local"] = a9

    doc = {"inputs": {"script": [SCRIPT, sha(SCRIPT)], "phrase_timings": [TIMINGS, sha(TIMINGS)],
                      "report": [REPORT, sha(REPORT)]},
           "chapter_start": CHAPTER_START, "checks": checks,
           "sims": [{"chapter": x.chapter, "label": x.label, "anchors": x.anchors, "events": x.events} for x in sims]}
    with open(out_path, "x", encoding="utf-8") as fh:
        json.dump(doc, fh, indent=1)
    print(json.dumps(checks, indent=1)[:6000])
    for x in sims:
        print(f"\n## {x.chapter}: {x.label}")
        for a in x.anchors:
            print("  A", {k: a[k] for k in a if k in ("kind", "phrase", "sentence", "source", "frame", "global",
                                                      "slack_frames", "late_frames", "what")})
        for e in x.events:
            if "video_local" in e:
                print(f"  E {e['video_local']} g{e['video_global']} n{e['frames']} {e['what'][:90]}")


main()
