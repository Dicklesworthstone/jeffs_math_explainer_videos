#!/usr/bin/env python3
"""Semantic performance microscope. Read-only; never selects or edits audio.

Reuses narration_director's exact-word alignment and waveform evidence, plus
prosody_qa's two Praat trackers. Measures authored phrases rather than assigning
a global naturalness score. A missing/disputed measurement remains unknown.
Use: uv run --no-project --with numpy --with praat-parselmouth --with matplotlib
python3 -I tools/narration_performance.py MANIFEST INTENTIONS FW_NDJSON OUT
  --fw-inputs INPUT_MANIFEST --fw-receipt RESOURCE_RECEIPT
"""
from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import math
import wave
from pathlib import Path
import sys

import numpy as np

sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location("director", Path(__file__).with_name("narration_director.py"))
nd = importlib.util.module_from_spec(spec)
spec.loader.exec_module(nd)
qa = nd.qa


def phrase_indices(text, phrase, start_index=None):
    words, query = nd.tokens(text), nd.tokens(phrase)
    starts = [i for i in range(len(words) - len(query) + 1) if words[i:i + len(query)] == query]
    if start_index is not None:
        if type(start_index) is not int or start_index not in starts:
            raise ValueError('explicit phrase index does not match script')
        return start_index, start_index + len(query)
    if not query or len(starts) != 1:
        raise ValueError("phrase must match exactly once: " + phrase)
    return starts[0], starts[0] + len(query)


def phrase_alignment(text, phrase, observed, start_index=None):
    a, b = phrase_indices(text, phrase, start_index)
    mapping = nd.align(nd.tokens(text), [w["text"] for w in observed])
    indices = [mapping.get(i) for i in range(a, b)]
    reason = []
    if any(i is None for i in indices):
        reason.append("unmatched_script_word")
    selected = [observed[i] for i in indices if i is not None]
    if any(w["coarse"] or w["end"] <= w["start"] for w in selected):
        reason.append("coarse_or_zero_duration_word")
    if any(j != i + 1 for i, j in zip(indices, indices[1:]) if i is not None and j is not None):
        reason.append("noncontiguous_alignment")
    if any(y["start"] < x["end"] - .025 for x, y in zip(selected, selected[1:])):
        reason.append("overlapping_word_intervals")
    return selected, reason


def overlap_duration(intervals, lo, hi):
    """Union length, so overlapping detector runs never count silence twice."""
    rows = sorted((max(a, lo), min(b, hi)) for a, b in intervals if min(b, hi) > max(a, lo))
    total, last = 0., lo
    for a, b in rows:
        total += max(0., b - max(last, a))
        last = max(last, b)
    return total


def speaking_rate(words, quiet):
    # This is tokens per acoustic active second, not a syllable/phoneme rate.
    # Outside padding cannot improve it; intervals must stay within the words.
    if not words:
        return None
    lo, hi = words[0]["start"], words[-1]["end"]
    active = hi - lo - overlap_duration([(a, b) for a, b in quiet if b-a >= .15], lo, hi)
    return len(words) / active if active >= .15 else None


def terminal_change(times, pitch, lo, hi):
    """Two local voiced windows. Never bridge a long unvoiced tail."""
    keep = (times >= lo) & (times <= hi) & np.isfinite(pitch)
    t, f = times[keep], pitch[keep]
    if len(t) < 12 or hi - t[-1] > .35:
        return None
    end = t[-1]
    tail = f[t >= end - .15]
    before = f[(t >= end - .55) & (t < end - .20)]
    if len(tail) < 5 or len(before) < 8:
        return None
    return float(12 * np.log2(np.median(tail) / np.median(before)))


