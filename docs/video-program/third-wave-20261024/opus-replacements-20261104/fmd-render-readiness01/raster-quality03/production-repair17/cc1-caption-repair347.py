# cc1 caption repair 347: a caption-only candidate built from the native 327 captions.
#
# Reads the 327 SRT/VTT, the rebind269 report.json, the 15 timeline269 tables and script.tsv (all pinned
# by sha256), and the hand-authored overrides JSON. Writes new files only ('x' mode), each named
# PREFIX + suffix: candidate.en.srt, candidate.en.vtt, changes.tsv, checks.json. No audio, source, timeline,
# Rust or engine file is read for writing; nothing on any host is touched.
#
# Timing follows explainer_kit captions.rs (28e1aa51) exactly: recorded sentence spans from the report's
# sample counts (parse_report, sentence_spans), each sentence's span shared between its captions in
# proportion to their characters (sentence_captions), cue offsets from the timeline tables (video_captions)
# and prediction holds from the timelines' "predict" marks (prediction_end). Positive control: the same
# code fed the 327 caption groups must reproduce all 571 327 stamps, or the script stops. Split times
# inside a sentence remain character-share estimates, not word timing.
#
# Usage: python3 -I -B cc1-caption-repair347.py INPUTS SCRIPT OVERRIDES PREFIX
#   INPUTS: folder with complex_analysis_visual.en.srt/.vtt (327), report.json (rebind269) and the 15
#           KEY.timeline.tsv (timeline269); SCRIPT: complex_analysis_visual/script.tsv (build268 copy)
import hashlib, importlib.util, json, math, sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = Path.cwd()  # run from the repository root, which holds the two pinned checkers
if not (ROOT/'AGENTS.md').is_file():
    sys.exit('refused: run from the repository root')
VERIFY = ROOT/'docs/video-program/third-wave-20261024/opus-replacements-20261104/logarithm-stage09-final-verification02/verify_stage09_final.py'
SCREEN = ROOT/'narration_lab/third-wave-20261024/root/six-machine-screened-production-20261103/caption-interval-audit01.py'
PIN = {
    'complex_analysis_visual.en.srt': '5d06423248f088a34774f361721bf05e1c4950ac834086e3666661e1adfe5e1b',
    'complex_analysis_visual.en.vtt': '26e0d7e0eec26eb6f9136bb56f795a6ff8ab9cead26f29fc76a75e9abc4bb2e3',
    'report.json': 'b4a4c91e8e8fd26dbad621d118b762660e60532a2bbcf535ad88b87a45406bdd',
    'script.tsv': '234f725781bde0bdf35afbc21fd2d6ec0f73f99a95723294259cc1b75a813d3c',
    'verify': 'f266a2315e4a9af247c648da416881f3a2999e2682e9c611ff82ca3727b70b94',
    'screen': '3cc2643615ce5263b01d429ae6f72326c95e777af3e7f29ef9266a05155fa0c4',
}
TIMELINES = {  # timeline269 tables in registry order, byte-equal to the 327 job's 15 symlink targets
    '01_the_bump': '6149d6fa5e0e7086f1a61a727b0d9492bf32952815e378ffb82dd6cb973197ef',
    '02_turning_numbers': 'd6786da2a0b27058798dba8f2c3428186dbeb10b01a4368150dc15981fac2a0d',
    '03_two_planes': '98960c5350162890005782a7e254b5cac5cf1b38bcb0ff124012041d4db288cb',
    '04_reading_color': '438587cb208d2bdf5fee9510ea2c7c57af7a1247d8a140e023f4edd9ce48edb4',
    '05_gallery': 'df3cdeae1dab98a6b28cfde7ac2e30ae0a7242de9bda6ad9b0659fb10e52a260',
    '06_one_derivative': '8abb0519b2945269f5def975e1adde7968905fab45f72b85b242dc4a11d0ca71',
    '07_one_point': '5bbc64ab95fe36169b1dee002c6ddc8222a7c1bbb32758c8e812b66b42be97bb',
    '08_loop_sums': 'e2276be4d8b190bddf5c944e82c00c4b4febc6d65886142b890b99594ef64945',
    '09_boundary': 'e05a6679f8105d7bec1647088c276d52871753770a3af4fbc99c3e3af5bffb76',
    '10_rigidity': '99fd22e62d5c6e53104ec3c622135d0748c3dc68e375141d3da4a38114d5860b',
    '11_counting': '29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d',
    '12_residues': 'd25e8b958cd32f2a233b38c293183f3d39b2d1502137c389c69ba49ff0d0dca7',
    '13_real_integral': '54995e05de124d1462cb9841ac0d14e6abbc351583c5743d2287ef24c44fd505',
    '14_continuation': '2b3a5bff177889f00fb06871a8910d98babb4ac3c4636f50758c897cb11fbc99',
    '15_closing': '1d0532d139899817358b829a095ac31f258833fb61aef314f2a758be7e38ebc7'}
