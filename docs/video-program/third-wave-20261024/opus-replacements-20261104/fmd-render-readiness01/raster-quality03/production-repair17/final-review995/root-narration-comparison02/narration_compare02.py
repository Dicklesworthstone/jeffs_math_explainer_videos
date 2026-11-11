"""narration_compare02.py: corrected comparison of the chapter 01 and 08 narration, CSS
css-repair-delivery309 against the CSD folders, from the read-only probe outputs (narr_probe01.py).

Correction over narration_compare01.py (kept unchanged, with its output): 01 selected phrase timings
from the TOP level of phrase-timings.json, but the cue map is nested under pt["cues"], so 01 compared
two empty dicts and its phrase-timing equality was vacuous (root found this). 02 selects pt["cues"],
fixes the denominator at the eleven expected cues, and refuses a field that is empty or missing any
expected cue in any folder. A cue whose marks list is empty is allowed only when it is empty in every
folder, and it is listed. Mutated controls built from the actual CSD data must each be refused.

Usage: python3 -I -B narration_compare02.py CSD_PROBE_JSON CSS_PROBE_JSON OUT_JSON
OUT_JSON is created with open('x'). Exit 0 only if the actual data agree on every field and every
control is refused.
"""
import copy
import json
import sys

D = '/data/projects/jmev-stroke-probe-20261106/jobs/'
CSS309 = '/data/projects/jmev-css-render-worker-20261107/jobs/css-repair-delivery309/narration'
CSD = {'csd849': D + 'csd-native-delivery849/narration', 'csd269': D + 'csd-review-repair-rebind269/narration',
       'csd313': D + 'csd-review-repair-rebind313/narration', 'csd19': D + 'csd-production-assembly19/narration'}
EXPECTED = ['01_1', '01_2', '01_3', '01_4', '01_5', '01_6', '08_1', '08_2', '08_3', '08_4', '08_5']
FIELDS = ('wavs', 'report_rows', 'assembly_outputs', 'assembly_units', 'plan_units', 'phrase_cues')


class Refused(Exception):
    pass


def need(ok, why):
    if not ok:
        raise Refused(why)


def subset(name, folder):
    """Per-cue selection of every audio and timing field, keyed by the eleven expected cues."""
    j = folder['json']
    for f in ('report.json', 'assembly.json', 'external-units.json', 'phrase-timings.json'):
        need(f in j, f'{name}: {f} missing')
    rep, asm, plan, pt = j['report.json'], j['assembly.json'], j['external-units.json'], j['phrase-timings.json']
    need(isinstance(pt.get('cues'), dict) and pt['cues'], f'{name}: phrase-timings.json has no cue map')
    units_by_line = {}
    for u in plan['units']:
        for line in u['line_ids']:
            units_by_line[line] = u['id']
    asm_units = {u['id']: u for u in asm['manifest']['units']}
    plan_units = {u['id']: u for u in plan['units']}
    out = {k: {} for k in FIELDS}
    for cue in EXPECTED:
        wav = folder['files'].get(cue + '.wav')
        need(wav is not None and wav['sha256'], f'{name}: {cue}.wav missing')
        out['wavs'][cue] = wav['sha256']
        need(cue in rep['lines'] and rep['lines'][cue], f'{name}: report rows for {cue} missing')
        out['report_rows'][cue] = [{k: v for k, v in r.items() if k != 'source'} for r in rep['lines'][cue]]
        need(cue in asm['outputs'], f'{name}: assembly output for {cue} missing')
        out['assembly_outputs'][cue] = asm['outputs'][cue]
        uid = units_by_line.get(cue)
        need(uid is not None and uid in asm_units and uid in plan_units, f'{name}: unit for {cue} missing')
        out['assembly_units'][cue] = asm_units[uid]
        out['plan_units'][cue] = plan_units[uid]
        need(cue in pt['cues'], f'{name}: phrase timings for {cue} missing')
        out['phrase_cues'][cue] = pt['cues'][cue]
    for k in FIELDS:
        need(sorted(out[k]) == EXPECTED, f'{name}: {k} does not cover exactly the 11 expected cues')
    out['marks'] = {cue: len(out['phrase_cues'][cue].get('marks') or []) for cue in EXPECTED}
    out['phrase_file_sha256'] = folder['files']['phrase-timings.json']['sha256']
    out['phrase_meta'] = {k: v for k, v in pt.items() if k != 'cues'}
    out['dispositions'] = [r for r in asm['manifest']['direction_dispositions'] if r['cue'] in EXPECTED]
    out['files'] = {n: f['sha256'] for n, f in folder['files'].items()}
    return out


