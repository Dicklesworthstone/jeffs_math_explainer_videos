//! fm-u1u acceptance: shaping fixtures (kerning, ligatures on/off), line
//! breaking under manim's width semantics, markup goldens with precise
//! diagnostics, the t2c-family maps via span containment, submobject
//! indexing per the Reference's conventions, and the QuadPath output seam.

use fmn_core::color::Srgb;
use fmn_text::layout::BASELINE_SKIP;
use fmn_text::{
    Align, FontBook, LineBreaker, StyleMaps, TextError, TextLayout, TextRequest, glyph_quadpath,
    layout_text,
};

fn book() -> FontBook {
    FontBook::bundled().expect("bundled fonts parse")
}

fn lay(book: &FontBook, req: &TextRequest<'_>) -> TextLayout {
    layout_text(book, req).unwrap_or_else(|e| panic!("`{}` failed: {e}", req.text))
}

fn plain(book: &FontBook, text: &str) -> TextLayout {
    lay(book, &TextRequest::plain(text))
}

const EPS: f64 = 1e-9;

// ── Shaping ─────────────────────────────────────────────────────────────

#[test]
fn kerning_applies_within_runs() {
    let book = book();
    // Find a kerned pair among common candidates in bundled CM; assert the
    // laid-out width is exactly advance+kern+advance for that pair.
    let candidates = ["AV", "VA", "To", "Ta", "Yo", "AW", "LT"];
    let mut kerned = None;
    for pair in candidates {
        let l = plain(&book, pair);
        assert_eq!(l.glyphs.len(), 2);
        let isolated: f64 = pair
            .chars()
            .map(|c| {
                let s = c.to_string();
                plain(&book, &s).width
            })
            .sum();
        if (l.width - isolated).abs() > 1e-6 {
            kerned = Some((pair, l.width - isolated));
            break;
        }
    }
    let (pair, delta) = kerned.expect("bundled CM kerns at least one candidate pair");
    assert!(delta.abs() > 1e-6, "{pair} kern {delta}");
}

#[test]
fn ligatures_are_off_by_default_and_opt_in() {
    let book = book();
    let off = plain(&book, "fi");
    assert_eq!(off.glyphs.len(), 2, "the familiar manim look: no ligature");
    assert_eq!(off.glyphs[0].span, (0, 1));
    assert_eq!(off.glyphs[1].span, (1, 2));
    let mut req = TextRequest::plain("fi");
    req.ligatures = true;
    let on = lay(&book, &req);
    assert!(on.glyphs.len() <= 2);
    if on.glyphs.len() == 1 {
        // The ligature covers its whole character range (§11.3).
        assert_eq!(on.glyphs[0].span, (0, 2));
        assert_eq!(on.glyphs[0].cluster_len, 2);
    }
}

#[test]
fn unmapped_characters_are_named_errors() {
    let book = book();
    let err = layout_text(&book, &TextRequest::plain("ok \u{10FFFD} bad")).unwrap_err();
    match err {
        TextError::UnmappedChar { ch, .. } => assert_eq!(ch, '\u{10FFFD}'),
        other => panic!("expected UnmappedChar, got {other}"),
    }
}

