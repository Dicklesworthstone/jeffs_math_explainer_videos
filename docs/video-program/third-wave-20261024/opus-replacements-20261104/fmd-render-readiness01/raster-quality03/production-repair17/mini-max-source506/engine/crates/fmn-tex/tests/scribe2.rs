//! The fm-7dw acceptance suite: Tex→submobject structural fixtures, the
//! span-map consumption surface, cache round-trip and key sensitivity
//! (pack/engine changes miss correctly; hits are bit-identical), the
//! parallel preflight's cache-warming contract, and the named error
//! surface at construction time.

#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use fmn_cache::{NamespacePolicy, Store, StoreConfig};
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::VirtualFs;
use fmn_tex::{MacroSet, Mode, Style, TexEngine, Typeset, TypesetError};
use std::sync::Arc;

const MAGIC_V2: &[u8; 8] = b"FMNTEX\x00\x02";

fn engine() -> TexEngine {
    TexEngine::new("fmd-math/pack/default", None).expect("engine")
}

// Host path semantics validate the root even over a VirtualFs.
const VIRTUAL_CACHE_ROOT: &str = if cfg!(windows) { r"C:\cache" } else { "/cache" };

fn store() -> Store {
    Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        VIRTUAL_CACHE_ROOT,
        StoreConfig::default(),
    )
    .expect("store")
}

// ---------------------------------------------------------------------------
// Structural fixtures: family shapes for corpus-shaped strings
// ---------------------------------------------------------------------------

#[test]
fn submobject_counts_are_locked_for_corpus_shapes() {
    let e = engine();
    // (source, glyphs, rules, drawn paths) — the family shape a consumer
    // builds VMobjects from. Locked: a drift is a layout change to
    // adjudicate.
    for (src, glyphs, rules, paths) in [
        ("=", 1, 0, 0),
        (r"e^{i\pi} + 1 = 0", 7, 0, 0),
        (r"\frac{d}{dx}", 3, 1, 0), // the fraction bar is a rule
        (r"\sqrt{x}", 2, 1, 0),     // surd glyph + overbar rule
        (r"\int_0^\infty e^{-x^2}\,dx", 9, 0, 0),
        // Matrix parens exceed the scale ceiling: cells as glyphs, two
        // drawn parens.
        (r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", 4, 0, 2),
        (r"\overbrace{x+y}", 3, 0, 1), // three glyphs + the drawn band
    ] {
        let t = e.typeset(Mode::Math(Style::Display), src).unwrap();
        assert_eq!(
            (
                t.layout.glyphs.len(),
                t.layout.rules.len(),
                t.layout.paths.len()
            ),
            (glyphs, rules, paths),
            "family shape drifted for {src:?}"
        );
        assert_eq!(
            t.subs.len(),
            glyphs + rules + paths,
            "submobject table covers every primitive for {src:?}"
        );
    }
}

#[test]
fn spans_select_submobjects_by_source_identity() {
    let e = engine();
    let t = e
        .typeset(Mode::Math(Style::Display), r"e^{i\pi} + 1 = 0")
        .unwrap();
    // t2c("\pi"): exactly one occurrence, exactly one glyph submobject.
    let pi = t.occurrences(r"\pi");
    assert_eq!(pi.len(), 1);
    assert_eq!(pi[0].len(), 1);
    // The ordinal indexes a glyph whose char is π.
    let ord = pi[0][0];
    match t.subs[ord].prim {
        fmn_tex::Prim::Glyph(i) => assert_eq!(t.layout.glyphs[i].ch, 'π'),
        other => panic!("expected a glyph, got {other:?}"),
    }
    // "i": the exponent's i only — never the i inside \pi. Byte-level
    // occurrences include \pi's interior, but selection by containment
    // yields nothing there (the π glyph's span is the whole command).
    let eyes = t.occurrences("i");
    let selected: usize = eyes.iter().map(Vec::len).sum();
    assert_eq!(selected, 1);
    // isolate= over a compound substring picks up its whole family.
    let one = t.occurrences("1");
    assert_eq!(one.len(), 1);
}

/// The primitive kinds a needle selects, flattened over its occurrences.
fn kinds(t: &Typeset, needle: &str) -> Vec<&'static str> {
    t.occurrences(needle)
        .into_iter()
        .flatten()
        .map(|ord| match t.subs[ord].prim {
            fmn_tex::Prim::Glyph(_) => "glyph",
            fmn_tex::Prim::Rule(_) => "rule",
            fmn_tex::Prim::Path(_) => "path",
        })
        .collect()
}

