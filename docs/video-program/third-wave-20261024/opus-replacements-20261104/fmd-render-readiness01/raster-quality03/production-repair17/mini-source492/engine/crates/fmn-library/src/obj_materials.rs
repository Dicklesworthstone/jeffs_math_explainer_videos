//! Material-aware OBJ composition over the existing owned geometry parser.
//!
//! This layer never opens files. The host supplies MTL bytes and decoded image
//! resources. Contiguous material runs preserve OBJ face order; UV/normal seams
//! retain per-corner indices rather than collapsing to position-only vertices.
//! The geometry-only [`super::obj_model::ThreeDModel`] API remains unchanged.

use std::collections::{BTreeMap, BTreeSet};
use std::ops::Range;

use fmn_core::color::Srgb;
use fmn_core::constants::GREY;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::uniforms::Uniforms;
use fmn_mobject::{ImageResource, Mobject, RenderPrimitive};

use super::obj_model::{MODEL_SHADING, ObjLimits, ObjMesh};

/// Import admission limits, applied before geometry or material allocation.
#[derive(Debug, Clone, Copy)]
pub struct ObjAssetLimits {
    /// Maximum bytes in one OBJ or MTL source.
    pub max_source_bytes: usize,
    /// Maximum bytes in one source line, including comments.
    pub max_line_bytes: usize,
    /// Maximum named materials, material runs, or referenced libraries.
    pub max_materials: usize,
    /// Maximum bytes in a material name or library/texture path.
    pub max_name_bytes: usize,
    /// Existing geometry parser budgets.
    pub geometry: ObjLimits,
}

impl Default for ObjAssetLimits {
    fn default() -> Self {
        Self {
            max_source_bytes: 64 * 1024 * 1024,
            max_line_bytes: 64 * 1024,
            max_materials: 4096,
            max_name_bytes: 4096,
            geometry: ObjLimits {
                max_triangles: 262_144,
                ..ObjLimits::default()
            },
        }
    }
}

/// A named, deterministic import refusal. Line numbers are one-based, or zero
/// for a whole-input/material binding error.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjAssetError {
    /// Source line, when available.
    pub line: usize,
    /// The reason the import was not published.
    pub reason: String,
}

impl std::fmt::Display for ObjAssetError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "OBJ material import, line {}: {}",
            self.line, self.reason
        )
    }
}
impl std::error::Error for ObjAssetError {}

fn error(line: usize, reason: impl Into<String>) -> ObjAssetError {
    ObjAssetError {
        line,
        reason: reason.into(),
    }
}

fn text<'a>(bytes: &'a [u8], limits: &ObjAssetLimits) -> Result<&'a str, ObjAssetError> {
    if bytes.len() > limits.max_source_bytes {
        return Err(error(0, "source byte budget exceeded"));
    }
    let value = std::str::from_utf8(bytes).map_err(|_| error(0, "source must be UTF-8"))?;
    for (i, line) in value.lines().enumerate() {
        if line.len() > limits.max_line_bytes {
            return Err(error(i + 1, "source line byte budget exceeded"));
        }
        if line.contains('\0') {
            return Err(error(i + 1, "NUL is not permitted in OBJ/MTL sources"));
        }
        if line
            .split('#')
            .next()
            .unwrap_or("")
            .trim_end()
            .ends_with('\\')
        {
            return Err(error(
                i + 1,
                "continued source lines are not supported; join the line explicitly",
            ));
        }
    }
    Ok(value)
}

fn statement(line: &str) -> (&str, &str) {
    let line = line.split('#').next().unwrap_or("").trim();
    line.split_once(char::is_whitespace)
        .map_or((line, ""), |(key, rest)| (key, rest.trim()))
}

fn name(value: &str, line: usize, limits: &ObjAssetLimits) -> Result<String, ObjAssetError> {
    if value.is_empty()
        || value.len() > limits.max_name_bytes
        || value.chars().any(char::is_control)
    {
        return Err(error(
            line,
            "empty, over-budget, or control-containing material name/path",
        ));
    }
    Ok(value.to_owned())
}

/// One contiguous face interval. Repeated materials are not globally regrouped:
/// changing their order would change transparent painter-order semantics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjMaterialRun {
    /// Selected material, or None before the first usemtl (or after usemtl off).
    pub material: Option<String>,
    /// Half-open triangle interval in the parsed geometry.
    pub triangles: Range<usize>,
}

