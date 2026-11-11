"""Build the combined-repair native sampling plan from frame-windows01.json (every frame below is looked up from the
computed events, not retyped). Writes sampling-plan02.json and per-chapter frame lists into OUT_DIR (refuses to
overwrite). Read-only otherwise.
usage: python3 -I -B make-plan02.py FRAME_WINDOWS.json OUT_DIR"""
import datetime
import hashlib
import json
import os
import sys

src_path, out_dir = sys.argv[1], sys.argv[2]
raw = open(src_path, "rb").read()
fw = json.loads(raw)
START = fw["chapter_start"]
FRAMES = {"01_the_bump": 6311, "08_loop_sums": 14959, "09_boundary": 16504}
P17 = ("docs/video-program/third-wave-20261024/opus-replacements-20261104/fmd-render-readiness01/raster-quality03/"
       "production-repair17/")
BUNDLE = P17 + "final-review995/combined-repair-bundle02-root-frozen01/"


def ev(chapter, what):
    hits = [e for s in fw["sims"] if s["chapter"] == chapter for e in s["events"]
            if "video_local" in e and what in e["what"]]
    if len(hits) != 1:
        raise SystemExit(f"{chapter}: {len(hits)} events match {what!r}")
    return hits[0]


def anchor(chapter, phrase):
    hits = [a for s in fw["sims"] if s["chapter"] == chapter for a in s["anchors"] if a.get("phrase") == phrase]
    return hits[0]


def span(a, b, step=1):
    out = list(range(a, b + 1, step))
    if out[-1] != b:
        out.append(b)
    return out


R = fw["checks"]
windows = []


def win(wid, chapter, finding, kind, required, extended, expect, basis):
    req = sorted(set(required))
    ext = sorted(set(extended) - set(req))
    for f in req + ext:
        if not 0 <= f < FRAMES[chapter]:
            raise SystemExit(f"{wid}: frame {f} outside {chapter}")
    windows.append({"id": wid, "chapter": chapter, "finding": finding, "kind": kind,
                    "local_range": [min(req + ext), max(req + ext)],
                    "global_range": [START[chapter] + min(req + ext), START[chapter] + max(req + ext)],
                    "required_local": req, "extended_local": ext, "expect": expect, "basis": basis})


# ------------------------------------------------------------------ chapter 01: arrow (candidate03 90b51c2b)
C = "01_the_bump"
a_in = ev(C, "fade_in chunk 2")
a_out = ev(C, "fade_out all")
c1 = ev(C, "fade_in chunk 1")
win("01-control-unchanged", C, "control", "control-expected-identical",
    [36, 1066, 3440, c1["video_local"][0] - 1, c1["video_local"][1], a_in["video_local"][0] - 1], [],
    "candidate03 changes only the 01_6 preview arrow (first drawn at the chunk-2 fade). Each frame should equal "
    "master993's decoded frame at the same index; a mismatch is engine/environment drift to explain before any "
    "repair verdict, not a finding about the arrow.",
    "first visible change = chunk-2 fade start")
lo, hi = a_in["video_local"]
win("01-arrow-fade-in", C, "preview-arrow (final995 sheet 23/24 collapse)", "dense-fade",
    span(lo - 2, hi + 4), [],
    "Arrow from the loop's centre (zero) to the loop at arg 0.9, head point on the loop, shaft visible at every "
    "alpha; no loose head detached from a vanished shaft (base build268). Half-disk path and the first zero mark "
    "fade in with it.",
    f"fade_in chunk 2 [arrow, half, zm0] 0.5 s = {a_in['frames']} frames at video {a_in['video_local']}")
held = [hi + 7, ev(C, "fade_in chunk 3")["video_local"][1], 5696, 5760, 5911, 6000, 6119, 6155, 6215, 6269,
        a_out["video_local"][0] - 1]
