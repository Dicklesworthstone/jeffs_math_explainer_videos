//! `ThreeDModel` and the owned OBJ-subset reader (§12.4, fm-2u6; Appendix A
//! `types/surface.py:ThreeDModel`).
//!
//! The Reference loads a Wavefront OBJ through trimesh (with a pywavefront
//! pass for MTL textures). Both are displaced here by a small owned
//! parser — the governed closure admits no new dependency for a text
//! format — reading exactly the geometry subset a model needs:
//!
//! # The supported subset (and its documented limits)
//!
//! | Statement | Support |
//! |---|---|
//! | `v x y z [w]` | yes; the homogeneous `w` is accepted and ignored |
//! | `vn i j k` | yes; stored raw, normalized at mobject conversion |
//! | `vt u [v [w]]` | yes; `v` defaults to 0, `w` ignored |
//! | `f c₁ c₂ …` | yes; corner forms `v`, `v/vt`, `v//vn`, `v/vt/vn`; 1-based and negative (relative) indices; polygons fan-triangulated |
//! | `o`, `g`, `s`, `usemtl`, `mtllib`, `l`, `p`, `vp` | parsed over (no effect) |
//! | anything else | skipped (trimesh-compatible tolerance) |
//! | `#` comments, blank lines, CRLF | handled |
//!
//! **Not supported:** MTL material/texture resolution (the Reference's
//! pywavefront pass) — there is no texture carriage channel in the
//! detached mobject yet (the same seam [`crate::image`] documents), so
//! `usemtl`/`mtllib` are parsed over and the model converts as a
//! plain-colored, lit triangle mesh. Free-form curves/surfaces (`cstype`
//! etc.) are skipped like every unknown statement. There is no axis
//! remap: OBJ coordinates are used as-is (trimesh behavior).
//!
//! # Untrusted input (§16.5/R14)
//!
//! The parser is line-based and total: every malformed byte sequence
//! yields a typed [`ObjError`] naming the line and the token — never a
//! panic — and resource budgets ([`ObjLimits`]) are declared before any
//! allocation, refusing over-budget input with a named error. It is a
//! target of the W10 fuzz campaign (`obj_model` in fmn-conformance's
//! `tests/fuzz_campaign.rs`).

use fmn_core::color::Srgb;
use fmn_core::constants::GREY;
use fmn_core::types::Vec3;
use fmn_mobject::record::{RecordBuffer, RecordSchema};
use fmn_mobject::uniforms::Uniforms;
use fmn_mobject::{Mobject, RenderPrimitive};

/// The Reference's default `height` for `ThreeDModel` (scene units).
pub const DEFAULT_MODEL_HEIGHT: f64 = 3.0;

/// The kept Surface shading default `(reflectiveness, gloss, shadow)` —
/// G0-2 decision row (e) (`surface.py:47`), which `ThreeDModel`'s
/// `TexturedSurface` children inherit.
pub const MODEL_SHADING: Vec3 = [0.3, 0.2, 0.4];

// ------------------------------------------------------------------ errors

/// Why byte input could not become an [`ObjMesh`]. Every variant names
/// the 1-based line; float/index variants name the offending token.
#[derive(Debug, Clone, PartialEq)]
pub enum ObjError {
    /// The input is not UTF-8; `offset` is the first invalid byte.
    NotUtf8 {
        /// Byte offset of the first invalid UTF-8 sequence.
        offset: usize,
    },
    /// A float token did not parse as a finite f64 (`nan`/`inf` parse but
    /// are rejected — geometry must be finite).
    BadFloat {
        /// 1-based line number.
        line: usize,
        /// The statement being parsed (`v`, `vn`, `vt`).
        statement: &'static str,
        /// The offending token.
        token: String,
    },
    /// An index token did not parse as an integer, or a corner had too
    /// many `/` fields.
    BadIndex {
        /// 1-based line number.
        line: usize,
        /// The offending token.
        token: String,
    },
    /// An index resolved outside its declared list. OBJ indices are
    /// 1-based; `0` is always out of range.
    IndexOutOfRange {
        /// 1-based line number.
        line: usize,
        /// The index as written (after sign, before resolution).
        index: i64,
        /// The declared length of the referenced list.
        len: usize,
    },
    /// A face named fewer than three corners.
    FaceTooSmall {
        /// 1-based line number.
        line: usize,
        /// How many corners the face named.
        corners: usize,
    },
    /// The vertex budget refused.
    TooManyVertices {
        /// 1-based line number.
        line: usize,
        /// The configured budget.
        limit: usize,
    },
    /// The texture-coordinate budget refused.
    TooManyTexCoords {
        /// 1-based line number.
        line: usize,
        /// The configured budget.
        limit: usize,
    },
    /// The normal budget refused.
    TooManyNormals {
        /// 1-based line number.
        line: usize,
        /// The configured budget.
        limit: usize,
    },
    /// The triangle budget refused (fan triangulation counts).
    TooManyTriangles {
        /// 1-based line number.
        line: usize,
        /// The configured budget.
        limit: usize,
    },
}

