#!/usr/bin/env python3
"""Preserve concurrent profile recovery while keeping steady frames O(1)."""
from pathlib import Path
name = Path('crates/fmn-render/src/plan.rs')
text = name.read_text()
if 'let (style, profiled) = match &previous' in text:
    raise SystemExit('profile dependency refinement already integrated')

def replace(old, new):
    global text
    if text.count(old) != 1:
        raise SystemExit('changed retained-plan source anchor: ' + old[:100])
    text = text.replace(old, new, 1)

replace('''            let previous_profiled = previous.is_some_and(|old| {
                self.styles
                    .get(old.style)
                    .or_else(|| {
                        pending_styles
                            .iter()
                            .find(|(i, _)| *i == old.style)
                            .map(|(_, s)| s)
                    })
                    .is_some_and(|style| {
                        style.stroke_profile.is_some() || style.fill_profile.is_some()
                    })
            });
            let has_varying_paint = crate::stroke_profile::has_varying_paint(stage, mob)
                || crate::fill_profile::has_varying_paint(stage, mob);
            let style_unsafe =
                style_unsafe || ((previous_profiled || has_varying_paint) && hint_unsafe);
            let style = match &previous {
                Some(retained) if !style_unsafe && !retained.style_dep.is_stale(&now) => {
                    retained.style
''', '''            // The dependency describes authored paint, not just a currently
            // measurable profile. Collapse removes stations, not dependency on
            // geometry. Reuse this per-object metadata without reading paint on
            // unchanged frames, including when style rows are shared.
            let previous_profiled = previous
                .is_some_and(|old| old.style_dep.axes().contains(&Axis::Geometry));
            let style_unsafe = style_unsafe || (previous_profiled && hint_unsafe);
            let (style, profiled) = match &previous {
                Some(retained) if !style_unsafe && !retained.style_dep.is_stale(&now) => {
                    (retained.style, previous_profiled)
''')
replace('''                    stats.styles_rebuilt += 1;
                    let mut row = read_style(stage, mob);
''', '''                    stats.styles_rebuilt += 1;
                    let has_varying_paint = crate::stroke_profile::has_varying_paint(stage, mob)
                        || crate::fill_profile::has_varying_paint(stage, mob);
                    let mut row = read_style(stage, mob);
''')
replace('''                    let key = row.bits();
                    if let Some(index) = self.styles.index_of(&row) {
''', '''                    let key = row.bits();
                    let style = if let Some(index) = self.styles.index_of(&row) {
''')
replace('''                        pending_styles.push((index, row));
                        index
                    }
                }
            };
''', '''                        pending_styles.push((index, row));
                        index
                    };
                    (style, has_varying_paint)
                }
            };
''')
replace('''            let profiled = self
                .styles
                .get(style)
                .or_else(|| {
                    pending_styles
                        .iter()
                        .find(|(i, _)| *i == style)
                        .map(|(_, s)| s)
                })
                .is_some_and(|row| row.stroke_profile.is_some() || row.fill_profile.is_some());
            let profile_dependent = profiled || has_varying_paint;
''', '')
replace('''                        if profile_dependent {
''', '''                        if profiled {
''')
name.write_text(text)