win("01-arrow-held", C, "preview-arrow", "sparse-hold", held, [],
    "Held previews through sentence 3, the line end (6119), the music breath (6155-6274): the arrow keeps its "
    "full shaft and head; f6269 is the base still that showed the collapse (sheet 24 slot 7).",
    "timeline breath 6155 x 2 s; sentence 3 at 5911")
lo, hi = a_out["video_local"]
win("01-arrow-fade-out", C, "preview-arrow", "dense-fade", span(lo - 2, hi, 2) + [lo - 1, lo, hi], span(lo - 2, hi),
    "Everything fades out together over the chapter's last 36 frames; the arrow leaves with its loop (no orphan "
    "head). Frames 6273-6310 are also final995's bump-transition window.",
    f"fade_out all 0.6 s at video {a_out['video_local']} = chapter end {FRAMES[C] - 1}")

# ------------------------------------------------------------------ chapter 08: square (bundle02 a8d62f56)
C = "08_loop_sums"
l5 = ev(C, "(L5 c, leader, junction)")
win("08-control-unchanged", C, "control", "control-expected-identical",
    [3748, 6360, 6506, ev(C, "nest cut 4: fade_out fill")["video_local"][1], l5["video_local"][0] - 1], [],
    "No candidate02/bundle02 change is drawn before the L5 fade; each frame should equal master993's.",
    "first visible change = L5 fade start")
lo, hi = l5["video_local"]
win("08-L5-fade-in", C, "square L5 (c name, leader, rim junction)", "dense-fade",
    [lo - 2, lo - 1, lo, lo + 1] + span(lo + 2, hi, 2) + [hi, hi + 1, hi + 2, hi + 4], span(lo - 2, hi + 4),
    "c at 0.57 from its dot in Q_2's open lower right; the thin dim leader starts at the dot's rim (review L1: the "
    "rim junction is thin, LEADER_START 0.072 vs dot 0.07) and crosses Q_5's corner and the Q_3/Q_4 shared edge "
    "on purpose; no stroke across the c glyph.",
    f"fade_in [point, point_leader, point_name] 0.35 s = {l5['frames']} frames at video {l5['video_local']}; "
    f"phrase 'These nested squares shrink toward a single point' idle frame "
    f"{anchor(C, 'These nested squares shrink toward a single point')['frame']}")
d = ev(C, "delta-circle")
hold5 = [hi + 18, 7100, ev(C, "fade_in one_point")["video_local"][1], ev(C, "fade_in expansion")["video_local"][1],
         8000, ev(C, "fade_in remainder")["video_local"][1], ev(C, "fade_in epsilon")["video_local"][1],
         d["video_local"][0] - 2]
win("08-L5-held", C, "square L5", "sparse-hold", hold5, [],
    "c, its leader and the nest held while the right column changes.", "event ends from the 08_5 chain")
lo, hi = d["video_local"]
win("08-L5-delta-circle", C, "square L5 x delta-circle", "dense-fade",
    span(lo - 2, hi + 3, 2) + [hi, hi + 30, 9450], span(lo - 2, hi + 3),
    "The delta-circle (radius 1.8 x the current square's diameter about c) fades in with |r(z)| <= eps|z-c| and "
    "|z-c| < delta, then holds; c's name stays outside the circle as the candidate intends; leader/circle "
    "crossing legible.",
    f"fade_in [error, local, neighbourhood] 0.4 s = {d['frames']} frames at video {d['video_local']}")
small = ev(C, "fade_in small")
gone = ev(C, "fade_out [point, point_leader, point_name, neighbourhood, small]")
win("08-L5-fade-out", C, "square L5", "dense-fade",
    [small["video_local"][0] - 2] + span(small["video_local"][0], gone["video_local"][1] + 4, 2),
    span(small["video_local"][0] - 2, gone["video_local"][1] + 4),
    "small fill (0.4), nest fade-out (0.25), then point, leader, c, delta-circle and fill leave together in one "
    "0.25 s fade: no orphan leader or c.",
    f"video {small['video_local'][0]}..{gone['video_local'][1]}")
