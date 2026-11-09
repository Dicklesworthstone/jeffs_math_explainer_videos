"""Watch only authored files beside installed runtimes; no rendering doubles."""
from pathlib import Path
import sys
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / 'python'))
from fmn_python import effect_audit, project_watch
from fmn_python.scene_loading import SceneSource


class RuntimeProjectWatch(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.scene = self.write('scene.py', 'pass\n')
        self.helper = self.write('helper.py', 'VALUE=1\n')

    def write(self, name, text):
        path = self.root/name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text)
        return path

    def snapshot(self, *extras):
        source = SceneSource(self.scene, object)
        return dict(project_watch._snapshot(SimpleNamespace(_source=source), extras))

    def test_visible_runtime_subtree_is_pruned_before_watch_budgets(self):
        for i in range(8):
            self.write(f'environment/lib/python/site-packages/pkg{i}/module.py', 'pass\n')
        with patch.object(sys, 'prefix', str(self.root/'environment')), \
             patch.object(project_watch, '_MAX_FILES', 3):
            self.assertEqual(set(self.snapshot()), {self.scene, self.helper})

    def test_site_and_dist_components_are_pruned_without_active_prefix(self):
        self.write('libraries/site-packages/a.py', 'pass\n')
        self.write('libraries/dist-packages/b.py', 'pass\n')
        self.assertEqual(set(self.snapshot()), {self.scene, self.helper})

    def test_inactive_environment_marker_excludes_its_scripts(self):
        self.write('other_env/pyvenv.cfg', 'include-system-site-packages=false\n')
        self.write('other_env/bin/tool.py', 'pass\n')
        self.assertEqual(set(self.snapshot()), {self.scene, self.helper})

    def test_explicit_runtime_input_and_source_symlink_are_ignored(self):
        installed = self.write('env/site-packages/a.py', 'pass\n')
        alias = self.root/'alias.py'
        alias.symlink_to(installed)
        self.assertEqual(set(self.snapshot(installed, alias)), {self.scene, self.helper})

    def test_hidden_project_asset_is_still_watched_when_explicit(self):
        asset = self.write('.assets/model.csv', '1,2,3\n')
        self.assertEqual(set(self.snapshot(asset)), {self.scene, self.helper, asset})
        before = self.snapshot(asset)
        asset.write_text('3,2,1\n')
        self.assertNotEqual(self.snapshot(asset), before)

    def test_runtime_changes_do_not_trigger_but_local_changes_do(self):
        installed = self.write('env/site-packages/a.py', 'VERSION=1\n')
        before = self.snapshot()
        installed.write_text('VERSION=2\n')
        self.assertEqual(self.snapshot(), before)
        self.helper.write_text('VALUE=2\n')
        self.assertNotEqual(self.snapshot(), before)

    def test_narrow_effect_exemptions_do_not_include_entire_runtime_prefix(self):
        # Runtime-owned executables/configuration need not be C6-exempt assets.
        # Factoring the root helper must not broaden the effect-audit policy.
        with patch.object(sys, 'prefix', str(self.root/'env')):
            self.assertNotIn(str(self.root/'env')+'/', effect_audit._library_roots())
        with patch.object(effect_audit, '_roots', effect_audit._library_roots()):
            self.assertFalse(effect_audit._library(str(self.helper)))


    def test_real_effect_audit_still_records_authored_reads_and_detects_edits(self):
        import hashlib
        asset = self.write('data.csv', '1,2,3\n')
        recorder = effect_audit.begin()
        try:
            contents = asset.read_bytes()
        finally:
            recorder.stop()
        digest = hashlib.sha256(contents).hexdigest()
        self.assertEqual(recorder.inputs(RuntimeError), [(f'read/{digest}/data.csv', digest)])
        asset.write_text('3,2,1\n')
        with self.assertRaisesRegex(RuntimeError, 'changed during rendering'):
            recorder.inputs(RuntimeError)


if __name__ == '__main__':
    unittest.main()
