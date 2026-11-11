"""PREPARED, NOT LAUNCHED: guarded native final 4K render of one repaired chapter (01, 08, 09 or 11).

csd-native968-run.py's route with its 09 constants moved into CHAINS and CHAPTERS. Same base761 owner,
guard policy, 16 distinct physical cores, score21 stems and encoder settings. Option A: 01/08/09 render
with the combined01 binary and delivery (kind native-combined01), 11 with the counting02 binary and
delivery (kind native-counting02). Refuses while a pin is unresolved. No speed claim; no picture,
motion, listening or AV acceptance. It carries no review scope: per root M1, a later step derives each
row's scope only from the actual new extraction and viewing, never from master993's old intervals.

CC0 successor of root candidate01 (6dceac47): both binary pins are the actual qualification.json values,
and REVIEW_ROUTES (09 only, owner direction via root) states how 09's complete viewing is bound without a
whole-chapter root pass. Still not launched; root writes every admission.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import signal
import sys

D = Path('/data/projects/jmev-stroke-probe-20261106')
Q = D/'qualification01/production-repair17'
RUNNER = Q/'csd-native-repair-final01-run.py'
FINAL = 'csd-native-repair-final01-'
BASE761_SHA = '32bf6d9e08c1f21f2445debfd46ba090e0374007e4da5516310cb122774099e5'
POLICY = dict(admission_gib=10, stop_free_gib=8, phase_growth_gib=4,
              work_cap_gib=56, target_growth_cap_gib=20, sample_seconds=10)
STEMS = D/'jobs/csd-production-score21/score/stems'
SCENES = 'workspace/explainer_corpus/complex_analysis_visual/src/chapters'
UNRESOLVED = None

# The job chain each kind's final rests on. binary_sha256 is the actual build's qualification.json value
# (csd-native-combined-build01 and csd-native-counting-build02, both exit 0); run and launch refuse while
# a pin is UNRESOLVED.
CHAINS = {
    'native-combined01': dict(
        build='csd-native-combined-build01', binary='complex_analysis_visual-combined01', tests=247,
        binary_sha256='aed99576b7a9cc42353c43963e1828acf3465426e028fc424eaf6712133725ce',
        source='work/native-combined01',
        delivery='csd-native-combined-delivery01', delivery_record='delivery-combined01.json', repaired_cues=20,
        timeline='csd-native-combined-timeline01', preview='csd-native-combined-preview01',
        comparison='comparison-combined01.json',
        pictures='csd-native-combined-pictures01', pictures_inputs='csd-combined01-pictures-inputs.json',
        sampling='combined-pictures01'),
    'native-counting02': dict(
        build='csd-native-counting-build02', binary='complex_analysis_visual-counting02', tests=250,
        binary_sha256='b1bc47b40711d4cfb295339459def8e8fac054d0118bc8896f33a4fd49369be1',
        source='work/native-counting02',
        delivery='csd-native-counting-delivery02', delivery_record='delivery-counting02.json', repaired_cues=28,
        timeline='csd-native-counting-timeline02', preview='csd-native-counting-preview02',
        comparison='comparison-counting02.json',
        pictures='csd-native-counting-pictures02', pictures_inputs='csd-counting02-pictures-inputs.json',
        sampling='counting-pictures02'),
}

# Per chapter: frames; the staged scene file both reviews must name; the normalized timeline the
# timeline and preview jobs asserted (their EXPECTED); the narration-only capture of the chapter's
# earlier preview (preview274, or preview243 for 08), which the new narration-only preview must equal as
# 853's equalled f7f8aa60 in 968; the sampling list the extraction retained (plus prior775 for 09); and
# the score21 stems (09's are 968's own literals). staged_capture is master993's staged scored soundtrack
# of the chapter (11's read by root on CSD); it is data for the stage step's replay check, not checked here.
CHAPTERS = {
    '01_the_bump': dict(
        kind='native-combined01', frames=6311, scene='the_bump.rs',
        staged_capture='4d77c2d66720d67160cea0feec6d86a5e5ab54720641d4586050104a2d592e07',
        scene_sha256='90b51c2b30877911bbb5f5e14ee810268e14efb24e7c4e2a55f7f7e128c06ee7',
        timeline_normalized='150147fe5bc7cdb0a111061c4530a16b104d196e004ae29bcf210d1c9c32ac81',
        preview_pcm='79fc6e7dea2c95c385b976409225a472a2d7d69cdb2314a0d6af9053614c4e60',
        sampling_sha256='89dfb46a68bc078c124035a51232c10c617f36f191cc45f987f428a6754ba227',
        stems={
            '01_the_bump.music.f32.wav': 'bb98d1129b340d01b248fcee769aac26dc8a20ce3f3b808607661ed71199b51e',
            'qa/01_the_bump.json': 'db09df0b6547fc8113d0a741f5f1512bb8e28d7c3c7f3d73c33392f4ae478c8d',
        }),
    '08_loop_sums': dict(
        kind='native-combined01', frames=14959, scene='loop_sums.rs',
        staged_capture='d713402c0c7fba3c17705fd624d38598b9aabebe2ef6220db9dc8cfeecbb328b',
        scene_sha256='a8d62f560321ea6ebea3d3859acd9a3ce3e5dde4375ec0ab42c1a1ba114d61ef',
        timeline_normalized='43c8bab2f7ca61c8692e0f1686798d54e187dcfeeab0d03311c2d4c9a5f54812',
        preview_pcm='2c277faaaa88661eab7b533bcc495759051b94b42018aaa39a1fe098e568ba98',
        sampling_sha256='6b70dab93816d34205b2c6de4be16bc4b492cda73a5e85d175573aa889c94911',
        stems={
            '08_loop_sums.music.f32.parts.tsv': '60b99ae9ca8bf9588a14ba42226688904e7afcd656e85a76b00a03c30c5a947f',
            '08_loop_sums.music.f32.wav.part0000.wav': '4b0fd3d2f8a3fe5aa197c9c7a48f4ae0f6b28284d0db39f0ecf510614305229c',
            '08_loop_sums.music.f32.wav.part0001.wav': '5c69b487a5f19e3ed26a9db42d1a4cf0aa1d686da996f4964e89c75fa9b1766c',
            '08_loop_sums.music.f32.wav.part0002.wav': '1e0e6a8ca09a070fe0693066d94faeaca0aab19ecd5cdaf704eb3940c9efb864',
            'qa/08_loop_sums.json': 'c4874309718a2f51b72176baab139416c89ee581e44128b321eafcb8d4d03ef2',
        }),
    '09_boundary': dict(
        kind='native-combined01', frames=16504, scene='boundary.rs',
        staged_capture='29899f9e42721331e2abe9088c399bca3eb63ea3b50de45dc731cbb8c4404c6c',
        scene_sha256='4b689791748132bd14c3929d191bb78ae7752bf9c3332beb22f774931873e932',
        timeline_normalized='5a91b8f9e2e5b5d6223a49285c06039b1b43bd16c11cecdd163a8e2e545baa9e',
        preview_pcm='f7f8aa6042a9f05d1b25db4c789f7f6b44cb393445015c057f1e87d9b0c8df51',
        sampling_sha256='019e0d9d5b15b7a3228d12cbb0e64e9d7c01cc8b2b4dd6a97ba19bbb186b01c7',
        prior='csd-native855-prior775.json',
        prior_sha256='2a1864771d82f9edb436823e100a3c6b97f8421e272a87bd7c2a8c95c1b4cf99',
        stems={
            '09_boundary.music.f32.parts.tsv': 'db31e88404eb9af836c6575cf5b4178a292eaeceea283d717bf2a4a7166ffe2e',
            '09_boundary.music.f32.wav.part0000.wav': 'ea2207e2a92ea9751fc688878b0a781aeb2a44c79a5e0a3de78197005d508483',
            '09_boundary.music.f32.wav.part0001.wav': '492613e4cefe62bf730532c375a17fd8815e497e570d81f07b76c052989b0758',
            '09_boundary.music.f32.wav.part0002.wav': 'a57f58d245399bdbc8a615b9c3d85cefd1891449de2bccfa9a541dc506efb3de',
            'qa/09_boundary.json': '445daec7a3652361923c34c2a0e6e3e50b23a2b596d89f4744ccf3f350e6b58e',
        }),
    '11_counting': dict(
        kind='native-counting02', frames=11932, scene='counting.rs',
        staged_capture='82bdc443218a5a761c678244089bc8d983a01466c77a7efed23ea358a4ec490a',
        scene_sha256='21c5043a3339cfadb10024a2e8227a62a0b06674139eace1a0d9029a4e1aa0db',
        timeline_normalized='29a3afd345fa3a376e5c163d5bf5e01267a866e2305bdd27c6413a5bab46807d',
        preview_pcm='7abda95ef24883bd9b4ed2fde0890409b5739939e3cdfe8f4869b2f18b8dd4dc',
        sampling_sha256='25965a518b027075fa4b3f3ba4bf18c9faa5d13b0da7026f2f178d6b0a4ba94c',
        stems={
            '11_counting.music.f32.parts.tsv': '6a47b284cd8d1d2d653a16be6207caf92f5dc742ba88b9aebe1d4a18a490cb13',
            '11_counting.music.f32.wav.part0000.wav': 'ee600304abeecc661a8cc7bfc602a4f059f18a7ca9d6769c310f12af620de10e',
            '11_counting.music.f32.wav.part0001.wav': 'ac2b494b88316b7b48ba74bd4bde1a29ec17da0f801b21a7d657da82122d94fd',
            'qa/11_counting.json': '65346d774cb7290f594479fb9d8d095fecc141dc238714d813c1a18e32ca2e81',
        }),
}

# Owner direction (root, 2026-11-11): no redundant whole-chapter root pass for 09. For a chapter listed here
# the admission carries review_route and author_review. Every selected picture was viewed by the independent
# reviewer (exact frames and sheets) and by the repair's author; root_review is root's explicit decision
# that binds exactly those two records and the extraction by sha and states
# coverage.root_whole_chapter_review false. Root's own targeted viewing is never counted as whole.
# Every other chapter keeps the original whole root review.
REVIEW_ROUTES = {'09_boundary': 'independent-and-author-complete-root-targeted'}

def sha(p):
    with Path(p).open('rb') as f:
        return hashlib.file_digest(f, 'sha256').hexdigest()

def read(p):
    return json.loads(Path(p).read_text())

def success(p):
    d = read(p)
    assert d.get('admitted') and d.get('finished') and d.get('exit') == 0, p
    assert d['inputs_sha256'] == d['final_inputs_sha256'], p
    assert not any(d.get(k) for k in ('resource_stop', 'stopped_for_resource_limit', 'error')), p
    return d

def job(chapter):
    return D/'jobs'/(FINAL+chapter.replace('_', '-'))

def admission(chapter):
    # Root authors this from actual root and independent viewing of the chapter's preview extraction.
    return Q/(FINAL+chapter.replace('_', '-')+'-picture-admission.json')

def unresolved(chapter):
    chain = CHAINS[CHAPTERS[chapter]['kind']]
    return [chain['build']+'/qualification.json binary_sha256'] if chain['binary_sha256'] is UNRESOLVED else []

def review_checks(chapter, review, root, independent, author, pictures, extraction_sha256):
    """The admission's evidence checks: 968's for every chapter, and REVIEW_ROUTES for a listed chapter.
    Pure: reads no file. author is None unless the chapter has a route."""
    spec = CHAPTERS[chapter]
    route = REVIEW_ROUTES.get(chapter)
    assert review.get('review_route') == route, chapter+': review_route'
    assert (author is None) == (route is None), chapter+': author_review'
    frames = pictures['selected_frames']
    for evidence in (root, independent) + ((author,) if route else ()):
        assert evidence['movie_sha256'] == pictures['movie_sha256']
        assert evidence['extraction_sha256'] == extraction_sha256
        assert evidence['source_sha256'] == spec['scene_sha256']
        assert evidence['concrete_defects'] == []
        assert evidence['safe_to_advance_to_final4k'] is True
    sheets = [item['path'] for item in pictures['sheets']]
    if route is None:
        assert root['viewed_frame_indices'] == frames
        assert root['viewed_sheets'] == sheets
    else:
        assert independent['viewed_frame_indices'] == frames and independent['viewed_sheets'] == sheets
        assert author['authored_repair'] is True and author['viewed_frame_indices'] == frames
        assert root['chapter'] == chapter and root['independent_blockers'] == []
        coverage = root['coverage']
        assert coverage['root_whole_chapter_review'] is False
        assert coverage['independent_selected_frames'] == coverage['author_selected_frames'] == len(frames)
        for key in ('independent_review', 'author_review', 'extraction_record'):
            assert root[key]['sha256'] == review[key]['sha256'], key
    assert independent['authored_repair'] is False
    assert independent['viewed_frame_indices']
    assert set(independent['viewed_frame_indices']) <= set(frames)

def render_args(chapter, binary, narration, out):
    return [binary, 'render', chapter, '--res', '3840x2160', '--fps', '60',
            '--threads', '16', '--narration', narration,
            '--score', STEMS, '--out', out, '--lint', '--crf', '16', '--preset', 'veryfast']

def proofs(chapter):
    """968's proofs for one chapter, read-only: chain receipts, binary, schedule, narration-only preview PCM,
    actual picture admission and stems. Returns (binary, narration, every file read)."""
    spec = CHAPTERS[chapter]
    chain = CHAINS[spec['kind']]
    pending = unresolved(chapter)
    assert not pending, chapter+': unresolved pin(s), refusing: '+', '.join(pending)
    files = set()
    def use(p):
        p = Path(p)
        files.add(p)
        return p
    jobs = {k: D/'jobs'/chain[k] for k in ('build', 'delivery', 'timeline', 'preview', 'pictures')}
    for name in ('build', 'delivery', 'timeline', 'preview', 'pictures'):
        success(use(jobs[name]/'resource-receipt.json'))
    binary = use(jobs['build']/chain['binary'])
    qualification = read(use(jobs['build']/'qualification.json'))
    assert qualification['binary'] == str(binary)
    assert sha(binary) == qualification['binary_sha256'] == chain['binary_sha256']
    assert qualification['tests_passed'] == chain['tests'] and qualification['clippy_passed'] is True
    assert qualification['explicit_native_plate_fade_probe_passed'] is True
    assert sha(use(D/chain['source']/SCENES/spec['scene'])) == spec['scene_sha256']
    delivery = read(use(jobs['delivery']/chain['delivery_record']))
    assert delivery['wavs_byte_identical'] == 105
    assert delivery['delivery']['repaired']['selected_cues'] == chain['repaired_cues']
    narration = jobs['delivery']/'narration'
    assert len(list(narration.glob('*.wav'))) == 105
    # The actual timeline and preview tables passed the exact normalized schedule check (CC0 handoff02:
    # read from the jobs' own records, never inferred from source).
    timeline = read(use(jobs['timeline']/chain['comparison']))['chapters'][chapter]
    previewed = read(use(jobs['preview']/chain['comparison']))['chapters'][chapter]
    for check in (timeline, previewed):
        assert check['frames'] == spec['frames'] and check['normalized_sha256'] == spec['timeline_normalized']
    table = use(jobs['preview']/'preview'/(chapter+'.timeline.tsv'))
    assert sha(table) == previewed['actual_sha256'] == timeline['actual_sha256']
    assert sha(use(jobs['timeline']/'timeline-output'/(chapter+'.timeline.tsv'))) == timeline['actual_sha256']
    pcm = use(jobs['preview']/'preview/soundtrack'/chapter/'soundtrack_0.wav')
    assert sha(pcm) == spec['preview_pcm']
    # Do not infer picture acceptance from the extractor's successful exit.
    review = read(use(admission(chapter)))
    assert review['chapter'] == chapter
    assert review['preview_job'] == chain['preview']
    assert review['extraction_job'] == chain['pictures']
    assert review['all_selected_pictures_viewed'] is True
    assert review['concrete_defects'] == [] and review['independent_blockers'] == []
    assert review['safe_to_advance_to_final4k'] is True
    route = REVIEW_ROUTES.get(chapter)
    for key in ('root_review', 'independent_review', 'extraction_record') + (('author_review',) if route else ()):
        item = review[key]
        assert sha(use(item['path'])) == item['sha256']
    extraction = jobs['pictures']/chapter/'pictures.json'
    assert review['extraction_record']['path'] == str(extraction)
    pictures = read(extraction)
    movie = use(jobs['preview']/'preview'/(chapter+'.mp4'))
    assert pictures['movie'] == str(movie) and pictures['movie_sha256'] == sha(movie)
    assert pictures['full_decode'] is True and pictures['frames'] == spec['frames']
    assert pictures['tables']['timeline.tsv'] == sha(table)
    frames = pictures['selected_frames']
    assert frames == sorted(set(frames))
    assert len(pictures['pictures']) == len(frames) and len(pictures['sheets']) == (len(frames)+7)//8
    sampling = use(Q/chain['sampling']/('frames-'+chapter+'-all.txt'))
    assert sha(sampling) == spec['sampling_sha256']
    required = {int(x) for x in sampling.read_text().split()}
    if 'prior' in spec:
        prior = use(Q/spec['prior'])
        assert sha(prior) == spec['prior_sha256']
        required |= set(read(prior)['selected_frames'])
    assert required <= set(frames), 'the extraction dropped retained samples'
    for item in pictures['pictures'] + pictures['sheets']:
        assert sha(use(item['path'])) == item['sha256']
    root = read(Path(review['root_review']['path']))
    independent = read(Path(review['independent_review']['path']))
    author = read(Path(review['author_review']['path'])) if route else None
    review_checks(chapter, review, root, independent, author, pictures, sha(extraction))
    assert sorted(p.name for p in STEMS.glob(chapter+'.*')) == sorted(n for n in spec['stems'] if '/' not in n)
    for name, digest in spec['stems'].items():
        assert sha(use(STEMS/name)) == digest, name
    return binary, narration, files

def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('--chapter', required=True, choices=sorted(CHAPTERS))
    chapter = ap.parse_args().chapter
    JOB = job(chapter)
    assert not unresolved(chapter), unresolved(chapter)
    base = Q/'csd-native844-base761.py'
    assert sha(base) == BASE761_SHA
    spec = importlib.util.spec_from_file_location('owned_repair_final01', base)
    h = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(h)
    h.own_group()
    h.STATE['job'] = JOB
    h.CRATE = D/'work/production-repo07'
    for sig in h.STOP_SIGNALS:
        signal.signal(sig, h.on_stop)
    receipt = read(JOB/'resource-receipt.json')
    assert receipt['admitted'] is True and 'finished' not in receipt and 'exit' not in receipt
    assert receipt['phase'] == 'native'
    assert receipt['policy'] == POLICY
    assert os.environ['CARGO_TARGET_DIR'] == str(D/'work')
    assert sorted(p.name for p in JOB.iterdir()) == ['command.log', 'resource-receipt.json']
    assert Path(__file__).resolve() == RUNNER.resolve()
    assert [str(x) for x in receipt['command']] == [sys.executable, '-I', '-B', str(Path(__file__).resolve()),
                                                    '--chapter', chapter]
    binary, narration, _ = proofs(chapter)
    cpus = sorted(os.sched_getaffinity(0))
    assert len(cpus) >= 16, 'Higher-worker recovery requires16 available CPUs'
    cores = set()
    for cpu in cpus[:16]:
        topology = Path('/sys/devices/system/cpu')/f'cpu{cpu}'/'topology'
        cores.add(((topology/'physical_package_id').read_text().strip(),
                   (topology/'core_id').read_text().strip()))
    assert len(cores) == 16, 'Do not count sibling logical CPUs as distinct cores'
    os.sched_setaffinity(0, cpus[:16])
    os.environ.update(PATH='/usr/bin:/bin', TMPDIR=str(D/'work/tmp'), XDG_CACHE_HOME=str(D/'work/cache'))
    args = render_args(chapter, binary, narration, JOB/'render')
    h.run_step(JOB, 'native', args, 86400, None)

if __name__ == '__main__':
    main()
