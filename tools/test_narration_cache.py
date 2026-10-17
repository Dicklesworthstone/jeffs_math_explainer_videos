#!/usr/bin/env python3
"""Cache behavior with a declared synthetic producer, never acoustic evidence.

Fixtures are retained per AGENTS.md. Run: python3 -I tools/test_narration_cache.py
"""
import copy
import importlib.util
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

spec = importlib.util.spec_from_file_location("narration_cache", Path(__file__).with_name("narration_cache.py"))
cache = importlib.util.module_from_spec(spec)
spec.loader.exec_module(cache)


class CacheTests(unittest.TestCase):
    def setUp(self):
        self.root = Path(tempfile.mkdtemp(prefix="narration-cache-test-"))
        self.model = self.root / "model"
        (self.model / "speech_tokenizer").mkdir(parents=True)
        for name in ("model.safetensors", "vocab.json", "merges.txt", "tokenizer_config.json", "speech_tokenizer/model.safetensors"):
            (self.model / name).write_text("synthetic asset " + name)
        self.voice = self.root / "voice.spk"
        self.voice.write_bytes(b"synthetic voice")
        self.script = self.root / "script.tsv"
        self.script.write_text("intro\tA complete thought.\n")
        self.lexicon = self.root / "lexicon.json"
        self.lexicon.write_text("{}")
        self.planner = self.root / "planner.py"
        self.planner.write_text(
            "import json,sys\nfrom pathlib import Path\n"
            "work=Path(sys.argv[3]);work.mkdir(parents=True,exist_ok=True)\n"
            "(work/'plan.json').write_text(json.dumps({'intro':[{'j':0,'text':'A complete thought.',"
            "'reads':[{'tag':'intro__s0__12345678','text':'A complete thought.','tts':'A complete thought.'}]}]}))\n")
        self.ftts = self.root / "ftts"
        self.ftts.write_text(f"#!{sys.executable}\n" + r'''
import json,sys,time,wave,unicodedata,hashlib
from pathlib import Path
args=sys.argv[1:]
root=Path(__file__).parent
def arg(name,default):
    return args[args.index(name)+1] if name in args else default
if '--check' in args:
    print(json.dumps({'event':'text_prepared','unicode_version':unicodedata.unidata_version}))
    print(json.dumps({'event':'check_complete','model':arg('--model',str(root/'model')),
      'voice':arg('--voice',str(root/'voice.spk')), 'profile':'balanced','packet_frames':'4',
      'math_mode':'fast','normalize':'verbatim'}))
    sys.exit(0)
with (root/'calls').open('a') as calls: calls.write('synthesis\n')
mode=(root/'mode').read_text() if (root/'mode').exists() else 'ok'
if mode=='slow': time.sleep(.3)
if mode=='hang': time.sleep(10)
if mode=='mutate_model': (root/'model'/'vocab.json').write_text('changed while producer ran')
if mode=='mutate_script': (root/'script.tsv').write_text('intro\tChanged during generation.\n')
out=Path(args[-1])
if mode=='partial':
    out.write_bytes(b'RIFFbroken');sys.exit(6)
with wave.open(str(out),'wb') as w:
    w.setnchannels(1);w.setsampwidth(2);w.setframerate(24000);w.writeframes(b'\x01\x00'*2400)
if mode=='error':
    print(json.dumps({'event':'run_error','kind':'cancelled'}));sys.exit(6)
if mode=='no_event': sys.exit(0)
trace=Path(arg('--trace',str(root/'unused-trace')))
trace.mkdir(parents=True,exist_ok=True)
sha=lambda path: hashlib.sha256(Path(path).read_bytes()).hexdigest()
model=Path(arg('--model',str(root/'model')))
generation={'schema':'ftts.generation','schema_version':1,'run_id':'fixture',
 'requested':{'seed':int(arg('--seed','0'))},
 'state':'completed','execution':'inline','producer':{'executable_sha256':sha(__file__)},
 'effective':{'seed':int(arg('--seed','0')),'normalization':{'mode':'verbatim'},
              'utterance_start':'fresh','sampling':{'mode':'production'}},
 'voice':{'route':'xvector'},'output':{'stop_reason':'end_of_speech','frames':2,'frame_limit':64,'samples':2400,'sample_rate':24000},
 'assets':{'voice_source_sha256':sha(arg('--voice',str(root/'voice.spk'))),
           'talker_sha256':sha(model/'model.safetensors'),
           'codec_sha256':sha(model/'speech_tokenizer/model.safetensors')}}
for name in ('vocab.json','merges.txt','tokenizer_config.json'):
    generation['assets'][name]={'sha256':sha(model/name)}
if mode=='cap': generation['output']['stop_reason']='frame_cap_reached';generation['state']='truncated'
if mode=='wrong_seed': generation['effective']['seed']+=1
if mode=='wrong_voice': generation['assets']['voice_source_sha256']='wrong'
if mode=='stale_trace': generation['run_id']='old-run'
if mode!='missing_trace': (trace/'fixture.generation.json').write_text(json.dumps(generation))
print(json.dumps({'event':'run_complete','run_id':'fixture','exit_code':0,'audio_bytes':4800,'frames':2,'samples':2400}))
''')
        self.ftts.chmod(0o755)
        self.env = cache.child_environment(os.environ)
        self.options = {"profile": "balanced", "route": "in-process", "normalization": "verbatim", "voice_cache": False}
        self.context = self.capture()
        self.plan = {"intro": [{"j": 0, "text": "A complete thought.", "reads": [
            {"tag": "intro__s0__12345678", "text": "A complete thought.", "tts": "A complete thought."}]}]}
        self.takes = self.root / "takes"

    def capture(self):
        return cache.make_context(str(self.ftts), str(self.voice), str(self.model), self.planner,
                                  self.script, self.lexicon, self.env, self.options)

    def job(self, context=None):
        return cache.bind_plan(copy.deepcopy(self.plan), context or self.context)[0]

    def run_job(self, job=None, timeout=5, verify=None):
        return cache.run_batch([job or self.job()], self.takes, self.env, 1, timeout,
                               verify or (lambda: None))[0]

    def calls(self):
        return (self.root / "calls").read_text().count("synthesis")

    def test_same_inputs_hit_without_reinvoking_producer(self):
        self.assertEqual(self.run_job()["state"], "generated")
        self.assertEqual(self.run_job()["state"], "hit")
        self.assertEqual(self.calls(), 1)

    def test_asset_content_mutation_matrix_and_same_mtime(self):
        original = self.job()["key"]
        for path in (self.ftts, self.voice, self.model / "model.safetensors", self.model / "vocab.json",
                     self.model / "speech_tokenizer/model.safetensors", self.lexicon, self.planner):
            with self.subTest(path=path.name):
                before, stamp = path.read_bytes(), path.stat()
                path.write_bytes(before + b"\n# changed same filename")
                os.utime(path, ns=(stamp.st_atime_ns, stamp.st_mtime_ns))
                self.assertNotEqual(self.job(self.capture())["key"], original)
                path.write_bytes(before)
        (self.model / "new-optional-config.json").write_text("{}"); self.assertNotEqual(self.job(self.capture())["key"], original)

    def test_unrelated_script_edit_keeps_unchanged_take_key(self):
        original = self.job()["key"]
        self.script.write_text(self.script.read_text() + "next\tAn unrelated thought.\n")
        self.assertEqual(self.job(self.capture())["key"], original)

    def test_seed_text_normalization_options_route_and_environment_keyed(self):
        original = self.job()["key"]
        for key, value in (("seed", 7), ("text", "Changed raw."), ("tts", "Changed spoken.")):
            plan = copy.deepcopy(self.plan)
            plan["intro"][0]["reads"][0][key] = value
            self.assertNotEqual(cache.bind_plan(plan, self.context)[0]["key"], original)
        for section, key, value in (("settings", "profile", "strict"), ("settings", "normalize", "conservative"),
                                    ("settings", "packet_frames", "2"), ("settings", "math_mode", "strict"),
                                    ("options", "route", "another-route"), ("platform", "machine", "another-isa")):
            context = copy.deepcopy(self.context); context[section][key] = value
            self.assertNotEqual(self.job(context)["key"], original)
        self.env["FTTS_INT8"] = "0"
        self.assertNotEqual(self.job(self.capture())["key"], original)

    def test_legacy_wav_retained_and_never_reused(self):
        self.takes.mkdir()
        legacy = self.takes / "intro__s0__12345678.wav"
        legacy.write_bytes(b"old legacy audio")
        self.assertEqual(self.run_job()["state"], "generated")
        self.assertEqual(legacy.read_bytes(), b"old legacy audio")

    def test_nfc_input_is_recorded_without_changing_submitted_text(self):
        self.plan["intro"][0]["reads"][0]["tts"] = "Cafe\u0301"
        request = self.job()["request"]
        self.assertEqual(request["submitted_text"], "Cafe\u0301")
        self.assertEqual(request["normalized_text"], "Café")
        context = copy.deepcopy(self.context)
        context["normalization"]["engine_unicode"] = "unknown"
        with self.assertRaisesRegex(ValueError, "Unicode versions"):
            self.job(context)
        context["normalization"]["engine_unicode"] = "99.0.0"
        self.assertEqual(self.job(context)["request"]["normalized_text"], "Café")
        self.plan["intro"][0]["reads"][0]["tts"] = "\U0010ffff"
        with self.assertRaisesRegex(ValueError, "Unicode versions"):
            self.job(context)

    def test_corrupt_wav_is_regenerated_with_bad_bytes_retained(self):
        self.run_job(); job = self.job()
        wav = self.takes / f"{job['tag']}.wav"
        wav.write_bytes(b"corrupt")
        result = self.run_job()
        self.assertEqual(result["state"], "generated")
        self.assertEqual((result["attempt"] / "previous.wav").read_bytes(), b"corrupt")
        self.assertTrue(cache.valid_hit(job, self.takes))

    def test_receipt_free_or_partial_or_wrong_voice_receipt_not_a_hit(self):
        self.run_job(); job = self.job()
        path = self.takes / f"{job['tag']}.receipt.json"
        receipt = json.loads(path.read_text())
        path.rename(self.root / "retained-receipt.json")
        self.assertFalse(cache.valid_hit(job, self.takes))
        path.write_text("{"); self.assertFalse(cache.valid_hit(job, self.takes))
        receipt["request"]["context"]["voice"]["input"]["sha256"] = "wrong voice"
        path.write_text(json.dumps(receipt)); self.assertFalse(cache.valid_hit(job, self.takes))

    def test_truncated_payload_does_not_pass_even_with_matching_digest(self):
        self.run_job(); job = self.job()
        path = self.takes / f"{job['tag']}.wav"
        path.write_bytes(path.read_bytes()[:-50])
        receipt_path = self.takes / f"{job['tag']}.receipt.json"
        receipt = json.loads(receipt_path.read_text()); receipt["wav"]["sha256"] = cache.digest(path)
        receipt_path.write_text(json.dumps(receipt))
        self.assertFalse(cache.valid_hit(job, self.takes))

    def test_cancelled_partial_missing_event_and_timeout_not_published(self):
        for mode in ("partial", "error", "no_event", "hang"):
            with self.subTest(mode=mode):
                (self.root / "mode").write_text(mode)
                result = self.run_job(timeout=.1 if mode == "hang" else 5)
                self.assertEqual(result["state"], "failed")
                self.assertTrue((result["attempt"] / "failure.json").exists())
                self.assertFalse(cache.valid_hit(self.job(), self.takes))

    def test_exit_zero_cannot_hide_capped_stale_wrong_voice_or_wrong_seed_evidence(self):
        for mode in ("cap", "wrong_seed", "wrong_voice", "stale_trace", "missing_trace"):
            with self.subTest(mode=mode):
                (self.root / "mode").write_text(mode)
                result = self.run_job()
                self.assertEqual(result["state"], "failed")
                self.assertTrue((result["attempt"] / "audio.wav").exists())
                self.assertFalse(cache.valid_hit(self.job(), self.takes))

    def test_changed_inputs_before_publication_rejected(self):
        def refuse():
            raise ValueError("changed model during generation")
        with self.assertRaisesRegex(ValueError, "changed model"):
            self.run_job(verify=refuse)
        self.assertFalse(cache.valid_hit(self.job(), self.takes))
        self.assertEqual(len(list((self.takes / "attempts").glob("*/audio.wav"))), 1)

    def test_seeded_plan_and_old_director_pick_fail_closed(self):
        self.plan["intro"][0]["reads"].append({"tag": "intro__s0__87654321", "text": "A complete thought.", "tts": "A complete thought.", "seed": 1})
        jobs = cache.bind_plan(self.plan, self.context)
        self.assertEqual([j["request"]["seed"] for j in jobs], [0, 1])
        self.assertNotEqual(jobs[0]["key"], jobs[1]["key"])
        picks = self.root / "picks.tsv"
        picks.write_text("intro\t0\tintro__s0__12345678\told approval\n")
        with self.assertRaisesRegex(ValueError, "old approval"):
            cache.check_picks(self.plan, picks)
        picks.write_text(f"intro\t0\t{jobs[1]['tag']}\tnew review\n")
        cache.check_picks(self.plan, picks)

    def test_full_cli_two_concurrent_requests_produce_once(self):
        (self.root / "mode").write_text("slow")
        cmd = [sys.executable, "-I", str(Path(cache.__file__)), "--planner", str(self.planner),
               "--script", str(self.script), "--work", str(self.root / "work"), "--lexicon", str(self.lexicon),
               "--voice", str(self.voice), "--model", str(self.model), "--ftts", str(self.ftts), "--workers", "1"]
        processes = [subprocess.Popen(cmd, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True) for _ in range(2)]
        try:
            for process in processes:
                stdout, stderr = process.communicate(timeout=20)
                self.assertEqual(process.returncode, 0, (stdout, stderr))
        finally:
            for process in processes:
                if process.poll() is None:
                    process.kill()
                process.communicate()
        self.assertEqual(self.calls(), 1)
        plan = json.loads((self.root / "work" / "plan.json").read_text())
        self.assertIn("__v2_", plan["intro"][0]["reads"][0]["tag"])

    def test_full_cli_rehash_detects_mid_run_model_and_script_changes(self):
        for mode in ("mutate_model", "mutate_script"):
            with self.subTest(mode=mode):
                (self.root / "mode").write_text(mode)
                work = self.root / mode
                cmd = [sys.executable, "-I", str(Path(cache.__file__)), "--planner", str(self.planner),
                       "--script", str(self.script), "--work", str(work), "--lexicon", str(self.lexicon),
                       "--voice", str(self.voice), "--model", str(self.model), "--ftts", str(self.ftts), "--workers", "1"]
                result = subprocess.run(cmd, capture_output=True, text=True, timeout=20)
                self.assertEqual(result.returncode, 1)
                self.assertIn("inputs changed during batch", result.stderr)
                self.assertEqual(list((work / "takes").glob("*.receipt.json")), [])
                self.assertEqual(len(list((work / "takes" / "attempts").glob("*/audio.wav"))), 1)


if __name__ == "__main__":
    unittest.main()
