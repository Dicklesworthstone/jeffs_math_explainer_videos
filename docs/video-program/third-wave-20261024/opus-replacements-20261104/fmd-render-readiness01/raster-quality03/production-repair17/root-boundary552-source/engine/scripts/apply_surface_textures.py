"""Apply the texture adapter's narrow module and native topology integration."""
from pathlib import Path


def replace(path, old, new, marker=None):
    file = Path(path)
    text = file.read_text()
    if (new if marker is None else marker) in text:
        return
    if text.count(old) != 1:
        raise RuntimeError('refusing drifted texture integration: ' + path)
    file.write_text(text.replace(old, new, 1))


replace('crates/fmn-python/src/lib.rs', 'mod portal_video;\n',
        'mod portal_video;\nmod portal_texture;\n', marker='mod portal_texture;')
replace('crates/fmn-python/src/lib.rs', '    execute_bootstrap(py, module)?;\n',
        '    portal_texture::install(module)?;\n    execute_bootstrap(py, module)?;\n', marker='portal_texture::install(module)?;')
replace('crates/fmn-python/python/fmn_python/initialization.py',
        '    ("rendering", "install_scene_rendering"),\n',
        '    ("rendering", "install_scene_rendering"),\n    ("surface_textures", "install_surface_textures"),\n')
replace('scripts/check_portal_runtime.sh', 'for suite in portal_initialization ',
        'for suite in textured_surfaces portal_initialization ')

replace('crates/fmn-library/src/solids.rs',
'''impl From<TexturedGeometry> for Mobject {
    fn from(t: TexturedGeometry) -> Self {
        let mut buffer = RecordBuffer::new(textured_surface_schema(), t.points.len())
            .expect("record sizing bounded by the surface grid");
        buffer.write_range("point", 0, &flat_f32(&t.points));
        buffer.write_range("d_normal_point", 0, &flat_f32(&t.d_normal_point));
        #[allow(clippy::cast_possible_truncation)]
        let im_coords: Vec<f32> = t
            .im_coords
            .iter()
            .flat_map(|c| c.iter().map(|v| *v as f32))
            .collect();
        buffer.write_range("im_coords", 0, &im_coords);
        #[allow(clippy::cast_possible_truncation)]
        let opacity: Vec<f32> = vec![t.opacity as f32; t.points.len()];
        buffer.write_range("opacity", 0, &opacity);''',
'''impl From<TexturedGeometry> for Mobject {
    fn from(t: TexturedGeometry) -> Self {
        // TriangleMesh consumes consecutive triples. Expanding only here
        // preserves shared-vertex normal computation while retaining face
        // winding, order, repetition and UV seams in the rendered records.
        let mut buffer = RecordBuffer::new(textured_surface_schema(), t.triangle_indices.len())
            .expect("record sizing bounded by the mesh face count");
        for (row, &index) in t.triangle_indices.iter().enumerate() {
            let index = index as usize; // validated by from_mesh
            #[allow(clippy::cast_possible_truncation)]
            let point = t.points[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let normal = t.d_normal_point[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let uv = t.im_coords[index].map(|value| value as f32);
            #[allow(clippy::cast_possible_truncation)]
            let opacity = [t.opacity as f32];
            buffer.write(row, "point", &point);
            buffer.write(row, "d_normal_point", &normal);
            buffer.write(row, "im_coords", &uv);
            buffer.write(row, "opacity", &opacity);
        }''', marker='record sizing bounded by the mesh face count')

replace('crates/fmn-python/src/lib.rs', 'mod portal_texture;\n',
        'mod portal_texture;\n#[cfg(feature = "gauntlet")]\npub use portal_texture::run_portal_gauntlet_textures;\n', marker='pub use portal_texture::run_portal_gauntlet_textures;')
replace('crates/fmn-conformance/tests/e2e_scenarios.rs',
        'fn python_surface_mesh_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {',
        '''fn python_textured_surfaces_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {
    let (first, last) = manimlib::run_portal_gauntlet_textures()
        .map_err(|error| fail(format!("textured surfaces: {error}")))?;
    let limits = fmn_codec::PngLimits::default();
    let decoded = fmn_codec::decode_png(&first, &limits)
        .map_err(|error| fail(format!("decode texture frame: {error}")))?;
    if (decoded.width, decoded.height) != (96, 54) || first == last {
        return Err(fail("textured surface did not animate native pixels"));
    }
    ctx.event(LogEvent::new("e2e.python.textures").field("frames", 4_u64).field("thread_counts", 3_u64));
    Ok(RunOutcome::ok().with_artifact("textures_first.png", first).with_artifact("textures_last.png", last))
}

fn python_surface_mesh_run(ctx: &mut RunCtx) -> Result<RunOutcome, ScenarioError> {''', marker='fn python_textured_surfaces_run(')
replace('crates/fmn-conformance/tests/e2e_scenarios.rs',
        '''    specs.push(spec(
        "render_matrix.python_surface_mesh.v1",''',
        '''    specs.push(spec(
        "render_matrix.python_textured_surfaces.v1",
        ScenarioClass::RenderMatrix,
        Surface::PythonInProcess,
        Invocation::new(python_textured_surfaces_run),
        vec![Assertion::ExitCode(0), Assertion::FileInventory(vec!["textures_first.png".to_owned(), "textures_last.png".to_owned()])],
        vec![LogExpect::span_present("e2e.python.textures", vec![FieldPred::u64_eq("frames", 4), FieldPred::u64_eq("thread_counts", 3)])],
    ));
    specs.push(spec(
        "render_matrix.python_surface_mesh.v1",''', marker='Invocation::new(python_textured_surfaces_run)')
replace('crates/fmn-conformance/tests/e2e_scenarios.rs',
        'fn python_surface_mesh_scenario_passes() {',
        '''fn python_textured_surfaces_scenario_passes() {
    let scenario = catalog().into_iter()
        .find(|scenario| scenario.name == "render_matrix.python_textured_surfaces.v1")
        .expect("textured surface scenario is registered");
    let report = Runner::from_env().run(scenario);
    assert!(report.is_pass(), "{}", report.summary());
}

#[test]
fn python_surface_mesh_scenario_passes() {''', marker='fn python_textured_surfaces_scenario_passes(')
