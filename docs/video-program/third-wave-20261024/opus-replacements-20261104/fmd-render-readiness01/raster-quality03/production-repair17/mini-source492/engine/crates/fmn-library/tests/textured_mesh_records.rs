//! Atlas mesh construction must not lose indexed face topology at arena entry.
use fmn_library::TexturedGeometry;
use fmn_mobject::{Mobject, RenderPrimitive};

#[test]
fn indexed_texture_mesh_expands_authored_faces_and_all_vertex_fields() {
    let vertices = vec![
        [0.0, 0.0, 0.0],
        [2.0, 0.0, 0.0],
        [0.0, 2.0, 0.0],
        [0.0, 0.0, 2.0],
    ];
    let faces = [2, 0, 1, 1, 0, 3, 2, 0, 1];
    let uv = [[0.0, 0.0], [1.0, 0.0], [0.0, 1.0], [1.0, 1.0]];
    let mesh = TexturedGeometry::from_mesh(vertices.clone(), &faces, &uv, "texture.png")
        .unwrap()
        .with_opacity(0.25);
    let expected = mesh.clone();
    let mob: Mobject = mesh.into();
    assert_eq!(mob.render_primitive, RenderPrimitive::TriangleMesh);
    assert_eq!(mob.buffer.len(), faces.len());
    for (row, &index) in faces.iter().enumerate() {
        let index = index as usize;
        let point = mob.buffer.read(row, "point").unwrap();
        assert_eq!(
            point
                .iter()
                .map(|&value| f64::from(value))
                .collect::<Vec<_>>(),
            vertices[index]
        );
        let normal = mob.buffer.read(row, "d_normal_point").unwrap();
        for (&got, &want) in normal.iter().zip(&expected.d_normal_points()[index]) {
            assert!((f64::from(got) - want).abs() < 1e-6);
        }
        assert_eq!(
            mob.buffer
                .read(row, "im_coords")
                .unwrap()
                .iter()
                .map(|&value| f64::from(value))
                .collect::<Vec<_>>(),
            expected.im_coords()[index]
        );
        assert_eq!(mob.buffer.read(row, "opacity").unwrap(), vec![0.25]);
    }
}
