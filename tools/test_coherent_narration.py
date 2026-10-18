"""Adapter invariants on synthetic PCM; these tests make no acoustic claims."""
import copy
import importlib.util
import json
from pathlib import Path
import struct
import tempfile
import types
import unittest
import wave

spec = importlib.util.spec_from_file_location("coherent", Path(__file__).with_name("coherent_narration.py"))
m = importlib.util.module_from_spec(spec)
spec.loader.exec_module(m)


class CoherentTests(unittest.TestCase):
    def setUp(self):
        # Retain test artifacts, consistent with this repository's no-deletion rule.
        self.root = Path(tempfile.mkdtemp(prefix="coherent-narration-test-"))
        self.script = self.root / "script.tsv"
        self.script.write_text("01_1\tFour pulses. They return.\n01_2\tDo not drop the negative.\n")
        self.units = self.root / "units.json"
        self.plan = {"schema_version": 1, "script_sha256": m.cache.digest(self.script), "units": [
            {"id": "01u1", "line_ids": ["01_1", "01_2"],
             "text": "Four pulses. They return. Do not drop the negative."}]}
        self.save_plan()

    def save_plan(self):
        self.units.write_text(json.dumps(self.plan))

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


if __name__ == "__main__":
    unittest.main()
