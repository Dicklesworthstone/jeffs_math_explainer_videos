"""string-tests01.py: the string assertions of candidate03's new source-reading tests, run on the file.

Mirrors only the include_str! parts of the_shrinking_names_start_where_the_shrink_puts_them and
the_growing_disks_pass_under_the_cards_and_round_a, which need no compile: every slice marker
exists where the test expects it, and each wanted line occurs exactly once in its slice. As in
Rust, include_str! sees the whole file, so a marker quoted inside the test module also counts.

usage: python3 -I -B string-tests01.py BOUNDARY_RS
"""
import sys

source = open(sys.argv[1], encoding="utf-8").read()
fails = 0


def check(name, ok):
    global fails
    fails += not ok
    print(("PASS " if ok else "FAIL ") + name)


parts = source.split("// Second limit:")
check("'// Second limit:' present (nth(1) exists)", len(parts) >= 2)
shrink = parts[1].split("// A finite picture")[0]
film_at = source.index("\n#[cfg(test)]\nmod tests {")
check("the shrink slice ends inside the film part", source.index(shrink) + len(shrink) < film_at)
for wanted in [
    "let [a_from, rho_from] = shrink_names(SMALL_R);",
    "fade_out_all(stage, &leaving, 0.2)?;",
    "fade_in_all(stage, &[bar, bar_name, ca_out, rho_out, leader], 0.3)?;",
    "let [a_at, rho_at] = shrink_names(r);",
]:
    check("shrink slice has once: " + wanted, shrink.count(wanted) == 1)
check("shrink slice lacks 'r + 0.06'", "r + 0.06" not in shrink)

film = source.split("\n#[cfg(test)]\nmod tests {")[0]
check("two growth opacity calls", film.count("a.set_opacity(a_name, a_name_shown(r), true);") == 2)
for wanted in [
    "let disk = stage.add_value_tracker(1e-3);",
    ".map(|k| p.p(C::polar(r, TAU * f64::from(k) / 120.0)))",
    "stage.play(tween_linear(disk, fact_f64(\"bump.radius0\")?, rt)?)?;",
    "a.move_to(a_name, p.p(c + A_NAME_FROM_CENTRE), ORIGIN);",
    "stage.play(tween_linear(slide, 1.0, 1.8)?)?;",
    "let unit = p.circle(stage, C::real(1.0), 1e-3, hero(), 5.0)?;",
    "let grow = stage.add_value_tracker(1e-3);",
    ".map(|k| p.p(C::real(1.0) + C::polar(r, TAU * f64::from(k) / 160.0)))",
    "let radius = fact_f64(\"recentre1.radius\")?;",
    "stage.play(tween_linear(grow, radius, rt)?)?;",
]:
    check("film has once: " + wanted, film.count(wanted) == 1)
disk_in = film.find("fade_in_all(stage, &[unit, reach1, reach_name[0], reach_name[1]], 0.3)?;")
card_up = film.find("for m in one_name {\n            stage.add_to_scene(m)?;")
growth = film.find("let grow = stage.add_value_tracker(1e-3);")
check("09_9 order: disk fade-in < card re-root < growth tracker", -1 < disk_in < card_up < growth)
print("%d failed" % fails)
sys.exit(1 if fails else 0)