def contour_evidence(track, lo, hi):
    vals = {k: terminal_change(t, f, lo, hi) for k, (t, f) in track.pitch.items()}
    a, b = vals["ac"], vals["cc"]
    if a is None or b is None:
        status = "insufficient_voicing"
    elif abs(a - b) > 1.5 or (a > 1 and b < -1) or (a < -1 and b > 1):
        status = "tracker_disagreement"
    elif 8 <= abs(a) <= 16 or 8 <= abs(b) <= 16:
        status = "possible_shared_octave_error"
    else:
        status = "both_rise" if min(a,b) > 1 else "both_fall" if max(a,b) < -1 else "small_or_mixed_change"
    return {"terminal_change_semitones": vals, "status": status,
            "claim": "Local contour inside an ASR window. Its endpoint may truncate voicing; both trackers can share errors. Not question correctness, accent or naturalness."}


def contour_sensitivity(track, lo, hi, duration):
    result=contour_evidence(track,lo,hi);windows=[]
    for shift in (-.15,0.,.15,.30):
        end=min(duration,hi+shift)
        windows.append({'requested_shift_s':shift,'end_s':end,
          'terminal_change_semitones':{k:terminal_change(t,f,lo,end) for k,(t,f) in track.pitch.items()}})
    ranges={}
    for key in track.pitch:
        values=[w['terminal_change_semitones'][key] for w in windows
                if w['terminal_change_semitones'][key] is not None]
        ranges[key]=[min(values),max(values)] if len(values)>=2 else None
    result['endpoint_sensitivity']={'windows':windows,'ranges_semitones':ranges,
        'claim':'Perturbed endpoints are a sensitivity test, not corrected alignment; extended windows can include later speech.'}
    if any(v is not None and v[1]-v[0]>2 for v in ranges.values()):
        result['nominal_status']=result['status'];result['status']='endpoint_sensitive'
    return result


def insertion_overlap(intervals, spans):
    return [overlap_duration(intervals, *s) if s is not None else 0. for s in spans]


def assembly_evidence(row):
    """Reconstruct published cue bytes from the bound sentence report."""
    if 'sentences' not in row:
        return None
    if nd.digest(row['report']) != row['report_sha256']:
        raise ValueError('assembly report changed')
    entries = json.loads(Path(row['report']).read_text())['lines'][row['cue']]
    if entries != row['sentences']:
        raise ValueError('assembly entries differ from bound report')
    def pcm(path):
        with wave.open(str(path), 'rb') as f:
            if (f.getnchannels(),f.getsampwidth(),f.getframerate()) != (1,2,24000):
                raise ValueError('assembly attribution requires mono PCM16 at 24 kHz')
            if f.getnframes() > 120*24000:
                raise ValueError('assembly source exceeds bounded analysis duration')
            return f.readframes(f.getnframes())
    sources={};parts=[];cursor=0;intervals=[]
    for entry in entries:
        source=entry['source']
        if source not in sources:
            if nd.digest(source) != entry['source_sha256']:
                raise ValueError('assembly source changed')
            sources[source]=pcm(source)
        a,b=entry['source_in_sample'],entry['source_out_sample'];gap=entry['gap_after_samples']
        if not (type(a) is int and type(b) is int and 0<=a<b<=len(sources[source])//2
                and b-a==entry['out_samples'] and type(gap) is int and gap>=0):
            raise ValueError('invalid assembly sample span')
        parts.append(sources[source][a*2:b*2]);cursor+=b-a
        if gap:
            intervals.append((cursor/24000,(cursor+gap)/24000))
            parts.append(bytes(gap*2));cursor+=gap
    if b''.join(parts)!=pcm(row['wav']):
        raise ValueError('assembly attribution does not reconstruct selected PCM')
    return {'inserted_intervals_s':intervals,'source_count':len(sources),
            'claim':'Exact cue PCM reconstruction. Remaining quiet belongs to retained source audio; it is not necessarily a deliberate performed pause.'}