/// GH #2: bundled Computer Modern has no U+2212 MINUS SIGN. The character
/// is drawn from the bundled fallback chain (the Noto Sans Math symbol
/// face), keeps its span and submobject slot, and is a real, wider glyph
/// than the ASCII hyphen-minus beside it.
#[test]
fn characters_missing_from_the_selected_face_fall_back_to_bundled_faces() {
    let book = book();
    let text = "-2.6 GW and \u{2212}2.6 GW";
    let l = plain(&book, text);
    let minus_at = text.find('\u{2212}').unwrap();
    let minus = l
        .glyphs
        .iter()
        .find(|g| g.ch == '\u{2212}')
        .expect("U+2212 is laid out");
    assert_eq!(minus.face.family, fmn_text::MATH_FAMILY);
    assert_eq!(minus.span, (minus_at, minus_at + '\u{2212}'.len_utf8()));
    let hyphen = &l.glyphs[0];
    assert_eq!(hyphen.ch, '-');
    assert_eq!(hyphen.face.family, fmn_text::DEFAULT_FAMILY);
    // Every other glyph stays in the selected face.
    assert!(
        l.glyphs
            .iter()
            .filter(|g| g.ch != '\u{2212}')
            .all(|g| g.face.family == fmn_text::DEFAULT_FAMILY)
    );
    // The fallback glyph has a real outline and advance, resolvable through
    // the book, and the true minus is wider than the hyphen.
    let minus_path = glyph_quadpath(&book, minus).expect("fallback outline");
    let hyphen_path = glyph_quadpath(&book, hyphen).expect("hyphen outline");
    let width = |p: &fmn_geom::QuadPath| {
        let xs = p.points().iter().map(|v| v[0]);
        xs.clone().fold(f64::NEG_INFINITY, f64::max) - xs.fold(f64::INFINITY, f64::min)
    };
    assert!(!minus_path.points().is_empty());
    assert!(
        width(&minus_path) > width(&hyphen_path),
        "true minus {} vs hyphen {}",
        width(&minus_path),
        width(&hyphen_path)
    );
    // The math axis: the minus sits clearly above the baseline.
    let mid_y = {
        let ys: Vec<f64> = minus_path.points().iter().map(|v| v[1]).collect();
        let lo = ys.iter().copied().fold(f64::INFINITY, f64::min);
        let hi = ys.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        (lo + hi) / 2.0 - minus.y
    };
    assert!((0.2..0.35).contains(&mid_y), "minus centre {mid_y} em");
    // Identical text laid out twice is identical (the chain is compiled in).
    assert_eq!(plain(&book, text), l);
}

#[test]
fn fallback_applies_to_explicit_families_and_is_not_selectable() {
    let book = book();
    // Explicit family (t2f) without the character: the chain still covers it,
    // and the covering face is recorded.
    let mut req = TextRequest::plain("a\u{2212}b");
    let binding = [("a\u{2212}b", "CM Typewriter")];
    req.maps = StyleMaps {
        t2f: &binding,
        ..StyleMaps::default()
    };
    let l = lay(&book, &req);
    let families: Vec<&str> = l.glyphs.iter().map(|g| g.face.family.as_str()).collect();
    assert_eq!(
        families,
        [
            fmn_text::MONO_FAMILY,
            fmn_text::MATH_FAMILY,
            fmn_text::MONO_FAMILY
        ]
    );
    // The chain's last resort: U+00BD VULGAR FRACTION ONE HALF is in neither
    // the Computer Modern subset nor the math symbol face, only Plex Sans.
    let half = plain(&book, "1\u{00BD}");
    assert_eq!(half.glyphs[0].face.family, fmn_text::DEFAULT_FAMILY);
    assert_eq!(half.glyphs[1].face.family, fmn_text::SANS_FAMILY);
    assert!(glyph_quadpath(&book, &half.glyphs[1]).is_ok());
    // The math face stays a coverage face, never a selectable family.
    assert!(book.family(fmn_text::MATH_FAMILY).is_err());
    let mut book = book;
    let conflict = book
        .add_family(
            fmn_text::MATH_FAMILY,
            fmd_font::bundled::CM_TYPEWRITER.to_vec(),
        )
        .unwrap_err();
    assert!(matches!(conflict, TextError::FontFamilyConflict { .. }));
}

// ── Font policy (D-08)──────────────────────────────────────────────────

#[test]
fn missing_family_is_a_named_capability_error_not_a_substitution() {
    let book = book();
    let mut req = TextRequest::plain("hi");
    let maps_binding = [("hi", "Comic Sans")];
    req.maps = StyleMaps {
        t2f: &maps_binding,
        ..StyleMaps::default()
    };
    let err = layout_text(&book, &req).unwrap_err();
    match err {
        TextError::FontUnavailable { family, available } => {
            assert_eq!(family, "Comic Sans");
            assert!(available.iter().any(|f| f == "Computer Modern"));
        }
        other => panic!("expected FontUnavailable, got {other}"),
    }
}