#[test]
fn a_command_keyword_selects_the_ink_it_draws() {
    // fm-5wq.22: the Reference selects a fraction bar by `\over`/`\frac`
    // and a radical by `\sqrt`; argument ink keeps its own spans.
    let e = engine();
    let math = |source: &str| e.typeset(Mode::Math(Style::Display), source).unwrap();

    let over = math(r"a \over b");
    assert_eq!(kinds(&over, r"\over"), ["rule"]);
    assert_eq!(kinds(&over, "a"), ["glyph"]);

    let frac = math(r"\frac{a}{b}");
    assert_eq!(kinds(&frac, r"\frac"), ["rule"]);
    assert_eq!(
        kinds(&frac, r"\frac{a}{b}").len(),
        3,
        "the whole construct still selects all"
    );

    let root = math(r"\sqrt{x}");
    let radical = kinds(&root, r"\sqrt");
    assert_eq!(radical.len(), 2, "radical sign and vinculum: {radical:?}");
    assert!(radical.contains(&"rule"));
    assert_eq!(kinds(&root, "x"), ["glyph"]);

    // Nesting: each keyword owns only its own construct's ink.
    let nested = math(r"\frac{\sqrt{x}}{2}");
    assert_eq!(kinds(&nested, r"\frac"), ["rule"]);
    assert_eq!(kinds(&nested, r"\sqrt").len(), 2);
    // An infix \over whose group opens with another command.
    let grouped = math(r"{\sqrt{x} \over 2}");
    assert_eq!(kinds(&grouped, r"\over"), ["rule"]);
    assert_eq!(kinds(&grouped, r"\sqrt").len(), 2);

    // Keyword boundaries: \over never matches \overline's rule.
    let line = math(r"\overline{AB}");
    assert!(kinds(&line, r"\over").is_empty());
    assert_eq!(kinds(&line, r"\overline"), ["rule"]);

    let hat = math(r"\hat x");
    assert_eq!(kinds(&hat, r"\hat").len(), 1);
    assert_eq!(kinds(&hat, "x"), ["glyph"]);

    // Text mode islands follow the same rule.
    let text = e.typeset(Mode::Text, r"so $a \over b$ here").unwrap();
    assert_eq!(kinds(&text, r"\over"), ["rule"]);
}

#[test]
fn textext_mode_typesets_the_mainland_contract() {
    let e = engine();
    let t = e
        .typeset(Mode::Text, r"the area $\pi r^2$ of a \textbf{circle}")
        .unwrap();
    assert!(!t.layout.glyphs.is_empty());
    // The math island's π is selectable by source identity.
    let pi = t.occurrences(r"\pi");
    assert_eq!(pi.len(), 1);
    assert!(!pi[0].is_empty());
}

// ---------------------------------------------------------------------------
// The cache: round trip, bit-identity, key sensitivity
// ---------------------------------------------------------------------------