ins = ev(C, "magnified square")
win("08-inset", C, "square (c_name_at rename: binding only)", "sparse-fade",
    span(ins["video_local"][0], ins["video_local"][1], 6) + [ins["video_local"][1], 9900], [],
    "Magnified square with c's name at c_name_at (bundle02 renames the constant only; same position).",
    f"fade_in inset 0.4 s at video {ins['video_local']}")
dg = ev(C, "WALK_START corner")
win("08-L11-diagonal-in", C, "square L11", "dense-fade", span(dg["video_local"][0] - 2, dg["video_local"][1] + 2, 2),
    span(dg["video_local"][0] - 2, dg["video_local"][1] + 2),
    "Whole teal D_n diagonal with its name; |z-c| from the bottom-right WALK_START corner (half the other "
    "diagonal), so D_n no longer reads as c-to-corner.",
    f"fade_in [diagonal, D_n, distance, walker] 0.4 s at video {dg['video_local']}")
wk = ev(C, "walk tween")
corners = wk["corner_video_local"]
win("08-L11-walk", C, "square L11 (motion)", "dense-motion",
    span(wk["video_local"][0] - 2, wk["video_local"][1] + 2, 2) + corners + wk["aside_fade_video_local"],
    span(wk["video_local"][0] - 2, wk["video_local"][1] + 2),
    "Walker and |z-c| sweep all four sides counterclockwise in 2.0 s; c and D_n names leave in the first 0.3 s "
    f"(video {wk['aside_fade_video_local']}); corners reached at video {corners}; the diagonal stays whole.",
    f"play_prepared [walk 2.0, c_aside 0.3, d_aside 0.3] = {wk['frames']} frames at video {wk['video_local']}")
p5 = ev(C, "(F5 baseline)")
win("08-L11-perimeter-F5", C, "square L11/F5 (glyph baseline)", "dense-fade",
    span(p5["video_local"][0] - 2, p5["video_local"][1] + 3) + [10380, 10420, anchor(C, "size at most epsilon, times its diagonal, times its perimeter")["frame"] - 1],
    [],
    "'perimeter P_n' centred under the square: text word and TeX P_n share a visual baseline (F5), keep their "
    "gap, and c and D_n return beside their marks.",
    f"fade_in [perimeter_word, P_n, centre_name, diagonal_name] 0.3 s at video {p5['video_local']}")
q = ev(C, "fade_in quarter_picture")
win("08-L12-quarter-in", C, "square L12 (tint/doubling)", "dense-fade",
    span(q["video_local"][0] - 2, q["video_local"][1] + 2), [],
    "Bisection cuts plus one kept quarter sharing a corner with Q_n (fill, edge, its diagonal), and c's dot and "
    "name redrawn on top: c must not tint or double while its copy fades in over the original.",
    f"fade_in quarter_picture 0.5 s = {q['frames']} frames at video {q['video_local']}")
qh = [q["video_local"][1] + 30] + [ev(C, w)["video_local"][1] for w in
      ("fade_in product", "fade_in lower", "fade_in accumulated", "fade_in upper", "fade_in [cancel_top",
       "fade_in [final_bound", "fade_in zero")]
sp = ev(C, "fade_out square_picture")
win("08-L12-held", C, "square L12", "sparse-hold", qh + [sp["video_local"][0] - 1], [],
    "Quarter picture held while the right column builds the 4^n argument; c and the kept quarter unchanged.",
    "event ends from the 08_5 chain")
win("08-L12-fade-out", C, "square L12 (fade-out lag)", "dense-fade",
    span(sp["video_local"][0] - 2, ev(C, "triangle_outline")["video_local"][1] + 2, 2),
    span(sp["video_local"][0] - 2, ev(C, "triangle_outline")["video_local"][1] + 2),
    "The whole square picture, including the redrawn c and every quarter stroke, leaves in one 0.4 s fade; no "
    "residual c, cut or fill under the triangle that fades in next.",
    f"fade_out square_picture at video {sp['video_local']}, triangle in at "
    f"{ev(C, 'triangle_outline')['video_local']}")