impl std::fmt::Display for ObjError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotUtf8 { offset } => {
                write!(
                    f,
                    "input is not UTF-8 (first invalid byte at offset {offset})"
                )
            }
            Self::BadFloat {
                line,
                statement,
                token,
            } => write!(
                f,
                "line {line}: `{statement}` coordinate {token:?} is not a finite float"
            ),
            Self::BadIndex { line, token } => {
                write!(f, "line {line}: face index {token:?} is not an integer")
            }
            Self::IndexOutOfRange { line, index, len } => write!(
                f,
                "line {line}: index {index} resolves out of range \
                 ({len} declared; OBJ indices are 1-based, negatives relative)"
            ),
            Self::FaceTooSmall { line, corners } => write!(
                f,
                "line {line}: face names {corners} corner(s); at least 3 are required"
            ),
            Self::TooManyVertices { line, limit } => {
                write!(f, "line {line}: vertex budget of {limit} exceeded")
            }
            Self::TooManyTexCoords { line, limit } => {
                write!(
                    f,
                    "line {line}: texture-coordinate budget of {limit} exceeded"
                )
            }
            Self::TooManyNormals { line, limit } => {
                write!(f, "line {line}: normal budget of {limit} exceeded")
            }
            Self::TooManyTriangles { line, limit } => {
                write!(f, "line {line}: triangle budget of {limit} exceeded")
            }
        }
    }
}

impl std::error::Error for ObjError {}

// ------------------------------------------------------------------ limits

/// Resource budgets for the OBJ reader, declared before any allocation
/// (§16.5/R14).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjLimits {
    /// Maximum `v` statements.
    pub max_vertices: usize,
    /// Maximum `vt` statements.
    pub max_tex_coords: usize,
    /// Maximum `vn` statements.
    pub max_normals: usize,
    /// Maximum emitted triangles (fan triangulation counts against this).
    pub max_triangles: usize,
}

impl Default for ObjLimits {
    fn default() -> Self {
        Self {
            max_vertices: 1 << 20,
            max_tex_coords: 1 << 20,
            max_normals: 1 << 20,
            max_triangles: 1 << 21,
        }
    }
}

// -------------------------------------------------------------------- mesh

/// One corner of a triangle: the vertex plus its optional texture
/// coordinate and normal, resolved to 0-based indices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ObjCorner {
    /// 0-based index into [`ObjMesh::vertices`].
    pub vertex: usize,
    /// 0-based index into [`ObjMesh::tex_coords`], if the corner named one.
    pub tex_coord: Option<usize>,
    /// 0-based index into [`ObjMesh::normals`], if the corner named one.
    pub normal: Option<usize>,
}

/// A parsed OBJ mesh: the declared attribute lists and the fan-triangulated
/// faces.
#[derive(Debug, Clone, PartialEq)]
pub struct ObjMesh {
    /// `v` statements, in declaration order.
    pub vertices: Vec<Vec3>,
    /// `vt` statements (`v` defaulted to 0), in declaration order.
    pub tex_coords: Vec<[f64; 2]>,
    /// `vn` statements, raw as declared (normalized at conversion).
    pub normals: Vec<Vec3>,
    /// Fan-triangulated faces.
    pub triangles: Vec<[ObjCorner; 3]>,
}

impl ObjMesh {
    /// Parse OBJ bytes under the default budgets.
    pub fn parse(bytes: &[u8]) -> Result<Self, ObjError> {
        Self::parse_with_limits(bytes, &ObjLimits::default())
    }