/// An immutable parsed OBJ and its material binding metadata.
#[derive(Debug, Clone)]
pub struct ObjDocument {
    mesh: ObjMesh,
    libraries: Vec<String>,
    runs: Vec<ObjMaterialRun>,
}

impl ObjDocument {
    /// Parse a bounded document, using ObjMesh as the sole geometry authority.
    pub fn parse(bytes: &[u8]) -> Result<Self, ObjAssetError> {
        Self::parse_with_limits(bytes, &ObjAssetLimits::default())
    }

    /// Parse using explicit host admission limits.
    pub fn parse_with_limits(bytes: &[u8], limits: &ObjAssetLimits) -> Result<Self, ObjAssetError> {
        let source = text(bytes, limits)?;
        let mesh = ObjMesh::parse_with_limits(bytes, &limits.geometry)
            .map_err(|cause| error(0, cause.to_string()))?;
        let mut libraries = Vec::new();
        let mut runs: Vec<ObjMaterialRun> = Vec::new();
        let mut current = None;
        let mut cursor = 0usize;
        // This second pass records bindings only. Vertices, corner resolution,
        // triangulation and normals are never decoded or recomputed here.
        for (index, raw) in source.lines().enumerate() {
            let line = index + 1;
            let (key, rest) = statement(raw);
            match key {
                "mtllib" => {
                    if rest.is_empty() {
                        return Err(error(line, "mtllib requires a library path"));
                    }
                    for path in rest.split_whitespace() {
                        if libraries.len() >= limits.max_materials {
                            return Err(error(line, "material library budget exceeded"));
                        }
                        libraries.push(name(path, line, limits)?);
                    }
                }
                "usemtl" => {
                    current = if rest == "off" {
                        None
                    } else {
                        Some(name(rest, line, limits)?)
                    };
                }
                "f" => {
                    let count = rest.split_whitespace().count().saturating_sub(2);
                    let end = cursor
                        .checked_add(count)
                        .filter(|&end| end <= mesh.triangle_count())
                        .ok_or_else(|| {
                            error(line, "material/geometry face accounting disagrees")
                        })?;
                    if let Some(last) = runs.last_mut().filter(|run| run.material == current) {
                        last.triangles.end = end;
                    } else {
                        if runs.len() >= limits.max_materials {
                            return Err(error(line, "material run budget exceeded"));
                        }
                        runs.push(ObjMaterialRun {
                            material: current.clone(),
                            triangles: cursor..end,
                        });
                    }
                    cursor = end;
                }
                _ => {}
            }
        }
        if cursor != mesh.triangle_count() {
            return Err(error(0, "material/geometry triangle accounting disagrees"));
        }
        Ok(Self {
            mesh,
            libraries,
            runs,
        })
    }

    /// Parsed geometry, including per-corner texture/normal indices.
    #[must_use]
    pub fn mesh(&self) -> &ObjMesh {
        &self.mesh
    }

    /// Material libraries in OBJ declaration order, resolved only by the host.
    #[must_use]
    pub fn libraries(&self) -> &[String] {
        &self.libraries
    }

    /// Painter-ordered contiguous material runs.
    #[must_use]
    pub fn runs(&self) -> &[ObjMaterialRun] {
        &self.runs
    }

    /// Material names actually selected by at least one face.
    #[must_use]
    pub fn material_names(&self) -> BTreeSet<String> {
        self.runs
            .iter()
            .filter_map(|run| run.material.clone())
            .collect()
    }