# ------------------------------------------------------------------ chapter 09: boundary (candidate03 4b689791)
C = "09_boundary"
arc = ev(C, "fade_in ARC_TEX")
win("09-control-unchanged", C, "control", "control-expected-identical",
    [36, 4887, 5600, arc["video_local"][0] - 2], [],
    "No candidate03 change is drawn before ARC_TEX; each frame should equal master993's (native844).",
    "first visible change = ARC_TEX fade start")
co = ev(C, "fade_out CONSTANT_TEX")
win("09-M1-arc", C, "boundary M1 (ARC_TEX closing bar)", "dense-fade",
    span(co["video_local"][0] - 2, arc["video_local"][1] + 3, 2) + [arc["video_local"][1], 5760, 6000,
                                                                  anchor(C, "Continuity makes every wobble small at once")["frame"] - 1],
    span(co["video_local"][0] - 2, arc["video_local"][1] + 3),
    "'g dz \\, \\Big|': a visible gap between the z of the second dz and the closing bar, like the opening bar's gap.",
    f"fade_out CONSTANT 0.2 s at {co['video_local']}, fade_in ARC_TEX 0.3 s at {arc['video_local']}")
sm = ev(C, "fade_in SUM_TEX")
win("09-M1-sum", C, "boundary M1 + L5 (SUM_TEX)", "dense-fade",
    span(sm["video_local"][0] - 15, sm["video_local"][1] + 3, 2) + [sm["video_local"][1], 6250,
                                                                  ev(C, "fade_out [SUM_TEX")["video_local"][0] - 1],
    span(sm["video_local"][0] - 15, sm["video_local"][1] + 3),
    "Closing-bar gap in SUM_TEX; right-edge margin of the widest formula (L5: possible ~1% rescale under the "
    "5.1 width cap; measure the glyph height against ARC_TEX).",
    f"fade_in SUM_TEX 0.3 s at {sm['video_local']}")
f1, f2, f3 = (ev(C, "fade_out [fixed, every]"), ev(C, "fade_out leaving"), ev(C, "fade_in [bar, bar_name"))
t1 = ev(C, "SMALL_R 0.18 -> MID_R")
win("09-M2-names-leader-in", C, "boundary M2 + leader", "dense-fade",
    span(f1["video_local"][0] - 3, f2["video_local"][1], 3) + span(f2["video_local"][1] - 2, t1["video_local"][0] + 3),
    span(f1["video_local"][0] - 3, t1["video_local"][0] + 3),
    "Old a (A_AT) and rho (RHO_AT) leave with the small circle's heads; riding a/rho and the leader fade in with "
    "the bar; the leader reaches its own 0.6 opacity and draws above the small circle's line; no rho jump at the "
    f"tween's first frame {t1['video_local'][0]} (base jumped at f74158-74163).",
    f"fades at {f1['video_local']}, {f2['video_local']}, {f3['video_local']}")
r1 = R["09_1_radius_frames_local"]["shrink1"]
win("09-shrink1", C, "boundary M2/M3 + leader (motion)", "dense-motion",
    span(t1["video_local"][0], t1["video_local"][1] + 2, 2) + list(r1.values()),
    span(t1["video_local"][0], t1["video_local"][1] + 2),
    f"Radius 0.18 -> 0.12 in {t1['frames']} frames (r at video: {r1}); a above and rho right of the line at their "
    "ride distances; leader foot fixed 0.02 outside the hole ring, top under a.",
    f"tween_linear 0.7 s at video {t1['video_local']}")
