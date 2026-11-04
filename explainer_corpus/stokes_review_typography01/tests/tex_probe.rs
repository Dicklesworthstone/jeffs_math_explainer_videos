//! Typesetting support probe only; it does not review the rendered pictures.
use fmn::prelude::Tex;
use fmn::tex::TexEngine;

#[test]
fn stokes_proof_formulas_typeset_with_the_native_engine() {
    let engine = TexEngine::new("fmd-math/pack/default", None).expect("bundled math engine");
    for source in [
        r"a_i,\quad b_i",
        r"\partial_y f_2\,dy",
        r"-\partial_y f_2\,dy\wedge dx\wedge dz=+\partial_y f_2\,dx\wedge dy\wedge dz",
        r"d\omega=(\sum_i\partial_i f_i)\,dx^1\wedge\cdots\wedge dx^n",
        r"\int_{a_i}^{b_i}\partial_i f_i\,dx^i=f_i(b_i)-f_i(a_i)",
    ] {
        Tex::new(source)
            .font_size(27.0)
            .build(&engine)
            .unwrap_or_else(|error| panic!("formula {source:?}: {error}"));
    }
}
