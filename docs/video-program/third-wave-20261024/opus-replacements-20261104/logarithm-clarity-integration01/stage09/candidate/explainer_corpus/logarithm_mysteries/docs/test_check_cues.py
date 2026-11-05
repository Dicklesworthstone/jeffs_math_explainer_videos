#!/usr/bin/env python3
"""Regression cases for docs/check_cues.py, on tiny fixture crates.

    python3 -I -B docs/test_check_cues.py EMPTY_SCRATCH_DIR

Each case builds a crate with one change and runs a copy of the checker on it,
from outside, the way a person would. A refusing case must print exactly the
reasons it lists: each listed reason matches one failure line, and no failure
line goes unlisted, so a case can't pass for the wrong reason. Nothing outside
EMPTY_SCRATCH_DIR is written. Exit status 1 if any case is wrong.
"""

import shutil
import subprocess
import sys
from pathlib import Path

CHECKER = Path(__file__).resolve().parent / "check_cues.py"

ALPHA, BETA, DRAW, MOD = (f"src/chapters/{n}.rs" for n in ("alpha", "beta", "draw", "mod"))
SCRIPT = "script.tsv"

BASE = {
    SCRIPT: (
        "# cue\ttext\n"
        "00_1\tOne sentence here. A second here. A third here.\n"
        "00_2\tAsk a question? Then answer it.\n"
        "01_1\tFirst line. Second line.\n"
        "01_2\tOnly one sentence.\n"
    ),
    MOD: """//! The registry.
mod alpha;
mod beta;
mod draw;
pub mod thumbnail;
use explainer_kit::Chapter;

pub const TEXT: &[&str] = &["0123456789"];

pub fn registry() -> Vec<Chapter> {
    vec![
        Chapter {
            key: "00_alpha",
            make: |kit| Box::new(alpha::Alpha { kit }),
            intensity: 0.4,
        },
        Chapter {
            key: "01_beta",
            make: |kit| Box::new(beta::Beta { kit }),
            intensity: 0.5,
        },
    ]
}

#[cfg(test)]
mod tests {
    #[test]
    fn registry_has_chapters() {
        assert_eq!(super::registry().len(), 2);
    }
}
""",
    ALPHA: """impl SceneConstruct for Alpha {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "00_1")?;
        kit.until_sentence(stage, "00_1", 1)?;
        kit.until_sentence(stage, "00_1", 2)?;
        kit.say(stage, "00_2")?;
        kit.until_sentence(stage, "00_2", 1)?;
        kit.finish(stage)
    }
}
""",
    BETA: """impl SceneConstruct for Beta {
    fn construct(&mut self, stage: &mut Stage<'_>) -> fmn::Result<()> {
        let kit = self.kit.clone();
        kit.say(stage, "01_1")?;
        kit.until_sentence(stage, "01_1", 1)?;
        kit.say(stage, "01_2")?;
        kit.finish(stage)
    }
}
""",
    DRAW: "pub fn helper() -> f64 {\n    1.0\n}\n",
    "src/chapters/thumbnail.rs": "pub fn make() {}\n",
}
BASE_FIRST = "4 cues, 8 sentences, 4 said"