#[test]
fn bundled_family_aliases_resolve() {
    let book = book();
    assert!(book.family("computer modern").is_ok());
    assert!(book.family("CMU Serif").is_ok());
    assert!(book.family("ibm-plex-sans").is_ok());
    assert!(book.family("Papyrus").is_err());
}

#[test]
fn user_family_registration_refuses_unselectable_names() {
    let bytes = fmd_font::bundled::CM_TYPEWRITER.to_vec();
    let mut book = book();
    let initial_families = book.available();

    for (name, font_bytes) in [
        ("computer-modern", Vec::new()),
        ("CMU Serif", bytes.clone()),
    ] {
        let error = book.add_family(name, font_bytes).unwrap_err();
        match error {
            TextError::FontFamilyConflict {
                requested,
                existing,
            } => {
                assert_eq!(requested, name);
                assert_eq!(existing, "Computer Modern");
            }
            other => std::panic::panic_any(format!("expected FontFamilyConflict, got {other}")),
        }
        assert_eq!(book.available(), initial_families);
    }

    book.add_family("My Face", bytes.clone())
        .expect("a distinct family is admitted");
    assert_eq!(book.family("my_face").unwrap().name, "My Face");
    let duplicate = book.add_family("my face", Vec::new()).unwrap_err();
    assert!(matches!(
        duplicate,
        TextError::FontFamilyConflict {
            ref requested,
            ref existing,
        } if requested == "my face" && existing == "My Face"
    ));
    assert_eq!(book.available().len(), initial_families.len() + 1);

    let malformed = book
        .add_family("Broken Face", bytes[..32].to_vec())
        .unwrap_err();
    assert!(matches!(
        malformed,
        TextError::FontParse { ref path, .. } if path == "Broken Face"
    ));
}

// ── Line breaking (manim width semantics) ───────────────────────────────

#[test]
fn newlines_always_break() {
    let book = book();
    let l = plain(&book, "ab\ncd");
    assert_eq!(l.lines.len(), 2);
    assert!((l.lines[0].baseline - 0.0).abs() < EPS);
    assert!((l.lines[1].baseline + BASELINE_SKIP).abs() < EPS);
    assert_eq!(l.glyphs[0].line, 0);
    assert_eq!(l.glyphs[2].line, 1);
}

#[test]
fn greedy_wrapping_fills_lines() {
    let book = book();
    let unwrapped = plain(&book, "aa bb cc");
    // A measure that fits "aa bb" but not "aa bb cc".
    let two_words = {
        let l = plain(&book, "aa bb");
        l.width
    };
    let mut req = TextRequest::plain("aa bb cc");
    req.width = Some(two_words + 0.05);
    let l = lay(&book, &req);
    assert_eq!(l.lines.len(), 2, "greedy: [aa bb][cc]");
    assert_eq!(l.lines[0].glyphs, (0, 4));
    assert_eq!(l.lines[1].glyphs, (4, 6));
    assert!(l.width <= unwrapped.width);
}

#[test]
fn no_width_means_no_wrapping() {
    let book = book();
    let l = plain(&book, "aa bb cc dd ee ff");
    assert_eq!(l.lines.len(), 1);
}

#[test]
fn least_badness_is_never_worse_than_greedy() {
    let book = book();
    let text = "aaa bb cc ddd ee fff g hh";
    let measure = 3.0;
    let slack_sq = |l: &TextLayout| -> f64 {
        l.lines
            .iter()
            .take(l.lines.len() - 1)
            .map(|line| {
                let s = (measure - line.width).max(0.0);
                s * s
            })
            .sum()
    };
    let mut greedy = TextRequest::plain(text);
    greedy.width = Some(measure);
    let g = lay(&book, &greedy);
    let mut lb = TextRequest::plain(text);
    lb.width = Some(measure);
    lb.breaker = LineBreaker::LeastBadness;
    let b = lay(&book, &lb);
    assert!(
        slack_sq(&b) <= slack_sq(&g) + EPS,
        "least-badness ({}) must not exceed greedy ({})",
        slack_sq(&b),
        slack_sq(&g)
    );
}

