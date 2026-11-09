//! Native macro preambles preserve source identity, cache semantics and bounds.
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fmn_tex::{
    LineAlign, Mode, Span, Style, TEX_PREAMBLE_MAX_BYTES, TEX_PREAMBLE_SOURCE_MAX_BYTES, TexEngine,
    TexError,
};

const MATH: Mode = Mode::Math(Style::Display);

fn engine() -> TexEngine {
    TexEngine::new("fmd-math/pack/default", None).unwrap()
}

#[test]
fn empty_preamble_is_the_unchanged_typeset_path_in_every_mode() {
    let engine = engine();
    for mode in [
        MATH,
        Mode::Math(Style::Text),
        Mode::Math(Style::Script),
        Mode::Text,
    ] {
        let expected = engine.typeset(mode, "x").unwrap().to_bytes().unwrap();
        let actual = engine.typeset_with_preamble(mode, "x", "").unwrap();
        assert_eq!(actual.to_bytes().unwrap(), expected);
    }
}

#[test]
fn declarations_and_trailing_comments_do_not_change_plain_formula_geometry() {
    let engine = engine();
    for mode in [MATH, Mode::Text] {
        let plain = engine.typeset(mode, "x+y").unwrap().to_bytes().unwrap();
        for preamble in [
            "% a comment without a newline",
            r"\newcommand{\unused}{x}% end",
            " \n% comment\n",
        ] {
            let actual = engine.typeset_with_preamble(mode, "x+y", preamble).unwrap();
            assert_eq!(actual.to_bytes().unwrap(), plain, "{mode:?}: {preamble}");
        }
    }
}

#[test]
fn parameters_call_sites_rules_and_drawn_paths_keep_original_formula_spans() {
    let engine = engine();
    let preamble = r"\newcommand{\half}[1]{\frac{#1}{2}}";
    let source = r"\half{x}+\left[\frac{y}{z}\right]";
    let actual = engine
        .typeset_with_preamble(MATH, source, preamble)
        .unwrap();
    assert_eq!(actual.source, source);
    assert_eq!(actual.layout.rules.len(), 2);
    assert!(!actual.occurrences("x")[0].is_empty());
    // Selection is containment, not overlap: generated ink belongs to the
    // complete call (including arguments), never to its command-name prefix.
    assert!(actual.occurrences(r"\half")[0].is_empty());
    assert!(!actual.occurrences(r"\half{x}")[0].is_empty());
    assert!(actual.subs.iter().any(|sub| sub.span == Span::new(6, 7)));
    assert!(actual.subs.iter().any(|sub| sub.span == Span::new(0, 8)));
    for sub in &actual.subs {
        assert!(sub.span.end <= source.len());
        assert!(source.is_char_boundary(sub.span.start));
        assert!(source.is_char_boundary(sub.span.end));
        engine.resolve_prim(&actual, sub.prim).unwrap();
    }
    assert_eq!(
        fmn_tex::Typeset::from_bytes(&actual.to_bytes().unwrap())
            .unwrap()
            .source,
        source
    );
}

#[test]
fn text_mainland_and_math_islands_keep_utf8_source_coordinates() {
    let engine = engine();
    let source = "é area $\\sq{x}$";
    let actual = engine
        .typeset_with_preamble(Mode::Text, source, r"\newcommand{\sq}[1]{#1^2}")
        .unwrap();
    assert_eq!(actual.source, source);
    for token in ["é", "x", r"\sq{x}"] {
        assert!(!actual.occurrences(token)[0].is_empty(), "{token}");
    }
    assert!(actual.occurrences(r"\sq")[0].is_empty());
    for sub in &actual.subs {
        assert!(source.is_char_boundary(sub.span.start));
        assert!(source.is_char_boundary(sub.span.end));
    }
}

#[test]
fn macros_are_request_local_and_pack_shadowing_rules_are_native() {
    let engine = engine();
    let preamble = r"\newcommand{\local}{x}";
    assert!(
        engine
            .typeset_with_preamble(MATH, r"\local", preamble)
            .is_ok()
    );
    assert!(engine.typeset(MATH, r"\local").is_err());
    assert!(
        engine
            .typeset_with_preamble(MATH, r"\minus", r"\newcommand{\minus}{+}")
            .is_err()
    );
    assert!(
        engine
            .typeset_with_preamble(MATH, r"\minus", r"\renewcommand{\minus}{+}")
            .is_ok()
    );
    let empty = TexEngine::new("fmd-math/pack/empty", None).unwrap();
    assert!(
        empty
            .typeset_with_preamble(MATH, r"\minus", r"\newcommand{\minus}{+}")
            .is_ok()
    );
    assert!(
        empty
            .typeset_with_preamble(MATH, "x", r"\renewcommand{\minus}{+}")
            .is_err()
    );
}