    /// Parse OBJ bytes under explicit budgets (the untrusted-input path).
    pub fn parse_with_limits(bytes: &[u8], limits: &ObjLimits) -> Result<Self, ObjError> {
        let text = std::str::from_utf8(bytes).map_err(|e| ObjError::NotUtf8 {
            offset: e.valid_up_to(),
        })?;
        let mut mesh = Self {
            vertices: Vec::new(),
            tex_coords: Vec::new(),
            normals: Vec::new(),
            triangles: Vec::new(),
        };
        for (line_no, raw_line) in text.lines().enumerate() {
            let line = line_no + 1;
            // `#` starts a comment; surrounding whitespace is insignificant.
            let content = raw_line.split('#').next().unwrap_or("").trim();
            if content.is_empty() {
                continue;
            }
            let mut tokens = content.split_whitespace();
            let statement = tokens.next().unwrap_or("");
            let args: Vec<&str> = tokens.collect();
            match statement {
                "v" => {
                    if mesh.vertices.len() >= limits.max_vertices {
                        return Err(ObjError::TooManyVertices {
                            line,
                            limit: limits.max_vertices,
                        });
                    }
                    let xyz = parse_floats(&args, line, "v")?;
                    mesh.vertices.push(xyz);
                }
                "vn" => {
                    if mesh.normals.len() >= limits.max_normals {
                        return Err(ObjError::TooManyNormals {
                            line,
                            limit: limits.max_normals,
                        });
                    }
                    let ijk = parse_floats(&args, line, "vn")?;
                    mesh.normals.push(ijk);
                }
                "vt" => {
                    if mesh.tex_coords.len() >= limits.max_tex_coords {
                        return Err(ObjError::TooManyTexCoords {
                            line,
                            limit: limits.max_tex_coords,
                        });
                    }
                    if args.is_empty() {
                        return Err(ObjError::BadFloat {
                            line,
                            statement: "vt",
                            token: String::new(),
                        });
                    }
                    let u = parse_float(args[0], line, "vt")?;
                    let v = match args.get(1) {
                        Some(token) => parse_float(token, line, "vt")?,
                        None => 0.0,
                    };
                    mesh.tex_coords.push([u, v]);
                }
                "f" => {
                    if args.len() < 3 {
                        return Err(ObjError::FaceTooSmall {
                            line,
                            corners: args.len(),
                        });
                    }
                    let mut corners = Vec::with_capacity(args.len());
                    for token in &args {
                        corners.push(parse_corner(token, line, &mesh)?);
                    }
                    // Fan triangulation: (0, i, i+1), matching trimesh's
                    // handling of convex OBJ polygons.
                    for i in 1..corners.len() - 1 {
                        if mesh.triangles.len() >= limits.max_triangles {
                            return Err(ObjError::TooManyTriangles {
                                line,
                                limit: limits.max_triangles,
                            });
                        }
                        mesh.triangles
                            .push([corners[0], corners[i], corners[i + 1]]);
                    }
                }
                // Known no-ops for the geometry subset (grouping, smoothing,
                // materials, lines, points, parameter-space vertices).
                "o" | "g" | "s" | "usemtl" | "mtllib" | "l" | "p" | "vp" => {}
                // Unknown statements are skipped, trimesh-compatible.
                _ => {}
            }
        }
        Ok(mesh)
    }

    /// No triangles.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.triangles.is_empty()
    }

    /// Declared vertex count.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Emitted triangle count.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// The geometric normal of a triangle: `cross(v1-v0, v2-v0)`
    /// normalized, or `[0, 0, 0]` for a degenerate (zero-area) triangle —
    /// a zero normal is a defined value the lighting treats as unlit,
    /// where a NaN would poison the fragment.
    #[must_use]
    pub fn face_normal(&self, triangle: usize) -> Vec3 {
        let Some(corners) = self.triangles.get(triangle) else {
            return [0.0; 3];
        };
        let v0 = self.vertices[corners[0].vertex];
        let v1 = self.vertices[corners[1].vertex];
        let v2 = self.vertices[corners[2].vertex];
        normalized(cross(sub(v1, v0), sub(v2, v0)))
    }

    /// The normal of one corner: its declared `vn` normalized, falling
    /// back to the geometric face normal when the corner named none (or a
    /// degenerate zero normal).
    #[must_use]
    pub fn corner_normal(&self, triangle: usize, corner: usize) -> Vec3 {
        let face = self.face_normal(triangle);
        let Some(corners) = self.triangles.get(triangle) else {
            return face;
        };
        let Some(corner) = corners.get(corner) else {
            return face;
        };
        match corner.normal {
            Some(index) => match self.normals.get(index) {
                Some(&vn) => {
                    let n = normalized(vn);
                    if n == [0.0; 3] { face } else { n }
                }
                None => face,
            },
            None => face,
        }
    }
}

/// Parse the first three tokens of a statement as finite f64s; a short
/// argument list reports the first missing slot as an empty token (a
/// statement that ran out of coordinates is malformed the same way a
/// non-numeric one is).
fn parse_floats(args: &[&str], line: usize, statement: &'static str) -> Result<Vec3, ObjError> {
    let mut out = [0.0; 3];
    for (slot, value) in out.iter_mut().enumerate() {
        let token = args.get(slot).copied().unwrap_or("");
        *value = parse_float(token, line, statement)?;
    }
    Ok(out)
}

/// One finite f64 token. `f64::from_str` accepts `nan`/`inf` spellings;
/// geometry must be finite, so those are refused here.
fn parse_float(token: &str, line: usize, statement: &'static str) -> Result<f64, ObjError> {
    match token.parse::<f64>() {
        Ok(value) if value.is_finite() => Ok(value),
        _ => Err(ObjError::BadFloat {
            line,
            statement,
            token: token.to_owned(),
        }),
    }
}