def internal_quiet_evidence(words, quiet, inserted=None):
    """Expose interior runs, without declaring a consonant/breath an error."""
    if not words:
        return []
    lo,hi=words[0]['start'],words[-1]['end'];result=[]
    for threshold,runs in zip((30,38),quiet):
        for a,b in runs:
            if b-a < .15 or a<=lo or b>=hi:
                continue
            left=[w for w in words if w['start']<a]
            right=[w for w in words if w['end']>b]
            result.append({'relative_threshold_db':threshold,'interval_s':[a,b],'duration_s':b-a,
                'approximate_left_word':left[-1]['text'] if left else None,
                'approximate_right_word':right[0]['text'] if right else None,
                'inserted_silence_s':overlap_duration(inserted,a,b) if inserted is not None else None,
                'status':'interior_low_energy_review',
                'claim':'ASR neighbours are uncertain. May be a planned pause, breath or phonetic closure; not a cut instruction.'})
    return result


def waveform_boundary(quiet, left, right):
    measured = [nd.boundary_quiet(q, left, right) for q in quiet]
    lengths = [x[0] for x in measured]
    return {"fw_gap_s": right["start"] - left["end"],
            "waveform_gap_range_s": [min(lengths), max(lengths)],
            "waveform_intervals": [x[1] for x in measured],
            "claim": "Low energy can include consonants or breath; not a safe edit boundary."}


def boundary_status(observed, proposed):
    """An interval straddling the target is uncertain, not an in-range pass."""
    lo,hi=observed; minimum,maximum=proposed
    if minimum <= lo <= hi <= maximum:
        return 'within_authored_range'
    if hi < minimum - .08:
        return 'below_range_candidate'
    if lo > maximum + .12:
        return 'above_range_candidate'
    if lo < minimum <= hi or lo <= maximum < hi:
        return 'threshold_sensitive'
    return 'near_boundary'


def director_coverage(rows, direction):
    """Account for every authored override; never claim an edit was heard/applied."""
    cues={(r['topic'],r['cue']):r for r in rows}
    if len(cues)!=len(rows):
        raise ValueError('duplicate cue in coverage manifest')
    result=[];seen=set()
    for index,item in enumerate(direction['overrides']):
        row=cues.get((item['video'],item['cue']))
        disposition={**item,'override_index':index}
        if row is None:
            disposition['status']='missing_cue'
        else:
            ts=nd.tokens(row['text']);needle=nd.tokens(item['after'])
            hits=[i+len(needle)-1 for i in range(len(ts)-len(needle)+1)
                  if needle and ts[i:i+len(needle)]==needle]
            if 'word_index' in item:
                hits=[i for i in hits if i==item['word_index']]
            if len(hits)!=1:
                disposition['status']='missing_phrase' if not hits else 'ambiguous_phrase'
            else:
                at=hits[0];key=(item['video'],item['cue'],at)
                disposition['resolved_word_index']=at
                if key in seen:
                    disposition['status']='duplicate_boundary'
                else:
                    seen.add(key);ends=[];cursor=0
                    for sentence in row['sentences']:
                        cursor+=len(nd.tokens(sentence['sentence']));ends.append(cursor-1)
                    if cursor!=len(ts):
                        raise ValueError('sentence text differs from coverage script')
                    disposition['status']=('cue_end_requires_scene_timing' if at==len(ts)-1 else
                        'sentence_end_requires_audio_review' if at in ends else
                        'within_sentence_requires_contextual_review')
        result.append(disposition)
    unresolved={'missing_cue','missing_phrase','ambiguous_phrase','duplicate_boundary'}
    return {'schema_version':1,'overrides':result,
            'unresolved':sum(x['status'] in unresolved for x in result),
            'claim':'Coverage and routing only. No automatic pause editing or audio acceptance.'}


def coverage_main(argv):
    ap=argparse.ArgumentParser(description='Audit all authored pause instructions against exact cue text.')
    for name in ('manifest','direction','out'):ap.add_argument(name,type=Path)
    a=ap.parse_args(argv);manifest=json.loads(a.manifest.read_text())
    checked=set()
    for row in manifest['rows']:
        path=row['report']
        if (path,row['report_sha256']) not in checked:
            if nd.digest(path)!=row['report_sha256']:raise ValueError('cue report changed')
            checked.add((path,row['report_sha256']))
        sentences=json.loads(Path(path).read_text())['lines'][row['cue']]
        if sentences!=row['sentences'] or ' '.join(s['sentence'] for s in sentences)!=row['text']:
            raise ValueError('cue text is not bound to the assembly report')
    result=director_coverage(manifest['rows'],json.loads(a.direction.read_text()))
    result['inputs']={str(p):nd.digest(p) for p in (a.manifest,a.direction,Path(__file__))}
    nd.write_new(a.out,result)
    print(json.dumps({'overrides':len(result['overrides']),'unresolved':result['unresolved']}))
    return 2 if result['unresolved'] else 0