mb = ev(C, "fade_in SHRINK_TEX")
win("09-hold-mid", C, "boundary leader (P1 radius look)", "sparse-hold",
    [t1["video_local"][1] + 12, 7180, mb["video_local"][0] - 14], [],
    "Held at MID_R: does the leader read as a pointer to the fixed hole, not as a radius (CC0 P1)?",
    "between tween 1 end and SHRINK_TEX")
win("09-M1-shrink", C, "boundary M1 (SHRINK_TEX)", "dense-fade",
    span(mb["video_local"][0] - 15, mb["video_local"][1] + 3, 2) + [mb["video_local"][1], 7300, 7450,
                                                                  anchor(C, "and that goes to zero")["frame"] - 1],
    span(mb["video_local"][0] - 15, mb["video_local"][1] + 3),
    "SHRINK_TEX closing-bar gap.", f"fade_in SHRINK_TEX 0.3 s at {mb['video_local']}")
t2 = ev(C, "MID_R 0.12 -> SHRUNK_R")
r2 = R["09_1_radius_frames_local"]["shrink2"]
win("09-shrink2-rest", C, "boundary M3 + L14 + leader (motion, rest)", "dense-motion",
    span(t2["video_local"][0] - 2, t2["video_local"][1] + 5), [],
    f"Radius 0.12 -> 0.06 in {t2['frames']} frames (r at video: {r2}); a never crossed (base crossed f74689-74717); "
    f"at rest (video {t2['video_local'][1]}): drawn gap to the hole ring (CC2 box model 7.6 px), a 12.05 px above "
    "A_AT, leader gaps 3.4 px to the ring and to a (CC2/CC0 box models), little leader inside the circle (CC2 "
    "2.3 px, CC0 4.3 px: measure).",
    f"tween_linear 1.0 s at video {t2['video_local']}")
z = ev(C, "fade_in ZERO_TEX")
pg = ev(C, "fade_out proof_gone")
win("09-rest-zero", C, "boundary L14 + leader (held rest)", "sparse-hold",
    [t2["video_local"][1] + 20] + span(z["video_local"][0] - 13, z["video_local"][1] + 2, 3) +
    [7900, pg["video_local"][0] - 1], [],
    "Rest state through the zero formula's swap.", f"fade_out SHRINK 0.2 + fade_in ZERO 0.3 ending {z['video_local'][1]}")
win("09-proof-gone", C, "boundary leader (fade-out)", "dense-fade",
    span(pg["video_local"][0] - 2, pg["video_local"][1] + 2, 2), span(pg["video_local"][0] - 2, pg["video_local"][1] + 2),
    "Leader, riding a/rho, circles and bar leave together in one 0.4 s fade (no orphan leader).",
    f"fade_out proof_gone 0.4 s at {pg['video_local']}")
u7 = ev(C, "fade_in unit circle r=0.001")
g7 = ev(C, "tween disk 0.001 -> 1.0")
a7 = R["09_7_radius_frames_local"]
win("09_7-growth", C, "boundary M4 + L2 (motion)", "dense-motion",
    span(u7["video_local"][0] - 2, a7["0.53"] + 8, 2) + list(a7.values()) +
    span(a7["0.53"] + 8, g7["video_local"][1] + 1, 8) + [g7["video_local"][1], g7["video_local"][1] + 1],
    span(u7["video_local"][0] - 2, g7["video_local"][1] + 1),
    f"Disk from r 0.001 to 1 in {g7['frames']} frames (rt capped at 3.0); a's name eases out over r 0.01->0.11 "
    f"(video {a7['0.01']}..{a7['0.11']}), hidden while the line is near (CC2/CC0: within CLEAR only r 0.125-0.413, "
    f"video {a7['0.12']}..{a7['0.41']}), back by r 0.53 (video {a7['0.53']}); the line never draws over a visible a.",
    f"fade_in unit 0.1 s = {u7['frames']} frames (BN-02) at {u7['video_local']}; tween at {g7['video_local']}")