#[test]
fn cache_round_trip_is_bit_identical_and_skips_relayout() {
    let store = store();
    let e = engine().with_cache(&store).unwrap();
    let src = r"\left(\frac{a}{x}\right) + \begin{pmatrix} 1 & 0 \\ 0 & 1 \end{pmatrix}";

    let cold = e.typeset(Mode::Math(Style::Display), src).unwrap();
    let warm = e.typeset(Mode::Math(Style::Display), src).unwrap();
    // A hit is definitionally equivalent: bit-for-bit equal layout,
    // including every f64 and every span.
    assert_eq!(cold, warm);

    // The payload really came from the cache: the namespace holds the key.
    let key = e
        .cache_key(Mode::Math(Style::Display), src)
        .expect("small source has a canonical cache key");
    let ns = store
        .namespace(
            "typeset",
            fmn_tex::TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    let bytes = ns.get(&key).unwrap().expect("cached entry");
    assert_eq!(Typeset::from_bytes(&bytes).unwrap(), cold);
}

#[test]
fn the_codec_round_trips_exactly() {
    let e = engine();
    for src in [
        "=",
        r"e^{i\pi} + 1 = 0",
        r"\sqrt[3]{x+1}",
        r"\left\{ \frac{1}{2} \right\}",
        r"\begin{cases} x & x > 0 \\ -x & x \le 0 \end{cases}",
        r"\widehat{abc}",
    ] {
        let t = e.typeset(Mode::Math(Style::Display), src).unwrap();
        let back = Typeset::from_bytes(&t.to_bytes().unwrap()).expect(src);
        assert_eq!(t, back, "codec drift for {src:?}");
    }
}

#[test]
fn the_codec_preserves_the_v2_bytes_for_valid_values() {
    let t = Typeset::new("x".to_owned(), fmn_tex::Layout::default()).unwrap();
    let mut expected = MAGIC_V2.to_vec();
    expected.extend_from_slice(&1_u32.to_le_bytes());
    expected.push(b'x');
    // Then width, height, depth and the glyph, rule, path, emission-order
    // and submobject counts, all zero.
    expected.resize(57, 0);

    assert_eq!(t.to_bytes().unwrap(), expected);
}

#[test]
fn the_codec_refuses_a_noncanonical_public_submobject_table() {
    let e = engine();
    let mut t = e.typeset(Mode::Math(Style::Display), "x").unwrap();
    assert!(!t.subs.is_empty());
    t.subs[0].prim = fmn_tex::Prim::Glyph(usize::MAX);

    assert!(matches!(
        t.to_bytes(),
        Err(TypesetError::NonCanonical {
            field: "submobject table",
            ..
        })
    ));
}

/// fm-aia1: submobjects follow TeX's emission order, as the Reference's do
/// (measured on 6199a00d): `G` a glyph, `R` a rule, `P` a drawn path.
#[test]
fn submobjects_follow_tex_emission_order() {
    let e = engine();
    let kinds = |src: &str| -> String {
        let t = e.typeset(Mode::Math(Style::Display), src).unwrap();
        t.subs
            .iter()
            .map(|sub| match sub.prim {
                fmn_tex::Prim::Glyph(_) => 'G',
                fmn_tex::Prim::Rule(_) => 'R',
                fmn_tex::Prim::Path(_) => 'P',
            })
            .collect()
    };
    // d, =, s, the fraction bar, the surd (a glyph or a drawn path), its
    // bar, then the radicand.
    let fraction = kinds(r"d = \frac{s}{\sqrt{\sigma_1^2 + \sigma_2^2}}");
    assert_eq!(&fraction[..4], "GGGR", "{fraction}");
    assert!(matches!(&fraction[4..6], "GR" | "PR"), "{fraction}");
    assert!(fraction[6..].chars().all(|c| c == 'G'), "{fraction}");
    // Upper limit, operator, lower limit: the sum sign is the second
    // submobject, after N.
    let t = e
        .typeset(Mode::Math(Style::Display), r"\sum_{n=1}^{N} n")
        .unwrap();
    let chars: Vec<char> = t
        .subs
        .iter()
        .map(|sub| match sub.prim {
            fmn_tex::Prim::Glyph(g) => t.layout.glyphs[g].ch,
            _ => '?',
        })
        .collect();
    assert_eq!(chars.iter().collect::<String>(), "N∑n=1n");
}

#[test]
fn the_codec_refuses_a_broken_emission_order() {
    let e = engine();
    let t = e
        .typeset(Mode::Math(Style::Display), r"\frac{1}{2}")
        .unwrap();
    // A repeated primitive is refused on encode.
    let mut repeated = t.clone();
    repeated.layout.order[1] = repeated.layout.order[0];
    assert!(matches!(
        repeated.to_bytes(),
        Err(TypesetError::NonCanonical {
            field: "emission order",
            ..
        })
    ));
    // An undefined primitive tag is refused on decode. The order table
    // is the last section before the four-byte submobject count.
    let mut bytes = t.to_bytes().unwrap();
    let entries = t.layout.order.len();
    let first_tag = bytes.len() - 4 - 5 * entries;
    bytes[first_tag] = 7;
    assert!(matches!(
        Typeset::from_bytes(&bytes),
        Err(TypesetError::NonCanonical {
            field: "emission order",
            ..
        })
    ));
}

#[test]
fn the_codec_refuses_spans_that_do_not_fit_its_wire_width() {
    let e = engine();
    let mut t = e.typeset(Mode::Math(Style::Display), "x").unwrap();
    let wide = fmn_tex::Span::new(usize::MAX, usize::MAX);
    t.layout.glyphs[0].span = wide;
    t.subs[0].span = wide;

    assert!(matches!(
        t.to_bytes(),
        Err(TypesetError::IntegerOutOfRange {
            field: "glyph span start",
            value: usize::MAX,
        })
    ));
}

#[test]
fn corrupt_cache_payloads_decode_to_none_never_panic() {
    let e = engine();
    let t = e
        .typeset(Mode::Math(Style::Display), r"\frac{a}{b}")
        .unwrap();
    let good = t.to_bytes().unwrap();
    // Truncations at every prefix; a flipped tag byte; garbage.
    for cut in 0..good.len().min(200) {
        let _ = Typeset::from_bytes(&good[..cut]); // must not panic
    }
    assert!(Typeset::from_bytes(b"not a typeset").is_err());
    let mut bad = good.clone();
    if let Some(b) = bad.last_mut() {
        *b ^= 0xff;
    }
    let _ = Typeset::from_bytes(&bad); // may decode or not; must not panic
}

#[test]
fn impossible_declared_counts_fail_before_collection_reservation() {
    let mut bytes = MAGIC_V2.to_vec();
    bytes.extend_from_slice(&0_u32.to_le_bytes()); // empty source
    bytes.extend_from_slice(&0_f64.to_bits().to_le_bytes());
    bytes.extend_from_slice(&0_f64.to_bits().to_le_bytes());
    bytes.extend_from_slice(&0_f64.to_bits().to_le_bytes());
    bytes.extend_from_slice(&u32::MAX.to_le_bytes());

    assert!(matches!(
        Typeset::from_bytes(&bytes),
        Err(TypesetError::ImpossibleCount {
            field: "glyphs",
            count,
            minimum_item_bytes: 44,
            remaining_bytes: 0,
        }) if count == usize::try_from(u32::MAX).unwrap()
    ));
}

#[test]
fn cache_keys_are_sensitive_to_every_semantic_input() {
    let base = engine();
    let src = r"a \minus b";
    let k = |e: &TexEngine, mode: Mode| {
        e.cache_key(mode, src)
            .expect("small source has a canonical cache key")
    };

    // Mode and style separate.
    assert_ne!(
        k(&base, Mode::Math(Style::Display)),
        k(&base, Mode::Math(Style::Text))
    );
    assert_ne!(k(&base, Mode::Math(Style::Display)), k(&base, Mode::Text));
    // Source separates.
    assert_ne!(
        base.cache_key(Mode::Text, "a"),
        base.cache_key(Mode::Text, "b")
    );

    // A pack change separates (the fingerprint folds the macro table in):
    // the same source under empty vs default pack must miss.
    let empty_pack = TexEngine::new("fmd-math/pack/empty", None).unwrap();
    assert_ne!(
        k(&base, Mode::Math(Style::Display)),
        k(&empty_pack, Mode::Math(Style::Display))
    );

    // A caller macro layered over the pack separates too.
    let mut extra = MacroSet::new();
    extra.define("half", 1, r"\frac{#1}{2}").unwrap();
    let with_extra = TexEngine::new("fmd-math/pack/default", Some(&extra)).unwrap();
    assert_ne!(
        k(&base, Mode::Math(Style::Display)),
        k(&with_extra, Mode::Math(Style::Display))
    );
    // And the layered macro actually expands.
    let t = with_extra
        .typeset(Mode::Math(Style::Display), r"\half{x}")
        .unwrap();
    assert!(t.layout.rules.len() == 1, "the macro's fraction bar");
}

#[test]
fn pack_content_ids_resolve_and_drift_is_named() {
    let e = TexEngine::new("fmd-math/pack/basic", None).unwrap();
    assert_eq!(e.pack_content_id(), "fmd-math/pack/basic");
    match TexEngine::new("fmd-math/pack/nonexistent", None) {
        Err(fmn_tex::TexError::UnknownPack { content_id }) => {
            assert_eq!(content_id, "fmd-math/pack/nonexistent");
        }
        other => panic!("expected UnknownPack, got {other:?}"),
    }
}

#[test]
fn config_template_wires_through_the_registry() {
    use fmn_config::config::Layer;
    let registry = fmn_config::PackRegistry::builtin();
    let resolved = fmn_config::Config::resolve(&[], None).unwrap();
    let e = TexEngine::from_config(&resolved.config, &registry).unwrap();
    assert_eq!(e.pack_content_id(), "fmd-math/pack/default");
    // The pack's \minus works end to end from config.
    let t = e
        .typeset(Mode::Math(Style::Display), r"a \minus b")
        .unwrap();
    assert!(t.layout.glyphs.iter().any(|g| g.ch == '−'));

    // An out-of-tier template is the registry's named refusal.
    let ctex = fmn_config::Config::resolve(
        &[Layer {
            name: "u",
            text: "tex:\n  template: \"ctex\"\n",
        }],
        None,
    )
    .unwrap();
    match TexEngine::from_config(&ctex.config, &registry) {
        Err(fmn_tex::TexError::Pack(e)) => {
            assert!(e.to_string().contains("no native preamble pack"), "{e}");
        }
        other => panic!("expected the pack refusal, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// The preflight
// ---------------------------------------------------------------------------

#[test]
fn preflight_warms_the_cache_in_parallel_and_collects_errors() {
    let store = store();
    let e = engine().with_cache(&store).unwrap();
    let items: Vec<(Mode, &str)> = vec![
        (Mode::Math(Style::Display), r"e^{i\pi} + 1 = 0"),
        (Mode::Math(Style::Display), r"\frac{d}{dx} \sin x"),
        (Mode::Math(Style::Display), r"\dx"), // still-pending tier-2: fails
        (Mode::Text, r"area $\pi r^2$"),
        (
            Mode::Math(Style::Display),
            r"\begin{pmatrix} 1 \\ 0 \end{pmatrix}",
        ),
    ];
    let outcomes = e.preflight(&items).expect("preflight result storage");
    assert_eq!(outcomes.len(), items.len());
    // Per-string outcomes, in order; the tier-2 string fails precisely
    // without aborting the batch.
    assert!(outcomes[0].is_ok());
    assert!(outcomes[1].is_ok());
    let err = outcomes[2].as_ref().unwrap_err();
    assert!(err.to_string().contains("tier T2"), "{err}");
    assert!(outcomes[3].is_ok());
    assert!(outcomes[4].is_ok());

    // The cache is warm: every successful string hits without re-layout.
    let ns = store
        .namespace(
            "typeset",
            fmn_tex::TYPESET_FORMAT_VERSION,
            NamespacePolicy::default(),
        )
        .unwrap();
    for (i, (mode, src)) in items.iter().enumerate() {
        let key = e
            .cache_key(*mode, src)
            .expect("small source has a canonical cache key");
        let cached = ns.get(&key).unwrap();
        assert_eq!(
            cached.is_some(),
            outcomes[i].is_ok(),
            "cache state mismatch for {src:?}"
        );
    }
}

// ---------------------------------------------------------------------------
// The error surface at construction time
// ---------------------------------------------------------------------------

#[test]
fn construct_errors_surface_named_and_tier_tagged() {
    let e = engine();
    // The pending tier-2 example advances as constructs graduate (fm-j5t).
    let err = e.typeset(Mode::Math(Style::Display), r"\dx").unwrap_err();
    let msg = err.to_string();
    assert!(msg.contains(r"`\dx` is not yet supported"), "{msg}");
    assert!(msg.contains("tier T2"), "{msg}");
    assert!(msg.contains("fm-j5t"), "tracked-at pointer: {msg}");

    // A genuinely malformed structure (a double superscript) is a
    // byte-positioned refusal. (`\frac{1}{` is NOT an error: the
    // SingleStringTex fragment tolerance lets end-of-input close open
    // arguments — each literal argument may be a piece of a balanced
    // whole.)
    let err = e.typeset(Mode::Math(Style::Display), r"a^b^c").unwrap_err();
    assert!(err.to_string().contains("malformed"), "{err}");
    assert!(
        e.typeset(Mode::Math(Style::Display), r"\frac{1}{").is_ok(),
        "fragment tolerance holds"
    );
}