/// Parse one face corner: `v`, `v/vt`, `v//vn`, or `v/vt/vn`, resolving
/// 1-based and negative (relative) indices against the lists declared so
/// far (OBJ semantics: an index may only reference earlier statements).
fn parse_corner(token: &str, line: usize, mesh: &ObjMesh) -> Result<ObjCorner, ObjError> {
    let fields: Vec<&str> = token.split('/').collect();
    if fields.len() > 3 {
        return Err(ObjError::BadIndex {
            line,
            token: token.to_owned(),
        });
    }
    let parse_index = |raw: &str, line: usize| -> Result<i64, ObjError> {
        raw.parse::<i64>().map_err(|_| ObjError::BadIndex {
            line,
            token: raw.to_owned(),
        })
    };
    let vertex = resolve_index(parse_index(fields[0], line)?, mesh.vertices.len(), line)?;
    let tex_coord = match fields.get(1) {
        Some(&"") | None => None,
        Some(raw) => Some(resolve_index(
            parse_index(raw, line)?,
            mesh.tex_coords.len(),
            line,
        )?),
    };
    let normal = match fields.get(2) {
        Some(&"") | None => None,
        Some(raw) => Some(resolve_index(
            parse_index(raw, line)?,
            mesh.normals.len(),
            line,
        )?),
    };
    Ok(ObjCorner {
        vertex,
        tex_coord,
        normal,
    })
}

/// Resolve an OBJ index to 0-based: positive is 1-based absolute,
/// negative is relative to the current list length, zero is invalid.
fn resolve_index(raw: i64, len: usize, line: usize) -> Result<usize, ObjError> {
    let resolved = if raw > 0 {
        raw - 1
    } else {
        #[allow(clippy::cast_possible_wrap)]
        let len = len as i64;
        len + raw
    };
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let out_of_range = resolved < 0 || resolved >= len as i64;
    if out_of_range {
        return Err(ObjError::IndexOutOfRange {
            line,
            index: raw,
            len,
        });
    }
    #[allow(clippy::cast_sign_loss)]
    Ok(resolved as usize)
}

