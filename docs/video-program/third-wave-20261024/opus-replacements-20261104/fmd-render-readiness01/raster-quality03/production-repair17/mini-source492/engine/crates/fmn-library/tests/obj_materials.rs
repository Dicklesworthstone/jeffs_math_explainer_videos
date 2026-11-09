//! Independent metadata, geometry and material binding oracles.
use std::collections::BTreeMap;

use fmn_core::color::Srgb;
use fmn_library::obj_materials::{ObjAssetLimits, ObjDocument, ObjMaterial, parse_mtl};
use fmn_mobject::{ImageColorSpace, ImageResource, ImageSampler, RenderPrimitive};

const GEOMETRY: &str =
    "v -2 -1 0\nv 0 -1 0\nv 0 1 0\nv 2 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nvn 0 0 4\n";
fn image() -> ImageResource {
    ImageResource::rgba8(
        2,
        1,
        vec![255, 0, 0, 255, 0, 0, 255, 128],
        ImageColorSpace::Srgb,
        ImageSampler::default(),
    )
    .unwrap()
}
fn material() -> ObjMaterial {
    ObjMaterial {
        diffuse: Srgb {
            r: 0.2,
            g: 0.5,
            b: 0.7,
        },
        opacity: 0.6,
        texture: None,
    }
}

#[test]
fn geometry_authority_and_contiguous_painter_order_are_preserved() {
    let source = format!(
        "mtllib one.mtl two.mtl\n{GEOMETRY}usemtl A\nf 1/1/1 2/2/1 3/3/1\nusemtl B\nf 2/1 4/2 3/3\nusemtl A\nf -4/-4 -3/-3 -2/-2\n"
    );
    let doc = ObjDocument::parse(source.as_bytes()).unwrap();
    assert_eq!(
        doc.mesh(),
        &fmn_library::ObjMesh::parse(source.as_bytes()).unwrap()
    );
    assert_eq!(doc.libraries(), &["one.mtl", "two.mtl"]);
    assert_eq!(
        doc.runs()
            .iter()
            .map(|run| run.material.as_deref())
            .collect::<Vec<_>>(),
        vec![Some("A"), Some("B"), Some("A")]
    );
    assert_eq!(doc.runs()[2].triangles, 2..3);
    assert_eq!(
        doc.material_names().into_iter().collect::<Vec<_>>(),
        vec!["A", "B"]
    );
}

#[test]
fn fan_triangulation_comments_and_redundant_bindings_do_not_shift_materials() {
    let source = format!(
        "{GEOMETRY}usemtl A\r\nf 1/1 2/2 4/3 3/4 # polygon\r\nusemtl A\nf 1 2 3\nusemtl off\nf 2 4 3\n"
    );
    let doc = ObjDocument::parse(source.as_bytes()).unwrap();
    assert_eq!(doc.runs().len(), 2);
    assert_eq!(doc.runs()[0].triangles, 0..3);
    assert_eq!(doc.runs()[1].material, None);
    assert_eq!(doc.runs()[1].triangles, 3..4);
}

#[test]
fn each_material_uses_one_shared_model_normalization() {
    let source = format!("{GEOMETRY}usemtl A\nf 1 2 3\nusemtl B\nf 2 4 3\n");
    let doc = ObjDocument::parse(source.as_bytes()).unwrap();
    let materials = BTreeMap::from([("A".into(), material()), ("B".into(), material())]);
    let model = doc.to_mobject(4.0, &materials, &BTreeMap::new()).unwrap();
    assert_eq!(model.submobjects.len(), 2);
    assert!(model.buffer.is_empty());
    let a = &model.submobjects[0];
    let b = &model.submobjects[1];
    assert_eq!(a.render_primitive, RenderPrimitive::TriangleMesh);
    assert_eq!(a.buffer.read(0, "point").unwrap(), vec![-4.0, -2.0, 0.0]);
    assert_eq!(b.buffer.read(1, "point").unwrap(), vec![4.0, 2.0, 0.0]);
    assert_eq!(a.buffer.read(1, "point"), b.buffer.read(0, "point"));
    assert_eq!(a.buffer.read(0, "rgba").unwrap(), vec![0.2, 0.5, 0.7, 0.6]);
    assert_eq!(
        a.buffer.read(0, "d_normal_point").unwrap(),
        vec![-4.0, -2.0, 1.0]
    );
    assert!(a.uniforms.depth_test);
    assert_eq!(a.uniforms.shading, fmn_library::MODEL_SHADING);
}

#[test]
fn texture_seams_preserve_corner_uvs_and_flip_v_exactly_once() {
    let source = format!("{GEOMETRY}usemtl A\nf 1/1/1 2/2/1 3/3/1\nf 1/4/1 3/3/1 4/2/1\n");
    let doc = ObjDocument::parse(source.as_bytes()).unwrap();
    let mut mat = material();
    mat.texture = Some("tiles.png".into());
    let resource = image();
    let model = doc
        .to_mobject(
            2.0,
            &BTreeMap::from([("A".into(), mat)]),
            &BTreeMap::from([("A".into(), resource.clone())]),
        )
        .unwrap();
    let child = &model.submobjects[0];
    assert_eq!(child.buffer.len(), 6);
    assert_eq!(child.image.as_ref(), Some(&resource));
    assert_eq!(child.buffer.read(0, "point"), child.buffer.read(3, "point"));
    assert_eq!(child.buffer.read(0, "im_coords").unwrap(), vec![0.0, 1.0]);
    assert_eq!(child.buffer.read(3, "im_coords").unwrap(), vec![0.0, 0.0]);
    assert_eq!(child.buffer.read(4, "im_coords").unwrap(), vec![1.0, 0.0]);
    assert_eq!(child.buffer.read(0, "opacity").unwrap(), vec![0.6]);
}

