"""Native preflight acceptance, embedded by portal_typesetting.rs.

Run this file with the installed wheel to exercise the real production engines.
The host-only argument tests are separate; this suite never substitutes a
layout engine or renderer.
"""
import numpy as np


def run_typeset_preflight(m):
    configure = m._fmn_configure_tex_cache
    preflight = m._preflight_tex
    info = m._fmn_tex_cache_info
    sources = [rf"x^{{{index}}} + {index}" for index in range(1, 21)]
    try:
        configure(enabled=False)
        expected = [m.Tex(source).get_all_points().copy() for source in sources]
        assert all(points.size for points in expected)
        for workers in (1, 4):
            configure(enabled=False)
            report = preflight(sources, max_workers=workers)
            assert report["schema"] == "fmn-python.typeset-preflight", report
            assert (report["count"], report["succeeded"], report["failed"]) == (20, 20, 0), report
            assert [row["index"] for row in report["items"]] == list(range(20)), report
            assert all(row["ok"] and row["error"] is None for row in report["items"]), report
            assert report["worker_limit"] == workers
            warm = report["after"]["layout_computations"]
            assert warm > report["before"]["layout_computations"], report
            for source, points in zip(sources, expected):
                np.testing.assert_array_equal(m.Tex(source).get_all_points(), points)
            after = info()["templates"]["default"]
            assert after["layout_computations"] == warm, (report, after)
            assert after["memory_hits"] > report["after"]["memory_hits"], (report, after)

        # A failed source keeps its input index and cannot abort later work.
        configure(enabled=False)
        report = preflight([r"a^2", r"\fmnUnknownPreflightCommand{q}", r"b^3"])
        assert [row["ok"] for row in report["items"]] == [True, False, True], report
        assert (report["succeeded"], report["failed"]) == (2, 1), report
        assert 0 < len(report["items"][1]["error"]) <= 2048
        layouts = report["after"]["layout_computations"]
        m.Tex(r"b^3")
        assert info()["templates"]["default"]["layout_computations"] == layouts

        # Text/mainland alignment is a distinct cache key, not display math.
        configure(enabled=False)
        source = r"wide words\\$x$"
        expected_text = m.TexText(source).get_all_points().copy()
        configure(enabled=False)
        report = preflight([source], text_mode=True)
        assert report["succeeded"] == 1, report
        np.testing.assert_array_equal(m.TexText(source).get_all_points(), expected_text)
        assert info()["templates"]["default"]["layout_computations"] == report["after"]["layout_computations"]

        # Preambles are request-local and must reach the real constructor key.
        declarations = r"\newcommand{\preflightword}{x^2}"
        source = r"\preflightword + 1"
        configure(enabled=False)
        expected_macro = m.Tex(source, additional_preamble=declarations).get_all_points().copy()
        configure(enabled=False)
        report = preflight([source], preamble=declarations)
        assert report["succeeded"] == 1, report
        # Constructor preamble admission may typeset an empty validation probe;
        # compare geometry here rather than hiding that work in a cache claim.
        np.testing.assert_array_equal(
            m.Tex(source, additional_preamble=declarations).get_all_points(), expected_macro,
        )
        assert preflight([source])["failed"] == 1, "preamble leaked into the engine"

        # Bad batch shape/admission fails before warming even its valid prefix.
        for bad_sources, options, exception in (
            (["fresh source", object()], {}, TypeError),
            (["x"] * 4097, {}, ValueError),
            (["x" * 262_145], {}, ValueError),
            (["x"], {"max_workers": 0}, ValueError),
            (["x"], {"max_workers": 65}, ValueError),
            (["x"], {"alignment": "center"}, ValueError),
        ):
            before = info()["templates"]["default"]["layout_computations"]
            try:
                preflight(bad_sources, **options)
            except exception:
                pass
            else:
                raise AssertionError("invalid native preflight input was accepted")
            assert info()["templates"]["default"]["layout_computations"] == before
        empty = preflight([])
        assert empty["items"] == [] and empty["count"] == 0
        assert empty["before"] == empty["after"], empty
    finally:
        configure(enabled=False)


if __name__ in ("__main__", "<run_path>"):
    import manimlib as m
    run_typeset_preflight(m)
    print("native preflight: geometry, constructor cache reuse, ordering and refusals passed")
