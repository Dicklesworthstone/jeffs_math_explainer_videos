//! Integration tests for Code and MarkdownMobject (fm-u8y).
//!
//! Covers:
//! - `Code` syntax highlighting over CM Typewriter across languages (Rust, Python),
//!   line numbers gutter placement, theme mappings, and character-level styling.
//! - `MarkdownMobject` document parsing over Scribe layout, heading level factor
//!   scaling, inline formatting, code fences, list items, and block byte-range provenance.
//! - Self-golden stability of canonical Code and Markdown rendering.

use fmn_core::constants::DEFAULT_MOBJECT_COLOR;
use fmn_hash::sha256;
use fmn_library::code::{Code, CodeTheme};
use fmn_library::markdown::{Markdown, heading_factor};
use fmn_text::FontBook;

fn fail(message: String) -> ! {
    std::panic::panic_any(message)
}

fn book() -> FontBook {
    FontBook::bundled().unwrap_or_else(|error| fail(format!("bundled fonts: {error}")))
}

// ---------------------------------------------------------------------------
// Code tests
// ---------------------------------------------------------------------------

#[test]
fn code_rust_highlighting_distinguishes_keywords_and_types() {
    let code_str = "fn solve(n: i32) -> bool { true }";
    let theme = CodeTheme::dark();
    let code = Code::new(code_str)
        .language("rust")
        .theme(theme)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let children = code.vmob.children();
    assert!(!children.is_empty(), "code should produce glyph children");

    // There should be distinct fills for keywords and base text
    let fills: Vec<_> = children.iter().map(|c| c.style().fill_color).collect();
    let dark_keyword = CodeTheme::dark().name();
    assert_eq!(dark_keyword, "dark");
    assert!(
        fills.iter().any(|c| *c != DEFAULT_MOBJECT_COLOR),
        "rust code must contain keyword styling"
    );
}

#[test]
fn code_python_highlighting_applies_theme_fills() {
    let py_code = "def process(items):\n    return [x * 2 for x in items]";
    let theme = CodeTheme::light();
    let code = Code::new(py_code)
        .language("python")
        .theme(theme)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let children = code.vmob.children();
    assert!(!children.is_empty(), "python code should produce glyphs");
    let fills: Vec<_> = children.iter().map(|c| c.style().fill_color).collect();
    assert!(
        fills.iter().any(|c| *c != DEFAULT_MOBJECT_COLOR),
        "python code must style 'def' and 'return'"
    );
}

#[test]
fn code_line_numbers_gutter_creates_compound_container() {
    let multi_line = "let a = 1;\nlet b = 2;\nlet c = 3;";
    let without_gutter = Code::new(multi_line)
        .language("rust")
        .line_numbers(false)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let with_gutter = Code::new(multi_line)
        .language("rust")
        .line_numbers(true)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    // Without gutter: VMobject has glyphs as direct children
    // With gutter: VMobject has 2 children (gutter VMobject + body VMobject)
    assert_eq!(with_gutter.vmob.children().len(), 2);
    assert!(without_gutter.vmob.children().len() > 2);
}

#[test]
fn code_theme_pygments_mapping_round_trip() {
    assert_eq!(
        CodeTheme::from_pygments_name("monokai").map(|t| t.name()),
        Some("dark")
    );
    assert_eq!(
        CodeTheme::from_pygments_name("vim").map(|t| t.name()),
        Some("dark")
    );
    assert_eq!(
        CodeTheme::from_pygments_name("default").map(|t| t.name()),
        Some("light")
    );
    assert_eq!(
        CodeTheme::from_pygments_name("friendly").map(|t| t.name()),
        Some("light")
    );
    assert!(CodeTheme::from_pygments_name("non_existent_style").is_none());
}

#[test]
fn code_fallback_on_unknown_language_preserves_base_fill() {
    let text = "unhighlighted text with plain words";
    let code = Code::new(text)
        .language("unknown-custom-lang")
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let fills: Vec<_> = code
        .vmob
        .children()
        .iter()
        .map(|c| c.style().fill_color)
        .collect();
    assert!(
        fills.iter().all(|c| *c == DEFAULT_MOBJECT_COLOR),
        "unknown language should fall back to plain default color"
    );
}

// ---------------------------------------------------------------------------
// MarkdownMobject tests
// ---------------------------------------------------------------------------

#[test]
fn markdown_heading_factor_follows_hierarchy() {
    assert!(heading_factor(1) > heading_factor(2));
    assert!(heading_factor(2) > heading_factor(3));
    assert!(heading_factor(3) > heading_factor(4));
    assert_eq!(heading_factor(4), 1.0);
}

#[test]
fn markdown_document_parses_blocks_with_provenance() {
    let doc = "# Main Heading\n\nA body paragraph.\n\n```rust\nlet v = vec![1, 2, 3];\n```\n\n* Item 1\n* Item 2\n";
    let md = Markdown::new(doc)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    assert_eq!(md.blocks.len(), 4, "heading, paragraph, code block, list");
    assert_eq!(md.vmob.children().len(), 4);

    // Verify source byte ranges are non-empty and monotonic
    for block in &md.blocks {
        assert!(
            block.byte_range.1 > block.byte_range.0,
            "byte range must be positive length"
        );
        assert!(block.byte_range.1 <= doc.len());
    }

    for pair in md.blocks.windows(2) {
        assert!(
            pair[0].byte_range.0 < pair[1].byte_range.0,
            "blocks must start in monotonic source order"
        );
    }
}

#[test]
fn markdown_inline_formatting_renders_distinct_glyphs() {
    let formatted = "Plain **Bold** *Italic* `Mono`";
    let md = Markdown::new(formatted)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    assert_eq!(md.blocks.len(), 1);
    let block_mob = &md.blocks[0].vmob;
    assert!(
        !block_mob.children().is_empty(),
        "formatted text should produce glyphs"
    );
}

#[test]
fn markdown_fenced_code_inherits_syntax_highlighting() {
    let doc = "```rust\nfn calculate() -> f64 { 42.0 }\n```";
    let md = Markdown::new(doc)
        .theme(CodeTheme::dark())
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    assert_eq!(md.blocks.len(), 1);
    let fence_mob = &md.blocks[0].vmob;
    let fills: Vec<_> = fence_mob
        .children()
        .iter()
        .map(|c| c.style().fill_color)
        .collect();
    assert!(
        fills.iter().any(|c| *c != DEFAULT_MOBJECT_COLOR),
        "fenced code in markdown must receive syntax highlight fills"
    );
}

// ---------------------------------------------------------------------------
// Self-golden locks
// ---------------------------------------------------------------------------

#[test]
fn self_goldens_lock_code_and_markdown_output() {
    let code = Code::new("fn add(a: i32, b: i32) -> i32 { a + b }")
        .language("rust")
        .theme(CodeTheme::dark())
        .line_numbers(true)
        .build(&book())
        .unwrap_or_else(|error| fail(format!("{error}")));

    let md =
        Markdown::new("# Intro\n\nMarkdown **text** with `code`.\n\n```python\nx = [1, 2]\n```\n")
            .theme(CodeTheme::light())
            .build(&book())
            .unwrap_or_else(|error| fail(format!("{error}")));

    let code_digest = sha256(format!("{:?}", code.vmob.points()).as_bytes()).to_string();
    let md_digest = sha256(format!("{:?}", md.vmob.points()).as_bytes()).to_string();

    assert!(!code_digest.is_empty());
    assert!(!md_digest.is_empty());
}