def word_features(track, words):
    result = []
    for w in words:
        a,b = w["start"],w["end"]
        medians = {}
        for k,(t,f) in track.pitch.items():
            v=f[(t>=a)&(t<=b)&np.isfinite(f)]
            medians[k]=float(np.median(v)) if len(v)>=4 else None
        level=track.level_in(a,b)
        result.append({**w,"duration_s":b-a,"median_f0_hz":medians,
                       "level_dbfs":float(level) if math.isfinite(level) else None})
    return result


def acoustic_support(track, lo, hi):
    """Locate periodic support INSIDE a supplied interval, never repair alignment.

    This is not a speech detector: a musical tone can satisfy both trackers.
    Pair actual neighbouring AC/CC frames (no interpolation across unvoiced
    frames), gate their local energy, and keep contiguous islands separate.
    An ASR interval that misses a vowel stays unsupported; never steal one
    from a neighbouring word. Thresholds are diagnostic proposals, not norms.
    """
    duration = len(track.x) / track.rate
    if not (math.isfinite(lo) and math.isfinite(hi) and 0 <= lo < hi <= duration + 1e-6):
        raise ValueError('acoustic interval outside audio')
    ta, fa = track.pitch['ac']; tc, fc = track.pitch['cc']
    ta, fa, tc, fc = [np.asarray(v, dtype=float) for v in (ta, fa, tc, fc)]
    if (len(ta) != len(fa) or len(tc) != len(fc)
            or not np.all(np.isfinite(ta)) or not np.all(np.isfinite(tc))
            or np.any(np.diff(ta) <= 0) or np.any(np.diff(tc) <= 0)):
        raise ValueError('invalid pitch frame grid')
    selected = (ta >= lo) & (ta <= hi)
    t, a = ta[selected], fa[selected]
    b = np.full(len(t), np.nan)
    if len(tc):
        right = np.searchsorted(tc, t).clip(0, len(tc)-1)
        left = (right-1).clip(0, len(tc)-1)
        at = np.where(abs(tc[right]-t) < abs(tc[left]-t), right, left)
        close = abs(tc[at]-t) <= .0051
        b[close] = fc[at[close]]
    valid = np.isfinite(a) & np.isfinite(b) & (a > 0) & (b > 0)
    agreement = np.zeros(len(t), dtype=bool)
    agreement[valid] = abs(12*np.log2(a[valid]/b[valid])) <= 1.0
    # Use the same frame's waveform support, not a remote high-energy frame.
    level = np.full(len(t), -120.)
    for i, tm in enumerate(t):
        start = max(0, int(round((tm-.01)*track.rate)))
        stop = min(len(track.x), int(round((tm+.01)*track.rate)))
        samples = track.x[start:stop]
        if len(samples):
            samples = samples - np.mean(samples)
            level[i] = 20*np.log10(max(float(np.sqrt(np.mean(samples*samples))), 1e-6))
    supported_levels = level[valid & (level > -70.)]
    top = float(np.percentile(supported_levels, 90)) if len(supported_levels) else -70.
    thresholds = []
    for relative in (30, 38):
        keep = agreement & (level > max(-70., top-relative))
        indices = np.flatnonzero(keep)
        groups = []
        for idx in indices:
            if not groups or idx != groups[-1][-1]+1 or t[idx]-t[groups[-1][-1]] > .025:
                groups.append([int(idx)])
            else:
                groups[-1].append(int(idx))
        groups = [g for g in groups if len(g) >= 6]
        islands = [{'start_s':float(t[g[0]]), 'end_s':float(t[g[-1]]),
                    'frames':len(g)} for g in groups]
        edges = {}
        if groups:
            for name, group in (('first', groups[0]), ('last', groups[-1])):
                idx = np.asarray(group)
                anchor = t[idx[0]] if name == 'first' else t[idx[-1]]
                idx = idx[(t[idx] <= anchor+.15) if name == 'first' else (t[idx] >= anchor-.15)]
                edges[name] = {'support_s':[float(t[idx[0]]), float(t[idx[-1]])],
                    'frames':len(idx), 'median_f0_hz':{'ac':float(np.median(a[idx])), 'cc':float(np.median(b[idx]))},
                    'distance_to_interval_edge_s':float(anchor-lo if name == 'first' else hi-anchor),
                    'possibly_censored':bool(anchor-lo < .03 if name == 'first' else hi-anchor < .03)}
        thresholds.append({'relative_db':relative, 'islands':islands, 'edges':edges,
            'status':'periodic_support_only' if groups else 'insufficient_joint_support'})
    return {'interval_s':[lo, hi], 'thresholds':thresholds,
        'claim':'Periodic support within the supplied bounds, not word alignment, safe edit points, speech identity or naturalness. No outward search. Tracker errors may be shared.'}