    /// Compose a complete native model before publishing it into a Stage.
    ///
    /// `materials` supplies appearance descriptions; `images` supplies decoded
    /// map_Kd/map_Ka resources by material name. A declared map is mandatory and
    /// every textured face must have UVs. Plain runs use Kd/d; mapped runs use
    /// the image/d with the existing Reference lighting (not an MTL PBR shader).
    /// All children share one model normalization, never one per material.
    pub fn to_mobject(
        &self,
        height: f64,
        materials: &BTreeMap<String, ObjMaterial>,
        images: &BTreeMap<String, ImageResource>,
    ) -> Result<Mobject, ObjAssetError> {
        if !height.is_finite() || height <= 0.0 || height > f64::from(f32::MAX) {
            return Err(error(
                0,
                "model height must be positive, finite and f32-representable",
            ));
        }
        // Check all resources before building any records. An unresolved map
        // is never replaced with an untextured grey success-shaped model.
        for run in &self.runs {
            if let Some(key) = &run.material {
                let material = materials
                    .get(key)
                    .ok_or_else(|| error(0, format!("material {key:?} is not defined")))?;
                material.validate()?;
                if material.texture.is_some() {
                    let image = images.get(key).ok_or_else(|| {
                        error(0, format!("texture for material {key:?} was not supplied"))
                    })?;
                    if image.dark_image().is_some() {
                        return Err(error(0, "OBJ diffuse maps require an unpaired raster"));
                    }
                    if self.mesh.triangles[run.triangles.clone()]
                        .iter()
                        .flatten()
                        .any(|corner| corner.tex_coord.is_none())
                    {
                        return Err(error(
                            0,
                            format!(
                                "textured material {key:?} has a face corner without UV coordinates"
                            ),
                        ));
                    }
                }
            }
        }
        let (center, scale) = self.normalization(height)?;
        let mut children = Vec::new();
        children
            .try_reserve_exact(self.runs.len())
            .map_err(|_| error(0, "model child allocation failed"))?;
        for run in &self.runs {
            let material = run.material.as_ref().and_then(|key| materials.get(key));
            let image = run.material.as_ref().and_then(|key| {
                material
                    .filter(|value| value.texture.is_some())
                    .and_then(|_| images.get(key))
            });
            let schema = if image.is_some() {
                super::solids::textured_surface_schema()
            } else {
                RecordSchema::new(
                    &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
                    &["point"],
                    &["point", "d_normal_point"],
                )
                .map_err(|cause| error(0, cause.to_string()))?
            };
            let count = run
                .triangles
                .len()
                .checked_mul(3)
                .ok_or_else(|| error(0, "model record count overflow"))?;
            let mut buffer =
                RecordBuffer::new(schema, count).map_err(|cause| error(0, cause.to_string()))?;
            let opacity = material.map_or(1.0, |value| value.opacity);
            let color = material.map_or(GREY, |value| value.diffuse);
            for (local, triangle) in run.triangles.clone().enumerate() {
                for (k, corner) in self.mesh.triangles[triangle].iter().enumerate() {
                    let vertex = self.mesh.vertices[corner.vertex];
                    let point =
                        std::array::from_fn::<_, 3, _>(|dim| (vertex[dim] - center[dim]) * scale);
                    let normal = self.mesh.corner_normal(triangle, k);
                    let normal_point =
                        std::array::from_fn::<_, 3, _>(|dim| point[dim] + normal[dim]);
                    let record = 3 * local + k;
                    buffer.write(record, "point", &narrow(point)?);
                    buffer.write(record, "d_normal_point", &narrow(normal_point)?);
                    if image.is_some() {
                        let uv = self.mesh.tex_coords
                            [corner.tex_coord.ok_or_else(|| error(0, "missing UV"))?];
                        buffer.write(record, "im_coords", &narrow([uv[0], 1.0 - uv[1]])?);
                        buffer.write(record, "opacity", &narrow([opacity])?);
                    } else {
                        buffer.write(
                            record,
                            "rgba",
                            &narrow([color.r, color.g, color.b, opacity])?,
                        );
                    }
                }
            }
            let mut child = Mobject::from_buffer(buffer)
                .with_render_primitive(RenderPrimitive::TriangleMesh)
                .with_uniforms(Uniforms {
                    shading: MODEL_SHADING,
                    depth_test: true,
                    ..Uniforms::default()
                });
            child.image = image.cloned();
            children.push(child);
        }
        Ok(Mobject::group(children))
    }

    fn normalization(&self, height: f64) -> Result<([f64; 3], f64), ObjAssetError> {
        let Some(&first) = self.mesh.vertices.first() else {
            return Ok(([0.0; 3], 1.0));
        };
        let (mut min, mut max) = (first, first);
        for point in &self.mesh.vertices {
            for dim in 0..3 {
                min[dim] = min[dim].min(point[dim]);
                max[dim] = max[dim].max(point[dim]);
            }
        }
        let center = std::array::from_fn(|dim| min[dim] * 0.5 + max[dim] * 0.5);
        let extent = max[1] - min[1];
        if !extent.is_finite() {
            return Err(error(0, "model bounds overflow"));
        }
        let scale = if extent > 0.0 { height / extent } else { 1.0 };
        if !scale.is_finite() {
            return Err(error(0, "model normalization overflow"));
        }
        Ok((center, scale))
    }
}

