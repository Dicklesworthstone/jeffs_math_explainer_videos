"""Read-only, current-csd preview review prerequisite for final production.

This validates recorded viewing against exact extracted assets; it cannot
establish that a person or agent actually viewed them. No automatic approval.
"""
import importlib.util
import json
import sys
from pathlib import Path

_spec = importlib.util.spec_from_file_location('csd_pictures', Path(__file__).with_name('pictures01.py'))
X = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(X)
P, S = X.P, X.S


def validate_record(review, extraction, chapters, digest, exists):
    """Pure record validation, separately exercised with explicit fixtures."""
    assert review['pictures_viewed'] is True
    assert review['unresolved_visual_defects'] == []
    assert isinstance(review['reviewer'], str) and review['reviewer'].strip()
    assert review['review_scope'] in ('producer selected-frame review', 'independent selected-frame review')
    assert review['chapters'] == list(chapters)
    assert review['extraction_sha256'] == digest
    assert review['listening_accepted'] is False, 'This gate does not establish listening'
    assert review['fine_sync_accepted'] is False, 'This gate does not establish fine sync'
    assert review['full_independent_av_accepted'] is False
    samples, sheets = extraction['samples'], extraction['sheets']
    assert samples and sheets
    assert {s['chapter'] for s in samples} == set(chapters)
    assert {s['chapter'] for s in sheets} == set(chapters)
    expected = {}
    for chapter in chapters:
        rows = [s for s in samples if s['chapter'] == chapter]
        assert [s['frame'] for s in rows] == sorted({s['frame'] for s in rows})
        chapter_sheets = [s for s in sheets if s['chapter'] == chapter]
        assert [(s['first_sample'], s['count']) for s in chapter_sheets] == [
            (i, min(6, len(rows) - i)) for i in range(0, len(rows), 6)]
        for row in rows:
            assert row['reasons'] and exists(row['picture'], row['sha256'])
        for sheet in chapter_sheets:
            assert sheet['path'] not in expected
            assert exists(sheet['path'], sheet['sha256'])
            expected[sheet['path']] = sheet['sha256']
    viewed = review['viewed_sheets']
    assert len(viewed) == len(expected)
    assert len({s['path'] for s in viewed}) == len(viewed)
    assert {s['path']: s['sha256'] for s in viewed} == expected
    return {'chapters': len(chapters), 'selected_pictures': len(samples), 'viewed_sheets': len(sheets)}


def check(review_path):
    P.check_score()
    S.success('stage07-preview01')
    S.success('stage07-pictures01')
    summary_path = P.JOB / 'preview-summary.json'
    summary = S.read(summary_path)
    assert summary['binary_sha256'] == S.BINARY_SHA
    assert [r['chapter'] for r in summary['chapters']] == list(S.CHAPTERS)
    for row in summary['chapters']:
        for suffix, key in [('mp4', 'mp4_sha256'), ('timeline.tsv', 'timeline_sha256'), ('layout.tsv', 'layout_sha256')]:
            assert S.sha(P.OUT / (row['chapter'] + '.' + suffix)) == row[key]
    extraction_path = X.PICTURES_OUT / 'extraction.json'
    extraction = S.read(extraction_path)
    assert extraction['preview_summary_sha256'] == S.sha(summary_path)
    review = S.read(review_path)
    assert review['preview_summary_sha256'] == S.sha(summary_path)
    def exact_file(path, digest):
        path = Path(path)
        assert path.is_relative_to(X.PICTURES_OUT), 'Unexpected picture path'
        return path.is_file() and S.sha(path) == digest
    result = validate_record(review, extraction, S.CHAPTERS, S.sha(extraction_path), exact_file)
    assert {str(p) for p in X.PICTURES_OUT.glob('*/sheet-*.png')} == {r['path'] for r in extraction['sheets']}
    result.update(review_sha256=S.sha(review_path), extraction_sha256=S.sha(extraction_path),
                  preview_summary_sha256=S.sha(summary_path), binary_sha256=S.BINARY_SHA,
                  scope='Recorded selected-frame review only; final 4K pictures and audiovisual checks remain.')
    return result


if __name__ == '__main__':
    assert len(sys.argv) == 2, 'Supply an actual completed review record; this tool never creates one'
    print(json.dumps(check(Path(sys.argv[1])), indent=2))
