"""Native 540p60 preview of the nine raster-fade chapters, for picture review of the plate fades.

Prepared by cc3 for root; cc3 has not run it. Root reviews it and runs it only under its own
exclusive guard (job csd-review-repair-preview274). Adapted from review-repair-preview243.py:
same admission check, render flags, extraction and sheet layout. What it adds is listed in
cc3-preview-fade274-notes.md. It approves nothing: it renders, compares schedules, places the
plate fades and extracts the frames a reviewer must look at, with every reason each frame was
picked. Picture, motion, 4K, listening, fine-sync and final AV review all remain separate.
"""
from fractions import Fraction
from functools import lru_cache
from pathlib import Path
import hashlib
import json
import math
import os
import re
import subprocess

R = Path('/data/projects/jmev-stroke-probe-20261106')
J = R / 'jobs/csd-review-repair-preview274'
S = J / 'preview'
N = R / 'jobs/csd-review-repair-rebind269/narration'
B = R / 'work/bin/review-repair268'
BUILD = R / 'jobs/csd-review-repair-build268'
T = R / 'jobs/csd-review-repair-timeline269'
BASE = R / 'jobs/csd-production-timeline21/timeline-output'
OLD = R / 'jobs/csd-production-assembly19/narration'
REPO = R / 'work/production-repo07'
WORKSPACE = R / 'work/review-repair268/workspace'
CHAPTER_DIR = 'explainer_corpus/complex_analysis_visual/src/chapters/'
FPS = 60

# The nine chapters whose plates fade, and the module each is built from.
CHAPTERS = {
    '01_the_bump': 'the_bump',
    '04_reading_color': 'reading_color',
    '05_gallery': 'gallery',
    '07_one_point': 'one_point',
    '09_boundary': 'boundary',
    '10_rigidity': 'rigidity',
    '11_counting': 'counting',
    '12_residues': 'residues',
    '14_continuation': 'continuing',
}

# The bytes build268 compiled (its source-copy.json), equal to cc3's local composed tree:
# preserved214 for the six unchanged chapters, recipe268's candidates for the rest.
SOURCE_SHA256 = {
    'the_bump': '8e93012e5b7598b9a99ffcdb07e436341f052d1175455f5581e108ee86e6ddb3',
    'reading_color': '8c79f5a1fc4cc18655b73a2f8755fec26fe0af586bcb2d426b018dd3f2ce932d',
    'gallery': 'd57e003a4067f3e67abb654e98c19ca0274c9b7edae5ac26da756757e33230b5',
    'one_point': '76a8bf1dc2ae4725d13cfee5b666d063b1cdbf00fc0d8c616dcbe43797e97b00',
    'boundary': 'dc80c541799209f10d02701d18c7fab5a14327000719929e5e2fad0c6fd62f9f',
    'rigidity': '0bcfbc7d7962b2ab8fa2d27895fce104ab6cb71ce5aac26fdd0b97dca04a3df0',
    'counting': 'ecff48f7ca1aea98b3ac8eac65ffe8f4a74d5366fd30ce25546def3b627339d1',
    'residues': '7d06f69b2aff86ac1bee103c7aaefc58891d0b335312e98df3610769f45af7c1',
    'continuing': '9cb4fc0bb87e48d9ce9793d09c0d31b4bfbfbfab7057074db204cf8d6b164f96',
    'draw': '9bb07b3ff030d1ac0885d1382097bde596e8cae419d86b51933e12e7eeed8218',
    'mod': 'e1f4d1b87f4b36f979e7800f7b2048117b9a4809b2372f385350698f5c3e7cbd',
}
TEST_SPLIT = '\n#[cfg(test)]\nmod tests {'


# ------------------------------------------------------------------ the plate-fade sites
#
# Every play in the nine chapters whose targets hold a raster plate, in program order. In
# candidate 264 such a play (Painter show/leave, or any fade_in_all/fade_out_all whose list
# holds a plate group) also carries the plate's own opacity ramp: same run time, same smooth
# rate, so it adds no frame.
#
# A play of run time rt lasts frames_covering(rt) = ceil(rt * 60) frames, on the exact double
# (fmn-anim clock.rs frames_covering at a399d626): 0.4 s is 25 frames, 0.8 s is 49, 0.1 s is 7.
# If it starts after s frames have been emitted, it fills video frames s .. s+N-1, and frame
# s+k-1 shows the plate at smooth(k / (60 rt)) going in, 1 - smooth(k / (60 rt)) going out
# (clock.rs sample, frame.rs alpha = t / run_time, rate.rs smooth).
#
# anchor:
#   ('line', cue, line)  s0 = the cue's start frame in the timeline (say records the line at
#                        the current frame and returns there);
#   ('breath', line)     s0 = the chapter's last breath row frame + its frames (music_breath
#                        opens a window at the current frame and waits it out);
#   ('after', id)        s0 = the end of site id (its last repeat); exact only if id is;
#   ('fit',)             no exact anchor: the site follows a measured sentence start or phrase.
#                        It is placed by fitting the exact source play sequence around it (pre +
#                        site + post, fixed offsets and run times) to the film's per-frame
#                        chroma, inside its cue and between its placed neighbours.
# before: the timed steps from the anchor to the site; s = s0 + their frames.
# pre/post: for 'fit' sites, the contiguous timed steps either side of the site in source.
# A step is ('play', line, text, rt, direction, holds_plate) or ('pause', line, text, seconds).
# repeat: consecutive identical plays of one loop with no timed step between them.
# evidence: further source lines whose exact text the placement relies on.
# ends: (line, text) when the site is the chapter's last play before kit.finish, so the
#       timeline's frame total must equal the site's end (an exact check of the anchors).

def play(line, text, rt, direction, plate=False):
    return ('play', line, text, rt, direction, plate)


def pause_step(line, text, seconds):
    return ('pause', line, text, seconds)


FINISH = 'kit.finish(stage)'
BREATH = 'kit.music_breath(stage, 2.0)?;'