win("09_7-after-control", C, "boundary M4 (a's opacity restored)", "control-expected-identical",
    [g7["video_local"][1] + 64, 14100, 14400, ev(C, "fade_in stops")["video_local"][1] + 1, 14814], [],
    "a_name_shown(1.0) = 1, so after the growth a's name should be pixel-identical to master993's; a bolder or "
    "dimmer a here means set_opacity changed more than the fill (risk named in the fmn a399d626 notes).",
    "after the 09_7 growth, before 09_8")
sl = ev(C, "tween slide")
one = ev(C, "fade_in one_name")
win("09_8-slide-card", C, "boundary 09_8 (control) + 1 card", "control-expected-identical",
    span(sl["video_local"][0] - 18, sl["video_local"][1] + 1, 6) + span(one["video_local"][0] - 2, one["video_local"][1] + 2, 2)
    + [15022, 15202, 15576], [],
    "The recentre slide carries the centre and a's name; the '1' card fades in. Source unchanged here: expect "
    "frames identical to master993 (a's opacity state is the only candidate influence).",
    f"slide 1.8 s = {sl['frames']} frames (BN-02) at {sl['video_local']}; card at {one['video_local']}")
g9 = ev(C, "tween grow 0.001 -> sqrt 2")
a9 = R["09_9_radius_frames_local"]
fa = ev(C, "fade_out asked")
win("09_9-growth-card", C, "boundary 09_9 card + M4 + L3 (motion)", "dense-motion",
    span(fa["video_local"][0] - 2, g9["video_local"][0], 3) + span(g9["video_local"][0], a9["0.910"] + 4, 2) +
    span(a9["0.910"] + 4, g9["video_local"][1] + 1, 4) + list(a9.values()) +
    [ev(C, "fade_in recipe1")["video_local"][1], ev(C, "fade_in [glow1, works]")["video_local"][1], 16123],
    span(fa["video_local"][0] - 2, g9["video_local"][1] + 1),
    f"Disk from 0.001 to sqrt 2 in {g9['frames']} frames; a hidden r 0.11-0.43 (video {a9['0.110']}..{a9['0.430']}), "
    f"back by 0.53 (video {a9['0.530']}); the line passes under the re-rooted '1' card for r about 0.38-0.91 "
    f"(video {a9['0.380']}..{a9['0.910']}; card extent estimated, L3) and ends under the sqrt 2 card.",
    f"fades at {fa['video_local']} and {ev(C, 'fade_in [unit r=0.001')['video_local']}; tween at {g9['video_local']}")

# ------------------------------------------------------------------ samples
samples = {}
for w in windows:
    for tier, frames in (("required", w["required_local"]), ("extended", w["extended_local"])):
        for f in frames:
            key = (w["chapter"], f)
            s = samples.setdefault(key, {"chapter": w["chapter"], "local_frame": f, "frame": START[w["chapter"]] + f,
                                         "seconds": round((START[w["chapter"]] + f) / 60, 6), "tier": tier,
                                         "windows": []})
            if tier == "required":
                s["tier"] = "required"
            s["windows"].append(w["id"])
order = sorted(samples.values(), key=lambda s: s["frame"])
counts = {}
for s in order:
    c = counts.setdefault(s["chapter"], {"required": 0, "extended": 0})
    c[s["tier"]] += 1
req_total = sum(c["required"] for c in counts.values())
all_total = len(order)