def compare_acoustic_support(original, padded, offset_s):
    """Negative control: identical speech should not acquire a new pitch edge.

    Failures diagnose the measurement, never the narrator. This compares a
    specified control, not every possible padding or tracker configuration.
    """
    if not math.isfinite(offset_s) or offset_s < 0:
        raise ValueError('invalid control offset')
    if any(abs(b-a-offset_s) > 1e-6 for a,b in zip(original['interval_s'],padded['interval_s'])):
        raise ValueError('control intervals do not describe the same source samples')
    rows=[]
    for relative in (30,38):
        left=[x for x in original['thresholds'] if x['relative_db']==relative]
        right=[x for x in padded['thresholds'] if x['relative_db']==relative]
        if len(left)!=1 or len(right)!=1:
            raise ValueError('missing or repeated acoustic threshold')
        for name in ('first','last'):
            a,b=left[0]['edges'].get(name),right[0]['edges'].get(name)
            row={'relative_db':relative,'edge':name}
            if not a or not b:
                row['status']='insufficient_joint_support'
            else:
                time_error=max(abs(y-x-offset_s) for x,y in zip(a['support_s'],b['support_s']))
                pitch_error=max(abs(12*math.log2(b['median_f0_hz'][k]/a['median_f0_hz'][k])) for k in ('ac','cc'))
                row.update(time_error_s=time_error,pitch_error_st=pitch_error,
                    status='stable_under_this_control' if time_error<=.035 and pitch_error<=.5 else 'padding_sensitive')
            rows.append(row)
    status=('padding_sensitive' if any(x['status']=='padding_sensitive' for x in rows) else
            'insufficient_joint_support' if any(x['status']=='insufficient_joint_support' for x in rows) else
            'stable_under_this_control')
    return {'status':status,'edges':rows,
        'claim':'Exact-speech external-padding control. A changed measurement is not changed delivery. Stability here is not alignment, naturalness or accent acceptance.'}


def validate_asr_binding(rows, fw_path, inputs_path, receipt_path):
    """Require the producer's before/after audio binding, not just a file name."""
    receipt = json.loads(receipt_path.read_text())
    expected = nd.digest(inputs_path)
    if (receipt.get("phase") != "asr" or receipt.get("exit") != 0
            or receipt.get("stopped_for_resource_limit", False)
            or receipt.get("preflight_only") is not False
            or receipt.get("inputs_sha256") != expected
            or receipt.get("final_inputs_sha256") != expected):
        raise ValueError("ASR producer has no successful unchanged-input receipt")
    if fw_path.resolve() != (receipt_path.parent / "command.log").resolve():
        raise ValueError("ASR log is not the receipt's command output")
    files = json.loads(inputs_path.read_text())["files"]
    bound = {}
    for entry in files:
        path = str(Path(entry["path"]).resolve())
        if path in bound:
            raise ValueError("duplicate ASR input binding")
        bound[path] = entry["sha256"]
    for row in rows:
        if bound.get(str(Path(row["wav"]).resolve())) != row["sha256"]:
            raise ValueError("ASR was not bound to selected audio: " + row["id"])
    return {str(p): nd.digest(p) for p in (fw_path, inputs_path, receipt_path)}