#[test]
fn undefined_material_missing_image_and_missing_uv_refuse() {
    let doc = ObjDocument::parse(format!("{GEOMETRY}usemtl A\nf 1 2 3\n").as_bytes()).unwrap();
    assert!(
        doc.to_mobject(2.0, &BTreeMap::new(), &BTreeMap::new())
            .err()
            .unwrap()
            .reason
            .contains("not defined")
    );
    let mut mat = material();
    mat.texture = Some("required.png".into());
    let materials = BTreeMap::from([("A".into(), mat)]);
    assert!(
        doc.to_mobject(2.0, &materials, &BTreeMap::new())
            .err()
            .unwrap()
            .reason
            .contains("not supplied")
    );
    assert!(
        doc.to_mobject(2.0, &materials, &BTreeMap::from([("A".into(), image())]))
            .err()
            .unwrap()
            .reason
            .contains("without UV")
    );
}

#[test]
fn untextured_models_remain_usable_without_any_library() {
    let doc = ObjDocument::parse(format!("{GEOMETRY}f 1 2 3\n").as_bytes()).unwrap();
    let model = doc
        .to_mobject(3.0, &BTreeMap::new(), &BTreeMap::new())
        .unwrap();
    assert_eq!(model.submobjects.len(), 1);
    assert!(model.submobjects[0].image.is_none());
    let plain: fmn_mobject::Mobject =
        fmn_library::ThreeDModel::from_obj(format!("{GEOMETRY}f 1 2 3\n").as_bytes())
            .unwrap()
            .into();
    let imported = &model.submobjects[0].buffer;
    assert_eq!(plain.buffer.schema(), imported.schema());
    for key in ["point", "d_normal_point", "rgba"] {
        assert_eq!(plain.buffer.read_column(key), imported.read_column(key));
    }
}

#[test]
fn native_mtl_rgb_opacity_map_precedence_and_spaced_paths() {
    let materials = parse_mtl(b"# appearance\nnewmtl wood\nKd .2 .4 .6\nd .8\nTr .25\nmap_Ka fallback.png\nmap_Kd textures/wood grain.png\nmap_Ka ignored.png\nnewmtl plain\nKd 1 0 0\n", &ObjAssetLimits::default()).unwrap();
    assert_eq!(
        materials["wood"].diffuse,
        Srgb {
            r: 0.2,
            g: 0.4,
            b: 0.6
        }
    );
    assert_eq!(materials["wood"].opacity, 0.75);
    assert_eq!(
        materials["wood"].texture.as_deref(),
        Some("textures/wood grain.png")
    );
    assert_eq!(materials["plain"].texture, None);
    assert_eq!(materials["plain"].opacity, 1.0);
}

#[test]
fn malformed_mtl_and_unsupported_diffuse_options_are_named_errors() {
    for source in [
        "Kd 1 0 0\n",
        "newmtl\n",
        "newmtl A\nKd nan 0 0\n",
        "newmtl A\nKd 1 2 0\n",
        "newmtl A\nd -1\n",
        "newmtl A\nKd spectral a.rfl\n",
        "newmtl A\nmap_Kd -s 2 2 1 a.png\n",
        "newmtl A\nmap_Kd\n",
    ] {
        assert!(
            parse_mtl(source.as_bytes(), &ObjAssetLimits::default()).is_err(),
            "{source}"
        );
    }
}

#[test]
fn import_budgets_precede_unbounded_geometry_and_metadata_allocation() {
    let tiny = ObjAssetLimits {
        max_source_bytes: 8,
        ..ObjAssetLimits::default()
    };
    assert!(
        ObjDocument::parse_with_limits(GEOMETRY.as_bytes(), &tiny)
            .err()
            .unwrap()
            .reason
            .contains("byte budget")
    );
    let tiny = ObjAssetLimits {
        max_materials: 1,
        ..ObjAssetLimits::default()
    };
    assert!(ObjDocument::parse_with_limits(b"mtllib a.mtl b.mtl", &tiny).is_err());
    assert!(parse_mtl(b"newmtl A\nnewmtl B\n", &tiny).is_err());
    let tiny = ObjAssetLimits {
        max_line_bytes: 4,
        ..ObjAssetLimits::default()
    };
    assert!(ObjDocument::parse_with_limits(b"v 1 2 3", &tiny).is_err());
    let tiny = ObjAssetLimits {
        max_name_bytes: 1,
        ..ObjAssetLimits::default()
    };
    assert!(ObjDocument::parse_with_limits(b"usemtl AB", &tiny).is_err());
    assert!(ObjDocument::parse(b"v 0 0 0\0").is_err());
    assert!(ObjDocument::parse(b"f 1 2 \\\n3").is_err());
}

#[test]
fn failed_conversion_never_changes_a_parsed_document_or_bound_resources() {
    let doc = ObjDocument::parse(format!("{GEOMETRY}f 1 2 3\n").as_bytes()).unwrap();
    let mesh = doc.mesh().clone();
    for height in [f64::NAN, 0.0, -1.0, f64::INFINITY, f64::MAX] {
        assert!(
            doc.to_mobject(height, &BTreeMap::new(), &BTreeMap::new())
                .is_err()
        );
    }
    assert_eq!(doc.mesh(), &mesh);
    assert!(
        doc.to_mobject(3.0, &BTreeMap::new(), &BTreeMap::new())
            .is_ok()
    );
    let large = ObjDocument::parse(b"v -1e308 0 0\nv 1e308 0 0\nv 0 1 0\nf 1 2 3\n").unwrap();
    assert!(
        large
            .to_mobject(3.0, &BTreeMap::new(), &BTreeMap::new())
            .is_err()
    );
}