# The one reviewed computed index, in a crate of its own.
PRODUCT = "src/chapters/product_rule.rs"
COMPUTED = {
    SCRIPT: "08_4\tOne. Two. Three. Four. Five. Six. Seven?\n",
    MOD: """mod product_rule;
pub fn registry() -> Vec<Chapter> {
    vec![Chapter { key: "08_product_rule", make: |kit| Box::new(product_rule::ProductRule { kit }), intensity: 0.5 }]
}
""",
    PRODUCT: """fn scene(kit: Kit, stage: &mut Stage<'_>) -> fmn::Result<()> {
    kit.say(stage, "08_4")?;
    let steps_src = [
        r"a",
        r"b",
        r"c",
        r"d",
    ];
    let mut steps = Vec::new();
    for (k, src) in steps_src.iter().enumerate() {
        steps.push(kit.tex_left(stage, src, 34.0, k)?);
    }
    for (k, &m) in steps.iter().enumerate() {
        kit.until_sentence(stage, "08_4", k + 1)?;
        fade_in_all(stage, &[m], 0.5)?;
    }
    kit.until_sentence(stage, "08_4", 5)?;
    kit.until_sentence(stage, "08_4", 6)?;
    kit.finish(stage)
}
""",
}
COMPUTED_FIRST = "1 cues, 7 sentences, 1 said"
LOOP = "for (k, &m) in steps.iter().enumerate() {"
HOOK_1 = 'kit.until_sentence(stage, "08_4", k + 1)?;'
TAIL_5 = 'kit.until_sentence(stage, "08_4", 5)?;'
SAY_01_2 = 'kit.say(stage, "01_2")?;'
SAY_00_2 = 'kit.say(stage, "00_2")?;'
UNTIL_00_1_1 = 'kit.until_sentence(stage, "00_1", 1)?;'
UNTIL_00_1_2 = 'kit.until_sentence(stage, "00_1", 2)?;'
TEST_MODULE = """
#[cfg(test)]
mod tests {
    const SRC: &str = include_str!("alpha.rs");
    fn found(s: &str) -> usize {
        SRC.find(s).unwrap()
    }
    #[test]
    fn strings_and_fixtures_are_not_the_scene() {
        let a = found(r#"kit.until_sentence(stage, "00_1", 9)"#);
        let b = found("kit.say(stage, \\"00_2\\")");
        assert!(SRC.contains("#[cfg(test)]"));
        kit.say(stage, "00_1");
        kit.until_sentence(stage, "99_1", 0);
    }
}
"""
REGISTRY_BODY = BASE[MOD][BASE[MOD].index("    vec![") : BASE[MOD].index("#[cfg(test)]") - 3]
ALPHA_ENTRY = BASE[MOD][BASE[MOD].index("        Chapter {\n            key: \"00_alpha\"") : BASE[MOD].index("        Chapter {\n            key: \"01_beta\"")]
BETA_ENTRY = BASE[MOD][BASE[MOD].index("        Chapter {\n            key: \"01_beta\"") : BASE[MOD].index("    ]\n}")]


def sub(path, old, new):
    def apply(files):
        found = files[path].count(old)
        if found != 1:
            raise AssertionError(f"{path}: {old!r} occurs {found} times, so this case would not change what it claims to")
        files[path] = files[path].replace(old, new)

    return apply


def put(path, text):
    def apply(files):
        files[path] = text

    return apply


def add(path, text):
    def apply(files):
        files[path] = files[path] + text

    return apply


def drop(path):
    def apply(files):
        del files[path]

    return apply


CASES = []


def passes(name, *edits, shows=(), base=BASE, first=BASE_FIRST):
    CASES.append(dict(name=name, edits=edits, fails=None, shows=shows, base=base, first=first))


def refuses(name, *edits, fails, base=BASE):
    CASES.append(dict(name=name, edits=edits, fails=fails, shows=(), base=base, first=None))


def before_say(code):
    return sub(BETA, SAY_01_2, f"{code}\n        {SAY_01_2}")


# ---------------------------------------------------- what must not stop the film