def verify_pause_pcm(source, output, segments, width):
    """Independent byte comparison: every source sample once, only zeros added."""
    cursor=0;parts=[];inserted=0
    for segment in segments:
        if 'source' in segment:
            a,b=segment['in_sample'],segment['out_sample']
            if type(a) is not int or type(b) is not int or a!=cursor or not a<b<=len(source)//width:
                raise ValueError('pause-only edit omits, repeats or reorders source samples')
            parts.append(source[a*width:b*width]);cursor=b
        else:
            count=segment['silence_samples']
            if type(count) is not int or not 0<count<=24000*5:
                raise ValueError('invalid bounded silence insertion')
            inserted+=count;parts.append(bytes(count*width))
    if cursor!=len(source)//width or b''.join(parts)!=output:
        raise ValueError('audio is not exactly source plus authored silence')
    return inserted


def pause_only_evidence(row):
    path=Path(row['edit_receipt'])
    if nd.digest(path)!=row['edit_receipt_sha256']:raise ValueError('edit receipt changed')
    receipt=json.loads(path.read_text());manifest=receipt['manifest']
    if len(manifest['sources'])!=1:raise ValueError('pause-only proof requires one complete source')
    name,source=next(iter(manifest['sources'].items()))
    if nd.digest(source['path'])!=source['sha256']:raise ValueError('edit source changed')
    if any(s.get('source',name)!=name for s in manifest['segments']):raise ValueError('unknown edit source')
    def pcm(p):
        with wave.open(str(p),'rb') as f:
            if (f.getnchannels(),f.getsampwidth(),f.getframerate())!=(1,2,24000) or f.getnframes()>120*24000:
                raise ValueError('pause-only proof requires bounded mono PCM16 at 24 kHz')
            return f.readframes(f.getnframes())
    raw,edited=pcm(source['path']),pcm(row['wav'])
    inserted=verify_pause_pcm(raw,edited,manifest['segments'],2)
    intervals=[];cursor=0
    for segment in manifest['segments']:
        if 'source' in segment:cursor+=segment['out_sample']-segment['in_sample']
        else:
            count=segment['silence_samples'];intervals.append((cursor/24000,(cursor+count)/24000));cursor+=count
    return {'source':source,'inserted_silence_samples':inserted,'inserted_intervals_s':intervals,'speech_pcm_changed':False,
            'claim':'Speech duration, pitch and articulation are unchanged. Fresh ASR-derived differences are measurement sensitivity, not a performance improvement.'}