plan = {
    "version": "combined-repair-native-sampling-plan02",
    "author": "CC1 (worker; plan only)",
    "time": datetime.datetime.now(datetime.timezone.utc).isoformat(),
    "scope": ("Sampling specification for the guarded native 4K render of bundle02 (arrow 01, square 08, boundary "
              "09). Frames are predicted from source, the selected timelines and the narration timing files; "
              "nothing has been built, rendered or viewed for these candidates. No acceptance of any kind."),
    "master_reference": {"sha256": "80244e88ca5970d0da07a5d96d6d825ae1851bae703d75bd200f8810b04e7b90",
                         "frames": 133662, "fps": 60, "final995_overview_sampling_sha256":
                         "c211e7d571a9ddd67f655c9b5070ff568555723b4e30f92891ee224b4b64626f"},
    "sources": {
        "bundle": BUNDLE,
        "bundle_manifest_sha256": "9c29672e24e7a64eca02ebede268a4b93a757f30fc42d0c8dc476c74817ad4b9",
        "bundle_receipt_sha256": "2d9fbc03e73e4ee984e9c03f6ae20ecc5682bbcceee0ba0d9390c83dc383ccfc",
        "staged": {
            "src/chapters/the_bump.rs": "90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7",
            "src/chapters/loop_sums.rs": "a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef",
            "src/chapters/mod.rs": "4de1e4a56fde5c11ac3d6a65e772d09ec33db4f5757d817aff370799d3423578",
            "src/chapters/boundary.rs": "4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932"},
        "bases": {
            "src/chapters/the_bump.rs": "8e93012e5b7598b9a99ffcdb07e436341f052d1175455f5581e108ee86e6ddb3",
            "src/chapters/loop_sums.rs": "8a13e8a190cd502e7b58efbcb7c40872cdb0e6e702feb69e1b789e451a7a42b5",
            "src/chapters/mod.rs": "e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd",
            "src/chapters/boundary.rs": "cd1900aadd46090083729d9aa768e59e1551c42ab94e5bd055f520cb4eff5e6e"},
        "timing_binding_note": ("08 windows were computed on reviewed candidate02 977c77df; a8d62f56 differs only by "
                                "the five centre_name_at -> c_name_at substitutions (diff: lines 656-657, 773, "
                                "1755-1756), none a timing call, so every 08 frame binds to a8d62f56 unchanged."),
    },
    "preconditions": [
        {"id": "P1-sources", "check": "the build's src/chapters/{the_bump,loop_sums,mod,boundary}.rs hash to sources.staged",
         "on_fail": "stop: the plan does not describe that build"},
        {"id": "P2-timelines", "check": ("each rendered chapter timeline's frames/fps/line/mark/breath/listen rows equal the "
                                          "selected timeline's (line WAV directory aside); with the selected narration "
                                          "directory substituted its sha256 equals chapters[].selected_timeline_sha256"),
         "on_fail": "stop: frame windows are void (narration placement moved)"},
        {"id": "P3-frames", "check": "decoded chapter frame counts 6311 / 14959 / 16504 at 3840x2160, 60 fps",
         "on_fail": "stop"},
        {"id": "P4-narration", "check": ("narration unchanged: ElevenLabs Frederick Surrey takes; phrase-timings b025eaa7 "
                                         "and the report sentence schedule as in derivation.inputs"),
         "on_fail": "stop"},
        {"id": "P5-controls", "check": ("control-expected-identical samples equal master993's frames at the same index "
                                        "(compare decoded pixels or framehashes)"),
         "on_fail": "explain the drift (engine, fonts, environment) before reading any repair window"},
    ],
    "chapters": [
        {"chapter": "01_the_bump", "master_start_frame": START["01_the_bump"], "frames": FRAMES["01_the_bump"],
         "selected_timeline_sha256": "150147fe5bc7cdb0a111061c4530a16b104d196e004ae29bcf210d1c9c32ac81",
         "narration_dir_in_selected": "/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration",
         "source": "src/chapters/the_bump.rs 90b51c2b", "master993_kind": "combined (binary 983954c5)"},
        {"chapter": "08_loop_sums", "master_start_frame": START["08_loop_sums"], "frames": FRAMES["08_loop_sums"],
         "selected_timeline_sha256": "43c8bab2f7ca61c8692e0f1686798d54e187dcfeeab0d03311c2d4c9a5f54812",
         "narration_dir_in_selected": "/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration",
         "source": "src/chapters/loop_sums.rs a8d62f56 (+ mod.rs 4de1e4a5)", "master993_kind": "combined (binary 983954c5)"},
        {"chapter": "09_boundary", "master_start_frame": START["09_boundary"], "frames": FRAMES["09_boundary"],
         "selected_timeline_sha256": "5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e",
         "narration_dir_in_selected": "/data/projects/jmev-stroke-probe-20261106/jobs/csd-review-repair-rebind313/narration",
         "source": "src/chapters/boundary.rs 4b689791", "master993_kind": "native844 (binary e7525873)"},
    ],
    "frame_convention": ("local_frame = decoded frame index in the chapter's own 4K movie (0-based); frame = "
                         "master_start_frame + local_frame, the master993-equivalent index if chapter lengths stay "
                         "equal; an event starting at clock frame F with n frames shows in local frames F..F+n-1."),
    "windows": windows,
    "samples": order,
    "counts": {"by_chapter": counts, "required": req_total, "required_plus_extended": all_total,
               "sheets_at_8_tiles": {"required": -(-req_total // 8), "all": -(-all_total // 8)}},
    "sheets": {"tiles_per_sheet": 8, "order": "samples in frame order, required tier first pass",
               "label": "chapter local_frame (master-equivalent frame) | window ids",
               "crops": ("4K crops at native pixels for: 01 arrow head/shaft; 08 c/leader/dot junction, delta-circle "
                         "vs c, P_n baseline, c redraw; 09 closing bars, leader ends, rest gap, a vs line, 1 card")},
    "executable": {
        "checker": "sampling-plan-check01.py",
        "checker_sha256": hashlib.sha256(open(os.path.join(os.path.dirname(os.path.abspath(__file__)),
                                                           "sampling-plan-check01.py"), "rb").read()).hexdigest(),
        "steps": [
            "python3 -I -B sampling-plan-check01.py sources sampling-plan02.json BUILD_CRATE_ROOT   # P1, before the build",
            "python3 -I -B sampling-plan-check01.py timeline sampling-plan02.json CHAPTER RENDERED.timeline.tsv   # P2, per chapter",
            "python3 -I -B sampling-plan-check01.py expect sampling-plan02.json   # P3 frame counts to compare with the decode",
            "python3 -I -B sampling-plan-check01.py frames|select sampling-plan02.json CHAPTER required|all   # stills to extract",
        ],
        "frame_lists": "frames-CHAPTER-required.txt / frames-CHAPTER-all.txt next to this plan (local indices)",
    },
    "derivation": {"script": "frame-windows01.py", "script_output": os.path.basename(src_path),
                   "script_output_sha256": hashlib.sha256(raw).hexdigest(), "inputs": fw["inputs"],
                   "validation": fw["checks"]},
    "acceptance": {"native_build": False, "native_render": False, "frames_viewed": False, "picture_acceptance": False,
                   "motion_acceptance": False, "listening": False, "fine_sync": False, "av_acceptance": False,
                   "publication": False},
}
os.makedirs(out_dir, exist_ok=True)
with open(os.path.join(out_dir, "sampling-plan02.json"), "x", encoding="utf-8") as fh:
    json.dump(plan, fh, indent=1)
for ch in FRAMES:
    for tier in ("required", "all"):
        frames = [s["local_frame"] for s in order if s["chapter"] == ch and (tier == "all" or s["tier"] == "required")]
        with open(os.path.join(out_dir, f"frames-{ch}-{tier}.txt"), "x", encoding="utf-8") as fh:
            fh.write("".join(f"{f}\n" for f in frames))
print(json.dumps(plan["counts"], indent=1))
for w in windows:
    print(f"{w['id']:28s} {w['chapter']:13s} local {w['local_range']} global {w['global_range']} "
          f"req {len(w['required_local'])} ext {len(w['extended_local'])}")