passes("the baseline crate", shows=["registered modules read, in registry order: alpha, beta", "draw, mod, thumbnail"])
passes(
    "a preserved retired file, with its says and bad hooks, is ignored and named",
    put("src/chapters/bookends.rs", 'fn old() { kit.say(stage, "00_1"); kit.until_sentence(stage, "00_1", 99); kit.say(stage, "99_9"); }\n'),
    shows=["so not compiled and not read: bookends.rs"],
)
passes("a preserved retired file that would not even lex is ignored", put("src/chapters/winding.rs", 'fn old() { "never closes\n'), shows=["winding.rs"])
passes("a .orig left by a patch run is named", put("src/chapters/alpha.rs.orig", 'kit.say(stage, "00_1")\n'), shows=["alpha.rs.orig"])
passes("test module strings and test fixtures don't count", add(ALPHA, TEST_MODULE))
passes(
    "a cfg(test) helper before the tests module doesn't count",
    add(DRAW, '#[cfg(test)]\npub fn probe(kit: &Kit, stage: &mut Stage<\'_>) {\n    kit.say(stage, "00_1").unwrap();\n}\n'),
)
passes("a #[test] fn outside any cfg(test) module doesn't count", add(BETA, '#[test]\nfn lone() {\n    kit.say(stage, "01_1");\n}\n'))
passes(
    "a test item keeps its attributes and visibility",
    add(BETA, '#[cfg(test)]\n#[allow(dead_code)]\npub(crate) fn probe() {\n    kit.say(stage, "01_1");\n}\n'),
)
passes("a real say after the string \"#[cfg(test)]\" still counts", before_say('let _note = "#[cfg(test)]";'))
passes(
    "char literals and lifetimes open no strings",
    before_say("let _q = '\"'; let _s = '\\''; let _b = b'\"'; let _u = '\\u{2212}'; let _n = '\\\\'; fn f<'a>(x: &'a str) -> &'a str { x }"),
)
passes("comments with quotes open no strings", before_say('// an open " quote\n        /* a /* nested " */ comment " */'))
passes("raw strings with quotes and hashes end where they end", before_say('let _r = r##"a "# b"##; let _b = br#"c " d"#;'))
passes("a string with an escaped quote and a line break", before_say('let _s = "an \\" escaped\n        line";'))
passes(
    "a rustfmt-wrapped hook with a trailing comma is read",
    sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(\n            stage,\n            "00_1",\n            1,\n        )?;'),
)
passes("a local variable named say is not a call", before_say("let say = 1.0;\n        let _early = say < 2.0;"))
passes("a comment that looks like a second say is not a second say", add(ALPHA, '// kit.say(stage, "00_1")\n'))

# ---------------------------------------------------- known gaps, pinned so nobody assumes otherwise
# These pass and shouldn't always. A text scan can't see them; a build and a real timeline can.

passes(
    "KNOWN GAP: a say in a function nothing calls counts as said",
    sub(BETA, SAY_01_2, ""),
    add(BETA, f"fn never_called(kit: Kit, stage: &mut Stage<'_>) {{\n    {SAY_01_2}\n}}\n"),
)
passes("KNOWN GAP: a say in a loop counts once", sub(BETA, SAY_01_2, f"for _ in 0..2 {{\n            {SAY_01_2}\n        }}"))
passes(
    "KNOWN GAP: a say in a macro body counts once, however often it expands",
    sub(BETA, SAY_01_2, f"macro_rules! speak {{\n            () => {{\n                {SAY_01_2}\n            }};\n        }}\n        speak!();\n        speak!();"),
)
passes(
    "KNOWN GAP: until_phrase on a bad cue is counted, not checked",
    before_say('until_phrase(stage, &kit, "99_9", "nothing")?;'),
    shows=["until_phrase 1"],
)
passes("KNOWN GAP: a say outside src/chapters isn't read", put("src/model.rs", 'fn stray() { kit.say(stage, "00_1"); }\n'))

# ---------------------------------------------------- counterfeits of a missing say