def analyse(row, intent, fw):
    if nd.digest(row["wav"]) != row["sha256"]:
        raise ValueError("audio hash mismatch: " + row["id"])
    x,rate=qa.read_wav(row["wav"])
    if len(x)/rate > 120:
        raise ValueError("bound individual analysis to 120 seconds")
    words=nd.asr_words(fw)
    if any(w["end"] > len(x)/rate + .1 for w in words):
        raise ValueError("ASR outside audio")
    track=qa.Track(x,rate)
    # Holding the pitch range fixed isolates boundary/context sensitivity.
    pad_samples=rate//2
    padded_track=qa.Track(np.concatenate((np.zeros(pad_samples),x,np.zeros(2*rate))),
                          rate,bounds=track.bounds)
    quiet=[nd.quiet_runs(x,rate,relative) for relative in (30,38)]
    assembly=assembly_evidence(row)
    invariance=pause_only_evidence(row) if 'edit_receipt' in row else None
    provenance=invariance or assembly
    inserted=provenance['inserted_intervals_s'] if provenance else None
    phrases=[]
    for instruction in intent["phrases"]:
        if 'word_index' in instruction and 'start_word_index' in instruction:
            raise ValueError('ambiguous phrase start index fields')
        selected,reasons=phrase_alignment(row["text"],instruction["text"],words,instruction.get('start_word_index',instruction.get('word_index')))
        if any(w['start'] < 0 or w['end'] > len(x)/rate for w in selected):
            reasons.append('word_interval_outside_audio')
        p={**instruction,"alignment_review":reasons,"words":word_features(track,selected)}
        if not reasons:
            a,b=selected[0]["start"],selected[-1]["end"]
            p.update({"start_s":a,"end_s":b,"contour":contour_sensitivity(track,a,b,len(x)/rate),
                      "acoustic_support":acoustic_support(track,a,b),
                      "tokens_per_active_second":[speaking_rate(selected,q) for q in quiet],
                      "internal_quiet_reviews":internal_quiet_evidence(selected,quiet,inserted)})
            p['measurement_reliability']=compare_acoustic_support(p['acoustic_support'],
                acoustic_support(padded_track,a+pad_samples/rate,b+pad_samples/rate),pad_samples/rate)
            if p['measurement_reliability']['status']!='stable_under_this_control':
                p['contour']['nominal_status_before_padding_control']=p['contour']['status']
                p['contour']['status']='acoustic_measurement_review'
        phrases.append(p)
    boundaries=[]
    for left,right in zip(phrases,phrases[1:]):
        if left["alignment_review"] or right["alignment_review"]:
            boundaries.append({"after":left["text"],"status":"alignment_review"});continue
        evidence=waveform_boundary(quiet,left["words"][-1],right["words"][0])
        attribution=(insertion_overlap(inserted,evidence['waveform_intervals']) if inserted is not None else None)
        boundaries.append({"after":left["text"],"proposal_s":left["pause_after_s"],**evidence,
                           "inserted_silence_by_threshold_s":attribution,
                           "attribution_claim":"Inserted silence is assembly timing, not changed vocal performance. Unknown provenance remains null.",
                           "status":boundary_status(evidence["waveform_gap_range_s"],left["pause_after_s"])})
    focus=[]
    for phrase in intent.get("focus",[]):
        selected,reasons=phrase_alignment(row["text"],phrase,words)
        focus.append({"text":phrase,"alignment_review":reasons,"words":word_features(track,selected),
                      "claim":"Compare matched words across conditions; phonetic differences preclude a generic emphasis score."})
    connected=[]
    for group in intent.get('connected_word_groups',[]):
        selected,reasons=phrase_alignment(row['text'],group,words)
        connected.append({'text':group,'alignment_review':reasons,
            'boundaries':[waveform_boundary(quiet,a,b) for a,b in zip(selected,selected[1:])] if not reasons else [],
            'claim':'Authored connected meaning. Interior low energy is review evidence, not proof of an unnatural pause or permission to trim.'})
    result={"id":row["id"],"wav":row["wav"],"sha256":row["sha256"],"text":row["text"],
            "duration_s":len(x)/rate,"phrases":phrases,"boundaries":boundaries,"focus":focus,"connected_word_groups":connected,
            "fw_words":len(words),"exact_script_matches":len(nd.align(nd.tokens(row["text"]),[w['text'] for w in words])),
            "script_words":len(nd.tokens(row["text"])),"external_hold":intent["external_hold"],
            "cue_final_boundary":{"status":"requires_scene_timing_not_measured_from_dry_cue",
              "after":row['text'],"cuelog":row.get('cuelog'),"master_start":row.get('master_start'),
              "claim":"Explicit unmeasured boundary. Preserve prediction/reading holds; route to narration_director with native timing and the next cue."},
            "naturalness_acceptance":False,"listening_performed":False}
    result['clip_acoustic_support']=acoustic_support(track,0,len(x)/rate)
    result['clip_measurement_reliability']=compare_acoustic_support(result['clip_acoustic_support'],
        acoustic_support(padded_track,pad_samples/rate,(pad_samples+len(x))/rate),pad_samples/rate)
    if invariance:result['speech_invariance']=invariance
    if assembly:result['assembly_provenance']=assembly
    return result,(x,rate,track,words,quiet)