#[test]
fn alignment_and_indent_offset_lines() {
    let book = book();
    let text = "aa\nbbbb";
    let left = plain(&book, text);
    let mut center = TextRequest::plain(text);
    center.align = Align::Center;
    let c = lay(&book, &center);
    let mut right = TextRequest::plain(text);
    right.align = Align::Right;
    let r = lay(&book, &right);
    // The short first line shifts right under center/right alignment.
    assert!(c.glyphs[0].x > left.glyphs[0].x + 1e-6);
    assert!(r.glyphs[0].x > c.glyphs[0].x + 1e-6);
    // Indent moves only the first line.
    let mut ind = TextRequest::plain(text);
    ind.indent = 0.5;
    let i = lay(&book, &ind);
    assert!((i.glyphs[0].x - (left.glyphs[0].x + 0.5)).abs() < EPS);
    let first_of_line2 = i.lines[1].glyphs.0;
    assert!((i.glyphs[first_of_line2].x - 0.0).abs() < EPS);
}

#[test]
fn justification_stretches_interword_spaces_except_the_last_line() {
    let book = book();
    let mut req = TextRequest::plain("aa bb cc dd ee");
    let two = plain(&book, "aa bb").width;
    req.width = Some(two + 0.4);
    req.justify = true;
    let l = lay(&book, &req);
    assert!(l.lines.len() >= 2);
    // Every justified line ends at the measure.
    for line in &l.lines[..l.lines.len() - 1] {
        assert!(
            (line.width - (two + 0.4)).abs() < 1e-6,
            "justified line width {} vs measure {}",
            line.width,
            two + 0.4
        );
    }
    // The last line stays natural (shorter).
    assert!(l.lines[l.lines.len() - 1].width < two + 0.4 - 1e-6);
}

#[test]
fn line_spacing_scales_the_baseline_skip() {
    let book = book();
    let mut req = TextRequest::plain("a\nb");
    req.line_spacing = 1.5;
    let l = lay(&book, &req);
    assert!((l.lines[1].baseline + BASELINE_SKIP * 1.5).abs() < EPS);
}

// ── Markup ──────────────────────────────────────────────────────────────

#[test]
fn markup_styles_resolve_to_faces_and_sizes() {
    let book = book();
    let l = lay(&book, &TextRequest::markup("<b>a</b>b<i>c</i>"));
    assert_eq!(l.glyphs.len(), 3);
    assert!(l.glyphs[0].face.key.bold && !l.glyphs[0].face.key.italic);
    assert!(!l.glyphs[1].face.key.bold);
    assert!(l.glyphs[2].face.key.italic);
    let l = lay(&book, &TextRequest::markup("<big>a</big>a<small>a</small>"));
    assert!((l.glyphs[0].size - 1.2).abs() < EPS);
    assert!((l.glyphs[1].size - 1.0).abs() < EPS);
    assert!((l.glyphs[2].size - 5.0 / 6.0).abs() < EPS);
}

#[test]
fn scripts_shift_the_baseline() {
    let book = book();
    let l = lay(&book, &TextRequest::markup("x<sup>2</sup><sub>i</sub>"));
    assert!((l.glyphs[0].y - 0.0).abs() < EPS);
    assert!(l.glyphs[1].y > 0.2, "sup raised: {}", l.glyphs[1].y);
    assert!(l.glyphs[2].y < -0.05, "sub lowered: {}", l.glyphs[2].y);
    assert!(l.glyphs[1].size < 1.0);
}

