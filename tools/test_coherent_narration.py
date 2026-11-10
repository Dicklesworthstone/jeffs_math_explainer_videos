"""Adapter invariants on synthetic PCM; these tests make no acoustic claims."""
import copy
import importlib.util
import json
from pathlib import Path
import shutil
import struct
import tempfile
import types
import unittest
from unittest.mock import patch
import wave

spec = importlib.util.spec_from_file_location("coherent", Path(__file__).with_name("coherent_narration.py"))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class CoherentTests(unittest.TestCase):
    def setUp(self):
        # Retain test artifacts, consistent with this repository's no-deletion rule.
        self.root = Path(tempfile.mkdtemp(prefix="coherent-narration-test-"))
        # The fixture is a complete miniature repository, with relative maps.
        repo = patch.object(m, 'REPO', self.root)
        repo.start()
        self.addCleanup(repo.stop)
        self.script = self.root / "script.tsv"
        self.script.write_text("01_1\tFour pulses. They return.\n01_2\tDo not drop the negative.\n")
        self.units = self.root / "units.json"
        self.plan = {"schema_version": 1, "script_sha256": m.cache.digest(self.script), "units": [
            {"id": "01u1", "line_ids": ["01_1", "01_2"],
             "text": "Four pulses. They return. Do not drop the negative."}]}
        self.save_plan()

    def save_plan(self):
        self.units.write_text(json.dumps(self.plan))

    def bind_direction(self):
        self.direction=self.root/'explainer_corpus/example/directions.json'
        self.direction.parent.mkdir(parents=True, exist_ok=True)
        self.direction.write_text(json.dumps({'overrides':[{'video':'example','cue':'01_1','after':'Four pulses'}]}))
        self.plan['direction_contract']={'schema_version':1,'topic':'example',
          'script_sha256':m.cache.digest(self.script),
          'map':{'path':'explainer_corpus/example/directions.json','sha256':m.cache.digest(self.direction)}}
        self.save_plan()

    def test_direction_map_paths_are_repository_relative(self):
        self.bind_direction()
        self.assertEqual(m.direction_map_path('explainer_corpus/example/directions.json', 'example'), self.direction.resolve())
        for path in (str(self.direction), '../directions.json', '', None):
            with self.subTest(path=path):
                self.plan['direction_contract']['map']['path'] = path
                self.save_plan()
                with self.assertRaisesRegex(ValueError, 'must be repository-relative'):
                    m.read_units(self.script, self.units, require_directions=True)

    def test_direction_map_refuses_symlink_escape(self):
        self.bind_direction()
        outside = Path(tempfile.mkdtemp(prefix='coherent-map-outside-'))/'map.json'
        outside.write_bytes(self.direction.read_bytes())
        (self.root/'escape.json').symlink_to(outside)
        self.plan['direction_contract']['map']['path'] = 'escape.json'
        self.save_plan()
        with self.assertRaisesRegex(ValueError, 'escapes repository'):
            m.read_units(self.script, self.units, require_directions=True)

    def test_map_must_stay_in_topic_tree_in_binding_and_verify(self):
        args = self.bound_assembly()
        for i, name in enumerate(('narration_lab/x/map.json',
                                  'explainer_corpus/another/map.json')):
            path = self.root/name
            path.parent.mkdir(parents=True)
            path.write_bytes(self.direction.read_bytes())
            self.plan['direction_contract']['map']['path'] = name
            self.save_plan()
            with self.subTest(path=name), self.assertRaisesRegex(ValueError, 'inside its topic tree'):
                m.read_units(self.script, self.units, require_directions=True)
            def change(report, _):
                report['meta']['direction_contract']['binding']['contract']['map']['path'] = name
            out = self.altered_assembly(args.out, f'wrong-map-tree-{i}', change)
            with self.assertRaisesRegex(ValueError, 'inside its topic tree'):
                m.verify(out)
        # A link lexically within the topic must not redirect to a lab map.
        link = self.direction.parent/'lab-link.json'
        link.symlink_to(self.root/'narration_lab/x/map.json')
        self.plan['direction_contract']['map']['path'] = str(link.relative_to(self.root))
        self.save_plan()
        with self.assertRaisesRegex(ValueError, 'inside its topic tree'):
            m.read_units(self.script, self.units, require_directions=True)

    def test_bound_direction_cannot_vanish_when_cue_is_renamed(self):
        self.bind_direction();m.read_units(self.script,self.units)
        self.script.write_text(self.script.read_text().replace('01_','98_'))
        self.plan['units'][0]['line_ids']=['98_1','98_2']
        self.plan['script_sha256']=m.cache.digest(self.script)
        self.plan['direction_contract']['script_sha256']=m.cache.digest(self.script)
        self.save_plan()
        with self.assertRaisesRegex(ValueError,'unknown cue'):
            m.read_units(self.script,self.units)

    def test_bound_direction_detects_rewording_after_rehash(self):
        self.bind_direction()
        self.script.write_text(self.script.read_text().replace('Four pulses','Three pulses'))
        self.plan['units'][0]['text']=self.plan['units'][0]['text'].replace('Four pulses','Three pulses')
        self.plan['script_sha256']=m.cache.digest(self.script)
        self.plan['direction_contract']['script_sha256']=m.cache.digest(self.script)
        self.save_plan()
        with self.assertRaisesRegex(ValueError,'missing or ambiguous'):
            m.read_units(self.script,self.units)

    def test_bound_direction_rejects_changed_map(self):
        self.bind_direction();self.direction.write_text('{"overrides":[]}')
        with self.assertRaisesRegex(ValueError,'map changed'):m.read_units(self.script,self.units)

    def test_bound_direction_requires_disposition_and_evidence(self):
        self.bind_direction();contract=m.direction_contract(self.script,self.plan)
        with self.assertRaisesRegex(ValueError,'explicit assembly disposition'):
            m.review_directions(contract,self.plan['units'],{})
        evidence=self.root/'review.md';evidence.write_text('Synthetic boundary evidence; not listening.')
        review={'override_index':0,'kind':'retained_source','reason':'Synthetic fixture preserves its source boundary.',
          'evidence':{'path':str(evidence),'sha256':m.cache.digest(evidence)}}
        result=m.review_directions(contract,self.plan['units'],{'direction_dispositions':[review]})
        self.assertEqual(len(result['dispositions']),1)
        with self.assertRaisesRegex(ValueError,'explicit assembly disposition'):
            m.review_directions(contract,self.plan['units'],{'direction_dispositions':[review,review]})
        evidence.write_text('Changed.')
        with self.assertRaisesRegex(ValueError,'evidence changed'):
            m.review_directions(contract,self.plan['units'],{'direction_dispositions':[review]})

    def test_bound_direction_refuses_duplicate_boundary(self):
        self.bind_direction();data=json.loads(self.direction.read_text());data['overrides']*=2
        self.direction.write_text(json.dumps(data))
        self.plan['direction_contract']['map']['sha256']=m.cache.digest(self.direction)
        self.save_plan()
        with self.assertRaisesRegex(ValueError,'duplicate direction'):
            m.read_units(self.script,self.units)

    def test_production_caller_can_require_contract_for_legacy_plan(self):
        with self.assertRaisesRegex(ValueError,'requires a direction contract'):
            m.read_units(self.script,self.units,require_directions=True)
        self.bind_direction();m.read_units(self.script,self.units,require_directions=True)

    def bound_assembly(self):
        self.bind_direction()
        record, _ = self.source()
        record['gap_after_samples'] = [0, 0, 0]
        evidence = self.root/'review.txt'
        evidence.write_text('Synthetic retained-boundary fixture; no listening.')
        manifest = {'schema_version': 1, 'units_sha256': m.cache.digest(self.units),
            'boundary_review': {'reviewer': 'unit-test', 'basis': 'synthetic', 'listening': 'unreviewed'},
            'units': [record], 'direction_dispositions': [{'override_index': 0,
                'kind': 'retained_source', 'reason': 'Keep synthetic source.',
                'evidence': {'path': str(evidence), 'sha256': m.cache.digest(evidence)}}]}
        spans = self.root/'spans.json'; spans.write_text(json.dumps(manifest))
        args = types.SimpleNamespace(script=self.script, units=self.units, spans=spans,
                                     out=self.root/'out', chapter='01', require_directions=True)
        m.assemble(args)
        return args

    def altered_assembly(self, original, label, change):
        out = self.root/label
        shutil.copytree(original, out)
        report = json.loads((out/'report.json').read_text())
        receipt = json.loads((out/'assembly.json').read_text())
        change(report, receipt)
        (out/'report.json').write_text(json.dumps(report))
        receipt['report_sha256'] = m.cache.digest(out/'report.json')
        (out/'assembly.json').write_text(json.dumps(receipt))
        return out

    def test_verify_requires_external_inputs_for_required_directions(self):
        with self.assertRaisesRegex(ValueError, 'external script'):
            m.verify(self.root/'absent', require_directions=True)
        with self.assertRaisesRegex(ValueError, 'both script'):
            m.verify(self.root/'absent', self.script)
        with self.assertRaisesRegex(ValueError, 'external script'):
            m.verify(self.root/'absent', chapter='01')

    def test_verify_refuses_rehashed_missing_direction_disposition(self):
        args = self.bound_assembly()
        self.assertTrue(m.verify(args.out, self.script, self.units, require_directions=True))
        def drop(report, receipt):
            report['meta']['direction_contract']['dispositions'] = []
            receipt['manifest']['direction_dispositions'] = []
        out = self.altered_assembly(args.out, 'missing-disposition', drop)
        for strict in (False, True):
            with self.assertRaisesRegex(ValueError, 'explicit assembly disposition'):
                if strict:m.verify(out, self.script, self.units, require_directions=True)
                else:m.verify(out)

    def test_strict_verify_refuses_stripped_or_self_rebound_contract(self):
        args = self.bound_assembly()
        expected = {
            'strip': 'direction binding differs',
            'resolve': 'explicit assembly disposition',
            'script': 'does not bind the supplied script and unit plan',
            'units': 'does not bind the supplied script and unit plan',
            'manifest': 'manifest differs from the requested unit plan/scope',
            'rate': 'metadata does not describe this adapter and sample rate',
            'adapter': 'metadata does not describe this adapter and sample rate',
            'consistent-rebind': 'direction binding differs',
            'manifest-rebind': 'explicit assembly disposition',
        }
        for mutation, message in expected.items():
            def change(report, receipt):
                if mutation == 'strip':report['meta']['direction_contract'] = None
                elif mutation == 'resolve':report['meta']['direction_contract']['binding']['resolved'] = []
                elif mutation in ('consistent-rebind', 'manifest-rebind'):
                    report['meta']['direction_contract']['binding']['resolved'] = []
                    report['meta']['direction_contract']['dispositions'] = []
                    if mutation == 'manifest-rebind':
                        receipt['manifest']['direction_dispositions'] = []
                elif mutation in ('script', 'units'):report['meta'][mutation+'_sha256'] = '0'*64
                elif mutation == 'rate':report['meta']['rate'] = 48000
                elif mutation == 'adapter':report['meta']['adapter'] = 'different-adapter'
                else:receipt['manifest']['units'][0]['id'] = 'different_unit'
            out = self.altered_assembly(args.out, mutation, change)
            if mutation in ('consistent-rebind', 'manifest-rebind'):
                self.assertTrue(m.verify(out))
            with self.subTest(mutation=mutation), self.assertRaisesRegex(ValueError, message):
                m.verify(out, self.script, self.units, require_directions=True)

    def test_boundary_review_schema_and_equality_in_both_verify_modes(self):
        args = self.bound_assembly()
        valid = {'reviewer': 'unit-test', 'basis': 'synthetic', 'listening': 'unreviewed'}
        cases = [None, [], {}, {**valid, 'reviewer': ' '}, {**valid, 'basis': 7},
                 {**valid, 'listening': 'every word heard'}]
        for index, review in enumerate(cases):
            def change(report, receipt):
                report['meta']['boundary_review'] = review
                receipt['manifest']['boundary_review'] = review
            out = self.altered_assembly(args.out, f'review-malformed-{index}', change)
            for strict in (False, True):
                with self.subTest(index=index, strict=strict), self.assertRaisesRegex(
                        ValueError, 'record boundary inspection basis'):
                    m.verify(out, *([self.script, self.units] if strict else []))
            manifest = json.loads(args.spans.read_text())
            manifest['boundary_review'] = review
            spans = self.root/f'malformed-spans-{index}.json'
            spans.write_text(json.dumps(manifest))
            invalid = types.SimpleNamespace(**vars(args))
            invalid.spans = spans; invalid.out = self.root/f'malformed-assembly-{index}'
            with self.assertRaisesRegex(ValueError, 'record boundary inspection basis'):
                m.assemble(invalid)
            self.assertFalse(invalid.out.exists())
        def disagree(report, _):
            report['meta']['boundary_review'] = {**valid, 'listening': 'reviewed'}
        out = self.altered_assembly(args.out, 'review-disagrees', disagree)
        for strict in (False, True):
            with self.subTest(strict=strict), self.assertRaisesRegex(ValueError, 'boundary review differs'):
                m.verify(out, *([self.script, self.units] if strict else []))

    def test_consistent_reviewed_claim_is_not_listening_evidence(self):
        args = self.bound_assembly()
        def claim(report, receipt):
            review = {'reviewer': 'invented', 'basis': 'unproven', 'listening': 'reviewed'}
            report['meta']['boundary_review'] = review
            receipt['manifest']['boundary_review'] = review
        out = self.altered_assembly(args.out, 'unproven-review-claim', claim)
        # Schema/equality cannot authenticate a reviewer or establish hearing.
        self.assertTrue(m.verify(out))
        self.assertTrue(m.verify(out, self.script, self.units, require_directions=True))

    def test_strict_verify_refuses_rehashed_text_and_source_unit_changes(self):
        args = self.bound_assembly()
        for mutation in ('sentence', 'unit_id'):
            def change(report, _):
                row = report['lines']['01_2'][0]
                row[mutation] = 'Do drop the negative.' if mutation == 'sentence' else 'unknown'
            out = self.altered_assembly(args.out, mutation, change)
            # Identical PCM alone does not detect corrupted descriptive text.
            self.assertTrue(m.verify(out))
            with self.assertRaisesRegex(ValueError, 'sentence/source spans'):
                m.verify(out, self.script, self.units, require_directions=True)

    def test_strict_verify_rechecks_generation_request_text(self):
        args = self.bound_assembly()
        receipt_path = self.root/'generation.json'
        receipt = json.loads(receipt_path.read_text())
        (self.root/'generation.before.json').write_bytes(receipt_path.read_bytes())
        receipt['request']['submitted_text'] = receipt['request']['submitted_text'].replace('not ', '')
        receipt_path.write_text(json.dumps(receipt))
        self.assertTrue(m.verify(args.out))
        with self.assertRaisesRegex(ValueError, "generation request does not contain"):
            m.verify(args.out, self.script, self.units, require_directions=True)

    def test_strict_verify_requires_explicit_partial_chapter_scope(self):
        self.script.write_text('01_1\tFour pulses. They return.\n02_1\tDo not drop the negative.\n')
        self.plan['script_sha256'] = m.cache.digest(self.script)
        self.plan['units'] = [
            {'id': '01u1', 'line_ids': ['01_1'], 'text': 'Four pulses. They return.'},
            {'id': '02u1', 'line_ids': ['02_1'], 'text': 'Do not drop the negative.'}]
        self.save_plan(); self.bind_direction()
        record, _ = self.source()
        record['sentence_ends'] = [1200, 2400]; record['gap_after_samples'] = [0, 0]
        evidence = self.root/'review.txt'; evidence.write_text('Synthetic scope check.')
        spans = self.root/'spans.json'
        spans.write_text(json.dumps({'schema_version': 1, 'units_sha256': m.cache.digest(self.units),
            'boundary_review': {'reviewer': 'test', 'basis': 'synthetic', 'listening': 'unreviewed'},
            'units': [record], 'direction_dispositions': [{'override_index': 0, 'kind': 'retained_source',
                'reason': 'Synthetic source.', 'evidence': {'path': str(evidence), 'sha256': m.cache.digest(evidence)}}]}))
        args = types.SimpleNamespace(script=self.script, units=self.units, spans=spans,
                                     out=self.root/'out', chapter='01', require_directions=True)
        m.assemble(args)
        self.assertTrue(m.verify(args.out, self.script, self.units, require_directions=True, chapter='01'))
        for scope in (None, '02'):
            with self.assertRaisesRegex(ValueError, 'cue coverage'):
                m.verify(args.out, self.script, self.units, require_directions=True, chapter=scope)

    def test_false_contract_fails_cleanly(self):
        self.plan['direction_contract']=False;self.save_plan()
        with self.assertRaisesRegex(ValueError,'does not bind'):
            m.read_units(self.script,self.units)

    def test_disposition_must_agree_with_samples(self):
        self.bind_direction();binding=m.direction_contract(self.script,self.plan)
        lines={'01_1':[{'sentence':'Four pulses.','gap_after_samples':0},{'sentence':'They return.','gap_after_samples':0}]}
        for kind in ['authored_silence','native_visual_hold']:
            directions={'binding':binding,'dispositions':[{'override_index':0,'kind':kind}]}
            with self.assertRaisesRegex(ValueError,'contradicts assembled samples'):
                m.check_dispositions(directions,lines)
        lines['01_1'][0]['gap_after_samples']=4800
        with self.assertRaisesRegex(ValueError,'contradicts assembled samples'):
            m.check_dispositions({'binding':binding,'dispositions':[{'override_index':0,'kind':'retained_source'}]},lines)
        m.check_dispositions({'binding':binding,'dispositions':[{'override_index':0,'kind':'authored_silence'}]},lines)

    def test_exact_coverage(self):
        script, units = m.read_units(self.script, self.units)
        self.assertEqual(list(script), ["01_1", "01_2"])
        self.assertEqual(len(units), 1)

    def test_omission_duplicate_reorder_and_changed_negation(self):
        original = copy.deepcopy(self.plan)
        for cues in (["01_1"], ["01_1", "01_1", "01_2"], ["01_2", "01_1"]):
            self.plan = copy.deepcopy(original)
            self.plan["units"][0]["line_ids"] = cues
            script = m.read_script(self.script)
            self.plan["units"][0]["text"] = " ".join(script[c] for c in cues)
            self.save_plan()
            with self.assertRaises(ValueError):
                m.read_units(self.script, self.units)
        self.plan = original
        self.plan["units"][0]["text"] = self.plan["units"][0]["text"].replace("not ", "")
        self.save_plan()
        with self.assertRaises(ValueError):
            m.read_units(self.script, self.units)

    def test_script_hash_and_duplicate_script_ids(self):
        self.script.write_text(self.script.read_text().replace("negative", "positive"))
        with self.assertRaises(ValueError):
            m.read_units(self.script, self.units)
        self.script.write_text("01_1\tOne.\n01_1\tTwo.\n")
        with self.assertRaises(ValueError):
            m.read_script(self.script)

    def test_cross_chapter_and_oversized_units(self):
        self.script.write_text("01_1\tOne.\n02_1\tTwo.\n")
        self.plan["script_sha256"] = m.cache.digest(self.script)
        self.plan["units"][0].update(line_ids=["01_1", "02_1"], text="One. Two.")
        self.save_plan()
        with self.assertRaises(ValueError):
            m.read_units(self.script, self.units)
        text = "word " * 100 + "end."
        self.script.write_text("01_1\t" + text + "\n")
        self.plan["script_sha256"] = m.cache.digest(self.script)
        self.plan["units"][0].update(line_ids=["01_1"], text=text)
        self.save_plan()
        with self.assertRaises(ValueError):
            m.read_units(self.script, self.units)

    def source(self):
        """Synthetic PCM and producer contract fixture; never live synthesis proof."""
        wav = self.root / "raw.wav"
        with wave.open(str(wav), "wb") as w:
            w.setparams((1, 2, 24000, 0, "NONE", "not compressed"))
            w.writeframes(b"".join(struct.pack("<h", n % 200 - 100) for n in range(2400)))
        assets = {"talker_sha256": "model", "codec_sha256": "codec", "voice_source_sha256": "voice"}
        for name in ("vocab.json", "merges.txt", "tokenizer_config.json"):
            assets[name] = {"sha256": name}
        generation = {"schema": "ftts.generation", "schema_version": 1, "run_id": "unit-fixture",
            "state": "completed", "execution": "inline", "requested": {"seed": 2},
            "effective": {"seed": 2, "normalization": {"mode": "verbatim"}, "utterance_start": "fresh",
                          "sampling": {"mode": "production"}},
            "output": {"samples": 2400, "frames": 10, "frame_limit": 600, "sample_rate": 24000,
                       "stop_reason": "end_of_speech"}, "producer": {"executable_sha256": "exe"},
            "voice": {"route": "xvector"}, "assets": assets}
        context = {"settings": {"normalize": "verbatim"}, "executable": {"sha256": "exe"},
            "voice": {"kind": "file", "input": {"sha256": "voice"}}, "model": "/synthetic/model.fttsq",
            "bundle": [{"path": "/synthetic/model.fttsq", "sha256": "model"},
                       {"path": "/synthetic/speech_tokenizer/model.safetensors", "sha256": "codec"}] +
                      [{"path": "/synthetic/" + n, "sha256": n} for n in ("vocab.json", "merges.txt", "tokenizer_config.json")]}
        completion = {"run_id": "unit-fixture", "event": "run_complete", "exit_code": 0,
                      "samples": 2400, "frames": 10}
        receipt = {"wav": m.cache.wav_identity(wav), "returncode": 0, "generation": generation,
                   "completion": completion, "request": {"seed": 2, "context": context,
                   "submitted_text": self.plan["units"][0]["text"]}}
        receipt_path = self.root / "generation.json"
        receipt_path.write_text(json.dumps(receipt))
        return {"id": "01u1", "wav": str(wav), "sha256": m.cache.digest(wav),
                "generation_receipt": str(receipt_path), "sentence_ends": [600, 1500, 2400],
                "gap_after_samples": [20, 0, 30]}, receipt

    def test_sample_roundtrip_report_and_overwrite_refusal(self):
        record, _ = self.source()
        spans = self.root / "spans.json"
        spans.write_text(json.dumps({"schema_version": 1, "units_sha256": m.cache.digest(self.units),
            "boundary_review": {"reviewer": "unit-test", "basis": "synthetic boundary fixture",
                                "listening": "unreviewed"}, "units": [record]}))
        out = self.root / "out"
        args = types.SimpleNamespace(script=self.script, units=self.units, spans=spans, out=out, chapter="01")
        m.assemble(args)
        self.assertTrue(m.verify(out))
        report = json.loads((out / "report.json").read_text())
        self.assertEqual([r["sentence"] for r in report["lines"]["01_1"]], ["Four pulses.", "They return."])
        self.assertEqual(m.cache.wav_identity(out / "01_1.wav")["frames"], 1520)
        with self.assertRaises(FileExistsError):
            m.assemble(args)
        with (out / "01_1.wav").open("r+b") as f:
            f.seek(46)
            f.write(b"\x00\x01")
        with self.assertRaises(ValueError):
            m.verify(out)

    def test_external_ui_source_binds_text_audio_and_complete_cue(self):
        original, _ = self.source()
        wav = Path(original['wav'])
        mp3 = self.root / 'download.mp3'
        mp3.write_bytes(b'illustrative downloaded bytes')
        unit = {'id': '01u1', 'line_ids': ['01_1'], 'text': 'Four pulses. They return.'}
        info = m.cache.wav_identity(wav)
        receipt = {'source': 'ElevenLabs UI, Oliver', 'records': [{
            'cue': '01_1', 'text': unit['text'], 'wav': str(wav),
            'wav_sha256': info['sha256'], 'samples': info['frames'], 'rate': 24000,
            'source': str(mp3), 'source_sha256': m.cache.digest(mp3)}]}
        receipt_path = self.root / 'external.json'
        receipt_path.write_text(json.dumps(receipt))
        record = {'wav': str(wav), 'generation_receipt': str(receipt_path),
                  'sha256': info['sha256'], 'sentence_ends': [1000, 2400],
                  'gap_after_samples': [0, 0]}
        rows = m.unit_spans(unit, m.read_script(self.script), record)
        self.assertEqual(len(rows), 2)
        self.assertEqual(b''.join(row[2] for row in rows), m.wav_pcm(wav)[1])
        receipt['records'][0]['text'] = 'Four pulses. They vanish.'
        receipt_path.write_text(json.dumps(receipt))
        with self.assertRaisesRegex(ValueError, 'external source'):
            m.unit_spans(unit, m.read_script(self.script), record)
        receipt['records'][0]['text'] = unit['text']
        receipt_path.write_text(json.dumps(receipt))
        mp3.write_bytes(b'changed download')
        with self.assertRaisesRegex(ValueError, 'external source'):
            m.unit_spans(unit, m.read_script(self.script), record)

    def test_source_gap_overlap_cap_and_content_refused(self):
        record, receipt = self.source()
        script = m.read_script(self.script)
        unit = self.plan["units"][0]
        for ends in ([600, 600, 2400], [600, 1500, 2399], [600, 2500, 2400], [600, 2400]):
            bad = dict(record, sentence_ends=ends)
            with self.assertRaises(ValueError):
                m.unit_spans(unit, script, bad)
        for reason in ("frame_limit", "cancelled"):
            bad_receipt = copy.deepcopy(receipt)
            bad_receipt["generation"]["output"]["stop_reason"] = reason
            Path(record["generation_receipt"]).write_text(json.dumps(bad_receipt))
            with self.assertRaises(ValueError):
                m.unit_spans(unit, script, record)
        receipt["request"]["submitted_text"] = unit["text"].replace("not ", "")
        Path(record["generation_receipt"]).write_text(json.dumps(receipt))
        with self.assertRaises(ValueError):
            m.unit_spans(unit, script, record)

    def test_contextual_target_preserves_prefix_and_refuses_unbound_crops(self):
        record, receipt = self.source()
        script = m.read_script(self.script)
        script["02_1"] = "The next complete thought follows."
        unit = self.plan["units"][0]
        expected = unit["text"] + " " + script["02_1"]
        self.assertEqual(m.request_text(unit, script, "02_1"), expected)
        for wrong in ("01_1", "02_2", ""):
            with self.assertRaises(ValueError):
                m.request_text(unit, script, wrong)
        with self.assertRaises(ValueError):
            m.request_text(unit, {**script, "02_1": "word " * 100}, "02_1")
        pickup = dict(record, context_after="02_1", target_end_sample=2100,
                      sentence_ends=[600, 1500, 2100])
        # A claimed pickup cannot relabel a normal producer receipt.
        with self.assertRaises(ValueError):
            m.unit_spans(unit, script, pickup)
        receipt["request"]["submitted_text"] = expected
        Path(record["generation_receipt"]).write_text(json.dumps(receipt))
        rows = m.unit_spans(unit, script, pickup)
        _, raw = m.wav_pcm(Path(record["wav"]))
        self.assertEqual(b"".join(pcm[:r["out_samples"] * 2] for _, r, pcm in rows), raw[:2100 * 2])
        self.assertEqual([r["sentence"] for _, r, _ in rows], m.sentences(unit["text"]))
        for bad in (dict(pickup, target_end_sample=True), dict(pickup, target_end_sample=2400),
                    dict(pickup, sentence_ends=[600, 1500, 2099]),
                    dict(pickup, context_after=None), dict(pickup, context_after="01_1")):
            with self.assertRaises(ValueError):
                m.unit_spans(unit, script, bad)
        receipt["request"]["submitted_text"] = unit["text"]
        Path(record["generation_receipt"]).write_text(json.dumps(receipt))
        with self.assertRaises(ValueError):
            m.unit_spans(unit, script, dict(record, target_end_sample=2100))

    def intra_fixture(self):
        record, receipt = self.source()
        _, raw = m.wav_pcm(Path(record['wav']))
        raw = raw[:480] + bytes(40) + raw[520:1780] + bytes(40) + raw[1820:]
        with wave.open(record['wav'], 'wb') as stream:
            stream.setparams((1, 2, 24000, 0, 'NONE', 'not compressed'))
            stream.writeframes(raw)
        record['sha256'] = m.cache.digest(record['wav'])
        receipt['wav'] = m.cache.wav_identity(Path(record['wav']))
        Path(record['generation_receipt']).write_text(json.dumps(receipt))
        self.bind_direction()
        self.direction.write_text(json.dumps({'overrides':[
            {'video':'example','cue':'01_1','after':'Four'},
            {'video':'example','cue':'01_1','after':'They'}]}))
        self.plan['direction_contract']['map']['sha256'] = m.cache.digest(self.direction)
        self.save_plan()
        record['intra_sentence_pauses'] = [
            {'override_index':0,'source_sample':250,'inserted_samples':40,'source_zero':[240,260]},
            {'override_index':1,'source_sample':900,'inserted_samples':60,'source_zero':[890,910]}]
        evidence=self.root/'review.txt';evidence.write_text('Synthetic only; no acoustic claims.')
        manifest={'schema_version':1,'units_sha256':m.cache.digest(self.units),
            'boundary_review':{'reviewer':'unit-test','basis':'synthetic exact samples','listening':'unreviewed'},
            'units':[record], 'direction_dispositions':[
                {'override_index':i,'kind':'authored_silence','reason':'Synthetic internal pause.',
                 'evidence':{'path':str(evidence),'sha256':m.cache.digest(evidence)}} for i in range(2)]}
        spans=self.root/'spans.json';spans.write_text(json.dumps(manifest))
        args=types.SimpleNamespace(script=self.script,units=self.units,spans=spans,out=self.root/'out',chapter='01',require_directions=True)
        return args,manifest,raw

    def test_two_internal_pauses_preserve_samples_and_native_sentence_counts(self):
        args,manifest,raw=self.intra_fixture()
        m.assemble(args)
        _,actual=m.wav_pcm(args.out/'01_1.wav')
        expected=raw[:500]+bytes(80)+raw[500:1200]+bytes(40)+raw[1200:1800]+bytes(120)+raw[1800:3000]
        self.assertEqual(actual,expected)
        report=json.loads((args.out/'report.json').read_text())
        self.assertEqual([r['out_samples'] for r in report['lines']['01_1']],[640,960])
        self.assertTrue(m.verify(args.out))
        report['lines']['01_1'][0]['out_samples']-=1
        (args.out/'report.json').write_text(json.dumps(report))
        assembly=json.loads((args.out/'assembly.json').read_text())
        assembly['report_sha256']=m.cache.digest(args.out/'report.json')
        (args.out/'assembly.json').write_text(json.dumps(assembly))
        with self.assertRaisesRegex(ValueError,'sample count'):
            m.verify(args.out)

    def test_internal_pause_rejects_wrong_anchor_missing_disposition_and_unbound_plan(self):
        args,manifest,_=self.intra_fixture()
        for mutation in ('wrong_anchor','retained_claim','missing_disposition','unbound'):
            bad=copy.deepcopy(manifest)
            if mutation=='wrong_anchor':bad['units'][0]['intra_sentence_pauses'][0]['override_index']=1
            elif mutation=='retained_claim':bad['direction_dispositions'][0]['kind']='retained_source'
            elif mutation=='missing_disposition':bad['direction_dispositions'].pop()
            else:
                self.plan.pop('direction_contract');self.save_plan()
                args.require_directions=False;bad['units_sha256']=m.cache.digest(self.units)
            args.spans.write_text(json.dumps(bad))
            with self.assertRaises(ValueError):m.assemble(args)
            self.assertFalse(args.out.exists())

    def test_internal_pause_rejects_duplicate_boundary_and_false_zero_claim(self):
        args,manifest,_=self.intra_fixture()
        record=manifest['units'][0]
        for at in (0,600,2400,True):
            bad=copy.deepcopy(record);bad['intra_sentence_pauses'][0]['source_sample']=at
            with self.assertRaises(ValueError):m.unit_spans(self.plan['units'][0],m.read_script(self.script),bad)
        for changes in ({'source_zero':[241,260]}, {'source_zero':[239,260]},
                        {'inserted_samples':0}, {'inserted_samples':120001}, {'inserted_samples':True}):
            bad=copy.deepcopy(record);bad['intra_sentence_pauses'][0].update(changes)
            with self.assertRaises(ValueError):m.unit_spans(self.plan['units'][0],m.read_script(self.script),bad)
        bad=copy.deepcopy(record);bad['intra_sentence_pauses'].append(bad['intra_sentence_pauses'][0])
        with self.assertRaises(ValueError):m.unit_spans(self.plan['units'][0],m.read_script(self.script),bad)

    # ElevenLabs UI takes. The labels and voices are written out here, not read
    # from EXTERNAL_UI_VOICES, so a changed table entry fails these tests.
    UI = {'frederick': ('ElevenLabs UI, Frederick Surrey', 'Frederick Surrey - Smooth and Velvety'),
          'oliver': ('ElevenLabs UI, Oliver', 'Oliver - Clean, British and Steady')}
    UI_SCHEMA = 'elevenlabs-ui-receipt-v2'
    UI_SPANS = {'01_1': ([1000, 2400], [0, 0]), '01_2': ([1200], [0])}

    def ui_take(self, cue, kind):
        """One decoded synthetic take of one cue and its receipt; never proof of a voice.

        kind is 'frederick' or 'oliver' (v2 receipt with a bound request) or
        'legacy' (the unversioned Oliver receipt of the retained takes).
        """
        self.ui_takes = getattr(self, 'ui_takes', 0) + 1
        folder = self.root/'ui'/f'{self.ui_takes:02d}-{cue}-{kind}'
        folder.mkdir(parents=True)
        wav = folder/'take.wav'
        ends, gaps = self.UI_SPANS[cue]
        with wave.open(str(wav), 'wb') as w:
            w.setparams((1, 2, 24000, 0, 'NONE', 'not compressed'))
            w.writeframes(b''.join(struct.pack('<h', (n * 7 + self.ui_takes) % 300 - 150) for n in range(ends[-1])))
        download = folder/'download.wav'
        download.write_bytes(f'synthetic stand-in for the retained download of {cue}, take {self.ui_takes}'.encode())
        info = m.cache.wav_identity(wav)
        row = {'cue': cue, 'text': m.read_script(self.script)[cue], 'wav': str(wav), 'wav_sha256': info['sha256'],
               'samples': info['frames'], 'rate': 24000, 'source': str(download), 'source_sha256': m.cache.digest(download)}
        if kind == 'legacy':
            receipt = {'source': self.UI['oliver'][0], 'records': [row]}
        else:
            label, voice = self.UI[kind]
            request = folder/'request.json'
            request.write_text(json.dumps({'cue': cue, 'text': row['text'], 'voice': voice, 'model': 'Eleven v4'}))
            row.update(voice=voice, request=str(request), request_sha256=m.cache.digest(request))
            receipt = {'schema': self.UI_SCHEMA, 'source': label, 'provider': 'ElevenLabs', 'voice': voice, 'records': [row]}
        (folder/'receipt.json').write_text(json.dumps(receipt))
        return {'id': 'u' + cue, 'wav': str(wav), 'sha256': info['sha256'], 'generation_receipt': str(folder/'receipt.json'),
                'sentence_ends': ends, 'gap_after_samples': gaps}

    def native_take(self, cue):
        """A one-cue native FrankenTTS take; source() binds the first plan unit's text, so cue 01_1 only."""
        record, _ = self.source()
        record.update(id='u' + cue, sentence_ends=[1000, 2400], gap_after_samples=[0, 0])
        return record

    def external_assembly(self, kinds, label='external'):
        """One-cue units under a bound direction contract, one take kind per cue."""
        self.bind_direction()
        script = m.read_script(self.script)
        self.plan['units'] = [{'id': 'u' + cue, 'line_ids': [cue], 'text': text} for cue, text in script.items()]
        self.save_plan()
        evidence = self.root/'ui-review.txt'
        evidence.write_text('Synthetic retained-boundary fixture; no listening.')
        manifest = {'schema_version': 1, 'units_sha256': m.cache.digest(self.units),
            'boundary_review': {'reviewer': 'unit-test', 'basis': 'synthetic', 'listening': 'unreviewed'},
            'units': [self.native_take(cue) if kind == 'native' else self.ui_take(cue, kind)
                      for cue, kind in zip(script, kinds)],
            'direction_dispositions': [{'override_index': 0, 'kind': 'retained_source', 'reason': 'Keep synthetic source.',
                'evidence': {'path': str(evidence), 'sha256': m.cache.digest(evidence)}}]}
        spans = self.root/f'{label}-spans.json'; spans.write_text(json.dumps(manifest))
        return types.SimpleNamespace(script=self.script, units=self.units, spans=spans,
                                     out=self.root/f'{label}-out', chapter='01', require_directions=True)

    def swap_voice(self, record, kind):
        """Rewrite one take's receipt as a self-consistent v2 receipt of another voice."""
        path = Path(record['generation_receipt'])
        row = json.loads(path.read_text())['records'][0]
        label, voice = self.UI[kind]
        request = path.with_name('swapped-request.json')
        request.write_text(json.dumps({'cue': row['cue'], 'text': row['text'], 'voice': voice}))
        row.update(voice=voice, request=str(request), request_sha256=m.cache.digest(request))
        path.write_text(json.dumps({'schema': self.UI_SCHEMA, 'source': label, 'provider': 'ElevenLabs',
                                    'voice': voice, 'records': [row]}))

    def test_native_limit_and_explicit_external_declaration(self):
        for count in (100, 101, 205):
            text = ' '.join(['word'] * count)
            self.script.write_text('01_1\t' + text + '\n')
            self.plan['script_sha256'] = m.cache.digest(self.script)
            unit = {'id': 'u01_1', 'line_ids': ['01_1'], 'text': text}
            self.plan['units'] = [unit]
            self.save_plan()
            if count == 100:
                m.read_units(self.script, self.units)
            else:
                with self.assertRaisesRegex(ValueError, 'at most 100'):
                    m.read_units(self.script, self.units)
            unit['external_ui_provider'] = self.UI['frederick'][0]
            self.save_plan()
            m.read_units(self.script, self.units)
            for bad in (None, False, [], {}, '', 'ElevenLabs', 1):
                unit['external_ui_provider'] = bad
                self.save_plan()
                with self.assertRaisesRegex(ValueError, 'known external UI provider'):
                    m.read_units(self.script, self.units)
            unit['external_ui_provider'] = self.UI['frederick'][0]
            unit['text'] = text + ' changed'
            self.save_plan()
            with self.assertRaisesRegex(ValueError, 'exact nonempty'):
                m.read_units(self.script, self.units)

    def test_external_group_and_native_generation_refused(self):
        self.plan['units'][0]['external_ui_provider'] = self.UI['frederick'][0]
        self.save_plan()
        with self.assertRaisesRegex(ValueError, 'one complete cue'):
            m.read_units(self.script, self.units)
        self.plan['units'] = [dict(id='u'+cue, line_ids=[cue], text=text,
            external_ui_provider=self.UI['frederick'][0]) for cue, text in m.read_script(self.script).items()]
        self.save_plan()
        args = types.SimpleNamespace(script=self.script, units=self.units, out=self.root/'must-not-exist')
        with patch.object(m.cache, 'child_environment') as env, patch.object(m.cache, 'make_context') as context:
            with self.assertRaisesRegex(ValueError, 'cannot enter native synthesis'):
                m.generate(args)
            env.assert_not_called()
            context.assert_not_called()
        self.assertFalse(args.out.exists())

    def test_long_external_cue_strict_assembly_and_receipt_binding(self):
        # Preserve the two-sentence fixture and its source direction anchor.
        text = 'Four pulses. ' + ' '.join(['word'] * 202) + ' return.'
        self.assertEqual(len(text.split()), 205)
        self.script.write_text('01_1\t' + text + '\n01_2\tDo not drop the negative.\n')
        self.plan['script_sha256'] = m.cache.digest(self.script)
        args = self.external_assembly(('frederick', 'frederick'), 'long')
        for unit in self.plan['units']:
            unit['external_ui_provider'] = self.UI['frederick'][0]
        self.save_plan()
        manifest = json.loads(args.spans.read_text())
        manifest['units_sha256'] = m.cache.digest(self.units)
        args.spans.write_text(json.dumps(manifest))
        m.assemble(args)
        self.assertTrue(m.verify(args.out, self.script, self.units, require_directions=True))
        delivery_spec = importlib.util.spec_from_file_location('delivery823', Path(__file__).with_name('narration_delivery.py'))
        delivery = importlib.util.module_from_spec(delivery_spec)
        delivery_spec.loader.exec_module(delivery)
        external = args.out/'external-units.json'
        external.write_bytes(self.units.read_bytes())
        chosen = delivery.external_plan(self.root, args.out, self.units)
        self.assertEqual(chosen, external)
        self.assertTrue(m.verify(args.out, self.script, chosen, require_directions=True))
        changed = copy.deepcopy(self.plan)
        changed['direction_contract']['map']['sha256'] = '0' * 64
        external.write_text(json.dumps(changed))
        with self.assertRaisesRegex(ValueError, 'changes the canonical direction contract'):
            delivery.external_plan(self.root, args.out, self.units)
        record = manifest['units'][0]
        self.assertEqual(m.wav_pcm(args.out/'01_1.wav')[1], m.wav_pcm(Path(record['wav']))[1])
        unit = self.plan['units'][0]
        unit['external_ui_provider'] = self.UI['oliver'][0]
        with self.assertRaisesRegex(ValueError, 'declaration does not match'):
            m.unit_spans(unit, m.read_script(self.script), record)
        unit['external_ui_provider'] = self.UI['frederick'][0]
        bad = copy.deepcopy(record)
        bad['sentence_ends'][-1] -= 1
        with self.assertRaises(ValueError):
            m.unit_spans(unit, m.read_script(self.script), bad)

    def test_external_declaration_refuses_native_receipt(self):
        record, _ = self.source()
        unit = copy.deepcopy(self.plan['units'][0])
        unit['external_ui_provider'] = self.UI['frederick'][0]
        with self.assertRaisesRegex(ValueError, 'declaration does not match'):
            m.unit_spans(unit, m.read_script(self.script), record)

    def test_frederick_ui_takes_assemble_and_verify_sample_for_sample(self):
        args = self.external_assembly(('frederick', 'frederick'))
        m.assemble(args)
        self.assertTrue(m.verify(args.out, self.script, self.units, require_directions=True))
        records = json.loads(args.spans.read_text())['units']
        for unit, record in zip(self.plan['units'], records):
            self.assertEqual(m.wav_pcm(args.out/f"{unit['line_ids'][0]}.wav")[1], m.wav_pcm(Path(record['wav']))[1])
        self.assertEqual(m.assembly_voice(records), 'Frederick Surrey - Smooth and Velvety')

    def test_frederick_receipt_binds_provider_voice_and_request(self):
        args = self.external_assembly(('frederick', 'frederick'))
        record = json.loads(args.spans.read_text())['units'][0]
        path = Path(record['generation_receipt'])
        good = json.loads(path.read_text())
        row = good['records'][0]
        stated = json.loads(Path(row['request']).read_text())
        unit, script = self.plan['units'][0], m.read_script(self.script)

        def rebound(**fields):
            # The request is rewritten and rehashed, so only its content can refuse it.
            changed = Path(row['request']).with_name('rebound-' + '-'.join(fields) + '.json')
            changed.write_text(json.dumps(dict(stated, **fields)))
            return dict(good, records=[dict(row, request=str(changed), request_sha256=m.cache.digest(changed))])
        unversioned = {k: v for k, v in good.items() if k != 'schema'}
        for message, bad in (
                ('provider or voice', dict(good, provider='ElevenLabs UI')),
                ('provider or voice', dict(good, voice='Frederick Surrey')),
                ('every external UI record', dict(good, records=[dict(row, voice='Oliver - Clean, British and Steady')])),
                ('lacks request_sha256', dict(good, records=[{k: v for k, v in row.items() if k != 'request_sha256'}])),
                ('request file changed', dict(good, records=[dict(row, request_sha256='0' * 64)])),
                ('this cue, text and voice', rebound(cue='01_2')),
                ('this cue, text and voice', rebound(text='Four pulses. They vanish.')),
                ('this cue, text and voice', rebound(voice='Oliver - Clean, British and Steady')),
                ('no exact known source label', dict(good, source='ElevenLabs UI, Frederick')),
                ('needs schema', unversioned),
                ('neither native', dict(unversioned, source='Frederick Surrey'))):
            with self.subTest(message=message):
                path.write_text(json.dumps(bad))
                with self.assertRaisesRegex(ValueError, message):
                    m.unit_spans(unit, script, record)
        path.write_text(json.dumps(good))
        self.assertEqual(len(m.unit_spans(unit, script, record)), 2)

    def test_legacy_and_v2_oliver_receipts_assemble_as_one_voice(self):
        for kinds, label in ((('legacy', 'legacy'), 'legacy'), (('oliver', 'legacy'), 'both-forms')):
            with self.subTest(label=label):
                args = self.external_assembly(kinds, label)
                m.assemble(args)
                self.assertTrue(m.verify(args.out, self.script, self.units, require_directions=True))
                records = json.loads(args.spans.read_text())['units']
                self.assertEqual(m.assembly_voice(records), 'Oliver - Clean, British and Steady')

    def test_one_assembly_refuses_two_narrator_voices(self):
        for kinds in (('frederick', 'legacy'), ('legacy', 'frederick'), ('oliver', 'frederick'),
                      ('native', 'frederick'), ('native', 'legacy')):
            with self.subTest(kinds=kinds):
                args = self.external_assembly(kinds, '-'.join(kinds))
                with self.assertRaisesRegex(ValueError, 'one narrator voice'):
                    m.assemble(args)
                self.assertFalse(args.out.exists())

    def test_verify_refuses_a_voice_swapped_in_after_assembly(self):
        args = self.external_assembly(('frederick', 'frederick'))
        m.assemble(args)
        record = json.loads(args.spans.read_text())['units'][1]
        self.swap_voice(record, 'oliver')
        # The swapped receipt is self-consistent alone; only the one-voice guard sees the mix.
        self.assertEqual(len(m.unit_spans(self.plan['units'][1], m.read_script(self.script), record)), 1)
        with self.assertRaisesRegex(ValueError, 'one narrator voice'):
            m.verify(args.out, self.script, self.units, require_directions=True)

    def test_legacy_oliver_receipt_cannot_state_another_voice_or_request(self):
        args = self.external_assembly(('legacy', 'legacy'), 'misbound')
        record = json.loads(args.spans.read_text())['units'][0]
        path = Path(record['generation_receipt'])
        legacy = json.loads(path.read_text())
        row = legacy['records'][0]
        # The stage08 helper states the short name; the UI string is Oliver too.
        for voice in ('Oliver', 'Oliver - Clean, British and Steady'):
            self.assertEqual(m.external_ui_voice(dict(legacy, voice=voice)), 'Oliver - Clean, British and Steady')
        for bad in (dict(legacy, voice='Frederick Surrey'),
                    dict(legacy, records=[dict(row, voice='Frederick Surrey - Smooth and Velvety')]),
                    dict(legacy, records=[dict(row, request=str(path), request_sha256=m.cache.digest(path))])):
            with self.subTest(bad=bad):
                path.write_text(json.dumps(bad))
                with self.assertRaisesRegex(ValueError, 'legacy Oliver receipt states another voice or a request'):
                    m.unit_spans(self.plan['units'][0], m.read_script(self.script), record)
                with self.assertRaisesRegex(ValueError, 'legacy Oliver receipt'):
                    m.assemble(args)
                self.assertFalse(args.out.exists())


if __name__ == "__main__":
    unittest.main()