#[test]
fn different_definitions_cannot_reuse_another_cached_expansion() {
    let engine = engine();
    let source = r"\coefficient";
    let x = r"\newcommand{\coefficient}{x}";
    let y = r"\newcommand{\coefficient}{y^2}";
    let first = engine
        .typeset_with_preamble(MATH, source, x)
        .unwrap()
        .to_bytes()
        .unwrap();
    let second = engine
        .typeset_with_preamble(MATH, source, y)
        .unwrap()
        .to_bytes()
        .unwrap();
    assert_ne!(first, second);
    let mut edited = engine.typeset_with_preamble(MATH, source, x).unwrap();
    edited.source.clear();
    edited.layout.glyphs.clear();
    edited.subs.clear();
    assert_eq!(
        engine
            .typeset_with_preamble(MATH, source, x)
            .unwrap()
            .to_bytes()
            .unwrap(),
        first
    );
    assert_eq!(
        engine
            .typeset_with_preamble(MATH, source, y)
            .unwrap()
            .to_bytes()
            .unwrap(),
        second
    );
    assert!(engine.memory_cache_stats().hits >= 4);
}

#[test]
fn disk_hits_project_once_without_corrupting_effective_source_documents() {
    use fmn_cache::{NamespacePolicy, Store, StoreConfig};
    use fmn_platform::clock::FakeClock;
    use fmn_platform::fs::VirtualFs;
    use std::sync::Arc;

    let root = if cfg!(windows) {
        r"C:\preamble-cache"
    } else {
        "/preamble-cache"
    };
    let store = Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        root,
        StoreConfig::default(),
    )
    .unwrap();
    let writer = engine().with_cache(&store).unwrap();
    let source = r"\sq{x}";
    let preamble = r"\newcommand{\sq}[1]{#1^2}";
    let expected = writer
        .typeset_with_preamble(MATH, source, preamble)
        .unwrap();
    let effective = format!("{preamble}\n{source}");
    let namespace = store
        .namespace(
            "typeset",
            fmn_tex::TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let key = writer.cache_key(MATH, &effective).unwrap();
    let cached_bytes = namespace.get(&key).unwrap().unwrap();
    let cached = fmn_tex::Typeset::from_bytes(&cached_bytes).unwrap();
    assert_eq!(cached.source, effective);
    assert!(
        cached
            .subs
            .iter()
            .all(|sub| sub.span.start > preamble.len())
    );

    // Fresh engines have no resident entries: each reader must project a disk
    // result. A subsequent resident hit must not subtract the prefix twice.
    for _ in 0..2 {
        let reader = engine().with_cache(&store).unwrap();
        for _ in 0..2 {
            let result = reader
                .typeset_with_preamble(MATH, source, preamble)
                .unwrap();
            assert_eq!(result.to_bytes().unwrap(), expected.to_bytes().unwrap());
            assert_eq!(result.occurrences("x"), expected.occurrences("x"));
        }
    }
    assert_eq!(namespace.get(&key).unwrap().unwrap(), cached_bytes);
}

#[test]
fn native_formula_diagnostics_are_rebased_and_declarations_are_named() {
    let engine = engine();
    let preamble = "% UTF-8 é\n\\newcommand{\\local}{x}";
    let source = r"x+\unknownnativecommand";
    let expected = engine.typeset(MATH, source).unwrap_err().to_string();
    assert_eq!(
        engine
            .typeset_with_preamble(MATH, source, preamble)
            .unwrap_err()
            .to_string(),
        expected
    );
    let error = engine
        .typeset_with_preamble(MATH, "x", r"\newcommand{\bad}[1]{#2}")
        .unwrap_err();
    assert!(matches!(error, TexError::Preamble { .. }));
    assert!(error.to_string().starts_with("additional_preamble:"));
    let recursion = engine
        .typeset_with_preamble(MATH, r"\loop", r"\newcommand{\loop}{\loop}")
        .unwrap_err();
    assert!(recursion.to_string().contains("loop"));
}