def compare(folders):
    """Every folder against css309, field by field and cue by cue. Raises Refused on any difference."""
    ref = folders['css309']
    rows = {}
    for name, f in folders.items():
        diffs = [f'{k}:{cue}' for k in FIELDS for cue in EXPECTED if f[k][cue] != ref[k][cue]]
        need(not diffs, f'{name} differs from css309 at ' + ', '.join(diffs))
        rows[name] = dict(fields_equal={k: len(f[k]) for k in FIELDS}, marks=f['marks'],
                          phrase_file_sha256=f['phrase_file_sha256'],
                          files_differing_from_css309=sorted(n for n in set(f['files']) | set(ref['files'])
                                                             if f['files'].get(n) != ref['files'].get(n)))
    empty = sorted(c for c in EXPECTED if ref['marks'][c] == 0)
    for name, f in folders.items():
        need(sorted(c for c in EXPECTED if f['marks'][c] == 0) == empty, f'{name}: empty-mark cues differ')
    need(sum(ref['marks'].values()) > 0, 'no phrase marks at all in the 01/08 cues')
    return rows, empty


def load(csd, css):
    folders = {'css309': subset('css309', css['narration'][CSS309])}
    for k, path in CSD.items():
        folders[k] = subset(k, csd['narration'][path])
    return folders


def controls(csd, css):
    """Mutations of the actual CSD 849 data. Each must be refused."""
    out = []

    def run(label, mutate):
        c = copy.deepcopy(csd)
        mutate(c['narration'][CSD['csd849']]['json'])
        try:
            compare(load(c, css))
            out.append(dict(control=label, refused=False))
        except Refused as e:
            out.append(dict(control=label, refused=True, reason=str(e)[:200]))

    cues = csd['narration'][CSD['csd849']]['json']['phrase-timings.json']['cues']
    marked01 = next(c for c in EXPECTED if c.startswith('01_') and cues[c].get('marks'))
    marked08 = next(c for c in EXPECTED if c.startswith('08_') and cues[c].get('marks'))

    def mark_sample(j):
        j['phrase-timings.json']['cues'][marked08]['marks'][0]['sample_24k'] += 1
    def mark_window(j):
        w = j['phrase-timings.json']['cues'][marked01]['marks'][0]['window_s']
        w[0] += 0.01
    def drop_cue(j):
        del j['phrase-timings.json']['cues']['01_6']
    def flatten(j):
        j['phrase-timings.json']['cues'] = {}
    def row_end(j):
        j['report.json']['lines']['08_5'][-1]['end'] += 0.02
    def wav_hash(j):
        j['assembly.json']['outputs']['01_2']['sha256'] = '0' * 64
    for label, fn in ((marked08 + ' first mark sample_24k + 1', mark_sample),
                      (marked01 + ' first mark window start + 0.01 s', mark_window),
                      ('01_6 phrase cue deleted', drop_cue), ('phrase cue map emptied', flatten),
                      ('08_5 last report row end + 0.02 s', row_end), ('01_2 assembly output sha256 zeroed', wav_hash)):
        run(label, fn)
    return out


def main():
    csd = json.load(open(sys.argv[1]))
    css = json.load(open(sys.argv[2]))
    folders = load(csd, css)
    rows, empty = compare(folders)
    ctl = controls(csd, css)
    stems = ({n: v['sha256'] for n, v in csd['stems'].items()}, {n: v['sha256'] for n, v in css['stems'].items()})
    need(len(stems[0]) == 12 and stems[0] == stems[1], 'stems differ or not 12 files')
    ok = all(c['refused'] for c in ctl)
    ref = folders['css309']
    result = dict(
        expected_cues=EXPECTED, denominator=len(EXPECTED),
        marks_per_cue=ref['marks'], marks_total=sum(ref['marks'].values()), cues_without_marks=empty,
        wavs=ref['wavs'], folders=rows,
        phrase_timings_file_identical_in_all_folders=len({r['phrase_file_sha256'] for r in rows.values()}) == 1,
        stems=dict(files=stems[0], equal_across_hosts=True, csd_dir=csd['stems_dir'], css_dir=css['stems_dir']),
        controls=ctl, all_controls_refused=ok,
        correction='narration_compare01.py compared top-level phrase-timings keys, which are not cues, so its '
                   'phrase equality was vacuous. 02 compares pt["cues"] for the eleven cues.',
        scope='Byte hashes and JSON field equality only. No audio heard or checked; Frederick narration unchanged and '
              'not assessed.')
    with open(sys.argv[3], 'x') as f:
        json.dump(result, f, indent=1)
        f.write('\n')
    print('denominator', len(EXPECTED), 'cues; marks per cue', ref['marks'], 'total', result['marks_total'])
    print('cues without marks:', empty or 'none')
    for name, r in rows.items():
        print(name, 'all 6 fields equal to css309 over 11 cues; files differing:', r['files_differing_from_css309'])
    print('phrase-timings.json byte-identical in all five folders:', result['phrase_timings_file_identical_in_all_folders'])
    print('stems: 12 files equal across hosts')
    for c in ctl:
        print('control', c['control'], 'REFUSED' if c['refused'] else 'NOT REFUSED', c.get('reason', ''))
    print('RESULT:', 'PASS' if ok else 'FAIL')
    return 0 if ok else 1


if __name__ == '__main__':
    try:
        raise SystemExit(main())
    except Refused as e:
        print('REFUSED:', e)
        raise SystemExit(1)