FPS, TOTAL_FRAMES, LINE_CHARS, MAX_LINES, MAX_CPS, MIN_DUR = 60, 133662, 42, 2, 20.0, 5 / 6

# Boundary rules (task 347). EXPR words are numbers, variables and operators of this script.
NUMW = set('zero one two three four five six seven eight nine ten eleven twelve thirteen fourteen fifteen sixteen '
           'seventeen eighteen nineteen twenty thirty forty fifty sixty seventy eighty ninety hundred thousand '
           'million half'.split())
VARS = set('z x y R p n m h a c s f g u v i e pi epsilon'.split())
OPS = set('plus minus times over squared cubed point prime'.split())
EXPR = NUMW | VARS | OPS
FUNC_END = set('the a an of to at by in on and or but so that which with from is are was will has have its '
               'their minus plus point over times though then as if into than for through no every each whose '
               'where when while because both all some any this these those'.split())
ARTICLES = {'the', 'a', 'an'}

def sha(p):
    with open(p, 'rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()
def load(path, name, digest):
    if sha(path) != digest:
        sys.exit(f'refused: {path} is not {digest[:12]}')
    spec = importlib.util.spec_from_file_location(name, path)
    m = importlib.util.module_from_spec(spec); spec.loader.exec_module(m); return m
def tok(w):
    s = w.strip('.,?!:;')
    return s if s == 'R' else s.lower()
def separated(w):
    return w[-1] in ',.;:?!'
def expr_break(prev, nxt, nxt2):
    """Why a break between prev | nxt (nxt2 = the word after nxt) splits an expression."""
    a, b, out = tok(prev), tok(nxt), []
    if not separated(prev) and a in EXPR and b in EXPR:
        out.append('number' if a in NUMW - {'half'} and b in NUMW - {'half'} else 'expr')
    if b in ('squared', 'cubed', 'prime'):
        out.append('power')
    if not separated(prev) and a in EXPR and b == 'to' and tok(nxt2 or '') == 'the':
        out.append('exponent')
    if a == 'square' and b == 'root':
        out.append('square-root')
    return out
def rules(groups):
    """Rule failures of one sentence's (or merged unit's) caption groups, each with its F code."""
    out = []
    flat = [(gi, li, w) for gi, g in enumerate(groups) for li, line in enumerate(g) for w in line.split()]
    for x in range(len(flat) - 1):
        gi, li, w = flat[x]; gj, lj, w2 = flat[x + 1]
        w3 = flat[x + 2][2] if x + 2 < len(flat) else None
        if gi != gj:
            if tok(w) in FUNC_END and not separated(w):
                out.append(('C1', 'F2' if tok(w) in OPS else 'F3', f'caption ends on "{w}" before "{w2}"'))
            for r in expr_break(w, w2, w3):
                out.append(('C2', 'F1' if r == 'number' else 'F2', f'{r} split across captions "{w} | {w2}"'))
        elif li != lj:
            for r in expr_break(w, w2, w3):
                out.append(('L2', 'F4', f'{r} split across lines "{w} / {w2}"'))
            if tok(w) in ARTICLES and not separated(w):  # "f of a," ends on the variable a
                out.append(('L3', 'F4', f'line ends on article "{w}"'))
    for g in groups:
        if len(g) == 2 and min(len(g[0]), len(g[1])) < 9:
            out.append(('L1', 'F4', f'short line in {" / ".join(g)!r}'))
    return out
def weight(lines):
    return sum(len(l) for l in lines)
def share(groups, start, end, weights=None):
    """explainer_kit sentence_captions: [start, end) shared in proportion to caption characters."""
    w = weights or [float(weight(g)) for g in groups]
    total, t, out = sum(w), start, []
    for g, x in zip(groups, w):
        e = t + (end - start) * x / total
        out.append([t, e, g]); t = e
    return out
def ms(t):
    return math.floor(max(t, 0.0) * 1000.0 + 0.5)  # f64::round on a non-negative value
def stamp(m, dec):
    return f'{m // 3600000:02}:{m // 60000 % 60:02}:{m // 1000 % 60:02}{dec}{m % 1000:03}'
def to_srt(caps):
    return ''.join(f'{k}\n{stamp(a, ",")} --> {stamp(b, ",")}\n' + '\n'.join(l) + '\n\n' for k, (a, b, l) in enumerate(caps, 1))
def to_vtt(caps):
    return 'WEBVTT\n\n' + ''.join(f'{stamp(a, ".")} --> {stamp(b, ".")}\n' + '\n'.join(l) + '\n\n' for a, b, l in caps)
def parse_srt(text):
    caps = []
    for block in text.strip('\n').split('\n\n'):
        rows = block.split('\n')
        a, b = rows[1].split(' --> ')
        def back(s):
            h, m, rest = s.split(':'); sec, milli = rest.split(',')
            return ((int(h) * 60 + int(m)) * 60 + int(sec)) * 1000 + int(milli)
        caps.append((back(a), back(b), rows[2:]))
    return caps

def main():
    INPUTS, SCRIPT, OVERRIDES, PREFIX = Path(sys.argv[1]), Path(sys.argv[2]), Path(sys.argv[3]), sys.argv[4]
    targets = {s: Path(PREFIX + s) for s in ('candidate.en.srt', 'candidate.en.vtt', 'changes.tsv', 'checks.json')}
    if any(p.exists() for p in targets.values()):
        sys.exit('refused: an output already exists')
    for name in ('complex_analysis_visual.en.srt', 'complex_analysis_visual.en.vtt', 'report.json'):
        if sha(INPUTS/name) != PIN[name]:
            sys.exit(f'refused: {name} is not the pinned 327/rebind269 input')
    if sha(SCRIPT) != PIN['script.tsv']:
        sys.exit('refused: script.tsv is not 234f7257')
    for key, digest in TIMELINES.items():
        if sha(INPUTS/f'{key}.timeline.tsv') != digest:
            sys.exit(f'refused: {key}.timeline.tsv is not timeline269')
    V = load(VERIFY, 'v347', PIN['verify']); S = load(SCREEN, 's347', PIN['screen'])
    script = V.read_script(SCRIPT)
    report = json.loads((INPUTS/'report.json').read_text())
    rate = float(report['meta']['rate'])
    srt0_text = (INPUTS/'complex_analysis_visual.en.srt').read_text()
    vtt0_text = (INPUTS/'complex_analysis_visual.en.vtt').read_text()
    orig = parse_srt(srt0_text)
    if to_vtt(orig) != vtt0_text or to_srt(orig) != srt0_text:
        sys.exit('refused: the 327 SRT/VTT do not round-trip through this writer')

    # cue placements and holds, as video_captions computes them
    cues, offset, placements, frame_offset = [], 0.0, [], 0
    for key in TIMELINES:
        t = V.parse_timeline((INPUTS/f'{key}.timeline.tsv').read_text(), key)
        assert t['chapter'] == key and t['fps'] == FPS
        for j, line in enumerate(t['lines']):
            rows = report['lines'][line['id']]
            dur = sum(r['out_samples'] + r['gap_after_samples'] for r in rows) / rate
            cue_end = line['start'] / FPS + dur
            hold = None
            if j + 1 < len(t['lines']):
                nxt = t['lines'][j + 1]['start'] / FPS
                if any(m['label'] == 'predict' and cue_end <= m['frame'] / FPS < nxt for m in t['marks']):
                    hold = offset + nxt
            cursor, spans = 0, []
            for r in rows:
                spans.append((cursor / rate, (cursor + r['out_samples']) / rate))
                cursor += r['out_samples'] + r['gap_after_samples']
            cues.append(dict(cue=line['id'], chapter=key, at=offset + line['start'] / FPS, hold=hold,
                             sentences=[r['sentence'] for r in rows], spans=spans))
            placements.append(dict(cue=line['id'], chapter=key, start_frame=frame_offset + line['start'], duration_s=dur))
        offset += t['frames'] / FPS; frame_offset += t['frames']
    if frame_offset != TOTAL_FRAMES or [c['cue'] for c in cues] != list(script):
        sys.exit('refused: timelines do not place the script')

    # the 327 captions grouped by sentence
    k = 0
    for c in cues:
        c['orig'] = []
        for s in c['sentences']:
            words, got, g = s.split(), [], []
            while len(got) < len(words):
                g.append(orig[k]); got += ' '.join(orig[k][2]).split(); k += 1
            if got != words:
                sys.exit(f'refused: 327 captions do not hold {c["cue"]} sentence words')
            c['orig'].append(g)
    assert k == len(orig)

    def timed(c, i, groups, merge_prefix=0):
        """Kit timing for sentence i of cue c with these groups. With merge_prefix (the answer's
        character count plus its space), groups[0] begins with the previous sentence (the answer):
        the shares are over sentence i's words only, and the first caption starts at the answer's start."""
        a, b = c['spans'][i]
        weights = [float(weight(g)) for g in groups]
        weights[0] -= merge_prefix
        out = share(groups, a, b, weights)
        if merge_prefix:
            out[0][0] = c['spans'][i - 1][0]
        for x in out:
            x[0] += c['at']; x[1] += c['at']
        if i == len(c['sentences']) - 1 and c['hold'] is not None:
            out[-1][1] = c['hold']
        return [(ms(x[0]), ms(x[1]), x[2]) for x in out]

    # positive control: the kit rule on the 327 groups reproduces every 327 stamp
    rebuilt = [cap for c in cues for i, g in enumerate(c['orig']) for cap in timed(c, i, [x[2] for x in g])]
    if rebuilt != orig:
        bad = next(n for n, (x, y) in enumerate(zip(rebuilt, orig), 1) if x != y)
        sys.exit(f'refused: timing model does not reproduce 327 caption {bad}')

    doc = json.loads(OVERRIDES.read_text())
    assert doc['schema'] == 'cc1-caption-repair347-overrides-v1'
    by_cue = {c['cue']: c for c in cues}
    plan = {}  # (cue, first sentence) -> override
    for o in doc['overrides']:
        c = by_cue[o['cue']]
        if [c['sentences'][i] for i in o['sentences']] != o['texts']:
            sys.exit(f'refused: override {o["cue"]} {o["sentences"]} text is not the report sentence')
        if ' '.join(' '.join(g) for g in o['captions']).split() != ' '.join(o['texts']).split():
            sys.exit(f'refused: override {o["cue"]} {o["sentences"]} words differ')
        for i in o['sentences']:
            if any(i in (p[1], p[1] + len(plan[p]['sentences']) - 1) for p in plan if p[0] == o['cue']):
                sys.exit('refused: overlapping overrides')
        plan[(o['cue'], o['sentences'][0])] = o
    exceptions = {(e['cue'], e['sentence'], e['rule']) for e in doc['exceptions']}

    cand, changes, units = [], [], []
    for c in cues:
        i = 0
        while i < len(c['sentences']):
            o = plan.get((c['cue'], i))
            g0 = c['orig'][i] if not o else [x for j in o['sentences'] for x in c['orig'][j]]
            if not o:
                caps = timed(c, i, [x[2] for x in g0]); kind, ids = 'kept', [i]
            elif len(o['sentences']) == 1:
                caps = timed(c, i, o['captions']); kind, ids = 'resplit', [i]
            else:
                ans = c['sentences'][i]
                if not (' '.join(o['captions'][0]).startswith(ans + ' ') and o['captions'][0][0].startswith(ans + ' ')):
                    sys.exit(f'refused: merge {c["cue"]} must open its first line with the whole answer')
                caps = timed(c, i + 1, o['captions'], merge_prefix=len(ans) + 1); kind, ids = 'merged', o['sentences']
            units.append(dict(cue=c['cue'], sentences=ids, kind=kind, first=len(cand) + 1, caps=caps,
                              orig=g0, failures_327=rules([x[2] for x in g0]),
                              failures=rules([x[2] for x in caps])))
            cand.extend(caps); i += len(ids)

    srt, vtt = to_srt(cand), to_vtt(cand)
    checks = {}
    # pinned checks on the candidate
    try:
        checks['check_captions_f266a231'] = V.check_captions(srt, vtt, script, report, placements, TOTAL_FRAMES)
    except V.Refused as error:
        checks['check_captions_f266a231'] = dict(refused=str(error))
    # the screen reads a path; give it the candidate text through a temporary file next to the outputs
    tmp = Path(PREFIX + 'candidate.en.srt')
    with tmp.open('x') as f:
        f.write(srt)
    screen = S.screen(tmp, TOTAL_FRAMES, FPS)
    checks['screen_3cc26436'] = dict(caption_count=screen['caption_count'], fastest=screen['fastest'],
                                     over_20=len(screen['over_20_characters_per_second']),
                                     lines_over_42=len(screen['lines_over_42_characters']))
    # structure (kit validate) and timing
    durs = [b - a for a, b, _ in cand]
    cps = [len(' '.join(' '.join(l).split())) * 1000 / (b - a) for a, b, l in cand]
    checks['structure'] = dict(
        captions=len(cand), captions_327=len(orig),
        start_before_end=all(a < b for a, b, _ in cand),
        lines_1_or_2=all(1 <= len(l) <= MAX_LINES for _, _, l in cand),
        line_chars_max=max(len(x) for _, _, l in cand for x in l),
        no_overlap=all(cand[n][0] >= cand[n - 1][1] for n in range(1, len(cand))),
        min_duration_ms=min(durs), max_cps=round(max(cps), 3),
        srt_equals_vtt=parse_srt(srt) == [(a, b, l) for a, b, l in cand] and to_vtt(parse_srt(srt)) == vtt)
    # exact text: whole video and per cue, against the 327 captions and the report
    checks['text'] = dict(
        whole_equals_327=' '.join(w for _, _, l in cand for w in ' '.join(l).split()) == ' '.join(w for _, _, l in orig for w in ' '.join(l).split()),
        per_cue_equals_report=all(' '.join(w for u in units if u['cue'] == c['cue'] for x in u['caps'] for w in ' '.join(x[2]).split()) == ' '.join(' '.join(c['sentences']).split()) for c in cues))
    # sentence and cue boundaries against the 327 timing
    sent, merged = [], []
    for u in units:
        o_first, o_last = u['orig'][0][0], u['orig'][-1][1]
        sent.append(u['caps'][0][0] == o_first and u['caps'][-1][1] == o_last)
        if u['kind'] == 'merged':
            c = by_cue[u['cue']]; ans_start = u['orig'][0][0]
            prev_end = cand[u['first'] - 2][1]
            follow_start_ms = ms(c['at'] + c['spans'][u['sentences'][1]][0])
            merged.append(dict(cue=u['cue'], answer=c['sentences'][u['sentences'][0]], first_caption=u['first'],
                               starts_at_answer_start=u['caps'][0][0] == ans_start,
                               starts_at_cue_placement=u['caps'][0][0] == ms(c['at']),
                               previous_caption_ends=prev_end, previous_is_held_question=prev_end == ans_start,
                               following_sentence_recorded_start_ms=follow_start_ms,
                               following_shown_from_ms=u['caps'][0][0],
                               following_lead_ms=follow_start_ms - u['caps'][0][0],
                               unit_ends_at_327_end=u['caps'][-1][1] == o_last))
    cov = []
    for c in cues:
        cu = [x for u in units if u['cue'] == c['cue'] for x in u['caps']]
        o = [x for g in c['orig'] for x in g]
        cov.append(cu[0][0] == o[0][0] and cu[-1][1] == o[-1][1] and all(cu[n][0] == cu[n - 1][1] for n in range(1, len(cu))))
    holds = [(c['cue'], [x for g in c['orig'] for x in g][-1], [x for u in units if u['cue'] == c['cue'] for x in u['caps']][-1]) for c in cues if c['hold'] is not None]
    checks['boundaries'] = dict(
        units=len(units), sentence_or_unit_start_and_end_equal_327=all(sent), merges=merged,
        cues_same_first_start_last_end_and_contiguous=sum(cov), cues=len(cov),
        prediction_holds=len(holds),
        holds_end_unchanged=all(x[1] == y[1] for _, x, y in holds),
        held_captions_end_with_the_question=all(' '.join(y[2]).endswith('?') for _, _, y in holds),
        kept_units_byte_identical=all(u['caps'] == [tuple(x) for x in u['orig']] for u in units if u['kind'] == 'kept'))
    # semantic boundary rules
    fail = [(u['cue'], s, r) for u in units for s in u['sentences'][:1] for r in u['failures']]
    unexpected = [f for f in fail if (f[0], f[1], f[2][0]) not in exceptions]
    checks['semantic_rules'] = dict(
        failures_327_by_code={c: sum(1 for u in units for r in u['failures_327'] if r[1] == c) for c in ('F1', 'F2', 'F3', 'F4')},
        units_failing_327=sum(1 for u in units if u['failures_327']),
        failures_candidate=[dict(cue=a, sentence=b, rule=r[0], code=r[1], detail=r[2]) for a, b, r in fail],
        unexpected_candidate_failures=len(unexpected),
        failing_327_units_not_changed=[(u['cue'], u['sentences']) for u in units if u['failures_327'] and u['kind'] == 'kept'],
        exceptions=doc['exceptions'])
    checks['durations'] = dict(
        candidate_min_ms=min(durs), candidate_under_833ms=[n for n, d in enumerate(durs, 1) if d < 833],
        candidate_under_1000ms=[(n, d, ' '.join(cand[n - 1][2])) for n, d in enumerate(durs, 1) if d < 1000],
        changed_caption_min_ms=min(b - a for u in units if u['kind'] != 'kept' for a, b, _ in u['caps']))
    passed = (checks['check_captions_f266a231'].get('text_equals_script') == 'whole and per cue'
              and checks['check_captions_f266a231'].get('srt_equals_vtt') is True
              and checks['screen_3cc26436']['over_20'] == 0 and checks['screen_3cc26436']['lines_over_42'] == 0
              and all(v for k2, v in checks['structure'].items() if isinstance(v, bool))
              and checks['structure']['line_chars_max'] <= LINE_CHARS and checks['structure']['max_cps'] <= MAX_CPS
              and checks['structure']['min_duration_ms'] >= 833
              and all(checks['text'].values())
              and checks['boundaries']['sentence_or_unit_start_and_end_equal_327']
              and all(m['starts_at_answer_start'] and m['starts_at_cue_placement'] and m['previous_is_held_question']
                      and m['following_lead_ms'] >= 0 and m['unit_ends_at_327_end'] for m in merged)
              and checks['boundaries']['cues_same_first_start_last_end_and_contiguous'] == len(cues)
              and checks['boundaries']['holds_end_unchanged'] and checks['boundaries']['held_captions_end_with_the_question']
              and checks['boundaries']['kept_units_byte_identical']
              and not unexpected)
    checks['all_passed'] = bool(passed)
    checks['inputs'] = dict(PIN, timelines=TIMELINES, overrides_sha256=sha(OVERRIDES))
    checks['limits'] = ('Caption files only. Split times inside a sentence are character-share estimates of the '
                        'kit rule, not word timing; no listening, ASR, sync or accessibility acceptance.')
    with targets['candidate.en.vtt'].open('x') as f:
        f.write(vtt)
    rows = ['unit\tkind\tside\tcaption\tstart\tend\tdur_s\tcps\ttext\tcodes\tdetail']
    for u in units:
        if u['kind'] == 'kept':
            continue
        key = f"{u['cue']}:{'+'.join(map(str, u['sentences']))}"
        o_first = orig.index(u['orig'][0]) + 1
        codes = ','.join(sorted({r[1] for r in u['failures_327']} | ({'F5'} if u['kind'] == 'merged' else set())))
        detail = '; '.join(r[2] for r in u['failures_327'])
        for n, (a, b, l) in enumerate(u['orig']):
            rows.append(f"{key}\t{u['kind']}\t327\t{o_first + n}\t{stamp(a, '.')}\t{stamp(b, '.')}\t{(b - a) / 1000:.3f}\t"
                        f"{len(' '.join(l)) * 1000 / (b - a):.1f}\t{' / '.join(l)}\t{codes if n == 0 else ''}\t{detail if n == 0 else ''}")
        for n, (a, b, l) in enumerate(u['caps']):
            rows.append(f"{key}\t{u['kind']}\t347\t{u['first'] + n}\t{stamp(a, '.')}\t{stamp(b, '.')}\t{(b - a) / 1000:.3f}\t"
                        f"{len(' '.join(l)) * 1000 / (b - a):.1f}\t{' / '.join(l)}\t\t")
    with targets['changes.tsv'].open('x') as f:
        f.write('\n'.join(rows) + '\n')
    checks['outputs'] = {s: sha(p) for s, p in targets.items() if p.exists()}
    with targets['checks.json'].open('x') as f:
        json.dump(checks, f, indent=1)
        f.write('\n')
    print(json.dumps(dict(all_passed=checks['all_passed'], structure=checks['structure'], screen=checks['screen_3cc26436'],
                          check_captions=checks['check_captions_f266a231'], text=checks['text'],
                          unexpected=len(unexpected), failures_327=checks['semantic_rules']['failures_327_by_code'],
                          durations={k2: v for k2, v in checks['durations'].items() if k2 != 'candidate_under_1000ms'}),
                     indent=1))
    for f_ in fail:
        print('candidate rule failure', f_)
    for m in merged:
        print('merge', m)

if __name__ == '__main__':
    main()