#[test]
fn visible_content_external_tools_and_over_budget_sources_are_refused() {
    let engine = engine();
    for preamble in [
        "x",
        r"\quad",
        r"\usepackage{amsmath}",
        r"\input{secret.tex}",
        r"\write18{command}",
    ] {
        let error = engine
            .typeset_with_preamble(MATH, "x", preamble)
            .unwrap_err();
        assert!(
            matches!(error, TexError::Preamble { .. }),
            "{preamble}: {error}"
        );
    }
    let at_limit = format!("%{}", "a".repeat(TEX_PREAMBLE_MAX_BYTES - 1));
    assert!(engine.typeset_with_preamble(MATH, "x", &at_limit).is_ok());
    let over = format!("{at_limit}a");
    assert!(engine.typeset_with_preamble(MATH, "x", &over).is_err());
    let huge_formula = "x".repeat(TEX_PREAMBLE_SOURCE_MAX_BYTES);
    assert!(matches!(
        engine.typeset_with_preamble(MATH, &huge_formula, "% comment"),
        Err(TexError::Preamble { .. })
    ));
}

/// Leftmost glyph origin of the glyphs whose source starts inside `bytes`.
fn line_start(typeset: &fmn_tex::Typeset, bytes: std::ops::Range<usize>) -> f64 {
    typeset
        .layout
        .glyphs
        .iter()
        .filter(|glyph| bytes.contains(&glyph.span.start))
        .map(|glyph| glyph.x)
        .fold(f64::INFINITY, f64::min)
}

#[test]
fn line_alignment_places_each_line_by_its_slack_and_keeps_body_spans() {
    let engine = engine();
    // "ab" is the short first line; the second line starts at byte 4.
    let source = r"ab\\abcdefgh";
    let short_line_lead = |align| {
        let typeset = engine
            .typeset_aligned(Mode::Text, source, "", align)
            .unwrap();
        assert_eq!(typeset.source, source);
        for sub in &typeset.subs {
            assert!(sub.span.end <= source.len(), "{align:?}: {:?}", sub.span);
        }
        line_start(&typeset, 0..2) - line_start(&typeset, 4..source.len())
    };
    let left = short_line_lead(LineAlign::Left);
    let center = short_line_lead(LineAlign::Center);
    let right = short_line_lead(LineAlign::Right);
    assert!(left.abs() < 1e-9, "{left}");
    assert!(right > 1.0, "{right}");
    // Centering places half of the slack that flush-right places.
    assert!((center - right / 2.0).abs() < 1e-9, "{center} vs {right}");
    // Left is the preamble path, byte for byte.
    let preamble = r"\newcommand{\mine}{x}";
    assert_eq!(
        engine
            .typeset_aligned(Mode::Text, source, preamble, LineAlign::Left)
            .unwrap()
            .to_bytes()
            .unwrap(),
        engine
            .typeset_with_preamble(Mode::Text, source, preamble)
            .unwrap()
            .to_bytes()
            .unwrap()
    );
}

#[test]
fn aligned_bodies_keep_preamble_macros_and_body_spans() {
    let engine = engine();
    let source = r"\mine\\ab";
    let typeset = engine
        .typeset_aligned(
            Mode::Text,
            source,
            r"\newcommand{\mine}{$x^2$}",
            LineAlign::Center,
        )
        .unwrap();
    assert_eq!(typeset.source, source);
    assert!(!typeset.occurrences(r"\mine")[0].is_empty());
    assert!(!typeset.occurrences("ab")[0].is_empty());
    for sub in &typeset.subs {
        assert!(source.is_char_boundary(sub.span.start));
        assert!(sub.span.end <= source.len());
    }
}

#[test]
fn alignment_is_text_only_and_body_errors_keep_body_coordinates() {
    let engine = engine();
    assert!(matches!(
        engine.typeset_aligned(MATH, "x", "", LineAlign::Center),
        Err(TexError::Preamble { .. })
    ));
    let source = r"ab \unknownnativecommand";
    let expected = engine.typeset(Mode::Text, source).unwrap_err().to_string();
    for align in [LineAlign::Center, LineAlign::Right] {
        let error = engine
            .typeset_aligned(Mode::Text, source, "", align)
            .unwrap_err();
        assert!(matches!(error, TexError::Math(_)), "{align:?}: {error}");
        assert_eq!(error.to_string(), expected, "{align:?}");
    }
}
