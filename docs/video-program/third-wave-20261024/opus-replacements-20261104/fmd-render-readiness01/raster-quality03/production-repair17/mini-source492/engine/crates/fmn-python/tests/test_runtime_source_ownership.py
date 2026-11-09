"""Real import-cache, namespace and filesystem regressions for fm-5wq.25.

The Scene class is just a selector; these tests make no native-render claim.
"""
from __future__ import annotations

import hashlib
import importlib
import os
from pathlib import Path
import sys
import tempfile
from types import ModuleType
import unittest
from unittest.mock import patch

sys.path.insert(0, str(Path(__file__).resolve().parents[1] / "python"))
from fmn_python import effect_audit, runtime_paths
from fmn_python.scene_loading import SceneSource, _local_namespace


class Scene:
    pass


class RuntimeSourceOwnership(unittest.TestCase):
    def setUp(self):
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name).resolve()
        self.site = self.root / '.venv/lib/python3.13/site-packages'
        self.site.mkdir(parents=True)
        self.path = self.write('scene.py', 'VALUE = 1\n')
        paths = patch.object(sys, 'path', [str(self.site), *sys.path])
        paths.start()
        self.addCleanup(paths.stop)
        modules = patch.dict(sys.modules)
        modules.start()
        self.addCleanup(modules.stop)
        self.prior_path, self.prior_meta = list(sys.path), list(sys.meta_path)

    def write(self, relative, text):
        path = self.root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(text, encoding='utf-8')
        return path

    def install(self, relative, text):
        return self.write(self.site.relative_to(self.root) / relative, text)

    def test_preimported_installation_retains_module_and_extension_identities(self):
        self.install('fmn_test_distribution/__init__.py', 'TOKEN = object()\n')
        module = importlib.import_module('fmn_test_distribution')
        extension = ModuleType('fmn_test_distribution._core')
        extension.__file__ = str(self.site / 'fmn_test_distribution/_core.so')
        sys.modules[extension.__name__] = extension
        self.path.write_text('import fmn_test_distribution\n')
        with SceneSource(self.path, Scene) as source:
            self.assertIs(source.module.fmn_test_distribution, module)
            self.assertIs(sys.modules[extension.__name__], extension)
            self.assertFalse(source._owns_source(Path(module.__file__)))
            self.assertEqual(set(source.sources), {'scene.py'})
        self.assertIs(sys.modules[module.__name__], module)
        self.assertEqual(sys.path, self.prior_path)
        self.assertEqual(sys.meta_path, self.prior_meta)

    def test_late_installed_import_survives_exit_without_fresh_loader(self):
        self.install('fmn_test_late.py', 'VALUE = 7\n')
        self.path.write_text('def run():\n import fmn_test_late\n return fmn_test_late\n')
        with SceneSource(self.path, Scene) as source:
            module = source.module.run()
            self.assertNotIn(module.__name__, source._loaded)
            self.assertEqual(type(module.__loader__).__name__, 'SourceFileLoader')
        self.assertIs(sys.modules[module.__name__], module)

    def test_local_same_size_edit_stays_fresh_beside_installed_package(self):
        self.install('fmn_test_runtime.py', 'TOKEN = object()\n')
        runtime = importlib.import_module('fmn_test_runtime')
        helper = self.write('fmn_test_helper.py', 'VALUE = 11\n')
        before = helper.stat().st_mtime_ns
        self.path.write_text('import fmn_test_helper, fmn_test_runtime\nVALUE = fmn_test_helper.VALUE\n')
        for value in (11, 22):
            helper.write_text(f'VALUE = {value}\n')
            os.utime(helper, ns=(before, before))
            with SceneSource(self.path, Scene) as source:
                self.assertEqual(source.module.VALUE, value)
                self.assertIs(source.module.fmn_test_runtime, runtime)
                self.assertEqual(set(source.sources), {'scene.py', 'fmn_test_helper.py'})

    def test_declared_runtime_sources_do_not_become_reloadable(self):
        installed = self.install('fmn_test_declared/__init__.py', 'TOKEN = object()\n')
        self.path.write_text('import fmn_test_declared\n')
        declarations = {path: hashlib.sha256(path.read_bytes()).hexdigest()
                        for path in (installed, self.path)}
        runtime = importlib.import_module('fmn_test_declared')
        with SceneSource(self.path, Scene, source_inputs=declarations) as source:
            self.assertIs(source.module.fmn_test_declared, runtime)
            self.assertNotIn(installed, source.source_digests)
            self.assertEqual(set(source.sources), {'scene.py'})

    def test_installed_namespace_and_new_children_are_not_refreshed(self):
        self.install('fmn_test_namespace/a.py', 'TOKEN = object()\n')
        self.install('fmn_test_namespace/b.py', 'VALUE = 9\n')
        old = importlib.import_module('fmn_test_namespace.a')
        namespace = sys.modules['fmn_test_namespace']
        self.path.write_text('from fmn_test_namespace import a, b\n')
        with SceneSource(self.path, Scene) as source:
            self.assertIs(source.module.a, old)
            self.assertIs(sys.modules['fmn_test_namespace'], namespace)
            self.assertEqual(set(source.sources), {'scene.py'})
            self.assertFalse(_local_namespace('fmn_test_namespace', (self.site/'fmn_test_namespace',), self.root))
        self.assertIs(sys.modules['fmn_test_namespace.b'], namespace.b)

    def test_mixed_namespace_refuses_instead_of_owning_runtime_location(self):
        self.install('fmn_test_mixed/runtime.py', 'VALUE = 1\n')
        self.write('fmn_test_mixed/local.py', 'VALUE = 2\n')
        self.path.write_text('from fmn_test_mixed import local\n')
        with self.assertRaisesRegex(ImportError, 'locations outside'):
            with SceneSource(self.path, Scene):
                self.fail('mixed namespace should not be refreshed')
        self.assertEqual(sys.path, self.prior_path)
        self.assertEqual(sys.meta_path, self.prior_meta)

    def test_component_exclusion_is_not_a_substring_exclusion(self):
        source = SceneSource(self.path, Scene)
        for directory in ('site-packages', 'dist-packages'):
            self.assertFalse(source._owns_source(self.root/directory/'x.py'))
            self.assertTrue(source._owns_source(self.root/(directory+'-examples')/'x.py'))

    def test_sysconfig_and_site_roots_include_nonstandard_install_locations(self):
        for accessor in ('sysconfig', 'site', 'user'):
            with self.subTest(accessor=accessor):
                directory = self.root / ('custom_'+accessor)
                options = dict(stdlib='', platstdlib='', purelib='', platlib='')
                options['purelib'] = str(directory) if accessor == 'sysconfig' else ''
                with patch.object(runtime_paths.sysconfig, 'get_paths', return_value=options), \
                     patch.object(runtime_paths.site, 'getsitepackages', return_value=[str(directory)] if accessor == 'site' else []), \
                     patch.object(runtime_paths.site, 'getusersitepackages', return_value=str(directory) if accessor == 'user' else ''):
                    source = SceneSource(self.path, Scene)
                    self.assertFalse(source._owns_source(directory/'module.py'))
                    self.assertTrue(source._owns_source(self.root/'local/module.py'))
                    self.assertIn(str(directory)+os.sep, effect_audit._library_roots())

    def test_nested_runtime_prefix_is_excluded_but_ancestor_prefix_is_not(self):
        for key in ('prefix', 'base_prefix', 'exec_prefix', 'base_exec_prefix'):
            with self.subTest(key=key), patch.object(sys, key, str(self.root/'env')):
                source = SceneSource(self.path, Scene)
                self.assertFalse(source._owns_source(self.root/'env/arbitrary/library.py'))
                self.assertTrue(source._owns_source(self.root/'environment/helper.py'))
        with patch.object(sys, 'prefix', str(self.root.parent)):
            self.assertTrue(SceneSource(self.path, Scene)._owns_source(self.path))

    def test_symlinked_install_root_protects_real_target(self):
        external = self.root/'vendor_runtime'
        external.mkdir()
        alias = self.root/'alias/site-packages'
        alias.parent.mkdir()
        alias.symlink_to(external, target_is_directory=True)
        with patch.object(sys, 'path', [str(alias), *sys.path]):
            source = SceneSource(self.path, Scene)
            self.assertFalse(source._owns_source(external/'library.py'))
            self.assertFalse(source._owns_source(alias/'library.py'))

    def test_symlinked_local_helper_into_runtime_is_not_owned(self):
        installed = self.install('fmn_test_target.py', 'VALUE = 7\n')
        alias = self.root/'fmn_test_alias.py'
        alias.symlink_to(installed)
        self.path.write_text('import fmn_test_alias\n')
        with SceneSource(self.path, Scene) as source:
            self.assertEqual(source.module.fmn_test_alias.VALUE, 7)
            self.assertEqual(set(source.sources), {'scene.py'})

    def test_source_file_inside_installation_is_refused_before_execution(self):
        path = self.install('example.py', "raise AssertionError('must not execute')\n")
        with self.assertRaisesRegex(ImportError, 'runtime installation'):
            SceneSource(path, Scene)

    def test_reload_preserves_runtime_identity_and_refreshes_only_local_sources(self):
        from fmn_python.source_reload import reload_source
        self.install('fmn_test_reload_runtime.py', 'TOKEN = object()\n')
        helper = self.write('fmn_test_reload_local.py', 'VALUE = 11\n')
        self.path.write_text('import fmn_test_reload_runtime\nfrom fmn_test_reload_local import VALUE\n')
        with SceneSource(self.path, Scene) as source:
            runtime = source.module.fmn_test_reload_runtime
            helper.write_text('VALUE = 22\n')
            reload_source(source)
            self.assertEqual(source.module.VALUE, 22)
            self.assertIs(source.module.fmn_test_reload_runtime, runtime)
            helper.write_text('raise RuntimeError("bad helper")\n')
            old = source.module
            with self.assertRaisesRegex(RuntimeError, 'bad helper'):
                reload_source(source)
            self.assertIs(source.module, old)
            self.assertIs(sys.modules[runtime.__name__], runtime)


if __name__ == '__main__':
    unittest.main()
