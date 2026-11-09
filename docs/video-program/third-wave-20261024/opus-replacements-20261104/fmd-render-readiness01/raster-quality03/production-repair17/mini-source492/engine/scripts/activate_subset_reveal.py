#!/usr/bin/env python3
"""Apply narrow, idempotent subset-reveal integration to the current source.

Large pre-existing bootstrap/bridge files are not replaced wholesale. The
activation workflow checks the resulting production code and commits only these
five integration paths, preserving other agents' changes on main.
"""
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def replace_once(path, before, after):
    text = path.read_text()
    if after in text:
        return
    if text.count(before) != 1:
        raise RuntimeError(f"subset integration anchor changed: {path}")
    path.write_text(text.replace(before, after, 1))


replace_once(
    ROOT / "crates/fmn-python/python/fmn_python/initialization.py",
    '    ("text_reveal", "install_text_reveal"),\n',
    '    ("text_reveal", "install_text_reveal"),\n'
    '    ("subset_reveal", "install_subset_reveal"),\n',
)

bridge = ROOT / "crates/fmn-python/tests/bridge.py"
text = bridge.read_text()
start = "# int_func routes as data: the two native rounding rules pass, anything\n"
end = "# fm-5wq.4.65: isolate= and tex_to_color_map= ride the native span map"
marker = "# Authored subset selectors run through the production callback boundary."
if marker not in text:
    if text.count(start) != 1 or text.count(end) != 1:
        raise RuntimeError("subset bridge witness anchors changed")
    lower, upper = text.index(start), text.index(end)
    if upper <= lower:
        raise RuntimeError("subset bridge witness anchors are out of order")
    replacement = '''# Authored subset selectors run through the production callback boundary.
assert (
    creation_animation.ShowIncreasingSubsets(
        manimlib.VGroup(geometry.Rectangle(width=0.5, height=0.5)),
        int_func=np.ceil,
    )._native_params()["int_round"]
    == "ceil"
)
custom_members = [geometry.Rectangle(width=0.5, height=0.5) for _ in range(3)]
custom_group = manimlib.VGroup(*custom_members)
custom_selector = creation_animation.ShowIncreasingSubsets(
    custom_group, int_func=lambda value: int(value), rate_func=manimlib.linear,
)
custom_selector.interpolate(.5)
assert list(custom_group.submobjects) == custom_members[:1]
custom_selector.int_func = lambda value: 2
custom_selector.interpolate(.5)
assert list(custom_group.submobjects) == custom_members[:2]
assert custom_selector._native_params() == {}
for invalid in ("round", 7):
    try:
        creation_animation.ShowIncreasingSubsets(custom_group, int_func=invalid)
    except TypeError as error:
        assert "int_func must be callable" in str(error)
    else:
        raise AssertionError("ShowIncreasingSubsets accepted a non-callable selector")
try:
    creation_animation.ShowIncreasingSubsets("not a mobject")
except TypeError as error:
    assert "requires a Mobject family" in str(error)
else:
    raise AssertionError("ShowIncreasingSubsets accepted a non-Mobject")
for empty_cls in (creation_animation.ShowIncreasingSubsets,
                  creation_animation.ShowSubmobjectsOneByOne):
    empty_group = manimlib.VGroup()
    empty_reveal = empty_cls(empty_group)
    empty_reveal.begin()
    empty_reveal.finish()
    assert list(empty_group.submobjects) == []

'''
    bridge.write_text(text[:lower] + replacement + text[upper:])

replace_once(
    ROOT / "scripts/check_portal_runtime.sh",
    "creation_semantics text_reveal ",
    "creation_semantics subset_reveal text_reveal ",
)

lib = ROOT / "crates/fmn-python/src/lib.rs"
text = lib.read_text()
if "mod subset_reveal_acceptance {" not in text:
    lib.write_text(text + '''

#[cfg(test)]
mod subset_reveal_acceptance {
    #[test]
    fn production_subset_reveal_acceptance() {
        crate::with_python_test_module("native subset reveals", |py, _module, globals| {
            let source = std::ffi::CString::new(include_str!("../tests/subset_reveal.py"))
                .expect("subset reveal source contains no NUL");
            py.run(source.as_c_str(), Some(globals), Some(globals))
                .inspect_err(|error| error.print(py))
                .expect("real native subset selection, lifecycle and output");
        });
    }
}
''')
# deepcopy may visit an Animation's child aliases before its mobject root.
# Marionette allocates a family of shells, but a previously memoized child
# remains the one authoritative Python copy. Never replace that memo entry or
# reinitialize an already copied descendant while filling the new root shell.
replace_once(
    ROOT / "crates/fmn-python/python/manimlib_bootstrap.py",
    "    mapping = {old: new for old, new in pairs}\n"
    "    for old, new in pairs:\n",
    "    mapping = {old: memo.get(id(old), new) for old, new in pairs}\n"
    "    pairs = [(old, new) for old, new in pairs if id(old) not in memo]\n"
    "    for old, new in pairs:\n",
)

print("subset reveal production initialization and native/wheel witnesses integrated")
