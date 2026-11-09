//! Authored variable paint keeps its geometry dependency even with no stations.
use fmn_mobject::{Mob, Mobject, Placement, RecordBuffer, RecordSchema, Stage};
use fmn_render::{RenderPlan, Style};

const POINTS: [[f32; 3]; 3] = [[0.0; 3], [1.0, 2.0, 0.0], [2.0, 0.0, 0.0]];

fn fixture(field: &str) -> (Stage, Mob) {
    let mut buffer = RecordBuffer::new(RecordSchema::vmobject(), 3).unwrap();
    for i in 0..3 {
        buffer.write(i, "point", &[0.0; 3]);
        buffer.write(i, "fill_rgba", &[1.0, 0.0, 0.0, 1.0]);
        buffer.write(i, "stroke_rgba", &[1.0, 0.0, 0.0, 1.0]);
        buffer.write(i, "stroke_width", &[2.0]);
    }
    if field == "stroke_width" {
        buffer.write(1, field, &[8.0]);
    } else {
        buffer.write(1, field, &[0.0, 0.0, 1.0, 0.25]);
    }
    let mut stage = Stage::new();
    let mob = stage.add(Mobject::from_buffer(buffer));
    stage.add_to_scene(mob).unwrap();
    (stage, mob)
}

fn style(plan: &RenderPlan) -> &Style {
    plan.styles()
        .get(plan.shapes().instances()[0].style)
        .unwrap()
}

fn geometry(stage: &mut Stage, mob: Mob, scale: f32) {
    for (i, point) in POINTS.iter().enumerate() {
        stage
            .get_mut(mob)
            .unwrap()
            .buffer
            .write(i, "point", &point.map(|x| x * scale));
    }
}

fn no_profile(plan: &RenderPlan) {
    assert!(style(plan).fill_profile.is_none());
    assert!(style(plan).stroke_profile.is_none());
}

#[test]
fn initially_collapsed_record_profiles_recover_without_making_flat_paint_geometry_dependent() {
    for field in ["fill_rgba", "stroke_rgba", "stroke_width"] {
        let (mut stage, mob) = fixture(field);
        let mut plan = RenderPlan::default();
        plan.sync(&stage, 0).unwrap();
        no_profile(&plan);
        assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 0);

        for scale in [1.0, 0.0, 2.0, 0.0, 1.0] {
            geometry(&mut stage, mob, scale);
            assert_eq!(
                plan.sync(&stage, 0).unwrap().styles_rebuilt,
                1,
                "lost dependency for {field}"
            );
            if scale == 0.0 {
                no_profile(&plan);
            } else {
                assert_eq!(style(&plan).fill_profile.is_some(), field == "fill_rgba");
                assert_eq!(style(&plan).stroke_profile.is_some(), field != "fill_rgba");
                let mut fresh = RenderPlan::default();
                fresh.sync(&stage, 0).unwrap();
                assert_eq!(style(&plan), style(&fresh));
            }
            assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 0);
        }

        // Recoloring to genuinely uniform paint must drop this dependency.
        // Retaining it forever would fix recovery but break retained fast paths.
        let buffer = &mut stage.get_mut(mob).unwrap().buffer;
        for i in 0..3 {
            buffer.write(i, "fill_rgba", &[1.0, 0.0, 0.0, 1.0]);
            buffer.write(i, "stroke_rgba", &[1.0, 0.0, 0.0, 1.0]);
            buffer.write(i, "stroke_width", &[2.0]);
        }
        assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 1);
        no_profile(&plan);
        geometry(&mut stage, mob, 3.0);
        assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 0);
    }
}

#[test]
fn singular_placement_can_recover_profiles_without_touching_point_records() {
    for field in ["fill_rgba", "stroke_rgba", "stroke_width"] {
        let (mut stage, mob) = fixture(field);
        geometry(&mut stage, mob, 1.0);
        let revision = stage.get(mob).unwrap().buffer.field_revision("point");
        stage
            .set_placement(mob, Placement::new([[0.0; 3]; 3], [0.0; 3]))
            .unwrap();
        let mut plan = RenderPlan::default();
        plan.sync(&stage, 0).unwrap();
        no_profile(&plan);
        assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 0);
        stage.set_placement(mob, Placement::IDENTITY).unwrap();
        assert_eq!(plan.sync(&stage, 0).unwrap().styles_rebuilt, 1);
        assert_eq!(
            stage.get(mob).unwrap().buffer.field_revision("point"),
            revision
        );
        assert_eq!(style(&plan).fill_profile.is_some(), field == "fill_rgba");
        assert_eq!(style(&plan).stroke_profile.is_some(), field != "fill_rgba");
        let mut fresh = RenderPlan::default();
        fresh.sync(&stage, 0).unwrap();
        assert_eq!(style(&plan), style(&fresh));
    }
}
