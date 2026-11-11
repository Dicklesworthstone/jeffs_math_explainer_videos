"""Write the OPTIONAL scratch candidate03 (candidate02 failed rustfmt --check on Ok(...?)) of the_bump.rs from root's candidate01 bytes (read-only input).

candidate03 = candidate01 (the one-line `.buff(0.0)`) + the arrow moved into `preview_arrow(&lp)`,
the loop radius shared by the circle and the arrow, and one native test that reads the built
arrow's outline. The picture is meant to be byte-for-byte the same mobject as candidate01: same
builder calls, same arguments, same push order (7 mobs), no animation call touched.

usage: python3 -I -B make-candidate02.py CANDIDATE01.rs OUT.rs   (refuses to overwrite OUT)
"""
import hashlib
import sys

src_path, out_path = sys.argv[1], sys.argv[2]
raw = open(src_path, "rb").read()
assert hashlib.sha256(raw).hexdigest() == "c52c0472196cbc68f3fca2b47fc2ad7d6cc9c3ba6e5ee8f356b882e4ffc27564", "not root's candidate01"
src = raw.decode("utf-8")

EDITS = [
    (
        """    out.push(lp.circle(stage, ZERO, 0.8, hero(), 4.0)?);
    let arrow = Arrow::new(lp.p(ZERO), lp.p(C::polar(0.8, 0.9)))
        .buff(0.0)
        .thickness(3.0)
        .color(Palette::ink())
        .build()?;
    out.push(stage.arena_mut().add(arrow));
""",
        """    out.push(lp.circle(stage, ZERO, PREVIEW_LOOP_R, hero(), 4.0)?);
    let arrow = preview_arrow(&lp)?;
    out.push(stage.arena_mut().add(arrow));
""",
    ),
    (
        """    out.insert(0, caption);
    Ok(out)
}

#[cfg(test)]
mod tests {
""",
        """    out.insert(0, caption);
    Ok(out)
}

/// The loop preview's radius in its plane, and the angle where its arrow from zero meets it.
const PREVIEW_LOOP_R: f64 = 0.8;
const PREVIEW_ARROW_ARG: f64 = 0.9;

/// The loop preview's arrow, from zero to the loop. `Arrow::new` alone trims `MED_SMALL_BUFF`
/// (0.25) off each end, and this arrow is only 0.5 long: all that was left was a loose head.
fn preview_arrow(lp: &Plane) -> fmn::Result<VMobject> {
    let arrow = Arrow::new(lp.p(ZERO), lp.p(C::polar(PREVIEW_LOOP_R, PREVIEW_ARROW_ARG)))
        .buff(0.0)
        .thickness(3.0)
        .color(Palette::ink())
        .build()?;
    Ok(arrow)
}

#[cfg(test)]
mod tests {
""",
    ),
    (
        """        let plane = Plane::new(v(g.center[0], 0.0), g.width, (-3.0, 3.0), (-1.9, 2.1));
        assert!((plane.unit() - g.width / 6.0).abs() < 1e-12);
    }
}
""",
        """        let plane = Plane::new(v(g.center[0], 0.0), g.width, (-3.0, 3.0), (-1.9, 2.1));
        assert!((plane.unit() - g.width / 6.0).abs() < 1e-12);
    }

    /// The 01_6 loop preview's arrow runs from zero to its point on the loop, head at the loop.
    /// Read from the real arrow's outline, as the turning chapter reads its arrows.
    #[test]
    fn the_preview_arrow_reaches_the_loop_head_first() {
        // the window previews() draws the loop in
        let lp = Plane::new(v(5.6, 0.0), 1.5, (-1.2, 1.2), (-1.2, 1.2));
        let (pa, pb) = (lp.p(ZERO), lp.p(C::polar(PREVIEW_LOOP_R, PREVIEW_ARROW_ARG)));
        let (dx, dy) = (pb[0] - pa[0], pb[1] - pa[1]);
        let len = (dx * dx + dy * dy).sqrt();
        assert!((len - PREVIEW_LOOP_R * lp.unit()).abs() < 1e-12);
        let m = preview_arrow(&lp).unwrap();
        // (along the arrow from zero, across it) for every point of the outline
        let pts: Vec<(f64, f64)> = m
            .points()
            .iter()
            .map(|q| {
                let (qx, qy) = (q[0] - pa[0], q[1] - pa[1]);
                ((qx * dx + qy * dy) / len, (qy * dx - qx * dy) / len)
            })
            .collect();
        let lo = pts.iter().map(|p| p.0).fold(f64::MAX, f64::min);
        let hi = pts.iter().map(|p| p.0).fold(f64::MIN, f64::max);
        assert!(lo.abs() < 1e-6 && (hi - len).abs() < 1e-6, "{lo} to {hi} of {len}");
        // the farthest point is the head's one point, on the loop, not the corners of a shaft
        for &(along, across) in &pts {
            if along > hi - 1e-6 {
                assert!(across.abs() < 1e-6, "{along}, {across}");
            }
        }
    }
}
""",
    ),
]

for old, new in EDITS:
    assert src.count(old) == 1, old[:60]
    src = src.replace(old, new)
with open(out_path, "x", encoding="utf-8") as f:
    f.write(src)
print("wrote", out_path, hashlib.sha256(src.encode("utf-8")).hexdigest())