fn narrow<const N: usize>(values: [f64; N]) -> Result<[f32; N], ObjAssetError> {
    if !values
        .iter()
        .all(|value| value.is_finite() && value.abs() <= f64::from(f32::MAX))
    {
        return Err(error(
            0,
            "model coordinates must be finite and f32-representable",
        ));
    }
    #[allow(clippy::cast_possible_truncation)]
    Ok(values.map(|value| value as f32))
}

/// The supported MTL appearance subset. Texture maps use existing native
/// image sampling and lighting; unknown reflectance models are not emulated.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjMaterial {
    /// Diffuse sRGB for a material without a raster map.
    pub diffuse: Srgb,
    /// Dissolve alpha: d, or one minus Tr. Last declaration wins.
    pub opacity: f64,
    /// Diffuse map path; map_Ka is a fallback only when map_Kd is absent.
    pub texture: Option<String>,
}

impl Default for ObjMaterial {
    fn default() -> Self {
        Self {
            diffuse: Srgb {
                r: 1.0,
                g: 1.0,
                b: 1.0,
            },
            opacity: 1.0,
            texture: None,
        }
    }
}

impl ObjMaterial {
    fn validate(&self) -> Result<(), ObjAssetError> {
        if ![self.diffuse.r, self.diffuse.g, self.diffuse.b, self.opacity]
            .iter()
            .all(|value| value.is_finite() && (0.0..=1.0).contains(value))
        {
            return Err(error(
                0,
                "material RGB and dissolve must be finite in 0..=1",
            ));
        }
        Ok(())
    }
}

/// Parse native MTL newmtl/Kd/d/Tr/map_Kd/map_Ka under explicit limits.
/// Unsupported diffuse map options and non-RGB Kd are refused rather than
/// silently rendering a different material. No paths are opened here.
pub fn parse_mtl(
    bytes: &[u8],
    limits: &ObjAssetLimits,
) -> Result<BTreeMap<String, ObjMaterial>, ObjAssetError> {
    let source = text(bytes, limits)?;
    let mut materials = BTreeMap::new();
    let mut current = None;
    let mut has_diffuse_map = false;
    for (index, raw) in source.lines().enumerate() {
        let line = index + 1;
        let (key, rest) = statement(raw);
        if key == "newmtl" {
            let key = name(rest, line, limits)?;
            if materials.len() >= limits.max_materials && !materials.contains_key(&key) {
                return Err(error(line, "material count budget exceeded"));
            }
            materials.insert(key.clone(), ObjMaterial::default());
            current = Some(key);
            has_diffuse_map = false;
            continue;
        }
        if !matches!(key, "Kd" | "d" | "Tr" | "map_Kd" | "map_Ka") {
            continue;
        }
        let material = current
            .as_ref()
            .and_then(|key| materials.get_mut(key))
            .ok_or_else(|| error(line, "material property appears before newmtl"))?;
        match key {
            "Kd" | "d" | "Tr" => {
                let mut values = Vec::new();
                for token in rest.split_whitespace() {
                    let number: f64 = token
                        .parse()
                        .map_err(|_| error(line, "expected numeric RGB/dissolve"))?;
                    if !number.is_finite() || !(0.0..=1.0).contains(&number) {
                        return Err(error(line, "material RGB/dissolve must be finite in 0..=1"));
                    }
                    values.push(number);
                    if values.len() > 3 {
                        return Err(error(line, "too many RGB/dissolve components"));
                    }
                }
                if key == "Kd" && values.len() == 3 {
                    material.diffuse = Srgb {
                        r: values[0],
                        g: values[1],
                        b: values[2],
                    };
                } else if key != "Kd" && values.len() == 1 {
                    material.opacity = if key == "Tr" {
                        1.0 - values[0]
                    } else {
                        values[0]
                    };
                } else {
                    return Err(error(
                        line,
                        "expected three RGB components or one dissolve component",
                    ));
                }
            }
            "map_Kd" | "map_Ka" => {
                if rest.starts_with('-') {
                    return Err(error(
                        line,
                        "diffuse texture map options are unsupported; bake the map transform into UVs",
                    ));
                }
                let path = name(rest, line, limits)?;
                if key == "map_Kd" || !has_diffuse_map {
                    material.texture = Some(path);
                }
                has_diffuse_map |= key == "map_Kd";
            }
            _ => unreachable!("matched material properties"),
        }
    }
    Ok(materials)
}
