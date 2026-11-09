//! Complete request preflight uses the same layout, cache and span authorities.

use fmn_cache::{Store, StoreConfig};
use fmn_platform::clock::FakeClock;
use fmn_platform::fs::VirtualFs;
use fmn_tex::{LineAlign, TexEngine, TypesetRequest};
use std::num::NonZeroUsize;
use std::sync::Arc;

fn store() -> Store {
    Store::open(
        Arc::new(VirtualFs::new()),
        Arc::new(FakeClock::new()),
        if cfg!(windows) { r"C:\cache" } else { "/cache" },
        StoreConfig::default(),
    )
    .unwrap()
}

fn engine() -> TexEngine {
    TexEngine::new("fmd-math/pack/default", None).unwrap()
}

#[test]
fn preflight_warms_full_preamble_and_alignment_keys_for_fresh_consumers() {
    let store = store();
    let first = engine().with_cache(&store).unwrap();
    let mut macro_request = TypesetRequest::math(r"\answer^2");
    macro_request.preamble = r"\newcommand{\answer}{x}";
    let mut text = TypesetRequest::text(r"Area $x^2$\\Result");
    text.align = LineAlign::Right;
    let requests = [macro_request, text, TypesetRequest::inline_math("0")];
    let outcomes = first
        .preflight_requests(&requests, NonZeroUsize::new(4).unwrap())
        .unwrap();
    assert!(outcomes.iter().all(Result::is_ok), "{outcomes:?}");
    let before = first.layout_computations();
    let expected: Vec<_> = requests
        .iter()
        .map(|request| {
            first
                .typeset_aligned(
                    request.mode,
                    request.source,
                    request.preamble,
                    request.align,
                )
                .unwrap()
                .to_bytes()
                .unwrap()
        })
        .collect();
    assert_eq!(first.layout_computations(), before);
    let second = engine().with_cache(&store).unwrap();
    for (request, expected) in requests.iter().zip(expected) {
        let actual = second
            .typeset_aligned(
                request.mode,
                request.source,
                request.preamble,
                request.align,
            )
            .unwrap();
        assert_eq!(actual.to_bytes().unwrap(), expected);
        assert_eq!(actual.source, request.source);
        assert!(
            actual
                .subs
                .iter()
                .all(|sub| sub.span.end <= request.source.len())
        );
    }
    assert_eq!(second.layout_computations(), 0);
    assert!(second.persistent_cache_hits() >= 3);
}

#[test]
fn errors_are_ordered_and_do_not_prevent_later_valid_layouts() {
    let engine = engine();
    let requests = [
        TypesetRequest::math("x+1"),
        TypesetRequest::math(r"\fmnUnknownConstruct"),
        TypesetRequest::text("valid"),
    ];
    let outcomes = engine
        .preflight_requests(&requests, NonZeroUsize::new(3).unwrap())
        .unwrap();
    assert_eq!(outcomes.len(), 3);
    assert!(outcomes[0].is_ok());
    assert!(outcomes[1].is_err());
    assert!(outcomes[2].is_ok());
    let before = engine.layout_computations();
    engine
        .typeset_aligned(requests[2].mode, "valid", "", requests[2].align)
        .unwrap();
    assert_eq!(engine.layout_computations(), before);
}

#[test]
fn worker_limit_and_empty_batches_do_not_change_layout_results() {
    let requests = [
        TypesetRequest::math(r"\frac{1}{2}"),
        TypesetRequest::inline_math(r"\frac{1}{2}"),
        TypesetRequest::text(r"area $\pi r^2$"),
    ];
    let mut results = Vec::new();
    for workers in [1, 4] {
        let engine = engine();
        assert!(
            engine
                .preflight_requests(&[], NonZeroUsize::new(workers).unwrap())
                .unwrap()
                .is_empty()
        );
        assert_eq!(engine.layout_computations(), 0);
        assert!(
            engine
                .preflight_requests(&requests, NonZeroUsize::new(workers).unwrap())
                .unwrap()
                .iter()
                .all(Result::is_ok)
        );
        results.push(
            requests
                .iter()
                .map(|request| {
                    engine
                        .typeset_aligned(
                            request.mode,
                            request.source,
                            request.preamble,
                            request.align,
                        )
                        .unwrap()
                        .to_bytes()
                        .unwrap()
                })
                .collect::<Vec<_>>(),
        );
    }
    assert_eq!(results[0], results[1]);
    assert_ne!(results[0][0], results[0][1]);
}