MISSING = ["01_2: said 0 times (never)"]
refuses("a comment can't stand in for a say", sub(BETA, SAY_01_2, f"// {SAY_01_2}"), fails=MISSING)
refuses("a block comment can't stand in for a say", sub(BETA, SAY_01_2, f"/* {SAY_01_2} */"), fails=MISSING)
refuses("a string can't stand in for a say", sub(BETA, SAY_01_2, 'let _s = "kit.say(stage, \\"01_2\\")";'), fails=MISSING)
refuses("a raw string can't stand in for a say", sub(BETA, SAY_01_2, 'let _s = r#"kit.say(stage, "01_2")"#;'), fails=MISSING)
refuses(
    "a say in a cfg(test) module can't stand in for the scene's",
    sub(BETA, SAY_01_2, ""),
    add(BETA, '#[cfg(test)]\nmod tests {\n    fn t() {\n        kit.say(stage, "01_2");\n    }\n}\n'),
    fails=MISSING,
)
refuses("a say in a #[test] can't stand in for the scene's", sub(BETA, SAY_01_2, ""), add(BETA, '#[test]\nfn t() {\n    kit.say(stage, "01_2");\n}\n'), fails=MISSING)
refuses(
    "a say in a cfg(test) helper can't stand in for the scene's",
    sub(BETA, SAY_01_2, ""),
    add(DRAW, '#[cfg(test)]\npub fn probe() {\n    kit.say(stage, "01_2");\n}\n'),
    fails=MISSING,
)
refuses("a say in a retired file can't stand in for the scene's", sub(BETA, SAY_01_2, ""), put("src/chapters/bookends.rs", f"{SAY_01_2}\n"), fails=MISSING)

# ---------------------------------------------------- the registry

refuses("mod.rs missing", drop(MOD), fails=["src/chapters/mod.rs missing"])
refuses("an empty registry", sub(MOD, REGISTRY_BODY, "    vec![]\n"), fails=["mod.rs registry is empty"])
refuses("no fn registry", sub(MOD, "pub fn registry()", "pub fn listing()"), fails=["0 fn registry found, not 1"])
refuses("a registry that is built, not listed", sub(MOD, REGISTRY_BODY, "    let mut v = vec![];\n    v.extend(other());\n    v\n"), fails=["fn registry is not a plain vec!"])
refuses("a registry entry that is not a Chapter", sub(MOD, BETA_ENTRY, "        other_chapter(),\n"), fails=["a registry entry that is not Chapter"])
refuses("a chapter key without a number", sub(MOD, 'key: "01_beta"', 'key: "beta"'), fails=['does not start key: "NN_name"'])
refuses(
    "a missing registered file",
    drop(BETA),
    fails=["mod.rs registry names beta, but beta.rs is missing", "01_1: said 0 times", "01_2: said 0 times"],
)
refuses("a registered module with no declaration", sub(MOD, "mod beta;\n", ""), fails=["mod.rs registry names beta, which mod.rs does not declare"])
refuses("a declared module with no file", sub(MOD, "mod draw;\n", "mod draw;\nmod gamma;\n"), fails=["mod.rs declares gamma, but gamma.rs is missing"])
refuses(
    "chapters out of order",
    sub(MOD, ALPHA_ENTRY + BETA_ENTRY, BETA_ENTRY + ALPHA_ENTRY),
    fails=["mod.rs registry out of order: ['01', '00']", "alpha.rs:4: 00_1 said after 01_2"],
)
refuses(
    "a repeated chapter number",
    sub(MOD, 'key: "01_beta"', 'key: "00_beta"'),
    fails=[
        "mod.rs registry repeats a chapter",
        "does not cover cue prefixes",
        "beta.rs:4: 01_1 said in chapter 00_beta",
        "beta.rs:6: 01_2 said in chapter 00_beta",
    ],
)
refuses(
    "a chapter with no cues",
    sub(MOD, BETA_ENTRY, BETA_ENTRY.replace('"01_beta"', '"02_beta"')),
    fails=["does not cover cue prefixes", "beta.rs:4: 01_1 said in chapter 02_beta", "beta.rs:6: 01_2 said in chapter 02_beta"],
)
refuses(
    "one module run by two chapters",
    sub(MOD, BETA_ENTRY, BETA_ENTRY + BETA_ENTRY.replace('"01_beta"', '"02_again"')),
    fails=[
        "does not cover cue prefixes",
        "registry runs beta more than once",
        "beta.rs:4: 01_1 said in chapter 02_again",
        "beta.rs:4: 01_1 said after 01_2",
        "beta.rs:6: 01_2 said in chapter 02_again",
        "01_1: said 2 times",
        "01_2: said 2 times",
    ],
)
refuses(
    "a retired file declared again is compiled, so it is read",
    sub(MOD, "mod draw;\n", "mod draw;\nmod bookends;\n"),
    put("src/chapters/bookends.rs", 'fn old() { kit.say(stage, "00_1"); }\n'),
    fails=["bookends.rs:1: say 00_1 in a module the registry does not run"],
)
refuses("#[path] on a module", sub(MOD, "mod draw;\n", '#[path = "elsewhere.rs"]\nmod draw;\n'), fails=["mod.rs: line 4: #[path]"])
refuses(
    "include! in a chapter",
    add(BETA, 'include!("more.rs");\n'),
    fails=["beta.rs: line 10: include!", "01_1: said 0 times", "01_2: said 0 times"],
)