SITES = {
    '01_the_bump': [
        dict(id='A1', line=330, text='fade_in_all(stage, chunk, 0.5)?;', dir='in', rt=0.5, cue='01_6',
             anchor=('fit',),
             why='first of three chunk fades: chunk 1 is [caption, pic.group, circle], so the z^2 '
                 'preview plate fades in here; the model repeats the loop body twice more',
             wait=(327, 'line.wait_sentence(stage, 2)?;'),
             pre=[],
             post=[pause_step(331, 'pause(stage, 0.6)?;', 0.6),
                   play(330, 'fade_in_all(stage, chunk, 0.5)?;', 0.5, 'in'),
                   pause_step(331, 'pause(stage, 0.6)?;', 0.6),
                   play(330, 'fade_in_all(stage, chunk, 0.5)?;', 0.5, 'in')],
             evidence={328:'let thumbs = previews(&kit, stage)?;',
                       329: 'for chunk in thumbs.chunks(thumbs.len().div_ceil(3)) {',
                       364: 'let mut out = Vec::new();',
                       367: 'let pic = portrait(stage, Func::Square, &small, 1.0)?;',
                       368: 'out.push(pic.group);',
                       370: 'out.push(lp.circle(stage, ZERO, 0.8, hero(), 4.0)?);',
                       375: 'out.push(stage.arena_mut().add(arrow));',
                       381: 'out.push(hp.path(stage, &half, hero(), 4.0)?);',
                       382: 'out.extend(zero_mark(stage, hp.p(I.scale(0.7))));',
                       384: 'out.insert(0, caption);'}),
        dict(id='A2', line=349, text='fade_out_all(stage, &all, 0.6)?;', dir='out', rt=0.6, cue='01_6',
             anchor=('breath', 335), before=[],
             why='closing fade; all holds thumbs, so the preview plate leaves here',
             evidence={348: 'all.extend(thumbs);'}, ends=(350, FINISH)),
    ],
    '04_reading_color': [
        dict(id='B1', line=221, text='painter.show(stage, 1.2)?;', dir='in', rt=1.2, cue='04_1',
             anchor=('fit',), why='identity plate in, after sentence 1',
             wait=(218, 'line.wait_sentence(stage, 1)?;'),
             pre=[], post=[play(223, 'fade_in_all(stage, &[border, formula], 0.3)?;', 0.3, 'in')]),
        dict(id='B2', line=438, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='04_7',
             anchor=('breath', 431), why='closing plate leave, after the vector closing fade',
             before=[play(437, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')], ends=(439, FINISH)),
    ],
    '05_gallery': [
        dict(id='C1', line=407, text='painter.show(stage, 1.0)?;', dir='in', rt=1.0, cue='05_1',
             anchor=('line', '05_1', 403), before=[], why='z^3 - 1 plate in at the cue start'),
        dict(id='C2', line=493, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_3',
             anchor=('line', '05_3', 486), why='z^3 - 1 plate out',
             before=[play(492, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='C3', line=497, text='exp.show(stage, 0.8)?;', dir='in', rt=0.8, cue='05_3',
             anchor=('after', 'C2'), before=[], why='e^z plate in, straight after C2'),
        dict(id='C4', line=564, text='exp.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_4',
             anchor=('line', '05_4', 547), why='e^z plate out',
             before=[play(548, 'fade_out_all(', 0.4, 'out')],
             evidence={562: '0.4,', 563: ')?;'}),
        dict(id='C5', line=568, text='trig.show(stage, 0.8)?;', dir='in', rt=0.8, cue='05_4',
             anchor=('after', 'C4'), before=[], why='sine plate in, straight after C4'),
        dict(id='C6', line=682, text='trig.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_6',
             anchor=('line', '05_6', 678), why='sine plate out',
             before=[play(681, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='C7', line=686, text='gamma.show(stage, 0.8)?;', dir='in', rt=0.8, cue='05_6',
             anchor=('after', 'C6'), before=[], why='Gamma plate in, straight after C6'),
        dict(id='C8', line=750, text='gamma.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_7',
             anchor=('line', '05_7', 745), why='Gamma plate out',
             before=[play(749, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='C9', line=757, text='zeta.show(stage, 0.8)?;', dir='in', rt=0.8, cue='05_7',
             anchor=('after', 'C8'), before=[], why='zeta plate in, straight after C8'),
        dict(id='C10', line=807, text='strip.show(stage, 0.8)?;', dir='in', rt=0.8, cue='05_7',
             anchor=('fit',), why='critical-strip plate in, after sentence 5',
             pre=[play(792, 'fade_out_all(stage, &[converges, pole_says, zedge], 0.3)?;', 0.3, 'out')],
             post=[play(829, 'fade_in_all(stage, &[start_box, sborder, band, half, squashed, critical], 0.4)?;',
                        0.4, 'in'),
                   play(830, 'fade_in_all(stage, &links, 0.3)?;', 0.3, 'in')],
             wait=(790, 'line.wait_sentence(stage, 5)?;')),
        dict(id='C11', line=896, text='zeta.leave(stage, 0.4)?;', dir='out', rt=0.4, cue='05_9',
             anchor=('line', '05_9', 877), why='zeta plate out',
             before=[play(895, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='C12', line=897, text='strip.leave(stage, 0.4)?;', dir='out', rt=0.4, cue='05_9',
             anchor=('after', 'C11'), before=[], why='strip plate out, straight after C11'),
        dict(id='C13', line=903, text='painter.show(stage, 0.25)?;', dir='in', rt=0.25, cue='05_9',
             anchor=('after', 'C12'), before=[], repeat=6,
             why='the six mosaic panels in, one after another (the loop body has no other timed step)',
             evidence={222: 'pub fn mosaic() -> [(Func, Plane); 6] {',
                       901: 'for &(f, p) in &panels {',
                       902: 'let painter = Painter::build(stage, Some(f), p, 1.0)?;',
                       904: 'frames.push(p.border(stage, Palette::faint())?);',
                       905: 'painters.push(painter);',
                       906: '}'}),
        dict(id='C14', line=942, text='fade_out_all(stage, &leaving, TILE_BEATS[1])?;', dir='out', rt=0.3,
             cue='05_9', anchor=('fit',), why='five of the six panels out at once, after "in little tiles"',
             pre=[play(931, 'fade_in_all(stage, &[boxed], TILE_BEATS[0])?;', 0.3, 'in')],
             post=[play(960, 'zoom.show(stage, TILE_BEATS[2])?;', 0.5, 'in', True),
                   play(963, 'fade_in_all(stage, &framing, TILE_BEATS[3])?;', 0.2, 'in')],
             wait=(927, 'line.wait_for(stage, "in little tiles")?;'),
             evidence={937:'for p in &painters[1..] {',
                       940: 'leaving.push(p.group);'}),
        dict(id='C15', line=960, text='zoom.show(stage, TILE_BEATS[2])?;', dir='in', rt=0.5, cue='05_9',
             anchor=('after', 'C14'), before=[], why='zoomed cubic plate in, straight after C14'),
        dict(id='C16', line=988, text='painters[0].leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_9',
             anchor=('breath', 981), why='the kept cubic panel out, after the vector closing fade',
             before=[play(987, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')]),
        dict(id='C17', line=989, text='zoom.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='05_9',
             anchor=('after', 'C16'), before=[], why='zoom plate out, last play', ends=(990, FINISH)),
    ],
    '07_one_point': [
        dict(id='D1', line=419, text='painter.show(stage, 1.0)?;', dir='in', rt=1.0, cue='07_3',
             anchor=('fit',), why='|z|^2 plate in, after sentence 1',
             wait=(413, 'line.wait_sentence(stage, 1)?;'),
             pre=[],
             post=[play(420, 'fade_in_all(stage, &[border, formula, hue], 0.4)?;', 0.4, 'in'),
                   play(421, 'fade_in_all(stage, &p_names, 0.2)?;', 0.2, 'in')]),
        dict(id='D2', line=585, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='07_5',
             anchor=('fit',), why='|z|^2 plate out, after sentence 4; the colourful vector mosaic '
                                  'fades in straight after',
             wait=(570, 'line.wait_sentence(stage, 4)?;'),
             pre=[play(584, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')],
             post=[play(587, 'fade_in_all(stage, &[mosaic], 0.5)?;', 0.5, 'in'),
                   play(590, 'fade_in_all(stage, &[squeeze, product_says], 0.4)?;', 0.4, 'in')]),
    ],
    '09_boundary': [
        dict(id='E1', line=474, text='painter.show(stage, 1.0)?;', dir='in', rt=1.0, cue='09_1',
             anchor=('line', '09_1', 470), before=[], why='bump plate in at the cue start'),
        dict(id='E2', line=1398, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='09_9',
             anchor=('breath', 1377), why='closing plate leave, after the vector closing fade',
             before=[play(1397, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')], ends=(1399, FINISH)),
    ],
    '10_rigidity': [
        dict(id='F1', line=457, text='painter.show(stage, 1.0)?;', dir='in', rt=1.0, cue='10_4',
             anchor=('line', '10_4', 427), why='sin(1/z) plate in (the plate rigidity247 moved up 0.1)',
             before=[play(443, 'fade_out_all(stage, &gone, 0.5)?;', 0.5, 'out')]),
        dict(id='F2', line=536, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='10_6',
             anchor=('line', '10_6', 529), why='sin(1/z) plate out',
             before=[play(535, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='F3', line=562, text='sine.show(stage, 1.0)?;', dir='in', rt=1.0, cue='10_6',
             anchor=('after', 'F2'), before=[], why='sin z plate in, straight after F2'),
        dict(id='F4', line=648, text='sine.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='10_6',
             anchor=('breath', 633), why='closing plate leave, after the vector closing fade',
             before=[play(647, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')], ends=(649, FINISH)),
    ],
    '11_counting': [
        dict(id='G1', line=196, text='painter.show(stage, 0.8)?;', dir='in', rt=0.8, cue='11_1',
             anchor=('line', '11_1', 194), before=[], why='identity input plate in at the cue start'),
        dict(id='G2', line=519, text='painter.leave(stage, 0.4)?;', dir='out', rt=0.4, cue='11_7',
             anchor=('line', '11_7', 504),
             why='identity plate out; the quintic strips field was added without a fade (add_to_scene) '
                 'just before this play, so it shows from the play\'s first frame',
             before=[play(511, 'fade_out_all(stage, &gone, 0.3)?;', 0.3, 'out')],
             evidence={514: 'let turning = Turning::build(stage, inp, out, walked_curve)?;',
                       653: 'let strips = portrait(stage, Func::Quintic, &inp, 1.0)?;',
                       654: 'let field = strips.group;',
                       675: 'stage.add_to_scene(field)?;'}),
        dict(id='G3', line=567, text='fade_out_all(stage, &acted, 0.3)?;', dir='out', rt=0.3, cue='11_7',
             anchor=('fit',), why='the strips field out with the acted picture, after sentence 3',
             wait=(563, 'line.wait_sentence(stage, 3)?;'),
             pre=[], post=[play(568, 'fade_in_all(stage, &wide_all, 0.5)?;', 0.5, 'in')],
             evidence={565:'let mut acted = vec![claim, degree_n, zero_kept, walked_loop, walked_curve, '
                            'turning.field];'}),
    ],
    '12_residues': [
        dict(id='H1a', line=282, text='painter.show(stage, 0.6)?;', dir='in', rt=0.6, cue='12_3',
             anchor=('fit',), why='first small plate in (loop k = 0), after sentence 1 and the zero note; '
                                  'the model also holds the k = 1 iteration',
             wait=(265, 'line.wait_sentence(stage, 1)?;'), skip={275, 278},
             pre=[play(268, 'fade_in_all(stage, &[zero_tex, zero_says], 0.4)?;', 0.4, 'in')],
             post=[play(310, 'fade_in_all(stage, &extra, 0.4)?;', 0.4, 'in'),
                   play(282, 'painter.show(stage, 0.6)?;', 0.6, 'in', True),
                   play(310, 'fade_in_all(stage, &extra, 0.4)?;', 0.4, 'in')],
             evidence={102: 'pub fn small_planes() -> [Plane; 3] {',
                       273: 'for (k, sp) in small_planes().into_iter().enumerate() {',
                       274: 'if k == 2 {',
                       280: '}'}),
        dict(id='H1b', line=282, text='painter.show(stage, 0.6)?;', dir='in', rt=0.6, cue='12_3',
             anchor=('after', 'H1a'), gap=False,
             why='second small plate in (loop k = 1): the k = 0 body ends with the extras fade and '
                 'k = 1 skips the k == 2 block, so it follows H1a by the extras fade alone',
             before=[play(310, 'fade_in_all(stage, &extra, 0.4)?;', 0.4, 'in')]),
        dict(id='H1c', line=282, text='painter.show(stage, 0.6)?;', dir='in', rt=0.6, cue='12_3',
             anchor=('fit',), why='third small plate in (loop k = 2), after sentence 2 and the pole note',
             wait=(275, 'line.wait_sentence(stage, 2)?;'),
             pre=[play(278, 'fade_in_all(stage, &[pole_tex, pole_says], 0.4)?;', 0.4, 'in')],
             post=[play(310, 'fade_in_all(stage, &extra, 0.4)?;', 0.4, 'in')]),
        dict(id='H2', line=324, text='painter.leave(stage, 0.4)?;', dir='out', rt=0.4, cue='12_4',
             anchor=('line', '12_4', 318), repeat=3, why='the three small plates out, one after another',
             before=[play(322, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')],
             evidence={323: 'for painter in &painters {', 325: '}'}),
        dict(id='H3', line=329, text='bump.show(stage, 0.8)?;', dir='in', rt=0.8, cue='12_4',
             anchor=('after', 'H2'), before=[], why='bump plate in, straight after H2'),
        dict(id='H4', line=362, text='bump.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='12_4',
             anchor=('breath', 358), why='closing plate leave, after the vector closing fade',
             before=[play(361, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')], ends=(363, FINISH)),
    ],
    '14_continuation': [
        dict(id='I1', line=275, text='painter.show(stage, 1.0)?;', dir='in', rt=1.0, cue='14_1',
             anchor=('line', '14_1', 271), before=[], why='bump plate in at the cue start'),
        dict(id='I2', line=321, text='painter.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='14_2',
             anchor=('line', '14_2', 317), why='bump plate out',
             before=[play(320, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='I3', line=686, text='lac.show(stage, 1.0)?;', dir='in', rt=1.0, cue='14_7',
             anchor=('line', '14_7', 679), why='lacunary plate in',
             before=[play(682, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='I4', line=738, text='lac.leave(stage, 0.4)?;', dir='out', rt=0.4, cue='14_8',
             anchor=('line', '14_8', 733), why='lacunary plate out',
             before=[play(737, 'fade_out_all(stage, &gone, 0.4)?;', 0.4, 'out')]),
        dict(id='I5', line=747, text='gamma.show(stage, 0.1)?;', dir='in', rt=0.1, cue='14_8',
             anchor=('after', 'I4'), before=[],
             why='blank Gamma painter (no function yet) in for 7 frames, straight after I4; little or '
                 'no visible change is expected',
             evidence={741: 'let gamma = Painter::build(stage, None, gp, 1.0)?;'}),
        dict(id='I6', line=799, text='gamma.leave(stage, 0.5)?;', dir='out', rt=0.5, cue='14_8',
             anchor=('breath', 795), why='closing plate leave, after the vector closing fade',
             before=[play(798, 'fade_out_all(stage, &all, 0.5)?;', 0.5, 'out')], ends=(800, FINISH)),
    ],
}

# Lines in the nine files that name a show/leave, a portrait or a group but are not plate-fade
# sites (or site evidence), with the reason. The completeness scan refuses any other such line.
ALLOW = {
    'gallery': {377: ('w.show(stage, 0.4)?;', 'walker: trail, arrow and two dots, all vector')},
    'counting': {
        161: ('w.show(stage, WALKER_IN)?;', 'walker: trail, arrow and two dots, all vector'),
        571: ('dog.show(stage)?;', 'DogWalk: dots and paths, all vector'),
        597: ('stage.add_to_scene(panel.group)?;', 'explainer_kit CodePanel: text and card, vector'),
        605: ('last.push(panel.group);', 'explainer_kit CodePanel: text and card, vector'),
    },
}
PLATE_LINE = re.compile(r'\.show\(stage|\.leave\(stage|\bportrait(_styled)?\(|Picture::build|ImageMobject'
                        r'|image_resource|\.group\b|\.field\b')
TIMED = re.compile(r'\bsay\(|fade_in_all\(|fade_out_all\(|\.show\(|\.leave\(|\.play\(|play_prepared\('
                   r'|\bpause\(|\bwait_|\.until\(|music_breath\(|\.hold\(|names_now\(|precisely_now\('
                   r'|reading_window\(|\bpredict\(|coarse_loop\(|\.walk\(|\.to\(stage|enhance_to\('
                   r'|\.listen\(|\.sonify\(|\btween|shift_all\(|slide_in\(')
TILE_LINE = 161
TILE_TEXT = 'pub const TILE_BEATS: [f64; 4] = [0.3, 0.3, 0.5, 0.2];'

# Old anchors from the gallery249 and rigidity247 handoffs (times on the production timeline,
# which the schedule comparison below proves unchanged).
GALLERY249_ANCHORS = [
    (153.92, 'gallery249 qualifier and cross: "nearly square" arrival, old final05 estimate 153.92 s'),
    (154.0, 'cc1 review255 stills: tile label not yet visible at 154.0 s'),
    (155.0, 'cc1 review255 stills: tile label fully up at 155.0 s'),
    (157.0, 'gallery249: sentence 4 ("next") between 157.0 and 157.6 s'),
    (157.6, 'gallery249: sentence 4 ("next") between 157.0 and 157.6 s'),
    (160.547, 'gallery249: end of the 05_9 take, 160.547 s'),
    (163.15, 'gallery249 / cc1 review255: closing fade begins 163.15 s (breath 9669 + 120 frames)'),
]
RIGIDITY247_WINDOWS = [
    (12.0, 16.7, 15, 'rigidity247 L3, review253 R1/R2: q, its backing ring and the pile-up (old anchor '
                     '12.0-16.7 s), quarter-second'),
    (62.3, 104.0, 120, 'rigidity247 L1, review253 R3: plate top, title and "Im" name (old anchor '
                       '62.3-104 s), two-second'),
    (88.0, 103.6, 60, 'rigidity247 L1: plate foot against the "precisely" card (old anchor 88.0-103.6 s), '
                      'one-second'),
]

# Chroma fit settings for the 'fit' sites. These are untested on real frames (see the notes).
MARGIN = 12        # frames of flat signal kept either side of the modelled plays
T_MIN = 8.0        # least slope t-statistic over the modelled plate plays
AMP_MIN = 0.25     # least plate slope, in SATAVG units
AMBIGUOUS = 0.8    # a separate runner-up scoring at least this share of the best is ambiguous
CHECK_SHIFT = 8    # exact anchors: report the best fit within this many frames, never move them
GAMMAS = (1.0, 0.75, 0.55, 0.4)  # response curves tried: chroma ~ visibility ** gamma


def sha(path):
    with path.open('rb') as stream:
        return hashlib.file_digest(stream, 'sha256').hexdigest()


@lru_cache(maxsize=None)
def frames_covering(seconds):
    """fmn's frames_covering at 60 fps: ceil of the exact double times 60."""
    return math.ceil(Fraction(seconds) * FPS) if seconds > 0 else 0


def smooth(t):
    s = 1.0 - t
    return t ** 3 * (10.0 * s * s + 5.0 * s * t + t * t)


def visibility(n, start, rt, direction):
    """How visible a play's targets are at video frame n, for a play starting at frame start."""
    k = n - start + 1
    if k <= 0:
        v = 0.0
    elif k >= frames_covering(rt):
        v = 1.0
    else:
        v = min(1.0, max(0.0, smooth(min(1.0, (k / FPS) / rt))))
    return v if direction == 'in' else 1.0 - v


def finite(x):
    return None if isinstance(x, float) and not math.isfinite(x) else x


def clean(x):
    """Strict JSON: infinities become null, tuples and sets become lists."""
    if isinstance(x, dict):
        return {k: clean(v) for k, v in x.items()}
    if isinstance(x, set):
        return sorted(clean(v) for v in x)
    if isinstance(x, (list, tuple)):
        return [clean(v) for v in x]
    return finite(x)


# ------------------------------------------------------------------ source checks

def rt_of(text, code):
    """The run time a source line passes: its last literal, or the TILE_BEATS entry it names."""
    tile = re.search(r'TILE_BEATS\[(\d)\]', text)
    if tile:
        assert code[TILE_LINE - 1].strip() == TILE_TEXT, ('TILE_BEATS changed', code[TILE_LINE - 1].strip())
        return [0.3, 0.3, 0.5, 0.2][int(tile[1])]
    last = re.search(r'(\d+\.\d+)\)\?;$', text)
    return float(last[1]) if last else None


def check_sources(read):
    """read(module) -> text. Refuses any drift between the site table and the compiled source."""
    report = {}
    for chapter, module in CHAPTERS.items():
        code = read(module).split(TEST_SPLIT)[0].split('\n')

        def at(line, text, what):
            got = code[line - 1].strip() if 0 < line <= len(code) else None
            assert got == text, (chapter, what, line, got, text)

        def quiet(first, last, listed, what):
            for line in range(first + 1, last):
                raw = code[line - 1]
                if TIMED.search(raw) and not raw.strip().startswith('//'):
                    assert line in listed, (chapter, what, 'unlisted timed line', line, raw.strip())
            return True

        known = {}
        ids = {}
        for site in SITES[chapter]:
            ids[site['id']] = site
            at(site['line'], site['text'], site['id'])
            assert rt_of(site['text'], code) == site['rt'], site['id']
            known[site['line']] = site['id']
            kind = site['anchor'][0]
            if kind == 'line':
                assert site['anchor'][1] == site['cue'], site['id']
                at(site['anchor'][2], 'let line = say(&kit, stage, "' + site['cue'] + '")?;', site['id'] + ' anchor')
            elif kind == 'breath':
                at(site['anchor'][1], BREATH, site['id'] + ' anchor')
            for step in site.get('before', []) + site.get('pre', []) + site.get('post', []):
                at(step[1], step[2], site['id'] + ' step')
                if step[2] == 'fade_out_all(':
                    assert site['evidence'][step[1] + 14] == str(step[3]) + ',', site['id']
                else:
                    assert rt_of(step[2], code) == step[3], (site['id'], step)
                known.setdefault(step[1], site['id'])
            for line, text in site.get('evidence', {}).items():
                at(line, text, site['id'] + ' evidence')
                known.setdefault(line, site['id'])
            if 'ends' in site:
                at(site['ends'][0], site['ends'][1], site['id'] + ' end')
            listed = {step[1] for step in site.get('before', [])}
            # nothing timed between the anchor (or the previous site) and the site but the listed
            # steps; loop iterations (gap=False) are justified in their 'why'
            if site.get('gap', True):
                if kind == 'line':
                    quiet(site['anchor'][2], site['line'], listed, site['id'])
                elif kind == 'breath':
                    quiet(site['anchor'][1], site['line'], listed, site['id'])
                elif kind == 'after':
                    quiet(ids[site['anchor'][1]]['line'], site['line'], listed, site['id'])
            if kind == 'fit':
                # the modelled sequence starts at the wait and is contiguous wherever it runs
                # forward in the file; `skip` lines sit in a branch this iteration does not take
                at(site['wait'][0], site['wait'][1], site['id'] + ' wait')
                assert TIMED.search(site['wait'][1]) and 'wait_' in site['wait'][1]
                known.setdefault(site['wait'][0], site['id'])
                skip = site.get('skip', set())
                chain = [site['wait'][0]] + [s[1] for s in site['pre']] + [site['line']] + \
                    [s[1] for s in site['post']]
                for a, b in zip(chain, chain[1:]):
                    if b > a:
                        quiet(a, b, skip, site['id'] + ' model')
        for line, (text, _) in ALLOW.get(module, {}).items():
            at(line, text, 'allow')
        for i, raw in enumerate(code, 1):
            if PLATE_LINE.search(raw) and not raw.strip().startswith('//'):
                assert i in known or i in ALLOW.get(module, {}), (chapter, 'unlisted plate line', i, raw.strip())
        report[chapter] = dict(sites=len(SITES[chapter]),
                               plays=sum(s.get('repeat', 1) for s in SITES[chapter]),
                               checked_lines=len(known))
    return report


# ------------------------------------------------------------------ timeline and chroma

def read_timeline(path):
    rows = [line.split('\t') for line in path.read_text().splitlines() if line and not line.startswith('#')]
    assert next(int(r[1]) for r in rows if r[0] == 'fps') == FPS
    return dict(lines=[(r[1], int(r[2])) for r in rows if r[0] == 'line'],
                marks=[(r[1], int(r[2])) for r in rows if r[0] == 'mark'],
                breaths=[(int(r[1]), float(r[2])) for r in rows if r[0] == 'breath'],
                frames=next(int(r[1]) for r in rows if r[0] == 'frames'))


def chroma(movie, out, count):
    """Per-frame mean chroma (ffmpeg signalstats SATAVG): one number a frame, no pictures."""
    graph = 'signalstats,metadata=mode=print:key=lavfi.signalstats.SATAVG:file=' + str(out)
    subprocess.run(['ffmpeg', '-nostdin', '-v', 'error', '-xerror', '-threads', '2', '-i', str(movie), '-an',
                    '-vf', graph, '-f', 'null', '-'], check=True, timeout=3600)
    values, frame = [], None
    for line in out.read_text().splitlines():
        if line.startswith('frame:'):
            frame = int(line.split()[0].split(':')[1])
        elif line.startswith('lavfi.signalstats.SATAVG='):
            assert frame == len(values), (movie, frame, len(values))
            values.append(float(line.split('=', 1)[1]))
    assert len(values) == count, (movie, len(values), count)
    return values


def solve(a, b):
    """Gauss-Jordan with partial pivoting; x with a x = b, or None if singular."""
    n = len(a)
    m = [row[:] + [b[i]] for i, row in enumerate(a)]
    for col in range(n):
        piv = max(range(col, n), key=lambda r: abs(m[r][col]))
        if abs(m[piv][col]) < 1e-12:
            return None
        m[col], m[piv] = m[piv], m[col]
        for r in range(n):
            if r != col:
                f = m[r][col] / m[col][col]
                for c in range(col, n + 1):
                    m[r][c] -= f * m[col][c]
    return [m[i][n] / m[i][i] for i in range(n)]


def regress(y, columns):
    """Least squares y ~ 1 + columns: [(slope, t statistic)] per column, or None."""
    xs = [[1.0] * len(y)] + columns
    p = len(xs)
    if len(y) <= p:
        return None
    xtx = [[sum(u * v for u, v in zip(xs[i], xs[j])) for j in range(p)] for i in range(p)]
    beta = solve(xtx, [sum(u * v for u, v in zip(xs[i], y)) for i in range(p)])
    if beta is None:
        return None
    rss = sum((y[n] - sum(beta[i] * xs[i][n] for i in range(p))) ** 2 for n in range(len(y)))
    sigma2 = max(rss / (len(y) - p), 1e-12)
    out = []
    for j in range(1, p):
        inv = solve(xtx, [1.0 if i == j else 0.0 for i in range(p)])
        if inv is None or inv[j] <= 0:
            return None
        out.append((beta[j], beta[j] / math.sqrt(sigma2 * inv[j])))
    return out


def model(site, start):
    """The modelled plays around a site starting at frame `start`, as (start, rt, dir, plate, label),
    and the frame where the modelled sequence ends: the steps before it (pre for a 'fit' site, the
    anchor chain otherwise), its repeats, and for a 'fit' site its post steps."""
    plays = []
    steps = site['pre'] if site['anchor'][0] == 'fit' else site.get('before', [])
    at = start
    for step in reversed(steps):
        at -= frames_covering(step[3])
        if step[0] == 'play':
            plays.insert(0, (at, step[3], step[4], step[5], 'pre:%d' % step[1]))
    at = start
    for r in range(site.get('repeat', 1)):
        plays.append((at, site['rt'], site['dir'], True, '%s#%d' % (site['id'], r + 1)))
        at += frames_covering(site['rt'])
    for step in site.get('post', []) if site['anchor'][0] == 'fit' else []:
        if step[0] == 'play':
            plays.append((at, step[3], step[4], step[5], 'post:%d' % step[1]))
        at += frames_covering(step[3])
    return plays, at


def score(signal, plays, others=()):
    """Fit the modelled plays to the chroma around them. `others` are plays already placed (held
    fixed, free sign) that reach into the window. The film's chroma is a concave, unknown function
    of each play's visibility (the renderer blends in linear light, the film is sRGB), so the
    visibility is raised to each power in GAMMAS and the best fitting one is kept."""
    first = max(0, min(p[0] for p in plays) - MARGIN)
    last = min(len(signal), max(p[0] + frames_covering(p[1]) for p in plays) + MARGIN)
    if last - first < 2 * MARGIN or min(p[0] for p in plays) < 0:
        return None
    seen = {(p[0], p[1], p[2]) for p in plays}
    plays = list(plays)
    for o in others:
        if o[0] < last and o[0] + frames_covering(o[1]) > first and (o[0], o[1], o[2]) not in seen:
            seen.add((o[0], o[1], o[2]))
            plays.append((o[0], o[1], o[2], False, 'fixed:' + o[4]))
    base = [[visibility(n, p[0], p[1], p[2]) for n in range(first, last)] for p in plays]
    best = None
    for gamma in GAMMAS:
        fit = regress(signal[first:last], [[v ** gamma for v in col] for col in base])
        if fit is None:
            continue
        plate = [bt for bt, p in zip(fit, plays) if p[3]]
        slopes = [dict(play=p[4], slope=b, t=t) for (b, t), p in zip(fit, plays)]
        amp = min(b for b, _ in plate)
        got = dict(score=min(t for _, t in plate) if amp > 0 else -math.inf, amplitude=amp, gamma=gamma,
                   slopes=slopes)
        if best is None or got['score'] > best['score']:
            best = got
    return best


def locate(signal, site, lo, hi, others=()):
    """Best start for a 'fit' site in [lo, hi]: a scan in steps of 3, then every frame near the best."""
    def at(s):
        got = score(signal, model(site, s)[0], others)
        return -math.inf if got is None else got['score']
    coarse = {s: at(s) for s in range(lo, hi + 1, 3)}
    if not coarse:
        return None
    best = max(coarse, key=coarse.get)
    fine = {s: at(s) for s in range(max(lo, best - 3), min(hi, best + 3) + 1)}
    best = max(fine, key=fine.get)
    extent = model(site, 0)[1]
    rivals = [v for s, v in coarse.items() if abs(s - best) > extent]
    detail = score(signal, model(site, best)[0], others) or {}
    return dict(start=best, score=fine[best], runner_up=max(rivals) if rivals else None,
                amplitude=detail.get('amplitude'), slopes=detail.get('slopes'))


# ------------------------------------------------------------------ placement

def place(chapter, tl, signal):
    """Place every site: exact anchors by arithmetic, 'fit' sites by the chroma fit."""
    cue_start = dict(tl['lines'])
    cue_ids = [c for c, _ in tl['lines']]
    cue_end = {c: (tl['lines'][i + 1][1] if i + 1 < len(cue_ids) else tl['frames']) for i, c in enumerate(cue_ids)}
    sites = SITES[chapter]
    placed = {}

    def end(p):
        return p['start'] + p['frames'] * p['repeat']

    def fixed(but):
        """Every play of every placed site except `but`, at its placed frames."""
        out = []
        for s in sites:
            q = placed.get(s['id'])
            if s['id'] != but and q is not None and q['start'] is not None:
                out.extend(model(s, q['start'])[0])
        return out

    def place_exact():
        """Place every site whose anchor is resolvable now (exact, or after a placed site)."""
        moved = True
        while moved:
            moved = False
            for site in sites:
                if site['id'] in placed or site['anchor'][0] == 'fit':
                    continue
                kind = site['anchor'][0]
                entry = dict(frames=frames_covering(site['rt']), repeat=site.get('repeat', 1),
                             source='%s.rs:%d' % (CHAPTERS[chapter], site['line']))
                before = sum(frames_covering(step[3]) for step in site.get('before', []))
                if kind == 'line':
                    entry.update(start=cue_start[site['anchor'][1]] + before, how='exact line', exact=True)
                elif kind == 'breath':
                    last = max(tl['breaths'])
                    entry.update(start=last[0] + frames_covering(last[1]) + before, how='exact breath', exact=True)
                else:
                    prev = placed.get(site['anchor'][1])
                    if prev is None:
                        continue
                    if prev['start'] is None:
                        entry.update(start=None, how='NOT LOCALIZED', exact=False, search=prev['search'],
                                     note='follows unlocalized ' + site['anchor'][1])
                    else:
                        entry.update(start=end(prev) + before, exact=prev['exact'],
                                     how='exact after' if prev['exact'] else 'after fitted ' + site['anchor'][1])
                placed[site['id']] = entry
                moved = True

    place_exact()
    while len(placed) < len(sites):
        # the first unplaced 'fit' site whose program-order predecessors are all placed
        k, site = next((k, s) for k, s in enumerate(sites) if s['id'] not in placed and s['anchor'][0] == 'fit'
                       and all(t['id'] in placed for t in sites[:k]))
        pre = sum(frames_covering(step[3]) for step in site['pre'])
        extent = model(site, 0)[1]
        lo = max([cue_start[site['cue']] + pre] +
                 [end(placed[s['id']]) + pre for s in sites[:k] if placed[s['id']]['start'] is not None])
        later = [placed[s['id']]['start'] for s in sites[k + 1:]
                 if s['id'] in placed and placed[s['id']]['start'] is not None]
        hi = min([cue_end[site['cue']] - extent] + [x - extent for x in later])
        found = locate(signal, site, lo, hi, fixed(site['id'])) if hi >= lo else None
        ok = bool(found) and found['score'] >= T_MIN and (found['amplitude'] or 0) >= AMP_MIN \
            and (found['runner_up'] is None or found['runner_up'] < AMBIGUOUS * found['score'])
        placed[site['id']] = dict(frames=frames_covering(site['rt']), repeat=site.get('repeat', 1),
                                  source='%s.rs:%d' % (CHAPTERS[chapter], site['line']),
                                  start=found['start'] if ok else None, how='chroma fit' if ok else 'NOT LOCALIZED',
                                  exact=False, search=[lo, hi], fit=found)
        place_exact()

    for k, site in enumerate(sites):
        p = placed[site['id']]
        if p['exact']:
            # report how well the film agrees with the exact anchor, the other sites held where
            # they are; never move it
            others = fixed(site['id'])
            shifts = {}
            for d in range(-CHECK_SHIFT, CHECK_SHIFT + 1):
                got = score(signal, model(site, p['start'] + d)[0], others)
                shifts[d] = -math.inf if got is None else got['score']
            best = max(shifts, key=shifts.get)
            at0 = score(signal, model(site, p['start'])[0], others) or {}
            p['check'] = dict(score_at_anchor=finite(shifts[0]), amplitude_at_anchor=at0.get('amplitude'),
                              best_shift=best, best_score=finite(shifts[best]))
            p['flag'] = bool(abs(best) > 3 and shifts[best] >= T_MIN and shifts[best] > 1.5 * max(shifts[0], 0))
        if 'ends' in site:
            assert p['exact'] and end(p) == tl['frames'], (chapter, site['id'], 'end anchor', p, tl['frames'])
            p['end_check'] = 'site end equals the timeline frame total %d' % tl['frames']
    return placed


def picks_for(chapter, tl, placed, add):
    """Add every frame to review for this chapter, each with its reason."""
    frame_count = tl['frames']
    add(0, 'First frame')
    add(frame_count - 1, 'Last frame')
    for kind, rows in (('line', tl['lines']), ('mark', tl['marks'])):
        for label, frame in rows:
            for delta in (0, 30, 60, 120):
                add(frame + delta, kind + ':' + label)
    for site in SITES[chapter]:
        p = placed[site['id']]
        n = p['frames']
        tag = '%s %s (%s, %s %.2f s = %d frames, cue %s, %s)' % (
            site['id'], site['text'], p['source'], site['dir'], site['rt'], n, site['cue'], p['how'])
        if p['start'] is None:
            lo, hi = p['search']
            step = 15 if hi - lo <= 3600 else 30
            for f in range(lo, hi + model(site, 0)[1] + 1, step):
                add(f, 'Plate fade NOT LOCALIZED by the chroma fit; search-window sample: ' + tag)
            continue
        for r in range(p['repeat']):
            s = p['start'] + r * n
            rep = tag + ('' if p['repeat'] == 1 else ' repeat %d of %d' % (r + 1, p['repeat']))
            add(s - 1, 'Plate fade, frame before start: ' + rep)
            add(s, 'Plate fade start, ramp frame 1 of %d: %s' % (n, rep))
            add(s + (n + 1) // 2 - 1, 'Plate fade middle, ramp frame %d of %d: %s' % ((n + 1) // 2, n, rep))
            add(s + n - 1, 'Plate fade last frame, ramp frame %d of %d: %s' % (n, n, rep))
            add(s + n, 'First hold frame after plate fade: ' + rep)
            add(s + n + 30, 'Hold +0.5 s after plate fade (persistence): ' + rep)
            if site['dir'] == 'out':
                for f in range(s + n - 3, s + n + 4):
                    add(f, 'Fade-out end bracket, every frame (no reappearance): ' + rep)
            if not p['exact']:
                for f in list(range(s - 3, s + 3)) + list(range(s + n - 3, s + n + 4)):
                    add(f, 'Localization bracket, start and end +-3 frames: ' + rep)
        if p.get('flag'):
            s = p['start'] + p['check']['best_shift']
            for f in (s - 1, s, s + (n + 1) // 2 - 1, s + n - 1, s + n):
                add(f, 'Chroma best fit %+d frames from the exact anchor (flagged): %s' % (p['check']['best_shift'], tag))
    if chapter == '05_gallery':
        for at, why in GALLERY249_ANCHORS:
            for delta in (-.2, 0, .2, 1, 2):
                add(round((at + delta) * 60), why)
        last = max(tl['breaths'])
        c0 = last[0] + frames_covering(last[1])
        for f, part in ((c0, 'start'), (c0 + 14, 'middle'), (c0 + 29, 'last frame')):
            add(f, 'gallery249 qualifier and cross leave in the closing 0.5 s vector fade: ' + part)
    if chapter == '10_rigidity':
        for t0, t1, step, why in RIGIDITY247_WINDOWS:
            for f in range(round(t0 * 60), round(t1 * 60) + 1, step):
                add(f, why)
            add(round(t1 * 60), why)
    for line in (S / (chapter + '.layout.tsv')).read_text().splitlines():
        if not line.strip():
            continue
        match = re.search(r'@([0-9]+(?:\.[0-9]+)?)s', line)
        assert match, line
        at = float(match[1])
        later = re.search(r'still there at (\d+) later looks', line)
        count = int(later[1]) if later else 0
        for delta in (-.2, 0, .2, 1, 2, 1 + .2 * count, 2 + .2 * count):
            add(round((at + delta) * 60), 'Current lint: ' + line)
        if later:
            for f in range(round(at * 60), frame_count, 300):
                add(f, 'Persistent current lint: ' + line)


# ------------------------------------------------------------------ main

def main():
    from PIL import Image, ImageDraw

    a = json.loads((J / 'resource-receipt.json').read_text())
    assert a['admitted'] and 'exit' not in a
    assert a['policy']['stop_free_gib'] == 8 and a['policy']['phase_growth_gib'] == 4
    qualification = json.loads((BUILD / 'qualification.json').read_text())
    assert sha(B) == qualification['binary_sha256']
    assert qualification['tests_passed'] == 233 and qualification['explicit_native_plate_fade_probe_passed']
    assert qualification['clippy_passed'] and set(CHAPTERS) <= set(qualification['chapters'])
    copied = json.loads((BUILD / 'source-copy.json').read_text())['files']
    for module, digest in SOURCE_SHA256.items():
        assert copied[CHAPTER_DIR + module + '.rs'] == digest, module
        assert sha(WORKSPACE / (CHAPTER_DIR + module + '.rs')) == digest, module
    source_report = check_sources(lambda m: (WORKSPACE / (CHAPTER_DIR + m + '.rs')).read_text())
    comparison = json.loads((T / 'comparison.json').read_text())
    compared = {(r['chapter'], r['kind']): r for r in comparison['tables']}

    os.sched_setaffinity(0, set(range(16)))
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(R / 'work/tmp'), XDG_CACHE_HOME=str(R / 'work/cache'))
    with (J / 'native.stdout').open('xb') as out, (J / 'native.stderr').open('xb') as err:
        subprocess.run([str(B), 'render', *CHAPTERS, '--res', '960x540', '--fps', '60', '--threads', '16',
                        '--narration', str(N), '--out', str(S), '--lint', '--crf', '18', '--preset', 'veryfast'],
                       cwd=REPO, stdout=out, stderr=err, check=True, timeout=172800)

    summary = dict(binary_sha256=qualification['binary_sha256'], sources=source_report, chapters={})
    for chapter in CHAPTERS:
        dest = J / chapter
        dest.mkdir()
        # exact schedule comparison: the preview's own tables against timeline269 (same narration
        # path, so byte equality) and against production timeline21 under its named path mapping
        schedule = []
        for kind in ('timeline', 'cues', 'cuelog'):
            name = chapter + '.' + kind + '.tsv'
            mine = (S / name).read_bytes()
            assert mine == (T / 'timeline-output' / name).read_bytes(), (chapter, kind, 'timeline269')
            assert mine == (BASE / name).read_bytes().replace(str(OLD).encode(), str(N).encode()), \
                (chapter, kind, 'production timeline21')
            row = compared[(chapter, kind)]
            assert row['equal_after_named_narration_path_mapping']
            assert row['new_sha256'] == hashlib.sha256(mine).hexdigest()
            schedule.append(dict(kind=kind, sha256=row['new_sha256'], equals_timeline269=True,
                                 equals_production_timeline21_after_path_mapping=True))
        tl = read_timeline(S / (chapter + '.timeline.tsv'))
        frame_count = tl['frames']
        assert frame_count == comparison['frames'][chapter]
        movie = S / (chapter + '.mp4')
        p = json.loads(subprocess.check_output(['ffprobe', '-v', 'error', '-show_streams', '-of', 'json', str(movie)]))
        v = [x for x in p['streams'] if x['codec_type'] == 'video']
        assert len(v) == 1 and (v[0]['width'], v[0]['height'], v[0]['avg_frame_rate'], int(v[0]['nb_frames'])) == \
            (960, 540, '60/1', frame_count)
        signal = chroma(movie, dest / 'chroma.txt', frame_count)
        placed = place(chapter, tl, signal)

        picked = {}

        def add(f, why):
            if 0 <= f < frame_count:
                picked.setdefault(f, []).append(why)

        picks_for(chapter, tl, placed, add)

        def balanced(xs):
            if len(xs) == 1:
                return xs[0]
            k = len(xs) // 2
            return '(' + balanced(xs[:k]) + '+' + balanced(xs[k:]) + ')'

        frames = sorted(picked)
        filter_path = dest / 'select.filter'
        filter_path.write_text("select='" + balanced(['eq(n,' + str(f) + ')' for f in frames]) + "'")
        pictures = dest / 'pictures'
        pictures.mkdir()
        subprocess.run(['ffmpeg', '-nostdin', '-n', '-v', 'error', '-xerror', '-threads', '2', '-i', str(movie), '-an',
                        '-filter_script:v', str(filter_path), '-vsync', '0', '-q:v', '2',
                        str(pictures / 'frame-%06d.jpg')], check=True, timeout=900)
        files = sorted(pictures.glob('frame-*.jpg'))
        assert len(files) == len(frames)
        samples = [dict(frame=f, seconds=f / 60, chroma=signal[f], reasons=picked[f], path=str(q), sha256=sha(q))
                   for f, q in zip(frames, files)]
        sheets = []
        for k in range(0, len(samples), 8):
            canvas = Image.new('RGB', (1920, 2260), '#10151d')
            draw = ImageDraw.Draw(canvas)
            for i, sample in enumerate(samples[k:k + 8]):
                with Image.open(sample['path']) as im:
                    assert im.size == (960, 540)
                    x, y = (i % 2) * 960, (i // 2) * 565
                    canvas.paste(im.convert('RGB'), (x, y))
                    draw.text((x + 5, y + 542), '%s frame %d %.3fs  %s' % (
                        chapter, sample['frame'], sample['seconds'], sample['reasons'][0][:96]), fill='white')
            q = dest / ('sheet-%04d.jpg' % (k // 8 + 1))
            canvas.save(q, quality=94)
            sheets.append(dict(path=str(q), sha256=sha(q)))
        fades = [dict(id=s['id'], why=s['why'], **placed[s['id']]) for s in SITES[chapter]]
        flags = [f['id'] for f in fades if f['how'] == 'NOT LOCALIZED' or f.get('flag')]
        with (dest / 'sampling.json').open('x') as f:
            json.dump(clean(dict(movie_sha256=sha(movie), schedule=schedule, frames=frame_count, fades=fades,
                                 fade_flags=flags, samples=samples, sheets=sheets,
                                 scope='Native 540p60 plate-fade preview extraction of build268; picks only. '
                                       'Actual viewing of every sheet, motion, 4K, listening, fine-sync and '
                                       'independent AV review still required; no picture or AV acceptance.')),
                      f, indent=2, allow_nan=False)
        summary['chapters'][chapter] = dict(
            frames=frame_count, samples=len(samples), sheets=len(sheets),
            plate_plays=sum(x['repeat'] for x in placed.values()),
            exact=sum(x['repeat'] for x in placed.values() if x['exact']),
            fitted=sum(x['repeat'] for x in placed.values() if x['how'] == 'chroma fit'),
            after_fitted=sum(x['repeat'] for x in placed.values() if x['how'].startswith('after fitted')),
            not_localized=sum(x['repeat'] for x in placed.values() if x['how'] == 'NOT LOCALIZED'),
            flags=flags)
    with (J / 'summary.json').open('x') as f:
        json.dump(clean(summary), f, indent=2, allow_nan=False)


if __name__ == '__main__':
    main()
