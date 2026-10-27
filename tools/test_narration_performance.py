"""Adversarial checks against padding, missing evidence and alignment laundering."""
import importlib.util
from pathlib import Path
import sys
import unittest
from unittest.mock import patch
import json

import numpy as np

sys.dont_write_bytecode=True
spec=importlib.util.spec_from_file_location('performance',Path(__file__).with_name('narration_performance.py'))
p=importlib.util.module_from_spec(spec);spec.loader.exec_module(p)


def word(text,a,b,coarse=False):
    return dict(text=text,start=a,end=b,coarse=coarse)


class EvidenceControls(unittest.TestCase):
    def support_track(self, silence=False):
        from types import SimpleNamespace
        rate=24000; x=np.sin(2*np.pi*120*np.arange(rate*2)/rate)*.2
        if silence:x[:]=0
        t=np.arange(.01,2.,.01);f=np.full(len(t),120.)
        return SimpleNamespace(x=x,rate=rate,pitch={'ac':(t,f),'cc':(t.copy(),f.copy())})

    def test_periodic_support_is_not_speech_acceptance(self):
        r=p.acoustic_support(self.support_track(),.2,.8)
        self.assertEqual(r['thresholds'][0]['status'],'periodic_support_only')
        self.assertIn('not word alignment',r['claim'])

    def test_stale_pitch_on_zeros_refused(self):
        r=p.acoustic_support(self.support_track(True),.2,.8)
        self.assertTrue(all(not row['edges'] for row in r['thresholds']))

    def test_dc_is_not_acoustic_voicing(self):
        tr=self.support_track();tr.x[:]=.2
        self.assertEqual(p.acoustic_support(tr,.2,.8)['thresholds'][0]['islands'],[])

    def test_discordant_trackers_refused(self):
        tr=self.support_track();tr.pitch['cc'][1][:]=240
        self.assertEqual(p.acoustic_support(tr,.2,.8)['thresholds'][0]['islands'],[])

    def test_no_interpolation_across_unvoiced_track(self):
        tr=self.support_track();tr.pitch['cc'][1][::2]=np.nan
        self.assertEqual(p.acoustic_support(tr,.2,.8)['thresholds'][0]['islands'],[])

    def test_no_outward_search_into_neighbour(self):
        tr=self.support_track();tr.x[:int(.9*tr.rate)]=0
        self.assertEqual(p.acoustic_support(tr,.2,.8)['thresholds'][0]['edges'],{})

    def test_disjoint_voicing_is_not_bridged(self):
        tr=self.support_track();tr.x[int(.7*tr.rate):int(1.3*tr.rate)]=0
        r=p.acoustic_support(tr,.2,1.8)['thresholds'][0]
        self.assertEqual(len(r['islands']),2)
        self.assertLess(r['edges']['first']['support_s'][1],.7)
        self.assertGreater(r['edges']['last']['support_s'][0],1.3)

    def test_acoustic_bounds_and_grids_fail_closed(self):
        tr=self.support_track()
        for a,b in [(-.1,.5),(.5,.5),(0.,3.),(float('nan'),.5)]:
            with self.assertRaises(ValueError):p.acoustic_support(tr,a,b)
        tr.pitch['cc'][0][10]=tr.pitch['cc'][0][9]
        with self.assertRaises(ValueError):p.acoustic_support(tr,0.,1.)

    def test_same_speech_padding_change_is_measurement_failure(self):
        import copy
        a=p.acoustic_support(self.support_track(),.2,.8); b=copy.deepcopy(a)
        b['interval_s']=[x+.5 for x in b['interval_s']]
        for threshold in b['thresholds']:
            for edge in threshold['edges'].values():edge['support_s']=[x+.5 for x in edge['support_s']]
        self.assertEqual(p.compare_acoustic_support(a,b,.5)['status'],'stable_under_this_control')
        b['thresholds'][0]['edges']['last']['median_f0_hz']['ac']*=1.4
        self.assertEqual(p.compare_acoustic_support(a,b,.5)['status'],'padding_sensitive')
        self.assertIn('not changed delivery',p.compare_acoustic_support(a,b,.5)['claim'])

    def test_control_cannot_shift_to_another_phrase(self):
        a=p.acoustic_support(self.support_track(),.2,.8)
        with self.assertRaises(ValueError):p.compare_acoustic_support(a,a,.5)

    def test_endpoint_sensitivity_does_not_invent_corrected_pitch(self):
        class Track:pass
        tr=Track();t=np.arange(0,1.5,.01);f=np.where(t>1.,80.,110.)
        tr.pitch={'ac':(t,f),'cc':(t,f.copy())}
        result=p.contour_sensitivity(tr,0,1.,1.5)
        self.assertEqual(result['status'],'endpoint_sensitive')
        self.assertEqual(result['terminal_change_semitones'],{'ac':0.,'cc':0.})

    def test_inside_phrase_break_is_visible(self):
        words=[word('half',0,.3),word('who',.9,1.2)]
        rows=p.internal_quiet_evidence(words,[[(.3,.9)],[(.31,.89)]],[(.4,.8)])
        self.assertEqual(len(rows),2)
        self.assertAlmostEqual(rows[0]['inserted_silence_s'],.4)
        self.assertEqual(rows[0]['status'],'interior_low_energy_review')

    def test_internal_run_excludes_outside_padding(self):
        rows=p.internal_quiet_evidence([word('half',1,1.4),word('who',1.5,2)],
          [[(0,.9),(2.1,4)],[(0,.9),(2.1,4)]])
        self.assertEqual(rows,[])

    def test_boundary_attribution_does_not_credit_assembly_as_voice(self):
        self.assertEqual(p.insertion_overlap([(1,1.5)],[(.8,1.6),None]),[.5,0.])

    def test_shared_octave_change_requires_review(self):
        class Track:pass
        tr=Track();t=np.arange(0,1,.01);f=np.where(t>.8,50.,100.)
        tr.pitch={'ac':(t,f),'cc':(t,f.copy())}
        self.assertEqual(p.contour_evidence(tr,0,1)['status'],'possible_shared_octave_error')

    def test_pause_proof_rejects_changed_speech(self):
        source=b'\x01\x00\x02\x00'
        seg=[dict(source='x',in_sample=0,out_sample=1),dict(silence_samples=1),dict(source='x',in_sample=1,out_sample=2)]
        self.assertEqual(p.verify_pause_pcm(source,b'\x01\x00\x00\x00\x02\x00',seg,2),1)
        with self.assertRaises(ValueError):p.verify_pause_pcm(source,b'\x03\x00\x00\x00\x02\x00',seg,2)

    def test_pause_proof_rejects_omitted_speech(self):
        with self.assertRaises(ValueError):
            p.verify_pause_pcm(b'\x01\x00\x02\x00',b'\x02\x00',[dict(source='x',in_sample=1,out_sample=2)],2)

    def test_threshold_straddle_is_not_a_pass(self):
        self.assertEqual(p.boundary_status([.07,.34],[.3,.9]),'threshold_sensitive')

    def test_rewording_cannot_silently_drop_direction(self):
        rows=[dict(topic='v',cue='01_1',text='The rewritten words.',sentences=[{'sentence':'The rewritten words.'}])]
        direction={'overrides':[dict(video='v',cue='01_1',after='The original words')]}
        result=p.director_coverage(rows,direction)
        self.assertEqual(result['unresolved'],1)
        self.assertEqual(result['overrides'][0]['status'],'missing_phrase')

    def test_mid_sentence_direction_is_retained_for_review(self):
        rows=[dict(topic='v',cue='01_1',text='One, then two.',sentences=[{'sentence':'One, then two.'}])]
        direction={'overrides':[dict(video='v',cue='01_1',after='One')]}
        result=p.director_coverage(rows,direction)
        self.assertEqual(result['overrides'][0]['status'],'within_sentence_requires_contextual_review')

    def test_coverage_duplicate_is_not_double_credit(self):
        rows=[dict(topic='v',cue='01_1',text='One.',sentences=[{'sentence':'One.'}])]
        item=dict(video='v',cue='01_1',after='One')
        self.assertEqual(p.director_coverage(rows,{'overrides':[item,item]})['unresolved'],1)

    def test_padding_cannot_slow_articulation(self):
        words=[word('one',1,1.5),word('two',1.8,2.3)]
        a=p.speaking_rate(words,[(1.5,1.8)])
        b=p.speaking_rate(words,[(0,1),(1.5,1.8),(2.3,12)])
        self.assertAlmostEqual(a,b)

    def test_overlapping_silences_count_once(self):
        self.assertAlmostEqual(p.overlap_duration([(0,2),(1,3),(8,9)],1,4),2)

    def test_omission_does_not_get_performance_measure(self):
        _,reasons=p.phrase_alignment('It is not one.','It is not one.',
          [word('it',0,.2),word('is',.2,.4),word('one',.4,.8)])
        self.assertIn('unmatched_script_word',reasons)

    def test_coarse_number_is_not_three_timed_words(self):
        _,reasons=p.phrase_alignment('one hundred two','one hundred two',
          [word('one',0,1,True),word('hundred',0,1,True),word('two',0,1,True)])
        self.assertIn('coarse_or_zero_duration_word',reasons)

    def test_repeated_anchor_refused(self):
        with self.assertRaises(ValueError):p.phrase_indices('one and one','one')

    def test_authored_index_disambiguates_repeated_payoff(self):
        self.assertEqual(p.phrase_indices('Back to four. Four.','Four.',3),(3,4))
        with self.assertRaisesRegex(ValueError,'does not match'):
            p.phrase_indices('Back to four. Four.','Four.',1)

    def test_inserted_word_demotes_alignment(self):
        _,reasons=p.phrase_alignment('one two','one two',
          [word('one',0,.3),word('extra',.3,.6),word('two',.6,1)])
        self.assertIn('noncontiguous_alignment',reasons)

    def test_whisper_overlap_demoted(self):
        _,reasons=p.phrase_alignment('one two','one two',[word('one',0,.8),word('two',.5,1)])
        self.assertIn('overlapping_word_intervals',reasons)

    def test_insufficient_pitch_abstains(self):
        t=np.arange(0,1,.01);f=np.full(len(t),np.nan)
        self.assertIsNone(p.terminal_change(t,f,0,1))

    def test_long_unvoiced_tail_abstains(self):
        t=np.arange(0,1,.01);f=np.where(t<.5,100.,np.nan)
        self.assertIsNone(p.terminal_change(t,f,0,1))

    def test_octave_disagreement_not_averaged(self):
        class Track:pass
        tr=Track();t=np.arange(0,1,.01)
        tr.pitch={'ac':(t,np.where(t>.8,200.,100.)), 'cc':(t,np.full(len(t),100.))}
        self.assertEqual(p.contour_evidence(tr,0,1)['status'],'tracker_disagreement')

    def test_zero_word_duration_demoted(self):
        _,r=p.phrase_alignment('one','one',[word('one',0,0)])
        self.assertIn('coarse_or_zero_duration_word',r)

    def test_old_asr_for_different_audio_is_refused(self):
        receipt=dict(phase='asr',exit=0,preflight_only=False,
                     inputs_sha256='manifest',final_inputs_sha256='manifest')
        inputs={'files':[{'path':'/clips/one.wav','sha256':'old'}]}
        with patch.object(Path,'read_text',side_effect=[json.dumps(receipt),json.dumps(inputs)]), patch.object(p.nd,'digest',return_value='manifest'):
            with self.assertRaisesRegex(ValueError,'not bound'):
                p.validate_asr_binding([dict(id='one',wav='/clips/one.wav',sha256='new')],
                    Path('/run/command.log'),Path('/inputs.json'),Path('/run/resource-receipt.json'))

    def test_input_mutation_during_asr_is_refused(self):
        receipt=dict(phase='asr',exit=0,preflight_only=False,
                     inputs_sha256='before',final_inputs_sha256='after')
        with patch.object(Path,'read_text',return_value=json.dumps(receipt)), patch.object(p.nd,'digest',return_value='after'):
            with self.assertRaisesRegex(ValueError,'unchanged-input'):
                p.validate_asr_binding([],Path('/run/command.log'),Path('/inputs.json'),Path('/run/resource-receipt.json'))


if __name__=='__main__':unittest.main()