# ---------------------------------------------------- the script

refuses("script.tsv missing", drop(SCRIPT), fails=["script.tsv missing"])
refuses("a script with only comments", put(SCRIPT, "# nothing\n"), fails=["script.tsv has no cues"])
refuses("a duplicate cue id", add(SCRIPT, "01_2\tSpoken twice.\n"), fails=["duplicate cue ids in script.tsv: 01_2"])
refuses("digits in spoken text", sub(SCRIPT, "Only one sentence.", "Only 1 sentence."), fails=["01_2: digits in spoken text"])
refuses("a row with no tab", add(SCRIPT, "01_3 no tab here\n"), fails=["script.tsv:6: no tab between cue id and text"])
refuses("a bad cue id", add(SCRIPT, "01_3-part2\tSome words.\n"), fails=["script.tsv:6: bad cue id '01_3-part2'"])
refuses("a cue with no text", add(SCRIPT, "01_3\t  \n"), fails=["script.tsv:6: 01_3 has no text"])
refuses("a cue nobody says", add(SCRIPT, "01_3\tSome words.\n"), fails=["01_3: said 0 times (never)"])

# ---------------------------------------------------- the says

refuses("a missing say", sub(BETA, SAY_01_2, ""), fails=MISSING)
refuses("a duplicate say in one module", sub(BETA, SAY_01_2, f"{SAY_01_2}\n        {SAY_01_2}"), fails=["01_2 said after 01_2", "01_2: said 2 times (beta.rs, beta.rs)"])
refuses(
    "a duplicate say across modules",
    sub(ALPHA, "kit.finish(stage)", 'kit.say(stage, "01_2")?;\n        kit.finish(stage)'),
    fails=["alpha.rs:9: 01_2 said in chapter 00_alpha", "beta.rs:4: 01_1 said after 01_2", "01_2: said 2 times (alpha.rs, beta.rs)"],
)
refuses(
    "two says out of order in one module",
    sub(ALPHA, 'kit.say(stage, "00_1")?;', "//"),
    sub(ALPHA, "kit.finish(stage)", 'kit.say(stage, "00_1")?;\n        kit.finish(stage)'),
    fails=[
        "until_sentence(00_1, 1) while None is the current line",
        "until_sentence(00_1, 2) while None is the current line",
        "alpha.rs:9: 00_1 said after 00_2",
    ],
)
refuses("an unknown cue", sub(BETA, SAY_01_2, 'kit.say(stage, "01_9")?;'), fails=["beta.rs:6: unknown cue 01_9", "01_2: said 0 times"])
refuses(
    "a cue said by the wrong chapter",
    sub(BETA, SAY_01_2, ""),
    sub(ALPHA, "kit.finish(stage)", f"{SAY_01_2}\n        kit.finish(stage)"),
    fails=["alpha.rs:9: 01_2 said in chapter 00_alpha", "beta.rs:4: 01_1 said after 01_2"],
)
refuses(
    "a real say in a module named tests but not marked cfg(test) counts",
    add(ALPHA, 'mod tests {\n    fn t() {\n        kit.say(stage, "00_1");\n    }\n}\n'),
    fails=["alpha.rs:14: 00_1 said after 00_2", "00_1: said 2 times (alpha.rs, alpha.rs)"],
)
refuses("a say in a helper module", add(DRAW, 'pub fn extra(kit: &Kit) {\n    kit.say(stage, "00_1");\n}\n'), fails=["draw.rs:5: say 00_1 in a module the registry does not run"])
refuses(
    "an until_sentence in a helper module",
    add(DRAW, 'pub fn extra(kit: &Kit) {\n    kit.until_sentence(stage, "00_1", 1);\n}\n'),
    fails=["draw.rs:5: until 00_1 in a module the registry does not run"],
)