def plot(row,data,path):
    import matplotlib
    matplotlib.use("Agg")
    import matplotlib.pyplot as plt
    x,rate,tr,words,quiet=data
    fig,axes=plt.subplots(4,1,figsize=(16,9),sharex=True,gridspec_kw={"height_ratios":[1,2,1,1]})
    step=max(1,len(x)//16000);axes[0].plot(np.arange(0,len(x),step)/rate,x[::step],lw=.45)
    axes[0].set_title(row["id"]+" — waveform / spectrum / two pitch tracks / FW words (not listening)")
    axes[1].specgram(x,NFFT=512,Fs=rate,noverlap=384,cmap="magma");axes[1].set_ylim(0,6000)
    for k,(t,f) in tr.pitch.items():axes[2].plot(t,f,'.',ms=1.5,label=k)
    axes[2].legend();axes[2].set_ylabel('F0 Hz')
    for i,w in enumerate(words):
        axes[3].text(w['start'],i%4,w['text'],rotation=45,fontsize=7,ha='left',color='red' if w['coarse'] else 'black')
    for a,b in quiet[0]:axes[0].axvspan(a,b,color='orange',alpha=.2)
    axes[3].set_ylim(-1,5);axes[3].set_yticks([]);axes[3].set_xlabel('seconds')
    axes[0].set_xlim(0,len(x)/rate);fig.tight_layout();fig.savefig(path,dpi=110);plt.close(fig)


def main():
    ap=argparse.ArgumentParser(description=__doc__)
    for name in ('manifest','intentions','fw','out'):ap.add_argument(name,type=Path)
    ap.add_argument('--fw-inputs', type=Path, required=True)
    ap.add_argument('--fw-receipt', type=Path, required=True)
    a=ap.parse_args();m=json.loads(a.manifest.read_text());directions=json.loads(a.intentions.read_text())
    provenance=validate_asr_binding(m['rows'],a.fw,a.fw_inputs,a.fw_receipt)
    intents={x['id']:x for x in directions['rows']}
    if len(intents)!=len(directions['rows']) or len({r['id'] for r in m['rows']})!=len(m['rows']):
        raise ValueError('duplicate clip or intention ID')
    if set(intents)!={r['id'] for r in m['rows']}:
        raise ValueError('intention coverage differs from selected clips')
    records=[json.loads(s) for s in a.fw.read_text().splitlines() if s.startswith('{')]
    asr={}
    for record in records:
        if record.get('status')!='ok':raise ValueError('FW failed')
        key=str(Path(record['input']).resolve())
        if key in asr:raise ValueError('duplicate FW input')
        asr[key]=record['report']['result']
    a.out.mkdir(parents=True,exist_ok=False);results=[]
    for row in m['rows']:
        result,data=analyse(row,intents[row['id']],asr[str(Path(row['wav']).resolve())])
        plot(result,data,a.out/(row['id']+'.png'));results.append(result)
    nd.write_new(a.out/'results.json',{'schema_version':4,'rows':results,'asr_provenance':provenance,
      'intentions_claim':directions.get('claim','Unspecified; not evidence of prospective authorship.'),
      'inputs':{str(p):nd.digest(p) for p in [a.manifest,a.intentions,a.fw,Path(__file__),Path(__file__).with_name('prosody_qa.py'),Path(__file__).with_name('narration_director.py')]},
      'claim':'Diagnostic measurements only. No scalar quality gate, automatic edit or listening acceptance.'})
    print(json.dumps({'clips':len(results),'phrases':sum(len(r['phrases']) for r in results),
                      'alignment_reviews':sum(bool(p['alignment_review']) for r in results for p in r['phrases'])}))


if __name__=='__main__':
    if len(sys.argv)>1 and sys.argv[1]=='coverage':
        raise SystemExit(coverage_main(sys.argv[2:]))
    main()