fn sub(a: Vec3, b: Vec3) -> Vec3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn cross(a: Vec3, b: Vec3) -> Vec3 {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Normalize, mapping the degenerate zero vector to `[0, 0, 0]` rather
/// than NaN.
fn normalized(v: Vec3) -> Vec3 {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len <= f64::EPSILON {
        return [0.0; 3];
    }
    [v[0] / len, v[1] / len, v[2] / len]
}

// -------------------------------------------------------------- ThreeDModel

/// `ThreeDModel`: an OBJ mesh as a lit, depth-tested triangle-soup mobject
/// (`surface.py:435`). The Reference's `TexturedGeometry` children carry
/// MTL textures; ours converts as a plain-colored mesh — the documented
/// subset limit at the top of this module.
#[derive(Debug, Clone, PartialEq)]
pub struct ThreeDModel {
    mesh: ObjMesh,
    height: f64,
    color: Srgb,
    opacity: f64,
    shading: Vec3,
    depth_test: bool,
    z_index: i32,
}

impl ThreeDModel {
    /// Parse an OBJ model (Reference `ThreeDModel(obj_file)`): Surface-grey,
    /// `height = 3`, shaded `(0.3, 0.2, 0.4)`, depth-tested, centered.
    pub fn from_obj(bytes: &[u8]) -> Result<Self, ObjError> {
        Self::from_obj_with_limits(bytes, &ObjLimits::default())
    }

    /// Parse under explicit budgets (the untrusted-input path).
    pub fn from_obj_with_limits(bytes: &[u8], limits: &ObjLimits) -> Result<Self, ObjError> {
        let mesh = ObjMesh::parse_with_limits(bytes, limits)?;
        Ok(Self {
            mesh,
            height: DEFAULT_MODEL_HEIGHT,
            color: GREY,
            opacity: 1.0,
            shading: MODEL_SHADING,
            depth_test: true,
            z_index: 0,
        })
    }

    /// The scene-space height (`height=`; default 3.0).
    #[must_use]
    pub fn with_height(mut self, height: f64) -> Self {
        self.height = height;
        self
    }

    /// The flat color (`color=`; Surface's GREY default).
    #[must_use]
    pub fn with_color(mut self, color: Srgb, opacity: f64) -> Self {
        self.color = color;
        self.opacity = opacity;
        self
    }

    /// The `(reflectiveness, gloss, shadow)` shading triple.
    #[must_use]
    pub fn with_shading(mut self, shading: Vec3) -> Self {
        self.shading = shading;
        self
    }

    /// Reference `apply_depth_test` / `remove_depth_test`.
    #[must_use]
    pub fn with_depth_test(mut self, depth_test: bool) -> Self {
        self.depth_test = depth_test;
        self
    }

    /// The scene-list sort key (§8.5).
    #[must_use]
    pub fn with_z_index(mut self, z_index: i32) -> Self {
        self.z_index = z_index;
        self
    }

    /// The parsed mesh.
    #[must_use]
    pub fn mesh(&self) -> &ObjMesh {
        &self.mesh
    }

    /// The transform applied at conversion: uniform scale to `height` and
    /// re-centering on the origin (Reference `set_height` + `center`),
    /// over the bounding box of the declared vertices.
    #[must_use]
    fn normalization(&self) -> (Vec3, f64) {
        let mut iter = self.mesh.vertices.iter();
        let Some(&first) = iter.next() else {
            return ([0.0; 3], 1.0);
        };
        let (mut min, mut max) = (first, first);
        for v in iter {
            for dim in 0..3 {
                min[dim] = min[dim].min(v[dim]);
                max[dim] = max[dim].max(v[dim]);
            }
        }
        let center = [
            (min[0] + max[0]) / 2.0,
            (min[1] + max[1]) / 2.0,
            (min[2] + max[2]) / 2.0,
        ];
        let extent_y = max[1] - min[1];
        let scale = if extent_y > 0.0 {
            self.height / extent_y
        } else {
            1.0
        };
        (center, scale)
    }
}

impl From<ThreeDModel> for Mobject {
    fn from(model: ThreeDModel) -> Self {
        let (center, scale) = model.normalization();
        let place = |v: Vec3| {
            [
                (v[0] - center[0]) * scale,
                (v[1] - center[1]) * scale,
                (v[2] - center[2]) * scale,
            ]
        };
        // The Reference's Surface dtype, field for field:
        // [('point', f32, 3), ('d_normal_point', f32, 3), ('rgba', f32, 4)]
        // — triangle soup, three consecutive records per triangle.
        let schema = RecordSchema::new(
            &[("point", 3), ("d_normal_point", 3), ("rgba", 4)],
            &["point"],
            &["point", "d_normal_point"],
        )
        .expect("the obj-model record schema is ten lanes");
        // The mesh index buffer is itself memory-resident, so its record
        // count is far below any sizing ceiling.
        let mut buffer = RecordBuffer::new(schema, model.mesh.triangles.len() * 3)
            .expect("obj-model record sizing bounded by the loaded mesh");
        let rgba = [model.color.r, model.color.g, model.color.b, model.opacity];
        for (triangle, corners) in model.mesh.triangles.iter().enumerate() {
            for (k, corner) in corners.iter().enumerate() {
                let point = place(model.mesh.vertices[corner.vertex]);
                // `d_normal_point - point` carries the normal direction;
                // the surface shaders normalize the difference.
                let normal = model.mesh.corner_normal(triangle, k);
                let d_normal_point = [
                    point[0] + normal[0],
                    point[1] + normal[1],
                    point[2] + normal[2],
                ];
                let record = triangle * 3 + k;
                #[allow(clippy::cast_possible_truncation)]
                buffer.write(
                    record,
                    "point",
                    &[point[0] as f32, point[1] as f32, point[2] as f32],
                );
                #[allow(clippy::cast_possible_truncation)]
                buffer.write(
                    record,
                    "d_normal_point",
                    &[
                        d_normal_point[0] as f32,
                        d_normal_point[1] as f32,
                        d_normal_point[2] as f32,
                    ],
                );
                #[allow(clippy::cast_possible_truncation)]
                buffer.write(
                    record,
                    "rgba",
                    &[
                        rgba[0] as f32,
                        rgba[1] as f32,
                        rgba[2] as f32,
                        rgba[3] as f32,
                    ],
                );
            }
        }
        let uniforms = Uniforms {
            shading: model.shading,
            depth_test: model.depth_test,
            ..Uniforms::default()
        };
        Mobject::from_buffer(buffer)
            .with_uniforms(uniforms)
            .with_render_primitive(RenderPrimitive::TriangleMesh)
            .with_z_index(model.z_index)
    }
}

// ------------------------------------------------------------------ tests

#[cfg(test)]
mod tests {
    use super::*;

    fn close_vec(a: Vec3, b: Vec3) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 1e-9)
    }

    /// A tetrahedron with per-vertex normals (the unit axes + one diagonal).
    const TETRAHEDRON: &str = "\
# a tetrahedron
v 0.0 1.0 0.0
v -1.0 -1.0 1.0
v 1.0 -1.0 1.0
v 0.0 -1.0 -1.0
vn 1.0 0.0 0.0
vn 0.0 1.0 0.0
vn 0.0 0.0 1.0
vn -1.0 0.0 0.0
f 1//1 2//2 3//3
f 1//1 3//3 4//4
f 1//1 4//4 2//2
f 2//2 4//4 3//3
";

    #[test]
    fn tetrahedron_fixture_counts() {
        let mesh = ObjMesh::parse(TETRAHEDRON.as_bytes()).expect("parses");
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.normals.len(), 4);
        assert_eq!(mesh.triangle_count(), 4);
        assert!(mesh.tex_coords.is_empty());
        // Indices resolve 1-based to 0-based.
        assert_eq!(mesh.triangles[0][0].vertex, 0);
        assert_eq!(mesh.triangles[0][2].vertex, 2);
        assert_eq!(mesh.triangles[0][0].normal, Some(0));
        assert!(mesh.triangles[0][0].tex_coord.is_none());
    }

    #[test]
    fn explicit_normals_are_used_and_normalized() {
        let mesh = ObjMesh::parse(TETRAHEDRON.as_bytes()).expect("parses");
        assert!(close_vec(mesh.corner_normal(0, 0), [1.0, 0.0, 0.0]));
        assert!(close_vec(mesh.corner_normal(0, 2), [0.0, 0.0, 1.0]));
        // A non-unit declared normal is normalized at read time.
        let scaled = "v 0 0 0\nv 1 0 0\nv 0 1 0\nvn 0 0 5\nf 1//1 2//1 3//1\n";
        let mesh = ObjMesh::parse(scaled.as_bytes()).expect("parses");
        assert!(close_vec(mesh.corner_normal(0, 0), [0.0, 0.0, 1.0]));
    }

    #[test]
    fn missing_normals_fall_back_to_face_normals() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 3\n";
        let mesh = ObjMesh::parse(obj.as_bytes()).expect("parses");
        assert!(close_vec(mesh.face_normal(0), [0.0, 0.0, 1.0]));
        assert!(close_vec(mesh.corner_normal(0, 1), [0.0, 0.0, 1.0]));
        // A degenerate triangle has a defined zero normal, not NaN.
        let degenerate = "v 0 0 0\nv 0 0 0\nv 0 0 0\nf 1 2 3\n";
        let mesh = ObjMesh::parse(degenerate.as_bytes()).expect("parses");
        assert_eq!(mesh.face_normal(0), [0.0; 3]);
    }

    #[test]
    fn negative_indices_are_relative() {
        let obj = "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n";
        let mesh = ObjMesh::parse(obj.as_bytes()).expect("parses");
        assert_eq!(mesh.triangles[0][0].vertex, 0);
        assert_eq!(mesh.triangles[0][1].vertex, 1);
        assert_eq!(mesh.triangles[0][2].vertex, 2);
    }

    #[test]
    fn polygons_fan_triangulate() {
        let quad = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\n";
        let mesh = ObjMesh::parse(quad.as_bytes()).expect("parses");
        assert_eq!(mesh.triangle_count(), 2);
        assert_eq!(mesh.triangles[0][0].vertex, 0);
        assert_eq!(mesh.triangles[1][0].vertex, 0, "fan root repeats");
        assert_eq!(mesh.triangles[1][1].vertex, 2);
        assert_eq!(mesh.triangles[1][2].vertex, 3);
        assert_eq!(mesh.triangles[1][2].tex_coord, Some(3));
    }

    #[test]
    fn malformed_inputs_name_their_errors() {
        // A non-finite coordinate ("nan" parses as f64 — must still refuse).
        let err = ObjMesh::parse(b"v 0 nan 0\n").expect_err("refused");
        assert!(matches!(
            err,
            ObjError::BadFloat {
                line: 1,
                statement: "v",
                ..
            }
        ));
        // A short vertex.
        let err = ObjMesh::parse(b"v 0 1\n").expect_err("refused");
        assert!(matches!(err, ObjError::BadFloat { line: 1, .. }));
        // A non-integer index.
        let err = ObjMesh::parse(b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 two 3\n").expect_err("refused");
        assert!(matches!(err, ObjError::BadIndex { line: 4, .. }));
        // Zero index (OBJ is 1-based).
        let err = ObjMesh::parse(b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 0 3\n").expect_err("refused");
        assert!(matches!(
            err,
            ObjError::IndexOutOfRange {
                line: 4,
                index: 0,
                len: 3
            }
        ));
        // Past-the-end index.
        let err = ObjMesh::parse(b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 2 9\n").expect_err("refused");
        assert!(matches!(err, ObjError::IndexOutOfRange { index: 9, .. }));
        // A two-corner face.
        let err = ObjMesh::parse(b"v 0 0 0\nv 1 0 0\nf 1 2\n").expect_err("refused");
        assert_eq!(
            err,
            ObjError::FaceTooSmall {
                line: 3,
                corners: 2
            }
        );
        // Invalid UTF-8 names the byte offset.
        let err = ObjMesh::parse(b"v 0 0 \xff\n").expect_err("refused");
        assert!(matches!(err, ObjError::NotUtf8 { .. }));
        // Every error text is precise (the fuzz campaign's bar).
        for input in [
            &b"v 0 nan 0\n"[..],
            b"v 0 1\n",
            b"v 0 0 0\nv 1 0 0\nv 0 1 0\nf 1 two 3\n",
            b"v 0 0 0\nv 1 0 0\nf 1 2\n",
            b"v 0 0 \xff\n",
        ] {
            let err = ObjMesh::parse(input).expect_err("refused");
            let message = err.to_string();
            assert!(!message.is_empty());
            assert!(message.contains("line") || message.contains("offset"));
        }
    }

    #[test]
    fn budgets_refuse_with_named_errors() {
        let limits = ObjLimits {
            max_vertices: 2,
            ..ObjLimits::default()
        };
        let err = ObjMesh::parse_with_limits(b"v 0 0 0\nv 1 0 0\nv 0 1 0\n", &limits)
            .expect_err("refused");
        assert_eq!(err, ObjError::TooManyVertices { line: 3, limit: 2 });
        let limits = ObjLimits {
            max_triangles: 1,
            ..ObjLimits::default()
        };
        let quad = "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nf 1 2 3 4\n";
        let err = ObjMesh::parse_with_limits(quad.as_bytes(), &limits).expect_err("refused");
        assert_eq!(err, ObjError::TooManyTriangles { line: 5, limit: 1 });
    }

    #[test]
    fn skipped_statements_are_tolerated() {
        let obj = "o thing\ng group\ns off\nusemtl mat\nmtllib lib.mtl\nl 1 2\np 1\nvp 0.5\n\
                   cstype rat bspline\ncurv 0 1 1 2\n\
                   v 0 0 0 1.0\nv 1 0 0\nv 0 1 0\nvt 0.5\nf 1/1 2 3\n";
        let mesh = ObjMesh::parse(obj.as_bytes()).expect("parses");
        assert_eq!(mesh.vertex_count(), 3);
        assert_eq!(mesh.tex_coords, vec![[0.5, 0.0]]);
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn crlf_and_comments_are_handled() {
        let obj = "v 0 0 0 # trailing comment\r\nv 1 0 0\r\nv 0 1 0\r\n# full line\r\nf 1 2 3\r\n";
        let mesh = ObjMesh::parse(obj.as_bytes()).expect("parses");
        assert_eq!(mesh.triangle_count(), 1);
    }

    #[test]
    fn three_d_model_normalizes_height_and_centers() {
        let model = ThreeDModel::from_obj(TETRAHEDRON.as_bytes()).expect("parses");
        let mob = Mobject::from(model);
        // 4 triangles × 3 corners of the Surface dtype.
        assert_eq!(mob.buffer.len(), 12);
        let names: Vec<&str> = mob
            .buffer
            .schema()
            .fields()
            .iter()
            .map(|f| f.name.as_str())
            .collect();
        assert_eq!(names, ["point", "d_normal_point", "rgba"]);
        // Height normalization: the tetrahedron's y-extent is 2, so the
        // scaled extent is the default model height of 3.
        let points = mob.buffer.read_column("point").expect("field");
        let ys: Vec<f32> = points.iter().skip(1).step_by(3).copied().collect();
        let (min, max) = ys
            .iter()
            .fold((f32::INFINITY, f32::NEG_INFINITY), |(lo, hi), &y| {
                (lo.min(y), hi.max(y))
            });
        assert!((f64::from(max - min) - DEFAULT_MODEL_HEIGHT).abs() < 1e-5);
        // Centered: the bbox midpoint lands on the origin.
        assert!((f64::from(max + min) / 2.0).abs() < 1e-5);
        // Normals ride the d_normal_point offset (first corner: vn 1,0,0).
        let point = mob.buffer.read(0, "point").expect("field");
        let dnp = mob.buffer.read(0, "d_normal_point").expect("field");
        assert!((dnp[0] - point[0] - 1.0).abs() < 1e-5);
        assert!((dnp[1] - point[1]).abs() < 1e-5);
        // Surface shading and depth test land in the uniform inventory.
        assert_eq!(mob.uniforms.shading, MODEL_SHADING);
        assert!(mob.uniforms.depth_test);
    }

    // ---- structure-aware fuzz: the parser never panics -------------------

    /// A small deterministic PRNG for the fuzz-style tests (xorshift64;
    /// fixed seed — no wall clock, no entropy).
    struct XorShift64(u64);

    impl XorShift64 {
        fn next(&mut self) -> u64 {
            let mut x = self.0;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            self.0 = x;
            x
        }

        fn below(&mut self, n: u64) -> u64 {
            self.next() % n
        }
    }

    /// Perturb a valid OBJ document in a structure-aware way: digit swaps
    /// in coordinates, index inflation/deflation, token corruption, line
    /// duplication, truncation, and byte flips.
    fn mutate(rng: &mut XorShift64, input: &mut Vec<u8>) {
        match rng.below(8) {
            0 => {
                // Digit perturbation: rewrite a random ASCII digit.
                let digits: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|(_, b)| b.is_ascii_digit())
                    .map(|(i, _)| i)
                    .collect();
                if !digits.is_empty() {
                    let at = digits[rng.below(digits.len() as u64) as usize];
                    input[at] = b'0' + (rng.below(10) as u8);
                }
            }
            1 => {
                // Truncate at a random line boundary (byte 10 is '\n').
                let lines: Vec<usize> = input
                    .iter()
                    .enumerate()
                    .filter(|&(_, b)| *b == 10)
                    .map(|(i, _)| i)
                    .collect();
                if !lines.is_empty() {
                    input.truncate(lines[rng.below(lines.len() as u64) as usize]);
                }
            }
            2 => {
                // Duplicate a random line.
                let text = String::from_utf8_lossy(input).into_owned();
                let lines: Vec<&str> = text.lines().collect();
                if !lines.is_empty() {
                    let line = lines[rng.below(lines.len() as u64) as usize];
                    let at = rng.below(input.len() as u64) as usize;
                    let mut insertion = line.as_bytes().to_vec();
                    insertion.push(b'\n');
                    input.splice(at..at, insertion);
                }
            }
            3 => {
                // Flip a random byte.
                if !input.is_empty() {
                    let at = rng.below(input.len() as u64) as usize;
                    input[at] ^= 1u8 << (rng.below(8) as u8);
                }
            }
            4 => {
                // Corrupt a random statement keyword.
                let statements: Vec<usize> = input
                    .windows(2)
                    .enumerate()
                    .filter(|(_, w)| matches!(*w, b"v " | b"f " | b"vn" | b"vt"))
                    .map(|(i, _)| i)
                    .collect();
                if !statements.is_empty() {
                    let at = statements[rng.below(statements.len() as u64) as usize];
                    input[at] = b'a' + (rng.below(26) as u8);
                }
            }
            5 => {
                // Splice a random chunk.
                if input.len() > 4 {
                    let at = rng.below((input.len() - 2) as u64) as usize;
                    let len = 1 + rng.below(4) as usize;
                    let end = (at + len).min(input.len());
                    input.splice(at..end, [b'/'; 1]);
                }
            }
            6 => {
                // Sign flip on the first '-' (0x2D).
                if let Some(at) = input.iter().position(|&b| matches!(b, 0x2D)) {
                    input[at] = b'+';
                }
            }
            _ => {
                // Inject a malformed statement line.
                let injection: &[u8] = match rng.below(4) {
                    0 => b"v nan 0 0\n",
                    1 => b"f 0 0 0\n",
                    2 => b"f 1/2/3/4 1 1\n",
                    _ => b"v 1 2\n",
                };
                let at = rng.below(input.len() as u64 + 1) as usize;
                input.splice(at..at, injection.iter().copied());
            }
        }
    }

    #[test]
    fn obj_parser_never_panics_under_structure_aware_mutation() {
        const CASES: u32 = 2_000;
        let seeds: [&str; 3] = [
            TETRAHEDRON,
            "v 0 0 0\nv 1 0 0\nv 1 1 0\nv 0 1 0\nvt 0 0\nvt 1 0\nvt 1 1\nvt 0 1\nf 1/1 2/2 3/3 4/4\n",
            "v 0 0 0\nv 1 0 0\nv 0 1 0\nf -3 -2 -1\n",
        ];
        let mut rng = XorShift64(0x0b1e_c7a5_5eed_0001);
        for case in 0..CASES {
            let mut input = seeds[rng.below(seeds.len() as u64) as usize]
                .as_bytes()
                .to_vec();
            let mutations = 1 + rng.below(6);
            for _ in 0..mutations {
                mutate(&mut rng, &mut input);
            }
            // Totality: every input is a typed verdict, never a panic
            // (a panic here fails the test by unwinding).
            let verdict = ObjMesh::parse_with_limits(
                &input,
                &ObjLimits {
                    max_vertices: 1 << 12,
                    max_tex_coords: 1 << 12,
                    max_normals: 1 << 12,
                    max_triangles: 1 << 13,
                },
            );
            match verdict {
                Ok(mesh) => {
                    // Accepted parses honor the budgets they were run under.
                    assert!(
                        mesh.vertex_count() <= 1 << 12,
                        "case {case}: budget breached"
                    );
                    assert!(mesh.triangle_count() <= 1 << 13);
                    // Every referenced index is in range by construction.
                    for triangle in &mesh.triangles {
                        for corner in triangle {
                            assert!(corner.vertex < mesh.vertex_count());
                            if let Some(n) = corner.normal {
                                assert!(n < mesh.normals.len());
                            }
                            if let Some(t) = corner.tex_coord {
                                assert!(t < mesh.tex_coords.len());
                            }
                        }
                    }
                }
                Err(err) => {
                    // Refusals are precise: non-empty and line-locating
                    // (the fuzz campaign's precision bar).
                    let message = err.to_string();
                    assert!(!message.is_empty(), "case {case}: empty error");
                    assert!(
                        message.contains("line") || message.contains("offset"),
                        "case {case}: imprecise error {message:?}"
                    );
                }
            }
        }
    }
}