# ---------------------------------------------------- the until_sentence calls

refuses("index zero", sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(stage, "00_1", 0)?;'), fails=["alpha.rs:5: until_sentence(00_1, 0) but 00_1 has 3 sentences"])
refuses(
    "index equal to the sentence count",
    sub(ALPHA, UNTIL_00_1_2, 'kit.until_sentence(stage, "00_1", 3)?;'),
    fails=["alpha.rs:6: until_sentence(00_1, 3) but 00_1 has 3 sentences"],
)
refuses(
    "index far past the end",
    sub(ALPHA, UNTIL_00_1_2, 'kit.until_sentence(stage, "00_1", 99)?;'),
    fails=["alpha.rs:6: until_sentence(00_1, 99) but 00_1 has 3 sentences"],
)
refuses(
    "an index going backwards",
    sub(ALPHA, UNTIL_00_1_1, UNTIL_00_1_2),
    sub(ALPHA, f"{UNTIL_00_1_2}\n        {UNTIL_00_1_2}", f"{UNTIL_00_1_2}\n        {UNTIL_00_1_1}"),
    fails=["alpha.rs:6: until_sentence(00_1, 1) goes backwards"],
)
refuses(
    "a hook on a cue that is not the current line",
    sub(ALPHA, 'kit.until_sentence(stage, "00_2", 1)?;', UNTIL_00_1_2),
    fails=["alpha.rs:8: until_sentence(00_1, 2) while 00_2 is the current line"],
)
refuses(
    "a hook before any say",
    sub(ALPHA, 'kit.say(stage, "00_1")?;', f'{UNTIL_00_1_1}\n        kit.say(stage, "00_1")?;'),
    fails=["alpha.rs:4: until_sentence(00_1, 1) while None is the current line"],
)
refuses("a hook on an unknown cue", sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(stage, "00_9", 1)?;'), fails=["alpha.rs:5: unknown cue 00_9"])

# ---------------------------------------------------- forms it can't read

CANNOT = ["01_2: said 0 times (never)"]
refuses("a say with a computed cue", sub(BETA, SAY_01_2, "kit.say(stage, id)?;"), fails=["say(stage , id) does not name a literal cue", *CANNOT])
refuses("a say with a raw string cue", sub(BETA, SAY_01_2, 'kit.say(stage, r"01_2")?;'), fails=["does not name a literal cue", *CANNOT])
refuses("a say with another receiver", sub(BETA, SAY_01_2, 'k.say(stage, "01_2")?;'), fails=["say not called as kit.say", *CANNOT])
refuses("a say as a path call", sub(BETA, SAY_01_2, 'Kit::say(&kit, stage, "01_2")?;'), fails=["say not called as kit.say", *CANNOT])
refuses("a say passed around as a value", before_say("let _f = Kit::say;"), fails=["say not called as kit.say"])
refuses("a say with another stage", sub(BETA, SAY_01_2, 'kit.say(other, "01_2")?;'), fails=["does not name a literal cue", *CANNOT])
refuses("a say with extra arguments", sub(BETA, SAY_01_2, 'kit.say(stage, "01_2", 1)?;'), fails=["has extra arguments", *CANNOT])
refuses("a computed index nobody reviewed", sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(stage, "00_1", j)?;'), fails=["until_sentence(00_1, j) has a computed index: nobody has reviewed"])
refuses("a suffixed index", sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(stage, "00_1", 1usize)?;'), fails=["until_sentence(00_1, 1usize) has a computed index"])
refuses("a hook with no index", sub(ALPHA, UNTIL_00_1_1, 'kit.until_sentence(stage, "00_1")?;'), fails=["has no index"])
for label, attribute in (
    ("cfg(not(test))", "#[cfg(not(test))]\nfn f() {}\n"),
    ("cfg(any(test, ..))", '#[cfg(any(test, feature = "x"))]\nfn f() {}\n'),
    ("cfg(feature)", '#[cfg(feature = "x")]\nfn f() {}\n'),
    ("cfg_attr", '#[cfg_attr(test, derive(Debug))]\nstruct S;\n'),
    ("an inner cfg(test)", "#![cfg(test)]\n"),
):
    refuses(f"{label} is refused", add(BETA, attribute), fails=["a cfg this check can't evaluate", "01_1: said 0 times", "01_2: said 0 times"])
refuses("cfg!", add(BETA, "fn f() { if cfg!(test) {} }\n"), fails=["cfg! switches code", "01_1: said 0 times", "01_2: said 0 times"])
refuses(
    "a test attribute on a match arm can't hide the next arm",
    before_say('match x {\n            #[cfg(test)]\n            A => 1,\n            B => { kit.say(stage, "01_1"); }\n        }'),
    fails=["a test attribute on something that is not an item", "01_1: said 0 times", "01_2: said 0 times"],
)
for label, tail, reason in (
    ("a string that never closes", 'fn f() { "open\n', "a string never closes"),
    ("a raw string that never closes", 'fn f() { r#"open"\n', "a raw string never closes"),
    ("a block comment that never closes", "/* /* */\n", "a block comment never closes"),
    ("an unbalanced bracket in a test item", "#[cfg(test)]\nmod tests {\n", "{ never closes"),
):
    refuses(f"{label} is refused", add(BETA, tail), fails=[reason, "01_1: said 0 times", "01_2: said 0 times"])
refuses("a file that isn't UTF-8", put(BETA, b"\xff\xfe"), fails=["beta.rs: cannot be read: UnicodeDecodeError", "01_1: said 0 times", "01_2: said 0 times"])

# ---------------------------------------------------- the one reviewed computed index

passes("the reviewed computed index", base=COMPUTED, first=COMPUTED_FIRST, shows=["until_sentence(08_4, k + 1) runs 1 to 4 of 7 sentences"])
refuses("its array grows past the cue", sub(PRODUCT, 'r"d",', 'r"d", r"e", r"f", r"g",'), base=COMPUTED, fails=["until_sentence(08_4, k + 1) but 08_4 has 7 sentences", "product_rule.rs:17: until_sentence(08_4, 5) goes backwards"])
refuses("its array grows past the next literal", sub(PRODUCT, 'r"d",', 'r"d", r"e", r"f",'), base=COMPUTED, fails=["product_rule.rs:17: until_sentence(08_4, 5) goes backwards"])
refuses("another computed index on the same cue", sub(PRODUCT, HOOK_1, 'kit.until_sentence(stage, "08_4", k + 2)?;'), base=COMPUTED, fails=["until_sentence(08_4, k + 2) has a computed index: nobody has reviewed"])
refuses("its loop changes", sub(PRODUCT, LOOP, "for (k, &m) in steps.iter().rev().enumerate() {"), base=COMPUTED, fails=["reviewed loop"])
refuses("its fill changes", sub(PRODUCT, "for (k, src) in steps_src.iter().enumerate() {", "for (k, src) in other.iter().enumerate() {"), base=COMPUTED, fails=["reviewed fill"])
refuses("a second push", sub(PRODUCT, "    for (k, &m) in", "    steps.push(extra);\n    for (k, &m) in"), base=COMPUTED, fails=["reviewed push"])
refuses("its array goes", sub(PRODUCT, "let steps_src = [", "let steps_src = make(["), sub(PRODUCT, 'r"d",\n    ];', 'r"d",\n    ]);'), base=COMPUTED, fails=["reviewed array steps_src is gone"])
refuses("a literal after it, below its end", sub(PRODUCT, TAIL_5, 'kit.until_sentence(stage, "08_4", 3)?;'), base=COMPUTED, fails=["until_sentence(08_4, 3) goes backwards"])
refuses("a literal before it, above its start", sub(PRODUCT, "    let mut steps", '    kit.until_sentence(stage, "08_4", 2)?;\n    let mut steps'), base=COMPUTED, fails=["until_sentence(08_4, k + 1) goes backwards"])
refuses(
    "the same index in another file is not reviewed",
    sub(MOD, "mod product_rule;", "mod other;"),
    sub(MOD, 'key: "08_product_rule", make: |kit| Box::new(product_rule::ProductRule { kit })', 'key: "08_other", make: |kit| Box::new(other::Other { kit })'),
    put("src/chapters/other.rs", COMPUTED[PRODUCT]),
    drop(PRODUCT),
    base=COMPUTED,
    fails=["other.rs:14: until_sentence(08_4, k + 1) has a computed index: nobody has reviewed"],
)


def build(crate, case):
    files = dict(case["base"])
    for edit in case["edits"]:
        edit(files)
    for rel, body in files.items():
        path = crate / rel
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_bytes(body if isinstance(body, bytes) else body.encode("utf-8"))
    (crate / "src/chapters").mkdir(parents=True, exist_ok=True)
    (crate / "docs").mkdir(exist_ok=True)
    shutil.copyfile(CHECKER, crate / "docs/check_cues.py")


def judge(case, code, out):
    """What is wrong with this run, or None. Also the failure lines, to show."""
    lines = out.splitlines()
    if "Traceback" in out:
        return f"a traceback: {lines[-1] if lines else ''}", []
    if not lines:
        return "no output", []
    failures = [f.strip() for f in lines[lines.index("FAIL") + 1 :]] if "FAIL" in lines else []
    if case["fails"] is None:
        if code != 0 or lines[-1] != "OK" or failures:
            return f"expected a pass, got exit {code}: {failures}", failures
        if lines[0] != case["first"]:
            return f"first line {lines[0]!r}", failures
        for needle in case["shows"]:
            if not any(needle in line for line in lines):
                return f"output doesn't show {needle!r}", failures
        return None, failures
    if code != 1 or lines[-1] == "OK" or not failures:
        return f"expected a refusal, got exit {code}, last line {lines[-1]!r}", failures
    left = list(failures)
    for reason in case["fails"]:
        hits = [f for f in left if reason in f]
        if len(hits) != 1:
            return f"{reason!r} matched {len(hits)} of {left}", failures
        left.remove(hits[0])
    if left:
        return f"unlisted failures: {left}", failures
    return None, failures


def main(argv):
    if len(argv) != 1:
        raise SystemExit("usage: test_check_cues.py EMPTY_SCRATCH_DIR")
    scratch = Path(argv[0])
    if not scratch.is_dir() or any(scratch.iterdir()):
        raise SystemExit("scratch must be an existing empty directory")
    wrong = 0
    for n, case in enumerate(CASES, 1):
        crate = scratch / f"{n:03d}"
        build(crate, case)
        done = subprocess.run([sys.executable, "-I", "-B", str(crate / "docs/check_cues.py")], capture_output=True, text=True)
        problem, failures = judge(case, done.returncode, done.stdout + done.stderr)
        verb = "passes " if case["fails"] is None else "refuses"
        print(f"{'ok ' if problem is None else 'BAD'} {verb} {case['name']}")
        for failure in failures:
            print(f"      {failure}")
        if problem:
            wrong += 1
            print(f"    WRONG: {problem}")
    print(f"{len(CASES)} cases, {sum(c['fails'] is None for c in CASES)} must pass, {sum(c['fails'] is not None for c in CASES)} must refuse, {wrong} wrong")
    return 1 if wrong else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
