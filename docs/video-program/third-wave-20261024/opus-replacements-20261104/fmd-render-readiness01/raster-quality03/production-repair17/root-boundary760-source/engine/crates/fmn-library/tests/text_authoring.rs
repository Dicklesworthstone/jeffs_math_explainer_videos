use fmn_library::{FontBook, Text};

#[test]
fn base_font_face_and_source_spans_stay_native() {
    let book = FontBook::bundled().unwrap();
    let text = Text::new("éA")
        .font("IBM Plex Sans")
        .bold(true)
        .italic(true)
        .build(&book)
        .unwrap();
    assert_eq!(&*text.source, "éA");
    assert_eq!(
        text.layout
            .glyphs
            .iter()
            .map(|g| g.span)
            .collect::<Vec<_>>(),
        [(0, 2), (2, 3)]
    );
    for glyph in &text.layout.glyphs {
        assert_eq!(glyph.face.family, "IBM Plex Sans");
        assert!(glyph.face.key.bold && glyph.face.key.italic);
    }
}

#[test]
fn markup_overrides_and_restores_inherited_style_without_prefixes() {
    let book = FontBook::bundled().unwrap();
    let source = "A<span weight=\"normal\" font_family=\"Computer Modern\">B</span>C";
    let text = Text::markup(source)
        .font("IBM Plex Sans")
        .bold(true)
        .build(&book)
        .unwrap();
    let glyphs = &text.layout.glyphs;
    assert_eq!(glyphs.len(), 3);
    assert!(glyphs[0].face.key.bold && glyphs[2].face.key.bold);
    assert!(!glyphs[1].face.key.bold);
    assert_eq!(glyphs[0].face.family, "IBM Plex Sans");
    assert_eq!(glyphs[1].face.family, "Computer Modern");
    assert_eq!(glyphs[2].face.family, "IBM Plex Sans");
    assert_eq!(&source[glyphs[1].span.0..glyphs[1].span.1], "B");
}

#[test]
fn local_maps_win_over_base_and_markup_in_source_order() {
    let book = FontBook::bundled().unwrap();
    let text = Text::markup("<b>ABA</b>")
        .font("IBM Plex Sans")
        .t2f(&[("B", "Computer Modern")])
        .t2w(&[("A", false)])
        .t2s(&[("B", true)])
        .build(&book)
        .unwrap();
    assert!(!text.layout.glyphs[0].face.key.bold);
    assert!(text.layout.glyphs[1].face.key.bold && text.layout.glyphs[1].face.key.italic);
    assert_eq!(text.layout.glyphs[1].face.family, "Computer Modern");
    assert!(!text.layout.glyphs[2].face.key.bold);
}

#[test]
fn empty_text_does_not_hide_unknown_base_font() {
    let book = FontBook::bundled().unwrap();
    assert!(
        Text::new("")
            .font("definitely-not-installed")
            .build(&book)
            .is_err()
    );
}

#[test]
fn explicit_default_preserves_existing_geometry() {
    let book = FontBook::bundled().unwrap();
    let old = Text::new("Text A").build(&book).unwrap();
    let new = Text::new("Text A")
        .font("")
        .bold(false)
        .italic(false)
        .build(&book)
        .unwrap();
    assert_eq!(old.layout.glyphs, new.layout.glyphs);
}

#[test]
fn inherited_family_does_not_mask_an_inner_monospace_tag() {
    let book = FontBook::bundled().unwrap();
    let source = "<tt>A</tt>B";
    let built = Text::markup(source)
        .font("IBM Plex Sans")
        .build(&book)
        .unwrap();
    assert_eq!(built.layout.glyphs[0].face.family, "CM Typewriter");
    assert_eq!(built.layout.glyphs[0].span, (4, 5));
    assert_eq!(built.layout.glyphs[1].face.family, "IBM Plex Sans");
}

#[test]
fn native_highlighter_exposes_positional_styles_without_source_rewriting() {
    use fmn_core::color::Srgb;
    use fmn_library::{Code, CodeTheme};
    let styles = Code::new("fn fnord")
        .language("rust")
        .theme(CodeTheme::dark())
        .character_overrides();
    assert_eq!(styles.len(), 8);
    assert_eq!(styles[0].color, Some(Srgb::from_rgb8(104, 194, 70)));
    assert!(
        styles[3..]
            .iter()
            .all(|style| style.color != styles[0].color)
    );
    assert!(
        styles
            .iter()
            .all(|style| style.family == Some("CM Typewriter"))
    );
}