#[test]
fn span_colors_and_tt_apply() {
    let book = book();
    let l = lay(
        &book,
        &TextRequest::markup(r##"<span foreground="#FF0000">r</span><tt>m</tt>"##),
    );
    let red = l.glyphs[0].fill.expect("colored");
    assert_eq!(red.to_hex().to_uppercase(), "#FF0000");
    assert_eq!(l.glyphs[1].face.family, fmn_text::MONO_FAMILY);
}

#[test]
fn underline_and_strike_produce_decorations() {
    let book = book();
    let l = lay(&book, &TextRequest::markup("<u>ab</u> <s>c</s>"));
    assert_eq!(l.decorations.len(), 2);
    assert!(l.decorations[0].y < 0.0, "underline below the baseline");
    assert!(l.decorations[1].y > 0.0, "strike above the baseline");
    assert_eq!(l.decorations[0].span, (3, 5));
}

#[test]
fn entities_decode_with_whole_entity_spans() {
    let book = book();
    let l = lay(&book, &TextRequest::markup("&amp;&#65;"));
    assert_eq!(l.glyphs[0].ch, '&');
    assert_eq!(l.glyphs[0].span, (0, 5));
    assert_eq!(l.glyphs[1].ch, 'A');
    assert_eq!(l.glyphs[1].span, (5, 10));
}

#[test]
fn malformed_markup_diagnostics_carry_line_and_column() {
    let book = book();
    let cases: &[(&str, &str, usize, usize)] = &[
        ("a<b>x", "never closed", 1, 6),
        ("</i>", "with nothing open", 1, 1),
        ("<b>x</i>", "closes <b>", 1, 5),
        ("ab\n<nope>x</nope>", "unknown tag", 2, 1),
        ("<span bad=\"1\">x</span>", "unknown <span> attribute", 1, 1),
        ("<b foo=\"1\">x</b>", "takes no attributes", 1, 1),
        ("a&nope;b", "unknown entity", 1, 2),
        ("a<b", "unclosed '<'", 1, 2),
    ];
    for (src, needle, line, col) in cases {
        let err = layout_text(&book, &TextRequest::markup(src)).unwrap_err();
        match err {
            TextError::Markup {
                what,
                line: l,
                col: c,
            } => {
                assert!(what.contains(needle), "`{src}`: {what}");
                assert_eq!((l, c), (*line, *col), "`{src}` position");
            }
            other => panic!("`{src}`: expected Markup error, got {other}"),
        }
    }
}

#[test]
fn tag_depth_is_bounded() {
    let book = book();
    let deep = format!("{}x{}", "<b>".repeat(40), "</b>".repeat(40));
    let err = layout_text(&book, &TextRequest::markup(&deep)).unwrap_err();
    assert!(matches!(err, TextError::Markup { ref what, .. } if what.contains("depth limit")));
}

// ── The t2c-family maps ─────────────────────────────────────────────────

#[test]
fn t2c_colors_by_source_occurrence() {
    let book = book();
    let t2c = [("bc", Srgb::from_hex("#00FF00").unwrap())];
    let mut req = TextRequest::plain("abc bc");
    req.maps = StyleMaps {
        t2c: &t2c,
        ..StyleMaps::default()
    };
    let l = lay(&book, &req);
    let colored: Vec<char> = l
        .glyphs
        .iter()
        .filter(|g| g.fill.is_some())
        .map(|g| g.ch)
        .collect();
    assert_eq!(colored, vec!['b', 'c', 'b', 'c']);
    assert!(l.glyphs[0].fill.is_none(), "'a' stays uncolored");
}

#[test]
fn t2w_changes_the_face_and_the_metrics() {
    let book = book();
    let t2w = [("bold", true)];
    let mut req = TextRequest::plain("bold thin");
    req.maps = StyleMaps {
        t2w: &t2w,
        ..StyleMaps::default()
    };
    let l = lay(&book, &req);
    assert!(l.glyphs[0].face.key.bold);
    assert!(!l.glyphs[4].face.key.bold);
}

#[test]
fn t2g_sweeps_a_gradient_across_the_occurrence() {
    let book = book();
    let stops = [
        Srgb::from_hex("#000000").unwrap(),
        Srgb::from_hex("#FFFFFF").unwrap(),
    ];
    let t2g: [(&str, &[Srgb]); 1] = [("abcd", &stops)];
    let mut req = TextRequest::plain("abcd");
    req.maps = StyleMaps {
        t2g: &t2g,
        ..StyleMaps::default()
    };
    let l = lay(&book, &req);
    let first = l.glyphs[0].fill.expect("gradient fill");
    let last = l.glyphs[3].fill.expect("gradient fill");
    assert!(first.r < 0.01, "first stop black");
    assert!(last.r > 0.99, "last stop white");
    let mid = l.glyphs[1].fill.expect("gradient fill");
    assert!(mid.r > 0.05 && mid.r < 0.95, "interior interpolates");
}

// ── Submobject indexing (the Reference's conventions) ───────────────────

#[test]
fn submobjects_are_nonspace_glyphs_in_order() {
    let book = book();
    let l = plain(&book, "ab cd");
    assert_eq!(l.submobject_count(), 4, "spaces produce no submobject");
    let chars: Vec<char> = l.glyphs.iter().map(|g| g.ch).collect();
    assert_eq!(chars, vec!['a', 'b', 'c', 'd']);
    for (i, g) in l.glyphs.iter().enumerate() {
        assert_eq!(g.submobject_index, i);
    }
    let slice: Vec<char> = l.submobject_slice(1, 3).iter().map(|g| g.ch).collect();
    assert_eq!(slice, vec!['b', 'c'], "Text[1:3]");
}

#[test]
fn isolate_style_selection_by_source_bytes() {
    let book = book();
    let src = "ab cd";
    let l = plain(&book, src);
    let cd = l.select((3, 5));
    assert_eq!(cd.len(), 2);
    assert_eq!(l.glyphs[cd[0]].ch, 'c');
}

// ── Output geometry ─────────────────────────────────────────────────────

#[test]
fn glyph_outlines_are_positioned_quadpaths() {
    let book = book();
    let l = plain(&book, "oo");
    let p0 = glyph_quadpath(&book, &l.glyphs[0]).unwrap();
    let p1 = glyph_quadpath(&book, &l.glyphs[1]).unwrap();
    assert!(p0.has_points() && p1.has_points());
    assert!(p0.num_points() > 8, "an 'o' has two contours of curves");
    // The second 'o' is the same outline translated by the advance.
    let dx = l.glyphs[1].x - l.glyphs[0].x;
    assert!(dx > 0.1);
    for (a, b) in p0.points().iter().zip(p1.points().iter()) {
        assert!((a[0] + dx - b[0]).abs() < 1e-9);
        assert!((a[1] - b[1]).abs() < 1e-9);
    }
}

// ── Chaos (markup is untrusted-adjacent) ────────────────────────────────

#[test]
fn random_markup_never_panics() {
    let book = book();
    let mut state = 0x00DD_F00D_u64;
    let pool = [
        "<",
        ">",
        "</",
        "b",
        "i",
        "span",
        "sub",
        "sup",
        "&",
        ";",
        "amp",
        "#",
        "\"",
        "=",
        " ",
        "foreground",
        "x",
        "\n",
        "big",
        "<b>",
        "</b>",
        "<span ",
        "'",
        "🦀",
        "&#x41;",
    ];
    for _ in 0..4_000 {
        state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        let len = (state >> 40) as usize % 20;
        let mut s = String::new();
        let mut local = state;
        for _ in 0..len {
            local = local
                .wrapping_mul(6_364_136_223_846_793_005)
                .wrapping_add(1_442_695_040_888_963_407);
            s.push_str(pool[(local >> 33) as usize % pool.len()]);
        }
        // Must return a Result — never panic, never hang.
        let _ = layout_text(&book, &TextRequest::markup(&s));
        let _ = layout_text(&book, &TextRequest::plain(&s));
    }
}
